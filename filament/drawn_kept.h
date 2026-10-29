// drawn_kept.h: part of drawn.h. Read only through drawn.h.
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
    int heads = 0; float headSize = 4.f; int headShape = 0, headStart = -1;   // (heads on an open edge's ends, as Cavalry's: 1 its end, 2 its start, 3 both; so many times its edge long; 0 arrow, 1 circle, 2 box, 3 chevron, 4 bar; from 5 Alight Motion's, drawn by AbsentMotion (drawn_flat.h); its start's own, -1 as its end's)
    float blur = 0.f; int blurWay = 0; float overHigh = 1.f; float4 shadow{0.f}; float fall = 0.f, way = 0.f, soft = 0.f; float tint = 0.f; float3 tints[2]{}; float tritone = 0.f; float3 tones[3]{}; int copies = 1;
                 std::vector<float4> ramp;   // (two colours or more: it is filled from the first at its top to the last at its bottom, ThorVG's linear gradient)
                 bool hollow = false;   // (a flat shape filled with nothing: only its edge is seen)
                 std::string media;     // (a flat shape filled with a picture's file instead of its colour, seen only where it is filled: Alight Motion's Fill > Media)
                 bool evenOdd = false;  // (where its outlines lie over each other, a hole every other time: SVG's fill-rule evenodd; else nonzero)
                 float reach = 0.f;     // (how far a bent shape goes out past its box at its most, a share of its height (bend_at): the picture larger all round by that)
                 float room = 0.f;      // (how far its effects may reach round it, made room for (doc_fx_room), a share of its height: changed within it, drawn again without being made again)
                 char order[5] = "trsb";   // (its effects in the order they are laid, as the thing's list says (doc_fx_list): t(int) r (tritone) s(hadow) b(lur))
                 bool radial = false; float rampAngle = 0.f, rampScale = 1.f; int spread = 0;   // (or from its middle out, ThorVG's radial gradient; turned so far, clockwise; so long, as a share of its reach; past its ends padded, reflected or repeated)
                 DrawnFx over(float high) const {   // (the same for a thing `high` units high: its lengths as shares of that)
                     DrawnFx f = *this;
                     f.blur /= high, f.fall /= high, f.soft /= high, f.room /= high, f.overHigh = high;
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
    std::string slots;                 // (an After Effects file's slots as last told: lottie_slots.h)
    std::vector<float> lit;            // (a MIDI roll's notes, how lit each is as last told: midi_roll.cpp)
    std::vector<tvg::Matrix> moves;    // (and how each is moved)
    uint32_t slotsId = 0;              // (ThorVG's for them, driver thread only)
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
    std::array<float, 10> bent{};   // (as last bent: round, bloat, zigzag, ridges, smooth, offset; roughened: how far, how fine; walked again at so many points; in squares so large)
    // Another shape's path in this one's points, and what is done with it: put together (how: skpath.cpp's op; -1 none),
    // or turned into (much: how far, 0 not; how above 1: so many steps from itself to it, all at once, a blend).
    struct Other {
        int how = -1; float much = 0.f; std::vector<tvg::PathCommand> by; std::vector<tvg::Point> at;
        bool operator==(const Other& o) const {
            return how == o.how && much == o.much && by == o.by && at.size() == o.at.size() && std::equal(at.begin(), at.end(), o.at.begin(), [](tvg::Point a, tvg::Point b) { return a.x == b.x && a.y == b.y; });
        }
    } joined, morphed, traced;   // (as last put together with, as last turned into, as last tracing: its whole path another's track)
    // Its paths as cut before, by what they were cut from (as PAG keeps a frame's shapes and Beutl a path by its version):
    // a moment cut once is not cut again while they are kept (all shapes' together at most cuts_most).
    struct Cut { std::vector<tvg::PathCommand> by; std::vector<tvg::Point> at; std::vector<float3> pos; std::vector<float2> uv; std::vector<uint32_t> idx, ends; uint32_t fillPoints, fillEnd; size_t bytes; };
    struct Cuts { std::unordered_map<uint64_t, Cut> all; std::deque<uint64_t> order; size_t bytes = 0; ~Cuts(); } cuts;
    std::vector<std::string> markers;  // an After Effects file's named stretches (ThorVG's Lottie markers)
    std::vector<float2> letters;       // a text split into its letters: where each lies across the picture (shares of its width); its copies show one each
    float fromFrame = 0.f, toFrame = 0.f;   // the stretch it plays, when one is named (frames of the file)
    std::vector<tvg::Scene*> effects;  // ThorVG's scene its effects are on (driver thread only)
    float tall = 0.f;                  // how high it was drawn, in its own points: what its effects' lengths are shares of
    DrawnFx fx;                        // its effects as last told
    tvg::Text* text = nullptr;         // a text's own words, ThorVG's (driver thread only): told again in place (words_at); none when split, wrapped or aligned
    std::string words;                 // (as last told)
    float textWide = 0.f, textAt = 0.f;   // (its picture's width and where its words stand down it, in its own points)
    std::atomic<bool> outgrown{false};    // (words told since that are wider than its picture: it is to be made again)
};

static void kept_drop(Stage& s, Drawn* d);

// One of ThorVG's own effects on a scene (k: a letter of DrawnFx::order), lengths as shares of `tall` points.
static void effect_on(tvg::Scene* on, char k, const DrawnFx& fx, float tall) {
    const auto c = [](float v) { return (int)std::lround(255.f * v); };
    if (k == 't' && fx.tint > 0.f) on->add(tvg::SceneEffect::Tint, c(fx.tints[0].x), c(fx.tints[0].y), c(fx.tints[0].z), c(fx.tints[1].x), c(fx.tints[1].y), c(fx.tints[1].z), (double)(100.f * fx.tint));
    if (k == 'r' && fx.tritone > 0.f)
        on->add(tvg::SceneEffect::Tritone, c(fx.tones[0].x), c(fx.tones[0].y), c(fx.tones[0].z), c(fx.tones[1].x), c(fx.tones[1].y), c(fx.tones[1].z), c(fx.tones[2].x), c(fx.tones[2].y), c(fx.tones[2].z), c(1.f - fx.tritone));
    if (k == 's' && fx.shadow.w > 0.f) on->add(tvg::SceneEffect::DropShadow, c(fx.shadow.x), c(fx.shadow.y), c(fx.shadow.z), c(fx.shadow.w), (double)fx.way, (double)(fx.fall * tall), (double)std::max(fx.soft * tall, 0.01f), 75);
    if (k == 'b' && fx.blur > 0.f) on->add(tvg::SceneEffect::GaussianBlur, (double)(fx.blur * tall), fx.blurWay, 0, 75);   // (its way: both, across, up and down)
}
// ThorVG's own effects on its scene, in the order the thing's list lays them; lengths as shares of `tall` points.
static void effects_on(const std::vector<tvg::Scene*>& chain, const DrawnFx& fx, float tall) {
    tvg::Scene* on = chain[0];
    on->add(tvg::SceneEffect::Clear);
    for (const char* k = fx.order; *k; ++k) effect_on(on, *k, fx, tall);
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
    const std::array<float, 10> is{round, bloat, zigzag, ridges, smooth, offset, 0.f, 0.f, 0.f, 0.f};
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

// An After Effects file's slots as they are now (lottie_slots_json), drawn again for the frame that comes next, as fx_at
// does for its effects; nothing is done if they are as they were. ThorVG's own way: the slots made, then applied.
static void slots_at(Stage& s, Drawn* d, const std::string& slots) {
    if (!d->lottie || slots == d->slots) return;
    d->slots = slots;
    kept_drop(s, d);
    door::on_driver(s.engine, [d, slots] {
        if (d->slotsId) d->lottie->del(d->slotsId), d->slotsId = 0;
        if (!slots.empty() && (d->slotsId = d->lottie->gen(slots.c_str()))) d->lottie->apply(d->slotsId);
        d->madeAt = -1;   // (its frame drawn again)
    });
}
// A MIDI roll's notes as they are now, each lit (0 its colour, 1 ALMaMPlayer's +0.3 of it: note.gd's modulate) and
// moved (`moves`: its own matrix, grown and dipped as note.gd's tweens do), drawn again for the frame that comes next,
// as slots_at does; nothing is done if they are as they were. Note i is the SVG's "note<i>" (midi_roll.cpp), found by
// ThorVG's own id of it (Accessor::id).
static void lit_at(Stage& s, Drawn* d, const std::vector<float>& lit, const std::vector<tvg::Matrix>& moves, float3 colour) {
    if (!d->film || (lit == d->lit && moves.size() == d->moves.size() && std::equal(moves.begin(), moves.end(), d->moves.begin(), [](const tvg::Matrix& a, const tvg::Matrix& b) { return !std::memcmp(&a, &b, sizeof a); }))) return;
    d->lit = lit, d->moves = moves;
    kept_drop(s, d);
    door::on_driver(s.engine, [d, lit, moves, colour] {
        for (size_t i = 0; i < lit.size(); ++i)
            if (auto* note = const_cast<tvg::Paint*>(d->film->picture()->paint(tvg::Accessor::id(("note" + std::to_string(i)).c_str()))); note && note->type() == tvg::Type::Shape) {
                const float3 c = min(colour + 0.3f * lit[i], float3{1.f});
                static_cast<tvg::Shape*>(note)->fill((uint8_t)std::lround(c.x * 255.f), (uint8_t)std::lround(c.y * 255.f), (uint8_t)std::lround(c.z * 255.f));
                if (i < moves.size()) note->transform(moves[i]);
            }
        d->film->picture()->opacity(254), d->film->picture()->opacity(255);   // (ThorVG redraws a picture only if it is marked changed itself, not its parts: tvgPicture.h update; Paint::opacity marks it)
        d->canvas->update();   // (a picture that does not move is drawn now, as fx_at draws it)
        d->canvas->draw(true);
        d->canvas->sync();
    });
}
// A text's words as they are now (AbsentMotion's text effects: doc_text_shown), drawn again in its own picture for the
// frame that comes next, as slots_at tells a film its slots, across its middle and at the height it was set; nothing is
// done if they are as they were. false: it has no words of its own to tell, or the last told did not fit (outgrown).
static bool words_at(Stage& s, Drawn* d, const std::string& words) {
    if (!d->text || d->outgrown) return false;
    if (words == d->words) return true;
    d->words = words;
    kept_drop(s, d);
    door::on_driver(s.engine, [d, words] {
        d->text->translate(0.f, 0.f);
        d->text->text(words.c_str());
        float x = 0.f, y = 0.f, wide = 0.f, tall = 0.f;
        if (d->text->bounds(&x, &y, &wide, &tall) != tvg::Result::Success) return;
        d->outgrown = wide > d->textWide + 0.5f;
        d->text->translate(0.5f * (d->textWide - wide) - x, d->textAt);
        d->canvas->update();
        d->canvas->draw(true);
        d->canvas->sync();
    });
    return true;
}
// A flat thing's effects as they are now (lengths in shares of its height, as it was made), drawn again for the frame
// that comes next; nothing is done if they are as they were. They reach no further than they did at their most.
static void fx_at(Stage& s, Drawn* d, const DrawnFx& told) {
    if (d->effects.empty()) return;
    const DrawnFx fx = told.over(d->fx.overHigh);
    const DrawnFx& was = d->fx;
    if (fx.blur == was.blur && fx.blurWay == was.blurWay && (fx.shadow == was.shadow) && fx.fall == was.fall && fx.way == was.way && fx.soft == was.soft && fx.tint == was.tint && fx.tritone == was.tritone
        && !strcmp(fx.order, was.order) && (fx.tints[0] == was.tints[0]) && (fx.tints[1] == was.tints[1]) && (fx.tones[0] == was.tones[0]) && (fx.tones[1] == was.tones[1]) && (fx.tones[2] == was.tones[2]))
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
    for (float v : {f.blur, (float)f.blurWay, f.shadow.x, f.shadow.y, f.shadow.z, f.shadow.w, f.fall, f.way, f.soft, f.tint, f.tritone, d->fromFrame, d->toFrame}) add(v);
    for (const float3& c : f.tints) add(c.x), add(c.y), add(c.z);
    for (const float3& c : f.tones) add(c.x), add(c.y), add(c.z);
    for (uint32_t id : d->deep) add((float)id);
    all += "|" + d->slots;
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
static std::atomic<long> films_kept{0}, films_disk{0}, films_drawn{0};   // (how each film's frame was shown: for the inside's spent line)
static void picture_at(Stage& s, Drawn* d, float seconds) {
    const int at = (int)std::floor(std::min(seconds / d->seconds * d->frames, d->frames - 1.f) + 1e-3f);
    if (at == d->shownAt) return;
    d->shownAt = at;
    const TextureSampler smooth(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    kept_take(s, d);
    if (const auto k = d->kept.find(at); k != d->kept.end() && k->second) {   // (drawn before: shown as kept)
        if (d->look) d->look->setParameter("picture", k->second, smooth);
        ++films_kept;
        return;
    }
    if (d->look) d->look->setParameter("picture", d->texture, smooth);
    const bool keep = d->look && d->kept.find(at) == d->kept.end() && kept_bytes < kept_most;
    if (keep) d->kept[at] = nullptr, kept_bytes += kept_one(d);   // (being kept)
    const std::filesystem::path file = kept_file(d, at);
    auto pixels = std::make_shared<std::vector<uint8_t>>();
    if (d->look && kept_read(file, *pixels, kept_one(d)))   // (drawn before, on disk: put up as it is, not drawn)
        ++films_disk, door::on_driver(s.engine, [&s, d, at, pixels, keep, age = d->keepAge.load()] { kept_put(s, d, at, *pixels, true, keep, age); });
    else
        ++films_drawn, door::on_driver(s.engine, [&s, d, at, keep, file, age = d->keepAge.load()] { kept_draw(s, d, at, keep, file, age); });
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
