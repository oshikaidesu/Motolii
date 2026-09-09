# AAの実測コストと軽量化候補

状態: **観察**。2026-09-09利用者「そのせいで重たくなってないか？外部資料に軽くする解決は？」。前回の画質検証では同条件の負荷比較が不足していたため、現行AAとOffを同じbinaryで比較した。製品の描画設定はこの調査では変更していない。

## 同条件の比較

Apple M4 / 1600×1000 / Light in form. / dev build。現行は4x MSAA＋Full tierのmeshのsample-rate shading。比較側はMSAA Off（centroid fallback）。reflection cache・GPU共有などその他の設定は同じ。各frameで実行順を交互に入れ替え、warmup 5回＋30 sampleを独立2回、各mode/条件60 sample。GPU timestampは無効にし、render_frameのCPU準備・submit・GPU完了待ち・readback回収を測る。Intent適用の時間と実窓FPSの測定ではない。

| 条件 | AA Off | 現行AA | 各mode中央値の比による増加 |
| --- | ---: | ---: | ---: |
| 静止・反射cache hit | 11.589 ms | 14.064 ms | +21.4% |
| ring回転・毎frame反射12面 | 14.827 ms | 19.297 ms | +30.2% |

CPU準備中央値は静止1.838→1.867ms、動的3.209→3.179msとほぼ同じで、差は主に完了待ちに現れた。待機時間をGPU単独の実行時間とは呼ばない。同一frameの対応差中央値は静止+3.217ms、動的+4.592msで、参考bootstrap 95%区間もいずれも正。対応差と各modeの中央値差は別の統計量である。

これは今回の作品の結果。全作品で同率になるとは主張しない。また、通常MSAAとsample-rate shadingの増分を個別には切り分けていない。

[集計](assets/2026-09-09-aa-cost/summary.json)、[run 1](assets/2026-09-09-aa-cost/run-1.json)、[run 2](assets/2026-09-09-aa-cost/run-2.json)。再実行は意図的にignoredにしたbenchmarkを使う。

```sh
MOTOLII_AA_COST_OUTPUT=/tmp/motolii-aa-cost.json cargo nextest run -p motolii-render --all-features --run-ignored only -E 'test(antialiasing_cost_comparison)' --test-threads 1
```

## 一次資料からの候補

| 候補 | 資料が示す機構 | Motoliiでの判断 |
| --- | --- | --- |
| Transient / tile-local MSAA | [Khronos/ArmのMSAA sample](https://docs.vulkan.org/samples/latest/samples/performance/msaa/README.html)は、同passでresolveし不要なmultisample/depthを保存しないこと、transient/lazy allocationを勧める | 最初に比較したい。現在のViewBuilderはinline resolveとStoreOp::Discardを既に使う。一方、MSAA color/depthのusageにTRANSIENTは付いていない。wgpu 29.0.4のTextureUsages::TRANSIENTはRA専用・Discard必須で、Apple/mobileのメモリと帯域を削減できる可能性がある。再利用するresolved画像・反射atlasへは付けない |
| 通常MSAA＋反射画像のfootprint filtering | 同[Khronos sample](https://docs.vulkan.org/samples/latest/samples/performance/msaa/README.html)は通常MSAAをpixel shading 1回として説明し、sample shadingはより高価と区別。[WGSL textureSampleGrad](https://www.w3.org/TR/WGSL/#texturesamplegrad)は明示的gradientによるsamplingを提供する | 現在のlocal_reflectionはroughnessだけでLODを決めている。画素が覆う反射画像の範囲に応じてfilterし、全面のsample shadingを外せるか比較する。cube face境界・透過・高輝度を含めて検証する必要がある |
| 鏡面専用AA | [Filament material docs](https://google.github.io/filament/main/materials.html#materialdefinitions/materialblock/anti-aliasing:specularantialiasing)、[UnityのCommonMaterial.hlsl](https://github.com/Unity-Technologies/Graphics/blob/master/Packages/com.unity.render-pipelines.core/ShaderLibrary/CommonMaterial.hlsl)は法線の変動からroughness/normal distributionをfilterする機構を示す | 光沢の細かいちらつきに対する候補。Filament自身も追加費用と、強すぎるfilterで不要に粗くなる可能性を明記する。現在より安いとは未実測。鏡に映る物体の輪郭や屈折境界をこれだけで全部解決するとは扱わない |
| SMAA 1x | [著者の論文・実装](https://www.iryoku.com/smaa/)は局所contrast・形状・対角線を使う画像ベースのAAを示す | GPU post-processとして比較できる予備候補。サブpixelの細い形やshader aliasingまで一律に回復できるとはしない。文字・alpha境界・カメラ移動を検収する。履歴を使うT2xとは区別する |
| 高輝度に強いresolve | [AMDのReversible Tonemapper](https://gpuopen.com/learn/optimized-reversible-tonemapper-for-resolve/)は、非常に明るいsampleがresolveを支配する問題を輝度依存weightで緩和する | 今の主targetはRGBA8 sRGBで、将来HDR化する際の参考。現在の全sample陰影の計算費用を直接なくす方法ではない |

通常4x MSAA、通常MSAA＋反射filter/鏡面AA、現在のsample shadingを同作品で並べて判断する。現在の重い方式は高品質の比較基準として残せるが、常用設定の最終形として固定するべきではない。TAA/履歴依存のAAは任意frame直行・逆シーク・export一致を先に解く必要があり、最初の候補にしない。

上記は検索と現行コード照合からの候補。軽量化の実装・速度改善の確認はまだ行っていない。
