# 採点 2回目・借り物を見る人（2026-10-14、表は2版、9番だけ）

何もビルドせず、何も走らせていない。読んだだけ。「読んだ」と「推測」は分けて書く。

## 1. 読んだ物

全部読んだ：
- `filament/kit.h`（908行）、`video.h`、`drawn.h`、`motion.h`、`doc.h`
- 材質：`flat.mat`、`thick_glass.mat`、`matte.mat`、`matte.glsl`、`far_look.mat`、`far_side.mat`、`studio.glsl`、`glasslight.mat`、`paper.mat`、`floor_shade.mat`、`paper_copy.mat`、`spectrum.glsl`
- 場面：`edit.mm`、`lay.mm`、`neko.mm`、`puppet.mm`、`mask.mm`、`wall.mm`、`ae.mm`、`led.mm`
- つなぎ：`dawn_probe/gpu_door.h`、`gpu_door.cpp`、`CMakeLists.txt`、`build.sh`、`kit_wgpu.sh`、`tvg_on_dawn.sh`、`edit.sh`、`records.sh`、`lay.sh`、`neko.sh`、`check/` の5個
- パッチ2本：`filament_dawn_compute.patch`（375行）、`thorvg_z.patch`（362行）
- 紙：`AE_RUBRIC.md`、`evidence.md`、`research_edit_core.md`、`round_01.md`、`records/doc.txt`、`records/when.txt`

持ち主の側で確かめた物（ヘッダとソース）：
- Filament：`backend/platforms/WebGPUPlatform.h`、`WebGPUPlatformApple.h`、`filament/Engine.h`、`MaterialInstance.h`、`View.h`、`backend/src/webgpu/WebGPUDriver.cpp`、`WebGPUPlatform.cpp`、`tools/matc`、`out/wgpu_install` の中身
- Dawn：`out/wgpu/third_party/dawn/gen/include/dawn/webgpu.h`
- ThorVG：`inc/thorvg.h`、`src/loaders/lottie/tvgLottieParser.cpp`、`tvgLottieModel.h`、`tvgLottieProperty.h`
- GStreamer：`gst/video/video.h`、`video-color.h`、`gst/app/gstappsink.h`、`gst/gstcontrolsource.h`、`gst/controller/`、`ges/ges-timeline-element.h`
- SQLite：Homebrew の `sqlite3.h` と本体の記号、macOS 付属の `sqlite3.h`

読めなかった物：
- `records/` の残り（速さ、色、時刻の表）。9番に要らないので開いていない。
- `probe_edit/doc_ges.c`、`undo_sqlite.c` は有るのを見ただけ。
- `motion_test.sh`、`ae_test.sh`、`led.json`。
- GStreamer の本体のソースは手元に無い（ヘッダだけ）。controller が「最初のキーより前」に何を返すかは確かめていない。
- ネットは使っていない。調査メモの「どこにも出来合いが無い」は、手元の物でしか確かめていない。

## 2. 数えた物

### (a) 見た目を計算する自作シェーダ：6個（ファイルで11本）

1. **厚いガラス**：`thick_glass.mat:55-159`（二面の屈折、暗さ、虹、ランプの映り）、`far_side.mat:22-26`、`far_look.mat:22-30`、`studio.glsl:4-34`（作り物の部屋）。支える側が `kit.h:544-596`（見た目の数字11個が 584-594、塊らしさの式が 573-583、`usual_depth` が 513-521）。材質は unlit で、Filament の光は使っていない（`thick_glass.mat:9-10` に自分でそう書いてある）。今夜の場面のうち edit、neko、puppet、mask、wall、led の6本が通る。
2. **色抜きと色こぼれ取り**：`flat.mat:30-40`、口は `kit.h:602-605`。
3. **動画の色直し**：`flat.mat:44-47`（行列を掛ける）と `flat.mat:53`（画面の数字を光に戻す式を手で）。行列を組むのは `video.h:216-219`。
4. **マスク**：`matte.glsl:9-24`（目ごとの写し、4枚の掛け合わせ、ぼかしは絵の段で）、`matte.mat:21-26`（明るさの重み）、仕組みは `kit.h:618-692`、`714-721`、`726-729`。
5. **落ちる光**：`glasslight.mat:39-98`、`kit.h:415-434`。led だけ。
6. **紙と影**：`paper.mat:24-76`、`floor_shade.mat:21-34`、`paper_copy.mat:11-16`、`kit.h:344-410`。led だけ。「Filament の太陽の影は使わない」と `kit.h:333-334` にある。

`spectrum.glsl` は MIT の借り物なので数えない。

### (b) 自作の時刻合わせ・混ぜ方：5個

1. **どの絵が今の絵か**：`video.h:289-321`（held、next、nextAt、origin、last）。GStreamer 自身の同期は切ってある（`video.h:178` の `sync FALSE`）。
2. **繰り返しの継ぎ目**：`video.h:142-145`（流れの中の合図を自分で見張る）と `video.h:311-314`（別スレッドから巻き戻しを頼む）。切り出しと速さを seek の数字に直す式が `video.h:133-138`、`194`。seek そのものは GStreamer の物。
3. **舞台の時計と刻み**：`kit.h:707-710`、`791-796`、`806-838`、`843-874`（`sleep_until` で60分の1秒を自分で刻む。画面の合図も GStreamer の時計も使わない）。
4. **AEの絵の時刻**：`drawn.h:33`（秒をコマに直す）、`drawn.h:92`（`fmod` で繰り返す）。
5. **文書の中の時間**：`doc.h:83-84`（切り出しの終わり＝始め＋長さ×速さ）、`doc.h:101-106`（何も終わらなければ8秒）、`doc.h:113-118`（いつ居るか）。

おまけ：`kit.h:784` は太鼓の音を式で作っている（鳴らすの自作）。edit は使わないので数に入れない。

### (c) 公開されてない口や並びの利用：6個と半分

1. `video.h:45`、`60-64`：GStreamer の Mac 用部品の札の並びを写して読む。公開ヘッダに無いのは確かめた（`/opt/homebrew/include/gstreamer-1.0` に corevideo の名は1つも無い）。
2. `gpu_door.cpp:8`、`24`：Filament の内側のヘッダ `details/Engine.h` で描画の列に仕事を入れる。そのために Filament のビルドの中で自分の物を作る（`CMakeLists.txt:8`、パッチ 10-13行）。`Engine.h` の公開の口に同じ物は無かった（読んだ）。
3. `video.h:81`、`89-93`：共有を貸して返さない。`video_off`（`video.h:263-277`）にも返す呼び出しが無い。共有の元も共有テクスチャも放していない。
4. `motion.h:47`、`50`：`const_cast` で回る。
5. `tvg_on_dawn.sh:14-20`：Dawn を `wgpu_native` という名前の偽の札で ThorVG のビルドに渡す。
6. `kit_wgpu.sh:31`：入れた SDK ではなく、ビルド途中の木の中（`out/wgpu/third_party/dawn/gen/include`、`thorvg_src/src/loaders/lottie`）を直に読む。
7. （半分）`kit.h:110`：`setFeatureFlag` は公開の口だが、旗の名前 `engine.color_grading.use_1d_lut` は内側の物（`filament/src/details/ColorGrading.cpp:812` で見た）。

### (d) パッチのうち、持ち主の振る舞いを変える行

**Filament へのパッチ**：足した行 159、消した行 12（数えた）。
- 元の動きを変える行：16行、7か所。
  - 45→47-49：全部の頂点バッファに Storage と CopySrc
  - 51→52-53：SHADER_STORAGE に CopySrc
  - 65：`mLayout = nullptr` を消す
  - 77、79：デバッグ用の囲いを外す
  - 263→264：待ちを 1ms から 50µs に
  - 339：Float32Blendable をどのデバイスにも頼む
  - 346-348：Mac の共有の機能3つを共通の表に足す
- 「未対応」「todo」の穴を埋める所：消した6行（91、107、120、121、170、171）と、その中身。足すだけに近い。
- 残りの約140行は足すだけ。

**ThorVG へのパッチ**：足した行 157、消した行 6（数えた）。
- 元の動きを変える行：23行、6か所。
  - 258→259-264：位置の読み方を分ける
  - 272、277-281：3Dレイヤーの anchor と scale を、元の読み方より先に横取りする
  - 326→327、335-336→337、342→343、350→351：鍵の曲線の型 `LottieVectorFrame` を書き換える（2D の全部の動きがここを通る）
  - 116：`friend` で中を開ける
- 139行は `(int)layer->type == 13` という裸の数字。ThorVG の型の表（`tvgLottieModel.h:1135`）にカメラは無い（読んだ）。
- 359行は新しい型に、別の種類の札（`Scalar3`）を付けている。
- 残りの約134行は足すだけ。

### (e) 使ってないのに残ってるパッチ：確かな物4個、推測2個（159行のうち約110行）

1. 計算用の口：172-215、251、222・227・239（並びを持ち回る）、65、77-79。
2. バッファの読み戻し：122-162、46-53。
3. SHADER_STORAGE の並び：92-95。
4. 環境変数 `FILAMENT_DAWN_TOGGLES`：356-364、329。
5. （推測）Float32Blendable：338-339。今夜の描き先は RGBA16F と R8 だけ。
6. （推測）SharedFenceMTLSharedEvent：347。`video.h` は柵を1度も使っていない。Dawn が要るかは未確認。

1〜4を使うのは `cbeam.h` と `dawn_probe.cpp` だけで、今夜の場面はどれも通らない（grep で確かめた）。`compute_door.cpp` も今夜のライブラリに一緒に入っている（`CMakeLists.txt:8`）。

### (f) ずっと自分で持つパッチ：2本

- `thorvg_z.patch`：本家に送らないと決めてある。
- `filament_dawn_compute.patch`：本家に返す予定がどこにも書いていない。

### 9番の点：**3**

- (a)〜(e) を足すと、(a) 6、(b) 5、(c) 6、(d) 13か所、(e) 4〜6。30個を超える。表の「3：9個以上」のまん中。
- 上げなかった訳：1回目から、パッチは1行も減っていない。(e) は消すだけで0になるのに残っている。新しく足した `doc.h` は中の借り方は正直だが、表の(a)〜(e)を1個も減らしていない。
- 下げなかった訳：読む（GStreamer、ThorVG）、描く（Filament、ThorVG）、AEの動き、キーの間（controller）、保存と取り消し（session）は本当に持ち主の仕事だった。下で1個ずつ確かめた。「自作エンジン」ではない。
- ただし、ガラス・マスク・光の4個は Filament の光を通らない自前の描き方で、ここだけ見れば小さな自作エンジンに近い。

**本当に借り物だった所（確かめた）**
- 取り消し：`doc.h:144-148` が session で変更の束を取り、`doc.h:176-177` が `sqlite3changeset_invert` と `sqlite3changeset_apply` で戻す。作り直しではない。Homebrew の本体に記号が38個ある。
- キーの間：`doc.h:44-45`、`50` が `gst_interpolation_control_source_new`、`gst_timed_value_control_source_set`、`gst_control_source_get_value`。自前の補間は無い。
- 重ね方：`flat.mat:9` の `blending` を `kit_wgpu.sh:21-23` が matc の公開の引数 `-T` で4回作る。Filament 自身の混ぜ方。ただし乗算だけ `flat.mat:54-56` に自前の直しが1行ある。
- 動画の色の重みと範囲の数字：`video.h:207-214` が `gst_video_color_matrix_get_Kr_Kb` と `gst_video_color_range_offsets`。ここまでは借り物。行列に組むのと掛けるのは自作（(a)3）。範囲は GStreamer の言う事を信じず、絵そのものに聞き直している（`video.h:211-213`）。
- 繰り返しと速さと逆再生：`video.h:136` の `gst_element_seek`（rate と SEGMENT）。GStreamer の物。

## 3. 借りられたのに自作した所

1. **AEの絵の透明**。ThorVG は透明つきで絵を渡せる（`drawn.h:72` で地を描かなければ透明、`matte.mat:23` はその透明を読んでいる）。なのに板の材質 `flat.mat:43` は `.rgb` しか読まない。だから白い地を描いて（`drawn.h:69-72`）、それを色抜きで消している（`drawn.h:112`、`neko.mm:77`、`doc.h:88`）。持ち主の物：ThorVG の透明と Filament の `blending : transparent`。
2. **回転の札の読み**。`gst_video_orientation_from_tag`（`gst/video/video.h:193`）がある。`video.h:225-226` は文字を自分で比べている。
3. **3Dの形**。うちの Dawn ビルドに `libgltfio.a`、`libgltfio_core.a`、`gltfio/AssetLoader.h` が入っている（`out/wgpu_install` で見た）。なのに立方体・輪・二十面体・球を手で作っている（`kit.h:215-294`、`471-487`）。「外の3Dファイルが入らない」もここで埋まるはず。gltfio の材質が WebGPU で通るかは推測（試していない）。
4. **デバイスを借りる口**。`WebGPUPlatform::requestDevice` は公開で virtual（`WebGPUPlatform.h` で読んだ）。`WebGPUPlatformApple.h` も入っている。`Engine::Builder::platform()`（`Engine.h:562`）に自分の子を渡せば、パッチの `getDevice` と `mDevice`（22-24、33、372行）はパッチでなく自分の側に書ける。機能の表（344-348行）まで子で足すには、持ち主の中身（`WebGPUPlatform.cpp:640-669`）を写す事になるので、そこは勧めない（推測）。
5. **貸し借りを返す口**。`wgpuSharedTextureMemoryEndAccess` と `wgpuSharedTextureMemoryRelease` が Dawn にある（`webgpu.h:4997`、`5002`）。呼んでいない。
6. **絵で絵を明るさで抜く**（`mask.mm:73-77`、同じ板に乗る2枚の ThorVG の絵）。ThorVG の `Paint::mask(target, MaskMethod)`（`thorvg.h:525`）が1枚の絵の中でやる。マスクの種類のうち、この1つだけは丸ごと借りられる。
7. **ガラス**。Filament 自身の屈折を使う材質 `own_glass.mat` と口 `kit.h:527-537` が、同じファイルにもうある。今夜の場面は1本も使っていない。「向こう側の面が見えない」という訳は書いてある（`kit.h:526`）ので「無駄に作った」とは言わない。ただ、一番大きい自作の隣に持ち主の版が寝ている。
8. **部屋の光**。edit、neko、puppet、mask、wall が `room()` で Filament の部屋（IBL）を読み込む（`edit.mm:17` ほか）。でもその場面の材質は全部 unlit で、部屋から使うのは明るさの数字1個だけ（`thick_glass.mat:159`、`far_look.mat:29`）。借りた物を使わずに、隣で `studio.glsl` を自作している形。
9. **文書の時間の言葉**。`doc.h:8` は「GES の言葉で」と言うが、借りたのは名前だけ。働きは `doc.h:83-84`、`113-114` の自作。GES 本体の口はこの Mac にある（`ges/ges-timeline-element.h:331-337`）。調査メモは「奥行きが無い、1個置くのに344ms」で外したと書いている。そこは読んだだけで、測り直していない。

## 4. 1回目の指摘のその後

自作：
- `thick_glass.mat` 全体と `kit.h` の支え：**残ってる**（今は `kit.h:544-596`）。
- `studio.glsl`：**残ってる**。
- 影と落ちる光（`kit.h`、`glasslight.mat`、`paper.mat`、`floor_shade.mat`）：**残ってる**（`kit.h:329-434`）。
- マスクの仕組み一式：**残ってる**。引数10個の写し（`flat.mat:12-14`、`thick_glass.mat:19-21`）、4個までの縛り、関数の中の静的な表（`kit.h:678`）もそのまま。**変わった**のは1つ：目の場所をコマの始まりの前に伝える（`kit.h:714-721`）。
- `flat.mat` の色抜き：**残ってる**。
- `flat.mat` の動画の色変換（BT.709 決め打ち）：**変わった**。重みと範囲はファイルから（`video.h:202-219`）。行列を組んで掛けるのは自作のまま。明るさの曲線は見ていない。
- `video.h` の「どの絵が今か」：**残ってる**（`video.h:289-321`、`sync FALSE` は 178行）。`video_patience` は**消えた**（代わりに舞台の `by_clock`）。
- `motion.h` の3Dの組み立てと親たどり：**残ってる**（同じ行）。
- `thorvg_z.patch` の `Bezier3`：**残ってる**。
- 手作りの形：**残ってる**。

迂回：
- GStreamer の非公開の並び：**残ってる**（`video.h:45`、`60-64`）。
- 貸しっぱなし、共有の元を放さない：**残ってる**（`video.h:81`、`89-93`）。
- `gpu_door.cpp` の内側のヘッダ：**残ってる**。
- `flat.mat` の出口の逆算：**残ってる**（53行）。内部の旗：**残ってる**（`kit.h:110`）。
- `thick_glass.mat` の配列の書き下ろし：**残ってる**（135-147行）。
- `motion.h` の `const_cast`：**残ってる**（47、50行）。
- `popen("ls …")`：**消えた**（`std::filesystem`、`neko.mm:30`、`puppet.mm:30`）。
- ffmpeg を文字列で起動：**残ってる**（`kit.h:782-787`）。
- AEの絵の白抜きで材質の中の名前を直に書く：**変わった**。`key_out(Drawn*)` になった（`drawn.h:112`）。元の回り道（透明を読まずに白を抜く）は残ってる。
- z の手ずらし：**残ってる**（`puppet.mm:39` の 0.01、`puppet.mm:51` の 0.002）。

パッチ：
- 待ち 1ms→50µs：**残ってる**。
- 頂点バッファの使い道、`mLayout`：**残ってる**。
- 使ってない計算用の口・読み戻し・環境変数：**残ってる**。
- Mac 専用の機能を共通の表に：**残ってる**。
- `thorvg_z.patch` の型いじりと `friend`：**残ってる**。

Mac 専用の混ざり（`kit.h:32-37`、文字の立体、PNG 書き、メモリ読み）：**残ってる**（9番では数えない）。

1回目の26個のうち、消えた2、変わった4、残ってる20。

## 5. doc.h：つなぎか、発明か

調査メモの「丸ごとの出来合いは無い」について：手元で見える範囲では同意する。SQLite には取り消しの「積み」は無い（ヘッダにあるのは束を取る・逆にする・当てる、だけ）。GES のヘッダはあるが取り消しの口は無い。ただし2つ言う。MLT、libopenshot、Yrs は入れても測ってもいない（表に「記憶」が多い）。GES は「奥行きが無い」で外したが、時間の並びだけ借りる道は測っていない。

| 所 | 行 | 何か |
|---|---|---|
| `doc_rows` | 28-36 | つなぎ（SQLite の用意・進める・片付ける） |
| `doc_number` | 39-52 | つなぎ（事実の行を controller に入れて読む）。発明が3つ混ざる：文字かどうかで値の種類を分ける決まり、行き過ぎない曲線に決め打ち、負の時刻は0に丸める |
| `doc_word` | 53-57 | つなぎ |
| `doc_match` | 63-98 | **発明**。文書と舞台を見比べて、作る・消す・作り直すを決める仕組み。「作りに関わる名前8個のどれかが変わったら壊して作り直す」という決まり（66行）。形の名前と大きさ（ring、gem、cube、90行）も発明 |
| `doc_lasts`、`doc_seconds` | 101-106 | 発明（終わりが無ければ8秒） |
| `doc_moment` | 109-123 | **発明**。いつ見えるかの決まり（start から lasts の間）、場所の組み立て（x、y、z、turn、high を 移動×z回り×大きさ に） |
| `doc_open` | 126-136 | 表の形 `fact(thing, name, at, value)` と `step(n, change, undone)` は**発明**（文書の形）。開くのはつなぎ |
| `doc_change` | 139-163 | つなぎ（session で束を取ってしまう）。152行の「取り消した先は捨てる」は取り消しの積みの決まりで、発明だが10行に満たない |
| `doc_step` | 166-183 | つなぎ（逆にして当てる）。積みの上下（undone の旗）は自作 |
| 呼び出し9個 | 186-198 | つなぎ（SQL の文を組むだけ。引用は SQLite の `%Q`）。番号は「一番大きい＋1」という決まり（188行）が発明 |

持ち主がもう持ってる物の作り直し：
- **動かせる値が5個だけ**。キーが効くのは x、y、z、turn、high（`doc.h:119-120`）。start、inpoint、duration、speed、key は時刻0の値しか読まない（`doc.h:81-83`、`113`）。
- **切り出しや速さを変えると動画を壊して作り直す**（`doc.h:66`、`70-74`）。GStreamer は流れたまま seek で変えられる（`video.h:136` でもう使ってる）。色抜きの入り切り（key）でも動画ごと作り直す。材質に数字を1つ伝えるだけの事（`video.h:281-285`）。ただし、これは「自分の行を増やさない」選び方でもあるので、9番では引かない。
- **時計が2つ**。`video()` と `picture()` は舞台の時計に自分を載せる（`video.h:251`、`drawn.h:92`）。`doc.h:85`、`88` がそれを外から消して、文書の時計に付け替える。舞台の中身（`s.playing`）を文書が直に触っている。
- **`add()` は呼ぶたびに材質のファイルを読んで取っておく**（`kit.h:173-175` の静的な入れ物）。文書の編集で作り直すたびに増える。`take_off` は材質を消すが、読んだ中身は残る（読んだだけ。量は測っていない）。

まとめ：198行のうち、SQLite と GStreamer を呼ぶだけの所が約80行、見比べと時間と場所の決まり（発明）が約70行、残りは型と説明。発明の中身は「文書の形」と「文書を舞台に伝える決まり」で、調査メモが「借りられないで残る2つ」と言った物と合っている。そこは正直。

## 6. 信じなかった所（evidence.md）

- **「公式でない口を使っている所」が2個だけ**。数えたら6個と半分（上の(c)）。`gpu_door.cpp` が Filament の内側のヘッダを使う事、`const_cast`、偽の `wgpu_native` の札、ビルド途中の木を直に読む事が書いていない。
- **「自分で書いた物 2124行」**。足し算は合う。でも入っていない物がある：場面 493、検査 115、スクリプト 60（`edit.sh`、`records.sh`、`tvg_on_dawn.sh`）、`CMakeLists.txt` 11、led 用 228、パッチの足した行 316。今夜の道ぜんぶで約3350行。
- **「今夜の場面が使っていない部分も残ったまま」**。本当。ただし量が書いていない：Filament のパッチ159行のうち約110行。
- **「色：ファイルが言う色の式で直す」**。半分。重みと範囲の数字は GStreamer から。行列は自作。明るさの曲線は見ていない（`flat.mat:53` は何でも画面の式で戻す）。10ビットは8ビットに頼んで受けている（`video.h:176`）。HLG と PQ が来たら色は違うはず（推測）。
- **「値に時刻つきの事実を2個以上書くと、その間をつなぐ」**。本当だが、効くのは5つの値だけ（上）。「どの値にも」ではない。
- **「取り消しの歩みもファイルに入っているので、閉じて開いても取り消せる」**。コードはそう読める（`step` の表）。でも `check/doc.sh` は開き直した後に取り消しを1回もしていない。記録が無い。
- **`records/when.txt`**。書いてあるコミットは `bd8d39f` で、`doc.h` が入ったコミット（`2c5542e`）より古い。`records/doc.txt` がどのコードで出た物か、記録からは言えない。
- **「Filament が元から持っている重ね方」**。本当。ただし乗算には自前の直しが1行ある（`flat.mat:54-56`）。
- **「色抜き（AEの絵の白）」が機能として書いてある**。実は、透明を読まない材質の回り道（上の3の1）。
- **「編集の中身を丸ごと持つ出来合いは見つからなかった」**。手元では反対の証拠は無い。でも候補の半分は「記憶」か「検索の要約」で、入れて測ったのは SQLite、controller、GES の3つだけ。

確かめられた所（公平のため）：session、controller、seek、matc の `-T`、Kr・Kb と範囲の数字、ファイル数えの `popen` が消えた事、パッチの足した行の数（159と157）。

## 7. この表の弱い所（9番）

1. **「1個」の大きさが決まってない**。(a) はガラス1個と数えても、ファイル4本と数えてもいい。今回は「見た目1つ＝1個」で6、ファイルなら11。
2. **9個も30個も同じ3点**。減らしても点が動かない。段を足すか（9〜15、16〜25、26以上）、文字ごとに上限を決めて足す形に。
3. **(d) だけ行で数える**。他は個数。どう足すか書いていない。今回は「か所」で足した。
4. **(f) は数えるのに、10点の条件に入っていない**（「(a)〜(e) が0」）。
5. **「理由が書いてある」は7点の所でしか効かない**。5点と3点では、訳がある自作も無い自作も同じ。
6. **発明の置き場が無い**。シェーダでも時刻でもパッチでもない自作（`doc.h` の文書の形と見比べの仕組み、`motion.h` の行列の組み立て、手作りの形、取り消しの積み）に文字が無い。(g)「自分で決めた形と決まり」を足す。
7. **寝ている物を数えない**。`filament/` に材質が93本ある。`kit.h` には今夜だれも呼ばない物がある（`glass_blocks`、`bubbles`、`box_light`、`own_glass`、`letters`）。「見通せない」側の傷なのに、どこにも入らない。
8. **持ち主の口の使い方の誤り**（貸して返さない、放さない）が (c) なのか別なのか書いていない。今回は (c) に入れた。
9. **「借りすぎて重い・配れない」側に数える物が無い**。Homebrew の GStreamer と SQLite に絶対の道でつなぐ（`edit.sh:6`）、Filament を15分かけてソースから作る、手を入れた写しが2つ。
10. **1場面しか通らない物（led の光と紙）と、全場面が通る物（板、ガラス）が同じ1個**。

## 8. 9番を一番上げる3つ（何を消すか、誰に返すか）

1. **Filament のパッチから、今夜だれも使っていない物を消す**。計算用の口、バッファの読み戻し、SHADER_STORAGE の並び、環境変数、頂点バッファの使い道の書き換え、`mLayout` の持ち回り、待ち時間の書き換え。あわせて `compute_door.cpp` を今夜のライブラリから外す。`getDevice` は Filament の公開の型（`WebGPUPlatform`）の子に返す。残るのは「テクスチャの受け取り」と「機能3つ」で約45行。(e) が0になり、(d) が7か所から1〜2か所になる。書く物は無く、消すだけ。
2. **ガラスと光を Filament に返す**。`thick_glass.mat`、`far_side.mat`、`far_look.mat`、`studio.glsl` と `kit.h:544-596` を消し、Filament 自身の屈折の材質（もうある `own_glass.mat`）と、読み込み済みの部屋の光に任せる。led の `glasslight.mat`、`paper.mat`、`floor_shade.mat`、`paper_copy.mat` と `kit.h:329-434` も消し、Filament の太陽の影に返す。(a) が6個から3個に。絵は変わる（向こう側の面、落ちる虹が無くなる）。それを受け入れるかはユーザーが決める事。
3. **動画の受け取りを持ち主のやり方に返す**。`video.h:289-321` の「どの絵が今か」を消し、時計で流す時は GStreamer 自身の同期（`sync`）に任せる。貸したら毎コマ返す（Dawn の `EndAccess`）。AEの絵は白い地と色抜きをやめて ThorVG の透明をそのまま読む（`drawn.h:69-72` と、絵に対する `key_out` を消す）。(b) が2個、(c) が1個減る。非公開の札の並び（`video.h:45`）は、公開の代わりが手元のヘッダに見つからなかったので残る。

次点：`thorvg_z.patch` と `motion.h:71-103` を消す（3Dレイヤーとカメラをあきらめる）。(f) が1本、(d) が6か所減る。機能を捨てる話なので3つには入れなかった。
