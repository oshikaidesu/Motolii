# CU-0A05A隔離worktreeの停止と再入場

日付: 2026-07-25

状態: **旧差分停止線 / 再入場decision成立 / 実装未着手**

対象: `CU-0A05A / R2A Easing trigger mock-side extraction`

## 1. 結論

`CU-0A05A`自体は、2026-07-25のControlled Microkernel／非信頼plugin／lane-local並列化決定後も
PRODUCT-ASSET laneの現行粒である。これは固定mock内のReact製品資産抽出であり、公開plugin runtime、
Host capability module、sandbox、process隔離の実装ではない。新しいplugin思想を理由にtaskを棄却せず、
逆に本taskをruntime隔離の成立証拠にも使わない。

一方、`/Users/member_ottoto/rust_ae/Motolii-cu-0a05a-v2`の未検収差分は現行branchへ直接採用しない。
旧差分は実装候補を比較する**証拠カプセル**であり、完了証拠ではない。再開する場合は最新mainから
fresh worktreeとfresh orderを作り、許可されたmock-side差分だけを縮小再適用して、全試験と独立検収を
最初から取り直す。

処分は次の通り。

| 対象 | 処分 |
|---|---|
| `CU-0A05A`の意味とPRODUCT-ASSET lane | **維持** |
| 隔離worktreeの8 path | **縮小再適用候補**。直接merge／commit／自動cherry-pick禁止 |
| 過去のguard、build、Playwright、visual結果 | **完了証拠として失効** |
| `CU-0A05B`、Motolii Studio Preview、product package | **未着手のまま** |
| plugin trust／runtime isolationとの関係 | **非証明** |

## 2. 確認したコード事実

- 隔離worktreeのHEADは`a69b1bf45fd210eed5e7115f51cca49fcec73ff5`。
- 本停止線作成時の統合branch先端は`8ddfcd79`で、共通祖先は`38254423`。旧側1 commit、
  統合側2 commitに分岐している。
- 隔離worktreeの未commit差分は次の8 pathだけである。
  - `docs/mocks-ui/guard-tests/source-asset-inventory.test.mjs`
  - `docs/mocks-ui/source-asset-inventory.json`
  - `docs/mocks-ui/src/candidates/TimelineCandidate.jsx`
  - `docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx`
  - `docs/mocks-ui/src/main.jsx`
  - `docs/mocks-ui/tests/browser-candidate.spec.js`
  - `docs/mocks-ui/src/candidates/EasingTriggerCandidate.jsx`
  - `docs/mocks-ui/src/candidates/easing-trigger-candidate.css`
- product package、`CU-0A05B`、Motolii Studio Previewのfileは変更していない。
- `TimelineCandidate.jsx`の`activeInterval`計算へ`playheadLeft`依存を加えた差分は存在するが、
  その後の全試験と検収は未実施である。

セッション報告では、修正前の途中証拠としてguard 99/99、Browser product ownership guard 3/3、
Vite build成功、Storybook build成功、Playwright 13/15が記録された。ただし、その後にcodeを変更しており、
固定hash更新、Playwright再実行、visual parity再実行、Grok再検収は行われていない。したがって数値を
fresh orderの合格証拠へ継承しない。

## 3. Opus 5 read-onlyレビュー

Opus 5へ発注書作成、編集、Spark起動を許さず、現行authorityと上記packetのread-onlyレビューだけを
依頼した。初回のrepo直接レビューは10分間応答がなく停止し、別modelへfallbackしなかった。検証済み事実へ
縮小した再レビューは`ADVICE: FIX`だった。

助言は次の通り。

- `CU-0A05A`はplugin trust軸ではなくproduct asset軸なので、architecture上は失効していない。
- 旧差分を旧baseのまま使わず、fresh baseへ8 pathを縮小再適用する。
- code変更後なので過去の試験証拠をすべて取り直す。
- plugin isolationの進捗へ数えない。
- fresh baseの無変更baselineと再適用後を比較し、既存失敗と回帰を混同しない。

これは仕様決定ではなく助言である。主担当Codexが現行authority、branch関係、実diffへ再照合し、
本書の停止線だけを採用した。

## 4. 再入場手順

1. 最新mainから隔離worktreeを新設し、`CU-0A05A`の現行authorityと固定source SHAを再確認する。
2. 変更前に同じbrowser／visual commandを実行し、fresh-base baselineを保存する。
3. 旧worktreeを完成差分としてcherry-pickせず、8 pathを一つずつ比較して必要な部分だけ再適用する。
4. product package、`CU-0A05B`、Motolii Studio Previewのdiffが0であることを機械確認する。
5. source inventoryの既存`sha256`は固定commit blobのhashとして維持し、変更済み既存closureには
   `currentSha256`を併記する。新規trigger closureは`currentSha256`だけを持つ。hashを試験合格の
   ためだけに更新しない。
6. source-asset guard、Browser ownership guard、Vite、Storybook、Playwright、visual parityを全再実行する。
7. Playwrightはfresh-base baselineより悪化していないことを確認し、失敗を「既存」と推測しない。
8. codeとhashが確定した同一snapshotをGrokへread-only再検収し、`VERDICT: ACCEPT`かつP0/P1=0になるまで
   採用しない。

## 5. STOP

次のどれか一つで当該再入場だけを停止する。無関係な並列laneへ停止を伝播させない。

- 8 path外の変更が必要になった。
- product package、`CU-0A05B`、Motolii Studio Previewへ変更が広がった。
- visible summary chrome、curve意味、Undo、Document、公開API、plugin契約の新判断が必要になった。
- visual parityが同形抽出として一致しない。
- 再適用後のPlaywrightがfresh-base baselineより悪化した。
- 固定commit hash、working hash、inventory roleを一つの値へ潰す必要が生じた。
- `CU-0A05A`をplugin sandbox、runtime isolation、Host module完成の証拠に使おうとした。

## 6. 非目標

- 隔離worktreeの修理、commit、push、main採用
- `CU-0A05A`の完了宣言
- `CU-0A05B` product ownership
- Motolii Studio Previewの実装
- plugin runtime、hot reload、worker respawn、sandboxの設計または実装

## 7. 再入場decision

Opus 5のread-only助言を、現行inventory、固定HTML、`TimelineCandidate`、
`LegacyHostBoundaryScreen`、旧8 path差分へ再照合し、Spark再発注前の三点を次のように閉じる。
これはmock fixture内の所有とprovenanceだけを決め、製品公開API、Document、plugin契約、
永続形式を新設しない。

### 7.1 固定oracleと受入closureのhash

- `fixedSourceCommit`と`triggerAuthority.path`／`sha256`／`selector`は、R2Aが再現する
  **不変の入力oracle**である。R2A実装後も更新しない
- 抽出後のReact sourceは、easing entryへ`triggerSource`を追加し、
  `componentPath`、`componentExport`、`runtimeClosure`、`localImports`、`externalPackages`、
  `provenanceRole: mock-side-current-closure`をexact keyとして持つ。`runtimeClosure`には
  component／CSSのpath、role、`currentSha256`を列挙する。これは受入snapshotの
  **mock-side provenance**であり、oracle同値、製品owner、永続形式を証明しない
- `TimelineCandidate`や`LegacyHostBoundaryScreen`等、既存surface closureにすでに属する
  変更fileは、既存`sha256`を固定commit blobのhashとして維持し、同じentryへ
  `currentSha256`を追加する。guardは`sha256`を固定commit、`currentSha256`をworktreeへ
  個別に照合する。`post05AHashes`のようなphase固有の第二hash表を追加せず、
  固定oracle hashと現行closure hashを一つの値へ潰さない
- hash更新は実装とparityが確定した同一snapshotで最後に行い、試験を通すための先行更新をしない

### 7.2 状態遷移

本decisionがmainへ到達したら`CU-0A05A`は`DECIDE`から`DO`へ移る。R2Aはfresh worktree、
全再試験、Grok `VERDICT: ACCEPT`かつP0/P1=0、Codex再照合まで`DONE`にしない。
R2Aが`DONE`になった時点で`CU-0A05B`を`READY-RECHECK`へ移し、product ownershipの
fresh orderを作る。R2Aの`DONE`は独立React triggerが**mock owner**になったことだけを表し、
product ownership、Host projection、Motolii Studio Previewを表さない。

### 7.3 `activeInterval`とtriggerの単一owner

- `TimelineCandidate`だけが、React Timelineの選択、key、playheadから`activeInterval`を導出する。
  `playheadLeft`を依存へ含め、DOM selectorでtriggerを直接変更しない
- `LegacyHostBoundaryScreen`はfixture-only adapterとして、その値をReact context/callbackで
  `EasingTriggerCandidate`と既存`EasingGraphCandidate`へ一方向に配線する。別の区間導出、
  curve意味、Undo正本を持たない
- `EasingTriggerCandidate`はbutton DOM/class/SVG/ARIAとcontrolled presentationだけを所有し、
  visible summary、curve／Interp state、Undoを持たない
- 固定HTMLは変更しない。`LegacyHostBoundaryScreen`が固定`legacyScript`を実行する前に、
  `#interval-easing`を変更する`syncIntervalEasing`経路、直接`onclick`、pressed変更を
  fixture限定のexact-match・must-match-once containmentで無効化する。anchorが0件または
  複数件なら初期化を型付き失敗させる
- open時のpressedはcontrolled trigger adapterが設定し、`#close-easing`の通常clickと
  panel open中のEscapeをfixture adapterが明示的に受けてfalseへ戻す。legacy DOM mutationの
  観測からpressedを逆算しない
- `MutationObserver`、capture listener、`stopImmediatePropagation`、再描画による
  legacy writeの奪回をsingle ownerの代用にしない

必須負例は、playhead移動、key click、trigger click、close click、Escapeの各列でReact triggerへ
legacy由来の中間mutationが発生しないこと、DOM `onclick` propertyが`null`であること、
一clickでpanel openが一度だけ起きること、close／EscapeでMutationObserverなしに
`aria-pressed=false`へ戻ること、containment anchor driftを拒否すること、
candidate／adapter sourceに`MutationObserver`、capture listener、`stopImmediatePropagation`、
trigger所有の再mount tickが無いこと、固定HTML hash不変、product差分0、popup／curve／Undo oracle
不変であることとする。既存`EasingGraphCandidate`のlegacy workspace再mountは本粒のtrigger
single-owner手段にも完了証拠にもせず、同file差分0を要求する。これらを8 path内で閉じられなければ
`ORDER: STOP`とする。
