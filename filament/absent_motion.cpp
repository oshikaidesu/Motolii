// absent_motion.cpp: AbsentMotion's own motion effects (Auto Shake, Random Displace, Random Jitter, Oscillate, Swing,
// Spin, Pulse Size: its motion.cpp ApplyMotion; Apache-2.0, Copyright 2026 The AbsentMotion authors), run as AbsentMotion
// runs them: a layer of its own (scene.h) given the thing's time and its effects in order, its placement begun where
// nothing is moved. Built apart (dawn_probe/kit_wgpu.sh), beside AbsentMotion's motion.cpp, scene.cpp and easing.cpp as
// they are (what of scene.cpp draws or reads a scene is not reached: the library is linked dead-stripped).
#include "absent_motion.h"
#include "scene.h"
#include <cmath>
#include <cstring>

static am::Layer absent_layer(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs) {
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
// As AbsentMotion's CopiesOf (styles.cpp) lays them, its layer's matrix one that moves nothing: outer by local.
std::vector<AbsentCopy> absent_copies(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, float wide, float high) {
    const am::Layer l = absent_layer(effects, thing, startMs, endMs);
    if (!am::HasRepeat(l)) return {};
    am::RepeatSpace space;
    space.width = wide, space.height = high, space.radius = std::max(wide, high) * 0.5f * 1.415f;
    std::vector<am::RepeatCopy> copies;
    am::RepeatCopies(l, nowMs, space, &copies);
    std::vector<AbsentCopy> out;
    out.reserve(copies.size());
    for (const am::RepeatCopy& c : copies) {
        const SkMatrix m = c.outer * c.local;
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
