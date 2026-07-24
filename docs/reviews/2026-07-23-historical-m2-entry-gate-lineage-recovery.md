# M2入口ゲートlineageの価値回収（Unit 4F、2026-07-23）

状態: **観察**（cutoff 43 historical blobの処分完了）

対象: `docs/reviews/2026-07-11-M2-entry-gate.md`のcutoff全43版。

関連: [M2入口ゲート](2026-07-11-M2-entry-gate.md)、[第一コード監査回収](2026-07-23-historical-first-code-audit-lineage-recovery.md)、[RenderCtx回収](2026-07-23-historical-render-ctx-thaw-lineage-recovery.md)、[oracle ruleset回収](2026-07-23-historical-test-oracle-ruleset-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このlineageはM2の全機能表ではなく、Document／journalへ複数エージェントが入る前に、誤りを増幅する穴だけを先に閉じた入口ゲートである。43版の価値は18項目の最終チェックより、次の運用モデルにある。

1. **影響を共有するtaskだけ止める**。D1/D2/D3/D7/D8を止め、独立したD4/D6までmilestone一括で止めなかった。
2. **A→B→Cの順で閉じる**。まず審判を起こし、次にplugin作者数で乗算する穴を閉じ、最後に恒久schemaの意味と骨格を固定した。
3. **観察・決定・審判を分ける**。既存監査の翻訳でも、保護対象、D1-prelude、型配置、色、時刻、互換性には新しい決定が必要だと訂正した。
4. **完了済みを再び未達へ戻せる**。M2E-9は共有uniformの偽陰性が見つかった時点でチェックを外し、renderごとのsubmitと負例が入るまで再完了しなかった。
5. **達成の範囲を固定する**。2026-07-12の「達成」は当時の並列着手条件の達成であり、後続へ送ったPB/TM/GR/CQ/LG残件や、その後のM2再締結まで永久に完了したという意味ではない。

したがって現行M2入口ゲートの達成表示は保持する。後年の残件は[第一コード監査回収](2026-07-23-historical-first-code-audit-lineage-recovery.md)と各現行gateで別に追い、歴史的達成を未達へ巻き戻さない。

## 2. 43版で起きた意味変更

| 段階 | 版の変化 | 現在の処分 |
|---|---|---|
| 初版→運用レビュー | M2全体ではなくDocument／journal依存taskだけを停止。`[自動]`と`[レビュー]`を分離 | **採択済み**。riskを共有しないtaskまでgateで包摂しない |
| 「新規判断なし」の訂正 | 翻訳中にもM2E-2/12/13/15/16/17の意味決定が必要と判明 | **採択済み**。既存所見の整理を仕様判断不要と偽らない |
| M2E-2初期案の敗北 | labelでtest変更を止める案は通常TDDを塞ぎ、実装者が自己解除できた | **棄却理由を保持**。限定したoracle、protected diff、CODEOWNERS、rulesetへ置換 |
| M2E-2 bootstrap | 保護assetとCODEOWNERSを先にmergeし、その後rulesetを有効化、別PRでblockedを実地確認 | **運用patternとして保持**。保護機構の自己deadlockを避ける順序 |
| M2E-4二分割 | tolerance定数の保護側変更と、実装／走査側変更を同じPRへ混ぜられなかった | **採択済み**。oracle更新と製品変更を分離する具体例 |
| M2E-9再開 | registry一括purityを一度完了したが、1 submit内の2 renderが共有uniform状態を隠した | **負例として保持**。完了印より審判の反証を優先 |
| M2E-12循環解消 | 「D1前にD1の一部を実装」の循環を、互換骨格だけのD1-preludeへ分割 | **成立理由を保持**。preludeをschema本体へ再拡張しない |
| M2E-13分離 | 保存Color、render中間、blend空間、`precise_color`配線を一項目に混ぜず、意味決定と経路予約へ分割 | **採択済み**。保存値がsRGBだからsRGB blendという推論を禁止 |
| M2E-15具体化 | `coreまたはdoc`という裁量を撤回し、恒久IDをdoc所有へ固定。load、atomic insert、retired ID負例を追補 | **採択済み**。予約型だけでruntime参照解決済みとしない |
| M2E-16具体化 | 「負を正規化」を、負の分母と正当な負時刻へ分解。公開演算と評価側のoverflow握り潰しも追補 | **採択済み**。serde正準化だけで呼び出し側安全を完了扱いしない |
| M2E-17互換判断 | 半開区間化でProjectV1の91→90 frame変更を明示し、使い捨て形式に限る正当な期待値更新として分離 | **歴史判断として保持**。恒久Documentの期待値更新へ一般化しない |
| M2E-18達成 | `precise_color`を合成分岐点まで通し、A/B/C全18項目を閉じた | **当時のgate達成**。linear blend実装済みの証明ではない |

## 3. 現行コードとの照合

2026-07-23の現行コードで、入口ゲートの代表的な機械審判を再実行した。

| 責任 | 現行証拠 |
|---|---|
| 審判の覚醒 | `skip_policy` 9件、`protected_assets` 9件、`tol_literals` 5件が緑 |
| plugin purity | `purity` 10件が緑。共有uniformの状態付きFilter負例とregistry一括検査を含む |
| Document境界 | `mut_document_deny` 8件、`unknown_keys_roundtrip` 2件、`canonical_from_core` 1件が緑 |
| 時刻不変条件 | `motolii-core` 84 unit + 2 proptest + 1走査が緑。負分母、ゼロ分母、`i64::MIN`、正のspeedを含む |
| Quality転送 | `motolii-nodes --test filter_node` 8件が緑。Draft Qualityの`RenderCtx`転送を含む |

これは18項目すべてを2026年時点の同じ形で再証明するものではない。rulesetのlive状態は[Unit 4E](2026-07-23-historical-test-oracle-ruleset-recovery.md)、RenderCtxの現行意味は[Unit 4D](2026-07-23-historical-render-ctx-thaw-lineage-recovery.md)、第一監査の残件は[Unit 4C-3](2026-07-23-historical-first-code-audit-lineage-recovery.md)を正本とする。

## 4. 再利用するgate設計

- gateはmilestone名で広く止めず、同じ恒久面・公開面・審判を共有するtask集合だけを止める。
- 実装順は依存だけでなく、**審判を有効にする順**にする。固定runnerより先にskipと自己参照oracleを塞ぐ。
- 保護機構が自分自身を保護する場合、bootstrap merge、設定有効化、独立負例の三段に分ける。
- `[x]`は不可逆ではない。新しい反例が完了条件を破ったら未達へ戻し、追補証拠を同じ行へ残す。
- schemaへ入る前のpreludeは、循環を切る最小骨格だけに限定し、製品schemaを先回りしない。
- gate達成後に見つかった別責任の残件は、元gateを永久未達へ戻さず、現行の所有先と停止線へ移す。

## 5. 復活させないもの

- M2全体または後続milestone全体を入口ゲート一つで停止すること。
- 自由に付与できるlabelをoracle変更の承認権限として使うこと。
- 通常TDDの新規testまで保護asset扱いし、実装とtest追加を一律分離すること。
- CODEOWNERSの存在だけでmerge拒否が成立したとみなすこと。
- reserved fieldや分岐点の存在を、Host解決、cache窓、linear blend等のruntime実装済み証拠にすること。
- ProjectV1の互換判断を恒久Documentへ転用すること。
- 当時のPR番号、test総数、ruleset IDだけを現在のlive保証にすること。
- 入口ゲートの達成を根拠に、第一監査で後続へ送ったPB/TM/GR/CQ/LG残件も完了と読むこと。

## 6. 固定歴史出典とcoverage

初版`1fc3d91c`を全文で読み、以後の親子差分、分岐snapshot、最終版`04f778a6`を確認した。処分した43 unique blob（995,857 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04f-m2-entry-gate.tsv`を正本とする。cutoff総数1,797のうち処分済みは297、未処分は1,500である。
