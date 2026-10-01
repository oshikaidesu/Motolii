#[test]
fn held_color_preview_changes_every_frame_but_commits_one_history_step() {
    use super::*;
    let mut rt=crate::EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
    let slot=rt.viewer.color_target.clone().unwrap();
    let history=rt.doc.history_depth();
    for step in 0..31 {
        let rgba=[step as f64/30.0,0.25,0.5,1.0];
        rt.request(json!({"op":"previewColor","slot":slot,"rgba":rgba})).unwrap();
        assert_eq!(rt.doc.history_depth(),history);
        assert_eq!(editor::color::read_color(&rt.doc,&slot,RationalTime::ZERO),Some(rgba));
    }
    rt.request(json!({"op":"commitPreview"})).unwrap();
    assert_eq!(rt.doc.history_depth().0,history.0+1);
    rt.request(json!({"op":"undo"})).unwrap();
    assert_eq!(editor::color::read_color(&rt.doc,&slot,RationalTime::ZERO),Some([1.0;4]));
}
#[test]
fn color_target_follows_selection_and_keeps_the_other_layer_unchanged() {
    use super::*;
    let mut rt=crate::EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
    let first=rt.viewer.selected().unwrap();
    let first_slot=rt.viewer.color_target.clone().unwrap();
    rt.request(json!({"op":"setColor","slot":first_slot,"rgba":[1,0,0,1]})).unwrap();
    rt.request(json!({"op":"create","kind":"ellipse"})).unwrap();
    let second=rt.viewer.selected().unwrap();
    assert_eq!(rt.viewer.color_target.as_ref().and_then(ColorSlot::layer),Some(second));
    rt.request(json!({"op":"select","ids":[first.0]})).unwrap();
    let target=rt.viewer.color_target.clone().unwrap();
    assert_eq!(target.layer(),Some(first));
    rt.request(json!({"op":"setColor","slot":target,"rgba":[0,1,0,1]})).unwrap();
    let other=editor::color::default_target(&rt.doc,second).unwrap();
    assert_eq!(editor::color::read_color(&rt.doc,&other,RationalTime::ZERO),Some([1.0;4]));
    rt.request(json!({"op":"undo"})).unwrap();
    assert_eq!(editor::color::read_color(&rt.doc,&first_slot,RationalTime::ZERO),Some([1.0,0.0,0.0,1.0]));
}

#[test]
fn solid_mode_keeps_the_stop_color_visible_at_the_current_time() {
    use super::*;
    let mut rt=crate::EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
    let layer=rt.viewer.selected().unwrap();
    let fill=editor::color::default_target(&rt.doc,layer).unwrap();
    rt.request(json!({"op":"setGradient","slot":fill,"kind":"linear"})).unwrap();
    let model=editor::gradient::model(&rt.doc,&fill,RationalTime::ZERO).unwrap();
    let stop:ColorSlot=serde_json::from_value(model["stops"][0]["slot"].clone()).unwrap();
    rt.request(json!({"op":"setColor","slot":stop,"rgba":[0,0,1,1]})).unwrap();
    assert_eq!(editor::color::read_color(&rt.doc,&stop,RationalTime::ZERO),Some([0.0,0.0,1.0,1.0]));
    rt.request(json!({"op":"setFillMode","slot":fill,"gradient":false})).unwrap();
    let solid=editor::color::default_target(&rt.doc,layer).unwrap();
    assert_eq!(editor::color::read_color(&rt.doc,&solid,RationalTime::ZERO),Some([0.0,0.0,1.0,1.0]));
}
/// 再生: play の後に時間が経てば tick で frame が進む。
#[test]
fn play_then_tick_advances_the_frame() {
    let path = std::env::var("MOTOLII_PROBE_DOC").unwrap_or_default();
    let mut rt = crate::EditorRuntime::open(&path).unwrap();
    rt.request(serde_json::json!({"op":"play"})).unwrap();
    assert!(rt.viewer.clock.playing(), "play の後に clock が走っていない");
    std::thread::sleep(std::time::Duration::from_millis(400));
    rt.request(serde_json::json!({"op":"tick"})).unwrap();
    assert!(rt.viewer.frame > 0, "0.4 秒経っても frame が {}", rt.viewer.frame);
    // 尺は壁ではない: マウスの seek も尺の先へ行ける(利用者 2026-09-07)。
    rt.request(serde_json::json!({"op":"pause"})).unwrap();
    rt.request(serde_json::json!({"op":"seek","frame":9_999})).unwrap();
    assert_eq!(rt.viewer.frame, 9_999);
    rt.request(serde_json::json!({"op":"play"})).unwrap();
    // 再生中の軽い status にも、Swift の render が毎コマ読む寸法が要る。
    let full = rt.status().unwrap();
    let light = rt.status().unwrap();
    assert!(light["liveLayers"].is_array(), "2 回目は軽い status のはず: {light}");
    for key in ["width", "height", "fps", "durationFrames"] {
        assert_eq!(light[key], full[key], "軽い status に {key} が無い");
    }
}
