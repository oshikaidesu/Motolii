use serde::{Deserialize, Serialize};

use motolii_core::RationalTime;

use crate::bezier::{cubic_bezier_ease, sample, solve_curve_x};
use crate::value::Value;

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum TrackError {
    #[error("Bezier control point x1/x2 must be in [0,1], got x1={x1} x2={x2}")]
    InvalidBezier { x1: f64, x2: f64 },
    #[error(
        "Bezier split is unrepresentable: progress={progress}, x1={x1}, y1={y1}, x2={x2}, y2={y2}"
    )]
    UnrepresentableBezierSplit {
        progress: f64,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        curve_parameter: Option<f64>,
        split_value: Option<f64>,
        left_x_denominator: f64,
        right_x_denominator: f64,
        left_y_denominator: Option<f64>,
        right_y_denominator: Option<f64>,
    },
    #[error("keyframes must be sorted by strictly increasing time without duplicates")]
    UnsortedOrDuplicateKeys,
    /// パラメトリック補間型のパラメータが定義域の外(`Bezier` は専用の
    /// [`TrackError::InvalidBezier`] が既にある)。
    #[error("補間パラメータが定義域の外: {0}")]
    InvalidInterp(String),
    /// 区間の分割(キーの割り込み)が同型2本で表せない補間型。
    /// **Bezier だけが de Casteljau で割れる** — Bounce/Elastic/Steps は
    /// 「半分のバウンス」が同じ族の中に居ないので、近似で埋めずに断る
    /// (`motolii-store` の速度積算が Bezier 区間で `Err` を返すのと同じ規律)。
    #[error("{kind} 区間は分割できない — u→値の純関数を同型2本へ割る規則が無い")]
    UnsplittableInterp { kind: &'static str },
}

/// キーフレーム区間(このキーから次のキーまで)の補間方法。
///
/// **どの variant も「区間の正規化位置 u∈[0,1] → 進み具合」の純関数**
/// ([`Interp::ease`])であって、fps・解像度・区間の実長のどれにも依存しない。
/// だから CSS の `cubic-bezier()`・Flow・Alight Motion と同じ表現になり、
/// UI はこの数値を編集するだけで済む。
///
/// `Bounce` / `Elastic` / `Steps` は **AE が式(`valueAtTime` の物理シミュ)を
/// 要求した領域を、GUI の選択肢へ畳んだ物**(2026-07-10 決定、`docs/concept.md`、
/// 先例 Alight Motion)。逐次状態を持たない閉形式なので、`render_frame(t)` が
/// 純関数であるという約束(スクラブ・区間キャッシュ・並列書き出しの全部が
/// 乗っている約束)を崩さない。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Interp {
    /// 次のキーまで値を保持
    Hold,
    Linear,
    /// cubic-bezier(x1,y1,x2,y2)イージング(x1,x2∈[0,1])
    Bezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    /// 解析反射のバウンス。`bounces` = 着地後に跳ね返る回数、
    /// `decay`∈[0,1] = 1回ごとに残る跳ね上がりの高さの比。
    /// 値は [0,1] に収まる(行き過ぎない — 跳ねるのは戻る側)。
    Bounce { bounces: u32, decay: f64 },
    /// 減衰正弦のバネ。`amplitude` = 最初の行き過ぎの強さ(1 で行き過ぎ最小)、
    /// `period` = 揺れの周期。**y は [0,1] の外へ出る**(オーバーシュート)。
    Elastic { amplitude: f64, period: f64 },
    /// 段階移動。`count` = 段数。
    Steps { count: u32 },
}

impl Interp {
    /// [`Interp::Bounce`] の跳ね回数の上限。[`Interp::ease`] の走査を有界に
    /// するための構造的な制限であって好みではない([`KeyframeTrack::validate`]
    /// はこれを超える値を拒む)。
    pub const MAX_BOUNCES: u32 = 32;

    /// 区間の正規化位置 `u`∈[0,1] → **正規化された進み具合**。
    ///
    /// **イージングの唯一の家**。[`KeyframeTrack::eval`] も、front の曲線
    /// プレビューもここを呼ぶ — 同じ規則の家を2つ作らないため(front が
    /// 曲線を描き直すと、見えている絵と評価される動きが黙ってずれる)。
    ///
    /// `x1`/`x2` は [`crate::bezier::cubic_bezier_ease`] の定義域
    /// ([0,1])へ丸めてから渡す。[`KeyframeTrack::validate`] が既にこの範囲を
    /// 強制しているので track 経由では起きないが、**編集中の値**(まだ track に
    /// 入っていない、UI が握っている途中の4値)がそのまま来る口でもあるため。
    pub fn ease(&self, u: f64) -> f64 {
        match *self {
            Interp::Hold => 0.0,
            Interp::Linear => u.clamp(0.0, 1.0),
            Interp::Bezier { x1, y1, x2, y2 } => {
                cubic_bezier_ease(x1.clamp(0.0, 1.0), y1, x2.clamp(0.0, 1.0), y2, u)
            }
            Interp::Bounce { bounces, decay } => bounce_ease(bounces, decay, u),
            Interp::Elastic { amplitude, period } => elastic_ease(amplitude, period, u),
            Interp::Steps { count } => steps_ease(count, u),
        }
    }

    /// エラー文と UI の見出しが同じ語を使うための名前。
    pub fn kind(&self) -> &'static str {
        match self {
            Interp::Hold => "Hold",
            Interp::Linear => "Linear",
            Interp::Bezier { .. } => "Bezier",
            Interp::Bounce { .. } => "Bounce",
            Interp::Elastic { .. } => "Elastic",
            Interp::Steps { .. } => "Steps",
        }
    }

    pub fn split_at(&self, progress: f64) -> Result<(Interp, Interp), TrackError> {
        // パラメトリック型は progress によらず割れない。progress の検査より先に
        // 断る — 「0.5 なら割れるかもしれない」と読める余地を残さない。
        if matches!(
            self,
            Interp::Bounce { .. } | Interp::Elastic { .. } | Interp::Steps { .. }
        ) {
            return Err(TrackError::UnsplittableInterp { kind: self.kind() });
        }
        if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
            if let Interp::Bezier { x1, y1, x2, y2 } = *self {
                return Err(TrackError::UnrepresentableBezierSplit {
                    progress,
                    x1,
                    y1,
                    x2,
                    y2,
                    curve_parameter: None,
                    split_value: None,
                    left_x_denominator: progress,
                    right_x_denominator: 1.0 - progress,
                    left_y_denominator: None,
                    right_y_denominator: None,
                });
            }
            return Err(TrackError::UnrepresentableBezierSplit {
                progress,
                x1: 0.0,
                y1: 0.0,
                x2: 0.0,
                y2: 0.0,
                curve_parameter: None,
                split_value: None,
                left_x_denominator: progress,
                right_x_denominator: 1.0 - progress,
                left_y_denominator: None,
                right_y_denominator: None,
            });
        }

        match *self {
            Interp::Hold => Ok((Interp::Hold, Interp::Hold)),
            Interp::Linear => Ok((Interp::Linear, Interp::Linear)),
            Interp::Bounce { .. } | Interp::Elastic { .. } | Interp::Steps { .. } => {
                Err(TrackError::UnsplittableInterp { kind: self.kind() })
            }
            Interp::Bezier { x1, y1, x2, y2 } => {
                let s = solve_curve_x(x1, x2, progress);
                if !s.is_finite() || !(0.0..=1.0).contains(&s) {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: None,
                        left_x_denominator: progress,
                        right_x_denominator: 1.0 - progress,
                        left_y_denominator: None,
                        right_y_denominator: None,
                    });
                }

                let v = sample(y1, y2, s);
                if !v.is_finite() {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: progress,
                        right_x_denominator: 1.0 - progress,
                        left_y_denominator: None,
                        right_y_denominator: None,
                    });
                }

                let inv = 1.0 - s;
                let left_x_denom = progress;
                let right_x_denom = 1.0 - progress;
                let left_y_denom = v;
                let right_y_denom = 1.0 - v;

                if !left_x_denom.is_finite()
                    || !right_x_denom.is_finite()
                    || !left_y_denom.is_finite()
                    || !right_y_denom.is_finite()
                    || left_x_denom == 0.0
                    || right_x_denom == 0.0
                    || left_y_denom == 0.0
                    || right_y_denom == 0.0
                {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: left_x_denom,
                        right_x_denominator: right_x_denom,
                        left_y_denominator: Some(left_y_denom),
                        right_y_denominator: Some(right_y_denom),
                    });
                }

                let l1 = (x1 * s, y1 * s);
                let l2 = (x1 * inv + x2 * s, y1 * inv + y2 * s);
                let l3 = (x2 * inv + s, y2 * inv + s);
                let l12 = (l1.0 * inv + l2.0 * s, l1.1 * inv + l2.1 * s);
                let l23 = (l2.0 * inv + l3.0 * s, l2.1 * inv + l3.1 * s);

                let left = Interp::Bezier {
                    x1: l1.0 / left_x_denom,
                    y1: l1.1 / left_y_denom,
                    x2: l12.0 / left_x_denom,
                    y2: l12.1 / left_y_denom,
                };
                let right = Interp::Bezier {
                    x1: (l23.0 - progress) / right_x_denom,
                    y1: (l23.1 - v) / right_y_denom,
                    x2: (l3.0 - progress) / right_x_denom,
                    y2: (l3.1 - v) / right_y_denom,
                };

                if !is_valid_bezier_control(left) || !is_valid_bezier_control(right) {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: left_x_denom,
                        right_x_denominator: right_x_denom,
                        left_y_denominator: Some(left_y_denom),
                        right_y_denominator: Some(right_y_denom),
                    });
                }

                Ok((left, right))
            }
        }
    }
}

/// 解析反射のバウンス。**逐次積分をしない**(2026-07-10「馬鹿正直に
/// シミュレートしない」)— 自由落下の閉形式をそのまま使う: 跳ね上がる高さは
/// 1回ごとに `decay` 倍、その滞空時間は `sqrt(decay)` 倍(h ∝ t² だから)。
/// 前フレームの状態を持たないので `u` の純関数のままで、スクラブしても
/// 逆再生しても同じ絵になる。
///
/// 形: `[0, 1)` の落下(加速)で 1 に着地し、以後 `bounces` 回、幅と高さが
/// 幾何級数で縮む放物線で 1 へ戻る。値は [0,1] を出ない。
fn bounce_ease(bounces: u32, decay: f64, u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let decay = if decay.is_finite() {
        decay.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let n = bounces.min(Interp::MAX_BOUNCES);
    let ratio = decay.sqrt();

    // 区間幅は 落下=1、k 回目の跳ね返り=ratio^k。合計で割って u を「区間何本目の
    // どこか」へ戻す(区間の実長には触れない — ここは正規化の中だけの話)。
    let mut total = 1.0;
    let mut width = 1.0;
    for _ in 0..n {
        width *= ratio;
        total += width;
    }
    let x = u * total;
    if x < 1.0 {
        // 落下。加速して 1(=着地)へ。
        return x * x;
    }

    let mut start = 1.0;
    let mut width = 1.0;
    let mut height = 1.0;
    for _ in 0..n {
        width *= ratio;
        height *= decay;
        if width <= 0.0 {
            break;
        }
        if x < start + width {
            let local = (x - start) / width;
            // 両端 1(接地)、中央 1-height(跳ねた高さ)の放物線。
            return 1.0 - height * 4.0 * local * (1.0 - local);
        }
        start += width;
    }
    1.0
}

/// 減衰正弦のバネ。Penner の `easeOutElastic` — CSS には無いが Flow /
/// Alight Motion / 主要トゥイーンライブラリが共通で使っている式で、ここでも
/// 数学は借りる(自作しない)。
///
/// `amplitude` が 1 未満の時は Penner の実装どおり 1 として扱う: `y(0)=0` を
/// 満たす位相 `s = period/2π · asin(1/amplitude)` は振幅 1 未満では実数に
/// ならないので、この帯は表現できない(UI 側の下限も 1)。
fn elastic_ease(amplitude: f64, period: f64, u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let period = if period.is_finite() && period > 0.0 {
        period
    } else {
        0.3
    };
    let (amplitude, phase) = if !amplitude.is_finite() || amplitude.abs() < 1.0 {
        (1.0, period / 4.0)
    } else {
        (
            amplitude,
            period / std::f64::consts::TAU * (1.0 / amplitude).asin(),
        )
    };
    amplitude * (-10.0 * u).exp2() * ((u - phase) * std::f64::consts::TAU / period).sin() + 1.0
}

/// 段階移動。CSS `steps(count, jump-end)` と同じ意味 — 各段の頭の値を保ち、
/// 段の終わりで飛ぶ。`count == 1` は区間の頭を保って端で飛ぶ形になり、
/// [`Interp::Hold`] と同じ絵になる(別の名前で同じ物を持たないための注記で
/// あって、Hold を置き換えるものではない — Hold は Lottie の `h:1`)。
fn steps_ease(count: u32, u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let count = count.max(1) as f64;
    (u * count).floor() / count
}

fn is_valid_bezier_control(interp: Interp) -> bool {
    if let Interp::Bezier { x1, y1, x2, y2 } = interp {
        [x1, y1, x2, y2].iter().all(|v| v.is_finite())
            && (0.0..=1.0).contains(&x1)
            && (0.0..=1.0).contains(&x2)
    } else {
        false
    }
}

/// 空間ベジェの接線(`position-keyframe ti`/`to`)。**Vec2 の position 系 track だけが
/// 意味を持つ** — position が Vec2 単一 property なので入る余地ができた(裁定61 の
/// 見込みどおり)。このキー自身の値からの相対オフセットという規約は
/// [`crate::value::PathVertex`] の `in_tangent`/`out_tangent` とそろえてある(新しい
/// 規約を作らない)。
///
/// **モーションパスの形**(位置が辿る曲線)を決めるだけで、**速さ**
/// ([`Keyframe::interp`] のイージング)とは独立(Lottie/AE と同じ分離 — `ti`/`to` は
/// 空間、`i`/`o` は時間)。描画は不要(地図の note どおり)なので、ここでは
/// [`KeyframeTrack::eval`] が返す `Value::Vec2` の値そのものが曲線に沿うことだけを保証する。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialTangent {
    /// `to`(Value Out Tangent)。次のキーへ向かう側の接線。
    pub out_tangent: [f64; 2],
    /// `ti`(Value In Tangent)。前のキーから来る側の接線。
    pub in_tangent: [f64; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub t: RationalTime,
    pub value: Value,
    pub interp: Interp,
    /// 空間ベジェの接線。`None` = 直線(補間は今までどおり [`Value::lerp`])。
    /// 補間の**速さ**は `interp` が別に決める(上記 [`SpatialTangent`] のドキュメント参照)。
    /// **`None` の時は JSON に出さない**(`skip_serializing_if`) — position 以外の
    /// 大半の track はこのフィールドを一切使わないので、書かないと R2 の予算試験
    /// (track を生で読む投影コスト)を巻き込まずに済む。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial: Option<SpatialTangent>,
}

/// 時刻順にソートされたキーフレーム列。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "KeyframeTrackDe")]
pub struct KeyframeTrack {
    keys: Vec<Keyframe>,
}

#[derive(Deserialize)]
struct KeyframeTrackDe {
    keys: Vec<Keyframe>,
}

impl TryFrom<KeyframeTrackDe> for KeyframeTrack {
    type Error = TrackError;

    fn try_from(value: KeyframeTrackDe) -> Result<Self, Self::Error> {
        let track = Self { keys: value.keys };
        track.validate()?;
        Ok(track)
    }
}

impl KeyframeTrack {
    pub fn new() -> Self {
        Self::default()
    }

    /// 生のキー列から検証込みで組む。`#[serde(try_from = "KeyframeTrackDe")]` が
    /// 内部で行うのと同じ経路——`motolii-store::slot::PropertySource` の
    /// カスタム `Deserialize`(裁定213 の性能修正、2026-08-23)が、`untagged` の
    /// `Content` バッファリングを避けて `"keys"` フィールドを直接
    /// `Vec<Keyframe>` として読んだ後、ここを呼んで検証込みで組み立て直す
    /// ために公開した。
    pub fn try_from_keys(keys: Vec<Keyframe>) -> Result<Self, TrackError> {
        let track = Self { keys };
        track.validate()?;
        Ok(track)
    }

    /// キーを挿入する。同時刻のキーが既にあれば置き換える。
    pub fn insert(&mut self, key: Keyframe) {
        match self.keys.binary_search_by(|k| k.t.cmp(&key.t)) {
            Ok(i) => self.keys[i] = key,
            Err(i) => self.keys.insert(i, key),
        }
    }

    pub fn keys(&self) -> &[Keyframe] {
        &self.keys
    }

    pub fn validate(&self) -> Result<(), TrackError> {
        for window in self.keys.windows(2) {
            if window[0].t >= window[1].t {
                return Err(TrackError::UnsortedOrDuplicateKeys);
            }
        }
        for key in &self.keys {
            match key.interp {
                Interp::Bezier { x1, y1, x2, y2 } => {
                    // y1/y2 も有限必須(x は範囲検査で NaN を弾けるが y は素通しだった — D1h)
                    if ![x1, y1, x2, y2].iter().all(|v| v.is_finite()) {
                        return Err(TrackError::InvalidBezier { x1, x2 });
                    }
                    if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
                        return Err(TrackError::InvalidBezier { x1, x2 });
                    }
                }
                Interp::Bounce { bounces, decay } => {
                    if !decay.is_finite()
                        || !(0.0..=1.0).contains(&decay)
                        || bounces > Interp::MAX_BOUNCES
                    {
                        return Err(TrackError::InvalidInterp(format!(
                            "Bounce{{bounces: {bounces}, decay: {decay}}} — decay は [0,1]、\
                             bounces は {} 以下",
                            Interp::MAX_BOUNCES
                        )));
                    }
                }
                Interp::Elastic { amplitude, period } => {
                    if !amplitude.is_finite() || !period.is_finite() || period <= 0.0 {
                        return Err(TrackError::InvalidInterp(format!(
                            "Elastic{{amplitude: {amplitude}, period: {period}}} — period は正、\
                             どちらも有限"
                        )));
                    }
                }
                Interp::Steps { count } => {
                    if count == 0 {
                        return Err(TrackError::InvalidInterp(
                            "Steps{count: 0} — 段数は 1 以上".to_owned(),
                        ));
                    }
                }
                Interp::Hold | Interp::Linear => {}
            }
        }
        Ok(())
    }

    /// 時刻tでの値。範囲外は端の値でクランプ。キーが無い場合はF64(0.0)。
    pub fn eval(&self, t: RationalTime) -> Value {
        let keys = &self.keys;
        if keys.is_empty() {
            return Value::F64(0.0);
        }
        if t <= keys[0].t {
            return keys[0].value.clone();
        }
        let last = keys.len() - 1;
        if t >= keys[last].t {
            return keys[last].value.clone();
        }
        // keys[i].t <= t < keys[i+1].t となるiを探す
        let i = match keys.binary_search_by(|k| k.t.cmp(&t)) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let (a, b) = (&keys[i], &keys[i + 1]);
        // Hold だけは `u` を計算しない。`Interp::ease` は 0.0 を返すので値としては
        // 同じだが、`interpolate_value` を通すと Vec2 の空間ベジェ経路へ入ってしまう
        // (Lottie の `h:1` は離散的に飛ぶ = 曲線を辿らない、という不変量を守る)。
        if let Interp::Hold = a.interp {
            return a.value.clone();
        }
        let u = a.interp.ease(segment_u(a.t, b.t, t));
        interpolate_value(a, b, u)
    }
}

/// `a` → `b` 区間の値。**空間タンジェントが片方でもあれば空間ベジェを通る**
/// (`u` は呼び手が渡す — 時間の速さ [`Interp`] のイージング後の値で、ここでは
/// 「どの経路を通るか」だけを決める、[`SpatialTangent`] のドキュメント参照)。
/// 両方 `None`、または `Vec2` 同士でない組は今までどおり [`Value::lerp`]。
fn interpolate_value(a: &Keyframe, b: &Keyframe, u: f64) -> Value {
    if let (Value::Vec2(p0), Value::Vec2(p3)) = (&a.value, &b.value) {
        if a.spatial.is_some() || b.spatial.is_some() {
            let p1 = match &a.spatial {
                Some(s) => [p0[0] + s.out_tangent[0], p0[1] + s.out_tangent[1]],
                None => *p0,
            };
            let p2 = match &b.spatial {
                Some(s) => [p3[0] + s.in_tangent[0], p3[1] + s.in_tangent[1]],
                None => *p3,
            };
            return Value::Vec2(cubic_bezier_point(*p0, p1, p2, *p3, u));
        }
    }
    Value::lerp(&a.value, &b.value, u)
}

/// 一般形の3次ベジェ(端点固定の [`crate::bezier::sample`] とは別 —
/// あちらは `y(0)=0, y(1)=1` のイージング専用で、こちらは4制御点そのものを補間する)。
fn cubic_bezier_point(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], u: f64) -> [f64; 2] {
    let inv = 1.0 - u;
    std::array::from_fn(|i| {
        inv * inv * inv * p0[i]
            + 3.0 * inv * inv * u * p1[i]
            + 3.0 * inv * u * u * p2[i]
            + u * u * u * p3[i]
    })
}

/// 区間内正規化位置u ∈ [0,1)。区間端は有理数で厳密に扱い、u自体はf64でよい
/// (uは1フレーム内の補間位置であり、蓄積しないためドリフトしない)。
fn segment_u(a: RationalTime, b: RationalTime, t: RationalTime) -> f64 {
    let den = seconds_since(b, a);
    if den == 0.0 {
        return 0.0;
    }
    seconds_since(t, a) / den
}

/// `t - origin` の秒。差分がi64 RationalTimeに収まれば厳密経路、溢れ時はf64秒差へフォールバック
/// (評価値を0に握り潰さない — M2E-16 P1)。
fn seconds_since(t: RationalTime, origin: RationalTime) -> f64 {
    match t.try_sub(origin) {
        Ok(rel) => rel.as_seconds_f64(),
        Err(_) => t.as_seconds_f64() - origin.as_seconds_f64(),
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::Fps;

    fn key(t: RationalTime, v: f64, interp: Interp) -> Keyframe {
        Keyframe {
            t,
            value: Value::F64(v),
            interp,
            spatial: None,
        }
    }

    #[test]
    fn empty_track_returns_zero() {
        assert_eq!(
            KeyframeTrack::new().eval(RationalTime::ZERO),
            Value::F64(0.0)
        );
    }

    #[test]
    fn clamps_outside_range() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(RationalTime::from_seconds(1), 10.0, Interp::Linear));
        tr.insert(key(RationalTime::from_seconds(2), 20.0, Interp::Linear));
        assert_eq!(tr.eval(RationalTime::ZERO), Value::F64(10.0));
        assert_eq!(tr.eval(RationalTime::from_seconds(5)), Value::F64(20.0));
    }

    #[test]
    fn linear_interpolation_at_rational_times() {
        let mut tr = KeyframeTrack::new();
        let fps = Fps::try_new(30, 1).unwrap();
        tr.insert(key(RationalTime::ZERO, 0.0, Interp::Linear));
        tr.insert(key(
            RationalTime::try_from_frame(30, fps).unwrap(),
            30.0,
            Interp::Linear,
        ));
        // フレーム12(=0.4秒)で値12.0
        let v = tr.eval(RationalTime::try_from_frame(12, fps).unwrap());
        assert!((v.as_f64().unwrap() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn hold_keeps_value_until_next_key() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(RationalTime::ZERO, 1.0, Interp::Hold));
        tr.insert(key(RationalTime::from_seconds(1), 2.0, Interp::Linear));
        assert_eq!(
            tr.eval(RationalTime::try_new(999, 1000).unwrap()),
            Value::F64(1.0)
        );
        assert_eq!(tr.eval(RationalTime::from_seconds(1)), Value::F64(2.0));
    }

    #[test]
    fn bezier_ease_in_out_midpoint() {
        let mut tr = KeyframeTrack::new();
        tr.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(0.0),
            interp: Interp::Bezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
            spatial: None,
        });
        tr.insert(key(RationalTime::from_seconds(2), 100.0, Interp::Linear));
        let mid = tr.eval(RationalTime::from_seconds(1)).as_f64().unwrap();
        assert!((mid - 50.0).abs() < 1e-3);
        // ease-in: 序盤は線形より遅い
        let early = tr
            .eval(RationalTime::try_new(1, 2).unwrap())
            .as_f64()
            .unwrap();
        assert!(early < 25.0);
    }

    #[test]
    fn rejects_unsorted_keys_on_validate() {
        let track = KeyframeTrack {
            keys: vec![
                key(RationalTime::from_seconds(2), 2.0, Interp::Linear),
                key(RationalTime::from_seconds(1), 1.0, Interp::Linear),
            ],
        };
        assert_eq!(track.validate(), Err(TrackError::UnsortedOrDuplicateKeys));
    }

    #[test]
    fn rejects_invalid_bezier_on_validate() {
        let track = KeyframeTrack {
            keys: vec![
                Keyframe {
                    t: RationalTime::ZERO,
                    value: Value::F64(0.0),
                    interp: Interp::Bezier {
                        x1: 1.5,
                        y1: 0.0,
                        x2: 0.5,
                        y2: 1.0,
                    },
                    spatial: None,
                },
                key(RationalTime::from_seconds(1), 1.0, Interp::Linear),
            ],
        };
        assert!(matches!(
            track.validate(),
            Err(TrackError::InvalidBezier { x1, x2 })
            if (x1 - 1.5).abs() < f64::EPSILON && (x2 - 0.5).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn insert_replaces_same_time_key() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(RationalTime::ZERO, 1.0, Interp::Linear));
        tr.insert(key(RationalTime::ZERO, 5.0, Interp::Linear));
        assert_eq!(tr.keys().len(), 1);
        assert_eq!(tr.eval(RationalTime::ZERO), Value::F64(5.0));
    }







    #[test]
    fn keyframe_linear_across_i64_span_does_not_collapse_to_zero() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(
            RationalTime::from_seconds(i64::MIN),
            10.0,
            Interp::Linear,
        ));
        tr.insert(key(
            RationalTime::from_seconds(i64::MAX),
            20.0,
            Interp::Linear,
        ));
        // ゼロ近傍は区間のほぼ中央 → 15付近。差分Overflowを0.0に握り潰さないこと。
        let mid = tr.eval(RationalTime::ZERO).as_f64().unwrap();
        assert!(
            (mid - 15.0).abs() < 1.0,
            "expected ~15 near span midpoint, got {mid}"
        );
        assert_eq!(
            tr.eval(RationalTime::from_seconds(i64::MAX)),
            Value::F64(20.0)
        );
    }

    fn vec2_key(t: RationalTime, v: [f64; 2], spatial: Option<SpatialTangent>) -> Keyframe {
        Keyframe {
            t,
            value: Value::Vec2(v),
            interp: Interp::Linear,
            spatial,
        }
    }

    /// 空間タンジェントが無ければ、position-keyframe 前と同じ直線補間(`Value::lerp`)。
    /// **motion-path(裁定61)を足しても既存の Vec2 track の挙動を変えない**ことの固定。
    #[test]
    fn vec2_without_spatial_tangents_still_lerps_in_a_straight_line() {
        let mut tr = KeyframeTrack::new();
        tr.insert(vec2_key(RationalTime::ZERO, [0.0, 0.0], None));
        tr.insert(vec2_key(RationalTime::from_seconds(1), [100.0, 0.0], None));
        let mid = tr.eval(RationalTime::try_new(1, 2).unwrap());
        assert_eq!(mid, Value::Vec2([50.0, 0.0]));
    }

    /// **空間ベジェの形はタンジェントが決める** — 区間の中点(u=0.5)が直線の中点から
    /// 外れることで、モーションパスが曲がっていることを固定する。
    #[test]
    fn spatial_tangent_bows_the_position_path_off_the_straight_line() {
        let mut tr = KeyframeTrack::new();
        // 出タンジェントを+yへ大きく振る、入タンジェントは無し(0)。
        tr.insert(vec2_key(
            RationalTime::ZERO,
            [0.0, 0.0],
            Some(SpatialTangent {
                out_tangent: [0.0, 100.0],
                in_tangent: [0.0, 0.0],
            }),
        ));
        tr.insert(vec2_key(RationalTime::from_seconds(1), [100.0, 0.0], None));
        let mid = tr.eval(RationalTime::try_new(1, 2).unwrap());
        let Value::Vec2([x, y]) = mid else {
            panic!("Vec2 が返らない");
        };
        // 直線なら y=0 のまま。タンジェントが効いていれば y が持ち上がる。
        assert!(y > 10.0, "空間タンジェントが効いていない: mid={x},{y}");
    }

    /// **速さ(`interp` のイージング)と形(空間タンジェント)は独立**。同じ空間タンジェント
    /// でも、時間イージングを変えれば区間内の「どこまで進んだか」(u)が変わるので、
    /// 曲線上の位置(点)も変わる — が、その点は常に同じ空間曲線の上に乗る。
    #[test]
    fn temporal_easing_moves_along_the_same_spatial_curve() {
        fn curve_with(interp: Interp) -> Value {
            let mut tr = KeyframeTrack::new();
            tr.insert(Keyframe {
                t: RationalTime::ZERO,
                value: Value::Vec2([0.0, 0.0]),
                interp,
                spatial: Some(SpatialTangent {
                    out_tangent: [0.0, 100.0],
                    in_tangent: [0.0, 0.0],
                }),
            });
            tr.insert(vec2_key(RationalTime::from_seconds(1), [100.0, 0.0], None));
            // 中点(u=0.5)は避ける — ease-in-out は対称カーブなので中点だけは
            // 直線と偶然一致する(f(0.5)=0.5)。1/4点なら常にズレる。
            tr.eval(RationalTime::try_new(1, 4).unwrap())
        }

        let linear = curve_with(Interp::Linear);
        let eased = curve_with(Interp::Bezier {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        });
        assert_ne!(
            linear, eased,
            "イージングを変えても同じ点になっている(速さが形と独立していない)"
        );
    }

    /// **窓で見えない不変量なのでここで固定する**(裁定270 の例外 —
    /// 曲線の端点が 0/1 に着いているかは、front のプレビューを目で見ても
    /// 1px 未満の話なので判定できない)。区間の端で値が飛ぶと、キーの上で
    /// 絵がガクッと動く。
    #[test]
    fn every_interp_starts_at_zero_and_ends_at_one() {
        let cases = [
            Interp::Linear,
            Interp::Bezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
            Interp::Bounce {
                bounces: 3,
                decay: 0.5,
            },
            Interp::Elastic {
                amplitude: 1.0,
                period: 0.3,
            },
            Interp::Steps { count: 5 },
        ];
        for interp in cases {
            assert!(
                interp.ease(0.0).abs() < 1e-9,
                "{} が 0 から始まっていない: {}",
                interp.kind(),
                interp.ease(0.0)
            );
            assert!(
                (interp.ease(1.0) - 1.0).abs() < 1e-9,
                "{} が 1 で終わっていない: {}",
                interp.kind(),
                interp.ease(1.0)
            );
        }
    }

    /// バウンスは**区間の継ぎ目で飛ばない**(落下→跳ね返り→跳ね返りの
    /// 幾何級数が、幅と高さの両方でつながっていること)。ここが切れていると
    /// 「跳ねる」ではなく「ワープする」動きになる。
    #[test]
    fn bounce_is_continuous_across_its_segments() {
        let interp = Interp::Bounce {
            bounces: 4,
            decay: 0.4,
        };
        let mut prev = interp.ease(0.0);
        for i in 1..=2000 {
            let y = interp.ease(i as f64 / 2000.0);
            assert!(
                (y - prev).abs() < 0.02,
                "u={} で跳んでいる: {prev} → {y}",
                i as f64 / 2000.0
            );
            assert!(
                (-1e-9..=1.0 + 1e-9).contains(&y),
                "バウンスが [0,1] を出た: {y}"
            );
            prev = y;
        }
    }

    /// バネは **1 を越える**(オーバーシュート)。越えないなら amplitude が
    /// 効いていないということで、Bezier と見分けが付かなくなる。
    #[test]
    fn elastic_overshoots_past_one() {
        let interp = Interp::Elastic {
            amplitude: 1.0,
            period: 0.3,
        };
        let peak = (1..100)
            .map(|i| interp.ease(i as f64 / 100.0))
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(peak > 1.0, "行き過ぎていない: peak={peak}");
    }

    /// 段階移動は段の中で**動かない**。
    #[test]
    fn steps_holds_inside_each_step() {
        let interp = Interp::Steps { count: 4 };
        assert!((interp.ease(0.10) - 0.0).abs() < 1e-12);
        assert!((interp.ease(0.24) - 0.0).abs() < 1e-12);
        assert!((interp.ease(0.26) - 0.25).abs() < 1e-12);
        assert!((interp.ease(0.51) - 0.5).abs() < 1e-12);
    }

    /// 壊れたパラメータは track に入れない(`ease` が黙って丸めるのは
    /// **編集中の値**への防御であって、保存される値への許可ではない)。
    #[test]
    fn validate_rejects_out_of_domain_parametric_interps() {
        for interp in [
            Interp::Bounce {
                bounces: 1,
                decay: 1.5,
            },
            Interp::Elastic {
                amplitude: 1.0,
                period: 0.0,
            },
            Interp::Steps { count: 0 },
        ] {
            let track = KeyframeTrack {
                keys: vec![
                    key(RationalTime::ZERO, 0.0, interp),
                    key(RationalTime::from_seconds(1), 1.0, Interp::Linear),
                ],
            };
            assert!(
                matches!(track.validate(), Err(TrackError::InvalidInterp(_))),
                "{:?} が通ってしまった",
                interp
            );
        }
    }

    /// `spatial` フィールドが無い旧 JSON も読める(`#[serde(default)]`)。
    #[test]
    fn keyframe_without_spatial_field_deserializes_as_none() {
        let json = r#"{"t":{"num":0,"den":1},"value":{"F64":0.0},"interp":"Linear"}"#;
        let key: Keyframe = serde_json::from_str(json).expect("旧形式の JSON が読めない");
        assert_eq!(key.spatial, None);
    }

    /// **Hold は空間タンジェントより優先する。** Lottie の `h:1` キーは離散的に
    /// 飛ぶだけで、`ti`/`to` が(壊れたデータ等で)同居していても曲線を辿らない
    /// ―― [`interpolate_value`] は `u` を受け取って初めて空間ベジェへ分岐するが、
    /// `Interp::Hold` の区間は `u` 自体を計算せず `a.value` をそのまま返す
    /// ([`KeyframeTrack::eval`] の match 節)。この不変量を固定する。
    #[test]
    fn hold_ignores_spatial_tangent_even_if_present() {
        let mut tr = KeyframeTrack::new();
        tr.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::Vec2([0.0, 0.0]),
            interp: Interp::Hold,
            spatial: Some(SpatialTangent {
                out_tangent: [0.0, 1000.0],
                in_tangent: [0.0, 0.0],
            }),
        });
        tr.insert(vec2_key(RationalTime::from_seconds(1), [100.0, 100.0], None));
        let just_before_next = tr.eval(RationalTime::try_new(999, 1000).unwrap());
        assert_eq!(
            just_before_next,
            Value::Vec2([0.0, 0.0]),
            "spatial tangent が Hold を突き破って値を曲げている"
        );
    }

    /// **成分ごとに別イージングは採らない**(地図 `properties/easing-handle/y` の
    /// 裁定)という決定を、Vec2 以外の多成分値(`Value::Color`、4成分)でも固定する。
    /// 全チャンネルへ同じ `u`(=`cubic_bezier_ease` が1回だけ計算した値)が
    /// 一律に効くので、どのチャンネルも「差分 × 同じ u」の比を保つ。
    #[test]
    fn bezier_easing_applies_one_shared_u_to_every_color_channel() {
        let mut tr = KeyframeTrack::new();
        tr.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::Color([0.0, 0.0, 0.0, 0.0]),
            interp: Interp::Bezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
            spatial: None,
        });
        tr.insert(Keyframe {
            t: RationalTime::from_seconds(1),
            value: Value::Color([10.0, 20.0, 30.0, 40.0]),
            interp: Interp::Linear,
            spatial: None,
        });
        let quarter = tr
            .eval(RationalTime::try_new(1, 4).unwrap())
            .as_color()
            .expect("Color が返らない");
        let expected_u = cubic_bezier_ease(0.42, 0.0, 0.58, 1.0, 0.25);
        let deltas = [10.0, 20.0, 30.0, 40.0];
        for (channel, delta) in quarter.iter().zip(deltas.iter()) {
            assert!(
                (channel - delta * expected_u).abs() < 1e-9,
                "channel={channel} expected={} (delta={delta} 共有u={expected_u})",
                delta * expected_u
            );
        }
    }
}

#[cfg(test)]
mod split_tests {
    use super::*;

    fn bezier_controls(interp: &Interp) -> (f64, f64, f64, f64) {
        match interp {
            Interp::Bezier { x1, y1, x2, y2 } => (*x1, *y1, *x2, *y2),
            _ => unreachable!(),
        }
    }

    fn eval_split_curve(
        left: (f64, f64, f64, f64),
        right: (f64, f64, f64, f64),
        split: f64,
        x: f64,
    ) -> f64 {
        if x <= 0.0 {
            0.0
        } else if x < split {
            let u = x / split;
            cubic_bezier_ease(left.0, left.1, left.2, left.3, u)
        } else if x < 1.0 {
            let u = if split == 1.0 {
                1.0
            } else {
                (x - split) / (1.0 - split)
            };
            cubic_bezier_ease(right.0, right.1, right.2, right.3, u)
        } else {
            1.0
        }
    }

    #[test]
    fn split_hold_and_linear_keeps_variant() {
        assert_eq!(
            Interp::Hold.split_at(0.42).expect("hold split"),
            (Interp::Hold, Interp::Hold)
        );
        assert_eq!(
            Interp::Linear.split_at(0.42).expect("linear split"),
            (Interp::Linear, Interp::Linear)
        );
    }

    #[test]
    fn split_bezier_returns_bezier_pair() {
        let interp = Interp::Bezier {
            x1: 0.42,
            y1: 0.1,
            x2: 0.58,
            y2: 0.9,
        };
        let (left, right) = interp.split_at(0.35).expect("bezier split");
        let (lx1, _, lx2, _) = bezier_controls(&left);
        let (rx1, _, rx2, _) = bezier_controls(&right);

        assert!((0.0..=1.0).contains(&lx1));
        assert!((0.0..=1.0).contains(&lx2));
        assert!((0.0..=1.0).contains(&rx1));
        assert!((0.0..=1.0).contains(&rx2));
    }

    #[test]
    fn split_bezier_preserves_curve_at_multiple_samples() {
        let interp = Interp::Bezier {
            x1: 0.28,
            y1: -0.2,
            x2: 0.84,
            y2: 0.8,
        };
        let split = 0.43;
        let (left, right) = interp.split_at(split).expect("bezier split");
        let left = bezier_controls(&left);
        let right = bezier_controls(&right);
        let split_value = cubic_bezier_ease(0.28, -0.2, 0.84, 0.8, split);

        for i in 0..=100 {
            let x = i as f64 / 100.0;
            if (x - split).abs() < f64::EPSILON {
                continue;
            }
            let original = cubic_bezier_ease(0.28, -0.2, 0.84, 0.8, x);
            let expected_normalized = if x < split {
                original / split_value
            } else {
                (original - split_value) / (1.0 - split_value)
            };
            let split_value = eval_split_curve(left, right, split, x);

            assert!(
                (expected_normalized - split_value).abs() <= 1e-6,
                "x={x} expected={expected_normalized} split={split_value}"
            );
        }
    }

    #[test]
    fn split_bezier_rejects_invalid_progress() {
        let interp = Interp::Bezier {
            x1: 0.3,
            y1: 0.1,
            x2: 0.7,
            y2: 0.9,
        };
        assert!(matches!(
            interp.split_at(f64::NAN),
            Err(TrackError::UnrepresentableBezierSplit { .. })
        ));
        assert!(matches!(
            interp.split_at(-0.01),
            Err(TrackError::UnrepresentableBezierSplit { .. })
        ));
        assert!(matches!(
            interp.split_at(1.01),
            Err(TrackError::UnrepresentableBezierSplit { .. })
        ));
    }

    #[test]
    fn split_bezier_rejects_unrepresentable() {
        let interp = Interp::Bezier {
            x1: 0.5,
            y1: 0.0,
            x2: 0.5,
            y2: -1.0,
        };
        let err = interp
            .split_at(0.0)
            .expect_err("must reject unrepresentable split");
        match err {
            TrackError::UnrepresentableBezierSplit {
                left_x_denominator,
                right_x_denominator,
                left_y_denominator,
                right_y_denominator,
                ..
            } => {
                assert_eq!(left_x_denominator, 0.0);
                assert_eq!(right_x_denominator, 1.0);
                assert!(left_y_denominator.is_some());
                assert!(right_y_denominator.is_some());
                assert!(left_y_denominator
                    .as_ref()
                    .expect("left y denom should be present")
                    .is_finite());
                assert!(right_y_denominator
                    .as_ref()
                    .expect("right y denom should be present")
                    .is_finite());
            }
            _ => panic!("wrong error kind"),
        }
    }

    #[test]
    fn split_bezier_rejects_infinite_control_value_with_finite_progress() {
        let interp = Interp::Bezier {
            x1: 0.5,
            y1: f64::INFINITY,
            x2: 0.5,
            y2: 0.8,
        };
        let err = interp
            .split_at(0.35)
            .expect_err("must reject inf y control");
        let TrackError::UnrepresentableBezierSplit { split_value, .. } = err else {
            panic!("wrong error kind");
        };

        assert!(split_value.expect("split value is computed").is_infinite());
    }
}
