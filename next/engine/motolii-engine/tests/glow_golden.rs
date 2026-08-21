//! 裁定153 S5: `"motolii.glow"`(S4 で main 着地済み)を golden PNG で回帰固定する。
//!
//! 固定 fixture(単色矩形 layer 1枚 + glow param 固定)の `Engine::render_frame`
//! 出力を、リポの既存 image golden 慣習で照合する:
//! - 許容差は旧 `crates/motolii-testkit::tol::GPU_RASTER`(max=1, mean<=0.5)と
//!   同じ定数を next/ 側へ移した [`motolii_testkit::tol`](発明しない、GPU
//!   ラスタライズ差の吸収)。
//! - golden の「無ければ初回に作り、意図した変更なら消して作り直す」規約は
//!   `crates/motolii-shell-iced/tests/snapshot_start_screen.rs`(`iced_test` の
//!   snapshot)がこのリポで既に持っている規約をそのまま借用
//!   ([`motolii_testkit::assert_rgba_matches_golden_file`] 参照)。
//!
//! param は2組: 「既定」(threshold=0.5・intensity=0.6、255 に飽和しない程度の
//! 控えめな明るみ)と「強め」(threshold=0.1・intensity=2.5、255 に飽和する
//! 明確な bloom)。
//!
//! **`Compositor::render_with_effects` は glow を layer 自身の texture 実寸
//! (`lwp.layer.texture.width_height()`)の上でだけ計算する**(`motolii-compositor/
//! src/lib.rs` の `render_with_effects` 実装参照)——comp 実寸へは広がらない。
//! つまり単色矩形 layer では bright-pass/blur の入力が全画素同値になり、
//! 「縁が滲んで外へ広がる halo」ではなく「layer 矩形が一様に明るくなる」絵に
//! なる(架空の広がりを golden の期待に書かない——実装の実際の挙動をそのまま
//! 固定する)。**layer は comp より小さく中央に置く**(`Intent::SetTrack` で
//! `position` を明示)ことで、layer 矩形の内と外(背景の黒)の境界がはっきり
//! 見える golden にする。intensity をそのまま 1.0 以上にすると layer 内が
//! 一様に 255 飽和し、「既定」「強め」の2枚が byte-for-byte 一致してしまう
//! (最初の実装がこの罠に落ちた)ので、「既定」は飽和しない intensity を選ぶ。

use motolii_engine::Engine;
use motolii_store::{
    property, Composition, Document, EffectId, EffectInstance, Fps, Intent, Interp, Keyframe,
    KeyframeTrack, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};
use motolii_testkit::{assert_rgba_matches_golden_file, tol, RgbaImageDesc};

const W: u32 = 64;
const H: u32 = 64;
/// comp(64x64)より小さい正方形(縁に halo の滲む余白を残す)。
const LAYER_SIZE: u32 = 24;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

/// 静止値の param track を1本立てる(`tests/effects.rs::set_static_param` と同じ形)。
fn set_static_param(doc: &mut Document, layer: LayerId, effect: EffectId, name: &str, value: f64) {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::F64(value),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::effect_param(effect, name).unwrap(),
        track,
    })
    .unwrap();
}

/// 単色矩形 layer 1枚(comp 中央・`LAYER_SIZE`四方)+ `"motolii.glow"` を固定 param
/// で積んだ Document。中間灰色(`tests/effects.rs` と同じ色)——閾値未満だと絵が
/// 変わらないので、bright-pass を確実に起動させられる値を呼び手が渡す。
fn doc_with_fixed_glow(threshold: f64, intensity: f64, radius: f64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();

    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [200, 200, 200, 255],
                width: LAYER_SIZE,
                height: LAYER_SIZE,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    // anchor 既定 [0,0](layer 左上)なので、position は「layer 左上が comp のどこへ
    // 着地するか」——comp 中央に置くには (comp_size - layer_size)/2 だけずらす
    // (`motolii-core::LayerPlacement::from_transform` の doc「position は anchor が
    // 着地する点」参照)。
    let offset = ((W - LAYER_SIZE) / 2) as f64;
    let mut position = KeyframeTrack::new();
    position.insert(Keyframe {
        t: t(0),
        value: Value::Vec2([offset, offset]),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::POSITION).expect("position は予約語ではない"),
        track: position,
    })
    .unwrap();

    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.glow".to_owned(),
            enabled: true,
        }],
    })
    .unwrap();
    set_static_param(&mut doc, layer, effect, "threshold", threshold);
    set_static_param(&mut doc, layer, effect, "intensity", intensity);
    set_static_param(&mut doc, layer, effect, "radius", radius);

    doc
}

fn golden_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

/// 既定強度: 255 飽和を避け、控えめな明るみ(中間灰色→ほんのり明るいグレー)に
/// 留める値。radius は proof の既定(1.0)。
#[test]
fn glow_default_strength_matches_golden() {
    let doc = doc_with_fixed_glow(0.5, 0.6, 1.0);
    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert_rgba_matches_golden_file(
        golden_path("glow_default.png"),
        "glow_default",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &frame,
        tol::GPU_RASTER,
    );
}

/// 強め: threshold を下げてより広い面を bright-pass に拾わせ、intensity/radius を
/// 上げて halo を明るく・広くする。
#[test]
fn glow_strong_matches_golden() {
    let doc = doc_with_fixed_glow(0.1, 2.5, 2.0);
    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert_rgba_matches_golden_file(
        golden_path("glow_strong.png"),
        "glow_strong",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &frame,
        tol::GPU_RASTER,
    );
}
