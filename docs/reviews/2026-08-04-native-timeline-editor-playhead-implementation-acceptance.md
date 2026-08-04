# P02-C3 native Timeline editor playhead implementation acceptance

- 日付: 2026-08-04
- 実装commit: `75ccd5e76a56f4614eb2c1411ec1993bf1cac2aa`
- 状態: **DONE（ruler producer/carrier sub-boundary） / HUMAN DEFERRED**
- 正本: [P02-C3 native Timeline editor playhead producer contract](2026-08-04-native-timeline-editor-playhead-contract.md)

## 1. 受入結果と到達範囲

commit `75ccd5e7` は native Timeline ruler の producer/carrier sub-boundary を main まで接続した。fresh `ProductApp` の private Project-session editor playhead は `RationalTime::ZERO` から始まり、native ruler の press / move / release が同じ値を選択・保持する。native Timeline の線と Stage の既存 `RenderRequest` evaluation time は同じ current time を読む。

Esc、focus loss、pointer capture loss、layout 変更、不正 ruler/layout/mapping は press 時値へ復元して scrub を消す。publish は in-memory current value を保つ。Document、journal、history、queue、公開 API、codec、snap、Transport、Inspector、Easing は変更していない。

これは `P02-C3` 親全体の完了ではない。focus、visible range、playback/audio、normal Inspector row/wiring、P04-C2 Easing などは未完である。active-interval ID を選定・発明せず、Easing / Inspector をこの接続済み carrier として扱わない。

## 2. 実diffと契約照合

変更は `crates/motolii-ui/src/product_runtime.rs` と `crates/motolii-ui/src/native_timeline_renderer.rs` のみである。既存 native ruler geometry、`RationalTime`、`ProductApp` input lifecycle、native Timeline render、Stage `RenderRequest` / existing RenderGeneration admission を再利用した。

- ruler press は private playhead scrub を開始し、move は同じ layout epoch 内で current time を更新し、release は最終値を維持する。
- cancel / invalid / stale layout は press 時値に戻す。publish は scrub だけを retire し、current time を変えない。
- native Timeline line と Stage evaluation は同一 private current time を受ける。Document `projection_generation`、D2 writer、journal/history/queue write はこの操作で 0 のままである。
- 専用playhead revisionはconsumerがなく、stale renderは既存RenderGeneration admissionが所有するため新設しない。実装時に判明したこの薄い残余へ合わせ、同じ契約文書から不要なrevision要求を除いた。

## 3. oracle と既知red

- `cargo fmt --check`: PASS
- `cargo test -p motolii-ui --lib product_runtime`: PASS、31/31
- `cargo test -p motolii-ui --lib native_timeline_renderer`: PASS、3/3
- `cargo clippy -p motolii-ui --all-targets -- -D warnings`: PASS
- `git diff --check`: PASS
- `./scripts/validate.sh local`: docs と Rust check は PASS。その後 workspace test は protected-assets `expected_failure` scanner で停止した。focused scanner も base `a4c13693` 上で同じ失敗を再現したため、本粒と無関係な finding として保持し、repair authority はない。

`cargo test -p motolii-ui` の full package lane は green と数えない。`cu110pt` source assertion が `&timeline_projection.projection` を期待する一方、base `a4c13693` はすでに `render_projection()` を使うため red であることを exact base reproduction で確認した。oracle は変更しない。

これらの自動証拠は通常製品windowの人間目視を代替しない。human visual は M3 最終 checklist まで deferred のままである。

## 4. 状態遷移と次手

- `P02-C3` ruler producer/carrier sub-boundary: code / main `DONE` at `75ccd5e7`。
- 親 `P02-C3`: `INCOMPLETE`。focus、visible range、playback/audio、normal Inspector row/wiring、P04-C2 Easing を完了扱いにしない。
- `CU-0A08ITI`: `WAIT_TARGET` のまま。current-playhead carrierは本粒で成立したが、normal Position row/projectionとtyped Host intentは未成立である。
- `P04-C2`: `TARGET_MISSING` のまま。active interval / outgoing `Interp` / owner / command / consumer を発明しない。

current implementation `DO` はなし。次の実装は、別の docs contract が実在 owner、target、command、consumer と oracle を閉じた時だけ選定する。
