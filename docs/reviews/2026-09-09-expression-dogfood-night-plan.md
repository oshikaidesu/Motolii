# 夜間に作る代表作品 — 光を受ける・届ける・空間に見せる

状態: **決定**（2026-09-09 利用者「夜間に行うべき例」「まとめて」「外部資料も検索」）。代表作品と調査方針を記録する。各方式の採用は比較中。この文書の追加時点では実装・実行予約は行っていない。

## 目的

ガラスの次に、難しい表現を実際の編集作品として作り、そのための部品を共通化して後続表現の実装負担を下げる。単発の映像や専用demo rendererを増やすことを目的にしない。作者が既存Document/Intent/効果の入口で変更でき、保存・Undo・任意時刻・exportまで届くものを検収する。

[採用方針](2026-09-09-entertainment-rendering-adoption.md)と[反射の次期計画](2026-09-09-qualiarts-rendering-next-plan.md)を継承する。作者のcamera・配置・色・形・動きは変えない。現在の共有反射の進捗は施工文書と現行codeを確認し、過去の完了報告から引き継がない。既存の反射作業が未検収なら、その穴を閉じてから本作品へ進む。

調査は**キャラクター産業（スマホ3Dライブ、ゲームVFX、制作ツール）を主軸**にする。演出上の要求と少ない計算で成立する近似を借り、実装・数式・反例はエンジン公式、公開ソース、標準仕様で補う。キャラクター作品での採用実績と、汎用エンジンの機能は同じ証拠ではない。

## 作品の優先順と検収

一晩で全部を完成させる約束ではない。N1/N2を先頭候補とし、一つずつ端から端まで閉じる。着手順は不足する共通入力と進行中の反射作業を確認して決め、朝の記録へ残す。

| ID | 作者が作る短い作品 | 共通化する対象 | 受け入れに必要な反例 |
| --- | --- | --- | --- |
| N1 プリズム | 白い光を回転する三角柱へ当て、床の虹が動く。最初は明るい背景を透かした色分散だけを分離比較 | 色ごとの屈折と厚み、光源→物体→受け面の依存、投影。分散とコースティクスは別工程 | 物体回転とcamera移動を別々に行う。光を切る/遮る、受け面を移動する、画面外へ出す。単にRGBを画面方向へずらすだけで床への虹を完了にしない |
| N2 煙と移動光 | 発生源から膨らんで消える煙へ、赤/青の光を前後左右から動かす | 発生・寿命・seed・時刻、煙素材/flipbook、光への応答、soft intersection、透過合成 | camera周回・接近・煙の内部、床と人物代替meshとの交差、逆シーク、密な重なり。6-wayの方向は照明方向であり自由視点の6画像ではない |
| N3 水面 | 波が動き、水底の光模様と映り込みが変化する | 時変法線/変形、深さ、反射/透過、投影 | 浅い/深い境界、斜視、動く遮蔽物、cameraの水面通過。投影模様と実際の水面由来の集光を区別 |
| N4 薄膜 | 変形するシャボン玉を回り込み、膜厚で虹色を変える | 薄膜干渉、角度・厚みの入力、coverage/透過 | 背景の明暗、重なる泡、薄い縁、zero strength。プリズムの分散と同じ「虹色」実装にまとめない |
| N5 異方性 | 曲がるヘアライン金属のリボンへ光を動かす | tangent/方向場、変形に追従する反射方向、粗さ | UV seam、非一様scale、ねじれ、方向反転。金属の成功を布/肌/毛髪すべての散乱の完成と呼ばない |

N1の床への虹は、物理的な光輸送、幾何に追従する近似投影、作者が配置する演出投影を比較候補として明示する。同じ見た目でも編集上の意味が異なる。演出投影の成功を自動的なプリズム分散・集光の成功へ繰り上げない。採用する意味を決めてから既存ownerへ値を載せる。

N1の分散単独比較には、dispersion=0で無分散へ戻る対照と、入射面/出射面の向きを変える対照を置く。厚みによるRGB透過近似だけなら、三角柱内部の複数面を通る光路・全反射を解いたとはしない。対象範囲を明示した縮小採用も許す。

反例は適用限界を見つけるために試すもので、全方式に全条件での成功を要求するものではない。特にN2の6-way/spriteで内部視点が破綻する場合は、cameraを勝手に制限せず、素材の適用範囲を明示した縮小採用か別方式への延期を記録する。自由に入れる体積表現を完了扱いしない。採用範囲の編集・検収が閉じれば次の題材へ進める。

## 外部資料 — 2026-09-09の確認範囲

| ID / 一次資料 | 確認できたもの | 用途と留保 |
| --- | --- | --- |
| S1 [Happy Elements・2023年MV演出ダイジェスト](https://zenn.dev/happy_elements/articles/hekk_ac_20231222) | ビームと床の交点に2種のmesh。床に平行な面は投影、折れた面は床付近のスモークで散乱した光を表現。本文確認 | N1/N2/N3の「光が届いた印象」の近似。虹の波長分散や汎用体積散乱の実装事例ではない |
| S2 [QualiArts・IDOLY PRIDE 3D制作](https://developers.cyberagent.co.jp/blog/archives/35674/) | 画面外の鏡面反射、smoothness、水面はUnity Boat Attackを参考に、水深をMayaで頂点色へ記録。本文確認 | N3。作者が制御する反射/屈折と事前計算の先例。動的形状の水深更新や水底の集光まで確認したとはしない |
| S3 [アプリボット・スマホ向けエフェクトShader](https://developers.cyberagent.co.jp/blog/archives/26521/) | Particle System上でUV、mask、歪み、輪郭透明度、SoftParticleを組合せ。本文確認 | N2。板らしい輪郭/接地を隠す機構。光が煙内部を散乱する計算とは別 |
| S4 [CyberAgent・NOVA Shader](https://github.com/CyberAgentGameEntertainment/NovaShader/blob/main/README_JA.md) | 公式READMEにflipbook、soft particles/depth fade、頂点変形、blend設定と要件。参照入口を確認 | N2の公開実装候補。Unity/URPの実装をそのままwgpuへ導入しない。採用前に該当shader・ライセンス・固定revisionを確認 |
| S5 [KLab・スクスタ CEDEC+KYUSHU 2021](https://www.klab.com/jp/blog/tech/2021/cedec-kyushu-2021-online-3d.html) / [公式掲載slide](https://www.slideshare.net/slideshow/3d-all-stars/250672570) | 登壇と公開先、slideの目次にステージのスモーク/潜れる雲を確認 | N2の追加精読先。今回SlideShareの取得本文は後半を網羅せず、公式Google Slidesは取得エラー。煙・雲の詳細algorithmは今回の確認済み根拠に含めない |
| S6 [Unity・6-way smoke lighting](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@17.0/manual/six-way-lighting.html) | 六方向の照明応答を事前生成し、実行時の照明で混合。公式manual確認 | N2の疑似体積照明。キャラクター作品での個別採用・低性能PCでの速度の証拠ではない。素材制作と視点依存・重なりの費用を含めて比較 |
| S7 [KHR_materials_dispersion](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_dispersion/README.md) | 分散の標準パラメータとvolume依存。本文確認 | N1の屈折色分散の定規。これだけでは他物体へ虹を投影しない |
| S8 [Three.js WebGPU caustics](https://threejs.org/examples/webgpu_caustics.html) / [ソース](https://github.com/mrdoob/three.js/blob/dev/examples/webgpu_caustics.html) | 公式実例とソースを閲覧 | N1/N3の集光近似の追加調査入口。今回は実行・計測せず、任意プリズムのスペクトル分散や任意受け面への対応を保証しない |
| S9 [KHR_materials_iridescence](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_iridescence/README.md) / [anisotropy](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_anisotropy/README.md) | 薄膜の厚み/IORと角度による色、異方性の強さ/回転/方向。仕様本文確認 | N4/N5。標準試料と参照画像を比較へ使い、パラメータがあるだけで適合としない |

キャラクター産業の検索範囲: Happy Elements公式Zenn、CyberAgent Developers（QualiArts/アプリボット）、KLab公式blogと講演、CEDEC公開索引、Cygames公式を、煙・雲・水面・反射・プリズム・分散・コースティクスで探索した。今回、**プリズムの床への虹、薄膜、異方性について具体的なキャラクター作品の開発者一次資料は確保できなかった**。不存在の結論ではなく追加調査の穴。S7〜S9で補い、会社や作品への帰属を推測しない。全画面プリレンダMVと実時間3Dの先例も区別する。

## 夜間の作業手順

1. 現行AGENTS、反射の施工記録、Git差分、起動中の作品を確認し、ユーザー作品を保存する。既存の未完・別作業の変更を記録する。
2. 対象をN1またはN2の一つに絞り、必要入力・既存部品・不足の対応表を作る。S資料の該当実装を精読し、revision・条件・反例を固定する。作品ごとのrendererや新しい意味の重複を作らない。
3. 部品単独の対照例から、正規Documentで編集可能な10秒程度の作品へ進む。これは比較尺の提案であり製品仕様ではない。作者のcameraを固定して、近似を変えた比較を作る。
4. パラメータ変更・Undo/Redo・保存再読込・逆シーク・同時刻再訪・exportを確認する。ノイズ/粒子は固定seedを用い、再生履歴依存の方式は任意時刻の再構成費用も検収する。
5. 同じ内容を2Dの受け手と3Dの受け手、単体と複製で比較。表面shaderで済まない照明/深度/時間の依存を既存render/re_rendererへ閉じる。効果dataはVism、編集値はDocument、UIはその窓。
6. 次の題材へ進むのは、入口→編集→結果→検証が揃ってから。終了時点の未完と次の最小作業を残し、元作品へ戻す。buildは必要な変更単位で完走させ、繰返しの全buildを進捗代わりにしない。

## 朝に見る成果物と判定

- 正規入口で開ける作品、生成手順と参照素材の所在。実行前の定規・source revision・runtime revisionを記録する。
- 静止PNGだけでなく、光/物体/cameraを別々に動かした短い動画と同時刻の比較PNG。画面外の光源位置や近似の違いが見て分かる構図にする。実窓の記録とEngineの出力を区別する。
- JSONに解像度、機器/API、品質、seed、時刻、sample/warmup、CPU準備/提出、信頼できる場合のみGPU時間、readback、画像bytes、撮影/pass数、粒数・画面占有率・透過重なりを記録。前回のMetal timestamp異常を無視してGPU値を断定しない。
- 見た目、応答、時間再現性、編集の往復、費用、適用外を別々に判定。採用/縮小/延期/棄却と理由を残す。美しい1frameやM4で動いたことだけを低性能PC対応の証拠にしない。

完成画像を事前に期待値へ置換しない。光学の定規と一致させる範囲、演出近似として動きの説得力を評価する範囲を分ける。古いPC向けの最終性能判定は対象実機が必要であり、M4で資源を制限した検証は補助証拠に留める。

## 独立した反対側レビュー（2026-09-09）

別エージェントが事実・転移条件・因果・より小さい対策を確認し、Unity 6-wayとKHR dispersionの一次資料を独立照合した。判定: 5作品を比較候補として採用、具体方式は比較中。指摘された自由視点/内部移動の過剰保証と、厚みによる分散近似を三角柱の光路へ拡張する危険に対し、上記の適用範囲・縮小採用・zero値/形状対照を追加した。資料の帰属と既決の意図補正延期との衝突は指摘されなかった。実装・性能・実窓の検収ではない。
