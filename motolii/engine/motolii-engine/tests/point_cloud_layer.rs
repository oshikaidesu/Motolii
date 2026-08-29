//! **検収条件**(発注「点群 vertical slice」): `.ply` → `LayerSource::PointCloud` →
//! `Engine::render_frame` が Stage の画素として実際に点群を描くこと。
//! `zero_copy_shape.rs::rectangle_shape_zero_copy_matches_cpu_export_and_renders_visible_pixels`
//! と同じ oracle の形(色つきピクセルの実測、パイプラインの終端で実測する)を
//! 点群へ適用する——手で作った `PointCloudData` を engine へ差し込むのではなく、
//! 実 `.ply` ファイルを書いて `motolii_media::load_point_cloud` の入口から通す。

use motolii_engine::Engine;
use motolii_store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, RationalTime,
};

const W: u32 = 256;
const H: u32 = 256;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

/// 実 `.ply`(ASCII)を一時ファイルへ書く——`ply-rs-bw`(Rerun の `.ply` パーサ、
/// `re_sdk_types::archetypes::Points3D::from_file_contents` 経由)が実際に読む
/// バイト列をこの試験でも通す。点は小さな球面上に散らし、色は白 —
/// 「点群として認識される最小の実データ」を作るだけで、見た目の作品性は問わない。
fn write_fixture_ply() -> std::path::PathBuf {
    let mut body = String::new();
    let n = 64;
    for i in 0..n {
        let theta = std::f32::consts::PI * (i as f32 + 0.5) / n as f32;
        let phi = 2.399963 * i as f32; // golden angle spiral、単純な一様分布で十分
        let r = 20.0_f32;
        let x = r * theta.sin() * phi.cos();
        let y = r * theta.sin() * phi.sin();
        let z = r * theta.cos();
        body.push_str(&format!("{x} {y} {z} 255 255 255\n"));
    }
    let content = format!(
        "ply\n\
format ascii 1.0\n\
element vertex {n}\n\
property float x\n\
property float y\n\
property float z\n\
property uchar red\n\
property uchar green\n\
property uchar blue\n\
end_header\n\
{body}"
    );
    let path = std::env::temp_dir().join(format!(
        "motolii-engine-test-point-cloud-{}.ply",
        std::process::id()
    ));
    std::fs::write(&path, content).expect("write fixture ply");
    path
}

fn place_point_cloud_layer(doc: &mut Document, layer: LayerId, path: &std::path::Path) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::PointCloud {
                path: path.to_string_lossy().into_owned(),
                fingerprint: None,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

#[test]
fn ply_point_cloud_renders_visible_pixels_through_render_frame() {
    let ply_path = write_fixture_ply();
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_point_cloud_layer(&mut doc, layer, &ply_path);

    let mut engine = Engine::new().expect("headless engine");
    let frame = engine
        .render_frame(&doc.view(), t(0))
        .expect("render_frame が点群 layer を描けるはず");

    assert_eq!(frame.len(), (W * H * 4) as usize);

    // comp background は不透明黒(`Composition::default_background` と同じ)——
    // 点群が実際に描かれていれば、黒(R=G=B=0)ではない画素が生じるはず
    // (白い点を白背景で塗っているので、非黒 = 点群由来と読める)。
    let non_background = frame
        .chunks_exact(4)
        .filter(|p| !(p[0] == 0 && p[1] == 0 && p[2] == 0))
        .count();
    assert!(
        non_background > 0,
        "点群の画素が1つも出ていない(re_importer 経由の parse → \
         PointCloudData → Compositor::render_point_cloud_to_texture → 合成、\
         のどこかで途切れている疑い)"
    );

    let failures = engine.layer_failures();
    assert!(
        failures.is_empty(),
        "点群 layer が失敗を報告している(A05隔離経路): {failures:?}"
    );

    std::fs::remove_file(&ply_path).ok();
}

/// 存在しないパスは他の layer を巻き込まず、この layer だけ「描く物が無い」に
/// 落ちること(`LayerSource::Media` の A05 隔離と同じ規律、`point_cloud_texture_for`
/// の doc 参照)。
#[test]
fn missing_ply_isolates_to_this_layer_without_erroring_the_whole_frame() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_point_cloud_layer(
        &mut doc,
        layer,
        std::path::Path::new("/nonexistent/does-not-exist.ply"),
    );

    let mut engine = Engine::new().expect("headless engine");
    let frame = engine
        .render_frame(&doc.view(), t(0))
        .expect("存在しないファイルでも comp 全体は Err にならないはず(A05隔離)");
    assert_eq!(frame.len(), (W * H * 4) as usize);

    assert!(
        !engine.layer_failures().is_empty(),
        "読めない点群は理由つきで layer_failures へ積まれるはず(Q3: 黙って握りつぶさない)"
    );
}
