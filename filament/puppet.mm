// puppet.mm: films from files in the scene, and a green screen: a puppet show. Every .mp4 in out/vid is read as a
// film, its green keyed out, and stands before a wall (an After Effects film) as a puppet, each at its own place
// and depth; the first is also shown small as it is, green and all. A glass ring goes round before them.
// (puppet.sh makes one clip itself: an After Effects cat played on a green ground. Others are put there by hand.)
// PUPPETS=38: that many more of the first clip, each read on its own (how many can play at once). WebGPU backend only.
//   ./puppet.sh          ->  out/wgpu_kit/out/puppet.png       ./puppet.sh film  ->  out/wgpu_kit/out/puppet.mp4
#include "kit.h"
#include "drawn.h"
#include "video.h"

static const int W = 1080;
static const int H = 1920;

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    const float3 eye{0.f, 1.4f, 6.f};
    const float fov = 30.f;
    s.camera->lookAt(eye, {0.f, 1.4f, 0.f}, {0, 1, 0});
    s.camera->setProjection(fov, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.2f).saturation(1.1f).build(*s.engine));
    if (!room(s, "out/small_studio/small_studio_ibl.ktx", 60000.f)) return fail("out/small_studio/small_studio_ibl.ktx");
    const float dist = 6.6f;
    Drawn* wall = picture(s, "../../third_party/thorvg_example/res/lottie/gradient_background.json", mat4f::translation(float3{eye.x, eye.y, eye.z - dist}),
                          2.f * dist * std::tan(0.5f * fov * TAU / 360.f));
    if (!wall) return fail("picture (WebGPU backend only)");

    std::vector<std::string> clips;
    const std::string films = getenv("FILMS") ? getenv("FILMS") : "../vid";   // (FILMS=../vid_odd: another folder of films)
    if (FILE* found = popen(("ls " + films + "/* 2>/dev/null").c_str(), "r")) {
        char line[1024];
        while (fgets(line, sizeof line, found)) clips.push_back(std::string(line, strcspn(line, "\n")));
        pclose(found);
    }
    if (clips.empty()) return fail("no film in out/vid (puppet.sh makes one)");
    const bool many = getenv("PUPPETS") != nullptr;   // (then the puppets stand below, small)
    // where the puppets stand: middle and height of each (they come round again past the fourth)
    const float3 stand[] = {{-0.4f, 2.3f, 0.2f}, {0.3f, 1.5f, 0.5f}, {-0.25f, 0.62f, 0.8f}, {0.5f, 0.45f, 0.35f}};
    const bool rows = clips.size() > 4;   // (many films: in rows of three, small)
    std::vector<Video*> puppets;
    for (size_t i = 0; i < clips.size(); ++i) {
        const float3 at = many ? float3{-0.6f + 0.6f * (float)(i % 3), 0.38f, 0.4f} : rows ? float3{-0.56f + 0.56f * (float)(i % 3), 2.62f - 0.62f * (float)(i / 3), 0.3f + 0.01f * (float)i} : stand[i % 4];
        puppets.push_back(video(s, clips[i].c_str(), mat4f::translation(at), many ? 0.55f : rows ? (i == 5 ? 0.6f : 0.34f) : 0.95f));
        if (!puppets.back()) { std::printf("could not show %s\n", clips[i].c_str()); return fail("video"); }
        green(puppets.back());
    }
    Video* plain = many ? nullptr : video(s, clips[0].c_str(), mat4f::translation(float3{0.52f, 2.62f, 0.1f}), 0.42f);   // (as it is)
    Mesh* ring = torus(0.3f, 0.11f);
    if (!thick_glass(s, ring, {mat4f::translation(float3{0.f, many ? 1.6f : 1.2f, 1.3f})})) return fail("thick_glass.filamat");

    // PUPPETS=40: that many more of the clip, each read on its own, keyed, in rows over the wall (how many can play at once)
    std::vector<Video*> crowd;
    for (int i = 0, n = getenv("PUPPETS") ? atoi(getenv("PUPPETS")) : 0; i < n; ++i) {
        crowd.push_back(video(s, clips[(size_t)i % clips.size()].c_str(), mat4f::translation(float3{-0.78f + 0.39f * (float)(i % 5), 2.75f - 0.385f * (float)(i / 5), 0.5f + 0.002f * (float)i}), 0.42f));
        if (!crowd.back()) return fail("video");
        green(crowd.back());
    }
    const auto pose = [&](float t) {
        picture_at(s, wall, std::fmod(t * 2.f, 1.f) * wall->seconds);
        if (plain) video_at(s, plain, t * 8.f);
        for (size_t i = 0; i < puppets.size(); ++i) {
            video_at(s, puppets[i], t * 8.f);
            // (each sways a little, as a puppet on a stick does)
            move(s, puppets[i]->sheet, [&](int, const mat4f& at) { return mat4f::translation(float3{0.12f * std::sin(TAU * (t + 0.3f * (float)i)), 0.05f * std::sin(2.f * TAU * (t + 0.2f * (float)i)), 0.f}) * at * mat4f::rotation(0.12f * std::sin(TAU * (2.f * t + 0.25f * (float)i)), float3{0, 0, 1}); });
        }
        for (size_t i = 0; i < crowd.size(); ++i) video_at(s, crowd[i], t * 8.f + 0.13f * (float)i);
        move(s, ring, [&](int, const mat4f& at) { return mat4f::translation(float3{0.7f * std::sin(2.f * TAU * t), 0.f, 0.f}) * at * mat4f::rotation(2.f * TAU * t, normalize(float3{0.3f, 1, 0.2f})); });
    };
    pose(0.3f);
    const bool timed = argc > 1 && (!strcmp(argv[1], "time") || !strcmp(argv[1], "soak"));
    if (timed) video_patience = 0.f;   // (played as the clock goes: no film is waited for)
    video_asked = video_late = 0;
    if (timed) clock_after = [] {   // (of the times a film was asked for a moment in that stretch, how often its picture was not there yet)
        std::printf("    films late %ld of %ld times asked (%.2f %%)\n", video_late, video_asked, video_asked ? 100.0 * (double)video_late / (double)video_asked : 0.0);
        video_asked = video_late = 0;
    };
    const int done = show(s, argc, argv, crowd.empty() ? "puppet" : "puppet_crowd", 480, 2.f, pose);
    return done;
}
