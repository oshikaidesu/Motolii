# 環境(HDRI)と面の応え方 — 空・鏡面・Glass

利用者裁定(2026-09-07): HDRI は AE の Environment Layer と同じく**画像層の属性**。rerun は照明を持たないので、fork の re_renderer に「環境 texture を受け取る口」を 1 つ足し、写像・畳み込み・陰影の式は借り物にする。同日に鏡面・Glass・exr まで進め、途中で見つけた仕様の穴は応急処置でなく普通に直した。

## 使い方

1. `.hdr` / `.exr` を Media に入れる(普通の画像と同じ札)
2. Timeline に置き、Inspector で **Environment** を On
3. 同じ comp の網(mesh)がその空で照らされ、背景にその空が敷かれる。重ね順で一番上の環境層だけが効く
4. 網の層に効果棚の **Glass** を載せると、Refraction・Roughness・Transmission・Metallic で面の応え方が変わる(Metallic 1・Roughness 0 で鏡)

## 中身(家ごと)

| 家 | 何を | 正本 |
|---|---|---|
| 門 | image crate の `hdr`・`exr` feature。拡張子の表は増やさない | root `Cargo.toml` |
| render | 画を線形 f32 で復号し Rgba16Float で上げる(1.0 超を潰さない)。64×32 に縮めて余弦畳み込み(照度 32×16)と Phong 葉 5 段(鏡面 128×64×5)を CPU で作る。背景の放射輝度は幅 2048 まで | `render/compositor/environment.rs`、`engine/texture.rs` |
| doc | 層属性 `environment` | `store/attrs.rs` |
| 棚 | 表面効果 `motolii.glass` は manifest 付き WGSL の 1 file(同日の最小コア C で `store/material.rs` から移した) | `render/vism/glass.wgsl` |
| fork | `Environment`(放射輝度・照度・鏡面 atlas・世界からの回転・強さ)を group(0) に bind。`GpuMeshInstance::surface`(粗さ・金属・透過・屈折率)。網は Lambert + split-sum 鏡面 + 屈折透過、`GenericSkyboxType::Environment` が背景 | `re_renderer/src/environment.rs`、`shader/utils/lighting.wgsl` |
| ui | Inspector の Image 層に Environment トグル。Glass は Effects 棚に並び、欄は property | `panels/inspector.dart` |

## 借りた定規

- 等距円筒の写像: +Y 上・-Z 正面(Khronos glTF Sample Viewer の規約)。Motolii の世界は y 下・z 奥なので X 軸まわり 180° で写す(鏡像にしない)
- 拡散: 余弦畳み込みを π で割った照度(Ramamoorthi & Hanrahan 2001)。一様な空 L は L に畳まれる
- 鏡面: split-sum(Karis 2013)。prefilter は粗さ→Phong 指数 `n = 2/α² − 2`、環境 BRDF は解析近似(Karis 2014)
- Fresnel: Schlick。屈折率から `F0 = ((n−1)/(n+1))²`
- 製品先例: AE Advanced 3D の Environment Layer

## 途中で直した仕様の穴

- **3D の大きさが奥行きに掛からず球が円盤になる**: scale は x/y の 2 成分しか無いので、奥行きを持つ素材(網・点群)の置き方にその平均を掛ける(`depth_scale`、`spatial_placement_from_bounds` の両経路)。板の変換は Lottie/web の定規のまま
- **VRAM**: 8K の HDRI を Rgba16Float で持つと 256MB 食うので、背景用は幅 2048 で留める
- **`motolii-ui.sh dev 書類`**: 何も開いていない時だけその書類を開く
- 照度図の経度の継ぎ目: sampler を「経度は repeat、緯度は clamp」に
- **再生が止まる**(同日夜、利用者報告): 再生中に native が返す軽い status に comp の寸法が無く、Swift の render が「Document dimensions missing」を投げて Dart の cadence が止まっていた。軽い status にも `width`/`height`/`fps`/`durationFrames` を入れ、play → tick → 軽い status に寸法がある事を native の契約 test に
- **Browser から入れた `.hdr` が背景にならない**(利用者報告): `.hdr` / `.exr` を置いた時は最初から Environment を On にする(`media::ENVIRONMENT_EXTENSIONS`、板にしたければ Inspector で切る)。Media の分類に **HDR** を足した

## 契約 test

- fork: 写像の往復、一様な空の畳み込み、上だけ明るい空で上向き法線が明るい、鏡面 5 段が鏡→艶消しへ並ぶ
- render(実 GPU): 上白・下黒の空で「画面上端は白、下端は黒、正面向きの網は照度 1/2 の灰、属性を外すと固定灯」。同じ空で「艶消し < 鏡(上の黒を映す) < ガラス(下の白を通す)」。hdr / exr の 4.0 が復号後に残る。Glass が棚に在り pass にならない
- compositor: 等方 scale 10 で奥行きも 10

## 宿題(あえての制限ではない)

- 透過は環境だけを通す。背後の層の屈折・他の網の映り込みは未着手
- 環境の回転・強さの UI(データは持っている)
- 点群は法線が無いので照明が効かない。宿題(2026-09-07 利用者): 取り込み時に法線を持たせる(PLY にあればそれ、無ければ k 近傍 PCA。Hoppe 1992)。点群 renderer に法線を 1 本渡す fork の口が要る。継ぎ目 D と同じ回。先例: Relightable 3D Gaussians(ECCV 2024、点に法線と BRDF を持たせて再照明)
- 宿題(2026-09-07 利用者): 法線の前に、点群の凹凸を見やすくする **EDL(eye-dome lighting)** を棚の 1 枚として。画面空間の pass なので今の 2D の口(`vism/*.wgsl`)で書ける。先例: CloudCompare・Potree
- Repeater で増えても照明の bind は 1 回。増えるのは網の draw だけ(環境が上乗せする物ではない)

## 実装の受け取り

fork `oshikaidesu/rerun` branch `motolii/expose-wgpu-resources` の `09f304dd`→`0586cd5f`→`ead8df8b`。Motolii 側は同日の作業ツリー(未 commit)。
