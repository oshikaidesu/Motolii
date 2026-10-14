// The same four scenes as jolt_bench.cpp, in Rapier: a pile of thin pieces, a grid of joints, a rope, and the dancing
// cat's outline as walls with balls knocked about it. Each is run twice: the time a step takes, and whether the two
// runs end the same to the bit (a hash of where every moving body is).
//   cargo run --release -- <cat outline file>
use rapier3d::prelude::*;
use std::time::Instant;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 40) as f32) / 16777216.0
    }
}

fn hash(world: &PhysicsWorld) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for (_, b) in world.bodies.iter() {
        if !b.is_dynamic() {
            continue;
        }
        let t = b.translation();
        let v = b.linvel();
        for f in [t.x, t.y, t.z, v.x, v.y, v.z] {
            for byte in f.to_bits().to_le_bytes() {
                h = (h ^ byte as u64).wrapping_mul(1099511628211);
            }
        }
    }
    h
}

fn energy(world: &PhysicsWorld) -> f32 {
    world.bodies.iter().filter(|(_, b)| b.is_dynamic()).map(|(_, b)| b.linvel().length_squared()).sum::<f32>()
}

fn pile(world: &mut PhysicsWorld) -> usize {
    world.insert_collider(ColliderBuilder::cuboid(20.0, 0.5, 5.0).translation(Vector::new(0.0, -0.5, 0.0)), None);
    for x in [-10.5f32, 10.5] {
        world.insert_collider(ColliderBuilder::cuboid(0.5, 10.0, 5.0).translation(Vector::new(x, 10.0, 0.0)), None);
    }
    for z in [-0.6f32, 0.6] {
        world.insert_collider(ColliderBuilder::cuboid(20.0, 10.0, 0.5).translation(Vector::new(0.0, 10.0, z + z.signum() * 0.5)), None);
    }
    let mut r = Lcg(7);
    for i in 0..200 {
        let (hx, hy) = (0.1 + 0.25 * r.next(), 0.1 + 0.25 * r.next());
        let p = Vector::new(-8.0 + 0.8 * (i % 20) as f32 + 0.1 * r.next(), 1.0 + 0.8 * (i / 20) as f32, 0.0);
        world.insert(RigidBodyBuilder::dynamic().translation(p), ColliderBuilder::cuboid(hx, hy, 0.08).restitution(0.2).friction(0.6));
    }
    600
}

fn grid(world: &mut PhysicsWorld) -> usize {
    let n = 20usize;
    let mut bodies = vec![RigidBodyHandle::invalid(); n * n];
    for k in 0..n {
        for i in 0..n {
            let body = if i == 0 { RigidBodyBuilder::fixed() } else { RigidBodyBuilder::dynamic().can_sleep(false) }.translation(Vector::new(k as f32, -(i as f32), 0.0));
            let h = world.insert_body(body);
            world.insert_collider(ColliderBuilder::ball(0.4), Some(h));
            if i > 0 {
                world.insert_impulse_joint(bodies[k * n + i - 1], h, SphericalJointBuilder::new().local_anchor1(Vector::new(0.0, -0.5, 0.0)).local_anchor2(Vector::new(0.0, 0.5, 0.0)));
            }
            if k > 0 {
                world.insert_impulse_joint(bodies[(k - 1) * n + i], h, SphericalJointBuilder::new().local_anchor1(Vector::new(0.5, 0.0, 0.0)).local_anchor2(Vector::new(-0.5, 0.0, 0.0)));
            }
            bodies[k * n + i] = h;
        }
    }
    600
}

fn rope(world: &mut PhysicsWorld) -> usize {
    let mut last = RigidBodyHandle::invalid();
    for i in 0..100 {
        let body = if i == 0 { RigidBodyBuilder::fixed() } else { RigidBodyBuilder::dynamic() }.translation(Vector::new(0.3 * i as f32, 10.0, 0.0));
        let h = world.insert_body(body);
        world.insert_collider(ColliderBuilder::capsule_x(0.1, 0.05).density(2.0), Some(h));
        if i > 0 {
            world.insert_impulse_joint(last, h, SphericalJointBuilder::new().local_anchor1(Vector::new(0.15, 0.0, 0.0)).local_anchor2(Vector::new(-0.15, 0.0, 0.0)));
        }
        last = h;
    }
    600
}

// The cat: each picture of its film an outline (bake.py: shares of the film, top left 0 0), in metres 100 points each
// of a 1920 x 1080 picture, y up; each edge a thin wall reaching 0.3 towards the eye and away.
struct Cat {
    fps: f64,
    pictures: Vec<Vec<Vec<(f32, f32)>>>,
}
fn read_cat(path: &str) -> Cat {
    let text = std::fs::read_to_string(path).expect("outline");
    let mut lines = text.lines();
    let fps: f64 = lines.next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
    let pictures = lines
        .map(|l| {
            let rest = l.splitn(2, ' ').nth(1).unwrap_or("");
            rest.split('|')
                .map(|ring| {
                    let v: Vec<f32> = ring.split_whitespace().map(|s| s.parse().unwrap()).collect();
                    v.chunks(2).map(|c| (c[0] * 19.2, (1.0 - c[1]) * 10.8)).collect::<Vec<_>>()
                })
                .filter(|r: &Vec<(f32, f32)>| r.len() >= 3)
                .collect()
        })
        .collect();
    Cat { fps, pictures }
}
fn inside(ring: &[(f32, f32)], p: (f32, f32)) -> bool {
    let mut c = false;
    let mut j = ring.len() - 1;
    for i in 0..ring.len() {
        let (a, b) = (ring[i], ring[j]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            c = !c;
        }
        j = i;
    }
    c
}

fn run_cat(world: &mut PhysicsWorld, cat: &Cat, times: &mut Vec<f64>) {
    let (w, h) = (19.2f32, 10.8f32);
    let mut r = Lcg(7);
    let mut balls = vec![];
    for i in 0..40 {
        let rad = 0.22 + 0.28 * r.next();
        let p = Vector::new(w * (0.3 + 0.4 * r.next()), h + 0.7 * i as f32, -0.2 + 0.4 * r.next());
        let (b, _) = world.insert(RigidBodyBuilder::dynamic().translation(p), ColliderBuilder::ball(rad).restitution(0.7).friction(0.3));
        balls.push((b, rad));
    }
    let steps = (cat.pictures.len() as f64 / cat.fps * 60.0) as usize;
    let mut walls: Vec<ColliderHandle> = vec![];
    let mut showing = usize::MAX;
    for k in 0..steps {
        let t0 = Instant::now();
        let now = ((k as f64 / 60.0 * cat.fps) as usize).min(cat.pictures.len() - 1);
        if now != showing {
            showing = now;
            for c in walls.drain(..) {
                world.remove_collider(c);
            }
            for ring in &cat.pictures[now] {
                for a in 0..ring.len() {
                    let (p, q) = (ring[a], ring[(a + 1) % ring.len()]);
                    let (dx, dy) = (q.0 - p.0, q.1 - p.1);
                    let len = (dx * dx + dy * dy).sqrt().max(1e-3);
                    let wall = ColliderBuilder::cuboid(0.5 * len, 0.03, 0.3).translation(Vector::new(0.5 * (p.0 + q.0), 0.5 * (p.1 + q.1), 0.0)).rotation(Vector::new(0.0, 0.0, dy.atan2(dx))).restitution(0.8);
                    walls.push(world.insert_collider(wall, None));
                }
            }
            // (a ball the cat has grown over, within its thickness, is put out at its nearest edge and sent off that way)
            for &(b, rad) in &balls {
                let t = world.bodies[b].translation();
                if t.z.abs() > 0.3 {
                    continue;
                }
                for ring in &cat.pictures[now] {
                    if !inside(ring, (t.x, t.y)) {
                        continue;
                    }
                    let mut best = (f32::MAX, (t.x, t.y), (0.0f32, 1.0f32));
                    for a in 0..ring.len() {
                        let (p, q) = (ring[a], ring[(a + 1) % ring.len()]);
                        let d = (q.0 - p.0, q.1 - p.1);
                        let s = (((t.x - p.0) * d.0 + (t.y - p.1) * d.1) / (d.0 * d.0 + d.1 * d.1).max(1e-6)).clamp(0.0, 1.0);
                        let at = (p.0 + s * d.0, p.1 + s * d.1);
                        let dist = ((t.x - at.0).powi(2) + (t.y - at.1).powi(2)).sqrt();
                        if dist < best.0 {
                            let n = (d.1, -d.0);
                            let l = (n.0 * n.0 + n.1 * n.1).sqrt().max(1e-6);
                            best = (dist, at, (n.0 / l, n.1 / l));
                        }
                    }
                    // (which way is out: the side of the edge away from the outline's middle of the box)
                    let mut n = best.2;
                    if inside(ring, (best.1 .0 + n.0 * 0.05, best.1 .1 + n.1 * 0.05)) {
                        n = (-n.0, -n.1);
                    }
                    let body = &mut world.bodies[b];
                    let v = body.linvel();
                    body.set_translation(Vector::new(best.1 .0 + n.0 * (rad + 0.02), best.1 .1 + n.1 * (rad + 0.02), t.z), true);
                    body.set_linvel(Vector::new(v.x + 9.0 * n.0, v.y + 9.0 * n.1, v.z), true);
                    break;
                }
            }
        }
        if k % 2 == 0 {
            for &(b, _) in &balls {
                let t = world.bodies[b].translation();
                if t.y < -1.0 || t.x < -1.0 || t.x > w + 1.0 {
                    let p = Vector::new(w * (0.3 + 0.4 * r.next()), h + 0.5 + 2.0 * r.next(), -0.2 + 0.4 * r.next());
                    let body = &mut world.bodies[b];
                    body.set_translation(p, true);
                    body.set_linvel(Vector::new(0.0, -2.0, 0.0), true);
                    body.set_angvel(Vector::ZERO, true);
                }
            }
        }
        world.step();
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
}

fn top_speed(world: &PhysicsWorld) -> f32 {
    world.bodies.iter().filter(|(_, b)| b.is_dynamic()).map(|(_, b)| b.linvel().length()).fold(0.0, f32::max)
}

// Overlaps, as a hand makes them: fifty pieces born in one place, and twenty pieces of a settled pile thrown into
// its middle at step 300. Returned: the fastest any piece goes in the two seconds after, and how long till all are
// slower than 0.5 a second (steps).
fn overlap(world: &mut PhysicsWorld, born: bool, times: &mut Vec<f64>) -> (f32, usize) {
    if born {
        world.insert_collider(ColliderBuilder::cuboid(20.0, 0.5, 5.0).translation(Vector::new(0.0, -0.5, 0.0)), None);
        let mut r = Lcg(11);
        for _ in 0..50 {
            world.insert(RigidBodyBuilder::dynamic().translation(Vector::new(0.01 * r.next(), 1.0, 0.0)), ColliderBuilder::cuboid(0.3, 0.3, 0.08));
        }
    } else {
        pile(world);
        for _ in 0..300 {
            world.step();
        }
        let moving: Vec<RigidBodyHandle> = world.bodies.iter().filter(|(_, b)| b.is_dynamic()).map(|(h, _)| h).take(20).collect();
        let mut r = Lcg(13);
        for h in moving {
            world.bodies[h].set_translation(Vector::new(-0.5 + r.next(), 0.6 + 0.5 * r.next(), 0.0), true);
        }
    }
    let (mut most, mut calm) = (0.0f32, 0usize);
    for k in 0..240 {
        let t0 = Instant::now();
        world.step();
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
        let v = top_speed(world);
        if k < 120 {
            most = most.max(v);
        }
        if v > 0.5 {
            calm = k + 1;
        }
    }
    // (still one in another: pairs whose middles are nearer than 0.15; and how far apart the first and last are)
    let at: Vec<Vector> = world.bodies.iter().filter(|(_, b)| b.is_dynamic()).map(|(_, b)| b.translation()).collect();
    let mut stacked = 0;
    for i in 0..at.len() {
        for j in i + 1..at.len() {
            if (at[i] - at[j]).length() < 0.15 {
                stacked += 1;
            }
        }
    }
    let (lo, hi) = at.iter().fold((f32::MAX, f32::MIN), |(l, h), p| (l.min(p.x), h.max(p.x)));
    println!("   (after 4 s: {stacked} pairs still one in another, spread {:.1} m)", hi - lo);
    (most, calm)
}

fn main() {
    for limit in [None, Some(1.0f32)] {
        for born in [true, false] {
            let mut world = PhysicsWorld::new();
            world.gravity = Vector::new(0.0, -9.81, 0.0);
            if let Some(l) = limit {
                world.integration_parameters.normalized_max_corrective_velocity = l;
            }
            let mut times = vec![];
            let (most, calm) = overlap(&mut world, born, &mut times);
            println!("rapier overlap {} (push-out limit {}): fastest {most:.1} m/s, calm after {calm} steps", if born { "born in one place" } else { "thrown into a pile" }, limit.map_or("as it comes".to_string(), |l| format!("{l} m/s")));
        }
    }
    let cat = read_cat(&std::env::args().nth(1).expect("cat outline file"));
    for name in ["pile", "grid", "rope", "cat"] {
        let mut hashes = vec![];
        let mut report = String::new();
        for run in 0..2 {
            let mut world = PhysicsWorld::new();
            world.gravity = Vector::new(0.0, -9.81, 0.0);
            let mut times = vec![];
            if name == "cat" {
                world.gravity = Vector::new(0.0, -14.0, 0.0);
                run_cat(&mut world, &cat, &mut times);
            } else {
                let steps = match name {
                    "pile" => pile(&mut world),
                    "grid" => grid(&mut world),
                    _ => rope(&mut world),
                };
                for _ in 0..steps {
                    let t0 = Instant::now();
                    world.step();
                    times.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
            }
            hashes.push(hash(&world));
            if run == 1 {
                let mean = times.iter().sum::<f64>() / times.len() as f64;
                let most = times.iter().cloned().fold(0.0, f64::max);
                report = format!("rapier {name}: {} steps, {mean:.3} ms a step, most {most:.3} ms, end energy {:.3}", times.len(), energy(&world));
            }
        }
        println!("{report}, the same twice: {}", if hashes[0] == hashes[1] { "yes" } else { "NO" });
    }
}
