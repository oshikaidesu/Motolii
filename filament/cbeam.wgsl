// CBEAMS=1: beam_split.h on the GPU, as compute (Filament's WebGPU backend, Dawn; see dawn_probe/compute_door.h).
// The same splitting, cut for cut: a beam is a flat polygon of light and its one direction; the faces of its cell
// that lie ahead cut it along their outlines; a piece that leaves the glass is written, as triangles, straight into
// the vertex buffer the floor light is drawn from; a piece that goes on inside is put down for the next level.
// One invocation is one beam. A level is one dispatch; levels take turns between two buffers of beams.
// The CPU sends the poses, the sun and the colour steps each frame, nothing else, and reads nothing back.
// Numbers are 32-bit here (64-bit in beam_split.h).
//
// The file is cut at the lines "// ==== pass" : what is above the first is common to all four programs.
// Slots (compute_door.h; slot i is binding 2 i): 0 frame, 1 the table of shapes, 2 poses, 3 counters, 4 beams in,
// 5 beams out, 6 vertices, 7 which level.
//
// The table T (cbeam.h), in vec4s:
//   T[shape * 16 + use] = (-, -, first seed, seeds)          use: one bit per film in use (split_shape)
//   a seed  = (cell, face)                                   every face of the solid itself, by cell
//   a cell  = (first face, faces, -, -), (the cell across film 0, 1, 2, 3; -1: none)
//   a face  = (plane: normal out, offset), (what, corners, -, -), (a ball round it: middle, radius), then per corner:
//             (corner), (the half the edge from this corner keeps: a, b; a . p + b >= 0)
//                                                            what >= 0: that face of the solid; < 0: film -(what + 1)
//             (faces take frame.caps.y vec4s each)

const CAP : i32 = 16;   // corners a piece of light may have

struct Frame {
    sun : vec4f,      // xyz: the way the sun's light goes, w: the floor's height
    band : vec4f,     // the band of sun on the floor: across (x, z), middle, half width
    misc : vec4f,     // band every, least light a piece must carry, faces a way may meet, 1: the light mirrored off the outside too
    counts : vec4u,   // solids, colour steps, most seeds of a shape, beams a level can hold
    caps : vec4u,     // vertices the buffer can hold, vec4s per face in T
    steps : array<vec4f, 16>,   // rgb, index of refraction
    films : array<vec4f, 4>,    // film f mirrors light of an index between x and y
};
struct Counters {
    level : array<atomic<u32>, 32>,   // beams put down at each level
    verts : atomic<u32>,              // vertices written this frame
    last : atomic<u32>,               // ... and the frame before
    lost : array<atomic<u32>, 4>,     // found no room: 0 beams, 1 vertices, 2 corners, 3 bands of sun
    was : array<atomic<u32>, 34>,     // the frame before: level[], verts, lost beams (for tests to read)
};
// A beam is BEAM vec4s in a plain array (a struct with an array in it is written many times slower: measured):
//   0: xyz where it goes (the solid's own frame), w irradiance on its plane (sun = 1)
//   1: x the cosine it makes with its plane's normal, y index of refraction, z solid, w colour step
//   2: cell, faces of the solid met, events, corners
//   3..: the corners
const BEAM : u32 = 19u;

@group(0) @binding(0) var<uniform> frame : Frame;
@group(0) @binding(2) var<storage, read_write> T : array<vec4f>;
@group(0) @binding(4) var<storage, read_write> poses : array<vec4f>;   // per solid: (axis x, size), (axis y, shape), (axis z, -), (place, -)
@group(0) @binding(6) var<storage, read_write> counters : Counters;
@group(0) @binding(8) var<storage, read_write> beamsIn : array<vec4f>;
@group(0) @binding(10) var<storage, read_write> beamsOut : array<vec4f>;
@group(0) @binding(12) var<storage, read_write> verts : array<f32>;    // 7 a vertex: x, y, z, r, g, b, distance flown
@group(0) @binding(14) var<uniform> level : vec4u;

var<private> P : array<vec3f, 16>;   // the piece being cut
var<private> PN : i32;

struct Pose { r0 : vec3f, r1 : vec3f, r2 : vec3f, at : vec3f, size : f32, shape : u32 };
fn pose(k : u32) -> Pose {
    let a = poses[k * 4u];
    let b = poses[k * 4u + 1u];
    return Pose(a.xyz, b.xyz, poses[k * 4u + 2u].xyz, poses[k * 4u + 3u].xyz, a.w, u32(b.w));
}
fn turn(ps : Pose, v : vec3f) -> vec3f { return ps.r0 * v.x + ps.r1 * v.y + ps.r2 * v.z; }

fn fres(ci : f32, n1 : f32, n2 : f32) -> f32 {
    let s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) { return 1.0; }
    let ct = sqrt(1.0 - s2);
    let rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct);
    let rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}
struct Bent { ok : bool, d : vec3f };
fn bend(d : vec3f, nrm : vec3f, eta : f32) -> Bent {   // nrm against d; eta = n from / n to
    let c = -dot(nrm, d);
    let k = 1.0 - eta * eta * (1.0 - c * c);
    if (k < 0.0) { return Bent(false, d); }
    return Bent(true, d * eta + nrm * (eta * c - sqrt(k)));
}

// keeps the part of P where h.xyz . p + h.w >= 0
fn clip(h : vec4f) {
    var g : array<f32, 16>;
    var anyNeg = false;
    for (var q = 0; q < PN; q++) {
        g[q] = dot(h.xyz, P[q]) + h.w;
        if (g[q] < 0.0) { anyNeg = true; }
    }
    if (!anyNeg) { return; }
    var O : array<vec3f, 16>;
    var m = 0;
    var over = false;
    for (var i = 0; i < PN; i++) {
        var j = i + 1;
        if (j == PN) { j = 0; }
        let a = g[i];
        let b = g[j];
        if (a >= 0.0) {
            if (m < CAP) { O[m] = P[i]; m++; } else { over = true; }
        }
        if ((a >= 0.0) != (b >= 0.0)) {
            if (m < CAP) { O[m] = P[i] + (P[j] - P[i]) * (a / (a - b)); m++; } else { over = true; }
        }
    }
    if (over) { atomicAdd(&counters.lost[2], 1u); }
    P = O;
    PN = m;
}
fn area() -> f32 {
    var s = vec3f(0.0);
    for (var i = 1; i + 1 < PN; i++) { s += cross(P[i] - P[0], P[i + 1] - P[0]); }
    return 0.5 * length(s);
}

// P leaves the glass along `out`: down to the floor, as triangles into the vertex buffer
fn land(ps : Pose, out : vec3f, rgb : vec3f, irrOnPlane : f32, cosOut : f32) {
    let dW = turn(ps, out);
    if (!(dW.y < -1e-6)) { return; }
    if ((level.y & 1u) != 0u) { return; }
    let c = rgb * (irrOnPlane * (-dW.y) / cosOut);
    let need = u32(3 * (PN - 2));
    let first = atomicAdd(&counters.verts, need);
    if (first + need > frame.caps.x) {
        atomicAdd(&counters.lost[1], need);
        return;
    }
    var X : array<vec3f, 16>;   // x, z, distance flown
    for (var i = 0; i < PN; i++) {
        let pw = turn(ps, P[i]) * ps.size + ps.at;
        let fl = (frame.sun.w - pw.y) / dW.y;
        X[i] = vec3f(pw.x + dW.x * fl, pw.z + dW.z * fl, fl);
    }
    var v = first * 7u;
    for (var i = 1; i + 1 < PN; i++) {
        for (var k = 0; k < 3; k++) {
            let x = X[select(i + k - 1, 0, k == 0)];
            verts[v] = x.x;
            verts[v + 1u] = 0.0;
            verts[v + 2u] = x.y;
            verts[v + 3u] = c.x;
            verts[v + 4u] = c.y;
            verts[v + 5u] = c.z;
            verts[v + 6u] = x.z;
            v += 7u;
        }
    }
}

// Where the beam that was given number `at` lies. Numbers are handed out one after another to whichever threads ask at
// that moment; were the beams put down in that order, threads working at the same time would write side by side,
// and that is many times slower (measured: the same writes, 4.5 ms against 0.7). So they are spread: an odd
// multiplier on a power of two of places gives every number its own place.
fn place(at : u32) -> u32 { return (at * 7919u) & (frame.counts.w - 1u); }

// P goes on inside: put down for level L
fn push(L : u32, d : vec3f, irr : f32, cosOn : f32, n : f32, cell : u32, faces : u32, events : u32, w : vec2f) {
    if ((level.y & 2u) != 0u) { return; }
    let at = atomicAdd(&counters.level[L], 1u);
    if (at >= frame.counts.w) {
        atomicAdd(&counters.lost[0], 1u);
        return;
    }
    if ((level.y & 4u) != 0u) { return; }
    var b = place(at) * BEAM;
    if ((level.y & 16u) != 0u) { b = at * BEAM; }   // (experiment: one after another, as they come)
    if ((level.y & 64u) != 0u) {                    // (experiment: the same numbers into the vertex buffer instead)
        let v = (at % 4096u) * 76u;
        verts[v] = d.x; verts[v + 1u] = d.y; verts[v + 2u] = d.z; verts[v + 3u] = irr;
        verts[v + 4u] = cosOn; verts[v + 5u] = n; verts[v + 6u] = w.x; verts[v + 7u] = w.y;
        verts[v + 8u] = f32(cell); verts[v + 9u] = f32(faces); verts[v + 10u] = f32(events); verts[v + 11u] = f32(PN);
        for (var i = 0; i < PN; i++) { verts[v + 12u + 4u * u32(i)] = P[i].x; verts[v + 13u + 4u * u32(i)] = P[i].y; verts[v + 14u + 4u * u32(i)] = P[i].z; }
        return;
    }
    if ((level.y & 128u) != 0u) {                   // (experiment: only the numbers that do not come from the cutting)
        beamsOut[b] = vec4f(1.0, 2.0, 3.0, 4.0);
        beamsOut[b + 1u] = vec4f(5.0, 6.0, 7.0, 8.0);
        beamsOut[b + 2u] = vec4f(9.0, 10.0, 11.0, 12.0);
        return;
    }
    beamsOut[b] = vec4f(d, irr);
    beamsOut[b + 1u] = vec4f(cosOn, n, w);
    beamsOut[b + 2u] = vec4f(f32(cell), f32(faces), f32(events), f32(PN));
    if ((level.y & 8u) != 0u) { return; }
    for (var i = 0; i < PN; i++) { beamsOut[b + 3u + u32(i)] = vec4f(P[i], 0.0); }
}

// ==== pass clear
// the vertices of the frame before, away (one invocation a vertex)
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id : vec3u) {
    if (id.x >= min(atomicLoad(&counters.last), frame.caps.x)) { return; }
    for (var k = 0u; k < 7u; k++) { verts[id.x * 7u + k] = 0.0; }
}

// ==== pass seed
// the light as it comes in: every sunlit face of every solid, by the cells it belongs to, cut to the band of sun.
// One invocation is (solid, colour step, seed of its shape).
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id : vec3u) {
    let stepsAll = frame.counts.y + u32(frame.misc.w);
    let slot = id.x % frame.counts.z;
    let step = (id.x / frame.counts.z) % stepsAll;
    let solid = id.x / frame.counts.z / stepsAll;
    if (solid >= frame.counts.x) { return; }
    let mirrored = step == frame.counts.y;
    let n = frame.steps[select(step, frame.counts.y / 2u, mirrored)].w;
    let rgb = select(frame.steps[min(step, 15u)].xyz, vec3f(1.0), mirrored);
    var films = 0u;
    for (var f = 0u; f < 4u; f++) {
        if (!mirrored && n > frame.films[f].x && n < frame.films[f].y) { films |= 1u << f; }
    }
    let ps = pose(solid);
    let entry = T[ps.shape * 16u + films];
    if (slot >= u32(entry.w)) { return; }
    let seed = T[u32(entry.z) + slot];
    let cell = u32(seed.x);
    let fr = u32(seed.y);
    let nf = T[fr].xyz;
    let sun = frame.sun.xyz;
    let sunL = normalize(vec3f(dot(sun, ps.r0), dot(sun, ps.r1), dot(sun, ps.r2)));
    let ci = -dot(nf, sunL);
    if (!(ci > 1e-6)) { return; }
    let corners = i32(T[fr + 1u].y);

    // the band of sun, as halves in the solid's own frame: g(p) = m . world(p) - c, kept where |g - k every| <= half
    let half = frame.band.w;
    let every = frame.misc.x;
    let mW = vec3f(frame.band.x, -(frame.band.x * sun.x + frame.band.y * sun.z) / min(sun.y, -1e-6), frame.band.y);
    let mL = vec3f(dot(mW, ps.r0), dot(mW, ps.r1), dot(mW, ps.r2)) * ps.size;
    let m0 = dot(mW, ps.at) - frame.band.z;
    var k0 = 0;
    var k1 = 0;
    if (half > 0.0 && every > 0.0) {
        var lo = 1e30;
        var hi = -1e30;
        for (var i = 0; i < corners; i++) {
            let g = dot(mL, T[fr + 3u + 2u * u32(i)].xyz) + m0;
            lo = min(lo, g);
            hi = max(hi, g);
        }
        k0 = i32(ceil((lo - half) / every));
        k1 = i32(floor((hi + half) / every));
        if (k1 - k0 > 31) {
            atomicAdd(&counters.lost[3], 1u);
            k1 = k0 + 31;
        }
    }
    let Rf = fres(ci, 1.0, n);
    for (var k = k0; k <= k1; k++) {
        PN = corners;
        for (var i = 0; i < corners; i++) { P[i] = T[fr + 3u + 2u * u32(i)].xyz; }
        if (half > 0.0) {
            let mid = m0 - f32(k) * every;
            clip(vec4f(mL, mid + half));
            if (PN >= 3) { clip(vec4f(-mL, -mid + half)); }
        }
        if (PN < 3 || area() < 1e-10) { continue; }
        if (mirrored) {
            land(ps, sunL + nf * (2.0 * ci), rgb, ci * Rf, ci);
            continue;
        }
        let b = bend(sunL, nf, 1.0 / n);
        if (!b.ok) { continue; }
        push(0u, b.d, ci * (1.0 - Rf), abs(dot(nf, b.d)), n, cell, 0u, 0u, vec2f(f32(solid), f32(step)));
    }
}

// ==== pass level
// each beam of the level before: cut by the faces of its cell that lie ahead
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id : vec3u) {
    let L = level.x;
    if (id.x >= min(atomicLoad(&counters.level[L - 1u]), frame.counts.w)) { return; }
    var b = place(id.x) * BEAM;
    if ((level.y & 16u) != 0u) { b = id.x * BEAM; }
    let d = beamsIn[b].xyz;
    let irrIn = beamsIn[b].w;
    let cosOn = beamsIn[b + 1u].x;
    let n = beamsIn[b + 1u].y;
    let w = beamsIn[b + 1u].zw;
    let m = vec4u(beamsIn[b + 2u]);
    let n0 = i32(m.w);
    var Q : array<vec3f, 16>;
    for (var i = 0; i < n0; i++) { Q[i] = beamsIn[b + 3u + u32(i)].xyz; }
    let ps = pose(u32(w.x));
    let rgb = frame.steps[u32(w.y)].xyz;
    let cell = m.x;
    let faceFirst = u32(T[cell].x);
    let faceCount = u32(T[cell].y);
    let across = T[cell + 1u];
    // a ball round the beam's polygon: the beam stays inside the tube that ball sweeps along d
    var mid = vec3f(0.0);
    for (var i = 0; i < n0; i++) { mid += Q[i]; }
    mid /= f32(n0);
    var reach = 0.0;
    for (var i = 0; i < n0; i++) { reach = max(reach, distance(Q[i], mid)); }
    // (level.y 512 / 1024 / 2048 / 4096: experiments, the work stopped after loading / after the facing test / after
    //  carrying the corners over / after the cutting)
    if ((level.y & 512u) != 0u) { return; }
    for (var again = 0u; again <= level.z; again++) {
    for (var j = 0u; j < faceCount; j++) {
        let fr = faceFirst + j * frame.caps.y;
        let plane = T[fr];
        let nB = plane.xyz;
        let db = dot(nB, d);
        if (!(db > 1e-7)) { continue; }
        // A face whose ball the tube passes by cannot be met: the cut would leave nothing. (Most faces ahead are
        // such: of the cuts beam_split.h makes, four in five leave nothing.) (level.y 256: experiment, not looked at)
        let ball = T[fr + 2u];
        let off = ball.xyz - mid;
        if ((level.y & 256u) == 0u && length(off - d * dot(off, d)) > (reach + ball.w) * 1.0001 + 1e-6) { continue; }
        if ((level.y & 1024u) != 0u) { continue; }
        let info = T[fr + 1u];
        let corners = i32(info.y);
        PN = n0;
        for (var i = 0; i < n0; i++) { P[i] = Q[i] + d * ((plane.w - dot(nB, Q[i])) / db); }
        if ((level.y & 2048u) != 0u) { continue; }
        for (var i = 0; i < corners && PN >= 3; i++) { clip(T[fr + 4u + 2u * u32(i)]); }
        if ((level.y & 4096u) != 0u) { continue; }
        if (PN < 3) { continue; }
        let a = area();
        let irr = irrIn * db / cosOn;
        if (!(a > 1e-10) || irr * a * db < frame.misc.y) { continue; }
        let what = i32(info.x);
        if (what >= 0) {
            let Ri = fres(db, n, 1.0);
            if (Ri < 1.0) {   // the glass lets it out
                let o = bend(d, -nB, n);
                if (o.ok) { land(ps, o.d, rgb, irr * (1.0 - Ri), abs(dot(nB, o.d))); }
            } else if (i32(m.y) + 1 < i32(frame.misc.z) && m.z < 30u) {
                push(L, d - nB * (2.0 * db), irr, db, n, cell, m.y + 1u, m.z + 1u, w);
            }
        } else {
            let film = u32(-(what + 1));
            if (m.z >= 30u) { continue; }
            if (n > frame.films[film].x && n < frame.films[film].y) {
                push(L, d - nB * (2.0 * db), irr, db, n, cell, m.y, m.z + 1u, w);
            } else if (across[film] >= 0.0) {
                push(L, d, irr, db, n, u32(across[film]), m.y, m.z + 1u, w);
            }
        }
    }
    }
}

// ==== pass finish
// the counts, kept for the next frame's clearing (and for tests), and set back to nothing
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id : vec3u) {
    if (id.x != 0u) { return; }
    for (var i = 0u; i < 32u; i++) {
        atomicStore(&counters.was[i], atomicLoad(&counters.level[i]));
        atomicStore(&counters.level[i], 0u);
    }
    atomicStore(&counters.was[32], atomicLoad(&counters.verts));
    atomicStore(&counters.was[33], atomicLoad(&counters.lost[0]));
    atomicStore(&counters.last, atomicLoad(&counters.verts));
    atomicStore(&counters.verts, 0u);
}
