// acid.mm: a poster of flat graphics with a burst of crushed glass over it (the reference: research_notes/refs/F).
// Two moves: the flat picture (ground, giant letters, lime bars, black figure, type, small marks, hairline arcs: one
// SVG, acid_flat.svg, drawn by ThorVG on Filament's own device, drawn.h) on a sheet; and the burst, one solid of glass
// made by one call and put down as it comes (Filament's own glass, kit.h: it bends and takes the colours of the flat
// picture behind it). WebGPU backend only.
//   ./dawn_probe/kit_wgpu.sh acid          ->  out/wgpu_kit/out/acid.png
#include "kit.h"
#include "drawn.h"

static const int W = 1920;
static const int H = 1080;
static const float HIGH = 1.f, WIDE = HIGH * W / H;   // the frame in the scene: the sheet is 2 high, 3.56 wide

// a place of the frame (fractions: x from the left, y from the top) in the scene
static float2 at(float fx, float fy) { return {(fx - 0.5f) * 2.f * WIDE, (0.5f - fy) * 2.f * HIGH}; }

// A burst of crushed blades, as ONE solid: `count` ribbons out of a knot, each bent in one arc, twisted and pinched
// like something squashed, widest about its middle and running out to needles. Nothing is placed by hand: the
// ribbons' ways, lengths, bends and twists are drawn by `seed` from one recipe: long and few towards `lean` (degrees,
// 0 right, 90 up), short and many the other way, a few hooked right round. knot: where; reach: how far the longest goes.
static Mesh* burst(float2 knot, float reach, float lean, int count, uint32_t seed) {
    const int rings = 96, sides = 14;
    Mesh* m = new Mesh;
    uint32_t state = seed * 2654435761u + 12345u;
    const auto rnd = [&]() { state = state * 1664525u + 1013904223u; return (float)(state >> 8) / 16777216.f; };   // 0..1
    const auto between = [&](float a, float b) { return a + (b - a) * rnd(); };
    for (int n = 0; n < count; ++n) {
        const bool longway = n < count / 5;   // a fifth go the long way
        const float heading = (lean + (longway ? between(-20.f, 20.f) : between(40.f, 320.f))) * TAU / 360.f;
        const float length = reach * (longway ? (n == 0 ? between(0.6f, 0.75f) : between(0.55f, 1.f)) : between(0.16f, 0.38f));
        const bool hook = n == 0 || rnd() < 0.2f;   // (the first of the long ones always ends in a sickle)
        const float bend = (rnd() < 0.5f ? -1.f : 1.f) * (hook ? between(1.6f, 2.6f) : longway ? between(0.25f, 1.0f) : between(0.2f, 0.9f));
        const float thin = rnd();
        const float wide = reach * (0.007f + 0.017f * thin) * (hook ? 1.4f : longway ? 1.f : 0.6f);   // (the short ones thinner)
        const float twist = between(-3.2f, 3.2f), depth = between(0.1f, 0.7f), bow = between(0.05f, 0.3f);
        const float pinches = between(1.f, 3.f), pinch_at = between(0.f, TAU), kink = between(-0.3f, 0.3f);
        const float2 from = knot + reach * 0.05f * float2{between(-1.f, 1.f), between(-1.f, 1.f)};   // (not all out of one point: a knot)
        const uint32_t first = (uint32_t)m->pos.size();
        float2 c = from;
        for (int i = 0; i <= rings; ++i) {
            const float u = (float)i / rings;
            const float h = heading + bend * (hook ? u * u * (3.f - 2.f * u) * u : u * u) + kink * std::sin(0.5f * TAU * u);   // (the arc tightens towards the tip; a hook turns late and fast, while it is still wide: a sickle)
            if (i) c += length / rings * float2{std::cos(h), std::sin(h)};
            const float3 centre{c.x, c.y, depth + bow * std::sin(0.5f * TAU * u)};
            // (the sine is held at 0 or more: at the very tip it comes out a hair below, and a power of that is no number at all)
            const float w = std::max(wide * std::pow(std::max(std::sin(0.5f * TAU * u), 0.f), 0.7f) * (1.f + 0.22f * std::sin(pinches * TAU * u + pinch_at)), 1e-4f), t = 0.22f * w;
            const float roll = twist * (u - 0.5f);
            const float3 across{-std::sin(h), std::cos(h), 0.f}, up{0.f, 0.f, 1.f};
            const float3 nn = across * std::cos(roll) + up * std::sin(roll), bb = up * std::cos(roll) - across * std::sin(roll);
            for (int k = 0; k < sides; ++k) {
                const float a = (float)k / sides * TAU;
                m->pos.push_back(centre + nn * (std::cos(a) * w) + bb * (std::sin(a) * t));
                m->nrm.push_back(normalize(nn * (std::cos(a) / w) + bb * (std::sin(a) / t)));
                m->uv.push_back({(float)n + 0.25f * u, (float)k / sides});   // (the whole part: which ribbon, own_glass.mat makes a mixed lot of them; a quarter of the uv along one, so the glass's waves run long)
            }
        }
        for (int i = 0; i < rings; ++i)
            for (int k = 0; k < sides; ++k) {
                const uint32_t p = first + (uint32_t)(i * sides + k), q = first + (uint32_t)(i * sides + (k + 1) % sides), r = p + sides, t2 = q + sides;
                m->idx.insert(m->idx.end(), {p, q, r, q, t2, r});
            }
    }
    return m;
}

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    // seen from far off through a long lens: nearly without perspective, so the flat picture's places hold for the glass before it
    const float far_off = 24.f;
    s.camera->lookAt({0.f, 0.f, far_off - 1.f}, {0.f, 0.f, -1.f}, {0, 1, 0});
    s.camera->setProjection(2.0 * std::atan(HIGH / far_off) * 360.0 / TAU, double(W) / double(H), 1.0, 60.0, Camera::Fov::VERTICAL);
    static LinearToneMapper as_it_is;   // (the flat colours are the screen's own: nothing bends them)
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&as_it_is).build(*s.engine));
    // (a room of many narrow lights, none of them blue: they lie on the glass as thin streaks; a room of a few large ones as white patches)
    if (!room(s, "out/graffiti_shelter_2k/graffiti_shelter_2k_ibl.ktx", 34000.f)) return fail("out/graffiti_shelter_2k/graffiti_shelter_2k_ibl.ktx");

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

    // the burst: crushed glass of a bottle green, put down as it comes (Filament's own glass, kit.h: it is given no depth)
    if (!own_glass(s, burst(at(0.76f, 0.52f), 0.74f * 2.f * WIDE, 178.f, 26, 77), {mat4f{}}, float3{0.33f, 0.40f, 0.06f}, 0.15f)) return fail("own_glass.filamat");

    const auto pose = [&](float) { drawn_now(s, flat); };
    pose(0.f);
    return show(s, argc, argv, "acid", 480, 0.f, pose);
}
