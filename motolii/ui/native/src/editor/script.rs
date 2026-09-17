//! スクリプトの口 — JS が窓の操作(port の op)を並べる。名前は窓の名前だけ([スクリプトの口](../../../../docs/reviews/2026-09-14-script-mouth.md))。
use rquickjs::{CatchResultExt, CaughtError, Context, Ctx, Exception, Function, Runtime};
use serde_json::json;

use crate::doc::store::LayerId;
use crate::EditorRuntime;

const PRELUDE: &str = include_str!("script/prelude.js");
/// 1 本のスクリプトが走ってよい時間。無限ループは窓を止めるので、ここで切る。
const BUDGET: std::time::Duration = std::time::Duration::from_secs(20);

fn thrown(ctx: &Ctx<'_>, message: impl std::fmt::Display) -> rquickjs::Error {
    Exception::throw_message(ctx, &message.to_string())
}

impl EditorRuntime {
    /// `name` は利用者の file 名。例外の行・列はこの名前で返る(LLM が読んで直す)。
    pub(crate) fn run_script(&mut self, source: &str, name: &str) -> Result<(), String> {
        self.run_script_with_budget(source, name, BUDGET)
    }

    fn run_script_with_budget(&mut self, source: &str, name: &str, budget: std::time::Duration) -> Result<(), String> {
        let head = self.doc.edit_head();
        let outcome = self.run_script_inner(source, name, budget);
        if outcome.is_err() {
            // 途中で断られたスクリプトは何も残さない。直して走らせ直す時、前の半端な層が混ざらない。
            while self.doc.edit_head() > head && self.doc.undo() {}
            self.selected_keys.clear();
        }
        outcome
    }

    fn run_script_inner(&mut self, source: &str, name: &str, budget: std::time::Duration) -> Result<(), String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        runtime.set_memory_limit(512 * 1024 * 1024);
        let started = std::time::Instant::now();
        runtime.set_interrupt_handler(Some(Box::new(move || started.elapsed() > budget)));
        let context = Context::full(&runtime).map_err(|e| e.to_string())?;
        // 関数は 'js より長く生きる物しか掴めない。context はこの関数の中で捨てるので、その間だけ self を指す。
        let this: *mut EditorRuntime = self;
        let outcome = context.with(|ctx| -> Result<(), String> {
            let bind = || -> rquickjs::Result<()> {
            let globals = ctx.globals();
            globals.set("__op", Function::new(ctx.clone(), move |ctx: Ctx<'_>, request: String| -> rquickjs::Result<String> {
                let runtime = unsafe { &mut *this };
                let j: serde_json::Value = serde_json::from_str(&request).map_err(|e| thrown(&ctx, e))?;
                runtime.request(j).map_err(|e| thrown(&ctx, e))?;
                Ok(json!({ "selected": runtime.selected.map(|id| id.0) }).to_string())
            })?)?;
            globals.set("__layer", Function::new(ctx.clone(), move |ctx: Ctx<'_>, id: f64| -> rquickjs::Result<String> {
                let runtime = unsafe { &*this };
                runtime.script_layer(LayerId(id as u64)).map_err(|e| thrown(&ctx, e))
            })?)?;
            globals.set("__effects", Function::new(ctx.clone(), || -> String {
                let rows: Vec<_> = crate::render::engine::known_effects().iter().map(|d| json!({ "name": d.label, "pluginId": d.plugin_id })).collect();
                json!(rows).to_string()
            })?)?;
            globals.set("__assets", Function::new(ctx.clone(), move |ctx: Ctx<'_>| -> rquickjs::Result<String> {
                let runtime = unsafe { &*this };
                let assets = runtime.doc.view().assets().map_err(|e| thrown(&ctx, e))?;
                Ok(json!(assets.iter().map(|a| json!({ "id": a.id.get(), "path": a.path_absolute, "name": a.name })).collect::<Vec<_>>()).to_string())
            })?)?;
            globals.set("__comp", Function::new(ctx.clone(), move |ctx: Ctx<'_>| -> rquickjs::Result<String> {
                let runtime = unsafe { &*this };
                let comp = runtime.doc.view().composition().map_err(|e| thrown(&ctx, e))?.ok_or_else(|| thrown(&ctx, "No composition"))?;
                let fps = comp.fps.as_f64();
                Ok(json!({ "width": comp.width, "height": comp.height, "fps": fps, "seconds": comp.duration_frames as f64 / fps }).to_string())
            })?)?;
            Ok(())
            };
            bind().map_err(|e| e.to_string())?;
            let run = |source: &str, file: &str| {
                let mut options = rquickjs::context::EvalOptions::default();
                options.filename = Some(file.to_owned());
                ctx.eval_with_options::<(), _>(source, options).catch(&ctx).map_err(|e| match e {
                    CaughtError::Exception(e) => format!("{}{}", e.message().unwrap_or_default(), e.stack().map(|s| format!("\n{s}")).unwrap_or_default()),
                    other => other.to_string(),
                })
            };
            run(PRELUDE, "motolii-prelude.js")?;
            run(source, name)
        });
        let _ = self.request(json!({ "op": "animate", "enabled": false }));
        let _ = self.request(json!({ "op": "seek", "frame": 0 }));
        if started.elapsed() > budget {
            return Err(format!("{name}: stopped after {:.1} seconds (a loop that never ends?)", budget.as_secs_f64()));
        }
        outcome
    }

    pub(crate) fn run_script_file(&mut self, path: &str) -> Result<(), String> {
        let source = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let before = self.doc.edit_head();
        self.run_script(&source, path)?;
        self.last_script = Some((path.to_owned(), before, self.doc.edit_head()));
        Ok(())
    }

    /// 直前のスクリプトの結果を戻して、同じ file を読み直して走らせる(書いて・保存して・見る)。
    /// 走った後に窓で手を入れていたら、その手を消さないように断る。
    pub(crate) fn rerun_script(&mut self) -> Result<(), String> {
        let (path, before, after) = self.last_script.clone().ok_or("Run a script first")?;
        if self.doc.edit_head() != after {
            return Err("The document changed after the script ran. Undo those edits or use Run Script…".into());
        }
        while self.doc.edit_head() > before && self.doc.undo() {}
        self.selected_keys.clear();
        self.run_script_file(&path)
    }

    /// 窓の Inspector に渡す行と同じ物(名前・id・値・効果)。
    fn script_layer(&self, id: LayerId) -> Result<String, String> {
        let view = self.doc.view();
        let comp = view.composition().map_err(|e| e.to_string())?.ok_or("No composition")?;
        let catalog = crate::render::engine::known_effects();
        let at = self.time()?;
        let row = self.layer_json(&view, id, at, comp.fps, &catalog, &Default::default(), false)?.ok_or_else(|| format!("Layer {} no longer exists", id.0))?;
        Ok(row.to_string())
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

    /// 同梱の例は説明書の一部。窓の名前が変わったらここが赤。
    #[test]
    fn every_example_builds_its_document() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/editor/script/examples");
        let mut ran = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "js") {
                let (rt, outcome) = run(&std::fs::read_to_string(&path).unwrap());
                outcome.unwrap_or_else(|message| panic!("{}: {message}", path.display()));
                assert!(rt.doc.view().layers().len() > 3, "{}", path.display());
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
