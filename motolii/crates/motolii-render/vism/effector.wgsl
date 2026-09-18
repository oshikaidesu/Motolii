/*{
  "ID": "motolii.effector",
  "LABEL": "Effector",
  "STAGE": "block",
  "DESCRIPTION": "Notch/Unreal/MoGraph's Effector, packaged: a box of influence sweeps across the room; inside it things rise, turn, grow and light up by one law, outside they sit still. Shape picks Sphere/Box/Plane, Sweep the pass, Strength how much",
  "INPUTS": [
    { "NAME": "shape", "LABEL": "Shape", "TYPE": "long", "DEFAULT": 1, "LABELS": ["Sphere", "Box", "Plane"] },
    { "NAME": "sweep", "LABEL": "Sweep", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.1, "MAX": 120.0 },
    { "NAME": "size", "LABEL": "Size", "TYPE": "float", "DEFAULT": 260.0, "MIN": 1.0, "MAX": 10000.0 },
    { "NAME": "soft", "LABEL": "Soft", "TYPE": "float", "DEFAULT": 120.0, "MIN": 0.0, "MAX": 10000.0 },
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "lift", "LABEL": "Lift", "TYPE": "float", "DEFAULT": -80.0, "MIN": -10000.0, "MAX": 10000.0 }
  ]
}*/
import package::effectors::{ now_centre, ef_apply, ef_sphere, ef_box, ef_plane };

// 法(何が起きるか)は 1 行、形(どこで起きるか)は ef_* の重み。掛けるだけ。
fn block(k: u32, p: BlockParams) -> Offset {
    let room_lo = objects[k].room_lo;
    let room_size = objects[k].room_size;
    let centre = now_centre(k); // 今の位置で判定: 前の段(Wave 等)が動かした後の所に居るかで決まる
    // 影響の中心: 部屋を左から右へ Sweep 秒で往復。
    let u = 0.5 - 0.5 * cos(host.time / max(p.sweep, 0.1) * 3.1415926);
    let cx = room_lo.x + u * room_size.x;
    let c = vec2f(cx, room_lo.y + room_size.y * 0.5);
    var w = 0.0;
    switch u32(p.shape) {
        case 0u: { w = ef_sphere(centre, c, p.size, p.soft / max(p.size, 1.0)); }
        case 2u: { w = ef_plane(centre, vec2f(-1.0, 0.0), -cx, p.soft); }
        default: { w = ef_box(centre, vec2f(cx - p.size * 0.5, room_lo.y), vec2f(cx + p.size * 0.5, room_lo.y + room_size.y), p.soft); }
    }
    let law = Offset(vec2f(0.0, p.lift), 18.0, 1.18, vec4f(2.6, 1.7, 0.5, 1.0));
    return ef_apply(law, w * p.strength);
}
