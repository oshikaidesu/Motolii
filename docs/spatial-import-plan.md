# Spatial import — 3D modelと点群を同じ物として扱う

日付: 2026-09-02

利用者裁定: **点群の振る舞いは普通の3Dを踏襲する。**

形式差はdecoder／primitiveだけに閉じる。BrowserへのImport、card、Timeline layer、Position／Scale／
Rotation X/Y/Z／Opacity、Stage選択、保存再読込、Preview／Exportは同じSpatial Asset契約を通す。

## 現窓baseline

| 経路 | 現在 |
|---|---|
| OBJ | Import→3D card→Timeline→白mesh描画までは通る |
| PLY | Import→3D card→Timeline→Points3D描画までは通る |
| GLB/GLTF/STL/DAE | Rerun拡張子表では受理されるが、製品`is_mesh_path`はOBJだけ。配置後はvideo decoderへ落ちる |
| 選択 | Meshは自然寸法を引けず枠0。点群はcomp寸法×数千倍scaleとなり枠が画面外 |
| 3D Transform | Position/Scale/Zは届くが、Rotation X/YをMesh/Points3D rendererへ渡していない |
| 点の大きさ | scene unit 2をimport fit scaleで数千倍し、大きな円盤になる |
| cache | 点群は毎frameでpositions/colorsをclone、Meshは毎frameでGPU upload |

## 検索receipt

| field | 内容 |
|---|---|
| `NEED` | 3D importがOBJの一部描画に留まり、点群も通常3Dと同じ選択・回転・見た目・cache挙動になっていない |
| `SEARCHED` | 現行Browser/Engine/Compositor、pin済みRerun `Asset3D`／`Points3D`／model importers／PointCloudBuilder、Khronos glTF 2.0、既決M5境界 |
| `DISPOSITION` | **REUSE_REMAP**。scene graph、mesh parser、point renderer、GPU model cacheを自作せず、pin済みRerun importerとdraw dataを既存Layerへ写す |
| `OWNER/ROUTE` | `File Import → AssetTable → one Media card → LayerSource::File → Spatial kind dispatch → Rerun CpuModel/GpuMeshInstance or Points3D → same sequential compositor` |
| `RULER` | [Rerun Asset3D](https://rerun.io/docs/reference/types/archetypes/asset3d)はGLTF/GLB/OBJ/STLをprepacked 3D assetとして扱う。[Rerun Points3D](https://docs.rerun.io/dev/reference/types/archetypes/points3d/)はpositionsを必須、color/radiusをoptionalとする。[glTF 2.0](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.pdf)はscene/node TRS、right-handed座標、meter単位を規定 |
| `ORACLE` | 同じSpatial TransformをMesh/Pointへ与えるとRotation X/Yで両方の画素が変わる。点径はUI pointでscale非依存。GLB/OBJ/STLとPLYが非背景画素を出し、選択枠がviewport内。Save/Open後に同じ画素、Preview=Export |

## 採る範囲

- 3D model: `.glb`、`.obj`、`.stl`
- point cloud: `.ply`
- glTFのscene/node transform、mesh instance、base color texture/factorはRerun importerの結果を保持
- OBJの外部MTL、外部参照を持つ`.gltf`、DAEは黙って白や別形へせず、今回の対応形式へ含めない
- 点群のpoint radiusはRerunのUI point。modelのfit scaleで太らせない
- Effect/Mask/Matteは2D textureへの処理なので、3Dを明示Flattenする契約が閉じるまでは素通しの既存境界を維持

## 共通Spatial Transform

Documentの2D Affine(Position/Scale/Rotation Z/Skew)をXY basisへ、Zをtranslationへ、Rotation X/Yを
同じquaternionへ写す。Mesh instanceのnode transformはその内側に残し、点群はidentity instanceとして同じ
`world_from_object`を使う。点群専用の回転・位置・cameraを作らない。

## 受入

1. format分類が実描画対応表と一致し、受理後video decoderへ落ちない
2. upstream fixture GLB/OBJ/STL/PLYを同じblank projectへImportし、card→layer→Stageを確認
3. Mesh/pointとも選択枠、Stage move、Rotation X/Y、Opacity、Undo/Redo
4. 同じprojectをSave/Openし、Preview/Export pixel一致
5. cache hit後はMesh GPU uploadとpoint positions/colors cloneがframe数へ比例しない
6. 全回帰、docs、diff check

## 実装結果

- `.glb/.obj/.stl`をpin済みRerunのmodel importerへ接続し、scene/node instance、material、textureを
  `GpuMeshInstance`のまま既存sequential compositorへ積む
- `.ply`はRerun `Points3D::from_file_contents`と`PointCloudBuilder`を継続使用
- Mesh/point共通の`SpatialBounds`とcenter-preserving XYZ transformを一つにし、source boundsを
  0始まりへ正規化。Browser fit、Stage選択枠、rendererが同じboundsを読む
- Point radiusをscene unitからUI pointへ変更。import fit scaleを変えても点径は太らない
- model GPU uploadはsourceごと、point positions/colorsは`Arc`でsourceごとにcacheし、frameごとの再構築を停止
- Inspector sourceは`3D model`／`point cloud`と表示するが、編集行とIntentは同じ

## 検証結果

| oracle | 結果 |
|---|---|
| format gate | GLB/OBJ/STL=`model/*`、PLY=`pointcloud.ply`。外部参照問題を残すGLTFとsubsetのDAEはImport時にunsupported |
| bounds/fit | 同じ3点のOBJ/PLYが同一Position/Scale。両方の中心が640×480 compの320×240へ一致 |
| GPU render | 生成fixtureのGLB/OBJ/STL/PLYが全て非背景画素。Rotation Xで全形式の画素が変化、Opacity 0.35で輝度低下 |
| selection | render後の全形式が有限の選択寸法を返し、rotation前後で同じsource frameを維持 |
| cache | repeated frameでmodel cache 1件・同じ`Arc<GpuModelData>`、point cache 1件・同じpositions `Arc` |
| persistence | PLY projectをSave/Openし、別Engineで同一pixel |
| 実窓 | blank→GLB/PLY Import→card→layer。GLB/PLY双方にviewport内の選択枠、Rotation X、Stage move、Opacity、Undo/Redo |
| 実窓Save/Open | `/tmp/spatial-import.rrd`へ保存し、GLB/PLY 2層・位置・回転・opacity・画を復元 |
| 実Export | `/tmp/spatial-import.mp4`を1800/1800完走。H.264 1920×1080 30fps 60秒、size 179,945 bytes。復号frameにGLB面とPLY点群を確認 |
| 全回帰 | `cargo test --locked --no-fail-fast` 104/104 PASS |

Rerun fixture PLYの`intensity/nx/ny/nz`はupstream `Points3D::from_file_contents`が警告して無視する。
これは黙ったMotolii変換ではなく上流loaderの現在の対応範囲として残す。法線／intensity駆動を使う時は
Points3D schemaの別拡張として扱い、今回のImportを独自parserへ戻さない。

現在判定: **PASS**。
