//! 窓を持たない GPU。**instance / adapter / limits は全部上流の物を使う**。
//!
//! ここで自前の limits を書くと、rerun の shader が要求する床(`max_bind_groups` 等)と
//! ずれた時に原因が分からなくなる。iced fork の seam 2 が同じ床の話だった。

use re_renderer::device_caps::{self, DeviceCaps};

#[derive(Debug, thiserror::Error)]
pub enum HeadlessError {
    #[error("GPU adapter が見つからない: {0}")]
    NoAdapter(String),
    #[error("この GPU は re_renderer の要求を満たさない: {0}")]
    InsufficientCaps(String),
    #[error("device を作れない: {0}")]
    Device(String),
}

pub struct HeadlessGpu {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl HeadlessGpu {
    pub fn new() -> Result<Self, HeadlessError> {
        let instance = wgpu::Instance::new(device_caps::testing_instance_descriptor());
        let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        let adapter = device_caps::select_adapter(&adapters, wgpu::Backends::all(), None)
            .map_err(HeadlessError::NoAdapter)?;

        let caps = DeviceCaps::from_adapter(&adapter)
            .map_err(|e| HeadlessError::InsufficientCaps(e.to_string()))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&caps.device_descriptor()))
            .map_err(|e| HeadlessError::Device(e.to_string()))?;

        Ok(Self {
            adapter,
            device,
            queue,
        })
    }
}
