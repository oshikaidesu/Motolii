//! PV-1: 単一windowへの保持texture lifecycle — 型付き状態機械とresource counter。
//! window無しで振る舞い試験可能。禁止構造の審判はカウンタと状態遷移で行う。

use std::sync::{Mutex, TryLockError};

use motolii_gpu::GpuCtx;
use serde::{Deserialize, Serialize};
use slint::wgpu_29::wgpu;
use thiserror::Error;

pub const TICKET: &str = "PV-1";
pub const DEFAULT_WIDTH: u32 = 640;
pub const DEFAULT_HEIGHT: u32 = 360;

/// upload_rgba と同型の usage (関数自体は毎frame呼ばない)
fn rgba_texture_descriptor(width: u32, height: u32) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some("pv1-retained-rgba"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Booting,
    Ready,
    Displaying,
    Hidden,
    Minimized,
    Regenerated,
    Restored,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    Hide,
    Show,
    Minimize,
    Restore,
    Resize { width: u32, height: u32 },
    ContentTick,
    Regenerate,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCounters {
    pub texture_create_count: u64,
    pub pipeline_create_count: u64,
    pub shader_module_create_count: u64,
    pub image_try_from_count: u64,
    pub content_update_count: u64,
    pub ui_property_set_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Pending,
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanChecklistEntry {
    pub id: String,
    pub spec: String,
    pub verdict: Verdict,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendEntry {
    pub backend: String,
    pub verdict: Verdict,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationStatus {
    pub lifecycle_behavior_tests: Verdict,
    pub release_build: Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pv1Manifest {
    pub ticket: String,
    pub spike_crate: String,
    pub recorded_at: String,
    pub runner: String,
    pub environment_note: String,
    pub overall: Verdict,
    pub automation: AutomationStatus,
    pub human_checks: Vec<HumanChecklistEntry>,
    pub backends: Vec<BackendEntry>,
    pub last_counters: ResourceCounters,
    pub last_state: String,
}

impl Pv1Manifest {
    pub fn skeleton_template() -> Self {
        Self {
            ticket: TICKET.into(),
            spike_crate: "spikes/pv1-texture-lifecycle".into(),
            recorded_at: "未実走".into(),
            runner: String::new(),
            environment_note:
                "人間実機審判は開発主機 release build で実施。未実測は pending のまま。".into(),
            overall: Verdict::Pending,
            automation: AutomationStatus {
                lifecycle_behavior_tests: Verdict::Pending,
                release_build: Verdict::Pending,
            },
            human_checks: vec![
                HumanChecklistEntry {
                    id: "H1".into(),
                    spec: "単一window継続表示 ≥10分、破綻・黒画面なし".into(),
                    verdict: Verdict::Pending,
                    notes: String::new(),
                },
                HumanChecklistEntry {
                    id: "H2".into(),
                    spec: "resize 100回後も表示継続、不要な毎frame再生成感なし".into(),
                    verdict: Verdict::Pending,
                    notes: String::new(),
                },
                HumanChecklistEntry {
                    id: "H3".into(),
                    spec: "hide/show（単一window・Hide後自動Show往復）".into(),
                    verdict: Verdict::Pending,
                    notes: String::new(),
                },
                HumanChecklistEntry {
                    id: "H4".into(),
                    spec: "minimize相当→自動Restore往復".into(),
                    verdict: Verdict::Pending,
                    notes: String::new(),
                },
                HumanChecklistEntry {
                    id: "H5".into(),
                    spec: "明示texture再生成後の復帰".into(),
                    verdict: Verdict::Pending,
                    notes: String::new(),
                },
                HumanChecklistEntry {
                    id: "H6".into(),
                    spec: "DPI/monitor移動（可能な環境のみ）".into(),
                    verdict: Verdict::Pending,
                    notes: String::new(),
                },
            ],
            backends: vec![
                BackendEntry {
                    backend: "Metal".into(),
                    verdict: Verdict::Pending,
                    notes: "開発主機で実測後に更新".into(),
                },
                BackendEntry {
                    backend: "DX12".into(),
                    verdict: Verdict::Pending,
                    notes: "未実測 — pending 維持".into(),
                },
                BackendEntry {
                    backend: "Vulkan".into(),
                    verdict: Verdict::Pending,
                    notes: "未実測 — pending 維持".into(),
                },
            ],
            last_counters: ResourceCounters::default(),
            last_state: "booting".into(),
        }
    }

    pub fn record_run(
        &mut self,
        recorded_at: String,
        counters: ResourceCounters,
        state: LifecycleState,
    ) {
        self.recorded_at = recorded_at;
        self.last_counters = counters;
        self.last_state = format!("{state:?}").to_lowercase();
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    #[error("lifecycle engine not initialized")]
    NotInitialized,
    #[error("invalid resize dimensions: {width}x{height}")]
    InvalidResize { width: u32, height: u32 },
    #[error("event {event:?} not allowed in state {state:?}")]
    InvalidTransition {
        state: LifecycleState,
        event: &'static str,
    },
    #[error("Image::try_from failed")]
    ImageBindFailed,
    #[error("lifecycle failed — no further events accepted")]
    Failed,
    #[error("mailbox mutex poisoned")]
    SlotPoisoned,
    #[error("worker command channel closed")]
    CommandChannelClosed,
}

/// Worker→UI 最新値 mailbox。guard を公開せず replace / try_take のみ。
pub struct LatestSlot<T> {
    inner: Mutex<Option<T>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SlotTake<T> {
    Item(T),
    Empty,
    WouldBlock,
    Poisoned,
}

impl<T> LatestSlot<T> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// 成功時のみ Ok。poison 時は回復書込後も Err(SlotPoisoned) を返す。
    pub fn replace(&self, value: T) -> Result<(), LifecycleError> {
        match self.inner.lock() {
            Ok(mut guard) => {
                *guard = Some(value);
                Ok(())
            }
            Err(_poisoned) => {
                let mut guard = _poisoned.into_inner();
                *guard = Some(value);
                self.inner.clear_poison();
                Err(LifecycleError::SlotPoisoned)
            }
        }
    }

    pub fn try_take(&self) -> SlotTake<T> {
        match self.inner.try_lock() {
            Ok(mut guard) => match guard.take() {
                Some(value) => SlotTake::Item(value),
                None => SlotTake::Empty,
            },
            Err(TryLockError::WouldBlock) => SlotTake::WouldBlock,
            Err(TryLockError::Poisoned(poisoned)) => {
                let mut guard = poisoned.into_inner();
                let _ = guard.take();
                self.inner.clear_poison();
                SlotTake::Poisoned
            }
        }
    }
}

/// Worker→UI status mailbox の最新値。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub state: LifecycleState,
    pub counters: ResourceCounters,
    pub last_error: Option<LifecycleError>,
}

/// UI tick 内の status 行更新判断。texture/status いずれかの poison は通常行より優先。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiTickStatusDecision {
    KeepPrevious,
    ShowStatus(String),
    ShowFatal(LifecycleError),
}

pub fn decide_ui_tick_status(
    command_channel_failed: bool,
    texture_poisoned: bool,
    status_poisoned: bool,
    status_line: Option<String>,
) -> UiTickStatusDecision {
    if command_channel_failed {
        return UiTickStatusDecision::ShowFatal(LifecycleError::CommandChannelClosed);
    }
    if texture_poisoned || status_poisoned {
        return UiTickStatusDecision::ShowFatal(LifecycleError::SlotPoisoned);
    }
    if let Some(line) = status_line {
        return UiTickStatusDecision::ShowStatus(line);
    }
    UiTickStatusDecision::KeepPrevious
}

pub fn format_status_snapshot(snapshot: &StatusSnapshot) -> String {
    let err_suffix = snapshot
        .last_error
        .as_ref()
        .map(|e| format!(" err={e}"))
        .unwrap_or_default();
    format!(
        "state={:?} tex_create={} try_from={} content_tick={} ui_set={}{err_suffix}",
        snapshot.state,
        snapshot.counters.texture_create_count,
        snapshot.counters.image_try_from_count,
        snapshot.counters.content_update_count,
        snapshot.counters.ui_property_set_count,
    )
}

/// `LatestSlot::replace` が `SlotPoisoned` を返したとき worker/UI 共通の収束先。
pub fn converge_mailbox_slot_poison(
    engine: &mut LifecycleEngine,
    status_slot: &LatestSlot<StatusSnapshot>,
) {
    engine.mark_failed();
    let _ = status_slot.replace(StatusSnapshot {
        state: LifecycleState::Failed,
        counters: engine.counters(),
        last_error: Some(LifecycleError::SlotPoisoned),
    });
}

impl<T> Default for LatestSlot<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// workerが長期保持する更新用texture。Slintへはclone handleを渡す。
pub struct RetainedRgbaTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub texture_generation: u64,
    content_phase: u64,
}

impl RetainedRgbaTexture {
    fn new(
        gpu: &GpuCtx,
        width: u32,
        height: u32,
        generation: u64,
        counters: &mut ResourceCounters,
    ) -> Self {
        let texture = gpu
            .device
            .create_texture(&rgba_texture_descriptor(width, height));
        counters.texture_create_count += 1;
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut retained = Self {
            texture,
            view,
            width,
            height,
            texture_generation: generation,
            content_phase: 0,
        };
        retained.clear_content(gpu, counters);
        retained
    }

    fn clear_content(&mut self, gpu: &GpuCtx, counters: &mut ResourceCounters) {
        let (r, g, b) = phase_to_rgb(self.content_phase);
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pv1-content-clear-encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pv1-content-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(r) / 255.0,
                            g: f64::from(g) / 255.0,
                            b: f64::from(b) / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }
        gpu.queue.submit(Some(encoder.finish()));
        counters.content_update_count += 1;
        self.content_phase += 1;
    }

    pub fn clone_texture_handle(&self) -> wgpu::Texture {
        self.texture.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventOutcome {
    pub state: LifecycleState,
    /// UIが `Image::try_from` + property set 後に `record_display_bound` すべきか
    pub needs_image_rebind: bool,
}

pub struct LifecycleEngine {
    state: LifecycleState,
    counters: ResourceCounters,
    retained: Option<RetainedRgbaTexture>,
}

impl LifecycleEngine {
    pub fn new(gpu: &GpuCtx, width: u32, height: u32) -> Result<Self, LifecycleError> {
        let mut counters = ResourceCounters::default();
        let retained = RetainedRgbaTexture::new(gpu, width, height, 1, &mut counters);
        Ok(Self {
            state: LifecycleState::Booting,
            counters,
            retained: Some(retained),
        })
    }

    pub fn state(&self) -> LifecycleState {
        self.state
    }

    pub fn counters(&self) -> ResourceCounters {
        self.counters
    }

    pub fn texture_generation(&self) -> Option<u64> {
        self.retained.as_ref().map(|r| r.texture_generation)
    }

    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.retained.as_ref().map(|r| (r.width, r.height))
    }

    pub fn boot_to_ready(&mut self) -> EventOutcome {
        if self.state == LifecycleState::Booting {
            self.state = LifecycleState::Ready;
        }
        EventOutcome {
            state: self.state,
            needs_image_rebind: false,
        }
    }

    pub fn apply_event(
        &mut self,
        gpu: &GpuCtx,
        event: LifecycleEvent,
    ) -> Result<EventOutcome, LifecycleError> {
        if self.state == LifecycleState::Failed {
            return Err(LifecycleError::Failed);
        }
        let _retained = self
            .retained
            .as_mut()
            .ok_or(LifecycleError::NotInitialized)?;

        match event {
            LifecycleEvent::ContentTick => {
                let allowed = matches!(
                    self.state,
                    LifecycleState::Ready
                        | LifecycleState::Displaying
                        | LifecycleState::Hidden
                        | LifecycleState::Minimized
                        | LifecycleState::Restored
                        | LifecycleState::Regenerated
                );
                if !allowed {
                    return Err(LifecycleError::InvalidTransition {
                        state: self.state,
                        event: "ContentTick",
                    });
                }
                self.retained
                    .as_mut()
                    .ok_or(LifecycleError::NotInitialized)?
                    .clear_content(gpu, &mut self.counters);
                Ok(EventOutcome {
                    state: self.state,
                    needs_image_rebind: false,
                })
            }
            LifecycleEvent::Hide => {
                if self.state != LifecycleState::Displaying
                    && self.state != LifecycleState::Restored
                {
                    return Err(LifecycleError::InvalidTransition {
                        state: self.state,
                        event: "Hide",
                    });
                }
                self.state = LifecycleState::Hidden;
                Ok(EventOutcome {
                    state: self.state,
                    needs_image_rebind: false,
                })
            }
            LifecycleEvent::Show => {
                if self.state != LifecycleState::Hidden {
                    return Err(LifecycleError::InvalidTransition {
                        state: self.state,
                        event: "Show",
                    });
                }
                self.state = LifecycleState::Displaying;
                Ok(EventOutcome {
                    state: self.state,
                    needs_image_rebind: false,
                })
            }
            LifecycleEvent::Minimize => {
                if self.state != LifecycleState::Displaying
                    && self.state != LifecycleState::Restored
                {
                    return Err(LifecycleError::InvalidTransition {
                        state: self.state,
                        event: "Minimize",
                    });
                }
                self.state = LifecycleState::Minimized;
                Ok(EventOutcome {
                    state: self.state,
                    needs_image_rebind: false,
                })
            }
            LifecycleEvent::Restore => {
                if self.state != LifecycleState::Minimized {
                    return Err(LifecycleError::InvalidTransition {
                        state: self.state,
                        event: "Restore",
                    });
                }
                self.state = LifecycleState::Restored;
                Ok(EventOutcome {
                    state: self.state,
                    needs_image_rebind: false,
                })
            }
            LifecycleEvent::Resize { width, height } => self.apply_resize(gpu, width, height),
            LifecycleEvent::Regenerate => self.apply_regenerate(gpu),
        }
    }

    fn apply_resize(
        &mut self,
        gpu: &GpuCtx,
        width: u32,
        height: u32,
    ) -> Result<EventOutcome, LifecycleError> {
        let retained = self
            .retained
            .as_ref()
            .ok_or(LifecycleError::NotInitialized)?;
        if retained.width == width && retained.height == height {
            return Ok(EventOutcome {
                state: self.state,
                needs_image_rebind: false,
            });
        }
        if !resize_dimensions_allowed(gpu, width, height) {
            return Err(LifecycleError::InvalidResize { width, height });
        }

        let next_generation = retained.texture_generation + 1;
        let new_retained =
            RetainedRgbaTexture::new(gpu, width, height, next_generation, &mut self.counters);
        self.retained = Some(new_retained);
        self.state = LifecycleState::Regenerated;

        Ok(EventOutcome {
            state: self.state,
            needs_image_rebind: true,
        })
    }

    fn apply_regenerate(&mut self, gpu: &GpuCtx) -> Result<EventOutcome, LifecycleError> {
        let retained = self
            .retained
            .as_ref()
            .ok_or(LifecycleError::NotInitialized)?;
        let (width, height) = (retained.width, retained.height);
        let next_generation = retained.texture_generation + 1;
        let new_retained =
            RetainedRgbaTexture::new(gpu, width, height, next_generation, &mut self.counters);
        self.retained = Some(new_retained);
        self.state = LifecycleState::Regenerated;
        Ok(EventOutcome {
            state: self.state,
            needs_image_rebind: true,
        })
    }

    pub fn mark_failed(&mut self) {
        self.state = LifecycleState::Failed;
    }

    /// 振る舞い試験用: clone handle → try_from。counter は record_display_bound で進める。
    pub fn bind_display_image(&mut self) -> Result<slint::Image, LifecycleError> {
        let texture = self.clone_display_texture()?;
        slint::Image::try_from(texture).map_err(|_| LifecycleError::ImageBindFailed)
    }

    /// UI threadへ送る表示用handle。try_fromはUI側で行う。
    pub fn handoff_display_texture(&self) -> Result<wgpu::Texture, LifecycleError> {
        self.clone_display_texture()
    }

    /// `Image::try_from` + property set 成功後、generation 一致時だけ counter/state を進める。
    pub fn record_display_bound(&mut self, generation: u64) -> Result<(), LifecycleError> {
        if self.state == LifecycleState::Failed {
            return Err(LifecycleError::Failed);
        }
        let current = self
            .retained
            .as_ref()
            .ok_or(LifecycleError::NotInitialized)?
            .texture_generation;
        if generation != current {
            return Ok(());
        }
        self.counters.image_try_from_count += 1;
        self.counters.ui_property_set_count += 1;
        self.advance_display_state_after_bind();
        Ok(())
    }

    fn advance_display_state_after_bind(&mut self) {
        if matches!(
            self.state,
            LifecycleState::Ready | LifecycleState::Regenerated | LifecycleState::Restored
        ) {
            self.state = LifecycleState::Displaying;
        }
    }

    /// UI 側の `Image::try_from` 失敗を現世代にだけ反映する。
    pub fn record_display_bind_failed(&mut self, generation: u64) {
        if self.texture_generation() == Some(generation) {
            self.mark_failed();
        }
    }

    fn clone_display_texture(&self) -> Result<wgpu::Texture, LifecycleError> {
        Ok(self
            .retained
            .as_ref()
            .ok_or(LifecycleError::NotInitialized)?
            .clone_texture_handle())
    }
}

fn resize_dimensions_allowed(gpu: &GpuCtx, width: u32, height: u32) -> bool {
    if width == 0 || height == 0 {
        return false;
    }
    let max_dim = gpu.device.limits().max_texture_dimension_2d as u64;
    (width as u64) <= max_dim && (height as u64) <= max_dim
}

fn phase_to_rgb(phase: u64) -> (u8, u8, u8) {
    let hue = (phase % 360) as f32;
    hsv_to_rgb(hue, 0.75, 0.75)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (rf, gf, bf) = match (h as u32 / 60) % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((rf + m) * 255.0).round() as u8,
        ((gf + m) * 255.0).round() as u8,
        ((bf + m) * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod slot_tests {
    use super::*;
    use motolii_testkit::unavailable_dep;

    fn poison_slot<T>(slot: &LatestSlot<T>) {
        let _ = std::panic::catch_unwind(|| {
            let _guard = slot.inner.lock().unwrap();
            panic!("intentional mailbox poison");
        });
    }

    fn gpu_for_slot_test() -> Option<GpuCtx> {
        match GpuCtx::new_for_ui() {
            Ok((gpu, _parts)) => Some(gpu),
            Err(e) => {
                unavailable_dep("GPU adapter", &e.to_string());
                None
            }
        }
    }

    #[test]
    fn replace_slot_returns_slot_poisoned_after_recovery_write() {
        let slot = LatestSlot::new();
        poison_slot(&slot);
        let err = slot.replace(7).unwrap_err();
        assert_eq!(err, LifecycleError::SlotPoisoned);
        match slot.try_take() {
            SlotTake::Item(7) => {}
            other => panic!("expected Item(7) after poison recovery write, got {other:?}"),
        }
    }

    #[test]
    fn try_take_reports_poisoned_not_would_block() {
        let slot: LatestSlot<i32> = LatestSlot::new();
        poison_slot(&slot);
        assert_eq!(slot.try_take(), SlotTake::Poisoned);
        assert_ne!(slot.try_take(), SlotTake::WouldBlock);
    }

    #[test]
    fn try_take_clears_poison_so_next_tick_is_observable() {
        let slot: LatestSlot<i32> = LatestSlot::new();
        poison_slot(&slot);
        assert_eq!(slot.try_take(), SlotTake::Poisoned);
        assert_eq!(slot.try_take(), SlotTake::Empty);
    }

    #[test]
    fn ui_tick_fatal_overrides_stale_status_line() {
        let decision = decide_ui_tick_status(
            false,
            true,
            false,
            Some("state=Ready tex_create=1 try_from=0 content_tick=1 ui_set=0".into()),
        );
        assert_eq!(
            decision,
            UiTickStatusDecision::ShowFatal(LifecycleError::SlotPoisoned)
        );
    }

    #[test]
    fn ui_tick_status_line_used_when_no_poison() {
        let line = "state=Displaying tex_create=1 try_from=1 content_tick=2 ui_set=1".to_string();
        let decision = decide_ui_tick_status(false, false, false, Some(line.clone()));
        assert_eq!(decision, UiTickStatusDecision::ShowStatus(line));
    }

    #[test]
    fn ui_tick_command_channel_fatal_is_sticky_over_status_line() {
        let decision = decide_ui_tick_status(
            true,
            false,
            false,
            Some("state=Displaying tex_create=1 try_from=1 content_tick=2 ui_set=1".into()),
        );
        assert_eq!(
            decision,
            UiTickStatusDecision::ShowFatal(LifecycleError::CommandChannelClosed)
        );
    }

    #[test]
    fn record_run_preserves_human_verdicts() {
        let mut manifest = Pv1Manifest::skeleton_template();
        manifest.human_checks[0].verdict = Verdict::Pass;
        manifest.overall = Verdict::Pending;
        let counters = ResourceCounters {
            texture_create_count: 3,
            ..ResourceCounters::default()
        };

        manifest.record_run("unix:123".into(), counters, LifecycleState::Displaying);

        assert_eq!(manifest.human_checks[0].verdict, Verdict::Pass);
        assert_eq!(manifest.overall, Verdict::Pending);
        assert_eq!(manifest.last_counters, counters);
        assert_eq!(manifest.last_state, "displaying");
    }

    #[test]
    fn converge_mailbox_slot_poison_after_replace_error() {
        let Some(gpu) = gpu_for_slot_test() else {
            return;
        };
        let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
        engine.boot_to_ready();
        let status_slot: LatestSlot<StatusSnapshot> = LatestSlot::new();
        poison_slot(&status_slot);
        let poison_err = status_slot
            .replace(StatusSnapshot {
                state: engine.state(),
                counters: engine.counters(),
                last_error: None,
            })
            .unwrap_err();
        assert_eq!(poison_err, LifecycleError::SlotPoisoned);
        converge_mailbox_slot_poison(&mut engine, &status_slot);
        assert_eq!(engine.state(), LifecycleState::Failed);
        match status_slot.try_take() {
            SlotTake::Item(snapshot) => {
                assert_eq!(snapshot.state, LifecycleState::Failed);
                assert_eq!(snapshot.last_error, Some(LifecycleError::SlotPoisoned));
            }
            other => panic!("expected fatal snapshot after convergence, got {other:?}"),
        }
    }
}
