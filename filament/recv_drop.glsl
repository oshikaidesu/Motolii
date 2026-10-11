// For the VERTEX part of a material that puts light or an outline on the floor: the same on a stage (walls, matte
// blocks; see "Receivers" in beam_glass.h). The material draws everything once per receiver (instances: the floor,
// each wall, then three faces of each block: the one across each of its axes that is turned to the sun), and calls
// recvDrop with where a corner lands on the floor. The corner is taken back from there ALONG THE SUN to where it
// started and dropped on this instance's receiver instead; the pixels are then cut to the part of that plane that is
// really surface (recvFinish in recv_finish.glsl). So a sheet of light or a shadow runs from the floor up a wall and
// over a block without a gap and without a doubled part, whatever made it, and no material needs to know more.
// THE LIE: light the glass has bent lands on a wall or a block as if the floor's picture had been thrown on it by
// a projector standing in the sun: in the wrong place by (height above the floor) x (how far it was bent). Shadows
// and straight-through light are exact. Light that never comes down to the floor is on no wall.
// The including material needs the parameters
//   room      float4[6]   the room, as a.x + b >= 0 inside: the floor first, then the walls, and LAST where the walls
//                         end (that one receives nothing)
//   roomCount float       how many of those; 0 (not set): there is no stage, only the floor, and nothing changes
//   blocks    float4[16]  matte blocks: per block its middle, then three times unit axis (xyz) and half size (w)
//   blockCount, recvCount (instances per thing drawn), recvChooses, recvSun (the way the sunlight travels), floorY
// Two ways, by `recvChooses`:
//   0  once per receiver, as above. Exact, for a material whose triangles are large (an outline): laid across a crease
//      like a chord a large triangle would be visibly wrong.
//   1  every corner chooses for itself what it lands on (recvChoose in recv_plane.glsl), and the material draws
//      everything once for the room (instance 0) and, if there are blocks, once more for the blocks (instance 1).
//      For a material whose light is a fine mesh: a small triangle whose corners chose differently lies across the
//      crease like a chord, its own width from the truth at most, and still shares its edges with its neighbours.
//      Far fewer instances.
#include "recv_plane.glsl"
int recvPer() { return materialParams.roomCount < 0.5 ? 1 : max(1, int(materialParams.recvCount + 0.5)); }
// Is X within `margin` of the part of the receiver's plane that is surface? (A corner further out than any triangle
// is long is not worth drawing: it would only be rasterised to be cut away.)
bool recvNearSurface(highp vec3 X, float owner, float margin) {
    if (owner < -0.5) {
        for (int j = 0; j < 6; j++) {
            if (float(j) >= materialParams.roomCount) break;
            highp vec4 pl = materialParams.room[j];
            if (dot(pl.xyz, X) + pl.w < -margin) return false;
        }
        return true;
    }
    int own = int(owner + 0.5);
    highp vec3 rel = X - materialParams.blocks[4 * own].xyz;
    for (int k = 1; k <= 3; k++) {
        highp vec4 ax = materialParams.blocks[4 * own + k];
        if (abs(dot(ax.xyz, rel)) > ax.w + margin) return false;
    }
    return true;
}
// r: which receiver (instance % recvPer()). floorXZ: where the corner lands on the floor; flown: how far it flew to get
// there. isLight: rgb is what the FLOOR gets, and is scaled to what this receiver gets (turned otherwise to the sun).
// Returns where to draw the corner; t: how far it flew to there; ok: far below 0 if this corner has no business on
// this receiver (the pixel part leaves its triangles out), else 1, or 3 if its light passes a block closely enough
// that the pixels must ask whether the block is in its way; owner: -1 the room, i block i; start: where the corner started.
highp vec3 recvDrop(int r, highp vec2 floorXZ, highp float flown, bool isLight, inout vec3 rgb, out float t, out float ok, out float owner,
                    out highp vec3 start) {
    highp vec3 Q = vec3(floorXZ.x, materialParams.floorY, floorXZ.y);
    t = flown;
    ok = 1.0;
    owner = -1.0;
    start = Q;
    if (materialParams.roomCount < 0.5) return Q;
    highp vec3 L = materialParams.recvSun;
    start = Q - L * flown;
    highp vec4 N;
    bool chooses = materialParams.recvChooses > 0.5;
    if (chooses) {
        if (!recvChoose(r, start, L, N, owner) || recvMiss > RECV_OVERHANG) {
            ok = -1000.0;
            // A corner with no business on any block goes to the middle of a block: a triangle none of whose corners
            // has any is then a single point and is not drawn at all, and one that has a corner there is hidden
            // inside the block. (Left where it started, the whole copy of the light for the blocks was rasterised
            // again as large as it lies on the floor, only to be cut away pixel by pixel: 10 to 20 ms for 36 round solids.)
            if (r == 1 && materialParams.blockCount > 0.5) return materialParams.blocks[4 * int(max(owner, 0.0) + 0.5)].xyz;
            return start;
        }
    } else {
        int planes = int(materialParams.roomCount + 0.5) - 1;
        if (r < planes) {
            highp vec4 pl = materialParams.room[r];
            N = vec4(pl.xyz, -pl.w);
        } else {
            int b = (r - planes) / 3;
            highp vec4 ax = materialParams.blocks[4 * b + 1 + (r - planes) - b * 3];
            highp vec3 n = dot(ax.xyz, L) > 0.0 ? -ax.xyz : ax.xyz;
            N = vec4(n, dot(n, materialParams.blocks[4 * b].xyz) + ax.w);
            owner = float(b);
        }
    }
    highp float heading = -dot(N.xyz, L);
    if (!(heading > 0.02)) {
        ok = -1000.0;
        return N.xyz * N.w;
    }
    t = (dot(N.xyz, start) - N.w) / heading;
    highp vec3 X = start + L * t;
    if (isLight) rgb *= heading / max(-L.y, 1e-4);
    if (!chooses && !recvNearSurface(X, owner, owner < -0.5 ? 0.6 : 0.4)) {
        ok = -1000.0;
        return N.xyz * N.w;
    }
    // does this corner's light pass any block closely? (within the ball round the block and as much again as a
    // triangle is long: most do not, and their pixels are spared the asking)
    int count = int(materialParams.blockCount + 0.5);
    for (int i = 0; i < 4; i++) {
        if (i >= count) break;
        highp vec3 rel = materialParams.blocks[4 * i].xyz - start;
        highp float along = dot(rel, L), side2 = dot(rel, rel) - along * along;
        highp float size = length(vec3(materialParams.blocks[4 * i + 1].w, materialParams.blocks[4 * i + 2].w, materialParams.blocks[4 * i + 3].w)) + 0.4;
        if (side2 < size * size) ok = 3.0;
    }
    // Light whose corners choose for themselves is a fine mesh, drawn many layers deep: there the blocks are asked
    // once per corner (recvClear, which comes in over a few centimetres so that nothing jumps from corner to corner)
    // and the pixels are spared. Asking per pixel cost 7 to 11 ms for 36 round solids near two blocks.
    if (chooses && isLight && ok > 1.01) {
        rgb *= recvClear(start, X, L, owner);
        ok = 1.0;
    }
    return X;
}
// Light that lies on a block lies in the block's own face, and the depth buffer (in which the blocks are) would let
// it through or not by chance. Called with the finished position (Filament's world space): pulls a corner that lies
// on a block a hair towards the camera, which changes nothing but its depth.
vec4 recvNearer(vec4 worldPosition, float owner) {
    if (owner < -0.5) return worldPosition;
    return vec4(mix(worldPosition.xyz, getWorldCameraPosition(), 0.004), worldPosition.w);
}

