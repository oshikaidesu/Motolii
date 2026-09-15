/*{
  "ID": "motolii.field",
  "LABEL": "Field",
  "STAGE": "block",
  "SCOPE": "room",
  "REACH": "reach",
  "DESCRIPTION": "The layer becomes a place that pulls, pushes, blows or swirls everything else in the same box (gravity, wind, attractor and vortex are one thing with four shapes; Cavalry's Fields)",
  "INPUTS": [
    { "NAME": "shape", "LABEL": "Shape", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 90.0, "MIN": -360.0, "MAX": 360.0 },
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 200.0, "MIN": -100000.0, "MAX": 100000.0 },
    { "NAME": "reach", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 100000.0 }
  ]
}*/

// 場: 矢印 1 本と届く輪。形は 0 = 平行(重力・風)、1 = 点(引き寄せ・反発)、2 = 渦。
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
    let t = host.time;
    if p.shape >= 1.5 {
        // 渦: 元の周りを回る。回る量は時刻に比例(近いほど速い)。
        if d <= 1e-4 { return NO_OFFSET; }
        let a = radians(force * t / max(d, 1.0));
        let turned = vec2f(away.x * cos(a) - away.y * sin(a), away.x * sin(a) + away.y * cos(a));
        return Offset(turned - away, 0.0, 1.0);
    }
    if p.shape >= 0.5 {
        // 点: 元へ寄る(負なら離れる)。距離の残りが時刻とともに減るので、行き過ぎない。
        if d <= 1e-4 { return NO_OFFSET; }
        let closed = 1.0 - exp(-abs(force) * t / max(d, 1.0));
        return Offset(-away / d * d * closed * sign(force), 0.0, 1.0);
    }
    // 平行: 角度の向きへ落ちる・流される(0.5 * g * t²、comp の下は +90°)。
    let a = radians(p.angle);
    return Offset(vec2f(cos(a), sin(a)) * 0.5 * force * t * t, 0.0, 1.0);
}
