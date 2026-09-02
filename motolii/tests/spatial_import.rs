use motolii::doc::store::{
    property, Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    PropertyId, RationalTime, Value,
};
use motolii::render::engine::Engine;

fn write_obj(path: &std::path::Path) {
    std::fs::write(
        path,
        "v -1 -1 0\nv 1 -1 0\nv -0.4 1 0\nf 1 2 3\n",
    )
    .unwrap();
}

fn write_ply(path: &std::path::Path) {
    std::fs::write(
        path,
        "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n-1 -1 0 255 0 0\n1 -1 0 0 255 0\n-0.4 1 0 0 0 255\n",
    )
    .unwrap();
}

fn write_stl(path: &std::path::Path) {
    std::fs::write(
        path,
        "solid triangle\nfacet normal 0 0 1\nouter loop\nvertex -1 -1 0\nvertex 1 -1 0\nvertex -0.4 1 0\nendloop\nendfacet\nendsolid triangle\n",
    )
    .unwrap();
}

fn write_glb(path: &std::path::Path) {
    let mut binary = Vec::new();
    for value in [-1.0f32, -1.0, 0.0, 1.0, -1.0, 0.0, -0.4, 1.0, 0.0] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u16, 1, 2] {
        binary.extend_from_slice(&index.to_le_bytes());
    }
    while binary.len() % 4 != 0 {
        binary.push(0);
    }
    let mut json = br#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}],"buffers":[{"byteLength":44}],"bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":36,"target":34962},{"buffer":0,"byteOffset":36,"byteLength":6,"target":34963}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-1,-1,0],"max":[1,1,0]},{"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}]}"#
        .to_vec();
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    let total_len = 12 + 8 + json.len() + 8 + binary.len();
    let mut glb = Vec::with_capacity(total_len);
    glb.extend_from_slice(&0x4654_6c67u32.to_le_bytes());
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());
    glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4e4f_534au32.to_le_bytes());
    glb.extend_from_slice(&json);
    glb.extend_from_slice(&(binary.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x004e_4942u32.to_le_bytes());
    glb.extend_from_slice(&binary);
    std::fs::write(path, glb).unwrap();
}

fn document(path: &std::path::Path, rotation_x: f64, opacity: f64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 128,
        height: 128,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 30,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: path.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 30),
            },
        },
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::POSITION).unwrap(),
            value: Value::Vec2([32.0, 32.0]),
        },
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::SCALE).unwrap(),
            value: Value::Vec2([32.0, 32.0]),
        },
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::ROTATION_X).unwrap(),
            value: Value::F64(rotation_x),
        },
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::OPACITY).unwrap(),
            value: Value::F64(opacity),
        },
    ])
    .unwrap();
    doc
}

fn render(path: &std::path::Path, rotation_x: f64, opacity: f64) -> (Vec<u8>, [f32; 2]) {
    let doc = document(path, rotation_x, opacity);
    let mut engine = Engine::new().unwrap();
    let pixels = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    let size = engine
        .selected_layer_size(&doc.view(), LayerId(1), RationalTime::ZERO)
        .expect("spatial layer has a selection frame");
    (pixels, size)
}

#[test]
fn rerun_asset3d_formats_and_ply_share_the_spatial_layer_contract() {
    let dir = tempfile::tempdir().unwrap();
    let obj = dir.path().join("triangle.obj");
    let glb = dir.path().join("triangle.glb");
    let stl = dir.path().join("triangle.stl");
    let ply = dir.path().join("triangle.ply");
    write_obj(&obj);
    write_glb(&glb);
    write_stl(&stl);
    write_ply(&ply);

    for path in [&obj, &glb, &stl, &ply] {
        let (flat, size) = render(path, 0.0, 1.0);
        let (tilted, tilted_size) = render(path, 45.0, 1.0);
        let (dimmed, _) = render(path, 0.0, 0.35);
        assert!(
            flat.chunks_exact(4).any(|pixel| pixel[..3] != [0, 0, 0]),
            "{} produced only background",
            path.display()
        );
        assert_ne!(flat, tilted, "Rotation X did not affect {}", path.display());
        let light = |pixels: &[u8]| {
            pixels
                .chunks_exact(4)
                .map(|pixel| u64::from(pixel[0]) + u64::from(pixel[1]) + u64::from(pixel[2]))
                .sum::<u64>()
        };
        assert!(
            light(&dimmed) < light(&flat),
            "Opacity did not dim {}",
            path.display()
        );
        assert_eq!(size, tilted_size);
        assert!(size[0] > 1.0 && size[0] <= 128.0, "bad width {size:?}");
        assert!(size[1] > 1.0 && size[1] <= 128.0, "bad height {size:?}");
    }
}

#[test]
fn spatial_project_reopens_with_the_same_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let ply = dir.path().join("cloud.ply");
    write_ply(&ply);
    let doc = document(&ply, 35.0, 0.35);
    let before = Engine::new()
        .unwrap()
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let project = dir.path().join("spatial.rrd");
    doc.save(&project).unwrap();
    let loaded = Document::load(&project).unwrap();
    let after = Engine::new()
        .unwrap()
        .render_frame(&loaded.view(), RationalTime::ZERO)
        .unwrap();
    assert_eq!(before, after);
}
