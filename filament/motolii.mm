// motolii.mm: motolii.h's calls, each handed on to doc.h (the document) and kit.h (the stage). Nothing is worked out here.
#include <CoreText/CoreText.h>
#include "kit.h"
#include "doc.h"
#include "live.h"
#include "stage_kept.h"
#include "heard.h"
#include "sound.h"
#include <CoreText/CoreText.h>
#include "motolii.h"
#include "rendering/FrameScheduler.hpp"   // (third_party/thermion_frame: the screen's own beat)
#include <set>
#include <sstream>
#include <atomic>
#include <condition_variable>
#include <deque>
#include <future>
#include <mutex>
#include <thread>
#include <pthread.h>
#include <sys/qos.h>
#include <unistd.h>

// One hand does everything that touches the engine: it opens the stage, runs each call as it comes (the caller waits
// for it), and, when the stage is live, draws a picture whenever something has changed or time is going on. The clock
// is the inside's own: the shell is told a picture is finished, it never asks for one to be drawn.
static const int RATE = 60;   // (pictures a second of a written film: the kit's `film` writes that many)
struct Motolii {
    Stage* s = nullptr;
    Doc* d = nullptr;
    Live* live = nullptr;
    int pick = 0, hover = 0;   // the thing picked in the window and the one under its hand: marked on the desk (motolii_pick)
    std::set<int> watched;     // the things the window shows the moving values of (later "watch", Anthem's subscription): in the moment's picture
    std::map<const void*, Heard> heard;   // the things' sound heard as the live stage plays (heard.h)
    std::string trouble, listed, word;
    std::thread hand;
    std::mutex asked_lock;
    std::condition_variable asked_now;
    std::unique_ptr<thermion::FrameScheduler> beat;   // (the screen's beat, Thermion's: a picture is drawn on each; CVDisplayLink, DXGI on Windows)
    uint64_t beats = 0, drew_at = 0;                  // (beats come, and the one the last picture was drawn on: under asked_lock)
    long beats_drawn = 0, beats_gpu_busy = 0, beats_still = 0, writes_on_beat = 0, beats_two = 0, beats_unseen = 0; uint64_t busy_at = 0, still_at = 0, seen_beat = 0; double films_ms = 0, films_most = 0, part_ms[7] = {}, part_most[7] = {};   // (the hand's round in parts: files looked at, changes, telling, GPU asked, ahead, the rest)   // (beats a picture was drawn on, and beats the GPU had not finished the last: for the spent line)
    std::atomic<void (*)(void*)> shown{nullptr};      // (motolii_finished: told when a picture is finished, to tell Flutter)
    std::atomic<void*> shown_with{nullptr};
    std::deque<std::function<void()>> asked;
    std::map<std::string, std::shared_ptr<std::function<void()>>> latest;
    int tickets = 0;                                             // (motolii_later: the last ticket given)
    std::map<int, std::pair<int, std::string>> answers;          // (and each change's answer once done: done or not, and why not)
    std::string answered_why;                                    // (the last answer's why, told to the shell)   // (posted and not done yet, by what it is: a later one of the same takes its place)
    bool closing = false, changed = true;
    std::atomic<bool> playing{false};
    std::chrono::steady_clock::time_point stirred{};   // (the hand's last stir: something asked, or playing, or changed)
    bool ahead = false;                                  // (films' frames ahead being kept while the hand rests: picture_ahead)
    std::atomic<bool> loop{true};   // (play goes round the work area; not: it stops at its end. Atomic: asked without the document)
    std::atomic<bool> keying{false};   // (the document's own, mirrored: asked each picture without waiting for the document)
    std::atomic<bool> telling{false};  // (the hand is telling the stage a moment, the document held: a shell reading now would wait)
    std::atomic<double> now{0.0};
    std::map<std::string, std::pair<long, double>> by_door;   // (what each call has cost, by its name: for measuring)
    std::map<std::string, double> worst;                      // (and its slowest once)
    int made = 0;   // (the thing the last add or make asked later made: a later call for thing -1 means it)
    std::atomic<long> revision{0};   // (how many times the document has changed: read by anyone, without the document's lock)
    std::atomic<long> carried{0};
    std::atomic<bool> watching{true};   // (motolii_watch: the window's thread may not wait on the hand; what did is kept in `broke`, the last 200; watched from the start)
    std::deque<std::string> broke;
    std::atomic<long> broken{0};   // (how many times, since watched)    // (and how many times what a hand carries has: values a window shows while a hand is on them)
    std::map<std::string, std::array<double, 4>> window;     // (a call from the window's own thread, by name: how many, waited in all, waited at most, took in all: what the window waits on)
    double read_waited = 0, read_waited_most = 0; long read_waits = 0;
    std::map<std::string, std::pair<double, long>> rounds;   // (a call through the hand: its longest wait for the answer, and how many waited over 4 ms)   // (reads waiting for the document: in all, the longest, how many waited over 2 ms)
    double budget = std::getenv("MOTOLII_BUDGET") ? std::atof(std::getenv("MOTOLII_BUDGET")) : 12.0;   // (the milliseconds the GPU may take for a live picture, of the 16.7 a picture has)
    long judged = 0;
    double gpu_ms = 0.0, gpu_most = 0.0;   // (the GPU's middle time for the last pictures, and the most it was)
    double scale = 1.0;   // (how large the stage's views are drawn now: stage_scale)
    StageKept kept;       // (the stage's pictures kept on disk for playing: stage_kept.h)
    double spent[4] = {0, 0, 0, 0};   // (seconds the hand spent: asked to change, asked to read, telling the moment, drawing)
    long many[4] = {0, 0, 0, 0};
    // The window's pictures of the document (picture.h, skin/BRIDGE.md), set down by the hand, read under their own lock.
    std::mutex pictureLock;
    std::map<int, std::pair<long, std::string>> pictured;   // (each thing's part of the document's picture, and the version it was made at)
    std::string pictureAll, momentPicture;                  // (the document's own part; the moment's picture)
    long pictureVersion = 0, momentVersion = 0;
    int pictureTold = 0;                                     // (which are new since the window was told: 1 the document's, 2 the moment's)
    long keptTold = 0;                                       // (what of the kept pictures the window was last told of)
    void (*told)(int) = nullptr;                             // (the window's one listener)
};
static void picture_update(Motolii* m);
static void picture_moment(Motolii* m);
static void picture_tell(Motolii* m);

// While it plays and the GPU takes longer than it may, the stage's views are drawn smaller and made large again (as
// Filament's own dynamic resolution does; its own reckoning needs timer queries Dawn's backend has not, so the scale is
// set here from the pictures' own times); at rest, full size, for looking. MOTOLII_SCALE: a scale kept, to measure.
static void stage_scale(Motolii* m) {
    static const double kept = std::getenv("MOTOLII_SCALE") ? std::atof(std::getenv("MOTOLII_SCALE")) : 0.0;
    double want = 1.0;
    if (kept > 0.0) want = kept;
    else if (m->playing && m->gpu_ms > 0.0) want = std::clamp(m->scale * std::sqrt(m->budget / m->gpu_ms), 0.5, 1.0);   // (the GPU's time goes with the pixels drawn: as many fewer as it is over)
    if (std::fabs(want - m->scale) < 0.05 && !(want == 1.0 && m->scale != 1.0)) return;
    m->scale = want;
    View::DynamicResolutionOptions o;
    o.enabled = want < 1.0, o.minScale = o.maxScale = math::float2{(float)want}, o.quality = QualityLevel::MEDIUM;
    m->s->view->setDynamicResolutionOptions(o);
}
template <class Does> static auto asked(Motolii* m, Does does, bool changes, const char* who) -> decltype(does()) {
    std::packaged_task<decltype(does())()> task(does);
    auto done = task.get_future();
    {
        std::lock_guard<std::mutex> hold(m->asked_lock);
        m->asked.push_back([&task, m, changes, who] {
            const auto t0 = std::chrono::steady_clock::now();
            task();
            if (changes) m->changed = true;
            const double took = std::chrono::duration<double>(std::chrono::steady_clock::now() - t0).count();
            m->spent[changes ? 0 : 1] += took, m->many[changes ? 0 : 1]++;
            auto& door = m->by_door[who];
            door.first++, door.second += took, m->worst[who] = std::max(m->worst[who], took);
        });
    }
    m->asked_now.notify_one();
    const auto sent = std::chrono::steady_clock::now();
    auto out = done.get();
    const double round = std::chrono::duration<double>(std::chrono::steady_clock::now() - sent).count();   // (the shell waited this long for the hand: in the queue and at work)
    { std::lock_guard<std::mutex> hold(m->asked_lock); auto& r = m->rounds[who]; r.first = std::max(r.first, round), r.second += round > 0.004;
      if (pthread_main_np() && m->watching) { char why[160]; std::snprintf(why, sizeof why, "%s waited %.1f ms for the hand (a change the window may only send: motolii_later)", who, 1e3 * round); m->broke.push_back(why); if (m->broke.size() > 200) m->broke.pop_front(); m->broken++; }
      if (pthread_main_np()) { auto& w = m->window[who]; w[0]++, w[1] += round, w[2] = std::max(w[2], round), w[3] += round; } }
    return out;
}
// Asked of the hand: by_hand for what may change what is seen (the stage is drawn again after), by_look for what only
// reads (nothing drawn again: a shell that reads its cards a hundred times a picture never waits on a hundred pictures).
template <class Does> static auto by_hand(Motolii* m, Does does, const char* who = __builtin_FUNCTION()) -> decltype(does()) {
    return asked(m, [m, does] {   // (the document changed: its revision counted before the asker hears the answer, so a copy it keeps is never read stale)
        const auto now = [m] { return m->d ? (long)sqlite3_total_changes(m->d->file) : 0L; };
        const long rows = now();
        auto out = does();
        if (now() != rows) m->revision.fetch_add(1, std::memory_order_release), picture_update(m), picture_moment(m);   // (the pictures new before the asker hears the answer)
        return out;
    }, true, who);
}
// A read is not queued behind the hand: it is done at once, on the asker's own thread, while it holds the document
// (the hand holds it only while it changes the document or tells it a moment, never while a picture is drawn), so a
// shell's hundreds of reads a build never wait on a picture. (The Flutter kin read so: research_notes/flutter_native_kin.md)
// A change the shell does not wait for (where a hand is, the moment the playhead is put at, play or stop): put in the
// hand's queue and answered at once; one of the same kind still waiting there is replaced by it, keeping its place, so
// a hand moving faster than the pictures leaves only where it is now. What it says absolutely (where the hand is, not
// how far it moved), the document's own reckoning.
static int posted(Motolii* m, const std::string& kind, std::function<void()> does) {
    {
        std::lock_guard<std::mutex> hold(m->asked_lock);
        auto& waiting = m->latest[kind];
        if (waiting) *waiting = std::move(does);
        else {
            waiting = std::make_shared<std::function<void()>>(std::move(does));
            m->asked.push_back([m, kind] {
                std::function<void()> now;
                { std::lock_guard<std::mutex> hold(m->asked_lock); now = std::move(*m->latest[kind]); m->latest.erase(kind); }
                now();
                m->changed = true;
            });
        }
    }
    m->asked_now.notify_one();
    return 1;
}
// A read of the document, done on the asker's thread while it holds the document: the hand's own (making the window's
// pictures, picture.h), a question asked on another isolate, a check. The window's thread asks none (skin/BRIDGE.md).
template <class Does> static auto by_look(Motolii* m, Does does, const char* who = __builtin_FUNCTION()) -> decltype(does()) {
    const auto asked = std::chrono::steady_clock::now();
    std::unique_lock<std::recursive_timed_mutex> hold(m->d->lock);
    const auto t0 = std::chrono::steady_clock::now();
    const double waited = std::chrono::duration<double>(t0 - asked).count();   // (a read waits while the hand holds the document: telling a moment, making a change)
    m->read_waited += waited, m->read_waited_most = std::max(m->read_waited_most, waited), m->read_waits += waited > 0.002;
    auto out = does();
    const double took = std::chrono::duration<double>(std::chrono::steady_clock::now() - t0).count();
    m->spent[1] += took, m->many[1]++;
    auto& door = m->by_door[who];
    door.first++, door.second += took, m->worst[who] = std::max(m->worst[who], took);
    if (pthread_main_np()) { auto& w = m->window[who]; w[0]++, w[1] += waited, w[2] = std::max(w[2], waited), w[3] += took; }
    if (pthread_main_np() && m->watching && waited > 0.002) { char why[160]; std::snprintf(why, sizeof why, "%s waited %.1f ms for the document", who, 1e3 * waited); std::lock_guard<std::mutex> h(m->asked_lock); m->broke.push_back(why); if (m->broke.size() > 200) m->broke.pop_front(); m->broken++; }
    return out;
}

static void motolii_hand(Motolii* m, std::string document, int wide, int high, std::promise<bool>* opened) {
    pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);   // (the hand makes the pictures a window shows: macOS's own word for that, on its fast cores)
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
    if (ok && !says) doc_size_is(m->d->file, wide, high), doc_forget(*m->d);   // (a new one is the size it was first opened at; what was read before is read again)
    if (ok) {   // (the window's first pictures of it)
        std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
        picture_update(m), picture_moment(m);
    }
    opened->set_value(ok);
    if (!ok) return;
    auto last = std::chrono::steady_clock::now();
    m->beat.reset(thermion::FrameScheduler::create(RATE));
    m->beat->setTargetFps(RATE);
    m->beat->start([](uint64_t, void* p) {
        auto* m = static_cast<Motolii*>(p);
        { std::lock_guard<std::mutex> hold(m->asked_lock); ++m->beats; }
        m->asked_now.notify_one();
    }, m);
    for (;;) {
    try {   // (a throw in the hand's round, a Filament precondition, say: its words said, the round given up: the hand goes on, as the window's own does)

        std::deque<std::function<void()>> todo;
        uint64_t beat;
        {
            std::unique_lock<std::mutex> hold(m->asked_lock);
            const bool goes = m->live && (m->playing || m->changed), begun = m->live && m->live->pending;   // (begun: a picture the GPU has not finished)
            if (begun) m->asked_now.wait_for(hold, std::chrono::milliseconds(1), [m] { return !m->asked.empty() || m->closing; });   // (looked at again a moment later)
            else if (goes) m->asked_now.wait(hold, [m] { return !m->asked.empty() || m->closing || m->beats != m->drew_at; });
            else m->asked_now.wait_for(hold, std::chrono::milliseconds(m->ahead ? 15 : 500), [m] { return !m->asked.empty() || m->closing || (m->live && (m->playing || m->changed)); });   // (twice a second: a file written over; often, while frames ahead are kept)
            if (m->closing) break;
            todo.swap(m->asked);
            beat = m->beats;
            if (m->seen_beat && beat > m->seen_beat + 1) m->beats_unseen += (long)(beat - m->seen_beat - 1);   // (beats that came while the hand was busy elsewhere)
            m->seen_beat = beat;
        }
        auto lap_at = std::chrono::steady_clock::now();
        const auto lap = [&](int i) { const auto n = std::chrono::steady_clock::now(); const double ms = 1e3 * std::chrono::duration<double>(n - lap_at).count(); m->part_ms[i] += ms; m->part_most[i] = std::max(m->part_most[i], ms); lap_at = n; };
        if (todo.empty() && m->live && !m->playing && !m->changed) {
            std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
            if (doc_files_moved(*m->d)) m->changed = true;
        }   // (a shader or a script saved: drawn again)
        lap(0);
        {
            std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
            const long rows = sqlite3_total_changes(m->d->file);
            const auto held = m->d->held.tried;
            for (auto& does : todo) does();
            if (!todo.empty()) ++m->writes_on_beat;
            lap(6);
            if (sqlite3_total_changes(m->d->file) != rows) picture_update(m);   // (the window's picture of the document, where it changed)
            if (sqlite3_total_changes(m->d->file) != rows || m->d->held.tried != held)
                (sqlite3_total_changes(m->d->file) != rows ? m->revision : m->carried).fetch_add(1, std::memory_order_release);   // (the document changed, or only what a hand carries: the kept pictures go by these)
        }
        lap(1);
        if (mosh_done.exchange(false)) {   // (a datamosh made meanwhile: the thing made again with it)
            std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
            doc_match(*m->d);
            m->changed = true;
        }
        picture_tell(m);   // (the document let go: the window told what is new)
        if (!m->playing) for (auto& [who, h] : m->heard) if (h.on) gst_element_set_state(h.bin, GST_STATE_PAUSED), h.on = false;   // (the stage stopped: so does its sound)
        lap(2);
        const auto finished = [m] { return m->live ? m->live->frames.load() + (m->live->desk ? m->live->desk->frames.load() : 0) : 0ull; };
        const auto had = finished();
        const bool ready = !m->live || live_ready(*m->s, m->live);   // (the picture begun before, offered once the GPU has finished it)
        if (finished() != had) if (auto tell = m->shown.load()) tell(m->shown_with.load());   // (and Flutter told: Thermion's markTextureFrameAvailable)
        lap(3);
        const auto t = std::chrono::steady_clock::now();
        if (!todo.empty() || m->playing || m->changed) m->stirred = t;
        m->ahead = false;
        if (m->live && ready && todo.empty() && !m->playing && !m->changed && t - m->stirred > std::chrono::milliseconds(300)) {   // (the hand rests: a few of each film's frames ahead kept, as Final Cut Pro renders once the hand stops)
            std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
            int begun = 0;
            for (auto& [thing, shown] : m->d->shown)
                if (shown.drawn) begun += picture_ahead(*m->s, shown.drawn, 2);
            if (begun) m->s->engine->flush();
            m->ahead = begun > 0;
            if (t - m->stirred > std::chrono::seconds(5) && !std::getenv("MOTOLII_KEEP_NONE")) {   // (and, five seconds on as Final Cut Pro's background render, the work area's pictures that would be late, kept on disk)
                stage_kept_fresh(m->kept, *m->d, *m->s, m->revision.load(), RATE);
                m->ahead = stage_kept_one(m->kept, *m->d, *m->s, RATE, m->budget) || m->ahead;
            }
        }
        lap(4);
        if (m->live && !m->playing && m->scale != 1.0 && !std::getenv("MOTOLII_SCALE")) stage_scale(m), m->changed = true;   // (stopped: the last picture drawn again full size, for looking)
        if (m->live && beat != m->drew_at && (m->playing || m->changed) && !ready && beat != m->busy_at) m->busy_at = beat, ++m->beats_gpu_busy;
        if (m->live && beat != m->drew_at && !m->playing && !m->changed && beat != m->still_at) m->still_at = beat, ++m->beats_still;   // (a beat with nothing new to draw)
        if (!m->live || beat == m->drew_at || !(m->playing || m->changed) || !ready) { last = m->playing || !ready ? last : t; continue; }   // (not ready: no other picture is begun till it is)
        if (m->playing) {   // (time goes on by the clock, and round again at the document's end)
            std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
            const double runs = doc_seconds(*m->d), on = m->now + std::chrono::duration<double>(t - last).count();
            double from = 0.0, to = runs;
            doc_work(*m->d, &from, &to);   // (round and round within the work area)
            if (on >= to && !m->loop) m->now = to, m->playing = false;   // (once through: it stops at the work area's end)
            else m->now = on >= to || on < from ? from : on;
        }
        last = t;
        { std::lock_guard<std::mutex> hold(m->asked_lock); m->drew_at = beat; }
        ++m->beats_drawn;
        if (m->writes_on_beat > 1) ++m->beats_two;   // (more than one change came for this picture: one of them never seen)
        m->writes_on_beat = 0;
        m->s->by_clock = m->playing;
        std::vector<std::function<void()>> films;   // (what the moment asks of the films: done once the document is let go)
        std::map<const void*, Stage::Sound> sounds;
        {
            std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
            m->telling = true;
            m->s->later = &films;
            time_is(*m->s, m->now);
            m->s->later = nullptr;
            if (m->s->desk.w) {   // (the marks on the desk: where the picked thing and the one under the hand are now, as drawn)
                auto& k = m->s->desk;
                k.picked = m->pick > 0 && doc_frame(*m->d, m->pick, m->now, k.pick);
                k.hovered = m->hover > 0 && m->hover != m->pick && doc_frame(*m->d, m->hover, m->now, k.hover);
                k.knob = doc_word(*m->d, m->pick, "kind") != "camera";
            }
            picture_moment(m);   // (the window's picture of this moment)
            m->telling = false;
            sounds = heard_told(*m->s);   // (where each sound is to be: read of the document)
        }
        picture_tell(m);
        const auto films_from = std::chrono::steady_clock::now();
        const long films_late = video_late;
        for (const auto& now : films) now();
        const bool again = !m->playing && video_late != films_late;   // (a film without its picture for now yet, a seek under way: drawn again next beat, as a video sink shows its picture once it comes)

        { const double ms = 1e3 * std::chrono::duration<double>(std::chrono::steady_clock::now() - films_from).count(); m->films_ms += ms; m->films_most = std::max(m->films_most, ms); }   // (films put back or stepped on, while the shell reads the document)
        heard(sounds, m->heard, m->now, m->playing);   // (and their sound sought and played so)
        const auto told = std::chrono::steady_clock::now();
        {
            std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
            stage_kept_fresh(m->kept, *m->d, *m->s, m->revision.load(), RATE);
        }
        if (!m->playing || !stage_kept_show(m->kept, *m->s, m->live, m->now, RATE)) live_draw(*m->s, m->live);   // (played: a picture kept is put on the stage, not drawn)
        const auto drawn = std::chrono::steady_clock::now();
        lap(5);
        if (++m->judged % 30 == 0) {   // (how long the GPU took for its last pictures, against the time each may have)
            std::vector<int64_t> took;
            for (const auto& f : m->s->renderer->getFrameInfoHistory(30))
                if (f.gpuFrameComplete > 0 && f.backendBeginFrame > 0) took.push_back(f.gpuFrameComplete - f.backendBeginFrame);
            if (took.size() > 10) {
                std::nth_element(took.begin(), took.begin() + took.size() / 2, took.end());
                const double ms = (double)took[took.size() / 2] / 1e6;
                m->gpu_ms = ms, m->gpu_most = std::max(m->gpu_most, ms);
            }
            stage_scale(m);
        }
        m->spent[2] += std::chrono::duration<double>(told - t).count(), m->many[2]++;
        m->spent[3] += std::chrono::duration<double>(drawn - told).count(), m->many[3]++;
        m->changed = again;
        } catch (const std::exception& e) {
        std::fprintf(stderr, "motolii: the hand's round threw: %s\n", e.what());
        m->trouble = std::string("the hand's round threw: ") + e.what();
        m->telling = false;
        m->changed = false;
        m->playing = false;
    }
    }
    for (auto& [who, h] : m->heard) heard_off(h);
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
    if (m->beat) m->beat->stop();
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
static int live_is(Motolii* m) { return m->live || (m->live = live_open(*m->s)) ? 1 : 0; }
int motolii_live(Motolii* m) { return by_hand(m, [m] { return live_is(m); }); }
void* motolii_front(Motolii* m) { return m->live ? live_front(m->live) : nullptr; }
unsigned long long motolii_frames(Motolii* m) { return m->live ? m->live->frames.load() : 0; }
void motolii_finished(Motolii* m, void (*tell)(void*), void* with) { m->shown_with = with; m->shown = tell; }
const char* motolii_kept(Motolii* m) {
    static thread_local std::string runs;   // (the asker's own copy: the hand may tell it again meanwhile)
    std::lock_guard<std::mutex> hold(m->kept.runsLock);
    return (runs = m->kept.runs).c_str();
}
static int pick_is(Motolii* m, int thing, int hover) { m->pick = thing, m->hover = hover, m->changed = true; return 1; }
static int desk_is(Motolii* m, int w, int h, double x, double y, double scale, double point) {
    if (!m->live) return 0;
    Stage& s = *m->s;
    if (w < 16 || h < 16 || scale <= 0.0) return s.desk.w = 0, m->changed = true, 0;
    if (!m->live->desk || s.desk.w != (uint32_t)w || s.desk.h != (uint32_t)h) {   // (made again at its new size)
        Live* was = m->live->desk;
        m->live->desk = nullptr;
        live_close(s, was);
        m->live->desk = live_open(s, (uint32_t)w, (uint32_t)h);
    }
    s.desk.w = (uint32_t)w, s.desk.h = (uint32_t)h, s.desk.x = x, s.desk.y = y, s.desk.scale = scale, s.desk.point = point;
    m->changed = true;
    return m->live->desk ? 1 : 0;
}
int motolii_pick(Motolii* m, int thing, int hover) { return by_hand(m, [=] { return pick_is(m, thing, hover); }); }
int motolii_desk(Motolii* m, int w, int h, double x, double y, double scale, double point) {
    return by_hand(m, [=] { return desk_is(m, w, h, x, y, scale, point); });
}
void* motolii_desk_front(Motolii* m) { return m->live && m->live->desk ? live_front(m->live->desk) : nullptr; }
unsigned long long motolii_desk_frames(Motolii* m) { return m->live && m->live->desk ? m->live->desk->frames.load() : 0; }
int motolii_loop(Motolii* m, int on) {   // (on below 0: only asks: answered at once, not queued behind the hand's picture)
    if (on < 0) return (int)m->loop.load();
    m->loop = on != 0;   // (said at once: the hand reads it as it goes round)
    return on != 0;
}
// What the document's steps changed since this was last asked, a line each: "things" first when a thing came or went,
// then "thing name" for each value; asking forgets them. And the names of the values that change what cards a thing
// has (their conditions, its kind, its script, its shape or file, its letters split): a shell builds a thing's cards
// again when one of these changes, and only tells its numbers otherwise.
// Each thing's sound as heard now, a line each: its file, where the stage's moment puts it in its file, where it is,
// and whether it is playing (seconds; -1: not heard now). For checks: the speakers are not asked.
const char* motolii_heard(Motolii* m) {
    return by_look(m, [=] {
        std::string all;
        for (const auto& [who, h] : m->heard) {
            gint64 is = -1;
            if (h.bin) gst_element_query_position(h.bin, GST_FORMAT_TIME, &is);
            char line[600];
            std::snprintf(line, sizeof line, "%s %.3f %.3f %d\n", h.file.c_str(), h.wanted, is < 0 ? -1.0 : (double)is / GST_SECOND, h.on ? 1 : 0);
            all += line;
        }
        return (m->word = all).c_str();
    });
}
const char* motolii_shaping(Motolii* m) {
    return by_look(m, [=] {
        std::string all = "kind script shape file split name";   // (and its name: the inspector says it)
        for (const DocCard& c : doc_cards) if (c.when && !doc_among(all, c.when)) all += std::string(" ") + c.when;
        return (m->word = all).c_str();
    });
}
const char* motolii_spent(Motolii* m) {
    return by_look(m, [=] {
        char line[400];
        std::snprintf(line, sizeof line, "changes %ld %.0f ms, reads %ld %.0f ms, moments %ld %.0f ms, draws %ld %.0f ms (waiting for the GPU %.0f ms); beats drawn on %ld, the GPU not done at %ld, nothing new at %ld, two changes in one %ld, unseen %ld; films put %.0f ms (most %.1f); films' frames kept %ld, from disk %ld, drawn %ld", m->many[0], 1e3 * m->spent[0], m->many[1], 1e3 * m->spent[1], m->many[2], 1e3 * m->spent[2], m->many[3], 1e3 * m->spent[3], m->live ? 1e3 * m->live->waited : 0.0, m->beats_drawn, m->beats_gpu_busy, m->beats_still, m->beats_two, m->beats_unseen, m->films_ms, m->films_most, films_kept.load(), films_disk.load(), films_drawn.load());
        std::vector<std::pair<double, std::string>> doors;
        for (const auto& [name, cost] : m->by_door) {
            char one[160];
            std::snprintf(one, sizeof one, "%s %ld %.0f ms (slowest %.1f)", name.c_str(), cost.first, 1e3 * cost.second, 1e3 * m->worst[name]);
            doors.push_back({cost.second, one});
        }
        std::sort(doors.rbegin(), doors.rend());
        m->word = line;
        { char parts[300]; std::snprintf(parts, sizeof parts, "; the hand's round: files %.0f ms (most %.1f), changes %.0f (%.1f) of which done %.0f (%.1f), telling %.0f (%.1f), GPU asked %.0f (%.1f), ahead %.0f (%.1f), moment and draw %.0f (%.1f)", m->part_ms[0], m->part_most[0], m->part_ms[1], m->part_most[1], m->part_ms[6], m->part_most[6], m->part_ms[2], m->part_most[2], m->part_ms[3], m->part_most[3], m->part_ms[4], m->part_most[4], m->part_ms[5], m->part_most[5]); m->word += parts; for (double& x : m->part_most) x = 0; }
        char waits[260];
        std::snprintf(waits, sizeof waits, "; reads waited for the document %.0f ms, the longest %.1f ms, %ld over 2 ms; the GPU took %.1f ms a picture, at most %.1f, of %.0f it may", 1e3 * m->read_waited, 1e3 * m->read_waited_most, m->read_waits, m->gpu_ms, m->gpu_most, m->budget);
        m->word += waits;
        {   // (what the window's own thread asked, the most waited on first)
            std::lock_guard<std::mutex> hold(m->asked_lock);
            std::vector<std::pair<std::string, std::array<double, 4>>> all(m->window.begin(), m->window.end());
            std::sort(all.begin(), all.end(), [](const auto& a, const auto& b) { return a.second[1] + a.second[3] > b.second[1] + b.second[3]; });
            m->word += "; FROM THE WINDOW";
            for (const auto& [name, w] : all) { char one[200]; std::snprintf(one, sizeof one, "; %s %.0f calls, waited %.1f ms (most %.2f), took %.1f ms", name.c_str(), w[0], 1e3 * w[1], 1e3 * w[2], 1e3 * w[3]); m->word += one; }
        }
        { std::lock_guard<std::mutex> hold(m->asked_lock); for (const auto& [name, r] : m->rounds) if (r.second) { char one[160]; std::snprintf(one, sizeof one, "; %s waited for the hand up to %.1f ms, %ld over 4 ms", name.c_str(), 1e3 * r.first, r.second); m->word += one; } }
        for (size_t i = 0; i < doors.size() && i < 20; ++i) m->word += "; " + doors[i].second;   // (the dearest calls)
        return m->word.c_str();
    });
}
int motolii_play(Motolii* m, int playing) { return posted(m, "play", [m, playing] { m->playing = playing != 0; }); }
double motolii_now(Motolii* m) { return m->now; }

int motolii_add(Motolii* m, const char* kind, const char* file) { return by_hand(m, [=] { return changes(m, (int)(doc_add(doc(m), kind, file))); }); }
int motolii_remove(Motolii* m, int thing) { return by_hand(m, [=] { return changes(m, (int)(doc_remove(doc(m), thing))); }); }
int motolii_set(Motolii* m, int thing, const char* name, double number) { return by_hand(m, [=] { return changes(m, (int)(doc_set(doc(m), thing, name, number))); }); }
int motolii_points(Motolii* m, int thing, double* xy, int room) { return by_look(m, [=] { return doc_points(*m->d, thing, m->d->stage->now, xy, room); }); }
int motolii_handles(Motolii* m, int thing, double* xy, double* of, int room) { return by_look(m, [=] { return doc_points(*m->d, thing, m->d->stage->now, xy, room, true, of); }); }
int motolii_pen(Motolii* m, int thing, double x, double y, double hx, double hy) { return by_hand(m, [=] { return changes(m, doc_pen(doc(m), thing, x, y, hx, hy)); }); }
int motolii_say(Motolii* m, int thing, const char* name, const char* word) { return by_hand(m, [=] { return changes(m, (int)(doc_say(doc(m), thing, name, word))); }); }
int motolii_key(Motolii* m, int thing, const char* name, double at, double number) { return by_hand(m, [=] { return changes(m, (int)(doc_key(doc(m), thing, name, at, number))); }); }
// Many keys of one value in one call and one step (a world's motion baked into keys, a film's beats...): asked of the
// hand once, made one act, the stage made again once at its end.
int motolii_key_many(Motolii* m, int thing, const char* name, int n, const double* at, const double* number) {
    return by_hand(m, [=] {
        return changes(m, (int)doc_key_many(doc(m), thing, name, n, at, number));
    });
}
int motolii_unkey(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)(doc_unkey(doc(m), thing, name, at))); }); }
int motolii_keys(Motolii* m, int thing, const char* name) { return by_look(m, [=] { return (int)(doc_keys(doc(m), thing, name)); }); }
static int way_is(Motolii* m, int thing, const char* name, double at, const char* way) {   // (on the hand)
    Doc& d = doc(m);
    bool here = false;
    doc_rows(d, sqlite3_mprintf("select 1 from way where name = %Q", way), [&](sqlite3_stmt*) { here = true; });
    const auto own = doc_own_ways().find(way);
    if (here || own == doc_own_ways().end()) return changes(m, (int)(doc_way(d, thing, name, at, way)));
    doc_act(d);   // (the maker's own way: written into the document and put on the key, one step)
    if (!doc_define(d, way, own->second.c_str()) || !doc_way(d, thing, name, at, way)) d.act_ok = false;
    return changes(m, (int)doc_acted(d));
}
int motolii_way(Motolii* m, int thing, const char* name, double at, const char* way) { return by_hand(m, [=] { return way_is(m, thing, name, at, way); }); }
const char* motolii_effect_names(Motolii* m) { return by_look(m, [m] { return (m->listed = doc_effect_names()).c_str(); }); }
int motolii_effect(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)(doc_effect(doc(m), thing, name, at))); }); }
int motolii_define(Motolii* m, const char* way, const char* expression) { return by_hand(m, [=] { return changes(m, (int)(doc_define(doc(m), way, expression))); }); }
const char* motolii_ways(Motolii* m) { return by_look(m, [m] { return (m->listed = doc_ways(*m->d)).c_str(); }); }
int motolii_value(Motolii* m, int thing, const char* name, double at, double* is) {
    return by_look(m, [=] { return (int)(doc_value(doc(m), thing, name, at, is)); });
}
// A value as the stage shows it: what what drives it (a relation, a behaviour) makes of it, 2; its own, 1; none, 0.
int motolii_shows(Motolii* m, int thing, const char* name, double at, double* is) {
    return by_look(m, [=] {
        if (!doc_value(*m->d, thing, name, at, is)) return 0;
        if (!doc_driven_by(*m->d, thing, name)) return 1;
        *is = doc_literal(*m->d, thing, doc_lasting(name), at, *is);
        return 2;
    });
}
int motolii_things(Motolii* m, int* things, int room) {
    return by_look(m, [=] { const std::vector<int> all = doc_things(*m->d); for (int i = 0; i < room && i < (int)all.size(); ++i) things[i] = all[(size_t)i]; return (int)all.size(); });
}
const char* motolii_word(Motolii* m, int thing, const char* name) { return by_look(m, [=] { return (m->word = !strcmp(name, "name") ? doc_name(*m->d, thing) : doc_word(*m->d, thing, name)).c_str(); }); }
double motolii_lasts(Motolii* m, int thing) { return by_look(m, [=] { return doc_lasts(*m->d, thing); }); }
int motolii_key_moments(Motolii* m, int thing, const char* name, double* at, int room) {
    return by_look(m, [=] { const std::vector<double> all = doc_key_moments(*m->d, thing, name); for (int i = 0; i < room && i < (int)all.size(); ++i) at[i] = all[(size_t)i]; return (int)all.size(); });
}
// A thing's frame on the picture at the stage's moment, as the stage's own eye sees it: its middle, how wide and how
// high (points of the picture, from the top left), and how far it is turned (degrees, clockwise as seen).
int motolii_frame(Motolii* m, int thing, double* frame) {
    return by_look(m, [=] { return (int)(doc_knows(doc(m), thing) && doc_frame(*m->d, thing, m->now, frame)); });
}
// What of a thing is seen at the stage's moment: the upright box of its outline on the picture (left, top, right, bottom).
int motolii_seen(Motolii* m, int thing, double* box) { return by_look(m, [=] { return (int)(doc_knows(doc(m), thing) && doc_seen_box(*m->d, thing, m->now, box)); }); }
int motolii_path(Motolii* m, int thing, int copy, double from, double to, int n, double* path) {
    return by_look(m, [=] {
        if (!doc_knows(doc(m), thing) || n < 1) return 0;
        for (int i = 0; i < n; ++i) {   // (the stage's own placing, at moments of the preview's own: the stage's moment stays)
            const double at = from + (to - from) * i / n;
            const math::mat4 p = doc_place(*m->d, thing, at, 0, copy, copy > 0 || doc_staggered(*m->d, thing, at));   // (as a copy, or as itself, as the stage places it)
            path[4 * i] = p[3][0], path[4 * i + 1] = p[3][1];
            path[4 * i + 2] = std::atan2(p[0][1], p[0][0]) * 360.0 / (double)TAU, path[4 * i + 3] = length(p[0].xyz);
        }
        return n;
    });
}
// The same, in the round: path[4i..4i+3] = where it is (x, y, z, units of the stage) and how large; for a preview that
// draws it as seen from any side.
int motolii_path3(Motolii* m, int thing, int copy, double from, double to, int n, double* path) {
    return by_look(m, [=] {
        if (!doc_knows(doc(m), thing) || n < 1) return 0;
        for (int i = 0; i < n; ++i) {
            const double at = from + (to - from) * i / n;
            const math::mat4 p = doc_place(*m->d, thing, at, 0, copy, copy > 0 || doc_staggered(*m->d, thing, at));
            path[4 * i] = p[3][0], path[4 * i + 1] = p[3][1], path[4 * i + 2] = p[3][2], path[4 * i + 3] = length(p[0].xyz);
        }
        return n;
    });
}
int motolii_keying(Motolii* m, int on) {   // (on below 0: only says whether every change is keyed: a read, answered at once, not queued behind the hand's picture)
    if (on < 0) return (int)m->keying.load();
    m->keying = on != 0;   // (said at once; the document told in the hand's turn, without waiting)
    return posted(m, "keying", [m, on] { m->d->keying = on != 0, m->d->keying_from = m->now; m->keying = m->d->keying; }), on != 0;
}
int motolii_hold(Motolii* m, int thing) { return by_hand(m, [=] { return (int)doc_hold(doc(m), thing); }); }
int motolii_carry(Motolii* m, const char* by, double a, double b) { return posted(m, std::string("carry ") + by, [m, by = std::string(by), a, b] { doc_carry(doc(m), by.c_str(), a, b); }); }   // (refused: said when it is let go)
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
int motolii_work_of(Motolii* m, double* from, double* to) { return by_look(m, [=] { return doc_work(*m->d, from, to), 1; }); }
// A film's sound: how loud it is, 100 times a second of its own (read once a file, as fast as it decodes), and the
// moments of the document it strikes (grows `rise` dB louder at once), where its film shows them.
static const std::vector<float>* loudness_of(const std::string& file, bool read_it = true) {   // (not read_it: only if read already)
    static std::map<std::string, std::vector<float>> read;
    static std::mutex one;
    std::lock_guard<std::mutex> hold(one);
    auto found = read.find(file);
    if (found == read.end()) {
        if (!read_it) return nullptr;
        found = read.emplace(file, sound_loudness(file.c_str(), 100.0)).first;
    }
    return &found->second;
}
int motolii_loudness(Motolii* m, int thing, float* db, int room) {
    const std::string file = by_look(m, [=] { return doc_knows(doc(m), thing) && doc_word(*m->d, thing, "kind") == "film" ? doc_word(*m->d, thing, "file") : std::string(); });
    if (file.empty()) return 0;
    const std::vector<float>& all = *loudness_of(file);
    for (int i = 0; i < room && i < (int)all.size(); ++i) db[i] = all[(size_t)i];
    return (int)all.size();
}
// How loud a film's sound is at n moments of the document from `from` to before `to`, where its film shows them (dB;
// -100 where it is silent or not shown). -1: its sound is not read yet (motolii_loudness reads it), 0: no sound.
int motolii_loudness_over(Motolii* m, int thing, double from, double to, int n, float* db) {
    const std::string file = by_look(m, [=] { return doc_knows(doc(m), thing) && doc_word(*m->d, thing, "kind") == "film" ? doc_word(*m->d, thing, "file") : std::string(); });
    if (file.empty()) return 0;
    const std::vector<float>* all = loudness_of(file, false);
    if (!all) return -1;
    if (all->empty()) return 0;
    return by_look(m, [&] {   // (the document's moment to the film's own: from its start, from where it begins, at its speed)
        Doc& d = *m->d;
        const double start = doc_number(d, thing, "start", 0, 0), begin = doc_number(d, thing, "begin", 0, 0), speed = doc_number(d, thing, "speed", 0, 1), lasts = doc_number(d, thing, "duration", 0, -1);
        for (int i = 0; i < n; ++i) {
            const double s = from + (to - from) * i / n, own = begin + (s - start) * speed;
            const long k = std::lround(own * 100.0);
            db[i] = s < start || (lasts > 0.0 && s >= start + lasts) || k < 0 || k >= (long)all->size() ? -100.f : (*all)[(size_t)k];
        }
        return n;
    });
}
int motolii_strikes(Motolii* m, int thing, double rise, int own, double* at, int room) {
    const std::string file = by_look(m, [=] { return doc_knows(doc(m), thing) && doc_word(*m->d, thing, "kind") == "film" ? doc_word(*m->d, thing, "file") : std::string(); });
    if (file.empty()) return 0;
    const std::vector<double> struck = sound_strikes(*loudness_of(file), 100.0, rise);
    if (own) {   // (in the film's own seconds, as its loudness is counted)
        for (int i = 0; i < room && i < (int)struck.size(); ++i) at[i] = struck[(size_t)i];
        return (int)struck.size();
    }
    return by_look(m, [&] {   // (the film's own seconds to the document's: from its start, from where it begins, at its speed, while it lasts)
        Doc& d = *m->d;
        const double start = doc_number(d, thing, "start", 0, 0), begin = doc_number(d, thing, "begin", 0, 0), speed = doc_number(d, thing, "speed", 0, 1), lasts = doc_number(d, thing, "duration", 0, -1);
        int n = 0;
        for (const double t : struck) {
            const double when = speed == 0.0 ? -1.0 : start + (t - begin) / speed;
            if (when < start || (lasts > 0.0 && when >= start + lasts)) continue;
            if (n < room) at[n] = when;
            ++n;
        }
        return n;
    });
}
int motolii_mark(Motolii* m, double at, const char* word) { return by_hand(m, [=] { return changes(m, (int)doc_mark(doc(m), at, word)); }); }
int motolii_guide(Motolii* m, const char* axis, double at, int on) { return by_hand(m, [=] { return changes(m, (int)doc_guide(doc(m), axis, at, on != 0)); }); }
const char* motolii_guides(Motolii* m) { return by_look(m, [=] { return (m->word = doc_guides(*m->d)).c_str(); }); }
const char* motolii_marks(Motolii* m) { return by_look(m, [=] { return (m->word = doc_marks(*m->d)).c_str(); }); }
int motolii_fits(Motolii* m) {
    return by_look(m, [=] {   // (a read: the document's size against the stage's)
        int wide = m->s->w, high = m->s->h;
        doc_rows(*m->d, sqlite3_mprintf("select name, value from fact where thing = 0 and name in ('wide', 'high')"),
                 [&](sqlite3_stmt* row) { (strcmp((const char*)sqlite3_column_text(row, 0), "wide") ? high : wide) = sqlite3_column_int(row, 1); });
        return (int)(wide == m->s->w && high == m->s->h);
    });
}
int motolii_size_of(Motolii* m, int* wide, int* high) { return by_look(m, [=] { return *wide = m->s->w, *high = m->s->h, 1; }); }
int motolii_raise(Motolii* m, int thing, int places) { return by_hand(m, [=] { return changes(m, (int)doc_raise(doc(m), thing, places)); }); }
int motolii_stay(Motolii* m, int thing, const char* name, double at) { return by_hand(m, [=] { return changes(m, (int)doc_stay(doc(m), thing, name, at)); }); }
int motolii_keyed(Motolii* m, int thing, const char* name, double at) {
    return by_look(m, [=] { return doc_keyed(*m->d, thing, name, at); });
}
const char* motolii_words(Motolii* m, int thing, const char* name) { return by_look(m, [=] { return (m->word = doc_words(*m->d, thing, name)).c_str(); }); }
// Every moment a key of any of a thing's values that take keys stands at, each once and in time: a timeline's bar
// marks in one call (asked a value at a time, a document of fifteen recipes was hundreds of calls each reading).
int motolii_thing_key_moments(Motolii* m, int thing, double* at, int room) {
    return by_look(m, [=] {   // (one question to SQLite: the moments of the values of it that have keys, as DOC_KEYED says one has)
        int i = 0;
        doc_rows(*m->d, sqlite3_mprintf("select distinct at from fact where thing = %d and name in (select name from fact where thing = %d group by name having count(*) > 1 or max(at) <> 0 or max(way) is not null) order by at", thing, thing), [&](sqlite3_stmt* row) {
            if (i < room) at[i] = sqlite3_column_double(row, 0);
            ++i;
        });
        return i;
    });
}
const char* motolii_moving(Motolii* m, int thing) { return by_look(m, [=] { return (m->word = doc_moving(*m->d, thing)).c_str(); }); }
const char* motolii_values(Motolii* m, int thing) { return by_look(m, [=] { return (m->word = doc_has(*m->d, thing)).c_str(); }); }
int motolii_range(Motolii* m, int thing, const char* name, double* low, double* high) { return by_look(m, [=] { return (int)doc_range(*m->d, thing, name, low, high); }); }
const char* motolii_recipes(Motolii* m) { return by_look(m, [=] { return (m->word = doc_recipe_names()).c_str(); }); }
const char* motolii_effects(Motolii* m) { return by_look(m, [=] { return (m->word = doc_folder_names("effects", ".wgsl .fs .mjs")).c_str(); }); }
const char* motolii_animators(Motolii* m) { return by_look(m, [=] { return (m->word = doc_folder_names("animators", ".js")).c_str(); }); }
int motolii_make(Motolii* m, const char* recipe) { return by_hand(m, [=] { return changes(m, doc_make(doc(m), recipe)); }); }
const char* motolii_shown(Motolii* m, const char* name) {
    return by_look(m, [=] {
        const DocShown* s = doc_shown_of(name);
        char line[512] = "";
        if (s) std::snprintf(line, sizeof line, "%s\t%s\t%s\t%g\t%g\t%s", s->look, s->unit, s->label, s->low, s->high, s->fold ? s->fold : "");
        return (m->word = line).c_str();
    });
}
const char* motolii_cards(Motolii* m, int thing) { return by_look(m, [=] { return (m->word = doc_cards_of(*m->d, thing)).c_str(); }); }
const char* motolii_fx_titles(Motolii* m, int thing) { return by_look(m, [=] { return (m->word = doc_fx_titles(*m->d, thing)).c_str(); }); }
const char* motolii_kind_of(const char* file) { return doc_kind_of(file); }
int motolii_sets(const char* font, const char* words) { return type_sets(font, words) ? 1 : 0; }
// The names of the ways a stretch may go (the nine kinds, then the ways defined in this document), with spaces between;
// and how one of the nine goes: how far it is, from 0 at its beginning to 1 at its end, at `u` of the way through.
// A kind's numbers as a hand takes them: each one's name, how low and how high ("seed 1 999 grain 0 0.5 ..."); none: "".
const char* motolii_way_numbers(Motolii*, const char* kind) {   // (the kinds are the maker's, not the document's: nothing waited for)
    static thread_local std::string said;
    said.clear();
    for (const auto& k : interp_kinds) {
        if (strcasecmp(k.name, kind)) continue;
        std::istringstream names(k.names);
        std::string name;
        char one[96];
        for (int j = 0; j < k.numbers && names >> name; ++j) std::snprintf(one, sizeof one, "%s%s %.9g %.9g", said.empty() ? "" : " ", name.c_str(), k.lo[j], k.hi[j]), said += one;
    }
    return said.c_str();
}
const char* motolii_way_names(Motolii* m) {
    return by_look(m, [=] {
        m->word.clear();
        for (const auto& kind : interp_kinds) m->word += std::string(m->word.empty() ? "" : " ") + kind.name;
        std::set<std::string> here;
        doc_rows(*m->d, sqlite3_mprintf("select name from way order by name"), [&](sqlite3_stmt* row) { here.insert((const char*)sqlite3_column_text(row, 0)); m->word += std::string(" ") + (const char*)sqlite3_column_text(row, 0); });
        for (const auto& [name, goes] : doc_own_ways()) if (!here.count(name)) m->word += " " + name;   // (then the maker's own, not yet in this document)
        return m->word.c_str();
    });
}
double motolii_way_at(Motolii* m, const char* way, double u) {
    Interp kind;
    if (interp_said(way, &kind) == 0) return interp_ease(kind, u);
    return by_look(m, [=] {
        bool here = false;
        doc_rows(*m->d, sqlite3_mprintf("select 1 from way where name = %Q", way), [&](sqlite3_stmt*) { here = true; });
        const auto own = doc_own_ways().find(way);
        if (!here && own != doc_own_ways().end()) {   // (the maker's own, not yet in this document: as its file says it)
            const double p[4] = {1, 1, 1, 1};
            sqlite3_stmt* goes = doc_goes(*m->d, own->second);
            return goes ? doc_went(goes, u, u, 1.0, 0.0, 1.0, p) : u;
        }
        return doc_went_by(*m->d, way, u, u, 1.0, 0.0, 1.0);   // (one defined in the document: as between a key of 0 and a key of 1, a second apart)
    });
}
// A film's first picture as a small .png: GStreamer's own line for it (decode, scale, into a .png), held at its first
// picture, which is taken and written. It asks nothing of a stage, so a shell may call it from another thread.
// (fraction: where in the film, 0 to 1; a picture there, not the first: a hand moving over a tile looks along the film)
static int thumbnail_at(const char* film, const char* png, int wide, double fraction) {
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
    if (first && fraction > 0.0) {
        gint64 length = 0;
        if (gst_element_query_duration(made, GST_FORMAT_TIME, &length) && length > 0) {
            gst_sample_unref(first);
            gst_element_seek_simple(made, GST_FORMAT_TIME, (GstSeekFlags)(GST_SEEK_FLAG_FLUSH | GST_SEEK_FLAG_ACCURATE), (gint64)(std::min(fraction, 0.98) * (double)length));
            first = gst_app_sink_try_pull_preroll(GST_APP_SINK(out), 4 * GST_SECOND);
        }
    }
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
int motolii_thumbnail(const char* film, const char* png, int wide) { return thumbnail_at(film, png, wide, 0.0); }
int motolii_thumbnail_at(const char* film, const char* png, int wide, double fraction) { return thumbnail_at(film, png, wide, fraction); }
// A font file's kinds, as CoreText reads them (its own: the file's family class and traits): serif, sans, script,
// mono, bold, light, italic, symbols, ornaments, space separated; "" for none.
const char* motolii_font_tags(const char* path) {
    static thread_local std::string out;
    out.clear();
    CFURLRef url = CFURLCreateFromFileSystemRepresentation(nullptr, (const UInt8*)path, (CFIndex)std::strlen(path), false);
    CFArrayRef all = url ? CTFontManagerCreateFontDescriptorsFromURL(url) : nullptr;
    if (all && CFArrayGetCount(all) > 0) {
        CTFontDescriptorRef one = (CTFontDescriptorRef)CFArrayGetValueAtIndex(all, 0);
        CFDictionaryRef traits = (CFDictionaryRef)CTFontDescriptorCopyAttribute(one, kCTFontTraitsAttribute);
        if (traits) {
            SInt32 symbolic = 0;
            float weight = 0.f;
            if (CFNumberRef n = (CFNumberRef)CFDictionaryGetValue(traits, kCTFontSymbolicTrait)) CFNumberGetValue(n, kCFNumberSInt32Type, &symbolic);
            if (CFNumberRef n = (CFNumberRef)CFDictionaryGetValue(traits, kCTFontWeightTrait)) CFNumberGetValue(n, kCFNumberFloatType, &weight);
            const uint32_t kind = ((uint32_t)symbolic & kCTFontClassMaskTrait) >> kCTFontClassMaskShift;
            if (kind >= 1 && kind <= 7) out += "serif ";
            else if (kind == 8) out += "sans ";
            else if (kind == 9) out += "ornaments ";
            else if (kind == 10) out += "script ";
            else if (kind == 12) out += "symbols ";
            if ((uint32_t)symbolic & kCTFontMonoSpaceTrait) out += "mono ";
            if ((uint32_t)symbolic & kCTFontBoldTrait || weight > 0.3f) out += "bold ";
            if (weight < -0.3f) out += "light ";
            if ((uint32_t)symbolic & kCTFontItalicTrait) out += "italic ";
            CFRelease(traits);
        }
    }
    if (all) CFRelease(all);
    if (url) CFRelease(url);
    if (!out.empty()) out.pop_back();
    return out.c_str();
}
int motolii_together(Motolii* m, int on) { return by_hand(m, [=] { return on ? (doc_act(*m->d), 1) : (m->d->act ? (int)doc_acted(*m->d) : 0); }); }
// Every stretch of a thing's keyed values, a line each: from, to, its way ("from\tto\tway\n"): a timeline draws how each
// goes between its two keys (the stretch is the inside's own: the window only draws what it says).
const char* motolii_thing_stretches(Motolii* m, int thing) {
    return by_look(m, [=] {
        std::string all;
        doc_rows(*m->d, sqlite3_mprintf("select at, next, way from (select at, lead(at) over (partition by name order by at) as next, coalesce(way, 'Linear') as way from fact where thing = %d and name in (select name from fact where thing = %d group by name having count(*) > 1 or max(at) <> 0 or max(way) is not null)) where next is not null order by at", thing, thing), [&](sqlite3_stmt* row) {
            std::string way = (const char*)sqlite3_column_text(row, 2);
            Interp kind;
            if (interp_said(way, &kind) == 0) way = interp_words(kind);
            char line[64];
            std::snprintf(line, sizeof line, "%.17g\t%.17g\t", sqlite3_column_double(row, 0), sqlite3_column_double(row, 1));
            all += line + way + "\n";
        });
        return (m->word = all).c_str();
    });
}
const char* motolii_way_of(Motolii* m, int thing, const char* name, double at) { return by_look(m, [=] { return (m->word = doc_way_of(*m->d, thing, name, at)).c_str(); }); }
// How many points of the picture one unit of the stage is, where things stand when their z is 0.
static double point_of(Motolii* m) {   // (points of the picture to one unit of the stage, at its middle, as the stage's own eye sees it:
    // 6 units off with 30 degrees up and down (motolii_open): the document's scale, whatever camera the stage looks through now)
    return m->s->h / (2.0 * 6.0 * std::tan(15.0 * TAU / 360.0));
}
double motolii_point(Motolii* m) { return by_look(m, [=] { return point_of(m); }); }
// How a hand reads a thing's value: as the value times `times`, and `from` more ("times from"). Places are counted in
// points of the picture from its top left, as After Effects counts them; scale and opacity in hundredths; the turn about
// z clockwise (a camera's turns as they are, its pitch looking down); the anchor in hundredths of the box from its top
// left (front -50 to back 50); a camera's distance in points of the picture (the stage's own eye 6 units off); any other
// value by its look (stage: points, down: points down the picture, share: hundredths), else as it is.
const char* motolii_reading(Motolii* m, int thing, const char* name) {
    return by_look(m, [=] {
        const double p = point_of(m), w = m->s->w, h = m->s->h;
        const bool camera = doc_word(*m->d, thing, "kind") == "camera";
        const std::string n = name;
        double times = 1, from = 0;
        if (n == "x") times = p, from = w / 2;
        else if (n == "y") times = -p, from = h / 2;
        else if (n == "z") times = p;
        else if (n == "scale_x" || n == "scale_y" || n == "scale_z" || n == "opacity" || n == "zoom") times = 100;
        else if (n == "rotate") times = camera ? 1 : -1;
        else if (n == "rotate_x") times = camera ? -1 : 1;
        else if (n == "anchor_x") times = 50, from = 50;
        else if (n == "anchor_y") times = -50, from = 50;
        else if (n == "anchor_z") times = -50;
        else if (n == "distance") times = 6 * p;
        else if (n == "size") times = p;
        else if (const DocShown* s = doc_shown_of(name)) {
            const std::string look = s->look;
            times = look == "stage" ? p : look == "down" ? -p : look == "share" ? 100 : 1;
        }
        char line[64];
        std::snprintf(line, sizeof line, "%.17g %.17g", times, from);
        return (m->word = line).c_str();
    });
}
// A change asked without waiting (a shell's hand that must not stand still while the hand works it): queued in order
// behind every other, a ticket said at once; motolii_answer says whether it is done (-1 not yet, 0 refused: then
// motolii_answer_why says why, 1 done; for add and make, the new thing). The calls: say, set, let_go (number: keep),
// undo, redo, together (number: begin 1, end 0: the end's answer is the step's), add (name: kind, word: file), make
// (word: recipe), duplicate (the answer is the new thing), remove, mark (number: at, word), guide (name: axis, number: at, word "off" takes it away), work
// (number: from, word: to), raise (number: places), sequence (number: overlap), key_here, unkey, stay (number: at), way
// (number: at, word: way), effect (number: at), hold (a hand takes hold of it). A thing -1: the one the last add or make made.
int motolii_later(Motolii* m, const char* call, int thing, const char* name, const char* word, double number) {
    const std::string c = call ? call : "", n = name ? name : "", w = word ? word : "";
    int ticket = 0;
    {
        std::lock_guard<std::mutex> hold(m->asked_lock);
        ticket = ++m->tickets;
        m->asked.push_back([m, c, n, w, thing = thing, number, ticket]() mutable {
            if (thing == -1) thing = m->made;   // (the thing the last add or make asked later made)
            const long rows = sqlite3_total_changes(m->d->file);
            int ok = 0;
            try {   // (a call that throws is refused with its words, not the end of the hand: the window waits on its ticket)
            if (c == "say") ok = changes(m, (int)doc_say(doc(m), thing, n.c_str(), w.c_str()));
            else if (c == "set") ok = changes(m, (int)doc_set(doc(m), thing, n.c_str(), w == "made" ? m->made : number));   // (word "made": the number is the thing the last add made, as a value that names a thing: a behaviour connected, a leader)
            else if (c == "let_go") ok = changes(m, (int)doc_let_go(doc(m), number != 0.0));
            else if (c == "undo") ok = (int)doc_undo(doc(m));
            else if (c == "redo") ok = (int)doc_redo(doc(m));
            else if (c == "together") ok = number != 0.0 ? (doc_act(*m->d), 1) : (m->d->act ? (int)doc_acted(*m->d) : 0);
            else if (c == "add") ok = m->made = changes(m, doc_add(doc(m), n.c_str(), w.c_str()));   // (name: the kind, word: the file; the answer is the new thing)
            else if (c == "make") ok = m->made = changes(m, doc_make(doc(m), w.c_str()));
            else if (c == "duplicate") ok = m->made = changes(m, doc_duplicate(doc(m), thing));   // (the answer is the new thing)
            else if (c == "remove") ok = changes(m, (int)doc_remove(doc(m), thing));
            else if (c == "relate") ok = changes(m, (int)doc_relate(doc(m), thing, w.c_str(), m->now));
            else if (c == "fx_file") ok = changes(m, (int)doc_fx_file(doc(m), thing, w.c_str()));   // (an effect file put at the end of the thing's effects, or taken out)
            else if (c == "fx_move") ok = changes(m, (int)doc_fx_move(doc(m), thing, w.c_str(), (int)std::lround(number)));   // (one of its effects moved up or down its list)   // (the things in word put in the relation `thing` as they show now)
            else if (c == "lead") ok = changes(m, (int)doc_lead(doc(m), thing, (int)number, m->now));   // (the relation `thing` led by the thing `number`, nothing moving)
            else if (c == "free") ok = changes(m, (int)doc_free(doc(m), thing, (int)number, m->now, false));   // (a thing out of the relation `number`: back to what is under it, as the relation deleted does; only bake keeps where it shows)
            else if (c == "release") ok = changes(m, (int)doc_release(doc(m), thing, m->now));   // (the relation `thing` let go, each of its things where it shows now)
            else if (c == "mark") ok = changes(m, (int)doc_mark(doc(m), number, w.c_str()));
            else if (c == "guide") ok = changes(m, (int)doc_guide(doc(m), n.c_str(), number, w != "off"));
            else if (c == "work") ok = changes(m, (int)doc_work_is(doc(m), number, std::atof(w.c_str())));
            else if (c == "size") {   // (word: "wide high", the stage's size the document says: opened at it again by the window)
                int wide = 0, high = 0;
                std::sscanf(w.c_str(), "%d %d", &wide, &high);
                char* say = doc_size_words(wide, high);
                ok = say ? changes(m, (int)doc_change(*m->d, say)) : (m->d->trouble = "a stage is from 16 to 8192 points each way, an even number of them", 0);
            }
            else if (c == "raise") ok = changes(m, (int)doc_raise(doc(m), thing, (int)number));
            else if (c == "sequence") ok = changes(m, (int)doc_sequence(doc(m), number, RATE));
            else if (c == "key_here") ok = changes(m, (int)doc_key_here(doc(m), thing, n.c_str(), number));
            else if (c == "key") ok = changes(m, (int)doc_key(doc(m), thing, n.c_str(), std::atof(w.c_str()), number));   // (word: the moment; number: what the key holds)
            else if (c == "unkey") ok = changes(m, (int)doc_unkey(doc(m), thing, n.c_str(), number));
            else if (c == "stay") ok = changes(m, (int)doc_stay(doc(m), thing, n.c_str(), number));
            else if (c == "way") ok = way_is(m, thing, n.c_str(), number, w.c_str());
            else if (c == "effect") ok = changes(m, (int)doc_effect(doc(m), thing, n.c_str(), number));
            else if (c == "pen") {   // (word: where the hand is and its handle, "x y hx hy", points of the picture; the answer is the path shape)
                double x = 0.0, y = 0.0, hx = 0.0, hy = 0.0;
                std::sscanf(w.c_str(), "%lf %lf %lf %lf", &x, &y, &hx, &hy);
                ok = m->made = changes(m, doc_pen(doc(m), thing, x, y, hx, hy));
            }
            else if (c == "live") ok = live_is(m);   // (the stage's live picture opened: answer 1 once it is)
            else if (c == "pick") ok = pick_is(m, thing, (int)number);   // (the thing picked in the window, number: the one under its hand)
            else if (c == "watch") {   // (word: the things the window shows the moving values of, spaced; their values in the moment's picture from the next)
                m->watched.clear();
                std::istringstream in(w);
                for (int t; in >> t;) m->watched.insert(t);
                picture_moment(m);   // (their values at once, not with the next picture)
                ok = 1, m->changed = true;
            }
            else if (c == "desk") {   // (word: "w h x y scale point", as motolii_desk; answer 1 once the desk's picture exists)
                int dw = 0, dh = 0;
                double x = 0.0, y = 0.0, sc = 0.0, pt = 0.0;
                std::sscanf(w.c_str(), "%d %d %lf %lf %lf %lf", &dw, &dh, &x, &y, &sc, &pt);
                ok = desk_is(m, dw, dh, x, y, sc, pt);
            }
            else if (c == "hold") ok = (int)doc_hold(doc(m), thing);   // (a hand takes hold: what it carries next is this thing's)
            else m->trouble = "a change asked later is one of: say, set, hold, let_go, undo, redo, together, add, make, pen, duplicate, remove, mark, guide, work, raise, sequence, key_here, unkey, stay, way, effect, fx_file, fx_move, size, watch, pick, desk, live, relate, lead, free, release";
            } catch (const std::exception& e) {
                ok = 0;
                std::fprintf(stderr, "motolii: %s threw: %s\n", c.c_str(), e.what());
                m->trouble = std::string(c) + " threw: " + e.what();
            }
            if (sqlite3_total_changes(m->d->file) != rows) m->revision.fetch_add(1, std::memory_order_release), picture_update(m), picture_moment(m);   // (the pictures new before the answer: the window reads what is so now)
            m->changed = true;
            std::lock_guard<std::mutex> hold(m->asked_lock);
            m->answers[ticket] = {ok, ok ? std::string() : std::string(motolii_trouble(m))};
        });
    }
    m->asked_now.notify_one();
    return ticket;
}
int motolii_answer(Motolii* m, int ticket) {
    std::lock_guard<std::mutex> hold(m->asked_lock);
    const auto found = m->answers.find(ticket);
    if (found == m->answers.end()) return -1;
    const int ok = found->second.first;
    m->answered_why = found->second.second;
    m->answers.erase(found);
    return ok;
}
const char* motolii_answer_why(Motolii* m) { return m->answered_why.c_str(); }
int motolii_undo(Motolii* m) { return by_hand(m, [=] { return (int)(doc_undo(doc(m))); }); }
int motolii_redo(Motolii* m) { return by_hand(m, [=] { return (int)(doc_redo(doc(m))); }); }

double motolii_seconds(Motolii* m) { return by_look(m, [m] { return doc_seconds(*m->d); }); }
int motolii_at(Motolii* m, double seconds) {
    // (sent, not waited for: the window reads what was drawn, at the moment it was drawn (the moment's picture, picture.h),
    //  so it need not wait for the hand to tell this one; a later one still waiting replaces it. Past the document's
    //  ends it is refused there, and said in the trouble)
    return posted(m, "at", [m, seconds] {
        if (!(seconds >= 0.0 && seconds <= doc_seconds(*m->d))) { char runs[80]; std::snprintf(runs, sizeof runs, "the document runs from 0 to %.2f seconds", doc_seconds(*m->d)); m->d->trouble = runs; return; }
        m->now = seconds;   // (told to the stage in the hand's round, the films put there after the document is let go: not here, under its lock)
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
int motolii_rate(Motolii*) { return RATE; }
// The window's thread watched (Chromium's ThreadRestrictions, in the manner of its DisallowBlocking): while on, any
// change it waits for and any read that waits on the document over 2 ms is kept, named and timed; motolii_broken says
// them all ("" none), one to a line, the last 200. Watched from the opening; turned on again, what was kept is forgotten.
void motolii_watch(Motolii* m, int on) { std::lock_guard<std::mutex> h(m->asked_lock); if (on) m->broke.clear(), m->broken = 0; m->watching = on != 0; }
long motolii_broken_count(Motolii* m) { return m->broken.load(); }   // (no lock: a window may ask it every picture)
const char* motolii_broken(Motolii* m) {
    std::lock_guard<std::mutex> h(m->asked_lock);
    m->listed.clear();
    for (const std::string& one : m->broke) m->listed += (m->listed.empty() ? "" : "\n") + one;
    return m->listed.c_str();
}
int motolii_sequence(Motolii* m, double overlap) { return by_hand(m, [=] { return changes(m, (int)doc_sequence(doc(m), overlap, RATE)); }); }
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
        m->d->lock.unlock();   // (held by the hand that asked this: let go while each picture is drawn, as when it is live)
        const int ok = told(m, film(*m->s, mp4, pictures, [m, pictures, from](float done) {
            std::lock_guard<std::recursive_timed_mutex> holding(m->d->lock);
            std::deque<std::function<void()>> todo;
            { std::lock_guard<std::mutex> hold(m->asked_lock); todo.swap(m->asked); }
            for (auto& does : todo) does();
            time_is(*m->s, from + (double)std::lround(done * (float)pictures) / RATE);   // (the picture's moment: from the work area's start)
        }), "the file could not be written");
        m->d->lock.lock();
        m->d->writing = false;
        m->d->written_from = 0.0;
        time_is(*m->s, m->now);   // (back to the moment it was at)
        return ok;
    });
}
const char* motolii_list(Motolii* m) {
    return by_look(m, [m] {
    m->listed.clear();
    doc_rows(*m->d, sqlite3_mprintf("select thing || ' ' || name || ' at ' || cast(at as real) || ': ' || value || coalesce(', then ' || way, '') from fact order by thing, name, at"), [m](sqlite3_stmt* row) { m->listed += std::string("  ") + (const char*)sqlite3_column_text(row, 0) + "\n"; });
    return m->listed.c_str();
    });
}
void motolii_timed(Motolii* m, int pictures) { by_hand(m, [m, pictures] { clock(*m->s, pictures, [](float) {}); return 0; }); }

#include "picture.h"
// The window's pictures of the document (skin/BRIDGE.md), read without the document: 1 the document's (its own part,
// and each thing's part newer than `since`), 2 the moment's. A copy as JSON, freed by the asker; *version: as of which.
char* motolii_picture(Motolii* m, int which, long since, long* version) {
    std::lock_guard<std::mutex> hold(m->pictureLock);
    if (which == 2) return *version = m->momentVersion, strdup(m->momentPicture.c_str());
    std::string all = "{\"document\":" + (m->pictureAll.empty() ? std::string("null") : m->pictureAll) + ",\"thing\":{";
    bool first = true;
    for (const auto& [thing, part] : m->pictured)
        if (part.first > since) all += (first ? "\"" : ",\"") + std::to_string(thing) + "\":" + part.second, first = false;
    *version = m->pictureVersion;
    return strdup((all + "}}").c_str());
}
void motolii_tell(Motolii* m, void (*told)(int which)) { m->told = told; }   // (the window's one listener: which pictures are new)
