mod plane_tests {
    use crate::ui::stage_widget::*;

    // 隅だけ合っていても、**内側がずれていれば掴めない**。描く側と掴む側は
    // この1枚を共有しているので、任意の傾きの任意の点で往復できなければ、
    // 絵と当たり判定が別の場所に居る。
    //
    // 生成する四角形は角度を回り順にして凸に保つ。潰れた四角形では写像が
    // 定義できないので、面積に下限を置く。
    proptest::proptest! {
        #[test]
        fn any_point_on_any_tilted_plane_comes_back_where_it_started(
            cx in -500.0f64..500.0,
            cy in -500.0f64..500.0,
            radii in proptest::array::uniform4(30.0f64..400.0),
            wobble in proptest::array::uniform4(-0.6f64..0.6),
            probes in proptest::collection::vec((0.0f64..1.0, 0.0f64..1.0), 1..12),
        ) {
            let corner = |i: usize| {
                let a = std::f64::consts::FRAC_PI_4 + i as f64 * std::f64::consts::FRAC_PI_2 + wobble[i];
                glam::dvec2(cx + radii[i] * a.cos(), cy + radii[i] * a.sin())
            };
            let p = [corner(0), corner(1), corner(2), corner(3)];

            let area: f64 = (0..4)
                .map(|i| {
                    let (a, b) = (p[i], p[(i + 1) % 4]);
                    a.x * b.y - b.x * a.y
                })
                .sum::<f64>()
                .abs()
                * 0.5;
            proptest::prop_assume!(area > 2_000.0);

            let m = homography_from_unit_square(p);
            let inv = m.inverse();
            for (u, v) in probes {
                let (sx, sy) = apply_h(&m, u, v);
                proptest::prop_assume!(sx.is_finite() && sy.is_finite());
                let (bu, bv) = apply_h(&inv, sx, sy);
                proptest::prop_assert!(
                    (bu - u).abs() < 1e-6 && (bv - v).abs() < 1e-6,
                    "({u:.3},{v:.3}) が ({bu:.3},{bv:.3}) で戻ってきた。隅={p:?}"
                );
            }
        }
    }
}

mod gizmo_reach {
    use crate::ui::stage_widget::*;

    // **箱の真ん中はいつでも動かせなければならない。**
    //
    // 取っ手の許容は画面の画素で決まるので、画面上で小さい層では許容が箱より
    // 広くなる。実測: 1920 の枠を 198px で映すと 64px の層は画面上 6.6px、
    // 角の許容は 77 comp px —— 箱(64)より広く、掴んだ所は必ず角になる。
    // 試し手はこれで「動かず巨大化した」。
    proptest::proptest! {
        #[test]
        fn the_middle_of_the_box_is_always_a_move(
            // **画面上の大きさで振る。** comp の大きさで振ると、画面で小さく
            // 映る組み合わせが千に一つしか出ず、当たりの領域を素通りする。
            screen_w in 0.5f64..400.0,
            screen_h in 0.5f64..400.0,
            bx in -2000.0f64..2000.0,
            by in -2000.0f64..2000.0,
            scale in 0.01f64..8.0,
            rings in proptest::bool::ANY,
        ) {
            let (bw, bh) = (screen_w / scale, screen_h / scale);
            let mode = gizmo_mode_at(bx + bw * 0.5, by + bh * 0.5, (bx, by, bw, bh), scale, rings);
            proptest::prop_assert_eq!(
                mode,
                Some(GizmoMode::Move),
                "画面上 {:.1}x{:.1} px の箱の真ん中が掴めない(拡大率 {:.3})",
                screen_w, screen_h, scale
            );
        }

        /// 片側だけ守っても道具にならない。**画面で十分に大きい箱では、角は
        /// 角のまま**掴めなければ拡縮ができない。
        #[test]
        fn a_big_enough_box_still_has_grabbable_corners(
            screen_w in 80.0f64..2000.0,
            screen_h in 80.0f64..2000.0,
            bx in -2000.0f64..2000.0,
            by in -2000.0f64..2000.0,
            scale in 0.05f64..8.0,
        ) {
            let (bw, bh) = (screen_w / scale, screen_h / scale);
            let mode = gizmo_mode_at(bx, by, (bx, by, bw, bh), scale, false);
            proptest::prop_assert_eq!(
                mode,
                Some(GizmoMode::ScaleCorner { sx: false, sy: false }),
                "画面上 {:.0}x{:.0} px の箱の角が掴めない(拡大率 {:.3})",
                screen_w, screen_h, scale
            );
        }
    }
}

mod previews {
    use crate::ui::stage_widget::*;

    fn drag(mode: GizmoMode) -> GizmoDrag {
        GizmoDrag {
            layer: LayerId(1),
            mode,
            grab: (100.0, 100.0),
            orig_position: (100.0, 100.0),
            orig_rotation: 0.0,
            orig_rotation_xy: (0.0, 0.0),
            orig_z: 0.0,
            anchor: (0.5, 0.5),
            natural: (200.0, 100.0),
            orig_box: (0.0, 50.0, 200.0, 100.0),
            fit_z: 0.0,
            last: None,
            preview: Vec::new(),
            others: vec![(LayerId(2), (300.0, 300.0))],
        }
    }

    /// Shift は支配軸に固定し、一緒に選んだ層も同じ差分で動く。
    #[test]
    fn shift_locks_the_dominant_axis_and_carries_companions() {
        let out = preview_values(&drag(GizmoMode::Move), (130.0, 108.0), true, false, 1.0);
        assert_eq!(out[0], (LayerId(1), property::POSITION, Value::Vec2([130.0, 100.0])));
        assert_eq!(out[1], (LayerId(2), property::POSITION, Value::Vec2([330.0, 300.0])));
    }

    /// 軌道は掴んだ輪の軸だけ書く。上へ引くと奥へ。
    #[test]
    fn orbit_touches_one_axis_and_depth_goes_up() {
        let out = preview_values(&drag(GizmoMode::Orbit { axis_x: true }), (100.0, 140.0), false, false, 1.0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].1, property::ROTATION_X);
        let out = preview_values(&drag(GizmoMode::Depth), (100.0, 60.0), false, false, 1.0);
        assert_eq!(out[0], (LayerId(1), property::POSITION_Z, Value::F64(40.0)));
    }

    /// 矢印は今の位置に差分を足す。
    #[test]
    fn nudge_adds_to_the_current_position() {
        let loaded = crate::ui::fixture::load_fixture();
        let mut doc = loaded.doc;
        let layer = doc.view().layers()[0];
        let prop = PropertyId::new(property::POSITION).unwrap();
        let before = match doc.view().value_at(layer, &prop, RationalTime::ZERO).unwrap() {
            Some(Value::Vec2(v)) => v,
            _ => [0.0, 0.0],
        };
        doc.apply_all(nudge_intents(&doc, &[layer], (10.0, -1.0), RationalTime::ZERO)).unwrap();
        let after = doc.view().value_at(layer, &prop, RationalTime::ZERO).unwrap();
        assert_eq!(after, Some(Value::Vec2([before[0] + 10.0, before[1] - 1.0])));
    }
}
