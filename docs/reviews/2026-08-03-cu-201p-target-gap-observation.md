# CU-201P native Timeline gesture target gap observation

状態: **PARTIALLY RESOLVED / RESIDUAL WAIT_TARGET**。`CU-201P-MOVE`はDONE、`CU-201P-TRIM-S`はSPEC DONE。広い親CU-201Pの残余targetだけを記録する。

## 確認した事実

- `crates/motolii-ui/src/host_pointer_capture.rs` の `HostPointerCandidate` は `Moved` / `Released` / `Cancelled` を返すが、現在の製品呼び出しはBrowserからStageへ置くactive place経路に閉じている。
- `crates/motolii-ui/src/product_runtime.rs` にはnative Timeline body-dragの`TimelineMoveGesture`と`MoveClip`配送が成立し、`CU-201P-MOVE`としてDONEである。
- 同ファイルの既存`handle_timeline_click`は、Timeline hitの`Key`／`Bar`を同じ`ReplacePrimary(layer)`へ写像し、`None`を`ClearPrimary`へ写像する。このselection routeは残余観察事実として維持し、trimのprivate Left/Right/Body refinementも全bar variantを同じlayerの`ReplacePrimary`へ写像する。
- `crates/motolii-ui/src/timeline_tools_host_runtime.rs` はTimeline Toolsのsnapshot publishとboundsだけを持ち、drag IPCまたはtyped Timeline intentを持たない。
- `crates/motolii-ui/src/document_edit_runtime.rs` には`MoveClip`が成立した。trimは[CU-201P-TRIM-S](2026-08-03-cu-201p-trim-edge-known-semantics-adoption-decision.md)でtargetを閉じ、実装`CU-201P-TRIM`を別粒とした。snap thresholdと高度interval操作は未接続である。

## 判定

`CU-201P-MOVE`と`CU-201P-TRIM-S`により、body moveとin/out trimは一契約境界へ閉じた。広い親CU-201Pではsnap threshold、slip/slide/roll/ripple、multi-select等のtargetが残る。Stage placementのpointer captureをTimelineへ流用したり、残余をgeneric coordinatorで束ねたりすることは禁止する。

## 最小の次手

接続可能な次粒は`CU-201P-TRIM`である。広い親CU-201Pは`SPLIT / WAIT_TARGET`とし、残余targetをtrim粒へ混ぜない。

## 非目標・停止条件

- `DocumentEditAction`、公開API、Document/serde/journal、Timeline UIの新しいstate ownerをこの観察から追加しない。
- Stage placementの成功をTimeline move/trimの製品証拠へ一般化しない。
- `CU-201P-TRIM`以外の残余targetを、検証済み既存targetなしに実装担当へ送らない。
