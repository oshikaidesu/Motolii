# 数値のメリハリ — クリエイティブ系の外の先例(2026-09-08)

状態: **観察 + 提案**(利用者「数値にメリハリがなくどこを触ればいいか分からない。クリエイティブソフトの体系で見るから駄目。数値をベースに触る、楽しいインタラクティブを重視した体験ベースの物を調査」)。

## 問題の言い直し

Test tab の井戸は全部同じ顔で並ぶ。読めるが、**どれが効くか・どこまで動くか・触ってよいか**が形に出ていない。
クリエイティブソフト(AE・Lightroom・Houdini)は表を前提にしているので、そこを見ても表が返ってくる。
「数値を触ることが体験の中心」の物を 6 系統見た。

## 先例と、それぞれが解いている物

| 先例 | 何をしているか | 解いている問い |
|---|---|---|
| **Teenage Engineering OP-1**(楽器) | 画面に出す欄は**常に 4 つ**、4 色の encoder と画面の色が 1:1。残りは shift の裏。「選択肢が少ないから Pro Tools より速い」 | **どれが効くか** — 制約そのものがメリハリ |
| **Bret Victor: Tangle / Scrubbing Calculator** | 文章の中の数字を**その場で左右にドラッグ**。触れる数字は色と点線の下線で見分けがつく。「式を反転するより、数字を擦って納得する方が自然」 | **触ってよいか** — 数字自身が取っ手 |
| **Desmos の slider** | 変数ごとに ▶ があり、押すと**範囲を自動で往復**して絵が動く | **何が起きるか** — 触る前に「この数はこう効く」が見える |
| **Explorable Explanations(Nicky Case)** | slider → 即座に結果、Sandbox は徐々に開く、「Place your bets」で先に予想させる | **遊びの設計** — 結果が見える速さと、少しずつ開く順 |
| **Vital(synth)** | 全 parameter が**自分の効果を自分で描く**(filter は曲線、wavetable は形)。変調された欄には動く弧 | **今どうなっているか** — 数字ではなく形で読む |
| **Apple Photos の調整 dial** | 目盛の帯・中央のゼロに detent・触覚。絵そのものが表示器 | **量感と基準** — ゼロと既定が手に伝わる |
| **Lightroom** | slider の帯が**効果の向き**を色で示す(色温度は青→黄)。hover で histogram の該当域が光る | **どこに効くか** — 帯が説明 |
| **Ableton Macro / Variations** | 何十の欄を **8 つの macro** に束ね、Variations で保存、🎲 で macro を乱数 | **多すぎる欄** — 束ねる・振る・戻す |
| **Game Builder Garage**(Nintendo、反例) | Nodon の設定は「生の数字と XYZ」で、レビューが「**尺度と基準点が無い**」と指摘 | 生の数字は子供向けでも遊べない |

## 抽出した規則(Motolii の欄に写す)

1. **主役は 4 つまで**(OP-1)。効果ごとに主役の欄を決め、大きく・色つきで上に。残りは Advanced の裏。
   主役は manifest の `HERO`(または宣言順の先頭 4)。Repeater なら Count / Along / Position each / Rotation each
2. **触れる数字は触れる顔**(Tangle)。編集できる数字だけ点線の下線と ink、読み取り専用は muted。
   今の井戸は全部同じ箱なので、箱ではなく数字の顔で区別する
3. **基準と尺度を必ず添える**(Apple / Game Builder Garage の反例)。既定値の目盛、双極(−x〜x)は中央ゼロ、範囲がある物は塗り、単位は小文字
4. **▶ で「何をする欄か」を見せる**(Desmos)。欄の ▶ を押すと範囲を 1 秒で往復 preview(キーは打たない、Esc で戻る)。言葉なしで欄の意味が伝わる
5. **欄が自分の効果を描く**(Vital / Lightroom)。角度はダイヤル(済)、量は塗り(済)、色温度型は帯に色、キーがある欄は小さな折れ線
6. **束ねる・振る・戻す**(Ableton)。効果ごとの 🎲(範囲内で乱数、1 undo)、Variations(効果の状態を 8 つ保存して切替)、既定へ戻す

## 優先(私の推奨)

安く、言葉が要らず、今の宣言だけで付く順:

- 2(触れる顔)と 3(既定の目盛・中央ゼロ)— 井戸の描き方だけ
- 4(▶ 往復 preview)— previewProperties → cancelPreview の既存経路で 1 秒の sweep
- 6 の 🎲(効果ごと)— setProperty の束、既定へ戻すは native の command が要る
- 1(主役 4 つ)— 宣言 `HERO` を足す(無ければ先頭 4)。裁定要: 何を主役にするかは効果の作者の判断
- 5 の折れ線と Variations — 後

## 出典

- Teenage Engineering OP-1 layout guide: https://teenage.engineering/guides/op-1/original/layout 、"Constraints as Aesthetic": https://blakecrosley.com/guides/design/teenage-engineering
- Bret Victor, Scrubbing Calculator: https://worrydream.com/ScrubbingCalculator/ 、Tangle / Explorable Explanations: https://worrydream.com/
- Desmos, Sliders and Movable Points: https://help.desmos.com/hc/en-us/articles/202529069-Sliders-and-Movable-Points-in-a-Graph
- Nicky Case, Explorable Explanations 4 more design patterns: https://blog.ncase.me/explorable-explanations-4-more-design-patterns/
- Vital: https://vital.audio/
- Apple Photos の調整(触覚 dial): https://www.macworld.com/article/233390/how-to-take-advantage-of-the-photo-editing-tools-in-ios-13-and-ipados-13.html
- Lightroom slider と histogram: https://lightroomkillertips.com/which-lightroom-sliders-effect-which-parts-of-your-image/
- Ableton Racks / Macros / Variations: https://www.ableton.com/en/manual/instrument-drum-and-effect-racks/
- Game Builder Garage review(尺度が無い数字の反例): https://www.pastemagazine.com/games/game-builder-garage/game-builder-garage-review
