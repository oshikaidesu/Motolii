// Is light that left the glass at E and was dropped on a plane at X really there, and does the camera see it?
// (See "Receivers" in beam_glass.h, and recv_plane.glsl for the parameters; everything in the user's world space.)
// Asked per pixel by the materials that draw light (ball_light.mat, cone_light.mat, recv_light.mat):
//   recvOn      - is X on the part of its plane that is surface? For the room (owner -1): inside the room. For a
//                 face of block `owner`: within the block.
//   recvBlocked - a block stands between E and X (the light went into that block instead), or a block stands
//                 between the sun and E (the glass was in that block's shadow; E stands in for the place the
//                 sunlight entered the glass, which is a glass's width away at most). recvSun: the way the sunlight travels.
//   recvSeen    - no block hides X from the camera. (The light is drawn into a picture taken by the camera, with no
//                 depth; a wall or the floor cannot hide anything in a convex room, a block can.)
// stage_through / stage_blocked / stage_on of beam_glass.h are the first two on the CPU, for the desk check.

// Does the line from a along ab enter block i before `far` times ab?
bool recvThrough(int i, highp vec3 a, highp vec3 ab, highp float far) {
    highp vec3 c = materialParams.blocks[4 * i].xyz;
    highp float t0 = 0.0, t1 = far;
    for (int k = 1; k <= 3; k++) {
        highp vec4 ax = materialParams.blocks[4 * i + k];
        highp float o = dot(ax.xyz, a - c), d = dot(ax.xyz, ab);
        if (abs(d) < 1e-7) {
            if (abs(o) > ax.w) return false;
            continue;
        }
        highp float ta = (-ax.w - o) / d, tb = (ax.w - o) / d;
        t0 = max(t0, min(ta, tb));
        t1 = min(t1, max(ta, tb));
    }
    return t0 < t1;
}
bool recvBlocked(highp vec3 X, highp vec3 E, highp vec3 recvSun) {
    int count = int(materialParams.blockCount + 0.5);
    for (int i = 0; i < 4; i++) {
        if (i >= count) break;
        // (0.9999 of the way: light that lands ON a block has not gone through it)
        if (recvThrough(i, E, X - E, 0.9999)) return true;
        if (recvThrough(i, E, -recvSun, 1e9)) return true;
    }
    return false;
}
bool recvOn(highp vec3 X, float owner) {
    int own = int(floor(owner + 0.5));
    if (own < 0) {
        for (int j = 0; j < 6; j++) {
            if (float(j) >= materialParams.roomCount) break;
            highp vec4 pl = materialParams.room[j];
            if (dot(pl.xyz, X) + pl.w < -1e-4) return false;
        }
        return true;
    }
    highp vec3 c = materialParams.blocks[4 * own].xyz;
    for (int k = 1; k <= 3; k++) {
        highp vec4 ax = materialParams.blocks[4 * own + k];
        if (abs(dot(ax.xyz, X - c)) > ax.w + 1e-4) return false;
    }
    return true;
}
bool recvSeen(highp vec3 X, float owner, highp vec3 eye) {
    int own = int(floor(owner + 0.5)), count = int(materialParams.blockCount + 0.5);
    highp vec3 way = X - eye;
    highp float far = length(way);
    for (int i = 0; i < 4; i++) {
        if (i >= count) break;
        // its own block hides what lies on its far sides; 2 cm are let through, for a triangle that lies across an
        // edge of the block like a chord and so a little inside it
        if (recvThrough(i, eye, way, i == own ? 1.0 - 0.02 / far : 0.9999)) return false;
    }
    return true;
}
// For light whose corners chose their own plane (recvChoose) and have already asked recvClear (recv_plane.glsl).
// `light` = rgb and how far it flew, `ownerTwice` = owner and owner squared as they arrive at the pixel.
vec4 recvKeep(vec4 light, vec2 ownerTwice) {
    if (light.a < 0.0) return vec4(0.0);   // behind where it left the glass
    // On the room there is nothing more to ask: every corner chose the plane it meets first, and what a block hides
    // from the camera is left out by the depth buffer (the blocks are drawn into it, see glass_scene.mm).
    if (ownerTwice.x < -0.5) return light;
    // a triangle whose corners chose different blocks has no place to be
    if (abs(ownerTwice.y - ownerTwice.x * ownerTwice.x) > 1e-3) return vec4(0.0);
    highp vec3 X = getUserWorldPosition(), eye = getWorldCameraPosition() + (X - getWorldPosition());
    if (!recvOn(X, ownerTwice.x) || !recvSeen(X, ownerTwice.x, eye)) return vec4(0.0);
    return light;
}
