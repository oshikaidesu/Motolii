/*{
  "ID": "motolii.concentrick",
  "LABEL": "Concentrick",
  "STAGE": "block",
  "DESCRIPTION": "Cavalry's Concentrick, packaged: rings at one point, Stagger steps their radius, an Oscillator breathes each ring a little later than the last — interference bands run across the stack. Put the rings at one point; the index does the rest",
  "INPUTS": [
    { "NAME": "step", "LABEL": "Step", "TYPE": "float", "DEFAULT": 0.12, "MIN": 0.0, "MAX": 10.0 },
    { "NAME": "breath", "LABEL": "Breath", "TYPE": "float", "DEFAULT": 0.08, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "frequency", "LABEL": "Frequency", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 60.0 },
    { "NAME": "stagger", "LABEL": "Stagger", "TYPE": "float", "DEFAULT": 0.08, "MIN": -10.0, "MAX": 10.0 },
    { "NAME": "fade", "LABEL": "Fade", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/
import package::cavalry::{ cv_oscillator, cv_stagger };

// Duplicator(Point) + Stagger → Radius + Oscillator(Stagger 付)。関数の棚(_cavalry.wgsl)を呼ぶだけ。
fn block(k: u32, p: BlockParams) -> Offset {
    let n = max(host.members, 1u);
    let delay = cv_stagger(k, n, p.stagger, 0u);
    // 先に置いた物(番号の小さい物)が大きい: 描き順が下の物ほど外の環になり、塗った円でも環に見える。
    let radius = 1.0 + f32(n - 1u - k) * p.step;
    let breath = 1.0 + p.breath * cv_oscillator(host.time - delay, p.frequency, 0.0, 0u);
    // 外の環ほど薄く(Fade)。
    let alpha = 1.0 - p.fade * f32(n - 1u - k) / f32(n);
    return Offset(vec2f(0.0), 0.0, radius * breath, vec4f(1.0, 1.0, 1.0, alpha));
}
