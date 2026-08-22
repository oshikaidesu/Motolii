//! wraps: iced — Inspector pane(選択レイヤの属性・transform の読み書き、drag-to-scrub)。書き込みは Intent 経由のみ。
//! Inspector pane(第1波: Transform行 + Attrs行)。
//!
//! **視覚正本は `docs/mocks-ui/public/inspector-library.html` + `.css` そのもの**
//! (発注書 CANON)。旧 `crates/` 側の egui/iced 実装は手本にしない — `next/` は
//! 移植元ではなく成果を作る側(`../GOALS.md` 冒頭の規律どおり)。
//!
//! [`project`] が `StoreView`/[`motolii_shell_state::Session`] から**Document の写しではない、
//! 使い捨ての投影**(`timeline_pane::rows` と同じ形、裁定5)を作る。[`view`] は
//! それを iced widget へ描くだけで、投影の中身を判断しない。
//!
//! **canvas を使わない**(`timeline_pane` と違う選択)。KNOWN.md の実測
//! (「canvas と slider は Simulator から構造的に不可視」)どおり、canvas に
//! text 入力を乗せると Q0 横断柵からも iced_test からも見えなくなる。Inspector の
//! 行は値の型入力・改名・トグルが主役なので、`text_input`/`button` の標準 widget で
//! 組み、柵がそのまま効く形を選んだ(視覚正本と食い違う点として終了報告に書く)。
//!
//! **編集の確定方式**: 打鍵のたびに `Intent` を出すと1文字ごとに undo が割れる
//! (`ui-quality-bar` Q2)。[`motolii_shell::Shell`] は [`FieldDraft`]/name 下書きという
//! **Document ではない一時状態**(`motolii_shell::Shell::pending_drops` と同じ形)を持ち、
//! `on_submit`(Enter)で初めて1回の `Intent::SetTrack`/`SetAttrs` を出す — 1 gesture
//! = 1 undo。**静的値の編集は `SetTrack` に1キー `Hold`** で書く([`single_hold_track`])
//! — 発注書がその流儀を名指ししている。**キー持ち track の値編集は playhead への
//! キー upsert**([`edited_value_track`]、AE 作法 — 2026-08-22 発注): track を
//! 静的に戻さず、playhead にキーが有れば値更新・無ければ新キー挿入。
//!
//! **型別 editor**(rerun `re_component_ui::create_component_ui_registry` の型→editor
//! 登録表と同じ考え方、コードは引かず型だけ写す): この第1波で使うのは
//! `motolii_eval::Value` のうち `F64`(数値行)と `Vec2`(2連 = X/Y 相当)だけ。
//! `Bool` は Attrs の hidden トグル(`Value` 経由ではなく `LayerAttrs::hidden` だが、
//! 型としては同格の on/off editor)。`Color`/`Enum`/`Path`/`LayerId` は Effect 束
//! (第1波の範囲外)の仕事。
//!
//! **drag-to-scrub**(第2波の一部を前倒し、利用者の直接依頼): 値セルは
//! `mouse_area` でラップし、press→(実質的な移動があれば)drag、移動が無いまま
//! release なら従来どおり click→type 編集(併存、[`value_cell`] 参照)。
//! **transient 値は `Document` へ直接書く**([`continue_field_drag`]) —
//! 1手ごとに `doc.undo()` してから書き直すことで history を1件に畳み、
//! release 時点の最後の1手がそのまま確定値になる(= 1 gesture 1 undo、
//! `next/core/motolii-store/src/document.rs` の `apply_all` doc comment
//! 「ドラッグは対象外…途中経過は pane が持ち、確定の1件だけが intent」を
//! 素直に実装した形)。Stage・Inspector セルの「ドラッグ中の即応」は**この
//! transient apply が投影を通して自然に見えるだけ**で、専用の preview 経路は
//! 持たない(`refresh_frame` の revision 判定がそのまま効く)。
//!
//! **iced 0.14 の制約**: `mouse_area` は自分の bounds を出た cursor を追えない
//! (pointer capture が無い実測、`mouse_area.rs::update` が `cursor.is_over` で
//! 弾く)。値セルは幅38pxしか無いので、感度どおりに大きく動かすとすぐ bounds を
//! 出てしまう — window 全体の `CursorMoved`/`ButtonReleased` を
//! `iced::event::listen_with` で拾う形に倒した(`motolii_shell::inspector_pointer_event`)。
//! mouse_area の `on_press` は「この field の drag を armed にする」ためだけに使う。
//!
//! ## 裁定160 切片8: crate 抽出(`motolii-shell` → `motolii-inspector-pane`)
//! `docs/reviews/2026-08-21-pane-split-survey.md` §6 切片8。**挙動ゼロ変更**。
//! 切片9(`motolii-settings-pane`)と同じ形を踏襲する。
//!
//! - **`Message` は pane ローカル**(この crate の [`Message`])。`motolii-shell`
//!   root の `Message::Inspector(inspector_pane::Message)` が1本で畳む。旧腕名の
//!   "Inspector" prefix は pane 名前空間で二重になるので剥がした
//!   (`InspectorFieldInput` → `FieldInput` 等、切片9の "Settings" prefix 剥がしと
//!   同じ判断)。cross-cutting な2 Message(`KeyboardModifiersChanged`/
//!   `EscapePressed` — timeline drag と inspector drag 両方が読む)はここへは
//!   移さず root に残した(pane split survey §1.3)。
//! - **`project` は `&Session` を直接取る**: `Session` は裁定160 切片6→切片7で
//!   `motolii-shell-state` leaf crate へ抽出済み(`motolii-timeline-pane` が
//!   `Session`/`KeySelector` を読むのと同じ理由 — root(`motolii-shell`)へ
//!   依存できない pane crate 同士の共通の親)。この crate も同じ leaf crate へ
//!   依存するので、`project` のシグネチャ・判定ロジックとも無改変で通る
//!   (root → pane の一方向依存を保ったまま、切片7で解消済みの循環回避策を
//!   ここでも使い回しただけ)。
//! - **書ける物を持たない、が書き口(自由関数)は持つ**(切片9と同じ形):
//!   [`commit_inspector_field`]/[`commit_inspector_name`]/[`start_field_drag`]/
//!   [`continue_field_drag`]/[`finish_field_drag`]/[`cancel_field_interaction`]
//!   は `&mut Document`/`&mut Option<_>` 下書き・`&mut Option<FieldDragState>` を
//!   明示引数で受け取る自由関数。呼び出し口は `motolii_shell::Shell::
//!   update_inspector` + 個々の glue メソッド(`self.doc`/`self.session.selection`
//!   等をそのまま貸すだけ)。
//! - **`iced::widget::operation::focus` を返す軌道は Shell 側に残した**:
//!   click→type 切替(`Shell::enter_field_editing`)は Document を一切読み書き
//!   しない UI 純粋な focus orchestration — `Task<motolii_shell::Message>` を
//!   直接組み立てられる root 側に置く方が pane crate を跨ぐ `Task` の型変換を
//!   増やさずに済む(この crate 自身は `Task` を返す関数を持たない)。
//!   [`finish_field_drag`] は「click だった」ことだけを `Ok(Some(field))` で
//!   知らせ、`Task` の組み立ては呼び出し側に委ねる。
//! - **`toggle_inspector_hidden` は移設していない**: 元の実装は
//!   `Session::selection` を読んで cross-cutting な `Shell::toggle_layer_hidden`
//!   (`LaneBarToggleMute` とも共有)へ委譲するだけで、Inspector 固有の書き
//!   ロジックを1行も持たない — 「pane が自分の write ロジックを持つ」形に
//!   当てはまらないので、この関数だけは `motolii-shell` root に残した
//!   (RETURN の write-set 外 finding 参照)。
//! - **`FieldDragState`(drag-to-scrub の transient 状態)はここへ移した**:
//!   `Session::selection` と同じく置き場(`Shell::inspector_drag` フィールド)は
//!   移設していないが、型定義とそれを読み書きする自由関数はこの crate 側
//!   (発注書「transient overlay 経由の drag はそのまま移動」— 挙動不変)。

use motolii_core::{Fps, RationalTime};
use motolii_store::{
    property, ContentTrack, Document, EffectId, EffectInstance, FontRef, Intent, Interp,
    Keyframe, KeyframeTrack, LayerAttrsPatch, LayerId, LayerSource, Mask, MaskId, MaskMode,
    PropertyId, StoreError, StoreView, TextAlignmentOptions, TextDocument, TextDocumentStyle,
    TextJustify, TextStyleId, Value,
};

use motolii_settings_pane::chrome::{
    panel_container_style, parse_number, section_header, value_input_style,
};
use motolii_shell_state::Session;
use motolii_tokens_rs::{Colors, Dimensions, Ink, TextWeight, LABEL_PALETTE_LEN};

// ---------------------------------------------------------------------------
// section 単位モジュール分割(利用者指摘 — 4,907行の単一ファイルが原因で merge
// 事故が起きた実害への対処、裁定160 pane 分割と同じ「積む前に割る」)。
// **公開 API は不変**: 下の `pub use` が旧来の `inspector_pane::X` フラットな
// 経路をそのまま維持する(shell 側は無改修)。続く `use X::*;`(pub 無し)は
// この lib.rs 自身の組み立てコード(`view`/`selected_body`/`ident_band` 等)と
// 末尾の `mod tests`(`use super::*;` 経由)が各 section の `pub(crate)` ヘルパ
// を無名で読むための取り込み ── 同名の explicit `pub use` が glob より優先
// されるので二重取り込みでも衝突しない。
// ---------------------------------------------------------------------------
mod attrs;
mod audio;
mod chrome;
// 色エディタ(2026-08-22 発注)。crate 本体の `Message`/`view` へはまだ結線
// していない自己完結モジュール(`motolii-settings-pane::sections` 第1切片と
// 同じ「供覧」の形、`color.rs` 冒頭 doc 参照)——この `pub mod color;` の1行
// だけがこの発注の `lib.rs` への変更点(`sections` と同じく `pub mod` —
// 未結線でも crate の公開 API の一部にする、dead_code 警告を呼ばない形)。
pub mod color;
mod effects;
// LINK section(2026-08-22 発注「レイヤーを指す」文法 第3号)。型付き
// `PropertySource::Link` の参照先を選ぶ pick_list とその意味・書き口。
mod link;
mod mask;
// MATTE 行(2026-08-22 発注「レイヤーを指す」文法 第1号)。`LayerAttrs.matte`
// の元を選ぶ pick_list とその意味・書き口。
mod matte;
mod projection;
mod text;
mod transform;

pub use attrs::{next_blend_mode, percent_to_speed_ratio, speed_percent, SUPPORTED_BLEND_MODES};
pub use chrome::{cycle_inspector_label_color, next_label_color, property_row_css, row_band_style};
pub use effects::{
    effects_with_moved_down, effects_with_moved_up, effects_with_removed,
    move_inspector_effect_down, move_inspector_effect_up, plugin_display_name, plugin_params,
    remove_inspector_effect, toggle_inspector_effect_bypass, GlowParam, GLOW_PLUGIN_ID,
};
pub use link::{
    clear_inspector_link, commit_inspector_link, LinkRowProjection, LinkSourceCandidate,
    LinkTarget,
};
pub use mask::{
    cycle_inspector_mask_mode, masks_with_cycled_mode, masks_with_toggled_inverted,
    next_mask_mode, toggle_inspector_mask_inverted,
};
pub use matte::{
    clear_inspector_matte, cycle_inspector_matte_mode, next_matte_mode,
    set_inspector_matte_source,
};
pub use projection::{
    project, AttrsProjection, AudioSectionProjection, ComponentSlot, EffectRowProjection,
    KeyCellProjection, LayerCandidate, MaskRowProjection, RowValue, SelectionProjection,
    TextSectionProjection, TransformRowProjection,
};
pub use text::{
    applied_text_field, commit_text_field, commit_text_font_pick, cycle_text_justify,
    default_text_document, default_text_style, next_text_justify, reset_text_line_height,
    reset_text_tracking, TextField, TextFieldDraft,
};
pub use transform::{
    cancel_field_interaction, commit_inspector_field, commit_inspector_name,
    continue_field_drag, default_vec2, display_number, drag_origin, dragged_value,
    edited_value_track, field_decimals, field_input_id, finish_field_drag, format_number,
    key_cell_state, key_row_default_value, key_row_property_id, next_value, property_id,
    single_hold_track, start_field_drag, toggled_key_track, FieldDragState, FieldDraft,
    KeyCellState, KeyRow, TransformField, DRAG_SHIFT_FACTOR, MAX_VALUE_CELL_CHARS,
};

use attrs::*;
use audio::*;
use chrome::*;
use effects::*;
use link::*;
use mask::*;
use projection::*;
use text::*;
use transform::*;

// ---------------------------------------------------------------------------
// pane ローカル Message(裁定160 切片8 — root `Message::Inspector(Message)` が
// 1本で畳む、上記 crate doc 参照)。
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// Transform 行の値セルへの打鍵。**まだ Document を書かない** — 下書きを
    /// 更新するだけ(`FieldDraft`、`motolii_shell::Shell::pending_drops` と同じ形)。
    FieldInput(TransformField, String),
    /// Transform 行の Enter — **ここで初めて `Intent::SetTrack` を1回出す**
    /// (1 gesture = 1 undo)。
    FieldSubmit(TransformField),
    /// Attrs の Name 欄への打鍵。同上、まだ書かない。
    NameInput(String),
    /// Attrs の Name 欄の Enter — `Intent::SetAttrs` を1回出す。
    NameSubmit,
    /// Attrs の Hidden トグル。下書きを経由せず即 `Intent::SetAttrs` を1回出す
    /// (header の Undo/Redo ボタンと同じ即時操作の形)。
    ToggleHidden,
    /// Attrs の Blend 巡回ボタン。`ToggleHidden` と同じ即時操作の形 — 下書きを
    /// 経由せず即 `Intent::SetAttrs` を1回出す。巡回先は
    /// [`SUPPORTED_BLEND_MODES`]/[`next_blend_mode`](発注書「決定済み事項」—
    /// pick_list は導入しない、対応 mode だけを巡る)。
    CycleBlendMode,
    /// 値セルの press。**まだ Document を書かない** — click か drag かは
    /// release まで未確定(`Shell::inspector_drag`)。
    ValuePressed(TransformField),
    /// window 全体の cursor 移動(`subscription()` の `inspector_pointer_event`
    /// 経由)。`mouse_area` 自身の bounds を出た cursor は iced 0.14 に pointer
    /// capture が無く追えない(実測)ので、drag 中の主経路はここ。drag が
    /// armed/dragging でなければ即 no-op。
    PointerMoved(iced::Point),
    /// 左クリック release(同じく window 全体から)。drag が実際に動いていれば
    /// 直前の move が確定値(1 gesture = 1 undo)、動いていなければ click として
    /// type 編集へ切り替える。
    PointerReleased,

    // ---- ATTRS: Speed 欄(SP1 第一波、supervisor 決定1-7) ----
    /// Speed 欄への打鍵。**まだ Document を書かない** — 下書きを更新するだけ
    /// (`FieldInput` と同じ形 — ただし Speed は `LayerTiming` の一部で
    /// `TransformField`/track を経由しないので、下書きは `TransformField` に
    /// 紐付かない単純な `String`)。
    SpeedInput(String),
    /// Speed 欄の Enter — ここで初めて1回の `Intent::SetTiming` を出す
    /// (1 gesture = 1 undo、duration も同時に再計算する — 決定4)。
    SpeedSubmit,
    /// Speed 行の Reset ボタン。下書きを経由せず即 100% へ — 既に100%なら
    /// no-op(`Intent` を出さない、決定7)。`ToggleHidden`/`CycleBlendMode` と
    /// 同じ即時操作の形。
    ResetSpeed,

    // ---- K1: Key 列(Inspector からキーフレームを打つ) ----
    /// Transform 行の Key セル click。下書きを経由せず即1回の `Intent::SetTrack`
    /// を出す(`ToggleHidden` と同じ即時操作の形 — 1 click = 1 undo)。3状態の
    /// 意味は [`toggled_key_track`] のとおり: 静的→playhead にキー1個 /
    /// キー上→除去(最後の1個は値を保って静的化)/ track 有りキー無し→評価値で
    /// キー追加。
    KeyPressed(KeyRow),

    // ---- MASK section(B02 第1切片、裁定184) ----
    /// この mask の mode を宣言順の次へ巡回。`CycleBlendMode` と同じ即時操作の
    /// 形(下書きを経由せず即1回の `Intent::SetMasks`)— pick_list は next/ に
    /// 前例が無い(BL2 の決定)ので、既存の巡回ボタン文法をそのまま使う。
    CycleMaskMode(MaskId),
    /// この mask の inverted を裏返す。`ToggleHidden` と同じ即時操作の形
    /// (即1回の `Intent::SetMasks`)。
    ToggleMaskInverted(MaskId),

    // ---- EFFECTS section(B38 編集側 第3切片、裁定184 型別 section 第2号) ----
    /// この effect を適用済み stack から取り除く。即1回の `Intent::SetEffects`
    /// (1 click = 1 undo — `CycleMaskMode` と同じ即時操作の形)。取り除いた
    /// effect の param track(`effect.{id}.param.*`)は Document に残る(inert —
    /// `StoreView::resolved_effects` は列に居る effect の分しか読まない)。track
    /// まで同時に消すと Intent が複数になり 1 click = 1 undo が割れるので消さない
    /// (undo で effect が戻れば param もそのまま戻る、という余得もある)。
    RemoveEffect(EffectId),
    /// この effect を1つ上(適用順の前)へ。既に先頭なら **Intent を出さない**
    /// no-op(空 undo 段を作らない — [`effects_with_moved_up`] が `None` を返す)。
    MoveEffectUp(EffectId),
    /// この effect を1つ下(適用順の後)へ。既に末尾なら同じく no-op。
    MoveEffectDown(EffectId),
    /// この effect の enabled(`effects/effect/en`)を裏返す — bypass。
    /// **消さずに切る**(`EffectInstance::enabled` の doc どおり、削除とは別物)。
    /// `ToggleMaskInverted` と同じ即時操作の形(即1回の `Intent::SetEffects`)。
    ToggleEffectBypass(EffectId),

    // ---- ラベル色チップ(B03、ident 帯) ----
    /// ident 帯の色チップ click — `LayerAttrs.label_color` の palette index を
    /// 宣言順の次へ巡回(即1回の `Intent::SetAttrs`)。pick_list は next/ に
    /// 前例が無い(BL2 の決定)ので、既存の巡回ボタン文法をそのまま使う。
    CycleLabelColor,

    // ---- TEXT section(B46 第1切片、裁定184) ----
    /// TEXT section の text_input 系フィールドへの打鍵。**まだ Document を
    /// 書かない** — 下書きを更新するだけ(`FieldInput` と同じ形)。
    /// `TextDocumentStyle` のフィールド(font/size/line_height/tracking)は
    /// 裁定92「v1でスパン style はキーフレーム化しない」により `KeyframeTrack`
    /// に乗らない — `TransformField`/`property_id`/drag-to-scrub の経路は
    /// 使わず、Speed 欄(ATTRS)と同じ「即時 text_input・on_submit で1回の
    /// Intent」文法をそのまま流用する(新しい編集文法を発明しない)。
    TextFieldInput(TextField, String),
    /// TEXT section のフィールドの Enter — ここで初めて1回の
    /// `Intent::SetTextDocument` を出す(1 gesture = 1 undo、`SpeedSubmit` と
    /// 同じ形。ただし書く先は `LayerTiming` ではなく `TextDocument` 丸ごと —
    /// `SetTextDocument` は部分更新を持たない、`text.rs` doc 参照)。
    TextFieldSubmit(TextField),
    /// Justify(揃え、`text-document j`)の巡回ボタン。`CycleBlendMode`/
    /// `CycleMaskMode` と同じ即時操作の形(下書きを経由せず即1回の
    /// `Intent::SetTextDocument`)。
    CycleTextJustify,
    /// Line Height を Auto(`None` — フォントのメトリクスから)へ戻すボタン。
    /// `ResetSpeed` と同じ即時操作の形(map「Auto leading for selected
    /// text」、採用済)。既に Auto なら no-op(決定7 と同じ判断)。
    ResetLineHeightAuto,
    /// Tracking を 0 へ戻すボタン。`ResetSpeed` と同じ即時操作の形(map
    /// 「Reset tracking to 0」、採用予定)。既に0なら no-op。
    ResetTracking,
    /// 2026-08-22 追い発注「フォントが選べる・選ばなくても落ちない」:
    /// `font_family_row` の pick_list からの選択(payload = 選んだ family)。
    /// 手打ち欄(`TextFieldInput`/`TextFieldSubmit` with
    /// `TextField::FontFamily`)とは別腕 — 下書きを経由しない即時操作
    /// (`CycleBlendMode` と同じ形)。書き込み本体は
    /// [`commit_text_font_pick`] — family と path を同時に書く(手打ち欄には
    /// `path` を編集する手段が無かった穴への対処、
    /// `text.rs::font_family_row` doc 参照)。
    PickFont(String),

    // ---- 色エディタ(`crate::color`、2026-08-22 発注「歌詞が入れられる道を
    // 通す」で結線) ----
    /// TEXT section の Fill/Stroke 色欄。`color` module は自己完結の
    /// pane-local `Message` を持つ(`Message::Timeline`/`Message::Settings`
    /// と同じ「子 pane の Message を親が wrap する」形、`color.rs` 冒頭 doc
    /// 「まだ結線していない」の解消)。
    Color(color::Message),

    // ---- MATTE(2026-08-22 発注「レイヤーを指す」文法 第1号 — engine は
    // `MatteMode`/`Matte` の消費を既に開始しているのに、指定する UI が無
    // かった) ----
    /// MATTE 元の pick_list からの選択。`PickFont` と同じ即時操作の形(下書き
    /// を経由しない、即1回の `Intent::SetAttrs`)。
    PickMatteSource(LayerId),
    /// MATTE mode 巡回ボタン。`CycleBlendMode`/`CycleMaskMode` と同じ即時
    /// 操作の形。
    CycleMatteMode,
    /// MATTE を外す。`ResetSpeed` と同じ即時操作の形(matte が無ければ
    /// no-op)。
    ClearMatte,

    // ---- LINK(2026-08-22 発注「レイヤーを指す」文法 第3号 — 型付き
    // `PropertySource::Link` は本日着地したが呼び手がゼロだった) ----
    /// LINK 元の pick_list からの選択(payload = 対象 property + 選んだ
    /// 参照先)。`PickFont`/`PickMatteSource` と同じ即時操作の形(即1回の
    /// `Intent::SetPropertyLink`)。
    PickLinkSource(LinkTarget, LinkSourceCandidate),
    /// LINK を外す(`Intent::SetTrack` を再び投げて static 値へ戻す —
    /// `crate::slot` doc「専用の解除 variant は要らない」の実装)。
    /// `ResetSpeed` と同じ即時操作の形(link でなければ no-op)。
    ClearLink(LinkTarget),
}

// ---------------------------------------------------------------------------
// view — StoreView の投影(SelectionProjection)と下書きだけを受け取る。書けない。
// ---------------------------------------------------------------------------

// `text` は `mod text;`(TEXT section モジュール、裁定160 型の分割)と同名衝突
// するため `iced_text` へ別名(呼び出し側の2箇所[`empty_state`]/[`ident_band`]
// だけ書き換え、意味は不変)。
// `button`(モジュールパス)は末尾の `mod tests` が `button::Status::..` を
// 直接参照するため必要(このファイル自身の残存コードは `button` widget 関数を
// 呼ばない — 中身の意匠は `crate::chrome` へ移設済み)。
use iced::widget::{button, column, container, row as row_widget, scrollable, text as iced_text, text_input};
use iced::{Element, Length};

pub fn view(
    projection: Option<&SelectionProjection>,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    view_with_speed_draft(projection, field_draft, name_draft, None, dims, colors)
}

/// [`view`] と同じだが、Speed 欄(ATTRS、SP1 第一波)の編集下書きも渡せる。
/// `motolii_shell::Shell::view` はこちらを呼ぶ。**`view` 自身の4引数シグネチャは
/// 変えていない** — 既存の呼び出し元(`tests/suite/inspector_pixel_fence.rs`・
/// `ident_band_drive.rs`・`ui_scale_fence.rs`、いずれも今回の発注書 ALLOWLIST
/// 外)を無改修のまま通すため(RETURN の FINDING 参照)。`view` はここへ
/// `speed_draft: None` で委譲するだけ — Speed 行は常に確定値(下書き無し)を
/// 表示する形で、挙動は変わらない。
pub fn view_with_speed_draft(
    projection: Option<&SelectionProjection>,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    speed_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    view_with_text_draft(
        projection,
        field_draft,
        name_draft,
        speed_draft,
        None,
        dims,
        colors,
    )
}

/// [`view_with_speed_draft`] と同じだが、TEXT section(B46 第1切片、裁定184)の
/// 編集下書きも渡せる。`motolii_shell::Shell::view` はこちらを呼ぶ。
/// **`view`/`view_with_speed_draft` 自身のシグネチャは変えていない**(既存
/// 呼び出し元・ALLOWLIST 外のテストを無改修のまま通すため、`view_with_speed_draft`
/// が導入された時と同じ判断)— どちらも `text_field_draft: None` で
/// ここへ委譲するだけ。
pub fn view_with_text_draft(
    projection: Option<&SelectionProjection>,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    speed_draft: Option<&str>,
    text_field_draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    view_with_color_draft(
        projection,
        field_draft,
        name_draft,
        speed_draft,
        text_field_draft,
        None,
        dims,
        colors,
    )
}

/// [`view_with_text_draft`] と同じだが、TEXT section の色エディタ
/// (`crate::color`、2026-08-22 発注)の編集下書きも渡せる。
/// `motolii_shell::Shell::view` はこちらを呼ぶ。**`view`/`view_with_speed_draft`/
/// `view_with_text_draft` 自身のシグネチャは変えていない**(既存呼び出し元・
/// ALLOWLIST 外のテストを無改修のまま通すため、上2つが導入された時と同じ
/// 判断)— 3つとも `color_field_draft: None` でここへ委譲するだけ。
pub fn view_with_color_draft(
    projection: Option<&SelectionProjection>,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    speed_draft: Option<&str>,
    text_field_draft: Option<&TextFieldDraft>,
    color_field_draft: Option<&color::ColorFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    // mock v3.1 の `.ptitle`("Inspector" 帯)は転写しない — pane 名の正本は
    // shell の pane 題帯(pane_grid title_bar、drag ハンドル兼任)へ移った。
    // 内部にも残すと "Inspector" が二重表示になる(題帯レーンの API 要求)。
    // mock 側の追随(ptitle 行の除去 or 注記)は supervisor キュー。
    let body: Element<'static, Message> = match projection {
        None => empty_state(dims, colors),
        Some(selection) => selected_body(
            selection,
            field_draft,
            name_draft,
            speed_draft,
            text_field_draft,
            color_field_draft,
            dims,
            colors,
        ),
    };

    // 線化 D5(裁定179 文法1): 容器の輪郭線は廃止 — `surface_panel` の面が
    // app 地から明度1段浮くことが pane の輪郭(`chrome::panel_container_style`
    // doc 参照、透明 border で幅だけ残す=幾何不変)。
    container(body)
        .width(Length::Fixed(dims.inspector_panel_width))
        .height(Length::Fill)
        .style(move |_theme| panel_container_style(dims, colors))
        .into()
}

/// **Q0**: 選択なし時は死に chrome を出さない(効かない行を並べない) — 文言1つだけ。
fn empty_state(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    container(
        iced_text("選択なし — layer を選ぶと Transform / Attrs が並ぶ")
            .size(dims.caption_text)
            .color(colors.text_muted),
    )
    .padding(dims.spacing_m)
    .into()
}

/// **視覚正本 `next/reference/mocks/ui-scale-and-z.html` の構造をそのまま写す**:
/// ident 帯 → column header 行 → TRANSFORM(Position/Scale/Rotation/Anchor)→
/// APPEARANCE(Opacity)→ ATTRS(Blend)→ hint 行。`selection.transform` 自体の
/// 並び(既存 `inspector_drive.rs` が固定している)は変えず、view 側で
/// ラベルによって TRANSFORM/APPEARANCE の見出しへ振り分けるだけ。
fn selected_body(
    selection: &SelectionProjection,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    speed_draft: Option<&str>,
    text_field_draft: Option<&TextFieldDraft>,
    color_field_draft: Option<&color::ColorFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut rows = column![
        ident_band(selection, name_draft, dims, colors),
        column_header_row(dims, colors),
        section_header("TRANSFORM", dims, colors),
    ];
    for row_projection in selection.transform.iter().filter(|r| r.label != "Opacity") {
        rows = rows.push(transform_row(row_projection, field_draft, dims, colors));
    }
    rows = rows.push(section_header("APPEARANCE", dims, colors));
    for row_projection in selection.transform.iter().filter(|r| r.label == "Opacity") {
        rows = rows.push(transform_row(row_projection, field_draft, dims, colors));
    }
    rows = rows.push(attrs_section(&selection.attrs, speed_draft, dims, colors));
    // MASK section(B02 第1切片、裁定184): mask を持つ layer でのみ現れる —
    // 空の section header を出さない(Q0: 効かない chrome を並べない)。
    // 既存 section 文法(fold 機構は無い = 常に開いている、既定開と同義)。
    if !selection.masks.is_empty() {
        rows = rows.push(mask_section(&selection.masks, field_draft, dims, colors));
    }
    // EFFECTS section(B38 第3切片、裁定184 型別 section 第2号): effect を持つ
    // layer でのみ現れる(MASK section と同じ Q0 判断)。
    if !selection.effects.is_empty() {
        rows = rows.push(effects_section(&selection.effects, field_draft, dims, colors));
    }
    // TEXT section(B46 第1切片、裁定184): `LayerSource::Text` の layer での
    // み現れる(`project` が `Some` を作るのも同じ layer 種別に限る)。
    if let Some(text_projection) = &selection.text {
        rows = rows.push(text_section(
            text_projection,
            text_field_draft,
            color_field_draft,
            dims,
            colors,
        ));
    }
    // AUDIO section(B42、裁定184 型別 section 第4号): `LayerSource::Media`
    // の layer でのみ現れる(`project` が `Some` を作るのも同じ layer 種別に
    // 限る、TEXT section と同じ判断)。
    if let Some(audio_projection) = &selection.audio {
        rows = rows.push(audio_section(audio_projection, field_draft, dims, colors));
    }
    // LINK section(2026-08-22 発注「レイヤーを指す」文法 第3号): masks/effects
    // と違い「無ければ出さない」の Q0 判断は適用しない ── どの layer でも
    // 他 layer の標準 property を指せて良いはずなので常に現れる
    // (`selection.links` は `LinkTarget::ALL` 分、常に5行)。
    rows = rows.push(link_section(&selection.links, dims, colors));
    rows = rows.push(hint_row(dims, colors));

    scrollable(rows).height(Length::Fill).into()
}

/// mock の `.ident` 帯: 名前(編集可)+ 種別(読み取り専用)+ M/S glyph。
///
/// **M は結線する**(supervisor 訂正、2026-08-20): `LayerAttrs.hidden` は既に
/// `Intent::SetAttrs` で動いている(旧「Hidden [On/Off]」行と同じ Message)。
/// M glyph へ置き換えることで重複 chrome を残さない。
///
/// **S はまだ出さない**: solo(非 solo 層を描かない)は engine/store 未実装
/// (別レーンが実装中)。幅の予約だけ([`reserved_glyph`]、内容も border も無い —
/// 「押せそうに見えて押せない」を Q0 的に作らない)。
fn ident_band(
    selection: &SelectionProjection,
    name_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let name_text = name_draft
        .map(|draft| draft.to_owned())
        .unwrap_or_else(|| selection.attrs.name.clone());
    let placeholder = format!("layer {}", selection.layer.0);

    // 名前欄は mock の `.ident b`(font-weight:600, t-base)の役目を持つが、
    // 実体は `text_input`(既に結線済みの改名 —
    // `Message::NameInput/NameSubmit`)。未フォーカス時は枠を消して
    // 静止テキストに見せる([`name_input_style`])。`.font(TextWeight::
    // Semibold)` で mock の 600 を写す(裁定137)。
    //
    // padding は縦0を維持(`value_cell` と同じ柵発見 — 既定 padding 5px が
    // 乗ると ident 帯の高さが mock の「b(11px)+s(9px)を2行積んだだけ」より
    // 約10px 余計に伸びる、実測: 修正前 name_field 高 24.3px、修正後
    // 14.3px)。**横だけ** [`name_field_padding`] で戻す(裁定139)。
    // 裁定170 M01: fork の text_input は借用寿命を返り値に縛る
    // (Fragment::Borrowed)ため、'static 返却には owned move が要る
    // (値は不変、clone 済みの String を渡すだけ)。
    let name_field = text_input(placeholder, name_text)
        .on_input(Message::NameInput)
        .on_submit(Message::NameSubmit)
        .size(dims.body_text)
        .font(TextWeight::Semibold.font())
        .padding(name_field_padding(dims))
        .style(move |_theme, status| name_input_style(dims, colors, status));

    // mock `.ident s{color:var(--ink2)}` — 旧実装は ink3(`text_muted`)を
    // 誤用していた(2026-08-21 更正)。
    let subtitle = iced_text(selection.kind)
        .size(dims.caption_text)
        .color(Ink::Secondary.resolve(&colors));

    let identity = column![name_field, subtitle]
        .spacing(0.0)
        .width(Length::Fill);

    let glyphs = row_widget![
        mute_glyph(dims, colors, selection.attrs.hidden),
        reserved_glyph(dims),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    container(
        row_widget![
            label_color_chip(selection.attrs.label_color, dims, colors),
            identity,
            glyphs
        ]
        .spacing(dims.spacing_s)
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding([dims.spacing_s, dims.spacing_m])
    .style(move |_theme| container::Style {
        // 線化 D5(裁定179 文法1): `surface_raised` の面が `surface_panel`
        // pane 地から明度1段浮く — 輪郭線は透明化(幅だけ残す=幾何不変)。
        background: Some(iced::Background::Color(colors.surface_raised)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    // `parse_number` は裁定160 切片5(pane split survey §2.4/§6)で
    // `chrome::parse_number` へ関数本体を移設し、切片9で `chrome` ごと
    // `motolii-settings-pane` crate へ、切片8でこの crate 自身の依存先へ
    // 移った(モジュール冒頭の `use motolii_settings_pane::chrome::{parse_number,
    // ..}` で読み込み済み、`use super::*;` 経由でここへ入る)。テストの
    // qualified name(`tests::parse_number_accepts_the_mock_minus_sign`)は
    // `--list` 完全一致のためここに残す — 呼ぶ本体だけ移設先を指す。

    // -----------------------------------------------------------------------
    // BL2: Blend 巡回ボタンの次の値
    // -----------------------------------------------------------------------

    /// Normal→Add→Multiply→…→Exclusion→Normal と、宣言順どおり13値を一周して戻る
    /// (**BL3** — `SUPPORTED_BLEND_MODES` を更新したらこのテストが長さのズレを拾う)。
    #[test]
    fn cycles_through_supported_modes_and_wraps() {
        use motolii_store::BlendMode;
        assert_eq!(SUPPORTED_BLEND_MODES.len(), 13);
        assert_eq!(next_blend_mode(BlendMode::Normal), BlendMode::Add);
        assert_eq!(next_blend_mode(BlendMode::Add), BlendMode::Multiply);
        assert_eq!(next_blend_mode(BlendMode::Multiply), BlendMode::Screen);
        assert_eq!(next_blend_mode(BlendMode::Screen), BlendMode::Overlay);
        assert_eq!(next_blend_mode(BlendMode::Overlay), BlendMode::Darken);
        assert_eq!(next_blend_mode(BlendMode::Darken), BlendMode::Lighten);
        assert_eq!(next_blend_mode(BlendMode::Lighten), BlendMode::ColorDodge);
        assert_eq!(next_blend_mode(BlendMode::ColorDodge), BlendMode::ColorBurn);
        assert_eq!(next_blend_mode(BlendMode::ColorBurn), BlendMode::HardLight);
        assert_eq!(next_blend_mode(BlendMode::HardLight), BlendMode::SoftLight);
        assert_eq!(next_blend_mode(BlendMode::SoftLight), BlendMode::Difference);
        assert_eq!(next_blend_mode(BlendMode::Difference), BlendMode::Exclusion);
        assert_eq!(next_blend_mode(BlendMode::Exclusion), BlendMode::Normal);
    }

    /// 現在値が非対応(将来の下位互換ケース)なら、エラーにせず一覧の先頭へ。
    /// **BL3** で Multiply は対応済みになったので、まだ非対応な非分離4種(BL4)の
    /// `Hue` で確かめる。
    #[test]
    fn unsupported_current_value_falls_back_to_the_first_supported_mode() {
        use motolii_store::BlendMode;
        assert_eq!(next_blend_mode(BlendMode::Hue), BlendMode::Normal);
    }

    // -----------------------------------------------------------------------
    // B38 第3切片: Glow param カタログ(engine 同期義務の柵)
    // -----------------------------------------------------------------------

    /// カタログの3点固定: 名前は engine `translate_glow_params` の `find` 名、
    /// 既定値は engine の `GLOW_DEFAULT_*`(private const)の写し —
    /// [`SUPPORTED_BLEND_MODES`] と同じ二重化なので、engine 側を変えたら
    /// ここが red になって同期漏れを拾う(値の正本は engine 側)。
    #[test]
    fn the_glow_param_catalog_mirrors_the_engine_names_and_defaults() {
        assert_eq!(GlowParam::ALL.len(), 3);
        assert_eq!(GlowParam::Threshold.name(), "threshold");
        assert_eq!(GlowParam::Intensity.name(), "intensity");
        assert_eq!(GlowParam::Radius.name(), "radius");
        assert_eq!(GlowParam::Threshold.default_value(), 1.0);
        assert_eq!(GlowParam::Intensity.default_value(), 0.75);
        assert_eq!(GlowParam::Radius.default_value(), 1.0);
    }

    /// 既知 plugin(`motolii.glow`)だけカタログと表示名を持ち、未知は
    /// param 行ゼロ + plugin_id そのまま(M13: 捏造しない)。
    #[test]
    fn plugin_catalog_and_display_name_are_honest_about_unknown_plugins() {
        assert_eq!(plugin_params(GLOW_PLUGIN_ID).len(), 3);
        assert!(plugin_params("third-party.sparkle").is_empty());
        assert_eq!(plugin_display_name(GLOW_PLUGIN_ID), "Glow");
        assert_eq!(
            plugin_display_name("third-party.sparkle"),
            "third-party.sparkle"
        );
    }

    /// effect param の field/KeyRow → property の対応が
    /// `effect.{id}.param.{name}` に落ちる(mask opacity の対応固定と同型)。
    #[test]
    fn effect_param_fields_and_key_rows_map_to_the_flat_effect_property() {
        let expected =
            PropertyId::effect_param(EffectId(7), "radius").expect("param 名は非予約語");
        assert_eq!(
            property_id(TransformField::EffectParam(EffectId(7), GlowParam::Radius))
                .expect("作れるはず"),
            expected
        );
        assert_eq!(
            key_row_property_id(KeyRow::EffectParam(EffectId(7), GlowParam::Radius))
                .expect("作れるはず"),
            expected
        );
        assert_eq!(
            key_row_default_value(KeyRow::EffectParam(EffectId(7), GlowParam::Intensity)),
            Value::F64(0.75),
            "Key 列の初キー値も engine 既定の写しのはず"
        );
    }

    /// ラベル色チップの1辺は timeline rail のチップ式(`round(0.462 × 行高)`)
    /// と同じで、**`inspector_glyph_width`(26px)とは一致しない** — shell 側
    /// `inspector_pixel_fence` の glyph 数え上げ(M 1個 + Key 5個 = 6個)を
    /// 壊さないための幾何の柵。
    #[test]
    fn the_label_chip_side_follows_the_timeline_swatch_formula_not_the_glyph_width() {
        let dims = Dimensions::default();
        let side = label_chip_side(dims.inspector_row_height);
        assert_eq!(side, (dims.inspector_row_height * 0.462).round());
        assert_ne!(
            side, dims.inspector_glyph_width,
            "チップが glyph 幅と同寸だと pixel fence の数え上げに紛れ込む"
        );
        assert_ne!(
            side,
            glyph_height(dims),
            "チップ高が glyph 高と同じでも幅26px側の柵対象になり得る(正方形なので両辺を外す)"
        );
    }

    // -----------------------------------------------------------------------
    // SP1 第一波: %⇄Speed 写像(ORACLE (b))
    // -----------------------------------------------------------------------

    /// 往復: 表示 % → (num, den) → 表示 % が同じ値へ戻る(小数1桁)。
    #[test]
    fn percent_round_trips_through_speed_ratio() {
        let (num, den) = percent_to_speed_ratio(200.0).unwrap();
        assert_eq!(format_number(speed_percent(num, den), 1), "200.0");

        let (num, den) = percent_to_speed_ratio(133.3).unwrap();
        assert_eq!(format_number(speed_percent(num, den), 1), "133.3");

        let (num, den) = percent_to_speed_ratio(50.0).unwrap();
        assert_eq!(format_number(speed_percent(num, den), 1), "50.0");
    }

    /// 分母は常に正(`Speed::try_new` の不変式を機械的に満たす)。
    #[test]
    fn speed_ratio_denominator_is_always_positive() {
        let (_, den) = percent_to_speed_ratio(100.0).unwrap();
        assert!(den > 0);
    }

    /// **0 は拒否**(決定3)。負・NaN・無限大も同様。
    #[test]
    fn non_positive_or_non_finite_percent_is_rejected() {
        assert_eq!(percent_to_speed_ratio(0.0), None);
        assert_eq!(percent_to_speed_ratio(-5.0), None);
        assert_eq!(percent_to_speed_ratio(f64::NAN), None);
        assert_eq!(percent_to_speed_ratio(f64::INFINITY), None);
    }

    /// 100% は `Speed::NORMAL`(1/1)と同じ比。
    #[test]
    fn one_hundred_percent_is_normal_speed() {
        let (num, den) = percent_to_speed_ratio(100.0).unwrap();
        assert_eq!(num as f64 / den as f64, 1.0);
    }

    // -----------------------------------------------------------------------
    // 裁定139/裁定168: value_cell/name_field は縦0を維持したまま横だけ内余白
    // (0.6em)を戻す
    // -----------------------------------------------------------------------

    /// **本命(red→green の柵)**: 旧実装は `.padding(0.0)` で縦横とも0だった
    /// (`git log` 参照 — このテストを旧コードに当てると
    /// `padding.left == 0.0`/`padding.right == 0.0` が真になり fail する)。
    /// 縦は行高合わせのため0のまま、横だけ 裁定168 の `0.6em`
    /// (`dims.body_text * 0.6` の最近傍丸め)が入っていること。旧実装は
    /// `spacing_xs`(mock `--sp1`=2px)を転用していたが、裁定168 施工で
    /// この式へ差し替えた(既定 dims では 11*0.6=6.6→7.0px、旧値2pxより広い)。
    #[test]
    fn value_cell_padding_keeps_the_vertical_zero_and_restores_only_horizontal_inset() {
        let dims = Dimensions::default();
        let padding = value_cell_padding(dims);
        let expected = single_row_horizontal_inset(dims.body_text);
        assert_eq!(padding.top, 0.0, "縦(上)は行高合わせのため0のはず");
        assert_eq!(padding.bottom, 0.0, "縦(下)は行高合わせのため0のはず");
        assert_eq!(
            padding.left, expected,
            "横(左)の内余白が裁定168 の0.6emと違う"
        );
        assert_eq!(
            padding.right, expected,
            "横(右)の内余白が裁定168 の0.6emと違う"
        );
        assert!(padding.left > 0.0, "横の内余白が0のまま(旧バグの再発)");
        assert_eq!(
            expected, 7.0,
            "既定dims(body_text=11)での0.6em丸め値が想定と違う"
        );
    }

    #[test]
    fn name_field_padding_matches_value_cell_padding_the_same_way() {
        let dims = Dimensions::default();
        assert_eq!(name_field_padding(dims), value_cell_padding(dims));
    }

    /// 150%でも横内余白がスケールに追従すること(適用点は `Dimensions::scaled`
    /// の1箇所だけ、という裁定117の不変量をここでも保つ)。**丸めは
    /// スケール後の `body_text` に対して1回だけ行う** — 丸め前の値を先に
    /// スケールしてから丸めるのと数値が一致するとは限らない(丸めの非線形性、
    /// 既定 dims では 7.0*1.5=10.5 だが実際は round(16.5*0.6)=round(9.9)=10.0)。
    #[test]
    fn value_cell_padding_scales_with_ui_scale() {
        let dims = Dimensions::default().scaled(1.5);
        let padding = value_cell_padding(dims);
        assert_eq!(padding.left, single_row_horizontal_inset(dims.body_text));
    }

    // -----------------------------------------------------------------------
    // 裁定137: weight/ink の実使用箇所(.glyph/.ident)
    // -----------------------------------------------------------------------

    #[test]
    fn mute_glyph_uses_bold_800_weight() {
        // `mute_glyph` 自体は `Element` を返すので font を直接読み出せない —
        // `iced_selector::Target`(`Container`/`TextInput`/…)は style(色/font
        // /padding)を一切運ばない実測(`tests/inspector_pixel_fence.rs` 冒頭
        // 参照)なので、iced_test 経由でも実配線の font weight は照合できない。
        // ここは token 側の対応(`TextWeight::Bold` = 800)を固定するだけの
        // 薄い柵 — 実際に `.font(TextWeight::Bold.font())` へ繋がっている
        // ことは呼び出し箇所のコードレビュー相当でしか確認できない、正直な
        // 限界(`--screenshot` 器具は Stage+Timeline のみの手組み合成で
        // Inspector を一切描かない — `screenshot.rs` 実測、write-set 外の
        // finding として最終報告に記録)。
        assert_eq!(
            TextWeight::Bold.font().weight,
            iced::font::Weight::ExtraBold
        );
    }

    #[test]
    fn parse_number_accepts_the_mock_minus_sign() {
        assert_eq!(parse_number("−0.075"), Some(-0.075));
        assert_eq!(parse_number("12.5"), Some(12.5));
        assert_eq!(parse_number("  3  "), Some(3.0));
        assert_eq!(parse_number("not a number"), None);
    }

    #[test]
    fn format_number_respects_decimals() {
        assert_eq!(format_number(1.0, 3), "1.000");
        assert_eq!(format_number(24.0, 1), "24.0");
        assert_eq!(format_number(100.0, 0), "100");
    }

    /// 裁定169: 表示はセルに収まる精度へ落ちる(編集 draft は全精度のまま —
    /// [`value_cell`] の editing 分岐が `format_number` 直呼びであることが対)。
    /// **I-tokens(2026-08-22)で cap を6→11へ再較正** — `inspector_value_width`
    /// が38→64pxへ束で再転写されたため、旧アンカー値(960/3840)はもう clip の
    /// 実例にならない(64px セルなら全精度のまま収まる)。[`MAX_VALUE_CELL_CHARS`]
    /// のdoc に記載の新アンカー(11字/12字)に合わせて例を差し替える。
    #[test]
    fn display_number_shrinks_precision_to_fit_the_cell() {
        // 収まる値は field 既定精度のまま。
        assert_eq!(display_number(1.0, 3), "1.000");
        assert_eq!(display_number(0.0, 3), "0.000");
        // 旧アンカー値(960/3840)は新セル幅(64px)では clip されず全精度のまま
        // 収まる(旧cap=6時代は "960.00"/"3840.0"/"-960.0" へ短縮していた)。
        assert_eq!(display_number(960.0, 3), "960.000");
        assert_eq!(display_number(3840.0, 3), "3840.000");
        assert_eq!(display_number(-960.0, 3), "-960.000");
        // 新アンカー(MAX_VALUE_CELL_CHARS のdoc参照): 整数部7桁+小数3桁=11字は
        // ちょうど cap に収まる(境界そのもの)。
        assert_eq!(display_number(1234567.0, 3), "1234567.000");
        // 整数部8桁+小数3桁=12字は cap を超える → 小数を1桁落として11字へ。
        assert_eq!(display_number(12345678.0, 3), "12345678.00");
        // 整数部だけで上限超え: これ以上落とせない(clip(true) が防波堤)
        assert_eq!(display_number(123456789012.0, 0), "123456789012");
    }

    #[test]
    fn next_value_preserves_the_other_vec2_component() {
        assert_eq!(
            next_value(TransformField::PositionX, 5.0, [1.0, 2.0]),
            Value::Vec2([5.0, 2.0])
        );
        assert_eq!(
            next_value(TransformField::PositionY, 5.0, [1.0, 2.0]),
            Value::Vec2([1.0, 5.0])
        );
    }

    #[test]
    fn next_value_converts_opacity_percent_to_the_stored_fraction() {
        assert_eq!(
            next_value(TransformField::Opacity, 50.0, [0.0, 0.0]),
            Value::F64(0.5)
        );
        // クランプ: 100 を超える入力・負の入力は store の 0..1 に収める。
        assert_eq!(
            next_value(TransformField::Opacity, 150.0, [0.0, 0.0]),
            Value::F64(1.0)
        );
        assert_eq!(
            next_value(TransformField::Opacity, -10.0, [0.0, 0.0]),
            Value::F64(0.0)
        );
    }

    #[test]
    fn single_hold_track_has_exactly_one_hold_keyframe() {
        let track = single_hold_track(Value::F64(2.5));
        assert_eq!(track.keys().len(), 1, "静的値は1キーのはず");
        assert_eq!(track.keys()[0].value, Value::F64(2.5));
        assert!(matches!(track.keys()[0].interp, Interp::Hold));
    }

    #[test]
    fn default_vec2_is_identity_scale_and_zero_elsewhere() {
        assert_eq!(default_vec2(TransformField::ScaleX), [1.0, 1.0]);
        assert_eq!(default_vec2(TransformField::PositionX), [0.0, 0.0]);
        assert_eq!(default_vec2(TransformField::AnchorY), [0.0, 0.0]);
    }

    /// ident 帯の種別ラベルは `LayerSource` の実 variant から引く(mock の
    /// 「shared FX」件数のような捏造値ではない)。
    #[test]
    fn source_kind_label_covers_every_layer_source_variant() {
        assert_eq!(
            source_kind_label(&LayerSource::Solid {
                rgba: [0, 0, 0, 255],
                width: 1,
                height: 1,
            }),
            "solid"
        );
        assert_eq!(
            source_kind_label(&LayerSource::Media {
                path: "x.mp4".to_owned(),
                fingerprint: None,
            }),
            "media"
        );
        assert_eq!(source_kind_label(&LayerSource::Null), "null");
        assert_eq!(source_kind_label(&LayerSource::Shape), "shape");
        assert_eq!(source_kind_label(&LayerSource::Text), "text");
        assert_eq!(source_kind_label(&LayerSource::Group), "group");
    }

    // -----------------------------------------------------------------------
    // drag-to-scrub — 感度表(発注書の表そのもの)
    // -----------------------------------------------------------------------

    #[test]
    fn dragged_value_applies_the_registry_sensitivity_per_field() {
        // Position/Anchor/Z = 1px→1.0。
        assert_eq!(
            dragged_value(TransformField::PositionX, 0.0, 10.0, false),
            10.0
        );
        assert_eq!(
            dragged_value(TransformField::AnchorY, 0.0, -4.0, false),
            -4.0
        );
        assert_eq!(
            dragged_value(TransformField::PositionZ, 0.0, 3.0, false),
            3.0
        );
        // Scale = 1px→0.01。
        assert!((dragged_value(TransformField::ScaleX, 1.0, 10.0, false) - 1.1).abs() < 1e-9);
        // Rotation = 1px→0.5度。
        assert!((dragged_value(TransformField::Rotation, 0.0, 10.0, false) - 5.0).abs() < 1e-9);
        // Opacity = 1px→1(%)。
        assert_eq!(
            dragged_value(TransformField::Opacity, 50.0, 20.0, false),
            70.0
        );
    }

    #[test]
    fn shift_drag_uses_a_tenth_of_the_normal_sensitivity() {
        let normal = dragged_value(TransformField::PositionX, 0.0, 100.0, false);
        let fine = dragged_value(TransformField::PositionX, 0.0, 100.0, true);
        assert_eq!(normal, 100.0);
        assert!(
            (fine - 10.0).abs() < 1e-9,
            "Shift+drag は1/10のはず: {fine}"
        );
    }

    #[test]
    fn drag_origin_reads_the_projected_value_and_keeps_the_other_vec2_component() {
        // Scale の既定(un-keyed)は X=Y=1.0 — X をドラッグ対象にしても
        // current_vec2 の Y は保たれる。
        let selection = SelectionProjection {
            layer: LayerId(1),
            kind: "solid",
            transform: vec![TransformRowProjection {
                label: "Scale",
                value: RowValue::Vector([
                    ComponentSlot {
                        axis: "X",
                        present: true,
                        value: 1.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::ScaleX),
                    },
                    ComponentSlot {
                        axis: "Y",
                        present: true,
                        value: 2.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::ScaleY),
                    },
                    absent_component("Z"),
                ]),
                decimals: 3,
                key: KeyCellProjection {
                    row: KeyRow::Scale,
                    state: KeyCellState::Static,
                },
            }],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
                speed_percent: 100.0,
                label_color: None,
                matte: None,
                matte_candidates: vec![],
            },
            masks: vec![],
            effects: vec![],
            text: None,
            audio: None,
            links: vec![],
        };

        let (start, current_vec2) =
            drag_origin(&selection, TransformField::ScaleX).expect("editable のはず");
        assert_eq!(start, 1.0);
        assert_eq!(current_vec2, [1.0, 2.0], "動かさない方(Y)を保っていない");

        // 対応する field が投影に無ければ `None`(呼び手はドラッグを始めない)。
        assert!(drag_origin(&selection, TransformField::Rotation).is_none());
    }

    /// キー持ち(keyed)の field も drag/type 編集の起点になる(Q0 —
    /// 2026-08-22 発注で旧規則「animated は編集不可」を撤去。編集の意味は
    /// [`edited_value_track`] のキー upsert)。
    #[test]
    fn drag_origin_accepts_keyed_fields() {
        let selection = SelectionProjection {
            layer: LayerId(1),
            kind: "solid",
            transform: vec![TransformRowProjection {
                label: "Rotation",
                value: RowValue::Vector([
                    absent_component("X"),
                    absent_component("Y"),
                    ComponentSlot {
                        axis: "Z",
                        present: true,
                        value: 45.0,
                        editable: true,
                        keyed: true, // 実キー持ち(旧 animated)
                        field: Some(TransformField::Rotation),
                    },
                ]),
                decimals: 1,
                key: KeyCellProjection {
                    row: KeyRow::Rotation,
                    state: KeyCellState::Between,
                },
            }],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
                speed_percent: 100.0,
                label_color: None,
                matte: None,
                matte_candidates: vec![],
            },
            masks: vec![],
            effects: vec![],
            text: None,
            audio: None,
            links: vec![],
        };
        let (start, _) = drag_origin(&selection, TransformField::Rotation)
            .expect("キー持ち field もドラッグを始められるはず(Q0)");
        assert_eq!(start, 45.0, "起点は投影の評価値のはず");
    }

    #[test]
    fn field_decimals_matches_the_projection_rows() {
        assert_eq!(field_decimals(TransformField::PositionX), 3);
        assert_eq!(field_decimals(TransformField::ScaleY), 3);
        assert_eq!(field_decimals(TransformField::AnchorX), 3);
        assert_eq!(field_decimals(TransformField::Rotation), 1);
        assert_eq!(field_decimals(TransformField::Opacity), 0);
    }

    // -----------------------------------------------------------------------
    // 裁定168 施工: 値セル間 gap(裁定167 下段)
    // -----------------------------------------------------------------------

    #[test]
    fn sibling_gap_px_matches_the_ladder_bottom_rung_rounded_to_the_nearest_pixel() {
        // `motolii-timeline-pane::lane_bar::sibling_gap_px` と同じ式・同じ期待値
        // (既定 inspector_row_height=20 でも一致する — 意図的に同じ token 段を
        // 使っているため、値の一致は式が揃っている検算にもなる)。
        assert_eq!(sibling_gap_px(20.0), 2.0);
        assert_eq!(sibling_gap_px(40.0), 3.0);
    }

    #[test]
    fn transform_row_widens_the_value_cell_gap_beyond_the_old_spacing_xs_token_at_a_larger_row_height(
    ) {
        // 既定 dims(row_height=20)では旧 `spacing_xs`(2px)と新式(round(1.5)=2px)
        // が偶然一致してしまい、この2つの違いを既定 dims だけでは検分できない
        // (`sibling_gap_px` 自体は独立式であることを別テストで固定済み)。
        // ここでは inspector_row_height を人為的に変えた `Dimensions` で
        // 「gap は `inspector_row_height` に追従し、`spacing_xs` には追従しない」
        // ことを確かめる — token 借用ではなく専用式になっている証拠。
        let dims = Dimensions {
            inspector_row_height: 40.0,
            ..Dimensions::default()
        };
        assert_eq!(sibling_gap_px(dims.inspector_row_height), 3.0);
        assert_ne!(
            sibling_gap_px(dims.inspector_row_height),
            dims.spacing_xs,
            "gap が旧トークン(spacing_xs)のままでは inspector_row_height の変化に追従しない"
        );
    }

    // -----------------------------------------------------------------------
    // 裁定183 taffy 転写(部分適用 — `property_row_css` の doc「FINDING」参照。
    // production 配線は見送ったが、CSS 文字列そのものが `motolii-taffy` の
    // parser で必ず解釈できること・mock の字面と一致することはここで固定する。
    // ±1px oracle 本体は `tests/property_row_taffy_oracle.rs`(モック実測との
    // 突き合わせ)。
    // -----------------------------------------------------------------------

    #[test]
    fn property_row_css_parses_and_declares_the_mock_grid_template() {
        // 裁定183 taffy 転写(部分適用)。production では呼ばない
        // ([`property_row_css`] の doc「FINDING」参照)ので、この crate の
        // 通常ビルドに `motolii_taffy::style_from_css_decl` を引きずらないよう
        // test scope だけで import する(`TaffyBox` はどこからも呼ばれない
        // ため import しない — `motolii-taffy` 自体への Cargo.toml 依存は
        // `tests/property_row_taffy_oracle.rs` が使う)。
        use motolii_taffy::style_from_css_decl;

        let dims = Dimensions::default();
        let css = property_row_css(dims);

        // mock の字面(`inspector-library.html` v3.1 `.columnHeader,.propertyRow`)
        // をそのまま含むこと — CSS 文字列が単一正本という裁定183 の趣旨どおり、
        // 値だけ dims で埋めた形になっているかを直接検算する。
        assert!(
            css.contains("grid-template-columns:minmax(132px,1fr) repeat(3,64px) 26px"),
            "grid-template-columns の字面が mock と食い違う: {css}"
        );

        let style = style_from_css_decl(&css)
            .expect("property_row_css は固定テンプレート+dims の px 値のみを埋める — 解釈は必ず成功する");
        // taffy の `grid_template_columns` は「track 定義の個数」であって展開後の
        // 列数ではない — `repeat(3, 64px)` は1個の `GridTemplateComponent::
        // Repeat` として1トラック扱い(motolii-taffy 側の実測 —
        // `motolii-taffy/tests/css_decl.rs::grid_template_splits_only_outside_parens`
        // が同じ旗艦例文字列で3を固定済み)。よってここは
        // `[minmax(label), repeat(3,64px), 26px]` の3。
        assert_eq!(
            style.grid_template_columns.len(),
            3,
            "track 定義数(label + repeat(3,X/Y/Z) + Key)が3でない"
        );
    }

    // -----------------------------------------------------------------------
    // 裁定168 EXACT TARGET 3: 文字寸検査(柵として固定・現値の乖離は FINDING)
    // -----------------------------------------------------------------------

    /// 裁定168 は「文字寸 = 0.42 × 行高」を単行の余白計算の前提に置く。
    /// **I-tokens(2026-08-22)で根治**: `inspector_row_height` を
    /// `next/reference/mocks/inspector-library.html` v3.1 実測値(25)へ
    /// 束で再転写した結果、`body_text`(11)/`inspector_row_height`(25)= **0.44**
    /// となり、裁定168 の帯(0.42±0.05 = 0.37〜0.47)の**内**に入った
    /// (旧値は 11/20=0.55 で帯の外 — `docs/reviews/
    /// 2026-08-22-inspector-ratio-ledger.md` の FINDING そのもの)。
    ///
    /// このテストは旧 `..._is_locked_at_its_current_out_of_band_value`
    /// (0.55 を固定していた pin)を置き換える —**0.55 の lock は撤去**し、
    /// 「帯の内に入っている」ことを固定する regression lock へ更新した
    /// (どちらかの値が黙って動いて帯の外へ出たら red になる)。両側チェックの
    /// 詳細(モック実測 vs 実装値)は `tests/inspector_ratio_ledger.rs` 側。
    // -----------------------------------------------------------------------
    // K1: Key 列 — 3状態 oracle と click→SetTrack 内容の純関数(落ちるテスト先行)
    // -----------------------------------------------------------------------

    use motolii_core::Fps;

    fn fps30() -> Fps {
        Fps::try_new(30, 1).expect("30fps は正値")
    }

    fn key_at(frame: i64, value: f64, interp: Interp) -> Keyframe {
        Keyframe {
            t: RationalTime::try_from_frame(frame, fps30()).expect("frame→時刻"),
            value: Value::F64(value),
            interp,
            spatial: None,
        }
    }

    fn track_of(keys: Vec<Keyframe>) -> KeyframeTrack {
        let mut track = KeyframeTrack::new();
        for key in keys {
            track.insert(key);
        }
        track
    }

    /// **状態1 oracle**: track 無し=静的。`single_hold_track`(1キー Hold @ZERO、
    /// この crate の静的値の正準表現)も同じ「静的」— Inspector の静的値編集が
    /// 書いた track を「キーが打たれている」と誤読しない。
    #[test]
    fn key_cell_state_is_static_without_a_track_and_for_the_canonical_static_track() {
        assert_eq!(key_cell_state(None, 0, fps30()), KeyCellState::Static);
        let static_track = single_hold_track(Value::F64(2.5));
        assert_eq!(
            key_cell_state(Some(&static_track), 0, fps30()),
            KeyCellState::Static,
            "正準静的表現(1キー Hold @ZERO)は playhead=0 でも静的のはず"
        );
        assert_eq!(key_cell_state(Some(&static_track), 10, fps30()), KeyCellState::Static);
        // 空 track(SetTrack で空を書いた場合の防御)も静的。
        assert_eq!(
            key_cell_state(Some(&KeyframeTrack::new()), 0, fps30()),
            KeyCellState::Static
        );
    }

    /// **状態2/3 oracle**: playhead のフレームにキーが有れば AtKey、track は有るが
    /// そのフレームにキーが無ければ Between。照合は timeline と同じ
    /// `try_to_frame_round`(frame 粒度)。
    #[test]
    fn key_cell_state_distinguishes_at_key_and_between() {
        let track = track_of(vec![
            key_at(10, 0.0, Interp::Linear),
            key_at(20, 5.0, Interp::Linear),
        ]);
        assert_eq!(key_cell_state(Some(&track), 10, fps30()), KeyCellState::AtKey);
        assert_eq!(key_cell_state(Some(&track), 20, fps30()), KeyCellState::AtKey);
        assert_eq!(key_cell_state(Some(&track), 15, fps30()), KeyCellState::Between);
        assert_eq!(
            key_cell_state(Some(&track), 0, fps30()),
            KeyCellState::Between,
            "track の範囲外でも track が有る限り Between(半表示)のはず"
        );
        // 1キーでも正準静的形(Hold @ZERO)でなければ本物のキー。
        let single_linear = track_of(vec![key_at(10, 1.0, Interp::Linear)]);
        assert_eq!(key_cell_state(Some(&single_linear), 10, fps30()), KeyCellState::AtKey);
        assert_eq!(key_cell_state(Some(&single_linear), 11, fps30()), KeyCellState::Between);
    }

    /// **状態1 click**: 現在の静的値で playhead 時刻にキー1個(track 先頭 insert は
    /// Linear)。静的 hold track が既に有ればその値、無ければ呼び手の現在値。
    #[test]
    fn toggling_from_static_creates_one_linear_key_at_the_playhead() {
        // track 無し → 呼び手が渡す現在値(既定値)で作る。
        let new = toggled_key_track(None, 12, fps30(), Value::Vec2([1.0, 1.0]))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 1);
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(12, fps30()).unwrap());
        assert_eq!(new.keys()[0].value, Value::Vec2([1.0, 1.0]));
        assert!(matches!(new.keys()[0].interp, Interp::Linear), "track 先頭 insert は Linear のはず");

        // 正準静的 track 有り → その track の値(呼び手の現在値ではなく)。
        let static_track = single_hold_track(Value::F64(2.5));
        let new = toggled_key_track(Some(&static_track), 12, fps30(), Value::F64(999.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 1);
        assert_eq!(new.keys()[0].value, Value::F64(2.5), "静的 hold track の値が正のはず");
        assert!(matches!(new.keys()[0].interp, Interp::Linear));
    }

    /// **状態2 click(キー2個以上)**: playhead 上のキーだけを除去し、他は保つ。
    #[test]
    fn toggling_on_a_key_removes_only_that_key() {
        let track = track_of(vec![
            key_at(10, 0.0, Interp::Linear),
            key_at(20, 5.0, Interp::Hold),
        ]);
        let new = toggled_key_track(Some(&track), 10, fps30(), Value::F64(0.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 1);
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(20, fps30()).unwrap());
        assert_eq!(new.keys()[0].value, Value::F64(5.0));
        assert!(matches!(new.keys()[0].interp, Interp::Hold), "残るキーの interp を変えない");
    }

    /// **状態2 click(最後の1個)**: track ごと静的化 — 消したキーの値を保った
    /// 正準静的表現(1キー Hold @ZERO)へ(AE のストップウォッチ解除と等価、
    /// 値は失わない)。
    #[test]
    fn removing_the_last_key_returns_a_static_hold_track_keeping_the_value() {
        let track = track_of(vec![key_at(10, 7.5, Interp::Linear)]);
        let new = toggled_key_track(Some(&track), 10, fps30(), Value::F64(0.0))
            .expect("toggle は成功するはず");
        assert_eq!(new, single_hold_track(Value::F64(7.5)), "値を保った静的化のはず");
        assert_eq!(
            key_cell_state(Some(&new), 10, fps30()),
            KeyCellState::Static,
            "静的化後の状態は Static へ戻るはず"
        );
    }

    /// **状態3 click**: playhead 時刻の**評価値**でキー追加。Interp は直前の
    /// キーの流儀に従い、track 先頭(最初のキーより前)への insert は Linear。
    #[test]
    fn toggling_between_keys_inserts_the_evaluated_value_with_the_neighbor_interp() {
        // Linear 区間の中点 → 評価値は補間の中点、interp は前のキーと同じ Linear。
        let track = track_of(vec![
            key_at(0, 0.0, Interp::Linear),
            key_at(20, 10.0, Interp::Linear),
        ]);
        let new = toggled_key_track(Some(&track), 10, fps30(), Value::F64(999.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 3);
        assert_eq!(new.keys()[1].t, RationalTime::try_from_frame(10, fps30()).unwrap());
        assert_eq!(new.keys()[1].value, Value::F64(5.0), "その時刻の eval 値のはず");
        assert!(matches!(new.keys()[1].interp, Interp::Linear));

        // Hold 区間 → 前のキーの値を保持したまま、interp も Hold を継ぐ。
        let track = track_of(vec![
            key_at(0, 3.0, Interp::Hold),
            key_at(20, 10.0, Interp::Linear),
        ]);
        let new = toggled_key_track(Some(&track), 10, fps30(), Value::F64(999.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys()[1].value, Value::F64(3.0), "Hold 区間の eval は前の値のはず");
        assert!(matches!(new.keys()[1].interp, Interp::Hold), "隣接(前)キーの流儀を継ぐはず");

        // 最初のキーより前への insert → Linear(track 先頭の既定)。
        let track = track_of(vec![key_at(20, 10.0, Interp::Hold)]);
        let new = toggled_key_track(Some(&track), 5, fps30(), Value::F64(999.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 2);
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(5, fps30()).unwrap());
        assert_eq!(new.keys()[0].value, Value::F64(10.0), "範囲外 clamp は端の値のはず");
        assert!(matches!(new.keys()[0].interp, Interp::Linear), "先頭 insert は Linear のはず");
    }

    /// undo 可逆の前提(純関数レベル): AtKey→toggle→(即)toggle で元の意味へ
    /// 戻る(2キー以上)。Document レベルの undo 可逆は shell の drive
    /// (`inspector_key_drive.rs`)が Intent 経由で確かめる。
    #[test]
    fn toggling_twice_on_a_key_round_trips_for_multi_key_tracks() {
        let track = track_of(vec![
            key_at(10, 0.0, Interp::Linear),
            key_at(20, 5.0, Interp::Linear),
        ]);
        let removed = toggled_key_track(Some(&track), 10, fps30(), Value::F64(0.0)).unwrap();
        let restored = toggled_key_track(Some(&removed), 10, fps30(), Value::F64(0.0)).unwrap();
        // 値は eval(範囲外 clamp で端の 0.0…ではなく残キーの 5.0)なので、
        // 復元されるのは「その時刻の評価値のキー」— 時刻集合は元どおり。
        assert_eq!(restored.keys().len(), 2);
        assert_eq!(restored.keys()[0].t, track.keys()[0].t);
        assert_eq!(restored.keys()[1].t, track.keys()[1].t);
    }

    // -----------------------------------------------------------------------
    // 値編集の意味(AE 作法): `edited_value_track` — 静的は静的のまま・
    // キー持ちは playhead へ upsert(2026-08-22 発注)
    // -----------------------------------------------------------------------

    /// キー無し(track 無し・正準静的表現)の値編集は従来どおり静的値の
    /// 書き換え — キーは生えない。
    #[test]
    fn edited_value_track_keeps_static_tracks_static() {
        let new = edited_value_track(None, 15, fps30(), Value::F64(4.0)).unwrap();
        assert_eq!(new, single_hold_track(Value::F64(4.0)));

        let static_track = single_hold_track(Value::F64(1.0));
        let new =
            edited_value_track(Some(&static_track), 15, fps30(), Value::F64(4.0)).unwrap();
        assert_eq!(new, single_hold_track(Value::F64(4.0)), "静的編集でキーが生えている");
        assert_eq!(key_cell_state(Some(&new), 15, fps30()), KeyCellState::Static);
    }

    /// キー持ち track の、playhead にキーが**無い**時刻での編集 = 新キー挿入
    /// (既存キーは無傷・interp は Between 挿入と同規則)。
    #[test]
    fn edited_value_track_inserts_a_new_key_at_the_playhead() {
        let track = track_of(vec![key_at(10, 1.0, Interp::Hold)]);
        let new = edited_value_track(Some(&track), 20, fps30(), Value::F64(3.0)).unwrap();
        assert_eq!(new.keys().len(), 2, "値編集でキーが増えるはず(AE 文法)");
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(10, fps30()).unwrap());
        assert_eq!(new.keys()[0].value, Value::F64(1.0), "既存キーは無傷のはず");
        assert_eq!(new.keys()[1].t, RationalTime::try_from_frame(20, fps30()).unwrap());
        assert_eq!(new.keys()[1].value, Value::F64(3.0));
        assert!(
            matches!(new.keys()[1].interp, Interp::Hold),
            "直前キー(Hold)の流儀を継ぐはず"
        );
        assert!(new.keys()[1].spatial.is_none());

        // 最初のキーより前への挿入は Linear(track 先頭の既定)。
        let new = edited_value_track(Some(&track), 5, fps30(), Value::F64(0.5)).unwrap();
        assert_eq!(new.keys().len(), 2);
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(5, fps30()).unwrap());
        assert!(matches!(new.keys()[0].interp, Interp::Linear), "先頭 insert は Linear のはず");
    }

    /// playhead にキーが**有る**時刻での編集 = そのキーの値だけ更新(個数
    /// 不変・時刻/interp/spatial は保つ)。
    #[test]
    fn edited_value_track_updates_the_key_under_the_playhead_in_place() {
        let track = track_of(vec![
            key_at(10, 1.0, Interp::Hold),
            key_at(20, 5.0, Interp::Linear),
        ]);
        let new = edited_value_track(Some(&track), 10, fps30(), Value::F64(9.0)).unwrap();
        assert_eq!(new.keys().len(), 2, "playhead 上の編集はキー個数を変えないはず");
        assert_eq!(new.keys()[0].value, Value::F64(9.0), "playhead 上のキーの値が更新されるはず");
        assert!(matches!(new.keys()[0].interp, Interp::Hold), "interp は保つはず");
        assert_eq!(new.keys()[1].value, Value::F64(5.0), "他のキーは無傷のはず");
    }

    /// 1キーでも実キー(正準静的表現でない)なら upsert — track を静的に
    /// 戻さない(利用者実窓指摘「キーが1つしか打てない」の機序そのもの:
    /// 旧実装はここで `single_hold_track` に置き換えてキーを消していた)。
    #[test]
    fn edited_value_track_never_collapses_a_real_single_key_track_to_static() {
        let track = track_of(vec![key_at(10, 1.0, Interp::Linear)]);
        let new = edited_value_track(Some(&track), 20, fps30(), Value::F64(2.0)).unwrap();
        assert_eq!(new.keys().len(), 2, "キーが1個へ潰れている(旧バグの再発)");
        assert_ne!(
            key_cell_state(Some(&new), 20, fps30()),
            KeyCellState::Static,
            "実キー持ち track が静的化されている"
        );
    }

    /// KeyRow → property / 既定値の対応表(Position/Scale/Rotation/Opacity/Anchor
    /// の5行全部)。
    #[test]
    fn key_rows_map_to_their_properties_and_defaults() {
        let name_of = |row: KeyRow| key_row_property_id(row).expect("標準 property は作れる");
        assert_eq!(name_of(KeyRow::Position), PropertyId::new(property::POSITION).unwrap());
        assert_eq!(name_of(KeyRow::Scale), PropertyId::new(property::SCALE).unwrap());
        assert_eq!(name_of(KeyRow::Rotation), PropertyId::new(property::ROTATION).unwrap());
        assert_eq!(name_of(KeyRow::Opacity), PropertyId::new(property::OPACITY).unwrap());
        assert_eq!(name_of(KeyRow::Anchor), PropertyId::new(property::ANCHOR).unwrap());
        // mask 行は id から動的に決まる(`PropertyId::mask_opacity` が正本)。
        assert_eq!(
            name_of(KeyRow::MaskOpacity(MaskId(7))),
            PropertyId::mask_opacity(MaskId(7))
        );

        assert_eq!(key_row_default_value(KeyRow::Position), Value::Vec2([0.0, 0.0]));
        assert_eq!(key_row_default_value(KeyRow::Scale), Value::Vec2([1.0, 1.0]));
        assert_eq!(key_row_default_value(KeyRow::Rotation), Value::F64(0.0));
        assert_eq!(key_row_default_value(KeyRow::Opacity), Value::F64(1.0));
        assert_eq!(key_row_default_value(KeyRow::Anchor), Value::Vec2([0.0, 0.0]));
        assert_eq!(
            key_row_default_value(KeyRow::MaskOpacity(MaskId(7))),
            Value::F64(1.0),
            "mask opacity の既定は layer Opacity と同じ比 1.0 のはず"
        );
    }

    // -----------------------------------------------------------------------
    // MASK section(B02 第1切片): mode 巡回・inverted トグル・opacity field
    // -----------------------------------------------------------------------

    /// mode は宣言順の6値を一周して戻る(`next_blend_mode` のテストと同型)。
    #[test]
    fn mask_mode_cycles_through_all_six_modes_and_wraps() {
        assert_eq!(next_mask_mode(MaskMode::Add), MaskMode::Subtract);
        assert_eq!(next_mask_mode(MaskMode::Subtract), MaskMode::Intersect);
        assert_eq!(next_mask_mode(MaskMode::Intersect), MaskMode::Lighten);
        assert_eq!(next_mask_mode(MaskMode::Lighten), MaskMode::Darken);
        assert_eq!(next_mask_mode(MaskMode::Darken), MaskMode::Difference);
        assert_eq!(next_mask_mode(MaskMode::Difference), MaskMode::Add);
    }

    fn three_masks() -> Vec<Mask> {
        vec![
            Mask {
                id: MaskId(1),
                mode: MaskMode::Add,
                inverted: false,
            },
            Mask {
                id: MaskId(2),
                mode: MaskMode::Darken,
                inverted: true,
            },
            Mask {
                id: MaskId(3),
                mode: MaskMode::Difference,
                inverted: false,
            },
        ]
    }

    /// mode 巡回は対象だけを動かし、並び・他の mask・inverted を保つ。
    /// 居ない id は `None`(stale click は no-op)。
    #[test]
    fn masks_with_cycled_mode_touches_only_the_target_and_keeps_the_order() {
        let masks = three_masks();
        let new = masks_with_cycled_mode(&masks, MaskId(2)).expect("対象は居るはず");
        assert_eq!(new.len(), 3);
        assert_eq!(new[0], masks[0], "対象外(前)の mask が動いている");
        assert_eq!(new[1].mode, MaskMode::Difference, "宣言順の次 mode のはず");
        assert_eq!(new[1].id, MaskId(2));
        assert!(new[1].inverted, "mode 巡回が inverted を巻き込んでいる");
        assert_eq!(new[2], masks[2], "対象外(後)の mask が動いている");

        assert_eq!(masks_with_cycled_mode(&masks, MaskId(99)), None);
        assert_eq!(masks_with_cycled_mode(&[], MaskId(1)), None);
    }

    /// inverted トグルも同型(対象だけ・mode は保つ・stale は `None`)。
    #[test]
    fn masks_with_toggled_inverted_flips_only_the_target() {
        let masks = three_masks();
        let new = masks_with_toggled_inverted(&masks, MaskId(2)).expect("対象は居るはず");
        assert!(!new[1].inverted, "true → false へ裏返るはず");
        assert_eq!(new[1].mode, MaskMode::Darken, "トグルが mode を巻き込んでいる");
        assert_eq!(new[0], masks[0]);
        assert_eq!(new[2], masks[2]);

        let back = masks_with_toggled_inverted(&new, MaskId(2)).expect("対象は居るはず");
        assert_eq!(back, masks, "2回のトグルで元へ戻るはず");

        assert_eq!(masks_with_toggled_inverted(&masks, MaskId(99)), None);
    }

    /// mask opacity field は既存の値セル文法の対応表(property/単位/精度/感度)へ
    /// layer Opacity と同格で乗る。
    #[test]
    fn the_mask_opacity_field_joins_the_existing_value_cell_grammar() {
        let field = TransformField::MaskOpacity(MaskId(4));
        assert_eq!(
            property_id(field).expect("mask opacity の property は作れる"),
            PropertyId::mask_opacity(MaskId(4))
        );
        // 表示 % → store 比(clamp 込み — layer Opacity と同じ写像)。
        assert_eq!(next_value(field, 50.0, [0.0, 0.0]), Value::F64(0.5));
        assert_eq!(next_value(field, 150.0, [0.0, 0.0]), Value::F64(1.0));
        assert_eq!(next_value(field, -10.0, [0.0, 0.0]), Value::F64(0.0));
        assert_eq!(field_decimals(field), 0, "% 表示は整数(layer Opacity と同じ)");
        assert_eq!(
            dragged_value(field, 50.0, 20.0, false),
            70.0,
            "drag 感度は 1px = 1%(layer Opacity と同じ)のはず"
        );
    }

    /// drag の起点は MASK section の opacity 行からも読める(drag-to-scrub が
    /// mask opacity セルでも同じに効くための投影側の口)。
    #[test]
    fn drag_origin_finds_the_mask_opacity_slot() {
        let field = TransformField::MaskOpacity(MaskId(1));
        let selection = SelectionProjection {
            layer: LayerId(1),
            kind: "solid",
            transform: vec![],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
                speed_percent: 100.0,
                label_color: None,
                matte: None,
                matte_candidates: vec![],
            },
            masks: vec![MaskRowProjection {
                id: MaskId(1),
                mode: MaskMode::Add,
                inverted: false,
                opacity: TransformRowProjection {
                    label: "Opacity",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Opacity",
                        present: true,
                        value: 80.0,
                        editable: true,
                        keyed: false,
                        field: Some(field),
                    }),
                    decimals: 0,
                    key: KeyCellProjection {
                        row: KeyRow::MaskOpacity(MaskId(1)),
                        state: KeyCellState::Static,
                    },
                },
            }],
            effects: vec![],
            text: None,
            audio: None,
            links: vec![],
        };
        let (start, _) = drag_origin(&selection, field).expect("mask opacity は editable のはず");
        assert_eq!(start, 80.0, "起点は投影の表示値(%)のはず");
        assert!(
            drag_origin(&selection, TransformField::MaskOpacity(MaskId(9))).is_none(),
            "別の mask id の field では drag を始めないはず"
        );
    }

    /// drag の起点は AUDIO section の4行からも読める(B42、裁定184 型別
    /// section 第4号 — mask opacity と同じ「専用 section の投影も
    /// `drag_origin` が舐める」拡張、`lib.rs::drag_origin` の AUDIO ループ参照)。
    #[test]
    fn drag_origin_finds_the_audio_section_slots() {
        let selection = SelectionProjection {
            layer: LayerId(1),
            kind: "media",
            transform: vec![],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
                speed_percent: 100.0,
                label_color: None,
                matte: None,
                matte_candidates: vec![],
            },
            masks: vec![],
            effects: vec![],
            text: None,
            audio: Some(AudioSectionProjection {
                level: TransformRowProjection {
                    label: "Level",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Level",
                        present: true,
                        value: 100.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::Level),
                    }),
                    decimals: 1,
                    key: KeyCellProjection {
                        row: KeyRow::Level,
                        state: KeyCellState::Static,
                    },
                },
                pan: TransformRowProjection {
                    label: "Pan",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Pan",
                        present: true,
                        value: -0.3,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::Pan),
                    }),
                    decimals: 2,
                    key: KeyCellProjection {
                        row: KeyRow::Pan,
                        state: KeyCellState::Static,
                    },
                },
                fade_in: TransformRowProjection {
                    label: "Fade In",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Fade In",
                        present: true,
                        value: 0.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::FadeIn),
                    }),
                    decimals: 2,
                    key: KeyCellProjection {
                        row: KeyRow::FadeIn,
                        state: KeyCellState::Static,
                    },
                },
                fade_out: TransformRowProjection {
                    label: "Fade Out",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Fade Out",
                        present: true,
                        value: 0.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::FadeOut),
                    }),
                    decimals: 2,
                    key: KeyCellProjection {
                        row: KeyRow::FadeOut,
                        state: KeyCellState::Static,
                    },
                },
            }),
            links: vec![],
        };
        let (level_start, _) =
            drag_origin(&selection, TransformField::Level).expect("Level は editable のはず");
        assert_eq!(level_start, 100.0);
        let (pan_start, _) =
            drag_origin(&selection, TransformField::Pan).expect("Pan は editable のはず");
        assert_eq!(pan_start, -0.3);
        assert!(
            drag_origin(&selection, TransformField::Rotation).is_none(),
            "AUDIO 投影に無い field では drag を始めないはず"
        );
    }

    #[test]
    fn inspector_character_size_ratio_is_locked_within_the_charter_168_band() {
        let dims = Dimensions::default();
        let ratio = dims.body_text / dims.inspector_row_height;

        const TARGET: f32 = 0.42;
        const TOLERANCE: f32 = 0.05;
        let in_band = (ratio - TARGET).abs() <= TOLERANCE;

        assert_eq!(
            ratio, 0.44,
            "body_text/inspector_row_height の実測比が動いた(I-tokens の再転写値 \
             0.44 から動いたなら、このテストと台帳・FINDING の記載を三箇所とも \
             更新すること)"
        );
        assert!(
            in_band,
            "比 {ratio} が裁定168 の帯(0.42±0.05)から外れた — I-tokens の \
             再転写(inspector_row_height=25)がこの根治の前提なので、\
             どちらかの値が意図せず動いた疑いがある"
        );
    }

    // -----------------------------------------------------------------------
    // 線化 D2(裁定179「箱は状態の器」): style 関数レベルの柵。
    // widget tree で hover の Status を作れない(iced_test の Simulator は
    // cursor を置けるが container の style closure は status を受けない)ので、
    // style fn を直接呼んで固定する(発注書の指定どおり)。
    // -----------------------------------------------------------------------

    /// 「輪郭が消えている」の判定: width 0 か色が完全透明のどちらかなら
    /// 輪郭は描かれない(`container::draw_background` は `border.width > 0.0`
    /// でだけ quad を出す、fork 実測)。
    fn border_is_invisible(border: iced::Border) -> bool {
        border.width == 0.0 || border.color.a == 0.0
    }

    /// 値セル(表示状態): 平常は素の数字(面なし・輪郭透明)、hover でだけ
    /// 箱が現れる(既存 surface_hover 文法 — name 欄 hover と同じ)。
    #[test]
    fn the_value_box_is_bare_at_rest_and_boxed_on_hover() {
        let dims = Dimensions::default();
        let colors = Colors::default();

        let idle = value_box_style(dims, colors, ValueBoxStatus::Idle);
        assert!(
            idle.background.is_none(),
            "平常の値セルに面が残っている: {:?}",
            idle.background
        );
        assert!(
            border_is_invisible(idle.border),
            "平常の値セルに不透明な輪郭が残っている: {:?}",
            idle.border
        );

        let hovered = value_box_style(dims, colors, ValueBoxStatus::Hovered);
        assert_eq!(
            hovered.background,
            Some(iced::Background::Color(colors.surface_hover)),
            "hover の値セルに面(surface_hover)が現れない"
        );
        assert_eq!(hovered.border.color, colors.border_default);
        assert!(
            hovered.border.width > 0.0 && hovered.border.color.a > 0.0,
            "hover の値セルに不透明な輪郭が現れない: {:?}",
            hovered.border
        );
    }

    /// Blend/Reset ボタン: 平常は素の文字(面なし・輪郭なし)、hover で面、
    /// press で選択面(menubar `leaf_style` と同じ裁定179 文法)。
    #[test]
    fn the_inspector_buttons_are_bare_at_rest_and_faced_on_hover() {
        let colors = Colors::default();

        let rest = flat_button_style(colors, button::Status::Active);
        assert!(
            rest.background.is_none(),
            "平常のボタンに面が残っている: {:?}",
            rest.background
        );
        assert!(
            border_is_invisible(rest.border),
            "平常のボタンに輪郭が残っている: {:?}",
            rest.border
        );
        assert_eq!(rest.text_color, colors.text_primary);

        let hovered = flat_button_style(colors, button::Status::Hovered);
        assert_eq!(
            hovered.background,
            Some(iced::Background::Color(colors.surface_hover))
        );
        assert!(border_is_invisible(hovered.border), "hover のボタンは面のみ(輪郭は出さない)");

        let pressed = flat_button_style(colors, button::Status::Pressed);
        assert_eq!(
            pressed.background,
            Some(iced::Background::Color(colors.state_selected))
        );
    }

    /// M glyph: 輪郭は active(hidden=on)の時だけ(裁定179「チップ輪郭=
    /// 選択時のみ」)。非 active の平常は素の文字、hover は面。
    #[test]
    fn the_mute_glyph_wears_its_outline_only_while_active() {
        let dims = Dimensions::default();
        let colors = Colors::default();

        let off = glyph_button_style(dims, colors, button::Status::Active, false);
        assert!(
            border_is_invisible(off.border),
            "非 active の M glyph に常時輪郭が残っている: {:?}",
            off.border
        );

        let off_hover = glyph_button_style(dims, colors, button::Status::Hovered, false);
        assert_eq!(
            off_hover.background,
            Some(iced::Background::Color(colors.surface_hover))
        );

        let on = glyph_button_style(dims, colors, button::Status::Active, true);
        assert_eq!(on.border.color, colors.action_active, "active の M glyph は accent 縁(状態の器)");
        assert!(on.border.width > 0.0);
        assert_eq!(on.text_color, colors.action_active);
    }

    /// Speed 欄が採る text_input 文法(name 欄と同一の `name_input_style`):
    /// 平常=素・hover=面+枠・focus=箱+focus 縁。既存文法の pin(この文法へ
    /// Speed 欄を合流させるのが D2 の変更 — 文法そのものは name 欄で施工済み)。
    #[test]
    fn the_bare_input_grammar_shows_its_box_only_on_hover_or_focus() {
        let dims = Dimensions::default();
        let colors = Colors::default();

        let rest = name_input_style(dims, colors, text_input::Status::Active);
        assert_eq!(
            rest.background,
            iced::Background::Color(iced::Color::TRANSPARENT)
        );
        assert!(border_is_invisible(rest.border));

        let hovered = name_input_style(dims, colors, text_input::Status::Hovered);
        assert_eq!(hovered.background, iced::Background::Color(colors.surface_hover));
        assert_eq!(hovered.border.color, colors.border_default);

        let focused = name_input_style(
            dims,
            colors,
            text_input::Status::Focused { is_hovered: false },
        );
        assert_eq!(focused.background, iced::Background::Color(colors.surface_app));
        assert_eq!(focused.border.color, colors.action_active);
    }
}
