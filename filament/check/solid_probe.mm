// (a check, not a scene: 3D files on the stage with the other kinds of thing: a wall that is an After Effects film
//  behind, a glass ring before)   solid_probe <out.png> <seconds> <file.glb>...
#include "../kit.h"
#include "../drawn.h"
#include "../solid.h"
int main(int argc, char** argv) {
    if (argc < 4) return fail("solid_probe <out.png> <seconds> <file.glb>...");
    Stage s = open_stage(1080, 1920);
    s.camera->lookAt({0.f, 0.f, 6.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    s.camera->setProjection(30.0, 1080.0 / 1920.0, 0.05, 60.0, Camera::Fov::VERTICAL);
    if (!room(s, "out/small_studio/small_studio_ibl.ktx", 60000.f)) return fail("room");
    if (!picture(s, "../../third_party/thorvg_example/res/lottie/gradient_background.json", mat4f::translation(float3{0.f, 0.f, -2.f}), 5.f)) return fail("picture");
    for (int i = 3; i < argc; ++i) {
        const auto t0 = std::chrono::steady_clock::now();
        Solid* d = solid(s, argv[i], mat4f::translation(float3{0.f, 1.f - 1.1f * (float)(i - 3), 0.f}) * mat4f::rotation(0.6f, float3{0, 1, 0}), 1.f);
        if (!d) return fail("the file could not be read");
        std::printf("%s: on the stage in %.1f ms, its own movement %.2f s\n", argv[i], std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count(), d->seconds);
    }
    if (!thick_glass(s, torus(0.37f, 0.13f), {mat4f::translation(float3{0.3f, -0.1f, 1.5f})})) return fail("thick_glass.filamat");
    time_is(s, atof(argv[2]));
    if (argc > 4 && getenv("TIME")) clock(s, 120, [](float) {});
    return shoot(s, argv[1]) ? 0 : fail("png");
}
