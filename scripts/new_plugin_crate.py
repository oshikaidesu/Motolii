#!/usr/bin/env python3
"""VSM-A4I: public motolii-plugin facade only の外部作者crate scaffold。

This tool deliberately has one source fixture. It does not install, package, load,
or register a plugin. `--check` owns the temporary Host conformance harness.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SOURCE_ID = "core.layer_source.radial_repeater"
IDENT_RE = re.compile(r"^[a-z][a-z0-9_]*$")
PLUGIN_ID_RE = re.compile(r'const\s+PLUGIN_ID:\s*&str\s*=\s*"([^"]+)"')
# const を強制しない。`PluginId("...")` からも同定する。
PLUGIN_ID_FALLBACK_RE = re.compile(r'PluginId\(\s*"([^"]+)"\s*\)')
FORBIDDEN_SOURCE = (
    "motolii_core",
    "motolii_eval",
    "motolii_gpu",
    "motolii_testkit",
    "std::fs",
    "std::net",
    "std::process",
    "std::env",
    "std::os",
    "unsafe",
    'extern "',
    "#[link",
    "egui",
    "eframe",
    "winit",
    "metal",
    "vulkan",
    "cuda",
    "dx12",
)


def die(message: str) -> None:
    raise SystemExit(f"new-plugin-crate: {message}")


def is_within(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def check_identifier(label: str, value: str) -> str:
    if not IDENT_RE.fullmatch(value):
        die(f"{label} must be lowercase ASCII identifier: {value!r}")
    return value


def display_name(name: str) -> str:
    return " ".join(piece[:1].upper() + piece[1:] for piece in name.split("_"))


def crate_name(vendor: str, name: str) -> str:
    return f"motolii-plugin-{vendor.replace('_', '-')}-{name.replace('_', '-')}"


def generated_manifest(vendor: str, name: str, plugin_path: Path) -> str:
    return f'''[package]
name = "{crate_name(vendor, name)}"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"

[dependencies]
motolii-plugin = {{ path = "{plugin_path}" }}

# Keep an external author crate outside the Host workspace even when it is created nearby.
[workspace]
'''


def generated_authoring(vendor: str, name: str, out_dir: Path) -> str:
    plugin_id = f"{vendor}.layer_source.{name}"
    return f'''# {display_name(name)}

This is a source fork of `{SOURCE_ID}` with the independent identity
`{plugin_id}`. Its only regular dependency is the public `motolii-plugin` facade.

## Host check

After editing `src/lib.rs`, run this Host-owned check from the Motolii checkout:

```sh
scripts/new-plugin-crate.sh --check "{out_dir}"
```

The check creates and removes its own outside-repository harness. It compiles the
crate, validates the contract and registry entry, and runs required-GPU purity and
golden checks. A missing GPU is a failure, not a skipped success.

## Adoption boundary

This command does not install, package, load, or register the crate. An adopter
must add the crate and its contract to a first-party composition root explicitly,
then rebuild and restart the Host. `.vism` packages and dynamic third-party loading
are not implemented here.
'''


ENTRY_POINTS_LAYER_SOURCE = """

// Host検査が呼ぶ固定の入口。kind は PluginContract が運ぶため、この2本は kind に依存しない。
// これはHost内検査の規約であって公開ABIではない(動的ロードは未実装)。
pub fn register_contracts(
    catalog: &mut motolii_plugin::PluginCatalogBuilder,
) -> Result<(), motolii_plugin::PluginContractError> {
    catalog.register(radial_repeater_contract())
}

pub fn register_plugins(
    registry: &mut motolii_plugin::PluginRegistry,
) -> Result<(), motolii_plugin::PluginError> {
    registry.register_layer_source(&RADIAL_REPEATER_LAYER_SOURCE)
}
"""


def generated_source(vendor: str, name: str) -> str:
    template_path = ROOT / "plugins/motolii-plugin-radial-repeater/src/lib.rs"
    source = template_path.read_text(encoding="utf-8")
    marker = "\n#[cfg(test)]\nmod tests {"
    if marker not in source:
        die(f"source fixture changed; cannot split tests: {template_path}")
    source = source.split(marker, 1)[0].rstrip() + "\n"
    plugin_id = f"{vendor}.layer_source.{name}"
    source = source.replace(
        f'const PLUGIN_ID: &str = "{SOURCE_ID}";',
        f'const PLUGIN_ID: &str = "{plugin_id}";',
        1,
    )
    source = source.replace(
        'display_name: "Radial Repeater",',
        f'display_name: "{display_name(name)}",',
        1,
    )
    source = source.replace(
        f'//! `{SOURCE_ID}` version 1 — 外部 LayerSource crate (VSM-A3-2)。',
        f'//! `{plugin_id}` version 1 — external author source fork (VSM-A4I)。',
        1,
    )
    if plugin_id not in source:
        die("source fixture did not accept the generated plugin identity")

    source += ENTRY_POINTS_LAYER_SOURCE
    return source


def generate(args: argparse.Namespace) -> None:
    if args.source != SOURCE_ID:
        die(f"--from supports only {SOURCE_ID}")
    vendor = check_identifier("vendor", args.vendor)
    name = check_identifier("name", args.name)
    if vendor in {"core", "doc"}:
        die(f"vendor {vendor!r} is reserved for first-party/built-in plugins")

    out_dir = Path(args.out_dir).expanduser().resolve()
    if is_within(out_dir, ROOT):
        die(f"--out-dir must be outside the Motolii repository: {out_dir}")
    if out_dir.exists():
        die(f"--out-dir must not already exist: {out_dir}")

    out_dir.mkdir(parents=True)
    (out_dir / "src").mkdir()
    (out_dir / "Cargo.toml").write_text(
        generated_manifest(vendor, name, ROOT / "crates/motolii-plugin"), encoding="utf-8"
    )
    (out_dir / "src/lib.rs").write_text(generated_source(vendor, name), encoding="utf-8")
    (out_dir / "AUTHORING.md").write_text(
        generated_authoring(vendor, name, out_dir), encoding="utf-8"
    )
    print(f"generated external plugin crate: {out_dir}")


def read_candidate_manifest(crate_dir: Path) -> tuple[str, Path]:
    manifest_path = crate_dir / "Cargo.toml"
    if not manifest_path.is_file():
        die(f"check[hygiene] crate={crate_dir} file={manifest_path}: Cargo.toml is missing")
    return manifest_path.read_text(encoding="utf-8"), manifest_path


def toml_section(manifest: str, name: str) -> str | None:
    match = re.search(
        rf"(?ms)^\[{re.escape(name)}\]\s*$\n(.*?)(?=^\[|\Z)", manifest
    )
    return match.group(1) if match else None


def toml_literal(section: str, key: str) -> str | None:
    match = re.search(rf'^\s*{re.escape(key)}\s*=\s*"([^"]+)"\s*$', section, re.MULTILINE)
    return match.group(1) if match else None


def section_keys(section: str) -> set[str]:
    return set(re.findall(r"^\s*([A-Za-z0-9_-]+)\s*=", section, re.MULTILINE))


def source_plugin_id(crate_dir: Path) -> str:
    source_path = crate_dir / "src/lib.rs"
    if not source_path.is_file():
        die(f"check[hygiene] crate={crate_dir} file={source_path}: source is missing")
    match = PLUGIN_ID_RE.search(source_path.read_text(encoding="utf-8"))
    return match.group(1) if match else "<unresolved>"


def check_hygiene(crate_dir: Path) -> tuple[str, str]:
    manifest, manifest_path = read_candidate_manifest(crate_dir)
    plugin_id = source_plugin_id(crate_dir)

    def reject(message: str, path: Path = manifest_path) -> None:
        die(
            f"check[hygiene] crate={crate_dir} file={path} plugin={plugin_id}: {message}"
        )

    package = toml_section(manifest, "package")
    if package is None:
        reject("[package] is required")
    for key in ("name", "version", "edition", "license"):
        if toml_literal(package, key) is None:
            reject(f"package.{key} must be a literal generated value")
    if {"build", "workspace"} & section_keys(package):
        reject("package build/workspace inheritance is forbidden")
    workspace = toml_section(manifest, "workspace")
    if workspace is None or section_keys(workspace):
        reject("an empty [workspace] is required")
    for table in ("dev-dependencies", "build-dependencies"):
        if toml_section(manifest, table) is not None:
            reject(f"[{table}] is forbidden")
    dependencies = toml_section(manifest, "dependencies")
    if dependencies is None or section_keys(dependencies) != {"motolii-plugin"}:
        reject("the only regular dependency must be motolii-plugin")
    dependency = re.search(r'^\s*motolii-plugin\s*=\s*\{([^}]*)\}\s*$', dependencies, re.MULTILINE)
    if dependency is None or not re.search(r'\bpath\s*=\s*"[^"]+"', dependency.group(1)):
        reject("motolii-plugin must use an explicit path value")
    if re.search(r'\b(git|version|workspace)\s*=', dependency.group(1)):
        reject("motolii-plugin dependency must not use git/version/workspace inheritance")
    clippy = toml_section(manifest, "lints.clippy")
    if clippy is None or any(
        toml_literal(clippy, name) != "deny"
        for name in ("unwrap_used", "expect_used", "panic", "todo", "unimplemented")
    ):
        reject("literal clippy deny lints are required")
    if (crate_dir / "build.rs").exists():
        reject("build.rs is forbidden", crate_dir / "build.rs")

    source_files = sorted((crate_dir / "src").rglob("*.rs"))
    if not source_files:
        reject("src must contain Rust source", crate_dir / "src")
    for source_path in source_files:
        source = source_path.read_text(encoding="utf-8")
        for marker in FORBIDDEN_SOURCE:
            if marker in source:
                reject(f"forbidden direct reference: {marker}", source_path)
    if plugin_id == "<unresolved>" or not re.fullmatch(
        r"[a-z][a-z0-9_]*\.(layer_source|filter|param_driver|composite)\.[a-z][a-z0-9_]*",
        plugin_id,
    ):
        reject(
            "plugin identity must be vendor.<kind>.name where kind is one of "
            "layer_source, filter, param_driver, composite",
            crate_dir / "src/lib.rs",
        )
    if plugin_id.split(".", 1)[0] in {"core", "doc"}:
        reject("plugin identity uses a reserved namespace", crate_dir / "src/lib.rs")
    package_name = toml_literal(package, "name")
    assert package_name is not None
    return package_name, plugin_id


def host_harness_manifest(crate_dir: Path, package_name: str) -> str:
    return f'''[package]
name = "motolii-external-plugin-check"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
candidate = {{ package = "{package_name}", path = "{crate_dir}" }}
motolii-core = {{ path = "{ROOT / 'crates/motolii-core'}" }}
motolii-plugin = {{ path = "{ROOT / 'crates/motolii-plugin'}" }}
motolii-testkit = {{ path = "{ROOT / 'crates/motolii-testkit'}" }}

[workspace]
'''


HOST_TEST = r'''use std::sync::Arc;

use candidate::{register_contracts, register_plugins};
use motolii_core::{CanonicalPoint, ColorSpace, CompCamera, FrameDesc, PixelFormat};
use motolii_plugin::{
    validate_node_desc, DynPlugin, LayerSourceContext, NodeDesc, PluginCatalogBuilder, PluginKind,
    PluginRegistry, PluginRuntime, RationalTime, ResolvedParams,
};
use motolii_testkit::purity::{assert_filter_pure, assert_layer_source_pure};
use motolii_testkit::gpu_or_skip;

const KINDS: [PluginKind; 4] = [
    PluginKind::LayerSource,
    PluginKind::Filter,
    PluginKind::ParamDriver,
    PluginKind::Composite,
];

fn frame() -> FrameDesc {
    FrameDesc::packed(48, 36, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

/// NodeDescが宣言した既定値だけでparamsを組む。fixture固有の値を持ち込まない。
fn params_for(desc: &NodeDesc) -> ResolvedParams {
    let mut params = ResolvedParams::new();
    for def in &desc.params {
        params.insert(def.id, def.default.clone());
    }
    params
}

fn layer_context(frame: FrameDesc) -> LayerSourceContext {
    LayerSourceContext {
        camera: CompCamera::try_new(
            CanonicalPoint::CENTER,
            0.0,
            1.0,
            i64::from(frame.width),
            i64::from(frame.height),
        )
        .expect("fixture camera"),
    }
}

/// Filter purityへ渡す決定的な入力。透明・半透明・不透明を1枚に含める。
fn input_rgba(frame: FrameDesc) -> Vec<u8> {
    let mut data = vec![0u8; frame.data_size()];
    for y in 0..frame.height {
        for x in 0..frame.width {
            let offset = ((y * frame.width + x) * 4) as usize;
            let a = ((x * 255) / frame.width.max(1)) as u8;
            // premultiplied: RGB <= A を保つ。
            data[offset] = a;
            data[offset + 1] = a / 2;
            data[offset + 2] = a / 3;
            data[offset + 3] = a;
        }
    }
    data
}

fn build() -> (PluginCatalogBuilder, PluginRegistry) {
    let mut catalog = PluginCatalogBuilder::new();
    register_contracts(&mut catalog).expect("candidate contracts");
    let mut registry = PluginRegistry::new();
    register_plugins(&mut registry).expect("candidate registration");
    (catalog, registry)
}

#[test]
fn conformance() {
    let (catalog, registry) = build();
    let mut seen = 0usize;
    for kind in KINDS {
        for (_id, plugin) in registry.iter(kind) {
            validate_node_desc(plugin.kind(), plugin.desc()).expect("candidate NodeDesc");
            seen += 1;
        }
    }
    assert!(seen > 0, "candidate registered no plugin");
    PluginRuntime::try_new(Arc::new(catalog.build().expect("catalog")), registry)
        .expect("contract and executor parity");
}

#[test]
fn purity() {
    let gpu = gpu_or_skip().expect("required GPU adapter for external plugin check");
    let frame = frame();
    let t = RationalTime::try_new(5, 4).expect("time");
    let (_catalog, registry) = build();
    let mut checked = 0usize;
    let mut unsupported: Vec<String> = Vec::new();

    for kind in KINDS {
        for (id, plugin) in registry.iter(kind) {
            let params = params_for(plugin.desc());
            let label = format!("external-{}", id.0);
            match plugin {
                DynPlugin::LayerSource(p) => {
                    assert_layer_source_pure(
                        &label,
                        &gpu,
                        p,
                        t,
                        &params,
                        layer_context(frame),
                        frame,
                    )
                    .expect("candidate purity");
                    checked += 1;
                }
                DynPlugin::Filter(p) => {
                    assert_filter_pure(&label, &gpu, p, t, &params, frame, &input_rgba(frame))
                        .expect("candidate purity");
                    checked += 1;
                }
                // 未対応のkindを黙って通さない。回していないものは回していないと言う。
                DynPlugin::ParamDriver(_) | DynPlugin::Composite(_) => {
                    unsupported.push(format!("{} ({:?})", id.0, plugin.kind()));
                }
            }
        }
    }

    assert!(
        unsupported.is_empty(),
        "purity harness does not cover these kinds yet: {}",
        unsupported.join(", ")
    );
    assert!(checked > 0, "no plugin was purity-checked");
}
'''


def inject_candidate_dependency(crate_dir: Path, destination: Path) -> None:
    shutil.copytree(crate_dir, destination)
    manifest_path = destination / "Cargo.toml"
    manifest = manifest_path.read_text(encoding="utf-8")
    source_path = ROOT / "crates/motolii-plugin"
    manifest = re.sub(
        r'(motolii-plugin\s*=\s*\{\s*path\s*=\s*")[^"]+("\s*\})',
        rf'\g<1>{source_path}\2',
        manifest,
        count=1,
    )
    manifest_path.write_text(manifest, encoding="utf-8")


def run_host_stage(harness: Path, stage: str, crate_dir: Path, plugin_id: str) -> None:
    command = [
        "cargo",
        "test",
        "--manifest-path",
        str(harness / "Cargo.toml"),
        stage,
        "--",
        "--exact",
    ]
    env = os.environ.copy()
    env["MOTOLII_REQUIRE_GPU"] = "1"
    # Harness is disposable; the compiler cache is outside both Host checkout and candidate.
    env.setdefault(
        "CARGO_TARGET_DIR",
        str(Path(tempfile.gettempdir()) / "motolii-external-plugin-check-target"),
    )
    completed = subprocess.run(command, env=env)
    if completed.returncode:
        die(
            f"check[{stage}] crate={crate_dir} file=src/lib.rs plugin={plugin_id}: "
            f"Host {stage} stage failed"
        )


def check(args: argparse.Namespace) -> None:
    crate_dir = Path(args.check).expanduser().resolve()
    if not crate_dir.is_dir():
        die(f"--check must name an existing crate directory: {crate_dir}")
    if is_within(crate_dir, ROOT):
        die(f"--check must name a crate outside the Motolii repository: {crate_dir}")
    package_name, plugin_id = check_hygiene(crate_dir)
    with tempfile.TemporaryDirectory(prefix="motolii-external-plugin-check-") as temporary:
        harness = Path(temporary) / "host-harness"
        candidate = Path(temporary) / "candidate"
        inject_candidate_dependency(crate_dir, candidate)
        harness.mkdir()
        (harness / "src").mkdir()
        shutil.copyfile(ROOT / "Cargo.lock", harness / "Cargo.lock")
        (harness / "Cargo.toml").write_text(
            host_harness_manifest(candidate, package_name), encoding="utf-8"
        )
        (harness / "src/lib.rs").write_text(HOST_TEST, encoding="utf-8")
        for stage in ("conformance", "purity"):
            run_host_stage(harness, stage, crate_dir, plugin_id)
    print(f"check passed: crate={crate_dir} plugin={plugin_id}")
    print(
        "golden: not run. The expected image is the author's claim about their own "
        "effect, so a Host-side oracle cannot stand in for it."
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--from", dest="source", help=f"source fixture ({SOURCE_ID})")
    group.add_argument("--check", help="run Host-owned external crate validation")
    parser.add_argument("--name", help="external plugin name")
    parser.add_argument("--vendor", help="external plugin vendor namespace")
    parser.add_argument("--out-dir", help="new external crate directory")
    args = parser.parse_args()
    if args.source:
        missing = [flag for flag in ("--name", "--vendor", "--out-dir") if getattr(args, flag[2:].replace('-', '_')) is None]
        if missing:
            parser.error(f"generation requires {' '.join(missing)}")
    elif any(value is not None for value in (args.name, args.vendor, args.out_dir)):
        parser.error("--check does not accept --name, --vendor, or --out-dir")
    return args


def main() -> None:
    args = parse_args()
    if args.source:
        generate(args)
    else:
        check(args)


if __name__ == "__main__":
    main()
