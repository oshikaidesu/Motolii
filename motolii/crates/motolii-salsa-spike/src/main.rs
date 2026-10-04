//! Spike: Document → GpuExecutionPlan through salsa 0.28.5.
//! Submit, decode, and textures are not queries. A memo hit still presents.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use salsa::{Database as _, Setter};

const LAYERS: i32 = 1000;
const FRAMES: i32 = 30;

#[salsa::db]
trait Db: salsa::Database {
    fn counts(&self) -> &Counts;
}

#[salsa::db]
#[derive(Default)]
struct Database {
    storage: salsa::Storage<Self>,
    counts: Counts,
}

#[salsa::db]
impl salsa::Database for Database {}

#[salsa::db]
impl Db for Database {
    fn counts(&self) -> &Counts {
        &self.counts
    }
}

#[derive(Default)]
struct Counts {
    transform: AtomicU64,
    appearance: AtomicU64,
    layer: AtomicU64,
    composition: AtomicU64,
    plan: AtomicU64,
}

#[derive(Clone, Copy, Debug)]
struct Runs {
    transform: u64,
    appearance: u64,
    layer: u64,
    composition: u64,
    plan: u64,
}

impl Counts {
    fn take(&self) -> Runs {
        Runs {
            transform: self.transform.swap(0, Ordering::Relaxed),
            appearance: self.appearance.swap(0, Ordering::Relaxed),
            layer: self.layer.swap(0, Ordering::Relaxed),
            composition: self.composition.swap(0, Ordering::Relaxed),
            plan: self.plan.swap(0, Ordering::Relaxed),
        }
    }
}

#[salsa::input]
struct Layer {
    #[returns(copy)]
    x: i32,
    #[returns(copy)]
    y: i32,
    #[returns(copy)]
    opacity: u16,
    #[returns(copy)]
    parent: Option<Layer>,
}

#[salsa::input]
struct Comp {
    #[returns(ref)]
    layers: Vec<Layer>,
    #[returns(copy)]
    camera_x: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Xform {
    x: i32,
    y: i32,
    cycled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Look {
    opacity: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayerEval {
    x: i32,
    y: i32,
    opacity: u16,
    cycled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GpuExecutionPlan {
    camera_x: i32,
    draws: Vec<Draw>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Draw {
    x: i32,
    y: i32,
    opacity: u16,
}

fn cycle_xform(_db: &dyn Db, _id: salsa::Id, _layer: Layer, _t: i32) -> Xform {
    Xform { x: 0, y: 0, cycled: true }
}

#[salsa::tracked(returns(copy), cycle_result = cycle_xform)]
fn sample_transform(db: &dyn Db, layer: Layer, t: i32) -> Xform {
    db.counts().transform.fetch_add(1, Ordering::Relaxed);
    let (px, py) = match layer.parent(db) {
        Some(parent) => {
            let baked = sample_transform(db, parent, t);
            (baked.x, baked.y)
        }
        None => (0, 0),
    };
    Xform {
        x: layer.x(db) + px + t,
        y: layer.y(db) + py,
        cycled: false,
    }
}

#[salsa::tracked(returns(copy))]
fn sample_appearance(db: &dyn Db, layer: Layer, _t: i32) -> Look {
    db.counts().appearance.fetch_add(1, Ordering::Relaxed);
    Look { opacity: layer.opacity(db) }
}

#[salsa::tracked(returns(copy))]
fn evaluate_layer(db: &dyn Db, layer: Layer, t: i32) -> LayerEval {
    db.counts().layer.fetch_add(1, Ordering::Relaxed);
    let xform = sample_transform(db, layer, t);
    let look = sample_appearance(db, layer, t);
    LayerEval {
        x: xform.x,
        y: xform.y,
        opacity: look.opacity,
        cycled: xform.cycled,
    }
}

#[salsa::tracked(returns(copy))]
fn evaluate_composition(db: &dyn Db, comp: Comp, t: i32) -> i64 {
    db.counts().composition.fetch_add(1, Ordering::Relaxed);
    let layers = comp.layers(db).clone();
    let mut sum = 0i64;
    for layer in layers {
        sum += evaluate_layer(db, layer, t).x as i64;
    }
    sum
}

#[salsa::tracked(returns(clone))]
fn build_gpu_plan(db: &dyn Db, comp: Comp, t: i32) -> GpuExecutionPlan {
    db.counts().plan.fetch_add(1, Ordering::Relaxed);
    let _ = evaluate_composition(db, comp, t);
    let layers = comp.layers(db).clone();
    let draws = layers
        .into_iter()
        .map(|layer| {
            let evaluated = evaluate_layer(db, layer, t);
            Draw {
                x: evaluated.x,
                y: evaluated.y,
                opacity: evaluated.opacity,
            }
        })
        .collect();
    GpuExecutionPlan { camera_x: comp.camera_x(db), draws }
}

fn present(frames: &mut u64) {
    *frames += 1;
}

struct Scene {
    db: Database,
    comp: Comp,
    layers: Vec<Layer>,
}

fn scene() -> Scene {
    let mut db = Database::default();
    let mut layers = Vec::with_capacity(LAYERS as usize);
    for i in 0..LAYERS {
        let parent = (i > 0 && i % 10 == 0).then(|| layers[(i as usize) - 1]);
        layers.push(Layer::new(&mut db, i, 0, 255, parent));
    }
    let comp = Comp::new(&mut db, layers.clone(), 0);
    Scene { db, comp, layers }
}

fn plan(scene: &Scene, t: i32, frames: &mut u64) -> GpuExecutionPlan {
    let plan = build_gpu_plan(&scene.db, scene.comp, t);
    present(frames);
    plan
}

fn report() {
    let mut scene = scene();
    let mut frames = 0u64;

    let cold_at = Instant::now();
    let cold = plan(&scene, 0, &mut frames);
    let cold_us = cold_at.elapsed().as_micros();
    let cold_runs = scene.db.counts().take();
    assert_eq!(cold.draws.len(), LAYERS as usize);
    assert_eq!(cold.camera_x, 0);

    let hit_at = Instant::now();
    let again = plan(&scene, 0, &mut frames);
    let hit_us = hit_at.elapsed().as_micros();
    let hit_runs = scene.db.counts().take();
    assert_eq!(again, cold);
    assert_eq!(hit_runs.plan, 0, "same t must not re-evaluate");
    assert_eq!(frames, 2, "a memo hit still presents");
    let presented_on_hit = frames;

    let next_at = Instant::now();
    let next = plan(&scene, 1, &mut frames);
    let next_us = next_at.elapsed().as_micros();
    let next_runs = scene.db.counts().take();
    assert_ne!(next.draws[0].x, cold.draws[0].x);
    assert_eq!(next_runs.transform, LAYERS as u64);

    scene.layers[0].set_x(&mut scene.db).to(50);
    let edit_at = Instant::now();
    let edited = plan(&scene, 1, &mut frames);
    let edit_us = edit_at.elapsed().as_micros();
    let edit_runs = scene.db.counts().take();
    assert_eq!(edited.draws[0].x, 50 + 1);
    assert_eq!(edited.draws[1].x, next.draws[1].x);
    assert_eq!(edit_runs.transform, 1, "one layer edit recomputes that layer");
    assert_eq!(edit_runs.appearance, 0, "opacity was not read again");

    let parent = 9;
    let child = 10;
    let before = scene.layers[parent].x(&scene.db);
    scene.layers[parent].set_x(&mut scene.db).to(before + 7);
    let _ = plan(&scene, 1, &mut frames);
    let parent_runs = scene.db.counts().take();
    assert_eq!(parent_runs.transform, 2, "parent and the child that reads it");

    scene.comp.set_camera_x(&mut scene.db).to(4);
    let camera_at = Instant::now();
    let moved = plan(&scene, 1, &mut frames);
    let camera_us = camera_at.elapsed().as_micros();
    let camera_runs = scene.db.counts().take();
    assert_eq!(moved.camera_x, 4);
    assert_eq!(camera_runs.transform, 0, "camera is not a transform input");
    assert_eq!(camera_runs.layer, 0);
    assert_eq!(camera_runs.composition, 0);
    assert_eq!(camera_runs.plan, 1, "the plan reads the camera");

    let cycled = cycle_pair();
    let cancelled = cancel_once();

    println!("layers={LAYERS} frames_in_scrub={FRAMES}");
    println!("cold_us={cold_us} runs={} {} {} {} {}", cold_runs.transform, cold_runs.appearance, cold_runs.layer, cold_runs.composition, cold_runs.plan);
    println!("hit_us={hit_us} runs={} {} {} {} {} presented={presented_on_hit}", hit_runs.transform, hit_runs.appearance, hit_runs.layer, hit_runs.composition, hit_runs.plan);
    println!("next_t_us={next_us} runs={} {} {} {} {}", next_runs.transform, next_runs.appearance, next_runs.layer, next_runs.composition, next_runs.plan);
    println!("edit_one_us={edit_us} runs={} {} {} {} {}", edit_runs.transform, edit_runs.appearance, edit_runs.layer, edit_runs.composition, edit_runs.plan);
    println!("edit_parent_transforms={}", parent_runs.transform);
    println!("camera_us={camera_us} runs={} {} {} {} {}", camera_runs.transform, camera_runs.appearance, camera_runs.layer, camera_runs.composition, camera_runs.plan);
    println!("cycle_recovered={cycled}");
    println!("cancel_caught={cancelled}");
    println!("handwritten_edges_for_parent=0");
    println!("submit_queries=0");
}

fn cycle_pair() -> bool {
    let mut db = Database::default();
    let left = Layer::new(&mut db, 0, 0, 255, None);
    let right = Layer::new(&mut db, 1, 0, 255, Some(left));
    left.set_parent(&mut db).to(Some(right));
    sample_transform(&db, left, 0).cycled && sample_transform(&db, right, 0).cycled
}

fn cancel_once() -> bool {
    let mut scene = scene();
    scene.db.cancellation_token().cancel();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = build_gpu_plan(&scene.db, scene.comp, 3);
    }));
    caught.is_err()
}

fn scrub() {
    let scene = scene();
    let mut frames = 0u64;
    let started = Instant::now();
    let mut sum = 0i64;
    for t in 0..FRAMES {
        let plan = plan(&scene, t, &mut frames);
        sum += plan.draws.iter().map(|draw| draw.x as i64).sum::<i64>();
    }
    let elapsed = started.elapsed();
    println!("scrub frames={frames} sum={sum} us={}", elapsed.as_micros());
}

fn replay() {
    let scene = scene();
    let mut frames = 0u64;
    let _ = plan(&scene, 0, &mut frames);
    let _ = scene.db.counts().take();
    let started = Instant::now();
    let mut sum = 0i64;
    for _ in 0..FRAMES {
        let plan = plan(&scene, 0, &mut frames);
        sum += plan.draws[0].x as i64;
    }
    let runs = scene.db.counts().take();
    let elapsed = started.elapsed();
    assert_eq!(runs.plan, 0);
    println!("replay frames={frames} sum={sum} us={} plan_runs={}", elapsed.as_micros(), runs.plan);
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("scrub") => scrub(),
        Some("replay") => replay(),
        _ => report(),
    }
}
