/*{
  "ID": "motolii.room",
  "LABEL": "Room",
  "STAGE": "block",
  "ROUNDS": 16,
  "EXPOSE": false,
  "DESCRIPTION": "The box things live in, solved: they keep their margins from each other and they stay inside. Nobody asks for this — the world is already there (Motolii's margin law, plus the box's own edges)",
  "INPUTS": [
    { "NAME": "down_x", "LABEL": "Down X", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "down_y", "LABEL": "Down Y", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 }
  ]
}*/

// 箱そのものの解き手。棚には出ない(`EXPOSE: false`)。人が頼むのは「重力」だけで、
// 床や壁を作る発想は生まれない — 既に在って当たり前だから(利用者 2026-09-16)。
// 1 回ごとに、近くの物と間合いを空け、それから箱の中へ戻す。
// 回った物の外枠(傾いた四角は角が出る)。当たりは外枠どうしで見る。
fn turned_half(k: u32) -> vec2f {
    let half = (objects[k].hi - objects[k].lo) * 0.5;
    let a = radians(state_in[k].rotate);
    return vec2f(abs(cos(a)) * half.x + abs(sin(a)) * half.y, abs(sin(a)) * half.x + abs(cos(a)) * half.y);
}

// 箱に収めた今の真ん中(相手も同じ見方で見る — 箱の外に居る物と押し合うと、毎コマ答えが変わる)。
fn mid_in_room(k: u32) -> vec2f {
    let mid = (now_lo(k) + now_hi(k)) * 0.5;
    let room = objects[k].room_size;
    if room.x <= 0.0 || room.y <= 0.0 {
        return mid;
    }
    let half = turned_half(k);
    let centre = objects[k].room_lo + room * 0.5;
    if objects[k].radius * 2.0 >= min(room.x, room.y) - 1e-3 {
        let reach = max(min(room.x, room.y) * 0.5 - length(half), 0.0);
        let away = mid - centre;
        let r = length(away);
        return select(mid, centre + away / r * reach, r > reach && r >= 1e-6);
    }
    return clamp(mid, objects[k].room_lo + half, objects[k].room_lo + max(room - half, half));
}

fn block(k: u32, p: BlockParams) -> Offset {
    let a = objects[k];
    // 下へ詰める: 場が向いている先へ毎回少しずつ寄せる。押し合いと箱が止めるので、積もって収まる
    // (詰めないと、押し合いで上へ逃げた物がそのまま浮いて見える)。
    let down = vec2f(p.down_x, p.down_y);
    // 重なりを 0 にしようとすると必ず震える(Catto, GDC 2009: "aiming for zero overlap leads to jitter")。
    // 少しめり込ませたまま(slop)、深さの一部だけ直す。回数を増やして 0 に寄せるのではなく、
    // 入力に対して滑らかな答えを返す事を選ぶ — こちらは前のコマを持てない(時刻の純関数)ので、
    // 収束の速さより滑らかさが要る(Macklin 2019 の substep も、回数では買えないと言っている)。
    let own_side = max(min(a.hi.x - a.lo.x, a.hi.y - a.lo.y), 1.0);
    let slop = own_side * 0.03;
    let press = down * own_side * 0.05;
    let amid = mid_in_room(k) + press;
    let ahalf = turned_half(k) + vec2f(a.margin);
    let alo = amid - ahalf;
    let ahi = amid + ahalf;
    var step = press + mid_in_room(k) - (now_lo(k) + now_hi(k)) * 0.5;
    // 重なった相手ごとの直しは足さずに平均する。全部足すと行き過ぎて、コマごとに別の並びへ
    // 落ち着く(利用者 2026-09-16「すごいガタガタ」)。
    var sep = vec2f(0.0);
    var hits = 0.0;
    for (var i = 0u; i < neighbor_count(k); i++) {
        let j = neighbor(k, i);
        let b = objects[j];
        let s = a.weight + b.weight;
        if s <= 0.0 { continue; }
        let bmid = mid_in_room(j);
        let bhalf = turned_half(j) + vec2f(b.margin);
        let blo = bmid - bhalf;
        let bhi = bmid + bhalf;
        // 中心から中心への向きに、離れるのに要るだけ(浅い軸で押すと、軸が入れ替わる瞬間に向きが 90° 跳ぶ)。
        let gap = (blo + bhi) * 0.5 - (alo + ahi) * 0.5;
        var dir = select(vec2f(-1.0, 0.0), vec2f(1.0, 0.0), k < j);
        if length(gap) > 1e-4 { dir = normalize(gap); }
        let half = ((ahi - alo) + (bhi - blo)) * 0.5;
        if half.x - abs(gap.x) <= 0.0 || half.y - abs(gap.y) <= 0.0 { continue; }
        let need_x = select(1e30, max((half.x - abs(gap.x)) / abs(dir.x), 0.0), abs(dir.x) >= 1e-6);
        let need_y = select(1e30, max((half.y - abs(gap.y)) / abs(dir.y), 0.0), abs(dir.y) >= 1e-6);
        let depth = min(need_x, need_y) - slop;
        if depth <= 0.0 || depth >= 1e29 { continue; }
        sep -= dir * depth * (a.weight / s);
        hits += 1.0;
    }
    if hits > 0.0 {
        // 深さの一部だけ(Baumgarte の bias)。全部直すと行き過ぎ、相手も同時に直すので二重になる。
        step += sep / hits * 0.25;
    }
    // 箱の中へ戻す(跳ね返さず、縁で止まる — 落ちた物は床に着いて、そこに在る)。
    let size = a.room_size;
    if size.x <= 0.0 || size.y <= 0.0 {
        return Offset(step, 0.0, 1.0);
    }
    let mid = (now_lo(k) + now_hi(k)) * 0.5 + step;
    let own = turned_half(k) * 2.0;
    let lo = mid - own * 0.5;
    let hi = mid + own * 0.5;
    if a.radius * 2.0 >= min(size.x, size.y) - 1e-3 {
        // 丸い箱(帯は短辺の丸): 真ん中からの長さで止める。
        let centre = a.room_lo + size * 0.5;
        let room = max(min(size.x, size.y) * 0.5 - length(own) * 0.5, 0.0);
        let away = (lo + hi) * 0.5 - centre;
        let r = length(away);
        if r > room && r >= 1e-6 {
            step += away / r * (room - r);
        }
        return Offset(step, 0.0, 1.0);
    }
    let low = a.room_lo;
    let high = a.room_lo + max(size - own, vec2f(0.0));
    step += clamp(lo, low, high) - lo;
    return Offset(step, 0.0, 1.0);
}
