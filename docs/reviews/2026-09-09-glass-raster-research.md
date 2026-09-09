# Glass: 説得力のある反射・屈折を2Dと3Dへ

2026-09-09。利用者方針: 物理的正確性は必須ではない。リアルさと他オブジェクトの映り込みが欲しい。嘘でよく、2Dにも適用する。重いレイトレーシングを前提にせず、まず文献を調べる。
状態: 文献・現行コード調査。以下の推奨は候補であり、方式の採用・実装・速度の実証ではない。

## 共通化前の接点（初回調査時点）

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


## 携帯機・モバイル向けの追加調査（2026-09-09）

利用者依頼: Repeaterで複製数が増えても成立する嘘を、ゲームエンジンのリアルタイム処理から探す。
状態: **観察**。以下は既存実装の調査とMotoliiへの転移仮説。反射方式の採用・実装・速度の検収ではない。
表面の共通化は[別の検収記録](2026-09-09-shared-surface-plan.md)を参照。

### 確認した一次資料・公開ソース

| 資料 | 確認できた機構 | Motoliiへの転移条件 |
|---|---|---|
| [Arm, Combined Reflections: Stereo Reflections in VR (2016)](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/combined-reflections-stereo-reflections-in-vr) | Ice Cave demoは静的な洞窟をlocal cubemap、動く物体を鏡映カメラ、空を遠方環境として合成。動的撮影の対象をlayer maskで絞り、反射面が見える時に描く | 静的/動的の混合が重要。全物体を毎回6面撮影する手法の実証ではない。作品で動く物体を静的扱いしてはならない |
| [Unity 6.0, Optimize reflections](https://docs.unity3d.com/6000.0/Documentation/Manual/RefProbePerformance.html) | 小さく遠い反射は低解像度、不要物をculling maskで除外。scriptで更新を選ぶ。6面描画と粗さ用処理を時間分散。リアルタイムprobeはメモリ内非圧縮 | 解像度・撮影対象・更新の独立制御を借りる。baked probeの圧縮による軽さを動的probeへそのまま計上しない |
| [Unity URP ReflectionProbeManager.cs (master、閲覧日2026-09-09)](https://github.com/Unity-Technologies/Graphics/blob/master/Packages/com.unity.render-pipelines.universal/Runtime/ReflectionProbeManager.cs) | visible probe数を制限し、cacheとatlasを管理。texture.updateCount等で更新判定し、cubemapをoctahedralの2D atlasへ転写。容量不足ならprobeをskip | これは既存の反射画像の配置・更新管理。シーンの再撮影費用を消す処理ではない。atlasは需要で拡張されるため、そのまま固定VRAM予算と呼ばない |
| [Godot 4.4 renderer_scene_cull.cpp](https://github.com/godotengine/godot/blob/4.4/servers/rendering/renderer_scene_cull.cpp#L3296) | probeは6方向の通常描画。mask・far距離・mesh LOD・影の有無を渡す。UPDATE_ONCEは1段ずつ進め、UPDATE_ALWAYSは全段を回す | 撮影品質と更新scheduleを分離する実装の参考。全probeにAlwaysを付けると上限付きになるわけではない |
| [Epic UE4.27 Mobile Rendering](https://dev.epicgames.com/documentation/en-us/unreal-engine/mobile-rendering?application_version=4.27) | mobile向けreflection capture圧縮と、4.26で加わったPixel Projected Reflectionを記載。PPRはMobileHDR等の条件があり、既定有効化はhigh-end mobile向け | mobile向けの採用先例。ただし全携帯機・全mobile GPUで同じ性能が出る証明ではない |
| [Adam Cichocki, Pixel-projected reflections, SIGGRAPH 2017](https://advances.realtimerendering.com/s2017/PixelProjectedReflectionsAC_v_1.92.pdf) | 画面の色・深度と解析的な平面形状から、反射先へ画素をscatterする。基本方式はray marching不要。複数の平行面の事前選別、透明ガラス板のatlas、穴埋めも説明 | 平面に制限され、画面外の情報は戻らない。大量の面を全画素から総当たりする設計も避ける。曲面や強い法線変化は別の近似が必要 |
| [The Forge, 10_ScreenSpaceReflections](https://github.com/ConfettiFX/The-Forge/tree/master/Examples_3/Unit_Tests/src/10_ScreenSpaceReflections) | 公開C++実装にPPR/SSSRの経路、平面情報、穴埋め・fade・強度・面数設定がある。PPRの中間bufferは画面サイズに基づく。UIの比較範囲は1〜4面 | 参照実装として読みやすいが、1000枚の透明板への対応や性能をこのsampleから主張しない。今回は実行していない |

PPR資料の測定はGTX 1070・3840×2160の特定比較場面で0.95ms（p.41–46）。携帯機の数値に読み替えない。公開検索はArm/Unity/Godot/Epic、SIGGRAPH資料、GDCのNintendo Switch+reflectionを対象にした。今回、特定のSwitch製品がどの方式で何msを達成したかを直接示す開発者一次資料は確保していない。

### 推奨の修正（転移仮説）

全動的シーンを少数のcubemapに撮り続ける案だけに絞らず、以下を同じ表面契約の内側で比較する。

1. **動かない周囲**: shared local probeをcache。関連する素材・配置・照明が変わった時に失効させる。
2. **動く物体・画面内の細部**: 板/ほぼ平面ならPPR、曲面ならSSRなどを候補にする。画面外・失敗領域はprobe/環境へ補完する。動的な画面外物体は古い静的probeでは映らないので、限定した動的captureが必要なケースとして測る。
3. **Repeater**: 複製ごとのprobeを既定にしない。同一平面の複製群は反射画像を共有し、元のcoverageで個々の形を切る案を比較する。平行な別平面には資料の選別法を検討。任意向きの曲面群へこの最適化を無条件に広げない。
4. **予算**: probe枚数/解像度、画面用中間buffer、反射に描くgeometryの量を別々に上限管理する。instancingはdraw-call/提出の重複を減らす候補だが、複製の全頂点・全pixelの費用を消すものではない。
5. **時刻**: 画面の現在frameを使う近似をまず比較する。時間分散はそのまま作品結果の定義に持ち込まない。同じDocument・時刻・camera・品質ならcacheの温まり方によらず同じ結果となるようにする。

### 次の比較に必要な反例

同一平面の2D複製 / 奥行きの違う平行面 / 向きがばらばらの板 / 曲面群を分け、各10・100・1000個で固定した反射予算のままGPU時間を測る。個数だけでなく総頂点数、画面占有率、重なり、反射面数を記録する。
主カメラ外へ出る物体、ガラスの重なり、自己反射、急な回転、任意frameへの直行を確認する。共通のsurface APIは維持し、画面方式のために3Dを2Dへ畳まない。


Happy Elementsの具体的なMV制作例と、反射・Repeaterを含む採用方針は[演出と実装の採用方針](2026-09-09-entertainment-rendering-adoption.md)へ集約した。
