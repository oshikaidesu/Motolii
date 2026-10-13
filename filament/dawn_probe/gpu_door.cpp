// See gpu_door.h.
#include "gpu_door.h"

#include <filament/Engine.h>

#include <backend/platforms/WebGPUPlatform.h>

#include "details/Engine.h"

namespace door {
using namespace filament;

static backend::WebGPUPlatform* platform(Engine* engine) {
    return engine->getBackend() == backend::Backend::WEBGPU ? static_cast<backend::WebGPUPlatform*>(engine->getPlatform()) : nullptr;
}
void* device(Engine* engine) {
    backend::WebGPUPlatform* p = platform(engine);
    return p ? p->getDevice().Get() : nullptr;
}
void* instance(Engine* engine) {
    backend::WebGPUPlatform* p = platform(engine);
    return p ? p->getInstance().Get() : nullptr;
}
void on_driver(Engine* engine, std::function<void()> work) { downcast(engine)->getDriverApi().queueCommand(std::move(work)); }
}  // namespace door
