# コア/拡張(vism)の分類 — 採用予定1,025行の振り分け

日付: 2026-08-22 / 状態: **決定**(利用者裁定「あくまで最小コアを目指す。拡張というスロットができたのは大きい。その分類を探そう」への応答) / 前提: 裁定193(パネル級の機能は「無い」ではなく vism=拡張)・裁定153(vism第1号=effect縫い目5切片)・裁定175(拡張verdictの新設、ペイント族/Expression族の先例)・裁定177(意図束)・[意図束の動線図](2026-08-22-intent-flow-map.md)

## 0. worktree差分についての注記

着手時、本 worktree の `next/DECISIONS.md`・`next/reference/normal-map.tsv` は main から54コミット遅れており(裁定193自体・裁定192「結線待ち」verdict新設を含む)、発注書が前提とする「採用予定1,025行」と実測(1,079行)が一致しなかった。ローカル変更ゼロ・main が祖先の純粋な遅れだったため `git merge --ff-only main` で追随(02d5cec6)。以後この状態で作業。

## 1. コアの定義(この製品が背負うと決めた範囲)

**コア = 「これが無いと動画が作れない」で判定する。** 普通の動画編集ソフトが成立するために必須な操作の束のみをコアに残す:

> 取り込み・並べる・切る(trim/split)・変形・キーフレーム・マスク・ブレンド・再生・書き出し・選択・履歴・パネル運用・設定

これは「機能が少ない」ことを目指す最小主義ではない — 普通地図(裁定154/158)が回収した1,025行のほぼ全部(982行、96%)は依然コアに残る。コアが指すのは「動画編集という行為の骨格」であり、それを超えて**特定のワークフロー(放送・グレーディング・音楽制作・VFX合成)を専門にする一式**はコアの外に置く。

**拡張(vism)= 戸を残す、閉じない**(裁定175/193)。コアでないと判定した機能は「無い」(不採用)ではなく、後から継ぎ目(プラグイン機構)で載せられる状態として台帳に残す。この発注では **verdict を不採用へ倒さない** — 純化はコアの外形を絞ることであって、機能を消すことではない(「拡張の哲学=自由の保護」2026-08-20 と同じ結論)。

### 判定式(2値・迷ったらコア)

1. その機能が無いと「動画編集ソフト」として成立しないか? → Yes ならコア
2. 成立はするが、特定の専門ワークフロー一式(色補正・音声・ノード合成・個別エフェクト・解析/トラッキング・特殊フォーマット・AI生成・マルチカム)を丸ごと持ち込む物か? → Yes なら拡張
3. どちらとも言えない → **触らない**(採用予定のまま。保留へ落とさない・拡張へ逃がさない)

## 2. 振り分け結果

- **拡張へ移動: 43行**(裁定193 該当34行 + 裁定175 ペイント族の拾い漏れ6行 + 裁定193/AI生成系 homeless再審3行 — 内訳は下記§3)
- **コア(採用予定)据え置き: 982行**

### 最終 verdict 分布(全1,551行)

| verdict | 変更前 | 変更後 |
|---|---:|---:|
| 採用予定 | 1,025 | **982** |
| 拡張 | 16 | **59** |
| 採用済 | 195 | 195(不変) |
| 結線待ち | 35 | 35(不変) |
| 不採用 | 266 | 266(不変) |
| 保留 | 14 | 14(不変) |

`cd next && ./check.sh` は EXIT=0(wraps/owns marker・Lottie地図・意図束3検査すべて通過)。verdict変更のみで bundle 列は一切触っていないため、意図束の size 一致検査(束45/割付1285行)も無傷。

## 3. 振り分けの中身(8束 + 1点)

各行の変更は verdict 列と理由列のみ(id・category・canonical・意味・freq・bundle 等は不変)。理由列は既存16行と同じ語彙(「vism圏: なぜコアでないか」+「継ぎ目=どの機構で載るか」)で統一。

### A. 色補正スイート(Lumetri級)— 8行
`389` Primaries/Log Color Wheels・`427/428` Video Scopes・`1010/1011` Lumetri Color・`1012/1013` Lumetri Scopes・`1030` Reference Monitor。
理由: 単発の色調整ではなく専用グレーディング/スコープ機構一式が要る専門機能。継ぎ目=将来のcolor providerプラグイン。

### B. オーディオスイート(Fairlight級)— 4行
`986` Audio Clip Mixer・`988` Audio Track Mixer・`1000` Essential Sound・`831` Edit in Adobe Audition(専用DAWへのハンドオフ)。
理由: クリップ音量・フェード止まりでなく専用ミキサー/DAW機構が要る専門機能。

### C. ノードコンポジット/3D(他製品の内部機構に依存)— 2行
`882/883` MAXON CINEMA 4D File…(Cinema 4Dとのライブリンク)。
理由: 単一comp前提のMotoliiコアには無い3D合成パイプライン。Fusionノード(既存29行、既に不採用)と同族だが、こちらは「他製品への依存」で不採用条件(裁定193 (b))にも触れるため、より積極的に拡張側とした。

### D. 個別エフェクト群(Glow以降=vism第1号の前例どおり)— 7行
`1388` Chroma Key・`905` Optical Flow・`908` Pixel Motion(同じ高品質リタイム系アルゴリズム族)・`993` Content-Aware Fill(AIインペインティング)・`747` Activate Roto Brush tool・`415` Show Refine Edge X-ray・`1379` Activate Refine Edge tool(Roto Brush系の縁取り調整、747と同族)。
理由: 裁定153のeffect provider経路(compositor内layer単位オフスクリーンパス)で後から載せる単発の専門エフェクト/アルゴリズム。コアはeffect適用UIの器のみ持つ(裁定70: 個別effect型はDocumentが持たない)。

### E. ペイント族の拾い漏れ(裁定175の既存9行と同族)— 6行
`418` Swap paint background/foreground colors・`711` Set opacity for paint tool・`1381` Activate specific Clone Stamp preset・`1390` Duplicate Clone Stamp preset・`1399` Momentarily activate Eraser with Last Stroke Only・`1528` Paint(workspace)。
理由: 裁定175で「ペイント族9行→拡張」が決まった際に拾われていなかった同族行(Brush/Clone Stamp/Eraser/Paintツールの周辺操作)。既存の9行(989/1023/1374/1427-1430/1468/1519/1523)と地続きなので同じ理由文で統一。

### F. 特殊フォーマット/コーデック対応 — 10行
`594` Adobe Premiere Pro Project…・`595` Batch List From EDL・`599` Import AAF, EDL, XML・`607` Media from XML・`612` Pre-conformed EDL・`613` Pro Import After Effects…(放送/ポスプロ間の相互運用フォーマット一式)・`1168` Decode Options(RAW/H.264等のデコード方式設定)・`691` Photoshop Layers…・`807/808` Adobe Photoshop File…(PSD相互運用)。
理由: 普通の動画編集ソフトに不要な、他製品・他業界標準との相互運用フォーマット。継ぎ目=import/exportプラグイン。

### G. AI生成系(homeless再審キューの解消)— 3行
`1382` AI design・`1383` AI writer・`1424` Script to Video。
理由: 素材のゼロからの生成(id2 Image to videoと同型 — 解析→調整路線ではなく新規生成)。この3行は[意図束の動線図](2026-08-22-intent-flow-map.md) §2 で bundle=`HOMELESS`(verdict再審キュー)と既に名指しされていた行で、拡張verdictはその再審への決着でもある(「継ぎ目を塞ぐ方向のverdictを機械的に付けない」裁定193の要求とも合致)。

### H. マルチカム — 3行
`239` Multicam・`240` Multicam Cut・`241` Multicam Switch。
理由: 複数カメラ素材の同期・切替は単一timeline前提のコアを超える専門ワークフロー。

## 4. 意図して触らなかった物(コア側に残した根拠)

- **B16「解析→自動キーフレーム」束の全10行**(Auto reframe/Track Camera/Warp Stabilizer VFX/Tracker/Track Motion/Track this Property/Motion Tracking/Detect Scene Cuts/Scene Cut Detector/Track Mask)は**移動しなかった**。裁定193の例示は「解析/トラッキング」も拡張候補として挙げるが、この10行は既に intent-bundles.tsv で「analyze_and_generate(target: reframe|track|stabilize|scene_cut)」という単一の Bake/Analysis provider機構に統合済み(L8基盤)で、id 1 Auto reframe の理由文が示す「解析→生成」路線(KNOWN.mdと整合・素材新規生成ではない)がこの束全体の存立根拠になっている。既に一つの機構としてbundleされた10行のうち一部だけを拡張へ倒すと、その機構の一枚岩性を壊す。**Track Camera(146)も同型**(3Dカメラの解析トラッキング)なので同じ理由で据え置いた。
- **Auto Color(354/355)・AI color correction(346)**は解析ベースの自動調整(生成ではない)で、id 1 と同じ「解析→調整」路線に乗るため据え置き。
- **Auto Caption/Auto captions(153/154)・Text(Speech-to-Text, 1035)**は現代の「普通」の一部(CapCut/Premiere/Resolveいずれも搭載)と判断し、コアの字幕束(B04)側に据え置き。
- **Creative Cloud Libraries(1008/1009)・Browse/Reveal in Bridge(814/933)・Internet Accounts(1186)・Sync Settings(1211/1212)・Collaboration設定(1162)**は他社サービス依存だが、裁定193が列挙した8分類(色/音声/ノード/個別エフェクト/解析/フォーマット/AI生成/マルチカム)のどれにも綺麗に当てはまらないため触らなかった(「拡張へ逃がしすぎない」の実践)。
- **Output Module…(907)・Reconform from Bins/Media Storage(919/920)・Essential Graphics(998/999)**も同様に境界事例だが、書き出し・素材管理というコア機能の延長と読める余地が残るため据え置いた。

## 5. 代表例10件

| id | canonical | 束 | verdict変更 |
|---|---|---|---|
| 1010 | Lumetri Color | A色補正 | 採用予定→拡張 |
| 427 | Video Scopes | A色補正 | 採用予定→拡張 |
| 986 | Audio Clip Mixer | Bオーディオ | 採用予定→拡張 |
| 882 | MAXON CINEMA 4D File… | Cノード/3D | 採用予定→拡張 |
| 1388 | Chroma Key | D個別エフェクト | 採用予定→拡張 |
| 747 | Activate Roto Brush tool | D個別エフェクト | 採用予定→拡張 |
| 1381 | Activate specific Clone Stamp preset | Eペイント拾い漏れ | 採用予定→拡張 |
| 599 | Import AAF, EDL, XML | F特殊フォーマット | 採用予定→拡張 |
| 1424 | Script to Video | G AI生成系 | 採用予定→拡張(homeless再審決着) |
| 239 | Multicam | Hマルチカム | 採用予定→拡張 |

## 6. 逸脱・留意点

- csv モジュールで一括置換を最初試みたところ、`QUOTE_NONE + escapechar` の組み合わせが既存データ中の生の `\`(id 225 `Option+\`)や `"` を巻き込んで無関係な5行(225/1144/1191/1310/1334)まで書き換えてしまった。検出後に `git checkout` で全面復元し、単純な `line.split('\t')` ベースの置換に書き直して再実行(csv dialect を持たないTSVにcsvモジュールの引用規則を持ち込んだのが原因)。最終差分は行順不変・43行の verdict+理由列のみで、他列の変化ゼロを機械確認済み。
- C束(Cinema 4D)の2行は「ノードコンポジット」というより「3D DCCとのライブリンク」に近く、発注書の8分類には字面上ぴったり一致しない。Fusionノード群との親和性(他製品依存・単一comp外)を根拠に拡張側へ寄せたが、コア側に残す判断もあり得た境界事例として明記する。
