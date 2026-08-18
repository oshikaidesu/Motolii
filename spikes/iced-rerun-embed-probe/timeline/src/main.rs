//! iced で仮のタイムライン UI を作る spike。
//!
//! 採用の判断はしない。測るのは2つだけ:
//!   (1) 利用者が窓で触れる現物があること
//!   (2) 同じ4ジェスチャ(移動・トリム・スクラブ・zoom)を Elm 構造で書くと
//!       どんな形になり、egui 版とどこが違うか
//!
//! 起動:
//!   cargo run -p iced-timeline-probe
//!   cargo run -p iced-timeline-probe -- --clips 500

mod app;
mod message;
mod model;
mod program;
mod view;

// テスト名は日本語のまま読める方が速いので、命名規約の警告だけ外す。
#[cfg(test)]
#[allow(non_snake_case)]
mod tests;

use iced::widget::canvas::Canvas;
use iced::{Element, Fill, Subscription, Theme};

use app::App;
use message::TimelineMsg;
use program::TimelineProgram;

/// 窓の初期サイズ。`App::new` が「全部入る zoom」を出すのにも使う。
pub const WINDOW_W: f32 = 1280.0;
pub const WINDOW_H: f32 = 520.0;

fn main() -> iced::Result {
    let clips = parse_clips(std::env::args().skip(1));

    iced::application(move || App::new(clips), App::update, view)
        .title(App::title)
        .subscription(subscription)
        .theme(theme)
        .window_size((WINDOW_W, WINDOW_H))
        .run()
}

/// **クロージャで書くと通らない。**
///
/// `|app: &App| -> Element<'_, _>` はクロージャの推論が高階の生存期間
/// (`for<'a> Fn(&'a App) -> Element<'a, _>`)を作れず、
/// `implementation of ViewFn is not general enough` で落ちる。
/// `fn` 項目にすると HRTB が付いて通る。
/// (詰まった箇所その4。iced の責任というより Rust のクロージャ推論の話だが、
///  `iced::application(state, update, view)` という API がクロージャを
///  受け取れる顔をしているので、素直に書くと必ず1回踏む。)
fn view(app: &App) -> Element<'_, TimelineMsg> {
    // 密な面は canvas 1ノードのまま。widget へ分解しない
    // (製品の P8 実測と同じ方針。分解した瞬間に比較対象が変わる)。
    Canvas::new(TimelineProgram { app })
        .width(Fill)
        .height(Fill)
        .into()
}

fn theme(_app: &App) -> Theme {
    Theme::Nord
}

/// **canvas だけでは足りなかった2本**。
///
/// - `ModifiersChanged`: `mouse::Event::WheelScrolled` が modifiers を運ばないので、
///   Cmd+ホイールを判定するには修飾キーの現在値を別で持つしかない。
///   canvas の `Program::State` に持つ手もあったが、それだと修飾キーだけが
///   message を通らない裏道になるので、ここで拾って model に入れる。
/// - `frames()`: フレーム間隔の実測。描画時刻を message として受け取る。
fn subscription(_app: &App) -> Subscription<TimelineMsg> {
    Subscription::batch([
        // `listen_with` は canvas が capture したイベントも `Status` 付きで見せてくれる。
        // `ModifiersChanged` は canvas が capture しないので素通しで届く。
        iced::event::listen_with(|event, _status, _window| match event {
            iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m)) => {
                Some(TimelineMsg::ModifiersChanged(m))
            }
            // 矢印キーの Shift は `KeyPressed` が自分で運ぶので、ここでは拾わない。
            _ => None,
        }),
        iced::window::frames().map(TimelineMsg::Rendered),
    ])
}

/// `--clips N`。無ければ 24。
fn parse_clips(args: impl Iterator<Item = String>) -> usize {
    let args: Vec<String> = args.collect();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--clips" {
            if let Some(n) = args.get(i + 1).and_then(|s| s.parse::<usize>().ok()) {
                return n.max(1);
            }
        }
        if let Some(rest) = args[i].strip_prefix("--clips=") {
            if let Ok(n) = rest.parse::<usize>() {
                return n.max(1);
            }
        }
        i += 1;
    }
    24
}
