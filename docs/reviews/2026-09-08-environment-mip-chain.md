# 環境の解像度 — 空は 4096、映り込みは radiance の mip を粗さで読む

利用者報告(2026-09-08): 「解像度悪くない? 環境マップの時から」。裁定: CPU 描画は捨てて GPU で。

## 原因(2 つ、どちらも `compositor/environment.rs` の定数)

1. 背景の空は幅 2048 に落としていた。縦 FOV 55°・16:9 で画面に入るのは約 490 px、それを 1920 へ約 4 倍に伸ばしていた。
2. 映り込みは 64×32 に縮めた空から CPU 総当たりで作った 128×64 の板(5 段)を読んでいた。roughness 0 以外はこの 128 px の世界を映す。Glass 既定の 0.05 でも 25% 混ざる。

## 直し方

- **fork(re_renderer)**: `TextureManager2D::create_with_mipmaps` — 全 mip を確保し、upload と同じ frame encoder の後ろで 2×2 の箱平均を段ごとに描く(`resource_managers/mipmap.rs`)。`Environment` から CPU の鏡面 atlas(`specular`・`SPECULAR_LEVELS`・`prefilter_specular`)を消し、`environment_specular_along` は radiance の mip を Karis 2013 の reflection capture の段選び `1 − 1.2·log2(r)`(1×1 から数え、等距円筒は cube 面の 4 倍幅なので +2 段)で読む(`roughness_to_lod`、Rust と WGSL で鏡映)。global binding 7 を撤去。**equirect sampler の lod 上限を開けた**: `SamplerDesc` は `Default` 導出で `lod_max_clamp` が 0.0 になり、`textureSampleLevel` の lod が全部 0 に固定されていた(mip があっても読めない。`trilinear_sampler_repeat` も同じ既定で、未修正)。fork commit `2bb0f017`(branch `motolii/expose-wgpu-resources`、未 push)。
- **Motolii**: `MAX_RADIANCE_WIDTH` 2048→4096(Rgba16Float + mip で約 85MB)。radiance は `create_with_mipmaps` で上げる。照度(拡散)は低周波なので 64×32→32×16 の CPU 畳み込みのまま(100 万回、ms 単位)。

借りた定規: roughness→mip の段選びは UE4 の `ComputeReflectionCaptureMipFromRoughness`(Karis 2013)。最初に自作した `r^1.5` は roughness 0.3 でも 1024 px の世界を映して鏡と見分けが付かず、捨てた。

## 関係なかった物

Stage は comp 寸法の IOSurface をそのまま出し、Flutter は bilinear。Retina で 1080p の comp なら等倍付近。

## 検収

- fork: `levels_are_box_averages_of_the_level_above`(実 GPU: 4×4 チェッカーの 1×1 段が 0.5)、`roughness_picks_sharp_levels_for_near_mirrors_and_the_average_for_full_roughness`、`level_count_reaches_one_by_one`、既存の照度 test。
- render: 既存の環境 3 本(空が背景と照明になる、鏡 < 艶消し < ガラス、exr)が mip 経路で通過。render 43 本、ui/native 28 本。
- 一時 probe: 4096×2048 の 8px チェッカーの空で、鏡(0.02)は縞をそのまま映し、roughness 1 で反射が完全に平均色(行の標準偏差 83.7 → 0.0)。sampler を直す前は lod を 6 に固定しても変わらなかった(標準偏差 60.0)のが手掛かりだった。

## 宿題

- 箱平均 mip は極付近で等距円筒の歪みを無視する(上下の極が縞になり得る)。気になれば mip 生成を緯度重み付きに
- Cargo の rerun 参照は検証用に clone の `file://`。fork を push したら GitHub URL + `2bb0f017` に戻す
