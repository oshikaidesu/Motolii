# Inspector Position key one-shot intent implementation acceptance

- 日付: 2026-08-04
- 実装commit: `98e38925c9b0787e822f233fe2c17227ec55c929`
- 状態: **DONE / ACCEPTED（`CU-0A08ITIB`、code/main） / EXTERNAL_GATE_PENDING**
- 正本: [Inspector Position key one-shot intent contract](2026-08-04-inspector-position-key-one-shot-intent-contract.md)

## 1. 受入結果と到達範囲

commit `98e38925` は、通常製品 Inspector の既存 Position `objectRow` に値表示と別の隣接
`Add Position Key` affordance を接続した。WebView は exact
`{"kind":"add-position-key","sequence":...}` だけを送信し、Rust Host は既存 opacity
gesture inbox と分離した bounded FIFO で受理する。target、time、value、version、automation、
generic key API は wire に追加していない。

`ProductApp` は `Wake` で position FIFO を drain し、その時点の `primary` と
`editor_playhead.current` を一件の private `AddPositionKeyRequest` に捕捉する。既存
`DocumentEditQueue` / `DocumentEditRuntime` は current-primary を再照合して、既存
`DocumentWriter::prepare_add_position_key` の `Prepared` だけを durable commit / Undo / Redo /
JournalEdit v2 / full publish へ送る。`AlreadyPresent`、primary 不在・不一致、unsupported
Position は mutation / revision / history / journal / projection generation / publish 0 のままである。

この差分は既存 UI / Host / queue route を接続するだけで、`motolii-doc`、Document schema、journal、
public API、dependency、新しい state owner を変更していない。normal host build に伴う generated
asset manifest / existing route include の更新も同じ integrated commit に含まれ、source authority は
product-owned Inspector source のままである。

## 2. integrated validation

- `cargo fmt --all --check`: PASS
- `cargo test --locked -p motolii-ui --lib`: PASS (207/207)
- `cargo clippy --locked -p motolii-ui --all-targets -- -D warnings`: PASS
- Inspector Host codec guard: PASS (7/7)
- Browser ownership guard: PASS (15/15)
- `npm --prefix ui/motolii-web run check:host`: PASS
- `./scripts/check-docs.sh`: PASS
- `git diff --check`: PASS

`PRIMARY_ORACLE` は、normal Position row の一回の activation が exact private message、separate
Host FIFO、Wake 時の current primary/playhead、one queue action、既存 prepare/durable/full publish
を通り、同時刻 duplicate は `AlreadyPresent` no-op になることを照合する。

## 3. 独立reviewと finding の処分

fresh Grok read-only review は `ACCEPT / P0=0 / P1=0` を返し、scope、negative routes、generated
host route を PASS とした。次の P2 は finding として保存し、修正 task をこの受入から発生させない。

- tests は IPC mux と `Wake` の結合を executable に cover していない。
- `ProductApp` の Wake test は static `include_str!` assertion であり、二つの FIFO を通す full integration test ではない。

これらは acceptance を P0/P1 failure へ繰り上げず、scope 外の新しい施工権限にもならない。

## 4. 状態遷移と残る gate

`CU-0A08ITIB` は code/main `DONE / ACCEPTED`。先行 A と B が接続されたため、親
`CU-0A08ITI` / `CU-0A08IT` の Add Position Key normal route も `DONE / ACCEPTED` とする。
native/WebView の visual、focus、accessibility、affordance recognizability は M3 final の
`EXTERNAL_GATE_PENDING` のままであり、上の repository lanes や review は代替しない。

P04-C2 Easing popup terminal は別の local `WAIT_TARGET` である。Easing popup、partial React/IPC、
renderer port、generic framework、Document/schema/journal 拡張を本受入から開始しない。
