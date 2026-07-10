use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use oc_core::{ColorSpace, Fps, FrameDesc, PixelFormat, Quality, RationalTime};
use oc_eval::{DataTrackId, DataTracks, ParamSource, Value};
use oc_export::{export_overlay_video, ExportOverlayRequest, ExportReport};
use oc_gpu::{GpuCtx, RgbaDownloader, YuvToRgba};
use oc_media::{probe, FrameReader, MediaInfo};
use oc_nodes::{ParamOverlayError, ParamRectOverlay};
use oc_plugin::{
    reference::register_reference_plugins, ParamDriverContext, PluginRegistry, ResolvedParams,
    TextureRef,
};
use oc_render::{render_frame_with_background_texture, BackgroundTextureRequest, RenderSession};

#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
pub struct ParamDriverV1 {
    /// プラグインID(例: `core.param.sine`)。
    pub plugin: String,
    /// 生成したDataTrackを格納するID。overlayの`ParamSource::Data`から参照する。
    pub track: String,
    #[serde(default)]
    pub params: HashMap<String, Value>,
}

/// ProjectV1のオーバーレイ。定数配列と`ParamSource` JSONの両方を受理する。
#[derive(Debug, Deserialize, Clone)]
pub struct RectOverlayParamV1 {
    pub center: ParamVec2V1,
    pub size: ParamVec2V1,
    pub color: ParamColorV1,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ParamVec2V1 {
    Const([f64; 2]),
    Source(ParamSource),
}

#[derive(Debug, Deserialize, Clone)]
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
    #[error("unknown param {param} for plugin {plugin}")]
    UnknownParam { plugin: String, param: String },
    #[error("cannot determine export length: input has no nb_frames or duration in probe")]
    IndeterminateExportLength,
    #[error("duplicate data track id: {0}")]
    DuplicateDataTrack(String),
    #[error(transparent)]
    Media(#[from] oc_media::MediaError),
    #[error(transparent)]
    Plugin(#[from] oc_plugin::PluginError),
    #[error(transparent)]
    Export(#[from] oc_export::ExportError),
    #[error(transparent)]
    Overlay(#[from] ParamOverlayError),
    #[error(transparent)]
    Render(#[from] oc_render::RenderError),
    #[error(transparent)]
    Gpu(#[from] oc_gpu::GpuRuntimeError),
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

fn export_frame_count(project: &ProjectV1, info: &MediaInfo) -> Result<usize, ProjectError> {
    if let Some(n) = project.frame_count {
        return Ok(n);
    }
    if let Some(nb) = info.nb_frames {
        return Ok((nb - project.start_frame).max(0) as usize);
    }
    let Some(duration) = info.duration else {
        return Err(ProjectError::IndeterminateExportLength);
    };
    let last_frame = duration.to_frame_floor(info.fps);
    Ok((last_frame - project.start_frame + 1).max(0) as usize)
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

        let known: HashSet<&str> = plugin.desc().params.iter().map(|p| p.id).collect();
        for key in driver.params.keys() {
            if !known.contains(key.as_str()) {
                return Err(ProjectError::UnknownParam {
                    plugin: driver.plugin.clone(),
                    param: key.clone(),
                });
            }
        }

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
    let prepared = prepare_project_export(project_path)?;
    prepared.export(gpu)
}

/// プロジェクトJSONを解決し、export/verifyで共有するコンテキストを構築する。
pub fn prepare_project_export(
    project_path: impl AsRef<Path>,
) -> Result<PreparedProject, ProjectError> {
    let project_path = project_path.as_ref().to_path_buf();
    let project = load_project_v1(&project_path)?;
    let base = project_path.parent().unwrap_or_else(|| Path::new("."));
    let input_path = base.join(&project.input);
    let output_path = base.join(&project.output);

    let info = probe(&input_path)?;
    let export_frames = export_frame_count(&project, &info)?;
    let start = RationalTime::from_frame(project.start_frame, info.fps);
    let duration = RationalTime::from_frame(export_frames.saturating_sub(1) as i64, info.fps);
    let data_tracks = build_data_tracks(&project.param_drivers, start, duration, info.fps)?;
    let overlay = project.overlay.clone().into_param_overlay();
    let render_desc = FrameDesc::packed(
        info.width,
        info.height,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    );

    Ok(PreparedProject {
        project_path,
        input_path,
        output_path,
        project,
        info,
        export_frames,
        overlay,
        data_tracks,
        render_desc,
    })
}

#[derive(Debug, Clone)]
pub struct PreparedProject {
    pub project_path: PathBuf,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub project: ProjectV1,
    pub info: MediaInfo,
    pub export_frames: usize,
    pub overlay: ParamRectOverlay,
    pub data_tracks: DataTracks,
    pub render_desc: FrameDesc,
}

impl PreparedProject {
    pub fn export(&self, gpu: &GpuCtx) -> Result<ExportReport, ProjectError> {
        Ok(export_overlay_video(
            gpu,
            &ExportOverlayRequest {
                input_path: &self.input_path,
                output_path: &self.output_path,
                start_frame: self.project.start_frame,
                frame_count: self.project.frame_count,
                overlay: self.overlay.clone(),
                data_tracks: self.data_tracks.clone(),
                qp0: self.project.qp0,
            },
        )?)
    }

    pub fn render_export_frame_rgba(
        &self,
        gpu: &GpuCtx,
        export_index: usize,
        session: &mut RenderSession,
        yuv: &mut YuvToRgba,
        downloader: &mut RgbaDownloader,
    ) -> Result<Vec<u8>, ProjectError> {
        let frame = self.read_source_frame(export_index)?;
        let overlay = self.overlay.eval(frame.pts, &self.data_tracks)?;
        let background = yuv.convert(gpu, &frame);
        let rendered = render_frame_with_background_texture(
            gpu,
            session,
            &BackgroundTextureRequest {
                desc: self.render_desc,
                timeline_time: frame.pts,
                source_time: frame.pts,
                background: TextureRef {
                    texture: &background,
                    desc: self.render_desc,
                },
                overlay,
            },
            Quality::FINAL,
        )?;
        Ok(downloader.download(gpu, &rendered.texture, oc_export::EXPORT_DOWNLOAD_TIMEOUT)?)
    }

    pub fn decode_exported_frame_rgba(
        &self,
        gpu: &GpuCtx,
        export_index: usize,
        yuv: &mut YuvToRgba,
        downloader: &mut RgbaDownloader,
    ) -> Result<Vec<u8>, ProjectError> {
        let out_info = probe(&self.output_path)?;
        let mut reader = FrameReader::open(&self.output_path, &out_info, export_index as i64)?;
        let frame = reader.next_frame()?.ok_or(ProjectError::Export(
            oc_export::ExportError::InvalidRequest("exported mp4 ended before expected frame"),
        ))?;
        let texture = yuv.convert(gpu, &frame);
        Ok(downloader.download(gpu, &texture, oc_export::EXPORT_DOWNLOAD_TIMEOUT)?)
    }

    fn read_source_frame(&self, export_index: usize) -> Result<oc_core::CpuFrame, ProjectError> {
        let mut reader = FrameReader::open(&self.input_path, &self.info, self.project.start_frame)?;
        for _ in 0..export_index {
            let _ = reader.next_frame()?.ok_or(ProjectError::Export(
                oc_export::ExportError::InvalidRequest("input ended before expected frame"),
            ))?;
        }
        reader.next_frame()?.ok_or(ProjectError::Export(
            oc_export::ExportError::InvalidRequest("input ended before expected frame"),
        ))
    }
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
    fn build_data_tracks_rejects_unknown_param() {
        let mut params = HashMap::new();
        params.insert("amplitud".into(), Value::F64(1.0));
        let drivers = vec![ParamDriverV1 {
            plugin: "core.param.sine".into(),
            track: "sine_x".into(),
            params,
        }];
        let err = build_data_tracks(
            &drivers,
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
            Fps { num: 12, den: 1 },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProjectError::UnknownParam { plugin, param }
            if plugin == "core.param.sine" && param == "amplitud"
        ));
    }

    #[test]
    fn export_frame_count_falls_back_to_duration_when_nb_frames_missing() {
        let project = ProjectV1 {
            version: 1,
            input: "in.mp4".into(),
            output: "out.mp4".into(),
            start_frame: 0,
            frame_count: None,
            qp0: false,
            param_drivers: vec![],
            overlay: RectOverlayParamV1 {
                center: ParamVec2V1::Const([0.0, 0.0]),
                size: ParamVec2V1::Const([0.5, 0.5]),
                color: ParamColorV1::Const([1.0, 0.0, 0.0, 1.0]),
            },
        };
        let info = MediaInfo {
            width: 64,
            height: 48,
            fps: Fps { num: 30, den: 1 },
            duration: Some(RationalTime::from_frame(89, Fps { num: 30, den: 1 })),
            nb_frames: None,
            color_space: oc_core::ColorSpace::Rec709Limited,
            rotation: 0,
        };
        assert_eq!(export_frame_count(&project, &info).unwrap(), 90);
    }

    #[test]
    fn export_frame_count_errors_without_nb_frames_or_duration() {
        let project = ProjectV1 {
            version: 1,
            input: "in.mp4".into(),
            output: "out.mp4".into(),
            start_frame: 0,
            frame_count: None,
            qp0: false,
            param_drivers: vec![],
            overlay: RectOverlayParamV1 {
                center: ParamVec2V1::Const([0.0, 0.0]),
                size: ParamVec2V1::Const([0.5, 0.5]),
                color: ParamColorV1::Const([1.0, 0.0, 0.0, 1.0]),
            },
        };
        let info = MediaInfo {
            width: 64,
            height: 48,
            fps: Fps { num: 30, den: 1 },
            duration: None,
            nb_frames: None,
            color_space: oc_core::ColorSpace::Rec709Limited,
            rotation: 0,
        };
        assert!(matches!(
            export_frame_count(&project, &info),
            Err(ProjectError::IndeterminateExportLength)
        ));
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
