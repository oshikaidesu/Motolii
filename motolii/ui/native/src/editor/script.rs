//! スクリプトの口 — JS が窓の操作(port の op)を並べる。名前は窓の名前だけ([スクリプトの口](../../../../docs/reviews/2026-09-14-script-mouth.md))。
use serde_json::json;
use crate::doc::store::LayerId;
use crate::EditorRuntime;
use motolii_script::{Host, Query, DEFAULT_BUDGET};

impl Host for EditorRuntime {
    fn command(&mut self, request: serde_json::Value) -> Result<serde_json::Value, String> {
        self.request(request)?;
        Ok(json!({ "selected": self.viewer.selected().map(|id| id.0) }))
    }

    fn query(&self, query: Query) -> Result<serde_json::Value, String> {
        match query {
            Query::Layer(id) => self.script_layer(LayerId(id)),
            Query::Effects => Ok(json!(crate::render::engine::known_effects().iter().map(|d| json!({ "name": d.label, "pluginId": d.plugin_id })).collect::<Vec<_>>())),
            Query::Assets => Ok(json!(self.doc.view().assets().map_err(|e| e.to_string())?.iter().map(|a| json!({ "id": a.id.get(), "path": a.path_absolute, "name": a.name })).collect::<Vec<_>>())),
            Query::Composition => {
                let comp = self.doc.view().composition().map_err(|e| e.to_string())?.ok_or("No composition")?;
                Ok(json!({ "width": comp.width, "height": comp.height, "fps": comp.fps.as_f64(), "seconds": comp.duration_frames as f64 / comp.fps.as_f64() }))
            }
        }
    }
}

impl EditorRuntime {
    /// `name` は利用者の file 名。例外の行・列はこの名前で返る(LLM が読んで直す)。
    pub(crate) fn run_script(&mut self, source: &str, name: &str) -> Result<(), String> {
        self.run_script_with_budget(source, name, DEFAULT_BUDGET)
    }

    fn run_script_with_budget(&mut self, source: &str, name: &str, budget: std::time::Duration) -> Result<(), String> {
        let head = self.doc.edit_head();
        // A run states its loop from scratch: a script that no longer asks for one plays on.
        let loop_before = self.viewer.clock.loop_range();
        self.viewer.clock.set_loop(None);
        let outcome = self.run_script_inner(source, name, budget);
        if outcome.is_err() {
            self.viewer.clock.set_loop(loop_before);
            // 途中で断られたスクリプトは何も残さない。直して走らせ直す時、前の半端な層が混ざらない。
            while self.doc.edit_head() > head && self.doc.undo() {}
            self.viewer.selected_keys.clear();
        }
        outcome
    }

    fn run_script_inner(&mut self, source: &str, name: &str, budget: std::time::Duration) -> Result<(), String> {
        let outcome = motolii_script::execute(self, source, name, budget);
        let _ = self.request(json!({ "op": "animate", "enabled": false }));
        let _ = self.request(json!({ "op": "seek", "frame": 0 }));
        outcome
    }

    pub(crate) fn run_script_file(&mut self, path: &str) -> Result<(), String> {
        let source = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let before = self.doc.edit_head();
        self.run_script(&source, path)?;
        self.last_script = Some((path.to_owned(), before, self.doc.edit_head()));
        self.last_script_source = Some(source);
        self.watch_script(path);
        Ok(())
    }

    /// Saving the script wakes the window, which asks for `reloadEffects`; that reruns the script
    /// when its source changed. The folder is watched, not the file: editors save by replacing it.
    fn watch_script(&mut self, path: &str) {
        use notify::Watcher;
        let Some(wake) = self.wake.clone() else { return };
        let file = std::path::PathBuf::from(path);
        let (Some(folder), Some(name)) = (file.parent().map(std::path::Path::to_path_buf), file.file_name().map(std::ffi::OsStr::to_owned)) else { return };
        let watcher = notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            let Ok(event) = event else { return };
            let touched = matches!(event.kind, notify::EventKind::Create(_) | notify::EventKind::Modify(_))
                && event.paths.iter().any(|p| p.file_name() == Some(name.as_os_str()));
            if touched { wake(); }
        });
        self.script_watch = watcher.ok().and_then(|mut w| w.watch(&folder, notify::RecursiveMode::NonRecursive).ok().map(|_| w));
    }

    /// Reruns the last script file if its source is no longer what it ran with.
    pub(crate) fn rerun_script_if_saved(&mut self) -> Result<(), String> {
        let Some((path, _, _)) = self.last_script.clone() else { return Ok(()) };
        let Ok(source) = std::fs::read_to_string(&path) else { return Ok(()) };
        if self.last_script_source.as_deref() == Some(source.as_str()) { return Ok(()); }
        self.rerun_script()
    }

    /// 直前のスクリプトの結果を戻して、同じ file を読み直して走らせる(書いて・保存して・見る)。
    /// 走った後に窓で手を入れていたら、その手を消さないように断る。
    pub(crate) fn rerun_script(&mut self) -> Result<(), String> {
        let (path, before, after) = self.last_script.clone().ok_or("Run a script first")?;
        if self.doc.edit_head() != after {
            return Err("The document changed after the script ran. Undo those edits or use Run Script…".into());
        }
        while self.doc.edit_head() > before && self.doc.undo() {}
        self.viewer.selected_keys.clear();
        self.run_script_file(&path)
    }

    /// 窓の Inspector に渡す行と同じ物(名前・id・値・効果)。
    fn script_layer(&self, id: LayerId) -> Result<serde_json::Value, String> {
        let view = self.doc.view();
        let comp = view.composition().map_err(|e| e.to_string())?.ok_or("No composition")?;
        let catalog = crate::render::engine::known_effects();
        let at = self.time()?;
        let row = self.layer_json(&view, id, at, comp.fps, &catalog, &Default::default(), false)?.ok_or_else(|| format!("Layer {} no longer exists", id.0))?;
        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::store::*;

    fn run(source: &str) -> (crate::EditorRuntime, Result<(), String>) {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        let outcome = rt.run_script(source, "test.js");
        (rt, outcome)
    }

    #[test]
    fn a_script_builds_layers_keys_and_effects_with_window_names() {
        let (rt, outcome) = run(r##"
            comp({ seconds: 4, background: "#101018" });
            const title = text("HELLO", { name: "Title" });
            title.set("Position", [960, 540]).key("Opacity", 0, 0, "Bezier").key("Opacity", 1, 1);
            title.effect("Glow");
            const box = rectangle().key("Rotation", 0, 0, "Elastic").key("Rotation", 2, 180);
            box.parent(title);
        "##);
        outcome.unwrap();
        let view = rt.doc.view();
        let comp = view.composition().unwrap().unwrap();
        assert_eq!(comp.duration_frames, 4 * 30);
        let layers = view.layers();
        let title = *layers.iter().find(|l| view.attrs(**l).unwrap().unwrap().name == "Title").unwrap();
        let opacity = view.track(title, &PropertyId::new(property::OPACITY).unwrap()).unwrap().unwrap();
        assert_eq!(opacity.keys().len(), 2);
        assert!(matches!(opacity.keys()[0].interp, Interp::Bezier { .. }));
        assert_eq!(opacity.keys()[1].t, RationalTime::try_from_frame(30, comp.fps).unwrap());
        assert_eq!(view.effects(title).unwrap().len(), 1);
        let child = *layers.iter().find(|l| view.attrs(**l).unwrap().unwrap().parent == Some(title)).unwrap();
        assert!(view.track(child, &PropertyId::new(property::ROTATION).unwrap()).unwrap().is_some());
    }

    /// 選択肢は名前で、対の片方(Repeater の Position X)は片方だけ書ける。窓で見える値と同じになる。
    #[test]
    fn choices_are_written_by_name_and_a_pair_by_its_half() {
        let (_, outcome) = run(r#"
            const r = ellipse().effect("Repeater", { Along: "Circle", "Position Y Each": 40 }).set("Position X Each", 25);
            const value = (name) => r.rows().find((p) => p.label === name).value;
            if (value("Along") !== 1) throw new Error("Along " + value("Along"));
            if (JSON.stringify(value("Position X Each")) !== "[25,40]") throw new Error("pair " + JSON.stringify(value("Position X Each")));
            let refused = "";
            try { r.set("Along", "Spiral"); } catch (e) { refused = e.message; }
            if (!refused.includes("Line, Circle, Grid")) throw new Error("choices " + refused);
        "#);
        outcome.unwrap();
    }

    /// 窓に無い名前・p5 の古い機構は、行き先を言って断る。
    #[test]
    fn names_the_window_does_not_show_are_refused_with_the_names_it_does() {
        let (rt, outcome) = run("rectangle(); text('x').set('Fill Opacity', 1)");
        let message = outcome.unwrap_err();
        assert!(rt.doc.view().layers().is_empty(), "a refused script leaves nothing behind");
        assert!(message.contains("has no \"Fill Opacity\"") && message.contains("Opacity") && message.contains("test.js:1"), "{message}");
        let (_, outcome) = run("Math.random()");
        assert!(outcome.unwrap_err().contains("random(seed)"));
        let (_, outcome) = run("rectangle().effect('Bloom Deluxe')");
        assert!(outcome.unwrap_err().contains("No effect named"));
    }

    /// GSAP の ease は文字列で(`power2.out` / `back.out(1.7)`)、既存の型(Bezier / Elastic / Bounce / Steps)に写って鍵に載る。知らない名前は両方の名前を並べて断る。
    #[test]
    fn gsap_ease_strings_become_gsap_keys() {
        let (rt, outcome) = run(r#"
            const box = rectangle().keys("Opacity", [[0, 0, "power2.out"], [1, 1, "back.out(1.7)"], [2, 0, "elastic.out(1, 0.3)"], [3, 1, "steps(5)"], [4, 0, "none"], [5, 1]]);
        "#);
        outcome.unwrap();
        let view = rt.doc.view();
        let layer = view.layers()[0];
        let track = view.track(layer, &PropertyId::new(property::OPACITY).unwrap()).unwrap().unwrap();
        let kinds: Vec<&str> = track.keys().iter().map(|k| k.interp.kind()).collect();
        assert_eq!(kinds, ["Bezier", "Bezier", "Elastic", "Steps", "Linear", "Linear"]);
        let fps = view.composition().unwrap().unwrap().fps;
        let at = |f: i64| view.value_at(layer, &PropertyId::new(property::OPACITY).unwrap(), RationalTime::try_from_frame(f, fps).unwrap()).unwrap().unwrap();
        assert!(matches!(at(15), Value::F64(v) if (v - 0.875).abs() < 1e-6), "power2.out at 0.5 = 1 − (1 − .5)³ (exact Bezier)");
        let (_, outcome) = run(r#"rectangle().key("Opacity", 0, 0, "swing")"#);
        let message = outcome.unwrap_err();
        assert!(message.contains("Bezier") && message.contains("power1"), "{message}");
    }

    /// 拍の表で層を背中合わせに並べる。1 つの層は 1 つの枠にだけ。
    #[test]
    fn cuts_lay_layers_back_to_back_on_the_beat() {
        let (rt, outcome) = run(r#"
            comp({ fps: 30, seconds: 4 });
            const a = rectangle({ name: "a" }), b = ellipse({ name: "b" }), c = star({ name: "c" });
            const total = cuts(120, [[a, 1], [[b, c], 2]]);
            if (total !== 1.5) throw new Error("total " + total);
            let refused = "";
            try { cuts(120, [[a, 1], [a, 1]]); } catch (e) { refused = e.message; }
            if (!refused.includes("two cuts")) throw new Error(refused);
        "#);
        outcome.unwrap();
        let view = rt.doc.view();
        let timing = |name: &str| {
            let id = *view.layers().iter().find(|l| view.attrs(**l).unwrap().unwrap().name == name).unwrap();
            let t = view.meta(id).unwrap().unwrap().timing;
            (t.start, t.duration)
        };
        assert_eq!(timing("a"), (0, 15), "one beat at 120 bpm is half a second");
        assert_eq!(timing("b"), (15, 30));
        assert_eq!(timing("c"), (15, 30));
    }

    /// 文字の入れ替わりは本文の鍵で、種が同じなら同じ並び、最後は元の文字に落ち着く。
    #[test]
    fn shuffle_writes_seeded_content_keys_that_settle_on_the_text() {
        let script = r#"
            comp({ fps: 30 });
            shuffle(text("HELLO"), { seconds: 1, rate: 10, seed: 3, settle: 0.5 });
        "#;
        let keys = |rt: &crate::EditorRuntime| {
            let view = rt.doc.view();
            let doc = view.text_document(view.layers()[0]).unwrap().unwrap();
            doc.content.keys().iter().map(|k| (k.t.try_to_frame_round(view.composition().unwrap().unwrap().fps).unwrap(), k.content.clone())).collect::<Vec<_>>()
        };
        let (rt, outcome) = run(script);
        outcome.unwrap();
        let first = keys(&rt);
        assert_eq!(first.iter().map(|k| k.0).collect::<Vec<_>>(), (0..=10).map(|i| i * 3).collect::<Vec<_>>(), "a key every 1/rate seconds, then the final one");
        assert_eq!(first.last().unwrap().1, "HELLO");
        assert_ne!(first[0].1, "HELLO");
        assert!(first.iter().all(|k| k.1.chars().count() == 5));
        assert!(first[9].1.starts_with("HELL"), "settling left to right: {}", first[9].1);
        let (again, outcome) = run(script);
        outcome.unwrap();
        assert_eq!(first, keys(&again), "the same seed gives the same run");
    }

    #[test]
    fn rerun_replaces_the_last_run_and_keeps_later_edits_safe() {
        let dir = std::env::temp_dir().join(format!("motolii-rerun-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("sketch.js");
        let path = file.to_str().unwrap();
        std::fs::write(&file, "rectangle(); rectangle();").unwrap();
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.request(serde_json::json!({ "op": "runScript", "path": path })).unwrap();
        assert_eq!(rt.doc.view().layers().len(), 2);
        std::fs::write(&file, "ellipse();").unwrap();
        rt.request(serde_json::json!({ "op": "rerunScript" })).unwrap();
        assert_eq!(rt.doc.view().layers().len(), 1, "the old run is gone, the edited file ran");
        rt.request(serde_json::json!({ "op": "create", "kind": "star" })).unwrap();
        assert!(rt.request(serde_json::json!({ "op": "rerunScript" })).unwrap_err().contains("changed after"));
        assert_eq!(rt.doc.view().layers().len(), 2);
    }

    #[test]
    fn a_css_gradient_fills_a_shape_across_its_box() {
        let (rt, outcome) = run(r##"
            comp({ seconds: 1 });
            rectangle({ name: "Band" }).fill("linear-gradient(to right, #000000, #ff0000 25%, #ffffff)");
        "##);
        outcome.unwrap();
        let view = rt.doc.view();
        let layer = view.layers()[0];
        let shapes = view.shapes(layer).unwrap();
        let crate::doc::store::ShapeNode::Leaf(shape) = &shapes[0] else { panic!("a leaf") };
        let crate::doc::vector::Brush::Gradient(g) = &shape.fill.as_ref().unwrap().brush else { panic!("a gradient fill") };
        assert_eq!(g.units, crate::doc::vector::GradientUnits::ObjectBoundingBox);
        assert_eq!(g.stops.iter().map(|s| s.offset).collect::<Vec<_>>(), [0.0, 0.25, 1.0]);
        let d = g.end.sub(g.start);
        assert!(d.x > 0.0 && d.y.abs() < 1e-9, "\"to right\" runs left to right: {:?} → {:?}", g.start, g.end);
    }

    #[test]
    fn a_css_radial_gradient_centres_on_the_box() {
        let (rt, outcome) = run(r##"
            comp({ seconds: 1 });
            ellipse({ name: "Core" }).fill("radial-gradient(#FFFFFF, #000000)");
        "##);
        outcome.unwrap();
        let view = rt.doc.view();
        let shapes = view.shapes(view.layers()[0]).unwrap();
        let crate::doc::store::ShapeNode::Leaf(shape) = &shapes[0] else { panic!("a leaf") };
        let crate::doc::vector::Brush::Gradient(g) = &shape.fill.as_ref().unwrap().brush else { panic!("a gradient fill") };
        assert_eq!(g.kind, crate::doc::vector::GradientType::Radial);
        assert_eq!((g.start.x, g.start.y), (0.5, 0.5), "CSS radial gradients centre on the box");
    }

    #[test]
    fn a_layer_can_be_seen_through_another_layers_matte() {
        let (rt, outcome) = run(r##"
            comp({ seconds: 1 });
            const mask = ellipse({ name: "Mask" });
            rectangle({ name: "Through" }).matte(mask, "Luma");
        "##);
        outcome.unwrap();
        let view = rt.doc.view();
        let find = |name: &str| view.layers().into_iter().find(|id| view.attrs(*id).unwrap().unwrap().name == name).unwrap();
        let matte = view.attrs(find("Through")).unwrap().unwrap().matte.unwrap();
        assert_eq!((matte.layer, matte.mode), (find("Mask"), MatteMode::Luma));
    }

    #[test]
    fn saving_the_script_reruns_it_on_the_next_wake() {
        let dir = std::env::temp_dir().join(format!("motolii-live-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("live.js");
        std::fs::write(&path, r#"comp({ seconds: 1 }); rectangle({ name: "One" });"#).unwrap();
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.request(serde_json::json!({ "op": "runScript", "path": path })).unwrap();
        rt.request(serde_json::json!({ "op": "reloadEffects" })).unwrap();
        assert_eq!(rt.doc.view().layers().len(), 1, "an unchanged file is not run again");
        std::fs::write(&path, r#"comp({ seconds: 1 }); rectangle({ name: "One" }); ellipse({ name: "Two" });"#).unwrap();
        rt.request(serde_json::json!({ "op": "reloadEffects" })).unwrap();
        let view = rt.doc.view();
        let mut names: Vec<_> = view.layers().iter().map(|id| view.attrs(*id).unwrap().unwrap().name).collect();
        names.sort();
        assert_eq!(names, ["One", "Two"], "the saved script replaced the old run");
    }

    #[test]
    fn a_loop_is_only_what_the_script_asks_for() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.run_script(r#"comp({ seconds: 8 }); rectangle();"#, "plain.js").unwrap();
        assert_eq!(rt.viewer.clock.loop_range(), None, "no loop unless asked");
        rt.run_script(r#"comp({ seconds: 8, loop: [2, 6] }); rectangle();"#, "range.js").unwrap();
        assert_eq!(rt.viewer.clock.loop_range(), Some((2.0, 6.0)));
        rt.run_script(r#"comp({ seconds: 8, loop: true });"#, "whole.js").unwrap();
        assert_eq!(rt.viewer.clock.loop_range(), Some((0.0, 8.0)));
        assert!(rt.run_script(r#"comp({ loop: [1, 2] }); throw new Error("half done");"#, "broken.js").is_err());
        assert_eq!(rt.viewer.clock.loop_range(), Some((0.0, 8.0)), "a refused run leaves the last loop");
        rt.run_script(r#"comp({ seconds: 8 });"#, "again.js").unwrap();
        assert_eq!(rt.viewer.clock.loop_range(), None, "removing loop from the script removes it on rerun");
        assert!(rt.run_script(r#"comp({ loop: [6, 2] });"#, "reversed.js").is_err());
    }

    #[test]
    fn an_effect_written_after_a_repeater_reads_the_copies() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.run_script(r#"comp({ seconds: 1 });
            const bloom = group(ellipse({ name: "Petal" })).name("Bloom");
            bloom.effect("Repeater", { "Count": 4 });
            bloom.effect("Glow");"#, "order.js").unwrap();
        let view = rt.doc.view();
        let bloom = view.layers().into_iter().find(|id| view.attrs(*id).unwrap().unwrap().name == "Bloom").unwrap();
        let ids: Vec<_> = view.effects(bloom).unwrap().iter().map(|e| e.plugin_id.clone()).collect();
        assert_eq!(ids.len(), 2);
        assert!(crate::render::extensions::placement::kind(&ids[0]).is_some(), "written order is the order: {ids:?}");
    }

    /// 同梱の例は説明書の一部。窓の名前が変わったらここが赤。
    #[test]
    fn every_example_builds_its_document() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/editor/script/examples");
        let mut ran = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "js") {
                let mut rt = crate::EditorRuntime::open("").unwrap();
                let outcome = rt.run_script(&std::fs::read_to_string(&path).unwrap(), &path.to_string_lossy());
                outcome.unwrap_or_else(|message| panic!("{}: {message}", path.display()));
                let view = rt.doc.view();
                let layers = view.layers();
                assert!(!layers.is_empty(), "{} produced no layers", path.display());
                assert!(view.composition().unwrap().is_some(), "{} has no composition", path.display());
                if path.file_name().is_some_and(|name| name == "web_c8_ink_settle.js") {
                    let mut names: Vec<_> = layers.iter().map(|id| view.attrs(*id).unwrap().unwrap().name).collect();
                    names.sort();
                    assert_eq!(names, ["INK BLEEDS", "THEN THE", "WORDS SET"]);
                }
                ran += 1;
            }
        }
        assert!(ran > 0);
    }

    #[test]
    fn a_loop_that_never_ends_is_stopped() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        let started = std::time::Instant::now();
        let outcome = rt.run_script_with_budget("for(;;){}", "loop.js", std::time::Duration::from_millis(300));
        assert!(outcome.unwrap_err().contains("stopped"));
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }
}

/// `MOTOLII_SCRIPT=作品.js MOTOLII_SAVE=作品.rrd cargo test -p motolii-ui --lib -- --ignored script_file`
/// 窓を開かずに走らせて保存する(開くのは `scripts/motolii-ui.sh dev 作品.rrd`)。
#[cfg(test)]
mod file {
    #[test]
    #[ignore]
    fn script_file() {
        let path = std::env::var("MOTOLII_SCRIPT").expect("MOTOLII_SCRIPT");
        let source = std::fs::read_to_string(&path).unwrap();
        let mut rt = crate::EditorRuntime::open("").unwrap();
        if let Err(message) = rt.run_script(&source, &path) {
            panic!("{message}");
        }
        if let Ok(save) = std::env::var("MOTOLII_SAVE") {
            rt.request(serde_json::json!({ "op": "save", "path": save })).unwrap();
        }
    }
}

/// `MOTOLII_SCRIPT=作品.js MOTOLII_OUT=dir MOTOLII_FRAMES=0,120,240 cargo test -p motolii-ui --lib -- --ignored script_frames`
/// 台本を走らせ、指定のコマを comp の大きさの PNG に描く(窓を開かずに絵を見る)。
#[cfg(test)]
mod frames {
    #[test]
    #[ignore]
    fn script_frames() {
        let path = std::env::var("MOTOLII_SCRIPT").expect("MOTOLII_SCRIPT");
        let out = std::env::var("MOTOLII_OUT").expect("MOTOLII_OUT");
        let frames: Vec<i64> = std::env::var("MOTOLII_FRAMES").unwrap_or("0".into()).split(',').map(|f| f.trim().parse().unwrap()).collect();
        std::fs::create_dir_all(&out).unwrap();
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.run_script(&std::fs::read_to_string(&path).unwrap(), &path).unwrap_or_else(|m| panic!("{m}"));
        let view = rt.doc.view();
        let comp = view.composition().unwrap().unwrap();
        let mut engine = crate::render::engine::Engine::new().unwrap();
        engine.set_realtime(true);
        if std::env::var("MOTOLII_SCENE_PROBE").is_ok() { engine.set_reflection_scene_probe(true); }
        if let Ok(count) = std::env::var("MOTOLII_TIMING") {
            // Time a run of consecutive frames (after one warm frame) and name what the last one spent.
            let count: i64 = count.parse().unwrap();
            let start = frames.first().copied().unwrap_or(0);
            engine.render_frame(&view, crate::render::doc::core::RationalTime::try_from_frame(start, comp.fps).unwrap()).unwrap();
            let began = std::time::Instant::now();
            for frame in start + 1..=start + count {
                engine.render_frame(&view, crate::render::doc::core::RationalTime::try_from_frame(frame, comp.fps).unwrap()).unwrap();
            }
            eprintln!("TIMING {:.2} ms/frame over {count} frames", began.elapsed().as_secs_f64() * 1e3 / count as f64);
            let before = engine.surface_work();
            engine.render_frame(&view, crate::render::doc::core::RationalTime::try_from_frame(start + count + 1, comp.fps).unwrap()).unwrap();
            let after = engine.surface_work();
            eprintln!("  one frame: captures {} runs {} backdrop copies {} mesh batches {} instances uploaded {}",
                after.scene_captures - before.scene_captures, after.main_runs - before.main_runs, after.backdrop_copies - before.backdrop_copies,
                after.mesh_batches - before.mesh_batches, after.mesh_instances_uploaded - before.mesh_instances_uploaded);
            let mut claims = engine.frame_claims().to_vec();
            claims.sort_by(|a, b| b.us.cmp(&a.us));
            for c in claims.iter().take(10) { eprintln!("  {:>8.2} ms {:<8} {} x{} — {}", c.us as f64 / 1e3, c.stage, c.who, c.count, c.why); }
            return;
        }
        for frame in frames {
            let time = crate::render::doc::core::RationalTime::try_from_frame(frame, comp.fps).unwrap();
            let pixels = engine.render_frame(&view, time).unwrap();
            for failure in engine.layer_failures() { eprintln!("frame {frame}: {failure}"); }
            image::save_buffer(format!("{out}/frame_{frame:04}.png"), &pixels, comp.width, comp.height, image::ColorType::Rgba8).unwrap();
        }
    }
}

/// `MOTOLII_HDR_IN=a.hdr MOTOLII_HDR_OUT=b.hdr MOTOLII_HDR_KEEP=1.0 cargo test -p motolii-ui --lib -- --ignored hdr_lights_only`
/// 環境マップから光源だけを残す(明るさが KEEP 未満の部屋を黒へ落とす)。暗いスタジオで硝子を見せる時に。
#[cfg(test)]
mod hdr {
    #[test]
    #[ignore]
    fn hdr_lights_only() {
        let input = std::env::var("MOTOLII_HDR_IN").expect("MOTOLII_HDR_IN");
        let output = std::env::var("MOTOLII_HDR_OUT").expect("MOTOLII_HDR_OUT");
        let keep: f32 = std::env::var("MOTOLII_HDR_KEEP").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0);
        let image = image::open(&input).unwrap().into_rgb32f();
        let (w, h) = image.dimensions();
        let pixels: Vec<image::Rgb<f32>> = image.pixels().map(|p| {
            let l = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
            let t = ((l - keep * 0.5) / (keep * 0.5)).clamp(0.0, 1.0);
            let k = t * t * (3.0 - 2.0 * t);
            image::Rgb([p[0] * k, p[1] * k, p[2] * k])
        }).collect();
        let file = std::io::BufWriter::new(std::fs::File::create(&output).unwrap());
        image::codecs::hdr::HdrEncoder::new(file).encode(&pixels, w as usize, h as usize).unwrap();
    }
}

/// 常駐の見張り・台本の側(往復を速くする、2026-09-16 利用者「こういう往復を早くしたい」「リリースいるかなー」):
/// 台本の保存を見張り、変わる度に 台本 → 書類(`MOTOLII_OUT/shot.rrd`)を、cargo を起動せずに作り直す。
/// 描くのは render の側の見張り(`zz_watch`、release で組んである)に任せる — ここは描かないので debug で足りる。
#[cfg(test)]
mod watch {
    #[test]
    #[ignore]
    fn watch_shot() {
        let script = std::env::var("MOTOLII_SCRIPT").expect("MOTOLII_SCRIPT");
        let out = std::env::var("MOTOLII_OUT").expect("MOTOLII_OUT");
        std::fs::create_dir_all(&out).unwrap();
        let mut seen = None;
        loop {
            let stamp = std::fs::metadata(&script).and_then(|m| m.modified()).ok();
            if stamp == seen || stamp.is_none() {
                std::thread::sleep(std::time::Duration::from_millis(300));
                continue;
            }
            seen = stamp;
            let started = std::time::Instant::now();
            let source = std::fs::read_to_string(&script).unwrap_or_default();
            let mut rt = match crate::EditorRuntime::open("") { Ok(rt) => rt, Err(e) => { eprintln!("open: {e}"); continue } };
            if let Err(message) = rt.run_script(&source, &script) {
                eprintln!("script: {message}");
                continue;
            }
            // 書きかけを描かせないよう、別名で保存してから差し替える。
            let (tmp, saved) = (format!("{out}/shot.rrd.tmp"), format!("{out}/shot.rrd"));
            if let Err(e) = rt.request(serde_json::json!({ "op": "save", "path": tmp })) { eprintln!("save: {e:?}"); continue }
            let _ = std::fs::rename(&tmp, &saved);
            eprintln!("doc: {saved} in {:.1}s", started.elapsed().as_secs_f32());
        }
    }
}

#[cfg(test)]
#[path = "script/sweep.rs"]
mod sweep;
