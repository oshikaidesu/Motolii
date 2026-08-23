//! 寸法トークン(`Dimensions`)。値の正本は `tokens/dimensions.json` — ここは複製しない。
//! `lib.rs` から分割(SP-8、中身は移送のみ)。

use std::path::{Path, PathBuf};

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
    /// pane_grid の題帯(各 pane 上端の pane 名入り drag ハンドル、2026-08-22
    /// 題帯レーン)の帯高。導出は `tokens/dimensions.json` の
    /// `_note_pane_header_height` 参照 — 帯内文字 `micro_text`(8)を裁定168 の
    /// 文字/帯 比率帯(0.42±0.05)へ入れる高さ(8/18=0.444、
    /// `inspector_row_height` の 11/25=0.44 と同じ導出形)。左右 padding は
    /// `spacing_m` を消費する(専用キーは起こさない)。
    #[serde(default = "default_pane_header_height")]
    pub pane_header_height: f32,
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
    /// Timeline pane 上端の transport 帯の高さ(発注 2026-08-22: 「タイムライン
    /// に再生ボタンが無いのは謎」— map 1041-1045/1138 の顔)。値は
    /// `transport_band`(Ableton Control Bar 実測30)と同源 — 帯の意味が同一
    /// なので同じ実測を写す。別キーな理由は `_note_timeline_transport_height`
    /// (朝の合否で pane 内 transport だけを独立に直せるように)。
    #[serde(default = "default_timeline_transport_height")]
    pub timeline_transport_height: f32,
    /// transport ボタン踏面の幅。高さは帯高いっぱい(S1)なので幅は帯高と
    /// 同値の正方形踏面(裁定167 の梯子は余白の梯子で踏面の段を持たない —
    /// 中間比を発明せず基準寸そのものを採る、`_note_*` 参照)。
    #[serde(default = "default_timeline_transport_button_width")]
    pub timeline_transport_button_width: f32,
    /// transport のボタン間・タイムコードとの間隔。裁定167 梯子下段
    /// `0.075×帯高30 = 2.25 → 2`(`lane_bar::sibling_gap_px` と同式・同段)。
    #[serde(default = "default_timeline_transport_gap")]
    pub timeline_transport_gap: f32,
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
    /// Browser 第5切片(List 表示の水平カード)。list mode の thumb 幅。出典:
    /// 視覚正本 `browser-library.css:306` `.libraryBrowser[data-view="list"]
    /// .libraryThumb{width:46px;flex:0 0 46px}` 実測(`_note_browser_list_
    /// thumb_width` 参照)。高さは呼び手側(`motolii-browser-pane`)が既存の
    /// `THUMB_ASPECT_W`/`THUMB_ASPECT_H`(16/9)をこの幅へ適用して逆算する —
    /// 専用の高さキーは起こさない。
    #[serde(default = "default_browser_list_thumb_width")]
    pub browser_list_thumb_width: f32,
    /// メニューバーの開いた menu 面の幅(menubar 節 — MB-2 上書き裁定 2026-08-22、
    /// `motolii-menubar` crate が読む)。新規実測は無い — 既存 shell `menu.rs` の
    /// 算術合成 `inspector_value_width * 3.0`(64×3=192)をトークンへ採番しただけ
    /// (`tokens/dimensions.json` の `_note_menubar_menu_width` 参照)。
    #[serde(default = "default_menubar_menu_width")]
    pub menubar_menu_width: f32,
    /// 枠の文法(裁定179)「開いた menu の面は明度1段+角丸」の角丸半径。
    /// 実測正本に対応段が無いので `spacing_s`(4)と同値
    /// (`_note_menubar_corner_radius` 参照 — 新しい寸法段を発明しない)。
    #[serde(default = "default_menubar_corner_radius")]
    pub menubar_corner_radius: f32,
    /// gizmo 節(Stage ギズモ第1弾発注 2026-08-22)。bbox の8ハンドル(正方形)の
    /// 1辺と回転ハンドル(円)の直径。AE/Figma のハンドル実寸帯(6〜8px)の上端 —
    /// `spacing_m` と同値だが意味は「掴む踏面」(`_note_gizmo_handle_size` 参照)。
    #[serde(default = "default_gizmo_handle_size")]
    pub gizmo_handle_size: f32,
    /// ギズモのハンドル命中半径(screen px)。見た目より広い判定側の遊びで
    /// Q0「見えている物は必ず触れる」を保証する(`_note_gizmo_hit_radius`)。
    #[serde(default = "default_gizmo_hit_radius")]
    pub gizmo_hit_radius: f32,
    /// 回転ハンドルの、bbox 上辺中点から外側への距離。`gizmo_handle_size × 2`
    /// (Top ハンドルと判定が重ならない最小段、`_note_gizmo_rotate_offset`)。
    #[serde(default = "default_gizmo_rotate_offset")]
    pub gizmo_rotate_offset: f32,
    /// anchor 表示(⊕)の円半径。ハンドルより一段小さい視覚重量
    /// (`_note_gizmo_anchor_radius`)。
    #[serde(default = "default_gizmo_anchor_radius")]
    pub gizmo_anchor_radius: f32,
}

fn default_inspector_glyph_width() -> f32 {
    26.0
}

fn default_pane_header_height() -> f32 {
    18.0
}

fn default_timeline_lane_bar_width() -> f32 {
    150.0
}

fn default_timeline_param_row_height() -> f32 {
    16.67
}

fn default_timeline_transport_height() -> f32 {
    30.0
}

fn default_timeline_transport_button_width() -> f32 {
    30.0
}

fn default_timeline_transport_gap() -> f32 {
    2.0
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

fn default_browser_list_thumb_width() -> f32 {
    46.0
}

fn default_menubar_menu_width() -> f32 {
    192.0
}

fn default_menubar_corner_radius() -> f32 {
    4.0
}

fn default_gizmo_handle_size() -> f32 {
    8.0
}

fn default_gizmo_hit_radius() -> f32 {
    8.0
}

fn default_gizmo_rotate_offset() -> f32 {
    16.0
}

fn default_gizmo_anchor_radius() -> f32 {
    4.0
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
            pane_header_height: 18.0,
            timeline_lane_bar_width: 150.0,
            timeline_param_row_height: 16.67,
            timeline_transport_height: 30.0,
            timeline_transport_button_width: 30.0,
            timeline_transport_gap: 2.0,
            ui_scale: 1.0,
            browser_tab_bar_height: 26.0,
            browser_tab_underline: 2.0,
            browser_list_thumb_width: 46.0,
            menubar_menu_width: 192.0,
            menubar_corner_radius: 4.0,
            gizmo_handle_size: 8.0,
            gizmo_hit_radius: 8.0,
            gizmo_rotate_offset: 16.0,
            gizmo_anchor_radius: 4.0,
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
            pane_header_height: self.pane_header_height * s,
            timeline_lane_bar_width: self.timeline_lane_bar_width * s,
            timeline_param_row_height: self.timeline_param_row_height * s,
            browser_tab_bar_height: self.browser_tab_bar_height * s,
            browser_tab_underline: self.browser_tab_underline * s,
            browser_list_thumb_width: self.browser_list_thumb_width * s,
            timeline_transport_height: self.timeline_transport_height * s,
            timeline_transport_button_width: self.timeline_transport_button_width * s,
            timeline_transport_gap: self.timeline_transport_gap * s,
            menubar_menu_width: self.menubar_menu_width * s,
            menubar_corner_radius: self.menubar_corner_radius * s,
            gizmo_handle_size: self.gizmo_handle_size * s,
            gizmo_hit_radius: self.gizmo_hit_radius * s,
            gizmo_rotate_offset: self.gizmo_rotate_offset * s,
            gizmo_anchor_radius: self.gizmo_anchor_radius * s,
            // 自分自身は「寸法」ではないので掛けない。この結果を再度 `scaled()`
            // に通す呼び出し側は無い(適用点は `Shell::dims` の1箇所だけ)。
            ui_scale: self.ui_scale,
        }
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

