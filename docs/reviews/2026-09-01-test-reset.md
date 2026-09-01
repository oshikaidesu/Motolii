# 試験の総入替 — 448本を一度に消す (2026-09-01)

利用者裁定。**多すぎるので一度全部消し、道ごとに書き直す。**

## なぜ

同じ日に、**333本が全部通っている間に**次が窓の中で起きていた。

- 静止画が一度も映っていない(層は建ち、枠も動き、寸法も出るのに絵だけ出ない)
- 窓のどの入力欄にも文字が1字も入らない(「double-click to type」と窓が嘘をついていた)
- 素材を入れる口が無い / 出す口が無い
- 窓と書き出しで違う絵が出る(間に合わない時に古いコマを今のコマの顔で返す)
- 名前を変えても行の字が変わらない

見つけたのは**説明書だけ読んだ試し手が窓を触った時**で、試験ではない。

原因は「試験が弱い」ではなく **その道の試験が1本も無い**。`a_keystroke_reaches_the_app`
に至っては `#app` に焦点を当ててから撃っていて、**バグの方を正解として書いてあった**。

規模: src 33,607行に対し試験 13,402行(src内 7,561 + tests/ 5,841)、448本。**4割**。

## 残す物 — 器具は主張ではない

- `src/ui/gui.rs` の `Gui`(窓を開けずに座標で叩くハーネス)
- `tests/testkit/`、`tests/fixtures/`、`tests/golden/`

器具は何も主張しないので、間違った安心を作らない。消すと書き直しの一手目が高くつく。

## 書き直しの規則

1. **利用者が歩く道を1本ずつ**通す。押す・打つ・出す、が本当に通るか
2. **例で不変条件を何度も書かない**。振れる物は生成させる(proptest)
3. **時計で判ずるなら一番速い回で**。1回の実測は機械の混み具合で倍振れる
4. 書いた試験が**直す前に落ちる**ことを必ず確かめる。落ちない試験は試験ではない

## 消した448本(道を書き直す時の引き当て表)

### tests(統合) (115)
- `a_layer_only_exists_inside_its_placement`
- `a_layer_with_depth_survives_the_accumulator_across_runs`
- `a_missing_obj_isolates_to_this_layer`
- `a_modulator_sum_beyond_the_lottie_range_is_reported_not_silently_clamped`
- `a_normal_stack_costs_the_same_as_a_blended_stack`
- `a_param_with_no_track_is_simply_absent`
- `adopted_rows_point_at_real_code`
- `adopted_rows_point_at_real_code`
- `all_four_matte_modes_render_through_zero_copy_path`
- `an_effect_with_no_enabled_track_defaults_to_enabled`
- `an_obj_layer_puts_pixels_on_the_frame`
- `animated_opacity_track_becomes_a_keyframed_lottie_property`
- `apply_all_is_atomic_the_valid_intent_does_not_stick_when_a_later_one_fails`
- `apply_all_rolls_back_even_when_the_batch_writes_multiple_components`
- `attrs_without_a_label_color_field_defaults_to_unassigned`
- `audio_only_file_has_no_frame_count_but_has_a_real_duration`
- `auto_save_rotates_and_caps_at_generations`
- `auto_save_skips_when_not_dirty`
- `auto_save_skips_when_there_is_no_project_path`
- `auto_save_survives_stray_tmp_debris_without_corrupting_the_prior_generation`
- `backward_compatible_wrapper_renders_nothing_for_shapes_but_does_not_error`
- `blend_layers_do_not_cost_a_submit_each`
- `camera_usage_is_reported_as_unsupported`
- `checkbox_param_is_bool_not_int_boolean`
- `color_param_round_trips`
- `composition_and_shape_layer_export_with_no_unsupported_items`
- `composition_background_round_trips_through_save_and_load`
- `composition_without_a_background_field_defaults_to_opaque_black`
- `darken_takes_the_darker_and_lighten_the_lighter`
- `depth_does_not_erase_the_picture`
- `difference_of_black_and_white_is_white`
- `disabling_an_effect_keeps_it_in_the_list`
- `drop_down_param_is_an_enum_index_and_holds_between_keys`
- `duplicate_effect_ids_are_still_rejected`
- `edit_storm_with_the_real_track_type`
- `effect_instance_is_reported_as_unsupported_not_silently_dropped`
- `effect_param_tracks_appear_in_the_property_list`
- `effect_type_is_a_plugin_id_string_shared_by_first_and_third_party`
- `effects_and_their_tracks_survive_a_save_and_load_round_trip`
- `enabling_and_disabling_can_be_keyframed_across_time`
- `every_planned_row_has_a_work_package`
- `export_still_returns_within_timeout_and_writes_a_real_png`
- `file_round_trips`
- `flattening_keeps_the_picture`
- `glow_spreads_in_both_directions`
- `glow_spreads_light_outside_the_shape`
- `hundreds_of_frames_do_not_grow_the_gpu_state`
- `hundreds_of_frames_through_the_window_path_do_not_grow`
- `japanese_text_zero_copy_matches_cpu_export_and_renders_visible_pixels`
- `label_color_round_trips_through_save_and_load`
- `layer_param_references_a_stable_layer_id_not_an_index`
- `luminosity_takes_the_source_brightness`
- `mask_shape_and_opacity_export_as_bezier_and_scalar_properties`
- `mask_without_a_mode_field_defaults_to_add`
- `matte_expands_into_explicit_tt_tp_td_fields`
- `matte_layers_do_not_cost_a_submit_each`
- `matte_zero_copy_matches_cpu_export_within_tolerance`
- `missing_file_yields_none_for_both`
- `missing_ply_isolates_to_this_layer_without_erroring_the_whole_frame`
- `move_trim_and_split_are_all_one_intent`
- `multiply_with_white_keeps_the_backdrop_and_with_black_goes_black`
- `new_edit_after_undo_drops_the_redo_space`
- `no_float_math_between_time_and_frames_outside_core`
- `old_media_and_point_cloud_json_deserialize_to_file`
- `owned_mechanisms_stay_within_the_budget`
- `placement_clamps_to_the_shorter_of_source_and_comp`
- `ply_point_cloud_renders_visible_pixels_through_render_frame`
- `point_param_round_trips`
- `presentable_target_accepts_host_spec`
- `presentable_target_rejects_missing_render_attachment`
- `presentable_target_rejects_wrong_format`
- `presentable_target_rejects_wrong_size`
- `property_link_bakes_into_a_normal_keyframed_property`
- `rectangle_shape_zero_copy_matches_cpu_export_and_renders_visible_pixels`
- `remove_layer_is_a_tombstone_not_a_delete`
- `render_into_applies_effect_passes`
- `render_into_applies_isf_effect_passes`
- `render_into_draws_tilted_plates`
- `render_into_matches_render_with_effects_for_separable_blends`
- `render_into_writes_the_external_target`
- `render_to_texture_is_deterministic_across_repeated_calls`
- `render_to_texture_matches_render_with_timing_for_an_opaque_solid_layer`
- `render_to_texture_reused_scratch_shows_the_new_frames_content_not_the_old_one`
- `render_to_texture_reuses_scratch_texture_across_frames`
- `report_coverage`
- `report_coverage`
- `report_inventory`
- `report_work_packages`
- `resolve_carries_the_enabled_effect_stack_with_evaluated_params`
- `resolve_does_not_carry_disabled_effects`
- `resolve_yields_an_empty_effect_stack_when_none_are_set`
- `resolved_effect_omits_params_with_no_track`
- `resolved_effect_params_interpolate_with_time`
- `save_and_load_round_trips_the_whole_document`
- `saving_folds_the_edit_history`
- `screen_with_white_goes_white`
- `shape_layer_exports_fill_and_geometry`
- `slider_and_angle_params_are_plain_scalar_tracks`
- `slot_referenced_property_exports_as_sid_reference`
- `source_in_shifts_which_frame_is_used`
- `speed_track_accumulates_variable_speed_over_time`
- `text_style_size_track_is_baked_the_same_as_resolved_text_document`
- `the_budget_is_not_stale`
- `the_counter_actually_counts`
- `the_map_covers_the_rive_defs`
- `the_map_covers_the_whole_schema`
- `the_map_covers_the_whole_vocabulary`
- `the_same_time_gives_the_same_picture_however_you_got_there`
- `tilt_changes_the_picture_without_erasing_it`
- `tilt_survives_a_pinned_background`
- `undo_and_redo_are_only_time_movement`
- `undo_to_empty_document`
- `value_at_goes_through_the_ported_evaluator`
- `video_file_reports_its_native_frame_count`
- `workspace_has_no_scattered_time_to_frame_f64_paths`

### doc::core (46)
- `doc::core::camera::tests::default_camera_matches_identity_pixel_mapping_at_z0`
- `doc::core::camera::tests::distance_from_camera_is_stable_across_zoom`
- `doc::core::camera::tests::nearer_planes_move_more_than_farther_planes_under_the_same_pan`
- `doc::core::camera::tests::non_finite_roll_degrees_does_not_panic_and_falls_back_to_no_roll`
- `doc::core::camera::tests::panning_the_camera_shifts_the_z0_plane_the_opposite_way`
- `doc::core::camera::tests::roll_rotates_the_z0_plane_clockwise_around_comp_center`
- `doc::core::camera::tests::zoom_scales_the_z0_plane_isotropically_around_comp_center`
- `doc::core::frame::layer_placement_tests::skew_along_x_axis_leaves_the_x_axis_fixed`
- `doc::core::frame::layer_placement_tests::skew_along_y_axis_leaves_the_y_axis_fixed`
- `doc::core::frame::layer_placement_tests::zero_skew_does_not_move_points`
- `doc::core::frame::tests::packed_desc_and_size`
- `doc::core::frame::tests::premultiplies_straight_color`
- `doc::core::frame::tests::same_aspect_integer_scale_accepts_draft_halving`
- `doc::core::frame::tests::same_aspect_integer_scale_handles_large_dimensions_without_overflow`
- `doc::core::frame::tests::same_aspect_integer_scale_rejects_mismatched_aspect`
- `doc::core::frame::tests::same_aspect_integer_scale_rejects_non_integer_scale`
- `doc::core::frame::tests::same_aspect_integer_scale_rejects_zero_dimension`
- `doc::core::frame::tests::try_yuv_rejects_odd_dimensions`
- `doc::core::frame::tests::validate_rejects_bad_stride`
- `doc::core::frame::tests::yuv420_size`
- `doc::core::frame::tests::yuv_rejects_odd_dimensions`
- `doc::core::time::tests::decimal_str_parses_ffprobe_style`
- `doc::core::time::tests::ffmpeg_seek_before_frame_matches_half_frame_offset`
- `doc::core::time::tests::fps_serde_rejects_non_positive`
- `doc::core::time::tests::fps_try_new_rejects_non_positive`
- `doc::core::time::tests::frame_conversion_exact_ntsc`
- `doc::core::time::tests::frame_floor_boundaries`
- `doc::core::time::tests::frame_round_ntsc_lattice`
- `doc::core::time::tests::mixed_fps_clips_align`
- `doc::core::time::tests::no_float_drift_accumulation`
- `doc::core::time::tests::normalizes_sign_and_reduces`
- `doc::core::time::tests::ordering_across_denominators`
- `doc::core::time::tests::sample_index_avoids_f64_underflow_at_ntsc_frame`
- `doc::core::time::tests::sample_index_fraction_is_rational_remainder`
- `doc::core::time::tests::sample_index_negative_fraction_stays_in_unit_interval`
- `doc::core::time::tests::sample_index_since_near_zero_despite_i128_overflow`
- `doc::core::time::tests::sample_index_since_survives_i64_span`
- `doc::core::time::tests::serde_rejects_zero_denominator`
- `doc::core::time::tests::try_neg_i64_min_overflows`
- `doc::core::time::tests::try_new_i64_min_with_negative_den_overflows`
- `doc::core::time::tests::try_new_rejects_zero_denominator`
- `doc::core::wide_div::tests::complement_unit_interval_stays_below_one`
- `doc::core::wide_div::tests::mul_div_review_counterexample_near_zero`
- `doc::core::wide_div::tests::mul_div_small`
- `doc::core::wide_div::tests::rem_over_den_never_reaches_one_for_den_minus_one`
- `doc::core::wide_div::tests::widening_mul_basic`

### doc::eval (40)
- `doc::eval::bezier::tests::ease_in_out_is_symmetric_and_monotone`
- `doc::eval::bezier::tests::endpoints_fixed`
- `doc::eval::bezier::tests::linear_curve_is_identity`
- `doc::eval::bezier::tests::overshoot_allowed_in_y`
- `doc::eval::track::split_tests::split_bezier_preserves_curve_at_multiple_samples`
- `doc::eval::track::split_tests::split_bezier_rejects_infinite_control_value_with_finite_progress`
- `doc::eval::track::split_tests::split_bezier_rejects_invalid_progress`
- `doc::eval::track::split_tests::split_bezier_rejects_unrepresentable`
- `doc::eval::track::split_tests::split_bezier_returns_bezier_pair`
- `doc::eval::track::split_tests::split_hold_and_linear_keeps_variant`
- `doc::eval::track::tests::a_smoothed_step_arrives_at_the_step_time`
- `doc::eval::track::tests::an_elastic_step_starts_at_the_step_time`
- `doc::eval::track::tests::bezier_ease_in_out_midpoint`
- `doc::eval::track::tests::bezier_easing_applies_one_shared_u_to_every_color_channel`
- `doc::eval::track::tests::bounce_is_continuous_across_its_segments`
- `doc::eval::track::tests::clamps_outside_range`
- `doc::eval::track::tests::elastic_overshoots_past_one`
- `doc::eval::track::tests::empty_track_returns_zero`
- `doc::eval::track::tests::every_interp_starts_at_zero_and_ends_at_one`
- `doc::eval::track::tests::hold_ignores_spatial_tangent_even_if_present`
- `doc::eval::track::tests::hold_keeps_value_until_next_key`
- `doc::eval::track::tests::insert_replaces_same_time_key`
- `doc::eval::track::tests::keyframe_linear_across_i64_span_does_not_collapse_to_zero`
- `doc::eval::track::tests::keyframe_without_spatial_field_deserializes_as_none`
- `doc::eval::track::tests::linear_interpolation_at_rational_times`
- `doc::eval::track::tests::rejects_invalid_bezier_on_validate`
- `doc::eval::track::tests::rejects_unsorted_keys_on_validate`
- `doc::eval::track::tests::spatial_tangent_bows_the_position_path_off_the_straight_line`
- `doc::eval::track::tests::step_width_may_be_a_continuous_value`
- `doc::eval::track::tests::steps_holds_inside_each_step`
- `doc::eval::track::tests::temporal_easing_moves_along_the_same_spatial_curve`
- `doc::eval::track::tests::validate_rejects_out_of_domain_parametric_interps`
- `doc::eval::track::tests::vec2_without_spatial_tangents_still_lerps_in_a_straight_line`
- `doc::eval::value::tests::bool_holds_until_the_next_key`
- `doc::eval::value::tests::enum_holds_until_the_next_key`
- `doc::eval::value::tests::layer_id_holds_until_the_next_key`
- `doc::eval::value::tests::lerp_mismatched_variants_returns_first`
- `doc::eval::value::tests::lerp_scalar_and_vector`
- `doc::eval::value::tests::path_interpolates_vertex_by_vertex`
- `doc::eval::value::tests::path_with_different_vertex_counts_does_not_interpolate`

### doc::store (40)
- `doc::store::asset::tests::admit_deduplicates_by_content_hash`
- `doc::store::asset::tests::asset_deserializes_from_pre_status_field_json`
- `doc::store::asset::tests::asset_table_roundtrip_keeps_multi_keys`
- `doc::store::asset::tests::asset_table_roundtrip_never_serializes_status`
- `doc::store::asset::tests::insert_rejects_retired_id_after_remove`
- `doc::store::asset::tests::path_normalization_uses_forward_slash`
- `doc::store::asset::tests::relink_preserves_asset_identity`
- `doc::store::asset::tests::relink_updates_only_the_asset_path`
- `doc::store::asset::tests::relinking_a_missing_asset_makes_it_present`
- `doc::store::asset::tests::resolve_status_falls_back_to_relative_path_when_absolute_is_gone`
- `doc::store::asset::tests::resolve_status_is_unchecked_without_any_path`
- `doc::store::asset::tests::resolve_status_missing_when_neither_path_resolves`
- `doc::store::asset::tests::resolve_status_missing_when_only_relative_given_but_no_project_root`
- `doc::store::asset::tests::resolve_status_present_via_absolute_path`
- `doc::store::asset::tests::restore_reinstates_identity_without_rewinding_next`
- `doc::store::document::tests::a_non_track_json_component_other_than_present_is_reported_not_silently_dropped`
- `doc::store::slot::constant_tests::a_constant_survives_the_round_trip_without_becoming_a_track`
- `doc::store::slot::tests::a_bare_keyframe_track_json_still_deserializes_as_a_track_base`
- `doc::store::slot::tests::a_bare_slot_id_string_still_deserializes_as_a_slot_base`
- `doc::store::slot::tests::a_legacy_bare_link_object_still_deserializes_as_link_only`
- `doc::store::slot::tests::a_reserved_name_in_source_property_fails_to_deserialize`
- `doc::store::slot::tests::distinct_slot_ids_are_accepted`
- `doc::store::slot::tests::duplicate_slot_ids_are_rejected`
- `doc::store::slot::tests::property_link_serializes_source_property_as_a_bare_string`
- `doc::store::slot::tests::property_source_round_trips_for_every_shape`
- `doc::store::slot::tests::property_source_track_round_trips_through_the_explicit_wire_shape`
- `doc::store::slot::tests::slot_id_serializes_identically_to_a_bare_string`
- `doc::store::slot::tests::translate_link_identity_passes_value_through_unchanged`
- `doc::store::slot::tests::translate_link_linear_applies_scale_and_offset`
- `doc::store::slot::tests::translate_link_linear_applies_uniformly_to_vec2_components`
- `doc::store::slot::tests::translate_link_linear_defaults_to_identity_when_params_are_absent`
- `doc::store::slot::tests::translate_link_rejects_type_mismatch_instead_of_approximating`
- `doc::store::slot::tests::translate_link_remap_clamps_only_when_requested`
- `doc::store::slot::tests::translate_link_remap_maps_between_ranges`
- `doc::store::slot::tests::translate_link_returns_none_for_unknown_plugin_id`
- `doc::store::text::tests::content_track_holds_until_the_next_key`
- `doc::store::text::tests::empty_content_track_evaluates_to_empty_string`
- `doc::store::text::tests::inserting_the_same_time_replaces_not_duplicates`
- `doc::store::view::resolve::tests::position_z_track_resolves_into_placement_z`
- `doc::store::view::resolve::tests::shared_ancestor_is_resolved_exactly_once_across_siblings`

### render::audio (81)
- `render::audio::cache::tests::frame_at_arbitrary_positions_matches_direct_index`
- `render::audio::cache::tests::frame_at_end_is_out_of_range`
- `render::audio::cache::tests::read_frames_at_exact_end_is_ok_when_empty`
- `render::audio::cache::tests::read_frames_out_of_range_is_typed_error_not_panic`
- `render::audio::cache::tests::read_frames_returns_contiguous_slice`
- `render::audio::cache::tests::rejects_misaligned_buffer`
- `render::audio::cache::tests::rejects_zero_channels`
- `render::audio::cache::tests::rejects_zero_sample_rate`
- `render::audio::clock::tests::device_wait_subtracts_from_elapsed_device_frames_only`
- `render::audio::clock::tests::drift_within_one_frame_at_same_floor`
- `render::audio::clock::tests::full_block_without_underrun_only_advances_supplied`
- `render::audio::clock::tests::one_second_origin_is_exact_at_device_rates`
- `render::audio::clock::tests::pause_freezes_position_even_if_counters_keep_advancing`
- `render::audio::clock::tests::perceptual_subtracts_device_wait_only`
- `render::audio::clock::tests::position_advances_monotonically_with_supply`
- `render::audio::clock::tests::repeated_underruns_accumulate_independently_of_supplied`
- `render::audio::clock::tests::sample_frames_to_time_matches_rational`
- `render::audio::clock::tests::seek_jumps_position_immediately`
- `render::audio::clock::tests::underrun_fills_silence_and_does_not_advance_logical_position`
- `render::audio::clock::tests::underrun_silence_does_not_advance_position`
- `render::audio::clock::tests::update_from_output_callback_maps_playback_minus_callback_to_frames`
- `render::audio::clock::tests::update_from_output_callback_zero_wait_when_playback_equals_callback`
- `render::audio::clock::tests::zero_sample_rate_is_rejected_at_construction`
- `render::audio::convert::tests::mono_to_stereo_duplicates_channels`
- `render::audio::convert::tests::rejects_more_than_two_channels`
- `render::audio::decode::tests::corrupt_input_is_typed_error_not_panic`
- `render::audio::decode::tests::missing_file_is_typed_io_error`
- `render::audio::device::tests::select_falls_back_when_source_unsupported`
- `render::audio::device::tests::select_prefers_exact_source_rate`
- `render::audio::device::tests::select_returns_none_for_empty_ranges`
- `render::audio::meter::tests::clip_latch_retains_clip_until_reset`
- `render::audio::meter::tests::peak_and_clip_match_known_samples`
- `render::audio::meter::tests::reset_clears_latched_clip`
- `render::audio::meter::tests::silence_does_not_clip`
- `render::audio::mix::tests::chunk_size_rebuild_matches_whole`
- `render::audio::mix::tests::equal_power_fade_differs_from_linear_at_midpoint`
- `render::audio::mix::tests::fade_duration_exceeding_clip_length_is_clamped_not_rejected`
- `render::audio::mix::tests::gain_scales_output_linearly`
- `render::audio::mix::tests::gain_zero_is_silence_without_affecting_other_sources`
- `render::audio::mix::tests::gap_is_silence_not_underflow_counter`
- `render::audio::mix::tests::hold_gain_keyframes_follow_eval`
- `render::audio::mix::tests::linear_fade_in_ramps_from_zero_to_full`
- `render::audio::mix::tests::linear_fade_out_ramps_from_full_to_zero`
- `render::audio::mix::tests::master_gain_applies_last_without_clamp`
- `render::audio::mix::tests::metering_does_not_change_pcm`
- `render::audio::mix::tests::mono_44100_and_stereo_48000_mix`
- `render::audio::mix::tests::normalize_gain_applied_through_existing_gain_path_hits_target_peak`
- `render::audio::mix::tests::normalize_gain_for_peak_computes_linear_scalar`
- `render::audio::mix::tests::normalize_gain_for_peak_rejects_invalid_target`
- `render::audio::mix::tests::normalize_gain_for_peak_silence_is_noop`
- `render::audio::mix::tests::out_of_range_loop_wraps`
- `render::audio::mix::tests::overlapping_fade_in_and_out_multiply_without_leaving_unit_range`
- `render::audio::mix::tests::pan_center_is_identity`
- `render::audio::mix::tests::pan_field_routes_through_mix_audio`
- `render::audio::mix::tests::pan_hard_left_sums_both_channels_into_left`
- `render::audio::mix::tests::pan_hard_right_sums_both_channels_into_right`
- `render::audio::mix::tests::pan_none_track_defaults_to_center`
- `render::audio::mix::tests::pan_out_of_range_values_clamp_instead_of_erroring`
- `render::audio::mix::tests::same_input_mixes_to_byte_identical_output`
- `render::audio::mix::tests::same_input_with_pan_and_fade_mixes_to_byte_identical_output`
- `render::audio::mix::tests::ten_minute_timeline_frame_maps_without_drift`
- `render::audio::mix::tests::two_sources_sum_deterministically`
- `render::audio::mix::tests::varispeed_doubles_source_advance`
- `render::audio::producer::tests::push_frames_ignores_channel_zero`
- `render::audio::producer::tests::push_frames_stops_at_free_slots`
- `render::audio::program::tests::fingerprint_is_the_soundtrack_cache_identity_when_present`
- `render::audio::program::tests::hidden_media_does_not_project_to_soundtrack_input`
- `render::audio::program::tests::non_media_does_not_project_to_soundtrack_input`
- `render::audio::program::tests::visible_media_projects_to_soundtrack_input_with_path_fallback`
- `render::audio::resample::tests::impulse_at_origin_lands_on_expected_device_frame`
- `render::audio::resample::tests::rejects_matching_rates`
- `render::audio::ring::tests::empty_ring_is_pure_silence_and_one_underrun_event`
- `render::audio::ring::tests::full_pop_advances_supplied_only`
- `render::audio::ring::tests::mismatched_frame_alignment_is_ignored_without_panic`
- `render::audio::ring::tests::underrun_fills_silence_and_does_not_advance_supplied_frames`
- `render::audio::time_map::tests::constant_speed_scales_clip_local`
- `render::audio::time_map::tests::identity_maps_same_time`
- `render::audio::time_map::tests::is_identity_is_semantic`
- `render::audio::time_map::tests::offset_maps_local_zero_to_source_start`
- `render::audio::time_map::tests::rejects_non_positive_speed_denominator`
- `render::audio::time_map::tests::rejects_non_positive_speed_num`

### render::compositor (4)
- `render::compositor::effects::isf::tests::compiles_the_host_vertex_source_to_wgsl`
- `render::compositor::effects::isf::tests::compiles_the_wrapped_fragment_source_to_wgsl`
- `render::compositor::effects::isf::tests::parses_real_isf_header_and_body_generically`
- `render::compositor::effects::isf::tests::reads_passes_and_refuses_persistent_buffers`

### render::engine (23)
- `render::engine::shape::tests::ellipse_with_stroke_only_produces_visible_pixels`
- `render::engine::shape::tests::empty_shape_list_yields_no_texture`
- `render::engine::shape::tests::rectangle_with_fill_produces_visible_pixels`
- `render::engine::shape::tests::shape_is_centered_on_the_canvas_not_anchored_to_the_top_left`
- `render::engine::text::tests::empty_content_yields_no_texture`
- `render::engine::text::tests::empty_style_table_yields_no_texture`
- `render::engine::text::tests::line_height_from_style_moves_the_second_line_baseline`
- `render::engine::text::tests::missing_font_is_an_explicit_error`
- `render::engine::text::tests::text_layer_produces_non_empty_pixels_english`
- `render::engine::text::tests::text_layer_produces_non_empty_pixels_japanese`
- `render::engine::text::tests::two_line_content_reaches_further_down_the_canvas_than_one_line`
- `render::engine::texture::cache_hole::leaving_the_frame_does_not_throw_away_the_pixels`
- `render::engine::texture::still_image_tests::a_broken_image_says_so_instead_of_going_silent`
- `render::engine::texture::still_image_tests::a_still_image_becomes_a_texture`
- `render::engine::translate::known_effects_tests::known_effects_are_all_actually_drawable`
- `render::engine::translate::known_effects_tests::known_effects_is_exactly_glow_and_isf_bloom_and_gradient_and_tri_led_today`
- `render::engine::translate::known_effects_tests::known_effects_isf_bloom_catalog_matches_the_generic_manifest`
- `render::engine::translate::translate_blend_mode_tests::add_is_accepted`
- `render::engine::translate::translate_blend_mode_tests::nonseparable_modes_are_accepted`
- `render::engine::translate::translate_blend_mode_tests::separable_modes_are_accepted`
- `render::engine::translate::translate_effect_passes_tests::no_effects_yields_no_passes`
- `render::engine::translate::translate_effect_passes_tests::unknown_plugin_id_is_skipped_silently`
- `render::engine::translate::translate_matte_mode_tests::all_four_matte_modes_translate_one_to_one`

### render::media (22)
- `render::media::encode::tests::encoder_rejects_non_rgba_format`
- `render::media::encode::tests::encoder_rejects_wrong_frame_size`
- `render::media::encode::tests::finish_drains_stderr_before_wait_without_deadlock`
- `render::media::mesh::tests::a_missing_file_is_an_error_not_a_panic`
- `render::media::mesh::tests::a_quad_loads_as_two_triangles`
- `render::media::mesh::tests::only_obj_is_claimed_as_a_mesh`
- `render::media::point_cloud::real_files::a_real_ply_loads_its_points`
- `render::media::point_cloud::real_files::every_fetched_sample_is_importable_and_gets_an_asset_type`
- `render::media::point_cloud::tests::extension_recognition_delegates_to_re_importer`
- `render::media::point_cloud::tests::load_point_cloud_reads_positions_and_colors_from_a_real_ply_file`
- `render::media::point_cloud::tests::rerun_importable_extension_covers_all_registered_formats_not_just_point_clouds`
- `render::media::probe::tests::duration_snaps_to_frame_grid`
- `render::media::probe::tests::maps_color_tags`
- `render::media::probe::tests::parses_fraction`
- `render::media::probe::tests::rejects_601_full_range`
- `render::media::probe::tests::rejects_odd_dimensions`
- `render::media::probe::tests::rejects_unknown_and_hdr_color_tags`
- `render::media::probe::tests::rejects_variable_frame_rate_when_rates_differ`
- `render::media::probe::tests::still_image_containers_are_recognised_by_format_name`
- `render::media::probe::tests::supported_audio_accepts_common_codecs`
- `render::media::probe::tests::unsupported_audio_codec_is_typed`
- `render::media::probe::tests::unsupported_layout_is_typed`

### render::vector (7)
- `render::vector::edit::internal_tests::split_segment_new_vertex_lies_on_the_original_curve`
- `render::vector::edit::internal_tests::split_segment_wraps_around_closed_contour_last_edge`
- `render::vector::geometry_tests::rounded_corners_add_vertices`
- `render::vector::geometry_tests::rounded_corners_keep_the_contour_closed`
- `render::vector::geometry_tests::trim_opens_the_contour`
- `render::vector::text::tests::empty_content_yields_no_contours_and_is_not_an_error`
- `render::vector::text::tests::missing_font_file_is_an_explicit_error`

### ui::app (2)
- `ui::app::detached_tests::a_detached_stage_fills_its_window`
- `ui::app::detached_tests::a_detached_timeline_fills_its_window`

### ui::browser (1)
- `ui::browser::spawn_diagnosis::spawn_on_fixture_doc_changes_pixels`

### ui::dock (7)
- `ui::dock::tests::a_detached_panel_is_in_no_zone_and_comes_back_when_its_window_closes`
- `ui::dock::tests::a_panel_never_lives_in_two_places`
- `ui::dock::tests::an_empty_zone_has_nothing_selected`
- `ui::dock::tests::dropping_a_tab_back_where_it_came_from_does_not_reorder`
- `ui::dock::tests::hiding_then_showing_puts_it_back_somewhere_reachable`
- `ui::dock::tests::moving_the_selected_panel_out_leaves_a_selection_behind`
- `ui::dock::tests::putting_a_detached_panel_back_in_a_zone_closes_the_detachment`

### ui::ease (4)
- `ui::ease::tests::applying_shapes_only_the_starting_key_of_the_segment`
- `ui::ease::tests::one_key_alone_is_treated_as_the_start_of_its_segment`
- `ui::ease::tests::the_three_easy_eases_are_three_different_shapes`
- `ui::ease::tests::two_keys_make_one_segment_and_the_later_one_is_not_a_start`

### ui::ease_model (2)
- `ui::ease_model::tests::every_kind_has_at_least_one_handle_except_the_straight_ones`
- `ui::ease_model::tests::moving_a_handle_puts_it_where_it_was_dropped`

### ui::fixture (7)
- `ui::fixture::admit_tests::a_dropped_file_shows_up_on_the_shelf`
- `ui::fixture::admit_tests::a_file_we_cannot_read_is_dropped_quietly`
- `ui::fixture::asset_family_folding::images_are_2d_and_video_is_its_own`
- `ui::fixture::asset_family_folding::point_clouds_and_meshes_are_one_family`
- `ui::fixture::asset_family_folding::unknown_types_fall_through_instead_of_getting_their_own_box`
- `ui::fixture::nest_tests::a_child_comes_under_its_parent_one_step_in_and_hides_when_folded`
- `ui::fixture::row_tests::the_name_column_and_the_band_column_describe_the_same_rows`

### ui::gui (21)
- `ui::gui::a_keystroke_reaches_the_app`
- `ui::gui::a_name_can_be_typed_after_double_clicking_it`
- `ui::gui::a_panel_that_draws_itself_still_draws_after_being_moved`
- `ui::gui::a_tab_dragged_into_another_zone_moves_there`
- `ui::gui::an_emptied_zone_opens_again_while_a_tab_is_held`
- `ui::gui::clicking_a_tab_swaps_the_body_without_losing_the_other_tabs`
- `ui::gui::dragging_a_tab_out_of_the_window_takes_it_off_the_dock`
- `ui::gui::dragging_far_outside_the_window_makes_it_a_separate_window_without_breaking`
- `ui::gui::dropping_a_tab_exactly_where_it_started_is_harmless`
- `ui::gui::every_panel_can_be_shown_without_breaking_the_next_render`
- `ui::gui::hammering_the_same_spot_is_harmless`
- `ui::gui::moving_a_panel_after_hiding_another_ones_body_survives`
- `ui::gui::pressing_twice_without_releasing_is_harmless`
- `ui::gui::releasing_without_pressing_is_harmless`
- `ui::gui::switching_back_and_forth_between_two_tabs_survives`
- `ui::gui::switching_tabs_while_holding_one_is_harmless`
- `ui::gui::the_dock_opens_with_every_panel_reachable_by_a_tab`
- `ui::gui::the_window_still_takes_orders_after_any_storm`
- `ui::gui::the_zone_under_the_pointer_lights_up_while_a_tab_is_held`
- `ui::gui::thrashing_the_pointer_over_the_timeline_is_harmless`
- `ui::gui::typing_in_the_middle_of_a_drag_is_harmless`

### ui::keymap (8)
- `ui::keymap::held_tests::a_modifier_that_arrived_as_its_own_event_still_counts`
- `ui::keymap::held_tests::a_stroke_typed_into_a_field_does_not_also_run_a_verb`
- `ui::keymap::tests::cmd_a_is_select_all`
- `ui::keymap::tests::cmd_k_is_split`
- `ui::keymap::tests::no_two_bindings_claim_the_same_stroke`
- `ui::keymap::tests::plain_a_is_not_bound`
- `ui::keymap::tests::plain_k_is_not_split`
- `ui::keymap::tests::space_is_play_pause`

### ui::session (5)
- `ui::session::selection_invariants::set_none_empties`
- `ui::session::selection_invariants::set_replaces_and_get_returns_it`
- `ui::session::selection_invariants::toggle_twice_returns_to_start`
- `ui::session::selection_invariants::toggled_in_layer_becomes_primary`
- `ui::session::selection_invariants::toggling_out_the_primary_promotes_the_previous`

### ui::stage_widget (4)
- `ui::stage_widget::orbit_tests::dragging_right_turns_the_plate_to_face_right_and_down_tips_it_back`
- `ui::stage_widget::orbit_tests::no_movement_keeps_the_original_angles`
- `ui::stage_widget::plane_tests::any_point_on_any_tilted_plane_comes_back_where_it_started`
- `ui::stage_widget::plane_tests::the_map_sends_the_corners_where_they_were_put_and_comes_back`

### ui::thumbnail (1)
- `ui::thumbnail::real_files::video_data_uri_reads_first_frame_of_sample_mp4`

### ui::timeline_widget (8)
- `ui::timeline_widget::duplicate_tests::a_duplicate_carries_the_whole_layer_and_sits_above_the_original`
- `ui::timeline_widget::edge_tests::head_to_playhead_moves_without_changing_length`
- `ui::timeline_widget::edge_tests::tail_to_playhead_never_walks_off_the_front`
- `ui::timeline_widget::edge_tests::tail_to_playhead_puts_the_end_at_the_playhead`
- `ui::timeline_widget::edge_tests::trimming_never_produces_an_empty_layer`
- `ui::timeline_widget::edge_tests::trimming_the_head_keeps_the_material_still`
- `ui::timeline_widget::edge_tests::trimming_the_tail_shortens_to_the_playhead`
- `ui::timeline_widget::keyframe_shift_tests::move_shifts_keyframes_by_the_same_delta`
