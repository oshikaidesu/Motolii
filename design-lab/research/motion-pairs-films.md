# 意図 → 動き の対 調査(映像制作者ルート)

調査日 2026-10-03 / 読み取り専用 / ダウンロード無し。
**証拠の限界(先に)**: 動画は見られない。静止画も 1 枚も見ていない。したがって下表の全行は `text-described`(ページ本文・文字起こし・ドキュメントの記述を読んだだけ)。`still-seen` は 0 件。WebFetch は小型モデルによる要約なので、記述は「要約経由の本文」であり原文との照合はしていない。
**ルートの結果(先に)**: 指定ルート(Figma/Linear/Notion/Arc/Raycast/Framer/Apple の launch film の制作者 → その個人サイト → 実験)は、制作者の特定まで届いたのが Figma Config 2025 の Relay だけ。残りは generic な解説か空振り(§3)。対は実質、Apple WWDC 文字起こし・Material・Bostock・Bret Victor・Cavalry docs・FUI インタビューから取った。

## 1. 対の表
Directness = 「やりたいこと」と「画面で動くもの」の一致度(1 弱〜5 強)。

| # | 意図の動詞 | 画面の動き | 出所 / 作者 | Directness と理由 | 証拠 |
|---|---|---|---|---|---|
| 1 | カードを開く(tap) | タップしたセルがそのまま拡大・移動して全画面の編集 view になる(zoom transition) | https://developer.apple.com/videos/play/wwdc2024/10145/ Apple | 5: 触った物が行き先になる。出所と行き先が同一物 | text-described |
| 2 | 行/コンテナを開く | 開始コンテナの境界が終了コンテナの大きさ・形に変形し、中身が入れ替わる(container transform) | https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md Google Material | 4: 「つながり」を一つの変形で言う。ただし規則が先で意図は推定 | text-described |
| 3 | ドラッグして離す(投げる) | spring が指の離した瞬間の速度を初速に引き継いで目標へ進む。ease-in-out は急停止する | https://developer.apple.com/videos/play/wwdc2023/10158/ Apple | 4: 手の勢いがそのまま動きになる。目標は UI 側が決める | text-described |
| 4 | 移動先を途中で変える(再タップ) | 動作中の spring が現在速度を初速にして新目標へ向きを変える(retarget) | 同上 | 4: 意図の変更が速度の連続として見える | text-described |
| 5 | 軽く下へスワイプ(ホームへ) | アプリ窓が縮んで下へ滑り、springboard が現れる | https://developer.apple.com/videos/play/wwdc2018/803/ Apple(Designing Fluid Interfaces) | 5: 「戻る」の方向と大きさが窓の動きと一致 | text-described |
| 6 | 押す(Control Center モジュール) | モジュールが指に向かって、最終形の方向へ伸びる | 同上 | 5: 押す→その場から育つ。行き先を方向で予告 | text-described |
| 7 | はじく(FaceTime PIP を隅へ) | 速度から投影した位置に最も近い隅へ減速して止まる(position + velocity/decelerationRate) | 同上 | 5: 意図=「あの隅へ」を勢いから読む | text-described |
| 8 | 触れたまま動かす | 内容が指と 1 対 1 で、触った相対位置を保って付いてくる | 同上 | 5: 意図と動きが同一(定義上の直接操作) | text-described |
| 9 | 端を越えてスクロール | 内容が弾性で範囲内へ戻る(rubberbanding) | 同上 | 4: 「限界」を壁でなく連続した抵抗で言う | text-described |
| 10 | アイコンを押して開く | アイコンが引き伸ばされて窓になり、閉じる時は逆向きに伸びる | 同上 | 4: 出所と窓の連続性。motion blur 付きと記述 | text-described |
| 11 | タブを左へスワイプして削除 | タブが左へ滑って消える。ボタン削除でも同じ動きを見せてジェスチャを教える | 同上 | 4: 削除=横へ去る。動きが操作の教師 | text-described |
| 12 | 十分強く押す(懐中電灯) | 押圧で大きさが変わり、足りない tap では弾んで「もっと押せ」と示す | 同上 | 4: 閾値を言葉でなく動きで伝える | text-described |
| 13 | メニューを出す(Liquid Glass ボタン) | ボタンがオーバーレイへ morph し、出所との連続を保つ。大きくなると材質も厚く見える | https://developer.apple.com/videos/play/wwdc2025/219/ Apple(検索結果の要約のみ。本文は未取得) | 4: 押した物がメニューになる | text-described(二次要約) |
| 14 | 並べ替える(sort) | key で追跡された棒が y 軸上で入れ替わる。軸も同時に再スケール | https://bost.ocks.org/mike/constancy/ Mike Bostock | 5: 順位が変わる=棒が動く。同一性が key で保たれる | text-described |
| 15 | 絞り込む(top ten が変わる) | 条件に入った要素が fade in、外れた要素が fade out、残りは新位置へ補間 | 同上 | 4: 絞り込みの「入る/出る/残る」を 3 状態で見せる。ただし対象は 1 チャート内 | text-described |
| 16 | ある次元の範囲を brush | 他の全チャートのヒストグラムと表が、その範囲に該当する分だけに更新される(<30ms と記述) | https://square.github.io/crossfilter/ Square(Crossfilter) | 4: 「世界を絞る」に最も近い実例。ただし動きは補間でなく即時再計算で、動きによる表現はほぼ無い | text-described |
| 17 | 要素ごとに時間をずらす | 要素ごとに delay/duration を変えて順に動く | https://bost.ocks.org/mike/transition/ Bostock | 3: 順序が見える。意図(なぜ順か)は作者が決める | text-described |
| 18 | 新しい変化を始める(割り込み) | 実行中の transition が止まり新しい方が優先される | 同上 | 3: 応答性の規則。画面上の意味は薄い | text-described |
| 19 | 複製に波を付ける(Duplicator + Oscillator + Stagger) | 各複製が前の複製より少し遅れて動き、群れの動きが波になる。Stagger は ID に最小〜最大値を割り振り、Time Offset = Stagger 値 + 現フレーム | https://cavalry.studio/docs/nodes/behaviours/stagger/ Cavalry(docs 要約)。波の説明は検索結果の要約(Cavalry docs / チュートリアル) | 3: 「ID が値に変わる」と書かれる。画面の波は結果で、操作は数値の割り振り | text-described |
| 20 | 繰り返す(Loop キー) | 範囲選択した手順が、データの別の列に対して繰り返され、図形が積み上がる | https://worrydream.com/DrawingDynamicVisualizationsTalkAddendum/ Bret Victor | 4: 手順を繰り返すと絵が増える。回数が見える。動画未視聴で記述のみ | text-described |
| 21 | 吸着(Snap)の候補を切り替える(Tab) | 重なった吸着点で Tab を押すと対象が黄色くなり注釈が更新される | 同上 | 3: 候補の切替が見える。強調はするが動かない | text-described |
| 22 | 磁石を置く(sub-picture から export) | 最終出力には出ない目印の点が、外側の絵からの吸着先として見える | 同上 | 4: 「ここへつなぐ」を見える点にする | text-described |
| 23 | 駅を指す(From/To) | 路線図上の From/To マーカーをドラッグして駅に置く。ドロップダウンは使わない | https://worrydream.com/MagicInk/ Bret Victor | 5: 対象の指定=その場所を指す。ただし静的記述で動きの記述は無い | text-described |
| 24 | レイヤーの同一性を示す(Smart Animate) | 同名・同階層のレイヤーが frame 間で位置・大きさ・回転・不透明度・塗りを補間する。グループに別名を付けると「新規」として扱われ、動かなくなる | https://help.figma.com/hc/en-us/articles/360039818874-Smart-animate-layers-between-frames Figma | 3: 動くかどうかが「名前の一致」で決まる。意図でなく命名規則 | text-described |
| 25 | 画面間で物を投げる(FUI) | 画面からの 3D モデルが投影され、画面の間で物を「fling」する。Nick Fury は動かし swipe、Tony Stark は回して指で滑らせる | https://www.pushing-pixels.org/2012/06/01/the-craft-of-screen-graphics-and-movie-user-interfaces-conversation-with-jayse-hansen.html Jayse Hansen(Avengers) | 4: 「あっちへ渡す」が投げる動き。記述は引用 1 行程度で細部は不明 | text-described |
| 26 | 死体を調べる(Blade Runner 2049 検死) | 骨へ「機械的・光学的に」ズームする。リズムと動きを重視したと記述 | https://www.awn.com/vfxworld/communicating-abstract-user-interfaces-blade-runner-2049 Territory Studio(Andrew Popplestone 談) | 2: 見る行為の雰囲気。操作の意味より物語 | text-described |
| 27 | 録る(Figma Motion の auto keyframe) | playhead を進めながら変更すると keyframe が記録される。位置・大きさ・回転・不透明度は独立 | https://explainx.ai/blog/figma-config-2026-complete-recap-motion-code-shaders-ai-2026 | 3: 変えた値が点になる。時間軸と値の対応は強いが UI 説明のみ | text-described(二次記事) |
| 28 | 文字 1 つを動かす(Config 2025 film) | 1 つのグリフが動いて構図全体に波紋を起こす。表現的な動きは時間をかけ、機能的な動きは速い | https://www.figma.com/blog/how-we-shaped-the-visual-identity-for-config-2025/ Figma / Relay | 2: 1 つが全体に波及する発想はあるが操作の意味ではなく装飾 | text-described |
| 29 | キーボードで高頻度に切替 | 動きを付けない(1 日数百回で遅く感じるため)。Raycast を例に挙げる | https://emilkowal.ski/ui/you-dont-need-animations Emil Kowalski | 2: 反例。意図が連打なら動きは邪魔になるという主張 | text-described |
| 30 | 無関係な画面へ移る | 出る側が fade out、入る側が fade in。位置は固定 | Material(#2 と同じ URL) | 2: 関係が無いことを「何も動かさない」で言う。意図の表現は弱い | text-described |

## 2. 次に追う作り手
- Relay(コペンハーゲン)https://relay.design/ : Figma Config 2025 のオープニング film の制作者(Figma 本文で確認)。サイトは取得できたが本文なしで作品内容は未確認。次はここの case study ページを人間が見る価値あり。
- Figma motion team(Chad Colby, Ben Hill, Gilles Desmadrille): 同ブログに名前あり。個人サイトは未確認。
- Rauno Freiberg https://rauno.me/craft : 元 The Browser Company(Arc)、現 Vercel。dock・radial menu・グラフ slider など「入力の意味を動きにする」実験が 2021–2026 並ぶ(目次的要約のみ読了)。最優先。https://devouringdetails.com/ には「Interaction metaphors / Simulating physics / Motion choreography」の項目名あり。
- Emil Kowalski https://emilkowal.ski/ : Linear の design engineer(検索結果の要約)。いつ動かす/動かさないの基準を文章で書く。
- Bret Victor https://worrydream.com/ : 操作=見える変化の最も明確な一次資料。Drawing Dynamic Visualizations の動画(Vimeo 66085662)は動画なので未視聴。
- Mike Bostock https://bost.ocks.org/mike/ : 同一性・enter/update/exit の整理。
- Jayse Hansen https://jayse.io/ : FUI で「操作が意味」の記述が比較的多い(要確認)。
- GMUNK https://gmunk.com/ : Oblivion / Tron の interview 群があるが、操作の意味に踏み込む記述は読めていない。
- Cavalry https://cavalry.studio/docs/ : モーショングラフィックス editor 自身の用語(Stagger/Duplicator/Behaviours)の一次資料。

## 3. ルートが失敗した所
- Linear / Notion / Raycast / Framer / Arc の launch film の制作者: 検索では generic な「launch video の作り方」記事と changelog しか出ず、制作者は特定できなかった。Notion は無関係な結果(テンプレート)のみ。
- Apple の feature film / keynote 動画の UI: Territory/Buck/GMUNK との結び付きは出ず、Shot on iPhone 広告の話だけ。
- Figma Config 2026 / Figma Motion の launch film: 制作者の記載無し(二次記事 2 本とも)。
- Behance: 検索ページと個別プロジェクトは 403(David Gomez「UI Motion Gallery」、Maximilian Müsgens「After Effects Redesign Concept」)。「Motion Concept Interface」という題の作品は検索で見つからず、プロジェクト説明文は読めていない。
- Blade Runner 2049(HUDS+GUIS): 操作と動きの対応の詳細は無し(状態は initial / action / looping の 3 種と記述のみ)。
- ゲーム内の架空 UI: 今回は扱えていない(none)。
- Material 3 / Apple HIG の公式 Motion ページ: JS 描画で本文取得不可。Material は GitHub の docs で代替。Adobe の pick whip ページ、Keynote Magic Move、Cinema 4D Effector の docs は 403/404 で未確認。
- WebFetch は要約モデル経由。原文の語句はそのまま保証できない。

## 4. モーショングラフィックス editor 自身の操作との関係
- Wave: #19(Cavalry)。波は「ID ごとの時間ずらし」の結果。操作と画面の一致は中(3)。
- Repeat: #20(Victor の Loop キー)。回数が絵として積み上がる点で強め。#19 の Duplicator も同系。
- Random: none(今回、根拠のある記述に当たらず)。
- Ease: #3/#4 に「ease-in-out は急停止、spring は速度を引き継ぐ」。Bostock の「easing は時間を歪める」は #17 の周辺に記述のみ。ease を操作として可視化する例は none。
- Step: none(C4D Effector docs が 404、他に当たらず)。
- Range: #19 の Stagger の Minimum/Maximum と Graph(ID 位置 → 値)が最も近い。画面に動きで出るのは結果のみ。
- Stagger: #17, #19。
- Group: none。#24 は「グループに別名を付けると match されない」という副作用の記述のみで、集まる動きの実例は見つからなかった。
- Connect(線が伸びて繋がる): none。#22(吸着先の点)と #23(指す)が近いが、線が伸びる動きの記述ではない。
- Copy-one-aspect: none。
- Search/filter: #15, #16。#16 は世界全体が絞られる点で近いが、動きではなく即時再計算。

設計への推奨は書いていない(指示どおり)。
