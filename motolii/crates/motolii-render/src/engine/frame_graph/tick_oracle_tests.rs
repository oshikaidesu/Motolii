//! The tick against the old path, the oracle: the same document, the same views, the same pixels.
//! Each ported feature adds its document here.

use super::tick::ViewRequest;
use crate::doc::core::{RationalTime, ResolvedCamera};
use crate::doc::store::{property, LayerId, PropertyId, Value};
use crate::frame_graph::ViewProjection;
use crate::render::compositor::{Window, PRESENTABLE_FORMAT};
use crate::render::engine::environment_tests::{file_layer, SIZE};
use crate::render::engine::Engine;
use motolii_edit::{Document, Intent};

fn target(engine: &Engine, window: Window) -> wgpu::Texture {
    engine.compositor.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("tick-oracle-view"),
        size: wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

struct Shown { window: Window, projection: ViewProjection, camera: Option<ResolvedCamera> }

fn views() -> Vec<Shown> {
    let comp = SIZE as f32;
    vec![
        Shown { window: Window { width: SIZE, height: SIZE, roi: [0.0, 0.0, comp, comp], projection_camera: None }, projection: ViewProjection::Camera, camera: None },
        Shown { window: Window { width: SIZE * 2, height: SIZE, roi: [-comp / 2.0, 0.0, comp * 2.0, comp], projection_camera: Some(Default::default()) }, projection: ViewProjection::Stage, camera: Some(Default::default()) },
    ]
}

/// Every view drawn by the old path and by one tick; the pictures must match.
pub(super) fn assert_matches_oracle(doc: &Document, what: &str) {
    let time = RationalTime::ZERO;
    let mut old = Engine::new().unwrap();
    let mut expected = Vec::new();
    for shown in views() {
        let camera = shown.camera.unwrap_or_else(|| old.frame_graph_document_camera(&doc.view(), time).unwrap());
        let texture = target(&old, shown.window);
        old.render_frame_graph_into_window(&doc.view(), time, &texture, camera, true, &[], shown.window, shown.projection).unwrap();
        expected.push(old.compositor.read_texture_bytes(&texture).unwrap());
    }
    assert!(old.layer_failures().is_empty(), "{:?}", old.layer_failures());

    let mut new = Engine::new().unwrap();
    let shown = views();
    let targets: Vec<_> = shown.iter().map(|s| target(&new, s.window)).collect();
    let document_camera = new.frame_graph_document_camera(&doc.view(), time).unwrap();
    let requests: Vec<_> = shown.iter().zip(&targets).map(|(s, target)| ViewRequest {
        target, window: s.window, camera: s.camera.unwrap_or(document_camera), projection: s.projection, include_background: true,
    }).collect();
    new.tick(&doc.view(), time, &requests).unwrap();
    for ((s, texture), expected) in shown.iter().zip(&targets).zip(&expected) {
        let actual = new.compositor.read_texture_bytes(texture).unwrap();
        let worst = actual.iter().zip(expected).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0);
        assert!(worst <= 1, "{what}, {:?} view: differs from the old path by up to {worst}", s.projection);
    }
}

fn png(dir: &std::path::Path, name: &str, rgba: [u8; 4]) -> std::path::PathBuf {
    let path = dir.join(name);
    image::RgbaImage::from_pixel(16, 16, image::Rgba(rgba)).save(&path).unwrap();
    path
}

fn comp(doc: &mut Document) {
    doc.apply(Intent::SetComposition(crate::doc::store::Composition {
        width: SIZE, height: SIZE, fps: crate::doc::store::Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.1, 0.2, 0.3, 1.0],
    })).unwrap();
}

/// Two overlapping pictures: one scaled, one turned and see-through.
pub(super) fn pictures(dir: &std::path::Path) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    comp(&mut doc);
    let red = file_layer(&mut doc, 1, 0, &png(dir, "red.png", [255, 0, 0, 255]));
    let green = file_layer(&mut doc, 2, 1, &png(dir, "green.png", [0, 255, 0, 160]));
    let set = |doc: &mut Document, layer: LayerId, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    set(&mut doc, red, property::SCALE, Value::Vec2([2.0, 2.0]));
    set(&mut doc, green, property::POSITION, Value::Vec2([40.0, 36.0]));
    set(&mut doc, green, property::ROTATION, Value::F64(30.0));
    set(&mut doc, green, property::OPACITY, Value::F64(70.0));
    doc
}

#[test]
fn overlapping_pictures_match_the_old_path() {
    let dir = tempfile::tempdir().unwrap();
    assert_matches_oracle(&pictures(dir.path()), "two overlapping pictures");
}
