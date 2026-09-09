# 軽量AAの施工と比較

状態: **縮小採用**。利用者「okじゃあそれやろう」。基準root `7683a48d` / fork `6592bd3c96e3b12fdd57046b1aed41e11dea7563`。[前回の負荷測定・一次資料](2026-09-09-aa-cost-and-options.md)を入口とする。別タスクの計画・3文書の未commit差分を保持する。

1. re_rendererの同passでDiscardするMSAA color/depthだけをTRANSIENT指定。外部resolved target・atlas・後続passが読む画像は対象外。
2. 旧高品質、TRANSIENT高品質、通常MSAA、通常MSAA＋反射footprint filteringを、同一binary・同一Documentで比較。品質方式とtransientを独立したRenderConfigにする。
3. filterはWGSLのpixel derivativesをfragment入口で取得し、反射/屈折rayと投影UVの変化から必要なmipを選ぶ。cube face境界では同じface基底へ隣接rayを投影し、atlasのタイル番号の跳びを微分しない。roughnessは作者の値を維持し、既存roughness LODとfootprint LODの大きい側を使う。
4. TRANSIENT前後は全画素一致必須。方式間は同じ画質を断言せず、実画像・拡大・動作を比較。相互反射・透過・文字・任意frame/Undoの既存経路を検証する。計時中にGUI操作やbuildを重ねない。

GPU timestampの逆転問題は継続して除外。CPU＋GPU完了待ち＋readbackを総時間として記録し、GPU単独時間とは呼ばない。M4のtransient_saves_memoryと論理attachment bytesを併記し、ドライバの物理VRAM実測とは区別する。画質と総費用を比較してから常用設定を決める。

## 実装と採否

fork `145ed61a3916dc556bd4d1900ceb20e9b0f977aa`を固定。RenderConfigのSurfaceSamplingをSample / Pixel / FilteredPixelに分け、TRANSIENT指定を独立させた。既定値は4x MSAA＋Sampleを維持し、TRANSIENTだけ有効化した。filterはrenderer設定と再現試験から選べる比較候補で、製品UIの品質切替にはしていない。

meshと2D rectangleのSurfaceProgramへ共通の反射・屈折footprint filteringを実装。meshの微分はcentroid位置の飛びを避けるためpixel-center varyingから取り、両面法線の符号反転を微分へ混ぜない。cube atlasは小さすぎるmipを避け、fractional LODに連続したguardを使う。HDR環境には4点の輝度重み付きsamplingを追加した。[AMDのresolve資料](https://gpuopen.com/learn/optimized-reversible-tonemapper-for-resolve/)の考えを環境lookupへ適用した候補であり、主targetのHDR resolve実装ではない。時間履歴・作者のroughness変更は使わない。

## 同条件比較

Apple M4 / dev build / 1600×1000 / Light in form.。各条件5回warmup＋30回測定を独立2回。4方式の実行順をframeごとに巡回。計時中にbuildやGUI操作を重ねていない。静止は反射cache hit、動的はring回転で反射12面を毎frame更新。CPU準備＋submit＋完了待ち＋readbackの中央値で、UI FPSやGPU単独時間ではない。

| 方式 | 静止 | 反射更新中 |
| --- | ---: | ---: |
| 高品質・通常allocation | 13.299 ms | 19.293 ms |
| 高品質・TRANSIENT（採用） | 13.403 ms | 19.272 ms |
| 通常4x MSAA・TRANSIENT | 11.590 ms | 15.965 ms |
| 4x MSAA＋軽量filter・TRANSIENT | 11.670 ms | 16.729 ms |

TRANSIENTの速度差は小さく、今回の測定では高速化を主張しない。M4の`transient_saves_memory=true`を確認。1600×1000の4x RGBA8＋Depth32で対象attachmentの論理容量は合計51,200,000 bytes（48.83 MiB）だが、この値を物理VRAM削減量とは扱わない。view数・allocationの重なりで全体量は変わる。

軽量filterは高品質TRANSIENT比で静止12.9%、動的13.2%短縮。ただし球の外周に不均一さ、細いハイライトにぼけが残る。拡大比較で画質同等の条件を満たさないため、既定値への採用は見送った。球の内部の映り込みには平滑化の効果もあり、速度だけで一律に優劣を付けない。

[集計](assets/2026-09-09-lightweight-aa/summary.json)、[run 1](assets/2026-09-09-lightweight-aa/run-1.json)、[run 2](assets/2026-09-09-lightweight-aa/run-2.json)、[採用画](assets/2026-09-09-lightweight-aa/transient_high-static-5.png)、[軽量候補](assets/2026-09-09-lightweight-aa/filtered-static-5.png)。

## 検証

全比較frameで高品質の通常allocationとTRANSIENTが全画素一致。静止PNGは従来ギャラリーと同じSHA256 `3ba1bb78eb187e9785ed322308d11b97f3a1c72da5bf12965b35fa1583f7973b`。renderの通常試験57件成功・benchmark2件ignored。追加試験は透過の有無でのallocation一致と、filter有効時の反射sender編集・Undo・カメラ切替・cache eviction・同frame再描画を検査する。既存の反射共有・2D透過・window/export画素一致も成功。UIのUndo操作そのものの検収とは区別する。

native build成功、保存済みギャラリーを再起動して実窓で文字・金属・ガラス・2D cardの表示を確認。Clippy成功（既存警告あり）、owned-budget試験成功。比較表示は736px/360pxと領域切替を確認した。docs checkerは既存の状態語「未決」で失敗。hygieneは既存の長大file 2804行・800行超23件、およびincremental 176件で失敗、target 68GiBは警告。今回のfork pinによるbuild cache増分を含み、cacheは削除していない。これらを全体check成功とは扱わない。

```sh
MOTOLII_LIGHTWEIGHT_AA_DIR=/tmp/motolii-lightweight-aa cargo test -p motolii-render --all-features lightweight_antialiasing_comparison -- --ignored --nocapture
cargo test -p motolii-render --all-features -- --test-threads=1
```
