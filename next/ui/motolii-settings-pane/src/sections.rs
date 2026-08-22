//! B12 第1切片(環境設定束、2026-08-22 発注): Settings 窓の中身拡充 —
//! section 分け(COMPOSITION / APPEARANCE / PLAYBACK)+ Composition 節の
//! W/H/FPS/尺 編集 + プレビューキャッシュの read-only 表示。
//!
//! ## 選定原則(発注書「顔だけの設定」禁止)
//! **意味が既に存在する状態にだけ顔を作る**。載っている物と紐づく実在識別子:
//!
//! - **COMPOSITION**: `motolii_store::Composition::{width, height, fps,
//!   duration_frames, background}`。書き口は既存の `Intent::SetComposition`
//!   read-modify-write(背景色 [`crate::commit_background_channel`] と同型)。
//! - **APPEARANCE**: `motolii_tokens_rs::Tokens::ui_scale`(既存項目の移設)。
//! - **PLAYBACK**: `motolii_engine::Engine::cached_frame_count()` /
//!   `Engine::FRAME_CACHE_LIMIT` の**表示のみ**(read-only)。変更口は
//!   キャッシュ束が来るまで作らない(発注書)。値は supervisor が
//!   [`ViewModel::preview_cache`] へ注入する — この crate に `motolii-engine`
//!   (GPU 系)依存を増やさないための注入形。**未結線(`None`)の間は行ごと
//!   出さない** — 空の数字を飾らない(飾り禁止の徹底)。
//!
//! theme 既定・再生ループ既定・スナップ既定・ゼブラ・自動保存間隔などの候補は
//! **紐づく実装状態がまだ無い**(スナップは gesture ごとの修飾キー派生
//! `!modifiers.command()` であって保存された既定値ではない、等)ため載せて
//! いない — 意味が生まれた束で追加する。
//!
//! ## なぜ `Message` がもう1本あるのか(結線互換の縫い目)
//! root(`motolii-shell::Shell::update_settings`)は旧 [`crate::Message`] を
//! **wildcard 無しで網羅 match**している。旧 enum へ腕を足すと shell が即
//! コンパイル不能になるが、shell 結線はこの発注の write-set 外(supervisor
//! 担当)。そこで新項目の Message はこの module に住み、旧腕は
//! [`Message::Legacy`] で丸ごと包む。supervisor の結線手順:
//!
//! 1. Settings 窓の view 呼び出しを [`view`](self::view) へ差し替える
//!    (`ViewModel` は `Shell` の既存フィールド+ `comp_draft: Option<
//!    CompFieldDraft>`(新設)+ `engine.cached_frame_count()` で組める)。
//! 2. match を `Legacy(m) => 既存 update_settings 本体` + `CompFieldInput` /
//!    `CompFieldSubmit` の2腕で書く(submit は [`commit_comp_field`] を呼ぶ
//!    だけ — `commit_background_channel` の呼び形と同じ)。
//! 3. その時点で旧 [`crate::view`] は撤去して良い(この節も一緒に畳む)。

use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use motolii_store::{Composition, Document, Fps, Intent};
use motolii_tokens_rs::{Colors, Dimensions};

use crate::chrome::{self, section_header, value_input_style};
use crate::{
    background_row, hairline_bottom, hint_row, preset_row, ui_scale_row, BackgroundFieldDraft,
};

// ---------------------------------------------------------------------------
// Message(module ローカル — 冒頭 doc「結線互換の縫い目」参照)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// 旧 [`crate::Message`] の全腕をそのまま包む(背景色・プリセット・
    /// ui_scale・歯車トグル)。挙動は旧 enum と1bit も変えない — 結線側は
    /// これを既存の `update_settings` 本体へ剥がして流すだけ。
    Legacy(crate::Message),
    /// Composition 数値欄(W/H/FPS/尺)への打鍵。**まだ Document を書かない**
    /// — 下書きを更新するだけ([`crate::Message::BackgroundChannelInput`] と
    /// 同じ形)。
    CompFieldInput(CompField, String),
    /// Composition 数値欄の Enter — ここで初めて `Intent::SetComposition` を
    /// 1回出す([`commit_comp_field`]、read-modify-write)。
    CompFieldSubmit(CompField),
}

// ---------------------------------------------------------------------------
// Composition 節(W/H/FPS/尺)
// ---------------------------------------------------------------------------

/// `Composition` の編集対象フィールド。`background` だけは既存の
/// [`crate::BackgroundChannel`] 経路が持つので、ここには含めない
/// (同じフィールドに書き口を2本作らない)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompField {
    /// `Composition::width`(px)
    Width,
    /// `Composition::height`(px)
    Height,
    /// `Composition::fps`(`motolii_core::Fps`、有理数)
    Fps,
    /// `Composition::duration_frames`(フレーム数、半開 `[0, n)`)
    DurationFrames,
}

impl CompField {
    /// 「全項目が実在識別子に紐づく」テストが回すための全列挙
    /// (`tests` の意味実在柵参照)。
    pub const ALL: [CompField; 4] = [
        CompField::Width,
        CompField::Height,
        CompField::Fps,
        CompField::DurationFrames,
    ];

    fn caption(self) -> &'static str {
        match self {
            Self::Width => "W",
            Self::Height => "H",
            Self::Fps => "FPS",
            Self::DurationFrames => "Frames",
        }
    }
}

/// Composition 数値欄、編集中の下書き。**Document ではない**
/// ([`BackgroundFieldDraft`] と同じ形 — Enter まで store に触らない)。
/// 置き場は `Shell`(supervisor 結線時に `background_draft` の隣へ新設)。
#[derive(Clone, Debug, PartialEq)]
pub struct CompFieldDraft {
    pub field: CompField,
    pub text: String,
}

/// comp 解像度の上限(px)。wgpu 既定の `max_texture_dimension_2d`(8192)に
/// 合わせた防波堤 — これを超える comp は現行 engine の1枚 texture 経路で
/// そもそも作れない。下限1は「0px の comp」を型より手前で拒むだけ。
pub const MAX_COMP_DIMENSION_PX: u32 = 8192;

/// fps 入力の上限。実用高 fps(240)まで — `Fps` 自体は正値なら何でも持てるが、
/// 打ち間違いの桁暴れ(3000fps 等)をここで吸収する(ui_scale の 50..200
/// クランプと同じ「入力欄側の頬」)。
pub const MAX_COMP_FPS: f64 = 240.0;

/// 尺(フレーム数)の上限。30fps 換算で約385日 — `RationalTime` の overflow
/// 防波堤で、実運用では届かない値。
pub const MAX_COMP_DURATION_FRAMES: i64 = 1_000_000_000;

/// 解像度欄の文字列 → 1..=[`MAX_COMP_DIMENSION_PX`] にクランプした px。
/// 読めなければ `None`([`commit_comp_field`] が status 帯へ理由を出す)。
pub fn parse_comp_dimension(text: &str) -> Option<u32> {
    chrome::parse_number(text)
        .map(|value| value.round().clamp(1.0, f64::from(MAX_COMP_DIMENSION_PX)) as u32)
}

/// fps 欄の文字列 → 1..=[`MAX_COMP_FPS`] にクランプ、1/1000 刻みで有理数化した
/// [`Fps`]。`29.97` → `2997/100`(`Fps::try_new` が既約化する)。NTSC 系の
/// 厳密値(`30000/1001`)を打ちたい場合は現状 `29.97` の10進近似になる —
/// 分数直接入力は意味(表示/入力文法)が増えるので、この切片では10進のみ。
pub fn parse_comp_fps(text: &str) -> Option<Fps> {
    let value = chrome::parse_number(text)?;
    let clamped = value.clamp(1.0, MAX_COMP_FPS);
    let per_mille = (clamped * 1000.0).round() as i64;
    Fps::try_new(per_mille, 1000).ok()
}

/// 尺欄の文字列 → 1..=[`MAX_COMP_DURATION_FRAMES`] にクランプしたフレーム数。
pub fn parse_comp_duration(text: &str) -> Option<i64> {
    chrome::parse_number(text)
        .map(|value| value.round().clamp(1.0, MAX_COMP_DURATION_FRAMES as f64) as i64)
}

/// [`Fps`] の表示文字列。整数 fps は素の整数(`30`)、非整数は小数3桁から
/// 末尾ゼロを落とす(`2997/100` → `29.97`、`24000/1001` → `23.976`)。
/// [`parse_comp_fps`] と往復しても値が動かない(表示→入力→確定で無編集なら
/// no-op)ことは tests が固定する。
pub fn format_fps(fps: Fps) -> String {
    if fps.den() == 1 {
        return fps.num().to_string();
    }
    format!("{:.3}", fps.as_f64())
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

/// フィールドの現在値の表示文字列(下書きが無い時に欄へ出す値)。
pub fn comp_field_display(field: CompField, composition: &Composition) -> String {
    match field {
        CompField::Width => composition.width.to_string(),
        CompField::Height => composition.height.to_string(),
        CompField::Fps => format_fps(composition.fps),
        CompField::DurationFrames => composition.duration_frames.to_string(),
    }
}

/// Composition の1フィールド — 下書きを確定して1回の `Intent::SetComposition`
/// を出す(read-modify-write、他フィールドは今の値のまま)。
/// [`crate::commit_background_channel`] と完全に同じ形 — `Err` は status 帯へ
/// 出す理由文そのもの(呼び出し側が `self.status` へ渡す)。
pub fn commit_comp_field(
    doc: &mut Document,
    draft: &mut Option<CompFieldDraft>,
    field: CompField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.field != field {
        *draft = Some(taken);
        return Ok(());
    }
    let Some(mut composition) = doc.view().composition().ok().flatten() else {
        return Err("comp が無い".to_owned());
    };
    match field {
        CompField::Width => {
            let Some(px) = parse_comp_dimension(&taken.text) else {
                return Err(format!("数値として読めない: {}", taken.text));
            };
            composition.width = px;
        }
        CompField::Height => {
            let Some(px) = parse_comp_dimension(&taken.text) else {
                return Err(format!("数値として読めない: {}", taken.text));
            };
            composition.height = px;
        }
        CompField::Fps => {
            let Some(fps) = parse_comp_fps(&taken.text) else {
                return Err(format!("数値として読めない: {}", taken.text));
            };
            composition.fps = fps;
        }
        CompField::DurationFrames => {
            let Some(frames) = parse_comp_duration(&taken.text) else {
                return Err(format!("数値として読めない: {}", taken.text));
            };
            composition.duration_frames = frames;
        }
    }
    doc.apply(Intent::SetComposition(composition))
        .map_err(|error| format!("comp を書けない: {error}"))
}

// ---------------------------------------------------------------------------
// Playback 節(プレビューキャッシュ表示 — read-only)
// ---------------------------------------------------------------------------

/// engine のフレームキャッシュ実測値。supervisor が
/// `motolii_engine::Engine::cached_frame_count()` /
/// `Engine::FRAME_CACHE_LIMIT` から組んで渡す(冒頭 doc「注入形」参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewCacheStats {
    /// `Engine::cached_frame_count()` — 今抱えている media フレーム数。
    pub held_frames: usize,
    /// `Engine::FRAME_CACHE_LIMIT` — LRU の上限枚数。
    pub limit: usize,
}

// ---------------------------------------------------------------------------
// view
// ---------------------------------------------------------------------------

/// [`view`] の入力一式。旧 [`crate::view`] の引数列+結線待ちの新項目。
/// `Shell` の既存フィールドをそのまま貸す形で組める(冒頭 doc の結線手順)。
#[derive(Clone, Copy, Debug)]
pub struct ViewModel<'a> {
    pub composition: Option<&'a Composition>,
    pub background_draft: Option<&'a BackgroundFieldDraft>,
    /// Composition 数値欄(W/H/FPS/尺)の下書き。`Shell` に新設
    /// (`background_draft` の隣、同じ「確定するまで front だけが持つ」身分)。
    pub comp_draft: Option<&'a CompFieldDraft>,
    pub ui_scale: f32,
    pub ui_scale_draft: Option<&'a str>,
    /// `None` = 未結線。その間 PLAYBACK 節は行ごと出さない(飾り禁止)。
    pub preview_cache: Option<PreviewCacheStats>,
}

/// B12 第1切片の view(旧 [`crate::view`] の後継 — 結線されたら旧関数は撤去、
/// 冒頭 doc 参照)。意匠は既存 settings の文法(D5 済み)のまま:
/// 見出し帯 = [`section_header`]、数値欄 = [`value_input_style`]、行区切り =
/// 行高+padding の間隔(裁定179 文法1)。**行が増えても破綻しないよう本体は
/// `scrollable`**(browser/inspector と同じ `height(Fill)` の形)。
pub fn view(model: ViewModel<'_>, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let body: Element<'static, Message> = match model.composition {
        None => container(
            text("comp が無い — 設定を編集できない")
                .size(dims.caption_text)
                .color(colors.text_muted),
        )
        .padding([0.0, dims.spacing_m])
        .into(),
        Some(composition) => {
            let mut rows: Vec<Element<'static, Message>> = vec![
                // COMPOSITION: 実体は Composition そのもの(Document、undo が
                // 効く)。背景色行は旧実装の行をそのまま Legacy へ写す。
                section_header("COMPOSITION", dims, colors),
                hairline_bottom(
                    comp_cells_row(
                        "Size (px)",
                        &[CompField::Width, CompField::Height],
                        composition,
                        model.comp_draft,
                        dims,
                        colors,
                    ),
                    dims,
                    colors,
                ),
                hairline_bottom(
                    comp_cells_row(
                        "Time",
                        &[CompField::Fps, CompField::DurationFrames],
                        composition,
                        model.comp_draft,
                        dims,
                        colors,
                    ),
                    dims,
                    colors,
                ),
                hairline_bottom(
                    background_row(composition, model.background_draft, dims, colors)
                        .map(Message::Legacy),
                    dims,
                    colors,
                ),
                hairline_bottom(preset_row(dims, colors).map(Message::Legacy), dims, colors),
                hairline_bottom(
                    hint_row("書き出しにもこの背景色が乗ります", dims, colors).map(Message::Legacy),
                    dims,
                    colors,
                ),
                // APPEARANCE: Tokens::ui_scale(既存項目の移設)。
                section_header("APPEARANCE", dims, colors),
                hairline_bottom(
                    ui_scale_row(model.ui_scale, model.ui_scale_draft, dims, colors)
                        .map(Message::Legacy),
                    dims,
                    colors,
                ),
            ];
            // PLAYBACK: 実測値が注入された時だけ現れる(未結線の飾り禁止)。
            if let Some(stats) = model.preview_cache {
                rows.push(section_header("PLAYBACK", dims, colors));
                rows.push(hairline_bottom(preview_cache_row(stats, dims, colors), dims, colors));
            }
            column(rows).into()
        }
    };

    container(column![
        section_header("SETTINGS", dims, colors),
        scrollable(body).width(Length::Fill).height(Length::Fill),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_theme| chrome::panel_container_style(dims, colors))
    .into()
}

/// 「行ラベル左・数値セル右」の行([`background_row`] と同じ列グリッド文法)。
fn comp_cells_row(
    label: &'static str,
    fields: &[CompField],
    composition: &Composition,
    draft: Option<&CompFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let cells: Vec<Element<'static, Message>> = fields
        .iter()
        .map(|&field| comp_field_cell(field, composition, draft, dims, colors))
        .collect();

    row![
        text(label)
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        row(cells).spacing(dims.spacing_xs),
    ]
    .spacing(dims.spacing_xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m])
    .into()
}

/// caption + 数値欄のセル([`crate::view`] の `channel_cell` と同じ形 —
/// 2箇所で別の意匠を発明しない)。
fn comp_field_cell(
    field: CompField,
    composition: &Composition,
    draft: Option<&CompFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|draft| draft.field == field)
        .map(|draft| draft.text.clone())
        .unwrap_or_else(|| comp_field_display(field, composition));

    column![
        text(field.caption())
            .size(dims.caption_text)
            .color(colors.text_muted)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Fixed(dims.inspector_value_width)),
        // 裁定170 M01: fork の `text_input` は Fragment::Borrowed で借用寿命を
        // 縛るため、`'static` 返却には owned move が要る(`channel_cell` と同じ)。
        text_input("", displayed)
            .on_input(move |text| Message::CompFieldInput(field, text))
            .on_submit(Message::CompFieldSubmit(field))
            .size(dims.body_text)
            .width(Length::Fixed(dims.inspector_value_width))
            .padding(0.0)
            .align_x(iced::alignment::Horizontal::Center)
            .style(move |_theme, status| value_input_style(dims, colors, status)),
    ]
    .spacing(0.0)
    .into()
}

/// プレビューキャッシュ行。**read-only** — 押せる顔をしない(text のみ、
/// Q0「触れそうで触れない物は不合格」の逆側: 触れないなら触れなさそうに)。
fn preview_cache_row(
    stats: PreviewCacheStats,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    row![
        text("Preview cache (frames)")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        text(format!("{} / {}", stats.held_frames, stats.limit))
            .size(dims.body_text)
            .color(colors.text_primary),
    ]
    .spacing(dims.spacing_xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m])
    .into()
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn doc_with_comp() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 360,
            fps: Fps::try_new(30, 1).expect("30fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .expect("fixture comp を書ける");
        doc
    }

    fn comp_of(doc: &Document) -> Composition {
        doc.view()
            .composition()
            .expect("comp を読める")
            .expect("comp が在る")
    }

    // -----------------------------------------------------------------
    // 意味実在柵(発注書「意味の無い設定が1つも無い」): 全 CompField が
    // Composition の実在フィールドを実際に動かす。
    // -----------------------------------------------------------------

    /// **本命**: [`CompField::ALL`] を尽くし、各 commit が対応フィールド
    /// **だけ**を動かすこと(= どの欄も実在識別子に紐づき、かつ他の意味を
    /// 巻き込まない)を1つずつ確かめる。
    #[test]
    fn every_comp_field_commits_into_its_real_composition_field_and_nothing_else() {
        for field in CompField::ALL {
            let mut doc = doc_with_comp();
            let before = comp_of(&doc);
            let text = match field {
                CompField::Width => "1024",
                CompField::Height => "768",
                CompField::Fps => "24",
                CompField::DurationFrames => "120",
            };
            let mut draft = Some(CompFieldDraft {
                field,
                text: text.to_owned(),
            });
            commit_comp_field(&mut doc, &mut draft, field)
                .unwrap_or_else(|error| panic!("{field:?} を書けない: {error}"));
            assert!(draft.is_none(), "{field:?}: 確定後も下書きが残っている");

            let after = comp_of(&doc);
            let expected_width = if field == CompField::Width { 1024 } else { before.width };
            let expected_height = if field == CompField::Height { 768 } else { before.height };
            let expected_fps = if field == CompField::Fps {
                Fps::try_new(24, 1).expect("24fps")
            } else {
                before.fps
            };
            let expected_duration = if field == CompField::DurationFrames {
                120
            } else {
                before.duration_frames
            };
            assert_eq!(after.width, expected_width, "{field:?} 確定後の width");
            assert_eq!(after.height, expected_height, "{field:?} 確定後の height");
            assert_eq!(after.fps, expected_fps, "{field:?} 確定後の fps");
            assert_eq!(
                after.duration_frames, expected_duration,
                "{field:?} 確定後の duration_frames"
            );
            assert_eq!(
                after.background, before.background,
                "{field:?} が背景色を巻き込んだ(read-modify-write 違反)"
            );
        }
    }

    /// 別フィールドの下書きしか無い時の Enter は no-op で、下書きも消さない
    /// ([`crate::commit_background_channel`] と同じ意味論)。
    #[test]
    fn commit_with_a_mismatched_draft_is_a_no_op_and_keeps_the_draft() {
        let mut doc = doc_with_comp();
        let before = comp_of(&doc);
        let mut draft = Some(CompFieldDraft {
            field: CompField::Width,
            text: "1024".to_owned(),
        });
        commit_comp_field(&mut doc, &mut draft, CompField::Height).expect("no-op のはず");
        assert!(draft.is_some(), "無関係の Enter が下書きを消した");
        let after = comp_of(&doc);
        assert_eq!(after.width, before.width);
        assert_eq!(after.height, before.height);
    }

    /// 読めない入力は Err(status 帯行き)で Document には触らない。
    #[test]
    fn commit_rejects_garbage_without_touching_the_document() {
        let mut doc = doc_with_comp();
        let before = comp_of(&doc);
        let mut draft = Some(CompFieldDraft {
            field: CompField::Fps,
            text: "not a number".to_owned(),
        });
        let error = commit_comp_field(&mut doc, &mut draft, CompField::Fps)
            .expect_err("読めない入力が通ってしまった");
        assert!(
            error.contains("not a number"),
            "理由文に入力が引用されていない: {error}"
        );
        let after = comp_of(&doc);
        assert_eq!(after.fps, before.fps);
    }

    // -----------------------------------------------------------------
    // parse / format
    // -----------------------------------------------------------------

    #[test]
    fn parse_comp_dimension_clamps_into_1_to_max() {
        assert_eq!(parse_comp_dimension("0"), Some(1));
        assert_eq!(parse_comp_dimension("-100"), Some(1));
        assert_eq!(parse_comp_dimension("99999"), Some(MAX_COMP_DIMENSION_PX));
        assert_eq!(parse_comp_dimension("1920"), Some(1920));
        assert_eq!(parse_comp_dimension("1919.6"), Some(1920), "px は整数へ丸める");
        assert_eq!(parse_comp_dimension("not a number"), None);
    }

    #[test]
    fn parse_comp_fps_builds_a_reduced_rational_and_clamps() {
        assert_eq!(parse_comp_fps("30"), Some(Fps::try_new(30, 1).expect("30fps")));
        assert_eq!(
            parse_comp_fps("29.97"),
            Some(Fps::try_new(2997, 100).expect("29.97fps")),
            "10進 fps は 1/1000 刻みの有理数として既約化されるはず"
        );
        assert_eq!(parse_comp_fps("0"), Some(Fps::try_new(1, 1).expect("1fps")), "下限1へクランプ");
        assert_eq!(
            parse_comp_fps("3000"),
            Some(Fps::try_new(240, 1).expect("240fps")),
            "上限240へクランプ"
        );
        assert_eq!(parse_comp_fps("not a number"), None);
    }

    #[test]
    fn parse_comp_duration_clamps_into_1_to_max() {
        assert_eq!(parse_comp_duration("300"), Some(300));
        assert_eq!(parse_comp_duration("0"), Some(1));
        assert_eq!(parse_comp_duration("-5"), Some(1));
        assert_eq!(parse_comp_duration("1e18"), Some(MAX_COMP_DURATION_FRAMES));
        assert_eq!(parse_comp_duration("not a number"), None);
    }

    #[test]
    fn format_fps_shows_integers_bare_and_trims_decimal_zeros() {
        assert_eq!(format_fps(Fps::try_new(30, 1).expect("30fps")), "30");
        assert_eq!(format_fps(Fps::try_new(2997, 100).expect("29.97fps")), "29.97");
        assert_eq!(format_fps(Fps::try_new(24000, 1001).expect("23.976fps")), "23.976");
        assert_eq!(format_fps(Fps::try_new(5, 2).expect("2.5fps")), "2.5");
    }

    /// 表示→入力→確定の往復で値が動かない(欄を触らず Enter しても no-op に
    /// なる、数値欄の基本礼儀)。
    #[test]
    fn fps_display_round_trips_through_parse_unchanged() {
        for fps in [
            Fps::try_new(30, 1).expect("30"),
            Fps::try_new(2997, 100).expect("29.97"),
            Fps::try_new(24, 1).expect("24"),
        ] {
            assert_eq!(parse_comp_fps(&format_fps(fps)), Some(fps), "{fps:?} が往復で動いた");
        }
    }

    #[test]
    fn comp_field_display_reads_the_real_composition_values() {
        let composition = Composition {
            width: 1920,
            height: 1080,
            fps: Fps::try_new(2997, 100).expect("29.97fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        };
        assert_eq!(comp_field_display(CompField::Width, &composition), "1920");
        assert_eq!(comp_field_display(CompField::Height, &composition), "1080");
        assert_eq!(comp_field_display(CompField::Fps, &composition), "29.97");
        assert_eq!(comp_field_display(CompField::DurationFrames, &composition), "300");
    }
}
