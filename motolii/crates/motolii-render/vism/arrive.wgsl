/*{
  "ID": "motolii.arrive",
  "LABEL": "Arrive",
  "STAGE": "block",
  "DESCRIPTION": "Things come in and land where the layout already put them: the box is the destination, this is only how they get there. From, Arrive, Bounce and Stagger — no keys",
  "INPUTS": [
    { "NAME": "side", "LABEL": "From", "TYPE": "float", "DEFAULT": -90.0, "MIN": -360.0, "MAX": 360.0 },
    { "NAME": "distance", "LABEL": "Distance", "TYPE": "float", "DEFAULT": 700.0, "MIN": -100000.0, "MAX": 100000.0 },
    { "NAME": "arrive", "LABEL": "Arrive", "TYPE": "float", "DEFAULT": 0.9, "MIN": 0.05, "MAX": 60.0 },
    { "NAME": "bounce", "LABEL": "Bounce", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "stagger", "LABEL": "Stagger", "TYPE": "float", "DEFAULT": 0.06, "MIN": -10.0, "MAX": 10.0 },
    { "NAME": "spin", "LABEL": "Spin", "TYPE": "float", "DEFAULT": 25.0, "MIN": -3600.0, "MAX": 3600.0 }
  ]
}*/

// 行き先はレイアウトが既に持っている(箱が決めた所)。このブロックは「どう届くか」だけを決める
// — 日常の物理の 3 手のうち、引きと当たり(提案 2026-09-16 の A 型)。
// 時刻の純関数。着いたあとは必ず止まる(利用者「ユーザはおさまりを求めます」)。
fn dice(k: u32) -> f32 {
    var h = k * 2654435761u;
    h = (h ^ (h >> 15u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    return f32(h >> 8u) / 8388608.0 - 1.0;
}

fn block(k: u32, p: BlockParams) -> Offset {
    // 掛かった順に少しずつ遅らせる(Cavalry・AE の Stagger)。
    let start = f32(k) * p.stagger;
    let span = max(p.arrive, 0.05);
    let u = clamp((host.time - start) / span, 0.0, 1.0);
    if u >= 1.0 {
        return NO_OFFSET;
    }
    // 残りの割合: 落ち込みは速く、着く手前で緩む。Bounce が大きいほど、着いてから揺り戻す。
    let fallen = 1.0 - pow(1.0 - u, 3.0);
    let ring = p.bounce * exp(-6.0 * u) * sin(18.8495559 * u);
    let left = clamp(1.0 - fallen + ring, -1.0, 1.0);
    let a = radians(p.side);
    let away = vec2f(cos(a), sin(a)) * p.distance * left;
    let turn = p.spin * left * (0.4 + 0.6 * abs(dice(k))) * sign(dice(k + 613u));
    return Offset(away, turn, 1.0);
}
