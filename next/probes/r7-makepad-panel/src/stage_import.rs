//! OS handle → wgpu Texture。`stage_surface` の Host 契約には出さない。

use crate::stage_surface::{SharedOsHandle, SharedPixelFormat, SharedSurfaceDesc};

pub fn import_presentable(
    device: &wgpu::Device,
    desc: SharedSurfaceDesc,
    handle: SharedOsHandle,
) -> Option<wgpu::Texture> {
    if desc.format != SharedPixelFormat::Rgba8Srgb {
        return None;
    }
    match handle {
        SharedOsHandle::IoSurfaceId(id) => import_iosurface(device, desc, id),
        SharedOsHandle::DxgiSharedHandle(_) | SharedOsHandle::DmaBufFd(_) => None,
    }
}

#[cfg(target_os = "macos")]
fn import_iosurface(
    device: &wgpu::Device,
    desc: SharedSurfaceDesc,
    id: u32,
) -> Option<wgpu::Texture> {
    if id == 0 {
        return None;
    }
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        import_iosurface_hal(device, desc.width, desc.height, id)
    }))
    .ok()
    .flatten()
}

#[cfg(not(target_os = "macos"))]
fn import_iosurface(
    _device: &wgpu::Device,
    _desc: SharedSurfaceDesc,
    _id: u32,
) -> Option<wgpu::Texture> {
    None
}

/// Metal が要求する `^{__IOSurface=}`。`*mut c_void` (`^v`) で送ると debug で abort する。
#[cfg(target_os = "macos")]
#[repr(C)]
struct IoSurface {
    _private: [u8; 0],
}

#[cfg(target_os = "macos")]
unsafe impl objc2::encode::RefEncode for IoSurface {
    const ENCODING_REF: objc2::encode::Encoding =
        objc2::encode::Encoding::Pointer(&objc2::encode::Encoding::Struct("__IOSurface", &[]));
}

#[cfg(target_os = "macos")]
unsafe fn import_iosurface_hal(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    id: u32,
) -> Option<wgpu::Texture> {
    #[link(name = "IOSurface", kind = "framework")]
    extern "C" {
        fn IOSurfaceLookup(id: u32) -> *mut IoSurface;
        fn IOSurfaceDecrementUseCount(buffer: *mut IoSurface);
    }

    let surface = IOSurfaceLookup(id);
    if surface.is_null() {
        return None;
    }

    let Some(hal_device) = device.as_hal::<wgpu::hal::api::Metal>() else {
        IOSurfaceDecrementUseCount(surface);
        return None;
    };
    let mtl_device = hal_device.raw_device().clone();

    let desc_obj: *mut objc2::runtime::AnyObject =
        objc2::msg_send![objc2::class!(MTLTextureDescriptor), new];
    if desc_obj.is_null() {
        IOSurfaceDecrementUseCount(surface);
        return None;
    }
    let _: () = objc2::msg_send![desc_obj, setTextureType: 2u64];
    let _: () = objc2::msg_send![desc_obj, setWidth: width as u64];
    let _: () = objc2::msg_send![desc_obj, setHeight: height as u64];
    let _: () = objc2::msg_send![desc_obj, setDepth: 1u64];
    let _: () = objc2::msg_send![desc_obj, setStorageMode: 2u64];
    let _: () = objc2::msg_send![desc_obj, setUsage: 5u64];
    let _: () = objc2::msg_send![desc_obj, setPixelFormat: 71u64];

    let tex: *mut objc2::runtime::AnyObject = objc2::msg_send![
        &*mtl_device,
        newTextureWithDescriptor: desc_obj,
        iosurface: surface,
        plane: 0u64
    ];
    let _: () = objc2::msg_send![desc_obj, release];
    IOSurfaceDecrementUseCount(surface);
    if tex.is_null() {
        return None;
    }

    let retained = objc2::rc::Retained::from_raw(tex)?;
    let mtl: objc2::rc::Retained<objc2::runtime::ProtocolObject<dyn objc2_metal::MTLTexture>> =
        unsafe { objc2::rc::Retained::cast_unchecked(retained) };
    let hal_texture = wgpu::hal::metal::Device::texture_from_raw(
        mtl,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        objc2_metal::MTLTextureType::Type2D,
        1,
        1,
        wgpu::hal::CopyExtent {
            width,
            height,
            depth: 1,
        },
    );
    Some(
        device.create_texture_from_hal::<wgpu::hal::api::Metal>(
            hal_texture,
            &wgpu::TextureDescriptor {
                label: Some("motolii-presentable"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        ),
    )
}
