# CU-201P native Timeline gesture target gap observation

状態: **WAIT_TARGET**。CU-201Pの製品gestureを実装せず、現行authorityとコードに存在するtargetの境界だけを記録する。

## 確認した事実

- `crates/motolii-ui/src/host_pointer_capture.rs` の `HostPointerCandidate` は `Moved` / `Released` / `Cancelled` を返すが、現在の製品呼び出しはBrowserからStageへ置くactive place経路に閉じている。
- `crates/motolii-ui/src/product_runtime.rs` はactive place中だけpointer candidateを読み、`place_preview` と `PendingStageDrop` を更新する。Timeline側の既存入口は `handle_timeline_click` の `TimelineHit`→`ReplacePrimary` / `ClearPrimary` だけである。
- `crates/motolii-ui/src/timeline_tools_host_runtime.rs` はTimeline Toolsのsnapshot publishとboundsだけを持ち、drag IPCまたはtyped Timeline intentを持たない。
- `crates/motolii-ui/src/document_edit_runtime.rs` の `DocumentEditAction` / `DocumentEditQueue` には、move / trim / snapを表す既存actionまたはqueue methodがない。既存D2 targetは `SetClipStart` / `TrimClipIn` / `TrimClipOut` のDocument側commandであり、Timeline gestureからの入力codec・owner・terminal admissionは未接続である。

## 判定

`CU-201N-S` の候補（`TimelineKey` / `TimelineBar` / transient threshold）は閉じているが、CU-201Pの `drag start → preview → release/cancel → existing D2 command` を一契約境界で実装できる既存targetは閉じていない。Stage placementのpointer captureをTimelineへ流用したり、Timeline-specific queue/action/coordinatorを新設したりして埋めることは禁止する。

## 最小の次手

既存targetが生じるまでCU-201Pは `WAIT_TARGET` のまま保持する。次に必要なのは、Timeline gesture owner、pointer captureの入力境界、既存 `SetClipStart` / `TrimClipIn` / `TrimClipOut` へのtyped request、release/cancel admissionの4点を既存authority内で一つの実在接続票へ閉じることだけである。未決のまま実装へ戻さない。

## 非目標・停止条件

- `DocumentEditAction`、公開API、Document/serde/journal、Timeline UIの新しいstate ownerをこの観察から追加しない。
- Stage placementの成功をTimeline move/trimの製品証拠へ一般化しない。
- 検証済みの既存targetが見つからない限り、CU-201PをSpark/Composer等へ発注しない。
