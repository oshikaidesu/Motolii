//! 256bit相当の符号なし乗除。i128積が溢れる相対時刻×rateでも床添字を厳密に求める(S7)。

use super::RationalTimeError;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct U256 {
    pub hi: u128,
    pub lo: u128,
}

impl U256 {
    pub const ZERO: Self = Self { hi: 0, lo: 0 };

    pub fn from_u128(v: u128) -> Self {
        Self { hi: 0, lo: v }
    }

    pub fn widening_mul(a: u128, b: u128) -> Self {
        const MASK: u128 = (1u128 << 64) - 1;
        let a0 = a & MASK;
        let a1 = a >> 64;
        let b0 = b & MASK;
        let b1 = b >> 64;
        let p00 = a0 * b0;
        let p01 = a0 * b1;
        let p10 = a1 * b0;
        let p11 = a1 * b1;
        let mid = (p00 >> 64) + (p01 & MASK) + (p10 & MASK);
        let lo = (p00 & MASK) | ((mid & MASK) << 64);
        let hi = p11 + (p01 >> 64) + (p10 >> 64) + (mid >> 64);
        Self { hi, lo }
    }

    pub fn checked_mul_u128(self, m: u128) -> Option<Self> {
        let lo_part = Self::widening_mul(self.lo, m);
        let hi_part = Self::widening_mul(self.hi, m);
        if hi_part.hi != 0 {
            return None;
        }
        let (new_hi, overflow) = lo_part.hi.overflowing_add(hi_part.lo);
        if overflow {
            return None;
        }
        Some(Self {
            hi: new_hi,
            lo: lo_part.lo,
        })
    }

    pub fn cmp(self, other: Self) -> Ordering {
        match self.hi.cmp(&other.hi) {
            Ordering::Equal => self.lo.cmp(&other.lo),
            o => o,
        }
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        debug_assert!(self.cmp(other) != Ordering::Less);
        let (lo, borrow) = self.lo.overflowing_sub(other.lo);
        let hi = self.hi - other.hi - u128::from(borrow);
        Self { hi, lo }
    }

    pub fn shl1(self) -> Self {
        let hi = (self.hi << 1) | (self.lo >> 127);
        let lo = self.lo << 1;
        Self { hi, lo }
    }

    pub fn bit(self, i: u32) -> bool {
        if i >= 128 {
            ((self.hi >> (i - 128)) & 1) == 1
        } else {
            ((self.lo >> i) & 1) == 1
        }
    }
}

/// `floor((num_abs * rate_num) / (d1 * d2 * d3))` と余り・約分後分母。すべて非負、分母因子 > 0。
pub(crate) fn mul_div_floor_3den(
    num_abs: u128,
    rate_num: u128,
    d1: u128,
    d2: u128,
    d3: u128,
) -> Result<(u128, U256, U256), RationalTimeError> {
    if d1 == 0 || d2 == 0 || d3 == 0 {
        return Err(RationalTimeError::ZeroDenominator);
    }

    let mut n = num_abs;
    let mut r = rate_num;
    let mut a = d1;
    let mut b = d2;
    let mut c = d3;
    gcd_reduce(&mut n, &mut a);
    gcd_reduce(&mut n, &mut b);
    gcd_reduce(&mut n, &mut c);
    gcd_reduce(&mut r, &mut a);
    gcd_reduce(&mut r, &mut b);
    gcd_reduce(&mut r, &mut c);

    let numer = U256::widening_mul(n, r);
    let den = U256::from_u128(a)
        .checked_mul_u128(b)
        .and_then(|x| x.checked_mul_u128(c))
        .ok_or(RationalTimeError::Overflow)?;
    if den == U256::ZERO {
        return Err(RationalTimeError::ZeroDenominator);
    }

    let (quot, rem) = u256_div_rem(numer, den)?;
    if quot.hi != 0 {
        return Err(RationalTimeError::Overflow);
    }
    Ok((quot.lo, rem, den))
}

fn gcd_reduce(x: &mut u128, y: &mut u128) {
    let g = gcd(*x, *y);
    if g > 1 {
        *x /= g;
        *y /= g;
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// 256÷256 復元除算。S7の分母は i64 三因子 ≤ 2^189 なので、
/// ループ不変条件 `rem < den` の下で `rem<<1` が 256bit を溢れることはない。
fn u256_div_rem(numer: U256, den: U256) -> Result<(U256, U256), RationalTimeError> {
    if den == U256::ZERO {
        return Err(RationalTimeError::ZeroDenominator);
    }
    if numer.cmp(den) == Ordering::Less {
        return Ok((U256::ZERO, numer));
    }
    if den.hi == 0 && numer.hi == 0 {
        return Ok((
            U256::from_u128(numer.lo / den.lo),
            U256::from_u128(numer.lo % den.lo),
        ));
    }

    let mut rem = U256::ZERO;
    let mut quot = U256::ZERO;

    for i in (0u32..256).rev() {
        rem = rem.shl1();
        if numer.bit(i) {
            rem.lo |= 1;
        }
        // 商の上位溢れは最終的に i64 検査へ回す（ここでは 256bit 商を許容）
        quot = quot.shl1();
        if rem.cmp(den) != Ordering::Less {
            rem = rem.saturating_sub(den);
            quot.lo |= 1;
        }
    }
    Ok((quot, rem))
}

/// 余り / 分母 を f64 補間率へ。契約は半開区間 `[0, 1)`。
///
/// 巨大分母で `rem = den - 1` のとき両者が同じ f64 へ丸まり `1.0` になり得るため、
/// その場合は `1.0` 未満の最大有限値へ落とす(次サンプルへの早着と debug_assert 発火を防ぐ)。
pub(crate) fn rem_over_den_f64(rem: U256, den: U256) -> f64 {
    if rem == U256::ZERO {
        return 0.0;
    }
    let raw = if den.hi == 0 && rem.hi == 0 {
        rem.lo as f64 / den.lo as f64
    } else {
        let shift = den.hi.leading_zeros().min(rem.hi.leading_zeros()).min(64);
        let rem_f = ldexp_u256(rem, shift);
        let den_f = ldexp_u256(den, shift);
        rem_f / den_f
    };
    clamp_unit_interval_exclusive(raw)
}

/// 負時刻側の `1 - u` も同じ半開区間へ。
pub(crate) fn complement_unit_interval(u: f64) -> f64 {
    clamp_unit_interval_exclusive(1.0 - u)
}

/// `[0, 1)` へ制限。`>= 1` は `1.0` 未満の最大 f64、負は `0.0`。
pub(crate) fn clamp_unit_interval_exclusive(u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u < 1.0 {
        return u;
    }
    // 1.0 未満の最大有限値(next_down(1.0))
    f64::from_bits(1.0f64.to_bits() - 1)
}

fn ldexp_u256(v: U256, shift: u32) -> f64 {
    // 上位から有効ビットを f64 へ（概算で補間率には十分）
    if v.hi != 0 {
        let s = shift.min(v.hi.leading_zeros());
        let top = if s < 128 {
            (v.hi << s) | (if s == 0 { 0 } else { v.lo >> (128 - s) })
        } else {
            v.lo << (s - 128)
        };
        (top as f64) * 2f64.powi(128 - s as i32)
    } else {
        v.lo as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widening_mul_basic() {
        assert_eq!(U256::widening_mul(2, 3), U256::from_u128(6));
        let p = U256::widening_mul(u128::MAX, 2);
        assert_eq!(p.hi, 1);
        assert_eq!(p.lo, u128::MAX - 1);
    }

    #[test]
    fn mul_div_small() {
        let (q, rem, den) = mul_div_floor_3den(100, 1, 7, 1, 1).unwrap();
        assert_eq!(q, 14);
        assert_eq!(rem, U256::from_u128(2));
        assert_eq!(den, U256::from_u128(7));
        assert!((rem_over_den_f64(rem, den) - 2.0 / 7.0).abs() < 1e-15);
    }

    #[test]
    fn mul_div_review_counterexample_near_zero() {
        // D=i64::MAX, t=1000/D, start=1/(D-2), rate=(D-1)/D
        // 実位置 ≈ 1e-16 → index 0。i128 中間積は溢れる。
        let d = i64::MAX as u128;
        let tn = 1000u128;
        let td = d;
        let on = 1u128;
        let od = d - 2;
        let rn = d - 1;
        let rd = d;
        let rel = tn * od - on * td;
        let (q, rem, _den) = mul_div_floor_3den(rel, rn, td, od, rd).unwrap();
        assert_eq!(q, 0);
        assert!(rem != U256::ZERO);
    }

    #[test]
    fn rem_over_den_never_reaches_one_for_den_minus_one() {
        // 前提: 巨大 u128 では (MAX-1)/MAX が f64 で 1.0 に丸まる
        let den_lo = u128::MAX;
        let rem_lo = u128::MAX - 1;
        let raw = rem_lo as f64 / den_lo as f64;
        assert_eq!(raw, 1.0, "precondition: f64 ratio must round to 1.0");

        let u = rem_over_den_f64(U256::from_u128(rem_lo), U256::from_u128(den_lo));
        assert!(
            (0.0..1.0).contains(&u),
            "interpolation factor must stay in [0,1), got {u}"
        );
        assert_eq!(u, f64::from_bits(1.0f64.to_bits() - 1));

        // 256bit 分母でも同様
        let den = U256 {
            hi: u128::MAX / 2,
            lo: u128::MAX,
        };
        let rem = den.saturating_sub(U256::from_u128(1));
        let u2 = rem_over_den_f64(rem, den);
        assert!(
            (0.0..1.0).contains(&u2),
            "wide den rem=den-1 must stay in [0,1), got {u2}"
        );
    }

    #[test]
    fn complement_unit_interval_stays_below_one() {
        // u が 0 へ丸まった場合でも 1-u が 1.0 にならない
        let c = complement_unit_interval(0.0);
        assert!((0.0..1.0).contains(&c));
        assert_eq!(c, f64::from_bits(1.0f64.to_bits() - 1));

        let c2 = complement_unit_interval(1.0);
        assert_eq!(c2, 0.0);
    }
}
