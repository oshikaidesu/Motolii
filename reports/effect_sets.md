# エフェクトの「まとまった集」さがし（shader-lab のような物を、もっと）

調べた日：2026-10-26。数は GitHub の中のファイルを数えたもの（`gh api` の木）。星は同じ日の GitHub の数。
ライセンスは repo の LICENSE ファイル（または README のライセンスの節）を見て確かめた。
「Motolii に有る？」は ASSETS.md だけを見て決めた。

調べた集：34 個（下の表 25 ＋ 終わりの「使えない・はっきりしない」9）。

---

## 順位（よさ × 数 × ライセンスの合い方）

| 順 | 名前 | URL | ライセンス | 数 | 書き方 | 星など | Motolii に有る？ | ひとこと |
|---|---|---|---|---|---|---|---|---|
| 1 | EffectCraft | https://github.com/storytold/effectcraft | MIT OR Apache-2.0 | 307 本（GPU で動くのは 280 本） | WGSL（28 ファイルにまとめて入っている）＋ Rust | 星 7,915、フォーク 4,245（2026-10-01 にできたばかり） | 無い | After Effects の効果を名前・つまみの並び・既定値まで真似た物。Motolii の「まず真似る」にいちばん近い。WGSL なので Dawn にそのまま近い。若いので見た目は 1 本ずつ確かめが要る |
| 2 | Shaders（shaders.com） | https://github.com/shader-effects-inc/shaders | MIT（中身と部品）。エディタと Pro のプリセットは別で閉じている | 約 200 本（`packages/core/src/shaders/` のフォルダを数えて約 200） | TypeGPU（TypeScript から WGSL を作る） | 星 5,481、2026-10-06 に公開 | 無い | 数と見た目の質が高い（Halftone、Ascii、Dither、PixelSort、FlutedGlass、VHS、DataMosh、Watercolor、InkFlow…）。JS を走らせて WGSL を出す形なので、QuickJS で出すか、一度出した WGSL を持つかを考える |
| 3 | Noisemaker | https://github.com/noisefactorllc/noisemaker | MIT | 約 210 本（filter 114、synth 29、mixer 15、points 11、3D など） | WGSL と GLSL の両方が 1 本ずつ揃っている | 星 123 | 無い | 1 本ごとに `definition.js`（つまみ）＋ `wgsl/` ＋ `glsl/`。WGSL がそのまま有るのでいちばん持ち込みやすい。halftone・photocopy・pixelSort・crt・watercolor・oilPaint・physarum・lenia まで |
| 4 | Shader Lab | https://github.com/basementstudio/shader-lab | Apache-2.0 | 45 本（`*-pass.ts` 50 から素材・グループ・自作用を引いた数） | TSL（three.js の WebGPU） | 星 713、Show HN 160 点 | 無い | 今回の見本。blob tracking の上書き、ASCII、ハーフトーン、ディザ、CRT、ピクセルソート、縦溝ガラス、インク、プロッター、スミア。重ねて使う前提で作られている |
| 5 | TiXL（もと Tooll3） | https://github.com/tixl3d/tixl | MIT | 画の効果 67 ＋ 画を作る物 36 ＋ ぼかし 10（ほかに点群・粒子が 100 以上） | HLSL（DirectX 11） | 星 5,158 | 無い | デモシーン育ちの「音に合わせて動く映像」の道具。RyojiPattern（池田亮司ふう）、PixelSortGlitch、OpticalFlow、TimeDisplace、FeedbackOkLab。HLSL なので Motolii の HLSL の読み（slang）の道に乗る |
| 6 | GL Transitions | https://github.com/gl-transitions/gl-transitions | ファイルごと：MIT 123、BSD-3 1、BSD-2 1（125 本全部の頭を見た） | 125 本のトランジション | GLSL（決まった形の 1 関数） | 星 2,164 | 無い | 切り替えだけの集で、いちばん大きく、形がそろっている。ISF のトランジションの道にのせやすい |
| 7 | Lowkey Studio（loki の中） | https://github.com/SteveCastle/loki （`docs/studio/shaders/`） | MIT | 96 本（adjust、crt 20、dither 10、glitch 11、motion、music 8、beat 3、split、stylize） | libretro の slang（.slang ＋ .slangp、Vulkan の GLSL） | 星 85、Show HN | 無い | 音楽に合わせる効果（beat-glitch、beat-strobe、pulse-rings）と、PC-98・Game Boy・テレテキスト・e-ink などの「画面のまね」が多い。若い |
| 8 | Paper Shaders | https://github.com/paper-design/shaders | Apache-2.0 | 30 本 | GLSL ES 3.0（TS の文字列の中） | 星 3,833 | 無い | 数は少ないが絵の質がとても高い（liquid-metal、fluted-glass、halftone-cmyk、paper-texture、god-rays） |
| 9 | X-PostProcessing-Library | https://github.com/QianMo/X-PostProcessing-Library | MIT（中国の作者、`Effects/` の 73 本） | 73 本（グリッチ 17、ピクセル化 10、エッジ 10、ぼかし 20 …） | HLSL（Unity の CG） | 星 3,569 | 無い | グリッチの種類がとても多い（RGB Split V1〜V5、Image Block V1〜V4、Scan Line Jitter…）。同じ repo に入っている Unity 自身の標準の効果は別ライセンスなので取らない |
| 10 | VFX-JS の effects | https://github.com/fand/vfx-js （`packages/effects/src/`） | MIT | 33 本 | GLSL | 星 1,158 | 無い | 日本の作者（fand）。datamosh、bad-jpeg、pixelsort、particle-explode、matrix など、MV 向けの荒い物がそろう |
| 11 | pmndrs/postprocessing | https://github.com/pmndrs/postprocessing | Zlib | 31 本 | GLSL | 星 2,899 | 無い | three.js の後処理の定番。ASCII、Glitch、ShockWave、GodRays、Bokeh。質は安定 |
| 12 | pixi-filters | https://github.com/pixijs/filters | MIT | 37 本 | GLSL と WGSL の両方（.frag 39、.wgsl 40） | 星 1,139 | 無い | WGSL も有る。old-film、crt、glitch、godray、shockwave、ascii、cross-hatch |
| 13 | Seriously.js | https://github.com/brianchirls/Seriously.js | MIT | 59 本 | GLSL（WebGL 1） | 星 3,885（最後の更新 2022） | 無い | 古いが、動画の合成のために作られた効果の集（tvglitch、nightvision、sketch、opticalflow、chroma key） |
| 14 | cables の ImageCompose | https://github.com/cables-gl/cables （`src/ops/base/Ops.Gl.ImageCompose*`） | MIT（cables の公式の説明「パッチを動かすのに要る物は全部 MIT」。repo には LICENSE ファイルが無い） | 版違いを入れて 199、名前だけで 109 | GLSL（cables の op の形） | 星 570 | 無い | 雑音の種類が多い（Gabor、Voronoise、Hexagon、PolkaDot…）、PixelSort、LensDirt。cables の枠に結びついているので取り出す手間がある |
| 15 | GPUImage（1 と 3） | https://github.com/BradLarson/GPUImage ・ https://github.com/BradLarson/GPUImage3 | BSD-3-Clause | 1 は 140 本（GLSL ES）、3 は Metal 86 本 | GLSL ES ／ Metal | 星 20,282 ／ 2,877 | 無い | 写真・カメラの効果の古典。数は多いが、色・エッジ・ぼかしなど地味な物が中心 |
| 16 | three.js の TSL の後処理 | https://github.com/mrdoob/three.js （`examples/jsm/tsl/display/`） | MIT | 47 本 | TSL | 星 116,944 | 一部だけ：GLSL の `examples/jsm/shaders` は 30/52 本入っている。TSL 版は無い | CRT、RetroPassNode、Lensflare、Godrays、TransitionNode。Shader Lab と同じ TSL の道を作るなら一緒に入る |
| 17 | GPUPixel | https://github.com/pixpark/gpupixel | Apache-2.0 | 44 本 | GLSL（C++） | 星 2,475 | 無い | 中国の作者。美顔（顔の形・口紅・ほお紅）など、ほかの集に無い物がある |
| 18 | px-stream | https://github.com/darosh/px-stream | MIT OR CC-BY-4.0（どちらか選ぶ） | ISF 50 本、トランジション 106 | ISF（.fs）＋ GLSL | 星 131 | 無い | ISF なのでそのまま置ける。ただし Shadertoy から移した物（例「The Universe Within」）と LYGIA から移した物（「X.Lygia」4 本）は元のライセンスが残るので外す |
| 19 | ReShade 系（prod80・SweetFX・FXShaders） | https://github.com/prod80/prod80-ReShade-Repository ・ https://github.com/CeeJayDK/SweetFX ・ https://github.com/luluco250/FXShaders | 3 つとも MIT | 29 ＋ 28 ＋ 36 本 | ReShade FX（HLSL に近い） | 星 213 ／ 271 ／ 110 | 無い | 色の仕上げ（prod80 の色の道具）がとても良い。MV の「最後の色」向け |
| 20 | Vuo の画の節 | https://github.com/vuo/vuo | 節は MIT（Pro でない物）、本体は LGPL-2.1 以上 | 画の節 35 | C ＋ GLSL | 星 162 | 無い | frost、stainedGlass、crosshatch、feedback。数は中くらい |
| 21 | libretro slang-shaders | https://github.com/libretro/slang-shaders | ファイルごとにばらばら（51 本を抜き出して頭を見た：MIT 13、GPL 10、パブリックドメイン 4、BSD 2、書いていない 23） | プリセット 2,660（ほとんどは額縁 1,494。CRT 132、NTSC 32、VHS 7…） | slang（Vulkan の GLSL） | 星 474 | 無い（ASSETS の slang は HLSL の読みの話で、この集ではない） | CRT のまねでは世界一。ただし GPL の物と書いていない物が多いので、1 本ずつ選ぶ必要がある |
| 22 | lostjared/shaders | https://github.com/lostjared/shaders | MIT | GLSL 5,302 ファイル（Vulkan 版 2,508 を含む） | GLSL | 星 11 | 無い | 数だけは最大（Acid Cam の作者）。似た物の版違いが多く、質はばらばら |
| 23 | ShaderKit ／ glfx.js | https://github.com/twostraws/ShaderKit ・ https://github.com/evanw/glfx.js | MIT ／ MIT | 26 ／ 23 本 | SpriteKit の GLSL ／ GLSL | 星 711 ／ 3,458 | 無い | 小さくて読みやすい。見本としては良いが、集としては小さい |
| 24 | Unity の小さな集（PostProcessingExpansionPack・smo-shaders・image-ultra） | https://github.com/ted10401/PostProcessingExpansionPack ・ https://github.com/daniel-ilett/smo-shaders ・ https://github.com/daniel-ilett/image-ultra | MIT | .shader が 128 ／ 34 ／ 18 ファイル（版違いや同じ名前の写しを含む） | HLSL（Unity の CG） | 星 27 ／ 229 ／ 80 | 無い | DOS・GameBoy・窓の雨つぶ・油絵など。ExpansionPack には Unity 自身の標準の効果も混ざっているので、それは取らない |
| 25 | obs-shaderfilter の見本 | https://github.com/exeldro/obs-shaderfilter （`data/examples/`） | GPL-2.0 | 164 本 | HLSL に近い OBS の効果 | 星 807 | 無い | 配信の人が作った、数の多い集。GPL なので Motolii には入れられない（中を見るだけ） |

---

## 使えない・はっきりしない（調べたが外した物）

| 名前 | URL | 理由 |
|---|---|---|
| Unicorn Studio | https://unicorn.studio | 効果 70 以上だが SDK も含めて閉じている。ライセンスの書類が見つからない |
| effect.app | https://effect.app | 閉じている。商用で使うには有料の Pro が要る |
| Notch | https://www.notch.one | 閉じている（有料の製品） |
| MasterSelects | https://github.com/Sportinger/MasterSelects | 効果 34 本、WGSL だが AGPL-3.0 |
| Hydra | https://github.com/hydra-synth/hydra-synth | AGPL-3.0（星 402） |
| LYGIA | https://github.com/patriciogonzalezvivo/lygia | Prosperity Public License 3.0（商用は 30 日だけの試し）。シェーダーのファイル 2,277 本あるが、効果ではなく部品の関数 |
| Shadertoy | https://www.shadertoy.com | 何も書いていない作品は CC BY-NC-SA 3.0（商用だめ）。集める元としては使えない |
| ossia score の user library | https://github.com/ossia/score-user-library | CC-BY-SA-4.0。もう 4 本入っている（ASSETS.md） |
| godotshaders.com | https://godotshaders.com | 作品ごとに CC0・MIT・GPL-3 から選ぶ形。まとまった集ではない |

ほかに見た物：
- modV（https://github.com/vcync/modV、MIT）の ISF 309 本は Vidvox の ISF-Files の写しで、Motolii にもう有る。
- keijiro/Kino（https://github.com/keijiro/Kino、星 2,416）は 10 本ほどで、repo に LICENSE ファイルが無い。
- TouchDesigner：開いたライセンスでまとまった .tox の効果の集は見つからなかった（DBraun/TouchDesigner_Shared は GPL-3.0）。
- FoxTrotte/FXT-TweakShaderEffectsCollection は LICENSE が無い。
- glsl-film-grain（MIT）は 1 本だけ、shader-park（MIT）は SDF の道具で効果の集ではない。
- Efecto（Codrops の記事）は repo もライセンスも見つからなかった。

---

## どこで見つけたか

- GitHub の topic（post-processing、shaders、glsl-shaders、tsl、wgsl、isf、image-effects、video-effects、glitch-art、dithering、halftone、vj、touchdesigner、motion-graphics、shader-effects）を星の多い順に
- Hacker News の Show HN（Algolia の検索）：Shader Lab（160 点）、Shaders、Lowkey Studio
- 日本語「シェーダー エフェクト 集」、中国語「着色器 特效 合集」、韓国語「셰이더 이펙트 모음」：lostjared/shaders、X-PostProcessing-Library、FXT-TweakShaderEffectsCollection が出てきた
- まとめ記事（いわゆる「おすすめプラグイン」）は使っていない

---

## 次にやるなら（順に）

1. **Noisemaker**：WGSL が 1 本ずつ揃い、つまみも `definition.js` に有る。Motolii の .wgsl の道にいちばん近い。
2. **EffectCraft**：AE と同じ名前・つまみの効果が 280 本 WGSL で有る。Rust のつまみの渡し方を見本に、WGSL を取り出す。
3. **GL Transitions**：125 本の切り替え。ISF のトランジションの道にのせる。
4. **Shader Lab と three.js の TSL**：TSL を読む道を 1 本作れば、両方がいっしょに入る。
5. **Shaders（shaders.com）**：TypeGPU を QuickJS で動かして WGSL を出せるかを試す。
