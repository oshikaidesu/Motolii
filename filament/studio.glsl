// studio.glsl: the room glass carries with it, the same wherever the glass is put: grey walls, three bright strip
// lamps, three black flags, a soft lamp overhead. Clear glass reads as glass only by what is around it (bright fields,
// black lines), so the glass brings its surroundings and the scene has no say. Used by glassbox.mat and thick_glass.mat.
float band(float x, float centre, float halfWidth, float soft) {
    float d = abs(x - centre);
    d = min(d, 6.2831853 - d);
    return 1.0 - smoothstep(halfWidth, halfWidth + soft, d);
}
float studioRoom(vec3 d) {   // how bright the room is that way
    float el = asin(clamp(d.y, -1.0, 1.0));
    float az = atan(d.z, d.x) + 3.14159265;
    float wall = mix(0.62, 0.30, smoothstep(0.0, 1.25, el));           // pale near the horizon, grey overhead
    float v = d.y < 0.0 ? mix(wall, 0.66, smoothstep(0.0, -0.2, d.y)) : wall;
    float rise = smoothstep(0.02, 0.12, el) * (1.0 - smoothstep(1.0, 1.2, el));
    v *= 1.0 - 0.92 * rise * (band(az, 1.35, 0.16, 0.05) + band(az, 3.55, 0.14, 0.05) + band(az, 5.35, 0.16, 0.05));
    v += rise * (7.0 * band(az, 0.95, 0.10, 0.03) + 5.0 * band(az, 3.15, 0.07, 0.03) + 4.0 * band(az, 4.95, 0.10, 0.03));
    v += 2.2 * smoothstep(1.15, 1.35, el);                              // overhead softbox
    return v;
}
// The room glass carries when it stands alone: about as bright as the ground, with six bright strip lamps standing all round, each with a black
// flag at either side (how glass is photographed: dark hard against bright). How bright it is that way, against the
// ground behind the glass (pale paper: a pale room; a black poster: a black one).
float roomLamp(vec3 d) {   // 1 on a strip lamp
    return (1.0 - smoothstep(0.05, 0.07, abs(fract(atan(d.z, d.x) * 6.0 / 6.2831853) - 0.5))) * (1.0 - smoothstep(0.75, 0.9, abs(d.y)));
}
float mirroredLamp(vec3 d) {   // what the glass's faces mirror of the room: two of the lamps, soft-edged (all six, hard: a cage of white lines)
    return (1.0 - smoothstep(0.008, 0.035, abs(fract(atan(d.z, d.x) * 2.0 / 6.2831853 + 0.15) - 0.5))) * (1.0 - smoothstep(0.6, 0.9, abs(d.y)))
         + 6.0 * smoothstep(0.9975, 0.9992, dot(d, vec3(0.35, 0.86, 0.37)));   // (and one small hot lamp high up: a glint here and there)
}
float glassRoom(vec3 d) {
    float x = abs(fract(atan(d.z, d.x) * 6.0 / 6.2831853) - 0.5);    // 0 at a lamp's middle
    float flag = (1.0 - smoothstep(0.14, 0.16, x)) - (1.0 - smoothstep(0.05, 0.07, x));
    return 0.8 + 2.7 * roomLamp(d) - 0.55 * flag * (1.0 - smoothstep(0.75, 0.9, abs(d.y)));   // (0.45: the solid's sides go grey, like metal; 0.95: they melt into the ground)
}
