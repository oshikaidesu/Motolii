/*{
  "ID": "motolii.petal_wave",
  "LABEL": "Petal Wave",
  "STAGE": "field",
  "DESCRIPTION": "A travelling wave, a twist and a lift bend a flat shape along its length. Phase 0→1 is one full cycle, so keying it linearly over a loop range repeats without a seam",
  "INPUTS": [
    { "NAME": "amount", "LABEL": "Wave", "TYPE": "float", "DEFAULT": 14.0, "MIN": 0.0, "MAX": 1000.0, "HERO": true },
    { "NAME": "wavelength", "LABEL": "Wavelength", "TYPE": "float", "DEFAULT": 160.0, "MIN": 1.0, "MAX": 10000.0 },
    { "NAME": "twist", "LABEL": "Twist", "TYPE": "float", "DEFAULT": 18.0, "MIN": -360.0, "MAX": 360.0 },
    { "NAME": "lift", "LABEL": "Lift", "TYPE": "float", "DEFAULT": 36.0, "MIN": -1000.0, "MAX": 1000.0 },
    { "NAME": "phase", "LABEL": "Phase", "TYPE": "float", "DEFAULT": 0.0, "SUBTYPE": "TIME" }
  ]
}*/

// Along x (the petal's length) a wave travels outward; the tip twists about the length and lifts
// out of the plane, so a mirror surface catches changing light. Every term is periodic in phase.
fn field(in: FieldIn, p: FieldParams) -> FieldOut {
    let tau = 6.28318530718;
    let x = in.frame_position.x;
    let k = tau / max(p.wavelength, 1e-3);
    let s = k * x - tau * p.phase;
    let reach = clamp(abs(x) / max(p.wavelength, 1e-3), 0.0, 1.0);
    // Every term turns a whole number of times per cycle, so phase 1 is phase 0.
    let angle = radians(p.twist) * reach * sin(0.5 * k * x + tau * p.phase);
    let y = in.frame_position.y;
    let wave = p.amount * sin(s);
    let offset = vec3f(0.0, wave + y * (cos(angle) - 1.0), y * sin(angle) + p.lift * reach * (0.5 + 0.5 * cos(s)));
    // The surface tilts with the slope of the wave and the twist.
    let slope = p.amount * k * cos(s);
    let normal = normalize(vec3f(-slope * 0.02, -sin(angle), cos(angle)));
    let base = select(normal, in.normal, all(in.normal == vec3f(0.0)));
    return FieldOut(offset, normalize(mix(base, normal, 0.85)));
}
