# M2基盤再締結ゲートlineageの価値回収（Unit 4G、2026-07-23）

状態: **観察**（cutoff 14 historical blobの処分完了）

対象: `docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md`のcutoff全14版。

関連: [M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)、[独立追補レビュー](2026-07-18-m2-foundation-supplementary-code-review.md)、[M2入口ゲート回収](2026-07-23-historical-m2-entry-gate-lineage-recovery.md)、[historical foundation回収](2026-07-23-historical-foundation-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このlineageは、レビューなしのM2コア締結を撤回した後、個別P1修復だけで再び「完了」と呼ばないために発効した停止ゲートである。価値は機能一覧ではなく、再締結を三つの異なる証明へ分けた点にある。

1. **A 恒久面の閉集合**: Shared Effect、planar camera、param境界持越し、project sidecar/sessionについて、採択・延期・棄却と実装順を閉じる。
2. **B 意味の審判**: migration、validate、command/Undo、Document→render、unknown/export、journal/sessionを、それぞれの負例とsemantic oracleで証明する。
3. **C 独立レビュー**: 実装担当と別のread-only reviewを残し、P0/P1=0になるまで解除しない。

さらに、コード到達PR #217、解除宣言PR #218、M3入場を同一イベントにしなかった。2026-07-18の解除は現在も有効だが、M3の各taskを無条件解禁した証明ではない。現在のM3直列順と個別停止線はM3仕様とimplementation ledgerを正とする。

## 2. 14版で起きた意味変更

| 段階 | 版の変化 | 現在の処分 |
|---|---|---|
| 発効宣言 | M3製品実装を停止し、`cargo test --workspace`を必要条件に限定。A/B/Cと別解除PRを要求 | **採択済み**。green CIだけで基盤締結しない |
| D1m追加 | 親directory共有sidecarとprocess間lockを、project-scoped sidecar/session所有の退出条件へ追加 | **実装・証跡済み**。D1mの全版回収はUnit 4A |
| Param/ED/CG処分 | PR #176相当の一括境界をM2へ焼かず、解凍gateまで持越し | **採択済み**。UI起点の推測schema/ABIを禁止 |
| planar camera採択 | 未決CompCameraをCAM-G0→D1j→D1k→D3fの直列へ具体化し、Spatial/PerspectiveをM5へ延期 | **実装・証跡済み**。single camera思想や3D全体完了とは別 |
| D1n分岐 | external revisionをD1m後の退出条件へ追加したbranchが存在したが、最終解除系列へは入らなかった | **価値を独立follow-upとして再採択済み**。過去の解除を遡及無効にしない |
| 現在地更新 | D1l、D5骨格、camera/param決定のmain到達を反映しつつ、証跡対応と製品統合を未完了のまま保持 | **進捗記録の規律を保持**。landed skeletonと完成を分ける |
| preview spike注記 | PV-1のMetal texture lifecycleを隔離spikeとして記録し、U1a/U1bやA〜Cの証拠へ数えなかった | **負例として保持**。spike成功を製品完了へ昇格しない |
| Overrun訂正 | 「未実装modeをvalidateで拒否」から、Black/Loopは保存可能、全評価入口でtyped unsupportedへ訂正 | **現行正本**。保存可能と実行可能を同一視しない |
| 解除草稿 | #217のコード・CIと追補レビューをA〜Cの命名審判へ対応付け、D5を閉集合外に明示 | **成立理由を保持**。閉集合外を暗黙完了にしない |
| 解除発効 | #218で解除宣言をmainへ到達。M3着手は別入場PRのまま | **歴史的解除として保持** |
| U0a入場 | egui骨格と依存方向CIだけを別に入場完了とし、U0b以降は各依存へ戻した | **歴史的入場点**。現在のM3進捗正本にはしない |

## 3. 現行状態との照合

2026-07-23の現行docsとコードは、歴史的解除と後続残件を次のように分離している。

| 主張 | 現行判定 |
|---|---|
| M2再締結A〜C | **解除済み**。D1l/D3e、D1m、CAM-G0/D1j/D1k/D3f、Param/ED/CG持越し、追補P0/P1=0の証跡を保持 |
| D1n | **決定済み・未実装**。D1mと過去の再締結解除を遡及停止せず、external change検出/cloud-safeを公約しない |
| D5 | **骨格landed・統合/E2E pending**。再締結の閉集合外であり、解除済みから完了を推論しない |
| M3 | U0aだけでなく複数U枝番が完了済み。次の直列taskとG0-9/G0-3停止線はM3仕様／implementation ledgerが正本 |

同じcommitの現行コードで、D1e migration 13件、D1h validate 14件、D1l lifecycle 19件、D1m 12件、D2 command 35件、D8 ownership 4件、D3e 8件、D3f 11件、preview/export 3件、unknown/export 6件、CAM-G0 1件、golden policy 24件を再実行し、すべて緑を確認した。Unit 4F公開前の同一code commitでは`cargo test --workspace`も全緑である。

この再実行は2026-07-18の独立レビューを現在の別レビューで置換しない。過去の解除根拠は固定PR/SHA/CIと追補レビューであり、今回の実行は現行コードが代表的な審判を失っていないことだけを確認する。

## 4. 再利用する再締結設計

- 「修復がmergeされた」と「基盤を再締結した」を別イベントにする。
- 恒久面の閉集合、機械審判、独立reviewを相互代用しない。
- 未merge branchの文書や台帳を既決として使わず、恒久面ごとの小さいdecisionへ分ける。
- 証拠表はtest名だけでなくmain SHA、PR、CI run、判定日を共通keyにする。
- 現在地の未完了文は歴史節と明示し、後の証拠表と同時に現行主張として読ませない。
- gate解除と次milestone入場を分け、後続taskは最新の依存表へ再翻訳する。
- 後から回収した価値は独立follow-upにし、過去gateへ後付けして達成履歴を改変しない。

## 5. 復活させないもの

- P1修復やworkspace greenだけで「M2基盤完了」とすること。
- #217のコード到達、#218の解除宣言、M3入場を一つの完了印へ潰すこと。
- PR #176相当の未merge集合を、一括で現行schema／runtime／UI契約として復活させること。
- Black/Loopを保存不能に戻すこと、または保存可能だから実行可能と読むこと。
- PV-1等の隔離spike、空crate、依存方向CIを製品UI完成へ数えること。
- planar cameraの採択をSpatial/Perspective、2.5D、3D全体の実装済み証拠にすること。
- D5を再締結解除に含めて完了扱いすること。
- D1n branch案を当時の解除必須条件だったと改稿し、#218を無効扱いすること。
- U0a入場時点の進捗を、現在のM3 task順序として使うこと。

## 6. 固定歴史出典とcoverage

初版`52f71b67`を全文で読み、以後の親子差分、D1n等の分岐snapshot、cutoff最終系列`c3ae81e9`を確認した。処分した14 unique blob（148,362 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04g-m2-reclosure-gate.tsv`を正本とする。cutoff総数1,797のうち処分済みは311、未処分は1,486である。
