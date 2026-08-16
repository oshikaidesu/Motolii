//! egui Timeline の Lab。**実際の Document を、実際の行モデルで、手で触る。**
//!
//! ここには偽のTimeline状態を置かない。行は `timeline_rows::rows()` が
//! 実 Document から毎フレーム作ったものであり、開閉はその `TimelineFoldState` である。
//! だから**この窓で見えていることは、そのまま製品の行モデルの挙動**である。
//!
//! 触れるもの:
//!   - グループ左端の `▸`/`▾` … 子レイヤーの開閉
//!   - `◇`/`◆`            … そのレイヤーのキーパラメータ行の開閉（子とは独立）
//!   - clip bar をドラッグ … 時間方向へ移動。**Position キーが一緒に動く**
//!
//!   - bar の左右端6px      … トリム（in / out）
//!   - Position 行の菱形    … 掴んでキーの時刻を変える
//!   - `M` / `S`           … mute（`visible`）/ solo の反転
//!   - `Cmd/Ctrl + Z` / `Shift+Z` … Undo / Redo
//!   - `Cmd/Ctrl + D`      … 選択中の layer を複製
//!
//! **キーの時刻を動かせるのは Position だけである。** `DocumentWriter` に
//! Scale/Rotation/Anchor/Opacity のキー時刻を変える `prepare_*` は無い。
//! remove + add で代用すると `KeyframeId` が変わり、意味論が変わってしまうので
//! やらない。clip を動かしたとき Position 以外のキーが取り残されることは
//! status 行に `N not movable` として出す。
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
    Clip, ClipSource, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Document, DocumentWriter,
    GestureId, Group, ItemEnvelope, KeyframeId, LayerId, Track, TrackItem, Transform2D,
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
}

/// clip / group を掴んだ瞬間の状態を採る。
///
/// **動くもの(clip の開始時刻)と、追従するもの(その clip の Position キー)を
/// ここで一度に確定させる。** ドラッグ中は毎フレーム絶対値で出し直すので、
/// 掴んだ瞬間の値が要る。
fn begin_move(document: &Document, layer: LayerId, grab_at: f32) -> Grab {
    let targets = movable_clips(document, layer);
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
    /// 選択中の layer。**Project session が持つ種類の状態**で、Document には入れない
    selected: Option<LayerId>,
    /// 見えている時間の窓。同上
    view: TimelineView,
    /// playhead(秒)。同上
    playhead: f32,
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
            selected: None,
            view: TimelineView {
                start: 0.0,
                span: TIMELINE_SECONDS,
            },
            playhead: 0.0,
            status: "arrow=children  diamond=key rows  drag=move  edges=trim  Cmd+Z=undo".to_owned(),
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

    /// 選択中の layer を丸ごと複製する。**1複製 = 1 `GestureId` = 1 Undo 単位**
    ///
    /// **深いところは D2 がやる。** `prepare_duplicate_track_item` は Group の
    /// 子も、シェイプの中の入れ子(`VectorContent::Group`)も再帰して写し、
    /// LayerId / KeyframeId / EffectId を全部新しく振り直す。Lab が子を辿って
    /// 複製し直すと、その再写像を二重にしてしまう — **ここでは source を1つ渡すだけ**。
    fn duplicate_selected(&mut self) {
        let Some(layer) = self.selected else {
            self.status = "nothing selected".to_owned();
            return;
        };
        let name = self.name(layer).to_owned();
        let command = match self.writer.prepare_duplicate_track_item(layer) {
            Ok(command) => command,
            Err(error) => {
                self.status = format!("{name} rejected: {error}");
                return;
            }
        };
        let gesture = self.writer.begin_gesture();
        match self.writer.apply_command(gesture, command) {
            Ok(()) => {
                refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                self.status = format!("duplicated {name}  undo {}", self.writer.undo_len());
            }
            Err(error) => self.status = format!("{name} rejected: {error}"),
        }
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
        for i in 0..=8 {
            // **目盛は窓を8等分した時刻である。** 寄れば 0:06.0 の隣が 0:06.5 になる
            let t = self.view.start + self.view.span * i as f32 / 8.0;
            let x = self.view.time_to_x(t, track_left, track_w);
            p.line_segment(
                [egui::pos2(x, ruler.top()), egui::pos2(x, ruler.bottom())],
                Stroke::new(1.0, Color32::from_rgb(0x44, 0x44, 0x44)),
            );
            p.text(
                egui::pos2(x + 4.0, ruler.bottom() - 5.0),
                Align2::LEFT_BOTTOM,
                format!("{}:{:04.1}", (t / 60.0).floor() as i64, t % 60.0),
                FontId::monospace(9.0),
                DIM,
            );
        }
        p.line_segment(
            [ruler.left_bottom(), ruler.right_bottom()],
            Stroke::new(1.0, RULE),
        );

        // ルーラのスクラブ。**Document は触らない** — playhead は session の状態
        let ruler_track = Rect::from_min_max(egui::pos2(track_left, ruler.top()), ruler.max);
        let scrub = ui.interact(
            ruler_track,
            ui.id().with("ruler"),
            Sense::click_and_drag(),
        );
        if scrub.is_pointer_button_down_on() {
            if let Some(pos) = scrub.interact_pointer_pos() {
                let comp = self.document.composition.duration.as_seconds_f64() as f32;
                self.playhead = self
                    .view
                    .x_to_time(pos.x, track_left, track_w)
                    .clamp(0.0, comp);
                self.status = format!("{:.2}s", self.playhead);
            }
        }

        // ---- 行 ----
        let mut y = ruler.bottom();
        let mut toggles: Vec<(LayerId, bool)> = Vec::new();
        // M / S のクリック。行を回している間は Document を触らず、回し終えてから書く
        let mut flags: Vec<(LayerId, bool)> = Vec::new();
        let mut pick: Option<LayerId> = None;

        for row in &visible {
            let h = match row.kind {
                RowKind::Object => ROW_H,
                RowKind::Property(_) => PROP_H,
            };
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
            // 左列のどこを押しても、その行の layer を選ぶ(ボタン類は上に載るので先に取られる)
            if matches!(row.kind, RowKind::Object) {
                let r = ui.interact(rail, ui.id().with(("pick", row.layer)), Sense::click());
                if r.clicked() {
                    pick = Some(row.layer);
                }
            }
            if self.selected == Some(row.layer) {
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

            // 時間面
            p.rect_filled(track, CornerRadius::ZERO, TRACK_A);
            for i in 0..=8 {
                let x = track_left + track_w * i as f32 / 8.0;
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
                                    begin_move(
                                        &self.document,
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

            y += h;
        }

        // ---- 横ズーム / パン ----
        // **時間面の上でだけ効く。** 左列の上でホイールしても窓は動かない
        let rows_bottom = y;
        let surface = Rect::from_min_max(
            egui::pos2(track_left, ruler.top()),
            egui::pos2(full.right(), rows_bottom.max(ruler.bottom())),
        );
        let comp = self.document.composition.duration.as_seconds_f64() as f32;
        let (scroll, shift, pointer) = ctx.input(|i| {
            (
                i.smooth_scroll_delta,
                i.modifiers.shift,
                i.pointer.latest_pos(),
            )
        });
        if let Some(pos) = pointer.filter(|p| surface.contains(*p)) {
            if scroll.x != 0.0 || (shift && scroll.y != 0.0) {
                // 横スクロール、または Shift + ホイール。**ピクセルの移動量を秒へ
                // 直すのも `x_to_time` の仕事**にして、換算をここに書かない
                let dx = if scroll.x != 0.0 { scroll.x } else { scroll.y };
                let seconds = self.view.x_to_time(track_left + dx, track_left, track_w)
                    - self.view.x_to_time(track_left, track_left, track_w);
                self.view = self.view.pan(-seconds, comp);
            } else if scroll.y != 0.0 {
                // **カーソルの下の時刻は動かない。** それがズームの手触りそのもの
                let anchor = self.view.x_to_time(pos.x, track_left, track_w);
                self.view = self
                    .view
                    .zoom_at(anchor, 0.9_f32.powf(scroll.y / 50.0), comp);
            }
        }

        // playhead は行を描き終えてから、面の上に1本
        let playhead_x = self.view.time_to_x(self.playhead, track_left, track_w);
        p.line_segment(
            [
                egui::pos2(playhead_x, ruler.top()),
                egui::pos2(playhead_x, rows_bottom.max(ruler.bottom())),
            ],
            Stroke::new(1.0, Color32::from_rgb(0xe8, 0xe8, 0xe8)),
        );
        p.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(playhead_x - 5.0, ruler.top()),
                egui::pos2(playhead_x + 5.0, ruler.top()),
                egui::pos2(playhead_x, ruler.top() + 7.0),
            ],
            Color32::from_rgb(0xe8, 0xe8, 0xe8),
            Stroke::NONE,
        ));

        if let Some(layer) = pick {
            self.selected = Some(layer);
            self.status = format!("selected {}", self.name(layer));
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
        let (undo, redo, escape, duplicate) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Z) && i.modifiers.command && !i.modifiers.shift,
                i.key_pressed(egui::Key::Z) && i.modifiers.command && i.modifiers.shift,
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::D) && i.modifiers.command,
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
        | Grab::TrimOut { layer } => *layer,
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

        lab.selected = Some(group);
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
}
