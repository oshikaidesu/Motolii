# コンポーネント台帳(機械導出)

実装ファイルの `motolii-component` 契約から生成。手で編集しない。
赤 = 契約の粒に対応する証拠が実装コードに無い、または参照する地図行が採用済でない。

| component | kind | weight | maps | entry | meaning | evaluation | render | observable | 判定 | source |
|---|---|---:|---|---|---|---|---|---|---|---|
| browser.asset_context_menu | semantic | convenience(1) | local | OpenContextMenu 緑 | RemoveAssetFromCard 緑 | actions_for 緑 | view 緑 | media_card_layout_is_anchored_below_the_card 緑 | 緑 | next/ui/motolii-browser-pane/src/context_menu.rs:10 |
| edit.batch_rename | semantic | core_edit(4) | 785 | BatchRenameSelectedLayers 緑 | apply_selected 緑 | apply_all 緑 | Timeline 緑 | auto_rename_follows_row_order_and_undoes_as_one_step 緑 | 緑 | next/shell/motolii-shell/src/batch_rename.rs:7 |
| io.folder_import_expansion | semantic | core_edit(4) | local | expand_import_paths 緑 | is_supported_import_file 緑 | append_supported_files 緑 | AdmitPaths 緑 | folder_expansion_recurses_in_sorted_order_and_filters_media 緑 | 緑 | next/shell/motolii-shell/src/file_dialogs.rs:96 |
