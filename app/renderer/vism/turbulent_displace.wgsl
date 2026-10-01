/*{
  "ID": "motolii.turbulent_displace",
  "LABEL": "Turbulent Displace",
  "STAGE": "field",
  "DESCRIPTION": "Fractal simplex noise moves the points of any material — board (grid), mesh, point cloud. Direction XYZ by default; XY is After Effects' Turbulent Displace (Z masked); Normal bends the surface",
  "INPUTS": [
    { "NAME": "amount", "LABEL": "Amount", "TYPE": "float", "DEFAULT": 50.0, "MIN": 0.0, "MAX": 100000.0 },
    { "NAME": "size", "LABEL": "Size", "TYPE": "float", "DEFAULT": 100.0, "MIN": 1.0, "MAX": 100000.0 },
    { "NAME": "complexity", "LABEL": "Complexity", "TYPE": "float", "DEFAULT": 3.0, "MIN": 1.0, "MAX": 8.0 },
    { "NAME": "evolution", "LABEL": "Evolution", "TYPE": "float", "DEFAULT": 0.0, "SUBTYPE": "TIME" },
    { "NAME": "along", "LABEL": "Direction", "TYPE": "long", "DEFAULT": 1, "LABELS": ["Normal", "XYZ", "XY", "X", "Y", "Z"] },
    { "NAME": "offset_x", "LABEL": "Offset X", "TYPE": "float", "DEFAULT": 0.0 },
    { "NAME": "offset_y", "LABEL": "Offset Y", "TYPE": "float", "DEFAULT": 0.0 },
    { "NAME": "offset_z", "LABEL": "Offset Z", "TYPE": "float", "DEFAULT": 0.0 },
    { "NAME": "seed", "LABEL": "Seed", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 10000.0 }
  ]
}*/

// Same coordinate as the point-cloud twin in compositor/point_cloud.rs: shifted, scaled to feature size, moved by evolution.
fn turbulent_coordinate(frame_position: vec3f, p: FieldParams) -> vec3f {
    let size = max(p.size, 1e-3);
    return (frame_position + vec3f(p.offset_x, p.offset_y, p.offset_z)) / size + p.evolution * vec3f(0.53, 0.71, 0.89) + p.seed * vec3f(0.137, 0.173, 0.193);
}

fn turbulent_vector(q: vec3f, octaves: u32) -> vec3f {
    return vec3f(fbm3(q, octaves), fbm3(q + vec3f(31.7, 0.0, 0.0), octaves), fbm3(q + vec3f(0.0, 47.3, 0.0), octaves));
}

// Direction の軸の伏せ(Blender Displace の Direction と同じ並び + Z を伏せた XY)。
// XY = AE の Turbulent Displace: 面内だけ動く。既定の XYZ はそれに浮き沈みが足された物。
fn axis_mask(along: f32) -> vec3f {
    if along < 1.5 { return vec3f(1.0, 1.0, 1.0); }
    if along < 2.5 { return vec3f(1.0, 1.0, 0.0); }
    if along < 3.5 { return vec3f(1.0, 0.0, 0.0); }
    if along < 4.5 { return vec3f(0.0, 1.0, 0.0); }
    return vec3f(0.0, 0.0, 1.0);
}

fn field(in: FieldIn, p: FieldParams) -> FieldOut {
    if p.amount == 0.0 {
        return FieldOut(vec3f(0.0), in.normal);
    }
    let octaves = u32(clamp(p.complexity, 1.0, 8.0));
    let q = turbulent_coordinate(in.frame_position, p);
    if p.along < 0.5 && any(in.normal != vec3f(0.0)) {
        let n = normalize(in.normal);
        let h = fbm3(q, octaves);
        // Bend the normal by the field's gradient (bump mapping): n' = n - amount * (∇h - n(n·∇h)) / size.
        let e = 0.01;
        let grad = vec3f(
            fbm3(q + vec3f(e, 0.0, 0.0), octaves) - fbm3(q - vec3f(e, 0.0, 0.0), octaves),
            fbm3(q + vec3f(0.0, e, 0.0), octaves) - fbm3(q - vec3f(0.0, e, 0.0), octaves),
            fbm3(q + vec3f(0.0, 0.0, e), octaves) - fbm3(q - vec3f(0.0, 0.0, e), octaves),
        ) / (2.0 * e * max(p.size, 1e-3));
        let tangent_grad = grad - n * dot(n, grad);
        return FieldOut(n * p.amount * h, normalize(n - p.amount * tangent_grad));
    }
    return FieldOut(p.amount * turbulent_vector(q, octaves) * axis_mask(p.along), in.normal);
}
