//! `Composition.background`(利用者要望: 黒だと気分が上がらない)が実際に
//! render_frame の絵へ乗ることを見る。
//!
//! **キーフレーム化しない**(静的値。動く背景は層でやる — 裁定20 の精神)ので、
//! ここでは時間経過ではなく色そのものが正しく出ることだけを縛る。

use motolii_engine::Engine;
use motolii_store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    RationalTime,
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

fn doc_with_background(background: [f32; 4]) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background,
    }))
    .unwrap();
    doc
}

/// 既定(不透明黒)は旧 clear 色と同じ見た目になる — これが崩れると、既存の
/// pixel 試験(frame.rs 等)が「なぜ緑のままなのか」の前提を失う。
#[test]
fn default_background_is_opaque_black() {
    let doc = doc_with_background(Composition::default_background());
    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert_eq!(
        pixel(&frame, 32, 32),
        [0, 0, 0, 255],
        "層が無い comp の既定背景は不透明黒のはず"
    );
}

/// 背景色を変えると、層に覆われていない場所の画素がその色になる。
#[test]
fn a_non_default_background_shows_through_where_no_layer_covers() {
    // 単純な整数比になる値を選ぶ(255 の丸め誤差を許容できる比較にする)。
    let doc = doc_with_background([1.0, 0.0, 0.0, 1.0]);
    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();

    let px = pixel(&frame, 32, 32);
    assert!(
        px[0] > 200 && px[1] < 50 && px[2] < 50,
        "background=赤 のはずが: {px:?}"
    );
}

/// 背景は**最も奥**に置かれる — comp 全域を覆う実 layer があれば背景色は一切見えない。
#[test]
fn background_sits_behind_every_real_layer() {
    let mut doc = doc_with_background([0.0, 1.0, 0.0, 1.0]);
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [0, 0, 255, 255],
                width: W,
                height: H,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();

    let px = pixel(&frame, 32, 32);
    assert!(
        px[2] > 200 && px[1] < 50,
        "comp 全域を覆う青 layer があるのに背景の緑が透けて見える: {px:?}"
    );
}

/// **readback アーティファクトの回帰試験**: 不透明背景の comp を合成した結果、
/// **外周含む全画素**の alpha が 255 になるはず。
///
/// 主担当が実測した症状(shell 側の `checkerboard_toggle_does_not_touch_interior_
/// pixels_when_background_is_opaque` の doc コメント参照): 640x360 comp の
/// `Shell::frame_rgba()` で外周ちょうど 1996 画素(= `2*(640+360)-4`、perimeter と
/// 厳密一致)が alpha=0 になる。ここでは同じ寸法・同じ「comp 全域を覆う不透明な
/// pinned 背景 layer」という条件を `Engine::render_frame` 単体で再現する。
#[test]
fn opaque_background_leaves_no_transparent_border_pixels() {
    const BW: u32 = 640;
    const BH: u32 = 360;

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: BW,
        height: BH,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert_eq!(frame.len(), (BW * BH * 4) as usize);

    let px = |buf: &[u8], x: u32, y: u32| -> [u8; 4] {
        let i = ((y * BW + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    };

    let mut transparent = Vec::new();
    for x in 0..BW {
        if px(&frame, x, 0)[3] != 255 {
            transparent.push((x, 0));
        }
        if px(&frame, x, BH - 1)[3] != 255 {
            transparent.push((x, BH - 1));
        }
    }
    for y in 1..BH - 1 {
        if px(&frame, 0, y)[3] != 255 {
            transparent.push((0, y));
        }
        if px(&frame, BW - 1, y)[3] != 255 {
            transparent.push((BW - 1, y));
        }
    }

    assert!(
        transparent.is_empty(),
        "不透明背景なのに外周に alpha!=255 の画素が {} 個ある(例: {:?})",
        transparent.len(),
        &transparent[..transparent.len().min(8)]
    );
}

/// undo で戻ると背景色も戻る — `SetComposition` は普通の編集(裁定40 の丸ごと置換)
/// なので、他の編集と同じく undo が効くはず。
#[test]
fn background_change_is_undoable() {
    let mut doc = doc_with_background([0.0, 0.0, 0.0, 1.0]);
    let mut engine = Engine::new().expect("engine");
    let black = engine.render_frame(&doc.view(), t(0)).unwrap();

    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 1.0, 1.0],
    }))
    .unwrap();
    let blue = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert_ne!(black, blue, "background を変えたのに絵が変わらない");

    assert!(doc.undo());
    let back_to_black = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert_eq!(black, back_to_black, "undo で背景色が戻らない");
}
