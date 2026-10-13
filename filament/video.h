// video.h: a film from a file (an .mp4 with H.264 in it), put in the scene as a thing like any other: a flat sheet
// whose look is the film's picture at a moment (flat.mat, as for ThorVG's pictures: so it can be cut by mattes, seen
// through glass, and keyed: `green`).
// GStreamer reads the file and has the machine's video decoder make the pictures; a picture stays where the decoder
// left it, on the GPU (on the Mac an IOSurface), and is shared with Filament's own WebGPU device as it is (Dawn's
// SharedTextureMemory): its light and its colour are two planes of one texture, turned to red, green and blue in
// flat.mat. Nothing of a picture goes through the CPU.
// Only on the WebGPU backend (BACKEND=webgpu, dawn_probe/kit_wgpu.sh with KIT_EXTRA: see puppet.sh), with Filament
// asking its device for the sharing (filament_dawn_compute.patch). The Mac's door only, so far.
#include "dawn_probe/gpu_door.h"
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
#include <webgpu/webgpu.h>
#include <atomic>
#include <map>
#include <string>

struct Video {
    GstElement* reader = nullptr;      // GStreamer's pipeline for this film
    GstAppSink* out = nullptr;
    GstSample* held = nullptr;         // the picture last taken from the reader (held, so that the decoder does not write over it)
    GstSample* on = nullptr;           // the picture that is on the sheet (the same, but for the moment a surface is first shared)
    int w = 0, h = 0, count = 0, shown = -1;   // count: its pictures in all; shown: which is held, counted on through its repeats
    float each = 0.f, seconds = 0.f;   // pictures a second, and how long it is
    bool ended = false;                // the reader has read to the film's end (its last pictures may still be on their way)
    Mesh* sheet = nullptr;             // the thing in the scene: moved like any other (kit.h: move)
    MaterialInstance* look = nullptr;
    struct Planes { std::atomic<WGPUTexture> shared{nullptr}; Texture* light = nullptr; Texture* colour = nullptr; };
    std::map<uint32_t, Planes> seen;   // the decoder's surfaces come round again: each is shared once
};

// The decoder's picture inside one of GStreamer's buffers: its Mac plugin hangs it there under this name, in this
// layout (gst-plugins-bad, sys/applemedia/corevideobuffer.h, which is not installed for others to include).
struct VideoCoreVideoMeta { GstMeta meta; CVBufferRef cvbuf; CVPixelBufferRef pixbuf; };

// How long a film may be waited for when its next picture is not there yet, in thousandths of a second. A film made
// picture by picture wants every one (the default); a stage that plays as the clock goes sets 0: a film that is not
// ready keeps its last picture for that one and catches up at the next.
static float video_patience = 500.f;
static long video_asked = 0, video_late = 0;   // how often a film was asked for a moment, and how often its picture for it was not there

// The film's next picture taken from the reader; false if it is not there (within the patience left).
static bool video_pull(Video* v, GstClockTime wait) {
    GstSample* next = gst_app_sink_try_pull_sample(v->out, wait);
    if (!next) return false;
    if (v->held && v->held != v->on) gst_sample_unref(v->held);
    v->held = next;
    ++v->shown;
    return true;
}

// The held picture put on the sheet. A surface seen for the first time is first shared with Filament's device, on
// Filament's own thread and without waiting for it: until that is done (a picture or two later) the sheet keeps the
// picture it had. false: not on the sheet yet.
static bool video_show(Stage& s, Video* v) {
    const GType api = g_type_from_name("GstCoreVideoMetaAPI");
    VideoCoreVideoMeta* meta = api ? (VideoCoreVideoMeta*)gst_buffer_get_meta(gst_sample_get_buffer(v->held), api) : nullptr;
    IOSurfaceRef surface = meta && meta->pixbuf ? CVPixelBufferGetIOSurface(meta->pixbuf) : nullptr;
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
    if (v->on && v->on != v->held) gst_sample_unref(v->on);
    v->on = v->held;
    return true;
}

// The reader set to a moment of the film, to go on from there and round again without a break at the film's end
// (GStreamer's own: a segment, which ends with a word on the bus instead of the end of the stream).
static void video_from(Video* v, float seconds, bool afresh) {
    gst_element_seek(v->reader, 1.0, GST_FORMAT_TIME, (GstSeekFlags)((afresh ? GST_SEEK_FLAG_FLUSH | GST_SEEK_FLAG_ACCURATE : 0) | GST_SEEK_FLAG_SEGMENT),
                     GST_SEEK_TYPE_SET, (gint64)((double)seconds * (double)GST_SECOND), GST_SEEK_TYPE_NONE, -1);
}

// The film in `file` put in the scene: a sheet `high` tall (as wide as the film makes it) with its middle at `place`,
// facing along the place's z.
static Video* video(Stage& s, const char* file, const mat4f& place, float high) {
    if (!door::device(s.engine)) return nullptr;   // (another backend)
    static const bool started = (gst_init(nullptr, nullptr), true);
    (void)started;
    Video* v = new Video;
    gchar* line = g_strdup_printf("filesrc location=\"%s\" ! qtdemux ! h264parse ! vtdec_hw ! video/x-raw,format=NV12 ! appsink name=out sync=false max-buffers=2", file);
    v->reader = gst_parse_launch(line, nullptr);
    g_free(line);
    if (!v->reader) return nullptr;
    v->out = GST_APP_SINK(gst_bin_get_by_name(GST_BIN(v->reader), "out"));
    gst_element_set_state(v->reader, GST_STATE_PAUSED);
    gst_element_get_state(v->reader, nullptr, nullptr, GST_CLOCK_TIME_NONE);   // (waits until the file is open and its first picture is there)
    video_from(v, 0.f, true);
    gst_element_set_state(v->reader, GST_STATE_PLAYING);
    if (!video_pull(v, 5 * GST_SECOND)) return nullptr;   // (its first picture says how large it is and how fast it goes)
    const GstStructure* said = gst_caps_get_structure(gst_sample_get_caps(v->held), 0);
    int num = 0, den = 1;
    gint64 length = 0;
    if (!gst_structure_get_int(said, "width", &v->w) || !gst_structure_get_int(said, "height", &v->h) || !gst_structure_get_fraction(said, "framerate", &num, &den) ||
        num <= 0 || !gst_element_query_duration(v->reader, GST_FORMAT_TIME, &length) || length <= 0) return nullptr;
    v->each = (float)num / (float)den;
    v->seconds = (float)((double)length / (double)GST_SECOND);
    v->count = std::max(1, (int)std::lround(v->seconds * v->each));
    v->sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        v->sheet->pos.push_back({0.5f * high * (float)v->w / (float)v->h * c.x, 0.5f * high * c.y, 0.f});
        v->sheet->nrm.push_back({0, 0, 1});
        v->sheet->uv.push_back(c * 0.5f + 0.5f);   // (a film's first row is its top: Filament turns a sheet's v over itself)
    }
    v->sheet->idx = {0, 1, 2, 0, 2, 3};
    v->look = add(s, v->sheet, "flat.filamat", {place});
    if (!v->look) return nullptr;
    v->look->setParameter("filmed", 1.f);
    v->look->setParameter("picture", white_point(s), TextureSampler());   // (until its first picture is shared)
    video_show(s, v);
    return v;
}

// A green screen: where the film is this colour it is clear. near: how near to the colour counts as the screen itself
// (in the two colour numbers of video, where pure green is 0.6 from grey: the screen's own noise must fit in it);
// soft: over how much more the film comes in (a point half screen, half grey is 0.3 from the screen);
// spill: how far from the screen its colour is still taken out of what stays.
static void green(Video* v, float3 colour = float3{0.f, 1.f, 0.f}, float near = 0.12f, float soft = 0.3f, float spill = 0.9f) {
    v->look->setParameter("key", float4{colour, near});
    v->look->setParameter("keySoft", float2{soft, spill});
}

// The film at its moment, `seconds` from its start (it goes round again when it ends), for the picture that comes next.
static void video_at(Stage& s, Video* v, float seconds) {
    const int want = (int)std::floor(seconds * v->each);   // (counted on through its repeats)
    if (want < v->shown) {   // (time went back: from that moment afresh)
        video_from(v, (float)(want % v->count) / v->each, true);
        v->shown = want - 1;
    }
    GstBus* bus = gst_element_get_bus(v->reader);
    const gint64 until = g_get_monotonic_time() + (gint64)(video_patience * 1000.f);
    while (true) {
        while (GstMessage* said = gst_bus_pop_filtered(bus, GST_MESSAGE_SEGMENT_DONE)) {
            gst_message_unref(said);
            v->ended = true;
        }
        if (v->shown >= want) break;
        if (video_pull(v, video_patience > 0.f ? 2 * GST_MSECOND : 0)) continue;
        if (v->ended) {   // (round again, without a break. Asked from another thread, GStreamer's own way: the reader
            v->ended = false;   //  cannot be turned back while it waits to hand a picture on, and this thread is the one that takes them)
            gst_element_call_async(v->reader, [](GstElement*, gpointer film) { video_from((Video*)film, 0.f, false); }, v, nullptr);
            continue;
        }
        if (g_get_monotonic_time() >= until) break;
    }
    gst_object_unref(bus);
    ++video_asked;
    video_late += v->shown < want;
    if (v->on != v->held) video_show(s, v);
}
