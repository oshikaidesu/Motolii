@description("Things come in and land where the layout already put them: the box is the destination, this is only how they get there. From, Arrive, Bounce and Stagger — no keys")

@label("From") @range(-360.0, 360.0)
override side: f32 = -90.0;
@label("Distance") @range(-100000.0, 100000.0)
override distance: f32 = 700.0;
@label("Arrive") @range(0.05, 60.0)
override arrive: f32 = 0.9;
@label("Bounce") @range(0.0, 1.0)
override bounce: f32 = 0.35;
@label("Stagger") @range(-10.0, 10.0)
override stagger: f32 = 0.06;
@label("Spin") @range(-3600.0, 3600.0)
override spin: f32 = 25.0;
@label("Radial") @range(0.0, 1.0)
override radial: f32 = 0.0;

// 行き先はレイアウトが既に持っている(箱が決めた所)。このブロックは「どう届くか」だけを決める
// — 日常の物理の 3 手のうち、引きと当たり(提案 2026-09-16 の A 型)。
// 時刻の純関数。着いたあとは必ず止まる(利用者「ユーザはおさまりを求めます」)。
fn dice(k: u32) -> f32 {
    var h = k * 2654435761u;
    h = (h ^ (h >> 15u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    return f32(h >> 8u) / 8388608.0 - 1.0;
}

fn block(k: u32) -> Offset {
    // 掛かった順に少しずつ遅らせる(Cavalry・AE の Stagger)。
    let start = f32(k) * stagger;
    let span = max(arrive, 0.05);
    let u = clamp((host.time - start) / span, 0.0, 1.0);
    if u >= 1.0 {
        return NO_OFFSET;
    }
    // 残りの割合: 落ち込みは速く、着く手前で緩む。Bounce が大きいほど、着いてから揺り戻す。
    let fallen = 1.0 - pow(1.0 - u, 3.0);
    let ring = bounce * exp(-6.0 * u) * sin(18.8495559 * u);
    let left = clamp(1.0 - fallen + ring, -1.0, 1.0);
    // From は 1 つの向き、Radial は箱の真ん中から見た自分の向き。間は連続(境目を作らない)。
    let a = radians(side);
    var dir = vec2f(cos(a), sin(a));
    let radial = clamp(radial, 0.0, 1.0);
    if radial > 0.0 {
        let mid = (objects[k].lo + objects[k].hi) * 0.5;
        let centre = objects[k].room_lo + objects[k].room_size * 0.5;
        let out = mid - centre;
        if length(out) > 1e-4 {
            dir = normalize(mix(dir, normalize(out), radial));
        }
    }
    let away = dir * distance * left;
    let turn = spin * left * (0.4 + 0.6 * abs(dice(k))) * sign(dice(k + 613u));
    return Offset(away, turn, 1.0, vec4f(1.0));
}
