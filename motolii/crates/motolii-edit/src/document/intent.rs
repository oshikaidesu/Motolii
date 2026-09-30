use motolii_doc::eval::Value;
use motolii_doc::store::ids::{LayerId, PropertyId};
use motolii_doc::store::slot::{PropertyLink, Slot, SlotId};
use motolii_doc::store::{LayerAttrsPatch, Mask};
#[derive(Clone, Debug)]
pub enum Intent {
    AddLayer(LayerId),
    RemoveLayer(LayerId),
    SetTrack {
        layer: LayerId,
        property: PropertyId,
        track: motolii_doc::eval::KeyframeTrack,
    },
    /// **動かない値**を置く。キーは作らない —— 利用者が ◇ を押すまで
    /// 時間の世界へ入れない(根底3)。
    SetConstant {
        layer: LayerId,
        property: PropertyId,
        value: motolii_doc::eval::Value,
    },
    SetPropertySlot {
        layer: LayerId,
        property: PropertyId,
        slot: SlotId,
    },
    SetPropertyLink {
        layer: LayerId,
        property: PropertyId,
        link: PropertyLink,
    },
    SetPropertyModulators {
        layer: LayerId,
        property: PropertyId,
        modulators: Vec<PropertyLink>,
    },
    SetCameraPropertyModulators {
        property: PropertyId,
        modulators: Vec<PropertyLink>,
    },
    SetMeta {
        layer: LayerId,
        meta: motolii_doc::store::LayerMeta,
    },
    SetSource {
        layer: LayerId,
        source: motolii_doc::store::LayerSource,
    },
    SetOrder {
        layer: LayerId,
        order: i16,
    },
    SetMasks {
        layer: LayerId,
        masks: Vec<motolii_doc::store::Mask>,
    },
    AddMask {
        layer: LayerId,
        mask: Mask,
        shape: motolii_doc::eval::KeyframeTrack,
    },
    SetTiming {
        layer: LayerId,
        timing: motolii_doc::store::LayerTiming,
    },
    SetAttrs {
        layer: LayerId,
        patch: LayerAttrsPatch,
    },
    SetEffects {
        layer: LayerId,
        effects: Vec<motolii_doc::store::EffectInstance>,
    },
    SetShapes {
        layer: LayerId,
        shapes: Vec<motolii_doc::store::ShapeNode>,
    },
    SetTextDocument {
        layer: LayerId,
        document: motolii_doc::store::TextDocument,
    },
    SetComposition(motolii_doc::store::Composition),
    SetNotebook { notebook: motolii_doc::store::Notebook },
    SetMarkers {
        markers: Vec<motolii_doc::store::Marker>,
    },
    /// カメラの属性へ**素の値**を置く。層側の `SetConstant` と同じ意味で、
    /// 置き場が composition なだけ。
    SetCameraConstant {
        property: PropertyId,
        value: Value,
    },
    SetCameraTrack {
        property: PropertyId,
        track: motolii_doc::eval::KeyframeTrack,
    },
    SetCameraPropertySlot {
        property: PropertyId,
        slot: SlotId,
    },
    SetSlots {
        slots: Vec<Slot>,
    },
    AdmitAsset {
        draft: motolii_doc::store::AssetDraft,
    },
    RemoveAsset {
        asset: motolii_doc::store::AssetId,
    },
    RelinkAsset {
        asset: motolii_doc::store::AssetId,
        path_absolute: String,
        project_root: Option<String>,
    },
    Freeze {
        group: LayerId,
    },
    Unfreeze {
        group: LayerId,
    },
}
