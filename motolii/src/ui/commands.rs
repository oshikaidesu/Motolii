use dioxus_native::prelude::*;

use crate::doc::store::{Intent as Edit, StoreError};
use crate::ui::app::{history_step, refresh_layer_projection, Panes};
use crate::ui::keymap::Intent;
use crate::ui::session::Session;
use crate::ui::timeline_widget::TimelineMsg;

fn run_terminal_actions(scrub: impl FnOnce() -> bool, gesture: impl FnOnce() -> bool) -> bool {
    let scrub = scrub();
    let gesture = gesture();
    scrub || gesture
}

pub(super) fn cancel_interactions(session: &Session) -> bool {
    let cancelled = run_terminal_actions(
        || crate::ui::inspector::cancel_scrub(session),
        || session.gesture.cancel(),
    );
    session.surface_capture.cancel_all();
    if cancelled {
        session.doc.lock().unwrap().clear_all_transients();
    }
    cancelled
}

pub(super) fn preflight(session: &Session, intent: Intent) {
    if matches!(
        intent,
        Intent::Split
            | Intent::Duplicate
            | Intent::DeleteLayer
            | Intent::Undo
            | Intent::Redo
            | Intent::Rename
            | Intent::Reorder(_)
            | Intent::SnapEdgeToPlayhead(_)
            | Intent::TrimToPlayhead(_)
            | Intent::ToggleMarker
            | Intent::Group
            | Intent::Ungroup
            | Intent::EasyEase(_)
            | Intent::Nudge(_, _)
            | Intent::Save
            | Intent::SaveAs
            | Intent::NewProject
            | Intent::OpenProject
            | Intent::Quit
            | Intent::CompositionSettings
    ) {
        cancel_interactions(session);
    }
}

pub(super) fn run(session: &Session, mut panes: Panes, intent: Intent) -> bool {
    preflight(session, intent);
    if matches!(
        intent,
        Intent::Save
            | Intent::SaveAs
            | Intent::NewProject
            | Intent::OpenProject
            | Intent::Quit
            | Intent::CompositionSettings
    ) {
        return false;
    }
    let result: Result<(), StoreError> = (|| match intent {
        Intent::Split | Intent::Duplicate => {
            let targets = session.editable_selection();
            let result = if matches!(intent, Intent::Split) {
                crate::ui::timeline_edit::split_layers(
                    &session.doc,
                    &targets,
                    session.clock.current_frame(),
                )
            } else {
                crate::ui::timeline_edit::duplicate_layers(&session.doc, &targets)
            };
            result.map(|copies| {
                if copies.is_empty() {
                    *session.project_notice.lock().unwrap() =
                        "No editable layers at this time".to_owned();
                } else {
                    let count = copies.len();
                    session.selection.replace(copies);
                    panes.selected.set(session.selection.get());
                    let verb = if matches!(intent, Intent::Split) {
                        "Split"
                    } else {
                        "Duplicated"
                    };
                    *session.project_notice.lock().unwrap() =
                        format!("{verb} {count} layers · ⌘Z to undo");
                    refresh(session, panes);
                }
            })
        }
        Intent::DeleteLayer => delete(session, panes),
        Intent::Rename => {
            if let Some(layer) = session
                .selection
                .get()
                .filter(|&layer| session.writable(layer))
            {
                let name = session
                    .doc
                    .lock()
                    .unwrap()
                    .view()
                    .attrs(layer)
                    .ok()
                    .flatten()
                    .map(|a| a.name)
                    .unwrap_or_default();
                session.open_field(crate::ui::session::FieldAt::Name(layer), name);
            }
            Ok(())
        }
        Intent::Undo | Intent::Redo => {
            history_step(
                session,
                panes.layer_rows,
                panes.attrs_state,
                &session.timeline_tx,
                panes.revision,
                if matches!(intent, Intent::Undo) {
                    -1
                } else {
                    1
                },
            );
            panes.selected.set(session.selection.get());
            Ok(())
        }
        Intent::Deselect => {
            if !cancel_interactions(session) {
                session.selection.clear();
                panes.selected.set(None);
                *session.focus.lock().unwrap() = None;
                session.selected_keys.lock().unwrap().clear();
                let _ = session.timeline_tx.send(TimelineMsg::DeselectKeys);
            }
            Ok(())
        }
        Intent::SelectAll => {
            let layers = session.doc.lock().unwrap().view().layers();
            session.selection.replace(layers);
            panes.selected.set(session.selection.get());
            Ok(())
        }
        Intent::PlayPause => {
            session.clock.toggle();
            Ok(())
        }
        Intent::StepFrame(delta) => {
            session
                .clock
                .seek_frame(session.clock.current_frame() + delta);
            Ok(())
        }
        Intent::Home => {
            session.clock.seek(0.0);
            Ok(())
        }
        Intent::End => {
            session.clock.seek(session.clock.duration());
            Ok(())
        }
        Intent::Group => {
            let targets = session.selection.all();
            let grouped = session.doc.lock().unwrap().group_layers(&targets)?;
            match grouped {
                Some(group) => {
                    crate::ui::fixture::expand(group);
                    session.selection.set(Some(group));
                    panes.selected.set(Some(group));
                    *session.project_notice.lock().unwrap() = "Grouped".to_owned();
                    refresh(session, panes);
                }
                None => println!("PROBE room=write verdict=group-noop reason=no-selection"),
            }
            Ok(())
        }
        Intent::Ungroup => {
            let targets = session.selection.all();
            let released = session.doc.lock().unwrap().ungroup_layers(&targets)?;
            if released.is_empty() {
                println!("PROBE room=write verdict=ungroup-noop reason=no-group-selection");
            } else {
                session.selection.replace(released);
                panes.selected.set(session.selection.get());
                *session.project_notice.lock().unwrap() = "Ungrouped".to_owned();
                refresh(session, panes);
            }
            Ok(())
        }
        Intent::EasyEase(side) => {
            let starts = crate::ui::ease::segments(&session.selected_keys.lock().unwrap());
            if starts.is_empty() {
                *session.project_notice.lock().unwrap() = "Select keyframes first".to_owned();
            } else {
                let count = crate::ui::ease::apply_easy(session, &starts, side)
                    .map_err(StoreError::Property)?;
                println!("PROBE room=write verdict=applied EasyEase tracks={count}");
            }
            Ok(())
        }
        Intent::Reveal(property) => {
            crate::ui::fixture::toggle_reveal(property);
            if crate::ui::fixture::reveal().is_some() {
                if let Some(layer) = session.selection.get() {
                    crate::ui::fixture::expand(layer);
                }
            }
            refresh(session, panes);
            Ok(())
        }
        Intent::ToggleKeyedOnly => {
            crate::ui::fixture::toggle_keyed_only();
            if crate::ui::fixture::keyed_only() {
                if let Some(layer) = session.selection.get() {
                    crate::ui::fixture::expand(layer);
                }
            }
            refresh(session, panes);
            Ok(())
        }
        Intent::ToggleMarker => {
            let time = session.clock.current_time();
            let sec = time.as_seconds_f64();
            let mut doc = session.doc.lock().unwrap();
            let mut markers = doc.view().markers().unwrap_or_default();
            let hit = markers.iter().position(|marker| {
                (marker.time.as_seconds_f64() - sec).abs()
                    < 0.5 * session.clock.frame_duration_sec()
            });
            match hit {
                Some(index) if !session.clock.playing() => {
                    markers.remove(index);
                }
                Some(_) => return Ok(()),
                None => {
                    markers.push(crate::doc::store::Marker {
                        name: session.clock.format_timecode(),
                        time,
                        duration: crate::doc::store::RationalTime::ZERO,
                        body: String::new(),
                    });
                    markers.sort_by(|a, b| {
                        a.time.as_seconds_f64().total_cmp(&b.time.as_seconds_f64())
                    });
                }
            }
            doc.apply(Edit::SetMarkers {
                markers: markers.clone(),
            })?;
            drop(doc);
            let seconds = markers
                .iter()
                .map(|marker| marker.time.as_seconds_f64())
                .collect();
            let _ = session.timeline_tx.send(TimelineMsg::SetMarkers(seconds));
            if hit.is_none()
                && *session.desk.lock().unwrap() == crate::ui::session::DeskState::Follow
            {
                *session.desk.lock().unwrap() =
                    crate::ui::session::DeskState::Open(crate::ui::desk::Drawer::Text);
            }
            Ok(())
        }
        Intent::JumpMarker(direction) => {
            let sec = session.clock.now_sec();
            let mut times: Vec<_> = session
                .doc
                .lock()
                .unwrap()
                .view()
                .markers()
                .unwrap_or_default()
                .iter()
                .map(|marker| marker.time.as_seconds_f64())
                .collect();
            times.sort_by(f64::total_cmp);
            let next = if direction < 0 {
                times.into_iter().rev().find(|time| *time < sec - 1e-6)
            } else {
                times.into_iter().find(|time| *time > sec + 1e-6)
            };
            if let Some(time) = next {
                session.clock.seek(time);
            }
            Ok(())
        }
        Intent::Reorder(delta) => {
            let Some(layer) = session.selection.get() else {
                return Ok(());
            };
            let mut doc = session.doc.lock().unwrap();
            let Some(current) = doc.view().meta(layer)?.map(|meta| meta.order) else {
                return Ok(());
            };
            doc.apply(Edit::SetOrder {
                layer,
                order: current.saturating_add(delta),
            })?;
            drop(doc);
            refresh(session, panes);
            Ok(())
        }
        Intent::SnapEdgeToPlayhead(tail) | Intent::TrimToPlayhead(tail) => {
            let trim = matches!(intent, Intent::TrimToPlayhead(_));
            let Some(layer) = session.selection.get() else {
                return Ok(());
            };
            let frame = session.clock.current_frame();
            let mut doc = session.doc.lock().unwrap();
            let Some(original) = doc.view().meta(layer)?.map(|meta| meta.timing) else {
                return Ok(());
            };
            let timing = crate::ui::timeline_widget::edge_to_frame(original, frame, tail, trim);
            doc.apply(Edit::SetTiming { layer, timing })?;
            drop(doc);
            refresh(session, panes);
            Ok(())
        }
        Intent::SelectStep(delta) | Intent::SelectExtend(delta) => {
            let doc = session.doc.lock().unwrap();
            let mut ordered = Vec::new();
            for layer in crate::ui::fixture::layer_rows_from_doc(&doc)
                .into_iter()
                .filter_map(|row| row.layer)
            {
                if !ordered.contains(&layer) {
                    ordered.push(layer);
                }
            }
            drop(doc);
            let extend = matches!(intent, Intent::SelectExtend(_));
            if let Some(layer) = session.selection.step(&ordered, delta, extend) {
                panes.selected.set(Some(layer));
                let _ = session.timeline_tx.send(TimelineMsg::RevealLayer(layer));
            }
            Ok(())
        }
        Intent::View(request) => {
            *session.view_request.lock().unwrap() = Some(request);
            Ok(())
        }
        Intent::Nudge(dx, dy) => {
            let keys: Vec<_> = session
                .selected_keys
                .lock()
                .unwrap()
                .iter()
                .filter(|key| session.writable(key.layer))
                .cloned()
                .collect();
            if !keys.is_empty() && dy == 0.0 {
                let mut doc = session.doc.lock().unwrap();
                let fps = crate::ui::timeline_widget::document_fps(&doc)?;
                let by = dx.signum() as i64 * if dx.abs() >= 10.0 { 10 } else { 1 };
                let mut edits = Vec::new();
                for key in &keys {
                    let frame = (key.at_sec * fps.as_f64()).round() as i64;
                    edits.extend(crate::ui::timeline_widget::keyframe_move_intents(
                        &doc,
                        key.layer,
                        key.property.as_ref(),
                        &[frame],
                        by,
                    )?);
                }
                doc.apply_all(edits)?;
                let rows = crate::ui::fixture::canvas_rows_from_doc(&doc);
                drop(doc);
                for key in session.selected_keys.lock().unwrap().iter_mut() {
                    key.at_sec += by as f64 / fps.as_f64();
                }
                let _ = session.timeline_tx.send(TimelineMsg::SetRows(rows));
            } else {
                let targets = session.editable_selection();
                if targets.is_empty() {
                    return Ok(());
                }
                let at = session.clock.current_time();
                let mut doc = session.doc.lock().unwrap();
                let edits = crate::ui::stage_widget::nudge_intents(&doc, &targets, (dx, dy), at)?;
                doc.apply_all(edits)?;
            }
            Ok(())
        }
        _ => unreachable!("host-dependent intent was filtered before semantic dispatch"),
    })();
    if let Err(error) = result {
        *session.project_notice.lock().unwrap() = format!("Edit failed: {error}");
    }
    *panes.revision.write() += 1;
    true
}

fn refresh(session: &Session, panes: Panes) {
    refresh_layer_projection(
        &session.doc,
        panes.layer_rows,
        panes.attrs_state,
        &session.timeline_tx,
        panes.revision,
    );
}

fn delete(session: &Session, mut panes: Panes) -> Result<(), StoreError> {
    let keys = session.selected_keys.lock().unwrap().clone();
    if !keys.is_empty() {
        let keys: Vec<_> = keys
            .into_iter()
            .filter(|k| session.writable(k.layer))
            .collect();
        let at = session.clock.current_time();
        let mut d = session.doc.lock().unwrap();
        let selected: Vec<_> = keys
            .iter()
            .map(|key| (key.layer, key.property.clone(), key.at_sec))
            .collect();
        let edits = crate::ui::timeline_edit::delete_key_selection_intents(&d, &selected, at)?;
        if edits.is_empty() {
            return Ok(());
        }
        d.apply_all(edits)?;
        drop(d);
        session.selected_keys.lock().unwrap().clear();
        let _ = session.timeline_tx.send(TimelineMsg::DeselectKeys);
        *session.project_notice.lock().unwrap() =
            format!("Deleted {} keyframes · ⌘Z to undo", keys.len());
    } else {
        let targets = session.editable_selection();
        if targets.is_empty() {
            return Ok(());
        }
        session
            .doc
            .lock()
            .unwrap()
            .apply_all(targets.iter().copied().map(Edit::RemoveLayer))?;
        session.forget_dead_layers();
        panes.selected.set(session.selection.get());
        *session.project_notice.lock().unwrap() =
            format!("Deleted {} layers · ⌘Z to undo", targets.len());
    }
    refresh(session, panes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_and_surface_terminals_are_both_evaluated() {
        let calls = std::cell::Cell::new(0);
        assert!(run_terminal_actions(
            || {
                calls.set(calls.get() + 1);
                true
            },
            || {
                calls.set(calls.get() + 1);
                true
            },
        ));
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn undo_preflight_cancels_an_active_surface_before_history_moves() {
        let loaded = crate::ui::fixture::load_fixture();
        let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
        session.gesture.begin();
        preflight(&session, Intent::Undo);
        assert!(!session.gesture.is_active());
    }
}
