// Real-time spectral caustics inside Filament only: no compute, no extra GPU API, no wgpu.
//   1. capture   sun-space front/back faces of each glass object (unlit materials, data in the colour)
//   2. hits      one fullscreen pass: where does each refracted ray land on the floor, per wavelength band
//   3. triangles every light-view cell is drawn as a triangle at its refracted corners, brightness =
//                oldArea / newArea (Wallace / Renou area ratio, energy conserving), additive RGBA16F
// Method, spectral constants and ray-march: visgl/luma.gl (MIT) + area-ratio triangles (see caustic_tri/README.md).
//
// Use: build a CausticPass once, then inside every frame (between beginFrame and endFrame) call draw() before
// rendering the view that shows the floor. caustic() is an RGBA16F texture, rgb = CIE XYZ of the floor
// irradiance relative to unrefracted sunlight (1.0 = what the floor gets without glass); footprint() r = 1 under the glass.
#pragma once
#include <filament/Camera.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/RenderableManager.h>
#include <filament/Renderer.h>
#include <filament/RenderTarget.h>
#include <filament/Scene.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>

#include <math/vec3.h>
#include <math/vec4.h>

#include <utils/EntityManager.h>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

namespace ctri {
using namespace filament;
using namespace filament::math;

inline const float PI = 3.14159265f;
// ------------------------------------------------------------------------------------------ geometry
struct Mesh {
    std::vector<float3> pos, nrm;
    std::vector<uint32_t> idx;
};
inline float3 rodrigues(float3 v, float3 axis, float a) {
    return v * std::cos(a) + cross(axis, v) * std::sin(a) + axis * (dot(axis, v) * (1.f - std::cos(a)));
}
inline float3 carry(float3 v, float3 ta, float3 tb) {
    float3 axis = cross(ta, tb);
    if (length(axis) < 1e-6f) return v;
    return rodrigues(v, normalize(axis), std::acos(std::clamp(dot(ta, tb), -1.f, 1.f)));
}
// Makes every triangle counter-clockwise seen from outside, using the vertex normals as "outside".
inline void orient_outward(Mesh& m) {
    for (size_t i = 0; i + 2 < m.idx.size(); i += 3) {
        float3 a = m.pos[m.idx[i]], b = m.pos[m.idx[i + 1]], c = m.pos[m.idx[i + 2]];
        float3 n = m.nrm[m.idx[i]] + m.nrm[m.idx[i + 1]] + m.nrm[m.idx[i + 2]];
        if (dot(cross(b - a, c - a), n) < 0) std::swap(m.idx[i + 1], m.idx[i + 2]);
    }
}
inline std::vector<float3> catmull_closed(const std::vector<float3>& p, int per_span) {
    int n = (int)p.size();
    auto at = [&](int i) { return p[((i % n) + n) % n]; };
    std::vector<float3> out;
    for (int i = 0; i < n; ++i) {
        float3 p0 = at(i), p3 = at(i + 1);
        float3 p1 = p0 + (at(i + 1) - at(i - 1)) * (1.f / 6.f);
        float3 p2 = p3 - (at(i + 2) - at(i)) * (1.f / 6.f);
        for (int s = 0; s < per_span; ++s) {
            float t = (float)s / per_span, u = 1.f - t;
            out.push_back(p0 * (u * u * u) + p1 * (3.f * u * u * t) + p2 * (3.f * u * t * t) + p3 * (t * t * t));
        }
    }
    return out;
}
// Circle swept along a closed path with parallel transport; the leftover twist is spread over the loop.
inline Mesh tube(const std::vector<float3>& path, int sides, float radius) {
    size_t n = path.size();
    auto tangent = [&](size_t i) { return normalize(path[(i + 1) % n] - path[(i + n - 1) % n]); };
    std::vector<float3> ns(n);
    float3 t0 = tangent(0);
    float3 helper = std::fabs(t0.y) < 0.9f ? float3{0, 1, 0} : float3{1, 0, 0};
    ns[0] = normalize(helper - t0 * dot(helper, t0));
    for (size_t i = 1; i < n; ++i) {
        float3 tb = tangent(i);
        float3 moved = carry(ns[i - 1], tangent(i - 1), tb);
        ns[i] = normalize(moved - tb * dot(moved, tb));
    }
    float3 tb = tangent(0);
    float3 end = carry(ns[n - 1], tangent(n - 1), tb);
    end = normalize(end - tb * dot(end, tb));
    float twist = std::atan2(dot(tb, cross(end, ns[0])), dot(end, ns[0]));
    Mesh m;
    for (size_t i = 0; i < n; ++i) {
        float3 t = tangent(i);
        float3 ni = rodrigues(ns[i], t, twist * (float)i / (float)n);
        float3 bi = cross(t, ni);
        for (int k = 0; k < sides; ++k) {
            float a = (float)k / sides * 2.f * PI;
            float3 radial = ni * std::cos(a) + bi * std::sin(a);
            m.pos.push_back(path[i] + radial * radius);
            m.nrm.push_back(radial);
        }
    }
    for (size_t i = 0; i < n; ++i) {
        size_t i1 = (i + 1) % n;
        for (int k = 0; k < sides; ++k) {
            int k1 = (k + 1) % sides;
            uint32_t a = (uint32_t)(i * sides + k), b = (uint32_t)(i * sides + k1);
            uint32_t c = (uint32_t)(i1 * sides + k), d = (uint32_t)(i1 * sides + k1);
            for (uint32_t v : {a, b, c, b, d, c}) m.idx.push_back(v);
        }
    }
    orient_outward(m);
    return m;
}
inline Mesh glass_loop() {
    const float src[6][3] = {{-0.78f, -0.22f, 0.12f}, {-0.22f, -0.48f, 0.38f}, {0.55f, -0.12f, 0.22f},
                             {0.42f, 0.46f, -0.22f},  {-0.18f, 0.38f, -0.48f}, {-0.68f, 0.08f, -0.12f}};
    std::vector<float3> ring;
    for (auto& q : src) ring.push_back({(q[0] + 0.095f) * 0.55f - 0.25f, 0.22f + q[2] * 0.20f, q[1] * 0.70f});
    return tube(catmull_closed(ring, 40), 64, 0.035f);
}
// Flattened ellipsoid, tilted about Z. radii all equal = sphere.
inline Mesh pebble(float3 center, float3 radii, float tilt) {
    const int rings = 48, segs = 96;
    float c = std::cos(tilt), s = std::sin(tilt);
    auto rotz = [&](float3 v) { return float3{v.x * c - v.y * s, v.x * s + v.y * c, v.z}; };
    Mesh m;
    for (int i = 0; i <= rings; ++i) {
        float th = (float)i / rings * PI;
        for (int j = 0; j <= segs; ++j) {
            float ph = (float)j / segs * 2.f * PI;
            float3 u{std::sin(th) * std::cos(ph), std::cos(th), std::sin(th) * std::sin(ph)};
            m.pos.push_back(center + rotz(u * radii));
            m.nrm.push_back(normalize(rotz(u / radii)));
        }
    }
    uint32_t w = segs + 1;
    for (uint32_t i = 0; i < (uint32_t)rings; ++i)
        for (uint32_t j = 0; j < (uint32_t)segs; ++j) {
            uint32_t a = i * w + j, b = a + 1, c2 = (i + 1) * w + j, d = c2 + 1;
            for (uint32_t v : {a, b, c2, b, d, c2}) m.idx.push_back(v);
        }
    orient_outward(m);
    return m;
}


// ------------------------------------------------------------------------------------------ pipeline
struct Params {
    float ior = 1.52f;           // n at 550 nm
    float dispersion = 0.03f;    // Cauchy: n(l) = ior + D((550/l)^2 - 1)
    float3 absorb{0.02f, 0.01f, 0.02f};
    float intensity = 6.0f;
    float ratioCap = 64.f;       // brightness limit at fold caustics (energy beyond it is dropped)
    float padTexels = 0.75f;     // each triangle grows by this many map texels (energy kept) so slivers are not lost between pixels
    int cap = 256;
    int map = 1024;
    float rsize = 3.1f;          // receiver edge length, receiver centred on the origin, plane y = 0
    bool footprint = true;       // also draw the unrefracted shadow of the glass (skip it when Filament casts the shadow)
    // opaque oriented box that blocks sunlight before and after the glass (boxHalf <= 0 disables it)
    float3 boxCenter{0.f}, boxAxis0{1, 0, 0}, boxAxis1{0, 1, 0}, boxAxis2{0, 0, 1};
    float boxHalf = 0.f;
    // > 0: one fixed capture window (half size, centre) for every mesh instead of each mesh's own bounds.
    // Needed when many posed solids share one capture (cap_*.mat from caustic_all/ follow the entity transform).
    bool mips = false;           // caustic and footprint maps get a mip chain every frame (for a softened lookup)
    float extentOverride = 0.f;
    float3 originOverride{0.f};
};
struct RT {
    Texture* color = nullptr;
    Texture* depth = nullptr;
    RenderTarget* rt = nullptr;
};
struct Materials {
    Material *front = nullptr, *back = nullptr, *hits = nullptr, *tri = nullptr;
    Material* grid = nullptr;  // optional: indexed-grid splatting (caustic_all/grid.mat) replaces tri
};
inline std::vector<uint8_t> load_file(const char* path) {
    std::ifstream in(path, std::ios::binary);
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(in)), {});
}
inline Material* load_material(Engine* e, const std::string& path) {
    auto b = load_file(path.c_str());
    if (b.empty()) return nullptr;
    return Material::Builder().package(b.data(), b.size()).build(*e);
}
inline bool load_materials(Engine* e, const std::string& dir, Materials& m) {
    m.front = load_material(e, dir + "/cap_front.filamat");
    m.back = load_material(e, dir + "/cap_back.filamat");
    m.hits = load_material(e, dir + "/hits.filamat");
    m.tri = load_material(e, dir + "/tri.filamat");
    return m.front && m.back && m.hits && m.tri;
}
inline RT make_rt(Engine* e, int w, int h, Texture::InternalFormat fmt, bool depth, bool mips = false) {
    RT r;
    auto usage = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC;
    if (mips) usage |= Texture::Usage::BLIT_DST | Texture::Usage::GEN_MIPMAPPABLE;
    r.color = Texture::Builder().width(w).height(h).levels(mips ? (uint8_t)(1 + (int)std::floor(std::log2((float)std::max(w, h)))) : 1)
                  .usage(usage).format(fmt).build(*e);
    RenderTarget::Builder b;
    b.texture(RenderTarget::AttachmentPoint::COLOR, r.color);
    if (depth) {
        r.depth = Texture::Builder().width(w).height(h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                      .format(Texture::InternalFormat::DEPTH32F).build(*e);
        b.texture(RenderTarget::AttachmentPoint::DEPTH, r.depth);
    }
    r.rt = b.build(*e);
    return r;
}
inline void setup_view(View* v, Scene* s, Camera* c, RenderTarget* rt, int w, int h) {
    v->setScene(s);
    v->setCamera(c);
    v->setRenderTarget(rt);
    v->setViewport({0, 0, (uint32_t)w, (uint32_t)h});
    v->setPostProcessingEnabled(false);
    v->setShadowingEnabled(false);
    v->setAntiAliasing(View::AntiAliasing::NONE);
    v->setDithering(View::Dithering::NONE);
}

class CausticPass {
public:
    struct Obj {
        Mesh mesh;
        VertexBuffer* vb;
        IndexBuffer* ib;
        float3 origin, right, up;
        float extent;
        Camera* cam;
        RT front, back, hits;
        Scene *sf, *sb, *sh;
        View *vf, *vb_, *vh;
        std::vector<utils::Entity> ents;  // front copies first, then back copies
        std::vector<MaterialInstance*> capMis, sunMis;  // instances told (origin, dir) / (origin, dir, right, up)
    };

    // copies[i] = how many posed instances of meshes[i] (default 1); poses are set with setPose().
    CausticPass(Engine* e, const Materials& M, std::vector<Mesh> meshes, float3 sunDir, const Params& params,
                std::vector<int> copies = {}) : e_(e), P(params) {
        const int n = P.cap;
        float3 sun = normalize(sunDir);
        float3 upHint = std::fabs(sun.y) > 0.99f ? float3{1, 0, 0} : float3{0, 1, 0};
        auto& em = utils::EntityManager::get();
        TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST,
                               TextureSampler::WrapMode::CLAMP_TO_EDGE);
        const float3 rOrigin{0, 0, 0}, rNormal{0, 1, 0}, rTan{1, 0, 0}, rBit{0, 0, 1};

        // an identity camera (origin, looking down -z, ortho [-1,1]^2) for every fullscreen / uv-space pass
        utils::Entity qcamE = em.create();
        qcam_ = e->createCamera(qcamE);
        qcam_->setProjection(Camera::Projection::ORTHO, -1, 1, -1, 1, 0.1, 10);
        qcam_->lookAt({0, 0, 0}, {0, 0, -1}, {0, 1, 0});

        // uv0.y = 0 on the TOP edge, so uv0.y * height is the texture row (Metal: row 0 = top)
        const float quad[4][5] = {{-1, -1, -1, 0, 1}, {1, -1, -1, 1, 1}, {1, 1, -1, 1, 0}, {-1, 1, -1, 0, 0}};
        VertexBuffer* qvb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                                .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                                .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*e);
        {
            float* d = new float[20];
            std::memcpy(d, quad, 80);
            qvb->setBufferAt(*e, 0, VertexBuffer::BufferDescriptor(d, 80, [](void* p, size_t, void*) { delete[] (float*)p; }));
        }
        IndexBuffer* qib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::USHORT).build(*e);
        {
            uint16_t* d = new uint16_t[6]{0, 1, 2, 0, 2, 3};
            qib->setBuffer(*e, IndexBuffer::BufferDescriptor(d, 12, [](void* p, size_t, void*) { delete[] (uint16_t*)p; }));
        }
        // dummy geometry for the triangle pass: 36 (caustic) or 6 (footprint) vertices, instanced per cell
        auto dummy_vb = [&](int count) {
            VertexBuffer* vb = VertexBuffer::Builder().vertexCount(count).bufferCount(1)
                                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3).build(*e);
            float* d = new float[count * 3]();
            vb->setBufferAt(*e, 0, VertexBuffer::BufferDescriptor(d, count * 12, [](void* p, size_t, void*) { delete[] (float*)p; }));
            return vb;
        };
        auto seq_ib = [&](int count) {
            IndexBuffer* ib = IndexBuffer::Builder().indexCount(count).bufferType(IndexBuffer::IndexType::USHORT).build(*e);
            uint16_t* d = new uint16_t[count];
            for (int i = 0; i < count; ++i) d[i] = (uint16_t)i;
            ib->setBuffer(*e, IndexBuffer::BufferDescriptor(d, count * 2, [](void* p, size_t, void*) { delete[] (uint16_t*)p; }));
            return ib;
        };
        VertexBuffer* tvb36 = dummy_vb(36);
        IndexBuffer* tib36 = seq_ib(36);
        VertexBuffer* tvb6 = dummy_vb(6);
        IndexBuffer* tib6 = seq_ib(6);

        // grid.mat path: one vertex per light-view texel, six indices per cell, shared by every object
        VertexBuffer* gvb = nullptr;
        IndexBuffer* gib = nullptr;
        if (M.grid) {
            gvb = dummy_vb(n * n);
            const uint32_t cells1 = (uint32_t)(n - 1);
            gib = IndexBuffer::Builder().indexCount(cells1 * cells1 * 6).bufferType(IndexBuffer::IndexType::UINT).build(*e);
            uint32_t* gi = new uint32_t[(size_t)cells1 * cells1 * 6];
            size_t w = 0;
            for (uint32_t y = 0; y < cells1; ++y)
                for (uint32_t x = 0; x < cells1; ++x) {
                    uint32_t a = y * n + x, b = a + 1, c = a + n, d = c + 1;
                    for (uint32_t v : {a, b, c, b, d, c}) gi[w++] = v;
                }
            gib->setBuffer(*e, IndexBuffer::BufferDescriptor(gi, (size_t)cells1 * cells1 * 24, [](void* p, size_t, void*) { delete[] (uint32_t*)p; }));
        }

        mapC_ = make_rt(e, P.map, P.map, Texture::InternalFormat::RGBA16F, false, P.mips);
        mapF_ = make_rt(e, P.map, P.map, Texture::InternalFormat::RGBA16F, false, P.mips);
        sC_ = e->createScene();
        sF_ = e->createScene();
        vC_ = e->createView();
        vF_ = e->createView();
        setup_view(vC_, sC_, qcam_, mapC_.rt, P.map, P.map);
        setup_view(vF_, sF_, qcam_, mapF_.rt, P.map, P.map);

        objs_.reserve(meshes.size());
        for (Mesh& m : meshes) {
            Obj o;
            o.mesh = std::move(m);
            float3 f = sun;  // light basis, same construction as Camera::lookAt
            o.right = normalize(cross(f, upHint));
            o.up = cross(o.right, f);
            float3 lo{1e9f, 1e9f, 1e9f}, hi{-1e9f, -1e9f, -1e9f};
            for (float3 p : o.mesh.pos) {
                float3 q{dot(p, o.right), dot(p, o.up), dot(p, f)};
                lo = min(lo, q);
                hi = max(hi, q);
            }
            float3 c = (lo + hi) * 0.5f;
            o.origin = o.right * c.x + o.up * c.y + f * c.z;
            o.extent = std::max(hi.x - lo.x, hi.y - lo.y) * 0.5f + 0.02f;
            if (P.extentOverride > 0.f) {
                o.origin = P.originOverride;
                o.extent = P.extentOverride;
            }
            const size_t objIndex = objs_.size();
            const int nCopies = objIndex < copies.size() ? copies[objIndex] : 1;

            size_t nv = o.mesh.pos.size();  // mesh buffers: POSITION + CUSTOM0 (normal)
            o.vb = VertexBuffer::Builder().vertexCount((uint32_t)nv).bufferCount(1)
                       .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 24)
                       .attribute(VertexAttribute::CUSTOM0, 0, VertexBuffer::AttributeType::FLOAT3, 12, 24).build(*e);
            float* vd = new float[nv * 6];
            for (size_t i = 0; i < nv; ++i) {
                std::memcpy(vd + i * 6, &o.mesh.pos[i], 12);
                std::memcpy(vd + i * 6 + 3, &o.mesh.nrm[i], 12);
            }
            o.vb->setBufferAt(*e, 0, VertexBuffer::BufferDescriptor(vd, nv * 24, [](void* p, size_t, void*) { delete[] (float*)p; }));
            o.ib = IndexBuffer::Builder().indexCount((uint32_t)o.mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*e);
            uint32_t* id = new uint32_t[o.mesh.idx.size()];
            std::memcpy(id, o.mesh.idx.data(), o.mesh.idx.size() * 4);
            o.ib->setBuffer(*e, IndexBuffer::BufferDescriptor(id, o.mesh.idx.size() * 4, [](void* p, size_t, void*) { delete[] (uint32_t*)p; }));

            utils::Entity ce = em.create();
            o.cam = e->createCamera(ce);
            o.cam->setProjection(Camera::Projection::ORTHO, -o.extent, o.extent, -o.extent, o.extent, 0.1, 20.0);
            o.cam->lookAt(o.origin - f * 10.f, o.origin, upHint);

            o.front = make_rt(e, n, n, Texture::InternalFormat::RGBA32F, true);
            o.back = make_rt(e, n, n, Texture::InternalFormat::RGBA32F, true);
            o.hits = make_rt(e, n * 7, n, Texture::InternalFormat::RGBA32F, false);

            auto glass_renderable = [&](Material* mat, Scene* s) {
                MaterialInstance* mi = mat->createInstance();
                mi->setParameter("origin", o.origin);
                mi->setParameter("dir", f);
                o.capMis.push_back(mi);
                for (int k = 0; k < nCopies; ++k) {
                    utils::Entity ent = em.create();
                    RenderableManager::Builder(1).boundingBox({{-5, -5, -5}, {5, 5, 5}})
                        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, o.vb, o.ib)
                        .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*e, ent);
                    s->addEntity(ent);
                    o.ents.push_back(ent);
                }
            };
            o.sf = e->createScene();
            o.sb = e->createScene();
            o.sh = e->createScene();
            glass_renderable(M.front, o.sf);
            glass_renderable(M.back, o.sb);
            o.vf = e->createView();
            o.vb_ = e->createView();
            o.vh = e->createView();
            setup_view(o.vf, o.sf, o.cam, o.front.rt, n, n);
            setup_view(o.vb_, o.sb, o.cam, o.back.rt, n, n);
            setup_view(o.vh, o.sh, qcam_, o.hits.rt, n * 7, n);

            auto common = [&](MaterialInstance* mi) {
                o.sunMis.push_back(mi);
                mi->setParameter("origin", o.origin);
                mi->setParameter("dir", f);
                mi->setParameter("right", o.right);
                mi->setParameter("up", o.up);
                mi->setParameter("extent", o.extent);
                mi->setParameter("captureSize", (float)n);
                mi->setParameter("rOrigin", rOrigin);
                mi->setParameter("rNormal", rNormal);
                mi->setParameter("rTangent", rTan);
                mi->setParameter("rBitangent", rBit);
                mi->setParameter("rSize", P.rsize);
            };
            MaterialInstance* hmi = M.hits->createInstance();
            common(hmi);
            hmi->setParameter("frontFace", o.front.color, nearest);
            hmi->setParameter("backFace", o.back.color, nearest);
            hmi->setParameter("ior", P.ior);
            hmi->setParameter("dispersion", P.dispersion);
            hmi->setParameter("absorption", P.absorb);
            hmi->setParameter("boxCenter", P.boxCenter);
            hmi->setParameter("boxAxis0", P.boxAxis0);
            hmi->setParameter("boxAxis1", P.boxAxis1);
            hmi->setParameter("boxAxis2", P.boxAxis2);
            hmi->setParameter("boxHalf", P.boxHalf);
            utils::Entity he = em.create();
            RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
                .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, qvb, qib)
                .material(0, hmi).culling(false).castShadows(false).receiveShadows(false).build(*e, he);
            o.sh->addEntity(he);

            const int cells = (n - 1) * (n - 1);
            const int chunk = 32000;  // Filament: at most 32767 instances per renderable
            for (int mode = 0; mode < (P.footprint ? 2 : 1); ++mode) {
                if (M.grid) {  // bands 0..5 as 6 instances into the caustic map, band 6 (footprint) into the footprint map
                    MaterialInstance* gmi = M.grid->createInstance();
                    common(gmi);
                    gmi->setParameter("hitTex", o.hits.color, nearest);
                    gmi->setParameter("intensity", P.intensity);
                    gmi->setParameter("ratioCap", P.ratioCap);
                    gmi->setParameter("bandBase", mode ? 6.f : 0.f);
                    gmi->setParameter("mapSize", (float)P.map);
                    utils::Entity ge = em.create();
                    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
                        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, gvb, gib)
                        .instances(mode ? 1 : 6)
                        .material(0, gmi).culling(false).castShadows(false).receiveShadows(false).build(*e, ge);
                    (mode ? sF_ : sC_)->addEntity(ge);
                    continue;
                }
                for (int base = 0; base < cells; base += chunk) {
                    MaterialInstance* tmi = M.tri->createInstance();
                    common(tmi);
                    tmi->setParameter("hitTex", o.hits.color, nearest);
                    tmi->setParameter("intensity", P.intensity);
                    tmi->setParameter("mode", (float)mode);
                    tmi->setParameter("ratioCap", P.ratioCap);
                    tmi->setParameter("instanceBase", (float)base);
                    tmi->setParameter("pad", P.padTexels / (float)P.map);
                    utils::Entity te = em.create();
                    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
                        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, mode ? tvb6 : tvb36, mode ? tib6 : tib36)
                        .instances((size_t)std::min(chunk, cells - base))
                        .material(0, tmi).culling(false).castShadows(false).receiveShadows(false).build(*e, te);
                    (mode ? sF_ : sC_)->addEntity(te);
                }
            }
            objs_.push_back(std::move(o));
        }
    }

    // Call between beginFrame and endFrame. Sets the renderer's clear options (every pass clears its own target).
    void draw(Renderer* r) {
        r->setClearOptions({.clearColor = {0, 0, 0, 0}, .clear = true, .discard = true});
        for (Obj& o : objs_) {  // skipMask (measuring only): 1 front, 2 back, 4 hits, 8 caustic map, 16 footprint map
            if (!(skipMask & 1)) r->render(o.vf);
            if (!(skipMask & 2)) r->render(o.vb_);
            if (!(skipMask & 4)) r->render(o.vh);
        }
        if (!(skipMask & 8)) r->render(vC_);
        if (P.footprint && !(skipMask & 16)) r->render(vF_);
        if (P.mips) {
            mapC_.color->generateMipmaps(*e_);
            if (P.footprint) mapF_.color->generateMipmaps(*e_);
        }
    }
    // Turn the sun (direction the light travels). Only for a pass built with extentOverride: the capture window keeps
    // its centre and size, so the size must cover the solids from every side.
    void setSun(float3 sunDir) {
        float3 f = normalize(sunDir);
        float3 upHint = std::fabs(f.y) > 0.99f ? float3{1, 0, 0} : float3{0, 1, 0};
        for (Obj& o : objs_) {
            o.right = normalize(cross(f, upHint));
            o.up = cross(o.right, f);
            o.cam->lookAt(o.origin - f * 10.f, o.origin, upHint);
            for (MaterialInstance* mi : o.capMis) mi->setParameter("dir", f);
            for (MaterialInstance* mi : o.sunMis) {
                mi->setParameter("dir", f);
                mi->setParameter("right", o.right);
                mi->setParameter("up", o.up);
            }
        }
    }
    Texture* caustic() const { return mapC_.color; }
    Texture* footprint() const { return mapF_.color; }
    RenderTarget* causticTarget() const { return mapC_.rt; }
    RenderTarget* footprintTarget() const { return mapF_.rt; }
    const std::vector<Obj>& objects() const { return objs_; }
    const Params& params() const { return P; }
    int skipMask = 0;

private:
    Engine* e_;
    Params P;
    Camera* qcam_ = nullptr;
    RT mapC_, mapF_;
    Scene *sC_ = nullptr, *sF_ = nullptr;
    View *vC_ = nullptr, *vF_ = nullptr;
    std::vector<Obj> objs_;
};

}  // namespace ctri
