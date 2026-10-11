// glass_scene.mm as a library that draws live into pixel buffers (built with -DGLASS_LIB, see build_live.sh).
// What the host and the drawing thread share: plain numbers, written by the host at any time, read once a frame.
#pragma once
#include <atomic>
#include <cstdint>
#include <string>

struct GlassLive {
    std::string base;                       // the folder of the .filamat files and out/studio, with a slash at the end
    int w = 1280, h = 720;                  // size of the picture, fixed at start
    std::atomic<int> run{0};                // 1 while the thread should draw (0: paused; the thread and the engine stay)
    std::atomic<int> alive{0};              // 1 while the thread exists
    std::atomic<int> front{-1};             // which buffer holds the last finished frame (glass_door.h; -1: none yet)
    std::atomic<uint64_t> frames{0};        // finished frames
    std::atomic<float> frameMs{0};          // the last finished frame: from the start of its CPU part to the GPU having finished it
    std::atomic<uint64_t> skipped{0};       // ticks on which no frame was drawn (no free buffer, or Filament asked to skip)
    // what can be changed while it runs
    std::atomic<float> sunAz{0.2347f};      // the sun: where it travels to, round the vertical (radians; 0.2347 = glass_scene's own)
    std::atomic<float> sunEl{0.4179f};      // how steeply it comes down (radians)
    std::atomic<float> camAz{0.5828f};      // the camera round the point it looks at (radians)
    std::atomic<float> camEl{0.5033f};      // above the floor (radians)
    std::atomic<float> camDist{3.0075f};     // how far away
    std::atomic<float> speed{0.5f};         // how fast the solids turn (0: still)
};
inline GlassLive g_live;

int glass_scene_main(int argc, char** argv);
