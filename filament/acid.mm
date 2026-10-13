// acid.mm: a poster of flat graphics with a burst of lacquered blades over it (the reference: research_notes/refs/F).
// Layers, back to front, as one picture: the flat layers (ground, giant letters, lime bars, black figure, type, small
// marks, hairline arcs) are one SVG, acid_flat.svg, drawn by ThorVG on Filament's own device (drawn.h) and shown on a
// sheet; the blades are Filament's own lit, clear-coated surfaces before it, mirroring a studio. WebGPU backend only.
//   ./dawn_probe/kit_wgpu.sh acid          ->  out/wgpu_kit/out/acid.png
#include "kit.h"
#include "drawn.h"

static const int W = 1920;
static const int H = 1080;
static const float HIGH = 1.f, WIDE = HIGH * W / H;   // the frame in the scene: 2 high, 3.56 wide, seen square-on without perspective

// a place of the frame (fractions: x from the left, y from the top) in the scene
static float2 at(float fx, float fy) { return {(fx - 0.5f) * 2.f * WIDE, (0.5f - fy) * 2.f * HIGH}; }

// A blade: a ribbon bent in one even arc, widest at mid-length and running out to a needle at both ends, its section a
// flat lens. from: where it starts; heading: which way (degrees, 0 right, 90 up); length; bend: how far it turns over
// its length (radians); wide: half its greatest width; depth: how far before the sheet it lies, bow: how far its
// middle rises towards the eye; twist: how far it rolls about itself from end to end.
static Mesh* blade(float2 from, float heading, float length, float bend, float wide, float depth, float bow, float twist) {
    const int rings = 120, sides = 16;
    Mesh* m = new Mesh;
    const float h0 = heading * TAU / 360.f;
    for (int i = 0; i <= rings; ++i) {
        const float u = (float)i / rings, h = h0 + bend * u;
        float2 c = from;
        if (std::fabs(bend) < 1e-4f) c += length * u * float2{std::cos(h0), std::sin(h0)};
        else c += length / bend * float2{std::sin(h) - std::sin(h0), std::cos(h0) - std::cos(h)};
        const float3 centre{c.x, c.y, depth + bow * std::sin(0.5f * TAU * u)};
        const float w = std::max(wide * std::pow(std::sin(0.5f * TAU * u), 0.8f), 1e-4f), t = 0.2f * w;
        const float roll = twist * (u - 0.5f);
        const float3 across{-std::sin(h), std::cos(h), 0.f}, up{0.f, 0.f, 1.f};
        const float3 n = across * std::cos(roll) + up * std::sin(roll), b = up * std::cos(roll) - across * std::sin(roll);
        for (int k = 0; k < sides; ++k) {
            const float a = (float)k / sides * TAU;
            m->pos.push_back(centre + n * (std::cos(a) * w) + b * (std::sin(a) * t));
            m->nrm.push_back(normalize(n * (std::cos(a) / w) + b * (std::sin(a) / t)));
            m->uv.push_back({u, (float)k / sides});
        }
    }
    for (int i = 0; i < rings; ++i)
        for (int k = 0; k < sides; ++k) {
            const uint32_t p = (uint32_t)(i * sides + k), q = (uint32_t)(i * sides + (k + 1) % sides), r = p + sides, t2 = q + sides;
            m->idx.insert(m->idx.end(), {p, q, r, q, t2, r});
        }
    return m;
}

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    s.camera->lookAt({0.f, 0.f, 10.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    s.camera->setProjection(Camera::Projection::ORTHO, -WIDE, WIDE, -HIGH, HIGH, 0.1, 40.0);
    static LinearToneMapper as_it_is;   // (the flat colours are the screen's own: nothing bends them)
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&as_it_is).build(*s.engine));
    if (!room(s, "out/lightroom/lightroom_ibl.ktx", 34000.f)) return fail("out/lightroom/lightroom_ibl.ktx");

    // the flat layers, drawn by ThorVG
    Drawn* flat = drawn_open(s, W, H, [](tvg::WgCanvas* canvas) {
        tvg::Text::load("/System/Library/Fonts/Supplemental/Arial Black.ttf");
        tvg::Picture* p = tvg::Picture::gen();
        for (const char* file : {"acid_flat.svg", "../../acid_flat.svg"})
            if (p->load(file) == tvg::Result::Success) break;
        p->size((float)W, (float)H);
        canvas->add(p);
    });
    if (!flat) return fail("drawn_open: the WebGPU backend, ThorVG built against Dawn and the patched Filament are needed (BACKEND=webgpu)");
    Mesh* sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        sheet->pos.push_back({WIDE * c.x, HIGH * c.y, -1.f});
        sheet->nrm.push_back({0, 0, 1});
        sheet->uv.push_back(c * 0.5f + 0.5f);
    }
    sheet->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* sheet_mi = add(s, sheet, "flat.filamat");
    if (!sheet_mi) return fail("flat.filamat");
    sheet_mi->setParameter("picture", flat->texture, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));

    // the burst: blades out of a knot right of the middle, long and few to the left, short and many to the right
    const float2 knot = at(0.76f, 0.52f);
    const float3 olive{0.16f, 0.22f, 0.03f}, lime{0.50f, 0.74f, 0.10f}, teal{0.02f, 0.10f, 0.09f}, sea{0.10f, 0.36f, 0.28f},
                 iron{0.20f, 0.22f, 0.22f}, silver{0.78f, 0.80f, 0.78f}, soot{0.008f, 0.012f, 0.008f}, moss{0.07f, 0.10f, 0.04f};
    struct Blade { float2 from; float heading, length, bend, wide, depth, bow, twist; float3 dark, light; float metal, hatch; };
    const float F = 2.f * WIDE;   // (lengths below are shares of the frame's width)
    const Blade blades[] = {
        // long and few to the left
        {knot, 176.f, 0.74f * F, -0.30f, 0.060f, 0.30f, 0.25f, 1.2f, olive, lime, 0.3f, 0.f},
        {knot, 187.f, 0.66f * F, 0.38f, 0.050f, 0.20f, 0.20f, -0.9f, teal, sea, 0.5f, 0.f},
        {knot, 166.f, 0.58f * F, 0.30f, 0.042f, 0.45f, 0.15f, 0.6f, moss, lime, 0.3f, 0.f},
        {knot, 181.f, 0.50f * F, 1.75f, 0.064f, 0.55f, 0.20f, 1.6f, iron, silver, 1.0f, 0.f},     // the sickle: thick, hooking
        {at(0.30f, 0.60f), 127.f, 0.44f * F, -0.22f, 0.020f, 0.60f, 0.05f, 0.f, soot, moss, 0.4f, 1.f},   // long, thin, dark, hatched, up to the top left corner
        // up
        {knot, 78.f, 0.30f * F, -0.55f, 0.050f, 0.35f, 0.20f, 1.0f, olive, lime, 0.3f, 0.f},
        {knot, 104.f, 0.27f * F, 0.35f, 0.040f, 0.25f, 0.15f, -0.8f, soot, moss, 0.6f, 0.f},
        {knot, 52.f, 0.24f * F, 0.70f, 0.045f, 0.50f, 0.20f, 0.9f, teal, sea, 0.5f, 0.f},
        {knot, 128.f, 0.36f * F, -0.45f, 0.044f, 0.40f, 0.25f, 1.3f, iron, silver, 1.0f, 0.f},
        {knot, 143.f, 0.44f * F, 0.25f, 0.036f, 0.15f, 0.15f, -0.5f, olive, lime, 0.3f, 0.f},
        // down
        {knot, -82.f, 0.22f * F, 0.50f, 0.048f, 0.30f, 0.20f, 1.1f, olive, lime, 0.3f, 0.f},
        {knot, -112.f, 0.27f * F, -0.55f, 0.042f, 0.45f, 0.15f, -1.0f, iron, silver, 1.0f, 0.f},
        {knot, -58.f, 0.20f * F, -0.60f, 0.040f, 0.20f, 0.15f, 0.7f, soot, moss, 0.6f, 0.f},
        {knot, -138.f, 0.36f * F, 0.45f, 0.046f, 0.55f, 0.25f, 1.4f, teal, sea, 0.5f, 0.f},
        {knot, -160.f, 0.46f * F, -0.35f, 0.034f, 0.10f, 0.15f, -0.6f, moss, lime, 0.3f, 0.f},
        // short and many to the right
        {knot, 12.f, 0.13f * F, 0.60f, 0.040f, 0.40f, 0.10f, 0.8f, olive, lime, 0.3f, 0.f},
        {knot, -22.f, 0.14f * F, -0.50f, 0.036f, 0.25f, 0.10f, -0.7f, iron, silver, 1.0f, 0.f},
        {knot, 33.f, 0.12f * F, -0.70f, 0.034f, 0.55f, 0.10f, 1.0f, soot, moss, 0.6f, 0.f},
        {knot, -42.f, 0.13f * F, 0.65f, 0.038f, 0.35f, 0.10f, -0.9f, teal, sea, 0.5f, 0.f},
    };
    for (const Blade& b : blades) {
        MaterialInstance* mi = add(s, blade(b.from, b.heading, b.length, b.bend, b.wide, b.depth, b.bow, b.twist), "blade.filamat");
        if (!mi) return fail("blade.filamat");
        mi->setParameter("dark", b.dark);
        mi->setParameter("light", b.light);
        mi->setParameter("metal", b.metal);
        mi->setParameter("hatch", b.hatch);
    }

    const auto pose = [&](float) { drawn_now(s, flat); };
    pose(0.f);
    return show(s, argc, argv, "acid", 480, 0.f, pose);
}
