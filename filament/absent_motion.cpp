// absent_motion.cpp: AbsentMotion's own motion effects (Auto Shake, Random Displace, Random Jitter, Oscillate, Swing,
// Spin, Pulse Size: its motion.cpp ApplyMotion; Apache-2.0, Copyright 2026 The AbsentMotion authors), run as AbsentMotion
// runs them: a layer of its own (scene.h) given the thing's time and its effects in order, its placement begun where
// nothing is moved. Built apart (dawn_probe/kit_wgpu.sh), beside AbsentMotion's motion.cpp, scene.cpp and easing.cpp as
// they are (what of scene.cpp draws or reads a scene is not reached: the library is linked dead-stripped).
#include "absent_motion.h"
#include "scene.h"
#include "textanim.h"
#include <algorithm>
#include <bit>
#include <cmath>
#include <cstring>

am::Layer absent_layer(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs) {
    am::Layer l;
    l.id = l.noiseId = thing;
    l.startTime = (int)startMs, l.endTime = (int)endMs;
    for (const AbsentEffect& e : effects) {
        am::LayerEffect one;
        one.id = "absent." + e.id;
        for (const auto& [name, v] : e.params) {
            am::Property p;
            p.name = name, p.anim.value = v;
            one.props.push_back(p);
        }
        l.effects.push_back(one);
    }
    return l;
}
AbsentMoved absent_motion(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs) {
    const am::Layer l = absent_layer(effects, thing, startMs, endMs);
    am::Placement p{};
    p.location = SkPoint{0, 0}, p.pivot = SkPoint{0, 0}, p.scale = SkPoint{1, 1}, p.rotation = 0, p.z = 0;
    am::ApplyMotion(l, nowMs, &p);
    return {p.location.fX, p.location.fY, p.z, p.rotation, p.scale.fX, p.scale.fY};
}
float absent_opacity(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs) {
    return am::MotionOpacity(absent_layer(effects, thing, startMs, endMs), nowMs);
}
// As AbsentMotion's CopiesOf (styles.cpp) lays them: its layer's matrix where it stands (its turn and size the thing's
// own), each copy outer by the layer by local, seen from the layer's own place.
std::vector<AbsentCopy> absent_copies(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, float wide, float high, float atX, float atY, float reach) {
    const am::Layer l = absent_layer(effects, thing, startMs, endMs);
    if (!am::HasRepeat(l)) return {};
    am::RepeatSpace space;
    space.width = wide, space.height = high, space.radius = reach;
    space.anchor = SkPoint{atX, atY}, space.layer = SkMatrix::Translate(atX, atY);
    std::vector<am::RepeatCopy> copies;
    am::RepeatCopies(l, nowMs, space, &copies);
    std::vector<AbsentCopy> out;
    out.reserve(copies.size());
    for (const am::RepeatCopy& c : copies) {
        const SkMatrix m = SkMatrix::Translate(-atX, -atY) * c.outer * space.layer * c.local;
        out.push_back({m.getTranslateX(), m.getTranslateY(), std::atan2(m.getSkewY(), m.getScaleX()) * 180.0 / M_PI, std::hypot(m.getScaleX(), m.getSkewY()), c.alpha});
    }
    return out;
}
// One of AbsentMotion's eases at u, its numbers as Alight Motion says them (easing.h: bounce, elastic, cyclic, random, step, elasticStep).
double absent_ease(const char* type, const double* p, int n, double u) {
    static const struct { const char* name; am::Easing::Type type; } kinds[] = {{"bounce", am::Easing::kBounce}, {"elastic", am::Easing::kElastic}, {"cyclic", am::Easing::kCyclic}, {"random", am::Easing::kRandom}, {"step", am::Easing::kStep}, {"elasticStep", am::Easing::kElasticStep}};
    am::Easing e;
    for (const auto& k : kinds) if (!std::strcmp(k.name, type)) e.type = k.type;
    for (int i = 0; i < n && i < 8; ++i) e.p[e.count++] = (float)p[i];
    return am::Ease(e, (float)u);
}
// As AbsentMotion's text.cpp Shown changes a text layer's words, its Choice and Seed as there (they lie in text.cpp
// with its Skia paragraphs, not built here): each text effect in its order, by textanim.cpp's own CountNumbers,
// FormatTimecode and Scramble. Text Randomizer's own characters are none here (a knob of words is not had): Scramble's own
// set then, as AbsentMotion's when none is typed.
bool absent_text_on(const std::vector<AbsentEffect>& effects) {
    return std::any_of(effects.begin(), effects.end(), [](const AbsentEffect& e) { return e.id == "countupdown" || e.id == "timecode" || e.id == "textrandomizer"; });
}
std::string absent_text(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, int fps, std::string text) {
    const am::Layer l = absent_layer(effects, thing, startMs, endMs);
    const float t = l.localTime(nowMs);
    const auto Choice = [&](const am::LayerEffect& e, const char* name, int fallback, int last) { return std::clamp((int)std::lround(e.value(name, t, (float)fallback)), 0, last); };
    const auto Seed = [&](const am::LayerEffect& e) { return am::textanim::Mix((uint64_t)l.id ^ am::textanim::Mix((uint64_t)std::llround((double)e.value("seed", t, 1) * 1000))); };
    const double begin = am::textanim::FrameMs(l.startTime, fps);
    const double now = am::textanim::FrameMs(l.startTime + (double)t * std::max(0, l.endTime - l.startTime), fps);
    const double local = (now - (std::floor(begin) == l.startTime ? begin : l.startTime)) / 1000.0;
    for (const am::LayerEffect& e : l.effects) {
        if (e.hidden) continue;
        if (e.id == "absent.countupdown") {
            text = am::textanim::CountNumbers(text, e.value("add", t, 0), e.value("multiply", t, 1), Choice(e, "decimals", 0, 4) - 1, Choice(e, "separator", 0, 3));
        } else if (e.id == "absent.timecode") {
            const double clock = Choice(e, "source", 0, 1) == 1 ? now / 1000.0 : local;
            text = am::textanim::FormatTimecode(clock * e.value("speed", t, 1) + e.value("offset", t, 0), fps, Choice(e, "format", 0, 5));
        } else if (e.id == "absent.textrandomizer") {
            const int64_t tick = (int64_t)std::floor(local * std::max(0.0f, e.value("speed", t, 12)) + 1e-6);
            const float evolution = e.value("evolution", t, 0);
            const uint64_t seed = Seed(e) ^ (evolution == 0 ? 0 : am::textanim::Mix(std::bit_cast<uint32_t>(evolution)));
            text = am::textanim::Scramble(text, std::clamp(e.value("progress", t, 0), 0.0f, 1.0f), Choice(e, "direction", 0, 3), seed, tick, Choice(e, "charset", 0, 6), "",
                                      e.value("keepSpaces", t, 1) >= 0.5f, std::clamp(e.value("amount", t, 1), 0.0f, 1.0f), e.value("start", t, 0), e.value("end", t, 1));
        }
    }
    return text;
}
bool absent_shutter(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, int fps, double* open, bool* moves) {
    am::ShutterBlur blur;
    if (!am::ShutterBlurOf(absent_layer(effects, thing, startMs, endMs), nowMs, &blur)) return false;
    *open = blur.angle / 360.0 / std::max(1, fps), *moves = blur.position;   // (as DrawShuttered opens it: a frame's time times its angle)
    return true;
}
