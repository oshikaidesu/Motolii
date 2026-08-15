//! Timeline pointerを既存gestureへ渡し、releaseまでDocumentを触らない。

use crate::timeline_move_gesture::TimelineMoveGesture;
use crate::timeline_projection::TimelineProjection;
use crate::timeline_trim_gesture::{TimelineTrimEdge, TimelineTrimGesture};

use super::MotoliiApp;

pub(super) struct EguiKeyDrag {
    layer: motolii_doc::LayerId,
    key: motolii_doc::KeyframeId,
    old: motolii_core::RationalTime,
    initial_pointer: motolii_core::RationalTime,
}

impl MotoliiApp {
    pub(super) fn handle_timeline_pointer(
        &mut self,
        phase: crate::timeline_egui::TimelinePointerPhase,
        time: Option<motolii_core::RationalTime>,
        hit: crate::timeline_egui::EguiTimelineHit,
        live_projection: Option<&TimelineProjection>,
    ) {
        use crate::timeline_egui::{EguiTimelineHit, TimelinePointerPhase};
        use crate::timeline_intent_adapter::{enqueue_timeline_intent, TimelineIntent};

        match phase {
            TimelinePointerPhase::Down => {
                self.clear_timeline_gestures();
                let mapped = match hit {
                    EguiTimelineHit::Key { layer, .. }
                    | EguiTimelineHit::Body { layer }
                    | EguiTimelineHit::Left { layer }
                    | EguiTimelineHit::Right { layer } => TimelineIntent::Select(Some(layer)),
                    EguiTimelineHit::None => TimelineIntent::Select(None),
                };
                let _ = enqueue_timeline_intent(&mut self.document_queue, mapped);
                let Some(pointer_time) = time else {
                    return;
                };
                match hit {
                    EguiTimelineHit::Body { layer } => {
                        if let Some(initial_start) = find_clip_start(&self.current_document, layer)
                        {
                            self.timeline_move = Some(TimelineMoveGesture::begin(
                                layer,
                                pointer_time,
                                initial_start,
                                self.projection_generation,
                            ));
                        }
                    }
                    EguiTimelineHit::Left { layer } | EguiTimelineHit::Right { layer } => {
                        if let Some((initial_start, initial_end)) =
                            find_clip_interval(&self.current_document, layer)
                        {
                            let edge = match hit {
                                EguiTimelineHit::Left { .. } => TimelineTrimEdge::Left,
                                _ => TimelineTrimEdge::Right,
                            };
                            self.timeline_trim = Some(TimelineTrimGesture::begin(
                                layer,
                                edge,
                                pointer_time,
                                initial_start,
                                initial_end,
                                self.projection_generation,
                            ));
                        }
                    }
                    EguiTimelineHit::Key { layer, key } => {
                        let old = find_position_key_time(&self.current_document, layer, key)
                            .or_else(|| {
                                live_projection.and_then(|projection| {
                                    projection
                                        .keys()
                                        .iter()
                                        .find(|item| item.layer == layer && item.key == key)
                                        .map(|item| item.t)
                                })
                            });
                        if let Some(old) = old {
                            self.timeline_key_drag = Some(EguiKeyDrag {
                                layer,
                                key,
                                old,
                                initial_pointer: pointer_time,
                            });
                        }
                    }
                    EguiTimelineHit::None => {}
                }
            }
            TimelinePointerPhase::Drag => {
                let Some(pointer_time) = time else {
                    return;
                };
                if let Some(gesture) = self.timeline_move {
                    if let Ok(new_start) = gesture.preview(pointer_time) {
                        self.timeline_preview = live_projection.and_then(|projection| {
                            projection.preview_move(
                                gesture.layer(),
                                new_start,
                                self.current_document.composition.duration,
                            )
                        });
                    }
                }
            }
            TimelinePointerPhase::Up => {
                if let Some(gesture) = self.timeline_move.take() {
                    if let Some(pointer_time) = time {
                        if let Ok(Some(request)) = gesture.release(pointer_time) {
                            let _ = enqueue_timeline_intent(
                                &mut self.document_queue,
                                TimelineIntent::MoveClip {
                                    layer: request.layer,
                                    new_start: request.new_start,
                                },
                            );
                        }
                    }
                }
                if let Some(gesture) = self.timeline_trim.take() {
                    if let Some(pointer_time) = time {
                        if let Ok(Some(request)) = gesture.release(pointer_time) {
                            let _ = enqueue_timeline_intent(
                                &mut self.document_queue,
                                TimelineIntent::TrimClip(request),
                            );
                        }
                    }
                }
                if let Some(drag) = self.timeline_key_drag.take() {
                    if let Some(pointer_time) = time {
                        let new = pointer_time
                            .try_sub(drag.initial_pointer)
                            .ok()
                            .and_then(|delta| drag.old.try_add(delta).ok());
                        if let Some(new) = new.filter(|new| *new != drag.old) {
                            let _ = enqueue_timeline_intent(
                                &mut self.document_queue,
                                TimelineIntent::MovePositionKey {
                                    target: drag.layer,
                                    key: drag.key,
                                    old: drag.old,
                                    new,
                                },
                            );
                        }
                    }
                }
                self.timeline_preview = None;
            }
        }
    }

    fn clear_timeline_gestures(&mut self) {
        self.timeline_move = None;
        self.timeline_trim = None;
        self.timeline_key_drag = None;
        self.timeline_preview = None;
    }
}

fn find_clip_start(
    document: &motolii_doc::Document,
    target: motolii_doc::LayerId,
) -> Option<motolii_core::RationalTime> {
    fn find(
        items: &[motolii_doc::TrackItem],
        target: motolii_doc::LayerId,
    ) -> Option<motolii_core::RationalTime> {
        for item in items {
            match item {
                motolii_doc::TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some(clip.start);
                }
                motolii_doc::TrackItem::Group(group) => {
                    if let Some(start) = find(&group.children, target) {
                        return Some(start);
                    }
                }
                motolii_doc::TrackItem::Clip(_) => {}
            }
        }
        None
    }

    document
        .tracks
        .iter()
        .find_map(|track| find(&track.items, target))
}

fn find_clip_interval(
    document: &motolii_doc::Document,
    target: motolii_doc::LayerId,
) -> Option<(motolii_core::RationalTime, motolii_core::RationalTime)> {
    fn find(
        items: &[motolii_doc::TrackItem],
        target: motolii_doc::LayerId,
    ) -> Option<(motolii_core::RationalTime, motolii_core::RationalTime)> {
        for item in items {
            match item {
                motolii_doc::TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some((clip.start, clip.start.try_add(clip.duration).ok()?));
                }
                motolii_doc::TrackItem::Group(group) => {
                    if let Some(interval) = find(&group.children, target) {
                        return Some(interval);
                    }
                }
                motolii_doc::TrackItem::Clip(_) => {}
            }
        }
        None
    }

    document
        .tracks
        .iter()
        .find_map(|track| find(&track.items, target))
}

fn find_position_key_time(
    document: &motolii_doc::Document,
    target: motolii_doc::LayerId,
    key: motolii_doc::KeyframeId,
) -> Option<motolii_core::RationalTime> {
    fn find(
        items: &[motolii_doc::TrackItem],
        target: motolii_doc::LayerId,
        key: motolii_doc::KeyframeId,
    ) -> Option<motolii_core::RationalTime> {
        for item in items {
            match item {
                motolii_doc::TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return match &clip.envelope.transform.position {
                        motolii_doc::DocParam::Keyframes(track) => track
                            .keys()
                            .iter()
                            .find(|item| item.id == key)
                            .map(|item| item.t),
                        _ => None,
                    };
                }
                motolii_doc::TrackItem::Group(group) => {
                    if let Some(time) = find(&group.children, target, key) {
                        return Some(time);
                    }
                }
                motolii_doc::TrackItem::Clip(_) => {}
            }
        }
        None
    }

    document
        .tracks
        .iter()
        .find_map(|track| find(&track.items, target, key))
}
