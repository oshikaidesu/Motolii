//! 裁定256 — 共有面検査は通す。書き込みは blit で先に通さない。

use motolii_compositor::{
    check_presentable_target, CompSpec, Compositor, CompositorError, HeadlessGpu, LayerWithPasses,
    ResolvedCamera, PRESENTABLE_FORMAT,
};

const W: u32 = 64;
const H: u32 = 64;

fn comp() -> CompSpec {
    CompSpec {
        width: W,
        height: H,
    }
}

fn with_device() -> (Compositor, wgpu::Device) {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let compositor = Compositor::with_device_using_headless_defaults(device.clone(), queue)
        .expect("with_device");
    (compositor, device)
}

fn texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("presentable-test"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn presentable(device: &wgpu::Device) -> wgpu::Texture {
    texture(
        device,
        W,
        H,
        PRESENTABLE_FORMAT,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    )
}

#[test]
fn presentable_target_accepts_host_spec() {
    let (_compositor, device) = with_device();
    check_presentable_target(&presentable(&device), comp()).expect("host spec");
}

#[test]
fn presentable_target_rejects_wrong_format() {
    let (_compositor, device) = with_device();
    let got = check_presentable_target(
        &texture(
            &device,
            W,
            H,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        ),
        comp(),
    );
    assert!(matches!(got, Err(CompositorError::PresentableFormat { .. })));
}

#[test]
fn presentable_target_rejects_wrong_size() {
    let (_compositor, device) = with_device();
    let got = check_presentable_target(
        &texture(
            &device,
            8,
            8,
            PRESENTABLE_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        ),
        comp(),
    );
    assert!(matches!(
        got,
        Err(CompositorError::PresentableSize {
            got: [8, 8],
            expected: [W, H]
        })
    ));
}

#[test]
fn presentable_target_rejects_missing_render_attachment() {
    let (_compositor, device) = with_device();
    let got = check_presentable_target(
        &texture(
            &device,
            W,
            H,
            PRESENTABLE_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING,
        ),
        comp(),
    );
    assert!(matches!(got, Err(CompositorError::PresentableUsage)));
}

#[test]
fn render_into_writes_the_external_target() {
    let (mut compositor, device) = with_device();
    let target = presentable(&device);
    compositor
        .render_into(&target, comp(), ResolvedCamera::default(), &[] as &[LayerWithPasses])
        .expect("external resolved へ直接書く");
}

#[test]
fn render_into_does_not_blit_before_external_resolved() {
    let (mut compositor, device) = with_device();
    let target = presentable(&device);
    let got = compositor.render_into(&target, comp(), ResolvedCamera::default(), &[] as &[LayerWithPasses]);
    assert!(
        matches!(got, Err(CompositorError::PresentableWriteNotWired)),
        "検査を通った共有面へ blit して先に通してはいけない: {got:?}"
    );
}
