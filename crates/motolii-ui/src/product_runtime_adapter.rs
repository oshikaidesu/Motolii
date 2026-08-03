//! direct product Hostに許可したwinit window lifecycle adapter。

impl winit::application::ApplicationHandler<crate::product_runtime::ProductEvent>
    for crate::product_runtime::ProductApp
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Err(error) = self.initialize(event_loop) {
            self.fail(event_loop, error);
        }
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: crate::product_runtime::ProductEvent,
    ) {
        self.handle_product_event(event_loop, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.poll_browser(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::Resized(size) => {
                self.resize(event_loop, size.width, size.height);
            }
            winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                self.scale_factor_changed(event_loop);
            }
            winit::event::WindowEvent::Occluded(occluded) => self.set_occluded(occluded),
            winit::event::WindowEvent::RedrawRequested => self.render(event_loop),
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.handle_window_cursor_moved([position.x, position.y])
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    let phase = if state.is_pressed() {
                        crate::InputPhase::Press
                    } else {
                        crate::InputPhase::Release
                    };
                    self.handle_window_pointer_phase(event_loop, phase);
                }
            }
            winit::event::WindowEvent::Focused(false) => self.handle_window_safety_interrupt(
                event_loop,
                crate::SafetyInterrupt::WindowFocusLost,
            ),
            winit::event::WindowEvent::CursorLeft { .. } => self.handle_window_safety_interrupt(
                event_loop,
                crate::SafetyInterrupt::PointerCaptureLost,
            ),
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                if let Some(modifiers) = normalized_modifiers(modifiers.state()) {
                    self.handle_window_modifiers(modifiers);
                }
            }
            winit::event::WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if let Some(key) =
                    normalized_key(is_synthetic, event.state, event.repeat, &event.logical_key)
                {
                    self.handle_window_key(event_loop, key);
                }
            }
            winit::event::WindowEvent::Ime(ime) => {
                if let Some(state) = normalized_ime(&ime) {
                    self.handle_window_ime(state);
                }
            }
            _ => self.poll_host_input(event_loop),
        }
    }
}

fn normalized_modifiers(state: winit::keyboard::ModifiersState) -> Option<crate::Modifiers> {
    crate::Modifiers::try_new(
        [
            state.control_key().then_some(crate::Modifier::Control),
            state.super_key().then_some(crate::Modifier::Meta),
            state.alt_key().then_some(crate::Modifier::Alt),
            state.shift_key().then_some(crate::Modifier::Shift),
        ]
        .into_iter()
        .flatten(),
    )
    .ok()
}

fn normalized_key(
    is_synthetic: bool,
    state: winit::event::ElementState,
    repeat: bool,
    logical_key: &winit::keyboard::Key,
) -> Option<crate::KeyToken> {
    (!is_synthetic
        && state.is_pressed()
        && !repeat
        && logical_key == &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape))
        .then_some(crate::KeyToken::Escape)
}

fn normalized_ime(ime: &winit::event::Ime) -> Option<crate::ImeGateState> {
    match ime {
        winit::event::Ime::Preedit(text, _) => Some(if text.is_empty() {
            crate::ImeGateState::Inactive
        } else {
            crate::ImeGateState::PreeditActive
        }),
        winit::event::Ime::Commit(_) | winit::event::Ime::Disabled => {
            Some(crate::ImeGateState::Inactive)
        }
        winit::event::Ime::Enabled => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{normalized_ime, normalized_key, normalized_modifiers};

    #[test]
    fn logical_escape_accepts_only_real_first_press() {
        let escape = winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape);
        assert_eq!(
            normalized_key(false, winit::event::ElementState::Pressed, false, &escape),
            Some(crate::KeyToken::Escape)
        );
        assert_eq!(
            normalized_key(true, winit::event::ElementState::Pressed, false, &escape),
            None
        );
        assert_eq!(
            normalized_key(false, winit::event::ElementState::Pressed, true, &escape),
            None
        );
        assert_eq!(
            normalized_key(false, winit::event::ElementState::Released, false, &escape),
            None
        );
        assert_eq!(
            normalized_key(
                false,
                winit::event::ElementState::Pressed,
                false,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Process),
            ),
            None
        );
    }

    #[test]
    fn modifiers_map_without_primary_alias() {
        let state = winit::keyboard::ModifiersState::CONTROL
            | winit::keyboard::ModifiersState::SUPER
            | winit::keyboard::ModifiersState::ALT
            | winit::keyboard::ModifiersState::SHIFT;
        let modifiers = normalized_modifiers(state).unwrap();
        assert_eq!(
            modifiers.iter().collect::<Vec<_>>(),
            vec![
                crate::Modifier::Control,
                crate::Modifier::Meta,
                crate::Modifier::Alt,
                crate::Modifier::Shift,
            ]
        );
    }

    #[test]
    fn ime_gate_tracks_preedit_without_text_ownership() {
        assert_eq!(
            normalized_ime(&winit::event::Ime::Preedit("変換".into(), None)),
            Some(crate::ImeGateState::PreeditActive)
        );
        assert_eq!(
            normalized_ime(&winit::event::Ime::Preedit("".into(), None)),
            Some(crate::ImeGateState::Inactive)
        );
        assert_eq!(
            normalized_ime(&winit::event::Ime::Commit("確定".into())),
            Some(crate::ImeGateState::Inactive)
        );
        assert_eq!(normalized_ime(&winit::event::Ime::Enabled), None);
    }
}
