import package::cavalry::cv_falloff;
import package::effectors::{ now_centre, ef_apply };

@description("A relation in the chain: each thing looks at its neighbours' current state (what the blocks before this one did to them) and catches it — grown or lit neighbours within Reach make this thing rise, turn and warm too. Put it after any effector and the effect spreads from thing to thing")

@label("Reach") @range(0.0, 5000.0) @reach
override reach: f32 = 140.0;
@label("Catch") @range(0.0, 1.0)
override contagion: f32 = 0.8;
@label("Lift") @range(-10000.0, 10000.0)
override lift: f32 = -40.0;
@label("Turn") @range(-3600.0, 3600.0)
override turn: f32 = -12.0;

// 隣の「今」を読む: 前の段が隣を育てた・明るくしたなら、その強さを自分の重みにする。
// 位置も今の位置(now_centre)で測るので、前の段の動きの後の近さで決まる。
fn block(k: u32) -> Offset {
    let here = now_centre(k);
    var w = 0.0;
    for (var i = 0u; i < neighbor_count(k); i++) {
        let j = neighbor(k, i);
        if j == k { continue; }
        let d = length(now_centre(j) - here);
        // 隣の「効き」: 大きさが 1 から離れた分と、明るさが 1 から離れた分の大きい方。
        let s = state_in[j];
        let got = max(abs(s.scale - 1.0) * 5.0, abs(s.tint.r - 1.0));
        w = max(w, clamp(got, 0.0, 1.0) * cv_falloff(d, max(reach, 1.0), 2u));
    }
    let law = Offset(vec2f(0.0, lift), turn, 1.12, vec4f(0.6, 1.9, 2.4, 1.0));
    return ef_apply(law, w * contagion);
}
