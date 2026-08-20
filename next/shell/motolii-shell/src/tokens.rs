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
    /// (旧 `docs/mocks-ui/public/inspector-library.css` `.inspectorShell { width:
    /// min(100%, 496px) }`)。**496 のまま据え置き**(300 への変更は利用者裁定待ち、
    /// CANON 記載)— 新 mock(`ui-scale-and-z.html`)の `--pane: 300` はこの pane 幅の
    /// 出典として採らない。
    pub inspector_panel_width: f32,
    /// Inspector property 行 / column header 行の高さ。出典: 視覚正本
    /// `ui-scale-and-z.html` `--row`(20)。旧値25(旧 CSS `.propertyRow
    /// { min-height: 25px }` 由来)から新 mock へ更新。
    pub inspector_row_height: f32,
    /// Inspector の `--section`(26)。**2箇所で共有**: panel タイトル帯(`.ptitle`、
    /// 旧実装は `panel_header_height` を誤用していた)と section 見出し
    /// (TRANSFORM/APPEARANCE、`.sec`)。mock 自身がこの2つに同じ変数を使っている
    /// ので、token も1本のまま両方へ渡す。
    pub inspector_section_header_height: f32,
    /// Inspector 値セル(X/Y/Z)1つぶんの幅。出典: 視覚正本の
    /// `grid-template-columns: 1fr repeat(3, 38px) hit` の `38px` 段。
    /// 旧値64(旧 CSS 由来)から新 mock へ更新。
    pub inspector_value_width: f32,
    /// Inspector の Key/M/S glyph 列の幅。出典: 視覚正本 `--hit`(18)。
    /// **Key 列自体は空のまま**(Q0: keyframe UI 未実装、列幅の予約だけ)。
    #[serde(default = "default_inspector_glyph_width")]
    pub inspector_glyph_width: f32,
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
}

fn default_inspector_glyph_width() -> f32 {
    18.0
}

fn default_ui_scale() -> f32 {
    1.0
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
            inspector_row_height: 20.0,
            inspector_section_header_height: 26.0,
            inspector_value_width: 38.0,
            inspector_glyph_width: 18.0,
            ui_scale: 1.0,
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
        let surface_raised = Color::from_rgb(0.1333, 0.1333, 0.1333);
        let surface_panel = Color::from_rgb(0.1020, 0.1020, 0.1020);
        let text_muted = Color::from_rgb(0.5725, 0.5725, 0.5725);
        let action_active = Color::from_rgb(0.8471, 0.7098, 0.4549);
        let (state_selected, state_disabled) =
            derive_state_colors(surface_raised, surface_panel, text_muted, action_active);
        Self {
            surface_app: Color::from_rgb(0.0784, 0.0784, 0.0784),
            surface_panel,
            surface_raised,
            surface_hover: Color::from_rgb(0.1725, 0.1725, 0.1725),
            border_default: Color::from_rgb(0.2314, 0.2314, 0.2314),
            border_strong: Color::from_rgb(0.4078, 0.4078, 0.4078),
            text_primary: Color::from_rgb(0.9412, 0.9412, 0.9412),
            text_secondary: Color::from_rgb(0.7765, 0.7765, 0.7765),
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
    let key_pos = json.find(key).ok_or_else(|| "ui_scale キーが無い".to_owned())?;
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
        assert_eq!(scaled.inspector_panel_width, dims.inspector_panel_width * 1.5);
        assert_eq!(scaled.inspector_row_height, dims.inspector_row_height * 1.5);
        assert_eq!(
            scaled.inspector_section_header_height,
            dims.inspector_section_header_height * 1.5
        );
        assert_eq!(scaled.inspector_value_width, dims.inspector_value_width * 1.5);
        assert_eq!(scaled.inspector_glyph_width, dims.inspector_glyph_width * 1.5);
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
