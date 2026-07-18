# M2基盤再締結ゲート（2026-07-15）

ステータス: **発効宣言**。本書のmain到達でゲートを発効し、別の再締結PRで退出条件の全証跡が揃うまでM3の製品実装を停止する。本書のmerge自体は再締結を意味しない。

## 判断

M2のDocument意味・migration・Undo・評価順・所有権は、後続フェーズが依存する基礎である。ここで意味の変更や審判漏れを残すと、M3以降のUI、キャッシュ、3Dが誤った前提を増幅する。したがって、M2コア締結の撤回後にP1を個別修復しただけでは再締結しない。

`cargo test --workspace`の成功は必要条件であり、再締結の十分条件ではない。意味論、旧project、拒否経路、並行所有権を別々の審判で確認する。

## 現在地

- [M2コア締結宣言](2026-07-14-m2-core-closure.md)は、レビュー0件でのmerge後にCI未検出のP1が2件見つかり、撤回された
- P1修復 #153/#154はmain到達済みだが、再締結の独立追補レビューは未完了
- Shared EffectのD1l実装はmain到達済み（`a23a4ad`、`74af37e`、lint follow-up `02192c2`）。D3eの専用評価fixture／実装は未着手であり、D1lを含む再締結証跡表への対応付けも未完了
- D5骨格はmain到達済み（`1cf4cb9`）だが、本番プレビューループ、GPU timestamp query収集、10分実機E2Eは未達。M2基盤再締結とは分離して完了を判定する
- M3仕様はドラフトである。#180/#191は先行してmainへ入ったが、依存方向CIと空のUIクレート骨格に限る。これらをM3入場完了の根拠にしない
- PR #176相当の将来境界案は分割処分が進み、planar v1 camera決定とParam Pipeline／Element Domain／Constraint Graphの持越し境界はmain到達済み。CAM-G0以降のcamera実装とD1mは未着手であり、残る差分の棚卸しと再締結証跡への対応付けは未完了

## 再締結宣言の退出条件

次の各項目に証跡が揃うまで、M2基盤を「凍結」「完了」「再締結」と記載しない。

### A. 恒久面の閉集合

1. **Shared Effect**: D1l schema/migrationをmainへ到達させ、その後にD3e評価接続を別PRで完了する。inline旧projectの要素数・順序・未知plugin field・pixelを保持し、Definition/Use欠落を型付き拒否する
2. **CompCamera**: [planar v1決定](2026-07-16-m2-comp-camera-decision.md)をmainへ到達させ、CAM-G0既存pixel fixture→D1l後のD1j v5 schema+default migration→D1k runtime契約→D3f接続を別PRで直列化する。v1は特異点のない`PlanarOrthographic`だけを焼き、既存2D pixelを保持する。Spatial/PerspectiveはM5の追加variant決定まで未実装
3. **Param Pipeline / Element Domain / Constraint Graph**: [M2持越し境界](2026-07-16-m2-param-element-constraint-disposition.md)をmainへ到達させる。現行`DocParam`/typed ID/LookAt・Followの解釈を変えず、PP/ED/CG各解凍gate前にUI・Document・plugin ABIへ推測のpipeline/generic domain/graphを焼かない
4. 下記の未merge棚卸しを、対象ごとの小さいdecision/spec PRで採択・延期・棄却する。Draft文書や別ブランチの台帳を暗黙の発注根拠にしない
5. **Project sidecar / session ownership**: D1dの親directory共有`.motolii`衝突とprocess間lock未規定をD1mで修復する。同一directory複数projectの隔離、canonical path alias排他、legacy layoutの非破壊移行を満たすまで保存基盤を閉じない

### 未merge棚卸しの初期集合

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
2. `Document::validate`が欠落参照、循環、型不一致、非有限値、未実装modeを黙って縮退せず型付き拒否する
3. D2のapply/revert、gesture merge、Undo/Redoがランダム操作列を含めて初期状態へ戻る。D1l操作はlifecycle決定のDelete Reject、Unlink、Copy Local、orphan保持、各操作1 Undoを個別に審判する
4. D3のmask/group/effect/transform/LookAt・Follow・Parent評価順を意味論ゴールデンで固定する。D3eは非隣接共有、Group合成後1回、preview/export同一、欠落typed errorを個別に審判する。D3fは既定planar cameraで既存2D pixel不変とpreview/export同一を追加する
5. 単一writerと`Arc<Document>`スナップショットを並行テストで確認し、UI/workerから直接書き換える公開口がない
6. D1fの未知plugin保持・警告とD6のdegraded plugin書き出し拒否を結合し、未来versionが既知扱いへ迂回しないことを確認する
7. ジャーナルの破損・途中書き込み・世代不一致を型付きで拒否し、復旧がDocument正本を上書きしないことを確認する。D1mで同一directoryの複数projectがsidecarを共有せず、別process/path aliasの同時read-write openを即時typed rejectする
8. 意味論ゴールデン更新禁止ゲートと`cargo test --workspace`が全緑である

### 証跡の形式

再締結PRは次の表を埋め、各行にmainのcommit SHA、PR、テスト名またはレビュー記録の節、判定日を記載する。単なる「CI green」や担当者の自己申告では代替できない。

| 面 | 判定 | 必須証跡 |
|---|---|---|
| A 恒久面の閉集合 | レビュー | D1l/D3e到達、#176相当の棚卸し、CompCamera等の採否記録 |
| B migration/validate | 自動 | 対象旧版コーパスと拒否テスト名 |
| B command/ownership | 自動+レビュー | D2/D1l操作列、単一writer、snapshotのテスト名と公開API確認 |
| B doc→render | 自動 | D3/D3e意味論ゴールデン。採択時はcamera審判 |
| B unknown/export | 自動 | D1f→D6結合テスト名 |
| B journal/session | 自動+レビュー | 破損・途中書き込み・世代不一致・復旧、project間隔離、subprocess lock、公開mutation API capabilityのテスト名/確認 |
| C 追補レビュー | レビュー | 固定パスの独立レビュー記録、P0/P1=0、各修復PR |
| 全体 | 自動 | golden policy gateと`cargo test --workspace`のrun URL、commit SHA |

### C. 独立レビュー

1. 再締結PRとは別に、実コードを対象とする追補レビュー記録を`docs/reviews/`へ残す。実装PRの作者と追補レビューの主担当を分ける
2. レビューは少なくとも、schema/migration、command/ownership、doc→render評価、未知plugin/書き出し拒否の4面を確認する
3. P0/P1が0件になるまで再締結しない。修復は1件1PRとし、テスト期待値の書換えで閉じない
4. レビュー記録とCI証跡なしに管理者権限でmerge条件を迂回しない

## M3への影響

- M3仕様は本ゲートが別PRで解除されるまでドラフトのまま維持する
- 許可済みの実装はmain到達済みの#180/#191だけとする。追加のU0/U1等は発注せず、Documentを読むUI、domain intent、preview、入力、timeline、plugin panelへ拡張しない
- 依存ゼロで製品コードを変更しない調査・fixture作成は可能。ただし結果を公開APIや永続形式へ焼かない
- 本ゲートが別の再締結PRで解除された後、M3入場PRでタスクごとのM2/G0依存と自動審判を最新mainへ翻訳し直す

## 発注順

1. 発効宣言PR（本PR）をmainへ到達させる。ここでは退出条件を満たした扱いにしない
2. ~~D1l実装をmainへ到達させる~~（`a23a4ad`、`74af37e`、`02192c2`）。再締結時の独立追補レビューと証跡対応付けは手順6で行う
3. D3eを最新D1l型からIssue化・実装する
4. #176相当のうちmain到達済みのcamera／Param・Element・Constraint決定を証跡表へ対応付け、残差分を恒久面ごとに採択・延期・棄却する。CAM-G0→D1j→D1k→D3fは別PRで直列実装する
5. D1mを独立実装・レビューし、project-scoped sidecarとsession ownershipをmainへ到達させる
6. M2追補実コードレビューを行い、発見事項を1件1PRで修復する
7. 発効宣言とは別のM2基盤再締結PRで本書A〜Cの証跡表を埋め、ステータスを解除へ変更する
8. その後にのみM3段階発注可PRを作る

順序を変える場合は本書の改訂PRを先に出す。並列化の都合だけで恒久面の依存を緩めない。
