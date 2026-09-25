/*{
  "ID": "motolii.glass",
  "LABEL": "Glass",
  "STAGE": "surface",
  "BACKDROP_INPUT": "transmission",
  "BACKDROP_BLUR": "roughness",
  "DESCRIPTION": "Environment-lit dielectric or metal: reflection by roughness, refracted see-through by transmission, colors split by dispersion (20 / Abbe number)",
  "INPUTS": [
    { "NAME": "ior", "LABEL": "Refraction", "TYPE": "float", "DEFAULT": 1.5, "MIN": 1.0, "MAX": 3.0 },
    { "NAME": "roughness", "LABEL": "Roughness", "TYPE": "float", "DEFAULT": 0.05, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "FACTOR" },
    { "NAME": "transmission", "LABEL": "Transmission", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "metallic", "LABEL": "Metallic", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "dispersion", "LABEL": "Dispersion", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "clearcoat", "LABEL": "Clearcoat", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "FACTOR" },
    { "NAME": "emission", "LABEL": "Emission", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 8.0, "SUBTYPE": "FACTOR" }
  ]
}*/

// The split-sum shading itself is Motolii's standard material (vism/material.wgsl, a module on the shelf);
// this sheet only maps its knobs.
fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f {
    // surface_extras hands the host's solid frame and the extra knobs to the material (it adds 0).
    return shade_surface(in.albedo, in.normal, in.view_dir, in.world_position, in.thickness + surface_extras(in.params[3], in.params[4], in.params[5], p.clearcoat, p.emission), vec4f(p.roughness, p.metallic, p.transmission, p.ior), p.dispersion);
}
