use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use motolii_core::{
    ColorSpace, Fps, FrameDesc, PixelFormat, Quality, RationalTime, RationalTimeError, TimeMap,
    TimeMapError,
};
use motolii_eval::{DataTrackId, DataTracks, ParamSource, Value};
use motolii_export::{export_overlay_video, ExportOverlayRequest, ExportReport};
use motolii_gpu::{GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_media::{probe, FrameReader, MediaInfo};
use motolii_nodes::{ParamOverlayError, ParamRectOverlay};
use motolii_plugin::{
    migrate_plugin_params, ParamDriverContext, PluginError, PluginRuntime, TextureRef,
};
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{
    render_frame_with_background_texture, BackgroundTextureRequest, RenderSession,
};

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
    /// クリップの時間写像(F-4)。省略時は恒等。
    #[serde(default)]
    pub time_map: TimeMap,
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
    /// 保存時のeffect version。省略時は1(旧JSON互換→migrate対象)。
    #[serde(default = "default_effect_version")]
    pub effect_version: u32,
    #[serde(default)]
    pub params: HashMap<String, Value>,
}

fn default_effect_version() -> u32 {
    1
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
    /// 非線形sRGB・straight・0..1(M2E-13。f32/f64どちらでも可)
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
    Media(#[from] motolii_media::MediaError),
    #[error(transparent)]
    Plugin(#[from] motolii_plugin::PluginError),
    #[error(transparent)]
    PluginContract(#[from] motolii_plugin::PluginContractError),
    #[error(transparent)]
    PluginRuntime(#[from] motolii_plugin::PluginRuntimeError),
    #[error("unsupported project time_map: only identity is accepted until M2")]
    UnsupportedTimeMap,
    #[error(transparent)]
    Export(#[from] motolii_export::ExportError),
    #[error(transparent)]
    Overlay(#[from] ParamOverlayError),
    #[error(transparent)]
    Render(#[from] motolii_render::RenderError),
    #[error(transparent)]
    Gpu(#[from] motolii_gpu::GpuRuntimeError),
    #[error(transparent)]
    Yuv(#[from] motolii_gpu::YuvError),
    #[error(transparent)]
    TimeMap(#[from] TimeMapError),
    #[error(transparent)]
    RationalTime(#[from] RationalTimeError),
    #[error(transparent)]
    FirstParty(#[from] motolii_plugins_firstparty::FirstPartyError),
}

fn reference_runtime() -> Result<PluginRuntime, ProjectError> {
    Ok(first_party_runtime()?)
}

pub fn load_project_v1(path: impl AsRef<Path>) -> Result<ProjectV1, ProjectError> {
    let text = std::fs::read_to_string(path.as_ref())?;
    load_project_v1_from_str(&text)
}

pub fn load_project_v1_from_str(text: &str) -> Result<ProjectV1, ProjectError> {
    let mut project: ProjectV1 = serde_json::from_str(text)?;
    if project.version != 1 {
        return Err(ProjectError::UnsupportedVersion(project.version));
    }
    project.time_map.validate()?;
    if !project.time_map.is_identity() {
        return Err(ProjectError::UnsupportedTimeMap);
    }
    // M2E-8: 型不一致・未知キーはロード時に構造化エラー(serde受理だけでは足りない)。
    normalize_param_drivers(&mut project.param_drivers)?;
    Ok(project)
}

/// migrate → resolve_params。成功時は default 充填済み params と現行 effect_version を書き戻す。
fn normalize_param_drivers(drivers: &mut [ParamDriverV1]) -> Result<(), ProjectError> {
    let runtime = reference_runtime()?;
    for driver in drivers.iter_mut() {
        let Some(plugin) = runtime.executors().param_driver_by_name(&driver.plugin) else {
            return Err(ProjectError::UnknownParamDriver(driver.plugin.clone()));
        };
        let resolved = resolve_raw_params(
            &driver.plugin,
            driver.effect_version,
            plugin.desc(),
            &driver.params,
        )?;
        driver.params = plugin
            .desc()
            .params
            .iter()
            .map(|def| {
                (
                    def.id.to_string(),
                    resolved
                        .get(def.id)
                        .cloned()
                        .unwrap_or_else(|| def.default.clone()),
                )
            })
            .collect();
        driver.effect_version = plugin.desc().version;
    }
    Ok(())
}

fn resolve_driver_params(
    runtime: &PluginRuntime,
    driver: &ParamDriverV1,
) -> Result<motolii_plugin::ResolvedParams, ProjectError> {
    let Some(plugin) = runtime.executors().param_driver_by_name(&driver.plugin) else {
        return Err(ProjectError::UnknownParamDriver(driver.plugin.clone()));
    };
    resolve_raw_params(
        &driver.plugin,
        driver.effect_version,
        plugin.desc(),
        &driver.params,
    )
}

fn resolve_raw_params(
    plugin_name: &str,
    effect_version: u32,
    desc: &motolii_plugin::NodeDesc,
    params: &HashMap<String, Value>,
) -> Result<motolii_plugin::ResolvedParams, ProjectError> {
    let mut raw_params = params.clone();
    migrate_plugin_params(plugin_name, effect_version, desc.version, &mut raw_params)?;

    match desc.resolve_params(&raw_params) {
        Ok(params) => Ok(params),
        Err(PluginError::Param {
            plugin, id, got, ..
        }) if got == "unknown" => Err(ProjectError::UnknownParam { plugin, param: id }),
        Err(err) => Err(err.into()),
    }
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
    // M2E-17: duration=総尺、半開 [start, start+duration)。
    // end_exclusive = floor(duration×fps)。旧 +1(最終PTS流儀)はオフバイワンになる。
    let end_exclusive = duration.try_to_frame_floor(info.fps)?;
    Ok((end_exclusive - project.start_frame).max(0) as usize)
}

/// ParamDriver宣言からDataTrack集合を構築する。exportループの外で1回だけ呼ぶ。
pub fn build_data_tracks(
    drivers: &[ParamDriverV1],
    start: RationalTime,
    duration: RationalTime,
    sample_rate: Fps,
) -> Result<DataTracks, ProjectError> {
    let runtime = reference_runtime()?;

    let mut tracks = DataTracks::new();
    let ctx = ParamDriverContext {
        start,
        duration,
        sample_rate,
    };

    for driver in drivers {
        let Some(plugin) = runtime.executors().param_driver_by_name(&driver.plugin) else {
            return Err(ProjectError::UnknownParamDriver(driver.plugin.clone()));
        };
        // ロード済みでも、直接構築経路の防御として再解決する。
        let params = resolve_driver_params(&runtime, driver)?;
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
    let start = RationalTime::try_from_frame(project.start_frame, info.fps)?;
    // M2E-17: ParamDriver に渡す duration も総尺(半開 [start, start+duration))。
    let duration = RationalTime::try_from_frame(export_frames as i64, info.fps)?;
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
                time_map: self.project.time_map,
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
        let background = yuv.convert(gpu, &frame)?;
        let rendered = render_frame_with_background_texture(
            gpu,
            session,
            &BackgroundTextureRequest {
                desc: self.render_desc,
                timeline_time: frame.pts,
                time_map: self.project.time_map,
                background: TextureRef {
                    texture: &background,
                    desc: self.render_desc,
                },
                overlay,
            },
            Quality::FINAL,
        )?;
        Ok(downloader.download(
            gpu,
            &rendered.texture,
            motolii_export::EXPORT_DOWNLOAD_TIMEOUT,
        )?)
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
            motolii_export::ExportError::InvalidRequest("exported mp4 ended before expected frame"),
        ))?;
        let texture = yuv.convert(gpu, &frame)?;
        Ok(downloader.download(gpu, &texture, motolii_export::EXPORT_DOWNLOAD_TIMEOUT)?)
    }

    fn read_source_frame(
        &self,
        export_index: usize,
    ) -> Result<motolii_core::CpuFrame, ProjectError> {
        let mut reader = FrameReader::open(&self.input_path, &self.info, self.project.start_frame)?;
        for _ in 0..export_index {
            let _ = reader.next_frame()?.ok_or(ProjectError::Export(
                motolii_export::ExportError::InvalidRequest("input ended before expected frame"),
            ))?;
        }
        reader.next_frame()?.ok_or(ProjectError::Export(
            motolii_export::ExportError::InvalidRequest("input ended before expected frame"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::RationalTime;
    use motolii_eval::{Interp, Keyframe, KeyframeTrack};
    use motolii_nodes::{CanonicalPoint, CanonicalSize, RectOverlay};
    use motolii_plugin::reference::SINE_PARAM_DRIVER;

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
        let mid = overlay
            .eval(RationalTime::try_new(1, 2).unwrap(), &tracks)
            .unwrap();
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
            Fps::try_new(12, 1).unwrap(),
        )
        .unwrap();
        let overlay = project.overlay.into_param_overlay();

        let fps = Fps::try_new(12, 1).unwrap();
        let start = overlay
            .eval(RationalTime::try_from_frame(0, fps).unwrap(), &tracks)
            .unwrap();
        let mid = overlay
            .eval(RationalTime::try_from_frame(6, fps).unwrap(), &tracks)
            .unwrap();
        // M2E-17 テスト更新: 半開 [0,1) @ 12fps の最終内包は frame 11。
        // 旧 frame 12(=終端ちょうど)は範囲外→末尾へクランプ(正弦のゼロ戻りは終端PTS流儀の名残)。
        let last_inclusive = overlay
            .eval(RationalTime::try_from_frame(11, fps).unwrap(), &tracks)
            .unwrap();
        let end_exclusive = overlay
            .eval(RationalTime::try_from_frame(12, fps).unwrap(), &tracks)
            .unwrap();

        assert!(start.center.x.abs() < 1e-9);
        assert!((mid.center.x - 0.25).abs() < 1e-9);
        assert!((end_exclusive.center.x - last_inclusive.center.x).abs() < 1e-9);
        assert_eq!(start.center.y, 0.0);
        assert_eq!(
            tracks
                .get(&DataTrackId("sine_x".into()))
                .unwrap()
                .values
                .len(),
            12
        );
    }

    #[test]
    fn build_data_tracks_rejects_unknown_plugin() {
        let drivers = vec![ParamDriverV1 {
            plugin: "nope".into(),
            track: "x".into(),
            effect_version: 1,
            params: HashMap::new(),
        }];
        let err = build_data_tracks(
            &drivers,
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
            Fps::try_new(12, 1).unwrap(),
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
            effect_version: 2,
            params,
        }];
        let err = build_data_tracks(
            &drivers,
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
            Fps::try_new(12, 1).unwrap(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProjectError::UnknownParam { plugin, param }
            if plugin == "core.param.sine" && param == "amplitud"
        ));
    }

    #[test]
    fn load_project_rejects_param_type_mismatch() {
        // M2E-8完了条件: 型不一致JSONはロード境界で構造化エラー。
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "param_drivers": [
                {
                    "plugin": "core.param.sine",
                    "track": "sine_x",
                    "effect_version": 2,
                    "params": {
                        "amplitude": {"Vec2": [1.0, 2.0]},
                        "frequency_hz": {"F64": 1.0},
                        "offset": {"F64": 0.0}
                    }
                }
            ],
            "overlay": {
                "center": [0.0, 0.0],
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let err = load_project_v1_from_str(json).unwrap_err();
        assert!(
            matches!(
                err,
                ProjectError::Plugin(PluginError::Param {
                    ref plugin,
                    ref id,
                    ref expected,
                    ref got,
                }) if plugin == "core.param.sine"
                    && id == "amplitude"
                    && expected == "F64"
                    && got == "Vec2"
            ),
            "expected PluginError::Param type mismatch via load, got {err:?}"
        );
    }

    #[test]
    fn old_sine_amp_param_migrates_on_load() {
        // FG-C4: effect_version=1 の `amp` が現行 `amplitude` に移行する。
        let drivers = vec![ParamDriverV1 {
            plugin: "core.param.sine".into(),
            track: "sine_x".into(),
            effect_version: 1,
            params: HashMap::from([("amp".into(), Value::F64(0.25))]),
        }];
        let tracks = build_data_tracks(
            &drivers,
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
            Fps::try_new(12, 1).unwrap(),
        )
        .unwrap();
        assert!(tracks.get(&DataTrackId("sine_x".into())).is_some());
    }

    #[test]
    fn project_time_map_defaults_to_identity() {
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
        assert_eq!(project.time_map, TimeMap::identity());
    }

    #[test]
    fn project_rejects_non_identity_time_map() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "time_map": {
                "source_start": {"num": 0, "den": 1},
                "speed_num": 2,
                "speed_den": 1
            },
            "overlay": {
                "center": [0.0, 0.0],
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let err = load_project_v1_from_str(json).unwrap_err();
        assert!(matches!(err, ProjectError::UnsupportedTimeMap));
    }

    #[test]
    fn project_rejects_black_overrun_even_when_affine_is_identity() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "time_map": {
                "source_start": {"num": 0, "den": 1},
                "speed_num": 1,
                "speed_den": 1,
                "overrun_mode": "black"
            },
            "overlay": {
                "center": [0.0, 0.0],
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let err = load_project_v1_from_str(json).unwrap_err();
        assert!(matches!(err, ProjectError::UnsupportedTimeMap));
    }

    #[test]
    fn project_rejects_legacy_timeline_start_field() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "time_map": {
                "source_start": {"num": 0, "den": 1},
                "timeline_start": {"num": 0, "den": 1},
                "speed_num": 1,
                "speed_den": 1
            },
            "overlay": {
                "center": [0.0, 0.0],
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let err = load_project_v1_from_str(json).unwrap_err();
        assert!(matches!(err, ProjectError::Json(_)), "{err:?}");
    }

    #[test]
    fn project_rejects_invalid_bezier_keyframes() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "overlay": {
                "center": {
                    "Keyframes": {
                        "keys": [
                            {
                                "t": {"num": 0, "den": 1},
                                "value": {"Vec2": [0.0, 0.0]},
                                "interp": {"Bezier": {"x1": 1.5, "y1": 0.0, "x2": 0.5, "y2": 1.0}}
                            },
                            {
                                "t": {"num": 1, "den": 1},
                                "value": {"Vec2": [1.0, 0.0]},
                                "interp": "Linear"
                            }
                        ]
                    }
                },
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        assert!(load_project_v1_from_str(json).is_err());
    }

    #[test]
    fn project_rejects_invalid_time_map() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "time_map": {
                "source_start": {"num": 0, "den": 1},
                "speed_num": 1,
                "speed_den": 0
            },
            "overlay": {
                "center": [0.0, 0.0],
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let err = load_project_v1_from_str(json).unwrap_err();
        // TimeMapのDeserializeがspeed_denを拒否するためJson境界で落ちる
        assert!(matches!(err, ProjectError::Json(_)), "{err:?}");
    }

    #[test]
    fn project_rejects_non_positive_speed_num() {
        let json = r#"{
            "version": 1,
            "input": "in.mp4",
            "output": "out.mp4",
            "time_map": {
                "source_start": {"num": 0, "den": 1},
                "speed_num": 0,
                "speed_den": 1
            },
            "overlay": {
                "center": [0.0, 0.0],
                "size": [0.5, 0.5],
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        }"#;
        let err = load_project_v1_from_str(json).unwrap_err();
        assert!(matches!(err, ProjectError::Json(_)), "{err:?}");
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
            time_map: TimeMap::identity(),
            param_drivers: vec![],
            overlay: RectOverlayParamV1 {
                center: ParamVec2V1::Const([0.0, 0.0]),
                size: ParamVec2V1::Const([0.5, 0.5]),
                color: ParamColorV1::Const([1.0, 0.0, 0.0, 1.0]),
            },
        };
        let fps = Fps::try_new(30, 1).unwrap();
        // M2E-17 テスト更新: duration=総尺。90フレーム素材は from_frame(90)。
        // 旧 from_frame(89)+1=90(最終PTS流儀)は規約変更に伴う正当な期待値更新。
        let info = MediaInfo {
            width: 64,
            height: 48,
            fps,
            duration: Some(RationalTime::try_from_frame(90, fps).unwrap()),
            nb_frames: None,
            color_space: motolii_core::ColorSpace::Rec709Limited,
            rotation: 0,
        };
        assert_eq!(export_frame_count(&project, &info).unwrap(), 90);
    }

    #[test]
    fn export_frame_count_half_open_excludes_end_frame() {
        // 総尺ちょうど(= end)のフレームは半開で範囲外。count に +1 しない。
        let fps = Fps::try_new(30, 1).unwrap();
        let project = ProjectV1 {
            version: 1,
            input: "in.mp4".into(),
            output: "out.mp4".into(),
            start_frame: 0,
            frame_count: None,
            qp0: false,
            time_map: TimeMap::identity(),
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
            fps,
            duration: Some(RationalTime::try_from_frame(90, fps).unwrap()),
            nb_frames: None,
            color_space: motolii_core::ColorSpace::Rec709Limited,
            rotation: 0,
        };
        assert_eq!(export_frame_count(&project, &info).unwrap(), 90);
        // 旧流儀なら 91 になっていた。半開では 90 のまま。
        assert_ne!(export_frame_count(&project, &info).unwrap(), 91);

        let mut late_start = project.clone();
        late_start.start_frame = 10;
        assert_eq!(export_frame_count(&late_start, &info).unwrap(), 80);
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
            time_map: TimeMap::identity(),
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
            fps: Fps::try_new(30, 1).unwrap(),
            duration: None,
            nb_frames: None,
            color_space: motolii_core::ColorSpace::Rec709Limited,
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
            effect_version: 2,
            params: HashMap::new(),
        }];
        let tracks = build_data_tracks(
            &drivers,
            RationalTime::ZERO,
            RationalTime::from_seconds(1),
            Fps::try_new(12, 1).unwrap(),
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
            .eval(RationalTime::try_new(1, 2).unwrap(), &DataTracks::new())
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
