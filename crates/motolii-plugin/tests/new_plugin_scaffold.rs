//! INF-7e: `scripts/new-plugin.sh` が吐く desc が validate_node_desc を通ることの機械判定。

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use motolii_gpu::{GpuCtx, PipelineCache};
use motolii_plugin::{
    validate_node_desc, FilterPlugin, NodeDesc, PluginError, PluginId, PluginKind, PluginRegistry,
    RenderCtx, ResolvedParams, TextureRef,
};
use serde::Deserialize;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-plugin は workspace 直下")
        .to_path_buf()
}

#[derive(Debug, Deserialize)]
struct ScaffoldDesc {
    id: String,
    version: u32,
    display_name: String,
    category: String,
    tags: Vec<String>,
    min_inputs: usize,
    max_inputs: usize,
    plugin_kind: String,
}

fn plugin_kind(s: &str) -> PluginKind {
    match s {
        "Filter" => PluginKind::Filter,
        "LayerSource" => PluginKind::LayerSource,
        "ParamDriver" => PluginKind::ParamDriver,
        "Composite" => PluginKind::Composite,
        other => panic!("unknown plugin_kind from generator: {other}"),
    }
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn node_desc_from_scaffold(s: &ScaffoldDesc) -> NodeDesc {
    let tags: Vec<&'static str> = s.tags.iter().cloned().map(leak).collect();
    let tags: &'static [&'static str] = Box::leak(tags.into_boxed_slice());
    NodeDesc {
        id: PluginId(leak(s.id.clone())),
        version: s.version,
        display_name: leak(s.display_name.clone()),
        category: leak(s.category.clone()),
        tags,
        params: vec![],
        min_inputs: s.min_inputs,
        max_inputs: s.max_inputs,
    }
}

fn run_print_desc(kind: &str, name: &str) -> ScaffoldDesc {
    let script = workspace_root().join("scripts/new_plugin.py");
    let out = Command::new("python3")
        .arg(&script)
        .args([kind, name, "--print-desc-json"])
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", script.display()));
    assert!(
        out.status.success(),
        "generator failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("desc json")
}

#[test]
fn generator_desc_json_passes_validate_for_all_kinds() {
    let cases = [
        ("filter", "glow"),
        ("layer_source", "solid"),
        ("param_driver", "lfo"),
        ("composite", "mix"),
    ];
    for (kind, name) in cases {
        let scaffold = run_print_desc(kind, name);
        let desc = node_desc_from_scaffold(&scaffold);
        validate_node_desc(plugin_kind(&scaffold.plugin_kind), &desc)
            .unwrap_or_else(|e| panic!("{kind}/{name}: {e}"));
    }
}

#[test]
fn generator_writes_rust_with_validate_test_stub() {
    let root = workspace_root();
    let out_dir = root.join("target/new-plugin-scaffold-test");
    std::fs::create_dir_all(&out_dir).unwrap();
    let out = out_dir.join("glow_filter.rs");
    let script = root.join("scripts/new_plugin.py");
    let status = Command::new("python3")
        .arg(&script)
        .args([
            "filter",
            "glow",
            "--vendor",
            "acme",
            "--out",
            out.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let src = std::fs::read_to_string(&out).unwrap();
    assert!(src.contains("PluginId(\"acme.filter.glow\")"));
    assert!(src.contains("tags: &[\"glow\", \"scaffold\"]"));
    assert!(src.contains("generated_desc_passes_validate_node_desc"));
    assert!(src.contains("impl FilterPlugin for Glow"));
}

/// スクリプトが吐く既定 desc をレジストリ登録経路でも拒否されないことの型紙。
#[test]
fn scaffold_filter_registers_cleanly() {
    struct Glow;
    impl FilterPlugin for Glow {
        fn desc(&self) -> &NodeDesc {
            static DESC: OnceLock<NodeDesc> = OnceLock::new();
            DESC.get_or_init(|| {
                let scaffold = run_print_desc("filter", "glow");
                node_desc_from_scaffold(&scaffold)
            })
        }

        fn render(
            &self,
            _gpu: &GpuCtx,
            _pipelines: &mut PipelineCache,
            _encoder: &mut wgpu::CommandEncoder,
            _ctx: &RenderCtx,
            _params: &ResolvedParams,
            _input: TextureRef<'_>,
            _output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            Ok(())
        }
    }
    static GLOW: Glow = Glow;
    let mut registry = PluginRegistry::new();
    registry.register_filter(&GLOW).unwrap();
}
