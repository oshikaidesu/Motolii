use super::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameMeasurement {
    pub total_us: u64,
    pub resolve_us: u64,
    pub layer_build_us: u64,
    pub prepare_us: u64,
    pub submit_us: u64,
    pub wait_us: u64,
    pub readback_us: u64,
    pub final_submission_gpu_us: Option<f64>,
    pub gpu_status: &'static str,
}

pub(super) struct GpuMeasurement {
    buffer: wgpu::Buffer,
    ready: std::sync::mpsc::Receiver<bool>,
}

impl Compositor {
    pub(super) fn measure_pending_gpu(&mut self) -> Option<GpuMeasurement> {
        let required =
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        if !self.measurement_enabled || !self.ctx.device.features().contains(required) {
            return None;
        }
        let device = &self.ctx.device;
        let query = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("reflection-comparison"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reflection-comparison-resolve"),
            size: 16,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reflection-comparison-readback"),
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut begin = device.create_command_encoder(&Default::default());
        begin.write_timestamp(&query, 0);
        self.pending.insert(0, begin.finish());
        let mut end = device.create_command_encoder(&Default::default());
        end.write_timestamp(&query, 1);
        end.resolve_query_set(&query, 0..2, &resolve, 0);
        end.copy_buffer_to_buffer(&resolve, 0, &buffer, 0, 16);
        let (send, ready) = std::sync::mpsc::channel();
        end.map_buffer_on_submit(&buffer, wgpu::MapMode::Read, .., move |r| {
            let _ = send.send(r.is_ok());
        });
        self.pending.push(end.finish());
        Some(GpuMeasurement { buffer, ready })
    }
}

impl GpuMeasurement {
    pub(super) fn finish(self, period: f32) -> Result<f64, &'static str> {
        if !self.ready.try_recv().map_err(|_| "callback_not_ready")? {
            return Err("map_failed");
        }
        let data = self.buffer.slice(..).get_mapped_range();
        let start = u64::from_ne_bytes(data[0..8].try_into().unwrap());
        let end = u64::from_ne_bytes(data[8..16].try_into().unwrap());
        let elapsed = end.checked_sub(start).ok_or("non_monotonic_timestamp")? as f64
            * period as f64
            / 1000.0;
        drop(data);
        self.buffer.unmap();
        Ok(elapsed)
    }
}
