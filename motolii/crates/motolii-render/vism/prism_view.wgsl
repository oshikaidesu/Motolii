/*{
  "ID": "shelf.prism_view",
  "LABEL": "Prism View",
  "STAGE": "surface",
  "BACKDROP_INPUT": "transmission",
  "BACKDROP_BLUR": "roughness",
  "DESCRIPTION": "Glass that shows the world from its layer's centre: three Views, one per colour (a many-eyed dispersion), folded like a kaleidoscope and mixed with what the glass refracts",
  "VIEWS": [
    { "FROM": "layer", "LOOK": [-0.06, -0.02, 1], "UP": [0, -1, 0], "FOV": 96 },
    { "FROM": "layer", "LOOK": [0, 0, 1], "UP": [0, -1, 0], "FOV": 88 },
    { "FROM": "layer", "LOOK": [0.06, 0.02, 1], "UP": [0, -1, 0], "FOV": 80 }
  ],
  "VIEW_SIZE": 512,
  "INPUTS": [
    { "NAME": "phase", "LABEL": "Phase", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "view_mix", "LABEL": "View", "TYPE": "float", "DEFAULT": 0.85, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "eyes", "LABEL": "Eyes", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "fold", "LABEL": "Fold", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 12.0 },
    { "NAME": "zoom", "LABEL": "Zoom", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 4.0 },
    { "NAME": "bend", "LABEL": "Bend", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "scatter", "LABEL": "Scatter", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "lens", "LABEL": "Lens", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "clear", "LABEL": "Clear", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "hue", "LABEL": "Hue", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "rim", "LABEL": "Rim", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "ior", "LABEL": "Refraction", "TYPE": "float", "DEFAULT": 1.45, "MIN": 1.0, "MAX": 3.0 },
    { "NAME": "roughness", "LABEL": "Roughness", "TYPE": "float", "DEFAULT": 0.06, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "transmission", "LABEL": "Transmission", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "dispersion", "LABEL": "Dispersion", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.0, "MAX": 2.0 }
  ]
}*/

fn prism_hash(p: vec3f) -> vec3f {
    var q = fract(p * vec3f(0.1031, 0.1030, 0.0973));
    q += dot(q, q.yxz + 33.33);
    return fract((q.xxy + q.yxx) * q.zyx);
}

fn prism_turn(v: vec2f, a: f32) -> vec2f {
    let c = cos(a);
    let s = sin(a);
    return vec2f(c * v.x - s * v.y, s * v.x + c * v.y);
}

// Past the View's edge the picture is mirrored back, like the walls of a kaleidoscope,
// instead of stretching the View's last row.
fn prism_mirror(uv: vec2f) -> vec2f {
    return 1.0 - abs(1.0 - 2.0 * fract(uv * 0.5));
}

// A kaleidoscope fold: `n` mirrored wedges round the centre; 0 leaves the picture as it is.
fn prism_fold(v: vec2f, n: f32) -> vec2f {
    if n < 0.5 { return v; }
    let wedge = 3.14159265 / n;
    let r = length(v);
    var a = atan2(v.y, v.x);
    a = abs((a - wedge * floor(a / (2.0 * wedge)) * 2.0) - wedge);
    return r * vec2f(cos(a), sin(a));
}

fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f {
    // How much world one unit of the picture covers: a copy's size, which turning never changes.
    let world_area = length(cross(dpdx(in.world_position), dpdy(in.world_position)));
    let du = dpdx(in.uv);
    let dv = dpdy(in.uv);
    let picture_area = max(abs(du.x * dv.y - du.y * dv.x), 1e-12);
    let size = log2(max(world_area / picture_area, 1e-6));

    let glass = shade_surface(in.albedo, in.normal, in.view_dir, in.world_position, in.thickness, vec4f(p.roughness, 0.0, p.transmission, p.ior), p.dispersion);
    let n = view_count();
    if n == 0u || sun_color().w > 0.0 { return glass; }

    // Repeater copies differ in size (Scale Random), so the size names the copy; Scatter 0 gives
    // every copy the same name.
    let id = mix(vec3f(0.5, 0.0, 0.0), prism_hash(vec3f(round(size * 24.0), 17.0, 5.0)), p.scatter);
    let spin = sign(id.x - 0.5);
    let t = p.phase * 6.2831853;

    var v = in.uv - 0.5;
    v = prism_turn(v, id.y * 6.2831853 + spin * t);
    v = prism_fold(v, floor(p.fold + id.z * 2.0));
    v = v / max(p.zoom * (0.7 + 0.6 * id.x), 0.05);
    // The glass bends the picture the way it faces.
    let lean = (in.normal.xy - in.view_dir.xy) * p.bend;
    let drift = 0.08 * vec2f(sin(t + id.z * 6.28), cos(t * 2.0 + id.y * 6.28));
    // Lens 1: read the Views along the ray the glass refracts, not across the picture (a crystal ball).
    let ray = refract(-in.view_dir, in.normal, 1.0 / max(p.ior, 1.0));
    let lens_uv = 0.5 + prism_turn(ray.xy, spin * t) * (0.5 / max(p.zoom, 0.05));
    let uv = prism_mirror(mix(v + 0.5 + lean + drift, lens_uv, p.lens));

    let lod = p.roughness * 5.0;
    let split = 0.012 * p.dispersion * (0.5 + id.x);
    let one = view_sample(min(1u, n - 1u), uv, lod);
    let r1 = view_sample(min(1u, n - 1u), uv + vec2f(split, 0.0), lod);
    let b1 = view_sample(min(1u, n - 1u), uv - vec2f(split, 0.0), lod);
    let r3 = view_sample(0u, uv, lod);
    let b3 = view_sample(min(2u, n - 1u), uv, lod);
    // Eyes = 0: one View split by a lens; 1: red, green and blue each seen by its own View.
    let red = mix(r1, r3, p.eyes);
    let blue = mix(b1, b3, p.eyes);
    let seen = vec4f(red.r, one.g, blue.b, max(max(red.a, one.a), blue.a));

    let n_dot_v = clamp(dot(in.normal, in.view_dir), 0.0, 1.0);
    let facing = mix(0.55, 1.0, n_dot_v);
    // Each copy colours its window its own way; `clear` of them are plain glass.
    let tint = mix(vec3f(1.0), 0.6 + 0.6 * cos(6.2831853 * (id.z + vec3f(0.0, 0.33, 0.67))), p.hue);
    let window = seen.rgb * tint * facing + (1.0 - seen.a) * glass;
    let share = p.view_mix * step(p.clear, fract(id.y * 7.31 + id.z));
    var out = mix(glass, window, share);

    // A thin-film rim: the edge of every pane runs through the spectrum as it turns.
    let edge = min(min(in.uv.x, in.uv.y), min(1.0 - in.uv.x, 1.0 - in.uv.y));
    let film = 0.5 + 0.5 * cos(6.2831853 * (vec3f(0.0, 0.33, 0.67) + n_dot_v * 1.7 + id.y + p.phase));
    out += film * p.rim * smoothstep(0.06, 0.0, edge) * (1.0 - n_dot_v * 0.5);
    return out;
}
