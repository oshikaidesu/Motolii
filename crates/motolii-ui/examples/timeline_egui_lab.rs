//! egui Timeline の Lab。**実際の Document を、実際の行モデルで、手で触る。**
//!
//! ここには偽のTimeline状態を置かない。行は `timeline_rows::rows()` が
//! 実 Document から毎フレーム作ったものであり、開閉はその `TimelineFoldState` である。
//! だから**この窓で見えていることは、そのまま製品の行モデルの挙動**である。
//!
//! 触れるもの:
//!   - グループ左端の `▸`/`▾` … 子レイヤーの開閉
//!   - `◇`/`◆`            … そのレイヤーのキーパラメータ行の開閉（子とは独立）
//!   - clip bar をドラッグ … 時間方向へ移動。**キーが一緒に動く**
//!
//!   - bar の左右端6px      … トリム（in / out）
//!   - Position 行の菱形    … 掴んでキーの時刻を変える
//!   - 左列を上下へドラッグ … **並べ替え。Group の中へも出し入れできる**
//!   - `M` / `S`           … mute（`visible`）/ solo の反転
//!   - 左列クリック         … 選択。`Cmd` で足し引き、`Shift` で範囲
//!   - `Cmd/Ctrl + Z` / `Shift+Z` … Undo / Redo
//!   - `Cmd/Ctrl + D`      … 選択中の layer を複製
//!   - `Delete` / `Backspace` … 選択中の layer を削除（Group は中身ごと）
//!
//! ## 面の動かし方（AE / Premiere と同じ割り当て）
//!
//! ```text
//! ホイール / 二本指        縦スクロール
//! Shift + ホイール         横パン
//! Cmd(⌘) + ホイール        横ズーム（カーソル下の時刻が動かない）
//! ピンチ                   横ズーム
//! 下端のナビゲータ帯       掴んで横パン、両端6pxでズーム
//! ```
//!
//! **レイヤーの全体地図(minimap)は置かない。** 1 Layer = 1行なので行の一覧が
//! 既に全体図であり、その縮小版を別に持っても情報が増えない。時間方向だけは
//! 寄ると全体が見えなくなるので、そこにナビゲータ帯を置く。
//!
//! **ドラッグは Document を実際に書き換える。** 経路は
//! `DocumentWriter::prepare_* -> apply_command(gesture, command)` で、
//! 1ドラッグ = 1 `GestureId` = 1 Undo 単位である。撤去された
//! `document_edit_runtime` を経由しない — D2 が既に薄い入口を持っていた。
//!
//! 実行: `cargo run --profile fast -p motolii-ui --example timeline_egui_lab`

use std::collections::HashMap;

use eframe::egui;
use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Vec2};
use motolii_core::{Fps, RationalTime};
use motolii_doc::{
    collect_layer_ids, find_item_location, Clip, ClipSource, Command, DocKeyframe,
    DocKeyframeTrack, DocParam, DocValue, Document, DocumentWriter, GestureId, Group, ItemEnvelope,
    KeyframeId, LayerId, ParentLocator, Track, TrackItem, Transform2D,
};
use std::sync::Arc;
use motolii_eval::Interp;
use motolii_ui::timeline_rows::{rows, ParamRef, RowKind, TimelineFoldState, TimelineRow};

// mock_tokens が timeline-library.html から出した値（面の大きさで動かないものだけ）
const BG: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
const HEAD_BG: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
const CELL: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
const CELL_CHILD: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
const CELL_PROP: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
const TRACK_A: Color32 = Color32::from_rgb(0x37, 0x37, 0x37);
const TRACK_B: Color32 = Color32::from_rgb(0x25, 0x25, 0x25);
/// 数字を持たない細目盛。**主目盛より弱く、下地より濃い**
const TRACK_MINOR: Color32 = Color32::from_rgb(0x2b, 0x2b, 0x2b);
/// 1区間おきの下地。**目盛と目盛のあいだを図にする**(Ableton の明暗)
const TRACK_BAND: Color32 = Color32::from_rgb(0x2f, 0x2f, 0x2f);
const RULE: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
const INK: Color32 = Color32::from_rgb(0xd4, 0xd4, 0xd4);
const DIM: Color32 = Color32::from_rgb(0x8d, 0x8d, 0x8d);
const ACCENT: Color32 = Color32::from_rgb(0xe9, 0xcf, 0x72);
const KEY_IDLE: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);
// M / S が入っているときの下地。モックの採用値
const MUTE_ON: Color32 = Color32::from_rgb(0x65, 0x3b, 0x34);
const SOLO_ON: Color32 = Color32::from_rgb(0x66, 0x5b, 0x32);

const RAIL_W: f32 = 196.0;
const ROW_H: f32 = 24.0;
const PROP_H: f32 = 20.0;
const HEAD_H: f32 = 34.0;
const RULER_H: f32 = 27.0;
/// ルーラが覆う秒数の**初期値**。以後は `TimelineView` が持つ。
const TIMELINE_SECONDS: f32 = 16.0;
/// これ以上は寄れない。1秒を4分割まで
const MIN_SPAN: f32 = 0.25;
/// 下端の時間ナビゲータ帯の高さ
const NAV_H: f32 = 14.0;
/// 縦のつまみの幅
const SCROLLBAR_W: f32 = 6.0;

/// 時間軸の見えている窓。**Project session が持つ状態**で、Document には入れない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineView {
    /// 左端の時刻(秒)
    pub start: f32,
    /// 見えている秒数
    pub span: f32,
}

impl TimelineView {
    /// 時刻 → 面の中の x。**時間↔x の換算はこの2本しか無い。**
    pub fn time_to_x(&self, t: f32, track_left: f32, track_w: f32) -> f32 {
        track_left + (t - self.start) / self.span * track_w
    }

    /// x → 時刻。`time_to_x` の逆。
    pub fn x_to_time(&self, x: f32, track_left: f32, track_w: f32) -> f32 {
        self.start + (x - track_left) / track_w * self.span
    }

    /// `anchor` の時刻を動かさずに寄る/引く。`factor` < 1 で寄る。
    ///
    /// **カーソルの下の時刻が動かないことが、ズームの手触りそのもの**である。
    pub fn zoom_at(self, anchor: f32, factor: f32, comp: f32) -> TimelineView {
        let span = (self.span * factor).clamp(MIN_SPAN, comp.max(MIN_SPAN));
        // anchor が窓の中で占める割合を保つ
        let ratio = if self.span > 0.0 {
            (anchor - self.start) / self.span
        } else {
            0.0
        };
        TimelineView {
            start: anchor - ratio * span,
            span,
        }
        .clamped(comp)
    }

    /// 横へずらす。
    pub fn pan(self, delta_seconds: f32, comp: f32) -> TimelineView {
        TimelineView {
            start: self.start + delta_seconds,
            span: self.span,
        }
        .clamped(comp)
    }

    /// **窓は composition の外へ出ない。** 0より前も、終端より後ろも見せない。
    pub fn clamped(self, comp: f32) -> TimelineView {
        let span = self.span.clamp(MIN_SPAN, comp.max(MIN_SPAN));
        let start = self.start.clamp(0.0, (comp - span).max(0.0));
        TimelineView { start, span }
    }
}

/// 並べ替えの落とし先。**行と行のあいだ**を指す。
///
/// `index` は D2 の `prepare_reparent_clip` と同じ意味、つまり
/// **外したあとの挿入位置**である。同じ親の中で下へ動かすときに1つずれるのは
/// このためで、`drop_target` がその調整を持つ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropTarget {
    pub parent: ParentLocator,
    pub index: usize,
    /// 線を引く y。**絵のためだけに持つ**
    pub y: f32,
}

/// 1フレームで進める上限(秒)。
///
/// **窓が隠れていた分をまとめて進めない。** eframe は見えていないと描画を間引くので、
/// 戻ってきたフレームの `dt` は数百msになりうる。そのまま足すと playhead が飛ぶ。
const MAX_STEP: f32 = 0.05;

/// `dt` 秒ぶん進んだ playhead と、**まだ再生中か**。
///
/// 終端で止まる(巻き戻さない)。頭へ戻すかどうかは押した側の判断である。
fn advance_playhead(playhead: f32, dt: f32, comp: f32) -> (f32, bool) {
    let at = playhead + dt.max(0.0);
    if at >= comp {
        (comp, false)
    } else {
        (at, true)
    }
}

/// 数字を出す目盛の間隔(秒)。**窓に16本前後入る、切りのいい値**を選ぶ。
///
/// 寄るとフレームの倍数へ、引くと秒・分の倍数へ移る。候補にしか無い値は出さない
/// — 「0.37秒ごと」のような目盛は読めないので。
fn tick_step(span: f32, fps: Fps) -> f32 {
    let frame = 1.0 / fps.as_f64() as f32;
    let target = span / 16.0;
    let mut candidates = vec![frame, frame * 2.0, frame * 5.0, frame * 10.0];
    candidates.extend_from_slice(&[
        0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0,
    ]);
    candidates.sort_by(f32::total_cmp);
    candidates
        .into_iter()
        .find(|c| *c >= target)
        .unwrap_or(600.0)
}

/// 数字を出さない細かい目盛の間隔。**`major` を割り切り、1フレームより細かくしない。**
///
/// これが「あいだがどれくらいか」を数えられる目盛で、数字の密度を上げずに
/// 読み取れる分解能だけを上げる。フレームまで寄ったら細目盛は消える。
fn minor_step(major: f32, fps: Fps) -> Option<f32> {
    let frame = 1.0 / fps.as_f64() as f32;
    // 5等分が基本。60秒台だけは 4等分(15秒)のほうが読める
    let divisor = if (major - 60.0).abs() < 1e-3 || (major - 120.0).abs() < 1e-3 {
        4.0
    } else {
        5.0
    };
    let candidate = major / divisor;
    if candidate >= frame * 0.999 {
        Some(candidate)
    } else if frame < major * 0.5 {
        Some(frame)
    } else {
        None
    }
}

/// いま見えている窓に入る、その間隔の目盛の時刻。
///
/// **目盛は時刻に貼り付く。** 画面をN等分すると、パンしても線が動かず
/// 数字だけが変わる — 方眼が紙ではなく窓に貼られているように見えてしまう。
fn ticks_every(view: TimelineView, step: f32) -> Vec<f32> {
    let first = (view.start / step).floor() * step;
    let mut out = Vec::new();
    let mut t = first;
    // 端数で無限に回らないよう、本数で止める
    while t <= view.start + view.span + step * 0.5 && out.len() < 1024 {
        if t >= -1e-4 {
            out.push(t);
        }
        t += step;
    }
    out
}

/// 数字を出す目盛
fn ticks(view: TimelineView, fps: Fps) -> Vec<f32> {
    ticks_every(view, tick_step(view.span, fps))
}

/// その区間を暗いほうで塗るか。**絶対時刻で決める。**
///
/// 窓の中で何本目かで決めると、パンした瞬間に縞が入れ替わって画面が沸く。
/// 0秒から数えた区間の偶奇なら、窓をどう動かしても縞は動かない。
fn band_is_dark(t: f32, step: f32) -> bool {
    if step <= 0.0 {
        return false;
    }
    (t / step).round() as i64 % 2 != 0
}

/// 目盛の文字。**間隔より細かい桁は出さない**
fn tick_label(t: f32, step: f32) -> String {
    let minutes = (t / 60.0).floor().max(0.0);
    let seconds = t - minutes * 60.0;
    if step >= 1.0 {
        format!("{minutes}:{seconds:04.1}")
    } else {
        format!("{minutes}:{seconds:05.2}")
    }
}

/// ナビゲータ帯のどこを掴んだか
#[derive(Debug, Clone, Copy, PartialEq)]
enum NavGrab {
    Pan,
    Left,
    Right,
}

/// ポインタの y が、object 行の**何番目の上の境界**に居るか。
///
/// 行の中心で切り替える。`objects.len()` は「いちばん下」を指す
fn boundary_at(objects: &[(LayerId, f32, f32)], y: f32) -> usize {
    for (i, (_, top, h)) in objects.iter().enumerate() {
        if y < top + h * 0.5 {
            return i;
        }
    }
    objects.len()
}

/// その境界に線を引く y
fn boundary_y(objects: &[(LayerId, f32, f32)], boundary: usize) -> f32 {
    match objects.get(boundary) {
        Some((_, top, _)) => *top,
        None => objects.last().map(|(_, top, h)| top + h).unwrap_or(0.0),
    }
}

/// 行の合計高。縦スクロールの上限はここから出る
fn content_height(rows: &[TimelineRow]) -> f32 {
    rows.iter()
        .map(|r| match r.kind {
            RowKind::Object => ROW_H,
            RowKind::Property(_) => PROP_H,
        })
        .sum()
}

/// **面より短い中身はスクロールしない。** 下は最後の行で止まる
fn clamp_scroll(scroll: f32, content: f32, viewport: f32) -> f32 {
    scroll.clamp(0.0, (content - viewport).max(0.0))
}

/// `layer` の subtree に `maybe` が含まれるか。**自分自身は含めない**
fn is_descendant(document: &Document, layer: LayerId, maybe: LayerId) -> bool {
    if layer == maybe {
        return false;
    }
    let Some(item) = find_item(document, layer) else {
        return false;
    };
    let mut ids = Vec::new();
    collect_layer_ids(item, &mut ids);
    ids.contains(&maybe)
}

/// 境界 `boundary`(object 行の何番目の**上**か)へ `dragged` を落とすとき、
/// D2 へ渡す `(parent, index)` を出す。
///
/// - 境界の**下にある行**の位置がそのまま挿入位置になる。開いた Group の
///   最初の子の上へ落とせば、それは「Group の中の先頭」である
/// - 末尾の境界だけは、**最後の行の次**を指す
/// - **自分自身の中へは落とせない。** Group を自分の子の中へ入れると木が壊れる
/// - 同じ親の中で下へ動かすときは1つ引く。`index` は外したあとの位置なので
fn drop_target(
    document: &Document,
    objects: &[LayerId],
    boundary: usize,
    dragged: LayerId,
) -> Option<(ParentLocator, usize)> {
    let (parent, mut index) = if boundary < objects.len() {
        let (parent, index, _) = find_item_location(document, objects[boundary])?;
        (parent, index)
    } else {
        let (parent, index, _) = find_item_location(document, *objects.last()?)?;
        (parent, index + 1)
    };
    if let ParentLocator::Group(group) = parent {
        if group == dragged || is_descendant(document, dragged, group) {
            return None;
        }
    }
    let (old_parent, old_index, _) = find_item_location(document, dragged)?;
    if old_parent == parent && old_index < index {
        index -= 1;
    }
    Some((parent, index))
}

fn main() -> eframe::Result<()> {
    let shot = std::env::args().nth(1);
    eframe::run_native(
        "Timeline egui Lab — 実Documentを実行モデルで",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 460.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            motolii_ui::install_egui_symbol_fallback(&cc.egui_ctx);
            Ok(Box::new(Lab::new(shot)))
        }),
    )
}

/// 掴んでいるもの。**何を掴んだかで、出す command が変わる**
#[derive(Clone)]
enum Grab {
    /// **Group を掴んだら子も同じ差分で動く。**`targets` は動かす clip と、
    /// 掴んだ瞬間の開始時刻(秒)。Group 自身は clip ではないので入らない
    Move {
        layer: LayerId,
        grab_at: f32,
        targets: Vec<(LayerId, f32, f32)>,
        /// 追従させる Position キーと、掴んだ瞬間の時刻(秒)。
        /// **絶対値で出し直すので、掴んだ瞬間の値を持ったままにする**
        /// 追従させるキー。**param ごとに出す command が違う**ので param も持つ
        keys: Vec<(LayerId, ParamRef, KeyframeId, f32)>,
        /// 追従できないキーの数。Scale/Rotation/Anchor/Opacity には
        /// 時刻を変える `prepare_*` が無い
        not_movable: usize,
    },
    /// Position キー1つを掴んで時刻を変える
    KeyTime {
        layer: LayerId,
        key: KeyframeId,
        grab_at: f32,
        original: f32,
    },
    TrimIn { layer: LayerId },
    TrimOut { layer: LayerId },
    /// 左列を掴んで上下へ。**離した瞬間に1回だけ書く。**
    /// 途中の位置は線で見せるだけで、Document は動かさない — 通り道の親へ
    /// 一度ずつ入れ直すと、1ドラッグが N 個の編集になってしまう
    Reorder { layer: LayerId },
}

/// clip / group を掴んだ瞬間の状態を採る。
///
/// **動くもの(clip の開始時刻)と、追従するもの(その clip の Position キー)を
/// ここで一度に確定させる。** ドラッグ中は毎フレーム絶対値で出し直すので、
/// 掴んだ瞬間の値が要る。
/// 1つだけ掴む。**窓は必ず複数選択の道(`begin_move_many`)を通る**ので、
/// これを呼ぶのはテストである。
#[cfg(test)]
fn begin_move(document: &Document, layer: LayerId, grab_at: f32) -> Grab {
    begin_move_many(document, &[layer], layer, grab_at)
}

/// 複数選択のドラッグ。**選ばれている全部が同じ差分で動く。**
///
/// `layer` は掴んだ行(status に出す代表)で、`roots` が実際に動かす集合である。
/// 親と子が両方選ばれていても、**動く clip は重複しない** — `movable_clips` が
/// 子孫まで返すので、集めたあとに LayerId で畳む。
fn begin_move_many(
    document: &Document,
    roots: &[LayerId],
    layer: LayerId,
    grab_at: f32,
) -> Grab {
    let mut targets: Vec<(LayerId, f32, f32)> = Vec::new();
    for root in roots {
        for target in movable_clips(document, *root) {
            if !targets.iter().any(|(l, _, _)| *l == target.0) {
                targets.push(target);
            }
        }
    }
    let mut keys = Vec::new();
    let mut not_movable = 0usize;
    for (clip, _, _) in &targets {
        // **envelope が持つ param は全部追従できる**(2026-08-16 に D2 の
        // `SetTransformParamKeyTime` と受け付け集合の統合が入った)。
        // 追従できないのは plugin 由来(EffectParam / SourceParam)だけで、
        // Lab はまだそれを描いていないので 0 のまま
        for param in [
            ParamRef::Position,
            ParamRef::Anchor,
            ParamRef::Scale,
            ParamRef::Rotation,
            ParamRef::Opacity,
        ] {
            for (key, t) in param_keys(document, *clip, param) {
                keys.push((*clip, param, key, t));
            }
        }
    }
    let _ = &mut not_movable;
    Grab::Move {
        layer,
        grab_at,
        targets,
        keys,
        not_movable,
    }
}

struct Lab {
    writer: DocumentWriter,
    document: Arc<Document>,
    /// `document` を取ったときの `writer.revision`。**これが取り直しの唯一の合図**
    revision: u64,
    fold: TimelineFoldState,
    drag: Option<(Grab, GestureId)>,
    /// 掴んだ瞬間の `undo_len`。Esc で戻すときに、この gesture が実際に何か
    /// 積んだのかを見る。**積んでいないなら undo しない** — 掴む前の、
    /// 関係ない編集を取り消してしまう
    drag_undo_base: usize,
    names: HashMap<LayerId, String>,
    /// 選択中の layer。**Project session が持つ種類の状態**で、Document には入れない。
    /// **順序は選んだ順**で、末尾が Shift 範囲選択の起点になる
    selected: Vec<LayerId>,
    /// 見えている時間の窓。同上
    view: TimelineView,
    /// 縦スクロール(px)。行の合計高が面より高いときだけ動く。同上
    scroll_y: f32,
    /// 並べ替えのドラッグ中に、いま落とすと決まる場所。**線を描く位置でもある**
    drop: Option<DropTarget>,
    /// ナビゲータ帯を掴んでいる間の掴み方。掴んだ瞬間に決めて、離すまで変えない
    nav: Option<NavGrab>,
    /// playhead(秒)。同上
    playhead: f32,
    /// 再生中か。**Space で入り切りする**。Document には入れない
    playing: bool,
    status: String,
    shot: Option<String>,
    frame: u32,
}

impl Lab {
    fn new(shot: Option<String>) -> Self {
        let (doc, names) = fixture();
        let catalog = Arc::new(
            motolii_plugin::reference::reference_catalog().expect("reference catalog"),
        );
        let writer = DocumentWriter::new(doc, catalog).expect("writer");
        let document = writer.snapshot();
        let revision = writer.revision;
        let mut fold = TimelineFoldState::default();
        // 最初から階層が見えているほうが、行モデルの確認になる
        for (&layer, name) in &names {
            match name.as_str() {
                // 子の軸
                "Title scene" => fold.open_children(layer),
                // パラメータの軸。両方が同時に効くことを最初の画面で見せる
                "Shared left" => fold.open_params(layer),
                _ => {}
            }
        }
        if let Some((&layer, _)) = names.iter().find(|(_, n)| n.as_str() == "Title scene") {
            fold.open_params(layer);
        }
        Self {
            writer,
            document,
            revision,
            fold,
            drag: None,
            drag_undo_base: 0,
            names,
            selected: Vec::new(),
            view: TimelineView {
                start: 0.0,
                span: TIMELINE_SECONDS,
            },
            scroll_y: 0.0,
            drop: None,
            nav: None,
            playhead: 0.0,
            playing: false,
            status: "space=play  drag bar=move  drag name=reorder  Cmd/Shift+click=select".to_owned(),
            shot,
            frame: 0,
        }
    }

    /// ポインタの時刻(秒)を、掴んでいるものに応じた D2 command にして適用する。
    ///
    /// **prepare_* が `Ok(None)` を返したら、それは「変化なし」であって失敗ではない。**
    /// 落ちた編集は status に出す。通ったことにしない。
    /// param ごとに、時刻を動かす command を選ぶ。
    ///
    /// Position だけ専用の入口があり(`SetPositionKeyTime`)、他は
    /// `SetTransformParamKeyTime` が受ける。**将来ここは1本に畳む**
    /// (台帳「キー編集APIを `ScalarPropertyId` 1本へ畳む」参照)。
    fn key_time_command(
        &self,
        layer: LayerId,
        param: ParamRef,
        key: KeyframeId,
        t: RationalTime,
    ) -> Result<Option<motolii_doc::Command>, String> {
        match param {
            ParamRef::Position => self
                .writer
                .prepare_set_position_key_time(layer, key, t)
                .map_err(|e| e.to_string()),
            other => self
                .writer
                .prepare_set_transform_param_key_time(layer, scalar_property(other), key, t)
                .map_err(|e| e.to_string()),
        }
    }

    fn commit_drag(&mut self, at_seconds: f32) {
        let Some((grab, gesture)) = self.drag.clone() else {
            return;
        };
        // **編集が作る時刻は全部フレーム境界へ乗る。** fps はここで1回だけ読む
        let fps = self.document.composition.fps;
        let Some(time) = seconds_to_time(at_seconds, fps) else {
            return;
        };

        // 掴んだものごとに、出す command の並びを作る。
        // 各要素は (layer, これはキーの command か, 準備結果)。
        // **Move だけが複数になりうる**(Group は子をまとめて動かす)
        let mut prepared = Vec::new();
        match &grab {
            Grab::Move {
                grab_at,
                targets,
                keys,
                ..
            } => {
                if targets.is_empty() {
                    self.status = "nothing to move (empty group)".to_owned();
                    return;
                }
                // **塊のまま動く。** 誰か1人でも端を越えるなら、全員そこで止まる。
                // 越えた者だけ置いていくと、Group の中の相対位置が壊れる
                let comp = self.document.composition.duration.as_seconds_f64() as f32;
                let floor = targets
                    .iter()
                    .map(|(_, start, _)| *start)
                    .fold(f32::INFINITY, f32::min);
                let headroom = targets
                    .iter()
                    .map(|(_, _, end)| comp - *end)
                    .fold(f32::INFINITY, f32::min);
                let delta = (at_seconds - grab_at).clamp(-floor, headroom.max(0.0));
                for (layer, start, _) in targets {
                    let Some(t) = seconds_to_time((start + delta).max(0.0), fps) else {
                        return;
                    };
                    prepared.push((*layer, false, self.writer.prepare_set_clip_start(*layer, t).map_err(|e| e.to_string())));
                }
                // **キーは出す順が要る。** 同じ時刻に2つは置けないので、後ろへ動かす
                // ときは遅いキーから、前へ動かすときは早いキーから出す。全員が同じ
                // delta で動くので、この順なら途中でぶつからない
                let mut ordered = keys.clone();
                ordered.sort_by(|a, b| {
                    if delta >= 0.0 {
                        b.3.total_cmp(&a.3)
                    } else {
                        a.3.total_cmp(&b.3)
                    }
                });
                for (layer, param, key, original) in ordered {
                    let Some(t) = seconds_to_time((original + delta).max(0.0), fps) else {
                        return;
                    };
                    prepared.push((layer, true, self.key_time_command(layer, param, key, t)));
                }
            }
            Grab::KeyTime {
                layer,
                key,
                grab_at,
                original,
            } => {
                // Move と同じ考え方でクランプする。0秒より前・composition の
                // 終端より後ろへは出さない
                let comp = self.document.composition.duration.as_seconds_f64() as f32;
                let moved = (original + (at_seconds - grab_at)).clamp(0.0, comp.max(0.0));
                let Some(t) = seconds_to_time(moved, fps) else {
                    return;
                };
                prepared.push((
                    *layer,
                    true,
                    self.writer.prepare_set_position_key_time(*layer, *key, t).map_err(|e| e.to_string()),
                ));
            }
            Grab::TrimIn { layer } => {
                prepared.push((*layer, false, self.writer.prepare_trim_clip_in(*layer, time).map_err(|e| e.to_string())))
            }
            Grab::TrimOut { layer } => prepared.push((
                *layer,
                false,
                self.writer.prepare_trim_clip_out(*layer, time).map_err(|e| e.to_string()),
            )),
            // 並べ替えは離した瞬間にしか書かない。ドラッグ中はここへ来ない
            Grab::Reorder { .. } => return,
        }

        let mut applied = 0usize;
        let mut keys_applied = 0usize;
        for (layer, is_key, result) in prepared {
            match result {
                Ok(Some(command)) => match self.writer.apply_command(gesture, command) {
                    Ok(()) => {
                        if is_key {
                            keys_applied += 1;
                        } else {
                            applied += 1;
                        }
                    }
                    Err(error) => {
                        self.status = format!("{} rejected: {error}", self.name(layer));
                        return;
                    }
                },
                Ok(None) => {}
                Err(error) => {
                    self.status = format!("{} rejected: {error}", self.name(layer));
                    return;
                }
            }
        }
        if applied + keys_applied > 0 {
            refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
            // **追従できなかったキーを黙らせない。** Position 以外は動いていない
            let detail = match &grab {
                Grab::Move { not_movable, .. } => format!(
                    "{applied} clip, {keys_applied} keys followed, {not_movable} not movable"
                ),
                Grab::KeyTime { .. } => format!("{keys_applied} key"),
                Grab::TrimIn { .. } | Grab::TrimOut { .. } => format!("{applied} clip"),
                Grab::Reorder { .. } => String::new(),
            };
            self.status = format!(
                "{} @ {at_seconds:.2}s  ({detail})  undo {}",
                self.name(grab_layer(&grab)),
                self.writer.undo_len()
            );
        }
    }

    /// ドラッグ中の Esc。**掴む前へ戻す。**
    ///
    /// 1ドラッグ = 1 `GestureId` なので、その gesture が積んだものは undo 1回で
    /// まとめて戻る。まだ何も積んでいない(掴んだだけで動かしていない)ときは
    /// **undo しない** — 掴む前の、関係ない編集を取り消してしまう。
    fn cancel_drag(&mut self) {
        let Some((grab, _)) = self.drag.take() else {
            return;
        };
        let layer = grab_layer(&grab);
        if self.writer.undo_len() <= self.drag_undo_base {
            self.status = format!("{} cancelled (nothing to undo)", self.name(layer));
            return;
        }
        match self.writer.undo() {
            Ok(()) => {
                refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                self.status = format!(
                    "{} cancelled  (undo {})",
                    self.name(layer),
                    self.writer.undo_len()
                );
            }
            Err(error) => self.status = format!("cancel rejected: {error}"),
        }
    }

    /// M / S を反転して Document へ書く。**1クリック = 1 `GestureId` = 1 Undo 単位**
    fn toggle_flag(&mut self, layer: LayerId, mute: bool) {
        let Some((visible, solo)) = item_flags(&self.document, layer) else {
            return;
        };
        let gesture = self.writer.begin_gesture();
        let prepared = if mute {
            self.writer.prepare_set_item_visible(layer, !visible)
        } else {
            self.writer.prepare_set_item_solo(layer, !solo)
        };
        match prepared {
            Ok(Some(command)) => match self.writer.apply_command(gesture, command) {
                Ok(()) => {
                    refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                    let what = match (mute, visible, solo) {
                        (true, true, _) => "mute",
                        (true, false, _) => "unmute",
                        (false, _, false) => "solo",
                        (false, _, true) => "unsolo",
                    };
                    self.status = format!(
                        "{} {what}  undo {}",
                        self.name(layer),
                        self.writer.undo_len()
                    );
                }
                Err(error) => self.status = format!("{} rejected: {error}", self.name(layer)),
            },
            Ok(None) => {}
            Err(error) => self.status = format!("{} rejected: {error}", self.name(layer)),
        }
    }

    fn is_selected(&self, layer: LayerId) -> bool {
        self.selected.contains(&layer)
    }

    /// 行をクリックしたときの選択。**普通のリストと同じ3通り。**
    ///
    /// - そのまま  … その1つだけにする
    /// - `Cmd`    … 足す / 外す
    /// - `Shift`  … 直前に触った行からここまで(**見えている object 行の上で**数える。
    ///   閉じた Group の中は見えないので入らない)
    fn select(&mut self, layer: LayerId, additive: bool, range: bool, objects: &[LayerId]) {
        if range {
            if let Some(anchor) = self.selected.last().copied() {
                let (Some(a), Some(b)) = (
                    objects.iter().position(|l| *l == anchor),
                    objects.iter().position(|l| *l == layer),
                ) else {
                    return;
                };
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                // anchor を末尾に残す — 続けて Shift を押したときの起点が動かない
                let mut next: Vec<LayerId> =
                    objects[lo..=hi].iter().copied().filter(|l| *l != anchor).collect();
                next.push(anchor);
                self.selected = next;
                self.status = format!("{} selected", self.selected.len());
                return;
            }
        }
        if additive {
            if let Some(at) = self.selected.iter().position(|l| *l == layer) {
                self.selected.remove(at);
            } else {
                self.selected.push(layer);
            }
        } else {
            self.selected = vec![layer];
        }
        self.status = match self.selected.len() {
            0 => "nothing selected".to_owned(),
            1 => format!("selected {}", self.name(self.selected[0])),
            n => format!("{n} selected"),
        };
    }

    /// 選択のうち、**他の選択の子孫でないもの**だけ。
    ///
    /// 親 Group と子が同時に選ばれているとき、複製すると子が2重に増え、
    /// 削除すると親を消した時点で子が消えて `LayerNotFound` になる。
    /// **構造を触る操作は必ずここを通す。**
    fn selection_roots(&self) -> Vec<LayerId> {
        self.selected
            .iter()
            .copied()
            .filter(|layer| {
                !self
                    .selected
                    .iter()
                    .any(|other| is_descendant(&self.document, *other, *layer))
            })
            .collect()
    }

    /// 選択中の layer を丸ごと複製する。**1複製 = 1 `GestureId` = 1 Undo 単位**
    ///
    /// **深いところは D2 がやる。** `prepare_duplicate_track_item` は Group の
    /// 子も、シェイプの中の入れ子(`VectorContent::Group`)も再帰して写し、
    /// LayerId / KeyframeId / EffectId を全部新しく振り直す。Lab が子を辿って
    /// 複製し直すと、その再写像を二重にしてしまう — **ここでは source を1つ渡すだけ**。
    ///
    /// 複製後は**増えたほうを選ぶ**。続けて動かすのが普通なので
    fn duplicate_selected(&mut self) {
        let roots = self.selection_roots();
        if roots.is_empty() {
            self.status = "nothing selected".to_owned();
            return;
        }
        let gesture = self.writer.begin_gesture();
        let mut made = Vec::new();
        for layer in roots {
            let name = self.name(layer).to_owned();
            let command = match self.writer.prepare_duplicate_track_item(layer) {
                Ok(command) => command,
                Err(error) => {
                    self.status = format!("{name} rejected: {error}");
                    return;
                }
            };
            // 増えたほうの LayerId は command の中にしか無い(まだ Document に無い)
            if let Command::AddTrackItem { item, .. } = &command {
                let mut ids = Vec::new();
                collect_layer_ids(item, &mut ids);
                made.extend(ids.first().copied());
            }
            if let Err(error) = self.writer.apply_command(gesture, command) {
                self.status = format!("{name} rejected: {error}");
                return;
            }
        }
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        // **1つなら名前を出す。** 何が増えたか分からない状態にしない
        let what = match made.as_slice() {
            [one] => self.name(*one).to_owned(),
            many => format!("{}", many.len()),
        };
        self.status = format!("duplicated {what}  undo {}", self.writer.undo_len());
        self.selected = made;
    }

    /// 選択中の layer を消す。**Group は中身ごと。1回の Delete = 1 Undo 単位**
    ///
    /// 消す順は関係ない — `selection_roots` が親子の重なりを外しているので、
    /// どれを先に消しても残りの `prepare` は当たる。
    fn delete_selected(&mut self) {
        let roots = self.selection_roots();
        if roots.is_empty() {
            self.status = "nothing selected".to_owned();
            return;
        }
        let gesture = self.writer.begin_gesture();
        let mut removed = 0usize;
        for layer in &roots {
            let name = self.name(*layer).to_owned();
            let command = match self.writer.prepare_remove_track_item(*layer) {
                Ok(command) => command,
                Err(error) => {
                    self.status = format!("{name} rejected: {error}");
                    return;
                }
            };
            if let Err(error) = self.writer.apply_command(gesture, command) {
                self.status = format!("{name} rejected: {error}");
                return;
            }
            removed += 1;
        }
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        self.selected.clear();
        self.status = format!("deleted {removed}  undo {}", self.writer.undo_len());
    }

    /// 並べ替えを1回だけ書く。**離した瞬間に呼ぶ。**
    ///
    /// 時刻は変えない(`new_start = None`) — 上下に動かしただけで clip が
    /// 時間方向へ跳ぶと、並べ替えのつもりが編集になってしまう。
    fn commit_reorder(&mut self, layer: LayerId, to: DropTarget) {
        let name = self.name(layer).to_owned();
        let prepared = self
            .writer
            .prepare_reparent_clip(layer, to.parent, to.index, None);
        match prepared {
            Ok(Some(command)) => {
                let gesture = self.writer.begin_gesture();
                match self.writer.apply_command(gesture, command) {
                    Ok(()) => {
                        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                        self.status = format!("moved {name}  undo {}", self.writer.undo_len());
                    }
                    Err(error) => self.status = format!("{name} rejected: {error}"),
                }
            }
            // 同じ場所へ落とした。**失敗ではない**
            Ok(None) => self.status = format!("{name} stayed"),
            Err(error) => self.status = format!("{name} rejected: {error}"),
        }
    }

    /// 下端の時間ナビゲータ帯。**寄っているときに、いま全体のどこを見ているか。**
    ///
    /// 溝が composition 全体、明るい所が見えている窓である。
    /// **掴めば横パン、両端6pxを掴めばズーム** — ホイールを縦へ渡した分の
    /// 代わりがここにある。レイヤーの地図は置かない(行そのものが一覧なので)。
    fn navigator(
        &mut self,
        ui: &mut egui::Ui,
        p: &egui::Painter,
        full: Rect,
        track_left: f32,
        comp: f32,
    ) {
        if comp <= 0.0 {
            return;
        }
        let bar = Rect::from_min_max(
            egui::pos2(track_left, full.bottom() - NAV_H),
            egui::pos2(full.right(), full.bottom()),
        );
        p.rect_filled(
            Rect::from_min_max(egui::pos2(full.left(), bar.top()), full.max),
            CornerRadius::ZERO,
            Color32::from_rgb(0x1e, 0x1e, 0x1e),
        );
        let to_x = |t: f32| bar.left() + (t / comp) * bar.width();
        let (x0, x1) = (to_x(self.view.start), to_x(self.view.start + self.view.span));
        let knob = Rect::from_min_max(
            egui::pos2(x0, bar.top() + 3.0),
            egui::pos2(x1.max(x0 + 8.0), bar.bottom() - 3.0),
        );
        let r = ui.interact(bar, ui.id().with("nav"), Sense::click_and_drag());
        if r.drag_started() {
            self.nav = r.interact_pointer_pos().map(|pos| {
                if (pos.x - knob.left()).abs() <= 6.0 {
                    NavGrab::Left
                } else if (pos.x - knob.right()).abs() <= 6.0 {
                    NavGrab::Right
                } else {
                    NavGrab::Pan
                }
            });
        }
        if r.dragged() {
            if let (Some(mode), Some(pos)) = (self.nav, r.interact_pointer_pos()) {
                let at = ((pos.x - bar.left()) / bar.width() * comp).clamp(0.0, comp);
                self.view = match mode {
                    // 掴んだ所が窓の中心へ来る
                    NavGrab::Pan => TimelineView {
                        start: at - self.view.span * 0.5,
                        span: self.view.span,
                    },
                    // 端を掴んだら、反対の端は動かさない
                    NavGrab::Left => TimelineView {
                        start: at,
                        span: (self.view.start + self.view.span - at).max(MIN_SPAN),
                    },
                    NavGrab::Right => TimelineView {
                        start: self.view.start,
                        span: (at - self.view.start).max(MIN_SPAN),
                    },
                }
                .clamped(comp);
            }
        }
        if r.drag_stopped() {
            self.nav = None;
        }
        p.rect_filled(knob, CornerRadius::same(2), Color32::from_rgb(0x4a, 0x4a, 0x4a));
        p.rect_stroke(
            knob,
            CornerRadius::same(2),
            Stroke::new(
                1.0,
                if r.hovered() || self.nav.is_some() {
                    ACCENT
                } else {
                    Color32::from_rgb(0x66, 0x66, 0x66)
                },
            ),
            StrokeKind::Inside,
        );
    }

    /// 行の名前。**Lab の控えに無ければ Document の台帳を見る。**
    /// 複製で増えた layer は控えに載らないので、これが無いと "?" になる。
    fn name(&self, layer: LayerId) -> &str {
        self.names
            .get(&layer)
            .map(String::as_str)
            .or_else(|| self.document.layers.display_name(layer))
            .unwrap_or("?")
    }
}

impl eframe::App for Lab {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();

        // **編集の出どころを問わない。** 自分が書いたときも、Browser が別の場所で
        // シェイプを置いたときも、`revision` が進んでいれば次のフレームで拾う
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);

        // **毎フレーム、実Documentから行を作り直す。** 行のキャッシュを持たない
        let visible = rows(&self.document, &self.fold);

        let full = ui.available_rect_before_wrap();
        let p = ui.painter().clone();
        p.rect_filled(full, CornerRadius::ZERO, BG);

        // ---- ヘッダ ----
        let head = Rect::from_min_size(full.min, Vec2::new(full.width(), HEAD_H));
        p.rect_filled(head, CornerRadius::ZERO, HEAD_BG);
        p.text(
            egui::pos2(head.left() + 10.0, head.center().y),
            Align2::LEFT_CENTER,
            "Timeline",
            FontId::proportional(15.0),
            INK,
        );
        p.text(
            egui::pos2(head.left() + 92.0, head.center().y),
            Align2::LEFT_CENTER,
            format!("{} rows", visible.len()),
            FontId::proportional(9.0),
            DIM,
        );
        p.text(
            egui::pos2(head.right() - 10.0, head.center().y),
            Align2::RIGHT_CENTER,
            &self.status,
            FontId::proportional(10.0),
            ACCENT,
        );

        // ---- ルーラ ----
        let ruler = Rect::from_min_size(
            egui::pos2(full.left(), head.bottom()),
            Vec2::new(full.width(), RULER_H),
        );
        let track_left = full.left() + RAIL_W;
        let track_w = (full.right() - track_left).max(1.0);
        p.rect_filled(ruler, CornerRadius::ZERO, Color32::from_rgb(0x2a, 0x2a, 0x2a));
        // **目盛は時刻に貼り付く。** ルーラも方眼もこの2本の列から引く
        let fps = self.document.composition.fps;
        let step = tick_step(self.view.span, fps);
        let ticks = ticks(self.view, fps);
        let minor = minor_step(step, fps);
        let minor_ticks: Vec<f32> = minor
            .map(|s| ticks_every(self.view, s))
            .unwrap_or_default();
        let ruler_clip = p.with_clip_rect(Rect::from_min_max(
            egui::pos2(track_left, ruler.top()),
            ruler.max,
        ));
        // 細目盛は数字を持たない。**下から短く出す** — 数字の密度を上げずに
        // 「あいだがどれだけか」を数えられるようにする
        for t in &minor_ticks {
            let x = self.view.time_to_x(*t, track_left, track_w);
            ruler_clip.line_segment(
                [
                    egui::pos2(x, ruler.bottom() - 5.0),
                    egui::pos2(x, ruler.bottom()),
                ],
                Stroke::new(1.0, Color32::from_rgb(0x3a, 0x3a, 0x3a)),
            );
        }
        for t in &ticks {
            let x = self.view.time_to_x(*t, track_left, track_w);
            ruler_clip.line_segment(
                [egui::pos2(x, ruler.top()), egui::pos2(x, ruler.bottom())],
                Stroke::new(1.0, Color32::from_rgb(0x55, 0x55, 0x55)),
            );
            ruler_clip.text(
                egui::pos2(x + 4.0, ruler.bottom() - 6.0),
                Align2::LEFT_BOTTOM,
                tick_label(*t, step),
                FontId::monospace(9.0),
                DIM,
            );
        }
        // いま何秒を見ているか。**窓の広さと粒**もここに出す
        p.text(
            egui::pos2(head.left() + 150.0, head.center().y),
            Align2::LEFT_CENTER,
            format!(
                "{:.2}s  view {:.2}–{:.2}s  grid {}",
                self.playhead,
                self.view.start,
                self.view.start + self.view.span,
                if step >= 1.0 {
                    format!("{step:.0}s")
                } else {
                    format!("{:.0}f", step * fps.as_f64() as f32)
                }
            ),
            FontId::monospace(9.0),
            DIM,
        );
        p.line_segment(
            [ruler.left_bottom(), ruler.right_bottom()],
            Stroke::new(1.0, RULE),
        );

        // ---- 再生 ----
        // **Space で入り切り。** 音も絵もまだ無いので、動くのは playhead だけである。
        // 掴んでいる最中は入り切りしない — ドラッグ中に時間が流れると何が起きたか読めない
        let comp_seconds = self.document.composition.duration.as_seconds_f64() as f32;
        let (space, dt) = ctx.input(|i| (i.key_pressed(egui::Key::Space), i.stable_dt));
        if space && self.drag.is_none() {
            self.playing = !self.playing;
            // 終端で押したら頭から。止まったまま何も起きないのが一番困る
            if self.playing && self.playhead >= comp_seconds - 1e-3 {
                self.playhead = 0.0;
            }
            self.status = if self.playing { "play" } else { "pause" }.to_owned();
        }
        if self.playing {
            // **溜まった時間をまとめて進めない。** 窓が隠れていた分は捨てる
            // (ここだけは指摘の時点より後の修正を残した。窓が他のウィンドウの
            //  後ろにあると eframe が描画を間引き、戻った1フレームの `dt` が
            //  数百msになる — 足すと playhead が数秒ぶん飛ぶ)
            let (at, keep) = advance_playhead(self.playhead, dt.min(MAX_STEP), comp_seconds);
            self.playhead = at;
            if !keep {
                self.playing = false;
                self.status = "end".to_owned();
            }
            // **面のほうが流れ、playhead は窓の中央に居続ける。**
            //
            // 2026-08-16: ここは一度「相対位置を保つ」へ変え、さらに DAW の
            // ページ送りへ変えたが、**利用者の指定でこの形へ戻した**。
            // 経緯は docs/reviews/2026-08-16-daw-playhead-follow-prior-art.md の
            // 「撤廃」。違和感の原因は追従ではなく目盛の明暗だった。
            //
            // 頭と終端では `clamped` が窓を止めるので、そこだけは playhead のほうが
            // 動く — 流れる物が無いところで無理に流さない
            self.view = TimelineView {
                start: self.playhead - self.view.span * 0.5,
                span: self.view.span,
            }
            .clamped(comp_seconds);
        }

        // ルーラのスクラブ。**Document は触らない** — playhead は session の状態
        let ruler_track = Rect::from_min_max(egui::pos2(track_left, ruler.top()), ruler.max);
        let scrub = ui.interact(
            ruler_track,
            ui.id().with("ruler"),
            Sense::click_and_drag(),
        );
        if scrub.is_pointer_button_down_on() {
            if let Some(pos) = scrub.interact_pointer_pos() {
                // 掴んだら再生は止まる。**手で動かしているものが勝手に進まない**
                self.playing = false;
                let at = self
                    .view
                    .x_to_time(pos.x, track_left, track_w)
                    .clamp(0.0, comp_seconds);
                // **playhead もフレームに乗る。** 編集の時刻と同じ粒でないと、
                // キーの上に置いたつもりで半端な位置に居ることになる
                self.playhead = seconds_to_time(at, fps)
                    .map(|t| t.as_seconds_f64() as f32)
                    .unwrap_or(at);
                self.status = format!("{:.2}s", self.playhead);
            }
        }

        // ---- 行 ----
        // **行の面は、ルーラの下からナビゲータ帯の上まで。** 縦スクロールはこの中だけ
        let nav_top = (full.bottom() - NAV_H).max(ruler.bottom());
        let rows_view = Rect::from_min_max(
            egui::pos2(full.left(), ruler.bottom()),
            egui::pos2(full.right(), nav_top),
        );
        let content_h = content_height(&visible);
        self.scroll_y = clamp_scroll(self.scroll_y, content_h, rows_view.height());

        // **位置を先に確定させる。** 描く順と、並べ替えの落とし先と、線の位置が
        // 同じ1つの表から出る(3箇所で y を数え直さない)
        let mut layout: Vec<(TimelineRow, f32, f32)> = Vec::with_capacity(visible.len());
        let mut y = rows_view.top() - self.scroll_y;
        for row in &visible {
            let h = match row.kind {
                RowKind::Object => ROW_H,
                RowKind::Property(_) => PROP_H,
            };
            layout.push((*row, y, h));
            y += h;
        }
        // 並べ替えが数える単位は object 行だけ。パラメータ行のあいだへは落とせない
        let objects: Vec<(LayerId, f32, f32)> = layout
            .iter()
            .filter(|(row, _, _)| matches!(row.kind, RowKind::Object))
            .map(|(row, top, h)| (row.layer, *top, *h))
            .collect();
        let object_layers: Vec<LayerId> = objects.iter().map(|(l, _, _)| *l).collect();

        let mut toggles: Vec<(LayerId, bool)> = Vec::new();
        // M / S のクリック。行を回している間は Document を触らず、回し終えてから書く
        let mut flags: Vec<(LayerId, bool)> = Vec::new();
        let mut pick: Option<(LayerId, bool, bool)> = None;
        let mut reorder_started: Option<LayerId> = None;
        let mut reorder_released = false;

        // **面からはみ出した行は描かない。** 1000行でも触るのは見えている分だけ
        let row_p = p.with_clip_rect(rows_view);
        for (row, top, h) in &layout {
            let p = &row_p;
            let (row, h) = (*row, *h);
            let y = *top;
            if y + h < rows_view.top() || y > rows_view.bottom() {
                continue;
            }
            let rect = Rect::from_min_size(egui::pos2(full.left(), y), Vec2::new(full.width(), h));
            let rail = Rect::from_min_size(rect.min, Vec2::new(RAIL_W, h));
            let track = Rect::from_min_max(egui::pos2(track_left, rect.top()), rect.max);

            // 左列
            let cell = match row.kind {
                RowKind::Property(_) => CELL_PROP,
                _ if row.depth > 0 => CELL_CHILD,
                _ => CELL,
            };
            p.rect_filled(rail, CornerRadius::ZERO, cell);
            // 左列のどこを押しても、その行の layer を選ぶ(ボタン類は上に載るので先に取られる)。
            // **同じ場所を掴んで上下へ引くと並べ替え**になる — AE と同じで、
            // 名前の列は「選ぶ」と「並べ替える」の両方の入口である
            if matches!(row.kind, RowKind::Object) {
                let r = ui.interact(
                    rail,
                    ui.id().with(("pick", row.layer)),
                    Sense::click_and_drag(),
                );
                if r.clicked() {
                    let (additive, range) =
                        ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
                    pick = Some((row.layer, additive, range));
                }
                if r.drag_started() {
                    reorder_started = Some(row.layer);
                }
                if r.drag_stopped() {
                    reorder_released = true;
                }
            }
            if self.is_selected(row.layer) {
                // 選択の帯。**行全体ではなく左端の細い帯**(モックの inset 3px と同じ)
                p.rect_filled(
                    Rect::from_min_size(rail.left_top(), Vec2::new(3.0, rail.height())),
                    CornerRadius::ZERO,
                    ACCENT,
                );
            }
            p.line_segment(
                [rail.left_bottom(), rail.right_bottom()],
                Stroke::new(1.0, RULE),
            );
            p.line_segment(
                [rail.right_top(), rail.right_bottom()],
                Stroke::new(1.0, Color32::from_rgb(0x09, 0x09, 0x09)),
            );

            let indent = 8.0 + row.depth as f32 * 14.0;
            let cy = rail.center().y;

            // 子の開閉（三角）— 子を持つ行だけ
            if row.has_children {
                let hit = Rect::from_center_size(
                    egui::pos2(rail.left() + indent + 2.0, cy),
                    Vec2::splat(16.0),
                );
                let r = ui.interact(hit, ui.id().with(("fold", row.layer)), Sense::click());
                p.text(
                    hit.center(),
                    Align2::CENTER_CENTER,
                    if row.children_open { "▾" } else { "▸" },
                    FontId::proportional(11.0),
                    if r.hovered() { ACCENT } else { DIM },
                );
                if r.clicked() {
                    toggles.push((row.layer, true));
                }
            }

            match row.kind {
                RowKind::Object => {
                    let icon = Rect::from_center_size(
                        egui::pos2(rail.left() + indent + 20.0, cy),
                        Vec2::splat(9.0),
                    );
                    p.rect_filled(
                        icon,
                        CornerRadius::same(2),
                        if row.has_children {
                            ACCENT
                        } else {
                            Color32::from_rgb(0x72, 0x92, 0x98)
                        },
                    );
                    p.text(
                        egui::pos2(rail.left() + indent + 32.0, cy),
                        Align2::LEFT_CENTER,
                        self.name(row.layer),
                        FontId::proportional(11.0),
                        INK,
                    );

                    // キー行の開閉（◇/◆）— キーを持つ行だけ
                    let has_keys = !visible_params(&self.document, row.layer).is_empty();
                    if has_keys {
                        let hit = Rect::from_center_size(
                            egui::pos2(rail.right() - 52.0, cy),
                            Vec2::splat(16.0),
                        );
                        let r =
                            ui.interact(hit, ui.id().with(("params", row.layer)), Sense::click());
                        p.text(
                            hit.center(),
                            Align2::CENTER_CENTER,
                            if row.params_open { "◆" } else { "◇" },
                            FontId::proportional(11.0),
                            if row.params_open || r.hovered() {
                                ACCENT
                            } else {
                                DIM
                            },
                        );
                        if r.clicked() {
                            toggles.push((row.layer, false));
                        }
                    }

                    // **押下状態は Document から読む。** ボタン側に状態を持たない
                    let (item_visible, item_solo) =
                        item_flags(&self.document, row.layer).unwrap_or((true, false));
                    for (i, label) in ["M", "S"].iter().enumerate() {
                        let b = Rect::from_center_size(
                            egui::pos2(rail.right() - 30.0 + i as f32 * 18.0, cy),
                            Vec2::splat(16.0),
                        );
                        let is_mute = i == 0;
                        let on = if is_mute { !item_visible } else { item_solo };
                        let r = ui.interact(
                            b,
                            ui.id().with(("flag", row.layer, is_mute)),
                            Sense::click(),
                        );
                        if on {
                            p.rect_filled(
                                b,
                                CornerRadius::ZERO,
                                if is_mute { MUTE_ON } else { SOLO_ON },
                            );
                        }
                        p.rect_stroke(
                            b,
                            CornerRadius::ZERO,
                            Stroke::new(
                                1.0,
                                if r.hovered() {
                                    ACCENT
                                } else {
                                    Color32::from_rgb(0x51, 0x51, 0x51)
                                },
                            ),
                            StrokeKind::Inside,
                        );
                        p.text(
                            b.center(),
                            Align2::CENTER_CENTER,
                            *label,
                            FontId::proportional(9.0),
                            if on {
                                INK
                            } else {
                                Color32::from_rgb(0xaa, 0xaa, 0xaa)
                            },
                        );
                        if r.clicked() {
                            flags.push((row.layer, is_mute));
                        }
                    }
                }
                RowKind::Property(param) => {
                    let chip = Rect::from_center_size(
                        egui::pos2(rail.left() + indent + 10.0, cy),
                        Vec2::new(4.0, 11.0),
                    );
                    p.rect_filled(chip, CornerRadius::same(2), param_color(param));
                    p.text(
                        egui::pos2(rail.left() + indent + 22.0, cy),
                        Align2::LEFT_CENTER,
                        param_label(param),
                        FontId::proportional(10.0),
                        Color32::from_rgb(0xa8, 0xa8, 0xa8),
                    );
                }
            }

            // 時間面。**方眼はルーラと同じ目盛の上に立つ**(細目盛は薄く)
            p.rect_filled(track, CornerRadius::ZERO, TRACK_A);
            // **1区間おきに下地を変える。** 線だけだと区間が全部同じ面に見えて、
            // どこからどこまでが1目盛なのかを目で掴めない。明暗があると
            // 「区間」そのものが図になる(Ableton の Arrangement と同じ作り)。
            // 濃淡は**絶対時刻で決める** — パンしても縞が入れ替わらない
            for t in &ticks {
                if band_is_dark(*t, step) {
                    let x0 = self.view.time_to_x(*t, track_left, track_w);
                    let x1 = self.view.time_to_x(*t + step, track_left, track_w);
                    let band = Rect::from_min_max(
                        egui::pos2(x0.max(track.left()), track.top()),
                        egui::pos2(x1.min(track.right()), track.bottom()),
                    );
                    if band.width() > 0.0 {
                        p.rect_filled(band, CornerRadius::ZERO, TRACK_BAND);
                    }
                }
            }
            for t in &minor_ticks {
                let x = self.view.time_to_x(*t, track_left, track_w);
                if x < track.left() {
                    continue;
                }
                p.line_segment(
                    [egui::pos2(x, track.top()), egui::pos2(x, track.bottom())],
                    Stroke::new(1.0, TRACK_MINOR),
                );
            }
            for t in &ticks {
                let x = self.view.time_to_x(*t, track_left, track_w);
                if x < track.left() {
                    continue;
                }
                p.line_segment(
                    [egui::pos2(x, track.top()), egui::pos2(x, track.bottom())],
                    Stroke::new(1.0, TRACK_B),
                );
            }
            p.line_segment(
                [track.left_bottom(), track.right_bottom()],
                Stroke::new(1.0, RULE),
            );

            // **時間面の描画は track の中だけ。** 左列へはみ出すと M/S を覆う
            let p = p.with_clip_rect(track);
            match row.kind {
                RowKind::Object => {
                    if let Some((start, end)) = clip_span(&self.document, row.layer) {
                        let x0 = self.view.time_to_x(start, track_left, track_w);
                        let x1 = self.view.time_to_x(end, track_left, track_w);
                        let bar = Rect::from_min_max(
                            egui::pos2(x0, track.top() + 3.0),
                            egui::pos2(x1, track.bottom() - 4.0),
                        );
                        let r = ui.interact(
                            bar,
                            ui.id().with(("bar", row.layer)),
                            Sense::click_and_drag(),
                        );
                        let color = if row.has_children {
                            Color32::from_rgb(0x4c, 0x49, 0x3c)
                        } else {
                            Color32::from_rgb(0x65, 0x75, 0x8c)
                        };
                        p.rect_filled(
                            bar,
                            CornerRadius::ZERO,
                            if r.dragged() { ACCENT } else { color },
                        );
                        p.rect_stroke(
                            bar,
                            CornerRadius::ZERO,
                            Stroke::new(1.0, Color32::from_rgb(0x17, 0x17, 0x17)),
                            StrokeKind::Inside,
                        );
                        if r.drag_started() {
                            if let Some(pos) = r.interact_pointer_pos() {
                                // **何を掴んだかは端からの距離で決める。** 6px はモックの
                                // `.trimHandle{width:7px}` に合わせた値
                                let grab = if pos.x - bar.left() <= 6.0 {
                                    Grab::TrimIn { layer: row.layer }
                                } else if bar.right() - pos.x <= 6.0 {
                                    Grab::TrimOut { layer: row.layer }
                                } else {
                                    // **選ばれているものは一緒に動く。** 選ばれていない
                                    // bar を掴んだときは、その1つを選び直してから動かす
                                    if !self.is_selected(row.layer) {
                                        self.selected = vec![row.layer];
                                    }
                                    let roots = self.selection_roots();
                                    begin_move_many(
                                        &self.document,
                                        &roots,
                                        row.layer,
                                        self.view.x_to_time(pos.x, track_left, track_w),
                                    )
                                };
                                let gesture = self.writer.begin_gesture();
                                self.drag_undo_base = self.writer.undo_len();
                                self.drag = Some((grab, gesture));
                            }
                        }
                        if r.dragged() {
                            if let Some(pos) = r.interact_pointer_pos() {
                                let at = self.view.x_to_time(pos.x, track_left, track_w).max(0.0);
                                self.commit_drag(at);
                            }
                        }
                        if r.drag_stopped() {
                            self.drag = None;
                        }
                    }
                }
                RowKind::Property(param) => {
                    // **時刻を動かせるのは Position だけ。** 他は掴んでも動かない
                    let movable = param == ParamRef::Position;
                    for (key, t) in param_keys(&self.document, row.layer, param) {
                        let x = self.view.time_to_x(t, track_left, track_w);
                        let c = egui::pos2(x, track.center().y);
                        let d = 4.0;
                        // 掴む的は菱形の中心から6px。bar の端と同じ寸法
                        let hit = Rect::from_center_size(c, Vec2::splat(12.0));
                        let r = ui.interact(
                            hit,
                            ui.id().with(("key", row.layer, param_label(param), key.get())),
                            Sense::click_and_drag(),
                        );
                        p.add(egui::Shape::convex_polygon(
                            vec![
                                egui::pos2(c.x, c.y - d),
                                egui::pos2(c.x + d, c.y),
                                egui::pos2(c.x, c.y + d),
                                egui::pos2(c.x - d, c.y),
                            ],
                            if movable && (r.dragged() || r.hovered()) {
                                ACCENT
                            } else {
                                KEY_IDLE
                            },
                            Stroke::new(1.0, Color32::from_rgb(0xee, 0xee, 0xee)),
                        ));
                        if r.drag_started() {
                            if !movable {
                                self.status = format!(
                                    "{} {} key: not movable",
                                    self.name(row.layer),
                                    param_label(param)
                                );
                            } else if let Some(pos) = r.interact_pointer_pos() {
                                let gesture = self.writer.begin_gesture();
                                self.drag_undo_base = self.writer.undo_len();
                                self.drag = Some((
                                    Grab::KeyTime {
                                        layer: row.layer,
                                        key,
                                        grab_at: self.view.x_to_time(pos.x, track_left, track_w),
                                        original: t,
                                    },
                                    gesture,
                                ));
                            }
                        }
                        if r.dragged() && movable {
                            if let Some(pos) = r.interact_pointer_pos() {
                                let at = self.view.x_to_time(pos.x, track_left, track_w);
                                self.commit_drag(at.max(0.0));
                            }
                        }
                        if r.drag_stopped() {
                            self.drag = None;
                        }
                    }
                }
            }
        }

        // ---- 並べ替え ----
        // **落とし先は境界で決まる。** どの行の上に居るかではなく、
        // どの行と行のあいだに居るか
        if let Some(layer) = reorder_started {
            if !self.is_selected(layer) {
                self.selected = vec![layer];
            }
            let gesture = self.writer.begin_gesture();
            self.drag_undo_base = self.writer.undo_len();
            self.drag = Some((Grab::Reorder { layer }, gesture));
        }
        if let Some((Grab::Reorder { layer }, _)) = self.drag.clone() {
            self.drop = ctx
                .input(|i| i.pointer.latest_pos())
                .and_then(|pos| {
                    let boundary = boundary_at(&objects, pos.y);
                    let y = boundary_y(&objects, boundary);
                    drop_target(&self.document, &object_layers, boundary, layer)
                        .map(|(parent, index)| DropTarget { parent, index, y })
                });
            if let Some(to) = self.drop {
                // 落とす前に線で見せる。**書くのは離した瞬間だけ**
                row_p.line_segment(
                    [
                        egui::pos2(full.left() + 4.0, to.y),
                        egui::pos2(full.right() - 4.0, to.y),
                    ],
                    Stroke::new(2.0, ACCENT),
                );
            }
            if reorder_released {
                if let Some(to) = self.drop.take() {
                    self.commit_reorder(layer, to);
                }
                self.drag = None;
            }
        }

        // ---- 縦スクロール / 横ズーム / 横パン ----
        // **割り当ては AE / Premiere と同じ。** 素のホイールは縦、Cmd で横ズーム
        let comp = self.document.composition.duration.as_seconds_f64() as f32;
        let (scroll, shift, command, pinch, pointer) = ctx.input(|i| {
            (
                i.smooth_scroll_delta,
                i.modifiers.shift,
                i.modifiers.command,
                i.zoom_delta(),
                i.pointer.latest_pos(),
            )
        });
        if let Some(pos) = pointer.filter(|p| full.contains(*p)) {
            // ズームの起点は時間面の中に留める。左列の上でピンチしても暴れない
            let anchor = self
                .view
                .x_to_time(pos.x.max(track_left), track_left, track_w);
            if (pinch - 1.0).abs() > 1e-3 {
                // ピンチ。**trackpad はこれが本命**
                self.view = self.view.zoom_at(anchor, 1.0 / pinch, comp);
            } else if command && scroll.y != 0.0 {
                // **カーソルの下の時刻は動かない。** それがズームの手触りそのもの
                self.view = self
                    .view
                    .zoom_at(anchor, 0.9_f32.powf(scroll.y / 50.0), comp);
            } else if scroll.x != 0.0 || (shift && scroll.y != 0.0) {
                // 横スクロール、または Shift + ホイール。**ピクセルの移動量を秒へ
                // 直すのも `x_to_time` の仕事**にして、換算をここに書かない
                let dx = if scroll.x != 0.0 { scroll.x } else { scroll.y };
                let seconds = self.view.x_to_time(track_left + dx, track_left, track_w)
                    - self.view.x_to_time(track_left, track_left, track_w);
                self.view = self.view.pan(-seconds, comp);
            } else if scroll.y != 0.0 {
                // 素のホイールは縦。**行が画面より多いときだけ動く**
                self.scroll_y =
                    clamp_scroll(self.scroll_y - scroll.y, content_h, rows_view.height());
            }
        }

        // ---- 縦のつまみ ----
        // 中身が面より高いときだけ出る。**掴んで動かせる**
        if content_h > rows_view.height() {
            let track_rect = Rect::from_min_max(
                egui::pos2(full.right() - SCROLLBAR_W, rows_view.top()),
                rows_view.max,
            );
            let ratio = rows_view.height() / content_h;
            let knob_h = (track_rect.height() * ratio).max(24.0);
            let travel = track_rect.height() - knob_h;
            let at = self.scroll_y / (content_h - rows_view.height());
            let knob = Rect::from_min_size(
                egui::pos2(track_rect.left(), track_rect.top() + travel * at),
                Vec2::new(SCROLLBAR_W, knob_h),
            );
            let r = ui.interact(track_rect, ui.id().with("vscroll"), Sense::click_and_drag());
            if r.dragged() && travel > 0.0 {
                let per_px = (content_h - rows_view.height()) / travel;
                self.scroll_y = clamp_scroll(
                    self.scroll_y + r.drag_delta().y * per_px,
                    content_h,
                    rows_view.height(),
                );
            }
            p.rect_filled(track_rect, CornerRadius::ZERO, Color32::from_rgb(0x22, 0x22, 0x22));
            p.rect_filled(
                knob,
                CornerRadius::same(2),
                if r.hovered() || r.dragged() {
                    ACCENT
                } else {
                    Color32::from_rgb(0x55, 0x55, 0x55)
                },
            );
        }

        // ---- 時間のナビゲータ帯 ----
        self.navigator(ui, &p, full, track_left, comp);

        // playhead は行を描き終えてから、面の上に1本。
        //
        // **時間面の中だけに描く。** 窓の左端より前の時刻に居ると x が
        // レールの中へ入るので、クリップしないと**レイヤー名の列を縦に貫く**。
        // 時刻を持たない列に時刻の線が出るのは、面の意味が壊れて見える
        let time_p = p.with_clip_rect(Rect::from_min_max(
            egui::pos2(track_left, ruler.top()),
            egui::pos2(full.right(), rows_view.bottom()),
        ));
        let playhead_x = self.view.time_to_x(self.playhead, track_left, track_w);
        time_p.line_segment(
            [
                egui::pos2(playhead_x, ruler.top()),
                egui::pos2(playhead_x, rows_view.bottom()),
            ],
            Stroke::new(1.0, Color32::from_rgb(0xe8, 0xe8, 0xe8)),
        );
        time_p.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(playhead_x - 5.0, ruler.top()),
                egui::pos2(playhead_x + 5.0, ruler.top()),
                egui::pos2(playhead_x, ruler.top() + 7.0),
            ],
            Color32::from_rgb(0xe8, 0xe8, 0xe8),
            Stroke::NONE,
        ));

        if let Some((layer, additive, range)) = pick {
            self.select(layer, additive, range, &object_layers);
        }

        for (layer, is_children) in toggles {
            if is_children {
                if self.fold.children_are_open(layer) {
                    self.fold.close_children(layer);
                } else {
                    self.fold.open_children(layer);
                }
            } else if self.fold.params_are_open(layer) {
                self.fold.close_params(layer);
            } else {
                self.fold.open_params(layer);
            }
        }

        for (layer, mute) in flags {
            self.toggle_flag(layer, mute);
        }

        // Undo / Redo。1ドラッグ = 1 GestureId なので、掴んで動かした分がまとめて戻る
        let (undo, redo, escape, duplicate, delete) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Z) && i.modifiers.command && !i.modifiers.shift,
                i.key_pressed(egui::Key::Z) && i.modifiers.command && i.modifiers.shift,
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::D) && i.modifiers.command,
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
            )
        });
        // **掴んでいる最中の Esc は、その gesture ごと取り消す。**
        // 掴んでいないときの Esc は何もしない — Undo の代わりではない
        if escape && self.drag.is_some() {
            self.cancel_drag();
        } else if undo {
            match self.writer.undo() {
                Ok(()) => {
                    refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                    self.status = format!("undo  ({} left)", self.writer.undo_len());
                }
                Err(error) => self.status = format!("undo rejected: {error}"),
            }
        } else if redo {
            match self.writer.redo() {
                Ok(()) => {
                    refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                    self.status = format!("redo  ({} left)", self.writer.redo_len());
                }
                Err(error) => self.status = format!("redo rejected: {error}"),
            }
        }

        // Cmd/Ctrl + D。**選択が無いときは何もしない** — 複製する対象が無い
        if duplicate {
            self.duplicate_selected();
        }

        // Delete / Backspace。**Group は中身ごと消える**(D2 の RemoveTrackItem)。
        // ドラッグ中は効かせない — 掴んだものが消えると gesture の行き先が無くなる
        if delete && self.drag.is_none() {
            self.delete_selected();
        }

        self.frame += 1;
        if let Some(path) = self.shot.clone() {
            if self.frame >= 20 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
            let image = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(image) = image {
                write_bmp(&path, &image);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

/// **外で入った編集を、次のフレームで拾う。**
///
/// Browser が別の場所でシェイプを置いたとき、Timeline はその編集を自分では
/// 知らない。編集の出どころを問わず `writer.revision` だけを見る。
///
/// `snapshot()` は Document を丸ごと clone するので、**毎フレーム呼ばない** —
/// revision が進んだフレームだけ取り直す。取り直したら `true`。
fn refresh_if_stale(
    writer: &DocumentWriter,
    cached: &mut Arc<Document>,
    cached_revision: &mut u64,
) -> bool {
    if writer.revision == *cached_revision {
        return false;
    }
    *cached = writer.snapshot();
    *cached_revision = writer.revision;
    true
}

/// Lab の `ParamRef` を D2 の property セレクタへ。
fn scalar_property(param: ParamRef) -> motolii_doc::ScalarPropertyId {
    use motolii_doc::ScalarPropertyId as S;
    match param {
        ParamRef::Position => S::Position,
        ParamRef::Anchor => S::Anchor,
        ParamRef::Scale => S::Scale,
        ParamRef::Rotation => S::Rotation,
        ParamRef::Opacity => S::Opacity,
    }
}

fn grab_layer(grab: &Grab) -> LayerId {
    match grab {
        Grab::Move { layer, .. }
        | Grab::KeyTime { layer, .. }
        | Grab::TrimIn { layer }
        | Grab::TrimOut { layer }
        | Grab::Reorder { layer } => *layer,
    }
}

/// その layer を動かしたとき、実際に開始時刻が変わる clip を集める。
///
/// **Group 自身は clip ではないので、動くのは子孫の clip である。**
/// モックの「Group barを動かすと子barも同じ差分で追従する」をここで満たす。
fn movable_clips(document: &Document, layer: LayerId) -> Vec<(LayerId, f32, f32)> {
    fn collect(item: &TrackItem, out: &mut Vec<(LayerId, f32, f32)>) {
        match item {
            TrackItem::Clip(clip) => {
                let start = clip.start.as_seconds_f64() as f32;
                out.push((
                    clip.envelope.layer_id,
                    start,
                    start + clip.duration.as_seconds_f64() as f32,
                ));
            }
            TrackItem::Group(group) => {
                for child in &group.children {
                    collect(child, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    if let Some(item) = find_item(document, layer) {
        collect(item, &mut out);
    }
    out
}

/// 秒 → **最寄りのフレーム境界**の時刻。
///
/// 普通のタイムラインはフレーム単位で動く。clip の頭が 2.0334 秒に来ると、
/// 揃うべきキーが揃わなくなる。丸めは `motolii-core` の変換に任せ、
/// **ここで `(x * fps).round()` を自前で書かない**(fps は有理数で、
/// 30000/1001 のような値がある)。
fn seconds_to_time(seconds: f32, fps: Fps) -> Option<RationalTime> {
    let raw = RationalTime::try_new((f64::from(seconds) * 1000.0).round() as i64, 1000).ok()?;
    let frame = raw.try_to_frame_round(fps).ok()?;
    RationalTime::try_from_frame(frame, fps).ok()
}

fn param_label(p: ParamRef) -> &'static str {
    match p {
        ParamRef::Position => "Position",
        ParamRef::Anchor => "Anchor",
        ParamRef::Scale => "Scale",
        ParamRef::Rotation => "Rotation",
        ParamRef::Opacity => "Opacity",
    }
}

fn param_color(p: ParamRef) -> Color32 {
    match p {
        ParamRef::Position => Color32::from_rgb(0xcf, 0x75, 0x6d),
        ParamRef::Scale => Color32::from_rgb(0x75, 0xa9, 0x78),
        ParamRef::Rotation => Color32::from_rgb(0x77, 0x9b, 0xd0),
        ParamRef::Opacity => Color32::from_rgb(0xb1, 0x8e, 0xc0),
        ParamRef::Anchor => Color32::from_rgb(0xe1, 0xb8, 0x66),
    }
}

/// Document を歩いて、その layer の item を探す
fn find_item(document: &Document, layer: LayerId) -> Option<&TrackItem> {
    fn walk(items: &[TrackItem], layer: LayerId) -> Option<&TrackItem> {
        for item in items {
            let env = match item {
                TrackItem::Clip(c) => &c.envelope,
                TrackItem::Group(g) => &g.envelope,
            };
            if env.layer_id == layer {
                return Some(item);
            }
            if let TrackItem::Group(g) = item {
                if let Some(found) = walk(&g.children, layer) {
                    return Some(found);
                }
            }
        }
        None
    }
    document.tracks.iter().find_map(|t| walk(&t.items, layer))
}

fn visible_params(document: &Document, layer: LayerId) -> Vec<ParamRef> {
    find_item(document, layer)
        .map(motolii_ui::timeline_rows::keyed_params)
        .unwrap_or_default()
}

fn clip_span(document: &Document, layer: LayerId) -> Option<(f32, f32)> {
    match find_item(document, layer)? {
        TrackItem::Clip(c) => {
            let start = c.start.as_seconds_f64() as f32;
            Some((start, start + c.duration.as_seconds_f64() as f32))
        }
        // Group は子の範囲を包む
        TrackItem::Group(g) => {
            let mut span: Option<(f32, f32)> = None;
            for child in &g.children {
                if let TrackItem::Clip(c) = child {
                    let s = c.start.as_seconds_f64() as f32;
                    let e = s + c.duration.as_seconds_f64() as f32;
                    span = Some(match span {
                        Some((a, b)) => (a.min(s), b.max(e)),
                        None => (s, e),
                    });
                }
            }
            span
        }
    }
}

/// その layer の `visible` / `solo`。M / S の押下状態はここから読む
fn item_flags(document: &Document, layer: LayerId) -> Option<(bool, bool)> {
    let env = match find_item(document, layer)? {
        TrackItem::Clip(c) => &c.envelope,
        TrackItem::Group(g) => &g.envelope,
    };
    Some((env.visible, env.solo))
}

/// キーを `(KeyframeId, 時刻(秒))` で返す。**掴んだ物を追い続けるには id が要る** —
/// 時刻は編集で変わるが `KeyframeId` は変わらない
fn param_keys(document: &Document, layer: LayerId, param: ParamRef) -> Vec<(KeyframeId, f32)> {
    let Some(item) = find_item(document, layer) else {
        return Vec::new();
    };
    let env = match item {
        TrackItem::Clip(c) => &c.envelope,
        TrackItem::Group(g) => &g.envelope,
    };
    let doc_param = match param {
        ParamRef::Position => &env.transform.position,
        ParamRef::Anchor => &env.transform.anchor,
        ParamRef::Scale => &env.transform.scale,
        ParamRef::Rotation => &env.transform.rotation,
        ParamRef::Opacity => &env.opacity,
    };
    match doc_param {
        DocParam::Keyframes(track) => track
            .keys()
            .iter()
            .map(|k| (k.id, k.t.as_seconds_f64() as f32))
            .collect(),
        _ => Vec::new(),
    }
}

// ---- fixture: Group ひとつ、子3枚、兄弟2枚 ----

fn time(ms: i64) -> RationalTime {
    RationalTime::try_new(ms, 1000).expect("fixture time")
}

fn keys_at(document: &mut Document, seconds: &[f64], v: DocValue) -> DocParam {
    let mut track = DocKeyframeTrack::new();
    for s in seconds {
        let id = KeyframeId::from_raw(document.next_stable_id.allocate().expect("key id"));
        track.insert(DocKeyframe {
            id,
            t: time((s * 1000.0) as i64),
            value: v.clone(),
            interp: Interp::Hold,
        });
    }
    DocParam::Keyframes(track)
}

fn make_clip(
    document: &mut Document,
    name: &str,
    start_s: f64,
    dur_s: f64,
    positions: &[f64],
    scales: &[f64],
) -> (TrackItem, LayerId, String) {
    let asset = document
        .assets
        .allocate(name, "video/mp4", &format!("{name}-hash"))
        .expect("asset");
    let layer = document.layers.allocate(name).expect("layer");
    let mut envelope = ItemEnvelope::new(layer);
    let mut transform = Transform2D::identity();
    if !positions.is_empty() {
        transform.position = keys_at(document, positions, DocValue::Vec2([0.0, 0.0]));
    }
    if !scales.is_empty() {
        transform.scale = keys_at(document, scales, DocValue::Vec2([1.0, 1.0]));
    }
    envelope.transform = transform;
    (
        TrackItem::Clip(Clip {
            envelope,
            start: time((start_s * 1000.0) as i64),
            duration: time((dur_s * 1000.0) as i64),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        }),
        layer,
        name.to_owned(),
    )
}

fn fixture() -> (Document, HashMap<LayerId, String>) {
    let mut document = Document::new_current();
    // ルーラが 0:00〜0:16 なので、composition もそこまで伸ばす
    document.composition.duration = time(16_000);
    let track = document.track_ids.allocate("V1").expect("track");
    let mut names = HashMap::new();

    let (a, la, na) = make_clip(&mut document, "Shared left", 0.6, 6.0, &[1.2, 5.0], &[2.4, 5.8]);
    let (b, lb, nb) = make_clip(&mut document, "Reference text", 5.4, 6.8, &[6.4, 11.2], &[]);
    let (c, lc, nc) = make_clip(&mut document, "Shared right", 11.0, 4.2, &[11.4, 13.1, 14.7], &[]);
    names.insert(la, na);
    names.insert(lb, nb);
    names.insert(lc, nc);

    let group_layer = document.layers.allocate("Title scene").expect("layer");
    names.insert(group_layer, "Title scene".to_owned());
    let mut group_envelope = ItemEnvelope::new(group_layer);
    group_envelope.transform = Transform2D {
        position: keys_at(&mut document, &[2.8, 10.1], DocValue::Vec2([0.0, 0.0])),
        ..Transform2D::identity()
    };
    group_envelope.opacity = keys_at(&mut document, &[1.9, 6.9, 12.5], DocValue::F64(1.0));
    let group = TrackItem::Group(Group {
        envelope: group_envelope,
        children: vec![a, b, c],
    });

    let (bg, lbg, nbg) = make_clip(&mut document, "Background", 0.0, 13.4, &[], &[]);
    let (audio, laudio, naudio) = make_clip(&mut document, "starter-tone.wav", 1.1, 12.5, &[], &[]);
    names.insert(lbg, nbg);
    names.insert(laudio, naudio);

    document.tracks.push(Track {
        id: track,
        items: vec![group, bg, audio],
    });
    document.validate().expect("valid fixture");
    (document, names)
}

fn write_bmp(path: &str, image: &egui::ColorImage) {
    let (w, h) = (image.size[0] as u32, image.size[1] as u32);
    let row_bytes = (w * 3).next_multiple_of(4);
    let mut out = Vec::with_capacity(54 + (row_bytes * h) as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(54 + row_bytes * h).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    for _ in 0..6 {
        out.extend_from_slice(&0u32.to_le_bytes());
    }
    for y in (0..h).rev() {
        let mut written = 0;
        for x in 0..w {
            let c = image.pixels[(y * w + x) as usize];
            out.push(c.b());
            out.push(c.g());
            out.push(c.r());
            written += 3;
        }
        while written < row_bytes {
            out.push(0);
            written += 1;
        }
    }
    std::fs::write(path, out).expect("write bmp");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer_named(names: &HashMap<LayerId, String>, want: &str) -> LayerId {
        *names
            .iter()
            .find(|(_, n)| n.as_str() == want)
            .map(|(l, _)| l)
            .expect("fixture layer")
    }

    /// マウス無しで、ドラッグが出すのと同じ command を通す。
    /// **手で触る前に、編集が Document へ届くことをここで確かめる。**
    #[test]
    fn dragging_a_clip_moves_it_in_the_document_and_undo_puts_it_back() {
        let (doc, names) = fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let fps = writer.snapshot().composition.fps;
        let layer = layer_named(&names, "Background");

        let before = clip_span(&writer.snapshot(), layer).expect("span");
        assert_eq!(before.0, 0.0, "fixture の Background は 0s 始まり");

        let gesture = writer.begin_gesture();
        let command = writer
            .prepare_set_clip_start(layer, seconds_to_time(2.0, fps).expect("time"))
            .expect("prepare")
            .expect("変化があるので command が出る");
        writer.apply_command(gesture, command).expect("apply");

        let after = clip_span(&writer.snapshot(), layer).expect("span");
        assert!((after.0 - 2.0).abs() < 1e-3, "clip が動いた: {after:?}");
        assert_eq!(
            after.1 - after.0,
            before.1 - before.0,
            "move は尺を変えない"
        );

        writer.undo().expect("undo");
        let restored = clip_span(&writer.snapshot(), layer).expect("span");
        assert!((restored.0 - before.0).abs() < 1e-3, "1ドラッグ = 1 Undo");
    }

    /// **Group を掴むと、子が同じ差分で動く。** モックで決めた挙動。
    #[test]
    fn dragging_a_group_moves_every_child_by_the_same_delta() {
        let (doc, names) = fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let fps = writer.snapshot().composition.fps;
        let group = layer_named(&names, "Title scene");

        let targets = movable_clips(&writer.snapshot(), group);
        assert_eq!(targets.len(), 3, "Group の子 clip は3枚");

        // Shared right が 15.2s で終わり、composition は 16s。
        // **塊で動ける余地は 0.8s しかない** — それを超える delta は頭打ちになる
        let delta = 0.5_f32;
        let gesture = writer.begin_gesture();
        for (layer, start, _) in &targets {
            let command = writer
                .prepare_set_clip_start(*layer, seconds_to_time(start + delta, fps).expect("time"))
                .expect("prepare")
                .expect("command");
            writer.apply_command(gesture, command).expect("apply");
        }

        let after = writer.snapshot();
        for (layer, start, _) in &targets {
            let moved = clip_span(&after, *layer).expect("span").0;
            assert!(
                (moved - (start + delta)).abs() < 1e-3,
                "子は同じ差分で動く: {moved} vs {}",
                start + delta
            );
        }

        // Group 行の bar は子を包むので、まとめて動いたように見える
        let group_span = clip_span(&after, group).expect("group span");
        assert!((group_span.0 - (0.6 + delta)).abs() < 1e-3);

        writer.undo().expect("undo");
        let restored = writer.snapshot();
        for (layer, start, _) in &targets {
            assert!((clip_span(&restored, *layer).expect("span").0 - start).abs() < 1e-3);
        }
    }

    /// **clip が動いたら、その Position キーも一緒に動く。**
    /// Scale は時刻を変える `prepare_*` が無いので置いていかれる — それも押さえる。
    #[test]
    fn moving_a_clip_carries_its_position_keys() {
        let mut lab = Lab::new(None);
        let layer = layer_named(&lab.names, "Shared left");

        let start_before = clip_span(&lab.document, layer).expect("span").0;
        let position_before = param_keys(&lab.document, layer, ParamRef::Position);
        let scale_before = param_keys(&lab.document, layer, ParamRef::Scale);
        assert_eq!(position_before.len(), 2, "fixture の Shared left は Position キー2つ");
        assert_eq!(scale_before.len(), 2, "Scale も2つ。こちらも追従する");

        let gesture = lab.writer.begin_gesture();
        lab.drag = Some((begin_move(&lab.document, layer, 3.0), gesture));
        lab.commit_drag(4.0); // +1.0s

        let after = lab.writer.snapshot();
        assert!(
            (clip_span(&after, layer).expect("span").0 - (start_before + 1.0)).abs() < 1e-3,
            "clip が +1.0s 動いた"
        );
        let position_after = param_keys(&after, layer, ParamRef::Position);
        for (key, before) in &position_before {
            let now = position_after
                .iter()
                .find(|(id, _)| id == key)
                .expect("KeyframeId は時刻編集で不変")
                .1;
            assert!(
                (now - (before + 1.0)).abs() < 1e-3,
                "Position キーが clip に追従する: {now} vs {}",
                before + 1.0
            );
        }
        // Scale キーも同じ delta で追従する(2026-08-16 に D2 の入口ができた)。
        // **守るべきは「時刻が動く」ことではなく「KeyframeId が変わらない」こと** —
        // remove+add で代用すると id が変わり、Undo と同一性が壊れる
        let scale_after = param_keys(&after, layer, ParamRef::Scale);
        assert_eq!(
            scale_after.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            scale_before.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            "KeyframeId は時刻編集で不変。remove+add で代用しない"
        );
        for ((_, before), (_, now)) in scale_before.iter().zip(scale_after.iter()) {
            assert!(
                (now - (before + 1.0)).abs() < 1e-3,
                "Scale キーも clip に追従する: {now} vs {}",
                before + 1.0
            );
        }

        lab.writer.undo().expect("undo");
        let restored = lab.writer.snapshot();
        assert!(
            (clip_span(&restored, layer).expect("span").0 - start_before).abs() < 1e-3,
            "1ドラッグ = 1 Undo"
        );
        assert_eq!(
            param_keys(&restored, layer, ParamRef::Position),
            position_before,
            "キーも同じ Undo でまとめて戻る"
        );
    }

    /// **キーを掴んで動かすと、そのキーだけが動く。** clip も他のキーも巻き込まない。
    #[test]
    fn dragging_a_position_key_changes_only_that_key() {
        let mut lab = Lab::new(None);
        let layer = layer_named(&lab.names, "Shared right");

        let before = param_keys(&lab.document, layer, ParamRef::Position);
        assert_eq!(before.len(), 3, "fixture の Shared right は Position キー3つ");
        let start_before = clip_span(&lab.document, layer).expect("span").0;
        let (key, t0) = before[0];

        let gesture = lab.writer.begin_gesture();
        lab.drag = Some((
            Grab::KeyTime {
                layer,
                key,
                grab_at: t0,
                original: t0,
            },
            gesture,
        ));
        lab.commit_drag(t0 + 1.0);

        let after = param_keys(&lab.writer.snapshot(), layer, ParamRef::Position);
        let moved = after.iter().find(|(id, _)| *id == key).expect("key").1;
        assert!(
            (moved - (t0 + 1.0)).abs() < 1e-3,
            "掴んだキーだけ +1.0s: {moved}"
        );
        for (id, t) in &before[1..] {
            let now = after.iter().find(|(i, _)| i == id).expect("key").1;
            assert_eq!(now, *t, "他のキーは動かない");
        }
        assert!(
            (clip_span(&lab.writer.snapshot(), layer).expect("span").0 - start_before).abs() < 1e-3,
            "clip.start は変わらない"
        );
    }

    /// **M / S は Document を書き換える。** 枠と文字だけではない。
    #[test]
    fn muting_a_layer_writes_through_to_the_document() {
        let mut lab = Lab::new(None);
        let layer = layer_named(&lab.names, "Background");
        assert_eq!(
            item_flags(&lab.document, layer),
            Some((true, false)),
            "既定は表示・非solo"
        );

        lab.toggle_flag(layer, true);
        assert_eq!(
            item_flags(&lab.document, layer),
            Some((false, false)),
            "M で visible=false"
        );

        lab.toggle_flag(layer, false);
        assert_eq!(
            item_flags(&lab.document, layer),
            Some((false, true)),
            "S で solo=true"
        );

        lab.writer.undo().expect("undo");
        lab.document = lab.writer.snapshot();
        assert_eq!(
            item_flags(&lab.document, layer),
            Some((false, false)),
            "1クリック = 1 Undo"
        );
        lab.writer.undo().expect("undo");
        lab.document = lab.writer.snapshot();
        assert_eq!(item_flags(&lab.document, layer), Some((true, false)));
    }

    /// **Scale のキーも clip について来る。** 2026-08-16 に D2 側の入口ができるまで
    /// できなかったこと。Position だけが動いて Scale が取り残される状態を潰す。
    #[test]
    fn moving_a_clip_carries_scale_keys_too_not_just_position() {
        let (doc, names) = fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let fps = writer.snapshot().composition.fps;
        let layer = layer_named(&names, "Shared left");

        let scale_before = param_keys(&writer.snapshot(), layer, ParamRef::Scale);
        assert_eq!(scale_before.len(), 2, "fixture の Shared left は Scale キー2つ");

        let delta = 0.5_f32;
        let gesture = writer.begin_gesture();
        // clip 本体
        let start = clip_span(&writer.snapshot(), layer).expect("span").0;
        let command = writer
            .prepare_set_clip_start(layer, seconds_to_time(start + delta, fps).expect("time"))
            .expect("prepare")
            .expect("command");
        writer.apply_command(gesture, command).expect("apply");
        // Scale キー(後ろから出す。同時刻の占有を避ける)
        for (key, t) in scale_before.iter().rev() {
            let command = writer
                .prepare_set_transform_param_key_time(
                    layer,
                    motolii_doc::ScalarPropertyId::Scale,
                    *key,
                    seconds_to_time(t + delta, fps).expect("time"),
                )
                .expect("prepare")
                .expect("command");
            writer.apply_command(gesture, command).expect("apply");
        }

        let after = param_keys(&writer.snapshot(), layer, ParamRef::Scale);
        for ((_, before_t), (_, after_t)) in scale_before.iter().zip(after.iter()) {
            assert!(
                (after_t - (before_t + delta)).abs() < 1e-3,
                "Scale キーが追従する: {after_t} vs {}",
                before_t + delta
            );
        }

        writer.undo().expect("undo");
        let restored = param_keys(&writer.snapshot(), layer, ParamRef::Scale);
        for ((_, before_t), (_, back_t)) in scale_before.iter().zip(restored.iter()) {
            assert!((back_t - before_t).abs() < 1e-3, "1ドラッグ = 1 Undo");
        }
    }

    // ---- 以下は未実装。実装者はこれを通すこと ----

    const COMP: f32 = 16.0;

    fn view() -> TimelineView {
        TimelineView { start: 0.0, span: 16.0 }
    }

    #[test]
    fn zoom_keeps_the_anchor_time_under_the_cursor() {
        let v = view();
        let (left, w) = (100.0_f32, 900.0_f32);
        let anchor = 6.0_f32;
        let x_before = v.time_to_x(anchor, left, w);

        let zoomed = v.zoom_at(anchor, 0.5, COMP);
        let x_after = zoomed.time_to_x(anchor, left, w);

        assert!(
            (x_before - x_after).abs() < 0.5,
            "カーソルの下の時刻は動かない: {x_before} vs {x_after}"
        );
        assert!((zoomed.span - 8.0).abs() < 1e-3, "span は半分になる");
    }

    #[test]
    fn zoom_clamps_at_the_whole_composition_and_at_a_quarter_second() {
        let out = view().zoom_at(8.0, 100.0, COMP);
        assert!((out.span - COMP).abs() < 1e-3, "composition より広く引けない");
        assert!(out.start.abs() < 1e-3);

        let mut deep = view();
        for _ in 0..20 {
            deep = deep.zoom_at(8.0, 0.5, COMP);
        }
        assert!((deep.span - MIN_SPAN).abs() < 1e-3, "0.25秒より寄れない");
    }

    #[test]
    fn pan_cannot_scroll_before_zero_or_past_the_end() {
        let v = view().zoom_at(8.0, 0.25, COMP); // span 4s
        assert!((v.span - 4.0).abs() < 1e-3);

        let left = v.pan(-999.0, COMP);
        assert!(left.start.abs() < 1e-3, "0より前へは行かない");

        let right = v.pan(999.0, COMP);
        assert!(
            (right.start - (COMP - 4.0)).abs() < 1e-3,
            "終端より後ろは見せない: {}",
            right.start
        );
    }

    #[test]
    fn x_to_time_is_the_inverse_of_time_to_x() {
        let v = TimelineView { start: 3.5, span: 4.0 };
        let (left, w) = (196.0_f32, 800.0_f32);
        for t in [3.5_f32, 4.0, 5.25, 7.5] {
            let back = v.x_to_time(v.time_to_x(t, left, w), left, w);
            assert!((back - t).abs() < 1e-3, "{t} -> {back}");
        }
    }

    #[test]
    fn a_document_edit_made_elsewhere_shows_up_on_the_next_frame() {
        // Browser がシェイプを置いたときに Timeline がすぐ出す、の最小形。
        // **Lab が自分で編集していないのに、行が増えること**を見る
        let (doc, names) = fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");

        let mut cached = writer.snapshot();
        let mut cached_revision = writer.revision;
        let rows_before = rows(&cached, &TimelineFoldState::default()).len();

        // Lab の外で編集する(ここでは M を落とすだけ。行数は変えない編集でも revision は進む)
        let layer = layer_named(&names, "Background");
        let gesture = writer.begin_gesture();
        let command = writer
            .prepare_set_item_visible(layer, false)
            .expect("prepare")
            .expect("command");
        writer.apply_command(gesture, command).expect("apply");

        assert_ne!(writer.revision, cached_revision, "編集で revision が進む");

        // 次のフレームで拾う
        let refreshed = refresh_if_stale(&writer, &mut cached, &mut cached_revision);
        assert!(refreshed, "revision が変わったら取り直す");
        assert_eq!(
            rows(&cached, &TimelineFoldState::default()).len(),
            rows_before
        );
        assert!(
            !item_flags(&cached, layer).expect("flags").0,
            "外からの編集が cached snapshot へ反映されている"
        );

        // 変化が無ければ取り直さない(毎フレーム clone しない)
        assert!(!refresh_if_stale(&writer, &mut cached, &mut cached_revision));
    }

    /// **Esc は掴む前へ戻す。** ドラッグは毎フレーム出し直すが、1ドラッグ =
    /// 1 `GestureId` なので、undo 1回でその gesture の編集がまとめて消える。
    #[test]
    fn escape_during_a_drag_restores_the_original_start() {
        let mut lab = Lab::new(None);
        let layer = layer_named(&lab.names, "Background");
        let before = clip_span(&lab.document, layer).expect("span").0;

        let gesture = lab.writer.begin_gesture();
        lab.drag_undo_base = lab.writer.undo_len();
        lab.drag = Some((begin_move(&lab.document, layer, 1.0), gesture));
        // 実際のドラッグと同じく、動かしている間に何度も出す
        lab.commit_drag(2.0);
        lab.commit_drag(3.0);
        assert!(
            (clip_span(&lab.document, layer).expect("span").0 - (before + 2.0)).abs() < 1e-3,
            "掴んでいる間は動いている: {:?}",
            clip_span(&lab.document, layer)
        );

        lab.cancel_drag();

        assert!(lab.drag.is_none(), "取り消したら、もう掴んでいない");
        assert!(
            (clip_span(&lab.document, layer).expect("span").0 - before).abs() < 1e-3,
            "Esc で掴む前の開始時刻へ戻る: {:?}",
            clip_span(&lab.document, layer)
        );
        assert_eq!(
            lab.writer.undo_len(),
            lab.drag_undo_base,
            "取り消した gesture は履歴に残らない"
        );
        assert!(
            lab.status.contains("cancelled"),
            "status に出す: {}",
            lab.status
        );
    }

    /// **複製の深さは D2 が持っている。** Group を1つ渡すだけで、子3枚が
    /// 新しい `LayerId` を持って一緒に来る。Lab 側で子を辿って複製し直さない。
    #[test]
    fn duplicating_a_group_copies_its_children_with_fresh_ids() {
        let mut lab = Lab::new(None);
        let group = layer_named(&lab.names, "Title scene");

        let items_before = lab.document.tracks[0].items.len();
        let children_before: Vec<LayerId> = movable_clips(&lab.document, group)
            .into_iter()
            .map(|(layer, _, _)| layer)
            .collect();
        assert_eq!(children_before.len(), 3, "fixture の Title scene は子3枚");

        lab.selected = vec![group];
        lab.duplicate_selected();

        // (a) TrackItem が1つ増えている
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before + 1,
            "複製が1つ増える: {}",
            lab.status
        );
        assert!(
            lab.status.contains("Title scene"),
            "status に複製した名前を出す: {}",
            lab.status
        );

        let copy = lab.document.tracks[0]
            .items
            .iter()
            .filter_map(|item| match item {
                TrackItem::Group(g) => Some(g.envelope.layer_id),
                _ => None,
            })
            .find(|layer| *layer != group)
            .expect("複製された Group");
        let children_after: Vec<LayerId> = movable_clips(&lab.document, copy)
            .into_iter()
            .map(|(layer, _, _)| layer)
            .collect();

        // (c) 子の枚数は3枚のまま。D2 が再帰して写している
        assert_eq!(children_after.len(), 3, "子も一緒に複製される");
        // (b) 複製された子の LayerId は元と全部違う
        for layer in &children_after {
            assert!(
                !children_before.contains(layer),
                "複製された子は新しい LayerId: {layer:?} が元と重なる"
            );
        }

        // (d) Undo で元へ戻る。1複製 = 1 GestureId
        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "1複製 = 1 Undo"
        );
        assert_eq!(
            movable_clips(&lab.document, group)
                .into_iter()
                .map(|(layer, _, _)| layer)
                .collect::<Vec<_>>(),
            children_before,
            "元の Group は触られていない"
        );
    }

    /// **ドラッグの結果はフレーム境界に乗る。** 半端な位置に置けると、
    /// 揃うべきキーが揃わなくなる。
    #[test]
    fn dragging_lands_on_a_frame_boundary() {
        let (doc, names) = fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let fps = writer.snapshot().composition.fps;
        let layer = layer_named(&names, "Background");

        // フレームの途中に相当する秒数(30fps なら 1/30 = 0.0333.. の非整数倍)
        let ragged = 2.0334_f32;
        let snapped = seconds_to_time(ragged, fps).expect("time");

        let frame = snapped.try_to_frame_round(fps).expect("frame");
        assert_eq!(
            snapped,
            RationalTime::try_from_frame(frame, fps).expect("time"),
            "丸めた時刻はフレーム境界と往復で一致する"
        );

        // 実際に適用しても境界のまま
        let gesture = writer.begin_gesture();
        let command = writer
            .prepare_set_clip_start(layer, snapped)
            .expect("prepare")
            .expect("command");
        writer.apply_command(gesture, command).expect("apply");

        let after = clip_span(&writer.snapshot(), layer).expect("span").0;
        let back = seconds_to_time(after, fps).expect("time");
        assert_eq!(back, snapped, "適用後も境界に乗ったまま: {after}");
    }

    // ---- 見えている行の並び。並べ替えと範囲選択が数える単位 ----
    fn objects_of(lab: &Lab) -> Vec<LayerId> {
        rows(&lab.document, &lab.fold)
            .into_iter()
            .filter(|r| matches!(r.kind, RowKind::Object))
            .map(|r| r.layer)
            .collect()
    }

    fn location(lab: &Lab, layer: LayerId) -> (ParentLocator, usize) {
        let (parent, index, _) = find_item_location(&lab.document, layer).expect("location");
        (parent, index)
    }

    /// **面より短い中身はスクロールしない。下は最後の行で止まる。**
    #[test]
    fn scrolling_stops_at_the_top_and_at_the_last_row() {
        // 中身(200) が面(300) より低い: どちらへ回しても 0
        assert_eq!(clamp_scroll(-40.0, 200.0, 300.0), 0.0);
        assert_eq!(clamp_scroll(80.0, 200.0, 300.0), 0.0);
        // 中身(500) が面(300) より高い: 下限は 0、上限は差の 200
        assert_eq!(clamp_scroll(-1.0, 500.0, 300.0), 0.0);
        assert_eq!(clamp_scroll(120.0, 500.0, 300.0), 120.0);
        assert_eq!(clamp_scroll(999.0, 500.0, 300.0), 200.0);

        // 行の合計高は object 24 / property 20 の実寸から出る
        let lab = Lab::new(None);
        let visible = rows(&lab.document, &lab.fold);
        let objects = visible
            .iter()
            .filter(|r| matches!(r.kind, RowKind::Object))
            .count() as f32;
        let props = visible.len() as f32 - objects;
        assert_eq!(content_height(&visible), objects * ROW_H + props * PROP_H);
    }

    /// **選ばれているものは、掴んだ1つだけでなく全部が同じ差分で動く。**
    #[test]
    fn moving_one_of_several_selected_clips_moves_them_all() {
        let mut lab = Lab::new(None);
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        let untouched = layer_named(&lab.names, "Shared left");

        let before = |lab: &Lab, l| clip_span(&lab.document, l).expect("span").0;
        let (bg0, tone0, other0) = (
            before(&lab, background),
            before(&lab, tone),
            before(&lab, untouched),
        );

        lab.selected = vec![background, tone];
        let gesture = lab.writer.begin_gesture();
        let roots = lab.selection_roots();
        lab.drag = Some((
            begin_move_many(&lab.document, &roots, background, 3.0),
            gesture,
        ));
        lab.commit_drag(3.5); // +0.5s

        assert!((before(&lab, background) - (bg0 + 0.5)).abs() < 1e-3, "掴んだほうが動く");
        assert!((before(&lab, tone) - (tone0 + 0.5)).abs() < 1e-3, "**もう一方も同じ差分で動く**");
        assert!((before(&lab, untouched) - other0).abs() < 1e-3, "選んでいないものは動かない");

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert!((before(&lab, background) - bg0).abs() < 1e-3, "1ドラッグ = 1 Undo");
        assert!((before(&lab, tone) - tone0).abs() < 1e-3);
    }

    /// 親 Group と子を同時に選んでも、**動くのは1回分の差分**である。
    #[test]
    fn selecting_a_group_and_its_child_moves_the_child_once() {
        let mut lab = Lab::new(None);
        let group = layer_named(&lab.names, "Title scene");
        let child = layer_named(&lab.names, "Shared left");
        let start0 = clip_span(&lab.document, child).expect("span").0;

        lab.selected = vec![group, child];
        assert_eq!(
            lab.selection_roots(),
            vec![group],
            "子は親の子孫なので構造操作からは外れる"
        );

        let roots = lab.selection_roots();
        let Grab::Move { targets, .. } = begin_move_many(&lab.document, &roots, group, 0.0) else {
            panic!("Move を掴んだはず");
        };
        assert_eq!(targets.len(), 3, "同じ clip を2度数えない");

        let gesture = lab.writer.begin_gesture();
        lab.drag = Some((
            begin_move_many(&lab.document, &roots, group, 1.0),
            gesture,
        ));
        lab.commit_drag(1.3); // +0.3s
        assert!(
            (clip_span(&lab.document, child).expect("span").0 - (start0 + 0.3)).abs() < 1e-3,
            "子は 0.3s だけ動く(0.6s ではない)"
        );
    }

    /// **Shift クリックは、見えている行の上で範囲を採る。**
    #[test]
    fn shift_click_selects_the_range_between_two_rows() {
        let mut lab = Lab::new(None);
        let objects = objects_of(&lab);
        assert!(objects.len() >= 4, "fixture の行数");

        lab.select(objects[0], false, false, &objects);
        assert_eq!(lab.selected, vec![objects[0]]);

        lab.select(objects[2], false, true, &objects);
        assert_eq!(lab.selected.len(), 3, "0..2 の3行");
        for layer in &objects[0..=2] {
            assert!(lab.is_selected(*layer));
        }
        assert_eq!(
            lab.selected.last(),
            Some(&objects[0]),
            "起点は末尾に残る。続けて Shift を押しても基準が動かない"
        );

        // Cmd は足し引き
        lab.select(objects[3], true, false, &objects);
        assert!(lab.is_selected(objects[3]));
        lab.select(objects[3], true, false, &objects);
        assert!(!lab.is_selected(objects[3]), "同じ行をもう一度 Cmd クリックで外れる");

        // 素のクリックは1つに戻す
        lab.select(objects[1], false, false, &objects);
        assert_eq!(lab.selected, vec![objects[1]]);
    }

    /// **行を上へ落とすと、Document の並びが変わる。** 時刻は変わらない。
    #[test]
    fn dropping_a_row_above_another_reorders_the_document() {
        let mut lab = Lab::new(None);
        let objects = objects_of(&lab);
        let background = layer_named(&lab.names, "Background");
        let start_before = clip_span(&lab.document, background).expect("span").0;
        assert_eq!(location(&lab, background).1, 1, "はじめは Group の次");

        // 境界0 = いちばん上の行の上
        let (parent, index) =
            drop_target(&lab.document, &objects, 0, background).expect("落とせる");
        lab.commit_reorder(background, DropTarget { parent, index, y: 0.0 });

        assert_eq!(location(&lab, background), (parent, 0), "先頭へ来た");
        assert!(
            (clip_span(&lab.document, background).expect("span").0 - start_before).abs() < 1e-3,
            "**並べ替えは時刻を動かさない**"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(location(&lab, background).1, 1, "1回の Undo で戻る");
    }

    /// 下へ動かすときは**外したあとの位置**になる。1つずれるのを埋める。
    #[test]
    fn dropping_a_row_at_the_end_lands_after_the_last_one() {
        let mut lab = Lab::new(None);
        let objects = objects_of(&lab);
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        assert_eq!(location(&lab, tone).1, 2, "はじめは最後");

        let (parent, index) =
            drop_target(&lab.document, &objects, objects.len(), background).expect("落とせる");
        assert_eq!(index, 2, "3つのうち自分を外したので、末尾は 2 である");
        lab.commit_reorder(background, DropTarget { parent, index, y: 0.0 });

        assert_eq!(location(&lab, background).1, 2, "最後へ来た");
        assert_eq!(location(&lab, tone).1, 1, "追い越された側は1つ上がる");
    }

    /// **開いた Group の中へも落とせる。** 出し入れは同じ1本の command で表す。
    #[test]
    fn dropping_a_row_into_an_open_group_reparents_it() {
        let mut lab = Lab::new(None);
        let objects = objects_of(&lab);
        let group = layer_named(&lab.names, "Title scene");
        let background = layer_named(&lab.names, "Background");
        assert_eq!(objects[0], group, "先頭は Group で、子が開いている");

        // 境界1 = Group の最初の子の上 = 「Group の中の先頭」
        let (parent, index) =
            drop_target(&lab.document, &objects, 1, background).expect("落とせる");
        assert_eq!(parent, ParentLocator::Group(group));
        lab.commit_reorder(background, DropTarget { parent, index, y: 0.0 });

        assert_eq!(location(&lab, background), (ParentLocator::Group(group), 0));
        assert_eq!(
            movable_clips(&lab.document, group).len(),
            4,
            "Group を動かすと、入れたものも一緒に動く"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            location(&lab, background),
            (ParentLocator::Track(lab.document.tracks[0].id), 1),
            "1回の Undo で元の親へ戻る"
        );
    }

    /// **自分の中へは落とせない。** Group を自分の子の中へ入れると木が壊れる。
    #[test]
    fn a_group_cannot_be_dropped_inside_itself() {
        let lab = Lab::new(None);
        let objects = objects_of(&lab);
        let group = layer_named(&lab.names, "Title scene");

        for boundary in 1..=3 {
            assert!(
                drop_target(&lab.document, &objects, boundary, group).is_none(),
                "境界 {boundary} は Group の中なので落とせない"
            );
        }
        // 子の外(いちばん上・いちばん下)へは動かせる
        assert!(drop_target(&lab.document, &objects, 0, group).is_some());
        assert!(drop_target(&lab.document, &objects, objects.len(), group).is_some());
    }

    /// **Delete は Group を中身ごと消し、1回の Undo で全員が戻る。**
    #[test]
    fn deleting_a_group_takes_its_children_and_one_undo_puts_them_back() {
        let mut lab = Lab::new(None);
        let group = layer_named(&lab.names, "Title scene");
        let child = layer_named(&lab.names, "Shared left");
        let items_before = lab.document.tracks[0].items.len();

        lab.selected = vec![group, child]; // 親と子を同時に選んでも壊れない
        lab.delete_selected();

        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before - 1,
            "status: {}",
            lab.status
        );
        assert!(find_item(&lab.document, child).is_none(), "子も消える");
        assert!(lab.selected.is_empty(), "消したものを選んだままにしない");

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(lab.document.tracks[0].items.len(), items_before);
        assert!(
            find_item(&lab.document, child).is_some(),
            "**同じ LayerId で戻る**。id を振り直さない"
        );
        assert_eq!(lab.name(child), "Shared left", "表示名も戻る");
    }

    /// 複数選んで消すのも**1回の Undo 単位**である。
    #[test]
    fn deleting_two_selected_layers_is_one_undo() {
        let mut lab = Lab::new(None);
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        let undo_before = lab.writer.undo_len();

        lab.selected = vec![background, tone];
        lab.delete_selected();
        assert_eq!(lab.document.tracks[0].items.len(), 1);
        assert_eq!(lab.writer.undo_len(), undo_before + 1, "1 gesture にまとまる");

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(lab.document.tracks[0].items.len(), 3, "2枚とも戻る");
    }

    /// 複数選んだ Cmd+D は**選んだ数だけ増え、増えたほうが選ばれる**。
    #[test]
    fn duplicating_two_selected_layers_makes_two_and_selects_them() {
        let mut lab = Lab::new(None);
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        let items_before = lab.document.tracks[0].items.len();

        lab.selected = vec![background, tone];
        lab.duplicate_selected();

        assert_eq!(lab.document.tracks[0].items.len(), items_before + 2);
        assert_eq!(lab.selected.len(), 2, "増えたほうが選ばれている");
        assert!(
            !lab.selected.contains(&background) && !lab.selected.contains(&tone),
            "選ばれているのは元ではなく複製: {:?}",
            lab.selected
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "2枚まとめて1回の Undo で戻る"
        );
    }

    /// **目盛は時刻に貼り付く。** 窓を N 等分すると、パンしても線が動かない。
    #[test]
    fn ticks_are_multiples_of_the_step_not_divisions_of_the_window() {
        let fps = Fps::try_new(30, 1).expect("fps");
        let view = TimelineView { start: 3.3, span: 16.0 };
        let step = tick_step(view.span, fps);
        let list = ticks(view, fps);

        assert!(list.len() >= 6, "窓に何本か入る: {list:?}");
        for t in &list {
            let n = t / step;
            assert!(
                (n - n.round()).abs() < 1e-3,
                "目盛は {step} の倍数である: {t}"
            );
        }
        // パンすると、目盛は同じ倍数の列のまま**時刻ごと動く**
        let panned = ticks(TimelineView { start: 5.3, span: 16.0 }, fps);
        assert_ne!(list, panned, "窓が動けば見える目盛も変わる");
        for t in &panned {
            let n = t / step;
            assert!((n - n.round()).abs() < 1e-3);
        }
        // 0 より前は出さない
        assert!(ticks(TimelineView { start: 0.0, span: 16.0 }, fps)
            .iter()
            .all(|t| *t >= 0.0));
    }

    /// 寄るほど細かい目盛になり、**最後はフレームの倍数**になる。
    #[test]
    fn tick_step_gets_finer_as_you_zoom_in() {
        let fps = Fps::try_new(30, 1).expect("fps");
        let frame = 1.0 / 30.0;

        let wide = tick_step(600.0, fps);
        let mid = tick_step(16.0, fps);
        let close = tick_step(MIN_SPAN, fps);
        assert!(wide > mid && mid > close, "{wide} > {mid} > {close}");
        assert!(close >= frame - 1e-6, "1フレームより細かくはしない: {close}");
        let n = close / frame;
        assert!((n - n.round()).abs() < 1e-3, "寄ったらフレームの倍数: {close}");

        // 文字は間隔より細かい桁を出さない
        assert_eq!(tick_label(64.5, 1.0), "1:04.5");
        assert_eq!(tick_label(64.5, frame), "1:04.50");
    }

    /// **Space の再生は終端で止まる。** 巻き戻さない
    #[test]
    fn playback_advances_and_stops_at_the_end() {
        let (at, playing) = advance_playhead(0.0, 0.5, 16.0);
        assert!((at - 0.5).abs() < 1e-6);
        assert!(playing);

        let (at, playing) = advance_playhead(15.9, 0.5, 16.0);
        assert_eq!(at, 16.0, "終端を越えない");
        assert!(!playing, "終端で止まる");

        // 止まったフレームで dt が来ても進まない
        let (at, playing) = advance_playhead(16.0, 0.016, 16.0);
        assert_eq!(at, 16.0);
        assert!(!playing);
    }

    /// **細目盛は主目盛を割り切り、1フレームより細かくならない。**
    #[test]
    fn minor_ticks_divide_the_labelled_ones_and_stop_at_a_frame() {
        let fps = Fps::try_new(30, 1).expect("fps");
        let frame = 1.0 / 30.0;

        for span in [600.0_f32, 120.0, 16.0, 4.0, 1.0, MIN_SPAN] {
            let major = tick_step(span, fps);
            let Some(minor) = minor_step(major, fps) else {
                assert!(
                    major <= frame * 2.0,
                    "細目盛を消していいのはフレームまで寄ったときだけ: major={major}"
                );
                continue;
            };
            assert!(minor >= frame * 0.999, "1フレームより細かくしない: {minor}");
            assert!(minor < major, "主目盛より細かい: {minor} < {major}");
            let n = major / minor;
            assert!(
                (n - n.round()).abs() < 1e-3,
                "主目盛を割り切る: {major} / {minor}"
            );
        }
    }

    /// **再生中は面が流れ、playhead は窓の中央に居続ける。**
    /// 頭と終端だけは窓が止まり、playhead のほうが窓の中を動く。
    ///
    /// 2026-08-16: 相対位置保持・ページ送りを経て、**利用者の指定でこの形に戻した**。
    #[test]
    fn playback_keeps_the_playhead_centred_and_stops_scrolling_at_both_ends() {
        let comp = 16.0_f32;
        let span = 4.0_f32;
        let centred = |playhead: f32| {
            TimelineView {
                start: playhead - span * 0.5,
                span,
            }
            .clamped(comp)
        };

        // 真ん中あたり: playhead は窓の中央
        let view = centred(8.0);
        assert!((view.start - 6.0).abs() < 1e-4);
        assert!(((8.0 - view.start) / view.span - 0.5).abs() < 1e-4);

        // 頭: 窓は0より前へ行かない = playhead は窓の左寄りに居る
        let view = centred(0.5);
        assert_eq!(view.start, 0.0);
        assert!((0.5 - view.start) / view.span < 0.5);

        // 終端: 窓はcompより後ろへ行かない = playhead は窓の右寄りに居る
        let view = centred(15.5);
        assert!((view.start - (comp - span)).abs() < 1e-4);
        assert!((15.5 - view.start) / view.span > 0.5);
    }

    /// 窓が隠れていた分を**まとめて進めない**。
    #[test]
    fn a_long_frame_does_not_teleport_the_playhead() {
        // 800ms 止まっていた次のフレーム
        let (at, playing) = advance_playhead(2.0, (0.8_f32).min(MAX_STEP), 16.0);
        assert!(playing);
        assert!(
            (at - 2.05).abs() < 1e-6,
            "1フレームで進むのは MAX_STEP まで: {at}"
        );
    }

    /// **縞は時刻に貼り付く。** 窓を動かしても入れ替わらない。
    ///
    /// 窓の中で何本目かで濃淡を決めると、パンした瞬間に全部の縞が反転して
    /// 画面が沸く。0秒から数えた区間の偶奇なら、窓の位置は式に入らない。
    #[test]
    fn the_bands_are_nailed_to_time_not_to_the_window() {
        assert!(!band_is_dark(0.0, 1.0), "0–1s は明");
        assert!(band_is_dark(1.0, 1.0), "1–2s は暗");
        assert!(!band_is_dark(2.0, 1.0));

        // 目盛が2秒粒になっても、偶奇は0秒から数える
        assert!(!band_is_dark(0.0, 2.0));
        assert!(band_is_dark(2.0, 2.0));
        assert!(!band_is_dark(4.0, 2.0));

        // **窓の位置は式に入らない** — どの窓から見ても同じ時刻は同じ濃さ
        let fps = Fps::try_new(30, 1).expect("fps");
        let step = tick_step(16.0, fps);
        let shade = band_is_dark(4.0, step);
        for start in [0.0_f32, 0.7, 3.3, 9.9] {
            let view = TimelineView { start, span: 16.0 };
            if let Some(t) = ticks(view, fps).into_iter().find(|t| (*t - 4.0).abs() < 1e-3) {
                assert_eq!(band_is_dark(t, step), shade, "start={start} で反転した");
            }
        }
    }
}
