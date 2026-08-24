    // `parse_number` は裁定160 切片5(pane split survey §2.4/§6)で
    // `chrome::parse_number` へ関数本体を移設し、切片9で `chrome` ごと
    // `motolii-settings-pane` crate へ、切片8でこの crate 自身の依存先へ
    // 移った(モジュール冒頭の `use motolii_settings_pane::chrome::{parse_number,
    // ..}` で読み込み済み、`use super::*;` 経由でここへ入る)。テストの
    // qualified name(`tests::parse_number_accepts_the_mock_minus_sign`)は
    // `--list` 完全一致のためここに残す — 呼ぶ本体だけ移設先を指す。

    // -----------------------------------------------------------------------
    // BL2: Blend 巡回ボタンの次の値
    // -----------------------------------------------------------------------

    /// Normal→Add→Multiply→…→Exclusion→Normal と、宣言順どおり13値を一周して戻る
    /// (**BL3** — `SUPPORTED_BLEND_MODES` を更新したらこのテストが長さのズレを拾う)。
    #[test]
    fn cycles_through_supported_modes_and_wraps() {
        use motolii_store::BlendMode;
        assert_eq!(SUPPORTED_BLEND_MODES.len(), 13);
        assert_eq!(next_blend_mode(BlendMode::Normal), BlendMode::Add);
        assert_eq!(next_blend_mode(BlendMode::Add), BlendMode::Multiply);
        assert_eq!(next_blend_mode(BlendMode::Multiply), BlendMode::Screen);
        assert_eq!(next_blend_mode(BlendMode::Screen), BlendMode::Overlay);
        assert_eq!(next_blend_mode(BlendMode::Overlay), BlendMode::Darken);
        assert_eq!(next_blend_mode(BlendMode::Darken), BlendMode::Lighten);
        assert_eq!(next_blend_mode(BlendMode::Lighten), BlendMode::ColorDodge);
        assert_eq!(next_blend_mode(BlendMode::ColorDodge), BlendMode::ColorBurn);
        assert_eq!(next_blend_mode(BlendMode::ColorBurn), BlendMode::HardLight);
        assert_eq!(next_blend_mode(BlendMode::HardLight), BlendMode::SoftLight);
        assert_eq!(next_blend_mode(BlendMode::SoftLight), BlendMode::Difference);
        assert_eq!(next_blend_mode(BlendMode::Difference), BlendMode::Exclusion);
        assert_eq!(next_blend_mode(BlendMode::Exclusion), BlendMode::Normal);
    }

    /// 現在値が非対応(将来の下位互換ケース)なら、エラーにせず一覧の先頭へ。
    /// **BL3** で Multiply は対応済みになったので、まだ非対応な非分離4種(BL4)の
    /// `Hue` で確かめる。
    #[test]
    fn unsupported_current_value_falls_back_to_the_first_supported_mode() {
        use motolii_store::BlendMode;
        assert_eq!(next_blend_mode(BlendMode::Hue), BlendMode::Normal);
    }

    // -----------------------------------------------------------------------
    // B38 第3切片: Glow param カタログ(engine 同期義務の柵)
    // -----------------------------------------------------------------------

    /// カタログの3点固定: 名前は engine `translate_glow_params` の `find` 名、
    /// 既定値は engine の `GLOW_DEFAULT_*`(private const)の写し —
    /// [`SUPPORTED_BLEND_MODES`] と同じ二重化なので、engine 側を変えたら
    /// ここが red になって同期漏れを拾う(値の正本は engine 側)。
    #[test]
    fn the_glow_param_catalog_mirrors_the_engine_names_and_defaults() {
        assert_eq!(GlowParam::ALL.len(), 3);
        assert_eq!(GlowParam::Threshold.name(), "threshold");
        assert_eq!(GlowParam::Intensity.name(), "intensity");
        assert_eq!(GlowParam::Radius.name(), "radius");
        assert_eq!(GlowParam::Threshold.default_value(), 1.0);
        assert_eq!(GlowParam::Intensity.default_value(), 0.75);
        assert_eq!(GlowParam::Radius.default_value(), 1.0);
    }

    /// 既知 plugin(`motolii.glow`)だけカタログと表示名を持ち、未知は
    /// param 行ゼロ + plugin_id そのまま(M13: 捏造しない)。
    #[test]
    fn plugin_catalog_and_display_name_are_honest_about_unknown_plugins() {
        assert_eq!(plugin_params(GLOW_PLUGIN_ID).len(), 3);
        assert!(plugin_params("third-party.sparkle").is_empty());
        assert_eq!(plugin_display_name(GLOW_PLUGIN_ID), "Glow");
        assert_eq!(
            plugin_display_name("third-party.sparkle"),
            "third-party.sparkle"
        );
    }

    /// effect param の field/KeyRow → property の対応が
    /// `effect.{id}.param.{name}` に落ちる(mask opacity の対応固定と同型)。
    #[test]
    fn effect_param_fields_and_key_rows_map_to_the_flat_effect_property() {
        let expected =
            PropertyId::effect_param(EffectId(7), "radius").expect("param 名は非予約語");
        assert_eq!(
            property_id(TransformField::EffectParam(EffectId(7), GlowParam::Radius))
                .expect("作れるはず"),
            expected
        );
        assert_eq!(
            key_row_property_id(KeyRow::EffectParam(EffectId(7), GlowParam::Radius))
                .expect("作れるはず"),
            expected
        );
        assert_eq!(
            key_row_default_value(KeyRow::EffectParam(EffectId(7), GlowParam::Intensity)),
            Value::F64(0.75),
            "Key 列の初キー値も engine 既定の写しのはず"
        );
    }

    /// ラベル色チップの1辺は timeline rail のチップ式(`round(0.462 × 行高)`)
    /// と同じで、**`inspector_glyph_width`(26px)とは一致しない** — shell 側
    /// `inspector_pixel_fence` の glyph 数え上げ(M 1個 + Key 5個 = 6個)を
    /// 壊さないための幾何の柵。
    #[test]
    fn the_label_chip_side_follows_the_timeline_swatch_formula_not_the_glyph_width() {
        let dims = Dimensions::default();
        let side = label_chip_side(dims.inspector_row_height);
        assert_eq!(side, (dims.inspector_row_height * 0.462).round());
        assert_ne!(
            side, dims.inspector_glyph_width,
            "チップが glyph 幅と同寸だと pixel fence の数え上げに紛れ込む"
        );
        assert_ne!(
            side,
            glyph_height(dims),
            "チップ高が glyph 高と同じでも幅26px側の柵対象になり得る(正方形なので両辺を外す)"
        );
    }

    // -----------------------------------------------------------------------
    // SP1 第一波: %⇄Speed 写像(ORACLE (b))
    // -----------------------------------------------------------------------

    /// 往復: 表示 % → (num, den) → 表示 % が同じ値へ戻る(小数1桁)。
    #[test]
    fn percent_round_trips_through_speed_ratio() {
        let (num, den) = percent_to_speed_ratio(200.0).unwrap();
        assert_eq!(format_number(speed_percent(num, den), 1), "200.0");

        let (num, den) = percent_to_speed_ratio(133.3).unwrap();
        assert_eq!(format_number(speed_percent(num, den), 1), "133.3");

        let (num, den) = percent_to_speed_ratio(50.0).unwrap();
        assert_eq!(format_number(speed_percent(num, den), 1), "50.0");
    }

    /// 分母は常に正(`Speed::try_new` の不変式を機械的に満たす)。
    #[test]
    fn speed_ratio_denominator_is_always_positive() {
        let (_, den) = percent_to_speed_ratio(100.0).unwrap();
        assert!(den > 0);
    }

    /// **0 は拒否**(決定3)。負・NaN・無限大も同様。
    #[test]
    fn non_positive_or_non_finite_percent_is_rejected() {
        assert_eq!(percent_to_speed_ratio(0.0), None);
        assert_eq!(percent_to_speed_ratio(-5.0), None);
        assert_eq!(percent_to_speed_ratio(f64::NAN), None);
        assert_eq!(percent_to_speed_ratio(f64::INFINITY), None);
    }

    /// 100% は `Speed::NORMAL`(1/1)と同じ比。
    #[test]
    fn one_hundred_percent_is_normal_speed() {
        let (num, den) = percent_to_speed_ratio(100.0).unwrap();
        assert_eq!(num as f64 / den as f64, 1.0);
    }

    // -----------------------------------------------------------------------
    // 裁定139/裁定168: value_cell/name_field は縦0を維持したまま横だけ内余白
    // (0.6em)を戻す
    // -----------------------------------------------------------------------

    /// **本命(red→green の柵)**: 旧実装は `.padding(0.0)` で縦横とも0だった
    /// (`git log` 参照 — このテストを旧コードに当てると
    /// `padding.left == 0.0`/`padding.right == 0.0` が真になり fail する)。
    /// 縦は行高合わせのため0のまま、横だけ 裁定168 の `0.6em`
    /// (`dims.body_text * 0.6` の最近傍丸め)が入っていること。旧実装は
    /// `spacing_xs`(mock `--sp1`=2px)を転用していたが、裁定168 施工で
    /// この式へ差し替えた(既定 dims では 11*0.6=6.6→7.0px、旧値2pxより広い)。
    #[test]
    fn value_cell_padding_keeps_the_vertical_zero_and_restores_only_horizontal_inset() {
        let dims = Dimensions::default();
        let padding = value_cell_padding(dims);
        let expected = single_row_horizontal_inset(dims.body_text);
        assert_eq!(padding.top, 0.0, "縦(上)は行高合わせのため0のはず");
        assert_eq!(padding.bottom, 0.0, "縦(下)は行高合わせのため0のはず");
        assert_eq!(
            padding.left, expected,
            "横(左)の内余白が裁定168 の0.6emと違う"
        );
        assert_eq!(
            padding.right, expected,
            "横(右)の内余白が裁定168 の0.6emと違う"
        );
        assert!(padding.left > 0.0, "横の内余白が0のまま(旧バグの再発)");
        assert_eq!(
            expected, 7.0,
            "既定dims(body_text=11)での0.6em丸め値が想定と違う"
        );
    }

    #[test]
    fn name_field_padding_matches_value_cell_padding_the_same_way() {
        let dims = Dimensions::default();
        assert_eq!(name_field_padding(dims), value_cell_padding(dims));
    }

    /// 150%でも横内余白がスケールに追従すること(適用点は `Dimensions::scaled`
    /// の1箇所だけ、という裁定117の不変量をここでも保つ)。**丸めは
    /// スケール後の `body_text` に対して1回だけ行う** — 丸め前の値を先に
    /// スケールしてから丸めるのと数値が一致するとは限らない(丸めの非線形性、
    /// 既定 dims では 7.0*1.5=10.5 だが実際は round(16.5*0.6)=round(9.9)=10.0)。
    #[test]
    fn value_cell_padding_scales_with_ui_scale() {
        let dims = Dimensions::default().scaled(1.5);
        let padding = value_cell_padding(dims);
        assert_eq!(padding.left, single_row_horizontal_inset(dims.body_text));
    }

    // -----------------------------------------------------------------------
    // 裁定137: weight/ink の実使用箇所(.glyph/.ident)
    // -----------------------------------------------------------------------

    #[test]
    fn mute_glyph_uses_bold_800_weight() {
        // `mute_glyph` 自体は `Element` を返すので font を直接読み出せない —
        // `iced_selector::Target`(`Container`/`TextInput`/…)は style(色/font
        // /padding)を一切運ばない実測(`tests/inspector_pixel_fence.rs` 冒頭
        // 参照)なので、iced_test 経由でも実配線の font weight は照合できない。
        // ここは token 側の対応(`TextWeight::Bold` = 800)を固定するだけの
        // 薄い柵 — 実際に `.font(TextWeight::Bold.font())` へ繋がっている
        // ことは呼び出し箇所のコードレビュー相当でしか確認できない、正直な
        // 限界(`--screenshot` 器具は Stage+Timeline のみの手組み合成で
        // Inspector を一切描かない — `screenshot.rs` 実測、write-set 外の
        // finding として最終報告に記録)。
        assert_eq!(
            TextWeight::Bold.font().weight,
            iced::font::Weight::ExtraBold
        );
    }

    #[test]
    fn parse_number_accepts_the_mock_minus_sign() {
        assert_eq!(parse_number("−0.075"), Some(-0.075));
        assert_eq!(parse_number("12.5"), Some(12.5));
        assert_eq!(parse_number("  3  "), Some(3.0));
        assert_eq!(parse_number("not a number"), None);
    }

    #[test]
    fn format_number_respects_decimals() {
        assert_eq!(format_number(1.0, 3), "1.000");
        assert_eq!(format_number(24.0, 1), "24.0");
        assert_eq!(format_number(100.0, 0), "100");
    }

    /// 裁定169: 表示はセルに収まる精度へ落ちる(編集 draft は全精度のまま —
    /// [`value_cell`] の editing 分岐が `format_number` 直呼びであることが対)。
    /// **I-tokens(2026-08-22)で cap を6→11へ再較正** — `inspector_value_width`
    /// が38→64pxへ束で再転写されたため、旧アンカー値(960/3840)はもう clip の
    /// 実例にならない(64px セルなら全精度のまま収まる)。[`MAX_VALUE_CELL_CHARS`]
    /// のdoc に記載の新アンカー(11字/12字)に合わせて例を差し替える。
    #[test]
    fn display_number_shrinks_precision_to_fit_the_cell() {
        // 収まる値は field 既定精度のまま。
        assert_eq!(display_number(1.0, 3), "1.000");
        assert_eq!(display_number(0.0, 3), "0.000");
        // 旧アンカー値(960/3840)は新セル幅(64px)では clip されず全精度のまま
        // 収まる(旧cap=6時代は "960.00"/"3840.0"/"-960.0" へ短縮していた)。
        assert_eq!(display_number(960.0, 3), "960.000");
        assert_eq!(display_number(3840.0, 3), "3840.000");
        assert_eq!(display_number(-960.0, 3), "-960.000");
        // 新アンカー(MAX_VALUE_CELL_CHARS のdoc参照): 整数部7桁+小数3桁=11字は
        // ちょうど cap に収まる(境界そのもの)。
        assert_eq!(display_number(1234567.0, 3), "1234567.000");
        // 整数部8桁+小数3桁=12字は cap を超える → 小数を1桁落として11字へ。
        assert_eq!(display_number(12345678.0, 3), "12345678.00");
        // 整数部だけで上限超え: これ以上落とせない(clip(true) が防波堤)
        assert_eq!(display_number(123456789012.0, 0), "123456789012");
    }

    #[test]
    fn next_value_preserves_the_other_vec2_component() {
        assert_eq!(
            next_value(TransformField::PositionX, 5.0, [1.0, 2.0]),
            Value::Vec2([5.0, 2.0])
        );
        assert_eq!(
            next_value(TransformField::PositionY, 5.0, [1.0, 2.0]),
            Value::Vec2([1.0, 5.0])
        );
    }

    #[test]
    fn next_value_converts_opacity_percent_to_the_stored_fraction() {
        assert_eq!(
            next_value(TransformField::Opacity, 50.0, [0.0, 0.0]),
            Value::F64(0.5)
        );
        // クランプ: 100 を超える入力・負の入力は store の 0..1 に収める。
        assert_eq!(
            next_value(TransformField::Opacity, 150.0, [0.0, 0.0]),
            Value::F64(1.0)
        );
        assert_eq!(
            next_value(TransformField::Opacity, -10.0, [0.0, 0.0]),
            Value::F64(0.0)
        );
    }

    #[test]
    fn single_hold_track_has_exactly_one_hold_keyframe() {
        let track = single_hold_track(Value::F64(2.5));
        assert_eq!(track.keys().len(), 1, "静的値は1キーのはず");
        assert_eq!(track.keys()[0].value, Value::F64(2.5));
        assert!(matches!(track.keys()[0].interp, Interp::Hold));
    }

    #[test]
    fn default_vec2_is_identity_scale_and_zero_elsewhere() {
        assert_eq!(default_vec2(TransformField::ScaleX), [1.0, 1.0]);
        assert_eq!(default_vec2(TransformField::PositionX), [0.0, 0.0]);
        assert_eq!(default_vec2(TransformField::AnchorY), [0.0, 0.0]);
    }

    /// ident 帯の種別ラベルは `LayerSource` の実 variant から引く(mock の
    /// 「shared FX」件数のような捏造値ではない)。
    #[test]
    fn source_kind_label_covers_every_layer_source_variant() {
        assert_eq!(
            source_kind_label(&LayerSource::Solid {
                rgba: [0, 0, 0, 255],
                width: 1,
                height: 1,
            }),
            "solid"
        );
        assert_eq!(
            source_kind_label(&LayerSource::Media {
                path: "x.mp4".to_owned(),
                fingerprint: None,
            }),
            "media"
        );
        assert_eq!(source_kind_label(&LayerSource::Null), "null");
        assert_eq!(source_kind_label(&LayerSource::Shape), "shape");
        assert_eq!(source_kind_label(&LayerSource::Text), "text");
        assert_eq!(source_kind_label(&LayerSource::Group), "group");
    }

    // -----------------------------------------------------------------------
    // drag-to-scrub — 感度表(発注書の表そのもの)
    // -----------------------------------------------------------------------

    #[test]
    fn dragged_value_applies_the_registry_sensitivity_per_field() {
        // Position/Anchor/Z = 1px→1.0。
        assert_eq!(
            dragged_value(TransformField::PositionX, 0.0, 10.0, false),
            10.0
        );
        assert_eq!(
            dragged_value(TransformField::AnchorY, 0.0, -4.0, false),
            -4.0
        );
        assert_eq!(
            dragged_value(TransformField::PositionZ, 0.0, 3.0, false),
            3.0
        );
        // Scale = 1px→0.01。
        assert!((dragged_value(TransformField::ScaleX, 1.0, 10.0, false) - 1.1).abs() < 1e-9);
        // Rotation = 1px→0.5度。
        assert!((dragged_value(TransformField::Rotation, 0.0, 10.0, false) - 5.0).abs() < 1e-9);
        // Opacity = 1px→1(%)。
        assert_eq!(
            dragged_value(TransformField::Opacity, 50.0, 20.0, false),
            70.0
        );
    }

    #[test]
    fn shift_drag_uses_a_tenth_of_the_normal_sensitivity() {
        let normal = dragged_value(TransformField::PositionX, 0.0, 100.0, false);
        let fine = dragged_value(TransformField::PositionX, 0.0, 100.0, true);
        assert_eq!(normal, 100.0);
        assert!(
            (fine - 10.0).abs() < 1e-9,
            "Shift+drag は1/10のはず: {fine}"
        );
    }

    #[test]
    fn drag_origin_reads_the_projected_value_and_keeps_the_other_vec2_component() {
        // Scale の既定(un-keyed)は X=Y=1.0 — X をドラッグ対象にしても
        // current_vec2 の Y は保たれる。
        let selection = SelectionProjection {
            layer: LayerId(1),
            selection_count: 1,
            text_layer_count: 0,
            kind: "solid",
            transform: vec![TransformRowProjection {
                label: "Scale",
                value: RowValue::Vector([
                    ComponentSlot {
                        axis: "X",
                        present: true,
                        value: 1.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::ScaleX),
                    },
                    ComponentSlot {
                        axis: "Y",
                        present: true,
                        value: 2.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::ScaleY),
                    },
                    absent_component("Z"),
                ]),
                decimals: 3,
                key: KeyCellProjection {
                    row: KeyRow::Scale,
                    state: KeyCellState::Static,
                },
            }],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
                speed_percent: 100.0,
                label_color: None,
                matte: None,
                matte_candidates: vec![],
            },
            masks: vec![],
            effects: vec![],
            text: None,
            audio: None,
            shape: None,
            links: vec![],
        };

        let (start, current_vec2) =
            drag_origin(&selection, TransformField::ScaleX).expect("editable のはず");
        assert_eq!(start, 1.0);
        assert_eq!(current_vec2, [1.0, 2.0], "動かさない方(Y)を保っていない");

        // 対応する field が投影に無ければ `None`(呼び手はドラッグを始めない)。
        assert!(drag_origin(&selection, TransformField::Rotation).is_none());
    }

    /// キー持ち(keyed)の field も drag/type 編集の起点になる(Q0 —
    /// 2026-08-22 発注で旧規則「animated は編集不可」を撤去。編集の意味は
    /// [`edited_value_track`] のキー upsert)。
    #[test]
    fn drag_origin_accepts_keyed_fields() {
        let selection = SelectionProjection {
            layer: LayerId(1),
            selection_count: 1,
            text_layer_count: 0,
            kind: "solid",
            transform: vec![TransformRowProjection {
                label: "Rotation",
                value: RowValue::Vector([
                    absent_component("X"),
                    absent_component("Y"),
                    ComponentSlot {
                        axis: "Z",
                        present: true,
                        value: 45.0,
                        editable: true,
                        keyed: true, // 実キー持ち(旧 animated)
                        field: Some(TransformField::Rotation),
                    },
                ]),
                decimals: 1,
                key: KeyCellProjection {
                    row: KeyRow::Rotation,
                    state: KeyCellState::Between,
                },
            }],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
                speed_percent: 100.0,
                label_color: None,
                matte: None,
                matte_candidates: vec![],
            },
            masks: vec![],
            effects: vec![],
            text: None,
            audio: None,
            shape: None,
            links: vec![],
        };
        let (start, _) = drag_origin(&selection, TransformField::Rotation)
            .expect("キー持ち field もドラッグを始められるはず(Q0)");
        assert_eq!(start, 45.0, "起点は投影の評価値のはず");
    }

    #[test]
    fn field_decimals_matches_the_projection_rows() {
        assert_eq!(field_decimals(TransformField::PositionX), 3);
        assert_eq!(field_decimals(TransformField::ScaleY), 3);
        assert_eq!(field_decimals(TransformField::AnchorX), 3);
        assert_eq!(field_decimals(TransformField::Rotation), 1);
        assert_eq!(field_decimals(TransformField::Opacity), 0);
    }

    // -----------------------------------------------------------------------
    // 裁定168 施工: 値セル間 gap(裁定167 下段)
    // -----------------------------------------------------------------------

    #[test]
    fn sibling_gap_px_matches_the_ladder_bottom_rung_rounded_to_the_nearest_pixel() {
        // `motolii-timeline-pane::lane_bar::sibling_gap_px` と同じ式・同じ期待値
        // (既定 inspector_row_height=20 でも一致する — 意図的に同じ token 段を
        // 使っているため、値の一致は式が揃っている検算にもなる)。
        assert_eq!(sibling_gap_px(20.0), 2.0);
        assert_eq!(sibling_gap_px(40.0), 3.0);
    }

    #[test]
    fn transform_row_widens_the_value_cell_gap_beyond_the_old_spacing_xs_token_at_a_larger_row_height(
    ) {
        // 既定 dims(row_height=20)では旧 `spacing_xs`(2px)と新式(round(1.5)=2px)
        // が偶然一致してしまい、この2つの違いを既定 dims だけでは検分できない
        // (`sibling_gap_px` 自体は独立式であることを別テストで固定済み)。
        // ここでは inspector_row_height を人為的に変えた `Dimensions` で
        // 「gap は `inspector_row_height` に追従し、`spacing_xs` には追従しない」
        // ことを確かめる — token 借用ではなく専用式になっている証拠。
        let dims = Dimensions {
            inspector_row_height: 40.0,
            ..Dimensions::default()
        };
        assert_eq!(sibling_gap_px(dims.inspector_row_height), 3.0);
        assert_ne!(
            sibling_gap_px(dims.inspector_row_height),
            dims.spacing_xs,
            "gap が旧トークン(spacing_xs)のままでは inspector_row_height の変化に追従しない"
        );
    }
