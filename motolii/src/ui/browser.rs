use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::doc::store::{
    property, ContentKeyframe, ContentTrack, Document, FontRef, Intent, Interp, Keyframe,
    KeyframeTrack, LayerAttrsPatch, LayerId, LayerMeta, LayerProjection, LayerSource, LayerTiming,
    Mask, MaskId, MaskMode, Path, PathSource, PathVertex, PropertyId, RationalTime, Shape,
    ShapeNode, TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify, TextStyleId,
    Value, VectorPoint,
};
use crate::doc::vector::{Brush, Contour, Fill, FillRule, Rgb, Vertex};

use crate::ui::fixture::ColorSwatch;

use crate::ui::browser_selection::{
    BrowserItemId, BrowserScope, BrowserSelection, CreateItem, MoveActive,
};
use crate::ui::color::{wheel_slot, ColorWheel};
use crate::ui::dock::Panel;
use crate::ui::fixture::{self, LayerRow};
use crate::ui::functions::verb::{color_block, effect_batch_intents};
use crate::ui::playback::Clock;
use crate::ui::semantic_menu::{FocusableItems, SemanticButton, SpatialDirection};
use crate::ui::session::Session;
use crate::ui::timeline_widget::TimelineMsg;

#[derive(Clone)]
enum NewKind {
    Text,
    Rectangle,
    Bezier,
    Media { path: String, name: String },
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct BrowserMarquee {
    scope: BrowserScope,
    pointer_type: String,
    pointer_id: i32,
    start: [f64; 2],
    current: [f64; 2],
    additive: bool,
}

impl BrowserMarquee {
    const THRESHOLD: f64 = 4.0;

    fn new(scope: BrowserScope, event: &PointerEvent) -> Self {
        let p = event.data().client_coordinates();
        Self {
            scope,
            pointer_type: event.data().pointer_type(),
            pointer_id: event.data().pointer_id(),
            start: [p.x, p.y],
            current: [p.x, p.y],
            additive: crate::ui::keymap::primary_modifier(event.data().modifiers()),
        }
    }

    fn matches(&self, event: &PointerEvent) -> bool {
        self.pointer_id == event.data().pointer_id()
            && self.pointer_type == event.data().pointer_type()
    }

    fn move_to(&mut self, event: &PointerEvent) {
        let p = event.data().client_coordinates();
        self.current = [p.x, p.y];
    }

    fn dragging(&self) -> bool {
        (self.current[0] - self.start[0]).abs() >= Self::THRESHOLD
            || (self.current[1] - self.start[1]).abs() >= Self::THRESHOLD
    }

    fn bounds(&self) -> (f64, f64, f64, f64) {
        (
            self.start[0].min(self.current[0]),
            self.start[1].min(self.current[1]),
            self.start[0].max(self.current[0]),
            self.start[1].max(self.current[1]),
        )
    }
}

fn browser_scope(panel: Panel) -> BrowserScope {
    match panel {
        Panel::Media => BrowserScope::Media,
        Panel::Effects => BrowserScope::Effects,
        Panel::Create => BrowserScope::Create,
        Panel::Colors => BrowserScope::Colors,
        _ => unreachable!("only Browser panels enter browser_panel"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BrowserKeyAction {
    None,
    Commit,
    Delete,
    FocusSearch,
    TypeAhead(String),
    Unavailable,
    Spatial(SpatialDirection, bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BrowserKeyResult {
    handled: bool,
    changed: bool,
    focus_key: Option<String>,
    action: BrowserKeyAction,
}

impl BrowserKeyResult {
    fn ignored() -> Self {
        Self {
            handled: false,
            changed: false,
            focus_key: None,
            action: BrowserKeyAction::None,
        }
    }
}

/// Browser-owned keys never reach the artwork keymap. Arrow keys are handed to
/// the mounted-card spatial registry; PageUp/PageDown use the deterministic
/// linear step documented by `browser_selection`.
fn browser_key(
    selection: &mut BrowserSelection,
    scope: BrowserScope,
    ordered: &[BrowserItemId],
    key: &Key,
    code: Code,
    modifiers: Modifiers,
    multiple: bool,
) -> BrowserKeyResult {
    let before = selection.clone();
    let command = crate::ui::keymap::primary_modifier(modifiers);
    let shift = modifiers.shift() && multiple;
    let spatial = match key {
        Key::ArrowLeft => Some(SpatialDirection::Left),
        Key::ArrowRight => Some(SpatialDirection::Right),
        Key::ArrowUp => Some(SpatialDirection::Up),
        Key::ArrowDown => Some(SpatialDirection::Down),
        _ => None,
    };
    if let Some(direction) = spatial {
        return BrowserKeyResult {
            handled: true,
            changed: false,
            focus_key: None,
            action: BrowserKeyAction::Spatial(direction, shift),
        };
    }
    let movement = match key {
        Key::PageUp => Some(MoveActive::PagePrevious),
        Key::PageDown => Some(MoveActive::PageNext),
        Key::Home => Some(MoveActive::First),
        Key::End => Some(MoveActive::Last),
        _ => None,
    };

    let (handled, action, focus) = if let Some(movement) = movement {
        let focus = selection.move_active(scope, ordered, movement, shift);
        (true, BrowserKeyAction::None, focus)
    } else if matches!(key, Key::Character(c) if command && c.eq_ignore_ascii_case("f")) {
        (true, BrowserKeyAction::FocusSearch, None)
    } else if matches!(key, Key::Character(c) if command && c.eq_ignore_ascii_case("a")) {
        if multiple {
            selection.select_all(scope, ordered);
        } else if let Some(active) = selection.active_in(scope, ordered) {
            selection.click(scope, &active, ordered, false, false);
        } else {
            selection.move_active(scope, ordered, MoveActive::First, false);
        }
        (
            true,
            BrowserKeyAction::None,
            selection.active_in(scope, ordered),
        )
    } else if matches!(key, Key::Character(c) if command && matches!(c.to_ascii_lowercase().as_str(), "c" | "v" | "x"))
    {
        (true, BrowserKeyAction::Unavailable, None)
    } else if matches!(key, Key::Character(c) if c == " ") {
        if let Some(active) = selection.active_in(scope, ordered) {
            selection.click(
                scope,
                &active,
                ordered,
                multiple && command,
                multiple && modifiers.shift(),
            );
        } else {
            selection.move_active(scope, ordered, MoveActive::First, false);
        }
        (
            true,
            BrowserKeyAction::None,
            selection.active_in(scope, ordered),
        )
    } else {
        match key {
            Key::Escape => {
                let focus = if selection.clear_query(scope) {
                    let active = selection.active_in(scope, ordered);
                    if active.is_none() {
                        selection.move_active(scope, ordered, MoveActive::First, false)
                    } else {
                        active
                    }
                } else {
                    selection.clear(scope);
                    None
                };
                (true, BrowserKeyAction::None, focus)
            }
            Key::Enter => (true, BrowserKeyAction::Commit, None),
            Key::Delete | Key::Backspace => (true, BrowserKeyAction::Delete, None),
            Key::Character(c)
                if !modifiers
                    .intersects(Modifiers::CONTROL | Modifiers::SUPER | Modifiers::ALT)
                    && c != " " =>
            {
                let query = c.to_string();
                selection.set_query(scope, query.clone());
                (true, BrowserKeyAction::TypeAhead(query), None)
            }
            _ => {
                let mapped = crate::ui::keymap::lookup_held(
                    key,
                    code,
                    command,
                    modifiers.shift(),
                    modifiers.alt(),
                );
                if matches!(
                    mapped,
                    Some(
                        crate::ui::contracts::Intent::Undo
                            | crate::ui::contracts::Intent::Redo
                            | crate::ui::contracts::Intent::Save
                            | crate::ui::contracts::Intent::SaveAs
                            | crate::ui::contracts::Intent::NewProject
                            | crate::ui::contracts::Intent::OpenProject
                            | crate::ui::contracts::Intent::Quit
                            | crate::ui::contracts::Intent::CompositionSettings
                    )
                ) {
                    return BrowserKeyResult::ignored();
                }
                if mapped.is_some() {
                    (true, BrowserKeyAction::Unavailable, None)
                } else {
                    return BrowserKeyResult::ignored();
                }
            }
        }
    };

    BrowserKeyResult {
        handled,
        changed: *selection != before,
        focus_key: focus.map(|id| id.focus_key()),
        action,
    }
}

#[allow(clippy::too_many_arguments)]
fn move_browser_spatial(
    focusable_items: FocusableItems,
    session: Session,
    scope: BrowserScope,
    ordered: Vec<BrowserItemId>,
    direction: SpatialDirection,
    extend: bool,
    poke: crate::ui::poke::Poke,
) {
    let current = session
        .browser_selection
        .lock()
        .unwrap()
        .active_in(scope, &ordered);
    let Some(current) = current else {
        let next = session.browser_selection.lock().unwrap().move_active(
            scope,
            &ordered,
            MoveActive::First,
            false,
        );
        if let Some(next) = next {
            poke.poke();
            focusable_items.focus(&next.focus_key());
        }
        return;
    };
    let current_key = current.focus_key();
    let fallback_direction = match direction {
        SpatialDirection::Left | SpatialDirection::Up => MoveActive::Previous,
        SpatialDirection::Right | SpatialDirection::Down => MoveActive::Next,
    };
    let moved_session = session.clone();
    let moved_order = ordered.clone();
    let moved_poke = poke.clone();
    let allowed: Vec<_> = ordered.iter().map(BrowserItemId::focus_key).collect();
    let moved = focusable_items.move_spatial(&current_key, &allowed, direction, move |next_key| {
        let Some(next) = moved_order
            .iter()
            .find(|id| id.focus_key() == next_key)
            .cloned()
        else {
            return;
        };
        let changed = moved_session.browser_selection.lock().unwrap().click(
            scope,
            &next,
            &moved_order,
            false,
            extend,
        );
        if changed {
            moved_poke.poke();
        }
    });
    if !moved {
        let next = session.browser_selection.lock().unwrap().move_active(
            scope,
            &ordered,
            fallback_direction,
            extend,
        );
        if let Some(next) = next {
            poke.poke();
            focusable_items.focus(&next.focus_key());
        }
    }
}

fn consume_browser_key(
    event: &Event<KeyboardData>,
    result: BrowserKeyResult,
    focusable_items: FocusableItems,
    poke: &crate::ui::poke::Poke,
) -> Option<BrowserKeyAction> {
    if !result.handled {
        return None;
    }
    event.prevent_default();
    event.stop_propagation();
    if result.changed {
        poke.poke();
    }
    if let Some(key) = result.focus_key {
        focusable_items.focus(&key);
    }
    Some(result.action)
}

fn choose_browser_item(
    session: &Session,
    scope: BrowserScope,
    id: &BrowserItemId,
    ordered: &[BrowserItemId],
    modifiers: Modifiers,
    multiple: bool,
    poke: &crate::ui::poke::Poke,
) {
    let changed = session.browser_selection.lock().unwrap().click(
        scope,
        id,
        ordered,
        multiple && crate::ui::keymap::primary_modifier(modifiers),
        multiple && modifiers.shift(),
    );
    if changed {
        poke.poke();
    }
}

fn query_matches(value: &str, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty() || value.to_lowercase().contains(&query)
}

fn media_rail_key(family: Option<fixture::AssetFamily>) -> String {
    format!(
        "browser:media-rail:{}",
        family.map_or("all", fixture::AssetFamily::label)
    )
}

fn result_tabindex(active: Option<&BrowserItemId>, item: &BrowserItemId) -> String {
    if active == Some(item) {
        "0".to_owned()
    } else {
        "-1".to_owned()
    }
}

fn consume_fixed_rail_key(event: &KeyboardEvent) {
    if matches!(
        event.key(),
        Key::ArrowLeft
            | Key::ArrowRight
            | Key::ArrowUp
            | Key::ArrowDown
            | Key::Home
            | Key::End
            | Key::Enter
    ) || matches!(event.key(), Key::Character(ref c) if c == " ")
    {
        event.prevent_default();
        event.stop_propagation();
    }
}

/// Pinned Blitz has no MountedData text-selection method. Its NodeHandle does
/// expose the underlying BaseDocument, so Cmd+F uses the same native text-input
/// default action as a physical Cmd/Ctrl+A after moving focus to the search box.
fn focus_and_select_search(handle: Option<std::rc::Rc<MountedData>>) -> bool {
    let Some(handle) = handle else { return false };
    let Some(node) = handle.downcast::<dioxus_native::NodeHandle>().cloned() else {
        return false;
    };
    let id = node.node_id();
    let mut doc = node.doc_mut();
    doc.set_focus_to(id);
    let mut event = blitz_traits::events::DomEvent::new(
        id,
        blitz_traits::events::DomEventData::KeyDown(blitz_traits::events::BlitzKeyEvent {
            key: Key::Character("a".into()),
            code: Code::KeyA,
            modifiers: if cfg!(target_os = "macos") {
                Modifiers::SUPER
            } else {
                Modifiers::CONTROL
            },
            location: keyboard_types::Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: blitz_traits::events::KeyState::Pressed,
            text: None,
        }),
    );
    doc.handle_dom_event(&mut event, |_| {});
    true
}

fn focus_search_with_text(handle: Option<std::rc::Rc<MountedData>>, text: &str) -> bool {
    let Some(handle) = handle else { return false };
    if !focus_and_select_search(Some(handle.clone())) {
        return false;
    }
    let Some(node) = handle.downcast::<dioxus_native::NodeHandle>().cloned() else {
        return false;
    };
    let id = node.node_id();
    let mut event = blitz_traits::events::DomEvent::new(
        id,
        blitz_traits::events::DomEventData::KeyDown(blitz_traits::events::BlitzKeyEvent {
            key: Key::Character(text.into()),
            code: Code::Unidentified,
            modifiers: Modifiers::empty(),
            location: keyboard_types::Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: blitz_traits::events::KeyState::Pressed,
            text: Some(text.into()),
        }),
    );
    node.doc_mut().handle_dom_event(&mut event, |_| {});
    true
}

fn browser_action_unavailable(session: &Session, poke: &crate::ui::poke::Poke) {
    *session.project_notice.lock().unwrap() =
        "That action is not available for Browser items".into();
    poke.poke();
}

fn begin_browser_marquee(
    session: &Session,
    scope: BrowserScope,
    event: &PointerEvent,
    mut marquee: Signal<Option<BrowserMarquee>>,
) {
    if !session.browser_selection.lock().unwrap().select_mode(scope)
        || !event.data().is_primary()
        || event.data().trigger_button()
            != Some(dioxus_native::prelude::dioxus_elements::input_data::MouseButton::Primary)
    {
        return;
    }
    event.prevent_default();
    event.stop_propagation();
    session.gesture.begin();
    marquee.set(Some(BrowserMarquee::new(scope, event)));
}

fn move_browser_marquee(event: &PointerEvent, mut marquee: Signal<Option<BrowserMarquee>>) {
    let mut state = marquee.write();
    let Some(drag) = state.as_mut().filter(|drag| drag.matches(event)) else {
        return;
    };
    drag.move_to(event);
    if drag.dragging() {
        event.prevent_default();
        event.stop_propagation();
    }
}

#[allow(clippy::too_many_arguments)]
fn finish_browser_marquee(
    session: &Session,
    scope: BrowserScope,
    ordered: &[BrowserItemId],
    event: &PointerEvent,
    focusable_items: FocusableItems,
    mut marquee: Signal<Option<BrowserMarquee>>,
    poke: &crate::ui::poke::Poke,
) {
    let Some(mut drag) = marquee.write().take() else {
        return;
    };
    if drag.scope != scope || !drag.matches(event) {
        marquee.set(Some(drag));
        return;
    }
    drag.move_to(event);
    session.gesture.end();
    let dragging = drag.dragging();
    event.prevent_default();
    event.stop_propagation();
    let (left, top, right, bottom) = drag.bounds();
    let allowed: Vec<_> = ordered.iter().map(BrowserItemId::focus_key).collect();
    let moved_session = session.clone();
    let moved_order = ordered.to_vec();
    let moved_poke = poke.clone();
    let additive = drag.additive;
    focusable_items.items_intersecting(&allowed, left, top, right, bottom, move |keys| {
        let key_set: std::collections::BTreeSet<_> = keys.into_iter().collect();
        let hits: Vec<_> = moved_order
            .iter()
            .filter(|id| key_set.contains(&id.focus_key()))
            .cloned()
            .collect();
        let changed = if dragging {
            moved_session.browser_selection.lock().unwrap().marquee(
                scope,
                &moved_order,
                &hits,
                additive,
            )
        } else if let Some(hit) = hits.first() {
            moved_session.browser_selection.lock().unwrap().click(
                scope,
                hit,
                &moved_order,
                true,
                false,
            )
        } else if additive {
            false
        } else {
            moved_session.browser_selection.lock().unwrap().clear(scope)
        };
        if changed {
            moved_poke.poke();
        }
    });
}

fn cancel_browser_marquee(
    session: &Session,
    event: &PointerEvent,
    mut marquee: Signal<Option<BrowserMarquee>>,
) {
    let matches = marquee
        .peek()
        .as_ref()
        .is_some_and(|drag| drag.matches(event));
    if matches {
        marquee.set(None);
        session.gesture.end();
    }
}

fn marquee_overlay(
    session: &Session,
    drag: Option<BrowserMarquee>,
    scope: BrowserScope,
    ordered: &[BrowserItemId],
    focusable_items: FocusableItems,
    marquee: Signal<Option<BrowserMarquee>>,
    poke: &crate::ui::poke::Poke,
) -> Element {
    let Some(drag) = drag.filter(|drag| drag.scope == scope) else {
        return rsx! {};
    };
    let (left, top, right, bottom) = drag.bounds();
    let width = right - left;
    let height = bottom - top;
    let moving = marquee;
    let up_session = session.clone();
    let up_order = ordered.to_vec();
    let up_poke = poke.clone();
    let cancel_session = session.clone();
    rsx!(div {
        class: "browser-marquee-capture",
        aria_hidden: "true",
        onpointermove: move |evt: PointerEvent| move_browser_marquee(&evt, moving),
        onpointerup: move |evt: PointerEvent| finish_browser_marquee(
            &up_session,
            scope,
            &up_order,
            &evt,
            focusable_items,
            marquee,
            &up_poke,
        ),
        onpointercancel: move |evt: PointerEvent| cancel_browser_marquee(
            &cancel_session,
            &evt,
            marquee,
        ),
        if drag.dragging() {
            div {
                class: "browser-marquee",
                style: "left:{left}px; top:{top}px; width:{width}px; height:{height}px;",
            }
        }
    })
}

/// 空間素材のファイル座標を comp のピクセルへ橋渡しする初期値。囲む球が画角の
/// 6割に収まる倍率と、正規化した箱の左上が中央配置になる位置を**一度だけ**書く。
/// 以後は利用者の物(キーフレームも打てる)。
/// 3D の素材を画角へ収める。中心を comp の真ん中へ、大きさを画面の6割へ。
fn spatial_fit_intents(layer: LayerId, path: &str, comp: (f64, f64)) -> Vec<Intent> {
    let bounds = if crate::render::media::is_mesh_path(path) {
        crate::render::media::load_mesh_bounds(path).ok()
    } else {
        crate::render::media::load_point_cloud(std::path::Path::new(path))
            .ok()
            .map(|data| data.bounds())
    };
    let Some(bounds) = bounds else {
        return Vec::new();
    };
    let radius = bounds.radius();
    if radius <= 0.0 {
        return Vec::new();
    }
    let fit = (comp.1 * 0.6) / (radius as f64 * 2.0);
    let size = bounds.size_xy();
    // 置いた時の初期値は**素の値**。0秒のキーにすると、利用者が ◇ を
    // 押していないのに時間の世界が開いてしまう。
    let put = |name: &str, value: Value| Intent::SetConstant {
        layer,
        property: PropertyId::new(name).expect("既知の属性"),
        value,
    };
    vec![
        put(property::SCALE, Value::Vec2([fit, fit])),
        put(
            property::POSITION,
            Value::Vec2([
                (comp.0 - size[0] as f64 * fit) * 0.5,
                (comp.1 - size[1] as f64 * fit) * 0.5,
            ]),
        ),
    ]
}

/// 位置を指定せずに生まれた層を、枠の真ん中へ置く。
///
/// 隅(0,0)に置くと、素材が小さいほど画面の角の点になって見つからない。
/// 大きさが分かる物は中心を合わせ、分からない物(文字は組んでみるまで
/// 大きさが決まらない)は左上を真ん中へ置く。
fn center_intents(layer: LayerId, natural: (f64, f64), comp: (f64, f64)) -> Vec<Intent> {
    let Ok(property) = crate::doc::store::PropertyId::new(crate::doc::store::property::POSITION)
    else {
        return Vec::new();
    };
    let value =
        crate::doc::store::Value::Vec2([(comp.0 - natural.0) * 0.5, (comp.1 - natural.1) * 0.5]);
    vec![Intent::SetConstant {
        layer,
        property,
        value,
    }]
}

/// 形の実寸。焼く前でも輪郭から測れる。
fn shape_natural(shapes: &[ShapeNode]) -> (f64, f64) {
    crate::doc::vector::content_bounds(shapes)
        .ok()
        .flatten()
        .map(|b| (b[2] - b[0], b[3] - b[1]))
        .unwrap_or((0.0, 0.0))
}

/// 素材の尺を comp のコマ数に直す。0.2 秒に満たない物(静止画の nb_frames=1 など)は尺無し。
fn source_frames_in(
    info: &crate::render::media::MediaInfo,
    fps: crate::doc::store::Fps,
) -> Option<i64> {
    let secs = info.duration.map(|d| d.as_seconds_f64()).or_else(|| {
        info.nb_frames
            .map(|n| n as f64 / info.fps.as_f64().max(1e-9))
    })?;
    if secs < 0.2 {
        return None;
    }
    Some((secs * fps.as_f64()).round().max(1.0) as i64)
}

/// 四角の初期辺。comp の短辺の 1/4 —— 4K で点にならず、SD で枠を覆わない。
fn rect_side(comp: (f64, f64)) -> f64 {
    (comp.0.min(comp.1) * 0.25).round().max(1.0)
}

/// 既にある名前なら `Text 2`(Finder・Figma)。
fn numbered(base: &str, taken: &[String]) -> String {
    if !taken.iter().any(|n| n == base) {
        return base.to_owned();
    }
    (2..)
        .map(|k| format!("{base} {k}"))
        .find(|candidate| !taken.iter().any(|n| n == candidate))
        .expect("an unbounded range always yields")
}

fn new_layer_intents(
    layer: LayerId,
    order: i16,
    playhead: i64,
    duration_frames: i64,
    fps: crate::doc::store::Fps,
    comp: (f64, f64),
    kind: NewKind,
) -> Vec<Intent> {
    let label_color = Some(Some((layer.0 % fixture::LABEL_PALETTE.len() as u64) as u8));
    match kind {
        NewKind::Media { path, name } => {
            let spatial = crate::render::media::is_point_cloud_path(&path)
                || crate::render::media::is_mesh_path(&path);
            let info = if spatial {
                None
            } else {
                crate::render::media::probe(&path).ok()
            };
            let fit = if spatial {
                spatial_fit_intents(layer, &path, comp)
            } else {
                let natural = info
                    .as_ref()
                    .map(|i| (i.width as f64, i.height as f64))
                    .unwrap_or((0.0, 0.0));
                center_intents(layer, natural, comp)
            };
            // 動画は**素材の尺**で入る(Premiere・Resolve)。静止画と尺の無い物は comp の終わりまで。
            let source_frames = info.as_ref().and_then(|i| source_frames_in(i, fps));
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::File {
                            path,
                            fingerprint: None,
                        },
                        order,
                        timing: LayerTiming::place(playhead, source_frames, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        name: Some(name),
                        label_color,
                        projection: Some(if spatial {
                            LayerProjection::ThreeD
                        } else {
                            LayerProjection::TwoPointFiveD
                        }),
                        ..Default::default()
                    },
                },
            ];
            out.extend(fit);
            out
        }
        NewKind::Rectangle => {
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order,
                        timing: LayerTiming::place(playhead, None, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        name: Some("Rectangle".to_owned()),
                        label_color,
                        projection: Some(LayerProjection::TwoPointFiveD),
                        ..Default::default()
                    },
                },
                Intent::SetShapes {
                    layer,
                    shapes: vec![ShapeNode::Leaf(Shape {
                        source: PathSource::Rectangle {
                            size: VectorPoint {
                                x: rect_side(comp),
                                y: rect_side(comp),
                            },
                        },
                        ops: Vec::new(),
                        fill: Some(Fill {
                            brush: Brush::Solid(Rgb {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                            }),
                            rule: FillRule::NonZero,
                            opacity: 1.0,
                            hidden: false,
                        }),
                        stroke: None,
                    })],
                },
            ];
            let shapes = match out.last() {
                Some(Intent::SetShapes { shapes, .. }) => shapes.clone(),
                _ => Vec::new(),
            };
            out.extend(center_intents(layer, shape_natural(&shapes), comp));
            out
        }
        NewKind::Bezier => {
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order,
                        timing: LayerTiming::place(playhead, None, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        name: Some("Bezier".to_owned()),
                        label_color,
                        projection: Some(LayerProjection::TwoPointFiveD),
                        ..Default::default()
                    },
                },
                Intent::SetShapes {
                    layer,
                    shapes: vec![ShapeNode::Leaf(Shape {
                        source: PathSource::Bezier(vec![Contour {
                            closed: false,
                            vertices: vec![
                                Vertex {
                                    point: VectorPoint { x: -150.0, y: 0.0 },
                                    in_tangent: VectorPoint { x: 0.0, y: 0.0 },
                                    out_tangent: VectorPoint {
                                        x: 100.0,
                                        y: -150.0,
                                    },
                                },
                                Vertex {
                                    point: VectorPoint { x: 150.0, y: 0.0 },
                                    in_tangent: VectorPoint {
                                        x: -100.0,
                                        y: 150.0,
                                    },
                                    out_tangent: VectorPoint { x: 0.0, y: 0.0 },
                                },
                            ],
                        }]),
                        ops: Vec::new(),
                        fill: None,
                        stroke: Some(crate::doc::vector::Stroke {
                            brush: Brush::Solid(Rgb {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                            }),
                            width: 6.0,
                            cap: crate::doc::vector::LineCap::Round,
                            join: crate::doc::vector::LineJoin::Round,
                            miter_limit: 4.0,
                            opacity: 1.0,
                            hidden: false,
                            dash: None,
                        }),
                    })],
                },
            ];
            let shapes = match out.last() {
                Some(Intent::SetShapes { shapes, .. }) => shapes.clone(),
                _ => Vec::new(),
            };
            out.extend(center_intents(layer, shape_natural(&shapes), comp));
            out
        }
        NewKind::Text => {
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Text,
                        order,
                        timing: LayerTiming::place(playhead, None, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        name: Some("Text".to_owned()),
                        label_color,
                        projection: Some(LayerProjection::TwoPointFiveD),
                        ..Default::default()
                    },
                },
                Intent::SetTextDocument {
                    layer,
                    document: TextDocument {
                        content: {
                            let mut track = ContentTrack::new();
                            track.insert(ContentKeyframe {
                                t: RationalTime::try_from_frame(playhead, fps)
                                    .unwrap_or(RationalTime::ZERO),
                                content: "Text".to_owned(),
                            });
                            track
                        },
                        justify: TextJustify::Center,
                        wrap_size: None,
                        styles: vec![TextDocumentStyle {
                            id: TextStyleId(0),
                            font: FontRef {
                                path: "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc".to_owned(),
                                fingerprint: None,
                                family: "Hiragino Sans".to_owned(),
                                style: "W3".to_owned(),
                            },
                            size: 96.0,
                            fill: [1.0, 1.0, 1.0, 1.0],
                            // 日本語の歌詞の既定: 行送り 1.5、約物を詰める(palt)、黒の縁取り(背景が動画でも読める)。
                            line_height: Some(96.0 * 1.5),
                            tracking: 0.0,
                            stroke_color: Some([0.0, 0.0, 0.0, 1.0]),
                            stroke_width: 96.0 * 0.08,
                            stroke_over_fill: false,
                            axes: Vec::new(),
                            features: vec![crate::doc::store::TextStyleFeature {
                                tag: "palt".to_owned(),
                                value: 1,
                            }],
                        }],
                        slot_id: None,
                        ranges: Vec::new(),
                        alignment: TextAlignmentOptions::default(),
                        runs: Vec::new(),
                    },
                },
            ];
            // 文字は組んでみるまで大きさが決まらない。実寸が要らない形で
            // 真ん中へ置く —— 左上を枠の中心に合わせる。
            out.extend(
                // 文字の箱は枠と同じ幅・左上起点。中央揃えが枠の中心軸に乗る(揃えは箱の幅で決まる)。
                center_intents(layer, comp, comp),
            );
            out
        }
    }
}

fn spawn_layer(
    doc: &Arc<Mutex<Document>>,
    clock: &Clock,
    mut layer_rows: Signal<Vec<LayerRow>>,
    mut attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: &Sender<TimelineMsg>,
    kind: NewKind,
    label: &'static str,
    mut revision: Signal<u32>,
) {
    // 最初の曲・動画の probe(ffprobe の process)は lock の外で。握ったまま起こすと窓が止まる。
    let probed = match &kind {
        NewKind::Media { path, .. } => {
            let fresh = {
                let doc = doc.lock().unwrap();
                let view = doc.view();
                !view.layers().into_iter().any(|layer| {
                    matches!(
                        view.meta(layer).ok().flatten().map(|meta| meta.source),
                        Some(LayerSource::File { .. })
                    )
                })
            };
            fresh
                .then(|| crate::render::media::probe(path).ok())
                .flatten()
        }
        _ => None,
    };
    let mut d = doc.lock().unwrap();
    let layer = LayerId(d.view().next_layer_id());
    let order = d
        .view()
        .layers()
        .iter()
        .filter_map(|l| d.view().meta(*l).ok().flatten().map(|m| m.order))
        .max()
        .map(|m| m.saturating_add(1))
        .unwrap_or(0);
    let composition = d.view().composition().ok().flatten();
    let fps = composition
        .as_ref()
        .map(|composition| composition.fps)
        .unwrap_or_else(|| crate::doc::store::Fps::try_new(30, 1).expect("30fps"));
    let playhead = clock.current_frame();
    let duration_frames = composition
        .as_ref()
        .map(|composition| composition.duration_frames)
        .unwrap_or(1800);
    let comp_size = composition
        .as_ref()
        .map(|composition| (composition.width as f64, composition.height as f64))
        .unwrap_or((1920.0, 1080.0));
    // 空の作品に最初の曲・動画が来たら、尺を素材に合わせる(60 秒の既定で 3 分の曲を黙って切らない)。
    let grow_to = probed
        .as_ref()
        .and_then(|i| source_frames_in(i, fps))
        .filter(|f| *f > duration_frames);
    let duration_frames = grow_to.unwrap_or(duration_frames);
    let mut intents = new_layer_intents(
        layer,
        order,
        playhead,
        duration_frames,
        fps,
        comp_size,
        kind,
    );
    if let (Some(frames), Some(comp)) = (grow_to, composition.as_ref()) {
        intents.insert(
            0,
            Intent::SetComposition(crate::doc::store::Composition {
                duration_frames: frames,
                ..comp.clone()
            }),
        );
        println!("PROBE room=write verdict=composition-grown frames={frames}");
    }
    let taken: Vec<String> = d
        .view()
        .layers()
        .into_iter()
        .filter_map(|l| d.view().attrs(l).ok().flatten().map(|a| a.name))
        .collect();
    for intent in &mut intents {
        if let Intent::SetAttrs { patch, .. } = intent {
            if let Some(name) = patch.name.take() {
                patch.name = Some(numbered(&name, &taken));
            }
        }
    }
    match d.apply_all(intents) {
        Ok(_) => {
            let rows = fixture::layer_rows_from_doc(&d);
            let attrs_vec = rows
                .iter()
                .map(|r| (r.hidden, r.solo, r.locked))
                .collect::<Vec<_>>();
            let canvas_rows = fixture::canvas_rows_from_doc(&d);
            drop(d);
            *layer_rows.write() = rows;
            *attrs_state.write() = attrs_vec;
            timeline_tx.send(TimelineMsg::SetRows(canvas_rows)).ok();
            *revision.write() += 1;
            println!(
                "PROBE room=write verdict=created kind={label} layer={}",
                layer.0
            );
        }
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
    }
}

fn replace_source(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    path: String,
    mut revision: Signal<u32>,
) {
    let mut d = doc.lock().unwrap();
    let source = crate::doc::store::LayerSource::File {
        path,
        fingerprint: None,
    };
    let applied = d.apply(Intent::SetSource { layer, source }).is_ok();
    drop(d);
    if applied {
        *revision.write() += 1;
    }
}

fn mask_frame(doc: &Document, layer: LayerId) -> Option<[f64; 2]> {
    let view = doc.view();
    let meta = view.meta(layer).ok().flatten()?;
    let comp = view.composition().ok().flatten()?;
    match meta.source {
        LayerSource::Text => Some([comp.width as f64, comp.height as f64]),
        LayerSource::Shape => {
            let shapes = view.shapes(layer).ok()?;
            let (width, height) = shape_natural(&shapes);
            (width > 0.0 && height > 0.0).then_some([width, height])
        }
        LayerSource::File { path, .. }
            if !crate::render::media::is_mesh_path(&path)
                && !crate::render::media::is_point_cloud_path(&path) =>
        {
            let info = crate::render::media::probe(&path).ok()?;
            Some([info.width as f64, info.height as f64])
        }
        LayerSource::File { .. } | LayerSource::Null | LayerSource::Group => None,
    }
}

fn add_rectangle_mask(doc: &Arc<Mutex<Document>>, layer: LayerId, mut revision: Signal<u32>) {
    let mut d = doc.lock().unwrap();
    let Some([width, height]) = mask_frame(&d, layer) else {
        println!(
            "PROBE room=write verdict=mask-skip layer={} reason=no-2d-frame",
            layer.0
        );
        return;
    };
    let next_id = d
        .view()
        .masks(layer)
        .unwrap_or_default()
        .iter()
        .map(|mask| mask.id.0)
        .max()
        .map(|id| id + 1)
        .unwrap_or(0);
    let x0 = width * 0.2;
    let x1 = width * 0.8;
    let y0 = height * 0.2;
    let y1 = height * 0.8;
    let mut shape = KeyframeTrack::new();
    shape.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::Path(Path {
            vertices: [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
                .into_iter()
                .map(|point| PathVertex {
                    point,
                    in_tangent: [0.0, 0.0],
                    out_tangent: [0.0, 0.0],
                })
                .collect(),
            closed: true,
        }),
        interp: Interp::Hold,
        spatial: None,
    });
    match d.apply(Intent::AddMask {
        layer,
        mask: Mask {
            id: MaskId(next_id),
            mode: MaskMode::Add,
            inverted: false,
        },
        shape,
    }) {
        Ok(_) => {
            drop(d);
            *revision.write() += 1;
            println!(
                "PROBE room=write verdict=mask-added layer={} id={next_id}",
                layer.0
            );
        }
        Err(error) => println!("PROBE room=write verdict=apply-error {error}"),
    }
}

fn commit_effect_selection(
    session: &Session,
    ordered: &[BrowserItemId],
    mut revision: Signal<u32>,
    poke: &crate::ui::poke::Poke,
) {
    let plugins: Vec<String> = session
        .browser_selection
        .lock()
        .unwrap()
        .selected_or_active_in_order(BrowserScope::Effects, ordered)
        .into_iter()
        .filter_map(|id| match id {
            BrowserItemId::Effect(plugin) => Some(plugin),
            _ => None,
        })
        .collect();
    if plugins.is_empty() {
        return;
    }
    if session.selection.all().is_empty() {
        *session.project_notice.lock().unwrap() = "Select a layer before adding effects".into();
        poke.poke();
        return;
    }
    match session.apply_each(None, |doc, target| {
        effect_batch_intents(doc, target, &plugins)
    }) {
        Ok(0) => {
            *session.project_notice.lock().unwrap() =
                "Selected effects are already attached".into();
            poke.poke();
        }
        Ok(_) => *revision.write() += 1,
        Err(error) => println!("PROBE room=write verdict=apply-error {error}"),
    }
}

fn commit_color(
    session: &Session,
    rgba: [u8; 4],
    mut revision: Signal<u32>,
    poke: &crate::ui::poke::Poke,
) {
    let rgb = [
        rgba[0] as f64 / 255.0,
        rgba[1] as f64 / 255.0,
        rgba[2] as f64 / 255.0,
    ];
    if let Some(crate::ui::session::Focus::Color(
        slot @ crate::ui::session::ColorSlot::TextStroke { .. },
    )) = session.live_focus()
    {
        if session.writable(slot.layer()) {
            match crate::ui::color::write_color(&session.doc, &slot, rgb) {
                Ok(()) => *revision.write() += 1,
                Err(error) => println!("PROBE room=write verdict=apply-error {error}"),
            }
        }
        return;
    }
    if session.selection.all().is_empty() {
        *session.project_notice.lock().unwrap() = "Select a layer before applying a color".into();
        poke.poke();
        return;
    }
    let result = session.apply_blocks(None, |doc, target| color_block(doc, target, rgba));
    crate::ui::session::noted(result.map(|_| ()), revision);
}

#[allow(clippy::too_many_arguments)]
fn commit_active_media(
    session: &Session,
    ordered: &[BrowserItemId],
    commands: &[(crate::doc::store::AssetId, Option<String>, String)],
    layer_rows: Signal<Vec<LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: &Sender<TimelineMsg>,
    revision: Signal<u32>,
    poke: &crate::ui::poke::Poke,
) {
    let active = session
        .browser_selection
        .lock()
        .unwrap()
        .active_in(BrowserScope::Media, ordered);
    let Some(BrowserItemId::Media(active)) = active else {
        return;
    };
    let Some((_, path, name)) = commands.iter().find(|(id, _, _)| *id == active) else {
        return;
    };
    let Some(path) = path else {
        *session.project_notice.lock().unwrap() = "This media file is missing".into();
        poke.poke();
        return;
    };
    spawn_layer(
        &session.doc,
        &session.clock,
        layer_rows,
        attrs_state,
        timeline_tx,
        NewKind::Media {
            path: path.clone(),
            name: name.clone(),
        },
        "media",
        revision,
    );
}

pub(super) fn media_context_info(
    session: &Session,
    asset: crate::doc::store::AssetId,
) -> Option<(String, Option<String>, bool)> {
    let doc = session.doc.lock().unwrap();
    let view = doc.view();
    let row = fixture::asset_rows_from_view(&view)
        .into_iter()
        .find(|row| row.id == asset)?;
    let in_use = row.path.as_ref().is_some_and(|path| {
        view.layers().into_iter().any(|layer| {
            view.meta(layer)
                .ok()
                .flatten()
                .is_some_and(|meta| matches!(meta.source, LayerSource::File { path: source, .. } if source == *path))
        })
    });
    Some((row.name, row.path, in_use))
}

pub(super) fn place_media_asset(
    session: &Session,
    asset: crate::doc::store::AssetId,
    panes: crate::ui::app::Panes,
) {
    let Some((name, path, _)) = media_context_info(session, asset) else {
        return;
    };
    let Some(path) = path else {
        *session.project_notice.lock().unwrap() = "This media file is missing".into();
        return;
    };
    spawn_layer(
        &session.doc,
        &session.clock,
        panes.layer_rows,
        panes.attrs_state,
        &session.timeline_tx,
        NewKind::Media { path, name },
        "media",
        panes.revision,
    );
}

pub(super) fn replace_with_media_asset(
    session: &Session,
    asset: crate::doc::store::AssetId,
    panes: crate::ui::app::Panes,
) {
    let Some(layer) = session.selection.get().filter(|layer| session.writable(*layer)) else {
        *session.project_notice.lock().unwrap() = "Select an editable layer to replace".into();
        return;
    };
    let Some((_, Some(path), _)) = media_context_info(session, asset) else {
        *session.project_notice.lock().unwrap() = "This media file is missing".into();
        return;
    };
    replace_source(&session.doc, layer, path, panes.revision);
}

pub(super) fn remove_media_asset(
    session: &Session,
    asset: crate::doc::store::AssetId,
    mut revision: Signal<u32>,
) {
    let Some((_, _, in_use)) = media_context_info(session, asset) else {
        return;
    };
    if in_use {
        *session.project_notice.lock().unwrap() = "This media is still in use".into();
        return;
    }
    match session.doc.lock().unwrap().apply(Intent::RemoveAsset { asset }) {
        Ok(()) => {
            *revision.write() += 1;
            *session.project_notice.lock().unwrap() = "Removed media".into();
        }
        Err(error) => *session.project_notice.lock().unwrap() = error.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
fn commit_create_item(
    session: &Session,
    item: CreateItem,
    layer_rows: Signal<Vec<LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: &Sender<TimelineMsg>,
    revision: Signal<u32>,
    poke: &crate::ui::poke::Poke,
) {
    let kind = match item {
        CreateItem::Text => Some((NewKind::Text, "Text")),
        CreateItem::Rectangle => Some((NewKind::Rectangle, "Rectangle")),
        CreateItem::Bezier => Some((NewKind::Bezier, "Bezier")),
        CreateItem::Mask => None,
    };
    if let Some((kind, label)) = kind {
        spawn_layer(
            &session.doc,
            &session.clock,
            layer_rows,
            attrs_state,
            timeline_tx,
            kind,
            label,
            revision,
        );
        return;
    }

    let layer = session
        .selection
        .get()
        .filter(|layer| mask_frame(&session.doc.lock().unwrap(), *layer).is_some());
    if let Some(layer) = layer {
        add_rectangle_mask(&session.doc, layer, revision);
    } else {
        *session.project_notice.lock().unwrap() = "Select a 2D layer before adding a mask".into();
        poke.poke();
    }
}

fn delete_selected_media(
    session: &Session,
    ordered: &[BrowserItemId],
    all: &[BrowserItemId],
    used: &std::collections::BTreeSet<crate::doc::store::AssetId>,
    mut revision: Signal<u32>,
    poke: &crate::ui::poke::Poke,
) {
    let selected = session
        .browser_selection
        .lock()
        .unwrap()
        .selected_in_order(BrowserScope::Media, ordered);
    let selected_count = selected.len();
    let removable: Vec<_> = selected
        .into_iter()
        .filter_map(|id| match id {
            BrowserItemId::Media(asset) if !used.contains(&asset) => Some(asset),
            _ => None,
        })
        .collect();
    if removable.is_empty() {
        if selected_count > 0 {
            *session.project_notice.lock().unwrap() = "Selected media is still in use".into();
            poke.poke();
        }
        return;
    }
    let result = session.doc.lock().unwrap().apply_all(
        removable
            .iter()
            .copied()
            .map(|asset| Intent::RemoveAsset { asset }),
    );
    match result {
        Ok(()) => {
            let remaining: Vec<_> = all
                .iter()
                .filter(
                    |id| !matches!(id, BrowserItemId::Media(asset) if removable.contains(asset)),
                )
                .cloned()
                .collect();
            session
                .browser_selection
                .lock()
                .unwrap()
                .reconcile(BrowserScope::Media, &remaining);
            let kept = selected_count.saturating_sub(removable.len());
            *session.project_notice.lock().unwrap() = match kept {
                0 => format!("Removed {} media", removable.len()),
                _ => format!("Removed {} media · {kept} still in use", removable.len()),
            };
            *revision.write() += 1;
            poke.poke();
        }
        Err(error) => println!("PROBE room=write verdict=apply-error {error}"),
    }
}

pub(super) fn browser_panel(
    session: &Session,
    doc: Arc<Mutex<Document>>,
    layer_rows: Signal<Vec<LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: Sender<TimelineMsg>,
    selected: Signal<Option<LayerId>>,
    revision: Signal<u32>,
    layout_tick: u32,
    panel: Panel,
    mut rail: Signal<Option<fixture::AssetFamily>>,
    poke: crate::ui::poke::Poke,
    mut search_node: Signal<Option<std::rc::Rc<MountedData>>>,
    mut marquee: Signal<Option<BrowserMarquee>>,
    menu: Signal<Option<crate::ui::context_menu::MenuRequest>>,
) -> Element {
    let scope = browser_scope(panel);
    let focusable_items = consume_context::<FocusableItems>();
    let query_value = session
        .browser_selection
        .lock()
        .unwrap()
        .query(scope)
        .to_owned();
    let query = query_value.clone();
    let search_id = format!("browser-search-{panel}");
    let grid_class = match layout_tick {
        0 => "tgrid",
        tick if tick % 2 == 0 => "tgrid browser-reflow-a",
        _ => "tgrid browser-reflow-b",
    };
    let color_grid_class = match layout_tick {
        0 => "tgrid color-grid",
        tick if tick % 2 == 0 => "tgrid color-grid browser-reflow-a",
        _ => "tgrid color-grid browser-reflow-b",
    };
    let rail_class = move |f: Option<fixture::AssetFamily>| {
        if rail() == f {
            "srow on"
        } else {
            "srow"
        }
    };
    // 棚は Document から引き直す。窓へ落ちてきた素材はここにしか現れない。
    let _ = revision();
    let assets = fixture::asset_rows_from_view(&doc.lock().unwrap().view());
    let families = {
        let mut f: Vec<_> = assets.iter().map(|a| a.family).collect();
        f.sort();
        f.dedup();
        f
    };
    let shown: Vec<_> = assets
        .iter()
        .filter(|a| rail().is_none_or(|f| a.family == f) && query_matches(&a.name, &query))
        .collect();
    let rail_label = rail().map_or("All media", |f| f.label());
    // 層が使っている素材は棚から外せない(外すと層が空を指す)。
    let used: std::collections::HashSet<String> = {
        let d = doc.lock().unwrap();
        let view = d.view();
        view.layers()
            .into_iter()
            .filter_map(|l| view.meta(l).ok().flatten())
            .filter_map(|m| match m.source {
                crate::doc::store::LayerSource::File { path, .. } => Some(path),
                _ => None,
            })
            .collect()
    };

    let all_media_ids: Vec<_> = assets
        .iter()
        .map(|asset| BrowserItemId::Media(asset.id))
        .collect();
    let search_all_ids = match scope {
        BrowserScope::Media => all_media_ids.clone(),
        BrowserScope::Effects => crate::render::engine::known_effects()
            .iter()
            .map(|effect| BrowserItemId::Effect(effect.plugin_id.clone()))
            .collect(),
        BrowserScope::Create => [
            CreateItem::Text,
            CreateItem::Rectangle,
            CreateItem::Bezier,
            CreateItem::Mask,
        ]
        .into_iter()
        .map(BrowserItemId::Create)
        .collect(),
        BrowserScope::Colors => {
            let mut swatches = fixture::used_colors_from_doc(&doc.lock().unwrap());
            for swatch in fixture::default_palette() {
                if !swatches.iter().any(|known| known.rgba == swatch.rgba) {
                    swatches.push(swatch);
                }
            }
            swatches
                .into_iter()
                .map(|swatch| BrowserItemId::Color(swatch.rgba))
                .collect()
        }
    };
    let media_ids: Vec<_> = shown
        .iter()
        .map(|asset| BrowserItemId::Media(asset.id))
        .collect();
    let used_asset_ids: std::collections::BTreeSet<_> = assets
        .iter()
        .filter(|asset| asset.path.as_ref().is_some_and(|path| used.contains(path)))
        .map(|asset| asset.id)
        .collect();
    let media_commands: Vec<_> = shown
        .iter()
        .map(|asset| (asset.id, asset.path.clone(), asset.name.clone()))
        .collect();
    let media_selection = {
        let mut selection = session.browser_selection.lock().unwrap();
        selection.reconcile(BrowserScope::Media, &all_media_ids);
        selection.ensure_active_in(BrowserScope::Media, &media_ids);
        selection.clone()
    };
    let media_active = media_selection.active_in(BrowserScope::Media, &media_ids);

    let media_count = shown.len();
    let asset_cards = shown.iter().enumerate().map(|(index, a)| {
        let preview = a.preview.as_deref();
        let in_use = a.path.as_ref().is_some_and(|p| used.contains(p));
        let asset_id = a.id;
        let item_id = BrowserItemId::Media(asset_id);
        let focus_key = item_id.focus_key();
        let card_class = if media_selection.is_selected(BrowserScope::Media, &item_id) {
            "tcard on"
        } else {
            "tcard"
        };
        let reveal_path = a.path.clone();
        let replace_path = a.path.clone();
        let replace_doc = doc.clone();
        let remove_doc = doc.clone();
        let select_session = session.clone();
        let select_ids = media_ids.clone();
        let select_item = item_id.clone();
        let select_poke = poke.clone();
        let commit_session = session.clone();
        let commit_ids = media_ids.clone();
        let commit_item = item_id.clone();
        let commit_poke = poke.clone();
        let commit_path = a.path.clone();
        let commit_name = a.name.clone();
        let commit_timeline = timeline_tx.clone();
        let context_session = session.clone();
        let context_ids = media_ids.clone();
        let context_item = item_id.clone();
        let context_poke = poke.clone();
        let mut context_menu = menu;
        rsx!(
            div { class: "tcell",
                SemanticButton {
                    class: "{card_class}",
                    role: "gridcell",
                    aria_selected: if media_selection.is_selected(BrowserScope::Media, &item_id) { "true" } else { "false" },
                    aria_posinset: Some((index + 1).to_string()),
                    aria_setsize: Some(media_count.to_string()),
                    focus_key: Some(focus_key),
                    tabindex: Some(result_tabindex(media_active.as_ref(), &item_id)),
                    selected: media_selection.is_selected(BrowserScope::Media, &item_id),
                    title: if a.path.is_some() { "Select · double-click or Enter to add as a layer" } else { "Missing media · select it to remove or relink" },
                    onclick: move |evt: MouseEvent| choose_browser_item(
                        &select_session,
                        BrowserScope::Media,
                        &select_item,
                        &select_ids,
                        evt.modifiers(),
                        true,
                        &select_poke,
                    ),
                    ondoubleclick: move |evt: MouseEvent| {
                        choose_browser_item(
                            &commit_session,
                            BrowserScope::Media,
                            &commit_item,
                            &commit_ids,
                            evt.modifiers(),
                            true,
                            &commit_poke,
                        );
                        if let Some(path) = &commit_path {
                            spawn_layer(
                                &commit_session.doc,
                                &commit_session.clock,
                                layer_rows,
                                attrs_state,
                                &commit_timeline,
                                NewKind::Media { path: path.clone(), name: commit_name.clone() },
                                "media",
                                revision,
                            );
                        } else {
                            *commit_session.project_notice.lock().unwrap() = "This media file is missing".into();
                            commit_poke.poke();
                        }
                    },
                    oncontextmenu: move |evt: MouseEvent| {
                        evt.prevent_default();
                        evt.stop_propagation();
                        let changed = {
                            let mut selection = context_session.browser_selection.lock().unwrap();
                            if selection.is_selected(BrowserScope::Media, &context_item) {
                                selection.activate(BrowserScope::Media, &context_item, &context_ids)
                            } else {
                                selection.click(
                                    BrowserScope::Media,
                                    &context_item,
                                    &context_ids,
                                    false,
                                    false,
                                )
                            }
                        };
                        if changed {
                            context_poke.poke();
                        }
                        let point = evt.client_coordinates();
                        context_menu.set(Some(crate::ui::context_menu::MenuRequest {
                            x: point.x,
                            y: point.y,
                            target: crate::ui::context_menu::MenuTarget::BrowserMedia(asset_id),
                        }));
                    },
                    if let Some(src) = preview {
                        img { class: "thumb", src: "{src}", alt: "" }
                    } else {
                        div { class: "thumb", style: "background:{a.thumb};" }
                    }
                    span { class: "tname", "{a.name}" }
                    span { class: "tmeta", if in_use { "{a.kind} · in use" } else { "{a.kind}" } }
                }
                // 札の上に出る手。隠し技(Alt+click)を表に出す(Premiere の Replace Footage、Finder の Reveal)。
                div {
                    class: "tacts",
                    onkeydown: move |evt: KeyboardEvent| {
                        if matches!(
                            evt.key(),
                            Key::ArrowLeft
                                | Key::ArrowRight
                                | Key::ArrowUp
                                | Key::ArrowDown
                                | Key::Home
                                | Key::End
                                | Key::PageUp
                                | Key::PageDown
                        ) {
                            evt.prevent_default();
                            evt.stop_propagation();
                        }
                    },
                    if let (Some(layer), Some(path)) = (selected(), replace_path) {
                        SemanticButton {
                            class: "chip",
                            title: "Replace the selected layer's source with this",
                            onclick: {
                                let session = session.clone();
                                move |_| {
                                    if session.writable(layer) {
                                        replace_source(&replace_doc, layer, path.clone(), revision)
                                    }
                                }
                            },
                            "Replace"
                        }
                    }
                    if let Some(path) = reveal_path {
                        SemanticButton {
                            class: "chip",
                            aria_label: "Reveal in Finder",
                            title: "Reveal in Finder",
                            onclick: move |_| crate::ui::output::reveal_in_finder(std::path::Path::new(&path)),
                            "Finder"
                        }
                    }
                    // 使用中は × を出さない(押せるのに反応しない、より正しい)。理由は tmeta に。
                    if !in_use {
                    SemanticButton {
                        class: "chip",
                        aria_label: "Remove from library",
                        title: "Remove from library",
                        onclick: move |_| {
                            crate::ui::session::noted(remove_doc.lock().unwrap().apply(Intent::RemoveAsset { asset: asset_id }), revision)
                        },
                        "×"
                    }
                    }
                }
            }
        )
    });
    let library_line = {
        let bytes: u64 = shown.iter().filter_map(|a| a.size).sum();
        let count = shown.len();
        let noun = if count == 1 { "item" } else { "items" };
        format!("{count} {noun} · {}", fixture::human_size(bytes))
    };
    let asset_count = shown.len();
    let filtered_out = shown.is_empty() && rail().is_some();
    let search_session = session.clone();
    let search_poke = poke.clone();
    let search_key_session = session.clone();
    let search_key_poke = poke.clone();
    let search_focus_node = search_node;
    let search_focus_items = focusable_items;
    let search_focus_ids = search_all_ids.clone();
    let can_marquee = matches!(scope, BrowserScope::Media | BrowserScope::Effects);
    let select_mode = session.browser_selection.lock().unwrap().select_mode(scope);
    let mode_session = session.clone();
    let mode_poke = poke.clone();

    rsx!(
        div { id: "browser",
            div { class: "rhead", style: "flex:0 0 auto;",
                input {
                    id: "{search_id}",
                    r#type: "search",
                    class: "csheet-in",
                    style: "width:100%; min-width:0;",
                    aria_label: "Search Browser",
                    placeholder: "Search {panel}",
                    value: "{query_value}",
                    onmounted: move |evt: MountedEvent| search_node.set(Some(evt.data())),
                    oninput: move |evt: FormEvent| {
                        if search_session
                            .browser_selection
                            .lock()
                            .unwrap()
                            .set_query(scope, evt.value())
                        {
                            search_poke.poke();
                        }
                    },
                    onkeydown: move |evt: KeyboardEvent| {
                        let command = crate::ui::keymap::primary_modifier(evt.modifiers());
                        if command
                            && matches!(evt.key(), Key::Character(ref c) if c.eq_ignore_ascii_case("f"))
                        {
                            evt.prevent_default();
                            evt.stop_propagation();
                            focus_and_select_search(search_focus_node.read().clone());
                            return;
                        }
                        if command
                            && matches!(evt.key(), Key::Character(ref c) if c.eq_ignore_ascii_case("a"))
                        {
                            // Keep the native text input's Cmd+A default action; only stop the
                            // Browser/artwork handlers above it.
                            evt.stop_propagation();
                            return;
                        }
                        if command
                            && matches!(evt.key(), Key::Character(ref c) if matches!(c.to_ascii_lowercase().as_str(), "c" | "v" | "x"))
                        {
                            // Clipboard editing stays with the native input.
                            evt.stop_propagation();
                            return;
                        }
                        if command
                            && matches!(evt.key(), Key::Character(ref c) if matches!(c.to_ascii_lowercase().as_str(), "d" | "g" | "k"))
                        {
                            evt.prevent_default();
                            evt.stop_propagation();
                            browser_action_unavailable(&search_key_session, &search_key_poke);
                            return;
                        }
                        if evt.key() == Key::Escape {
                            evt.prevent_default();
                            evt.stop_propagation();
                            let (changed, focus) = {
                                let mut selection = search_key_session.browser_selection.lock().unwrap();
                                let changed = selection.clear_query(scope);
                                selection.ensure_active_in(scope, &search_focus_ids);
                                (
                                    changed,
                                    selection.active_in(scope, &search_focus_ids),
                                )
                            };
                            if changed {
                                search_key_poke.poke();
                            }
                            if let Some(focus) = focus {
                                search_focus_items.focus(&focus.focus_key());
                            }
                        }
                    },
                }
                if can_marquee {
                    SemanticButton {
                        class: if select_mode { "chip on" } else { "chip" },
                        selected: select_mode,
                        aria_label: "Select several Browser items",
                        title: "Drag a rectangle to select several items",
                        onclick: move |_| {
                            let enabled = mode_session
                                .browser_selection
                                .lock()
                                .unwrap()
                                .toggle_select_mode(scope);
                            if !enabled {
                                marquee.set(None);
                                mode_session.gesture.end();
                            }
                            mode_poke.poke();
                        },
                        "Select"
                    }
                }
            }
            if panel == Panel::Colors {
                {
                    let layer = selected();
                    // 白紙では使われた色が無い。最初の一歩は既定のパレットから(Canva・CapCut)。
                    let used = fixture::used_colors_from_doc(&doc.lock().unwrap());
                    let used_count = used.len();
                    // 使われた色の後ろに既定のパレットを繋ぐ(1 色使った瞬間に棚が空にならない)。
                    let mut swatches = used;
                    for sw in fixture::default_palette() {
                        if !swatches.iter().any(|s| s.rgba == sw.rgba) {
                            swatches.push(sw);
                        }
                    }
                    let all_color_ids: Vec<_> = swatches
                        .iter()
                        .map(|swatch| BrowserItemId::Color(swatch.rgba))
                        .collect();
                    {
                        let mut selection = session.browser_selection.lock().unwrap();
                        selection.reconcile(BrowserScope::Colors, &all_color_ids);
                    }
                    swatches.retain(|swatch| query_matches(&swatch.hex, &query));
                    let has_swatches = !swatches.is_empty();
                    let color_ids: Vec<_> = swatches
                        .iter()
                        .map(|swatch| BrowserItemId::Color(swatch.rgba))
                        .collect();
                    let color_selection = {
                        let mut selection = session.browser_selection.lock().unwrap();
                        selection.ensure_active_in(BrowserScope::Colors, &color_ids);
                        selection.clone()
                    };
                    let color_active = color_selection.active_in(BrowserScope::Colors, &color_ids);
                    let color_count = swatches.len();
                    let cards = swatches.into_iter().enumerate().map(|(index, ColorSwatch { hex, rgba })| {
                        let item_id = BrowserItemId::Color(rgba);
                        let is_selected = color_selection.is_selected(BrowserScope::Colors, &item_id);
                        let card_class = if is_selected { "tcard color-swatch on" } else { "tcard color-swatch" };
                        let focus_key = item_id.focus_key();
                        let select_session = session.clone();
                        let select_item = item_id.clone();
                        let select_ids = color_ids.clone();
                        let select_poke = poke.clone();
                        let commit_session = session.clone();
                        let commit_item = item_id.clone();
                        let commit_ids = color_ids.clone();
                        let commit_poke = poke.clone();
                        rsx!(
                            SemanticButton {
                                class: "{card_class}",
                                role: "gridcell",
                                aria_selected: if is_selected { "true" } else { "false" },
                                aria_posinset: Some((index + 1).to_string()),
                                aria_setsize: Some(color_count.to_string()),
                                focus_key: Some(focus_key),
                                tabindex: Some(result_tabindex(color_active.as_ref(), &item_id)),
                                selected: is_selected,
                                title: if layer.is_none() { "Select · choose a layer before applying" } else if session.selection.all().len() > 1 { "Select · double-click or Enter to apply to the selected layers" } else { "Select · double-click or Enter to apply to the selected layer" },
                                onclick: move |evt: MouseEvent| choose_browser_item(
                                    &select_session,
                                    BrowserScope::Colors,
                                    &select_item,
                                    &select_ids,
                                    evt.modifiers(),
                                    false,
                                    &select_poke,
                                ),
                                ondoubleclick: move |evt: MouseEvent| {
                                    choose_browser_item(
                                        &commit_session,
                                        BrowserScope::Colors,
                                        &commit_item,
                                        &commit_ids,
                                        evt.modifiers(),
                                        false,
                                        &commit_poke,
                                    );
                                    commit_color(&commit_session, rgba, revision, &commit_poke);
                                },
                                div { class: "thumb", style: "background:{hex};" }
                                span { class: "tname", "{hex}" }
                            }
                        )
                    });
                    let key_session = session.clone();
                    let key_ids = color_ids.clone();
                    let key_poke = poke.clone();
                    let key_search = search_node;
                    rsx!(
                        div { class: "bwork",
                            onkeydown: move |evt: KeyboardEvent| {
                                if key_session.field().is_some() || crate::ui::keymap::is_typing() {
                                    return;
                                }
                                let result = {
                                    let mut selection = key_session.browser_selection.lock().unwrap();
                                    browser_key(
                                        &mut selection,
                                        BrowserScope::Colors,
                                        &key_ids,
                                        &evt.key(),
                                        evt.code(),
                                        evt.modifiers(),
                                        false,
                                    )
                                };
                                match consume_browser_key(
                                    &evt,
                                    result,
                                    focusable_items,
                                    &key_poke,
                                ) {
                                    Some(BrowserKeyAction::Commit) => {
                                        let active = key_session
                                            .browser_selection
                                            .lock()
                                            .unwrap()
                                            .active_in(BrowserScope::Colors, &key_ids);
                                        if let Some(BrowserItemId::Color(rgba)) = active {
                                            commit_color(&key_session, rgba, revision, &key_poke);
                                        }
                                    }
                                    Some(BrowserKeyAction::Spatial(direction, extend)) => {
                                        move_browser_spatial(
                                            focusable_items,
                                            key_session.clone(),
                                            BrowserScope::Colors,
                                            key_ids.clone(),
                                            direction,
                                            extend,
                                            key_poke.clone(),
                                        );
                                    }
                                    Some(BrowserKeyAction::FocusSearch) => {
                                        focus_and_select_search(key_search.read().clone());
                                    }
                                    Some(BrowserKeyAction::TypeAhead(query)) => {
                                        focus_search_with_text(key_search.read().clone(), &query);
                                    }
                                    Some(BrowserKeyAction::Delete | BrowserKeyAction::Unavailable) => {
                                        browser_action_unavailable(&key_session, &key_poke);
                                    }
                                    _ => {}
                                }
                            },
                            div {
                                class: "bside",
                                onkeydown: move |evt: KeyboardEvent| consume_fixed_rail_key(&evt),
                                h3 { class: "sh", "Colors" }
                                div { class: "srow on", tabindex: "0", if used_count == 0 { "Starter palette" } else { "Used here · then starter" } }
                            }
                            div { class: "bresults",
                                div { class: "rhead",
                                    div {
                                        h2 { "Colors" }
                                        span { class: "sub",
                                            if layer.is_some() { "Choose a color to apply it" } else { "Select a layer first" }
                                        }
                                    }
                                }
                                match wheel_slot(session) {
                                    Some(slot) => rsx!(ColorWheel { session: session.clone(), slot, revision, wake: layout_tick }),
                                    None => rsx!(div { class: "rcount", "No color yet · select a layer to edit one" }),
                                }
                                if has_swatches {
                                    div { class: "{color_grid_class}", role: "grid", {cards} }
                                } else {
                                    div { class: "rcount", "No colors yet · select a layer to apply one" }
                                }
                            }
                        }
                    )
                }
            } else if panel == Panel::Effects {
                {
                    let layer = selected();
                    let attached: Vec<String> = layer
                        .and_then(|l| doc.lock().unwrap().view().effects(l).ok())
                        .unwrap_or_default()
                        .into_iter()
                        .map(|e| e.plugin_id)
                        .collect();
                    let catalog = crate::render::engine::known_effects();
                    let all_effect_ids: Vec<_> = catalog
                        .iter()
                        .map(|desc| BrowserItemId::Effect(desc.plugin_id.clone()))
                        .collect();
                    {
                        let mut selection = session.browser_selection.lock().unwrap();
                        selection.reconcile(BrowserScope::Effects, &all_effect_ids);
                    }
                    let shown_effects: Vec<_> = catalog
                        .iter()
                        .filter(|desc| query_matches(&desc.plugin_id, &query))
                        .collect();
                    let effect_ids: Vec<_> = shown_effects
                        .iter()
                        .map(|desc| BrowserItemId::Effect(desc.plugin_id.clone()))
                        .collect();
                    let effect_selection = {
                        let mut selection = session.browser_selection.lock().unwrap();
                        selection.ensure_active_in(BrowserScope::Effects, &effect_ids);
                        selection.clone()
                    };
                    let effect_active = effect_selection.active_in(BrowserScope::Effects, &effect_ids);
                    let effect_count = shown_effects.len();
                    let cards = shown_effects.iter().enumerate().map(|(index, desc)| {
                        let plugin_id = desc.plugin_id.to_owned();
                        let is_on = attached.contains(&plugin_id);
                        let item_id = BrowserItemId::Effect(plugin_id.clone());
                        let is_selected = effect_selection.is_selected(BrowserScope::Effects, &item_id);
                        let card_class = if is_selected { "tcard on" } else { "tcard" };
                        let focus_key = item_id.focus_key();
                        let select_session = session.clone();
                        let select_item = item_id.clone();
                        let select_ids = effect_ids.clone();
                        let select_poke = poke.clone();
                        let commit_session = session.clone();
                        let commit_item = item_id.clone();
                        let commit_ids = effect_ids.clone();
                        let commit_poke = poke.clone();
                        rsx!(
                            SemanticButton {
                                class: "{card_class}",
                                role: "gridcell",
                                aria_selected: if is_selected { "true" } else { "false" },
                                aria_posinset: Some((index + 1).to_string()),
                                aria_setsize: Some(effect_count.to_string()),
                                focus_key: Some(focus_key),
                                tabindex: Some(result_tabindex(effect_active.as_ref(), &item_id)),
                                title: if layer.is_none() { "Select · choose a layer before attaching" } else { "Select · double-click or Enter to attach" },
                                selected: is_selected,
                                onclick: move |evt: MouseEvent| choose_browser_item(
                                    &select_session,
                                    BrowserScope::Effects,
                                    &select_item,
                                    &select_ids,
                                    evt.modifiers(),
                                    true,
                                    &select_poke,
                                ),
                                ondoubleclick: move |evt: MouseEvent| {
                                    choose_browser_item(
                                        &commit_session,
                                        BrowserScope::Effects,
                                        &commit_item,
                                        &commit_ids,
                                        evt.modifiers(),
                                        true,
                                        &commit_poke,
                                    );
                                    commit_effect_selection(
                                        &commit_session,
                                        &commit_ids,
                                        revision,
                                        &commit_poke,
                                    );
                                },
                                div { class: "thumb", style: "background:#222; display:flex; align-items:center; justify-content:center;",
                                    span { style: "color:#fff; font-size:20px;", "ƒ" }
                                }
                                span { class: "tname", "{plugin_id}" }
                                span { class: "tmeta", if is_on { "Attached" } else { "Effect" } }
                            }
                        )
                    });
                    let key_session = session.clone();
                    let key_ids = effect_ids.clone();
                    let key_poke = poke.clone();
                    let key_search = search_node;
                    let marquee_down_session = session.clone();
                    rsx!(
                        div { class: "bwork",
                            onkeydown: move |evt: KeyboardEvent| {
                                if key_session.field().is_some() || crate::ui::keymap::is_typing() {
                                    return;
                                }
                                let result = {
                                    let mut selection = key_session.browser_selection.lock().unwrap();
                                    browser_key(
                                        &mut selection,
                                        BrowserScope::Effects,
                                        &key_ids,
                                        &evt.key(),
                                        evt.code(),
                                        evt.modifiers(),
                                        true,
                                    )
                                };
                                match consume_browser_key(
                                    &evt,
                                    result,
                                    focusable_items,
                                    &key_poke,
                                ) {
                                    Some(BrowserKeyAction::Commit) => {
                                        commit_effect_selection(
                                            &key_session,
                                            &key_ids,
                                            revision,
                                            &key_poke,
                                        );
                                    }
                                    Some(BrowserKeyAction::Spatial(direction, extend)) => {
                                        move_browser_spatial(
                                            focusable_items,
                                            key_session.clone(),
                                            BrowserScope::Effects,
                                            key_ids.clone(),
                                            direction,
                                            extend,
                                            key_poke.clone(),
                                        );
                                    }
                                    Some(BrowserKeyAction::FocusSearch) => {
                                        focus_and_select_search(key_search.read().clone());
                                    }
                                    Some(BrowserKeyAction::TypeAhead(query)) => {
                                        focus_search_with_text(key_search.read().clone(), &query);
                                    }
                                    Some(BrowserKeyAction::Delete | BrowserKeyAction::Unavailable) => {
                                        browser_action_unavailable(&key_session, &key_poke);
                                    }
                                    _ => {}
                                }
                            },
                            div {
                                class: "bside",
                                onkeydown: move |evt: KeyboardEvent| consume_fixed_rail_key(&evt),
                                h3 { class: "sh", "Effects" }
                                div { class: "srow on", tabindex: "0", "All" }
                            }
                            div { class: "bresults",
                                div { class: "rhead",
                                    div {
                                        h2 { "Effects" }
                                        span { class: "sub",
                                            if layer.is_some() { "Choose an effect to add it" } else { "Select a layer first" }
                                        }
                                    }
                                }
                                div {
                                    class: "{grid_class}",
                                    role: "grid",
                                    onpointerdown: move |evt: PointerEvent| begin_browser_marquee(
                                        &marquee_down_session,
                                        BrowserScope::Effects,
                                        &evt,
                                        marquee,
                                    ),
                                    {cards}
                                    {marquee_overlay(
                                        session,
                                        marquee(),
                                        BrowserScope::Effects,
                                        &effect_ids,
                                        focusable_items,
                                        marquee,
                                        &poke,
                                    )}
                                }
                            }
                        }
                    )
                }
            } else if panel == Panel::Create {
                {
                    let mask_layer = selected().filter(|layer| {
                        mask_frame(&doc.lock().unwrap(), *layer).is_some()
                    });
                    let mask_count = mask_layer
                        .and_then(|layer| doc.lock().unwrap().view().masks(layer).ok())
                        .map(|masks| masks.len())
                        .unwrap_or(0);
                    let all_create = [
                        (CreateItem::Text, "Text", "Adds a text layer", "T"),
                        (CreateItem::Rectangle, "Rectangle", "Adds a shape layer", "■"),
                        (CreateItem::Bezier, "Bezier", "Adds a path layer", "〜"),
                        (CreateItem::Mask, "Mask", "layer mask", "□"),
                    ];
                    let all_create_ids: Vec<_> = all_create
                        .iter()
                        .map(|(item, _, _, _)| BrowserItemId::Create(*item))
                        .collect();
                    {
                        let mut selection = session.browser_selection.lock().unwrap();
                        selection.reconcile(BrowserScope::Create, &all_create_ids);
                    }
                    let shown_create: Vec<_> = all_create
                        .into_iter()
                        .filter(|(_, label, _, _)| query_matches(label, &query))
                        .collect();
                    let create_ids: Vec<_> = shown_create
                        .iter()
                        .map(|(item, _, _, _)| BrowserItemId::Create(*item))
                        .collect();
                    {
                        session
                            .browser_selection
                            .lock()
                            .unwrap()
                            .ensure_active_in(BrowserScope::Create, &create_ids);
                    }
                    let create_selection = session.browser_selection.lock().unwrap().clone();
                    let create_active = create_selection.active_in(BrowserScope::Create, &create_ids);
                    let create_count = shown_create.len();
                    let cards = shown_create.into_iter().enumerate().map(|(index, (item, label, meta, glyph))| {
                        let item_id = BrowserItemId::Create(item);
                        let is_selected = create_selection.is_selected(BrowserScope::Create, &item_id);
                        let card_class = if is_selected { "tcard on" } else { "tcard" };
                        let focus_key = item_id.focus_key();
                        let select_session = session.clone();
                        let select_item = item_id.clone();
                        let select_ids = create_ids.clone();
                        let select_poke = poke.clone();
                        let commit_session = session.clone();
                        let commit_item = item_id.clone();
                        let commit_ids = create_ids.clone();
                        let commit_poke = poke.clone();
                        let commit_timeline = timeline_tx.clone();
                        let detail = if item == CreateItem::Mask && mask_count > 0 {
                            format!("{mask_count} attached")
                        } else {
                            meta.to_owned()
                        };
                        rsx!(SemanticButton {
                            class: "{card_class}",
                            role: "gridcell",
                            aria_selected: if is_selected { "true" } else { "false" },
                            aria_posinset: Some((index + 1).to_string()),
                            aria_setsize: Some(create_count.to_string()),
                            focus_key: Some(focus_key),
                            tabindex: Some(result_tabindex(create_active.as_ref(), &item_id)),
                            selected: is_selected,
                            title: "Select · double-click or Enter to create",
                            onclick: move |evt: MouseEvent| choose_browser_item(
                                &select_session,
                                BrowserScope::Create,
                                &select_item,
                                &select_ids,
                                evt.modifiers(),
                                false,
                                &select_poke,
                            ),
                            ondoubleclick: move |evt: MouseEvent| {
                                choose_browser_item(
                                    &commit_session,
                                    BrowserScope::Create,
                                    &commit_item,
                                    &commit_ids,
                                    evt.modifiers(),
                                    false,
                                    &commit_poke,
                                );
                                commit_create_item(
                                    &commit_session,
                                    item,
                                    layer_rows,
                                    attrs_state,
                                    &commit_timeline,
                                    revision,
                                    &commit_poke,
                                );
                            },
                            div { class: "thumb glyphy", span { "{glyph}" } }
                            span { class: "tname", "{label}" }
                            span { class: "tmeta", "{detail}" }
                        })
                    });
                    let key_session = session.clone();
                    let key_ids = create_ids.clone();
                    let key_timeline = timeline_tx.clone();
                    let key_poke = poke.clone();
                    let key_search = search_node;
                    rsx!(div { class: "bwork",
                        onkeydown: move |evt: KeyboardEvent| {
                            if key_session.field().is_some() || crate::ui::keymap::is_typing() {
                                return;
                            }
                            let result = {
                                let mut selection = key_session.browser_selection.lock().unwrap();
                                browser_key(
                                    &mut selection,
                                    BrowserScope::Create,
                                    &key_ids,
                                    &evt.key(),
                                    evt.code(),
                                    evt.modifiers(),
                                    false,
                                )
                            };
                            match consume_browser_key(
                                &evt,
                                result,
                                focusable_items,
                                &key_poke,
                            ) {
                                Some(BrowserKeyAction::Commit) => {
                                    let active = key_session
                                        .browser_selection
                                        .lock()
                                        .unwrap()
                                        .active_in(BrowserScope::Create, &key_ids);
                                    if let Some(BrowserItemId::Create(item)) = active {
                                        commit_create_item(
                                            &key_session,
                                            item,
                                            layer_rows,
                                            attrs_state,
                                            &key_timeline,
                                            revision,
                                            &key_poke,
                                        );
                                    }
                                }
                                Some(BrowserKeyAction::Spatial(direction, extend)) => {
                                    move_browser_spatial(
                                        focusable_items,
                                        key_session.clone(),
                                        BrowserScope::Create,
                                        key_ids.clone(),
                                        direction,
                                        extend,
                                        key_poke.clone(),
                                    );
                                }
                                Some(BrowserKeyAction::FocusSearch) => {
                                    focus_and_select_search(key_search.read().clone());
                                }
                                Some(BrowserKeyAction::TypeAhead(query)) => {
                                    focus_search_with_text(key_search.read().clone(), &query);
                                }
                                Some(BrowserKeyAction::Delete | BrowserKeyAction::Unavailable) => {
                                    browser_action_unavailable(&key_session, &key_poke);
                                }
                                _ => {}
                            }
                        },
                        div {
                            class: "bside",
                            onkeydown: move |evt: KeyboardEvent| consume_fixed_rail_key(&evt),
                            h3 { class: "sh", "Create" }
                            div { class: "srow on", tabindex: "0", "All" }
                        }
                        div { class: "bresults",
                            div { class: "rhead",
                                div {
                                    h2 { "Create" }
                                    span { class: "sub", "Select a card, then press Enter or double-click" }
                                }
                            }
                            div { class: "{grid_class}", role: "grid", {cards} }
                        }
                    })
                }
            } else {
                {
                let key_session = session.clone();
                let key_ids = media_ids.clone();
                let key_all = all_media_ids.clone();
                let key_commands = media_commands.clone();
                let key_used = used_asset_ids.clone();
                let key_timeline = timeline_tx.clone();
                let key_poke = poke.clone();
                let key_search = search_node;
                let marquee_down_session = session.clone();
                let rail_options: Vec<_> = std::iter::once(None)
                    .chain(families.iter().copied().map(Some))
                    .collect();
                let rail_keys: Vec<_> = rail_options.iter().copied().map(media_rail_key).collect();
                let rail_key_options = rail_options.clone();
                let rail_key_names = rail_keys.clone();
                let mut key_rail = rail;
                rsx!(div { class: "bwork",
                    onkeydown: move |evt: KeyboardEvent| {
                        if key_session.field().is_some() || crate::ui::keymap::is_typing() {
                            return;
                        }
                        let result = {
                            let mut selection = key_session.browser_selection.lock().unwrap();
                            browser_key(
                                &mut selection,
                                BrowserScope::Media,
                                &key_ids,
                                &evt.key(),
                                evt.code(),
                                evt.modifiers(),
                                true,
                            )
                        };
                        match consume_browser_key(
                            &evt,
                            result,
                            focusable_items,
                            &key_poke,
                        ) {
                            Some(BrowserKeyAction::Commit) => commit_active_media(
                                &key_session,
                                &key_ids,
                                &key_commands,
                                layer_rows,
                                attrs_state,
                                &key_timeline,
                                revision,
                                &key_poke,
                            ),
                            Some(BrowserKeyAction::Delete) => delete_selected_media(
                                &key_session,
                                &key_ids,
                                &key_all,
                                &key_used,
                                revision,
                                &key_poke,
                            ),
                            Some(BrowserKeyAction::Spatial(direction, extend)) => {
                                move_browser_spatial(
                                    focusable_items,
                                    key_session.clone(),
                                    BrowserScope::Media,
                                    key_ids.clone(),
                                    direction,
                                    extend,
                                    key_poke.clone(),
                                );
                            }
                            Some(BrowserKeyAction::FocusSearch) => {
                                focus_and_select_search(key_search.read().clone());
                            }
                            Some(BrowserKeyAction::TypeAhead(query)) => {
                                focus_search_with_text(key_search.read().clone(), &query);
                            }
                            Some(BrowserKeyAction::Unavailable) => {
                                browser_action_unavailable(&key_session, &key_poke);
                            }
                            _ => {}
                        }
                    },
                    div {
                        class: "bside",
                        onkeydown: move |evt: KeyboardEvent| {
                            let current = rail_key_options
                                .iter()
                                .position(|family| *family == key_rail())
                                .unwrap_or(0);
                            let next = match evt.key() {
                                Key::ArrowLeft | Key::ArrowUp => current.saturating_sub(1),
                                Key::ArrowRight | Key::ArrowDown => {
                                    (current + 1).min(rail_key_options.len().saturating_sub(1))
                                }
                                Key::Home => 0,
                                Key::End => rail_key_options.len().saturating_sub(1),
                                Key::Enter => current,
                                Key::Character(ref c) if c == " " => current,
                                _ => return,
                            };
                            evt.prevent_default();
                            evt.stop_propagation();
                            key_rail.set(rail_key_options[next]);
                            focusable_items.focus(&rail_key_names[next]);
                        },
                        h3 { class: "sh", "Media" }
                        SemanticButton {
                            class: "{rail_class(None)}",
                            focus_key: Some(media_rail_key(None)),
                            tabindex: Some(if rail().is_none() { "0".to_owned() } else { "-1".to_owned() }),
                            selected: rail().is_none(),
                            onclick: move |_| rail.set(None),
                            "All media"
                        }
                        {families.iter().copied().map(|f| rsx!(
                            SemanticButton {
                                class: "{rail_class(Some(f))}",
                                focus_key: Some(media_rail_key(Some(f))),
                                tabindex: Some(if rail() == Some(f) { "0".to_owned() } else { "-1".to_owned() }),
                                selected: rail() == Some(f),
                                onclick: move |_| rail.set(Some(f)),
                                "{f.label()}"
                            }
                        ))}
                    }
                    div { class: "bresults",
                        div { class: "rhead",
                            div {
                                h2 { "{rail_label}" }
                                span { class: "sub", "Media" }
                            }
                        }
                        div { class: "rcount",
                            "Results"
                            em { "{asset_count}" }
                        }
                        if filtered_out {
                            div { class: "rcount",
                                "No {rail_label} in this project yet · "
                                SemanticButton { class: "chip", onclick: move |_| rail.set(None), "Show all media" }
                            }
                        } else {
                            div {
                                class: "{grid_class}",
                                role: "grid",
                                onpointerdown: move |evt: PointerEvent| begin_browser_marquee(
                                    &marquee_down_session,
                                    BrowserScope::Media,
                                    &evt,
                                    marquee,
                                ),
                                {asset_cards}
                                {marquee_overlay(
                                    session,
                                    marquee(),
                                    BrowserScope::Media,
                                    &media_ids,
                                    focusable_items,
                                    marquee,
                                    &poke,
                                )}
                            }
                        }
                        div { class: "bfoot",
                            span { class: "dot", style: "background:var(--accent);" }
                            "{library_line}"
                        }
                    }
                })
                }
            }
        }
    )
}

#[cfg(test)]
mod placement {
    use super::*;

    fn effect_ids(names: &[&str]) -> Vec<BrowserItemId> {
        names
            .iter()
            .map(|name| BrowserItemId::Effect((*name).to_owned()))
            .collect()
    }

    #[test]
    fn first_printable_key_enters_search_and_filters_the_first_result() {
        let ids = effect_ids(&["Blur", "Glow"]);
        let mut selection = BrowserSelection::default();
        selection.ensure_active_in(BrowserScope::Effects, &ids);
        let result = browser_key(
            &mut selection,
            BrowserScope::Effects,
            &ids,
            &Key::Character("g".into()),
            Code::KeyG,
            Modifiers::empty(),
            true,
        );
        assert_eq!(result.action, BrowserKeyAction::TypeAhead("g".into()));
        assert_eq!(selection.query(BrowserScope::Effects), "g");
        let shown: Vec<_> = ["Blur", "Glow"]
            .into_iter()
            .filter(|name| query_matches(name, selection.query(BrowserScope::Effects)))
            .collect();
        assert_eq!(shown, vec!["Glow"]);
    }

    #[test]
    fn one_visible_result_is_in_the_tab_order() {
        let ids = effect_ids(&["a", "b", "c"]);
        let mut selection = BrowserSelection::default();
        selection.ensure_active_in(BrowserScope::Effects, &ids);
        let active = selection.active_in(BrowserScope::Effects, &ids);
        assert_eq!(
            ids.iter()
                .filter(|id| result_tabindex(active.as_ref(), id) == "0")
                .count(),
            1
        );
    }

    #[test]
    fn select_mode_pointer_taps_build_and_toggle_a_multi_selection() {
        let ids = effect_ids(&["a", "b", "c"]);
        let mut selection = BrowserSelection::default();
        selection.toggle_select_mode(BrowserScope::Effects);
        selection.click(BrowserScope::Effects, &ids[0], &ids, true, false);
        selection.click(BrowserScope::Effects, &ids[1], &ids, true, false);
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[0].clone(), ids[1].clone()]
        );
        selection.click(BrowserScope::Effects, &ids[0], &ids, true, false);
        assert_eq!(
            selection.selected_in_order(BrowserScope::Effects, &ids),
            vec![ids[1].clone()]
        );
    }

    #[test]
    fn effect_batch_is_atomic_and_does_not_duplicate_plugin_ids() {
        let layer = LayerId(1);
        let mut doc = Document::new();
        doc.apply(Intent::AddLayer(layer)).unwrap();
        let chosen = vec!["blur".to_owned(), "blur".to_owned(), "glow".to_owned()];
        let intents = effect_batch_intents(&doc, layer, &chosen).unwrap();
        doc.apply_all(intents).unwrap();
        assert!(effect_batch_intents(&doc, layer, &chosen)
            .unwrap()
            .is_empty());
        assert_eq!(
            doc.view()
                .effects(layer)
                .unwrap()
                .into_iter()
                .map(|effect| effect.plugin_id)
                .collect::<Vec<_>>(),
            vec!["blur", "glow"]
        );
    }

    /// 位置を指定せずに生まれた層は、**枠の真ん中に立つ**。
    ///
    /// 隅(0,0)に置くと、素材が小さいほど画面の角の点になって見つからない。
    /// 説明書も最初から「画面の真ん中に立つ」と書いている。
    fn box_of(kind: NewKind, comp: (f64, f64), natural: (f64, f64)) -> (f64, f64) {
        let layer = LayerId(1);
        let intents = new_layer_intents(
            layer,
            0,
            0,
            30,
            crate::doc::store::Fps::try_new(30, 1).unwrap(),
            comp,
            kind,
        );
        let mut doc = Document::new();
        doc.apply_all(intents).unwrap();
        let view = doc.view();
        let property =
            crate::doc::store::PropertyId::new(crate::doc::store::property::POSITION).unwrap();
        let position = view
            .value_at(layer, &property, crate::doc::store::RationalTime::ZERO)
            .unwrap()
            .and_then(|v| match v {
                crate::doc::store::Value::Vec2([x, y]) => Some((x, y)),
                _ => None,
            })
            .unwrap_or((0.0, 0.0));
        (position.0 + natural.0 * 0.5, position.1 + natural.1 * 0.5)
    }

    #[test]
    fn a_rectangle_is_born_in_the_middle_of_the_frame() {
        let comp = (640.0, 480.0);
        let side = rect_side(comp);
        let center = box_of(NewKind::Rectangle, comp, (side, side));
        assert!(
            (center.0 - 320.0).abs() < 2.0 && (center.1 - 240.0).abs() < 2.0,
            "四角の中心が枠の真ん中に無い: {center:?}"
        );
    }

    #[test]
    fn an_image_is_born_in_the_middle_of_the_frame() {
        let dir = std::env::temp_dir().join("motolii-placement");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("logo.png");
        image::RgbaImage::from_pixel(64, 48, image::Rgba([255, 0, 0, 255]))
            .save(&path)
            .unwrap();
        let comp = (640.0, 480.0);
        let center = box_of(
            NewKind::Media {
                path: path.to_str().unwrap().to_owned(),
                name: "logo".into(),
            },
            comp,
            (64.0, 48.0),
        );
        assert!(
            (center.0 - 320.0).abs() < 2.0 && (center.1 - 240.0).abs() < 2.0,
            "絵の中心が枠の真ん中に無い: {center:?}"
        );
    }

    #[test]
    fn mesh_and_point_cloud_receive_the_same_spatial_fit_contract() {
        let dir = tempfile::tempdir().unwrap();
        let obj = dir.path().join("triangle.obj");
        let ply = dir.path().join("triangle.ply");
        std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        std::fs::write(
            &ply,
            "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nend_header\n-1 -1 0\n1 -1 0\n0 1 0\n",
        )
        .unwrap();

        let values = |path: &std::path::Path| {
            let layer = LayerId(1);
            let mut doc = Document::new();
            doc.apply_all(spatial_fit_intents(
                layer,
                path.to_str().unwrap(),
                (640.0, 480.0),
            ))
            .unwrap();
            let view = doc.view();
            let read = |name| {
                view.value_at(layer, &PropertyId::new(name).unwrap(), RationalTime::ZERO)
                    .unwrap()
                    .unwrap()
            };
            (read(property::POSITION), read(property::SCALE))
        };

        let mesh = values(&obj);
        let points = values(&ply);
        assert_eq!(mesh, points);
        let (Value::Vec2(position), Value::Vec2(scale)) = mesh else {
            panic!("spatial fit did not write Position and Scale")
        };
        assert!((position[0] + scale[0] - 320.0).abs() < 0.01);
        assert!((position[1] + scale[1] - 240.0).abs() < 0.01);
    }

    #[test]
    fn text_content_starts_on_the_composition_frame() {
        let fps = crate::doc::store::Fps::try_new(24, 1).unwrap();
        let layer = LayerId(1);
        let mut doc = Document::new();
        doc.apply_all(new_layer_intents(
            layer,
            0,
            37,
            240,
            fps,
            (640.0, 480.0),
            NewKind::Text,
        ))
        .unwrap();

        let text = doc.view().text_document(layer).unwrap().unwrap();
        assert_eq!(
            text.content.keys()[0].t.try_to_frame_round(fps).unwrap(),
            37
        );
    }
}
