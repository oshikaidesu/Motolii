/*{
  "ID": "motolii.wave_block",
  "LABEL": "Wave",
  "STAGE": "block",
  "DESCRIPTION": "Moves each thing up and down on a sine wave that travels along them in order (Cavalry's Wave behaviour; AE's wiggly text selector made regular). The same wave can also breathe the size and the opacity: Scale and Opacity are how far each swings (0 = untouched)",
  "INPUTS": [
    { "NAME": "amplitude", "LABEL": "Amplitude", "TYPE": "float", "DEFAULT": 12.0, "MIN": -10000.0, "MAX": 10000.0 },
    { "NAME": "frequency", "LABEL": "Frequency", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 100.0 },
    { "NAME": "wavelength", "LABEL": "Wavelength", "TYPE": "float", "DEFAULT": 8.0, "MIN": 1.0, "MAX": 10000.0 },
    { "NAME": "scale", "LABEL": "Scale", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "opacity", "LABEL": "Opacity", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

// 物の番号(効果の列が掛かった順)を波の上の位置に: Wavelength 個で 1 周。
// 同じ 1 本の波が位置(足す)・大きさ(掛ける)・不透明(掛ける)に掛かる。山で大きく明るく、谷で小さく薄く。
fn block(k: u32, p: BlockParams) -> Offset {
    let phase = 6.2831853 * (p.frequency * host.time - f32(k) / p.wavelength);
    let s = sin(phase);
    let size = 1.0 + p.scale * s;
    let alpha = 1.0 - p.opacity * (0.5 - 0.5 * s);
    return Offset(vec2f(0.0, p.amplitude * s), 0.0, size, vec4f(1.0, 1.0, 1.0, alpha));
}
