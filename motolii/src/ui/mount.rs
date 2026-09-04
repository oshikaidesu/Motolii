use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use anyrender::{RenderContext, Scene};
use blitz_dom::{node::ComputedStyles, Widget};
use blitz_traits::events::UiEvent;
use dioxus_native::{prelude::*, winit::window::WindowId, CustomWidgetAttr};

type SurfaceKey = (WindowId, &'static str);

#[derive(Clone, Default)]
pub(crate) struct MountStore(Rc<RefCell<HashMap<SurfaceKey, Rc<dyn Any>>>>);

impl MountStore {
    pub(super) fn get<S: SurfaceState>(
        &self,
        window: WindowId,
        key: &'static str,
        initialize: impl FnOnce() -> S,
    ) -> Rc<RefCell<S>> {
        let mut states = self.0.borrow_mut();
        if let Some(state) = states.get(&(window, key)) {
            return state
                .clone()
                .downcast::<RefCell<S>>()
                .unwrap_or_else(|_| panic!("Retained surface type changed: {key}"));
        }
        let state = Rc::new(RefCell::new(initialize()));
        println!(
            "MOTOLII_RELOAD {}",
            serde_json::json!({"event":"surface_created", "window":format!("{window:?}"), "surface":key, "owner":Rc::as_ptr(&state) as usize})
        );
        states.insert((window, key), state.clone());
        state
    }

    pub(crate) fn remove_window(&self, window: WindowId) {
        self.0.borrow_mut().retain(|(owner, _), _| *owner != window);
    }
}

/// State belongs to the window; bindings and renderer registrations belong to one mount.
pub(super) trait SurfaceState: 'static {
    type Bindings: Clone + PartialEq + 'static;
    type Mount: Default + 'static;

    fn can_create_surfaces(&mut self, _mount: &mut Self::Mount, _ctx: &mut dyn RenderContext) {}
    fn destroy_surfaces(&mut self, _mount: &mut Self::Mount) {}
    fn requires_redraw(&self, _bindings: &Self::Bindings) -> bool {
        false
    }
    fn handle_event(
        &mut self,
        _mount: &mut Self::Mount,
        _bindings: &Self::Bindings,
        _event: &UiEvent,
    ) {
    }
    fn paint(
        &mut self,
        mount: &mut Self::Mount,
        bindings: &Self::Bindings,
        ctx: &mut dyn RenderContext,
        styles: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Scene;
}

#[derive(Props)]
pub(super) struct SurfaceProps<S: SurfaceState> {
    handle: Rc<RefCell<S>>,
    bindings: S::Bindings,
    label: String,
}

impl<S: SurfaceState> Clone for SurfaceProps<S> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            bindings: self.bindings.clone(),
            label: self.label.clone(),
        }
    }
}

impl<S: SurfaceState> PartialEq for SurfaceProps<S> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.handle, &other.handle)
            && self.bindings == other.bindings
            && self.label == other.label
    }
}

#[allow(non_snake_case)]
pub(super) fn SurfaceView<S: SurfaceState>(props: SurfaceProps<S>) -> Element {
    let outputs = use_hook(|| Rc::new(RefCell::new(props.bindings.clone())));
    *outputs.borrow_mut() = props.bindings.clone();
    let label = props.label.clone();
    let mut mounted = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let attr = use_hook(|| {
        CustomWidgetAttr::new(MountedSurface::<S> {
            state: props.handle.clone(),
            bindings: outputs,
            mount: S::Mount::default(),
        })
    });
    rsx! { object {
        class: "custom-surface",
        role: "application",
        tabindex: "0",
        aria_label: "{label}",
        onmounted: move |evt: MountedEvent| mounted.set(Some(evt.data())),
        onpointerdown: move |evt: PointerEvent| {
            if evt.data().is_primary()
                && evt.data().trigger_button()
                    == Some(dioxus_native::prelude::dioxus_elements::input_data::MouseButton::Primary)
            {
                if let Some(handle) = mounted.read().as_ref().cloned() {
                    crate::ui::semantic_menu::focus_mounted(handle);
                }
            }
        },
        "data": attr,
        span { class: "a11y", "{label}" }
    } }
}

struct MountedSurface<S: SurfaceState> {
    state: Rc<RefCell<S>>,
    bindings: Rc<RefCell<S::Bindings>>,
    mount: S::Mount,
}

/// Blitz makes custom-widget `client` coordinates local while forwarding the
/// default action, but leaves its window-relative `screen` point intact and
/// `element` is already local. Restore window client/page once at the adapter
/// boundary so menus and cross-DOM capture match ordinary Dioxus controls.
fn restore_window_pointer(event: &UiEvent) -> UiEvent {
    fn restore(
        pointer: &blitz_traits::events::BlitzPointerEvent,
    ) -> blitz_traits::events::BlitzPointerEvent {
        let mut pointer = pointer.clone();
        pointer.coords.client_x = pointer.coords.screen_x;
        pointer.coords.client_y = pointer.coords.screen_y;
        pointer.coords.page_x = pointer.coords.screen_x;
        pointer.coords.page_y = pointer.coords.screen_y;
        pointer
    }
    match event {
        UiEvent::PointerMove(pointer) => UiEvent::PointerMove(restore(pointer)),
        UiEvent::PointerDown(pointer) => UiEvent::PointerDown(restore(pointer)),
        UiEvent::PointerUp(pointer) => UiEvent::PointerUp(restore(pointer)),
        UiEvent::PointerCancel(pointer) => UiEvent::PointerCancel(restore(pointer)),
        _ => event.clone(),
    }
}

type SurfaceFn<S> = fn(&mut S, &mut <S as SurfaceState>::Mount, &mut dyn RenderContext);
type DestroyFn<S> = fn(&mut S, &mut <S as SurfaceState>::Mount);
type RedrawFn<S> = fn(&S, &<S as SurfaceState>::Bindings) -> bool;
type EventFn<S> =
    fn(&mut S, &mut <S as SurfaceState>::Mount, &<S as SurfaceState>::Bindings, &UiEvent);
type PaintFn<S> = fn(
    &mut S,
    &mut <S as SurfaceState>::Mount,
    &<S as SurfaceState>::Bindings,
    &mut dyn RenderContext,
    &ComputedStyles,
    u32,
    u32,
    f64,
) -> Scene;

impl<S: SurfaceState> Widget for MountedSurface<S> {
    fn can_create_surfaces(&mut self, ctx: &mut dyn RenderContext) {
        let mut state = self.state.borrow_mut();
        dioxus_devtools::subsecond::HotFn::current(S::can_create_surfaces as SurfaceFn<S>).call((
            &mut *state,
            &mut self.mount,
            ctx,
        ));
    }

    fn destroy_surfaces(&mut self) {
        let mut state = self.state.borrow_mut();
        dioxus_devtools::subsecond::HotFn::current(S::destroy_surfaces as DestroyFn<S>)
            .call((&mut *state, &mut self.mount));
    }

    fn requires_redraw(&self) -> bool {
        let state = self.state.borrow();
        let bindings = self.bindings.borrow();
        dioxus_devtools::subsecond::HotFn::current(S::requires_redraw as RedrawFn<S>)
            .call((&*state, &*bindings))
    }

    fn handle_event(&mut self, event: &UiEvent) {
        let mut state = self.state.borrow_mut();
        let bindings = self.bindings.borrow();
        let event = restore_window_pointer(event);
        dioxus_devtools::subsecond::HotFn::current(S::handle_event as EventFn<S>).call((
            &mut *state,
            &mut self.mount,
            &*bindings,
            &event,
        ));
    }

    fn paint(
        &mut self,
        ctx: &mut dyn RenderContext,
        styles: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Scene {
        let mut state = self.state.borrow_mut();
        let bindings = self.bindings.borrow();
        dioxus_devtools::subsecond::HotFn::current(S::paint as PaintFn<S>).call((
            &mut *state,
            &mut self.mount,
            &*bindings,
            ctx,
            styles,
            width,
            height,
            scale,
        ))
    }
}

pub(super) fn resource_identity(value: &impl std::hash::Hash) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod pointer_coordinates {
    use super::*;
    use crate::ui::session::{CapturePhase, CustomSurface, SurfaceCapture};
    use blitz_traits::events::{
        BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point,
        PointerCoords, UiEvent,
    };

    fn forwarded_pointer() -> BlitzPointerEvent {
        BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords {
                page_x: 10.0,
                page_y: 20.0,
                screen_x: 400.0,
                screen_y: 240.0,
                client_x: 10.0,
                client_y: 20.0,
            },
            button: MouseEventButton::Main,
            buttons: MouseEventButtons::Primary,
            mods: Default::default(),
            details: Default::default(),
            element: Point { x: 10.0, y: 20.0 },
            active_pointers: Default::default(),
        }
    }

    #[test]
    fn custom_widget_keeps_local_element_but_restores_window_client_coordinates() {
        let UiEvent::PointerDown(pointer) =
            restore_window_pointer(&UiEvent::PointerDown(forwarded_pointer()))
        else {
            panic!("pointer kind changed")
        };
        assert_eq!((pointer.element.x, pointer.element.y), (10.0, 20.0));
        assert_eq!((pointer.client_x(), pointer.client_y()), (400.0, 240.0));
    }

    #[test]
    fn restored_origin_keeps_cross_dom_drag_under_the_pointer() {
        let UiEvent::PointerDown(pointer) =
            restore_window_pointer(&UiEvent::PointerDown(forwarded_pointer()))
        else {
            panic!("pointer kind changed")
        };
        let capture = SurfaceCapture::default();
        assert!(capture.begin(CustomSurface::Stage, &pointer));
        assert!(capture.relay_dom(
            CapturePhase::Move,
            "mouse",
            0,
            true,
            [420.0, 250.0],
            true,
            Default::default(),
        ));
        let UiEvent::PointerMove(moved) = capture.take(CustomSurface::Stage)[0].event() else {
            panic!("capture did not preserve the move")
        };
        assert_eq!((moved.element.x, moved.element.y), (30.0, 30.0));
        assert_eq!((moved.client_x(), moved.client_y()), (420.0, 250.0));
    }
}
