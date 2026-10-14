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
    const bool says = doc_size(document.c_str(), &wide, &high);   // (a document that says its own size is opened at it)
    m->s = new Stage(open_stage(wide, high));
    m->s->camera->lookAt({0.f, 0.f, 6.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    m->s->camera->setProjection(30.0, double(wide) / double(high), 0.05, 60.0, Camera::Fov::VERTICAL);
    m->s->view->setAmbientOcclusionOptions({.enabled = true});
    // (a 3D file may call its faces light-passing: Filament draws such faces in a turn of their own, after the solid
    //  things and before every sheet, so any sheet, however far behind, was painted over them. With that turn off
    //  they are drawn with the solid things, and depth decides. The kit's own glass does not use that turn.)
    m->s->view->setScreenSpaceRefractionEnabled(false);   // (Filament's own: where faces meet it is darker, so a plain solid reads as a solid)
    static LinearToneMapper as_it_is;   // (a film, a photo, typed words keep the colours they came with)
    m->s->view->setColorGrading(ColorGrading::Builder().toneMapper(&as_it_is).build(*m->s->engine));
    const bool ok = room(*m->s, "out/small_studio/small_studio_ibl.ktx", 60000.f) && (m->d = doc_open(*m->s, document.c_str()));
    if (ok && !says) doc_size_is(m->d->file, wide, high);   // (a new one is the size it was first opened at)
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
            double from = 0.0, to = runs;
            doc_work(*m->d, &from, &to);   // (round and round within the work area)
            m->now = on >= to || on < from ? from : on;
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
// What a call that changes the document answers. Refused within an act, at whatever door (a number out of its range
// is refused before anything is changed), the act it is part of is not made.
static int changes(Motolii* m, int done) {
    if (!done && m->d->act) m->d->act_ok = false;
    return done;
}

// The stage's picture, live: started once; then the last finished picture and how many there have been (any thread).
int motolii_live(Motolii* m) { return by_hand(m, [m] { return m->live || (m->live = live_open(*m->s)) ? 1 : 0; }); }
void* motolii_front(Motolii* m) { return m->live ? live_front(m->live) : nullptr; }
unsigned long long motolii_frames(Motolii* m) { return m->live ? m->live->frames.load() : 0; }
int motolii_play(Motolii* m, int playing) { return by_hand(m, [m, playing] { m->playing = playing != 0; return 1; }); }
double motolii_now(Motolii* m) { return m->now; }

int motolii_add(Motolii* m, const char* kind, const char* file) { return by_hand(m, [=] { return changes(m, (int)(doc_add(doc(m), kind, file))); }); }
int motolii_remove(Motolii* m, int thing) { return by_hand(m, [=] { return changes(m, (int)(doc_remove(doc(m), thing))); }); }
int motolii_set(Motolii* m, int thing, const char* name, double number) { return by_hand(m, [=] { return changes(m, (int)(doc_set(doc(m), thing, name, number))); }); }
int motolii_say(Motolii* m, int thing, const char* name, const char* word) { return by_hand(m, [=] { return changes(m, (int)(doc_say(doc(m), thing, name, word))); }); }
int motolii_key(Motolii* m, int thing, const char* name, double at, double number) { return by_hand(m, [=] { return changes(m, (int)(doc_key(doc(m), thing, name, at, number))); }); }
int motolii_unkey(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)(doc_unkey(doc(m), thing, name, at))); }); }
int motolii_keys(Motolii* m, int thing, const char* name) { return by_hand(m, [=] { return (int)(doc_keys(doc(m), thing, name)); }); }
int motolii_way(Motolii* m, int thing, const char* name, double at, const char* way) { return by_hand(m, [=] { return changes(m, (int)(doc_way(doc(m), thing, name, at, way))); }); }
int motolii_define(Motolii* m, const char* way, const char* expression) { return by_hand(m, [=] { return changes(m, (int)(doc_define(doc(m), way, expression))); }); }
const char* motolii_ways(Motolii* m) { return by_hand(m, [m] { return (m->listed = doc_ways(*m->d)).c_str(); }); }
int motolii_value(Motolii* m, int thing, const char* name, double at, double* is) { return by_hand(m, [=] { return (int)(doc_value(doc(m), thing, name, at, is)); }); }
int motolii_things(Motolii* m, int* things, int room) {
    return by_hand(m, [=] { const std::vector<int> all = doc_things(*m->d); for (int i = 0; i < room && i < (int)all.size(); ++i) things[i] = all[(size_t)i]; return (int)all.size(); });
}
const char* motolii_word(Motolii* m, int thing, const char* name) { return by_hand(m, [=] { return (m->word = !strcmp(name, "name") ? doc_name(*m->d, thing) : doc_word(*m->d, thing, name)).c_str(); }); }
double motolii_lasts(Motolii* m, int thing) { return by_hand(m, [=] { return doc_lasts(*m->d, thing); }); }
int motolii_key_moments(Motolii* m, int thing, const char* name, double* at, int room) {
    return by_hand(m, [=] { const std::vector<double> all = doc_key_moments(*m->d, thing, name); for (int i = 0; i < room && i < (int)all.size(); ++i) at[i] = all[(size_t)i]; return (int)all.size(); });
}
// A thing's frame on the picture at the stage's moment, as the stage's own eye sees it: its middle, how wide and how
// high (points of the picture, from the top left), and how far it is turned (degrees, clockwise as seen).
int motolii_frame(Motolii* m, int thing, double* frame) { return by_hand(m, [=] { return (int)(doc_knows(doc(m), thing) && doc_frame(*m->d, thing, m->now, frame)); }); }
int motolii_keying(Motolii* m, int on) {   // (on below 0: only says whether every change is keyed)
    return by_hand(m, [=] { if (on >= 0) m->d->keying = on != 0, m->d->keying_from = m->now; return (int)m->d->keying; });
}
int motolii_hold(Motolii* m, int thing) { return by_hand(m, [=] { return (int)doc_hold(doc(m), thing); }); }
int motolii_carry(Motolii* m, const char* by, double a, double b) { return by_hand(m, [=] { return (int)doc_carry(doc(m), by, a, b); }); }
int motolii_let_go(Motolii* m, int keep) { return by_hand(m, [=] { return changes(m, (int)doc_let_go(doc(m), keep != 0)); }); }
int motolii_put(Motolii* m, int thing, const char* name, double at, double number) { return by_hand(m, [=] { return changes(m, (int)doc_put(doc(m), thing, name, at, number)); }); }
int motolii_key_here(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)doc_key_here(doc(m), thing, name, at)); }); }
int motolii_size(Motolii* m, int wide, int high) {
    return by_hand(m, [=] {
        char* say = doc_size_words(wide, high);
        return say ? changes(m, (int)doc_change(*m->d, say)) : (m->d->trouble = "a stage is from 16 to 8192 points each way, an even number of them", 0);
    });
}
int motolii_work(Motolii* m, double from, double to) { return by_hand(m, [=] { return changes(m, (int)doc_work_is(doc(m), from, to)); }); }
int motolii_work_of(Motolii* m, double* from, double* to) { return by_hand(m, [=] { return doc_work(*m->d, from, to), 1; }); }
int motolii_mark(Motolii* m, double at, const char* word) { return by_hand(m, [=] { return changes(m, (int)doc_mark(doc(m), at, word)); }); }
const char* motolii_marks(Motolii* m) { return by_hand(m, [=] { return (m->word = doc_marks(*m->d)).c_str(); }); }
int motolii_fits(Motolii* m) {
    return by_hand(m, [=] {
        int wide = m->s->w, high = m->s->h;
        doc_rows(*m->d, sqlite3_mprintf("select name, value from fact where thing = 0 and name in ('wide', 'high')"),
                 [&](sqlite3_stmt* row) { (strcmp((const char*)sqlite3_column_text(row, 0), "wide") ? high : wide) = sqlite3_column_int(row, 1); });
        return (int)(wide == m->s->w && high == m->s->h);
    });
}
int motolii_size_of(Motolii* m, int* wide, int* high) { return by_hand(m, [=] { return *wide = m->s->w, *high = m->s->h, 1; }); }
int motolii_raise(Motolii* m, int thing, int places) { return by_hand(m, [=] { return changes(m, (int)doc_raise(doc(m), thing, places)); }); }
int motolii_stay(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)doc_stay(doc(m), thing, name, at)); }); }
int motolii_keyed(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return doc_keyed(*m->d, thing, name, at); }); }
const char* motolii_words(Motolii* m, int thing, const char* name) { return by_hand(m, [=] { return (m->word = doc_words(*m->d, thing, name)).c_str(); }); }
const char* motolii_moving(Motolii* m, int thing) { return by_hand(m, [=] { return (m->word = doc_moving(*m->d, thing)).c_str(); }); }
const char* motolii_kind_of(const char* file) { return doc_kind_of(file); }
int motolii_sets(const char* font, const char* words) { return type_sets(font, words) ? 1 : 0; }
// The names of the ways a stretch may go (the nine kinds, then the ways defined in this document), with spaces between;
// and how one of the nine goes: how far it is, from 0 at its beginning to 1 at its end, at `u` of the way through.
const char* motolii_way_names(Motolii* m) {
    return by_hand(m, [=] {
        m->word.clear();
        for (const auto& kind : interp_kinds) m->word += std::string(m->word.empty() ? "" : " ") + kind.name;
        doc_rows(*m->d, sqlite3_mprintf("select name from way order by name"), [&](sqlite3_stmt* row) { m->word += std::string(" ") + (const char*)sqlite3_column_text(row, 0); });
        return m->word.c_str();
    });
}
double motolii_way_at(Motolii* m, const char* way, double u) {
    Interp kind;
    if (interp_said(way, &kind) == 0) return interp_ease(kind, u);
    return by_hand(m, [=] { return doc_went_by(*m->d, way, u, u, 1.0, 0.0, 1.0); });   // (one defined in the document: as between a key of 0 and a key of 1, a second apart)
}
// A film's first picture as a small .png: GStreamer's own line for it (decode, scale, into a .png), held at its first
// picture, which is taken and written. It asks nothing of a stage, so a shell may call it from another thread.
int motolii_thumbnail(const char* film, const char* png, int wide) {
    gst_init(nullptr, nullptr);
    gchar* uri = gst_filename_to_uri(film, nullptr);
    if (!uri) return 0;
    gchar* line = g_strdup_printf("uridecodebin uri=\"%s\" ! videoconvert ! videoscale ! video/x-raw,width=%d,pixel-aspect-ratio=1/1 ! pngenc ! appsink name=out", uri, wide);
    GstElement* made = gst_parse_launch(line, nullptr);
    g_free(line), g_free(uri);
    if (!made) return 0;
    GstElement* out = gst_bin_get_by_name(GST_BIN(made), "out");
    gst_element_set_state(made, GST_STATE_PAUSED);
    GstSample* first = gst_app_sink_try_pull_preroll(GST_APP_SINK(out), 4 * GST_SECOND);
    bool done = false;
    GstMapInfo bytes;
    if (first && gst_buffer_map(gst_sample_get_buffer(first), &bytes, GST_MAP_READ)) {
        done = g_file_set_contents(png, (const gchar*)bytes.data, (gssize)bytes.size, nullptr);
        gst_buffer_unmap(gst_sample_get_buffer(first), &bytes);
    }
    if (first) gst_sample_unref(first);
    gst_element_set_state(made, GST_STATE_NULL);
    gst_object_unref(out);
    gst_object_unref(made);
    return done;
}
int motolii_together(Motolii* m, int on) { return by_hand(m, [=] { return on ? (doc_act(*m->d), 1) : (m->d->act ? (int)doc_acted(*m->d) : 0); }); }
const char* motolii_way_of(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return (m->word = doc_way_of(*m->d, thing, name, at)).c_str(); }); }
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
static const int RATE = 60;   // (pictures a second of a written film: the kit's `film` writes that many)
int motolii_rate(Motolii*) { return RATE; }
int motolii_write(Motolii* m, const char* mp4) {
    return by_hand(m, [m, mp4] {
        doc(m);
        if (!told(m, ends(mp4, ".mp4"), "a film is written as an .mp4") || !told(m, placed(mp4), "there is no such folder") || !told(m, !m->d->writing, "it is being written already")) return 0;
        double from = 0.0, to = 0.0;
        doc_work(*m->d, &from, &to);   // (the work area is what is written)
        const int pictures = (int)std::lround((to - from) * RATE);
        m->d->writing = true;
        m->d->written_from = from;
        // (between two pictures the hand does what it was asked meanwhile, so a shell that asks does not stand still;
        //  the document is read then, not changed, and the picture being written is told its moment again after)
        const int ok = told(m, film(*m->s, mp4, pictures, [m, pictures, from](float done) {
            std::deque<std::function<void()>> todo;
            { std::lock_guard<std::mutex> hold(m->asked_lock); todo.swap(m->asked); }
            for (auto& does : todo) does();
            time_is(*m->s, from + (double)std::lround(done * (float)pictures) / RATE);   // (the picture's moment: from the work area's start)
        }), "the file could not be written");
        m->d->writing = false;
        m->d->written_from = 0.0;
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
