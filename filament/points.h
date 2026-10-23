// A point cloud as a solid thing: a .ply of points (a scan's, red green blue) or of Gaussian splats as 3DGS writes them
// (f_dc colours, a logit opacity, log scales, a turn as a quaternion), drawn as antimatter15/splat draws them (MIT,
// Copyright (c) 2023 Kevin Kwok): each splat's data in a texture once (its middle, how much it is seen, its spread in
// space, its colour), one square a slot that points.mat turns into the ellipse its spread makes on the screen, faded
// out by exp(-r^2); the slots laid in order of depth by a 16-bit counting sort on a thread of its own each time the eye
// has moved, only the table of which splat is in which slot sent again; then laid one over another from the farthest,
// Filament's own see-through blending. A plain scan's point is round, as large as its neighbours are far. Fitted to 1
// tall, its middle at the place, as solid.h fits a file. Its Mesh is a name only: the kit knows the thing by it.
// (kit.h: read before this)
#pragma once
#include <cmath>
#include <fstream>
#include <sstream>
#include <condition_variable>
#include <thread>
#include <memory>

// (floats handed to a texture, freed by Filament once it has them)
static backend::PixelBufferDescriptor points_pixels(std::vector<float>&& from, backend::PixelDataFormat format) {
    auto* held = new std::vector<float>(std::move(from));
    return backend::PixelBufferDescriptor(held->data(), held->size() * sizeof(float), format, backend::PixelDataType::FLOAT, [](void*, size_t, void* user) { delete (std::vector<float>*)user; }, held);
}

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
    // (each splat four texels: middle and how much it is seen; its spread R S S^T R^T, as fitted (y turned up, all over
    //  `tall`); the rest of the spread and red, green; blue)
    const uint32_t wide = 4096, rows = (uint32_t)((m * 4 + wide - 1) / wide);
    std::vector<float> data((size_t)wide * rows * 4, 0.f);
    std::vector<float3> middles(m);
    for (size_t k = 0; k < m; ++k) {
        const size_t i = kept[k];
        const float3 p = (float3{(float)get(x, i), (float)-get(y, i), (float)get(z, i)} - mid) / tall;
        float3 colour{1.f};
        if (r && g && b) colour = float3{(float)get(r, i), (float)get(g, i), (float)get(b, i)} / 255.f;
        else if (d0 && d1 && d2) colour = clamp(0.5f + 0.2820948f * float3{(float)get(d0, i), (float)get(d1, i), (float)get(d2, i)}, 0.f, 1.f);   // (3DGS's zero band)
        mat3f spread = mat3f(1.f) * (0.25f * plain * plain);
        if (s0 && s1 && s2) {
            float4 q = r0 && r1 && r2 && r3 ? float4{(float)get(r1, i), (float)get(r2, i), (float)get(r3, i), (float)get(r0, i)} : float4{0, 0, 0, 1};   // (x y z, then w: 3DGS writes w first)
            q = length(q) > 1e-12f ? normalize(q) : float4{0, 0, 0, 1};
            const float3 sc = float3{std::exp((float)get(s0, i)), std::exp((float)get(s1, i)), std::exp((float)get(s2, i))} / tall;
            const mat3f m3 = mat3f(quatf{q.w, q.x, q.y, q.z}) * mat3f(float3{sc.x, 0, 0}, float3{0, sc.y, 0}, float3{0, 0, sc.z});
            const mat3f flip(float3{1, 0, 0}, float3{0, -1, 0}, float3{0, 0, 1});
            spread = flip * m3 * transpose(m3) * flip;
        }
        const float seen = op ? 1.f / (1.f + std::exp(-(float)get(op, i))) : 1.f;
        middles[k] = p;
        const float four[16] = {p.x, p.y, p.z, seen, spread[0][0], spread[0][1], spread[0][2], spread[1][1], spread[1][2], spread[2][2], colour.r, colour.g, colour.b, 0, 0, 0};
        memcpy(&data[k * 16], four, sizeof four);
    }
    Material* mat = material(s, "points.filamat");
    if (!mat) return why = "points.filamat is not built", nullptr;
    Texture* splats = Texture::Builder().width(wide).height(rows).levels(1).format(Texture::InternalFormat::RGBA32F).sampler(Texture::Sampler::SAMPLER_2D).build(*s.engine);
    splats->setImage(*s.engine, 0, points_pixels(std::move(data), backend::PixelDataFormat::RGBA));
    const uint32_t slotRows = (uint32_t)((m + wide - 1) / wide);
    Texture* order = Texture::Builder().width(wide).height(slotRows).levels(1).format(Texture::InternalFormat::R32F).sampler(Texture::Sampler::SAMPLER_2D).build(*s.engine);
    {   // (in the file's order until the first sort is in)
        std::vector<float> first((size_t)wide * slotRows, 0.f);
        for (size_t k = 0; k < m; ++k) first[k] = (float)k;
        order->setImage(*s.engine, 0, points_pixels(std::move(first), backend::PixelDataFormat::R));
    }
    // (four corners a slot, each its corner and its slot; six indices a slot; all as they are for good)
    std::vector<float3> corners(m * 4);
    std::vector<uint32_t> idx(m * 6);
    static const float2 c4[4] = {{-2, -2}, {2, -2}, {2, 2}, {-2, 2}};   // (two of its spread each way: past that it is all but gone)
    for (size_t k = 0; k < m; ++k) {
        for (int c = 0; c < 4; ++c) corners[k * 4 + (size_t)c] = float3{c4[c].x, c4[c].y, (float)k};
        const uint32_t v = (uint32_t)(k * 4);
        const uint32_t six[6] = {v, v + 1, v + 2, v, v + 2, v + 3};
        memcpy(&idx[k * 6], six, sizeof six);
    }
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)corners.size()).bufferCount(1).attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3).build(*s.engine);
    vb->setBufferAt(*s.engine, 0, copied(corners.data(), corners.size() * sizeof(float3)));
    IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*s.engine);
    ib->setBuffer(*s.engine, copied(idx.data(), idx.size() * sizeof(uint32_t)));
    MaterialInstance* look = mat->createInstance();
    TextureSampler exact(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    look->setParameter("splats", splats, exact);
    look->setParameter("order", order, exact);
    Mesh* name = new Mesh;
    name->vb = vb, name->ib = ib;
    utils::Entity e = utils::EntityManager::get().create();
    const float3 box = 0.5f * (hi - lo) / tall + float3{0.05f};
    RenderableManager::Builder(1).boundingBox({float3{0.f}, box * 3.f}).geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib, 0, idx.size())
        .material(0, look).castShadows(false).receiveShadows(false).culling(false).build(*s.engine, e);
    s.engine->getTransformManager().create(e);
    s.scene->addEntity(e);
    s.made.push_back({e, name, 0, mat4f{}});

    // (the sort, antimatter15's: each splat's depth along the eye, put to 16 bits between the nearest and the farthest,
    //  one counting pass; on a thread of its own, asked again whenever the eye has moved; farthest first here)
    struct Sorter {
        std::vector<float3> middles;
        std::mutex lock;
        std::condition_variable told;
        float4 asked{0.f}, done{1e9f};
        bool fresh = false, ready = false, stop = false;
        std::vector<float> slots;
        std::thread worker;
    };
    std::shared_ptr<Sorter> sort(new Sorter, [](Sorter* p) {   // (gone with the moment's hand that holds it: its thread stopped first)
        { std::lock_guard<std::mutex> hold(p->lock); p->stop = true; }
        p->told.notify_one();
        p->worker.join();
        delete p;
    });
    sort->middles = std::move(middles);
    sort->worker = std::thread([sort = sort.get(), wide, slotRows] {
        std::vector<int32_t> depth(sort->middles.size());
        std::vector<uint32_t> counts(65536), starts(65536);
        for (;;) {
            float4 row;
            {
                std::unique_lock<std::mutex> hold(sort->lock);
                sort->told.wait(hold, [sort] { return sort->fresh || sort->stop; });
                if (sort->stop) return;
                row = sort->asked, sort->fresh = false;
            }
            int32_t least = INT32_MAX, most = INT32_MIN;
            for (size_t i = 0; i < depth.size(); ++i) {
                const float3& p = sort->middles[i];
                depth[i] = (int32_t)((row.x * p.x + row.y * p.y + row.z * p.z + row.w) * 4096.f);   // (the eye's z: farther is less)
                least = std::min(least, depth[i]), most = std::max(most, depth[i]);
            }
            const float inv = 65535.f / (float)std::max(1, most - least);
            std::fill(counts.begin(), counts.end(), 0u);
            for (int32_t& d : depth) d = (int32_t)((float)(d - least) * inv), ++counts[(size_t)d];
            starts[0] = 0;
            for (size_t k = 1; k < 65536; ++k) starts[k] = starts[k - 1] + counts[k - 1];
            std::vector<float> slots((size_t)wide * slotRows, 0.f);
            for (size_t i = 0; i < depth.size(); ++i) slots[starts[(size_t)depth[i]]++] = (float)i;
            std::lock_guard<std::mutex> hold(sort->lock);
            sort->slots = std::move(slots), sort->done = row, sort->ready = true;
        }
    });
    // (each moment: where the eye is against the cloud now, asked of the sort if it has moved; a finished order sent on)
    s.playing[name] = [&s, sort, e, order, wide, slotRows](double) {
        const auto& places = s.engine->getTransformManager();
        const math::mat4 eye = s.camera->getViewMatrix() * math::mat4(places.getWorldTransform(places.getInstance(e)));
        const float4 row{(float)eye[0][2], (float)eye[1][2], (float)eye[2][2], (float)eye[3][2]};
        std::lock_guard<std::mutex> hold(sort->lock);
        if (sort->ready) {
            order->setImage(*s.engine, 0, points_pixels(std::move(sort->slots), backend::PixelDataFormat::R));
            sort->ready = false;
        }
        if (length(row - sort->asked) > 1e-4f) sort->asked = row, sort->fresh = true, sort->told.notify_one();
    };
    if (looks) *looks = look;
    return name;
}
