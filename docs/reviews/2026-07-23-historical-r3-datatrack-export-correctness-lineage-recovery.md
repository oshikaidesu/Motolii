# R3/DataTrack統合reviewの価値回収（Unit 5M、2026-07-23）

状態: **当時の軽微修正維持／後続の半開総尺を優先／helper集約driftあり**

対象: [R3/DataTrack統合review](2026-07-09-R3-datatrack-review.md) cutoff全3版（2,406 bytes）

関連: [M2仕様](../specs/M2-document-model.md)、[Unit 4C-2 D1 code audit回収](2026-07-23-historical-d1-code-audit-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

3版の意味差はproject名の訂正だけで、R3/R5/R6統合時の軽微修正を1 lineageとして処分できる。未知paramを黙殺しないこと、frame数を判定不能なまま0へ潰さないこと、`VideoSource`語彙、test helper集約方針は現在も正しい。

ただし当時の[x]を現行全体の永久完了にはしない。export長は後続M2E-17の「総尺・半開区間」へ精密化され、Vec2Axes fallbackはvalidationを抜けたmissing runtime dataの防御へ縮小された。また後発media testにlocal `tmp_dir`が2つ再発している。

## 2. 3版の処分

| 主題 | 処分 |
|---|---|
| 未知plugin paramをtyped reject | **維持**。現行runtime／NodeDesc解決にも定着 |
| nb_frames欠落時duration fallback | **維持**。exact rationalの半開総尺へ後続精密化 |
| 両方欠落時IndeterminateExportLength | **維持**。0 frameや推測値へ潰さない |
| Vec2Axes型不一致fallback | **縮小**。Document型不一致のsilent coercionではない |
| ExternalTexture→VideoSource | **維持**。旧語彙を復活させない |
| gpu_or_skip／tmp_dir集約完了 | **当時完了／現行drift**。public helperはあるがlocal重複が再発 |

## 3. 現行コードとの照合

- CLI project adapterは未知plugin／param、parameter型不一致をtyped errorで拒否する。
- export frame countは`nb_frames`を優先し、欠落時はdurationをexact rationalでframe floorへ変換し、両方が無ければtyped errorにする。
- source durationは「最終PTS+1」ではなく半開区間の総尺である。R3の旧説明から時間意味を逆戻りさせない。
- DataTrackのfallbackはruntime data欠落時の防御として残るが、load／validate境界の型不一致拒否を迂回しない。
- `motolii-testkit::tmp_dir`は存在する一方、mediaのcancel／roundtrip testにlocal helperが残る。これは狭いcleanup候補で、歴史回収へcode変更を混ぜない。

## 4. 再入場条件

1. DataTrack identityはGAP-21／VSM-B0/B2でproducer、version、output、source、provider／consumer、materialize方式を比較する。R3の文字列IDを恒久identityへ昇格しない。
2. 解析producerはANA系列で入り、unknown param拒否を弱めず、missing dataとinvalid definitionを別診断にする。
3. export lifecycle、readback、color、release acceptanceは既存GAP-26／29／31／32へ送る。R3から重複gapを作らない。
4. test helper cleanupはlocal重複だけを既存public helperへ移し、回収docs、export意味、goldenを変更しない。

## 5. 復活させないもの

- 未知paramの黙殺、型違いの0.0 coercion、判定不能なexport長の0 frame化。
- durationを最終PTSとして扱う`+1`方式。
- `ExternalTexture`旧名、ProjectV1 skeleton、旧plugin APIを現行製品契約へ戻すこと。
- R3の[x]からDataTrack identity／解析producer／export pipeline完成を推論すること。
- helper cleanupをencoder hardeningやcache改修と同じ変更へ束ねること。
- testを通すためgolden、tolerance、時間期待値を変更すること。

## 6. 固定証跡とcoverage

3 blobの完全SHAはreceipt `05m-r3-datatrack-export-correctness.tsv`を正本とする。合計2,406 bytes。

本Unit後のstrict progressは420 / 1,797（23.4%）、未処分1,377である。
