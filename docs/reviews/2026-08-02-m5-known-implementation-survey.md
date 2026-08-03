# M5既知実装調査（2026-08-02）

状態: **比較中**

対象: [M5 3D／post仕様](../specs/M5-3d-and-post.md)、
[既知実装採択・置換開発モデル](../known-implementation-adoption-model.md)

## 1. 調査目的と判定境界

M5のP0I〜P7を独自engineの実装順として読まず、一般機構を既知実装へ割り当てる。調査対象は
具体的なcrate／API、version、license、所有する状態、failure mode、platform条件である。Motolii固有の
単一XYZ世界、遮蔽policy、Document identity、D2更新、plugin能力、preview／export oracleは外部実装から
逆算しない。

本調査は依存追加、公開型追加、Document schema変更、runtime実装を許可しない。`ADOPT-PROBE`候補は
小さいcompatibility fixtureと独立した反対側レビューを通してから採択地図へ移す。候補の不適合を
独自scene frameworkの実装許可にはせず、`REMAP / REDUCE`へ戻す。

## 2. 現行コード事実

| 領域 | 現行事実 | M5として証明しないもの |
|---|---|---|
| Document | stable ID、parent、Transform2D、単一writer／Undo routeがある | 3D object、mesh、material、camera schema、ECS |
| camera | `motolii-core::CompCamera`はplanar orthographic observationを表す | perspective、view/projection matrix、depth policy |
| render | `motolii-render`はwgpu 29上のlinear RGBA graphとtarget poolを持つ | depth attachment、mesh pass、glTF scene、post graph |
| vector/text | Vello 0.9、fontique、harfrustを製品codeで接続し、`draw_glyphs`を使う | 3D text、全組版、Velloを全scene rendererにすること |
| identity | Documentのstable IDsとworkspaceの`sha2`がある | deterministic duplicate用のversion付きInstanceId導出 |
| UI | native Timeline／Inspectorとtyped intent→D2 routeがある | 3D gizmo、GPU picking、scene stateのUI所有 |
| media | GPU textureをrender入力へ供給する既存routeがある | image planeのdepth／material意味、scene ownership |

したがってM5は新しいworld、ECS、render graph、text stackを先に所有しない。既存Document、published
snapshot、wgpu device、Vello renderer、text shaping、D2 commandを接合先とする。

## 3. 機構class別の一次資料照合

### 3.1 private 3D mathとcamera observation

- **候補**: `glam` 0.33.2、MIT OR Apache-2.0。
- **具体API**: `Vec3`、`Quat`、`Mat4`、`Affine3A`、WebGPU向けprojection constructor。
- **供給route**: `ADOPT-PROBE`。render／projection adapter内のprivate mathへ限定する。
- **移さないもの**: glam型をDocument、serde、公開plugin APIへ露出しない。正準Y-up、高さ1.0、cameraの
  observable意味はMotolii authorityのままにする。
- **probe必須**: handedness、clip-space Z、逆行列、negative scale、parent chain、orthographic互換、
  macOS／Windowsの同一fixture。

math crateを採ってもscene modelは増やさない。Documentの既存object／parent／stable IDからimmutableな
render snapshotへ必要値を投影し、cameraは同じworldを観測する値として渡す。

### 3.2 glTF読込と検証

- **第一候補**: `gltf` 1.4.1、MIT OR Apache-2.0。
- **具体API**: `Gltf`／`Glb`、accessor、buffer、image、mesh、material、node、animation traversal。
- **供給route**: parserを`ADOPT-PROBE`。`import()`のambient filesystem読込を製品境界にせず、Hostが
  解決済みbytes／許可resourceを渡す薄いadapterを置く。
- **外部検証**: Khronos `glTF-Validator`を`EXTERNAL` fixture oracle、`glTF-Sample-Assets`を
  `EXTERNAL` corpusとする。validatorの成功をMotolii表示互換の証明にはしない。
- **probe必須**: embedded／external buffer、data URI、Unicode、multiple scenes、negative scale、
  sparse accessor、unlit、alpha mode、missing resource、巨大宣言、NaN、unsupported extension。
- **移さないもの**: `gltf`の型をDocument／公開APIへ保存しない。scene graph、material、animationを
  丸ごとMotoliiの恒久schemaへ写さず、対応subsetをHost-owned import resultへ変換する。

Khronos validatorはschema、reference、buffer、accessor、animation、extensionの不正を広く検出するが、
GPU budget、malicious size、Motolii軸変換、色、遮蔽policy、preview／export一致は別fixtureで閉じる。

### 3.3 OBJ変換

- **候補**: Cesium `obj2gltf`、Apache-2.0、Node CLI。
- **供給route**: v1必須runtime依存ではなく`EXTERNAL-COMPARE`。`--secure`、up-axis、material／texture
  変換を隔離fixtureで確認する。
- **停止線**: Node runtimeやnpm treeを製品必須配布へ追加する、変換結果を検証せず取り込む、OBJ／MTLの
  pathをambientに辿る必要があるなら採らない。初期成果をglTFに縮小できるかを先に比較する。

### 3.4 mesh、depth、visibility、post

- **既知実装**: workspaceのwgpu 29と`motolii-render::RenderSession`を`REUSE/PATTERN`する。
- **具体境界**: vertex/index buffer、depth texture／state、render pass、bind group、pipeline cache、既存の
  preview／export共通関数と`Quality`。
- **薄い残余**: Motolii snapshotからdraw packetへ写すadapter、既存遮蔽policyからpass列を選ぶplanner、
  color変換直前に閉じるpost pass。
- **移さないもの**: 別GPU device、別render graph owner、engine-owned world、preview専用scene renderer、
  postごとの色変換。

`rend3`はworld、renderer、render graphまで所有し、公式repositoryは2025-06-07にarchiveされている。
現行wgpuとDocument ownerを二重化し、保守停止した別engineへ固定するため`REJECT`する。wgpu exampleは
API使用の`PATTERN`であって製品仕様ではない。

### 3.5 bounds、ray、picking、gizmo

- **候補**: `parry3d` 0.29.0、Apache-2.0。
- **具体API**: `Aabb`、`Ray`、`RayCast`、`TriMesh`、bounding volume／partitioning。
- **供給route**: CPU broad-phase／ray fixtureだけを`ADOPT-PROBE`。依存closureとmath変換が、必要な
  AABB／triangle intersectionをprivate helperで閉じるより責任を減らす場合だけ採る。
- **通常route**: published render snapshotのworld boundsをCPUでhit-testし、stable object IDを返す。
  native overlay／既存typed intent／D2 commandを`REUSE`してgizmo操作を接続する。
- **拒否**: color-ID GPU picking、独立gizmo framework、UI-local scene selection、physics world。
- **保守fallback**: boundsが`Unknown`ならpick最適化を諦め、可視候補へ保守的に広げる。誤って対象を
  除外しない。

### 3.6 textとVello局所pass

- **供給route**: workspaceのfontique→harfrust→Vello `draw_glyphs`を`REUSE`する。既存native Timelineの
  concrete接続を共通ownerへ移す候補として扱い、M5用に複製しない。
- **Vello**: vector／glyph／2D overlayの局所passとして既存long-lived rendererと同一deviceを使う。
  Velloをmesh、depth、全postのscene engineへ昇格しない。
- **alpha境界**: Velloのstraight-alpha出力は既存CQ-7に従い、premultiplyを一度だけ行う。
- **非証明範囲**: vertical text、全組版、3D extrusion、font asset permanence、public authoring API。

### 3.7 deterministic duplication

- **候補**: `rand_pcg` 0.10.2の`Pcg32`を`ADOPT-PROBE`し、既存`sha2`を`REUSE`する。
- **供給route**: Hostが`document identity + source object ID + duplicate slot + algorithm version`を
  canonical bytesへ固定し、seed／streamを導出する。OS entropy、thread order、hash map iterationを使わない。
- **probe必須**: upstream reference vectors、Mac／Windows、serialization round-trip、Undo／redo、
  insert／delete後の既存instance不変、version migration拒否。
- **非証明範囲**: field列、byte order、slotの寿命、collision policy、公開InstanceId。これらはM5の
  製品fixtureで決める。

## 4. M5へ採らない一般機構

| 候補 | 処分 | 理由 |
|---|---|---|
| rend3 | `REJECT` | archive済みで、world／render graph／wgpu ownerを二重化する |
| Bevy等のfull engine／ECS | `REJECT` | app loop、world、asset、schedule、reflectionまで恒久責任が増える |
| glTF型をDocumentへ保存 | `REJECT` | importer versionと外部formatを恒久schema／plugin契約へ漏らす |
| 独自glTF／OBJ parser | `REJECT` | parser、安全性、extension、fixture corpusを再所有する |
| GPU color-ID picking | `REJECT` | readback、同期、ID format、別passを増やし、CPU bounds routeで成果を閉じられる |
| 新しいtext stack | `REJECT` | fontique／harfrust／Velloの既存接合を複製する |
| Velloを全scene engine化 | `REJECT` | mesh／depth／post ownerを局所vector rendererへ押し込む |
| scene-local cache／scheduler | `REJECT` | M4のHost-owned resource／artifact routeと競合する |

## 5. M4と一度だけ閉じる共通接合部

1. **Resource admission**: mesh、texture、depth、Vello targetもM4のHost resource admissionへ入り、
   M5専用budget managerを持たない。
2. **Artifact identity**: import変換物、baked geometry、shader成果物はM4のversion付きrecipe key／CAS候補を
   使い、scene独自DBを持たない。
3. **Snapshot generation**: Document変更は新しいimmutable published snapshotへ移り、旧renderは旧世代を
   読み切る。scene invalidation frameworkを追加しない。
4. **Bounds semantics**: M4の2D RoD／RoIとM5の3D AABBは同じ型に潰さない。ただし`Unknown`で除外せず
   保守的に広げるfailure policyは共有する。
5. **Vello lifecycle**: device、renderer、straight→premul adapterを一つだけ所有し、M4 SVG、M5 text／overlay、
   M3 native UIが別々に初期化しない。

## 6. 反対側レビューで再判定する問い

1. `glam`のWebGPU projectionとMotolii正準座標の写像は一意か。negative scaleとparent chainの反例は何か。
2. `gltf` crateのfeature／dependency closureはHost-resolved resource境界を保てるか。
3. validator／sample corpusが通っても未検出になるresource exhaustion、色、animation、axisの負例は何か。
4. `parry3d`は責任を減らすか。AABB＋triangle rayだけなら既存依存内の小さいrouteが勝たないか。
5. PCG採択後もInstanceIdの永続意味を新設せず、private render identityに縮小できるか。
6. 既存Vello／text接合を共通化する変更がM3所有面を壊さず、一つのownerへ収まるか。
7. M4 resource／artifact routeが未確定でも先に閉じられるM5 fixtureはどれか。

## 7. 次の成果物と停止線

次は本票を反対側レビューし、各機構classを`REUSE / ADOPT / WRAP / PATTERN / EXTERNAL / REJECT`へ
再裁定したM5採択地図を作る。地図は具体version／API、Motolii target、owner、failure mapping、fixture、
非証明範囲、並列可能なwrite setを持つ。

次のいずれかではruntimeへ進まない: Document／公開plugin契約／永続形式へ外部型を漏らす必要がある、
新しいworld／engine／scheduler／cache ownerが必要に見える、M4共通接合部が未裁定、unsupported glTFを
silent fallbackする、previewとexportを別routeにする、色変換をpost内部へ増やす。

## 8. 一次資料

- [glam 0.33.2](https://docs.rs/glam/0.33.2/glam/)
- [gltf 1.4.1](https://docs.rs/gltf/1.4.1/gltf/)
- [Khronos glTF Validator](https://github.com/KhronosGroup/glTF-Validator)
- [Khronos glTF Sample Assets](https://github.com/KhronosGroup/glTF-Sample-Assets)
- [Cesium obj2gltf](https://github.com/CesiumGS/obj2gltf)
- [wgpu 29](https://docs.rs/wgpu/29.0.4/wgpu/)
- [parry3d 0.29.0](https://docs.rs/parry3d/0.29.0/parry3d/)
- [rand_pcg 0.10.2](https://docs.rs/rand_pcg/0.10.2/rand_pcg/)
- [rend3 repository archive](https://github.com/BVE-Reborn/rend3)

発明禁止と採択地図の原則は既にreview済みなので、その原則をOpusへ二重相談していない。本票の新しい
候補判断は一次資料と現行codeへ限定し、反対側レビュー前の`比較中`として保持する。
