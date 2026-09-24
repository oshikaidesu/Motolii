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

## K: 曲線塗りの帯表(fork の patch 候補 `explore/curve-fill` = `66a439b20f`、**bit 同一を証明、採択候補・未 push**)

- Lengyel 式の行/列の帯表を upload 時に CPU で作り、同じ data texture に置く。fragment は自分の行帯と列帯の曲線だけを走査(曲線ごとの式と順序は不変)。公開 API 不変、fork の試験 66 + oracle(楕円・五芒星・5 px の薄片・穴・gradient、64² と 512²、18 枚の参照 PNG)で **0 画素差**。
- Motolii 側 A/B(file:// pin、push なし): Prism Garden 1080p の t=0 を別 build の before と比較して **2,073,600 画素すべて一致**。全試験は基準と同じ失敗 8 件。**GPU 71.2 → 39.2 ms**。CPU +2 ms は帯表(約 0.4 ms)ではなく、path mesh を毎コマ作り直す既存の仕事(`path_model` → `into_gpu_meshes` 約 1.1 ms)。
- K は最終形ではない: M の順位では「帯表 → 曲線塗りの画素ごと実行(縁の品質 seam)→ coverage mask pass + atlas(View 間で共有)」、または Vello 系の strips。P(2026 の prior-art 監査)の結果次第で CurveFill 自体を捨てる可能性を残す。

## M: 品質 rendering の調査(vector raster)

- 現行方式の異常さ(業界比): fragment で全曲線走査(Slug が最初に最適化で消す baseline)、標本ごとに材質まで再実行(UE/Skia/Rive/Slug は画素ごと)、coverage の前に paint を計算、coverage と shading が 1 つの program に融合(StC・Rive・Vello・Pathfinder・Graphite は全部分離)、仕事が bbox × 標本に比例(他は縁や tile に比例)。
- 対応表(抜粋): 帯表 = GENERIC(K)。bbox → 8 角形の cover polygon。coverage mask の DrawData(view 空間、または shape 空間の atlas を revision で共有 → 6 面の View が coverage を 1 回で済ませる)= GENERIC「Coverage」資源。`SurfaceProgram` ごとの標本率(曲線塗りは centroid、glass は sample)。paint を coverage の後に(bit 同一)。Thin Translucent 相当の dual-source blend(透過色と反射を 1 pass、glass の分散は Vism 側)= UPSTREAM_SEAM。TAA/TSR は不要(解析的 AA が既に決定論的)。
- 順位: 1 K(同一)→ 2 K + 曲線塗りの画素ごと実行(縁の seam)→ 3 coverage mask + atlas → 4 Loop–Blinn(縁の品質は Rive 級)。stencil-then-cover は 4× MSAA では品質が下がるので却下。

## 自分で確かめた物

- run の切れ目 MeshAfterRect は歴史的ではなかった: 消すと 2 本の contract の絵が変わる(gap 台帳に記録)。
- 空の run の固定費は GPU 約 0.08 ms、CPU は pass 数に比例(wgpu の `finish` が View 6 面で約 4 ms)。
