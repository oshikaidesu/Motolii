@id("motolii.wave_block")
@description("Moves each thing up and down on a sine wave that travels along them in order (Cavalry's Wave behaviour; AE's wiggly text selector made regular). The same wave can also breathe the size and the opacity: Scale and Opacity are how far each swings (0 = untouched)")

@label("Amplitude") @range(-10000.0, 10000.0)
override amplitude: f32 = 12.0;
@label("Frequency") @range(0.0, 100.0)
override frequency: f32 = 1.0;
@label("Wavelength") @range(1.0, 10000.0)
override wavelength: f32 = 8.0;
@label("Scale") @range(0.0, 1.0)
override scale: f32 = 0.0;
@label("Opacity") @range(0.0, 1.0)
override opacity: f32 = 0.0;

// 物の番号(効果の列が掛かった順)を波の上の位置に: Wavelength 個で 1 周。
// 同じ 1 本の波が位置(足す)・大きさ(掛ける)・不透明(掛ける)に掛かる。山で大きく明るく、谷で小さく薄く。
fn block(k: u32) -> Offset {
    let phase = 6.2831853 * (frequency * host.time - f32(k) / wavelength);
    let s = sin(phase);
    let size = 1.0 + scale * s;
    let alpha = 1.0 - opacity * (0.5 - 0.5 * s);
    return Offset(vec2f(0.0, amplitude * s), 0.0, size, vec4f(1.0, 1.0, 1.0, alpha));
}
