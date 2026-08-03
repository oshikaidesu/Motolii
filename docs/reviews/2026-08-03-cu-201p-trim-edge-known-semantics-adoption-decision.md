# CU-201P-TRIM native Timeline trim-edge known-semantics adoption

- 日付: 2026-08-03
- 状態: **SPEC DONE / REDUCE: IN-OUT TRIM ONLY**
- 親: CU-201P / U3b / VS-2

## 1. 結論

Timeline上の既存Clipは、`Key`を最優先し、残るbar内部を`LeftEdge / RightEdge / Body`へ分ける。
左edgeは既存`TrimClipIn`、右edgeは既存`TrimClipOut`、bodyは実装済み`SetClipStart`へ写す。
edge dragは`CU-201P-MOVE`と同じ`ProductApp`内のprivate Transient lifecycleを再利用するが、
`TimelineMoveGesture`を汎用interval frameworkへ変えず、trim専用のprivate型として閉じる。

hit targetはBlender VSEのhandle選択実装を`PATTERN / REDUCE`採択する。Motoliiの通常製品routeは
pointerをphysical pxからlogical pxへ一度変換済みなので、bar内部のedge幅を
`min(15 logical px, bar logical width / 4)`とする。bar幅が25 logical px未満、またはbar高が
16 logical px未満なら左右edgeを無効にし、全域をbodyとして扱う。これにより短いbarで左右edgeを
推測分割せず、既存moveへ安全に縮退する。

現行`ProductTimelineProjection`はwhole-composition viewport（`ZERO..composition.duration`）を使い、
この粒には既に認可されたzoom／semantic-zoom routeがない。したがって全尺表示でbar logical幅が25未満、
またはband logical高が16未満になるclipは意図的にbody-onlyのままにする。trimを全clipへ到達可能とは
扱わず、zoomをこの粒で発明しない。

Blenderのbar外側padding、隣接二stripの両handle同時選択、effect strip、lock、channel、multi-selectは
Motoliiへ持ち込まない。GPL sourceを移植せず、利用者interactionと数値規則だけをMotolii fixtureで再表現する。

## 2. 既知実装の固定根拠

比較時点のBlender固定commitは`6e15da150d397d3c6e95e4d3ca147f0150bb7311`。

| 根拠 | 証明すること | 証明しないこと |
|---|---|---|
| [Blender `sequencer_select.cc` handle size / cutoff](https://github.com/blender/blender/blob/6e15da150d397d3c6e95e4d3ca147f0150bb7311/source/blender/editors/space_sequencer/sequencer_select.cc#L883-L945) | 内側handleは15 pxかstrip幅1/4の小さい方。25 px未満または高さ16 px未満ではhandle無効 | left/rightの順序、MotoliiのDocument意味、公開型、outside padding、隣接strip選択 |
| [Blender `pick_strip_and_handle`](https://github.com/blender/blender/blob/6e15da150d397d3c6e95e4d3ca147f0150bb7311/source/blender/editors/space_sequencer/sequencer_select.cc#L1017-L1035) | `sequencer_select.cc`の1017-1035行でleft-before-rightのhit-test順序を証明し、bar選択とleft/right handle分類が一つのpointer hit ownerにある | Motoliiのqueue、writer、cancel/stale規則 |
| [Blender manual: strip handles](https://docs.blender.org/manual/en/3.0/video_editing/sequencer/editing.html) | 左handleを動かすとsource先頭を飛ばして開始を変え、handleがtrim interactionである | pixel値、MotoliiのTimeMap式、Undo/journal |
| [CU-201T-S](2026-08-01-cu-201t-s-clip-trim-timemap-contract-decision.md) | Motoliiのleft/right trim意味、拒否順、Undo/journal | pointer hit target |

Kdenlive／Shotcutは左右edge dragの収束確認には使えるが、今回のexact targetはBlender固定sourceだけで閉じる。
複数実装の数値を混ぜたり、GPL codeをコピーしたりしない。

## 3. 既存契約接続票

| 項目 | 内容 |
|---|---|
| `AUTHORITY` | [M3 U3b](../specs/M3-ui-integration.md)、[CU-201T-S](2026-08-01-cu-201t-s-clip-trim-timemap-contract-decision.md)、[CU-201N-S](2026-08-03-cu-201n-s-snap-target-contract-decision.md)、[CU-201P-MOVE](2026-08-03-cu-201p-move-known-semantics-adoption-decision.md) |
| `INTERNAL TARGET` | 公開`TimelineProjection::hit_test`／`TimelineHit`、`TimelineBar::{x_start,x_end,y_top,y_bottom}`、private `ProductTimelineHit`／`ProductTimelineProjection::hit_test`、`ProductApp::handle_timeline_click`とnative pointer lifecycle、`DocumentEditRuntime`、`DocumentWriter::prepare_trim_clip_in/out` |
| `OWNER` | hit分類とdrag candidateは既存`ProductTimelineProjection`／`ProductApp`内のcrate-private Host Transient。`ProductTimelineHit`はこのprivate境界だけが所有し、確定intervalとTimeMapはDocument。新public owner、coordinator、第二writerなし |
| `WRITE ROUTE` | 公開hitの`Key`／`None`は同じ意味でprivate hitへ写像し、公開`Bar`だけをLeft/Right/Bodyへ精密化する。Left/Right pressで開始pointer／edge値／generationを凍結し、drag中はread-only preview、release一回だけ既存Trim prepareをsingle writerへ渡す。Bodyは既存move |
| `GAP` | 公開`TimelineHit`が`Key / Bar / None`のみである一方、製品側privateな左右edge refinement、logical px幅／高さのadmission、trim専用typed request、pointer初期値固定が未接続 |
| `RESOLUTION ROUTE` | Blender VSEの固定sourceを`PATTERN / REDUCE`採択し、現行projection／move lifecycle／Trim commandへ薄く接続 |
| `DISPOSITION` | `PASS`。次の一契約はPRODUCT `CU-201P-TRIM` |

## 4. Target contract

1. 公開`TimelineHit`と公開`TimelineProjection::hit_test`はbyte-for-byteでAPI／意味を変更しない。すなわち
   `pub fn hit_test(&self, x: f64, y: f64) -> TimelineHit`、`Key { layer, key }`、`Bar { layer }`、`None`、
   Key優先と最小`LayerId` tie-breakをそのまま維持し、public側でedge分類を行わない。
2. 製品側だけにcrate-private `ProductTimelineHit`を置き、private `ProductTimelineProjection`／`ProductApp`
   だけが所有する。`Key { layer, key }`と`None`は既存public hitからそのまま写像し、public `Bar { layer }`だけを
   `Left { layer }`／`Right { layer }`／`Body { layer }`へ精密化する。public re-export、public variant、public
   signatureは追加しない。
3. private hitはまずlogical time surface内かを確認し、既存public hitへ同じ正規化座標を渡す。bar logical幅は
   `(bar.x_end - bar.x_start) * time_surface.width`、bar logical高は正確に
   `time_surface.height / band_span`から導出する。`time_surface.height`、`band_span`、導出値がfiniteかつpositive
   である場合だけbar高をadmitし、そうでなければLeft/Rightを生成せずpublic BarをBodyへ写像する。
4. bar logical幅`>= 25`かつadmit済みbar logical高`>= 16`の時だけedgeを有効にする。内部edge幅は
   `min(15, bar_width / 4)` logical px。left/rightは重ならず、残りはbodyになる。cutoff未満はBodyだけを返し、
   中央分割、片edge優先、不可視handle、外側hit paddingを作らない。
5. 現行viewportはwhole compositionであり、bar logical幅25未満またはband logical高16未満のclipは、既に認可された
   zoom routeが現れるまで意図的にBody-onlyとする。この粒でzoom／semantic zoomを発明せず、trimをuniversally reachable
   と扱わない。
6. Left/Right press時に対象`layer`、`initial_pointer_time`、対象clipの`initial_start`／`initial_end`、選択edge
   （Leftなら`initial_start`、Rightなら`initial_end`）、開始projection generationを凍結する。live Documentや
   pointerから初期edgeを再解決しない。
7. previewとreleaseは毎回`delta = current_pointer_time - initial_pointer_time`だけを使う。Leftは
   `new_start = initial_start + delta`、Rightは`new_end = initial_end + delta`とし、press／preview／releaseのどこでも
   `current_pointer_time`へ直接jumpしない。Left releaseは`prepare_trim_clip_in(layer, new_start)`、Right releaseは
   `prepare_trim_clip_out(layer, new_end)`へprivate typed requestを一件だけ渡す。same-valueは`None`。
8. `handle_timeline_click`はprivate barの`Left`／`Right`／`Body`をすべて同じ`ReplacePrimary(layer)`へ写像し、
   private `Key`も既存どおり`ReplacePrimary(layer)`、private `None`は既存どおり`ClearPrimary`へ写像する。edge clickで
   selection layerを変えない。
9. drag中Document/journal/history/revision/publishは0。Escape、focus loss、capture loss、stale generation、
   対象消失、算術失敗、prepare拒否はcommit 0で終了する。Bodyは既存`CU-201P-MOVE`をそのまま使い、trim追加のため
   move threshold、snap、selectionを変更しない。

## 5. Primary oracle

- Key上は常にKey。通常幅barのleft/right/body境界値と境界外をtable testする。
- 25 logical px直前はbodyのみ、25 logical pxちょうどでedge有効。高さ16も同じ境界を持つ。
- 幅60以上ではedge内部15 logical px、幅25〜60では幅の1/4で左右非重複。
- whole-composition viewportで幅25未満／導出高16未満がBody-onlyになること、認可済みzoomなしにtrimが全clipへ到達可能とならないこと。
- press時の`initial_pointer_time`／initial edgeを固定し、同一点releaseはno-op、別時刻は常に上記delta式になること。pointerへのjump 0。
- Left/RightのpreviewはDocument不変、release一回で対応Trim command一件・Undo一回。
- same-value、cancel、stale、invalid、duplicate releaseはDocument/history/revision不変。
- public `TimelineHit`／`TimelineProjection::hit_test`のAPI／意味不変、private bar全variantのedge clickが同じlayerのReplacePrimaryになること、Body moveの既存oracleとKey優先、最小`LayerId` tie-breakを維持する。

## 6. 非目標とSTOP

- Blender source code、型名、outside padding、隣接strip dual-handleを移植しない。
- snap threshold、slip、slide、roll、ripple、multi-select、lane変更、playhead／marker／frame-grid snapを含めない。
- `TimelineMoveGesture`のgeneric化、Stage placement capture再利用、zoom／semantic zoom、新coordinator、新public APIを行わない。
- public `TimelineHit` variant、public `TimelineProjection::hit_test` signature／meaning、selection meaningを変更しない。
- Document schema、serde、journal version、plugin契約、永続形式、Trim command意味を変更しない。
- logical rectからbar widthと`time_surface.height / band_span`をfinite positiveに導けない、またはedge previewに新しいDocument意味が必要なら
  `CU-201P-TRIM`をSTOPしてSolへ戻す。
