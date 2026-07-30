# CU-0B04S Browser→native Place終端所有の再締結

- 日付: 2026-07-29
- 状態: **決定 / DONE**
- 対象: VS-1 Rectangle、`CU-0B04N` / `CU-0B04R` / `CU-0B05`
- 前提: `CU-0B03`

## 1. 再現した停止点

実Macの通常project sessionで、product-owned Browserから`browser.place`はnative
Host inboxへ届き、既存Rectangleはcanonical GPU renderでStageへ表示できた。一方、
HTML5 dragがpointer releaseをWebView内で消費するため、比較baselineのegui Stageは
最終releaseを受け取れず、新しいRectangleはDocumentへ追加されなかった。

したがって、`Browser→Host inbox→egui release→Place D2`を製品経路の成立証拠とは
数えない。既存文書の「native Stage final releaseまで実装済み」という表現も本決定で
撤回する。

## 2. 採択

1. Reactはdecode済み`scope_ref` / `item_id`から、drag開始時にtyped
   `browser.place`を一件だけ送る。座標、terminal、commit、Document、selection、
   Undoを所有しない。
2. Host callbackが存在する製品経路では、同じgestureにlegacy HTML5
   `DataTransfer` payloadを併用しない。Host callbackが無いmock / diagnostic consumer
   の既存interactionだけは互換経路として維持できる。
3. pointer lifecycle、candidate terminal、Esc / window外 / capture loss / focus loss
   cancelは、Host private platform capture adapterが所有する。OS型、window座標、
   layout epochはTransientに閉じ、Document、D2、plugin契約、公開raw APIへ出さない。
4. native Stageは同じ最新`layout_epoch`のviewportでhit-testし、その時点の表示cameraを
   用いてcanonical Y-upへ変換する。Stage外release、stale epoch、camera不在は変更0。
   中央配置、double-click、Reactからの最終座標送信へ代替しない。
5. 本決定はcapture ownerを閉じるだけであり、`CU-0B04N` / `CU-0B04R` /
   `CU-0B05`を完了扱いにしない。preview / terminalの認可、epoch / sequence /
   dedupeは`CU-107PV→CU-107TC→CU-107AD→CU-107TD`、D2 mutationは`CU-110`、
   Undo/Redoは`CU-111`を通す。
6. `CU-0B02N`のvisual token consumerはcapture、viewport geometry、hit-testの
   load-bearing prerequisiteではない。状態は`WAIT`のまま保ち、本筋へ先行させない。

## 3. 次の一粒

`CU-0B04P`をHost private platform capture seamの実装粒とする。macOS adapterと
toolkit非依存の小さなstate machineでpointer sample / release / cancel候補を取得し、
Document mutation、Stage admission、D2、Undoを行わない。既存OS入力・依存で包めない、
React terminal、egui event、公開API、永続field、既定配置が必要になった時点でSTOPする。

`CU-0B04P`の後も、最新layout epochを所有するnative Stage viewportへ接続する
`CU-0B04N/R`、製品再投影`CU-0B05`の順序を追い越さない。

## 4. 反対側確認

- Cursor Grok 4.5 Highのread-only確認では、HTML5 dragとtyped intentの二重gesture、
  egui releaseを製品terminalにした経路、実機未成立を完了表現した文書をP0と判定した。
- Claude Opus 5はread-only相談を開始したが応答が返らず中断した。別modelへの黙った
  fallbackは行っていない。後続の限定相談では完全model IDを維持し、CLIの
  `--effort low`を使用する。

外部回答はauthorityではなく、本決定を既存surface topology、UI境界、Place責任連鎖へ
再照合するための助言として扱った。
