// The C face of the live glass (see glass_live.h): start the drawing thread, set a number, hand out the last frame.
// Everything here may be called from any thread of the host.
#include "glass_live.h"

#import <CoreVideo/CoreVideo.h>

#include <cstdlib>
#include <thread>

extern "C" {

// Settings that are read once at start (the same names as glass_scene's environment: ONLY, COUNT, TUMBLE, EXNEW ...).
__attribute__((visibility("default"))) void glass_env(const char* name, const char* value) { setenv(name, value, 1); }

// base: the folder that holds the compiled materials and out/studio. Returns 0, or 1 if it existed already and was only
// told to go on.
__attribute__((visibility("default"))) int glass_start(const char* base, int w, int h) {
    if (g_live.alive.exchange(1)) {   // already there: go on drawing (the size and the settings stay as they were)
        g_live.run = 1;
        return 1;
    }
    g_live.base = base;
    if (!g_live.base.empty() && g_live.base.back() != '/') g_live.base += '/';
    g_live.w = w;
    g_live.h = h;
    g_live.run = 1;
    std::thread([] {
        char name[] = "live";
        char* argv[] = {name, name, nullptr};
        glass_scene_main(2, argv);
        g_live.front = nullptr;
        g_live.alive = 0;
    }).detach();
    return 0;
}
// Pauses the drawing. The thread and the engine stay; glass_start goes on.
__attribute__((visibility("default"))) void glass_stop(void) { g_live.run = 0; }
__attribute__((visibility("default"))) int glass_alive(void) { return g_live.alive; }

// 0 sun round, 1 sun down, 2 camera round, 3 camera up, 4 camera distance, 5 speed
__attribute__((visibility("default"))) void glass_set(int what, float v) {
    std::atomic<float>* slot[] = {&g_live.sunAz, &g_live.sunEl, &g_live.camAz, &g_live.camEl, &g_live.camDist, &g_live.speed};
    if (what >= 0 && what < 6) *slot[what] = v;
}
__attribute__((visibility("default"))) float glass_get(int what) {
    std::atomic<float>* slot[] = {&g_live.sunAz, &g_live.sunEl, &g_live.camAz, &g_live.camEl, &g_live.camDist, &g_live.speed};
    // 6: milliseconds the last frame took (CPU start to GPU finished), 7: ticks skipped so far
    return what >= 0 && what < 6 ? slot[what]->load() : (what == 6 ? g_live.frameMs.load() : (what == 7 ? (float)g_live.skipped.load() : 0.f));
}
__attribute__((visibility("default"))) unsigned long long glass_frames(void) { return g_live.frames; }

// The last finished frame, retained for the caller (who releases it), or NULL before the first one.
__attribute__((visibility("default"))) void* glass_front(void) {
    CVPixelBufferRef pb = (CVPixelBufferRef)g_live.front.load();
    return pb ? (void*)CVPixelBufferRetain(pb) : nullptr;
}

}  // extern "C"
