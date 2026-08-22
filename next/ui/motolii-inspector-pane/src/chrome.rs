//! pane 横断の見た目 ── 値セル・HoverValueBox・button/container style・
//! ラベル色チップ・共通 widget(glyph 系)。
//!
//! **持つ**: 値セルそのものの意匠([`value_cell`]/`draggable_value_cell`/
//! `HoverValueBox`)・行ラッパー([`bordered_row`]/`row_band_style`)・
//! button/container/text_input の style 関数群([`flat_button_style`]/
//! `glyph_button_style`/`value_box_style`/`name_input_style`)・glyph 系
//! widget(`mute_glyph`/`key_glyph`/`reserved_glyph`)・ラベル色チップの view と
//! 巡回の書き口([`next_label_color`]/`cycle_inspector_label_color`/
//! `label_color_chip`)・寸法計算の小さな純関数群(`value_cell_height`/
//! `sibling_gap_px`/…)・taffy 転写 CSS 宣言([`property_row_css`])・
//! `column_header_row`/`hint_row`。
//!
//! **持たない**: 各 section 固有の行組み立て(`transform_row`/`mask_section`/
//! `effects_section`/`text_section`/`attrs_section`)や投影の中身の判断 ──
//! ここは「どう描くか」だけを持ち、「何を描くか」は呼び手が渡す。

use motolii_settings_pane::chrome::value_input_style;
use motolii_store::{Document, Intent, LayerAttrsPatch, LayerId};
use motolii_tokens_rs::{Colors, Dimensions, Ink, TextWeight, LABEL_PALETTE_LEN};

use iced::widget::{button, container, mouse_area, row as row_widget, text, text_input, Space};
use iced::{Element, Length};

use crate::projection::{ComponentSlot, KeyCellProjection};
use crate::transform::{display_number, field_input_id, format_number, FieldDraft, KeyCellState, TransformField};
use crate::Message;

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
pub(crate) fn bordered_row(content: Element<'static, Message>, dims: Dimensions) -> Element<'static, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fixed(dims.inspector_row_height))
        .padding([0.0, dims.spacing_m])
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme| row_band_style(dims))
        .into()
}

/// ラベル色チップ(B03、ident 帯)。正方形の色見本 — 塗りは timeline の行
/// スウォッチ(`motolii-timeline-pane::lane_bar::swatch_color`)と同じ源・
/// 同じ既定: `label_color` index → `colors.label_palette`、未割当は
/// `way_timeline` へフォールバック(同じ意味役割の色を2箇所で別の式にしない)。
/// click で palette を巡回([`Message::CycleLabelColor`] → [`next_label_color`]
/// — 巡回ボタン文法、BL2 と同じ理由で pick_list は導入しない)。
pub(crate) fn label_color_chip(
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
pub(crate) fn label_chip_side(row_height: f32) -> f32 {
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
pub(crate) fn column_header_row(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
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

/// 発注書「読み取り専用値は編集セルと同一形状で色だけ落とす」を1箇所で守る —
/// absent(muted)・editable(text_input)・animated(accent, 表示のみ)のどれでも
/// 同じ形(同じ幅高さ)を作る。線化 D2(裁定179「箱は状態の器」)以降、
/// 平常はどれも素の表示(面・輪郭なし)— 箱が現れるのは editable セルの
/// hover([`value_box_style`])と編集中(`value_input_style`)だけ。
pub(crate) fn value_cell(
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
pub(crate) fn single_row_horizontal_inset(text_size: f32) -> f32 {
    (text_size * 0.6).round()
}

/// 値セル(`.prow .v`)の text_input 横内余白(裁定139・裁定168)。**縦は0の
/// まま** — 行高合わせの実測修正([`value_cell_height`] の doc 参照)。旧実装は
/// grid gap の最小段トークン `spacing_xs`(mock `--sp1`=2px)を転用していたが、
/// 裁定168(「文字の余白」)は単行の横余白を `0.6em` と定めたので、そちらへ
/// 差し替える(セル幅自体は変えない、38px のまま — 内側の呼吸だけが広がる)。
pub(crate) fn value_cell_padding(dims: Dimensions) -> iced::Padding {
    iced::Padding::from([0.0, single_row_horizontal_inset(dims.body_text)])
}

/// ident 帯の名前欄(`.ident b`)の横内余白。[`value_cell_padding`] と同じ
/// 理由・同じ式を使う(裁定139 は `value_cell`/`name_field` を並記している —
/// 2箇所で別の値を発明しない、裁定168 適用後もこの対称は保つ)。
pub(crate) fn name_field_padding(dims: Dimensions) -> iced::Padding {
    iced::Padding::from([0.0, single_row_horizontal_inset(dims.body_text)])
}

/// 兄弟要素間の gap(裁定167 の梯子下段: `0.075 × 行高`、px 最近傍丸め)。
/// `motolii-timeline-pane::lane_bar::sibling_gap_px` と同型 — 別 crate なので
/// 共有関数は置けない(式だけ揃える、値は pane ごとに token 経由で持つ)。
///
pub(crate) fn sibling_gap_px(row_height: f32) -> f32 {
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
pub(crate) fn blank_value_cell(dims: Dimensions, _colors: Colors) -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fixed(dims.inspector_value_width))
        .height(Length::Fixed(value_cell_height(dims)))
        .style(move |_theme| container::Style::default())
        .into()
}

/// Key/M/S glyph 列の高さ。mock `.glyph { height: calc(var(--row) - 2*var(--s)*1px) }`
/// の `2` は `spacing_xs`(既定2)と同じ値。
pub(crate) fn glyph_height(dims: Dimensions) -> f32 {
    (dims.inspector_row_height - dims.spacing_xs).max(1.0)
}

/// **M glyph — 結線済み**(supervisor 訂正、2026-08-20)。`LayerAttrs.hidden` を
/// トグルする。on(hidden=true)は mock `.glyph.on` と同じ accent 縁取り+文字色。
/// `.font(TextWeight::Bold)` で mock `.glyph{font-weight:800}` を写す(裁定137)。
pub(crate) fn mute_glyph(dims: Dimensions, colors: Colors, hidden: bool) -> Element<'static, Message> {
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
pub(crate) fn key_glyph(key: KeyCellProjection, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
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
pub(crate) fn reserved_glyph(dims: Dimensions) -> Element<'static, Message> {
    Space::new()
        .width(Length::Fixed(dims.inspector_glyph_width))
        .height(Length::Fixed(glyph_height(dims)))
        .into()
}

pub(crate) fn glyph_button_style(
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
pub(crate) enum ValueBoxStatus {
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
pub(crate) fn value_box_style(dims: Dimensions, colors: Colors, status: ValueBoxStatus) -> container::Style {
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
pub(crate) fn flat_button_style(colors: Colors, status: button::Status) -> button::Style {
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
pub(crate) fn name_input_style(
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

/// mock の hint 行。**「Drag to scrub」は実装済みなので復活させる**
/// (drag-to-scrub、利用者依頼)。「double-click to type」も実際の挙動と違う —
/// この実装の値セルは、動かさず release すれば単クリックで打鍵できる
/// (二度打ちは要らない)ので「click」へ言い換える(M13: 実装と違う手順を
/// 案内しない)。「Esc to cancel」も drag の復元・打鍵下書きの破棄の両方で
/// 今回初めて本当に効く(`motolii_shell::Shell::cancel_inspector_interaction`)。
pub(crate) fn hint_row(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
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

