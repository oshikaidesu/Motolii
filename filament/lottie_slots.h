// lottie_slots.h: an After Effects file's slots (the Lottie format's "slots": values its maker let be set from outside,
// each by its sid, {"sid": {"p": {"a": 0, "k": value}}}) as knobs on its card, and the values as they are now said back
// to ThorVG in that same form (LottieAnimation::gen, then apply: as ThorVG's own test/testLottie.cpp does).
// A number is one knob, two numbers x and y, three or four colour (0 to 1). Lottie says no range: a number's is from 0
// (or four times below it, when below 0) to four times itself (at least 1). Text and picture slots are not knobs.
#pragma once
#include "script.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/document.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/stringbuffer.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/writer.h"
#include <cmath>
#include <fstream>
#include <map>
#include <sstream>
#include <string>
#include <vector>

// Each knob's words: "lottie <sid> <which of its numbers> <how many it has>".
static std::vector<ScriptKnob> lottie_knobs(const std::string& file) {
    std::vector<ScriptKnob> knobs;
    std::ifstream in(file);
    std::stringstream text;
    text << in.rdbuf();
    rapidjson::Document doc;
    if (doc.Parse(text.str().c_str()).HasParseError() || !doc.IsObject() || !doc.HasMember("slots") || !doc["slots"].IsObject()) return knobs;
    for (auto& slot : doc["slots"].GetObject()) {
        if (!slot.value.IsObject() || !slot.value.HasMember("p") || !slot.value["p"].IsObject() || !slot.value["p"].HasMember("k")) continue;
        const rapidjson::Value* k = &slot.value["p"]["k"];
        const bool moves = slot.value["p"].HasMember("a") && slot.value["p"]["a"].IsNumber() && slot.value["p"]["a"].GetInt() == 1;
        if (moves && k->IsArray() && !k->Empty() && (*k)[0].IsObject() && (*k)[0].HasMember("s")) k = &(*k)[0]["s"];   // (keyed in the file: its first key's value)
        std::vector<double> v;
        if (k->IsNumber()) v.push_back(k->GetDouble());
        else if (k->IsArray()) for (auto& n : k->GetArray()) if (n.IsNumber()) v.push_back(n.GetDouble());
        if (v.empty() || v.size() > 4) continue;
        const std::string sid = slot.name.GetString();
        static const char* const xy[] = {"x", "y"}, *const rgba[] = {"r", "g", "b", "a"};
        for (size_t i = 0; i < v.size(); ++i) {
            ScriptKnob one;
            one.kind = "track";
            one.name = v.size() == 1 ? sid : sid + "_" + (v.size() == 2 ? xy[i] : rgba[i]);
            one.label = v.size() == 1 ? sid : sid + " " + (v.size() == 2 ? xy[i] : rgba[i]);
            one.usual = v[i];
            if (v.size() >= 3) one.low = 0, one.high = 1;
            else one.low = v[i] < 0 ? 4 * v[i] : 0, one.high = std::max(1.0, 4 * std::fabs(v[i]));
            one.step = 0.001;
            one.words = "lottie " + sid + " " + std::to_string(i) + " " + std::to_string(v.size());
            knobs.push_back(one);
        }
    }
    return knobs;
}
// The slots as the values are now ({"sid": {"p": {"a": 0, "k": ...}}}), each knob's value by its place in `values`:
// only those set otherwise than the file has them (a slot left alone goes on as the file moves it).
static std::string lottie_slots_json(const std::vector<ScriptKnob>& knobs, const std::vector<double>& values) {
    std::map<std::string, std::vector<double>> by;
    std::map<std::string, bool> touched;
    for (size_t i = 0; i < knobs.size() && i < values.size(); ++i) {
        std::istringstream w(knobs[i].words);
        std::string lottie, sid;
        if (w >> lottie >> sid && lottie == "lottie" && values[i] != knobs[i].usual) touched[sid] = true;
    }
    for (size_t i = 0; i < knobs.size() && i < values.size(); ++i) {
        std::istringstream w(knobs[i].words);
        std::string lottie, sid;
        size_t at = 0, count = 0;
        if (!(w >> lottie >> sid >> at >> count) || lottie != "lottie" || !touched.count(sid)) continue;
        std::vector<double>& v = by[sid];
        v.resize(count, 0.0);
        if (at < count) v[at] = values[i];
    }
    rapidjson::StringBuffer out;
    rapidjson::Writer<rapidjson::StringBuffer> w(out);
    w.StartObject();
    for (const auto& [sid, v] : by) {
        w.Key(sid.c_str()), w.StartObject(), w.Key("p"), w.StartObject(), w.Key("a"), w.Int(0), w.Key("k");
        if (v.size() == 1) w.Double(v[0]);
        else {
            w.StartArray();
            for (double n : v) w.Double(n);
            w.EndArray();
        }
        w.EndObject(), w.EndObject();
    }
    w.EndObject();
    return by.empty() ? std::string() : std::string(out.GetString());
}
