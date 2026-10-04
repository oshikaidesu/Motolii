# 色の独立批評 (Colour knob: Grey / Palette A / Palette B / A + CVD sim) 2026-10-02

方法: 実アプリ(lab_book, dev.lab.labBook)の Addons タブ「Colour」を切り替え、7 部品を等倍で撮影し目視で判定した。コードは変更していない。ピクセル値は計算でなく目視(拡大)で、数値は color-ud.md の計算表を引いた。
読んだ物: research/color-ud.md、DESIGN.md、panel-checklist 11 / 20 / 26-29。
注意: 部品によっては knob に全く反応しない(下の B2 / B5 / I9)。反応する物は I2 / I4 / I6 / Inspector(Relations) / timeline_parts。

## 総評(先に結論)

- 「色が無くて読みにくい」への効きは、Inspector の値欄で最もはっきり出た。changed が「空青の値 + 空青の点線下線 + 角の三角」になり、Grey(白字・白下線)より 1 秒で拾える。
- ただし役割色が効いているのは Inspector 系の一部だけ。Browser(B2 タグ帯・B5 自分のコマ)と I9 Whip は Grey / A / B で画面が同一。Browser の「選択中のチップ」は灰の塗り + 1px 下線のまま。
- 推奨は Palette B。理由は末尾。A は黄色が強すぎ、mode のピンクが関係家族の Scatter と衝突し、On のスイッチが家族に関係なく赤茶になる。

## 1. 部品ごとの所見

### 1-1 Inspector I4-b Animate on / edit (Transform, Blur, Glow ...)

| 観点 | Grey | A | B | A+deut | 備考 |
|---|---|---|---|---|---|
| 変更の見つけやすさ(1) | 白字・白点線下線。未変更の白字と差が小さい | 空青の値 + 空青下線 + 角印。一目で拾える | 同系だがやや落ち着く(#56B4E9) | 周辺青に化ける(薄い青紫)が、未変更の白との差は保つ | A/B とも changed は効いている |
| キー状態(2) | 菱形が全部同じ青紫 | 静止行=ピンクの白抜き菱形、キー間=黄の「◇─」、現在キー=塗り菱形 | 静止行=薄青紫、キー間=黄 | 静止行=ベージュ、キー間=黄 | 形が先(白抜き / 線付き / 塗り)なので deut でも区別は保たれる |
| 色規律(3) | accent 1 つ | **ピンクの白抜き菱形が数十個**並ぶ。mode 色(ピンク)の面積が大きく、落ち着きが無くなる。Animate スイッチの On も赤茶 | 落ち着く | 落ち着く | ピンクの大量反復は規則(1 行 2 色まで)ではなく「量」で違反気味 |
| 文字コントラスト(5) | 十分 | #66CCFF の値は暗地で十分読める | #56B4E9 も十分 | 薄い青紫の値は少し沈むが読める | 問題なし |
| 密度(4) | 保つ | 保つ(ピンクが騒がしい) | 保つ | 保つ | |
| 変更行の「値」と「ラベル」 | — | 両方が空青。行ごとに 2 か所青いが色は 1 種 | 同左 | 同左 | 1 行 1 色の規律は守っている |

欠点: 静止行の菱形に mode 色を使うのは意味のずれ。「Animate が On」(mode) と「この行はまだキーが無い」(状態)は別の事柄なのに同じピンクで表している。

### 1-2 Inspector I2 Number(states: 変更 / 複数 / 固定 / 駆動 / 限界 / 数値でない)

| 観点 | Grey | A | B | A+deut |
|---|---|---|---|---|
| 変更(角印+点線下線) | 白 | 空青 | 空青 | 青紫 |
| 複数(値が「—」) | 変更と同じ見た目 | 同左 | 同左 | 同左 |
| 固定(斜線ハッチ、枠無し) | グレー | グレー(正しい) | グレー | グレー |
| 駆動(家族色の丸点 + 「Follow · Jewel 02」) | 橙 | 橙 | 橙 | **黄**(deut で橙が黄に) |
| 数値でない(エラー、1px の枠) | 赤 | 赤 #FF4B3A | **橙赤 #F0643A** | **黄褐色** |

- エラーの手掛かりは「枠の色」と、下の帯の文言「"abc" is not a number」だけ。記号(×・!)は無い(color-ud.md の「error は記号必須」に未達)。deut では枠が黄褐色になり、駆動の丸点(黄)と同系になる。
- **B ではエラー枠(橙赤)と駆動の丸点(Follow 橙)が同系**。同じ画面に出ると 1 秒で取り違える。A の赤 #FF4B3A は橙と離れていて良い。
- 「複数」と「変更」が色で同じ。複数は mode でも変更でもなく「混在」なので灰の下線か、別の形が要る。
- 固定(ハッチ)は色に頼らず形で言えており、CVD でも崩れない。これは手本。

### 1-3 Inspector I6 Reset(I6-d hovering a target)

| 観点 | 所見 |
|---|---|
| Grey | 白字で変更が拾いにくい。ホバー中の「→ 8.0」(プレビュー値)が通常値と同じ白 |
| A | 変更ラベルが空青の下線、値が空青。On のスイッチが**赤茶の塗り + 青の縁**で 2 色が 1 部品に乗る(騒がしい) |
| B | スイッチはスレート青のまま。落ち着く |
| deut | 青紫の下線が白と分かる。OK |

欠点: プレビュー(「→ 8.0」「→ 30.0」「→ Horizontal and Vertical」)が Role を持たず灰。未書き込みの「見るだけ」状態に色か形(斜体・点線)が無い。Role.info(灰)なので意図通りとも言えるが、変更の青いラベルの隣に灰のプレビュー値が並び、どちらがプレビューか迷う。

### 1-4 Inspector I9 Whip(I9-a Pick-whip mid-grab)

Grey / A / B で同一。橙の綱・橙の目盛り・上の選択行の薄い橙の面がすべて関係家族(Follow 橙)で描かれ、Role の knob を読んでいない。
deut / prot では橙が**黄褐色**に変わり、綱は残るが「橙の tick」と文の説明(「the grabbed ring is orange」)が成り立たなくなる。形(綱の曲線、左端の 2px 帯、30% に落とした不適合行)で意味は伝わるので CVD でも使える。減点は小さい。

### 1-5 Browser B2-a Tag bands (Ableton type)

Grey / A / B / deut のどれも同一。有効なチップ(Effect 8、Soft 8)は灰の塗り + 1px の青っぽい下線のみ。「2 filters」「Clear」「+7 hidden tags」も灰。
- (1) 「今どのフィルタが ON か」の見つけやすさは Grey と同じ。オーナーの「色が無くて読みにくい」はここで未解決。
- (2) 形の手掛かり(塗り + 下線)は有り。CVD でも壊れない。
- 欠点: 有効チップに Role.selected / Role.mode が掛かっていない。panel_browser_a.dart がチップの色を Role でなく中立灰で直書きしているとみられる。

### 1-6 Browser B5-c Own frame peek

プレビュー枠の点線の枠(外枠)だけが変わる: Grey=紫寄り、A=青、B=青紫。選択された候補行(Glow)は左の白い短い縦棒 + 灰の面。PREVIEW のチップは灰。
- 点線の枠の色が「提案(まだ書いていない)」の意味を持つのに、色の意味が Role に載っていない(Grey の紫と A の青が異なる=Role.mode の取り違えの疑い)。
- 形の手掛かり(点線)は有り。CVD でも崩れない。良い。

### 1-7 Timeline(timeline_parts In context / full timeline、Timeline の Layer bars)

| 観点 | Grey | A | B | A+deut / prot |
|---|---|---|---|---|
| 関係バー | 6 家族色 | 同左 | 同左 | **6 色が 3 色に潰れる**: Scatter / Along Path が灰、Face / Follow が黄、Stagger / Attach が青 |
| キー菱形 | 白 | **鮮黄 #FAF500(強い)** | 柔らかい黄 #F0E442 | 黄 |
| 選択行 | 白の左端の縦棒 | 同左 | 同左 | 同左 |
| 現在位置(playhead) | 青 | 青 | 青 | 青 |

- 欠点 1(重大): **Face(黄)のバーの上のキー菱形(黄)が見えなくなる**(A も B も)。Grey は白なので見える。キーは「バー上に載る最重要の印」なので、黄は家族色 Face と衝突する。黄 key を使うなら外周に暗い縁を 1px 足す(菱形にアウトライン)か、Face の家族色を黄から外す。
- 欠点 2: CVD で家族の色が潰れる(color-ud.md 3.4 の Stagger-Attach 5.3)。ただし行頭に家族名のラベルがあり、バーそのものの識別は名前で保たれる。
- 選択行は色でなく左端の白棒(形・位置)で出していて、CVD で崩れない。手本。
- Label cell / all states: ロックのみが青い鍵(Role.selected か linked)。hidden は薄い字 + 斜線の目。色以外の手掛かりが有り。良い。

### 1-8 Inspector(Relations タブ)と Transport の mode switch(補足で撮った物)

| 部品 | 所見 |
|---|---|
| Inspector Relations | A では On のスイッチの塗りが**赤茶(mode のピンク)**で、Scatter(ピンク家族)だけでなく Stagger(青)・Along Path(緑)の行でも赤茶。**ピンクが 2 つの意味(mode と Scatter 家族)を持つ衝突**。B は青紫のままで衝突しない。 |
| 同 trit | Stagger(青緑)と Along Path(薄緑)が近づく。スイッチの赤茶と Scatter のピンクは同色相のまま |
| 選択タブの縁 | 全 palette で青・青紫・青緑の細い縁。**ピンクではない**(良い) |
| Transport の Mode switch(EDIT / PLAY / EXPORT) | A でも**灰のプレート**。mode 色(ピンク)が載っていない。「mode = Role.mode」の約束を誰も使っていない部品 |

## 2. 焦点・選択の縁がピンクか(観点 3 の指定)

- 確認した範囲でフォーカス / 選択の縁がピンクになっている所は**無い**。選択は青系(タブ縁・playhead・Animate の下線)か白(行の左棒)。
- ただし「mode = ピンク」が、On のスイッチの塗り(Inspector)と静止行の菱形(I4-b)に広く出ている。これは縁ではなく**面と点**。A のピンクは Scatter の家族ピンク(#E974AB)と見た目が同じ色相で、画面の中に「ピンク = 散らす」と「ピンク = On」が共存する。

## 3. パレットごとの判定

### Palette A (CUDO 系): NOT RECOMMENDED(条件付きなら OK)
- 良い: changed の空青 #66CCFF は読みやすく、エラーの赤 #FF4B3A が橙の家族と離れる。deut でも崩れにくい(color-ud.md の最小 ΔE00 は 5.6 以上)。
- 悪い: (a) key の鮮黄 #FAF500 は強すぎて「安っぽい / うるさい」(観点 4)。Face バーでは完全に見えなくなる。(b) mode のピンクが静止行の菱形・On のスイッチにあふれ、Scatter 家族のピンクと同色相で衝突。(c) 1 部品(スイッチ)に赤茶の塗りと青の縁が同居し、1 行 2 色まで(規則)を守らない。
- 直せば OK: mode を「塗り」から「小さなチップ + 文言」に絞り、スイッチを灰青に戻す。

### Palette B (Okabe-Ito 系): RECOMMENDED(ただし修正 2 点つき)
- 良い: changed #56B4E9 は読める。key #F0E442 は落ち着く。mode は青紫で、Scatter のピンクと衝突しない。選択(青)と mode(青紫)が近いが、形(下線 vs チップ)で分かれる。見た目は Grey に最も近く、「安っぽさ」が出ない。
- 悪い: (a) **error #F0643A が Follow の橙家族と同系**(I2 の駆動の点 vs エラー枠)。(b) deut では changed(空青)と mode(薄紫)が ΔE00 2.6 で区別不能(計算表どおり)。ただし形で分かれ、実画面でも I4-b の「青紫の菱形」と「空青の下線」は別の部品なので誤読は少ない。(c) Face(黄)上の key 菱形が見えない(A と共通)。
- 条件: error を A の #FF4B3A に戻す(または赤紫寄りに)。key 菱形に暗い縁を付ける。

## 4. 具体的な欠点一覧(file 目安)

| # | 欠点 | 深刻度 | 場所の見当 |
|---|---|---|---|
| 1 | Face(黄)のバー上の key 菱形(黄)が消える(A / B 共通) | 高 | tokens.dart の Role.key、timeline の菱形描画(panel_timeline 系)。菱形に 1px の暗い縁、または Face の家族色を変更 |
| 2 | A の mode ピンクが Scatter 家族ピンクと同色相、On スイッチ・静止行菱形に大量に出る | 高 | tokens.dart Role.mode(A)、スイッチ部品(Controls の Switch)、panel_inspector_a_kit.dart の静止行の菱形 |
| 3 | Browser(B2 タグ帯 / B5)が Role を読まず Grey / A / B / CVD で同一。チップの ON が灰のまま | 高 | panel_browser_a.dart のチップ・候補行の色直書き。有効チップに Role.selected(下線 + 塗り) |
| 4 | B の error(#F0643A)が Follow の橙(駆動の点)と同系 | 中 | tokens.dart Role.error(B) |
| 5 | I9 Whip が Role を読まない。CVD で橙が黄褐色化 | 中 | I9 の綱・目盛りの色(家族色の直接参照)。形で伝わるので優先は低い |
| 6 | Transport の Mode switch が灰プレートのまま(mode 色を誰も使わない) | 中 | Shell / transport の Mode switch。C.mode の heuristic 分岐(選択=mode と誤判定する部品がある疑い) |
| 7 | エラーに記号(×・!)が無く、枠の色と文言だけ | 中 | I2 の「数値でない」、feedback 系の Error row。color-ud.md の「記号必須」に未達 |
| 8 | 「複数」と「変更」が同じ見た目、プレビュー値(「→ 8.0」)が灰で変更と見分けにくい | 低〜中 | I2 の混在、I6 のプレビュー。Role.info(灰)の斜体や点線で形を足す |
| 9 | CVD で 6 家族が 3 色に潰れる(Scatter / Along Path が灰、Face / Follow が黄、Stagger / Attach が青) | 低(名前ラベルで補える) | 関係バー。バーの端に家族の記号(形)を足せば解消 |
| 10 | 1 部品に塗り(赤茶)+ 縁(青)が同居 | 低 | A のスイッチ(2 の解消で消える) |

## 5. 観点ごとの合否(まとめ)

| 観点 | A | B |
|---|---|---|
| (1) 読みやすさ(Grey 比) | 良。ただし Browser では変化無し | 良。Browser では変化無し |
| (2) CVD で changed / key / linked / selected / mode / error を見分けられるか + 形の手掛かり | 概ね可。形(角印・下線・菱形・ハッチ・点線)が有り。error は記号が無く弱い | 同左。deut で changed と mode が近いが形で分かれる |
| (3) 色規律(大きい塗り無し・1 行 2 色・ピンク衝突) | 違反: ピンクの塗り(スイッチ)と家族ピンクの衝突、スイッチが 2 色 | 概ね遵守。error と橙家族の衝突が 1 件 |
| (4) 安っぽくない・落ち着き | 黄が鮮烈、ピンクの菱形が数十個で騒がしい | 落ち着く |
| (5) 色付き文字のコントラスト | 十分 | 十分 |
| (6) 灰のままの物 / 誤った色 | Browser 全般、I9、Mode switch、プレビュー値 / ピンク mode が広すぎる | 同左(ただしピンクは無い) |

## 6. 撮影した画像(実在するパス。拡大 zoom を含む)

すべて `/Users/member_ottoto/.claude/projects/-Users-member-ottoto-rust-ae-Motolii/77f277f7-741f-4e9f-8e94-ddd760f7f088/tool-results/` の下。ファイル名のみ記す。
- I4-b Grey 全体: `mcp-computer-use-blob-1790910694590-azcotf.jpg`
- I4-b A 全体: `mcp-computer-use-blob-1790910702039-svvklv.jpg` / A 拡大: `...1790910705539-19c6ni.jpg`
- I4-b B 拡大: `...1790910710378-r298xs.jpg`
- I4-b A+deut 拡大: `...1790910715100-3d6clz.jpg`
- I2 states A+deut: `...1790910733403-av43f6.jpg` / A: `...1790910735903-nbsd3m.jpg` / B: `...1790910742813-wds8wh.jpg` / Grey: `...1790910743369-wcba2j.jpg`
- I6-d Grey: `...1790910757129-dveibz.jpg` / A: `...1790910761067-vhdxzt.jpg` / B: `...1790910769040-457mgl.jpg` / A+deut: `...1790910769676-97plu1.jpg`
- I9 A+deut 全体: `...1790910788039-rytpq6.jpg` / A: `...1790910794797-4vdwrr.jpg` / B: `...1790910795401-nykkjz.jpg` / Grey: `...1790910803506-bntdap.jpg` / A+prot: `...1790910804092-mgmg7s.jpg`
- B5-c A+prot: `...1790910813554-5fhfdf.jpg`、拡大 prot `...1790910816353-q05ijl.jpg` / Grey `...1790910820948-d0a1ug.jpg` / A `...1790910821507-glaffe.jpg`
- B2-a A: `...1790910833969-hmn9m9.jpg` / Grey: `...1790910838466-1xqb3h.jpg` / B: `...1790910838996-qtsp70.jpg`
- Timeline(Layer bars) B: `...1790910858476-u6njib.jpg`、拡大 B `...1790910862259-ym2h5q.jpg` / A `...1790910866625-7bg1jo.jpg` / A+deut `...1790910867220-h8eedz.jpg`
- timeline_parts 全体 A+deut: `...1790910880982-9j74rg.jpg`、拡大 A+deut `...1790910883972-5eelio.jpg` / Grey `...1790910888401-01c12m.jpg` / A `...1790910888985-3rdt4y.jpg`(注: この 2 枚は撮影順で Grey = 白菱形、A = 鮮黄の菱形。後の B は `...1790910888401` と `...1790910888985` のどちらかとして取れているので、Grey / A の取り違えが有り得る。目視で白菱形の画像を Grey、鮮黄の画像を A とした)
- Label cell / all states A: `...1790910897330-s97te8.jpg`
- Transport Mode switch A: `...1790910904724-2evfsn.jpg`
- Inspector(Relations)A: `...1790910913361-p6l3ub.jpg`、拡大 A `...1790910916174-04z7uf.jpg` / B `...1790910920637-n04fjw.jpg` / Grey `...1790910921206-jycxbm.jpg` / A+trit `...1790910927175-qfy6zd.jpg`

## 7. 未実施 / 限界

- Workflow の「W1+W2 Select, inspect, edit」は到達できず(サイドバーの階層が深く、スクロールが重い)。代わりに timeline_parts と Timeline 部品を使った。
- 8 方向の全 CVD(prot / trit)は I9 / B5 / Relations でのみ。I4-b と I2 は deut のみ。
- 一部の撮影で、knob の切替を同一画面に重ね撮りしたため、Grey と B の取り違えが無いかは Dropdown の表示で確認した(B2 のみ「Palette B」を明示確認)。他は切替の順序から推定。
- 数値(ΔE、コントラスト比)は新規計算せず color-ud.md の値を引用。ピクセルの測色はしていない(目視)。
