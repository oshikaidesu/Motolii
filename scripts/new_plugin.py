#!/usr/bin/env python3
"""INF-7e / M2E-10: plugin-authoring 規約準拠のスケルトンを生成する。

Usage:
  scripts/new-plugin.sh <kind> <name> [--vendor VENDOR] \\
      [--out PLUGIN.rs] [--out-test TEST.rs] [--plugin-import PATH]

kind: filter | layer_source | param_driver | composite
name: lowercase ascii + underscore (例: glow)

成果物は2つに分離する(M2E-10 P1):
  1. 製品コード → motolii-plugin に貼る(testkit 非参照。validate 単体のみ)
  2. testkit テスト → motolii-testkit/tests/ に置く(purity + ゴールデン)

`--out` のみ指定時、`--out-test` は `{stem}_test.rs` を同ディレクトリに書く。
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

KINDS = ("filter", "layer_source", "param_driver", "composite")
IDENT_RE = re.compile(r"^[a-z][a-z0-9_]*$")

KIND_SEG = {
    "filter": "filter",
    "layer_source": "layer_source",
    "param_driver": "param",
    "composite": "composite",
}

KIND_META = {
    "filter": {
        "trait": "FilterPlugin",
        "plugin_kind": "Filter",
        "category": "Utility",
        "min_inputs": 1,
        "max_inputs": 1,
        "param_id": "amount",
        "param_default": "Value::F64(1.0)",
        "param_type": "ValueType::F64",
        "purity_fn": "assert_filter_pure",
    },
    "layer_source": {
        "trait": "LayerSourcePlugin",
        "plugin_kind": "LayerSource",
        "category": "Generate",
        "min_inputs": 0,
        "max_inputs": 0,
        "param_id": "amount",
        "param_default": "Value::F64(1.0)",
        "param_type": "ValueType::F64",
        "purity_fn": "assert_layer_source_pure",
    },
    "param_driver": {
        "trait": "ParamDriverPlugin",
        "plugin_kind": "ParamDriver",
        "category": "Generate",
        "min_inputs": 0,
        "max_inputs": 0,
        "param_id": "amplitude",
        "param_default": "Value::F64(1.0)",
        "param_type": "ValueType::F64",
        "purity_fn": "assert_param_driver_pure",
    },
    "composite": {
        "trait": "CompositePlugin",
        "plugin_kind": "Composite",
        "category": "Composite",
        "min_inputs": 2,
        "max_inputs": 8,
        "param_id": "amount",
        "param_default": "Value::F64(1.0)",
        "param_type": "ValueType::F64",
        "purity_fn": "assert_composite_pure",
    },
}

# 機械走査用マーカー(検証テストが参照する)
FAIL_CLOSED_RGBA = 'expected.expect("scaffold golden: set expected RGBA oracle")'
FAIL_CLOSED_TRACK = 'expected.expect("scaffold golden: set expected Value sequence")'


def to_pascal(name: str) -> str:
    return "".join(p[:1].upper() + p[1:] for p in name.split("_") if p)


def method_body(kind: str, plugin_id: str) -> str:
    todo = f'TODO: implement {plugin_id}'
    if kind == "filter":
        return f"""    fn render(
        &self,
        _gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        _encoder: &mut wgpu::CommandEncoder,
        _ctx: &RenderCtx,
        _params: &ResolvedParams,
        _input: TextureRef<'_>,
        _output: TextureRef<'_>,
    ) -> Result<(), PluginError> {{
        // wgpu/WGSLのみ。パイプラインは pipelines.get_or_create_* でホストから借りる。
        // 出力は ctx.t + params + input だけで決める(純関数)。Quality は ctx.quality。
        // params は require_f64 等で読む(f64_or 禁止)。
        Err(PluginError::Render("{todo}".into()))
    }}"""
    if kind == "layer_source":
        return f"""    fn render(
        &self,
        _gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        _encoder: &mut wgpu::CommandEncoder,
        _t: RationalTime,
        _params: &ResolvedParams,
        _ctx: LayerSourceContext,
        _output: TextureRef<'_>,
    ) -> Result<(), PluginError> {{
        Err(PluginError::Render("{todo}".into()))
    }}"""
    if kind == "param_driver":
        return f"""    fn build_track(
        &self,
        _ctx: ParamDriverContext,
        _params: &ResolvedParams,
    ) -> Result<DataTrack, PluginError> {{
        Err(PluginError::Render("{todo}".into()))
    }}"""
    if kind == "composite":
        return f"""    fn render(
        &self,
        _gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        _encoder: &mut wgpu::CommandEncoder,
        _ctx: &RenderCtx,
        _params: &ResolvedParams,
        _inputs: &[TextureRef<'_>],
        _output: TextureRef<'_>,
    ) -> Result<(), PluginError> {{
        Err(PluginError::Render("{todo}".into()))
    }}"""
    raise ValueError(kind)


def render_plugin_source(*, kind: str, name: str, vendor: str) -> str:
    """motolii-plugin に貼る製品コード。motolii_testkit を参照しない。"""
    if kind not in KINDS:
        raise ValueError(f"unknown kind: {kind}")
    if not IDENT_RE.match(vendor):
        raise ValueError(f"vendor must be lowercase ascii id: {vendor}")
    if not IDENT_RE.match(name):
        raise ValueError(f"name must be lowercase ascii id: {name}")

    meta = KIND_META[kind]
    pascal = to_pascal(name)
    plugin_id = f"{vendor}.{KIND_SEG[kind]}.{name}"
    trait = meta["trait"]
    plugin_kind = meta["plugin_kind"]
    category = meta["category"]
    min_in = meta["min_inputs"]
    max_in = meta["max_inputs"]
    param_id = meta["param_id"]
    param_default = meta["param_default"]
    param_type = meta["param_type"]

    imports = [
        "use std::sync::OnceLock;",
        "",
    ]
    if kind == "layer_source":
        imports.append("use motolii_core::RationalTime;")
    if kind == "param_driver":
        imports.append("use motolii_eval::DataTrack;")
    imports.append("use motolii_eval::Value;")
    imports.append("use motolii_gpu::{GpuCtx, PipelineCache};")

    plugin_imports = [
        trait,
        "NodeDesc",
        "ParamDef",
        "PluginError",
        "PluginId",
        "PluginKind",
        "ResolvedParams",
        "ValueType",
        "validate_node_desc",
    ]
    if kind != "param_driver":
        plugin_imports.append("TextureRef")
    if kind == "layer_source":
        plugin_imports.append("LayerSourceContext")
    if kind == "param_driver":
        plugin_imports.append("ParamDriverContext")
    if kind in ("filter", "composite"):
        plugin_imports.append("RenderCtx")
    imports.append(f"use crate::{{ {', '.join(plugin_imports)} }};")

    body = method_body(kind, plugin_id)
    return f"""//! Scaffold plugin (INF-7e / M2E-10).
//! `motolii-plugin` クレート内に貼って肉付けする(自己クレート配置 → `use crate::...`)。
//! desc は validate_node_desc 通過状態から始まる。
//! purity/ゴールデンは別成果物(`--out-test`)を `motolii-testkit/tests/` へ置く。

{chr(10).join(imports)}

pub struct {pascal};

impl {trait} for {pascal} {{
    fn desc(&self) -> &NodeDesc {{
        static DESC: OnceLock<NodeDesc> = OnceLock::new();
        DESC.get_or_init(|| NodeDesc {{
            id: PluginId("{plugin_id}"),
            version: 1,
            display_name: "{pascal}",
            category: "{category}",
            tags: &["{name}", "scaffold"],
            // ParamDef 例(M2E-10)。id は安定、default 必須。不要なら削る。
            params: vec![ParamDef {{
                id: "{param_id}",
                value_type: {param_type},
                default: {param_default},
            }}],
            min_inputs: {min_in},
            max_inputs: {max_in},
        }})
    }}

{body}
}}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod scaffold_tests {{
    use super::*;

    #[test]
    fn generated_desc_passes_validate_node_desc() {{
        validate_node_desc(PluginKind::{plugin_kind}, {pascal}.desc()).unwrap();
    }}
}}
"""


def render_test_source(*, kind: str, name: str, vendor: str, plugin_import: str) -> str:
    """motolii-testkit/tests/ 向け。purity + fail-closed ゴールデン。"""
    if kind not in KINDS:
        raise ValueError(f"unknown kind: {kind}")
    if not IDENT_RE.match(vendor):
        raise ValueError(f"vendor must be lowercase ascii id: {vendor}")
    if not IDENT_RE.match(name):
        raise ValueError(f"name must be lowercase ascii id: {name}")
    if not plugin_import.strip():
        raise ValueError("plugin_import must be non-empty (e.g. motolii_plugin::scaffold_fixture::glow_filter)")

    meta = KIND_META[kind]
    pascal = to_pascal(name)
    param_id = meta["param_id"]
    purity_fn = meta["purity_fn"]

    header = f"""//! Scaffold tests for motolii-testkit/tests/ (INF-7e / M2E-10).
//! プラグイン製品コードとは別成果物。`use` は登録先モジュールに合わせる。
//! ゴールデンは期待オラクル未設定時に必ず失敗する(fail-closed)。

use {plugin_import}::{pascal};
"""

    if kind == "filter":
        body = f"""use motolii_core::{{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime}};
use motolii_eval::Value;
use motolii_gpu::{{download_rgba, upload_rgba, PipelineCache}};
use motolii_plugin::{{FilterPlugin, RenderCtx, ResolvedParams, TextureRef}};
use motolii_testkit::gpu_or_skip;
use motolii_testkit::purity::{purity_fn};
use motolii_testkit::{{assert_rgba_close, tol, RgbaImageDesc}};

#[test]
fn scaffold_is_pure() {{
    let Some(gpu) = gpu_or_skip() else {{ return }};
    let frame = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let input = vec![10u8; frame.data_size()];
    let mut params = ResolvedParams::new();
    params.insert("{param_id}", Value::F64(1.0));
    {purity_fn}(
        "scaffold-pure",
        &gpu,
        &{pascal},
        RationalTime::ZERO,
        &params,
        frame,
        &input,
    )
    .expect("purity");
}}

#[test]
fn scaffold_golden_stub() {{
    let Some(gpu) = gpu_or_skip() else {{ return }};
    let frame = FrameDesc::packed(4, 2, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let input_rgba = vec![10u8; frame.data_size()];
    let input = upload_rgba(&gpu, &frame, &input_rgba);
    let output = gpu.device.create_texture(&wgpu::TextureDescriptor {{
        label: Some("scaffold-golden-out"),
        size: wgpu::Extent3d {{
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        }},
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }});
    let mut params = ResolvedParams::new();
    params.insert("{param_id}", Value::F64(1.0));
    let mut pipelines = PipelineCache::new();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {{
            label: Some("scaffold-golden"),
        }});
    {pascal}
        .render(
            &gpu,
            &mut pipelines,
            &mut encoder,
            &RenderCtx::new(RationalTime::ZERO, Quality::FINAL),
            &params,
            TextureRef {{
                texture: &input,
                desc: frame,
            }},
            TextureRef {{
                texture: &output,
                desc: frame,
            }},
        )
        .expect("render");
    gpu.queue.submit(std::iter::once(encoder.finish()));
    let actual = download_rgba(&gpu, &output).expect("download");
    // Some(&[r,g,b,a, ...]) に置換するまで fail-closed。自己参照(actual の複製を期待値に使う)は禁止。
    let expected: Option<&[u8]> = None;
    assert_rgba_close(
        "scaffold-golden",
        RgbaImageDesc {{
            width: frame.width,
            height: frame.height,
        }},
        &actual,
        {FAIL_CLOSED_RGBA},
        tol::GPU_RASTER,
    );
}}
"""
    elif kind == "layer_source":
        body = f"""use motolii_core::{{ColorSpace, CompCamera, FrameDesc, PixelFormat, RationalTime}};
use motolii_eval::Value;
use motolii_gpu::{{download_rgba, PipelineCache}};
use motolii_plugin::{{LayerSourceContext, LayerSourcePlugin, ResolvedParams, TextureRef}};
use motolii_testkit::gpu_or_skip;
use motolii_testkit::purity::{purity_fn};
use motolii_testkit::{{assert_rgba_close, tol, RgbaImageDesc}};

#[test]
fn scaffold_is_pure() {{
    let Some(gpu) = gpu_or_skip() else {{ return }};
    let frame = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let mut params = ResolvedParams::new();
    params.insert("{param_id}", Value::F64(1.0));
    {purity_fn}(
        "scaffold-pure",
        &gpu,
        &{pascal},
        RationalTime::ZERO,
        &params,
        LayerSourceContext {{
            camera: CompCamera::DEFAULT,
        }},
        frame,
    )
    .expect("purity");
}}

#[test]
fn scaffold_golden_stub() {{
    let Some(gpu) = gpu_or_skip() else {{ return }};
    let frame = FrameDesc::packed(4, 2, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let output = gpu.device.create_texture(&wgpu::TextureDescriptor {{
        label: Some("scaffold-golden-out"),
        size: wgpu::Extent3d {{
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        }},
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }});
    let mut params = ResolvedParams::new();
    params.insert("{param_id}", Value::F64(1.0));
    let mut pipelines = PipelineCache::new();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {{
            label: Some("scaffold-golden"),
        }});
    {pascal}
        .render(
            &gpu,
            &mut pipelines,
            &mut encoder,
            RationalTime::ZERO,
            &params,
            LayerSourceContext {{
                camera: CompCamera::DEFAULT,
            }},
            TextureRef {{
                texture: &output,
                desc: frame,
            }},
        )
        .expect("render");
    gpu.queue.submit(std::iter::once(encoder.finish()));
    let actual = download_rgba(&gpu, &output).expect("download");
    let expected: Option<&[u8]> = None;
    assert_rgba_close(
        "scaffold-golden",
        RgbaImageDesc {{
            width: frame.width,
            height: frame.height,
        }},
        &actual,
        {FAIL_CLOSED_RGBA},
        tol::GPU_RASTER,
    );
}}
"""
    elif kind == "composite":
        body = f"""use motolii_core::{{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime}};
use motolii_eval::Value;
use motolii_gpu::{{download_rgba, upload_rgba, PipelineCache}};
use motolii_plugin::{{CompositePlugin, RenderCtx, ResolvedParams, TextureRef}};
use motolii_testkit::gpu_or_skip;
use motolii_testkit::purity::{purity_fn};
use motolii_testkit::{{assert_rgba_close, tol, RgbaImageDesc}};

#[test]
fn scaffold_is_pure() {{
    let Some(gpu) = gpu_or_skip() else {{ return }};
    let frame = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let a = vec![10u8; frame.data_size()];
    let b = vec![20u8; frame.data_size()];
    let mut params = ResolvedParams::new();
    params.insert("{param_id}", Value::F64(1.0));
    {purity_fn}(
        "scaffold-pure",
        &gpu,
        &{pascal},
        RationalTime::ZERO,
        &params,
        frame,
        &[&a, &b],
    )
    .expect("purity");
}}

#[test]
fn scaffold_golden_stub() {{
    let Some(gpu) = gpu_or_skip() else {{ return }};
    let frame = FrameDesc::packed(4, 2, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let a_rgba = vec![10u8; frame.data_size()];
    let b_rgba = vec![20u8; frame.data_size()];
    let a = upload_rgba(&gpu, &frame, &a_rgba);
    let b = upload_rgba(&gpu, &frame, &b_rgba);
    let output = gpu.device.create_texture(&wgpu::TextureDescriptor {{
        label: Some("scaffold-golden-out"),
        size: wgpu::Extent3d {{
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        }},
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }});
    let mut params = ResolvedParams::new();
    params.insert("{param_id}", Value::F64(1.0));
    let mut pipelines = PipelineCache::new();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {{
            label: Some("scaffold-golden"),
        }});
    let inputs = [
        TextureRef {{
            texture: &a,
            desc: frame,
        }},
        TextureRef {{
            texture: &b,
            desc: frame,
        }},
    ];
    {pascal}
        .render(
            &gpu,
            &mut pipelines,
            &mut encoder,
            &RenderCtx::new(RationalTime::ZERO, Quality::FINAL),
            &params,
            &inputs,
            TextureRef {{
                texture: &output,
                desc: frame,
            }},
        )
        .expect("render");
    gpu.queue.submit(std::iter::once(encoder.finish()));
    let actual = download_rgba(&gpu, &output).expect("download");
    let expected: Option<&[u8]> = None;
    assert_rgba_close(
        "scaffold-golden",
        RgbaImageDesc {{
            width: frame.width,
            height: frame.height,
        }},
        &actual,
        {FAIL_CLOSED_RGBA},
        tol::GPU_RASTER,
    );
}}
"""
    elif kind == "param_driver":
        body = f"""use motolii_core::{{Fps, RationalTime}};
use motolii_eval::{{DataTrack, Value}};
use motolii_plugin::{{ParamDriverContext, ParamDriverPlugin, ResolvedParams}};
use motolii_testkit::purity::{purity_fn};

#[test]
fn scaffold_is_pure() {{
    let mut params = ResolvedParams::new();
    params.insert("{param_id}", Value::F64(1.0));
    {purity_fn}(
        "scaffold-pure",
        &{pascal},
        ParamDriverContext {{
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(1),
            sample_rate: Fps::try_new(8, 1).unwrap(),
        }},
        &params,
    )
    .expect("purity");
}}

#[test]
fn scaffold_golden_stub() {{
    let mut params = ResolvedParams::new();
    params.insert("{param_id}", Value::F64(1.0));
    let track = {pascal}
        .build_track(
            ParamDriverContext {{
                start: RationalTime::ZERO,
                duration: RationalTime::from_seconds(1),
                sample_rate: Fps::try_new(8, 1).unwrap(),
            }},
            &params,
        )
        .expect("build_track");
    // Some(fixed_track) に置換するまで fail-closed。トラックを捨てるだけの無assertは禁止。
    let expected: Option<DataTrack> = None;
    assert_eq!(track, {FAIL_CLOSED_TRACK});
}}
"""
    else:
        raise ValueError(kind)

    return header + "\n" + body


def scaffold_desc_fields(*, kind: str, name: str, vendor: str) -> dict:
    meta = KIND_META[kind]
    return {
        "id": f"{vendor}.{KIND_SEG[kind]}.{name}",
        "version": 1,
        "display_name": to_pascal(name),
        "category": meta["category"],
        "tags": [name, "scaffold"],
        "min_inputs": meta["min_inputs"],
        "max_inputs": meta["max_inputs"],
        "plugin_kind": meta["plugin_kind"],
        "param_id": meta["param_id"],
        "purity_fn": meta["purity_fn"],
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=KINDS)
    parser.add_argument("name", help="lowercase ascii + underscore")
    parser.add_argument("--vendor", default="core")
    parser.add_argument("--out", type=Path, help="plugin output path (product code)")
    parser.add_argument(
        "--out-test",
        type=Path,
        help="testkit test output path (default: {{out.stem}}_test.rs beside --out)",
    )
    parser.add_argument(
        "--plugin-import",
        default="motolii_plugin::reference",
        help="Rust path used in test `use PATH::Type` (change to the module that exports the plugin type)",
    )
    parser.add_argument(
        "--print-desc-json",
        action="store_true",
        help="print desc fields as JSON instead of Rust (for tests)",
    )
    parser.add_argument(
        "--print-test",
        action="store_true",
        help="print testkit test source to stdout (instead of plugin)",
    )
    args = parser.parse_args(argv)

    try:
        if args.print_desc_json:
            import json

            print(json.dumps(scaffold_desc_fields(kind=args.kind, name=args.name, vendor=args.vendor)))
            return 0

        plugin_src = render_plugin_source(kind=args.kind, name=args.name, vendor=args.vendor)
        test_src = render_test_source(
            kind=args.kind,
            name=args.name,
            vendor=args.vendor,
            plugin_import=args.plugin_import,
        )
    except ValueError as e:
        print(e, file=sys.stderr)
        return 1

    if args.out:
        out_test = args.out_test
        if out_test is None:
            out_test = args.out.with_name(f"{args.out.stem}_test.rs")
        args.out.parent.mkdir(parents=True, exist_ok=True)
        out_test.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(plugin_src if plugin_src.endswith("\n") else plugin_src + "\n", encoding="utf-8")
        out_test.write_text(test_src if test_src.endswith("\n") else test_src + "\n", encoding="utf-8")
        print(f"wrote plugin {args.out}", file=sys.stderr)
        print(f"wrote test   {out_test}", file=sys.stderr)
        return 0

    if args.print_test:
        sys.stdout.write(test_src if test_src.endswith("\n") else test_src + "\n")
    else:
        sys.stdout.write(plugin_src if plugin_src.endswith("\n") else plugin_src + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
