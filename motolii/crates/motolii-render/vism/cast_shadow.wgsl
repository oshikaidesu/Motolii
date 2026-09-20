/*{
  "ID": "motolii.cast_shadow",
  "LABEL": "Cast Shadow",
  "STAGE": "shadow",
  "DESCRIPTION": "This layer takes light away. It is drawn into the sun's stencil, and every layer that has a surface catches the shadow — its colour, if this layer is glass. There is no lamp to place: the light comes from the environment",
  "INPUTS": [
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "FACTOR", "HERO": true }
  ]
}*/

// shader は無い。型紙を描くのは compositor の 1 箇所(surface_scene.rs capture_light_cookie)で、
// この file は棚の札と欄の宣言(clip.wgsl と同じ形)。
