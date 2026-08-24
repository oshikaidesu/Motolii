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
| io.folder_import_expansion | semantic | core_edit(4) | local | expand_import_paths 緑 | is_supported_import_file 緑 | append_supported_files 緑 | AdmitPaths 緑 | folder_expansion_recurses_in_sorted_order_and_filters_media 緑 | 緑 | next/shell/motolii-shell/src/file_dialogs.rs:96 |
