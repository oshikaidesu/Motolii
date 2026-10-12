// Shared by gbeam_node.mat (and its notes by gbeam_index.mat, gbeam_draw.mat): GBEAMS=1, the exact beam splitting of
// beam_split.h done on the GPU, level by level in tables. Needs the sampler `cells` and the parameters named below.
//
// `cells` (built once per shape, glass_scene.mm): a line per (set of films in use, cell); the films a colour is
// mirrored by cut the solid into convex cells, 16 lines per set. Across, 14 texels per wall of the cell (18 walls at
// most): its plane (outward normal, offset); (what: >= 0 a face of the solid, < 0 a film; 0; how many corners; 0);
// then its corners, up to 12, in the solid's own frame.
//
// A node of a level's table is 12 texels: (direction of the light, irradiance on the polygon's plane);
// (the cosine the light makes with that plane's normal, kind + 4 cell + 64 faces met + 1024 walls it goes on to,
// those walls as bits, how many corners); then up to 10 corners. kind: 0 none, 1 goes on inside, 2 has left the
// glass (then the first texel is the direction outside and how bright it lands on the floor).
const int GB_WALLS = 18, GB_WALL_W = 14, GB_NODE_W = 12, GB_CORNERS = 10;

float gbFresnel(float ci, float n1, float n2) {
    float s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) return 1.0;
    float ct = sqrt(1.0 - s2);
    float rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct);
    float rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}
// keeps the part of the polygon where a . p + b >= 0
void gbClip(inout vec3 P[12], inout int n, vec3 a, float b) {
    vec3 Q[12];
    int m = 0;
    for (int i = 0; i < 12; i++) {
        if (i >= n) break;
        int j = i + 1 < n ? i + 1 : 0;
        float gi = dot(a, P[i]) + b, gj = dot(a, P[j]) + b;
        if (gi >= 0.0 && m < 12) { Q[m] = P[i]; m++; }
        if ((gi >= 0.0) != (gj >= 0.0) && m < 12) { Q[m] = P[i] + (P[j] - P[i]) * (gi / (gi - gj)); m++; }
    }
    for (int i = 0; i < 12; i++) P[i] = i < m ? Q[i] : vec3(0.0);
    n = m;
}
float gbArea(vec3 P[12], int n) {
    vec3 s = vec3(0.0);
    for (int i = 1; i + 1 < 12; i++) {
        if (i + 1 >= n) break;
        s += cross(P[i] - P[0], P[i + 1] - P[0]);
    }
    return 0.5 * length(s);
}
vec4 gbWall(int line, int wall, int at) { return texelFetch(materialParams_cells, ivec2(wall * GB_WALL_W + at, line), 0); }
// The polygon carried along d onto wall `wall` of the cell on `line`, and cut to that wall's outline.
void gbOnto(inout vec3 P[12], inout int n, vec3 d, int line, int wall) {
    vec4 pl = gbWall(line, wall, 0);
    int cn = int(gbWall(line, wall, 1).z + 0.5);
    float db = dot(pl.xyz, d);
    for (int i = 0; i < 12; i++) {
        if (i >= n) break;
        P[i] += d * ((pl.w - dot(pl.xyz, P[i])) / db);
    }
    vec3 c[12];
    vec3 mid = vec3(0.0);
    for (int k = 0; k < 12; k++) {
        c[k] = k < cn ? gbWall(line, wall, 2 + k).xyz : vec3(0.0);
        mid += c[k];
    }
    mid /= float(max(cn, 1));
    for (int k = 0; k < 12; k++) {
        if (k >= cn || n < 3) break;
        vec3 a = c[k], b = c[k + 1 < cn ? k + 1 : 0];
        vec3 inward = cross(pl.xyz, b - a);
        if (dot(inward, mid - a) < 0.0) inward = -inward;
        gbClip(P, n, inward, -dot(inward, a));
    }
}
// Which walls of its cell a polygon of light goes on to (as bits), and how many: those ahead of it that some of it
// reaches and that carry at least materialParams.least. faces: how many faces of the solid it has met.
void gbAhead(vec3 P[12], int n, vec3 d, float irr, float cosOn, int line, int faces, float index, out int bits, out int count) {
    bits = 0;
    count = 0;
    for (int w = 0; w < GB_WALLS; w++) {
        vec4 info = gbWall(line, w, 1);
        if (info.z < 2.5) continue;
        vec4 pl = gbWall(line, w, 0);
        float db = dot(pl.xyz, d);
        if (!(db > 1e-7)) continue;
        vec3 Q[12];
        for (int i = 0; i < 12; i++) Q[i] = P[i];
        int m = n;
        gbOnto(Q, m, d, line, w);
        if (m < 3) continue;
        float a = gbArea(Q, m);
        if (!(a > 1e-10) || irr * db / cosOn * a * db < materialParams.least) continue;
        // (a face that would mirror it once more than is followed is not gone on to)
        if (info.x > -0.5 && gbFresnel(db, index, 1.0) >= 1.0 && faces + 1 >= int(materialParams.mostFaces + 0.5)) continue;
        bits |= 1 << w;
        count++;
    }
}
