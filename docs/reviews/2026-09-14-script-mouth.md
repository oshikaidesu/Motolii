# スクリプトの口 — p5.js と LLM ポン出しの客を迎える

2026-09-14 利用者と相談して決定。

## 動機

利用者の段取り: 将来 p5.js の利用者と LLM ポン出しの利用者を獲得しないと、今の市場で勝負するのは厳しい。
縛りは **実物に無いパラメータは無い** — スクリプトの名前は窓に出る名前だけ。バック(doc/render)とフロント(Flutter)が同じ表を見るので、両方の効率を取れる。

## 決めたこと

1. **言語は TypeScript/JavaScript**(2026-08-01 の作者言語の既決と揃える)。p5 もポン出しの LLM もここで書く。
2. **p5 から名前も機構も借りない。** 取るのは「1 file 書けば出る、保存したら出る」という手触りだけ。名前が似ると LLM と利用者が p5 の古い機構ごと持ち込む:
   - `draw()` の中で前のフレームの状態を足していく → 時刻を飛ばすと絵が変わる
   - `background()` を塗らずに残像を作る → 同上。残像は効果の仕事
   - `Math.random()`・`millis()` → 決定性が崩れる
   - `fill()`・`push()/pop()` といった全体の状態 → 意味が Document に残らず、窓で直せない
   - px 決め打ちの `createCanvas` → 解像度と素材座標の法に反する

   古い癖は黙って通さず、赤にして行き先を言う(例:「残像なら Echo 効果へ」「乱数なら種の帯へ」)。
3. **名前は属性の表から作ったものだけ**(`store/kind.rs` の `Param`、効果の ISF manifest の inputs)。窓の英語 label をそのまま名前にする。窓に無い名前がスクリプトに 1 つでもあれば赤にする test を置く。
4. **書き込みは Intent の列だけ。** Undo は今の操作と同じように効く。
5. **2026-08-31「汎用機構(ノードグラフ・式言語)は作らない」の改定:** 作ってよいのは「作品を組むスクリプト(Intent を並べる)」と「スケッチの層(閉じた札 1 枚)」。Document の中で属性ごとに式を書く AE の Expression 型は、引き続き作らない。
6. **順番:** ポン出しの口(TS → Intent + 名前の突き合わせ test)が先、スケッチの層は後。

## 時間の型(先例)

| 型 | 先例 | Motolii |
|---|---|---|
| 時刻の純関数 | Shadertoy `iTime`、Remotion `useCurrentFrame`、Hydra、Strudel/Tidal、Houdini `$F`、Blender Scene Time | 既にある: ISF、粒子の閉じた式 |
| 順に並べる台本 | Motion Canvas `yield*`、Manim `play`、GSAP timeline | ポン出しで「順に出す」を書く口。実行結果はキーの列として Document に落ちる |
| コードが値を宣言し、時間は UI のキーで | Theatre.js `sheet.object`、Cavalry | **足りない口。** 宣言先は属性の表にある名前だけ |
| 状態は囲って焼く | Blender Simulation Zone、Houdini DOP、TouchDesigner feedback | 既にある: host が持つ feedback(入点を初期条件にする) |

p5 との差: Timeline もキーも無く、書いた後に手で直せず、動画の書き出しが苦しい — Motolii が埋めるのはこの「作品に仕上げる」段。

## 取説で確かめたこと(2026-09-14)

- **Theatre.js**: `sheet.object(key, { x: types.number(0, {range}) })` で宣言すると Details Panel に欄が出て、右クリック「Sequence」で初めて時間の世界へ入る(Motolii の ◇ と同じ)。保存は object key + prop path ごとの JSON。rename と型変更で何が起きるかは取説に書かれていない(ソースを見る限り、古い path は置き去りになる)。https://www.theatrejs.com/docs/latest/manual/prop-types 、https://www.theatrejs.com/docs/latest/manual/projects
- **Motion Canvas**: `yield*` の台本でも scrub できるが、戻る時は scene を頭から実行し直している(ソース)。`waitUntil('event')` は editor で掴んで動かせる印になり、置き場は scene の meta file(乱数の種と同じ file)。https://motioncanvas.io/docs/time-events 、https://motioncanvas.io/docs/random-values
- **Remotion**: 絵は `useCurrentFrame()` だけから作る、どの順で描いても同じ絵、乱数は `random(seed)` だけ、`Math.random()` は lint で警告。https://www.remotion.dev/docs/flickering
- 写すもの: Theatre.js の「宣言 → 欄 → 押したら時間へ」と、rename で置き去りにしないこと(Motolii は表の id で持つ)。Motion Canvas の「コードの待ちを editor で掴める印にする」。Remotion の決定性の規則と、`Math.random` を lint で赤にすること。

## 未決

- JS エンジン(QuickJS 等)。取説を読んで比べてから選ぶ
- 台本型を実行した結果を、どの粒度でキーへ落とすか
- スケッチの層の中身(何を返すか)
