import package::cavalry::{ cv_circle, cv_grid, cv_line, cv_random, cv_spiral, cv_stagger };

@description("Stop and snap. Every beat each thing gets a new place to stand (the formation walks grid → circle → spiral → line, and every thing jitters to its own spot); for Hold of the beat it does not move at all (tome), then it snaps there with an overshoot and lands (hame). Turns snap by 90°, size pops on the hit. Four dials: BPM, Hold, Snap, Wander")

@label("BPM") @range(1.0, 400.0)
override bpm: f32 = 140.0;
@label("Hold") @range(0.0, 0.95)
override hold: f32 = 0.6;
@label("Snap") @range(0.0, 1.0)
override snap: f32 = 0.35;
@label("Wander") @range(0.0, 5000.0)
override wander: f32 = 60.0;
@label("Cell") @range(1.0, 5000.0)
override cell: f32 = 90.0;
@label("Radius") @range(1.0, 5000.0)
override radius: f32 = 400.0;

fn stand(i: u32, k: u32, n: u32) -> vec2f {
    switch i % 4u {
        case 0u: { return cv_grid(k, n, 12u, vec2f(cell, cell)); }
        case 1u: { return cv_circle(k, n, radius, -90.0); }
        case 2u: { return cv_spiral(k, n, radius, 2.5); }
        default: { return cv_line(k, n, vec2f(-radius * 1.8, 0.0), vec2f(radius * 1.8, 0.0)); }
    }
}

// 拍 b での立ち位置: 4 拍ごとの形 + その拍だけの各自のずれ(番号 × 拍の乱数)。
fn place(b: u32, k: u32, n: u32) -> vec2f {
    let form = stand(b / 4u, k, n);
    let j = vec2f(cv_random(k, b * 7u + 1u), cv_random(k, b * 7u + 2u)) * wander;
    return form + j;
}

// ハメの ease: 行き過ぎて戻る(back out)を Snap で強く。
fn hame(u: f32, snap: f32) -> f32 {
    let s = 1.70158 * (0.5 + 3.0 * snap);
    let v = u - 1.0;
    return 1.0 + v * v * ((s + 1.0) * v + s);
}

fn block(k: u32) -> Offset {
    let n = max(host.members, 1u);
    let beat = 60.0 / max(bpm, 1.0);
    // 番号でほんの少し遅らせる(from center): 群れが 1 コマでなく 2〜3 コマで揃う。
    let t = max(host.time - cv_stagger(k, n, 0.0015, 2u), 0.0);
    let b = u32(floor(t / beat));
    let phase = fract(t / beat);
    // トメ: Hold の間は前の場所で完全に止まる。ハメ: 残りで新しい場所へ行き過ぎて戻る。
    let u = clamp((phase - hold) / max(1.0 - hold, 0.01), 0.0, 1.0);
    let e = select(0.0, hame(u, snap), phase >= hold);
    let before = place(b, k, n);
    let after = place(b + 1u, k, n);
    let pos = mix(before, after, e);
    // 回転は 90° 刻みで跳ぶ(拍ごとに各自の向き)。
    let turn_before = floor(cv_random(k, b * 3u + 9u) * 2.0 + 2.0) * 90.0;
    let turn_after = floor(cv_random(k, (b + 1u) * 3u + 9u) * 2.0 + 2.0) * 90.0;
    let turn = mix(turn_before, turn_after, e);
    // 大きさは着いた瞬間に膨らんで縮む(ハメの余韻)。
    let landed = select(0.0, 1.0 - u, phase >= hold);
    let size = 1.0 + snap * 0.8 * landed * landed;
    // 動いている間だけ薄く(トメで濃く、ハメで走る)。
    let moving = select(0.0, 4.0 * u * (1.0 - u), phase >= hold);
    return Offset(pos, turn, size, vec4f(1.0, 1.0, 1.0, 1.0 - 0.5 * moving));
}
