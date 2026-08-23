//! 色トークン(`Colors`)・色ロールの導出(`derive_state_colors` 等)・
//! `iced::Theme` への変換(`theme_from_colors`)。値の正本は
//! `ui/motolii-tokens/sources/motolii-dark.json` — ここは複製しない。
//! `lib.rs` から分割(SP-8、中身は移送のみ)。

use std::path::{Path, PathBuf};

use iced::Color;

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
