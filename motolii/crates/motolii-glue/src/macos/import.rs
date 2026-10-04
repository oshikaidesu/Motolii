use std::ptr::NonNull;

use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CFRetained;
use objc2_core_video::{
    kCVReturnSuccess, CVMetalTexture, CVMetalTextureCache, CVMetalTextureGetTexture,
    CVPixelBuffer, CVPixelBufferGetHeightOfPlane, CVPixelBufferGetWidthOfPlane,
};
use objc2_metal::{MTLDevice, MTLPixelFormat, MTLTexture, MTLTextureType};

use super::decode::Sample;

pub(super) struct Imported {
    pub(super) y: wgpu::Texture,
    pub(super) uv: wgpu::Texture,
    _y_ref: CFRetained<CVMetalTexture>,
    _uv_ref: CFRetained<CVMetalTexture>,
    _pixel: CFRetained<CVPixelBuffer>,
    _sample: objc2::rc::Retained<objc2_core_media::CMSampleBuffer>,
}

pub(super) struct Importer {
    cache: CFRetained<CVMetalTextureCache>,
}

impl Importer {
    pub(super) fn new(device: &wgpu::Device) -> Result<Self, String> {
        let metal = metal_device(device)?;
        let mut cache = std::ptr::null_mut();
        let status = unsafe {
            CVMetalTextureCache::create(
                None,
                None,
                &metal,
                None,
                NonNull::new(&mut cache).unwrap(),
            )
        };
        if status != kCVReturnSuccess {
            return Err(format!("CVMetalTextureCacheCreate {status}"));
        }
        let cache = NonNull::new(cache).ok_or("texture cache was null")?;
        Ok(Self {
            cache: unsafe { CFRetained::from_raw(cache) },
        })
    }

    pub(super) fn import(&self, device: &wgpu::Device, sample: &Sample) -> Result<Imported, String> {
        let y_ref = plane(&self.cache, &sample.pixel, 0, MTLPixelFormat::R8Unorm)?;
        let uv_ref = plane(&self.cache, &sample.pixel, 1, MTLPixelFormat::RG8Unorm)?;
        let y = hal_texture(device, &y_ref, wgpu::TextureFormat::R8Unorm)?;
        let uv = hal_texture(device, &uv_ref, wgpu::TextureFormat::Rg8Unorm)?;
        Ok(Imported {
            y,
            uv,
            _y_ref: y_ref,
            _uv_ref: uv_ref,
            _pixel: sample.pixel.clone(),
            _sample: sample.buffer.clone(),
        })
    }

    pub(super) fn flush(&self) {
        self.cache.flush(0);
    }
}

fn plane(
    cache: &CVMetalTextureCache,
    pixel: &CVPixelBuffer,
    index: usize,
    format: MTLPixelFormat,
) -> Result<CFRetained<CVMetalTexture>, String> {
    let width = CVPixelBufferGetWidthOfPlane(pixel, index);
    let height = CVPixelBufferGetHeightOfPlane(pixel, index);
    if width == 0 || height == 0 {
        return Err(format!("plane {index} is empty"));
    }
    let mut texture = std::ptr::null_mut();
    let status = unsafe {
        CVMetalTextureCache::create_texture_from_image(
            None,
            cache,
            pixel,
            None,
            format,
            width,
            height,
            index,
            NonNull::new(&mut texture).unwrap(),
        )
    };
    if status != kCVReturnSuccess {
        return Err(format!("CVMetalTextureCacheCreateTextureFromImage {status}"));
    }
    let texture = NonNull::new(texture).ok_or("metal texture was null")?;
    Ok(unsafe { CFRetained::from_raw(texture) })
}

fn hal_texture(
    device: &wgpu::Device,
    texture: &CVMetalTexture,
    format: wgpu::TextureFormat,
) -> Result<wgpu::Texture, String> {
    let metal = CVMetalTextureGetTexture(texture).ok_or("CVMetalTexture has no MTLTexture")?;
    let width = metal.width() as u32;
    let height = metal.height() as u32;
    let hal = unsafe {
        wgpu_hal::metal::Device::texture_from_raw(
            metal,
            format,
            MTLTextureType::Type2D,
            1,
            1,
            wgpu_hal::CopyExtent {
                width,
                height,
                depth: 1,
            },
        )
    };
    Ok(unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Metal>(
            hal,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        )
    })
}

fn metal_device(
    device: &wgpu::Device,
) -> Result<objc2::rc::Retained<ProtocolObject<dyn MTLDevice>>, String> {
    let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }.ok_or("device is not Metal")?;
    Ok(hal.raw_device().clone())
}
