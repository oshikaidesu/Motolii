import package::kynd::{ linearstep, linearstepUpDown };
import package::easing::{ backOut, elasticOut, cubicIn };
import package::iq::{ expImpulse };
import package::cavalry::cv_stagger;
import package::compute_toys::beat;

@description("kynd's Sequencing (MotionToolKit) as one block: every thing runs the same timeline of windows — slide in with a back overshoot, hold, snap a quarter turn with an elastic, pulse on the beat, then leave — each delayed by its place. Time windows in seconds, Stagger per thing, BPM for the pulse")

@label("Stagger") @range(0.0, 5.0)
override stagger: f32 = 0.03;
@label("Enter") @range(0.05, 10.0)
override enter: f32 = 0.6;
@label("Hold") @range(0.0, 10.0)
override hold: f32 = 0.8;
@label("Turn") @range(0.05, 10.0)
override turn: f32 = 0.6;
@label("Pulses") @range(0.0, 60.0)
override pulses: f32 = 2.0;
@label("BPM") @range(1.0, 400.0)
override bpm: f32 = 120.0;
@label("Leave") @range(0.05, 10.0)
override leave: f32 = 0.5;
@label("Drop") @range(-10000.0, 10000.0)
override drop: f32 = 240.0;


// 窓は秒で並ぶ: [0, enter) 入場 → hold → [t1, t1+turn) 回転 → 拍の脈 pulses 拍 → [t3, t3+leave) 退場。
fn block(k: u32) -> Offset {
    let t = host.time - cv_stagger(k, host.objects, stagger, 2u);
    let t1 = enter + hold;
    let t2 = t1 + turn;
    let t3 = t2 + pulses * 60.0 / bpm;
    // 1. 入場: 下から、行き過ぎて戻る。
    let e = backOut(linearstep(0.0, enter, t));
    // 2. 回転: 90 度、弾んで止まる。
    let r = 90.0 * elasticOut(linearstep(t1, t2, t));
    // 3. 拍の脈: 拍ごとに急に膨らんで戻る(expImpulse)。回転の後だけ。
    let b = beat(max(t - t2, 0.0), bpm);
    let inside = step(t2, t) * (1.0 - step(t3, t));
    let pulse = expImpulse(fract(b), 8.0) * inside;
    // 4. 退場: 上へ、加速して消える。行きと帰りの引き算で不透明。
    let l = cubicIn(linearstep(t3, t3 + leave, t));
    let alpha = linearstepUpDown(0.0, enter * 0.5, t3, t3 + leave, t);
    let y = drop * (1.0 - e) - drop * 0.6 * l;
    return Offset(vec2f(0.0, y), r, 1.0 + 0.3 * pulse, vec4f(1.0, 1.0 - 0.6 * pulse, 1.0 - 0.9 * pulse, alpha));
}
