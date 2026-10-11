// Shared by extrude_find.mat, extrude_walk.mat, extrude_strip.mat, extrude_cap.mat and glass_solid.mat: solids with
// dents and holes (an outline of lines and arcs pushed through a thickness). This is beam_extrude.h line for line
// (outline_cast, ex_ctx, ex_shadow_at, ex_mirror_at, ex_wall_at, ex_cut_at, ex_find, ex_point_u ...); change them
// together. The material that includes this must have a parameter
//     { type : sampler2d, name : shapes, precision : high, format : float }     beam::ex_shape_table, three lines per shape
// and, if it defines EX_WITH_POSES before including,
//     { type : sampler2d, name : poses, precision : high, format : float }      beam::ex_pose, one column per solid

#define EX_MAX_EDGES 32
#define EX_COLUMNS 160
#define EX_STEPS 8
#define EX_CAP_STEPS 4
#define EX_FIND_ROWS 10
#define EX_ROW_SHADOW 0
#define EX_ROW_MIRROR 2
#define EX_ROW_WALL 7
#define EX_ROW_CUT 87
#define EX_ROWS 119
#define EX_TMIN 1e-5
#define EX_HALVINGS 10
#define EX_BOUNCES 8
#define EX_CAP 40.0
#define EX_GAP 1.0
#define EX_SOFT 0.1
#define EX_EDGE 0.25
#define EX_BIG 1e30
#define EX_OVER 2e-3
#define EX_FLAT 0.02
#define EX_REACH 8.0

struct ExShape {
    int row;        // its first line in the table
    int edges;
    int stretches;
    int tris;
    float h;        // half the thickness
    float leg;      // of the chamfer
    float reach;
};
ExShape exShape(int row) {
    vec4 a = texelFetch(materialParams_shapes, ivec2(0, row), 0), b = texelFetch(materialParams_shapes, ivec2(1, row), 0);
    ExShape s;
    s.row = row;
    s.edges = int(a.x + 0.5);
    s.stretches = int(a.y + 0.5);
    s.tris = int(a.z + 0.5);
    s.h = b.x;
    s.leg = b.y;
    s.reach = b.z;
    return s;
}
struct ExEdge {
    vec2 a;
    vec2 b;
    vec2 c;
    float r;      // 0: straight; > 0: an arc counter-clockwise round c; < 0: clockwise
    float sweep;
};
ExEdge exEdge(ExShape s, int e) {
    vec4 p = texelFetch(materialParams_shapes, ivec2(2 + 2 * e, s.row), 0), q = texelFetch(materialParams_shapes, ivec2(3 + 2 * e, s.row), 0);
    ExEdge ed;
    ed.a = p.xy;
    ed.b = p.zw;
    ed.c = q.xy;
    ed.r = q.z;
    ed.sweep = q.w;
    return ed;
}
// stretch m of the shape: (edge, u0, u1)
vec3 exStretch(ExShape s, int m) { return texelFetch(materialParams_shapes, ivec2(m, s.row + 1), 0).xyz; }
float exCross(vec2 a, vec2 b) { return a.x * b.y - a.y * b.x; }
vec2 exPoint(ExEdge e, float u) {
    if (u <= 0.0) return e.a;
    if (u >= 1.0) return e.b;
    if (e.r == 0.0) return e.a + (e.b - e.a) * u;
    float a = e.sweep * u, cs = cos(a), sn = sin(a);
    vec2 d = e.a - e.c;
    return e.c + vec2(d.x * cs - d.y * sn, d.x * sn + d.y * cs);
}
// out of the glass
vec2 exNormal(ExEdge e, float u) {
    if (e.r == 0.0) {
        vec2 ab = e.b - e.a;
        return normalize(vec2(ab.y, -ab.x));
    }
    float a = e.sweep * u, cs = cos(a), sn = sin(a);
    vec2 d = e.a - e.c;
    return vec2(d.x * cs - d.y * sn, d.x * sn + d.y * cs) / e.r;
}
// (a ray is started a hair inside the ends of an edge, so that it never starts on two edges at once)
float exIn(float u) { return clamp(u, 1e-4, 1.0 - 1e-4); }

struct ExHit {
    bool is;
    float t;
    int edge;
    vec2 n;
};
// The nearest place beyond tmin where o + t d crosses the outline. want > 0: only where it leaves the glass,
// < 0: only where it goes in, 0: either. (beam::outline_cast)
ExHit exCast(ExShape s, vec2 o, vec2 d, float tmin, int want) {
    ExHit hit;
    hit.is = false;
    hit.t = EX_BIG;
    hit.edge = -1;
    hit.n = vec2(1.0, 0.0);
    bool straight = false;
    for (int i = 0; i < EX_MAX_EDGES; i++) {
        if (i >= s.edges) break;
        ExEdge e = exEdge(s, i);
        vec2 ab = e.b - e.a;
        if (e.r == 0.0) {
            float den = exCross(d, ab);
            if (want > 0 ? den <= 0.0 : (want < 0 ? den >= 0.0 : den == 0.0)) continue;
            vec2 ao = e.a - o;
            float t = exCross(ao, ab) / den;
            if (!(t > tmin) || t >= hit.t) continue;
            float w = exCross(ao, d) / den;
            if (w < -1e-5 || w > 1.0 + 1e-5) continue;
            hit.t = t;
            hit.edge = i;
            hit.n = vec2(ab.y, -ab.x);
            straight = true;
        } else {
            vec2 oc = o - e.c;
            float A = dot(d, d), B = dot(oc, d), C = dot(oc, oc) - e.r * e.r, disc = B * B - A * C;
            if (!(disc > 0.0)) continue;
            float sq = sqrt(disc);
            for (int k = 0; k < 2; k++) {
                float t = (k == 1 ? -B + sq : -B - sq) / A;
                if (!(t > tmin) || t >= hit.t) continue;
                vec2 p = o + d * t;
                if (exCross(ab, p - e.a) * e.r > 1e-5 * e.r * e.r) continue;   // on the circle, but not on this arc
                vec2 n = (p - e.c) / e.r;
                float dn = dot(d, n);
                if (want > 0 ? dn <= 0.0 : (want < 0 ? dn >= 0.0 : false)) continue;
                hit.t = t;
                hit.edge = i;
                hit.n = n;
                straight = false;
            }
        }
    }
    hit.is = hit.edge >= 0;
    if (hit.is && straight) hit.n = normalize(hit.n);
    return hit;
}

float exFresnel(float ci, float n1, float n2) {
    float s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) return 1.0;
    float ct = sqrt(1.0 - s2);
    float rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct);
    float rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}
float exSmooth(float x) {
    x = clamp(x, 0.0, 1.0);
    return x * x * (3.0 - 2.0 * x);
}
// beam::detail::ex_kept / ex_let: light that is only just kept in the glass, or only just gets out, is dimmed
float exKept(float sin2) { return exSmooth((sin2 - 1.0) / EX_EDGE); }
float exLet(float sin2) { return exSmooth((1.0 - sin2) / EX_EDGE); }
int exMix(int key, int v) {
    int k = key * 31 + v + 1;
    return k - (k / 65521) * 65521;
}
int exNextStep(int k) { return k + 1 < EX_STEPS ? k + 1 : k - 1; }

#if defined(EX_WITH_POSES)
// Where the solid is, and the sun as the solid sees it. (beam::detail::ExCtx)
struct ExCtx {
    ExShape s;
    vec3 O;
    vec3 ex;
    vec3 ey;
    vec3 ez;
    float sc;
    vec3 Lw;       // the sun in the world
    vec3 L;        // and in the solid's own frame
    vec2 sh;       // which way the sunlight goes across the outline
    float lat;     // how much of the sun's direction lies across
    float size;
    float top;     // +1: the cap at +z has the sun, -1: the one at -z
    float rise;
    float floorY;
};
ExCtx exCtx(int solid, vec3 sunDir, float floorY) {
    vec4 a = texelFetch(materialParams_poses, ivec2(solid, 0), 0), b = texelFetch(materialParams_poses, ivec2(solid, 1), 0), c = texelFetch(materialParams_poses, ivec2(solid, 2), 0);
    ExCtx x;
    x.s = exShape(int(b.w + 0.5));
    x.O = a.xyz;
    x.sc = a.w;
    x.ex = b.xyz;
    x.ey = c.xyz;
    x.ez = cross(x.ex, x.ey);
    x.Lw = normalize(sunDir);
    x.L = vec3(dot(x.Lw, x.ex), dot(x.Lw, x.ey), dot(x.Lw, x.ez));
    x.lat = length(x.L.xy);
    x.sh = x.lat > 1e-6 ? x.L.xy / x.lat : vec2(0.0);
    x.size = sqrt(x.s.reach * x.s.reach + x.s.h * x.s.h) * x.sc;
    x.top = x.L.z < 0.0 ? 1.0 : -1.0;
    x.rise = abs(x.L.z) / max(x.lat, 1e-6);
    x.floorY = floorY;
    return x;
}
vec3 exWorld(ExCtx c, vec2 p, float z) { return c.O + (c.ex * p.x + c.ey * p.y + c.ez * z) * c.sc; }
vec3 exDir(ExCtx c, vec3 d) { return c.ex * d.x + c.ey * d.y + c.ez * d.z; }
// where light from P along D lands: x, distance flown, z
vec3 exLand(ExCtx c, vec3 P, vec3 D) {
    float t = max(0.0, (c.floorY - P.y) / min(D.y, -1e-6));
    return vec3(P.x + D.x * t, t, P.z + D.z * t);
}
// how much of the light is still drawn: 1, falling to 0 where it leaves too level or lands too far away (beam::ball_fade)
float exFade(ExCtx c, float flown, float down) {
    return exSmooth((EX_REACH - flown / c.size) / (0.5 * EX_REACH)) * exSmooth((down - EX_FLAT) / (4.0 * EX_FLAT));
}
// the sideways step of light through the caps
float exCapStep(ExCtx c, float n) { return 2.0 * c.s.h * c.lat / sqrt(n * n - c.lat * c.lat); }
float exArea(vec3 a, vec3 b, vec3 c, vec3 d) { return 0.5 * abs((c.x - a.x) * (d.z - b.z) - (d.x - b.x) * (c.z - a.z)); }

// -- what becomes of the light at one point of the outline (edge e, u along it)

// beam::detail::ex_shadow_at: (how much of the wall's height throws its own shadow, key)
vec2 exShadowAt(ExCtx c, ExEdge ed, float u) {
    ExHit hit = exCast(c.s, exPoint(ed, exIn(u)), c.sh, EX_TMIN, 0);
    if (hit.is) return vec2(min(2.0 * c.s.h, hit.t * c.rise), float(hit.edge + 1));
    return vec2(2.0 * c.s.h, 0.0);
}
// beam::detail::ex_mirror_at
struct ExMirror {
    float back;
    float at1;
    vec2 n1;
    int key;
};
ExMirror exMirrorAt(ExCtx c, ExEdge ed, float u, float longest) {
    ExMirror m;
    m.back = 0.0;
    m.at1 = EX_BIG;
    m.n1 = vec2(1.0, 0.0);
    m.key = 1;
    vec2 n = exNormal(ed, u);
    float sn = dot(c.sh, n);
    if (!(sn > 1e-6)) return m;
    vec2 from = exPoint(ed, exIn(u));
    ExHit hit = exCast(c.s, from, -c.sh, EX_TMIN, 0);
    if (hit.is && hit.t < longest) {
        m.back = hit.t;
        m.key = exMix(m.key, hit.edge + 1);
    } else {
        m.back = EX_BIG;
        m.key = exMix(m.key, 0);
    }
    hit = exCast(c.s, from, c.sh - n * (2.0 * sn), EX_TMIN, 1);
    if (hit.is && hit.t < longest) {
        m.at1 = hit.t;
        m.n1 = hit.n;
        m.key = exMix(m.key, hit.edge + 1);
    }
    return m;
}
// beam::detail::ex_wall_at
struct ExWall {
    float lift;
    float path;
    vec2 q;
    vec2 o;
    float weight;
    int key;
};
ExWall exWallAt(ExCtx c, ExEdge ed, float u, float n) {
    ExWall m;
    m.lift = EX_BIG;
    m.path = 0.0;
    m.q = vec2(0.0);
    m.o = c.L.xy;
    m.weight = -1.0;
    m.key = 1;
    vec2 nrm = exNormal(ed, u);
    float ci = -c.lat * dot(c.sh, nrm);
    if (!(ci > 1e-6)) return m;
    vec2 from = exPoint(ed, exIn(u));
    m.q = from;
    ExHit hit = exCast(c.s, from, -c.sh, EX_TMIN, 0);
    if (hit.is) {
        m.lift = hit.t * c.rise;
        m.key = exMix(m.key, hit.edge + 1);
    } else {
        m.key = exMix(m.key, 0);
    }
    vec3 d = refract(c.L, vec3(nrm, 0.0), 1.0 / n);
    float weight = 1.0 - exFresnel(ci, 1.0, n);
    vec2 at = from, d2 = d.xy;
    for (int b = 0; b <= EX_BOUNCES; b++) {
        hit = exCast(c.s, at, d2, EX_TMIN, 1);
        if (!hit.is) break;
        m.path += hit.t;
        at += d2 * hit.t;
        float co = dot(d2, hit.n), sin2 = n * n * (1.0 - co * co);
        if (sin2 >= 1.0) {
            d2 -= hit.n * (2.0 * co);
            weight *= exKept(sin2);
            m.key = exMix(m.key, hit.edge * 2 + 1);
            continue;
        }
        vec3 o = refract(vec3(d2, d.z), vec3(-hit.n, 0.0), n);
        m.weight = weight * (1.0 - exFresnel(co, n, 1.0)) * exLet(sin2);
        m.q = at;
        m.o = o.xy;
        m.key = exMix(m.key, hit.edge * 2);
        return m;
    }
    m.key = exMix(m.key, 4001);
    return m;
}
// beam::detail::ex_cut_at
struct ExCut {
    vec2 qin;
    vec2 qout;
    float e;
    float fade;
    float fly;
    int key;
};
ExCut exCutAt(ExCtx c, ExEdge ed, float u, float rim, float n) {
    ExCut m;
    m.qin = vec2(0.0);
    m.qout = vec2(0.0);
    m.e = 0.0;
    m.fade = 0.0;
    m.fly = 0.0;
    m.key = 1;
    float h = c.s.h, leg = c.s.leg;
    vec2 nn = exNormal(ed, u), p = exPoint(ed, u), from = exPoint(ed, exIn(u));
    vec3 cut = vec3(nn, rim) * 0.70710678;
    float ci = -dot(c.L, cut);
    if (!(ci > 1e-6)) return m;
    float open = 1.0;
    ExHit hit;
    if (rim != c.top && c.lat > 1e-6) {
        hit = exCast(c.s, from, -c.sh, EX_TMIN, 0);
        if (hit.is) {
            // the rim away from the sun: lit only if the sun gets past the rest of the solid
            open = clamp((hit.t * c.rise - (2.0 * h - leg)) / leg, 0.0, 1.0);
            m.key = exMix(m.key, hit.edge + 1);
        }
    }
    vec3 d = refract(c.L, cut, 1.0 / n);
    if (!(open > 0.0)) {
        m.key = exMix(m.key, 4001);
        return m;
    }
    vec3 pin = vec3(p - nn * leg, rim * h), pout = vec3(p, rim * (h - leg));
    vec3 at = vec3(from - nn * (0.5 * leg), rim * (h - 0.5 * leg));
    float weight = 1.0 - exFresnel(ci, 1.0, n);
    for (int b = 0; b <= EX_BOUNCES; b++) {
        float tc = EX_BIG;
        if (d.z > 1e-6) tc = (h - at.z) / d.z;
        else if (d.z < -1e-6) tc = (-h - at.z) / d.z;
        vec3 N;
        float t;
        int id;
        hit = exCast(c.s, at.xy, d.xy, EX_TMIN, 1);
        if (hit.is && hit.t < tc) {
            t = hit.t;
            N = vec3(hit.n, 0.0);
            id = hit.edge;
        } else if (tc < 0.5 * EX_BIG) {
            t = tc;
            N = vec3(0.0, 0.0, d.z > 0.0 ? 1.0 : -1.0);
            id = c.s.edges + (d.z > 0.0 ? 0 : 1);
        } else {
            break;
        }
        at += d * t;
        float off = dot(N, at), den = dot(N, d);
        pin += d * ((off - dot(N, pin)) / den);
        pout += d * ((off - dot(N, pout)) / den);
        float sin2 = n * n * (1.0 - den * den);
        if (sin2 >= 1.0) {
            d -= N * (2.0 * den);
            weight *= exKept(sin2);
            m.key = exMix(m.key, id * 2 + 1);
            continue;
        }
        vec3 o = refract(d, -N, n);
        m.key = exMix(m.key, id * 2);
        vec3 D = exDir(c, o);
        if (!(D.y < -1e-6)) break;
        vec3 a = exLand(c, exWorld(c, pin.xy, pin.z), D), b2 = exLand(c, exWorld(c, pout.xy, pout.z), D), mid = exLand(c, exWorld(c, at.xy, at.z), D);
        m.qin = a.xz;
        m.qout = b2.xz;
        m.e = ci * weight * (1.0 - exFresnel(den, n, 1.0)) * exLet(sin2) * open;
        m.fade = exFade(c, mid.y, -D.y);
        m.fly = mid.y;
        return m;
    }
    m.key = exMix(m.key, 4001);
    return m;
}

// beam::detail::ex_point_u / ex_point_at: the five points of a stretch the strips are hung on. st: exStretch,
// f: the stretch's texel of step one.
float exPointU(vec3 st, vec4 f, int which) {
    if (which == 0) return st.y;
    float ua = which < 3 ? f.x : f.z, ub = abs(which < 3 ? f.y : f.w), gap = ub - ua;
    float u = (which == 1 || which == 3) ? ua - gap : ub + gap;
    return min(st.z, max(st.y, u));
}
float exPointAt(vec3 st, vec4 f, int which) {
    if (which == 0) return st.y;
    return 0.5 * ((which < 3 ? f.x : f.z) + abs(which < 3 ? f.y : f.w));
}
#endif

#if defined(EX_WITH_SOLID)
// -- one sight line against the whole solid, in its own frame (for glass_solid.mat). The solid with sharp rims.

// from a point in or on the solid along d: how far to its surface, and the outward normal there (beam::extrude_leave)
float exLeave(ExShape s, vec3 o, vec3 d, out vec3 n) {
    float tc = EX_BIG;
    if (d.z > 1e-6) tc = (s.h - o.z) / d.z;
    else if (d.z < -1e-6) tc = (-s.h - o.z) / d.z;
    ExHit hit = exCast(s, o.xy, d.xy, EX_TMIN, 1);
    if (hit.is && hit.t < tc) {
        n = vec3(hit.n, 0.0);
        return hit.t;
    }
    n = vec3(0.0, 0.0, d.z > 0.0 ? 1.0 : -1.0);
    return tc < 0.5 * EX_BIG ? max(tc, 0.0) : 0.0;
}
// a ray from outside: does it meet the solid, how far, and the outward normal there (beam::extrude_meet / outline_scan)
bool exMeet(ExShape s, vec3 o, vec3 d, out float t, out vec3 n) {
    t = 0.0;
    n = vec3(0.0, 0.0, 1.0);
    // while the ray is between the planes of the caps
    float t0 = -EX_BIG, t1 = EX_BIG;
    if (abs(d.z) > 1e-6) {
        float a = (-s.h - o.z) / d.z, b = (s.h - o.z) / d.z;
        t0 = min(a, b);
        t1 = max(a, b);
    } else if (abs(o.z) >= s.h) {
        return false;
    }
    float tmin = 1e-4;
    if (!(t1 > tmin)) return false;
    float from = max(t0, tmin), best = EX_BIG;
    int crossings = 0;
    vec2 o2 = o.xy, d2 = d.xy, bn = vec2(1.0, 0.0);
    for (int i = 0; i < EX_MAX_EDGES; i++) {
        if (i >= s.edges) break;
        ExEdge e = exEdge(s, i);
        vec2 ab = e.b - e.a;
        if (e.r == 0.0) {
            float den = exCross(d2, ab);
            if (den == 0.0) continue;
            vec2 ao = e.a - o2;
            float tt = exCross(ao, ab) / den;
            if (!(tt > from)) continue;
            float w = exCross(ao, d2) / den;
            if (w < 0.0 || w >= 1.0) continue;   // (half open: a corner counts for one of its two edges)
            crossings++;
            if (den < 0.0 && tt < t1 && tt < best) {
                best = tt;
                bn = normalize(vec2(ab.y, -ab.x));
            }
        } else {
            vec2 oc = o2 - e.c;
            float A = dot(d2, d2), B = dot(oc, d2), C = dot(oc, oc) - e.r * e.r, disc = B * B - A * C;
            if (!(disc > 0.0)) continue;
            float sq = sqrt(disc);
            for (int k = 0; k < 2; k++) {
                float tt = (k == 1 ? -B + sq : -B - sq) / A;
                if (!(tt > from)) continue;
                vec2 p = o2 + d2 * tt;
                if (exCross(ab, p - e.a) * e.r > 0.0) continue;
                crossings++;
                vec2 nn = (p - e.c) / e.r;
                if (dot(d2, nn) < 0.0 && tt < t1 && tt < best) {
                    best = tt;
                    bn = nn;
                }
            }
        }
    }
    if (crossings - (crossings / 2) * 2 == 1) {
        if (!(t0 > tmin)) return false;   // it starts inside the glass
        t = t0;
        n = vec3(0.0, 0.0, d.z > 0.0 ? -1.0 : 1.0);
        return true;
    }
    if (best > 0.5 * EX_BIG) return false;
    t = best;
    n = vec3(bn, 0.0);
    return true;
}
// The face a point of the drawn mesh lies on: `facet` (unit, towards the viewer) is the normal of the mesh's triangle
// there. Caps and straight walls are what the mesh says; on an arc the mesh has facet sides, so the true normal is
// taken from the arc (and `snapped` is the point put on the true wall); a chamfer leans 45 degrees between the two.
vec3 exFace(ExShape s, vec3 p, vec3 facet, out vec3 snapped) {
    snapped = p;
    if (abs(facet.z) > 0.92) return vec3(0.0, 0.0, facet.z > 0.0 ? 1.0 : -1.0);
    vec2 n2 = normalize(facet.xy);
    float best = 2.0 * s.leg + 0.02;
    for (int i = 0; i < EX_MAX_EDGES; i++) {
        if (i >= s.edges) break;
        ExEdge e = exEdge(s, i);
        if (e.r == 0.0) continue;
        vec2 d = p.xy - e.c;
        float len = length(d), off = abs(len - abs(e.r));
        vec2 nn = d / (len * sign(e.r));
        if (off < best && exCross(e.b - e.a, p.xy - e.a) * e.r <= 1e-3 && dot(nn, normalize(facet.xy)) > 0.9) {
            best = off;
            n2 = nn;
            if (abs(facet.z) < 0.3) snapped = vec3(e.c + d * (abs(e.r) / len), p.z);
        }
    }
    if (abs(facet.z) < 0.3) return vec3(n2, 0.0);
    return normalize(vec3(n2, facet.z > 0.0 ? 1.0 : -1.0));
}
#endif
