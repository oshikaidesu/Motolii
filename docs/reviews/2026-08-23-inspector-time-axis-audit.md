# Inspector 時間軸監査 — 裁定214(identity/property の境界)の棚卸し

日付: 2026-08-23 / 状態: **棚卸し完了**。write-set は本ファイルと `next/check.sh` のみ、
`next/core/motolii-store/`・`next/core/motolii-eval/` は別レーン編集中のため不読(型定義の
確認だけ grep で行い、コードは1行も触っていない)。

前提: `git merge main` 実行済み(差分0、already up to date)。

## 0. 根拠と方法

- `next/DECISIONS.md` **裁定214**: 「Inspector に映る物は全て時間軸で評価できる」。境界は
  **identity と property の区別** — 名前・id・フォントの実体 path は identity なので乗らない、
  それ以外は全て track を持つ。
- **裁定213**: 変調(加算)可否は別の話。`Bool`/`Enum`/`LayerId` は補間 Hold でキーフレームは
  打てるが加算はされない。
- `next/core/motolii-eval/src/value.rs::Value` — **`String` バリアントが無い**(確認済み、
  読むだけ)。バリアントは `F64`/`Vec2`/`Color`/`Bool`/`Path`/`Enum`/`LayerId` の7つ。
- 「track を持つか」は自己申告ではなく、以下を grep で裏取りした:
  - `next/ui/motolii-inspector-pane/src/transform.rs::property_id`/`key_row_property_id`
    (`PropertyId` を作る唯一の場所 — ここに載っていない値は `Intent::SetTrack` を経由しない)
  - `next/core/motolii-store/src/lib.rs` の `property::` 定数群(実在する標準 property 名)
  - 各 section の `Intent::` 呼び出し(`SetAttrs`/`SetMasks`/`SetEffects`/`SetTextDocument`/
    `SetPropertyLink` — これらは丸ごと差し替えで `KeyframeTrack` を経由しない)

対象ファイル(発注書指定9本 + 実在するが発注書に無い2本、`link.rs`/`matte.rs` —
2026-08-22 発注「レイヤーを指す」文法で今回の発注書より後に着地した2 section。
裁定214 の対象は「Inspector に映る物」全部なので、発注書の書き漏れとして追加監査した
— FINDING 参照):

```
attrs.rs / audio.rs / chrome.rs / color.rs / effects.rs / lib.rs /
link.rs / mask.rs / matte.rs / projection.rs / text.rs / transform.rs
```

## 1. 棚卸し表

「〇欄」列は同種の行が複数回現れるもの(mask 1枚ごと・effect 1本ごと・LINK 5行)を
1種として数えた行数。判定の3値は発注書のとおり: **乗っている** / **K=乗せるべきなのに
乗っていない** / **identity**。型が `Value` に無く identity と言い切れない物は **未決**。

### TRANSFORM / APPEARANCE(`transform.rs`・`projection.rs`)

| 欄 | section | 型 | track | 判定 |
|---|---|---|---|---|
| Position X/Y | TRANSFORM | `Value::Vec2` | ○(`property::POSITION`、Key列あり) | 乗っている |
| Position Z | TRANSFORM | `Value::F64` | △(`property::POSITION_Z` は実在し値セル/drag は書けるが、**Key列は `KeyRow::Position` に含まれず**明示的な初回キー追加の入口が無い — `transform.rs` 冒頭コメントが「1 click = 1 undo を保つための仕様」と明記) | 乗っている(ただし FINDING 1 参照) |
| Scale X/Y | TRANSFORM | `Value::Vec2` | ○(`property::SCALE`) | 乗っている |
| Rotation | TRANSFORM | `Value::F64` | ○(`property::ROTATION`) | 乗っている |
| Opacity | APPEARANCE | `Value::F64` | ○(`property::OPACITY`) | 乗っている |
| Anchor X/Y | TRANSFORM | `Value::Vec2` | ○(`property::ANCHOR`) | 乗っている |

### ATTRS(`attrs.rs`・`chrome.rs`(ident帯)・`matte.rs`)

| 欄 | section | 型 | track | 判定 |
|---|---|---|---|---|
| Name | ident帯 | `String`(`LayerAttrs.name`) | ✗(`Intent::SetAttrs`) | **identity**(裁定214 が名指し) |
| Hidden(M glyph) | ident帯 | `bool`(`LayerAttrs.hidden`) | ✗(`Intent::SetAttrs`) | **K**(裁定214 本文が名指しした「AE のレイヤースイッチ」の実例そのもの) |
| Label Color(色チップ) | ident帯 | `Option<u8>`(`LayerAttrs.label_color`、palette index) | ✗(`Intent::SetAttrs`) | **未決**(FINDING 2) |
| Blend | ATTRS | `BlendMode`(enum、`LayerAttrs.blend_mode`) | ✗(`Intent::SetAttrs`、巡回ボタン) | **K** |
| Speed | ATTRS | `Speed`(`num/den`、`LayerTiming.speed`) | ✗(`Intent::SetTiming`) | **K** |
| Matte 元(source layer) | ATTRS(matte行) | `LayerId`(`Matte.layer`) | ✗(`Intent::SetAttrs`) | **未決**(FINDING 2) |
| Matte mode | ATTRS(matte行) | `MatteMode`(enum、`Matte.mode`) | ✗(`Intent::SetAttrs`、巡回ボタン) | **K** |

### MASK(`mask.rs`、mask 1枚あたり)

| 欄 | 型 | track | 判定 |
|---|---|---|---|
| Mask id(ラベル表示のみ) | `MaskId` | — | **identity**(どの mask かの識別子、`LayerId` と同格) |
| Mode | `MaskMode`(enum) | ✗(`Intent::SetMasks`、巡回ボタン) | **K** |
| Inverted | `bool` | ✗(`Intent::SetMasks`) | **K** |
| Opacity | `Value::F64` | ○(`PropertyId::mask_opacity`、Key列あり) | 乗っている |

### EFFECTS(`effects.rs`、effect 1本あたり)

| 欄 | 型 | track | 判定 |
|---|---|---|---|
| 名前(plugin表示名) | `String`(plugin_id から導出、`EffectInstance.plugin_id`) | — | **identity**(どの plugin かの識別、フォント path と同格) |
| Enabled(Bypass) | `bool`(`EffectInstance.enabled`) | ✗(`Intent::SetEffects`) | **K**(**裁定213 が名指しした本人** — 「`EffectInstance::enabled` を単なる bool フィールドから track へ」という裁定の文言そのものが未着手であることを確認) |
| Param(Threshold/Intensity/Radius) | `Value::F64` | ○(`PropertyId::effect_param`、Key列あり) | 乗っている(3欄) |
| 適用順(↑↓ reorder) | 構造(Vec の並び) | — | 対象外(値ではなく列の並び、Position/Scale と同格の「値」ではない) |

### AUDIO(`audio.rs`)

| 欄 | 型 | track | 判定 |
|---|---|---|---|
| Level | `Value::F64`(`property::LEVEL`) | ○ | 乗っている |
| Pan | `Value::F64`(`property::PAN`) | ○ | 乗っている |
| Fade In | `Value::F64`(`property::FADE_IN`) | ○ | 乗っている |
| Fade Out | `Value::F64`(`property::FADE_OUT`) | ○ | 乗っている |

### TEXT(`text.rs`・`color.rs`)

| 欄 | 型 | track | 判定 |
|---|---|---|---|
| Content(本文) | `String`(`TextDocument::content`、`ContentTrack`) | △(FINDING 3 — 別種の track は既にあるが Inspector の Key 列には乗らない) | **未決** |
| Font Family + path | `String`(`FontRef.family`/`path`) | ✗(`Intent::SetTextDocument`) | **identity**(裁定214 が名指し) |
| Size | `f32`(`TextDocumentStyle.size`) | ✗(裁定92「v1でキーフレーム化しない」、`Intent::SetTextDocument`) | **K**(FINDING 4 — 裁定92 と裁定214 の衝突) |
| Line Height | `Option<f32>`(`TextDocumentStyle.line_height`) | ✗(同上) | **K**(同上) |
| Tracking | `f32`(`TextDocumentStyle.tracking`) | ✗(同上) | **K**(同上) |
| Justify | `TextJustify`(enum、`TextDocument.justify`) | ✗(`Intent::SetTextDocument`、巡回ボタン) | **K** |
| Fill Color | `[f64;4]`(`TextDocumentStyle.fill`) | ✗(`Intent::SetTextDocument`) | **K**(`Value::Color` が既に在るのに一番素直に乗る型) |
| Stroke Color | `Option<[f64;4]>`(`TextDocumentStyle.stroke_color`) | ✗(同上) | **K** |

### LINK(`link.rs`、`LinkTarget::ALL` の5行)

| 欄 | 型 | track | 判定 |
|---|---|---|---|
| Link 元(source layer + property) | `PropertySource::Link`(`LayerId`+`PropertyId`) | — (`PropertySource` は `Track`/`Link` の二択そのもの、`KeyframeTrack` の中身ではない) | **未決**(FINDING 2) |

### chrome(`chrome.rs`、ident帯の残り)

| 欄 | 型 | track | 判定 |
|---|---|---|---|
| Kind ラベル(solid/media/text/…) | `&'static str`(`LayerSource` variant から導出、読み取り専用) | — | **identity**(層の種別そのもの、書き換え不能) |
| Solo(S glyph) | — | — | 対象外(`reserved_glyph` は空白のみ描画 — 「まだ出さない」段階で、そもそも何も映っていない) |

## 2. 数

**欄の種類ベース**(mask/effect のように複数インスタンスに繰り返す行は1種として数える。
Position Z を「乗っている」に含めた場合の数):

| 区分 | 数 | 内訳 |
|---|---|---|
| 乗っている | 14 | Position(XY)・PositionZ・Scale(XY)・Rotation・Opacity・Anchor(XY)・MaskOpacity・EffectParam×3(Threshold/Intensity/Radius)・Level・Pan・FadeIn・FadeOut |
| **K(乗せるべきなのに乗っていない)** | **13** | Hidden・Blend・Speed・MatteMode・MaskMode・MaskInverted・EffectEnabled・TextSize・TextLineHeight・TextTracking・TextJustify・TextFill・TextStroke |
| identity | 6 | Name・FontFamily/path・MaskId・EffectName(plugin表示名)・KindLabel・(EFFECTS の適用順は「値」に該当しないため対象外扱い、この6には含めない) |
| 未決 | 4 | LabelColor・MatteSource(layer参照)・LinkSource(layer+property参照)・TextContent |

合計 = 14 + 13 + 6 + 4 = **37 種の欄**(mask/effect のインスタンス複数化・LINK の5行分割・
Position/Scale/Anchor の X/Y 分割まで数えるとこれより多くなるが、判定の単位としては
「1 property = 1 種」で数えるのが `PropertyId`/`Intent` の粒度と一致する)。

**K の一覧(成果物、13件)**:

1. Hidden(`LayerAttrs.hidden`、ATTRS/ident帯)
2. Blend(`LayerAttrs.blend_mode`、ATTRS)
3. Speed(`LayerTiming.speed`、ATTRS)
4. Matte Mode(`Matte.mode`、ATTRS matte行)
5. Mask Mode(`Mask.mode`、MASK section)
6. Mask Inverted(`Mask.inverted`、MASK section)
7. Effect Enabled/Bypass(`EffectInstance.enabled`、EFFECTS section)
8. Font Size(`TextDocumentStyle.size`、TEXT section)
9. Line Height(`TextDocumentStyle.line_height`、TEXT section)
10. Tracking(`TextDocumentStyle.tracking`、TEXT section)
11. Justify(`TextDocument.justify`、TEXT section)
12. Fill Color(`TextDocumentStyle.fill`、TEXT section)
13. Stroke Color(`TextDocumentStyle.stroke_color`、TEXT section)

## 3. identity と判定した物とその根拠

- **Name**(`LayerAttrs.name`): 裁定214 本文が「名前・id・フォントの実体 path は identity」と
  明示。
- **Font Family + path**(`FontRef`): 同上、裁定214 が名指し。
- **Mask id**(`MaskId`): どの mask かを指す識別子。`LayerId` と同格 — 時間で「変わる」種類の
  値ではなく、行そのものの存在を指すラベル。
- **Effect 名(plugin 表示名)**: `plugin_display_name(&effect.plugin_id)` から導出 —
  どの plugin のインスタンスかという識別で、フォント path と同型(「どの実体を使っているか」
  であって「今どんな値か」ではない)。
- **Kind ラベル**(`LayerSource` variant): solid/media/text/… という層の種別そのもの。
  読み取り専用で書き口が無く、時間で変わる性質のものでもない。

## 4. 未決の物(4件、identity へ寄せずに未決のまま挙げる)

1. **Label Color**(`LayerAttrs.label_color: Option<u8>`、ident帯の色チップ)。
   Ableton のトラック色は organizational tag に近く「今どんな値か」より「どのグループに
   属するか」の性格が強い一方、`Value::Enum(i64)` へそのまま乗る型ではあるので技術的な
   障壁は無い。名前・id・フォント path のどれとも一致しないため、identity 側へ寄せずに
   未決とした。
2. **Matte 元**(`Matte.layer: LayerId`、ATTRS matte行)。「どの層をマットにするか」という
   配線(structural wiring)で、Ableton の automation という比喩よりも「どのケーブルを
   繋ぐか」という配線選択に近い。裁定214 の例示(名前・id・フォント path)のどれにも
   直接一致しない — id という語に近いが、`Matte.layer` 自身は「このレイヤーの識別子」
   ではなく「別レイヤーへの参照」なので、裁定214 の「id は identity」がこの意味の id
   まで含むかは本文からは読み取れない。
3. **Link 元**(`PropertySource::Link` の `source_layer`+`source_property`、LINK section)。
   Matte 元と同型の配線選択。`PropertySource` 自体が `Track`(値の時間変化)と
   `Link`(参照)の二択であり、「どちらを選ぶか」自体は `KeyframeTrack` の中身ではない
   — この選択自体を時間で切り替える(t<5s は Link、t>=5s は Track、のような)機構は
   現行モデルに無い。
4. **Text Content**(`TextDocument::content: ContentTrack`、TEXT section)。**この4件の
   中で最も強く「乗せるべき」寄りの未決**: `String` は `Value` に無い型だが、`content` は
   `TextDocumentStyle` とは別に**独自の `ContentTrack`(`Vec<ContentKeyframe { t, content:
   String }>`、時刻昇順を型で保証)を既に持っている** — つまり「時間評価できる
   String」という仕組み自体は store に実在する。歌詞動画のペルソナ(2026-08-22 発注
   「歌詞が入れられる道を通す」の根拠になった `docs/reviews/2026-08-22-persona-lyric-mv.md`)
   が「文字が時間で変わる」ことを明示的に要求しているので、これは名前やフォント path
   のような「時間で変わらない識別子」とは性格が逆 — **property 側に倒れる根拠の方が強い**。
   ただし Inspector の Key 列(3状態 oracle・`toggled_key_track`)には Content 用の
   `KeyRow` 変種が無く、`text.rs::commit_text_field` は常に単一の Hold キー(t=0)で
   丸ごと差し替えるだけ(`applied_text_content` の doc「アニメーション(複数キーは作らない)」)
   — 器はあるが Inspector からは使えない状態。identity と言い切れないので K へ
   寄せずに未決とした(K に含めると「型が無いのに乗せろ」という誤読を招くため)。

## 5. FINDING(判定に添える追加所見、K/未決の数には数えていない)

**FINDING 1 — Position Z の Key 列欠落**: `property::POSITION_Z` は実在の property で
`TransformField::PositionZ`/値セル/drag-to-scrub はすべて動くが、`KeyRow::Position` は
`property::POSITION`(Vec2、X/Y)だけを対象にしており Z を含まない(`transform.rs` の
コメントが「1 click = 1 `SetTrack` = 1 undo を保つため」と明記した仕様)。結果として、
Z は `edited_value_track` が「既に実キーが有る時だけ upsert する」経路しか無く、**Inspector
だけでは Z の最初のキーを打つ手段が無い**(Key glyph が無いので `toggled_key_track` を
一度も起動できない)。これは identity/property の境界問題ではなく実装の wiring 漏れ
— 裁定214 の対象(境界)には入らないが、K の実務的な隣接問題として記録する。

**FINDING 2 — 「レイヤーを指す」3種(Label Color を除く)の未決は同型**: Matte 元・
Link 元は共に「別レイヤー(または別 property)への参照」であり、裁定214 の例示に無い
第4のカテゴリ(structural wiring)を形成している可能性がある。この監査はこの3件を
無理に identity か property かへ寄せなかった — 発注書の指示どおり「言い切れない物は
未決」に従った。

**FINDING 3 — 発注書に無かった2 section を追加監査**: `link.rs`/`matte.rs` は
2026-08-22 発注「レイヤーを指す」文法で、本発注書(裁定214棚卸し)より前日に着地した
Inspector section だが、発注書の対象ファイル一覧(9本)には含まれていなかった。
裁定214 の対象は「Inspector に映る物」全部なので、この2本を含めないと棚卸しが
不完全になる — 発注書の書き漏れと判断し、追加で監査した(write-set はそのまま
docs/reviews と check.sh のみ、コード非改変なので write-set 逸脱ではない)。

**FINDING 4 — 裁定92 と裁定214 の衝突**: 裁定92「v1 では `TextDocumentStyle` を
キーフレーム化しない」は、Size/Line Height/Tracking/Fill/Stroke の5欄を明示的に
時間軸から除外した決定。裁定214「Inspector に映る物は全て時間軸で評価できる
(identity 以外)」はこれらの欄をどれも identity と認めていない(フォント size が
「どの実体か」を表す識別子ではないことは明らか)。**この監査はどちらが優先するかを
決めない** — 裁定214 が裁定92 を上書きする意図だったかは利用者裁定でしか判断できない
論点として提示するに留める。K の一覧にはこの5件を含めた(裁定214 の文面だけを機械的に
適用した結果であり、裁定92 の意図を否定する主張ではない)。

## 6. check.sh への機械化(裁定212 と同じ形式で試みた・足した)

`next/check.sh` に情報表示のみの新節「Inspector 時間軸監査」を追加した(fail させない —
既存の Lottie coverage 節・Intent 到達可能性節と同じ流儀)。

**足した部分**: 本ファイルの §2(数の表)と §1 の K 一覧が**表記としてズレていないか**の
自己整合性検査 — 見出し行に埋め込んだ数(`K(乗せるべきなのに乗っていない): 13件`)と
実際の箇条書き行数が一致するかを grep で数える。ドキュメントを更新した時に
「本文は直したが見出しの数を直し忘れる」事故(この種の台帳でよく起きる — 既存の
`intent-bundles.tsv`/`normal-map.tsv` の size 列不一致検査と同じ動機)を拾う。

**足さなかった部分とその理由**: 「Inspector が何の欄を描いているか」を静的に
列挙する検査は**足していない**。理由:

1. `Intent` 到達可能性節が使える理由は、`Intent` が`pub enum Intent { .. }` という
   **単一の enum**として `document.rs` に定義されており、awk で機械的に全変種を
   列挙できるからである。Inspector の「欄」はそのような単一の列挙点を持たない —
   `view()` を呼ぶたびに各 section の関数(`transform_row`/`mask_ident_row`/
   `text_field_row`/`color_row`/`matte_row`/`link_row`/…)が個別に組み立てる散在した
   構造で、「これが全欄である」という単一の情報源が無い。
2. 仮に欄を列挙できても、identity/property の判定は **裁定214 の文面が要求する
   意味論的判断**(「名前やフォント path のように時間で変わらない性質か」)であって、
   Rust の型シグネチャからは機械的に導出できない。本監査で最も紛糾した4件
   (§4 未決)は全て「型だけでは判定できない」ケースであり、これは今後も人間の
   判断が要る領域として残る。
3. 唯一 grep で機械的に裏取りできる部分(「この property は `PropertyId`/`SetTrack`
   を経由するか」)は既に §0 の方法でこの監査自身が行った。これを check.sh の
   恒久検査にする価値はある(K の一覧に挙げた13件が今後も track を持たないままで
   あることを継続的に確認する)が、**確認対象(K の13件)自体が固定リストであり、
   リストの正当性(なぜこの13件が K なのか)は裁定214 の意味論から来る**ため、
   機械検査は「このリストが陳腐化していないか」の弱い drift 検出にしかならない
   (裁定209「柵は明示マーカーの要求にする」の精神に照らすと、13件それぞれに
   コード側の明示マーカー — 例えば `//! K214: hidden はまだ track を持たない` の
   ような doc comment — を要求する方が誤爆せず理由も強制できる設計だが、
   これは write-set 外のコード変更になるため本発注では行わず、次発注への
   提案として §7 に記す)。

## 7. 次発注への提案(write-set 外、提案のみ)

- K の13件それぞれへ`//! K214:` のようなコード側の明示マーカーを足し、check.sh が
  そのマーカーの生死(該当フィールドがまだ track を持たないままか)を機械検査する
  形にすると、裁定209 の「明示マーカー」流儀に合う(本発注の write-set 外のため
  実施せず、提案のみ)。
- Position Z(FINDING 1)の Key 列欠落は境界問題ではなく実装 wiring の抜けなので、
  裁定214 とは別に独立した issue として起票するのが筋。
- 裁定92 と裁定214 の衝突(FINDING 4)は利用者裁定が要る — この監査では判断しない。
