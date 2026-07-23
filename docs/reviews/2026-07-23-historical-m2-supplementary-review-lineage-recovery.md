# M2独立追補コードレビューlineageの価値回収（Unit 4H、2026-07-23）

状態: **観察**（cutoff 3 historical blobの処分完了）

対象: `docs/reviews/2026-07-18-m2-foundation-supplementary-code-review.md`のcutoff全3版。

関連: [独立追補レビュー](2026-07-18-m2-foundation-supplementary-code-review.md)、[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)、[再締結ゲート回収](2026-07-23-historical-m2-reclosure-gate-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このlineageの価値は、「別担当が読んだ」という肩書ではなく、greenだった統合候補から意味審判の欠落をP1として発見し、製品経路を通る負例を追加して、同じ固定面を再審査した過程にある。

初回審査はD3fに二つのP1を見つけた。

1. 既定cameraを持つDocumentがCAM-G0のexact bytesを保つ審判がない。
2. 非既定Document cameraについてpreview/exportが同じ最終render経路を通る審判がない。

修復はDocument→graph→GPUを通るcurrent/migrated CAM-G0照合、非既定cameraのpreview/export一致、そして非既定が既定と実際に異なる負例を追加した。oracle、classification、生産API、schema、migration期待値は変えず、再審査でP0/P1=0になった。したがって、**最初からgreenだったことではなく、反例で再開し、修復後に別審査を通したこと**が再利用すべき証拠である。

## 2. 3版で増えた証拠

| 段階 | その版が証明したこと | 証明しなかったこと |
|---|---|---|
| ローカル審査 | 固定された統合候補`e58f0a4`でP1二件を修復し、同じ面の再審査がP0/P1=0 | main到達、remote CI、gate解除、M3入場 |
| コード到達 | PR #217、main SHA、PR/push CIによりコードとremote実行を固定 | 再締結宣言の発効、M3入場 |
| 解除宣言 | PR #218とmain SHAによりA〜C解除宣言の発効を固定 | M3各taskの自動解禁、現在のコード全体の無期限保証 |

ローカル審査、コード到達、解除宣言を後の一行へ潰さない。後段の証拠は前段の対象と判定を固定するが、前段で審査していない面まで遡って保証しない。

## 3. 再利用する独立審査の型

- 対象commitと固定面を先に列挙し、実装担当とread-only検収者を分ける。
- test greenを十分条件にせず、oracle書換え、fixture special-case、生JSON走査、公開raw mutation、暗黙migration、partial mutation、重複planner/helper、camera二重投影、近似skipを負例として探す。
- 欠けた審判は内部helperだけで閉じず、Documentから実際の製品経路を通す。
- 「非既定同士が一致」だけでなく、その非既定入力が既定出力と異なることも固定し、入力が無視された偽陽性を拒否する。
- P1修復後は変更面だけでなく、最初に固定した審査面へ戻って再判定する。
- local green、remote CI、main到達、解除宣言、次milestone入場を別の証拠イベントにする。
- historical reviewのP0/P1=0を、後続変更を含む現在のbranchへ無期限に外挿しない。

現在のAGENTS運用は担当モデルや発注動線が変わっている。2026-07-18の担当名は歴史的provenanceであり、特定モデル名を恒久な品質根拠にはしない。再利用対象は役割分離、固定SHA、負例、再審査、remote証跡の責任構造である。

## 4. P2の現行処分

| P2 | 2026-07-23判定 |
|---|---|
| migrated CAM-G0はmigration後にoracle sceneを載せる | **非証明範囲として保持**。旧scene全体のpixel保持はD1e/D1jとの合成証拠であり、この単体testだけの主張にしない |
| preview/export fixtureはcenter=0、roll=0、height=0.75 | **非証明範囲として保持**。一般式はD3fの独立inverse-UV oracleとの合成証拠であり、一fixtureを全camera空間の証明にしない |
| 再締結ゲートの古い現在地 | **解消済み**。現行文書で「2026-07-15時点の歴史」と明示されている |
| D1k冒頭の`D3f (WAIT)` | **歴史表記として残存**。同書はD1k実装前の凍結記録であり、現在のtask状態として参照しない |
| `DocumentWriter::edit` | **狭い互換面として残存**。undo無し旧来経路であり、通常編集はcommandを使う。motolii-doc外の`&mut Document`禁止審判を維持する |
| stable ID `from_raw` / `peek_next` | **復元・test・予約検証面として残存**。通常採番はwriter／sequence経由というコメントと審判を維持し、存在だけを単一writer破りとしない |

後二項は「いつか消す」という未承認の約束へ変換しない。公開面をさらに広げる、または通常編集・通常採番が迂回し始める場合は、別のAPI縮退判断と負例が必要である。

## 5. 復活させないもの

- workspace greenだけで独立reviewを代用すること。
- 初回P1を消して、最初からP0/P1=0だったと歴史を書き換えること。
- internal camera数式testだけでDocument→GPU、preview/export接続を証明すること。
- preview/exportが同じ誤出力を返す偽陽性を許し、defaultとの差の負例を外すこと。
- oracle、classification、期待値、toleranceを修復側へ寄せて審判を通すこと。
- #217のcode landing、#218の解除宣言、M3入場を同一の完了イベントとすること。
- 旧ローカルSHAや当時のモデル名を、現在のコード・運用への権威として使うこと。
- P2のfixture限界を全camera／全migration保証へ拡張すること。

## 6. 固定歴史出典とcoverage

初版`49ae50a0`を全文で読み、以後の親子差分`49ae50a0..64ce5399`、`64ce5399..e8be2512`を確認した。処分した3 unique blob（21,510 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04h-m2-supplementary-review.tsv`を正本とする。cutoff総数1,797のうち処分済みは314、未処分は1,483である。
