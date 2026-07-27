# U2h-1P selection入力到達性決定

- 日付: 2026-07-27
- 状態: **決定**
- U2h-1PR: **DONE**
- U2h-1P: **WAIT**（[CU-106 selection consumer分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md)のCU-106Pへ統合）

## 1. 発見した欠落

`U2h-1P`はprivate `ReplacePrimary` / `ClearPrimary` actionとCU-104 P5だけを
producer-onlyで先に実装する粒として登録されていた。しかし最新main系では、selection actionを
構築するproduction callerが存在しない。

- `DocumentEditQueue`のproduction callerはU2b-1 smokeのApply / Undo / Redoだけである。
- publicなheadless `TimelineHit`は実装済みだが、`MotoliiApp`を含むproduction callerは0件である。
- 未使用の`pub(crate)` methodと未構築enum variantは、どちらも
  `cargo clippy -p motolii-ui --all-targets -- -D warnings`で拒否される。
- lint抑制、dummy caller、`#[cfg(test)]`隠蔽、公開化、既存smoke期待列への混入は、
  現行発注規約またはU2h-1Pの非目標に反する。

よって、U2h-1Pを単独producer粒のまま施工してはならない。これはCU-104のowner、
field閉集合、generation規則、P5の意味を変更する判断ではなく、最初のproduction到達性を
同じ差分で成立させるための順序修復である。

## 2. 到達性の所有

1. producer-onlyの独立U2h-1P実装を廃止する。U2h-1PはP5の受入IDとして残す。
2. CU-105のdense Timeline projection / hit-test責任が、完了済みU3a-1Iと比べて何を追加するかを
   `READY-RECHECK`で先に再確認する。
3. 再確認後、CU-106をprimary selectionとessential focusへ分け、最初のprimary-selection sliceへ
   U2h-1Pを統合する。そのsliceはproducerと実在する最小production callerを同じ差分で成立させ、
   lint抑制やdummy到達性を使わない。
4. 最小callerのsurface、入力event、公開境界はCU-105再確認前に発明しない。既存の
   headless hit-testを使えない、または新しい公開`DomainIntent` / keymap / transport契約が必要ならSTOPする。

CU-106全体、essential focus、三surface接続、hidden selection、additive/range/marquee/AXを
最初のprimary-selection sliceへ束ねない。

## 3. selection-only判定優先順

`ReplacePrimary(target)`は次の順で判定する。

1. `DocumentWriter::find_envelope(target)`だけで存在を検証する。
2. unknownまたはtable-onlyなら、current primaryとの同値にかかわらず同じtyped errorで拒否する。
3. live targetがcurrent primaryと同じならno-opとする。
4. accepted changeだけが既存のgeneration枯渇preflightへ進む。

`ClearPrimary`はcurrent primaryが`None`ならno-op、`Some`ならaccepted changeとして枯渇preflightへ進む。
拒否とno-opはいずれもactionを1回消費し、publish 0、generation / revision / Document /
history / primary不変とする。存在拒否を先にするのは、dangling primaryを同値no-opとして
黙認せず、CU-104 SN5のnonexistent target拒否を常に維持するためである。

## 4. 非目標

- Rust / JS / fixture / guard test / golden / threshold変更。
- CU-105 / CU-106の実装、または最小callerのsurface・event・公開signature決定。
- `DomainIntent`、`CommandRegistry`、`InputRouter`、keymap、Host transportの変更。
- CU-109 / CU-110 / CU-111、Place receipt、consumer三面接続、essential focusの実装。
- 公開API、Document、serde、journal、Undo/history、ProjectSession、plugin契約の変更。
- lint抑制、dummy caller、test-only production入力面の許可。

## 5. STOP

1. CU-105再確認前に最小callerのsurfaceまたは入力eventを決める必要がある。
2. U2h-1Pを単独producer粒へ戻すためlint抑制、dummy参照、`#[cfg(test)]`隠蔽が必要に見える。
3. public intent / keymap / transport / Document / plugin契約を変えないと到達性が成立しない。
4. U2h-1PをCU-106全体、essential focus、三surface接続と一粒へ束ねる必要が見える。

## 6. 引き渡し

CU-105RとCU-106Sは完了し、U2h-1PはCU-106Pへ統合された。発注依存証跡を閉じた後の
docs-only `U3a-2S`とdocs-only `U3a-2R`とdocs-only `U3a-2Z`とdocs-only `U3a-2A`とdocs-only `U3a-2P`とdocs-only `U3a-2Q`とdocs-only `U3a-2Q-P`とdocs-only `U3a-2Q-P2`とdocs-only `U3a-2Q-P3`とdocs-only `U3a-2Q-P4`は`DONE`で、現行の次PRODUCT-ASSET `DO`は0件（`U3a-2Q-V` `WAIT`）である。実consumer surfaceとproduction入力が成立するまで
U2h-1P / CU-106P/Fのclosed orderを作らない。
