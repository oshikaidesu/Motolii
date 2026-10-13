// motolii.mm: motolii.h's calls, each handed on to doc.h (the document) and kit.h (the stage). Nothing is worked out here.
#include "kit.h"
#include "doc.h"
#include "live.h"
#include "motolii.h"
#include <condition_variable>
#include <deque>
#include <future>
#include <mutex>
#include <thread>
#include <unistd.h>

// One hand does everything that touches the engine: it opens the stage, runs each call as it comes (the caller waits
// for it), and, when the stage is live, draws a picture whenever something has changed or time is going on. The clock
// is the inside's own: the shell is told a picture is finished, it never asks for one to be drawn.
struct Motolii {
    Stage* s = nullptr;
    Doc* d = nullptr;
    Live* live = nullptr;
    std::string trouble, listed, word;
    std::thread hand;
    std::mutex asked_lock;
    std::condition_variable asked_now;
    std::deque<std::function<void()>> asked;
    bool closing = false, changed = true;
    std::atomic<bool> playing{false};
    std::atomic<double> now{0.0};
};

template <class Does> static auto by_hand(Motolii* m, Does does) -> decltype(does()) {
    std::packaged_task<decltype(does())()> task(does);
    auto done = task.get_future();
    {
        std::lock_guard<std::mutex> hold(m->asked_lock);
        m->asked.push_back([&task] { task(); });
    }
    m->asked_now.notify_one();
    return done.get();
}

static void motolii_hand(Motolii* m, std::string document, int wide, int high, std::promise<bool>* opened) {
    m->s = new Stage(open_stage(wide, high));
    m->s->camera->lookAt({0.f, 0.f, 6.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    m->s->camera->setProjection(30.0, double(wide) / double(high), 0.05, 60.0, Camera::Fov::VERTICAL);
    static LinearToneMapper as_it_is;   // (a film, a photo, typed words keep the colours they came with)
    m->s->view->setColorGrading(ColorGrading::Builder().toneMapper(&as_it_is).build(*m->s->engine));
    const bool ok = room(*m->s, "out/small_studio/small_studio_ibl.ktx", 60000.f) && (m->d = doc_open(*m->s, document.c_str()));
    opened->set_value(ok);
    if (!ok) return;
    auto last = std::chrono::steady_clock::now(), due = last;
    for (;;) {
        std::deque<std::function<void()>> todo;
        {
            std::unique_lock<std::mutex> hold(m->asked_lock);
            const bool goes = m->live && (m->playing || m->changed);
            if (goes) m->asked_now.wait_until(hold, due, [m] { return !m->asked.empty() || m->closing; });
            else m->asked_now.wait(hold, [m] { return !m->asked.empty() || m->closing || (m->live && (m->playing || m->changed)); });
            if (m->closing) break;
            todo.swap(m->asked);
        }
        for (auto& does : todo) does(), m->changed = true;
        const auto t = std::chrono::steady_clock::now();
        if (!m->live || t < due || !(m->playing || m->changed)) { last = m->playing ? last : t; continue; }
        if (m->playing) {   // (time goes on by the clock, and round again at the document's end)
            const double runs = doc_seconds(*m->d), on = m->now + std::chrono::duration<double>(t - last).count();
            m->now = on >= runs ? 0.0 : on;
        }
        last = t;
        m->s->by_clock = m->playing;
        time_is(*m->s, m->now);
        live_draw(*m->s, m->live);
        m->changed = false;
        due = std::max(due + std::chrono::microseconds(16667), t);   // (sixty a second)
    }
}

void motolii_home(const char* folder) { if (chdir(folder)) std::printf("motolii_home: %s is not a folder\n", folder); }
Motolii* motolii_open(const char* document, int wide, int high) {
    setenv("BACKEND", "webgpu", 0);   // (the inside is on Filament's WebGPU backend, wherever it is opened from)
    Motolii* m = new Motolii;
    std::promise<bool> opened;
    auto ok = opened.get_future();
    m->hand = std::thread(motolii_hand, m, std::string(document), wide, high, &opened);
    if (ok.get()) return m;
    m->hand.join();
    return nullptr;
}
void motolii_close(Motolii* m) {
    if (!m) return;
    { std::lock_guard<std::mutex> hold(m->asked_lock); m->closing = true; }
    m->asked_now.notify_one();
    m->hand.join();
    if (m->d) sqlite3_close(m->d->file);
}
const char* motolii_trouble(Motolii* m) { return m->trouble.empty() ? m->d->trouble.c_str() : m->trouble.c_str(); }
static int told(Motolii* m, bool ok, const char* otherwise = nullptr) { if (!ok && otherwise) m->trouble = otherwise; return ok ? 1 : 0; }
static Doc& doc(Motolii* m) { m->trouble.clear(); m->d->trouble.clear(); return *m->d; }

// The stage's picture, live: started once; then the last finished picture and how many there have been (any thread).
int motolii_live(Motolii* m) { return by_hand(m, [m] { return m->live || (m->live = live_open(*m->s)) ? 1 : 0; }); }
void* motolii_front(Motolii* m) { return m->live ? live_front(m->live) : nullptr; }
unsigned long long motolii_frames(Motolii* m) { return m->live ? m->live->frames.load() : 0; }
int motolii_play(Motolii* m, int playing) { return by_hand(m, [m, playing] { m->playing = playing != 0; return 1; }); }
double motolii_now(Motolii* m) { return m->now; }

int motolii_add(Motolii* m, const char* kind, const char* file) { return by_hand(m, [=] { return (int)(doc_add(doc(m), kind, file)); }); }
int motolii_remove(Motolii* m, int thing) { return by_hand(m, [=] { return (int)(doc_remove(doc(m), thing)); }); }
int motolii_set(Motolii* m, int thing, const char* name, double number) { return by_hand(m, [=] { return (int)(doc_set(doc(m), thing, name, number)); }); }
int motolii_say(Motolii* m, int thing, const char* name, const char* word) { return by_hand(m, [=] { return (int)(doc_say(doc(m), thing, name, word)); }); }
int motolii_key(Motolii* m, int thing, const char* name, double at, double number) { return by_hand(m, [=] { return (int)(doc_key(doc(m), thing, name, at, number)); }); }
int motolii_unkey(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return (int)(doc_unkey(doc(m), thing, name, at)); }); }
int motolii_keys(Motolii* m, int thing, const char* name) { return by_hand(m, [=] { return (int)(doc_keys(doc(m), thing, name)); }); }
int motolii_way(Motolii* m, int thing, const char* name, double at, const char* way) { return by_hand(m, [=] { return (int)(doc_way(doc(m), thing, name, at, way)); }); }
int motolii_define(Motolii* m, const char* way, const char* expression) { return by_hand(m, [=] { return (int)(doc_define(doc(m), way, expression)); }); }
const char* motolii_ways(Motolii* m) { return by_hand(m, [m] { return (m->listed = doc_ways(*m->d)).c_str(); }); }
int motolii_value(Motolii* m, int thing, const char* name, double at, double* is) { return by_hand(m, [=] { return (int)(doc_value(doc(m), thing, name, at, is)); }); }
int motolii_things(Motolii* m, int* things, int room) {
    return by_hand(m, [=] { const std::vector<int> all = doc_things(*m->d); for (int i = 0; i < room && i < (int)all.size(); ++i) things[i] = all[(size_t)i]; return (int)all.size(); });
}
const char* motolii_word(Motolii* m, int thing, const char* name) { return by_hand(m, [=] { return (m->word = doc_word(*m->d, thing, name)).c_str(); }); }
double motolii_lasts(Motolii* m, int thing) { return by_hand(m, [=] { return doc_lasts(*m->d, thing); }); }
int motolii_key_moments(Motolii* m, int thing, const char* name, double* at, int room) {
    return by_hand(m, [=] { const std::vector<double> all = doc_key_moments(*m->d, thing, name); for (int i = 0; i < room && i < (int)all.size(); ++i) at[i] = all[(size_t)i]; return (int)all.size(); });
}
// A thing's frame on the picture at the stage's moment, as the stage's own eye sees it: its middle, how wide and how
// high (points of the picture, from the top left), and how far it is turned (degrees, clockwise as seen).
int motolii_frame(Motolii* m, int thing, double* frame) {
    return by_hand(m, [=] {
        float2 half{0.f};
        if (!doc_knows(doc(m), thing) || !doc_half(*m->d, thing, &half)) return 0;
        Doc& d = *m->d;
        const double now = m->now, size = doc_number(d, thing, "size", now, 1);
        const math::double3 at{doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0)};
        const math::mat4 eye = m->s->camera->getProjectionMatrix() * m->s->camera->getViewMatrix();
        const auto seen = [&](math::double3 p) { const math::double4 c = eye * math::double4{p, 1.0}; return math::double2{(0.5 + 0.5 * c.x / c.w) * m->s->w, (0.5 - 0.5 * c.y / c.w) * m->s->h}; };
        const math::double2 middle = seen(at), right = seen(at + math::double3{1, 0, 0});
        const double point = right.x - middle.x;   // (how many points of the picture one unit of the stage is, where the thing stands)
        frame[0] = middle.x, frame[1] = middle.y, frame[2] = 2.0 * half.x * size * point, frame[3] = 2.0 * half.y * size * point;
        frame[4] = -doc_number(d, thing, "rotate", now, 0);
        return 1;
    });
}
int motolii_try(Motolii* m, int thing, const char* name, double number) { return by_hand(m, [=] { return (int)doc_try(doc(m), thing, name, number); }); }
int motolii_try_drop(Motolii* m) { return by_hand(m, [=] { return doc_try_drop(*m->d), 1; }); }
// How many points of the picture one unit of the stage is, where things stand when their z is 0.
double motolii_point(Motolii* m) {
    return by_hand(m, [=] {
        const math::mat4 eye = m->s->camera->getProjectionMatrix() * m->s->camera->getViewMatrix();
        const math::double4 a = eye * math::double4{0, 0, 0, 1}, b = eye * math::double4{1, 0, 0, 1};
        return 0.5 * (b.x / b.w - a.x / a.w) * m->s->w;
    });
}
int motolii_undo(Motolii* m) { return by_hand(m, [=] { return (int)(doc_undo(doc(m))); }); }
int motolii_redo(Motolii* m) { return by_hand(m, [=] { return (int)(doc_redo(doc(m))); }); }

double motolii_seconds(Motolii* m) { return by_hand(m, [m] { return doc_seconds(*m->d); }); }
int motolii_at(Motolii* m, double seconds) {
    return by_hand(m, [m, seconds] {
        char runs[80];
        std::snprintf(runs, sizeof runs, "the document runs from 0 to %.2f seconds", doc_seconds(doc(m)));
        if (!told(m, seconds >= 0.0 && seconds <= doc_seconds(*m->d), runs)) return 0;
        m->now = seconds;
        return time_is(*m->s, seconds), 1;
    });
}
static bool ends(const char* name, const char* with) { return strlen(name) > strlen(with) && !strcmp(name + strlen(name) - strlen(with), with); }
static bool placed(const char* name) { return std::filesystem::is_directory(std::filesystem::absolute(name).parent_path()); }
int motolii_still(Motolii* m, const char* png) {
    return by_hand(m, [m, png] {
        doc(m);
        if (!told(m, ends(png, ".png"), "a still is a .png") || !told(m, placed(png), "there is no such folder")) return 0;
        time_is(*m->s, m->now);
        return told(m, shoot(*m->s, png), "the file could not be written");
    });
}
int motolii_write(Motolii* m, const char* mp4) {
    return by_hand(m, [m, mp4] {
        doc(m);
        if (!told(m, ends(mp4, ".mp4"), "a film is written as an .mp4") || !told(m, placed(mp4), "there is no such folder")) return 0;
        const int ok = told(m, film(*m->s, mp4, (int)std::lround(doc_seconds(*m->d) * 60.0), [](float) {}), "the file could not be written");
        time_is(*m->s, m->now);   // (back to the moment it was at)
        return ok;
    });
}
const char* motolii_list(Motolii* m) {
    return by_hand(m, [m] {
    m->listed.clear();
    doc_rows(*m->d, sqlite3_mprintf("select thing || ' ' || name || ' at ' || at || ': ' || value || coalesce(', then ' || way, '') from fact order by thing, name, at"), [m](sqlite3_stmt* row) { m->listed += std::string("  ") + (const char*)sqlite3_column_text(row, 0) + "\n"; });
    return m->listed.c_str();
    });
}
void motolii_timed(Motolii* m, int pictures) { by_hand(m, [m, pictures] { clock(*m->s, pictures, [](float) {}); return 0; }); }
