# CU-201P native Timeline gesture target gap observation

状態: **REOPENED / EXTERNAL_GATE_PENDING**。`CU-201P-MOVE-S`と`CU-201P-TRIM-S`はSPEC DONEだが、製品pointer adapter authorityは未閉鎖である。

## 確認した事実

- `crates/motolii-ui/src/host_pointer_capture.rs` の `HostPointerCandidate` は `Moved` / `Released` / `Cancelled` を返すが、現在の製品呼び出しはBrowserからStageへ置くactive place経路に閉じている。
- PR #441のRust CI再検証で、`product_runtime_adapter.rs`へ追加された`CursorMoved / MouseInput / Focused / CursorLeft`と、`product_runtime.rs`へ漏れたraw winit型を`raw_input_boundary`が拒否した。`CU-0B04NA`は同adapterをlifecycle 5種だけへ閉じ、pointerを明示拒否している。
- 同ファイルの既存`handle_timeline_click`は、Timeline hitの`Key`／`Bar`を同じ`ReplacePrimary(layer)`へ写像し、`None`を`ClearPrimary`へ写像する。このselection routeは残余観察事実として維持し、trimのprivate Left/Right/Body refinementも全bar variantを同じlayerの`ReplacePrimary`へ写像する。
- `crates/motolii-ui/src/timeline_tools_host_runtime.rs` はTimeline Toolsのsnapshot publishとboundsだけを持ち、drag IPCまたはtyped Timeline intentを持たない。
- `SetClipStart`と`TrimClipIn / TrimClipOut`のDocument command／Writer／journal／Undoは成立済みである。製品windowからそれらへ渡すTimeline pointer routeだけが未接続で、snap thresholdと高度interval操作も未接続である。

## 判定

`CU-201P-MOVE-S`と`CU-201P-TRIM-S`によりbody moveとin/out trimの意味は閉じたが、製品接続は閉じていない。違反実装grain `43727b77`はPR #441 CI修復で局所revertし、製品MOVEを`EXTERNAL_GATE_PENDING`、製品TRIMを`WAIT_TARGET`へ戻す。広い親CU-201Pではpointer adapterに加え、snap threshold、slip/slide/roll/ripple、multi-select等のtargetが残る。Stage placementのpointer captureをTimelineへ流用したり、残余をgeneric coordinatorで束ねたりすることは禁止する。

## 最小の次手

次粒は製品実装ではない。`CU-0B04NA`を黙って広げず、raw winit型をadapter file内でtoolkit-free eventへ変換するexact private pointer adapter authorityを独立に閉じる。成立後だけMOVEを再投入し、TRIMはMOVEの境界成立を再確認してから`DO`へ戻す。

## 非目標・停止条件

- `DocumentEditAction`、公開API、Document/serde/journal、Timeline UIの新しいstate ownerをこの観察から追加しない。
- Stage placementの成功をTimeline move/trimの製品証拠へ一般化しない。
- MOVE／TRIMを、検証済みexact private pointer adapter authorityなしに実装担当へ送らない。
