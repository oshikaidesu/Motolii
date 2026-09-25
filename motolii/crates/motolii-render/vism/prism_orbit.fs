/*{
  "ID": "motolii.prism_orbit",
  "LABEL": "Prism Orbit",
  "STAGE": "pass",
  "DESCRIPTION": "Paints the layer as a dark stage lit by drifting spectral light leaks, with a thin glowing orbit (a tilted ellipse) whose light runs round it. Draw the back half behind and the front half over the subject with Part. Phase 0→1 is one full cycle, so keying it linearly over a loop range repeats without a seam",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "part", "LABEL": "Part", "TYPE": "long", "DEFAULT": 0, "LABELS": ["Stage and Orbit", "Orbit Back", "Orbit Front"] },
    { "NAME": "center", "LABEL": "Center", "TYPE": "point2D", "DEFAULT": [50.0, 50.0] },
    { "NAME": "radius_x", "LABEL": "Radius X", "TYPE": "float", "DEFAULT": 600.0, "MIN": 1.0, "MAX": 10000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "radius_y", "LABEL": "Radius Y", "TYPE": "float", "DEFAULT": 160.0, "MIN": 1.0, "MAX": 10000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": -14.0, "MIN": -360.0, "MAX": 360.0 },
    { "NAME": "width", "LABEL": "Line Width", "TYPE": "float", "DEFAULT": 1.4, "MIN": 0.1, "MAX": 100.0 },
    { "NAME": "glow", "LABEL": "Glow", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 10.0, "HERO": true },
    { "NAME": "leaks", "LABEL": "Light Leaks", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "phase", "LABEL": "Phase", "TYPE": "float", "DEFAULT": 0.0, "SUBTYPE": "TIME" }
  ]
}*/

const float TAU = 6.28318530718;

// A smooth spectrum: hue 0..1 → linear RGB.
vec3 spectrum(float h) {
    vec3 c = clamp(abs(fract(h + vec3(0.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0) - 1.0, 0.0, 1.0);
    return c * c;
}

// Distance (px, approximately) from p to the ellipse, and the angle round it.
vec2 ellipse_distance(vec2 p) {
    float a = radians(angle);
    // Center is in percent of the layer, measured from its top left.
    vec2 q = mat2(cos(a), sin(a), -sin(a), cos(a)) * (p - center / 100.0 * RENDERSIZE.xy);
    vec2 r = vec2(radius_x, radius_y);
    float k = length(q / r);
    // First-order distance to the level set k = 1: (k - 1) / |grad k|.
    float d = (k - 1.0) * k / max(length(q / (r * r)), 1e-6);
    return vec2(d, atan(q.y / r.y, q.x / r.x));
}

void main() {
    vec4 base = IMG_THIS_PIXEL(inputImage);
    if (base.a <= 1e-5) { gl_FragColor = vec4(0.0); return; }
    float t = TAU * phase;
    vec2 uv = vec2(isf_FragNormCoord.x, 1.0 - isf_FragNormCoord.y);
    vec2 p = uv * RENDERSIZE.xy;

    // The orbit: a hairline, a soft halo, a spectral sheen that turns with the angle, and a
    // bright run of light that goes once round per cycle.
    vec2 e = ellipse_distance(p);
    float line = exp(-pow(e.x / width, 2.0));
    float halo = exp(-abs(e.x) / (3.0 + 9.0 * glow)) * glow * 0.45;
    float run = pow(0.5 + 0.5 * cos(e.y - t), 24.0);
    vec3 sheen = mix(vec3(0.85, 0.9, 1.0), spectrum(e.y / TAU + phase), 0.45);
    vec3 orbit = sheen * (line * (0.55 + 1.6 * run) + halo * (0.4 + run));
    // Back half: the part of the ellipse above its centre line (sin < 0 in screen space).
    float front = smoothstep(-0.05, 0.05, sin(e.y));
    float mask = part < 0.5 ? 1.0 : (part < 1.5 ? 1.0 - front : front);
    orbit *= mask;

    if (part > 0.5) {
        float a = clamp(max(orbit.r, max(orbit.g, orbit.b)), 0.0, 1.0);
        gl_FragColor = vec4(orbit, a) * base.a;
        return;
    }

    // The stage: near black, two spectral leaks drifting on closed paths, a vignette.
    vec2 l1 = vec2(0.72, 0.30) + 0.08 * vec2(cos(t), sin(t));
    vec2 l2 = vec2(0.22, 0.78) + 0.07 * vec2(sin(t + 1.1), cos(t + 0.4));
    float s1 = exp(-pow(distance(uv * vec2(1.78, 1.0), l1 * vec2(1.78, 1.0)) / 0.35, 2.0));
    float s2 = exp(-pow(distance(uv * vec2(1.78, 1.0), l2 * vec2(1.78, 1.0)) / 0.30, 2.0));
    vec3 leak = spectrum(uv.x * 0.6 + uv.y * 0.3 + phase) * s1 + spectrum(0.55 - uv.y * 0.5 + phase) * s2;
    float vignette = smoothstep(1.25, 0.25, distance(uv, vec2(0.5)));
    // A little grain keeps the dark gradients from banding.
    float grain = fract(sin(dot(p + phase * 97.0, vec2(12.9898, 78.233))) * 43758.5453) - 0.5;
    vec3 color = vec3(0.004, 0.004, 0.008) + leak * leaks * 0.08 * vignette + orbit + grain * 0.004;
    gl_FragColor = vec4(color * base.a, base.a);
}
