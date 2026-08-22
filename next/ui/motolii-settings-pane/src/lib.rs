//! wraps: iced — Settings pane(背景/ui_scale)+pane 横断 chrome。書き込みは Intent 経由のみ。
//! Settings パネル(タスク#18: Stage 背景色・ui_scale)。ヘッダの歯車
//! ボタン(`Message::ToggleSettingsPanel`)でトグルする。
//!
//! **市松トグルは裁定163(S 空間スコア)で Stage 下縁状態帯へ引っ越した** —
//! `motolii_stage_pane::Message::ToggleCheckerboard`(`docs/ui-spatial-score.md`
//! 初期違反在庫「市松トグル: a型が Settings に同居」の根治)。この crate は
//! もう `Message::ToggleCheckerboard`/`checkerboard_row` を持たない —
//! [`composite_checkerboard`](実際の市松合成ロジック)自体は Settings の
//! UI 概念ではなく、`motolii-shell::build_stage_handle` が今も呼ぶ表示器具
//! なのでここに残る(意味の置き場と実装の置き場は別問題)。
//!
//! **Inspector と同じ列グリッド/tokens 流儀**(発注書)を踏襲する — 見出し帯は
//! `chrome::section_header` をそのまま再利用し、数値欄の枠色ロールは
//! `chrome::value_input_style` を、ボタンの意味色ロールは
//! `chrome::button_style` を共有する(2箇所で別の意匠を発明しない、裁定160
//! 切片5で `inspector_pane`/`lib.rs` から `chrome` へ吸い上げ済み。切片9で
//! `chrome` ごとこの crate へ抽出——`chrome.rs` 冒頭 doc 参照)。
//!
//! ## 裁定160 切片9: crate 抽出(`motolii-shell` → `motolii-settings-pane`)
//! `docs/reviews/2026-08-21-pane-split-survey.md` §6 切片9。**挙動ゼロ変更**。
//!
//! - **`Message` は pane ローカル**(この crate の [`Message`])。`motolii-shell`
//!   root の `Message::Settings(settings_pane::Message)` が1本で畳む(iced
//!   標準の「子 pane の Message を親が wrap する」形)。
//! - **書ける物を持たない、が書き口(自由関数)は持つ**: [`view`] は他 pane と
//!   同じく `&Composition`・下書き・`Dimensions`・`Colors` しか受け
//!   取らない。一方 [`apply_background_preset`]/[`commit_background_channel`]/
//!   [`commit_ui_scale`] は「pane が自分の write ロジックを持つ」形の自由関数
//!   ——`&mut Document`/`&mut Tokens`/下書きの `&mut Option<_>` を明示引数で
//!   受け取る(`&mut self` の暗黙アクセスではない、pane crate は `Shell` を
//!   持てないため)。呼び出し口は `motolii_shell::Shell::update_settings`
//!   (`&mut self.doc` 等をそのまま貸すだけの glue)。
//! - `Shell` 側の状態(`settings_panel_open`/`background_draft`/
//!   `ui_scale_draft`)は**移設していない** — pane split survey §1.2 の
//!   「Settings 小計 63行」は3関数の write ロジックだけを指しており、フィールド
//!   の置き場は対象外(struct 分割は判断要の別問題、このぶんは動かさない)。
//!   `checkerboard` フィールド自体は裁定163で Stage 下縁状態帯の状態へ
//!   意味が移ったが、置き場(`Shell::checkerboard`)は無改変(状態帯・旧
//!   Settings のどちらから触っても同じフィールドを書く)。
//!
//! ## B12 第1切片(2026-08-22)+第2切片(同日、AUTOSAVE): section 分けと
//! 新項目は [`sections`] module
//! Composition 節(W/H/FPS/尺)+ Playback 節(キャッシュ表示)+ Autosave 節
//! (間隔・世代数・有効/無効、AS レーンの `motolii_store::AutoSaveConfig` 着地で
//! 意味が実在するようになった)+
//! COMPOSITION/APPEARANCE/AUTOSAVE/PLAYBACK の section 構成は
//! [`sections::view`] が持つ(結線互換のため Message は module ローカルの
//! [`sections::Message`] — 経緯と supervisor の結線手順は `sections.rs` 冒頭
//! doc)。この [`view`] と [`Message`] は結線されるまでの現行経路で、結線後は
//! 撤去して良い。
//!
//! ## 2項目の置き場(発注書どおり、市松は裁定163で状態帯へ移設済み)
//! - **Stage 背景色**: `Composition.background` そのもの(Document、undo が効く)。
//!   [`apply_background_preset`]/[`commit_background_channel`] が
//!   read-modify-write で `Intent::SetComposition` を出す。
//! - **ui_scale(%)**: 既存 `motolii_tokens_rs::Tokens::ui_scale` をそのまま読み書きする
//!   — ここでは新しい置き場を作らない。書き戻しは [`motolii_tokens_rs::save_ui_scale`]。
//!
//! ## 市松 = AE型の透明可視化モード(裁定141)
//! 市松 ON の間、Stage プレビューは `Composition.background` を敷かずに合成した
//! 結果(`motolii_engine::Engine::render_frame_without_background`)へ市松を敷く
//! — 既定の不透明黒背景のままでもトグルは実際に画面を変える。Document・undo・
//! 書き出しは不変(プレビュー専用の入力差分、`motolii-engine` 側の裁定141 doc
//! 参照)。結線は `crate::Shell::refresh_frame`。旧仕様(「フレームの alpha<1 の
//! 画素にだけ乗るので不透明背景では無反応」)が実機で不合格になった経緯と、
//! その場しのぎだった `checkerboard_invisible_reason` の理由文が今回撤去された
//! 経緯は `next/DECISIONS.md` 裁定141 参照。[`BackgroundPreset::Transparent`] は
//! 背景を実際に透明で書き出したい人向けとして引き続き残す。

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use motolii_store::{Composition, Document, Intent};
use motolii_tokens_rs::{Colors, Dimensions, Tokens};

pub mod chrome;
pub mod sections;

use chrome::{button_style, section_header, value_input_style};

// ---------------------------------------------------------------------------
// pane ローカル Message(裁定160 切片9 — root `Message::Settings(Message)` が
// 1本で畳む)。旧腕名の "Settings" prefix は pane 名前空間で二重になるので
// 剥がした(`ToggleSettingsPanel`/`UiScaleInput`/`UiScaleSubmit` はもともと
// prefix が無いので無改名)。**`ToggleCheckerboard` は裁定163で
// `motolii_stage_pane::Message::ToggleCheckerboard` へ引っ越した**(crate doc
// 参照)——この enum にはもう無い。
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// ヘッダの歯車ボタン。表示だけのトグル — Document にも undo 履歴にも乗らない。
    ToggleSettingsPanel,
    /// 背景色プリセット(黒/白/グレー18%)。押した瞬間に確定する
    /// (`Intent::SetComposition` を1回、1 gesture = 1 undo)。
    BackgroundPreset(BackgroundPreset),
    /// 背景 RGBA の1チャンネルへの打鍵。**まだ Document を書かない** —
    /// 下書きを更新するだけ(Inspector の Transform 行入力と同じ形)。
    BackgroundChannelInput(BackgroundChannel, String),
    /// 背景 RGBA の1チャンネルの Enter — ここで初めて `Intent::SetComposition` を
    /// 1回出す(read-modify-write、他チャンネルは現在値のまま)。
    BackgroundChannelSubmit(BackgroundChannel),
    /// ui_scale(%)欄への打鍵。まだ書かない。
    UiScaleInput(String),
    /// ui_scale(%)欄の Enter — 50..200 にクランプして `Tokens`/`Dimensions` を
    /// 更新し、debug ビルドでは正本 JSON へも書き戻す(`save_ui_scale`)。
    UiScaleSubmit,
}

// ---------------------------------------------------------------------------
// 背景色
// ---------------------------------------------------------------------------

/// `Composition.background` の編集対象チャンネル。R/G/B/A の4本、常にどれか1つ
/// だけ編集中(`inspector_pane::FieldDraft` と同じ「同時に1つ」の形)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BackgroundChannel {
    R,
    G,
    B,
    A,
}

impl BackgroundChannel {
    /// `Composition::background`(`[f32; 4]`)内の添字。
    pub fn index(self) -> usize {
        match self {
            Self::R => 0,
            Self::G => 1,
            Self::B => 2,
            Self::A => 3,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::R => "R",
            Self::G => "G",
            Self::B => "B",
            Self::A => "A",
        }
    }
}

/// 背景 RGBA 数値欄、編集中の下書き。**Document ではない**
/// (`inspector_pane::FieldDraft` と同じ形 — Enter まで store に触らない)。
#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundFieldDraft {
    pub channel: BackgroundChannel,
    pub text: String,
}

/// プリセット4種。押した瞬間に確定する(下書きを経由しない、Inspector の
/// M glyph トグルと同じ「即時1 Intent」の形)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BackgroundPreset {
    Black,
    White,
    /// 写真の「18%グレー」という呼び名を 0..255 の8bit値へそのまま当てた値
    /// (46 ≈ 255*0.18)。反射率換算のガンマ補正はしない — store の既存 solid
    /// rgba(`fixture.rs::LayerSpec::rgba` 等)と同じ「素の8bit値」慣習を保つ。
    Gray18,
    /// alpha=0 の1クリック経路。背景そのものを実際に透明で書き出したい人向け
    /// (裁定141以降、市松を見るための必須経路ではない — 市松は不透明背景の
    /// ままでも `render_frame_without_background` 経由で見える)。
    Transparent,
}

/// プリセットの実値。**丸め込み先はここ1箇所だけ**
/// (`apply_background_preset` は呼ぶだけ)。
pub fn preset_rgba(preset: BackgroundPreset) -> [f32; 4] {
    let channel = |v: u8| v as f32 / 255.0;
    match preset {
        BackgroundPreset::Black => [channel(0), channel(0), channel(0), 1.0],
        BackgroundPreset::White => [channel(255), channel(255), channel(255), 1.0],
        BackgroundPreset::Gray18 => [channel(46), channel(46), channel(46), 1.0],
        BackgroundPreset::Transparent => [channel(0), channel(0), channel(0), 0.0],
    }
}

/// 数値入力欄の文字列 → 0..255 にクランプした値。読めなければ `None`
/// (`commit_background_channel` が status 帯へ理由を出す)。
pub fn parse_channel_u8(text: &str) -> Option<f32> {
    chrome::parse_number(text).map(|value| value.clamp(0.0, 255.0) as f32)
}

// ---------------------------------------------------------------------------
// ui_scale
// ---------------------------------------------------------------------------

/// 入力文字列(%) → 50..200 にクランプ・1%刻みに丸めた `ui_scale`(1.00基準)。
/// 読めなければ `None`。
pub fn parse_ui_scale_percent(text: &str) -> Option<f32> {
    chrome::parse_number(text).map(|value| {
        let clamped_percent = value.clamp(50.0, 200.0).round();
        (clamped_percent / 100.0) as f32
    })
}

// ---------------------------------------------------------------------------
// 書き口(裁定160 切片9で lib.rs から移設): `&mut Document`/`&mut Tokens`/
// 下書きを明示引数で受け取る自由関数。呼び出し側(`motolii_shell::Shell::
// update_settings`)が `self.doc` 等をそのまま貸す — pane crate は `Shell` を
// 持てない(root → pane の一方向依存、循環禁止)ための形。
// ---------------------------------------------------------------------------

/// 背景色プリセット — 現在の `Composition` を読み、`background` だけ書き換えて
/// 丸ごと書き戻す(read-modify-write、`Intent::SetComposition` は丸ごと置換の
/// intent なので width/height/fps/duration_frames を巻き込まないよう毎回読む)。
/// `Err` は status 帯へ出す理由文そのもの(呼び出し側が `self.status` へ渡す)。
pub fn apply_background_preset(doc: &mut Document, preset: BackgroundPreset) -> Result<(), String> {
    let Some(mut composition) = doc.view().composition().ok().flatten() else {
        return Err("comp が無い".to_owned());
    };
    composition.background = preset_rgba(preset);
    doc.apply(Intent::SetComposition(composition))
        .map_err(|error| format!("背景を書けない: {error}"))
}

/// 背景 RGBA の1チャンネル — 下書きを確定して1回の `Intent::SetComposition`
/// を出す(read-modify-write、他チャンネルは今の値のまま)。
pub fn commit_background_channel(
    doc: &mut Document,
    draft: &mut Option<BackgroundFieldDraft>,
    channel: BackgroundChannel,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.channel != channel {
        *draft = Some(taken);
        return Ok(());
    }
    let Some(mut composition) = doc.view().composition().ok().flatten() else {
        return Err("comp が無い".to_owned());
    };
    let Some(value_0_255) = parse_channel_u8(&taken.text) else {
        return Err(format!("数値として読めない: {}", taken.text));
    };
    composition.background[channel.index()] = value_0_255 / 255.0;
    doc.apply(Intent::SetComposition(composition))
        .map_err(|error| format!("背景を書けない: {error}"))
}

/// ui_scale(%)欄 — 下書きを確定して 50..200 にクランプ、`Tokens`/`Dimensions`
/// を更新する。**Document を経由しない**(`ui_scale` は既存の置き場どおり
/// トークン扱い、undo 対象ではない)。debug ビルドでは正本 JSON へも書き戻す —
/// 失敗しても in-memory の値は既に更新済みなので、画面上は反映される
/// (書き込み失敗は status 帯へ理由を出すだけで機能は止めない、M16)。
pub fn commit_ui_scale(tokens: &mut Tokens, draft: &mut Option<String>) -> Result<(), String> {
    let Some(text) = draft.take() else {
        return Ok(());
    };
    let Some(ui_scale) = parse_ui_scale_percent(&text) else {
        return Err(format!("数値として読めない: {text}"));
    };
    tokens.dims.ui_scale = ui_scale;
    tokens.ui_scale = ui_scale;
    motolii_tokens_rs::save_ui_scale(ui_scale).map_err(|error| format!("ui_scale を保存できない: {error}"))
}

// ---------------------------------------------------------------------------
// 市松(表示専用 — 書き出しに乗らない)
// ---------------------------------------------------------------------------

/// 市松タイル1マスの目標寸法(pt、`ui_scale=1`・縮小なしの表示空間基準)。
/// AE 実機準拠(市松v2、利用者較正 2026-08-21「市松が見えない」— 実測タイル
/// 約5px・2色 Δ8/255 が実質不可視だった根治)。旧 `CHECKERBOARD_TILE_PX`(8、
/// comp 画素空間固定・`ui_scale` を掛けない「物理px床」扱い)から置き換え —
/// 罫線と違い、市松タイルは視認合図なので他の UI 密度と同じく `ui_scale` に
/// 追随させる([`checkerboard_tile_px`] 参照)。
const CHECKERBOARD_TILE_TARGET_PT: f32 = 8.0;

/// [`CHECKERBOARD_TILE_TARGET_PT`] を実際のタイル px へ変換する純関数
/// (市松v2 ORACLE (b))。
///
/// - `ui_scale`: 他の UI 密度と同じ拡大率。市松タイルは視認合図であって
///   hairline のような「装飾は物理px床」ではないので、ここでは掛ける
///   (発注書 EXACT TARGET 2「表示上の目安 8pt 角(ui_scale 追随)」)。
/// - `effective_scale`: 合成対象バッファが comp 画素空間からどれだけ縮小
///   されているか(`motolii-shell::build_stage_handle` が Stage handle を
///   sync アップロード予算内へ縮める `stage_auto_scale`×解像度上限の合成値。
///   縮小されていなければ 1.0)。**タイルはこの逆数で膨らませる** —
///   縮小後の handle 内でタイルが小さくなっても、表示空間(画面に引き伸ばして
///   出す時点)では常に `CHECKERBOARD_TILE_TARGET_PT` に見えるようにする
///   補正(根因1「comp 画素空間固定のタイルが Auto 縮小後にさらに痩せる」の
///   直接の修正対象)。
///
/// `effective_scale <= 0.0` は呼び出し元([`stage::effective_preview_scale`]、
/// 常に正)からは来ない想定だが、防波堤として `1.0` 扱いにする(縮小なし)。
pub fn checkerboard_tile_px(ui_scale: f32, effective_scale: f64) -> u32 {
    let scale = if effective_scale > 0.0 { effective_scale } else { 1.0 };
    let target_pt = f64::from(CHECKERBOARD_TILE_TARGET_PT * ui_scale.max(0.0));
    (target_pt / scale).round().max(1.0) as u32
}

/// Stage 画像の下に敷く市松。**フレームの実 rgba(engine の premultiplied alpha
/// 出力そのもの、`../reference/KNOWN.md`「実 alpha が乗る」)を上書きする** —
/// 呼び出し側(`crate::Shell::refresh_frame`)は screenshot/export 用の生 rgba
/// (`RenderedFrame::rgba`)には触らせず、表示用 Handle の複製にだけ適用する
/// (「合成器が出せる」と「書き出しが吐く」は別問題)。
///
/// 完全不透明(alpha=255)の画素はそのまま — 市松は「透明の可視化」だけが仕事。
/// タイル色は `Colors::checkerboard_light`/`Colors::checkerboard_dark`(市松
/// 専用ロール、市松v2で `surface_raised`/`surface_panel` の借用から独立させた
/// — `Colors::checkerboard_light` doc 参照)。
///
/// **`pub`、シグネチャ不変**: `screenshot.rs`(`motolii-shell` root crate、
/// このレーンでは非改変対象)が「実際に画面へ出る絵」を再現するのに同じ
/// 呼び出しを使う(comp 画素空間のまま合成する経路なので `effective_scale`
/// 補正は不要 — この関数を経由するだけで新ロールの色を自動的に受け取る)。
/// `tests/settings_drive.rs` も容疑2(このロジック自体のバグ)を
/// `frame_rgba()` 生値に対して直接固定するのにこの4引数版を使う。
/// スケール補正込みのタイル寸を渡したい呼び出し側(`motolii-shell::
/// build_stage_handle`)は [`composite_checkerboard_with_tile_px`] を使う。
pub fn composite_checkerboard(width: u32, height: u32, rgba: &mut [u8], colors: Colors) {
    composite_checkerboard_with_tile_px(width, height, rgba, colors, checkerboard_tile_px(1.0, 1.0));
}

/// [`composite_checkerboard`] のタイル寸指定版。実体はこちら — 呼び出し側が
/// [`checkerboard_tile_px`] で算出した(または規定の)handle 空間タイル px を
/// 明示的に渡す。
pub fn composite_checkerboard_with_tile_px(
    width: u32,
    height: u32,
    rgba: &mut [u8],
    colors: Colors,
    tile_px: u32,
) {
    if width == 0 || height == 0 {
        return;
    }
    let tile_px = tile_px.max(1);
    let light = color_u8(colors.checkerboard_light);
    let dark = color_u8(colors.checkerboard_dark);

    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            if i + 3 >= rgba.len() {
                continue;
            }
            let alpha = rgba[i + 3];
            if alpha == 255 {
                continue;
            }
            let tile = if (x / tile_px + y / tile_px).is_multiple_of(2) {
                light
            } else {
                dark
            };
            let inv_alpha = 255u32 - alpha as u32;
            for c in 0..3 {
                // premultiplied alpha の over 演算: src はすでに alpha 込みなので、
                // dst(市松タイル、不透明)側だけ `(1-alpha)` を掛けて足す
                // (`engine/motolii-compositor` の `Premultiplied` 経路と同じ式)。
                let src = rgba[i + c] as u32;
                let dst = tile[c] as u32;
                rgba[i + c] = (src + (dst * inv_alpha) / 255).min(255) as u8;
            }
            rgba[i + 3] = 255; // 市松で埋めた画素は表示上は不透明になる。
        }
    }
}

fn color_u8(color: iced::Color) -> [u8; 3] {
    [
        (color.r * 255.0).round().clamp(0.0, 255.0) as u8,
        (color.g * 255.0).round().clamp(0.0, 255.0) as u8,
        (color.b * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

// ---------------------------------------------------------------------------
// view
// ---------------------------------------------------------------------------

pub fn view(
    composition: Option<&Composition>,
    background_draft: Option<&BackgroundFieldDraft>,
    ui_scale: f32,
    ui_scale_draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let body: Element<'static, Message> = match composition {
        None => container(
            text("comp が無い — 背景を編集できない")
                .size(dims.caption_text)
                .color(colors.text_muted),
        )
        .padding([0.0, dims.spacing_m])
        .into(),
        Some(composition) => {
            // 各行を hairline で区切る(裁定139「面色の塗り分けで区切っている
            // 残余を全部 hairline へ置換する」を Settings へ展開)。旧実装は
            // 行同士の区切りを何も持たず(面色でも線でもない、無地のまま
            // 積んでいた)、Inspector の `.prow` と同格の情報行として同じ
            // 弱い hairline ロール(`tokens::Colors` 経由)を延長する — 新しい
            // 視覚言語の発明ではない。**市松の行は裁定163で Stage 下縁状態帯
            // へ引っ越した**(crate doc 参照)— ここには残らない。
            let rows: Vec<Element<'static, Message>> = vec![
                hairline_bottom(background_row(composition, background_draft, dims, colors), dims, colors),
                hairline_bottom(preset_row(dims, colors), dims, colors),
                hairline_bottom(hint_row("書き出しにもこの背景色が乗ります", dims, colors), dims, colors),
                hairline_bottom(ui_scale_row(ui_scale, ui_scale_draft, dims, colors), dims, colors),
            ];
            column(rows).into()
        }
    };

    // 線化 D5(裁定179 文法1): 容器の輪郭線は廃止 — `surface_panel` の面が
    // app 地から明度1段浮くことが pane の輪郭(`chrome::panel_container_style`
    // doc 参照、透明 border で幅だけ残す=幾何不変)。
    container(column![section_header("SETTINGS", dims, colors), body])
        .width(Length::Fill)
        .style(move |_theme| chrome::panel_container_style(dims, colors))
        .into()
}

/// 行の器(旧「行の下 hairline」、裁定139)。線化 D5(裁定179 文法1)で
/// 罫線は透明化した — 行の区切りは線でなく行高+padding の**間隔**が担う
/// (参照3製品とも情報行を線で区切らない、`docs/reviews/
/// 2026-08-22-chrome-grammar-audit.md` 文法1)。透明 border で幅だけ残す
/// (幾何不変)。**`.width(Length::Fill)` をここで明示するのが本体**:
/// `content` 側(row!)が自分の幅を宣言していなくても、外側 container が
/// Fill を持てば行は pane 全幅に伸びる(Inspector の `header`/`hint_row` と
/// 同じ実修正パターン)。
/// `Message` generic は B12 第1切片で追加([`sections::view`] が
/// [`sections::Message`] の行にも同じ器を使うため — `chrome::section_header`
/// の generic 化と同じ理由・同じ形)。旧呼び出し(この file 内)は型推論で
/// 無改変のまま通る。
pub(crate) fn hairline_bottom<Message: 'static>(
    content: Element<'static, Message>,
    dims: Dimensions,
    _colors: Colors,
) -> Element<'static, Message> {
    container(content)
        .width(Length::Fill)
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

pub(crate) fn background_row(
    composition: &Composition,
    draft: Option<&BackgroundFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let channels = [
        BackgroundChannel::R,
        BackgroundChannel::G,
        BackgroundChannel::B,
        BackgroundChannel::A,
    ];
    let cells: Vec<Element<'static, Message>> = channels
        .into_iter()
        .map(|channel| channel_cell(channel, composition, draft, dims, colors))
        .collect();

    row![
        text("Background")
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

fn channel_cell(
    channel: BackgroundChannel,
    composition: &Composition,
    draft: Option<&BackgroundFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let current_u8 = (composition.background[channel.index()] * 255.0)
        .round()
        .clamp(0.0, 255.0) as u32;
    let displayed = draft
        .filter(|draft| draft.channel == channel)
        .map(|draft| draft.text.clone())
        .unwrap_or_else(|| current_u8.to_string());

    column![
        text(channel.label())
            .size(dims.caption_text)
            .color(colors.text_muted)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Fixed(dims.inspector_value_width)),
        // 裁定170 M01: fork(0.15.0-dev)の `text_input()` は `&str`/`&String`
        // を `Fragment::Borrowed` として受け、返り値のライフタイムを入力の
        // 借用に縛る。`displayed` はこの関数のローカルで `Element<'static, _>`
        // を返す必要があるため、owned のまま move する(値は同じ、clone 済みの
        // String を渡すだけで表示文字列は不変)。
        text_input("", displayed)
            .on_input(move |text| Message::BackgroundChannelInput(channel, text))
            .on_submit(Message::BackgroundChannelSubmit(channel))
            .size(dims.body_text)
            .width(Length::Fixed(dims.inspector_value_width))
            .padding(0.0)
            .align_x(iced::alignment::Horizontal::Center)
            .style(move |_theme, status| value_input_style(dims, colors, status)),
    ]
    .spacing(0.0)
    .into()
}

pub(crate) fn preset_row(dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    row![
        preset_button("Black", BackgroundPreset::Black, dims, colors),
        preset_button("White", BackgroundPreset::White, dims, colors),
        preset_button("Gray 18%", BackgroundPreset::Gray18, dims, colors),
        // 背景を実際に透明で書き出したい人向け(裁定141以降、市松を見るための
        // 必須経路ではない — `BackgroundPreset::Transparent` の doc 参照)。
        preset_button("Transparent", BackgroundPreset::Transparent, dims, colors),
    ]
    .spacing(dims.spacing_xs)
    .padding([0.0, dims.spacing_m])
    .into()
}

fn preset_button(
    label: &'static str,
    preset: BackgroundPreset,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    button(text(label).size(dims.caption_text))
        .on_press(Message::BackgroundPreset(preset))
        .style(move |_theme, status| button_style(dims, colors, status))
        .into()
}

// `checkerboard_row` は裁定163で撤去した(Stage 下縁状態帯へ引っ越し —
// crate 冒頭 doc・`motolii_stage_pane::state_band_view` 参照)。

pub(crate) fn ui_scale_row(
    ui_scale: f32,
    draft: Option<&str>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let current_percent = (ui_scale * 100.0).round() as i64;
    let displayed = draft
        .map(|text| text.to_owned())
        .unwrap_or_else(|| current_percent.to_string());

    row![
        text("UI Scale (%)")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        // 裁定170 M01: channel_cell と同じ理由(fork の text_input が
        // Fragment::Borrowed で借用寿命を縛るため、'static 返却には owned move
        // が要る)。
        text_input("", displayed)
            .on_input(Message::UiScaleInput)
            .on_submit(Message::UiScaleSubmit)
            .size(dims.body_text)
            .width(Length::Fixed(dims.inspector_value_width))
            .padding(0.0)
            .align_x(iced::alignment::Horizontal::Center)
            .style(move |_theme, status| value_input_style(dims, colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m])
    .into()
}

pub(crate) fn hint_row(
    message: &'static str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    container(
        text(message)
            .size(dims.caption_text)
            .color(colors.text_muted),
    )
    .padding([dims.spacing_xs, dims.spacing_m])
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_rgba_values_match_black_white_and_18_percent_gray() {
        assert_eq!(preset_rgba(BackgroundPreset::Black), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(preset_rgba(BackgroundPreset::White), [1.0, 1.0, 1.0, 1.0]);
        let gray = preset_rgba(BackgroundPreset::Gray18);
        assert!((gray[0] - 46.0 / 255.0).abs() < 0.0001, "Gray18 の値: {gray:?}");
        assert_eq!(gray[0], gray[1]);
        assert_eq!(gray[1], gray[2]);
        assert_eq!(gray[3], 1.0, "Black/White/Gray18 は常に不透明のはず");
    }

    /// 背景を実際に透明で書き出したい人向けのプリセット。他3プリセットと違い、
    /// これだけが alpha=0 を出す(裁定141以降、市松を見るための必須経路では
    /// ない — `render_frame_without_background` が不透明背景でも市松を見せる)。
    #[test]
    fn transparent_preset_is_fully_transparent() {
        assert_eq!(preset_rgba(BackgroundPreset::Transparent), [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn parse_channel_u8_clamps_into_0_255() {
        assert_eq!(parse_channel_u8("-10"), Some(0.0));
        assert_eq!(parse_channel_u8("999"), Some(255.0));
        assert_eq!(parse_channel_u8("128"), Some(128.0));
        assert_eq!(parse_channel_u8("not a number"), None);
    }

    #[test]
    fn parse_ui_scale_percent_clamps_into_50_to_200_and_rounds_to_whole_percent() {
        assert_eq!(parse_ui_scale_percent("100"), Some(1.0));
        assert_eq!(parse_ui_scale_percent("10"), Some(0.5), "50%未満は50%へクランプ");
        assert_eq!(parse_ui_scale_percent("500"), Some(2.0), "200%超えは200%へクランプ");
        assert_eq!(parse_ui_scale_percent("123.6"), Some(1.24), "1%刻みに丸まっていない");
        assert_eq!(parse_ui_scale_percent("not a number"), None);
    }

    /// **市松トグルの表示分岐、本命**: 不透明画素はそのまま・透明画素は市松タイル色
    /// そのものになる。単なる塗りつぶしと違い、この2分岐が両方とも実際に起きることを
    /// 見る(「表示専用トグルが実際に絵を分岐させる」ことの直接証拠)。
    #[test]
    fn composite_checkerboard_leaves_opaque_pixels_untouched_and_fills_transparent_ones_with_the_tile()
    {
        let colors = Colors::default();
        let mut rgba = vec![
            10, 20, 30, 255, // (0,0) 不透明 — 変わらないはず
            0, 0, 0, 0, // (1,0) 完全透明 — 市松タイル色そのものになるはず
        ];
        composite_checkerboard(2, 1, &mut rgba, colors);

        assert_eq!(&rgba[0..4], &[10, 20, 30, 255], "不透明画素が変わってしまった");

        // 市松v2(利用者較正 2026-08-21): タイル色はもう `surface_raised`/
        // `surface_panel`(パネル面ロール)の借用ではなく、専用ロール
        // `checkerboard_light`/`checkerboard_dark` を使う(`Colors::
        // checkerboard_light` doc 参照)。
        let light = color_u8(colors.checkerboard_light);
        let dark = color_u8(colors.checkerboard_dark);
        let got = [rgba[4], rgba[5], rgba[6]];
        assert!(
            got == light || got == dark,
            "透明画素が市松タイル色になっていない: {got:?}(light={light:?}, dark={dark:?})"
        );
        assert_eq!(rgba[7], 255, "市松で埋めた画素は不透明になるはず");
    }

    /// タイル境界([`checkerboard_tile_px`])で実際に色が切り替わること —
    /// 単なる塗りつぶしとの区別(「市松」を名乗るなら格子であるはず)。
    #[test]
    fn composite_checkerboard_alternates_tiles_across_the_grid() {
        let colors = Colors::default();
        let tile_px = checkerboard_tile_px(1.0, 1.0);
        let width = tile_px * 2;
        let mut rgba = vec![0u8; (width * 4) as usize];
        composite_checkerboard(width, 1, &mut rgba, colors);

        let pixel = |x: u32| -> [u8; 3] {
            let i = (x * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2]]
        };
        assert_ne!(
            pixel(0),
            pixel(tile_px),
            "タイル境界で市松の色が切り替わっていない — 単なる塗りつぶしになっている"
        );
    }

    #[test]
    fn composite_checkerboard_is_a_no_op_on_fully_opaque_frames() {
        let colors = Colors::default();
        let mut rgba = vec![1, 2, 3, 255, 4, 5, 6, 255];
        let original = rgba.clone();
        composite_checkerboard(2, 1, &mut rgba, colors);
        assert_eq!(rgba, original, "全画素不透明なら市松は何も変えないはず");
    }

    // -----------------------------------------------------------------
    // 市松v2 ORACLE (b): タイル寸は実効スケールで逆補正され、表示空間では
    // 常に約8pt(±1px)に見える。
    // -----------------------------------------------------------------

    /// 縮小なし(`effective_scale=1.0`)・`ui_scale=1.0` の基準では、handle
    /// 空間のタイル px がそのまま目標 pt(8)に一致する。
    #[test]
    fn checkerboard_tile_px_at_unit_scale_matches_the_target_pt() {
        assert_eq!(checkerboard_tile_px(1.0, 1.0), 8);
    }

    /// **本命**: Auto 縮小で handle が縮んでも(`effective_scale=0.5` —
    /// supervisor 実測の ~0.62 と同じ「1未満」領域)、`tile_px * effective_scale`
    /// が表示空間で約8pt(±1px)に戻る — 縮小前と同じ大きさに見える、が
    /// 発注書の要件そのもの。
    #[test]
    fn checkerboard_tile_px_compensates_for_handle_downscale() {
        for effective_scale in [0.5, 0.62, 0.25, 0.9] {
            let tile_px = checkerboard_tile_px(1.0, effective_scale);
            let displayed_pt = tile_px as f64 * effective_scale;
            assert!(
                (displayed_pt - f64::from(CHECKERBOARD_TILE_TARGET_PT)).abs() <= 1.0,
                "effective_scale={effective_scale}: tile_px={tile_px} → 表示 {displayed_pt}pt \
                 が目標 {CHECKERBOARD_TILE_TARGET_PT}pt(±1px)から外れている"
            );
        }
    }

    /// `ui_scale` にも追随する(発注書 EXACT TARGET 2「表示上の目安 8pt 角
    /// (ui_scale 追随)」)— 200% なら目標 pt 自体が2倍になる。
    #[test]
    fn checkerboard_tile_px_follows_ui_scale() {
        assert_eq!(checkerboard_tile_px(2.0, 1.0), 16);
        assert_eq!(checkerboard_tile_px(0.5, 1.0), 4);
    }

    /// 縮小率0以下(来ない想定の防波堤)は1.0扱い — panic もゼロ除算もしない。
    #[test]
    fn checkerboard_tile_px_treats_non_positive_effective_scale_as_unscaled() {
        assert_eq!(checkerboard_tile_px(1.0, 0.0), checkerboard_tile_px(1.0, 1.0));
        assert_eq!(checkerboard_tile_px(1.0, -1.0), checkerboard_tile_px(1.0, 1.0));
    }

    /// [`composite_checkerboard_with_tile_px`] が実際に渡した `tile_px` を
    /// 使うこと([`composite_checkerboard`](既定 tile_px)と別の値になる、
    /// つまり呼び出し側の補正が無視されずに効くことの直接証拠)。
    #[test]
    fn composite_checkerboard_with_tile_px_honors_the_caller_supplied_tile_size() {
        let colors = Colors::default();
        let tile_px = 4;
        let width = tile_px * 2;
        let mut rgba = vec![0u8; (width * 4) as usize];
        composite_checkerboard_with_tile_px(width, 1, &mut rgba, colors, tile_px);

        let pixel = |x: u32| -> [u8; 3] {
            let i = (x * 4) as usize;
            [rgba[i], rgba[i + 1], rgba[i + 2]]
        };
        assert_ne!(
            pixel(0),
            pixel(tile_px),
            "呼び出し側が渡した tile_px でタイル境界が切り替わっていない"
        );
    }
}
