import package::cavalry::cv_falloff;

@description("Cavalry's Falloff entrance, packaged: a circle sweeps across; each thing it passes rises from small and clear to its place — Position Blend + Color Blend driven by one Falloff. Sweep sets the pass, Radius the circle, Lift where things wait")

@label("Sweep") @range(0.1, 120.0)
override sweep: f32 = 3.0;
@label("Radius") @range(1.0, 10000.0)
override radius: f32 = 320.0;
@label("Lift") @range(-10000.0, 10000.0)
override lift: f32 = 60.0;
@label("Shrink") @range(0.0, 1.0)
override shrink: f32 = 0.6;
@label("Ease") @options("Linear", "In", "Out", "In Out", "Back", "Elastic")
override ease: u32 = 2;

// Falloff(Circle, moving) → Position Blend / Color Blend。円は部屋の左から右へ Sweep 秒で通る。
fn block(k: u32) -> Offset {
    let room_lo = objects[k].room_lo;
    let room_size = objects[k].room_size;
    let centre = (objects[k].lo + objects[k].hi) * 0.5;
    // 円の中心: 部屋の左外から右外へ。
    let u = clamp(host.time / max(sweep, 0.1), 0.0, 1.0);
    let cx = room_lo.x - radius + u * (room_size.x + 2.0 * radius);
    let cy = room_lo.y + room_size.y * 0.5;
    let d = length(centre - vec2f(cx, cy));
    // 円が通り過ぎた後は 1 のまま(入場は戻らない): 円の左側は通過済み。
    let passed = select(0.0, 1.0, centre.x < cx);
    let w = max(cv_falloff(d, radius, ease), passed);
    let size = mix(1.0 - shrink, 1.0, w);
    return Offset(vec2f(0.0, lift * (1.0 - w)), 0.0, size, vec4f(1.0, 1.0, 1.0, w));
}
