//! Blob Track — 絵から拾った塊を配置として返す配置効果(2026-09-13 裁定: 拾う元 3 つ、ID の持続 / 非持続、描く物は開く)。
//! 先例は TouchDesigner の Blob Track TOP。塊は host が元の層を描いて解き(`AnalysisInputs`)、resolve はそれを読むだけ。
//! 箱・十字・線・文字は、この効果を持つ層(素材)を塊ごとに置いて作る。

use crate::doc::eval::Value;
use crate::doc::store::kind::{Param, ParamKind};

pub struct BlobKind {
    pub plugin_id: &'static str,
    pub label: &'static str,
    pub params: &'static [Param],
}

pub const BLOB_TRACK: &str = "motolii.blob_track";
pub const SOURCES: &[&str] = &["Brightness", "Motion", "Color"];
pub const FITS: &[&str] = &["Position", "Box"];
const SWITCH: &[&str] = &["Off", "On"];

const fn number(name: &'static str, label: &'static str, section: &'static str, default: f64, range: (f64, f64)) -> Param {
    Param { name, label, section, kind: ParamKind::Number, default: [default, 0.0], range: Some(range), modes: None }
}
const fn choice(name: &'static str, label: &'static str, section: &'static str, choices: &'static [&'static str], default: f64) -> Param {
    Param { name, label, section, kind: ParamKind::Choice(choices), default: [default, 0.0], range: Some((0.0, (choices.len() - 1) as f64)), modes: None }
}

pub const KINDS: &[BlobKind] = &[BlobKind {
    plugin_id: BLOB_TRACK,
    label: "Blob Track",
    params: &[
        Param { name: "source", label: "Track Layer", section: "Find", kind: ParamKind::Layer, default: [0.0, 0.0], range: None, modes: None },
        choice("mode", "Find By", "Find", SOURCES, 0.0),
        number("threshold", "Threshold", "Find", 0.5, (0.0, 1.0)),
        choice("invert", "Invert", "Find", SWITCH, 0.0),
        number("red", "Red", "Color", 1.0, (0.0, 1.0)),
        number("green", "Green", "Color", 0.0, (0.0, 1.0)),
        number("blue", "Blue", "Color", 0.0, (0.0, 1.0)),
        number("tolerance", "Tolerance", "Color", 0.25, (0.0, 2.0)),
        // 解析する絵の長辺(px)。細かいほど人どうしの隙間が残り、塊が小さくなる(重くなる)。
        number("detail", "Detail", "Find", 960.0, (120.0, 3840.0)),
        number("min_area", "Min Size", "Filter", 200.0, (0.0, 1.0e9)),
        number("max_area", "Max Size", "Filter", 1.0e9, (0.0, 1.0e9)),
        // 数える前に削る幅(comp の px)。細い橋で触れた塊を切り離す。
        number("separation", "Separation", "Filter", 2.0, (0.0, 200.0)),
        number("max_blobs", "Max Blobs", "Filter", 100.0, (1.0, 1000.0)),
        choice("persist", "Keep IDs", "Track", SWITCH, 1.0),
        number("max_move", "Max Move", "Track", 40.0, (0.0, 10000.0)),
        number("revive", "Revive Frames", "Track", 5.0, (0.0, 1000.0)),
        choice("fit", "Fit", "Place", FITS, 1.0),
        Param { name: "material", label: "Material Size", section: "Place", kind: ParamKind::Vec2, default: [100.0, 100.0], range: None, modes: None },
    ],
}];

pub fn is_blob_track(plugin_id: &str) -> bool {
    plugin_id == BLOB_TRACK
}

pub fn program(plugin_id: &str) -> Option<crate::doc::store::kind::PlacementProgram> {
    KINDS.iter().find(|kind| kind.plugin_id == plugin_id).map(|kind| crate::doc::store::kind::PlacementProgram {
        plugin_id: kind.plugin_id,
        needs_position: true,
        evaluate: placements,
        pick: crate::doc::store::kind::pick_in_turn,
        moves_whole: crate::doc::store::kind::never_moves_whole,
    })
}

fn placements(input: &crate::doc::store::kind::PlacementInput<'_>) -> Vec<crate::doc::store::kind::PlacementOutput> {
    use crate::doc::store::{kind::PlacementOutput, EffectId, Placement, RationalTime};
    let Some(marks) = input.analysis.and_then(|a| a.blobs(input.layer, EffectId(0), input.time)) else { return Vec::new() };
    let position = glam::Vec2::from(input.position);
    let material = vec2_of(input.params, "material").map(|v| v.max(1e-3) as f32);
    let fit_box = number_of(input.params, "fit") >= 0.5;
    marks.iter().map(|mark| {
        let stretch = if fit_box { [mark.size[0] / material[0], mark.size[1] / material[1]] } else { [1.0, 1.0] };
        let half = glam::vec2(material[0] * stretch[0], material[1] * stretch[1]) * 0.5;
        let offset = glam::Vec2::from(mark.center) - position - half;
        let (placed, outline_stretch) = if input.stretch_outline { ([1.0, 1.0], stretch) } else { (stretch, [1.0, 1.0]) };
        PlacementOutput { placement: Placement { index: mark.id, offset: offset.into(), rotation_degrees: 0.0, scale: 1.0, offset_z: 0.0, opacity: 1.0, time_offset: RationalTime::ZERO, stretch: placed }, outline_stretch }
    }).collect()
}

/// 取っ手の値。無い欄は宣言の既定。
pub fn number_of(params: &[(String, Value)], name: &str) -> f64 {
    let default = KINDS[0].params.iter().find(|p| p.name == name).map_or(0.0, |p| p.default[0]);
    match params.iter().find(|(n, _)| n == name).map(|(_, v)| v) {
        Some(Value::F64(v)) if v.is_finite() => *v,
        Some(Value::LayerId(id)) => *id as f64,
        _ => default,
    }
}

pub fn vec2_of(params: &[(String, Value)], name: &str) -> [f64; 2] {
    let default = KINDS[0].params.iter().find(|p| p.name == name).map_or([0.0; 2], |p| p.default);
    match params.iter().find(|(n, _)| n == name).map(|(_, v)| v) {
        Some(Value::Vec2(v)) if v.iter().all(|c| c.is_finite()) => *v,
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::core::RationalTime;
    use crate::doc::store::analysis::{AnalysisInputs, BlobMark};
    use crate::doc::store::{property, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, Value};

    /// 塊は解析の入力から来る: 入力が無ければ置かれず、あれば素材を塊ごとに置き、Box なら箱の大きさへ伸ばす(左上が層の位置の素材)。
    #[test]
    fn blob_track_places_the_material_on_each_mark_from_the_analysis() {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: 640, height: 360, fps: Fps::try_new(25, 1).unwrap(), duration_frames: 10, background: [0.0; 4] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order: 0, timing: LayerTiming::place(0, None, 10) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([10.0, 20.0]) },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: super::BLOB_TRACK.into() }] },
        ]).unwrap();
        let t = RationalTime::ZERO;
        let copies = |view: crate::doc::store::StoreView<'_>| crate::picture::resolve::resolved_layers(&view, t).unwrap().into_iter().filter(|l| l.id == layer).collect::<Vec<_>>();
        assert!(copies(doc.view()).is_empty(), "解析の入力が無ければ置かれない");
        let mut inputs = AnalysisInputs::default();
        inputs.set_blobs(layer, EffectId(0), t, vec![
            BlobMark { id: 7, center: [100.0, 100.0], size: [50.0, 20.0], age: 0 },
            BlobMark { id: 9, center: [300.0, 200.0], size: [100.0, 100.0], age: 3 },
        ]);
        let placed = copies(doc.view().with_analysis(&inputs));
        assert_eq!(placed.iter().map(|l| l.copy).collect::<Vec<_>>(), vec![7, 9]);
        // 素材 100 × 100 の左上(0, 0)と右下(100, 100)が、箱の左上と右下へ。
        let box_of = |l: &crate::doc::store::ResolvedLayer| (l.placement.transform.transform_point2(glam::Vec2::ZERO), l.placement.transform.transform_point2(glam::vec2(100.0, 100.0)));
        let (lo, hi) = box_of(&placed[0]);
        assert!(lo.distance(glam::vec2(75.0, 90.0)) < 1e-3 && hi.distance(glam::vec2(125.0, 110.0)) < 1e-3, "{lo} {hi}");
        let (lo, hi) = box_of(&placed[1]);
        assert!(lo.distance(glam::vec2(250.0, 150.0)) < 1e-3 && hi.distance(glam::vec2(350.0, 250.0)) < 1e-3, "{lo} {hi}");
        assert_eq!(placed[0].shape_stretch, [1.0, 1.0], "形でない素材は置き場所で伸ばす");
        // 形の素材は置き場所では伸ばさず、輪郭を伸ばす倍率を渡す(線は太らない)。
        doc.apply(Intent::SetSource { layer, source: LayerSource::Shape }).unwrap();
        let shaped = copies(doc.view().with_analysis(&inputs));
        assert_eq!(shaped[0].shape_stretch, [0.5, 0.2]);
        let lo = shaped[0].placement.transform.transform_point2(glam::Vec2::ZERO);
        let unit = shaped[0].placement.transform.transform_point2(glam::vec2(1.0, 1.0)) - lo;
        assert!(lo.distance(glam::vec2(75.0, 90.0)) < 1e-3 && unit.distance(glam::Vec2::ONE) < 1e-3, "置き場所は動かすだけ: {lo} {unit}");
    }
}
