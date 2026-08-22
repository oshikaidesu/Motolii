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
}

/// TEXT section の text_input 系フィールドの識別。**`TransformField` とは
/// 別の enum にする** — 対象が `KeyframeTrack`(`property_id`/
/// `commit_inspector_field`/drag-to-scrub の経路)ではなく `TextDocumentStyle`
/// の静止フィールド(裁定92)なので、track を前提にした既存の型に無理に
/// 押し込まない。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextField {
    /// `text-document f`(Font Family)。`FontRef::family` だけを書き換える
    /// (`path`/`fingerprint`/`style` はこの切片では触らない)。
    FontFamily,
    /// `text-document s`(Font Size)。
    Size,
    /// `text-document lh`(Line Height)。`None` = Auto。
    LineHeight,
    /// `text-document tr`(Tracking)。
    Tracking,
}

/// TEXT section 入力欄の下書き。**Document ではない**(`FieldDraft` と同型 —
/// commit(Enter)まで store に触らない)。
#[derive(Clone, Debug, PartialEq)]
pub struct TextFieldDraft {
    pub field: TextField,
    pub text: String,
}

// ---------------------------------------------------------------------------
// 型別 editor の対象 field
// ---------------------------------------------------------------------------

/// Transform 行が動かす field の識別。**`LayerId` を持たない** — 対象は常に
/// `Session::selection`(commit 時に読む)。選択が edit の合間に変わる稀なケースは
/// 「そのまま捨てる」で安全側に倒す(`motolii_shell::Shell::commit_inspector_field` 参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransformField {
    PositionX,
    PositionY,
    PositionZ,
    ScaleX,
    ScaleY,
    Rotation,
    Opacity,
    AnchorX,
    AnchorY,
    /// この mask の不透明度(MASK section、B02 第1切片)。**新しい編集文法を
    /// 発明しない** — 既存の値セル文法(`FieldDraft`/drag-to-scrub/
    /// `commit_inspector_field` → `Intent::SetTrack`)へそのまま乗るために
    /// `TransformField` を拡張する形を採る。track 名は id から決まる
    /// (`PropertyId::mask_opacity`)ので、mask の並べ替え・削除で別の mask へ
    /// 付き直さない(store の同一性設計そのまま)。
    MaskOpacity(MaskId),
    /// この effect の param(EFFECTS section、B38 第3切片)。
    /// [`TransformField::MaskOpacity`] と同型の拡張 — 既存の値セル文法
    /// (`FieldDraft`/drag-to-scrub/`commit_inspector_field` → `Intent::SetTrack`)
    /// へそのまま乗る。track 名は id + param 名から決まる
    /// (`PropertyId::effect_param` — `effect.{id}.param.{name}`)ので、stack の
    /// 並べ替え・削除で別の effect へ付き直さない。param は [`GlowParam`]
    /// (既知 plugin のカタログ)に閉じる — store は plugin の param カタログを
    /// 知らない(裁定70、`ResolvedEffect::params` doc)ので、既定値を埋める
    /// 仕事はこの「plugin 定義を知っている層」の側([`GlowParam::default_value`])。
    EffectParam(EffectId, GlowParam),
}

// ---------------------------------------------------------------------------
// EFFECTS: 既知 plugin の param カタログ(B38 第3切片)
// ---------------------------------------------------------------------------

/// 内蔵 vism 第1号 Glow の plugin id(裁定153 S4)。**engine 側の変換表
/// (`next/engine/motolii-engine/src/lib.rs::translate_effect_passes`)と同期を
/// 保つ義務がある**([`SUPPORTED_BLEND_MODES`] と同じ二重化の形 — engine が
/// 対応する plugin だけをここに書く)。
pub const GLOW_PLUGIN_ID: &str = "motolii.glow";

/// Glow の param カタログ(engine `translate_glow_params` が読む3つの named
/// param)。**enum で閉じる** — [`TransformField`]/[`KeyRow`] は `Copy` なので
/// param 名を `String` で運べない。既定値・小数桁・drag 感度もここに束ねる
/// (型別 editor registry の考え方、crate doc 参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlowParam {
    /// bright-pass 閾値(engine 既定 1.0 — proof `bright_fs` のハードコード値)。
    Threshold,
    /// composite の減衰率(engine 既定 0.75 — proof `composite_fs`)。
    Intensity,
    /// blur タップ間隔スケール(engine 既定 1.0 = proof の固定オフセット)。
    Radius,
}

impl GlowParam {
    /// 宣言順 = 表示順(engine `translate_glow_params` の読み出し順と同じ並び)。
    pub const ALL: [GlowParam; 3] = [
        GlowParam::Threshold,
        GlowParam::Intensity,
        GlowParam::Radius,
    ];

    /// track 名の断片(`effect.{id}.param.{name}` の `{name}`)。engine の
    /// `find("threshold", ..)` 等と一致する義務がある(上記の同期義務と同じ)。
    pub fn name(self) -> &'static str {
        match self {
            Self::Threshold => "threshold",
            Self::Intensity => "intensity",
            Self::Radius => "radius",
        }
    }

    /// 行ラベル(表示)。`name` の頭を大文字化しただけ — 発明ではない。
    pub fn label(self) -> &'static str {
        match self {
            Self::Threshold => "Threshold",
            Self::Intensity => "Intensity",
            Self::Radius => "Radius",
        }
    }

    /// track の無い param の既定値。**engine の既定
    /// (`GLOW_DEFAULT_THRESHOLD`/`INTENSITY`/`RADIUS`、private const)の写し** —
    /// engine と同期を保つ義務([`GLOW_PLUGIN_ID`] と同じ)。表示既定が engine
    /// 既定とズレると「値を出しただけで絵が変わって見える」誤読になるため。
    pub fn default_value(self) -> f64 {
        match self {
            Self::Threshold => 1.0,
            Self::Intensity => 0.75,
            Self::Radius => 1.0,
        }
    }
}

/// plugin id → param カタログ。**未知 plugin は空**(store は catalog を知らず、
/// engine も未知 plugin_id を無音 skip する — param 行を捏造しない、M13)。
pub fn plugin_params(plugin_id: &str) -> &'static [GlowParam] {
    if plugin_id == GLOW_PLUGIN_ID {
        &GlowParam::ALL
    } else {
        &[]
    }
}

/// plugin id → 表示名。既知 plugin だけ人間可読名、未知は plugin_id をそのまま
/// (M13: 無い意味を有るふりで出さない — id を隠して汎用名を出す方が嘘になる)。
pub fn plugin_display_name(plugin_id: &str) -> &str {
    if plugin_id == GLOW_PLUGIN_ID {
        "Glow"
    } else {
        plugin_id
    }
}

/// この field の store 上の property。標準 property は予約語でも空でもなく、
/// mask opacity は `PropertyId::mask_opacity`(構築が失敗し得ない形)、effect
/// param も名前が [`GlowParam::name`](静的・非予約語)に閉じるので実質失敗
/// し得ない — `motolii_shell::Shell` はこの `Result` を「コードの誤り」
/// として扱ってよい。
pub fn property_id(field: TransformField) -> Result<PropertyId, StoreError> {
    match field {
        TransformField::PositionX | TransformField::PositionY => PropertyId::new(property::POSITION),
        TransformField::PositionZ => PropertyId::new(property::POSITION_Z),
        TransformField::ScaleX | TransformField::ScaleY => PropertyId::new(property::SCALE),
        TransformField::Rotation => PropertyId::new(property::ROTATION),
        TransformField::Opacity => PropertyId::new(property::OPACITY),
        TransformField::AnchorX | TransformField::AnchorY => PropertyId::new(property::ANCHOR),
        TransformField::MaskOpacity(id) => Ok(PropertyId::mask_opacity(id)),
        TransformField::EffectParam(id, param) => PropertyId::effect_param(id, param.name()),
    }
}

/// 入力欄の下書き。**Document ではない** — commit(Enter)まで store に触らない。
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDraft {
    pub field: TransformField,
    pub text: String,
}

/// `field` を編集した結果の新しい `Value`。Vec2 系(Position/Scale/Anchor の X/Y)は
/// **現在値の他成分を保つ** — X だけ書き換えて Y を 0 に潰す事故を防ぐ
/// (`current_vec2` は commit 側が `value_at` で読んだ今の値、無ければ [`default_vec2`])。
pub fn next_value(field: TransformField, input: f64, current_vec2: [f64; 2]) -> Value {
    match field {
        TransformField::PositionX | TransformField::ScaleX | TransformField::AnchorX => {
            Value::Vec2([input, current_vec2[1]])
        }
        TransformField::PositionY | TransformField::ScaleY | TransformField::AnchorY => {
            Value::Vec2([current_vec2[0], input])
        }
        TransformField::PositionZ | TransformField::Rotation => Value::F64(input),
        // effect param は表示 = store 単位(換算なし)。clamp もしない — 値域は
        // plugin(engine 側 shader)の意味で、editor が知ったかぶりしない
        // (engine `translate_glow_params` も clamp しない)。
        TransformField::EffectParam(_, _) => Value::F64(input),
        // 表示は % だが store は 0..1 の比(`property::OPACITY` の既定と同じ単位)。
        // mask opacity も同じ単位(比、`motolii_store::mask` doc「不透明度は比」)。
        TransformField::Opacity | TransformField::MaskOpacity(_) => {
            Value::F64((input / 100.0).clamp(0.0, 1.0))
        }
    }
}

/// track が無い(まだキーを打っていない)Vec2 property の既定値。Scale だけ等倍(1.0)、
/// 他は 0(`view.rs::resolve` の既定と同じ、裁定20 の応用)。
pub fn default_vec2(field: TransformField) -> [f64; 2] {
    match field {
        TransformField::ScaleX | TransformField::ScaleY => [1.0, 1.0],
        _ => [0.0, 0.0],
    }
}

/// 静的値を書く唯一の形。**1キー `Hold`**(発注書が名指しした流儀)。時刻は
/// `RationalTime::ZERO` — 1キーだけの track は `KeyframeTrack::eval` がどの時刻でも
/// 同じ値を返す(`t <= keys[0].t` と `t >= keys[last].t` が同じキーに落ちる、
/// `motolii-eval` の実装どおり)ので、時刻自体に意味は無い。
pub fn single_hold_track(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

// ---------------------------------------------------------------------------
// K1: Key 列 — 行→property の対応・3状態 oracle・click の意味(全部純関数)
// ---------------------------------------------------------------------------

/// Key セルが動かす行の識別。**行 = property 1本**(X/Y/Z 軸は現行モデルどおり
/// 1 track に畳まれている — 軸別キーは対象外)。Position 行の Key は
/// `property::POSITION`(Vec2)だけを対象にし、`POSITION_Z`(別 property)は
/// 含めない — 1 click = 1 `SetTrack` = 1 undo を保つため(RETURN に仕様として
/// 注記、逸脱ではない)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyRow {
    Position,
    Scale,
    Rotation,
    Opacity,
    Anchor,
    /// mask の不透明度行(MASK section、B02 第1切片)。既存 Key 列文法
    /// (3状態 oracle・`toggled_key_track`)へそのまま乗る —
    /// [`TransformField::MaskOpacity`] と同じ拡張の形。
    MaskOpacity(MaskId),
    /// effect param 行(EFFECTS section、B38 第3切片)。同上 —
    /// [`TransformField::EffectParam`] と同じ拡張の形。
    EffectParam(EffectId, GlowParam),
}

impl KeyRow {
    /// 標準 property 行の名前(旧 `property_name` — mask 行は id から動的に
    /// 決まるので、この対応表には載らない。[`key_row_property_id`] が正本)。
    fn static_property_name(self) -> Option<&'static str> {
        match self {
            Self::Position => Some(property::POSITION),
            Self::Scale => Some(property::SCALE),
            Self::Rotation => Some(property::ROTATION),
            Self::Opacity => Some(property::OPACITY),
            Self::Anchor => Some(property::ANCHOR),
            Self::MaskOpacity(_) | Self::EffectParam(_, _) => None,
        }
    }
}

/// この行の store 上の property。標準 property・mask opacity・effect param の
/// いずれも構築が失敗し得ない([`property_id`] と同じ理由 — 呼び手は `Result`
/// を「コードの誤り」として扱ってよい)。
pub fn key_row_property_id(row: KeyRow) -> Result<PropertyId, StoreError> {
    match row {
        KeyRow::MaskOpacity(mask) => Ok(PropertyId::mask_opacity(mask)),
        KeyRow::EffectParam(effect, param) => PropertyId::effect_param(effect, param.name()),
        _ => PropertyId::new(
            row.static_property_name()
                .expect("mask/effect 以外の行は静的な property 名を持つ"),
        ),
    }
}

/// track がまだ無い行の既定値(`project` の各行の default と同じ値 —
/// Scale だけ等倍、Opacity(layer/mask とも)は store 単位の 0..1 で 1.0、
/// 他は 0)。静的値も無い行を初キー化する時の値の正本。
pub fn key_row_default_value(row: KeyRow) -> Value {
    match row {
        KeyRow::Position | KeyRow::Anchor => Value::Vec2([0.0, 0.0]),
        KeyRow::Scale => Value::Vec2([1.0, 1.0]),
        KeyRow::Rotation => Value::F64(0.0),
        KeyRow::Opacity | KeyRow::MaskOpacity(_) => Value::F64(1.0),
        // effect param の既定は plugin カタログ(= engine 既定の写し)から。
        KeyRow::EffectParam(_, param) => Value::F64(param.default_value()),
    }
}

/// Key セルの3状態(AE 文法 — 意図優先・裁定174)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCellState {
    /// track 無し(または正準静的表現 = [`single_hold_track`] の形)。空菱形 ◇。
    Static,
    /// track 有り・playhead のフレームにキー有り。実菱形 ◆(accent・強い面)。
    AtKey,
    /// track 有り・playhead のフレームにキー無し。実菱形 ◆(accent・弱い面 —
    /// mock `.keyButton.animated` の転写)。
    Between,
}

/// この track が「静的値の正準表現」([`single_hold_track`] が書く 1キー
/// `Hold` @`ZERO`)か。**キーが打たれているとは見なさない** — Inspector の
/// 静的値編集が書いた track を Key セルが「キー有り」と誤読すると、値を
/// 打っただけの行が playhead=0 で実菱形になってしまう。既知の限界: 利用者が
/// 本当に frame 0 に `Hold` キーを1個だけ置いた track も同じ形なので静的と
/// 読まれる(現行 UI にその作成経路は無い — Key click の先頭 insert は
/// `Linear`。RETURN に注記)。
fn is_canonical_static_track(track: &KeyframeTrack) -> bool {
    track.keys().len() == 1
        && matches!(track.keys()[0].interp, Interp::Hold)
        && track.keys()[0].t == RationalTime::ZERO
}

/// playhead のフレームに載っているキーの添字。照合は timeline と同じ
/// `try_to_frame_round`(frame 粒度 — `timeline_pane::write` の
/// `commit_key_frames`/`delete_selected_keys` と同じ規約、RationalTime の
/// 厳密一致にしない)。
fn key_index_at_frame(track: &KeyframeTrack, frame: i64, fps: Fps) -> Option<usize> {
    track
        .keys()
        .iter()
        .position(|key| key.t.try_to_frame_round(fps) == Ok(frame))
}

/// **3状態 oracle**(表示と click 判定の共通正本 — view と
/// [`toggled_key_track`] の両方がここを通るので、見た目と操作が食い違わない)。
pub fn key_cell_state(
    track: Option<&KeyframeTrack>,
    playhead_frame: i64,
    fps: Fps,
) -> KeyCellState {
    let Some(track) = track else {
        return KeyCellState::Static;
    };
    if track.keys().is_empty() || is_canonical_static_track(track) {
        return KeyCellState::Static;
    }
    if key_index_at_frame(track, playhead_frame, fps).is_some() {
        KeyCellState::AtKey
    } else {
        KeyCellState::Between
    }
}

/// **Key click の意味**: 今の track から、click 後に `Intent::SetTrack` で
/// 書くべき新しい track を作る(純関数 — Document には触れない)。
///
/// - **Static** → 現在の静的値([`single_hold_track`] が有ればその値、無ければ
///   呼び手が渡す `current_value`)で playhead 時刻にキー1個。track 先頭
///   insert の interp 既定は `Linear`(発注書)。
/// - **AtKey** → そのキーを除去。最後の1個なら track ごと静的化
///   ([`single_hold_track`] に消したキーの値を移す — AE のストップウォッチ
///   解除と等価、値は失わない。undo は `SetTrack` 経由なので効く)。
/// - **Between** → その時刻の `KeyframeTrack::eval` 値でキー追加。interp は
///   直前のキーの流儀を継ぎ、最初のキーより前への insert は `Linear`。
///   `spatial` は `None`(空間タンジェントの分割は対象外 — RETURN に注記)。
pub fn toggled_key_track(
    track: Option<&KeyframeTrack>,
    playhead_frame: i64,
    fps: Fps,
    current_value: Value,
) -> Result<KeyframeTrack, String> {
    let t = RationalTime::try_from_frame(playhead_frame, fps)
        .map_err(|error| format!("playhead を時刻へ写せない: {error}"))?;
    match key_cell_state(track, playhead_frame, fps) {
        KeyCellState::Static => {
            let value = track
                .filter(|tr| !tr.keys().is_empty())
                .map(|tr| tr.keys()[0].value.clone())
                .unwrap_or(current_value);
            let mut new_track = KeyframeTrack::new();
            new_track.insert(Keyframe {
                t,
                value,
                interp: Interp::Linear,
                spatial: None,
            });
            Ok(new_track)
        }
        KeyCellState::AtKey => {
            // state が AtKey なら track と添字は必ず有る(oracle と同じ判定)。
            let track = track.expect("AtKey なら track が有る");
            let index = key_index_at_frame(track, playhead_frame, fps)
                .expect("AtKey なら playhead 上のキーが有る");
            let removed_value = track.keys()[index].value.clone();
            let mut new_track = KeyframeTrack::new();
            for (i, key) in track.keys().iter().enumerate() {
                if i != index {
                    new_track.insert(key.clone());
                }
            }
            if new_track.keys().is_empty() {
                Ok(single_hold_track(removed_value))
            } else {
                Ok(new_track)
            }
        }
        KeyCellState::Between => {
            let track = track.expect("Between なら track が有る");
            let value = track.eval(t);
            let interp = track
                .keys()
                .iter()
                .rev()
                .find(|key| key.t < t)
                .map(|key| key.interp)
                .unwrap_or(Interp::Linear);
            let mut new_track = track.clone();
            new_track.insert(Keyframe {
                t,
                value,
                interp,
                spatial: None,
            });
            Ok(new_track)
        }
    }
}

/// この track が「実キーを持つ」か(空・track 無し・正準静的表現
/// [`is_canonical_static_track`] はどれも実キー無し)。値編集の意味
/// ([`edited_value_track`])と投影の `keyed`(accent 表示)の共通判定。
fn has_real_keys(track: Option<&KeyframeTrack>) -> bool {
    track.is_some_and(|tr| !tr.keys().is_empty() && !is_canonical_static_track(tr))
}

/// **値編集の意味**(AE 作法、2026-08-22 発注 — 利用者実窓指摘「キーが1つ
/// しか打てない」の根治): 値セルの Enter 確定・数値ドラッグ確定が
/// `Intent::SetTrack` で書くべき新しい track を作る(純関数)。
///
/// - **キー無し**(track 無し・空・正準静的表現)→ 従来どおり静的値の
///   書き換え([`single_hold_track`] — キーは生えない。キー化は Key 列 click
///   が明示的に行う)。
/// - **キー持ち**(実キー >= 1)→ **playhead 位置へのキー upsert**:
///   playhead のフレームにキーが有ればそのキーの値だけ更新(時刻・interp・
///   spatial は保つ)、無ければ新キー挿入。interp は Between 挿入
///   ([`toggled_key_track`])と同規則 — 直前キーの流儀を継ぎ、先頭は
///   `Linear`。`spatial` は `None`。**track を静的に戻さない** — これが
///   「値を変えるとキーが増える」の AE 文法。
pub fn edited_value_track(
    track: Option<&KeyframeTrack>,
    playhead_frame: i64,
    fps: Fps,
    value: Value,
) -> Result<KeyframeTrack, String> {
    if !has_real_keys(track) {
        return Ok(single_hold_track(value));
    }
    let track = track.expect("実キーが有るなら track も有る");
    let t = RationalTime::try_from_frame(playhead_frame, fps)
        .map_err(|error| format!("playhead を時刻へ写せない: {error}"))?;
    let mut new_track = track.clone();
    if let Some(index) = key_index_at_frame(track, playhead_frame, fps) {
        // 既存キーの値更新(個数不変)。時刻は**既存キーのもの**を保つ —
        // 照合は frame 丸め(`key_index_at_frame`)なので、丸め前の厳密時刻で
        // insert し直すと同フレームに2個生える事故になる。
        let existing = &track.keys()[index];
        new_track.insert(Keyframe {
            t: existing.t,
            value,
            interp: existing.interp,
            spatial: existing.spatial.clone(),
        });
    } else {
        let interp = track
            .keys()
            .iter()
            .rev()
            .find(|key| key.t < t)
            .map(|key| key.interp)
            .unwrap_or(Interp::Linear);
        new_track.insert(Keyframe {
            t,
            value,
            interp,
            spatial: None,
        });
    }
    Ok(new_track)
}

pub fn format_number(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// 値セルの**表示**文字数の上限(裁定169、**I-tokens(2026-08-22)で再較正**)。
/// 旧値6は `inspector_value_width=38px` 時代のアンカー(「1.000」5字は収まる・
/// 「960.000」7字は先頭末尾が clip される、いずれも 2026-08-21 深夜の実窓実測、
/// φ 検収)—`inspector_value_width` が I-tokens で 38→64px へ束ごと再転写された
/// ため、このアンカーも引き直す必要がある(発注書の指示どおり)。
///
/// **再較正の方法**: 実窓実測(φ 期のように実際のビルドを目で見る)がこの
/// レーンでは行えないため、`value_cell_legibility.rs` と同じ手口 —
/// `iced_test` で実フォント(0.15 fork の "Fira Sans" スタック、実窓と同一
/// レンダラ)を使い `text(content).size(dims.body_text)` の自然幅(`Target::Text`
/// の layout bounds、`iced_widget::text::layout` が `Length::Shrink` を
/// clamp する前の値)を直接測る決定論的な代替測定を採った(px からの近似換算は
/// 等幅仮定が崩れて信用できないという旧アンカーの理由はそのまま — ここでは
/// 「文字数から px への換算」ではなく「候補文字列ごとの実測 px」を使うので、
/// その理由に反しない)。新しいアンカー2点(数字のみ・小数点あり文字列の
/// 自然幅、実測 2026-08-22):
/// - 「1234567.000」(11字)の自然幅 58.861px は箱幅64pxの**内**(収まる)。
/// - 「12345678.000」(12字)の自然幅 64.922px は箱幅64pxの**外**(clip される)。
///
/// 旧アンカー(箱幅38px)も同じ相対マージンで境界に立っていた(6字
/// 「960.00」32.7px=箱の86%・7字「960.000」38.83px=箱の102%)— 新アンカー
/// (11字58.9px=箱の92%・12字64.9px=箱の101%)は同型の境界なので、この
/// 再較正は旧アンカーの選び方をそのまま踏襲した外挿(6→11は単純な比例
/// 64/38≈1.68倍では6×1.68≈10.1になるところ、実測の境界に合わせて11とした)。
/// **実窓での目視確認はこの発注の範囲外**(見送り — 利用者チェックリストへ
/// 追加候補として RETURN に記録)。
pub const MAX_VALUE_CELL_CHARS: usize = 11;

/// セルに収まる精度へ落とした**表示専用**の整形(裁定169)。field 既定の
/// `decimals` から始め、[`MAX_VALUE_CELL_CHARS`] を超える間 1 桁ずつ落とす
/// (最小 0 — 整数部だけでも超える値はそのまま出す: clip(true) が防波堤)。
/// **編集 draft は全精度のまま**([`value_cell`] の editing 分岐は
/// [`format_number`] を直接呼ぶ) — 表示は丸め、編集は真値、の分担。
/// モック出典: inspector-library の値サンプルは「24.0」「80.0」「2.50」と
/// 幅広値ほど短精度(AE の Position 1桁小数と同型)。
pub fn display_number(value: f64, decimals: usize) -> String {
    let mut d = decimals;
    loop {
        let s = format_number(value, d);
        if s.chars().count() <= MAX_VALUE_CELL_CHARS || d == 0 {
            return s;
        }
        d -= 1;
    }
}

// ---------------------------------------------------------------------------
// Speed 欄(ATTRS、SP1 第一波)— %⇄`motolii_store::Speed` の写像だけをここに置く。
// ---------------------------------------------------------------------------
//
// **`LayerTiming`/`Intent::SetTiming` の組み立て・duration 再計算はここでは
// 行わない**: duration 再計算の純関数(`retimed_duration`、supervisor 決定4)は
// `motolii-timeline-pane::clip_gesture` に住む(第二波 Shift+端drag と共有する
// ため、δ 採択理由)が、この crate は `motolii-timeline-pane` へ依存できない
// (`Cargo.toml` は今回の発注書 ALLOWLIST に含まれない — root→pane の一方向
// 依存を保つ既存の判断を、新しい循環を作らずに守った結果)。**両方に依存できる
// `motolii-shell` root がその組み立てを担う**(`Shell::apply_speed` —
// `commit_inspector_field` が `Value`/`Intent::SetTrack` まで組むのと違う分担、
// RETURN の FINDING 参照)。ここが持つのは「% ⇄ (num, den)」の純粋な往復だけ。

/// 表示 % → `motolii_store::Speed` の `(num, den)`。**p は正の有限値のみ受理**
/// (0・負・NaN・∞は `None` — supervisor 決定3「0 は拒否」)。**分母は 1000 固定**
/// (表示の小数1桁をそのまま整数化できる最小の桁 — `Speed::try_new` の不変式
/// 「分母は正」を機械的に満たす、値を約分はしない)。
pub fn percent_to_speed_ratio(percent: f64) -> Option<(i64, i64)> {
    if !percent.is_finite() || percent <= 0.0 {
        return None;
    }
    let tenths = (percent * 10.0).round();
    if tenths <= 0.0 {
        // 丸めで0以下になる極小値(例: 0.04%)も同じ理由で拒む。
        return None;
    }
    Some((tenths as i64, 1000))
}

/// `Speed` の `(num, den)` → 表示 %(逆算、[`percent_to_speed_ratio`] の逆写像)。
/// `format_number(_, 1)` と組み合わせて小数1桁で表示する(view 側)。`den == 0`
/// は `Speed::try_new` の不変式により本来起こらないが、安全側で 100.0 を返す。
pub fn speed_percent(num: i64, den: i64) -> f64 {
    if den == 0 {
        return 100.0;
    }
    num as f64 / den as f64 * 100.0
}

/// Blend 巡回ボタンが回る mode の一覧。**engine 側の変換表
/// (`next/engine/motolii-engine/src/lib.rs::translate_blend_mode`)と同期を保つ義務が
/// ある**(発注書「決定済み事項」— 対応 mode の一覧を engine 側と同じ場所には置か
/// ない、という決定に沿って Inspector 側にハードコードする)。**BL3(2026-08-22)**で
/// 分離可能11種(Multiply〜Exclusion)を追加——並びは `motolii_store::BlendMode` の
/// 宣言順(AE のメニュー順同型、Normal 直後に Add)のまま。非分離4種
/// (Hue/Saturation/Color/Luminosity、BL4)はまだここに無い(engine 側 `translate_blend_mode`
/// が対応するまで、対応 mode だけを巡る発注書の決定どおり)。dropdown 化する時に
/// この二重化は解消する。
pub const SUPPORTED_BLEND_MODES: &[motolii_store::BlendMode] = &[
    motolii_store::BlendMode::Normal,
    motolii_store::BlendMode::Add,
    motolii_store::BlendMode::Multiply,
    motolii_store::BlendMode::Screen,
    motolii_store::BlendMode::Overlay,
    motolii_store::BlendMode::Darken,
    motolii_store::BlendMode::Lighten,
    motolii_store::BlendMode::ColorDodge,
    motolii_store::BlendMode::ColorBurn,
    motolii_store::BlendMode::HardLight,
    motolii_store::BlendMode::SoftLight,
    motolii_store::BlendMode::Difference,
    motolii_store::BlendMode::Exclusion,
];

/// Blend 巡回ボタンの次の値。**現在値が [`SUPPORTED_BLEND_MODES`] に無い場合**
/// (将来の下位互換 — engine がまだ対応していない mode が Document に既に入って
/// いた時)は `Err` にしない — 現在値をそのまま表示し続け、次クリックで一覧の
/// 先頭へ進む(発注書「決定済み事項」)。
pub fn next_blend_mode(current: motolii_store::BlendMode) -> motolii_store::BlendMode {
    let modes = SUPPORTED_BLEND_MODES;
    match modes.iter().position(|mode| *mode == current) {
        Some(i) => modes[(i + 1) % modes.len()],
        None => modes[0],
    }
}

// ---------------------------------------------------------------------------
// MASK section(B02 第1切片、裁定184)— mode 巡回・inverted トグルの意味と書き口。
// 値(opacity)は `TransformField::MaskOpacity` 経由で既存の値セル文法が書くので、
// ここに opacity の書き口は無い。
// ---------------------------------------------------------------------------

/// mask mode 巡回ボタンの次の値。並びは `motolii_store::MaskMode` の宣言順
/// (= Lottie `mask-mode` から `None` を落とした6値、store の設計どおり)。
/// [`next_blend_mode`] と同じ巡回ボタン文法 — pick_list は next/ に前例が無い
/// (BL2 の決定)ので導入しない。blend と違い「engine 未対応の mode」は無い
/// (MK2 被覆代数が6値全部を実装済み)ため、対応表の部分集合も持たない。
pub fn next_mask_mode(mode: MaskMode) -> MaskMode {
    match mode {
        MaskMode::Add => MaskMode::Subtract,
        MaskMode::Subtract => MaskMode::Intersect,
        MaskMode::Intersect => MaskMode::Lighten,
        MaskMode::Lighten => MaskMode::Darken,
        MaskMode::Darken => MaskMode::Difference,
        MaskMode::Difference => MaskMode::Add,
    }
}

/// mode 巡回後の mask 一覧(純関数 — Document には触れない)。対象の mask が
/// 居なければ `None`(呼び手は no-op — 選択が edit の合間に変わる稀なケースを
/// 捨てる、`commit_inspector_field` と同じ安全側)。**他の mask・並び順・
/// inverted は一切変えない**(`Intent::SetMasks` は一覧の丸ごと差し替えなので、
/// ここが「対象だけを動かす」ことの正本)。
pub fn masks_with_cycled_mode(masks: &[Mask], target: MaskId) -> Option<Vec<Mask>> {
    masks.iter().any(|mask| mask.id == target).then(|| {
        masks
            .iter()
            .map(|mask| {
                if mask.id == target {
                    Mask {
                        mode: next_mask_mode(mask.mode),
                        ..*mask
                    }
                } else {
                    *mask
                }
            })
            .collect()
    })
}

/// inverted トグル後の mask 一覧([`masks_with_cycled_mode`] と同型の純関数)。
pub fn masks_with_toggled_inverted(masks: &[Mask], target: MaskId) -> Option<Vec<Mask>> {
    masks.iter().any(|mask| mask.id == target).then(|| {
        masks
            .iter()
            .map(|mask| {
                if mask.id == target {
                    Mask {
                        inverted: !mask.inverted,
                        ..*mask
                    }
                } else {
                    *mask
                }
            })
            .collect()
    })
}

/// MASK section の mode 巡回 — 即1回の `Intent::SetMasks` を出す(1 click =
/// 1 undo、`ToggleHidden`/`CycleBlendMode` と同じ即時操作の形)。選択なし・
/// 対象 mask なしは黙って no-op(`Ok(())`)。書き込み失敗だけ `Err` の理由文
/// (M13、呼び出し側が status 帯へ渡す)。
pub fn cycle_inspector_mask_mode(
    doc: &mut Document,
    selection: Option<LayerId>,
    mask: MaskId,
) -> Result<(), String> {
    apply_mask_list_edit(doc, selection, mask, masks_with_cycled_mode)
}

/// MASK section の inverted トグル([`cycle_inspector_mask_mode`] と同型)。
pub fn toggle_inspector_mask_inverted(
    doc: &mut Document,
    selection: Option<LayerId>,
    mask: MaskId,
) -> Result<(), String> {
    apply_mask_list_edit(doc, selection, mask, masks_with_toggled_inverted)
}

/// mode 巡回・inverted トグル共通の書き口: 今の一覧を読み、純関数で編集後の
/// 一覧を作り、1回の `Intent::SetMasks` で書く。
fn apply_mask_list_edit(
    doc: &mut Document,
    selection: Option<LayerId>,
    mask: MaskId,
    edit: fn(&[Mask], MaskId) -> Option<Vec<Mask>>,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let masks = doc
        .view()
        .masks(layer)
        .map_err(|error| format!("mask を読めない: {error}"))?;
    let Some(new_masks) = edit(&masks, mask) else {
        return Ok(()); // 対象 mask が居ない(stale click)— 黙って捨てる。
    };
    doc.apply(Intent::SetMasks {
        layer,
        masks: new_masks,
    })
    .map_err(|error| format!("mask を書けない: {error}"))
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// EFFECTS section(B38 編集側 第3切片、裁定184 型別 section 第2号)— stack の
// remove / reorder / bypass の意味と書き口。param の値編集は
// `TransformField::EffectParam` 経由で既存の値セル文法が書くので、ここに
// param の書き口は無い(MASK section と同じ分担)。
// ---------------------------------------------------------------------------

/// 取り除いた後の effect 一覧(純関数 — [`masks_with_cycled_mode`] と同型)。
/// 対象が居なければ `None`(stale click — 呼び手は no-op)。param track の
/// 扱いは [`Message::RemoveEffect`] の doc(残す — 1 click = 1 undo を保つ)。
pub fn effects_with_removed(
    effects: &[EffectInstance],
    target: EffectId,
) -> Option<Vec<EffectInstance>> {
    effects.iter().any(|effect| effect.id == target).then(|| {
        effects
            .iter()
            .filter(|effect| effect.id != target)
            .cloned()
            .collect()
    })
}

/// 1つ上(適用順の前)へ動かした後の一覧。対象が居ない**か既に先頭**なら
/// `None` — 端での click に空の `Intent::SetEffects`(実質無変更の undo 段)を
/// 積まないため(mask 系の「stale click は黙って捨てる」と同じ安全側の拡張)。
pub fn effects_with_moved_up(
    effects: &[EffectInstance],
    target: EffectId,
) -> Option<Vec<EffectInstance>> {
    let index = effects.iter().position(|effect| effect.id == target)?;
    if index == 0 {
        return None;
    }
    let mut out = effects.to_vec();
    out.swap(index - 1, index);
    Some(out)
}

/// 1つ下(適用順の後)へ。[`effects_with_moved_up`] の対 — 末尾なら `None`。
pub fn effects_with_moved_down(
    effects: &[EffectInstance],
    target: EffectId,
) -> Option<Vec<EffectInstance>> {
    let index = effects.iter().position(|effect| effect.id == target)?;
    if index + 1 >= effects.len() {
        return None;
    }
    let mut out = effects.to_vec();
    out.swap(index, index + 1);
    Some(out)
}

/// enabled を裏返した後の一覧([`masks_with_toggled_inverted`] と同型)。
pub fn effects_with_toggled_enabled(
    effects: &[EffectInstance],
    target: EffectId,
) -> Option<Vec<EffectInstance>> {
    effects.iter().any(|effect| effect.id == target).then(|| {
        effects
            .iter()
            .map(|effect| {
                if effect.id == target {
                    EffectInstance {
                        enabled: !effect.enabled,
                        ..effect.clone()
                    }
                } else {
                    effect.clone()
                }
            })
            .collect()
    })
}

/// EFFECTS section の remove — 即1回の `Intent::SetEffects`
/// ([`cycle_inspector_mask_mode`] と同じ即時操作の形)。選択なし・対象なしは
/// 黙って no-op(`Ok(())`)、書き込み失敗だけ `Err` の理由文(M13)。
pub fn remove_inspector_effect(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
) -> Result<(), String> {
    apply_effect_list_edit(doc, selection, |effects| {
        effects_with_removed(effects, effect)
    })
}

/// EFFECTS section の上へ移動([`remove_inspector_effect`] と同型)。端は no-op。
pub fn move_inspector_effect_up(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
) -> Result<(), String> {
    apply_effect_list_edit(doc, selection, |effects| {
        effects_with_moved_up(effects, effect)
    })
}

/// EFFECTS section の下へ移動(同上)。
pub fn move_inspector_effect_down(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
) -> Result<(), String> {
    apply_effect_list_edit(doc, selection, |effects| {
        effects_with_moved_down(effects, effect)
    })
}

/// EFFECTS section の bypass トグル(同上)。
pub fn toggle_inspector_effect_bypass(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
) -> Result<(), String> {
    apply_effect_list_edit(doc, selection, |effects| {
        effects_with_toggled_enabled(effects, effect)
    })
}

/// remove/reorder/bypass 共通の書き口([`apply_mask_list_edit`] と同型):
/// 今の一覧を読み、純関数で編集後の一覧を作り、1回の `Intent::SetEffects` で書く。
/// `edit` が `None` を返したら Intent を出さない(stale click・端 reorder)。
fn apply_effect_list_edit(
    doc: &mut Document,
    selection: Option<LayerId>,
    edit: impl FnOnce(&[EffectInstance]) -> Option<Vec<EffectInstance>>,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let effects = doc
        .view()
        .effects(layer)
        .map_err(|error| format!("effect を読めない: {error}"))?;
    let Some(new_effects) = edit(&effects) else {
        return Ok(());
    };
    doc.apply(Intent::SetEffects {
        layer,
        effects: new_effects,
    })
    .map_err(|error| format!("effect を書けない: {error}"))
}

// ---------------------------------------------------------------------------
// ラベル色チップ(B03)— 巡回の意味と書き口。
// ---------------------------------------------------------------------------

/// チップ click 後の palette index。未割当(`None` — 旧ドキュメントの読み戻し)
/// は先頭(0)から始め、以後は宣言順で一周する([`next_blend_mode`] と同じ
/// 巡回ボタン文法)。`LABEL_PALETTE_LEN` 以上の index が Document に入っていた
/// 場合(起こらないはず — 書き手は全て `% LABEL_PALETTE_LEN` 済み)も
/// 剰余で一覧内へ戻る(`next_blend_mode` の「非対応値は先頭へ」と同じ寛容)。
pub fn next_label_color(current: Option<u8>) -> u8 {
    match current {
        None => 0,
        Some(index) => ((index as usize + 1) % LABEL_PALETTE_LEN) as u8,
    }
}

/// ラベル色チップ — 即1回の `Intent::SetAttrs`(`ToggleHidden` と同じ即時操作の
/// 形、patch は `label_color` 1フィールドのみ)。選択なしは黙って no-op。
/// `None`(未割当へ戻す)への巡回は持たない — 生成時に全 layer が自動割当
/// (`motolii_shell::label_color_for_new_layer`)されるので「色を外す」意図は
/// 束の採用行に無い(B03 の None 行は見送り、RETURN 参照)。
pub fn cycle_inspector_label_color(
    doc: &mut Document,
    selection: Option<LayerId>,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let attrs = doc
        .view()
        .attrs(layer)
        .map_err(|error| format!("attrs を読めない: {error}"))?
        .unwrap_or_default();
    let patch = LayerAttrsPatch {
        label_color: Some(Some(next_label_color(attrs.label_color))),
        ..Default::default()
    };
    doc.apply(Intent::SetAttrs { layer, patch })
        .map_err(|error| format!("ラベル色を書けない: {error}"))
}
// ---------------------------------------------------------------------------
// TEXT section(B46 第1切片、裁定184)— font/size/line-height/tracking/justify
// の意味と書き口。`TextDocumentStyle` は裁定92によりキーフレーム化しない
// (v1)ので、MASK opacity と違い `PropertyId`/`Intent::SetTrack` には乗らない
// — 丸ごと差し替えの `Intent::SetTextDocument`(`SetMasks` と同じ形)を使う。
// ---------------------------------------------------------------------------

/// text_document が未着手の layer に**表示専用**で見せる既定値。**保存しない**
/// — [`apply_text_document_edit`] がここから編集後コピーを作り、実際に値が
/// 変わった時だけ `Intent::SetTextDocument` を出す(`default_vec2` と同じ
/// 「無ければ既定」の形)。
pub fn default_text_document() -> TextDocument {
    TextDocument {
        content: ContentTrack::new(),
        // Lottie/AE とも既定は左揃え。
        justify: TextJustify::Left,
        wrap_size: None,
        styles: vec![default_text_style()],
        slot_id: None,
        ranges: Vec::new(),
        alignment: TextAlignmentOptions::default(),
        runs: Vec::new(),
    }
}

/// スタイル表の既定行(裁定98: `styles[0]` = document 既定値)。この切片は
/// この1行だけを編集する(範囲スタイル表・アニメーターは次切片)。
pub fn default_text_style() -> TextDocumentStyle {
    TextDocumentStyle {
        id: TextStyleId(0),
        font: FontRef::default(),
        size: 100.0,
        fill: [0.0, 0.0, 0.0, 1.0],
        line_height: None,
        tracking: 0.0,
        stroke_color: None,
        stroke_width: 0.0,
        stroke_over_fill: false,
        axes: Vec::new(),
        features: Vec::new(),
    }
}

/// Justify 巡回ボタンの次の値。`TextJustify` の宣言順(Left → Right →
/// Center)をそのまま辿る(`next_mask_mode`/`next_blend_mode` と同じ
/// 「型の宣言順を巡回順の正本にする」判断)。
pub fn next_text_justify(current: TextJustify) -> TextJustify {
    match current {
        TextJustify::Left => TextJustify::Right,
        TextJustify::Right => TextJustify::Center,
        TextJustify::Center => TextJustify::Left,
    }
}

/// 下書き文字列を [`TextField`] の意味で `style` へ適用した新しいコピー
/// (`next_value` の Vec2 保存と同じ「他フィールドは保つ」考え方)。数値として
/// 読めない入力は `Err` の理由文(M13)。`FontFamily` だけは数値変換をしない
/// — 空文字列も許す(「フォント未指定」を表現できる、`FontRef` の既定と同型)。
pub fn applied_text_field(
    style: &TextDocumentStyle,
    field: TextField,
    input: &str,
) -> Result<TextDocumentStyle, String> {
    let mut next = style.clone();
    match field {
        TextField::FontFamily => next.font.family = input.to_owned(),
        TextField::Size => {
            let value =
                parse_number(input).ok_or_else(|| format!("数値として読めない: {input}"))?;
            next.size = value as f32;
        }
        TextField::LineHeight => {
            let value =
                parse_number(input).ok_or_else(|| format!("数値として読めない: {input}"))?;
            next.line_height = Some(value as f32);
        }
        TextField::Tracking => {
            let value =
                parse_number(input).ok_or_else(|| format!("数値として読めない: {input}"))?;
            next.tracking = value as f32;
        }
    }
    Ok(next)
}

/// TEXT section 共通の書き口(`apply_mask_list_edit` と同型): 選択が無ければ
/// no-op、選択層の `TextDocument` を読み(無ければ [`default_text_document`])、
/// `edit` で編集後コピーを作り、**実際に値が変わった時だけ**1回の
/// `Intent::SetTextDocument` を出す(決定7「同値は Undo を積まない」と同じ
/// 判断 — Reset ボタンが既に既定値の時・打鍵で同値を submit した時の両方を
/// この1箇所で満たす)。
fn apply_text_document_edit(
    doc: &mut Document,
    selection: Option<LayerId>,
    edit: impl FnOnce(TextDocument) -> Result<TextDocument, String>,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let current = doc
        .view()
        .text_document(layer)
        .map_err(|error| format!("text document を読めない: {error}"))?;
    let base = current.clone().unwrap_or_else(default_text_document);
    let next = edit(base)?;
    let unchanged = match &current {
        Some(existing) => existing == &next,
        None => next == default_text_document(),
    };
    if unchanged {
        return Ok(());
    }
    doc.apply(Intent::SetTextDocument {
        layer,
        document: next,
    })
    .map_err(|error| format!("text document を書けない: {error}"))
}

/// TEXT section の text_input 系フィールド — 下書きを確定して1回の
/// `Intent::SetTextDocument` を出す(1 gesture = 1 undo、`commit_inspector_field`
/// と同じ形)。下書きが無い・別 field の submit・選択が無い、のいずれも
/// `Ok(())`(何もしない)。
pub fn commit_text_field(
    doc: &mut Document,
    draft: &mut Option<TextFieldDraft>,
    selection: Option<LayerId>,
    field: TextField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.field != field {
        // 別の field の submit(起こらないはずだが、安全側で下書きを戻す —
        // `commit_inspector_field` と同じ判断)。
        *draft = Some(taken);
        return Ok(());
    }
    apply_text_document_edit(doc, selection, |mut document| {
        let style = document
            .styles
            .first()
            .cloned()
            .unwrap_or_else(default_text_style);
        let new_style = applied_text_field(&style, field, &taken.text)?;
        if document.styles.is_empty() {
            document.styles.push(new_style);
        } else {
            document.styles[0] = new_style;
        }
        Ok(document)
    })
}

/// Justify の巡回 — 即1回の `Intent::SetTextDocument`(`CycleBlendMode`/
/// `CycleMaskMode` と同じ即時操作の形)。選択なしは黙って no-op。
pub fn cycle_text_justify(doc: &mut Document, selection: Option<LayerId>) -> Result<(), String> {
    apply_text_document_edit(doc, selection, |mut document| {
        document.justify = next_text_justify(document.justify);
        Ok(document)
    })
}

/// Line Height を Auto(`None`)へ戻す(`ResetSpeed` と同じ即時操作の形)。
/// styles が空のまま(まだ何も書かれていない)なら既に Auto —
/// [`apply_text_document_edit`] の同値判定が no-op にする。
pub fn reset_text_line_height(doc: &mut Document, selection: Option<LayerId>) -> Result<(), String> {
    apply_text_document_edit(doc, selection, |mut document| {
        if let Some(style) = document.styles.first_mut() {
            style.line_height = None;
        }
        Ok(document)
    })
}

/// Tracking を 0 へ戻す(map「Reset tracking to 0」、[`reset_text_line_height`]
/// と同型)。
pub fn reset_text_tracking(doc: &mut Document, selection: Option<LayerId>) -> Result<(), String> {
    apply_text_document_edit(doc, selection, |mut document| {
        if let Some(style) = document.styles.first_mut() {
            style.tracking = 0.0;
        }
        Ok(document)
    })
}
/// [`project`] が組む行の decimals は field ごとに固定(Position/Scale/Anchor=3、
/// Rotation=1、Opacity=0)。click→type 編集の下書き初期値を作るのに要る
/// (`TransformRowProjection::decimals` を持つ行を毎回作り直さずに済む)。
pub fn field_decimals(field: TransformField) -> usize {
    match field {
        TransformField::Rotation => 1,
        TransformField::Opacity | TransformField::MaskOpacity(_) => 0,
        // Glow 既定(1.0/0.75/1.0)の桁がそのまま読める最小の桁。
        TransformField::EffectParam(_, _) => 2,
        _ => 3,
    }
}


// ---------------------------------------------------------------------------
// drag-to-scrub — 1px の cursor 移動を「表示単位」の量へ写す(editor registry)
// ---------------------------------------------------------------------------

/// `field` の drag 感度(1px あたりの表示単位の変化量)。**tokens ではなくここに
/// 置く** — 寸法・色ではなく値の型に紐づく振る舞いなので([`motolii_tokens_rs`] は
/// 裁定117により寸法・色専用、Document 由来でも値の意味でもない — 型別 editor の
/// 一部としてここへ置く)。
///
/// | field                | 1px    | 理由 |
/// |-----------------------|--------|------|
/// | Position X/Y/Z        | 1.0    | 画面座標と1:1(After Effects 系の慣習) |
/// | Anchor X/Y             | 1.0    | Position と同じ空間感覚(単位が同じ) |
/// | Scale X/Y              | 0.01   | 既定1.0からの微調整域 — 1px=1.0では数pxで0や負へ振り切れる |
/// | Rotation                | 0.5(度)| 720px の drag で1周(360度)分 — 粗すぎず細かすぎない |
/// | Opacity                  | 1(%)   | 0〜100の全域が100pxの drag で動く |
fn drag_step_per_pixel(field: TransformField) -> f64 {
    match field {
        TransformField::PositionX
        | TransformField::PositionY
        | TransformField::PositionZ
        | TransformField::AnchorX
        | TransformField::AnchorY => 1.0,
        TransformField::ScaleX | TransformField::ScaleY => 0.01,
        TransformField::Rotation => 0.5,
        // effect param は Scale と同じ微調整域(既定 0.75〜1.0 前後の値 —
        // 1px=1.0 では数 px で意味域を振り切る、Scale と同じ理由)。
        TransformField::EffectParam(_, _) => 0.01,
        // mask opacity は layer Opacity と同じ感度(0〜100% が 100px で動く)。
        TransformField::Opacity | TransformField::MaskOpacity(_) => 1.0,
    }
}

/// Shift 押下中の微調整係数(発注書「Shift+drag = 1/10 微調整」)。
pub const DRAG_SHIFT_FACTOR: f64 = 0.1;

/// press 開始点からの x 差分(px)を「表示単位」の新しい値へ写す。`fine` は
/// Shift 押下中かどうか([`DRAG_SHIFT_FACTOR`] を掛ける)。純粋関数 — 呼び手
/// (`continue_field_drag`)が結果を `next_value` へ渡して store の
/// `Value` へ変換する。
pub fn dragged_value(field: TransformField, start_value: f64, delta_px: f32, fine: bool) -> f64 {
    let step = drag_step_per_pixel(field);
    let factor = if fine { DRAG_SHIFT_FACTOR } else { 1.0 };
    start_value + f64::from(delta_px) * step * factor
}

/// drag(または click→type 編集)を始める前に読む、`field` の現在値。
/// **投影から読むだけ** — `project` が計算した表示単位の値をそのまま使う
/// (Opacity の % 換算などを2箇所に書かない)。対応する field が投影に無い
/// (または `editable=false` の穴)なら `None`(呼び手はドラッグも編集も
/// 始めない)。present な成分は常に editable(Q0、2026-08-22 発注)なので、
/// キー持ち track もここを通って drag/type 編集できる。
///
/// 戻り値の第2要素は Vec2 系(Position/Scale/Anchor)の「動かさない方の成分」
/// ([`next_value`] にそのまま渡す) — scalar 系(Z/Rotation/Opacity)では未使用
/// (`[0.0, 0.0]` のダミー)。
pub fn drag_origin(
    selection: &SelectionProjection,
    field: TransformField,
) -> Option<(f64, [f64; 2])> {
    for row in &selection.transform {
        match &row.value {
            RowValue::Vector(components) => {
                if let Some(slot) = components.iter().find(|s| s.field == Some(field)) {
                    let current_vec2 = [components[0].value, components[1].value];
                    return slot.editable.then_some((slot.value, current_vec2));
                }
            }
            RowValue::Scalar(slot) => {
                if slot.field == Some(field) {
                    return slot.editable.then_some((slot.value, [0.0, 0.0]));
                }
            }
        }
    }
    // MASK section の opacity 行(scalar のみ — mask に Vec2 の値セルは無い)。
    for mask_row in &selection.masks {
        if let RowValue::Scalar(slot) = &mask_row.opacity.value {
            if slot.field == Some(field) {
                return slot.editable.then_some((slot.value, [0.0, 0.0]));
            }
        }
    }
    // EFFECTS section の param 行(scalar のみ — Glow カタログに Vec2 は無い)。
    for effect_row in &selection.effects {
        for param_row in &effect_row.params {
            if let RowValue::Scalar(slot) = &param_row.value {
                if slot.field == Some(field) {
                    return slot.editable.then_some((slot.value, [0.0, 0.0]));
                }
            }
        }
    }
    None
}

/// text_input へ確定的な `Id` を割る — click→type 編集へ切り替わった直後、
/// `iced::widget::operation::focus` でこのセルへフォーカスを戻すために要る
/// (mouse_area は press を own できても、フォーカスは text_input 自身の仕事
/// — click 直後にはまだ text_input が木に無いので自動フォーカスされない)。
pub fn field_input_id(field: TransformField) -> iced::widget::Id {
    // mask opacity / effect param は id が動的(枚数は静的に決まらない)。fork の
    // `Id::new` は `&'static str` 限定だが `From<String>`(Cow::Owned)がある。
    if let TransformField::MaskOpacity(mask) = field {
        return iced::widget::Id::from(format!("inspector-field-mask-{mask}-opacity"));
    }
    if let TransformField::EffectParam(effect, param) = field {
        return iced::widget::Id::from(format!(
            "inspector-field-effect-{effect}-{}",
            param.name()
        ));
    }
    let name: &'static str = match field {
        TransformField::PositionX => "inspector-field-position-x",
        TransformField::PositionY => "inspector-field-position-y",
        TransformField::PositionZ => "inspector-field-position-z",
        TransformField::ScaleX => "inspector-field-scale-x",
        TransformField::ScaleY => "inspector-field-scale-y",
        TransformField::Rotation => "inspector-field-rotation",
        TransformField::Opacity => "inspector-field-opacity",
        TransformField::AnchorX => "inspector-field-anchor-x",
        TransformField::AnchorY => "inspector-field-anchor-y",
        TransformField::MaskOpacity(_) | TransformField::EffectParam(_, _) => {
            unreachable!("上の early return が拾う")
        }
    };
    iced::widget::Id::new(name)
}

// ---------------------------------------------------------------------------
// 投影 — Document の写しではなく、1度描くための使い捨て値(`timeline_pane::rows` と同じ形)
// ---------------------------------------------------------------------------

/// 1成分(X/Y/Z、または scalar 1個)の投影。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentSlot {
    pub axis: &'static str,
    /// store の意味モデルにこの軸があるか。無ければ表示は `—`(mock の
    /// `emptyComponent` と同じ — Rotation の X/Y、Scale/Anchor の Z は Motolii の
    /// 2.5D モデル(裁定113)に無いので `false`)。
    pub present: bool,
    /// 表示単位での値(Opacity だけ % — store は 0..1)。`present=false` なら無意味。
    pub value: f64,
    /// 編集可能か。**present な成分は常に `true`**(Q0: キー数で触れなくなる
    /// 状態を作らない — 2026-08-22 発注で旧規則「keys.len()<=1 のみ編集可」を
    /// 撤去した。キー持ち track の編集は playhead へのキー upsert =
    /// [`edited_value_track`])。`present=false` の穴だけ `false`。
    pub editable: bool,
    /// この track が実キーを持つか([`has_real_keys`])。表示は accent —
    /// 「編集すると記録される」ことの視覚合図(AE のキー付き値と同型)。
    pub keyed: bool,
    /// この成分が編集される時に動く field。`present=false` なら `None`。
    pub field: Option<TransformField>,
}

fn absent_component(axis: &'static str) -> ComponentSlot {
    ComponentSlot {
        axis,
        present: false,
        value: 0.0,
        editable: false,
        keyed: false,
        field: None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RowValue {
    Vector([ComponentSlot; 3]),
    Scalar(ComponentSlot),
}

/// Key セルの投影(K1)。`row` は click 時に `Message::KeyPressed` が運ぶ宛先、
/// `state` は3状態 oracle([`key_cell_state`])の結果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyCellProjection {
    pub row: KeyRow,
    pub state: KeyCellState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformRowProjection {
    pub label: &'static str,
    pub value: RowValue,
    pub decimals: usize,
    /// Key 列(K1)。5行全部が持つ — 「触れそうで触れない」空予約は残さない(Q0)。
    pub key: KeyCellProjection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttrsProjection {
    pub name: String,
    pub hidden: bool,
    /// **クリックで巡回するボタンの現在値**(BL2)。`Message::CycleBlendMode` を
    /// 押すたび engine が対応する次の mode へ進む(現状 Normal→Add→Normal、
    /// [`SUPPORTED_BLEND_MODES`] 参照)。`BlendMode` の `Debug` 表示をそのまま使う
    /// (`Normal`/`Add`/…)。
    pub blend_mode: String,
    /// `LayerTiming.speed` の表示 %(100=等速、SP1 第一波)。`meta` が読めない
    /// (起こらないはず)場合は等速(100.0)。[`speed_percent`] がこの写像の実体。
    pub speed_percent: f64,
    /// ラベル色の palette index(B03、`LayerAttrs.label_color` の写し)。
    /// `None` = 未割当 — チップは timeline スウォッチと同じフォールバック
    /// (`way_timeline`)で塗る(`lane_bar::swatch_color` と同じ源・同じ既定)。
    pub label_color: Option<u8>,
}

/// effect 1本ぶんの投影(EFFECTS section、B38 第3切片・裁定184 型別 section
/// 第2号)。静止する部分(名前・enabled)は store の [`EffectInstance`] の写し、
/// 動く部分(param)は既存の値セル行文法([`TransformRowProjection`])を
/// そのまま再利用する — [`MaskRowProjection`] と同じ分担。
#[derive(Clone, Debug, PartialEq)]
pub struct EffectRowProjection {
    pub id: EffectId,
    /// 表示名([`plugin_display_name`] — 既知 plugin は人間可読名、未知は
    /// plugin_id そのまま)。
    pub name: String,
    /// `effects/effect/en`。false = bypass 中(消えてはいない)。
    pub enabled: bool,
    /// param 値行([`plugin_params`] のカタログ順)。未知 plugin は空 —
    /// store は catalog を知らないので param 行を捏造しない(M13)。
    pub params: Vec<TransformRowProjection>,
}

/// mask 1枚ぶんの投影(MASK section、B02 第1切片・裁定184)。静止する部分
/// (mode/inverted)は store の [`Mask`] の写し、動く部分(opacity)は既存の
/// 値セル行文法([`TransformRowProjection`])をそのまま再利用する — view は
/// opacity 行を [`transform_row`] と同じ関数で描く(新しい編集文法を発明しない)。
#[derive(Clone, Debug, PartialEq)]
pub struct MaskRowProjection {
    pub id: MaskId,
    pub mode: MaskMode,
    pub inverted: bool,
    /// `mask.{id}.opacity` track の値行(label="Opacity"・decimals=0・% 表示 —
    /// layer Opacity 行と同じ形)。Key 列は [`KeyRow::MaskOpacity`]。
    pub opacity: TransformRowProjection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionProjection {
    pub layer: LayerId,
    /// ident 帯の種別ラベル(mock の `.ident s` = 「clip · 0 shared FX」相当)。
    /// **`LayerSource` の実データから引く** — mock の「shared FX」は store に
    /// 対応する概念が無い(effect に "shared" フラグは無い)ので、種別だけ載せ、
    /// FX 件数は捏造しない(M13: 無い意味を有るふりで出さない)。
    pub kind: &'static str,
    pub transform: Vec<TransformRowProjection>,
    pub attrs: AttrsProjection,
    /// MASK section の行(store の並びどおり)。**空なら section 自体を出さない**
    /// (Q0: mask の無い layer に効かない chrome を並べない — `empty_state` と
    /// 同じ判断)。裁定184「型別 section で拡張」の第1号。
    pub masks: Vec<MaskRowProjection>,
    /// EFFECTS section の行(store の並び = 適用順どおり)。空なら section 自体を
    /// 出さない(masks と同じ Q0 判断)。裁定184 型別 section の第2号。
    pub effects: Vec<EffectRowProjection>,
    /// TEXT section の投影(B46 第1切片、裁定184)。`LayerSource::Text` の
    /// layer でのみ `Some`(MASK section と同じ「型が合わない chrome を出さ
    /// ない」判断、Q0)。`text_document` が未着手でも [`default_text_document`]
    /// で表示専用の既定値を作る(store には書かない、`default_vec2` と同じ形)。
    pub text: Option<TextSectionProjection>,
}

/// TEXT section の投影(裁定98: `styles[0]` = document 既定行のみを対象 —
/// 範囲スタイル表・アニメーターは次切片)。Key 列は無い — 対象フィールドは
/// どれも `KeyframeTrack` に乗らない静止値(裁定92)なので、Position/Scale 等
/// の3状態 oracle は適用対象外。
#[derive(Clone, Debug, PartialEq)]
pub struct TextSectionProjection {
    /// `text-document f`(Font Family)。
    pub font_family: String,
    /// `text-document s`(Font Size)。
    pub size: f32,
    /// `text-document lh`(Line Height)。`None` = Auto。
    pub line_height: Option<f32>,
    /// `text-document tr`(Tracking)。
    pub tracking: f32,
    /// `text-document j`(Justify)。
    pub justify: TextJustify,
}

/// [`SelectionProjection::kind`] の出典。`LayerSource` の variant 名をそのまま
/// 小文字化した語彙(発明ではなく store の型そのものを読める言葉にしただけ)。
fn source_kind_label(source: &LayerSource) -> &'static str {
    match source {
        LayerSource::Solid { .. } => "solid",
        LayerSource::Media { .. } => "media",
        LayerSource::Null => "null",
        LayerSource::Shape => "shape",
        LayerSource::Text => "text",
        // 裁定173。H3(親選択 UI/Inspector の parent 表示)は非目標のまま —
        // ここは非網羅マッチをコンパイラに教えてもらって埋めた最小限の一言のみ。
        LayerSource::Group => "group",
    }
}

fn scalar_component(
    store: &StoreView<'_>,
    layer: LayerId,
    name: &str,
    axis: &'static str,
    field: TransformField,
    t: RationalTime,
    default: f64,
) -> Result<ComponentSlot, StoreError> {
    let property = PropertyId::new(name)?;
    let track = store.track(layer, &property)?;
    let keyed = has_real_keys(track.as_ref());
    let value = match store.value_at(layer, &property, t)? {
        Some(Value::F64(v)) => v,
        _ => default,
    };
    Ok(ComponentSlot {
        axis,
        present: true,
        value,
        editable: true,
        keyed,
        field: Some(field),
    })
}

fn vec2_components(
    store: &StoreView<'_>,
    layer: LayerId,
    name: &str,
    field_x: TransformField,
    field_y: TransformField,
    t: RationalTime,
    default: [f64; 2],
) -> Result<[ComponentSlot; 2], StoreError> {
    let property = PropertyId::new(name)?;
    let track = store.track(layer, &property)?;
    let keyed = has_real_keys(track.as_ref());
    let [x, y] = match store.value_at(layer, &property, t)? {
        Some(Value::Vec2(v)) => [v[0], v[1]],
        _ => default,
    };
    Ok([
        ComponentSlot {
            axis: "X",
            present: true,
            value: x,
            editable: true,
            keyed,
            field: Some(field_x),
        },
        ComponentSlot {
            axis: "Y",
            present: true,
            value: y,
            editable: true,
            keyed,
            field: Some(field_y),
        },
    ])
}

/// `store`/`session` から選択層の Inspector 投影を組み立てる。**読むだけ**。
/// 選択なし・選択層が削除済み(present でない)・comp が無い、のいずれも `Ok(None)`
/// (M13: 壊れているのではなく「まだ映す物が無い」)。
///
/// **`&Session` を直接取る**(裁定160 切片7 以降、crate doc 参照)——`Session`
/// は `motolii-shell-state` leaf crate に住む(`motolii-timeline-pane` と同じ
/// 依存)ので、root(`motolii-shell`)を経由せずにここへ持ち込める。切片7以前は
/// `Session` が `motolii-shell` root に残っていたため、循環を避けて
/// `selection`/`playhead` の2引数へ分解する回避策を取っていたが、切片7の
/// leaf crate 化でこの回避策は不要になった。
pub fn project(
    store: &StoreView<'_>,
    session: &Session,
) -> Result<Option<SelectionProjection>, StoreError> {
    let Some(layer) = session.selection else {
        return Ok(None);
    };
    if !store.has_layer(layer) {
        return Ok(None);
    }
    let Some(composition) = store.composition()? else {
        return Ok(None);
    };
    let t = RationalTime::try_from_frame(session.playhead, composition.fps)
        .unwrap_or(RationalTime::ZERO);

    // Key 列(K1)の3状態を行ごとに読む(表示と click 判定の共通 oracle =
    // `key_cell_state`)。playhead は timeline と同じ frame 粒度で照合する。
    let key_cell = |row: KeyRow| -> Result<KeyCellProjection, StoreError> {
        let property = key_row_property_id(row)?;
        let track = store.track(layer, &property)?;
        Ok(KeyCellProjection {
            row,
            state: key_cell_state(track.as_ref(), session.playhead, composition.fps),
        })
    };

    let position_xy = vec2_components(
        store,
        layer,
        property::POSITION,
        TransformField::PositionX,
        TransformField::PositionY,
        t,
        [0.0, 0.0],
    )?;
    let position_z = scalar_component(
        store,
        layer,
        property::POSITION_Z,
        "Z",
        TransformField::PositionZ,
        t,
        0.0,
    )?;
    let position_row = TransformRowProjection {
        label: "Position",
        value: RowValue::Vector([position_xy[0], position_xy[1], position_z]),
        decimals: 3,
        key: key_cell(KeyRow::Position)?,
    };

    let scale_xy = vec2_components(
        store,
        layer,
        property::SCALE,
        TransformField::ScaleX,
        TransformField::ScaleY,
        t,
        [1.0, 1.0],
    )?;
    let scale_row = TransformRowProjection {
        label: "Scale",
        value: RowValue::Vector([scale_xy[0], scale_xy[1], absent_component("Z")]),
        decimals: 3,
        key: key_cell(KeyRow::Scale)?,
    };

    let rotation_z = scalar_component(
        store,
        layer,
        property::ROTATION,
        "Z",
        TransformField::Rotation,
        t,
        0.0,
    )?;
    let rotation_row = TransformRowProjection {
        label: "Rotation",
        value: RowValue::Vector([absent_component("X"), absent_component("Y"), rotation_z]),
        decimals: 1,
        key: key_cell(KeyRow::Rotation)?,
    };

    let mut opacity = scalar_component(
        store,
        layer,
        property::OPACITY,
        "Opacity",
        TransformField::Opacity,
        t,
        1.0,
    )?;
    opacity.value *= 100.0; // store は 0..1、表示は %。
    let opacity_row = TransformRowProjection {
        label: "Opacity",
        value: RowValue::Scalar(opacity),
        decimals: 0,
        key: key_cell(KeyRow::Opacity)?,
    };

    let anchor_xy = vec2_components(
        store,
        layer,
        property::ANCHOR,
        TransformField::AnchorX,
        TransformField::AnchorY,
        t,
        [0.0, 0.0],
    )?;
    let anchor_row = TransformRowProjection {
        label: "Anchor",
        value: RowValue::Vector([anchor_xy[0], anchor_xy[1], absent_component("Z")]),
        decimals: 3,
        key: key_cell(KeyRow::Anchor)?,
    };

    let attrs = store.attrs(layer)?.unwrap_or_default();
    // `kind`/`speed_percent` は同じ `meta()` から読む(2回叩かない)。
    let meta = store.meta(layer)?;
    let attrs_projection = AttrsProjection {
        name: attrs.name,
        hidden: attrs.hidden,
        blend_mode: format!("{:?}", attrs.blend_mode),
        speed_percent: meta
            .as_ref()
            .map(|meta| speed_percent(meta.timing.speed.num(), meta.timing.speed.den()))
            .unwrap_or(100.0),
        label_color: attrs.label_color,
    };

    let kind = meta
        .as_ref()
        .map(|meta| source_kind_label(&meta.source))
        .unwrap_or("layer");

    // MASK section(B02 第1切片): store の並びどおり。opacity 行は layer
    // Opacity 行と同じ組み方(track を読み、無ければ既定 1.0 → 表示 %)。
    let mut mask_rows = Vec::new();
    for mask in store.masks(layer)? {
        let property = PropertyId::mask_opacity(mask.id);
        let track = store.track(layer, &property)?;
        let keyed = has_real_keys(track.as_ref());
        let value = match store.value_at(layer, &property, t)? {
            Some(Value::F64(v)) => v,
            _ => 1.0, // `motolii-store::mask` の既定(比 1.0 = 全掩)。
        };
        let state = key_cell_state(track.as_ref(), session.playhead, composition.fps);
        mask_rows.push(MaskRowProjection {
            id: mask.id,
            mode: mask.mode,
            inverted: mask.inverted,
            opacity: TransformRowProjection {
                label: "Opacity",
                value: RowValue::Scalar(ComponentSlot {
                    axis: "Opacity",
                    present: true,
                    value: value * 100.0, // store は 0..1、表示は %(layer Opacity と同じ)。
                    editable: true,
                    keyed,
                    field: Some(TransformField::MaskOpacity(mask.id)),
                }),
                decimals: 0,
                key: KeyCellProjection {
                    row: KeyRow::MaskOpacity(mask.id),
                    state,
                },
            },
        });
    }

    // EFFECTS section(B38 第3切片): store の並び = 適用順どおり。param 行は
    // 既知 plugin のカタログ([`plugin_params`])分だけ — track を読み、無ければ
    // engine 既定の写し([`GlowParam::default_value`])。表示 = store 単位
    // (opacity 系と違い % 換算しない)。
    let mut effect_rows = Vec::new();
    for effect in store.effects(layer)? {
        let mut param_rows = Vec::new();
        for param in plugin_params(&effect.plugin_id) {
            let property = PropertyId::effect_param(effect.id, param.name())?;
            let track = store.track(layer, &property)?;
            let keyed = has_real_keys(track.as_ref());
            let value = match store.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => v,
                _ => param.default_value(),
            };
            let state = key_cell_state(track.as_ref(), session.playhead, composition.fps);
            param_rows.push(TransformRowProjection {
                label: param.label(),
                value: RowValue::Scalar(ComponentSlot {
                    axis: param.label(),
                    present: true,
                    value,
                    editable: true,
                    keyed,
                    field: Some(TransformField::EffectParam(effect.id, *param)),
                }),
                decimals: field_decimals(TransformField::EffectParam(effect.id, *param)),
                key: KeyCellProjection {
                    row: KeyRow::EffectParam(effect.id, *param),
                    state,
                },
            });
        }
        effect_rows.push(EffectRowProjection {
            id: effect.id,
            name: plugin_display_name(&effect.plugin_id).to_owned(),
            enabled: effect.enabled,
            params: param_rows,
        });
    }

    // TEXT section(B46 第1切片、裁定184): `LayerSource::Text` の layer での
    // み現れる(MASK section の「空なら出さない」と同じ Q0 判断 — ここは
    // 「型が合わない」判断)。`text_document` 未着手でも [`default_text_document`]
    // で表示専用の既定値を作る(store には書かない、Position 等の
    // `default_vec2` と同じ形)。
    let text = match meta.as_ref().map(|meta| &meta.source) {
        Some(LayerSource::Text) => {
            let document = store.text_document(layer)?.unwrap_or_else(default_text_document);
            let style = document
                .styles
                .first()
                .cloned()
                .unwrap_or_else(default_text_style);
            Some(TextSectionProjection {
                font_family: style.font.family,
                size: style.size,
                line_height: style.line_height,
                tracking: style.tracking,
                justify: document.justify,
            })
        }
        _ => None,
    };

    Ok(Some(SelectionProjection {
        layer,
        kind,
        transform: vec![
            position_row,
            scale_row,
            rotation_row,
            opacity_row,
            anchor_row,
        ],
        attrs: attrs_projection,
        masks: mask_rows,
        effects: effect_rows,
        text,
    }))
}

// ---------------------------------------------------------------------------
// drag-to-scrub、進行中の一時状態(裁定160 切片8で lib.rs から移設)。
// ---------------------------------------------------------------------------

/// Inspector 値セルの drag-to-scrub、進行中の一時状態。**Document ではない**
/// (`FieldDraft` と同じ「pane が持つ transient」の形)。値そのものの置き場は
/// `Document` の transient overlay(`Document::set_transient`)— ここは overlay
/// の宛先と、click/drag 判定・確定時の Intent 組み立てに要る最小限だけを持つ。
///
/// **置き場(`motolii_shell::Shell::inspector_drag: Option<FieldDragState>`)は
/// 移設していない**(crate doc 参照) — 型定義とこれを読み書きする自由関数
/// ([`start_field_drag`]/[`continue_field_drag`]/[`finish_field_drag`]/
/// [`cancel_field_interaction`])だけがここにある。
pub struct FieldDragState {
    field: TransformField,
    layer: LayerId,
    /// press 時点の playhead(frame)と fps。確定時のキー upsert
    /// ([`edited_value_track`])の宛先 — drag の起点値は press 時点の
    /// playhead で読んだ値なので、確定の宛先も同じ時刻に固定する。
    playhead_frame: i64,
    fps: Fps,
    /// press 時点の表示単位の値([`drag_origin`] が投影から読む)。確定
    /// Intent・Esc(overlay を外すだけで使わない)双方が参照する起点。
    start_value: f64,
    /// Vec2 系(Position/Scale/Anchor)の動かさない方の成分。scalar 系では未使用。
    current_vec2: [f64; 2],
    /// 最初の `PointerMoved` で確定する基準 x(window 座標)。`None` の間は
    /// click か drag かまだ未確定 — 確定前に値を動かすと press 直後の
    /// sub-pixel な揺れで値が動いてしまう。
    origin_x: Option<f32>,
    /// 少なくとも1回 `set_transient` を呼んだか。release 時の click/drag 判定と、
    /// Esc で overlay を外す必要があるかどうかの両方に使う。
    moved: bool,
    /// 直近の `set_transient` に渡した値。release の確定 Intent はこれをそのまま
    /// 1回 `apply` する — pointer の最終座標を release 時に持っていない
    /// (`PointerReleased` は位置を運ばない)ので、最後に計算した値をここへ
    /// 持ち回す。`moved` が `false` の間は未使用。
    last_value: Option<Value>,
}

// ---------------------------------------------------------------------------
// 書き口(裁定160 切片8で lib.rs から移設): `&mut Document`/`&mut Option<_>`
// 下書き・`&mut Option<FieldDragState>` を明示引数で受け取る自由関数。
// 呼び出し側(`motolii_shell::Shell::update_inspector` + 個々の glue メソッド)
// が `self.doc`/`self.session.selection` 等をそのまま貸す — pane crate は
// `Shell` を持てない(root → pane の一方向依存、循環禁止)ための形
// (`motolii_settings_pane` の同型セクションと同じ判断)。
// ---------------------------------------------------------------------------

/// Inspector の Transform 行 — 下書きを確定して1回の `Intent::SetTrack` を出す
/// (1 gesture = 1 undo)。数値として読めない・書き込み失敗は**黙って消さず**
/// `Err` の理由文を返す(M13、呼び出し側が status 帯へ渡す)。下書きが無い・
/// 別 field の submit・選択が無い、のいずれも `Ok(())`(何もしない)。
///
/// **書く track の意味は [`edited_value_track`]**(AE 作法、2026-08-22 発注):
/// キー無しなら静的値の書き換え、キー持ちなら playhead へのキー upsert。
/// 旧規則「2キー以上は編集拒否」は撤去した(Q0 — 値セルは常に編集可能)。
pub fn commit_inspector_field(
    doc: &mut Document,
    draft: &mut Option<FieldDraft>,
    selection: Option<LayerId>,
    playhead_frame: i64,
    fps: Fps,
    field: TransformField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.field != field {
        // 別の field の submit(起こらないはずだが、安全側で下書きを戻す)。
        *draft = Some(taken);
        return Ok(());
    }
    let Some(layer) = selection else {
        return Ok(());
    };
    let Some(input) = parse_number(&taken.text) else {
        return Err(format!("数値として読めない: {}", taken.text));
    };
    let Ok(property) = property_id(field) else {
        return Err("property を作れない".to_owned());
    };
    let playhead_time = RationalTime::try_from_frame(playhead_frame, fps)
        .map_err(|error| format!("playhead を時刻へ写せない: {error}"))?;

    let store = doc.view();
    let track = store.track(layer, &property).ok().flatten();
    let current_vec2 = match store.value_at(layer, &property, playhead_time) {
        Ok(Some(Value::Vec2(v))) => v,
        _ => default_vec2(field),
    };
    let value = next_value(field, input, current_vec2);
    let new_track = edited_value_track(track.as_ref(), playhead_frame, fps, value)?;
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: new_track,
    })
    .map_err(|error| format!("値を書けない: {error}"))
}

/// Attrs の Name 欄 — 下書きを確定して1回の `Intent::SetAttrs` を出す。下書きが
/// 無い・選択が無い、のいずれも `Ok(())`(何もしない)。
pub fn commit_inspector_name(
    doc: &mut Document,
    draft: &mut Option<String>,
    selection: Option<LayerId>,
) -> Result<(), String> {
    let Some(text) = draft.take() else {
        return Ok(());
    };
    let Some(layer) = selection else {
        return Ok(());
    };
    let patch = LayerAttrsPatch {
        name: Some(text),
        ..Default::default()
    };
    doc.apply(Intent::SetAttrs { layer, patch })
        .map_err(|error| format!("名前を書けない: {error}"))
}

/// 値セルの press — click か drag かはまだ未確定(`FieldDragState::origin_x` が
/// `None` のまま)。選択なし・対応する field が投影に無い、のいずれも黙って
/// 無視([`commit_inspector_field`] と同じ二重の柵)。既に別の drag が進行中
/// なら多重起動しない。`playhead_frame`/`fps` は press 時点の物を捕まえて
/// 確定([`finish_field_drag`] のキー upsert)の宛先に固定する。
pub fn start_field_drag(
    drag: &mut Option<FieldDragState>,
    selection: Option<LayerId>,
    projection: Option<&SelectionProjection>,
    field: TransformField,
    playhead_frame: i64,
    fps: Fps,
) {
    if drag.is_some() {
        return; // 既に別の drag が進行中 — 多重起動しない
    }
    let Some(layer) = selection else {
        return;
    };
    let Some(selection_projection) = projection else {
        return;
    };
    let Some((start_value, current_vec2)) = drag_origin(selection_projection, field) else {
        return;
    };
    *drag = Some(FieldDragState {
        field,
        layer,
        playhead_frame,
        fps,
        start_value,
        current_vec2,
        origin_x: None,
        moved: false,
        last_value: None,
    });
}

/// window 全体の cursor 移動。drag が armed/dragging でなければ即 no-op。
/// **1px = 感度表の刻み**([`dragged_value`])。press 直後の最初の move は基準点を
/// 確定するだけで値は動かさない(そうしないと press した瞬間の sub-pixel な
/// 揺れで値が動く)。
///
/// **transient overlay(`Document::set_transient`)を毎 move 呼ぶだけ** —
/// `edit timeline` には一切触れないので、undo/redo の意味論(`revision()`)は
/// drag 中ずっと不変。
pub fn continue_field_drag(
    doc: &mut Document,
    drag: &mut Option<FieldDragState>,
    point: iced::Point,
    fine: bool,
) {
    let Some(state) = drag.as_mut() else {
        return;
    };
    let Some(origin_x) = state.origin_x else {
        state.origin_x = Some(point.x);
        return;
    };

    let delta_px = point.x - origin_x;
    if delta_px == 0.0 && !state.moved {
        return; // まだ実質的に動いていない — click 候補のまま据え置く
    }

    let field = state.field;
    let layer = state.layer;
    let start_value = state.start_value;
    let current_vec2 = state.current_vec2;

    let Ok(property) = property_id(field) else {
        return;
    };
    let new_display = dragged_value(field, start_value, delta_px, fine);
    let value = next_value(field, new_display, current_vec2);

    doc.set_transient(layer, property, value.clone());
    if let Some(state) = drag.as_mut() {
        state.moved = true;
        state.last_value = Some(value);
    }
}

/// 左クリック release(window 全体から)。**drag が実際に動いていたら確定**:
/// 最後の transient 値そのものを1回の本編集 `Intent` として `apply` してから
/// `clear_transient`(1 gesture = 1 undo、overlay を残さない)。
///
/// **`Ok(Some(field))`**: drag が動かないまま release された(click) —
/// 呼び出し側は `field` で type 編集へ切り替える
/// (`motolii_shell::Shell::enter_field_editing`、focus task の構築は crate doc
/// のとおり root 側の仕事)。**`Ok(None)`**: drag が実際に動いた(確定済み)、
/// または drag 自体が無かった — 呼び出し側の追加作業なし。**`Err`**: 確定
/// Intent の書き込みが失敗した理由文(呼び出し側が status 帯へ渡す —
/// `clear_transient` は書き込み失敗時も必ず呼ぶ、元実装と同じ、overlay を
/// 残さないため)。
pub fn finish_field_drag(
    doc: &mut Document,
    drag: &mut Option<FieldDragState>,
) -> Result<Option<TransformField>, String> {
    let Some(state) = drag.take() else {
        return Ok(None);
    };
    if !state.moved {
        return Ok(Some(state.field));
    }
    let Ok(property) = property_id(state.field) else {
        // 起こらないはず(`moved` は property_id が通った move でしか立たない)
        // だが、安全側で overlay だけは残さず抜ける実害は無い(次の press で
        // 上書きされる)。
        return Ok(None);
    };
    let mut write_error = None;
    if let Some(value) = state.last_value {
        // 確定の track も値セル Enter と同じ意味([`edited_value_track`] —
        // キー無しは静的書き換え・キー持ちは press 時点の playhead へ upsert)。
        // transient overlay は `track()` には映らないので、ここで読むのは
        // drag 開始前の本 track そのもの。
        let base_track = doc.view().track(state.layer, &property).ok().flatten();
        match edited_value_track(base_track.as_ref(), state.playhead_frame, state.fps, value) {
            Ok(track) => {
                if let Err(error) = doc.apply(Intent::SetTrack {
                    layer: state.layer,
                    property: property.clone(),
                    track,
                }) {
                    write_error = Some(format!("値を書けない: {error}"));
                }
            }
            Err(error) => write_error = Some(error),
        }
    }
    doc.clear_transient(state.layer, &property);
    match write_error {
        Some(error) => Err(error),
        None => Ok(None),
    }
}

/// Esc: 進行中の drag があれば `clear_transient` で復元し(overlay は edit
/// timeline に一切触れていないので undo/redo 履歴は最初から無傷)、無ければ
/// typing 下書き(値セル)を破棄する。
///
/// **`true`** を返したら呼び出し側はここで終わり(元の `motolii_shell::Shell::
/// cancel_inspector_interaction` の早期 return を保つ — 名前欄・Settings 下書き
/// へは踏み込まない、それらは Inspector pane の write-set 外)。
pub fn cancel_field_interaction(
    doc: &mut Document,
    drag: &mut Option<FieldDragState>,
    field_draft: &mut Option<FieldDraft>,
) -> bool {
    if let Some(state) = drag.take() {
        if state.moved {
            if let Ok(property) = property_id(state.field) {
                doc.clear_transient(state.layer, &property);
            }
        }
        return true;
    }
    field_draft.take().is_some()
}

// ---------------------------------------------------------------------------
// view — StoreView の投影(SelectionProjection)と下書きだけを受け取る。書けない。
// ---------------------------------------------------------------------------

use iced::widget::{
    button, column, container, mouse_area, row as row_widget, scrollable, text, text_input, Space,
};
use iced::{Element, Length};

/// `.cols`/`.prow` 系の行スタイル。線化 D5(裁定179 文法1、
/// `docs/reviews/2026-08-22-chrome-grammar-audit.md`)で罫線は透明化した —
/// 参照3製品(ableton/figma/AE)はプロパティ行を線で区切らず、区切りは
/// **固定行高+間隔**が担う(旧: 裁定137/139 の hairline。mock の
/// `.cols{border-bottom}`/`.prow{border-bottom}` は裁定179 が上書きする —
/// `chip_outline_fence` の「沈黙部分の上書き」と同じ整理)。透明 border で
/// 幅だけ残す=幾何不変。`pub`: `tests/row_line_fence.rs` が機械照合する。
pub fn row_band_style(dims: Dimensions) -> container::Style {
    container::Style {
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// `.cols`/`.prow` 系の行の共通ラッパー。padding・固定高・pane 全幅は
/// **ここ(外側 container)だけ**が持つ — `content` 自身は spacing/align_y
/// だけを持ち、自分の width/height を宣言しない(`ident_band` と同じ構造。
/// Fill な子孫[label 等]を持つ Shrink な row は祖先の container が与える
/// Limits の上限までしか伸びないので、外側 container の bounds とは一致
/// しない — 496幅ちょうど/20px高ちょうどの `Container` candidate が二重に
/// 現れて `tests/inspector_pixel_fence.rs` の数え上げを壊す事故を避けられる、
/// 実測)。スタイルは [`row_band_style`](線化 D5 — 罫線なし・幾何不変)。
fn bordered_row(content: Element<'static, Message>, dims: Dimensions) -> Element<'static, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fixed(dims.inspector_row_height))
        .padding([0.0, dims.spacing_m])
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme| row_band_style(dims))
        .into()
}

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
        text("選択なし — layer を選ぶと Transform / Attrs が並ぶ")
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
        rows = rows.push(text_section(text_projection, text_field_draft, dims, colors));
    }
    rows = rows.push(hint_row(dims, colors));

    scrollable(rows).height(Length::Fill).into()
}

/// MASK section: mask 1枚 = ident 行(id + mode 巡回 + Inverted トグル)+
/// opacity 値行([`transform_row`] そのまま — 値セル/Key 列の文法を再利用)。
/// section header・行高・余白はすべて既存トークン([`section_header`]/
/// [`bordered_row`])— 新しい寸法・色ロールを発明しない(裁定179/S4)。
fn mask_section(
    masks: &[MaskRowProjection],
    field_draft: Option<&FieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut section = column![section_header("MASK", dims, colors)];
    for mask_row in masks {
        section = section.push(mask_ident_row(mask_row, dims, colors));
        section = section.push(transform_row(&mask_row.opacity, field_draft, dims, colors));
    }
    section.into()
}

/// mask 1枚の ident 行: 「Mask {id}」ラベル + mode 巡回ボタン
/// ([`flat_button_style`]、Blend 行と同じ文法)+ Inverted トグル
/// ([`glyph_button_style`]、M glyph と同じ「状態の器」文法 — inverted=on の
/// 時だけ accent 縁)。
fn mask_ident_row(
    mask_row: &MaskRowProjection,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let id = mask_row.id;
    let inverted = mask_row.inverted;

    let content = row_widget![
        text(format!("Mask {id}"))
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        // mode 巡回(`CycleBlendMode` と同じ即時操作・同じ意匠)。表示は
        // `MaskMode` の `Debug`(`Add`/`Subtract`/… — blend の表示と同じ流儀)。
        button(text(format!("{:?}", mask_row.mode)).size(dims.body_text))
            .on_press(Message::CycleMaskMode(id))
            .style(move |_theme, status| flat_button_style(colors, status)),
        // inverted トグル(M glyph と同じ「チップ輪郭=状態の器」文法 —
        // 裁定179。glyph 幅1文字では意図が読めないので語で出す: 意図優先・裁定174)。
        button(
            text("Inverted")
                .size(dims.caption_text)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .height(Length::Fixed(glyph_height(dims)))
        .padding([0.0, dims.spacing_s])
        .on_press(Message::ToggleMaskInverted(id))
        .style(move |_theme, status| glyph_button_style(dims, colors, status, inverted)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// EFFECTS section: effect 1本 = ident 行(名前 + ↑↓ reorder + Bypass トグル +
/// Remove)+ param 値行([`transform_row`] そのまま — 値セル/Key 列の文法を
/// 再利用)。[`mask_section`] と同じ構成 — section header・行高・余白はすべて
/// 既存トークン、新しい寸法・色ロールを発明しない(裁定179/S4)。
fn effects_section(
    effects: &[EffectRowProjection],
    field_draft: Option<&FieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut section = column![section_header("EFFECTS", dims, colors)];
    for effect_row in effects {
        section = section.push(effect_ident_row(effect_row, dims, colors));
        for param_row in &effect_row.params {
            section = section.push(transform_row(param_row, field_draft, dims, colors));
        }
    }
    section.into()
}

/// effect 1本の ident 行: 名前ラベル(bypass 中は ink2 — 「効いていない」の
/// 視覚合図、hidden layer の扱いと同型)+ ↑/↓(reorder、[`flat_button_style`])+
/// Bypass トグル(mask の Inverted と同じ「チップ輪郭=状態の器」文法 —
/// bypass=on の時だけ accent 縁)+ Remove([`flat_button_style`])。
/// glyph 1文字では意図が読めない語(Bypass/Remove)は語で出す(意図優先・
/// 裁定174、mask Inverted と同じ判断)。↑↓ は「上へ/下へ」の意図がそのまま
/// 読める最小の記号なので語にしない。
fn effect_ident_row(
    effect_row: &EffectRowProjection,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let id = effect_row.id;
    let bypassed = !effect_row.enabled;
    let name_color = if bypassed {
        Ink::Secondary.resolve(&colors)
    } else {
        colors.text_primary
    };

    let caption_button = |label: &'static str, message: Message| {
        button(
            text(label)
                .size(dims.caption_text)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .height(Length::Fixed(glyph_height(dims)))
        .padding([0.0, dims.spacing_s])
        .on_press(message)
        .style(move |_theme, status| flat_button_style(colors, status))
    };

    let content = row_widget![
        text(effect_row.name.clone())
            .size(dims.body_text)
            .color(name_color)
            .width(Length::Fill),
        caption_button("↑", Message::MoveEffectUp(id)),
        caption_button("↓", Message::MoveEffectDown(id)),
        // bypass トグル(mask Inverted と同じ「状態の器」文法 — on の時だけ
        // accent 縁。押しても消えない = 「消さずに切る」を器で語る)。
        button(
            text("Bypass")
                .size(dims.caption_text)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .height(Length::Fixed(glyph_height(dims)))
        .padding([0.0, dims.spacing_s])
        .on_press(Message::ToggleEffectBypass(id))
        .style(move |_theme, status| glyph_button_style(dims, colors, status, bypassed)),
        caption_button("Remove", Message::RemoveEffect(id)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}


/// TEXT section: テキストレイヤー選択時のみ現れる(裁定184 型別 section 第3号)。
/// **Key 列は無い** — `TextDocumentStyle`/`TextDocument::justify` はどれも
/// `KeyframeTrack` に乗らない静止フィールド(裁定92)なので、Position/Scale
/// 行の3状態 oracle は適用対象外。Font/Size/Line Height/Tracking は
/// [`speed_row`] と同じ「即時 text_input・on_submit で1回の Intent」文法、
/// Justify は [`mask_ident_row`] の mode 巡回と同じ即時操作文法 — どちらも
/// **既存の grammar の適用**であって新しい視覚言語の発明ではない(NON-GOALS)。
/// 塗り色(`fc`)・線色(`sc`)は実在するが `Value::Color` 用の editor が
/// まだ無い(crate doc「Color/Enum/Path/LayerId は Effect 束の仕事」)ため
/// この切片では見送る(RETURN の見送り台帳参照)。
fn text_section(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    column![
        section_header("TEXT", dims, colors),
        text_field_row(
            "Font",
            TextField::FontFamily,
            text_projection.font_family.clone(),
            draft,
            dims,
            colors,
        ),
        text_field_row(
            "Size",
            TextField::Size,
            format_number(text_projection.size as f64, 1),
            draft,
            dims,
            colors,
        ),
        line_height_row(text_projection, draft, dims, colors),
        tracking_row(text_projection, draft, dims, colors),
        justify_row(text_projection.justify, dims, colors),
    ]
    .into()
}

/// TEXT section の text_input 行の共通形(`speed_row` の value_field と同じ
/// 組み方)。下書きがあればそれを、無ければ投影の確定値を表示する。
fn text_field_row(
    label: &'static str,
    field: TextField,
    current: String,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|d| d.field == field)
        .map(|d| d.text.clone())
        .unwrap_or(current);

    // 裁定170 M01: fork の text_input は借用寿命を返り値に縛るため owned move
    // (`speed_row`/`ident_band` と同じ回避)。
    let value_field = text_input("", displayed)
        .on_input(move |text| Message::TextFieldInput(field, text))
        .on_submit(Message::TextFieldSubmit(field))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text(label)
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// Line Height 行。`None`(Auto)は「Auto」文字列で表示し、`Auto` ボタンで
/// 明示的に戻せる(`speed_row` の Reset ボタンと同じ即時操作文法。map
/// 「Auto leading for selected text」、採用済)。
fn line_height_row(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|d| d.field == TextField::LineHeight)
        .map(|d| d.text.clone())
        .unwrap_or_else(|| {
            text_projection
                .line_height
                .map(|value| format_number(value as f64, 1))
                .unwrap_or_else(|| "Auto".to_owned())
        });

    let value_field = text_input("", displayed)
        .on_input(|text| Message::TextFieldInput(TextField::LineHeight, text))
        .on_submit(Message::TextFieldSubmit(TextField::LineHeight))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text("Line Height")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
        button(text("Auto").size(dims.caption_text))
            .on_press(Message::ResetLineHeightAuto)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// Tracking 行。`Reset` ボタンは 0 でも常に出す(`speed_row` の Reset と同じ
/// 「無反応ゼロより一貫を優先」判断。map「Reset tracking to 0」、採用予定)。
fn tracking_row(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|d| d.field == TextField::Tracking)
        .map(|d| d.text.clone())
        .unwrap_or_else(|| format_number(text_projection.tracking as f64, 1));

    let value_field = text_input("", displayed)
        .on_input(|text| Message::TextFieldInput(TextField::Tracking, text))
        .on_submit(Message::TextFieldSubmit(TextField::Tracking))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text("Tracking")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
        button(text("Reset").size(dims.caption_text))
            .on_press(Message::ResetTracking)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// Justify(揃え)行。`mask_ident_row` の mode 巡回ボタンと同じ即時操作文法 —
/// 表示は `TextJustify` の `Debug`(`Left`/`Right`/`Center` — blend/mask の
/// 表示と同じ流儀)。
fn justify_row(justify: TextJustify, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let content = row_widget![
        text("Justify")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        button(text(format!("{justify:?}")).size(dims.body_text))
            .on_press(Message::CycleTextJustify)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
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
    let subtitle = text(selection.kind)
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

/// ラベル色チップ(B03、ident 帯)。正方形の色見本 — 塗りは timeline の行
/// スウォッチ(`motolii-timeline-pane::lane_bar::swatch_color`)と同じ源・
/// 同じ既定: `label_color` index → `colors.label_palette`、未割当は
/// `way_timeline` へフォールバック(同じ意味役割の色を2箇所で別の式にしない)。
/// click で palette を巡回([`Message::CycleLabelColor`] → [`next_label_color`]
/// — 巡回ボタン文法、BL2 と同じ理由で pick_list は導入しない)。
fn label_color_chip(
    label_color: Option<u8>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let chip_color = label_color
        .and_then(|index| colors.label_palette.get(index as usize))
        .copied()
        .unwrap_or(colors.way_timeline);
    let side = label_chip_side(dims.inspector_row_height);
    button(text(""))
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .padding(0.0)
        .on_press(Message::CycleLabelColor)
        .style(move |_theme, status| label_chip_style(dims, colors, chip_color, status))
        .into()
}

/// チップの1辺。timeline rail の正方形チップ
/// (`motolii-timeline-pane::lane_bar::glyph_size_px` = `round(0.462 × 行高)`、
/// 裁定172 §2 (2))と同じ式 — 別 crate なので共有関数は置けない(式だけ揃える、
/// [`sibling_gap_px`] と同じ判断)。**`inspector_glyph_width`(26px)は使わない**
/// — その寸法は shell 側 `inspector_pixel_fence` が「M 1個 + Key 5個 = 6個」を
/// 数え上げる柵の対象なので、同寸の箱を足すと柵が壊れる(幾何を壊さない、
/// 発注書の柵)。
fn label_chip_side(row_height: f32) -> f32 {
    (row_height * 0.462).round().max(1.0)
}

/// チップの style。面 = ラベル色そのもの(色見本 — 色が内容なので平常から
/// 塗る。裁定179「箱は状態の器」の例外ではなく、これは箱ではなく swatch)。
/// hover で `border_default` の縁(値セル hover と同じ「触れる」合図の文法 —
/// 新しい意匠を発明しない)。
fn label_chip_style(
    dims: Dimensions,
    colors: Colors,
    chip_color: iced::Color,
    status: button::Status,
) -> button::Style {
    let border_color = match status {
        button::Status::Hovered | button::Status::Pressed => colors.border_default,
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(chip_color)),
        text_color: colors.text_primary,
        border: iced::Border {
            color: border_color,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}

/// mock の `.cols` 行: 「Property X Y Z Key」を1度だけ出す。各 `.prow` 側は
/// もう軸ラベルを繰り返さない(旧実装は cell ごとに X/Y/Z を再掲していた —
/// mock にその繰り返しは無い)。
fn column_header_row(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let value_width = Length::Fixed(dims.inspector_value_width);
    let axis = |label: &'static str| {
        text(label)
            .size(dims.caption_text)
            .color(colors.text_muted)
            .width(value_width)
            .align_x(iced::alignment::Horizontal::Center)
    };

    let content = row_widget![
        text("Property")
            .size(dims.caption_text)
            .color(colors.text_muted)
            .width(Length::Fill),
        row_widget![axis("X"), axis("Y"), axis("Z")].spacing(dims.spacing_xs),
        text("Key")
            .size(dims.caption_text)
            .color(colors.action_active)
            .width(Length::Fixed(dims.inspector_glyph_width))
            .align_x(iced::alignment::Horizontal::Center),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    // mock `.cols{border-bottom:var(--line) solid #1a1a1a}` は線化 D5
    // (裁定179 文法1)が上書き — 罫線なし([`row_band_style`] doc 参照)。
    bordered_row(content.into(), dims)
}

// `section_header` は裁定160 切片5(pane split survey §2.4/§6)で
// `chrome::section_header` へ移設した(純粋な再配置・挙動ゼロ変更)。

// ---------------------------------------------------------------------------
// 裁定183 taffy 転写 本丸: `transform_row` = mock `.propertyRow`/`.columnHeader`
// の grid 宣言(`next/reference/mocks/inspector-library.html` v3.1)。
//
// CSS 宣言は mock の字面をそのまま(`display:grid; grid-template-columns:
// minmax(132px,1fr) repeat(3, 64px) 26px;`)——settings-pane の第1号
// (`motolii-settings-pane::sections::comp_cells_row_css` — 裁定183)と同じ
// 「CSS 文字列が単一正本」の形。値だけ `Dimensions`(トークン)から埋める。
//
// **`transform_row` の本体からは今回まだ呼ばない — 部分適用(発注書
// 「行き詰まったら部分適用でよい」)。FINDING(150%実測、RETURN 本文参照)**:
// `TaffyBox`(`motolii-taffy`、書き換え禁止)は内部で毎回
// `taffy::TaffyTree::new()` を作り、rounding を無効化せずに
// `compute_layout_with_measure` を呼ぶ — taffy 0.13 の既定は「解いた矩形を
// 最近偶数丸め(banker's rounding)で整数 px へ丸める」。`inspector_row_height`
// (=25、奇数)は 150%(`Dimensions::scaled(1.5)`)で 37.5 という**構造的に
// 半端な**値になり、そこから引く [`value_cell_height`]/[`glyph_height`]
// (31.5/34.5)も同様に半端になる — これらが `TaffyBox` を1回でも通ると
// (root の `width`/`height` を明示 px にしても、子の高さを明示にしても)
// 32.0/34.0/38.0 のような整数へ丸められてしまうことを raw `taffy::TaffyTree`
// 直叩きの probe で実測確認した(`property_row_css` の root に
// `height:37.5px` を明示しても出力は 38.0 — 丸めは「auto/明示の別」ではなく
// 木全体への無条件後処理)。既存の柵(shell 側、書き換え禁止)
// `inspector_pixel_fence.rs::the_grid_shape_is_preserved_at_150_percent_scale`
// は `EPS_EXACT=0.05` という極めて厳しい許容で `dims.inspector_value_width ×
// (dims.inspector_row_height - dims.spacing_s)`(= 96×31.5 ちょうど)を要求する
// ため、`TaffyBox` 経由の値セルはこの柵を必ず落とす — `motolii-taffy` 側に
// rounding を無効化する口が無い(`TaffyBox::new` は `taffy::Style` 1個しか
// 受けない)以上、**この crate 単独では解けない**。
//
// 解除には次のいずれかが要る(この発注の write-set 外 — 供覧のみ):
// 1. `motolii-taffy::TaffyBox` に rounding 無効化(または `taffy::Style::
//    Composite`? いや `TaffyTree::disable_rounding()`)を露出する API 追加。
// 2. `inspector_pixel_fence.rs` の `EPS_EXACT` を taffy 経由の行にだけ緩める
//    (該当ファイルは書き換え禁止・shell 側の判断)。
//
// **width/height/gap の値そのものは決めてある**(下記関数)——上の理由で
// production 配線だけ見送った。`property_row_css` は
// `tests/property_row_taffy_oracle.rs`(`Dimensions::default()`、整数 px の
// みなので上記の丸め問題が顕在化しない)がモック実測と ±1px で照合済み。
//
// **gap は mock 自体には宣言が無い**(`.columnHeader, .propertyRow` は
// `grid-template-columns` のみ)——裁定168 施工の兄弟間隔を1本の grid gap へ
// 統合する設計(`Dimensions::default()`/150%どちらでも `sibling_gap_px` と
// `spacing_xs` は同値 — `sibling_gap_px_matches_the_ladder_bottom_rung_
// rounded_to_the_nearest_pixel`/`ui_scale_fence.rs`/`inspector_pixel_fence.rs`
// が実際に踏む2値、settings-pane の `comp_cells_row_css` と同じフラット化)。
pub fn property_row_css(dims: Dimensions) -> String {
    // `bordered_row` の `.padding([0.0, dims.spacing_m])`(左右)を差し引いた
    // あとの中身幅。
    let content_width = dims.inspector_panel_width - 2.0 * dims.spacing_m;
    format!(
        "display:grid; width:{content_width}px; height:{height}px; grid-template-columns:minmax(132px,1fr) repeat(3,{value}px) {glyph}px; align-items:center; gap:0 {gap}px;",
        height = dims.inspector_row_height,
        value = dims.inspector_value_width,
        glyph = dims.inspector_glyph_width,
        gap = dims.spacing_xs,
    )
}

/// 発注書の固定列グリッド `Property | X | Y | Z | Key` = `1fr + 3×value幅 + hit`。
/// scalar 行(Opacity)は mock どおり3列目(Z の位置)へ値を置き、残り2列は
/// 空箱で埋める([`blank_value_cell`] — `absent_component` の「このモデルに
/// 無い軸」とは別の意味で、単に scalar 行が3値グリッドに収まるための穴埋め)。
///
/// **裁定183(taffy 転写)は今回ここへ配線していない**([`property_row_css`]
/// の doc「FINDING」参照 — 150%実測で `motolii-taffy` の既定 rounding が
/// 既存の shell 側柵を壊すことを発見したため、部分適用として CSS 宣言+oracle
/// だけを確立した)。並べ方は引き続き旧来どおり手組みの `row_widget!`。
fn transform_row(
    row_projection: &TransformRowProjection,
    field_draft: Option<&FieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let label = text(row_projection.label)
        .size(dims.body_text)
        .color(colors.text_primary)
        .width(Length::Fill);

    let value_cells: Vec<Element<'static, Message>> = match &row_projection.value {
        RowValue::Vector(components) => components
            .iter()
            .map(|slot| value_cell(slot, field_draft, row_projection.decimals, dims, colors))
            .collect(),
        RowValue::Scalar(slot) => vec![
            blank_value_cell(dims, colors),
            blank_value_cell(dims, colors),
            value_cell(slot, field_draft, row_projection.decimals, dims, colors),
        ],
    };

    let content = row_widget![
        label,
        // 裁定168 施工: 値セル同士の gap は裁定167 下段(0.075×行高)へ —
        // 違反(B)「960.000540.000」と読める密着の緩衝(値セル自体の幅
        // 38px は変えない、[`value_cell`]/`inspector_pixel_fence.rs` 参照)。
        row_widget(value_cells).spacing(sibling_gap_px(dims.inspector_row_height)),
        key_glyph(row_projection.key, dims, colors), // Key 列(K1 — 結線済み)。
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    // mock `.prow{border-bottom:var(--line) solid rgba(0,0,0,.35)}` は線化 D5
    // (裁定179 文法1)が上書き — 罫線なし([`row_band_style`] doc 参照)。
    bordered_row(content.into(), dims)
}

/// 発注書「読み取り専用値は編集セルと同一形状で色だけ落とす」を1箇所で守る —
/// absent(muted)・editable(text_input)・animated(accent, 表示のみ)のどれでも
/// 同じ形(同じ幅高さ)を作る。線化 D2(裁定179「箱は状態の器」)以降、
/// 平常はどれも素の表示(面・輪郭なし)— 箱が現れるのは editable セルの
/// hover([`value_box_style`])と編集中(`value_input_style`)だけ。
fn value_cell(
    slot: &ComponentSlot,
    field_draft: Option<&FieldDraft>,
    decimals: usize,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    if !slot.present {
        // mock の absent 表現と同じ意味 — このモデルに無い軸だと明示する
        // (空欄ではなく「—」)。読み取り専用値と同じ箱形。
        return boxed_value("—".to_owned(), colors.text_muted, dims, colors);
    }

    match (slot.editable, slot.field) {
        (true, Some(field)) => {
            let editing = field_draft.is_some_and(|draft| draft.field == field);
            if editing {
                let displayed = field_draft
                    .filter(|draft| draft.field == field)
                    .map(|draft| draft.text.clone())
                    .unwrap_or_else(|| format_number(slot.value, decimals));
                container(
                    // 裁定170 M01: fork の text_input が借用寿命を返り値に縛るため
                    // owned move(値不変)。
                    text_input("", displayed)
                        .id(field_input_id(field))
                        .on_input(move |text| Message::FieldInput(field, text))
                        .on_submit(Message::FieldSubmit(field))
                        .size(dims.body_text)
                        .width(Length::Fill)
                        // 縦0を維持(柵で発見した実修正 — `text_input` の既定 padding
                        // `iced_widget::text_input::DEFAULT_PADDING` = 5px 全辺が固定高
                        // `value_cell_height`(row-4 = 16px)を食い潰し、文字の描画領域が
                        // 16 - 2*5 = 6px まで押し潰される、実測: 修正前は text_input 内の
                        // paragraph 高が 6px)。**横だけ** [`value_cell_padding`] で戻す
                        // (裁定139: セル幅38pxいっぱいに文字が縁へ接触しないよう内余白を
                        // 確保 — 裁定168 施工でその横内余白の式を `spacing_xs` 固定から
                        // `0.6em`(`single_row_horizontal_inset`)へ差し替えた)。
                        .padding(value_cell_padding(dims))
                        .align_x(iced::alignment::Horizontal::Center)
                        .style(move |_theme, status| value_input_style(dims, colors, status)),
                )
                .width(Length::Fixed(dims.inspector_value_width))
                .height(Length::Fixed(value_cell_height(dims)))
                .align_y(iced::alignment::Vertical::Center)
                // 裁定168 施工(違反(B)の根治): セル幅38pxは変えない(グリッドの
                // 形は不変)ので、桁数の多い値は依然としてこの箱より広く
                // シェイプされ得る(実測: 例 "960.000" は自然幅38.83px。
                // `text_input` は自前でスクロールするため通常はこの経路で
                // はみ出さないが、padding が増えて内側が狭まる分の防波堤として
                // 揃えて `clip(true)` を掛ける — 隣セルへの paint 越境を構造的に
                // 断つ(padding/gap だけでは箱幅そのものは広がらないので、
                // これが実際の「文字が隣へ滲む」ことへの根治点)。
                .clip(true)
                .into()
            } else {
                // click せず(まだ)編集していない見た目 — drag-to-scrub の起点
                // ([`draggable_value_cell`])。表示する値は投影(`slot.value`)
                // そのものなので、drag 中の transient 値もここが自動で映す。
                // キー持ち行は accent — 「編集すると(playhead へ)記録される」
                // ことの視覚合図(AE 作法、2026-08-22 発注。旧実装の animated
                // 表示専用セルと同じ色を、編集可能なまま引き継ぐ)。
                let value_color = if slot.keyed {
                    colors.action_active
                } else {
                    colors.text_primary
                };
                draggable_value_cell(
                    field,
                    display_number(slot.value, decimals),
                    value_color,
                    dims,
                    colors,
                )
            }
        }
        // present なのに field が無い(起こらないはず)— 安全側の表示のみ fallback。
        _ => boxed_value(
            display_number(slot.value, decimals),
            colors.action_active,
            dims,
            colors,
        ),
    }
}

/// present・editable(un-keyed)な field の**まだ編集していない**見た目。
/// `mouse_area` は press だけを own する — move/release は window 全体を追う
/// `Shell::subscription` 側の担当(`motolii_shell::inspector_pointer_event`)。iced 0.14
/// の `mouse_area` は自分の bounds を出た cursor を追えない(pointer capture が
/// 無い実測)ので、値セル自身の当たり判定は「drag を armed にする press」だけに
/// 絞ってある — 感度どおりに動かすとすぐこの38px幅を出るため。
///
/// 線化 D2(裁定179): 箱は [`HoverValueBox`] — 平常は素の数字、hover で箱
/// ([`value_box_style`])。幾何・clip・event 経路は旧 `container` と同一。
fn draggable_value_cell(
    field: TransformField,
    displayed: String,
    value_color: iced::Color,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    mouse_area(hover_value_box(
        text(displayed)
            .size(dims.body_text)
            .color(value_color)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into(),
        dims,
        colors,
    ))
    .interaction(iced::mouse::Interaction::ResizingHorizontally)
    .on_press(Message::ValuePressed(field))
    .into()
}

// ---------------------------------------------------------------------------
// 線化 D2: hover を知る値セルの箱(`container` 同型の最小 widget)
// ---------------------------------------------------------------------------

use iced::advanced::layout::{self as adv_layout, Layout};
use iced::advanced::renderer as adv_renderer;
use iced::advanced::widget::{self as adv_widget, Widget};

/// 値セル(表示状態)の箱。`container` と同型の最小 widget だが、style を
/// **draw 時の cursor 実測**で選ぶ([`ValueBoxStatus`] → [`value_box_style`])。
/// iced 0.15 の `container` は style closure に status を渡さない
/// (`Fn(&Theme) -> Style`)ため、「hover でだけ箱が現れる」(裁定179)を
/// container のままでは書けない — hover 状態を Shell に持たせて view へ配る
/// 迂回より、自分の境界に素直な口を1本作る(wrapper>ハック、2026-08-18裁定。
/// `button` が draw 時に `is_mouse_over` で `Status::Hovered` を決めるのと
/// 同じ判定・同じ時点であり、状態は transient すら持たない)。
///
/// **幾何は旧 container と同一**(発注書「幾何を変えない」の施工点):
/// - `operate` は `container::operate` と同じく自分の bounds を
///   `operation.container(None, ..)` で1回だけ登録してから内容へ traverse —
///   shell 側 `inspector_pixel_fence` の Container 数え上げ(値セル
///   64×(row-4) が15個)はこの widget でも同じ1個として見える。
/// - layout は公開ヘルパ `container::layout` を固定幅高・padding 0・中央寄せで
///   そのまま呼ぶ(旧 `.width(Fixed)/.height(Fixed)/.align_*(Center)` と同値)。
/// - draw は `container::draw_background` + clipped viewport(旧 `.clip(true)`
///   と同じ越境遮断 — 裁定168 の根治点を維持)。
/// - update/mouse_interaction/overlay は内容へ素通し(event 経路に触れない —
///   press を own するのは外側の `mouse_area` のまま)。
struct HoverValueBox {
    content: Element<'static, Message>,
    dims: Dimensions,
    colors: Colors,
}

fn hover_value_box(
    content: Element<'static, Message>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    Element::new(HoverValueBox {
        content,
        dims,
        colors,
    })
}

impl Widget<Message, iced::Theme, iced::Renderer> for HoverValueBox {
    fn tag(&self) -> adv_widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> adv_widget::tree::State {
        self.content.as_widget().state()
    }

    fn diff(&mut self, tree: &mut adv_widget::Tree) {
        self.content.as_widget_mut().diff(tree);
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size {
            width: Length::Fixed(self.dims.inspector_value_width),
            height: Length::Fixed(value_cell_height(self.dims)),
        }
    }

    fn layout(
        &mut self,
        tree: &mut adv_widget::Tree,
        renderer: &iced::Renderer,
        limits: &adv_layout::Limits,
    ) -> adv_layout::Node {
        container::layout(
            limits,
            Length::Fixed(self.dims.inspector_value_width),
            Length::Fixed(value_cell_height(self.dims)),
            iced::Padding::ZERO,
            iced::alignment::Horizontal::Center,
            iced::alignment::Vertical::Center,
            |limits| self.content.as_widget_mut().layout(tree, renderer, limits),
        )
    }

    fn operate(
        &mut self,
        tree: &mut adv_widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn adv_widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                tree,
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut adv_widget::Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &adv_widget::Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            tree,
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &adv_widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        renderer_style: &adv_renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let status = if cursor.is_over(bounds) {
            ValueBoxStatus::Hovered
        } else {
            ValueBoxStatus::Idle
        };
        let style = value_box_style(self.dims, self.colors, status);

        // 旧実装の `.clip(true)` と同値: 内容の paint は常に箱の bounds へ
        // 切り詰める(裁定168 — 隣接セルへの文字の越境を構造的に断つ)。
        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            container::draw_background(renderer, &style, bounds);
            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                &adv_renderer::Style {
                    text_color: style.text_color.unwrap_or(renderer_style.text_color),
                },
                layout.children().next().unwrap(),
                cursor,
                &clipped_viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut adv_widget::Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

/// mock `.prow .v { height: calc(var(--row) - 4*var(--s)*1px) }` の `4` は
/// `spacing_s`(既定4)と同じ値 — スケール済みの `spacing_s` を使うことで
/// `ui_scale` を再度掛け直さずに済む(適用点は `Dimensions::scaled` の1箇所だけ)。
fn value_cell_height(dims: Dimensions) -> f32 {
    (dims.inspector_row_height - dims.spacing_s).max(1.0)
}

/// 単行の横余白(裁定168): `0.6em`(`em` = その文字の size)の px 最近傍丸め。
/// 値セル/名前欄はどちらも `dims.body_text` サイズの文字を持つので、`em` は
/// `dims.body_text` を使う。
fn single_row_horizontal_inset(text_size: f32) -> f32 {
    (text_size * 0.6).round()
}

/// 値セル(`.prow .v`)の text_input 横内余白(裁定139・裁定168)。**縦は0の
/// まま** — 行高合わせの実測修正([`value_cell_height`] の doc 参照)。旧実装は
/// grid gap の最小段トークン `spacing_xs`(mock `--sp1`=2px)を転用していたが、
/// 裁定168(「文字の余白」)は単行の横余白を `0.6em` と定めたので、そちらへ
/// 差し替える(セル幅自体は変えない、38px のまま — 内側の呼吸だけが広がる)。
fn value_cell_padding(dims: Dimensions) -> iced::Padding {
    iced::Padding::from([0.0, single_row_horizontal_inset(dims.body_text)])
}

/// ident 帯の名前欄(`.ident b`)の横内余白。[`value_cell_padding`] と同じ
/// 理由・同じ式を使う(裁定139 は `value_cell`/`name_field` を並記している —
/// 2箇所で別の値を発明しない、裁定168 適用後もこの対称は保つ)。
fn name_field_padding(dims: Dimensions) -> iced::Padding {
    iced::Padding::from([0.0, single_row_horizontal_inset(dims.body_text)])
}

/// 兄弟要素間の gap(裁定167 の梯子下段: `0.075 × 行高`、px 最近傍丸め)。
/// `motolii-timeline-pane::lane_bar::sibling_gap_px` と同型 — 別 crate なので
/// 共有関数は置けない(式だけ揃える、値は pane ごとに token 経由で持つ)。
///
fn sibling_gap_px(row_height: f32) -> f32 {
    (row_height * 0.075).round()
}

fn boxed_value(
    content: String,
    color: iced::Color,
    dims: Dimensions,
    _colors: Colors,
) -> Element<'static, Message> {
    container(
        text(content)
            .size(dims.body_text)
            .color(color)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(dims.inspector_value_width))
    .height(Length::Fixed(value_cell_height(dims)))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    // 線化 D2(裁定179): 面も輪郭も塗らない — 素の表示だけ。非対話セル
    // (absent「—」・fallback)なので hover 箱も出さない(触れない物に
    // 触れそうな箱を出さない — Q0 と同じ判断)。幾何(固定幅高)は不変。
    .style(move |_theme| container::Style::default())
    // 裁定168 施工(違反(B)の根治): `draggable_value_cell` と同じ理由 —
    // absent(「—」)/animated 表示も同じ箱形を共有するので、越境の柵も揃える。
    .clip(true)
    .into()
}

/// scalar 行(Opacity)の空き枠(X/Y列)。中身の無い箱 — grid の穴埋めであって
/// 「このモデルに無い軸」ではない(`value_cell` の absent 表現とは別の意味)。
/// 線化 D2(裁定179)で面(`surface_app`)を落とした — 穴は何も描かない。
/// 幾何(固定幅高の Container)は不変(`inspector_pixel_fence` の数え上げに
/// そのまま載る)。
fn blank_value_cell(dims: Dimensions, _colors: Colors) -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fixed(dims.inspector_value_width))
        .height(Length::Fixed(value_cell_height(dims)))
        .style(move |_theme| container::Style::default())
        .into()
}

/// Key/M/S glyph 列の高さ。mock `.glyph { height: calc(var(--row) - 2*var(--s)*1px) }`
/// の `2` は `spacing_xs`(既定2)と同じ値。
fn glyph_height(dims: Dimensions) -> f32 {
    (dims.inspector_row_height - dims.spacing_xs).max(1.0)
}

/// **M glyph — 結線済み**(supervisor 訂正、2026-08-20)。`LayerAttrs.hidden` を
/// トグルする。on(hidden=true)は mock `.glyph.on` と同じ accent 縁取り+文字色。
/// `.font(TextWeight::Bold)` で mock `.glyph{font-weight:800}` を写す(裁定137)。
fn mute_glyph(dims: Dimensions, colors: Colors, hidden: bool) -> Element<'static, Message> {
    button(
        text("M")
            .size(dims.caption_text)
            .font(TextWeight::Bold.font())
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(dims.inspector_glyph_width))
    .height(Length::Fixed(glyph_height(dims)))
    .padding(0.0)
    .on_press(Message::ToggleHidden)
    .style(move |_theme, status| glyph_button_style(dims, colors, status, hidden))
    .into()
}

/// **Key glyph — 結線済み(K1)**。視覚は mock(`next/reference/mocks/
/// inspector-library.html` v3.1 `.keyButton`)の転写:
/// - Static: ◇(text_muted)・面なし(`background: transparent`)、hover で
///   accent 12% の面(mock `.keyButton:hover`)
/// - Between: ◆(accent)・accent 12% の面(mock `.keyButton.animated` —
///   CSS の後勝ちで hover でも面は据え置き)
/// - AtKey: ◆(accent)・accent 20% の面(mock `.keyButton.current`)
///
/// 枠の文法 = 裁定179: **常時輪郭なし・hover で面・filled は accent**(mock も
/// `border: 0`)。色ロールは `action_active`/`text_muted` の既存2つだけ
/// (S4 — 新ロール禁止。「状態の瞬間」の filled=accent は正当)。セル全域が的
/// (S1 — button が `inspector_glyph_width × glyph_height` の箱ごと押せる)。
fn key_glyph(key: KeyCellProjection, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let (glyph, text_color, base_alpha): (&str, iced::Color, f32) = match key.state {
        KeyCellState::Static => ("◇", colors.text_muted, 0.0),
        KeyCellState::Between => ("◆", colors.action_active, 0.12),
        KeyCellState::AtKey => ("◆", colors.action_active, 0.20),
    };
    button(
        text(glyph)
            .size(dims.caption_text)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(dims.inspector_glyph_width))
    .height(Length::Fixed(glyph_height(dims)))
    .padding(0.0)
    .on_press(Message::KeyPressed(key.row))
    .style(move |_theme, status| {
        // mock の CSS 後勝ちどおり: hover の面(12%)は Static でだけ見える
        // (animated/current は自分の面が勝つ)= `max` で写す。
        let alpha = match status {
            button::Status::Hovered | button::Status::Pressed => base_alpha.max(0.12),
            _ => base_alpha,
        };
        // mock の `color-mix(accent 12%, transparent)` = accent のアルファ縮小
        // (tokens の色を作り替えない — 裁定142、raw `Color` 構築はしない)。
        let background = colors.action_active.scale_alpha(alpha);
        button::Style {
            background: Some(iced::Background::Color(background)),
            text_color,
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..button::Style::default()
        }
    })
    .into()
}

/// 列幅の予約だけ(**空のまま** — Q0: 押せそうに見えて押せない chrome を作らない)。
/// S glyph(solo、engine/store 未実装)がこれを使う — 内容も枠も無い、幅だけの
/// `Space`(各行の Key 列は K1 で [`key_glyph`] へ結線済み、もう使わない)。
fn reserved_glyph(dims: Dimensions) -> Element<'static, Message> {
    Space::new()
        .width(Length::Fixed(dims.inspector_glyph_width))
        .height(Length::Fixed(glyph_height(dims)))
        .into()
}

fn glyph_button_style(
    dims: Dimensions,
    colors: Colors,
    status: button::Status,
    active: bool,
) -> button::Style {
    // mock `.glyph{color:var(--ink2)}` — 非 active 状態は ink2(secondary)。
    // 旧実装は ink3(`text_muted`)を誤用していた(2026-08-21 更正)。
    //
    // 線化 D2(裁定179「チップ輪郭=選択時のみ」): 輪郭は active(hidden=on)の
    // 時だけ accent 縁で出す — 状態の器。非 active の平常は素の文字、hover は面。
    let (border, text_color) = if active {
        (
            iced::Border {
                color: colors.action_active,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            colors.action_active,
        )
    } else {
        (iced::Border::default(), Ink::Secondary.resolve(&colors))
    };
    let background = match status {
        button::Status::Hovered => colors.surface_hover,
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color,
        border,
        ..button::Style::default()
    }
}

// ---------------------------------------------------------------------------
// 線化 D2(裁定179「箱は状態の器」): 常時輪郭の廃止 — style 層だけ。
// 正本= docs/reviews/2026-08-22-chrome-grammar-audit.md §文法4。
// ---------------------------------------------------------------------------

/// 値セル(表示状態)の箱の状態。**Shell に hover 状態を持たせない** —
/// [`hover_value_box`] の widget が draw 時の cursor 実測
/// (`Cursor::is_over`)からその場で決める(`button` が `Status::Hovered` を
/// 作るのと同じ時点・同じ判定 — transient すら持たない、純粋な描画時判定)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueBoxStatus {
    /// 平常 — 素の数字(箱なし。AE 型、裁定179「値セル=素の数字+hover 箱」)。
    Idle,
    /// cursor が箱の上 — 箱が現れる(name 欄 hover と同じ
    /// `surface_hover`+`border_default` の文法 — 2箇所で別の意匠を発明しない)。
    Hovered,
}

/// 値セル(表示状態)の箱 style。編集中(typing draft)は従来どおり
/// `chrome::value_input_style`(箱+focus 縁)、drag 中は cursor が
/// セル上に居る間この Hovered 箱がそのまま見える(view は drag 状態を
/// 受け取らない — 逸脱として RETURN に記録)。
fn value_box_style(dims: Dimensions, colors: Colors, status: ValueBoxStatus) -> container::Style {
    match status {
        // 面も輪郭も無し — 素の数字だけ(数字の ink は呼び手が text 側で塗る:
        // text_primary / キー持ち accent は従来のまま)。
        ValueBoxStatus::Idle => container::Style::default(),
        // name 欄 hover(`name_input_style` の Hovered)と同じ面+枠。
        ValueBoxStatus::Hovered => container::Style {
            background: Some(iced::Background::Color(colors.surface_hover)),
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        },
    }
}

/// Inspector 内の button(Blend 巡回・Speed Reset)の style。枠の文法
/// (裁定179): 輪郭なし・素の文字、hover/press で面だけが変わる —
/// `motolii-menubar::leaf_style` と同じ文法(menubar レーンが先に施工した
/// 裁定179 の正本転写。crate 境界を跨ぐので式だけ揃える、menubar と同じ判断)。
fn flat_button_style(colors: Colors, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Some(iced::Background::Color(colors.surface_hover)),
        button::Status::Pressed => Some(iced::Background::Color(colors.state_selected)),
        button::Status::Active | button::Status::Disabled => None,
    };
    let text_color = if status == button::Status::Disabled {
        colors.state_disabled
    } else {
        colors.text_primary
    };
    button::Style {
        background,
        text_color,
        border: iced::Border::default(),
        ..button::Style::default()
    }
}

/// ident 帯の名前欄。未フォーカス時は枠・背景を消して静止 bold テキストに見せ、
/// フォーカス時だけ枠と背景を出す(mock はここを編集可能な `text_input` として
/// 描いていない — 実装が実際に持つ機能=改名を隠さないための最小限の意匠)。
fn name_input_style(
    dims: Dimensions,
    colors: Colors,
    status: text_input::Status,
) -> text_input::Style {
    let (background, border_color) = match status {
        text_input::Status::Focused { .. } => (colors.surface_app, colors.action_active),
        text_input::Status::Hovered => (colors.surface_hover, colors.border_default),
        _ => (iced::Color::TRANSPARENT, iced::Color::TRANSPARENT),
    };
    text_input::Style {
        background: iced::Background::Color(background),
        border: iced::Border {
            color: border_color,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        // 裁定170 M01: fork(0.15.0-dev)で `icon` フィールドが消えた。
        // `.icon(..)` 呼び出しはこの crate に無い(usage 実測)ため見た目不変。
        placeholder: colors.text_muted,
        value: colors.text_primary,
        selection: colors.action_active,
    }
}

// `value_input_style` は裁定160 切片5(pane split survey §2.4/§6)で
// `chrome::value_input_style` へ移設した(純粋な再配置・挙動ゼロ変更)。

/// **ATTRS**: mock 断片には対応が無い行(Blend)だけ残す — Name は ident 帯へ、
/// Hidden は M glyph へ移した(重複 chrome を残さない、supervisor 訂正 2026-08-20)。
/// blend は**クリックで巡回するボタン**(BL2、supervisor 決定済み — pick_list は
/// next/ 全体に前例が無いので導入しない)。巡回先は [`SUPPORTED_BLEND_MODES`]
/// (現状 Normal→Add→Normal の2値、engine が対応する分だけ)。意匠は新規発明せず
/// `motolii_settings_pane::chrome::button_style`(`checkerboard_row` と同じ「押すたび
/// 即トグル」の形、他の意味色ロールは足さない)を流用する。
fn attrs_section(
    attrs: &AttrsProjection,
    speed_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let blend_content = row_widget![
        text("Blend")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        button(text(attrs.blend_mode.clone()).size(dims.body_text))
            .on_press(Message::CycleBlendMode)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    // `.prow` 系の行として同じ hairline を使う(mock 断片には Blend 行自体は
    // 無いが、`.prow` の row grammar をそのまま延長する — 発注書 NON-GOALS に
    // ある「新しい視覚言語の発明」ではなく、既存 grammar の適用)。
    let blend_row = bordered_row(blend_content.into(), dims);

    column![
        section_header("ATTRS", dims, colors),
        blend_row,
        speed_row(attrs, speed_draft, dims, colors),
    ]
    .into()
}

/// Speed 行(SP1 第一波、supervisor 決定1-7)。**click→type**(drag-to-scrub は
/// 第一波に含めない、NON-GOALS)— `text_input` は常に存在し、Name 欄
/// ([`ident_band`])と同じ「フォーカスするだけで打鍵できる」形。Enter
/// (`Message::SpeedSubmit`)で確定、下書きが無い間は投影の現在値を表示する。
/// Reset ボタンは 100% でも常に出す(押せるが変わらない = 無反応ゼロより一貫を
/// 優先、決定7)。
fn speed_row(
    attrs: &AttrsProjection,
    speed_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = speed_draft
        .map(|text| text.to_owned())
        .unwrap_or_else(|| format_number(attrs.speed_percent, 1));

    // 裁定170 M01: fork の text_input が借用寿命を返り値に縛るため
    // owned move(値不変)。
    // 線化 D2(裁定179): 常設の text_input なので `value_input_style`(常時箱)
    // ではなく name 欄と同じ [`name_input_style`](平常=素・hover=面+枠・
    // focus=箱+focus 縁)へ合流 — 2箇所で別の意匠を発明しない。
    let value_field = text_input("", displayed)
        .on_input(Message::SpeedInput)
        .on_submit(Message::SpeedSubmit)
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text("Speed")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
        text("%").size(dims.caption_text).color(colors.text_muted),
        button(text("Reset").size(dims.caption_text))
            .on_press(Message::ResetSpeed)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// mock の hint 行。**「Drag to scrub」は実装済みなので復活させる**
/// (drag-to-scrub、利用者依頼)。「double-click to type」も実際の挙動と違う —
/// この実装の値セルは、動かさず release すれば単クリックで打鍵できる
/// (二度打ちは要らない)ので「click」へ言い換える(M13: 実装と違う手順を
/// 案内しない)。「Esc to cancel」も drag の復元・打鍵下書きの破棄の両方で
/// 今回初めて本当に効く(`motolii_shell::Shell::cancel_inspector_interaction`)。
fn hint_row(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    // `.width(Length::Fill)`: 同上(柵で発見)— mock の `.hint` も pane 全幅の帯。
    // 線化 D5(裁定179 文法1): hint 行の縁取りも罫線 — 透明化(幅だけ残す=
    // 幾何不変)。注記は ink 段(`text_muted`)と間隔だけで区別する。
    container(
        text("drag to scrub · click to type · Esc to cancel")
            .size(dims.caption_text)
            .color(colors.text_muted),
    )
    .width(Length::Fill)
    .padding([dims.spacing_xs, dims.spacing_m])
    .style(move |_theme| container::Style {
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
            },
            masks: vec![],
            effects: vec![],
            text: None,
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
            },
            masks: vec![],
            effects: vec![],
            text: None,
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
        };
        let (start, _) = drag_origin(&selection, field).expect("mask opacity は editable のはず");
        assert_eq!(start, 80.0, "起点は投影の表示値(%)のはず");
        assert!(
            drag_origin(&selection, TransformField::MaskOpacity(MaskId(9))).is_none(),
            "別の mask id の field では drag を始めないはず"
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
