/*{
  "ID": "motolii.push_apart",
  "LABEL": "Push Apart",
  "STAGE": "block",
  "ROUNDS": 32,
  "DESCRIPTION": "Things in the same box push each other apart until their margins clear (C4D Push Apart; Motolii's margin law). Each round measures everyone near at once and moves them together",
  "INPUTS": [
    { "NAME": "margin", "LABEL": "Margin", "TYPE": "float", "DEFAULT": 10.0, "MIN": 0.0, "MAX": 1000.0 }
  ]
}*/

fn block(k: u32, p: BlockParams) -> Offset {
    let a = objects[k];
    let ma = select(p.margin, a.margin, a.margin > 0.0);
    let alo = now_lo(k) - vec2f(ma);
    let ahi = now_hi(k) + vec2f(ma);
    var step = vec2f(0.0);
    for (var i = 0u; i < neighbor_count(k); i++) {
        let j = neighbor(k, i);
        let b = objects[j];
        let s = a.weight + b.weight;
        if s <= 0.0 { continue; }
        let mb = select(p.margin, b.margin, b.margin > 0.0);
        let blo = now_lo(j) - vec2f(mb);
        let bhi = now_hi(j) + vec2f(mb);
        // 中心から中心への向きに、離れるのに要るだけ(浅い軸で押すと、軸が入れ替わる瞬間に向きが 90° 跳ぶ)。
        let gap = (blo + bhi) * 0.5 - (alo + ahi) * 0.5;
        var dir = select(vec2f(-1.0, 0.0), vec2f(1.0, 0.0), k < j);
        if length(gap) > 1e-4 { dir = normalize(gap); }
        let half = ((ahi - alo) + (bhi - blo)) * 0.5;
        if half.x - abs(gap.x) <= 0.0 || half.y - abs(gap.y) <= 0.0 { continue; }
        let need_x = select(1e30, max((half.x - abs(gap.x)) / abs(dir.x), 0.0), abs(dir.x) >= 1e-6);
        let need_y = select(1e30, max((half.y - abs(gap.y)) / abs(dir.y), 0.0), abs(dir.y) >= 1e-6);
        let depth = min(need_x, need_y);
        if depth <= 0.0 || depth >= 1e29 { continue; }
        step -= dir * depth * (a.weight / s) * 0.5;
    }
    return Offset(step, 0.0, 1.0);
}
