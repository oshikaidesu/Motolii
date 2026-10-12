// GBEAMS=2: the exact beam splitting on the GPU with ONE fragment per beam (beam_split.h is the method and the check).
// A beam ("node") is written by a single fragment into the colour attachments of its level's render target, so its
// cut is worked out once. A post-process material may declare eight outputs, but Filament 1.77.2's Metal backend
// only sets up the first FOUR (MetalState.mm:62, `for (size_t i = 0; i < 4; i++)`; mrt_probe.mm and the notes in
// glass_scene.mm), so a node has to fit 16 numbers:
//   0: the direction of the light, x and y (the sign of z is in the meta number); its irradiance on the node's plane
//      (a leaf: how bright it lands on the floor); meta = kind + 4 cell + 64 faces met + 1024 the wall it lies on
//      + 32768 how many corners + 524288 if the direction's z is negative
//   1: the walls it goes on to, as bits; then corners
//   2, 3: corners. Eleven in all, ONE number each: where the corner is on the wall, as two whole numbers 0..4095
//      across the box of the wall's own corners (a + 4096 b).
// THIS IS NOT EXACT: a corner is put on a grid of 1/4096 of its wall, and a piece of light with twelve corners loses
// one. An exact node needs about 28 numbers (seven outputs). This version exists to MEASURE what one fragment per
// beam costs; see the notes in glass_scene.mm.
// kind: 0 none, 1 goes on inside, 2 has left the glass.
// `cells` (built once per shape): a line per (set of films in use, cell), 11 texels per wall (18 walls at most):
// plane (outward normal, offset); (what: >= 0 a face of the solid, < 0 a film; 0; how many corners; 0); the frame's
// two directions on the wall (its origin is normal * offset); the box of its corners in that frame; then the corners
// in that frame, up to 12, two to a texel.
const int GM_WALLS = 18, GM_WALL_W = 11, GM_CORNERS = 12;

float gmFresnel(float ci, float n1, float n2) {
    float s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) return 1.0;
    float ct = sqrt(1.0 - s2);
    float rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct);
    float rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}
vec4 gmWall(int line, int wall, int at) { return texelFetch(materialParams_cells, ivec2(wall * GM_WALL_W + at, line), 0); }
// keeps the part of a polygon (on a wall, in its frame) where a . p + b >= 0
void gmClip(inout vec2 P[12], inout int n, vec2 a, float b) {
    vec2 Q[12];
    int m = 0;
    for (int i = 0; i < 12; i++) {
        if (i >= n) break;
        int j = i + 1 < n ? i + 1 : 0;
        float gi = dot(a, P[i]) + b, gj = dot(a, P[j]) + b;
        if (gi >= 0.0 && m < 12) { Q[m] = P[i]; m++; }
        if ((gi >= 0.0) != (gj >= 0.0) && m < 12) { Q[m] = P[i] + (P[j] - P[i]) * (gi / (gi - gj)); m++; }
    }
    for (int i = 0; i < 12; i++) P[i] = i < m ? Q[i] : vec2(0.0);
    n = m;
}
float gmArea(vec2 P[12], int n) {
    float s = 0.0;
    for (int i = 1; i + 1 < 12; i++) {
        if (i + 1 >= n) break;
        vec2 a = P[i] - P[0], b = P[i + 1] - P[0];
        s += a.x * b.y - a.y * b.x;
    }
    return 0.5 * abs(s);
}
// A polygon on wall `from` carried along d onto wall `to` (both of the cell on `line`) and cut to that wall's
// outline: in `to`'s frame. A box round it is compared with the wall's own box first, only ever to tell early that
// nothing can be left.
void gmOnto(vec2 P[12], int n, int line, int from, int to, vec3 d, out vec2 Q[12], out int m) {
    vec4 pf = gmWall(line, from, 0), pt = gmWall(line, to, 0);
    vec3 uf = gmWall(line, from, 2).xyz, vf = gmWall(line, from, 3).xyz, of = pf.xyz * pf.w;
    vec3 ut = gmWall(line, to, 2).xyz, vt = gmWall(line, to, 3).xyz, ot = pt.xyz * pt.w;
    float db = dot(pt.xyz, d);
    vec2 lo = vec2(1e30), hi = vec2(-1e30);
    for (int i = 0; i < 12; i++) {
        Q[i] = vec2(0.0);
        if (i >= n) continue;
        vec3 x = of + uf * P[i].x + vf * P[i].y;
        x += d * ((pt.w - dot(pt.xyz, x)) / db);
        Q[i] = vec2(dot(x - ot, ut), dot(x - ot, vt));
        lo = min(lo, Q[i]);
        hi = max(hi, Q[i]);
    }
    m = n;
    vec4 box = gmWall(line, to, 4);
    if (lo.x > box.z || hi.x < box.x || lo.y > box.w || hi.y < box.y) { m = 0; return; }
    int cn = int(gmWall(line, to, 1).z + 0.5);
    vec2 c[12];
    vec2 mid = vec2(0.0);
    for (int k = 0; k < 6; k++) {
        vec4 two = gmWall(line, to, 5 + k);
        c[2 * k] = two.xy;
        c[2 * k + 1] = two.zw;
    }
    for (int k = 0; k < 12; k++) if (k < cn) mid += c[k];
    mid /= float(max(cn, 1));
    for (int k = 0; k < 12; k++) {
        if (k >= cn || m < 3) break;
        vec2 a = c[k], e = c[k + 1 < cn ? k + 1 : 0] - a;
        vec2 inward = vec2(-e.y, e.x);
        if (dot(inward, mid - a) < 0.0) inward = -inward;
        gmClip(Q, m, inward, -dot(inward, a));
    }
}

// a corner on a wall, to and from one number (see the top)
float gmPack(vec2 p, vec4 box) {
    vec2 q = floor(clamp((p - box.xy) / max(box.zw - box.xy, vec2(1e-12)), 0.0, 1.0) * 4095.0 + 0.5);
    return q.x + 4096.0 * q.y;
}
vec2 gmUnpack(float c, vec4 box) {
    float b = floor(c / 4096.0), a = c - 4096.0 * b;
    return box.xy + vec2(a, b) / 4095.0 * (box.zw - box.xy);
}
// the corners of a node, from its three pictures (the first number of the first is its walls)
void gmCorners(vec4 t1, vec4 t2, vec4 t3, int n, vec4 box, out vec2 P[12]) {
    float c[11];
    c[0] = t1.y; c[1] = t1.z; c[2] = t1.w;
    c[3] = t2.x; c[4] = t2.y; c[5] = t2.z; c[6] = t2.w;
    c[7] = t3.x; c[8] = t3.y; c[9] = t3.z; c[10] = t3.w;
    for (int i = 0; i < 12; i++) P[i] = i < n && i < 11 ? gmUnpack(c[i], box) : vec2(0.0);
}
