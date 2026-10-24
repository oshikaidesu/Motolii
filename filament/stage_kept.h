// The stage's own pictures kept on disk for playing (Final Cut Pro's background render, After Effects' disk cache): what
// will be played is known (the work area), so while the hand rests each of its pictures is drawn once as a written film
// draws it, and one that took longer than a picture may (it would be late when played) is kept, zstd on disk as kept
// frames are (drawn.h). Played, a kept picture is put on the screen instead of drawn. A light document keeps nothing.
// Kept by the document as it is (its facts, its ways, the files it uses as they are, its size): any change, drawn again.
// (motolii.mm: read after kit.h, doc.h and live.h)
#pragma once
#include <sys/stat.h>
#include <future>
#include <deque>
#include <thread>
#include <atomic>
#include <mutex>

struct StageKept {
    uint64_t key = 0;               // (the document as it is)
    long revision = -1;             // (the revision the key was taken at)
    std::vector<uint8_t> state;     // (each picture of the document: 0 not looked at, 1 drawn in time (not kept), 2 kept)
    int w = 0, h = 0;
    RenderTarget* target = nullptr; // (where a picture to keep is drawn, as large as the stage)
    Texture* into = nullptr;
    Texture* shown = nullptr;       // (a kept picture, put on the stage when played)
    std::deque<uint64_t> keys;      // (the last documents kept for: an undo back to one finds its pictures; older ones go)
    std::atomic<long> told{0};      // (how many times what is kept has changed: a window reads it without waiting, as the revision)
    std::mutex runsLock;
    std::string runs;               // (the kept pictures as stretches, "from to from to", for a window to mark: After Effects' green line)
    int ahead = -1;                 // (the next picture, read from disk on a thread of its own while this one is shown)
    std::future<std::vector<uint8_t>*> next;
};

// What is kept, told again as stretches (each picture's state changed: kept, or all thrown away).
static void stage_kept_tell(StageKept& k) {
    std::string runs;
    for (int i = 0; i < (int)k.state.size(); ++i) {
        if (k.state[i] != 2 || (i > 0 && k.state[i - 1] == 2)) continue;
        int j = i;
        while (j + 1 < (int)k.state.size() && k.state[j + 1] == 2) ++j;
        runs += (runs.empty() ? "" : " ") + std::to_string(i) + " " + std::to_string(j + 1);
    }
    { std::lock_guard<std::mutex> hold(k.runsLock); k.runs = std::move(runs); }
    k.told.fetch_add(1, std::memory_order_release);
}
static uint64_t stage_kept_key(Doc& d, int w, int h) {
    uint64_t hash = 1469598103934665603ull;   // (FNV-1a, as kept frames are named)
    const auto add = [&](const std::string& s) { for (unsigned char c : s) hash = (hash ^ c) * 1099511628211ull; hash = (hash ^ 0xff) * 1099511628211ull; };
    add(std::to_string(w) + "x" + std::to_string(h));
    doc_rows(d, sqlite3_mprintf("select thing, name, at, value, coalesce(way, '') from fact order by thing, name, at"), [&](sqlite3_stmt* row) {
        for (int i = 0; i < 5; ++i) add(sqlite3_column_text(row, i) ? (const char*)sqlite3_column_text(row, i) : "");
        if (!strcmp((const char*)sqlite3_column_text(row, 1), "file")) {   // (a file it uses, as it is now)
            struct stat st {};
            if (sqlite3_column_text(row, 3) && stat((const char*)sqlite3_column_text(row, 3), &st) == 0) add(std::to_string(st.st_mtimespec.tv_sec) + "." + std::to_string(st.st_size));
        }
    });
    doc_rows(d, sqlite3_mprintf("select name, coalesce(goes, '') from way order by name"), [&](sqlite3_stmt* row) { add((const char*)sqlite3_column_text(row, 0)); add((const char*)sqlite3_column_text(row, 1)); });
    return hash;
}

static std::filesystem::path stage_kept_file(const StageKept& k, int picture) {
    char name[64];
    std::snprintf(name, sizeof name, "stage_%016llx_%05d.zst", (unsigned long long)k.key, picture);
    return kept_dir() / name;
}

// The key taken again once the document has changed; what is on disk for it already is found as it is asked.
static void stage_kept_fresh(StageKept& k, Doc& d, Stage& s, long revision, int rate) {
    if (k.revision == revision && k.w == s.w && k.h == s.h) return;
    k.revision = revision, k.w = s.w, k.h = s.h;
    k.key = stage_kept_key(d, s.w, s.h);
    std::erase(k.keys, k.key);
    k.keys.push_back(k.key);
    if (k.keys.size() > 3) {   // (the document as it was three changes ago: its pictures off the disk, on a thread of its own)
        char gone[32];
        std::snprintf(gone, sizeof gone, "stage_%016llx_", (unsigned long long)k.keys.front());
        k.keys.pop_front();
        std::thread([prefix = std::string(gone)] {
            std::error_code e;
            for (const auto& f : std::filesystem::directory_iterator(kept_dir(), e))
                if (f.path().filename().string().rfind(prefix, 0) == 0) std::filesystem::remove(f.path(), e);
        }).detach();
    }
    k.state.assign((size_t)std::max(1.0, std::ceil(doc_seconds(d) * rate)) + 1, 0);
    if (k.target) s.engine->destroy(k.target), s.engine->destroy(k.into), s.engine->destroy(k.shown), k.target = nullptr;
    for (int i = 0; i < (int)k.state.size(); ++i) {   // (what is on disk for it already, from before: an undo back to it, or the document opened again)
        std::error_code e;
        if (std::filesystem::exists(stage_kept_file(k, i), e)) k.state[i] = 2;
    }
    stage_kept_tell(k);
}

// One picture of the work area not looked at yet, drawn as a written film draws it and kept if it took too long (the
// hand rests: what it draws is not shown). false: none left to look at.
static bool stage_kept_one(StageKept& k, Doc& d, Stage& s, int rate, double budget_ms) {
    double from = 0.0, to = 0.0;
    doc_work(d, &from, &to);
    int at = -1;
    for (int i = (int)std::floor(from * rate); i < (int)std::ceil(to * rate) && i < (int)k.state.size(); ++i) {
        if (k.state[i]) continue;
        at = i;
        break;
    }
    if (at < 0) return false;
    if (!k.target) {
        k.into = Texture::Builder().width(s.w).height(s.h).levels(1).format(Texture::InternalFormat::RGBA8)
                     .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC).build(*s.engine);
        k.target = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, k.into).build(*s.engine);
    }
    const auto t0 = std::chrono::steady_clock::now();
    time_is(s, (double)at / rate);
    const double moment = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();   // (the hand's part, as when it plays)
    RenderTarget* wasOut = s.out;
    Texture* const wasInto = s.drawnInto;
    s.out = k.target;
    if (!s.over) {
        s.drawnInto = k.into;
        s.view->setRenderTarget(k.target);
    }
    auto* pixels = new std::vector<uint8_t>((size_t)s.w * s.h * 4);
    while (!begins(s)) {}
    pictures_of(s);
    s.renderer->readPixels(k.target, 0, 0, (uint32_t)s.w, (uint32_t)s.h,
                           backend::PixelBufferDescriptor(pixels->data(), pixels->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
    s.renderer->endFrame();
    s.engine->flushAndWait();   // (the hand rests: it may wait for the one picture)
    double gpu = 0.0;   // (the GPU's part, as Filament tells it of its last pictures: played, the two go side by side, so either may make it late)
    for (const auto& f : s.renderer->getFrameInfoHistory(2))
        if (f.gpuFrameComplete > 0 && f.backendBeginFrame > 0) gpu = std::max(gpu, (double)(f.gpuFrameComplete - f.backendBeginFrame) / 1e6);
    const double took = std::max(moment, gpu);
    if (std::getenv("MOTOLII_KEEP_LOG")) std::fprintf(stderr, "KEPT %d moment %.1f gpu %.1f ms\n", at, moment, gpu);
    if (!s.over) {
        s.view->setRenderTarget(nullptr);
    }
    s.out = wasOut, s.drawnInto = wasInto;   // (left at that moment: the next picture shown is told its own first, and films step on rather than seek back)
    if (took > budget_ms) {
        ++kept_unwritten;
        kept_write(stage_kept_file(k, at), std::move(*pixels));
        k.state[at] = 2;
        stage_kept_tell(k);
    } else k.state[at] = 1;
    delete pixels;
    return true;
}

// The kept picture of this moment put on the stage instead of drawn. false: none kept for it (draw it).
static bool stage_kept_show(StageKept& k, Stage& s, Live* l, double now, int rate) {
    const int at = (int)std::lround(now * rate);
    if (at < 0 || at >= (int)k.state.size() || k.state[at] != 2) return false;
    const size_t size = (size_t)s.w * s.h * 4;
    const auto read = [size](std::filesystem::path file) { auto* p = new std::vector<uint8_t>(); if (!kept_read(file, *p, size)) delete p, p = nullptr; return p; };
    std::vector<uint8_t>* pixels = nullptr;
    if (k.next.valid()) {
        std::vector<uint8_t>* got = k.next.get();
        if (k.ahead == at) pixels = got; else delete got;
    }
    if (!pixels) pixels = read(stage_kept_file(k, at));   // (not read ahead: the play began here, or skipped)
    if (at + 1 < (int)k.state.size() && k.state[at + 1] == 2) k.ahead = at + 1, k.next = std::async(std::launch::async, read, stage_kept_file(k, at + 1));
    if (!pixels) { k.state[at] = 0; stage_kept_tell(k); return false; }
    if (!k.shown) k.shown = Texture::Builder().width(s.w).height(s.h).levels(1).format(Texture::InternalFormat::RGBA8).usage(Texture::Usage::SAMPLEABLE | Texture::Usage::UPLOADABLE).build(*s.engine);
    k.shown->setImage(*s.engine, 0, backend::PixelBufferDescriptor(pixels->data(), pixels->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE,
                                                                   [](void*, size_t, void* user) { delete (std::vector<uint8_t>*)user; }, pixels));
    live_show(s, l, k.shown);
    return true;
}
