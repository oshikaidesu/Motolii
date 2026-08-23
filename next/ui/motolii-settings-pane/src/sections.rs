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
//! - **AUTOSAVE**(第2切片、2026-08-22): `motolii_store::AutoSaveConfig
//!   {interval_secs, generations}`。AS レーンが `Document::auto_save` として
//!   store 側の機構を着地させ、前切片で「機構なし」として見送っていた行が
//!   実在するようになった(crate doc の第2切片 doc 参照)。書き口は
//!   [`commit_auto_save_field`](Composition と違い `Document`/`Intent` を
//!   経由しない — `AutoSaveConfig` は undo 対象の Document データではなく
//!   `ui_scale`/`Tokens` と同じ「Settings が直接持つ値」、`persist.rs` の
//!   `AutoSaveConfig` doc「Settings が読める形の置き場」参照)。
//!   有効/無効は `bool`(`Message::AutoSaveToggle`)— タイマー自体を shell が
//!   引くかどうかを決めるだけの shell-local フラグで、`ToggleSettingsPanel`
//!   と同じ「Document を経由しないが実際に効果を持つ」身分(結線は supervisor、
//!   下記 doc の結線手順参照)。
//!
//! theme 既定・再生ループ既定・スナップ既定・ゼブラなどの候補は
//! **紐づく実装状態がまだ無い**(スナップは gesture ごとの修飾キー派生
//! `!modifiers.command()` であって保存された既定値ではない、等)ため載せて
//! いない — 意味が生まれた束で追加する。
//!
//! ### map 再走査(第2切片、AS 着地後): 消化/見送りの再判定
//! B12 の「採用予定」59行を、AS 着地(`AutoSaveConfig`)を踏まえて再走査した。
//! **自動保存関連の4行だけが「意味が実在するようになった」**行で、今回
//! [`AutoSaveField`]/[`Message::AutoSaveToggle`] として消化する:
//! id 1157(Auto Save/Premiere)・1158(Auto-Save/Adobe公式)・1206(Project Save
//! and Load/BMD)・1231(Auto-Save…/After Effects) — いずれも「自動保存の
//! 間隔・世代数(・BMD分は追加でライブセーブ/バックアップも束ねた1行)」で、
//! 今回の interval_secs/generations/enabled の3項目でカバーされる範囲は消化、
//! BMD の「ライブセーブ」(編集の度に別経路で保存し続ける Auto-Save とは別機構)
//! は今回のスコープ外(紐づく実装が無い、見送りのまま)。
//! **他55行(audio hardware/keyboard shortcuts/appearance highlight color/
//! grid & guides/language/proxy/…)は AS の着地と無関係** — AS が足したのは
//! 自動保存機構だけで、他の環境設定カテゴリに対応する実装状態は今回の着地で
//! 何も増えていない。よってそれらは見送り台帳のまま(map 自体は不触 —
//! 発注書「台帳類 不触」、この doc は判断理由の記録)。
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
//!
//! ## 裁定183 taffy 転写 第1号(2026-08-22)
//! [`comp_cells_row`] の並べ方(flex/gap/justify)は `motolii-taffy::TaffyBox`
//! (CSS 宣言の字面から `taffy::Style` を組む機構、
//! `docs/reviews/2026-08-22-weblike-layout-survey.md` 案A)へ委譲した——
//! 「モックを手で写す」工程を機械検証可能な転写へ置換する最初の適用例。
//! 詳細・oracle の形は [`comp_cells_row`] 直前のコメントと
//! `tests/comp_cells_row_taffy_oracle.rs` 冒頭 doc 参照。他の行
//! (`background_row`/`ui_scale_row`/`preview_cache_row`)は今回のスコープ外
//! (発注書「部分適用でよい」— 型を確立するのが目的)。

use iced::widget::{column, container, mouse_area, row, scrollable, text, text_input, toggler};
use iced::{Element, Length};

use motolii_store::{AutoSaveConfig, Composition, Document, Fps, Intent};
use motolii_taffy::{style_from_css_decl, TaffyBox};
use motolii_tokens_rs::{Colors, Dimensions};

use crate::chrome::{self, section_header, toggler_style, value_input_style};
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
    /// AUTOSAVE 有効/無効トグル(裁定187 toggler)。**shell-local な bool**
    /// (`ToggleSettingsPanel` と同じ身分 — Document/undo を経由しないが、
    /// 結線後は実際に自動保存タイマーを引くかどうかを決める)。
    AutoSaveToggle(bool),
    /// 自動保存の数値欄(間隔[分]・世代数)への打鍵。**まだ `AutoSaveConfig` を
    /// 書かない** — 下書きを更新するだけ([`CompFieldInput`] と同じ形)。
    AutoSaveFieldInput(AutoSaveField, String),
    /// 自動保存の数値欄の Enter — ここで初めて [`AutoSaveConfig`] の対応
    /// フィールドを1つ更新する([`commit_auto_save_field`])。
    AutoSaveFieldSubmit(AutoSaveField),
    /// Composition 数値欄のキャプション press(裁定217 連続量 drag 化、E-5)。
    /// click か drag かはまだ未確定 — `motolii_shell::Shell::start_value_drag`
    /// が投影を読んで下書きの起点値を確定する(`inspector_pane::Message::
    /// ValuePressed` と同じ「press だけを own する」形、crate doc「A-2 実測」
    /// 参照)。move/release は Inspector と同じ window 全体購読
    /// (`inspector_pointer_event`)を共有する — この module は自己完結の
    /// pane-local `Message` しか持てないため、続きは shell 側の状態
    /// (`Shell::value_drag`)が持つ。
    CompFieldDragPressed(CompField),
    /// AUTOSAVE 数値欄のキャプション press。[`Message::CompFieldDragPressed`]
    /// と同じ形。
    AutoSaveFieldDragPressed(AutoSaveField),
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
// Autosave 節(第2切片、2026-08-22: AS レーンが `Document::auto_save`/
// `AutoSaveConfig` を着地させたことで意味が実在するようになった — crate 冒頭
// doc「AUTOSAVE」参照)。
// ---------------------------------------------------------------------------

/// `AutoSaveConfig` の編集対象フィールド。「有効/無効」は `bool` 単体
/// (下書きを経由しない即時トグル、[`Message::AutoSaveToggle`])なのでここには
/// 含めない — 数値の2フィールドだけがこの enum の対象(`CompField` が
/// `background` を含めないのと同じ理由)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AutoSaveField {
    /// `AutoSaveConfig::interval_secs`。**欄には分単位で出す**(AE `Auto-Save`
    /// の `Save Every __ Minutes` 慣習・[`format_auto_save_interval_minutes`]/
    /// [`parse_auto_save_interval_secs`] が秒との往復を担う)。
    IntervalMinutes,
    /// `AutoSaveConfig::generations`。
    Generations,
}

impl AutoSaveField {
    /// 「全項目が実在識別子に紐づく」テストが回すための全列挙。
    pub const ALL: [AutoSaveField; 2] = [Self::IntervalMinutes, Self::Generations];

    fn caption(self) -> &'static str {
        match self {
            Self::IntervalMinutes => "Min",
            Self::Generations => "Gens",
        }
    }
}

/// 自動保存数値欄、編集中の下書き。**`AutoSaveConfig` ではない**
/// ([`CompFieldDraft`] と同じ形 — Enter まで確定しない)。置き場は `Shell`
/// (supervisor 結線時に `comp_draft` の隣へ新設)。
#[derive(Clone, Debug, PartialEq)]
pub struct AutoSaveFieldDraft {
    pub field: AutoSaveField,
    pub text: String,
}

/// 間隔欄(分)の下限。0分連射は disk I/O を毎 tick 起こしかねないので、
/// 「間隔」を名乗れる最小値として1分を床にする([`MAX_COMP_FPS`] 等と同じ
/// 「入力欄側の頬」)。
pub const MIN_AUTO_SAVE_INTERVAL_MINUTES: f64 = 1.0;
/// 間隔欄(分)の上限。24時間 — これを超える値は実質「自動保存しない」と
/// 変わらず、打ち間違いの桁暴れ(分のつもりで秒を打つ等)をここで吸収する。
pub const MAX_AUTO_SAVE_INTERVAL_MINUTES: f64 = 24.0 * 60.0;
/// 保持世代数欄の上限。AE 既定(5世代)の10倍で「多めに残したい」要求は
/// 十分吸収しつつ、桁暴れ(世代数のつもりで別の値を打つ)を防ぐ床。
pub const MAX_AUTO_SAVE_GENERATIONS: usize = 50;

/// 間隔欄(分)の文字列 → [`AutoSaveConfig::interval_secs`](秒、整数)。
/// [`MIN_AUTO_SAVE_INTERVAL_MINUTES`]..=[`MAX_AUTO_SAVE_INTERVAL_MINUTES`] に
/// クランプしてから60倍する。読めなければ `None`。
pub fn parse_auto_save_interval_secs(text: &str) -> Option<u64> {
    chrome::parse_number(text).map(|value| {
        let clamped_minutes =
            value.clamp(MIN_AUTO_SAVE_INTERVAL_MINUTES, MAX_AUTO_SAVE_INTERVAL_MINUTES);
        (clamped_minutes * 60.0).round() as u64
    })
}

/// [`AutoSaveConfig::interval_secs`] の表示文字列(分、末尾ゼロを落とす —
/// [`format_fps`] と同じ整形)。
pub fn format_auto_save_interval_minutes(interval_secs: u64) -> String {
    let minutes = interval_secs as f64 / 60.0;
    format!("{minutes:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

/// 保持世代数欄の文字列 → 1..=[`MAX_AUTO_SAVE_GENERATIONS`] にクランプした
/// [`AutoSaveConfig::generations`]。読めなければ `None`。
pub fn parse_auto_save_generations(text: &str) -> Option<usize> {
    chrome::parse_number(text)
        .map(|value| value.round().clamp(1.0, MAX_AUTO_SAVE_GENERATIONS as f64) as usize)
}

/// フィールドの現在値の表示文字列(下書きが無い時に欄へ出す値)。
pub fn auto_save_field_display(field: AutoSaveField, config: &AutoSaveConfig) -> String {
    match field {
        AutoSaveField::IntervalMinutes => format_auto_save_interval_minutes(config.interval_secs),
        AutoSaveField::Generations => config.generations.to_string(),
    }
}

/// `AutoSaveConfig` の1フィールド — 下書きを確定して対応フィールドを書き換える。
/// **`Document`/`Intent` を経由しない**([`crate::commit_ui_scale`] と同じ形 —
/// `AutoSaveConfig` は undo 対象の Document データではなく、Settings が直接
/// 持ち回して `Document::auto_save` へそのまま渡す値の置き場、
/// `motolii_store::persist::AutoSaveConfig` doc「Settings が読める形の置き場」
/// 参照)。`Err` は status 帯へ出す理由文そのもの。
pub fn commit_auto_save_field(
    config: &mut AutoSaveConfig,
    draft: &mut Option<AutoSaveFieldDraft>,
    field: AutoSaveField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.field != field {
        *draft = Some(taken);
        return Ok(());
    }
    match field {
        AutoSaveField::IntervalMinutes => {
            let Some(secs) = parse_auto_save_interval_secs(&taken.text) else {
                return Err(format!("数値として読めない: {}", taken.text));
            };
            config.interval_secs = secs;
        }
        AutoSaveField::Generations => {
            let Some(generations) = parse_auto_save_generations(&taken.text) else {
                return Err(format!("数値として読めない: {}", taken.text));
            };
            config.generations = generations;
        }
    }
    Ok(())
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
    /// AUTOSAVE 有効/無効(shell-local bool、`ToggleSettingsPanel` と同じ身分
    /// — crate 冒頭 doc「AUTOSAVE」参照)。`AutoSaveConfig` と違い Option で
    /// 包まない — `bool` は注入待ちの生成コストが無く(GPU 等の外部依存が無い)
    /// [`PreviewCacheStats`] のような「未結線なら隠す」防波堤が要らない
    /// (`Shell` 新設フィールドの既定値をそのまま渡せる、`comp_draft` と同じ
    /// 「結線待ちだが値自体は実在」の身分)。
    pub auto_save_enabled: bool,
    /// `motolii_store::AutoSaveConfig`(間隔・世代数)。既定値は
    /// `AutoSaveConfig::default()` — Shell が Document を持つかどうかとは
    /// 無関係に値として実在する(`Composition` と違い外部注入を要らない)。
    pub auto_save_config: AutoSaveConfig,
    /// 自動保存数値欄(間隔・世代数)の下書き。`Shell` に新設(`comp_draft` の
    /// 隣、同じ「確定するまで front だけが持つ」身分)。
    pub auto_save_draft: Option<&'a AutoSaveFieldDraft>,
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
                // AUTOSAVE(第2切片): AutoSaveConfig は外部注入を要らない生の値
                // なので PLAYBACK と違い Option 分岐を挟まない — 常に出す。
                section_header("AUTOSAVE", dims, colors),
                hairline_bottom(
                    auto_save_toggle_row(model.auto_save_enabled, dims, colors),
                    dims,
                    colors,
                ),
                hairline_bottom(
                    auto_save_cells_row(
                        "Save Every",
                        &AutoSaveField::ALL,
                        model.auto_save_config,
                        model.auto_save_draft,
                        dims,
                        colors,
                    ),
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

// ---------------------------------------------------------------------------
// taffy 転写 第1号(裁定183・weblike-layout 調査): [`comp_cells_row`] の
// 並べ方(flex/gap/justify)だけを `motolii-taffy::TaffyBox` へ委譲する。
// 寸法(gap/height/padding/セル幅)は引き続き `Dimensions`(tokens)が正本 —
// ここは CSS 宣言の字面としてそれを taffy へ渡すだけ。
//
// 旧実装は「label(`Length::Fill`) + 内側 `row(cells).spacing(xs)`」の2段
// 構造だったが、外側 `.spacing(dims.spacing_xs)` と内側
// `row(cells).spacing(dims.spacing_xs)` が同値だったため、1段の flex row
// (`gap` 一発)へフラット化しても見た目は不変 — label→セル0 の間隔・
// セル間の間隔がどちらも同じ `gap` になる(2フィールドの行しか無いこの
// section では区別が付かない=フラット化で失う情報が無い)。
//
// 正本はこの3つの CSS 宣言関数だけ — `tests/comp_cells_row_taffy_oracle.rs`
// が同じ関数を呼んで taffy 直接解と ±1px で突き合わせる(Settings 用の mock
// は `next/reference/mocks/` に無いため、分母は taffy 自身の直接計算 —
// `motolii-taffy/tests/layout_oracle.rs` と同じ手口)。文字列の二重管理を
// 避けるため、テスト側もこの関数を直接呼ぶ。
// ---------------------------------------------------------------------------

/// [`comp_cells_row`] の外枠(flex row)の CSS 宣言。
pub fn comp_cells_row_css(dims: Dimensions) -> String {
    format!(
        "display:flex; align-items:center; gap:{gap}px; height:{height}px; padding:0px {pad}px",
        gap = dims.spacing_xs,
        height = dims.inspector_row_height,
        pad = dims.spacing_m,
    )
}

/// label セル(旧 `.width(Length::Fill)`)の CSS — 残り幅を占める。
pub const COMP_CELLS_LABEL_CSS: &str = "flex:1";

/// 値セル(caption+text_input の列、[`comp_field_cell`])の CSS — 固定幅
/// (旧 `Length::Fixed(dims.inspector_value_width)`)。高さは指定しない —
/// 旧実装も明示 height を持たず、`column!` の自然高のまま `align-items:center`
/// (旧 `.align_y(Vertical::Center)`)で行高の中に中央寄せされていた。
pub fn comp_cell_css(dims: Dimensions) -> String {
    format!("width:{width}px", width = dims.inspector_value_width)
}

/// 「行ラベル左・数値セル右」の行([`background_row`] と同じ列グリッド文法)。
/// 並べ方は `TaffyBox`([`comp_cells_row_css`]/[`COMP_CELLS_LABEL_CSS`]/
/// [`comp_cell_css`])に委譲する — 冒頭の taffy 転写 doc 参照。
fn comp_cells_row(
    label: &'static str,
    fields: &[CompField],
    composition: &Composition,
    draft: Option<&CompFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let row_style = style_from_css_decl(&comp_cells_row_css(dims)).expect(
        "comp_cells_row_css は固定テンプレート+dims の px 値のみを埋める — 解釈は必ず成功する",
    );
    let label_style = style_from_css_decl(COMP_CELLS_LABEL_CSS)
        .expect("COMP_CELLS_LABEL_CSS は固定文字列 — 解釈は必ず成功する");
    let cell_style = style_from_css_decl(&comp_cell_css(dims))
        .expect("comp_cell_css は固定テンプレート+dims の px 値のみを埋める — 解釈は必ず成功する");

    let label_text: Element<'static, Message> = text(label)
        .size(dims.body_text)
        .color(colors.text_primary)
        .into();

    let mut taffy_row: TaffyBox<'static, Message, iced::Theme, iced::Renderer> =
        TaffyBox::new(row_style).push(label_style, label_text);
    for &field in fields {
        taffy_row = taffy_row.push(
            cell_style.clone(),
            comp_field_cell(field, composition, draft, dims, colors),
        );
    }
    taffy_row.into()
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
        // 裁定217 連続量 drag 化(E-5): キャプション自体を drag ハンドルにする
        // (AE のスクラブ精神論と同じ「パラメータ名の上を drag すると値が動く」
        // 慣習 — 値そのものより先にラベルへドラッグの当たり判定を足す先例、
        // RETURN 参照)。click か drag かは shell 側
        // (`motolii_shell::Shell::value_drag`)が判定する — ここは press だけ
        // own する(`inspector_pane::Message::ValuePressed` と同じ形)。
        // text_input 自体は無改変なので click→type 編集は従来どおり効く。
        mouse_area(
            text(field.caption())
                .size(dims.caption_text)
                .color(colors.text_muted)
                .align_x(iced::alignment::Horizontal::Center)
                .width(Length::Fixed(dims.inspector_value_width))
        )
        .interaction(iced::mouse::Interaction::ResizingHorizontally)
        .on_press(Message::CompFieldDragPressed(field)),
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

// ---------------------------------------------------------------------------
// taffy 転写 第2適用(第2切片): [`auto_save_cells_row`] は新しい CSS 宣言を
// 発明せず、[`comp_cells_row_css`]/[`COMP_CELLS_LABEL_CSS`]/[`comp_cell_css`]
// (第1号で確立済みの正本)をそのまま再利用する — 「行ラベル左・固定幅の値
// セルN個・右」という並べ方の CSS 宣言は Composition 節と AUTOSAVE 節の
// どちらでも同じ形なので、文字列を複製せず関数そのものを共有する
// (CSS 宣言が単一正本という裁定183の趣旨そのもの)。
// ---------------------------------------------------------------------------

/// AUTOSAVE 有効/無効の行(裁定187 toggler、[`crate::toggle_row`] に相当する
/// 概念だが `motolii-export-pane` とは別 crate なので `chrome::toggler_style`
/// 経由でこの crate に色ロールを持つ、chrome.rs doc 参照)。
fn auto_save_toggle_row(enabled: bool, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    row![
        text("Automatically Save Projects")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        toggler(enabled)
            .size(dims.body_text)
            .on_toggle(Message::AutoSaveToggle)
            .style(move |_theme, status| toggler_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .height(Length::Fixed(dims.inspector_row_height))
    .align_y(iced::alignment::Vertical::Center)
    .padding([0.0, dims.spacing_m])
    .into()
}

/// 「行ラベル左・数値セル右」の行(AUTOSAVE 版、[`comp_cells_row`] と同じ列
/// グリッド文法 — 冒頭「taffy 転写 第2適用」doc 参照)。
fn auto_save_cells_row(
    label: &'static str,
    fields: &[AutoSaveField],
    config: AutoSaveConfig,
    draft: Option<&AutoSaveFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let row_style = style_from_css_decl(&comp_cells_row_css(dims)).expect(
        "comp_cells_row_css は固定テンプレート+dims の px 値のみを埋める — 解釈は必ず成功する",
    );
    let label_style = style_from_css_decl(COMP_CELLS_LABEL_CSS)
        .expect("COMP_CELLS_LABEL_CSS は固定文字列 — 解釈は必ず成功する");
    let cell_style = style_from_css_decl(&comp_cell_css(dims))
        .expect("comp_cell_css は固定テンプレート+dims の px 値のみを埋める — 解釈は必ず成功する");

    let label_text: Element<'static, Message> = text(label)
        .size(dims.body_text)
        .color(colors.text_primary)
        .into();

    let mut taffy_row: TaffyBox<'static, Message, iced::Theme, iced::Renderer> =
        TaffyBox::new(row_style).push(label_style, label_text);
    for &field in fields {
        taffy_row = taffy_row.push(
            cell_style.clone(),
            auto_save_field_cell(field, config, draft, dims, colors),
        );
    }
    taffy_row.into()
}

/// caption + 数値欄のセル([`comp_field_cell`] と同じ形 — 2箇所で別の意匠を
/// 発明しない)。
fn auto_save_field_cell(
    field: AutoSaveField,
    config: AutoSaveConfig,
    draft: Option<&AutoSaveFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|draft| draft.field == field)
        .map(|draft| draft.text.clone())
        .unwrap_or_else(|| auto_save_field_display(field, &config));

    column![
        // 裁定217 連続量 drag 化(E-5): `comp_field_cell` と同じキャプション
        // drag ハンドル(冒頭コメント参照)。
        mouse_area(
            text(field.caption())
                .size(dims.caption_text)
                .color(colors.text_muted)
                .align_x(iced::alignment::Horizontal::Center)
                .width(Length::Fixed(dims.inspector_value_width))
        )
        .interaction(iced::mouse::Interaction::ResizingHorizontally)
        .on_press(Message::AutoSaveFieldDragPressed(field)),
        // 裁定170 M01: comp_field_cell と同じ理由(owned move で 'static を返す)。
        text_input("", displayed)
            .on_input(move |text| Message::AutoSaveFieldInput(field, text))
            .on_submit(Message::AutoSaveFieldSubmit(field))
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

    // -----------------------------------------------------------------
    // AUTOSAVE(第2切片): 意味実在柵 — 全 AutoSaveField が
    // motolii_store::AutoSaveConfig の実在フィールドを実際に動かす。
    // 発注規律(裁定189): 書くが実行しない。検収線は
    // `cargo check --tests -p motolii-settings-pane` 緑まで。
    // -----------------------------------------------------------------

    /// **本命**: [`AutoSaveField::ALL`] を尽くし、各 commit が対応フィールド
    /// **だけ**を動かすこと(= どの欄も `AutoSaveConfig` の実在識別子に紐づき、
    /// かつ他方の意味を巻き込まない)を1つずつ確かめる
    /// ([`every_comp_field_commits_into_its_real_composition_field_and_nothing_else`]
    /// と同じ形の柵)。
    #[test]
    fn every_auto_save_field_commits_into_its_real_config_field_and_nothing_else() {
        for field in AutoSaveField::ALL {
            let mut config = AutoSaveConfig::default();
            let before = config;
            let text = match field {
                AutoSaveField::IntervalMinutes => "10",
                AutoSaveField::Generations => "8",
            };
            let mut draft = Some(AutoSaveFieldDraft {
                field,
                text: text.to_owned(),
            });
            commit_auto_save_field(&mut config, &mut draft, field)
                .unwrap_or_else(|error| panic!("{field:?} を書けない: {error}"));
            assert!(draft.is_none(), "{field:?}: 確定後も下書きが残っている");

            let expected_interval_secs = if field == AutoSaveField::IntervalMinutes {
                600
            } else {
                before.interval_secs
            };
            let expected_generations = if field == AutoSaveField::Generations {
                8
            } else {
                before.generations
            };
            assert_eq!(
                config.interval_secs, expected_interval_secs,
                "{field:?} 確定後の interval_secs"
            );
            assert_eq!(
                config.generations, expected_generations,
                "{field:?} 確定後の generations"
            );
        }
    }

    /// 別フィールドの下書きしか無い時の Enter は no-op で、下書きも消さない
    /// ([`commit_with_a_mismatched_draft_is_a_no_op_and_keeps_the_draft`] と
    /// 同じ意味論)。
    #[test]
    fn auto_save_commit_with_a_mismatched_draft_is_a_no_op_and_keeps_the_draft() {
        let mut config = AutoSaveConfig::default();
        let before = config;
        let mut draft = Some(AutoSaveFieldDraft {
            field: AutoSaveField::IntervalMinutes,
            text: "10".to_owned(),
        });
        commit_auto_save_field(&mut config, &mut draft, AutoSaveField::Generations)
            .expect("no-op のはず");
        assert!(draft.is_some(), "無関係の Enter が下書きを消した");
        assert_eq!(config.interval_secs, before.interval_secs);
        assert_eq!(config.generations, before.generations);
    }

    /// 読めない入力は Err(status 帯行き)で `AutoSaveConfig` には触らない。
    #[test]
    fn auto_save_commit_rejects_garbage_without_touching_the_config() {
        let mut config = AutoSaveConfig::default();
        let before = config;
        let mut draft = Some(AutoSaveFieldDraft {
            field: AutoSaveField::Generations,
            text: "not a number".to_owned(),
        });
        let error = commit_auto_save_field(&mut config, &mut draft, AutoSaveField::Generations)
            .expect_err("読めない入力が通ってしまった");
        assert!(
            error.contains("not a number"),
            "理由文に入力が引用されていない: {error}"
        );
        assert_eq!(config.generations, before.generations);
    }

    #[test]
    fn parse_auto_save_interval_secs_clamps_into_1_to_1440_minutes() {
        assert_eq!(parse_auto_save_interval_secs("0"), Some(60), "0分未満は1分へクランプ");
        assert_eq!(parse_auto_save_interval_secs("-5"), Some(60));
        assert_eq!(parse_auto_save_interval_secs("5"), Some(300));
        assert_eq!(
            parse_auto_save_interval_secs("999999"),
            Some((MAX_AUTO_SAVE_INTERVAL_MINUTES * 60.0) as u64),
            "24時間超は24時間へクランプ"
        );
        assert_eq!(parse_auto_save_interval_secs("not a number"), None);
    }

    #[test]
    fn parse_auto_save_generations_clamps_into_1_to_50() {
        assert_eq!(parse_auto_save_generations("0"), Some(1));
        assert_eq!(parse_auto_save_generations("-5"), Some(1));
        assert_eq!(parse_auto_save_generations("5"), Some(5));
        assert_eq!(
            parse_auto_save_generations("999"),
            Some(MAX_AUTO_SAVE_GENERATIONS)
        );
        assert_eq!(parse_auto_save_generations("not a number"), None);
    }

    /// 表示→入力→確定の往復で値が動かない([`fps_display_round_trips_through_parse_unchanged`]
    /// と同じ礼儀 — 秒↔分の単位変換をまたいでも安定する)。
    #[test]
    fn auto_save_interval_display_round_trips_through_parse_unchanged() {
        for interval_secs in [60, 300, 600, 3600] {
            let displayed = format_auto_save_interval_minutes(interval_secs);
            assert_eq!(
                parse_auto_save_interval_secs(&displayed),
                Some(interval_secs),
                "{interval_secs}秒 が往復で動いた(表示={displayed:?})"
            );
        }
    }

    #[test]
    fn auto_save_field_display_reads_the_real_config_values() {
        let config = AutoSaveConfig {
            interval_secs: 300,
            generations: 5,
        };
        assert_eq!(auto_save_field_display(AutoSaveField::IntervalMinutes, &config), "5");
        assert_eq!(auto_save_field_display(AutoSaveField::Generations, &config), "5");
    }

    /// [`AutoSaveConfig::default`](既定 5分/5世代、発注書指定)がそのまま
    /// 欄の初期表示になること — 「顔だけの設定」ではなく AS の既定値が実際に
    /// 見える(飾り禁止柵の裏返し)。
    #[test]
    fn default_auto_save_config_displays_ae_precedent_defaults() {
        let config = AutoSaveConfig::default();
        assert_eq!(
            auto_save_field_display(AutoSaveField::IntervalMinutes, &config),
            "5",
            "既定 5分(AutoSaveConfig::DEFAULT_INTERVAL_SECS = 300秒)"
        );
        assert_eq!(
            auto_save_field_display(AutoSaveField::Generations, &config),
            "5",
            "既定 5世代(AutoSaveConfig::DEFAULT_GENERATIONS)"
        );
    }
}
