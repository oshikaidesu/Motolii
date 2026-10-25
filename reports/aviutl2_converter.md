# AviUtl2 のスクリプトを Motolii でそのまま動かす — 下調べ

2026-10-26。Motolii のコードは書いていない。調べた物・試した物の置き場は一番下。

## 要約（5 行）

1. **できる見込み。ただし上位 50 本のうち、そのまま動かせるのは多くて 40 本（80%）。** 残りの 10 本は Windows の DLL（.mod2 / .aux2）を同梱していて、Mac では動かない。
2. **obj の大きさ**：50 本が使う obj は関数 28 個と値 39 個、合わせて 67 個。関数だけで数えると、よく使う上位 18 個で 40/50 本（80%）の呼び出しがそろう。DLL 付きを「動かない」と数えると、80% に届くには関数 28 個すべてと、`obj.effect("ぼかし")` のように本体の効果を名前で呼ぶ所（19 本、29 種）が要る。
3. **HLSL は今の glslang→Tint の道（isf.cpp）に乗る。** HLSL を読む部分は Filament の glslang の元ソースにある（glslang/HLSL の 7 本）。ただ、今組み立ててある libglslang.a には入っていないので、7 本を足して組み直す。試した結果、141 本のうち 137 本が glslang（HLSL）を通り、133 本（94%）が Tint で WGSL になった。DXC は要らない。この Mac にも無い。
4. **大きな危険の 1 つ目：定数の並び。** 定数の入れ物（cbuffer）の詰め方が HLSL と WGSL で違う。Tint は何も言わずに WGSL の並びに直すので、12〜21 本（6〜8 包）で値がずれる。止まりはせず、画が黙って違う。読み込む時に並べ替える小さな手間が要る。
5. **2 つ目：DLL が 10/50。3 つ目：本体の効果を名前で呼ぶ所（19/50）と、描き方の約束。** 描き方の約束は、仮想バッファ、合成 16 種、AviUtl のカメラ、文字・図形の出し方。これを AviUtl と同じ画にするのが一番の手間。ほかに、Lua は LuaJIT が要る（11/50 が bit・ffi・string.buffer を使う）。

---

## 1. 出典

| 何 | どこ | 備考 |
|---|---|---|
| 公式のスクリプト仕様 | `lua.txt`（AviUtl2 本体の zip に同梱。1,656 行） https://spring-fragrance.mints.ne.jp/aviutl/aviutl2_v2.1.15.zip | v2.1.15、2026-10-24。以下の仕様はすべてここから引いた |
| 本体の説明 | 同じ zip の `aviutl2.txt` | 中の画の形式は「RGBA 16bit Float（乗算済み α）」、DirectX 11.3 以上と ROV が要る |
| プラグイン SDK | https://spring-fragrance.mints.ne.jp/aviutl/aviutl2_sdk.zip | `.mod2`（スクリプトから呼ぶ DLL）の作り方。中身は C++ の DLL |
| 本体が使うシェーダーの翻訳器 | aviutl2.exe の中の文字列 | `D3DCOMPILER_47.dll`、`D3DCompile`、`D3DReflect`、`ps_5_0`、`cs_5_0`。つまり FXC の SM5.0 |
| 本体同梱の Lua | lua.txt の末尾 | `lua.dll` = Lua 5.1.4（パッチ 5.1.4-2）、`luaJIT.dll` = LuaJIT 2.1 |
| 人気の順位 | https://github.com/Neosku/aviutl2-catalog-data の `index.json` | 445 件、うちスクリプト 248 件。`popularity` の数で並べた |
| 似た物 1 | https://github.com/routersys/YMM4-LuaScript （MIT、C#） | YMM4 の中で AviUtl2 の `obj.pixelshader` を D3D11 で動かしている。Windows 専用だが、振る舞いの見本になる |
| 似た物 2 | https://github.com/kagemorikosame/sashimono-edit （MIT、Python） | AviUtl スクリプトを lupa（Lua 5.1）で動かす編集ソフト。`obj.pixelshader` と `obj.computeshader` は「未対応」と記録するだけ |
| 非公式の読みやすい説明 | https://github.com/aviutl2/aviutl2_modern_docs 、補完の拡張 https://github.com/hirokawa-beach/AviUtl2-IntelliSense | どちらも MIT |

## 2. 公式の仕様（lua.txt の要点）

### ファイルの種類

- `*.anm2`（アニメーション効果）、`*.obj2`（カスタムオブジェクト）、`*.cam2`（カメラ効果）、`*.scn2`（シーンチェンジ）、`*.tra2`（トラックバーの動き）。
- 古い `*.anm / *.obj / *.cam / *.scn / *.tra` も読める。ただし文字コードは SJIS で、既定は Lua。新しい形式は UTF-8 で、既定は LuaJIT。
- スクリプトのモジュール `*.mod2` は DLL。`obj.module("名前")` で関数の表を受け取る。
- ファイル名の先頭が `@` だと、1 本の中に `@名前` で区切って何本も入れられる。本体同梱の script.anm2 もこの形で、36 本入っている。
- 置き場所は `ProgramData\aviutl2\Script\` と、その 1 つ下のフォルダ。

### 頭書き（ファイルの先頭の `--` 行）

| 書き方 | 意味 | 50 本中で使う数 |
|---|---|---|
| `--track@変数:名前,最小,最大,既定[,単位,0の名前,倍率]` | つまみ（古い形 `--track0:` も可） | 49 |
| `--label:分類` | 追加メニューの分類 | 46 |
| `--information:文` | 情報 | 39 |
| `--select@変数:名前=既定,選択肢=値,...` | 一覧から選ぶ | 39 |
| `--group:名前[,開閉]` / `--group` | まとめる・まとめ終わり | 38 |
| `--check@変数:名前,0/1 か true/false` | チェック（0/1 なら数、true/false なら真偽） | 35 |
| `--color@変数:名前,0xRRGGBB か nil` | 色 | 27 |
| `--value@変数:名前,既定` | 数・文字・表を手で入れる | 26 |
| `--require:2003500` | 要る本体の版 | 17 |
| `--filter` | フィルタオブジェクトにも使える | 16 |
| `--trackgroup@x,y,z:名前` | つまみの組 | 13 |
| `--hide@変数:条件` | 条件で隠す | 12 |
| `--checksection@...` | 区間ごとのチェック | 11 |
| `--file@` `--folder@` `--font@` `--figure@` `--text@` `--string@` `--separator:` `--hidemenu` `--data@名:大きさ` `--script:lua` | その他 | 1〜8 |
| `--dialog` `--param` `--twopoint` `--speed` `--timecontrol` | 古い形と .tra2 用 | 1〜2 |

Motolii の `script.h` は、すでに JS の頭で `// --track@` を同じ書き方で読んでいる。違いは頭の `//` が `--` になるだけ。

### シェーダーの埋め込みと呼び出し

```
--[[pixelshader@psmain:
  cbuffer constant0 : register(b0) { float bright; };
  float4 psmain(float4 pos : SV_Position) : SV_Target { ... }
]]
obj.pixelshader("psmain","object",nil,{bright/100},"add")
```

- `--[[pixelshader@登録名:` … `]]` と `--[[computeshader@登録名:` … `]]`。登録名がそのまま入口の関数名になる。
- ピクセルシェーダーの入力は 2 通りだけ：`(float4 pos : SV_Position)` か、それに `float2 uv : TEXCOORD` を足した形。uv は描く範囲で 0〜1。どちらの形かは本体が D3DReflect で見分ける。
- `obj.pixelshader(name, target, {resource...}, {constant...}, blend, sampler)`
  - target：描く先は "object" / "tempbuffer" / "framebuffer" / "cache:名前"。
  - resource：`t0` から順に入る。"random"（256×256、R32_FLOAT の乱数）も使える。描く先と同じ物は写しを作って渡す。
  - constant：Lua の数の並びを、float の並びのまま `b0` に入れる。
  - blend："copy"（既定）/ "mask"（α だけ掛ける）/ "draw"（α で重ねる）/ "add"。
  - sampler：`s0` に入る。"clip"（外は透明）/ "clamp" / "loop" / "mirror" / "dot"（補間なし）。指定しないと何も入らない。
- `obj.computeshader(name, {target...}, {resource...}, {constant...}, countX, countY, countZ, sampler)`
  - target は `u0` から順、resource は `t0` から順、定数は `b0`。数はスレッドの組の数。
- `"登録名@スクリプト名"` と書くと、別のスクリプトにあるシェーダーを借りられる。
- 中の画は RGBA16F で、乗算済みの α。

### Lua

- 既定は LuaJIT 2.1。`--script:lua` を書くと Lua 5.1.4。50 本のうち `--script:lua` を書いた物は 0 本。
- スクリプト制御で使えるのは table・string・math だけ。スクリプトファイルでは os・debug・ffi.C を外してある。つまり **ffi そのものは使える**。`obj.getpixeldata` が返すポインタを `ffi.cast` で読むのが決まった使い方になっている。
- `require` は UTF-8 の名前を受け取れるように手が入っている。ほかに `print` / `debug_print` があり、表 `global` は文字列として共有される。

## 3. obj の一覧（lua.txt にある物すべて、まとめて）

**値（読むだけの物も含む）**
- 位置など：`ox oy oz rx ry rz cx cy cz sx sy sz zoom aspect alpha`（書き換えると描き方が変わる）
- 読むだけ：`x y z w h screen_w screen_h framerate frame time totalframe totaltime layer index num id effect_id frame_s frame_e effect_layer originframe scene_id`
- 古い形の値：`track0〜3`、`check0`（50 本中 6 本が今も使う）

**描く**：`draw`、`drawpoly`（4 点・表・頂点の表の 3 つの形）、`multiobject`、`effect`

**画を読む・作る**：`load`（"movie" "movie.frame" "movie.info" "image" "text" "text.layout" "figure" "framebuffer" "tempbuffer" "layer" "before"）、`setfont`、`getfont`

**画素とバッファ**：`getpixel` `putpixel` `copypixel` `pixeloption` `getpixeldata` `putpixeldata` `copybuffer` `clearbuffer`（ふた形）

**シェーダー**：`pixelshader` `computeshader`

**設定と情報**：`setoption`（culling, billboard, blend 16 種, drawtarget, draw_state, focus_mode, camera_param, camera_focus, sampler）、`getoption`（およそ 25 種）、`getvalue`（3 つの形。基準の値・別のレイヤー・別の効果の値）、`getinfo`（およそ 15 種）、`getpoint`（.tra2 用）、`setanchor`（編集画面のつまみ点）、`getaudio`、`data`、`module`、`interpolation`、`rand`、`rand1`、`mes`

**どこからでも使える関数**：`RGB` `HSV` `OR` `AND` `XOR` `SHIFT` `rotation` `print` `debug_print` `require`、表 `global`

## 4. 人気の 50 本を数えた

### 選び方

カタログの `popularity` 順に並べ、許諾が開いている物（MIT 45、BSD-3 3、CC0 2）で、GitHub か gist にある物の上から 50 包を取った。1 つの置き場に入っている物は 1 包と数えた。中身は配布の zip（au2pkg）を落として調べた。配布が無い 6 包は、カタログが指すファイルそのものを使った。どれも実行はしていない（読んで数えただけ）。

- 入っていたファイル：anm2 40、obj2 14、lua 9（require で読む部品）、mod2 8、aux2 3、cam2 2、scn2 2、tra2 2。
- シェーダーを持つ包：**31/50（62%）**。うちコンピュートを持つのは 3。シェーダーは全部で 141 本。
- Windows の DLL を同梱する包：**10/50（20%）**。.mod2 が 8、.aux2 が 3（1 包は両方）。`obj.module` を呼ぶのは 9 包。
- 本体の効果を名前で呼ぶ包：**19/50**。名前は 29 種（多い順に 単色化 7、ぼかし 5、領域拡張 5、反転 5、クリッピング 5、リサイズ 4、マスク 3、色調補正 3、縁取り 3 …）。引数なしの `obj.effect()` は 9 包。
- LuaJIT にしか無い物を使う包：**11/50**（bit 7、ffi 4、string.buffer 3、goto 1）。

### obj を使う包の数（多い順）

| 包の数 | 関数 | | 包の数 | 値 |
|---|---|---|---|---|
| 31 | pixelshader | | 25 | oy / w / h |
| 28 | setoption | | 24 | ox |
| 26 | copybuffer / load | | 19 | cx / cy |
| 25 | effect | | 16 | oz |
| 24 | draw | | 15 | rx / ry / rz |
| 23 | getvalue | | 14 | alpha |
| 19 | clearbuffer / setanchor | | 13 | sy / index |
| 17 | getoption / getinfo | | 12 | sx / sz / cz |
| 15 | drawpoly | | 11 | time / screen_w / screen_h |
| 11 | getpixeldata | | 10 | effect_id / layer / num |
| 10 | setfont / putpixeldata | | 9 | frame |
| 9 | getpixel / module | | 6 | framerate / id / zoom / totaltime / track1 |
| 6 | multiobject | | 5 | track0 / track2 |
| 5 | rand | | 4 | track3 |
| 3 | computeshader / rand1 / pixeloption | | 3 | totalframe / check0 / aspect |
| 2 | getaudio / interpolation / getpoint | | 2 | x / y / z |
| 1 | putpixel / getfont / set(?) | | 1 | track4 / effect_layer |

細かい種類で多い物：`load` は tempbuffer 14、figure 11、text 11、layer 8、image 6。`setoption` は drawtarget 24、blend 11、draw_state 7。`getinfo` は filter 8、image_max 7、saving 5。

### 最小の obj（何個そろえると何本動くか）

よく使う順に関数をそろえていった時に、obj の呼び出しが全部そろう包の数。

| 関数の数 | obj だけで見る | DLL 付きは動かないと数える | さらに本体の効果を 1 つも作らない時 |
|---|---|---|---|
| 10 | 14 本（28%） | 13 本 | 10 本 |
| 14 | 26 本（52%） | 25 本 | 16 本 |
| **18** | **40 本（80%）** | 34 本（68%） | 23 本（46%） |
| 20 | 45 本（90%） | 38 本（76%） | 25 本（50%） |
| 28（全部） | 50 本（100%） | **40 本（80%）＝天井** | 25 本（効果 29 種をそろえると 40 本） |

上位 18 個は次の通り：pixelshader, setoption, load, effect, copybuffer, draw, getvalue, setanchor, clearbuffer, getoption, drawpoly, getinfo, getpixeldata, setfont, putpixeldata, getpixel, module, multiobject。値（39 個）は数を入れておくだけなので軽い。

### 50 本の表

× = glslang か Tint で落ちたシェーダーがある。△ = 定数の並びが HLSL と WGSL で違うシェーダーがある（5 章）。

| # | 人気 | 名前 | 許諾 | 中の obj 数 | シェーダー | DLL | LuaJIT だけの物 | 本体の効果を名前で |
|---|---|---|---|---|---|---|---|---|
| 1 | 13205 | sigma-axis.Basic_S | MIT | 38 | 50 △ | - | bit | 6 |
| 2 | 11119 | NeAc.LyricMotioN | MIT | 12 | - | - | - | - |
| 3 | 6145 | sevenc-nanashi.ntsc-rs | MIT | 23 | - | あり | - | - |
| 4 | 5610 | korarei.AutoLyricAnimation_K | MIT | 16 | - | - | - | 1 |
| 5 | 5570 | NeAc.BezierPath | MIT | 18 | - | - | - | - |
| 6 | 5300 | sigma-axis.RadRotDirBlur_S | MIT | 11 | 2 × | - | - | - |
| 7 | 5259 | korarei.FlowType_K | MIT | 41 | 5 × | あり | string.buffer | 1 |
| 8 | 5058 | sigma-axis.Path_S | MIT | 34 | 6 △ | - | bit,ffi,goto,string.buffer | - |
| 9 | 4880 | sevenc-nanashi.ported_tim | MIT | 47 | 5 △ | あり | - | 28 |
| 10 | 4610 | NeAc.CinemaScope | MIT | 5 | - | - | - | - |
| 11 | 4551 | azurite.PointZoom | MIT | 10 | 1 | - | - | 2 |
| 12 | 4157 | teruyoshii.noiseGradation | MIT | 12 | 1 △ | - | - | 1 |
| 13 | 3931 | sigma-axis.AutoClipping_S | MIT | 9 | 1 | - | ffi | 2 |
| 14 | 3886 | seilor0.twinkle_sl | MIT | 3 | 2 | - | - | - |
| 15 | 3262 | korarei.PixelSorter_K | MIT | 13 | 1 | あり | - | - |
| 16 | 3152 | sigma-axis.GroundShadow2_S | MIT | 23 | 1 | - | - | 3 |
| 17 | 3075 | seilor0.roundedPolygonSl | MIT | 3 | 1 | - | - | - |
| 18 | 2893 | azurite.ColorHalftone | MIT | 7 | 1 × | - | - | 1 |
| 19 | 2853 | sigma-axis.Tile_S | MIT | 5 | 4 △ | - | bit | - |
| 20 | 2674 | sevenc-nanashi.disassembler-anm2 | MIT | 30 | 2 | あり | - | 1 |
| 21 | 2601 | vandojapan.typing-effect | MIT | 13 | - | あり | - | - |
| 22 | 2438 | korarei.CameraTransform_K | MIT | 24 | - | あり | string.buffer | - |
| 23 | 2438 | sevenc-nanashi.transform_specific_chars-anm2 | MIT | 20 | - | あり | - | 1 |
| 24 | 2314 | NeAc.ScreenAspect | MIT | 9 | - | - | - | 2 |
| 25 | 1994 | zopty.BlindLoop | BSD-3-Clause | 3 | 1 | - | - | - |
| 26 | 1992 | sigma-axis.WhiteBalance_S | MIT | 2 | 2 | - | bit | - |
| 27 | 1927 | CatWithNeko.Halftone | CC0-1.0 | 6 | 1 △ | - | - | 1 |
| 28 | 1888 | KuroNeko.VariableFont | MIT | 7 | - | あり | - | - |
| 29 | 1863 | sigma-axis.Region_S | MIT | 35 | 4 | - | bit,ffi | 4 |
| 30 | 1761 | sevenc-nanashi.aviutl2-scripts（ドット絵変形） | MIT | 11 | 5 | - | - | - |
| 31 | 1725 | Karoterra.OilPainting | MIT | 1 | 1 | - | - | - |
| 32 | 1715 | korarei.Stylize_K | MIT | 24 | 13 | - | - | 2 |
| 33 | 1687 | sigma-axis.PageRoll_S | MIT | 26 | 2 × | - | - | 1 |
| 34 | 1642 | nctype.AutoBlendLight | MIT | 31 | 6 | - | - | 5 |
| 35 | 1634 | sagami121.BlockNoise | MIT | 14 | - | - | - | - |
| 36 | 1620 | Gohankaiju.rich-audio-spectrum | MIT | 20 | - | - | - | - |
| 37 | 1597 | beive60.scene-lighting | MIT | 7 | 1 | - | ffi | - |
| 38 | 1553 | zopty.DokodemoBuffer | BSD-3-Clause | 27 | 2 | - | - | - |
| 39 | 1521 | korarei.Kaleidoscope_K | MIT | 10 | 1 △ | - | - | - |
| 40 | 1498 | Fu-Majime.AviUtl2_PolygonMorph_F | MIT | 6 | 1 | - | bit | - |
| 41 | 1461 | korarei.Constraint | CC0-1.0 | 16 | - | - | bit | - |
| 42 | 1450 | Karoterra.Bend | MIT | 10 | - | - | - | - |
| 43 | 1436 | IZUisMe.OpenGlow | MIT | 7 | 4 | - | - | 1 |
| 44 | 1418 | korarei.Diffusion_K | MIT | 11 | 11 × △ | - | - | - |
| 45 | 1346 | zopty.OpticalFlowMotionBlur | BSD-3-Clause | 7 | 3 | - | - | - |
| 46 | 1294 | NeAc.Loop | MIT | 10 | - | - | - | - |
| 47 | 1263 | sigma-axis.PixelSnap_S | MIT | 19 | - | - | - | - |
| 48 | 1262 | NeAc.Egasumi | MIT | 7 | - | - | - | 2 |
| 49 | 1222 | arcana.connection_line | MIT | 5 | - | - | - | - |
| 50 | 1218 | azurite.AdjustPivot_A | MIT | 7 | - | あり | - | - |

人気上位でも許諾が「独自」「不明」の物（Asu、Rhapthorne_0、momo_neeeko、gometh など、Google Drive や BOOTH で配る物）は入れていない。カタログの上位 100 のうち、およそ 3 割がそういう物。

## 5. この Mac の道具と、HLSL の入口に要る物

### 道具があるか

- **Filament の glslang**（`filament/third_party/filament_src/third_party/glslang`）
  - 元ソースには HLSL を読む部分がある：`glslang/HLSL/` の hlslGrammar.cpp、hlslParseHelper.cpp ほか 7 本。上の CMakeLists.txt には `option(ENABLE_HLSL ... ON)` がある。
  - ところが Filament が実際に組み立てる `tnt/CMakeLists.txt` では `ENABLE_HLSL` が消してある（コメントになっている）。組み上がった `out/wgpu/third_party/glslang/tnt/glslang/libglslang.a` の中身 30 個に hlsl*.o は 1 つも無い。
  - → 今のままでは HLSL を読めない。7 本を足し、`-DENABLE_HLSL` を付けて組み直せば読める。
- **DXC**：無い。slangc・spirv-cross も無い。
- **Homebrew の glslang 16.6.0**（`/opt/homebrew/bin/glslangValidator`、lib あり）：HLSL を読める。下の試験はこれでやった。
- **naga 30.0.1**：ある（比べるためだけに使った）。
- **Tint**：Filament の組み立ての中に `libtint_*.a` が 66 本ある。isf.cpp と同じ `tint::spirv::reader::ReadIR` → `wgsl::writer` を呼ぶ使い捨ての試験を scratchpad で組んで回した（Motolii には何も足していない）。

### 試した結果（141 本 ＋ 本体同梱の 5 本）

| 段 | 通った | 落ちた物 |
|---|---|---|
| HLSL → SPIR-V（glslang、`-D -e 登録名`） | 137/141 | float で switch（ColorHalftone）、glslang が再帰と誤って見る（RadRotDirBlur ×2）、知らない変数（PageRoll）。どれも FXC なら通る緩い書き方 |
| SPIR-V → WGSL（Tint） | 133/137 | テクスチャの配列 `Texture2D tex[2]`（Diffusion_K、FlowType_K ×2）、glslang の ldexp が不正な SPIR-V を出す（Diffusion_K） |
| **合わせて** | **133/141（94%）** | 31 包のうち 5 包に落ちる物がある |
| 本体同梱 script.anm2 / .scn2 | 5/5 | — |

比べのため naga でも回すと 107/141。naga は並びの違いをはっきり誤りとして止める（下）。

### 定数の並び（一番大事）

Lua の `{a,b,c,...}` は、float の並びのまま `b0` に入る。HLSL の cbuffer は「16 バイトの区切りをまたがなければ詰める」決まりで、WGSL の uniform とは場所がずれる。

- 例：`float size; float2 resolution;` で、HLSL の resolution は 36 バイト目、WGSL は 40 バイト目（CatWithNeko.Halftone。glslang の Offset 飾りで 36 を確かめた）。
- `float3x3` は HLSL では 44 バイト、WGSL では 48 バイト（Basic_S の by_xyz など）。
- `float2x2` は HLSL では 24 バイト、Tint は 16 バイトずつの並びにして 32 バイト。
- Tint は止まらずに WGSL を出す。だから値が黙ってずれる。数えると、cbuffer 136 本のうち、ずれる物が少なくとも 12 本（6 包）。型を読めなかった物を入れると最大 21 本（8 包）。
- 直し方（どちらも機械で読める物だけでできる）：glslang の `TProgram::buildReflection()` が HLSL の場所を出し、Tint の WGSL の struct が WGSL の場所を出す。上げる時に float をその場所へ並べ替える。

### HLSL の入口に要る物（isf.cpp の隣に置く場合）

- glslang：`TShader(EShLangFragment か EShLangCompute)`、`setEnvInput(EShSourceHlsl, …, EShClientVulkan, 100)`、`setEntryPoint(登録名)`、`EShMsgReadHlsl`。register の番号が重ならないように `setShiftBinding` を使う。試しでは t→0〜、s→16〜、b→32、u（RWTexture は glslang では EResImage）→48 にした。どの関数もこの glslang の ShaderLang.h にあることを確かめた。
- WGSL の結び付け：t0〜t7 → texture_2d、s0 → sampler、b0 → uniform、u0 → storage texture。"random" は 256×256 の r32float を 1 枚用意する。
- 入力：`SV_Position` は @builtin(position)。D3D も WebGPU も画素の真ん中が .5 なので同じ。`TEXCOORD` の uv（描く範囲で 0〜1）は、Motolii の全面を描く頂点の段から渡す。
- blend の 4 種は WebGPU の blend の設定（乗算済み α）。sampler の "clip"（外は透明）は WebGPU に縁の色が無いので、そのままでは出せない。
- コンピュートの `RWTexture2D<half4>` は、Tint が `rgba32float, read_write` にする。中の画を rgba16float のまま読み書きするには、Dawn の TextureFormatsTier2 の機能が要る。
- 中の画は rgba16float、乗算済み α（AviUtl2 と同じ）。

## 6. Lua

- AviUtl2 の既定は **LuaJIT 2.1**。`lua.dll`（5.1.4）を使うのは `--script:lua` を書いた時と、古い形式だけ。
- 50 本のうち 11 本が LuaJIT にしか無い物を使う：`require("bit")` 7、`ffi.cast` / `ffi.typeof`（getpixeldata のポインタを読む）4、`require("string.buffer")` 3、`goto` 1。PUC Lua 5.1 では動かない。
- 借りる物：**LuaJIT**（https://luajit.org 、MIT、macOS arm64 と Windows で動く）。この Mac には入っていない（Homebrew にも無い）。
- `require` で同梱の .lua を読む包が 5 つある（Basic_S、Path_S、Region_S、ported_tim、Bend）。スクリプトのフォルダを `package.path` に入れる。
- Motolii の今のスクリプトは QuickJS（JS）なので、Lua は 2 つ目の言語になる。

## 7. YMM4（ゆっくりMovieMaker4）のプラグイン

- 公式の見本 https://github.com/manju-summoner/YukkuriMovieMaker4PluginSamples の `SampleHLSLShaderVideoEffect` は、`PixelShader.hlsl` を vcxproj で SM4.1 の **.cso（DXBC）** に変えて同梱する。
- 実際の例 panko200/ColoredCollar（sigma-axis の ColoredCollar_S を YMM4 に移した物）も `fxc.exe /T ps_5_0` で .cso を作り、`EmbeddedResource` として DLL に埋めている。
- だから配られるのは **C# の DLL と、その中の .cso**。HLSL の元は、GitHub に元ソースがある物にしか無い。
- しかも入力は Direct2D の効果の決まり（`SCENE_POSITION`、`TEXCOORD0`、`packoffset`）で、定数は C# の側で決める。HLSL の元があれば上と同じ glslang→Tint の道に乗る。ただ、何を渡すかは C# を読まないと分からない。置くだけでは動かない。
- .cso（DXBC）から SPIR-V に変える物として DXVK の DXBC 翻訳器がある（zlib）。これは確かめていない。

## 8. 危ない所

1. **定数の並び**（5 章）。黙って画が違うので、気づきにくい。並べ替えを入れないと 6〜8 包がおかしくなる。
2. **Windows の DLL**（10/50）。.mod2 / .aux2 は Mac では動かない。ntsc-rs、ported_tim、FlowType_K、PixelSorter_K など上位の物が多い。DLL の部分は作者の Rust / C++ の元ソースが公開されている物もある（ntsc-rs など）。ただ、それを Motolii 向けに組み直すのは「写す」側の仕事になる。
3. **画が同じになるか**。`obj.effect("ぼかし","範囲",…)` のように本体の効果を名前と項目名で呼ぶ所が 19 包・29 種ある。ほかにも次の物を AviUtl と同じ結果にする必要がある：
   - 仮想バッファ（drawtarget tempbuffer 24 包）
   - 合成 16 種
   - `drawpoly` の 3D と AviUtl のカメラ（Filament のカメラとの対応）
   - `load("text")` の制御文字と Windows の字体、`load("figure","円")` の図形
   - `setanchor`（編集画面のつまみ点）

   これは「効果ごとに対応表を足す」形になりやすい。
4. 細かい所：
   - glslang が FXC より厳しい（4 本）。
   - Tint がテクスチャの配列を通さない（3 本）。
   - sampler の "clip"。
   - read_write の rgba16float。
   - `--hide` などの頭書きの UI。
   - `global` 表と `cache:` バッファがフレームをまたいで共有されること。
5. 本体同梱の script.anm2 など（KENくん作）は「再配布はご遠慮ください」。Motolii に同梱はできない。使う人が自分の物を置く形になる。

## 9. 借りられる見本（先に読む物）

- **routersys/YMM4-LuaScript（MIT）**：AviUtl2 互換の `obj.pixelshader` を D3D11 で実装し、試験（AviUtlScriptShaderTests.cs）もある。"framebuffer" の扱いや blend の数値指定など、仕様に書いていない振る舞いの見本になる。LuaJIT と MoonSharp を使う。
- **kagemorikosame/sashimono-edit（MIT）**：`compat/aviutl/objapi.py`（約 84KB）に obj のほぼ全部があり、本体の効果（クリッピング・領域拡張・反転・リサイズなど）の名前の読み替えもある。シェーダーは未対応。
- **AviUtl2-IntelliSense（MIT）**：obj の関数と引数の一覧。

## 10. 手元に残した物（scratchpad。実行はしていない）

- 公式 zip と SDK：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/aviutl2/release/`（rel/lua.txt が仕様）
- 50 包の配布物：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/aviutl2/pkgs/`、置き場の写し：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/aviutl2/repos/`
- 数えた結果：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/aviutl2/scan.json`、カタログ：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/aviutl2/catalog/index.json`
- シェーダー 141 本と、その SPIR-V と WGSL：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/hlsl_test/`、Tint の試験：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/tintprobe/`
- 数えた道具（Python）：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/tools/`
