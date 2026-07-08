use std::path::Path;

use serde::Deserialize;

use oc_eval::{ParamSource, Value};
use oc_export::{export_overlay_video, ExportOverlayRequest, ExportReport};
use oc_gpu::GpuCtx;
use oc_nodes::ParamRectOverlay;

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
    pub overlay: RectOverlayParamV1,
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

pub fn export_project_v1(
    gpu: &GpuCtx,
    project_path: impl AsRef<Path>,
) -> Result<ExportReport, ProjectError> {
    let project = load_project_v1(project_path)?;
    let input_path = Path::new(&project.input);
    let output_path = Path::new(&project.output);
    let overlay = project.overlay.into_param_overlay();

    Ok(export_overlay_video(
        gpu,
        &ExportOverlayRequest {
            input_path,
            output_path,
            start_frame: project.start_frame,
            frame_count: project.frame_count,
            overlay,
            qp0: project.qp0,
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oc_core::RationalTime;
    use oc_eval::{DataTracks, Interp, Keyframe, KeyframeTrack};
    use oc_nodes::{CanonicalPoint, CanonicalSize, RectOverlay};

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
        // constant helper stil compiles with RectOverlay
        let _ = ParamRectOverlay::constant(RectOverlay {
            center: CanonicalPoint::CENTER,
            size: CanonicalSize {
                width: 0.1,
                height: 0.1,
            },
            color: [1.0, 1.0, 1.0, 1.0],
        });
    }
}
