// Where a ray of light is dropped, when there is more than the floor (see "Receivers" in beam_glass.h). For the
// part of a material that puts light on a plane: ball_light.mat (vertex), cone_walk.mat, recv_light.mat (vertex).
// The including material needs the parameters
//   room      float4[6]   the room, as a.x + b >= 0 inside: the floor first, then the walls, and LAST where the walls
//                         end (that one receives nothing)
//   roomCount float       how many of those; 0 (not set): there is no stage, only the floor y = floorY, as ever
//   blocks    float4[16]  matte blocks: per block its middle, then three times unit axis (xyz) and half size (w)
//   blockCount, floorY
//
// recvChoose: the plane a ray from E along D (unit) is dropped on. N = unit normal towards the light and offset (its
// points: dot(N.xyz, x) = N.w), owner = -1 for the room, or the number of the block. False: nothing to drop it on.
//   kind 0, the room: the plane of the room the ray meets first. The room is convex and the ray starts inside, so
//           that is simply the nearest of the planes it runs into. Every corner chooses for itself: a triangle whose
//           corners chose differently lies across the crease like a chord, a triangle's width from the truth at most,
//           and its neighbours still share its edges. So a sheet of light runs from the floor up a wall without a gap
//           and without a doubled part, for nothing: no copies, no clipping.
//   kind 1, the blocks: the face through which the ray enters the first block it meets. A ray that meets none is
//           dropped on the face plane it comes nearest to entering by (the last plane it crosses of the block it
//           misses least), outside the face: so a triangle that hangs over the edge of a face lies in the face's
//           plane, and each pixel can cut it off exactly at the edge (recvOn in recv_lands.glsl). The same light is
//           also drawn as kind 0, where recvClear takes away what the blocks caught.
//           recvMiss says by about how much the ray misses (0: it meets a block). A corner that misses by more than
//           RECV_OVERHANG is not worth dropping: no triangle that long hangs over an edge, and left alone it and
//           its triangles cost nothing (the caller treats it as light that does not land).
const float RECV_OVERHANG = 0.15;
highp float recvMiss;
bool recvChoose(int kind, highp vec3 E, highp vec3 D, out highp vec4 N, out float owner) {
    N = vec4(0.0, 1.0, 0.0, materialParams.floorY);
    owner = -1.0;
    recvMiss = 0.0;
    if (materialParams.roomCount < 0.5) return true;
    if (kind == 0) {
        highp float best = 1e30;
        bool found = false;
        int planes = int(materialParams.roomCount + 0.5) - 1;
        for (int j = 0; j < 5; j++) {
            if (j >= planes) break;
            highp vec4 pl = materialParams.room[j];
            highp float heading = -dot(pl.xyz, D);
            if (heading < 1e-5) continue;
            highp float t = (dot(pl.xyz, E) + pl.w) / heading;
            if (t < best) {
                best = t;
                N = vec4(pl.xyz, -pl.w);
                found = true;
            }
        }
        return found;
    }
    int count = int(materialParams.blockCount + 0.5);
    highp float bestKey = 1e30;
    for (int i = 0; i < 4; i++) {
        if (i >= count) break;
        highp vec3 c = materialParams.blocks[4 * i].xyz;
        highp float t0 = -1e30, t1 = 1e30;
        highp vec4 enter = vec4(0.0, 1.0, 0.0, 0.0);
        bool past = false;
        for (int k = 1; k <= 3; k++) {
            highp vec4 ax = materialParams.blocks[4 * i + k];
            highp float o = dot(ax.xyz, E - c), d = dot(ax.xyz, D);
            if (abs(d) < 1e-6) {
                if (abs(o) > ax.w) past = true;
                continue;
            }
            highp float ta = (-ax.w - o) / d, tb = (ax.w - o) / d;
            if (min(ta, tb) > t0) {
                t0 = min(ta, tb);
                enter = vec4(d > 0.0 ? -ax.xyz : ax.xyz, ax.w);
            }
            t1 = min(t1, max(ta, tb));
        }
        if (past) continue;
        // met: how far to it. Missed: by how much, and after every block that is met.
        highp float key = (t0 < t1 && t1 > 0.0) ? t0 : 1e6 + (t0 - t1);
        if (key < bestKey) {
            bestKey = key;
            N = vec4(enter.xyz, dot(enter.xyz, c) + enter.w);
            owner = float(i);
        }
    }
    recvMiss = max(0.0, bestKey - 1e6);
    return bestKey < 1e29;
}
// recvNear: can light of a solid (middle c, nothing of it further than `size` from c) that flies no further than
// `reach` land on anything of this kind? The copy of a solid's light for the blocks is thrown out here when no block
// is near, before anything is worked out for it. Light that has flown `reach` has faded to nothing, so a copy comes
// and goes unseen.
bool recvNear(int kind, highp vec3 c, highp float size, highp float reach) {
    if (kind == 0 || materialParams.roomCount < 0.5) return true;
    int count = int(materialParams.blockCount + 0.5);
    for (int i = 0; i < 4; i++) {
        if (i >= count) break;
        highp vec3 rel = c - materialParams.blocks[4 * i].xyz;
        highp float far2 = 0.0;
        for (int k = 1; k <= 3; k++) {
            highp vec4 ax = materialParams.blocks[4 * i + k];
            highp float q = abs(dot(ax.xyz, rel)) - ax.w;
            if (q > 0.0) far2 += q * q;
        }
        if (far2 < (reach + size) * (reach + size)) return true;
    }
    return false;
}
// recvClear: how much of a corner's light gets from E (where it left the glass) to X (where it was dropped): 0 if a
// block stands in its way, or between the sun and E (the glass was in that block's shadow; E stands in for the place
// the sunlight entered the glass, a glass's width away at most). `owner`: the block X lies on (not in its own way).
// Asked per CORNER by the materials whose light is a fine mesh, so it is not a yes or no but comes in over a few
// centimetres of passing the block by: a yes or no per corner would make the edge of a block's shadow in the
// light jump from corner to corner as things move. (recv_light.mat, whose triangles are large, asks per pixel
// instead: recvBlocked in recv_lands.glsl.)
highp float recvPast(int i, highp vec3 a, highp vec3 ab, highp float far) {   // > 0: the line passes block i by, by about so much
    highp vec3 c = materialParams.blocks[4 * i].xyz;
    highp float t0 = 0.0, t1 = far;
    for (int k = 1; k <= 3; k++) {
        highp vec4 ax = materialParams.blocks[4 * i + k];
        highp float o = dot(ax.xyz, a - c), d = dot(ax.xyz, ab);
        if (abs(d) < 1e-6) {
            if (abs(o) > ax.w) return 1e3;
            continue;
        }
        highp float ta = (-ax.w - o) / d, tb = (ax.w - o) / d;
        t0 = max(t0, min(ta, tb));
        t1 = min(t1, max(ta, tb));
    }
    return t0 - t1;
}
float recvClear(highp vec3 E, highp vec3 X, highp vec3 recvSun, float owner) {
    int count = int(materialParams.blockCount + 0.5), own = int(floor(owner + 0.5));
    highp vec3 way = X - E;
    highp float far = length(way);
    if (!(far > 1e-6)) return 1.0;
    way /= far;
    float clear = 1.0;
    for (int i = 0; i < 4; i++) {
        if (i >= count) break;
        if (i != own) clear *= smoothstep(0.0, 0.03, recvPast(i, E, way, far));
        clear *= smoothstep(0.0, 0.03, recvPast(i, E, -recvSun, 1e6));
    }
    return clear;
}
