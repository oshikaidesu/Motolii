//! Stage 第4切片(発注 2026-08-22)— map bundle **B24(ズーム束)** + **B31(選択束)**
//! の残りのうち、Stage/キャンバスで意味を持つ行だけを純関数化する。
//!
//! ## なぜ選択の関数もこのファイルに同居するか
//!
//! 発注書 EXACT TARGET が新規ファイルを `src/zoom.rs` + `tests/zoom_fence.rs` の
//! 1本ずつに絞っている(gizmo/marquee/sheets/viewer_bar/lib 他部は読み専用)ため、
//! `select.rs` を別に起こさず B31 残りの4関数([`step_layer_selection`]・
//! [`select_all_layers`]・[`deselect_all_layers`]・[`invert_layer_selection`])を
//! ここへまとめてある。ファイル名とB31側の中身が一致しない点は承知の上の制約遵守
//! (発注書の逸脱欄に明記)。
//!
//! ## 対象行(`next/reference/normal-map.tsv`、freq降順、Stage/キャンバスで意味を
//! 持つ行だけを抽出 — Timeline専用ズーム・Audio波形ズーム・Viewerウィンドウ
//! モード切替・ツール状態切替は同じ束IDでも対象外)
//!
//! **B24(ズーム)**:
//! - 1441 `Zoom In`(freq3)/1442 `Zoom Out`(freq3) — 離散ズーム段の主根拠。
//!   1485/1487 `Zoom in/out in Composition… panel`(freq1)が同じ動詞の
//!   Composition パネル限定版として重なる。→ [`zoom_step`]。
//! - 1491/1492 `Zoom to fit`/`Zoom to Fit`(freq1×2)→ [`NamedZoomLevel::Fit`]。
//! - 1490 `Zoom to 100%`・1480 `Viewer Actual Size`(freq1×2、同義)→
//!   [`NamedZoomLevel::ActualSize`]。
//! - 976 `Zoom Around Mouse Pointer` — **新規実装ではない**。
//!   [`crate::zoom_at_screen_point`](既存の連続ホイールズーム)がこの行の実装
//!   そのもの、ここでは根拠行としてのみ記録する。
//!
//! **B31(選択)**:
//! - 290 `Select next layer in stacking order`/291 `Select previous layer in
//!   stacking order`(freq1×2)→ [`step_layer_selection`]。「stacking order」は
//!   `motolii-timeline-pane::projection` 冒頭コメントが踏襲している規約と同じ
//!   `StoreView::layers()` の `LayerId` 昇順をそのまま使う(新しい順序概念を
//!   増やさない)。
//! - 452 `Deselect all layers`・444 `Clear Selection`(freq1×2)→
//!   [`deselect_all_layers`]。433 `Deselect All`/436 `Select All` は**既に
//!   採用済**で `motolii-shell` 側(Edit メニュー、`clipboard::select_all`)へ
//!   結線済み — ここに置く [`select_all_layers`]/[`deselect_all_layers`] は
//!   Stage pane crate からも呼べる同型の純関数を用意するだけで、shell 側の
//!   実装を置き換えるものではない(一本化するかは結線波の判断、発注書の逸脱欄
//!   に明記)。
//!
//! ## map に無い行(発注書が直接指示した先例採用、逸脱として記録)
//!
//! - **50%/200%**・**選択にフィット**: normal-map.tsv に直接対応する行が無い
//!   (B24/B31 双方を検索済み)。発注書本文が名指しした4動詞のうち2つがこれ
//!   にあたる — AE/Figma の実機挙動を先例として直接採用する(下記 doc 参照)。
//! - **選択の反転**(invert selection): 同様に map に対応行が無い。AE/Figma
//!   共にレイヤー選択の反転は標準機能ではない([`invert_layer_selection`] の
//!   doc 参照)が、発注書が明示要求した動詞なのでそのまま実装する。
//!
//! ## ズーム段の刻み(先例、`ZOOM_LADDER`)
//!
//! **AE**: Composition/Layer/Footage パネル左下の倍率ポップアップは
//! 100%を基準にした2のべき倍のラダー(3.13/6.25/12.5/25/50/**100**/200/400/
//! 800/1600/3200/6400%)を刻む(実機既知動作 — 2026-08-22 時点で
//! `helpx.adobe.com/after-effects/using/modifying-using-views.html` への
//! WebFetch はタイムアウトしたため、URLでの一次資料固定はできていない
//! [EVIDENCE_GAP]。一般に広く確認されている挙動として記載)。
//! **Figma**: `Shift+1`=Zoom to fit・`Shift+2`=Zoom to selection・`Shift+0`=
//! Zoom to 100%(公式ヘルプ相当、2026-08-22 WebSearch で確認 —
//! <https://help.figma.com/hc/en-us/articles/360041065034-Adjust-your-zoom-and-view-options>
//! 系記事群)。この3値の型(Fit/選択にフィット/100%)がそのまま発注書の動詞と
//! 一致する。
//!
//! 既存の連続ズーム(`crate::ZOOM_STEP_FACTOR = 1.1` のホイール1ノッチ)とは
//! 別物 — こちらは「段」を明示的に飛ぶ離散動詞(ボタン/ショートカット向け)。
//! ラダーは [`crate::OBSERVATION_ZOOM_MIN`]/[`crate::OBSERVATION_ZOOM_MAX`]
//! (0.1〜20.0)の内側に収まる8段(0.125〜16.0)に絞ってある — AE の全12段
//! そのままだと観測カメラの実用範囲を超える端(3.13%/6400%相当)を持て余す。
//!
//! ## 結線は次波
//!
//! ここは純関数のみ。`Message` へのバリアント追加・`StageOverlay::update`/
//! `state_band_view` への配線は行わない(`lib.rs`/`gizmo.rs` は読み専用)。

use std::collections::HashSet;

use glam::Vec2;
use iced::Rectangle;

use motolii_core::CompSpec;
use motolii_engine::ObservationCamera;
use motolii_store::{LayerId, StoreView};

use crate::GizmoTarget;

// ---------------------------------------------------------------------------
// ズーム段(B24)
// ---------------------------------------------------------------------------

/// 離散ズーム段のラダー(モジュール冒頭 doc「先例」参照)。昇順固定。
/// 1.0(観測カメラ既定 = actual-size 相当の基準点ではなく「無変換」点)を
/// 中心に2のべき倍で刻む。
pub const ZOOM_LADDER: &[f32] = &[0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];

/// [`zoom_step`] の向き。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomStepDirection {
    In,
    Out,
}

/// 「ズームイン/ズームアウト段」動詞(map 1441/1442 他)。[`ZOOM_LADDER`] 上で
/// 現在値より真に大きい(In)/小さい(Out)最寄りの段へジャンプする — 現在値が
/// ラダー上に無くても(連続ホイールズーム後の半端な値でも)効く。既にラダー両端
/// に達している/それを超えている場合はその端でクランプ(無反応にはしない —
/// 端の段へ再度 publish しても値は変わらないので、呼び出し側は結線波で
/// 「変化が無ければ Message を出さない」を選べる)。
///
/// `pan` は変えない — [`crate::zoom_at_screen_point`] の導出(モジュール冒頭
/// doc)と同じ写像で、pan 不変のままズームすると「今画面中央に見えている
/// world 点」が動かない(= ビューポート中心アンカー)。ホイール(カーソル
/// アンカー)とは別の、ボタン/ショートカット向けの手触り(先例: AE の
/// `+`/`-` ズームはカーソル位置ではなく現在のビュー中心を保つ)。
pub fn zoom_step(current: ObservationCamera, direction: ZoomStepDirection) -> ObservationCamera {
    let zoom = match direction {
        ZoomStepDirection::In => ZOOM_LADDER
            .iter()
            .copied()
            .filter(|&rung| rung > current.zoom)
            .fold(f32::INFINITY, f32::min),
        ZoomStepDirection::Out => ZOOM_LADDER
            .iter()
            .copied()
            .filter(|&rung| rung < current.zoom)
            .fold(f32::NEG_INFINITY, f32::max),
    };
    let zoom = if zoom.is_finite() {
        zoom
    } else {
        match direction {
            ZoomStepDirection::In => *ZOOM_LADDER.last().expect("ラダーは非空"),
            ZoomStepDirection::Out => ZOOM_LADDER[0],
        }
    };
    ObservationCamera {
        pan: current.pan,
        zoom,
    }
}

/// 「1 comp px = 1 screen px」になる観測ズーム値(map 1490 `Zoom to 100%`/
/// 1480 `Viewer Actual Size`の基準)。[`crate::letterboxed_rect`] が comp を
/// `bounds` へ contain-fit する倍率の逆数 — letterbox が既に comp を
/// `fit_scale` 倍している分を観測ズームで打ち消す。
///
/// `bounds`/`comp` が退化していて letterbox が組めなければ `None`
/// ([`crate::letterboxed_rect`] と同じ契約)。
pub fn actual_size_zoom(bounds: Rectangle, comp: CompSpec) -> Option<f32> {
    let rect = crate::letterboxed_rect(bounds, comp)?;
    if rect.width <= 0.0 {
        return None;
    }
    let raw = comp.width as f32 / rect.width;
    Some(raw.clamp(crate::OBSERVATION_ZOOM_MIN, crate::OBSERVATION_ZOOM_MAX))
}

/// 名前付きズーム段(map 1491/1492 `Zoom to fit`・1490/1480 `Zoom to 100%`/
/// `Viewer Actual Size`、および map に対応行の無い50%/200% — モジュール冒頭
/// doc「map に無い行」参照)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedZoomLevel {
    /// 全体表示。観測カメラを既定(`pan=[0,0]`・`zoom=1.0`)へ戻すのと同じ値 —
    /// letterbox 自体が既に comp 全体を `bounds` へ contain-fit しているので、
    /// 「Fit」は常に `ObservationCamera::default()`(`bounds`/`comp` を問わない、
    /// 退化していても意味が壊れないので唯一 `None` を返さない枝)。
    Fit,
    /// 実寸(1 comp px = 1 screen px)。[`actual_size_zoom`] そのもの。
    ActualSize,
    /// 実寸の50%。
    Half,
    /// 実寸の200%。
    Double,
}

/// [`NamedZoomLevel`] を実際の [`ObservationCamera`] へ写す。`Fit` 以外は
/// [`actual_size_zoom`] に依存するため、letterbox が組めない(`bounds`/`comp`
/// 退化)なら `None`。
pub fn named_zoom_level(bounds: Rectangle, comp: CompSpec, level: NamedZoomLevel) -> Option<ObservationCamera> {
    if level == NamedZoomLevel::Fit {
        return Some(ObservationCamera::default());
    }
    let actual = actual_size_zoom(bounds, comp)?;
    let raw = match level {
        NamedZoomLevel::Fit => unreachable!("上で早期リターン済み"),
        NamedZoomLevel::ActualSize => actual,
        NamedZoomLevel::Half => actual * 0.5,
        NamedZoomLevel::Double => actual * 2.0,
    };
    Some(ObservationCamera {
        pan: [0.0, 0.0],
        zoom: raw.clamp(crate::OBSERVATION_ZOOM_MIN, crate::OBSERVATION_ZOOM_MAX),
    })
}

/// [`fit_to_world_bbox`]/[`fit_to_gizmo_target`] が対象を画面いっぱいに詰め込み
/// すぎないための余白(先例: AE/Figma の「選択にフィット」系動詞は視認用の
/// 余白を残す挙動が一般的 — 自由なデザイン値、裁定に既定値の指定は無い。
/// `crate::ZOOM_STEP_FACTOR` の doc comment と同じ「自由な値」の扱い)。
/// 「選択にフィット」(map に対応行なし、発注書直接指示 — モジュール冒頭 doc
/// 参照)。comp-pixel/world 空間の軸並行 bbox(`bbox_min`..`bbox_max`)が
/// ちょうど comp 全体(= letterbox 後は `bounds` いっぱい)へ収まる観測カメラを
/// 返す。
///
/// **導出**: [`crate::zoom_at_screen_point`] の写像
/// `screen = comp_center + zoom*(world - comp_center - pan)` で、
/// `pan = bbox_center - comp_center` と選ぶと `world = bbox_center` は
/// `screen = comp_center` に写る(bbox 中心を画面中心へ)。この pan の下で
/// bbox の半幅/半高が `comp_center` ちょうどへ写るように zoom を選べば
/// (`zoom = comp.width / bbox_width` 系)bbox は comp の描画域いっぱいに
/// 広がる — そこへ [`FIT_TO_SELECTION_MARGIN`] を掛けて余白を作る。
///
/// `comp` が退化(幅/高さ0)なら `None`。bbox が退化(点、幅/高さ0)でも
/// `zoom` は無限大 → [`crate::OBSERVATION_ZOOM_MAX`] へクランプされるだけで
/// `NaN` にはならない(片方の軸だけ0でも `min` が有限側を拾う)。
pub fn fit_to_world_bbox(comp: CompSpec, bbox_min: [f32; 2], bbox_max: [f32; 2]) -> Option<ObservationCamera> {
    fit_to_world_bbox_with_margin(
        comp,
        bbox_min,
        bbox_max,
        motolii_tokens_rs::Dimensions::default()
            .components
            .stage
            .fit_to_selection_margin,
    )
}

/// `fit_to_world_bbox` のUI余白を呼び手のトークンから受け取る版。既存の純関数
/// 入口は互換のため残し、実際のUI結線はこの版へ寄せられる。
pub fn fit_to_world_bbox_with_margin(
    comp: CompSpec,
    bbox_min: [f32; 2],
    bbox_max: [f32; 2],
    margin: f32,
) -> Option<ObservationCamera> {
    if comp.width == 0 || comp.height == 0 {
        return None;
    }
    let comp_center = [comp.width as f32 * 0.5, comp.height as f32 * 0.5];
    let bbox_center = [
        (bbox_min[0] + bbox_max[0]) * 0.5,
        (bbox_min[1] + bbox_max[1]) * 0.5,
    ];
    let width = (bbox_max[0] - bbox_min[0]).abs();
    let height = (bbox_max[1] - bbox_min[1]).abs();
    let zoom_x = comp.width as f32 / width;
    let zoom_y = comp.height as f32 / height;
    let zoom = (zoom_x.min(zoom_y) * margin)
        .clamp(crate::OBSERVATION_ZOOM_MIN, crate::OBSERVATION_ZOOM_MAX);
    Some(ObservationCamera {
        pan: [bbox_center[0] - comp_center[0], bbox_center[1] - comp_center[1]],
        zoom,
    })
}

/// [`fit_to_world_bbox`] を [`GizmoTarget`](選択レイヤー、`gizmo.rs` 既存の
/// 選択表現)から直接呼ぶための部品。レイヤーローカルの内容矩形
/// `(0,0)..size` を [`GizmoTarget::world_from_local`] で world(comp)空間へ
/// 写し、4隅の軸並行外接矩形(AABB)を bbox として使う(回転済みレイヤーは
/// 見た目の bbox より広い AABB になるが、AE/Figma の「選択にフィット」も
/// 回転済みオブジェクトは同じ AABB 基準 — 新しい規約を発明しない)。
pub fn fit_to_gizmo_target(comp: CompSpec, target: &GizmoTarget) -> Option<ObservationCamera> {
    let world_from_local = target.world_from_local();
    let corners = [
        Vec2::new(0.0, 0.0),
        Vec2::new(target.size[0], 0.0),
        Vec2::new(target.size[0], target.size[1]),
        Vec2::new(0.0, target.size[1]),
    ]
    .map(|local| world_from_local.transform_point2(local));
    let min = [
        corners.iter().map(|c| c.x).fold(f32::INFINITY, f32::min),
        corners.iter().map(|c| c.y).fold(f32::INFINITY, f32::min),
    ];
    let max = [
        corners.iter().map(|c| c.x).fold(f32::NEG_INFINITY, f32::max),
        corners.iter().map(|c| c.y).fold(f32::NEG_INFINITY, f32::max),
    ];
    fit_to_world_bbox(comp, min, max)
}

// ---------------------------------------------------------------------------
// 選択の残り(B31)— モジュール冒頭 doc「なぜ選択の関数もこのファイルに
// 同居するか」参照。
// ---------------------------------------------------------------------------

/// [`step_layer_selection`] の向き。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerStepDirection {
    Next,
    Previous,
}

/// 「次/前のレイヤーを選択」(map 290/291「Select next/previous layer in
/// stacking order」)。「stacking order」は `StoreView::layers()` が返す
/// `LayerId` 昇順をそのまま踏襲する(`motolii-timeline-pane::projection`
/// 冒頭コメント「兄弟の並びは常に LayerId 昇順」と同じ規約 — 新しい順序概念を
/// 増やさない)。
///
/// `current` は「今の単一アンカー」(複数選択の代表 — この関数自体は複数選択を
/// 扱わない、発注書「選択の残り」の対象は単一ステップ動詞のみ)。
/// - `current` が `None`、または今の Document に存在しない id なら、
///   「未選択から矢印キーで最初/最後のレイヤーへ入る」を既定挙動として
///   `Next` は先頭、`Previous` は末尾を返す(AE で選択が無い状態から
///   矢印移動すると先頭/末尾に着地する挙動を踏襲、自由なフォールバック選択)。
/// - 境界(先頭で `Previous`・末尾で `Next`)は wrap せずクランプする。
/// - layer が1つも無ければ `None`。
pub fn step_layer_selection(
    view: &StoreView<'_>,
    current: Option<LayerId>,
    direction: LayerStepDirection,
) -> Option<LayerId> {
    let layers = view.layers();
    if layers.is_empty() {
        return None;
    }
    let index = current.and_then(|id| layers.iter().position(|&candidate| candidate == id));
    let Some(index) = index else {
        return Some(match direction {
            LayerStepDirection::Next => layers[0],
            LayerStepDirection::Previous => *layers.last().expect("空でないことを確認済み"),
        });
    };
    match direction {
        LayerStepDirection::Next => Some(layers[(index + 1).min(layers.len() - 1)]),
        LayerStepDirection::Previous => Some(layers[index.saturating_sub(1)]),
    }
}

/// 「全選択」(map 436 `Select All`/452 `Deselect all layers` の対、290/291と
/// 同じ「stacking order」規約)。**`motolii-shell` 側は既に `Edit` メニューで
/// 同義の動詞を採用済み**(`clipboard::select_all`、モジュール冒頭 doc 参照) —
/// これは pane crate からも呼べる同型の純関数で、shell 側実装を置き換えない
/// (一本化は結線波の判断)。
pub fn select_all_layers(view: &StoreView<'_>) -> Vec<LayerId> {
    view.layers()
}

/// 「全解除」(map 444 `Clear Selection`/452 `Deselect all layers`)。
pub fn deselect_all_layers() -> Vec<LayerId> {
    Vec::new()
}

/// 「選択の反転」(map に対応行なし — モジュール冒頭 doc「map に無い行」
/// 参照。AE/Figma 双方ともレイヤー選択そのものの反転コマンドは持たない
/// (Figma のインバートはブールオペレーション、AE のInvertはマスク専用 —
/// 1382/1383/1424 宿無し行の判定と同じ「似て非なる動詞に丸めない」原則で
/// 別物と確認した上で、発注書が明示要求した動詞として実装する)。
///
/// 今の Document に存在する layer のうち、`current` に**含まれていない**物を
/// 返す(`current` に含まれるが今は存在しない id は無視 — 削除済み layer が
/// 反転後の集合に紛れ込まない)。
pub fn invert_layer_selection(view: &StoreView<'_>, current: &[LayerId]) -> Vec<LayerId> {
    let current: HashSet<LayerId> = current.iter().copied().collect();
    view.layers()
        .into_iter()
        .filter(|layer| !current.contains(layer))
        .collect()
}
