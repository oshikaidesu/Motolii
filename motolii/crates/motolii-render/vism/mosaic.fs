/*{
  "ID": "motolii.mosaic",
  "LABEL": "Mosaic",
  "STAGE": "pass",
  "DESCRIPTION": "Cuts the layer into a grid of solid blocks, alpha included, so the silhouette turns into square pixels (After Effects' Mosaic). Put it after a wipe and the wipe's edge becomes a staircase",
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "columns", "LABEL": "Horizontal Blocks", "TYPE": "float", "DEFAULT": 10.0, "MIN": 1.0, "MAX": 4000.0, "HERO": true },
    { "NAME": "rows", "LABEL": "Vertical Blocks", "TYPE": "float", "DEFAULT": 10.0, "MIN": 1.0, "MAX": 4000.0 },
    { "NAME": "sharp", "LABEL": "Sharp Colors", "TYPE": "bool", "DEFAULT": false }
  ]
}*/

// Sharp Colors は各ブロックの中心の色、切ればブロックの中の 4×4 の平均。
void main() {
    vec2 cells = vec2(max(floor(columns), 1.0), max(floor(rows), 1.0));
    vec2 cell = floor(isf_FragNormCoord * cells);
    vec4 centre = IMG_NORM_PIXEL(inputImage, (cell + 0.5) / cells);
    vec4 sum = vec4(0.0);
    for (int j = 0; j < 4; j++) {
        for (int i = 0; i < 4; i++) {
            sum += IMG_NORM_PIXEL(inputImage, (cell + (vec2(float(i), float(j)) + 0.5) / 4.0) / cells);
        }
    }
    gl_FragColor = sharp > 0.5 ? centre : sum / 16.0;
}
