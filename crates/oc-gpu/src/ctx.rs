#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("no suitable GPU adapter found: {0}")]
    NoAdapter(String),
    #[error("device request failed: {0}")]
    Device(String),
    #[error("adapter does not satisfy compositor requirements: {0}")]
    Requirements(String),
}

/// コンポジタが必要とするデバイス要件の**単一の情報源**。
///
/// feature/limitはデバイス生成時にしか確定できない(後から足せない)ため、
/// UI(Slint)とコアが共有するデバイスは必ずこの要件で生成する(第2回レビュー#1)。
/// 要件を増やす時はここを変えれば、ヘッドレス経路とUI共有経路の両方に効く。
pub fn required_features() -> wgpu::Features {
    // 現時点で必須の追加featureはない。将来の候補(fp16ストレージ、
    // FLOAT32_FILTERABLE、TIMESTAMP_QUERY等)を足す時はここに追加し、
    // 全アダプタで使えるとは限らないものはoptional扱いの仕組みを併設すること。
    wgpu::Features::empty()
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

/// UI(Slint)へ渡すためのデバイス一式。
/// `slint::wgpu_29::WGPUConfiguration::Manual`にそのまま供給する。
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
/// - `new_for_ui()`: **要件を明示したデバイスを自前で作り、UI(Slint)に渡す**(M3の正規経路。
///   Slint任せのデバイス生成だとコンポジタのfeature/limitが有効化されない恐れがある)
/// - `from_device_queue()`: 既存device/queueの共有(要件検証は呼び出し側の責任。
///   可能なら`new_for_ui`を使うこと)
pub struct GpuCtx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: Option<wgpu::AdapterInfo>,
}

impl GpuCtx {
    pub fn new_headless() -> Result<Self, GpuError> {
        let (ctx, _parts) = pollster::block_on(Self::new_async())?;
        Ok(ctx)
    }

    /// コンポジタ要件でデバイスを生成し、Slintへ渡すパーツと共有GpuCtxを返す。
    pub fn new_for_ui() -> Result<(Self, UiDeviceParts), GpuError> {
        pollster::block_on(Self::new_async())
    }

    /// 既存のdevice/queueを共有する。要件(device_requirements)を満たしている保証は
    /// 呼び出し側にある。UI統合では`new_for_ui`を優先すること。
    pub fn from_device_queue(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            adapter_info: None,
        }
    }

    async fn new_async() -> Result<(Self, UiDeviceParts), GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
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
                label: Some("oc-gpu"),
                required_features: features,
                // アダプタの実力値をそのまま要求(最低ラインは検証済み)。
                // 固定値要求だと弱いGPUを無用に弾き、強いGPUの能力も使えない
                required_limits: adapter_limits,
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::Device(e.to_string()))?;

        let ctx = Self {
            device: device.clone(),
            queue: queue.clone(),
            adapter_info: Some(adapter_info),
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
