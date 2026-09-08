# Glass: 説得力のある反射・屈折を2Dと3Dへ

2026-09-09。利用者方針: 物理的正確性は必須ではない。リアルさと他オブジェクトの映り込みが欲しい。嘘でよく、2Dにも適用する。重いレイトレーシングを前提にせず、まず文献を調べる。
状態: 文献・現行コード調査。以下の推奨は候補であり、方式の採用・実装・速度の実証ではない。

## 現行の接点

- `motolii/crates/motolii-render/vism/glass.wgsl`: surface hookからforkの`shade_surface`を呼ぶ。
- `motolii/crates/motolii-render/src/compositor/sequential.rs`: Modelは`MeshDrawData`、2Dは`TexturedRect` / `RectangleDrawData`へ分岐。背後のmip画像はModelがあるrunに渡す。
- Cargoのfork参照は`5e32b3f281803c6f9b4379fc6a6de978e50fba14`。ローカルforkの`shader/rectangle_vs.wgsl`は頂点番号から板の位置を生成し、`rectangle_fs.wgsl`は画像の読出し・色・alphaを扱う。mesh用surface hookと同じ入口ではない。
- forkの`shader/utils/lighting.wgsl`: 反射は`environment_specular_along`、屈折はbackdrop。作品中の物体を反射用画像へ描く経路が必要。
- 「両方とも三角形で描く」は成立するが、既に同じ材質処理を通るという意味ではない。2Dのsurface受け口と、反射画像を作る仕組みは別の不足。

## 一次文献と実装

| 方式・資料 | 借りられる機構 | 制約・費用 |
|---|---|---|
| [GPU Gems 2, Ch.19, Tiago Sousa / Crytek](https://developer.nvidia.com/gpugems/gpugems2/part-ii-shading-lighting-and-shadows/chapter-19-generic-refraction-simulation) | 背景画像のUVを法線マップでずらす。マスクで前景の漏れを抑える。反射画像とFresnel近似で混ぜる。Far Cryでの水・熱気・レンズの先例 | 実際の光路を解かない。大きな歪みでは遮蔽・輪郭に破綻が出る |
| [Three.js CubeCamera](https://threejs.org/docs/pages/CubeCamera.html)、[実装](https://raw.githubusercontent.com/mrdoob/three.js/dev/src/cameras/CubeCamera.js) | 周囲6方向を通常描画し、反射用cubemapにする。撮影対象自身を隠す公式例あり | 6方向の追加描画。複数物体で共有すると反射位置は近似。各物体専用にすると費用が増える |
| [Godot Reflection probes](https://docs.godotengine.org/en/stable/tutorials/3d/global_illumination/reflection_probes.html) | 画面外の物体も反射。Box Projectionで視差補正、対象選別、粗いLOD、SSRとの混合 | 動的更新は再描画。箱による補正は任意形状の正確な交差ではない |
| [Three.js Reflector](https://threejs.org/docs/pages/Reflector.html)、[実装](https://raw.githubusercontent.com/mrdoob/three.js/dev/examples/jsm/objects/Reflector.js) | 面に対して鏡映したカメラで描き、平面へ投影。レイトレーシング不要 | 平面向け。反射面ごとの追加描画。曲面へ貼るだけでは正しい位置にならない |
| [AMD FidelityFX SSSR](https://gpuopen.com/manuals/fidelityfx_sdk/techniques/stochastic-screen-space-reflections/) | 深度階層を使った画面内探索、粗さによる探索密度調整、denoise | 画面内の情報に依存。深度・法線・粗さ等が必要。時間方向denoiseは編集時のシークも検討が必要。DX12/Vulkan向けSDKをwgpuへそのまま導入できるわけではない |
| [Godot screen-reading shaders](https://docs.godotengine.org/en/stable/tutorials/shaders/screen-reading_shaders.html) | 2Dにも画面画像を読み、mipでぼかす先例。明示的なback buffer copy | 重なる効果はコピーのタイミングに依存。3Dの通常screen copyでは透明物が抜けるなど、取得時点の契約が必要 |
| [Three.js MeshPhysicalMaterial](https://threejs.org/docs/pages/MeshPhysicalMaterial.html) | thickness / thicknessMap、normal、transmissionを材質パラメータとして扱う | 幾何の厚みと見せたい光学的厚みを分ける参考。これだけでシーンの相互反射が生えるわけではない |

SSRはBVHやRTハードウェアを必要としないが、画面の深度に対してray marchする方式ではある。「rayを一切追わない」候補はプローブ・平面反射・UV変形。

## Motoliiへの推奨候補（推論）

主候補は、動的反射プローブ + 既存backdrop屈折 + 2D/3D共通のsurface契約。平らな鏡を強く見せたい場合は平面反射を比較する。SSRは画面内の細部を補う追加候補とする。

1. 相手の姿が映るための反射画像を通常のre_renderer描画で作る。初回の比較では反射の再帰を切り、撮影時の材質は既存の環境反射までにする。AにB、BにAを映すことと、合わせ鏡を何段も解くことは分ける。共有probeの自己像・視差誤差は、対象除外や複数probeとの比較で確かめる。
2. 共通にするのはsurface入力（位置・法線・UV・coverage・見かけの厚み）と反射/屈折画像の参照。2Dを厚い立体に作り直すことは前提にしない。板は平面法線、3Dは既存法線を供給する。
3. 2Dに膨らんだガラスらしさを付ける候補として、法線マップや高さからの擬似法線、任意の見かけの厚みを使う。alpha輪郭から縁の丸みを作る案は追加の調査・比較が必要。平面法線だけでは、正面から見た板に大きなレンズ状の歪みは生まれない。
4. 2D素材のalphaは形のcoverageとして保持し、ガラスの透過率とは分ける。文字・切り抜き画像が四角いガラス板に化けるのを防ぐ。
5. 反射用の撮影は現在の「そこまでの合成」だけを読む経路から分ける。先に描いた層しか参照できないままでは、相互の映り込みを満たせない。カメラ独立2Dの反射撮影時の姿は主カメラ時点の配置を固定する案とし、明示的に比較する。

## 比較で確認すること

- 3D球に2D文字が映り、2Dガラスにも3D物体が映る。同じGlassを画像・文字・図形に適用できる。
- 主カメラの画面外へ物体を移動しても映り込みが消えない。
- ガラス2枚、透明な文字の穴、交差、強い歪み、粗さ0と1、自己像、反射の位置ずれ。
- 正順再生・逆シーク・任意フレーム直行・exportで同じ時刻の像が一致する。時間分散更新を使う場合、古い反射を作品の結果として固定しない。
- 反射解像度・probe数・対象数・画面占有率を変え、同じ場面でGPU時間と見た目を比較。通常描画方式でも6面×多数の毎frame更新は重くなり得るため、軽いとは未計測で断言しない。

この調査ではコード変更・build・実窓での性能比較は行っていない。
