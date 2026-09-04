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
}

impl<S: SurfaceState> Clone for SurfaceProps<S> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            bindings: self.bindings.clone(),
        }
    }
}

impl<S: SurfaceState> PartialEq for SurfaceProps<S> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.handle, &other.handle) && self.bindings == other.bindings
    }
}

#[allow(non_snake_case)]
pub(super) fn SurfaceView<S: SurfaceState>(props: SurfaceProps<S>) -> Element {
    let outputs = use_hook(|| Rc::new(RefCell::new(props.bindings.clone())));
    *outputs.borrow_mut() = props.bindings;
    let attr = use_hook(|| {
        CustomWidgetAttr::new(MountedSurface::<S> {
            state: props.handle.clone(),
            bindings: outputs,
            mount: S::Mount::default(),
        })
    });
    rsx! { object { "data": attr } }
}

struct MountedSurface<S: SurfaceState> {
    state: Rc<RefCell<S>>,
    bindings: Rc<RefCell<S::Bindings>>,
    mount: S::Mount,
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
        dioxus_devtools::subsecond::HotFn::current(S::handle_event as EventFn<S>).call((
            &mut *state,
            &mut self.mount,
            &*bindings,
            event,
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
