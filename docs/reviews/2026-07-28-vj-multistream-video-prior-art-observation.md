# VJ多数動画同時再生の先例観察（2026-07-28）

状態: **比較中**

要旨: 40本前後の動画を同時再生し、合成後の1枚へGlow等を掛ける負荷は、一般的なNLE／compositorより
VJ・メディアサーバー界隈が近い問題を長く扱ってきた。先例は「万能なpixel cache」ではなく、
表示に必要な解像度へ媒体を準備し、decoder出力をGPUへ最短で渡し、合成後post effectを低解像度階層で
処理する。本観察はHAP、hardware decode、外部texture、特定Glow algorithmの採用決定ではない。

関連:

- [性能モデル §7](../performance-model.md#7-同時レイヤー数の設計目標2026-07-08)
- [M4仕様](../specs/M4-cache-and-analysis.md)
- [メディア可搬性／GPU再調査の価値回収](2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md)
- [参考ライブラリ](../references.md)

## 1. 今回固定した問題

代表fixtureは次とする。

- 約40本の動画sourceが同じ作品時刻でactive
- 画面分割型のcollageと、全画面overlap型を分ける
- 全sourceの画素は毎frame変わり得る
- 子を順に合成したgroup compositeへ、Glowをgroup effectとして1回適用する
- per-child Glowは見た目も計算式も異なるため代替にしない
- Preview／Finalは同じ関数へ`Quality`／`FrameDesc`を渡す既存契約を維持する

このfixtureでは、初回再生の全frameが変わるため時間pixel cacheのhitを前提にできない。ただし同一区間の
再確認ではK7／K8のBake／全曲Draftが再計算をレイヤー数非依存の1系列読みへ置換できる。初回ライブと
ベイク後再生を同じ性能問題として扱わない。

## 2. 40本を一つの倍率にしない

負荷は少なくとも次の四段へ分ける。

| 段 | 支配変数 | 40本との関係 |
|---|---|---|
| decode | source解像度、codec、fps、参照frame、decoder能力 | active source数にほぼ線形 |
| upload／decoder surface | YUV／RGBA bytes、copy回数、surface pool | active source数にほぼ線形 |
| composite | 出力上のcoverage、overdraw、blend mode | 画面分割なら総coverageは約1画面、全画面overlapなら最大40画面 |
| group Glow | composite解像度、filter半径、pyramid段数 | group入力1枚に対して1回。layer数には直接依存しない |

1080p YUV 4:2:0を1.5 byte/pixelとして一度読むだけでも、
`1920 × 1080 × 1.5 × 40 × 30 ≈ 3.7 GB/s`である。RGBA8なら約`10 GB/s`となる。
これはdecode参照面、再読、blend target、Glow、cacheを含まない下限である。

一方、40本が非重複の小tileに分かれるcollageでは、rasterizerが実際に塗る総coverageは概ね出力1画面分で
あり得る。それでもdecoderは各sourceを元解像度で展開し得る。したがって本fixtureの第一仮説は、
**Glowより先に「decode費用と表示必要量の不一致」が詰まる**である。機種別順位は未測定であり断定しない。

## 3. 外部先例

### 3.1 Pro Apps／hardware decoder

[Apple Metal for Pro Apps](https://developer.apple.com/videos/play/wwdc2019/608/)は、streamごとの非同期decode、
`IOSurface`、`CVPixelBufferPool`、`CVMetalTextureCache`を組み合わせ、decode結果とMetalが同じphysical
memoryを使う経路を示す。[VideoToolboxのdecompression properties](https://developer.apple.com/documentation/videotoolbox/decompression-properties?language=objc)
にはhardware decode使用状況、reduced-resolution decode、reduced frame delivery、QoS tier、output poolの
観測／要求口がある。

[NVIDIA NVDEC application note](https://docs.nvidia.com/video-technologies/video-codec-sdk/13.0/nvdec-application-note/index.html)
は複数decode contextを並行実行できるが、総数はhardware throughputとmemoryに制約されるとする。
[NVDEC programming guide](https://docs.nvidia.com/video-technologies/video-codec-sdk/13.1/nvdec-video-decoder-api-prog-guide/index.html)
はdecode surface数を減らすとmemoryは下がる一方、surface再利用増加により個別stream throughputが下がり得る
交換を明記する。ゆえにdecoder surface、DPB、output surface、先読みringはResourceLedgerの外に置けない。

これらのAPIはMotolii公開plugin契約へ出さず、失敗時に既存YUV upload／software decode／proxyへ落ちる
platform adapterとしてのみ比較する。

### 3.2 VJのGPU-native media

[HAP](https://hap.video/developers)はstandard compressed textureへ軽い二次圧縮を掛け、再生時に
compressed texture dataをOpenGL／Metal／Direct3Dへ渡す。公式repositoryは「複数の高解像度動画を
real-timeで扱い、CPUが制約になる場面」を目的として明示し、BSD-2-Clauseの仕様・reference source・
test materialを公開する（[Vidvox/hap](https://github.com/Vidvox/hap)）。

これはdecode engine／CPU codecを、より大きなdisk streamとGPU samplerへ付け替える先例である。ただし、

- long-GOPよりdisk容量／帯域が増える
- lossy compressed textureでありFinal同値を自動では満たさない
- adapterのcompressed texture featureとformat差がある
- alpha／色空間／奇数寸法／cropをfixtureで閉じる必要がある
- FFmpegにHAP codecがあることと、圧縮textureを展開せずGPUへ渡せることは同義でない

ため、現時点では**多層Draft向けproxy tier候補**であり、Final形式、必須依存、K4の採択codecではない。

### 3.3 decoder surfaceの直接sample

[WebGPU external texture](https://gpuweb.github.io/gpuweb/#external-texture)は、video frameを最大3 planeと
色変換metadataを持つsample-only textureとして表現し、実装条件が合えばsource copyなしでimportできる。
Motoliiで同じ責任を採る場合も、OS／backend固有surfaceをDocument、plugin API、永続cache formatへ出さない。

plain video layerはYUV planeを合成時に直接sampleできる余地がある。一方、個別pixel effectを持つlayerは
pluginの正準RGBA入力へ実体化する必要がある。二経路はHost内部のplanで選び、色行列、range、chroma
siting、premultiplyのgoldenを共用する。現在の`YuvToRgba`を迂回して別の色authorityを作ってはならない。

### 3.4 合成後Glow

[Unreal Bloom](https://dev.epicgames.com/documentation/en-us/unreal-engine/bloom-in-unreal-engine)は広いblurを
1/4〜1/32解像度へ置き、複数scaleを合成する。[Unity URP Bloom](https://docs.unity3d.com/ja/current/Manual/urp/post-processing-bloom.html)
はlow-end向けにQuarter開始、filter品質、iteration数の縮退を公開する。
[AMD FidelityFX SPD](https://gpuopen.com/fidelityfx-spd/)はBloom等で使うmip列をpatch単位の1 dispatchで
生成し、level間の全GPU同期を減らす。

Motoliiへ移す核は、子へGlowを分配することではなく、**group composite後に一度だけ明部抽出と最初の
downsampleを行い、広い半径ほど低解像度で処理する**ことである。ただし低解像度pyramid、dual filter、
Gaussian、FFT convolutionは同じpixel意味ではない。既存plugin意味を無言で差し替えず、plugin自身の
algorithmまたは`Quality`別の正式な結果としてfixtureを締結する。

## 4. Motoliiへの候補処分

| 候補 | 処分 | 理由 |
|---|---|---|
| decoder surface／DPB／先読みringをResourceLedgerへ載せる | REUSE／BUILD | hard budgetの既存責任をdecodeまで貫通する |
| OS別hardware decodeとzero-copy import | WRAP | 公開契約へvendor／OS型を出さず、fallbackを必須にする |
| 表示coverage／Qualityに応じたproxy解像度選択 | BUILD候補 | source解像度と表示必要量の不一致を直接減らす |
| plain動画のYUV plane直接合成 | SPIKE | RGBA中間を減らせるが色一元化とeffect境界の審判が必要 |
| HAP型GPU-native proxy | 比較継続 | VJの直接先例だがdisk、lossy、format portabilityを未測定 |
| 合成後の低解像度Glow pyramid | PATTERN | layer数非依存の標準部品。既存pixel意味を変えない条件つき |
| per-child Glow | REJECT | group effectと非等価 |
| hardware overlay | REJECT | offscreen group composite＋Glowから外れ、最終合成を作れない |
| 動画atlasへの毎frame再packing | REJECT | copyを増やし、decoder surface直結を壊す |

## 5. 検証順

### S1. 多stream decode天井

同一codec／解像度／fpsのframe番号入り素材をN本へ増やす。decode throughput、software fallback、CPU、
decoder surface／RSS／VRAM、deadline missを機種別に記録する。無言software fallback、単調増加、
editor thread blockを負oracleとする。

### S2. 表示必要量へ揃えるproxy

40本を非重複gridと全画面overlapの二fixtureにし、原本、通常proxy、GPU-native proxy候補を比較する。
source decode pixel数、disk read、upload bytes、合成coverage、画質差を別々に記録する。proxyへ色解釈、
LUT、Document意味を焼いた時点で失敗とする。

### S3. YUV→合成経路

現行`YuvToRgba`、YUV plane直接sample、利用可能な実機だけzero-copy surface importを比較する。
BT.601／709、limited／full、chroma siting、alpha境界のgoldenを共有し、経路ごとに別の色authorityを
作らない。zero-copy不能は失敗でなく通常fallbackとする。

### S4. 合成後Glow

decodeを手続き生成textureへ置換して切り離し、全解像度参照、pyramid、dual-filter候補を比較する。
GPU pass時間、target生存量、banding、shimmer、NaNを測る。Glowが支配項でなければ追加最適化を止め、
decode／proxyへ投資を戻す。

## 6. 旧ノートPCでの縮退経路

1. capability probeでhardware decodeとformatを観測し、無言fallbackを禁止する
2. 表示coverageに合うproxyを既定利用する
3. Draftを1/2→1/4へ下げ、Glowの基底解像度／段数も`Quality`で縮退する
4. 再生deadline超過時はaudio／Transport時刻を遅らせず`t`以下の最新frameを表示する
5. backgroundでK8全曲Draft coverageを増やし、2周目以降を合成済み1系列へ置換する

初回から40本の原本を実時間decodeできるとは公約しない。「編集は即時に反映し、止めている間に通し再生が
滑らかになる」を製品経路とする。proxy／Bake完了を編集操作の前提にはしない。

## 7. 非目標と停止線

- HAP／BC／hardware decodeをFinal、Document、plugin公開APIの意味へすること
- 40本fixtureをGlow benchmarkだけで閉じること
- cache、memory削減、decode削減、pass削減を同じ性能改善として報告すること
- 仮想GPUや単一高性能GPUから旧iGPUの実時間性能を保証すること
- Draftの結果差をcacheの透明性と混同すること
- per-child処理、frame history、隠れた可変状態でgroup effectを近似すること
- 実測前にproxy閾値、decoder本数、先読みframe数を固定すること

## 8. Fable協力の処分

Fable 5にはread-onlyで、decode、zero-copy、VJ codec、Glow、old laptopの反例探索を依頼した。採用したのは
「Glowはlayer数非依存」「decodeが第一仮説」「VJのGPU-native mediaを候補に戻す」「初回とBake後を
分離する」の四点である。次は縮小した。

- 「HAPはFFmpeg実装をそのまま使えばcompressed texture直結」: 未証明。reference decoderとFFmpegの
  pixel展開経路を分けて調べる
- 「低解像度Glowが最大のbreakthrough」: Glow標準部品としては支持するが、40本に線形なdecodeを減らさない
- 数値threshold: 実測前なので採らない
