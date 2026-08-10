# M5 平面グラデーションとPathクリッピングマスクのRerun proof

日付: 2026-08-10
状態: **決定／M5-FILTERMASK-P0 `DONE / PRIVATE PROBE`、製品Stage未接続**

## 1. Outcome

Motoliiの主部が2D graphicsであることを優先し、最初のfilter proofを3D materialへの色付けではなく、
**正準2D平面のlinear gradientをPath coverageで切り抜く処理**とする。

```text
planar gradient content × Path coverage mask -> premultiplied RGBA -> source-over
```

円Pathの内側だけに青から桃のgradientを表示し、外側は透明、背面の半透明Rectとのsource-overを維持する。
これはクリッピングマスク意味の最小例であり、3D mesh、texture authoring、Document variant、第二rendererを作らない。

## 2. 既知実装preflight

| 項目 | 裁定 |
|---|---|
| MECHANISM CLASS | 2D contentをvector coverageでclipするpremultiplied fill pass |
| KNOWN IMPLEMENTATION SEARCH | 既存`Path2DFill` custom visualizer／renderer、Rerun transparent phase、既存source-over oracle |
| CANDIDATES | 別mask texture、Rerun custom renderer内の融合pass、3D mesh vertex color |
| ADOPTION ROUTE | 既存Path2D rendererを`REUSE`し、fragment内でgradientを評価。Path raster coverageをclip maskとして`ADOPT / WRAP` |
| REJECTED CANDIDATES | 3D vertex colorは2D filter意味ではない。別mask textureは一回だけ使うvector maskには過剰 |
| THIN MOTOLII SEAM | `PlanarPaint + Path -> triangle coverage -> premultiplied gradient -> source-over` |
| THIN MOTOLII RESIDUAL | 正準座標、mask／content意味、effect順序、Preview／Export、Document／D2 |
| RETIREMENT | probe Blob codec、convex fan、frame内resource生成は製品へ昇格しない |
| BUILD JUSTIFICATION | NONE |
| BUILD | FORBIDDEN |

## 3. 成立したproof

private probeはsolid paintを同色両端の`PlanarPaint`として保持し、linear gradientではobject-localな
start／endと2色をpayloadへ渡す。fragment shaderは各fragment位置をgradient軸へ射影する。
Pathのtriangle外ではfragmentが生成されず、MSAA coverageが境界maskになる。色はGPUへ渡す前にpremultiplyし、
既存Rerun transparent phaseのsource-overを変えない。

![円Pathでclipした平面グラデーション](../../spikes/rerun-path2d-probe/rerun-planar-gradient-clipped-circle.png)

自動oracleは、mask coverage 0で`[0, 0, 0, 0]`、gradient中央かつcoverage 1で両端色のpremultiplied中間値、
payload roundtrip、既存source-over、Path閉路を固定する。

## 4. 中間mask textureの再入場条件

今回のvector Pathを一度だけ使う処理では、mask textureを生成して再sampleする必要はない。
次のいずれかが実在する時だけ、独立mask surface／textureを再選定する。

- 同じmaskを複数effectまたは複数contentで再利用する
- image／video alpha、別layer、lumaをmask sourceにする
- mask自体へblur／feather／expand／invertを適用する
- nested maskと明示的な合成modeを持つ

その場合もmaskはderived GPU resourceであり、Document writer、CPU pixel buffer、第二Preview／Export経路にしない。

## 5. Halftoneと3D RGBAへの適用

halftoneはgradientの色補間を置き換える単純なvertex colorではない。dot frequencyをmesh分割へ依存させず、
2D Pathと3D meshが投影済みRGBAへ合流した後の同じscreen-space filterとして扱う。座標はcomposition正規化座標、
frequencyは出力高さ当たりのcell数とするため、Preview／Exportの解像度差でcell配置を変えない。入力のlinear RGB輝度から
円dot半径を決め、dot coverageと入力alphaを乗算する。object UV、mesh tessellation、camera zoomをdot座標ownerにしない。

`M5-R0`のprivate offscreen fixtureでは、透明clear上へ投影した3D material三角形をRGBA textureに保持し、別のfullscreen
halftone passでtextureをsampleする。自動oracleは、3D silhouette外のalphaが0のまま、silhouette内にdotとholeの両方があり、
dot画素が入力RGBAを超えないことを固定する。これにより`M5-FILTERMASK-H0`を`DONE / PRIVATE PROBE`とする。
製品では同じ処理をGPU texture間で行い、CPU readbackや2D／3D別実装を作らない。本probeのreadbackはoracle専用である。

binary edgeのanti-alias、製品scene-color format、実mesh／Rerun Stage接続は未成立であり、本成立を製品runtimeへ繰り上げない。
製品接続の次edgeは既決どおり`M5-PATH2D-S1` Stage seat compileであり、本probeから別window filter runtimeを作らない。
