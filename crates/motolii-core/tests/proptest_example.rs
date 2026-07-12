//! M2E-6 / M2E-16: プロパティテストは workspace の `proptest` を使う。

use motolii_core::RationalTime;
use proptest::prelude::*;

proptest! {
    // 入力範囲を抑えて加減算中間の i128→i64 溢れを避ける。
    #[test]
    fn rational_time_add_sub_roundtrip(
        num_a in -500i64..=500,
        den_a in 1i64..=500,
        num_b in -500i64..=500,
        den_b in 1i64..=500,
    ) {
        let a = RationalTime::new(num_a, den_a);
        let b = RationalTime::new(num_b, den_b);
        prop_assert_eq!((a + b) - b, a);
        prop_assert_eq!((a - b) + b, a);
    }

    #[test]
    fn try_new_invariants(
        num in i64::MIN..=i64::MAX,
        den in i64::MIN..=i64::MAX,
    ) {
        match RationalTime::try_new(num, den) {
            Err(motolii_core::RationalTimeError::ZeroDenominator) => {
                prop_assert_eq!(den, 0);
            }
            Err(motolii_core::RationalTimeError::Overflow) => {}
            Ok(t) => {
                prop_assert!(t.den() > 0);
                if t.num() == 0 {
                    prop_assert_eq!(t.den(), 1);
                } else {
                    let g = gcd(t.num().unsigned_abs() as u128, t.den() as u128);
                    prop_assert_eq!(g, 1);
                }
            }
        }
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
