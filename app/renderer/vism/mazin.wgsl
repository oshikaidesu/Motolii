import package::cavalry::{ cv_ease, cv_random };

@description("Cavalry's Mazin, packaged: a grid of short diagonal strokes (the 10 PRINT maze); a Falloff band sweeps across and every stroke it touches turns 90°, so the maze rewrites itself where the band has been. Put the strokes on a grid; the index decides each one's first tilt")

@label("Sweep") @range(0.1, 120.0)
override sweep: f32 = 4.0;
@label("Band") @range(1.0, 10000.0)
override band: f32 = 220.0;
@label("Seed") @range(0.0, 10000.0)
override seed: f32 = 7.0;
@label("Ease") @options("Linear", "In", "Out", "In Out", "Back", "Elastic")
override ease: u32 = 3;

// Duplicator(Grid) + Value Array {0, 90} + Falloff(Linear, moving) + Jsmath。
fn block(k: u32) -> Offset {
    let room_lo = objects[k].room_lo;
    let room_size = objects[k].room_size;
    let centre = (objects[k].lo + objects[k].hi) * 0.5;
    // 最初の傾き: 番号の乱数で 0 か 90(10 PRINT CHR$(205.5+RND(1)); : GOTO 10)。
    let first = select(0.0, 90.0, cv_random(k, u32(seed)) > 0.0);
    // 帯は左外から右外へ Sweep 秒で通り、また左から。通った回数だけ 90° 足す(通過中は Ease で回る)。
    let period = max(sweep, 0.1);
    let passes = floor(host.time / period);
    let u = fract(host.time / period);
    let bx = room_lo.x - band + u * (room_size.x + 2.0 * band);
    let d = centre.x - bx;
    // 帯の右端がまだ来ていない: 0、帯の中: 回っている途中、帯が過ぎた: 1。
    let w = 1.0 - cv_ease(clamp(d / band + 0.5, 0.0, 1.0), ease);
    let turned = (passes + w) * 90.0;
    return Offset(vec2f(0.0), first + turned, 1.0, vec4f(1.0));
}
