// Forty-two films played by the clock, all put back at once to a moment inside their pictures (the playhead moved):
// how many times a film was late, and how many pictures were left out, each quarter second after.
//   check/seek_settle.sh
#include "../kit.h"
#include "../drawn.h"
#include "../video.h"

int main() {
    Stage s = open_stage(1920, 1080);
    s.camera->lookAt({0.f, 0.f, 6.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    std::vector<std::string> clips;
    for (const auto& found : std::filesystem::directory_iterator(getenv("FILMS") ? getenv("FILMS") : "../vid")) clips.push_back(found.path().string());
    std::sort(clips.begin(), clips.end());
    std::vector<Video*> films;
    for (int i = 0; i < 42; ++i) {
        Video* v = video(s, clips[(size_t)i % clips.size()].c_str(), mat4f::translation(float3{-3.f + 0.6f * (float)(i % 11), 1.5f - 0.75f * (float)(i / 11), 0.f}), 0.5f);
        if (!v) return fail(video_trouble);
        films.push_back(v);
    }
    s.by_clock = true;
    const auto start = std::chrono::steady_clock::now();
    long late = video_late, left = 0;
    double t = 0.0;
    for (int i = 0; i < 60 * 6; ++i) {
        std::this_thread::sleep_until(start + std::chrono::microseconds((long long)i * 1000000 / 60));
        if (i == 180) {
            t = 1.37;   // (all put back at once, to a moment between their key pictures)
            std::printf("put back to %.2f s:\n", t);
        }
        time_is(s, t);
        if (begins(s)) pictures_of(s), s.renderer->endFrame();
        else ++left;
        t += 1.0 / 60.0;
        if (i >= 180 && (i - 180) % 15 == 14) {
            std::printf("  %.2f s after: films late %ld times, pictures left out %ld\n", (double)(i - 179) / 60.0, video_late - late, left);
            late = video_late, left = 0;
        }
    }
    s.engine->flushAndWait();
    return 0;
}
