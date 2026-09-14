# 書いて作る — スクリプト

窓で押す操作を、JavaScript で並べる。**名前は窓に出ている名前だけ。**
書いたものは普通の作品になる。開けば Timeline にキーが並び、Inspector で直せる。

p5.js のように 1 file 書けば出る。違うのは、出た後に手で仕上げられること。
LLM に書かせてもいい。窓に無い名前を書くと、窓にある名前を並べて断るので、読んで直せる。

## 走らせる

窓なら **File → Run Script…** で `.js` を選ぶ。今開いている作品に足される。
直したら **File → Rerun Script**。前の結果を戻して、同じ file を読み直して走らせる。
走った後に窓で手を入れていたら、その手を消さないように断る。
途中で断られたスクリプトは何も残さない。

窓を開かずに作品 file を作るなら(repo の根で):

```bash
MOTOLII_SCRIPT=$PWD/作品.js MOTOLII_SAVE=$PWD/作品.rrd cargo test -p motolii-ui --lib -- --ignored script_file
```

開くのは `scripts/motolii-ui.sh dev 作品.rrd`。

例は `motolii/ui/native/src/editor/script/examples/` にある(`intro.js`・`wave_grid.js`・`burst.js`)。

## 時間の考え方

**draw loop は無い。** スクリプトは 1 回だけ走り、キーを置く。動かすのは窓。
だから前のフレームの値を足していく書き方(`x += 1`)はできない。代わりに「何秒にどの値」を書く。
ループは層を作るのに使う。ずらしたい時は、キーの秒をずらす。

使えないもの — 使うとその場で止まり、代わりを言う:

| 書いたもの | 代わり |
|---|---|
| `Math.random()` | `random(seed)` — 同じ種なら毎回同じ絵 |
| `Date`・`setTimeout`・`requestAnimationFrame` | `key()` の秒 |

## 言葉(英語で書く、LLM にもこの節を渡す)

```text
Motolii script API. Every call is a window operation; names are the window's names.

comp({ width, height, fps, seconds, background })   // background "#rrggbb"
text(content, options) / rectangle(options) / roundedRectangle / ellipse / star / polygon / line
nullLayer(options) / particles(options) / camera(options)
  options: { name, <Property>: value, ... }            // e.g. { name: "Title", Position: [960, 540] }
group(...layers) -> Layer
effects() -> [effect names]
random(seed) -> () => 0..1

Layer
  .set(name, value)                  // a still value, no keys
  .key(name, seconds, value, ease?)  // ease: how the value travels from this key to the next
  .keys(name, [[seconds, value, ease?], ...])
  .names()                           // the property names this layer shows
  .fill("#rrggbb")                   // shape fill / text color
  .text(content) .font(family) .name(text) .parent(layer) .blend("Add")
  .projection("2D" | "2.5D" | "3D") .time(startSeconds, durationSeconds)
  .effect(effectName, { <Param>: value }) -> Effect

Effect
  .set(name, value) .key(name, seconds, value, ease?) .names() .enabled(bool)
  // grid rows are named with their column: "Rotation Each", "Position X Random"
  // a choice is written by its name (Along: "Circle"), a layer-picking field takes a Layer

Eases: Hold, Linear, Bezier, Bounce, Elastic, Cyclic, Random, Steps, ElasticSteps
Units: Position px (comp origin top-left), Scale [1, 1] = 100%, Rotation degrees, Opacity 0..1, time seconds.

Common names
  Shapes:    Position, Position Z, Scale, Scale Z, Rotation, Tilt X, Tilt Y, Opacity, Depth, Anchor
  Text:      Content, Position, Scale, Rotation, Opacity, Tracking, Line height, Fill, Alignment
  Particles: Rate, Life, Direction, Spread, Speed, Size, Color, Gravity, Wind, Turbulence, Seed, Connect Distance
  Camera:    Center, Target Z, Target, Orbit, Distance, Zoom, Roll
Ask the layer when unsure: layer.names(), effect.names(), effects().
```

## まだ無いもの

- TypeScript(型を剥がす道は選んである: swc_ts_fast_strip)
- 保存しただけで走り直す(今は Rerun Script を押す)
- 1 本のスクリプトを Undo 1 回で戻す(今は操作の数だけ戻る)
- スケッチの層(毎フレーム描く札)
