# 2D・3Dの表面契約を統一する

利用者承認: 2026-09-09、re_renderer世界の統一を自走。UI・編集モデルは別の開発者の作業と分離する。
定規は[反射・屈折調査](2026-09-09-glass-raster-research.md)、既存のre_renderer mesh hookとrectangleの画像・alpha処理。新しい描画エンジンは作らない。

## 範囲と順序

1. forkのsurface入力を共通WGSLへ抽出。同じprogramがmeshとrectangleの表面を描けるようにする。既定の板は従来の画像表示、既定のmeshは従来の照明を保つ。
2. 板は位置・UV・平面法線・coverage・見かけの厚みを供給。色のdecode/filter、premultiplied alpha、clip、深度・合成の既存処理を維持。fieldによる頂点変形はこの表面契約の統一に混ぜない。
3. Motolii renderのModel限定を解消し、板にも同じcatalog/program/paramsを渡す。backdropは表面効果のある板にも届かせる。mix合成経路も含める。
4. GPUの既存Glass oracleを板へ展開し、透過・鏡・透明穴・既存mesh・合成の回帰を確認。実窓で同じ効果が画像・文字・図形・meshに届くか確認する。
5. 実際の検証結果と残件をこの文書へ記録。forkは参照可能なcommitにし、root Cargoの参照更新は直前に他の変更を再確認する。

## 検収境界

物体同士の反射用撮影、擬似法線/丸み、複数回反射は次の仕事。今回はGlassを名指しせず、新しいsurface効果が2D/3D両方に届く共通入口を完成させる。
シェーダーcompile成功と実画像/実窓の確認は区別する。既存native/UI変更は編集しない。

## 結果

実装済み。GPU出力検収済み、実窓検収は未完。

- fork: `bcf1470fcf136b3dd04cef534e54c2baeb75cc3d`。`SurfaceProgram` / `SurfaceProgramDesc`をmeshとrectangleが共有する。旧`MeshProgram`名は互換alias。WGSLの`SurfaceIn`は`shader/surface.wgsl`の1定義で、UVとcoverageも供給する。
- 色はlinear・非乗算でsurfaceへ入り、返った放射輝度にcoverageを掛けて合成する。メッシュもこの契約へ揃え、不透明度を下げても反射が明るいまま残る点を修正した。
- Motolii側の組み立てとcacheは`effects/surface_program.rs`へ集約。Model限定をTextureにも拡張し、通常runとmix-modeの両経路で背景を供給する。効果のIDによるGlass特別分岐は追加していない。
- forkの`build.rs`を実行して埋め込みshader一覧を生成した。依存として使う時は自動生成されないため、新しいshaderの追加だけでは不足することをGPU validationが検出した。
- `cargo nextest run -p motolii-render --all-features --no-fail-fast`: **47/47通過**。既存のclip、radiance、配置、preview/export一致を含む。追加oracleは白い空・赤い背景・白いガラス素材で、2DのGlass/鏡、Normal/Screen、透明穴/半透明coverageを確認する。mesh側にも鏡のopacity 0.5の検証を追加した。
- `cargo clippy -p motolii-render --tests`通過（既存warningあり、最終のopacity修正前）。最終forkで`scripts/motolii-ui.sh native`完走・exit 0。
- [適用前](assets/2026-09-09-shared-surfaces/before.png) / [同じGlassを4種類へ適用](assets/2026-09-09-shared-surfaces/glass.png)。画像は実際のDocumentを正本Engineで描いて目視したもの。実窓のスクリーンショットではない。文字、図形、alphaに穴のあるPNG、OBJの板を比較し、roughness=0.1・transmission=0.35・metallic=0を共通に設定した。
- 再描画入口: `cargo run -p motolii-render --example surface_frame -- document.rrd output.png`。比較作品・生成scriptは`/tmp/motolii-shared-surface-qa/`にあり、素材参照はローカル絶対pathなので配布用fixtureではない。

## 未完と検証上の制約

- 独立した比較用アプリはDocument読込で止まり、実窓検収を完了できなかった。process sampleは`ProbeRuntime.open → motolii_probe_open → Document::load → File::open`に留まる。原因は断定していない。最終nativeの別プロセスから同じ比較作品を開く操作は約0.23秒・error無し、正本EngineのPNG出力も成功した。
- 比較用プロセスは終了。元の作業中のMotolii窓と作品には保存・再起動・編集を行っていない。Flutter再buildはSDKの読取りで進まなかったため、不要な検証用起動を終了した。native buildは両回とも完走した。
- docs checkerは既存の「比率 aspect」行の状態`未決`を未定義語彙として拒否する。変更前HEADにも同じ状態があることを照合済み。今回の表面契約の行は`決定`。他の開発者の裁定は書き換えていない。
- fieldの板への変形、擬似法線/丸み、作品中の物体を反射用に撮影する仕組みは対象外。Glassの色は元素材の色に従い、現在の光学計算は環境層を必要とする。平面に掛けるだけで膨らんだレンズにはならない。
