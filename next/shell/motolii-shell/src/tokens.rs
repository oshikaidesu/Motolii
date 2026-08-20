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
    /// type scale: panel header・section title 相当。
    pub title_text: f32,
    /// type scale: 本文相当の文字サイズ。
    pub body_text: f32,
    /// type scale: 小さめの文字サイズ(caption 相当)。旧 `small_text`
    /// (型スケール語彙 title/body/caption/micro への統一、2026-08-20)。
    pub caption_text: f32,
    /// type scale: 最小の可読文字サイズ(status 帯の脇役文言等)。
    pub micro_text: f32,
    /// spacing scale の最小段。
    pub spacing_xs: f32,
    /// spacing scale の小段。
    pub spacing_s: f32,
    /// spacing scale の中段。
    pub spacing_m: f32,
    /// spacing scale の大段。
    pub spacing_l: f32,
    /// 罫線幅(ui-visual-language: フラット・細罫線)。
    pub border_width: f32,
    /// panel header 帯の高さ。
    pub panel_header_height: f32,
    /// Inspector pane の固定幅。出典は Ableton 実測ではなく**視覚正本 HTML/CSS 自体**
    /// (`docs/mocks-ui/public/inspector-library.css` `.inspectorShell { width:
    /// min(100%, 496px) }`)— Inspector 第1波の発注書が正本として名指ししたのは
    /// この HTML/CSS そのものなので、他寸法と違い Ableton 密度表ではなくここから写す。
    pub inspector_panel_width: f32,
    /// Inspector property 行の高さ。出典: 同 CSS `.propertyRow { min-height: 25px }`。
    pub inspector_row_height: f32,
    /// Inspector section 見出し(TRANSFORM/ATTRS)の高さ。出典: 同 CSS
    /// `.tableSection h2 { height: 26px }`。
    pub inspector_section_header_height: f32,
    /// Inspector 値セル(X/Y/Z 等)の幅。出典: 同 CSS `.propertyRow` の
    /// `grid-template-columns: minmax(132px, 1fr) repeat(3, 64px) 26px` の `64px` 段。
    pub inspector_value_width: f32,
    /// Inspector 行のラベル列(Property 名)の幅。出典: 同 CSS の
    /// `grid-template-columns` 先頭 `minmax(132px, 1fr)` の `132px`。
    pub inspector_label_width: f32,
    /// Inspector 選択サマリ帯の高さ。出典: 同 CSS `.selectionSummary { height: 46px }`。
    pub inspector_summary_height: f32,
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
            title_text: 15.0,
            body_text: 13.0,
            caption_text: 11.0,
            micro_text: 9.0,
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
            inspector_label_width: 132.0,
            inspector_summary_height: 46.0,
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
}

impl Default for Tokens {
    fn default() -> Self {
        Self {
            dims: Dimensions::default(),
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
            Self { dims, colors }
        }
        #[cfg(not(debug_assertions))]
        {
            Self {
                dims: Dimensions::parse(DIMENSIONS_JSON).unwrap_or_default(),
                colors: Colors::parse(COLOR_TOKENS_JSON).unwrap_or_default(),
            }
        }
    }
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
