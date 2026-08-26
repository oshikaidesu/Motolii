# layer 属性 vs property — 「静的値の置き場が property の外にある」の棚卸し

日付: 2026-08-25 / 状態: **観察**。裁定ではない。write-set は本ファイルと
`next/DECISIONS.md` の追記2行(裁定238/239)のみで、実装コードは1行も触っていない
(`next/core/motolii-store/`・`next/ui/motolii-inspector-pane/` は読むだけ)。

## 0. 判定基準と方法

判定基準は**1文だけ**である。この文の外に基準を作らない:

> **layer は property の集合しか持たない。値と補間は property が持つ。静的値は
> keyframe 0本の property である。**

出典は Lottie のモデル(`ks` 配下の各 transform が animatable property であり、値と
in/out tangent は keyframe 側が持つ)。裁定215「借りるのが既定」の「借りる」の形そのもの。

3値の判定:

- **P** — property であるべき。時間で変わりうる / 補間の対象になりうる
- **A** — layer 属性のままでよい。同一性・構造・参照など、補間の意味がない物
- **?** — 判断が要る。**書かずに保留**し、理由を書く(§4 に一覧)

各行に **現状の実装場所(file:line)**・**現状の書き口**(`Intent::Set*` の種別)・
**対応する `PropertyId` が既に在るか**を付けた。「在るか」は自己申告ではなく
`next/core/motolii-store/src/` の `impl crate::PropertyId` ブロックと
`pub mod property` の定数群を grep して裏取りしている。

この棚卸しは**裁定214(Inspector 時間軸監査)の続きではない**。裁定214 は
「Inspector に映る物は時間軸に乗るか」を問うた。ここが問うのは1段手前
——**そもそも値がどこに住んでいるか**である。

## 1. 棚卸し表

### 1.1 `LayerAttrs`(`next/core/motolii-store/src/attrs.rs`)

| # | 値 | 実装場所 | 書き口 | 対応 `PropertyId` | 判定 |
|---|---|---|---|---|---|
| 1 | `hidden` | `attrs.rs:162` | `Intent::SetAttrs`(`LayerAttrsPatch.hidden`) | **在る**・`PropertyId::hidden()` `attrs.rs:260`(doc が「配線済み」— `resolve_with_solo` が `value_at` 経由で読む) | **P** |
| 2 | `parent` | `attrs.rs:165` | `Intent::SetAttrs` | 無 | **A**(層の所属 = 参照。裁定173 §4.4 が「正本は常に子側の1フィールド」と定めた構造そのもの) |
| 3 | `blend_mode` | `attrs.rs:167` | `Intent::SetAttrs` | **在る**・`PropertyId::blend_mode()` `attrs.rs:268`(doc が「配線済み」) | **P** |
| 4 | `matte.mode` | `attrs.rs:144`(保持は `attrs.rs:169`) | `Intent::SetAttrs` | **在る**・`PropertyId::matte_mode()` `attrs.rs:279`(doc が「配線済み」) | **P** |
| 5 | `matte.layer` | `attrs.rs:143`(保持は `attrs.rs:169`) | `Intent::SetAttrs` | 無(`matte_mode` の doc が「**`Matte.layer`(参照先)は対象外**」と明記) | **A**(参照) |
| 6 | `name` | `attrs.rs:172` | `Intent::SetAttrs` | 無 | **A**(同一性。裁定214 が名指し) |
| 7 | `auto_orient` | `attrs.rs:175` | `Intent::SetAttrs` | 無 | **P**(出力に効く bool。`hidden` と同型) |
| 8 | `pinned` | `attrs.rs:179` | `Intent::SetAttrs` | 無 | **P**(カメラ変換を受けるかを切り替える = 出力に効く。`hidden` と同型) |
| 9 | `solo` | `attrs.rs:186` | `Intent::SetAttrs` | **在る**・`PropertyId::solo()` `attrs.rs:252`(doc が「配線は完了」) | **P** |
| 10 | `locked` | `attrs.rs:193` | `Intent::SetAttrs`(解除だけは常に通る) | 無 | **?**(§4) |
| 11 | `label_color` | `attrs.rs:203` | `Intent::SetAttrs`(二重 `Option`) | 無 | **?**(§4) |
| 12 | `frozen` | `attrs.rs:222` | **`Intent::Freeze`/`Unfreeze` 専用**(`LayerAttrsPatch` に無い) | 無 | **?**(§4) |

### 1.2 `LayerMeta` / `LayerTiming`(`next/core/motolii-store/src/lib.rs`)

| # | 値 | 実装場所 | 書き口 | 対応 `PropertyId` | 判定 |
|---|---|---|---|---|---|
| 13 | `meta.source` | `lib.rs:576` | `Intent::SetMeta`(新規配置専用)/ `Intent::SetSource` | 無 | **A**(素材の種別と参照) |
| 14 | `meta.order` | `lib.rs:578` | `Intent::SetMeta` / `Intent::SetOrder` | 無 | **A**(重ね順 = 構造) |
| 15 | `timing.start` | `lib.rs:313` | `Intent::SetTiming` | 無 | **?**(§4) |
| 16 | `timing.duration` | `lib.rs:315` | `Intent::SetTiming` | 無 | **?**(§4) |
| 17 | `timing.source_in` | `lib.rs:317` | `Intent::SetTiming` | **同名は無いが、時間で変わる版は既に在る** — `property::TIME_REMAP` `lib.rs:165`(「値がそのまま**素材のフレーム番号**」・「track が無ければ通常どおり `LayerTiming::source_frame` の写像を使う」) | **P**(TIME_REMAP の存在自体が「時間で変わりうる」の実証。静的値が property の外に居るという形は #18 と同じ) |
| 18 | `timing.speed` | `lib.rs:321` | `Intent::SetTiming`(shell の read-modify-write。§2) | **器だけ在る** — `property::SPEED` `lib.rs:173` + `PropertyId::speed()` `attrs.rs:293`。**配線未完**(§2) | **P** |

### 1.3 layer 直下の他 component(列・存在)

| # | 値 | 実装場所 | 書き口 | 対応 `PropertyId` | 判定 |
|---|---|---|---|---|---|
| 19 | `Layer:present`(墓標) | `components.rs:237` | `Intent::AddLayer` / `RemoveLayer` | 無 | **A**(存在 = 構造) |
| 20 | `Layer:masks`(列そのもの) | `components.rs:185` | `Intent::SetMasks` / `AddMask` | 無 | **A**(並びと同一性 = 構造) |
| 21 | `Mask.mode` | `mask.rs:158` | `Intent::SetMasks` | **在る**・`PropertyId::mask_mode()` `mask.rs:131`(doc が「配線済み」— `resolved_masks` が読む) | **P** |
| 22 | `Mask.inverted` | `mask.rs:161` | `Intent::SetMasks` | **在る**・`PropertyId::mask_inverted()` `mask.rs:136`(doc が「配線済み」) | **P** |
| 23 | `Mask.id` | `mask.rs:153` | `Intent::SetMasks` / `AddMask` | 無 | **A**(同一性) |
| 24 | `Layer:effects`(列そのもの) | `components.rs:256` | `Intent::SetEffects` | 無 | **A**(並びと同一性 = 構造) |
| 25 | `EffectInstance.id` | `effect.rs:58` | `Intent::SetEffects` | 無 | **A**(同一性) |
| 26 | `EffectInstance.plugin_id` | `effect.rs:62` | `Intent::SetEffects` | 無 | **A**(どの plugin か = 参照) |
| 27 | `Layer:shapes`(列そのもの) | `components.rs:267` | `Intent::SetShapes` | 無 | **A**(内側の値は裁定173 H4 の語彙で、本監査の粒度外) |
| 28 | `Layer:text`(`TextDocument`) | `components.rs:276` | `Intent::SetTextDocument` | 一部在る(`PropertyId::text_justify()` `text.rs:635`、`text_style.*`/`text_range.*` 前置) | **A**(文書そのものは構造。内側の値は裁定92 の別棚卸し) |

### 1.4 件数

| 判定 | 件数 |
|---|---|
| **P** | 10 |
| **A** | 13 |
| **?** | 5 |
| 計 | 28 |

### 1.5 既に直した先例が1件ある(事実の記録)

`EffectInstance.enabled` は**元は静止 `bool` フィールドだった**が、裁定213(利用者裁定
「効果の on/off をキーフレームで設定できるようにして解決」)で `EffectInstance` から
**消し**、`effect.{id}.enabled` の平坦 track へ移した(`effect.rs:18-34`)。
本棚卸しの P 群は、この移送と同じ形の残りである。

## 2. 1件目の実例: Speed が二重化している

- ① `ATTRS` section の Speed 欄(%表示) → `Intent::SetTiming` の read-modify-write →
  `LayerTiming.speed`(静的な1点) → **動く**。写像は
  `next/ui/motolii-inspector-pane/src/attrs.rs`(`percent_to_speed_ratio` /
  `speed_percent`。組み立てと duration 再計算は `motolii-shell::Shell::apply_speed`)
- ② `PropertyId::speed()` / `property::SPEED` → keyframe track の器は在る(裁定214)が
  **配線未完**。`next/core/motolii-store/src/attrs.rs:284-296` が自ら
  「**この track を Inspector が書いても再生速度は変わらない**」と書いている
- 判定 = **P**。かつ「Speed が特別なのではなく、**静的値の置き場が property の外にある**
  ことが原因」なので、他の属性でも同型が起きうる —— これがこの棚卸しの動機である

同じ目で見るべきと名指しされていた候補の結果: ATTRS の blend(#3)・matte(#4/#5)は
**器も配線も既に在る**(裁定214 で先に閉じていた)、timing 系は source_in(#17)が P、
trim を成す start/duration(#15/#16)は **?** で保留した。

## 3. Inspector 側の帰結(調査のみ・裁定なし)

利用者の指摘は「**Inspector を毎回工事している。あくまで窓であるべきで、コンポーネントの
旨味を取れていない**」。これを裏付ける実測:

- `next/ui/motolii-inspector-pane/src` は **10,059行**。手書きの `*_section` 関数が
  **8本** —— `audio_section`(`audio.rs:32`)/ `attrs_section`(`attrs.rs:117`)/
  `mask_section`(`mask.rs:153`)/ `effects_section`(`effects.rs:290`)/
  `link_section`(`link.rs:201`)/ `shape_fill_section`(`shape_fill.rs:159`)/
  `shape_stroke_section`(`shape_stroke.rs:163`)/ `shape_section`(`shape.rs:113`)
- `TransformField`(`next/ui/motolii-inspector-pane/src/transform/mod.rs:56`)は
  property を1つ足すたびに variant が増える設計で、doc コメント自身が
  「既存の値セル文法へそのまま乗るために `TransformField` を拡張する形を採る」旨を
  繰り返している(`transform/mod.rs:67`(MaskOpacity)・`transform/mod.rs:77`
  (EffectParam)、および `mask.rs:41` / `effects.rs:115` / `audio.rs:12` の同趣旨の記述)。
  現在の variant は Position X/Y/Z・Scale X/Y・Rotation・Opacity・Anchor X/Y・
  MaskOpacity(MaskId)・MaskExpansion(MaskId)・EffectParam(EffectId, GlowParam)・
  Level・Pan・FadeIn・FadeOut
- **決定的な証拠**: `GlowParam` は `next/ui/motolii-inspector-pane/src/effects.rs:44` の
  ハードコード3 variant enum(Threshold / Intensity / Radius)で、
  `TransformField::EffectParam(EffectId, GlowParam)` という**型シグネチャに特定
  plugin 1本が焼き込まれている**。2本目の effect plugin を足すと、Inspector の enum と
  型を書き換えることになる
- 背景(事実として): store は plugin の param カタログを知らない(裁定70 —
  `effect.rs:80-84` が「store は plugin の param カタログを知らないので、既定値を埋める
  仕事はここではなく呼び手(plugin 定義を知っている層)の役目」と明記)。その知識の
  置き場が、現状は Inspector の閉じた enum になっている。`GlowParam` の doc も
  「**enum で閉じる** — `TransformField`/`KeyRow` は `Copy` なので param 名を `String` で
  運べない」と、閉じた理由を型の制約として名乗っている。良し悪しの判断はここでは書かない

**未裁定**: property が data になれば Inspector は列を投影するだけになりうるが、
descriptor(ラベル・単位・範囲・刻み・感度・表示形式)の持ち主をどこにするかは未決。

## 4. `?` の5件 — 判断が要るので書かなかった

| # | 値 | 保留の理由 |
|---|---|---|
| 10 | `LayerAttrs.locked` | **描画には無関係**で(`attrs.rs:187` が「合成器・resolve は読まない」と明記)、効くのは書き口の許可だけ。同一性でも構造でも参照でもないので A の定義に収まらないが、「時間で変わる locked」に意味があるかは基準文からは出ない。判定には別の裁定が要る |
| 11 | `LayerAttrs.label_color` | 表示専用の差し色で、パレット index(RGB ではない)。裁定214 の棚卸し(`docs/reviews/2026-08-23-inspector-time-axis-audit.md` FINDING 2)でも**未決**のまま。判定を変える新しい事実をこの監査では見つけていないので、そのまま保留する |
| 12 | `LayerAttrs.frozen` | doc 自身が「**Document の意味は1bit も変わらない**(裁定119 OUTCOME) — 凍結は導出キャッシュの許可証であって Document データではない」と名乗っている。Document データでない物をこの基準で判定してよいかがまず未決 |
| 15 | `LayerTiming.start` | comp 上の配置そのもの。「時間で変わる start」は自己参照になるが、それを理由に A と言い切るのは**この基準文の外の議論**(自己参照の可否は基準文が触れていない)。#17/#18 と違い、時間で変わる版の property が既に在るという実証もない |
| 16 | `LayerTiming.duration` | #15 と同じ理由。start/duration は trim を成す一対で、片方だけ判定を割ることはできない |

## 5. 本監査が触っていないもの

- `Composition` / camera の property(`camera.center` / `camera.zoom` / `camera.roll`、
  `lib.rs:182-186`)は layer 属性ではないので対象外。裁定239(向き)の前提条件が本監査を
  名指ししているが、camera 側の棚卸しは別途
- `Layer:shapes` / `Layer:text` の内側の語彙(裁定173 H4 / 裁定92)は粒度が違うので
  列の1行として扱った
- descriptor(ラベル・単位・範囲・刻み・感度・表示形式)の設計は**していない**(§3 末尾)
