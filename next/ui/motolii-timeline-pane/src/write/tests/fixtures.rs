//! 第3切片(B15+B52)のテスト共通 fixture。**テストは書くが実行しない**
//! (裁定189 追いつきターン — supervisor が波末一括で回す)。SP-2 分割で
//! 元は `third_slice_fixtures`(`write.rs` 内の兄弟モジュール)だった物を
//! そのまま移設(中身は無改変)。

use crate::write::*;
use motolii_store::{Fps, Keyframe, LayerMeta, LayerSource, Value};

pub(super) fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).expect("30/1 は正の既約 fps"),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp 設定");
    doc
}

pub(super) fn place(doc: &mut Document, layer: LayerId, order: i16) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid { rgba: [255, 0, 0, 255], width: 64, height: 64 },
                order,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("layer 配置");
}

pub(super) fn lock(doc: &mut Document, layer: LayerId) {
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch { locked: Some(true), ..Default::default() },
    })
    .expect("ロック設定");
}

pub(super) fn track_with(frames: &[i64]) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    for &frame in frames {
        track.insert(Keyframe {
            t: RationalTime::try_new(frame, 30).expect("frame は収まる"),
            value: Value::F64(1.0),
            interp: Interp::Linear,
            spatial: None,
        });
    }
    track
}

pub(super) fn set_track(doc: &mut Document, layer: LayerId, property: &PropertyId, frames: &[i64]) {
    doc.apply(Intent::SetTrack { layer, property: property.clone(), track: track_with(frames) })
        .expect("track を書ける");
}

pub(super) fn selector(layer: LayerId, property: &PropertyId, frame: i64) -> KeySelector {
    KeySelector { layer, property: property.clone(), frame }
}

pub(super) fn opacity() -> PropertyId {
    PropertyId::new(motolii_store::property::OPACITY).expect("opacity は予約語ではない")
}

pub(super) fn position_x() -> PropertyId {
    PropertyId::new(motolii_store::property::POSITION_X).expect("position.x は予約語ではない")
}

/// track の keys を(時刻順のまま)`(frame, interp)` へ写す検分ヘルパー。
pub(super) fn interps_of(doc: &Document, layer: LayerId, property: &PropertyId) -> Vec<(i64, Interp)> {
    let fps = doc.view().composition().unwrap().unwrap().fps;
    doc.view()
        .track(layer, property)
        .unwrap()
        .unwrap()
        .keys()
        .iter()
        .map(|key| (key.t.try_to_frame_round(fps).unwrap(), key.interp))
        .collect()
}

pub(super) fn no_mods() -> iced::keyboard::Modifiers {
    iced::keyboard::Modifiers::default()
}
