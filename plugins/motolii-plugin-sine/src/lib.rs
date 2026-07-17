//! `core.param.sine` version 2 — 外部参照ParamDriver crate実証(VSM-A2)。

use std::f64::consts::TAU;
use std::sync::OnceLock;

use motolii_plugin::DataTrack;
use motolii_plugin::Fps;
use motolii_plugin::MigrationOp;
use motolii_plugin::MigrationStep;
use motolii_plugin::NodeDesc;
use motolii_plugin::ParamDef;
use motolii_plugin::ParamDriverContext;
use motolii_plugin::ParamDriverPlugin;
use motolii_plugin::PluginContract;
use motolii_plugin::PluginError;
use motolii_plugin::PluginId;
use motolii_plugin::PluginKind;
use motolii_plugin::RationalTime;
use motolii_plugin::ResolvedParams;
use motolii_plugin::Value;
use motolii_plugin::ValueType;

pub static SINE_PARAM_DRIVER: SineParamDriver = SineParamDriver;

pub fn sine_contract() -> PluginContract {
    PluginContract {
        kind: PluginKind::ParamDriver,
        node: sine_param_desc().clone(),
        migrations: vec![MigrationStep {
            from_version: 1,
            to_version: 2,
            ops: vec![MigrationOp::RenameParam {
                from: "amp",
                to: "amplitude",
            }],
        }],
    }
}

pub struct SineParamDriver;

impl ParamDriverPlugin for SineParamDriver {
    fn desc(&self) -> &NodeDesc {
        sine_param_desc()
    }

    fn build_track(
        &self,
        ctx: ParamDriverContext,
        params: &ResolvedParams,
    ) -> Result<DataTrack, PluginError> {
        let amplitude = require_finite_f64(
            "core.param.sine",
            "amplitude",
            params.require_f64("core.param.sine", "amplitude")?,
        )?;
        let frequency_hz = require_finite_f64(
            "core.param.sine",
            "frequency_hz",
            params.require_f64("core.param.sine", "frequency_hz")?,
        )?;
        let offset = require_finite_f64(
            "core.param.sine",
            "offset",
            params.require_f64("core.param.sine", "offset")?,
        )?;
        let count_i64 = sample_count(ctx.duration, ctx.sample_rate)?;
        let values = (0..count_i64)
            .map(|i| {
                let secs = i as f64 / ctx.sample_rate.as_f64();
                Value::F64(offset + amplitude * (TAU * frequency_hz * secs).sin())
            })
            .collect();
        Ok(DataTrack {
            start: ctx.start,
            sample_rate: ctx.sample_rate,
            values,
        })
    }
}

fn sine_param_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.param.sine"),
        version: 2,
        display_name: "Sine",
        category: "Generate",
        tags: &["lfo", "oscillator", "reference"],
        params: vec![
            ParamDef {
                id: "amplitude",
                value_type: ValueType::F64,
                default: Value::F64(1.0),
                f64_domain: None,
            },
            ParamDef {
                id: "frequency_hz",
                value_type: ValueType::F64,
                default: Value::F64(1.0),
                f64_domain: None,
            },
            ParamDef {
                id: "offset",
                value_type: ValueType::F64,
                default: Value::F64(0.0),
                f64_domain: None,
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
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

/// 半開 `[0, duration)` 上の等間隔サンプル数。`duration` は総尺(M2E-17)。
fn sample_count(duration: RationalTime, sample_rate: Fps) -> Result<i64, PluginError> {
    if duration <= RationalTime::ZERO {
        return Ok(0);
    }
    let (idx, frac) = duration.try_to_sample_index(sample_rate)?;
    if frac == 0.0 {
        Ok(idx)
    } else {
        let origin = RationalTime::try_from_frame(-1, sample_rate)?;
        Ok(duration.try_to_sample_index_since(origin, sample_rate)?.0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::collections::HashMap;

    use super::*;

    #[test]
    fn build_track_rejects_non_finite_amplitude() {
        let mut params = ResolvedParams::new();
        params.insert("amplitude", Value::F64(f64::NAN));
        params.insert("frequency_hz", Value::F64(1.0));
        params.insert("offset", Value::F64(0.0));

        let err = SINE_PARAM_DRIVER
            .build_track(
                ParamDriverContext {
                    start: RationalTime::ZERO,
                    duration: RationalTime::from_seconds(1),
                    sample_rate: Fps::try_new(4, 1).unwrap(),
                },
                &params,
            )
            .unwrap_err();
        assert!(
            matches!(
                err,
                PluginError::Param {
                    ref plugin,
                    ref id,
                    ref expected,
                    ref got,
                } if plugin == "core.param.sine"
                    && id == "amplitude"
                    && expected == "F64"
                    && got == "non-finite"
            ),
            "{err:?}"
        );
    }

    #[test]
    fn sine_param_driver_builds_typed_data_track() {
        let mut params = ResolvedParams::new();
        params.insert("amplitude", Value::F64(2.0));
        params.insert("frequency_hz", Value::F64(1.0));
        params.insert("offset", Value::F64(10.0));

        let track = SINE_PARAM_DRIVER
            .build_track(
                ParamDriverContext {
                    start: RationalTime::ZERO,
                    duration: RationalTime::from_seconds(1),
                    sample_rate: Fps::try_new(4, 1).unwrap(),
                },
                &params,
            )
            .unwrap();

        assert_eq!(track.values.len(), 4);
        assert_eq!(track.values[0], Value::F64(10.0));
        assert!((track.values[1].as_f64().unwrap() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn sample_count_is_half_open_excluding_end() {
        assert_eq!(
            sample_count(RationalTime::from_seconds(1), Fps::try_new(4, 1).unwrap()).unwrap(),
            4
        );
        let fps = Fps::try_new(30, 1).unwrap();
        let duration = RationalTime::try_from_frame(90, fps).unwrap();
        assert_eq!(sample_count(duration, fps).unwrap(), 90);
        assert_eq!(RationalTime::try_from_frame(90, fps).unwrap(), duration);
        assert!(RationalTime::try_from_frame(89, fps).unwrap() < duration);
    }

    #[test]
    fn sample_count_ceil_keeps_in_range_samples_off_grid() {
        let rate = Fps::try_new(4, 1).unwrap();
        assert_eq!(
            sample_count(RationalTime::try_new(3, 10).unwrap(), rate).unwrap(),
            2
        );
        assert_eq!(
            sample_count(RationalTime::try_new(1, 10).unwrap(), rate).unwrap(),
            1
        );
        assert_eq!(
            sample_count(RationalTime::from_seconds(1), rate).unwrap(),
            4
        );
    }

    #[test]
    fn sine_contract_declares_amp_to_amplitude_migration() {
        let contract = sine_contract();
        assert_eq!(contract.node.version, 2);
        assert_eq!(
            contract.migrations,
            vec![MigrationStep {
                from_version: 1,
                to_version: 2,
                ops: vec![MigrationOp::RenameParam {
                    from: "amp",
                    to: "amplitude",
                }],
            }]
        );
    }

    #[test]
    fn resolve_params_fills_defaults_and_rejects_unknown_or_mismatch() {
        let desc = SINE_PARAM_DRIVER.desc();
        let empty = HashMap::new();
        let filled = desc.resolve_params(&empty).unwrap();
        assert_eq!(
            filled.require_f64("core.param.sine", "amplitude").unwrap(),
            1.0
        );
        assert_eq!(
            filled
                .require_f64("core.param.sine", "frequency_hz")
                .unwrap(),
            1.0
        );
        assert_eq!(
            filled.require_f64("core.param.sine", "offset").unwrap(),
            0.0
        );

        let mut unknown = HashMap::new();
        unknown.insert("nope".into(), Value::F64(1.0));
        let err = desc.resolve_params(&unknown).unwrap_err();
        assert!(
            matches!(
                err,
                PluginError::Param {
                    ref id,
                    ref got,
                    ..
                } if id == "nope" && got == "unknown"
            ),
            "{err:?}"
        );

        let mut mismatch = HashMap::new();
        mismatch.insert("amplitude".into(), Value::Vec2([0.0, 1.0]));
        let err = desc.resolve_params(&mismatch).unwrap_err();
        assert!(
            matches!(
                err,
                PluginError::Param {
                    ref id,
                    ref expected,
                    ref got,
                    ..
                } if id == "amplitude" && expected == "F64" && got == "Vec2"
            ),
            "{err:?}"
        );
    }
}
