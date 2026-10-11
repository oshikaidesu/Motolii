// Desk check of out/libglass_live.dylib without any host app: start it, let it run, look at what is in the pixel buffer
// (read back here only to check; a host shows the buffer as it is), move the camera and the sun, stop.
#import <CoreVideo/CoreVideo.h>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <thread>
#include <vector>
#include <unistd.h>

extern "C" {
void glass_env(const char*, const char*);
int glass_start(const char*, int, int);
void glass_stop(void);
int glass_alive(void);
void glass_set(int, float);
float glass_get(int);
unsigned long long glass_frames(void);
void* glass_front(void);
}

static bool dump(const char* path, double* mean) {
    CVPixelBufferRef pb = (CVPixelBufferRef)glass_front();
    if (!pb) return false;
    CVPixelBufferLockBaseAddress(pb, kCVPixelBufferLock_ReadOnly);
    const size_t w = CVPixelBufferGetWidth(pb), h = CVPixelBufferGetHeight(pb), row = CVPixelBufferGetBytesPerRow(pb);
    const uint8_t* base = (const uint8_t*)CVPixelBufferGetBaseAddress(pb);
    FILE* f = std::fopen(path, "wb");
    std::fprintf(f, "P6\n%zu %zu\n255\n", w, h);
    double sum = 0;
    for (size_t y = 0; y < h; ++y)
        for (size_t x = 0; x < w; ++x) {
            const uint8_t* p = base + y * row + x * 4;   // BGRA
            const uint8_t rgb[3] = {p[2], p[1], p[0]};
            std::fwrite(rgb, 1, 3, f);
            sum += rgb[0] + rgb[1] + rgb[2];
        }
    std::fclose(f);
    *mean = sum / (double)(w * h * 3);
    std::printf("wrote %s (%zux%zu, IOSurface-backed: %s, mean %.1f)\n", path, w, h, CVPixelBufferGetIOSurface(pb) ? "yes" : "NO", *mean);
    CVPixelBufferUnlockBaseAddress(pb, kCVPixelBufferLock_ReadOnly);
    CVPixelBufferRelease(pb);
    return true;
}

int main(int argc, char** argv) {
    const char* base = argc > 1 ? argv[1] : ".";
    glass_env("ONLY", "cube,slab,prism,pillar,hexagon,bar,pyramid,wedge,octahedron,ball,star,washer,letterA,letterB,letterO,letterM,letterR");
    glass_env("COUNT", "30");
    glass_env("EXNEW", "1");
    if (glass_start(base, 1280, 720)) return 1;
    auto wait = [](double s) { std::this_thread::sleep_for(std::chrono::duration<double>(s)); };
    wait(2.0);
    const unsigned long long f0 = glass_frames();
    wait(3.0);
    const unsigned long long f1 = glass_frames();
    std::printf("frames in 3 s: %llu (%.1f fps), last frame %.2f ms\n", f1 - f0, (f1 - f0) / 3.0, glass_get(6));
    double m0 = 0, m1 = 0;
    bool ok = dump("out/glass-live-a.ppm", &m0);
    glass_set(2, 2.2f);    // the camera round to the other side
    glass_set(3, 0.9f);    // and higher
    glass_set(0, 2.0f);    // the sun from another side
    glass_set(5, 0.f);     // stand still
    wait(1.0);
    ok = dump("out/glass-live-b.ppm", &m1) && ok;
    glass_stop();   // a pause
    wait(0.3);
    const unsigned long long p0 = glass_frames();
    wait(0.5);
    const unsigned long long p1 = glass_frames();
    glass_start(base, 1280, 720);   // go on
    wait(0.5);
    const unsigned long long p2 = glass_frames();
    std::printf("paused: %llu frames in 0.5 s; going on: %llu frames in 0.5 s; ticks skipped in all: %.0f\n", p1 - p0, p2 - p1, glass_get(7));
    const bool pass = ok && f1 - f0 > 170 && m0 > 20 && m1 > 20 && m0 != m1 && p1 == p0 && p2 - p1 > 20;
    std::printf("%s\n", pass ? "PASS" : "FAIL");
    std::fflush(stdout);
    _exit(pass ? 0 : 1);   // (the drawing thread lives as long as the process)
}
