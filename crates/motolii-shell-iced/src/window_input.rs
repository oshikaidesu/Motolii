//! 窓ぜんたいが受ける入力 — 近道キー・OS ドロップ・閉じる要求。
//!
//! egui shell はこれを `handle_file_entry(ctx)` として「描く前に1回」通していた。
//! iced に `ctx` は無く、生の事象は **widget 木の根**へ流れてくる
//! (`iced_runtime::UserInterface::update` が全ての `Event` を根へ渡す)。そこで
//! 窓ぜんたいを包む widget を1枚だけ置き、そこで [`Message`] へ写す。
//!
//! ## なぜ subscription ではないのか
//!
//! iced の慣行としては `keyboard::listen` / `window::events()` の購読でも書ける。
//! **しかし購読は `iced_test::Simulator` の外に居る** — 運転席から近道キーも
//! ドロップも注入できなくなり、2026-08-18 裁定が乗り換えの根拠にした
//! 「段差ゼロが公式不変量」がこの2つだけ成り立たなくなる。木の中に置けば、
//! 窓が受ける事象と運転席が注入する事象が**同じ1本の道**を通る。
//!
//! (例外は [`Message::ExportPolled`] だけ。あれは入力ではなく thread からの
//! 返事なので、窓では `window::frames()` の購読が運ぶ。)
//!
//! ## ドロップは「フレーム1回」にまとめる
//!
//! winit は**ファイル1つにつき1事象**を出すが、egui は1フレームぶんの
//! `dropped_files` をまとめて渡す。そのままだと「3本まとめて落とす」の帯の
//! 言い方が host で変わってしまうので、フレームの区切り
//! (`window::Event::RedrawRequested`。窓は毎フレームこれを木へ流す)まで path を
//! 溜めてから1つの [`Message::FilesDropped`] にする。

use std::path::PathBuf;

use iced::advanced::widget::{tree, Operation, Tree};
use iced::advanced::{layout, mouse, overlay, renderer, Layout, Shell as IcedShell, Widget};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

use crate::message::Message;

/// 窓ぜんたいを包んで、生の事象を [`Message`] にする1枚。
pub struct WindowInput<'a> {
    content: Element<'a, Message>,
}

/// 溜めているドロップ。フレームの区切りで吐き出して空に戻る。
#[derive(Default)]
struct Pending {
    dropped: Vec<PathBuf>,
}

/// 窓ぜんたいを [`WindowInput`] で包む。
pub fn window_input<'a>(content: impl Into<Element<'a, Message>>) -> WindowInput<'a> {
    WindowInput {
        content: content.into(),
    }
}

/// 近道キーの表。New/Open/Save の3本(egui shell の `handle_file_entry` が
/// `consume_shortcut` で並べていた物と同じ)。**窓ぜんたいの近道キーの正本は
/// これだけではない** — `crate::shortcuts` のモジュール doc に、この殻へ
/// 散った実装の全体地図がある(2026-08-19 iced 近道キー移植レーン)。
fn shortcut(character: &str) -> Option<Message> {
    match character.to_ascii_lowercase().as_str() {
        "n" => Some(Message::NewProjectPressed),
        "o" => Some(Message::OpenProjectPressed),
        "s" => Some(Message::SavePressed),
        _ => None,
    }
}

/// **既定の Theme / Renderer にだけ噛む。** この殻は窓を1枚しか持たず、
/// 中身(`Element<'a, Message>`)も既定で組まれているので、ここを型変数にすると
/// 「中身の Theme と包みの Theme が同じ」ことを書き足すだけになる。
impl Widget<Message, iced::Theme, iced::Renderer> for WindowInput<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Pending>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Pending::default())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        shell: &mut IcedShell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // 中身が先。テキスト入力のような「キーを自分の物として使う面」が
        // 木の中に立った時、窓の近道がそれを横取りしない(M-4 以降で効く)。
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
        if shell.is_event_captured() {
            return;
        }

        let pending: &mut Pending = tree.state.downcast_mut();
        match event {
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key, modifiers, ..
            }) if modifiers.command() => {
                let iced::keyboard::Key::Character(character) = key else {
                    return;
                };
                // New/Open/Save はこの file の表(`shortcut`)、それ以外で
                // このレーンが足した近道(Cmd+A = 全選択)は `crate::shortcuts`
                // の表(単一の正本 — モジュール doc に理由がある)。
                let Some(message) = shortcut(character)
                    .or_else(|| crate::shortcuts::additional_window_shortcut(character))
                else {
                    return;
                };
                shell.publish(message);
                shell.capture_event();
            }
            Event::Window(iced::window::Event::FileDropped(path)) => {
                // まだ言わない。フレームの区切りで**まとめて**1回言う。
                pending.dropped.push(path.clone());
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Window(iced::window::Event::RedrawRequested(_)) => {
                if pending.dropped.is_empty() {
                    return;
                }
                shell.publish(Message::FilesDropped(std::mem::take(&mut pending.dropped)));
            }
            Event::Window(iced::window::Event::CloseRequested) => {
                // 閉じるかどうかは殻が決める(未保存なら3択)。ここは伝えるだけ。
                shell.publish(Message::CloseRequested);
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a> From<WindowInput<'a>> for Element<'a, Message> {
    fn from(input: WindowInput<'a>) -> Self {
        Element::new(input)
    }
}
