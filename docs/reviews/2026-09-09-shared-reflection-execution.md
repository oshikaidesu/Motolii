# 共有反射とRepeater描画の施工

状態: **決定**。2026-09-09 利用者「それで進めて」。[採用範囲](2026-09-09-entertainment-rendering-adoption.md)に従い、カメラ・作品の値を変えずに描画する。
先例はHappy Elements/Armのlocal probe、Godotの6面撮影、re_rendererのinstance batching、ISFの入力宣言。調査資料の一般性能をMotoliiの実測に読み替えない。

## 施工順

1. 表面効果の背景読出し依存をmanifestへ宣言する。背景を読まないmesh/効果ではrunを切らず、同じclip条件のmeshを既存MeshDrawDataでまとめる。依存不明のsurfaceは保守的に分割する。
2. 現在のシーンを低解像度で最大2地点×6方向へ撮る共有reflection captureを追加。撮影にはsurfaceの反射を再帰適用しない。共通SurfaceProgramが位置と方向から共有画像を読む。撮影数は複製数に比例させない。
3. 背景読出しのある透明面は従来の順序を維持。反射は2地点のprobeを混ぜる位置近似なので、自己像・視差誤差・capture原点とgeometryの関係を実画像で評価する。
4. GPU test、複製10/100/1000の背景copy/run/capture回数、カメラ変更・時刻再訪・opaque/transparentの回帰を確認。GPU時間とCPU提出/待機時間を区別する。実窓の確認を別に記録する。

## 今回の境界

自動再構図・形状変更・PPR/SSRの同時導入・新しい煙エフェクトは含めない。まず既存効果の内部効率化と、他の物体が映る共有反射を端から端まで接続する。毎frameの決定的な撮影を先に成立させ、古い反射を残す時間分散は使わない。

## 検証結果

実装済み。forkは`e71fbcd6a7345833b5827f7655b458c929fce1a0`。rootの`Cargo.toml`/lockも同じrevisionを指す。

- `BACKDROP_INPUT`をsurface manifestへ追加。Glassはtransmissionが0なら背景を読まないと宣言する。宣言のないsurfaceは従来通り背景依存として扱う。
- `surface_scene.rs`が2D・mesh・point cloudの補助ビューを組み立て、re_rendererが最大2地点の6面を描く。幾何的な端のreceiverを原点に選び、box projectionした2地点の反射を位置で混ぜる。原点選択はレイヤー列順に依存しない。カメラと作者のpropertyは書き換えない。
- surfaceがあるsceneでは主カメラだけのoffscreen除去を抑え、画面横・カメラ後方の物体も反射用に残す。反射を必要としないsceneは従来の除去を維持する。
- 同一clip平面のmeshを既存MeshDrawDataへまとめる。Rect→Modelのrun境界は重なり順のため保持する。2Dの個別rectangle draw自体は集約していない。
- atlasと6面のGPU画像を再利用。透過用の同寸法mip画像も1枚を順序付きで再利用する。毎評価時刻に反射を撮り直し、過去frameの像を残さない。

### 自動検証と計測

最終`cargo nextest run -p motolii-render --all-features --no-fail-fast --test-threads 1`は**51/51通過**。`scripts/motolii-ui.sh native`はexit 0。clippy（tests/examples/all-features）は画面外除去の最終修正前に通過、既存・行数warningあり。

追加のGPU検証は、mesh/2D双方で画面外の赤→青→赤が反射へ届き元の画素へ戻ること、カメラ変更後に元へ戻して画素が一致すること、主frustum横の反射元を残すこと、Repeater全個体が描画へ届くこと、透過3段の順序と資源再利用を確認する。

1280×720・Apple M4・warmup1回・5sampleの実測。**CPU準備＋GPU完了待ち＋readback**を含み、GPU単独時間や実窓FPSではない。

| 鏡の複製数 | 中央値 | capture | main run | 背景copy | mesh draw-data群 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10 | 18.447 ms | 12 | 2 | 0 | 3 |
| 100 | 18.427 ms | 12 | 2 | 0 | 3 |
| 1000 | 27.228 ms | 12 | 2 | 0 | 3 |

この作品には文字のrectangleがあるのでmain runは2。小さいGPU testのmesh主体の作品では1。draw-data群数をGPU drawcall数とは呼ばない。[測定値](assets/2026-09-09-shared-reflections/measurements.json)・[作品生成器](assets/2026-09-09-shared-reflections/create_scene.py)。生成器はnative build済みのmacOSとPillowを使い、出力先を引数に取る。RRDは生成先の素材を絶対pathで参照する。

[2Dとmeshの共有反射](assets/2026-09-09-shared-reflections/shared-reflection-final.png)・[1000複製](assets/2026-09-09-shared-reflections/repeater-1000-final.png)は正本Engineの出力で、実窓screen captureではない。

### 実窓

正本`motolii-ui.sh dev`で同じ比較作品を開き、カメラ後方z=-1600の色柄がmeshと2Dへ映ることを確認した。meshのroughnessを0.02→0.54へdragすると反射がぼけ、EditメニューのUndoで0.02と鮮明な像へ戻った。1000複製も表示・再生・seekを確認。尺の先で物体が消えるのは既存の帯の仕様で、seekで戻る。

Documents内の既存Flutter SDKと作品をCLIで読む処理はFile::openで停止したため、Finderでコピーした同じSDKを`FLUTTER_BIN`に指定して通常のdev経路を起動した。元SDKのsymlinkは変更していない。原因は未確定。元作品は再起動前に保存し、実窓検証後は同内容のローカル控えから復帰した。保存先は`~/.local/state/motolii-stage5/checkpoints/shared-reflection-2026-09-09/motolii-ux-current-2026-09-05.rrd`。控えのSHA256は`785caa90a758be63d04e4d1067641a16bb50aa5048021d436a77dd8221a5a625`。元のDocumentsファイルは残している。再起動に伴うUI状態の初期化あり。

### 残る近似・制約

- 最大12撮影は**評価されたsceneごと**。入れ子の独立合成があればframe全体では増える。単純な1000鏡でも60fpsを保証しない。
- 反射撮影中はsurface/fieldを保つが、共有反射と背景透過を再帰入力しない。自己像・複雑な形状・近接視差は2地点box近似の限界がある。
- 補助ビューのblendはAdd以外をNormalとして近似する。局所captureはRGBA8 sRGBでHDRではない。写っていない部分は既存HDR環境反射へfallbackする。
- 透過する物体がN枚重なる場合の背景copy・main passはN段必要。削減したのは背景不要の分割と中間画像の反復確保。1000枚の重なるガラスを定数時間にはしていない。
- PPR/SSR、時間分散、意図の自動補正、新しい煙/VAT素材は未導入。
- owned budgetとStage5検査は通過。hygieneは変更前からある旧UIの長大ファイル（最大2804行）等で不通過。変更したsequentialは917→855行へ減少。docs checkerも既存「比率 aspect」の状態`未決`を拒否し、別開発者の裁定は変更していない。
