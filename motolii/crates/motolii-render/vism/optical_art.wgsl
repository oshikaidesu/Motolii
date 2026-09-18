/*{
  "ID": "motolii.optical_art",
  "LABEL": "Optical Art",
  "STAGE": "block",
  "DESCRIPTION": "Cavalry's Optical Art, packaged: a stack of bars; every other bar swings the opposite way (Modulate), each a little later than the last (Stagger), on one Oscillator — the stripes ripple. Put the bars in a column or a row; the index alternates them",
  "INPUTS": [
    { "NAME": "swing", "LABEL": "Swing", "TYPE": "float", "DEFAULT": 60.0, "MIN": -10000.0, "MAX": 10000.0 },
    { "NAME": "frequency", "LABEL": "Frequency", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 60.0 },
    { "NAME": "stagger", "LABEL": "Stagger", "TYPE": "float", "DEFAULT": 0.05, "MIN": -10.0, "MAX": 10.0 },
    { "NAME": "divisor", "LABEL": "Divisor", "TYPE": "float", "DEFAULT": 2.0, "MIN": 1.0, "MAX": 64.0 },
    { "NAME": "waveform", "LABEL": "Waveform", "TYPE": "long", "DEFAULT": 0, "LABELS": ["Sine", "Triangle", "Square", "Sawtooth"] }
  ]
}*/
import package::cavalry::{ cv_oscillator, cv_stagger };

// Duplicator + Stagger + Modulate(Remainder: pass → +1, fail → −1) + Oscillator。
fn block(k: u32, p: BlockParams) -> Offset {
    let n = max(host.members, 1u);
    let delay = cv_stagger(k, n, p.stagger, 0u);
    let sign = select(-1.0, 1.0, (k % u32(max(p.divisor, 1.0))) == 0u);
    let x = p.swing * sign * cv_oscillator(host.time - delay, p.frequency, 0.0, u32(p.waveform));
    return Offset(vec2f(x, 0.0), 0.0, 1.0, vec4f(1.0));
}
