// CBEAMS=1: beam_split.h on the GPU (cbeam.wgsl), through Filament's WebGPU backend (dawn_probe/compute_door.h).
// Here: the table of shapes the shader reads (made once), the programs and buffers, and what a frame sends and runs.
// Per frame the CPU sends the poses, the sun and the colour steps; nothing comes back (check() is for tests).
#pragma once
#include "beam_split.h"
#include "dawn_probe/compute_door.h"

#include <filament/BufferObject.h>
#include <filament/Engine.h>

#include <array>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>

namespace cbeam {

struct Table {
    std::vector<float> t;   // vec4s, see cbeam.wgsl
    uint32_t faceRec = 0;   // vec4s a face takes
    uint32_t seedMost = 0;  // most seeds (faces of the solid itself, by cell) any shape has
};
// shapes[s][use]: split_shape of shape s for the set of films `use`. False: a face has more corners than a piece may.
inline bool table(const std::vector<std::array<beam::SplitShape, 16>>& shapes, Table& out) {
    int most = 3;
    for (const auto& s : shapes)
        for (const beam::SplitShape& sh : s)
            for (const beam::SplitCell& c : sh.cells)
                for (const beam::Poly& F : c.faces) most = std::max(most, F.n);
    if (most > 16) return false;
    out.faceRec = 3u + 2u * (uint32_t)most;
    std::vector<float>& t = out.t;
    t.assign(shapes.size() * 16 * 4, 0.f);
    auto put = [&](size_t at, double a, double b, double c, double d) {
        t[at * 4] = (float)a; t[at * 4 + 1] = (float)b; t[at * 4 + 2] = (float)c; t[at * 4 + 3] = (float)d;
    };
    for (size_t s = 0; s < shapes.size(); ++s)
        for (int use = 0; use < 16; ++use) {
            const beam::SplitShape& sh = shapes[s][(size_t)use];
            const size_t cellAt = t.size() / 4;
            t.resize(t.size() + sh.cells.size() * 2 * 4, 0.f);
            std::vector<std::pair<size_t, size_t>> seeds;
            for (size_t c = 0; c < sh.cells.size(); ++c) {
                const beam::SplitCell& cell = sh.cells[c];
                const size_t faceFirst = t.size() / 4;
                int count = 0;
                for (size_t f = 0; f < cell.faces.size(); ++f) {
                    const beam::Poly& F = cell.faces[f];
                    if (F.n < 3) continue;
                    const size_t rec = t.size() / 4;
                    t.resize(t.size() + out.faceRec * 4, 0.f);
                    const beam::V3 nrm = cell.planes[f].n;
                    put(rec, nrm.x, nrm.y, nrm.z, cell.planes[f].o);
                    put(rec + 1, cell.what[f], F.n, 0, 0);
                    beam::V3 mid{0, 0, 0};
                    for (int i = 0; i < F.n; ++i) mid = mid + F.p[(size_t)i];
                    mid = mid * (1.0 / F.n);
                    double reach = 0;
                    for (int i = 0; i < F.n; ++i) reach = std::max(reach, length(F.p[(size_t)i] - mid));
                    put(rec + 2, mid.x, mid.y, mid.z, reach);
                    for (int i = 0; i < F.n; ++i) {   // (the halves beam_split.h's cut_to_face makes)
                        const beam::V3 a = F.p[(size_t)i], b = F.p[(size_t)((i + 1) % F.n)];
                        beam::V3 in = cross(nrm, b - a);
                        if (dot(in, mid - a) < 0) in = in * -1.0;
                        put(rec + 3 + 2 * (size_t)i, a.x, a.y, a.z, 0);
                        put(rec + 4 + 2 * (size_t)i, in.x, in.y, in.z, -dot(in, a));
                    }
                    ++count;
                    if (cell.what[f] >= 0) seeds.push_back({cellAt + 2 * c, rec});
                }
                put(cellAt + 2 * c, (double)faceFirst, count, 0, 0);
                double ac[4];
                for (int f = 0; f < 4; ++f) ac[f] = cell.across[f] >= 0 ? (double)(cellAt + 2 * (size_t)cell.across[f]) : -1.0;
                put(cellAt + 2 * c + 1, ac[0], ac[1], ac[2], ac[3]);
            }
            const size_t seedFirst = t.size() / 4;
            t.resize(t.size() + seeds.size() * 4, 0.f);
            for (size_t i = 0; i < seeds.size(); ++i) put(seedFirst + i, (double)seeds[i].first, (double)seeds[i].second, 0, 0);
            put(s * 16 + (size_t)use, (double)cellAt, (double)sh.cells.size(), (double)seedFirst, (double)seeds.size());
            out.seedMost = std::max(out.seedMost, (uint32_t)seeds.size());
        }
    return t.size() / 4 < (1u << 24);   // (places in the table are kept as floats)
}

struct Frame {   // cbeam.wgsl's Frame, as it lies in the buffer
    float sun[4], band[4], misc[4];
    uint32_t counts[4], caps[4];
    float steps[16][4], films[4][4];
};
struct Counters {   // cbeam.wgsl's Counters
    uint32_t level[32], verts, last, lost[4], was[34];
};
constexpr uint32_t BEAM_BYTES = 19 * 16, VERT_FLOATS = 7;   // (cbeam.wgsl: BEAM vec4s)

struct Run {
    filament::Engine* engine = nullptr;
    door::Program *clear = nullptr, *seed = nullptr, *level = nullptr, *finish = nullptr;
    door::Buffer *frame = nullptr, *table = nullptr, *poses = nullptr, *counters = nullptr, *beamsA = nullptr, *beamsB = nullptr, *verts = nullptr;
    door::Set *clearSet = nullptr, *seedSet = nullptr, *finishSet = nullptr;
    std::vector<door::Set*> levelSets;   // [L - 1] for level L
    uint32_t beamCap = 0, vertCap = 0, levels = 0, seedMost = 0, faceRec = 0, solidsMost = 0;
};

// wgsl: the text of cbeam.wgsl. vertices: the buffer object the floor light is drawn from (vertCap vertices of 7 floats).
inline Run* open(filament::Engine* engine, const std::string& wgsl, const Table& tb, filament::BufferObject* vertices, uint32_t vertCap, uint32_t beamCap,
                 uint32_t levels, uint32_t solidsMost) {
    const std::string mark = "// ==== pass ";
    const size_t first = wgsl.find(mark);
    if (first == std::string::npos) return nullptr;
    auto pass = [&](const char* name) {
        const size_t at = wgsl.find(mark + name + "\n");
        if (at == std::string::npos) return std::string();
        const size_t end = wgsl.find(mark, at + mark.size());
        return wgsl.substr(0, first) + wgsl.substr(at, end == std::string::npos ? std::string::npos : end - at);
    };
    using door::Kind;
    const Kind slots[8] = {Kind::UNIFORM, Kind::STORAGE, Kind::STORAGE, Kind::STORAGE, Kind::STORAGE, Kind::STORAGE, Kind::STORAGE, Kind::UNIFORM};
    auto* r = new Run;
    r->engine = engine;
    const char* names[4] = {"clear", "seed", "level", "finish"};
    door::Program** programs[4] = {&r->clear, &r->seed, &r->level, &r->finish};
    for (int i = 0; i < 4; ++i) {
        const std::string text = pass(names[i]);
        if (text.empty()) return nullptr;
        *programs[i] = door::program(engine, (std::string("cbeam_") + names[i]).c_str(), text.c_str(), slots, 8);
    }
    r->beamCap = beamCap; r->vertCap = vertCap; r->levels = std::min<uint32_t>(31, std::max<uint32_t>(1, levels));
    r->seedMost = std::max<uint32_t>(1, tb.seedMost); r->faceRec = tb.faceRec; r->solidsMost = solidsMost;
    r->frame = door::buffer(engine, Kind::UNIFORM, sizeof(Frame));
    r->table = door::buffer(engine, Kind::STORAGE, (uint32_t)(tb.t.size() * sizeof(float)));
    door::write(engine, r->table, tb.t.data(), (uint32_t)(tb.t.size() * sizeof(float)));
    r->poses = door::buffer(engine, Kind::STORAGE, solidsMost * 64);
    r->counters = door::buffer(engine, Kind::STORAGE, sizeof(Counters));
    r->beamsA = door::buffer(engine, Kind::STORAGE, beamCap * BEAM_BYTES);
    r->beamsB = door::buffer(engine, Kind::STORAGE, beamCap * BEAM_BYTES);
    r->verts = door::buffer(engine, vertices);
    auto which = [&](uint32_t L) {
        door::Buffer* b = door::buffer(engine, Kind::UNIFORM, 16);
        // (y, z: experiments, see cbeam.wgsl. Only for the levels: the seeds must stay whole, or the levels have nothing real to cut)
        const uint32_t v[4] = {L, (uint32_t)(L && std::getenv("CBEAMSKIP") ? std::atoi(std::getenv("CBEAMSKIP")) : 0), (uint32_t)(L && std::getenv("CBEAMAGAIN") ? std::atoi(std::getenv("CBEAMAGAIN")) : 0), 0};
        door::write(engine, b, v, 16);
        return b;
    };
    auto set = [&](door::Program* p, door::Buffer* in, door::Buffer* out, uint32_t L) {
        door::Buffer* const b[8] = {r->frame, r->table, r->poses, r->counters, in, out, r->verts, which(L)};
        return door::set(engine, p, b);
    };
    r->clearSet = set(r->clear, r->beamsB, r->beamsA, 0);
    r->seedSet = set(r->seed, r->beamsB, r->beamsA, 0);       // level 0 goes into A
    for (uint32_t L = 1; L <= r->levels; ++L) r->levelSets.push_back(L % 2 ? set(r->level, r->beamsA, r->beamsB, L) : set(r->level, r->beamsB, r->beamsA, L));
    r->finishSet = set(r->finish, r->beamsB, r->beamsA, 0);
    return r;
}

// What changes from frame to frame. f.counts and f.caps are filled in here but for counts[1] (colour steps);
// poses: 16 floats a solid (axis x, size; axis y, shape; axis z, 0; place, 0).
inline void send(Run* r, Frame f, const float* poses, uint32_t solids) {
    f.counts[0] = solids; f.counts[2] = r->seedMost; f.counts[3] = r->beamCap;
    f.caps[0] = r->vertCap; f.caps[1] = r->faceRec; f.caps[2] = f.caps[3] = 0;
    door::write(r->engine, r->frame, &f, sizeof(Frame));
    if (solids) door::write(r->engine, r->poses, poses, solids * 64);
}
// The frame's compute passes, in order. stepsAll: colour steps, and one more if the mirrored light is on.
// `beforeFinish` (tests): called when all levels are queued and the counts are still there.
template <typename F>
inline void run(Run* r, uint32_t solids, uint32_t stepsAll, F beforeFinish) {
    door::run(r->engine, r->clear, r->clearSet, (r->vertCap + 63) / 64);
    door::run(r->engine, r->seed, r->seedSet, (solids * stepsAll * r->seedMost + 63) / 64);
    for (door::Set* s : r->levelSets) door::run(r->engine, r->level, s, (r->beamCap + 63) / 64);
    beforeFinish();
    door::run(r->engine, r->finish, r->finishSet, 1);
}
inline void run(Run* r, uint32_t solids, uint32_t stepsAll) { run(r, solids, stepsAll, [] {}); }

}  // namespace cbeam
