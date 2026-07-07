use std::cmp::Ordering;
use std::ops::{Add, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

/// 有理数タイムスタンプ。秒 = num/den。
///
/// 浮動小数の秒を使うとフレーム境界の丸めが蓄積してドリフトする(落とし穴B-1)ため、
/// タイムライン上の時刻・長さは常にこの型で扱う。常に正規化(den > 0、既約)して保持する。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RationalTime {
    num: i64,
    den: i64,
}

impl RationalTime {
    pub const ZERO: RationalTime = RationalTime { num: 0, den: 1 };

    /// den == 0 はプログラミングエラーとしてpanicする。
    pub fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "RationalTime: denominator must not be zero");
        Self::reduce(num as i128, den as i128)
    }

    pub fn from_seconds(secs: i64) -> Self {
        Self { num: secs, den: 1 }
    }

    pub fn num(&self) -> i64 {
        self.num
    }

    pub fn den(&self) -> i64 {
        self.den
    }

    /// 表示・デバッグ用途のみ。比較や演算にはf64を使わないこと。
    pub fn as_seconds_f64(&self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// フレーム番号から時刻へ(frame / fps)。
    pub fn from_frame(frame: i64, fps: Fps) -> Self {
        Self::reduce(frame as i128 * fps.den as i128, fps.num as i128)
    }

    /// 時刻が属するフレーム番号(床関数)。負の時刻でも数学的な床を返す。
    pub fn to_frame_floor(&self, fps: Fps) -> i64 {
        let n = self.num as i128 * fps.num as i128;
        let d = self.den as i128 * fps.den as i128;
        i128_div_floor(n, d)
    }

    fn reduce(num: i128, den: i128) -> Self {
        let (num, den) = if den < 0 { (-num, -den) } else { (num, den) };
        let g = gcd(num.unsigned_abs(), den.unsigned_abs()).max(1);
        let num = num / g as i128;
        let den = den / g as i128;
        Self {
            num: i64::try_from(num).expect("RationalTime overflow"),
            den: i64::try_from(den).expect("RationalTime overflow"),
        }
    }
}

/// フレームレート(num/den フレーム毎秒)。例: 30fps = 30/1、29.97fps = 30000/1001。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fps {
    pub num: i64,
    pub den: i64,
}

impl Fps {
    pub fn new(num: i64, den: i64) -> Self {
        assert!(num > 0 && den > 0, "Fps must be positive");
        Self { num, den }
    }

    /// 1フレームの長さ。
    pub fn frame_duration(&self) -> RationalTime {
        RationalTime::new(self.den, self.num)
    }

    pub fn as_f64(&self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn i128_div_floor(n: i128, d: i128) -> i64 {
    debug_assert!(d > 0);
    let q = n.div_euclid(d);
    i64::try_from(q).expect("frame index overflow")
}

impl PartialEq for RationalTime {
    fn eq(&self, other: &Self) -> bool {
        // 常に既約・den>0で保持しているためフィールド比較でよい
        self.num == other.num && self.den == other.den
    }
}

impl Eq for RationalTime {}

impl std::hash::Hash for RationalTime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.num.hash(state);
        self.den.hash(state);
    }
}

impl PartialOrd for RationalTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RationalTime {
    fn cmp(&self, other: &Self) -> Ordering {
        // den > 0 が保証されているため交差乗算で比較できる
        let lhs = self.num as i128 * other.den as i128;
        let rhs = other.num as i128 * self.den as i128;
        lhs.cmp(&rhs)
    }
}

impl Add for RationalTime {
    type Output = RationalTime;
    fn add(self, rhs: Self) -> Self {
        Self::reduce(
            self.num as i128 * rhs.den as i128 + rhs.num as i128 * self.den as i128,
            self.den as i128 * rhs.den as i128,
        )
    }
}

impl Sub for RationalTime {
    type Output = RationalTime;
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl Neg for RationalTime {
    type Output = RationalTime;
    fn neg(self) -> Self {
        Self {
            num: -self.num,
            den: self.den,
        }
    }
}

impl Mul<i64> for RationalTime {
    type Output = RationalTime;
    fn mul(self, rhs: i64) -> Self {
        Self::reduce(self.num as i128 * rhs as i128, self.den as i128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_sign_and_reduces() {
        let t = RationalTime::new(2, -4);
        assert_eq!(t.num(), -1);
        assert_eq!(t.den(), 2);
        assert_eq!(RationalTime::new(30, 30), RationalTime::new(1, 1));
    }

    #[test]
    fn arithmetic() {
        let a = RationalTime::new(1, 3);
        let b = RationalTime::new(1, 6);
        assert_eq!(a + b, RationalTime::new(1, 2));
        assert_eq!(a - b, RationalTime::new(1, 6));
        assert_eq!(b * 3, RationalTime::new(1, 2));
    }

    #[test]
    fn ordering_across_denominators() {
        let a = RationalTime::new(1001, 30000); // 29.97fpsの1フレーム
        let b = RationalTime::new(1, 30); // 30fpsの1フレーム
        assert!(a > b);
        assert!(RationalTime::new(-1, 30) < RationalTime::ZERO);
    }

    #[test]
    fn frame_conversion_exact_ntsc() {
        // 29.97fps(30000/1001)で丸め誤差なく往復できること
        let fps = Fps::new(30000, 1001);
        for frame in [0i64, 1, 29, 30, 1799, 1800, 123_456] {
            let t = RationalTime::from_frame(frame, fps);
            assert_eq!(t.to_frame_floor(fps), frame, "frame {frame}");
        }
    }

    #[test]
    fn frame_floor_boundaries() {
        let fps = Fps::new(30, 1);
        // フレーム1のちょうど境界
        assert_eq!(RationalTime::new(1, 30).to_frame_floor(fps), 1);
        // 境界の直前はフレーム0
        assert_eq!(RationalTime::new(999, 30000).to_frame_floor(fps), 0);
        // 負の時刻は数学的床(-1フレーム)
        assert_eq!(RationalTime::new(-1, 60).to_frame_floor(fps), -1);
    }

    #[test]
    fn mixed_fps_clips_align() {
        // 24fpsの素材2秒分の長さは、30fpsタイムライン上でちょうど60フレーム
        let len = RationalTime::from_frame(48, Fps::new(24, 1));
        assert_eq!(len, RationalTime::from_seconds(2));
        assert_eq!(len.to_frame_floor(Fps::new(30, 1)), 60);
    }

    #[test]
    fn no_float_drift_accumulation() {
        // 1フレームずつ10時間分加算しても正確(f64秒ならドリフトする)
        let fps = Fps::new(30000, 1001);
        let mut t = RationalTime::ZERO;
        let n = 30 * 60 * 60 * 10;
        for _ in 0..n {
            t = t + fps.frame_duration();
        }
        assert_eq!(t, RationalTime::from_frame(n, fps));
    }
}
