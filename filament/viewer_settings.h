// viewer_settings.h: Filament's own settings file as a whole-screen effect on the shelf. The file is libs/viewer's JSON
// (viewer/Settings.h: {"view": {"colorGrading": {...}, "bloom": {...}, "fog": {...}, ...}}, what gltf_viewer saves and
// reads); its numbers and on-offs are knobs on its card, as an After Effects file's slots are (lottie_slots.h, and by its
// rule for a range none says), and the values as they are now are read back by Filament's own JsonSerializer and laid
// on the stage's view over what the stage's own values make (doc_place.h doc_look): a part the file does not name stays
// as the stage has it. Its words (enum names: a tone mapping, a custom LUT) are not knobs: the file says them.
#pragma once
#include "script.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/document.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/pointer.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/stringbuffer.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/writer.h"
#include <viewer/Settings.h>
#include <cmath>
#include <fstream>
#include <functional>
#include <map>
#include <sstream>
#include <string>
#include <vector>

// The file's text, read again only after the window says a file changed (files_changed).
static const std::string& viewer_text(const std::string& file) {
    struct Kept { int seen = -1; std::string text; };
    static std::map<std::string, Kept> kept;
    Kept& one = kept[file];
    if (one.seen != files_changed()) {
        std::ifstream in(file);
        std::stringstream text;
        text << in.rdbuf();
        one.text = text.str(), one.seen = files_changed();
    }
    return one.text;
}
// Read as Filament's own reader (jsmn, not strict, numbers by strtof) takes what Filament's own writer makes: a comma
// before a closing brace (its agxToneMapper), none after a list (its cascadeSplitPositions), a colour space said bare
// ("colorspace": Rec709-sRGB-D65) and a fog that never ends said inf (Settings.cpp's writeJson); an After Effects file
// has its layers.
static bool viewer_read(std::string text, rapidjson::Document& doc) {
    const auto say = [&](const std::string& was, const std::string& as) {
        for (size_t at = text.find(was); at != std::string::npos; at = text.find(was, at + as.size())) text.replace(at, was.size(), as);
    };
    for (const char* bare : {"Rec709-sRGB-D65", "Rec709-Linear-D65"}) say(std::string(": ") + bare, std::string(": \"") + bare + "\"");
    say(": inf", ": Infinity"), say(": -inf", ": -Infinity"), say("]\n\"", "],\n\"");
    constexpr unsigned loose = rapidjson::kParseTrailingCommasFlag | rapidjson::kParseNanAndInfFlag;
    return !doc.Parse<loose>(text.c_str()).HasParseError() && doc.IsObject() && doc.HasMember("view") && doc["view"].IsObject() && !doc.HasMember("layers");
}
static bool viewer_is(const std::string& file) {
    rapidjson::Document doc;
    return viewer_read(viewer_text(file), doc);
}

// Each knob's words: "viewer <its place in the file, a JSON Pointer> <which of its numbers, -1 when it is one>".
static void viewer_walk(const rapidjson::Value& v, const std::string& at, std::vector<ScriptKnob>& knobs) {
    if (v.IsObject()) {
        for (auto& m : v.GetObject()) viewer_walk(m.value, at + "/" + m.name.GetString(), knobs);
        return;
    }
    std::string name = at.substr(6), label = name;   // (after "/view/")
    for (char& c : name) if (c == '/') c = '_';
    for (char& c : label) if (c == '/') c = ' ';
    if (v.IsBool()) {
        ScriptKnob one;
        one.kind = "check", one.name = name, one.label = label, one.usual = v.GetBool() ? 1 : 0, one.words = "viewer " + at + " -1";
        knobs.push_back(one);
        return;
    }
    std::vector<double> n;
    if (v.IsNumber() && std::isfinite(v.GetDouble())) n.push_back(v.GetDouble());
    else if (v.IsArray() && v.Size() <= 4) for (auto& e : v.GetArray()) { if (!e.IsNumber() || !std::isfinite(e.GetDouble())) return; n.push_back(e.GetDouble()); }
    static const char* const xyzw[] = {"x", "y", "z", "w"};
    for (size_t i = 0; i < n.size(); ++i) {
        ScriptKnob one;
        one.kind = "track";
        one.name = v.IsArray() ? name + "_" + xyzw[i] : name;
        one.label = v.IsArray() ? label + " " + xyzw[i] : label;
        one.usual = n[i];
        one.low = n[i] < 0 ? 4 * n[i] : 0, one.high = std::max(1.0, 4 * std::fabs(n[i]));   // (lottie_slots.h's rule for a range none says)
        one.step = 0.001;
        one.words = "viewer " + at + " " + std::to_string(v.IsArray() ? (int)i : -1);
        knobs.push_back(one);
    }
}
static std::vector<ScriptKnob> viewer_knobs(const std::string& file) {
    std::vector<ScriptKnob> knobs;
    rapidjson::Document doc;
    if (viewer_read(viewer_text(file), doc)) viewer_walk(doc["view"], "/view", knobs);
    return knobs;
}

// The file with each knob's number now put in its place, as Filament's JsonSerializer reads it ("" when it does not read).
static std::string viewer_said(const std::string& file, const std::vector<ScriptKnob>& knobs, const std::function<double(size_t)>& now) {
    rapidjson::Document doc;
    if (!viewer_read(viewer_text(file), doc)) return "";
    for (size_t k = 0; k < knobs.size(); ++k) {
        std::istringstream words(knobs[k].words);
        std::string viewer, at;
        int i = -1;
        words >> viewer >> at >> i;
        rapidjson::Value* v = rapidjson::Pointer(at.c_str()).Get(doc);
        if (!v) continue;
        if (v->IsBool()) v->SetBool(now(k) > 0.5);
        else if (v->IsNumber() && i < 0) v->SetDouble(now(k));
        else if (v->IsArray() && i >= 0 && i < (int)v->Size()) (*v)[i].SetDouble(now(k));
    }
    rapidjson::StringBuffer out;
    rapidjson::Writer<rapidjson::StringBuffer, rapidjson::UTF8<>, rapidjson::UTF8<>, rapidjson::CrtAllocator, rapidjson::kWriteNanAndInfFlag> write(out);   // (Infinity: strtof reads it)
    doc.Accept(write);
    return out.GetString();
}
