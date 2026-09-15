/*{
  "ID": "motolii.bounce_block",
  "LABEL": "Bounce",
  "STAGE": "block",
  "DESCRIPTION": "Folds each box back inside the box it lives in, like a ball between walls (a lie of physics: straight motion mirrored at the walls)",
  "INPUTS": [
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

// x を [a, a + room] の中へ鏡で折り返す(周期 2 room の三角波)。
fn fold(x: f32, a: f32, room: f32) -> f32 {
    if room <= 1e-3 { return a + room * 0.5; }
    let period = 2.0 * room;
    let m = (x - a) - floor((x - a) / period) * period;
    return a + select(period - m, m, m <= room);
}

fn block(k: u32, p: BlockParams) -> Offset {
    let it = objects[k];
    let lo = now_lo(k);
    let hi = now_hi(k);
    let size = it.room_size;
    var shift = vec2f(0.0);
    if it.radius * 2.0 >= min(size.x, size.y) - 1e-3 && size.x > 0.0 && size.y > 0.0 {
        // 円(帯は短辺の円): 中心を通る直線の上で、向こうの壁まで往復する。
        let center = it.room_lo + size * 0.5;
        let own = max(hi.x - lo.x, hi.y - lo.y) * 0.5;
        let room = max(min(size.x, size.y) * 0.5 - own, 0.0);
        let mid = (lo + hi) * 0.5;
        let away = mid - center;
        let r = length(away);
        if r > room && r >= 1e-6 {
            shift = center + away / r * fold(r, -room, 2.0 * room) - mid;
        }
    } else {
        let w = hi - lo;
        shift = vec2f(fold(lo.x, it.room_lo.x, size.x - w.x), fold(lo.y, it.room_lo.y, size.y - w.y)) - lo;
    }
    return Offset(shift * p.strength, 0.0, 1.0);
}
