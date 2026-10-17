// Each look card's value changed: does the stage's picture change, and the card's solo picture of the thing? (a change
// that reaches one and not the other is what the hand sees as "the preview does not follow" or "the stage does not")
//   clang++ -std=c++20 -fobjc-arc -I. check/solo_follows.mm -Lout -lmotolii -Wl,-rpath,$PWD/out -framework Foundation -framework CoreVideo -o out/solo_follows
//   (cd out/wgpu_kit && ../solo_follows /tmp/f.motolii) ; ... hold   (set: as a value is set; hold: as a hand carries it)
#include "motolii.h"
#import <CoreVideo/CoreVideo.h>
#include <cstdio>
#include <cstring>
#include <string>
#include <unistd.h>
#include <vector>
static std::vector<uint8_t> grab(void* p) {
    CVPixelBufferRef b = (CVPixelBufferRef)p;
    if (!b) return {};
    CVPixelBufferLockBaseAddress(b, kCVPixelBufferLock_ReadOnly);
    const uint8_t* at = (const uint8_t*)CVPixelBufferGetBaseAddress(b);
    std::vector<uint8_t> v(at, at + CVPixelBufferGetBytesPerRow(b) * CVPixelBufferGetHeight(b));
    CVPixelBufferUnlockBaseAddress(b, kCVPixelBufferLock_ReadOnly);
    CVPixelBufferRelease(b);
    return v;
}
static double differ(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b) {
    if (a.size() != b.size() || a.empty()) return -1;
    size_t n = 0;
    for (size_t i = 0; i < a.size(); i += 4) n += abs(a[i] - b[i]) + abs(a[i + 1] - b[i + 1]) + abs(a[i + 2] - b[i + 2]) > 12;
    return 100.0 * n / (a.size() / 4);
}
int main(int argc, char** argv) {
    Motolii* m = motolii_open(argv[1], 1920, 1080);
    if (!m || !motolii_live(m)) return printf("not live\n"), 1;
    struct Case { const char *card, *kind, *shape, *name, *word; double number; const char* fill = "ff3366"; bool stroke = false; };
    const Case cases[] = {
        {"Gradient", "shape", "rect", "fill", "ff3366 3366ff", 0}, {"Gradient angle", "shape", "rect", "gradient_angle", nullptr, 90, "ff3366 3366ff"},
        {"Gradient radial", "shape", "rect", "gradient", "radial", 0, "ff3366 3366ff"},
        {"Line dash", "shape", "rect", "dash", nullptr, 20, "ff3366", true}, {"Path zigzag", "shape", "rect", "zigzag", nullptr, 0.3},
        {"Path round", "shape", "rect", "round", nullptr, 0.5}, {"Deform bend (shape)", "shape", "rect", "bend", nullptr, 60},
        {"Deform twist (shape)", "shape", "rect", "twist", nullptr, 90}, {"Surface metallic", "solid", "cube", "metallic", nullptr, 1},
        {"Surface roughness", "solid", "cube", "roughness", nullptr, 0.05}, {"Glass on", "solid", "cube", "fill", "glass", 0},
        {"Shine", "shape", "rect", "emissive", nullptr, 4}, {"Trim Paths", "shape", "rect", "trim", nullptr, 0.5, "ff3366", true},
        {"Gaussian Blur", "shape", "rect", "blur", nullptr, 30}, {"Drop Shadow", "shape", "rect", "shadow", "000000", 0},
        {"Glow", "shape", "rect", "glow", "ffd166", 0}, {"Duotone", "shape", "rect", "tint", nullptr, 1},
        {"Gradient Map", "shape", "rect", "tritone", nullptr, 1}, {"Type spacing", "text", "/System/Library/Fonts/Supplemental/Arial.ttf", "letter_spacing", nullptr, 3},
    };
    for (const Case& c : cases) {
        const int t = motolii_add(m, c.kind, c.shape);
        if (strcmp(c.kind, "solid")) motolii_say(m, t, "fill", c.fill);
        if (c.stroke) motolii_say(m, t, "stroke", "ffffff"), motolii_set(m, t, "stroke_width", 12);
        if (!strcmp(c.kind, "text")) motolii_say(m, t, "text", "Motolii");
        motolii_solo(m, t, 292);
        usleep(400000);
        const auto stage0 = grab(motolii_front(m)), solo0 = grab(motolii_solo_front(m));
        const bool held = argc > 2 && !strcmp(argv[2], "hold");   // (as the window carries a number under a hand: held, carried, not yet let go)
        if (held && c.word) { motolii_remove(m, t); continue; }
        if (held) motolii_hold(m, t), motolii_carry(m, c.name, c.number, 0);
        else if (c.word) motolii_say(m, t, c.name, c.word); else motolii_set(m, t, c.name, c.number);
        double s = -1, o = -1; int when = -1;
        for (int k = 1; k <= 20; k++) {   // (watched for a second, a look every 50 ms)
            usleep(50000);
            s = differ(stage0, grab(motolii_front(m))), o = differ(solo0, grab(motolii_solo_front(m)));
            if (o > 0.05 && when < 0) when = k * 50;
        }
        printf("%-22s stage %6.2f%%  solo %6.2f%%  solo followed %s\n", c.card, s, o, when < 0 ? "never" : (std::to_string(when) + " ms").c_str());
        if (held) motolii_let_go(m, 0);
        motolii_remove(m, t);
    }
    motolii_close(m);
}
