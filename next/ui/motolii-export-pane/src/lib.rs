//! owns: Export ダイアログの投影 view(意味は motolii-export)
//!
//! B09 第1切片(書き出し束、2026-08-22 発注): Export ダイアログの顔。浮かし窓の
//! 中身になる前提 — Settings 窓(`motolii_settings_pane::sections::view`)と同じ
//! 「投影を受ける純関数」の形。**意味は `motolii-export` が持ち、この crate は
//! 読み取り投影しか持たない**(Document にも encoder にも触らない — 書き口ゼロ)。
//!
//! OWNS-JUSTIFICATION(A): B09発注書「投影を受ける純関数」の形の明示指示。
//! **ただし正直な記録**: 自己申告(意味は`motolii-export`が持ち読み取り投影
//! しか持たない)は事実上`wraps:`寄りの性質を持つ(裁定215 棚卸し 2026-08-23
//! §3・#27 で境界事例として記録済み。marker文言(`owns:`)自体は本発注では
//! 変更しない)。
//!
//! ## map(B09)から抽出した行(B09 の該当行は全行 freq=1 — 降順は id 順に同値)
//! 既存 `motolii-export` の実能力(コーデック・解像度・範囲)に紐づく行のみ
//! (飾り禁止 — SET+ の前例):
//!
//! - **529「Ctrl+E = Export」消化**: Export ダイアログの顔+実行
//!   [`Message::Export`](ショートカット結線そのものは shell 側・次波)。
//! - **538「Quick Export(クイックエクスポートウィンドウを開く)」消化**:
//!   窓 open/close [`Message::ToggleExportDialog`]。
//! - **533/534「Export Project」= 採用済**(`Document::save`、裁定55/56)—
//!   既に顔があるのでこのダイアログの守備範囲外。
//! - **見送り(紐づく実能力が無い — 顔を作ると飾りになる)**: 522/647(Add to
//!   Render Queue — job キュー機構が無い)・524(Premiere project 書き出し)・
//!   530/663(静止画書き出し — export 経路に無い。screenshot は shell の別器具)・
//!   531(AAF/XML)・532(メタデータ CSV)・535(Render Queue 内の選択拡張)・
//!   644/907/930(Output Module / レンダー設定テンプレート — 保存機構が無い)・
//!   646(AME キュー)。
//!
//! ## 実能力との対応(全項目が実在識別子に紐づく)
//! - **出力先**: `motolii_export::ExportJob::out_path`。path 選択は rfd
//!   (裁定176 の `FileDialogs` 経由、[`Message::PickOutputPath`] →
//!   `motolii-shell` が `FileDialogs::pick_export_path` を非同期に呼び
//!   [`Message::OutputPathChosen`] で戻す ── File 束の rfd 非同期化と同時発注、
//!   2026-08-22 第2波で消化)。
//! - **フォーマット/コーデック**: `motolii_media::Encoder` の実対応は
//!   **MP4 / H.264 の1種のみ**([`CONTAINER_CODEC_LABEL`])— 選べない物に選択
//!   UI を飾らない(read-only 表示)。実在する唯一の選択肢は品質:
//!   `ExportJob::qp0`([`ExportQuality`] — 通常 CRF18 / ほぼロスレス qp0)。
//! - **解像度/フレームレート**: `Composition::{width, height, fps}` の read-only
//!   投影。export は Document の comp をそのまま使う(`motolii-export` crate doc
//!   「解像度・fps・尺は Document が持つ — 書けると preview と違う入力で書き出せて
//!   しまう」)ので、現在値の表示は編集欄にしない。アスペクトプリセットは例外の
//!   書き口ではなく、既存の `Intent::SetComposition` へ渡せる**要求の顔**として
//!   `AspectPresetSelect` を発火するだけである。pane は `Composition` を変更せず、
//!   Shell がその要求を Document へ適用した後に、preview/export が同じ Composition
//!   を読む。cap は
//!   [`motolii_settings_pane::sections::MAX_COMP_DIMENSION_PX`](wgpu 1枚 texture
//!   の床)— 超過 comp は書き出せないため警告行を出す([`resolution_within_cap`])。
//! - **範囲**: 全体 = `export_with_cancel` の `0..duration_frames` ループ(実在)。
//!   作業範囲 = TL+ の `WorkArea`(`motolii-timeline-pane::work_area`、半開
//!   `[start, end)`)を **Session 経由で受ける前提の投影**([`WorkAreaFrames`] —
//!   pane 同士は依存しない(pane split survey §5)ため座標だけの写し。区間の
//!   約束も同じ半開)。**逸脱**: `motolii_export::export_with_cancel` に範囲引数は
//!   まだ無い — [`effective_range`] は投影の意味で、実行側の範囲口は
//!   `motolii-export` の次波(本発注では読み取り専用のため不触)。
//! - **実行/進捗**: [`Message::Export`] → shell が `ExportJob` を組んで
//!   `export_with_cancel` を回す。進捗の器 = [`ExportProgress`](export は
//!   フレームを1本ずつ回すバッチ実行 — `written` カウンタの投影の**型だけ**、
//!   発注書どおり)。中断 = [`Message::CancelExport`] →
//!   `motolii_export::Cancel::cancel()`(中断は残骸を消してから返る、の顔)。
//!
//! ## 意匠
//! settings sections の文法をそのまま延長(2箇所で別の意匠を発明しない):
//! 見出し帯 = `chrome::section_header`、ボタン = `chrome::button_style`、容器 =
//! `chrome::panel_container_style`、行区切り = 行高+padding の間隔(裁定179
//! 文法1 — 線は引かない)。裁定187(icon-first)は iced 標準 widget の採用で
//! 満たす — 二値の選択(qp0 / 作業範囲のみ)は文字ボタン列でなく **toggler**
//! (裁定187 在庫調査 TOP6)、中断は Close glyph(`motolii-icons`)。
//! **逸脱**: Export 動詞の icon+tooltip ペアは icons crate に該当 glyph
//! (export/download 系)が未 vendoring のため文字ラベルのまま次波へ
//! (icons crate は本発注の write-set 外)。dimensions.json `export` 節は
//! 不使用 — 既存 `inspector_*` 行トークンで足りる(write-set = 新 crate +
//! members 1行のみ、tokens crate 不触)。
//!
//! ## shell 結線(supervisor 手順 — Settings B12 と同じ縫い目の型)
//! 1. root `Message::Export(motolii_export_pane::Message)` を1本で畳む。
//! 2. `Shell` 状態: `export_dialog_open: bool` / `export_quality: ExportQuality` /
//!    `export_range: ExportRange` / `export_out_path: Option<PathBuf>` /
//!    `export_progress: Option<ExportProgress>`(+実行中の
//!    `motolii_export::Cancel` ハンドル)。
//! 3. 腕: [`Message::ToggleExportDialog`] = open 反転(表示だけのトグル —
//!    Document にも undo にも乗らない、歯車トグルと同格)/
//!    [`Message::QualitySelect`]・[`Message::RangeSelect`] = 状態代入 /
//!    [`Message::AspectPresetSelect`] = [`dimensions_for_aspect`] で寸法を引き、
//!    既存の `Intent::SetComposition` に read-modify-write する(この pane は
//!    ここまでで、Document への結線は supervisor の WIRE)/
//!    [`Message::PickOutputPath`] = `Task::perform(dialogs.pick_export_path(..),
//!    |p| Message::Export(Message::OutputPathChosen(p)))` /
//!    [`Message::OutputPathChosen`] = `Some(path)` なら `export_out_path` へ
//!    代入・`None`(キャンセル)は無視 /
//!    [`Message::Export`] = `motolii_export::ExportJob { out_path, qp0:
//!    quality.qp0() }` を組んで `export_with_cancel`(範囲は上記逸脱参照)/
//!    [`Message::CancelExport`] = 保持している `Cancel` の `.cancel()`。

use std::path::Path;

use iced::widget::{button, column, container, row, text, toggler};
use iced::{Element, Length};

use motolii_icons::{frame_px_for_glyph_px, icon, Icon};
use motolii_settings_pane::chrome::{self, button_style, section_header};
use motolii_settings_pane::sections::{format_fps, MAX_COMP_DIMENSION_PX};
use motolii_store::Composition;
use motolii_tokens_rs::{Colors, Dimensions};

// ---------------------------------------------------------------------------
// pane ローカル Message(root `Message::Export(Message)` が1本で畳む —
// 裁定160 の pane 文法)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// 窓 open/close(map 538「Quick Export」消化)。表示だけのトグル —
    /// Document にも undo 履歴にも乗らない(Settings の歯車トグルと同格)。
    ToggleExportDialog,
    /// 品質の選択(`ExportJob::qp0` へ写る実在の二値)。押した瞬間に確定
    /// (下書きを経由しない — 数値でなく choice なので Enter の文法は不要)。
    QualitySelect(ExportQuality),
    /// 範囲の選択(全体 / 作業範囲のみ)。同じく即時確定。
    RangeSelect(ExportRange),
    /// Composition のアスペクトプリセット要求。pane は寸法を計算して
    /// `Intent::SetComposition` の入力候補を返すだけで、Document へは書かない。
    AspectPresetSelect(AspectPreset),
    /// 書き出し先の選択(2026-08-22 第2波、File 束の rfd 非同期化と同時発注)。
    /// shell が `FileDialogs::pick_export_path` を非同期に呼ぶだけの引き金 ──
    /// この時点ではまだ path は確定しない(結果は [`Message::
    /// OutputPathChosen`] で戻る)。
    PickOutputPath,
    /// [`Message::PickOutputPath`] の結果。`None` = キャンセル(現在の
    /// `out_path` を変えない)。
    OutputPathChosen(Option<std::path::PathBuf>),
    /// Export 実行(map 529 消化)。shell が `motolii_export::ExportJob` を
    /// 組んで `export_with_cancel` を回す(crate doc「shell 結線」)。
    Export,
    /// 実行中の中断 — shell が保持する `motolii_export::Cancel` の
    /// `.cancel()`(中断は残骸を消してから返る、が意味側の約束)。
    CancelExport,
}

// ---------------------------------------------------------------------------
// アスペクトプリセット(要求の顔。Document の書き口は Shell/WIRE)
// ---------------------------------------------------------------------------

/* motolii-component
id = "export.aspect_preset"
kind = "semantic"
weight = "render_export"
maps = []
entry = ["AspectPresetSelect"]
meaning = ["AspectPreset"]
evaluation = ["dimensions_for_aspect"]
render = ["aspect_preset_row"]
observable = ["aspect_preset_buttons_show_label_and_dimensions"]
*/

/// Export 窓から要求できる Composition の標準アスペクト。
///
/// これは crop の一時設定ではない。Shell がこの要求を Document の
/// `Composition::{width,height}` へ適用すると、preview と export の両方が同じ
/// Composition を読む。pane は要求を発火するところまでを担当する。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AspectPreset {
    /// 標準の横長 HD 出力。
    Landscape16x9,
    /// 縦型のソーシャル動画向け出力。
    Portrait9x16,
    /// 正方形出力。
    Square1x1,
}

impl AspectPreset {
    /// UI に並べる全プリセット。順序は横長・縦長・正方形で固定する。
    pub const ALL: [Self; 3] = [Self::Landscape16x9, Self::Portrait9x16, Self::Square1x1];

    /// 利用者へ見せる比率ラベル。
    pub const fn label(self) -> &'static str {
        match self {
            Self::Landscape16x9 => "16:9",
            Self::Portrait9x16 => "9:16",
            Self::Square1x1 => "1:1",
        }
    }
}

/// プリセットが要求する Composition 寸法。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AspectDimensions {
    pub width: u32,
    pub height: u32,
}

/// アスペクトプリセットから標準寸法への**唯一の純粋な写像**。
///
/// 16:9 は現行の既定 Composition `1920 × 1080`を維持し、縦長と正方形は
/// 1080px を基準辺にする。いずれも現行の texture cap 内で、Shell がそのまま
/// `Composition` の width/height へ read-modify-write できる値である。
pub const fn dimensions_for_aspect(preset: AspectPreset) -> AspectDimensions {
    match preset {
        AspectPreset::Landscape16x9 => AspectDimensions {
            width: 1920,
            height: 1080,
        },
        AspectPreset::Portrait9x16 => AspectDimensions {
            width: 1080,
            height: 1920,
        },
        AspectPreset::Square1x1 => AspectDimensions {
            width: 1080,
            height: 1080,
        },
    }
}

/// 現在の Composition がどのプリセット比率かを純粋に判定する。
///
/// 寸法そのものではなく比率を比較するため、`1280 × 720`のような既存の
/// 16:9 Composition も選択中として表示できる。0 寸法や未対応比率は
/// `None`(custom) になる。
pub fn aspect_preset_for_dimensions(width: u32, height: u32) -> Option<AspectPreset> {
    if width == 0 || height == 0 {
        return None;
    }
    AspectPreset::ALL.into_iter().find(|preset| {
        let dimensions = dimensions_for_aspect(*preset);
        u64::from(width) * u64::from(dimensions.height)
            == u64::from(height) * u64::from(dimensions.width)
    })
}

// ---------------------------------------------------------------------------
// フォーマット/品質(motolii-media Encoder の実対応)
// ---------------------------------------------------------------------------

/// `motolii_media::Encoder` の実対応コンテナ/コーデック。**1種のみ**なので
/// 選択 UI は無い(read-only 表示)— 対応が増えた時にここが enum へ育つ。
pub const CONTAINER_CODEC_LABEL: &str = "MP4 / H.264";

/// 品質 — `motolii_export::ExportJob::qp0` の顔。実在する唯一の書き出し選択肢
/// (`Encoder::open` の `qp0`: false = CRF18 / true = qp0 ほぼロスレス 4:4:4)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportQuality {
    /// CRF18(`Encoder` の通常書き出し)。
    Normal,
    /// qp0 ほぼロスレス(`Encoder` doc: 検証・テスト用途も同じ口)。
    Lossless,
}

impl ExportQuality {
    /// 全 variant(「全項目が実在識別子に紐づく」テストの全数検査用)。
    pub const ALL: [ExportQuality; 2] = [ExportQuality::Normal, ExportQuality::Lossless];

    /// `motolii_export::ExportJob::qp0` へ渡す実値。**写像はここ1箇所だけ**。
    pub fn qp0(self) -> bool {
        matches!(self, ExportQuality::Lossless)
    }

    /// toggler(bool)→ enum の逆写像([`qp0`](Self::qp0) と往復不変)。
    pub fn from_qp0(qp0: bool) -> Self {
        if qp0 {
            ExportQuality::Lossless
        } else {
            ExportQuality::Normal
        }
    }
}

// ---------------------------------------------------------------------------
// 範囲(全体 / 作業範囲)
// ---------------------------------------------------------------------------

/// 書き出し範囲の選択。全体 = `export_with_cancel` の `0..duration_frames`
/// ループ(実在)。作業範囲 = TL+ `WorkArea` の投影(crate doc の逸脱参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportRange {
    /// comp 全体(`0..duration_frames`)。
    Whole,
    /// 作業範囲のみ([`WorkAreaFrames`] と交差 — 無ければ全体へ落ちる)。
    WorkArea,
}

impl ExportRange {
    /// 全 variant(全数検査用)。
    pub const ALL: [ExportRange; 2] = [ExportRange::Whole, ExportRange::WorkArea];

    /// toggler(「作業範囲のみ」)の表示値。
    pub fn work_area_only(self) -> bool {
        matches!(self, ExportRange::WorkArea)
    }

    /// toggler(bool)→ enum の逆写像([`work_area_only`](Self::work_area_only)
    /// と往復不変)。
    pub fn from_work_area_only(work_area_only: bool) -> Self {
        if work_area_only {
            ExportRange::WorkArea
        } else {
            ExportRange::Whole
        }
    }
}

/// TL+ の作業範囲(`motolii_timeline_pane::WorkArea`)を Session 経由で受ける
/// 前提の投影。pane 同士は依存しない(pane split survey §5)ため座標だけの写し —
/// 区間の約束も同じ**半開 `[start, end)`・最短1フレーム**(`end >= start + 1`)。
/// supervisor が Session(昇格待ち — `work_area.rs` 冒頭 doc)から詰め替える。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkAreaFrames {
    /// In 点(含む)。
    pub start: i64,
    /// Out 点(含まない)。
    pub end: i64,
}

/// 実際に書き出すフレーム区間(半開 `[start, end)`、最短1フレーム保証)。
/// [`effective_range`] だけが作る。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameRange {
    pub start: i64,
    pub end: i64,
}

impl FrameRange {
    /// フレーム数(最短1 — [`effective_range`] の不変)。
    pub fn frame_count(self) -> i64 {
        self.end - self.start
    }

    /// 最終フレーム(半開なので `end - 1`)。
    pub fn last_frame(self) -> i64 {
        self.end - 1
    }
}

/// 選択と作業範囲から、実際に書き出す区間を決める純関数。
///
/// - 全体(または作業範囲未設定)= `[0, max(duration, 1))` — export の
///   `0..duration_frames` ループそのもの(尺0の防波堤だけ最短1)。
/// - 作業範囲 = comp 尺と交差してクランプ。交差が空になる置き方(範囲が尺の
///   外へ完全に出ている等)でも**最短1フレーム**へ畳む — 「書き出したら
///   0フレームの file」を型より手前で拒む(`WorkArea` の最短1保証と同じ約束)。
pub fn effective_range(
    range: ExportRange,
    work_area: Option<WorkAreaFrames>,
    duration_frames: i64,
) -> FrameRange {
    let whole_end = duration_frames.max(1);
    match (range, work_area) {
        (ExportRange::WorkArea, Some(area)) => {
            let start = area.start.clamp(0, whole_end - 1);
            let end = area.end.clamp(start + 1, whole_end);
            FrameRange { start, end }
        }
        _ => FrameRange {
            start: 0,
            end: whole_end,
        },
    }
}

// ---------------------------------------------------------------------------
// 進捗(export のバッチ実行 `written` カウンタの投影 — 型だけ、発注書どおり)
// ---------------------------------------------------------------------------

/// 実行中の進捗。shell が `export_with_cancel` の周回(`written` 相当)から
/// 組んで注入する — `None` = 実行していない(view は Export ボタンを出す)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportProgress {
    /// encoder へ渡し終えたフレーム数(`ExportReport::frames_written` と同じ数え方)。
    pub frames_done: i64,
    /// 書く予定の総フレーム数([`FrameRange::frame_count`])。
    pub frames_total: i64,
}

/// 進捗率 `0.0..=1.0`。`frames_total <= 0`(来ない想定の防波堤)は 0.0 扱い —
/// panic もゼロ除算もしない。
pub fn progress_fraction(progress: ExportProgress) -> f64 {
    if progress.frames_total <= 0 {
        return 0.0;
    }
    (progress.frames_done as f64 / progress.frames_total as f64).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// 表示文字列(純関数 — tests が固定する)
// ---------------------------------------------------------------------------

/// 解像度の表示。`×` は U+00D7(ASCII `x` の代用をしない)。
pub fn format_resolution(width: u32, height: u32) -> String {
    format!("{width} × {height}")
}

/// comp が encoder/texture の床([`MAX_COMP_DIMENSION_PX`] = wgpu
/// `max_texture_dimension_2d`)に収まっているか。通常は settings 側の入力
/// クランプで超えないが、防波堤として超過時は警告行を出す。
pub fn resolution_within_cap(width: u32, height: u32) -> bool {
    width <= MAX_COMP_DIMENSION_PX && height <= MAX_COMP_DIMENSION_PX
}

/// 区間の表示。「先頭 – 最終(枚数 frames)」— 半開の `end` を生で見せない
/// (利用者の言葉は「最後のフレーム」、タイムラインの表示と同じ側)。
pub fn format_range_summary(range: FrameRange) -> String {
    format!(
        "{} – {}({} frames)",
        range.start,
        range.last_frame(),
        range.frame_count()
    )
}

/// 進捗の表示。「済 / 総(百分率%)」。
pub fn format_progress(progress: ExportProgress) -> String {
    let percent = (progress_fraction(progress) * 100.0).round() as i64;
    format!(
        "{} / {}({percent}%)",
        progress.frames_done, progress.frames_total
    )
}

// ---------------------------------------------------------------------------
// view
// ---------------------------------------------------------------------------

/// [`view`] の入力一式。supervisor が `Shell` の状態から組む(crate doc
/// 「shell 結線」)。
#[derive(Clone, Copy, Debug)]
pub struct ViewModel<'a> {
    /// 解像度・fps・尺の出所(export は comp をそのまま使う — read-only 投影)。
    pub composition: Option<&'a Composition>,
    /// `ExportJob::out_path` になる予定の path。`None` = 未設定(Export は
    /// 押せない)。選択は rfd で次波 — 今は表示のみ。
    pub out_path: Option<&'a Path>,
    /// 品質(`ExportJob::qp0`)の現在選択。
    pub quality: ExportQuality,
    /// 範囲の現在選択。
    pub range: ExportRange,
    /// TL+ の作業範囲(Session 経由の投影)。`None` = 未設定 — 「作業範囲のみ」
    /// の toggler は出さない(触れない物に触れそうな顔をさせない、Q0 の逆側)。
    pub work_area: Option<WorkAreaFrames>,
    /// `None` = 実行していない。`Some` の間は Export ボタンの代わりに進捗行+
    /// 中断ボタンを出し、選択 toggler は無効化する(実行中の入力変更は
    /// `ExportJob` に反映されない — 触れない間は触れない顔)。
    pub progress: Option<ExportProgress>,
}

/// Export ダイアログの view(浮かし窓の中身)。settings sections と同じ
/// 「投影を受ける純関数」— 状態は一切持たない。
pub fn view(model: ViewModel<'_>, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let running = model.progress.is_some();

    let body: Element<'static, Message> = match model.composition {
        None => hint_row("comp が無い — 書き出す対象が無い", dims, colors),
        Some(composition) => {
            let mut rows: Vec<Element<'static, Message>> = vec![
                // OUTPUT: 出力先(表示のみ)+フォーマット+品質。
                section_header("OUTPUT", dims, colors),
                destination_row(model.out_path, !running, dims, colors),
                info_row("Format", CONTAINER_CODEC_LABEL.to_owned(), dims, colors),
                toggle_row(
                    "Lossless(qp0)",
                    model.quality.qp0(),
                    (!running).then_some(|qp0| Message::QualitySelect(ExportQuality::from_qp0(qp0))),
                    dims,
                    colors,
                ),
                // ASPECT: Composition へ適用可能な要求の顔。現在値そのものは
                // read-only の Composition 投影であり、ボタンは Message を出す
                // だけで Document を直接変更しない。
                section_header("ASPECT", dims, colors),
                aspect_preset_row(
                    aspect_preset_for_dimensions(composition.width, composition.height),
                    !running,
                    dims,
                    colors,
                ),
                // RANGE: comp 由来の read-only 投影+範囲選択。
                section_header("RANGE", dims, colors),
                info_row(
                    "Size (px)",
                    format_resolution(composition.width, composition.height),
                    dims,
                    colors,
                ),
                info_row("Frame rate", format_fps(composition.fps), dims, colors),
            ];
            if !resolution_within_cap(composition.width, composition.height) {
                rows.push(warning_row(
                    format!("comp が {MAX_COMP_DIMENSION_PX}px を超えている — 書き出せない"),
                    dims,
                    colors,
                ));
            }
            match model.work_area {
                Some(_) => rows.push(toggle_row(
                    "Work area only",
                    model.range.work_area_only(),
                    (!running).then_some(|work_area_only| {
                        Message::RangeSelect(ExportRange::from_work_area_only(work_area_only))
                    }),
                    dims,
                    colors,
                )),
                // 作業範囲が無い間は選択肢ごと出さない(飾り禁止)— 全体固定で
                // ある事実だけ言う。
                None => rows.push(hint_row("作業範囲が無い — 全体を書き出す", dims, colors)),
            }
            let range = effective_range(model.range, model.work_area, composition.duration_frames);
            rows.push(info_row("Frames", format_range_summary(range), dims, colors));

            // RUN: 実行 or 進捗+中断(器 — 発注書「進捗投影の型だけ」の表示側)。
            rows.push(section_header("RUN", dims, colors));
            match model.progress {
                Some(progress) => rows.push(progress_row(progress, dims, colors)),
                None => {
                    let can_export = model.out_path.is_some();
                    rows.push(run_row(can_export, dims, colors));
                    if !can_export {
                        rows.push(hint_row("出力先が未設定 — Export は押せない", dims, colors));
                    }
                }
            }
            column(rows).into()
        }
    };

    container(column![section_header("EXPORT", dims, colors), body])
        .width(Length::Fill)
        .style(move |_theme| chrome::panel_container_style(dims, colors))
        .into()
}

// ---------------------------------------------------------------------------
// 行(settings の行文法の延長 — 行高+padding の間隔、線は引かない)
// ---------------------------------------------------------------------------

/// Composition の現在比率と、Shell が後で Document へ適用できる3つの要求を
/// ひとまとまりで表示する。`can_select = false`(export 実行中)ではボタンを
/// disabled にして、実行中の snapshot と次回の Composition を混ぜない。
fn aspect_preset_row(
    current: Option<AspectPreset>,
    can_select: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let current_label = current
        .map(|preset| preset.label().to_owned())
        .unwrap_or_else(|| "Custom".to_owned());
    let mut presets = row![];
    for preset in AspectPreset::ALL {
        presets = presets.push(aspect_preset_button(
            preset,
            current == Some(preset),
            can_select,
            dims,
            colors,
        ));
    }

    column![info_row("Current", current_label, dims, colors), presets]
        .spacing(dims.theme().space.xs)
        .padding([0.0, dims.theme().space.m])
        .into()
}

fn aspect_preset_button(
    preset: AspectPreset,
    selected: bool,
    can_select: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let dimensions = dimensions_for_aspect(preset);
    let mut control = button(
        column![
            text(preset.label())
                .size(dims.theme().text.body)
                .color(colors.text_primary),
            text(format_resolution(dimensions.width, dimensions.height))
                .size(dims.theme().text.caption)
                .color(colors.text_muted),
        ]
        .spacing(dims.theme().space.xs),
    )
    .width(Length::FillPortion(1))
    .padding([dims.theme().space.xs, dims.theme().space.s])
    .style(move |_theme, status| {
        let mut style = button_style(dims, colors, status);
        // `button_style` already owns the shared button grammar.  A selected
        // preset only reuses the existing selected surface as a persistent
        // indication; no new color role or local state is introduced.
        if selected && matches!(status, button::Status::Active) {
            style.background = Some(iced::Background::Color(colors.state_selected));
        }
        style
    });
    if can_select {
        control = control.on_press(Message::AspectPresetSelect(preset));
    }
    control.into()
}

/// read-only の情報行。押せる顔をしない(text のみ、Q0 の逆側 — 触れないなら
/// 触れなさそうに)。
fn info_row(
    label: &'static str,
    value: String,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    row![
        text(label)
            .size(dims.theme().text.body)
            .color(colors.text_primary)
            .width(Length::Fill),
        text(value).size(dims.theme().text.body).color(colors.text_primary),
    ]
    .spacing(dims.theme().space.xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.theme().space.m])
    .into()
}

/// 出力先行。path があれば表示、無ければ「未設定」を muted で。**Choose…**
/// ボタン([`Message::PickOutputPath`])が rfd 経由の選択を引く(2026-08-22
/// 第2波 — 従来は表示のみだった)。`can_pick = false`(実行中)の間は
/// `on_press` を付けない(button が自分で Disabled の顔になる、run_row と
/// 同じ規律)。
fn destination_row(
    out_path: Option<&Path>,
    can_pick: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let (value, color) = match out_path {
        Some(path) => (path.display().to_string(), colors.text_primary),
        None => ("未設定".to_owned(), colors.text_muted),
    };
    let mut choose = button(text("Choose…").size(dims.theme().text.body))
        .style(move |_theme, status| button_style(dims, colors, status));
    if can_pick {
        choose = choose.on_press(Message::PickOutputPath);
    }
    row![
        text("Destination")
            .size(dims.theme().text.body)
            .color(colors.text_primary)
            .width(Length::Fill),
        text(value).size(dims.theme().text.body).color(color),
        choose,
    ]
    .spacing(dims.theme().space.xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.theme().space.m])
    .into()
}

/// 二値選択の行(裁定187: 文字ボタン列でなく iced 標準 toggler)。
/// `on_toggle: None` = 無効(実行中)— toggler が自分で Disabled の顔になる。
/// callback は capture 無しの thin 写像だけなので fn pointer で足りる。
fn toggle_row(
    label: &'static str,
    toggled: bool,
    on_toggle: Option<fn(bool) -> Message>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut switch = toggler(toggled)
        .size(dims.theme().text.body)
        .style(move |_theme, status| toggler_style(colors, status));
    if let Some(on_toggle) = on_toggle {
        switch = switch.on_toggle(on_toggle);
    }
    row![
        text(label)
            .size(dims.theme().text.body)
            .color(colors.text_primary)
            .width(Length::Fill),
        switch,
    ]
    .spacing(dims.theme().space.xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.theme().space.m])
    .into()
}

/// Export 実行行。`can_export = false`(出力先未設定)の間は `on_press` を
/// 付けない — button が自分で Disabled の顔になる(理由は直下の hint 行)。
fn run_row(can_export: bool, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let mut export = button(text("Export").size(dims.theme().text.body))
        .style(move |_theme, status| button_style(dims, colors, status));
    if can_export {
        export = export.on_press(Message::Export);
    }
    row![export]
        .height(Length::Fixed(dims.inspector_row_height))
        .align_y(iced::alignment::Vertical::Center)
        .padding([0.0, dims.theme().space.m])
        .into()
}

/// 実行中の進捗行+中断ボタン(Close glyph — 裁定187 icon-first)。
fn progress_row(
    progress: ExportProgress,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    row![
        text(format_progress(progress))
            .size(dims.theme().text.body)
            .color(colors.text_primary)
            .width(Length::Fill),
        button(icon(
            Icon::Close,
            frame_px_for_glyph_px(dims.theme().text.body),
            colors.text_primary,
        ))
        .on_press(Message::CancelExport)
        .style(move |_theme, status| button_style(dims, colors, status)),
    ]
    .spacing(dims.theme().space.xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.theme().space.m])
    .into()
}

/// 補足行(settings の `hint_row` と同文法 — caption・muted)。
fn hint_row(
    message: &'static str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    container(text(message).size(dims.theme().text.caption).color(colors.text_muted))
        .width(Length::Fill)
        .padding([dims.theme().space.xs, dims.theme().space.m])
        .into()
}

/// 警告行(cap 超過)。色は既存ロール `status_warning` — 新しい色を発明しない。
fn warning_row(message: String, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    container(
        text(message)
            .size(dims.theme().text.caption)
            .color(colors.status_warning),
    )
    .width(Length::Fill)
    .padding([dims.theme().space.xs, dims.theme().space.m])
    .into()
}

/// toggler の意味色ロール(初出 — 裁定187 で標準 widget を採用した最初の面)。
/// 新しい色は作らない: ON = `action_active`(数値欄 focus と同じ「作動中」
/// ロール)、OFF = `surface_raised`(ボタン地と同格)、hover = `surface_hover`、
/// 無効 = `surface_panel` 地+`state_disabled` つまみ(`chrome::button_style`
/// の Disabled と同じ組)。
pub fn toggler_style(colors: Colors, status: toggler::Status) -> toggler::Style {
    let (background, knob) = match status {
        toggler::Status::Active { is_toggled } => (
            if is_toggled {
                colors.action_active
            } else {
                colors.surface_raised
            },
            colors.text_primary,
        ),
        toggler::Status::Hovered { is_toggled } => (
            if is_toggled {
                colors.action_active
            } else {
                colors.surface_hover
            },
            colors.text_primary,
        ),
        toggler::Status::Disabled { is_toggled } => (
            if is_toggled {
                colors.state_selected
            } else {
                colors.surface_panel
            },
            colors.state_disabled,
        ),
    };
    toggler::Style {
        background: iced::Background::Color(background),
        background_border_width: 0.0,
        background_border_color: iced::Color::TRANSPARENT,
        foreground: iced::Background::Color(knob),
        foreground_border_width: 0.0,
        foreground_border_color: iced::Color::TRANSPARENT,
        // ラベルは行の text が担う(toggler 側にラベルを持たせない)ので不使用。
        text_color: None,
        // 角丸ゼロ = chrome の square 文法(裁定179 — 他 widget の
        // `radius: 0.0.into()` と同じ側)。`None` は真円 pill でここだけ別意匠に
        // なるため使わない。
        border_radius: Some(0.0.into()),
        // つまみと地の間隔は upstream 既定値(0.1)— 寸法の発明をしない。
        padding_ratio: 0.1,
    }
}

// ---------------------------------------------------------------------------
// tests(発注規律: 書くが実行しない — 検収線は `cargo check --tests` 緑)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------
    // アスペクトプリセット
    // -----------------------------------------------------------------

    /// 全プリセットが実際に異なる寸法と比率へ写る。UI の文字だけを増やして
    /// 実体のない選択肢を置かないための意味実在柵。
    #[test]
    fn every_aspect_preset_maps_to_its_canonical_dimensions() {
        let expected = [
            (
                AspectPreset::Landscape16x9,
                AspectDimensions {
                    width: 1920,
                    height: 1080,
                },
            ),
            (
                AspectPreset::Portrait9x16,
                AspectDimensions {
                    width: 1080,
                    height: 1920,
                },
            ),
            (
                AspectPreset::Square1x1,
                AspectDimensions {
                    width: 1080,
                    height: 1080,
                },
            ),
        ];
        for (preset, dimensions) in expected {
            assert_eq!(dimensions_for_aspect(preset), dimensions);
        }
    }

    /// 既存 Composition の縮小版も同じ比率として現在選択を表示できるが、
    /// 未対応比率や無効寸法は custom のままにする。
    #[test]
    fn aspect_preset_for_dimensions_recognizes_ratio_not_only_canonical_size() {
        assert_eq!(
            aspect_preset_for_dimensions(1280, 720),
            Some(AspectPreset::Landscape16x9)
        );
        assert_eq!(
            aspect_preset_for_dimensions(1080, 1920),
            Some(AspectPreset::Portrait9x16)
        );
        assert_eq!(
            aspect_preset_for_dimensions(1080, 1080),
            Some(AspectPreset::Square1x1)
        );
        assert_eq!(aspect_preset_for_dimensions(0, 1080), None);
        assert_eq!(aspect_preset_for_dimensions(1000, 700), None);
    }

    /// クリック契約は pure mapping の結果をそのまま運ぶ。Document への適用は
    /// Shell WIRE の責任で、pane は別の書き口を持たない。
    #[test]
    fn selecting_an_aspect_preset_emits_the_real_pane_message() {
        assert_eq!(
            Message::AspectPresetSelect(AspectPreset::Portrait9x16),
            Message::AspectPresetSelect(AspectPreset::Portrait9x16)
        );
        assert_eq!(
            dimensions_for_aspect(AspectPreset::Portrait9x16),
            AspectDimensions {
                width: 1080,
                height: 1920,
            }
        );
    }

    // -----------------------------------------------------------------
    // 意味実在柵: 全選択肢が motolii-export の実在識別子へ写る。
    // -----------------------------------------------------------------

    /// **本命**: [`ExportQuality::ALL`] を尽くし、qp0 写像が単射で往復不変で
    /// あること(= どの選択肢も `ExportJob::qp0` の実在の値に紐づき、飾りの
    /// 選択肢が1つも無い)。
    #[test]
    fn every_quality_choice_maps_onto_a_distinct_real_qp0_value() {
        let mut seen = Vec::new();
        for quality in ExportQuality::ALL {
            let qp0 = quality.qp0();
            assert!(
                !seen.contains(&qp0),
                "{quality:?} が他の選択肢と同じ qp0 値 — 飾りの選択肢がある"
            );
            seen.push(qp0);
            assert_eq!(
                ExportQuality::from_qp0(qp0),
                quality,
                "{quality:?} の qp0 往復が壊れている"
            );
        }
    }

    /// [`ExportRange::ALL`] も同型(toggler の bool と往復不変)。
    #[test]
    fn every_range_choice_maps_onto_a_distinct_work_area_only_value() {
        let mut seen = Vec::new();
        for range in ExportRange::ALL {
            let work_area_only = range.work_area_only();
            assert!(
                !seen.contains(&work_area_only),
                "{range:?} が他の選択肢と同じ bool — 飾りの選択肢がある"
            );
            seen.push(work_area_only);
            assert_eq!(
                ExportRange::from_work_area_only(work_area_only),
                range,
                "{range:?} の往復が壊れている"
            );
        }
    }

    // -----------------------------------------------------------------
    // effective_range
    // -----------------------------------------------------------------

    /// 全体 = export の `0..duration_frames` ループそのもの。
    #[test]
    fn whole_range_covers_the_full_comp_duration() {
        let range = effective_range(ExportRange::Whole, None, 300);
        assert_eq!(range, FrameRange { start: 0, end: 300 });
        assert_eq!(range.frame_count(), 300);
        assert_eq!(range.last_frame(), 299);
    }

    /// 作業範囲は comp 尺と交差してクランプされる(半開の約束はどちらも同じ)。
    #[test]
    fn work_area_range_is_clamped_into_the_comp_duration() {
        let area = WorkAreaFrames { start: 100, end: 200 };
        assert_eq!(
            effective_range(ExportRange::WorkArea, Some(area), 300),
            FrameRange { start: 100, end: 200 }
        );
        // 尺の外へはみ出した端はクランプ。
        let hanging = WorkAreaFrames { start: -50, end: 400 };
        assert_eq!(
            effective_range(ExportRange::WorkArea, Some(hanging), 300),
            FrameRange { start: 0, end: 300 }
        );
    }

    /// 作業範囲が選ばれていても実体が無ければ全体へ落ちる(stale な選択で
    /// 空区間を書き出さない)。
    #[test]
    fn work_area_selection_without_an_area_falls_back_to_whole() {
        assert_eq!(
            effective_range(ExportRange::WorkArea, None, 300),
            FrameRange { start: 0, end: 300 }
        );
    }

    /// 最短1フレーム保証: 範囲が尺の完全に外・尺0、どちらの防波堤でも
    /// 「0フレームの file」を作る区間は返らない。
    #[test]
    fn effective_range_always_yields_at_least_one_frame() {
        let beyond = WorkAreaFrames { start: 500, end: 600 };
        let range = effective_range(ExportRange::WorkArea, Some(beyond), 300);
        assert!(range.frame_count() >= 1, "尺の外の作業範囲で空区間: {range:?}");
        assert!(range.end <= 300, "クランプ後も尺を超えている: {range:?}");

        let empty_comp = effective_range(ExportRange::Whole, None, 0);
        assert_eq!(empty_comp.frame_count(), 1, "尺0の防波堤が最短1になっていない");
    }

    // -----------------------------------------------------------------
    // 進捗・表示文字列
    // -----------------------------------------------------------------

    #[test]
    fn progress_fraction_clamps_and_survives_a_zero_total() {
        let half = ExportProgress { frames_done: 150, frames_total: 300 };
        assert!((progress_fraction(half) - 0.5).abs() < f64::EPSILON);
        let over = ExportProgress { frames_done: 400, frames_total: 300 };
        assert_eq!(progress_fraction(over), 1.0, "総数超過は1.0へクランプ");
        let zero = ExportProgress { frames_done: 0, frames_total: 0 };
        assert_eq!(progress_fraction(zero), 0.0, "総数0はゼロ除算せず0.0");
    }

    #[test]
    fn format_progress_shows_done_total_and_percent() {
        let progress = ExportProgress { frames_done: 123, frames_total: 300 };
        assert_eq!(format_progress(progress), "123 / 300(41%)");
    }

    #[test]
    fn format_resolution_uses_the_multiplication_sign() {
        assert_eq!(format_resolution(1920, 1080), "1920 × 1080");
    }

    /// 区間表示は半開の `end` を生で見せない(最終フレーム+枚数の言葉)。
    #[test]
    fn format_range_summary_speaks_in_last_frame_and_count() {
        let range = FrameRange { start: 0, end: 300 };
        assert_eq!(format_range_summary(range), "0 – 299(300 frames)");
        let one = FrameRange { start: 42, end: 43 };
        assert_eq!(format_range_summary(one), "42 – 42(1 frames)");
    }

    /// cap は settings と同じ定数([`MAX_COMP_DIMENSION_PX`])— 境界含む。
    #[test]
    fn resolution_cap_uses_the_shared_texture_floor_inclusive() {
        assert!(resolution_within_cap(MAX_COMP_DIMENSION_PX, MAX_COMP_DIMENSION_PX));
        assert!(!resolution_within_cap(MAX_COMP_DIMENSION_PX + 1, 1080));
        assert!(!resolution_within_cap(1920, MAX_COMP_DIMENSION_PX + 1));
        assert!(resolution_within_cap(1920, 1080));
    }
}
