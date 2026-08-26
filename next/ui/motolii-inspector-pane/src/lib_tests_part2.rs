
    // -----------------------------------------------------------------------
    // 裁定183 taffy 転写(部分適用 — `property_row_css` の doc「FINDING」参照。
    // production 配線は見送ったが、CSS 文字列そのものが `motolii-taffy` の
    // parser で必ず解釈できること・mock の字面と一致することはここで固定する。
    // ±1px oracle 本体は `tests/property_row_taffy_oracle.rs`(モック実測との
    // 突き合わせ)。
    // -----------------------------------------------------------------------

    #[test]
    fn property_row_css_parses_and_declares_the_mock_grid_template() {
        // 裁定183 taffy 転写(部分適用)。production では呼ばない
        // ([`property_row_css`] の doc「FINDING」参照)ので、この crate の
        // 通常ビルドに `motolii_taffy::style_from_css_decl` を引きずらないよう
        // test scope だけで import する(`TaffyBox` はどこからも呼ばれない
        // ため import しない — `motolii-taffy` 自体への Cargo.toml 依存は
        // `tests/property_row_taffy_oracle.rs` が使う)。
        use motolii_taffy::style_from_css_decl;

        let dims = Dimensions::default();
        let css = property_row_css(dims);

        // mock の字面(`inspector-library.html` v3.1 `.columnHeader,.propertyRow`)
        // をそのまま含むこと — CSS 文字列が単一正本という裁定183 の趣旨どおり、
        // 値だけ dims で埋めた形になっているかを直接検算する。
        assert!(
            css.contains("grid-template-columns:minmax(132px,1fr) repeat(3,64px) 26px"),
            "grid-template-columns の字面が mock と食い違う: {css}"
        );

        let style = style_from_css_decl(&css)
            .expect("property_row_css は固定テンプレート+dims の px 値のみを埋める — 解釈は必ず成功する");
        // taffy の `grid_template_columns` は「track 定義の個数」であって展開後の
        // 列数ではない — `repeat(3, 64px)` は1個の `GridTemplateComponent::
        // Repeat` として1トラック扱い(motolii-taffy 側の実測 —
        // `motolii-taffy/tests/css_decl.rs::grid_template_splits_only_outside_parens`
        // が同じ旗艦例文字列で3を固定済み)。よってここは
        // `[minmax(label), repeat(3,64px), 26px]` の3。
        assert_eq!(
            style.grid_template_columns.len(),
            3,
            "track 定義数(label + repeat(3,X/Y/Z) + Key)が3でない"
        );
    }

    // -----------------------------------------------------------------------
    // 裁定168 EXACT TARGET 3: 文字寸検査(柵として固定・現値の乖離は FINDING)
    // -----------------------------------------------------------------------

    /// 裁定168 は「文字寸 = 0.42 × 行高」を単行の余白計算の前提に置く。
    /// **I-tokens(2026-08-22)で根治**: `inspector_row_height` を
    /// `next/reference/mocks/inspector-library.html` v3.1 実測値(25)へ
    /// 束で再転写した結果、`body_text`(11)/`inspector_row_height`(25)= **0.44**
    /// となり、裁定168 の帯(0.42±0.05 = 0.37〜0.47)の**内**に入った
    /// (旧値は 11/20=0.55 で帯の外 — `docs/reviews/
    /// 2026-08-22-inspector-ratio-ledger.md` の FINDING そのもの)。
    ///
    /// このテストは旧 `..._is_locked_at_its_current_out_of_band_value`
    /// (0.55 を固定していた pin)を置き換える —**0.55 の lock は撤去**し、
    /// 「帯の内に入っている」ことを固定する regression lock へ更新した
    /// (どちらかの値が黙って動いて帯の外へ出たら red になる)。両側チェックの
    /// 詳細(モック実測 vs 実装値)は `tests/inspector_ratio_ledger.rs` 側。
    // -----------------------------------------------------------------------
    // K1: Key 列 — 3状態 oracle と click→SetTrack 内容の純関数(落ちるテスト先行)
    // -----------------------------------------------------------------------

    use motolii_core::Fps;

    fn fps30() -> Fps {
        Fps::try_new(30, 1).expect("30fps は正値")
    }

    fn key_at(frame: i64, value: f64, interp: Interp) -> Keyframe {
        Keyframe {
            t: RationalTime::try_from_frame(frame, fps30()).expect("frame→時刻"),
            value: Value::F64(value),
            interp,
            spatial: None,
        }
    }

    fn track_of(keys: Vec<Keyframe>) -> KeyframeTrack {
        let mut track = KeyframeTrack::new();
        for key in keys {
            track.insert(key);
        }
        track
    }

    /// **状態1 oracle**: track 無し=静的。`single_hold_track`(1キー Hold @ZERO、
    /// この crate の静的値の正準表現)も同じ「静的」— Inspector の静的値編集が
    /// 書いた track を「キーが打たれている」と誤読しない。
    #[test]
    fn key_cell_state_is_static_without_a_track_and_for_the_canonical_static_track() {
        assert_eq!(key_cell_state(None, 0, fps30()), KeyCellState::Static);
        let static_track = single_hold_track(Value::F64(2.5));
        assert_eq!(
            key_cell_state(Some(&static_track), 0, fps30()),
            KeyCellState::Static,
            "正準静的表現(1キー Hold @ZERO)は playhead=0 でも静的のはず"
        );
        assert_eq!(key_cell_state(Some(&static_track), 10, fps30()), KeyCellState::Static);
        // 空 track(SetTrack で空を書いた場合の防御)も静的。
        assert_eq!(
            key_cell_state(Some(&KeyframeTrack::new()), 0, fps30()),
            KeyCellState::Static
        );
    }

    /// **状態2/3 oracle**: playhead のフレームにキーが有れば AtKey、track は有るが
    /// そのフレームにキーが無ければ Between。照合は timeline と同じ
    /// `try_to_frame_round`(frame 粒度)。
    #[test]
    fn key_cell_state_distinguishes_at_key_and_between() {
        let track = track_of(vec![
            key_at(10, 0.0, Interp::Linear),
            key_at(20, 5.0, Interp::Linear),
        ]);
        assert_eq!(key_cell_state(Some(&track), 10, fps30()), KeyCellState::AtKey);
        assert_eq!(key_cell_state(Some(&track), 20, fps30()), KeyCellState::AtKey);
        assert_eq!(key_cell_state(Some(&track), 15, fps30()), KeyCellState::Between);
        assert_eq!(
            key_cell_state(Some(&track), 0, fps30()),
            KeyCellState::Between,
            "track の範囲外でも track が有る限り Between(半表示)のはず"
        );
        // 1キーでも正準静的形(Hold @ZERO)でなければ本物のキー。
        let single_linear = track_of(vec![key_at(10, 1.0, Interp::Linear)]);
        assert_eq!(key_cell_state(Some(&single_linear), 10, fps30()), KeyCellState::AtKey);
        assert_eq!(key_cell_state(Some(&single_linear), 11, fps30()), KeyCellState::Between);
    }

    /// **状態1 click**: 現在の静的値で playhead 時刻にキー1個(track 先頭 insert は
    /// Linear)。静的 hold track が既に有ればその値、無ければ呼び手の現在値。
    #[test]
    fn toggling_from_static_creates_one_linear_key_at_the_playhead() {
        // track 無し → 呼び手が渡す現在値(既定値)で作る。
        let new = toggled_key_track(None, 12, fps30(), Value::Vec2([1.0, 1.0]))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 1);
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(12, fps30()).unwrap());
        assert_eq!(new.keys()[0].value, Value::Vec2([1.0, 1.0]));
        assert!(matches!(new.keys()[0].interp, Interp::Linear), "track 先頭 insert は Linear のはず");

        // 正準静的 track 有り → その track の値(呼び手の現在値ではなく)。
        let static_track = single_hold_track(Value::F64(2.5));
        let new = toggled_key_track(Some(&static_track), 12, fps30(), Value::F64(999.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 1);
        assert_eq!(new.keys()[0].value, Value::F64(2.5), "静的 hold track の値が正のはず");
        assert!(matches!(new.keys()[0].interp, Interp::Linear));
    }

    /// **状態2 click(キー2個以上)**: playhead 上のキーだけを除去し、他は保つ。
    #[test]
    fn toggling_on_a_key_removes_only_that_key() {
        let track = track_of(vec![
            key_at(10, 0.0, Interp::Linear),
            key_at(20, 5.0, Interp::Hold),
        ]);
        let new = toggled_key_track(Some(&track), 10, fps30(), Value::F64(0.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 1);
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(20, fps30()).unwrap());
        assert_eq!(new.keys()[0].value, Value::F64(5.0));
        assert!(matches!(new.keys()[0].interp, Interp::Hold), "残るキーの interp を変えない");
    }

    /// **状態2 click(最後の1個)**: track ごと静的化 — 消したキーの値を保った
    /// 正準静的表現(1キー Hold @ZERO)へ(AE のストップウォッチ解除と等価、
    /// 値は失わない)。
    #[test]
    fn removing_the_last_key_returns_a_static_hold_track_keeping_the_value() {
        let track = track_of(vec![key_at(10, 7.5, Interp::Linear)]);
        let new = toggled_key_track(Some(&track), 10, fps30(), Value::F64(0.0))
            .expect("toggle は成功するはず");
        assert_eq!(new, single_hold_track(Value::F64(7.5)), "値を保った静的化のはず");
        assert_eq!(
            key_cell_state(Some(&new), 10, fps30()),
            KeyCellState::Static,
            "静的化後の状態は Static へ戻るはず"
        );
    }

    /// **状態3 click**: playhead 時刻の**評価値**でキー追加。Interp は直前の
    /// キーの流儀に従い、track 先頭(最初のキーより前)への insert は Linear。
    #[test]
    fn toggling_between_keys_inserts_the_evaluated_value_with_the_neighbor_interp() {
        // Linear 区間の中点 → 評価値は補間の中点、interp は前のキーと同じ Linear。
        let track = track_of(vec![
            key_at(0, 0.0, Interp::Linear),
            key_at(20, 10.0, Interp::Linear),
        ]);
        let new = toggled_key_track(Some(&track), 10, fps30(), Value::F64(999.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 3);
        assert_eq!(new.keys()[1].t, RationalTime::try_from_frame(10, fps30()).unwrap());
        assert_eq!(new.keys()[1].value, Value::F64(5.0), "その時刻の eval 値のはず");
        assert!(matches!(new.keys()[1].interp, Interp::Linear));

        // Hold 区間 → 前のキーの値を保持したまま、interp も Hold を継ぐ。
        let track = track_of(vec![
            key_at(0, 3.0, Interp::Hold),
            key_at(20, 10.0, Interp::Linear),
        ]);
        let new = toggled_key_track(Some(&track), 10, fps30(), Value::F64(999.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys()[1].value, Value::F64(3.0), "Hold 区間の eval は前の値のはず");
        assert!(matches!(new.keys()[1].interp, Interp::Hold), "隣接(前)キーの流儀を継ぐはず");

        // 最初のキーより前への insert → Linear(track 先頭の既定)。
        let track = track_of(vec![key_at(20, 10.0, Interp::Hold)]);
        let new = toggled_key_track(Some(&track), 5, fps30(), Value::F64(999.0))
            .expect("toggle は成功するはず");
        assert_eq!(new.keys().len(), 2);
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(5, fps30()).unwrap());
        assert_eq!(new.keys()[0].value, Value::F64(10.0), "範囲外 clamp は端の値のはず");
        assert!(matches!(new.keys()[0].interp, Interp::Linear), "先頭 insert は Linear のはず");
    }

    /// undo 可逆の前提(純関数レベル): AtKey→toggle→(即)toggle で元の意味へ
    /// 戻る(2キー以上)。Document レベルの undo 可逆は shell の drive
    /// (`inspector_key_drive.rs`)が Intent 経由で確かめる。
    #[test]
    fn toggling_twice_on_a_key_round_trips_for_multi_key_tracks() {
        let track = track_of(vec![
            key_at(10, 0.0, Interp::Linear),
            key_at(20, 5.0, Interp::Linear),
        ]);
        let removed = toggled_key_track(Some(&track), 10, fps30(), Value::F64(0.0)).unwrap();
        let restored = toggled_key_track(Some(&removed), 10, fps30(), Value::F64(0.0)).unwrap();
        // 値は eval(範囲外 clamp で端の 0.0…ではなく残キーの 5.0)なので、
        // 復元されるのは「その時刻の評価値のキー」— 時刻集合は元どおり。
        assert_eq!(restored.keys().len(), 2);
        assert_eq!(restored.keys()[0].t, track.keys()[0].t);
        assert_eq!(restored.keys()[1].t, track.keys()[1].t);
    }

    // -----------------------------------------------------------------------
    // 値編集の意味(AE 作法): `edited_value_track` — 静的は静的のまま・
    // キー持ちは playhead へ upsert(2026-08-22 発注)
    // -----------------------------------------------------------------------

    /// キー無し(track 無し・正準静的表現)の値編集は従来どおり静的値の
    /// 書き換え — キーは生えない。
    #[test]
    fn edited_value_track_keeps_static_tracks_static() {
        let new = edited_value_track(None, 15, fps30(), Value::F64(4.0)).unwrap();
        assert_eq!(new, single_hold_track(Value::F64(4.0)));

        let static_track = single_hold_track(Value::F64(1.0));
        let new =
            edited_value_track(Some(&static_track), 15, fps30(), Value::F64(4.0)).unwrap();
        assert_eq!(new, single_hold_track(Value::F64(4.0)), "静的編集でキーが生えている");
        assert_eq!(key_cell_state(Some(&new), 15, fps30()), KeyCellState::Static);
    }

    /// キー持ち track の、playhead にキーが**無い**時刻での編集 = 新キー挿入
    /// (既存キーは無傷・interp は Between 挿入と同規則)。
    #[test]
    fn edited_value_track_inserts_a_new_key_at_the_playhead() {
        let track = track_of(vec![key_at(10, 1.0, Interp::Hold)]);
        let new = edited_value_track(Some(&track), 20, fps30(), Value::F64(3.0)).unwrap();
        assert_eq!(new.keys().len(), 2, "値編集でキーが増えるはず(AE 文法)");
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(10, fps30()).unwrap());
        assert_eq!(new.keys()[0].value, Value::F64(1.0), "既存キーは無傷のはず");
        assert_eq!(new.keys()[1].t, RationalTime::try_from_frame(20, fps30()).unwrap());
        assert_eq!(new.keys()[1].value, Value::F64(3.0));
        assert!(
            matches!(new.keys()[1].interp, Interp::Hold),
            "直前キー(Hold)の流儀を継ぐはず"
        );
        assert!(new.keys()[1].spatial.is_none());

        // 最初のキーより前への挿入は Linear(track 先頭の既定)。
        let new = edited_value_track(Some(&track), 5, fps30(), Value::F64(0.5)).unwrap();
        assert_eq!(new.keys().len(), 2);
        assert_eq!(new.keys()[0].t, RationalTime::try_from_frame(5, fps30()).unwrap());
        assert!(matches!(new.keys()[0].interp, Interp::Linear), "先頭 insert は Linear のはず");
    }

    /// playhead にキーが**有る**時刻での編集 = そのキーの値だけ更新(個数
    /// 不変・時刻/interp/spatial は保つ)。
    #[test]
    fn edited_value_track_updates_the_key_under_the_playhead_in_place() {
        let track = track_of(vec![
            key_at(10, 1.0, Interp::Hold),
            key_at(20, 5.0, Interp::Linear),
        ]);
        let new = edited_value_track(Some(&track), 10, fps30(), Value::F64(9.0)).unwrap();
        assert_eq!(new.keys().len(), 2, "playhead 上の編集はキー個数を変えないはず");
        assert_eq!(new.keys()[0].value, Value::F64(9.0), "playhead 上のキーの値が更新されるはず");
        assert!(matches!(new.keys()[0].interp, Interp::Hold), "interp は保つはず");
        assert_eq!(new.keys()[1].value, Value::F64(5.0), "他のキーは無傷のはず");
    }

    /// 1キーでも実キー(正準静的表現でない)なら upsert — track を静的に
    /// 戻さない(利用者実窓指摘「キーが1つしか打てない」の機序そのもの:
    /// 旧実装はここで `single_hold_track` に置き換えてキーを消していた)。
    #[test]
    fn edited_value_track_never_collapses_a_real_single_key_track_to_static() {
        let track = track_of(vec![key_at(10, 1.0, Interp::Linear)]);
        let new = edited_value_track(Some(&track), 20, fps30(), Value::F64(2.0)).unwrap();
        assert_eq!(new.keys().len(), 2, "キーが1個へ潰れている(旧バグの再発)");
        assert_ne!(
            key_cell_state(Some(&new), 20, fps30()),
            KeyCellState::Static,
            "実キー持ち track が静的化されている"
        );
    }

    /// KeyRow → property / 既定値の対応表(Position/Scale/Rotation/Opacity/Anchor
    /// の5行全部)。
    #[test]
    fn key_rows_map_to_their_properties_and_defaults() {
        let name_of = |row: KeyRow| key_row_property_id(row).expect("標準 property は作れる");
        assert_eq!(name_of(KeyRow::Position), PropertyId::new(property::POSITION).unwrap());
        assert_eq!(name_of(KeyRow::Scale), PropertyId::new(property::SCALE).unwrap());
        assert_eq!(name_of(KeyRow::Rotation), PropertyId::new(property::ROTATION).unwrap());
        assert_eq!(name_of(KeyRow::Opacity), PropertyId::new(property::OPACITY).unwrap());
        assert_eq!(name_of(KeyRow::Anchor), PropertyId::new(property::ANCHOR).unwrap());
        // mask 行は id から動的に決まる(`PropertyId::mask_opacity` が正本)。
        assert_eq!(
            name_of(KeyRow::MaskOpacity(MaskId(7))),
            PropertyId::mask_opacity(MaskId(7))
        );
        assert_eq!(
            name_of(KeyRow::MaskExpansion(MaskId(7))),
            PropertyId::mask_expansion(MaskId(7))
        );

        assert_eq!(key_row_default_value(KeyRow::Position), Value::Vec2([0.0, 0.0]));
        assert_eq!(key_row_default_value(KeyRow::Scale), Value::Vec2([1.0, 1.0]));
        assert_eq!(key_row_default_value(KeyRow::Rotation), Value::F64(0.0));
        assert_eq!(key_row_default_value(KeyRow::Opacity), Value::F64(1.0));
        assert_eq!(key_row_default_value(KeyRow::Anchor), Value::Vec2([0.0, 0.0]));
        assert_eq!(
            key_row_default_value(KeyRow::MaskOpacity(MaskId(7))),
            Value::F64(1.0),
            "mask opacity の既定は layer Opacity と同じ比 1.0 のはず"
        );
        assert_eq!(
            key_row_default_value(KeyRow::MaskExpansion(MaskId(7))),
            Value::F64(0.0),
            "mask expansion の既定は無効値 0.0 のはず"
        );
    }

    // -----------------------------------------------------------------------
    // MASK section(B02 第1切片): mode 巡回・inverted トグル・opacity field
    // -----------------------------------------------------------------------

    /// mode は宣言順の6値を一周して戻る(`next_blend_mode` のテストと同型)。
    #[test]
    fn mask_mode_cycles_through_all_six_modes_and_wraps() {
        assert_eq!(next_mask_mode(MaskMode::Add), MaskMode::Subtract);
        assert_eq!(next_mask_mode(MaskMode::Subtract), MaskMode::Intersect);
        assert_eq!(next_mask_mode(MaskMode::Intersect), MaskMode::Lighten);
        assert_eq!(next_mask_mode(MaskMode::Lighten), MaskMode::Darken);
        assert_eq!(next_mask_mode(MaskMode::Darken), MaskMode::Difference);
        assert_eq!(next_mask_mode(MaskMode::Difference), MaskMode::Add);
    }

    fn three_masks() -> Vec<Mask> {
        vec![
            Mask {
                id: MaskId(1),
                mode: MaskMode::Add,
                inverted: false,
            },
            Mask {
                id: MaskId(2),
                mode: MaskMode::Darken,
                inverted: true,
            },
            Mask {
                id: MaskId(3),
                mode: MaskMode::Difference,
                inverted: false,
            },
        ]
    }

    /// mode 巡回は対象だけを動かし、並び・他の mask・inverted を保つ。
    /// 居ない id は `None`(stale click は no-op)。
    #[test]
    fn masks_with_cycled_mode_touches_only_the_target_and_keeps_the_order() {
        let masks = three_masks();
        let new = masks_with_cycled_mode(&masks, MaskId(2)).expect("対象は居るはず");
        assert_eq!(new.len(), 3);
        assert_eq!(new[0], masks[0], "対象外(前)の mask が動いている");
        assert_eq!(new[1].mode, MaskMode::Difference, "宣言順の次 mode のはず");
        assert_eq!(new[1].id, MaskId(2));
        assert!(new[1].inverted, "mode 巡回が inverted を巻き込んでいる");
        assert_eq!(new[2], masks[2], "対象外(後)の mask が動いている");

        assert_eq!(masks_with_cycled_mode(&masks, MaskId(99)), None);
        assert_eq!(masks_with_cycled_mode(&[], MaskId(1)), None);
    }

    /// inverted トグルも同型(対象だけ・mode は保つ・stale は `None`)。
    #[test]
    fn masks_with_toggled_inverted_flips_only_the_target() {
        let masks = three_masks();
        let new = masks_with_toggled_inverted(&masks, MaskId(2)).expect("対象は居るはず");
        assert!(!new[1].inverted, "true → false へ裏返るはず");
        assert_eq!(new[1].mode, MaskMode::Darken, "トグルが mode を巻き込んでいる");
        assert_eq!(new[0], masks[0]);
        assert_eq!(new[2], masks[2]);

        let back = masks_with_toggled_inverted(&new, MaskId(2)).expect("対象は居るはず");
        assert_eq!(back, masks, "2回のトグルで元へ戻るはず");

        assert_eq!(masks_with_toggled_inverted(&masks, MaskId(99)), None);
    }

    /// mask opacity field は既存の値セル文法の対応表(property/単位/精度/感度)へ
    /// layer Opacity と同格で乗る。
    #[test]
    fn the_mask_opacity_field_joins_the_existing_value_cell_grammar() {
        let field = TransformField::MaskOpacity(MaskId(4));
        assert_eq!(
            property_id(field).expect("mask opacity の property は作れる"),
            PropertyId::mask_opacity(MaskId(4))
        );
        // 表示 % → store 比(clamp 込み — layer Opacity と同じ写像)。
        assert_eq!(next_value(field, 50.0, [0.0, 0.0]), Value::F64(0.5));
        assert_eq!(next_value(field, 150.0, [0.0, 0.0]), Value::F64(1.0));
        assert_eq!(next_value(field, -10.0, [0.0, 0.0]), Value::F64(0.0));
        assert_eq!(field_decimals(field), 0, "% 表示は整数(layer Opacity と同じ)");
        assert_eq!(
            dragged_value(field, 50.0, 20.0, false),
            70.0,
            "drag 感度は 1px = 1%(layer Opacity と同じ)のはず"
        );
    }

    /// drag の起点は MASK section の opacity 行からも読める(drag-to-scrub が
    /// mask opacity セルでも同じに効くための投影側の口)。
    #[test]
    fn drag_origin_finds_the_mask_opacity_slot() {
        let field = TransformField::MaskOpacity(MaskId(1));
        let selection = SelectionProjection {
            layer: LayerId(1),
            selection_count: 1,
            text_layer_count: 0,
            kind: "solid",
            transform: vec![],
            attrs: AttrsProjection {
                name: String::new(),
                hidden: false,
                blend_mode: "Normal".to_owned(),
                speed_percent: 100.0,
                label_color: None,
                matte: None,
                matte_candidates: vec![],
            },
            masks: vec![MaskRowProjection {
                id: MaskId(1),
                mode: MaskMode::Add,
                inverted: false,
                opacity: TransformRowProjection {
                    label: "Opacity",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Opacity",
                        present: true,
                        value: 80.0,
                        editable: true,
                        keyed: false,
                        field: Some(field),
                    }),
                    decimals: 0,
                    key: KeyCellProjection {
                        row: KeyRow::MaskOpacity(MaskId(1)),
                        state: KeyCellState::Static,
                    },
                },
                expansion: TransformRowProjection {
                    label: "Expansion",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Expansion",
                        present: true,
                        value: 0.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::MaskExpansion(MaskId(1))),
                    }),
                    decimals: 2,
                    key: KeyCellProjection {
                        row: KeyRow::MaskExpansion(MaskId(1)),
                        state: KeyCellState::Static,
                    },
                },
            }],
            effects: vec![],
            text: None,
            audio: None,
            shape: None,
            shape_fill: None,
            shape_stroke: None,
            links: vec![],
        };
        let (start, _) = drag_origin(&selection, field).expect("mask opacity は editable のはず");
        assert_eq!(start, 80.0, "起点は投影の表示値(%)のはず");
        assert!(
            drag_origin(&selection, TransformField::MaskOpacity(MaskId(9))).is_none(),
            "別の mask id の field では drag を始めないはず"
        );
    }

    /// drag の起点は AUDIO section の4行からも読める(B42、裁定184 型別
    /// section 第4号 — mask opacity と同じ「専用 section の投影も
    /// `drag_origin` が舐める」拡張、`lib.rs::drag_origin` の AUDIO ループ参照)。
    #[test]
    fn drag_origin_finds_the_audio_section_slots() {
        let selection = SelectionProjection {
            layer: LayerId(1),
            selection_count: 1,
            text_layer_count: 0,
            kind: "media",
            transform: vec![],
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
            audio: Some(AudioSectionProjection {
                level: TransformRowProjection {
                    label: "Level",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Level",
                        present: true,
                        value: 100.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::Level),
                    }),
                    decimals: 1,
                    key: KeyCellProjection {
                        row: KeyRow::Level,
                        state: KeyCellState::Static,
                    },
                },
                pan: TransformRowProjection {
                    label: "Pan",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Pan",
                        present: true,
                        value: -0.3,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::Pan),
                    }),
                    decimals: 2,
                    key: KeyCellProjection {
                        row: KeyRow::Pan,
                        state: KeyCellState::Static,
                    },
                },
                fade_in: TransformRowProjection {
                    label: "Fade In",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Fade In",
                        present: true,
                        value: 0.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::FadeIn),
                    }),
                    decimals: 2,
                    key: KeyCellProjection {
                        row: KeyRow::FadeIn,
                        state: KeyCellState::Static,
                    },
                },
                fade_out: TransformRowProjection {
                    label: "Fade Out",
                    value: RowValue::Scalar(ComponentSlot {
                        axis: "Fade Out",
                        present: true,
                        value: 0.0,
                        editable: true,
                        keyed: false,
                        field: Some(TransformField::FadeOut),
                    }),
                    decimals: 2,
                    key: KeyCellProjection {
                        row: KeyRow::FadeOut,
                        state: KeyCellState::Static,
                    },
                },
            }),
            shape: None,
            shape_fill: None,
            shape_stroke: None,
            links: vec![],
        };
        let (level_start, _) =
            drag_origin(&selection, TransformField::Level).expect("Level は editable のはず");
        assert_eq!(level_start, 100.0);
        let (pan_start, _) =
            drag_origin(&selection, TransformField::Pan).expect("Pan は editable のはず");
        assert_eq!(pan_start, -0.3);
        assert!(
            drag_origin(&selection, TransformField::Rotation).is_none(),
            "AUDIO 投影に無い field では drag を始めないはず"
        );
    }

    #[test]
    fn inspector_character_size_ratio_is_locked_within_the_charter_168_band() {
        let dims = Dimensions::default();
        let ratio = dims.theme().text.body / dims.inspector_row_height;

        const TARGET: f32 = 0.42;
        const TOLERANCE: f32 = 0.05;
        let in_band = (ratio - TARGET).abs() <= TOLERANCE;

        assert_eq!(
            ratio, 0.44,
            "body_text/inspector_row_height の実測比が動いた(I-tokens の再転写値 \
             0.44 から動いたなら、このテストと台帳・FINDING の記載を三箇所とも \
             更新すること)"
        );
        assert!(
            in_band,
            "比 {ratio} が裁定168 の帯(0.42±0.05)から外れた — I-tokens の \
             再転写(inspector_row_height=25)がこの根治の前提なので、\
             どちらかの値が意図せず動いた疑いがある"
        );
    }

    // -----------------------------------------------------------------------
    // 線化 D2(裁定179「箱は状態の器」): style 関数レベルの柵。
    // widget tree で hover の Status を作れない(iced_test の Simulator は
    // cursor を置けるが container の style closure は status を受けない)ので、
    // style fn を直接呼んで固定する(発注書の指定どおり)。
    // -----------------------------------------------------------------------

    /// 「輪郭が消えている」の判定: width 0 か色が完全透明のどちらかなら
    /// 輪郭は描かれない(`container::draw_background` は `border.width > 0.0`
    /// でだけ quad を出す、fork 実測)。
    fn border_is_invisible(border: iced::Border) -> bool {
        border.width == 0.0 || border.color.a == 0.0
    }

    /// 値セル(表示状態): 平常は素の数字(面なし・輪郭透明)、hover でだけ
    /// 箱が現れる(既存 surface_hover 文法 — name 欄 hover と同じ)。
    #[test]
    fn the_value_box_is_bare_at_rest_and_boxed_on_hover() {
        let dims = Dimensions::default();
        let colors = Colors::default();

        let idle = value_box_style(dims, colors, ValueBoxStatus::Idle);
        assert!(
            idle.background.is_none(),
            "平常の値セルに面が残っている: {:?}",
            idle.background
        );
        assert!(
            border_is_invisible(idle.border),
            "平常の値セルに不透明な輪郭が残っている: {:?}",
            idle.border
        );

        let hovered = value_box_style(dims, colors, ValueBoxStatus::Hovered);
        assert_eq!(
            hovered.background,
            Some(iced::Background::Color(colors.surface_hover)),
            "hover の値セルに面(surface_hover)が現れない"
        );
        assert_eq!(hovered.border.color, colors.border_default);
        assert!(
            hovered.border.width > 0.0 && hovered.border.color.a > 0.0,
            "hover の値セルに不透明な輪郭が現れない: {:?}",
            hovered.border
        );
    }

    /// Blend/Reset ボタン: 平常は素の文字(面なし・輪郭なし)、hover で面、
    /// press で選択面(menubar `leaf_style` と同じ裁定179 文法)。
    #[test]
    fn the_inspector_buttons_are_bare_at_rest_and_faced_on_hover() {
        let colors = Colors::default();

        let rest = flat_button_style(colors, button::Status::Active);
        assert!(
            rest.background.is_none(),
            "平常のボタンに面が残っている: {:?}",
            rest.background
        );
        assert!(
            border_is_invisible(rest.border),
            "平常のボタンに輪郭が残っている: {:?}",
            rest.border
        );
        assert_eq!(rest.text_color, colors.text_primary);

        let hovered = flat_button_style(colors, button::Status::Hovered);
        assert_eq!(
            hovered.background,
            Some(iced::Background::Color(colors.surface_hover))
        );
        assert!(border_is_invisible(hovered.border), "hover のボタンは面のみ(輪郭は出さない)");

        let pressed = flat_button_style(colors, button::Status::Pressed);
        assert_eq!(
            pressed.background,
            Some(iced::Background::Color(colors.state_selected))
        );
    }

    /// M glyph: 輪郭は active(hidden=on)の時だけ(裁定179「チップ輪郭=
    /// 選択時のみ」)。非 active の平常は素の文字、hover は面。
    #[test]
    fn the_mute_glyph_wears_its_outline_only_while_active() {
        let dims = Dimensions::default();
        let colors = Colors::default();

        let off = glyph_button_style(dims, colors, button::Status::Active, false);
        assert!(
            border_is_invisible(off.border),
            "非 active の M glyph に常時輪郭が残っている: {:?}",
            off.border
        );

        let off_hover = glyph_button_style(dims, colors, button::Status::Hovered, false);
        assert_eq!(
            off_hover.background,
            Some(iced::Background::Color(colors.surface_hover))
        );

        let on = glyph_button_style(dims, colors, button::Status::Active, true);
        assert_eq!(on.border.color, colors.action_active, "active の M glyph は accent 縁(状態の器)");
        assert!(on.border.width > 0.0);
        assert_eq!(on.text_color, colors.action_active);
    }

    /// Speed 欄が採る text_input 文法(name 欄と同一の `name_input_style`):
    /// 平常=素・hover=面+枠・focus=箱+focus 縁。既存文法の pin(この文法へ
    /// Speed 欄を合流させるのが D2 の変更 — 文法そのものは name 欄で施工済み)。
    #[test]
    fn the_bare_input_grammar_shows_its_box_only_on_hover_or_focus() {
        let dims = Dimensions::default();
        let colors = Colors::default();

        let rest = name_input_style(dims, colors, text_input::Status::Active);
        assert_eq!(
            rest.background,
            iced::Background::Color(iced::Color::TRANSPARENT)
        );
        assert!(border_is_invisible(rest.border));

        let hovered = name_input_style(dims, colors, text_input::Status::Hovered);
        assert_eq!(hovered.background, iced::Background::Color(colors.surface_hover));
        assert_eq!(hovered.border.color, colors.border_default);

        let focused = name_input_style(
            dims,
            colors,
            text_input::Status::Focused { is_hovered: false },
        );
        assert_eq!(focused.background, iced::Background::Color(colors.surface_app));
        assert_eq!(focused.border.color, colors.action_active);
    }
