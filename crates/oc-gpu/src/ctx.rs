#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("no suitable GPU adapter found: {0}")]
    NoAdapter(String),
    #[error("device request failed: {0}")]
    Device(String),
}

/// wgpuのデバイス一式。
///
/// 生成方法は2つ:
/// - `new_headless()`: 自前でadapter/deviceを作る(CLI・テスト・書き出し用)
/// - `from_device_queue()`: **UI(Slint)が作ったdevice/queueを共有する**(M3のゼロコピー
///   埋め込みの要。UIとコアが同一デバイスでなければテクスチャは共有できない)
pub struct GpuCtx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: Option<wgpu::AdapterInfo>,
}

impl GpuCtx {
    pub fn new_headless() -> Result<Self, GpuError> {
        pollster::block_on(Self::new_headless_async())
    }

    /// 既存のdevice/queue(例: Slintのrendering notifierから得たもの)を共有する。
    pub fn from_device_queue(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            adapter_info: None,
        }
    }

    async fn new_headless_async() -> Result<Self, GpuError> {
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
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("oc-gpu"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::Device(e.to_string()))?;
        Ok(Self {
            device,
            queue,
            adapter_info: Some(adapter_info),
        })
    }
}
