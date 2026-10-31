// blend_way.glsl: a thing laid on what is under it by reading it, for flat.mat and effected.mat (their `backdrop`, `blendWay`).
// The W3C's compositing ways, as After Effects and ThorVG have them, on the screen's own numbers (0 to 1): b what
// lies under, s what it lays on it.
float wayLum(vec3 c) { return dot(c, vec3(0.3, 0.59, 0.11)); }
vec3 wayClip(vec3 c) {
    float l = wayLum(c), n = min(c.r, min(c.g, c.b)), x = max(c.r, max(c.g, c.b));
    if (n < 0.0) c = l + (c - l) * l / max(l - n, 1e-5);
    if (x > 1.0) c = l + (c - l) * (1.0 - l) / max(x - l, 1e-5);
    return c;
}
vec3 waySetLum(vec3 c, float l) { return wayClip(c + (l - wayLum(c))); }
float waySat(vec3 c) { return max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b)); }
vec3 waySetSat(vec3 c, float sat) {
    float n = min(c.r, min(c.g, c.b)), x = max(c.r, max(c.g, c.b));
    return x > n ? (c - n) * sat / (x - n) : vec3(0.0);
}
vec3 waySoft(vec3 b, vec3 s) {
    vec3 d = mix(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, step(b, vec3(0.25)));
    return mix(b + (2.0 * s - 1.0) * (d - b), b - (1.0 - 2.0 * s) * b * (1.0 - b), step(s, vec3(0.5)));
}
vec3 wayHard(vec3 b, vec3 s) { return mix(1.0 - (1.0 - b) * (1.0 - (2.0 * s - 1.0)), b * 2.0 * s, step(s, vec3(0.5))); }
vec3 wayDodge(vec3 b, vec3 s) { return mix(min(vec3(1.0), b / max(1.0 - s, vec3(1e-5))), vec3(1.0), step(1.0, s)) * step(1e-5, b); }
vec3 wayBurn(vec3 b, vec3 s) { return mix(1.0 - min(vec3(1.0), (1.0 - b) / max(s, vec3(1e-5))), vec3(0.0), step(s, vec3(0.0))) * step(b, vec3(0.99999)) + step(vec3(0.99999), b); }
vec3 way(int k, vec3 b, vec3 s) {
    if (k == 1) return min(b, s);
    if (k == 2) return max(b, s);
    if (k == 3) return wayHard(s, b);   // (overlay: hard light with the two the other way round)
    if (k == 4) return waySoft(b, s);
    if (k == 5) return wayHard(b, s);
    if (k == 6) return wayDodge(b, s);   // (color dodge)
    if (k == 7) return wayBurn(b, s);   // (color burn)
    if (k == 8) return abs(b - s);
    if (k == 9) return b + s - 2.0 * b * s;
    if (k == 10) return waySetLum(waySetSat(s, waySat(b)), wayLum(b));   // (hue)
    if (k == 11) return waySetLum(waySetSat(b, waySat(s)), wayLum(b));   // (saturation)
    if (k == 12) return waySetLum(s, wayLum(b));   // (color)
    if (k == 13) return waySetLum(b, wayLum(s));   // (luminosity)
    if (k == 14) return b + s - 1.0;   // (linear burn)
    if (k == 15) return b + 2.0 * s - 1.0;   // (linear light)
    if (k == 16) return mix(wayDodge(b, 2.0 * s - 1.0), wayBurn(b, 2.0 * s), step(s, vec3(0.5)));   // (vivid light: burn below the middle, dodge above)
    if (k == 17) return mix(max(b, 2.0 * s - 1.0), min(b, 2.0 * s), step(s, vec3(0.5)));   // (pin light)
    if (k == 18) return wayLum(s) < wayLum(b) ? s : b;   // (darker color: the whole colour)
    if (k == 19) return wayLum(s) > wayLum(b) ? s : b;   // (lighter color)
    if (k == 20) return b - s;   // (subtract)
    if (k == 21) return mix(b / max(s, vec3(1e-5)), vec3(1.0), step(s, vec3(0.0)));   // (divide)
    return s;
}
vec3 toLight(vec3 c) { return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), c)); }
vec3 toScreen(vec3 c) { return mix(c * 12.92, 1.055 * pow(max(c, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), c)); }

// Laid on what is under by reading it: the way's colour where it is, as much of it as it is there. Where nothing is under,
// as itself: the W3C's (1 - ab) Cs + ab B.
vec3 wayLaid(vec4 c) {
    vec4 below = textureLod(materialParams_backdrop, uvToRenderTargetUV(getNormalizedViewportCoord().xy), 0.0);
    vec3 under = toScreen(below.a > 1e-4 ? below.rgb / below.a : vec3(0.0));
    vec3 over = toScreen(c.rgb / c.a);
    vec3 mixed = mix(over, clamp(way(int(materialParams.blendWay + 0.5), under, over), 0.0, 1.0), clamp(below.a, 0.0, 1.0));
    return toLight(mixed) * c.a;
}
