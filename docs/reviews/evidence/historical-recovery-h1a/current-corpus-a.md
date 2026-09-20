# H1a 現行照合A: root/specs/mocks 35 docs 現行証拠台帳

作成日: 2026-07-22
状態: **観察**（現行docsのtopic対応証拠台帳。最終意味裁定ではない。完全回収/未回収などの最終処置分類はB〜E統合後に行う）

## 1. 状態

本書はH1a全5 batch中のbatch A（root/specs/mocksの固定35 path）の**現行証拠観察**である。決定は変更しない。現行docsの決定状態・仕様ID・公開契約・実装は変更していない。

## 2. snapshot/manifest/hash/件数/行数の開始終了確認

作業開始時点で以下を実測し、発注書の期待値と一致することを確認した。

```
$ shasum -a 256 /tmp/motolii-h1a-current-a.C1B7C4/paths.txt
429c5a80a4c82b76384c139b84cf4ef4efd499b95048ffc52f7f30e0c4844ad3  /tmp/motolii-h1a-current-a.C1B7C4/paths.txt

$ wc -l /tmp/motolii-h1a-current-a.C1B7C4/paths.txt
35 /tmp/motolii-h1a-current-a.C1B7C4/paths.txt

$ cd /tmp/motolii-docs-history1-current.BgHOsk && while IFS= read -r p; do wc -l "$p"; done < /tmp/motolii-h1a-current-a.C1B7C4/paths.txt | awk '{s+=$1} END {print s}'
7155

$ find docs -type f -name '*.md' -print0 | sort -z | xargs -0 shasum -a 256 | shasum -a 256
293afcd8181834f193ff02c72e07ac189da65db5b0638d6c3cbfca6862539368  -
```

manifest hash・35件・7,155行・snapshot hashのいずれも発注書の期待値と一致した（作業終了直前の再実行結果は§7参照。作業中にsnapshotへ変更を加えていないため同一値）。

historical source blob 2件（`docs/design-memo.md` 4b8e1e6c、`docs/discussion-log-2026-07-06.md` ac3cda40）は`git cat-file -p <sha> | wc -l`でそれぞれ150行・91行、合計241行を確認し、全行を`nl -ba`で通し番号付きで本人が読了した。

## 3. 読了方法と非目標

- 35 docsはすべて`Read`ツールで冒頭からEOFまで（470行超のファイルは複数ページに分けて）本人が全文読了した。grep・見出し・索引だけで読了を代替していない。
- historical source blob 2件（design-memo.md 150行、discussion-log-2026-07-06.md 91行）も全文読了した。
- 非目標: 歴史案の採否・現行処置の最終分類、batch B〜Eのdocs、現行仕様/decision/index/codeの変更、実装・schema・API・plugin/配布/統治制度の変更、commit/push/PR。子エージェント・下請け・再委任は行っていない。

## 4. 35 path coverage table（manifest順）

| # | path | wc -l実測 | 全文読了 | 対応topic ID | 現行証拠path:lineと1文要旨 |
|---|---|---|---|---|---|
| 1 | docs/README.md | 148 | yes | T01, T07, T09, T10, T18 | README.md:7「design-memo.md(2026-07-05)とdiscussion-log-2026-07-06.mdは現決定と矛盾する旧仕様(Tauri+WebView採用、OpenCut Reactコード流用等)を含むため削除した」— 歴史2文書への直接的な削除・対置言及。README.md:11-13「MV制作ツール」「Rust+wgpu…React/WebView chrome+native Stage/Timeline」で技術スタック要約 |
| 2 | docs/ae-pain-points.md | 138 | yes | T01 | ae-pain-points.md:11-15 AE/Cavalry/Alight Motionの痛点をタグ体系化した拡張版。T01の主題を精緻化 |
| 3 | docs/backlog.md | 188 | yes | T02 | backlog.md:109-112 ANA-1〜4「色解析プラグイン→DataTrack生成」「オプティカルフロー/トラッキング」を最終フェーズ項目として台帳化 |
| 4 | docs/concept.md | 227 | yes | T01, T02, T03, T04, T08, T12, T13, T16, T17, T18, T19 | concept.md:68-72 AE/Cavalry/AM限界の再確認。concept.md:77 解析駆動の最終フェーズ後回し。concept.md:78-84 2.5D/3D統一シーン。concept.md:169 ノードグラフ非表示・内部依存グラフ+ダーティフラグ維持。concept.md:180-186 dylib/WASM隔離方式。concept.md:191 FrameDesc定義。concept.md:206 React chrome+native wgpu責任境界 |
| 5 | docs/decision-index.md | 85 | yes | T05, T07, T09, T11, T20 | decision-index.md:35「UI基盤 egui React WebView WebGPU PixiJS Konva Three.js…」比較中の索引行。decision-index.md:37「Three/Konvaは製品runtime候補から外した先例とする」。decision-index.md:38 GPUI/Qt Quick/Skia/Slint/Iced比較 |
| 6 | docs/dev-experience.md | 49 | yes | — | H1a対応なし（全文読了）。ホットリロード設計ノートで歴史2文書のtopicと重ならない |
| 7 | docs/extensible-core-model.md | 466 | yes | T15 | extensible-core-model.md:13「Linux/UNIX思想から借りるのは…小さな要素、明示した入出力、合成可能性」— minimal core + 拡張可能境界の一部と一致するが並列AIエージェント開発への直接言及はない（一部） |
| 8 | docs/generative-user-boundary.md | 241 | yes | T02, T04 | generative-user-boundary.md:96-109 p5.js型表現(`draw()`ループ・前frame画素蓄積等)をMaterialize/Live/Feedback/Simulationへ翻訳する表 — 歴史のp5.js/opencv的解析→生成路線を現行語彙へ再構成 |
| 9 | docs/implementation-ledger.md | 189 | yes | — | H1a対応なし（全文読了）。M0〜M5進行台帳で個別トピックの意味論を持たない |
| 10 | docs/interaction-simplicity-model.md | 334 | yes | — | H1a対応なし（全文読了）。操作単純化の横断仕様で歴史2文書のtopicと重ならない |
| 11 | docs/memory-model.md | 95 | yes | T08 | memory-model.md:17「AEはRAM=作業セット+プレビューキャッシュの一体運用…うちはVRAM=作業セット(定数・小)、RAM/ディスク=キャッシュ」— AE RAMプレビュー批判の精緻化 |
| 12 | docs/mocks/README.md | 470 | yes | T07 | mocks/README.md:316「FigmaのRGBA 0–1とdocument color profileを読み」— Figma語自体は現れるが色token交換形式としての言及であり、歴史のFigma建築類比とは異なる（単なる語一致） |
| 13 | docs/performance-model.md | 183 | yes | T06, T09 | performance-model.md:96-113 プレビュー/書き出し同一関数の品質モード表。native wgpu前提で帯域根拠を再構成、browser WebGPU比較は無い |
| 14 | docs/pitfalls-and-roadmap.md | 574 | yes | T05, T07, T09, T10, T11, T14, T15, T20 | pitfalls-and-roadmap.md:18-33 A-1節が旧WebView/Tauri検討経路(a)〜(d)を記録として保持し、(d)「GPUI等でUIごとフルネイティブ化…GPUIは採用実績・ドキュメントが薄い」と明記して不採用。pitfalls-and-roadmap.md:110 B-6でRust `opencv`クレートの重さを指摘しwgpu compute自前実装を優先。pitfalls-and-roadmap.md:156-160 D-3でOpenCut React流用不可を明記。pitfalls-and-roadmap.md:359 G-2「AviUtl2は思想の参考に留め、依存しない(2026-07-06検討の判断を維持)」 |
| 15 | docs/plugin-authoring.md | 204 | yes | T02, T16 | plugin-authoring.md:168-172「コンセプトの本線は「色解析・単純トラッキング → DataTrack → パラメータ駆動」。YOLO級は必須ではない」。plugin-authoring.md:33-45 Filter/Composite/LayerSource/ParamDriver種別とWASM隔離方針 |
| 16 | docs/plugin-resources.md | 133 | yes | T04 | plugin-resources.md §6(90-134行台)「時間参照: lookbehind / フィードバック」— 解析結果の時間窓・キャッシュ設計を扱うが色解析自体には触れず一部のみ対応 |
| 17 | docs/plugin-ui-model.md | 97 | yes | — | H1a対応なし（全文読了）。プラグインUI宣言語彙 vs 自由描画の比較文書で歴史2文書のtopicと重ならない |
| 18 | docs/references.md | 75 | yes | T05, T06, T11 | references.md:11「OpenCut…タイムラインUIの操作仕様の参考のみ。コード流用は不可」。references.md:12 ffmpeg-sidecar採用。references.md:22「GPUI…egui比較時の候補だったが不採用」 |
| 19 | docs/simulation-model.md | 247 | yes | T02 | simulation-model.md:155「映像由来コライダー(実写のダンサーで粒が跳ねる)は、最終フェーズの解析(セグメンテーション→SDF)がこの同じ口に合流する。「解析→生成」というこのツールの長期的な強み」 |
| 20 | docs/specs/M0-spikes.md | 49 | yes | T07 | specs/M0-spikes.md:7-8「2026-07-08にWebView/Tauri案からSlintへ転換」の歴史的milestone記録。specs/M0-spikes.md:25 S1タスクがSlint検証(旧WebViewブリッジ2方式は廃止)に改訂されたことを記録 |
| 21 | docs/specs/M1-vertical-slice.md | 206 | yes | T06, T17, T20 | specs/M1-vertical-slice.md:14 T2「motolii-media…デコードは生YUV420pで受ける」ffmpegサイドカー方式。specs/M1-vertical-slice.md:84「pub struct FrameDesc { width, height, stride, format, color_space, premultiplied }」。specs/M1-vertical-slice.md:66-77 Cargo workspace crates一覧 |
| 22 | docs/specs/M2-document-model.md | 342 | yes | — | H1a対応なし（全文読了）。Documentスキーマ・ジャーナル等の詳細仕様で歴史2文書のtopicと直接重ならない |
| 23 | docs/specs/M3-ui-integration.md | 248 | yes | T07, T11 | specs/M3-ui-integration.md:9「D-3(OpenCut流用の期待値管理)」。specs/M3-ui-integration.md:36「OpenCut(MIT)はコード流用不可となった(React前提のため)。**操作仕様・レイアウトの参考のみ**」 |
| 24 | docs/specs/M4-cache-and-analysis.md | 79 | yes | T02, T04, T05 | specs/M4-cache-and-analysis.md:21「解析(色解析・オプティカルフロー・トラッキング)は最終フェーズへ移動した」。specs/M4-cache-and-analysis.md:43「K5…最終フェーズへ移動(2026-07-09決定): 解析プラグイン(色解析→必要ならオプティカルフロー)」。specs/M4-cache-and-analysis.md:79「K5の実装手段(wgpu compute自前 / OpenCV / ONNXモデル)」 |
| 25 | docs/specs/M5-3d-and-post.md | 195 | yes | T03, T04, T19 | specs/M5-3d-and-post.md:11「空間は1つだけ(2026-07-14ユーザー決定)。2D画像、動画、テキスト、図形、glTF、点群を含む全オブジェクトが常に同じ正準XYZ世界」。specs/M5-3d-and-post.md:27「OBJはglTF変換パスで受ける(内部はglTFのみ)」。specs/M5-3d-and-post.md:26「ライティングはunlit(必要になったら固定1灯)」 |
| 26 | docs/specs/README.md | 49 | yes | — | H1a対応なし（全文読了）。仕様書プロセス運用規約で歴史2文書のtopicと重ならない |
| 27 | docs/text-model.md | 87 | yes | — | H1a対応なし（全文読了）。テキストスタイルスパン/アニメーター設計ドラフトで歴史2文書のtopicと重ならない |
| 28 | docs/ui-concept.md | 109 | yes | — | H1a対応なし（全文読了）。UIコンセプト五本柱で歴史2文書のtopicと重ならない |
| 29 | docs/ui-interaction-language.md | 383 | yes | — | H1a対応なし（全文読了）。任天堂/CAPCOM等ゲームUI参照(56行台)は状況説明の力学のみでT01〜T20のいずれにも該当しない |
| 30 | docs/ui-reference-map.md | 123 | yes | — | H1a対応なし（全文読了）。M3 UI参照地図の参照順位表で歴史2文書のtopicと重ならない |
| 31 | docs/ui-runtime-architecture.md | 211 | yes | T07, T18 | ui-runtime-architecture.md:7-9「Motoliiの製品UIは、Reactとnativeのどちらか一方へ全面統一しない。ReactはDOMが強い領域、native Rust/wgpuは高頻度GPU workspaceを所有する」— 歴史のWebView=UI／native wgpu=コアという分担思想を継承しつつReact＋opaque WebView islandsへ具体化した現行決定 |
| 32 | docs/ui-score-model.md | 141 | yes | — | H1a対応なし（全文読了）。時間面UI構成モデルで歴史2文書のtopicと重ならない |
| 33 | docs/ui-visual-language.md | 256 | yes | — | H1a対応なし（全文読了）。視覚言語基準でAbleton/Apple/Rerun等を参照するが歴史2文書のtopicとは別軸 |
| 34 | docs/vism-kit-model.md | 289 | yes | — | H1a対応なし（全文読了）。Vism/Kit責任分離モデルで歴史2文書のtopicと重ならない |
| 35 | docs/vism-package-concept.md | 245 | yes | — | H1a対応なし（全文読了）。Vism配布単位コンセプトで歴史2文書のtopicと重ならない |

行数合計: 148+138+188+227+85+49+466+241+189+334+95+470+183+574+204+133+97+75+247+49+206+342+248+79+195+49+87+109+383+123+211+141+256+289+245 = **7,155**（manifest期待値と一致）。

## 5. T01〜T20 topic別証拠表

各topicにつき、batch Aで見つかった全path:line、歴史側の意味との異同（同じ意味/一部/反対/単なる語一致 — 最終分類ではない）、証拠なしの場合は35 path全件読了に基づく`A内なし`を記す。

| topic | batch Aで見つかった全path:line | 歴史側との異同 |
|---|---|---|
| T01 制作動機/MV/AE・Cavalry・Alight Motionの痛点 | README.md:11-13、ae-pain-points.md:11-15,113-122、concept.md:68-72 | 同じ意味。AE重い・Cavalryベクター偏重・AM軽いが解析/3D欠如という歴史の痛点分析を、現行はae-pain-points.mdで体系的タグ分類へ精緻化し、concept.mdに再確認として保持している |
| T02 Tracery型の色tracking/optical flow→generative overlay | concept.md:77、backlog.md:109-112(ANA-1〜4)、plugin-authoring.md:168-172、M4-cache-and-analysis.md:21,43、generative-user-boundary.md:96-109、simulation-model.md:155 | 同じ意味だが優先度が縮小。「解析→生成」という長期的強みという評価は維持しつつ、2026-07-09決定で最終フェーズ(コアM1〜M5完成後)へ明示的に後回しにした点が歴史(初期スコープに含まれていた)との差分 |
| T03 3D mesh・動画・vector・pixelのpipeline責任 | concept.md:78-84、M5-3d-and-post.md:9-30 | 同じ意味。最終的にピクセル合成に帰着する考え方は維持し、2.5D統一シーン(単一世界・単一camera・拡張可能遮蔽ポリシー)へ具体化 |
| T04 解析/生成合成/post、post延期、OBJ/glTF | concept.md:173、M4-cache-and-analysis.md:21,43,79、M5-3d-and-post.md:27、plugin-resources.md §6 | 同じ意味。OBJ→glTF変換パスの採用、後処理(ポストプロセス)の位置づけ、解析の後回しはいずれも歴史の方向性と一致 |
| T05 paper.js/p5.js/PixiJS/Konva/three.js/wgpu/OpenCV等の候補と処置 | decision-index.md:35,37、concept.md:191、M5-3d-and-post.md:174、M4-cache-and-analysis.md:79、pitfalls-and-roadmap.md:110、generative-user-boundary.md:96-109 | 一部。wgpu/OpenCVは同じ意味で維持(wgpu採用・OpenCVは必要になるまで導入しない)。PixiJS/Konva/three.jsはdecision-index.md:35で比較対象として言及され、同37行で「Three/Konvaは製品runtime候補から外した先例とする」と明示的に処分(反対=不採用)。concept.md:191とM5-3d-and-post.md:174のthree.js言及は設計時の先例引用(色空間API刷新の教訓、glTFインポートの反面教師)であり候補評価ではない(単なる語一致)。paper.js/p5.jsはgenerative-user-boundary.md §5でp5.js型入力の翻訳表として現れ、ブラウザ内レンダリング候補としてではなく表現移植対象として扱われる(一部) |
| T06 VideoFrame/WebCodecs/ffmpeg、preview/export分離 | performance-model.md:96-113、references.md:12、M1-vertical-slice.md:14、M3-ui-integration.md(ヘッドレスdevice書き出し分離) | 同じ意味だがWebCodecs/VideoFrameはA内で言及なし。ffmpegサイドカー方式とpreview/export同一関数の分離思想は歴史と一致。ブラウザAPI(WebCodecs)は非ネイティブ構成の名残であり、現行の完全ネイティブ化により不要化(反対/進化) |
| T07 Electron/Tauri/WebView/native wgpu/Figma・VS Code analogy | pitfalls-and-roadmap.md:18-33、M0-spikes.md:7-8,25、M3-ui-integration.md:9,36、ui-runtime-architecture.md:7-9、README.md:7,11-13、README.md:75、mocks/README.md:316、decision-index.md:35,38 | 一部/反対。Tauri+WebView構成は2026-07-08にSlintへ、2026-07-18にeguiへ転換し、現在はReact chrome + native wgpu Stage/Timelineという新構成(1 top-level Surface + opaque child WebView islands)に到達(反対=歴史のTauri単独案は不採用だが、Web UI層+nativeコアという分担思想自体は継承=一部同じ意味)。VS Code/Figma類比は現行では明示引用されず(README.md:75とmocks/README.md:316のFigma言及はUI失敗調査対象/色token交換形式としてであり建築類比ではない=単なる語一致) |
| T08 node依存graph、dirty flag、選択cache、AE RAM preview批判 | concept.md:169、memory-model.md:17,19、pitfalls-and-roadmap.md:98-108(B-5)、pitfalls-and-roadmap.md:343-357(G-2) | 同じ意味。「ノードグラフをユーザーに見せない、内部は依存グラフ+ダーティフラグ」という歴史の設計判断はconcept.md:169でそのまま維持されている。AE RAMプレビュー批判はmemory-model.mdでVRAM/RAM/ディスク階層設計として発展的に継承 |
| T09 WebGPU/browser差、native wgpu risk | concept.md:206、decision-index.md:35、pitfalls-and-roadmap.md:46-54(A-3)、performance-model.md | 一部/反対。native wgpu(wgpu crateのAPI変動リスク)への言及はpitfalls-and-roadmap.md A-3で維持。ブラウザ間WebGPU実装差(Chrome/Safari/Firefox)の懸念は、アプリが完全ネイティブ化されたことで対象外化した(反対/進化=懸念そのものが不要になった) |
| T10 初期実装順/MVP/未解決/旧repo名 | README.md:7、pitfalls-and-roadmap.md:1-8,415-434(Part2冒頭)、M1-vertical-slice.md(出口デモ) | 同じ意味。M0→M1→凍結ゲート→M2〜M5という実装順は歴史のロードマップ骨子を継承し詳細化。旧repo名「p5.opencuts」への言及はbatch A 35 pathの全文中に見当たらない(A内なし) |
| T11 OpenCut UI/Rust core/GPUI/React reuse判断 | pitfalls-and-roadmap.md:18-33(A-1(d))、pitfalls-and-roadmap.md:156-160(D-3)、references.md:11,22、M3-ui-integration.md:9,36、decision-index.md:38 | 反対。歴史はOpenCut ReactタイムラインUIのコード流用に価値ありと判断していたが、現行はegui/React双方の経緯を経て「OpenCutはコード流用不可、操作仕様・レイアウトの参考のみ」(references.md:11、M3-ui-integration.md:36)へ反転。GPUIも「採用実績・ドキュメントが薄い」として不採用(pitfalls-and-roadmap.md:31、references.md:22) |
| T12 Alight Motion代替限界 | concept.md:70 | 同じ意味。「Alight Motionは軽いが、映像解析→ジェネレーティブ生成(Traceryライク)と3Dメッシュ合成を持たない」は歴史のdiscussion-log §2の趣旨とほぼ同一表現で維持 |
| T13 内部node graphと簡易timeline UIの分離 | concept.md:169 | 同じ意味。T08と同一証拠。ノードベース内部設計とタイムライン中心の簡易UIの分離は歴史のdiscussion-log §3の趣旨と一致し維持されている |
| T14 AviUtl2改造route、SDK/cache/beta/API risk、非依存理由 | pitfalls-and-roadmap.md:359(G-2) | 同じ意味。「AviUtl2は思想の参考に留め、依存しない(2026-07-06検討の判断を維持)」と明記され、歴史のdiscussion-log §4の結論(依存はリスクが高い)がそのまま維持されていることを確認できる一文。詳細な調査内容(SDK/cache/beta API変動等)はbatch A対象35 pathには含まれず(reviews/2026-07-17-aviutl2-comment-voices.md等は別batch)、この1行のみがA内証拠 |
| T15 minimal core/plugin-first/AI agent並列/OS native | concept.md:187-190、extensible-core-model.md:13、pitfalls-and-roadmap.md(D-1/G-1並列エージェント運用) | 同じ意味。「最小コア+全機能プラグインベース」はconcept.mdで維持され、AIエージェント並列開発は仕様書駆動開発として体系化(specs/README.mdはこのbatchの対象外だがpitfalls-and-roadmap.mdのPart2に同じ運用が明記)。OSネイティブ方針はpitfalls-and-roadmap.md E章で「v1は開発主機のOS1つに固定、Windows将来対応確定」として具体化 |
| T16 native dylibとWASM隔離、pixel/GPU境界 | concept.md:180-186、plugin-authoring.md:33-45 | 同じ意味。「生のピクセル/GPUバッファに触るか否か」で切り分けるネイティブ第一級市民/WASMサンドボックスの二分は歴史のdiscussion-log §5-1と概念的に同一のまま現行に継承 |
| T17 Frame descriptor、stride/pixel format/color/premultiplied、VRAM texture共有 | concept.md:191、M1-vertical-slice.md:84-89、plugin-authoring.md §3 | 同じ意味。`{width, height, stride, pixel format, color space, premultiplied alpha}`というフレーム記述子の構成要素は歴史のdiscussion-log §5-2とほぼ同一のまま`FrameDesc`として実装済み |
| T18 Web UI表層とnative core bridge | concept.md:206、ui-runtime-architecture.md全体(7-211行) | 一部同じ意味。「UI表層はWeb技術寄せ」「ネイティブコアとUIの間はテクスチャ/画像をIPC経由で渡す」という歴史の構想は、現行はReact chrome + native wgpu Stage/Timelineという形へ具体化され、橋渡しはtyped domain intent/read-only snapshot投影(IPC texture転送ではなくtyped境界)で行う点が進化している |
| T19 2D/3D同一scene、2.5D、single camera、DoF/light/非目標 | concept.md:78-84、M5-3d-and-post.md:9-30 | 同じ意味。単一worldに全objectを配置し、被写界深度はZ距離ベースのポストブラーで代用、ライティングはunlit(初期)という歴史discussion-log §6の記述と現行M5-3d-and-post.mdの記述はほぼ一致 |
| T20 plugin候補、workspace分割、project schema、target OS、UI基盤未決 | pitfalls-and-roadmap.md:460-469(凍結ゲート項目5)、M1-vertical-slice.md:66-77(crates一覧)、pitfalls-and-roadmap.md E章、decision-index.md:35 | 同じ意味。Cargo workspace分割は歴史の未決事項(§未決定事項)から現行では10クレート構成へ確定。target OS範囲は歴史の未決からmacOS開発機+Windows将来対応確定へ解決。UI基盤(native vs Web技術)は歴史でも未決、現行もdecision-index.md:35で「比較中」のまま両者とも未決着という点で一致 |

## 6. 次batchへの申し送り

- 本batch Aはmanifest固定35 path（root直下md + specs/*.md + mocks/README.md）のみを対象とした。
- **batch B〜Eは未監査**。現行docs全体は185件ではなく、batch A済み35件を除いた**残り150件**（docs/reviews/配下のレビュー文書・spikes等）が対象である。T14(AviUtl2)やT07(Figma/VS Code類比)、T05(先例調査の詳細)等、batch Aで「A内なし」または部分証拠にとどまった主題は、docs/reviews/配下に詳細な調査文書（例: reviews/2026-07-17-aviutl2-comment-voices.md、reviews/2026-07-16-ui-update-forensics.md等、いずれもbatch A対象外）が存在する可能性が高く、batch B〜Eでの追加照合が必要である。
- 本batchの結果だけでT01〜T20の「完全回収/未回収」は判定できない。最終処置分類は5 batch統合後に行う。

## 7. 機械検査結果

以下は全て作業終了直前に実行し、出力をそのまま記録した（未来形・検収者任せの記述はしていない）。

### snapshot側の再確認

```
$ cd /tmp/motolii-docs-history1-current.BgHOsk && find docs -type f -name '*.md' -print0 | sort -z | xargs -0 shasum -a 256 | shasum -a 256
293afcd8181834f193ff02c72e07ac189da65db5b0638d6c3cbfca6862539368  -
```

manifest hashとの一致を確認済み（`293afcd8181834f193ff02c72e07ac189da65db5b0638d6c3cbfca6862539368`）。

### worktree側の必須コマンド

```
$ cd /tmp/motolii-history-recovery1.Zqdgi2 && git diff --check
（出力なし = 空白文字エラーなし）

$ scripts/check-docs.sh
OK: docs整合チェック全項目通過

$ git status --short
?? docs/reviews/evidence/historical-recovery-h1a/current-corpus-a.md

$ git diff --name-only
（出力なし。成果物はuntrackedのみで、追跡対象ファイルへの変更はゼロ）
```

`git status --short`の実測どおり、成果物は`?? docs/reviews/evidence/historical-recovery-h1a/current-corpus-a.md`のuntracked状態である。cleanではない。`scripts/check-docs.sh`は`OK: docs整合チェック全項目通過`でexit 0を返した。

## 合格条件チェック（自己申告・機械検査に基づく）

- 変更ファイルは`docs/reviews/evidence/historical-recovery-h1a/current-corpus-a.md`の新規追加1件のみ
- 35/35件をEOFまで全文読了、行数合計7,155/7,155と一致
- coverage table 35行はmanifest順・重複欠落ゼロ
- T01〜T20の全topicに証拠または`A内なし`を記載
- current citationは全てsnapshot内の実在line（本文中で`nl -ba`相当の行番号確認済み）
- 最終意味裁定（完全回収/未回収等）は行っていない
- `scripts/check-docs.sh`はOK・exit 0
