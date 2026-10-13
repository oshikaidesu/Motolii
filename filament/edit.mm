// edit.mm: a document on the stage, edited by calls (doc.h): one program for every document, never built again for a
// scene. It reads one call a line (what a shell on a screen would send), and shows the stage as it then is.
//   ./edit.sh out/first.motolii       then a call a line (`help` lists them)
// A call that is not done says why, and leaves the document as it was.
#include "kit.h"
#include "doc.h"
#include <iostream>
#include <sstream>

static const char* const calls =
    "  add film <file> | add picture <an After Effects file, a .jpg, a .png> | add solid <file.glb> | add glass ring|gem|cube\n"
    "  remove <thing> | set <thing> <name> <number> | say <thing> <name> <word> | key <thing> <name> <at> <number> | unkey <thing> <name> <at>\n"
    "  undo | redo | list | at <seconds> | still <file.png> | write <file.mp4> | time\n";

int main(int argc, char** argv) {
    if (argc < 2) return fail("edit <document>");
    Stage s = open_stage(1080, 1920);
    s.camera->lookAt({0.f, 0.f, 6.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    s.camera->setProjection(30.0, 1080.0 / 1920.0, 0.05, 60.0, Camera::Fov::VERTICAL);
    if (!room(s, "out/small_studio/small_studio_ibl.ktx", 60000.f)) return fail("out/small_studio/small_studio_ibl.ktx");
    Doc* d = doc_open(s, argv[1]);
    if (!d) return fail("the document could not be opened");
    for (std::string line; std::getline(std::cin, line);) {
        std::istringstream said(line);
        std::vector<std::string> w;
        for (std::string word; said >> word;) w.push_back(word);
        if (w.empty()) continue;
        const std::string& call = w[0];
        bool numbers = true;   // (whether every word asked for as a number was one)
        const auto number = [&](size_t i) { char* end = nullptr; const double is = std::strtod(w[i].c_str(), &end); if (*end || !std::isfinite(is)) numbers = false; return is; };
        const auto thing = [&] { const double n = number(1); return numbers && n == std::floor(n) ? (int)n : (numbers = false, 0); };
        const auto ends = [&](size_t i, const char* with) { return w[i].size() > strlen(with) && !w[i].compare(w[i].size() - strlen(with), std::string::npos, with); };
        const auto t0 = std::chrono::steady_clock::now();
        bool ok = true, known = true;
        d->trouble.clear();
        if (call == "add" && w.size() == 3) { const int made = doc_add(*d, w[1].c_str(), w[2].c_str()); if ((ok = made > 0)) std::printf("thing %d\n", made); }
        else if (call == "remove" && w.size() == 2) ok = doc_remove(*d, thing());
        else if (call == "set" && w.size() == 4) {
            const int n = thing(), keys = numbers ? doc_keys(*d, n, w[2].c_str()) : 0;
            const double is = number(3);
            if (numbers && (ok = doc_set(*d, n, w[2].c_str(), is)) && keys) std::printf("(its %d keys are gone: it stays at that)\n", keys);
        }
        else if (call == "say" && w.size() == 4) ok = doc_say(*d, thing(), w[2].c_str(), w[3].c_str());
        else if (call == "key" && w.size() == 5) { const int n = thing(); const double at = number(3), is = number(4); if (numbers) ok = doc_key(*d, n, w[2].c_str(), at, is); }
        else if (call == "unkey" && w.size() == 4) { const int n = thing(); const double at = number(3); if (numbers) ok = doc_unkey(*d, n, w[2].c_str(), at); }
        else if (call == "undo" && w.size() == 1) ok = doc_undo(*d);
        else if (call == "redo" && w.size() == 1) ok = doc_redo(*d);
        else if (call == "at" && w.size() == 2) {
            const double at = number(1);
            char runs[80];
            std::snprintf(runs, sizeof runs, "the document runs from 0 to %.2f seconds", doc_seconds(*d));
            if (numbers && !(ok = at >= 0.0 && at <= doc_seconds(*d))) d->trouble = runs;
            if (numbers && ok) time_is(s, at);
        }
        else if (call == "still" && w.size() == 2) {
            if (!(ok = ends(1, ".png"))) d->trouble = "a still is a .png";
            else if (time_is(s, s.now), !(ok = shoot(s, w[1].c_str()))) d->trouble = "the file could not be written";
        }
        else if (call == "write" && w.size() == 2) {
            if (!(ok = ends(1, ".mp4"))) d->trouble = "a film is written as an .mp4";
            else if (!(ok = film(s, w[1].c_str(), (int)std::lround(doc_seconds(*d) * 60.0), [](float) {}))) d->trouble = "the file could not be written";
        }
        else if (call == "time" && w.size() == 1) clock(s, 120, [](float) {});
        else if (call == "help") { std::printf("%s", calls); continue; }
        else if (call == "list" && w.size() == 1) {
            doc_rows(*d, sqlite3_mprintf("select thing, name, at, value from fact order by thing, name, at"), [](sqlite3_stmt* row) {
                std::printf("  %d %s at %g: %s\n", sqlite3_column_int(row, 0), sqlite3_column_text(row, 1), sqlite3_column_double(row, 2), sqlite3_column_text(row, 3)); });
            std::printf("  (the document runs %.2f seconds)\n", doc_seconds(*d));
            continue;
        }
        else known = false;
        if (!known || !numbers) std::printf("not a call: %s\n%s", line.c_str(), !numbers ? "  (a number was asked for where a word stands)\n" : calls);
        else std::printf("%s: %s, %.1f ms\n", call.c_str(), ok ? "done" : ("NOT done: " + d->trouble).c_str(), std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        std::fflush(stdout);
    }
    return 0;
}
