// video.h: a film from a file (whatever GStreamer can read: it chooses the parts itself), put in the scene as a thing like any other: a flat sheet
// whose look is the film's picture at a moment (flat.mat, as for ThorVG's pictures: so it can be cut by mattes, seen
// through glass, and keyed: `green`).
// GStreamer reads the file and has the machine's video decoder make the pictures; a picture stays where the decoder
// left it, on the GPU (on the Mac an IOSurface), and is shared with Filament's own WebGPU device as it is (Dawn's
// SharedTextureMemory): its light and its colour are two planes of one texture, turned to red, green and blue in
// flat.mat. Nothing of a picture goes through the CPU. A picture is chosen by the time the film gives it (a phone's
// film does not keep an even pace), and a film that says it is to be turned (shot upright) is turned.
// Only on the WebGPU backend (BACKEND=webgpu, dawn_probe/kit_wgpu.sh with KIT_EXTRA: see puppet.sh), with Filament
// asking its device for the sharing (filament_dawn_compute.patch). The Mac's door only, so far.
#include "dawn_probe/gpu_door.h"
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include <gst/video/video.h>
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
#include <webgpu/webgpu.h>
#include <atomic>
#include <map>
#include <string>

struct Video {
    GstElement* reader = nullptr;      // GStreamer's pipeline for this film
    GstAppSink* out = nullptr;
    GstSample* held = nullptr;         // the picture for now (held, so that the decoder does not write over it)
    GstSample* next = nullptr;         // the one after it, once it has come: it says until when `held` is the picture
    GstSample* on = nullptr;           // the picture that is on the sheet (`held`, but for the moment a surface is first shared)
    double nextAt = 0.0;               // when `next` is due, in seconds of the film's run (on through its repeats)
    double origin = 0.0, last = 0.0;   // the stage's time at which that run began; the stage's time last asked for
    int w = 0, h = 0, turned = 0;      // turned: quarter turns clockwise the film asks for before it is shown
    float seconds = 0.f;               // how long it is as it is shown (its cut, at its speed)
    float starts = 0.f;                // how far into its cut it is when the stage's time is 0
    float from = 0.f, to = 0.f, speed = 1.f;   // the cut of the file that is shown, in the file's seconds, and how fast (below 0: backwards)
    std::atomic<bool> ended{false};    // the reader has read to the cut's end: the mark of it has come down the stream of pictures, after the last of them
    float4 key{0.f};                   // its green screen, if it has one (green); until its first picture is on the sheet the sheet is all clear
    float2 keySoft{0.f};
    Mesh* sheet = nullptr;             // the thing in the scene: moved like any other (kit.h: move)
    MaterialInstance* look = nullptr;
    struct Planes { std::atomic<WGPUTexture> shared{nullptr}; WGPUSharedTextureMemory lent = nullptr; Texture* light = nullptr; Texture* colour = nullptr; };
    std::map<uint32_t, Planes> seen;   // the decoder's surfaces come round again: each is shared once
};

// The decoder's picture inside one of GStreamer's buffers: its Mac plugin hangs it there under this name, in this
// layout (gst-plugins-bad, sys/applemedia/corevideobuffer.h, which is not installed for others to include).
struct VideoCoreVideoMeta { GstMeta meta; CVBufferRef cvbuf; CVPixelBufferRef pixbuf; };

static const char* video_trouble = "";   // why the last film that could not be shown could not
static bool video_counted = false;
static long video_asked = 0, video_late = 0;   // how often a film was asked for a moment, and how often it could not yet tell its picture for it

// When a picture is due, in seconds of the film's run: its own time stamp, on through the film's repeats.
static double video_due(GstSample* picture) {
    const GstClockTime at = gst_segment_to_running_time(gst_sample_get_segment(picture), GST_FORMAT_TIME, GST_BUFFER_PTS(gst_sample_get_buffer(picture)));
    return GST_CLOCK_TIME_IS_VALID(at) ? (double)at / (double)GST_SECOND : 0.0;
}

// The held picture put on the sheet. A surface seen for the first time is first shared with Filament's device, on
// Filament's own thread and without waiting for it: until that is done (a picture or two later) the sheet keeps the
// picture it had. false: not on the sheet yet.
static CVPixelBufferRef video_surface(GstSample* picture) {
    const GType api = g_type_from_name("GstCoreVideoMetaAPI");
    VideoCoreVideoMeta* meta = api ? (VideoCoreVideoMeta*)gst_buffer_get_meta(gst_sample_get_buffer(picture), api) : nullptr;
    return meta ? meta->pixbuf : nullptr;
}
static bool video_show(Stage& s, Video* v) {
    CVPixelBufferRef held = video_surface(v->held);
    IOSurfaceRef surface = held ? CVPixelBufferGetIOSurface(held) : nullptr;
    if (!surface) return false;
    auto found = v->seen.find(IOSurfaceGetID(surface));
    if (found == v->seen.end()) {
        Video::Planes* planes = &v->seen[IOSurfaceGetID(surface)];
        CFRetain(surface);
        const uint32_t w = (uint32_t)v->w, h = (uint32_t)v->h;
        Engine* engine = s.engine;
        door::on_driver(engine, [planes, surface, w, h, engine] {
            WGPUSharedTextureMemoryIOSurfaceDescriptor from{};
            from.chain.sType = WGPUSType_SharedTextureMemoryIOSurfaceDescriptor;
            from.ioSurface = surface;
            WGPUSharedTextureMemoryDescriptor what{};
            what.nextInChain = &from.chain;
            WGPUSharedTextureMemory memory = wgpuDeviceImportSharedTextureMemory((WGPUDevice)door::device(engine), &what);
            WGPUTextureDescriptor like{};
            like.size = {w, h, 1};
            like.format = WGPUTextureFormat_R8BG8Biplanar420Unorm;   // (light, and colour at half the size: what a decoder leaves)
            like.usage = WGPUTextureUsage_TextureBinding;
            like.dimension = WGPUTextureDimension_2D;
            like.mipLevelCount = 1;
            like.sampleCount = 1;
            WGPUTexture shared = wgpuSharedTextureMemoryCreateTexture(memory, &like);
            // (lent for good: the decoder writes a surface only while nobody holds its picture, and the one shown is held)
            WGPUSharedTextureMemoryBeginAccessDescriptor lend{};
            lend.initialized = true;
            wgpuSharedTextureMemoryBeginAccess(memory, shared, &lend);
            CFRelease(surface);
            planes->lent = memory;
            planes->shared = shared;
        });
        return false;
    }
    Video::Planes& planes = found->second;
    WGPUTexture shared = planes.shared;
    if (!shared) return false;   // (being shared)
    if (!planes.light) {
        const auto plane = [&](Texture::InternalFormat as, int w, int h) {
            return Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels(1).sampler(Texture::Sampler::SAMPLER_2D).format(as)
                .usage(Texture::Usage::SAMPLEABLE).import((intptr_t)shared).build(*s.engine);
        };
        planes.light = plane(Texture::InternalFormat::R8, v->w, v->h);
        planes.colour = plane(Texture::InternalFormat::RG8, v->w / 2, v->h / 2);
    }
    const TextureSampler smooth(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
    v->look->setParameter("picture", planes.light, smooth);
    v->look->setParameter("colours", planes.colour, smooth);
    if (!v->on) {   // (its first picture: seen from now on)
        v->look->setParameter("key", v->key);
        v->look->setParameter("keySoft", v->keySoft);
    }
    if (v->on && v->on != v->held) gst_sample_unref(v->on);
    v->on = v->held;
    return true;
}

// The held picture on the sheet for the picture drawn next: a film made picture by picture waits for a surface that
// is being shared (a moment, once for each of the decoder's surfaces); one played by the clock does not.
static void video_put(Stage& s, Video* v) {
    if (v->on == v->held || video_show(s, v) || s.by_clock) return;
    s.engine->flushAndWait();
    video_show(s, v);
}

// The reader set to a moment of the film as it is shown (`seconds` into its cut, at its speed), to go on from there
// and round again without a break at the cut's end (GStreamer's own: a segment with a rate, which ends with a word
// on the bus instead of the end of the stream; backwards it runs from the segment's end down to its start).
static void video_from(Video* v, float seconds, bool afresh) {
    const double at = (double)seconds * std::fabs((double)v->speed);   // (that far into the cut, in the file's seconds)
    const double start = v->speed > 0.f ? (double)v->from + at : (double)v->from, stop = v->speed > 0.f ? (double)v->to : (double)v->to - at;
    gst_element_seek(v->reader, (double)v->speed, GST_FORMAT_TIME, (GstSeekFlags)((afresh ? GST_SEEK_FLAG_FLUSH | GST_SEEK_FLAG_ACCURATE : 0) | GST_SEEK_FLAG_SEGMENT),
                     GST_SEEK_TYPE_SET, (gint64)(start * (double)GST_SECOND), GST_SEEK_TYPE_SET, (gint64)(stop * (double)GST_SECOND));
}

// The mark that the cut is read to its end, seen as it comes down the stream of pictures (GStreamer also says so on
// its bus, but only when every stream of the file has ended, and played backwards the sound's never does).
static GstPadProbeReturn video_ending(GstPad*, GstPadProbeInfo* info, gpointer film) {
    if (GST_EVENT_TYPE(GST_PAD_PROBE_INFO_EVENT(info)) == GST_EVENT_SEGMENT_DONE) ((Video*)film)->ended = true;
    return GST_PAD_PROBE_OK;
}

static void video_at(Stage& s, Video* v, float seconds);

// Which of the file's streams the reader reads: its pictures only (GStreamer asks, stream by stream). The sound is
// not read at all. (Read and dropped it cost 51 decoders their work, and an HEVC film played slower than it was shot
// then gave no picture at all: so GStreamer alone, 1.28.8, with both streams taken.)
static gint video_wanted(GstElement*, GstStreamCollection*, GstStream* stream, gpointer) {
    return (gst_stream_get_stream_type(stream) & GST_STREAM_TYPE_VIDEO) ? 1 : 0;
}

// The stream of pictures the reader found in the file goes to `out`.
static void video_found(GstElement*, GstPad* stream, gpointer film) {
    GstPad* to = gst_element_get_static_pad(GST_ELEMENT(((Video*)film)->out), "sink");
    if (!gst_pad_is_linked(to)) gst_pad_link(stream, to);
    gst_object_unref(to);
}

// The film in `file` put in the scene: a sheet `high` tall (as wide as the film makes it) with its middle at `place`,
// facing along the place's z. from, to: the cut of it that is shown, in seconds of the file (to below 0: to its end);
// speed: how fast (2: twice as fast, 0.5: half, below 0: backwards).
static Video* video(Stage& s, const char* file, const mat4f& place, float high, float from = 0.f, float to = -1.f, float speed = 1.f, const char* lay = nullptr) {
    if (!door::device(s.engine)) return video_trouble = "not the WebGPU backend", nullptr;   // (another backend)
    static const bool started = (gst_init(nullptr, nullptr), true);
    (void)started;
    Video* v = new Video;
    v->reader = gst_pipeline_new(nullptr);
    GstElement* opens = gst_element_factory_make("filesrc", nullptr);
    GstElement* reads = gst_element_factory_make("decodebin3", nullptr);   // (chooses the parts for whatever the file is: the machine's decoder where there is one)
    v->out = GST_APP_SINK(gst_element_factory_make("appsink", nullptr));
    if (!opens || !reads || !v->out) return video_trouble = "GStreamer lacks a part (filesrc, decodebin3 or appsink)", nullptr;
    GstCaps* wanted = gst_caps_from_string("video/x-raw,format=NV12");   // (light and colour apart, eight bits: what video.h shares)
    g_object_set(opens, "location", file, nullptr);
    g_object_set(v->out, "sync", FALSE, "max-buffers", 2, "caps", wanted, nullptr);
    gst_caps_unref(wanted);
    gst_bin_add_many(GST_BIN(v->reader), opens, reads, GST_ELEMENT(v->out), nullptr);
    gst_element_link(opens, reads);
    g_signal_connect(reads, "select-stream", G_CALLBACK(video_wanted), nullptr);
    g_signal_connect(reads, "pad-added", G_CALLBACK(video_found), v);
    GstPad* in = gst_element_get_static_pad(GST_ELEMENT(v->out), "sink");
    gst_pad_add_probe(in, GST_PAD_PROBE_TYPE_EVENT_DOWNSTREAM, video_ending, v, nullptr);
    gst_object_unref(in);
    gst_element_set_state(v->reader, GST_STATE_PAUSED);
    gst_element_get_state(v->reader, nullptr, nullptr, 5 * GST_SECOND);   // (waits until the file is open and its first picture is there)
    gint64 length = 0;
    if (!gst_element_query_duration(v->reader, GST_FORMAT_TIME, &length) || length <= 0) return video_trouble = "the file does not open as a film (no length)", nullptr;
    v->from = std::max(0.f, from);
    v->to = to < 0.f ? (float)((double)length / (double)GST_SECOND) : std::min(to, (float)((double)length / (double)GST_SECOND));
    if (speed == 0.f) return video_trouble = "a speed of 0 shows nothing: a film goes forwards or backwards", nullptr;
    v->speed = speed;
    v->seconds = (v->to - v->from) / std::fabs(v->speed);
    if (v->seconds <= 0.f) return video_trouble = "the cut is empty", nullptr;
    video_from(v, 0.f, true);
    gst_element_set_state(v->reader, GST_STATE_PLAYING);
    v->held = gst_app_sink_try_pull_sample(v->out, 5 * GST_SECOND);   // (its first picture says how large it is)
    if (!v->held) return video_trouble = "no picture came within 5 seconds", nullptr;
    const GstStructure* said = gst_caps_get_structure(gst_sample_get_caps(v->held), 0);
    if (!gst_structure_get_int(said, "width", &v->w) || !gst_structure_get_int(said, "height", &v->h)) return video_trouble = "the picture does not say its size", nullptr;
    // the film's own rule for its colours, as GStreamer read it from the file: which two weights (BT.601, BT.709,
    // BT.2020: Kr, Kb), and where black, white and no-colour lie in its numbers (the narrow range or the full one)
    GstVideoInfo told;
    gdouble kr = 0.2126, kb = 0.0722;
    gint zero[GST_VIDEO_MAX_COMPONENTS] = {16, 128, 128}, span[GST_VIDEO_MAX_COMPONENTS] = {219, 224, 224};
    if (gst_video_info_from_caps(&told, gst_sample_get_caps(v->held))) {
        gst_video_color_matrix_get_Kr_Kb(told.colorimetry.matrix, &kr, &kb);
        // (the range is asked of the picture itself where it says: the Mac's decoder hands on the narrow range
        //  whatever the file was in, and the caps go on naming the file's)
        CVPixelBufferRef first = video_surface(v->held);
        const GstVideoColorRange range = !first ? told.colorimetry.range
            : CVPixelBufferGetPixelFormatType(first) == kCVPixelFormatType_420YpCbCr8BiPlanarFullRange ? GST_VIDEO_COLOR_RANGE_0_255 : GST_VIDEO_COLOR_RANGE_16_235;
        gst_video_color_range_offsets(range, told.finfo, zero, span);
    }
    const float kg = 1.f - (float)kr - (float)kb, red = 2.f * (1.f - (float)kr), blue = 2.f * (1.f - (float)kb);
    const float3 gain{255.f / (float)span[0], 255.f / (float)span[1], 255.f / (float)span[2]}, base{-(float)zero[0] / (float)span[0], -(float)zero[1] / (float)span[1], -(float)zero[2] / (float)span[2]};
    const float3 byBlue{0.f, -blue * (float)kb / kg, blue}, byRed{red, -red * (float)kr / kg, 0.f};   // (what its two colour numbers add to red, green, blue)
    const mat4f rule(float4{float3{gain.x}, 0.f}, float4{gain.y * byBlue, 0.f}, float4{gain.z * byRed, 0.f}, float4{float3{base.x} + base.y * byBlue + base.z * byRed, 1.f});
    GstBus* bus = gst_element_get_bus(v->reader);
    while (GstMessage* said = gst_bus_pop_filtered(bus, GST_MESSAGE_TAG)) {   // (a film shot upright says which way it is to be turned)
        GstTagList* tags = nullptr;
        GstVideoOrientationMethod way = GST_VIDEO_ORIENTATION_IDENTITY;
        gst_message_parse_tag(said, &tags);
        if (gst_video_orientation_from_tag(tags, &way)) v->turned = way == GST_VIDEO_ORIENTATION_90R ? 1 : way == GST_VIDEO_ORIENTATION_180 ? 2 : way == GST_VIDEO_ORIENTATION_90L ? 3 : 0;
        gst_tag_list_unref(tags);
        gst_message_unref(said);
    }
    gst_object_unref(bus);
    const float wide = v->turned % 2 ? (float)v->h / (float)v->w : (float)v->w / (float)v->h;   // (as it is shown)
    v->sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        v->sheet->pos.push_back({0.5f * high * wide * c.x, 0.5f * high * c.y, 0.f});
        v->sheet->nrm.push_back({0, 0, 1});
        // which point of the film is at this corner: the corner turned back the other way
        const float2 of = v->turned == 1 ? float2{-c.y, c.x} : v->turned == 2 ? float2{-c.x, -c.y} : v->turned == 3 ? float2{c.y, -c.x} : c;
        v->sheet->uv.push_back(of * 0.5f + 0.5f);   // (a film's first row is its top: Filament turns a sheet's v over itself)
    }
    v->sheet->idx = {0, 1, 2, 0, 2, 3};
    v->look = add(s, v->sheet, flat_laid(lay).c_str(), {place});
    if (!v->look) return video_trouble = "flat.filamat", nullptr;
    v->look->setParameter("filmed", 1.f);
    v->look->setParameter("filmRule", rule);
    v->look->setParameter("picture", white_point(s), TextureSampler());   // (until its first picture is shared: a white that is
    v->look->setParameter("key", float4{1.f, 0.5f, 1.f, 9.f});            //  keyed out whole, so the sheet is not seen)
    v->look->setParameter("keySoft", float2{1.f, 1.f});
    video_put(s, v);
    s.playing[v] = [&s, v](double now) { video_at(s, v, (float)now + v->starts); };   // (it goes on by itself as the stage's time does)
    if (s.said_after.empty() || !video_counted) {
        video_counted = true;
        s.said_after.push_back([] {   // (of the times a film was asked for a moment in that stretch, how often it could not yet tell its picture)
            std::printf("    films late %ld of %ld times asked (%.2f %%)\n", video_late, video_asked, video_asked ? 100.0 * (double)video_late / (double)video_asked : 0.0);
            video_asked = video_late = 0;
        });
    }
    return v;
}

// A film taken off the stage for good: its sheet gone, its reader stopped and let go.
static void video_off(Stage& s, Video* v) {
    s.playing.erase(v);
    take_off(s, v->sheet);
    s.engine->flushAndWait();   // (a surface may be on its way to being shared)
    gst_element_set_state(v->reader, GST_STATE_NULL);
    if (v->next) gst_sample_unref(v->next);
    if (v->held && v->held != v->on) gst_sample_unref(v->held);
    if (v->on) gst_sample_unref(v->on);
    gst_object_unref(v->reader);
    for (auto& [surface, planes] : v->seen) {
        if (planes.light) s.engine->destroy(planes.light);
        if (planes.colour) s.engine->destroy(planes.colour);
    }
    door::on_driver(s.engine, [v] {   // (every surface that was lent is given back: Dawn's own end of a lend)
        for (auto& [surface, planes] : v->seen) {
            if (!planes.shared) continue;
            WGPUSharedTextureMemoryEndAccessState ended{};
            wgpuSharedTextureMemoryEndAccess(planes.lent, planes.shared, &ended);
            wgpuSharedTextureMemoryEndAccessStateFreeMembers(ended);
            wgpuTextureRelease(planes.shared);
            wgpuSharedTextureMemoryRelease(planes.lent);
        }
    });
    s.engine->flushAndWait();
    delete v;
}

// A colour of the film made clear (kit.h: key_out), green by default: a green screen. Until the film's first picture is
// on the sheet the sheet stays all clear.
static void key_out(Video* v, float3 colour = float3{0.f, 1.f, 0.f}, float near = 0.12f, float soft = 0.3f, float spill = 0.9f) {
    v->key = float4{colour, near};
    v->keySoft = float2{soft, spill};
    if (v->on) key_out(v->look, colour, near, soft, spill);
}

// The film at its moment, `seconds` of the stage's time (it goes round again when it ends), for the picture that comes
// next. The picture for a moment is the last one due by then, by the film's own time stamps.
static void video_at(Stage& s, Video* v, float seconds) {
    if (seconds < v->last) {   // (time went back: from that moment afresh. Asked on this thread: tried from another,
        video_from(v, std::fmod(seconds, v->seconds), true);   //  it left out as many pictures and the films took longer to settle)
        v->origin = seconds;
        v->ended = false;
        if (v->next) { gst_sample_unref(v->next); v->next = nullptr; }
    }
    v->last = seconds;
    const double now = (double)seconds - v->origin;
    // (a stage played by the clock does not wait for a film: one that is not ready keeps its picture for that frame and
    //  catches up at the next; made picture by picture, every picture is waited for, up to half a second)
    const gint64 until = g_get_monotonic_time() + (s.by_clock ? 0 : 500000);
    bool sure = false;   // (known: the picture held is the one for now)
    while (true) {
        if (!v->next && (v->next = gst_app_sink_try_pull_sample(v->out, s.by_clock ? 0 : 2 * GST_MSECOND))) v->nextAt = video_due(v->next);
        if (v->next) {
            if (v->nextAt > now) { sure = true; break; }
            if (v->held != v->on) gst_sample_unref(v->held);   // (the next one is due: it is the picture now)
            v->held = v->next;
            v->next = nullptr;
            continue;
        }
        if (v->ended) {   // (round again, without a break. Asked from another thread, GStreamer's own way: the reader
            v->ended = false;   //  cannot be turned back while it waits to hand a picture on, and this thread is the one that takes them)
            gst_element_call_async(v->reader, [](GstElement*, gpointer film) { video_from((Video*)film, 0.f, false); }, v, nullptr);
            continue;
        }
        if (g_get_monotonic_time() >= until) break;
    }
    ++video_asked;
    video_late += !sure;
    video_put(s, v);
}
