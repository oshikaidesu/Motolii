# CU-106P native Timeline primary selection実装決定

- 日付: 2026-07-30
- 状態: **決定**
- CU-106P: **DONE**
- U2h-1P: **DONE**

## 1. 成立したproduction経路

通常製品Hostのnative Timeline clickを、既存AppKit Host pointer境界へ追加した
`NSEvent` local monitorで一回だけ観測し、
`ProductTimelineProjection::hit_test`から既存`TimelineHit`へ写像した。
`Key` / `Bar`は同じ`LayerId`の`ReplacePrimary`、Timeline内の`None`は
`ClearPrimary`として既存`DocumentEditQueue` / `DocumentEditRuntime::process_next`へ配送する。
winit raw pointer、公開`DomainIntent`、keymap、React Timeline、test-only callerは追加しない。
monitor blockには既存objc2系が採用する`block2`を直接依存として宣言し、独自FFIを作らない。

## 2. selection-only producer

- `ReplacePrimary(target)`は`DocumentWriter::find_envelope(target)`を先に評価する。
  unknown / table-only targetは、same-idまたはgeneration枯渇より先にtyped rejectする。
- live same-idと`ClearPrimary(None)`はactionを一回消費してpublish 0。
- accepted changeだけが`projection_generation.checked_add(1)`を通り、
  current snapshot / revisionと新primaryを一つの`PublishedDocument`で返す。
- Document、journal、Undo/Redo history、revision、Layer tableは変更しない。
- 製品callerは同じpublished primary / generationを採用し、既存Inspector Host islandへ
  current Document / primaryを再配送する。第2selection store / generation / reconcileはない。

## 3. 証拠

```text
cargo test -p motolii-ui --lib
cargo test -p motolii-ui --test cu106p_primary_selection_consumer
cargo test -p motolii-ui --test raw_input_boundary
cargo clippy -p motolii-ui --all-targets -- -D warnings
cargo test --workspace -- --test-threads=1
```

unit testはlive replace、same-id、clear、empty clear、table-only same-id、
`u64::MAX`拒否、queue消費、Document / revision / history不変を固定する。
production reachability guardはAppKit Host click、typed `TimelineHit`、selection action一件、
既存publish adoptionがnon-test `product_runtime.rs`に共存することを固定する。

## 4. 非目標

- essential focus、hover、additive / range / marquee、hidden件数、bounded AX。
- Stage / Browser click selection、三surface selection chrome。
- Undo/Redo配送、shortcut、public input contract。
- Document / serde / journal / plugin契約、React state owner。

## 5. 次

`CU-106P`と内包された`U2h-1P`は`DONE`。次の唯一のPRODUCT-ASSET `DO`は
VS-1のUndo/Redo配送`CU-111`。`CU-106F`とtoken後続は`WAIT`を維持する。
