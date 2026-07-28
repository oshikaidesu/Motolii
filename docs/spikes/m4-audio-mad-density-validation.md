# M4 音MAD編集密度 validation

状態: **GRAPH DEMAND HARNESS / GPU・UI統合未検証**

## personaの要求

音声へ合わせた大量の短い映像clip・画像・effectを、再生・scrub・細かな移動のたびに
UI停止なく扱う。平均layer数だけでなく、短clip境界の多さと要求切替を主語にする。

## fixture

製品の`Document`と`build_document_frame_graph`をそのまま使い、次を生成する。

- 1000個の動画clip
- 各clipは4 frame、開始位置は1 frameずつずらすため最大4 clipが同時active
- 16個のasset identityを繰り返し利用
- 各clipに同じshared effect definitionへの3つの`EffectUse`
- 連続300 frameと、全clip範囲を巡回する300回のscrub要求
- 各要求のgraph生成時間、active video slot数、render step数を生配列で保存

```sh
MOTOLII_AUDIO_MAD_DEMAND_OUT=/tmp/motolii-audio-mad-demand.json \
  cargo test --release -p motolii-doc --test audio_mad_density_bench \
  record_audio_mad_graph_demand_without_thresholds -- --ignored --nocapture
```

## 分離するもの

この粒はCPU側のDocument評価・plugin prepare・graph loweringだけを測る。decode、upload、
GPU effect pass、composite、display、egui drawは含めない。したがって速くてもpreview成立を
証明せず、遅ければGPU cacheでは直せないCPU側密度問題を示す。

4-frame clipを1000個並べても同時activeは最大4である。必要VRAMを総clip数へ比例させず、
active working setとcache/prefetchを分ける。一方、現行graph loweringが全TrackItemを毎要求
走査する事実はこのfixtureで観測し、実測が必要なら時間indexをM3/M4境界で別途選ぶ。

性能閾値は低スペックWindows実機と製品UI経路を測るまで固定しない。

## 2026-07-29 開発Mac観測

Apple M4 / 16GB、release test binaryで次を観測した。製品保証ではない。

| metric | sequential | scrub |
|---|---:|---:|
| median graph demand | 0.93 ms | 0.89 ms |
| p95 graph demand | 1.32 ms | 0.94 ms |
| max graph demand | 2.02 ms | 1.06 ms |

最大active video slotは設計どおり4だった。一方、frame 4のrender graphは1015 steps、frame 299でも
720 stepsを持った。現行`build_document`は未来のinactive clipも`build_clip`へ渡し、
transparentを返した後、先にactive pixelが一つ存在するとそのtransparentをcompositionへ積む。
つまり要求生成時間は現Macでまだframe budget内でも、GPUへ渡す仕事量がactive working setではなく
未来の総clip数へ比例する。

これはResourceLedgerやframe cacheで隠す対象ではない。pixel意味を保ったままinactive itemをgraphへ
出さないliveness/interval loweringが先である。ただしmask、group、parent、LookAt/Followの依存を
過小評価してはならないため、この観測だけで即座に単純skipを製品実装しない。M3 graph契約の負例と
pixel一致を別粒で閉じる。

現行step数はこのfixtureでは次で一致した。world transformは既定camera・identity transformのため
省略されている。

```text
1 transparent source + 4 × active clip + (総clip数 - 1 - 最初のactive clip index)
frame 4:   1 + 4×4 + 998 = 1015
frame 299: 1 + 4×4 + 703 = 720
```

## M3 graph-livenessへのhandoff

状態: **M4 TUNING STOP / M3 grain待ち**

Opus 5のread-only反例確認を現行codeへ再照合し、最小契約を「inactive itemを走査から外す」
ではなく、**共有`transparent_id`だと証明済みのforegroundとのidentity compositeだけを省略する**
とする。

- 全itemの訪問、frame非依存validation、`resolve_document_spaces`、mask連鎖は維持する
- `apply_envelope_opacity(transparent)`は同じtransparent IDを返す
- document/group合成でforegroundがそのIDならaccumulatorを変更せずCompositeを出さない
- mask結果、group effect結果、active opacity 0を推測でtransparent扱いしない
- 公開API、Document、plugin契約、cache keyを変えない
- pixelは現行とbit一致し、変わるのは不要stepと内部TextureId番号だけ

必須負例:

1. `active A → inactive B → clipping-mask C`でBを飛ばしてAをmaskとして誤継承しない
2. inactive時刻でも非Freeze overrunを現行どおり型付き拒否する
3. Normal/Add/Multiplyの各blendでtransparent foregroundがbackgroundとpixel一致する
4. 全child inactiveでもgroup effectが透明入力から画素を生成できる経路を消さない
5. active clipのopacity 0を「未評価でよい」と一般化しない
6. `visible=false + next_needs_mask`とinactive soloの評価契約を維持する
7. 1000/2000 clipでframe 4/299のstep数が同じになり、このfixtureでは両方20 stepsになる
8. 全clip inactive時もtransparent source、空video slots、evaluation time由来source timeを維持する

このM3粒がpixel oracleと上記負例を通るまで、現曲線からcache budget、admission閾値、
最低スペックを導出しない。M4のdecode需要測定とResourceLedger純粋ロジックは独立して続行できる。
