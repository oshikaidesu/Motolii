/*{
  "ID": "motolii.checker",
  "LABEL": "Checker",
  "STAGE": "surface",
  "DESCRIPTION": "A checkerboard across the surface's picture: the same cells on an image, a shape and a solid",
  "INPUTS": [
    { "NAME": "cells", "LABEL": "Cells", "TYPE": "float", "DEFAULT": 8.0, "MIN": 1.0, "MAX": 64.0 },
    { "NAME": "dark", "LABEL": "Dark", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f {
    let cell = floor(in.uv * max(p.cells, 1.0));
    let odd = (i32(cell.x) + i32(cell.y)) % 2 != 0;
    return in.albedo * select(1.0, p.dark, odd);
}
