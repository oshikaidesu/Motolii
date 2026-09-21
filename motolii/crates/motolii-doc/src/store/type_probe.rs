//! 同じ component が編集ごとに別の Arrow の型を持てるか。
//!
//! 持てるなら、動く値を `TrackJson`(文字列)から型のある component へ移す時に、
//! 消しの札(tombstone)も名前の付け替えも要らない — 新しい型の行が古い型を覆う。
//! 持てないなら、移行は名前を分ける設計になる。読むだけでは決まらないので試す。

use crate::doc::store::components::{LayerPresent, TrackJson};

#[test]
fn one_component_may_change_type_across_edits() {
    use re_log_types::{EntityPath, TimePoint, Timeline};
    use re_types_core::{Component as _, ComponentDescriptor, Loggable, SerializedComponentBatch};

    let mut db = re_entity_db::EntityDb::new(re_log_types::StoreId::random(
        re_log_types::StoreKind::Recording,
        "motolii",
    ));
    let path = EntityPath::from("/layer/1");
    let timeline = Timeline::new_sequence("edit");
    let component: re_types_core::ComponentIdentifier = "Layer:probe".into();

    let as_text = ComponentDescriptor { archetype: None, component, component_type: Some(TrackJson::name()) };
    let as_bool = ComponentDescriptor { archetype: None, component, component_type: Some(LayerPresent::name()) };

    let mut write = |at: i64, batch: SerializedComponentBatch| {
        let chunk = re_chunk::Chunk::builder(path.clone())
            .with_serialized_batches(re_chunk::RowId::new(), TimePoint::default().with(timeline, at), vec![batch])
            .build()
            .unwrap();
        db.add_chunk(&std::sync::Arc::new(chunk)).unwrap();
    };

    write(1, SerializedComponentBatch {
        descriptor: as_text,
        array: <TrackJson as Loggable>::to_arrow([TrackJson("{\"k\":1}".to_owned())]).unwrap(),
    });
    write(2, SerializedComponentBatch {
        descriptor: as_bool,
        array: <LayerPresent as Loggable>::to_arrow([LayerPresent(true)]).unwrap(),
    });

    let at = |t: i64| re_chunk_store::LatestAtQuery::new(*timeline.name(), t);
    let first = db.latest_at(&at(1), &path, [component]);
    let second = db.latest_at(&at(2), &path, [component]);

    assert!(first.component_batch::<TrackJson>(component).is_some(), "編集 1 では文字列が読める");
    assert!(second.component_batch::<LayerPresent>(component).is_some(), "編集 2 では bool が読める");
    assert!(second.component_batch::<TrackJson>(component).is_none(), "新しい型が古い型を覆う");
}
