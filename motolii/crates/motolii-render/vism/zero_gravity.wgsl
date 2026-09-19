@description("Nothing falls. Each thing drifts on its own slow course, tumbles a little, and breathes — the weightless float of a cabin with the gravity off. Pure function of time and index: no solver, no memory")

@label("Drift") @range(0.0, 10000.0)
override drift: f32 = 60.0;
@label("Tumble") @range(-3600.0, 3600.0)
override tumble: f32 = 12.0;
@label("Breath") @range(0.0, 1.0)
override breath: f32 = 0.06;
@label("Slowness") @range(0.2, 120.0)
override slowness: f32 = 7.0;

// 番号から決まる、その物だけの向きと拍(dice)。同じ法が全員に掛かり、誰も同じ道を行かない。
fn dice(k: u32) -> f32 {
    var h = k * 2654435761u;
    h = (h ^ (h >> 15u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    return f32(h >> 8u) / 8388608.0 - 1.0;
}

fn block(k: u32) -> Offset {
    let t = host.time;
    // 漂い: 2 つの遅い正弦の重ね。周期は物ごとに少し違うので、群れが揃わず、ぶつからず、戻って来る。
    let w = 6.2831853 / max(slowness, 0.2);
    let a = dice(k) * 3.1415926;
    let b = dice(k + 97u) * 3.1415926;
    let r1 = 0.5 + 0.5 * abs(dice(k + 211u));
    let r2 = 0.5 + 0.5 * abs(dice(k + 331u));
    let x = cos(a + w * r1 * t) * 0.6 + cos(b + w * r2 * 0.37 * t) * 0.4;
    let y = sin(a + w * r1 * 0.83 * t) * 0.6 + sin(b + w * r2 * 0.29 * t) * 0.4;
    // 転がり: ゆっくり、向きは物ごと。
    let turn = tumble * sin(w * 0.5 * r2 * t + b);
    // 息: 近づいたり遠のいたりを大きさで。
    let size = 1.0 + breath * sin(w * r1 * 0.61 * t + a);
    return Offset(vec2f(x, y) * drift, turn, size, vec4f(1.0));
}
