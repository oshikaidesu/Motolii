/*{
  "ID": "motolii.neighbour_effector",
  "LABEL": "Neighbour Effector",
  "STAGE": "block",
  "REACH": "reach",
  "DESCRIPTION": "A relation in the chain: each thing looks at its neighbours' current state (what the blocks before this one did to them) and catches it — grown or lit neighbours within Reach make this thing rise, turn and warm too. Put it after any effector and the effect spreads from thing to thing",
  "INPUTS": [
    { "NAME": "reach", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 140.0, "MIN": 0.0, "MAX": 5000.0 },
    { "NAME": "contagion", "LABEL": "Catch", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "lift", "LABEL": "Lift", "TYPE": "float", "DEFAULT": -40.0, "MIN": -10000.0, "MAX": 10000.0 },
    { "NAME": "turn", "LABEL": "Turn", "TYPE": "float", "DEFAULT": -12.0, "MIN": -3600.0, "MAX": 3600.0 }
  ]
}*/
import package::cavalry::cv_falloff;
import package::effectors::{ now_centre, ef_apply };

// 隣の「今」を読む: 前の段が隣を育てた・明るくしたなら、その強さを自分の重みにする。
// 位置も今の位置(now_centre)で測るので、前の段の動きの後の近さで決まる。
fn block(k: u32, p: BlockParams) -> Offset {
    let here = now_centre(k);
    var w = 0.0;
    for (var i = 0u; i < neighbor_count(k); i++) {
        let j = neighbor(k, i);
        if j == k { continue; }
        let d = length(now_centre(j) - here);
        // 隣の「効き」: 大きさが 1 から離れた分と、明るさが 1 から離れた分の大きい方。
        let s = state_in[j];
        let got = max(abs(s.scale - 1.0) * 5.0, abs(s.tint.r - 1.0));
        w = max(w, clamp(got, 0.0, 1.0) * cv_falloff(d, max(p.reach, 1.0), 2u));
    }
    let law = Offset(vec2f(0.0, p.lift), p.turn, 1.12, vec4f(0.6, 1.9, 2.4, 1.0));
    return ef_apply(law, w * p.contagion);
}
