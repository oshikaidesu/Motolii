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
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
pub struct Dimensions {
    /// Timeline の行高。
    pub row_height: f32,
    /// transport/Control 帯の高さ。
    pub transport_band: f32,
    /// 本文相当の文字サイズ。
    pub body_text: f32,
    /// 小さめの文字サイズ(caption 相当)。
    pub small_text: f32,
}

impl Default for Dimensions {
    fn default() -> Self {
        // ファイルが読めない・壊れている時の最終防波堤(M16: render 失敗でも画面を
        // 空にしない、と同じ理由)。値は正本 JSON と同じ Ableton 実測 — 正本が2つに
        // 増えるわけではなく、正本を読めなかった時だけ使う既定値。
        Self {
            row_height: 20.0,
            transport_band: 30.0,
            body_text: 13.0,
            small_text: 11.0,
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors {
    pub surface_app: Color,
    pub surface_panel: Color,
    pub surface_raised: Color,
    pub surface_hover: Color,
    pub border_default: Color,
    pub border_strong: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub focus: Color,
    pub action_active: Color,
    pub data: Color,
    pub shape: Color,
    pub status_warning: Color,
    pub status_ok: Color,
    /// Timeline pane 専用のアクセント色(`way.timeline`)。他 pane も同族の `way.*`
    /// を同じ要領で足せる。
    pub way_timeline: Color,
}

impl Default for Colors {
    fn default() -> Self {
        // Dimensions と同じ理由の最終防波堤。数値は motolii-dark.json のスナップショットだが
        // **正本はあくまで JSON 側**(読めた時は常にそちらを使う)。
        Self {
            surface_app: Color::from_rgb(0.0784, 0.0784, 0.0784),
            surface_panel: Color::from_rgb(0.1020, 0.1020, 0.1020),
            surface_raised: Color::from_rgb(0.1333, 0.1333, 0.1333),
            surface_hover: Color::from_rgb(0.1725, 0.1725, 0.1725),
            border_default: Color::from_rgb(0.2314, 0.2314, 0.2314),
            border_strong: Color::from_rgb(0.4078, 0.4078, 0.4078),
            text_primary: Color::from_rgb(0.9412, 0.9412, 0.9412),
            text_secondary: Color::from_rgb(0.7765, 0.7765, 0.7765),
            text_muted: Color::from_rgb(0.5725, 0.5725, 0.5725),
            focus: Color::from_rgb(0.9412, 0.9412, 0.9412),
            action_active: Color::from_rgb(0.8471, 0.7098, 0.4549),
            data: Color::from_rgb(0.4706, 0.7098, 0.6902),
            shape: Color::from_rgb(0.6667, 0.6275, 0.8157),
            status_warning: Color::from_rgb(0.8824, 0.5412, 0.4275),
            status_ok: Color::from_rgb(0.5647, 0.6980, 0.5294),
            way_timeline: Color::from_rgb(0.8000, 0.5843, 0.5294),
        }
    }
}

impl Colors {
    pub fn parse(json: &str) -> Result<Self, String> {
        let root: serde_json::Value =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        let color = root.get("color").ok_or("color 節が無い")?;
        Ok(Self {
            surface_app: color_at(color, &["surface", "app"])?,
            surface_panel: color_at(color, &["surface", "panel"])?,
            surface_raised: color_at(color, &["surface", "raised"])?,
            surface_hover: color_at(color, &["surface", "hover"])?,
            border_default: color_at(color, &["border", "default"])?,
            border_strong: color_at(color, &["border", "strong"])?,
            text_primary: color_at(color, &["text", "primary"])?,
            text_secondary: color_at(color, &["text", "secondary"])?,
            text_muted: color_at(color, &["text", "muted"])?,
            focus: color_at(color, &["focus"])?,
            action_active: color_at(color, &["action", "active"])?,
            data: color_at(color, &["data"])?,
            shape: color_at(color, &["shape"])?,
            status_warning: color_at(color, &["status", "warning"])?,
            status_ok: color_at(color, &["status", "ok"])?,
            way_timeline: color_at(color, &["way", "timeline"])?,
        })
    }

    /// debug ビルドでの読み込み元。**正本は1つ** — `ui/motolii-tokens` 配下のこのファイル
    /// をそのまま読み、コピーは作らない。
    pub fn debug_source_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ui/motolii-tokens/sources/motolii-dark.json")
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
            let dims = Dimensions::load_from_path(&Dimensions::debug_source_path())
                .unwrap_or_default();
            let colors =
                Colors::load_from_path(&Colors::debug_source_path()).unwrap_or_default();
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

#[cfg(debug_assertions)]
fn watch_stream() -> impl iced::futures::Stream<Item = ()> {
    iced::stream::channel(8, |mut output: iced::futures::channel::mpsc::Sender<()>| async move {
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

            for event in rx {
                if event.is_err() {
                    continue;
                }
                if let Err(error) = output.try_send(()) {
                    // 詰まっているだけ(容量超過)なら次の event を待てばよい。
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
    })
}
