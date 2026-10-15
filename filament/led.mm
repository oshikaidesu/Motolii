// led.mm: glass things on paper, led by After Effects' motion: where each thing is at a moment, and where the eye
// is, is read from the layer of its name in led.json (a Lottie file: a flat layer on a curved path with easing,
// turning to face its way; 3D layers: a child riding on a turning parent and rising off the paper, a thing whose two
// keys are looped for ever by an expression, a thing flying arches over the paper (a path curved in space); a camera drifting
// and zooming). ThorVG works the motion out (motion.h;
// the file goes to it as it is); nothing of the motion is written here.
//   ./led.sh  ->  out/wgpu_kit/out/led.png        ./led.sh film  ->  out/wgpu_kit/out/led.mp4
#include "kit.h"
#include "drawn.h"
#include "motion.h"

static const int W = 1080;
static const int H = 1920;

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.4f).saturation(1.15f).build(*s.engine));
    Sunlit lit;
    lit.sunWay = normalize(float3{-0.12f, -0.50f, 0.86f});
    if (!paper_and_sun(s, lit)) return fail("paper and sun (paper.filamat, floor_shade.filamat, out/lightroom)");

    Motion led;
    const bool card = getenv("CARD") != nullptr;   // (and a 3D shape layer riding the flyer, its own picture where it is: led_card.json)
    if (!motion_open(led, card ? "led_card.json" : "led.json")) return fail("led.json");
    // the composition lies on the paper: its 1080 x 1920 pixels are about 4.1 x 7.3 of the paper, its top away from the eye
    const float pixel = 0.0038f;
    const float3 corner{-0.5f * led.w * pixel, 0.f, 0.55f - 0.5f * led.h * pixel}, right{pixel, 0.f, 0.f}, down{0.f, 0.f, pixel};
    // a thing as it stands on the paper, in the layer's own axes (x right, y down the picture, z into the paper):
    // stood up (its own up is the plane's -z), lifted by `high`, and for the runner turned so that it rolls along its way
    const auto standing = [](float high, float turned) { return mat4f::translation(float3{0.f, 0.f, -high}) * mat4f::rotation(-0.25f * TAU, float3{1, 0, 0}) * mat4f::rotation(turned, float3{0, 1, 0}); };
    struct Thing { const char* layer; Mesh* mesh; mat4f stands; };
    const Thing things[] = {
        {"runner", torus(0.45f, 0.17f), standing(0.62f, 0.25f * TAU)},
        {"rider", gem(0.5f), standing(0.43f, 0.f)},
        {"hopper", gem(0.36f), standing(0.31f, 0.4f)},
        {"flyer", gem(0.42f), standing(0.36f, 0.2f)},
    };
    for (const Thing& t : things) {
        if (!thick_glass(s, t.mesh, {mat4f{}})) return fail("thick_glass.filamat");
        if (!shadow_of(s, lit, t.mesh, {mat4f{}})) return fail("floor_shade.filamat");
        if (!glass_light(s, lit, t.mesh, {mat4f{}})) return fail("glasslight.filamat");
    }
    float4 box{0.f};
    Drawn* cardSheet = card ? layer_picture(s, "led_card.json", "card", pixel, &box) : nullptr;
    if (card && !cardSheet) return fail("the card layer of led_card.json");
    const auto pose = [&](float t) {
        motion_at(led, t * led.frames);
        mat4f eye;
        float view = 30.f;
        if (motion_camera(led, "eye", &eye, &view, led.h)) {   // (Filament's camera: x right, y up, looking along -z)
            s.camera->setModelMatrix(motion_in_scene(eye, corner, right, down) * mat4f::scaling(float3{1, -1, -1}));
            s.camera->setProjection(view, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
        }
        for (const Thing& thing : things) {
            mat4f at;
            float seen = 0.f;
            const bool there = motion_of(led, thing.layer, &at, &seen) && seen > 0.f;
            // (a layer that is not there, or is wholly clear, is a thing of no size: nothing of it is drawn. A half-clear layer is drawn whole: the glass has no opacity of its own)
            move(s, thing.mesh, [&](int, const mat4f&) { return there ? motion_in_scene(at, corner, right, down) * thing.stands : mat4f::scaling(0.f); });
        }
        if (cardSheet) {   // (its sheet's middle where its picture's middle is in the layer's space; the sheet's up is the layer's -y)
            mat4f at;
            layer_at(s, cardSheet, t * led.frames);
            const bool there = motion_of(led, "card", &at);
            move(s, cardSheet->sheet, [&](int, const mat4f&) {
                return there ? motion_in_scene(at, corner, right, down) * mat4f::translation(float3{(box.x + 0.5f * box.z) * pixel, (box.y + 0.5f * box.w) * pixel, 0.f}) * mat4f::scaling(float3{1, -1, 1})
                             : mat4f::scaling(0.f);
            });
        }
    };
    pose(0.f);
    return show(s, argc, argv, "led", 480, pose);
}
