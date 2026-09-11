# パス効果と Create の図形 — 輪郭に掛ける物は効果の棚、置く物は Create

利用者裁定(2026-09-10): AE の shape 層で当たり前の物を 2 つに分けて持つ。
**Create は名詞**(層を 1 枚置く)。**輪郭を変える演算は効果の棚の Path 族**(パンク・膨張、ジグザグ、ねじり、オフセット、角丸、トリム)。
「Path は 7 つで閉じているのでは」と疑ったが、Lottie の外に Wiggle Paths(AE)、Roughen(Illustrator)、Smooth・Subdivide(Cavalry)が続く。増える物は棚に置く。点爆発は輪郭ではなく配置なので配置効果側。

## 編集の意味

- Create の図形は**レシピ**(源・塗りか線・最初から積む効果)。Rectangle・Rounded Rectangle・Ellipse・Star・Polygon・Line・Bezier と Null。
  角丸矩形は Rectangle + Rounded Corners 効果 1 枚で生まれる(地図の裁定「rect.r は不採用、演算子を 1 段積む糖衣」)。Line は Bezier の接線ゼロ・角 cap。星は真上起点(polystar.r 不採用と同じ)。
- パス効果は配置効果と同じ契約(`store/kind.rs` の表)。**値は層の property** なので、キー・イーズ・Undo・有効/無効・並べ替えは他の効果と同じ機構。
- 書類(`SetShapes`)は書き換えない。描画と Lottie 出力が「効果を掛けた姿」を読む(`pathop::with_effects`)。輪郭は絵より先に決まるので、**積んだ位置(配置効果の上下・raster 効果の前後)に依らず同じに効く**。
- 掛かるのは形の層だけ。他の層を選んで掛けると理由を返す。棚には形の層を選んでいる間だけ並ぶ(AE の shape 層の「追加」と同じ見え方、panel は増やさない)。
- Lottie の repeater(`OpKind::Repeater`)は棚に出さない。増やす物は配置効果 1 枚(2026-08-31/09-06 裁定)。doc の variant は Lottie 入出力の語彙として残す。

## 残余

- `OpKind::TrimPath.offset` は評価が割合(0..1)、Lottie 出力が生値。既存の食い違いで、この変更では触っていない。
- 文字の層のアウトラインにも同じ演算が掛けられる(文字は Bezier へ落ちる)。今回は形の層だけ。
- Wiggle・Roughen・Smooth は Lottie に出ない。card に「Lottie に出ない」印を持たせる場所は棚側で用意する。

## 描画の境目(利用者裁定 2026-09-10、追補)

「線は re_renderer、塗りは当面 tiny-skia」の描き分けは**棄却**。fork してまで作った境目は深く 1 本で、パスの描き手(線も塗りも)は fork の re_renderer 側に置く。Motolii は輪郭と値を渡すだけ。描き分けを許すと以後の判断が全部ぶれる。
文字・SVG・図形・マスクも同じ re_renderer の世界から生まれ、raster 効果(グロー等)もパス効果も全部掛かること。tiny-skia の raster は Motolii 側から退く。

### 実装(2026-09-10)

- fork `re_renderer` に `renderer/paths.rs` + `shader/paths.wgsl`(commit ee94be71)。三次ベジェの輪郭を lyon で三角化(fill は nonzero/evenodd、stroke は cap・join・miter、破線は path の長さで刻む)、1 本の premultiplied 三角形リストで描く。色は頂点ごと(直線 2 色の gradient は厳密、それ以外は三角形の補間)。
- Motolii は `compositor/paths.rs` の `render_paths` 1 本: 形の木を fork の builder へ積み、canvas 大の offscreen(直交 TopLeft)へ描き、`import_gpu_premultiplied` で他の層と同じ texture にする。出口が同じなので raster 効果(blur・glow・Vism)は今までどおり全部掛かる。
- 形の層と文字の層がここを通る(`engine/texture.rs`、`engine/text.rs` は文字を形の木に組むだけになった)。`engine/shape.rs`(tiny-skia)は削除。
- 残余: マスクの coverage(`engine/mask.rs`)と Colors の見本画(`ui/native/src/editor/visual_samples.rs`)はまだ CPU の tiny-skia で、GPU 読み戻しの口に付け替えるまで doc の `vector/raster.rs` が残る。gradient の radial と多段 stop は頂点補間なので粗い(専用 shader が次)。
