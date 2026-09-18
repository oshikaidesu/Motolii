/*{
  "ID": "motolii.ring_ting",
  "LABEL": "Ring Ting",
  "STAGE": "block",
  "DESCRIPTION": "Cavalry's Ring Ting, packaged: things on a ring; a Noise deformer pushes them off the ring only where a Falloff spot is, and the spot walks round. Put everything at one point; the index puts it on the ring",
  "INPUTS": [
    { "NAME": "radius", "LABEL": "Radius", "TYPE": "float", "DEFAULT": 360.0, "MIN": 1.0, "MAX": 10000.0 },
    { "NAME": "amount", "LABEL": "Amount", "TYPE": "float", "DEFAULT": 90.0, "MIN": -10000.0, "MAX": 10000.0 },
    { "NAME": "frequency", "LABEL": "Frequency", "TYPE": "float", "DEFAULT": 0.008, "MIN": 0.0, "MAX": 10.0 },
    { "NAME": "spot", "LABEL": "Spot", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "orbit", "LABEL": "Orbit", "TYPE": "float", "DEFAULT": 10.0, "MIN": 0.1, "MAX": 600.0 },
    { "NAME": "ease", "LABEL": "Ease", "TYPE": "long", "DEFAULT": 3, "LABELS": ["Linear", "In", "Out", "In Out", "Back", "Elastic"] }
  ]
}*/
import package::processing::noise;
import package::cavalry::{ cv_circle, cv_falloff, cv_noise };

// Duplicator(Path: circle) + Noise(Deformer) + Falloff(Circle, moving)。
fn block(k: u32, p: BlockParams) -> Offset {
    let n = max(host.members, 1u);
    let u = f32(k) / f32(n);
    let on_ring = cv_circle(k, n, p.radius, -90.0);
    // 落ちる所: 環の上を Orbit 秒で 1 周する点。Spot は環の何割に効くか(角度の距離で Falloff)。
    let spot_u = fract(host.time / max(p.orbit, 0.1));
    var d = abs(u - spot_u);
    d = min(d, 1.0 - d);
    let w = cv_falloff(d, max(p.spot, 1e-3) * 0.5, u32(p.ease));
    // 法線方向へ noise だけ押す(環の外へ / 内へ)。
    let normal = normalize(on_ring);
    let noise = cv_noise(on_ring, p.frequency, host.time * 0.6);
    return Offset(on_ring + normal * noise * p.amount * w, 0.0, 1.0 + 0.6 * w, vec4f(1.0));
}
