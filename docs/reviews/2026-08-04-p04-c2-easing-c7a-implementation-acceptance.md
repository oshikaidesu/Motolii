# P04-C2 Easing C7A implementation acceptance

- 日付: 2026-08-04
- 実装commit: `bb0624d8157b62def8a2d106ad16033b4b8e3fd3`, `87bf026edb606b641ec41064389028a7c06d9d38`, `56f61e7b0e85f7dc67a2c1d49fc21415373bcde2`
- 状態: **DONE / ACCEPTED（`P04-C2-EASING-C7A`、code/main） / EXTERNAL_GATE_PENDING**
- 正本: [P04-C2 Easing product producer / popup adoption contract](2026-08-04-p04-c2-easing-product-route-contract.md#6-2026-08-04-terminal-adoption-amendment)

## 1. 受入結果と通常製品route

この3 commit は `P04-C2-EASING-C7A` の一つの閉じた product route を接続する。product-owned React
Easing trigger は logical anchor と layout epoch だけを strict surface-local inbound へ送る。Host は
current Document、primary、editor playhead から strict-interior Position interval を再導出し、private
native child `WindowId` の egui popup/session を開く。basic preset 又は validated custom Bezier の
terminal は、一つの narrow Position-only `SetPositionKeyInterp` request を既存 D2 writer へ送り、
value-changing terminal だけが durable command / journal / Undo / publish を一回行う。

```text
React anchor/layout only
  -> Host current interval re-derivation
  -> private child WindowId egui popup on the sole EventLoop/shared GPU
  -> preset or custom terminal
  -> Position-only SetPositionKeyInterp
  -> existing durable D2
```

React に Layer/key identity、time、Interp、Document/revision は渡さない。popup は ProductApp の sole
EventLoop、existing `ProductGpuParts` instance/adapter、shared `GpuCtx` device/queue を再利用する private
owner であり、second App/EventLoop/WebView/device はない。real child `WindowId` は adapter で popup にだけ
dispatch され、late child event は main window route へ漏らさない。

`Linear`、`Smooth`、`Ease In`、`Ease Out` と custom Bezier だけが terminal である。`Hold` と advanced
Bounce/Elastic は disabled で intent 0、interval 不在・stale/cancel/duplicate・same-value terminal は
queue / command / journal / Undo / publish 0 のままである。custom input は既存 `validate_interp` により
admit される。left key の outgoing `Interp` だけが変わり、right key を含む隣接 key は変更しない。

Stage Easing readiness は document generation と layout epoch を照合する。reload は prior layout
acknowledgement を無効化し、bounds epoch 変更は再delivery を要求する。stale acknowledgement は新しい
generation を synced と扱えない。late IPC injection は DOMContentLoaded retry で ready を送る。

## 2. 検証結果

- `cargo test --locked -p motolii-ui --lib`: PASS (218/218)
- `cargo clippy --locked -p motolii-ui --all-targets -- -D warnings`: PASS
- Stage Easing codec guard: PASS (2/2)
- `npm --prefix ui/motolii-web run check:host`: PASS
- `./scripts/check-docs.sh`: PASS
- `git diff --check`: PASS

`./scripts/validate.sh local` は **NOT GREEN**。base `c96bba3a` にも存在する out-of-scope
protected-assets scanner の `sdk_s0_path2d_semantics.rs` `expected_failure` により red であり、C7A
差分の result として green を主張しない。この受入は上記 focused lanes と実diff を根拠にする。

`PRIMARY_ORACLE` は、anchor/layout-only codec、current interval re-derivation、zero-write negative set、
one value-changing terminal = one D2 durable command、same-value no-op、reload/epoch resync、child-window
isolation、single EventLoop/shared GPU を照合する。repository lanes は native visual、z-order、focus、DPI、
accessibility、second monitor の実機審判を代替しない。

## 3. 独立reviewと finding の処分

fresh Opus 5 read-only final review は `ACCEPT / P0=0 / P1=0` を返した。scope は easing readiness
bookkeeping、initialization retry、focused tests に閉じ、public API、Document mutation、dependency、protocol
shape の拡張なしを確認した。

P2 は次の二件だけであり、受入後の修正をここから発注しない。

- generation-guarded acknowledgement と再delivery の間は self-scheduled retry でなく後続
  `sync_easing_layout` に依存する。
- `document_generation` は `u64::MAX` で saturating するため、実用上到達しない枯渇時は reload を区別しない。

review envelope 外の wake-to-sync caller と live reload timing は evidence gap として残るが、P0/P1 ではない。
manual native gates も reviewer が代替しない。

## 4. 状態遷移と非目標

`P04-C2-EASING-C7A` は code/main `DONE / ACCEPTED`。これは P04-C2 parent、U4b 全体、M3 全体の完了を
主張しない。manual native visual parity、z-order、focus/dismiss、DPI、accessibility、second-monitor は
M3-final `EXTERNAL_GATE_PENDING` のままである。

G0-9 stores/counters/`PopupGfx`、`NativeTimelineRenderer` copy/change、generic popup/framework、partial
React/IPC dead route、advanced interpolation semantics、Inspector/Add Position Key、public API、Document/schema/
journal/dependency changes は本受入の非目標である。
