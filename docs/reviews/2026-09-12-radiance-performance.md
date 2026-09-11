# RadianceのGPU負荷削減 — 2026-09-12

利用者の「CPUが得意でない処理をやらせない」「修正」に対応。CPU画像読み戻し・光線数削減・元画像の解像度低下を導入せず、RadianceのGPU内処理を変更した。

## 変更

- Radiance Cascadesの方向事前平均とdirection-first配置。4方向の平均を一度保存し、下位段はGPUの線形補間で読む。探針位置と光線数は維持。
- 距離場の種をRGBA16FからRG16F、距離と発光判定をR16Fへ。使用していない成分を保存しない。種の有無は負の座標で表す。
- Jump Floodの最近傍比較は平方距離。すでに読んだ中央の画素を再読しない。
- 発光の有無をGPU上のmax reductionで求め、発光ゼロなら探索段の三角形を退化させてfragment実行を省く。
- Radianceの段番号をコンパイル時に固定。別のsource identityで登録し、他のエフェクトは既定で従来の動的段番号のまま。
- 共通Vism側は作者宣言に応じた中間寸法・float成分数・サンプラー・段固定を扱う。Radiance固有IDによる分岐はない。

参照: [Radiance CascadesのGPU最適化](https://mini.gmshaders.com/p/radiance-cascades2)、[論文](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/RadianceCascades.tex)、[ISF中間buffer寸法](https://docs.isf.video/ref_json)。WIDTH/HEIGHTは正の整数、整数除算、ceilによる整数tile寸法の部分集合。FILTER / CHANNELS / SPECIALIZE_PASSESはVism拡張。

## 比較方法

`radiance_preaverage_preserves_reference_pixels` が同じEngine実装とDocumentに対し、保存した修正前のshaderと修正版を別Engineで実行する。旧shaderの基準は `motolii/reference/radiance-before-2026-09-12.wgsl`。

実作品の検証コピー（SHA256 `b8cafb2e9545e97ef8dab67bddf60afc63ec9f2f5831ad4bbcdf7507dd4eeae7`）は、元のエフェクトがオフだったため、メモリ内コピーだけでオンにした。実際の作品の設定を変更していない。0/30/60/90フレームを比較し、RGBA各成分の差が3/255以内であることを検査。各地点でwarm renderを3回行い、CPU readback込みの平均を測定した。

途中の計測で修正前32.450ms、修正版25.359ms（約22%短縮）、全比較画素が許容範囲内。最終確認結果は追記する。

古い実行ファイルはshaderを埋め込んでいたため、shaderファイルだけを入れ替える比較は無効だった。その結果は性能改善の根拠に採用していない。上記は同一実行ファイルの中で2つのshaderを明示的に構築する比較。

## 検証の限界

- 修正前のRadiance一式へ戻しても、現行treeでは旧テストの「遠方の減衰」と「半径0 blurの色保持」が失敗した。段固定を外しても再現し、本修正による新規失敗とは判定していない。実作品の修正前後の画素比較と区別する。
- GPU完了待ちは引き続き編集と同じ直列キューにある。今回短くしたのはGPU処理時間であり、非同期化を完了したとは主張しない。
- 調査中に別作業の依存pinが変わった。比較試験では同じEngineでshaderのみを交換し、この交絡を避けた。
