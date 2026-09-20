//! 並べた結果の検査 — 箱・流し込み・切り・移り方。絵の側の物なので、絵の家で試す。

#[cfg(test)]
mod tests {
    use motolii_edit::{Animate, Document, Intent};
    use motolii_doc::store::*;
    use motolii_doc::store::layout::*;
    use motolii_render::picture::*;
    use motolii_doc::store::{
        rect_shape, ContentKeyframe, ContentTrack, FontRef, LayerAttrsPatch, LayerMeta, LayerTiming, TextDocument, TextDocumentStyle,
        TextJustify, TextStyleId,
    };

    const T: RationalTime = RationalTime::ZERO;

    fn add(doc: &mut Document, id: u64, source: LayerSource, parent: Option<LayerId>) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(0, None, 300) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(parent), ..Default::default() } },
        ])
        .unwrap();
        layer
    }

    fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    }

    fn rect(doc: &mut Document, id: u64, parent: LayerId, size: [f32; 2]) -> LayerId {
        let layer = add(doc, id, LayerSource::Shape, Some(parent));
        doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], size)] }).unwrap();
        layer
    }

    /// 画面の上の箱(world): 解いた変換を、輪郭を伸ばした後の箱に掛ける。
    fn shown(doc: &Document, layer: LayerId, t: RationalTime) -> [f32; 4] {
        let view = doc.view();
        let resolved = motolii_render::picture::resolve::resolved_layers(&view, t).unwrap();
        let r = resolved.iter().find(|l| l.id == layer).unwrap();
        let b = if r.source == LayerSource::Shape {
            motolii_render::picture::boxes::stretched_shape_box(&motolii_render::picture::shapes::shapes_at(&view, layer, t).unwrap(), r.shape_stretch).unwrap()
        } else {
            motolii_render::picture::boxes::layer_box(&view, layer, t).unwrap().unwrap()
        };
        let (lo, hi) = (r.placement.transform.transform_point2(glam::vec2(b[0], b[1])), r.placement.transform.transform_point2(glam::vec2(b[2], b[3])));
        // 群の箱の左上(素材座標の 1 画素の外)から測る。
        [lo.x.min(hi.x), lo.y.min(hi.y), lo.x.max(hi.x), lo.y.max(hi.y)].map(|v| ((v - CANVAS_MARGIN) * 100.0).round() / 100.0)
    }

    fn flex_row(doc: &mut Document) -> LayerId {
        let group = add(doc, 1, LayerSource::Group, None);
        put(doc, group, DISPLAY, Value::Enum(1));
        put(doc, group, GAP, Value::F64(10.0));
        put(doc, group, PADDING, Value::Vec2([20.0, 8.0]));
        group
    }

    #[test]
    fn a_flex_row_butts_boxes_with_gap_and_padding_and_hugs_them() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        let c = rect(&mut doc, 4, group, [40.0, 50.0]);
        assert_eq!(shown(&doc, a, T), [20.0, 8.0, 120.0, 58.0]);
        assert_eq!(shown(&doc, b, T), [130.0, 8.0, 190.0, 58.0]);
        assert_eq!(shown(&doc, c, T), [200.0, 8.0, 240.0, 58.0]);
        assert_eq!(motolii_render::picture::boxes::layer_box(&doc.view(), group, T).unwrap(), Some([1.0, 1.0, 261.0, 67.0]), "Hug: padding + boxes + gaps");
        let background = motolii_render::picture::boxes::background_shapes(&doc.view(), group, T).unwrap();
        assert!(background.is_none(), "no Background colour, nothing to draw");
        put(&mut doc, group, BACKGROUND, Value::Color([0.2, 0.2, 0.6, 1.0]));
        let background = motolii_render::picture::boxes::background_shapes(&doc.view(), group, T).unwrap().unwrap();
        assert_eq!(motolii_render::picture::boxes::stretched_shape_box(&background, [1.0, 1.0]), Some([1.0, 1.0, 261.0, 67.0]), "the background is drawn where the box is");
    }

    #[test]
    fn scale_is_zoom_and_position_is_a_relative_offset() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, a, property::SCALE, Value::Vec2([2.0, 1.0]));
        assert_eq!(shown(&doc, a, T), [20.0, 8.0, 220.0, 58.0], "the box grows");
        assert_eq!(shown(&doc, b, T)[0], 230.0, "and pushes the neighbour");
        put(&mut doc, a, property::POSITION, Value::Vec2([0.0, 30.0]));
        assert_eq!(shown(&doc, a, T)[1], 38.0, "Position moves the layer from its place");
        assert_eq!(shown(&doc, b, T)[0], 230.0, "without moving anyone else");
    }

    #[test]
    fn an_absolute_child_and_a_layer_out_of_time_take_no_space() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        let c = rect(&mut doc, 4, group, [40.0, 50.0]);
        put(&mut doc, a, POSITION_TYPE, Value::Enum(1));
        assert_eq!(shown(&doc, b, T)[0], 20.0);
        doc.apply(Intent::SetTiming { layer: b, timing: LayerTiming::place(50, None, 300) }).unwrap();
        assert_eq!(shown(&doc, c, T)[0], 20.0, "b is not there yet at frame 0");
    }

    #[test]
    fn grid_tracks_are_keyable_fr_and_fill_stretches_the_outline() {
        let mut doc = blank_project();
        let grid = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [
            (DISPLAY, Value::Enum(2)),
            (GRID_COLUMNS, Value::F64(2.0)),
            (GRID_ROWS, Value::F64(1.0)),
            (HORIZONTAL_SIZING, Value::Enum(2)),
            (VERTICAL_SIZING, Value::Enum(2)),
            (WIDTH, Value::F64(400.0)),
            (HEIGHT, Value::F64(100.0)),
            ("layout.column.1", Value::F64(3.0)),
        ] {
            put(&mut doc, grid, name, value);
        }
        let wide = rect(&mut doc, 2, grid, [10.0, 10.0]);
        let round = rect(&mut doc, 3, grid, [10.0, 10.0]);
        for layer in [wide, round] {
            put(&mut doc, layer, HORIZONTAL_SIZING, Value::Enum(1));
            put(&mut doc, layer, VERTICAL_SIZING, Value::Enum(1));
        }
        put(&mut doc, round, OBJECT_FIT, Value::Enum(1));
        assert_eq!(shown(&doc, wide, T), [0.0, 0.0, 300.0, 100.0], "3fr of 400, squashed and stretched");
        assert_eq!(shown(&doc, round, T), [300.0, 0.0, 400.0, 100.0], "Contain: 100 x 100, centred in the 100 wide cell");

        let mut track = motolii_doc::eval::KeyframeTrack::new();
        track.insert(motolii_doc::eval::Keyframe { t: T, value: Value::F64(3.0), interp: motolii_doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(motolii_doc::eval::Keyframe { t: RationalTime::try_from_frame(10, doc.view().composition().unwrap().unwrap().fps).unwrap(), value: Value::F64(1.0), interp: motolii_doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: grid, property: PropertyId::new("layout.column.1").unwrap(), track }).unwrap();
        let later = RationalTime::try_from_frame(10, doc.view().composition().unwrap().unwrap().fps).unwrap();
        assert_eq!(shown(&doc, wide, later), [0.0, 0.0, 200.0, 100.0], "the key on Column 1 moves the line");
    }

    fn put_text(doc: &mut Document, layer: LayerId, content: &str) {
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: T, content: content.to_owned() });
        let style = TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
            size: 48.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
        };
        doc.apply(Intent::SetTextDocument { layer, document: TextDocument {
            content: track, justify: TextJustify::Left, wrap_size: None, styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        } }).unwrap();
    }

    #[test]
    fn text_filling_its_cell_wraps_at_the_cell_and_grows_down() {
        let mut doc = blank_project();
        let column = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(1)), (FLEX_DIRECTION, Value::Enum(1)), (HORIZONTAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(300.0))] {
            put(&mut doc, column, name, value);
        }
        let words = add(&mut doc, 2, LayerSource::Text, Some(column));
        put_text(&mut doc, words, "Taro Yamada is the creative director of this studio");
        let below = rect(&mut doc, 3, column, [40.0, 40.0]);
        let one_line = shown(&doc, below, T)[1];
        put(&mut doc, words, HORIZONTAL_SIZING, Value::Enum(1));
        let view = doc.view();
        let wrap = motolii_render::picture::resolve::text::resolved_text_document(&view, words, T).unwrap().unwrap().wrap_size.expect("wraps");
        assert_eq!(wrap[0], 300.0, "at the cell width");
        drop(view);
        assert!(shown(&doc, below, T)[1] > one_line + 40.0, "the wrapped lines push the next item down: {one_line} → {}", shown(&doc, below, T)[1]);

        put(&mut doc, words, property::SCALE, Value::Vec2([2.0, 2.0]));
        assert_eq!(motolii_render::picture::resolve::text::resolved_text_document(&doc.view(), words, T).unwrap().unwrap().wrap_size.unwrap()[0], 150.0, "Scale is zoom: the words wrap at half the width, then double");
    }

    #[test]
    fn overflow_clip_cuts_every_descendant_at_the_groups_box() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        assert!(motolii_render::picture::resolve::resolved_layers(&doc.view(), T).unwrap().iter().find(|l| l.id == a).unwrap().masks.is_empty(), "Visible cuts nothing");
        put(&mut doc, group, OVERFLOW, Value::Enum(1));
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        put(&mut doc, a, property::POSITION, Value::Vec2([0.0, 30.0]));
        let resolved = motolii_render::picture::resolve::resolved_layers(&doc.view(), T).unwrap();
        let layer = resolved.iter().find(|l| l.id == a).unwrap();
        assert_eq!(layer.masks.len(), 1);
        assert_eq!(layer.masks[0].mode, motolii_doc::store::MaskMode::Intersect);
        let world: Vec<glam::Vec2> = layer.masks[0].shape.vertices.iter().map(|v| layer.placement.transform.transform_point2(glam::vec2(v.point[0] as f32, v.point[1] as f32))).collect();
        let (lo, hi) = world.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        assert_eq!((lo.round(), hi.round()), (glam::vec2(301.0, 201.0), glam::vec2(441.0, 267.0)), "the group's box on screen, whatever the child's offset");
    }

    /// CSS `clip-path: inset(10% 0 0 0)` の写し: 箱の上から 10% を削った矩形が見える範囲。辺ごとに鍵が打てる(値は px)。
    #[test]
    fn clip_inset_cuts_the_box_from_each_edge_like_css_inset() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        for (name, value) in [(HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(400.0)), (HEIGHT, Value::F64(300.0))] {
            put(&mut doc, group, name, value);
        }
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let bounds = |doc: &Document, id: LayerId| {
            let resolved = motolii_render::picture::resolve::resolved_layers(&doc.view(), T).unwrap();
            let layer = resolved.iter().find(|l| l.id == id).unwrap().clone();
            let world: Vec<glam::Vec2> = layer.masks.iter().flat_map(|m| m.shape.vertices.iter().map(|v| layer.placement.transform.transform_point2(glam::vec2(v.point[0] as f32, v.point[1] as f32)))).collect();
            let (lo, hi) = world.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            (layer.masks.len(), lo.round(), hi.round())
        };
        assert_eq!(bounds(&doc, a).0, 0, "inset(0) cuts nothing, Overflow Visible");
        put(&mut doc, group, CLIP_TOP, Value::F64(30.0));
        assert_eq!(bounds(&doc, a), (1, glam::vec2(301.0, 231.0), glam::vec2(701.0, 501.0)), "inset(10% 0 0 0) of a 400 x 300 box: the top 30 px are gone");
        assert_eq!(bounds(&doc, group), (1, glam::vec2(301.0, 231.0), glam::vec2(701.0, 501.0)), "the group's own background is cut too (clip-path is per element)");
        put(&mut doc, group, CLIP_RIGHT, Value::F64(100.0));
        put(&mut doc, group, CLIP_BOTTOM, Value::F64(50.0));
        put(&mut doc, group, CLIP_LEFT, Value::F64(40.0));
        assert_eq!(bounds(&doc, a), (1, glam::vec2(341.0, 231.0), glam::vec2(601.0, 451.0)), "inset(top right bottom left) in CSS order");
        put(&mut doc, group, CLIP_LEFT, Value::F64(1000.0));
        let (_, lo, hi) = bounds(&doc, a);
        assert_eq!(hi.x - lo.x, 0.0, "an edge past the opposite edge leaves an empty box");
        put(&mut doc, group, CLIP_LEFT, Value::F64(40.0));
        put(&mut doc, group, OVERFLOW, Value::Enum(1));
        assert_eq!(bounds(&doc, a).0, 2, "Overflow Clip and the inset are two cuts (border-radius and `round` are separate in CSS)");
    }

    /// 箱の切りは箱の枠に付く(CSS の overflow: clip / clip-path は要素の箱に掛かり、中で transform した子孫は箱で切れる):
    /// Overflow Clip の箱の子が持つ切りは `Box`(ブロックのずれの後に箱で切る)、箱自身の inset は `Layer`(要素ごと、自分と動く)。
    #[test]
    fn a_boxs_clip_binds_to_the_box_and_a_layers_own_clip_to_the_layer() {
        use motolii_doc::store::MaskFrame;
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        put(&mut doc, group, OVERFLOW, Value::Enum(1));
        put(&mut doc, group, CLIP_TOP, Value::F64(10.0));
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let frames = |doc: &Document, id: LayerId| -> Vec<MaskFrame> {
            motolii_render::picture::resolve::resolved_layers(&doc.view(), T).unwrap().iter().find(|l| l.id == id).unwrap().masks.iter().map(|m| m.frame).collect()
        };
        assert_eq!(frames(&doc, a), vec![MaskFrame::Box, MaskFrame::Box], "the child is cut by the box's Overflow and inset, both bound to the box");
        assert_eq!(frames(&doc, group), vec![MaskFrame::Layer], "the box's own inset moves with the box (clip-path is per element)");
    }

    /// Split Words + Stagger: 語ごとに時刻がずれる(鍵は 1 本、単位は箱の mask、書類に子の層は無い)。
    /// Stagger は箱の子と同じ法 = 全体の幅(GSAP の `stagger: {amount}`): 3 語で 0.2 なら 2 語目は 0.1、3 語目は 0.2 遅れる。
    #[test]
    fn split_words_give_each_word_its_own_time_under_the_texts_stagger() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let words = add(&mut doc, 2, LayerSource::Text, None);
        put_text(&mut doc, words, "ONE TWO THREE");
        let mut track = motolii_doc::eval::KeyframeTrack::new();
        track.insert(motolii_doc::eval::Keyframe { t: at(0), value: Value::F64(0.0), interp: motolii_doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(motolii_doc::eval::Keyframe { t: at(30), value: Value::F64(1.0), interp: motolii_doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: words, property: PropertyId::new(property::OPACITY).unwrap(), track }).unwrap();
        put(&mut doc, words, STAGGER, Value::F64(0.2));
        let copies = |doc: &Document| -> Vec<(f32, f32)> {
            let mut out: Vec<_> = motolii_render::picture::resolve::resolved_layers(&doc.view(), at(15)).unwrap().into_iter().filter(|l| l.id == words)
                .map(|l| (l.placement.opacity, l.masks.first().map_or(f32::NAN, |m| m.shape.vertices[0].point[0] as f32))).collect();
            out.sort_by(|a, b| b.0.total_cmp(&a.0));
            out
        };
        assert_eq!(copies(&doc).len(), 1, "Split None: one layer, no unit");
        put(&mut doc, words, motolii_doc::store::names::TEXT_SPLIT, Value::Enum(2));
        let c = copies(&doc);
        assert_eq!(c.len(), 3, "three words");
        assert!((c[0].0 - 0.5).abs() < 0.02 && (c[1].0 - 0.4).abs() < 0.02 && (c[2].0 - 0.3).abs() < 0.02, "at 0.5 s the words read their keys at 0.5 / 0.4 / 0.3 s: {c:?}");
        assert!(c[0].1 < c[1].1 && c[1].1 < c[2].1, "the units are in reading order, each cut to its own box: {c:?}");
        put(&mut doc, words, motolii_doc::store::names::TEXT_SPLIT, Value::Enum(1));
        assert_eq!(copies(&doc).len(), 11, "Chars: the spaces are not units");
        put(&mut doc, words, motolii_doc::store::names::TEXT_SPLIT, Value::Enum(3));
        assert_eq!(copies(&doc).len(), 1, "Lines: one line is not split");
    }

    /// Loop: 鍵 0 → 1 s、Loop Duration 1 なら t = 2.5 は 0.5 として読む。Alternate は奇数回目が逆向き(t = 1.5 → 0.5、1.2 → 0.8)。
    #[test]
    fn loop_folds_the_layers_time_like_css_animation_direction() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let thing = add(&mut doc, 2, LayerSource::Shape, None);
        let mut track = motolii_doc::eval::KeyframeTrack::new();
        track.insert(motolii_doc::eval::Keyframe { t: at(0), value: Value::F64(0.0), interp: motolii_doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(motolii_doc::eval::Keyframe { t: at(30), value: Value::F64(1.0), interp: motolii_doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: thing, property: PropertyId::new(property::OPACITY).unwrap(), track }).unwrap();
        let opacity = |doc: &Document, f: i64| match doc.view().value_at(thing, &PropertyId::new(property::OPACITY).unwrap(), at(f)).unwrap() {
            Some(Value::F64(v)) => v,
            other => panic!("{other:?}"),
        };
        assert_eq!(opacity(&doc, 75), 1.0, "no loop: past the last key it holds");
        put(&mut doc, thing, LOOP_DURATION, Value::F64(1.0));
        assert!((opacity(&doc, 75) - 0.5).abs() < 1e-6, "Normal: t = 2.5 reads as 0.5");
        assert!((opacity(&doc, 36) - 0.2).abs() < 1e-6, "Normal: t = 1.2 reads as 0.2");
        put(&mut doc, thing, LOOP_DIRECTION, Value::Enum(2));
        assert!((opacity(&doc, 45) - 0.5).abs() < 1e-6, "Alternate: t = 1.5 reads as 0.5");
        assert!((opacity(&doc, 36) - 0.8).abs() < 1e-6, "Alternate: t = 1.2 is on the way back, 0.8");
        assert!((opacity(&doc, 15) - 0.5).abs() < 1e-6, "Alternate: the first pass runs forward");
        put(&mut doc, thing, LOOP_DIRECTION, Value::Enum(1));
        assert!((opacity(&doc, 15) - 0.5).abs() < 1e-6 && (opacity(&doc, 7) - (1.0 - 7.0 / 30.0)).abs() < 1e-6, "Reverse runs every pass backwards");
        assert_eq!(doc.view().value_at(thing, &PropertyId::new(LOOP_DURATION).unwrap(), at(75)).unwrap(), Some(Value::F64(1.0)), "the loop rows themselves are read unfolded");
    }

    #[test]
    fn things_on_a_laid_out_face_share_its_point_even_when_the_face_is_tilted() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        put(&mut doc, group, property::ROTATION_Y, Value::F64(30.0));
        let flat = rect(&mut doc, 2, group, [100.0, 50.0]);
        let lifted = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, lifted, property::POSITION_Z, Value::F64(-40.0));
        let resolved = motolii_render::picture::resolve::resolved_layers(&doc.view(), T).unwrap();
        let plane = |id| resolved.iter().find(|l| l.id == id).unwrap().placement.plane;
        let world = resolved.iter().find(|l| l.id == group).unwrap().placement.world_transform.unwrap();
        let b = motolii_render::picture::boxes::layer_box(&doc.view(), group, T).unwrap().unwrap();
        let face = world.transform_point3(glam::vec3((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5, 0.0)).to_array();
        assert_eq!(plane(group), Some(face), "the tilted group is the face");
        assert_eq!(plane(flat), Some(face), "a flat child lies on it and stacks by order");
        assert_eq!(plane(lifted), None, "a child lifted off the face sorts by distance");
    }

    #[test]
    fn depth_alignment_sets_each_childs_z_against_the_deepest_one() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let slab = rect(&mut doc, 2, group, [100.0, 50.0]);
        let card = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, slab, property::DEPTH, Value::F64(100.0));
        let z = |doc: &Document, id| motolii_render::picture::frame::layout_frame(&doc.view(), T).unwrap().slots[&id].z;
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, 0.0), "Back: backs on the face, the slab stands out toward the camera");
        assert_eq!(motolii_render::picture::frame::layout_frame(&doc.view(), T).unwrap().depths[&group], [-100.0, 0.0]);
        put(&mut doc, group, DEPTH_ALIGNMENT, Value::Enum(1));
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, -50.0), "Center: the card floats at the slab's middle");
        put(&mut doc, group, DEPTH_ALIGNMENT, Value::Enum(2));
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, -100.0), "Front: fronts together");
        let resolved = motolii_render::picture::resolve::resolved_layers(&doc.view(), T).unwrap();
        assert_eq!(resolved.iter().find(|l| l.id == card).unwrap().placement.z, -100.0, "the z reaches the resolved layer");
    }

    #[test]
    fn flex_direction_depth_stacks_children_back_by_thickness_and_gap() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        put(&mut doc, group, FLEX_DIRECTION, Value::Enum(DIRECTION_DEPTH));
        put(&mut doc, group, GAP, Value::F64(20.0));
        let back = rect(&mut doc, 2, group, [100.0, 50.0]);
        let slab = rect(&mut doc, 3, group, [100.0, 50.0]);
        let front = rect(&mut doc, 4, group, [100.0, 50.0]);
        put(&mut doc, slab, property::DEPTH, Value::F64(30.0));
        let frame = motolii_render::picture::frame::layout_frame(&doc.view(), T).unwrap();
        assert_eq!([frame.slots[&front].z, frame.slots[&slab].z, frame.slots[&back].z], [0.0, 20.0, 70.0], "top of the stack in front, then thickness + gap each");
        assert_eq!(frame.depths[&group], [0.0, 70.0]);
        assert_eq!(frame.slots[&front].position, frame.slots[&back].position, "one spot on the face");
    }

    #[test]
    fn layout_rotation_lays_out_the_turned_box_and_rotation_does_not() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let turned = rect(&mut doc, 2, group, [100.0, 50.0]);
        let next = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, turned, property::ROTATION, Value::F64(90.0));
        assert_eq!(shown(&doc, next, T)[0], 130.0, "Rotation is only the look");
        put(&mut doc, turned, property::ROTATION, Value::F64(0.0));
        put(&mut doc, turned, LAYOUT_ROTATION, Value::F64(90.0));
        assert_eq!(shown(&doc, next, T)[0], 80.0, "Layout Rotation: the 100 x 50 box stands 50 wide");
        let turned_box = shown(&doc, turned, T);
        assert_eq!([turned_box[0], turned_box[2] - turned_box[0], turned_box[3] - turned_box[1]], [20.0, 50.0, 100.0], "and sits in its place turned");
    }

    #[test]
    fn exclusions_leave_the_cells_a_tracked_blob_covers_empty() {
        let mut doc = blank_project();
        let grid = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [
            (DISPLAY, Value::Enum(2)), (GRID_COLUMNS, Value::F64(3.0)), (GRID_ROWS, Value::F64(3.0)),
            (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(300.0)), (HEIGHT, Value::F64(300.0)),
            (EXCLUSIONS, Value::LayerId(20)),
        ] {
            put(&mut doc, grid, name, value);
        }
        let cards: Vec<LayerId> = (2..11).map(|id| rect(&mut doc, id, grid, [10.0, 10.0])).collect();
        for &card in &cards {
            put(&mut doc, card, HORIZONTAL_SIZING, Value::Enum(1));
            put(&mut doc, card, VERTICAL_SIZING, Value::Enum(1));
        }
        let mut inputs = motolii_doc::store::analysis::AnalysisInputs::default();
        // 真ん中の枠(comp の 101..201)に人の群れが 1 つ。
        inputs.set_blobs(LayerId(20), motolii_doc::store::EffectId(0), T, vec![motolii_doc::store::analysis::BlobMark { id: 1, center: [151.0, 151.0], size: [30.0, 30.0], age: 0 }]);
        let view = doc.view().with_analysis(&inputs);
        let resolved = motolii_render::picture::resolve::resolved_layers(&view, T).unwrap();
        let centre = |id: LayerId| {
            let r = resolved.iter().find(|l| l.id == id).unwrap();
            let b = motolii_render::picture::boxes::stretched_shape_box(&motolii_render::picture::shapes::shapes_at(&view, id, T).unwrap(), r.shape_stretch).unwrap();
            r.placement.transform.transform_point2(glam::vec2((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5))
        };
        let in_middle = cards.iter().filter(|&&c| { let p = centre(c); p.x > 101.0 && p.x < 201.0 && p.y > 101.0 && p.y < 201.0 }).count();
        assert_eq!(in_middle, 0, "no card sits where the crowd is");
        assert!(cards.iter().any(|&c| centre(c).y >= 300.0), "the ninth card flows into an implicit row below the grid");
    }

    #[test]
    fn free_boxes_keep_their_declared_distance_and_the_one_that_does_not_yield_stays() {
        let mut doc = blank_project();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for (layer, x) in [(a, 100.0), (b, 150.0)] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, property::POSITION, Value::Vec2([x, 100.0]));
            put(&mut doc, layer, MARGIN, Value::F64(10.0));
        }
        let left = |doc: &Document, id| shown(doc, id, T)[0];
        // 素のままなら 50 px 重なる。間合い 10 ずつ → 箱の間は 20 空く。譲りは半分ずつ。
        assert_eq!(left(&doc, b) - (left(&doc, a) + 100.0), 20.0, "they stand apart by both margins");
        assert_eq!((left(&doc, a), left(&doc, b)), (65.0, 185.0), "each yields half");
        put(&mut doc, a, FLEX_SHRINK, Value::F64(0.0));
        assert_eq!((left(&doc, a), left(&doc, b)), (100.0, 220.0), "a does not yield, b takes all of it");
    }

    #[test]
    fn a_transition_moves_to_the_new_place_over_its_duration_without_state() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for layer in [a, b] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, MARGIN, Value::F64(10.0));
        }
        put(&mut doc, a, property::POSITION, Value::Vec2([100.0, 100.0]));
        put(&mut doc, a, FLEX_SHRINK, Value::F64(0.0));
        // b は 30 コマ目に a の真上へ飛び込む(鍵は Hold)。押し戻されて右へ。
        let mut track = motolii_doc::eval::KeyframeTrack::new();
        track.insert(motolii_doc::eval::Keyframe { t: at(0), value: Value::Vec2([600.0, 100.0]), interp: motolii_doc::eval::Interp::Hold, spatial: Default::default() });
        track.insert(motolii_doc::eval::Keyframe { t: at(30), value: Value::Vec2([150.0, 100.0]), interp: motolii_doc::eval::Interp::Hold, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: b, property: PropertyId::new("position").unwrap(), track }).unwrap();
        put(&mut doc, b, TRANSITION_DURATION, Value::F64(0.5));
        put(&mut doc, b, TRANSITION_EASING, Value::Enum(1));
        // 移り方が掛かるのは解いた関係(押し戻し)だけ。b 自身の鍵の動きは区間イージングの係で、そのまま効く。
        let x = |f: i64| shown(&doc, b, at(f))[0];
        assert!(x(30) < 160.0, "b lands where its key says, on top of a, and only begins to be pushed: {}", x(30));
        assert!(x(37) > 170.0 && x(37) < 210.0, "halfway through the duration it is being pushed out: {}", x(37));
        assert_eq!(x(45), 220.0, "after the duration it rests at the solved distance");
        assert_eq!(x(37), shown(&doc, b, at(37))[0], "the same frame asked again gives the same picture");
    }

    #[test]
    fn a_container_staggers_its_childrens_transitions_by_distance_and_nothing_jumps() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        // 1 行に 4 つ並べ、30 コマ目に Flex Direction を Column へ(Hold)。子は下へ積み直す。
        let group = flex_row(&mut doc);
        let mut track = motolii_doc::eval::KeyframeTrack::new();
        track.insert(motolii_doc::eval::Keyframe { t: at(0), value: Value::Enum(0), interp: motolii_doc::eval::Interp::Hold, spatial: Default::default() });
        track.insert(motolii_doc::eval::Keyframe { t: at(30), value: Value::Enum(1), interp: motolii_doc::eval::Interp::Hold, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: group, property: PropertyId::new(FLEX_DIRECTION).unwrap(), track }).unwrap();
        let children: Vec<LayerId> = (2..6).map(|id| rect(&mut doc, id, group, [60.0, 60.0])).collect();
        for &child in &children {
            put(&mut doc, child, TRANSITION_DURATION, Value::F64(0.5));
            put(&mut doc, child, TRANSITION_EASING, Value::Enum(4));
        }
        put(&mut doc, group, STAGGER, Value::F64(1.0));
        let y = |doc: &Document, child: LayerId, f: i64| shown(doc, child, at(f))[1];
        // 最初の子は並びの上で位置が変わらない。2 番目から、起点から遠いほど遅れて動き出す。
        let started = |doc: &Document, child: LayerId| (30..120).find(|f| (y(doc, child, *f) - y(doc, child, 29)).abs() > 1.0);
        let (s1, s3) = (started(&doc, children[1]).expect("moves"), started(&doc, children[3]).expect("moves"));
        assert!(s3 > s1 + 5, "the far child starts later than the near one: {s1} vs {s3}");
        for &child in &children {
            let ys: Vec<f32> = (25..120).map(|f| y(&doc, child, f)).collect();
            // 速さの変わり方(2 階の差)が小さい = 跳ばない、尖らない。
            let biggest = ys.windows(3).map(|w| (w[2] - 2.0 * w[1] + w[0]).abs()).fold(0.0, f32::max);
            assert!(biggest < 12.0, "no frame jumps or kinks (largest change of speed {biggest})");
        }
        put(&mut doc, group, STAGGER_FROM, Value::Enum(2));
        let (e1, e3) = (started(&doc, children[1]).expect("moves"), started(&doc, children[3]).expect("moves"));
        assert!(e1 > e3, "from the end, the near-the-start child waits: {e1} vs {e3}");
    }

    #[test]
    fn solid_things_push_apart_in_depth_only_when_their_depths_overlap() {
        let mut doc = blank_project();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for layer in [a, b] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, MARGIN, Value::F64(5.0));
            put(&mut doc, layer, property::DEPTH, Value::F64(100.0));
            put(&mut doc, layer, property::POSITION, Value::Vec2([300.0, 300.0]));
        }
        put(&mut doc, b, property::POSITION_Z, Value::F64(400.0));
        let frame = motolii_render::picture::frame::layout_frame(&doc.view(), T).unwrap();
        assert!(frame.nudges.is_empty() && frame.nudges_z.is_empty(), "one behind the other with room between: nobody moves");
        put(&mut doc, b, property::POSITION_Z, Value::F64(60.0));
        let frame = motolii_render::picture::frame::layout_frame(&doc.view(), T).unwrap();
        let (za, zb) = (frame.nudges_z.get(&a).copied().unwrap_or(0.0), frame.nudges_z.get(&b).copied().unwrap_or(0.0));
        assert!(za < 0.0 && zb > 0.0, "overlapping in depth, they push apart along depth: {za} {zb}");
        assert!((zb - za - 50.0).abs() < 2.0, "just enough to clear depth + margins: {}", zb - za);
        assert!(frame.nudges.values().all(|d| d[0].abs() < 1e-3 && d[1].abs() < 1e-3), "straight behind each other: no sideways push");
        doc.apply(Intent::SetAttrs { layer: b, patch: LayerAttrsPatch { projection: Some(motolii_doc::store::LayerProjection::TwoD), ..Default::default() } }).unwrap();
        let frame = motolii_render::picture::frame::layout_frame(&doc.view(), T).unwrap();
        assert!(frame.nudges_z.is_empty(), "a 2D thing has no depth to push along");
    }

    /// なぞる形の Margin は箱からの間合いで、押し合いの余白ではない(渋谷の窓で角の掴みが奥行きに押されて跳んだ)。
    #[test]
    fn a_trace_keeps_its_margin_to_its_box_and_is_not_pushed() {
        let mut doc = blank_project();
        let thing = add(&mut doc, 1, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: thing, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
        put(&mut doc, thing, MARGIN, Value::F64(5.0));
        put(&mut doc, thing, property::POSITION, Value::Vec2([300.0, 300.0]));
        let trace = add(&mut doc, 2, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: trace, shapes: vec![rect_shape([255; 4], [10.0, 10.0])] }).unwrap();
        put(&mut doc, trace, CONNECT_FROM, Value::LayerId(thing.0));
        put(&mut doc, trace, TRACE, Value::Enum(2));
        put(&mut doc, trace, MARGIN, Value::F64(6.0));
        let frame = motolii_render::picture::frame::layout_frame(&doc.view(), T).unwrap();
        assert!(frame.nudges.is_empty() && frame.nudges_z.is_empty(), "the trace and its box overlap by design: nobody moves");
    }

    /// 押された跡(Trace = Push)と、押された量を読む文字(Readout = Push): 跡の箱は書いた場所、矢印は今の中心へ、文字は押された px。
    #[test]
    fn a_push_trace_shows_where_it_wanted_to_be_and_a_readout_reads_how_far() {
        let mut doc = blank_project();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for (layer, x) in [(a, 300.0), (b, 360.0)] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, MARGIN, Value::F64(5.0));
            put(&mut doc, layer, property::POSITION, Value::Vec2([x, 300.0]));
            doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(motolii_doc::store::LayerProjection::TwoD), ..Default::default() } }).unwrap();
        }
        let trace = add(&mut doc, 3, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: trace, shapes: vec![rect_shape([255; 4], [10.0, 10.0])] }).unwrap();
        put(&mut doc, trace, CONNECT_FROM, Value::LayerId(b.0));
        put(&mut doc, trace, TRACE, Value::Enum(7));
        let view = doc.view();
        let pushed = glam::Vec2::from(motolii_render::picture::boxes::nudge(&view, b, T).unwrap());
        assert!(pushed.x > 10.0, "b is pushed right, away from a: {pushed:?}");
        let (lo, hi) = motolii_render::picture::boxes::box_seen_from(&view, b, trace, T).unwrap().unwrap();
        let path = motolii_render::picture::connect::trace_path(&view, trace, T).unwrap().unwrap();
        assert_eq!(path.len(), 3, "the wanted box, the shaft and the head");
        let ghost: Vec<glam::Vec2> = path[0].vertices.iter().map(|v| glam::vec2(v.point.x as f32, v.point.y as f32)).collect();
        assert!((ghost[0] - (lo - pushed)).length() < 0.01, "the wanted box is the box moved back by the push: {ghost:?} {lo:?}");
        let tip = path[1].vertices[1].point;
        assert!((glam::vec2(tip.x as f32, tip.y as f32) - (lo + hi) * 0.5).length() < 0.01, "the arrow ends at the centre of where it is");
        drop(view);

        let reader = add(&mut doc, 4, LayerSource::Text, None);
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: T, content: "# px".to_owned() });
        let style = TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
            size: 48.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
        };
        doc.apply(Intent::SetTextDocument { layer: reader, document: TextDocument {
            content: track, justify: TextJustify::Left, wrap_size: None, styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        } }).unwrap();
        put(&mut doc, reader, READOUT, Value::Enum(1));
        put(&mut doc, reader, READOUT_OF, Value::LayerId(b.0));
        let text = motolii_render::picture::resolve::text::resolved_text_document(&doc.view(), reader, T).unwrap().unwrap();
        assert_eq!(text.content.eval(T), format!("{} px", pushed.length().round() as i64), "the # becomes the push in px");
    }

    /// Overflow = Bounce: 箱の外へ書いた動きは内側へ鏡で折り返る(四角は軸ごと、円は中心を通る線の上)。中にいれば動かない。
    #[test]
    fn a_bouncing_box_folds_what_leaves_it_back_inside() {
        let m = CANVAS_MARGIN;
        assert_eq!(bounced([m + 10.0, m + 10.0], [m + 30.0, m + 30.0], [200.0, 100.0], 0.0), [0.0, 0.0], "inside: untouched");
        let right = bounced([m + 200.0, m + 10.0], [m + 220.0, m + 30.0], [200.0, 100.0], 0.0);
        assert!((right[0] + 40.0).abs() < 1e-3 && right[1] == 0.0, "20 px past the right wall comes back 20 px from it: {right:?}");
        let far = bounced([m + 180.0 + 360.0, m + 10.0], [m + 200.0 + 360.0, m + 30.0], [200.0, 100.0], 0.0);
        assert!((far[0] + 360.0).abs() < 1e-3, "one full round trip lands where it started: {far:?}");
        // 円: 半径 50 の中の直径 20 の物。中心から 60 外へ出た物は、同じ線の上を戻る。
        let out = bounced([m + 100.0 + 60.0 - 10.0, m + 50.0 - 10.0], [m + 100.0 + 60.0 + 10.0, m + 50.0 + 10.0], [200.0, 100.0], 50.0);
        assert!((out[0] + 40.0).abs() < 1e-3 && out[1].abs() < 1e-3, "room 40: 60 out folds to 20 out: {out:?}");
        let through = bounced([m + 100.0 + 120.0 - 10.0, m + 40.0], [m + 100.0 + 120.0 + 10.0, m + 60.0], [200.0, 100.0], 50.0);
        assert!((through[0] + 160.0).abs() < 1e-3, "80 past the wall crosses the centre and reaches the far wall: {through:?}");
    }

    /// 2 つ目の相手を指すと、相手の箱は 2 つの箱の重なり(離れていれば間)。
    #[test]
    fn a_label_with_two_anchors_sits_on_their_overlap_or_between_them() {
        let mut doc = blank_project();
        let square = |doc: &mut Document, id: u64, at: [f64; 2]| {
            let layer = add(doc, id, LayerSource::Shape, None);
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(doc, layer, property::POSITION, Value::Vec2(at));
            layer
        };
        let a = square(&mut doc, 1, [100.0, 100.0]);
        let b = square(&mut doc, 2, [100.0, 160.0]);
        let label = add(&mut doc, 3, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: label, shapes: vec![rect_shape([255; 4], [20.0, 10.0])] }).unwrap();
        put(&mut doc, label, POSITION_ANCHOR, Value::LayerId(a.0));
        put(&mut doc, label, POSITION_ANCHOR_2, Value::LayerId(b.0));
        put(&mut doc, label, POSITION_AREA, Value::Enum(5));
        let (ba, bb) = (shown(&doc, a, T), shown(&doc, b, T));
        let bl2 = shown(&doc, label, T);
        let centre = |r: [f32; 4]| [(r[0] + r[2]) * 0.5, (r[1] + r[3]) * 0.5];
        let overlap = [ba[0].max(bb[0]), ba[1].max(bb[1]), ba[2].min(bb[2]), ba[3].min(bb[3])];
        assert!((centre(bl2)[0] - centre(overlap)[0]).abs() < 0.01 && (centre(bl2)[1] - centre(overlap)[1]).abs() < 0.01, "centred on the overlap: {bl2:?} {overlap:?}");
        put(&mut doc, b, property::POSITION, Value::Vec2([100.0, 300.0]));
        let (ba, bb, bl) = (shown(&doc, a, T), shown(&doc, b, T), shown(&doc, label, T));
        let gap_y = (ba[3] + bb[1]) * 0.5;
        assert!((centre(bl)[1] - gap_y).abs() < 0.01, "apart: centred in the gap between them: {bl:?} {ba:?} {bb:?}");
    }

    /// 配置はコマをまたいで覚えるが、書類を直せば古い覚えは読まない(版で捨てる)。仮の編集の view は覚えを使わない。
    #[test]
    fn the_layout_cache_forgets_on_edit_and_ignores_previews() {
        let mut doc = blank_project();
        let row = add(&mut doc, 1, LayerSource::Group, None);
        put(&mut doc, row, DISPLAY, Value::Enum(1));
        put(&mut doc, row, GAP, Value::F64(10.0));
        let a = add(&mut doc, 2, LayerSource::Shape, Some(row));
        let b = add(&mut doc, 3, LayerSource::Shape, Some(row));
        for layer in [a, b] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [40.0, 40.0])] }).unwrap();
        }
        let x = |doc: &Document| motolii_render::picture::frame::layout_frame(&doc.view(), T).unwrap().slots[&b].position[0];
        let first = x(&doc);
        assert_eq!(x(&doc), first, "a second view reads the same frame");
        put(&mut doc, row, GAP, Value::F64(50.0));
        assert!((x(&doc) - first - 40.0).abs() < 0.01, "the edit shows at once: {} → {}", first, x(&doc));
        let owner = doc.begin_preview();
        doc.preview_edits(owner, &[Intent::SetConstant { layer: row, property: PropertyId::new(GAP).unwrap(), value: Value::F64(0.0) }]).unwrap();
        assert!((x(&doc) - (first - 10.0)).abs() < 0.01, "a preview is laid out fresh, not from the cache: {}", x(&doc));
        doc.clear_preview_edits(owner);
        assert!((x(&doc) - first - 40.0).abs() < 0.01, "and after the preview the committed layout is back");
        doc.set_transient(row, PropertyId::new(GAP).unwrap(), Value::F64(0.0));
        let shown = doc.view();
        let transient = motolii_render::picture::frame::layout_frame(&shown, T).unwrap();
        let committed = motolii_render::picture::frame::layout_frame(&shown.clone().without_transients(), T).unwrap();
        assert!((committed.slots[&b].position[0] - first - 40.0).abs() < 0.01,
            "excluding transients must not reuse the shown layout");
        assert!((transient.slots[&b].position[0] - (first - 10.0)).abs() < 0.01);
        assert_eq!(motolii_render::picture::frame::layout_frame(&shown, T).unwrap(), transient,
            "the original view keeps its own evaluation inputs");
        let owner = doc.begin_preview();
        doc.preview_edits(owner, &[Intent::SetConstant { layer: row, property: PropertyId::new(GAP).unwrap(), value: Value::F64(25.0) }]).unwrap();
        let unchanged = motolii_render::picture::frame::layout_frame(&doc.view().without_transients(), T).unwrap();
        assert!(std::sync::Arc::ptr_eq(&committed, &unchanged),
            "a committed read must reuse its layout while excluded previews change");
    }

    #[test]
    fn a_label_anchored_to_a_thing_sits_on_the_side_it_names_and_follows() {
        let mut doc = blank_project();
        let thing = add(&mut doc, 1, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: thing, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
        put(&mut doc, thing, property::POSITION, Value::Vec2([500.0, 300.0]));
        let label = add(&mut doc, 2, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: label, shapes: vec![rect_shape([255; 4], [40.0, 20.0])] }).unwrap();
        put(&mut doc, label, property::POSITION, Value::Vec2([10.0, 10.0]));
        put(&mut doc, label, MARGIN, Value::F64(10.0));
        put(&mut doc, label, POSITION_ANCHOR, Value::LayerId(thing.0));
        put(&mut doc, label, POSITION_AREA, Value::Enum(2));
        let (a, b) = (shown(&doc, thing, T), shown(&doc, label, T));
        assert!((b[3] - (a[1] - 10.0)).abs() < 0.01, "Top: its bottom edge a margin above the thing's top: {a:?} {b:?}");
        assert!(((b[0] + b[2]) * 0.5 - (a[0] + a[2]) * 0.5).abs() < 0.01, "centred along the thing");
        put(&mut doc, label, POSITION_AREA, Value::Enum(6));
        let b = shown(&doc, label, T);
        assert!((b[0] - (a[2] + 10.0)).abs() < 0.01 && ((b[1] + b[3]) * 0.5 - (a[1] + a[3]) * 0.5).abs() < 0.01, "Right: beside it, middles level: {b:?}");
        put(&mut doc, thing, property::POSITION, Value::Vec2([800.0, 600.0]));
        let (a, b) = (shown(&doc, thing, T), shown(&doc, label, T));
        assert!((b[0] - (a[2] + 10.0)).abs() < 0.01, "moving the thing carries the label");
        let placed = shown(&doc, label, T);
        put(&mut doc, label, POSITION_AREA, Value::Enum(0));
        let back = shown(&doc, label, T);
        assert!(back != placed && (back[0] + back[2]) * 0.5 < 100.0, "None: back where it was written: {back:?}");
    }

    #[test]
    fn a_free_child_snaps_to_the_nearest_field_of_its_grid() {
        let mut doc = blank_project();
        let grid = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(2)), (GRID_COLUMNS, Value::F64(4.0)), (GRID_ROWS, Value::F64(4.0)), (GAP, Value::F64(20.0)),
            (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(860.0)), (HEIGHT, Value::F64(860.0))] {
            put(&mut doc, grid, name, value);
        }
        put(&mut doc, grid, property::POSITION, Value::Vec2([100.0, 50.0]));
        let card = rect(&mut doc, 2, grid, [150.0, 120.0]);
        put(&mut doc, card, POSITION_TYPE, Value::Enum(1));
        put(&mut doc, card, property::POSITION, Value::Vec2([250.0, 470.0]));
        let free = shown(&doc, card, T);
        put(&mut doc, card, SNAP_TO_GRID, Value::F64(1.0));
        let snapped = shown(&doc, card, T);
        // 升目は 200 px + 溝 20 px: 画面の上で列は Group の Position から 220 px ごとに始まる。
        let starts_x: Vec<f32> = (0..4).map(|i| 100.0 + i as f32 * 220.0).collect();
        let starts_y: Vec<f32> = (0..4).map(|i| 50.0 + i as f32 * 220.0).collect();
        let near = |v: f32, list: &[f32]| list.iter().cloned().min_by(|a, b| (a - v).abs().total_cmp(&(b - v).abs())).unwrap();
        assert!((snapped[0] - near(free[0], &starts_x)).abs() < 0.5 && (snapped[1] - near(free[1], &starts_y)).abs() < 0.5,
            "the box's corner lands on the nearest field's corner: free {free:?} snapped {snapped:?}");
        assert!(((snapped[2] - snapped[0]) - (free[2] - free[0])).abs() < 0.5, "Size Off keeps the size");

        put(&mut doc, card, SNAP_SIZE, Value::Enum(1));
        let fitted = shown(&doc, card, T);
        assert!(((fitted[2] - fitted[0]) - 200.0).abs() < 1.0 && ((fitted[3] - fitted[1]) - 200.0).abs() < 1.0, "Size Fields: one whole field: {fitted:?}");

        put(&mut doc, card, SNAP_SIZE, Value::Enum(0));
        put(&mut doc, card, SNAP_TO_GRID, Value::F64(0.5));
        let half = shown(&doc, card, T);
        assert!((half[0] - (free[0] + snapped[0]) * 0.5).abs() < 0.5, "half strength goes half way");
    }

    #[test]
    fn a_transform_origin_keeps_its_corner_on_the_position_as_the_box_grows() {
        let mut doc = blank_project();
        let card = add(&mut doc, 1, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: card, shapes: vec![rect_shape([255; 4], [100.0, 60.0])] }).unwrap();
        put(&mut doc, card, property::POSITION, Value::Vec2([400.0, 300.0]));
        put(&mut doc, card, TRANSFORM_ORIGIN, Value::Enum(7));
        let b = shown(&doc, card, T);
        assert!((b[0] - 400.0).abs() <= 1.01 && (b[3] - 300.0).abs() <= 1.01, "Bottom Left: that corner sits on the Position: {b:?}");
        put(&mut doc, card, property::SHAPE_SIZE, Value::Vec2([300.0, 200.0]));
        let grown = shown(&doc, card, T);
        assert!((grown[0] - 400.0).abs() <= 1.01 && (grown[3] - 300.0).abs() <= 1.01, "the box grows up and to the right, the corner stays: {grown:?}");
        put(&mut doc, card, property::SCALE, Value::Vec2([2.0, 2.0]));
        let scaled = shown(&doc, card, T);
        assert!((scaled[0] - 400.0).abs() <= 1.01 && (scaled[3] - 300.0).abs() <= 1.01 && (scaled[2] - scaled[0] - 600.0).abs() < 1.0, "and scaling grows from that corner: {scaled:?}");
        put(&mut doc, card, TRANSFORM_ORIGIN, Value::Enum(5));
        let centred = shown(&doc, card, T);
        assert!(((centred[0] + centred[2]) * 0.5 - 400.0).abs() <= 1.01, "Center: the middle sits on the Position: {centred:?}");
    }

    #[test]
    fn free_children_follow_the_parents_edges_by_their_constraints() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let panel = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(1)), (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (HEIGHT, Value::F64(300.0))] {
            put(&mut doc, panel, name, value);
        }
        let mut track = motolii_doc::eval::KeyframeTrack::new();
        track.insert(motolii_doc::eval::Keyframe { t: at(0), value: Value::F64(400.0), interp: motolii_doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(motolii_doc::eval::Keyframe { t: at(30), value: Value::F64(600.0), interp: motolii_doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: panel, property: PropertyId::new(WIDTH).unwrap(), track }).unwrap();
        let badge = rect(&mut doc, 2, panel, [40.0, 40.0]);
        let bar = rect(&mut doc, 3, panel, [300.0, 20.0]);
        for (layer, position, mode) in [(badge, [360.0, 40.0], 1), (bar, [200.0, 280.0], 2)] {
            put(&mut doc, layer, POSITION_TYPE, Value::Enum(1));
            put(&mut doc, layer, property::POSITION, Value::Vec2(position));
            put(&mut doc, layer, HORIZONTAL_CONSTRAINT, Value::Enum(mode));
        }
        let (b0, b30) = (shown(&doc, badge, at(0)), shown(&doc, badge, at(30)));
        assert!(((b30[0] - b0[0]) - 200.0).abs() < 0.5 && ((b30[2] - b30[0]) - (b0[2] - b0[0])).abs() < 0.5, "Right: keeps its distance from the right edge: {b0:?} {b30:?}");
        let (r0, r30) = (shown(&doc, bar, at(0)), shown(&doc, bar, at(30)));
        assert!((r30[0] - r0[0]).abs() < 0.5 && ((r30[2] - r0[2]) - 200.0).abs() < 0.5, "Left & Right: both distances kept, it stretches: {r0:?} {r30:?}");
        put(&mut doc, badge, HORIZONTAL_CONSTRAINT, Value::Enum(3));
        let c30 = shown(&doc, badge, at(30));
        assert!(((c30[0] - b0[0]) - 100.0).abs() < 0.5, "Center: moves half as much: {c30:?}");
    }

    #[test]
    fn an_object_travels_the_border_box_of_its_parent_and_turns_with_it() {
        let mut doc = blank_project();
        let card = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(1)), (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(400.0)), (HEIGHT, Value::F64(200.0))] {
            put(&mut doc, card, name, value);
        }
        let dot = rect(&mut doc, 2, card, [10.0, 10.0]);
        put(&mut doc, dot, POSITION_TYPE, Value::Enum(1));
        put(&mut doc, dot, OFFSET_PATH, Value::Enum(1));
        let at = |doc: &mut Document, percent: f64| {
            put(doc, dot, OFFSET_DISTANCE, Value::F64(percent));
            let view = doc.view();
            (path::on_offset_path(&view, dot, T).unwrap().unwrap(), path::offset_rotation(&view, dot, T).unwrap())
        };
        let ((p, _), turn) = at(&mut doc, 25.0);
        assert!((p[0] - (1.0 + 300.0)).abs() < 0.01 && (p[1] - 1.0).abs() < 0.01 && turn.abs() < 0.01, "a quarter of 1200 px is 300 px along the top: {p:?}");
        let ((p, _), turn) = at(&mut doc, 40.0);
        assert!((p[0] - 401.0).abs() < 0.01 && (p[1] - (1.0 + 80.0)).abs() < 0.01 && (turn - 90.0).abs() < 0.01, "down the right side, turned to face down: {p:?} {turn}");
        let ((p, _), _) = at(&mut doc, 125.0);
        assert!((p[0] - 301.0).abs() < 0.01, "past 100% it goes round again");
        put(&mut doc, card, BORDER_RADIUS, Value::F64(50.0));
        let ((p, _), turn) = at(&mut doc, 0.0);
        assert!((p[0] - 51.0).abs() < 0.01 && turn.abs() < 0.01, "with round corners the path starts after the corner");
        // 角を曲がる間も滑らかに(1% ずつで大きく跳ばない)。
        let mut previous: Option<[f32; 2]> = None;
        for k in 0..=100 {
            let ((p, _), _) = at(&mut doc, k as f64);
            if let Some(q) = previous {
                assert!(((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt() < 12.0, "no jump around the corners at {k}%");
            }
            previous = Some(p);
        }
        let shown_at = shown(&doc, dot, T);
        assert!(shown_at[0] < 60.0, "the object is drawn where the path puts it: {shown_at:?}");
    }

    #[test]
    fn a_field_box_swells_what_is_near_it_and_leaves_the_far_alone() {
        let mut doc = blank_project();
        let lens = add(&mut doc, 1, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: lens, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
        put(&mut doc, lens, property::POSITION, Value::Vec2([500.0, 300.0]));
        let near = add(&mut doc, 2, LayerSource::Shape, None);
        let far = add(&mut doc, 3, LayerSource::Shape, None);
        for (layer, at) in [(near, [540.0, 300.0]), (far, [1200.0, 300.0])] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [20.0, 20.0])] }).unwrap();
            put(&mut doc, layer, property::POSITION, Value::Vec2(at));
            put(&mut doc, layer, FIELD, Value::LayerId(lens.0));
            put(&mut doc, layer, FIELD_SCALE, Value::F64(3.0));
            put(&mut doc, layer, FIELD_FALLOFF, Value::F64(150.0));
        }
        let (n, f) = (shown(&doc, near, T), shown(&doc, far, T));
        assert!(((n[2] - n[0]) - 60.0).abs() < 1.0, "inside the field box: full strength, three times the size: {n:?}");
        assert!(((f[2] - f[0]) - 20.0).abs() < 0.5, "beyond the falloff: untouched: {f:?}");
        put(&mut doc, near, property::POSITION, Value::Vec2([625.0, 300.0]));
        let half = shown(&doc, near, T);
        assert!((half[2] - half[0]) > 21.0 && (half[2] - half[0]) < 59.0, "in the falloff: part way: {half:?}");
        put(&mut doc, near, FIELD_SCALE, Value::F64(1.0));
        put(&mut doc, near, property::POSITION, Value::Vec2([540.0, 300.0]));
        let still = shown(&doc, near, T);
        put(&mut doc, near, FIELD_PUSH, Value::F64(40.0));
        let pushed = shown(&doc, near, T);
        let lens_box = shown(&doc, lens, T);
        let away = glam::vec2((still[0] + still[2]) * 0.5 - (lens_box[0] + lens_box[2]) * 0.5, (still[1] + still[3]) * 0.5 - (lens_box[1] + lens_box[3]) * 0.5).normalize();
        let moved = glam::vec2((pushed[0] + pushed[2] - still[0] - still[2]) * 0.5, (pushed[1] + pushed[3] - still[1] - still[3]) * 0.5);
        assert!((moved - away * 40.0).length() < 1.5, "pushed 40 px away from the field's centre: {moved:?} along {away:?}");
    }

    #[test]
    fn a_picture_takes_the_size_the_host_measured() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let picture = add(&mut doc, 2, LayerSource::File { path: "/tmp/photo.png".to_owned(), fingerprint: None }, Some(group));
        let after = rect(&mut doc, 3, group, [40.0, 40.0]);
        let mut inputs = motolii_doc::store::analysis::AnalysisInputs::default();
        inputs.set_extent("/tmp/photo.png", [320.0, 180.0, 0.0]);
        let view = doc.view().with_analysis(&inputs);
        let frame = motolii_render::picture::frame::layout_frame(&view, T).unwrap();
        assert_eq!(motolii_render::picture::boxes::layer_box(&view, picture, T).unwrap(), Some([0.0, 0.0, 320.0, 180.0]));
        // 箱の左端 = 位置 - 中心 + 箱の左(形の素材座標は輪郭の canvas の 1 画素外が原点、画は 0)。
        let left = |id: LayerId, min: f32| frame.slots[&id].position[0] - frame.slots[&id].anchor[0] + min;
        assert_eq!(left(after, 1.0), left(picture, 0.0) + 320.0 + 10.0, "the next item starts after the picture and the gap");
    }

    #[test]
    fn wrapping_text_flows_around_a_sibling_that_declares_its_shape() {
        let mut doc = blank_project();
        let words = add(&mut doc, 2, LayerSource::Text, None);
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: T, content: "Shibuya crossing at nine in the evening, the lights change and three thousand people walk at once across the white lines".to_owned() });
        let style = TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
            size: 40.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
        };
        let comp = doc.view().composition().unwrap().unwrap();
        doc.apply(Intent::SetTextDocument { layer: words, document: TextDocument {
            content: track, justify: TextJustify::Left, wrap_size: Some([600.0, comp.height as f32]), styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        } }).unwrap();
        let object = add(&mut doc, 3, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: object, shapes: vec![rect_shape([255; 4], [160.0, 160.0])] }).unwrap();
        let middle = motolii_render::picture::boxes::world_2d(&doc.view(), words, T).unwrap().transform_point2(glam::vec2(300.0, comp.height as f32 * 0.5));
        put(&mut doc, object, property::POSITION, Value::Vec2([middle.x as f64, middle.y as f64]));
        let canvas = motolii_render::picture::shapes_ops::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        // 物の占める範囲(文字の枠の座標)と重なる字の数。
        let overlapping = |doc: &Document| {
            let view = doc.view();
            let resolved = motolii_render::picture::resolve::resolved_layers(&view, T).unwrap();
            let around = resolved.iter().find(|l| l.id == words).unwrap().flow_around.clone().expect("the object is declared");
            let (lo, hi) = around.iter().flat_map(|o| o.points.iter()).fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(glam::Vec2::from(*p)), hi.max(glam::Vec2::from(*p))));
            let document = motolii_render::picture::resolve::text::resolved_text_document(&view, words, T).unwrap().unwrap();
            let count = |shaped: &motolii_doc::vector::text::ShapedText| shaped.contours.iter().filter(|c| {
                let (a, b) = c.vertices.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(a, b), v| (a.min(glam::vec2(v.point.x as f32, v.point.y as f32)), b.max(glam::vec2(v.point.x as f32, v.point.y as f32))));
                a.x < hi.x && b.x > lo.x && a.y < hi.y && b.y > lo.y
            }).count();
            let plain = motolii_render::picture::text_frame::shape_document(&document, T, &canvas).unwrap().unwrap();
            let flowed = motolii_render::picture::text_frame::shape_document_around(&document, T, &canvas, &around).unwrap().unwrap();
            let right_of = flowed.contours.iter().any(|c| c.vertices.iter().all(|v| v.point.x as f32 > hi.x && (v.point.y as f32) > lo.y && (v.point.y as f32) < hi.y));
            (count(&plain), count(&flowed), plain.contours.len() == flowed.contours.len(), right_of)
        };
        put(&mut doc, object, SHAPE_OUTSIDE, Value::Enum(2));
        assert!(motolii_render::picture::resolve::resolved_layers(&doc.view(), T).unwrap().iter().find(|l| l.id == words).unwrap().flow_around.is_some());
        let (before, after, all_glyphs, both_sides) = overlapping(&doc);
        assert!(before > 0, "without flowing, the words run under the object");
        assert_eq!(after, 0, "the words flow around it");
        assert!(all_glyphs, "no word is lost");
        assert!(both_sides, "lines continue on the other side of the object (wrap-flow: both)");

        put(&mut doc, object, SHAPE_OUTSIDE, Value::Enum(0));
        assert!(motolii_render::picture::resolve::resolved_layers(&doc.view(), T).unwrap().iter().find(|l| l.id == words).unwrap().flow_around.is_none(), "None declares nothing");
    }

    #[test]
    fn a_growing_line_of_text_pushes_its_neighbour() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let name = add(&mut doc, 2, LayerSource::Text, Some(group));
        let icon = rect(&mut doc, 3, group, [40.0, 40.0]);
        let x = |doc: &mut Document, content: &str| {
            let mut track = ContentTrack::new();
            track.insert(ContentKeyframe { t: T, content: content.to_owned() });
            let style = TextDocumentStyle {
                id: TextStyleId(0),
                font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
                size: 48.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
            };
            doc.apply(Intent::SetTextDocument { layer: name, document: TextDocument {
                content: track, justify: TextJustify::Left, wrap_size: None, styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
            } }).unwrap();
            shown(doc, icon, T)[0]
        };
        let short = x(&mut doc, "Taro");
        let long = x(&mut doc, "Taro Yamada");
        assert!(long > short + 50.0, "the icon moves right as the name grows: {short} → {long}");
        let text = shown(&doc, name, T);
        assert_eq!((text[0], text[1]), (20.0, 8.0), "the line box sits at the padding");
    }
}




