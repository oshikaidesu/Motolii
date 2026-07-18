# VSM-A2S — 旧CLI ProjectV1 migration処分

作成日: 2026-07-17

状態: **設計決定／VSM-A2実装可**。VSM-A2で中央`migrate_plugin_params`を削除するとき、旧CLI `ProjectV1<HashMap<String, Value>>`のSine v1 `amp`互換をどう扱うかを確定する。

関連文書: [A0D contract／migration所有](2026-07-17-vism-a0d-contract-migration-ownership-decision.md)、[A0S Contract Catalog](2026-07-17-vism-a0s-contract-catalog-spec.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[A1公開crate境界](2026-07-17-vism-a1-public-crate-boundary-spec.md)

## 1. 決定

旧CLI `ProjectV1`のSine v1 `amp`読込互換は維持する。ただし、`motolii-plugin`の中央`migrate_plugin_params`、Sine ID match、plugin固有callbackは削除する。

`motolii-cli::project`にprivateなlegacy adapterを置き、`PluginRuntime`のimmutable catalogに登録された`PluginContract.migrations`を読む。adapterは`HashMap<String, Value>`のcloneへ宣言的`MigrationOp::RenameParam`だけを適用し、成功後にexecutorの現行`NodeDesc::resolve_params`へ渡す。

```text
ProjectV1 raw params（不変）
  ↓ clone
catalogの宣言的migration chain
  ↓ RenameParamだけをlegacy Value mapへ適用
current NodeDesc::resolve_params
  ↓
ResolvedParams
```

これはDocument recipe migrationの流用ではない。Documentは`BTreeMap<String, DocParam>`をHost interpreterがprepared cloneへ移行する。旧CLI adapterは使い捨て`ProjectV1`の`HashMap<String, Value>`だけを扱い、`DocParam`、Document revision、Undo、保存bytesへ触れない。

## 2. 採択理由

| 案 | 判定 | 理由 |
|---|---|---|
| 旧`amp`互換を削除 | 棄却 | 既存ProjectV1を理由なく読めなくし、FG-C4 fixtureを期待値変更で消す |
| `amp`を明示拒否 | 棄却 | 同じ後方互換退行であり、A2のcrate移動に必要ない |
| 中央Sine ID matchを維持 | 棄却 | migration意味がplugin contractでなくHost直書きへ戻る |
| plugin Rust callbackを呼ぶ | 棄却 | Project openで任意codeを実行しないA0D契約を破る |
| Document `prepare_plugin_recipe`へ変換して流用 | 棄却 | A0Sが分離した`DocParam` recipeと旧CLI Value mapを混同する |
| contractを読むprivate legacy adapter | **採用** | 旧互換を保ち、migration意味を外部contractへ移し、公開面を増やさない |

## 3. legacy adapter契約

adapterはplugin IDを分岐に使わない。対象contractの`current_version`まで、保存versionから隣接stepを順に読む。

`RenameParam { from, to }`の規則:

| clone上のshape | 結果 |
|---|---|
| `from`あり、`to`なし | 値をそのまま移動 |
| `from`あり、`to`あり | typed conflict |
| `from`なし、`to`あり | 保存versionとshapeのtyped矛盾 |
| `from`なし、`to`なし | 保存versionとshapeのtyped矛盾 |

追加規則:

1. `saved_version == current_version`はmigrationなしで現行parameter解決へ進む。
2. future versionはtyped拒否し、downgradeしない。
3. 隣接step欠落はtyped拒否する。
4. unknown migration operationを推測実行しない。v1では`RenameParam`以外を追加しない。
5. raw `ParamDriverV1.params`は変更しない。全step成功後だけcloneを公開する。
6. default補完と未知parameter拒否は既存`NodeDesc::resolve_params`だけが担当する。
7. adapter、error、helperを`motolii-plugin`公開façadeへ追加しない。

失敗分類はCLI内の構造化errorとして、少なくともfuture、step missing、rename conflict、old shape mismatchを区別する。文字列比較を制御フローに使わない。

## 4. VSM-A2公開façadeレビュー

現行Sineが必要とする型はすでに`motolii-plugin`から到達可能である。

- plugin契約型: `ParamDriverPlugin`、`ParamDriverContext`、`PluginContract`、`MigrationStep`、`MigrationOp`
- descriptor／値型: `NodeDesc`、`ParamDef`、`PluginId`、`PluginKind`、`ResolvedParams`、`PluginError`、`ValueType`
- A1 façade再export済み: `DataTrack`、`Value`、`Fps`、`RationalTime`

GPU、wgpu、bytemuckはSineで使わない。`sample_count` private helperや`RationalTimeError`を便宜で再exportしない。外部Sine crateは公開された`RationalTime`／`Fps`操作だけで半開区間のsample countを計算し、移動前値列fixtureで意味一致を固定する。

追加façadeが必要に見えた場合、A2実装者は代替helperを公開せず停止し、必要な型・操作・最小fixtureを返す。

## 5. VSM-A2実装範囲

VSM-A2は次を同じ実装コミットで行う。

- `plugins/motolii-plugin-sine`がSine executor、version 2 contract、v1→v2 rename planを所有する。
- `motolii-plugin::reference`からSine executor／contract／migration plan／ID断言を削除する。
- 公開`migrate_plugin_params`と中央Sine ID matchを削除する。
- `motolii-plugins-firstparty`が外部Sine contract／executorを合成する。
- CLIのprivate legacy adapterへ旧ProjectV1 fixtureを接続する。
- fixed first-party catalog／executor ID集合を維持する。
- external plugin allowlist、公開path閉集合、purity、値列fixtureをSineへ適用する。

非目標:

- ParamDriver input port、provider／consumer、BeatEvents。
- `.vism` package、loader、generator、Kit。
- Document schema、永続upgrade command、migration algebra追加。
- A3以降、他reference pluginの同時移動。

## 6. 必須fixture

1. 外部Sine contractにversion 2と`1→2 RenameParam amp→amplitude`がある。
2. Document prepared migrationはcloneだけを変更し、raw recipe、revision、Undoが不変。
3. 旧ProjectV1 `amp`はprivate legacy adapterで読め、raw paramsは不変。
4. ProjectV1のboth-key conflict、old shape mismatch、future、chain欠落がtypedに分かれる。
5. 移動前と同じcontext／paramsでDataTrack値列が一致する。
6. 非有限入力と型不一致をtyped拒否する。
7. Sine単体とassembled first-party registryのpurityが通る。
8. `motolii-plugin`以外の依存、private path、panic経路、中央Sine ID matchが機械検査で赤になる。
9. `cargo test --workspace`と`git diff --check`が通る。

