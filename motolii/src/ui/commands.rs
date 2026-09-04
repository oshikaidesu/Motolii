use dioxus_native::prelude::*;

use crate::doc::store::{Intent as Edit, StoreError};
use crate::ui::app::{history_step, refresh_layer_projection, Panes};
use crate::ui::keymap::Intent;
use crate::ui::session::Session;
use crate::ui::timeline_widget::TimelineMsg;

pub(super) fn run(session: &Session, mut panes: Panes, intent: Intent) -> bool {
    let result: Result<(), StoreError> = match intent {
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
            if !crate::ui::inspector::cancel_scrub(session) && !session.gesture.cancel() {
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
        _ => return false,
    };
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
