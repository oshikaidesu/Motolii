// drawn.h: a picture ThorVG draws, put in the scene as a thing like any other. The scene and its things are
// Filament's (an entity with a place and a look); this only makes one whose look is ThorVG's picture: a flat sheet.
// The picture is a file: an After Effects film (Lottie) with all it holds, or an SVG. ThorVG draws it with WebGPU on
// Filament's own device straight into a texture Filament reads: nothing goes through the CPU, nothing is copied.
// Only on the WebGPU backend (BACKEND=webgpu, dawn_probe/kit_wgpu.sh), with ThorVG built against Dawn
// (dawn_probe/tvg_on_dawn.sh) and Filament patched to lend its device and take a texture (gpu_door.h).
// All of ThorVG's work is queued like Filament's own calls and runs on Filament's driver thread.
#include "dawn_probe/gpu_door.h"
#include <thorvg.h>
#include <thorvg_lottie.h>
#include <tesselator.h>
#include <webgpu/webgpu.h>

#include <array>
#include <set>
#include <condition_variable>
#include <cstring>
#include <deque>
#include <filesystem>
#include <fstream>
#include <mutex>
#include <thread>
#include "third_party/filament_src/third_party/zstd/lib/zstd.h"

// tint: how much its darks and lights are made two colours (0: not at all); tritone: how much its darks, middles and
// lights are made three (0: not at all).
struct DrawnFx { bool split = false;   // (a text's letters each a copy of their own)
    bool together = false;   // (its copies drawn in one go, each an instance of one thing: none coloured or cleared each of its own, kit.h add_parts)
    float letters = 1.f, lines = 1.f, italic = 0.f, wrapWidth = 0.f; int wrap = 0;   // (a text's lines broken in a box so many of its letters' heights wide, ThorVG's layout and wrap; 0: as typed)
    float dash = 0.f, dashFrom = 0.f; int cap = -1, join = -1;   // (ThorVG's own: Text::spacing, italic; Shape::strokeDash, strokeCap, strokeJoin; -1: as it comes)
    int heads = 0; float headSize = 4.f;   // (arrow heads on an open edge's ends, as Cavalry's basicArrowHead: 1 its end, 2 its start, 3 both; so many times its edge long)
    float blur = 0.f; int blurWay = 0; float overHigh = 1.f; float4 shadow{0.f}; float fall = 0.f, way = 0.f, soft = 0.f; float tint = 0.f; float3 tints[2]{}; float tritone = 0.f; float3 tones[3]{}; float4 glow{0.f}; float glowSoft = 0.f; int copies = 1;
                 std::vector<float3> ramp;   // (two colours or more: it is filled from the first at its top to the last at its bottom, ThorVG's linear gradient)
                 bool hollow = false;   // (a flat shape filled with nothing: only its edge is seen)
                 float reach = 0.f;     // (how far a bent shape goes out past its box at its most, a share of its height (bend_at): the picture larger all round by that)
                 bool radial = false; float rampAngle = 0.f, rampScale = 1.f; int spread = 0;   // (or from its middle out, ThorVG's radial gradient; turned so far, clockwise; so long, as a share of its reach; past its ends padded, reflected or repeated)
                 DrawnFx over(float high) const {   // (the same for a thing `high` units high: its lengths as shares of that)
                     DrawnFx f = *this;
                     f.blur /= high, f.fall /= high, f.soft /= high, f.glowSoft /= high, f.overHigh = high;
                     return f;
                 } };

struct Drawn {
    tvg::WgCanvas* canvas = nullptr;   // (touched on the driver thread only)
    WGPUTexture made = nullptr;        // (the texture it draws into, which a frame kept is copied from; the same)
    // A film's frames as drawn, kept on the GPU, so that a frame drawn once is shown again, not drawn again (After
    // Effects' RAM preview, a layer's own; kept_drop throws them away when what it draws changes): by frame (main
    // thread), and the copies just made (the driver thread puts them in `keeping`, the main thread takes them)
    std::map<int, Texture*> kept;
    std::vector<WGPUTexture> keptMade;   // (each kept frame's own texture, let go of when they are thrown away: Filament takes it later, on its thread)
    std::mutex keepingLock;
    std::vector<std::pair<int, WGPUTexture>> keeping;
    int shownAt = -1;
    int madeAt = -1;                   // (the frame its own texture holds; driver thread only)
    std::set<int> ahead;               // (frames being put onto disk only, while the hand rests: picture_ahead; main thread)
    std::string keyBase;               // (what its frames are known by on disk: its file as it is; none, not kept on disk)
    std::atomic<int> keepAge{0};       // (one more each time its kept frames are thrown away: a copy made before is not kept)
    tvg::Animation* film = nullptr;    // (the same)
    Texture* texture = nullptr;        // what Filament reads
    Mesh* sheet = nullptr;             // the thing in the scene: moved like any other (kit.h: move)
    MaterialInstance* look = nullptr;  // its look (none if it is not shown)
    float seconds = 0.f, frames = 0.f; // a film's length (a still: 0)
    float starts = 0.f;                // how far into itself it is when the stage's time is 0
    tvg::Shape* shape = nullptr;       // a flat shape's own path (driver thread only), and how much of it is cut off its end
    float3 trimmed{0.f};   // (as last cut: off the end, off the beginning, slid along)
    std::vector<tvg::PathCommand> plain; std::vector<tvg::Point> plainAt;   // (its path as made, before it is bent (bend_at))
    float plainHigh = 0.f, plainSide = 0.f;   // (and how high it was drawn and its shorter side, in its own points)
    float pictureHigh = 0.f;   // (how high its picture is, room and all, in the same points)
    float fine = 0.f;          // (its lines cut in pieces no longer than so many points)
    bool cut = false;          // (a flat shape drawn by Filament as its own triangles (flat_mesh): no picture, no canvas; bent in its look)
    float4 wash{0.f};          // (its own colour laid over its look, as a copy's is: a shape cut in triangles is filled so)
    float wideBy = 1.f;        // (how many times as wide as it is high: what its bends are counted across)
    tvg::Shape* outline = nullptr;   // (a shape in triangles: its path, ThorVG's, kept on the CPU only, never drawn)
    float cutW = 0.f, sheetHigh = 1.f, edgeWide = 0.f;   // (its path's width in points; how high it stands on the stage; its edge's width in points)
    bool hollow = false;
    uint32_t fillPoints = 0, fillEnd = 0;   // (where its fill's points and triangles end in its sheet: its edge comes after)
    MaterialInstance* edgeLook = nullptr;   // (its edge's own look: the second part of its triangles)
    tvg::Animation* source = nullptr;   // (one layer of an After Effects file drawn alone, layer_picture: the file, the layer, where it is held, how it is fitted)
    uint32_t layerId = 0;
    tvg::Scene* holder = nullptr;
    tvg::Matrix fit{1, 0, 0, 0, 1, 0, 0, 0, 1};
    tvg::LottieAnimation* lottie = nullptr;   // (its film, when it is an After Effects file: what says its layers' poses)
    std::vector<uint32_t> deep;   // (an After Effects file's 3D layers: not drawn in its own picture, each a sheet of its own where it is: picture_deep)
    std::array<float, 8> bent{};   // (as last bent: round, bloat, zigzag, ridges, smooth, offset; roughened: how far, how fine)
    // Another shape's path in this one's points, and what is done with it: put together (how: skpath.cpp's op; -1 none),
    // or turned into (much: how far, 0 not).
    struct Other {
        int how = -1; float much = 0.f; std::vector<tvg::PathCommand> by; std::vector<tvg::Point> at;
        bool operator==(const Other& o) const {
            return how == o.how && much == o.much && by == o.by && at.size() == o.at.size() && std::equal(at.begin(), at.end(), o.at.begin(), [](tvg::Point a, tvg::Point b) { return a.x == b.x && a.y == b.y; });
        }
    } joined, morphed;   // (as last put together with, as last turned into)
    std::vector<std::string> markers;  // an After Effects file's named stretches (ThorVG's Lottie markers)
    std::vector<float2> letters;       // a text split into its letters: where each lies across the picture (shares of its width); its copies show one each
    float fromFrame = 0.f, toFrame = 0.f;   // the stretch it plays, when one is named (frames of the file)
    std::vector<tvg::Scene*> effects;  // ThorVG's scene its effects are on (driver thread only)
    float tall = 0.f;                  // how high it was drawn, in its own points: what its effects' lengths are shares of
    DrawnFx fx;                        // its effects as last told
};

static void kept_drop(Stage& s, Drawn* d);

// ThorVG's own effects on its scene, in the order they are laid: its colours first (tint, tritone), then its shadow,
// its glow, and last its blur; lengths as shares of `tall` points.
static void effects_on(const std::vector<tvg::Scene*>& chain, const DrawnFx& fx, float tall) {
    const float blur = fx.blur * tall, fall = fx.fall * tall, soft = fx.soft * tall, glowSoft = fx.glowSoft * tall;
    const auto c = [](float v) { return (int)std::lround(255.f * v); };
    tvg::Scene* on = chain[0];
    on->add(tvg::SceneEffect::Clear);
    if (fx.tint > 0.f) on->add(tvg::SceneEffect::Tint, c(fx.tints[0].x), c(fx.tints[0].y), c(fx.tints[0].z), c(fx.tints[1].x), c(fx.tints[1].y), c(fx.tints[1].z), (double)(100.f * fx.tint));
    if (fx.tritone > 0.f)
        on->add(tvg::SceneEffect::Tritone, c(fx.tones[0].x), c(fx.tones[0].y), c(fx.tones[0].z), c(fx.tones[1].x), c(fx.tones[1].y), c(fx.tones[1].z), c(fx.tones[2].x), c(fx.tones[2].y), c(fx.tones[2].z), c(1.f - fx.tritone));
    if (fx.shadow.w > 0.f) on->add(tvg::SceneEffect::DropShadow, c(fx.shadow.x), c(fx.shadow.y), c(fx.shadow.z), c(fx.shadow.w), (double)fx.way, (double)fall, (double)std::max(soft, 0.01f), 75);
    if (fx.glow.w > 0.f)   // (a glow is ThorVG's drop shadow that does not fall: a light all round it)
        on->add(tvg::SceneEffect::DropShadow, c(fx.glow.x), c(fx.glow.y), c(fx.glow.z), c(fx.glow.w), 0.0, 0.0, (double)std::max(glowSoft, 0.01f), 75);
    if (blur > 0.f) on->add(tvg::SceneEffect::GaussianBlur, (double)blur, fx.blurWay, 0, 75);   // (its way: both, across, up and down)
}

// A flat shape with so much of its path cut off the end and so much off its beginning, the part left slid along it by
// `offset` of the way round (ThorVG's own trimpath, which goes round past either end: After Effects' trim paths),
// drawn again for the frame that comes next; nothing is done if it is cut as it was.
static void trim_at(Stage& s, Drawn* d, float cut, float begin = 0.f, float offset = 0.f) {
    const float3 is{cut, begin, offset};
    if (!d->shape || (is == d->trimmed)) return;
    d->trimmed = is;
    kept_drop(s, d);
    door::on_driver(s.engine, [d, cut, begin, offset] {
        d->shape->trimpath(std::clamp(begin, 0.f, 1.f) + offset, 1.f - std::clamp(cut, 0.f, 1.f) + offset);
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
}

// After Effects' path operators, ThorVG's own (bend.cpp): its corners rounded (round: a share of its shorter side),
// pucked in or bloated out (bloat, -1 to 1), zig-zagged (so high, a share of its height; so many ridges a side; smooth
// or sharp), offset out or in (a share of its height); drawn again for the frame that comes next, if it is bent anew.
void tvg_bend(tvg::Shape* shape, const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, uint32_t ptsCnt,
              float round, float bloat, float zigzag, int ridges, bool smooth, float offset, float fine);
static void bend_at(Stage& s, Drawn* d, float round, float bloat, float zigzag, float ridges, float smooth, float offset) {
    const std::array<float, 8> is{round, bloat, zigzag, ridges, smooth, offset, 0.f, 0.f};
    if (!d->shape || d->plain.empty() || is == d->bent) return;
    d->bent = is;
    kept_drop(s, d);
    door::on_driver(s.engine, [d, is] {
        const float high = d->plainHigh, side = d->plainSide;
        tvg_bend(d->shape, d->plain.data(), (uint32_t)d->plain.size(), d->plainAt.data(), (uint32_t)d->plainAt.size(), std::clamp(is[0], 0.f, 0.5f) * side,
                 100.f * std::clamp(is[1], -1.f, 1.f), std::clamp(is[2], -4.f, 4.f) * high, (int)std::clamp(is[3], 0.f, 100.f), is[4] > 0.5f, std::clamp(is[5], -4.f, 4.f) * high, d->fine);
        // (a shape bent to more than sixty-four times its picture is not drawn: none of it would be seen, and ThorVG's own
        //  measuring, dashing and staging of so much once took the window down)
        float x = 0.f, y = 0.f, w = 0.f, h = 0.f;
        if (d->shape->bounds(&x, &y, &w, &h) == tvg::Result::Success && !(std::max(w, h) < 64.f * std::max(1.f, d->pictureHigh))) d->shape->reset();
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
}

// A flat thing's effects as they are now (lengths in shares of its height, as it was made), drawn again for the frame
// that comes next; nothing is done if they are as they were. They reach no further than they did at their most.
static void fx_at(Stage& s, Drawn* d, const DrawnFx& told) {
    if (d->effects.empty()) return;
    const DrawnFx fx = told.over(d->fx.overHigh);
    const DrawnFx& was = d->fx;
    if (fx.blur == was.blur && fx.blurWay == was.blurWay && (fx.shadow == was.shadow) && fx.fall == was.fall && fx.way == was.way && fx.soft == was.soft && fx.tint == was.tint && fx.tritone == was.tritone
        && (fx.glow == was.glow) && fx.glowSoft == was.glowSoft && (fx.tints[0] == was.tints[0]) && (fx.tints[1] == was.tints[1]) && (fx.tones[0] == was.tones[0]) && (fx.tones[1] == was.tones[1]) && (fx.tones[2] == was.tones[2]))
        return;
    d->fx = fx;
    kept_drop(s, d);
    door::on_driver(s.engine, [d, fx] {
        effects_on(d->effects, fx, d->tall);
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
}

// A font for the pictures put after it (an SVG names its fonts; ThorVG knows only the ones it is given).
static void font(Stage& s, const char* path) {
    door::on_driver(s.engine, [path] {
        tvg::Initializer::init(0);
        tvg::Text::load(path);
    });
}

// Kept frames on disk too (~/Library/Caches/Motolii/frames), so that a document opened again shows a film's frames drawn
// before without drawing them: compressed by zstd (as Filament builds it), a file a frame, named by a hash of what made
// it (its file as it is, how large it is drawn, its effects, its stretch, the frame), as Nuke names its cache by what goes
// in; past 20 GB the oldest files go, as the folder is first used.
static const std::filesystem::path& kept_dir() {
    static const std::filesystem::path dir = [] {
        const char* home = std::getenv("HOME");
        const std::filesystem::path p = std::filesystem::path(home ? home : "/tmp") / "Library/Caches/Motolii/frames";
        std::error_code e;
        std::filesystem::create_directories(p, e);
        std::thread([p] {
            std::vector<std::pair<std::filesystem::file_time_type, std::filesystem::path>> files;
            uintmax_t total = 0;
            std::error_code e;
            for (const auto& f : std::filesystem::directory_iterator(p, e))
                if (f.is_regular_file(e)) files.push_back({f.last_write_time(e), f.path()}), total += f.file_size(e);
            std::sort(files.begin(), files.end());
            for (const auto& [when, f] : files) {
                if (total <= (uintmax_t(20) << 30)) break;
                total -= std::filesystem::file_size(f, e);
                std::filesystem::remove(f, e);
            }
        }).detach();
        return p;
    }();
    return dir;
}
static std::filesystem::path kept_file(const Drawn* d, int at) {
    if (d->keyBase.empty() || !d->made) return {};
    std::string all = d->keyBase;
    const auto add = [&](float v) { all += "|" + std::to_string(v); };
    const DrawnFx& f = d->fx;
    add((float)wgpuTextureGetWidth(d->made)), add((float)wgpuTextureGetHeight(d->made));
    for (float v : {f.blur, (float)f.blurWay, f.shadow.x, f.shadow.y, f.shadow.z, f.shadow.w, f.fall, f.way, f.soft, f.tint, f.tritone, f.glow.x, f.glow.y, f.glow.z, f.glow.w, f.glowSoft, d->fromFrame, d->toFrame}) add(v);
    for (const float3& c : f.tints) add(c.x), add(c.y), add(c.z);
    for (const float3& c : f.tones) add(c.x), add(c.y), add(c.z);
    for (uint32_t id : d->deep) add((float)id);
    uint64_t hash = 1469598103934665603ull;   // (FNV-1a)
    for (unsigned char c : all) hash = (hash ^ c) * 1099511628211ull;
    char name[64];
    std::snprintf(name, sizeof name, "%016llx_%05d.zst", (unsigned long long)hash, at);
    return kept_dir() / name;
}
static std::mutex kept_writing_lock;
static std::condition_variable kept_writing_wake;
static std::deque<std::pair<std::filesystem::path, std::vector<uint8_t>>> kept_writing;
static std::atomic<int> kept_unwritten{0};   // (frames on their way to disk: read back or waiting to be written)
static void kept_write(std::filesystem::path file, std::vector<uint8_t> pixels) {   // (on a thread of its own: compressed, written, then named)
    static std::once_flag started;
    std::call_once(started, [] {
        std::thread([] {
            for (;;) {
                std::unique_lock<std::mutex> hold(kept_writing_lock);
                kept_writing_wake.wait(hold, [] { return !kept_writing.empty(); });
                auto job = std::move(kept_writing.front());
                kept_writing.pop_front();
                hold.unlock();
                std::vector<uint8_t> packed(ZSTD_compressBound(job.second.size()));
                const size_t n = ZSTD_compress(packed.data(), packed.size(), job.second.data(), job.second.size(), 1);
                if (ZSTD_isError(n)) { --kept_unwritten; continue; }
                std::filesystem::path part = job.first;
                part += ".part";
                if (FILE* f = std::fopen(part.c_str(), "wb")) {
                    const bool whole = std::fwrite(packed.data(), 1, n, f) == n;
                    std::fclose(f);
                    std::error_code e;
                    if (whole) std::filesystem::rename(part, job.first, e);
                    else std::filesystem::remove(part, e);
                }
                --kept_unwritten;
            }
        }).detach();
        std::atexit([] {   // (what is on its way is written before the program ends, for at most ten seconds)
            for (int i = 0; i < 1000 && kept_unwritten > 0; ++i) std::this_thread::sleep_for(std::chrono::milliseconds(10));
        });
    });
    {
        std::lock_guard<std::mutex> hold(kept_writing_lock);
        kept_writing.push_back({std::move(file), std::move(pixels)});
    }
    kept_writing_wake.notify_one();
}
static bool kept_read(const std::filesystem::path& file, std::vector<uint8_t>& pixels, size_t size) {
    FILE* f = file.empty() ? nullptr : std::fopen(file.c_str(), "rb");
    if (!f) return false;
    std::fseek(f, 0, SEEK_END);
    std::vector<uint8_t> packed((size_t)std::max(0L, std::ftell(f)));
    std::fseek(f, 0, SEEK_SET);
    const bool whole = std::fread(packed.data(), 1, packed.size(), f) == packed.size();
    std::fclose(f);
    pixels.resize(size);
    const size_t n = whole ? ZSTD_decompress(pixels.data(), size, packed.data(), packed.size()) : 0;
    return whole && !ZSTD_isError(n) && n == size;
}
// A kept frame's texture read back to the CPU (as the GPU finishes it: Dawn's MapAsync) and written to disk.
struct KeptReadBack { WGPUBuffer buffer; uint32_t w, h, row; std::filesystem::path file; };
static KeptReadBack* kept_read_back(WGPUDevice device, WGPUCommandEncoder encoder, WGPUTexture texture, std::filesystem::path file) {
    const uint32_t w = wgpuTextureGetWidth(texture), h = wgpuTextureGetHeight(texture), row = (4 * w + 255) / 256 * 256;
    WGPUBufferDescriptor like{};
    like.size = (uint64_t)row * h;
    like.usage = WGPUBufferUsage_CopyDst | WGPUBufferUsage_MapRead;
    const WGPUBuffer buffer = wgpuDeviceCreateBuffer(device, &like);
    WGPUTexelCopyTextureInfo from{};
    from.texture = texture;
    WGPUTexelCopyBufferInfo to{};
    to.buffer = buffer;
    to.layout.bytesPerRow = row, to.layout.rowsPerImage = h;
    const WGPUExtent3D size{w, h, 1};
    wgpuCommandEncoderCopyTextureToBuffer(encoder, &from, &to, &size);
    ++kept_unwritten;
    return new KeptReadBack{buffer, w, h, row, std::move(file)};
}
static void kept_map(KeptReadBack* job) {   // (once the encoder that copies it is submitted)
    WGPUBufferMapCallbackInfo done{};
    done.mode = WGPUCallbackMode_AllowSpontaneous;
    done.callback = [](WGPUMapAsyncStatus status, WGPUStringView, void* one, void*) {
        auto* job = (KeptReadBack*)one;
        if (status == WGPUMapAsyncStatus_Success) {
            const auto* at = (const uint8_t*)wgpuBufferGetConstMappedRange(job->buffer, 0, (size_t)job->row * job->h);
            std::vector<uint8_t> pixels((size_t)4 * job->w * job->h);
            if (at)
                for (uint32_t y = 0; y < job->h; ++y) std::memcpy(pixels.data() + (size_t)4 * job->w * y, at + (size_t)job->row * y, (size_t)4 * job->w);
            wgpuBufferUnmap(job->buffer);
            if (at) kept_write(std::move(job->file), std::move(pixels));
            else --kept_unwritten;
        } else {
            --kept_unwritten;
        }
        wgpuBufferRelease(job->buffer);
        delete job;
    };
    done.userdata1 = job;
    wgpuBufferMapAsync(job->buffer, WGPUMapMode_Read, 0, (size_t)job->row * job->h, done);
}

// The frames a film keeps, thrown away (what it draws has changed); all films' kept frames are held to so many bytes.
static std::atomic<size_t> kept_bytes{0};
static constexpr size_t kept_most = size_t(2) << 30;   // (2 GB)
static size_t kept_one(const Drawn* d) { return d->made ? 4 * (size_t)wgpuTextureGetWidth(d->made) * wgpuTextureGetHeight(d->made) : 0; }
static void kept_drop(Stage& s, Drawn* d) {
    {
        std::lock_guard<std::mutex> hold(d->keepingLock);
        for (auto& [at, made] : d->keeping) d->kept[at] = nullptr, wgpuTextureRelease(made);
        d->keeping.clear();
    }
    const size_t one = d->made ? 4 * (size_t)wgpuTextureGetWidth(d->made) * wgpuTextureGetHeight(d->made) : 0;
    for (auto& [at, texture] : d->kept) {
        if (texture) s.engine->destroy(texture);
        kept_bytes -= one;
    }
    d->kept.clear();
    d->ahead.clear();
    door::on_driver(s.engine, [made = std::move(d->keptMade)] { for (WGPUTexture t : made) wgpuTextureRelease(t); });   // (after Filament has taken and let go of them)
    d->keptMade.clear();
    d->shownAt = -1;
    ++d->keepAge;
    if (d->look) d->look->setParameter("picture", d->texture, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
}

// The kept frames the driver thread has made since, taken as Filament's (main thread).
static void kept_take(Stage& s, Drawn* d) {
    std::lock_guard<std::mutex> hold(d->keepingLock);
    for (auto& [frame, made] : d->keeping) {
        d->kept[frame] = Texture::Builder().width(wgpuTextureGetWidth(made)).height(wgpuTextureGetHeight(made)).levels(1).sampler(Texture::Sampler::SAMPLER_2D)
                             .format(Texture::InternalFormat::RGBA8).usage(Texture::Usage::SAMPLEABLE).import((intptr_t)made).build(*s.engine);
        d->keptMade.push_back(made);
    }
    d->keeping.clear();
}
// A frame read from disk put up (driver thread): into the texture it draws into (`shown`: it is what is seen now) and,
// if `keep`, into one of its own that keeps it.
static void kept_put(Stage& s, Drawn* d, int at, const std::vector<uint8_t>& pixels, bool shown, bool keep, int age) {
    const uint32_t w = wgpuTextureGetWidth(d->made), h = wgpuTextureGetHeight(d->made);
    const WGPUDevice device = (WGPUDevice)door::device(s.engine);
    WGPUTextureDescriptor like{};
    like.size = {w, h, 1};
    like.format = wgpuTextureGetFormat(d->made);
    like.usage = WGPUTextureUsage_TextureBinding | WGPUTextureUsage_CopyDst | WGPUTextureUsage_CopySrc;
    like.dimension = WGPUTextureDimension_2D;
    like.mipLevelCount = 1;
    like.sampleCount = 1;
    const WGPUTexture copy = keep ? wgpuDeviceCreateTexture(device, &like) : nullptr;
    WGPUTexelCopyBufferLayout rows{};
    rows.bytesPerRow = 4 * w, rows.rowsPerImage = h;
    for (WGPUTexture t : {shown ? d->made : nullptr, copy}) {
        if (!t) continue;
        WGPUTexelCopyTextureInfo to{};
        to.texture = t;
        wgpuQueueWriteTexture(wgpuDeviceGetQueue(device), &to, pixels.data(), pixels.size(), &rows, &like.size);
    }
    if (shown) d->madeAt = at;
    if (!copy) return;
    std::lock_guard<std::mutex> hold(d->keepingLock);
    if (age == d->keepAge) d->keeping.push_back({at, copy});
    else wgpuTextureRelease(copy);
}
// A frame drawn by ThorVG into the texture it draws into (driver thread), and, if `keep`, copied on the GPU to a
// texture of its own that keeps it; and onto disk, once, if `file` is not there yet.
static void kept_draw(Stage& s, Drawn* d, int at, bool keep, const std::filesystem::path& file, int age) {
    if (d->film->frame(d->fromFrame + (float)at) == tvg::Result::Success || d->madeAt != at) {
        for (uint32_t id : d->deep)   // (its 3D layers are sheets of their own: not drawn flat here)
            if (const tvg::Paint* one = d->film->picture()->paint(id)) const_cast<tvg::Paint*>(one)->visible(false);
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
        d->madeAt = at;
    }
    std::error_code e;
    const bool toDisk = d->look && !file.empty() && !std::filesystem::exists(file, e);
    if (!keep && !toDisk) return;
    const WGPUDevice device = (WGPUDevice)door::device(s.engine);
    WGPUCommandEncoder encoder = wgpuDeviceCreateCommandEncoder(device, nullptr);
    WGPUTexture copy = nullptr;
    if (keep) {
        WGPUTextureDescriptor like{};
        like.size = {wgpuTextureGetWidth(d->made), wgpuTextureGetHeight(d->made), 1};
        like.format = wgpuTextureGetFormat(d->made);
        like.usage = WGPUTextureUsage_TextureBinding | WGPUTextureUsage_CopyDst | WGPUTextureUsage_CopySrc;
        like.dimension = WGPUTextureDimension_2D;
        like.mipLevelCount = 1;
        like.sampleCount = 1;
        copy = wgpuDeviceCreateTexture(device, &like);
        WGPUTexelCopyTextureInfo from{}, to{};
        from.texture = d->made, to.texture = copy;
        wgpuCommandEncoderCopyTextureToTexture(encoder, &from, &to, &like.size);
    }
    KeptReadBack* back = toDisk ? kept_read_back(device, encoder, copy ? copy : d->made, file) : nullptr;
    WGPUCommandBuffer commands = wgpuCommandEncoderFinish(encoder, nullptr);
    wgpuQueueSubmit(wgpuDeviceGetQueue(device), 1, &commands);
    wgpuCommandBufferRelease(commands);
    wgpuCommandEncoderRelease(encoder);
    if (back) kept_map(back);
    if (!copy) return;
    std::lock_guard<std::mutex> hold(d->keepingLock);
    if (age == d->keepAge) d->keeping.push_back({at, copy});
    else wgpuTextureRelease(copy);   // (what it draws changed while this was drawn)
}

// A film at its moment, `seconds` from its start (after its end it holds its last picture), for the frame that comes next:
// at its own frames, as After Effects plays it, so that a moment within the frame already drawn draws nothing again
// (ThorVG's frame() says when the frame is the same; the ThorVG author's own advice for Lottie on a GPU, Godot #91580).
// A frame drawn once is shown again from where it was kept, on the GPU or on disk.
static void picture_at(Stage& s, Drawn* d, float seconds) {
    const int at = (int)std::floor(std::min(seconds / d->seconds * d->frames, d->frames - 1.f) + 1e-3f);
    if (at == d->shownAt) return;
    d->shownAt = at;
    const TextureSampler smooth(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    kept_take(s, d);
    if (const auto k = d->kept.find(at); k != d->kept.end() && k->second) {   // (drawn before: shown as kept)
        if (d->look) d->look->setParameter("picture", k->second, smooth);
        return;
    }
    if (d->look) d->look->setParameter("picture", d->texture, smooth);
    const bool keep = d->look && d->kept.find(at) == d->kept.end() && kept_bytes < kept_most;
    if (keep) d->kept[at] = nullptr, kept_bytes += kept_one(d);   // (being kept)
    const std::filesystem::path file = kept_file(d, at);
    auto pixels = std::make_shared<std::vector<uint8_t>>();
    if (d->look && kept_read(file, *pixels, kept_one(d)))   // (drawn before, on disk: put up as it is, not drawn)
        door::on_driver(s.engine, [&s, d, at, pixels, keep, age = d->keepAge.load()] { kept_put(s, d, at, *pixels, true, keep, age); });
    else
        door::on_driver(s.engine, [&s, d, at, keep, file, age = d->keepAge.load()] { kept_draw(s, d, at, keep, file, age); });
}

// While the hand rests: a few of a film's frames after the one shown, kept before they are asked for (as Final Cut
// Pro renders once the hand stops, Resolve's smart cache, After Effects' Cache Frames When Idle): read from disk if
// there, else drawn into the texture it draws into (free while the frame shown is shown from a kept one). How many were
// begun (0: nothing more can be done for it now).
static int picture_ahead(Stage& s, Drawn* d, int many) {
    if (!d->look || !d->film || d->frames <= 1.f || d->shownAt < 0 || !d->made) return 0;
    kept_take(s, d);
    const auto now = d->kept.find(d->shownAt);
    if (now == d->kept.end() || !now->second) return 0;   // (the frame shown is not yet shown from a kept one: the texture it draws into is in use)
    d->look->setParameter("picture", now->second, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    const int frames = (int)d->frames;
    int begun = 0;
    std::error_code e;
    for (int i = 1; i < frames && begun < many; ++i) {
        const int at = (d->shownAt + i) % frames;
        if (d->kept.count(at) || d->ahead.count(at)) continue;
        const std::filesystem::path file = kept_file(d, at);
        const bool keep = kept_bytes + kept_one(d) <= kept_most;
        const bool onDisk = !file.empty() && std::filesystem::exists(file, e);
        if (!keep && (onDisk || file.empty())) continue;   // (no room on the GPU, and on disk already: nothing to do)
        if (keep) d->kept[at] = nullptr, kept_bytes += kept_one(d);
        else d->ahead.insert(at);   // (only onto disk: not asked again while it is written)
        auto pixels = std::make_shared<std::vector<uint8_t>>();
        if (onDisk && kept_read(file, *pixels, kept_one(d)))
            door::on_driver(s.engine, [&s, d, at, pixels, age = d->keepAge.load()] { kept_put(s, d, at, *pixels, false, true, age); });
        else
            door::on_driver(s.engine, [&s, d, at, keep, file, age = d->keepAge.load()] { kept_draw(s, d, at, keep, file, age); });
        ++begun;
    }
    return begun;
}

// What `picture` and `type` share: something of ThorVG's (made by `paints` on Filament's own thread, which says how
// many points wide and high it is drawn) on a canvas that draws into a texture of Filament's device, and the sheet
// that shows it. ground below 0: clear where nothing is drawn. shown = false: made but not put in the scene (for a matte).
// What ThorVG's own scene effects do to it, each as a share of its height (0: none): a blur; a shadow of a colour, so
// much of it (shadow.w), falling so far, that way (degrees, as ThorVG counts them) and so soft.
   // (copies: how many times it is put down: the document tells each where)
static Drawn* drawn(Stage& s, const std::function<tvg::Paint*(Drawn*, int*, int*)>& paints, const mat4f& place, float high, float3 ground, bool shown, const char* lay, const DrawnFx& fx = {}) {
    if (!door::device(s.engine)) return nullptr;   // (another backend)
    Drawn* d = new Drawn;
    WGPUTexture made = nullptr;
    tvg::Paint* loose = nullptr;
    bool owned = false;
    int w = 0, h = 0;
    door::on_driver(s.engine, [&] {
        tvg::Initializer::init(0);
        tvg::Paint* p = paints(d, &w, &h);
        if (!p || w <= 0 || h <= 0) return;
        const float tall = (float)h;   // (what its effects' lengths are shares of: the thing as drawn, without room)
        if (fx.reach > 0.f) {
            const float room = std::ceil(fx.reach * (float)h);
            p->translate(room, room);
            high *= ((float)h + 2.f * room) / (float)h;   // (the thing itself stays as large)
            w += 2 * (int)room, h += 2 * (int)room;
        }
        if (fx.blur > 0.f || fx.shadow.w > 0.f || fx.tint > 0.f || fx.tritone > 0.f || fx.glow.w > 0.f) {   // (in a scene of ThorVG's, which takes the effects; the picture larger all round by as far as they reach)
            const float blur = fx.blur * tall, fall = fx.fall * tall, soft = fx.soft * tall;
            const float glowSoft = fx.glowSoft * tall;
            const int room = (int)std::ceil(std::max({3.f * blur, fx.shadow.w > 0.f ? fall + 3.f * soft : 0.f, fx.glow.w > 0.f ? 3.f * glowSoft : 0.f}));
            tvg::Scene* scene = tvg::Scene::gen();   // (one scene takes them all, in order)
            scene->add(p);
            d->effects.push_back(scene);
            p = scene;
            p->translate((float)room, (float)room);
            effects_on(d->effects, fx, tall);
            d->tall = tall, d->fx = fx;   // (told again at each moment: those that move are drawn again)
            high *= (float)(h + 2 * room) / (float)h;   // (the thing itself stays as large)
            w += 2 * room, h += 2 * room;
        }
        d->pictureHigh = (float)h;
        if (const float most = (float)std::max(w, h); most > 4096.f) {   // (no larger than a picture the device takes, with room to spare: the whole drawn smaller, its sheet as large)
            tvg::Scene* smaller = tvg::Scene::gen();
            smaller->add(p);
            smaller->scale(4096.f / most);
            p = smaller;
            w = std::max(2, (int)std::floor((float)w * 4096.f / most)), h = std::max(2, (int)std::floor((float)h * 4096.f / most));
        }
        loose = d->film ? nullptr : p;   // (a paint that no film owns is let go here if the canvas never takes it)
        WGPUTextureDescriptor like{};
        like.size = {(uint32_t)w, (uint32_t)h, 1};
        like.format = WGPUTextureFormat_BGRA8Unorm;   // (what ThorVG's last step writes)
        like.usage = WGPUTextureUsage_RenderAttachment | WGPUTextureUsage_TextureBinding | WGPUTextureUsage_CopySrc | WGPUTextureUsage_CopyDst;   // (CopyDst: a frame kept on disk is put up in it)
        like.dimension = WGPUTextureDimension_2D;
        like.mipLevelCount = 1;
        like.sampleCount = 1;
        made = wgpuDeviceCreateTexture((WGPUDevice)door::device(s.engine), &like);
        d->made = made;
        d->canvas = tvg::WgCanvas::gen();
        if (!d->canvas || d->canvas->target(door::device(s.engine), door::instance(s.engine), made, (uint32_t)w, (uint32_t)h, tvg::ColorSpace::ABGR8888S, 1) != tvg::Result::Success) return;
        if (ground.x >= 0.f) {
            tvg::Shape* under = tvg::Shape::gen();
            under->appendRect(0, 0, (float)w, (float)h);
            under->fill((uint8_t)std::lround(255.f * ground.x), (uint8_t)std::lround(255.f * ground.y), (uint8_t)std::lround(255.f * ground.z), 255);
            d->canvas->add(under);
        }
        d->canvas->add(p);
        owned = true;
        if (d->film) d->frames = d->film->totalFrame(), d->seconds = d->film->duration();
        if (d->film && d->toFrame > d->fromFrame && d->frames > 0.f) d->seconds *= (d->toFrame - d->fromFrame) / d->frames, d->frames = d->toFrame - d->fromFrame;   // (a named stretch of it: so long)
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
    s.engine->flushAndWait();   // (once, at set-up: the lines above ran on the other thread)
    if (!made || !owned) {   // (not made: what was begun is let go)
        door::on_driver(s.engine, [&] { if (loose) tvg::Paint::rel(loose); delete d->canvas; delete d->film; if (made) wgpuTextureRelease(made); });
        s.engine->flushAndWait();
        delete d;
        return nullptr;
    }
    d->texture = Texture::Builder().width((uint32_t)w).height((uint32_t)h).levels(1).sampler(Texture::Sampler::SAMPLER_2D).format(Texture::InternalFormat::RGBA8)
                     .usage(Texture::Usage::SAMPLEABLE).import((intptr_t)made).build(*s.engine);
    d->sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        d->sheet->pos.push_back({0.5f * high * (float)w / (float)h * c.x, 0.5f * high * c.y, 0.f});
        d->sheet->nrm.push_back({0, 0, 1});
        d->sheet->uv.push_back(c * 0.5f + 0.5f);
    }
    d->sheet->idx = {0, 1, 2, 0, 2, 3};
    if (d->frames > 1.f)   // (a film: it goes on by itself as the stage's time does, round and round)
        s.playing[d] = [&s, d](double now) { picture_at(s, d, std::fmod((float)now + d->starts, d->seconds)); };
    if (!shown) return d;
    d->look = add(s, d->sheet, flat_laid(lay).c_str(), std::vector<mat4f>((size_t)std::max(1, d->letters.empty() ? fx.copies : (int)d->letters.size()), place), 1, false, 1, 4, fx.together && d->letters.empty());   // (split into letters: a copy each)
    if (!d->look) return nullptr;
    d->look->setParameter("picture", d->texture, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    return d;
}


// The picture in `file` put in the scene (an After Effects film, an SVG, a photo): a sheet `high` tall (as wide as
// the picture makes it) with its middle at `place`, facing along the place's z, drawn as finely as the stage is.
// The frame an After Effects file begins at (its own "ip", at the top of the file: ThorVG does not say it).
static float lottie_in(const char* file) {
    std::ifstream in(file);
    std::string all((std::istreambuf_iterator<char>(in)), std::istreambuf_iterator<char>());
    int depth = 0;
    bool quoted = false;
    for (size_t i = 0; i < all.size(); ++i) {   // (the key at the outermost depth only: the layers have theirs)
        const char c = all[i];
        if (quoted) { if (c == '\\') ++i; else if (c == '"') quoted = false; continue; }
        if (c == '"') {
            if (depth == 1 && all.compare(i, 4, "\"ip\"") == 0) {
                const size_t colon = all.find(':', i + 4);
                return colon == std::string::npos ? 0.f : std::strtof(all.c_str() + colon + 1, nullptr);
            }
            quoted = true;
        } else if (c == '{' || c == '[') ++depth;
        else if (c == '}' || c == ']') --depth;
    }
    return 0.f;
}

// marker: a named stretch of an After Effects file it plays, over and over (ThorVG's Lottie markers; none: all of it).
static Drawn* picture(Stage& s, const char* file, const mat4f& place, float high, float3 ground = float3{-1.f}, bool shown = true, const char* lay = nullptr, const DrawnFx& fx = {}, const char* marker = "") {
    if (!std::filesystem::is_regular_file(file)) return nullptr;   // (ThorVG ends the program on a folder)
    const std::string ext = std::filesystem::path(file).extension().string();
    const bool lottie = ext == ".json" || ext == ".lot";
    Drawn* made = drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        tvg::LottieAnimation* own = lottie ? tvg::LottieAnimation::gen() : nullptr;
        d->film = own ? own : tvg::Animation::gen();
        d->lottie = own;
        tvg::Picture* p = d->film->picture();
        float fw = 0.f, fh = 0.f;
        if (p->load(file) != tvg::Result::Success || p->size(&fw, &fh) != tvg::Result::Success || fw <= 0.f || fh <= 0.f) return nullptr;
        const float first = lottie ? lottie_in(file) : 0.f;
        std::vector<float> starts;   // (where each named stretch begins: one of no length runs to the next, or to the end)
        for (uint32_t i = 0; own && i < own->markersCnt(); ++i) {
            float from = 0.f, to = 0.f;
            if (own->marker(i, &from, &to)) starts.push_back(from);
        }
        for (uint32_t i = 0; own && i < own->markersCnt(); ++i) {   // (its stretches by name, and the one it plays)
            float from = 0.f, to = 0.f;
            const char* name = own->marker(i, &from, &to);
            if (!name || !*name) continue;
            std::string word(name);
            word.erase(std::remove_if(word.begin(), word.end(), [](char c) { return std::isspace((unsigned char)c); }), word.end());   // (a name is one word: "End\r\n" is End)
            d->markers.push_back(word);
            if (to <= from) {   // (none long: to the next, or to its end)
                to = 1e9f;
                for (float b : starts) if (b > from) to = std::min(to, b);
            }
            if (marker && *marker && word == marker)   // (a marker counts the file's own frames, which begin at its in point; ThorVG counts from 0)
                d->fromFrame = from - first, d->toFrame = std::min(to - first, d->film->totalFrame());
        }
        const float fit = (float)std::max(s.w, s.h) / std::max(fw, fh);
        *w = 2 * (int)std::lround(0.5f * fw * fit);
        *h = 2 * (int)std::lround(0.5f * fh * fit);
        p->size((float)*w, (float)*h);
        return p;
    }, place, high, ground, shown, lay, fx);
    if (made && made->frames > 1.f) {   // (its kept frames known on disk by its file as it is now: kept_file)
        std::error_code e;
        made->keyBase = std::filesystem::absolute(file, e).string() + "|" + std::to_string(std::filesystem::file_size(file, e)) + "|" +
                        std::to_string((long long)std::filesystem::last_write_time(file, e).time_since_epoch().count());
    }
    return made;
}

// Words put in the scene, set by ThorVG in the font of `fontFile` (a .ttf): a sheet `high` tall for each line, as
// wide as the words make it, its middle at `place`.
// The colours of a ramp as ThorVG's linear gradient, from the first at y0 to the last at y1 (evenly spaced).
static tvg::Fill* ramp_of(const std::vector<float3>& ramp, float w, float h, bool radial = false, float angle = 0.f, float scale = 1.f, int spread = 0) {
    tvg::Fill* g = nullptr;
    const float cx = 0.5f * w, cy = 0.5f * h, k = std::max(scale, 0.01f);
    if (radial) {   // (from its middle out to its corners)
        tvg::RadialGradient* r = tvg::RadialGradient::gen();
        const float reach = 0.5f * std::hypot(w, h) * k;
        r->radial(cx, cy, reach, cx, cy, 0.f);
        g = r;
    } else {   // (across it the way it is turned: 0 from its top to its bottom, as far as its box reaches that way)
        tvg::LinearGradient* l = tvg::LinearGradient::gen();
        const float a = angle * (float)TAU / 360.f, dx = std::sin(a), dy = std::cos(a), reach = (0.5f * std::fabs(w * dx) + 0.5f * std::fabs(h * dy)) * k;
        l->linear(cx - dx * reach, cy - dy * reach, cx + dx * reach, cy + dy * reach);
        g = l;
    }
    g->spread((tvg::FillSpread)std::clamp(spread, 0, 2));
    std::vector<tvg::Fill::ColorStop> stops;
    for (size_t i = 0; i < ramp.size(); ++i)
        stops.push_back({(float)i / (float)(ramp.size() - 1), (uint8_t)std::lround(255.f * ramp[i].x), (uint8_t)std::lround(255.f * ramp[i].y), (uint8_t)std::lround(255.f * ramp[i].z), 255});
    g->colorStops(stops.data(), (uint32_t)stops.size());
    return g;
}

// Whether ThorVG sets `words` in a font file: it reads the file and the words come out with a size (a font of other
// letters gives them none). ThorVG's own loader keeps its fonts under a lock, so any thread may ask, once ThorVG is up.
static bool type_sets(const char* fontFile, const char* words) {
    if (!std::filesystem::is_regular_file(fontFile) || tvg::Text::load(fontFile) != tvg::Result::Success) return false;
    tvg::Text* t = tvg::Text::gen();
    t->font(std::filesystem::path(fontFile).stem().string().c_str());
    t->size(100.f);
    t->text(words);
    float x = 0.f, y = 0.f, wide = 0.f, tall = 0.f;
    const bool sets = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success && wide > 0.f && tall > 0.f;
    tvg::Paint::rel(t);
    return sets;
}
// edge: an outline round the letters, ThorVG's own (its colour; w: how wide, a share of the line's height; 0: none).
// Lines: the words may be broken by \n; align: how the lines lie one under another (ThorVG's own: 0 left, 1 right).
static Drawn* type(Stage& s, const char* words, const char* fontFile, const mat4f& place, float high, float3 colour = float3{0.f}, const char* lay = nullptr, float4 edge = float4{0.f}, float align = 0.f, const DrawnFx& fx = {}) {
    if (!std::filesystem::is_regular_file(fontFile)) return nullptr;
    return drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        if (tvg::Text::load(fontFile) != tvg::Result::Success) return nullptr;
        tvg::Text* t = tvg::Text::gen();
        float x = 0.f, y = 0.f, wide = 0.f, tall = 0.f;
        const float fine = 0.12f * (float)std::max(s.w, s.h);   // (letters this many points tall: a line as fine as an eighth of the stage)
        t->font(std::filesystem::path(fontFile).stem().string().c_str());   // (ThorVG names a font by its file)
        float sized = fine;   // (as large as it is set: ThorVG does not say it back)
        t->size(sized);
        t->text(words);
        t->spacing(fx.letters, fx.lines);
        if (fx.italic > 0.f) t->italic(fx.italic);
        if (fx.wrapWidth > 0.f) t->layout(fx.wrapWidth * fine, 0.f), t->wrap((tvg::TextWrap)std::clamp(fx.wrap, 0, 4)), t->align(align, 0.f);   // (in a box: broken into lines there, ThorVG's own)
        t->fill((uint8_t)std::lround(255.f * colour.x), (uint8_t)std::lround(255.f * colour.y), (uint8_t)std::lround(255.f * colour.z));
        bool measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success && wide > 0.f && tall > 0.f;
        if (measured && wide > 4096.f) {   // (a line too long for one picture is set smaller)
            t->size(sized = fine * 4096.f / wide);
            measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success;
        }
        if (!measured) return tvg::Paint::rel(t), nullptr;
        if (align > 0.f && fx.wrapWidth <= 0.f) {   // (the lines aligned in a box as wide as the widest of them: ThorVG's layout and align)
            t->layout(wide, 0.f);
            t->align(align, 0.f);
            measured = t->bounds(&x, &y, &wide, &tall) == tvg::Result::Success;
        }
        const float outline = edge.w * tall;   // (half of it lies outside the letters: the picture is that much larger all round)
        if (outline > 0.f) t->outline(outline, (uint8_t)std::lround(255.f * edge.x), (uint8_t)std::lround(255.f * edge.y), (uint8_t)std::lround(255.f * edge.z));
        t->translate(-x + 0.5f * outline, -y + 0.5f * outline);
        *w = 2 * (int)std::ceil(0.5f * (wide + outline));
        *h = 2 * (int)std::ceil(0.5f * (tall + outline));
        if (fx.split && !strchr(words, '\n') && fx.wrapWidth <= 0.f) {   // (one line split into its letters: where each ends, by how wide the words up to it are set)
            const auto across = [&](const std::string& some) {   // (how wide some letters are set, a share of the picture's width; and where their ink ends)
                tvg::Text* up = tvg::Text::gen();
                up->font(std::filesystem::path(fontFile).stem().string().c_str());
                up->size(sized);
                up->spacing(fx.letters, fx.lines);
                if (fx.italic > 0.f) up->italic(fx.italic);
                up->text(some.c_str());
                float ux = 0.f, uy = 0.f, uw = 0.f, uh = 0.f;
                const bool ok = up->bounds(&ux, &uy, &uw, &uh) == tvg::Result::Success;
                tvg::Paint::rel(up);
                return float2{ok ? (ux + uw - x + 0.5f * outline) / (float)*w : 0.f, ok ? (uw + outline) / (float)*w : 0.f};
            };
            std::vector<float2> inks;   // (each letter: where the words up to it end, and how wide it is alone)
            for (const char* at = words; *at;) {
                const char* next = at + 1;
                while (*next && ((unsigned char)*next & 0xC0) == 0x80) ++next;   // (one letter, however many bytes)
                inks.push_back(float2{across(std::string(words, next)).x, across(std::string(at, next)).y});
                at = next;
            }
            float was = 0.f;   // (cut half way across the gap between a letter's ink and the next one's: no edge of a neighbour in either)
            for (size_t i = 0; i < inks.size(); ++i) {
                const float end = std::max(was, inks[i].x);
                const float cut = i + 1 < inks.size() ? std::max(end, 0.5f * (end + std::max(end, inks[i + 1].x - inks[i + 1].y))) : 1.f;
                d->letters.push_back(float2{was, cut});
                was = cut;
            }
        }
        if (fx.ramp.size() < 2) return t;
        // (a ramp: a rect as large as the words, filled with it, seen only where the letters are: ThorVG's own alpha
        //  mask; ThorVG counts a text's own gradient in the font's units, which are not to be had from outside)
        tvg::Shape* ramp = tvg::Shape::gen();
        ramp->appendRect(0.f, 0.f, (float)*w, (float)*h);
        ramp->fill(ramp_of(fx.ramp, (float)*w, (float)*h, fx.radial, fx.rampAngle, fx.rampScale, fx.spread));
        ramp->mask(t, tvg::MaskMethod::Alpha);
        return ramp;
    }, place, high, float3{-1.f}, true, lay, fx);
}

// A flat shape's path, as `flat` draws it in a picture w by h points, set in by `in` (half its stroke): ThorVG's own
// rectangle, circle, polygon, star, line or wave; its corners rounded (a share of its shorter side), its lines and curves
// cut in pieces no longer than `fine` points (0: as they are), for what bends it as it is drawn. Kept as made in `d`.
bool sk_svg(const char* data, std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts);
int sk_ends(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float from, float to, std::vector<float>& out);
static void flat_path(Drawn* d, tvg::Shape* shape, const char* form, int w, int h, float in, int points, float inner, float round, float fine) {
    const float cx = 0.5f * (float)w, cy = 0.5f * (float)h, rx = cx - in, ry = cy - in;
    const float thick = in > 0.f ? 2.f * in : 0.08f * (float)h;   // (a wave's line: its stroke, or a share of its height)
    if (form[0] == 'M' || form[0] == 'm') {   // (a path of its own, drawn by a pen: SVG's path data, in its height from its middle, down as SVG goes; Skia reads it)
        std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
        if (sk_svg(form, c, p)) {
            for (tvg::Point& at : p) at = {cx + at.x * (float)h, cy + at.y * (float)h};
            shape->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
    }
    else if (!strcmp(form, "circle")) shape->appendCircle(cx, cy, rx, ry);
    else if (!strcmp(form, "wave")) {   // (a line waving across it: so many times along (points), so far up and down (inner, a share of its half height))
        const int cycles = std::clamp(points, 1, 100), n = 48 * cycles;
        const float amp = (cy - thick) * std::clamp(inner, 0.f, 1.f);
        for (int k = 0; k <= n; ++k) {
            const float x = thick + ((float)w - 2.f * thick) * (float)k / (float)n, y = cy + amp * std::sin(TAU * (float)cycles * (float)k / (float)n);
            k ? shape->lineTo(x, y) : shape->moveTo(x, y);
        }
    }
    else if (!strcmp(form, "spiral")) {   // (a line winding out from its middle, as Cavalry's spiral: so many turns (points), from `inner` of the way out)
        const float turns = (float)std::clamp(points, 1, 50), from = std::clamp(inner, 0.f, 0.95f);
        const int n = (int)(96.f * turns);
        for (int k = 0; k <= n; ++k) {
            const float u = (float)k / (float)n, a = -0.25f * TAU + TAU * turns * u, r = from + (1.f - from) * u;
            const float x = cx + (rx - thick) * r * std::cos(a), y = cy + (ry - thick) * r * std::sin(a);
            k ? shape->lineTo(x, y) : shape->moveTo(x, y);
        }
    }
    else if (!strcmp(form, "ring") || !strcmp(form, "arc")) {   // (a ring: its hole `inner` of the way out; an arc: so many degrees of it (points), `inner` thick, from straight up clockwise)
        const bool arc = !strcmp(form, "arc");
        const float hole = arc ? 1.f - std::clamp(inner, 0.02f, 1.f) : std::clamp(inner, 0.f, 0.98f), sweep = arc ? TAU * std::clamp((float)points, 1.f, 360.f) / 360.f : TAU;
        const int n = std::max(8, (int)(96.f * sweep / TAU));
        for (int k = 0; k <= n; ++k) {   // (round the outside, then back round the inside: the hole a contour of its own the other way)
            const float a = -0.25f * TAU + sweep * (float)k / (float)n;
            k ? shape->lineTo(cx + rx * std::cos(a), cy + ry * std::sin(a)) : shape->moveTo(cx + rx * std::cos(a), cy + ry * std::sin(a));
        }
        if (!arc) shape->close();
        if (hole > 0.f) {
            for (int k = n; k >= 0; --k) {
                const float a = -0.25f * TAU + sweep * (float)k / (float)n;
                (k == n && !arc) ? shape->moveTo(cx + rx * hole * std::cos(a), cy + ry * hole * std::sin(a)) : shape->lineTo(cx + rx * hole * std::cos(a), cy + ry * hole * std::sin(a));
            }
        }
        shape->close();
    }
    else if (!strcmp(form, "arrow")) {   // (pointing right: its shaft `inner` of its height thick, its head a third of its length)
        const float shaft = 0.5f * ry * std::clamp(inner, 0.05f, 1.f), head = (float)w - in - std::min(2.f * rx / 3.f, 1.2f * ry);
        const float pts[][2] = {{in, cy - shaft}, {head, cy - shaft}, {head, in}, {(float)w - in, cy}, {head, (float)h - in}, {head, cy + shaft}, {in, cy + shaft}};
        for (int k = 0; k < 7; ++k) k ? shape->lineTo(pts[k][0], pts[k][1]) : shape->moveTo(pts[k][0], pts[k][1]);
        shape->close();
    }
    else if (!strcmp(form, "squircle")) {   // (a superellipse, |x|^p + |y|^p = 1: points its power, 2 an ellipse, higher squarer)
        const float p = std::clamp((float)points, 2.f, 40.f);
        for (int k = 0; k < 128; ++k) {
            const float a = TAU * (float)k / 128.f, c = std::cos(a), sn = std::sin(a);
            const float x = cx + rx * std::copysign(std::pow(std::fabs(c), 2.f / p), c), y = cy + ry * std::copysign(std::pow(std::fabs(sn), 2.f / p), sn);
            k ? shape->lineTo(x, y) : shape->moveTo(x, y);
        }
        shape->close();
    }
    else if (!strcmp(form, "cog")) {   // (a cogwheel: so many teeth (points), each `inner` of its radius deep, flat topped)
        const int teeth = std::clamp(points, 3, 100), n = 4 * teeth;
        const float deep = 1.f - std::clamp(inner, 0.02f, 0.9f);
        for (int k = 0; k < n; ++k) {   // (a tooth: rising, top, falling, bottom: four corners)
            const float a0 = -0.25f * TAU + TAU * (float)k / (float)n, r = (k % 4 == 1 || k % 4 == 2) ? 1.f : deep;
            k ? shape->lineTo(cx + rx * r * std::cos(a0), cy + ry * r * std::sin(a0)) : shape->moveTo(cx + rx * r * std::cos(a0), cy + ry * r * std::sin(a0));
        }
        shape->close();
    }
    else if (!strcmp(form, "polygon") || !strcmp(form, "star")) {
        const bool star = !strcmp(form, "star");
        const int n = std::clamp(points, 3, 100) * (star ? 2 : 1);
        for (int k = 0; k < n; ++k) {
            const float a = -0.5f * TAU / 2.f + TAU * (float)k / (float)n, r = star && (k & 1) ? std::clamp(inner, 0.01f, 1.f) : 1.f;   // (the first point straight up)
            k ? shape->lineTo(cx + rx * r * std::cos(a), cy + ry * r * std::sin(a)) : shape->moveTo(cx + rx * r * std::cos(a), cy + ry * r * std::sin(a));
        }
        shape->close();
    } else {
        const float side = std::min((float)w, (float)h) - 2.f * in, corner = !strcmp(form, "line") ? 0.5f * side : 0.f;   // (a rect's corners: round, bent as the others)
        shape->appendRect(in, in, (float)w - 2.f * in, (float)h - 2.f * in, corner, corner);
    }
    const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
    shape->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
    d->plain.assign(cmds, cmds + cmdsCnt), d->plainAt.assign(pts, pts + ptsCnt);
    d->plainHigh = (float)h, d->plainSide = std::min((float)w, (float)h) - 2.f * in;
    d->fine = fine;
    if ((round > 0.f && strcmp(form, "line")) || d->fine != 0.f)
        tvg_bend(shape, d->plain.data(), cmdsCnt, d->plainAt.data(), ptsCnt, strcmp(form, "line") ? std::clamp(round, 0.f, 0.5f) * d->plainSide : 0.f, 0.f, 0.f, 0, false, 0.f, d->fine), d->bent[0] = round;
}

// A flat shape in one colour, drawn by ThorVG's own Shape (a rectangle, a circle): `wide` times as wide as it is high.
// edge: ThorVG's stroke round it, inside its frame (its colour; w: how wide, a share of its height; 0: none).
// form: rect (round: its corners, a share of its shorter side), circle, polygon and star (points of it; a star's
// inner points at `inner` of the way out), line (a long rect, round at its ends), null (nothing to see: a thing to
// go with); all drawn by ThorVG's own Shape.
static Drawn* flat(Stage& s, const char* form, float wide, const mat4f& place, float high, float3 colour, const char* lay = nullptr, float4 edge = float4{0.f}, const DrawnFx& fx = {},
                   int points = 6, float inner = 0.5f, float round = 0.f) {
    return drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        const float tall = std::min(0.5f * (float)std::max(s.w, s.h), 4096.f / std::max(wide, 1e-3f));   // (as fine as half the stage; a long thin bar less high, so that it is as long as it is said)
        *h = std::max(2, 2 * (int)std::ceil(0.5f * tall));
        *w = std::max(2, 2 * (int)std::ceil(0.5f * tall * wide));
        const float in = 0.5f * std::min(edge.w * (float)*h, 0.5f * (float)std::min(*w, *h));   // (half the stroke: the path is set in by that, so all of it is seen)
        const float thick = in > 0.f ? 2.f * in : 0.08f * (float)*h;   // (a wave's line: its stroke, or a share of its height)
        tvg::Shape* shape = d->shape = tvg::Shape::gen();
        flat_path(d, shape, form, *w, *h, in, points, inner, round, 0.f);
        shape->fill((uint8_t)std::lround(255.f * colour.x), (uint8_t)std::lround(255.f * colour.y), (uint8_t)std::lround(255.f * colour.z), strcmp(form, "null") && !fx.hollow ? 255 : 0);
        if (fx.ramp.size() > 1 && strcmp(form, "null")) shape->fill(ramp_of(fx.ramp, (float)*w, (float)*h, fx.radial, fx.rampAngle, fx.rampScale, fx.spread));
        if (in > 0.f) shape->strokeWidth(2.f * in), shape->strokeFill((uint8_t)std::lround(255.f * edge.x), (uint8_t)std::lround(255.f * edge.y), (uint8_t)std::lround(255.f * edge.z));
        if (!strcmp(form, "wave") || !strcmp(form, "spiral")) {   // (only its line is seen: in its edge's colour, or its fill's; round at its ends)
            const float3 ink = in > 0.f ? float3{edge.x, edge.y, edge.z} : colour;
            shape->fill(0, 0, 0, 0);
            shape->strokeWidth(thick), shape->strokeFill((uint8_t)std::lround(255.f * ink.x), (uint8_t)std::lround(255.f * ink.y), (uint8_t)std::lround(255.f * ink.z));
            shape->strokeCap(tvg::StrokeCap::Round);
        }
        if (fx.cap >= 0) shape->strokeCap((tvg::StrokeCap)fx.cap);
        if (fx.join >= 0) shape->strokeJoin((tvg::StrokeJoin)fx.join);
        if (fx.dash > 0.f) { const float gap[2] = {fx.dash * (float)*h, fx.dash * (float)*h}; shape->strokeDash(gap, 2, fx.dashFrom * (float)*h); }   // (as long as the gaps; in shares of its height)
        return shape;
    }, place, high, float3{-1.f}, true, lay, fx);
}

void tvg_edge(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, uint32_t ptsCnt, float width, int cap, int join, float dash, float dashFrom,
              float from, float to, std::vector<float>& xy, std::vector<uint32_t>& idx);   // (bend.cpp)

// A flat shape's triangles as its path is now (flat_mesh): its fill (none if hollow: cut by libtess2, made again only
// when its path changes, `refill`), then its edge (ThorVG's own stroker: trimmed and dashed as told, its triangles
// straight), in shares of its height from its middle (uv) and in the stage's units (pos), into its sheet; where each
// part's indices end (a part with none: one empty triangle).
static std::vector<uint32_t> shape_cut(Drawn* d, float trimFrom, float trimTo, bool refill) {
    const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
    d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
    Mesh& m = *d->sheet;
    const float w = d->cutW, h = d->plainHigh;
    const auto point = [&](float x, float y) {
        const float2 at{(x - 0.5f * w) / h, (0.5f * h - y) / h};
        m.pos.push_back({d->sheetHigh * at, 0.f}), m.nrm.push_back({0, 0, 1}), m.uv.push_back(at);
    };
    const auto none = [&] { if (m.pos.empty()) point(0.5f * w, 0.5f * h); for (int k = 0; k < 3; ++k) m.idx.push_back(0); };
    if (refill || m.idx.empty()) {
        m.pos.clear(), m.nrm.clear(), m.uv.clear(), m.idx.clear();
        if (!d->hollow) {   // (its outline: its pieces are short, so a curve is its end)
            TESStesselator* t = tessNewTess(nullptr);
            std::vector<float2> ring;
            const auto end = [&] { if (ring.size() > 2) tessAddContour(t, 2, ring.data(), sizeof(float2), (int)ring.size()); ring.clear(); };
            for (uint32_t c = 0, p = 0; c < cmdsCnt; ++c) switch (cmds[c]) {
                case tvg::PathCommand::MoveTo: end(); ring.push_back({pts[p].x, pts[p].y}); ++p; break;
                case tvg::PathCommand::LineTo: ring.push_back({pts[p].x, pts[p].y}); ++p; break;
                case tvg::PathCommand::CubicTo: ring.push_back({pts[p + 2].x, pts[p + 2].y}); p += 3; break;
                case tvg::PathCommand::Close: end(); break;
            }
            end();
            if (tessTesselate(t, TESS_WINDING_NONZERO, TESS_POLYGONS, 3, 2, nullptr)) {
                const TESSreal* v = tessGetVertices(t);
                for (int i = 0; i < tessGetVertexCount(t); ++i) point(v[2 * i], v[2 * i + 1]);
                const TESSindex* e = tessGetElements(t);
                for (int i = 0; i < 3 * tessGetElementCount(t); ++i) m.idx.push_back((uint32_t)e[i]);
            }
            tessDeleteTess(t);
        }
        if (m.idx.empty()) none();
        d->fillPoints = (uint32_t)m.pos.size(), d->fillEnd = (uint32_t)m.idx.size();
    } else {   // (its fill as it was: its edge only made again)
        m.pos.resize(d->fillPoints), m.nrm.resize(d->fillPoints), m.uv.resize(d->fillPoints), m.idx.resize(d->fillEnd);
    }
    if (d->edgeWide <= 0.f) return {d->fillEnd};
    std::vector<float> xy;
    std::vector<uint32_t> idx;
    tvg_edge(cmds, cmdsCnt, pts, ptsCnt, d->edgeWide, d->fx.cap < 0 ? (int)tvg::StrokeCap::Square : d->fx.cap, d->fx.join < 0 ? (int)tvg::StrokeJoin::Bevel : d->fx.join,   // (ThorVG's own: square, bevel)
             d->fx.dash * h, d->fx.dashFrom * h, trimFrom, trimTo, xy, idx);
    const uint32_t base = (uint32_t)m.pos.size();
    for (size_t i = 0; i < xy.size(); i += 2) point(xy[i], xy[i + 1]);
    for (uint32_t i : idx) m.idx.push_back(base + i);
    if (d->fx.heads) {   // (an arrow head on each open outline's end, its start, or both, where its trim leaves them: a triangle in its edge's colour, pointing the way the line goes)
        const auto head = [&](tvg::Point at, tvg::Point back) {
            float dx = at.x - back.x, dy = at.y - back.y;
            const float len = std::hypot(dx, dy), l = d->fx.headSize * d->edgeWide;
            if (len < 1e-4f) return;
            dx /= len, dy /= len;
            const uint32_t k = (uint32_t)m.pos.size();
            point(at.x + 0.5f * l * dx, at.y + 0.5f * l * dy);
            point(at.x - 0.5f * l * dx - 0.4f * l * dy, at.y - 0.5f * l * dy + 0.4f * l * dx);
            point(at.x - 0.5f * l * dx + 0.4f * l * dy, at.y - 0.5f * l * dy - 0.4f * l * dx);
            m.idx.insert(m.idx.end(), {k, k + 1, k + 2});
        };
        std::vector<float> ends;
        for (int k = 0, n = sk_ends(cmds, cmdsCnt, pts, trimFrom, trimTo, ends); k < n; ++k) {
            const float* e = &ends[8 * (size_t)k];
            if (d->fx.heads & 1) head({e[0], e[1]}, {e[0] - e[2], e[1] - e[3]});
            if (d->fx.heads & 2) head({e[4], e[5]}, {e[4] - e[6], e[5] - e[7]});
        }
    }
    if (idx.empty()) none();
    return {d->fillEnd, (uint32_t)m.idx.size()};
}

// A flat shape drawn by Filament itself, as the scene's other things are: its path made as `flat` makes it (ThorVG's),
// cut into triangles (libtess2, as Unity's vector shapes are) again only when its path changes (cut_at), shown with
// flat.mat washed in its colour on a white point (its edge a second part of it, in its edge's colour, ThorVG's own stroke); antialiased by
// the view's MSAA, its points moved in the look's vertex stage (bends). Nothing is drawn for it into a picture of its
// own: forty of them waving cost a quarter of the time as ThorVG pictures do (check/shape_mesh.sh). No ramp, no effects:
// those are `flat`'s. edge: its colour, and how wide (a share of its height; inside its frame, as `flat`'s).
static Drawn* flat_mesh(Stage& s, const char* form, float wide, const mat4f& place, float high, float3 colour, const char* lay = nullptr, float4 edge = float4{0.f},
                        const DrawnFx& fx = {}, int points = 6, float inner = 0.5f, float round = 0.f, bool bends = true) {
    const int h = 1024, w = std::max(2, (int)std::lround(1024.f * wide));   // (points to make its path in: its own, as fine as it is)
    Drawn* d = new Drawn;
    d->fx = fx, d->cutW = (float)w, d->sheetHigh = high, d->hollow = fx.hollow;
    const float in = 0.5f * std::min(edge.w * (float)h, 0.5f * (float)std::min(w, h));   // (half its edge: the path set in by that, as `flat` does)
    d->edgeWide = 2.f * in;
    d->outline = tvg::Shape::gen();
    flat_path(d, d->outline, form, w, h, in, points, inner, round, (bends ? 1.f : -1.f) * (float)h / 64.f);   // (bent as it is drawn: cut fine all along; not: only its curves, its straight lines whole)
    d->sheet = new Mesh;
    const std::vector<uint32_t> ends = shape_cut(d, 0.f, 1.f, true);
    d->cut = true, d->wideBy = wide;
    d->wash = float4{Color::toLinear<ACCURATE>(sRGBColor{colour}), 1.f};
    const std::vector<MaterialInstance*> looks = add_parts(s, d->sheet, flat_laid(lay).c_str(), std::vector<mat4f>((size_t)std::max(1, fx.copies), place), ends, 1, false, 1, 4, fx.together);
    if (looks.empty()) return nullptr;
    d->look = looks[0];
    d->edgeLook = looks.size() > 1 ? looks[1] : nullptr;
    for (MaterialInstance* look : looks) look->setParameter("picture", white_point(s), TextureSampler());
    d->look->setParameter("wash", d->wash);
    if (d->edgeLook) d->edgeLook->setParameter("wash", float4{Color::toLinear<ACCURATE>(sRGBColor{edge.xyz}), 1.f});
    return d;
}

bool sk_rough(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float apart, float far, uint32_t seed, std::vector<tvg::PathCommand>& outCmds, std::vector<tvg::Point>& outPts);
bool sk_morph(const tvg::PathCommand* ac, uint32_t acn, const tvg::Point* ap, const tvg::PathCommand* bc, uint32_t bcn, const tvg::Point* bp, float t,
              std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts);
bool sk_op(int op, const tvg::PathCommand* ac, uint32_t acn, const tvg::Point* ap, const tvg::PathCommand* bc, uint32_t bcn, const tvg::Point* bp,
           std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts);

// A flat shape in triangles cut anew for the frame that comes next, if its path is told otherwise: After Effects' trim
// paths on its edge (off its end, off its beginning, slid along), its operators (round, bloat, zig zag, offset: bend.cpp),
// then put together with another shape's path (`join`: Skia's path operation, skpath.cpp; -1 none; `by`/`at` that path
// in this one's points), as Cavalry's Boolean.
static void cut_at(Stage& s, Drawn* d, float3 trim, const std::array<float, 8>& bend, Drawn::Other join = {}, Drawn::Other morph = {}) {
    if (join.how < 0 || join.by.empty()) join = {};
    if (morph.much <= 0.f || morph.by.empty()) morph = {};
    const bool joins = !(join == d->joined) || !(morph == d->morphed);
    if (!d->cut || !d->outline || (trim == d->trimmed && bend == d->bent && !joins)) return;
    d->trimmed = trim;
    const bool refill = bend != d->bent || joins;
    if (refill) {
        d->bent = bend, d->joined = join, d->morphed = morph;
        const float high = d->plainHigh;
        tvg_bend(d->outline, d->plain.data(), (uint32_t)d->plain.size(), d->plainAt.data(), (uint32_t)d->plainAt.size(), std::clamp(bend[0], 0.f, 0.5f) * d->plainSide,
                 100.f * std::clamp(bend[1], -1.f, 1.f), bend[2] * high, (int)std::clamp(bend[3], 0.f, 100.f), bend[4] > 0.5f, bend[5] * high, d->fine);
        if (bend[6] > 0.f) {   // (roughened, as Illustrator's Roughen and After Effects' Roughen Edges: Skia's own discrete path effect, so far out, a point so often)
            const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
            d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
            std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
            if (sk_rough(cmds, cmdsCnt, pts, std::max(bend[7], 0.005f) * high, std::min(bend[6], 0.5f) * high, 0, c, p))
                d->outline->reset(), d->outline->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
        if (morph.much > 0.f) {   // (turned into the other so far, as Cavalry's Morph: both walked at even steps, point to point)
            const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
            d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
            std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
            if (sk_morph(cmds, cmdsCnt, pts, morph.by.data(), (uint32_t)morph.by.size(), morph.at.data(), std::min(morph.much, 1.f), c, p))
                d->outline->reset(), d->outline->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
        if (join.how >= 0) {
            const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
            d->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
            std::vector<tvg::PathCommand> c; std::vector<tvg::Point> p;
            sk_op(join.how, cmds, cmdsCnt, pts, join.by.data(), (uint32_t)join.by.size(), join.at.data(), c, p);
            d->outline->reset();
            if (!c.empty()) d->outline->appendPath(c.data(), (uint32_t)c.size(), p.data(), (uint32_t)p.size());
        }
    }
    const float from = std::clamp(trim.y, 0.f, 1.f) + trim.z, to = 1.f - std::clamp(trim.x, 0.f, 1.f) + trim.z;
    reshape(s, d->sheet, shape_cut(d, from, to, refill));
}

// One layer of an After Effects file drawn alone, in its own space: its place and turn left out (they are its sheet's,
// put in the scene by its pose: motion.h), on a sheet of its own `pixel` units a point of the composition. A 3D layer's
// picture where it is. `box`: where its sheet lies in the layer's own space (x, y, w, h, in points of the composition).
static Drawn* layer_picture(Stage& s, const char* file, uint32_t id, float pixel, float4* box) {
    if (!door::device(s.engine)) return nullptr;
    float4 found{0.f};
    door::on_driver(s.engine, [&] {   // (how large it is, at its first moment: ThorVG's own bounds of it alone)
        tvg::Initializer::init(0);
        tvg::Animation* look = tvg::LottieAnimation::gen();
        if (look->picture()->load(file) == tvg::Result::Success && (look->frame(1.f), look->frame(0.f), true))   // (built: ThorVG builds a moment when it is told one other than the one it has)
            if (const tvg::Paint* one = look->picture()->paint(id)) {
                tvg::Paint* copy = one->duplicate();
                copy->transform(tvg::Matrix{1, 0, 0, 0, 1, 0, 0, 0, 1});
                copy->bounds(&found.x, &found.y, &found.z, &found.w);
                tvg::Paint::rel(copy);
            }
        delete look;
    });
    s.engine->flushAndWait();
    if (found.z <= 0.f || found.w <= 0.f) return nullptr;
    *box = found;
    const float sharp = std::max(1.f, 512.f / found.w);   // (drawn at least 512 points high, however small it is in the composition)
    return drawn(s, [&](Drawn* d, int* w, int* h) -> tvg::Paint* {
        d->source = tvg::LottieAnimation::gen();
        if (d->source->picture()->load(file) != tvg::Result::Success) return nullptr;
        d->source->frame(1.f);   // (so that its first moment, asked for, is built)
        d->layerId = id;
        d->fit = tvg::Matrix{sharp, 0, -found.x * sharp, 0, sharp, -found.y * sharp, 0, 0, 1};
        *w = std::max(2, (int)std::ceil(found.z * sharp)), *h = std::max(2, (int)std::ceil(found.w * sharp));
        d->holder = tvg::Scene::gen();
        return d->holder;
    }, mat4f{}, found.w * pixel, float3{-1.f}, true, nullptr);
}

static Drawn* layer_picture(Stage& s, const char* file, const char* layer, float pixel, float4* box) { return layer_picture(s, file, tvg::Accessor::id(layer), pixel, box); }

// An After Effects picture's 3D layers (ThorVG says each layer's pose and whether it is 3D): from now on not drawn in
// its own picture, each to be a sheet of its own where it is (layer_picture). Their ids.
static std::vector<uint32_t> picture_deep(Stage& s, Drawn* d) {
    std::vector<uint32_t> ids;
    tvg::LottieAnimation* film = d ? d->lottie : nullptr;
    if (!film) return ids;
    door::on_driver(s.engine, [&] {
        auto* walk = tvg::Accessor::gen();
        walk->set(film->picture(), [&](const tvg::Paint* one, void*) {
            float n[19];
            uint32_t above = 0;
            int kind = 0;
            if (one->id && film->pose(one->id, n, &above, &kind) == tvg::Result::Success && kind == 1 && std::find(ids.begin(), ids.end(), one->id) == ids.end()) ids.push_back(one->id);
            return true;
        }, nullptr);
        delete walk;
    });
    s.engine->flushAndWait();
    d->deep = ids;
    return ids;
}

// That layer as it is at `frame` of its file, drawn again for the frame that comes next.
static void layer_at(Stage& s, Drawn* d, float frame) {
    if (!d || !d->source) return;
    door::on_driver(s.engine, [d, frame] {
        d->source->frame(frame);
        d->holder->remove();
        if (const tvg::Paint* one = d->source->picture()->paint(d->layerId)) {
            tvg::Paint* copy = one->duplicate();
            copy->transform(d->fit);
            d->holder->add(copy);
        }
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
}

// A picture taken off the stage for good.
static void picture_off(Stage& s, Drawn* d) {
    s.playing.erase(d);
    take_off(s, d->sheet);
    door::on_driver(s.engine, [d] { delete d->canvas; delete d->film; });
    s.engine->flushAndWait();
    d->look = nullptr;
    kept_drop(s, d);
    if (d->texture) s.engine->destroy(d->texture);
    delete d;
}

