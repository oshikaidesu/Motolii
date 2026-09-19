import package::processing::TWO_PI;
import package::cavalry::{ cv_circle, cv_ease, cv_falloff, cv_grid, cv_line, cv_oscillator, cv_random, cv_spiral, cv_stagger };

@description("Too much on purpose: every law on the shelf multiplied into one. The index puts each thing on a walking formation (grid → circle → spiral → line), a slow float drifts it, the beat kicks size and brightness, a band sweeps and flips every other thing, everything tumbles. Excess by stacking laws, not by adding hands")

@label("BPM") @range(1.0, 400.0)
override bpm: f32 = 128.0;
@label("Kick") @range(0.0, 2.0)
override kick: f32 = 0.35;
@label("Hold") @range(0.1, 60.0)
override hold: f32 = 1.875;
@label("Cell") @range(1.0, 5000.0)
override cell: f32 = 70.0;
@label("Radius") @range(1.0, 5000.0)
override radius: f32 = 420.0;
@label("Drift") @range(0.0, 5000.0)
override drift: f32 = 26.0;
@label("Tumble") @range(-3600.0, 3600.0)
override tumble: f32 = 40.0;
@label("Sweep") @range(0.1, 120.0)
override sweep: f32 = 3.75;

fn stand(i: u32, k: u32, n: u32) -> vec2f {
    switch i % 4u {
        case 0u: { return cv_grid(k, n, 16u, vec2f(cell, cell)); }
        case 1u: { return cv_circle(k, n, radius, -90.0); }
        case 2u: { return cv_spiral(k, n, radius, 3.0); }
        default: { return cv_line(k, n, vec2f(-radius * 1.9, 0.0), vec2f(radius * 1.9, 0.0)) + vec2f(0.0, sin(f32(k) * 0.7) * cell * 1.5); }
    }
}

fn block(k: u32) -> Offset {
    let n = max(host.members, 1u);
    let t = host.time;
    // 1. 立つ場所: 拍 8 つごとに次の形へ(Formations)。移る間は Ease で。
    let beat = 60.0 / max(bpm, 1.0);
    let period = hold + beat * 2.0;
    let i = u32(floor(t / period));
    let u = clamp((t - f32(i) * period - hold) / (beat * 2.0), 0.0, 1.0);
    var pos = mix(stand(i, k, n), stand(i + 1u, k, n), cv_ease(u, 3u));
    // 2. 漂い(Zero Gravity): 各自の遅い道。
    let w = TWO_PI / 9.0;
    let a = cv_random(k, 3u) * 3.1415926;
    let r1 = 0.5 + 0.5 * abs(cv_random(k, 11u));
    pos += vec2f(cos(a + w * r1 * t), sin(a + w * r1 * 0.83 * t)) * drift;
    // 3. 拍(Beat): 全員が同じ瞬間に大きく明るく、番号で少しだけ遅れる(Stagger)。
    let delay = cv_stagger(k, n, 0.004, 2u);
    let phase = fract((t - delay) / beat);
    let hit = pow(1.0 - phase, 4.0);
    // 4. 走査する帯(Falloff Sweep): 通った所は交互に反転(Modulate)し、一瞬膨らむ。
    let room_lo = objects[k].room_lo;
    let room_size = objects[k].room_size;
    let centre = (objects[k].lo + objects[k].hi) * 0.5 + pos;
    let su = fract(t / max(sweep, 0.1));
    let bx = room_lo.x - 200.0 + su * (room_size.x + 400.0);
    let band = cv_falloff(abs(centre.x - bx), 180.0, 2u);
    let sign = select(-1.0, 1.0, (k % 2u) == 0u);
    pos += vec2f(0.0, sign * band * cell * 0.8);
    // 5. 転がり(Tumble): 番号ごとの向きで、拍で蹴られる。
    let turn = tumble * cv_oscillator(t, 0.11, cv_random(k, 5u), 0u) + sign * band * 90.0;
    // 6. 大きさ・不透明: 拍 × 帯 × 番号の交互。
    let size = (1.0 + kick * hit) * (1.0 + 0.5 * band);
    let dim = 0.55 + 0.45 * hit + 0.3 * band;
    // 7. 色: 帯の中だけ色を裏返す(tint の rgb を落として背景色に近づける)。
    let tint = mix(vec3f(1.0), vec3f(0.15, 0.15, 0.2), band * 0.7);
    return Offset(pos, turn, size, vec4f(tint, clamp(dim, 0.0, 1.0)));
}
