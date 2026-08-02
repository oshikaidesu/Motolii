# U4b-1 製品Easing接続 実装記録

作成日: 2026-08-02
状態: **自動検証DONE / 通常Mac製品window人間E2E DONE**

## 1. 利用者成果

通常製品windowでPositionの隣接key区間を選ぶと、Stage Transportに既存のEasing triggerが現れる。
triggerからnative popupを開き、preset選択またはcurve dragをreleaseすると、既存D2
`SetPositionKeyInterp`を一度だけcommitして同じDocument snapshotを再投影する。

## 2. 既存契約接続票

| field | resolution |
|---|---|
| `AUTHORITY` | [M3 U4b](../specs/M3-ui-integration.md)、[outgoing Interp契約](2026-08-02-u4b1-outgoing-interp-command-contract-decision.md)、[native Easing popup受入契約](2026-07-22-m3-native-easing-popup-acceptance.md)、[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md) |
| `INTERNAL TARGET` | product-owned `EasingTriggerCandidate`、`project_position_interval`、`SetPositionKeyInterpRequest`、`DocumentEditQueue` |
| `OWNER` | Reactはtrigger表示、Hostは選択・generation・layout epoch・popup lifecycle、native popupはlocal presentation、DocumentはD2 single writer |
| `WRITE ROUTE` | popup release → `SetPositionKeyInterpRequest` → `DocumentEditRuntime` → `DocumentWriter::prepare_set_position_key_interp` → durable command |
| `GAP` | 既存trigger、受入済みpopup model/shader、既存commandの通常製品route接続が無かった |
| `RESOLUTION ROUTE` | `REUSE`: triggerとcommandを再利用。`PORT`: G0-9 model/shaderを製品crateへ移す。既存product wgpu device/queueを共有 |
| `DISPOSITION` | `PASS`。公開API、Document意味、plugin契約、永続形式の新設0 |

## 3. 接続境界

- Stage Transportはplaceholder buttonを廃止し、product-owned `EasingTriggerCandidate`を直接構成する。
- Hostは`LayerId + left/right KeyframeId + projection generation + layout epoch + anchor rect`を照合し、
  stale identity、古いlayout、非有限または空のanchorをpopup生成前に拒否する。
- popupは製品の既存wgpu instance / adapter / device / queueを共有し、drag loop内でGPU resourceを生成しない。
- winit生入力はprivate `easing_popup_input_adapter`だけが閉集合で受け、popup runtimeへtyped inputとして渡す。
  既存のlifecycle専用`product_runtime_adapter`とegui layout adapterの許可範囲は広げない。
- drag中はDocument write 0、成功releaseまたはpreset選択は1 commit、Escape・focus loss・closeは0 commitとする。
- commit後は通常のfull publishでStage / Timeline / Inspectorとtriggerへ同じsnapshotを再投影する。

## 4. 保持した負例と非目標

- `docs/mocks-ui` runtime import、別trigger、別popup model、二重selection、raw Document mutationを作らない。
- catalog labelやopaque IDからinterval意味を推測しない。
- popup adapterはprimary button、Escape / Tab / 矢印、focus / resize / redraw / cursorだけを許可し、
  right button、Enter、modifier、device eventをraw input guardで拒否する。
- visual threshold、golden、Journal version、schemaを変更しない。
- My Presets / Save / Favoriteの永続化はUser settings codecが未確定なので実装しない。UI面は保持し、
  操作は`user-settings-unavailable`として観測する。Documentやlocal ad-hoc fileへ代替保存しない。
- Add Position Keyの製品trigger、Transport/playhead、AX全審判は本粒へ束ねない。

## 5. 自動検証と残る受入

- `cargo test -p motolii-ui easing_popup --lib`
- `cargo test -p motolii-ui stage_chrome_host_runtime --lib`
- `cargo clippy -p motolii-ui --all-targets -- -D warnings`
- `node --test ui/motolii-web/guard-tests/browser-ownership.test.mjs`
- `npm --prefix ui/motolii-web run check:host`
- `npm --prefix ui/motolii-web run build:host`

自動検証は、exact interval受理、stale / unknown / unprojected拒否、interval変更時queue破棄、
popup modelのdrag / commit / cancel、React source ownershipと二重trigger不在を閉じる。
通常Mac製品windowで2つのPosition keyを持つ実projectを開き、trigger表示、popup、Smooth preset、
Undo / Redoを確認した。プロセスを閉じて同じprojectを再openし、`journal.wal` replay後もpopupのSmooth
curveが復元されることを確認した。My Presets / FavoriteのUser settings永続化は本決定の非目標として残す。
