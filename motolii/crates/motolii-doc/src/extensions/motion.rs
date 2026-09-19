//! 層の動きのぼけ — 効果で、取っ手の家は Inspector(2026-09-14 利用者裁定、AE の comp 設定とスイッチ列は作らない)。
//! 先例は Alight Motion の Motion Blur: Tune(1 = 隣のコマとの差ぶん)と Position / Scale / Angle の on/off。
//! 1 コマの中のずらした時刻で**位置・大きさ・角度だけ**を取り直した写しを作り、足して平均する。色などはぼかさない。

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;
use crate::doc::store::kind::{Param, ParamKind};

pub struct MotionKind {
    pub plugin_id: &'static str,
    pub label: &'static str,
    pub params: &'static [Param],
}

pub const MOTION_BLUR: &str = "motolii.motion_blur";
pub const SWITCH: &[&str] = &["Off", "On"];

const fn switch(name: &'static str, label: &'static str) -> Param {
    Param { name, label, section: "", kind: ParamKind::Choice(SWITCH), default: [1.0, 0.0], range: Some((0.0, 1.0)), modes: None }
}

pub const KINDS: &[MotionKind] = &[MotionKind {
    plugin_id: MOTION_BLUR,
    label: "Motion Blur",
    params: &[
        Param::number("tune", "Tune", 1.0, Some((0.0, 4.0))),
        switch("position", "Position"),
        switch("scale", "Scale"),
        switch("angle", "Angle"),
    ],
}];

pub fn is_motion_blur(plugin_id: &str) -> bool {
    plugin_id == MOTION_BLUR
}

/// 取っ手の値: (tune, [位置, 大きさ, 角度])。無い欄は宣言の既定。
pub fn settings(params: &[(String, Value)]) -> (f64, [bool; 3]) {
    let get = |name: &str| {
        let default = KINDS[0].params.iter().find(|p| p.name == name).map_or(0.0, |p| p.default[0]);
        match params.iter().find(|(n, _)| n == name).map(|(_, v)| v) {
            Some(Value::F64(v)) if v.is_finite() => *v,
            _ => default,
        }
    };
    (get("tune").clamp(0.0, 4.0), [get("position") >= 0.5, get("scale") >= 0.5, get("angle") >= 0.5])
}

/// 写しを取る時刻: t を中心に、幅 `tune` コマを `count` 等分した各区間の真ん中。
pub fn sample_times(t: RationalTime, frame_seconds: f64, tune: f64, count: u32) -> Vec<RationalTime> {
    const DEN: i64 = 1_000_000;
    (0..count)
        .filter_map(|k| {
            let s = (f64::from(k) + 0.5) / f64::from(count) - 0.5;
            let offset = RationalTime::try_new((s * tune * frame_seconds * DEN as f64).round() as i64, DEN).ok()?;
            t.try_add(offset).ok()
        })
        .collect()
}

/// 写しの枚数: 1 コマの間に層の縁が動く道のり(px)から。1.5 px ごとに 1 枚、止まっていれば 1 枚(ぼかさない)。
pub fn sample_count(travel_px: f32) -> u32 {
    if travel_px < 0.5 { 1 } else { (travel_px / 1.5).ceil().clamp(2.0, 64.0) as u32 }
}

pub struct Sampling {
    pub channels: [bool; 3],
    edges: [RationalTime; 2],
    time: RationalTime,
    frame_seconds: f64,
    tune: f64,
}

pub fn sampling(params: &[(String, Value)], time: RationalTime, frame_seconds: f64) -> Option<Sampling> {
    let (tune, channels) = settings(params);
    if tune <= 0.0 || frame_seconds <= 0.0 || !channels.contains(&true) { return None; }
    let edges = [-0.5, 0.5].map(|s| {
        RationalTime::try_new((s * tune * frame_seconds * 1_000_000.0).round() as i64, 1_000_000).ok().and_then(|o| time.try_add(o).ok())
    });
    let [Some(early), Some(late)] = edges else { return None; };
    Some(Sampling { channels, edges: [early, late], time, frame_seconds, tune })
}

impl Sampling {
    pub fn deltas<E>(
        &self,
        size: [f32; 2],
        transform: glam::Affine2,
        parent: glam::Affine2,
        mut read_delta: impl FnMut(RationalTime) -> Result<glam::Affine2, E>,
    ) -> Result<impl ExactSizeIterator<Item = Result<glam::Affine2, E>>, E> {
        let inverse = parent.inverse();
        let early = parent * read_delta(self.edges[0])? * inverse;
        let late = parent * read_delta(self.edges[1])? * inverse;
        // 未確定の素材寸法は原点の周り200pxで回転の道のりを見積もる。
        let [w, h] = size;
        let (lo, hi) = if w > 0.0 && h > 0.0 { ([0.0, 0.0], [w, h]) } else { ([-100.0, -100.0], [100.0, 100.0]) };
        let travel = [[lo[0], lo[1]], [hi[0], lo[1]], [lo[0], hi[1]], [hi[0], hi[1]]]
            .map(|corner| {
                let p = transform.transform_point2(glam::Vec2::from(corner));
                early.transform_point2(p).distance(late.transform_point2(p))
            }).into_iter().fold(0.0f32, f32::max);
        let count = sample_count(travel);
        let times = if count <= 1 { Vec::new() } else { sample_times(self.time, self.frame_seconds, self.tune, count) };
        Ok(times.into_iter().map(read_delta))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampling_policy_needs_only_transform_observations() {
        assert!(sampling(&[("tune".into(), Value::F64(0.0))], RationalTime::ZERO, 1.0/30.0).is_none());
        let plan = sampling(&[], RationalTime::ZERO, 1.0/30.0).unwrap();
        assert_eq!(plan.channels, [true; 3]);
        let samples = plan.deltas([10.0; 2], glam::Affine2::IDENTITY, glam::Affine2::IDENTITY,
            |at| Ok::<_, ()>(glam::Affine2::from_translation(glam::vec2(at.as_seconds_f64() as f32 * 100.0, 0.0)))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(samples.len(), 3);
        assert!(samples[1].translation.x.abs() < 0.001);
        assert!((samples[0].translation.x + samples[2].translation.x).abs() < 0.001);
        let still = plan.deltas([0.0; 2], glam::Affine2::IDENTITY, glam::Affine2::IDENTITY, |_| Ok::<_, ()>(glam::Affine2::IDENTITY)).unwrap();
        assert_eq!(still.len(), 0);
    }
}
