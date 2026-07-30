# CU-0B03 native Browser Host codec/inbox実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-0B03H`

## 1. 完了した一本

product-owned Browser offline bundleをnative `wry` WebViewへ載せ、Web側と同じexact
codecをRust側でdecodeした後、instance epoch / strict sequence gateを通して容量16の
private inboxへenqueueする実callerを成立させた。

- session-backed `motolii_ui_shell`の既存eframe event loopをHost ownerとして再利用し、
  committed `generated-host` closureだけを同じtop-level native windowのopaque childへ
  埋め込み、`motolii-browser://product/` custom protocolから配る。localhost /
  network / development fixtureを使わない。zero-argv bootstrap fixtureではWebViewを
  作らず、実project session経路だけをcallerとする。
- 初期化scriptがprivate `window.__MOTOLII_BUILTIN_HOST__`へsnapshotと
  `postMessage`だけを注入する。
- IPC callback内の責任はlock、exact decode、session gate、bounded enqueueだけ。
  Document、D2、selection、Undo、terminal、commitを呼ばない。
- Rectangle sourceの`scope_ref`はHost sessionが発行するopaque IDであり、
  label / thumbnail / DOM / catalog表示値から導かない。Reactは受け取った
  `(scope_ref, item_id)`をそのまま返す。
- transport型とinboxは`motolii-ui` private moduleに閉じ、plugin / Document /
  domainの公開raw APIへ出さない。既存session-backed shell入口以外の公開起動面も
  追加しない。
- native shellは左420ptをBrowser childへ予約し、残る同一top-level Surfaceに既存の
  native Stage / Timelineを保持する。新しいwinit event loopやraw input adapterを
  追加しない。

## 2. 負例

wrong version / direction / role / kind、unknown field、stale epoch、duplicate / gap
sequence、非canonical u64、空または128 UTF-8 bytes超のID、1024 bytes超messageを
fail closedで拒否する。満杯時はsequenceを消費しない。

## 3. 非目標

Stage / Timelineの新しい意味・renderer・token実装、Place preview / terminal /
admission / D2 / Undo、Inspector、reload / crash再投影、focus / geometry epoch、
community panel、generic invoke、公開plugin UI契約、token後続は含めない。
本入口だけを`CU-0B04N/R`、`CU-0B05`、Motolii Studio Preview完成とは扱わない。

## 4. 証拠

```text
cargo clippy -p motolii-ui --lib --bins -- -D warnings
passed

cargo test -p motolii-ui browser_host --lib
3 passed

cargo test -p motolii-ui --test raw_input_boundary
4 passed

cargo test -p motolii-ui --test cu109_session_backed_edit_entry \
  apply_roundtrip_through_session_backed_shell_entry
session-backed native shell / offline child WebView起動、1 passed
```

次は横展開せず、Host inboxから既決Place責任連鎖へ渡すために必要な
`CU-0B04N/R`と`CU-0B05`の最小製品経路を再締結する。token・provenance・診断入口を
先行させない。
