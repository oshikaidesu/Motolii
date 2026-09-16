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
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 400.0, "MIN": -100000.0, "MAX": 100000.0 },
    { "NAME": "reach", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 100000.0 },
    { "NAME": "tumble", "LABEL": "Tumble", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "hold", "LABEL": "Hold", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

// 場は 1 つで、モードの境目は無い(提案 2026-09-16、利用者「既存の概念を連続体にする」)。
//   Turn   0° 引き寄せ → 90° 渦 → 180° 押し出し。途中は渦を巻きながら寄る
//   Spread 0 元から(点の場)→ 1 一様な向き(重力・風)。元が無限に遠い極限が重力
// 元は掛かった層(`host.source`)の箱の真ん中。時刻の純関数で解く(積み上げない)ので、
// 巻き戻しても書き出しても同じ絵になる。重さ(weight)はその場がどれだけ効くか。
// 物ごとのさいころ(番号から作るので、巻き戻しても書き出しても同じ)。-1 〜 1。
fn dice(k: u32) -> f32 {
    var h = k * 2654435761u;
    h = (h ^ (h >> 15u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    return f32(h >> 8u) / 8388608.0 - 1.0;
}

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
    let turn = radians(p.turn);
    // 人は収まりを求める(利用者 2026-09-16)。力は加速し続けず、落ち着く:
    //   一様 = 落ち始めだけ加速して終端の速さへ(EASE 秒)。止まるのは箱の縁と、下に居る物
    //   元から = 指数で寄る・離れる。回りは決まった角度まで回って止まる
    let ease = 0.35;
    let ramp = 1.0 - exp(-t / ease);
    // 立ち上がりの時刻: 0 から急に動き出さない(利用者 2026-09-16「動きは離散的にならないように」)。
    // 家に留めてある物(Hold)は、時間で積み上げず距離だけで決める — 積み上げると紐で頭打ちになり、
    // 元が離れた時に一気に戻ってガタつく。留まる物にとって場は「今どれだけ押されているか」でよい。
    let held = clamp(p.hold, 0.0, 1.0);
    let te = mix(t - ease * ramp, min(t, 1.0) * 0.6, held);

    // 一様: 角度の向きへ、終端の速さ force px/秒 で落ちる・流される(comp の下は +90°)。
    let a = radians(p.angle);
    let uniform = vec2f(cos(a), sin(a)) * force * te;

    // 元から: 寄る・離れる長さは指数、回る角は決まった量まで。
    // 元のすぐ近くで向きが跳ねないよう、距離に芯を持たせる(N 体計算の softening と同じ)。
    var local = vec2f(0.0);
    if d > 1e-4 {
        let core = max(objects[k].hi.x - objects[k].lo.x, objects[k].hi.y - objects[k].lo.y) * 0.75;
        let soft = sqrt(d * d + core * core);
        let rate = force / 200.0 * (d / soft);
        let r = d * exp(-cos(turn) * rate * te);
        let spin = sin(turn) * 12.566371 * (1.0 - exp(-abs(rate) * te));
        let dir = away / d;
        let turned = vec2f(dir.x * cos(spin) - dir.y * sin(spin), dir.x * sin(spin) + dir.y * cos(spin));
        local = turned * r - away;
    }
    // まわり: 進んだ長さを自分の大きさで割った分だけ転がる(落ちれば転がり、止まれば止まる)。
    // 向きと速さは物ごとのさいころで散らす — 揃って回ると作り物に見える。
    var moved = mix(local, uniform, spread);
    let own = max(max(objects[k].hi.x - objects[k].lo.x, objects[k].hi.y - objects[k].lo.y), 1.0);
    // Hold: 配置(レイアウト)が家。人は箱を並べた時点で床も余白も決めている(利用者 2026-09-16
    // 「ユーザはもう既にレイアウト構図という形で、壁や地面を無意識下で設定している」)。
    // 0 = 家を離れて箱の底まで行く、1 = 家から離れない(膨らんで戻る)。間は連続。
    let hold = held;
    if hold > 0.0 {
        // 紐の長さは Hold に反比例(1 で自分の大きさの 1.5 倍、0.1 で 15 倍)。混ぜ算だと 0.8 でも
        // ほぼ無限に伸びてしまい、家に留まらなかった。
        let leash = own * 1.5 / max(hold, 1.0e-3);
        let far = length(moved);
        if far > 1e-4 {
            moved = moved * (leash / (leash + far));
        }
    }
    // 箱から出さない: 出た分は場が自分で止める。止めないと、落ち続けた物を箱の解き手が毎コマ
    // 引き戻す事になり、毎コマ違う並びに落ち着いて画がガタつく(利用者 2026-09-16「すごいガタガタ」)。
    let room = objects[k].room_size;
    if room.x > 0.0 && room.y > 0.0 {
        let size = objects[k].hi - objects[k].lo;
        let low = objects[k].room_lo;
        let high = objects[k].room_lo + max(room - size, vec2f(0.0));
        let want = objects[k].lo + moved;
        moved += clamp(want, low, high) - want;
    }

    let spin = degrees(length(moved) / (own * 0.5)) * p.tumble * (0.4 + 0.6 * abs(dice(k))) * sign(dice(k + 977u));
    return Offset(moved, spin, 1.0);
}
