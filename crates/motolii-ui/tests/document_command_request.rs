//! U2a-1: prepared D2 command付きDocument intentをone-shot requestへ畳む。

use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_doc::{
    layer_names_for_item, Clip, ClipSource, Command, CommandKind, DocParam, Document,
    DocumentWriter, ItemEnvelope, LayerId, ParentLocator, ScalarPropertyId, Track, TrackId,
    TrackItem,
};
use motolii_plugin::PluginCatalogBuilder;
use motolii_ui::{
    builtin_command_registry, CommandId, DocumentCommandRequest, DocumentCommandRequestError,
    DomainIntent, InputPhase, InputRouter, NormalizedInput, RouterOutput, SafetyInterrupt,
};
use syn::visit::Visit;

struct Fixture {
    doc: Document,
    layers: [LayerId; 3],
    track: TrackId,
}

fn fixture() -> Fixture {
    let mut doc = Document::new_current();
    let layers = [
        doc.layers.allocate("a").unwrap(),
        doc.layers.allocate("b").unwrap(),
        doc.layers.allocate("c").unwrap(),
    ];
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: layers
            .iter()
            .copied()
            .map(|layer| {
                TrackItem::Clip(Clip {
                    envelope: ItemEnvelope::new(layer),
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(1, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::asset_video_only(asset),
                })
            })
            .collect(),
    });
    doc.validate().unwrap();
    Fixture { doc, layers, track }
}

fn writer(doc: Document) -> DocumentWriter {
    let catalog = PluginCatalogBuilder::new().build().unwrap();
    DocumentWriter::new(doc, Arc::new(catalog)).unwrap()
}

fn remove_command(doc: &Document, track: TrackId, index: usize) -> Command {
    let item = doc.tracks[0].items[index].clone();
    let layer_names = layer_names_for_item(doc, &item).unwrap();
    Command::RemoveTrackItem {
        parent: ParentLocator::Track(track),
        index,
        item,
        layer_names,
    }
}

fn writer_state(writer: &DocumentWriter) -> (Vec<u8>, u64, usize, usize) {
    (
        serde_json::to_vec(&*writer.snapshot()).unwrap(),
        writer.revision,
        writer.undo_len(),
        writer.redo_len(),
    )
}

#[test]
fn one_and_multiple_target_requests_use_one_atomic_macro_each() {
    let f = fixture();
    let mut writer = writer(f.doc.clone());

    let first = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![remove_command(&f.doc, f.track, 2)],
    )
    .unwrap();
    assert_eq!(first.intent(), DomainIntent::DeleteTargetedItems);
    let first_gesture = writer.apply_macro(first.into_commands()).unwrap();
    assert_eq!(writer.undo_len(), 1);
    assert_eq!(writer.snapshot().tracks[0].items.len(), 2);

    let after_first = writer.snapshot();
    let second = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![
            remove_command(&after_first, f.track, 1),
            remove_command(&after_first, f.track, 0),
        ],
    )
    .unwrap();
    let second_gesture = writer.apply_macro(second.into_commands()).unwrap();

    assert_ne!(first_gesture, second_gesture);
    assert_eq!(writer.revision, 2);
    assert_eq!(writer.undo_len(), 2);
    assert!(writer.snapshot().tracks[0].items.is_empty());

    writer.undo().unwrap();
    assert_eq!(&*writer.snapshot(), &*after_first);
    writer.undo().unwrap();
    assert_eq!(&*writer.snapshot(), &f.doc);
    writer.redo().unwrap();
    assert_eq!(&*writer.snapshot(), &*after_first);
    writer.redo().unwrap();
    assert!(writer.snapshot().tracks[0].items.is_empty());
}

#[test]
fn repeated_request_for_the_same_target_gets_a_fresh_gesture() {
    let f = fixture();
    let mut writer = writer(f.doc.clone());
    let command = remove_command(&f.doc, f.track, 1);

    let first =
        DocumentCommandRequest::try_new(DomainIntent::DeleteTargetedItems, vec![command.clone()])
            .unwrap();
    let first_gesture = writer.apply_macro(first.into_commands()).unwrap();
    writer.undo().unwrap();

    let second =
        DocumentCommandRequest::try_new(DomainIntent::DeleteTargetedItems, vec![command]).unwrap();
    let second_gesture = writer.apply_macro(second.into_commands()).unwrap();

    assert_ne!(first_gesture, second_gesture);
    assert_eq!(writer.undo_len(), 1);
    writer.undo().unwrap();
    assert_eq!(&*writer.snapshot(), &f.doc);
}

#[test]
fn invalid_requests_are_typed_and_never_reach_the_writer() {
    let f = fixture();
    let writer = writer(f.doc.clone());
    let before = writer_state(&writer);

    assert!(matches!(
        DocumentCommandRequest::try_new(DomainIntent::DeleteTargetedItems, Vec::new()),
        Err(DocumentCommandRequestError::EmptyCommands)
    ));

    for intent in [
        DomainIntent::EnableReduceMotion,
        DomainIntent::ResetWorkspaceProfile,
        DomainIntent::FitStageView,
        DomainIntent::CancelInFlightGesture,
    ] {
        assert!(matches!(
            DocumentCommandRequest::try_new(
                intent,
                vec![remove_command(&f.doc, f.track, 0)]
            ),
            Err(DocumentCommandRequestError::NonDocumentIntent { intent: got }) if got == intent
        ));
    }

    let mismatch = Command::SetProperty {
        target: f.layers[0],
        property: ScalarPropertyId::Opacity,
        old_value: DocParam::const_f64(1.0),
        new_value: DocParam::const_f64(0.5),
    };
    assert_eq!(
        DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![remove_command(&f.doc, f.track, 2), mismatch],
        )
        .unwrap_err(),
        DocumentCommandRequestError::CommandKindMismatch {
            intent: DomainIntent::DeleteTargetedItems,
            index: 1,
            expected: CommandKind::RemoveTrackItem,
            actual: CommandKind::SetProperty,
        }
    );
    assert_eq!(writer_state(&writer), before);
}

#[test]
fn dropped_request_and_router_cancels_leave_the_writer_unchanged() {
    let f = fixture();
    let writer = writer(f.doc.clone());
    let before = writer_state(&writer);
    let request = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![remove_command(&f.doc, f.track, 0)],
    )
    .unwrap();
    drop(request);

    let mut router = InputRouter::new(builtin_command_registry().unwrap());
    router
        .route(NormalizedInput::Phase(InputPhase::DragStart))
        .unwrap();
    assert!(matches!(
        router
            .route(NormalizedInput::SafetyInterrupt(
                SafetyInterrupt::PointerCaptureLost
            ))
            .unwrap(),
        RouterOutput::SafetyCancel {
            intent: DomainIntent::CancelInFlightGesture,
            ..
        }
    ));

    let mut router = InputRouter::new(builtin_command_registry().unwrap());
    router
        .route(NormalizedInput::Phase(InputPhase::DragStart))
        .unwrap();
    let cancel_id = CommandId::try_new("motolii.gesture.cancel").unwrap();
    assert_eq!(
        router
            .route(NormalizedInput::Command {
                phase: InputPhase::Press,
                id: cancel_id.clone(),
            })
            .unwrap(),
        RouterOutput::Intent {
            phase: InputPhase::Cancel,
            id: cancel_id,
            intent: DomainIntent::CancelInFlightGesture,
        }
    );

    assert_eq!(writer_state(&writer), before);
}

#[derive(Default)]
struct BoundaryVisitor {
    violations: Vec<String>,
}

impl<'ast> Visit<'ast> for BoundaryVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments: Vec<_> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        for forbidden in [
            "egui",
            "eframe",
            "winit",
            "serde",
            "Serialize",
            "Deserialize",
            "Document",
            "DocumentWriter",
            "UndoHistory",
            "GestureId",
        ] {
            if segments.iter().any(|segment| segment == forbidden) {
                self.violations.push(segments.join("::"));
            }
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_type_reference(&mut self, reference: &'ast syn::TypeReference) {
        if reference.mutability.is_some() {
            self.violations.push("mutable reference".into());
        }
        syn::visit::visit_type_reference(self, reference);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if ["sort", "sort_by", "sort_by_key", "reverse"].contains(&call.method.to_string().as_str())
        {
            self.violations
                .push(format!("reordering method {}", call.method));
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

#[test]
fn request_boundary_has_no_toolkit_persistence_writer_or_planner_contract() {
    let source = include_str!("../src/document_command_request.rs");
    let syntax = syn::parse_file(source).unwrap();
    let mut visitor = BoundaryVisitor::default();
    visitor.visit_file(&syntax);
    assert!(
        visitor.violations.is_empty(),
        "runtime request boundary violations: {:?}",
        visitor.violations
    );
}
