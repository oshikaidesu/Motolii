//! M2E-6 模範例: プロパティテストは workspace の `proptest` を使う。
//! 独断で quickcheck / arbitrary 直依存を増やさない(監査 EN-5)。

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
}
