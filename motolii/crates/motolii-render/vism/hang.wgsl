/*{
  "ID": "motolii.hang",
  "LABEL": "Hang",
  "STAGE": "block",
  "DESCRIPTION": "The thing hangs from a point above where the layout put it, swings, and comes to rest. Length sets the beat the way a real pendulum does, so there is no frequency to tune",
  "INPUTS": [
    { "NAME": "length", "LABEL": "Length", "TYPE": "float", "DEFAULT": 300.0, "MIN": 1.0, "MAX": 100000.0 },
    { "NAME": "swing", "LABEL": "Swing", "TYPE": "float", "DEFAULT": 18.0, "MIN": -180.0, "MAX": 180.0 },
    { "NAME": "settle", "LABEL": "Settle", "TYPE": "float", "DEFAULT": 2.5, "MIN": 0.05, "MAX": 120.0 },
    { "NAME": "stagger", "LABEL": "Stagger", "TYPE": "float", "DEFAULT": 0.12, "MIN": -10.0, "MAX": 10.0 },
    { "NAME": "sway", "LABEL": "Sway", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "beat", "LABEL": "Beat", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 120.0 }
  ]
}*/

// 繋がりの型(提案 2026-09-16 の C): 留め具はレイアウトが決めた場所の真上。振れ幅は Swing、
// 拍は Length が決める(本物の振り子と同じ T = 2π√(L/g)。だから周波数の欄は要らない)。
// Settle で必ず止まる。Sway を上げると、止まらずに息をする(のれん・吊り看板)。
fn block(k: u32, p: BlockParams) -> Offset {
    let t = max(host.time - f32(k) * p.stagger, 0.0);
    let length = max(p.length, 1.0);
    // 重力は comp の 1 秒 1000px を 1G と見る(画の縮尺の嘘。長いほどゆっくり振れる)。
    // Beat を入れると拍をそちらに合わせる — 糸と札のように、1 つの物を何枚かで組む時に要る。
    let beat = select(6.2831853 * sqrt(length / 1000.0), p.beat, p.beat > 0.0);
    let fade = exp(-t / max(p.settle, 0.05));
    let alive = mix(fade, mix(fade, 1.0, 0.65), clamp(p.sway, 0.0, 1.0));
    let angle = radians(p.swing) * alive * cos(6.2831853 * t / beat);
    // 留め具の周りに回す(留め具は箱の真ん中の Length だけ上)。
    let mid = (objects[k].lo + objects[k].hi) * 0.5;
    let pin = mid - vec2f(0.0, length);
    let arm = mid - pin;
    let turned = vec2f(arm.x * cos(angle) - arm.y * sin(angle), arm.x * sin(angle) + arm.y * cos(angle));
    return Offset(turned - arm, degrees(angle), 1.0, vec4f(1.0));
}
