# M5-C0 Schema preflight — 公開型・wire・Document versionの停止線

状態: **preflight完了／仕様化STOP**（2026-08-02）

M5-C0の意味決定とprivate semantic fixture（`c59c05c5`、5/5）を受け、公開Rust型、serde／wire、
Document version、provider identityを実装せずに再照合した。意味が閉じたことはschema形が閉じたことを
意味しない。M2恒久焼き込み防止に従い、未決の公開契約をコードへ先行追加しない。

## 現行コードと正本

| 対象 | 現行target／証跡 | 成立していること | 未成立／停止理由 |
|---|---|---|---|
| Camera Document | `crates/motolii-doc/src/schema.rs` `CompCameraDoc::PlanarOrthographic` | internally tagged `kind=planar_orthographic`、`DocParam`、既存Planar意味 | Spatial／Perspective variant、Camera Object、active bindingの公開形は未決 |
| Document version | `crates/motolii-doc/src/persist.rs` `READER_VERSION/WRITER_VERSION=5`、`validate.rs` `MIN_READER_VERSION_FOR_COMP_CAMERA=5` | v5の既存camera migration／旧reader拒否 | 新variant追加時の版上げ、migration、旧project不変条件は新decisionが必要 |
| Runtime camera | `crates/motolii-doc/src/camera_eval.rs` → `crates/motolii-core/src/camera.rs` | Planarの時刻評価、FrameDesc整合、typed refusal | `CompCamera`をObservation Contractへ昇格しない。Spatial provider routeなし |
| Provider identity | [Camera Object / Provider決定](2026-07-24-camera-object-provider-decision.md) | stable object、provider identity／versionが意味上必要 | package／entry／artifact identity、registry、wire形の正本が未決。static `PluginId`や名前を流用しない |
| Resource gate | [M4仕様](../specs/M4-cache-and-analysis.md) K1a、implementation ledger | Host ResourceLedger／hard budgetが意味として必要 | ResourceLedger、全owner accounting、admissionは未実装。M5 runtime／compiled assetへ接続しない |

## 既存契約接続票

| AUTHORITY | INTERNAL TARGET | OWNER | WRITE ROUTE | GAP | RESOLUTION ROUTE | DISPOSITION |
|---|---|---|---|---|---|---|
| M5-C0意味決定、M2 camera schema | `CompCameraDoc`、`Composition.camera`、v5 migration | Document／D2 single writer | schema＋明示migration＋既存validate | Spatial／Perspectiveの永続形がない | `REUSE`既存Planar保持 → `SPECIFY`を別decisionで閉じる | `RESOLVE` |
| M2 Document version／恒久焼き込み防止 | `READER_VERSION`、`WRITER_VERSION`、`MIN_READER_VERSION_FOR_COMP_CAMERA` | motolii-doc persistence | version bumpとmigration test | 新variantを旧readerが黙殺／再保存し得る | version方針・旧project oracleを先に仕様化 | `RESOLVE` |
| Camera Provider決定 | provider pin、active binding | Host provider resolution | typed preflight → D2 atomic adoption | package／entry／version identityの発行者・寿命不明 | Vism identity／plugin境界と照合して別decision | `RESOLVE` |

## 4軸の判定

1. **公開列挙型の形:** 既存`PlanarOrthographic`は保持する。追加variantの名前・parameter・pose表現は未決。
2. **serde／後方互換:** 既存のinternally tagged wireを互換baselineとするが、新variantの受理、未知kind、
   旧reader拒否、roundtrip、migration値は未決。
3. **Document version:** 現行v5を確認した。新variantがネスト恒久面へ入るならversion／minimum readerを
   同時に決める必要があるが、上げ幅と旧project migrationは未決。
4. **provider identity:** stable objectとprovider identity／versionは意味上必要だが、package／entry／artifact
   の発行者、寿命、rename／fork、欠落診断、wire形式は未決。既存`PluginId`や文字列名から推測しない。

`AUTHORITY_SPAN: MULTIPLE`、`OWNER_CLOSURE: MULTIPLE_KNOWN`、`CAUSE_CLOSURE: LOCALIZED`、
`CONTRACT_CLOSURE: UNRESOLVED`、`ORACLE_CLOSURE: PARTIAL`、`REUSE_CLOSURE: CHOICE`、
`CONTRACT_IMPACT: PERMANENT`、`VIEW_PROFILE: WIDE` とする。公開契約の未決が残るため、狭い実装粒へ圧縮しない。

## STOP／次の受け渡し

- `CompCameraDoc::Spatial`、`Observation`公開型、provider registry、serde／JSON、version定数を追加しない。
- `camera_eval.rs`やrendererへruntime routeを追加しない。M4 K1aのAPI／backend型も発明しない。
- `glam` private fixtureの型を公開型へ昇格しない。fixtureは意味oracleでありschema authorityではない。
- 次は、provider identity／package境界とDocument migration権限を含む**別のdecision粒**を閉じた後に、
  `C0-Schema`を再分類する。そこが未決のままなら、M5製品runtimeはWAITのままとする。

