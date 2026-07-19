# M2基盤再締結ゲート（2026-07-15）

ステータス: **M2基盤再締結解除宣言**（main発効済み）。本書はM2基盤再締結のA〜C退出条件を充足したことを記録する正本である。再締結解除はmain上で発効済みであり、発効根拠はPR [#218](https://github.com/oshikaidesu/Motolii/pull/218) / merge SHA `cc87d8aa1d2cf2a2d24937d43e66c11df4aa769c` である。解除のコード証跡前提はPR [#217](https://github.com/oshikaidesu/Motolii/pull/217) / main `fa6850a3981c319973cf120e64976e6f8d79b969` / [PR CI](https://github.com/oshikaidesu/Motolii/actions/runs/29646476618) / [push CI](https://github.com/oshikaidesu/Motolii/actions/runs/29646451595) である。**M3製品実装の着手許可は自動解禁されず**、別のM3入場PRのみがU0/U1依存の再翻訳と実装許可を行える。

## 判断

M2のDocument意味・migration・Undo・評価順・所有権は、後続フェーズが依存する基礎である。ここで意味の変更や審判漏れを残すと、M3以降のUI、キャッシュ、3Dが誤った前提を増幅する。したがって、M2コア締結の撤回後にP1を個別修復しただけでは再締結しない。

`cargo test --workspace`の成功は必要条件であり、再締結の十分条件ではない。意味論、旧project、拒否経路、並行所有権を別々の審判で確認する。

## 現在地（2026-07-15 時点の歴史）

以下は発効宣言当時の未着手・未完了記述である。2026-07-18のA〜C完了証跡表（下記）により退出条件は充足済み。

- [M2コア締結宣言](2026-07-14-m2-core-closure.md)は、レビュー0件でのmerge後にCI未検出のP1が2件見つかり、撤回された
- P1修復 #153/#154はmain到達済みだが、再締結の独立追補レビューは未完了
- Shared EffectのD1l実装はmain到達済み（`a23a4ad`、`74af37e`、lint follow-up `02192c2`）。D3eの専用評価fixture／実装は未着手であり、D1lを含む再締結証跡表への対応付けも未完了
- D5骨格はmain到達済み（`1cf4cb9`）だが、本番プレビューループ、GPU timestamp query収集、10分実機E2Eは未達。M2基盤再締結とは分離して完了を判定する
- M3仕様はドラフトである。#180/#191は先行してmainへ入ったが、依存方向CIと空のUIクレート骨格に限る。これらをM3入場完了の根拠にしない
- PR #176相当の将来境界案は分割処分が進み、planar v1 camera決定とParam Pipeline／Element Domain／Constraint Graphの持越し境界はmain到達済み。CAM-G0以降のcamera実装とD1mは未着手であり、残る差分の棚卸しと再締結証跡への対応付けは未完了

## 再締結宣言の退出条件

次の各項目に証跡が揃うまで、M2基盤を「凍結」「完了」「再締結」と記載しない（要件本文。2026-07-18時点でA〜Cは下記完了証跡表により充足）。

### 充足と閉集合外

本再締結解除宣言で充足と記録する面:

- D1l / D3e、CAM-G0 / D1j / D1k / D3f、D1m
- [Param Pipeline / Element Domain / Constraint Graph 持越し境界](2026-07-16-m2-param-element-constraint-disposition.md)
- migration / validate、command / ownership、doc→render、unknown / export、journal / session
- golden policy、`cargo test --workspace`、独立追補レビュー P0/P1=0

再締結の閉集合外（本宣言で完了主張しない）:

- D5（本番previewループ、GPU timestamp query収集、10分実機E2E）
- 将来のM3 / M5 製品実装作業

### A. 恒久面の閉集合

1. **Shared Effect**: D1l schema/migrationをmainへ到達させ、その後にD3e評価接続を別PRで完了する。inline旧projectの要素数・順序・未知plugin field・pixelを保持し、Definition/Use欠落を型付き拒否する
2. **CompCamera**: [planar v1決定](2026-07-16-m2-comp-camera-decision.md)をmainへ到達させ、CAM-G0既存pixel fixture→D1l後のD1j v5 schema+default migration→D1k runtime契約→D3f接続を別PRで直列化する。v1は特異点のない`PlanarOrthographic`だけを焼き、既存2D pixelを保持する。Spatial/PerspectiveはM5の追加variant決定まで未実装
3. **Param Pipeline / Element Domain / Constraint Graph**: [M2持越し境界](2026-07-16-m2-param-element-constraint-disposition.md)をmainへ到達させる。現行`DocParam`/typed ID/LookAt・Followの解釈を変えず、PP/ED/CG各解凍gate前にUI・Document・plugin ABIへ推測のpipeline/generic domain/graphを焼かない
4. 下記の再締結棚卸しを、対象ごとの小さいdecision/spec PRで採択・延期・棄却する。Draft文書や別ブランチの台帳を暗黙の発注根拠にしない
5. **Project sidecar / session ownership**: D1dの親directory共有`.motolii`衝突とprocess間lock未規定をD1mで修復する。同一directory複数projectの隔離、canonical path alias排他、legacy layoutの非破壊移行を満たすまで保存基盤を閉じない

### 再締結棚卸しの初期集合

| 対象 | mainでの状態 | 再締結までの処置 |
|---|---|---|
| #173 / D1l | 実装main到達済み（`a23a4ad`、`74af37e`、`02192c2`）。再締結証跡表への対応付け未完了 | D3eを独立実装し、D1lのmigration／command／journal証跡を追補レビューで再確認 |
| #176相当の将来境界文書群 | planar v1 camera決定とParam/Element/Constraint持越し処分はmain到達済み。camera実装と残差分監査は未完了 | 一括mergeせず、CAM-G0→D1j→D1k→D3fを直列実装。残差分を再締結証跡表で採択・延期・棄却へ対応付け |
| main上の実装準備台帳にあるU1f/U2f/U2gと、依存欄へ残るD1k等の未翻訳参照語 | 意味決定の記録や参照語はmainにあるが、M3/M2正本のタスク表へ未翻訳 | 本ゲート中は`BLOCKED`。cameraのM2側はCAM-G0/D1j/D1k/D3fへ翻訳し、M3側は再締結後の入場PRで採否・ID・依存を再翻訳 |
| #182 / D5 | 骨格main到達済み（`1cf4cb9`）。本番preview／GPU計測／10分実機E2Eは未達 | M2基盤再締結の閉集合外。別レーンで完了判定 |
| #179 | closed / 未merge | 旧M3入場判断として採用しない。再締結後の新しい入場PRで置換 |

#176相当から棚卸し対象とする最小ファイル集合は、`docs/reviews/2026-07-14-unified-stage-camera-design.md`、`docs/reviews/2026-07-14-m2-exit-param-pipeline-disposition.md`、`docs/implementation-ledger.md`、`docs/interaction-simplicity-model.md`、同ブランチ版のM2/M3仕様である。main到達済みの`2026-07-14-motion-foundation-known-tech-disposition.md`は未merge集合に含めず、#176側との差分だけを採否確認する。差分からM2のschema/runtime/評価順へ影響する文書が追加で見つかった場合は、再締結PRの棚卸し表へ追加する。

### B. 意味の審判

1. D1eを含む旧版コーパスのmigrationがin-place変更なし、再実行冪等、要素数・ID・未知field保持を満たす
2. `Document::validate`が欠落参照、循環、型不一致、非有限値を黙って縮退せず型付き拒否する。`OverrunMode::Black`/`Loop`は保存可能な予約値として`validate`を通し、v1の全評価入口が`UnsupportedOverrunMode`で型付き拒否して黙ってFreezeへ縮退しない（D1g正本）
3. D2のapply/revert、gesture merge、Undo/Redoがランダム操作列を含めて初期状態へ戻る。D1l操作はlifecycle決定のDelete Reject、Unlink、Copy Local、orphan保持、各操作1 Undoを個別に審判する
4. D3のmask/group/effect/transform/LookAt・Follow・Parent評価順を意味論ゴールデンで固定する。D3eは非隣接共有、Group合成後1回、preview/export同一、欠落typed errorを個別に審判する。D3fは既定planar cameraで既存2D pixel不変とpreview/export同一を追加する
5. 単一writerと`Arc<Document>`スナップショットを並行テストで確認し、UI/workerから直接書き換える公開口がない
6. D1fの未知plugin保持・警告とD6のdegraded plugin書き出し拒否を結合し、未来versionが既知扱いへ迂回しないことを確認する
7. ジャーナルの破損・途中書き込み・世代不一致を型付きで拒否し、復旧がDocument正本を上書きしないことを確認する。D1mで同一directoryの複数projectがsidecarを共有せず、別process/path aliasの同時read-write openを即時typed rejectする
8. 意味論ゴールデン更新禁止ゲートと`cargo test --workspace`が全緑である

### 完了 A〜C 証跡表（2026-07-18）

各行は main `fa6850a3981c319973cf120e64976e6f8d79b969`、PR [#217](https://github.com/oshikaidesu/Motolii/pull/217)、PR CI [29646476618](https://github.com/oshikaidesu/Motolii/actions/runs/29646476618)、push CI [29646451595](https://github.com/oshikaidesu/Motolii/actions/runs/29646451595)、判定日 **2026-07-18** を共通証跡とする。命名審判は[追補レビュー](2026-07-18-m2-foundation-supplementary-code-review.md)の既存節・試験名のみを用いる。

| 面 | 判定 | 命名審判・レビュー節 |
|---|---|---|
| A 恒久面の閉集合 | レビュー+main | D1l: `d1l_effect_definition`、`d1l_v2_lifecycle_commands`、`d1l_writer_prepare`、`d1l_journal_v1_compat`。D3e: `d3e_shared_effect_eval` P1–P6・N1・N3、`d3e_preview_export_same::p7_preview_and_export_share_final_render_path`。CAM: `cam_g0_planar_identity_matches_semantic_oracle`、`d1j_comp_camera`、追補レビュー D1k runtime camera 節、D3f各試験（`d3f_comp_camera_eval`、`d3f_preview_export_camera`）。D1m: `d1m_sidecar_paths`、`d1m_session_lock`、`d1m_legacy_migration`、`d1m_public_api_closure`。Param/ED/CG: [2026-07-16-m2-param-element-constraint-disposition.md](2026-07-16-m2-param-element-constraint-disposition.md) |
| B migration/validate | 自動 | `d1e_migrate`、`d1h_validate`、`d1j_comp_camera`、OverrunMode拒否（追補レビュー記載範囲） |
| B command/ownership | 自動+レビュー | `d2_command`、`d8_ownership`、`mut_document_deny`、D1l lifecycle試験（上記A行） |
| B doc→render | 自動 | D3e / D3f / CAM-G0（上記A行の評価・camera試験） |
| B unknown/export | 自動 | `d1f_unknown_plugin`、`d6_audio_mux` degraded/future/contract-only拒否 |
| B journal/session | 自動+レビュー | `d1d_journal`、`d1m_session_lock`、`d1m_public_api_closure` |
| C 追補レビュー | レビュー | [2026-07-18-m2-foundation-supplementary-code-review.md](2026-07-18-m2-foundation-supplementary-code-review.md) 判定表 P0=0・P1=0 |
| 全体 | 自動 | golden policy + `cargo test --workspace`（上記2 CI URL、`fa6850a3981c319973cf120e64976e6f8d79b969`） |

### C. 独立レビュー

1. 再締結PRとは別に、実コードを対象とする追補レビュー記録を`docs/reviews/`へ残す。実装PRの作者と追補レビューの主担当を分ける
2. レビューは少なくとも、schema/migration、command/ownership、doc→render評価、未知plugin/書き出し拒否の4面を確認する
3. P0/P1が0件になるまで再締結しない。修復は1件1PRとし、テスト期待値の書換えで閉じない
4. レビュー記録とCI証跡なしに管理者権限でmerge条件を迂回しない

## M3への影響

- 再締結解除宣言（PR [#218](https://github.com/oshikaidesu/Motolii/pull/218) / `cc87d8aa1d2cf2a2d24937d43e66c11df4aa769c`）はmain上で発効済みである
- **M3製品実装**（Documentを読むUI、domain intent、preview、入力、timeline、plugin panel等）**の着手許可は自動解禁されない**
- **別のM3入場PRのみ**が、U0/U1依存の再翻訳と実装許可を行える
- 許可済みの先行到達はmain上の#180/#191（依存方向CIと空UIクレート骨格）に限る。これらをM3入場完了の根拠にしない
- 依存ゼロで製品コードを変更しない調査・fixture作成は可能。ただし結果を公開APIや永続形式へ焼かない

## 発注順

1. ~~発効宣言PRをmainへ到達させる~~（2026-07-15完了）
2. ~~D1l実装をmainへ到達させる~~（`a23a4ad`、`74af37e`、`02192c2`）
3. ~~D3eを最新D1l型から実装する~~（#217 / `fa6850a3981c319973cf120e64976e6f8d79b969`）
4. ~~#176相当のcamera／Param・Element・Constraint決定を証跡表へ対応付け、CAM-G0→D1j→D1k→D3fを直列実装する~~（#217 / `fa6850a3981c319973cf120e64976e6f8d79b969`）
5. ~~D1mを独立実装・レビューし、project-scoped sidecarとsession ownershipをmainへ到達させる~~（#217 / `fa6850a3981c319973cf120e64976e6f8d79b969`）
6. ~~M2追補実コードレビューを行い、P0/P1=0を確認する~~（[2026-07-18-m2-foundation-supplementary-code-review.md](2026-07-18-m2-foundation-supplementary-code-review.md)）
7. ~~本書A〜Cの完了証跡表を充填し、ステータスを再締結解除宣言へ変更する~~（#218 / `cc87d8aa1d2cf2a2d24937d43e66c11df4aa769c`）
8. その後にのみ、別のM3入場PRでU0/U1依存と自動審判を最新mainへ再翻訳する

D5統合/E2Eは手順7の再締結必須条件に含めない。順序を変える場合は本書の改訂PRを先に出す。
