// midi_roll.cpp: a MIDI file's piano roll as an SVG, made by bladeSk's PianoRollSvg.js as it is
// (third_party/midi_roll, MIT), run in QuickJS as three_isf.cpp runs three.js. Its MIDI reader (midi-parser-js) is
// GPL-3 and not here: it is handed the notes GStreamer's midiparse reads (midi.h), in that reader's shape
// (MidiParser.parse: timeDivision, formatType, track[].event[] of deltaTime, type 9 on / 8 off, channel, data).
// Its look is ALMaMPlayer's (github.com/almam72/ALMAMPlayer, its numbers only): a whole song a row, 300 points a
// second (GlobalVariables.speed), a row 688 / the notes' range (main_interface.gd), notes 18 high with round ends
// (note_texture.png, 20 x 18: a row's rect, its rx half of it, drawn in from its edge by a stroke of the ground's
// colour: a round-ended bar drawn in is one still), its colour on black.
#include "midi_roll.h"
#include "midi.h"
#include <quickjs.h>
#include <cmath>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <cstring>

std::string midi_roll(const std::string& file, double bpm, std::string& why) {
    std::vector<MidiEvent> events = midi_events(file.c_str());
    if (events.empty()) return why = "no notes in " + file, "";
    const double by = bpm > 0 ? midi_bpm(file.c_str()) / bpm : 1.0;   // (the file's beats at another tempo: midi.h)
    std::ostringstream song;   // (a tick a millisecond: timeDivision 1000 ticks a "beat" of a second)
    song << "{\"timeDivision\":1000,\"formatType\":0,\"track\":[{\"event\":[";
    double was = 0, end = 0;
    for (size_t i = 0; i < events.size(); ++i) {
        const MidiEvent& e = events[i];
        const double at = std::round(e.at * by * 1000.0);
        song << (i ? "," : "") << "{\"deltaTime\":" << (at - was) << ",\"type\":" << (e.on ? 9 : 8) << ",\"channel\":" << e.channel << ",\"data\":[" << e.pitch << "," << (e.on ? e.velocity : 0) << "]}";
        was = at, end = std::max(end, at);
    }
    song << "]}]}";
    const int seconds = (int)std::ceil(end / 1000.0) + 1;
    const double spacing = midi_spacing(events), inset = std::max(0.0, spacing - 18.0);   // (a row ALMaMPlayer's; a note 18 high in it, its edge the ground's)
    std::ifstream in(std::filesystem::path(__FILE__).parent_path() / "third_party/midi_roll/PianoRollSvg.js");
    std::stringstream source;
    source << in.rdbuf();
    if (source.str().empty()) return why = "PianoRollSvg.js is not there", "";
    const std::string run = "var module = { exports: {} };\nvar MidiParser = { parse: () => (" + song.str() + ") };\n" + source.str() +
        "\nglobalThis.__said = new module.exports('').render({ barsPerRow: " + std::to_string(seconds) + ", barSubdivisions: 1, timeDivision: 1000, width: " +
        std::to_string(seconds * 300) + ", lineHeight: " + std::to_string(spacing) + ", staggered: false, removeShorterThan: 0, tracksToRender: Object.fromEntries([...Array(16).keys()].map(i => [i, true])) });\n";
    JSRuntime* rt = JS_NewRuntime();
    JSContext* cx = JS_NewContext(rt);
    std::string out;
    JSValue ran = JS_Eval(cx, run.c_str(), run.size(), "<PianoRollSvg>", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(ran)) {
        JSValue e = JS_GetException(cx);
        const char* s = JS_ToCString(cx, e);
        why = std::string("PianoRollSvg.js: ") + (s ? s : "?");
        JS_FreeCString(cx, s), JS_FreeValue(cx, e);
    } else {
        JSValue g = JS_GetGlobalObject(cx), s = JS_GetPropertyStr(cx, g, "__said");
        if (const char* c = JS_ToCString(cx, s)) out = c, JS_FreeCString(cx, c);
        JS_FreeValue(cx, s), JS_FreeValue(cx, g);
    }
    JS_FreeValue(cx, ran);
    JS_FreeContext(cx), JS_FreeRuntime(rt);
    if (out.empty()) return why.empty() ? why = "PianoRollSvg.js made nothing" : why, "";
    // ALMaMPlayer's look, said after its own style (ThorVG takes a <style> only before what it styles, the later rule
    // winning, as CSS does), and the round ends as each note's rx (ThorVG's CSS reads no rx).
    const std::string look = "<style>.line { display: none; } .blackRow { display: none; } .octaveText { display: none; } "
                             ".note { fill: #ed9e85; stroke: #000; stroke-width: " + std::to_string(inset) + "; } .note_black { fill: #ed9e85; stroke: #000; stroke-width: " + std::to_string(inset) + "; }</style>";
    if (const size_t at = out.find("</style>"); at != std::string::npos) out.insert(at + 8, look);
    int i = 0;   // (each note's id, in the order it is drawn: lit by it, midi_roll_notes)
    for (size_t at = 0; (at = out.find("<rect class=\"note", at)) != std::string::npos; at += 6) out.insert(at + 5, " id=\"note" + std::to_string(i++) + "\" rx=\"" + std::to_string(0.5 * spacing) + "\"");
    return out;
}

std::string midi_roll_file(const std::string& file, double bpm, std::string& why) {   // (named by what goes in, as mosh.h names its films)
    std::string key = file + "|" + std::to_string(bpm);
    std::error_code e;
    if (auto at = std::filesystem::last_write_time(file, e); !e) key += "|" + std::to_string((long long)at.time_since_epoch().count());
    uint64_t hash = 1469598103934665603ull;
    for (unsigned char c : key) hash = (hash ^ c) * 1099511628211ull;
    const char* home = std::getenv("HOME");
    const std::filesystem::path dir = std::filesystem::path(home ? home : "/tmp") / "Library/Caches/Motolii/midi";
    char name[40];
    std::snprintf(name, sizeof name, "%016llx.svg", (unsigned long long)hash);
    const std::string out = (dir / name).string();
    if (std::filesystem::exists(out, e)) return out;
    const std::string svg = midi_roll(file, bpm, why);
    if (svg.empty()) return "";
    std::filesystem::create_directories(dir, e);
    std::ofstream(out) << svg;
    return out;
}

bool midi_roll_box(const std::string& svg, double& w, double& h) {   // (its viewBox: points of the roll, 300 a second)
    std::ifstream in(svg);
    std::string head(512, '\0');
    in.read(head.data(), (std::streamsize)head.size());
    const size_t at = head.find("viewBox=\"");
    return at != std::string::npos && std::sscanf(head.c_str() + at + 9, "%*f %*f %lf %lf", &w, &h) == 2 && w > 0 && h > 0;
}

std::vector<MidiRollNote> midi_roll_notes(const std::string& svg) {   // (each note's x, y, width and height, 300 a second: bladeSk's x is a moment, its width a length, + 0.5)
    std::ifstream in(svg);
    std::stringstream all;
    all << in.rdbuf();
    const std::string text = all.str();
    std::vector<MidiRollNote> notes;
    const auto number = [&](size_t from, const char* name) { const size_t at = text.find(name, from); return at == std::string::npos ? 0.0 : std::atof(text.c_str() + at + std::strlen(name)); };
    for (size_t at = 0; (at = text.find("id=\"note", at)) != std::string::npos; ++at) {
        MidiRollNote n;
        n.x = number(at, " x=\""), n.y = number(at, " y=\""), n.high = number(at, " height=\"");
        n.on = (n.x - 0.5) / 300.0, n.off = n.on + number(at, " width=\"") / 300.0;
        notes.push_back(n);
    }
    return notes;
}
