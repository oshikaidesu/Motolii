//! owns: デザイン token の読み口(寸法JSON+DTCG色の parse・watch・ui_scale 乗算点)。値の正本は JSON 側 — ここは複製しない。
//! デザイン値の外出し(裁定117)。
//!
//! **正本は2つ、どちらもここへコピーしない**:
//! - 寸法: `tokens/dimensions.json`(このファイルが機械可読正本。値は Ableton Live 12
//!   実測 — `docs/reviews/2026-08-19-ableton-density-measurements.md`)
//! - 色: `ui/motolii-tokens/sources/motolii-dark.json`(DTCG 形式。ここでも複製しない)
//!
//! debug ビルドはどちらも起動時にファイルから読み、[`watch_subscription`] が notify で
//! 変更を検知して再読込する。release は `include_str!` で埋め込んだ文字列を起動時に
//! 1回だけ parse する — **file I/O はゼロ**(iced の `Theme` は色・境界・影しか
//! 持てず寸法を Theme 化できないため、自前の [`Tokens`] を `State` に持つ形を採る)。
//!
//! raw 値の直書き禁止 — 全 pane はここ経由で寸法・色を読む。

use std::path::{Path, PathBuf};

use iced::Color;

/// 寸法トークン。**Ableton 実測**(`docs/reviews/2026-08-19-ableton-density-measurements.md`)。
/// 実測に無い値の導出根拠は `tokens/dimensions.json` の `_note_*` キーに書く
/// (JSON はコメントを持てないため)。
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
pub struct Dimensions {
    /// Timeline の行高。
    pub row_height: f32,
    /// transport/Control 帯の高さ。
    pub transport_band: f32,
    /// type scale: panel header・section title 相当。出典: 視覚正本
    /// `next/reference/mocks/ui-scale-and-z.html` の `--t-title`(正典バンド
    /// `{8,9,11,12}` の最大段)。旧値15(裁定117以来)から更新(2026-08-20)。
    pub title_text: f32,
    /// type scale: 本文・property 行相当の文字サイズ。出典: 同 mock `--t-base`。
    /// 旧値13から更新。
    pub body_text: f32,
    /// type scale: section 見出し・column header・hint・glyph 相当(caption)。
    /// 出典: 同 mock `--t-dense`。旧 `small_text`(型スケール語彙統一、2026-08-20)。
    /// 旧値11から更新。
    pub caption_text: f32,
    /// type scale: 正典バンド最小段。出典: 同 mock `--t-micro`。旧値9から更新。
    /// **現時点でどの pane からも未消費**(mock 自体もこの断片では使っていない —
    /// バンド4段のうち予約されているだけの段)。
    pub micro_text: f32,
    /// spacing scale の最小段。**mock `ui-scale-and-z.html` の `--sp1`(2)と同値**
    /// — Inspector の grid gap(X/Y/Z/Key の間隔、`inspector_pane.rs` の
    /// `.spacing(dims.spacing_xs)`)と glyph 高の窪み(`row - spacing_xs` =
    /// `--hit` 相当の `calc(row - 2*s*1px)`)の両方の出典(`tests/
    /// inspector_pixel_fence.rs` が実測で確認)。
    pub spacing_xs: f32,
    /// spacing scale の小段。**mock の `--sp2`(4)と同値** — ident 帯 padding
    /// (縦)と値セル高の窪み(`row - spacing_s` = `.prow .v` の
    /// `calc(row - 4*s*1px)`)の出典。
    pub spacing_s: f32,
    /// spacing scale の中段。**mock の `--sp4`(8)と同値** — ident/cols/prow/
    /// sec/hint の左右 padding の出典(`--sp3`(6、`.ptitle` の icon-to-text
    /// gap)に対応する token は無い — 現実装は `.ptitle` の icon/em バッジを
    /// 描かないので消費先が無いため未採番、`tests/inspector_pixel_fence.rs`
    /// 冒頭の「対象外」に明記)。
    pub spacing_m: f32,
    /// spacing scale の大段。
    pub spacing_l: f32,
    /// 罫線幅(ui-visual-language: フラット・細罫線)。**`ui_scale` を掛けない**
    /// (`Dimensions::scaled` 参照 — mock `--line: 1px` が `--s` の calc から独立して
    /// いる=「拡大しない」ことの直接の出典)。
    pub border_width: f32,
    /// panel header 帯の高さ。Shell 全体の header(Undo/Redo/+Layer 帯)専用 —
    /// Ableton 実測(`docs/reviews/2026-08-19-ableton-density-measurements.md`)。
    /// Inspector 自身のタイトル帯は `inspector_section_header_height` を使う
    /// (mock の `--section` が `.ptitle`/`.sec` 両方に効くのと同じ理由、下記参照)。
    pub panel_header_height: f32,
    /// Inspector pane の固定幅。出典は Ableton 実測ではなく**視覚正本 HTML/CSS 自体**
    /// — **I-tokens(2026-08-22)**: 出典を `next/reference/mocks/inspector-library.html`
    /// v3.1(利用者合格・転写正本)へ更新(`.inspectorShell{width:min(100%,496px)}`)。
    /// 値そのものは旧 `docs/mocks-ui/public/inspector-library.css` から不変(496) —
    /// I-ratio 台帳が発見した二重モック構造(この値だけ inspector-library 由来、
    /// row_height/value_width/glyph_width は ui-scale-and-z.html 由来)を、4値とも
    /// inspector-library v3.1 へ揃えて解消した(300 への変更は不採用のまま確定)。
    pub inspector_panel_width: f32,
    /// Inspector property 行 / column header 行の高さ。**I-tokens(2026-08-22)**:
    /// 出典を `next/reference/mocks/inspector-library.html` v3.1 へ統一
    /// (`.propertyRow{min-height:25px}`)。旧値20(`ui-scale-and-z.html` `--row`
    /// 由来、二重モック構造の片割れ)から更新 — I-ratio 台帳の FINDING
    /// (`body_text`/`inspector_row_height` = 0.55 が裁定168 の帯 0.42±0.05 の外)
    /// をこの書き換えで根治(11/25=0.44 は帯の内)。
    pub inspector_row_height: f32,
    /// Inspector の section 見出し帯高(26)。**2箇所で共有**: panel タイトル帯
    /// (`.ptitle`、旧実装は `panel_header_height` を誤用していた)と section 見出し
    /// (TRANSFORM/APPEARANCE、`.sec`)。inspector-library.css の `.tableSection h2`
    /// (26)と旧 `ui-scale-and-z.html` の `--section`(26)が値として一致していたため
    /// 変更なし — 出典だけ I-tokens(2026-08-22)で inspector-library v3.1 へ統一。
    pub inspector_section_header_height: f32,
    /// Inspector 値セル(X/Y/Z)1つぶんの幅。**I-tokens(2026-08-22)**: 出典を
    /// `next/reference/mocks/inspector-library.html` v3.1 へ統一
    /// (`grid-template-columns: minmax(132px,1fr) repeat(3,64px) 26px` の `64px` 段)。
    /// 旧値38(`ui-scale-and-z.html` `--pane:300` 前提の別モック由来 — 300px pane へ
    /// 正規化した時だけ旧64px値と辻褄が合っていた、I-ratio 台帳 §3.1)から更新。
    pub inspector_value_width: f32,
    /// Inspector の Key/M/S glyph 列の幅。**I-tokens(2026-08-22)**: 出典を
    /// `next/reference/mocks/inspector-library.html` v3.1 へ統一(同 grid の末尾
    /// `26px` 段)。旧値18(`ui-scale-and-z.html` `--hit` 由来)から更新。**Key 列
    /// 自体は空のまま**(Q0: keyframe UI 未実装、列幅の予約だけ)。Timeline の
    /// M/S/L glyph は T-rail(裁定172 §2)で既にこの token の借用をやめ独自比率
    /// (`lane_bar::glyph_size_px`)へ転写済みなので、この値変更は Timeline に波及しない。
    #[serde(default = "default_inspector_glyph_width")]
    pub inspector_glyph_width: f32,
    /// Timeline のレーンバー(行ヘッダ列)幅。出典: 視覚正本
    /// `ui-scale-and-z.html` の `.thead{width:calc(150 * var(--s) * 1px)}`
    /// (裁定147「面の構成」— レーンバーの視覚正本はこの mock の行ヘッダ列)。
    /// M/S/L glyph 幅は新トークンを増やさず `inspector_glyph_width` を使い回す
    /// (Inspector の Key/M/S 列と同じ意味段、`timeline/lane_bar.rs` 参照)。
    #[serde(default = "default_timeline_lane_bar_width")]
    pub timeline_lane_bar_width: f32,
    /// property 行(キー行、第2波 T3)1本の高さ。**本行(`row_height`)より低い**。
    /// mock に対応する段が無いので新設 — 出典は egui 版
    /// `timeline_editor::{ROW_H, PROP_H}`(24/20)の比を `row_height`(20)へ
    /// 適用した値(`tokens/dimensions.json` の `_note_timeline_param_row_height`
    /// 参照)。
    #[serde(default = "default_timeline_param_row_height")]
    pub timeline_param_row_height: f32,
    /// mock `--s` 相当の UI 拡大率(1.00 基準、0.01 刻み)。**適用点は
    /// [`Dimensions::scaled`] の1箇所だけ** — 個々の pane はここを直接読まず、
    /// [`crate::Shell::dims`] が返す「掛け算済みの」`Dimensions` を読む。
    ///
    /// **仮の置き場**: 発注書は正本を Workspace 永続に置くよう指示しているが、
    /// この裁定時点で Workspace 機構がまだ無いため、暫定的にこの JSON トークン
    /// ファイル(`tokens/dimensions.json`)の値として持つ。Workspace が実装され
    /// 次第そちらへ移す。
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    /// Browser pane のタブ帯(Media/Effects/Create/Panels)の高さ。出典:
    /// 視覚正本 `next/reference/mocks/browser-library.html` の
    /// `.libraryTabs{height:26px}` 実測(`tokens/dimensions.json` の
    /// `_note_browser_tab_bar_height` 参照、2026-08-22 利用者裁定
    /// 「デザイン値の外出し徹底」で JSON 正本へ)。
    #[serde(default = "default_browser_tab_bar_height")]
    pub browser_tab_bar_height: f32,
    /// Browser タブ帯の active タブ下線の太さ。出典: 同 mock
    /// `.libraryTabs button{border-bottom:2px}` 実測(`_note_browser_tab_
    /// underline` 参照)。`spacing_xs` と同値(2)だが意味は「選択状態の縁」 —
    /// spacing の段を縁へ転用しない。
    #[serde(default = "default_browser_tab_underline")]
    pub browser_tab_underline: f32,
}

fn default_inspector_glyph_width() -> f32 {
    26.0
}

fn default_timeline_lane_bar_width() -> f32 {
    150.0
}

fn default_timeline_param_row_height() -> f32 {
    16.67
}

fn default_ui_scale() -> f32 {
    1.0
}

fn default_browser_tab_bar_height() -> f32 {
    26.0
}

fn default_browser_tab_underline() -> f32 {
    2.0
}

impl Default for Dimensions {
    fn default() -> Self {
        // ファイルが読めない・壊れている時の最終防波堤(M16: render 失敗でも画面を
        // 空にしない、と同じ理由)。値は正本 JSON と同じ Ableton 実測(または
        // dimensions.json の `_note_*` と同じ導出根拠) — 正本が2つに増えるわけでは
        // なく、正本を読めなかった時だけ使う既定値。
        Self {
            row_height: 20.0,
            transport_band: 30.0,
            title_text: 12.0,
            body_text: 11.0,
            caption_text: 9.0,
            micro_text: 8.0,
            spacing_xs: 2.0,
            spacing_s: 4.0,
            spacing_m: 8.0,
            spacing_l: 12.0,
            border_width: 1.0,
            panel_header_height: 29.0,
            inspector_panel_width: 496.0,
            inspector_row_height: 25.0,
            inspector_section_header_height: 26.0,
            inspector_value_width: 64.0,
            inspector_glyph_width: 26.0,
            timeline_lane_bar_width: 150.0,
            timeline_param_row_height: 16.67,
            ui_scale: 1.0,
            browser_tab_bar_height: 26.0,
            browser_tab_underline: 2.0,
        }
    }
}

impl Dimensions {
    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|error| error.to_string())
    }

    /// debug ビルドでの読み込み元。`CARGO_MANIFEST_DIR` は compile time に決まるので、
    /// 実行時の cwd に依存しない。
    pub fn debug_source_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tokens/dimensions.json")
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        Self::parse(&text)
    }

    /// `ui_scale` を全寸法・全文字サイズへ適用する**唯一の乗算点**(発注書:
    /// 「全寸法・全文字サイズの読み出し口で乗算(適用点1箇所)」)。呼び出すのは
    /// [`crate::Shell::dims`] だけ — 個々の pane 関数は掛け算済みの `Dimensions`
    /// を受け取るだけで、自分で `ui_scale` を読まない。
    ///
    /// **罫線幅だけ例外**: mock(`ui-scale-and-z.html`)の `--line: 1px` は `--s` の
    /// calc から独立している(拡大しない、コメント「線だけは物理1px床」)。ここでは
    /// 掛け算せず、物理1px の床にだけクランプする(トークン側の設定が万一1未満でも
    /// 沈み込まない防波堤)。
    pub fn scaled(&self, ui_scale: f32) -> Self {
        let s = ui_scale;
        Self {
            row_height: self.row_height * s,
            transport_band: self.transport_band * s,
            title_text: self.title_text * s,
            body_text: self.body_text * s,
            caption_text: self.caption_text * s,
            micro_text: self.micro_text * s,
            spacing_xs: self.spacing_xs * s,
            spacing_s: self.spacing_s * s,
            spacing_m: self.spacing_m * s,
            spacing_l: self.spacing_l * s,
            border_width: self.border_width.max(1.0),
            panel_header_height: self.panel_header_height * s,
            inspector_panel_width: self.inspector_panel_width * s,
            inspector_row_height: self.inspector_row_height * s,
            inspector_section_header_height: self.inspector_section_header_height * s,
            inspector_value_width: self.inspector_value_width * s,
            inspector_glyph_width: self.inspector_glyph_width * s,
            timeline_lane_bar_width: self.timeline_lane_bar_width * s,
            timeline_param_row_height: self.timeline_param_row_height * s,
            browser_tab_bar_height: self.browser_tab_bar_height * s,
            browser_tab_underline: self.browser_tab_underline * s,
            // 自分自身は「寸法」ではないので掛けない。この結果を再度 `scaled()`
            // に通す呼び出し側は無い(適用点は `Shell::dims` の1箇所だけ)。
            ui_scale: self.ui_scale,
        }
    }
}

/// 色トークン。**正本は `ui/motolii-tokens/sources/motolii-dark.json`**(DTCG 形式)。
/// ここへ色そのものを複製しない — 読む口だけを持つ。
///
/// `state_selected`/`state_disabled` は正本 JSON に対応するロールが無い
/// (`ui/motolii-tokens` はこのレーンの変更範囲外 — shell のみ)。近縁色から
/// [`derive_state_colors`] で導出する(発明ではなく、正本ロールの合成)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors {
    pub surface_app: Color,
    pub surface_panel: Color,
    pub surface_raised: Color,
    /// 状態: hover(正本ロール `surface.hover`)。
    pub surface_hover: Color,
    pub border_default: Color,
    pub border_strong: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub focus: Color,
    /// accent(正本ロール `action.active`)。
    pub action_active: Color,
    pub data: Color,
    pub shape: Color,
    /// status 帯の警告系(正本ロール `status.warning`)。
    pub status_warning: Color,
    pub status_ok: Color,
    /// Timeline pane 専用のアクセント色(`way.timeline`)。他 pane も同族の `way.*`
    /// を同じ要領で足せる。
    pub way_timeline: Color,
    /// 状態: 選択。正本に無いロール — `surface.raised` を `action.active` へ 18%
    /// ブレンドして導出(hover の中立グレーと区別が付く、accent 味の選択強調)。
    pub state_selected: Color,
    /// 状態: 無効。正本に無いロール — `text.muted` を `surface.panel` へ 40%
    /// ブレンドして導出(text.muted より一段暗く、読めるが「押せない」と分かる)。
    pub state_disabled: Color,
    /// hairline(弱)。**裁定142 の先行整備で `inspector_pane.rs` の
    /// `PROW_HAIRLINE`(生 `rgba(0,0,0,.35)` リテラル)をここへ昇格**したロール —
    /// 出典は mock `ui-scale-and-z.html` の `.prow{border-bottom:...rgba(0,0,0,.35)}`
    /// (`.trow{border-bottom:...rgba(0,0,0,.4)}` も同系統だが、正本ロールを2本に
    /// 割らず1本へ丸める — 裁定142「値の正本は tokens 1箇所」)。`border_default`
    /// (不透明 `#1a1a1a` 相当、`.cols`/`.ptitle` 系の強い区切り)より薄い、
    /// 行同士の弱い区切り。DTCG 正本(`ui/motolii-tokens`)にロールが無いので
    /// `state_selected`/`state_disabled` と同じ「合成して1箇所に持つ」扱い
    /// ([`fixed_wash_colors`] 参照、`Default`/`parse` の両方が同じ値を使う)。
    pub border_hairline_weak: Color,
    /// Timeline 明暗リズム — 時間方向(裁定148(1))。区間(秒)ごとに交互に乗せる
    /// 白の薄い wash。出典: mock `ui-scale-and-z.html` の
    /// `.tick{background:rgba(255,255,255,.035)}`(裁定148 doc「Ableton の拍
    /// グリッド陰影と同型」の実装形そのもの)。区切りの手段ではない —
    /// 読解補助の「地」の微差(§1.6)。
    pub timeline_time_band: Color,
    /// Timeline 明暗リズム — 行方向(裁定148(2))。レーン交互のゼブラ、奇数行
    /// にだけ乗せる白の薄い wash。**直接の実測ソースは無い** —
    /// [`docs/reviews/2026-08-19-flat-grammar-canon-revision.md`] が転記した
    /// cosmic-theme の状態 α ladder(hover=0.10)を上限に、hover(操作可能性の
    /// 合図)より弱い ambient な差にとどまるよう `timeline_time_band` と同じ
    /// 桁の値(0.05)を採る(発明ではなく上限からの逆算、2026-08-21)。
    pub timeline_row_zebra: Color,
    /// Timeline 縦線 — 全目盛の投影(利用者裁定 2026-08-21 夜、mock
    /// `timeline-semantics.html` 意味層注記が出典)。時間方向は周波数で役割
    /// 分担する: [`Colors::timeline_time_band`](面、大目盛周期の粗いリズム)
    /// に対して、この2ロールは**線**(全目盛の細かいリズム)を担う。
    /// `timeline_grid_minor` は小目盛(弱)。S4 柵(裁定164「意味役割が新しい
    /// 時は既存段を借用せず専用ロールを起こす」)適用 — mock 注記は
    /// 「既存 hairline_weak 流用 or 新 grid ロール」を実装時の裁定点として
    /// 残していたが、[`Colors::border_hairline_weak`] は「区切り」の意味役割
    /// (行/面の境界線)であって「時間の細かいリズムの投影」ではないため、
    /// 専用ロールを起こす側を採る。DTCG 正本にロールが無いので
    /// [`Colors::border_hairline_weak`] 等と同じ「固定値を1箇所に持つ」扱い
    /// ([`fixed_grid_colors`] 参照)。値は黒 α0.18(mock 実測値の写し)。
    pub timeline_grid_minor: Color,
    /// Timeline 縦線 — 大目盛(わずかに強い確認線)。[`Colors::
    /// timeline_grid_minor`] と対 — 面の分割ではなく「帯の境界の確認線」
    /// (mock 注記)。値は黒 α0.30(mock 実測値の写し、`timeline_grid_minor`
    /// より強いが `border_hairline_weak` の α0.35 より弱い)。
    pub timeline_grid_major: Color,
    /// 市松(透明の可視化、`motolii_settings_pane::composite_checkerboard`)の
    /// 明タイル。**`surface_raised`/`surface_panel`(パネル面ロール)からの
    /// 独立ロール** — 市松v2(利用者較正 2026-08-21「市松が見えない」)の
    /// 根治対象。旧実装はこの2色を `surface_raised`/`surface_panel` から
    /// 借用していたが、その2色は「パネルの面」という別の意味役割のために
    /// 選ばれた値で、たまたま並べても Δ8/255(実測 54/62)しか差が無く実質
    /// 不可視だった。市松は「透明の合図」という独自の意味役割を持つので、
    /// `docs/ui-spatial-score.md` S4 の柵(裁定164:「意味役割が新しい時は
    /// 段の借用でなく新ロールを起こす」— この事件が由来)に従い専用ロールを
    /// 起こす。DTCG 正本にロールが無いので `border_hairline_weak` 等と同じ
    /// 「固定値を1箇所に持つ」扱い([`fixed_checkerboard_colors`] 参照)。
    /// 値は AE 実機準拠の視認差(明 0.42 灰、Δ≈30/255)。
    pub checkerboard_light: Color,
    /// 市松の暗タイル。[`Colors::checkerboard_light`] と対。値は AE 実機準拠
    /// (暗 0.30 灰、`checkerboard_light` との差 Δ≈30/255)。
    pub checkerboard_dark: Color,
    /// レイヤー差し色パレット(利用者裁定2026-08-21「色が足りない。Ableton は
    /// レイヤー全部に色」)。[`motolii_store::LayerAttrs::label_color`] が保存する
    /// index(`0..LABEL_PALETTE_LEN`)がこの配列を引く — DTCG 正本(`ui/motolii-tokens`)
    /// にロールが無い新設ロール(裁定164 の S4 柵: 意味役割が新しいので
    /// `surface`/`accent` 等の既存段を借用せず専用ロールを起こす。
    /// [`Colors::checkerboard_light`] と同じ理由)。`Default`/`parse` の両方で
    /// [`fixed_label_palette`] を使うので値は1箇所。
    pub label_palette: [Color; LABEL_PALETTE_LEN],
}

/// [`Colors::label_palette`] の長さ。生成時の決定論自動割当(`motolii_shell` の
/// `LayerId % LABEL_PALETTE_LEN`)と、表示側の index 境界チェックの両方が
/// この定数を参照する(値を2箇所に持たない)。
pub const LABEL_PALETTE_LEN: usize = 12;

/// [`Colors::label_palette`] の固定値。HSL→RGB で 12色、hue は 0° から 30° 刻み
/// (`i * 30.0`、昇順)。
///
/// **採択は候補C**(トンマナ従属・低彩度、`S=0.32, L=0.62` — 現行 bar 色
/// `way_timeline` と同明度帯)。比較のために供覧した他候補(発注書 supervisor 指定、
/// いずれも実装値は変えていない):
/// - 候補A(Ableton 風・識別強): `S=0.55, L=0.60`
/// - 候補B(AE ラベル風・伝統): `S=0.45, L=0.55`
///
/// 比較PNGは3候補それぞれでこの関数の `saturation`/`lightness` 引数を一時的に
/// 差し替えて撮った(ゼブラ比較と同じ一時差し替え方式)。この commit に残るのは
/// 採択前の既定である候補Cのみ。
fn fixed_label_palette() -> [Color; LABEL_PALETTE_LEN] {
    hsl_palette(0.32, 0.62)
}

/// hue を `0..LABEL_PALETTE_LEN` へ 30° 刻みで割り、`(saturation, lightness)` 固定で
/// [`hsl_to_rgb`] へ渡す。
fn hsl_palette(saturation: f32, lightness: f32) -> [Color; LABEL_PALETTE_LEN] {
    let mut out = [Color::from_rgb(0.0, 0.0, 0.0); LABEL_PALETTE_LEN];
    for (index, slot) in out.iter_mut().enumerate() {
        let hue = index as f32 * 30.0;
        *slot = hsl_to_rgb(hue, saturation, lightness);
    }
    out
}

/// 標準の HSL→RGB 変換(`hue` は度 `[0, 360)`、`saturation`/`lightness` は
/// `[0.0, 1.0]`)。発明の余地が無い教科書アルゴリズムなのでそのまま採る。
fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> Color {
    let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let h_prime = (hue.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x), // 5: h_prime in [5,6)
    };
    let m = lightness - c / 2.0;
    Color::from_rgb(r1 + m, g1 + m, b1 + m)
}

/// [`Colors::border_hairline_weak`]/[`Colors::timeline_time_band`]/
/// [`Colors::timeline_row_zebra`] の固定値。DTCG 正本にロールが無い3本を
/// `Default`/`parse` の両方で同じ式にするための唯一の実装
/// ([`derive_state_colors`] と同じ理由)。
fn fixed_wash_colors() -> (Color, Color, Color) {
    let border_hairline_weak = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.35,
    };
    let timeline_time_band = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.035,
    };
    let timeline_row_zebra = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.05,
    };
    (border_hairline_weak, timeline_time_band, timeline_row_zebra)
}

/// [`Colors::checkerboard_light`]/[`Colors::checkerboard_dark`] の固定値。
/// `Default`/`parse` の両方で同じ式にするための唯一の実装([`fixed_wash_colors`]
/// と同じ理由)。**`surface_raised`/`surface_panel` を参照しない** — パネル面
/// ロールからの独立を保つのがこの2色を新設した理由そのもの(市松v2、
/// [`Colors::checkerboard_light`] doc 参照)。
fn fixed_checkerboard_colors() -> (Color, Color) {
    let checkerboard_light = Color::from_rgb(0.42, 0.42, 0.42);
    let checkerboard_dark = Color::from_rgb(0.30, 0.30, 0.30);
    (checkerboard_light, checkerboard_dark)
}

/// [`Colors::timeline_grid_minor`]/[`Colors::timeline_grid_major`] の固定値。
/// `Default`/`parse` の両方で同じ式にするための唯一の実装([`fixed_wash_colors`]
/// と同じ理由)。値は mock `timeline-semantics.html` の `bands()` 第2ループ
/// (`rgba(0,0,0,${f%major===0?0.30:0.18})`)の実測写し — 黒(区切り hairline と
/// 同じ色相)、α だけ2段(小=弱・大=わずかに強い確認線)。
fn fixed_grid_colors() -> (Color, Color) {
    let timeline_grid_minor = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.18,
    };
    let timeline_grid_major = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.30,
    };
    (timeline_grid_minor, timeline_grid_major)
}

/// `surface.raised`/`text.muted`/`surface.panel`/`action.active` から
/// `state_selected`/`state_disabled` を合成する。**正本 JSON にも [`Default`] にも
/// 同じ式を使う**(2箇所で別の値にならないようにするための唯一の実装)。
fn derive_state_colors(
    surface_raised: Color,
    surface_panel: Color,
    text_muted: Color,
    action_active: Color,
) -> (Color, Color) {
    let selected = blend(surface_raised, action_active, 0.18);
    let disabled = blend(text_muted, surface_panel, 0.40);
    (selected, disabled)
}

fn blend(from: Color, to: Color, t: f32) -> Color {
    Color::from_rgb(
        from.r + (to.r - from.r) * t,
        from.g + (to.g - from.g) * t,
        from.b + (to.b - from.b) * t,
    )
}

impl Default for Colors {
    fn default() -> Self {
        // Dimensions と同じ理由の最終防波堤。数値は motolii-dark.json のスナップショットだが
        // **正本はあくまで JSON 側**(読めた時は常にそちらを使う)。
        let surface_raised = Color::from_rgb(0.2431, 0.2431, 0.2431);
        let surface_panel = Color::from_rgb(0.2118, 0.2118, 0.2118);
        let text_muted = Color::from_rgb(0.4588, 0.4588, 0.4588);
        let action_active = Color::from_rgb(0.8471, 0.7098, 0.4549);
        let (state_selected, state_disabled) =
            derive_state_colors(surface_raised, surface_panel, text_muted, action_active);
        let (border_hairline_weak, timeline_time_band, timeline_row_zebra) = fixed_wash_colors();
        let (timeline_grid_minor, timeline_grid_major) = fixed_grid_colors();
        let (checkerboard_light, checkerboard_dark) = fixed_checkerboard_colors();
        let label_palette = fixed_label_palette();
        Self {
            surface_app: Color::from_rgb(0.1412, 0.1412, 0.1412),
            surface_panel,
            surface_raised,
            surface_hover: Color::from_rgb(0.2745, 0.2745, 0.2745),
            border_default: Color::from_rgb(0.1020, 0.1020, 0.1020),
            border_strong: Color::from_rgb(0.3569, 0.3569, 0.3569),
            text_primary: Color::from_rgb(0.7216, 0.7216, 0.7216),
            text_secondary: Color::from_rgb(0.5490, 0.5490, 0.5490),
            text_muted,
            focus: Color::from_rgb(0.9412, 0.9412, 0.9412),
            action_active,
            data: Color::from_rgb(0.4706, 0.7098, 0.6902),
            shape: Color::from_rgb(0.6667, 0.6275, 0.8157),
            status_warning: Color::from_rgb(0.8824, 0.5412, 0.4275),
            status_ok: Color::from_rgb(0.5647, 0.6980, 0.5294),
            way_timeline: Color::from_rgb(0.8000, 0.5843, 0.5294),
            state_selected,
            state_disabled,
            border_hairline_weak,
            timeline_time_band,
            timeline_row_zebra,
            timeline_grid_minor,
            timeline_grid_major,
            checkerboard_light,
            checkerboard_dark,
            label_palette,
        }
    }
}

impl Colors {
    pub fn parse(json: &str) -> Result<Self, String> {
        let root: serde_json::Value =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        let color = root.get("color").ok_or("color 節が無い")?;
        let surface_raised = color_at(color, &["surface", "raised"])?;
        let surface_panel = color_at(color, &["surface", "panel"])?;
        let text_muted = color_at(color, &["text", "muted"])?;
        let action_active = color_at(color, &["action", "active"])?;
        let (state_selected, state_disabled) =
            derive_state_colors(surface_raised, surface_panel, text_muted, action_active);
        let (border_hairline_weak, timeline_time_band, timeline_row_zebra) = fixed_wash_colors();
        let (timeline_grid_minor, timeline_grid_major) = fixed_grid_colors();
        let (checkerboard_light, checkerboard_dark) = fixed_checkerboard_colors();
        let label_palette = fixed_label_palette();
        Ok(Self {
            surface_app: color_at(color, &["surface", "app"])?,
            surface_panel,
            surface_raised,
            surface_hover: color_at(color, &["surface", "hover"])?,
            border_default: color_at(color, &["border", "default"])?,
            border_strong: color_at(color, &["border", "strong"])?,
            text_primary: color_at(color, &["text", "primary"])?,
            text_secondary: color_at(color, &["text", "secondary"])?,
            text_muted,
            focus: color_at(color, &["focus"])?,
            action_active,
            data: color_at(color, &["data"])?,
            shape: color_at(color, &["shape"])?,
            status_warning: color_at(color, &["status", "warning"])?,
            status_ok: color_at(color, &["status", "ok"])?,
            way_timeline: color_at(color, &["way", "timeline"])?,
            state_selected,
            state_disabled,
            border_hairline_weak,
            timeline_time_band,
            timeline_row_zebra,
            timeline_grid_minor,
            timeline_grid_major,
            checkerboard_light,
            checkerboard_dark,
            label_palette,
        })
    }

    /// debug ビルドでの読み込み元。**正本は1つ** — `ui/motolii-tokens` 配下のこのファイル
    /// をそのまま読み、コピーは作らない。
    pub fn debug_source_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../ui/motolii-tokens/sources/motolii-dark.json")
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        Self::parse(&text)
    }
}

/// `iced::application` の `.theme()` に渡す、tokens 由来の `iced::Theme`。
///
/// **未結線だと何が起きるか**(実測当時 — `next/` workspace の `iced` は
/// crates.io 版 0.14.0 そのもの、fork/patch なし。`cargo metadata` で確認済み。
/// 裁定170 M01 で fork `oshikaidesu/iced#motolii/host-seams`(rev
/// `73e686ee05efd7d1b61cfea2647186b336d9ab9c`、0.15.0-dev)へ pin した後も
/// この分岐の力学自体は不変、行番号は 0.14.0 系のもの):
/// `.theme()` を呼ばないと `Program::theme` の既定実装
/// (`iced_program-0.14.0/src/lib.rs` 100-106行)が常に `None` を返し、winit 側は
/// `<Theme as theme::Base>::default(system_theme)`
/// (`iced_winit-0.14.0/src/window/state.rs` 60行)へフォールバックする。
/// `system_theme` は OS の外観設定(`event_loop.system_theme()`、
/// `iced_winit-0.14.0/src/lib.rs` 182行)から来るが、取得できない場合は
/// `Mode::None` → `iced_core-0.14.0/src/theme.rs` 277-280行の
/// `Mode::None | Mode::Light => Self::Light` で **iced 組み込みの
/// `Theme::Light`**(tokens と無関係な既定パレット)に落ちる。OS が Dark でも
/// 得られるのは iced 組み込みの `Theme::Dark` であって、`Colors`(このモジュール
/// の tokens 正本)由来の色ではない — どちらの分岐でも実窓の地色
/// (`theme::Style::background_color` = `extended_palette().background.base.color`、
/// `iced_core-0.14.0/src/theme.rs` 291-336行の `Base::base`/`default`)は
/// tokens と切れている。
///
/// 直書き禁止(裁定142)なので raw な `Color::from_rgb(..)` を並べず、
/// `Colors` から `iced::theme::palette::Seed`(background/text/primary/
/// success/warning/danger の6色だけを持つ「種」— fork
/// `core/src/theme/palette.rs:152-165`。**0.14.0 では同じ形の構造体が
/// `palette::Palette` という名前だった**が、fork ではこの名前は「`Seed` から
/// 生成された拡張パレット」(`Background`/`Swatch` 各段・`is_dark` を持つ、旧
/// 0.14 の `palette::Extended` 相当、同ファイル 8-23行)へ意味が移っている —
/// 裁定170 M01 の実装時に検出・裁定済みの名前の入れ替わり)を組んで
/// `Theme::custom` に渡す。`Theme::custom` は内部で `Palette::generate` を呼び、
/// そこから widget 既定色一式(`Background`/`Primary`/… の各段)を導出する
/// (同ファイル 25-38行の `Palette::generate`、上流の標準経路。0.14.0 系での
/// 呼称は `Extended::generate`)。
///
/// **danger ロールが正本に無い**(`Colors` のフィールド一覧参照) —
/// `status_warning` を仮当てする(発明ではなく既存ロールの再利用。危険色が
/// 要る場面が実装されたら正本側にロールを起こす)。
///
/// **watch 追随**: この関数は `Colors` の純粋関数(ファイル I/O をしない)。
/// `.theme(|state| theme_from_colors(&state.colors()))` の形で結線すれば、
/// winit は毎 `synchronize`(`iced_winit-0.14.0/src/window/state.rs` 219行、
/// update のたびに走る)でこのクロージャを呼び直すので、
/// `Message::TokensFileChanged` で `Shell.tokens` が debug watch 経由で
/// 差し替わった次の再描画には新しい色がそのまま反映される — 追加の配線は
/// 要らない。
pub fn theme_from_colors(colors: &Colors) -> iced::Theme {
    let seed = iced::theme::palette::Seed {
        background: colors.surface_app,
        text: colors.text_primary,
        primary: colors.action_active,
        success: colors.status_ok,
        warning: colors.status_warning,
        danger: colors.status_warning,
    };
    iced::Theme::custom("Motolii Dark".to_owned(), seed)
}

/// DTCG の `{"$value": {"components": [r,g,b]}}` を辿って `Color` を取り出す。
fn color_at(root: &serde_json::Value, path: &[&str]) -> Result<Color, String> {
    let mut node = root;
    for segment in path {
        node = node
            .get(segment)
            .ok_or_else(|| format!("token path 不明: {}", path.join(".")))?;
    }
    let components = node
        .get("$value")
        .and_then(|value| value.get("components"))
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("{} に $value.components が無い", path.join(".")))?;
    if components.len() < 3 {
        return Err(format!("{} の components が3未満", path.join(".")));
    }
    let component = |index: usize| -> Result<f32, String> {
        components[index]
            .as_f64()
            .map(|value| value as f32)
            .ok_or_else(|| format!("{} の component が数値でない", path.join(".")))
    };
    Ok(Color::from_rgb(component(0)?, component(1)?, component(2)?))
}

/// 文字の太さ3段(裁定137「文字の階層はサイズでなく weight(400/600/800)と
/// ink 3段で作る」)。CSS `font-weight` の値とそのまま対応させ、視覚正本
/// `next/reference/mocks/ui-scale-and-z.html` の実使用箇所を名指しする:
/// `.glyph`(M/S/Key マーカー全般)= 800、`.ident b`(identity 名の強調)= 600、
/// それ以外の本文は既定 400(明示しなくても iced の既定と同じ)。
/// `iced::font::Weight` は 100刻みの9段(Thin..Black)を持つので、CSS の
/// 400/600/800 は `Normal`/`Semibold`/`ExtraBold` に1:1で対応する
/// (`iced_core::font::Weight` 実測、上流に per-CSS-value のズレは無い)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWeight {
    /// 400 — 本文既定。
    Regular,
    /// 600 — mock `.ident b`(identity 名)。
    Semibold,
    /// 800 — mock `.glyph`(M/S/Key マーカー)。
    Bold,
}

impl TextWeight {
    /// `text`/`text_input` の `.font(..)` へそのまま渡せる `iced::Font`。
    pub fn font(self) -> iced::Font {
        iced::Font {
            weight: match self {
                TextWeight::Regular => iced::font::Weight::Normal,
                TextWeight::Semibold => iced::font::Weight::Semibold,
                TextWeight::Bold => iced::font::Weight::ExtraBold,
            },
            ..iced::Font::DEFAULT
        }
    }
}

/// ink 3段(裁定137)。**新色は発明しない** — 既存 [`Colors`] の `text_*` を
/// そのまま返す薄いラッパー。呼び出し側が raw な `colors.text_muted` 等を
/// 直書きする代わりに、mock の `--ink`/`--ink2`/`--ink3` と同じ語彙(意味段)
/// で選べるようにするだけで、色の実体は増えない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    /// mock `--ink`(既定の本文色。`.prow .n` 等)。
    Primary,
    /// mock `--ink2`(`.glyph`/`.ident s` — 二次的な情報)。
    Secondary,
    /// mock `--ink3`(`.sec`/`.cols`/`.hint` — 最も控えめな注記)。
    Muted,
}

impl Ink {
    pub fn resolve(self, colors: &Colors) -> Color {
        match self {
            Ink::Primary => colors.text_primary,
            Ink::Secondary => colors.text_secondary,
            Ink::Muted => colors.text_muted,
        }
    }
}

/// 全 pane が読む、この起動時点でのデザイン値の姿。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub dims: Dimensions,
    pub colors: Colors,
    /// mock `--s` 相当の UI 拡大率。**正本は `dims.ui_scale`**(JSON トークン
    /// ファイル経由でホットリロードされる) — ここへは [`Tokens::load`]/
    /// [`Default`] がその値をそのまま写す(発注書が指定した置き場 `Tokens.ui_scale`
    /// を公開しつつ、実体は1つに保つ)。
    pub ui_scale: f32,
}

impl Default for Tokens {
    fn default() -> Self {
        let dims = Dimensions::default();
        Self {
            ui_scale: dims.ui_scale,
            dims,
            colors: Colors::default(),
        }
    }
}

// release ビルドは正本 JSON をコンパイル時に埋め込む。**file I/O ゼロ**。
#[cfg(not(debug_assertions))]
const DIMENSIONS_JSON: &str = include_str!("../tokens/dimensions.json");
#[cfg(not(debug_assertions))]
const COLOR_TOKENS_JSON: &str =
    include_str!("../../../../ui/motolii-tokens/sources/motolii-dark.json");

impl Tokens {
    /// 起動時の読み込み。debug はファイルから、release は埋め込み文字列から。
    pub fn load() -> Self {
        #[cfg(debug_assertions)]
        {
            let dims =
                Dimensions::load_from_path(&Dimensions::debug_source_path()).unwrap_or_default();
            let colors = Colors::load_from_path(&Colors::debug_source_path()).unwrap_or_default();
            Self {
                ui_scale: dims.ui_scale,
                dims,
                colors,
            }
        }
        #[cfg(not(debug_assertions))]
        {
            let dims = Dimensions::parse(DIMENSIONS_JSON).unwrap_or_default();
            let colors = Colors::parse(COLOR_TOKENS_JSON).unwrap_or_default();
            Self {
                ui_scale: dims.ui_scale,
                dims,
                colors,
            }
        }
    }
}

/// [`Dimensions::ui_scale`] だけを書き戻す surgical replace(テキストレベル)。
/// **`serde_json::to_string` で丸ごと書き直さない** — 正本 JSON は `_note_*` キー
/// (コメント代わり、`Dimensions` 構造体に対応フィールドが無い)を持つので、struct
/// 経由の再シリアライズはそれを消してしまう。`"ui_scale"` キーの値部分だけを
/// テキストとして置換し、それ以外の1バイトも変えない。
pub fn replace_ui_scale(json: &str, ui_scale: f32) -> Result<String, String> {
    let key = "\"ui_scale\"";
    let key_pos = json
        .find(key)
        .ok_or_else(|| "ui_scale キーが無い".to_owned())?;
    let after_key = &json[key_pos + key.len()..];
    let colon_offset = after_key
        .find(':')
        .ok_or_else(|| "ui_scale キーの直後に : が無い".to_owned())?;
    let value_start = key_pos + key.len() + colon_offset + 1;
    let rest = &json[value_start..];
    let end_offset = rest
        .find(|c: char| c == ',' || c == '}')
        .ok_or_else(|| "ui_scale の値の終端(, か })が見つからない".to_owned())?;

    let mut result = String::with_capacity(json.len() + 8);
    result.push_str(&json[..value_start]);
    result.push_str(&format!(" {ui_scale:.2}"));
    result.push_str(&json[value_start + end_offset..]);
    Ok(result)
}

/// [`replace_ui_scale`] を実ファイルへ適用する(read-modify-write)。**path 引数を
/// 取る** — `tokens/dimensions.json` は複数 worktree・並列試験間で共有される
/// delicate なファイル(`../reference/KNOWN.md`「レーン運用」)なので、試験は
/// `motolii_testkit::tmp_dir()` の隔離コピーでこの関数を叩く(実ファイルは
/// [`save_ui_scale`] からしか触らない)。
pub fn write_ui_scale_to_path(path: &Path, ui_scale: f32) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let replaced = replace_ui_scale(&text, ui_scale)?;
    std::fs::write(path, replaced).map_err(|error| error.to_string())
}

/// `ui_scale` の実行時の書き戻し口。**debug ビルドだけが実際に正本 JSON へ触る**
/// (`watch_subscription` と同じ判断) — release は `include_str!` で埋め込み済み
/// なので、書いても次回起動には反映されない(file I/O 自体をしない)。
///
/// **この関数自体は自動試験の対象にしない**: `tokens/dimensions.json` は複数
/// worktree・並列試験間で共有されるファイルで、ここを試験で書き換えると他の
/// 試験(このファイルを読む `tests/drive.rs` 等)とレースする。書き戻しの実質
/// (テキスト置換の正しさ)は [`replace_ui_scale`]/[`write_ui_scale_to_path`] が
/// 隔離された文字列・一時ファイルで検分済み — ここは経路を1行つなぐだけ。
#[cfg(debug_assertions)]
pub fn save_ui_scale(ui_scale: f32) -> Result<(), String> {
    write_ui_scale_to_path(&Dimensions::debug_source_path(), ui_scale)
}

#[cfg(not(debug_assertions))]
pub fn save_ui_scale(_ui_scale: f32) -> Result<(), String> {
    Ok(())
}

/// トークンファイルの変更を見張る `Subscription`。**debug ビルドのみ実際に見張る**
/// — release はホットリロードを前提にしない(裁定117)ので何も発行しない。
///
/// 発行するのは `()` だけ(このモジュールは `Message` 型を知らない)。呼び出し側
/// (`Shell::subscription`)が `.map(|_| Message::TokensFileChanged)` で繋ぐ。
#[cfg(debug_assertions)]
pub fn watch_subscription() -> iced::Subscription<()> {
    iced::Subscription::run(watch_stream)
}

#[cfg(not(debug_assertions))]
pub fn watch_subscription() -> iced::Subscription<()> {
    iced::Subscription::none()
}

/// 1回の保存につき notify が出す raw event を1回の通知へ束ねる窓。
///
/// **実測(2026-08-20)**: 多くのエディタ/OS はファイル1本の保存で write+rename
/// 等、**複数の raw event を連続して出す**(notify 自体のドキュメントにも
/// 明記されている一般的挙動)。束ねずに全部 `Message::TokensFileChanged` へ流すと
/// 1回の保存で `Tokens::load()`(file I/O + JSON parse ×2)が複数回走り、
/// そのたび `view()` が再構築される — Stage の Handle 自体には触れない
/// (`refresh_frame` は revision/playhead が同じなら早期 return する)ので
/// チラつきの直接原因ではないが、無駄な再描画の連打であることに変わりはない
/// (発注書の容疑者2)。
const TOKENS_WATCH_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(80);

/// 受信を1回に束ねる**純粋なロジック**(テスト可能)。`is_significant` を
/// 満たす最初の1件をブロッキングで待ち(満たさない件は無視して待ち続ける —
/// notify のエラー event を通知扱いしない、という元の挙動を保つ)、その後
/// `window` 以内に来た追加分は種類を問わず全部飲み込んで捨てる。戻り値は
/// 「1回分の通知が来た」を表すだけで、束ねた個数は数えない(呼び出し側は
/// 「変わった」以上の情報を必要としない)。
///
/// 送信側が消えた(`Err(RecvError)`)場合は `None` — 監視終了のサイン。
fn debounce_recv<T>(
    rx: &std::sync::mpsc::Receiver<T>,
    window: std::time::Duration,
    is_significant: impl Fn(&T) -> bool,
) -> Option<()> {
    loop {
        let item = rx.recv().ok()?;
        if is_significant(&item) {
            break;
        }
    }
    while rx.recv_timeout(window).is_ok() {}
    Some(())
}

#[cfg(debug_assertions)]
fn watch_stream() -> impl iced::futures::Stream<Item = ()> {
    iced::stream::channel(
        8,
        |mut output: iced::futures::channel::mpsc::Sender<()>| async move {
            let dims_path = Dimensions::debug_source_path();
            let colors_path = Colors::debug_source_path();

            // notify の watcher は監視対象スレッドでコールバックを呼ぶ実装のため、
            // 受信は専用の OS スレッドへ逃がす(async executor を止めない)。
            // `try_send` は poll を要らないので、executor を挟まず同期コールバックから
            // 直接呼べる — 詰まっていたら単に取りこぼす(M16: 見張りが完璧でなくても
            // shell 自体は止めない)。
            std::thread::spawn(move || {
                use notify::Watcher;

                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher = match notify::recommended_watcher(tx) {
                    Ok(watcher) => watcher,
                    // 見張れなくても shell 自体は動く(M16)。token は起動時の値のまま。
                    Err(_) => return,
                };
                if watcher
                    .watch(&dims_path, notify::RecursiveMode::NonRecursive)
                    .is_err()
                {
                    return;
                }
                if watcher
                    .watch(&colors_path, notify::RecursiveMode::NonRecursive)
                    .is_err()
                {
                    return;
                }

                // **デバウンス**: 1回の保存が出す連続 raw event を1回の通知へ束ねる。
                // エラー event(`Result::Err`)は「変わった」の合図として扱わない
                // (元の実装の `if event.is_err() { continue; }` と同じ意味)。
                loop {
                    if debounce_recv(&rx, TOKENS_WATCH_DEBOUNCE, |event| event.is_ok()).is_none() {
                        // 送信側(watcher)が消えた = 監視を続けられない。
                        return;
                    }
                    if let Err(error) = output.try_send(()) {
                        // 詰まっているだけ(容量超過)なら次の束ねへ進めばよい。
                        // 受け手(Shell)がもう無い(disconnected)なら見張りを終える。
                        if error.is_disconnected() {
                            return;
                        }
                    }
                }
            });

            // 実際の送信は上の OS スレッドが行う。この Future 自体は消費されないまま
            // stream を生かしておくためだけに待ち続ける。
            std::future::pending::<()>().await;
        },
    )
}

#[cfg(test)]
mod debounce_tests {
    use super::debounce_recv;
    use std::time::Duration;

    /// **容疑者2の柵**: 1回の保存で notify が出す連続バーストを、1回の
    /// `debounce_recv` 呼び出しへ束ねる。束ねた後は channel が空になっている
    /// こと(=呼び出し側が2回目を呼んでも新しい通知が無い)まで確かめる。
    #[test]
    fn a_burst_of_events_collapses_into_one_notification() {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        // 保存直後の連続 raw event を模す(sleep なし — 実際のバーストと同じく
        // ほぼ同時に届く)。
        for _ in 0..5 {
            tx.send(()).unwrap();
        }

        let window = Duration::from_millis(50);
        let result = debounce_recv(&rx, window, |_| true);
        assert!(result.is_some(), "束ねた1回の通知が出ない");

        // 束ねた後は空 — 5件が5回の通知に化けていないことの直接証拠。
        assert!(
            rx.try_recv().is_err(),
            "バーストの一部が束ねられずに残っている(デバウンス欠如)"
        );
    }

    /// エラー扱いの event は「変わった」の合図にしない(notify のエラー通知を
    /// トークン再読込のトリガにしない、元の実装の意味を保つ)。
    #[test]
    fn insignificant_events_do_not_trigger_a_notification_on_their_own() {
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), ()>>();
        tx.send(Err(())).unwrap();
        tx.send(Err(())).unwrap();
        tx.send(Ok(())).unwrap();

        let window = Duration::from_millis(50);
        let result = debounce_recv(&rx, window, |event| event.is_ok());
        assert!(result.is_some(), "有効な event が来ているのに通知が出ない");
        assert!(
            rx.try_recv().is_err(),
            "Ok の後に残りが無いはず(全部1回へ束ねられているべき)"
        );
    }

    /// 送信側が消えたら `None` — 監視を終える合図として使える。
    #[test]
    fn a_closed_channel_yields_none() {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        drop(tx);
        assert!(debounce_recv(&rx, Duration::from_millis(10), |_| true).is_none());
    }
}

#[cfg(test)]
mod ui_scale_writeback_tests {
    use super::replace_ui_scale;

    const SAMPLE: &str = r#"{
  "row_height": 20,
  "_note_row_height": "note",

  "ui_scale": 1.00,
  "_note_ui_scale": "仮の置き場うんぬん、カンマも波括弧も含まない",

  "border_width": 1.0
}"#;

    /// **本命**: 値だけが変わり、直後の `_note_ui_scale`(コメント代わりのキー、
    /// `Dimensions` 構造体には存在しない)は1バイトも変わらない。
    #[test]
    fn replacing_ui_scale_changes_only_that_value() {
        let replaced = replace_ui_scale(SAMPLE, 1.5).expect("置換できない");
        assert!(
            replaced.contains("\"ui_scale\": 1.50"),
            "新しい値が入っていない: {replaced}"
        );
        assert!(
            replaced.contains("_note_ui_scale"),
            "note キーが消えている: {replaced}"
        );
        assert!(
            replaced.contains("\"row_height\": 20"),
            "無関係なキーまで変わっている: {replaced}"
        );
        assert!(
            replaced.contains("\"border_width\": 1.0"),
            "ui_scale より後ろのキーが壊れている: {replaced}"
        );
    }

    /// 最後のキー(直後が `}`)でも壊れない — `,` 前提の実装だと落ちる境界。
    #[test]
    fn replacing_the_last_key_before_the_closing_brace_still_works() {
        let json = r#"{"a": 1, "ui_scale": 1.0}"#;
        let replaced = replace_ui_scale(json, 2.0).expect("置換できない");
        assert_eq!(replaced, r#"{"a": 1, "ui_scale": 2.00}"#);
    }

    #[test]
    fn missing_key_is_a_clear_error_not_a_panic() {
        assert!(replace_ui_scale("{}", 1.0).is_err());
    }

    /// 書き戻した文字列は `Dimensions::parse` でそのまま読み直せて、他フィールドは
    /// 元の `Dimensions::default()` と一致する(構造としても壊れていない)。
    #[test]
    fn the_rewritten_text_round_trips_through_dimensions_parse() {
        use super::Dimensions;
        let full = r#"{"row_height": 20, "transport_band": 30, "title_text": 12,
            "body_text": 11, "caption_text": 9, "micro_text": 8,
            "spacing_xs": 2, "spacing_s": 4, "spacing_m": 8, "spacing_l": 12,
            "border_width": 1.0, "panel_header_height": 29,
            "inspector_panel_width": 496, "inspector_row_height": 20,
            "inspector_section_header_height": 26, "inspector_value_width": 38,
            "inspector_glyph_width": 18, "ui_scale": 1.0}"#;
        let replaced = replace_ui_scale(full, 1.75).expect("置換できない");
        let dims = Dimensions::parse(&replaced).expect("書き戻した JSON を読めない");
        assert_eq!(dims.ui_scale, 1.75);
        assert_eq!(dims.row_height, 20.0, "無関係なフィールドが壊れている");
    }
}

#[cfg(test)]
mod ui_scale_tests {
    use super::Dimensions;

    /// **適用点そのものの柵**: 100%(恒等)。`ui_scale: 1.0` は何も変えない —
    /// 変えてしまうと「掛けているのに1倍で変化が無い」ことすら保証できなくなる。
    #[test]
    fn scaling_by_one_is_the_identity() {
        let dims = Dimensions::default();
        let scaled = dims.scaled(1.0);
        assert_eq!(scaled, dims, "1.0倍で寸法が変わってしまっている");
    }

    /// 150%: mock(`--s: 1.50`)と同じ倍率。**罫線幅以外の全寸法・全文字サイズ**が
    /// 掛かること(発注書「全寸法・全文字サイズの読み出し口で乗算」)。
    #[test]
    fn scaling_by_one_point_five_multiplies_every_dimension_but_the_border() {
        let dims = Dimensions::default();
        let scaled = dims.scaled(1.5);

        assert_eq!(scaled.row_height, dims.row_height * 1.5);
        assert_eq!(scaled.transport_band, dims.transport_band * 1.5);
        assert_eq!(scaled.title_text, dims.title_text * 1.5);
        assert_eq!(scaled.body_text, dims.body_text * 1.5);
        assert_eq!(scaled.caption_text, dims.caption_text * 1.5);
        assert_eq!(scaled.micro_text, dims.micro_text * 1.5);
        assert_eq!(scaled.spacing_xs, dims.spacing_xs * 1.5);
        assert_eq!(scaled.spacing_s, dims.spacing_s * 1.5);
        assert_eq!(scaled.spacing_m, dims.spacing_m * 1.5);
        assert_eq!(scaled.spacing_l, dims.spacing_l * 1.5);
        assert_eq!(scaled.panel_header_height, dims.panel_header_height * 1.5);
        assert_eq!(
            scaled.inspector_panel_width,
            dims.inspector_panel_width * 1.5
        );
        assert_eq!(scaled.inspector_row_height, dims.inspector_row_height * 1.5);
        assert_eq!(
            scaled.inspector_section_header_height,
            dims.inspector_section_header_height * 1.5
        );
        assert_eq!(
            scaled.inspector_value_width,
            dims.inspector_value_width * 1.5
        );
        assert_eq!(
            scaled.inspector_glyph_width,
            dims.inspector_glyph_width * 1.5
        );
    }

    /// **罫線だけ物理1px床(クランプ)**: mock `--line: 1px` は `--s` の calc から
    /// 独立している(拡大しない)。150%でも罫線幅は1.0のまま。
    #[test]
    fn the_border_width_never_scales_past_its_one_pixel_floor() {
        let dims = Dimensions::default();
        assert_eq!(dims.scaled(1.5).border_width, 1.0);
        assert_eq!(dims.scaled(1.0).border_width, 1.0);
        // 設定側が万一1未満でも沈み込まない(防波堤)。
        let thin = Dimensions {
            border_width: 0.4,
            ..dims
        };
        assert_eq!(thin.scaled(1.0).border_width, 1.0);
        assert_eq!(thin.scaled(1.5).border_width, 1.0);
    }

    /// 正典バンド `{8,9,11,12}`(mock `ui-scale-and-z.html` の出典)が
    /// title>body>caption>micro の順序を保ったまま tokens に入っていること。
    #[test]
    fn the_canonical_type_band_matches_the_mock() {
        let dims = Dimensions::default();
        assert_eq!(dims.title_text, 12.0);
        assert_eq!(dims.body_text, 11.0);
        assert_eq!(dims.caption_text, 9.0);
        assert_eq!(dims.micro_text, 8.0);
    }

    /// `--section`(26)が Inspector の section 高そのものであること
    /// (発注書「section 高 26」)。
    #[test]
    fn the_inspector_section_height_matches_the_mock() {
        assert_eq!(Dimensions::default().inspector_section_header_height, 26.0);
    }
}

#[cfg(test)]
mod text_weight_and_ink_tests {
    use super::{Colors, Ink, TextWeight};

    /// 裁定137の3段(400/600/800)が `iced::font::Weight` の対応する段に
    /// 正しく写ること。CSS の値そのものが変数名に現れる上流列挙
    /// (`Normal`=400/`Semibold`=600/`ExtraBold`=800)へ1:1で繋がっているかの柵。
    #[test]
    fn text_weight_maps_to_the_canonical_css_bands() {
        assert_eq!(
            TextWeight::Regular.font().weight,
            iced::font::Weight::Normal
        );
        assert_eq!(
            TextWeight::Semibold.font().weight,
            iced::font::Weight::Semibold
        );
        assert_eq!(
            TextWeight::Bold.font().weight,
            iced::font::Weight::ExtraBold
        );
    }

    /// ink 3段は既存 `Colors::text_*` をそのまま返すだけ(新色を発明しない、
    /// 裁定139)。
    #[test]
    fn ink_resolves_to_the_existing_colors_without_inventing_new_ones() {
        let colors = Colors::default();
        assert_eq!(Ink::Primary.resolve(&colors), colors.text_primary);
        assert_eq!(Ink::Secondary.resolve(&colors), colors.text_secondary);
        assert_eq!(Ink::Muted.resolve(&colors), colors.text_muted);
    }
}

#[cfg(test)]
mod hairline_and_rhythm_wash_tests {
    use super::Colors;

    /// **本命**: `border_hairline_weak`(旧 `inspector_pane::PROW_HAIRLINE`)が
    /// tokens 側へ昇格していて、`Default`/`parse` の両経路で同じ黒 35% wash に
    /// なること(裁定142 の先行整備 — 2箇所で別の値を発明しない)。
    #[test]
    fn border_hairline_weak_is_a_black_alpha_35_wash() {
        let colors = Colors::default();
        assert_eq!(colors.border_hairline_weak.r, 0.0);
        assert_eq!(colors.border_hairline_weak.g, 0.0);
        assert_eq!(colors.border_hairline_weak.b, 0.0);
        assert!((colors.border_hairline_weak.a - 0.35).abs() < 1e-6);
    }

    /// Timeline 明暗リズム(裁定148)の2ロールは、hairline と混同しない**白**の
    /// 薄い wash であること(区切り=黒hairline、リズム=白wash — 見て区別が
    /// つく、§1.6 の両立整理)。
    #[test]
    fn timeline_rhythm_washes_are_white_and_distinct_from_the_hairline() {
        let colors = Colors::default();
        for wash in [colors.timeline_time_band, colors.timeline_row_zebra] {
            assert_eq!(wash.r, 1.0);
            assert_eq!(wash.g, 1.0);
            assert_eq!(wash.b, 1.0);
            assert!(
                wash.a > 0.0 && wash.a < 0.35,
                "hairline と紛れない薄さのはず"
            );
        }
    }

    /// **本命(σ EXACT TARGET 1)**: `timeline_grid_minor`/`timeline_grid_major`
    /// は mock `timeline-semantics.html` の実測値どおり黒 α0.18/0.30 で、
    /// 大目盛の方が小目盛より強い(「わずかに強い確認線」)。
    #[test]
    fn timeline_grid_roles_are_black_with_the_mock_alphas() {
        let colors = Colors::default();
        for grid in [colors.timeline_grid_minor, colors.timeline_grid_major] {
            assert_eq!(grid.r, 0.0);
            assert_eq!(grid.g, 0.0);
            assert_eq!(grid.b, 0.0);
        }
        assert!((colors.timeline_grid_minor.a - 0.18).abs() < 1e-6);
        assert!((colors.timeline_grid_major.a - 0.30).abs() < 1e-6);
        assert!(
            colors.timeline_grid_major.a > colors.timeline_grid_minor.a,
            "大目盛の縦線は小目盛よりわずかに強いはず(帯の境界の確認線)"
        );
    }

    /// `timeline_grid_major` は `border_hairline_weak`(区切りロール、α0.35)
    /// より弱い — 縦線は「区切り」ではなく「時間の細かいリズムの投影」
    /// なので、同じ黒でも区切り hairline より控えめであるべき(mock 実測値
    /// 0.30 < 0.35 の関係を固定する)。
    #[test]
    fn timeline_grid_major_is_weaker_than_the_hairline_separator() {
        let colors = Colors::default();
        assert!(colors.timeline_grid_major.a < colors.border_hairline_weak.a);
    }
}

#[cfg(test)]
mod checkerboard_role_tests {
    use super::Colors;

    /// **市松v2 の本命(ORACLE (a))**: `checkerboard_light`/`checkerboard_dark`
    /// の2色コントラスト差が Δ≥24/255(単純差、WCAG式ではなく合図の視認性)。
    /// 旧実装(`surface_raised`/`surface_panel` の借用)は Δ≈8/255 で
    /// supervisor 実測「実質不可視」だった — 専用ロールが実際にその根治に
    /// なっていることをここで固定する。
    #[test]
    fn checkerboard_tiles_have_a_visible_contrast_delta() {
        let colors = Colors::default();
        let to_255 = |c: f32| (c * 255.0).round();
        let light = to_255(colors.checkerboard_light.r);
        let dark = to_255(colors.checkerboard_dark.r);
        assert_eq!(colors.checkerboard_light.r, colors.checkerboard_light.g);
        assert_eq!(colors.checkerboard_light.g, colors.checkerboard_light.b);
        assert_eq!(colors.checkerboard_dark.r, colors.checkerboard_dark.g);
        assert_eq!(colors.checkerboard_dark.g, colors.checkerboard_dark.b);
        assert!(
            (light - dark).abs() >= 24.0,
            "市松2色のコントラストが弱すぎる(旧根因の再発): light={light}, dark={dark}, Δ={}",
            (light - dark).abs()
        );
    }

    /// `checkerboard_light`/`checkerboard_dark` は `surface_raised`/
    /// `surface_panel`(パネル面ロール)の値をそのまま指してはいない —
    /// 借用ではなく独立した新ロールであることの直接証拠(裁定164 S4)。
    #[test]
    fn checkerboard_colors_are_independent_from_the_surface_roles() {
        let colors = Colors::default();
        assert_ne!(colors.checkerboard_light, colors.surface_raised);
        assert_ne!(colors.checkerboard_light, colors.surface_panel);
        assert_ne!(colors.checkerboard_dark, colors.surface_raised);
        assert_ne!(colors.checkerboard_dark, colors.surface_panel);
    }

    /// `Default`/`parse` の両経路が同じ固定値を使うこと(`fixed_wash_colors`
    /// と同型の柵、正本 JSON にロールが無いので値の2重管理を許さない)。
    #[test]
    fn checkerboard_colors_match_between_default_and_parse() {
        let default_colors = Colors::default();
        let json = std::fs::read_to_string(Colors::debug_source_path())
            .expect("motolii-dark.json を読めない");
        let parsed = Colors::parse(&json).expect("motolii-dark.json を parse できない");
        assert_eq!(parsed.checkerboard_light, default_colors.checkerboard_light);
        assert_eq!(parsed.checkerboard_dark, default_colors.checkerboard_dark);
    }
}

#[cfg(test)]
mod label_palette_tests {
    use super::{hsl_to_rgb, Colors, LABEL_PALETTE_LEN};

    /// 12色ちょうど(発注書指定)。
    #[test]
    fn label_palette_has_twelve_entries() {
        assert_eq!(Colors::default().label_palette.len(), LABEL_PALETTE_LEN);
        assert_eq!(LABEL_PALETTE_LEN, 12);
    }

    /// hue は index * 30° の昇順 — 隣接色が単純な HSL 数式で予測できること
    /// (発明した並びではなく hue 昇順という機械的な規則であることの直接証拠)。
    #[test]
    fn label_palette_hues_ascend_in_thirty_degree_steps() {
        let palette = Colors::default().label_palette;
        for (index, color) in palette.iter().enumerate() {
            let expected = hsl_to_rgb(index as f32 * 30.0, 0.32, 0.62);
            assert_eq!(
                *color,
                expected,
                "index {index} の色が hue={}° の HSL 変換と一致しない",
                index as f32 * 30.0
            );
        }
    }

    /// hue=0° (赤)の HSL→RGB を教科書どおりの数値で固定する — 変換式自体の
    /// 正しさを、パレット生成とは独立に確かめる。S=0.32, L=0.62 の hue=0° は
    /// C=(1-|2*0.62-1|)*0.32=0.2432、m=0.62-C/2=0.4984 なので
    /// R=C+m=0.7416, G=B=m=0.4984。
    #[test]
    fn hsl_to_rgb_matches_the_textbook_formula_at_hue_zero() {
        let color = hsl_to_rgb(0.0, 0.32, 0.62);
        assert!((color.r - 0.7416).abs() < 1e-4, "R={}", color.r);
        assert!((color.g - 0.4984).abs() < 1e-4, "G={}", color.g);
        assert!((color.b - 0.4984).abs() < 1e-4, "B={}", color.b);
    }

    /// hue=120°(緑)でも同じ式が成り立つこと — 分岐(`match h_prime as i32`)の
    /// 別セグメントも検分する。C/m は hue=0° と同じ(S/L 固定なので不変)、
    /// G=C+m=0.7416, R=B=m=0.4984。
    #[test]
    fn hsl_to_rgb_matches_the_textbook_formula_at_hue_120() {
        let color = hsl_to_rgb(120.0, 0.32, 0.62);
        assert!((color.r - 0.4984).abs() < 1e-4, "R={}", color.r);
        assert!((color.g - 0.7416).abs() < 1e-4, "G={}", color.g);
        assert!((color.b - 0.4984).abs() < 1e-4, "B={}", color.b);
    }

    /// `Default`/`parse` の両経路が同じ固定パレットを使うこと(`checkerboard_*`
    /// と同型の柵 — 正本 JSON にロールが無いので値の2重管理を許さない)。
    #[test]
    fn label_palette_matches_between_default_and_parse() {
        let default_colors = Colors::default();
        let json = std::fs::read_to_string(Colors::debug_source_path())
            .expect("motolii-dark.json を読めない");
        let parsed = Colors::parse(&json).expect("motolii-dark.json を parse できない");
        assert_eq!(parsed.label_palette, default_colors.label_palette);
    }

    /// 12色すべてが異なる(同じ hue に潰れていないことの直接証拠)。
    #[test]
    fn label_palette_entries_are_all_distinct() {
        let palette = Colors::default().label_palette;
        for i in 0..palette.len() {
            for j in (i + 1)..palette.len() {
                assert_ne!(palette[i], palette[j], "index {i} と {j} の色が同じ");
            }
        }
    }
}
