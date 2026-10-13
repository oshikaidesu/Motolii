// motolii.mm: motolii.h's calls, each handed on to doc.h (the document) and kit.h (the stage). Nothing is worked out here.
#include "kit.h"
#include "doc.h"
#include "motolii.h"

struct Motolii { Stage s; Doc* d = nullptr; std::string trouble, listed; };

Motolii* motolii_open(const char* document, int wide, int high) {
    Motolii* m = new Motolii{open_stage(wide, high)};
    m->s.camera->lookAt({0.f, 0.f, 6.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    m->s.camera->setProjection(30.0, double(wide) / double(high), 0.05, 60.0, Camera::Fov::VERTICAL);
    static LinearToneMapper as_it_is;   // (a film, a photo, typed words keep the colours they came with)
    m->s.view->setColorGrading(ColorGrading::Builder().toneMapper(&as_it_is).build(*m->s.engine));
    if (!room(m->s, "out/small_studio/small_studio_ibl.ktx", 60000.f) || !(m->d = doc_open(m->s, document))) return nullptr;
    return m;
}
void motolii_close(Motolii* m) { if (m && m->d) sqlite3_close(m->d->file); }
const char* motolii_trouble(Motolii* m) { return m->trouble.empty() ? m->d->trouble.c_str() : m->trouble.c_str(); }
static int told(Motolii* m, bool ok, const char* otherwise = nullptr) { if (!ok && otherwise) m->trouble = otherwise; return ok ? 1 : 0; }
static Doc& doc(Motolii* m) { m->trouble.clear(); m->d->trouble.clear(); return *m->d; }

int motolii_add(Motolii* m, const char* kind, const char* file) { return doc_add(doc(m), kind, file); }
int motolii_remove(Motolii* m, int thing) { return doc_remove(doc(m), thing); }
int motolii_set(Motolii* m, int thing, const char* name, double number) { return doc_set(doc(m), thing, name, number); }
int motolii_say(Motolii* m, int thing, const char* name, const char* word) { return doc_say(doc(m), thing, name, word); }
int motolii_key(Motolii* m, int thing, const char* name, double at, double number) { return doc_key(doc(m), thing, name, at, number); }
int motolii_unkey(Motolii* m, int thing, const char* name, double at) { return doc_unkey(doc(m), thing, name, at); }
int motolii_keys(Motolii* m, int thing, const char* name) { return doc_keys(doc(m), thing, name); }
int motolii_way(Motolii* m, int thing, const char* name, double at, const char* way) { return doc_way(doc(m), thing, name, at, way); }
int motolii_define(Motolii* m, const char* way, const char* expression) { return doc_define(doc(m), way, expression); }
const char* motolii_ways(Motolii* m) { return (m->listed = doc_ways(*m->d)).c_str(); }
int motolii_value(Motolii* m, int thing, const char* name, double at, double* is) { return doc_value(doc(m), thing, name, at, is); }
int motolii_undo(Motolii* m) { return doc_undo(doc(m)); }
int motolii_redo(Motolii* m) { return doc_redo(doc(m)); }

double motolii_seconds(Motolii* m) { return doc_seconds(*m->d); }
int motolii_at(Motolii* m, double seconds) {
    char runs[80];
    std::snprintf(runs, sizeof runs, "the document runs from 0 to %.2f seconds", doc_seconds(doc(m)));
    if (!told(m, seconds >= 0.0 && seconds <= doc_seconds(*m->d), runs)) return 0;
    return time_is(m->s, seconds), 1;
}
static bool ends(const char* name, const char* with) { return strlen(name) > strlen(with) && !strcmp(name + strlen(name) - strlen(with), with); }
static bool placed(const char* name) { return std::filesystem::is_directory(std::filesystem::absolute(name).parent_path()); }
int motolii_still(Motolii* m, const char* png) {
    doc(m);
    if (!told(m, ends(png, ".png"), "a still is a .png") || !told(m, placed(png), "there is no such folder")) return 0;
    time_is(m->s, m->s.now);
    return told(m, shoot(m->s, png), "the file could not be written");
}
int motolii_write(Motolii* m, const char* mp4) {
    doc(m);
    if (!told(m, ends(mp4, ".mp4"), "a film is written as an .mp4") || !told(m, placed(mp4), "there is no such folder")) return 0;
    return told(m, film(m->s, mp4, (int)std::lround(doc_seconds(*m->d) * 60.0), [](float) {}), "the file could not be written");
}
const char* motolii_list(Motolii* m) {
    m->listed.clear();
    doc_rows(*m->d, sqlite3_mprintf("select thing || ' ' || name || ' at ' || at || ': ' || value || coalesce(', then ' || way, '') from fact order by thing, name, at"), [m](sqlite3_stmt* row) { m->listed += std::string("  ") + (const char*)sqlite3_column_text(row, 0) + "\n"; });
    return m->listed.c_str();
}
void motolii_timed(Motolii* m, int pictures) { clock(m->s, pictures, [](float) {}); }
