/*{
  "ID": "motolii.glass",
  "LABEL": "Glass",
  "STAGE": "surface",
  "BACKDROP_INPUT": "transmission",
  "DESCRIPTION": "Environment-lit dielectric or metal: reflection by roughness, refracted see-through by transmission",
  "INPUTS": [
    { "NAME": "ior", "LABEL": "Refraction", "TYPE": "float", "DEFAULT": 1.5, "MIN": 1.0, "MAX": 3.0 },
    { "NAME": "roughness", "LABEL": "Roughness", "TYPE": "float", "DEFAULT": 0.05, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "transmission", "LABEL": "Transmission", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "metallic", "LABEL": "Metallic", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

// The split-sum shading itself is a borrowed ruler that lives in the fork (utils/lighting.wgsl);
// this sheet only maps its knobs.
fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f {
    return shade_surface(in.albedo, in.normal, in.view_dir, in.world_position, in.thickness, vec4f(p.roughness, p.metallic, p.transmission, p.ior));
}
