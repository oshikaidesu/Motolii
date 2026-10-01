import package::cavalry::{ cv_oscillator, cv_stagger };

@description("Cavalry's Concentrick, packaged: rings at one point, Stagger steps their radius, an Oscillator breathes each ring a little later than the last — interference bands run across the stack. Put the rings at one point; the index does the rest")

@label("Step") @range(0.0, 10.0)
override step: f32 = 0.12;
@label("Breath") @range(0.0, 2.0)
override breath: f32 = 0.08;
@label("Frequency") @range(0.0, 60.0)
override frequency: f32 = 0.5;
@label("Stagger") @range(-10.0, 10.0)
override stagger: f32 = 0.08;
@label("Fade") @range(0.0, 1.0)
override fade: f32 = 0.5;

// Duplicator(Point) + Stagger → Radius + Oscillator(Stagger 付)。関数の棚(_cavalry.wgsl)を呼ぶだけ。
fn block(k: u32) -> Offset {
    let n = max(host.members, 1u);
    let delay = cv_stagger(k, n, stagger, 0u);
    // 先に置いた物(番号の小さい物)が大きい: 描き順が下の物ほど外の環になり、塗った円でも環に見える。
    let radius = 1.0 + f32(n - 1u - k) * step;
    let breath = 1.0 + breath * cv_oscillator(host.time - delay, frequency, 0.0, 0u);
    // 外の環ほど薄く(Fade)。
    let alpha = 1.0 - fade * f32(n - 1u - k) / f32(n);
    return Offset(vec2f(0.0), 0.0, radius * breath, vec4f(1.0, 1.0, 1.0, alpha));
}
