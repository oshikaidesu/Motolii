// compute_door: the one place that reaches Filament's driver for compute. Filament 1.77.2 has no public way to a
// compute shader or a storage buffer; its WebGPU backend can run them once filament_dawn_compute.patch is in. This is
// only that reach, nothing more: a program (WGSL), buffers, a set of buffers for a program, run. When Filament gets a
// public way, only compute_door.cpp changes.
// Built inside Filament's own build (it needs the engine's private headers): libmotolii_door.a, see CMakeLists.txt.
// Calls are queued like any other Filament call: `run` between beginFrame and the render of what uses the result is
// in order with it on the GPU, and nothing is waited for. `read` is for tests: it waits for the GPU.
#pragma once
#include <cstdint>

namespace filament {
class Engine;
class BufferObject;
}  // namespace filament

namespace door {
struct Program;
struct Buffer;
struct Set;
enum class Kind { UNIFORM, STORAGE };

// One buffer per slot; slot i is Filament's binding i, which is `@group(0) @binding(2 * i)` in the WGSL. A storage
// slot must be declared `var<storage, read_write>` (the layout says so for all of them).
Program* program(filament::Engine*, const char* name, const char* wgsl, const Kind* slots, int count);
Buffer* buffer(filament::Engine*, Kind, uint32_t bytes);
Buffer* buffer(filament::Engine*, filament::BufferObject* vertices);   // a vertex buffer object, to be written by compute
void write(filament::Engine*, Buffer*, const void* data, uint32_t bytes, uint32_t at = 0);   // (copies `data`)
Set* set(filament::Engine*, Program*, Buffer* const* buffers);
void run(filament::Engine*, Program*, Set*, uint32_t groupsX, uint32_t groupsY = 1, uint32_t groupsZ = 1);
void read(filament::Engine*, Buffer*, uint32_t at, uint32_t bytes, void* into);   // tests only: waits for the GPU
}  // namespace door
