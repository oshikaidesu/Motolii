# カラーユニバーサルデザインと状態色の調査(2026-10-02)

目的: 「色が無くて読みにくい。ユニバーサルデザインを調べよ」(利用者)への答え。旧規則「色は関係の 6 色 + アクセント 1 つだけ」(panel-checklist 11) は見直し対象。ここは **選択肢と数値**であり、決定ではない。

出所タグ: `[seen]` 今回ページを取得して読んだ / `[snippet]` 検索結果の抜粋のみ / `[memory]` 取得できず記憶による(要再確認) / `[inferred]` 既知の挙動からの推論 / `[computed]` 今回 Python で計算。
取得の注意: WebFetch は小型モデルの要約を返す。Okabe-Ito の 16 進が誤って返ったので、下の値は公開値(記憶)を採り `[memory]` とした。CUDO ガイドブック PDF は文字抽出できず、16 進は `[memory]`。

---

## 1. 標準が言うこと

| 出所 | 言うこと | タグ |
|---|---|---|
| CUDO(NPO カラーユニバーサルデザイン機構) 推奨配色セット https://jfly.uni-koeln.de/colorset/ (カラーユニバーサルデザイン推奨配色セット) | 20 色。アクセント 9 色(赤 #FF2800 黄 #FAF500 緑 #35A16B 青 #0041FF 空 #66CCFF ピンク #FF99A0 橙 #FF9900 紫 #9A0079 茶 #663300)、ベース 7 色、無彩 4 色。「異なる色覚型でも見分けられる」ことが目的。赤と緑の混同を避ける | 構成=[seen] / 16 進=[memory] |
| 岡部・伊藤 Color Universal Design https://jfly.uni-koeln.de/color/ | 配色だけに頼らず**冗長符号化**: 形・線種・ハッチを併用、凡例でなく直接ラベル、細い物は色が分からないので太く・大きく、明度・彩度も変える。「色と形の両方で差を示せ」。パレット(Okabe-Ito): 橙 #E69F00 空 #56B4E9 青緑 #009E73 黄 #F0E442 青 #0072B2 朱 #D55E00 赤紫 #CC79A7 黒 | 原則=[seen] / 16 進=[memory] |
| WCAG 2.2 SC 1.4.1 Use of Color https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html | 色の違いだけで情報を伝えない。文字・記号・パターンを必ず併用 | [seen] |
| WCAG 1.4.3 / 1.4.11 | 文字 4.5:1(大きい文字 3:1)。UI 部品の境界・状態を示す図形は隣接色に対し 3:1 | [memory] |
| JIS Z 8501(安全色) | 赤=防火・禁止、黄赤=危険、黄=注意、緑=安全・進行、青=指示、赤紫=放射能。**対比色**(白黒)と組み合わせ形も併用する前提。本アプリは UI なので参考(赤=危険 / 黄=注意 / 緑=OK は一般認識と一致)のみ | [memory] |
| ColorBrewer(Brewer) https://colorbrewer2.org | 質的(カテゴリ)配色は CVD-safe 印付きで 8 色前後が上限。段階は明度差で作る | [memory] |
| Apple HIG / Material 3 / Adobe Spectrum | 状態色(error/warning/success/info)は暗い面で**明度を上げた別トーン**を持つ(Material は tone 80 前後)。色単独にせず**アイコン/文言**を付ける。Spectrum の semantic 色は informative / negative / notice / positive の 4 役 | [memory] 要再確認 |

読み取り: 「色を禁止」ではなく「**色を使ってよい、ただし色だけにしない**」が標準の結論。小さい図形(1px 線・3px 点)ほど色は伝わらないので、形か位置か明度差を併用する。

## 2. 出荷済み暗色ツールの状態色・役割色

| ツール | 役割 → 色相 | タグ |
|---|---|---|
| Blender | キー位置のフィールド=黄、アニメ済(キー無しフレーム)=緑、値を変えたがキー未登録=橙、ドライバ=紫、(版により)オーバーライド=青系。https://www.oreilly.com/library/view/learning-blender-a/9780133886283/ch12lev2sec4.html / 検索抜粋 | [snippet] |
| Blender | タイムラインのキー菱形: 選択=橙黄、非選択=白。テーマで状態色を個別編集可(Preferences > Themes) | [inferred] |
| After Effects | ラベル色 16 種(レイヤー分類用、役割ではない)。式=赤い字・ストップウォッチ、キー=グレー/黄菱形(選択で黄)、エラー式=黄の警告三角 | [inferred] |
| Ableton Live | クリップ色=ユーザー分類(色相 14 種)。録音=赤、選択=ウィンドウ全体の枠線が明るい青系、警告・オフライン=橙/黄、無効=グレーアウト | [inferred] |
| Figma | 選択=青(#0D99FF 前後)、変数・コンポーネント=紫、バリアント差分=紫系の点、エラー=赤 | [inferred] |
| DaVinci Resolve | 選択=赤橙のクリップ枠、タイムライン編集モード(Trim 等)=色変更、キー=グレー菱形 | [inferred] |

共通の相場: **黄=キー/注意、緑=アニメ済/OK、赤=録音/エラー、紫=ドライバ/変数/リンク、青=選択**。「変更あり」(changed-from-default)の色は出荷物に決まった相場が無く、Blender の橙が最も近い。上の相場は実機未確認の物が多い。後で実画面で確かめる。

## 3. 計算(Python、CIEDE2000、Machado 2009 severity 1.0 を線形 RGB へ適用)

基準面: ground #191919 / panel #202020 / bandA #1E1E1E / bandB #2A2A2A。参考: 主文字 #F2F2F2 は ground 15.71、g63 #A1A1A1 は 6.80、g44 #717171 は 3.60(bandB で 2.94)。

読み方: ΔE00 は 約 10 以上=はっきり別色、5〜10=並べれば別・離れていると怪しい、5 未満=区別不能とみなす。

### 3.1 候補パレット

| 役割 | 冗長な形の手掛かり | A (CUDO 系) | 対比 ground/bandB | B (Okabe-Ito 系) | 対比 ground/bandB |
|---|---|---|---|---|---|
| changed 変更あり | 角の三角印 + 点線下線 | #66CCFF | 9.75 / 7.96 | #56B4E9 | 7.62 / 6.22 |
| key/animated | 菱形(塗り=キー位置、輪郭=間) | #FAF500 | 15.18 / 12.39 | #F0E442 | 13.30 / 10.85 |
| linked/driven | 丸点 + 家族色の線(関係) | #C77DD8 (家族があれば家族色) | 6.16 / 5.03 | #CC79A7 | 5.74 / 4.69 |
| selected/focus | 1px 下線/外枠(位置) | #5C8DFF | 5.62 / 4.59 | #4A8FE0 | 5.27 / 4.30 |
| mode/active | 塗りの小チップ + 文言 | #FF99A0 | 8.64 / 7.05 | #8F9BEA | 6.74 / 5.50 |
| ok | チェック記号 | #3DBE82 | 7.44 / 6.07 | #1FB98C | 7.01 / 5.73 |
| warning | 三角 + 「!」 | #FF9900 | 8.21 / 6.70 | #E69F00 | 7.81 / 6.37 |
| error/danger | 丸×印 + 文言 + 太い縁 | #FF4B3A | 5.29 / 4.32 | #F0643A | 5.52 / 4.50 |
| info | 「i」記号 | #A1A1A1 (グレー) | 6.80 / 5.56 | 同左 | 同左 |
| disabled | 斜線なし、薄い字 + 縁なし | #717171 | 3.60 / 2.94 | 同左 | 同左 |

全役割が ground で 3:1 以上(非文字 1.4.11 OK)。4.5:1(文字)を bandB でも満たすのは A の changed/key/mode/ok/warning/info、B の changed/key/mode/ok/warning/error(4.50)。error A と selected A は bandB で 4.32 / 4.59 付近で文字値には使わず、記号・縁に使うのが安全。disabled は**意図的に** 3:1 級(無効の見た目)。

### 3.2 色覚シミュレーション後の 16 進

| 役割 | A prot | A deut | A trit | B prot | B deut | B trit |
|---|---|---|---|---|---|---|
| changed | #B3CAFF | #9EBAFE | #00DADD | #9BB3EC | #87A4E8 | #00C2C6 |
| key | #FFEB00 | #FFF12F | #FFE4D1 | #F8DC23 | #FCE34E | #FFD4C5 |
| linked | #7094DB | #849CD5 | #C888A0 | #808BA9 | #9498A5 | #D6788A |
| selected | #559AFF | #398BFD | #00A7BA | #6B94E3 | #5586DE | #00A2AD |
| mode | #ADAAA0 | #C5BD9E | #FF8E9C | #81A4ED | #7A9DE8 | #73ABB8 |
| ok | #BCB07E | #ABA486 | #00BDAE | #B4AB8A | #A29E8F | #00BAAC |
| warning | #B9A300 | #D2BB06 | #FF8182 | #B9A200 | #CAB411 | #FB8C87 |
| error | #807436 | #AD9B32 | #FF0049 | #8B7D35 | #AD9C35 | #FF435B |

### 3.3 役割色どうしの最小 ΔE00(info/disabled を除く 8 色)と最悪の組

| | 通常 | prot | deut | trit |
|---|---|---|---|---|
| A | 19.6 (mode-error) | **5.6 (linked-selected)** | 7.2 (mode-ok) | 5.6 (mode-warning) |
| B | 12.3 (selected-mode) | **5.0 (selected-mode)** | **2.6 (changed-mode)** | 6.5 (changed-ok) |
| 現状(accent 共用) | changed=selected=mode が同一(ΔE 0)、key=warning も同一 | | | |

A の次点(deut): changed-linked 8.9 / linked-selected 10.5 / warning-error 10.5。A の trit は changed-ok 9.6。B の次点(prot): changed-mode 5.5。

**区別不能になる組(<5)**: B deut の changed(空青)と mode(薄紫)=2.6。これは形(下線 vs チップ)で拾う。A は 5 未満が無いが 5〜8 の境界が 4 組(linked-selected prot、mode-ok deut、mode-warning trit、changed-linked 8.9 は弱)。**緑 ok と赤 error は両パレットとも prot/deut で茶〜オリーブに寄る**(ok #BCB07E 対 error #807436、明度差は残るが色相差が消える)=記号(チェック / ×)が必須。

### 3.4 関係 6 家族(#E974AB #4781E5 #7DD5B1 #EFCB4E #F69260 #A889E9)

| | 通常 | prot | deut | trit |
|---|---|---|---|---|
| 家族どうしの最小 ΔE | 18.5 (Stagger-Attach) | **5.3 (Stagger-Attach)** | 9.3 (Face-Follow) | 6.9 (Scatter-Follow) |

家族色の対比: Stagger #4781E5 は ground 4.64 / bandB 3.79(文字には不足、線・点なら可)。他は ground 5.9 以上。
家族色 vs 役割色の最悪: A は prot で Attach-linked 2.8、Attach-selected 3.4、trit で Follow-warning 2.6。B は prot で Scatter-linked 1.1、Attach-selected 1.9、deut で Attach-mode 1.4。つまり**家族色の面で役割色を重ねない**か、「linked」の役割色は家族色そのものを使い別の紫を足さない(推奨、A/B 共通)。

### 3.5 計算から出る設計上の結論

1. 暗色地では CUDO の素の青 #0041FF(対比 2.68)・紫 #9A0079(2.22)・茶(1.71)は使えない。明度を上げて別トーンにする(A の selected #5C8DFF / linked #C77DD8)。素の Okabe-Ito 青 #0072B2 も 3.39 で下線 1px には弱い。
2. 暗地で CVD に強いのは「明度差 + 青〜橙軸」。青〜橙〜黄の軸は prot/deut でも保たれ、赤〜緑軸は潰れる。error と ok を別トーンにしても、色だけに頼らない。
3. selected と mode は同じ青系に置くと prot/deut で潰れる。形(selected=下線、mode=塗りチップ)で分けるのが先で、色は第 2 の手掛かり。

## 4. 方針案(選択肢、未決)

- 許す場所: 状態印(菱形・角印・点)、小さい塗り(チップ・バッジ、面積が行高の 1 割未満)、1px の縁と下線、値の文字(changed の値のみ)。
- 許さない場所: 大きい塗り(行・パネル・バーの全面。バーは今まで通り関係色 1 本で、役割色と競合させない)、太い線(2px 超)、背景のグラデ。
- 密度と上品さ: 同時に出る役割色は 1 行に最大 2(Blender は 1 フィールド 1 色)。彩度は A/B の値より下げない(下げると対比を割る)が、**面積で抑える**。disabled と info は無彩のまま。
- 選択肢 P1 = グレー + 状態印だけ着色(最小)/ P2 = P1 + changed の値の文字色 + 行頭 3px 帯 / P3 = P2 + 関係のあるバーに家族色の細い縁。いずれも冗長な形を常に併記。
- 「changed」の色は出荷物に相場が無く、これだけは利用者の判断に近い(空青か橙か、本稿は空青)。

## 5. Widgetbook で試す計画

1 つの全局 knob `Colour`: **Grey (today)** | **Palette A** | **Palette B** | **Palette A + CVD sim (deuteranopia)**(4 つ目は全 UI に ColorFiltered で Machado 行列を掛け、利用者に見えをそのまま見せる)。

`book/lib/tokens.dart` に 1 か所で足す名前(各 knob 値で切替、Grey 時は全て今の灰/accent に落とす):
`Role.changed, Role.key, Role.linked, Role.selected, Role.mode, Role.ok, Role.warning, Role.error, Role.info, Role.disabled` と `Role.palette` (enum: grey/a/b)、`Role.cvd` (none/deut)。`C.danger` は `Role.error` へ寄せる。部品は番号を持たず `Role.xxx` だけ読む。

これを読むべき既存の部品・パネル(状態印を持つ物): changed の角印・点線下線(Inspector の値欄)、キー菱形(Timeline のキー、Inspector のキー欄)、driven/linked 点(Relations の行・関係欄)、エラー表示(`C.danger` の全使用箇所、Export・Browser の失敗行)、選択下線/外枠(部品の selected / focus、Timeline の選択行)、mode(accent の `C.mode` 全使用、Dose の selHue)、ok/warning(Export の結果、Browser の読込状態)、disabled(部品シートの無効状態)。panel-checklist 11 は「色は関係の 6 色とアクセント」から「役割色は Role だけ、色だけで伝えない」へ差し替え案。
