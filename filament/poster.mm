// poster.mm: a band and a heap of blocks of clear glass in front of a black poster with white letters, seen straight on.
// The band is Filament's own glass (screen-space refraction, dispersion, thin film); the blocks are kit.h's.
// One picture, no switches.   ./poster.sh  ->  out/poster.png
#include "kit.h"

static const int W = 1080;
static const int H = 1920;

// The band: a rounded slab swept round a loop that leaves the view on the right. It leans, twists a little and
// swells and thins in slow waves (whole counts along the loop, so it closes).
static float3 band_dir(float u) {   // the way the loop runs
    const float a = u * TAU;
    return normalize(float3{-2.f * std::sin(a), 1.84f * std::cos(a), 0.3f * std::cos(2.f * a + 0.6f)});
}
static float3 band_point(float u, float ang) {
    auto centre = [](float a) { return float3{1.5f + 2.f * std::cos(a), 1.4f + 1.84f * std::sin(a), 0.15f * std::sin(2.f * a + 0.6f)}; };
    const float a = u * TAU, wide = 0.42f, thick = 0.315f, round = 2.4f;
    float3 T = band_dir(u);
    float3 N = normalize(cross(float3{0, 0, 1}, T)), B = cross(T, N);
    const float twist = 0.5f + 0.2f * std::sin(4.f * a);
    float3 n = N * std::cos(twist) + B * std::sin(twist), b = cross(T, n);
    const float wave = 1.f + 0.09f * (0.55f * std::sin(2.f * ang + TAU * 9.f * u) + 0.35f * std::sin(3.f * ang - TAU * 14.f * u + 1.3f)
                                      + 0.25f * std::sin(ang + TAU * 23.f * u + 4.0f) + 0.5f * std::sin(TAU * 5.f * u + 0.7f));
    const float c = std::cos(ang), s = std::sin(ang);
    return centre(a) + n * (std::copysign(std::pow(std::fabs(c), 2.f / round), c) * thick * wave)
                     + b * (std::copysign(std::pow(std::fabs(s), 2.f / round), s) * wide * wave);
}

static Mesh* band() {
    const int rings = 384, sides = 64;
    Mesh* m = new Mesh;
    for (int i = 0; i < rings; ++i)
        for (int k = 0; k < sides; ++k) {
            m->pos.push_back(band_point((float)i / rings, (float)k / sides * TAU));
            m->uv.push_back({(float)i / rings, (float)k / sides});
        }
    auto at = [&](int i, int k) { return m->pos[(size_t)(((i + rings) % rings) * sides + (k + sides) % sides)]; };
    for (int i = 0; i < rings; ++i)
        for (int k = 0; k < sides; ++k) {
            // the waves along the band are left out of the normal: the lamps then mirror as long lines, not as dots
            m->nrm.push_back(normalize(cross(at(i, k + 1) - at(i, k - 1), band_dir((float)i / rings))));
            const uint32_t i1 = (uint32_t)((i + 1) % rings), k1 = (uint32_t)((k + 1) % sides);
            const uint32_t p = (uint32_t)(i * sides + k), q = (uint32_t)(i * sides) + k1, r = i1 * sides + (uint32_t)k, t = i1 * sides + k1;
            m->idx.insert(m->idx.end(), {p, q, r, q, t, r});
        }
    return m;
}

// The heap of blocks, placed by hand: on the letters (over plain black a block shows little), clear of one another as seen
// (a block does not show another block through itself), one large with a corner towards the eye.
// {middle}, half the edge, {the axis it is turned about}, how far
static std::vector<mat4f> heap() {
    struct Placed { float3 c; float h; float3 axis; float turn; };
    const Placed all[] = {{{0.22f, 2.30f, 0.5f}, 0.15f, {0.3f, 1.f, 0.2f}, 0.7f},  {{0.62f, 1.70f, 0.6f}, 0.20f, {1.f, -0.6f, 0.3f}, 0.85f},
                          {{0.10f, 1.02f, 0.2f}, 0.10f, {0.5f, 1.f, -0.4f}, 1.1f}, {{0.60f, 0.66f, 0.7f}, 0.15f, {-0.4f, 1.f, 0.6f}, 0.6f},
                          {{0.84f, 1.21f, 0.3f}, 0.06f, {1.f, 0.3f, 0.5f}, 0.9f}};
    std::vector<mat4f> out;
    for (const Placed& p : all) out.push_back(mat4f::translation(p.c) * mat4f::rotation(p.turn, normalize(p.axis)) * mat4f::scaling(float3{2.f * p.h}));
    return out;
}

// White letters on black, drawn by the system's own text drawing.
static std::vector<uint8_t> poster_pixels() {
    std::vector<uint8_t> px((size_t)W * H * 4, 0);
    CGColorSpaceRef cs = CGColorSpaceCreateDeviceRGB();
    CGContextRef cg = CGBitmapContextCreate(px.data(), W, H, 8, W * 4, cs, kCGImageAlphaPremultipliedLast);
    CGContextSetRGBFillColor(cg, 0, 0, 0, 1);
    CGContextFillRect(cg, CGRectMake(0, 0, W, H));
    [NSGraphicsContext setCurrentContext:[NSGraphicsContext graphicsContextWithCGContext:cg flipped:NO]];
    auto text = [&](NSString* t, CGFloat size, CGFloat x, CGFloat top) {
        [t drawAtPoint:NSMakePoint(x, H - top - size * 1.2) withAttributes:@{NSFontAttributeName : [NSFont fontWithName:@"Helvetica-Bold" size:size], NSForegroundColorAttributeName : NSColor.whiteColor}];
    };
    auto dashes = [&](CGFloat top) {
        CGContextSetRGBFillColor(cg, 1, 1, 1, 1);
        for (int x = 90; x < W - 90; x += 22) CGContextFillRect(cg, CGRectMake(x, H - top, 12, 6));
    };
    text(@"MOTOLII", 26, 90, 80);
    text(@"2026", 40, 90, 110);
    text(@"FILAMENT", 26, 520, 80);
    text(@"1.77.2", 40, 520, 110);
    text(@"CLEAR", 268, 78, 300);
    text(@"GLASS", 268, 78, 560);
    dashes(940);
    text(@"MOVE", 268, 78, 990);
    text(@"MORE", 268, 78, 1250);
    dashes(1640);
    text(@"REFRACTION", 34, 90, 1670);
    text(@"DISPERSION", 34, 90, 1712);
    text(@"ONE PASS", 34, 620, 1670);
    text(@"AS IT IS", 34, 620, 1712);
    [NSGraphicsContext.currentContext flushGraphics];
    [NSGraphicsContext setCurrentContext:nil];
    CGContextRelease(cg);
    CGColorSpaceRelease(cs);
    return px;
}

int main() {
    Stage s = open_stage(W, H);
    const float3 eye{0.f, 1.4f, 6.f};
    const float fov = 30.f;
    s.camera->lookAt(eye, {0.f, 1.4f, 0.f}, {0, 1, 0});
    s.camera->setProjection(fov, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.3f).saturation(1.2f).build(*s.engine));

    // the room: the small photo studio that comes with Filament (poster.sh makes the .ktx with cmgen), and a sun
    if (!room(s, "out/small_studio/small_studio_ibl.ktx", 60000.f)) return fail("out/small_studio/small_studio_ibl.ktx");
    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color({1.0f, 0.98f, 0.95f}).intensity(57000.f).direction(normalize(float3{0.35f, -0.6f, -0.7f}))
        .castShadows(false).build(*s.engine, sun);
    s.scene->addEntity(sun);

    // the poster fills the view, 0.6 behind the band's middle (further back and the letters tear instead of bending)
    const float dist = 6.6f, hh = dist * std::tan(0.5f * fov * TAU / 360.f), hw = hh * W / H;
    Mesh* sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        sheet->pos.push_back({eye.x + hw * c.x, eye.y + hh * c.y, eye.z - dist});
        sheet->nrm.push_back({0, 0, 1});
        sheet->uv.push_back(c * 0.5f + 0.5f);
    }
    sheet->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* sheet_mi = add(s, sheet, "poster.filamat");
    if (!sheet_mi) return fail("poster.filamat");
    static std::vector<uint8_t> px = poster_pixels();
    Texture* tex = Texture::Builder().width(W).height(H).levels(1).format(Texture::InternalFormat::SRGB8_A8).build(*s.engine);
    tex->setImage(*s.engine, 0, Texture::PixelBufferDescriptor(px.data(), px.size(), Texture::Format::RGBA, Texture::Type::UBYTE));
    sheet_mi->setParameter("color", tex, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));

    // the band: on layer 2 with the blocks, so a block does not show the band through itself. (With the band in the
    // picture of what is behind, Filament made its copy of the screen and its seven blurred steps twice a frame; a
    // music video wants the lavish picture, not the exact one.)
    MaterialInstance* band_mi = add(s, band(), "poster_glass.filamat", {mat4f{}}, 2);
    if (!band_mi) return fail("poster_glass.filamat");
    band_mi->setParameter("thickness", 0.6f);
    band_mi->setParameter("lean", 0.f);

    MaterialInstance* blocks_mi = glass_blocks(s, heap(), 0.1f);
    if (!blocks_mi) return fail("glassbox.filamat");
    blocks_mi->setParameter("abbe", 2.5f);       // (the colours split wide: below 2 the eight steps show as stripes)
    blocks_mi->setParameter("envGain", 0.05f);   // (glassbox's own studio, nearly off: at 1 the blocks go milky, at 0 they sink into black)
    blocks_mi->setParameter("lean", 0.08f);      // (the letters inside run like water; at 0.15 they break up)

    if (!shoot(s, "out/poster.png")) return fail("png");
    std::printf("wrote out/poster.png\n");
    return 0;
}
