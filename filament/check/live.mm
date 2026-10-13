// (a check, not a scene: the live picture without any shell. A document is opened and made live; it plays for two
//  seconds and the pictures finished are counted; then it stands at a moment, and the picture in the shared buffer is
//  set beside the still the inside writes of the same moment, number by number)   live_check <document> <out folder>
#include "../motolii.h"
#import <CoreVideo/CoreVideo.h>
#import <Foundation/Foundation.h>
#import <ImageIO/ImageIO.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#include <string>
#include <unistd.h>
int main(int argc, char** argv) {
    if (argc < 3) return printf("live_check <document> <out folder>\n"), 1;
    Motolii* m = motolii_open(argv[1], 1080, 1920);
    if (!m || !motolii_live(m)) return printf("not live\n"), 1;
    motolii_play(m, 1);
    usleep(500000);   // (films settle)
    const unsigned long long before = motolii_frames(m);
    usleep(2000000);
    printf("playing: %llu pictures finished in 2 seconds; the stage's moment %.2f\n", motolii_frames(m) - before, motolii_now(m));
    motolii_play(m, 0);
    motolii_at(m, 1.5);
    usleep(300000);
    const unsigned long long stood = motolii_frames(m);
    usleep(1000000);
    printf("standing: %llu pictures finished in the next second (none are asked for)\n", motolii_frames(m) - stood);
    CVPixelBufferRef front = (CVPixelBufferRef)motolii_front(m);
    if (!front) return printf("no picture\n"), 1;
    CVPixelBufferLockBaseAddress(front, kCVPixelBufferLock_ReadOnly);
    CGColorSpaceRef rgb = CGColorSpaceCreateDeviceRGB();
    CGContextRef seen = CGBitmapContextCreate(CVPixelBufferGetBaseAddress(front), CVPixelBufferGetWidth(front), CVPixelBufferGetHeight(front), 8, CVPixelBufferGetBytesPerRow(front), rgb,
                                              kCGBitmapByteOrder32Little | kCGImageAlphaNoneSkipFirst);
    CGImageRef image = CGBitmapContextCreateImage(seen);
    const std::string out = std::string(argv[2]) + "/live_front.png", still = std::string(argv[2]) + "/live_still.png";
    CGImageDestinationRef file = CGImageDestinationCreateWithURL((__bridge CFURLRef)[NSURL fileURLWithPath:@(out.c_str())], (__bridge CFStringRef)UTTypePNG.identifier, 1, nullptr);
    CGImageDestinationAddImage(file, image, nullptr);
    printf("the shared buffer written out: %s\n", CGImageDestinationFinalize(file) ? "yes" : "NO");
    CVPixelBufferUnlockBaseAddress(front, kCVPixelBufferLock_ReadOnly);
    CVPixelBufferRelease(front);
    printf("the inside's own still of that moment: %s\n", motolii_still(m, still.c_str()) ? "yes" : motolii_trouble(m));
    motolii_close(m);
    return 0;
}
