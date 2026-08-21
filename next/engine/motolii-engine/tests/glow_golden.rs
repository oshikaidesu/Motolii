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
//! param は2組: 「既定」(threshold=0.5・intensity=0.6・radius=1.0、255 に飽和
//! しない程度の控えめな明るみ)と「強め」(threshold=0.1・intensity=2.5・
//! radius=2.0、255 に飽和する明確な bloom)。
//!
//! ## 出力拡張(padding、既知の穴の根治、2026-08-21)
//!
//! 旧: `Compositor::render_with_effects` は glow を layer 自身の texture 実寸
//! の上でだけ計算していたため、単色矩形の glow は「縁が滲んで外へ広がる halo」
//! ではなく「layer 矩形が一様に明るくなる」絵になっていた(`next/reference/
//! KNOWN.md` の既知の穴)。
//!
//! 新: [`EffectPass::padding`](motolii_compositor::EffectPass::padding) が
//! pass ごとの出力拡張量(texel、`radius` 由来)を宣言し、`render_with_effects`
//! はその分だけ scratch を layer 実寸より広く確保して source を中央へ置いてから
//! blur を回す——**layer 矩形の外(背景の黒)へ実際に halo が滲み出す**ようになった
//! (`brighter_pixels_appear_outside_the_original_layer_rect_bounds` がこれを
//! 数値で縛る)。**layer は comp より小さく中央に置く**(`Intent::SetTrack` で
//! `position` を明示)ことで、layer 矩形の内と外の境界と、外側へ滲んだ halo が
//! はっきり見える golden にする。intensity をそのまま 1.0 以上にすると layer 内が
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

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
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

/// **落ちるテスト先行 → 既知の穴の根治**(`next/reference/KNOWN.md`「effect pass
/// は layer 自身のテクスチャ境界内のみで計算」): 単色矩形 + glow の
/// `render_frame` で、layer 矩形の**外側**(comp の黒背景)の画素に非ゼロ輝度の
/// halo が出る。padding(`EffectPass::padding`、`motolii-compositor`)を実装する
/// 前は、pass が layer 実寸の中でしか計算しないため、矩形の外側は常に
/// `[0,0,0,255]`(赤)だった。
#[test]
fn brighter_pixels_appear_outside_the_original_layer_rect_bounds() {
    let doc = doc_with_fixed_glow(0.1, 2.5, 2.0);
    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();

    // layer は `LAYER_SIZE`(24)四方、comp(64)中央 → x,y とも [20,44) を占める。
    // すぐ外側の行(y=19、layer 矩形の1画素上)は旧実装だと常に黒のままだった。
    let outside_top = pixel(&frame, W / 2, 19);
    assert!(
        outside_top[0] > 0 || outside_top[1] > 0 || outside_top[2] > 0,
        "layer 矩形の外側(1画素上)に halo が出ていない(padding 未実装の症状): {outside_top:?}"
    );

    // 矩形からさらに離れた画素(comp の隅寄り)は radius=2.0 が宣言する
    // padding(`step*2`=4)の届く範囲の外なので、依然として黒のはず——
    // 「画面全体が明るくなっただけ」ではなく、halo が矩形の縁からの距離に
    // 応じて減衰していることの対照点。
    let far_from_layer = pixel(&frame, 4, 4);
    assert_eq!(
        far_from_layer,
        [0, 0, 0, 255],
        "layer から十分離れた画素まで明るくなっている(halo が無限に広がっている・別の回帰): {far_from_layer:?}"
    );
}
