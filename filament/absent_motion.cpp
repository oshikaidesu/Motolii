// absent_motion.cpp: AbsentMotion's own motion effects (Auto Shake, Random Displace, Random Jitter, Oscillate, Swing,
// Spin, Pulse Size: its motion.cpp ApplyMotion; Apache-2.0, Copyright 2026 The AbsentMotion authors), run as AbsentMotion
// runs them: a layer of its own (scene.h) given the thing's time and its effects in order, its placement begun where
// nothing is moved. Built apart (dawn_probe/kit_wgpu.sh), beside AbsentMotion's motion.cpp, scene.cpp and easing.cpp as
// they are (what of scene.cpp draws or reads a scene is not reached: the library is linked dead-stripped).
#include "absent_motion.h"
#include "scene.h"

AbsentMoved absent_motion(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs) {
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
    am::Placement p{};
    p.location = SkPoint{0, 0}, p.pivot = SkPoint{0, 0}, p.scale = SkPoint{1, 1}, p.rotation = 0, p.z = 0;
    am::ApplyMotion(l, nowMs, &p);
    return {p.location.fX, p.location.fY, p.z, p.rotation, p.scale.fX, p.scale.fY};
}
