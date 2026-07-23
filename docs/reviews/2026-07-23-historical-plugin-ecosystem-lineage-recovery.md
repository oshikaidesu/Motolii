# Plugin ecosystem lineageの価値回収（Unit 9B、2026-07-23）

状態: **縮小採用**（未処分11 blobの処分、community politicsと旧schemaの分離）

> **後続決定（2026-07-23）**: 本書で暫定名として採用した`Plugin Set`は、[Kit / Plugin Set統合決定](2026-07-23-vism-kit-rack-unification-decision.md)により独立artifactとして廃止した。接続済み一式はRack型Vism Kit、推薦だけの集合はcurator list／feedへ分ける。以下は当時のlineage処分記録として維持する。

対象: historical-only `docs/plugin-ecosystem.md` 12版のうち、Unit 1で処分済みの最終版`9794b686`を除く11版。

関連: [Community distribution model](../community-distribution-model.md)、[Unit 1](2026-07-23-losing-specification-value-recovery.md)、[Unit 9A](2026-07-23-historical-vism-kit-distribution-lineage-recovery.md)、[全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

初版全文と後続11 commit差分を読み、最終版だけでは見えにくかったcommunity設計を現行正本へ戻した。

```text
中央store／人気順／公式dedupe
          REJECT
             ↓
作者の実体 ─ 分散した地図 ─ 利用者の小さな棚
                                 │
                           Plugin Setで伝播
                                 │
                           Project Lockで再現
```

生き残る主張は次の六つである。

1. **中央で似た表現を間引かない。** 類似VHSやGlowの複数実装は許容し、identity、作者、由来、互換を識別可能にする。公式正規版や意味的dedupeを運営責任にしない。
2. **download数を正義にしない。** 累積人気は既知のものを増幅し、登場直後のゲームチェンジャーを沈める。地図は存在と語彙、推薦の時間軸は外部記事、個人index、Setへ置く。
3. **全catalogと日常の棚を分ける。** 利用者は導入済み、Folder、Label、History等の安定したUser libraryで日常を選び、全世界の類似候補を毎回読まない。
4. **界隈のガラパゴスを消さず入口をコピー可能にする。** Plugin Setは人へ一式を渡し、Project Lockは作品を再現する。公式必須セットへ統一しない。
5. **lookとprimitiveの双方を認める。** 完成した意図を一発で使える入口と、再発明を防ぐ部品を対立させず、Vism／Kitの責任で比較する。
6. **地図とloaderを別の時計にする。** discoveryはruntimeより先に価値を持ち得るが、現行identityとprovenanceを固定するB0/B1/B3Hを飛ばして旧GAP-13や仮schemaを復活させない。

## 2. 版ごとに追加された価値

| blob／変更 | 歴史上の追加 | 現在の判定 |
|---|---|---|
| `ce8bc9b2` 初版 | 分散tap、lock、D&D、text authority、verify/repair/doctor | 責任分離と可観測性を採用。schema／CLI名はarchive |
| `f406f24d` | DL数正義の否定 | community原則へ採用 |
| `f0d335fa` | 使用kitのexport/importを主伝播路にする | Plugin Set候補へ改名して採用 |
| `4bddc129` | look／primitiveの重複処分 | Vism／Kit粒度原則へ縮小採用 |
| `f7c85ade` | 類似pluginをuser shelfで耐える | User library原則へ採用 |
| `c54abc88` | primitiveは再発明防止、WGSL look排除ではない | 負例として採用 |
| `7e7b1b42` | 界隈差をkit入口にする | community原則へ採用 |
| `68a5b59f` | 地図、Set、人気の責任分離 | community原則へ採用 |
| `52754f17` | Brewfile／VS Code／Steam Collection等の類例 | 成立理由。製品仕様の根拠にはしない |
| `c25a987f` | DAW圏の写像追記記録 | 本文の無い中間版。archiveのみ |
| `b412e4c0` | KVR、ReaPack、Pack、template等の写像本文 | 成立理由。固有形式は採らない |
| `9794b686` 最終 | AviUtl2 catalog姿勢、早期地図GAP-13 | Unit 1で処分済み。早期価値だけ保持し旧task状態は戻さない |

全12版は加筆型のlineageで、後続で前のcommunity原則が削除されたわけではない。今回の回収は、既に処分済みの最終版を重複countせず、先行11 blobがいつ何を加えたかを個別に処分する。

## 3. 復活させない旧具体

初版は現在のVism／Document／trust境界より前に書かれ、責任分離を壊す具体を多く含む。次はarchiveであり、実装根拠にしない。

- `package id = NodeDesc.id`。現行はpackage、entry、Project instance、artifact identityを分離する。
- `motolii.toml`が`NodeDesc`の鏡という二重正本。
- `tap.toml`、`plugins.lock.toml`、`installed.toml`、`.motoliipack`、`.motolii-kit`のfieldと拡張子。
- lockをDocument外sidecarへ置く固定案。Projectとのatomicity、移動、journal、欠落保持を未検証である。
- `~/.motolii/...`という固定install path。
- XXH3、hash scope、minisign、URL選択をtrust境界とする具体。
- tap優先順だけで同一identity衝突を解決する規則。
- `git fetch → cargo build → register`を通常導入にするsource実行。
- unknown pluginを「警告＋pass-through」で描画できるという旧表現。現行はraw保持／degraded openとstrict export拒否を分ける。
- C ABI／`abi_stable`をV2-1既定にする記述。
- Slint Asset Browser、三画面、カード／一覧、D&D拡張子等の旧UI具体。
- 「GAP-13を今〜M3近傍で実装」という旧進捗。現在のVSM-B0/B1/B3H、Phase D/Eを上書きしない。

## 4. 先例の扱い

旧文書はReaPack、Homebrew Bundle、VS Code Profiles、Steam Workshop Collections、KVR、Ableton Pack、VCV Rack、ComfyUI等を比較した。これらは「地図」「一式共有」「外部キュレーション」「作品再現」が別々に成功し得る成立理由として保持する。

ただし本Unitは2026-07-12当時の記述を歴史証拠として処分したもので、各serviceの2026-07-23時点の現行仕様を再調査したものではない。固有API、URL、人気機構、file formatを現在の実装根拠に使う場合は、一次資料で別途再検証する。

## 5. 固定歴史出典とcoverage

`git log --all --reverse -p -- docs/plugin-ecosystem.md`で初版`69993812`から最終`2cbfc813`まで12 commit／12 unique blobの直線lineageを確認した。初版`ce8bc9b2`を全文、後続11差分を順に読み、最終`9794b686`はUnit 1 receiptと重複するため今回のreceiptへ入れていない。

今回処分した11 blobの完全SHAは`09b-plugin-ecosystem-lineage.tsv`を正本とする。これによりhistorical-only `plugin-ecosystem.md`の全12版は処分済みになる。community／catalog／third-party runtimeを名前に含む他pathの処分はUnit 9C以後へ残し、Unit 9全体完了とはしない。
