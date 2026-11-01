// deep.glsl: a thing with depth (Scale Z): its picture pushed through a box, Alight Motion's Raster Extrude as AbsentMotion
// writes it (effects/am_rasterextrude.wgsl, Apache-2.0): from where a ray comes into the box to where it goes out, 64
// steps through the picture's coverage, the first over a half closed in on six times; on the box's front or back it
// is the picture, else a side, facing down the coverage's slope. A ray that only brushes it is its soft edge. A side's
// shading, light and shine are that effect's own defaults; its front is left as drawn, the same as at Z 0. Here the ray is the world camera's, in the box's own lengths
// (deep_vertex.glsl), and the point it meets is written as the depth (Filament's custom depth, GLSLTools.cpp), so a
// thing crosses others where it really is. The one who reads it says deepThere (how much of it is at a point of its
// picture), deepCells (how many points its picture has) and deepAwayAt (how many from its edge).
// A bevel is one section, how high it stands by how far in from its edge, as a font's height field: After Effects'
// Angular, Convex and Concave, Easy Typography's Ogee and Step; carried on past the middle of a stroke it is Chizzle's
// chisel. It cuts the box's front and back alike, each ray's test being only whether a point is still under the section.
highp vec2 deepUV(highp vec2 q) {   // (a point of the box's front to its picture: its corners as the sheet's, video.h; v turned over as Filament turns a sheet's)
    vec2 o = mat2(materialParams.deepTurn.xy, materialParams.deepTurn.zw) * (q / materialParams.deep.xy) * 0.5 + 0.5;
    return vec2(o.x, 1.0 - o.y);
}
float deepCovered(highp vec2 q) { return all(lessThanEqual(abs(q), materialParams.deep.xy)) ? deepThere(deepUV(q)) : 0.0; }
float deepProfile(float t) {   // (how high it stands, 0 to 1, t of the way in)
    float way = materialParams.deepBevel.z;
    if (way < 1.5) return t;
    if (way < 2.5) return sqrt(max(t * (2.0 - t), 0.0));
    if (way < 3.5) return 1.0 - sqrt(max(1.0 - t * t, 0.0));
    if (way < 4.5) return t * t * (3.0 - 2.0 * t);
    float rise = t * 3.0;   // (three stairs; a quarter of each is its riser, or a stair's face is no wider than a line)
    return (min(floor(rise), 2.0) + clamp((fract(rise) - 0.75) * 4.0, 0.0, 1.0) * step(rise, 2.999) + step(2.999, rise)) / 3.0;
}
float deepAway(highp vec2 q) { return deepAwayAt(deepUV(q)) * 2.0 * materialParams.deep.x / deepCells().x; }
float deepIn(highp vec3 q) {   // (how much of it is at a point of its box)
    float a = deepCovered(q.xy);
    highp vec3 cut = materialParams.deepBevel.xyz;
    if (cut.z < 0.5 || a <= 0.0) return a;
    highp float under = materialParams.deep.z - abs(q.z);
    if (under >= cut.y) return a;
    highp float point = 2.0 * materialParams.deep.x / deepCells().x * cut.y / cut.x;   // (a point's width up its section: its edge as soft as a drawn one)
    return a * clamp(0.5 + (under - cut.y * (1.0 - deepProfile(clamp(deepAway(q.xy) / cut.x, 0.0, 1.0)))) / point, 0.0, 1.0);
}
// uv: the point of the picture seen here; k, shine: how it is shaded (AbsentMotion's: k times it, shine on top); edge:
// how much of it is there. Nothing seen here: not there at all, as a clear point of a sheet.
void deepMeet(inout highp vec2 uv, inout float k, inout float shine, inout float edge) {
    highp vec3 h = materialParams.deep.xyz, at = variable_deepAt.xyz;
    if (materialParams.deep.w < 0.5) {   // (Z 0: its box shut, one sheet: as flat as any)
        if (at.z < 0.0) discard;
        return;
    }
    bool eyed = variable_deepEye.w > 0.5;
    highp vec3 rd = eyed ? at - variable_deepEye.xyz : -1000.0 * variable_deepEye.xyz, ro = at - rd;   // (t is 1 here)
    highp vec3 inv = sign(rd + 1e-7) / max(abs(rd), vec3(1e-6)), t1 = (-h - ro) * inv, t2 = (h - ro) * inv, lo = min(t1, t2), hi = max(t1, t2);
    highp float t0 = max(max(lo.x, lo.y), max(lo.z, 0.0)), tEnd = min(hi.x, min(hi.y, hi.z));
    if (t0 >= tEnd || abs(tEnd - 1.0) > abs(t0 - 1.0)) discard;   // (a ray meets the box twice: where it goes out is seen, an eye within it sees out too)
    highp float dt = (tEnd - t0) / 64.0, t = t0, tp = t0;
    float peak = 0.0;
    bool hit = false;
    for (int i = 0; i <= 64; i++) {
        t = t0 + dt * float(i);
        float a = deepIn(ro + rd * t);
        if (a > peak) peak = a, tp = t;
        if (a > 0.5) { hit = true; break; }
    }
    if (!hit && peak < 0.02) discard;
    if (hit) {
        highp float low = max(t - dt, t0), high = t;
        for (int i = 0; i < 6; i++) {
            highp float mid = 0.5 * (low + high);
            if (deepIn(ro + rd * mid) > 0.5) high = mid; else low = mid;
        }
        t = high;
    } else {   // (only brushed: its nearest, closed in on, or the 64 steps' luck shows as a torn edge)
        highp float by = dt;
        for (int i = 0; i < 4; i++) {
            by *= 0.5;
            float a = deepIn(ro + rd * (tp - by)), b = deepIn(ro + rd * (tp + by));
            if (a > peak && a >= b) peak = a, tp -= by; else if (b > peak) peak = b, tp += by;
        }
        t = tp;
    }
    highp vec3 p = ro + rd * t;
#if defined(LIT)
    highp vec3 sideX = variable_copyOf.yzw, sideY = variable_deepUp.xyz, sideZ = normalize(cross(sideX, sideY)) * variable_deepUp.w;
#endif
    uv = deepUV(p.xy);
    edge = hit ? 1.0 : min(peak * 2.0, 1.0);
    highp vec4 eye = getWorldFromClipMatrix()[2];   // (deep_vertex.glsl)
    highp vec3 ahead = eyed ? vertex_worldPosition.xyz - eye.xyz / eye.w : -1000.0 * normalize(eye.xyz);
    shading_position = vertex_worldPosition.xyz + (t - 1.0) * ahead;
    highp vec4 clip = getClipFromWorldMatrix() * vec4(shading_position, 1.0);
    gl_FragDepth = clip.z / clip.w;
    highp vec3 cut = materialParams.deepBevel.xyz;
    highp float near = max(dt * abs(rd.z) * 1.5, 0.0005), along = cut.z > 0.5 ? clamp(deepAway(p.xy) / cut.x, 0.0, 1.0) : 1.0;
    bool bevelled = cut.z > 0.5 && along < 1.0 && h.z - abs(p.z) < cut.y + near;
    if (abs(p.z) > h.z - near && !bevelled) {   // (its front and back: the picture as it is, as at Z 0)
#if defined(LIT)
        shading_geometricNormal = normalize(sideZ * sign(p.z));
#endif
        return;
    }
    highp vec2 e = 8.0 * h.xy / deepCells();   // (four points of its picture, read round a ring: a slanted edge is in steps, and nearer or fewer they show as stripes down a side)
    vec3 n3;
    if (bevelled) {   // (a bevel faces out by how steep its section is here: its slope across, the way away from the edge)
        highp float by = 6.0 * h.x / deepCells().x;   // (three points: nearer, a slanted edge's steps show as stripes, as on a side)
        highp vec2 g = vec2(deepAway(p.xy + vec2(by, 0.0)) - deepAway(p.xy - vec2(by, 0.0)), deepAway(p.xy + vec2(0.0, by)) - deepAway(p.xy - vec2(0.0, by))) / (2.0 * by);
        float slope = (deepProfile(min(along + 0.02, 1.0)) - deepProfile(max(along - 0.02, 0.0))) / 0.04 * cut.y / cut.x * materialParams.deepBevel.w;   // (w: the box's lengths to the world's: it is the world's slope a lamp sees)
        n3 = normalize(vec3(-g * slope, sign(p.z)));
        if (dot(g, g) > 1e-4) uv = deepUV(p.xy + normalize(g) * e);   // (its colour from within its edge)
    } else {
        vec2 g = vec2(0.0);
        for (int i = 0; i < 12; i++) {
            vec2 round = vec2(cos(0.5235988 * float(i)), sin(0.5235988 * float(i)));
            g += round * deepCovered(p.xy + round * e);
        }
        vec2 n = length(g) > 1e-3 ? -normalize(g) : abs(p.x / h.x) > abs(p.y / h.y) ? vec2(sign(p.x), 0.0) : vec2(0.0, sign(p.y));   // (filled to its edge: the box's own side)
        uv = deepUV(p.xy - n * e);   // (its colour from within its edge)
        n3 = vec3(n, 0.0);
    }
#if defined(LIT)
    shading_geometricNormal = normalize(sideX * n3.x + sideY * n3.y + sideZ * n3.z);   // (in the lamps' light: they shade it, the world's normal; flat.mat turns it to the sheet's)
    return;
#endif
    // (AbsentMotion's own axes: x right, y down, z away)
    vec3 seen = normalize(mat3(getViewFromWorldMatrix()) * (variable_copyOf.yzw * n3.x + variable_deepUp.xyz * n3.y + normalize(cross(variable_copyOf.yzw, variable_deepUp.xyz)) * variable_deepUp.w * n3.z)) * vec3(1.0, -1.0, -1.0);
    vec3 way = normalize(mat3(getViewFromWorldMatrix()) * ahead) * vec3(1.0, -1.0, -1.0);
    vec3 light = normalize(vec3(cos(2.35619449), -sin(2.35619449), -1.2));   // (135 degrees)
    k = 0.4 + 0.6 * (0.3 + 0.85 * max(dot(seen, light), 0.0));
    shine = 0.3 * pow(max(dot(seen, normalize(light - way)), 0.0), 40.0);
}
