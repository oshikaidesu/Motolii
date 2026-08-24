# コンポーネント台帳(機械導出)

実装ファイルの `motolii-component` 契約から生成。手で編集しない。
赤 = 契約の粒に対応する証拠が実装コードに無い、または参照する地図行が採用済でない。

| component | kind | weight | maps | entry | meaning | evaluation | render | observable | 判定 | source |
|---|---|---:|---|---|---|---|---|---|---|---|
| audio.media_soundtrack_input | semantic | render_export(4) | local | AudioProgram::from_view 緑 | project_soundtrack_input 緑 | layer_mix_source 緑 | MixSource 緑 | media_layers_become_mix_sources 緑 | 緑 | next/engine/motolii-audio/src/program.rs:59 |
| browser.asset_context_menu | semantic | convenience(1) | local | OpenContextMenu 緑 | RemoveAssetFromCard 緑 | actions_for 緑 | view 緑 | media_card_layout_is_anchored_below_the_card 緑 | 緑 | next/ui/motolii-browser-pane/src/context_menu.rs:10 |
| browser.media_card_preview | semantic | render_export(4) | local | PreviewMedia 緑 | preview_media_request 緑 | preview_media_target 緑 | media_card_preview 緑 | media_card_double_click_publishes_preview 緑 | 緑 | next/ui/motolii-browser-pane/src/media_preview.rs:8 |
| edit.batch_rename | semantic | core_edit(4) | 785 | BatchRenameSelectedLayers 緑 | apply_selected 緑 | apply_all 緑 | Timeline 緑 | auto_rename_follows_row_order_and_undoes_as_one_step 緑 | 緑 | next/shell/motolii-shell/src/batch_rename.rs:7 |
| export.aspect_preset | semantic | render_export(4) | local | AspectPresetSelect 緑 | AspectPreset 緑 | dimensions_for_aspect 緑 | aspect_preset_row 緑 | aspect_preset_buttons_show_label_and_dimensions 緑 | 緑 | next/ui/motolii-export-pane/src/lib.rs:146 |
| inspector.shape | semantic | core_edit(4) | local | ShapeSectionProjection 緑<br>commit_shape_field 緑 | Width 緑<br>Height 緑<br>Radius 緑 | project 緑<br>commit_shape_field 緑 | shape_section 緑<br>shape_numeric_input 緑 | shape_inspector_changes_geometry 緑 | 緑 | next/ui/motolii-inspector-pane/src/shape.rs:21 |
| io.folder_import_expansion | semantic | core_edit(4) | local | expand_import_paths 緑 | is_supported_import_file 緑 | append_supported_files 緑 | AdmitPaths 緑 | folder_expansion_recurses_in_sorted_order_and_filters_media 緑 | 緑 | next/shell/motolii-shell/src/file_dialogs.rs:96 |
| shell.shape_tool_writer | semantic | core_edit(4) | local | ShapeTool 緑<br>create_drawn_shape 緑 | Create 緑<br>CreatePen 緑 | primitive_path 緑<br>pen_path 緑 | SetShapes 緑 | drawn_shape_adds_selected_layer 緑 | 緑 | next/shell/motolii-shell/src/shape_ops.rs:15 |
| shell.source_preview | semantic | render_export(4) | local | SourcePreview 緑<br>open_source_preview 緑 | update 緑<br>read_preview_frame 緑 | update 緑<br>yuv420p_to_rgba 緑 | view 緑 | source_preview_renders_decoded_frame 緑 | 緑 | next/shell/motolii-shell/src/source_preview.rs:7 |
| stage.shape_tool | semantic | core_edit(4) | local | ShapeTool 緑<br>ShapeToolOverlay 緑 | Select 緑<br>Create 緑<br>CreatePen 緑 | comp_from_screen 緑<br>screen_from_comp 緑 | toolbar 緑<br>preview 緑 | shape_tool_draws_shape 緑 | 緑 | next/ui/motolii-stage-pane/src/shape_tool.rs:17 |
