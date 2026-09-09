# 表面の縁のアンチエイリアス

状態: **決定**。利用者の「縁のザラつき」への対応。基準root `ccf50b37`、比較作品はLight in form.。作品の形・カメラ・材質・解像度を固定して描画を比較する。

re_rendererのRenderConfig標準は4x MSAAだが、Motoliiのheadlessと共有deviceの2入口はOffへ上書きしていた。外部resolved targetも上流ViewBuilderはmultisample target→resolveを既に実装している。まず同じ`best_for_device_caps`を双方で使い、主描画と反射captureへ適用する。新しいAA shaderは作らない。

先例: re_renderer `context.rs` / `view_builder.rs`と、上流が参照する[MJPのMSAA解説](https://therealmjp.github.io/posts/msaa-overview/)。MSAAはgeometry coverageの改善であり、反射や光沢のshader aliasing全般を解消する保証ではない。resolve後の画像をぼかさず、renderer出力を直接比較する。中間color/depthのsample数増加とGPU費用を記録し、文字や透過を含む回帰検証を行う。

MSAA比較では輪郭の階段が改善したが、光沢面の細い明線が残ったため、meshの色・UV・法線・世界位置をcentroid補間へ揃える。[WGSLのinterpolation仕様](https://www.w3.org/TR/WGSL/#interpolation)に基づき、部分被覆pixelの中心がtriangle外でも、被覆内で属性を評価する。材質のroughnessや形状分割数は変更しない。


## 採用構成

- 主描画・反射captureともre_renderer標準の4x MSAA。headless/exportと共有device/nativeの入口を同じ関数へ揃えた。
- centroid補間も比較したが、鏡面内の細い縁の荒れが残るため、FullWebGpuSupportではmeshの色・UV・法線・世界位置をsample補間で評価する。輪郭のcoverageに加え、陰影もサンプルごとに評価する。rectangleの既存のpixel shadingとalpha合成は保持する。
- Limited tierとMSAA Offは既存のshader replacement機構でcentroidへ戻す。sample-rate shadingを持たない下位backendに新しい要件を押し付けない。
- 最終fork: `6592bd3c96e3b12fdd57046b1aed41e11dea7563`。新しいshader fileは増やしておらず、bundle一覧再生成は不要。材質・形状・カメラ・出力解像度を変えずに再描画した。

## 費用と限界

4x MSAAはmultisample color/depth targetのsample数を4倍にする。さらにFull tierのmeshではfragment shadingも最大4 sample分に増える。CPUで画像をぼかす処理は追加していない。aliasingの完全除去や無料の改善とは主張しない。局所反射自体は既存の低解像度probeによる近似で、細部の精度制約は残る。

M4 / 1600×1000 / 1回warmup後5回のsample-shading描画は20.913〜21.363ms（中央値21.137ms）。CPU・GPU待ち・readback込み。同条件を交互実行した速度比較ではないため、従来比の性能差や実窓FPSには読み替えない。

## 検証

回転した平面についてMSAA Offと標準設定を比較するGPU testを追加。meshとrectangleの両方で境界の部分被覆pixelが増え、完全に覆われた一定色の内部は全画素が不変であることを確認する。MJPのcoverageの説明を検収に写したもので、輪郭以外を画像blurして通していない。

centroid構成・最終sample構成とも55/55通過。最終clippy（all-features/tests/examples）とnative buildも完走した。共有deviceの実窓で比較作品を開き、輪郭・透過・文字を確認した。修正版をアプリに開いた状態で残している。元の作業作品は保存済みのローカル控えのままで、変更していない。

拡大比較は736px/360pxで確認し、狭い幅では縦に並ぶ。領域選択で両画像の同一viewBoxが切り替わることを確認した。最終PNGは`3ba1bb78eb187e9785ed322308d11b97f3a1c72da5bf12965b35fa1583f7973b`で、fallback追加前のFull tier結果とも全bytes一致。

最終buildでの5回描画は40.945 / 21.526 / 20.949 / 21.607 / 21.887ms（総時間）。最初のsampleには大きな振れがあり、AA導入前との速度比較には使わない。owned-budget全pattern一致、Stage5検査通過。docs検査は既存の状態語`未決`1件で不通過。

元の画像・MSAAのみ・centroid・sample shadingの各renderer出力を比較作品のdirectoryへ残す。会話内の前後比較は元PNGを埋め込み、同じ領域をpixelatedで拡大表示するだけで、画素を修正していない。

hygieneは長大な旧UI（最大2804行、閾値超過23件）とbuild後のincremental session 131（上限60）で不通過。live cacheは削除していない。
