import package::cavalry::{ cv_oscillator, cv_stagger };

@description("Cavalry's Optical Art, packaged: a stack of bars; every other bar swings the opposite way (Modulate), each a little later than the last (Stagger), on one Oscillator — the stripes ripple. Put the bars in a column or a row; the index alternates them")

@label("Swing") @range(-10000.0, 10000.0)
override swing: f32 = 60.0;
@label("Frequency") @range(0.0, 60.0)
override frequency: f32 = 0.4;
@label("Stagger") @range(-10.0, 10.0)
override stagger: f32 = 0.05;
@label("Divisor") @range(1.0, 64.0)
override divisor: f32 = 2.0;
@label("Waveform") @options("Sine", "Triangle", "Square", "Sawtooth")
override waveform: u32 = 0;

// Duplicator + Stagger + Modulate(Remainder: pass → +1, fail → −1) + Oscillator。
fn block(k: u32) -> Offset {
    let n = max(host.members, 1u);
    let delay = cv_stagger(k, n, stagger, 0u);
    let sign = select(-1.0, 1.0, (k % u32(max(divisor, 1.0))) == 0u);
    let x = swing * sign * cv_oscillator(host.time - delay, frequency, 0.0, waveform);
    return Offset(vec2f(x, 0.0), 0.0, 1.0, vec4f(1.0));
}
