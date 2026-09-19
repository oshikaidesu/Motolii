@description("The thing hangs from a point above where the layout put it, swings, and comes to rest. Length sets the beat the way a real pendulum does, so there is no frequency to tune")

@label("Length") @range(1.0, 100000.0)
override length: f32 = 300.0;
@label("Swing") @range(-180.0, 180.0)
override swing: f32 = 18.0;
@label("Settle") @range(0.05, 120.0)
override settle: f32 = 2.5;
@label("Stagger") @range(-10.0, 10.0)
override stagger: f32 = 0.12;
@label("Sway") @range(0.0, 1.0)
override sway: f32 = 0.0;
@label("Beat") @range(0.0, 120.0)
override beat: f32 = 0.0;

// 繋がりの型(提案 2026-09-16 の C): 留め具はレイアウトが決めた場所の真上。振れ幅は Swing、
// 拍は Length が決める(本物の振り子と同じ T = 2π√(L/g)。だから周波数の欄は要らない)。
// Settle で必ず止まる。Sway を上げると、止まらずに息をする(のれん・吊り看板)。
fn block(k: u32) -> Offset {
    let t = max(host.time - f32(k) * stagger, 0.0);
    let length = max(length, 1.0);
    // 重力は comp の 1 秒 1000px を 1G と見る(画の縮尺の嘘。長いほどゆっくり振れる)。
    // Beat を入れると拍をそちらに合わせる — 糸と札のように、1 つの物を何枚かで組む時に要る。
    let beat = select(6.2831853 * sqrt(length / 1000.0), beat, beat > 0.0);
    let fade = exp(-t / max(settle, 0.05));
    let alive = mix(fade, mix(fade, 1.0, 0.65), clamp(sway, 0.0, 1.0));
    let angle = radians(swing) * alive * cos(6.2831853 * t / beat);
    // 留め具の周りに回す(留め具は箱の真ん中の Length だけ上)。
    let mid = (objects[k].lo + objects[k].hi) * 0.5;
    let pin = mid - vec2f(0.0, length);
    let arm = mid - pin;
    let turned = vec2f(arm.x * cos(angle) - arm.y * sin(angle), arm.x * sin(angle) + arm.y * cos(angle));
    return Offset(turned - arm, degrees(angle), 1.0, vec4f(1.0));
}
