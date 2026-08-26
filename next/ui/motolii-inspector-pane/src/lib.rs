//! wraps: iced (frozen host, not product front) — Inspector pane(選択レイヤの属性・transform の読み書き、drag-to-scrub)。書き込みは Intent 経由のみ。
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
mod bulk;
mod chrome;
mod device;
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
mod mask_expansion;
// MATTE 行(2026-08-22 発注「レイヤーを指す」文法 第1号)。`LayerAttrs.matte`
// の元を選ぶ pick_list とその意味・書き口。
mod matte;
mod projection;
mod shape;
mod shape_fill;
mod shape_stroke;
mod text;
mod transform;

pub use attrs::{next_blend_mode, percent_to_speed_ratio, speed_percent, SUPPORTED_BLEND_MODES};
pub use chrome::{cycle_inspector_label_color, next_label_color, property_row_css, row_band_style};
pub use device::{
    device_for, device_for_provider, device_registry, parameter_for_provider,
    parameters_for_provider, CollapseState, DeviceCapabilities, DeviceId, DeviceVisibility,
    InspectorDevice, InspectorHostState, ParameterCapabilities, ParameterDescriptor, ParameterKind,
    ProjectionReadBoundary, AUDIO_DEVICE, ATTRS_DEVICE,
    EFFECTS_DEVICE, GLOW_DEVICE, MASK_DEVICE, SHAPE_DEVICE, TEXT_DEVICE, TRANSFORM_DEVICE,
};
pub use effects::{
    effects_with_moved_down, effects_with_moved_up, effects_with_removed,
    move_inspector_effect_down, move_inspector_effect_up, plugin_display_name, plugin_params,
    remove_inspector_effect, toggle_inspector_effect_bypass, GLOW_PLUGIN_ID,
};
pub use link::{
    clear_inspector_link, commit_inspector_link, LinkRowProjection, LinkSourceCandidate,
    LinkTarget,
};
pub use mask::{
    cycle_inspector_mask_mode, masks_with_cycled_mode, masks_with_toggled_inverted,
    next_mask_mode, toggle_inspector_mask_inverted,
};
pub use mask_expansion::MaskExpansionInput;
pub use matte::{
    clear_inspector_matte, cycle_inspector_matte_mode, next_matte_mode,
    set_inspector_matte_source,
};
pub use projection::{
    project, AttrsProjection, AudioSectionProjection, ComponentSlot, EffectRowProjection,
    KeyCellProjection, LayerCandidate, MaskRowProjection, RowValue, SelectionProjection,
    ShapeFillProjection, ShapeFillRowProjection, ShapeRowProjection, ShapeSectionProjection,
    TextSectionProjection, TransformRowProjection,
};
pub use shape::{commit_shape_field, ShapeField, ShapeFieldDraft};
pub use shape_fill::{
    apply_shape_gradient, commit_shape_fill, format_fill_hex, parse_hex_color,
    shape_fill_input_id, ShapeFillDraft, ShapeFillField,
};
pub use shape_stroke::{
    commit_shape_stroke, cycle_shape_stroke_cap, cycle_shape_stroke_join,
    parse_stroke_width, shape_stroke_input_id, toggle_shape_stroke_dash, ShapeStrokeDraft,
    ShapeStrokeField, ShapeStrokeProjection, ShapeStrokeRowProjection,
};
pub use text::{
    applied_text_content, applied_text_field, commit_text_field, commit_text_font_pick,
    commit_text_style_track_field, commit_text_style_track_field_for_layers,
    continue_text_style_drag, cycle_text_justify,
    default_text_document, default_text_style, finish_text_style_drag, next_text_justify,
    reset_text_line_height, reset_text_tracking, start_text_style_drag, text_document_content,
    text_field_track_target, toggle_text_style_key, TextField, TextFieldDraft,
    TextStyleDragState, TextStyleField, TextStyleTrackDraft,
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
use shape::*;
use shape_fill::*;
use shape_stroke::*;
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

    // ---- SHAPE section(P3 #15) ----
    /// SHAPE の寸法/角丸欄への打鍵。Enter まで Document へ書かない。
    ShapeFieldInput(ShapeField, String),
    /// SHAPE の寸法/角丸欄の Enter。`SetShapes` を1回だけ出す。
    ShapeFieldSubmit(ShapeField),
    /// Shape の fill の16進欄への打鍵。Enter まで Document へ書かない。
    ShapeFillInput(ShapeFillField, String),
    /// Shape の fill の16進欄の Enter。`SetShapes` を1回だけ出す。
    ShapeFillSubmit(ShapeFillField),
    /// Shape の fill を既定の linear gradient へ切り替える。
    ShapeFillGradient(usize),
    /// 色見本を押して fill の16進欄へ focus する。
    ShapeFillFocus(usize),
    /// Shape の stroke 線幅への打鍵。Enter まで Document へ書かない。
    ShapeStrokeInput(ShapeStrokeField, String),
    /// Shape の stroke 線幅の Enter。`SetShapes` を1回だけ出す。
    ShapeStrokeSubmit(ShapeStrokeField),
    /// Shape の stroke cap を次の形へ巡回する。
    ShapeStrokeCap(usize),
    /// Shape の stroke join を次の形へ巡回する。
    ShapeStrokeJoin(usize),
    /// Shape の stroke dash を on/off する。
    ShapeStrokeDash(usize),

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

    // ---- S4(2026-08-23、#46 の穴塞ぎ): Content 行の複数行 text_editor ----
    /// `text_editor` への編集操作(打鍵/カーソル移動/選択、すべて含む —
    /// `iced::widget::text_editor::Action`)。**まだ Document を書かない** —
    /// `motolii_shell::Shell::inspector_content_editor`(永続 `Content`)へ
    /// `perform` するだけ(`FieldInput` 系と同じ「確定するまで書かない」形、
    /// `text.rs::applied_text_content` doc「複数行の扱い」参照)。
    ContentEditorAction(text_editor::Action),
    /// Cmd/Ctrl+Enter(`text.rs::content_key_binding` が横取りする唯一の
    /// chord)——ここで初めて現在の編集バッファ全体を1回の
    /// `Intent::SetTextDocument` として書く(1 gesture = 1 undo)。マウス完遂路
    /// は選択を他レイヤーへ移すこと(`Shell::sync_inspector_content_editor`
    /// の blur-commit、裁定216)。
    ContentEditorCommit,

    // ---- D-1(2026-08-23): TEXT section の Size/Line Height/Tracking の
    // Key 列 + drag(A-1b が `text.rs` に用意した track 書き口の結線) ----
    /// Key 列 click。`KeyPressed` と同じ即時操作の形(即1回の
    /// `Intent::SetTrack`、書き込み本体は
    /// [`toggle_text_style_key`])。**3状態表示は無い**(`text.rs`
    /// `text_style_key_button` doc 参照 — `TextSectionProjection` に track の
    /// 有無が乗っていないため、常に同じ見た目)。
    TextStyleKeyPressed(TextStyleField),
    /// drag ハンドル press。`ValuePressed` と同じ形 — click か drag かは
    /// release まで未確定、move/release は window 全体購読
    /// (`PointerMoved`/`PointerReleased` を共有し、`Shell.
    /// inspector_text_style_drag` 側を追う——`inspector_drag` と同格の
    /// 別状態、`ValuePressed`/`inspector_drag` とは排他)。
    TextStyleValuePressed(TextStyleField),

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
use iced::widget::{
    button, column, container, row as row_widget, scrollable, text as iced_text, text_editor,
    text_input,
};
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
    view_with_content_editor(
        projection,
        field_draft,
        name_draft,
        speed_draft,
        text_field_draft,
        color_field_draft,
        None,
        None,
        None,
        None,
        dims,
        colors,
    )
}

/// [`view_with_color_draft`] と同じだが、Content 行(S4、#46 の穴塞ぎ)の
/// 永続 `text_editor::Content` も渡せる。`motolii_shell::Shell::view` は
/// こちらを呼ぶ。**`view`/`view_with_speed_draft`/`view_with_text_draft`/
/// `view_with_color_draft` 自身のシグネチャは変えていない**(既存呼び出し元・
/// ALLOWLIST 外のテストを無改修のまま通すため、上3つが導入された時と同じ
/// 判断)— 4つとも `content_editor: None` でここへ委譲するだけ。
///
/// **戻り値の寿命が `'a` になる唯一の入口**——`content_editor: Some(&'a
/// Content)` を渡すと戻り値は `Element<'a, Message>`(`'static` ではない)。
/// `None` を渡す上4つの経路では今までどおり `'static` に収まる(variance、
/// `text.rs::text_section` doc 参照)。
pub fn view_with_content_editor<'a>(
    projection: Option<&SelectionProjection>,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    speed_draft: Option<&str>,
    text_field_draft: Option<&TextFieldDraft>,
    color_field_draft: Option<&color::ColorFieldDraft>,
    shape_field_draft: Option<&ShapeFieldDraft>,
    shape_fill_draft: Option<&ShapeFillDraft>,
    shape_stroke_draft: Option<&ShapeStrokeDraft>,
    content_editor: Option<&'a text_editor::Content>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    // mock v3.1 の `.ptitle`("Inspector" 帯)は転写しない — pane 名の正本は
    // shell の pane 題帯(pane_grid title_bar、drag ハンドル兼任)へ移った。
    // 内部にも残すと "Inspector" が二重表示になる(題帯レーンの API 要求)。
    // mock 側の追随(ptitle 行の除去 or 注記)は supervisor キュー。
    let body: Element<'a, Message> = match projection {
        None => empty_state(dims, colors),
        Some(selection) => selected_body(
            selection,
            field_draft,
            name_draft,
            speed_draft,
            text_field_draft,
            color_field_draft,
            shape_field_draft,
            shape_fill_draft,
            shape_stroke_draft,
            content_editor,
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
            .size(dims.theme().text.caption)
            .color(colors.text_muted),
    )
    .padding(dims.theme().space.m)
    .into()
}

/// **視覚正本 `next/reference/mocks/ui-scale-and-z.html` の構造をそのまま写す**:
/// ident 帯 → column header 行 → TRANSFORM(Position/Scale/Rotation/Anchor)→
/// APPEARANCE(Opacity)→ ATTRS(Blend)→ hint 行。`selection.transform` 自体の
/// 並び(既存 `inspector_drive.rs` が固定している)は変えず、view 側で
/// ラベルによって TRANSFORM/APPEARANCE の見出しへ振り分けるだけ。
fn selected_body<'a>(
    selection: &SelectionProjection,
    field_draft: Option<&FieldDraft>,
    name_draft: Option<&str>,
    speed_draft: Option<&str>,
    text_field_draft: Option<&TextFieldDraft>,
    color_field_draft: Option<&color::ColorFieldDraft>,
    shape_field_draft: Option<&ShapeFieldDraft>,
    shape_fill_draft: Option<&ShapeFillDraft>,
    shape_stroke_draft: Option<&ShapeStrokeDraft>,
    content_editor: Option<&'a text_editor::Content>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    if selection.selection_count > 1 {
        let mut rows = column![multi_selection_band(selection, dims, colors)];
        if let Some(text_projection) = &selection.text {
            rows = rows.push(text_section(
                text_projection,
                text_field_draft,
                color_field_draft,
                None,
                true,
                dims,
                colors,
            ));
        } else {
            rows = rows.push(
                container(
                    iced_text("選択中に編集できる TEXT レイヤーがありません")
                        .size(dims.theme().text.caption)
                        .color(colors.text_muted),
                )
                .padding(dims.theme().space.m),
            );
        }
        rows = rows.push(hint_row(dims, colors));
        return scrollable(rows).height(Length::Fill).into();
    }

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
            content_editor,
            false,
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
    if let Some(shape_projection) = &selection.shape {
        rows = rows.push(shape_section(shape_projection, shape_field_draft, dims, colors));
    }
    if let Some(shape_fill_projection) = &selection.shape_fill {
        rows = rows.push(shape_fill_section(shape_fill_projection, shape_fill_draft, dims, colors));
    }
    if let Some(shape_stroke_projection) = &selection.shape_stroke {
        rows = rows.push(shape_stroke_section(
            shape_stroke_projection,
            shape_stroke_draft,
            dims,
            colors,
        ));
    }
    // LINK section(2026-08-22 発注「レイヤーを指す」文法 第3号): masks/effects
    // と違い「無ければ出さない」の Q0 判断は適用しない ── どの layer でも
    // 他 layer の標準 property を指せて良いはずなので常に現れる
    // (`selection.links` は `LinkTarget::ALL` 分、常に5行)。
    rows = rows.push(link_section(&selection.links, dims, colors));
    rows = rows.push(hint_row(dims, colors));

    scrollable(rows).height(Length::Fill).into()
}

/// Multi-selection identity band. It intentionally has no name input, mute
/// glyph, or single-layer metadata because those controls would have no
/// unambiguous target for the current Inspector write route.
fn multi_selection_band(
    selection: &SelectionProjection,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let text_summary = if selection.text_layer_count == selection.selection_count {
        format!("{} text layers", selection.text_layer_count)
    } else {
        format!("{} text layers / {} selected", selection.text_layer_count, selection.selection_count)
    };
    container(column![
        iced_text(format!("{} layers selected", selection.selection_count))
            .size(dims.theme().text.body)
            .font(TextWeight::Semibold.font())
            .color(colors.text_primary),
        iced_text(text_summary)
            .size(dims.theme().text.caption)
            .color(Ink::Secondary.resolve(&colors)),
    ])
    .padding([dims.theme().space.s, dims.theme().space.m])
    .into()
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
        .size(dims.theme().text.body)
        .font(TextWeight::Semibold.font())
        .padding(name_field_padding(dims))
        .style(move |_theme, status| name_input_style(dims, colors, status));

    // mock `.ident s{color:var(--ink2)}` — 旧実装は ink3(`text_muted`)を
    // 誤用していた(2026-08-21 更正)。
    let subtitle = iced_text(selection.kind)
        .size(dims.theme().text.caption)
        .color(Ink::Secondary.resolve(&colors));

    let identity = column![name_field, subtitle]
        .spacing(0.0)
        .width(Length::Fill);

    let glyphs = row_widget![
        mute_glyph(dims, colors, selection.attrs.hidden),
        reserved_glyph(dims),
    ]
    .spacing(dims.theme().space.xs)
    .align_y(iced::alignment::Vertical::Center);

    container(
        row_widget![
            label_color_chip(selection.attrs.label_color, dims, colors),
            identity,
            glyphs
        ]
        .spacing(dims.theme().space.s)
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding([dims.theme().space.s, dims.theme().space.m])
    .style(move |_theme| container::Style {
        // 線化 D5(裁定179 文法1): `surface_raised` の面が `surface_panel`
        // pane 地から明度1段浮く — 輪郭線は透明化(幅だけ残す=幾何不変)。
        background: Some(iced::Background::Color(colors.surface_raised)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.theme().stroke.hairline,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    // 裁定160 切片8で lib.rs から抽出、SP-4(2026-08-23)で本体を2ファイルへ
    // 分割(1,972行 → 800行以下)。`include!` はモジュール境界を増やさず
    // テキストをそのまま展開するだけ ── qualified test name(`tests::foo`、
    // すぐ上のコメントが `--list` 完全一致で名指ししている)を1つも変えない
    // ための選択(`mod part1; mod part2;` にすると `tests::part1::foo` へ
    // 変わってしまう)。
    include!("lib_tests_part1.rs");
    include!("lib_tests_part2.rs");
}
