
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
        // GPU tests must be diagnosable. In particular, do not silently touch
        // every compiled backend when the caller pinned one with WGPU_BACKEND:
        // a broken driver/backend can otherwise hang before the first assertion.
        let backends = wgpu::Backends::from_env().unwrap_or(wgpu::Backends::all());
        let adapter_name = std::env::var("WGPU_ADAPTER_NAME").ok().filter(|name| !name.trim().is_empty());
        eprintln!(
            "MOTOLII_GPU_INIT phase=enumerate backends={backends:?} adapter_filter={:?}",
            adapter_name.as_deref().unwrap_or("<any>")
        );
        let mut instance_descriptor = device_caps::testing_instance_descriptor();
        instance_descriptor.backends = backends;
        let instance = wgpu::Instance::new(instance_descriptor);
        let adapters = pollster::block_on(instance.enumerate_adapters(backends));
        let adapters: Vec<_> = match adapter_name.as_deref() {
            Some(needle) => {
                let needle = needle.to_ascii_lowercase();
                adapters.into_iter().filter(|adapter| adapter.get_info().name.to_ascii_lowercase().contains(&needle)).collect()
            }
            None => adapters,
        };
        eprintln!("MOTOLII_GPU_INIT phase=select candidates={}", adapters.len());
        let adapter = device_caps::select_adapter(&adapters, backends, None)
            .map_err(HeadlessError::NoAdapter)?;
        let info = adapter.get_info();
        eprintln!(
            "MOTOLII_GPU_INIT phase=selected name={:?} backend={:?} device_type={:?} driver={:?} driver_info={:?}",
            info.name, info.backend, info.device_type, info.driver, info.driver_info
        );

        let caps = DeviceCaps::from_adapter(&adapter)
            .map_err(|e| HeadlessError::InsufficientCaps(e.to_string()))?;

        let mut descriptor = caps.device_descriptor();
        // rerun は WebGL2 相当の上限で device を頼む(compute 無し)。選択の籠を GPU で畳む(`selection_bounds`)分と、
        // 箱のブロック(`block_program`: 箱の並びとずれの 2 本、物の数に天井を作らない)の分だけ、adapter が持つ範囲で上限を借りる。
        {
            let have = adapter.limits();
            let want = &mut descriptor.required_limits;
            want.max_storage_buffers_per_shader_stage = have.max_storage_buffers_per_shader_stage.min(8);
            // vism の fx は欄 1 つに uniform を 1 本束ねる。WebGL2 相当の 11 本では欄 9 個で束ねが無効になり、層が黙って描かれない。
            want.max_uniform_buffers_per_shader_stage = want.max_uniform_buffers_per_shader_stage.max(have.max_uniform_buffers_per_shader_stage.min(31));
            want.max_storage_buffer_binding_size = have.max_storage_buffer_binding_size.min(1 << 30);
            want.max_buffer_size = want.max_buffer_size.max(have.max_buffer_size.min(1 << 30));
            want.max_compute_workgroup_storage_size = have.max_compute_workgroup_storage_size.min(16384);
            want.max_compute_invocations_per_workgroup = have.max_compute_invocations_per_workgroup.min(256);
            want.max_compute_workgroup_size_x = have.max_compute_workgroup_size_x.min(256);
            want.max_compute_workgroup_size_y = have.max_compute_workgroup_size_y.min(16);
            want.max_compute_workgroup_size_z = have.max_compute_workgroup_size_z.min(1);
            want.max_compute_workgroups_per_dimension = have.max_compute_workgroups_per_dimension.min(65535);
        }
        let timestamps = wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        if adapter.features().contains(timestamps) { descriptor.required_features |= timestamps; }
        eprintln!("MOTOLII_GPU_INIT phase=request_device");
        let (device, queue) = pollster::block_on(adapter.request_device(&descriptor))
            .map_err(|e| HeadlessError::Device(e.to_string()))?;
        eprintln!("MOTOLII_GPU_INIT phase=ready");

        Ok(Self {
            adapter,
            device,
            queue,
        })
    }
}
