//! `core.layer_source.radial_repeater` version 1 — 外部 LayerSource crate (VSM-A3-2)。

use std::sync::OnceLock;

use motolii_plugin::bytemuck;
use motolii_plugin::wgpu;
use motolii_plugin::F64Domain;
use motolii_plugin::GpuCtx;
use motolii_plugin::LayerSourceContext;
use motolii_plugin::LayerSourcePlugin;
use motolii_plugin::NodeDesc;
use motolii_plugin::ParamDef;
use motolii_plugin::PipelineCache;
use motolii_plugin::PipelineCacheKey;
use motolii_plugin::PluginContract;
use motolii_plugin::PluginError;
use motolii_plugin::PluginId;
use motolii_plugin::PluginKind;
use motolii_plugin::RationalTime;
use motolii_plugin::ResolvedParams;
use motolii_plugin::TextureRef;
use motolii_plugin::Value;
use motolii_plugin::ValueType;

const PLUGIN_ID: &str = "core.layer_source.radial_repeater";

pub static RADIAL_REPEATER_LAYER_SOURCE: RadialRepeaterLayerSource = RadialRepeaterLayerSource;

pub fn radial_repeater_contract() -> PluginContract {
    PluginContract {
        kind: PluginKind::LayerSource,
        node: radial_repeater_desc().clone(),
        migrations: vec![],
    }
}

pub struct RadialRepeaterLayerSource;

struct ValidatedParams {
    count: f32,
    radius: f32,
    dot_radius: f32,
    phase: f32,
    angular_speed: f32,
    color: [f32; 4],
}

impl LayerSourcePlugin for RadialRepeaterLayerSource {
    fn desc(&self) -> &NodeDesc {
        radial_repeater_desc()
    }

    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        t: RationalTime,
        params: &ResolvedParams,
        _ctx: LayerSourceContext,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        if output.desc.width == 0 || output.desc.height == 0 {
            return Err(PluginError::Render(
                "output dimensions must be non-zero".into(),
            ));
        }

        let validated = validated_params(params)?;
        let width = output.desc.width as f32;
        let height = output.desc.height as f32;
        let t_seconds = t.as_seconds_f64() as f32;

        let uniform: [f32; 16] = [
            validated.count,
            validated.radius,
            validated.dot_radius,
            validated.phase,
            validated.angular_speed,
            t_seconds,
            width,
            height,
            validated.color[0],
            validated.color[1],
            validated.color[2],
            validated.color[3],
            0.0,
            0.0,
            0.0,
            0.0,
        ];

        let cached = pipelines.get_or_create_fullscreen_uniform16(
            gpu,
            PipelineCacheKey {
                id: PLUGIN_ID,
                wgsl: RADIAL_REPEATER_WGSL,
            },
        );
        gpu.queue
            .write_buffer(&cached.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("core.layer_source.radial_repeater.bg"),
            layout: &cached.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: cached.uniform_buffer.as_entire_binding(),
            }],
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("core.layer_source.radial_repeater.pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&cached.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        Ok(())
    }
}

fn radial_repeater_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId(PLUGIN_ID),
        version: 1,
        display_name: "Radial Repeater",
        category: "Generate",
        tags: &["radial", "repeater", "generate"],
        params: vec![
            ParamDef {
                id: "count",
                value_type: ValueType::F64,
                default: Value::F64(12.0),
                f64_domain: Some(F64Domain::new(Some(1.0), Some(64.0), true)),
            },
            ParamDef {
                id: "radius",
                value_type: ValueType::F64,
                default: Value::F64(0.30),
                f64_domain: Some(F64Domain::new(Some(0.0), None, false)),
            },
            ParamDef {
                id: "dot_radius",
                value_type: ValueType::F64,
                default: Value::F64(0.04),
                f64_domain: Some(F64Domain::new(Some(0.0), None, false)),
            },
            ParamDef {
                id: "phase",
                value_type: ValueType::F64,
                default: Value::F64(0.0),
                f64_domain: None,
            },
            ParamDef {
                id: "angular_speed",
                value_type: ValueType::F64,
                default: Value::F64(0.0),
                f64_domain: None,
            },
            ParamDef {
                id: "color",
                value_type: ValueType::Color,
                default: Value::Color([1.0, 1.0, 1.0, 1.0]),
                f64_domain: None,
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
    })
}

fn validated_params(params: &ResolvedParams) -> Result<ValidatedParams, PluginError> {
    let count = require_count(params.require_f64(PLUGIN_ID, "count")?)?;
    let radius = require_non_negative_f64(
        PLUGIN_ID,
        "radius",
        params.require_f64(PLUGIN_ID, "radius")?,
    )?;
    let dot_radius = require_non_negative_f64(
        PLUGIN_ID,
        "dot_radius",
        params.require_f64(PLUGIN_ID, "dot_radius")?,
    )?;
    let phase = require_finite_f64(PLUGIN_ID, "phase", params.require_f64(PLUGIN_ID, "phase")?)?;
    let angular_speed = require_finite_f64(
        PLUGIN_ID,
        "angular_speed",
        params.require_f64(PLUGIN_ID, "angular_speed")?,
    )?;
    let color = require_unit_color(PLUGIN_ID, params.require_color(PLUGIN_ID, "color")?)?;

    Ok(ValidatedParams {
        count: count as f32,
        radius: radius as f32,
        dot_radius: dot_radius as f32,
        phase: phase as f32,
        angular_speed: angular_speed as f32,
        color: [
            color[0] as f32,
            color[1] as f32,
            color[2] as f32,
            color[3] as f32,
        ],
    })
}

fn require_finite_f64(plugin: &str, id: &str, value: f64) -> Result<f64, PluginError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PluginError::Param {
            plugin: plugin.to_string(),
            id: id.to_string(),
            expected: "F64".to_string(),
            got: "non-finite".to_string(),
        })
    }
}

fn require_count(value: f64) -> Result<u32, PluginError> {
    let value = require_finite_f64(PLUGIN_ID, "count", value)?;
    if value.fract() != 0.0 {
        return Err(PluginError::Param {
            plugin: PLUGIN_ID.to_string(),
            id: "count".to_string(),
            expected: "integer in 1..=64".to_string(),
            got: "non-integer".to_string(),
        });
    }
    if !(1.0..=64.0).contains(&value) {
        return Err(PluginError::Param {
            plugin: PLUGIN_ID.to_string(),
            id: "count".to_string(),
            expected: "integer in 1..=64".to_string(),
            got: "out of range".to_string(),
        });
    }
    Ok(value as u32)
}

fn require_non_negative_f64(plugin: &str, id: &str, value: f64) -> Result<f64, PluginError> {
    let value = require_finite_f64(plugin, id, value)?;
    if value < 0.0 {
        return Err(PluginError::Param {
            plugin: plugin.to_string(),
            id: id.to_string(),
            expected: "non-negative F64".to_string(),
            got: "negative".to_string(),
        });
    }
    Ok(value)
}

fn require_unit_color(plugin: &str, color: [f64; 4]) -> Result<[f64; 4], PluginError> {
    for (index, component) in color.into_iter().enumerate() {
        if !component.is_finite() || !(0.0..=1.0).contains(&component) {
            return Err(PluginError::Param {
                plugin: plugin.to_string(),
                id: "color".to_string(),
                expected: "finite color component in 0..=1".to_string(),
                got: format!("invalid component[{index}]"),
            });
        }
    }
    Ok(color)
}

const RADIAL_REPEATER_WGSL: &str = r#"
struct Params {
    data: array<vec4<f32>, 4>,
};

@group(0) @binding(0) var<uniform> params: Params;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let p = positions[vertex_index];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let count = params.data[0].x;
    let radius = params.data[0].y;
    let dot_radius = params.data[0].z;
    let phase = params.data[0].w;
    let angular_speed = params.data[1].x;
    let t = params.data[1].y;
    let width = params.data[1].z;
    let height = params.data[1].w;
    let color = params.data[2];
    let r = color.x;
    let g = color.y;
    let b = color.z;
    let a = color.w;

    let px = (in.pos.x - width * 0.5) / height;
    let py = (height * 0.5 - in.pos.y) / height;
    let p = vec2<f32>(px, py);

    let TAU = 6.28318530718;
    var d = 1e10;
    for (var i = 0u; i < 64u; i = i + 1u) {
        if (f32(i) >= count) {
            break;
        }
        let theta = phase + angular_speed * t + TAU * f32(i) / count;
        let center = vec2<f32>(radius * cos(theta), radius * sin(theta));
        d = min(d, length(p - center) - dot_radius);
    }
    let w = 1.0 / height;
    let C = clamp(0.5 - d / w, 0.0, 1.0);
    return vec4<f32>(r * a * C, g * a * C, b * a * C, a * C);
}
"#;

#[cfg(test)]
mod tests {
    use motolii_plugin::validate_node_desc;

    use super::*;

    #[test]
    fn radial_repeater_contract_matches_desc() {
        let contract = radial_repeater_contract();
        assert_eq!(contract.kind, PluginKind::LayerSource);
        assert_eq!(contract.node.id.0, PLUGIN_ID);
        assert_eq!(contract.node.version, 1);
        assert_eq!(contract.node.display_name, "Radial Repeater");
        assert_eq!(contract.node.category, "Generate");
        assert_eq!(contract.node.tags, &["radial", "repeater", "generate"]);
        assert_eq!(contract.node.min_inputs, 0);
        assert_eq!(contract.node.max_inputs, 0);
        assert_eq!(contract.migrations, vec![]);
        assert_eq!(contract.node.params.len(), 6);
        assert!(
            validate_node_desc(PluginKind::LayerSource, RADIAL_REPEATER_LAYER_SOURCE.desc())
                .is_ok()
        );
    }

    #[test]
    fn rejects_non_integer_count() {
        let mut params = ResolvedParams::new();
        params.insert("count", Value::F64(12.5));
        params.insert("radius", Value::F64(0.30));
        params.insert("dot_radius", Value::F64(0.04));
        params.insert("phase", Value::F64(0.0));
        params.insert("angular_speed", Value::F64(0.0));
        params.insert("color", Value::Color([1.0, 1.0, 1.0, 1.0]));

        let err = validated_params(&params);
        assert!(matches!(
            err,
            Err(PluginError::Param {
                ref id,
                ref got,
                ..
            }) if id == "count" && got == "non-integer"
        ));
    }

    #[test]
    fn rejects_count_out_of_range() {
        let mut params = ResolvedParams::new();
        params.insert("count", Value::F64(0.0));
        params.insert("radius", Value::F64(0.30));
        params.insert("dot_radius", Value::F64(0.04));
        params.insert("phase", Value::F64(0.0));
        params.insert("angular_speed", Value::F64(0.0));
        params.insert("color", Value::Color([1.0, 1.0, 1.0, 1.0]));

        let err = validated_params(&params);
        assert!(matches!(
            err,
            Err(PluginError::Param {
                ref id,
                ref got,
                ..
            }) if id == "count" && got == "out of range"
        ));
    }

    #[test]
    fn rejects_negative_radius() {
        let mut params = ResolvedParams::new();
        params.insert("count", Value::F64(12.0));
        params.insert("radius", Value::F64(-0.1));
        params.insert("dot_radius", Value::F64(0.04));
        params.insert("phase", Value::F64(0.0));
        params.insert("angular_speed", Value::F64(0.0));
        params.insert("color", Value::Color([1.0, 1.0, 1.0, 1.0]));

        let err = validated_params(&params);
        assert!(matches!(
            err,
            Err(PluginError::Param {
                ref id,
                ref got,
                ..
            }) if id == "radius" && got == "negative"
        ));
    }

    #[test]
    fn rejects_non_finite_phase() {
        let mut params = ResolvedParams::new();
        params.insert("count", Value::F64(12.0));
        params.insert("radius", Value::F64(0.30));
        params.insert("dot_radius", Value::F64(0.04));
        params.insert("phase", Value::F64(f64::NAN));
        params.insert("angular_speed", Value::F64(0.0));
        params.insert("color", Value::Color([1.0, 1.0, 1.0, 1.0]));

        let err = validated_params(&params);
        assert!(matches!(
            err,
            Err(PluginError::Param {
                ref id,
                ref got,
                ..
            }) if id == "phase" && got == "non-finite"
        ));
    }

    #[test]
    fn rejects_color_out_of_range() {
        let mut params = ResolvedParams::new();
        params.insert("count", Value::F64(12.0));
        params.insert("radius", Value::F64(0.30));
        params.insert("dot_radius", Value::F64(0.04));
        params.insert("phase", Value::F64(0.0));
        params.insert("angular_speed", Value::F64(0.0));
        params.insert("color", Value::Color([1.1, 1.0, 1.0, 1.0]));

        let err = validated_params(&params);
        assert!(matches!(
            err,
            Err(PluginError::Param {
                ref id,
                ref got,
                ..
            }) if id == "color" && got == "invalid component[0]"
        ));
    }
}
