// Two squares, the upper laid on the lower each way: the colour where they meet, as drawn (compare with the W3C compositing formulas).
//   clang++ -std=c++20 -fobjc-arc -I. check/blend_ways.mm -Lout -lmotolii -Wl,-rpath,$PWD/out -framework Foundation -framework CoreVideo -o out/blend_ways; (cd out/wgpu_kit && ../blend_ways /tmp/b.motolii)
#include "motolii.h"
#import <CoreVideo/CoreVideo.h>
#include <cstdio>
#include <unistd.h>
static void at(void* p, int x, int y, int* rgb) {
    CVPixelBufferRef b = (CVPixelBufferRef)p;
    CVPixelBufferLockBaseAddress(b, kCVPixelBufferLock_ReadOnly);
    const uint8_t* px = (const uint8_t*)CVPixelBufferGetBaseAddress(b) + y * CVPixelBufferGetBytesPerRow(b) + x * 4;
    rgb[0] = px[2], rgb[1] = px[1], rgb[2] = px[0];
    CVPixelBufferUnlockBaseAddress(b, kCVPixelBufferLock_ReadOnly);
    CVPixelBufferRelease(b);
}
int main(int argc, char** argv) {
    Motolii* m = motolii_open(argv[1], 640, 360);
    if (!m || !motolii_live(m)) return 1;
    const int under = motolii_add(m, "shape", "rect"), over = motolii_add(m, "shape", "rect");
    motolii_say(m, under, "fill", "cc4d33"), motolii_set(m, under, "x", -0.3);
    motolii_say(m, over, "fill", "3380e6"), motolii_set(m, over, "x", 0.3);
    const char* ways[] = {"normal", "multiply", "screen", "darken", "lighten", "overlay", "soft_light", "hard_light", "color_dodge", "color_burn", "difference", "exclusion", "hue", "saturation", "color", "luminosity"};
    for (const char* w : ways) {
        printf("%-12s said %d ", w, motolii_say(m, over, "blend", w));
        usleep(300000);
        int c[3];
        at(motolii_front(m), 320, 180, c);
        printf("meet %3d %3d %3d\n", c[0], c[1], c[2]);
    }
    motolii_close(m);
}
