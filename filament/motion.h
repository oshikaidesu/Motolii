// motion.h: After Effects' motion for the things on a stage. The motion is a Lottie file (what After Effects writes
// out, or anything that writes the same); ThorVG reads it and works out where every layer is at a moment: keys,
// easing, paths, parents, precompositions, time remapping, loops, expressions are all ThorVG's own code, none of it
// is done again here. What is taken from ThorVG is a layer's place by the layer's name. Nothing is drawn by ThorVG
// for this.
// Flat layers and 3D layers are one world: every place is a 4x4 in the composition's own space, which is After
// Effects': pixels, x to the right, y down, z away from the eye. ThorVG keeps only what is flat of a 3D layer, so a
// file with 3D layers goes through lottie3d.py first, which writes the numbers ThorVG would drop onto helper layers;
// here they are read back and the layer is put together from them, in After Effects' order (checked against
// lottie-web's own answers, motion_test.mm):  place = position · turn x · turn y · turn z · skew · size · anchor back,
// with the orientation turned before the three turns (and, before that, the turn to face its way along its path, if
// the layer has one), and a child under its parent.
#include <math/mat4.h>
#include <math/vec3.h>
#include <thorvg.h>
#include <thorvg_lottie.h>
#include <cmath>
#include <cstdio>

struct Motion {
    tvg::LottieAnimation* film = nullptr;
    tvg::Picture* picture = nullptr;
    float w = 0.f, h = 0.f, frames = 0.f, seconds = 0.f;
};

static bool motion_open(Motion& m, const char* lottie) {
    static const bool started = tvg::Initializer::init(0) == tvg::Result::Success;   // (no threads of its own: a moment is worked out when it is asked for)
    if (!started) return false;
    m.film = tvg::LottieAnimation::gen();
    m.picture = m.film->picture();
    if (m.picture->load(lottie) != tvg::Result::Success) return false;
    m.picture->size(&m.w, &m.h);
    m.frames = m.film->totalFrame();
    m.seconds = m.film->duration();
    return m.frames > 0.f;
}

// The moment: a frame of the composition, fractions too.
static void motion_at(Motion& m, float frame) { m.film->frame(frame); }

// A layer's flat place as ThorVG has it, in the composition's pixels: its own matrix (its parents are in it already)
// under the precompositions it lies in (the scenes above it), and how opaque it is under them. false while the layer
// is not there (before its in point, after its out point).
static bool motion_flat(Motion& m, uint32_t id, tvg::Matrix* place, float* opacity = nullptr) {
    const tvg::Paint* p = m.picture->paint(id);   // (found by walking the layers; `Picture::accessible`, ThorVG's index by name, finds nothing in a Lottie in 1.1.2)
    if (!p) return false;
    tvg::Matrix all = const_cast<tvg::Paint*>(p)->transform();
    float seen = (float)p->opacity() / 255.f;
    for (const tvg::Paint* up = p->parent(); up && up->type() != tvg::Type::Picture; up = up->parent()) {
        const tvg::Matrix o = const_cast<tvg::Paint*>(up)->transform();
        all = {o.e11 * all.e11 + o.e12 * all.e21, o.e11 * all.e12 + o.e12 * all.e22, o.e11 * all.e13 + o.e12 * all.e23 + o.e13,
               o.e21 * all.e11 + o.e22 * all.e21, o.e21 * all.e12 + o.e22 * all.e22, o.e21 * all.e13 + o.e22 * all.e23 + o.e23, 0.f, 0.f, 1.f};
        seen *= (float)up->opacity() / 255.f;
    }
    *place = all;
    if (opacity) *opacity = seen;
    return true;
}

static bool motion_flat(Motion& m, const char* layer, tvg::Matrix* place, float* opacity = nullptr) { return motion_flat(m, tvg::Accessor::id(layer), place, opacity); }

// A flat place as a place in the one world; its depth is sized by the layer's size, so a cube stays a cube.
static filament::math::mat4f motion_deep(const tvg::Matrix& f) {
    using namespace filament::math;
    return mat4f(float4{f.e11, f.e21, 0.f, 0.f}, float4{f.e12, f.e22, 0.f, 0.f}, float4{0.f, 0.f, std::sqrt(std::fabs(f.e11 * f.e22 - f.e12 * f.e21)), 0.f},
                 float4{f.e13, f.e23, 0.f, 1.f});
}

// Where the layer `id` is: a 3D layer (or camera) put together from its own numbers, which ThorVG says whole
// (LottieAnimation::pose), under its parent; a flat layer as ThorVG has it.
static filament::math::mat4f motion_built(Motion& m, uint32_t id, float* zoom = nullptr) {
    using namespace filament::math;
    float n[19] = {};
    uint32_t above = 0;
    int kind = 0;
    if (m.film->pose(id, n, &above, &kind) != tvg::Result::Success || kind == 0) {
        tvg::Matrix f{1, 0, 0, 0, 1, 0, 0, 0, 1};
        motion_flat(m, id, &f);
        return motion_deep(f);
    }
    const float deg = 3.14159265f / 180.f;
    const float3 p{n[0], n[1], n[2]}, a{n[3], n[4], n[5]}, s{n[6], n[7], n[8]}, o{n[9], n[10], n[11]}, r{n[12], n[13], n[14]};
    const float skew = n[15], axis = n[16];
    if (zoom) *zoom = n[18];
    const mat4f facing = mat4f::rotation(n[17] * deg, float3{0, 0, 1});   // (a layer that turns to face its way along its path: about z)
    const auto turns = [&](float3 by) {
        return mat4f::rotation(by.x * deg, float3{1, 0, 0}) * mat4f::rotation(by.y * deg, float3{0, 1, 0}) * mat4f::rotation(by.z * deg, float3{0, 0, 1});
    };
    mat4f lean;   // the skew: a shear along x, about the skew's axis
    lean[1][0] = std::tan(-skew * deg);
    lean = mat4f::rotation(-axis * deg, float3{0, 0, 1}) * lean * mat4f::rotation(axis * deg, float3{0, 0, 1});
    mat4f own;
    if (kind == 1) own = mat4f::translation(p) * facing * turns(o) * turns(r) * lean * mat4f::scaling(s * 0.01f) * mat4f::translation(-a);
    else {   // a camera: no size and no anchor; one that looks at a point is first turned to face it, level (no roll)
        mat4f towards;
        if (kind == 2 && length(a - p) > 1e-4f) {
            const float3 z = normalize(a - p), x = normalize(cross(float3{0, 1, 0}, z)), y = cross(z, x);
            towards = mat4f(float4{x, 0.f}, float4{y, 0.f}, float4{z, 0.f}, float4{0.f, 0.f, 0.f, 1.f});
        }
        own = mat4f::translation(p) * towards * turns(o) * turns(r);
    }
    return above ? motion_built(m, above) * own : own;
}

// Where a layer is at that moment, flat or 3D alike: its place in the composition's space, and how opaque it is.
// false while the layer is not there (before its in point, after its out point).
static bool motion_of(Motion& m, const char* layer, filament::math::mat4f* place, float* opacity = nullptr) {
    tvg::Matrix flat;
    if (!motion_flat(m, tvg::Accessor::id(layer), &flat, opacity)) return false;
    *place = motion_built(m, tvg::Accessor::id(layer));
    return true;
}

// A place of the composition put into the scene: the composition's corner (0, 0, 0) lies at `corner`, a pixel to the
// right is `right`, a pixel down is `down` (and a pixel away from the eye is square to both). What comes back takes a
// thing given in the scene's units, in the layer's own axes (x right, y down, z away), to where the layer has it.
static filament::math::mat4f motion_in_scene(const filament::math::mat4f& place, filament::math::float3 corner, filament::math::float3 right, filament::math::float3 down) {
    using namespace filament::math;
    const float pixel = length(right);
    const float3 x = right / pixel, y = normalize(down), z = cross(x, y);
    return mat4f(float4{x * pixel, 0.f}, float4{y * pixel, 0.f}, float4{z * pixel, 0.f}, float4{corner, 1.f}) * place * mat4f::scaling(1.f / pixel);
}

// A camera layer at that moment: where it stands and looks (its own axes: x right, y down, looking along +z), and
// its angle of view from top to bottom, in degrees, for a picture `high` pixels high (After Effects gives a camera a
// zoom: the distance in pixels at which a pixel of the composition is a pixel of the picture).
// For Filament's camera, whose own axes are x right, y up, looking along -z:
//   camera->setModelMatrix(motion_in_scene(place, corner, right, down) * mat4f::scaling(float3{1, -1, -1}));
static bool motion_camera(Motion& m, const char* layer, filament::math::mat4f* place, float* view_degrees, float high) {
    tvg::Matrix flat;
    float zoom = 0.f;
    if (!motion_flat(m, layer, &flat)) return false;
    *place = motion_built(m, tvg::Accessor::id(layer), &zoom);
    if (zoom <= 0.f) return false;
    *view_degrees = 2.f * std::atan(0.5f * high / zoom) * 180.f / 3.14159265f;
    return true;
}
