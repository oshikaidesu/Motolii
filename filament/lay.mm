// lay.mm: the ways a sheet lies on what is behind it (Filament's own blending): over it, add, multiply, screen, from
// left to right. Above: a film with its green keyed out. Below: an After Effects film on its white ground.
//   ./lay.sh          ->  out/wgpu_kit/out/lay.png       ./lay.sh film  ->  out/wgpu_kit/out/lay.mp4
#include "kit.h"
#include "drawn.h"
#include "video.h"

int main(int argc, char** argv) {
    Stage s = open_stage(1080, 1920);
    const float3 eye{0.f, 1.4f, 6.f};
    s.camera->lookAt(eye, {0.f, 1.4f, 0.f}, {0, 1, 0});
    s.camera->setProjection(30.0, 1080.0 / 1920.0, 0.05, 60.0, Camera::Fov::VERTICAL);
    const std::string ae = "../../third_party/thorvg_example/res/lottie/";
    if (!picture(s, (ae + "gradient_background.json").c_str(), mat4f::translation(float3{0.f, 1.4f, -0.6f}), 3.6f)) return fail("picture (WebGPU backend only)");
    const char* const ways[] = {nullptr, "add", "multiply", "screen"};
    for (int i = 0; i < 4; ++i) {
        const float x = -0.66f + 0.44f * (float)i;
        Video* cat = video(s, "../vid/happy.mp4", mat4f::translation(float3{x, 2.f, 0.f}), 0.7f, 0.f, -1.f, 1.f, ways[i]);
        if (!cat) return fail(video_trouble);
        key_out(cat);
        if (!picture(s, (ae + "confetti.json").c_str(), mat4f::translation(float3{x, 0.9f, 0.f}), 0.42f, float3{1.f}, true, ways[i])) return fail("picture");
    }
    return show(s, argc, argv, "lay", 240, [](float) {});
}
