# M3製品モック一括回収計画（2026-07-21）

状態: **停止線**。D&D transport、windowed Timeline、同一画面の複数Surface、community brokerは
部分検証済みだが、Rectangleの製品編集契約、Vector描画、三面selection、製品Timeline操作、強隔離、
追加hardwareとWindows受入は未完了である。

2026-07-22追補: 本計画のReact Browser、Inspector、`KEYS / LAYERS`、Easing Panelは
[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)に従う。
固定モックをoracleだけにして縮約版を作る経路は棄却し、固定sourceをproduct ownerへ移してmockをconsumerへ
反転する。Inspectorは現固定sourceに正しい独立React componentが無いため、legacy出力を固定モック内で
同形React化してから移管する。契約確認用縮約画面はdevelopment専用diagnostic routeへ分離し、通常製品面の
代替にしない。

## 1. 採択する到達点

最初の製品縦切りは次で固定する。

```text
React Browser Rectangle intent
  -> host coordinator
  -> transient preview
  -> release時だけD2 single writerへ1 macro
  -> Arc<Document> snapshot + selected LayerId
     -> native Preview
     -> native Timeline bar
     -> React Inspector projection
  -> Undo 1回で三面から消える
```

Timeline barは別Document objectとして作らず、同じ`TrackItem::Clip`のstable `LayerId`、start、durationから導出する。
ReactはDocument、selection、Undoの所有者にならない。Esc、pointer cancel、capture/focus loss、Stage外dropは
Document serialize、revision、Undo/Redo、ID counterを完全不変にする。duplicate dropとterminal後の遅延messageは
runtime drag ID、単調sequence、WebView/layout epochの有界dedupeで拒否する。これらのtransport IDをDocumentへ保存しない。

## 2. 現行コードで判明した停止線

1. `DocumentCommandRequest`は`DeleteTargetedItems`と`RemoveTrackItem`だけを受理する。Rectangle配置intentを
   既存Deleteへ偽装できない。
2. `DocumentWriter::apply_macro`は1 gesture=1 Undoと失敗時rollbackを既に持つため再実装しない。一方、
   `LayerId`予約を`AddTrackItem`と同じ原子操作へ含める契約は未定義である。drag startで`reserve_layer_id`を呼ぶと
   Cancel変更ゼロを破る。
3. 正本のRectangleは`VectorRecipe::StandardShape::Rect`だが、現行D3は`ClipSource::Vector`を
   `UnsupportedVectorSource`で拒否する。static previewのfixture用rectを製品Shape完成の代用にしない。
4. selection/focusの三面共有はU2h、native TimelineはU3aに属する。現行直列順を迂回してReact worktreeへ
   独自writer、selection、historyを作らない。
5. React資産の接続worktreeはmainより古く、main到達済みU2a/U2bを含まない。製品統合前にmainへ再結合する。

したがって、公開Command、journal、LayerId予約方式をこのspike内で発明せず、D2の個別仕様改訂とVector loweringを
先行させる。短期fixtureは「編集transportだけ」と明記し、製品完成へ数えない。

D2案のread-only反対側レビューはP0=0 / P1=6だった。汎用planner案は採択せず、U2bがactionをpopした後に
writer現状態からPlace専用draftを同期planし、live-nextをtyped検査して同じcall stackで既存`AddTrackItem`を
`apply_macro`するprivate経路D0を最小比較候補とする。journalは現U2bの所有外なので、互換可能性とdurability合格を
分離する。drop位置のTransform/Shape center、size/scale、表示名も仕様で決まるまでdefault化しない。

## 3. 並列実測で回収した事実

### D&D lifecycle

- macOS actual pointerでWebView外もpreviewが継続し、最終手動sessionだけでpreview 686件を受信した
- ReactへEsc、window blur、`pointercancel`、`lostpointercapture`回収を追加したisolated spikeで、
  Playwrightはdrag途中Escをdrop 0 / cancel 1として確認した
- macOS appへのEsc実操作でも`cancel_count=1`、`drag_preview_visible=false`を確認した
- `drag.id`+単調sequenceと64件のterminal ID表で、duplicate drop、同一sequence、drop後の遅延preview/cancelを
  unit testで拒否した
- drop自身が最終座標を持つため、preview欠落時に過去preview位置をcommitへ使わない

この結果はTransient transportの成立であり、D2 exactly-once commitの証明ではない。

### Timeline

責任境界は、`KEYS / LAYERS`切替とAlign/Stagger/Stretch等のtool panelをReact、time ruler、各rowへ
同期するS/M rail、bar、key、playhead、Z軸Timeline / depth railをnativeとする。React側はintentを送るだけで、
native Timelineと別のselection、Undo、semantic stateを所有しない。

Apple M4 / Metalで既存native benchmarkを再実行した。

| 条件 | 結果 |
|---|---:|
| clip / key | 1,000 / 100,000 |
| 同時visible key | 20,005 |
| warmup / measured frame | 120 / 600 |
| median / p95 | 2.846 ms / 4.294 ms |

この後、実windowのdirect wgpu fixtureへ100,000 key（うち10,000 selected）とopaque WKWebView 2枚を同居させた。
Apple M4 / Metal、30.017秒、1,729 measured frameで、acquire/presentは1,849/1,849、readback 0、
hot-loopのpipeline/buffer/bind group/texture生成0だった。acquire-to-present p95は15.352ms、present間隔p95は
17.006ms、throughputは57.60fpsである。100,000 keyは常用規模を大きく超えるstress条件なので、
約1.078秒の外れ値と16.667ms超849件を診断記録として保持した上で、**容量・描画基盤は合格**とする。

text/icon、theme、React parity、playhead、marquee、snap、hit-test、10,000 key実drag、D2 commit、GPU timestamp、
input latency、Windowsは未証明である。次は同じharnessへtoolkit非依存layout/hit-testとTransient drag projectionを
接続し、drag中semantic write 0、release時D2 1 commitを測る。Velloは複雑path/textの独立比較枝とする。

### detached Preview / 複数Surface

接続済みReact worktreeのmacOS fixtureで、EditorとPreviewを2 top-level window / 2 wgpu Surfaceとして生成し、
1 device/queueを共有した。Previewだけへの決定的な疑似surface-lost再configure、fullscreen往復、close後の
Editor present継続、Preview再生成、Host snapshotのstable ID・selection・Shape数保持に合格した。

これは同一Retina画面の構造証明に限る。疑似lostはdriver障害ではなく既存の`Lost / Outdated`分岐を通す注入であり、
実surface/device lost、異DPI monitor、第二monitor、HDR/SDR、Windows WebView2は未証明である。

### community panel

既存のopaque-origin iframe負例はparent DOM、storage、network、native bridgeの直接access拒否に合格した。
fixture typed brokerは`theme.read`だけを許可し、`document.raw`と`native.invoke`を拒否した。Vite production static
buildも成立した。ただしiframe/CSPはWebView native IPC、infinite loop、OOM、renderer crashの
隔離証明ではない。Hostとcommunityは同じversioned React kitを使い、実行realmとcapability profileだけを分ける。
raw Document、汎用native invoke、filesystem/network/GPU handleは渡さない。

強いCPU/RAM隔離が完成条件ならWebView renderer分離だけでなく、panel専用helper process、watchdog、kill/recreate、
host snapshotからの再構成が必要である。WebView2ではorigin検査と`ProcessFailed`、WKWebViewではcontent process
terminationを審判へ含める。

## 4. 実行順と並列可能範囲

依存する意味契約は直列、同じfixtureを読む実測は並列にする。

1. **D2仕様**: valid drop時のLayerId予約+`AddTrackItem`を失敗時も原子的にする個別契約を決める
2. **Vector経路**: `StandardShape::Rect`のD3 lowering / GPU previewを既存M2意味から実装する
3. **共通fixture**: 1 Rectangle、stable ID、RationalTime、正準Y-up座標、selection projectionを固定する
4. ここから次を並列化する
   - D&D exactly-once + Apply/Undo/Redo
   - Timeline layout kernel / 100k windowed render / 10k transient drag
   - host snapshot -> native Preview/Timeline + React Inspector
   - same-display detach window / surface fault injection
   - bounded AccessKit tree / keyboard / IME / focus
   - capability broker / offline bundle / crash-loop復旧
5. 追加hardwareで異DPI、第二monitor、HDR/SDR、penを実測する
6. Windows 10/11実機でWebView2、PMv2、MS-IME、NVDA、offline runtime、process failureを測る

## 5. 次fixtureの必須負例

- start / preview / Esc、Stage外drop、capture/focus loss: Documentと全counter不変
- preview欠落 + valid drop: drop座標でShape 1件だけ追加
- duplicate drop、terminal後のstale preview/cancel: revision、Undo長、ID counterが1回分だけ
- commit失敗注入: Document、history、revision、ID counter不変
- accepted drop: Preview、Timeline、Inspectorが同じLayerIdとsnapshot revisionを表示
- Undo 1回でShape/bar/Inspector selectionが消え、Redo 1回で同じLayerIdが復帰
- drag move中: semantic write、snapshot publish、D2 callが全て0。release時`apply_macro` 1回
- 10,000 key drag: 相対間隔維持、RationalTime snap、Cancel完全復元
- 100,000 key: density境界往復でselection、playhead、visible time range不変
- React reload / panel crash: HostのDocumentとselectionを失わず、最新snapshotから再投影

## 6. platform分類

| 分類 | 対象 |
|---|---|
| 現Macで計装後に実行可能 | cancel/lost capture/window外release、同一画面の複数Surface、fullscreen、focus、macOS IME、VoiceOver、offline、panel crash/loop |
| 追加hardware必須 | 異DPI monitor、第二monitor、HDR/SDR差、pen。現機は内蔵Retina 1台 |
| Windows実機必須 | WebView2 z-order/capture、PMv2 DPI、MS-IME、NVDA、offline runtime、`ProcessFailed`復旧 |

macOSの結果をWindows合格へ外挿せず、追加hardwareが無い項目をsynthetic scaleや一次資料だけでPASSへ上げない。

## 7. 既知技術

- pointer lifecycle: [Pointer Events](https://www.w3.org/TR/pointerevents/)
- WebView host: [wry WebViewBuilder](https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html)
- Windows broker安全: [WebView2 security](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security)
- Windows process failure: [WebView2 process-related events](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-related-events)
- offline runtime: [WebView2 distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)
- Windows DPI: [Per-monitor DPI](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)
- bounded native accessibility: [AccessKit](https://docs.rs/accesskit/latest/accesskit/)
- GPU計測: [wgpu timestamp queries](https://wgpu.rs/doc/wgpu_examples/timestamp_queries/index.html)
- native curve/path: [Vello](https://github.com/linebender/vello)、[kurbo](https://docs.rs/kurbo/latest/kurbo/)

Rerunはこの判断の根拠に使っていない。React visual/interaction mockはoracleとして維持するが、CSS px、DOM identity、
React stateをDocument/APIへ焼かない。

## 8. 二重状態の禁止

React、native Preview、native Timeline、community panelはsemantic stateの所有者にならない。正本はHostの
D2 single writerが管理するDocumentと、Host coordinatorが管理するTransient selection/sessionだけである。
各surfaceは同じrevision付きsnapshotから導出したread-only projectionを表示し、intentだけをHostへ返す。

独自Document cloneへの編集、surface別selection store、React側Undo/history、native側の別semantic model、
projection失敗時のローカル確定を禁止する。再描画、WebView reload、window detach/reopenは最新Host snapshotから
再構成し、片側の状態をもう片側へ同期して整合させる経路を作らない。
