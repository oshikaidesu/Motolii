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
    property, Document, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrsPatch, LayerId,
    LayerSource, PropertyId, StoreError, StoreView, Value,
};

use motolii_settings_pane::chrome::{parse_number, section_header, value_input_style};
use motolii_shell_state::Session;
use motolii_tokens_rs::{Colors, Dimensions, Ink, TextWeight};

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
}

impl TransformField {
    fn property_name(self) -> &'static str {
        match self {
            Self::PositionX | Self::PositionY => property::POSITION,
            Self::PositionZ => property::POSITION_Z,
            Self::ScaleX | Self::ScaleY => property::SCALE,
            Self::Rotation => property::ROTATION,
            Self::Opacity => property::OPACITY,
            Self::AnchorX | Self::AnchorY => property::ANCHOR,
        }
    }
}

/// この field の store 上の property。標準 property は予約語でも空でもないので
/// 失敗し得ない — `motolii_shell::Shell` はこの `Result` を「コードの誤り」として扱ってよい。
pub fn property_id(field: TransformField) -> Result<PropertyId, StoreError> {
    PropertyId::new(field.property_name())
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
        // 表示は % だが store は 0..1 の比(`property::OPACITY` の既定と同じ単位)。
        TransformField::Opacity => Value::F64((input / 100.0).clamp(0.0, 1.0)),
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
}

impl KeyRow {
    pub fn property_name(self) -> &'static str {
        match self {
            Self::Position => property::POSITION,
            Self::Scale => property::SCALE,
            Self::Rotation => property::ROTATION,
            Self::Opacity => property::OPACITY,
            Self::Anchor => property::ANCHOR,
        }
    }
}

/// この行の store 上の property。標準 property なので失敗し得ない
/// ([`property_id`] と同じ理由 — 呼び手は `Result` を「コードの誤り」として扱ってよい)。
pub fn key_row_property_id(row: KeyRow) -> Result<PropertyId, StoreError> {
    PropertyId::new(row.property_name())
}

/// track がまだ無い行の既定値(`project` の各行の default と同じ値 —
/// Scale だけ等倍、Opacity は store 単位の 0..1 で 1.0、他は 0)。
/// 静的値も無い行を初キー化する時の値の正本。
pub fn key_row_default_value(row: KeyRow) -> Value {
    match row {
        KeyRow::Position | KeyRow::Anchor => Value::Vec2([0.0, 0.0]),
        KeyRow::Scale => Value::Vec2([1.0, 1.0]),
        KeyRow::Rotation => Value::F64(0.0),
        KeyRow::Opacity => Value::F64(1.0),
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

/// [`project`] が組む行の decimals は field ごとに固定(Position/Scale/Anchor=3、
/// Rotation=1、Opacity=0)。click→type 編集の下書き初期値を作るのに要る
/// (`TransformRowProjection::decimals` を持つ行を毎回作り直さずに済む)。
pub fn field_decimals(field: TransformField) -> usize {
    match field {
        TransformField::Rotation => 1,
        TransformField::Opacity => 0,
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
        TransformField::Opacity => 1.0,
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
    None
}

/// text_input へ確定的な `Id` を割る — click→type 編集へ切り替わった直後、
/// `iced::widget::operation::focus` でこのセルへフォーカスを戻すために要る
/// (mouse_area は press を own できても、フォーカスは text_input 自身の仕事
/// — click 直後にはまだ text_input が木に無いので自動フォーカスされない)。
pub fn field_input_id(field: TransformField) -> iced::widget::Id {
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
    };

    let kind = meta
        .as_ref()
        .map(|meta| source_kind_label(&meta.source))
        .unwrap_or("layer");

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

/// mock の `.cols`/`.prow` 系の行の border-bottom を模す共通ラッパー(裁定137
/// 「区切りは面でなく線」・裁定139「面色の塗り分けで区切っている残余を
/// hairline へ置換する」)。padding・固定高・pane 全幅は**ここ(外側
/// container)だけ**が持つ — `content` 自身は spacing/align_y だけを持ち、
/// 自分の width/height を宣言しない(`ident_band` と同じ構造。Fill な子孫
/// [label 等]を持つ Shrink な row は祖先の container が与える Limits の上限
/// までしか伸びないので、外側 container の bounds とは一致しない — 496幅
/// ちょうど/20px高ちょうどの `Container` candidate が二重に現れて
/// `tests/inspector_pixel_fence.rs` の数え上げを壊す事故を避けられる、実測)。
///
/// **既知の限界**(`tests/inspector_pixel_fence.rs` 冒頭に明記済みの限界と
/// 同じ trade-off): mock は border-bottom のみだが `iced_core::Border`
/// (0.14.0 実測)は4辺一律にしかできない(per-edge API 無し)。header/
/// ident_band/section_header と同じ trade-off をここでも受け入れる。
fn bordered_row(
    content: Element<'static, Message>,
    dims: Dimensions,
    border_color: iced::Color,
) -> Element<'static, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fixed(dims.inspector_row_height))
        .padding([0.0, dims.spacing_m])
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme| container::Style {
            border: iced::Border {
                color: border_color,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
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
            dims,
            colors,
        ),
    };

    container(body)
        .width(Length::Fixed(dims.inspector_panel_width))
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_panel)),
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
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
        row_widget![identity, glyphs]
            .spacing(dims.spacing_s)
            .align_y(iced::alignment::Vertical::Center),
    )
    .padding([dims.spacing_s, dims.spacing_m])
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(colors.surface_raised)),
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
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

    // mock `.cols{border-bottom:var(--line) solid #1a1a1a}` — 不透明な hairline
    // (`border_default` と同値)。
    bordered_row(content.into(), dims, colors.border_default)
}

// `section_header` は裁定160 切片5(pane split survey §2.4/§6)で
// `chrome::section_header` へ移設した(純粋な再配置・挙動ゼロ変更)。

/// 発注書の固定列グリッド `Property | X | Y | Z | Key` = `1fr + 3×value幅 + hit`。
/// scalar 行(Opacity)は mock どおり3列目(Z の位置)へ値を置き、残り2列は
/// 空箱で埋める([`blank_value_cell`] — `absent_component` の「このモデルに
/// 無い軸」とは別の意味で、単に scalar 行が3値グリッドに収まるための穴埋め)。
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

    // mock `.prow{border-bottom:var(--line) solid rgba(0,0,0,.35)}` — `.cols`
    // より薄い hairline(裁定142 の先行整備で `tokens::Colors::border_hairline_weak`
    // へ昇格済み)。
    bordered_row(content.into(), dims, colors.border_hairline_weak)
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
    let blend_row = bordered_row(blend_content.into(), dims, colors.border_hairline_weak);

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

    bordered_row(content.into(), dims, colors.border_hairline_weak)
}

/// mock の hint 行。**「Drag to scrub」は実装済みなので復活させる**
/// (drag-to-scrub、利用者依頼)。「double-click to type」も実際の挙動と違う —
/// この実装の値セルは、動かさず release すれば単クリックで打鍵できる
/// (二度打ちは要らない)ので「click」へ言い換える(M13: 実装と違う手順を
/// 案内しない)。「Esc to cancel」も drag の復元・打鍵下書きの破棄の両方で
/// 今回初めて本当に効く(`motolii_shell::Shell::cancel_inspector_interaction`)。
fn hint_row(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    // `.width(Length::Fill)`: 同上(柵で発見)— mock の `.hint` も pane 全幅の帯。
    container(
        text("drag to scrub · click to type · Esc to cancel")
            .size(dims.caption_text)
            .color(colors.text_muted),
    )
    .width(Length::Fill)
    .padding([dims.spacing_xs, dims.spacing_m])
    .style(move |_theme| container::Style {
        border: iced::Border {
            color: colors.border_default,
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
            },
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
            },
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
        assert_eq!(KeyRow::Position.property_name(), property::POSITION);
        assert_eq!(KeyRow::Scale.property_name(), property::SCALE);
        assert_eq!(KeyRow::Rotation.property_name(), property::ROTATION);
        assert_eq!(KeyRow::Opacity.property_name(), property::OPACITY);
        assert_eq!(KeyRow::Anchor.property_name(), property::ANCHOR);

        assert_eq!(key_row_default_value(KeyRow::Position), Value::Vec2([0.0, 0.0]));
        assert_eq!(key_row_default_value(KeyRow::Scale), Value::Vec2([1.0, 1.0]));
        assert_eq!(key_row_default_value(KeyRow::Rotation), Value::F64(0.0));
        assert_eq!(key_row_default_value(KeyRow::Opacity), Value::F64(1.0));
        assert_eq!(key_row_default_value(KeyRow::Anchor), Value::Vec2([0.0, 0.0]));
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
