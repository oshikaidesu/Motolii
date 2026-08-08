# U4b-0V Position key value edit implementation acceptance

- 日付: 2026-08-04
- 実装commit: `c404a050`
- 状態: **DONE / ACCEPTED（code/main） / EXTERNAL_GATE_PENDING**
- 正本: [U4b-0V closed contract](2026-08-04-u4b0v-position-key-value-edit-contract.md)

## 1. 受入結果

commit `c404a050`で、既存product-owned React InspectorのPosition行をexact current-playhead Vec2 keyの
X/Y編集へ接続した。Constとoff-keyはread-onlyのままであり、別時刻では既存の明示的`Add Position Key`を
先に使う。Auto Key、暗黙key挿入、whole-curve `SetProperty`、second Inspector ownerは追加していない。

durable routeは専用`SetPositionKeyValue { target, key, old, new }`を使う。apply/replayはkey-local CASを行い、
対象keyのvalueだけを置換してID、time、outgoing interpolation、他key、stable-ID counterを保持する。
previewはcloneだけを変更し、successful terminalだけが既存single writer、JournalEdit v2、Undo/Redo、
save/reopenへ一件で入る。private JS messageはkind/phase/session/sequence/axis/finite valueだけを運ぶ。

Timeline sourceとmarker geometryは変更していない。marker size固定、x=time projection、bar width=duration、
zoom/viewport=projectionという既存NLE設計を維持する。

## 2. validation とreview

- `cargo test -p motolii-doc --lib`: 93 passed
- `cargo test -p motolii-ui --lib`: 227 passed
- `cargo clippy -p motolii-doc -p motolii-ui --all-targets -- -D warnings`: PASS
- `cargo fmt --check`: PASS
- `npm --prefix ui/motolii-web run check:host`: PASS
- normal `build:host`二回: PASS、二回目のgenerated tree hash不変
- `git diff --check`: PASS

fresh Opus medium read-only reviewはblind evidence envelope
`d6e2fa64cc5c3c0d8c00c6c4957d554217b811d80ca0102d0441aee38a03a362`に対し
`ACCEPT / P0=0 / P1=0 / P2=0 / EVIDENCE_GAP=none`を返した。reviewer mutationはない。

## 3. 残るgate

native/WebView visual、focus、keyboard、a11y、recognizabilityは利用者方針どおりM3-final
`EXTERNAL_GATE_PENDING`へ集約する。自動試験とLLM reviewで人間審判を代替せず、M3全体の完成は本受入だけでは
主張しない。
