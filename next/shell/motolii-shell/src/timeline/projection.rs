//! Timeline の投影の純関数(`rows`/`frame_to_x`/`frame_at_x`/
//! `time_band_segment_frames`)。**読むだけ** — Document/Session を書き換えない。

use motolii_store::{Fps, LayerId, StoreView};

use crate::Session;

/// 1層分の読み取り投影。**Document の写しではなく、1度描くための使い捨て値**。
#[derive(Clone, Debug, PartialEq)]
pub struct RowProjection {
    pub id: LayerId,
    pub name: String,
    pub hidden: bool,
    /// solo(`LayerAttrs.solo`)。レーンバーの S トグルが読む(裁定147)。
    pub solo: bool,
    /// locked(`LayerAttrs.locked`)。レーンバーの L トグルが読む(裁定147)。
    pub locked: bool,
    pub start: i64,
    pub duration: i64,
    pub selected: bool,
}

/// `store`/`session` から Timeline の行を組み立てる。**読むだけ**。
///
/// `store.layers()` は「present な layer」しか返さない(削除は墓標なので既に除外
/// 済み — `view.rs`)。ここでは並び順を `LayerId` 昇順のまま使う。bar の重ね順
/// (`meta.order`)は Stage 側の合成順であって、Timeline の縦位置の所有者にしない
/// (`ui-score-model.md` 4層構成: 縦位置は packing 結果にすぎない)。
pub fn rows(store: &StoreView<'_>, session: &Session) -> Vec<RowProjection> {
    let mut out = Vec::new();
    for id in store.layers() {
        let Ok(Some(meta)) = store.meta(id) else {
            continue;
        };
        let attrs = store.attrs(id).ok().flatten().unwrap_or_default();
        out.push(RowProjection {
            id,
            name: attrs.name,
            hidden: attrs.hidden,
            solo: attrs.solo,
            locked: attrs.locked,
            start: meta.timing.start,
            duration: meta.timing.duration,
            selected: session.selection == Some(id),
        });
    }
    out
}

/// comp フレーム → x px。`duration_frames <= 0` の空 comp では常に 0。
///
/// `pub(crate)`: screenshot 器具(`crate::screenshot`)が Timeline canvas と同じ
/// x 座標計算を使うため(マーカー・bar の位置を2箇所で別の式にしない)。
pub(crate) fn frame_to_x(frame: i64, width: f32, duration_frames: i64) -> f32 {
    if duration_frames <= 0 || width <= 0.0 {
        return 0.0;
    }
    let ratio = frame as f32 / duration_frames as f32;
    (ratio * width).clamp(0.0, width)
}

/// x px → comp フレーム。**scrub の core**(canvas の click/drag と単体 test の両方が
/// これを呼ぶ)。範囲外は端へ丸める。
pub fn frame_at_x(x: f32, width: f32, duration_frames: i64) -> i64 {
    if duration_frames <= 0 || width <= 0.0 {
        return 0;
    }
    let ratio = (x / width).clamp(0.0, 1.0);
    let frame = (ratio * duration_frames as f32).round() as i64;
    frame.clamp(0, (duration_frames - 1).max(0))
}

/// ルーラー目盛りの分割数。fps が引けない(comp 無し)時の時間方向リズム
/// ([`time_band_segment_frames`]、`super::canvas::draw_ruler_ticks` の実描画)の
/// フォールバックも同じ分割を使う — 「ルーラーと違う区間の刻み方」という
/// 新しい規則を増やさない。
/// `pub(crate)`: `super::canvas`(同じ `draw_ruler_ticks`)と `screenshot.rs`
/// 器具が同じ区間の刻み方を再現するのにも使う(`frame_to_x` と同じ理由)。
pub(crate) const RULER_TICK_DIVISIONS: i64 = 8;

/// 時間方向の明暗リズム(裁定148(1))の区間幅(フレーム数)。fps が引ければ
/// 1秒、引けなければ [`RULER_TICK_DIVISIONS`] 等分へ落ちる。`draw_time_bands`
/// と screenshot 器具の両方がこの1つの式から区間境界を出す(2箇所で別の
/// フォールバックを持たない)。
///
/// `pub(crate)`: `crate::screenshot` が Timeline canvas と同じ区間の刻み方を
/// 再現するため(`frame_to_x` と同じ理由)。
pub(crate) fn time_band_segment_frames(fps: Option<Fps>, duration_frames: i64) -> i64 {
    fps.map(|fps| fps.as_f64().round().max(1.0) as i64)
        .unwrap_or_else(|| (duration_frames / RULER_TICK_DIVISIONS).max(1))
}
