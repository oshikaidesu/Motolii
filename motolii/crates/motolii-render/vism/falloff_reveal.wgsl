/*{
  "ID": "motolii.falloff_reveal",
  "LABEL": "Falloff Reveal",
  "STAGE": "block",
  "DESCRIPTION": "Cavalry's Falloff entrance, packaged: a circle sweeps across; each thing it passes rises from small and clear to its place — Position Blend + Color Blend driven by one Falloff. Sweep sets the pass, Radius the circle, Lift where things wait",
  "INPUTS": [
    { "NAME": "sweep", "LABEL": "Sweep", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.1, "MAX": 120.0 },
    { "NAME": "radius", "LABEL": "Radius", "TYPE": "float", "DEFAULT": 320.0, "MIN": 1.0, "MAX": 10000.0 },
    { "NAME": "lift", "LABEL": "Lift", "TYPE": "float", "DEFAULT": 60.0, "MIN": -10000.0, "MAX": 10000.0 },
    { "NAME": "shrink", "LABEL": "Shrink", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "ease", "LABEL": "Ease", "TYPE": "long", "DEFAULT": 2, "LABELS": ["Linear", "In", "Out", "In Out", "Back", "Elastic"] }
  ]
}*/
import package::cavalry::cv_falloff;

// Falloff(Circle, moving) → Position Blend / Color Blend。円は部屋の左から右へ Sweep 秒で通る。
fn block(k: u32, p: BlockParams) -> Offset {
    let room_lo = objects[k].room_lo;
    let room_size = objects[k].room_size;
    let centre = (objects[k].lo + objects[k].hi) * 0.5;
    // 円の中心: 部屋の左外から右外へ。
    let u = clamp(host.time / max(p.sweep, 0.1), 0.0, 1.0);
    let cx = room_lo.x - p.radius + u * (room_size.x + 2.0 * p.radius);
    let cy = room_lo.y + room_size.y * 0.5;
    let d = length(centre - vec2f(cx, cy));
    // 円が通り過ぎた後は 1 のまま(入場は戻らない): 円の左側は通過済み。
    let passed = select(0.0, 1.0, centre.x < cx);
    let w = max(cv_falloff(d, p.radius, u32(p.ease)), passed);
    let size = mix(1.0 - p.shrink, 1.0, w);
    return Offset(vec2f(0.0, p.lift * (1.0 - w)), 0.0, size, vec4f(1.0, 1.0, 1.0, w));
}
