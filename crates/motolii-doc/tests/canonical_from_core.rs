//! M2E-14: motolii-doc は nodes に依存せず、core の正準座標型を参照できる。

use motolii_core::{
    CanonicalPoint, CanonicalSize, FrameDesc, PixelPoint, PixelSize, ViewportTransform,
};

#[test]
fn doc_crate_can_use_canonical_types_from_core() {
    let tx = ViewportTransform::from_desc(&FrameDesc::packed(
        1920,
        1080,
        motolii_core::PixelFormat::Rgba8Unorm,
        motolii_core::ColorSpace::Srgb,
        true,
    ));
    assert_eq!(
        tx.point_to_px(CanonicalPoint::CENTER),
        PixelPoint { x: 960.0, y: 540.0 }
    );
    assert_eq!(
        tx.size_to_px(CanonicalSize {
            width: 0.25,
            height: 0.5
        }),
        PixelSize {
            width: 270.0,
            height: 540.0
        }
    );
}
