#include "dawn_probe/screen_door.h"
// drawn_edge.h: how far each point of a picture is from its edge, in points, for its bevel (deep.glsl): the pictures of
// bevy_mod_outline's flood (src/flood/sobel_init.wgsl, jump_flood.wgsl; MIT OR Apache-2.0), its seeds and steps as they
// are, the door's way of saying them: seeds from what is not the picture, so it floods inward, and a last pass that
// keeps the length, which reads smoothly between points where the way to the edge turns round.
static const char* const edge_seeds_wgsl = R"(
@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
fn sample(p: vec2i) -> f32 {
    if (any(p < vec2i(0)) || any(p >= vec2i(textureDimensions(picture)))) { return 1.0; }
    return 1.0 - textureLoad(picture, p, 0).a;
}
@fragment fn main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
    let pos = vec2i(frag.xy);
    let coverage = sample(pos);
    let last = vec2i(textureDimensions(picture)) - vec2i(1);
    if (coverage <= 0.1) {   // (bevy's is 0, no seam in its outlines; a glyph's own, a hair under full, is not its edge. A picture's rim is: it stops there)
        if (any(pos == vec2i(0)) || any(pos == last)) { return vec4f(0.0, 0.0, 0.0, 1.0); }
        return vec4f(4096.0, 4096.0, 0.0, 1.0);
    }
    let tl = sample(pos + vec2i(-1, -1)); let tc = sample(pos + vec2i(0, -1)); let tr = sample(pos + vec2i(1, -1));
    let ml = sample(pos + vec2i(-1, 0)); let mr = sample(pos + vec2i(1, 0));
    let bl = sample(pos + vec2i(-1, 1)); let bc = sample(pos + vec2i(0, 1)); let br = sample(pos + vec2i(1, 1));
    let g = vec2f((tr + 2.0 * mr + br) - (tl + 2.0 * ml + bl), (bl + 2.0 * bc + br) - (tl + 2.0 * tc + tr));
    var delta = vec2f(0.0);
    if (length(g) > 1e-4) { delta = -g / length(g) * (coverage - 0.5); }
    return vec4f(delta, 0.0, 1.0);
}
)";
static const char* const edge_flood_wgsl = R"(
struct Knobs { step_length: f32, a: f32, b: f32, c: f32 };
@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@group(0) @binding(2) var<uniform> instance: Knobs;
@fragment fn main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
    let dims = vec2i(textureDimensions(screen_texture));
    let step = i32(instance.step_length);
    let pos = vec2i(frag.xy);
    var result = textureLoad(screen_texture, pos, 0).xy;
    var closest_dist = length(result);
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            if (dx == 0 && dy == 0) { continue; }
            let offset = vec2i(dx * step, dy * step);
            let neighbour_coord = pos + offset;
            if (any(neighbour_coord < vec2i(0)) || any(neighbour_coord >= dims)) { continue; }
            let delta = textureLoad(screen_texture, neighbour_coord, 0).xy + vec2f(offset);
            let dist = length(delta);
            if (dist < closest_dist) { closest_dist = dist; result = delta; }
        }
    }
    return vec4f(result, 0.0, 1.0);
}
)";
static const char* const edge_length_wgsl = R"(
@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@fragment fn main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
    return vec4f(length(textureLoad(picture, vec2i(frag.xy), 0).xy), 0.0, 0.0, 1.0);
}
)";

// Its pictures, as large as it is: two the flood goes between, one the look reads.
static void edge_make(Stage& s, Drawn* d, uint32_t w, uint32_t h) {
    for (int k = 0; k < 3; k++) {
        d->edge[k] = Texture::Builder().width(w).height(h).levels(1).format(Texture::InternalFormat::RGBA16F).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).build(*s.engine);
        d->edgeInto[k] = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, d->edge[k]).build(*s.engine);
    }
    d->edgeW = w, d->edgeH = h;
}
static void edge_off(Stage& s, Drawn* d) {
    s.once.erase(d);
    for (int k = 0; k < 3; k++) if (d->edge[k]) s.engine->destroy(d->edgeInto[k]), s.engine->destroy(d->edge[k]), d->edge[k] = nullptr;
}
// Flooded as far as `reach` points, the steps halving from the power of two at or past it, once in the next frame.
static void edge_ask(Stage& s, Drawn* d, int reach) {
    d->stale = false;
    s.once[d] = [&s, d, reach] {
        static std::map<Engine*, std::array<door::Screen*, 3>> doors;
        auto& made = doors[s.engine];
        if (!made[0]) made = {door::screen(s.engine, "edge seeds", edge_seeds_wgsl, 0), door::screen(s.engine, "edge flood", edge_flood_wgsl, 16), door::screen(s.engine, "edge length", edge_length_wgsl, 0)};
        if (!made[0] || !made[1] || !made[2]) return;
        door::draw(s.engine, made[0], d->texture, d->edgeInto[0], d->edgeW, d->edgeH);
        int front = 0, pass = 0, step = 1;
        while (step < reach) step *= 2;
        for (; step >= 1; step /= 2, pass++) {
            const float k[4] = {(float)step, 0.f, 0.f, 0.f};
            door::knobs(s.engine, made[1], k, sizeof k, pass);
            Texture* from = d->edge[front];
            door::draw(s.engine, made[1], &from, d->edgeInto[!front], d->edgeW, d->edgeH, pass);
            front = !front;
        }
        door::draw(s.engine, made[2], d->edge[front], d->edgeInto[2], d->edgeW, d->edgeH);
    };
}
