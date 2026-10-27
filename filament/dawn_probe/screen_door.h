// screen_door: one picture drawn over another by a fragment shader in WGSL, through Filament's own driver (as
// compute_door runs compute): a whole-screen effect. Filament 1.77.2 has no public way to put one's own shader over
// a picture; its post-processing draws Filament's own full-screen triangle with a program, and so does this (its makers:
// "currently not directly possible ... via off screen render targets", google/filament discussion #7676; the same calls as
// its own: third_party/filament_src/filament/src/PostProcessManager.cpp, beginRenderPass then
// draw(pipeline, getFullScreenRenderPrimitive(), 0, 3, 1)). Filament is built here from its source as kept (third_party),
// so its inside is the one this was written against until that is changed on purpose.
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
// `layered`: which pictures are arrays of layers (bit k: picture k is a `texture_2d_array<f32>`, a stage's past frames).
// `mesh`: drawn on triangles given to `draw` (`corners`), not on the whole target: each corner its place in the target
// (-1 to 1, up) and the uv it hands the shader, as Filament's own full-screen triangle is given (a vertex buffer of
// float4 at POSITION and an index buffer made a render primitive: details/Engine.cpp).
Screen* screen(filament::Engine*, const char* name, const char* fragment, uint32_t knobBytes, std::string* why = nullptr, int pictures = 1, unsigned layered = 0, bool mesh = false);
void knobs(filament::Engine*, Screen*, const void* data, uint32_t bytes, int pass = 0);
void draw(filament::Engine*, Screen*, filament::Texture* const* from, filament::RenderTarget* to, uint32_t w, uint32_t h, int pass = 0, const int* at = nullptr, bool over = false, int blend = 0, int sampler = 0, const float* corners = nullptr, uint32_t count = 0);   // corners: a mesh's `count` (x, y, u, v); to null: the screen; at: only there (left, bottom, wide, high), the rest cleared; over: laid over what `to` holds (its alpha premultiplied, as Filament's own pictures are), not drawn in its place
// (blend, with over: 1 added to what is there, 2 what is there times the alpha drawn (AviUtl2's "add" and "mask");
//  sampler: 0 linear and clamped, 1 nearest, 2 repeated, 3 mirrored (its SamplerState's ways))
inline void draw(filament::Engine* e, Screen* s, filament::Texture* from, filament::RenderTarget* to, uint32_t w, uint32_t h, const int* at = nullptr) { draw(e, s, &from, to, w, h, 0, at); }
void drop(filament::Engine*, Screen*);
}  // namespace door
