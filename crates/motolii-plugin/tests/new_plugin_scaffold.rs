//! INF-7e / M2E-10: `scripts/new-plugin.sh` 生成物の機械判定。
//!
//! - 製品コードは自己クレート配置(`use crate::`)。testkit テストは別成果物
//! - 文字列走査に加え、motolii-plugin / motolii-testkit 実配置で `--locked` コンパイルする
//! - ゴールデンは fail-closed(自己参照・無assertを拒否)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use motolii_eval::Value;
use motolii_gpu::{GpuCtx, PipelineCache};
use motolii_plugin::{
    validate_node_desc, FilterPlugin, NodeDesc, ParamDef, PluginError, PluginId, PluginKind,
    PluginRegistry, RenderCtx, ResolvedParams, TextureRef, ValueType,
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
    param_id: String,
    purity_fn: String,
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
        params: vec![ParamDef {
            id: leak(s.param_id.clone()),
            value_type: ValueType::F64,
            default: Value::F64(1.0),
        }],
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

fn generate_pair(
    kind: &str,
    name: &str,
    vendor: &str,
    plugin_out: &Path,
    test_out: &Path,
    plugin_import: &str,
) {
    let script = workspace_root().join("scripts/new_plugin.py");
    let status = Command::new("python3")
        .arg(&script)
        .args([
            kind,
            name,
            "--vendor",
            vendor,
            "--out",
            plugin_out.to_str().unwrap(),
            "--out-test",
            test_out.to_str().unwrap(),
            "--plugin-import",
            plugin_import,
        ])
        .status()
        .unwrap();
    assert!(status.success(), "generator failed for {kind}/{name}");
}

/// 製品コード: 自己クレート配置 + ParamDef / validate。testkit 非参照。
fn assert_plugin_artifact(src: &str) {
    assert!(
        src.contains("use crate::{") || src.contains("use crate::"),
        "plugin artifact must use crate:: imports (paste into motolii-plugin)"
    );
    assert!(
        !src.contains("use motolii_plugin::"),
        "plugin artifact must not use motolii_plugin:: (that is external-crate layout)"
    );
    assert!(
        !src.contains("motolii_testkit"),
        "plugin artifact must not reference motolii_testkit"
    );
    assert!(
        !src.contains("assert_filter_pure")
            && !src.contains("assert_layer_source_pure")
            && !src.contains("assert_composite_pure")
            && !src.contains("assert_param_driver_pure")
            && !src.contains("assert_rgba_close"),
        "purity/golden must live in the separate test artifact"
    );
    assert!(src.contains("ParamDef"), "missing ParamDef example");
    assert!(
        src.contains("generated_desc_passes_validate_node_desc"),
        "missing validate unit test"
    );
}

/// testkit 成果物: purity + fail-closed ゴールデン。自己参照・無assertを拒否。
fn assert_test_artifact(src: &str, purity_fn: &str, is_param_driver: bool) {
    assert!(src.contains(purity_fn), "missing purity stub ({purity_fn})");
    assert!(src.contains("scaffold_is_pure"), "missing purity test fn");
    assert!(src.contains("scaffold_golden_stub"), "missing golden stub");
    assert!(
        src.contains("motolii_testkit"),
        "test artifact must use motolii_testkit"
    );

    assert!(
        !src.contains("actual.clone()"),
        "golden must not self-compare via actual.clone()"
    );
    assert!(
        !src.contains("let expected = actual"),
        "golden must not alias expected to actual"
    );
    assert!(
        !src.contains("let _ = track"),
        "param_driver golden must not discard track without assert"
    );

    if is_param_driver {
        assert!(
            !src.contains("assert_rgba_close"),
            "param_driver has no pixels"
        );
        assert!(
            src.contains("scaffold golden: set expected Value sequence"),
            "missing fail-closed Value oracle marker"
        );
        assert!(
            src.contains("let expected: Option<DataTrack> = None"),
            "expected oracle must start as None"
        );
    } else {
        assert!(src.contains("assert_rgba_close"));
        assert!(src.contains("gpu_or_skip"));
        assert!(
            src.contains("scaffold golden: set expected RGBA oracle"),
            "missing fail-closed RGBA oracle marker"
        );
        assert!(
            src.contains("let expected: Option<&[u8]> = None"),
            "expected oracle must start as None"
        );
    }
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
        assert!(!scaffold.param_id.is_empty());
        assert!(scaffold.purity_fn.starts_with("assert_"));
    }
}

#[test]
fn generator_writes_separate_plugin_and_test_artifacts() {
    let root = workspace_root();
    let out_dir = root.join("target/new-plugin-scaffold-test");
    std::fs::create_dir_all(&out_dir).unwrap();

    let cases = [
        ("filter", "glow", "acme", "assert_filter_pure", false),
        (
            "layer_source",
            "blob",
            "acme",
            "assert_layer_source_pure",
            false,
        ),
        (
            "param_driver",
            "lfo",
            "acme",
            "assert_param_driver_pure",
            true,
        ),
        ("composite", "mix", "acme", "assert_composite_pure", false),
    ];
    for (kind, name, vendor, purity_fn, is_pd) in cases {
        let plugin_out = out_dir.join(format!("{name}_{kind}.rs"));
        let test_out = out_dir.join(format!("{name}_{kind}_test.rs"));
        generate_pair(
            kind,
            name,
            vendor,
            &plugin_out,
            &test_out,
            "motolii_plugin::reference",
        );
        let plugin_src = std::fs::read_to_string(&plugin_out).unwrap();
        let test_src = std::fs::read_to_string(&test_out).unwrap();
        assert_plugin_artifact(&plugin_src);
        assert_test_artifact(&test_src, purity_fn, is_pd);
        assert!(plugin_src.contains(&format!(
            "PluginId(\"{vendor}.{}.{name}\")",
            match kind {
                "filter" => "filter",
                "layer_source" => "layer_source",
                "param_driver" => "param",
                "composite" => "composite",
                _ => unreachable!(),
            }
        )));
    }

    let filter_src = std::fs::read_to_string(out_dir.join("glow_filter.rs")).unwrap();
    assert!(filter_src.contains("impl FilterPlugin for Glow"));
    assert!(filter_src.contains("RenderCtx"));
    assert!(
        !filter_src.contains("use motolii_core::RationalTime"),
        "filter plugin must not import unused RationalTime"
    );

    let layer_src = std::fs::read_to_string(out_dir.join("blob_layer_source.rs")).unwrap();
    assert!(layer_src.contains("use motolii_core::RationalTime"));
}

/// 生成物を motolii-plugin / motolii-testkit の実配置で `--locked` コンパイルする。
#[test]
fn generated_artifacts_compile_in_self_crate_layout() {
    let root = workspace_root();
    let fixture = root.join("target/scaffold-plugin-fixture");
    let in_plugin = fixture.join("in_plugin");
    let in_testkit = fixture.join("in_testkit");

    // 生成物ディレクトリだけ作り直す。workspace の Cargo.lock は触らない。
    let _ = std::fs::remove_dir_all(&in_plugin);
    let _ = std::fs::remove_dir_all(&in_testkit);
    std::fs::create_dir_all(&in_plugin).unwrap();
    std::fs::create_dir_all(&in_testkit).unwrap();

    let cases = [
        ("filter", "glow"),
        ("layer_source", "blob"),
        ("param_driver", "lfo"),
        ("composite", "mix"),
    ];
    let mut plugin_mod =
        String::from("//! M2E-10 fixture: generated plugins compiled inside motolii-plugin.\n\n");
    let mut testkit_mod = String::from("//! M2E-10 fixture: generated testkit tests.\n\n");

    for (kind, name) in cases {
        let mod_name = format!("{name}_{kind}");
        let plugin_out = in_plugin.join(format!("{mod_name}.rs"));
        let test_out = in_testkit.join(format!("{mod_name}_test.rs"));
        let plugin_import = format!("motolii_plugin::scaffold_fixture::{mod_name}");
        generate_pair(kind, name, "acme", &plugin_out, &test_out, &plugin_import);

        let plugin_src = std::fs::read_to_string(&plugin_out).unwrap();
        assert_plugin_artifact(&plugin_src);
        // 生成物は integration test 向け(`motolii_testkit::`)。lib 単体テスト取り込みでは `crate::` へ。
        let test_src = std::fs::read_to_string(&test_out).unwrap();
        let test_src = test_src.replace("use motolii_testkit::", "use crate::");
        std::fs::write(&test_out, test_src).unwrap();
        plugin_mod.push_str(&format!("pub mod {mod_name};\n"));
        testkit_mod.push_str(&format!(
            "#[path = \"{mod_name}_test.rs\"]\nmod {mod_name}_test;\n"
        ));
    }
    std::fs::write(in_plugin.join("mod.rs"), plugin_mod).unwrap();
    std::fs::write(in_testkit.join("mod.rs"), testkit_mod).unwrap();

    // 自己クレート配置: 独自 cfg は env 経由(Cargo feature ではない)。workspace --locked。
    // 親の `target/` を上書きすると、後続の workspace テストが fixture 付きバイナリを
    // 実行してしまうため、ネスト cargo は専用 target-dir へ隔離する。
    let nested_target = fixture.join("cargo-target");
    let check_plugin = Command::new("cargo")
        .args([
            "check",
            "-p",
            "motolii-plugin",
            "--locked",
            "--target-dir",
            nested_target.to_str().unwrap(),
        ])
        .env("MOTOLII_SCAFFOLD_FIXTURE", "1")
        .current_dir(&root)
        .output()
        .expect("cargo check motolii-plugin scaffold fixture");
    assert!(
        check_plugin.status.success(),
        "motolii-plugin self-crate fixture failed to compile:\n{}\n{}",
        String::from_utf8_lossy(&check_plugin.stdout),
        String::from_utf8_lossy(&check_plugin.stderr)
    );

    // testkit 実配置: lib 単体テストとして OUT_DIR 経由で取り込む(実行はしない)
    let check_tests = Command::new("cargo")
        .args([
            "test",
            "-p",
            "motolii-testkit",
            "--lib",
            "--no-run",
            "--locked",
            "--target-dir",
            nested_target.to_str().unwrap(),
        ])
        .env("MOTOLII_SCAFFOLD_FIXTURE", "1")
        .current_dir(&root)
        .output()
        .expect("cargo test --lib --no-run scaffold fixture");
    assert!(
        check_tests.status.success(),
        "motolii-testkit fixture tests failed to compile:\n{}\n{}",
        String::from_utf8_lossy(&check_tests.stdout),
        String::from_utf8_lossy(&check_tests.stderr)
    );
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
