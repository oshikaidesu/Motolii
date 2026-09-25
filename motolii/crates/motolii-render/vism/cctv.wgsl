/*{
  "ID": "shelf.cctv",
  "LABEL": "CCTV",
  "STAGE": "surface",
  "DESCRIPTION": "A screen showing what a camera standing at the screen sees of the work, with scanlines",
  "VIEWS": [
    { "FROM": "layer", "LOOK": [-0.55, -0.35, 0.75], "UP": [0, -1, 0], "FOV": 60 }
  ],
  "VIEW_SIZE": 512,
  "INPUTS": [
    { "NAME": "scanlines", "LABEL": "Scanlines", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f {
    if view_count() == 0u { return in.albedo; }
    let seen = view_sample(0u, in.uv, 0.0);
    let picture = seen.rgb + (1.0 - seen.a) * vec3f(0.02, 0.03, 0.05);
    let lines = 1.0 - p.scanlines * 0.5 * (1.0 + sin(in.uv.y * 540.0));
    return picture * lines;
}
