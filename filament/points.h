// A point cloud as a solid thing: a .ply of points (a scan's, red green blue) or of Gaussian splats as 3DGS writes them
// (f_dc colours, a logit opacity, log scales, a turn as a quaternion), each point a splat points.mat turns to the eye as
// an ellipse that fades out (antimatter15/splat's way, MIT, Copyright (c) 2023 Kevin Kwok: its spread in space (the
// covariance, worked out here once) laid on the screen, its two axes, exp(-r^2)), its own colour. A plain scan's point
// is round, as large as its neighbours are far. Not sorted: masked, so Filament's alpha to coverage blends them by the
// picture's samples (the stage's MSAA). All in one vertex buffer, four corners a point, drawn in one go; fitted to 1
// tall, its middle at the place, as solid.h fits a file. Its Mesh is a name only: the kit knows the thing by it.
// (kit.h: read before this)
#pragma once
#include <cmath>
#include <fstream>
#include <sstream>

static Mesh* points(Stage& s, const char* file, std::string& why, MaterialInstance** looks = nullptr) {
    std::ifstream in(file, std::ios::binary);
    std::string line, format;
    size_t n = 0;
    struct Prop { std::string name, type; size_t at = 0; };
    std::vector<Prop> props;
    size_t stride = 0;
    bool inVertex = false;
    const auto bytes = [](const std::string& t) -> size_t { return t == "double" || t == "float64" ? 8 : t == "float" || t == "float32" || t == "int" || t == "uint" || t == "int32" || t == "uint32" ? 4 : t == "short" || t == "ushort" || t == "int16" || t == "uint16" ? 2 : 1; };
    while (std::getline(in, line)) {   // (the header: which properties a point has, in order)
        if (!line.empty() && line.back() == '\r') line.pop_back();
        std::istringstream w(line);
        std::string word;
        w >> word;
        if (word == "format") w >> format;
        else if (word == "element") { std::string what; w >> what >> n; inVertex = what == "vertex"; if (!inVertex && !props.empty()) break; }
        else if (word == "property" && inVertex) { Prop p; w >> p.type; if (p.type == "list") return why = "a point cloud's points have no lists", nullptr; w >> p.name; p.at = stride; stride += bytes(p.type); props.push_back(p); }
        else if (word == "end_header") break;
    }
    if (format != "binary_little_endian" || !n || !stride) return why = "a point cloud is a binary .ply (little-endian)", nullptr;
    std::vector<uint8_t> raw(n * stride);
    if (!in.read((char*)raw.data(), (std::streamsize)raw.size())) return why = "the point cloud is shorter than its header says", nullptr;
    const auto find = [&](const char* name) -> const Prop* { for (const Prop& p : props) if (p.name == name) return &p; return nullptr; };
    const auto get = [&](const Prop* p, size_t i) -> double {
        const uint8_t* at = raw.data() + i * stride + p->at;
        if (p->type == "float" || p->type == "float32") { float v; memcpy(&v, at, 4); return v; }
        if (p->type == "double" || p->type == "float64") { double v; memcpy(&v, at, 8); return v; }
        if (p->type == "uchar" || p->type == "uint8") return *at;
        if (p->type == "ushort" || p->type == "uint16") { uint16_t v; memcpy(&v, at, 2); return v; }
        return 0;
    };
    const Prop *x = find("x"), *y = find("y"), *z = find("z"), *r = find("red"), *g = find("green"), *b = find("blue"), *d0 = find("f_dc_0"), *d1 = find("f_dc_1"), *d2 = find("f_dc_2"),
               *op = find("opacity"), *s0 = find("scale_0"), *s1 = find("scale_1"), *s2 = find("scale_2"),
               *r0 = find("rot_0"), *r1 = find("rot_1"), *r2 = find("rot_2"), *r3 = find("rot_3");
    if (!x || !y || !z) return why = "a point cloud's points have x, y and z", nullptr;
    std::vector<uint32_t> kept;   // (the points seen: a splat nearly clear is left out)
    kept.reserve(n);
    for (size_t i = 0; i < n; ++i) if (!op || 1.0 / (1.0 + std::exp(-get(op, i))) >= 0.05) kept.push_back((uint32_t)i);
    if (kept.empty()) return why = "the point cloud has no points to see", nullptr;
    // (its box from the most of it, a hundredth left out at each end of each way: a few far-off grains, as a capture's
    //  sky and floaters are, do not make the rest small; those points stay, drawn where they are)
    float3 lo, hi;
    for (int a = 0; a < 3; ++a) {
        const Prop* along = a == 0 ? x : a == 1 ? y : z;
        std::vector<float> v(kept.size());
        for (size_t k = 0; k < kept.size(); ++k) v[k] = (float)(a == 1 ? -get(along, kept[k]) : get(along, kept[k]));   // (the scan's y points down, as COLMAP's does: up here)
        std::nth_element(v.begin(), v.begin() + (long)(v.size() / 100), v.end());
        lo[a] = v[v.size() / 100];
        std::nth_element(v.begin(), v.begin() + (long)(v.size() - 1 - v.size() / 100), v.end());
        hi[a] = v[v.size() - 1 - v.size() / 100];
    }
    // (fitted to 1 tall about its middle; for a scan of a room or a street the tall way is its height)
    const float3 mid = 0.5f * (lo + hi);
    const float tall = std::max(1e-6f, std::max({hi.y - lo.y, 0.5f * (hi.x - lo.x), 0.5f * (hi.z - lo.z)}));
    const float plain = 1.2f * std::cbrt((hi.x - lo.x) * (hi.y - lo.y) * (hi.z - lo.z) / (float)kept.size()) / tall;   // (a scan's grain: as far as its neighbours are, on the whole)
    const size_t m = kept.size();
    struct Corner { float3 pos; float4 own; float4 spreadA; float4 spreadB; uint8_t rgba[4]; };
    std::vector<Corner> corners(m * 4);
    std::vector<uint32_t> idx(m * 6);
    static const float2 c4[4] = {{-2, -2}, {2, -2}, {2, 2}, {-2, 2}};   // (two of its spread each way: past that it is all but gone)
    for (size_t k = 0; k < m; ++k) {
        const size_t i = kept[k];
        const float3 p = (float3{(float)get(x, i), (float)-get(y, i), (float)get(z, i)} - mid) / tall;
        float3 colour{1.f};
        if (r && g && b) colour = float3{(float)get(r, i), (float)get(g, i), (float)get(b, i)} / 255.f;
        else if (d0 && d1 && d2) colour = clamp(0.5f + 0.2820948f * float3{(float)get(d0, i), (float)get(d1, i), (float)get(d2, i)}, 0.f, 1.f);   // (3DGS's zero band)
        // (its spread in space, R S S R^T, as fitted: y turned up and all over `tall`; a plain scan's round, half its gap)
        mat3f spread = mat3f(1.f) * (0.25f * plain * plain);
        if (s0 && s1 && s2) {
            float4 q = r0 && r1 && r2 && r3 ? float4{(float)get(r1, i), (float)get(r2, i), (float)get(r3, i), (float)get(r0, i)} : float4{0, 0, 0, 1};   // (x y z, then w: 3DGS writes w first)
            q = length(q) > 1e-12f ? normalize(q) : float4{0, 0, 0, 1};
            const mat3f turn = mat3f(quatf{q.w, q.x, q.y, q.z});
            const float3 sc = float3{std::exp((float)get(s0, i)), std::exp((float)get(s1, i)), std::exp((float)get(s2, i))} / tall;
            const mat3f m3 = turn * mat3f(float3{sc.x, 0, 0}, float3{0, sc.y, 0}, float3{0, 0, sc.z});
            const mat3f flip(float3{1, 0, 0}, float3{0, -1, 0}, float3{0, 0, 1});
            spread = flip * m3 * transpose(m3) * flip;
        }
        const float seen = op ? 1.f / (1.f + std::exp(-(float)get(op, i))) : 1.f;
        const float seed = (float)(k % 100003) * 0.731f;
        for (int c = 0; c < 4; ++c) {
            Corner& o = corners[k * 4 + (size_t)c];
            o.pos = p, o.own = float4{c4[c].x, c4[c].y, seen, seed};
            o.spreadA = float4{spread[0][0], spread[0][1], spread[0][2], spread[1][1]}, o.spreadB = float4{spread[1][2], spread[2][2], 0, 0};
            for (int j = 0; j < 3; ++j) o.rgba[j] = (uint8_t)std::lround(255.f * colour[j]);
            o.rgba[3] = 255;
        }
        const uint32_t v = (uint32_t)(k * 4);
        const uint32_t six[6] = {v, v + 1, v + 2, v, v + 2, v + 3};
        memcpy(&idx[k * 6], six, sizeof six);
    }
    Material* mat = material(s, "points.filamat");
    if (!mat) return why = "points.filamat is not built", nullptr;
    // (in parts of at most so many splats each, a vertex buffer and its indices a part: one buffer of WebGPU holds 256 MB
    //  at most; all of them parts of the one thing, drawn with its one look)
    const size_t part = 1u << 19;
    const size_t parts = (m + part - 1) / part;
    std::vector<std::pair<VertexBuffer*, IndexBuffer*>> made;
    for (size_t at = 0; at < m; at += part) {
        const size_t here = std::min(part, m - at);
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(here * 4)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, offsetof(Corner, pos), sizeof(Corner))
                               .attribute(VertexAttribute::CUSTOM0, 0, VertexBuffer::AttributeType::FLOAT4, offsetof(Corner, own), sizeof(Corner))
                               .attribute(VertexAttribute::CUSTOM1, 0, VertexBuffer::AttributeType::FLOAT4, offsetof(Corner, spreadA), sizeof(Corner))
                               .attribute(VertexAttribute::CUSTOM2, 0, VertexBuffer::AttributeType::FLOAT4, offsetof(Corner, spreadB), sizeof(Corner))
                               .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::UBYTE4, offsetof(Corner, rgba), sizeof(Corner))
                               .normalized(VertexAttribute::COLOR).build(*s.engine);
        vb->setBufferAt(*s.engine, 0, copied(corners.data() + at * 4, here * 4 * sizeof(Corner)));
        std::vector<uint32_t> six(here * 6);
        for (size_t k = 0; k < six.size(); ++k) six[k] = idx[at * 6 + k] - (uint32_t)(at * 4);
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)six.size()).bufferType(IndexBuffer::IndexType::UINT).build(*s.engine);
        ib->setBuffer(*s.engine, copied(six.data(), six.size() * sizeof(uint32_t)));
        made.push_back({vb, ib});
    }
    MaterialInstance* look = mat->createInstance();
    look->setParameter("dissolve", 0.f);
    look->setParameter("grain", 1.f);
    Mesh* name = new Mesh;
    name->vb = made[0].first, name->ib = made[0].second;
    utils::Entity e = utils::EntityManager::get().create();
    const float3 box = 0.5f * (hi - lo) / tall + float3{0.05f};
    RenderableManager::Builder all(parts);
    all.boundingBox({float3{0.f}, box * 3.f}).castShadows(false).receiveShadows(false).culling(false);
    for (size_t k = 0; k < parts; ++k) all.geometry(k, RenderableManager::PrimitiveType::TRIANGLES, made[k].first, made[k].second, 0, made[k].second->getIndexCount()).material(k, look);
    all.build(*s.engine, e);
    s.engine->getTransformManager().create(e);
    s.scene->addEntity(e);
    s.made.push_back({e, name, 0, mat4f{}});
    if (looks) *looks = look;
    return name;
}
