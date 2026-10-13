// edit.mm: a document on the stage, edited by calls (doc.h): one program for every document, never built again for a
// scene. It reads one call a line (what a shell on a screen would send), and shows the stage as it then is.
//   ./edit.sh out/first.motolii            then, a line each:
//     add film ../vid/happy.mp4 | add picture <an After Effects file, a photo> | add solid <file.glb> | add glass ring      -> the new thing's number
//     set 1 x 0.5 | say 1 lay add | key 1 x 2 -0.5 | unkey 1 x 2 | remove 1 | undo | redo
//     list | at 1.5 (the stage's moment) | still out/a.png | write out/a.mp4 | time
// A call that is not done says why, and leaves the document as it was.
#include "kit.h"
#include "doc.h"
#include <iostream>
#include <sstream>

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
        std::string call, name, word;
        int thing = 0;
        double at = 0, is = 0;
        said >> call;
        const auto t0 = std::chrono::steady_clock::now();
        bool ok = true;
        if (call == "add" && said >> name >> word) std::printf("thing %d\n", thing = doc_add(*d, name.c_str(), word.c_str())), ok = thing > 0;
        else if (call == "remove" && said >> thing) ok = doc_remove(*d, thing);
        else if (call == "set" && said >> thing >> name >> is) ok = doc_set(*d, thing, name.c_str(), is);
        else if (call == "say" && said >> thing >> name >> word) ok = doc_say(*d, thing, name.c_str(), word.c_str());
        else if (call == "key" && said >> thing >> name >> at >> is) ok = doc_key(*d, thing, name.c_str(), at, is);
        else if (call == "unkey" && said >> thing >> name >> at) ok = doc_unkey(*d, thing, name.c_str(), at);
        else if (call == "undo") ok = doc_undo(*d);
        else if (call == "redo") ok = doc_redo(*d);
        else if (call == "at" && said >> at) {
            if (at < 0.0 || at > doc_seconds(*d)) std::printf("(the document runs from 0 to %.2f)\n", doc_seconds(*d));
            time_is(s, at);
        }
        else if (call == "still" && said >> word) { time_is(s, s.now); ok = shoot(s, word.c_str()); }
        else if (call == "write" && said >> word) ok = film(s, word.c_str(), (int)std::lround(doc_seconds(*d) * 60.0), [](float) {});
        else if (call == "time") clock(s, 120, [](float) {});
        else if (call == "list") doc_rows(*d, sqlite3_mprintf("select thing, name, at, value from fact order by thing, name, at"), [](sqlite3_stmt* row) {
            std::printf("  %d %s at %g: %s\n", sqlite3_column_int(row, 0), sqlite3_column_text(row, 1), sqlite3_column_double(row, 2), sqlite3_column_text(row, 3)); });
        else if (!call.empty()) { std::printf("not a call: %s\n", line.c_str()); continue; }
        if (!call.empty() && call != "list") std::printf("%s: %s, %.1f ms\n", call.c_str(), ok ? "done" : ("NOT done: " + d->trouble).c_str(), std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        std::fflush(stdout);
    }
    return 0;
}
