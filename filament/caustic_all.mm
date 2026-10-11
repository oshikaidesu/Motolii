// Many posed glass solids, ONE sun-space capture, one floor caustic map (caustic_tri.h + caustic_all/cap_*.mat).
// Per frame the CPU hands over poses only (one 4x4 per solid); capture, refraction and splatting are GPU.
// Output: out/caustic-all.ppm (top-down floor), cost = saturated throughput of K frames submitted back to back.
// Run with no settings for the picture (out/caustic-all-beauty.ppm, same camera as glass_scene) and the cost.
// Env (all optional): N (40) CAP (256) MAP (1024) DISP (0.15) INTENSITY (7) BLURCM (1.7) SHADOWCM (2.4) FILL (0.55) LUX (8000) BACK (0.35) K (120)
#include <filament/ColorGrading.h>
#include <filament/IndirectLight.h>
#include <filament/LightManager.h>
#include <filament/Skybox.h>
#include <filament/SwapChain.h>
#include <filament/ToneMapper.h>
#include <filament/TransformManager.h>
#include <image/Ktx1Bundle.h>
#include <ktxreader/Ktx1Reader.h>

#include <math/mat4.h>

#include <chrono>
#include <cstdlib>

#include "caustic_tri.h"

using namespace ctri;

static float env_f(const char* k, float d) {
    const char* v = std::getenv(k);
    return v ? (float)std::atof(v) : d;
}

// ------------------------------------------------------------------------------------------ shapes (flat faces)
static void poly(Mesh& m, std::vector<float3> pts, float3 n) {  // convex polygon, points in cyclic order
    uint32_t base = (uint32_t)m.pos.size();
    n = normalize(n);
    for (float3 p : pts) { m.pos.push_back(p); m.nrm.push_back(n); }
    for (uint32_t i = 1; i + 1 < pts.size(); ++i) { m.idx.push_back(base); m.idx.push_back(base + i); m.idx.push_back(base + i + 1); }
}
// Cube with bevelled edges and corners: the slanted faces are what make rainbows (a plain cube has parallel faces).
static Mesh chamfer_cube(float h, float c) {
    Mesh m;
    const float a = h - c;
    for (int ax = 0; ax < 3; ++ax)
        for (float s : {-1.f, 1.f}) {
            int u = (ax + 1) % 3, v = (ax + 2) % 3;
            auto P = [&](float su, float sv) { float3 p; p[ax] = s * h; p[u] = su * a; p[v] = sv * a; return p; };
            float3 n{0.f}; n[ax] = s;
            poly(m, {P(1, 1), P(-1, 1), P(-1, -1), P(1, -1)}, n);
        }
    for (int ax = 0; ax < 3; ++ax)
        for (int bx = ax + 1; bx < 3; ++bx)
            for (float sa : {-1.f, 1.f})
                for (float sb : {-1.f, 1.f}) {
                    int t = 3 - ax - bx;
                    auto P = [&](float ea, float eb, float st) { float3 p; p[ax] = sa * ea; p[bx] = sb * eb; p[t] = st * a; return p; };
                    float3 n{0.f}; n[ax] = sa; n[bx] = sb;
                    poly(m, {P(h, a, 1), P(h, a, -1), P(a, h, -1), P(a, h, 1)}, n);
                }
    for (float sx : {-1.f, 1.f})
        for (float sy : {-1.f, 1.f})
            for (float sz : {-1.f, 1.f})
                poly(m, {{sx * h, sy * a, sz * a}, {sx * a, sy * h, sz * a}, {sx * a, sy * a, sz * h}}, {sx, sy, sz});
    orient_outward(m);
    return m;
}
// Triangular prism lying along z: the classic dispersing shape.
static Mesh prism(float r, float len) {
    Mesh m;
    float3 t[3];
    for (int i = 0; i < 3; ++i) { float a = 2.f * PI * (float)i / 3.f + PI * 0.5f; t[i] = {r * std::cos(a), r * std::sin(a), 0.f}; }
    float hz = len * 0.5f;
    for (int i = 0; i < 3; ++i) {
        float3 p = t[i], q = t[(i + 1) % 3];
        float3 mid = (p + q) * 0.5f;
        poly(m, {{p.x, p.y, hz}, {p.x, p.y, -hz}, {q.x, q.y, -hz}, {q.x, q.y, hz}}, mid);
    }
    poly(m, {{t[0].x, t[0].y, hz}, {t[1].x, t[1].y, hz}, {t[2].x, t[2].y, hz}}, {0, 0, 1});
    poly(m, {{t[0].x, t[0].y, -hz}, {t[1].x, t[1].y, -hz}, {t[2].x, t[2].y, -hz}}, {0, 0, -1});
    orient_outward(m);
    return m;
}

static int fail(const char* msg) {
    std::printf("FAIL %s\n", msg);
    return 1;
}

int main() {
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* r = engine->createRenderer();
    SwapChain* sc = engine->createSwapChain(64, 64, 0);

    Materials M;
    M.front = load_material(engine, "caustic_all/cap_front.filamat");
    M.back = load_material(engine, "caustic_all/cap_back.filamat");
    M.hits = load_material(engine, "caustic_tri/hits.filamat");
    M.tri = load_material(engine, "caustic_tri/tri.filamat");
    if (env_f("GRID", 0) > 0.5f) M.grid = load_material(engine, "caustic_all/grid.filamat");
    if (!M.front || !M.back || !M.hits || !M.tri) return fail("materials (run caustic_all/build.sh and caustic_tri/build.sh)");

    const int N = (int)env_f("N", 30);
    const int K = (int)env_f("K", 120);
    Params P;
    P.cap = (int)env_f("CAP", 256);
    P.map = (int)env_f("MAP", 1024);
    P.dispersion = env_f("DISP", 0.05f);   // kept small enough that the six colour steps run together under the softening
    P.intensity = env_f("INTENSITY", 7.f);
    P.mips = env_f("MIPS", 1) > 0.5f;
    float3 sun = normalize(float3{env_f("SUNX", 0.92f), -env_f("SUNY", 0.42f), env_f("SUNZ", 0.22f)});   // glass_scene's sun

    // one capture window over the whole play area (fixed: no per-frame bounds work)
    const float spread = env_f("SPREAD", 1.1f), area = spread + 0.3f, top = 0.6f;
    P.rsize = 2.f * (area + 0.6f);
    P.originOverride = float3{0, top * 0.5f, 0};
    P.extentOverride = std::sqrt(2.f * area * area + 0.25f * top * top) + 0.05f;   // bounding sphere of the play volume

    std::vector<Mesh> kinds{chamfer_cube(0.11f, 0.22f * env_f("CHAM", 0.10f)), prism(0.13f, 0.26f), pebble({0, 0, 0}, {0.12f, 0.12f, 0.12f}, 0.f)};
    std::vector<int> copies{(N + 2) / 3, (N + 1) / 3, N / 3};
    CausticPass cp(engine, M, kinds, sun, P, copies);
    cp.skipMask = (int)env_f("SKIP", 0);

    auto& tm = engine->getTransformManager();
    // solid i sits on a jittered grid and tumbles; the only per-frame CPU work is these matrices
    const int side = (int)std::ceil(std::sqrt((double)N));
    // Still picture: every solid rests on the floor on a jittered grid (kinds mixed), turned about the upright axis,
    // sizes 0.6x to 1.2x. TUMBLE=1 gives the old hovering tumble instead.
    const bool tumble = env_f("TUMBLE", 0) > 0.5f;
    int mixStep = 7;
    for (int p : {7, 11, 13, 17}) if (N % p != 0) { mixStep = p; break; }
    auto pose = [&](int i, float t) {
        int slot = (i * mixStep + 3) % N;
        float gx = ((float)(slot % side) + 0.5f) / side * 2.f - 1.f, gz = ((float)(slot / side) + 0.5f) / side * 2.f - 1.f;
        float ph = (float)i * 2.399f;
        auto h01 = [&](float k) { float s = std::sin(ph * k) * 43758.5453f; return s - std::floor(s); };
        float3 at{gx * spread + 0.05f * std::sin(ph * 3.f), 0.f, gz * spread + 0.05f * std::cos(ph * 5.f)};
        if (tumble) {
            float3 axis = normalize(float3{std::sin(ph), std::cos(ph * 1.7f), std::sin(ph * 2.3f) + 1.5f});
            return mat4f::translation(at + float3{0, 0.22f + 0.05f * std::sin(t * 1.3f + ph), 0}) * mat4f::rotation(t * (0.6f + 0.5f * std::fmod(ph, 1.f)) + ph, axis);
        }
        int kind = i < copies[0] ? 0 : (i < copies[0] + copies[1] ? 1 : 2);
        float sc = 0.6f + 0.6f * h01(1.3f);
        float restY = (kind == 0 ? 0.11f : (kind == 1 ? 0.065f : 0.12f)) * sc;   // cube half edge, prism's flat side, ball radius
        return mat4f::translation(at + float3{0, restY, 0}) * mat4f::rotation(h01(2.1f) * 6.2832f + t * 0.3f, float3{0, 1, 0}) * mat4f::scaling(sc);
    };

    // ---- The picture: the same solids seen by the camera of glass_scene, with its floor, sky, studio light and grading.
    // BEAUTY=0 skips it (cost of the light passes only).
    const bool beautyOn = env_f("BEAUTY", 1) > 0.5f;
    // DARK: one number for how dark the room is (0 = the pale room of glass_scene, 1 = dark): floor, sky, fill light and
    // the room the glass mirrors all go down together; the sun stays as it is.
    const float dark = env_f("DARK", 0.f);
    auto dk = [&](float pale, float night) { return pale + (night - pale) * dark; };
    const int W = 1920, H = 1080;
    std::vector<utils::Entity> bodies, backs;
    utils::Entity sunE;
    std::vector<MaterialInstance*> solidMis;
    View* vBeauty = nullptr;
    View* vBackdrop = nullptr;   // the scene without the glass, read by the old body (glass_solid.mat)
    RT beautyRT, backdropRT;
    const bool oldBody = env_f("OLDBODY", 1) > 0.5f;   // 1: glass_solid.mat as it is (the old picture's body); 0: Filament's own glass
    if (beautyOn) {
        auto& em = utils::EntityManager::get();
        Material* glassMat = load_material(engine, "glass_shadow.filamat");
        Material* floorMat = load_material(engine, "caustic_all/floor_caustic.filamat");
        Material* backMat = load_material(engine, "caustic_all/glass_back.filamat");
        if (!glassMat || !floorMat || !backMat) return fail("beauty materials");
        MaterialInstance* back = backMat->createInstance();
        back->setParameter("tint", float3{env_f("BACK", 0.35f)});
        back->setParameter("roughness", env_f("BACKROUGH", 0.03f));
        back->setCullingMode(backend::CullingMode::FRONT);
        Scene* bscene = engine->createScene();
        auto frame = [](float3 n) {
            float3 a = std::fabs(n.y) < 0.9f ? float3{0, 1, 0} : float3{1, 0, 0};
            float3 T = normalize(cross(a, n)), B = cross(n, T);
            return mat3f::packTangentFrame(mat3f{T, B, n});
        };
        auto lit_buffers = [&](const std::vector<float3>& pos, const std::vector<float3>& nrm, const std::vector<uint32_t>& idx) {
            float* v = new float[pos.size() * 7];
            for (size_t i = 0; i < pos.size(); ++i) {
                quatf q = frame(nrm[i]);
                float f[7] = {pos[i].x, pos[i].y, pos[i].z, q.x, q.y, q.z, q.w};
                std::memcpy(v + i * 7, f, 28);
            }
            VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)pos.size()).bufferCount(1)
                                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28).build(*engine);
            vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(v, pos.size() * 28, [](void* p, size_t, void*) { delete[] (float*)p; }));
            IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
            uint32_t* d = new uint32_t[idx.size()];
            std::memcpy(d, idx.data(), idx.size() * 4);
            ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(d, idx.size() * 4, [](void* p, size_t, void*) { delete[] (uint32_t*)p; }));
            return std::pair<VertexBuffer*, IndexBuffer*>{vb, ib};
        };
        MaterialInstance* glass = glassMat->createInstance();
        glass->setParameter("ior", 1.55f);
        glass->setParameter("transmission", 1.f);
        glass->setParameter("dispersion", env_f("GLASSDISP", 36.f));
        glass->setParameter("reflectance", 0.5f);
        glass->setParameter("alpha", 1.f);
        glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
        glass->setParameter("thickness", 0.22f);
        // The old body, unchanged: per kind its list of faces (xyz outward normal, w offset, in the mesh's own space;
        // the body shader divides by the transform's scale) or, for the ball, its radius. Glass seen through glass is
        // off (solidCount 0).
        if (oldBody) {
            Material* solidMat = load_material(engine, "glass_solid.filamat");
            if (!solidMat) return fail("glass_solid.filamat");
            Texture* dummy = Texture::Builder().width(8).height(1).levels(1).format(Texture::InternalFormat::RGBA32F).build(*engine);
            TextureSampler ns(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
            const float disp = env_f("BODYDISP", 10.f);
            for (size_t k = 0; k < kinds.size(); ++k) {
                const bool ball = k == 2;
                std::vector<float4> planes;
                if (!ball)
                    for (size_t i = 0; i < kinds[k].idx.size(); i += 3) {
                        float3 n = kinds[k].nrm[kinds[k].idx[i]];
                        float o = dot(n, kinds[k].pos[kinds[k].idx[i]]);
                        bool seen = false;
                        for (float4 p : planes) seen = seen || (dot(p.xyz, n) > 0.9999f && std::fabs(p.w - o) < 1e-5f);
                        if (!seen) planes.push_back(float4{n, o});
                    }
                if (planes.size() > 40) return fail("a solid has more than 40 faces");
                const int32_t count = (int32_t)planes.size();
                planes.resize(40, float4{0, 1, 0, 1e6f});
                MaterialInstance* mi = solidMat->createInstance();
                mi->setParameter("ior", 1.5168f);
                mi->setParameter("abbe", 64.17f / disp);
                mi->setParameter("planes", planes.data(), 40);
                mi->setParameter("planeCount", count);
                mi->setParameter("kind", (int32_t)(ball ? 1 : 0));
                mi->setParameter("dims", float4{0.12f, 0, 0, 0});
                mi->setParameter("scene", dummy, ns);
                mi->setParameter("shapes", dummy, ns);
                mi->setParameter("solidCount", (int32_t)0);
                mi->setParameter("self", (int32_t)0);
                mi->setParameter("nearCone", 0.f);
                mi->setParameter("spread", env_f("BODYSPREAD", 2.f));
                mi->setParameter("sunTo", -sun);
                mi->setParameter("sunGlint", float3{1, 0.98f, 0.96f} * env_f("GLINT", 4.0e5f));
                mi->setParameter("sunCos", std::cos(2.0f * PI / 180.f));
                mi->setParameter("floorY", 0.f);
                mi->setParameter("envGain", 0.f);
                mi->setParameter("sky", float3{0.95f, 0.97f, 1.0f} * (1800.f * dk(1.f, 0.05f)));
                mi->setParameter("absorption", float3{0.9f, 0.35f, 0.5f} * 0.15f);
                mi->setParameter("roomCount", 0.f);
                mi->setParameter("blockCount", 0.f);
                solidMis.push_back(mi);
            }
        }
        for (size_t k = 0; k < kinds.size(); ++k) {
            auto [vb, ib] = lit_buffers(kinds[k].pos, kinds[k].nrm, kinds[k].idx);
            for (int c = 0; c < copies[k]; ++c) {
                utils::Entity e = em.create();
                RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {0.6f, 0.6f, 0.6f}})
                    .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                    .material(0, oldBody ? solidMis[k] : glass).culling(false).castShadows(false).receiveShadows(false)
                    .layerMask(0xFF, oldBody ? 0x02 : 0x01).build(*engine, e);   // layer 2: left out of the backdrop picture
                engine->getTransformManager().create(e);
                bscene->addEntity(e);
                bodies.push_back(e);
                if (!oldBody && env_f("BACK", 0.35f) > 0.f) {  // inner faces, drawn after the body (priority) and added on top
                    utils::Entity b = em.create();
                    RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {0.6f, 0.6f, 0.6f}})
                        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                        .material(0, back).priority(6).culling(false).castShadows(false).receiveShadows(false).build(*engine, b);
                    engine->getTransformManager().create(b);
                    bscene->addEntity(b);
                    backs.push_back(b);
                }
            }
        }
        {   // floor, 6 m square, brightness from the two maps
            const float F = 3.f;
            std::vector<float3> fp{{-F, 0, -F}, {F, 0, -F}, {F, 0, F}, {-F, 0, F}}, fn(4, float3{0, 1, 0});
            auto [vb, ib] = lit_buffers(fp, fn, {0, 2, 1, 0, 3, 2});
            MaterialInstance* fmi = floorMat->createInstance();
            TextureSampler lin(P.mips ? TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR : TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
            // softening in centimetres on the floor, turned into a mip level of the map (so it does not depend on the map size)
            const float texelCm = P.rsize / (float)P.map * 100.f;
            fmi->setParameter("blurLod", P.mips ? std::max(0.f, std::log2(env_f("BLURCM", 1.7f) / texelCm)) : 0.f);
            fmi->setParameter("sharpMix", env_f("SHARP", 0.f));
            fmi->setParameter("coverLod", P.mips ? std::max(0.f, std::log2(env_f("SHADOWCM", 2.4f) / texelCm)) : 0.f);
            fmi->setParameter("color", float4{float3{0.86f, 0.85f, 0.84f} * dk(1.f, 0.14f), 1});
            fmi->setParameter("roughness", 0.42f);
            fmi->setParameter("mapCenter", float2{0, 0});
            fmi->setParameter("mapSize", P.rsize);
            fmi->setParameter("lightNits", env_f("LUX", 8000.f) / PI);
            fmi->setParameter("fill", env_f("FILL", dk(0.55f, 0.12f)));
            fmi->setParameter("causticGain", env_f("CGAIN", 1.f));
            fmi->setParameter("caustic", cp.caustic(), lin);
            fmi->setParameter("cover", cp.footprint(), lin);
            utils::Entity e = em.create();
            RenderableManager::Builder(1).boundingBox({{0, 0, 0}, {4.f, 4.f, 4.f}})
                .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
                .material(0, fmi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
            bscene->addEntity(e);
        }
        sunE = em.create();
        LightManager::Builder(LightManager::Type::SUN).color(float3{1, 0.98f, 0.96f}).intensity(env_f("LUX", 8000.f))
            .direction(sun).sunAngularRadius(0.35f).sunHaloSize(1.2f).castShadows(false).build(*engine, sunE);
        bscene->addEntity(sunE);
        bscene->setSkybox(Skybox::Builder().color({0.72f * dk(1.f, 0.04f), 0.74f * dk(1.f, 0.04f), 0.78f * dk(1.f, 0.05f), 1}).build(*engine));
        {   // studio environment, as in glass_scene
            auto ib = load_file("out/studio/_ibl.ktx");
            std::ifstream shf("out/studio/sh.txt");
            std::vector<float3> shv;
            std::string line;
            while (std::getline(shf, line)) {
                size_t l = line.find('('), r2 = line.find(')');
                float x, y, z;
                if (l != std::string::npos && r2 != std::string::npos && std::sscanf(line.c_str() + l + 1, "%f,%f,%f", &x, &y, &z) == 3) shv.push_back({x, y, z});
            }
            if (ib.empty() || shv.size() < 9) return fail("studio environment (out/studio)");
            Texture* ibl = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ib.data(), (uint32_t)ib.size()), false);
            TextureSampler cubeS(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
            for (MaterialInstance* mi : solidMis) mi->setParameter("env", ibl, cubeS);
            bscene->setIndirectLight(IndirectLight::Builder().reflections(ibl).irradiance(3, shv.data()).intensity(env_f("IBL", dk(900.f, 150.f))).build(*engine));
        }
        Camera* bcam = engine->createCamera(em.create());
        bcam->setExposure(env_f("APERTURE", 4.0f), 1.f / 125.f, 100.f);
        float cam[7] = {1.55f, 1.45f, 2.15f, 0.10f, 0.0f, -0.05f, 32.f};
        if (const char* c = std::getenv("CAM")) std::sscanf(c, "%f %f %f %f %f %f %f", &cam[0], &cam[1], &cam[2], &cam[3], &cam[4], &cam[5], &cam[6]);
        bcam->lookAt(float3{cam[0], cam[1], cam[2]}, float3{cam[3], cam[4], cam[5]}, float3{0, 1, 0});
        bcam->setProjection((double)cam[6], double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);
        beautyRT = make_rt(engine, W, H, Texture::InternalFormat::RGBA8, true);
        if (oldBody) {  // as glass_scene: same scene and camera, the glass layer left out, no post-processing, float
            backdropRT = make_rt(engine, W, H, Texture::InternalFormat::RGBA16F, true);
            TextureSampler blin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, TextureSampler::WrapMode::CLAMP_TO_EDGE);
            for (MaterialInstance* mi : solidMis) mi->setParameter("backdrop", backdropRT.color, blin);
            vBackdrop = engine->createView();
            vBackdrop->setScene(bscene);
            vBackdrop->setCamera(bcam);
            vBackdrop->setRenderTarget(backdropRT.rt);
            vBackdrop->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
            vBackdrop->setVisibleLayers(0xFF, 0x01);
            vBackdrop->setDithering(View::Dithering::NONE);
            vBackdrop->setPostProcessingEnabled(false);
            vBackdrop->setShadowingEnabled(false);
        }
        vBeauty = engine->createView();
        vBeauty->setScene(bscene);
        vBeauty->setVisibleLayers(0xFF, 0x03);
        vBeauty->setCamera(bcam);
        vBeauty->setRenderTarget(beautyRT.rt);
        vBeauty->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        vBeauty->setDithering(View::Dithering::NONE);
        static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
        vBeauty->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
        vBeauty->setPostProcessingEnabled(true);
        vBeauty->setScreenSpaceRefractionEnabled(true);
        vBeauty->setShadowingEnabled(false);
        BloomOptions bloom{};
        bloom.enabled = true;
        bloom.strength = env_f("BLOOM", 0.10f);
        bloom.levels = 6;
        bloom.resolution = 1080;
        bloom.threshold = true;
        bloom.highlight = 10.f;
        bloom.blendMode = BloomOptions::BlendMode::ADD;
        vBeauty->setBloomOptions(bloom);
        vBeauty->setAntiAliasing(View::AntiAliasing::FXAA);
    }
    std::vector<mat4f> simPose;   // filled by the video's bounce loop; empty = the analytic tumble above
    auto set_poses = [&](float t) {
        int i = 0;
        for (auto& o : cp.objects())
            for (size_t k = 0; k < o.ents.size() / 2; ++k, ++i) {
                mat4f m = simPose.empty() ? pose(i, t) : simPose[i];
                tm.setTransform(tm.getInstance(o.ents[k]), m);                      // front copy
                tm.setTransform(tm.getInstance(o.ents[o.ents.size() / 2 + k]), m);  // back copy
                if (beautyOn) {
                    tm.setTransform(tm.getInstance(bodies[i]), m);
                    if (!backs.empty()) tm.setTransform(tm.getInstance(backs[i]), m);
                }
            }
    };
    auto open_frame = [&]() {
        auto t0 = std::chrono::steady_clock::now();
        while (std::chrono::steady_clock::now() - t0 < std::chrono::seconds(3)) if (r->beginFrame(sc)) return true;
        return false;
    };


    // ---- VIDEO=n: n frames at 60 fps to out/caustic-all.mp4. The sun goes once round; the solids fly about and bounce off
    // each other, the floor and an invisible box (round bounds, equal weights: lively, not physics).
    if (const int frames = (int)env_f("VIDEO", 0); frames > 0 && beautyOn) {
        struct Body { float3 p, v, axis; float ang, spin, r, floorR; };   // floorR: the furthest corner, so nothing dips into the floor
        std::vector<Body> bs(N);
        auto rnd = [seed = 12345u]() mutable { seed = seed * 1664525u + 1013904223u; return (float)(seed >> 8) / 16777216.f; };
        for (int i = 0; i < N; ++i) {
            float gx = ((float)(i % side) + 0.5f) / side * 2.f - 1.f, gz = ((float)(i / side) + 0.5f) / side * 2.f - 1.f;
            bs[i].p = {gx * spread, 0.2f + 0.3f * rnd(), gz * spread};
            bs[i].v = float3{rnd() - 0.5f, rnd() - 0.2f, rnd() - 0.5f} * 1.6f;
            bs[i].axis = normalize(float3{rnd() - 0.5f, rnd() - 0.5f, rnd() - 0.5f} + float3{0, 0.01f, 0});
            bs[i].ang = rnd() * 6.28f;
            bs[i].spin = (rnd() - 0.5f) * 5.f;
            bs[i].r = 0.14f;
            bs[i].floorR = i < copies[0] ? 0.19f : (i < copies[0] + copies[1] ? 0.20f : 0.12f);
        }
        const float dt = 1.f / 60.f, wall = spread + 0.1f, ceil = top - 0.12f, g = 2.2f;
        auto step = [&]() {
            for (Body& b : bs) {
                b.v.y -= g * dt;
                b.p += b.v * dt;
                b.ang += b.spin * dt;
                if (b.p.y < b.floorR) { b.p.y = b.floorR; b.v.y = std::fabs(b.v.y) * 0.98f + 0.25f; }   // the floor gives a little back, so it never settles
                if (b.p.y > ceil) { b.p.y = ceil; b.v.y = -std::fabs(b.v.y); }
                for (int a : {0, 2})
                    if (std::fabs(b.p[a]) > wall) { b.p[a] = wall * (b.p[a] > 0 ? 1.f : -1.f); b.v[a] = -b.v[a]; }
            }
            for (int i = 0; i < N; ++i)
                for (int j = i + 1; j < N; ++j) {
                    float3 d = bs[j].p - bs[i].p;
                    float dist = length(d), need = bs[i].r + bs[j].r;
                    if (dist >= need || dist < 1e-5f) continue;
                    float3 n = d / dist;
                    float rel = dot(bs[j].v - bs[i].v, n);
                    bs[i].p -= n * (0.5f * (need - dist));
                    bs[j].p += n * (0.5f * (need - dist));
                    if (rel < 0) {   // swap the speeds along the line between them, and knock the spins
                        bs[i].v += n * rel;
                        bs[j].v -= n * rel;
                        bs[i].spin = -bs[i].spin * 0.9f + rel;
                        bs[j].spin = -bs[j].spin * 0.9f - rel;
                    }
                }
        };
        FILE* ff = popen("ffmpeg -loglevel error -y -f rawvideo -pix_fmt rgba -s 1920x1080 -r 60 -i - -c:v libx264 -pix_fmt yuv420p -crf 17 out/caustic-all.mp4", "w");
        if (!ff) return fail("ffmpeg");
        std::vector<uint8_t> px((size_t)W * H * 4);
        simPose.resize(N);
        auto& lm = engine->getLightManager();
        const float elev = std::asin(0.42f), az0 = std::atan2(0.22f, 0.92f);
        for (int f = -1; f < frames; ++f) {   // (frame -1 is the warm-up, see the still picture)
            step();
            for (int i = 0; i < N; ++i) simPose[i] = mat4f::translation(bs[i].p) * mat4f::rotation(bs[i].ang, bs[i].axis);
            set_poses(0);
            float az = az0 + 2.f * PI * (float)std::max(f, 0) / (float)frames;
            float3 s{std::cos(elev) * std::cos(az), -std::sin(elev), std::cos(elev) * std::sin(az)};
            cp.setSun(s);
            lm.setDirection(lm.getInstance(sunE), s);
            for (MaterialInstance* mi : solidMis) mi->setParameter("sunTo", -s);
            if (!open_frame()) return fail("beginFrame");
            cp.draw(r);
            if (vBackdrop) r->render(vBackdrop);
            r->render(vBeauty);
            if (f >= 0) r->readPixels(beautyRT.rt, 0, 0, W, H, backend::PixelBufferDescriptor(px.data(), px.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
            r->endFrame();
            engine->flushAndWait();
            if (f >= 0) std::fwrite(px.data(), 1, px.size(), ff);
        }
        pclose(ff);
        std::printf("wrote out/caustic-all.mp4 (%d frames)\n", frames);
        return 0;
    }
    // ---- the picture (frame at t = 0.7)
    std::vector<float> caustic((size_t)P.map * P.map * 4), cover((size_t)P.map * P.map * 4);
    set_poses(0.7f);
    // warm-up frame: Filament (Metal) lets a view read freshly generated mip levels only from the second frame on
    if (!open_frame()) return fail("beginFrame");
    cp.draw(r);
    r->endFrame();
    engine->flushAndWait();
    if (!open_frame()) return fail("beginFrame");
    cp.draw(r);
    r->readPixels(cp.causticTarget(), 0, 0, P.map, P.map, backend::PixelBufferDescriptor(caustic.data(), caustic.size() * 4, backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    r->readPixels(cp.footprintTarget(), 0, 0, P.map, P.map, backend::PixelBufferDescriptor(cover.data(), cover.size() * 4, backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
    std::vector<uint8_t> beautyPx;
    if (beautyOn) {
        beautyPx.assign((size_t)W * H * 4, 0);
        if (vBackdrop) r->render(vBackdrop);
        r->render(vBeauty);
        r->readPixels(beautyRT.rt, 0, 0, W, H, backend::PixelBufferDescriptor(beautyPx.data(), beautyPx.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
    }
    r->endFrame();
    engine->flushAndWait();
    if (beautyOn) {
        const bool flip = env_f("FLIP", 0) > 0.5f;
        const char* bn = std::getenv("BOUT") ? std::getenv("BOUT") : "out/caustic-all-beauty.ppm";
        FILE* bf = std::fopen(bn, "wb");
        std::fprintf(bf, "P6\n%d %d\n255\n", W, H);
        for (int y = 0; y < H; ++y)
            for (int x = 0; x < W; ++x) {
                const uint8_t* px = &beautyPx[((size_t)(flip ? H - 1 - y : y) * W + x) * 4];
                std::fwrite(px, 1, 3, bf);
            }
        std::fclose(bf);
        std::printf("wrote %s\n", bn);
    }

    // ---- cost: K frames back to back, one wait at the end (throughput, includes the pose matrices)
    for (int w = 0; w < 10; ++w) { set_poses(w * 0.016f); if (open_frame()) { cp.draw(r); r->endFrame(); } }
    engine->flushAndWait();
    const bool fullTime = env_f("FULL", 0) > 0.5f;        // also draw the 1080p picture every frame
    const bool fullSkipLight = env_f("NOLIGHT", 0) > 0.5f;  // measuring: leave the light passes out
    auto a = std::chrono::steady_clock::now();
    int done = 0;
    for (int f = 0; f < K; ++f) {
        set_poses(1.f + f / 60.f);
        if (!open_frame()) break;
        if (!(fullSkipLight)) cp.draw(r);
        if (fullTime && beautyOn) {
            if (vBackdrop) r->render(vBackdrop);
            r->render(vBeauty);
        }
        r->endFrame();
        ++done;
    }
    engine->flushAndWait();
    double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - a).count() / std::max(1, done);
    std::printf("TIME %d solids, capture %d, map %d%s: %.2f ms per frame (throughput of %d frames, poses included)\n", N, P.cap, P.map, fullTime ? " + 1080p picture" : "", ms, done);

    const int m = P.map;
    const float exposure = env_f("EXPOSURE", 0.9f);
    const char* outName = std::getenv("OUT") ? std::getenv("OUT") : "out/caustic-all.ppm";
    FILE* fp = std::fopen(outName, "wb");
    if (!fp) return fail("ppm");
    std::fprintf(fp, "P6\n%d %d\n255\n", m, m);
    double maxY = 0;
    for (int i = 0; i < m * m; ++i) {
        float x = caustic[(size_t)i * 4], y = caustic[(size_t)i * 4 + 1], z = caustic[(size_t)i * 4 + 2];
        float cv = std::clamp(cover[(size_t)i * 4], 0.f, 1.f);
        maxY = std::max<double>(maxY, y);
        float lin[3] = {3.2409699f * x - 1.5373832f * y - 0.4986108f * z, -0.9692436f * x + 1.8759675f * y + 0.0415551f * z, 0.0556301f * x - 0.2039770f * y + 1.0569715f * z};
        for (int ch = 0; ch < 3; ++ch) {
            const float fill = env_f("FILL", 0.55f), gain = env_f("GAIN", 1.6f);
            float sunTerm = (1.f - cv) + std::max(lin[ch], 0.f) * exposure;
            float v = gain * (fill + (1.f - fill) * sunTerm);
            float t = v / (1.f + v * 0.25f);
            float s = t <= 0.0031308f ? 12.92f * t : 1.055f * std::pow(t, 1.f / 2.4f) - 0.055f;
            std::fputc((int)(std::clamp(s, 0.f, 1.f) * 255.f), fp);
        }
    }
    std::fclose(fp);
    std::printf("map Y max %.3f; wrote out/caustic-all.ppm\n", maxY);
    return 0;
}
