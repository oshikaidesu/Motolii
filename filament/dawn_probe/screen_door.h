// screen_door: one picture drawn over another by a fragment shader in WGSL, through Filament's own driver (as
// compute_door runs compute): a whole-screen effect. Filament 1.77.2 has no public way to put one's own shader over
// a picture; its post-processing draws Filament's own full-screen triangle with a program, and so does this.
// The shader is a fragment shader as vgpu writes one: `fn main(@builtin(position) frag: vec4f, @location(0) uv: vec2f)
// -> @location(0) vec4f` (uv 0,0 at the top left), the picture `@group(0) @binding(0) var <name>: texture_2d<f32>`,
// its sampler `@binding(1)`, and, if it has knobs, `@binding(2) var<uniform> <name>: <struct>`.
// Calls are queued like any other Filament call: `draw` between the render of the picture it reads and the render of
// what shows its result is in order with them on the GPU.
#pragma once
#include <cstdint>
#include <string>

namespace filament {
class Engine;
class Texture;
class RenderTarget;
}  // namespace filament

namespace door {
struct Screen;
// A fragment shader Dawn would refuse is not made (nullptr, `why` says what Dawn said): it is tried first on Filament's
// own device, inside an error scope, so a shader written wrong takes nothing else down.
// It may read several pictures (`pictures`: picture i at `@binding(2i)`, its sampler at `2i+1`, the knobs then at
// `2 * pictures`), and be drawn several times a frame as passes, each pass with knobs of its own (`pass`).
Screen* screen(filament::Engine*, const char* name, const char* fragment, uint32_t knobBytes, std::string* why = nullptr, int pictures = 1);
void knobs(filament::Engine*, Screen*, const void* data, uint32_t bytes, int pass = 0);
void draw(filament::Engine*, Screen*, filament::Texture* const* from, filament::RenderTarget* to, uint32_t w, uint32_t h, int pass = 0);   // to null: the screen
inline void draw(filament::Engine* e, Screen* s, filament::Texture* from, filament::RenderTarget* to, uint32_t w, uint32_t h) { draw(e, s, &from, to, w, h, 0); }
void drop(filament::Engine*, Screen*);
}  // namespace door
