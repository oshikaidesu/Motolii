# 欄の性格を使われ方から読む — 次元解析(2026-09-08)

状態: **決定**(利用者「静的解析によるアプローチはできるか」→ 可、自走)。

## 何を決めたか

Vism の欄(parameter)の性格(角度・長さ・平行移動・量・不透明・段・数・時間・種)を、
**欄の名前ではなく shader の中での使われ方**から読む。名前は言語や表現でぶれるが、
`sin` に入る値は角度で、座標に足される値は平行移動で、色に掛かる値は量である。

決め方の順位:

1. manifest の `SUBTYPE`(宣言)。語彙は Blender の property subtype
   (PIXEL / FACTOR / PERCENTAGE / ANGLE / TIME / DISTANCE / TRANSLATION)に、無い物
   (SEED / OPACITY / COUNT / LEVEL)を足した
2. 使われ方の次元解析(`render/compositor/effects/subtype.rs`)
3. どちらも無ければ**不明のまま**。Test tab は名前の表へ fallback するが、嘘の性格は付けない

## 借りた定規

- **次元解析(units of measure の型推論)**: 足し算は同じ次元、掛け算は次元の積。ここでは
  6 種の起点(px 座標・0〜1 座標・色・alpha・時間・render size)からの汚染追跡に縮めた
- **naga の IR**: catalog が binding の検証にすでに使っている parser。parser の口は
  `catalog::parse_wgsl` 1 つ(自前天井 `naga::front` は 3 のまま)
- **Blender の subtype 語彙**: 宣言側の言葉

## 起点(名前を見ない)

| 起点 | 何で分かるか |
|---|---|
| px 座標 | `@builtin(position)`、FieldIn / SurfaceIn の `frame_position` / `world_position`(ABI) |
| 0〜1 座標 | fragment の `@location(0)` の vec2、`isf_FragNormCoord` |
| 色 / alpha | `textureSample` / `textureLoad` の戻り、その `.a`、SurfaceIn の `albedo` |
| render size / 時間 | group 1 の欄の直後の 2 slot(ABI の layout) |
| 欄 | group 1 の binding 0..n、hook では `FieldParams` / `SurfaceParams` の member |

## 規則(証拠 → 性格)

| 使われ方 | 性格 | 単位 |
|---|---|---|
| px 座標に足す / 引く | TRANSLATION(向きベクトルで伸ばしてからなら DISTANCE) | px |
| 0〜1 座標に足す | TRANSLATION | "" |
| 座標 / 欄、`1/render_size` を掛ける、座標由来と比べる | DISTANCE | px |
| `sin` `cos` `radians`、π/180 を掛ける | ANGLE | ° |
| 色に掛ける、`mix` の第 3 引数 | FACTOR | %(範囲 0〜1 か 0〜100) |
| `.a` に掛ける(色に掛ける票が無い時だけ) | OPACITY | % |
| 色(輝度)から引く、色と比べる、`pow` の指数 | LEVEL | "" |
| 時間に足す / 掛ける | TIME | |
| 整数へ cast、整数と比べる(ループ上限) | COUNT | |
| xor / shift / and | SEED | |
| 同じ `vec` に詰めて座標に足した欄同士 | 1 つの点(group) | |

票は**汚染追跡が収束した最後の周回だけ**で数える(途中の周回は local の値が揃っておらず、
`bloom` が純粋な intensity に見える瞬間があり、嘘の票が残った)。色や座標が混ざった後の
値は証拠にしない(純度)。関数呼び出しは引数と戻りで汚染を渡す(文脈非依存、4 周)。

## 写像(oracle、`catalog::character_oracle`)

| 欄 | 読めた性格 |
|---|---|
| blur.radius、glow.radius、bloom.radius | DISTANCE px |
| glow.threshold、bloom.threshold | LEVEL |
| gain.gain、glow.intensity、bloom.intensity、turbulent.amount | FACTOR |
| glass.roughness / metallic / transmission | FACTOR % |
| turbulent.size | DISTANCE px |
| turbulent.complexity | COUNT |
| turbulent.offset_x / y / z | TRANSLATION px、1 つの点 |
| turbulent.evolution | 解析では TRANSLATION(noise 空間をずらす)→ manifest に `SUBTYPE: TIME` |
| tri_led.glow | 不明(shader が読まず定数式にだけ使う) |

## 届け先

`EffectParamDescriptor { subtype, unit, group }` → snapshot の行 `subtype` / `unit` / `group`
→ Test tab は宣言・解析を名前表より優先し、group は `_x/_y` 命名なしでパッドに畳む。

## 限界

- noise / hash を通った値は無次元になるが、追跡は座標由来の印を残す(amount が FACTOR で
  済んでいるのは purity の規則のおかげ)。厳密には関数の要約が要る
- fork の helper(`simplex3` / `fbm3` / `shade_surface`)は stub で読む。fork が関数を足したら
  stub にも足す(`subtype::hook_stub`)
- 配置(Repeater 等、shader を持たない棚)は解析の外。`kind.rs` の宣言に subtype を書く席は未設
