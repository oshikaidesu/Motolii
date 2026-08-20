# effect stack → engine 消費: 縫い目調査(読むだけレーン)

対象リポ: `/Users/member_ottoto/rust_ae/Motolii`(main checkout)。設計判断はしない — 事実と選択肢のみ。

---

## 1. 現状の縫い目地図

### 1a. store 読み口(実装済み・engine から未参照)

- `EffectId(u32)` / `EffectInstance{ id, plugin_id: String, enabled: bool }` —
  `next/core/motolii-store/src/effect.rs:28-52`。param 値は**持たない**(モジュール doc
  同ファイル 1-18 行)。
- 保存 component は `Layer:effects`(TrackJson、`descriptor_effects()`) —
  `next/core/motolii-store/src/components.rs:241-247`。
- 書き口: `Intent::SetEffects{ layer, effects: Vec<EffectInstance> }` —
  `next/core/motolii-store/src/document.rs:316-319`。適用は
  `document.rs:883-890`(`validate_unique_ids` → `serde_json::to_string` →
  `descriptor_effects()` へ ingest、丸ごと差し替え)。
- 読み口: `StoreView::effects(layer) -> Result<Vec<EffectInstance>, StoreError>` —
  `next/core/motolii-store/src/view.rs:537-550`。masks と同型(無ければ空)。
- param の平坦 track 名: `PropertyId::effect_param(effect: EffectId, name: &str)` —
  `next/core/motolii-store/src/document.rs:88-91`、`effect.{id}.param.{name}`
  (`property::EFFECT_PREFIX = "effect."` — `next/core/motolii-store/src/lib.rs:101`)。
  **全 param が animatable**(裁定72)なので、値は既存の汎用口
  `StoreView::value_at(layer, &property, t) -> Result<Option<Value>, StoreError>` —
  `next/core/motolii-store/src/view.rs:376-383` でそのまま引ける。`resolve()` が
  scalar/vec2 を読むのと**同じ関数**(`view.rs:705`,`724` 等で実例)。
- 型語彙: `motolii_eval::Value` の `F64`/`Bool`/`Enum`/`Color`/`Vec2`/`LayerId`
  (裁定72/78/133、`next/core/motolii-eval/src/value.rs` が正本)。
- **どの param track が存在するかを列挙する口は汎用のものが既にある**:
  `StoreView::properties(layer) -> Vec<PropertyId>` —
  `next/core/motolii-store/src/view.rs:131-150`。`all_components_for_entity` へ
  「store に聞く」(裁定57)。effect 専用の列挙 API は要らない — 名前が
  `effect.` prefix の `PropertyId` を filter すれば `(EffectId, param name)` を復元できる
  (mask が同型の prefix 走査を要求されていないのは masks が別 component
  `descriptor_masks()` に構造ごと入っているためだが、param 名の集合は plugin ごとに
  可変なのでこの汎用列挙が唯一の発見経路になる)。

### 1b. store → engine の断絶(ここが「縫い目の穴」)

- `ResolvedLayer`(`next/core/motolii-store/src/lib.rs:376-400`)は
  `source`/`placement`/`declared_size`/`source_frame`/`masks`/`blend_mode`/`matte`/`pinned`
  の8フィールドのみ。**`effects` フィールドが無い**。
- `StoreView::resolve`/`resolve_with_solo`(`view.rs:643-772`)は `self.attrs()`
  (673)・`self.composition()`(692)・`self.resolved_masks()`(767)は呼ぶが
  **`self.effects(layer)` を一度も呼ばない**。`ResolvedLayer` 構築
  (`view.rs:755-771`)に effect 関連の行が無い。
- つまり store 側は「plugin id・順序・enabled・全 param の animatable 値」を
  **完全に持っているが、resolve() の外に出ていない**。ここが縫い目の第一関節。

### 1c. engine 組み立て(effect 未着手)

- `Engine::render`(`next/engine/motolii-engine/src/lib.rs:143-243`)は
  `view.resolved_layers(t)`(160-162)から `Vec<ResolvedLayer>` を受け、各要素を
  `motolii_compositor::Layer{ texture, size, placement, pinned, blend_mode }`
  (232-239)に変換して `self.compositor.render(comp, camera, &layers)`(242)へ渡す
  1本道。**effect を挿す分岐が存在しない**。
- `EngineError` は `UnsupportedBlendMode`/`UnsupportedMatte`(35-42)の**先例パターン**
  を持つ — 「store の語彙は広い・compositor が表現できる分だけ受け、対応外は明示的に
  弾く(黙って近似しない)」という縫い目の型が **blend mode で既に1回実装済み**
  (`translate_blend_mode`, `lib.rs:377-384`)。

### 1d. compositor(GPU 実行、単一パス)

- `motolii-compositor` は `motolii-core`/`glam`/`macaw`/`re_renderer`/`wgpu` にのみ依存
  (`next/engine/motolii-compositor/Cargo.toml`)。**`motolii-store` に依存しない**。
  これは `LayerPlacement`/`CompSpec`/`ResolvedCamera` が `motolii-core` に**共有型として
  出されている**のと同じ理由(裁定33/41: 「compositor が store を引くと第二経路を
  作れる」の逆型)。→ effect の値を運ぶ型を compositor まで通すなら、
  **store 型(`EffectInstance`/`motolii_eval::Value`)をそのまま持ち込めない**制約がある
  (`motolii-eval` 自体は `motolii-core` にしか依存しない軽量 crate なので
  `Value` だけを compositor が引くことは技術的に可能 — ただし「compositor が
  Document の語彙を知らない」という現行の設計原則(compositor モジュール doc
  `next/engine/motolii-compositor/src/lib.rs:66-68`、BlendMode の節で明言)には反する)。
- `Compositor::render_with_timing`(`motolii-compositor/src/lib.rs:281-438`)は
  **単一 `ViewBuilder`/`TargetConfiguration`**(362-390)・**単一 `queue_draw`**(392)・
  **単一 `schedule_screenshot`+readback**(397,417-436)。layer 全部を1回の
  `RectangleDrawData`(297-346,350-351)にまとめて描く。中間 texture を作る機構は
  **無い**(`ctx.texture_manager_2d.create` は素材アップロード専用、
  `upload_rgba`/`upload_yuv420p` 194-265 の2口のみで、空の render target を作って
  再利用する API は今は無い)。
- `HeadlessGpu` は pub 公開されている(`pub use headless::{HeadlessError, HeadlessGpu}`,
  `lib.rs:96`)が、これは「probe が compositor の持たない描画を組み立てたい時に
  adapter/device の起こし方だけ使う」用途(doc 90-95)であって、
  `Compositor::ctx`(`RenderContext`)自体は private field(`lib.rs:161`)。
  つまり **engine 側に compositor の `RenderContext`/device への直接アクセスは無い**
  — effect の GPU pass を追加するなら、その口を `motolii-compositor` の公開 API として
  新設する必要がある(既存の `HeadlessGpu` 経由で engine が「別の」device を起こす
  選択肢は理論上あるが、それは devcie を2つ持つことになり 1d 冒頭の共有前提と食い違う)。

---

## 2. effect pass 挿入点の選択肢

### 案1(推し): compositor 内・layer ごとのオフスクリーンパスとして、`Compositor::render_with_timing` の中に増設

`Compositor::Layer`(`motolii-compositor/src/lib.rs:111-125`)に BlendMode と同型の
**compositor 自身の語彙で書ける effect 表現**を足す(例:
`effect_stack: Vec<CompositorEffect>` — `CompositorEffect` は compositor が
`motolii-core` 由来の生の値(f32/Vec2 等、`Value` を経由しない)で持つ closed enum。
BlendMode が「store の16値のうち表現できる分だけ」を持つのと同じ形)。
`render_with_timing` は最終合成(297-346 の `TexturedRect` 列)の**前**に、
effect_stack を持つ layer だけ追加の `ViewBuilder`/render-to-texture パスを回し、
結果 texture で元の `layer.texture` を差し替えてから通常の合成へ渡す。
`Engine::render` は BlendMode と同じ形の `translate_effect(EffectInstance, params)
-> Result<Option<CompositorEffect>, EngineError>` を通して store 語彙→compositor 語彙
へ変換するだけ(GPU コードは書かない、現行の役割分担を維持)。

- **裁定13(trait まだ作らない)との関係**: 満たす。plugin_id 文字列→closed enum の
  match は trait を作らずに済む(2つ目の利用者が現れるまで待つ、という裁定13の
  趣旨とも合う — 今 vism 第1号を作る段階でも trait は不要)。
- **第二 render パス禁止との関係**: 満たす。`Compositor::render`/`render_frame` の
  呼び出し回数は1のまま(裁定15/18)。増えるのは**同じ `RenderContext`・同じ
  command buffer 提出の中の追加 `ViewBuilder` パス**であり、`render_frame_without_
  background`(裁定141)が「第二経路ではなく同一合成器への入力差分」と整理した
  のと同じ論法が使える。
- **含意**: `motolii-store` 依存を compositor へ持ち込まない(1d の制約を守る)。
  ただし store 語彙(`EffectInstance`+`Value` map)→compositor 語彙(closed enum)の
  変換表を engine 側に持つ必要があり、**このマッピングの管理コスト**が
  UnsupportedBlendMode と同様に effect が増えるたびに engine 側へ蓄積する
  (現状 blend mode は14/16 が未対応のまま止まっている前例あり)。

### 案2: engine が独自の `HeadlessGpu` インスタンスで effect pass を実行

`motolii_compositor::HeadlessGpu`(既に pub、`lib.rs:96`)を engine が直接使い、
compositor とは**別の** `wgpu::Device`/`RenderContext` 相当を起こして effect パスを
実行、結果を compositor へ渡す前に texture として焼く。

- **裁定13との関係**: 影響なし(trait 不要な点は案1と同じ)。
- **第二 render パス禁止との関係**: **グレー**。`Compositor::render` 呼び出し回数は
  1のままだが、**GPU device/RenderContext が2つ**になる。裁定27
  (「別 device でも byte 一致するか未検証」)が指摘した論点がそのまま再燃する —
  「同じ入力から byte 一致する」の保証(裁定15)が2 device 構成では前例が無い。
  さらに texture は device をまたいでゼロコピー共有できない(re_renderer の
  `GpuTexture2D` は特定 device に紐づく資源)ため、**CPU 経由の再アップロード**
  (裁定44 が Stage で一度選んだ「CPU 経路」の再演)が必要になり、GPU 常駐という
  効果の利点を失う。
- **含意**: 実装量は少なく見えるが(compositor 本体を触らずに済む)、
  device 分割の実測負債(裁定27 の宿題)を新たに1つ増やす。**非推奨として記録**。

### 案3: フレーム全体へのポストプロセス(comp レベル、1回だけ)

`Compositor::render_with_timing` の最終 readback 後(または最終 `TexturedRect` 合成後)
に、**comp 全体**へ1回だけ追加の GPU パスをかける(layer 単位ではなく画面単位)。

- **裁定13/第二パス禁止との関係**: 「第二 pass」の定義次第。同一 `Compositor::render`
  呼び出しの中に留めれば裁定15/18 は形式的に満たせるが、**effect は Document の
  意味として「layer が持つ」**(裁定70 冒頭「effect は Document が型で持たない…
  Document が持つのは『どの effect を・どの順で・どの param で・有効か』」— 主語は
  常に layer)。comp レベルの適用先(adjustment layer 相当)は現行 Document モデルに
  存在しない。layer の重ね順と effect の相互作用(裁定70/72 が想定する「この layer の
  上にこの順で effect を掛ける」)を **表現できない** — layer A に blur、layer B は
  素通し、という基本形が comp レベル一括では作れない。
- **含意**: 実装コストは最小(既存の1回描画の後に1パス足すだけ)だが、
  Document の effect stack モデル(裁定70/72、layer 単位)と**構造的に合わない**。
  将来 comp 全体に効く効果(adjustment layer 的な物)を Document へ足す日が来たら
  再考の価値はあるが、**vism 第1号(layer effect の消費)の答えにはならない**。

**推し = 案1**。理由1行: BlendMode で既に実証済みの「store 語彙は広い/compositor
語彙は closed enum・engine が変換して対応外を明示的に弾く」という型をそのまま
再利用でき、第二 render パス禁止・裁定13・store⇏compositor 依存の3制約を同時に
満たす。

---

## 3. 裁定13/70/72/142 との整合(制約整理)

- **裁定13**(trait はまだ作らない・2つ目の利用者が現れるまで待つ): 3案とも
  plugin_id 文字列→closed enum の match で実装可能なので**抵触しない**。vism 第1号
  だけを通す段階では「拡張の口」自体を作らないほうが裁定13の趣旨(先に口を決めると
  中身が歪む)に忠実。
- **裁定70**(Document は effect を型で持たない・plugin id 文字列+順序+param map+
  enabled のみ): store 側は既にこの形で実装済み(effect.rs)。engine/compositor 側で
  「どの plugin_id を実装しているか」を**closed match**で持つのは、store が型を
  持たないこととは別の階層(裁定70 が縛るのは Document、engine/compositor の内部
  実装まで縛らない)。
- **裁定72**(param は名前つき map・全 param animatable・新機構ゼロ): 読み出し側は
  `value_at`+`properties()` の汎用口だけで足りる(1a 参照)。新しい store 側機構は
  **不要**。
- **裁定142**(トンマナ tokens+柵、raw 値直書き禁止): これは **UI chrome の色/寸法**
  に関する柵(`next/DECISIONS.md:149`、対象は pane コード)であり、effect の GPU
  pass(shader 定数・shader uniform)は「データ由来の値」寄りで、裁定142 の除外規定
  (「データ由来の色…ユーザ作品の内容」「製品意味の定数」)に近い性質。engine/
  compositor 側のシェーダ定数は pane コードではないため**この柵の対象外** — 衝突なし。

---

## 4. 実装切片の割り案(重み均等)

前提: 案1(compositor 内オフスクリーンパス、BlendMode と同型の変換)を土台に、
vism 第1号(Glow、`spikes/m5-known-implementation/M5-R0/src/glow.rs` を移植元)まで
通す場合の切片。**各切片の重みをおおよそ揃えた**(80〜150行・書き手の判断量を
均す)。write-set はファイル単位で互いに素。

| # | 切片 | 推定行数 | 設計判断の重さ | 跨る領域数 | write-set | 依存 |
|---|---|---|---|---|---|---|
| S1 | store: `resolve()` が effect stack を読み `ResolvedLayer` へ添える | 約80〜110行 | 中(表現型を新設する必要 — `EffectInstance`+評価済み param map を1つの型に畳む。masks の `ResolvedMask` が先例なので発明ではなく模倣) | 1(store のみ) | `next/core/motolii-store/src/view.rs`(`resolve_with_solo` 内に effect 読み出し追加、`resolved_masks` と同型の private helper 新設)、`next/core/motolii-store/src/lib.rs`(`ResolvedLayer` へフィールド追加) | 先行なし(起点) |
| S2 | compositor: `Layer` へ effect 表現を追加し、`render_with_timing` に layer 単位オフスクリーンパスの**枠**(pipeline 未定・no-op でも通る形)を挿す | 約100〜140行 | 高(第二パス禁止との整合を実装で成立させる箇所、案1の中核。中間 texture 生成・destroy タイミング・単一 `RenderContext`/command buffer 内での複数 `ViewBuilder` 呼び出し順序を決める) | 1(compositor のみ) | `next/engine/motolii-compositor/src/lib.rs` | 先行なし(S1と並行可。型は S3 で橋渡し) |
| S3 | engine: `translate_effect`(BlendMode と同型の変換関数)+ `Engine::render` の layer 組み立てへ effect_stack を渡す配線 | 約60〜90行 | 中(店語彙→compositor 語彙の対応表と、未対応 plugin_id の扱い — `UnsupportedBlendMode` 型の `EngineError` variant を新設するか、黙って通すか、の1点だけを決める) | 2(store 型を読み・compositor 型へ渡す、橋渡し役) | `next/engine/motolii-engine/src/lib.rs` | S1・S2 完了後(両者の型を橋渡しするため) |
| S4 | Glow shader/pipeline の compositor 内実装(bright-pass→2方向 blur→additive composite) | 約120〜160行 | 低〜中(数式・shader 定数は glow.rs から**そのまま移せる**、判断が要るのは fixture 依存の剥がし方のみ — 4節参照) | 1(compositor 内、S2 が空けた枠の中身) | `next/engine/motolii-compositor/src/lib.rs`(または新設 `next/engine/motolii-compositor/src/effects/glow.rs` — S2 の枠設計次第) | S2 完了後 |
| S5 | oracle/柵: golden 試験(store→engine→compositor 経由で Glow 1個だけの回帰) | 約80〜120行 | 低(既存試験の型を模倣 — `motolii-engine/tests/frame.rs` 相当のパターンを再利用するだけ、新しい検証手法は要らない) | 1(engine/compositor の test crate) | `next/engine/motolii-engine/tests/*.rs`(新規ファイル) | S1〜S4 完了後(全経路が繋がってから) |

**依存順**: S1・S2 は並行着手可(互いに書かない)。S3 は両方の型が揃ってから
(store 側の `ResolvedLayer.effects` の型と compositor 側の `Layer.effect_stack` の型を
両方見て変換関数を書くため)。S4 は S2 が用意した「枠」(pipeline 差し込み口)に
中身を入れるので S2 の後。S5 は全部が通ってから。

---

## 5. Glow proof の移植元一覧と、製品化で変えるべき点

移植元: `spikes/m5-known-implementation/M5-R0/src/glow.rs`(README:
`spikes/m5-known-implementation/M5-R0/README.md:11-13`)。

| 製品側で要る部品 | proof 内の該当箇所(file:line) |
|---|---|
| bright-pass(輝度>1.0だけ抽出、`Rgba16Float` HDR ソース前提) | `glow.rs:401-407`(`bright_fs`、luminance 係数 `0.2126/0.7152/0.0722` = Rec.709 相対輝度) |
| 半径2の分離型(横→縦)ガウシアン近似blur、5-tap `[0.0625,0.25,0.375,0.25,0.0625]` | `glow.rs:409-428`(`blur_at`/`blur_horizontal_fs`/`blur_vertical_fs`) |
| additive composite(`source.rgb+glow.rgb*0.75`、alpha は `a+b*(1-a)`) | `glow.rs:430-435`(`composite_fs`) |
| Host 所有の texture/pipeline/bind group を fixture 生成時に1度だけ作り、
  評価のたびに使い回す構成(= 毎フレーム再生成しない設計) | `glow.rs:16-36`(`GlowFixture` フィールド一覧)、`38-221`(`new()` が texture 4枚・pipeline 5本・bind group 4本を一括生成)、`224-309`(`render()` は encoder を作るだけで pipeline/bind group を再生成しない) |
| 5パスの実行順序(source→bright→blur-h→blur-v→composite)と各パスのターゲット | `glow.rs:230-264`(`draw_pass` 呼び出し5連) |
| clamp-to-edge 境界(blur が範囲外を `clamp(p±offset, lo, hi)` で読む) | `glow.rs:411-412` |
| f16 readback のデコード(`half_to_f32`) | `glow.rs:355-382`(**製品では不要** — readback は最終フレームのみ、中間 `Rgba16Float` は GPU 常駐のまま次パスへ渡すので CPU 側 half→f32 変換は proof の検証専用) |

**proof→製品化で変えるべき点(fixture 依存の剥がし)**:

1. **adapter/device の起こし方**を `GlowFixture::new()` 独自の
   `wgpu::Instance::new(...)`(`glow.rs:40-52`)から**外す** — 製品では
   `Compositor::headless()`(`motolii-compositor/src/lib.rs:168-191`)が既に持つ
   `RenderContext`/device を再利用する(裁定17: adapter 選択/limits は
   `re_renderer::device_caps` の物をそのまま使う、を2重に満たすため。proof は
   standalone なので独自に起こしているだけで、製品はここを流用しない)。
2. **ソース texture の生成元**を差し替える — proof の `source_fs`
   (`glow.rs:394-399`、画面中央に固定矩形を焼くだけ)は完全にテスト用のダミー。
   製品では「effect が付いた layer 自身の描画結果」(=案1の「layer をまずオフス
   クリーンへ描く」ステップの出力 texture)を source として使う。
3. **固定サイズ**(`WIDTH=32, HEIGHT=32`、`lib.rs:9-10`)を comp 解像度(可変)へ
   一般化する — texture 生成・bind group・readback バッファのサイズ計算
   (`glow.rs:54-71`,`194-199`)を comp の `width`/`height` から動的に作る必要がある。
4. **readback(CPU 転送)を製品パスから外す** — proof は毎回3枚(source/bloom/output)
   を CPU へ読み戻して assert している(`glow.rs:265-309`)が、製品では最終
   `output` texture を**次の合成ステップ(通常の `TexturedRect` 経由の合成)へ
   GPU 上のまま**渡す。readback が要るのは export 時の最終フレーム1回だけ(既存の
   `Compositor::render_with_timing` の readback、`lib.rs:424-437`)。
5. **param のハードコード値を animatable にする** — proof の `bright_fs` の閾値
   `1.0`(`glow.rs:405`)・blur 半径2(tap 数固定)・composite の `0.75`
   (`glow.rs:433`)は、製品では `effect.{id}.param.{name}` 経由で読んだ
   `Value::F64` を uniform buffer へ渡す形に変える(裁定72 が既に「全 param
   animatable」を保証しているので、値の受け渡し経路自体は S1〜S3 が作る)。
   blur 半径を可変にする場合は tap 数固定の5-tapを崩さず「半径→sigma」の
   マッピングだけ param 化するか、これも既定の5-tapで固定と扱うかは未決
   (proof は半径2固定、S4 の設計判断)。
6. **透明 clear(`wgpu::Color::TRANSPARENT`)を毎パス使う点**(`glow.rs:339`)は
   製品でもそのまま使える(compositor 本体も同じ規約、`compositor/src/lib.rs:401`
   の `Rgba::TRANSPARENT`)— ここは変更不要。

halftone(`next/../M5-R0/src/lib.rs:81-83,182-297,530-558`)と feedback
(`next/../M5-R0/src/feedback.rs`)も同 README に proof として存在するが、
今回の任務(Glow を vism 第1号候補として明示)では**参考として名指しのみ**にとどめる
— halftone は解像度非依存(composition 座標系で cell を固定、`lib.rs:548-551`)という
別の設計論点を持ち、feedback は Host 所有 ping-pong texture という**フレーム間状態**
(単発 render_frame の範囲を超える)を要求するため、Glow(単発フレーム内で完結する
複数パス)より1段複雑な縫い目が要る — vism 第2号以降の候補として温存が妥当。

---

## 縫い目要約(最終報告用メモ)

- 1行要約: store は effect stack(id/plugin_id/enabled/animatable param)を完全に
  持つが、`resolve()` が `effects()` を一度も呼ばず `ResolvedLayer` に運んでいない
  ため、engine/compositor は effect を一切見ていない — 縫い目は「まだ無い」状態。
- 挿入点の推し: 案1(compositor 内・layer 単位オフスクリーンパスを
  `render_with_timing` の中に増設、BlendMode 変換と同型の store→compositor 語彙
  変換を engine が持つ)。理由: 第二 render パス禁止・裁定13・「compositor は
  store に依存しない」の3制約を、既存の BlendMode 縫い目パターンの再利用だけで
  同時に満たせるため。
- 切片数と依存順: 5切片(S1 store resolve 拡張・S2 compositor 枠・S3 engine 変換
  配線・S4 Glow shader 中身・S5 golden 試験)。S1/S2 並行可 → S3(両方待ち)→ S4 →
  S5。
