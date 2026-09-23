// as: motolii/crates/motolii-render/src/engine/texture.rs
fn production() {}

#[cfg(test)]
mod tests {
    #[test]
    fn a_test_reads_back_as_an_embedder_would() {
        let texture = device.create_texture(&desc);
        queue.submit([encoder.finish()]);
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}
