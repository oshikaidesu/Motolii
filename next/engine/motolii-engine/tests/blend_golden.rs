//! BL3(分離可能 blend 11 モード)の golden 回帰(ORACLE「golden: 各モード最低1枚
//! (α/glow の golden 流儀 — max=0 安定を確認)」)。
//!
//! `glow_golden.rs` と同じ規律(固定 fixture の `Engine::render_frame` 出力を
//! 既存 image golden 慣習で照合、無ければ初回に作り、意図した変更なら消して作り直す
//! ——`motolii_testkit::assert_rgba_matches_golden_file` 参照)。分離可能 blend は
//! 8bit sRGB round-trip 以外の非決定要素(blur・時間依存乱数等)を一切持たないので、
//! 許容差は `tol::GPU_RASTER` ではなく**`tol::EXACT`(max=0)**——同じ入力から
//! 同じフレームが出ることをバイト単位で固定する(数値の正しさ自体は
//! `tests/blend_separable.rs` の独立 W3C 式照合が担う、ここは「安定性」専任)。

use motolii_engine::Engine;
use motolii_store::{
    BlendMode, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta,
    LayerSource, LayerTiming, RationalTime,
};
use motolii_testkit::{assert_rgba_matches_golden_file, tol, RgbaImageDesc};

const W: u32 = 32;
const H: u32 = 32;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn place_solid_gray(doc: &mut Document, layer: LayerId, gray: u8, order: i16) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [gray, gray, gray, 255],
                width: W,
                height: H,
            },
            order,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

fn doc_with_mode(mode: BlendMode) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();

    let (base, top) = (LayerId(1), LayerId(2));
    // base/top とも中庸値(0/255 の飽和を避け、モード同士の絵の違いが golden 上で
    // 見分けられる値、`tests/blend_separable.rs` の数値検証と同じ組)。
    place_solid_gray(&mut doc, base, 153, 0);
    place_solid_gray(&mut doc, top, 77, 1);
    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            blend_mode: Some(mode),
            ..Default::default()
        },
    })
    .unwrap();
    doc
}

fn golden_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn assert_mode_golden(mode: BlendMode, file_stem: &str) {
    let doc = doc_with_mode(mode);
    let mut engine = Engine::new().expect("engine");
    let frame = engine
        .render_frame(&doc.view(), t(0))
        .unwrap_or_else(|e| panic!("{mode:?} が engine に拒まれた: {e}"));

    assert_rgba_matches_golden_file(
        golden_path(&format!("blend_{file_stem}.png")),
        &format!("blend_{file_stem}"),
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &frame,
        tol::EXACT,
    );
}

#[test]
fn multiply_matches_golden() {
    assert_mode_golden(BlendMode::Multiply, "multiply");
}

#[test]
fn screen_matches_golden() {
    assert_mode_golden(BlendMode::Screen, "screen");
}

#[test]
fn overlay_matches_golden() {
    assert_mode_golden(BlendMode::Overlay, "overlay");
}

#[test]
fn darken_matches_golden() {
    assert_mode_golden(BlendMode::Darken, "darken");
}

#[test]
fn lighten_matches_golden() {
    assert_mode_golden(BlendMode::Lighten, "lighten");
}

#[test]
fn color_dodge_matches_golden() {
    assert_mode_golden(BlendMode::ColorDodge, "color_dodge");
}

#[test]
fn color_burn_matches_golden() {
    assert_mode_golden(BlendMode::ColorBurn, "color_burn");
}

#[test]
fn hard_light_matches_golden() {
    assert_mode_golden(BlendMode::HardLight, "hard_light");
}

#[test]
fn soft_light_matches_golden() {
    assert_mode_golden(BlendMode::SoftLight, "soft_light");
}

#[test]
fn difference_matches_golden() {
    assert_mode_golden(BlendMode::Difference, "difference");
}

#[test]
fn exclusion_matches_golden() {
    assert_mode_golden(BlendMode::Exclusion, "exclusion");
}
