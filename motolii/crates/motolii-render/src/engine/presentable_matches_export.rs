use motolii_edit::{Document, Intent};
use super::*;
use crate::doc::store::{Composition, Fps, RationalTime};

#[test]
fn the_window_target_holds_the_same_bytes_as_the_export_readback() {
    let (w, h) = (64u32, 32u32);
    let fps = Fps::try_new(30, 1).unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: w,
        height: h,
        fps,
        duration_frames: 1,
        // 中間調でないと encode の回数が見えない
        background: [0.5, 0.25, 0.1, 1.0],
    }))
    .unwrap();
    let mut engine = super::Engine::new().unwrap();
    let t = RationalTime::try_from_frame(0, fps).unwrap();
    let export = engine.render_frame(&doc.view(), t).unwrap();
    assert!(export[0] > 8 && export[0] < 247, "中間調のはず: {:?}", &export[..4]);

    let format = crate::render::compositor::PRESENTABLE_FORMAT;
    // The window's target is the embedder's (the native bridge hands one over); here the test is
    // the embedder and takes one from the pool.
    let target = engine.compositor.view_canvas_for(crate::render::compositor::Window::output(doc.view().composition().unwrap().unwrap().spec()), format);
    engine.render_frame_into(&doc.view(), t, &target.texture).unwrap();
    let data = engine.read_texture_offline(&target.texture).unwrap();
    let bytes_per_row = w * 4;

    let bgra = format!("{format:?}").starts_with("Bgra");
    for y in 0..h as usize {
        for x in 0..w as usize {
            let window = &data[y * bytes_per_row as usize + x * 4..][..4];
            let exported = &export[(y * w as usize + x) * 4..][..4];
            for c in 0..3 {
                let wc = if bgra { 2 - c } else { c };
                assert_eq!(window[wc], exported[c], "({x},{y}) channel {c}: 窓 {:?} export {:?}", window, exported);
            }
        }
    }
}
