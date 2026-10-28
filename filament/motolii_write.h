// motolii_write.h: part of motolii.mm. Read only through motolii.mm.
int motolii_write(Motolii* m, const char* mp4) {
    return by_hand(m, [m, mp4] {
        doc(m);
        if (!told(m, ends(mp4, ".mp4") || ends(mp4, ".mov") || ends(mp4, ".gif") || ends(mp4, ".png"), "a film is written as an .mp4, .mov (HEVC), .gif or .png (a picture each)") || !told(m, placed(mp4), "there is no such folder") || !told(m, !m->d->writing, "it is being written already")) return 0;
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
        }, FilmWay{(int)doc_number(*m->d, 0, "render_height", 0, 0), (int)doc_number(*m->d, 0, "render_rate", 0, 0), doc_number(*m->d, 0, "render_mbps", 0, 0)}), "the file could not be written");
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
