//! probe が使うレイヤー素材と、それを `SpatialStage` へ置くための最小の道具。
//!
//! レイヤーは全て `SpatialStage::copy_gpu_image` で渡す。2026-08-17の実測で
//! 「画素ごとの alpha が通るのはこの経路だけ」と決まっているため。

use std::sync::Arc;

use re_chunk::{Chunk, RowId};
use re_log_types::TimePoint;
use re_sdk_types::archetypes::{GridMap, Transform3D};
use re_view_spatial::SpatialStage;

pub const LAYER_W: u32 = 320;
pub const LAYER_H: u32 = 240;
/// `copy_gpu_image` は cell_size=1/height で置くので、矩形は縦1.0・横 aspect になる。
pub const LAYER_ASPECT: f32 = LAYER_W as f32 / LAYER_H as f32;

/// 4象限を別々の色で塗った不透明テクスチャ。
/// (b) で「どの角がどこへ写るか」まで見たいので、対称にしない。
///
/// 画像座標の左上/右上/左下/右下 = 赤/緑/青/黄。
pub fn quadrants() -> Vec<u8> {
    let mut rgba = vec![0u8; (LAYER_W * LAYER_H * 4) as usize];
    for y in 0..LAYER_H {
        for x in 0..LAYER_W {
            let index = ((y * LAYER_W + x) * 4) as usize;
            let left = x < LAYER_W / 2;
            let top = y < LAYER_H / 2;
            let color = match (top, left) {
                (true, true) => [230, 30, 30],
                (true, false) => [30, 230, 30],
                (false, true) => [30, 30, 230],
                (false, false) => [230, 230, 30],
            };
            rgba[index] = color[0];
            rgba[index + 1] = color[1];
            rgba[index + 2] = color[2];
            rgba[index + 3] = 255;
        }
    }
    rgba
}

/// 全面不透明の単色。(c) の「奥のレイヤー」。
pub fn solid(color: [u8; 3]) -> Vec<u8> {
    let mut rgba = vec![0u8; (LAYER_W * LAYER_H * 4) as usize];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[0] = color[0];
        pixel[1] = color[1];
        pixel[2] = color[2];
        pixel[3] = 255;
    }
    rgba
}

/// 中央だけ不透明な円、外側は alpha=0。(c) の「手前のレイヤー」。
///
/// premultiplied なので alpha=0 の画素は RGB も 0 にしておく。中間の alpha を作らないのは、
/// 「遮蔽できたか」と「半透明で混ざったか」を色で取り違えないため。
pub fn center_disc(color: [u8; 3], radius_fraction: f32) -> Vec<u8> {
    let mut rgba = vec![0u8; (LAYER_W * LAYER_H * 4) as usize];
    let center_x = LAYER_W as f32 * 0.5;
    let center_y = LAYER_H as f32 * 0.5;
    let radius = LAYER_H as f32 * radius_fraction;
    for y in 0..LAYER_H {
        for x in 0..LAYER_W {
            let index = ((y * LAYER_W + x) * 4) as usize;
            let distance =
                ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
            if distance <= radius {
                rgba[index] = color[0];
                rgba[index + 1] = color[1];
                rgba[index + 2] = color[2];
                rgba[index + 3] = 255;
            }
        }
    }
    rgba
}

/// premultiplied RGBA を GPU へ置く。`copy_gpu_image` が要求する形式に合わせる。
pub fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    rgba: &[u8],
) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: LAYER_W,
        height: LAYER_H,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(LAYER_W * 4),
            rows_per_image: Some(LAYER_H),
        },
        size,
    );
    queue.submit([]);
    texture
}

/// 親 entity に平行移動を置く。`copy_gpu_image` は自分自身の path へ
/// 「原点中心・z=-0.01」を書くので、レイヤーの z は親側で決める。
pub fn place_parent(stage: &mut SpatialStage, path: &str, z: f32) -> Result<(), String> {
    ingest(
        stage,
        Chunk::builder(path)
            .with_archetype(
                RowId::new(),
                TimePoint::STATIC,
                &Transform3D::from_translation([0.0, 0.0, z]),
            )
            .build(),
    )
}

/// 同一平面のときだけ効く前後キー。z が違う場合は距離ソートが優先する。
pub fn set_draw_order(stage: &mut SpatialStage, path: &str, order: f32) -> Result<(), String> {
    ingest(
        stage,
        Chunk::builder(path)
            .with_archetype(
                RowId::new(),
                TimePoint::STATIC,
                &GridMap::update_fields().with_draw_order(order),
            )
            .build(),
    )
}

/// document camera 相当として `Pinhole` を1本立てる。
/// これを Rerun のカメラ機構へ「外注」できるかが (b) の本題である。
pub fn place_pinhole_camera(
    stage: &mut SpatialStage,
    path: &str,
    translation: [f32; 3],
) -> Result<(), String> {
    let pinhole = re_sdk_types::archetypes::Pinhole::from_focal_length_and_resolution(
        [LAYER_H as f32, LAYER_H as f32],
        [LAYER_W as f32, LAYER_H as f32],
    );
    ingest(
        stage,
        Chunk::builder(path)
            .with_archetype(RowId::new(), TimePoint::STATIC, &pinhole)
            .with_archetype(
                RowId::new(),
                TimePoint::STATIC,
                &Transform3D::from_translation(translation),
            )
            .build(),
    )
}

fn ingest(
    stage: &mut SpatialStage,
    chunk: Result<Chunk, re_chunk::ChunkError>,
) -> Result<(), String> {
    let chunk = chunk.map_err(|error| format!("build chunk: {error}"))?;
    stage
        .ingest_chunk(Arc::new(chunk))
        .map_err(|error| format!("ingest chunk: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disc_gutter_is_fully_transparent() {
        let pixels = center_disc([30, 230, 30], 0.30);
        assert_eq!(&pixels[0..4], &[0, 0, 0, 0]);
    }

    #[test]
    fn disc_core_is_opaque() {
        let pixels = center_disc([30, 230, 30], 0.30);
        let center = (((LAYER_H / 2) * LAYER_W + LAYER_W / 2) * 4) as usize;
        assert_eq!(&pixels[center..center + 4], &[30, 230, 30, 255]);
    }

    #[test]
    fn quadrants_are_four_distinct_colors() {
        let pixels = quadrants();
        let at = |x: u32, y: u32| {
            let index = ((y * LAYER_W + x) * 4) as usize;
            [pixels[index], pixels[index + 1], pixels[index + 2]]
        };
        assert_eq!(at(10, 10), [230, 30, 30]);
        assert_eq!(at(LAYER_W - 10, 10), [30, 230, 30]);
        assert_eq!(at(10, LAYER_H - 10), [30, 30, 230]);
        assert_eq!(at(LAYER_W - 10, LAYER_H - 10), [230, 230, 30]);
    }
}
