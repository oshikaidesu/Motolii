//! `Engine::render_frame_with_view_camera` の通し試験(裁定157、S0)。
//!
//! 縫い目調査(`docs/reviews/2026-08-21-camera-seam-survey.md`)の推しどおり、観測視点は
//! `render_frame` と**同じ合成器・同じ層構築**を使う第二エントリとして実装されている
//! (裁定141の `render_frame_without_background` と同型)。ここは:
//! (a) 観測視点のズームが実際に絵を変える(拡大して見える) — 第二エントリが本当に
//!     独立した camera を運んでいることの確認
//! (b) 既定の観測視点(パン0・ズーム1)の絵の性質 — Document のカメラも既定(未編集)の
//!     ときは `render_frame` と**バイト一致**する(観測視点の既定値がレンダリングカメラの
//!     既定値と数学的に同じ `ResolvedCamera::default()` へ写るため)
//! (c) 既存 `render_frame`/`render_frame_without_background` は無改変 — このテストは
//!     それらのファイルを一切触らず、`tests/camera.rs` 等の既存試験と並んで緑になる。

use motolii_engine::{Engine, ObservationCamera};
use motolii_store::{
    property, Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerId, LayerMeta,
    LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

const W: u32 = 64;
const H: u32 = 64;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
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

/// comp 中心(32,32)に 24x24 の赤 solid を1枚だけ置いた Document。
///
/// `LayerPlacement` の既定 `transform` は `Affine2::IDENTITY`(anchor/position とも
/// 未編集=0、裁定20)なので、板のローカル矩形 `(0,0)-(size)`(`motolii-compositor` の
/// `Layer::size` doc 参照)はそのまま comp 座標 [0,24)x[0,24) — 左上に居る。
/// **comp 中心へ揃えるため `POSITION` track を明示的に [20,20] へ置く**
/// (anchor は既定 [0,0] のままなので、position がそのまま局所矩形の原点の
/// 着地点になる) — 結果、板は x,y とも [20,44) を占める
/// (`motolii-core::camera` の「zoom は comp 中心を軸に等方拡大」試験と同じ幾何)。
fn doc_with_centered_red_square() -> Document {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [255, 0, 0, 255],
                width: 24,
                height: 24,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let mut position = KeyframeTrack::new();
    position.insert(Keyframe {
        t: t(0),
        value: Value::Vec2([20.0, 20.0]),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::POSITION).unwrap(),
        track: position,
    })
    .unwrap();

    doc
}

/// (a) ズーム2の観測視点は、レンダリングカメラ(既定、パン0・ズーム1)とは
/// 異なる絵——しかも「拡大」した絵を返す。24x24 の板は zoom=1 で x,y とも
/// [20,44) を占め、zoom=2 では comp 中心(32,32)を軸に等方2倍(裁定113/116)で
/// [8,56) まで広がる。pixel(14,14) は zoom=1 の板の外(背景・黒)だが、
/// zoom=2 の板の内側(赤)に入る——これが「異なる絵」の中身を具体的に固定する。
#[test]
fn a_zoom_two_view_camera_renders_a_magnified_image_that_differs_from_the_render_camera() {
    let doc = doc_with_centered_red_square();
    let mut engine = Engine::new().expect("engine");

    let render_camera_frame = engine.render_frame(&doc.view(), t(0)).unwrap();
    let view_camera_frame = engine
        .render_frame_with_view_camera(
            &doc.view(),
            t(0),
            &ObservationCamera {
                pan: [0.0, 0.0],
                zoom: 2.0,
            },
        )
        .unwrap();

    assert_ne!(
        render_camera_frame, view_camera_frame,
        "ズーム2の観測視点がレンダリングカメラと同じ絵を返した — 第二エントリが\
         独立した camera を運んでいない"
    );

    // pixel(14,14): zoom=1(レンダリングカメラ既定)では板の外(黒背景)。
    let outside_at_zoom_one = pixel(&render_camera_frame, 14, 14);
    assert_eq!(
        outside_at_zoom_one,
        [0, 0, 0, 255],
        "既定レンダリングカメラで pixel(14,14) は板の外(黒背景)のはず: {outside_at_zoom_one:?}"
    );

    // 同じ pixel(14,14) が zoom=2 の観測視点では板の内側(赤)に入る = 拡大している。
    let inside_at_zoom_two = pixel(&view_camera_frame, 14, 14);
    assert_eq!(
        inside_at_zoom_two[0], 255,
        "ズーム2の観測視点で pixel(14,14) が赤くなっていない(拡大していない): {inside_at_zoom_two:?}"
    );
}

/// (b) Document のカメラが未編集(既定)のとき、既定の観測視点(パン0・ズーム1)は
/// `render_frame` と**バイト一致**する——`ObservationCamera::default()` が
/// `ResolvedCamera::default()` と同じ値へ写るため、数学的に同一の camera で描く
/// ことになる。この等価性をここで固定する(裁定157 EXACT TARGET #3 の実測結果)。
#[test]
fn the_default_view_camera_matches_the_render_camera_byte_for_byte_when_the_document_camera_is_untouched(
) {
    let doc = doc_with_centered_red_square();
    let mut engine = Engine::new().expect("engine");

    let render_camera_frame = engine.render_frame(&doc.view(), t(0)).unwrap();
    let default_view_camera_frame = engine
        .render_frame_with_view_camera(&doc.view(), t(0), &ObservationCamera::default())
        .unwrap();

    assert_eq!(
        render_camera_frame, default_view_camera_frame,
        "既定の観測視点が既定のレンダリングカメラとバイト一致しない"
    );
}
