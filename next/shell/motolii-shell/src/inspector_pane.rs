//! Inspector pane(第1波: Transform行 + Attrs行)。
//!
//! **視覚正本は `docs/mocks-ui/public/inspector-library.html` + `.css` そのもの**
//! (発注書 CANON)。旧 `crates/` 側の egui/iced 実装は手本にしない — `next/` は
//! 移植元ではなく成果を作る側(`../GOALS.md` 冒頭の規律どおり)。
//!
//! [`project`] が `StoreView`/[`crate::Session`] から**Document の写しではない、
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
//! (`ui-quality-bar` Q2)。[`crate::Shell`] は [`FieldDraft`]/name 下書きという
//! **Document ではない一時状態**(`crate::Shell::pending_drops` と同じ形)を持ち、
//! `on_submit`(Enter)で初めて1回の `Intent::SetTrack`/`SetAttrs` を出す — 1 gesture
//! = 1 undo。**静的値の編集は `SetTrack` に1キー `Hold`** で書く([`single_hold_track`])
//! — 発注書がその流儀を名指ししている。
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
//! **transient 値は `Document` へ直接書く**([`crate::Shell::continue_field_drag`]) —
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
//! `iced::event::listen_with` で拾う形に倒した(`crate::inspector_pointer_event`)。
//! mouse_area の `on_press` は「この field の drag を armed にする」ためだけに使う。

use motolii_core::RationalTime;
use motolii_store::{
    property, Interp, Keyframe, KeyframeTrack, LayerId, LayerSource, PropertyId, StoreError,
    StoreView, Value,
};

use crate::tokens::{Colors, Dimensions, Ink, TextWeight};
use crate::{Message, Session};

// ---------------------------------------------------------------------------
// 型別 editor の対象 field
// ---------------------------------------------------------------------------

/// Transform 行が動かす field の識別。**`LayerId` を持たない** — 対象は常に
/// `Session::selection`(commit 時に読む)。選択が edit の合間に変わる稀なケースは
/// 「そのまま捨てる」で安全側に倒す(`crate::Shell::commit_inspector_field` 参照)。
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
/// 失敗し得ない — `crate::Shell` はこの `Result` を「コードの誤り」として扱ってよい。
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

/// 入力文字列 → 数値。mock(`inspector-library.html`)は負号に `−`(U+2212)を使うので
/// 両対応する。
pub fn parse_number(text: &str) -> Option<f64> {
    text.trim().replace('\u{2212}', "-").parse::<f64>().ok()
}

pub fn format_number(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
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
/// 置く** — 寸法・色ではなく値の型に紐づく振る舞いなので([`crate::tokens`] は
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
/// (`crate::Shell::continue_field_drag`)が結果を `next_value` へ渡して store の
/// `Value` へ変換する。
pub fn dragged_value(field: TransformField, start_value: f64, delta_px: f32, fine: bool) -> f64 {
    let step = drag_step_per_pixel(field);
    let factor = if fine { DRAG_SHIFT_FACTOR } else { 1.0 };
    start_value + f64::from(delta_px) * step * factor
}

/// drag(または click→type 編集)を始める前に読む、`field` の現在値。
/// **投影から読むだけ** — `project` が計算した表示単位の値をそのまま使う
/// (Opacity の % 換算などを2箇所に書かない)。animated(編集不可)/対応する
/// field が投影に無い、のいずれも `None`(呼び手はドラッグも編集も始めない —
/// `commit_inspector_field` と同じ二重の柵)。
///
/// 戻り値の第2要素は Vec2 系(Position/Scale/Anchor)の「動かさない方の成分」
/// ([`next_value`] にそのまま渡す) — scalar 系(Z/Rotation/Opacity)では未使用
/// (`[0.0, 0.0]` のダミー)。
pub fn drag_origin(selection: &SelectionProjection, field: TransformField) -> Option<(f64, [f64; 2])> {
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
    /// track が 0〜1 キー(裁定20「キーを打っていない property は静止値」の範囲)なら
    /// 編集可。2キー以上(animated)は**この第1波では表示のみ**(発注書の指示 —
    /// 理由つきdisabledではなく、そもそも編集用 control を出さない)。
    pub editable: bool,
    /// この成分が編集される時に動く field。`present=false` なら `None`。
    pub field: Option<TransformField>,
}

fn absent_component(axis: &'static str) -> ComponentSlot {
    ComponentSlot {
        axis,
        present: false,
        value: 0.0,
        editable: false,
        field: None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RowValue {
    Vector([ComponentSlot; 3]),
    Scalar(ComponentSlot),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformRowProjection {
    pub label: &'static str,
    pub value: RowValue,
    pub decimals: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttrsProjection {
    pub name: String,
    pub hidden: bool,
    /// **表示のみ**(KNOWN.md: 対応 mode が Normal だけ — 既知の穴であって新発見では
    /// ない)。`BlendMode` の `Debug` 表示をそのまま使う(`Normal`/`Multiply`/…)。
    pub blend_mode: String,
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
    let editable = track.as_ref().map(|tr| tr.keys().len() <= 1).unwrap_or(true);
    let value = match store.value_at(layer, &property, t)? {
        Some(Value::F64(v)) => v,
        _ => default,
    };
    Ok(ComponentSlot {
        axis,
        present: true,
        value,
        editable,
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
    let editable = track.as_ref().map(|tr| tr.keys().len() <= 1).unwrap_or(true);
    let [x, y] = match store.value_at(layer, &property, t)? {
        Some(Value::Vec2(v)) => [v[0], v[1]],
        _ => default,
    };
    Ok([
        ComponentSlot {
            axis: "X",
            present: true,
            value: x,
            editable,
            field: Some(field_x),
        },
        ComponentSlot {
            axis: "Y",
            present: true,
            value: y,
            editable,
            field: Some(field_y),
        },
    ])
}

/// `store`/`session` から選択層の Inspector 投影を組み立てる。**読むだけ**。
/// 選択なし・選択層が削除済み(present でない)・comp が無い、のいずれも `Ok(None)`
/// (M13: 壊れているのではなく「まだ映す物が無い」)。
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
    };

    let attrs = store.attrs(layer)?.unwrap_or_default();
    let attrs_projection = AttrsProjection {
        name: attrs.name,
        hidden: attrs.hidden,
        blend_mode: format!("{:?}", attrs.blend_mode),
    };

    let kind = store
        .meta(layer)?
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
// view — StoreView の投影(SelectionProjection)と下書きだけを受け取る。書けない。
// ---------------------------------------------------------------------------

use iced::widget::{
    button, column, container, mouse_area, row as row_widget, scrollable, text, text_input, Space,
};
use iced::{Element, Length};

/// mock `.prow`(値行)の border-bottom 色。生の CSS リテラル `rgba(0,0,0,.35)`
/// — `Colors` の意味色ロールには対応が無い(`ui/motolii-tokens` の DTCG 正本は
/// この lane の write-set 外なので新ロールを追加しない)。`.cols`/header 等が
/// 使う不透明 `#1a1a1a`(`Colors::border_default` と同値)より薄い、行同士の
/// 弱い区切り(裁定137「区切りは面でなく線」)。
// `pub(crate)`: `screenshot.rs` の検分器具が同じ hairline 色を再利用する
// (発注書「同じ tokens・同じ読み口から描く」— 別の rgba リテラルを新規発明しない)。
pub(crate) const PROW_HAIRLINE: iced::Color = iced::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.35,
};

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
    // **`--section`(26)**: mock の ptitle(パネルタイトル)は section 見出しと
    // 同じ高さトークンを共有する(`inspector_section_header_height`) — 旧実装は
    // Shell 全体の `panel_header_height`(29、Ableton実測)を誤って流用していた。
    //
    // **`.width(Length::Fill)` は柵で見つかった実修正**(`tests/inspector_pixel_fence.rs`):
    // `container(text(...))` の既定幅は content の `size_hint` 追従(`Length::Shrink`)
    // なので、これが無いと帯が "Inspector" の文字幅ぶんしか広がらず、mock の
    // `.ptitle`(block要素、pane 全幅の帯)と食い違う(実測: 修正前は幅 67.5px)。
    let header = container(
        text("Inspector")
            .size(dims.title_text)
            .color(colors.text_primary),
    )
    .width(Length::Fill)
    .height(Length::Fixed(dims.inspector_section_header_height))
    .padding([0.0, dims.spacing_m])
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_theme| container::Style {
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });

    let body: Element<'static, Message> = match projection {
        None => empty_state(dims, colors),
        Some(selection) => selected_body(selection, field_draft, name_draft, dims, colors),
    };

    container(column![header, body])
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
    rows = rows.push(attrs_section(&selection.attrs, dims, colors));
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
    // `Message::InspectorNameInput/Submit`)。未フォーカス時は枠を消して
    // 静止テキストに見せる([`name_input_style`])。`.font(TextWeight::
    // Semibold)` で mock の 600 を写す(裁定137)。
    //
    // padding は縦0を維持(`value_cell` と同じ柵発見 — 既定 padding 5px が
    // 乗ると ident 帯の高さが mock の「b(11px)+s(9px)を2行積んだだけ」より
    // 約10px 余計に伸びる、実測: 修正前 name_field 高 24.3px、修正後
    // 14.3px)。**横だけ** [`name_field_padding`] で戻す(裁定139)。
    let name_field = text_input(&placeholder, &name_text)
        .on_input(Message::InspectorNameInput)
        .on_submit(Message::InspectorNameSubmit)
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

/// `pub(crate)`: `settings_pane` も同じ見出し帯(パネルタイトル/section 見出し
/// 共通トークン)を再利用する — 2箇所で別の意匠を発明しない。
pub(crate) fn section_header(
    label: &'static str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    // `.width(Length::Fill)`: `header` と同じ理由(柵で発見) — mock の `.sec` も
    // block 要素で pane 全幅の帯(実測: 修正前は幅 65〜68px)。
    //
    // **背景は塗らない**(裁定137/139、2026-08-21 更正): mock `.sec` は
    // `background`/`border` のどちらも持たない — 見出しは letter-spacing +
    // ink3(`text_muted`)+ 行高だけで区別する(旧実装は `surface_app` で塗って
    // 「面色の塗り分けで区切る」を犯していた — TRANSFORM/APPEARANCE/ATTRS の
    // 帯が周囲の `.prow` 行と違う沈んだ色の箱に見えていたのが実体)。
    container(
        text(label)
            .size(dims.caption_text)
            .color(Ink::Muted.resolve(&colors)),
    )
    .width(Length::Fill)
    .height(Length::Fixed(dims.inspector_section_header_height))
    .padding([0.0, dims.spacing_m])
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

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
        row_widget(value_cells).spacing(dims.spacing_xs),
        reserved_glyph(dims), // Key 列 — keyframe UI 未実装(Q0)。幅の予約だけ、空のまま。
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    // mock `.prow{border-bottom:var(--line) solid rgba(0,0,0,.35)}` — `.cols`
    // より薄い hairline。
    bordered_row(content.into(), dims, PROW_HAIRLINE)
}

/// 発注書「読み取り専用値は編集セルと同一形状で色だけ落とす」を1箇所で守る —
/// absent(muted)・editable(text_input)・animated(accent, 表示のみ)のどれでも
/// 同じ箱(背景 `surface_app`・同じ幅高さ)を作る。
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
                    text_input("", &displayed)
                        .id(field_input_id(field))
                        .on_input(move |text| Message::InspectorFieldInput(field, text))
                        .on_submit(Message::InspectorFieldSubmit(field))
                        .size(dims.body_text)
                        .width(Length::Fill)
                        // 縦0を維持(柵で発見した実修正 — `text_input` の既定 padding
                        // `iced_widget::text_input::DEFAULT_PADDING` = 5px 全辺が固定高
                        // `value_cell_height`(row-4 = 16px)を食い潰し、文字の描画領域が
                        // 16 - 2*5 = 6px まで押し潰される、実測: 修正前は text_input 内の
                        // paragraph 高が 6px)。**横だけ** [`value_cell_padding`] で戻す
                        // (裁定139: セル幅38pxいっぱいに文字が縁へ接触しないよう最小段
                        // トークン `spacing_xs` を左右に確保)。
                        .padding(value_cell_padding(dims))
                        .align_x(iced::alignment::Horizontal::Center)
                        .style(move |_theme, status| value_input_style(dims, colors, status)),
                )
                .width(Length::Fixed(dims.inspector_value_width))
                .height(Length::Fixed(value_cell_height(dims)))
                .align_y(iced::alignment::Vertical::Center)
                .into()
            } else {
                // click せず(まだ)編集していない見た目 — drag-to-scrub の起点
                // ([`draggable_value_cell`])。表示する値は投影(`slot.value`)
                // そのものなので、drag 中の transient 値もここが自動で映す。
                draggable_value_cell(field, format_number(slot.value, decimals), dims, colors)
            }
        }
        // animated(2キー以上) — **表示のみと明示**(理由つきdisabledではなく、
        // そもそも編集 control を出さない。accent 色で「動いている値」と分かる —
        // 箱形自体は編集セルと同じ)。
        _ => boxed_value(format_number(slot.value, decimals), colors.action_active, dims, colors),
    }
}

/// present・editable(un-keyed)な field の**まだ編集していない**見た目。
/// `mouse_area` は press だけを own する — move/release は window 全体を追う
/// `Shell::subscription` 側の担当(`crate::inspector_pointer_event`)。iced 0.14
/// の `mouse_area` は自分の bounds を出た cursor を追えない(pointer capture が
/// 無い実測)ので、値セル自身の当たり判定は「drag を armed にする press」だけに
/// 絞ってある — 感度どおりに動かすとすぐこの38px幅を出るため。
fn draggable_value_cell(
    field: TransformField,
    displayed: String,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    mouse_area(
        container(
            text(displayed)
                .size(dims.body_text)
                .color(colors.text_primary)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(dims.inspector_value_width))
        .height(Length::Fixed(value_cell_height(dims)))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_app)),
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }),
    )
    .interaction(iced::mouse::Interaction::ResizingHorizontally)
    .on_press(Message::InspectorValuePressed(field))
    .into()
}

/// mock `.prow .v { height: calc(var(--row) - 4*var(--s)*1px) }` の `4` は
/// `spacing_s`(既定4)と同じ値 — スケール済みの `spacing_s` を使うことで
/// `ui_scale` を再度掛け直さずに済む(適用点は `Dimensions::scaled` の1箇所だけ)。
fn value_cell_height(dims: Dimensions) -> f32 {
    (dims.inspector_row_height - dims.spacing_s).max(1.0)
}

/// 値セル(`.prow .v`)の text_input 横内余白(裁定139)。**縦は0のまま** —
/// 行高合わせの実測修正([`value_cell_height`] の doc 参照)。mock 自身の
/// `.prow .v` は padding を持たない(flex center)ので実測値の直接転記では
/// なく、grid gap の最小段トークン `spacing_xs`(mock `--sp1`=2px、cols/prow
/// の X→Y→Z 間隔と同じ token)を左右に使う — セル幅38pxの縁へ数字グリフが
/// 接触しない最小限の呼吸(セル幅自体は変えない、38px のまま)。
fn value_cell_padding(dims: Dimensions) -> iced::Padding {
    iced::Padding::from([0.0, dims.spacing_xs])
}

/// ident 帯の名前欄(`.ident b`)の横内余白。[`value_cell_padding`] と同じ
/// 理由・同じトークンを使う(裁定139 は `value_cell`/`name_field` を並記して
/// いる — 2箇所で別の値を発明しない)。
fn name_field_padding(dims: Dimensions) -> iced::Padding {
    iced::Padding::from([0.0, dims.spacing_xs])
}

fn boxed_value(
    content: String,
    color: iced::Color,
    dims: Dimensions,
    colors: Colors,
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
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(colors.surface_app)),
        ..container::Style::default()
    })
    .into()
}

/// scalar 行(Opacity)の空き枠(X/Y列)。中身の無い箱 — grid の穴埋めであって
/// 「このモデルに無い軸」ではない(`value_cell` の absent 表現とは別の意味)。
fn blank_value_cell(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fixed(dims.inspector_value_width))
        .height(Length::Fixed(value_cell_height(dims)))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_app)),
            ..container::Style::default()
        })
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
    .on_press(Message::InspectorToggleHidden)
    .style(move |_theme, status| glyph_button_style(dims, colors, status, hidden))
    .into()
}

/// 列幅の予約だけ(**空のまま** — Q0: 押せそうに見えて押せない chrome を作らない)。
/// S glyph(solo、engine/store 未実装)と各行の Key 列(keyframe UI 未実装)の両方が
/// これを使う — 内容も枠も無い、幅だけの `Space`。
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
    let (border_color, text_color) = if active {
        (colors.action_active, colors.action_active)
    } else {
        (colors.border_default, Ink::Secondary.resolve(&colors))
    };
    let background = match status {
        button::Status::Hovered => colors.surface_hover,
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color,
        border: iced::Border {
            color: border_color,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}

/// ident 帯の名前欄。未フォーカス時は枠・背景を消して静止 bold テキストに見せ、
/// フォーカス時だけ枠と背景を出す(mock はここを編集可能な `text_input` として
/// 描いていない — 実装が実際に持つ機能=改名を隠さないための最小限の意匠)。
fn name_input_style(dims: Dimensions, colors: Colors, status: text_input::Status) -> text_input::Style {
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
        icon: colors.text_muted,
        placeholder: colors.text_muted,
        value: colors.text_primary,
        selection: colors.action_active,
    }
}

/// `pub(crate)`: `settings_pane` の数値欄(背景RGBA・ui_scale%)も同じ枠色
/// ロールを使う — 2箇所で別の意匠を発明しない。
pub(crate) fn value_input_style(
    dims: Dimensions,
    colors: Colors,
    status: text_input::Status,
) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => colors.action_active,
        _ => colors.border_default,
    };
    text_input::Style {
        background: iced::Background::Color(colors.surface_app),
        border: iced::Border {
            color: border_color,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        icon: colors.text_muted,
        placeholder: colors.text_muted,
        value: colors.text_primary,
        selection: colors.action_active,
    }
}

/// **ATTRS**: mock 断片には対応が無い行(Blend)だけ残す — Name は ident 帯へ、
/// Hidden は M glyph へ移した(重複 chrome を残さない、supervisor 訂正 2026-08-20)。
/// blend 自体は**表示のみ**(対応 mode が Normal だけなのは KNOWN の既知の穴)。
fn attrs_section(attrs: &AttrsProjection, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let blend_content = row_widget![
        text("Blend")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        text(attrs.blend_mode.clone())
            .size(dims.body_text)
            .color(colors.text_muted),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    // `.prow` 系の行として同じ hairline を使う(mock 断片には Blend 行自体は
    // 無いが、`.prow` の row grammar をそのまま延長する — 発注書 NON-GOALS に
    // ある「新しい視覚言語の発明」ではなく、既存 grammar の適用)。
    let blend_row = bordered_row(blend_content.into(), dims, PROW_HAIRLINE);

    column![section_header("ATTRS", dims, colors), blend_row].into()
}

/// mock の hint 行。**「Drag to scrub」は実装済みなので復活させる**
/// (drag-to-scrub、利用者依頼)。「double-click to type」も実際の挙動と違う —
/// この実装の値セルは、動かさず release すれば単クリックで打鍵できる
/// (二度打ちは要らない)ので「click」へ言い換える(M13: 実装と違う手順を
/// 案内しない)。「Esc to cancel」も drag の復元・打鍵下書きの破棄の両方で
/// 今回初めて本当に効く(`crate::Shell::cancel_inspector_interaction`)。
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

    // -----------------------------------------------------------------------
    // 裁定139: value_cell/name_field は縦0を維持したまま横だけ内余白を戻す
    // -----------------------------------------------------------------------

    /// **本命(red→green の柵)**: 旧実装は `.padding(0.0)` で縦横とも0だった
    /// (`git log` 参照 — このテストを旧コードに当てると
    /// `padding.left == 0.0`/`padding.right == 0.0` が真になり fail する)。
    /// 縦は行高合わせのため0のまま、横だけ `spacing_xs`(mock `--sp1`)が
    /// 入っていること。
    #[test]
    fn value_cell_padding_keeps_the_vertical_zero_and_restores_only_horizontal_inset() {
        let dims = Dimensions::default();
        let padding = value_cell_padding(dims);
        assert_eq!(padding.top, 0.0, "縦(上)は行高合わせのため0のはず");
        assert_eq!(padding.bottom, 0.0, "縦(下)は行高合わせのため0のはず");
        assert_eq!(padding.left, dims.spacing_xs, "横(左)の内余白が戻っていない");
        assert_eq!(padding.right, dims.spacing_xs, "横(右)の内余白が戻っていない");
        assert!(padding.left > 0.0, "横の内余白が0のまま(旧バグの再発)");
    }

    #[test]
    fn name_field_padding_matches_value_cell_padding_the_same_way() {
        let dims = Dimensions::default();
        assert_eq!(name_field_padding(dims), value_cell_padding(dims));
    }

    /// 150%でも横内余白がスケールに追従すること(適用点は `Dimensions::scaled`
    /// の1箇所だけ、という裁定117の不変量をここでも保つ)。
    #[test]
    fn value_cell_padding_scales_with_ui_scale() {
        let dims = Dimensions::default().scaled(1.5);
        let padding = value_cell_padding(dims);
        assert_eq!(padding.left, Dimensions::default().spacing_xs * 1.5);
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
            crate::tokens::TextWeight::Bold.font().weight,
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
        assert_eq!(next_value(TransformField::Opacity, 50.0, [0.0, 0.0]), Value::F64(0.5));
        // クランプ: 100 を超える入力・負の入力は store の 0..1 に収める。
        assert_eq!(next_value(TransformField::Opacity, 150.0, [0.0, 0.0]), Value::F64(1.0));
        assert_eq!(next_value(TransformField::Opacity, -10.0, [0.0, 0.0]), Value::F64(0.0));
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
    }

    // -----------------------------------------------------------------------
    // drag-to-scrub — 感度表(発注書の表そのもの)
    // -----------------------------------------------------------------------

    #[test]
    fn dragged_value_applies_the_registry_sensitivity_per_field() {
        // Position/Anchor/Z = 1px→1.0。
        assert_eq!(dragged_value(TransformField::PositionX, 0.0, 10.0, false), 10.0);
        assert_eq!(dragged_value(TransformField::AnchorY, 0.0, -4.0, false), -4.0);
        assert_eq!(dragged_value(TransformField::PositionZ, 0.0, 3.0, false), 3.0);
        // Scale = 1px→0.01。
        assert!((dragged_value(TransformField::ScaleX, 1.0, 10.0, false) - 1.1).abs() < 1e-9);
        // Rotation = 1px→0.5度。
        assert!((dragged_value(TransformField::Rotation, 0.0, 10.0, false) - 5.0).abs() < 1e-9);
        // Opacity = 1px→1(%)。
        assert_eq!(dragged_value(TransformField::Opacity, 50.0, 20.0, false), 70.0);
    }

    #[test]
    fn shift_drag_uses_a_tenth_of_the_normal_sensitivity() {
        let normal = dragged_value(TransformField::PositionX, 0.0, 100.0, false);
        let fine = dragged_value(TransformField::PositionX, 0.0, 100.0, true);
        assert_eq!(normal, 100.0);
        assert!((fine - 10.0).abs() < 1e-9, "Shift+drag は1/10のはず: {fine}");
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
                        field: Some(TransformField::ScaleX),
                    },
                    ComponentSlot {
                        axis: "Y",
                        present: true,
                        value: 2.0,
                        editable: true,
                        field: Some(TransformField::ScaleY),
                    },
                    absent_component("Z"),
                ]),
                decimals: 3,
            }],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
            },
        };

        let (start, current_vec2) =
            drag_origin(&selection, TransformField::ScaleX).expect("editable のはず");
        assert_eq!(start, 1.0);
        assert_eq!(current_vec2, [1.0, 2.0], "動かさない方(Y)を保っていない");

        // 対応する field が投影に無ければ `None`(呼び手はドラッグを始めない)。
        assert!(drag_origin(&selection, TransformField::Rotation).is_none());
    }

    #[test]
    fn drag_origin_refuses_animated_fields() {
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
                        editable: false, // animated(2キー以上)
                        field: Some(TransformField::Rotation),
                    },
                ]),
                decimals: 1,
            }],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
            },
        };
        assert!(
            drag_origin(&selection, TransformField::Rotation).is_none(),
            "animated な field はドラッグを始められないはず"
        );
    }

    #[test]
    fn field_decimals_matches_the_projection_rows() {
        assert_eq!(field_decimals(TransformField::PositionX), 3);
        assert_eq!(field_decimals(TransformField::ScaleY), 3);
        assert_eq!(field_decimals(TransformField::AnchorX), 3);
        assert_eq!(field_decimals(TransformField::Rotation), 1);
        assert_eq!(field_decimals(TransformField::Opacity), 0);
    }
}
