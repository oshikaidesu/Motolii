//! 運転席の共有部品 — 押す・落とす・叩く、そして溜まった Message を流す。
//!
//! egui 版の `DrivenShell`(`blitz_shell/drive.rs`)に当たる層だが、こちらは
//! **crate の中に運転席を持たない**。`iced_test::Simulator` が製品の `view` を
//! そのまま借りて回すので、テスト側に要るのは「生の `iced::Event` を組む」
//! 短い関数だけである。
//!
//! ここに在るのは**注入の道具**だけで、合否の言い方は各テストが持つ。

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use motolii_shell_iced::{Message, Shell};

/// repo に入っている実 media(starter kit)。egui 版 `drive_tests.rs` と同じ根。
pub fn starter_media_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/mocks-ui/starter-media/media")
        .canonicalize()
        .expect("starter media lives in the repo")
}

/// 押した結果を殻へ流し込む。iced の `Simulator` は message を**溜める**ので、
/// 「押す」と「起こる」の間が明示的に切れている。
///
/// **型が順番を強制する。** `Simulator` は `view(&shell)` が返した Element を
/// 借りているので、殻を可変で触る前に必ず消費し切らなければならない。
pub fn press(mut ui: iced_test::Simulator<'_, Message>, selector: &str) -> Vec<Message> {
    ui.click(selector)
        .unwrap_or_else(|error| panic!("{selector:?} が押せる物として立っていない: {error}"));
    ui.into_messages().collect()
}

/// 生の `iced::Event` 列を窓へ流し込んで、出てきた Message を受け取る。
/// 近道キー・OS ドロップ・閉じる要求は全部この口を通る。
pub fn feed(
    mut ui: iced_test::Simulator<'_, Message>,
    events: impl IntoIterator<Item = iced::event::Event>,
) -> Vec<Message> {
    let events: Vec<iced::event::Event> = events.into_iter().collect();
    let _ = ui.simulate(events);
    ui.into_messages().collect()
}

/// 溜めた message を順に流す。最後の [`Outcome`](motolii_shell_iced::Outcome) を返す。
pub fn drain(shell: &mut Shell, messages: Vec<Message>) -> motolii_shell_iced::Outcome {
    let mut outcome = motolii_shell_iced::Outcome::Stay;
    for message in messages {
        outcome = shell.update(message);
    }
    outcome
}

/// `Cmd`(macOS 以外は `Ctrl`)+ 1文字の押下と離し。
///
/// `iced_test::tap_key` は modifiers を持てない(既定 = 何も押していない)ので、
/// 近道キーはここで生の `KeyPressed` / `KeyReleased` を組む。
pub fn command_key(character: char) -> Vec<iced::event::Event> {
    let key = iced::keyboard::Key::Character(character.to_string().into());
    let modifiers = iced::keyboard::Modifiers::COMMAND;
    vec![
        iced::event::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers,
            repeat: false,
            text: None,
        }),
        iced::event::Event::Keyboard(iced::keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers,
        }),
    ]
}

/// OS ドロップ1件。winit は**ファイル1つにつき1事象**を出す。
pub fn file_dropped(path: &Path) -> iced::event::Event {
    iced::event::Event::Window(iced::window::Event::FileDropped(path.to_path_buf()))
}

/// フレームの終わり。窓は毎フレームこれを widget 木へ流す
/// (`iced_winit/src/lib.rs` の redraw ループ)。落ちてきた path が
/// **1回の取り込み**にまとまるのはこの区切りである。
pub fn redraw() -> iced::event::Event {
    iced::event::Event::Window(iced::window::Event::RedrawRequested(
        iced::time::Instant::now(),
    ))
}

/// 窓を閉じる要求。`exit_on_close_request(false)` の窓ではこれが widget 木へ来る。
pub fn close_requested() -> iced::event::Event {
    iced::event::Event::Window(iced::window::Event::CloseRequested)
}

/// GPU(wgpu adapter)が無い環境では Stage 島の審判ができないので skip する。
/// egui 版運転席の「GPU 無ければ skip」と同じ扱い。`Some(())` なら回してよい。
pub fn gpu_or_skip() -> Option<()> {
    let instance = wgpu::Instance::new(re_renderer::device_caps::instance_descriptor(None));
    let adapters =
        iced::futures::executor::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    match re_renderer::device_caps::select_adapter(&adapters, wgpu::Backends::all(), None) {
        Ok(_) => Some(()),
        Err(error) => {
            println!("skip: no usable GPU adapter ({error})");
            None
        }
    }
}
