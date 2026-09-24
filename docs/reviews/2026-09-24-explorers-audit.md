# 探索者の監査(2026-09-24)と Composer の採択

状態: **監査済み・採択は既決から一意な物だけ**。方式: 共通の憲法(Vism = cassette、View = 作品の観測、表現 graph は自由・実行は有限、Motolii は GPU 実行を所有しない、fork は最後の手段、画を減らして速くしない、gap を fixture で隠さない)を渡した探索者が証拠と patch 候補を出し、Composer が「既決から一意」なら採択、新しい意味は人間へ。

## F: Rust にまだ焼いてある表現(30 件)

| 分類 | 件 | 代表 |
|---|---|---|
| EXPRESSION_IN_RUST | 14 | 標準 material の文字列(`surface_program.rs` STANDARD_MESH/PICTURE/MOVED_PICTURE: 粗さ 1・IOR 1.5・`sun_shade` の facing_floor 0.5・場で動いた絵は無照明)、Turbulent Displace の CPU 写し(点群だけ、`noise.rs`)、Track Overlay 4 札の全部(`extensions/overlay.rs`・`engine/overlay.rs`)、Rope(剛性 60・減衰 6・`ROPE_PASS` の WGSL 文字列)、room-scope Block の欄を位置で読む(`engine/blocks.rs:266-274`)、物性→摩擦/反発の写像(`engine/physics.rs`)、light cookie 512²、場の格子 128/8px/4px、Plexus 線の定数、Group の影 32 環 |
| 宿主の fork に漏れた Motolii の意味 | 1 | `backdrop_levels_read`(material.wgsl の lod 曲線の写し)→ **採択済み**: Motolii 側へ移動(`07f5d731e`)。fork 側は次の fork 改訂で削除 |
| CONTRACT_CONSTANT / HOST_LOWERING | 9 | 2D の積み順 bias 0.02、Add は α 0、FOV/VIEW_SIZE の既定、太陽の投影、blend 番号 |
| UNSURE(製品判断) | 6 | 8-bit sRGB の stack(HDR が run 間で切れる)、Glass は Glass を見ない、generator は層の形で切る、Motion Blur の標本密度、点の既定径 |

採択判定:
- **一意に決まる(順に着手可)**: 標準 material の既定値を棚の Vism へ(a693456ba の続き)。room-scope Block の欄を manifest の名前で読む(隠れ ABI の解消)。`backdrop_levels_read` は済み。
- **gap として残す(新しい一般 primitive が要る)**: 点群に届く場は Turbulent Displace 1 種だけ(field Vism が点群へ届く経路が無い)。影は `strength` 1 欄だけ・cookie 512 固定。場の格子密度を Vism が宣言できない。
- **人間へ(意味)**: stack の HDR 化(Target/View/Backdrop が float を持てるか)、Glass が Glass を見るか、generator の切り方、Track Overlay/Rope を Vism 化する時の contract。

## G: fork の差分(上流 `2309184bbb` 比 +4750/−785 行、64 file)

| 分類 | 内容 |
|---|---|
| そのまま上流候補(UPSTREAM_FIX/SEAM) | `read_buffer`、`queue_commands`/`before_submit` の戻り値、`new_with_external_resolved` + texture import、compute pipeline pool、型の再 export、`instance_vertex_positions` |
| 手直し後に上流候補 | mipmap 生成(pool を通していない) |
| fork に残る一般 primitive | ClipPlane、surface/field/motion/tint の hook と `params[24]`、group 0 の View 資源(environment・backdrop・view_capture・coverage・motion・program_constants・near_fade)、DrawOrder/`new_ordered`、`select_source_instances`、material の旗、CurveFill |
| **上流の挙動を変えている(上流に出せない)** | mesh が MSAA 標本ごとに shading(既定)、不透明 mesh を 2 回描く(depth pre-pass)、頂点 α<255 で透明扱い、rect の outline mask が texture α に従う、`params: [0.0; 12]` のままの bench/例が build できない |
| **死んでいる** | `new_layered`/`new_clipped`、`MeshProgram*` 互換 alias(互換 wrapper は既決で禁止)、`compose_mesh_program_source` 再 export、`surface_thickness`(常に 1.0)、`roughness_to_lod`/`equirect_uv_from_direction`、lib.rs の未使用 re-export、Motolii 側 `Engine::with_device` |

採択判定:
- **採択済み**: Motolii 側の死んだ `Engine::with_device`・`with_device_using_headless_defaults` を削除(`07f5d731e`)。
- **一意に決まる(fork 改訂で一括)**: 死んだ API の削除(約 45 行)、`surface_thickness` の削除、`main_target()` + その試験(Motolii 側 6 行の書き換えで不要)、`new_from_device`(adapter を渡せば上流の `RenderContext::new` で足りる、Motolii 側 10 行)、`backdrop_levels_read`(Motolii へ移動済み)、environment.rs の CPU 補助関数を Motolii へ。
- **候補(比率は低いが fork が最も縮む)**: paths module(約 1000 行)を Motolii 側の lowering へ(`into_mesh` が既に上流の `CpuMesh` を出す)。`draw_into` を上流の `draw()` + `queue_commands` で置換。
- **人間へ**: `layer_sort_key`(消すと opt-in していない描画物との順序が変わる)、`SurfaceSampling::FilteredPixel`(production は使っていない、試験だけ)。

## B: 1 コマ約 250 の GPU pass の分解

- 家系ごと(view-run 10・plate の bake 2・View の面の run 48・sky 7・合成の mix 66・Glow の鎖 26・海の Caustic 14・View atlas の mip 20・backdrop の mip 7・blit 20・shown/composite 2)を、どの code が出すか、view ごと/run ごと/plate ごと/コマに 1 回かで分類。
- **発見**: 本番の準備経路(incremental)では `observed` が立たず、View は plate をカードとして見ていた(Plate の run = 0)。裁定 B と食い違い → **修正済み**(`9e2c65f79`: scene 自身に `asks_for_views` を問う)。
- 採択(code から一意、`9e2c65f79`): 環境だけの run は builder も mix も作らない(−14 pass)、後で誰も読まない非 glass の絵を更新しない(−7)、canvas と同じ形式の screen pass 結果は COPY しない(−7)、spill の coverage は padded 済みの元を使う(−6 blit)。
- 採択(D、`eafbf7144`): View atlas の mip を宣言した段数だけ(20 → 2)。
- **裁定待ち**: rect の per-draw blend(Screen/Add を固定機能で。fork の GENERIC_PRIMITIVE + 「混色の単位は下を読む」の裁定)→ Heart の alone 4 本/面が run に入る。海の Caustic を観測者ごとでなく層に 1 回描く(意味の seam: 効果は層に塗るか観測者に塗るか)。
- fork の seam(UPSTREAM_SEAM、今回はしない): 面を array layer へ直接描く(複写と mip の bleed が消える)、stack を shown 無しで composite、SRC_OVER の run を stack へ load で描く(MSAA 4x では疑わしい)。

## C: plate の中身の mesh instance を stack 間で共有(patch は `explore/C-shared-plate-meshes`、保留)

- 形は `SharedMeshScene` の先例どおり(`PreparedMembers.meshes`、コマに 1 回)。試験 35 通過、gate PASS、契約試験 1 本追加。
- **Prism Garden では利得 0**(2178 → 2178): 先例の `shared_mesh_scene` が「透明な材質の mesh は共有しない」と拒む(OBJ の材質が透明扱い、板は Glass)。透明 instance の共有 = **seam**(透明物の順序は目のもの、という判断を崩すか)。裁定が出れば patch はそのまま載る。

## D: `VIEW_BLUR`(採択、`eafbf7144`)

- Backdrop の `BACKDROP_BLUR` と 1 対 1: 名前の無い欄は名前で拒む、宣言無しは全段(Cube Mirror・CCTV は不変)、`view_sample` は host が作った段数で clamp(`program_constants[6].y`)。
- 発見: fork の `backdrop_levels_read(1.0, 11)` は 7(粗さ 1 = 全段ではない)。宣言無しは `Option` で「全段」。
- seam(Composer が「絵を動かさない」で決めた): 宣言する Vism の lod の写像は Vism 自身の式のまま(Prism View は `roughness * 5`)、host の曲線 `view_lod()` は棚に置く。

## H: rect の draw 単位 blend(fork の patch 候補 `explore/rect-blend`、**不採択**)

- 候補: `RectangleOptions.blend: PremultipliedBlend { Over, Plus, Screen }`、blend ごとの pipeline(fork +233/−71、re_renderer の試験 67 本 green)。式は正しい(premultiplied の Screen = Cs + Cb − Cs·Cb、α = αs + αb − αs·αb は wgpu の `One / OneMinusSrc` で厳密)。
- **同値でない**(pixel の反例、`a_mix_blend_reads_the_picture_below_across_runs`): run の中で draw 単位に混ぜると相手は **その run の canvas** だけ。下の層が別の run(例: 3D の層、glass の境)にあると読まれず、半分の赤の Screen が 3D の青の上で `[188, 0, 187]`(正解 `[188, 0, 255]`)。旧経路(Alone → 中間 → mix pass)は stack 全体を相手にする。同値になるのは run を stack へ直接描く(MSAA target への load)場合だけで、それは別の UPSTREAM_SEAM。
- 上流の既存 contract で足りるか: rect は pipeline 1 本の固定 blend、mesh も同じ。draw 単位の blend 口は無い。fork の gate も「一般 primitive の family に無い」5 件で FAIL。
- 結論: 裁定「blend を理由に run を分けることを意味論にしない」は保つが、実現は固定機能 blend では無い。run の分離は現状維持、Prism Garden の現在の画には alone の run は無い(0/コマ)。

## I: View の面の中の花弁 1 枚が 0.1〜0.3 ms かかる理由(証拠)

- 重なりではない(輪の花弁の重なりは 1.02〜1.11 倍、面の 5〜24 %)。**画素あたり 65〜210 ns**: 楕円 1 つ = 4 本の 3 次曲線 → tolerance 1e‑5·extent で約 40 本の 2 次曲線、bbox 四角形の全 fragment が 40 本を走査(1 本につき依存する `textureLoad` 2 回 + 約 65 ALU)、latency 律速(ALU 利用率 約 8〜10 %)。細い花弁(Inner、5〜13 px)は 2×2 quad の無駄で約 2 倍。shading(IBL)の取り分 ≈ 0(Studio Light を隠しても差 0)。
- fork の既定 `SurfaceSampling::Sample` で fragment shader が **MSAA 標本ごとに 4 回**走る。曲線の被覆は texcoord(中心補間)から計算するので 4 回とも同じ値。
- 面ごとに繰り返している view 非依存の仕事: 曲線の被覆・gradient・拡散(法線は instance で一定)。View 依存: 投影・raster・AA 幅・specular の view_dir。
- 候補(絵を変えない): fork の CurveFill 内部の帯表で走査本数を 5〜10 分の 1 に(推定 −18〜22 ms、**K で patch 候補 + pixel oracle**)、curve data の詰め込みと展開(−8〜13)、bbox より狭い hull(−4)。
- **人間へ(品質の seam)**: 曲線塗りの quad を画素ごとに実行(標本補間の入力に標本ごとの情報が無い、推定 −18〜20 ms)、tolerance 1e‑5 → 1e‑3(40 → 12 本、≤ 0.1 画面 px、推定 −15〜19 ms)、x/y 2 本の ray を 1 本に(縁の AA が変わる)。
- 計測の限界: M4 の xctrace では shader profiler / GPU counters が空(Xcode の GPU frame capture が残る道)。

## J: plate の bake の複写 2.2 ms(証拠 → **測り方の誤り**)

- 複写そのものは 0.07〜0.13 ms(1080p RGBA8、約 16 MB)。2.2 ms は Metal が compute channel の copy kernel を次の command buffer の fragment pass と同時に走らせ、fragment に押されて区間が伸びた見かけ。ラベルごとの区間の総和(`gpu_owners.py`)は channel の重なりを二重に数える。critical path(`explore/J/critical.py`): frame 73.0 ms、fragment busy 66.1、compute busy 5.4(全部 fragment の中)。
- zero-init も sRGB の再解釈も無し(両側 `Rgba8UnormSrgb`、全面複写)。pool の churn(1 コマ約 10 枚の生成/破棄、re_renderer の 1 コマ retire)はあるが原因ではない。
- 採択: `record_picture` は `into` 無しなら stack をそのまま絵にする(複写と 8 MiB の pool churn 1 枚が消える、実費 0.1〜0.2 ms)。
- 人間へ: pool の retire を N コマにする(fork の memory residency の contract)。
- 副産物: wgpu-core 30 は pass ごとに Metal command buffer を 1 本作る(1 コマ約 650 本、`vism-pass` 分の driver CPU 5.4 ms)。CPU の pass 比例費用の正体。

## K: 曲線塗りの帯表(fork `66a439b20f`、**採択**: ESTABLISHED_RENDERING_TECHNIQUE の裁定で push・pin)

- Lengyel 式の行/列の帯表を upload 時に CPU で作り、同じ data texture に置く。fragment は自分の行帯と列帯の曲線だけを走査(曲線ごとの式と順序は不変)。公開 API 不変、fork の試験 66 + oracle(楕円・五芒星・5 px の薄片・穴・gradient、64² と 512²、18 枚の参照 PNG)で **0 画素差**。
- Motolii 側 A/B(file:// pin、push なし): Prism Garden 1080p の t=0 を別 build の before と比較して **2,073,600 画素すべて一致**。全試験は基準と同じ失敗 8 件。**GPU 71.2 → 39.2 ms**。CPU +2 ms は帯表(約 0.4 ms)ではなく、path mesh を毎コマ作り直す既存の仕事(`path_model` → `into_gpu_meshes` 約 1.1 ms)。
- K は最終形ではない: M の順位では「帯表 → 曲線塗りの画素ごと実行(縁の品質 seam)→ coverage mask pass + atlas(View 間で共有)」、または Vello 系の strips。P(2026 の prior-art 監査)の結果次第で CurveFill 自体を捨てる可能性を残す。

## M: 品質 rendering の調査(vector raster)

- 現行方式の異常さ(業界比): fragment で全曲線走査(Slug が最初に最適化で消す baseline)、標本ごとに材質まで再実行(UE/Skia/Rive/Slug は画素ごと)、coverage の前に paint を計算、coverage と shading が 1 つの program に融合(StC・Rive・Vello・Pathfinder・Graphite は全部分離)、仕事が bbox × 標本に比例(他は縁や tile に比例)。
- 対応表(抜粋): 帯表 = GENERIC(K)。bbox → 8 角形の cover polygon。coverage mask の DrawData(view 空間、または shape 空間の atlas を revision で共有 → 6 面の View が coverage を 1 回で済ませる)= GENERIC「Coverage」資源。`SurfaceProgram` ごとの標本率(曲線塗りは centroid、glass は sample)。paint を coverage の後に(bit 同一)。Thin Translucent 相当の dual-source blend(透過色と反射を 1 pass、glass の分散は Vism 側)= UPSTREAM_SEAM。TAA/TSR は不要(解析的 AA が既に決定論的)。
- 順位: 1 K(同一)→ 2 K + 曲線塗りの画素ごと実行(縁の seam)→ 3 coverage mask + atlas → 4 Loop–Blinn(縁の品質は Rive 級)。stencil-then-cover は 4× MSAA では品質が下がるので却下。

## 品質 rendering の調査(2026-09-24、調査のみ・実装なし。ESTABLISHED_RENDERING_TECHNIQUE の裁定下)

### N1 Material(現行 BSDF が持たないもの)
- 厚みが無い: `SurfaceIn::thickness` は instance の scale(板は約 1 で屈折量ほぼ 0 = 「透明な四角形」、球だけ屈折する)。Extrude の深さはシェーダに届かない。Beer–Lambert・第 2 界面・全反射・内部反射が無い。単散乱のみ(多重散乱の補償無し、拡散の重みが Fresnel を無視 → 「黒いプラスチック」)。Fresnel は n·v の Schlick だけで F90 も薄殻の 2 界面も無い(縁の線に見える)。specular AA 無し。粗さの空間変化の入力が無い。粗い屈折は backdrop の mip だけで、宣言した粗さ 0.05 では **11 段のうち 2 段しか host が作らない**(ぼけの予算が無い)。薄膜は cos の虹の縁で Fresnel 項ではない。
- 上位 5(見た目の効きの順): ①実厚み + Beer–Lambert + 薄殻の 2 界面 Fresnel(fork の GENERIC: instance に幾何の厚みと slab/sphere の bit、doc に厚み・減衰色/距離)、②多重散乱 + エネルギー整合の誘電体(Vism のみ 15 行、Fdez-Agüera 2019)、③specular AA(Tokuyoshi–Kaplanyan)+ 粗さ依存 F90 + 第 2 lobe、④粗い屈折(IOR 依存の cone、`BACKDROP_BLUR` を実効粗さで)、⑤薄膜を Fresnel 項に(Belcour–Barla、KHR_iridescence)。先例: KHR_volume/transmission/iridescence/clearcoat、Filament、three.js、OpenPBR/Standard Surface、EEVEE Next。
### N4 Image formation(**HDR は run の前段(ROP)で既に潰れている**)
- ViewBuilder の main target が `Rgba8UnormSrgb`(fork `view_builder.rs:411`、`new_with_external_resolved` が同形式を強制)で、最初の描画から [0,1] の 8 bit。stack・backdrop の mip・View の絵も 8 bit。`composite.wgsl:83` の `saturate` が事実上の tone mapper、exposure・view transform・dither 無し。glow は float だが入力が ≤ 1(閾値 0.6 は「明るい粥色」)。現行画像の実測: 全 chan が 255 の画素 4.5 %、真っ白 0.09 %(1 chan だけ飽和して色相が飛ぶ = 6 原色への崩れ)、黒 0 %(輝度 1 %tile = 45/255、海の床が浮いている)。
- 上位 5: ①float canvas の policy(fork の main target 形式 + Motolii の `BLEND_TARGET_FORMAT` → `Rgba16Float`、`saturate` 除去。土台)、②view transform の Vism + exposure(AgX 既定、ACES/PBR Neutral、dither 込み。**expression は Vism/document、host は「最後の Vism が出力変換」の契約だけ**)、③glow を radiance 域へ(閾値・spill を add)、④HDR 対応の MSAA resolve(fork の GENERIC: 可逆 tonemap resolve、fork 自身のコメントが既に指摘)、⑤export の 2× supersample + float で downsample。TAA/TSR と auto-exposure は不要。
### M Vector raster / 2026 の先例(**Vello sparse strips が現実の選択肢**)
- 現行の fork の異常さ: 曲線の全走査(Slug が最初に消す baseline)、標本ごとに材質まで、coverage と paint と material が 1 program、仕事が bbox × 標本に比例。他(Vello/Rive/Graphite/Pathfinder)は coverage を画素ごと・縁だけ・paint と分離。
- **Vello GPU(sparse strips、0.10.0 = 2026-08-14、main は 09-24)**: CPU が flatten → tile → strip、GPU は strip の画素だけ。wgpu 30(fork と同じ)、呼び出し口は「caller の TextureView へ render」。fork の `paths.rs` の exact fill(`CurveFill`・`quadratic_curves`・gradient ramp・curve loop)を **丸ごと捨てられる**。ただし: 平面/アフィン前提(斜めの View では View 解像度で再 raster が要る)、API 安定性の保証無し、MSAA target には直接描けず非 MSAA の Picture を挟む(seam は「外部 pass が texture を提供」)。Motolii の wgpu 29 pin を 30 に上げる必要。
- 順位: 1 Vello GPU(捨てる量が最大)/ 2 coverage mask pass + 画素ごとの shading(`Coverage` 資源)/ 3 K(帯表、採択・pin 済み)/ 4 Loop–Blinn(縁が今より落ちる)/ 5 標本率の per-program 化。
### Rerun upstream(2026-09)
- upstream `main` は wgpu 30、0.38.1(09-16)。**環境光/IBL・vector path・tonemap・HDR・mipmap・clip・標本ごと shading は上流に無い**。sorted transparency と custom `Renderer` の registry(`renderers_mut().register`)、group 1 = phase data(scene depth)、3D texture、Gaussian splat・voxel・volume raymarcher が入った。`re_renderer` は `crates/viewer/` → `crates/viewer_support/` へ移動済み(fork は既に新 path)。`Renderer` trait が `DrawInstruction` 形へ、`DrawPhase` に `Volume`。→ fork の一般 primitive のうち上流が持っていない物は依然として fork にしか居ない。

### N2 Lighting(現行が持たないもの)
- 環境は LDR の PNG(≤ 1.0、`.hdr/.exr` だけが float)、irradiance 32×16、radiance の mip は **2×2 box 平均(GGX の prefilter ではない)**、DFG LUT・多重散乱・specular/horizon occlusion・AO 無し。**直接光の sun 項が無い**(sun は影を引くだけ)。影は 512² の cookie(depth 無し)で、**Prism Garden には Cast Shadow の層が無く影は 0**。**発光層(Core・Heart)は何も照らさない**(絵として View に写るだけ、`shade_surface` は環境 texture しか読まない)。局所光・area light 無し。Caustic は海の矩形に描く 2D の pass で、レンズとは無関係。exposure・tone mapping 無し。
- 上位 5: ①Core/Heart を光にする(発光層 → area light + View を SH へ畳んで局所 probe。Frostbite §4.8、Lumen の emissive)、②HDR 環境 + exposure + 直接 sun 項(EEVEE の world sun 抽出)、③GGX prefilter の radiance + DFG LUT + 多重散乱 + dominant direction(upload 時に 1 回、コマ費用 0)、④depth/normal の capture → AO + specular/horizon occlusion + 柔らかい depth shadow、⑤lens の screen-space caustics(JCGT 2026 Newton 法、light-view の G-buffer)。
### N3 Optics(「宝石」感)
- `thickness` は placeholder(rect は 1.0 固定・Motolii は一度も設定しない、mesh は x 軸の scale)。**Extrude の深さ(doc に在る)が shader へ届かない**。TIR の場合に反射方向の backdrop を読んでいる(誤った絵)。第 2 界面・Beer–Lambert・内部反射・分散が経路長と結び付かない。
- 上位 5: ①instance ごとの実厚み + slab/sphere の形 flag(host + document、小)、②2 界面の出射 + Beer–Lambert(Filament/EEVEE/KHR_volume の解析式、Vism のみ)、③Fresnel 重み付き内部反射と正しい TIR(Environment/View を使う)、④分散を 2 界面の経路の上に置き直す + 粗さの LOD ∝ 厚み、⑤sun cookie での面積比 caustic(受け面距離の定数を 1 つ)。fallback: Wyman の back-face capture(fork の GENERIC、大)。
### 2026 の既製部品(2D vector と周辺、要約)
- **Vello の状況**: sparse strips は `vello_hybrid 0.2.0`(2026-08-07)→ `vello_gpu` へ改名中(crate 名は予約のみ)。released は wgpu 29、**wgpu 30 は main のみ(2026-09-15)**。beta 品質(mask layer・一部 blend・複雑な filter は panic、API 安定性の保証なし)。HDR 無し、2D アフィンのみ、MSAA 無し(解析的 AA)。`TargetInit`/depth は main のみ。**MSAA を持つのは古典の compute Vello(0.10.0、research 扱い)だけ**。
- Rive Renderer は C++ で Rust/wgpu の経路が無い。Skia Graphite は rust-skia 0.153 で Metal が使えるが wgpu と device を共有する公式手段が無い。Pathfinder は死んでいる。lyon は AA 無し。femtovg 0.27 は wgpu 30 だが stencil-and-fringe(正確な coverage では無い)。**成熟した Rust/wgpu の置換部品は Vello 系だけで、それも今は beta**。
- SIGGRAPH 2025–26: Metal で動く物は mip/FFT bloom、HypeHype の stochastic tile lighting(pixel shader のみ、RT 不要)、PPLL/WBOIT/MBOIT の透過、Hable の compute tessellation、Vello の sparse strips。MegaLights・idTech8・ORCA は RT 前提。wgpu の Metal ray query は 2026-02 に入ったが open bug が 2 件(#9100、#9215)で、product の毎コマ経路には不向き。NVIDIA の neural shading SDK は Metal/wgpu では動かない。Bevy 自身が ReSTIR を既定で切った。OIT は AE 風の層には不要(painter's order が意味)。bloom は Bevy の `bloom.wesl`(COD 方式)が WESL の先例で、Motolii の Glow と同じ。

### wgpu 30 に載る既製の PBR 部品(P の生態系調査、2026-09-24)
- **Bevy(MIT OR Apache-2.0)は「部品庫」として使える**: `environment_filter.wesl`(GGX の VNDF 重点サンプリングによる radiance の prefilter + 32×32 の irradiance cubemap、SPD の downsample、Rgba16Float、Metal で動く。ただし Bevy は毎コマ走らせる — Motolii は upload 時に 1 回)、`tonemapping.wesl`(ACES Hill・AgX の LUT 版・PBR Neutral・TonyMcMapface・Reinhard)、`bloom.wesl`(COD 方式、Motolii の Glow と同じ)、OIT の線形リスト(MSAA 不可)、多重散乱(Fdez-Agüera 2019)と DFG LUT の両方。StandardMaterial は transmission・volume(厚み・減衰)・ior・clearcoat・anisotropy・specular を持つが、**sheen・iridescence・dispersion は wgpu 系のどの engine にも無い**(Khronos の GLSL が正本)。
- 版の摩擦: Bevy 0.19 は wgpu 29 + naga_oil の `.wgsl`、**0.20-rc は wgpu 30 + WESL 0.5**。Motolii は wgpu 30・`wesl = 0.4.2` 固定なので、0.5 の構文/可視性の差がある(0.19 の `.wgsl` は naga_oil 方言で結局書き直し)。lift の単位は `tonemapping.wesl`・`bloom.wesl`・`environment_filter.wesl` が素直で、`pbr_fragment`/`pbr_functions` は Bevy の bind group に溶接されていて重い。
- tone map の単体部品: `dmnsgn/shaders-tone-map`(MIT、WGSL、AgX 解析式・ACES Hill・PBR Neutral など 13 種)が最も素直な drop-in。TonyMcMapface は LUT が要る(Apache/MIT)。GT7(SIGGRAPH 2025)は唯一 HDR 表示を意識した curve だが WGSL 版無し。
- **HDR/EDR の出力は wgpu 30 が既に持つ**(`SurfaceColorSpace::ExtendedSrgbLinear`/`ExtendedDisplayP3`、`Surface::display_hdr_info()` に EDR の headroom、Metal で動く)。float canvas ができれば「値 > 1 を Apple のディスプレイへ」まで届く(注意: Metal の sRGB が colorspace = nil になる不具合の修正は 30.0.1 に未収載)。
- 使えない/避ける: rend3(archived)、three-d(OpenGL)、Solari(DLSS-RR のみ)、DLSS、wgpu-ffx(実験、wgpu 29、SPIR-V)、MetalFX(wgpu に API 無し)、OCIO は `ocio-rs` が MSL text までで WGSL 無し。IBL の prefilter を CPU で焼く `ibl_core`(MIT)もあるが、Bevy の GPU 版の方が素直。

### Q: 2024–26 の real-time 研究(wgpu 30 / Metal の forward renderer に載る物)
- 前提: Metal で使えるのは compute・storage・atomics・`DUAL_SOURCE_BLENDING`・`SUBGROUP`・`SHADER_F16`・`TIMESTAMP_QUERY`・HDR surface(`ExtendedSrgbLinear`)。**RT(ray query/pipeline)は Vulkan/DX12 のみと読む資料があり、Metal は experimental で open bug 2 件(#9100・#9215)**(探索者の間で記述が割れている: wgpu の docstring が古い)。**ROV/pixel-local storage/tile shader は wgpu に無い**。coop-vector・MetalFX は wgpu から届かない。
- **この scene 種(球・板・花弁)は解析形/SDF で書けるので、「解析 occluder buffer + WGSL `trace()`」を 1 つ持てば RT 前提の研究(DDGI/SDFDDGI・Radiance Cascades・ReSTIR 系)の多くが RT 無しで動く**。
- 順位(効果 ÷ (コスト + fork 表面)): ①rough refraction = scene-color の mip pyramid + slab/sphere の解析 2 界面(Substrate/Frostbite、コスト低)、②specular AA(Tokuyoshi 2021 の射影空間版、コスト ≈ 0)、③GGX prefilter の IBL(または HPG 2025 の Spherical Harmonic Exponentials)、④EDR 出力 + headroom を見る view transform(PBR Neutral / AgX / ACES 2.0 を LUT 化、ACES 2.0 は Apple silicon で shader が遅いので LUT)、⑤vector 層の Vello sparse strips(3D の後に合成)、⑥glass 層の Moment-Based OIT(ROV 不要)、⑦発光の SG area light(Tokuyoshi 2024)、⑧SDF DDGI の probe(拡散の相互反射)、⑨Newton 法の screen-space 屈折(JCGT 2026、vector 背景の屈折)、⑩physically based bloom/glare(separable は Glow に既にある)。TAA/TSR/MetalFX は vector 層に不適(文字と細線がにじむ)。時間方向の蓄積は 3D の glass 層に限る。
- **訂正(2026-09-24、利用者が一次資料で確認)**: PaRas は実在する。「PaRas: A Rasterizer for Large-Scale Parametric Surfaces」(Kechun Wang / Renjie Chen、SIGGRAPH 2025 Conference Papers、DOI 10.1145/3721238.3730658、167:1–167:9、著者実装 github.com/renjiec/Paras)。対象は quartic triangular / bicubic rational Bézier など高次の 3D parametric surface を GPU 上の Newton 型反復で直接 rasterize する手法で、**2D の CurveFill の直接代替ではない**が、将来の高品質な曲面 primitive の先例。Q の「一覧で見つからない」は検索の取りこぼし。

### 捨てられるもの / 残すもの(現時点の仮判定、P と R の結果で更新)
- **捨てる候補(証拠が揃った順)**: ①曲線塗りの自作(`paths.rs` の exact fill・`CurveFill`・curve loop)→ Vello GPU sparse strips への thin seam(ただし beta・wgpu 30 は main のみ・斜めの View は再 raster が要る)。当面は K(帯表、採択済み)で持たせる。②radiance の 2×2 box mip → GGX prefilter(Filament/Frostbite の標準)。③Motolii 側の Karis 近似の env BRDF → DFG LUT。④8 bit の run stack → float canvas。⑤自作の tone(saturate だけ)→ view transform(AgX)。
- **残す(上流に無い一般 primitive)**: surface hook、View/Backdrop/Environment の per-view 資源、DrawOrder、ClipPlane、mipmap 生成、外部 texture の取り込み、data texture。上流は 2026-09 時点でも IBL・path・tonemap・HDR・mipmap・clip を持たない。
- **人間が決めること(生き残る seam)**: 何を Vism から操作可能にするか(厚み・減衰・薄膜・exposure・view transform の選択)。**裁定済み(2026-09-24)**: Emission と Light Contribution は分離。発光は scene-linear HDR radiance(> 1.0 可)で、bloom/glare や反射・屈折から明るい物として観測される。周囲を照らすのは別の generic 能力で、Emission だけを理由に自動で照明/GI にはしない。N2 の「Core/Heart を光にする」は Light Contribution の realization 候補として別 lane に置く(seam: 新しい Motolii semantic として公開する必要が生じた時だけ報告)。

### 待ち: Cycles の参照 oracle(R)、P の最終報告(wgpu 生態系の PBR/IBL/tonemap の部品)、Q。目標画像が届けば、5 班の対応表に花弁・板・球・ハイライト・暗部の個別比較を載せる。

## Quality migration の正本と、捨てられる自作の地図(利用者の整理 2026-09-24、調査に基づく)

正本の並び: **Khronos PBR の語彙 → Filament/EEVEE の realtime 実現 → Cycles の oracle → Vello/Rive(vector)→ wgpu の HDR 出力**。

- **Material の意味は Khronos に既にある**: transmission・volume(thickness・attenuation・refraction・absorption)・ior・specular・clearcoat・sheen・iridescence・anisotropy・dispersion(2024 から正式)・emissive strength。Khronos glTF-Sample-Viewer は float framebuffer・HDR environment・transmission・volume・volume scattering・iridescence・dispersion を実装済みで reference になる。Standard Glass の意味を Motolii が発明する必要は薄い。
- **Filament は realtime の教科書**: transmission・absorption・thickness/microThickness・ior・dispersion・iridescence・clearcoat・anisotropy・emissive、refraction は cubemap(遠景の安い近似)と screenspace(scene の物まで屈折)の 2 種で、Motolii の Environment + Backdrop と同型。IBL は roughness の prefilter・DFG LUT・多重散乱 GGX・拡散の irradiance/SH の生成道具を持つ。post は HDR bloom・AgX・PBR Neutral・ACES・filmic・gamut mapping・exposure・white balance・color grading・TAA/FXAA/MSAA。
- **HDR は wgpu が持つ**: `ExtendedSrgb`/`ExtendedSrgbLinear`・DisplayP3・HDR10 PQ・HLG と `Surface::display_hdr_info()`(輝度・EDR headroom・primaries・bit 深度)、Metal/macOS 実装済み。scene-linear float → view transform → EDR の経路が正式に想定されている。
- **Vector は Vello GPU(sparse strips: CPU で path/tile/strip、GPU で strip raster + 合成、wgpu backend、compute 不要)が本命候補**、Rive(Bézier を triangle patch に幾何的に還元して標準の rasterizer で描く)が別解。
- **Cycles/EEVEE の分担**: Cycles は path tracer(oracle)、EEVEE は realtime、両者は shader node を共有。意味は Cycles 級、実現は EEVEE 級。Cycles は Emission を「見た目の発光」と「light として sampling するか」に分ける(Emission Sampling: None/Auto/Front/Back)→ 2026-09-24 の Emission ≠ Light Contribution の裁定と一致。
- neural appearance(NVIDIA の Real-Time Neural Appearance Models)・Falcor(NRD 付きの denoised real-time path tracer)は研究資料。DX12/Vulkan 中心で今の Metal/wgpu へは入れない。

| Motolii/fork の現在 | 2026 の先例 |
|---|---|
| CurveFill の全 curve fragment 走査 | Vello GPU sparse strips / Rive Renderer(当面は K = 帯表で持たせる) |
| box 平均の env mip | Filament 式の GGX prefilter |
| Karis のみの env BRDF | DFG LUT + 多重散乱 GGX |
| `saturate` の出力 | scene-linear HDR + AgX/PBR Neutral + EDR |
| thickness が placeholder の Glass | KHR Volume/IOR/Transmission + Filament の realtime 実現 |
| 独自の分散・iridescence の意味 | KHR_materials_dispersion / iridescence を意味の正本に |
| 発光の意味を発明 | KHR emissive strength + Cycles の light sampling 分離 |
| 8 bit sRGB の中間形式 | float scene color + wgpu の HDR surface |

Motolii 固有として残るもの: View / Plate / Repeater / 2D・2.5D・3D の composition / Vism の resource graph / timeline。**最初の実装は float scene-linear の pipeline**(8 bit のままだと後の GGX・Glass・Emission が頭打ち)。S(8 bit 経路の完全追跡)と R(Cycles 参照画像)の結果を待つ。

## 部品取りの結果(T1〜T3、現物監査 2026-09-24)と、次の探索枠

- **T1 renderling**(0.6.0、MIT/Apache、wgpu 26・rust-gpu、shader は Rust → SPIR-V を naga spv-in で Metal へ、保守者 実質 1 人): 直接依存 0、削れる行 0〜約 200(IBL の標準アルゴリズム)。理由: wgpu が 4 メジャー遅れ、入力が crabslab の slab に結合、fork の pool/ViewBuilder へつなぐ seam が無い、曲線は lyon の CPU テッセレーション(fork の解析的 coverage より劣る)、OIT・SSAO は無い、transmission/volume/iridescence/分散は無い、tone map は ACES と Reinhard のみ(AgX/PBR Neutral 無し)。**負けた理由は「adapter が大きい割に消える物が少ない」であって「renderer を丸ごと/半分刺す発想」が悪いのではない**。
- **T3 Vello GPU**(main `a31f319`、wgpu 30.0.0 で動作): Rgba16Float への楕円 + radial gradient を実測、CPU は花弁 3000 枚で 1 view 約 4 ms(6 面を面ごとに作ると 約 24 ms)。**8 bit の中間形式で HDR 不可**(gradient は sRGB エンコード空間、1.0 超は保てない)、affine のみ・mask 無し・diamond gradient 無し・3D 押し出しや頂点変形無し。fork の曲線塗りは削れず、平面 2D の分(約 400〜600 行 + shader 約 70 行)だけが対象、当面は「速い経路の追加」。Penumbra は無視、rend3 は参考のみ。
- **判定式(2026-09-24 の方針)**: 採用の評価は接続コストだけにしない。**削れる量(fork の行数・shader の数・独自 material 基盤・public seam)と品質の増分の差引**で判断する。例: adapter +1,500 行 / fork −9,000 行 / WGSL −12 file / public seam −20 / 品質 ↑↑ なら勝ち。
- **次の探索枠(候補集めは利用者側、現物の照合は Fable 側)**: 「Rerun を host のまま、外部 renderer に共有の device/target を渡して描かせ、Rerun の Texture として合成する」経路を探す。外部 renderer が device まで所有する必要は無く、「この Texture にこの DrawData を描いて」だけ任せられればよい。条件: Rust/wgpu なら最良だが限定しない、共有 device/target へ描ける、render graph が分離可能、headless/offscreen 可、PBR/HDR/transmission が強い、shader 層だけでも使える、scene の所有を強制しない。分類: DIRECT EMBED / RENDER-TO-TEXTURE / SUBSYSTEM EXTRACTION / REFERENCE ONLY。
- **完成した renderer を読む価値(問いの入れ替え)**: 今の Motolii を基準にすると「Karis をどう改善するか」になるが、Filament を基準にすると「なぜ DFG LUT + 多重散乱 GGX でないのか」になる。Cycles なら「Glass の改善」でなく「なぜ volume boundary/thickness/absorption が無いのか」、Vello なら「40 curve の loop をどう速くするか」でなく「なぜ fragment が全 curve を読んでいるのか」。

## T2 Bevy PBR(部品取り、現物監査 2026-09-24)

- Bevy main `ad31a06` / 0.20-rc.1 は wgpu 30・`wesl 0.4.2`(`naga-ext`)で Motolii と同じ pin。WESL 0.5(PR #25768)は未マージの WIP。0.19.1 は naga_oil の `.wgsl`。
- `environment_filter.wesl`・`tonemapping.wesl`・`bloom.wesl` を `wesl =0.4.2` + `VirtualResolver` でコンパイル、naga 29.0.4 で検証して通った(naga 30 は未検証)。書き換えは import 行と binding 2 か所。
- Rust crate は ECS に結合していて依存にできない、再利用できるのは shader 文字列だけ。`mesh_view_bindings` を import する物(transmission・shadow・light probe・decal・OIT・`pbr_lighting` 全体・`pbr_functions`)は取れない。
- prefilter は fork の equirect(2D texture)用に書き直しが要る。DFG LUT の生成器はライセンス表記の無い gist なので自前(約 40 行)。AgX/Filmic の LUT はライセンス未確認。LTC LUT は論文の引用が要る。Motolii の `glow.wgsl` は既に Bevy の bloom の写し。
- **部品取り(A)としての数字**: 追加 shader 約 640 行 + Rust 約 150 行、削除 約 140 行。**この数字は旧前提(今の renderer を残す)の問いへの答え**で、下の前提変更で B を別に評価する。

## 前提変更: Destructive replacement(利用者裁定 2026-09-24)

今の renderer 実装は保存しない。守るのは上位 semantic(Document/Timeline、FrameGraph の依存、Vism、View、Plate/Flatten/Matte/Clip、Repeater、2D/2.5D/3D、composition order と blend、hot reload)だけ。候補ごとに **A Surgical** と **B Destructive** を出し、B は「その候補を正本にしたら今の Motolii/fork の renderer を何丸ごと消せるか」で評価する。**最終 owned complexity = 旧経路の削除 + adapter**、依存の内部 LOC は数えない。binding layout・Stage・material 構造の不一致は B の棄却理由にならない。移行は 新経路完成 → oracle/reference で検証 → 切替 → 旧経路削除、恒久の二重経路は作らない。T1〜T3 はこの基準で再評価する(U5: 所有コードの棚卸し = 分母、U6: Bevy/renderling の B、U1 Rendiation・U2 threers・U3 netrender/Vello・U4 接続雛形にも同じ基準を追記)。

## 現物監査の結果 U1〜U10(2026-09-24、破壊的置換の基準)

分母(U5): 消せる自前 renderer は **約 15,200 行**(Motolii Rust 約 9,530 + fork Rust 3,991 + shader 1,684)、fork 由来の public seam 38 + shader の hook 契約 1(`surface.wgsl` の `SurfaceIn`/`program_*` は守る契約だが fork の replaceable code の中にある → 置換時は移す)。守る semantic の芯は motolii-render 約 22,750 行 + motolii-doc 9,790 行。fork は upstream `2309184bbb` から 4 commit、+6,467/−835、shader 7 本。

| 候補 | pin | wgpu | A(部品) | B(置換) | 決め手 |
|---|---|---|---|---|---|
| **Bevy main/0.20** | `ad31a06` | 30、WESL 0.4.2 | SUBSYSTEM EXTRACTION(WESL module の輸入、安い) | **RENDER-TO-TEXTURE、spike 条件付き** | `RenderCreation::manual`・`ManualTextureView`・外部駆動 headless(公式 example)・hot reload(`pipeline_cache.rs:973`)・macOS は同期 compile。blend mode は無いので AE 合成は Motolii に残る(今の run 分割に写る)。Vism は 1 つの `VismExt` + `specialize(key)` で fragment 差し替え。消える: fork 約 2.6k + Motolii 約 0.8k、書き直し 2〜3k、残る: CurveFill/paths.rs(Bevy に path fill 無し)、mipmap、re_video。adapter 3.5〜5k、owned 行数は横ばいだが lighting/IBL/glass/tone の所有は 0。リスク: 3〜5 か月ごとの破壊的 release、wgpu 三者一致、TAA/auto exposure の履歴と seek/export、N camera のコスト未測定、bindless の device feature |
| threers | `dac5cd8` | 30 | SUBSYSTEM EXTRACTION(3D surface)/ path tracer は oracle | partly | 接続は最良(`Renderer::new(Arc<Device>, Arc<Queue>, …)`、`render(…, &TextureView, linear)`)。**だが material shader 末尾で `clamp(0,1)`(`shader.rs:1538`)、render-to-texture/glass/instanced/custom は MSAA 外(`renderer.rs:6018`)**、env BRDF は Karis 級、prefilter は CPU PMREM、raster の画は three.js 級(`material_chart.png` を手元で再現、README と同じ)。1 人・10 commit・28 万行(3 か月)。消える約 4.3k、adapter 約 1k。Vism の field/motion hook は vertex stage の patch が要る |
| Rendiation | `7f7471f` | 29、nightly | REFERENCE(ECS 非依存の post-fx/IBL prefilter の数式、reactive diff → GPU 資源の型) | REFERENCE ONLY | Device 注入無し(`GPU::new` が自作)、`.wgsl` 0(Rust EDSL → naga)、material は global ECS。1 人 |
| netrender | `aba7d83` | 30、MPL-2.0 | REFERENCE(`WgpuDevice::with_external` の型は 150 行で真似できる) | REFERENCE ONLY | 全 target Rgba8Unorm、作者自身が linear-light blending を Vello 上流待ちと明記、29k 行の殻は DOM paint list 用 |
| Vello 0.10 | `92e7cd3` | 30 | RENDER-TO-TEXTURE(平面 2D の速い経路) | B: 全 vector を Vello、CurveFill 削除 = fork 約 1,150〜1,200 行 + WGSL 90 が消え adapter 500〜900 | `Renderer::new(&Device)`、`render_to_texture` は **Rgba8Unorm + STORAGE_BINDING 必須、HDR 無し**、affine のみ、自前 AA(hardware MSAA と別)。3D 配置は「平面で描いて板に貼る」への意味変更が唯一の実質リスク |
| welding | `65d057d` | 30 | — | 先例のみ | Metal の IOSurface → MTLTexture → wgpu を zero-copy で確認。Vello は同一 device なので不要 |
| Penumbra | `e2c3f23` | **24**、lock 無し | REFERENCE ONLY | **REJECT** | 15 commit 全部 2026-04-03 の 1 日、9,131 行。pbr/post/shadow は config + `include_str!` 断片で読む場所 0。描くのは inline Blinn-Phong 1 灯、swapchain のみ、headless は Metal で panic。「crate 分割が綺麗」は箱が空だから |
| Impact | `49b7562` | 30、Apache-2.0 | REFERENCE + shader 文字列 4 片は DIRECT EMBED 可 | **REJECT** | 確立した技法の縦串: Hammon BRDF、DFG LUT(Gauss-Legendre 生成)、fp16 pre-exposed 蓄積、bruop histogram + SBS 露出、Jimenez bloom(threshold 無し)、ACES fit + **Khronos PBR Neutral**、Playdead/Karis TAA、Karis 球光源、PCSS/Vogel、Alchemy AO。だが **IBL 無し**(定数 ambient → 画が平板)、露出は毎コマ `poll(wait_indefinitely)` の同期 readback、`IMMEDIATES` 等の device feature 必須、Device 注入無し、material は Fixed/Physical の 2 種で user WGSL の入口無し、hot reload 無し、MSAA 無し。「画が成立する最小」= fp16 線形蓄積 + 蓄積前の露出 + 本物の tone curve を最後に + threshold 無し bloom |
| Fyrox HEAD | `b608961` | 30(backend)、GL 既定 | REFERENCE(`hdr_map` の Yxy/ACES、`prefilter`/`irradiance`、DFG LUT 256² 生成器) | **REJECT** | scene buffer が **RGB10A2 UNORM**(1.0.0 で rgba16f を捨てた)、IBL は RGB8、算術平均 luminance、フレーム依存の線形適応、luminance だけ ACES。`WgpuGraphicsServer::new` が winit 窓と Device を自作、headless 無し、wgpu backend は MSAA 1・occlusion stub。renderer は fyrox-impl の module で scene に結合 |
| rafx | `bcf91a2` | 独自 RHI、2024-08 から停止 | — | REFERENCE ONLY | render graph = usage id の read-after-write で順序・寿命・cull を決める。Motolii の `view.rs` の `later_reads_below`・手動 `unglazed/transmission/backdrops`・3 か所の alloc・`SameFrameCycle` DFS はこの型で消え、`RunBreak` は「なぜ」の意味だけ残る。HDR 順: scene R16F → TAA → extract → CAS → log-luma histogram → blur → tone+bloom |
| wgpu_pbr | `3cd3d19` | 29、wesl 0.4.0 | — | REFERENCE ONLY | deferred PBR + IBL + CSM + 透明 + multi-camera を素直に書くと WESL 1.1k + Rust 3k。ただし tone/exposure 無し、prefilter の `sin` typo、DFG は 8 bit PNG。「仕組みの大きさ」の物差しで画質の物差しではない |
| renderling | `46bf54c` | 26 | REFERENCE | REJECT | device 共有不可(毎コマ readback)、rust-gpu prebuilt に Vism が入れない、transmission 無し |

**結論**: 「Rerun の Device/Queue → 外部 3D quality subsystem → Rgba16F → Motolii compositor」に入れる候補で生き残ったのは Bevy だけ。理由は機能数ではなく境界の実在。SDK 形の完成品は存在せず、SDK に最も近い実現は Bevy の WESL 層を SUBSYSTEM EXTRACTION で切り出す形(Bevy B の fallback)。「Rerun + Impact の image formation + Bevy の WESL PBR 部品」という第二案は、Impact の縦串が shader 文字列としてしか取れず、その中身(ACES/PBR Neutral、Jimenez bloom、TAA)は Bevy の WESL にも同等以上があるので、**第二案 = SUBSYSTEM EXTRACTION と同じ物**に畳まれる。次の実験は 100〜300 行の spike(decision-index 参照)。

### 15,200 行の 3 分割と「Motolii が今後理解・保守する graphics 技術」(利用者の指標、2026-09-24)

U5 の分類から私が振り分けた概算(行単位の数えではない):

| 区分 | 概算 | 中身 | 行き先 |
|---|---|---|---|
| 1. Motolii 固有 semantic が renderer code の中に埋まっている分 | 約 1.0〜1.5k | `SurfaceIn`/`program_*` の hook 契約(fork `surface.wgsl`)、`translate.rs` の AE 慣習(y-down・camera zoom・2D in 3D)、`BlendMode` の W3C 番号、`RunBreak` の理由、Clip の意味 | **Motolii に残る**(capability lowering として再配置) |
| 2. 既存技術なのに自作している quality rendering | 約 6〜6.5k | fork: curve fill 1,016、surface programs/mesh 1,677、rect surface 461、environment/IBL 864。Motolii: host shader 709(material/encoding/glow/glass/blend/matte/noise/motion)、environment/light/mesh 330、extrude・material.rs・vism.rs・surface_program の GPU 側 約 1.2k | **Bevy(3D)/ Vello(vector)/ Khronos(語彙)へ** |
| 3. 手書きの GPU scheduling / infrastructure | 約 5.5〜6k | `view.rs` の run 実行(texture 確保・保持・順序)、`layer_views.rs` の DFS と atlas、`compositor.rs` の format/texture plumbing、`frame_graph_scene`・`analysis`・`render/build`・`blocks`・`tick`・`warm`・`reuse`・`frozen`・`texture*.rs`、fork の views/embedding/targets 698 と pool 群 | **Rerun + render graph へ**(usage id の read-after-write で寿命・順序・cull を決める型) |

wgpu_pbr の 4k と 15.2k の差は「11k 多い」ではなく、4k が品質と composition semantics を捨てている分。理想の最終状態は 1 だけが Motolii に残る。

| 案 | Motolii が今後理解・保守する graphics 技術 |
|---|---|
| 現状 | PBR・IBL・DFG・glass/transmission・vector raster(curve fill)・HDR・bloom・tone・MSAA・texture 寿命と pass 順の手書き scheduling・fork の re_renderer 差分 |
| Bevy B(render-to-texture) | AE composition(blend・Plate/Matte/Clip)、Vism → `VismExt` の統合、capability lowering、ECS sync、時刻ランダムアクセスと temporal state の reset、CurveFill(残る) |
| threers B | AE composition + three.js 移植の差分(HDR clamp・MSAA)+ Vism の vertex patch + CurveFill |
| Bevy SUBSYSTEM EXTRACTION(WESL 輸入) | PBR の plumbing・pass・binding・IBL prefilter の dispatch・tone/bloom の pass は依然 Motolii 側。数式だけ借りる |
| Bevy + Vello + render graph(傾いている形) | composition / Vism 境界と capability lowering が中心。3D = Bevy、vector = Vello、video = re_video、host = Rerun、oracle = threers PT / Cycles |

この列は「映像表現ソフト」から「PBR renderer 研究」へ脱線しない指標。新候補は「Bevy B より何が明確に良いか」の challenger 方式で見る。

### サイド検索の候補 U11〜U15(2026-09-24、検索語を「PBR renderer」から viewport / render core / 2D engine へずらした結果)

| 候補 | pin | wgpu | 分類 | 決め手 |
|---|---|---|---|---|
| **viewport-lib** 0.22 | `5e500d6` | 27/29/30 の 3 leg | **REFERENCE ONLY**(SDK の形は本物、GPL-3.0-only + 商用ライセンス条項) | 手元の Metal で headless 動作、`--features wgpu30` check 通過。`ViewportRenderer::new(&device, format)`、`prepare_scene`(1 回: upload・lighting・shadow・batching)→ `prepare_viewport`(view ごと cull)→ `render_viewport`(view ごと HDR chain 全部)は実在。GPU IBL(irradiance/GGX prefilter/BRDF LUT、Rgba16F storage、無ければ CPU fallback)。**runtime WGSL hook**(`register_shading_hook` が lit shader の slot に文字列を継ぐ、`shade_surface/shade_light/shade_ambient/recolor`)は Bevy の `ExtendedMaterial` より Vism 寄り。auto exposure(histogram)・DoF・SSAO・contact shadow・decal・deformer(time は caller 書き)。**落ちる所**: frame の出口が必ず tonemap で **alpha = 1.0**(線形 HDR texture は `pub(crate)`)、raster material に transmission/thickness/multiscatter 無し、hook 契約に backdrop 無し(glass 退行)、**TAA 無し**(HDR 経路は MSAA も無し)、bloom は half-res Gaussian、AgX 無し、**hook の unregister 無し**(hot reload 不可、pipeline leak)、storage buffer ≥ 8 / bind group ≥ 4、upload の promotion window が scrub と衝突。描いた画に cast shadow が見えず moiré(未確認)。Bevy B との行比較: hook・multi-viewport・adapter LOC(1.5〜2.5k)・auto exposure・wgpu 版は **better**、HDR/alpha 出口・transmission・TAA・tonemap・hot reload・license・保守は **worse** |
| viewport-lib の系譜(U15) | — | — | 実年齢 ≈ 6 か月 | 親は **brimcraft**(作者の私的な流体シミュレータ、egui app、未公開・未出荷)。root commit `e660128f` 2026-04-17「Stage files brought over from brimcraft」36.9k 行。brimcraft の最古の痕跡は 2026-03-28(profile README)、2026 以前の著作権・日付・旧 crate 名は無し。作者の最初の Rust repo が viewport-lib。1,142 commit(Grim 1,133・utfus 9、utfus は 2026-06-04 作成の account)、0.1.0〜0.4.0 は yank + 履歴書き換え、5 か月で src 169k 行(0.1.0 の行は 43.5 % が逐語で残存 = 半分書き直し)。CI は publish のみ、test job 無し。利用者は作者自身の crate 群だけ(総 DL 1,163)。**「数年運用された CAD renderer の library 化」ではない** |
| Myth Render 0.3 | `1c18111` | **29**(30 更新を 10 日で revert) | 3D subsystem = **REJECT** / **render graph core = SUBSYSTEM EXTRACTION 候補 / 参照** | Metal で headless PNG・36 test 通過。品質は Bevy 同級(three.js `MeshPhysicalMaterial` の逐語移植、**attribution 無し**、AgX/Neutral、Karis bloom、TAA、Hillaire 大気、SSGI/SSR)。落ちる: Device 自作(`assemble_state` private)、外部 TextureView 無し、scene 必須、temporal 履歴が `Renderer` 単位、**tonemap 出口が LINEAR でも `saturate()`+gamma**、minijinja template の WGSL(WESL でない)、hot reload 無し、DFG LUT の compute は dead code。**`graph/core` 約 3k 行(MIT/Apache)**: `add_pass` + `create/read/write/mutate`、SSA(2 度目の write は panic)、blit の辺縮約 → 到達性 cull → topo → 寿命 → 区間 alias、Load/Store を寿命から導出、13 unit test。Motolii の `record_stack` の手動 alloc/保持/`mix_onto`/`layer_views` DFS を置き換え、`RunBreak` の意味だけ残る。adapter 約 150 行、`NodeSlot` の unsafe を所有することになる |
| Valo | `9d121cc` | 30 | **RENDER-TO-TEXTURE 成立、今は REFERENCE ONLY**、`valo-text` は将来の SUBSYSTEM EXTRACTION | Impeller の写し(stencil-then-cover、CPU flatten、**MSAA 4× のみ**)。実測: `Context::new(device, queue)`、**Rgba16Float に >1 が通る**、**4×4 perspective で正しい台形**、golden 53/53。Skia の blend 全種(HSL は draw ごと snapshot)、luminance mask、backdrop blur、Gaussian/drop shadow、**text の 3 段(atlas/SDF/outline)**。落ちる: AA が fork の解析 coverage から 4 段へ退行、3D は平面描き→貼るのみ、`Plus` は 1 に clamp、8 stop 超の gradient と image は 8 bit、色管理無し、1 人・5 週・README に AI 支援明記。Vello 比: target 形式/HDR/perspective/filter/text は Valo 上、AA/成熟度は Vello 上 |
| facett-map3d | `1ad8e85` | 29 | REFERENCE ONLY | egui facet の 3D crate。`RenderScene<'a>`(mesh 1・camera 1・light 1 を毎コマ借りる)+ `RenderBackend { caps, analyze, paint(scene, &mut egui::Ui), reset_cache }`、null backend 14 行。multi-view/RTT 無し(renderer が process singleton)。**Motolii の `render_graph/work.rs` の IR は既にこれより正しい方向で豊か**、`CountingBackend` は null backend の芽、本番の消費者(`execute_render_graph`)は trait を通っていない。取る: device 無しの `analyze(&RenderGraph) -> Stats`、semantic の `caps()`。取らない: renderer の pass 名を scene flag に(`ssao/bloom/flat_2d`)、shader 定数を中立 module に、cargo feature で scene の中身が変わる、trait に `Ui`、singleton |

**サイド検索の結論**: 「CAD/sciviz viewport は他人に埋め込まれるのが仕事」という観察は正しく、viewport-lib は Bevy より Motolii 寄りの境界(prepare の 3 分割・文字列 hook・小さい adapter)を実際に持っていた。しかし quality の芯(linear HDR + alpha の出口、transmission、temporal 安定)と hot reload が無く、GPL で、実年齢 6 か月。**Bevy B の challenger としては勝てない**。参照として抜く価値: `shade.rs`+`shade.wgsl` の hook composer、prepare の 3 分割、GPU IBL bake。Myth の graph core は scheduler の負債(view.rs の手書き)に対する「wgpu 上で動く test 付きの抜ける実装」。Valo は HDR と perspective が通る raster 2D の先例で、text 層が将来の部品。

### 「子供検索」の候補 U16〜U22(2026-09-24、有名 renderer の埋め込み化・現代化・別 engine への移植を探した結果)

| 候補 | pin | wgpu | 分類 | 決め手 |
|---|---|---|---|---|
| **nightshade-renderer** 0.57 | tarball `30e4b23`(**GitHub repo は 404、HEAD 取得不能**、Wayback 2026-04-30 に 845 commit) | 29.0.4(30 への移植は約 25 か所・1〜2 日、graph は 0 か所) | **RENDER-TO-TEXTURE(fork 前提)/ graph は REFERENCE ONLY / Myth の代替としては REJECT** | この Mac の Metal で正しく描画(`pbr_late.png`)。`new_with_device(adapter, device, queue, **surface**, …)` — 外部 Device は通るが **`wgpu::Surface` 必須**(OpenXR 用)、macOS の hidden window は `Occluded` で描かない。**host の `RenderInputs` を毎コマ渡す owned struct + dirty set** で ECS scheduler 無し(ただし nightshade-ecs 17k 行は非 optional 依存、single-view helper は毎コマ `full_rebuild_needed`)。品質は Bevy 同級で実コード: KHR transmission/volume/IOR/**dispersion**/clearcoat/sheen/iridescence/anisotropy、GGX + 高さ相関 Smith + LUT の energy compensation、GPU DFG LUT、GPU GGX prefilter + SH、PCSS 4 cascade atlas、WBOIT、SSAO、SSGI(半解像度、probe 無し)、SSR、Karis bloom 6 mip、ACES/AgX(近似)/Neutral、auto exposure(GPU storage)、LTC、clustered 1024 灯。**落ちる所**: 外部 texture への出口は **LDR・post-tonemap・alpha=1・surface format**(HDR は `scene_color` Rgba16F を host の `PassNode` で copy する必要あり、`targets.scene_color`/`graph` は `pub`)、**material shader hook 無し**(`mesh.wgsl` 家族 5.4k 行・8 group 48 binding の fork が要る)、hot reload 無し(`include_str!` + naga_oil)、**MSAA 無し**(TAA のみ、しかも post-tonemap)、camera ごとに graph 全体を execute + submit(cascade も post も TAA 履歴も View ごと)、`#[test]` 0、1 人・9 か月で 57 release・Rust 64k + WGSL 21k。warmup は実測では初コマから全 geometry が出る(README の記述は再現せず)。seek は TAA 履歴 drop + auto exposure off + particle/cloth 無しで可。timing の 16 B `map_async` が毎コマ(非同期、1 行で外せる)。graph 単独(feature `rendergraph`、3.4k 行、`&Device,&Queue` で動く)は SSA 無し・greedy alias・test 0 で Myth より下。**Bevy B 比**: ECS sync と adapter(1.5〜2.5k)は better、HDR 出口・headless・custom WGSL・hot reload・TAA・保守(repo 消失)は worse |
| rend3-hp | `b724a39`(`wgpu30testwriter`)、upstream base `d088a84`(2024-05、以後 upstream commit 0) | 30.0.1 で check 通過 | REFERENCE ONLY(B は REJECT) | 139 commit 全部 John Nagle 1 人、**機能追加ゼロ**(wgpu/winit の追随のみ)、`***NOT SURE***`/`todo!()` 残置、CI は macOS build のみ、test 画像は LFS pointer。**Metal で動かすのに 3 修正**(`create_iad` の限界表が旧単位で Metal adapter を全拒否、upload の slice 範囲、scatter copy の unmap)、scratch patch 4 行で headless 描画に成功(`headless-shadow-cube-256*.png`)。GpuDriven は Metal 不可(CpuDriven のみ)。良い所: `InstanceAdapterDevice` が全 `pub`、`add_imported_render_target(&Texture)`、forward target Rgba16F 固定で clamp 無し、temporal 状態ゼロ。無い物: **IBL**、multiscatter(`energy_comp = 1.0 // TODO`)、transmission・bloom・tone・exposure・TAA、point light の影、cascade(1 枚で viewport camera 追随)、multi-camera(`Viewport | Shadow(u32)` の閉じた enum)、hot reload(Handlebars template)。graph 1.5k 行は texture のみ・自分で submit。消える Motolii 行 0 |
| three-rs 0.1.2 | `c2e53e7` | 30.0.1、**Vulkan のみ**(scratch patch で Metal 動作) | REFERENCE ONLY | `oneilltomhq/three-rs`、**repo 作成 2026-09-13、11 日で 565 commit、216 commit に `Co-Authored-By: Claude`**。three.js r186 の `test/e2e/image.js` で pixelmatch 採点(39 example)。`NodeBuilder`/`WGSLNodeBuilder`/`NodeMaterial.setup()` を逐語移植、`PhysicalLightingModel`(multiscatter・DFG LUT・sheen・clearcoat・anisotropy)、PMREM、transmission は opaque frame の mip copy を読む本物の backdrop 経路。**`wgslFn` は移植済み**(body 逐語)だが **param 型に struct も `array<vec4f,6>` も無い** → Vism の `surface(in: SurfaceIn, p)` は wrapper を body 内に埋める(Motolii の flatten と同じ)。`coverage` 相当無し、`thickness` は uniform。`NodeBuilder::build` は three 自前の group/varying/attribute 規約で完全な program を吐き、消費者は three-rs の `Renderer` だけ → 抜けない。借りるなら `UniformSource` enum + 1 つの `match` writer、`version → dynamic_key → program(WGSL hash)→ pipeline(state)` の多段 cache |
| SkyEngine | `f8d40bf` | host 29(+ renderling 22 を別 link、Kajiya は ash) | REFERENCE ONLY | `SceneRenderer` trait の入力は **ECS の `&World` 丸ごと**、`SceneSnapshot` は Kajiya/renderling の 2 backend に後付けで **本命 wgpu backend は通らない**。**1 app に renderer 1 つ**、device 共有・interop 無し、非 wgpu backend で UI panic。取る: 中立 id + backend 内 handle map + content signature、outcome 付き frame token。取らない: trait の `wgpu_*` escape hatch、renderer 名の enum、pass flag の `RenderSettings`、mesh のディスク bake。Motolii の「lowering した `RenderGraph` だけが `execute` の入力」は SkyEngine より厳密に良い |
| Voidin | `36e84bb` | **0.17.1**、`Backends::VULKAN` hardcode | REJECT / REFERENCE | 2023-06〜10 の 4 か月で停止、test 0、CI 0。point light は Lambert + `pow(dot,16)`、IBL/shadow/GI/bloom/exposure 無し、TAA は camera reprojection のみ。本物は **LTC rect light**(LUT 込み 約 190 行)、**CPU SAH BVH + WGSL software 走査**(約 650 行、ray query 不要)、`#import` + `notify` の hot reload(約 300 行)。Kajiya は「参照」で「移植」ではない |
| bevy-kajiya + kajiya | `fff0f44` / `6145eaa`(2022) | — | REFERENCE ONLY(bridge の先例) | **bridge 1,594 行**、semantic の写像(instance/transform/camera/sun)は `HashMap<Entity,{state,handle}>` + command queue 約 350 行で自明に薄い。**難しかったのは ownership・asset・time**: kajiya の `draw_frame` は `&mut Swapchain` 必須で offscreen 出力が無く host は texture を受け取れない、`bevy_render` を compile できず Bevy の render sub-app 内部を手で複製して 0.8 に固定され死亡、asset pipeline と camera 数式を二重化、change detection 無しで毎コマ全 push + TLAS 全再構築、**temporal 状態を無視**(kajiya に realtime 履歴の reset API 無し)。kajiya の `RenderMode::{Standard, Reference}` は同じ TLAS/material/light/post を共有して graph だけ分岐。透明は本当に無し(alpha mode 無し、any-hit 無し)。kajiya-rg の aliasing は `// TODO` |

**子供検索の結論**: 仮説(親は保守停止・密結合、子が wgpu 30 化・renderer 抜き出し・別 engine 挿入・WebGPU 版移植)は当たっていたが、現物では **子が現代化したのは API(rend3-hp)か形(three-rs)で、renderer の芯は親のまま**。唯一 Bevy と品質で並ぶ nightshade は「host が毎コマ `RenderInputs` を渡す」設計が Motolii に最も近い反面、Surface 必須・LDR 出口・hook 無し・hot reload 無し・repo 消失で、**Bevy B の位置は動かない**。bridge の先例(bevy-kajiya)から **spike の 5 項目目 = temporal reset**(seek 後に履歴を消して 1 コマで cold render)を追加する。

## 調査の品質規則(2026-09-24 追加)

**「見つからない」は「存在しない」の証拠にしない。** 論文名・著者・会議名まで分かっている物は、公式 conference program → DOI/DBLP → 著者の repo の順にクロスチェックしてから NOT_FOUND と判定する。(例: PaRas は SIGGRAPH 2025 の公式一覧に載っていたのに Q 班が取りこぼし、M 班は「3D 曲面用で 2D の塗りには使えない」と正しく書いたが実在は未確認のままだった。)探索者への指示には「NOT_FOUND は検索した場所と語を列挙して出す」を入れる。

## 自分で確かめた物

- run の切れ目 MeshAfterRect は歴史的ではなかった: 消すと 2 本の contract の絵が変わる(gap 台帳に記録)。
- 空の run の固定費は GPU 約 0.08 ms、CPU は pass 数に比例(wgpu の `finish` が View 6 面で約 4 ms)。
