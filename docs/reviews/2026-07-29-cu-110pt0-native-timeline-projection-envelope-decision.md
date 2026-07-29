# CU-110PT0 native Timeline投影envelope決定

- 日付: 2026-07-29
- 状態: **決定 / DONE**
- 後続: **CU-110PT DO**

## 1. 結論

`CU-110PT`が既存`project_timeline`へ渡す初期描画envelopeを、採用済み
published snapshotの`[RationalTime::ZERO, composition.duration)`とする。
これはDocumentから毎回導出するread-only入力であり、保存、操作、復元される
visible-range stateではない。

metricsは時間軸を`0.0..=1.0`へ正規化する
`units_per_second = 1.0 / composition.duration.as_seconds_f64()`とし、
band軸は`band_height = 1.0`、未描画のkey hit extentは`1.0`を渡す。
native rendererはbarの正規化済みxと最大band spanを現在のTimeline logical rectへ写像する。
window size、DPI、monitor値をDocument、公開API、serdeへ流さない。

## 2. 根拠と境界

- `CU-110P`は同じsnapshotを既存`project_timeline`へ渡すことを決定済みである。
- `timeline_projection`はviewport / metricsをcaller注入で受け取り、owned rangeを持たない。
- `U3a-2Q-V`が未決としているのは、操作・寿命・復元を伴うvisible-range **state owner**、
  値shape、defaultである。本envelopeはそれらを追加せず、Documentの既存有効区間を
  一回の描画入力へ写すだけである。
- `CU-110PT`ではpan、zoom、scroll、Fit command、selection、hit-test caller、playheadを
  一切追加しない。したがって`U3a-2Q-V`は`WAIT`のまま維持する。

## 3. 実装許可

- `product_runtime.rs`内のprivate projection holder、published snapshot採用直後の再投影、
  native Timeline bar rect変換と描画。
- 既存`project_timeline` / `TimelineProjection` / `NativeHostLayout`の再利用。
- 同じsurface、device、queue、render pass内の追加draw。
- headless projectionの既存typed errorを`ProductRuntimeError`で透過する。

## 4. 非目標

- visible-range state owner、値shape、default、lifetime、復元、serialization。
- Timeline pointer入力、`TimelineHit` production caller、selection、focus、playhead。
- React Timeline、semantic zoom、key描画、label、scroll、zoom、Fit command。
- 公開API、Document、serde、journal、plugin契約、Undo/historyの変更。
- 新しいprojection実装、surface、renderer、GPU/CPU pixel path。

## 5. 負例

1. viewportまたはmetricsをDocument / settings / sessionへ保存する。
2. windowごとのrange正本、二つ目のDocument snapshot、selection storeを作る。
3. `project_timeline`を迂回してclipを再走査し、別bar layoutを作る。
4. pointer / wheel / gestureから本envelopeを書き換える。
5. React `TimelineCandidate`、mock値、fixture defaultを通常製品へ持ち込む。
6. visual threshold / golden /既存受入期待値を変更する。

## 6. STOP

1. 描画成立にinteractive visible-range stateまたはそのowner決定が必要になる。
2. `project_timeline`の公開型、Document意味、永続形式の変更が必要になる。
3. key / semantic zoom / selection / hit-test / playheadを同じ粒へ含める必要が出る。
4. 同期GPU readback、別surface、別rendererが必要になる。

`CU-110PT0`は`DONE`。次の唯一のPRODUCT-ASSET `DO`は`CU-110PT`。
