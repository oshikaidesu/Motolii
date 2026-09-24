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

## 調査の品質規則(2026-09-24 追加)

**「見つからない」は「存在しない」の証拠にしない。** 論文名・著者・会議名まで分かっている物は、公式 conference program → DOI/DBLP → 著者の repo の順にクロスチェックしてから NOT_FOUND と判定する。(例: PaRas は SIGGRAPH 2025 の公式一覧に載っていたのに Q 班が取りこぼし、M 班は「3D 曲面用で 2D の塗りには使えない」と正しく書いたが実在は未確認のままだった。)探索者への指示には「NOT_FOUND は検索した場所と語を列挙して出す」を入れる。

## 自分で確かめた物

- run の切れ目 MeshAfterRect は歴史的ではなかった: 消すと 2 本の contract の絵が変わる(gap 台帳に記録)。
- 空の run の固定費は GPU 約 0.08 ms、CPU は pass 数に比例(wgpu の `finish` が View 6 面で約 4 ms)。
