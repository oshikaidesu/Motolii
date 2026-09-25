//! Regression gates for the render wrap-up (2026-09-25): 300 Jewel Repeater, Glass Garden and an
//! ordinary 2D + 3D poster. Structure is counted (runs, batches, uploads, preparations), never timed;
//! wall time lives in the `#[ignore]`d benchmarks at the bottom with the numbers they were set from.
use crate::doc::store::RationalTime;

fn here() -> std::path::PathBuf { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/editor/script") }

fn open(source: &str) -> crate::EditorRuntime { open_as(source, "gate.js") }

/// `name` is the script's path when it names media beside itself.
fn open_as(source: &str, name: &str) -> crate::EditorRuntime {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    rt.run_script(source, name).unwrap_or_else(|message| panic!("{message}"));
    rt
}

fn engine() -> crate::render::engine::Engine {
    let mut engine = crate::render::engine::Engine::new().unwrap();
    engine.set_realtime(true);
    engine
}

fn at(rt: &crate::EditorRuntime, frame: i64) -> RationalTime {
    let comp = rt.doc.view().composition().unwrap().unwrap();
    RationalTime::try_from_frame(frame, comp.fps).unwrap()
}

/// One jewel field: `count` copies of one glass mesh by one Repeater, the 1080p 60 fps piece U79 timed.
fn jewels(count: usize) -> String {
    let (cols, dx, dy) = (count.clamp(1, 20), 90.0, 70.0);
    let rows = count.div_ceil(cols);
    let (x0, y0) = (960.0 - dx * (cols as f64 - 1.0) / 2.0, 540.0 - dy * (rows as f64 - 1.0) / 2.0);
    format!(r##"
comp({{ width: 1920, height: 1080, fps: 60, seconds: 4, background: "#000000" }});
const gem = media("{mesh}/brilliant.obj", {{ name: "Jewel" }}).set("Scale", [34, 34]).set("Tilt X", -58);
gem.effect("Glass", {{ "Transmission": 1, "Refraction": 2.42, "Dispersion": 0.36 }});
gem.keys("Tilt Y", [[0, -40, "Linear"], [4, 320]]);
gem.effect("Repeater", {{ "Count": {count}, "Along": 2, "Columns": {cols}, "Position X Each": {dx}, "Position Y Each": {dy}, "Rotation Random": 360, "Scale Random": 25, "Position Z Random": 80, "Seed Random": 7 }});
gem.set("Position", [{x0}, {y0}]);
gem.keys("Rotation", [[0, 0, "Linear"], [4, 8]]);
"##, mesh = here().join("gates/mesh").display())
}


/// The ordinary 2D + 3D poster U78/U79 judged by eye: flat type and logo shapes, extruded type,
/// chrome, plastic, emissive, and a row of glass, crystal and diamond, on a chosen background.
fn poster(background: &str) -> String {
    format!(r##"
comp({{ width: 1920, height: 1080, fps: 60, seconds: 4, background: "{background}" }});
const spin = (layer, a, b) => layer.keys("Tilt Y", [[0, a, "sine.inOut"], [4, b]]);
text("WHITE", {{ name: "White Text" }}).fill("#FFFFFF").set("Position", [330, 170]).set("Size", 130).projection("2D");
text("BLACK", {{ name: "Black Text" }}).fill("#101010").set("Position", [930, 170]).set("Size", 130).projection("2D");
ellipse({{ name: "Logo C" }}).fill("#00A6E0").set("Position", [1500, 150]).set("Scale", [1.1, 1.1]).projection("2D");
ellipse({{ name: "Logo M" }}).fill("#E6007EC0").set("Position", [1580, 190]).set("Scale", [1.1, 1.1]).projection("2D");
star({{ name: "Logo Y" }}).fill("#FFE100C0").set("Position", [1660, 150]).set("Scale", [0.9, 0.9]).projection("2D");
const word = text("TYPE", {{ name: "Extruded Text" }}).fill("#F4F1EA").set("Position", [330, 540]).set("Size", 170);
word.effect("Extrude", {{ "Depth": 50 }}); word.effect("Bevel", {{ "Radius": 6 }}); word.set("Tilt X", 10); spin(word, -14, 10);
const chrome = media("{orb}", {{ name: "Chrome" }}).set("Position", [830, 540]).set("Scale", [135, 135]);
chrome.effect("Glass", {{ "Refraction": 1.5, "Roughness": 0.06, "Transmission": 0, "Metallic": 1 }});
const puck = ellipse({{ name: "Plastic" }}).fill("#E0452B").set("Position", [1230, 540]).set("Scale", [1.1, 1.1]);
puck.effect("Extrude", {{ "Depth": 50 }}); puck.effect("Bevel", {{ "Radius": 20 }});
puck.effect("Glass", {{ "Refraction": 1.5, "Roughness": 0.4, "Transmission": 0, "Metallic": 0 }}); puck.set("Tilt X", 38); spin(puck, -22, 10);
const glow = text("NEON", {{ name: "Emissive" }}).fill("#FF3FA4").set("Position", [1650, 540]).set("Size", 150);
glow.effect("Extrude", {{ "Depth": 20 }}); glow.effect("Glass", {{ "Transmission": 0, "Roughness": 0.3, "Emission": 3 }}); spin(glow, -10, 10);
const clear = (layer, v = {{}}) => layer.effect("Glass", {{ "Transmission": 1, ...v }});
const sphere = media("{orb}", {{ name: "Glass Sphere" }}).set("Position", [400, 880]).set("Scale", [135, 135]); clear(sphere);
const crystal = media("{mesh}/crystal.obj", {{ name: "Crystal" }}).set("Position", [960, 880]).set("Scale", [170, 170]);
clear(crystal, {{ "Refraction": 1.54, "Dispersion": 0.3 }}); crystal.set("Tilt X", 12); spin(crystal, -25, 25);
const gem = media("{mesh}/brilliant.obj", {{ name: "Diamond" }}).set("Position", [1520, 880]).set("Scale", [170, 170]);
clear(gem, {{ "Refraction": 2.42, "Dispersion": 0.36 }}); gem.set("Tilt X", -58); spin(gem, -20, 20);
"##, orb = here().join("examples/glass_garden/orb.obj").display(), mesh = here().join("gates/mesh").display())
}

/// Where each object of the poster sits: name, centre and half extent in composition pixels.
const CELLS: [(&str, i32, i32, i32, i32); 11] = [
    ("white text", 330, 170, 170, 90), ("black text", 930, 170, 170, 90), ("logos", 1580, 170, 130, 90),
    ("extruded text", 330, 540, 170, 110), ("chrome", 830, 540, 130, 130), ("plastic", 1230, 540, 130, 130), ("emissive", 1650, 540, 170, 110),
    ("sphere", 400, 880, 130, 130), ("crystal", 960, 880, 150, 150), ("diamond", 1520, 880, 150, 150), ("row", 960, 880, 960, 150),
];

const FIELD: [(&str, i32, i32, i32, i32); 1] = [("field", 960, 540, 960, 540)];

/// Per cell: the share of pixels that are not the background, and the mean colour.
fn signature(pixels: &[u8], width: usize, background: [u8; 3], cells: &[(&str, i32, i32, i32, i32)]) -> Vec<(f32, [f32; 3])> {
    cells.iter().map(|&(_, cx, cy, hx, hy)| {
        let (mut lit, mut total, mut sum) = (0u32, 0u32, [0f64; 3]);
        for y in (cy - hy).max(0)..(cy + hy) {
            for x in (cx - hx).max(0)..(cx + hx) {
                let p = &pixels[(y as usize * width + x as usize) * 4..][..4];
                total += 1;
                for k in 0..3 { sum[k] += p[k] as f64; }
                if (0..3).map(|k| (p[k] as i32 - background[k] as i32).abs()).sum::<i32>() > 24 { lit += 1; }
            }
        }
        (lit as f32 / total as f32, [0, 1, 2].map(|k| (sum[k] / total as f64) as f32))
    }).collect()
}


fn garden() -> String { std::fs::read_to_string(here().join("examples/glass_garden.js")).unwrap() }


/// Glass Garden's ring, alone: one glass petal repeated round a centre as a whole (a plate), then a Glow.
fn ring(count: usize) -> String {
    format!(r##"
comp({{ width: 1280, height: 720, fps: 60, seconds: 8, background: "#030306" }});
const petal = ellipse({{ name: "Petal" }}).fill("#0B0B16").set("Scale", [0.55, 0.12]).set("Position", [62, 0]);
petal.effect("Extrude", {{ "Depth": 8 }});
petal.effect("Bevel", {{ "Radius": 6 }});
petal.effect("Glass", {{ "Refraction": 1.5, "Roughness": 0.02, "Transmission": 1, "Dispersion": 1.6 }});
const ring = group(petal).name("Ring").set("Position", [640, 360]);
ring.effect("Repeater", {{ "Count": {count}, "Position X Each": 0, "Position Y Each": 0, "Rotation Each": {step} }}).whole();
ring.keys("Rotation", [[0, 0, "Linear"], [8, {step}]]);
ring.effect("Glow", {{ "Threshold": 0.6, "Intensity": 0.55, "Radius": 48 }});
text("STILL", {{ name: "Still Text" }}).fill("#D8DCE8").set("Position", [200, 100]).set("Size", 40).projection("2D");
"##, step = 360.0 / count as f64)
}


/// What one drawn frame cost in structure: the surface work it added and the contributions it prepared.
#[derive(Debug)]
struct Frame { runs: u64, backdrops: u64, batches: u64, instances: u64, upload_bytes: u64, light: u64, views: u64, drawn: usize, tick: crate::render::engine::TickStats, prepared: Vec<String> }

/// Draws frame `at` (the frame before it warmed the caches) and reports what that one frame did.
fn draw(rt: &crate::EditorRuntime, engine: &mut crate::render::engine::Engine, frame: i64) -> Frame {
    let names: std::collections::HashMap<String, String> = {
        let view = rt.doc.view();
        view.layers().iter().map(|l| (format!("layer {}", l.0), view.attrs(*l).unwrap().unwrap().name.clone())).collect()
    };
    let before = engine.surface_work();
    engine.render_frame(&rt.doc.view(), at(rt, frame)).unwrap();
    let after = engine.surface_work();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    Frame {
        runs: after.main_runs - before.main_runs, backdrops: after.backdrop_copies - before.backdrop_copies, batches: after.mesh_batches - before.mesh_batches,
        instances: after.mesh_instances_uploaded - before.mesh_instances_uploaded, upload_bytes: after.mesh_instance_upload_bytes - before.mesh_instance_upload_bytes,
        light: after.light_captures - before.light_captures, views: after.layer_views - before.layer_views,
        drawn: engine.drawn_layers(), tick: engine.tick_stats(),
        prepared: engine.frame_claims().iter().filter(|c| c.stage == "prepare").map(|c| names.get(&c.who).cloned().unwrap_or_else(|| c.who.clone())).collect(),
    }
}

/// A steady frame of `source`: one to warm, the next measured.
fn steady(source: &str, frame: i64) -> Frame {
    let rt = open(source);
    let mut engine = engine();
    engine.render_frame(&rt.doc.view(), at(&rt, frame - 1)).unwrap();
    draw(&rt, &mut engine, frame)
}

const COUNTS: [usize; 5] = [1, 10, 50, 100, 300];

/// One view run, one backdrop copy, one mesh batch and one instance buffer for the whole field, and
/// nothing prepared again: 300 jewels cost the structure of 1 (U79: p50 8.8 ms at 1080p on the M4).
#[test]
fn a_jewel_repeaters_structure_does_not_grow_with_its_count() {
    for count in COUNTS {
        let f = steady(&jewels(count), 2);
        assert_eq!((f.runs, f.backdrops, f.batches, f.light, f.views), (1, 1, 1, 0, 0), "{count} jewels: {f:?}");
        assert_eq!((f.tick.preparations, f.tick.views, f.tick.submits, f.tick.begin_frames), (1, 1, 1, 1), "{count} jewels: {f:?}");
        assert_eq!(f.drawn, count, "every copy is drawn");
        assert_eq!(f.instances, count as u64, "one instance per copy, uploaded once: {f:?}");
        assert_eq!(f.upload_bytes, 168 * count as u64, "a copy is one fixed-size record: {f:?}");
        assert!(f.prepared.is_empty(), "a spinning field prepares nothing again: {f:?}");
    }
}

/// A Repeater with an effect after it is a plate: drawn as one stack whatever its count. Runs, batches
/// and preparations stay flat; only the instance records grow with the copies (each is in the world's
/// shared instances and in the plate's, two meshes each).
#[test]
fn a_repeated_plates_structure_does_not_grow_with_its_count() {
    let flat = steady(&ring(1), 2);
    for count in COUNTS {
        let f = steady(&ring(count), 2);
        assert_eq!((f.runs, f.backdrops, f.batches, f.light, f.views), (flat.runs, flat.backdrops, flat.batches, 0, 0), "{count} petals: {f:?}");
        assert_eq!((f.tick.preparations, f.tick.views, f.tick.submits), (1, 1, 1), "{count} petals: {f:?}");
        assert_eq!(f.prepared, vec!["Ring".to_string()], "the ring's plate is the only preparation: {f:?}");
        assert!(f.instances <= 4 * count as u64 + 2, "instance records grow linearly, at most four per copy: {f:?}");
    }
}

/// A still layer is prepared once: while the ring turns and its wave moves, the text beside it is not
/// lowered, rastered or extruded again (it is in the contribution cache, placed as before).
#[test]
fn a_still_text_is_not_prepared_again_while_a_ring_turns() {
    let rt = open(&ring(8));
    let mut engine = engine();
    engine.render_frame(&rt.doc.view(), at(&rt, 1)).unwrap();
    for frame in 2..8 {
        let f = draw(&rt, &mut engine, frame);
        assert_eq!(f.prepared, vec!["Ring".to_string()], "frame {frame}: {f:?}");
    }
}

/// Glass Garden, the shipped example: per frame it prepares only what changes with time (the two Prism
/// Orbit lines, whose Phase turns, and the two rings, which are plates), never its type, glass disc,
/// slab, ember or riders; and it draws in the runs, backdrop copies and batches it draws in today.
/// Ceilings, not targets: fewer is welcome.
#[test]
fn glass_garden_prepares_only_what_moves_and_keeps_its_structure() {
    let path = here().join("examples/glass_garden.js");
    let rt = open_as(&std::fs::read_to_string(&path).unwrap(), path.to_str().unwrap());
    let mut engine = engine();
    engine.render_frame(&rt.doc.view(), at(&rt, 1)).unwrap();
    for frame in 2..10 {
        let f = draw(&rt, &mut engine, frame);
        let allowed = ["Orbit Stage and Orbit", "Orbit Orbit Front", "Outer Bloom", "Inner Bloom"];
        for name in &f.prepared {
            assert!(allowed.contains(&name.as_str()), "frame {frame}: {name} was prepared again ({f:?})");
        }
        assert!(f.prepared.len() <= allowed.len(), "frame {frame}: {f:?}");
        assert!(f.runs <= 8 && f.backdrops <= 3 && f.batches <= 4, "frame {frame}: {f:?}");
        assert!(f.instances <= 210, "frame {frame}: {f:?}");
        assert_eq!((f.light, f.views, f.tick.preparations, f.tick.views, f.tick.submits), (0, 0, 1, 1, 1), "frame {frame}: {f:?}");
    }
}

/// The frame as the poster's cells see it: BGRA to RGB first, since the presentable format is BGRA.
fn seen(source: &str, frame: i64, background: [u8; 3], cells: &[(&str, i32, i32, i32, i32)]) -> (Vec<u8>, Vec<(f32, [f32; 3])>) {
    let rt = open(source);
    let mut engine = engine();
    let pixels = engine.render_frame(&rt.doc.view(), at(&rt, frame)).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    let rgba: Vec<u8> = pixels.chunks_exact(4).flat_map(|p| [p[2], p[1], p[0], p[3]]).collect();
    let cells = signature(&rgba, 1920, background, cells);
    (pixels, cells)
}

/// Poster reference (M4, frame 30 of `poster`, 1080p): per background, for each of `CELLS`, the share
/// of pixels off the background and the mean RGB. Refresh with `MOTOLII_GATES_DUMP=1 cargo test --release
/// -p motolii-ui --lib gates::dump_signatures -- --nocapture` after a look change meant to move it.
const POSTER: [(&str, &str, [u8; 3], [(f32, [f32; 3]); 11]); 4] = [
    ("white", "#FFFFFF", [255; 3], [(0.000, [255.0, 255.0, 255.0]), (0.179, [217.0, 216.0, 216.0]), (0.990, [237.0, 89.0, 83.0]), (0.266, [245.0, 244.0, 241.0]), (0.649, [155.0, 154.0, 154.0]), (0.763, [177.0, 89.0, 69.0]), (0.291, [255.0, 203.0, 229.0]), (0.419, [233.0, 233.0, 234.0]), (0.246, [242.0, 241.0, 241.0]), (0.746, [192.0, 191.0, 192.0]), (0.215, [239.0, 239.0, 239.0])]),
    ("grey", "#808080", [128; 3], [(0.198, [150.0, 150.0, 150.0]), (0.173, [110.0, 110.0, 110.0]), (0.989, [236.0, 88.0, 82.0]), (0.262, [151.0, 150.0, 148.0]), (0.827, [135.0, 134.0, 134.0]), (0.763, [146.0, 59.0, 38.0]), (0.735, [185.0, 114.0, 147.0]), (0.369, [130.0, 130.0, 130.0]), (0.222, [134.0, 133.0, 133.0]), (0.691, [137.0, 136.0, 137.0]), (0.198, [131.0, 131.0, 131.0])]),
    ("red", "#E8342A", [232, 52, 42], [(0.197, [236.0, 88.0, 80.0]), (0.170, [197.0, 46.0, 38.0]), (0.990, [237.0, 87.0, 81.0]), (0.269, [228.0, 94.0, 84.0]), (0.860, [151.0, 122.0, 121.0]), (0.761, [171.0, 41.0, 18.0]), (0.788, [248.0, 60.0, 97.0]), (0.827, [214.0, 76.0, 71.0]), (0.352, [222.0, 71.0, 64.0]), (0.877, [182.0, 109.0, 107.0]), (0.307, [220.0, 68.0, 60.0])]),
    ("black", "#000000", [0; 3], [(0.217, [47.0, 46.0, 47.0]), (0.165, [3.0, 3.0, 3.0]), (0.990, [234.0, 87.0, 81.0]), (0.273, [57.0, 56.0, 54.0]), (0.978, [117.0, 116.0, 116.0]), (0.763, [117.0, 28.0, 8.0]), (1.000, [138.0, 33.0, 85.0]), (0.844, [52.0, 52.0, 53.0]), (0.481, [36.0, 36.0, 36.0]), (0.932, [100.0, 98.0, 99.0]), (0.365, [30.0, 30.0, 30.0])]),
];

/// Whether `got` is the reference within tolerance: the lit share by 0.04, each mean channel by 12 of 255
/// (another GPU's rounding of glass and Look passes moves a cell by less than this; a lost or wrong
/// material moves it by far more). Says which cell and what it drew.
fn matches(name: &str, cells: &[(&str, i32, i32, i32, i32)], got: &[(f32, [f32; 3])], want: &[(f32, [f32; 3])]) -> Result<(), String> {
    for (((cell, ..), got), want) in cells.iter().zip(got).zip(want) {
        let off = (0..3).map(|k| (got.1[k] - want.1[k]).abs()).fold(0.0f32, f32::max);
        if (got.0 - want.0).abs() > 0.04 || off > 12.0 {
            return Err(format!("{name} / {cell}: lit {:.3} mean {:?}, reference lit {:.3} mean {:?}", got.0, got.1, want.0, want.1));
        }
    }
    Ok(())
}

/// The ordinary poster (flat text, SVG-like shapes, extruded text, chrome, plastic, glass, crystal,
/// diamond, emissive) keeps its look on four backgrounds, and the same frame drawn twice is the same
/// bytes.
#[test]
fn the_poster_keeps_its_look_on_white_grey_red_and_black() {
    let mut failures = Vec::new();
    for (name, hex, rgb, want) in POSTER {
        let (pixels, cells) = seen(&poster(hex), 30, rgb, &CELLS);
        if let Err(message) = matches(name, &CELLS, &cells, &want) { failures.push(message); }
        let rt = open(&poster(hex));
        let mut engine = engine();
        let a = engine.render_frame(&rt.doc.view(), at(&rt, 30)).unwrap();
        let b = engine.render_frame(&rt.doc.view(), at(&rt, 30)).unwrap();
        assert!(a == b, "{name}: a frame drawn twice differs");
        assert!(a == pixels, "{name}: a fresh engine draws another picture");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// 300 jewels on black, frame 30: the field fills the frame (U79: nine tenths lit) and stays bright in
/// the middle of the range, not blown out or dark.
#[test]
fn a_field_of_300_jewels_keeps_its_look() {
    let (_, cells) = seen(&jewels(300), 30, [0; 3], &FIELD);
    matches("300 jewels", &FIELD, &cells, &[(0.913, [81.0, 78.0, 79.0])]).unwrap();
}

/// Frames per second in wall time, the window path of `script_frames`: draw into a window-sized target
/// and wait for the GPU, so each sample is the frame's whole CPU + GPU time. Returns p50, p95, worst.
fn timing(source: &str, name: &str, frames: i64) -> [f64; 3] {
    let rt = open_as(source, name);
    let mut engine = engine();
    let comp = rt.doc.view().composition().unwrap().unwrap();
    let target = engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
        label: Some("gate window"), size: wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: crate::render::compositor::PRESENTABLE_FORMAT, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[],
    });
    engine.render_frame(&rt.doc.view(), at(&rt, 0)).unwrap();
    let mut each = Vec::new();
    for frame in 1..=frames {
        let started = std::time::Instant::now();
        engine.render_frame_into(&rt.doc.view(), at(&rt, frame), &target).unwrap();
        engine.gpu_device().poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).unwrap();
        each.push(started.elapsed().as_secs_f64() * 1e3);
    }
    each.sort_by(f64::total_cmp);
    let q = |q: f64| each[((each.len() - 1) as f64 * q).round() as usize];
    [q(0.5), q(0.95), q(1.0)]
}

/// `cargo test --release -p motolii-ui --lib gates::bench -- --ignored --nocapture --test-threads=1` (one at a time:
/// they share the GPU), on the M4, 1080p, 300 frames.
/// The ceilings are regression fences a little above what the frame costs; the budget is one 60 fps frame.
const BUDGET_MS: f64 = 16.7;

/// U79: 300 jewels p50 8.79 / p95 9.23 / worst 17.58 ms (GPU 7.21); 59.49 ms on the U78 path. Re-measured
/// 2026-09-25 on the wrap-up base: 8.43 / 8.93 / 14.65 ms.
#[test]
#[ignore]
fn bench_300_jewels_holds_60fps() {
    let [p50, p95, worst] = timing(&jewels(300), "gate.js", 300);
    eprintln!("300 JEWELS p50 {p50:.2} p95 {p95:.2} worst {worst:.2} ms (budget {BUDGET_MS})");
    assert!(p50 <= BUDGET_MS && p95 <= BUDGET_MS, "300 jewels must hold 60 fps: p50 {p50:.2} p95 {p95:.2}");
}

/// Glass Garden: 27.0 / 41.8 ms in U79, 24.4 / 25.5 / 60.9 ms with the extrusion silhouette carried over
/// (2026-09-25: the ~17 ms CPU frame every 3rd is gone; what is left is the GPU cost of two plates).
/// The budget is not met yet: the fence is the state it was left in, so it cannot get worse unseen.
#[test]
#[ignore]
fn bench_glass_garden() {
    let path = here().join("examples/glass_garden.js");
    let [p50, p95, worst] = timing(&std::fs::read_to_string(&path).unwrap(), path.to_str().unwrap(), 300);
    eprintln!("GLASS GARDEN p50 {p50:.2} p95 {p95:.2} worst {worst:.2} ms (budget {BUDGET_MS}{})", if p50 <= BUDGET_MS && p95 <= BUDGET_MS { ", met" } else { ", NOT met" });
    assert!(p50 <= 28.0 && p95 <= 30.0, "Glass Garden got slower than it was left: p50 {p50:.2} p95 {p95:.2}");
}

/// The poster on black at 1080p, for a cost that is not a Repeater: 8.86 / 9.89 / 25.11 ms (2026-09-25).
#[test]
#[ignore]
fn bench_poster() {
    let [p50, p95, worst] = timing(&poster("#000000"), "gate.js", 300);
    eprintln!("POSTER p50 {p50:.2} p95 {p95:.2} worst {worst:.2} ms (budget {BUDGET_MS})");
}

#[test]
fn dump_signatures() {
    if std::env::var("MOTOLII_GATES_DUMP").is_err() { return; }
    for (name, hex, rgb) in [("white", "#FFFFFF", [255u8; 3]), ("grey", "#808080", [128; 3]), ("red", "#E8342A", [232, 52, 42]), ("black", "#000000", [0; 3])] {
        let (_, cells) = seen(&poster(hex), 30, rgb, &CELLS);
        eprintln!("(\"{name}\", [{}]),", cells.iter().map(|(s, m)| format!("({s:.3}, [{:.0}.0, {:.0}.0, {:.0}.0])", m[0], m[1], m[2])).collect::<Vec<_>>().join(", "));
    }
    let (_, cells) = seen(&jewels(300), 30, [0; 3], &FIELD);
    eprintln!("jewels {cells:?}");
}
