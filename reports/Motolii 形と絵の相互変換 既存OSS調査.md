# つなぐだけで届くのは六枚中三枚

答えは「つなぐだけでは足りない」です。ただし、足りない所は狭く、名前で言えます(2026年10月11日時点)。六枚の絵のうち、絵2(ガラスの星)、絵3(クロムの文字)、絵5(HELLOの中の別世界)は、いまある持ち主(ThorVG、Filament、OSの動画デコーダ、CPUの三角形分割ライブラリ)と、それらをつなぐ材質やシェーダだけで最後まで届きます。絵1は「シルエットが文字へ変形する」一か所だけ、持ち主が見つかりません。絵4の「溶けて複雑な形になる」と、絵6の「過去のシルエットがガラスの立体になる」は、核心で止まります。止めているものは三つです。一つ目は、絵から作った形をGPUの上で本物の立体にして、別の描画エンジンへ渡す公開の持ち主がいないこと。二つ目は、Filament がガラスの背景を1ビューにつき1回しか写し取らないこと。三つ目は、Windows で ThorVG の出力を外へ渡す口が wgpu-native に無いこと。二つ目は Bevy が、三つ目は Dawn と Rust 版 wgpu が、同じ制約をすでに解いています。ただしどれも「そのエンジンの中で」解いていて、横に置いてつなぐだけの部品にはなっていません。つまり「一方通行のルールを決めず、Motolii 専用のエンジンも作らず」は、絵4と絵6については、いまの公開ソフトだけでは成り立ちません。

この報告の読み方を先に書きます。「持ち主」とは、その仕事をすでに引き受けて出荷している公開ソフトのことです。「持ち主なし」は「探した範囲で見つからなかった」という意味で、「技術的にできない」とは別です。この二つは最後まで分けて書きます。大事な主張には証拠ラベルを付けます。「実証済み」は数値つきの公開デモやベンチマークがあるもの。「ソース確認」は調査中にコードや公式文書を自分で読んだもの。「推測」は読んだ事実からの推論。「未検証」は検索の抜粋や記憶だけのものです。この報告は「何があるか、何がかかるか、何が欠けているか」だけを書きます。作り方の提案や、エンジンの乗り換えの勧めは書きません。

## 六つの調査対象:そろっているのは「距離の地図」と「共有の口」、欠けているのは「立体化」

### [1] マスクから距離の地図へ:持ち主は七つ、単体の部品はゼロ

距離の地図(SDF)とは、各ピクセルが「形のふちまで何ピクセル離れているか」を持つ絵です。内側か外側かも符号でわかります。光るふち取り、ふくらみ、形から形への変形の材料になります。公開ソフトがこれをGPUで作る方法は、見つけた限りすべてジャンプフラッディング(JFA。全画面の処理を十回ほど重ねて「いちばん近いふち」を広げていく方法)でした。「ソース確認」 正確な方式(PBA+)は CUDA 専用で、macOS を満たせません。「実証済み」([PBA](https://www.comp.nus.edu.sg/~tants/pba.html))

速さは一度だけ数値で示されています。**1920×1080、RTX 2080 Super で、100ピクセル幅のふち取りが約0.7ミリ秒**、「フルスクリーンの1080pで1ミリ秒未満」です。「実証済み」([Ben Golus](https://bgolus.medium.com/the-quest-for-very-wide-outlines-ba82ed442cd9)) 別の計測では RTX 2060 Super で1パスあたり約0.16〜0.18ミリ秒でした。「実証済み」([Duncan Readle](https://medium.com/@duncanreadle.20/jump-flood-outlines-in-ue5-a-case-study-a7f8e9404a60)) 画面全体の距離の地図には11パスほど要るので、合計はおよそ0.8〜2.1ミリ秒になります。これは掛け算による「推測」です。Apple Silicon、内蔵GPU、ほかの描画エンジンと同時に動かした場合の数値は、どこにもありませんでした。

GPU読み戻し(GPUの上のデータをCPU側へコピーして戻すこと。再生が止まる原因になります)は、どの持ち主にもありません。「ソース確認」 履歴も乱数も使わないので、同じフレームなら同じ結果になります。Blender、Godot、bevy_mod_outline は「ソース確認」、全体としての結論は「推測」です。

欠けているものは四つあります。第一に、**「よそのGPUテクスチャを入れたら距離の地図のテクスチャが返る」単体のライブラリは、どのグラフィックスAPIにも見つかりませんでした**(持ち主なし。不可能という意味ではありません)。処理はいつもエンジンやアプリの中に住んでいます。第二に、やわらかい境界のマット(髪、ブレ、ぼかした人物の切り抜き)から、ピクセル未満の精度で距離の地図を作るGPUの持ち主はいません。あるのは「ふちのぼけが1ピクセル以内」を前提にした Sobel 方式だけで、作者自身がその限界を書いています。「ソース確認」([Golus](https://bgolus.medium.com/the-quest-for-very-wide-outlines-ba82ed442cd9)) きちんとした方式(Use.GPU の ESDT、Gustavson)はCPUのコードです。「ソース確認」([acko.net](https://acko.net/blog/subpixel-distance-transform/)) 第三に、「二つのマスクから二つの距離の地図を作り、混ぜて変形させる」機能を出荷している持ち主は見つかりませんでした(持ち主なし)。第四に、Skia、OpenCV は距離の地図をCPUで作ります。「ソース確認」 ThorVG と Vello が距離の地図を外へ出す証拠もありません(Vello は「ソース確認」、ThorVG 1.1 のヘッダとは照合していないので「未検証」)。

| 候補 | 場所(ファイル・API) | Windows / macOS | GPU読み戻し | よそのGPU資源とのつなぎ方 | ライセンス | 証拠 |
|---|---|---|---|---|---|---|
| [Blender GPU コンポジタ](https://raw.githubusercontent.com/blender/blender/main/source/blender/compositor/algorithms/intern/jump_flooding.cc) | `compositor/algorithms/intern/jump_flooding.cc`、`jump_flooding()`、シェーダ `compositor_jump_flooding` | アプリとして両対応(macOS の Metal 上での動作は未確認) | なし | 内部の `Result` 型だけ。ライブラリではない | **GPL-2.0-or-later(危険)** | 「ソース確認」 |
| [Godot 4 の 2D](https://raw.githubusercontent.com/godotengine/godot/master/servers/rendering/renderer_rd/shaders/canvas_sdf.glsl) | `renderer_rd/shaders/canvas_sdf.glsl`、`TextureStorage::render_target_sdf_process`、`texture_sdf()` | 両対応 | なし | 入力は [LightOccluder2D の多角形だけ](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/canvas_item_shader.html)。任意のテクスチャは入らない | MIT(今回は読み直していない) | 「ソース確認」 |
| [bevy_mod_outline](https://github.com/komadori/bevy_mod_outline/tree/master/src/flood) | `src/flood/sobel_init.wgsl`、`jump_flood.wgsl` | wgpu 経由で両対応 | なし | Bevy の描画の世界が必要。WGSL ファイルが Motolii の今の構成にいちばん近い(「推測」) | MIT または Apache-2.0 | 「ソース確認」 |
| [Rerun re_renderer](https://docs.rs/re_renderer/latest/src/re_renderer/draw_phases/outlines.rs.html) | `src/draw_phases/outlines.rs`、`jumpflooding_init.wgsl`、`jumpflooding_step.wgsl` | wgpu 経由で両対応 | なし | 自分のレンダラの描画段の一つ。外には出ていない | MIT / Apache-2.0 | 「ソース確認」 |
| [OPENRNDR orx-jumpflood](https://github.com/openrndr/orx/tree/master/orx-jumpflood) | `org.openrndr.extra.jumpfill` の `DistanceField`、`DirectionalField`、`SDFSmoothDifference` ほか | Kotlin / JVM | なし(結果はGPU画像) | JVM と OpenGL の実行環境が必要 | BSD-2-Clause | 「ソース確認」(README のみ) |
| [TypeGPU `@typegpu/sdf`](https://docs.swmansion.com/TypeGPU/ecosystem/typegpu-sdf) | `createJumpFlood({root, size, classify, ...})`、`sdfOutput` | ブラウザの WebGPU | なし | 「テクスチャ入力、テクスチャ出力」の形だが JavaScript の WebGPU 専用 | MIT | 「ソース確認」(文書のみ) |
| [TiXL](https://github-wiki-see.page/m/tixl3d/tixl/wiki/lib) | `Lib.image.generate.misc.JumpFloodFill`、`Lib.field.generate.sdf.Image2dSDF` | **Windows のみ**(DirectX 11) | 個別には未確認 | TiXL の中だけ | MIT | 「ソース確認」(オペレータ名)。「符号なし」という注記は「未検証」 |
| Ben Golus の Unity サンプル | [記事](https://bgolus.medium.com/the-quest-for-very-wide-outlines-ba82ed442cd9)末尾の gist | Unity | なし | Unity の中だけ | **不明** | 「実証済み」(数値) |
| PBA+ / NVIDIA NPP | [リポジトリ](https://github.com/orzzzjq/Parallel-Banding-Algorithm-plus) | NVIDIA のみ。macOS 不可 | — | CUDA | **不明** | 「実証済み」(数値) |
| Apple `MPSImageEuclideanDistanceTransform` | Metal Performance Shaders | Apple のみ | — | — | Apple 専有 | 「未検証」 |

### [2] 距離の地図から立体へ:メッシュは読み戻しなしの公開部品が無く、レイマーチは Windows 専用の親アプリの中

ここは二つに分かれます。A はメッシュ(三角形の集まり)をGPUで作る方法。B はメッシュを作らず、距離の地図を光線で少しずつたどって描く方法(レイマーチ)です。

A の結論は「成熟していて、ゆるいライセンスで、Windows と macOS で動き、描画エンジンを選ばず、読み戻しもしない」持ち主は見つからなかった、です(持ち主なし)。できないわけではありません。TouchDesigner の Trace POP は画像の明るい所をGPUでなぞって線や三角形にしますし、Polygonize POP は3Dテクスチャから閉じた面を作ります。どちらも「最大数を先に決めておく」方式で、CPUへ戻すのは任意です。「ソース確認」([Trace POP](https://docs.derivative.ca/Trace_POP)、[Polygonize POP](https://docs.derivative.ca/Polygonize_POP)) ただし非公開の商用製品です。公開側でいちばん文書がそろっている Will Usher の WebGPU 版は、**1回の抽出につき2回、数をCPUへ読み戻します**。「ソース確認」([marching_cubes.ts](https://raw.githubusercontent.com/Twinklebear/webgpu-marching-cubes/main/src/marching_cubes.ts)) 速さも **256³ で27〜54ミリ秒**(M2 Max で40.8〜54.4、RTX 3080 で31〜37.5)で、60fps の16.7ミリ秒に入りません。「実証済み」([Will Usher](https://www.willusher.io/graphics/2024/04/22/webgpu-marching-cubes/)) 読み戻しなしの公開例は keijiro の Unity 版で、三角形の上限を先に決め、余りを消す方式です。「ソース確認」([MeshBuilder.cs](https://raw.githubusercontent.com/keijiro/ComputeMarchingCubes/main/Assets/MarchingCubes/MeshBuilder.cs)) ただし Unity 専用です。2Dの距離の地図からGPUで輪郭をなぞって面にする公開ライブラリは、一つも見つかりませんでした(持ち主なし)。1080p での数値もありません。

受け取る側にも壁があります。**Filament の公開APIは、頂点データを「CPUからのコピー」としてしか受け取らず、頂点数は作成時に固定です**。GPUが書いた頂点バッファを取り込む口も、GPUが決めた数で描く口も、ヘッダにありません。「ソース確認」([VertexBuffer.h](https://raw.githubusercontent.com/google/filament/main/filament/include/filament/VertexBuffer.h)、[BufferObject.h](https://raw.githubusercontent.com/google/filament/main/filament/include/filament/BufferObject.h)) 一方、テクスチャなら入ります(Metal の `id<MTLTexture>` を `Texture::Builder::import` で。ただしヘッダ自身が「最後の手段。変更や削除がありうる」と書いています)。「ソース確認」([Texture.h](https://raw.githubusercontent.com/google/filament/main/filament/include/filament/Texture.h)) だから「ある描画エンジンが持つ場を、別の描画エンジンが形として描く」に今の持ち主で合うのは B のほうです。これは「推測」です。

B の結論は「距離の地図のレイマーチを主役の機能として出荷している公開ソフトはあるが、どれも自分の親アプリの中で描く」です。VL.Fuse と TiXL はどちらも MIT で、レイマーチと距離の地図の演算を持っています。「ソース確認」([VL.Fuse](https://github.com/TheFuseLab/VL.Fuse)、[TiXL releases](https://github.com/tixl3d/tixl/releases)) しかし VL.Fuse は vvvv gamma(商用)の中、TiXL は Windows と DirectX 11 の中です。**「屈折するレイマーチの立体を、別の描画エンジンの場面に奥行きつきで重ねる」再利用できる部品は見つかりませんでした**(持ち主なし)。技術としては普通です。Unity 用の uRaymarching はレイマーチの当たり位置から奥行きを書き込みます(「未検証」)。レイマーチの屈折は、ばらばらのシェーダ例と TouchDesigner 用ツールの旗一つがあるだけです(「未検証」)。

| 候補 | 種類 | 場所(ファイル・API) | Windows / macOS | GPU読み戻し | よそのGPU資源とのつなぎ方 | ライセンス | 証拠 |
|---|---|---|---|---|---|---|---|
| [webgpu-marching-cubes](https://github.com/Twinklebear/webgpu-marching-cubes) | A・3D | `src/marching_cubes.ts`:`computeActiveVoxels`、`computeVertexOffsets`、`computeVertices` | WebGPU なら両対応 | **あり(1回の抽出で2回)** | 自分の WebGPU デバイスの中だけ | MIT | 「実証済み」 |
| [keijiro/ComputeMarchingCubes](https://github.com/keijiro/ComputeMarchingCubes) | A・3D | `Assets/MarchingCubes/MeshBuilder.cs` | Unity 経由 | なし(上限固定) | Unity のメッシュのバッファへ直接書く。Unity 専用 | Unlicense | 「ソース確認」 |
| TouchDesigner [Trace POP](https://docs.derivative.ca/Trace_POP) / [Polygonize POP](https://docs.derivative.ca/Polygonize_POP) | A・2D / 3D | POP のパラメータ(Max Num Verts、Fraction of Max Allocation) | ページに記載なし | なし(CPUへの写しは任意。すると止まる) | 自分のレンダラの中 | **商用・非公開** | 「ソース確認」(文書) |
| Notch [Procedural Meshing](https://manual.notch.one/2026.2/en/docs/reference/nodes/procedurals/render-nodes/procedural-meshing/) | A・3D | ノード | Windows のみ | 未確認 | 自分の中 | **商用・非公開** | 「未検証」 |
| [VL.Stride.SDFToMesh](https://github.com/tebjan/VL.Stride.SDFToMesh) | A・3D | `shaders/` | vvvv gamma の中 | 未記載 | Stride の中 | **種類を読めず(不明)** | 「ソース確認」 |
| [wgpu のメッシュシェーダ](https://wgpu.rs/doc/wgpu/documentation/extensions/mesh_shading/index.html) | A の土台 | `Features::EXPERIMENTAL_MESH_SHADER` | Vulkan は完全、Metal と DX12 は素通しのみ。実験的 | なし | — | MIT / Apache(今回は読み直していない) | 「ソース確認」 |
| [fast-surface-nets](https://github.com/bonsairobo/fast-surface-nets-rs) | A だが CPU | `surface_nets` | 両対応 | 対象外(CPU) | CPUからアップロード | MIT または Apache-2.0 | 「実証済み」(1コアで毎秒約2000万三角形) |
| [vtracer](https://github.com/visioncortex/vtracer) | 2Dなぞり・CPU | — | 両対応 | GPUの絵に使うなら読み戻しが要る | CPU | MIT | 「ソース確認」 |
| [VL.Fuse](https://github.com/TheFuseLab/VL.Fuse) | B | 「Distance Fields & Raymarching」ノード群 | vvvv gamma の中(Windows は「推測」) | 自分の中ではなし | Stride の中 | MIT(親の vvvv は商用) | 「ソース確認」 |
| [TiXL](https://github.com/tixl3d/tixl) | B | `[RaymarchField]`、`[BlendSDFWithSDF]`、`[SdfToColor]` | **Windows のみ**(DirectX 11) | 自分の中ではなし | 自分の中 | MIT | 「ソース確認」 |
| [RayTK](https://github.com/t3kt/raytk) | B | TouchDesigner 用の部品集 | TouchDesigner の中 | — | TOP 画像として出る | **CC BY 4.0(表示義務)** | 「ソース確認」 |
| [bevy_march](https://github.com/NiseVoid/bevy_march) | B | — | wgpu 経由 | なし | Bevy の中。奥行きの共有は未記載 | MIT または Apache-2.0 | 「ソース確認」 |
| three.js [`RaymarchingBox`](https://raw.githubusercontent.com/mrdoob/three.js/dev/examples/jsm/tsl/utils/Raymarching.js) | B の補助 | `RaymarchingBox(steps, callback)` | ブラウザ | なし | 奥行きを読みも書きもしない | MIT(今回は読み直していない) | 「ソース確認」 |
| [hg_sdf](https://mercury.sexy/hg_sdf/) | B の数式 | GLSL の関数集 | シェーダ文なのでどこでも | — | ただのシェーダ文 | MIT **または CC-BY-NC-4.0**(MIT を選ぶこと) | 「ソース確認」 |

### [3] 動く輪郭の押し出し:公開の持ち主は全員CPU、ただし星ひとつは数十マイクロ秒

押し出しとは、平らな形に厚みをつけて立体にすることです。確認できた公開の持ち主(three.js、Godot、Blender、Manifold、fontmesh、meshtext、Unity Splines、Unreal Text3D)は、**全員CPUで立体を組み立て、輪郭が変わったら「作り直し」**です。「ソース確認」 自分の文書で「毎フレーム用ではない」と言っているものもあります。three.js のフォーラムは「THREE の押し出しはリアルタイム用に作られていない」、Godot は CSG に「大きなCPUコスト」、Unity は「1秒あたりの作り直し回数の上限」を持っています。「ソース確認」([three.js forum](https://discourse.threejs.org/t/changing-shapes-geometries-on-every-frame-solved/34769)、[Godot CSGPolygon3D](https://docs.godotengine.org/en/stable/classes/class_csgpolygon3d.html)、[Unity SplineExtrude](https://docs.unity3d.com/Packages/com.unity.splines@2.5/api/UnityEngine.Splines.SplineExtrude.html)) 輪郭から三角形分割、押し出しまでをGPUに置いている出荷品は TouchDesigner の POP だけで、非公開です。その Text POP ですら「仕事のほとんどをCPUでする」と書いています。「ソース確認」([Extrude POP](https://docs.derivative.ca/Extrude_POP)、[Triangulate POP](https://docs.derivative.ca/Triangulate_POP)、[Text POP](https://docs.derivative.ca/Text_POP))

では、CPUの作り直しは60fpsの壁でしょうか。正直な数値はこうです。三角形分割(ふた部分を三角形に切り分ける処理)の定番 earcut の公開ベンチマークでは、**穴つき104点の形で毎秒28,319回、つまり1回およそ35マイクロ秒**です。15点なら約1.3マイクロ秒、94点なら約28マイクロ秒です。「実証済み」(回数。マイクロ秒への換算は割り算による「推測」)([earcut v2.2.4](https://github.com/mapbox/earcut/blob/v2.2.4/README.md)) これは2015年ごろの JavaScript の数字です。重くなるのは点が数千のときで、**2,523点で約1.84ミリ秒、5,667点で約10.5ミリ秒**です。新しい C++ 版は、119,680個の多角形(190万頂点)を M1 Pro で約276ミリ秒、1頂点あたり約145ナノ秒で処理します。「実証済み」([earcut.hpp](https://github.com/mapbox/earcut.hpp)) つまり星、文字、単純なロゴ(数十〜数百点)は1フレーム16.7ミリ秒の1%未満です。画像をなぞった輪郭や、長文を一つの形にした場合のような数千点では、目に見える割合になります。これは「推測」です。

取れなかった数値もはっきり書きます。Lyon、`Manifold::Extrude`、Clipper2 の絶対時間は公開されていません。壁づくり、面取り、法線、GPUへの転送の時間もどこにもありません。「動く文字を毎フレーム押し出して描く」全体のベンチマークは、どのプロジェクトにもありませんでした。

GPUだけでやる道の持ち主はどうか。「2Dの道をCPUの三角形分割なしで塗る」成熟した持ち主はいます(Rive Renderer、Vello、Slug、Skia Graphite)。Slug は2026年3月17日に特許が公有に置かれ、参考シェーダが MIT で公開されました。「ソース確認」([A Decade of Slug](https://terathon.com/blog/decade-slug.html)) ただし彼らが出すのはピクセルで、三角形ではありません。立体の「ふた」を、塗りつぶしの絵として供給できるだけです。「推測」 **任意の動く輪郭をGPUで押し出す公開の持ち主、穴つき輪郭のGPU三角形分割ライブラリ、動く輪郭のGPU面取りは、どれも見つかりませんでした**(持ち主なし。TouchDesigner が出荷しているので不可能ではありません)。面取りはCPUでもいちばんもろい部分です。three.js は自己交差の処理を持たず、Notch は文字の面取りを一時取り下げました。「ソース確認」([ExtrudeGeometry.js](https://github.com/mrdoob/three.js/blob/dev/src/geometries/ExtrudeGeometry.js)、[Notch forum](https://forum.notch.one/t/text-bevel-parameter-missing/1540)) 輪郭が動くときに、ふたの三角形の割り方がフレームごとに変わってちらつくかどうかは、誰も測っていません。

| 候補 | 場所(ファイル・API) | 仕事をする所 | Windows / macOS | GPU読み戻し | よそのGPU資源とのつなぎ方 | ライセンス | 証拠 |
|---|---|---|---|---|---|---|---|
| [mapbox/earcut.hpp](https://github.com/mapbox/earcut.hpp) | `mapbox::earcut<N>(polygon)`。最初の輪が外側、残りが穴 | CPU | 両対応 | なし(CPUからGPUへの片道) | 頂点を作って転送 | ISC | 「実証済み」 |
| [nical/lyon](https://github.com/nical/lyon) | `FillTessellator::tessellate_path`、`VertexBuffers` | CPU | 両対応(Rust) | なし | 頂点と添字のバッファを作る | MIT / Apache-2.0 | 「ソース確認」(数値なし) |
| [elalish/manifold](https://manifoldcad.org/docs/html/classmanifold_1_1_manifold.html) | `Manifold::Extrude`、`Revolve`、`CalculateNormals`、`GetMeshGL` | CPU | 両対応 | なし | `GetMeshGL` が「レンダラへ押し込みやすい」形 | Apache-2.0 | 「ソース確認」(数値なし) |
| [Clipper2](https://angusj.com/clipper2/Docs/Overview.htm) | `InflatePaths`、`ClipperOffset` | CPU | 両対応 | なし | 面取り用の輪を作る | 「商用でも無料」(本文未読。Boost は記憶のみ) | 「ソース確認」 |
| [fontmesh](https://docs.rs/fontmesh) / [meshtext](https://github.com/FrankenApps/meshtext) | `glyph_to_mesh_3d`、`extrude` | CPU | 両対応(Rust) | なし | 頂点を作って転送 | MIT または Apache-2.0 | 「ソース確認」 |
| [CGAL `extrude_skeleton`](https://doc.cgal.org/6.1/Straight_skeleton_2/group__PkgStraightSkeleton2Extrusion.html) | `<CGAL/extrude_skeleton.h>` | CPU(正確な計算は「とても遅い」) | 両対応 | なし | — | **未確認。GPL の可能性(危険)** | 「ソース確認」(ライセンスは「未検証」) |
| [libtess2](https://github.com/memononen/libtess2) | GLU 由来 | CPU | 両対応 | なし | — | SGI Free Software License B 2.0(珍しい。要確認) | 「ソース確認」 |
| TouchDesigner [Extrude POP](https://docs.derivative.ca/Extrude_POP) / [Triangulate POP](https://docs.derivative.ca/Triangulate_POP) | POP | GPU(コンピュートシェーダ) | macOS での POP は未記載 | 形の情報をCPUへ写すと止まる | 自分の中 | **商用・非公開** | 「ソース確認」 |
| [Rive Renderer](https://rive.app/docs/runtimes/cpp/renderers.md) | `rive::gpu::RenderContext`、D3D11 の `setTargetTexture` | GPU | Metal、Vulkan、D3D11、D3D12、GL | なし | 呼び手のテクスチャへ描ける(D3D11 と GL は明記。Metal と Vulkan の描き先は未確認) | MIT | 「ソース確認」 |
| [Vello](https://docs.rs/vello/latest/vello/struct.Renderer.html) | `Renderer::render_to_texture` | GPU(wgpu) | wgpu が動く所 | なし | 同じ wgpu デバイスのテクスチャへ描く | Apache-2.0 / MIT | 「ソース確認」(本体は自称アルファ) |
| [Slug 参考シェーダ](https://terathon.com/blog/decade-slug.html) | `EricLengyel/Slug` | GPU(曲線データはCPUで用意) | シェーダ文 | なし | 曲線と帯のテクスチャを書き換える | MIT。特許は公有 | 「ソース確認」 |
| [NV_path_rendering](https://developer.download.nvidia.com/assets/gamedev/files/GL_NV_path_rendering.txt) | OpenGL 拡張 | GPU | **macOS 不可** | なし | — | NVIDIA の拡張 | 「ソース確認」 |

Filament への渡し方は二つ文書化されています。CPUで作った頂点を `setBufferAt` でコピーする道(非同期版あり)と、材質の頂点段でカスタム属性8本と頂点段のテクスチャから位置を動かす道です。「ソース確認」([VertexBuffer.h](https://raw.githubusercontent.com/google/filament/main/filament/include/filament/VertexBuffer.h)、[Materials guide](https://google.github.io/filament/Materials.md.html)) どちらもCPUからGPUへの片道で、読み戻しはありません。毎フレームの頂点更新で Filament が止まるかどうかの計測は見つかりませんでした。

### [4] 2Dと3Dをまたぐ合成:全部できる道具は非公開、公開側で出入り口がそろうのは Skia だけ

**リアルタイムのノード系ツール。** 画像、形、点、場を対等に扱い、GPUの上で自由に変換し合うのは三つだけです。TouchDesigner(2025年の POP 群から。商用)、Notch(商用、Windows のみ)、TiXL(MIT、ただし今は Windows と DirectX 11)です。「ソース確認」([Learning About POPs](https://docs.derivative.ca/Learning_About_POPs)、[TiXL](https://github.com/tixl3d/tixl)) 三つとも、すべてを一つのGPUデバイス(GPUを使うための窓口。別々の窓口で作った絵は、そのままでは貸し借りできません)で動かしています。TouchDesigner は Vulkan 一本で、macOS では MoltenVK を通します。「ソース確認」([2022 release notes](https://derivative.ca/release/202224200/66252)) Motolii と同じ「2Dベクターの描画エンジンと別の3Dエンジンを貼り合わせる」例は vvvv gamma(Skia と Stride)だけ見つかりました。最初は別々のデバイスで、間をCPUへのダウンロード(「かなり高い」)か2〜3フレームの遅れでつなぎ、最後は Skia を Stride と同じ Direct3D 11 デバイスの上で動かす形に落ち着いたと、フォーラムは伝えています。これは検索抜粋だけなので「未検証」です。([vvvv forum](https://forum.vvvv.org/t/skiarenderer-not-working-since-3-0/18810/33)) 時間の扱いにも共通点があります。残像や粒子は「前のフレームを入れて次を出す」積み重ねで、好きな時刻へ飛んで同じ結果を得る機能を文書化した道具はありませんでした(持ち主なし)。

**エンジン。** 六枚の絵を全部、備えつけの機能だけでまかなうエンジンはありません。部品として近いのは three.js と drei です。`MeshPortalMaterial`(別の場面への窓)、`Mask`(ステンシルで切り抜く)、`MeshTransmissionMaterial`(ほかのガラスが見えるガラス)がそろっています。「ソース確認」([MeshPortalMaterial](https://drei.docs.pmnd.rs/portals/mesh-portal-material)、[Mask](https://drei.docs.pmnd.rs/portals/mask)、[MeshTransmissionMaterial](https://drei.docs.pmnd.rs/shaders/mesh-transmission-material)) ただしブラウザの中です。Godot 4.5以降は入れ子のビューポート、2Dの切り抜き、実験的な3Dステンシルを一つのデバイスに持ちます。けれどもリアルタイムのベクター描画が無く(Lottie は絵に焼いて取り込む案が未マージ)、内蔵動画は Theora だけで、ガラスはほかの透明物を屈折できません。「ソース確認」([Standard Material 3D](https://docs.godotengine.org/en/stable/tutorials/3d/standard_material_3d.html)、[godot PR #91580](https://github.com/godotengine/godot/pull/91580)) Bevy と bevy_vello は、ベクター、文字、3D、重なるガラスを wgpu 一つに持ちます。しかし SVG と Lottie は「限定的」、ステンシルは未出荷、描画の骨組みが2026年6月に入れ替わりました。「ソース確認」([bevy_vello README](https://raw.githubusercontent.com/linebender/bevy_vello/main/README.md)、[Bevy 0.19](https://bevy.org/news/bevy-0-19/)) Flutter に入れて出荷された実績があるのは Filament だけです(Thermion。Toyota の Fluorite は報道のみで「未検証」)。エンジンを一つにしても、エンジンから Flutter への受け渡しと、動画デコーダからエンジンへの受け渡しは残ります。「推測」

| 候補 | 場所(API) | Windows / macOS | GPU読み戻し | よそのGPU資源とのつなぎ方 | ライセンス | 証拠 |
|---|---|---|---|---|---|---|
| TouchDesigner | [TOP to POP](https://docs.derivative.ca/TOP_to_POP)、[Trace POP](https://docs.derivative.ca/Trace_POP)、[Render TOP](https://docs.derivative.ca/Render_TOP) | 両対応(Vulkan / MoltenVK) | TOP と POP の中だけならなし。SOP、CHOP、DAT へ行くとあり | ほかのアプリとは Syphon / Spout だけ | **商用・非公開** | 「ソース確認」(「読み戻しなし」は「推測」) |
| Notch | [Image generator](https://manual.notch.one/2026.1/en/docs/reference/nodes/procedurals/generators/image/)、[Render To Texture](https://manual.notch.one/2026.1/en/docs/reference/nodes/2d/render-to-texture/) | **Windows のみ** | 未記載 | 未記載 | **商用・非公開** | 「ソース確認」(一部「未検証」) |
| [TiXL](https://github.com/tixl3d/tixl) | `PointsOnImage`、`TextureDisplaceMesh`、`RenderTarget`、`Image2dSDF` | **Windows のみ** | ほぼなし(「詰め直しには読み戻しが要る」) | 文書なし | MIT | 「ソース確認」 |
| VL.Fuse と Stride | [VL.Fuse](https://github.com/TheFuseLab/VL.Fuse)、`SkiaTexture`、`SceneTexture` | 実質 Windows | 同じ D3D11 デバイスならなし | Stride のデバイスを共有 | Fuse と Stride は MIT、**VL.StandardLibs は LGPL-3.0、vvvv 本体は商用** | 「ソース確認」(つなぎの話は「未検証」) |
| [ISF](https://github.com/mrRay/ISF_Spec) | GLSL と JSON の画像効果の形式 | 親しだいで両対応 | なし | 設計上、親がテクスチャとデバイスを渡す | 仕様は MIT | 「ソース確認」(画像効果だけ) |
| Godot 4.x | [SubViewport](https://docs.godotengine.org/en/stable/tutorials/rendering/viewports.html)、`clip_children`、[LibGodot](https://godotengine.org/releases/4.6/) | 両対応 | 親への渡し方は公式に未記載 | `texture_create_from_extension`(VkImage)、`get_driver_resource` | MIT(未読) | 「ソース確認」 |
| Bevy と bevy_vello | [transmission](https://bevy.org/news/bevy-0-12/)、[bevy_vello](https://raw.githubusercontent.com/linebender/bevy_vello/main/README.md) | wgpu 経由 | なし | 外の wgpu デバイスとの共有は未調査 | MIT / Apache-2.0 | 「ソース確認」 |
| three.js と drei | 上記の三部品 | ブラウザ | 画面表示はなし。書き出しはキャンバスの読み戻し | ネイティブのテクスチャ共有の道は不明 | MIT(未読) | 「ソース確認」 |
| Babylon Native | [ExternalTexture プラグイン](https://github.com/BabylonJS/BabylonNative/tree/master/Plugins/ExternalTexture) | D3D11 / Metal(D3D12 と Vulkan は開発中) | 未確認 | プラグインで親のテクスチャを渡す | Apache-2.0(未読) | 「ソース確認」(ソースのみの公開プレビュー) |
| Filament | [View.h](https://raw.githubusercontent.com/google/filament/main/filament/include/filament/View.h) の `setRenderTarget`、ステンシル、[屈折の材質](https://google.github.io/filament/main/materials.html) | Metal(macOS)、Vulkan・OpenGL・WebGPU(Windows)。**Direct3D なし** | なし | [6] を参照 | Apache-2.0 | 「ソース確認」 |
| Blender EEVEE | — | 両対応 | — | ライブラリとして使う形は見つからず | **GPL(危険)** | 「未検証」 |

**合成の持ち主になれるのは誰か。** 「よそのテクスチャを入力にとり、よそのテクスチャへ描く」口を、公開ヘッダで両方そろえている公開ソフトは三つでした。いちばんそろっているのは **Skia Graphite** です。`SkImages::WrapTexture`(よそのテクスチャを画像にする)、`SkSurfaces::WrapBackendTexture`(よそのテクスチャへ描く)、`SkImages::TextureFromYUVATextures`(動画デコーダの出す面ごとのテクスチャを受ける)があります。「ソース確認」([Image.h](https://github.com/google/skia/blob/main/include/gpu/graphite/Image.h)、[Surface.h](https://github.com/google/skia/blob/main/include/gpu/graphite/Surface.h)) マットの語彙も豊かで(ずらしマップ、ぼかし、影、膨張と収縮、アルファを高さにした照明、任意のシェーダ)、Lottie 再生の Skottie はアルファ、反転アルファ、輝度、反転輝度のマットを持ちます。「ソース確認」([SkImageFilters.h](https://github.com/google/skia/blob/main/include/effects/SkImageFilters.h)、[Skottie Layer.cpp](https://github.com/google/skia/blob/main/modules/skottie/src/Layer.cpp)) 代わりに、Windows の Graphite は Dawn 経由になる見込みで(「未検証」)、Skia を「動画と2Dと3Dエンジン出力をまたぐ合成役」として出荷したエディタは見つかりませんでした。二つ目は **Vello** です。`Renderer::register_texture` で wgpu のテクスチャを受けますが、**毎フレーム自分のアトラスへGPUからGPUへコピー**し、形式は `Rgba8Unorm` だけです。小数の奥行きや HDR は入りません。任意の絵へのぼかしもありません。「ソース確認」([vello Renderer](https://docs.rs/vello/latest/vello/struct.Renderer.html)、[vello Scene](https://docs.rs/vello/latest/vello/struct.Scene.html)) 三つ目は **libplacebo** です。`pl_vulkan_wrap` と `pl_d3d11_wrap` を持ちますが、LGPL で、本流に Metal がなく、マットの語彙も見当たりません。「ソース確認」([vulkan.h](https://github.com/haasn/libplacebo/blob/master/src/include/libplacebo/vulkan.h)、[d3d11.h](https://github.com/haasn/libplacebo/blob/master/src/include/libplacebo/d3d11.h))

いま Motolii が使っている **ThorVG は、GPUテクスチャを絵の材料として受け取れません**。`Picture::load` は「ファイル」「符号化データ」「CPU上の生ピクセル」の三つだけです。出力は `WgCanvas::target` で `WGPUTexture` へ書けます。「ソース確認」([thorvg.h](https://github.com/thorvg/thorvg/blob/main/inc/thorvg.h)) つまり ThorVG は合成役への「入力」にはなれますが、よそのテクスチャを混ぜる「持ち主」にはなれません。マットとブレンドの語彙(Alpha、InvAlpha、Luma、InvLuma、Add、Subtract、Intersect ほか)は豊かですが、相手は自分の絵だけです。上流には動画資源を少ないコピーで受ける提案(#4635)がありますが、出荷はされていません。「ソース確認」(抜粋)([thorvg #4635](https://github.com/thorvg/thorvg/issues/4635)) Filament は、独自の後処理を公開APIにしていません。「ソース確認」([filament #1818](https://github.com/google/filament/issues/1818)) OpenFX の標準のGPU経路は GL テクスチャか Metal の「バッファ」で、D3D と Vulkan は標準にありません。「ソース確認」([OpenFX Rendering](https://openfx.readthedocs.io/en/main/Reference/ofxRendering.html)) 1920×1080、60fps、複数レイヤー、マットとぼかし、外部テクスチャという条件に合う公開ベンチマークは、どの候補にもありませんでした。

**映画の合成ソフトの考え方。** Nuke は1本の流れに最大1023本の名前つきチャンネル(奥行き、動きベクトル、マスク)を持たせ、Fusion は決まった追加チャンネル(Z、被覆、物体ID、法線、ワールド位置)を運び、Cryptomatte は「IDと被覆」の組を順位つきで持って、ブレや透明でもマットが崩れないようにします。どれも検索抜粋による「ソース確認」です。([Nuke Channels](https://learn.foundry.com/nuke/content/comp_environment/channels/channels.html)、[Psyop/Cryptomatte](https://github.com/Psyop/Cryptomatte)) 成熟した道具は「特別なマットの種類を増やす」のではなく、「絵と一緒に名前つきの追加チャンネルを運び、どれをマットに読むか選ぶ」形で解いています。「推測」 今回の2Dの候補に、奥行きや法線をマット入力として扱う考え方を持つものはありませんでした。「推測」

**動画デコード。** GPUテクスチャへのデコードの持ち主はOSです。macOS は VideoToolbox、Windows は D3D11 / D3D12 のビデオ機能で、公開の入り口は FFmpeg の hwaccel か GStreamer です。検索抜粋による「ソース確認」です。Vulkan Video は MoltenVK に無いので macOS の道になりません。「ソース確認」(抜粋)([mpv #11739](https://github.com/mpv-player/mpv/issues/11739)) デコード役と合成役は、OSごとに同じグラフィックスAPIにいないとコピーなしになりません。GStreamer の保守者は、D3D11 のデコーダを GL へ渡すと「GPUからCPU、またGPUへのコピーになる」と述べています。「ソース確認」(抜粋)([GStreamer Discourse](https://discourse.gstreamer.org/t/graphics-api-interoperability-and-zero-copy-memory/464)) 両OSで Flutter のテクスチャまで動画を届けている出荷例は media_kit(libmpv を包む MIT の層)です。「実証済み」([media_kit README](https://github.com/media-kit/media-kit/blob/main/README.md)) ただし libmpv の公開描画APIは OpenGL だけです。アルファつき動画は、HEVC アルファが Apple の道、VP9 アルファは libvpx か特定のハードウェア、HAP はGPU圧縮テクスチャそのものを運ぶコーデックです。「ソース確認」(抜粋)

### [5] 重なるガラス:Filament は1層、Bevy は写し直しでN層

Filament は、不透明な物を描いたあと、屈折する物を描く前に、色の絵を**1ビューにつき1回だけ**写し取ります。それをぼかしの段階つきにして、すべてのガラスが同じ1枚を見ます。だから、ガラスの中に別のガラスは映りません。これはコード(`RendererUtils::refractionPass`)を読んで確かめました。「ソース確認」([RendererUtils.cpp](https://github.com/google/filament/blob/99e469339adddbfe3200b719c9a3e60f4a504f6f/filament/src/RendererUtils.cpp)) 写し取りはGPUの中のコピーで、読み戻しはありません。屈折に乱数も履歴も使わないので、同じフレームなら同じ絵です(シェーダは「ソース確認」、結論は「推測」)。厚み、吸収、分散(色の分かれ)の設定はそろっています。ただし立体の屈折は「厚みを半径とする球」とみなす近似で、分散は立体モードだけです。「ソース確認」([Materials guide](https://google.github.io/filament/main/materials.html)) ハードウェアの光線追跡は Filament にありません。「ソース確認」(検索で該当なし)

1層より先へ行っている公開の出荷品は三つありました。**Bevy** は、屈折する物を奥から手前へN組に分け、組ごとに画面を写し直します(`ScreenSpaceTransmission.steps`、既定は1。1段につきテクスチャコピー1回)。「ソース確認」([node.rs](https://github.com/bevyengine/bevy/blob/68c80f3b19ddf9e4ae1ce7dd8b5a68273204c2e8/crates/bevy_pbr/src/transmission/node.rs)) **drei** は、ガラスの材質一つにつき場面全体を1回か2回描き直します。「ソース確認」([MeshTransmissionMaterial.tsx](https://github.com/pmndrs/drei/blob/master/src/core/MeshTransmissionMaterial.tsx)) **Babylon.js** は、物ごとの映り込み用の絵を屈折の元に使えば、奥のガラスが手前のガラスの中で曲がって見えます。これは公開された画像を見て確かめました。「ソース確認」([Babylon forum](https://forum.babylonjs.com/t/is-it-possible-to-have-refractive-transmissive-objects-visible-through-other-transmissive-objects-without-using-alpha-transparency/36110)) どれも「奥のガラスを、もう平らになった絵として曲げる」方式で、光線を二つの立体の中まで追うわけではありません。「推測」 本当の入れ子は Unreal と Unity HDRP のハードウェア光線追跡だけで、どちらも商用です。「ソース確認」

**重ねた屈折の1920×1080でのフレーム時間を公表しているエンジンは、一つもありません。** 唯一の目安は Unity の担当者の発言「色のピラミッドをもう1回作ると PS4 の 1080p で約2ミリ秒」です。「ソース確認」([Unity forum](https://discussions.unity.com/t/hdrp-transparent-material-with-distortion-causes-foreground-objects-to-be-distorted/723330)) Filament の横に置いて重なるガラスを受け持てるライブラリは見つかりませんでした(持ち主なし)。Bevy の仕組みは `bevy_pbr` の一部で、単体の部品ではありません。「ソース確認」 Filament の上流に、多層の屈折の計画や議論も見つかりませんでした。

| 実装 | 仕組み(場所) | 重なるガラス | Windows / macOS | GPU読み戻し | よそのGPU資源とのつなぎ方 | ライセンス | 証拠 |
|---|---|---|---|---|---|---|---|
| Filament `refractionMode: screenspace` | 1ビューに写し取り1回([RendererUtils.cpp](https://github.com/google/filament/blob/99e469339adddbfe3200b719c9a3e60f4a504f6f/filament/src/RendererUtils.cpp)) | **不可** | 両対応 | なし | [6] を参照 | Apache-2.0 | 「ソース確認」 |
| Bevy `ScreenSpaceTransmission.steps` | 奥から手前へN組、組ごとに画面コピー([node.rs](https://github.com/bevyengine/bevy/blob/68c80f3b19ddf9e4ae1ce7dd8b5a68273204c2e8/crates/bevy_pbr/src/transmission/node.rs)) | 可(`steps` まで) | wgpu で両対応 | なし | Bevy の描画段の中。外の wgpu デバイスとの共有は「未検証」 | MIT または Apache-2.0 | 「ソース確認」 |
| drei `MeshTransmissionMaterial` | 材質ごとに場面を1〜2回描き直し | 可 | ブラウザのみ | なし(「推測」) | three.js と React の中 | MIT | 「ソース確認」 |
| three.js 本体の transmission | 不透明だけ写し取り1回 | 不可 | ブラウザのみ | なし | — | MIT | 「ソース確認」 |
| Babylon の glTF 補助 / 物ごとのプローブ | 不透明だけ1枚 / 物ごとの立方体の絵 | 不可 / 可 | ブラウザ、Babylon Native | なし | — | Apache-2.0 | 「ソース確認」 |
| Godot 4 | 不透明のあとに画面コピー1回 | 不可 | 両対応 | なし | — | MIT | 「ソース確認」 |
| Unity HDRP / Unreal の光線追跡 | ハードウェア光線追跡、跳ね返り上限つき | 可 | macOS は未確認 | なし | 組み込み不可 | **商用** | 「ソース確認」 |
| Blender EEVEE | 画面内の追跡 | 「屈折1回だけ正しい」 | 両対応 | — | — | **GPL(危険)** | 「未検証」 |
| wgpu の光線問い合わせ | `EXPERIMENTAL_RAY_QUERY`([CHANGELOG](https://github.com/gfx-rs/wgpu/blob/v30.0.1/CHANGELOG.md)) | 土台のみ。ガラスの描画役ではない | Metal、Vulkan、DX12(実験的) | なし | — | MIT / Apache | 「ソース確認」 |

### [6] GPU資源の共有:macOS は一か所で出会い、Windows は口が二つ足りない

**macOS** では、すべての部品がコピーなしの口をすでに出荷していて、同じ物で出会います。システム既定の `MTLDevice` の上の `id<MTLTexture>` か、IOSurface つきの `CVPixelBuffer` です。ThorVG の出力は `wgpuTextureGetNativeMetalTexture` で取り出せます。Rust 版 wgpu は `Device::create_texture_from_hal` で包めます。Filament は `Texture::Builder::import` で取り込めます。動画は面ごとの `MTLTexture` になります。Flutter は `CVPixelBuffer` を受けます。それぞれの口は「ソース確認」です(ThorVG から Rust 版 wgpu への道は、Motolii の手元で確認済みと調査メモにあります)。([wgpu-native wgpu.h](https://github.com/gfx-rs/wgpu-native/blob/trunk/ffi/wgpu.h)、[wgpu Device](https://docs.rs/wgpu/latest/wgpu/struct.Device.html)、[Texture.h](https://github.com/google/filament/blob/main/filament/include/filament/Texture.h)、[Flutter の Metal 外部テクスチャ](https://github.com/flutter/flutter/blob/master/engine/src/flutter/shell/platform/darwin/graphics/FlutterDarwinExternalTextureMetal.mm)) Filament から Flutter へは Thermion が、動画から Flutter へは media_kit が出荷済みです。「実証済み」([Thermion ARCHITECTURE.md](https://github.com/nmfisher/thermion/blob/master/thermion_flutter/thermion_flutter/ARCHITECTURE.md)) ただし「ThorVG の出力を Filament へ」と「三つを同時につなぐ」組み合わせを出荷した公開プロジェクトは見つかりませんでした(持ち主なし)。弱い所は二つです。Filament は呼び手から `MTLDevice` を受け取れません(GPUが複数ある Intel Mac では食い違いうる)。Flutter は同期を一切しません。「ソース確認」([PlatformMetal.mm](https://github.com/google/filament/blob/main/filament/backend/src/metal/PlatformMetal.mm)) 描く順番を止まらずにそろえる道具(`MTLEvent`、`MTLSharedEvent`)は Apple にありますが、これら三つをつないで使っている例はなく、時間の計測もありません。

**Windows** では、口は部品ごとにありますが、一か所で出会いません。Flutter が受けるのは **D3D11 のテクスチャか DXGI 共有ハンドルだけ**です。「ソース確認」([flutter_texture_registrar.h](https://github.com/flutter/flutter/blob/master/engine/src/flutter/shell/platform/common/public/flutter_texture_registrar.h)) Filament の Vulkan 版は `Texture::Builder::import` が**未対応**で、コードは「Not supported in Vulkan backend」と書いています。「ソース確認」([VulkanDriver.cpp](https://github.com/google/filament/blob/main/filament/backend/src/vulkan/VulkanDriver.cpp)) 代わりの口は `VulkanPlatform` を継いで外部画像と描き先を差し込む道で、これは Thermion が出荷しています(D3D11 の共有テクスチャを Vulkan に取り込み、ハンドルを Flutter へ渡す)。「実証済み」([WindowsVulkanTexture.cpp](https://github.com/nmfisher/thermion/blob/master/thermion_dart/native/src/vulkan/windows/WindowsVulkanTexture.cpp)) Rust 版 wgpu には DX12 と Vulkan の両方に生のテクスチャを包む口があります。「ソース確認」([wgpu-hal dx12](https://github.com/gfx-rs/wgpu/blob/trunk/wgpu-hal/src/dx12/device.rs)) そして**いちばん大きな穴は、ThorVG の WebGPU エンジンがリンクしている wgpu-native のC APIです。Metal 用の取り出し口が三つあるだけで、Vulkan と D3D12 には取り出し口も取り込み口もありません**。「ソース確認」([wgpu-native wgpu.h](https://github.com/gfx-rs/wgpu-native/blob/trunk/ffi/wgpu.h)) これは「口が無い」のであって「不可能」ではありません。下の Rust 層には `as_hal` があります。「推測」 取り込みの要望は2026年7月に出ていて、未解決です。報告者の計測では、共有テクスチャが約27マイクロ秒/フレーム、読み戻して再転送が約114マイクロ秒/フレームでした(自己申告、解像度不明の弱い「実証済み」)。([wgpu-native #621](https://github.com/gfx-rs/wgpu-native/issues/621))

Dawn(Chrome の WebGPU 実装)は、この種の共有をいちばん完成した形で持っています。`SharedTextureMemory` が IOSurface、DXGI 共有ハンドル、D3D11 テクスチャを一つの考え方で扱い、使い始めと使い終わりに同期の柵(フェンス)を運びます。Chromium が出荷しています。「実証済み」([shared_texture_memory.md](https://dawn.googlesource.com/dawn/+/refs/heads/main/docs/dawn/features/shared_texture_memory.md)) しかし ThorVG は wgpu-native でしかビルドされません。「ソース確認」([wg/meson.build](https://github.com/thorvg/thorvg/blob/main/src/renderer/gpu_engine/wg/meson.build)) Filament の WebGPU 版は Dawn の上にありますが、テクスチャの取り込みも外部画像も未対応です。「ソース確認」([WebGPUDriver.cpp](https://github.com/google/filament/blob/main/filament/backend/src/webgpu/WebGPUDriver.cpp)) 「両方 WebGPU」でも一つのデバイスにはなりません。「推測」 ThorVG には別の口もあります。`GlCanvas::target` で呼び手の GL のFBOへ描く道です。「ソース確認」 これを ANGLE 経由で D3D11 につなぐ(media_kit が libmpv でしているのと同じ手)例は、ThorVG では見つかりませんでした。

| 組み合わせ | macOS | Windows |
|---|---|---|
| ThorVG(WebGPU)→ Rust wgpu の合成層 | 同じ `MTLDevice`。`wgpuTextureGetNativeMetalTexture` → `create_texture_from_hal`。「ソース確認」 | **wgpu-native のC APIに口なし。「ソース確認」** |
| ThorVG(WebGPU)→ Filament | `MTLTexture` → `Texture::Builder::import`。半分ずつ「ソース確認」。組み合わせの持ち主は見つからず(「未検証」) | **道が見つからず。両側とも口なし。「ソース確認」** |
| Filament → Rust wgpu | 取り込んだ `MTLTexture` へ Filament が描く(Thermion 型、「実証済み」)。wgpu も同じ物を包む(「ソース確認」)。組み合わせは「推測」 | Filament が呼び手の `VkImage` へ描く(Thermion、「実証済み」)。wgpu-Vulkan の `texture_from_raw`(「ソース確認」)。同じ `VkDevice` に乗る組み合わせの持ち主は見つからず(「未検証」) |
| Rust wgpu → Filament(テクスチャとして) | `as_hal` → `MTLTexture` → `import`。「推測」 | `createVkImageFromExternal` 経由のみ。「推測」 |
| 動画 → 各レンダラ | `CVPixelBuffer` / IOSurface → 面ごとの `MTLTexture`。「実証済み」(Flutter、GStreamer) | D3D11 の中ならコピーなし。「実証済み」(GStreamer、media_kit)。Vulkan や D3D12 のレンダラへ NV12 を入れる出荷例は見つからず(「未検証」) |
| 各レンダラ → Flutter | IOSurface つき `CVPixelBuffer`。「実証済み」(Thermion、media_kit、flutter_wgpu) | DXGI 共有ハンドルの D3D11 テクスチャ。「実証済み」(Thermion、media_kit、minigpu_view) |

Flutter まわりの持ち主のライセンスは、Thermion が Apache-2.0、media_kit、minigpu_view、flutter_wgpu、irondash_texture が MIT です。「ソース確認」 wgpu から Flutter への出荷例は、macOS はコピーなし(flutter_wgpu)ですが、Windows ではピクセルバッファのコピーです。Dawn から Flutter は Windows だけコピーなし(minigpu_view)で、macOS は「予定」です。「実証済み」([flutter_wgpu](https://pub.dev/packages/flutter_wgpu)、[minigpu_view](https://pub.dev/packages/minigpu_view)) Flutter 自身のGPU機能(Flutter GPU)は、別のGPU APIが作った画像をまったく使えないと、2026年10月10日の要望に書かれています。「ソース確認」([flutter #194142](https://github.com/flutter/flutter/issues/194142)) 「2Dベクター、3D、合成層、動画、Flutter を、コピーなしで、両OSで」一つにまとめた公開プロジェクトは見つかりませんでした(持ち主なし)。大きなシステム(Chromium、Qt Quick)は、別々のエンジンを橋渡しせず、一つの描画の抽象で全部を持っています。「推測」

## 六枚の絵の再評価:三枚は届き、一枚は一か所欠け、二枚は核心で止まる

まず一覧です。「届く」は、既存の持ち主と、そのあいだをつなぐ材質やシェーダ(Filament の材質、今ある wgpu の合成層)で最後まで行けるという意味です。エンジンを新しく作る必要がないことを指します。

| 絵 | 判定 | いちばんの理由 |
|---|---|---|
| 絵1 シルエットの中の動画、白い文字へ変形 | 一か所欠ける | 二つのマスクの距離の地図を混ぜて変形させる持ち主が見つからない |
| 絵2 ベジェの星がガラスに | 届く(費用あり) | 押し出しの公開の持ち主は全員CPUで作り直し。星なら数十マイクロ秒 |
| 絵3 回るクロムの文字 | 届く | シルエットは「テクスチャへ描いて読む」型。ガラスは1層ですむ |
| 絵4 手描きの境界がガラスの帯に、溶ける | 核心で止まる | 「溶けて複雑な形」をGPUで立体にして Filament に渡す持ち主がいない |
| 絵5 HELLO の中の別世界 | 届く | 入れ子のカメラと1層のガラスは Filament が持つ |
| 絵6 過去のシルエットがガラスの立体に | 核心で止まる | 絵→立体の公開の持ち主なし。重なるガラスは Filament が1層 |

### 絵1:動くシルエットの中に別の動画、そのシルエットが白い文字へ変形

**既存の公開ソフトでまかなえる部分。** 3Dモデルのシルエットは、Filament の別ビューを描き先テクスチャへ描けば得られます(`View::setRenderTarget`)。「ソース確認」 「物Aのシルエットを物Bのマットにする」は、どのエンジンでも結局「Aをアルファの絵に描いて、Bで読む」形になります。「推測」 中に見せる動画は、OSのデコーダ(VideoToolbox、D3D11 ビデオ)が持ち主です。アルファや輝度で切り抜く語彙は、ThorVG(自分の絵どうし)と Skia と Vello が持っています。「ソース確認」 シルエットや文字から距離の地図を作る処理は、七つの持ち主が出荷しているジャンプフラッディングです。読み戻しなしで、1080p で1ミリ秒前後(単体のデスクトップGPUで「実証済み」)です。

**本当に新しい作業が要る部分。** 一つ目は**変形そのもの**です。「二つのマスク → 二つの距離の地図 → 混ぜて変形」を出荷している持ち主は見つかりませんでした(持ち主なし。不可能ではありません。なめらかに混ぜる演算は、数式で書いた形向けには OPENRNDR や TypeGPU にあります)。二つ目は、ジャンプフラッディングが**単体の部品として存在しない**ことです。いちばん近い bevy_mod_outline と Rerun の WGSL は、親から取り出して使うことになります。それは「持ち主を採用する」ではなく「コードを採用する」です。「推測」 三つ目は、動画の人物のようなやわらかい境界から、ピクセル未満の精度で距離の地図を作るGPUの持ち主がいないことです(持ち主なし)。四つ目は名前のある制約です。動画やシルエットのテクスチャを ThorVG のマットに直接入れることは、**ThorVG がGPUテクスチャを入力に取れない**のでできません。「ソース確認」 Windows では、文字を描いた ThorVG の出力を合成層へ渡す口が **wgpu-native に無い**ことにも当たります。「ソース確認」 なお「動画から人物を切り抜く」公開の持ち主は、今回は調べていません(アルファつき動画のコーデックだけ確認)。

### 絵2:編集できるベジェの星が厚みを得て、屈折し分散するガラスになる

**まかなえる部分。** 厚み、吸収、分散つきのガラスは Filament が持ちます。「ソース確認」 ガラス越しの動画は、動画の板が不透明なら、Filament の画面空間の屈折がそのまま曲げます(不透明な物が屈折の対象と文書にあります)。macOS では `CVPixelBuffer` がそのまま Filament の外部画像になります。「ソース確認」 星を立体にするのは、earcut.hpp か Lyon(ふた)、Manifold(水漏れのない立体)、Clipper2(面取り用の輪)です。星ひとつの三角形分割は数十マイクロ秒で、輪郭を触るたびに作り直しても1フレームの1%未満です(三角形分割の数値は「実証済み」、全体は「推測」)。頂点はCPUからGPUへの片道で、読み戻しはありません。

**新しい作業が要る部分。** 「毎フレームのCPU仕事を一切しない」を厳密に守るなら、**動く輪郭をGPUで押し出す公開の持ち主はいません**(持ち主なし。非公開の TouchDesigner は出荷済み)。面取りは、動く輪郭に対してGPUで作る持ち主がおらず(持ち主なし)、CPUでも自己交差に弱い部分です。費用として知っておくことが二つあります。Filament の立体屈折は「厚みを半径とする球」の近似なので、星の本当の形を光がたどるわけではありません。「ソース確認」 Windows では、D3D11 のデコーダが出す NV12 の動画を Filament の Vulkan 版へコピーなしで入れる出荷例が見つかりません(「未検証」)。

### 絵3:高速で回るクロムの3D文字。シルエットに2D動画、遠い所は奥行きで色づけ、別のガラスを欠けさせる

**まかなえる部分。** クロムの文字は Filament、文字の立体化は fontmesh や meshtext が持ちます。回るだけなら立体の作り直しは要らず、姿勢を渡すだけです。「推測」 シルエットは別ビューを描き先テクスチャへ。「ソース確認」 シルエットの中の2D動画は、合成側で「アルファの絵で切り抜く」だけです。別のガラスを欠けさせるのは、そのシルエットの絵をガラスの材質で読む形になります。Filament にはステンシルのAPIもあります(ただし色の描画のあとに捨てられるので、ほかのパスからは読めません)。「ソース確認」 クロムは不透明なので、ガラスは1層ですみます。

**新しい作業が要る部分。** 「3D物体のシルエットと奥行きを、2Dレイヤーと別の屈折物の両方のマットにする」を備えつけで持つエンジンは、どこにもありませんでした(持ち主なし)。つなぎは材質とシェーダの自作になります。これはエンジンの新造ではなく、既存の口(Filament の材質)の中の仕事です。「推測」 奥行きをマットの入力として扱う考え方は、映画の合成ソフト(Nuke の `depth.Z`、Fusion の Z)にはありますが、リアルタイムの公開2D合成役にはありません。Vello は `Rgba8Unorm` しか取り込めないので、小数の奥行きは入りません。「ソース確認」 いちばん近い出来合いの drei では、マスクとガラスの組み合わせに2026年に不具合報告がありました(「未検証」)。Windows では、Filament の出力と wgpu の合成層を同じ `VkDevice` に乗せる組み合わせの持ち主が見つかっていません(「未検証」)。

### 絵4:手描きの曲線が実写と3D世界を分け、境界が光り、厚いガラスの帯になって両側を屈折し、溶けて複雑な形になる

**まかなえる部分。** 曲線で二つを分けるのは、ThorVG が曲線の片側を塗ってマットにし、合成側が実写と3Dの絵を選ぶ形です。境界の光は、マットから距離の地図を作って光らせる定番で、ふち取りと光は距離の地図の持ち主がみな出荷しています。「ソース確認」 ガラスの帯の屈折は Filament の1層ですみます(両側が不透明な絵として先に写し取られていれば)。

**新しい作業が要る部分。** 「手描きの線 → 3Dのガラスの帯」を持つエンジンは、どこにも見つかりませんでした(持ち主なし)。そして**「溶けて複雑な形になる」で止まります**。形のつながり方が変わる変形は、距離の地図を混ぜるのが定番ですが、その結果を立体にする道が両方ふさがっています。メッシュにする道は、読み戻しなしで両OSで動く公開の持ち主がおらず(持ち主なし)、さらに **Filament の公開APIが頂点を「CPUからのコピー、数は固定」でしか受けない**という名前のある制約に当たります。「ソース確認」 レイマーチの道は、屈折するレイマーチの立体をよその描画エンジンに奥行きつきで重ねる持ち主がいません(持ち主なし)。Filament の材質がレイマーチと奥行きの書き込みをできるかは、今回調べていません。帯が両側を屈折するためには、合成した絵を Filament へ戻す必要があります。macOS は `import` で入りますが(「最後の手段」という注意つき)、Windows の Vulkan 版は `import` 未対応です。「ソース確認」

### 絵5:実写の街に巨大な HELLO。中は別カメラの3D世界で、ガラス玉が別の動画を中に持ち、その世界を屈折する

**まかなえる部分。** 自分のカメラを持つ別の世界は、Filament の第二のビューを描き先テクスチャへ描く機能が持ちます。描き先つきのビューは重ねて混ぜられます。「ソース確認」 文字の形の窓は、その絵を文字の形で読むだけです。中のガラス玉は、不透明な内側の世界を1層で屈折します。玉の中の動画の板は屈折しない物なので、写し取りより前に描けばガラス1層で曲がります(Filament のチャンネル0か1に置く道。「推測」、未実行)。HELLO 自身はガラスではないので、1層の制約に当たりません。外の街はOSのデコーダです。

**新しい作業が要る部分。** 窓の帳簿づけ(どの絵をどこで読むか)を出来合いで持つのは drei の `MeshPortalMaterial` ですが、ブラウザの中です。ネイティブ側には出来合いがないので、つなぎの材質は自作になります(エンジンの新造ではありません)。Filament 自身が「ビューをたくさん使うことは勧めない」と書いています。「ソース確認」 入れ子のビューを増やしたときの 1080p、60fps の費用は、どこにも数値がありません。もし HELLO や玉の外側もガラスにすると、絵6と同じ1層の制約に当たります。

### 絵6:ダンサーの0.1、0.2、0.3秒前のシルエットが色や動画ちがいで残り、過去のシルエットがガラスの立体になる

**まかなえる部分。** 部品はそろっています。ジャンプフラッディングは履歴を持たないので、同じ時刻なら同じ結果です。「推測」 Filament と ThorVG は自分の時計を持たず、時刻は親が渡すものです。動画は正確シーク(GStreamer の ACCURATE など)で好きな時刻のコマを取り出せます(仕組みだけ「ソース確認」。こすり動かしの遅れを測った資料はありません)。色ちがいや動画ちがいで残すのは、絵1と同じ切り抜きです。

**新しい作業が要る部分。** まず**時間ずらし**。「同じ物を1フレームの中で複数の時刻で評価する」を持つエンジンも、「好きな時刻へ飛べる決定的な残像」を持つ道具も見つかりませんでした(持ち主なし)。ライブ系の道具は前のフレームの積み重ねで残像を作るので、条件の「積もる雑音なし、どの時刻へも飛べる」と合いません。「ソース確認」 次に**ガラスの立体化で止まります**。動くシルエットの絵を立体にする持ち主は、非公開の TouchDesigner(Trace POP)と Notch(画像から距離の地図、そこからメッシュ)だけです。公開側は、GPUで輪郭をなぞる持ち主なし、Filament は頂点をCPUコピーでしか受けない(名前のある制約。「ソース確認」)、CPUのなぞり役(vtracer)を使うとGPUの絵をCPUへ読み戻すことになる(再生中は読み戻さない、という条件に当たる)、の三重です。最後に、過去の三つのガラスが重なる所は **Filament の「1ビューに写し取り1回」**に当たり、奥のガラスは手前のガラスの中に映りません。「ソース確認」

## 最終回答:つなぐだけでは足りない。止めているのは三つの制約

既存の成熟した公開ソフトを**つなぐだけ**で「形と絵がどの向きにも使い合える空間」が全部できるか。答えは「いいえ」です。ただし六枚中三枚(絵2、絵3、絵5)は届き、絵1も変形の一点を除けば届きます。届く範囲の役割分担は次のとおりです。

| 役割 | 持ち主 | 証拠 |
|---|---|---|
| 2Dベクター、文字、Lottie、自分の絵どうしのマット | ThorVG(出力は `WGPUTexture`。入力にGPUテクスチャは不可) | 「ソース確認」 |
| 3D、1層のガラス(厚み・吸収・分散)、入れ子のカメラ、ステンシル | Filament | 「ソース確認」 |
| 動画デコード | macOS は VideoToolbox、Windows は D3D11 / D3D12 ビデオ。公開の入り口は FFmpeg か GStreamer(LGPL に注意) | 「ソース確認」(抜粋) |
| マスク → 距離の地図 | ジャンプフラッディング。持ち主は親アプリの中(wgpu に近いのは bevy_mod_outline と Rerun の WGSL) | 「ソース確認」 |
| 輪郭 → 立体(CPU) | earcut.hpp、Lyon、Manifold、Clipper2、fontmesh | 「実証済み」(earcut の数値)、ほかは「ソース確認」 |
| Flutter への出口 | macOS は IOSurface つき `CVPixelBuffer`、Windows は DXGI 共有ハンドル。先例は Thermion と media_kit | 「実証済み」 |
| よそのテクスチャどうしの合成 | **未決**。出入り口がそろうのは Skia Graphite、次いで Vello。どちらも「エンジンをまたぐ合成役」としての出荷例は見つからず | 「ソース確認」 |
| 絵 → GPU上の立体 → よその描画エンジン | **持ち主なし**(非公開では TouchDesigner、Notch) | 「ソース確認」 |
| 重なるガラス(Filament の横で) | **持ち主なし**(Bevy、drei の中にはある) | 「ソース確認」 |
| どの時刻へも飛べる決定的な残像 | **持ち主なし** | 「推測」(見つからず) |

届かない部分を、絵と制約の名前で言います。あわせて「その制約を、別の公開プロジェクトがすでに解いているか」を書きます。これは事実の報告で、乗り換えの勧めではありません。

| 止まる絵 | 止めている制約 | 種類 | 同じ制約を解いている公開プロジェクト |
|---|---|---|---|
| 絵6、絵4(溶ける) | 絵や距離の地図からGPUで立体を作る、読み戻しなし・両OS・ゆるいライセンスの持ち主がいない | 持ち主なし | 型は keijiro(Unity 専用、上限固定で読み戻しなし)。WGSL の核は Will Usher(MIT、ただし読み戻し2回、256³ で27〜54ミリ秒)。2Dの輪郭なぞりは非公開の Trace POP だけ |
| 絵6、絵4(溶ける) | Filament の公開APIが頂点を「CPUからのコピー、数は固定」でしか受けない | 名前のある制約(「ソース確認」) | Filament については解いている公開プロジェクトなし。Unity は自分のメッシュのバッファを開けているので keijiro が成り立つ |
| 絵6(重なる過去のガラス) | Filament がガラスの背景を1ビューにつき1回しか写し取らない | 名前のある制約(「ソース確認」) | **Bevy が解いている**(N組に分けて写し直し)。drei、Babylon も別の方法で。ただしどれも自分のエンジンの中。数値はどこにもない |
| 絵1(変形) | 二つのマスクの距離の地図を混ぜる変形の出荷品がない | 持ち主なし | 部品はある(ジャンプフラッディングは七つの持ち主、数式の形向けのなめらかな混ぜは OPENRNDR と TypeGPU) |
| 絵1、絵4、絵6(Windows。ThorVG の絵をよそで使うすべて) | wgpu-native のC APIに Vulkan / D3D12 の取り出し口も取り込み口もない | 名前のある制約(「ソース確認」) | **Dawn が解いている**(`SharedTextureMemory`、Chromium が出荷)。Rust 版 wgpu も `as_hal` で口を持つ。ただし ThorVG は Dawn でビルドされず、要望 #621 は未解決 |
| 絵1ほか(動画やよその絵を ThorVG のマットに入れる向き) | ThorVG がGPUテクスチャを入力に取れない | 名前のある制約(「ソース確認」) | Skia Graphite(`WrapTexture`)と Vello(`register_texture`、毎フレームGPUコピー)は自分のレンダラで解いている。ThorVG 上流には提案 #4635 だけ |
| 絵4(Windows。合成結果をガラスに見せる向き) | Filament の Vulkan 版で `Texture::Builder::import` が未対応 | 名前のある制約(「ソース確認」) | **Thermion が別の口で解いている**(`VulkanPlatform` を継いで外部画像を差し込む、「実証済み」) |

「一方通行のルールを決めない」という望みについて、証拠が言うことをまとめます。形から絵へ(描く)と、絵から絵へ(マット、屈折)は、既存の持ち主で両向きに流れます。絵から「場としての形」(距離の地図)へも、GPUの上で流れます。**流れないのは、絵から「立体としての形」へ戻って、それを別の描画エンジンが描く向きだけ**です。ここは、持ち主がいないことと、Filament の受け口の制約が重なっています。非公開の TouchDesigner と Notch が出荷しているので、技術的に不可能ではありません。

## 以前の机上評価の訂正:五つとも、言い方を狭める必要がある

**(a)「絵から形へ戻るにはGPU読み戻しが要る」** → 一般論としては誤りです。絵から距離の地図へは、どのジャンプフラッディングの持ち主もGPUの中で完結し、テクスチャで終わります。「ソース確認」 距離の地図からメッシュへも、読み戻しは必須ではありません。keijiro は「三角形の上限を先に決めて余りを消す」、TouchDesigner は「最大数を先に確保する」で読み戻しを避けています。「ソース確認」 レイマーチにはそもそも数える物がありません。「推測」 読み戻すのは特定の実装です(Will Usher 版は2回。CPUのなぞり役はGPUの絵をCPUへ持ってくる必要がある)。正しく残るのは「読み戻しなしの公開の持ち主を、Filament につなげる形では見つけられなかった」です。

**(b)「動く押し出しは毎フレームのCPU仕事になるしかない」** → 「なるしかない」は誤りで、「公開の持ち主は今みなそうしている」が正確です。TouchDesigner の POP は輪郭、三角形分割、押し出しをGPUのコンピュートシェーダで出荷しています(非公開)。「ソース確認」 公開側は全員CPUの作り直しです。「ソース確認」 そのCPUの値段は、星や文字で数十マイクロ秒、数千点で1〜10ミリ秒です。「実証済み」(三角形分割のみ) 壁の部分は三角形分割が要らず、Filament には頂点段でカスタム属性とテクスチャから位置を動かす文書化された入り口があります(入り口は「ソース確認」、それで壁が足りるという読みは「推測」)。ふたは、GPUの2D描画役が塗りの絵として出せます。「推測」

**(c)「合成層は手書きシェーダ数本」(合成の持ち主が未定)** → 未定のままです。ただし候補と値段がわかりました。出入り口がそろう公開ソフトは Skia Graphite(Windows は Dawn 経由の見込みで「未検証」)、Vello(毎フレームGPUコピー、`Rgba8Unorm` のみ、ぼかしなし)、libplacebo(LGPL、Metal なし)です。「ソース確認」 ThorVG は入力の口が無いので持ち主になれません。「ソース確認」 Filament は独自の後処理を公開していません。「ソース確認」 どの候補にも「エンジンをまたぐ合成役」としての出荷例と、1080p・60fps・多層の数値がありません。映画の合成ソフトは、マットの種類を増やすのではなく、名前つきの追加チャンネル(奥行き、ID、法線)を絵と一緒に運ぶ形で解いています。「ソース確認」(抜粋)

**(d)「本当に屈折するガラスは1層だけ」** → Filament については正しく、技術一般としては誤りです。原因は Filament の描画の段取り(1ビューに写し取り1回)で、コードで確かめました。「ソース確認」 Bevy は写し直しでN層、drei は材質ごとの描き直しで相互に見え、Babylon は物ごとの絵で奥のガラスが曲がって見えます。「ソース確認」 言い直すなら「Filament は今、1ビューにつき1層を屈折する。ほかのエンジンは画面の写し直しや物ごとの描き直しで複数層を屈折し、光線追跡のエンジンは自由に入れ子にする」です。ただし複数層の費用の数値は、どこにもありません。

**(e)「Windows で難しいのはデバイス間の共有」** → 的が広すぎました。デバイスやAPIをまたぐ共有そのものは、Windows で出荷された道があります。DXGI 共有ハンドルで Thermion(Vulkan の Filament から Flutter)、media_kit(動画から Flutter)、minigpu_view(Dawn から Flutter)が動いています。「実証済み」 Vulkan は D3D11 / D3D12 のテクスチャやフェンスをハンドルで取り込めます。「ソース確認」 本当に欠けているのは、もっと狭い二つの口です。**wgpu-native のC APIに Windows 用の取り出し口が無いこと**(ThorVG の出力が外へ出られない)と、**Filament の Vulkan 版の `import` が未対応なこと**(代わりの口は出荷済み)です。「ソース確認」 加えて、Flutter の入口が D3D11 に限られること、D3D11 デコーダの NV12 を Vulkan のレンダラへ入れる出荷例が無いことが残ります。

## 配布するエディタで危ないライセンス

閉じた形で配るエディタに組み込むと危ないものを、種類ごとに挙げます。**GPL**:Blender(ジャンプフラッディング、Curve to Mesh、EEVEE、コンポジタのすべて)、Natron、Olive、Friction、OBS、openfx-misc / openfx-io / openfx-arena、Gustavson の edtaa(GPL v3以降との報告)、Processing の IDE。CGAL の Straight_skeleton は未確認ですが GPL の可能性が高く、potrace も GPL と記憶していますが今回は確認していません(どちらも「未検証」)。**AGPL**:hydra-synth、Theatre.js の studio。**LGPL**:libplacebo(LGPL-2.1以降)、GStreamer、FFmpeg(ビルドによって GPL)、libmpv(ビルドによって GPL か LGPL)、VL.StandardLibs(LGPL-3.0)、Qt、TuttleOFX、Processing の core。media_kit は自分は MIT ですが、中の libmpv のライセンスを引き継ぐ点に注意が要ります。**非商用**:Shadertoy の既定(CC BY-NC-SA 3.0。「未検証」)。距離の地図やレイマーチのガラスの公開例の多くがここにあります。hg_sdf は MIT と CC-BY-NC-4.0 の二択なので、MIT を選ぶ必要があります。HAP の Rust 実装 haplab は PolyForm-Noncommercial です。**表示義務**:RayTK(CC BY 4.0)。**商用・非公開**:TouchDesigner、Notch、Smode、Resolume、vvvv gamma、Unity、Unreal、MudBun、Remotion(4人以上の会社は有料)。Diligent は一部の組み合わせが商用ライセンスのみです。**不明**:Ben Golus の gist、fastsdf、PBA+ のリポジトリ、Use.GPU の glyph、Humus のデモ、Pikimov、VL.Stride.SDFToMesh、GPUOpen の記事のコード、ttf2mesh のC本体。libtess2 の SGI Free Software License B は珍しいので確認が要ります。Clipper2 は「商用でも無料」と書かれていますが、本文は読めていません。

特許の旗が二つあります。Microsoft の US11074724B2「GPUによる距離の地図の計算」は、点の列で定義した形から距離の地図を作る方法で、有効とされています。マスクからではなくベクターの形からの話で、請求項は読んでいません。([Google Patents](https://patents.google.com/patent/US11074724)) Slug の特許は2026年3月17日に公有に置かれました。「ソース確認」 Loop-Blinn の特許の失効日は「未検証」です。

ゆるいライセンスと確認または記憶しているものは、ThorVG(MIT)、Filament(Apache-2.0)、wgpu と wgpu-native(MIT / Apache-2.0)、Dawn と Skia(BSD-3)、Rive(MIT)、Vello(Apache-2.0 / MIT)、Bevy、Godot、three.js、drei、Manifold、earcut、Lyon、TiXL、VL.Fuse、Thermion です。ただし多くは今回の調査で LICENSE ファイルを読み直していません。採用を決める前に、それぞれ読む必要があります。

## 今回の調査で取れなかったもの

何もビルドせず、何も動かしていません。だから、この報告に Motolii の環境での計測は一つもありません。取れなかったものを領域ごとに書きます。

| 領域 | 取れなかったもの |
|---|---|
| 速さ全般 | Apple Silicon / Metal、内蔵GPU、wgpu でのジャンプフラッディングの時間。ほかの描画エンジンと同時に動かしたときの時間。1080p での2Dの場 → メッシュの時間。重ねた屈折のフレーム時間(どのエンジンも未公表)。合成役の候補すべての「1080p、60fps、多層、マットとぼかし」の数値。入れ子ビューの費用。エンジン間の同期の費用 |
| 押し出し | Lyon、`Manifold::Extrude`、Clipper2 の絶対時間。壁、面取り、法線、転送の時間。「動く輪郭を毎フレーム押し出して描く」全体のベンチマーク。Filament の毎フレームの頂点更新が止まるかどうか。ふたの三角形の割り方が動きでちらつくかの計測 |
| 距離の地図 | Blender の境界シェーダ、OPENRNDR と TypeGPU のシェーダ本体。Shadertoy の規約本文(403)。Apple の MPS の文書本文。TiXL、cables.gl、vvvv のオペレータ一覧(見つからないことの確認には至らず)。各リポジトリの最終コミット日 |
| 立体化 | Inigo Quilez の記事(404)。Notch と TouchDesigner のOS対応とGPU / CPUの別。VL.Fuse、TiXL、bevy_march が奥行きを書くか、屈折を持つか。Filament の材質でレイマーチと奥行きの書き込みができるか。Filament の内部(非公開)の口。HPMC やGPUデュアルコンタリングの各リポジトリ |
| エンジン | Unreal と Unity の公式文書。Babylon Native、Stride、O3DE の機能表。Godot の LibGodot が親のテクスチャへ描けるか。**Filament の WebGPU 版の成熟度**(記述がどこにもない)。走っているデモの確認(GPU計測のじゃまをしないよう、あえて見ていません) |
| 合成と動画 | Cryptomatte の仕様本文(404)。Adobe の Advanced 3D の制限一覧(403)。Skia の Ganesh のAPIと、Graphite の Windows での状況の一次資料。Rive が外部テクスチャを画像の元にできるか。AVFoundation と Media Foundation の調査。FFmpeg の hwcontext のソース。ProRes 4444 のアルファのハードウェアデコード。こすり動かしの遅れと、シークの決定性。**動画から人物を切り抜く公開の持ち主(未調査)** |
| 共有 | Apple の `MTLSharedEvent` と IOSurface の文書本文。Thermion の macOS 側ソースと Windows の現行文書。Flutter 3.47 の公式リリースノート(Windows の Impeller が ANGLE か Vulkan かで二次情報が食い違い)。DXGI ハンドルが NT ハンドルも通るか。D3D11 の共有ハンドルから開いた資源を wgpu の DX12 に渡したときの挙動。NV12 のテクスチャ配列を Vulkan へ入れる出荷例 |
| 探し方の限界 | 探索はウェブ検索と直接のファイル読みだけで、GitHub のコード検索や課題一覧の全件確認はしていません。「見つからなかった」は、網羅的な否定より弱い証拠です |

## 結論:切れているのは一点、欠けているのは口と数値

調査の前は、問いが「全部つなげるか、専用エンジンを作るか」の二択に見えていました。証拠が示したのは、もっと細い形です。形から絵へ、絵から絵へ、絵から「距離の地図」へは、すでに公開の持ち主がGPUの上で流しています。流れが切れるのは一点だけ、「絵から本物の立体へ戻り、それを別の描画エンジンが描く」所です。そこには、持ち主の不在と、Filament の受け口の制約と、ガラス1層の段取りが、同じ場所に重なっていました。絵4と絵6がそろって止まるのは偶然ではなく、同じ一点を踏んでいるからです。

もう一つわかったのは、欠けている物の多くが「エンジン」ではなく「口」だということです。Windows の wgpu-native の取り出し口、ThorVG のテクスチャ入力、Filament の Vulkan 版の取り込み、Filament の頂点の受け口。どれも、同じ制約を別の公開プロジェクト(Dawn、Skia、Thermion、Unity 向けの keijiro)がすでに解いた実例があります。ただし、解いた場所がそれぞれ別のエンジンの中なので、今は貸し借りができません。そして、どの道を見ても数値がありません。重なるガラスも、合成役も、エンジン間の同期も、1080p・60fps での費用を誰も公表していません。いま足りないのは文献ではなく、数値です。
