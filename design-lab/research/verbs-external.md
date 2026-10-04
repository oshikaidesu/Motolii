# 外部ツールの動詞調査(Wave / Repeat / Random / Ease / Step / Range-Stagger)

調査日 2026-10-03。read-only。注意: WebFetch は小モデルによる要約を返すため、引用は「ページ由来だが要約器経由」。確度が低い物は (要約器) と付す。設計提案は書かない。

## 開けなかった / 未確認
- After Effects(helpx.adobe.com)全ページ 403。Wiggle / Wiggler / Echo / ease() / Sequence Layers は未確認。Repeater は検索結果の抜粋のみ(原頁未読)。
- Blender 最新版 (docs.blender.org/latest) は目次のみ返り本文なし → 2.93 版を使用。F-Curve Modifiers(Noise, Stepped)は 2.93 でも 404、未確認。
- Cavalry Stagger ノード頁 404(Noise の Stagger 属性は確認)。Cavalry Oscillator 未読。
- Apple Motion: Ramp / Replicator / Stagger 未確認。Randomize 頁の要約は「typical」と曖昧(Transition / Link Speed は裏が取れていない)。
- C4D: Step Effector の Parameter 個別名、Delay の Mode/Strength、Falloff 頁は未確認(s22 頁が別内容を返した)。
- Ableton Random(MIDI/Max)は未確認。LFO / Envelope Follower は Max for Live 頁。

## 表1 現象 x ツール x 表示名 x 文書の言い方 x ユーザーの動詞

| 現象 | ツール | 表示名 | 文書の言い方(≤15語) / URL | ユーザーの動詞 |
|---|---|---|---|---|
| Wave | Blender Wave modifier | Height | "The height or amplitude of the ripple effect" (要約器) / https://docs.blender.org/manual/en/2.93/modeling/modifiers/deform/wave.html | 揺れを大きく/小さくする (inference) |
| Wave | 同上 | Speed | "speed per frame at which the ripple propagates" (要約器) 同URL | 波を速く/遅く進める (inference) |
| Wave | 同上 | Width / Narrowness | "Half the width between tops of two subsequent ripples" (要約器) 同URL | 波の間隔を詰める/広げる (inference) |
| Wave | 同上 | Falloff | "how fast the waves fade out as they travel away" 同URL | 離れるほど弱める |
| Wave | 同上 | Time Offset | "frame at which the wave begins" 同URL | 開始をずらす (inference) |
| Wave | Cavalry Wave behaviour | Amplitude | "Set the height of the waves." https://cavalry.studio/docs/nodes/behaviours/wave/ | 高さを決める |
| Wave | 同上 | Number of Waves | "Set the number of complete waves" 同URL | 波の数を決める |
| Wave | 同上 | Travel | "Move the waves along/around the Path." 同URL | 波を流す |
| Wave | 同上 | Mode | "Set the shape of the distortion: Sine, Square, Sawtooth, or Triangle." 同URL | 波形を選ぶ |
| Wave | Apple Motion Oscillate | Amount / Speed(Frequency) / Phase | "Create a decaying oscillation" (要約器; ラベルは要約器の列挙) https://support.apple.com/guide/motion/oscillate-behavior-motn13745133/mac | 振れ幅・速さ・位相をずらす (inference) |
| Wave | Ableton LFO device | Rate | "The Rate control sets the LFO rate." https://www.ableton.com/en/manual/max-for-live-devices/ | 速さを決める |
| Wave | 同上 | Shape | "bends or skews the shape of the LFO waveform" 同URL | 波形を曲げる |
| Wave | 同上 | Depth | "Depth sets the amount of modulation for all mapped parameters." 同URL | 効きの量を決める |
| Wave | 同上 | Phase / Offset | "shifts the position of the LFO within its cycle" / "adjusts the center point" 同URL | 位相をずらす / 中心をずらす |
| Wave | Ableton Auto Filter 等 | LFO Rate / Amt | "adjust how much the low frequency oscillators affect the filter" https://www.ableton.com/en/manual/live-audio-effect-reference/ | 効きを強める/弱める |
| Repeat | Blender Array | Count | "Generates the number of copies specified in Count" https://docs.blender.org/manual/en/2.93/modeling/modifiers/generate/array.html | 何個並べるか |
| Repeat | 同上 | Relative / Constant / Object Offset | "Adds a constant translation component to the duplicate object's offset" 同URL | 1個ごとにずらす |
| Repeat | AE Repeater | Copies / Offset / Transform(Position,Rotation,Scale,Start/End Opacity) | 検索抜粋: "apply each transformation ... n times to copy number n" (原頁403) https://helpx.adobe.com/after-effects/using/shape-attributes-paint-operations-path.html | 複製して 1 個ごとに累積変形 (inference) |
| Repeat | C4D (Cloner は未読) | - | 未確認 | - |
| Random | Blender Noise Texture | Scale / Detail / Roughness / Distortion | "Scale of the base noise octave." (要約器) https://docs.blender.org/manual/en/2.93/render/shader_nodes/textures/noise.html | 粗さ・細かさを変える (inference) |
| Random | Cavalry Noise | Seed | "Changing this value will generate a new noise pattern." https://cavalry.studio/docs/nodes/behaviours/noise/ | 別の乱数にする |
| Random | 同上 | Frequency | "Set the size/scale of the Noise." 同URL | ノイズの大きさ |
| Random | 同上 | Time Scale | "multiplier to increase/decrease the speed of the Noise's evolution." 同URL | 変化を速く/遅く |
| Random | 同上 | Octaves / Lacunarity / Gain | "Set the number of noise layers combined together" 同URL | 細部を足す |
| Random | 同上 | Minimum / Maximum / Offset | "Set a minimum output value." / "Add/subtract a value to the output(s)." 同URL | 出力範囲を決める/ずらす |
| Random | Apple Motion Wriggle | Amount / Noisiness / Speed(Frequency) | "adds an additional overlay of random variance" (検索要約) https://support.apple.com/guide/motion/wriggle-behavior-motn137475d5/mac | 揺らぎを大きく・荒く・速く (inference) |
| Random | Apple Motion Randomize | Amount / Frequency / Noisiness | "creates a continuous sequence of randomly increasing and decreasing values" (検索要約) https://support.apple.com/guide/motion/randomize-behavior-motn137479d5/mac | ガタつかせる |
| Random | C4D Random Effector | Strength | "Adjust the overall strength of the effect." https://help.maxon.net/c4d/s22/us/html/OERANDOMIZE-ID_MG_BASEEFFECTOR_GROUPEFFECTOR.html | 効きを強める/弱める |
| Random | 同上 | Seed | "Changing the Seed value will result in entirely different random values." 同URL | 別の乱数にする |
| Random | 同上 | Random Mode | "Random, Gaussian, Noise, Turbulence, and Sorted" (要約器) 同URL | 分布の種類を選ぶ |
| Random | 同上 | Animation Speed / Scale | "Represents the noise's global scale" (要約器) 同URL | 時間変化の速さ / 大きさ |
| Random | 同上 | Position P[XYZ] / Scale / Rotation | "Enter the position range within which a given Effector should vary the clones." https://help.maxon.net/c4d/s22/us/html/OERANDOMIZE-ID_MG_BASEEFFECTOR_GROUPPARAMETER.html | 何をどれだけばらす |
| Random | Ableton LFO | Jitter | "adds randomness to the LFO output" https://www.ableton.com/en/manual/max-for-live-devices/ | 乱れを足す |
| Random | AE wiggle(freq, amp, ...) | freq / amp | 未確認(403) | (未確認) |
| Ease | Ableton Envelope Follower | Rise / Fall | "smooths the attack of the envelope" 同上URL | 立上り/戻りをなだらかに |
| Ease | Apple Motion Wriggle | Transition(要約器) | "Controls how smoothly the wriggle effect begins and ends" (要約器・原文は未検証) | 入り/抜きをなだらかに (inference) |
| Ease | AE グラフエディタ / ease() / Blender F-curve 補間 | - | 未確認 | - |
| Step | Ableton LFO | Steps | "adds up to 24 steps to the waveform" 同上URL | 階段状にする |
| Step | C4D Step Effector | (Parameter 名未確認) | "ascertains consecutive initial values between 0 and 1, which are then assigned to each clone." https://help.maxon.net/c4d/2025/en-us/Content/html/7443.html | クローンに 0→1 を順に割り振る |
| Step | Blender F-curve Stepped Interpolation | Step Size / Offset | 未確認(404) | (未確認) |
| Range/Stagger | Cavalry Noise | Stagger | "Set a value to offset each noise sample in time." https://cavalry.studio/docs/nodes/behaviours/noise/ | 要素ごとに時間をずらす |
| Range/Stagger | C4D Random Effector | Minimum / Maximum | "Increase or decrease internal range values" (要約器) 同Random URL | 範囲を広げる/狭める |
| Range/Stagger | C4D Effectors | Falloff | "multiply it by strength, min/max values, falloff" (要約器) https://help.maxon.net/c4d/2025/en-us/Content/html/7443.html | 距離/領域で効きを変える (inference) |
| Range/Stagger | C4D Time Offset (Effector Parameter) | Time Offset | "Generate clones from animated objects at different temporal intervals" 同Random Parameter URL | 要素ごとに時間をずらす |
| Range/Stagger | C4D Delay Effector | - | "to let them all bounce" (要約器) 同2025 URL | 遅れて追従させる |
| Range/Stagger | Blender Wave | Start Position X/Y | 検索抜粋: "coordinates of the center of the waves" | 起点からの距離で時間差 (inference) |
| Range/Stagger | AE Sequence Layers / Apple Replicator / Motion Stagger | - | 未確認 | - |

## 表2 同じ動詞で表示名が違う(意図は収束・ラベルは発散)

| 動詞(ユーザーの意図) | ラベル(ツール) |
|---|---|
| 効きを強める/弱める | Depth (Ableton LFO) / Amt (Ableton Auto Filter) / Strength (C4D Random Effector) / Amount (Apple Motion Wriggle, Randomize, Oscillate) / Height (Blender Wave) / Amplitude (Cavalry Wave) / Curl Amount (Cavalry Noise) |
| 速くする/遅くする | Rate (Ableton) / Speed (Blender Wave, Apple Wriggle) / Frequency (Apple Randomize) / Animation Speed (C4D) / Time Scale (Cavalry Noise) |
| 荒さ・細かさ | Noisiness (Apple) / Roughness, Detail (Blender Noise Tex) / Frequency, Octaves (Cavalry Noise) / Scale (C4D Random) / Jitter (Ableton) |
| 別の乱数にする | Seed (Cavalry, C4D)。Blender Noise Tex の W は「評価する座標」で別名の同機能 (inference) |
| 開始/位置をずらす | Phase, Offset (Ableton) / Phase (Apple Oscillate) / Time Offset (Blender Wave, C4D) / Offset (AE Repeater, Cavalry Noise) / Travel (Cavalry Wave) |
| 要素ごとに時間差 | Stagger (Cavalry) / Time Offset (C4D) / Delay Effector (C4D、別の効果器として存在) |
| 範囲を決める | Minimum, Maximum (Cavalry Noise, C4D Random) / P[XYZ] range (C4D) / Offset (Ableton: 中心) |
| 数を決める(複製) | Count (Blender Array) / Copies (AE Repeater) / Number of Waves (Cavalry Wave、波の数) |
| 波形を選ぶ | Mode (Cavalry Wave) / Shape (Ableton LFO は「曲げる」) / Wave chooser (Ableton Auto Filter) / Random Mode (C4D は分布) |
| 階段状 | Steps (Ableton LFO) / Step Effector (C4D は割り振りの意味で別物) |
