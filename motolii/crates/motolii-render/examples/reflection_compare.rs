//! Paired current-time comparison, with and without reflection content reuse.
use motolii_render::{
    doc::store::{Document, Intent, PropertyId, RationalTime, Value},
    engine::Engine,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let args: Vec<_> = std::env::args().collect();
    if !(3..=4).contains(&args.len()) {
        return Err("usage: reflection_compare document.rrd result.json [gpu-sharing]".into());
    }
    let mut doc = Document::load(&args[1])?;
    let comp = doc.view().composition()?.ok_or("no composition")?;
    let layers = doc.view().resolved_layers(RationalTime::ZERO)?;
    let receiver = layers
        .iter()
        .find(|l| l.effects.iter().any(|e| e.plugin_id == "motolii.glass"))
        .ok_or("no glass receiver")?
        .id;
    let mut off = Engine::new()?;
    let mut on = Engine::new()?;
    let gpu_sharing = args.get(3).is_some_and(|s| s == "gpu-sharing");
    if gpu_sharing {
        off.set_gpu_instance_sharing_enabled(false);
    } else {
        off.set_reflection_cache_enabled(false);
    }
    off.set_render_measurement_enabled(true);
    on.set_render_measurement_enabled(true);
    let mut records = Vec::new();
    for scenario in ["static", "moving"] {
        for frame in 0..23 {
            if scenario == "moving" {
                doc.apply(Intent::SetConstant {
                    layer: receiver,
                    property: PropertyId::new("rotation.y")?,
                    value: Value::F64(20.0 + frame as f64 * 0.7),
                })?;
            }
            let mut outputs = Vec::new();
            for mode in if frame % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let engine = if mode { &mut on } else { &mut off };
                let before = engine.surface_work();
                let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
                if !engine.layer_failures().is_empty() {
                    return Err(engine.layer_failures().join("; ").into());
                }
                let after = engine.surface_work();
                let m = engine.frame_measurement();
                if frame >= 3 {
                    records.push(json!({"scenario":scenario,"frame":frame-3,"variant_enabled":mode,
                        "cache":if gpu_sharing { true } else { mode },
                        "gpu_instance_sharing":if gpu_sharing { mode } else { true },
                        "total_us":m.total_us,"prepare_us":m.prepare_us,"submit_us":m.submit_us,
                        "resolve_us":m.resolve_us,"layer_build_us":m.layer_build_us,
                        "draw_data_prepare_us":after.draw_data_prepare_us-before.draw_data_prepare_us,
                        "mesh_instances_uploaded":after.mesh_instances_uploaded-before.mesh_instances_uploaded,
                        "mesh_instance_upload_bytes":after.mesh_instance_upload_bytes-before.mesh_instance_upload_bytes,
                        "wait_us":m.wait_us,"readback_us":m.readback_us,"gpu_us":m.final_submission_gpu_us,"gpu_status":m.gpu_status,
                        "key_us":after.cache_key_us-before.cache_key_us,
                        "captures":after.scene_captures-before.scene_captures,
                        "hits":after.cache_hits-before.cache_hits,"misses":after.cache_misses-before.cache_misses,
                        "bypasses":after.cache_bypasses-before.cache_bypasses,
                        "retained_input_texture_bytes":after.cache_retained_texture_bytes,
                        "main_runs":after.main_runs-before.main_runs,"copies":after.backdrop_copies-before.backdrop_copies,
                    }));
                }
                outputs.push(pixels);
            }
            if outputs[0] != outputs[1] {
                return Err(format!("pixel mismatch in {scenario} frame {frame}").into());
            }
        }
    }
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(&json!({
            "comparison":if gpu_sharing { "gpu-sharing" } else { "reflection-cache" },
            "source":args[1],"resolution":[comp.width,comp.height],"warmup":3,"samples":20,
            "pixel_equality":true,"alternating_order":true,
            "gpu_scope":"final submitted command batch, includes screenshot GPU copy; excludes prior uploads/submits",
            "readback_scope":"CPU collection after GPU completion; GPU copy belongs to submitted batch",
            "records":records,
        }))?,
    )?;
    println!("matched all frames; wrote {}", args[2]);
    Ok(())
}
