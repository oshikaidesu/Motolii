import package::processing::noise;
import package::cavalry::{ cv_circle, cv_falloff, cv_noise };

@description("Cavalry's Ring Ting, packaged: things on a ring; a Noise deformer pushes them off the ring only where a Falloff spot is, and the spot walks round. Put everything at one point; the index puts it on the ring")

@label("Radius") @range(1.0, 10000.0)
override radius: f32 = 360.0;
@label("Amount") @range(-10000.0, 10000.0)
override amount: f32 = 90.0;
@label("Frequency") @range(0.0, 10.0)
override frequency: f32 = 0.008;
@label("Spot") @range(0.0, 1.0)
override spot: f32 = 0.25;
@label("Orbit") @range(0.1, 600.0)
override orbit: f32 = 10.0;
@label("Ease") @options("Linear", "In", "Out", "In Out", "Back", "Elastic")
override ease: u32 = 3;

// Duplicator(Path: circle) + Noise(Deformer) + Falloff(Circle, moving)。
fn block(k: u32) -> Offset {
    let n = max(host.members, 1u);
    let u = f32(k) / f32(n);
    let on_ring = cv_circle(k, n, radius, -90.0);
    // 落ちる所: 環の上を Orbit 秒で 1 周する点。Spot は環の何割に効くか(角度の距離で Falloff)。
    let spot_u = fract(host.time / max(orbit, 0.1));
    var d = abs(u - spot_u);
    d = min(d, 1.0 - d);
    let w = cv_falloff(d, max(spot, 1e-3) * 0.5, ease);
    // 法線方向へ noise だけ押す(環の外へ / 内へ)。
    let normal = normalize(on_ring);
    let noise = cv_noise(on_ring, frequency, host.time * 0.6);
    return Offset(on_ring + normal * noise * amount * w, 0.0, 1.0 + 0.6 * w, vec4f(1.0));
}
