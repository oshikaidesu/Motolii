fn live(device: &wgpu::Device) {
    // ruleid: motolii-native-host-never-waits-the-gpu, motolii-rust-unbounded-gpu-wait
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    // ruleid: motolii-native-host-never-waits-the-gpu
    let _ = wait_for_gpu(device, "live");
    // ok: motolii-native-host-never-waits-the-gpu
    let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(std::time::Duration::ZERO) });
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // ok: motolii-native-host-never-waits-the-gpu, motolii-rust-unbounded-gpu-wait
        let _ = self.engine.gpu_device().poll(wgpu::PollType::wait_indefinitely());
    }
}

fn gpu(device: &wgpu::Device) {
    // ruleid: motolii-rust-raw-gpu-infrastructure
    let _ = device.create_render_pipeline(&desc);
    // ruleid: motolii-rust-raw-gpu-infrastructure
    let _ = device.create_compute_pipeline(&desc);
    // ruleid: motolii-rust-raw-gpu-infrastructure
    let _ = device.create_shader_module(desc);
    // ruleid: motolii-rust-raw-gpu-infrastructure
    let _ = device.create_bind_group_layout(&desc);
    // ok: motolii-rust-raw-gpu-infrastructure
    let _ = device.create_buffer(&desc);
    // ok: motolii-rust-raw-gpu-infrastructure
    let _ = ctx.gpu_resources.render_pipelines.get_or_create(ctx, &pipeline_desc);
}
