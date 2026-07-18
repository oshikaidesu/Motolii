//! 外部作者が内部crateへ直接依存せず、A1S §2.1の公開面だけを使えることを固定する。

#[test]
fn facade_exports_resolve_from_external_paths() {
    let track = motolii_plugin::DataTrack {
        start: motolii_plugin::RationalTime::ZERO,
        sample_rate: motolii_plugin::Fps::try_new(1, 1).unwrap(),
        values: vec![motolii_plugin::Value::F64(1.0)],
    };
    assert_eq!(track.values.len(), 1);

    let value = motolii_plugin::Value::F64(0.5);
    assert_eq!(value.as_f64(), Some(0.5));

    let key = motolii_plugin::PipelineCacheKey {
        id: "vism-a1-1",
        wgsl: "@compute fn main() {}",
    };
    assert_eq!(key.id, "vism-a1-1");

    fn accepts_gpu_facade(
        _gpu: &motolii_plugin::GpuCtx,
        _cache: &mut motolii_plugin::PipelineCache,
        _key: &motolii_plugin::PipelineCacheKey,
    ) {
    }

    let camera = motolii_plugin::CompCamera::default();

    let fps = motolii_plugin::Fps::try_new(30, 1).unwrap();
    assert_eq!(fps.num(), 30);

    // PixelFormat等を追加公開せず、FrameDesc自体の到達性だけを証明する。
    fn accepts_frame_desc(_: motolii_plugin::FrameDesc) {}

    let quality = motolii_plugin::Quality::DRAFT;
    assert_eq!(quality.resolution_scale, 2);

    let t = motolii_plugin::RationalTime::from_seconds(1);
    assert!(t > motolii_plugin::RationalTime::ZERO);

    let format = motolii_plugin::wgpu::TextureFormat::Rgba8Unorm;
    assert_eq!(format, motolii_plugin::wgpu::TextureFormat::Rgba8Unorm);

    let uniform = [1.0f32, 0.0, 0.0, 0.0];
    let bytes = motolii_plugin::bytemuck::bytes_of(&uniform);
    assert_eq!(bytes.len(), 16);

    let _ = (
        camera,
        fps,
        quality,
        t,
        format,
        accepts_gpu_facade,
        accepts_frame_desc,
    );
}
