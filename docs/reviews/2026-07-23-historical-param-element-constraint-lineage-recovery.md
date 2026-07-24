# Param Pipeline／Element Domain／Constraint Graph lineageの価値回収（Unit 4N、2026-07-23）

状態: **決定維持**（cutoff 2 historical blobの処分完了、三能力は未実装、各解凍gate前は停止）

対象: [M2終了前判定](2026-07-14-m2-exit-param-pipeline-disposition.md)と[三境界の持越し決定](2026-07-16-m2-param-element-constraint-disposition.md)のcutoff全2版。

関連: [操作単純化モデル](../interaction-simplicity-model.md)、[M2仕様](../specs/M2-document-model.md)、[M3仕様](../specs/M3-ui-integration.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

二文書は競合案ではなく、第一文書がParam PipelineをM2終了条件から外しつつPP-Gateを置き、第二文書が同じ非干渉境界をgeneric Element DomainとConstraint Graphへ拡張した二段の決定である。両blobは現行treeとbyte一致し、失われた追補はない。

現行コードも決定どおりである。`DocParam`は`Const / Keyframes / Data / Vec2Axes / LookAt / Follow`の6 variantで、generic `ElementId`、constraint node collection、parameter `Modifier[]`は存在しない。UI keymapの`Modifier`、layout constraint、one-shot Generatorは別の責任であり、三能力の実装証拠に数えない。

したがって回収する価値は「将来機能候補」そのものより、既存意味をUIやplugin都合から守る三つの発火条件である。

| Gate | 発火前に禁止する越境 | 解凍時に最低限決めること |
|---|---|---|
| PP-Gate | 常設post-key offset、Data＋手補正の同時適用、Generator/Modifier parameter plugin、評価列並べ替え、共通Add/Multiply/Clamp/Remap | 正準形、型表、stage順、逆編集、循環、未知plugin、migration、cache、preview/export oracle |
| ED-Gate | layer/path point/keyframe/mask/effect等を一つの永続`ElementId`へ統合 | 3つ以上の必要操作、identity寿命・所有・nest・duplicate/remap・delete・selection・Undo/journal・未知kind |
| CG-Gate | generic constraint node、user-visible順序、constraint plugin ABI | node/edge型、source/modifier/transformとの順序、cycle/singularity、multi-target、未知plugin、migration、semantic oracle |

## 2. 採択した境界

### 2.1 現行single sourceを完成状態として扱う

M2は将来の全parameter表現を焼くフェーズではない。既存6 variant、typed LookAt／Follow、D2 command／Undo、未知plugin保持とexport拒否、決定的評価順を閉じればよい。Relative Moveは選択keyへ同じ差分を書くone-shot macroであり、永続offsetの代理ではない。

M3の通常property UIは現在の一つのsourceを編集・検査する範囲で進められる。存在しないpipelineを表示したり、second sourceや順序をUI stateから保存したりしてはならない。

### 2.2 三境界を一つの万能抽象へ畳まない

Paramの評価列、cross-kind identity、constraint graphは相互に関係し得るが同じ意味ではない。PPを理由にElement identityを導入せず、EDを理由にconstraint ABIを公開せず、CGを理由にLookAt／Followの現行順序を変更しない。それぞれ独立したdecision/spec PRと反対側レビューを通す。

将来変更は追加variant／field＋version gate、またはD1e明示migrationで行う。既存fieldやvariantの意味をその場で読み替えない。

### 2.3 UI・作者境界との区別

- keymapの`Modifier`は入力修飾キーで、parameter stageではない。
- one-shot Generatorはsnapshotからtyped D2 command batchを作り、確定後に通常Documentだけを残す。live parameter Generatorではない。
- transient multi-selectionはdomain intentの合成で、generic Element DomainをDocumentへ保存する証拠ではない。
- typed LookAt／Followは現行D3順序を持つ参照で、generic constraint graphの先行実装ではない。

## 3. 現行正本との照合

| 面 | 現在地 | 処分 |
|---|---|---|
| `DocParam` | 6 variantのclosed source | 決定どおり。pipeline未実装を欠陥扱いしない |
| PP-Gate | interaction simplicityとM3実装ガードへ反映済み | 発火条件までWAIT |
| ED／CG gate | M2持越し決定と停止条件に存在 | generic公開型／永続形式なしを維持 |
| M3 U4c | current sourceの検査とround-tripを要求 | fictional pipeline禁止を維持 |
| M4 cache | PP通過後だけModifier変異を追加 | 先行key追加をしない |
| task ID | M2表とglobal backlogが双方`GAP-15`を使用 | 本単位でM2側を`M2-GAP-15`へ明確化 |

最後のID衝突は意味の衝突ではないが、backlogの基本Shape語彙とM2の三境界を検索・発注時に取り違える。基本Shape側はglobal backlog IDとして維持し、M2ローカルの持越し行だけを`M2-GAP-15`へ改称する。契約や優先度は変えない。

## 4. 再入場条件

1. 発火した具体ユースケースを一つ示し、D2 key差分、専用field、preset、typed referenceなど小さい代替で不足する証拠を置く。
2. 対象Gateだけを独立仕様にし、恒久形式、公開API、plugin ABI、評価順、Undo/journal、unknown保持、cacheの影響を閉じる。
3. 旧projectの画が不変であるoracleと、新能力のpreview/export一致、拒否型、循環／型不一致等の負例を作る。
4. UIは意味決定後に投影し、UI mock／component／stateをDocument設計の根拠にしない。
5. 一つのGateから他二つが必要に見えたら一括実装せずSTOPし、別の発火証拠とdecisionへ戻す。

## 5. 復活させないもの

- `DocParam`へ推測の`Modified`／`Generator` variantを追加すること。
- 常設Relative Offsetをone-shot Relative Moveの実装詳細として忍ばせること。
- LookAt／Followを文字列expressionまたはgeneric nodeへ読み替えること。
- transient selectionやUI共通componentをgeneric `ElementId`の正当化に使うこと。
- keymap modifier、layout constraint、one-shot Generatorを三Gateの実装到達として数えること。
- 「将来柔軟」のためにuniversal element／constraint APIを先に公開すること。
- PP／ED／CGを一つのnode graph発注へ束ねること。

## 6. 固定歴史出典とcoverage

両blobを全文で読み、相互差分ではなく二段の決定として現行docs／codeへ照合した。処分した2 unique blob（13,352 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04n-param-element-constraint.tsv`を正本とする。cutoff総数1,797のうち処分済みは333、未処分は1,464である。
