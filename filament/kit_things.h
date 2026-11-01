// kit_things.h: part of kit.h. Read only through kit.h.
// Letters as a solid, by the system's own solid text (SceneKit makes it, ModelIO hands over the mesh): `deep`
// through, the edges cut off by `chamfer`; a capital is about 0.7 high. The letters' lower left is at the origin,
// they run along x and stand up along y, the middle of their depth is at z = 0.
static Mesh* letters(const char* text, float deep, float chamfer) {
    const float size = 10.f;   // (SceneKit cuts curves by a fixed fineness: asked large, then made small)
    SCNText* solid = [SCNText textWithString:@(text) extrusionDepth:deep * size];
    solid.font = [NSFont fontWithName:@"Helvetica-Bold" size:size];
    solid.chamferRadius = chamfer * size;
    solid.flatness = 0.02;
    MDLMesh* made = [MDLMesh meshWithSCNGeometry:solid];
    MDLVertexAttributeData* ps = [made vertexAttributeDataForAttributeNamed:MDLVertexAttributePosition asFormat:MDLVertexFormatFloat3];
    MDLVertexAttributeData* ns = [made vertexAttributeDataForAttributeNamed:MDLVertexAttributeNormal asFormat:MDLVertexFormatFloat3];
    if (!made.vertexCount || !ps || !ns) return nullptr;
    Mesh* m = new Mesh;
    float3 low{1e9f};
    for (NSUInteger i = 0; i < made.vertexCount; ++i) {
        const float* p = (const float*)((const uint8_t*)ps.dataStart + i * ps.stride);
        const float* n = (const float*)((const uint8_t*)ns.dataStart + i * ns.stride);
        m->pos.push_back(float3{p[0], p[1], p[2]} / size);
        m->nrm.push_back(normalize(float3{n[0], n[1], n[2]}));
        m->uv.push_back({0, 0});
        low = min(low, m->pos.back());
    }
    for (float3& p : m->pos) p = p - float3{low.x, low.y, 0.f};
    for (MDLSubmesh* part in made.submeshes) {
        if (part.geometryType != MDLGeometryTypeTriangles) continue;
        const void* b = [part.indexBuffer map].bytes;
        for (NSUInteger i = 0; i < part.indexCount; ++i)
            m->idx.push_back(part.indexType == MDLIndexBitDepthUInt32 ? ((const uint32_t*)b)[i] : part.indexType == MDLIndexBitDepthUInt16 ? (uint32_t)((const uint16_t*)b)[i] : (uint32_t)((const uint8_t*)b)[i]);
    }
    return m->idx.empty() ? nullptr : m;
}

// Paper under a hard sun, with one of Filament's studios as the sky that fills the shadows (kept low, so the shadows
// stay dark). What the glass things stand on in blocks.mm and shapes.mm.
// How much sun reaches the paper is one picture, `sunlight`: the floor from straight above, 1 where the sun is bare,
// 0 in a thing's shade (shadow_of), more where glass throws its light (glass_light, box_light). paper.mat softens it
// and turns it into light, so shade and thrown light are equally soft and of one piece. (Filament's own sun shadow is
// not used: it is shade or no shade, and knows nothing of thrown light.)
struct Sunlit {
    float3 paper{0.52f, 0.60f, 0.65f};   // (a middle blue-grey: the light the glass throws must be able to be brighter than the paper)
    float3 sunWay = normalize(float3{-0.55f, -0.50f, 0.67f});   // (the sun low, the shadows long)
    float sunLux = 100000.f;
    float grain = 0.12f;                 // (the paper's tooth)
    float sky = 7000.f;                  // (the studio's light, which fills the shadows: low, so they stay dark)
    const char* studio = "out/lightroom/lightroom_ibl.ktx";   // (the room: it lights the shadows and is what Filament's glass mirrors)
    float sunWide = 0.022f;              // (the sun's width: how far shade spreads for every unit it flew; far wider than the true sun, for the softness)
};
static bool paper_and_sun(Stage& s, const Sunlit& lit) {
    if (!room(s, lit.studio, lit.sky)) return false;
    s.sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color({1.0f, 0.98f, 0.95f}).intensity(lit.sunLux).direction(lit.sunWay)
        .castShadows(false).build(*s.engine, s.sun);
    s.scene->addEntity(s.sun);

    // the picture of sunlight: 8 units of floor about the middle, seen from above
    const float half = 4.f;
    const uint32_t size = 2048;
    s.sunlight = Texture::Builder().width(size).height(size).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE)
                     .format(Texture::InternalFormat::RGBA16F).build(*s.engine);
    Texture* depth = Texture::Builder().width(size).height(size).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*s.engine);
    Camera* above = s.engine->createCamera(utils::EntityManager::get().create());
    above->setProjection(Camera::Projection::ORTHO, -half, half, -half, half, 0.1, 40.0);
    above->lookAt({0, 20, 0}, {0, 0, 0}, {0, 0, -1});
    s.view_floor = s.engine->createView();
    s.view_floor->setScene(s.scene);
    s.view_floor->setCamera(above);
    s.view_floor->setViewport({0, 0, size, size});
    s.view_floor->setRenderTarget(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, s.sunlight).texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*s.engine));
    s.view_floor->setPostProcessingEnabled(false);
    s.view_floor->setShadowingEnabled(false);
    s.view_floor->setVisibleLayers(0xFF, 0x04);

    Mesh* floor = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        floor->pos.push_back({2.f * half * c.x, 0.f, -2.f * half * c.y});
        floor->nrm.push_back({0, 1, 0});
        floor->uv.push_back(c * 0.5f + 0.5f);
    }
    floor->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* bare = add(s, floor, "floor_shade.filamat", {mat4f{}}, 4, false, 1, 0);   // the bare sun, everywhere, first
    if (!bare) return false;
    bare->setParameter("sunWay", lit.sunWay);
    s.by_sun.push_back(bare);
    bare->setParameter("value", 1.f);
    bare->setParameter("floorY", 0.f);
    Mesh* sheet = new Mesh(*floor);   // (the paper itself reaches further than its picture of sunlight: beyond it the sun is bare)
    for (float3& p : sheet->pos) p = p * 4.f;
    // The paper is drawn once, into the picture of the scene without the glass (layer 16: that picture only); on the
    // screen it is a copy out of that picture (layer 2: the screen only).
    MaterialInstance* paper = add(s, sheet, "paper.filamat", {mat4f{}}, 16);
    MaterialInstance* copy = add(s, sheet, "paper_copy.filamat", {mat4f{}}, 2);
    if (!paper || !copy) return false;
    s.view_behind->setVisibleLayers(0x10, 0x10);
    copy->setParameter("behind", s.behind, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST, TextureSampler::WrapMode::CLAMP_TO_EDGE));
    s.behindRead = true;
    paper->setParameter("color", lit.paper);
    paper->setParameter("grain", lit.grain);
    paper->setParameter("sunlight", s.sunlight, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE));
    paper->setParameter("pictureFromScene", mat4f(above->getProjectionMatrix() * above->getViewMatrix()));
    paper->setParameter("sunWide", lit.sunWide);   // (a sun some 4.5 degrees wide: far wider than the true one, for the softness)
    paper->setParameter("perUnit", 1.f / (2.f * half));
    paper->setParameter("furthest", 2.5f);   // (softened over 0.02 of the floor's units)
    return true;
}

// A glass thing's shade on the paper: its mesh laid flat along the sun's way, in 0, into the picture of sunlight.
static bool shadow_of(Stage& s, const Sunlit& lit, Mesh* m, std::vector<mat4f> places) {
    MaterialInstance* mi = add(s, m, "floor_shade.filamat", places, 4, false, 1, 1);
    if (!mi) return false;
    mi->setParameter("sunWay", lit.sunWay);
    s.by_sun.push_back(mi);
    mi->setParameter("value", 0.f);
    mi->setParameter("floorY", 0.f);
    return true;
}

// The light a glass thing throws on the paper (glasslight.mat): the thing's own mesh at its places, once a colour.
// Exaggerated apart from the look of the glass: the colours' indices far apart, the light a little stronger than true.
// uneven: how far its faces lean from their own facing (0 for a mesh that is uneven itself), over `unevenSize` of the mesh's units.
static MaterialInstance* glass_light(Stage& s, const Sunlit& lit, Mesh* m, std::vector<mat4f> places, float uneven = 0.f, float unevenSize = 1.f) {
    const int colours = 20;
    MaterialInstance* mi = add(s, m, "glasslight.filamat", places, 4, false, colours);
    if (!mi) return nullptr;
    mi->setParameter("sunWay", lit.sunWay);
    s.by_sun.push_back(mi);
    mi->setParameter("colours", (float)colours);
    float size = 0.f;
    for (const float3& p : m->pos) size = std::max(size, std::max(std::fabs(p.x), std::max(std::fabs(p.y), std::fabs(p.z))));
    mi->setParameter("size", size);
    mi->setParameter("ior", 1.5f);
    mi->setParameter("abbe", 6.f);
    mi->setParameter("lean", uneven);
    mi->setParameter("leanSize", unevenSize);
    mi->setParameter("facing", float2{0.6f, 0.74f});
    mi->setParameter("most", 6.f);
    mi->setParameter("gain", 0.9f);
    mi->setParameter("floorY", 0.f);
    return mi;
}

// The light a glass BOX throws on the paper (boxlight.mat): as glass_light, but the light is taken through the box
// itself (in by one face, mirrored inside where it cannot leave, out by another), which a box allows by plain
// formulas. That is what gives a glass box its true shadow: dark nearly all over, since most of its light leaves
// through its underside, with a few small windows of light in it. places: the unit cube's places (a box each).
static bool box_light(Stage& s, const Sunlit& lit, std::vector<mat4f> places, float round) {
    const int colours = 20;
    if (places.size() > 64) return false;
    static float4 poseX[64], poseY[64], poseZ[64], poseAt[64];
    for (size_t i = 0; i < places.size(); ++i) {
        const mat4f& p = places[i];
        poseX[i] = {normalize(p[0].xyz), length(p[0].xyz) * 0.5f};
        poseY[i] = {normalize(p[1].xyz), length(p[1].xyz) * 0.5f};
        poseZ[i] = {normalize(p[2].xyz), length(p[2].xyz) * 0.5f};
        poseAt[i] = {p[3].xyz, 0.f};
    }
    MaterialInstance* mi = add(s, cube(round, 16), "boxlight.filamat", {mat4f{}}, 4, false, (int)places.size() * colours);
    if (!mi) return false;
    mi->setParameter("poseX", poseX, 64);
    mi->setParameter("poseY", poseY, 64);
    mi->setParameter("poseZ", poseZ, 64);
    mi->setParameter("poseAt", poseAt, 64);
    mi->setParameter("sunWay", lit.sunWay);
    s.by_sun.push_back(mi);
    mi->setParameter("colours", (float)colours);
    mi->setParameter("ior", 1.5f);
    mi->setParameter("abbe", 6.f);
    mi->setParameter("round", round);
    mi->setParameter("lean", 0.f);
    mi->setParameter("most", 6.f);
    mi->setParameter("gain", 1.f);
    mi->setParameter("floorY", 0.f);
    return true;
}

// A ball of radius 1.
static Mesh* ball(int around = 20, int up = 12) {
    Mesh* m = new Mesh;
    for (int j = 0; j <= up; ++j)
        for (int i = 0; i <= around; ++i) {
            const float a = TAU * (float)i / around, b = 0.5f * TAU * ((float)j / up - 0.5f);
            const float3 out{std::cos(a) * std::cos(b), std::sin(b), std::sin(a) * std::cos(b)};
            m->pos.push_back(out);
            m->nrm.push_back(out);
            m->uv.push_back({(float)i / around, (float)j / up});
        }
    for (int j = 0; j < up; ++j)
        for (int i = 0; i < around; ++i) {
            const uint32_t p = (uint32_t)(j * (around + 1) + i), q = p + 1, r = p + (uint32_t)around + 1, t = r + 1;
            m->idx.insert(m->idx.end(), {p, r, q, q, r, t});
        }
    return m;
}

// Bubbles of air caught in the glass blocks, as cast blocks have: small balls strewn inside each (most tiny, a few
// larger, some in a rising string), drawn into the picture of what is behind, so a block shows them the way it shows
// everything else: bent, and mirrored inside. each: how many a block. The same every run.
static bool bubbles(Stage& s, std::vector<mat4f> places, int each) {
    uint32_t seed = 11u;
    auto rnd = [&]() { seed = seed * 1664525u + 1013904223u; return (float)(seed >> 8) / 16777216.f; };
    std::vector<mat4f> all;
    for (const mat4f& block : places) {
        float3 at{0.f};
        for (int i = 0; i < each; ++i) {
            if (i % 6 == 0) at = float3{rnd() - 0.5f, rnd() - 0.5f, rnd() - 0.5f} * 0.8f;   // a new string starts
            else at = clamp(at + float3{(rnd() - 0.5f) * 0.06f, 0.03f + rnd() * 0.08f, (rnd() - 0.5f) * 0.06f}, float3{-0.44f}, float3{0.44f});
            const float size = rnd() < 0.15f ? 0.012f + 0.014f * rnd() : 0.003f + 0.006f * rnd();   // (as a part of the block's edge)
            all.push_back(block * mat4f::translation(at) * mat4f::scaling(float3{size} / float3{length(block[0].xyz), length(block[1].xyz), length(block[2].xyz)} * length(block[0].xyz)));
        }
    }
    MaterialInstance* mi = add(s, ball(), "bubble.filamat", all);
    if (!mi) return false;
    mi->setParameter("strength", 1.f);
    return true;
}

// How deep a solid is as a rule: four times its volume over its surface (a ball: two thirds of its width; a bar: its
// width; a sheet: twice its thickness). What the glasses below take their depth from, so that a scene gives none.
static float usual_depth(const Mesh* m) {
    double volume = 0, surface = 0;
    for (size_t i = 0; i + 2 < m->idx.size(); i += 3) {
        const float3 a = m->pos[m->idx[i]], b = m->pos[m->idx[i + 1]], c = m->pos[m->idx[i + 2]];
        volume += dot(a, cross(b, c)) / 6.0;
        surface += length(cross(b - a, c - a)) / 2.0;
    }
    return (float)(4.0 * std::fabs(volume) / surface);
}

// Glass by Filament's own means (own_glass.mat: its refraction, absorption, dispersion and mirroring of the room),
// put down as it comes: its depth is the mesh's own. colour: what white light is left as after a usual depth of
// this glass ({1, 1, 1}: clear). crushed: how far its faces lean this way and that (0: smooth).
// Thin, bent solids (blades, ribbons, sheets) read well in it; it does not show its own far side as thick_glass does.
static MaterialInstance* own_glass(Stage& s, Mesh* m, std::vector<mat4f> places, float3 colour = float3{1.f}, float crushed = 0.f) {
    MaterialInstance* mi = add(s, m, "own_glass.filamat", places, 2);
    if (!mi) return nullptr;
    const float deep = 8.f * usual_depth(m) * length(places[0][0].xyz);   // (overdone eight times: what is behind swims in it)
    mi->setParameter("thickness", deep);
    mi->setParameter("absorb", float3{-std::log(std::max(colour.x, 1e-3f)), -std::log(std::max(colour.y, 1e-3f)), -std::log(std::max(colour.z, 1e-3f))} / deep);
    mi->setParameter("lean", crushed);
    mi->setParameter("film", 0.f);
    mi->setParameter("split", 0.5f);
    return mi;
}

// Glass seen through both of its sides (thick_glass.mat): the mesh at its places, drawn three times. Its far faces go
// into a picture of their own, which way they face and how far away they are (far_side.mat; nothing else is in that
// picture), and as they look from within over the picture of the scene without the glass (far_look.mat);
// its near faces, on the screen, bend the eye's way in, look the far face up there, and bend it out again to the scene.
// The first call makes the picture. Nothing is asked of the caller: the glass is put down as it comes.
static MaterialInstance* thick_glass(Stage& s, Mesh* m, std::vector<mat4f> places) {
    if (!s.view_through) {
        s.through = Texture::Builder().width(s.w).height(s.h).levels(1).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE)
                        .format(Texture::InternalFormat::RGBA16F).build(*s.engine);
        s.through_depth = Texture::Builder().width(s.w).height(s.h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT | Texture::Usage::SAMPLEABLE)
                              .format(Texture::InternalFormat::DEPTH32F).build(*s.engine);
        s.view_through = s.engine->createView();
        s.view_through->setScene(s.scene);
        s.view_through->setCamera(s.camera);
        s.view_through->setViewport({0, 0, (uint32_t)s.w, (uint32_t)s.h});
        s.view_through->setRenderTarget(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, s.through).texture(RenderTarget::AttachmentPoint::DEPTH, s.through_depth).build(*s.engine));
        s.view_through->setPostProcessingEnabled(false);
        s.view_through->setVisibleLayers(0xFF, 0x08);
    }
    MaterialInstance* back = add(s, m, "far_side.filamat", places, 8);
    MaterialInstance* look = add(s, m, "far_look.filamat", places, 16);   // (layer 16: only in the picture of the scene without the glass)
    MaterialInstance* front = add(s, m, "thick_glass.filamat", places, 2);
    if (!back || !look || !front) return nullptr;
    back->setCullingMode(backend::CullingMode::FRONT);
    look->setCullingMode(backend::CullingMode::FRONT);
    s.view_behind->setVisibleLayers(0x10, 0x10);
    const TextureSampler smooth(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
    const TextureSampler exact(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST, TextureSampler::WrapMode::CLAMP_TO_EDGE);
    front->setParameter("behind", s.behind, smooth);
    s.behindRead = true;
    front->setParameter("farSide", s.through, smooth);
    front->setParameter("farDepth", s.through_depth, exact);
    // How deep the solid is as a rule: four times its volume over its surface (a ball: two thirds of its width; a bar:
    // its width; a sheet: twice its thickness). Worked out once, here; a way through the glass much longer than this
    // is the glass seen along itself, and goes dark.
    const float usual = usual_depth(m) * length(places[0][0].xyz);
    // A bar or a sheet has places where it is seen along itself; a lump (a ball, a gem) has none: its longest way is
    // straight through its middle, about one and a half times `usual`. So the nearer to a lump the solid is (how deep
    // against how far it reaches), the longer a way must be before it counts as long: the middle of a gem stays clear.
    float3 mid{0.f};
    for (float3 p : m->pos) mid += p;
    mid = mid / (float)m->pos.size();
    float reach = 0.f;
    for (float3 p : m->pos) reach = std::max(reach, length(p - mid));
    const float lump = usual / (2.f * reach * length(places[0][0].xyz));   // (a ball: 0.67; a cube: 0.38; a ring of bar: about 0.1)
    const float longFrom = 1.f + 0.7f * std::min(std::max((lump - 0.3f) / 0.25f, 0.f), 1.f);
    front->setParameter("ior", 1.5f);
    front->setParameter("thickness", 0.8f * usual);
    front->setParameter("swim", 0.5f);       // (what is behind the glass swims in it, overdone; at 1 letters melt)
    front->setParameter("usual", usual);
    front->setParameter("longFrom", longFrom);
    front->setParameter("dark", 4.f);        // (how fast: at 7 a ring seen edge-on carries two navy bruises; at 3 the darks are pale lilac)
    front->setParameter("edgeFire", 13.f);   // (14: a yellow haze far beside every dark; 20: an orange stain along every edge)
    front->setParameter("split", 0.10f);     // (0.2: the colours turn to blocks)
    front->setParameter("samples", 6.f);     // (each one colour of the rainbow; 8: 0.6 ms dearer)
    front->setParameter("vivid", 1.3f);      // (laid on after the dark, so the rainbow at its edge is the most vivid thing; 1.35 and up: neon)
    front->setParameter("roomGain", 1.f);    // (the glass's own room, studio.glsl: the same wherever the glass is put)
    return front;
}

// A colour of a picture made clear (a green screen): for a sheet that shows a picture (flat.mat). near: how near to
// the colour counts as the screen itself (in the two colour numbers of video, where pure green is 0.6 from grey: the
// screen's own noise must fit in it); soft: over how much more the picture comes in (a point half screen, half grey
// is 0.3 from the screen); spill: how far from the screen its colour is still taken out of what stays.
static void key_out(MaterialInstance* look, float3 colour = float3{0.f, 1.f, 0.f}, float near = 0.12f, float soft = 0.3f, float spill = 0.9f) {
    look->setParameter("key", float4{colour, near});
    look->setParameter("keySoft", float2{soft, spill});
}

// A matte, as After Effects has them: a shape that is not seen itself and cuts other things. Any mesh is one: a solid,
// or a sheet with a picture on it (ThorVG's), which cuts by how clear the picture is, or by its light (After Effects'
// luma matte). It is a thing like any other: `move` moves it.
// on: where the matte is. nullptr: in the film's picture, a window on the screen: it cuts by where its shape is seen
// from the film's camera, whatever lies between, and stays in the picture while things move behind it. A mesh on the
// stage: on that thing, a sticker: it rides on the thing (Filament's own parent and child) and cuts by where its
// shape lies on it, seen along the thing's own z.
// strength: how much it covers at most (a half-clear matte). into: a matte made before, which this shape joins.
// What comes back names the matte. Each matte is a picture of its own, taken before all else (`draw`) by its own eye,
// the film's camera or one riding on the thing, with nothing but its shapes in it; the things it cuts look themselves
// up in it by that eye (matte.glsl), so they are cut the same whichever eye looks at the stage.
static int matte(Stage& s, Mesh* m, std::vector<mat4f> places, Texture* picture = nullptr, bool byLight = false, float strength = 1.f, int into = -1, const Mesh* on = nullptr) {
    auto& tm = s.engine->getTransformManager();
    if (into < 0) {
        Stage::Matte made{};
        uint32_t w = 2 * (uint32_t)s.w, h = 2 * (uint32_t)s.h;   // (in the film's picture: twice as fine as the screen)
        made.eye = s.film;
        made.onPlace = mat4f{};
        for (const Stage::Made& d : s.made) {
            if (!on || d.of != on) continue;
            float3 low = on->pos[0], high = on->pos[0];
            for (const float3& p : on->pos) { low = min(low, p); high = max(high, p); }
            // an eye before the thing, looking along its z at all of it, and going where it goes
            utils::Entity e = utils::EntityManager::get().create();
            tm.create(e, tm.getInstance(d.thing), mat4f::translation(float3{0.f, 0.f, high.z + 1.f}));
            made.eye = s.engine->createCamera(e);
            made.eye->setProjection(Camera::Projection::ORTHO, low.x, high.x, low.y, high.y, 0.1, double(high.z - low.z) + 2.0);
            const float fine = (float)std::max(s.w, s.h) / std::max(high.x - low.x, high.y - low.y);   // (its longer side as fine as the screen's)
            w = 2 * (uint32_t)std::lround(0.5f * fine * (high.x - low.x));
            h = 2 * (uint32_t)std::lround(0.5f * fine * (high.y - low.y));
            made.on = d.thing;
            made.onPlace = d.place;
            break;
        }
        uint8_t levels = 1;
        while ((std::max(w, h) >> levels) > 0) ++levels;
        made.picture = Texture::Builder().width(w).height(h).levels(levels).format(Texture::InternalFormat::R8)
                           .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::GEN_MIPMAPPABLE).build(*s.engine);
        made.scene = s.engine->createScene();
        made.view = s.engine->createView();
        made.view->setScene(made.scene);
        made.view->setCamera(made.eye);
        made.view->setViewport({0, 0, w, h});
        made.view->setRenderTarget(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, made.picture).build(*s.engine));
        made.view->setPostProcessingEnabled(false);
        s.mattes.push_back(made);
        into = (int)s.mattes.size() - 1;
    }
    Stage::Matte& to = s.mattes[(size_t)into];
    MaterialInstance* shape = add(s, m, "matte.filamat", places);
    if (!shape) return -1;
    for (size_t i = s.made.size() - places.size(); i < s.made.size(); ++i) {   // (its shape is in the matte's own scene, and in no other picture)
        s.scene->remove(s.made[i].thing);
        to.scene->addEntity(s.made[i].thing);
        s.made[i].matte = true;
        if (to.eye == s.film) continue;
        s.made[i].place = inverse(to.onPlace) * s.made[i].place;   // (on a thing: its place is told from the thing from now on)
        tm.setParent(tm.getInstance(s.made[i].thing), tm.getInstance(to.on));
        tm.setTransform(tm.getInstance(s.made[i].thing), s.made[i].place);
    }
    shape->setParameter("picture", picture ? picture : white_point(s), TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    shape->setParameter("byLight", byLight ? 1.f : 0.f);
    shape->setParameter("strength", strength);
    if (!picture) shape->setCullingMode(backend::CullingMode::BACK);   // (a solid: its near faces only, so that it covers once)
    return into;
}

// The thing with this look is cut by that matte: seen only inside it (or only outside it). soft: how far the edge is
// blurred, in pixels of the screen (of the matte's own picture, if it is on a thing; 0: as sharp as the matte is). Up to four mattes cut one thing (call again): what
// all of them leave is seen. false: the look's material cannot be cut (it does not read mattes: matte.glsl), or four cut it already.
static bool matted(Stage& s, MaterialInstance* look, int matte, bool inside = true, float soft = 0.f) {
    static const char* const names[] = {"matte0", "matte1", "matte2", "matte3"};
    float4 use{0.f}, softs{0.f};   // (what cuts this look already: the stage's own list says)
    int n = 0;
    for (const Stage::Cut& c : s.cuts)
        if (c.look == look) use[c.slot] = c.use, softs[c.slot] = c.soft, ++n;
    if (n == 4 || !look->getMaterial()->hasParameter("matteUse")) return false;
    look->setParameter(names[n], s.mattes[(size_t)matte].picture,
                       TextureSampler(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE));
    use[n] = inside ? 1.f : -1.f;
    softs[n] = std::log2(1.f + 2.f * std::max(soft, 0.f));   // (a level of the matte's picture: each is twice as coarse as the one before)
    look->setParameter("matteUse", use);
    look->setParameter("matteSoft", softs);
    if (soft > 0.f) s.mattes[(size_t)matte].soft = true;
    s.cuts.push_back({look, n, matte, use[n], softs[n]});
    return true;
}

// Another eye on the stage than the film's camera: on, the stage is looked at through it (`camera`), as one stands beside
// a set and watches; the film's own camera (`film`) goes on taking the mattes that are in the film's picture. Off, the film's again.
static Camera* god_view(Stage& s, bool on) {
    if (on && !s.god) {
        s.god = s.engine->createCamera(utils::EntityManager::get().create());
        s.god->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    }
    if (Camera* eye = on ? s.god : s.film; eye != s.camera) {
        s.camera = eye;
        for (View* v : {s.view, s.view_behind, s.view_through})
            if (v) v->setCamera(s.camera);
    }
    return s.camera;
}

// Three frames, the last one read back and written as a .png.
// One picture of the stage drawn (the helper pictures first), and read back into `pic` if one is given.
// The stage's time set: everything that goes on by itself is at that moment for the picture drawn next.
static void time_is(Stage& s, double seconds) {
    s.now = s.frames > 0.0 ? std::floor(seconds * s.frames + 1e-6) / s.frames : seconds;   // (a document of so many frames a second is seen only at them: Alight Motion's project frame rate, After Effects' composition's)
    seconds = s.now;
    std::vector<const void*> all;   // (each by its key, looked for again as it comes: one may make things again, and take some off, as it goes (doc_moment))
    for (const auto& [who, goes] : s.playing) all.push_back(who);
    for (const void* who : all)
        if (const auto it = s.playing.find(who); it != s.playing.end()) it->second(seconds);
}

// A frame begins (Filament's beginFrame: false if this one is to be left out). What the looks are told is sent on
// as a frame begins, so each matte's eye is told just before: where it is now, a pose handed over, nothing more.
static bool begins(Stage& s) {
    static const char* const eyes[] = {"matteEye0", "matteEye1", "matteEye2", "matteEye3"};
    for (const auto& one : s.aim) one();   // (the things with effects of their own aimed, their looks told: thing_fx.h; as samples/rendertarget.cpp aims its offscreen camera before the frame)
    for (const Stage::Cut& c : s.cuts) {
        const Camera* eye = s.mattes[(size_t)c.matte].eye;
        c.look->setParameter(eyes[c.slot], mat4f(eye->getProjectionMatrix() * eye->getViewMatrix()));
    }
    return s.renderer->beginFrame(s.swap);
}

// All the pictures of one frame, the helper pictures first (between Filament's beginFrame and endFrame). Every way
// of drawing the stage goes through here.
static void pictures_of(Stage& s) {
    if (s.backdropRead) s.renderer->render(s.view_behind);   // (what is behind, for an effect that reads it (screen.h's backdrop): drawn before the effects, those things' own sheets left out of it)
    for (const auto& [of, one] : s.once) one();
    s.once.clear();
    for (const auto& one : s.before) one();   // (the things with effects of their own, each drawn alone first, as FilamentApp draws its offscreen views before every other: thing_fx.h)
    for (Stage::Matte& m : s.mattes) {
        s.renderer->render(m.view);
        if (m.soft) m.picture->generateMipmaps(*s.engine);
    }
    if (s.view_floor) s.renderer->render(s.view_floor);
    if (s.behindRead) s.renderer->render(s.view_behind);   // (the picture of what is behind: only once glass or the paper reads it; a whole pass of every thing's points)
    if (s.view_through) s.renderer->render(s.view_through);
    s.renderer->render(s.view);   // (one view: everything is in the stage's own scene)
    for (const auto& one : s.after) one();
    if (s.over) s.over();
    if (s.lastly) s.lastly();
}

static void draw(Stage& s, std::vector<uint8_t>* pic = nullptr, bool wait = true) {
    while (!begins(s)) {}
    pictures_of(s);
    if (pic)
        s.renderer->readPixels(0, 0, (uint32_t)s.w, (uint32_t)s.h,
                               backend::PixelBufferDescriptor(pic->data(), pic->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
    s.renderer->endFrame();
    if (wait) s.engine->flushAndWait();
}

// Whether the things made of mesh `m` are in the picture (they stay made, and are moved and played as before).
static void seen(Stage& s, const Mesh* m, bool in) {
    for (const Stage::Made& d : s.made)
        if (d.of == m) { if (in) s.scene->addEntity(d.thing); else s.scene->remove(d.thing); }
}

// The things made of mesh `m` taken off the stage for good.
static void take_off(Stage& s, const Mesh* m) {
    auto& looks = s.engine->getRenderableManager();
    MaterialInstance* look = nullptr;
    for (size_t i = s.made.size(); i-- > 0;) {
        if (s.made[i].of != m) continue;
        look = looks.getMaterialInstanceAt(looks.getInstance(s.made[i].thing), 0);
        s.scene->remove(s.made[i].thing);
        s.engine->destroy(s.made[i].thing);
        if (s.made[i].many) s.engine->destroy(s.made[i].many);   // (after the thing that draws by it)
        s.made.erase(s.made.begin() + (long)i);
    }
    if (!look) return;
    std::erase_if(s.cuts, [look](const Stage::Cut& c) { return c.look == look; });   // (no matte's eye is told to it any more)
    s.engine->destroy(look);   // (its look goes with it)
}
