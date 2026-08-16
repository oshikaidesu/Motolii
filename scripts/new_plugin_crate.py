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
        r"[a-z][a-z0-9_]*\.layer_source\.[a-z][a-z0-9_]*", plugin_id
    ):
        reject("plugin identity must be vendor.layer_source.name", crate_dir / "src/lib.rs")
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

use candidate::{radial_repeater_contract, RADIAL_REPEATER_LAYER_SOURCE};
use motolii_core::{CanonicalPoint, ColorSpace, CompCamera, FrameDesc, PixelFormat};
use motolii_plugin::{
    validate_node_desc, LayerSourceContext, LayerSourcePlugin, PluginCatalogBuilder, PluginKind,
    PluginRegistry, PluginRuntime, RationalTime, ResolvedParams, Value,
};
use motolii_testkit::purity::{
    assert_layer_source_pure, render_layer_source_rgba, LayerSourceRenderRequest,
};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

fn frame() -> FrameDesc {
    FrameDesc::packed(48, 36, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn params() -> ResolvedParams {
    let mut params = ResolvedParams::new();
    params.insert("count", Value::F64(7.0));
    params.insert("radius", Value::F64(0.27));
    params.insert("dot_radius", Value::F64(0.055));
    params.insert("phase", Value::F64(0.35));
    params.insert("angular_speed", Value::F64(0.85));
    params.insert("color", Value::Color([0.82, 0.41, 0.19, 0.72]));
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

#[test]
fn conformance() {
    validate_node_desc(PluginKind::LayerSource, RADIAL_REPEATER_LAYER_SOURCE.desc())
        .expect("candidate NodeDesc");
    let contract = radial_repeater_contract();
    let mut catalog = PluginCatalogBuilder::new();
    catalog.register(contract).expect("candidate contract");
    let mut registry = PluginRegistry::new();
    registry
        .register_layer_source(&RADIAL_REPEATER_LAYER_SOURCE)
        .expect("candidate registration");
    PluginRuntime::try_new(Arc::new(catalog.build().expect("catalog")), registry)
        .expect("contract and executor parity");
}

#[test]
fn purity() {
    let gpu = gpu_or_skip().expect("required GPU adapter for external plugin check");
    let frame = frame();
    let params = params();
    assert_layer_source_pure(
        "external-radial-purity",
        &gpu,
        &RADIAL_REPEATER_LAYER_SOURCE,
        RationalTime::try_new(5, 4).expect("time"),
        &params,
        layer_context(frame),
        frame,
    )
    .expect("candidate purity");
}

#[test]
fn golden() {
    let gpu = gpu_or_skip().expect("required GPU adapter for external plugin check");
    let frame = frame();
    let params = params();
    let t = RationalTime::try_new(5, 4).expect("time");
    let mut pipelines = motolii_plugin::PipelineCache::new();
    let actual = render_layer_source_rgba(
        "external-radial-golden",
        &gpu,
        &mut pipelines,
        &LayerSourceRenderRequest {
            plugin: &RADIAL_REPEATER_LAYER_SOURCE,
            t,
            params: &params,
            ctx: layer_context(frame),
            frame,
        },
    )
    .expect("candidate render");
    let expected = radial_oracle(frame, t);
    assert!(expected.iter().any(|value| *value != 0), "golden must not default transparent");
    assert_rgba_close(
        "external-radial-golden",
        RgbaImageDesc { width: frame.width, height: frame.height },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

fn radial_oracle(frame: FrameDesc, t: RationalTime) -> Vec<u8> {
    let count = 7u32;
    let radius = 0.27;
    let dot_radius = 0.055;
    let phase = 0.35;
    let angular_speed = 0.85;
    let color = [0.82, 0.41, 0.19, 0.72];
    let width = f64::from(frame.width);
    let height = f64::from(frame.height);
    let mut output = vec![0; frame.data_size()];
    for y in 0..frame.height {
        for x in 0..frame.width {
            let px = (f64::from(x) + 0.5 - width / 2.0) / height;
            let py = (height / 2.0 - (f64::from(y) + 0.5)) / height;
            let mut distance = f64::INFINITY;
            for index in 0..count {
                let theta = phase
                    + angular_speed * t.as_seconds_f64()
                    + 2.0 * std::f64::consts::PI * f64::from(index) / f64::from(count);
                let center = (radius * theta.cos(), radius * theta.sin());
                distance = distance.min(((px - center.0).powi(2) + (py - center.1).powi(2)).sqrt() - dot_radius);
            }
            let coverage = (0.5 - distance / (1.0 / height)).clamp(0.0, 1.0);
            let pixel = [
                color[0] * color[3] * coverage,
                color[1] * color[3] * coverage,
                color[2] * color[3] * coverage,
                color[3] * coverage,
            ];
            let offset = ((y * frame.width + x) * 4) as usize;
            for (channel, value) in pixel.into_iter().enumerate() {
                output[offset + channel] = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    }
    output
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
        for stage in ("conformance", "purity", "golden"):
            run_host_stage(harness, stage, crate_dir, plugin_id)
    print(f"check passed: crate={crate_dir} plugin={plugin_id}")


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
