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
        match self.tick_playback() {
            Ok(true) => event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(16),
            )),
            Ok(false) => {}
            Err(error) => self.fail(event_loop, error),
        }
        if let Some(retry_at) = self.request_auxiliary_redraw() {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(retry_at));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.owns_auxiliary_window(window_id) {
            let input = crate::easing_popup_input_adapter::normalize_easing_popup_input(event);
            self.handle_auxiliary_window_input(event_loop, window_id, input);
            return;
        }
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::Resized(size) => {
                self.resize(event_loop, size.width, size.height);
            }
            winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                self.scale_factor_changed(event_loop);
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let scale_factor = self.window_scale_factor().unwrap_or(1.0);
                let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
                    scale_factor
                } else {
                    1.0
                };
                self.set_cursor([position.x / scale_factor, position.y / scale_factor]);
                self.poll_host_input(event_loop);
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(horizontal, vertical) => {
                        self.scroll_timeline_line_delta([horizontal.into(), vertical.into()]);
                    }
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {
                        self.scroll_timeline_pixel_delta(
                            [delta.x, delta.y],
                            self.window_scale_factor().unwrap_or(1.0),
                        );
                    }
                }
                self.poll_host_input(event_loop);
            }
            winit::event::WindowEvent::Occluded(occluded) => self.set_occluded(occluded),
            winit::event::WindowEvent::RedrawRequested => self.render(event_loop),
            _ => self.poll_host_input(event_loop),
        }
    }
}
