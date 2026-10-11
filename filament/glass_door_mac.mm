// glass_door.h on macOS: Filament's own swap chain on a CVPixelBuffer (Metal), one per buffer. Filament says when the
// GPU has finished a frame (setFrameCompletedCallback); nothing is waited for.
#include "glass_door.h"

#include <backend/CallbackHandler.h>
#include <filament/Engine.h>
#include <filament/SwapChain.h>

#include <pthread.h>
#import <CoreVideo/CoreVideo.h>

using namespace filament;

namespace {
struct Now : backend::CallbackHandler {   // the completion callback as it comes, on Metal's own thread
    void post(void* user, Callback callback) override { callback(user); }
};
Now now;
CVPixelBufferRef buffer[GLASS_DOOR_BUFFERS];
SwapChain* chain[GLASS_DOOR_BUFFERS];
}  // namespace

Engine* glass_door_engine() {
    return Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
}

bool glass_door_open(Engine* engine, int w, int h, void (*finished)(int)) {
    NSDictionary* attrs = @{(id)kCVPixelBufferIOSurfacePropertiesKey : @{}, (id)kCVPixelBufferMetalCompatibilityKey : @YES};
    for (int i = 0; i < GLASS_DOOR_BUFFERS; ++i) {
        if (CVPixelBufferCreate(kCFAllocatorDefault, (size_t)w, (size_t)h, kCVPixelFormatType_32BGRA, (__bridge CFDictionaryRef)attrs, &buffer[i]) != kCVReturnSuccess)
            return false;
        chain[i] = engine->createSwapChain((void*)buffer[i], SwapChain::CONFIG_APPLE_CVPIXELBUFFER);
        chain[i]->setFrameCompletedCallback(&now, [i, finished](SwapChain*) { finished(i); });
    }
    // (a thread of default priority is woken late: asked for 16.7 ms it slept about 22)
    pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
    return true;
}

SwapChain* glass_door_chain(int i) { return chain[i]; }

void* glass_door_hand(int i) {
    return i >= 0 && i < GLASS_DOOR_BUFFERS && buffer[i] ? (void*)CVPixelBufferRetain(buffer[i]) : nullptr;
}
