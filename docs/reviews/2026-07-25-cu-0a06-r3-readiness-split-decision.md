# CU-0A06 / R3 KEYS/LAYERS readiness分割決定（2026-07-25）

状態: **決定 / CU-0A06A DONE / CU-0A06B READY-RECHECK**

対象: M3 Presentation OwnershipのR3。固定commit
`56c318edcddab7cf95d263cc2f7dd2b4e6791134`にあるKEYS/LAYERS tool panelを、
native Timeline本体、ruler、bar、key、playhead、packing stateから分離してproduct ownerへ移す。

## 1. 再確認事実

- 独立したKEYS/LAYERS React sourceは存在せず、`TimelineCandidate.jsx`内のinline subtreeである。
- panelは`selectedKeys.size`と`selectedObjects.size`を読むが、`applyKeyOperation`、
  `applyLayerOperation`、selection、object offsets、playheadはTimeline側が所有する。
- `keyToolsOpen`はTimeline bodyの`has-key-tools`にも使われるためTimeline側へ残す。
- panel専用CSSは`timeline-candidate.css`内の連続した`.candidate-key-*` rule群である。
  幅変数の宣言とrail／viewport offset ruleはTimeline側へ残す。
- 既存PlaywrightはKEYS/LAYERS排他、selection count、operation結果、dock隣接、scroll固定を
  実画面で審判する。stored PNGやKEYS/LAYERSのlegacy counterpartは存在しない。

## 2. 決定

`CU-0A06`を次の二粒へ分ける。

1. `CU-0A06A / R3A mock-side extraction`:
   - 未変更sourceに対し、既存Playwrightへ6状態（KEYS align/stagger/stretch、
     LAYERS align/stagger/shift）のcomputed style、ARIA、section toggle、scope、
     close/reopen、operation dispatchを先に固定する
   - mock内に独立componentとpanel専用CSSを抽出する
   - Timeline側に`keyToolsOpen`、selection、operation本体、幅変数宣言、
     rail／viewport offsetを残す
2. `CU-0A06B / R3B product ownership`:
   - R3Aのcomponent/CSSをbyte同一で`ui/motolii-web`へ移す
   - mockをproduct export consumerへ反転し、mock側copyを0にする
   - existing ownership guardへ固定hash、import closure、Timeline-state識別子拒否を追加する

R3AとR3Bを一つの変更許可へ束ねない。R3Aは
[#347](https://github.com/oshikaidesu/Motolii/pull/347)、merge
`3f3b174337ed6efbbc3c2e69ebf06109bf3119c0`で完了した。独立sourceは
`docs/mocks-ui/src/candidates/KeyToolsCandidate.jsx`
（SHA-256 `bf38656a99957a9f2d1465057820510f525fd394a3965cff9f291415223f87ba`）、
panel専用CSSは`docs/mocks-ui/src/candidates/key-tools-candidate.css`
（SHA-256 `f84eb7f98f05844fa3bfc72b702cee2709f1fc0bb9be614f2b01039a65b5190d`）
として固定した。GrokはP0/P1/P2=0、`VERDICT: ACCEPT`、対象22件・全UI 54件・
reference guard 100件・workspace全体を通過した。CU-0A06Bはこの二つのbytes、
既存Easing product ownership topology、mock consumer反転、ownership guard closureを
再確認してから`DO`へ上げる。

## 3. visual matrixの適用

直接移管契約の17-stateは全surfaceを横断する閉集合であり、一面の移管ごとにlegacy counterpartが
存在しないstateまで新しいstored goldenで発明する要求ではない。各grainは対象surfaceのstateだけを、
既存oracleで審判する。

- Browserは既存live legacy-vs-React pixel oracle
- Easingは既存trigger／popup oracle
- KEYS/LAYERSは既存Timeline Playwrightへ6状態のcomputed style、ARIA、geometryを追加
- InspectorはR4でlegacy parityを先に成立させる

threshold、viewport、font、golden、`visual-parity.spec.js`は移管都合で変更しない。

## 4. STOP

- `TimelineCandidate`全体、ruler、bar、key、playhead、packing codeの移動・複製が必要
- selection、operation本体、Document、Undo、Host projection／intentをpanelへ移す必要がある
- panel CSSの連続順序を保てない、幅変数を二重宣言する、Timeline offset ruleの変更が必要
- 既存geometry testを変更しないと成立しない
- new PNG、stored baseline機構、threshold／golden変更が必要
- public API、serde、Document、journal、plugin contractへ変更が必要

## 5. 助言の処分

Opus 5のread-only相談は、独立source不在、R2型の二粒分割、Timeline側state owner、
既存ownership guard再利用、Timeline-state識別子拒否を有効な助言として採用した。
当初の「CSS分離をownership粒まで遅らせる」案は、byte同一移管を壊すため再照会後に撤回された。
normalized AST frameworkと新しいPNG基盤は責任拡大として採用しない。
