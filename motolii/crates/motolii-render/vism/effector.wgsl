import package::effectors::{ now_centre, anchor_centre, ef_apply, ef_sphere, ef_box, ef_plane };

@description("Notch/Unreal/MoGraph's Effector, packaged: a box of influence sweeps across the room; inside it things rise, turn, grow and light up by one law, outside they sit still. Shape picks Sphere/Box/Plane, Sweep the pass, Strength how much")

@label("Shape") @options("Sphere", "Box", "Plane")
override shape: u32 = 1;
@label("Sweep") @range(0.1, 120.0)
override sweep: f32 = 3.0;
@label("Size") @range(1.0, 10000.0)
override size: f32 = 260.0;
@label("Soft") @range(0.0, 10000.0)
override soft: f32 = 120.0;
@label("Strength") @range(0.0, 1.0)
override strength: f32 = 1.0;
@label("Lift") @range(-10000.0, 10000.0)
override lift: f32 = -80.0;

// 法(何が起きるか)は 1 行、形(どこで起きるか)は ef_* の重み。掛けるだけ。
fn block(k: u32) -> Offset {
    let room_lo = objects[k].room_lo;
    let room_size = objects[k].room_size;
    let centre = now_centre(k); // 今の位置で判定: 前の段(Wave 等)が動かした後の所に居るかで決まる
    // 影響の中心: Position Anchor の相手が居ればその今の中心(鍵で動く相手 1 つが全員の法になる)、
    // 無ければ部屋を左から右へ Sweep 秒で往復。
    let u = 0.5 - 0.5 * cos(host.time / max(sweep, 0.1) * 3.1415926);
    var c = vec2f(room_lo.x + u * room_size.x, room_lo.y + room_size.y * 0.5);
    if objects[k].anchor_slot != NO_OBJECT { c = anchor_centre(k); }
    var w = 0.0;
    switch shape {
        case 0u: { w = ef_sphere(centre, c, size, soft / max(size, 1.0)); }
        case 2u: { w = ef_plane(centre, vec2f(-1.0, 0.0), -c.x, soft); }
        default: { w = ef_box(centre, vec2f(c.x - size * 0.5, room_lo.y), vec2f(c.x + size * 0.5, room_lo.y + room_size.y), soft); }
    }
    let law = Offset(vec2f(0.0, lift), 18.0, 1.18, vec4f(2.6, 1.7, 0.5, 1.0));
    return ef_apply(law, w * strength);
}
