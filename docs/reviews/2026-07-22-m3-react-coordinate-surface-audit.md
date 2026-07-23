# M3 React coordinate surface機械監査

実施日: 2026-07-22

状態: **観察**。固定React source asset内の座標描画面を棚卸しし、既決のReact/native所有境界へ割り当てた。新しい公開API、Document意味、製品統合許可は追加しない。

## 1. 対象と方法

固定commit `56c318ed`の`docs/mocks-ui/src`を対象に、`canvas`、`getContext`、WebGPU、SVG、path、
pointer capture、pointer move、absolute-position projection、`requestAnimationFrame`を機械検索し、該当sourceを
目視した。Storybookの`canvasElement`、CSS class名の`stage-canvas`、asset名の`.svg`は描画runtimeの証拠から除外した。

```bash
rg -l --glob '*.{js,jsx,ts,tsx,html}' '<canvas\b|getContext\(' docs/mocks-ui/src
rg -l --glob '*.{js,jsx,ts,tsx}' '<svg\b' docs/mocks-ui/src
rg -l --glob '*.{js,jsx,ts,tsx}' 'onPointer|setPointerCapture|pointermove' docs/mocks-ui/src
```

結果はliteral `<canvas>` / `getContext()` **0件**、SVG component 3件だった。現行React正本には
browser WebGPU、Three.js、Konva、Pixiを製品描画runtimeとして使う箇所もない。

## 2. 座標描画面の分類

| React source | 描画・入力の実体 | 製品owner | 現在地 |
|---|---|---|---|
| `EasingGraphCandidate.jsx` | 5 SVG、Bezier/高度preset path、handle pointer操作 | native popup。Reactはtriggerと現在値要約 | native `510 x 284`外観、drag 1 commit、user preset/favoriteのisolated spike合格。高度補間意味と全visual matrixは未接続 |
| `GraphViewCandidate.jsx` | 1 SVG、multi-channel curve、key/tangent drag、playhead、resize座標変換 | native Timeline系surface + headless interaction | Blender-like native core fixture合格。pan/zoom/marquee/AX、Motolii snapshot/D2接続は後続 |
| `TimelineCandidate.jsx` time plane | absolute-position DOM、bar/key/playhead、row/depth投影 | native Timeline | time ruler、S/M rail、bar、key、playheadはnative外観first pass合格。headless操作、D2、AXは未接続 |
| `TimelineCandidate.jsx` Depth Rail | absolute-position DOM上の一次元Z projection、stack、range、分布preview | native Z軸Timeline | ownerと意味は既決。native UI fixtureは未作成 |
| `TimelineCandidate.jsx` Key Tools内SVG | Stagger説明用の小さな静的curve | React `KEYS / LAYERS` tool panel | Reactへ残す。nativeへ複製しない |
| `StageSurface.jsx` | `stage-canvas`というclass名の静的DOM | native Stage renderer | DOMの箱を移植しない。実映像、selection outline、2D handle、3D gizmoをnative presentation overlayへ投影する |

Browser、Inspector、panel resize、検索、tag、form、thumbnail/list、`KEYS / LAYERS`、Align / Stagger /
Stretchは座標描画面ではない。pointer handlerや`requestAnimationFrame`があっても、それだけでnativeへ移さない。

## 3. native再現の残量

Reactモックの外観をnativeへ写す作業として残る主対象は次である。

1. **Z軸Timeline / Depth Rail**: 固定React Depth Railをoracleに、一次元Z軸、marker、同一Z stack、range、分布previewをnativeへ投影する。

Multi-key Graph Viewは固定Reactの同じ3 channel / 9 keyをnative curve、key、tangent、playheadへ投影し、
core外観とkey/handle dragまで合格した。pan/zoom/marquee/AX/D2は外観再現ではなく製品操作接続の残件である。

Stage handle/gizmoはReactモック移植ではなく、本物のnative Stage overlay実装である。通常Timeline time planeは
すでに外観first passがあるため、同じ画面を作り直さず操作kernelと製品projectionを接続する。

## 4. 進行順

現行M3直列ledgerとG0-9製品統合停止線を変更せず、各依存が開いた時に次の順で閉じる。

1. 固定React fixtureからsemantic inputだけを抽出し、React DOM/SVG stateをownerにしない。
2. Multi-key Graph Viewのnative **外観fixture**を先に作り、Easing popupとtheme/curve語彙を共有する。
3. Graph Viewへheadless hit-test、pan/zoom、key/tangent drag、Cancel、release 1 commitのfixtureを接続する。
4. Z軸Timelineを独立rendererにせず、既存native Timelineの同じlayout/hit-test/selection kernelへ一次元axisとして追加する。
5. 同一Z stack、Group scope、range分布preview、Cancel 0、Apply 1 Undoを閉じる。
6. Stage overlayは別ticketでselection outline → 2D handle → 3D gizmoの順に進め、canonical画素と混ぜない。
7. 最後に一つのHost snapshotをGraph View、Timeline、Depth Rail、Stage、React Inspectorへ投影し、二重stateが無いことをE2Eで固定する。

各段階で「見た目」「headless操作」「D2/Undo」「platform/AX」を別の合格欄にする。外観fixtureが動いたことを
製品接続完了と数えず、既知技術であることを理由にDocument意味や公開transaction APIを発明しない。

## 5. 非目標と停止線

- React所有のKey ToolsやStagger SVGをnativeへ写さない。
- Graph View、Depth Rail、Stageごとにselection、Undo、zoom正本を作らない。
- SVG path、CSS px、DOM event列をDocumentへ保存しない。
- Z軸を簡単な数直線として描けることと、Group scope・同一Z stack・D2意味の合格を混同しない。
- この監査だけでWebView/native製品統合、egui撤去、plugin UI公開契約を解除しない。
