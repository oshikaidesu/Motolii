
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

        let mut descriptor = caps.device_descriptor();
        // rerun は WebGL2 相当の上限で device を頼む(compute 無し)。選択の籠を GPU で畳む
        // (`selection_bounds`)分だけ、adapter が持つ範囲で上限を借りる。無い機械では籠は写した点へ戻る。
        {
            let have = adapter.limits();
            let want = &mut descriptor.required_limits;
            want.max_storage_buffers_per_shader_stage = have.max_storage_buffers_per_shader_stage.min(1);
            want.max_storage_buffer_binding_size = have.max_storage_buffer_binding_size.min(1 << 16);
            want.max_compute_workgroup_storage_size = have.max_compute_workgroup_storage_size.min(16384);
            want.max_compute_invocations_per_workgroup = have.max_compute_invocations_per_workgroup.min(256);
            want.max_compute_workgroup_size_x = have.max_compute_workgroup_size_x.min(16);
            want.max_compute_workgroup_size_y = have.max_compute_workgroup_size_y.min(16);
            want.max_compute_workgroup_size_z = have.max_compute_workgroup_size_z.min(1);
            want.max_compute_workgroups_per_dimension = have.max_compute_workgroups_per_dimension.min(65535);
        }
        let timestamps = wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        if adapter.features().contains(timestamps) { descriptor.required_features |= timestamps; }
        let (device, queue) = pollster::block_on(adapter.request_device(&descriptor))
            .map_err(|e| HeadlessError::Device(e.to_string()))?;

        Ok(Self {
            adapter,
            device,
            queue,
        })
    }
}
