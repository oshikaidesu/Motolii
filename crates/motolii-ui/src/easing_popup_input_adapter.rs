//! native Easing popupだけに許可するwinit入力のprivate正規化adapter。

use crate::easing_popup_runtime::{EasingPopupArrow, EasingPopupInput};

pub(crate) fn normalize_easing_popup_input(event: winit::event::WindowEvent) -> EasingPopupInput {
    match event {
        winit::event::WindowEvent::CloseRequested => EasingPopupInput::CloseRequested,
        winit::event::WindowEvent::Focused(focused) => EasingPopupInput::Focused(focused),
        winit::event::WindowEvent::Resized(size) => EasingPopupInput::Resized {
            width: size.width,
            height: size.height,
        },
        winit::event::WindowEvent::RedrawRequested => EasingPopupInput::RedrawRequested,
        winit::event::WindowEvent::CursorMoved { position, .. } => EasingPopupInput::CursorMoved {
            physical_x: position.x,
            physical_y: position.y,
        },
        winit::event::WindowEvent::MouseInput {
            state,
            button: winit::event::MouseButton::Left,
            ..
        } => match state {
            winit::event::ElementState::Pressed => EasingPopupInput::PrimaryPressed,
            winit::event::ElementState::Released => EasingPopupInput::PrimaryReleased,
        },
        winit::event::WindowEvent::KeyboardInput { event, .. }
            if event.state == winit::event::ElementState::Pressed =>
        {
            match event.logical_key {
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) => {
                    EasingPopupInput::EscapePressed
                }
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
                    EasingPopupInput::TabPressed
                }
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
                    EasingPopupInput::ArrowPressed(EasingPopupArrow::Left)
                }
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
                    EasingPopupInput::ArrowPressed(EasingPopupArrow::Right)
                }
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                    EasingPopupInput::ArrowPressed(EasingPopupArrow::Up)
                }
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                    EasingPopupInput::ArrowPressed(EasingPopupArrow::Down)
                }
                _ => EasingPopupInput::Ignored,
            }
        }
        _ => EasingPopupInput::Ignored,
    }
}
