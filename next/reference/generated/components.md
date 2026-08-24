# コンポーネント台帳(機械導出)

実装ファイルの `motolii-component` 契約から生成。手で編集しない。
赤 = 契約の粒に対応する証拠が実装コードに無い、または参照する地図行が採用済でない。

| component | kind | weight | maps | entry | meaning | evaluation | render | observable | 判定 | source |
|---|---|---:|---|---|---|---|---|---|---|---|
| edit.batch_rename | semantic | core_edit(4) | 785 | BatchRenameSelectedLayers 緑 | apply_selected 緑 | apply_all 緑 | Timeline 緑 | auto_rename_follows_row_order_and_undoes_as_one_step 緑 | 緑 | next/shell/motolii-shell/src/batch_rename.rs:7 |
