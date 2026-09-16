/*{
  "ID": "motolii.field",
  "LABEL": "Field",
  "STAGE": "block",
  "SCOPE": "room",
  "REACH": "reach",
  "DESCRIPTION": "The layer becomes a place that pulls, swirls, pushes or blows everything else in the same box. Gravity, attractor, vortex and wind are one thing on two dials with no modes between them",
  "INPUTS": [
    { "NAME": "turn", "LABEL": "Turn", "TYPE": "float", "DEFAULT": 0.0, "MIN": -180.0, "MAX": 180.0 },
    { "NAME": "spread", "LABEL": "Spread", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 90.0, "MIN": -360.0, "MAX": 360.0 },
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 200.0, "MIN": -100000.0, "MAX": 100000.0 },
    { "NAME": "reach", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 100000.0 }
  ]
}*/

// 場は 1 つで、モードの境目は無い(提案 2026-09-16、利用者「既存の概念を連続体にする」)。
//   Turn   0° 引き寄せ → 90° 渦 → 180° 押し出し。途中は渦を巻きながら寄る
//   Spread 0 元から(点の場)→ 1 一様な向き(重力・風)。元が無限に遠い極限が重力
// 元は掛かった層(`host.source`)の箱の真ん中。時刻の純関数で解く(積み上げない)ので、
// 巻き戻しても書き出しても同じ絵になる。重さ(weight)はその場がどれだけ効くか。
fn block(k: u32, p: BlockParams) -> Offset {
    let s = objects[host.source];
    let src = (s.lo + s.hi) * 0.5;
    let mid = (now_lo(k) + now_hi(k)) * 0.5;
    let away = mid - src;
    let d = length(away);
    // 届く輪: 0 なら箱じゅう。輪の中は縁へ行くほど薄れる。
    var fall = 1.0;
    if p.reach > 0.0 {
        if d > p.reach { return NO_OFFSET; }
        fall = 1.0 - d / p.reach;
    }
    let force = p.strength * fall * objects[k].weight;
    let t = max(host.time, 0.0);
    let spread = clamp(p.spread, 0.0, 1.0);

    // 一様: 角度の向きへ落ちる・流される(0.5 * g * t²、comp の下は +90°)。
    let a = radians(p.angle);
    let uniform = vec2f(cos(a), sin(a)) * 0.5 * force * t * t;

    // 元から: 進んだ長さを、向き(Turn)で内向きと回りに分ける。
    var local = vec2f(0.0);
    if d > 1e-4 {
        let travel = force * t;
        let turn = radians(p.turn);
        let r = max(d - cos(turn) * travel, 0.0);
        let spin = sin(turn) * travel / max(d, 1.0);
        let dir = away / d;
        let turned = vec2f(dir.x * cos(spin) - dir.y * sin(spin), dir.x * sin(spin) + dir.y * cos(spin));
        local = turned * r - away;
    }
    return Offset(mix(local, uniform, spread), 0.0, 1.0);
}
