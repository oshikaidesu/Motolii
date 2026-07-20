use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("no suitable GPU adapter found: {0}")]
    NoAdapter(String),
    #[error("device request failed: {0}")]
    Device(String),
    #[error("adapter does not satisfy compositor requirements: {0}")]
    Requirements(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuOrigin {
    /// CLI・テスト・書き出し用の専用デバイス。同期読み戻し可。
    Headless,
    /// UI shell(egui)と共有中のデバイス。`poll(Wait)`/`download_rgba`は禁止。
    UiShared,
}

#[derive(Debug, thiserror::Error)]
pub enum GpuRuntimeError {
    #[error("GPU device was lost: {reason}")]
    DeviceLost { reason: String },
    #[error("uncaptured GPU error: {0}")]
    Uncaptured(String),
    #[error("GPU buffer map failed: {0}")]
    Map(String),
    #[error("GPU poll failed: {0}")]
    Poll(String),
    #[error("GPU operation timed out after {0:?}")]
    Timeout(std::time::Duration),
    #[error(
        "synchronous GPU readback (poll Wait / download_rgba) is forbidden on UI-shared device; use GpuCtx::new_headless() for export"
    )]
    SyncReadbackForbidden,
}

/// コンポジタが必要とするデバイス要件の**単一の情報源**。
///
/// feature/limitはデバイス生成時にしか確定できない(後から足せない)ため、
/// UI shell(egui)とコアが共有するデバイスは必ずこの要件で生成する(第2回レビュー#1)。
/// 要件を増やす時はここを変えれば、ヘッドレス経路とUI共有経路の両方に効く。
pub fn required_features() -> wgpu::Features {
    // 現時点で必須の追加featureはない。将来の候補(fp16ストレージ、
    // FLOAT32_FILTERABLE、TIMESTAMP_QUERY等)を足す時はここに追加し、
    // 全アダプタで使えるとは限らないものはoptional扱いの仕組みを併設すること。
    wgpu::Features::empty()
}

/// DRS等で使うが全アダプタ必須ではないfeature(D5縮退規約)。
pub fn optional_features() -> wgpu::Features {
    wgpu::Features::TIMESTAMP_QUERY
}

/// 自動DRSが利用可能か(timestamp query非対応時は無効+ドロップ継続)。
pub fn drs_available(device: &wgpu::Device) -> bool {
    device.features().contains(wgpu::Features::TIMESTAMP_QUERY)
}

/// コンポジタが最低限必要とするlimitの検証(第3回レビュー#2)。
///
/// 「固定値で要求」だと弱いGPUでrequest_deviceが原因不明に失敗するため、
/// **最低ライン(4K素材が扱える4096)だけを明確に検証**し、実際の要求は
/// アダプタの実力値をそのまま使う(=満たせる範囲で最大、clampの最単純形)。
pub fn check_minimum_limits(l: &wgpu::Limits) -> Result<(), String> {
    const MIN_TEX_2D: u32 = 4096; // 4K UHD(3840)+余白
    if l.max_texture_dimension_2d < MIN_TEX_2D {
        return Err(format!(
            "max_texture_dimension_2d {} < required {MIN_TEX_2D} (cannot handle 4K footage)",
            l.max_texture_dimension_2d
        ));
    }
    Ok(())
}

/// UI shell(egui)へ渡すためのデバイス一式。
/// egui-wgpuの`WgpuSetup::Existing`にそのまま供給する。
pub struct UiDeviceParts {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

/// wgpuのデバイス一式。
///
/// 生成方法:
/// - `new_headless()`: 自前でadapter/deviceを作る(CLI・テスト・書き出し用)
/// - `new_for_ui()`: **要件を明示したデバイスを自前で作り、UI shell(egui)に渡す**(M3の正規経路。
///   toolkit任せのデバイス生成だとコンポジタのfeature/limitが有効化されない恐れがある)
/// - `from_device_queue()`: 既存device/queueの共有(要件検証は呼び出し側の責任。
///   可能なら`new_for_ui`を使うこと)
pub struct GpuCtx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: Option<wgpu::AdapterInfo>,
    origin: GpuOrigin,
    runtime_state: Arc<Mutex<GpuRuntimeState>>,
}

impl GpuCtx {
    pub fn new_headless() -> Result<Self, GpuError> {
        let (ctx, _parts) = pollster::block_on(Self::new_async(GpuOrigin::Headless))?;
        Ok(ctx)
    }

    /// コンポジタ要件でデバイスを生成し、UI shellへ渡すパーツと共有GpuCtxを返す。
    pub fn new_for_ui() -> Result<(Self, UiDeviceParts), GpuError> {
        pollster::block_on(Self::new_async(GpuOrigin::UiShared))
    }

    pub fn origin(&self) -> GpuOrigin {
        self.origin
    }

    /// M3規約3: UI共有デバイスでは`poll(Wait)`を呼ばない。
    pub fn poll_wait(&self, timeout: Option<std::time::Duration>) -> Result<(), GpuRuntimeError> {
        self.ensure_sync_readback_allowed()?;
        match self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout,
        }) {
            Ok(_) | Err(wgpu::PollError::Timeout) => Ok(()),
            Err(e) => Err(GpuRuntimeError::Poll(e.to_string())),
        }
    }

    pub(crate) fn ensure_sync_readback_allowed(&self) -> Result<(), GpuRuntimeError> {
        if self.origin == GpuOrigin::UiShared {
            return Err(GpuRuntimeError::SyncReadbackForbidden);
        }
        Ok(())
    }

    /// 既存のdevice/queueを共有する。要件(device_requirements)を満たしている保証は
    /// 呼び出し側にある。UI統合では`new_for_ui`を優先すること。
    ///
    /// **コールバックスロット制約**: wgpuの`set_device_lost_callback`と
    /// `on_uncaptured_error`はデバイスあたり1スロットのみ(後から登録すると置換)。
    /// 本関数はランタイム監視用ハンドラを登録するため、M3でホスト(egui等)が
    /// 別ハンドラを持つ構成にする場合は登録の所有者を1箇所に集約すること。
    pub fn from_device_queue(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self::from_device_queue_with_origin(device, queue, GpuOrigin::UiShared)
    }

    /// 既存device/queueを共有する。`origin`はデバイスの用途に合わせて呼び出し側が指定する。
    pub fn from_device_queue_with_origin(
        device: wgpu::Device,
        queue: wgpu::Queue,
        origin: GpuOrigin,
    ) -> Self {
        let runtime_state = install_runtime_handlers(&device);
        Self {
            device,
            queue,
            adapter_info: None,
            origin,
            runtime_state,
        }
    }

    pub fn check_health(&self) -> Result<(), GpuRuntimeError> {
        let mut state = self
            .runtime_state
            .lock()
            .expect("GPU runtime state poisoned");
        if let Some(reason) = &state.device_lost {
            return Err(GpuRuntimeError::DeviceLost {
                reason: reason.clone(),
            });
        }
        // 一過性のuncaptured errorは報告後にクリアし、次の操作で復帰できるようにする。
        if let Some(error) = state.uncaptured_error.take() {
            return Err(GpuRuntimeError::Uncaptured(error));
        }
        Ok(())
    }

    /// テスト専用: 次の`check_health`が`Uncaptured`を返すよう注入する。
    #[doc(hidden)]
    pub fn inject_uncaptured_error_for_test(&self, message: &str) {
        self.runtime_state
            .lock()
            .expect("GPU runtime state poisoned")
            .uncaptured_error = Some(message.to_string());
    }

    async fn new_async(origin: GpuOrigin) -> Result<(Self, UiDeviceParts), GpuError> {
        // OOMの失敗モードをdevice lost(全リソース喪失)より手前の、型付きエラーで
        // 捕捉できるリソース作成失敗に寄せる(安全に品質を落として継続する前提)。
        // 作成失敗の閾値をdevice loss側より低くし、必ず先に失敗の余地を作る。
        // 注意: wgpu 29ではD3D12とVulkan(VK_EXT_memory_budget有効時)のみ実効。
        // Metalでは無効のため、macOSのOOM検知はdevice lost/uncapturedハンドラ頼み。
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                for_resource_creation: Some(90),
                for_device_loss: Some(95),
            },
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| GpuError::NoAdapter(e.to_string()))?;
        let adapter_info = adapter.get_info();

        // 要求前にアダプタ能力(feature/limit両方)を検証し、足りない場合は
        // 明確なエラーにする(デバイス生成後に発覚して作り直せない、を防ぐ)
        let features = required_features();
        let missing = features - adapter.features();
        if !missing.is_empty() {
            return Err(GpuError::Requirements(format!(
                "missing features: {missing:?} (adapter: {})",
                adapter_info.name
            )));
        }
        let adapter_limits = adapter.limits();
        check_minimum_limits(&adapter_limits)
            .map_err(|e| GpuError::Requirements(format!("{e} (adapter: {})", adapter_info.name)))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("motolii-gpu"),
                required_features: features | (optional_features() & adapter.features()),
                // アダプタの実力値をそのまま要求(最低ラインは検証済み)。
                // 固定値要求だと弱いGPUを無用に弾き、強いGPUの能力も使えない
                required_limits: adapter_limits,
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::Device(e.to_string()))?;

        let runtime_state = install_runtime_handlers(&device);
        let ctx = Self {
            device: device.clone(),
            queue: queue.clone(),
            adapter_info: Some(adapter_info),
            origin,
            runtime_state,
        };
        let parts = UiDeviceParts {
            instance,
            adapter,
            device,
            queue,
        };
        Ok((ctx, parts))
    }
}

#[derive(Default, Debug)]
struct GpuRuntimeState {
    device_lost: Option<String>,
    uncaptured_error: Option<String>,
}

fn install_runtime_handlers(device: &wgpu::Device) -> Arc<Mutex<GpuRuntimeState>> {
    let state = Arc::new(Mutex::new(GpuRuntimeState::default()));

    let lost_state = Arc::clone(&state);
    device.set_device_lost_callback(move |reason, message| {
        eprintln!("GPU device lost ({reason:?}): {message}");
        lost_state
            .lock()
            .expect("GPU runtime state poisoned")
            .device_lost = Some(format!("{reason:?}: {message}"));
    });

    let error_state = Arc::clone(&state);
    device.on_uncaptured_error(Arc::new(move |error| {
        eprintln!("uncaptured GPU error: {error}");
        error_state
            .lock()
            .expect("GPU runtime state poisoned")
            .uncaptured_error = Some(error.to_string());
    }));

    state
}
