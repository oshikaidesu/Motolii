// neko.mm: the cat populace. Every film in out/vid (green-screen clips of cats: two real meme clips put there by hand,
// one made by puppet.sh) is read many times over, each on its own, keyed, and stood in rising rows like a crowd on
// terraces: 46 of them, placed by one seed. Before the crowd two great heads rise; behind and above it one great cat
// is seen only through the shape of a turning gem (a matte: the solid cuts the film); confetti (an After Effects
// film, its white keyed out) bursts before all, twice over; glass goes round through it; the eye sways.
// 51 films at once (the machine's decoder reads at most 59). WebGPU backend only.
//   ./neko.sh          ->  out/wgpu_kit/out/neko.png       ./neko.sh film  ->  out/wgpu_kit/out/neko.mp4
#include "kit.h"
#include "drawn.h"
#include "video.h"

static const int W = 1080;
static const int H = 1920;

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    const float3 look{0.f, 1.5f, 0.f};
    const float fov = 30.f;
    s.camera->lookAt(look + float3{0.f, 0.f, 6.f}, look, {0, 1, 0});
    s.camera->setProjection(fov, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.25f).saturation(1.2f).build(*s.engine));
    if (!room(s, "out/small_studio/small_studio_ibl.ktx", 60000.f)) return fail("out/small_studio/small_studio_ibl.ktx");
    const std::string ae = "../../third_party/thorvg_example/res/lottie/";
    Drawn* wall = picture(s, (ae + "gradient_background.json").c_str(), mat4f::translation(float3{0.f, 2.2f, -5.5f}), 12.f);
    if (!wall) return fail("picture (WebGPU backend only)");

    std::vector<std::string> clips;
    if (FILE* found = popen("ls ../vid/* 2>/dev/null", "r")) {
        char line[1024];
        while (fgets(line, sizeof line, found)) clips.push_back(std::string(line, strcspn(line, "\n")));
        pclose(found);
    }
    if (clips.empty()) return fail("no film in out/vid (puppet.sh makes one)");

    // the crowd: rows that rise and go back, more to a row the further back; who stands where, how tall, and when
    // each one's film starts are drawn from one seed
    struct One { Video* film; float starts, sways; };
    std::vector<One> crowd;
    uint32_t seed = 20261013u;
    const auto chance = [&] { seed = seed * 1664525u + 1013904223u; return (float)(seed >> 8) / 16777216.f; };
    for (int row = 0; row < 6; ++row) {
        const int here = 5 + row + (row > 3);   // (5, 6, 7, 8, 10, 10 = 46)
        const float back = 1.3f - 0.8f * (float)row, up = 0.2f + 0.46f * (float)row;
        const float wide = 1.25f * (6.f - back) * std::tan(0.5f * fov * TAU / 360.f) * 2.f * (float)W / (float)H;   // (a little wider than the eye sees there)
        for (int i = 0; i < here; ++i) {
            const float x = wide * (((float)i + 0.5f + 0.5f * (chance() - 0.5f)) / (float)here - 0.5f);
            Video* v = video(s, clips[(size_t)(chance() * 2.999f) % clips.size()].c_str(),
                             mat4f::translation(float3{x, up + 0.12f * (chance() - 0.5f), back + 0.25f * (chance() - 0.5f)}), 0.62f + 0.3f * chance());
            if (!v) return fail("video");
            green(v);
            crowd.push_back({v, 10.f * chance(), chance()});
        }
    }
    // two great heads before the crowd, and one great cat behind and above it, seen through a gem's shape
    for (int side : {-1, 1}) {
        Video* head = video(s, clips[clips.size() > 2 ? 2 : 0].c_str(), mat4f::translation(float3{0.62f * (float)side, -0.05f, 2.6f}), 1.5f);
        if (!head) return fail("video");
        green(head);
        crowd.push_back({head, 1.7f * (float)(side + 1), 0.5f + 0.2f * (float)side});
    }
    const mat4f high = mat4f::translation(float3{0.f, 3.75f, -3.9f});
    Video* great = video(s, clips[clips.size() > 1 ? 1 : 0].c_str(), high, 6.f);
    Mesh* window = gem(1.6f);
    if (!great) return fail("video");
    green(great);
    const int byGem = matte(s, window, {high});
    if (byGem < 0 || !matted(s, great->look, byGem, true, 3.f)) return fail("matte.filamat");

    // confetti before all, twice over: an After Effects film, its white ground keyed out as a film's green is
    // (0.3 ms each; balloons_with_string.json cost 5 ms and confetti2.json 3: many shapes, drawn over the whole screen)
    std::vector<Drawn*> party;
    for (int twice = 0; twice < 2; ++twice) {
        const char* name = "confetti.json";
        Drawn* d = picture(s, (ae + name).c_str(), mat4f::translation(float3{twice ? 0.25f : -0.1f, twice ? 1.9f : 1.3f, 3.1f + 0.5f * (float)twice}) * mat4f::rotation(twice ? 2.6f : 0.f, float3{0, 0, 1}), twice ? 1.9f : 1.6f);
        if (!d) return fail(name);
        d->look->setParameter("key", float4{1.f, 1.f, 1.f, 0.035f});
        d->look->setParameter("keySoft", float2{0.05f, 0.f});
        party.push_back(d);
    }
    struct Glass { Mesh* mesh; float round, tilt, height, far; };
    const Glass glasses[] = {{torus(0.34f, 0.12f), 1.f, 0.3f, 1.15f, 1.5f}, {gem(0.3f), -1.f, 0.8f, 2.2f, 1.1f}, {torus(0.22f, 0.08f), 2.f, 1.4f, 1.75f, 1.9f}};
    for (const Glass& g : glasses)
        if (!thick_glass(s, g.mesh, {mat4f{}})) return fail("thick_glass.filamat");

    const auto pose = [&](float t) {
        // the eye sways from side to side and breathes in and out
        const float round = 0.3f * std::sin(TAU * t), near = 6.f - 0.5f * std::sin(2.f * TAU * t);
        s.camera->lookAt(look + near * float3{std::sin(round), 0.1f * std::sin(TAU * t + 1.f), std::cos(round)}, look, {0, 1, 0});
        picture_at(s, wall, std::fmod(t * 2.f, 1.f) * wall->seconds);
        for (size_t i = 0; i < party.size(); ++i) picture_at(s, party[i], std::fmod(t * std::max(1.f, std::round(8.f / party[i]->seconds)) + 0.5f * (float)i, 1.f) * party[i]->seconds);
        for (const One& one : crowd) {
            video_at(s, one.film, t * 8.f + one.starts);
            // (each jumps to its own beat and leans as it lands)
            move(s, one.film->sheet, [&](int, const mat4f& at) {
                const float beat = TAU * (4.f * t + one.sways);
                return mat4f::translation(float3{0.f, 0.07f * std::fabs(std::sin(beat)), 0.f}) * at * mat4f::rotation(0.1f * std::sin(beat), float3{0, 0, 1});
            });
        }
        video_at(s, great, t * 8.f);
        move(s, window, [&](int, const mat4f& at) { return at * mat4f::rotation(TAU * t, normalize(float3{0.3f, 1, 0.1f})); });
        for (const Glass& g : glasses)
            move(s, g.mesh, [&](int, const mat4f&) {
                const float a = TAU * t * g.round + g.tilt * 3.f;
                return mat4f::translation(float3{0.85f * std::sin(a), g.height + 0.3f * std::sin(2.f * a + g.tilt), g.far + 0.5f * std::cos(a)}) * mat4f::rotation(2.f * a, normalize(float3{g.tilt, 1, 0.3f}));
            });
    };
    pose(0.3f);
    const bool timed = argc > 1 && (!strcmp(argv[1], "time") || !strcmp(argv[1], "soak"));
    if (timed) video_patience = 0.f;   // (played as the clock goes: no film is waited for)
    video_asked = video_late = 0;
    if (timed) clock_after = [] {
        std::printf("    films late %ld of %ld times asked (%.2f %%)\n", video_late, video_asked, video_asked ? 100.0 * (double)video_late / (double)video_asked : 0.0);
        video_asked = video_late = 0;
    };
    return show(s, argc, argv, "neko", 480, 2.f, pose);
}
