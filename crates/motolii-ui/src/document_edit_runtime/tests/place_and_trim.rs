use super::super::*;
use super::fixtures::*;

#[test]
fn rectangle_place_uses_one_add_command_and_publishes_the_same_layer_id() {
    let (document, _) = fixture();
    let selected = fixture_layer(&document);
    let initial_next = document.layers.peek_next();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_place_rectangle(PlaceRectangleRequest {
        position: [0.25, -0.125],
        playhead: RationalTime::try_new(1, 1).unwrap(),
    });

    let published = runtime
        .process_next(&mut queue, Some(selected), 4)
        .unwrap()
        .expect("published Rectangle");
    let placed = published.primary.expect("placed selection receipt");
    assert_eq!(placed.get(), initial_next);
    assert_eq!(published.kind, DocumentEditActionKind::PlaceRectangle);
    assert_eq!(published.revision, 1);
    assert_eq!(published.projection_generation, 5);
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert_eq!(published.snapshot.tracks[0].items.len(), 2);
    let TrackItem::Clip(clip) = &published.snapshot.tracks[0].items[1] else {
        panic!("Rectangle must be a Clip");
    };
    assert_eq!(clip.envelope.layer_id, placed);
    assert_eq!(
        clip.envelope.transform.position,
        motolii_doc::DocParam::const_vec2([0.25, -0.125])
    );
    assert_eq!(clip.start, RationalTime::try_new(1, 1).unwrap());
    assert_eq!(
        clip.duration,
        published
            .snapshot
            .composition
            .duration
            .try_sub(clip.start)
            .unwrap()
    );
    assert!(matches!(
        clip.source,
        ClipSource::Vector {
            recipe: VectorRecipe {
                content: VectorContent::StandardShape {
                    shape: StandardShape::Rect { .. }
                },
                ..
            }
        }
    ));
    assert_eq!(
        published.snapshot.layers.display_name(placed),
        Some("Rectangle")
    );
}

#[test]
fn ellipse_place_uses_one_add_command_and_publishes_the_same_layer_id() {
    let (document, _) = fixture();
    let selected = fixture_layer(&document);
    let initial_next = document.layers.peek_next();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_place_ellipse(PlaceEllipseRequest {
        position: [0.25, -0.125],
        playhead: RationalTime::try_new(1, 1).unwrap(),
    });

    let published = runtime
        .process_next(&mut queue, Some(selected), 4)
        .unwrap()
        .expect("published Ellipse");
    let placed = published.primary.expect("placed selection receipt");
    assert_eq!(placed.get(), initial_next);
    assert_eq!(published.kind, DocumentEditActionKind::PlaceEllipse);
    assert_eq!(published.revision, 1);
    assert_eq!(published.projection_generation, 5);
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert_eq!(published.snapshot.tracks[0].items.len(), 2);
    let TrackItem::Clip(clip) = &published.snapshot.tracks[0].items[1] else {
        panic!("Ellipse must be a Clip");
    };
    assert_eq!(clip.envelope.layer_id, placed);
    assert_eq!(
        clip.envelope.transform.position,
        motolii_doc::DocParam::const_vec2([0.25, -0.125])
    );
    assert_eq!(clip.start, RationalTime::try_new(1, 1).unwrap());
    assert_eq!(
        clip.duration,
        published
            .snapshot
            .composition
            .duration
            .try_sub(clip.start)
            .unwrap()
    );
    let ClipSource::Vector {
        recipe:
            VectorRecipe {
                content: VectorContent::StandardShape { shape },
                modifiers,
            },
    } = &clip.source
    else {
        panic!("Ellipse must be a Vector clip");
    };
    assert!(modifiers.is_empty());
    assert_eq!(
        *shape,
        StandardShape::Ellipse {
            width: motolii_doc::DocParam::const_f64(0.2),
            height: motolii_doc::DocParam::const_f64(0.2),
        }
    );
    assert!(!matches!(shape, StandardShape::Rect { .. }));
    assert_eq!(
        published.snapshot.layers.display_name(placed),
        Some("Ellipse")
    );
}

#[test]
fn rejected_ellipse_place_changes_no_document_counter_history_or_revision() {
    let (document, _) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let initial_next = document.layers.peek_next();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_place_ellipse(PlaceEllipseRequest {
        position: [f64::NAN, 0.0],
        playhead: RationalTime::ZERO,
    });
    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::NonFiniteDropPosition)
    ));
    assert_eq!(runtime.snapshot().layers.peek_next(), initial_next);
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}

#[test]
fn media_place_admits_asset_and_publishes_the_same_layer_id() {
    let (document, _) = fixture();
    let initial_next = document.layers.peek_next();
    let initial_assets = document.assets.len();
    let (_path, mut runtime) = open_runtime(document);
    let media = crate::media_library::default_media_library_root().join("starter-still.png");
    let mut queue = DocumentEditQueue::default();
    queue.push_place_media(PlaceMediaRequest {
        path: media,
        name: "starter-still.png".into(),
        kind: "image".into(),
        asset_type: "image/png".into(),
        position: [0.1, -0.2],
        playhead: RationalTime::ZERO,
    });

    let published = runtime
        .process_next(&mut queue, None, 0)
        .unwrap()
        .expect("published media");
    let placed = published.primary.expect("placed selection receipt");
    assert_eq!(placed.get(), initial_next);
    assert_eq!(published.kind, DocumentEditActionKind::PlaceMedia);
    assert_eq!(published.snapshot.assets.len(), initial_assets + 1);
    assert_eq!(
        published.snapshot.layers.display_name(placed),
        Some("starter-still.png")
    );
    let TrackItem::Clip(clip) = published
        .snapshot
        .tracks
        .first()
        .and_then(|track| track.items.last())
        .expect("placed clip")
    else {
        panic!("media must be a Clip");
    };
    assert_eq!(clip.envelope.layer_id, placed);
    assert!(matches!(
        clip.source,
        ClipSource::Asset { video: Some(_), .. }
    ));
    assert_eq!(
        clip.envelope.transform.position,
        motolii_doc::DocParam::const_vec2([0.1, -0.2])
    );

    queue.push_undo();
    let undone = runtime.process_next(&mut queue, None, 1).unwrap().unwrap();
    assert!(undone
        .snapshot
        .tracks
        .iter()
        .flat_map(|track| track.items.iter())
        .all(|item| item_layer_id(item) != placed));
}

#[test]
fn missing_media_file_is_consumed_without_document_or_history_change() {
    let (document, _) = fixture();
    let before = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_place_media(PlaceMediaRequest {
        path: std::path::PathBuf::from("/no/such/motolii-media.png"),
        name: "missing.png".into(),
        kind: "image".into(),
        asset_type: "image/png".into(),
        position: [0.0, 0.0],
        playhead: RationalTime::ZERO,
    });
    let pre_history = runtime.history_lengths();
    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::LibraryFileUnreadable)
    ));
    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.history_lengths(), pre_history);
    assert_eq!(serde_json::to_vec(&*runtime.snapshot()).unwrap(), before);
    assert!(!runtime.is_write_blocked());
}

#[test]
fn move_clip_commits_once_and_undo_restores_the_original_start() {
    let (document, _) = fixture();
    let layer = fixture_layer(&document);
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_move_clip(TimelineMoveRequest {
        layer,
        new_start: RationalTime::try_new(1, 4).unwrap(),
    });

    let moved = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
    assert_eq!(moved.kind, DocumentEditActionKind::MoveClip);
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert_eq!(
        clip_start(&moved.snapshot, layer),
        RationalTime::try_new(1, 4).unwrap()
    );

    queue.push_undo();
    let undone = runtime.process_next(&mut queue, None, 1).unwrap().unwrap();
    assert_eq!(undone.kind, DocumentEditActionKind::Undo);
    assert_eq!(clip_start(&undone.snapshot, layer), RationalTime::ZERO);
}

#[test]
fn invalid_move_is_consumed_without_document_or_history_change() {
    let (document, _) = fixture();
    let layer = fixture_layer(&document);
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_move_clip(TimelineMoveRequest {
        layer,
        new_start: RationalTime::try_new(99, 1).unwrap(),
    });

    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::Command(_))
    ));
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(runtime.revision(), 0);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}

fn clip_start(document: &Document, layer: LayerId) -> RationalTime {
    document
        .tracks
        .iter()
        .flat_map(|track| track.items.iter())
        .find_map(|item| match item {
            TrackItem::Clip(clip) if clip.envelope.layer_id == layer => Some(clip.start),
            _ => None,
        })
        .expect("fixture clip")
}

#[test]
fn trim_preview_is_read_only_and_left_release_commits_once_with_undo() {
    let (document, _) = fixture();
    let layer = fixture_layer(&document);
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let request = TimelineTrimRequest::In {
        layer,
        new_start: RationalTime::try_new(1, 4).unwrap(),
    };

    let preview = runtime
        .preview_trim(request)
        .unwrap()
        .expect("changed trim must produce a transient snapshot");
    assert_eq!(
        clip_interval(&preview, layer),
        (
            RationalTime::try_new(1, 4).unwrap(),
            RationalTime::try_new(1, 1).unwrap(),
        )
    );
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(runtime.revision(), 0);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );

    let mut queue = DocumentEditQueue::default();
    queue.push_trim_clip(request);
    let trimmed = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
    assert_eq!(trimmed.kind, DocumentEditActionKind::TrimClip);
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert_eq!(
        clip_interval(&trimmed.snapshot, layer),
        clip_interval(&preview, layer)
    );

    queue.push_undo();
    let undone = runtime.process_next(&mut queue, None, 1).unwrap().unwrap();
    assert_eq!(undone.kind, DocumentEditActionKind::Undo);
    assert_eq!(
        clip_interval(&undone.snapshot, layer),
        (RationalTime::ZERO, RationalTime::try_new(1, 1).unwrap())
    );
}

#[test]
fn right_trim_commits_duration_only_and_invalid_edge_writes_nothing() {
    let (document, _) = fixture();
    let layer = fixture_layer(&document);
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_trim_clip(TimelineTrimRequest::Out {
        layer,
        new_end: RationalTime::try_new(1, 2).unwrap(),
    });

    let trimmed = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
    assert_eq!(trimmed.kind, DocumentEditActionKind::TrimClip);
    assert_eq!(
        clip_interval(&trimmed.snapshot, layer),
        (RationalTime::ZERO, RationalTime::try_new(1, 2).unwrap())
    );
    assert_eq!(runtime.history_lengths(), (1, 0));

    let before_invalid = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let revision = runtime.revision();
    queue.push_trim_clip(TimelineTrimRequest::Out {
        layer,
        new_end: RationalTime::ZERO,
    });
    assert!(matches!(
        runtime.process_next(&mut queue, None, 1),
        Err(DocumentEditRuntimeError::Command(_))
    ));
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert_eq!(runtime.revision(), revision);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        before_invalid
    );
}

fn clip_interval(document: &Document, layer: LayerId) -> (RationalTime, RationalTime) {
    document
        .tracks
        .iter()
        .flat_map(|track| track.items.iter())
        .find_map(|item| match item {
            TrackItem::Clip(clip) if clip.envelope.layer_id == layer => Some((
                clip.start,
                clip.start.try_add(clip.duration).expect("fixture interval"),
            )),
            _ => None,
        })
        .expect("fixture clip")
}

#[test]
fn rejected_rectangle_place_changes_no_document_counter_history_or_revision() {
    let (document, _) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let initial_next = document.layers.peek_next();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_place_rectangle(PlaceRectangleRequest {
        position: [f64::NAN, 0.0],
        playhead: RationalTime::ZERO,
    });
    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::NonFiniteDropPosition)
    ));
    assert_eq!(runtime.snapshot().layers.peek_next(), initial_next);
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}
