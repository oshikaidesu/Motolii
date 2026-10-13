// gpu_door: Filament's own WebGPU device lent to someone else who draws with WebGPU (ThorVG), and their drawing put
// in line with Filament's. Filament 1.77.2 has no public way to either; with filament_dawn_compute.patch its WebGPU
// platform remembers the device it made and its Texture::Builder::import takes a WGPUTexture of that device.
// Built inside Filament's own build like compute_door (it needs the engine's private headers): libmotolii_door.a.
#pragma once
#include <functional>

namespace filament {
class Engine;
}

namespace door {
void* device(filament::Engine*);     // the WGPUDevice Filament draws with (null on another backend)
void* instance(filament::Engine*);   // its WGPUInstance
// `work` is queued like any other Filament call: it runs on Filament's driver thread, in order with what was asked
// before and after it, so what it draws on the device is ready for the frame rendered after this call.
void on_driver(filament::Engine*, std::function<void()> work);
}  // namespace door
