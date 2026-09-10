
mod asset;
mod attrs;
mod components;
mod document;
mod effect;
mod fingerprint;
mod marker;
mod mask;
mod persist;
pub mod kind;
pub mod placement;
mod slot;
mod text;
mod view;

pub use asset::{Asset, AssetDraft, AssetError, AssetId, AssetRole, AssetStatus, AssetTable};
pub use attrs::{BlendMode, LayerAttrs, LayerAttrsPatch, LayerProjection, Matte, MatteMode, LABEL_PALETTE_LEN};
pub use document::{Animate, DisplayRevision, Document, Intent, LayerId, PropertyId, Revision};
pub use effect::{EffectId, EffectInstance, ResolvedEffect};
pub use placement::Placement;
pub use fingerprint::{SourceFingerprintDecode, SourceFingerprintError, SourceFingerprintV1};
pub use marker::Marker;
mod notebook;
pub use notebook::{Notebook, NotePage, NoteBlock, NoteContent};
pub use mask::{Mask, MaskId, MaskMode, ResolvedMask};
pub use persist::AutoSaveConfig;
pub use slot::{PropertyBase, PropertyLink, PropertySource, Slot, SlotId};
pub use text::{
    ContentKeyframe, ContentTrack, FontRef, TextAlignmentOptions, TextBasedOn, TextDocument,
    TextDocumentStyle, TextGrouping, TextJustify, TextRandomize, TextRange, TextRangeId,
    TextRangeSelector, TextRangeUnits, TextRun, TextShape, TextStyleAxis, TextStyleFeature,
    TextStyleId, TextVariationAxis,
};
pub use view::StoreView;

pub use crate::doc::core::{CompSpec, Fps, LayerPlacement, RationalTime, ResolvedCamera};
pub use crate::doc::eval::{Interp, Keyframe, KeyframeTrack, Path, PathVertex, SpatialTangent, Value};
pub use crate::doc::vector::{
    OpKind, PathSource, Point as VectorPoint, RepeaterTransform, Shape, ShapeGroup, ShapeNode,
    ShapeOp,
};

/// 単色の四角。AE の平面(Solid)に当たる物は、専用の型ではなくこれで作る。
///
/// 矩形の左上が層の原点に来る(`PathSource::Rectangle` は原点中心なので、
/// 群の transform で半分ずらす)。
pub fn rect_shape(rgba: [u8; 4], size: [f32; 2]) -> ShapeNode {
    use crate::doc::vector::{Brush, Fill, FillRule, RepeaterTransform, Rgb, ShapeGroup};
    let leaf = ShapeNode::Leaf(Shape {
        source: PathSource::Rectangle {
            size: VectorPoint {
                x: size[0] as f64,
                y: size[1] as f64,
            },
        },
        ops: Vec::new(),
        fill: Some(Fill {
            brush: Brush::Solid(Rgb {
                r: rgba[0] as f64 / 255.0,
                g: rgba[1] as f64 / 255.0,
                b: rgba[2] as f64 / 255.0,
            }),
            rule: FillRule::NonZero,
            opacity: rgba[3] as f64 / 255.0,
            hidden: false,
        }),
        stroke: None,
    });
    ShapeNode::Group(ShapeGroup {
        transform: RepeaterTransform {
            position: VectorPoint {
                x: size[0] as f64 * 0.5,
                y: size[1] as f64 * 0.5,
            },
            ..RepeaterTransform::IDENTITY
        },
        children: vec![leaf],
    })
}

pub const EDIT_TIMELINE: &str = "edit";

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("chunk の組み立てに失敗した: {0}")]
    Chunk(String),
    #[error("store への追加に失敗した: {0}")]
    Ingest(String),
    #[error("track の符号化に失敗した: {0}")]
    Encode(#[from] serde_json::Error),
    #[error("property 名が不正: {0}")]
    Property(String),
    #[error("file の読み書きに失敗した: {0}")]
    Io(String),
}

pub mod property {
    pub const RESERVED: &[&str] = &[
        "meta", "present", "masks", "attrs", "effects", "shapes", "text",
    ];

    pub const MASK_PREFIX: &str = "mask.";

    pub const TEXT_RANGE_PREFIX: &str = "text_range.";

    pub const TEXT_STYLE_PREFIX: &str = "text_style.";

    pub const EFFECT_PREFIX: &str = "effect.";

    pub const ANCHOR: &str = "anchor";
    pub const POSITION: &str = "position";
    pub const POSITION_X: &str = "position.x";
    pub const POSITION_Y: &str = "position.y";
    pub const SCALE: &str = "scale";
    pub const ROTATION: &str = "rotation";
    pub const OPACITY: &str = "opacity";
    pub const SKEW: &str = "skew";
    pub const SKEW_AXIS: &str = "skew_axis";
    pub const LEVEL: &str = "level";
    pub const PAN: &str = "pan";
    pub const FADE_IN: &str = "fade_in";
    pub const FADE_OUT: &str = "fade_out";
    pub const TIME_REMAP: &str = "time_remap";
    pub const SPEED: &str = "speed";
    pub const POSITION_Z: &str = "position.z";
    pub const ROTATION_X: &str = "rotation.x";
    pub const ROTATION_Y: &str = "rotation.y";
    pub const SCALE_Z: &str = "scale.z";

    pub const CAMERA_CENTER: &str = "camera.center";
    pub const CAMERA_ZOOM: &str = "camera.zoom";
    pub const CAMERA_ROLL: &str = "camera.roll";
    /// 注視点の奥行き。XY は `camera.center`。
    pub const CAMERA_TARGET_Z: &str = "camera.target.z";
    /// 注視点のまわりで eye が居る角度(pitch, yaw、度)。rerun の eye と同じ球面座標。
    pub const CAMERA_ORBIT: &str = "camera.orbit";
    /// 注視点までの距離。comp が縦画角 55° に収まる既定距離への倍率。
    pub const CAMERA_DISTANCE: &str = "camera.distance";
    /// 注視する層(AE の Point of Interest に null を親付けする型)。0 は無し。あれば center と target.z より優先。
    pub const CAMERA_TARGET: &str = "camera.target";

    use crate::doc::eval::Value;
    /// Camera 層の欄: (property, label, 既定値, 範囲)。登録・既定・生成時の複写はこの 1 表から。
    pub const CAMERA_ROWS: &[(&str, &str, Value, Option<(f64, f64)>)] = &[
        (CAMERA_CENTER, "Center", Value::Vec2([0.0, 0.0]), None),
        (CAMERA_TARGET_Z, "Target Z", Value::F64(0.0), None),
        (CAMERA_TARGET, "Target", Value::LayerId(0), None),
        (CAMERA_ORBIT, "Orbit", Value::Vec2([0.0, 0.0]), None),
        (CAMERA_DISTANCE, "Distance", Value::F64(1.0), Some((0.01, 100.0))),
        (CAMERA_ZOOM, "Zoom", Value::F64(1.0), Some((0.01, 100.0))),
        (CAMERA_ROLL, "Roll", Value::F64(0.0), None),
    ];

    /// 解決済みカメラを Camera 層の欄の値へ戻す(層ターゲットは含まない)。
    pub fn camera_values(camera: &crate::doc::core::ResolvedCamera) -> [(&'static str, Value); 6] {
        [
            (CAMERA_CENTER, Value::Vec2(camera.center.map(f64::from))),
            (CAMERA_TARGET_Z, Value::F64(camera.target_z as f64)),
            (CAMERA_ORBIT, Value::Vec2(camera.orbit_degrees.map(f64::from))),
            (CAMERA_DISTANCE, Value::F64(camera.distance_scale as f64)),
            (CAMERA_ZOOM, Value::F64(camera.zoom as f64)),
            (CAMERA_ROLL, Value::F64(camera.roll_degrees as f64)),
        ]
    }

    /// Stage 層: 出力枠の外側にどれだけ作業範囲を広げるか(左・上・右・下、comp px)。
    pub const STAGE_MARGINS: [&str; 4] = ["stage.left", "stage.top", "stage.right", "stage.bottom"];
}

/// その時刻に効いている作業範囲。`layer` が無ければ出力枠そのもの。
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct StageExtent {
    pub layer: Option<LayerId>,
    pub margins: [f32; 4],
}

impl StageExtent {
    /// comp 座標の [x, y, w, h]。
    pub fn rect(&self, comp: crate::doc::core::CompSpec) -> [f32; 4] {
        let [l, t, r, b] = self.margins;
        [-l, -t, comp.width as f32 + l + r, comp.height as f32 + t + b]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LayerSource {
    #[serde(alias = "Media", alias = "PointCloud")]
    File {
        path: String,
        fingerprint: Option<String>,
    },
    Null,
    Camera,
    Stage,
    Shape,
    Text,
    Group,
}

impl LayerSource {
    pub fn declared_size(&self) -> Option<[f32; 2]> {
        match self {
            Self::File { .. }
            | Self::Null
            | Self::Camera
            | Self::Stage
            | Self::Shape
            | Self::Text
            | Self::Group => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Composition {
    pub width: u32,
    pub height: u32,
    pub fps: crate::doc::core::Fps,
    pub duration_frames: i64,
    #[serde(default = "Composition::default_background")]
    pub background: [f32; 4],
}

impl Composition {
    pub fn default_background() -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    pub fn spec(&self) -> crate::doc::core::CompSpec {
        crate::doc::core::CompSpec {
            width: self.width,
            height: self.height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerTiming {
    pub start: i64,
    pub duration: i64,
    pub source_in: i64,
    pub speed: Speed,
}

impl Default for LayerTiming {
    fn default() -> Self {
        Self {
            start: 0,
            duration: 0,
            source_in: 0,
            speed: Speed::NORMAL,
        }
    }
}

impl LayerTiming {
    pub fn covers(&self, comp_frame: i64) -> bool {
        comp_frame >= self.start && comp_frame < self.start + self.duration
    }

    /// 層を置く。尺は壁ではない(裁定 2026-09-07): 素材の尺があればその長さ、無ければ
    /// 「尺の終わりまで」を既定にするが、尺の先に置いた時も 0 にはせず尺と同じ長さにする。
    pub fn place(start: i64, source_frames: Option<i64>, comp_duration: i64) -> Self {
        let remaining = comp_duration - start;
        let duration = match source_frames {
            Some(frames) => frames.max(1),
            None if remaining > 0 => remaining,
            None => comp_duration.max(1),
        };
        Self {
            start,
            duration,
            source_in: 0,
            speed: Speed::NORMAL,
        }
    }

    pub fn source_frame(&self, comp_frame: i64) -> Option<i64> {
        self.covers(comp_frame).then(|| {
            let offset = comp_frame - self.start;
            self.source_in + self.speed.scale_frame_offset(offset)
        })
    }

    pub fn source_frame_with_speed_track(
        &self,
        comp_frame: i64,
        track: &crate::doc::store::KeyframeTrack,
        fps: crate::doc::core::Fps,
    ) -> Result<Option<i64>, StoreError> {
        if !self.covers(comp_frame) {
            return Ok(None);
        }
        let accumulated = accumulate_speed_offset(track, fps, self.start, comp_frame)?;
        Ok(Some(self.source_in + accumulated))
    }
}

fn accumulate_speed_offset(
    track: &crate::doc::store::KeyframeTrack,
    fps: crate::doc::core::Fps,
    start: i64,
    comp_frame: i64,
) -> Result<i64, StoreError> {
    use crate::doc::store::{Interp, RationalTime};

    if comp_frame <= start {
        return Ok(0);
    }
    let keys = track.keys();
    if keys.is_empty() {
        return Ok(0);
    }

    let frame_time = |f: i64| -> Result<RationalTime, StoreError> {
        RationalTime::try_from_frame(f, fps).map_err(|e| StoreError::Property(e.to_string()))
    };
    let ceil_frame = |t: RationalTime| -> Result<i64, StoreError> {
        let f = t
            .try_to_frame_floor(fps)
            .map_err(|e| StoreError::Property(e.to_string()))?;
        let recon = frame_time(f)?;
        Ok(if recon < t { f + 1 } else { f })
    };
    let value_f64 = |v: &crate::doc::store::Value| -> Result<f64, StoreError> {
        match v {
            crate::doc::store::Value::F64(x) => Ok(*x),
            other => Err(StoreError::Property(format!(
                "{} に数値でない値が入っている: {other:?}",
                crate::doc::store::property::SPEED
            ))),
        }
    };

    let mut total = 0.0f64;
    let n = keys.len();

    let mut add_hold = |lo: i64, hi: i64, v: f64| {
        let lo = lo.max(start);
        let hi = hi.min(comp_frame);
        if hi > lo {
            total += v * (hi - lo) as f64;
        }
    };

    {
        let hi = ceil_frame(keys[0].t)?;
        add_hold(start, hi, value_f64(&keys[0].value)?);
    }
    {
        let lo = ceil_frame(keys[n - 1].t)?;
        add_hold(lo, comp_frame, value_f64(&keys[n - 1].value)?);
    }
    for i in 0..n.saturating_sub(1) {
        let (a, b) = (&keys[i], &keys[i + 1]);
        let lo = ceil_frame(a.t)?.max(start);
        let hi = ceil_frame(b.t)?.min(comp_frame);
        if hi <= lo {
            continue;
        }
        match a.interp {
            Interp::Hold => {
                total += value_f64(&a.value)? * (hi - lo) as f64;
            }
            Interp::Linear => {
                let va = value_f64(&a.value)?;
                let vb = value_f64(&b.value)?;
                let u = |f: i64| -> Result<f64, StoreError> {
                    let num = seconds_since(frame_time(f)?, a.t);
                    let den = seconds_since(b.t, a.t);
                    Ok(if den == 0.0 { 0.0 } else { num / den })
                };
                let u_lo = u(lo)?;
                let u_hi_last = u(hi - 1)?;
                let count = (hi - lo) as f64;
                let sum_u = count * (u_lo + u_hi_last) / 2.0;
                total += va * count + (vb - va) * sum_u;
            }
            other @ (Interp::Bezier { .. }
            | Interp::Bounce { .. }
            | Interp::Elastic { .. }
            | Interp::Cyclic { .. }
            | Interp::Random { .. }
            | Interp::Steps { .. }
            | Interp::ElasticSteps { .. }) => {
                return Err(StoreError::Property(format!(
                    "{} の {} 補間区間は積算未対応(発注の検収条件は Hold のみ、\
                     黙って近似しない — 対応するなら別発注で判断すること)",
                    crate::doc::store::property::SPEED,
                    other.kind()
                )));
            }
        }
    }

    Ok(total.floor() as i64)
}

fn seconds_since(t: crate::doc::core::RationalTime, origin: crate::doc::core::RationalTime) -> f64 {
    match t.try_sub(origin) {
        Ok(rel) => rel.as_seconds_f64(),
        Err(_) => t.as_seconds_f64() - origin.as_seconds_f64(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Speed {
    num: i64,
    den: i64,
}

impl Speed {
    pub const NORMAL: Speed = Speed { num: 1, den: 1 };

    pub fn try_new(num: i64, den: i64) -> Result<Self, StoreError> {
        if den <= 0 {
            return Err(StoreError::Property(
                "speed の分母は正でなければならない".to_owned(),
            ));
        }
        Ok(Self { num, den })
    }

    pub const fn num(self) -> i64 {
        self.num
    }

    pub const fn den(self) -> i64 {
        self.den
    }

    fn scale_frame_offset(self, offset: i64) -> i64 {
        let num = offset as i128 * self.num as i128;
        let den = self.den as i128;
        (num.div_euclid(den)) as i64
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerMeta {
    pub source: LayerSource,
    pub order: i16,
    pub timing: LayerTiming,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedLayer {
    pub id: LayerId,
    pub source: LayerSource,
    pub placement: LayerPlacement,
    pub declared_size: [f32; 2],
    pub source_frame: i64,
    pub source_time: RationalTime,
    pub masks: Vec<ResolvedMask>,
    pub effects: Vec<ResolvedEffect>,
    pub blend_mode: BlendMode,
    pub matte: Option<Matte>,
    pub clip_to_below: bool,
    pub projection: LayerProjection,
    /// 3D の素材を平面へ収めるか。既定は収めない(裁定 2026-08-30)。
    pub flatten: bool,
    pub environment: bool,
    /// ゴースト(同じ層を遅れて見た姿)なら true。掴めない・枠に入らない(裁定 2026-09-07)。
    pub ghost: bool,
    /// 配置効果が増やした何番目か。増やしていなければ 0。
    pub copy: u32,
    /// 配置効果より**下**に積まれた効果。配置を 1 枚に合わせてから掛かる。
    pub after_effects: Vec<ResolvedEffect>,
}

/// 白紙。**枠だけは要る** —— 枠が無いと何も描けず、窓が空を出す。
/// 大きさは既定の 1920x1080 30fps 60秒。
pub fn blank_project() -> Document {
    use crate::doc::store::{Composition, Document, Fps, Intent};
    let mut doc = Document::new();
    let comp = Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30, 1).expect("30fps"),
        duration_frames: 1800,
        background: [0.0, 0.0, 0.0, 1.0],
    };
    let _ = doc.apply(Intent::SetComposition(comp));
    doc
}

pub mod text_edit;
