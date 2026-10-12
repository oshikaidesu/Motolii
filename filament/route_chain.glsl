// Shared by route_tree.mat and route_draw.mat (ROUTES=2): the way light goes through a flat-faced solid, worked out
// from where the faces and films ARE, in the solid's own frame. Needs the parameters planes, films and the sampler
// corners (per face, a line: its corners, up to 8, and in w of the first how many).
// A way is named by the faces and films met after the face the light came in by, in order: the place in `planes` + 1
// (films: 41 and up), six bits each, the first four in x and the next four in y.

// unpolarised Fresnel reflectance, n1 -> n2, 1 on total reflection
float routeFresnel(float ci, float n1, float n2) {
    float s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) return 1.0;
    float ct = sqrt(1.0 - s2);
    float rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct);
    float rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}
// A frame on face A to measure in: its first corner, the edge to its second, and the direction square to that
// (as long as that edge). A point of the face's plane is c0 + a e1 + b e2.
void routeFrame(int A, out vec3 c0, out vec3 e1, out vec3 e2) {
    c0 = texelFetch(materialParams_corners, ivec2(0, A), 0).xyz;
    e1 = texelFetch(materialParams_corners, ivec2(1, A), 0).xyz - c0;
    e2 = cross(normalize(materialParams.planes[A].xyz), e1);
}
vec2 routeAB(vec3 p, vec3 c0, vec3 e1, vec3 e2) {
    vec3 w = p - c0;
    return vec2(dot(w, e1) / dot(e1, e1), dot(w, e2) / dot(e2, e2));
}
// The way named by seq, for light of index n that came in by a face of outward normal nf from the sun's direction
// sunL: what it meets (met, ids) and which way it travels before each (dir[0]: straight after the face it came in
// by; dir[e + 1]: after the e-th has mirrored it). Returns how many it meets.
int routeChain(vec2 seq, vec3 sunL, vec3 nf, float n, out vec3 dir[9], out vec4 met[8], out int ids[8]) {
    int lo = int(seq.x + 0.5), hi = int(seq.y + 0.5), events = 0;
    for (int e = 0; e < 8; e++) { dir[e + 1] = vec3(0.0, 1.0, 0.0); met[e] = vec4(0.0, 1.0, 0.0, 0.0); ids[e] = -1; }
    dir[0] = refract(normalize(sunL), nf, 1.0 / n);
    for (int e = 0; e < 8; e++) {
        int id = (e < 4 ? (lo >> (6 * e)) : (hi >> (6 * (e - 4)))) & 63;
        if (id == 0) break;
        id -= 1;
        ids[e] = id;
        met[e] = id >= 40 ? materialParams.films[id - 40] : materialParams.planes[id];
        dir[e + 1] = reflect(dir[e], normalize(met[e].xyz));
        events = e + 1;
    }
    return events;
}
// A point on the plane of the `at`-th thing met (or, at == events, anywhere ahead), carried back against the light to
// the face it came in by.
vec3 routeBack(vec3 p, int at, vec3 dir[9], vec4 met[8], vec3 nf, float entryAt) {
    for (int j = at - 1; j >= 0; j--) p -= dir[j + 1] * ((dot(met[j].xyz, p) - met[j].w) / dot(met[j].xyz, dir[j + 1]));
    return p - dir[0] * ((dot(nf, p) - entryAt) / dot(nf, dir[0]));
}
// The box, in the frame of the face the light came in by, that face `face` fits into when carried back to it; the
// light has met `at` things before it.
void routeWindow(int face, int at, vec3 dir[9], vec4 met[8], vec3 nf, float entryAt, vec3 c0, vec3 e1, vec3 e2, out vec2 lo, out vec2 hi) {
    lo = vec2(1e9);
    hi = vec2(-1e9);
    int cn = int(texelFetch(materialParams_corners, ivec2(0, face), 0).w + 0.5);
    for (int k = 0; k < 8; k++) {
        if (k >= cn) break;
        vec2 q = routeAB(routeBack(texelFetch(materialParams_corners, ivec2(k, face), 0).xyz, at, dir, met, nf, entryAt), c0, e1, e2);
        lo = min(lo, q);
        hi = max(hi, q);
    }
}
