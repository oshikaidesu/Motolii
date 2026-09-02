
use super::RationalTimeError;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct U256 {
    pub hi: u128,
    pub lo: u128,
}

impl U256 {
    pub(crate) const ZERO: Self = Self { hi: 0, lo: 0 };

    pub(crate) fn from_u128(v: u128) -> Self {
        Self { hi: 0, lo: v }
    }

    pub(crate) fn widening_mul(a: u128, b: u128) -> Self {
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

    pub(crate) fn checked_mul_u128(self, m: u128) -> Option<Self> {
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

    pub(crate) fn cmp(self, other: Self) -> Ordering {
        match self.hi.cmp(&other.hi) {
            Ordering::Equal => self.lo.cmp(&other.lo),
            o => o,
        }
    }

    pub(crate) fn saturating_sub(self, other: Self) -> Self {
        debug_assert!(self.cmp(other) != Ordering::Less);
        let (lo, borrow) = self.lo.overflowing_sub(other.lo);
        let hi = self.hi - other.hi - u128::from(borrow);
        Self { hi, lo }
    }

    pub(crate) fn shl1(self) -> Self {
        let hi = (self.hi << 1) | (self.lo >> 127);
        let lo = self.lo << 1;
        Self { hi, lo }
    }

    pub(crate) fn bit(self, i: u32) -> bool {
        if i >= 128 {
            ((self.hi >> (i - 128)) & 1) == 1
        } else {
            ((self.lo >> i) & 1) == 1
        }
    }
}

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
        quot = quot.shl1();
        if rem.cmp(den) != Ordering::Less {
            rem = rem.saturating_sub(den);
            quot.lo |= 1;
        }
    }
    Ok((quot, rem))
}

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

pub(crate) fn complement_unit_interval(u: f64) -> f64 {
    clamp_unit_interval_exclusive(1.0 - u)
}

pub(crate) fn clamp_unit_interval_exclusive(u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u < 1.0 {
        return u;
    }
    f64::from_bits(1.0f64.to_bits() - 1)
}

fn ldexp_u256(v: U256, shift: u32) -> f64 {
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
