# RenderCtx解凍手続きlineageの価値回収（Unit 4D、2026-07-23）

状態: **観察**（cutoff 2 historical blobの処分完了）

対象: `docs/reviews/2026-07-12-M2E-7-render-ctx-thaw.md`のcutoff全2版。

関連: [M2E-7解凍手続き](2026-07-12-M2E-7-render-ctx-thaw.md)、[凍結ゲート](2026-07-10-freeze-gate-declaration.md)、[第一監査回収](2026-07-23-historical-first-code-audit-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

2版は、凍結済みplugin traitを壊す必要が生じた時に、理由、旧データmigration、pixel oracleの三面を分けて解凍した証跡である。初版で`RenderCtx`を導入し、第二版で`FilterNode`が呼出元Qualityを捨てて`FINAL`へ戻していた配線穴を製品経路の試験で閉じた。

- **現行決定として保持**: Filter/Compositeのper-call情報は`#[non_exhaustive] RenderCtx`へ集約する。時刻とQualityはHostが明示的に渡す。
- **成立理由として保持**: plugin数で乗算するtrait引数追加を一度のContext化で止める。型が存在するだけでなく、製品経路が値を転送する試験を要求する。
- **非証明範囲を固定**: `instance`、`lookbehind`、`temporal_footprint`は予約fieldであり、Repeater instance解決、前後frame供給、cache key窓、simulation runtimeの実装ではない。
- **棄却する誤読**: `RenderCtx`を全plugin kind共通Contextへ拡張すること、予約fieldをDocumentへ保存すること、Qualityをtexture解像度から推測すること。

## 2. 二版の処分

| blob | 変化 | 現在の判定 |
|---|---|---|
| `6cc1332d` | Filter/Compositeを裸の`t`から`RenderCtx`へ解凍。三点セットを記録 | **現行決定へ採用** |
| `60e2f515` | `FilterNode`の`FINAL`固定を除去し、Draft Qualityの製品配線試験を追補 | **現行実装へ採用**。初版だけでは配線完成を証明しない |

## 3. 現行コードとの照合

`motolii-plugin::RenderCtx`は現在も`#[non_exhaustive]`で、`t`、`quality`、`instance`、`lookbehind`、`temporal_footprint`を持つ。外部crateは`RenderCtx::new`から構築し、Filter/Composite、nodes、scaffold、purity fixtureが同じ境界を使う。

`motolii-nodes`のFilter経路にはDraftの`resolution_scale`を観測する試験があり、Quality転送の追補も生きている。一方、`instance`と`lookbehind`は現行constructorで`None`、`temporal_footprint`はzero-window defaultである。型の席と実行責任を混同しない。

予約の後続は既存の責任へ戻す。

- `InstanceIndex`: F-7のinstance評価と合成意味。
- `CompLookbehind`: F-11のHost所有frame windowと自己参照切断。
- `TemporalFootprint`: F-12/SIM-2、M4 cache key、admission control。

これらは同じfieldに見えても一括解凍しない。必要入力、寿命、cache寄与、Quality縮退、preview/export一致をそれぞれの後続fixtureで閉じる。

## 4. 解凍手続きから残す型紙

1. **理由**: どの既存契約が将来の利用者を阻害し、変更コストがどこで乗算するかを書く。
2. **migration**: Document、Project、plugin parameter、source APIを別々に処分する。「serde変更なし」をmigration不要の根拠として明記する。
3. **oracle**: pixel意味が不変ならgoldenを更新せず、配線した新しい意味は専用probeで観測する。
4. **追補**: public型の存在だけで終えず、最終製品経路が値を落とさないことを試験する。

この型紙は将来の公開API解凍にも再利用できるが、三項目を書けば自動的に変更が承認されるという意味ではない。現行のclosed contract、負例、停止線が先に必要である。

## 5. 復活させないもの

- Filter/CompositeへQuality、instance、lookbehind等を裸引数として再追加すること。
- TextureRefの解像度差からDraft/Finalを推測すること。
- `RenderCtx`予約fieldをそのままDocument schemaまたはVism wireへ直列化すること。
- `instance.is_none()`をinstance 0、空lookbehindを透明frame等へ黙ってdefault化すること。
- LayerSource/ParamDriverの既存Contextを、利用者と意味表なしにRenderCtxへ統合すること。
- 初版のworkspace全緑だけでQuality製品配線も証明済みとすること。

## 6. 固定歴史出典とcoverage

初版`6cc1332d`を全文で読み、第二版`60e2f515`の差分を確認した。処分した2 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/04d-render-ctx-thaw.tsv`を正本とする。cutoff総数1,797のうち処分済みは253、未処分は1,544である。
