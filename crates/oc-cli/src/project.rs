use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use oc_core::{Fps, RationalTime};
use oc_eval::{DataTrackId, DataTracks, ParamSource, Value};
use oc_export::{export_overlay_video, ExportOverlayRequest, ExportReport};
use oc_gpu::GpuCtx;
use oc_media::{probe, MediaInfo};
use oc_nodes::ParamRectOverlay;
use oc_plugin::{
    reference::register_reference_plugins, ParamDriverContext, PluginRegistry, ResolvedParams,
};

#[derive(Debug, Deserialize)]
pub struct ProjectV1 {
    /// スキーマバージョン。v1は今後の破壊的変更のための保険。
    pub version: u32,
    pub input: String,
    pub output: String,
    #[serde(default)]
    pub start_frame: i64,
    #[serde(default)]
    pub frame_count: Option<usize>,
    #[serde(default)]
    pub qp0: bool,
    /// ParamDriverが生成するDataTrack宣言(M1最小: sine 1本)。
    #[serde(default)]
    pub param_drivers: Vec<ParamDriverV1>,
    pub overlay: RectOverlayParamV1,
}

/// ProjectV1でParamDriverプラグインを1本宣言する最小形。
#[derive(Debug, Deserialize)]
pub struct ParamDriverV1 {
    /// プラグインID(例: `core.param.sine`)。
    pub plugin: String,
    /// 生成したDataTrackを格納するID。overlayの`ParamSource::Data`から参照する。
    pub track: String,
    #[serde(default)]
    pub params: HashMap<String, Value>,
}

/// ProjectV1のオーバーレイ。定数配列と`ParamSource` JSONの両方を受理する。
#[derive(Debug, Deserialize)]
pub struct RectOverlayParamV1 {
    pub center: ParamVec2V1,
    pub size: ParamVec2V1,
    pub color: ParamColorV1,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ParamVec2V1 {
    Const([f64; 2]),
    Source(ParamSource),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ParamColorV1 {
    /// straight RGBA, 0..1 (f32/f64どちらでも可)
    Const([f64; 4]),
    Source(ParamSource),
}

impl ParamVec2V1 {
    fn into_param_source(self) -> ParamSource {
        match self {
            ParamVec2V1::Const(v) => ParamSource::Const(Value::Vec2(v)),
            ParamVec2V1::Source(s) => s,
        }
    }
}

impl ParamColorV1 {
    fn into_param_source(self) -> ParamSource {
        match self {
            ParamColorV1::Const(v) => ParamSource::Const(Value::Color(v)),
            ParamColorV1::Source(s) => s,
        }
    }
}

impl RectOverlayParamV1 {
    pub fn into_param_overlay(self) -> ParamRectOverlay {
        ParamRectOverlay {
            center: self.center.into_param_source(),
            size: self.size.into_param_source(),
            color: self.color.into_param_source(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported project version: {0}")]
    UnsupportedVersion(u32),
    #[error("unknown param driver plugin: {0}")]
    UnknownParamDriver(String),
    #[error("duplicate data track id: {0}")]
    DuplicateDataTrack(String),
    #[error(transparent)]
    Media(#[from] oc_media::MediaError),
    #[error(transparent)]
    Plugin(#[from] oc_plugin::PluginError),
    #[error(transparent)]
    Export(#[from] oc_export::ExportError),
}

pub fn load_project_v1(path: impl AsRef<Path>) -> Result<ProjectV1, ProjectError> {
    let text = std::fs::read_to_string(path.as_ref())?;
    load_project_v1_from_str(&text)
}

pub fn load_project_v1_from_str(text: &str) -> Result<ProjectV1, ProjectError> {
    let project: ProjectV1 = serde_json::from_str(text)?;
    if project.version != 1 {
        return Err(ProjectError::UnsupportedVersion(project.version));
    }
    Ok(project)
}

fn export_frame_count(project: &ProjectV1, info: &MediaInfo) -> usize {
    project.frame_count.unwrap_or_else(|| {
        info.nb_frames
            .map(|n| (n - project.start_frame).max(0) as usize)
            .unwrap_or(0)
    })
}

/// ParamDriver宣言からDataTrack集合を構築する。exportループの外で1回だけ呼ぶ。
pub fn build_data_tracks(
    drivers: &[ParamDriverV1],
    start: RationalTime,
    duration: RationalTime,
    sample_rate: Fps,
) -> Result<DataTracks, ProjectError> {
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry)?;

    let mut tracks = DataTracks::new();
    let ctx = ParamDriverContext {
        start,
        duration,
        sample_rate,
    };

    for driver in drivers {
        let Some(plugin) = registry.param_driver_by_name(&driver.plugin) else {
            return Err(ProjectError::UnknownParamDriver(driver.plugin.clone()));
        };

        let mut params = ResolvedParams::new();
        for def in &plugin.desc().params {
            let value = driver
                .params
                .get(def.id)
                .cloned()
                .unwrap_or_else(|| def.default.clone());
            params.insert(def.id, value);
        }

        let track = plugin.build_track(ctx, &params)?;
        let id = DataTrackId(driver.track.clone());
        if tracks.get(&id).is_some() {
            return Err(ProjectError::DuplicateDataTrack(driver.track.clone()));
        }
        tracks.insert(id, track);
    }

    Ok(tracks)
}

pub fn export_project_v1(
    gpu: &GpuCtx,
    project_path: impl AsRef<Path>,
) -> Result<ExportReport, ProjectError> {
    let project_path = project_path.as_ref();
    let project = load_project_v1(project_path)?;
    let base = project_path.parent().unwrap_or_else(|| Path::new("."));
    let input_path = base.join(&project.input);
    let output_path = base.join(&project.output);

    let info = probe(&input_path)?;
    let export_frames = export_frame_count(&project, &info);
    let start = RationalTime::from_frame(project.start_frame, info.fps);
    let duration = RationalTime::from_frame(
        export_frames.saturating_sub(1) as i64,
        info.fps,
    );
    let data_tracks = build_data_tracks(
        &project.param_drivers,
        start,
        duration,
        info.fps,
    )?;
    let overlay = project.overlay.into_param_overlay();

    Ok(export_overlay_video(
        gpu,
        &ExportOverlayRequest {
            input_path: &input_path,
            output_path: &output_path,
            start_frame: project.start_frame,
            frame_count: project.frame_count,
            overlay,
            data_tracks,
            qp0: project.qp0,
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oc_core::RationalTime;
    use oc_eval::{Interp, Keyframe, KeyframeTrack};
    use oc_nodes::{CanonicalPoint, CanonicalSize, RectOverlay};
    use oc_plugin::reference::SINE_PARAM_DRIVER;

    fn keyed_center_overlay(
        start: CanonicalPoint,
        end: CanonicalPoint,
        size: CanonicalSize,
        color: [f32; 4],
        t0: RationalTime,
        t1: RationalTime,
    ) -> ParamRectOverlay {
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: t0,
            value: Value::Vec2([start.x, start.y]),
            interp: Interp::Linear,
        });
        track.insert(Keyframe {
            t: t1,
            value: Value::Vec2([end.x, end.y]),
            interp: Interp::Linear,
        });
        ParamRectOverlay {
            center: ParamSource::Keyframes(track),
            size: ParamSource::Const(Value::Vec2([size.width, size.height])),
            color: ParamSource::Const(Value::Color(color.map(|c| c as f64))),
        }
    }

    #[test]
    fn const_arrays_still_parse() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "overlay": {
                "center": [0.0, 0.0],
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let project: ProjectV1 = serde_json::from_str(json).unwrap();
        let overlay = project.overlay.into_param_overlay();
        let rect = overlay
            .eval(RationalTime::ZERO, &DataTracks::new())
            .unwrap();
        assert_eq!(rect.center, CanonicalPoint::CENTER);
        assert_eq!(
            rect.size,
            CanonicalSize {
                width: 0.5,
                height: 0.5
            }
        );
    }

    #[test]
    fn keyframes_json_parses_and_moves() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "overlay": {
                "center": {
                    "Keyframes": {
                        "keys": [
                            {"t": {"num": 0, "den": 1}, "value": {"Vec2": [-0.25, 0.0]}, "interp": "Linear"},
                            {"t": {"num": 1, "den": 1}, "value": {"Vec2": [0.25, 0.0]}, "interp": "Linear"}
                        ]
                    }
                },
                "size": [0.2, 0.2],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let project: ProjectV1 = serde_json::from_str(json).unwrap();
        let overlay = project.overlay.into_param_overlay();
        let tracks = DataTracks::new();
        let start = overlay.eval(RationalTime::ZERO, &tracks).unwrap();
        let mid = overlay.eval(RationalTime::new(1, 2), &tracks).unwrap();
        let end = overlay
            .eval(RationalTime::from_seconds(1), &tracks)
            .unwrap();
        assert!((start.center.x - (-0.25)).abs() < 1e-9);
        assert!(mid.center.x.abs() < 1e-9);
        assert!((end.center.x - 0.25).abs() < 1e-9);
    }

    #[test]
    fn datatrack_json_parses_and_moves_center_x() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "param_drivers": [
                {
                    "plugin": "core.param.sine",
                    "track": "sine_x",
                    "params": {
                        "amplitude": {"F64": 0.25},
                        "frequency_hz": {"F64": 0.5},
                        "offset": {"F64": 0.0}
                    }
                }
            ],
            "overlay": {
                "center": {
                    "Vec2Axes": {
                        "x": {"Data": {"track": "sine_x", "fallback": {"F64": 0.0}}},
                        "y": {"Const": {"F64": 0.0}}
                    }
                },
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let project: ProjectV1 = serde_json::from_str(json).unwrap();
        let tracks = build_data_tracks(
            &project.param_drivers,
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
            Fps { num: 12, den: 1 },
        )
        .unwrap();
        let overlay = project.overlay.into_param_overlay();

        let fps = Fps { num: 12, den: 1 };
        let start = overlay
            .eval(RationalTime::from_frame(0, fps), &tracks)
            .unwrap();
        let mid = overlay
            .eval(RationalTime::from_frame(6, fps), &tracks)
            .unwrap();
        let end = overlay
            .eval(RationalTime::from_frame(12, fps), &tracks)
            .unwrap();

        assert!(start.center.x.abs() < 1e-9);
        assert!((mid.center.x - 0.25).abs() < 1e-9);
        assert!(end.center.x.abs() < 1e-9);
        assert_eq!(start.center.y, 0.0);
    }

    #[test]
    fn build_data_tracks_rejects_unknown_plugin() {
        let drivers = vec![ParamDriverV1 {
            plugin: "nope".into(),
            track: "x".into(),
            params: HashMap::new(),
        }];
        let err = build_data_tracks(
            &drivers,
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
            Fps { num: 12, den: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ProjectError::UnknownParamDriver(id) if id == "nope"));
    }

    #[test]
    fn build_data_tracks_registers_sine_plugin() {
        let drivers = vec![ParamDriverV1 {
            plugin: "core.param.sine".into(),
            track: "sine_x".into(),
            params: HashMap::new(),
        }];
        let tracks = build_data_tracks(
            &drivers,
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
            Fps { num: 12, den: 1 },
        )
        .unwrap();
        assert!(tracks.get(&DataTrackId("sine_x".into())).is_some());
    }

    #[test]
    fn keyed_helper_matches_eval() {
        let overlay = keyed_center_overlay(
            CanonicalPoint { x: -0.25, y: 0.0 },
            CanonicalPoint { x: 0.25, y: 0.0 },
            CanonicalSize {
                width: 0.2,
                height: 0.2,
            },
            [1.0, 0.0, 0.0, 1.0],
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
        );
        let mid = overlay
            .eval(RationalTime::new(1, 2), &DataTracks::new())
            .unwrap();
        assert!(mid.center.x.abs() < 1e-9);
        let _ = ParamRectOverlay::constant(RectOverlay {
            center: CanonicalPoint::CENTER,
            size: CanonicalSize {
                width: 0.1,
                height: 0.1,
            },
            color: [1.0, 1.0, 1.0, 1.0],
        });
        let _ = SINE_PARAM_DRIVER;
    }
}
