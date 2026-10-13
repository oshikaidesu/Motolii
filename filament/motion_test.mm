// motion_test.mm: motion.h against answers worked out by hand: a turn at an even pace, a child under a turning
// parent, an eased move, a looping expression, a layer that is only there for a while.   ./build_one.sh motion_test && ./out/motion_test
#include "motion.h"
#include <chrono>
#include <cmath>
#include <cstdio>

static int failed = 0;
static void near(const char* what, float got, float want, float within = 0.02f) {
    const bool ok = std::fabs(got - want) <= within;
    if (!ok) ++failed;
    std::printf("%s %-46s got %9.3f  want %9.3f\n", ok ? "ok  " : "FAIL", what, got, want);
}
// a point of a layer (flat or 3D) at a frame, against where it should be in the composition
static void there3(Motion& m, const char* layer, float frame, float px, float py, float pz, float ex, float ey, float ez) {
    filament::math::mat4f place;
    motion_at(m, frame);
    if (!motion_of(m, layer, &place)) { ++failed; std::printf("FAIL %s not found\n", layer); return; }
    const filament::math::float4 p = place * filament::math::float4{px, py, pz, 1.f};
    char what[96];
    std::snprintf(what, sizeof what, "%s (%g %g %g) x", layer, px, py, pz); near(what, p.x, ex, 0.01f);
    std::snprintf(what, sizeof what, "%s (%g %g %g) y", layer, px, py, pz); near(what, p.y, ey, 0.01f);
    std::snprintf(what, sizeof what, "%s (%g %g %g) z", layer, px, py, pz); near(what, p.z, ez, 0.01f);
}
// the eased share of a move at share t of its time, for the easing curve (x1, y1, x2, y2): After Effects' own kind
static float eased(float t, float x1, float y1, float x2, float y2) {
    float u = t;
    for (int i = 0; i < 30; ++i) {   // which place on the curve has the time t
        const float x = 3 * (1 - u) * (1 - u) * u * x1 + 3 * (1 - u) * u * u * x2 + u * u * u;
        const float dx = 3 * (1 - u) * (1 - u) * x1 + 6 * (1 - u) * u * (x2 - x1) + 3 * u * u * (1 - x2);
        u -= (x - t) / dx;
    }
    return 3 * (1 - u) * (1 - u) * u * y1 + 3 * (1 - u) * u * u * y2 + u * u * u;
}

int main() {
    Motion m;
    if (!motion_open(m, "motion_test.json")) { std::printf("FAIL open\n"); return 1; }
    near("width", m.w, 1080); near("height", m.h, 1920); near("frames", m.frames, 480); near("seconds", m.seconds, 8);
    tvg::Matrix a{};
    motion_at(m, 120);   // a quarter of the way: the rig has turned 90 degrees
    if (!motion_flat(m, "rig", &a)) { std::printf("FAIL rig not found\n"); return 1; }
    near("rig at 120: e11 (cos 90)", a.e11, 0); near("rig at 120: e12 (-sin 90)", a.e12, -1); near("rig at 120: e21", a.e21, 1);
    near("rig at 120: x", a.e13, 540); near("rig at 120: y", a.e23, 960);
    if (!motion_flat(m, "arm", &a)) { std::printf("FAIL arm not found\n"); return 1; }
    // the arm: 200 -> 400 along x over 240 frames, eased in and out; halfway in time is halfway along; under the rig turned 90
    near("arm at 120: x (under the rig)", a.e13, 540); near("arm at 120: y (under the rig)", a.e23, 960 + 300, 0.5f);
    near("arm at 120: size 1.25, turned 90: e12", a.e12, -1.25f); near("arm at 120: e21", a.e21, 1.25f);
    motion_at(m, 60);
    motion_flat(m, "arm", &a);
    const float along = 200.f + 200.f * eased(0.25f, 0.42f, 0.f, 0.58f, 1.f), turn = 45.f * 3.14159265f / 180.f;
    near("arm at 60: x (eased, rig at 45)", a.e13, 540 + std::cos(turn) * along, 0.5f);
    near("arm at 60: y (eased, rig at 45)", a.e23, 960 + std::sin(turn) * along, 0.5f);
    float seen = 0.f;
    motion_at(m, 90);    // the swing goes 0 -> 90 in 60 frames and its expression loops it to and fro: at 90 it is back at 45
    motion_flat(m, "swing", &a, &seen);
    near("swing at 90: angle (looped by its expression)", std::atan2(a.e21, a.e11) * 180.f / 3.14159265f, 45, 0.3f);
    motion_at(m, 150);   // down to 0 at 120, then up again: halfway up at 150
    motion_flat(m, "swing", &a);
    near("swing at 150: angle", std::atan2(a.e21, a.e11) * 180.f / 3.14159265f, 45, 0.3f);
    motion_at(m, 135);
    motion_flat(m, "swing", &a);
    near("swing at 135: angle (a quarter up again)", std::atan2(a.e21, a.e11) * 180.f / 3.14159265f, 22.5f, 0.3f);
    motion_at(m, 60);
    near("late at 60: there? (0 no)", motion_flat(m, "late", &a) ? 1 : 0, 0);
    motion_at(m, 180);
    near("late at 180: there? (1 yes)", motion_flat(m, "late", &a) ? 1 : 0, 1);
    near("late at 180: x", a.e13, 10); near("late at 180: y", a.e23, 20);
    motion_at(m, 240.5f);
    near("late at 240.5: there?", motion_flat(m, "late", &a) ? 1 : 0, 0);
    // a layer inside a precomposition: the precomposition stands at (500, 500) twice the size and half as opaque, and its
    // time is remapped to half speed; inside, a layer goes 0 -> 100 px in a second. At frame 60 of the film the inside
    // is half a second on: 50 px, which is 100 px of the film: x 600.
    motion_at(m, 60);
    seen = 0.f;
    near("inside (in a precomposition) at 60: there?", motion_flat(m, "inside", &a, &seen) ? 1 : 0, 1);
    near("inside at 60: x (precomposition's place, size and slowed time)", a.e13, 600, 0.5f);
    near("inside at 60: y", a.e23, 500, 0.5f); near("inside at 60: size 2", a.e11, 2);
    near("inside at 60: half as opaque, from the precomposition", seen, 0.5f);
    // anchor (10, 20), twice as wide and half as high, turned 90, put at (300, 400): the layer's own corner (0, 0) goes
    // (-10, -20) -> sized (-20, -10) -> turned (10, -20) -> placed (310, 380)
    motion_at(m, 10);
    motion_flat(m, "odd", &a);
    near("odd: its corner's x (anchor, size, turn, place)", a.e13, 310); near("odd: its corner's y", a.e23, 380);
    near("odd: e12 (-0.5)", a.e12, -0.5f); near("odd: e21 (2)", a.e21, 2);
    // a hold key: 10 degrees until frame 100, then 50 at once
    motion_at(m, 99);
    motion_flat(m, "held", &a);
    near("held at 99: still 10 degrees", std::atan2(a.e21, a.e11) * 180.f / 3.14159265f, 10, 0.05f);
    motion_at(m, 100);
    motion_flat(m, "held", &a);
    near("held at 100: 50 degrees", std::atan2(a.e21, a.e11) * 180.f / 3.14159265f, 50, 0.05f);
    // position with its x and y keyed apart: x 0 -> 100 over 100 frames, y fixed at 7
    motion_at(m, 25);
    motion_flat(m, "split", &a);
    near("split at 25: x", a.e13, 25, 0.05f); near("split at 25: y", a.e23, 7);
    // skew 45 degrees along x, as lottie-web has it: x' = x - y
    motion_flat(m, "skewed", &a);
    near("skewed: e12 (-1)", a.e12, -1); near("skewed: e11", a.e11, 1); near("skewed: e21", a.e21, 0); near("skewed: e22", a.e22, 1);
    // a layer put on a plane of the scene: here the floor, a hundredth of a unit a pixel, the picture's down towards +z
    using namespace filament::math;
    const float3 right{0.01f, 0.f, 0.f}, down{0.f, 0.f, 0.01f};
    const tvg::Matrix moved{1, 0, 100, 0, 1, 200, 0, 0, 1}, turned{0, -1, 0, 1, 0, 0, 0, 0, 1}, doubled{2, 0, 0, 0, 2, 0, 0, 0, 1};
    float4 p = motion_in_scene(motion_deep(moved), float3{0.f}, right, down) * float4{0, 0, 0, 1};
    near("on the floor: 100 px right is x 1", p.x, 1); near("on the floor: 200 px down is z 2", p.z, 2); near("on the floor: height 0", p.y, 0);
    p = motion_in_scene(motion_deep(turned), float3{0.f}, right, down) * float4{1, 0, 0, 0};
    near("turned 90: the thing's right points down the picture (z)", p.z, 1); near("turned 90: and not right (x)", p.x, 0);
    p = motion_in_scene(motion_deep(turned), float3{0.f}, right, down) * float4{0, 0, 1, 0};
    near("the plane's depth goes into the floor (y -1)", p.y, -1);
    p = motion_in_scene(motion_deep(doubled), float3{0.f}, right, down) * float4{0, 0, 1, 0};
    near("a layer twice the size is twice as deep", p.y, -2);
    // the third dimension: motion_test3d.json (motion_test3d.src.json through lottie3d.py)
    {
        Motion deep;
        if (!motion_open(deep, "motion_test3d.src.json")) { std::printf("FAIL open motion_test3d.json\n"); return 1; }
        Motion& m = deep;
#include "motion_test3d.inc"
        there3(m, "deep", 25, 0, 0, 0, 1, 2, 50);          // position keyed dimension by dimension: z 0 -> 200 over 100 frames
        there3(m, "vec", 50, 0, 0, 0, 50, 0, 150);         // position keyed as one, straight: halfway to (100, 0, 300)
        there3(m, "c3", 0, 0, 0, 0, 100, 0, -10);          // a child 10 along x under a parent at x 100 turned 90 about y: 10 towards the eye
        there3(m, "flat2d", 0, 0, 0, 0, 5, 6, 0);          // a flat layer in the same file
        there3(m, "under2d", 0, 0, 0, 0, 6, 8, 3);         // a 3D layer under that flat one
        there3(m, "az", 0, 0, 0, 0, 0, 0, -20);            // anchor 10 deep, twice as deep: its corner 20 towards the eye
        there3(m, "ori", 0, 1, 0, 0, 0, 0, -1);            // orientation 90 about y: its right points at the eye
        // a 3D layer that turns to face its way: it goes straight down the picture (and 40 towards the eye), so its right points down
        there3(m, "heads_down", 50, 1, 0, 0, 0, 51, -20);
        // cameras. One in front of the middle looking at it, zoom 1000 for a picture 1920 high: 2 atan(960 / 1000) degrees
        filament::math::mat4f stands;
        float view = 0.f;
        near("cam_front: there?", motion_camera(m, "cam_front", &stands, &view, 1920.f) ? 1 : 0, 1);
        near("cam_front: angle of view", view, 2.f * std::atan(0.96f) * 180.f / 3.14159265f, 0.01f);
        there3(m, "cam_front", 0, 0, 0, 1000, 540, 960, 0);     // what is 1000 before it is the middle of the composition
        there3(m, "cam_front", 0, 10, 20, 0, 550, 980, -1000);  // not turned: its right is the composition's right
        // one standing 1000 to the right of the middle, looking at the middle: forward is -x, its right is +z, its down is down
        there3(m, "cam_side", 0, 0, 0, 500, 1040, 960, 0);
        there3(m, "cam_side", 0, 100, 0, 0, 1540, 960, 100);
        there3(m, "cam_side", 0, 0, 50, 0, 1540, 1010, 0);
        motion_camera(m, "cam_side", &stands, &view, 1920.f);
        near("cam_side: angle of view (zoom 960: 90 degrees)", view, 90, 0.01f);
        // one that looks at no point, turned 90 about y: forward is +x
        there3(m, "cam_free", 0, 0, 0, 100, 100, 0, -500);
    }
    // what a moment costs the CPU: led.json (four layers, one with an expression), every frame of it once
    Motion led;
    if (motion_open(led, "led.src.json")) {
        const auto from = std::chrono::steady_clock::now();
        for (int f = 0; f < 480; ++f) {
            motion_at(led, (float)f);
            for (const char* name : {"runner", "rider", "hopper"}) motion_flat(led, name, &a);
        }
        std::printf("led.json: %.1f microseconds a moment (ThorVG working out all layers, three read)\n",
                    std::chrono::duration<double, std::micro>(std::chrono::steady_clock::now() - from).count() / 480.0);
    }
    std::printf(failed ? "%d FAILED\n" : "all as worked out by hand\n", failed);
    return failed ? 1 : 0;
}
