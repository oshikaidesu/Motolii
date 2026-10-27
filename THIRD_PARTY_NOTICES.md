# 同梱した他の人の作品（Third-party notices）

Motolii は「ネットで見つけたかっこいい表現を、自分の動画にすぐ使う」ための道具です。そのために、多くの作者が公開してくれた作品を**手を入れずに**同梱し、作者の作り方のまま動かしています。作者のみなさんに感謝します。

- 同梱した物はそれぞれの作者の物で、それぞれのライセンスに従います。Motolii のリポジトリのライセンスはこれらには掛かりません。
- 各作品の隣に、作者のライセンスの文書（`LICENSE` など）と、取ってきた所（`SOURCE.txt` や `README.md`）を置いています。
- 手を入れて使っている物は、元に当てる差分（`.patch`）として分けて置き、元のソースは組み立ての時に作者の配布元から取ってきます（下の「組み立ての時に取ってくる物」）。
- 配ってよいと書かれていない作品は同梱していません。
- **作者の方へ**：載せ方に問題があれば、Issue でお知らせください。すぐに直すか、外します。

*Motolii bundles works by other authors, unchanged, each under its own licence (its licence text and where it came from sit beside it). Modified upstream code is kept as patches applied at build time. If you are an author and want a change or removal, please open an issue.*

## 1. 同梱している物

| 物 | 作者・出所 | ライセンス | 置き場所 |
|---|---|---|---|
| AviUtl2 のスクリプト 193 パッケージ（一覧は下） | 各作者。集め方は [Neosku/aviutl2-catalog-data](https://github.com/Neosku/aviutl2-catalog-data) の入れ方の手順どおり | MIT 151・CC0 27・Unlicense 6・BSD-3-Clause 6・MIT-0 2・Apache-2.0 1 | `filament/aviutl_scripts/<作者>__<名前>/`（各フォルダに LICENSE と SOURCE.txt） |
| ISF の効果 | [Vidvox/ISF-Files](https://github.com/Vidvox/ISF-Files) ほか（各 .fs の `CREDIT` に作者） | MIT | `filament/effects/*.fs`、`ISF-Files-LICENSE.txt` |
| ossia score の ISF | [ossia/score-user-library](https://github.com/ossia/score-user-library) | MIT・CC BY-SA 4.0 | `filament/effects/`、`ossia-LICENSE-ISF.txt`、`ossia-score-user-library-LICENSE.txt` |
| three.js の画面の効果（WGSL にした物） | [mrdoob/three.js](https://github.com/mrdoob/three.js) の examples/jsm/shaders | MIT | `filament/effects/*.wgsl`（各ファイルの 1 行目に出所） |
| 点群の効果 4 本 | [playcanvas/engine](https://github.com/playcanvas/engine) の scripts/esm/gsplat | MIT | `filament/effects/*.mjs`、`playcanvas-LICENSE.txt` |
| three.js（形を作る部品） | [mrdoob/three.js](https://github.com/mrdoob/three.js) r186 | MIT | `filament/third_party/js/three/` |
| d3-contour・d3-array | [d3](https://github.com/d3) | ISC | `filament/third_party/js/d3-contour/`、`d3-array/` |
| anime.js v4.5.0 | [juliangarnier/anime](https://github.com/juliangarnier/anime) | MIT | `filament/third_party/anime/` |
| Godot のイーズの式 | [godotengine/godot](https://github.com/godotengine/godot) の easing_equations.h（Robert Penner の式） | MIT | `filament/third_party/godot_easing/` |
| ピアノロール（PianoRollSvg.js） | [bladeSk/midi-to-piano-roll](https://github.com/bladeSk/midi-to-piano-roll) | MIT（作者の package.json の申告。ライセンスの文書は作者の側に無い） | `filament/third_party/midi_roll/` |
| 画面の拍子 | [nmfisher/thermion](https://github.com/nmfisher/thermion) | Apache-2.0 | `filament/third_party/thermion_frame/`（NOTICE.md） |
| ブロブの追跡 | [k1sh0r/blob-tracker](https://github.com/k1sh0r/blob-tracker) | MIT | `filament/blobtrack/kit/` |

## 2. 組み立ての時に取ってくる物（リポジトリには入っていない）

版を決めて作者の配布元から取り、Motolii の差分（`.patch`）があれば当てます。取ってくる所と版は各スクリプトの先頭にあります。

| 物 | 版 | ライセンス | 取ってくるスクリプト | Motolii の差分 |
|---|---|---|---|---|
| [Filament](https://github.com/google/filament)（Dawn・glslang を含む） | v1.77.2 | Apache-2.0 ほか | `filament/dawn_probe/build.sh` | `filament/dawn_probe/filament_dawn_compute.patch`、`glslang_hlsl.patch`、`filament/tsl/dawn_lend.patch` |
| [ThorVG](https://github.com/thorvg/thorvg) | 9203245 | MIT | `filament/dawn_probe/tvg_on_dawn.sh` | `filament/dawn_probe/thorvg_*.patch` |
| [Skia](https://skia.org/)（パスの部分だけ） | chrome/m149 | BSD-3-Clause | `filament/skia.sh` | なし |
| [libtess2](https://github.com/memononen/libtess2) | main | SGI Free Software License B 2.0 | `filament/libtess2.sh` | なし |
| [Jolt Physics](https://github.com/jrouwe/JoltPhysics) | a4b24c3 | MIT | `filament/jolt.sh` | なし |
| [BabylonJS/JsRuntimeHost](https://github.com/BabylonJS/JsRuntimeHost) の Node-API | 980cfb0 | MIT | `filament/tsl/jsrh_napi/build.sh` | `jsrh_qjs017_dawn.patch` |
| TSL の表現 48 か所 | `filament/tsl/sources.tsv` のとおり | 各作者 | `filament/tsl/fetch.sh` | `filament/tsl/*_target.patch`、`ics_tilt.patch` |

## 3. AviUtl2 のスクリプトの一覧（193）

| 名前 | 作者 | ライセンス | 配布元 | 置き場所 |
|---|---|---|---|---|
| PSD連携アニメーション(非公式) | CJF | MIT | https://github.com/ColorJellyFish/aviutl2_cjf_psdanimation | `CJF__PSD_anime` |
| ペン&矩形ツール | CJF | MIT | https://github.com/ColorJellyFish/aviutl2_cjf_pentool | `CJF__PenTool` |
| AlphaDither_C | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#alphadither_c | `CatWithNeko__AlphaDither` |
| Canny | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts | `CatWithNeko__Canny` |
| Chromatic Aberration | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#chromatic-aberration | `CatWithNeko__ChromaticAberration` |
| Colorama | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#colorama | `CatWithNeko__Colorama` |
| Composite | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts | `CatWithNeko__Composite` |
| Dither | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#dither | `CatWithNeko__Dither` |
| Erosion | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#erosion | `CatWithNeko__Erosion` |
| Exposure | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#exposure | `CatWithNeko__Exposure` |
| Gamma | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#gamma | `CatWithNeko__Gamma` |
| Halftone | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts | `CatWithNeko__Halftone` |
| Leave Color | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#leave-color | `CatWithNeko__LeaveColor` |
| Lens | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts | `CatWithNeko__Lens` |
| Level | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#level | `CatWithNeko__Level` |
| LongShadow | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts | `CatWithNeko__LongShadow` |
| MaxMin | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#maxmin | `CatWithNeko__MaxMin` |
| Mosaic_C | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#mosaic_c | `CatWithNeko__Mosaic` |
| SelfDisplace | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#selfdisplace | `CatWithNeko__SelfDisplace` |
| Tonecurve_C | こんにゃくねこ | CC0-1.0 | https://x.com/CatWithNeko/status/1972685066963960255 | `CatWithNeko__ToneCurve` |
| Turbulent | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#turbulent | `CatWithNeko__Turbulent` |
| Wave Warp | こんにゃくねこ | CC0-1.0 | https://github.com/CatWithNeko/aviutl2-scripts#wave-warp | `CatWithNeko__WaveWarp` |
| Aviutl2_CJF | ColorJellyFish | MIT | https://github.com/ColorJellyFish/Aviutl2_CJF | `ColorJellyFish__Aviutl2_CJF` |
| aviutl2_cjf_maskselector | ColorJellyFish | MIT | https://github.com/ColorJellyFish/aviutl2_cjf_maskselector | `ColorJellyFish__aviutl2_cjf_maskselector` |
| 多角形モーフ_F | フマジメ | MIT | https://github.com/fu-majime/AviUtl2_PolygonMorph_F | `Fu-Majime__AviUtl2_PolygonMorph_F` |
| 日付と時刻 | Garech | CC0-1.0 | https://www.nicovideo.jp/watch/sm36307237 | `Garech__DateTime` |
| リッチな音声波形_Gk | Gohankaiju | MIT | https://github.com/Gohankaiju/aviutl2-rich-audio-spectrum | `Gohankaiju__rich-audio-spectrum` |
| AviUtl2Browser_H | HexBrowns | MIT | https://github.com/HexBrowns/AviUtl2Browser_H | `HexBrowns__AviUtl2Browser_H` |
|  | one download for: HexBrowns.Filters_H, HexBrowns.FineParticles_H, HexBrowns.ExtendedParticle_H, HexBrowns.Lowpoly, HexBrowns.Bubbles_H, HexBrowns.VerticalText, HexBrowns.HandheldCamera_H, HexBrowns.Beam_H, HexBrowns.OpticalBokeh, HexBrowns.ShapeDistort_H, HexBrowns.Grid_H, HexBrowns.LightShaft_H, HexBrowns.ProceduralPattern_H, HexBrowns.LightPath_H, HexBrowns.RadioWaves_H, HexBrowns.DecoParticle_H, HexBrowns.Caustics_H, HexBrowns.AutoFitText_H, HexBrowns.Lighting2D_H, HexBrowns.SceneTransitions, HexBrowns.CardDance_H, HexBrowns.Extrude_H, HexBrowns.TimeWarp_H, HexBrowns.VectorBlur_H, HexBrowns.BallAction_H, HexBrowns.Keying_H, HexBrowns.TestPattern_H, HexBrowns.RangeWarning_H, HexBrowns.PlanarTracking_H, HexBrowns.ContentAwareFill_H, HexBrowns.ColorSpaceBlend_H | MIT No Attribution | https://github.com/HexBrowns/HexScript | `HexBrowns__HexScript` |
| 色付き残像(画像履歴) | HexBrowns | MIT | https://github.com/HexBrowns/ColoredEchoImg | `HexBrowns__色付き残像_画像履歴_` |
| Open Glow | 絵に描いた虹 | MIT | https://github.com/IZUisMe/aviutl2-open-glow | `IZUisMe__OpenGlow` |
| ColorLink | 犬好アトム | MIT | https://github.com/InuyoshiAtom/aviutl2-colorlink | `InuyoshiAtom__ColorLink` |
| KSA | KAZOON | MIT | https://github.com/cycloawaodorin/ksa | `KAZOON__KSA` |
| 曲げKR | かろてら | MIT | https://github.com/karoterra/aviutl2-Bend | `Karoterra__Bend` |
| 色覚シミュレーションKR | かろてら | Unlicense | https://github.com/karoterra/aviutl2-ColorVisionSimulation | `Karoterra__ColorVisionSimulation` |
| 油絵KR | かろてら | MIT | https://github.com/karoterra/aviutl2-OilPainting | `Karoterra__OilPainting` |
| Constraint | Korarei | CC0-1.0 | https://gist.github.com/korarei/d5b089a69e0c8ba5903d641d677c89e8 | `Korarei__Constraint` |
| CRTディスプレイ風.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/CRT%E3%83%87%E3%82%A3%E3%82%B9%E3%83%97%E3%83%AC%E3%82%A4%E9%A2%A8/README.md | `Nanashi___CRTディスプレイ風.anm2` |
| GPUピクセルソート.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/GPU%E3%83%94%E3%82%AF%E3%82%BB%E3%83%AB%E3%82%BD%E3%83%BC%E3%83%88/README.md | `Nanashi___GPUピクセルソート.anm2` |
| TextBackplate | なたのこ | MIT | https://github.com/Natadecoco2539/AviUtl2-TextBackplate | `Natadecoco2539__TextBackplate` |
| SoftAnchorBone | なたのこ | MIT | https://github.com/Natadecoco2539/AviUtl2-SoftAnchorBone | `Natadecoco__SoftAnchorBone` |
| 無印ノイズ | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_AviUtlNoise | `NeAc__AviUtlNoise` |
| BezierPath | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_BezierPath | `NeAc__BezierPath` |
| シネマスコープ | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_CinemaScope | `NeAc__CinemaScope` |
| エ霞(えがすみ) | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_Egasumi | `NeAc__Egasumi` |
| LongShadowN | 猫愛 | Apache-2.0 | https://github.com/nea-c/aviutl2_script_LongShadowN | `NeAc__LongShadowN` |
| Loop | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_Loop | `NeAc__Loop` |
| LyricMotioN | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_LyricMotioN | `NeAc__LyricMotioN` |
| Scatter | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_Scatter | `NeAc__Scatter` |
| 画面アスペクト | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_ScreenAspect | `NeAc__ScreenAspect` |
| スピード線 | 猫愛 | MIT | https://github.com/nea-c/aviutl2_script_SpeedLine | `NeAc__SpeedLine` |
| SeiSa | SLDC | MIT | https://github.com/samuelladoco/aviutl2-script-seisa | `SLDC__SeiSa` |
| AviUtl2-Script-GB_Style | SagyouDarui900 | MIT | https://github.com/SagyouDarui900/AviUtl2-Script-GB_Style | `SagyouDarui900__AviUtl2-Script-GB_Style` |
| DVD | サ行 | MIT | https://github.com/SagyouDarui900/AviUtl2_Script_SagyouDarui | `Sagyou__DVD` |
| JpegNoise_for_AviUtl2 | サ行 | MIT | https://github.com/SagyouDarui900/JPEGNoise_AviUtl2 | `Sagyou__JpegNoise` |
| OpticsCompensation_s_for_AviUtl2 | サ行 | MIT | https://github.com/SagyouDarui900/OpticsCompensation_s_for_AviUtl2 | `Sagyou__OpticsCompensation_s_for_AviUtl2` |
| Mathy_Su | Suzukeh | MIT | https://github.com/Suzukeh/Mathy_Su | `Suzukeh__Mathy_Su` |
| NCS風音声波形 (NCSAudioVisualizer) | tettu0530 | MIT | https://github.com/Tettu0530/NCSAudioVisualizer | `Tettu0530__NCS風音声波形__NCSAudioVisualizer_` |
| uta-resources | UtaUtaUtau | MIT | https://github.com/UtaUtaUtau/uta-resources | `UtaUtaUtau__uta-resources` |
| グランジ背景を生成 | arcana | MIT | https://github.com/ar-ca-na/aviutl2-grunge-bg | `ar-ca-na__grunge-bg` |
| 波形ライン | arcana | MIT | https://github.com/ar-ca-na/aviutl2-waveform-line | `ar-ca-na__waveform-line` |
| 連結ライン | arcana | MIT | https://github.com/ar-ca-na/connection_line | `arcana__connection_line` |
| AviUtl2-GradientEditor | azurite581 | MIT | https://github.com/azurite581/AviUtl2-GradientEditor | `azurite581__AviUtl2-GradientEditor` |
| 中心ずらし_A | azurite | MIT | https://github.com/azurite581/AviUtl2-AdjustPivot_A | `azurite__AdjustPivot_A` |
| ColorHalftone | azurite | MIT | https://github.com/azurite581/AviUtl2-ColorHalftone | `azurite__ColorHalftone` |
| グラデーション+ | azurite | CC0-1.0 | https://github.com/azurite581/AviUtl2-GradientPlus | `azurite__GradientPlus` |
| Hex-Tiling | azurite | MIT No Attribution | https://github.com/azurite581/AviUtl2-Hex-Tiling | `azurite__Hex-Tiling` |
| レイヤーグリッド表示 | azurite | MIT | https://github.com/azurite581/AviUtl2-LayerGridView | `azurite__LayerGridView` |
| ポイントズーム | azurite | MIT | https://github.com/azurite581/AviUtl2-PointZoom | `azurite__PointZoom` |
| scene-lighting | Beive60 | MIT | https://github.com/beive60/aviutl2-scene-lighting | `beive60__scene-lighting` |
| RhythmShapesForAviutl2 | coffin299 | MIT | https://github.com/coffin299/RhythmShapesForAviutl2 | `coffin299__RhythmShapesForAviutl2` |
| KSA | KAZOON | MIT | https://github.com/cycloawaodorin/ksa | `cycloawaodorin__KSA` |
| プレースホルダー | 映像学区 | MIT | https://github.com/eizo-gak/aviutl2-Placeholder | `eizo-gak__aviutl2-Placeholder` |
| aviutl2-script-skill | gasdyueer | MIT | https://github.com/gasdyueer/aviutl2-script-skill | `gasdyueer__aviutl2-script-skill` |
| PSDToolkit用自然な口パク | ただのごみ | MIT | https://github.com/gomi1124/PSDToolKit-Natural-LipSync | `gomi1124__PSDToolkit用自然な口パク` |
| aviutl2-RoundedRect | karoterra | MIT | https://github.com/karoterra/aviutl2-RoundedRect | `karoterra__aviutl2-RoundedRect` |
| AutoLyricAnimation_K | Korarei | MIT | https://github.com/korarei/AviUtl2_AutoLyricAnimation_K_Script | `korarei__AutoLyricAnimation_K` |
| AviUtl_GrungeConverter_Script | korarei | MIT | https://github.com/korarei/AviUtl_GrungeConverter_Script | `korarei__AviUtl_GrungeConverter_Script` |
| CameraTransform_K | Korarei | MIT | https://github.com/korarei/AviUtl2_CameraTransform_K_Script | `korarei__CameraTransform_K` |
| Diffusion_K | Korarei | MIT | https://github.com/korarei/AviUtl2_Diffusion_K_Script | `korarei__Diffusion_K` |
| FlowType_K | Korarei | MIT | https://github.com/korarei/AviUtl2_FlowType_K_Script | `korarei__FlowType_K` |
| Kaleidoscope_K | Korarei | MIT | https://github.com/korarei/AviUtl2_Kaleidoscope_K_Script | `korarei__Kaleidoscope_K` |
| PixelSorter_K | Korarei | MIT | https://github.com/korarei/AviUtl2_PixelSorter_K_Script | `korarei__PixelSorter_K` |
| @Stylize_K | Korarei | MIT | https://github.com/korarei/AviUtl2_Stylize_K_Script | `korarei__Stylize_K` |
| aviutl2-scripts | kusugawatsumiki | MIT | https://github.com/kusugawatsumiki/aviutl2-scripts | `kusugawatsumiki__aviutl2-scripts` |
| Unmult | mes51 | MIT | https://github.com/mes51/AVIUtl2_Unmult | `mes51__Unmult` |
| ベクターキャンバス | もげ | MIT | https://github.com/Mg32/aviutl2-vectorcanvas | `mogesystem__VectorCanvas` |
| SubObjRelativeMotionEX | natuneko722 | MIT | https://github.com/natuneko722/SubObjRelativeMotionEX | `natuneko722__SubObjRelativeMotionEX` |
| 吹き出し背景@テキスト | 夏猫 | MIT | https://github.com/natuneko722/TextBackgroundBalloonScript | `natuneko722__text-background-balloon-script` |
| AutoBlendLight | nctype | MIT | https://github.com/nctype/aviutl2_script_AutoBlendLight | `nctype__AutoBlendLight` |
| FilmPrintLUT | nctype | MIT | https://github.com/nctype/aviutl2_script_FilmPrintLUT | `nctype__FilmPrintLUT` |
| Pixelizer | nctype | MIT | https://github.com/nctype/aviutl2_script_Pixelizer | `nctype__Pixelizer` |
| PuniCollision | nctype | MIT | https://github.com/nctype/aviutl2_script_PuniCollision | `nctype__PuniCollision` |
| RainOnPane | nctype | MIT | https://github.com/nctype/aviutl2_script_RainOnPane | `nctype__RainOnPane` |
| ReflectionOnPane | nctype | MIT | https://github.com/nctype/aviutl2_script_ReflectionOnPane | `nctype__ReflectionOnPane` |
| SomethingLikeGlitch | nctype | MIT | https://github.com/nctype/aviutl2_script_SomethingLikeGlitch | `nctype__SomethingLikeGlitch` |
| AviUtl-Scripts | nea-c | MIT | https://github.com/nea-c/AviUtl-Scripts | `nea-c__AviUtl-Scripts` |
| aviutl2_script_Alignment | nea-c | MIT | https://github.com/nea-c/aviutl2_script_Alignment | `nea-c__aviutl2_script_Alignment` |
| aviutl2_script_CircularArrangementMask | nea-c | MIT | https://github.com/nea-c/aviutl2_script_CircularArrangementMask | `nea-c__aviutl2_script_CircularArrangementMask` |
| aviutl2_script_MousePointer | nea-c | MIT | https://github.com/nea-c/aviutl2_script_MousePointer | `nea-c__aviutl2_script_MousePointer` |
| aviutl2_script_ZoomJustFit | nea-c | MIT | https://github.com/nea-c/aviutl2_script_ZoomJustFit | `nea-c__aviutl2_script_ZoomJustFit` |
| VRM Toolkit for AviUtl2 | novenove | MIT | https://github.com/nove-nove/aviutl2-vrm-toolkit | `nove-nove__VRM_Toolkit_for_AviUtl2` |
| ObjectSlicerAnm2 | nssuperx | CC0-1.0 | https://github.com/nssuperx/ObjectSlicerAnm2 | `nssuperx__ObjectSlicerAnm2` |
| colorQuantizeAnm2 | nssuperx | CC0-1.0 | https://github.com/nssuperx/colorQuantizeAnm2 | `nssuperx__colorQuantizeAnm2` |
| dithering.anm2 | nssuperx | CC0-1.0 | https://github.com/nssuperx/dithering.anm2 | `nssuperx__dithering.anm2` |
| playground | nssuperx | Unlicense | https://github.com/nssuperx/playground | `nssuperx__playground` |
| AviUtl2-MIDI | rokujyushi | MIT | https://github.com/rokujyushi/AviUtl2-MIDI | `rokujyushi__AviUtl2-MIDI` |
| TTSFileRename | 黒猫大福 | MIT | https://github.com/rokujyushi/GCMZScripts | `rokujyushi__TTSFileRename` |
| VariableFont | 黒猫大福 | MIT | https://github.com/rokujyushi/Variable-Font-Plug-ins | `rokujyushi__VariableFont` |
| audio2obj | 黒猫大福 | MIT | https://github.com/rokujyushi/GCMZScripts | `rokujyushi__audio2obj` |
| midi2EAP2 | 黒猫大福 | MIT | https://github.com/rokujyushi/GCMZScripts | `rokujyushi__midi2EAP2` |
| svg2obj | 黒猫大福 | MIT | https://github.com/rokujyushi/GCMZScripts | `rokujyushi__svg2obj` |
| Aviutl2-RZoom | ryou8134 | MIT | https://github.com/ryou8134/Aviutl2-RZoom | `ryou8134__Aviutl2-RZoom` |
| BlockNoise | 3aGaM1 | MIT | https://github.com/sagami121/Aviutl2-BlockNoise | `sagami121__BlockNoise` |
| GParticle | 3aGaM1 | MIT | https://github.com/sagami121/GParticle | `sagami121__GParticle` |
| Glitch_T | 3aGaM1 | MIT | https://github.com/sagami121/Glitch_T | `sagami121__Glitch_T` |
| QRCode | 3aGaM1 | MIT | https://github.com/sagami121/Aviutl2-QRCode | `sagami121__QRCode` |
| 墨だまり | saluf | MIT | https://github.com/5qlufz-1536/AviUtl2_SumiDamari | `saluf__SumiDamari` |
| 墨だまり | saluf | MIT | https://github.com/5qlufz-1536/AviUtl2_SumiDamari | `saluf__墨だまり` |
| 角丸正多角形_sl | セイル | MIT | https://github.com/seilor0/roundedPolygon_sl | `seilor0__roundedPolygonSl` |
| twinkle_sl | セイル | MIT | https://github.com/seilor0/twinkle_sl | `seilor0__twinkle_sl` |
| ベベルとエンボス.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/%E3%83%99%E3%83%99%E3%83%AB%E3%81%A8%E3%82%A8%E3%83%B3%E3%83%9C%E3%82%B9/README.md | `sevenc-nanashi__aviutl2-scripts.bevel_and_emboss-anm2` |
| カメラフレーム.obj2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/%E3%82%AB%E3%83%A1%E3%83%A9%E3%83%95%E3%83%AC%E3%83%BC%E3%83%A0/README.md | `sevenc-nanashi__aviutl2-scripts.camera_frame-obj2` |
| CRTディスプレイ風.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/CRT%E3%83%87%E3%82%A3%E3%82%B9%E3%83%97%E3%83%AC%E3%82%A4%E9%A2%A8/README.md | `sevenc-nanashi__aviutl2-scripts.crt-display.anm2` |
| ドカベン.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/tree/main/scripts/%E3%83%89%E3%82%AB%E3%83%99%E3%83%B3 | `sevenc-nanashi__aviutl2-scripts.dokaben-anm2` |
| GPUピクセルソート.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/GPU%E3%83%94%E3%82%AF%E3%82%BB%E3%83%AB%E3%82%BD%E3%83%BC%E3%83%88/README.md | `sevenc-nanashi__aviutl2-scripts.gpu_pixelsort-anm2` |
| 出力時に非表示.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/%E5%87%BA%E5%8A%9B%E6%99%82%E3%81%AB%E9%9D%9E%E8%A1%A8%E7%A4%BA/README.md | `sevenc-nanashi__aviutl2-scripts.hide_on_export-anm2` |
| 旧発光.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/tree/main/scripts/%E6%97%A7%E7%99%BA%E5%85%89 | `sevenc-nanashi__aviutl2-scripts.legacy_glow-anm2` |
| ドット絵変形.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/tree/main/scripts/%E3%83%89%E3%83%83%E3%83%88%E7%B5%B5%E5%A4%89%E5%BD%A2#%E3%83%89%E3%83%83%E3%83%88%E7%B5%B5%E5%A4%89%E5%BD%A2anm2 | `sevenc-nanashi__aviutl2-scripts.pixel-transform-anm2` |
| 目標指定拡大縮小.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/%E7%9B%AE%E6%A8%99%E6%8C%87%E5%AE%9A%E6%8B%A1%E5%A4%A7%E7%B8%AE%E5%B0%8F/README.md | `sevenc-nanashi__aviutl2-scripts.targeted-scale-anm2` |
| 縦動画ガイド.obj2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/aviutl2-scripts/blob/main/scripts/%E7%B8%A6%E5%8B%95%E7%94%BB%E3%82%AC%E3%82%A4%E3%83%89/README.md | `sevenc-nanashi__aviutl2-scripts.vertical-video-obj2` |
| budoux.obj2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/budoux.obj2 | `sevenc-nanashi__budoux.obj2` |
| disassembler.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/disassembler.anm2 | `sevenc-nanashi__disassembler.anm2` |
| extra_blend_mode.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/extra_blend_mode.anm2 | `sevenc-nanashi__extra_blend_mode-anm2` |
| ntsc-rs.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/ntsc-rs.anm2 | `sevenc-nanashi__ntsc-rs.anm2` |
| ported_tim | Nanashi. | MIT | https://github.com/sevenc-nanashi/ported_tim | `sevenc-nanashi__ported_tim` |
| time_control_assistant.tra2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/time_control_assistant.tra2 | `sevenc-nanashi__time_control_assistant-tra2` |
| transform_specific_chars.anm2 | Nanashi. | MIT | https://github.com/sevenc-nanashi/transform_specific_chars.anm2 | `sevenc-nanashi__transform_specific_chars.anm2` |
| AutoClipping_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_AutoClipping_S | `sigma-axis__AutoClipping_S` |
| Basic_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_Basic_S | `sigma-axis__Basic_S` |
| ColoredCollar_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_ColoredCollar_S | `sigma-axis__ColoredCollar_S` |
| GroundShadow2_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_GroundShadow2_S | `sigma-axis__GroundShadow2_S` |
| Histogram_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_Histogram_S | `sigma-axis__Histogram_S` |
| 最背面に配置 | σ軸 | Unlicense | https://github.com/sigma-axis/aviutl2_script_Layback | `sigma-axis__Layback` |
| Lens_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_Lens_S | `sigma-axis__Lens_S` |
| MovingMosaic_S | σ軸 | Unlicense | https://github.com/sigma-axis/aviutl2_script_MovingMosaic_S | `sigma-axis__MovingMosaic_S` |
| PageRoll_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_PageRoll_S | `sigma-axis__PageRoll_S` |
| Path_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_Path_S/ | `sigma-axis__Path_S` |
| PixelSnap_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_PixelSnap_S | `sigma-axis__PixelSnap_S` |
| RadRotDirBlur_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_RadRotDirBlur_S | `sigma-axis__RadRotDirBlur_S` |
| Region_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_Region_S | `sigma-axis__Region_S` |
| Resize_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_Resize_S | `sigma-axis__Resize_S` |
| シーンチェンジ自作キット | σ軸 | Unlicense | https://github.com/sigma-axis/aviutl2_script_SceneChangeKit_S | `sigma-axis__SceneChangeKit_S` |
| Tile_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_Tile_S | `sigma-axis__Tile_S` |
| WhiteBalance_S | σ軸 | MIT | https://github.com/sigma-axis/aviutl2_script_WhiteBalance_S | `sigma-axis__WhiteBalance_S` |
| aviutl2_script_ColorSolid_S | sigma-axis | MIT | https://github.com/sigma-axis/aviutl2_script_ColorSolid_S | `sigma-axis__aviutl2_script_ColorSolid_S` |
| aviutl2_script_GameOfLife | sigma-axis | Unlicense | https://github.com/sigma-axis/aviutl2_script_GameOfLife | `sigma-axis__aviutl2_script_GameOfLife` |
| aviutl2_script_PenroseTile_S | sigma-axis | MIT | https://github.com/sigma-axis/aviutl2_script_PenroseTile_S | `sigma-axis__aviutl2_script_PenroseTile_S` |
| 放射クリッピング | スーパーNyan | MIT | https://github.com/supernyan74/Radial-Clipping | `supernyan74__RadialClipping` |
| CrEasy | suti7yk5032 | CC0-1.0 | https://github.com/suti7yk5032/CrEasy | `suti7yk5032__CrEasy` |
| 四隅に配置 | てるよし | MIT | https://github.com/teruyoshii/Teruyoshi_AviUtl2_Scripts/blob/main/%E5%9B%9B%E9%9A%85%E3%81%AB%E9%85%8D%E7%BD%AE.anm2 | `teruyoshii__4corner` |
| AviUtl2_Keychain_to_Transform | teruyoshii | MIT | https://github.com/teruyoshii/AviUtl2_Keychain_to_Transform | `teruyoshii__AviUtl2_Keychain_to_Transform` |
| トラックバー音声同期 | てるよし | MIT | https://x.com/teruyoshi_uec/status/1965389717224276231 | `teruyoshii__TrackbarAudioSync` |
| ノイズグラデーション | てるよし | MIT | https://x.com/teruyoshi_uec/status/1979927691215471002 | `teruyoshii__noiseGradation` |
| ノイズ_Te | てるよし | MIT | https://github.com/teruyoshii/AviUtl2_Noise_Te | `teruyoshii__noiseTe` |
| パスブラー | てるよし | MIT | https://github.com/teruyoshii/Teruyoshi_AviUtl2_Scripts/blob/main/%E3%83%91%E3%82%B9%E3%83%96%E3%83%A9%E3%83%BC.anm2 | `teruyoshii__pathBlur` |
| パスグラデーション | てるよし | MIT | https://github.com/teruyoshii/Teruyoshi_AviUtl2_Scripts/blob/main/%E3%83%91%E3%82%B9%E3%82%B0%E3%83%A9%E3%83%87%E3%83%BC%E3%82%B7%E3%83%A7%E3%83%B3.anm2 | `teruyoshii__pathGradation` |
| 段階グラデーション | てるよし | MIT | https://github.com/teruyoshii/Teruyoshi_AviUtl2_Scripts/blob/main/%E6%AE%B5%E9%9A%8E%E3%82%B0%E3%83%A9%E3%83%87%E3%83%BC%E3%82%B7%E3%83%A7%E3%83%B3.anm2 | `teruyoshii__stepGradation` |
| Liquid Glass エフェクト | tettu0530 | MIT | https://github.com/Tettu0530/Liquid-Glass | `tettu0530__LiquidGlass` |
| Aviutl2-ImageLoop-Kai | uprpr3249 | MIT | https://github.com/uprpr3249/Aviutl2-ImageLoop-Kai | `uprpr3249__Aviutl2-ImageLoop-Kai` |
| Aviutl2-RandomPlacement-Kai | uprpr3249 | MIT | https://github.com/uprpr3249/Aviutl2-RandomPlacement-Kai | `uprpr3249__Aviutl2-RandomPlacement-Kai` |
| TypingEffect | 4vando | MIT | https://github.com/vandojapan/TypingEffect.obj2 | `vandojapan__TypingEffect` |
| aviutl2_metadata_script | yuratexi | MIT | https://github.com/yuratexi/aviutl2_metadata_script | `yuratexi__aviutl2_metadata_script` |
| どこでもバッファ | zopty | BSD-3-Clause | https://github.com/zopty/DokodemoBuffer | `zopty__DokodemoBuffer` |
| OSC4AU2 | zopty | BSD-3-Clause | https://github.com/zopty/osc4au2 | `zopty__OSC4AU2` |
| PerceptualColor.anm2 | zopty | BSD-3-Clause | https://github.com/zopty/PerceptualColor.anm2 | `zopty__PerceptualColor.anm2` |
| SmoothNoise | zopty | MIT | https://github.com/zopty/SmoothNoise | `zopty__SmoothNoise` |
| どこでもバッファ | zopty | BSD-3-Clause | https://github.com/zopty/DokodemoBuffer | `zopty__どこでもバッファ` |
| ブラインドループ | zopty | BSD-3-Clause | https://gist.github.com/zopty/ff3439d79fe609c7714d6db4ef1a39a6 | `zopty__ブラインドループ` |
| 光学モーションブラー | zopty | BSD-3-Clause | https://gist.github.com/zopty/51deda495876451c1dcfb09ceb05345d | `zopty__光学モーションブラー` |
| JPEGCrush | しけもち | MIT | https://note.com/sikemoti/n/n7017a0e6eabf | `しけもち__JPEGCrush` |
| トラックバー音声同期 | てるよし | MIT | https://x.com/teruyoshi_uec/status/1965389717224276231 | `てるよし__トラックバー音声同期` |
| ノイズグラデーション | てるよし | MIT | https://x.com/teruyoshi_uec/status/1979927691215471002 | `てるよし__ノイズグラデーション` |
| パスグラデーション | てるよし | MIT | https://github.com/teruyoshii/Teruyoshi_AviUtl2_Scripts/blob/main/%E3%83%91%E3%82%B9%E3%82%B0%E3%83%A9%E3%83%87%E3%83%BC%E3%82%B7%E3%83%A7%E3%83%B3.anm2 | `てるよし__パスグラデーション` |
| パスブラー | てるよし | MIT | https://github.com/teruyoshii/Teruyoshi_AviUtl2_Scripts/blob/main/%E3%83%91%E3%82%B9%E3%83%96%E3%83%A9%E3%83%BC.anm2 | `てるよし__パスブラー` |
| 四隅に配置 | てるよし | MIT | https://github.com/teruyoshii/Teruyoshi_AviUtl2_Scripts/blob/main/%E5%9B%9B%E9%9A%85%E3%81%AB%E9%85%8D%E7%BD%AE.anm2 | `てるよし__四隅に配置` |
| 段階グラデーション | てるよし | MIT | https://github.com/teruyoshii/Teruyoshi_AviUtl2_Scripts/blob/main/%E6%AE%B5%E9%9A%8E%E3%82%B0%E3%83%A9%E3%83%87%E3%83%BC%E3%82%B7%E3%83%A7%E3%83%B3.anm2 | `てるよし__段階グラデーション` |
| ウェーブクリッピング | もも猫 | MIT | https://x.com/momo_neeeko/status/2037491184260784191 | `もも猫__ウェーブクリッピング` |
| スカラップ円 | もも猫 | MIT | https://x.com/momo_neeeko/status/2040680596410941589 | `もも猫__スカラップ円` |
| ルーローの多角形 | もも猫 | MIT | https://drive.google.com/file/d/1X7q80FYOqJINB2CIKaJ5A9nHXAusRX0r/view?usp=drive_link | `もも猫__ルーローの多角形` |
| DVD | サ行 | MIT | https://github.com/SagyouDarui900/AviUtl2_Script_SagyouDarui | `サ行__DVD` |
| プレースホルダー | 映像学区 | MIT | https://github.com/eizo-gak/aviutl2-Placeholder | `映像学区__プレースホルダー` |
| SliceGlitch | 暘 弥涼 | MIT | https://note.com/isuzu_hinode/n/n3a7d7b761b98 | `暘_弥涼__SliceGlitch` |
