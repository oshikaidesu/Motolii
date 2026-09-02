
use crate::doc::vector::Raster;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

impl Coverage {
    pub fn full(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: vec![255u8; (width as usize) * (height as usize)],
        }
    }

    pub fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: vec![0u8; (width as usize) * (height as usize)],
        }
    }

    pub fn from_raster_alpha(raster: &Raster) -> Self {
        let bytes = raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .map(|px| px[3])
            .collect();
        Self {
            width: raster.width,
            height: raster.height,
            bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("coverage の寸法が合わない: {aw}x{ah} と {bw}x{bh}")]
pub struct CoverageSizeMismatch {
    pub aw: u32,
    pub ah: u32,
    pub bw: u32,
    pub bh: u32,
}

fn zip_pixels(
    a: &Coverage,
    b: &Coverage,
    f: impl Fn(u8, u8) -> u8,
) -> Result<Coverage, CoverageSizeMismatch> {
    if a.width != b.width || a.height != b.height {
        return Err(CoverageSizeMismatch {
            aw: a.width,
            ah: a.height,
            bw: b.width,
            bh: b.height,
        });
    }
    let bytes = a
        .bytes
        .iter()
        .zip(b.bytes.iter())
        .map(|(&x, &y)| f(x, y))
        .collect();
    Ok(Coverage {
        width: a.width,
        height: a.height,
        bytes,
    })
}

pub fn add(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::saturating_add)
}

pub fn subtract(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::saturating_sub)
}

pub fn intersect(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::min)
}

pub fn lighten(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::max)
}

pub fn darken(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::min)
}

pub fn difference(a: &Coverage, b: &Coverage) -> Result<Coverage, CoverageSizeMismatch> {
    zip_pixels(a, b, u8::abs_diff)
}
