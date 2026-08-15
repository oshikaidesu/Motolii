# Blitz(HTML/CSS)をUI基盤候補として実測したプローブ

ステータス: **比較中**。**裁定は未了**。本文書は実測記録と提案であり、[レビュー規律](README.md)の1に従い、
**この結論をそのまま設計根拠にしない**。反対側レビュー未実施、egui側の同条件測定も未実施。

- 実測日: 2026-08-15（開発主機 macOS / Metal）
- プローブ: [`spikes/blitz-probe/`](../../spikes/blitz-probe/)
- 対象: `dioxus-native 0.8.0-alpha.1` / `blitz-* 0.3.0-beta.1` / `vello_hybrid 0.0.9` / `wgpu 29.0.4`

## なぜ測ったか

現行UI基盤は[React Native + rust-skia + wgpu](2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md)（2026-08-07再基線）だが、
実測すると **RN↔Rust の橋が `rn_product_host` 実装8,247行 + テスト7,273行、`host_bridge` 4,488行**に達し、
直近60コミットの**36件(60%)が所有権・世代・生存期間の同期バグ**だった。
Rerunに乗っている部分は `rerun_stage/` **1,600行のみ**で、`crates/` 18クレートに `re_*` 依存は**0件**。

「HTML/CSSでパネルとTimelineを書き、出力をテクスチャとして受け取って Rerun と合成する」構成が
成立するかを、机上でなく実物で確かめる目的で測った。

## 結果

| # | 検証 | 結果 |
|---|---|---|
| P2 | 日本語IME 4項目（[`spikes/ime-acceptance`](../../spikes/ime-acceptance/)の基準を流用） | **合格**（利用者審判） |
| P3 | Timeline形状DOMの毎フレーム更新 | 天井 約1,500〜3,000ノード |
| P4 | **自前wgpu29デバイスのテクスチャへ描画** | **PASS（ピクセル完全一致）** |
| P5 | clip/trim/key/playhead の掴み | **合格**（利用者審判） |
| P6 | **テクスチャレンダラモードの端から端まで** | **PASS** |
| P7 | **提案構成の実走(eframe host + Blitzテクスチャ)と上限測定** | **PASS。天井 約3,600ノード** |
| P8 | custom widget で密な面を1ノードにできるか | **PASS。resolve が消える** |
| P9 | 差分更新は HTML 再パースを置き換えられるか | **PASS。32.25ms → 8.31ms** |
| P10 | 現行Timeline UI の再現 | **合格**（利用者審判） |
| P11 | Browserパネル（外部フォルダ参照+サムネイル+D&D） | **条件付きPASS。ただし下記P11訂正を必ず読むこと**（panicしないが**全部は映らない**） |
| P12 | **透過合成（Stageの上へ重ねる）** | **PASS（全320×200pxで最大誤差0）** |

### P4 — 要石

`vello_hybrid::Renderer::render(scene, resources, device, queue, encoder, render_size, view: &TextureView, texture_bindings)`
に、Motolii側が作った `wgpu::Texture` の view を渡して成立した。窓もdioxusも介さないヘッドレス実行。

```
P4/1 自前デバイス取得: backend=Metal
P4/5 bg   = [36, 36, 36, 255]     期待 36,36,36
P4/5 clip = [150, 170, 219, 255]  期待 150,170,219
P4/5 key  = [255, 173, 86, 255]   期待 255,173,86
P4 RESULT: PASS
```

逆方向（外部テクスチャをBlitzのSceneへ）は `anyrender::RenderContext::try_register_custom_resource`
が `Box<wgpu::Texture>` を downcast で受ける実装をコードで確認した（**実走は未了**）。

### P6 — 採用予定構成そのものの実走

窓もdioxusも使わず、`EventDriver::handle_ui_event` で**合成ポインタイベントを自前で注入**し、
DOMのスタイルが変わって**自前テクスチャの絵が実際に変わる**ところまで通した。

```
P6/1 hit(80,44)=node 7 / hit(8,8)=None      ← 自前ヒット判定が使える
P6/2 hover前 clip = [150, 170, 219, 255]
P6/3 UiEvent::PointerMove(80,44) を注入      ← ルーティングはMotolii側
P6/4 hover後 clip = [255, 173, 86, 255]      ← :hover が発火
P6 RESULT: PASS
```

`BaseDocument::hit(x, y) -> Option<HitResult>` が取れるため、
**「ポインタが今Blitzパネルの上か、Stage/Timeline canvasの上か」をMotolii側で判定して振り分けられる**。
これが下表の制約が本構成に当てはまらない根拠。

### P7 — 提案構成そのものを窓で動かし、上限を測る

`eframe(egui)` が窓と wgpu29 デバイスを持ち、Blitz は毎フレームそのデバイス上のテクスチャへ描き、
ポインタは Motolii 側が `blitz-dom` へ注入、egui がテクスチャを合成する。**Motolii本体と同じ eframe 0.35。**

利用者審判: **描画の劣化なし**（`FilterMode::Nearest` の等倍合成）、掴みと追従は良好。

横方向ズームを自動掃引して測った（release、300〜400フレーム、p50）。

| ノード数 | resolve | render | total | 判定 |
|---|---|---|---|---|
| 200 | 0.83ms | 1.25ms | **2.09ms** | 60fps |
| 680 | 2.45ms | 1.60ms | **4.10ms** | 60fps |
| 1,320 | 4.65ms | 1.68ms | **6.40ms** | 60fps |
| 2,600 | 9.22ms | 1.90ms | **11.18ms** | 60fps |
| 5,160 | 19.56ms | 2.03ms | **21.66ms** | 46fps |
| 10,280 | 41.59ms | 2.48ms | **44.07ms** | 23fps |

**60fpsの天井は約3,600ノード。** 内訳が設計上重要。

- **`render`(テクスチャ経路)はノード50倍でも1.25→2.48msとほぼ横ばい。経路そのものは安い**
- コストは全て **`resolve`(blitz-domの再スタイル+再レイアウト)** で、**1ノードあたり約4.0µs の線形**
- **P3(dioxus-native経由、天井1,500〜3,000)より速い。** DioxusのVDOM差分が無い分、
  テクスチャモードの方が有利。**P3の数値は本構成の値ではない**

#### 2000レイヤー規模(MV想定)への含意

2000レイヤー × (行+名前+clip2+key10) ≈ 28,000ノード ≈ 115ms(8〜9fps)で、**virtualization無しでは不可能**。
可視域のみ描画すれば1080p高で約35行×20ノード≈700ノード≈4msとなり、**プロジェクト規模から完全に独立する**。
本測定が与える設計制約は **「描画ノードを約3,600以下に保つ」** の一点。

#### Timelineの横ズームは transform でも Viewport.zoom でも代替できない

`Viewport.zoom` は文書全体の一様拡大で**文字と行高も伸びる**。`transform: scaleX()` は**文字を横に潰す**。
Timelineの横ズームは x位置と幅にだけ倍率を掛け、文字サイズ・行高・keyグリフは据え置くため、
**left/width の再計算(=全ノード resolve)が不可避**。上表はその条件で測っている。

### P8 — custom widget で密な面を1ノードにする(スプレッドシート型)

`blitz_dom::Widget` トレイト(`can_create_surfaces` / `handle_event` / `paint`)で、
密な面を自前描画にできる。`paint` には解決済み `ComputedStyles` が渡る。

ヘッドレス実測(1200x600、120フレーム):

| 描画アイテム | DOMノード | resolve | render | total |
|---|---|---|---|---|
| 5,000 | **6** | **0.00ms** | 2.94ms | **2.94ms** |
| 20,000 | **6** | 0.01ms | 14.61ms | 14.62ms |
| 100,000 | **6** | 0.02ms | 76.82ms | 76.84ms |

**`resolve` が消える。** 天井が「約3,600 DOMノード」から「約20,000 描画プリミティブ」へ移る。
単価は **描画プリミティブ 0.73µs/個 vs DOMノード 4.0µs/個**。

**ただし widget 内では文字を自分で描く必要がある**(`draw_glyphs` + parley)。
実務的な分担は「文字が要るものはDOM、要らないものは widget」。Timelineでは
clip(文字あり、数十)がDOM、key(文字なし、数千)が widget となり、両者が重ならない。

### P9 — 差分更新（最大の仮定の検証）

P3〜P8で最大の数字だった `rebuild`(HTML全再パース 25〜33ms)を「実装では消える」と
扱ってきたが、それは**仮定**だった。900クリップ、毎フレームのズーム(全要素が変わる最悪ケース):

| | update | resolve | render | **TOTAL** |
|---|---|---|---|---|
| REPARSE(HTML再パース) | 25.12ms | 4.88ms | 2.25ms | **32.25ms** |
| **MUTATE(差分更新)** | **0.74ms** | 5.44ms | 2.14ms | **8.31ms** |

**仮定は成立した。約4倍、60fps予算内。** `resolve` は 5.44ms と正しく発生しており、
レイアウトは再計算されている。

#### 罠: `set_style_property` はレイアウトを無効化しない

最初 `BaseDocument::set_style_property(id, "left", ..)` で測ったところ TOTAL 3.25ms と出たが、
**ピクセル健全性検査で不合格**だった(ズームを3倍にしても描画位置が変わらない)。
属性は設定されるが再レイアウトが走らないため、「何もしていない速さ」を測っていた。

正しいのは **`doc.mutate().set_attribute(id, qual_name!("style"), ..)`**。
数値だけ見て健全性検査を入れなければ、誤った10倍という結論を記録するところだった。

### P3 — 天井と、transform予測の外れ

release、自動駆動300フレーム、playheadとzoomを毎フレーム変更。

| ノード数 | `left`再計算 p50 | container transform p50 |
|---|---|---|
| 1,576 | 16.64ms | 16.67ms |
| 6,184 | 24.14ms | 21.46ms |
| 12,328 | 43.97ms | 37.65ms |

（**注: P3は dioxus-native 経由の測定であり、採用予定のテクスチャモードの値ではない。P7を見ること。**）

**transform化の効果は11〜14%に留まった**（当初「全ノード再レイアウトがほぼ消える」と見積もったが外れ）。
レイアウトではなく**VDOM差分がO(ノード数)で残る**ためと考えられる。CSSではなくDioxus側のメモ化が要る。
ただし virtualization（可視域のみ描画）を入れればノード数がビューポート面積で決まるため、この論点は消える。

現行cap（[friction ledger](../ui-friction-ledger.md) F13: 16 layer / 64 key）を埋めると概算1,400〜1,500ノードで、
**今のcapなら60fps・余裕ゼロ**。capを上げるなら virtualization が前提。

## 確認できた制約

**いずれも「Blitzに窓を持たせる」モード固有であり、オフスクリーン方式には当てはまらない。**

| 制約 | 内容 | オフスクリーン方式での扱い |
|---|---|---|
| **キーはフォーム要素にしか届かない** | `div` に `tabindex` を付けても `onkeydown` が発火しない。`<input>` では発火する（利用者確認） | winitイベントを自分でルーティングするので配り先はMotolii側が決める |
| **トラックパッドのピンチが届かない** | macOSではwinitが専用の `PinchGesture` として渡す。ブラウザは `ctrl+wheel` へ合成するがBlitzはしない | `PinchGesture` を拾って `handle_event` へ流す |
| ホイールに修飾キーが乗らない疑い | 未確定（キーイベントが届かないため切り分け途中） | 同上 |

**P6により、この「オフスクリーン方式での扱い」列は推測ではなく実証になった。**
イベント注入とヒット判定がMotolii側で成立することを実走で確認しているため、
配り先の決定権はMotoliiにある。

`pointer-events: none` は**効く**（子要素をイベント透過にして自前hit判定に切り替えた修正が機能した）。

## 版の罠（後続が最も間違えやすい点）

```
wgpu 29.0.4  ← Motolii本体・egui(eframe 0.35)・Rerun fork と一致
  └─ vello_hybrid 0.0.9 / anyrender_vello_hybrid 0.8.0 / dioxus-native 0.8.0-alpha.1

wgpu 26.0.1  ← Motoliiと型が繋がらない。使わない
  └─ vello 0.6 / anyrender_vello 0.6.2 / dioxus-native 0.7.10
```

`anyrender_vello`(classic) 側にある **`ImageRenderer` / `CustomPaintSource` / `TextureHandle` / `use_wgpu` は
hybrid側に存在しない**。この4つを使う実装例・記事は全部 wgpu 26 系であり、そのまま持ち込むと型が繋がらない。
hybrid側の等価物は `Renderer::render(.., view, ..)` と `RenderContext::try_register_custom_resource`。

## 統合コストの実測根拠

Bevy↔Blitz の前例が2つある。

- [rectalogic/bevy_blitz](https://github.com/rectalogic/bevy_blitz) — src 557行。**表示専用（入力転送なし）**。描画プラグインは95行
- DioxusLabs/dioxus PR #4427（merged 2025-07-22, [discussion #4362](https://github.com/DioxusLabs/dioxus/discussions/4362)）
  — `examples/native-headless-in-bevy/`。`dioxus_in_bevy_plugin.rs` **1,031行**だが、
  うち約500行は Bevy→Blitz のキーコード対応表。**実質的な統合ロジックは300〜400行**

入力転送のAPIは `dioxus_doc.handle_event(UiEvent::MouseDown(..))` と `dioxus_doc.hit(x, y) -> node_id`。
Motoliiはwinitを使うため、Bevyで必要だったキーコード変換表は `blitz-shell` の既存実装を流用できる見込み（**未検証**）。

### P11 — Browserパネル（完成条件を塞ぐ `N-MEDIA-PICK` への代替案）

`blitz-net` は `file` スキームを `std::fs::read` で処理するため、
**`<img src="file:///…">` が実ファイルを読む**。native file dialog を待たずに
「フォルダを参照してサムネイル格子を出す」形が組める。実測でフォルダ走査45件→表示まで到達した。

**ただし `vello_hybrid` の画像アトラスに厳しい上限がある。**

| 画像 | 枚数 | 結果 |
|---|---|---|
| 1480×1400（元寸） | 4 | **`AtlasLimitReached` で panic** |
| 160×151（縮小） | 16 | OK |
| 160×151（縮小） | 24 | **`AtlasLimitReached` で panic** |

**CSSの表示サイズではなく元解像度でアトラスへ載る。** `width:124px` を指定しても
1480×1400 のまま消費する。よって Browser パネルでは：

- **表示前に縮小した実体を用意する**（`<img src="original">` は不可）
- 可視域ぶんだけ載せ、スクロールアウトしたものは解放する（virtualization が**必須**）
- panic するので、`AtlasLimitReached` を握って劣化表示へ落とす経路が要る

これは custom widget 経由（`try_register_custom_resource` で自前テクスチャ）でも
同じアトラスを使うのかは**未確認**。回避路として調べる価値がある。

なお `blitz_net::Provider` は **Tokio reactor を要求する**（無いと panic）。

### P11 — Browserパネル（完成条件を塞ぐ `N-MEDIA-PICK` への代替案）

`blitz-net` は `file` スキームを `std::fs::read` で処理するため、
**`<img src="file:///…">` が実ファイルを読む**。native file dialog を待たずに
「フォルダを参照してサムネイル格子を出す」形が組める。

実測: `docs/mocks` を走査して **1480×1400 のPNGを45枚**、サムネイル格子として表示。
掴んでドラッグも成立（ドラッグ中の絵はホスト側が描く＝P6の原則どおり）。
**自前テクスチャは不要**で、通常の `<img>` 経路で足りる。

#### 罠1: `ImageManager` の画像キャッシュはフレームを跨いで保持する

最初 `ImageManager::new(.., &mut cache)` の `cache` を**毎フレーム新規に作っていた**ため、
毎フレーム全画像が atlas へ再確保され、数秒で `AtlasLimitReached` により **panic** した。
「vello_hybrid の画像アトラスが小さい」という誤った結論を出すところだった。

`cache: FxHashMap<u64, ImageId>` は画像ハッシュ→ImageIdの対応表であり、**アプリ側で保持する**。
`texture_bindings` も同様。保持すれば元寸45枚が通る。

なお `image_cache.allocate(..).unwrap()`（`vello_hybrid/src/render/wgpu.rs:596`）は
**ライブラリ内の`.unwrap()`なので捕捉できない**。atlas を溢れさせない側の責任になる。
既定の `AtlasConfig` は 4096×4096 × 最大8面、`Renderer::new_with` で変更可能。

#### 罠2: `blitz_net::Provider` は Tokio reactor を要求する

無いと `there is no reactor running` で panic する。実装でもファイル/ネットワーク取得を
使うなら runtime を張る必要がある。

## なぜ現行実装は Skia なのか、Blitz はその器になり得るか

### 採択の経緯(台帳166行目、[Skia REJECT→ADOPT裁定](2026-08-08-skia-reject-to-adopt-authority-reconciliation.md))

| 日付 | 出来事 |
|---|---|
| 2026-07-21 | Skia = **`REJECT`**。理由「**既存wgpu/Velloと重複する** renderer、cache、alpha、backend lifetime を持ち込む」 |
| 2026-07-27 | `U3a-2A` が REJECT を維持 |
| 2026-08-07 | RN/rust-skia再基線が**旧REJECTを引用も撤回もせず** rust-skia を標準に定めた(衝突) |
| 2026-08-08 | 裁定: 2026-08-07を正とし旧REJECTを**撤回**。実質的根拠は「**『Velloと重複』という前提が、同じ再基線がVelloを製品標準から外したことで消滅した**」 |

**Skiaが勝った理由は機能ではなく、Velloが降りたこと。** そして **Blitz は Vello である**。
Blitzを採ると Vello が製品標準へ戻るため、旧`REJECT`の前提が復活し、今度は Skia の側が重複になる。

`N-OVERLAY依存ゲート`(2026-08-08)は交換を想定した EXIT 条項を既に書いている:
> `EXIT`は Motolii fixture を skia非依存に保ち、交換時は **overlay描画層のみ**

### 実装が実際に使っている Skia API の全量

`ui/motolii-rn/native-renderer/src/timeline_skia/` の実測:

```
draw_rect 12 / draw_path 8 / draw_line 3 / draw_circle 3
save,restore 2 / scale 1 / clip_rect 1 / draw_str 1
型: Canvas, Typeface, EncodedImageFormat::PNG(fixture出力)
```

**シェーダ、イメージフィルタ、パスエフェクト、SkSL、ブレンドモード、カラーフィルタは1つも使っていない。**
すべて Vello/anyrender に直接の対応がある。唯一の実作業は `draw_str` → `draw_glyphs` + parley。

行数の内訳も交換前提の形になっている:

| 層 | 行数 | 交換時 |
|---|---|---|
| 描画(`draw.rs` / `paint.rs` / `scene.rs`) | 約1,200 | 差し替え対象 |
| 論理(`hit.rs` / `geometry.rs` / `layout.rs` / `session.rs`) | 約1,570 | **renderer非依存。そのまま使える** |
| テスト | 約2,600 | fixture の PNG 出力のみ skia非依存化が要る |

**結論(裁定待ち)**: Blitz は器になり得る。移植面は薄く、EXIT条項の想定通り描画層に限定される。

## ライセンス

Blitz本体 **Apache-2.0 OR MIT、CLAなし**。`stylo_taffy` のみ MPL-2.0 が加わるが
**ファイル単位のコピーレフトで製品全体には伝播しない**。fork可能。

## 確立していないこと

規律3に従い、以下は「仮説と整合する事例」に留める。

- **反対側レビュー未実施**
- **egui側のIMEと手触りを同条件で測っていない** — Blitzだけ測った状態であり、比較として片手落ち
- 透過合成（Stageの上へ重ねる）は未検証。classic vello の `base_color: TRANSPARENT` とhybridの経路は異なる
- P6はキーイベントの注入までは通していない（ポインタのみ）。ただしショートカットは
  blitz-domへ渡さずMotolii側の既存keymapで処理する想定のため、経路として必須ではない
- ドッキング（[`ui-interaction-language.md`](../ui-interaction-language.md) の製品要件）はBlitzに無い。
  `egui_tiles`(rerun-io, MIT OR Apache-2.0) のツリーとD&D状態機械はツールキット非依存に見えるが**移植可能性は未検証**
- `dioxus-native 0.8` は alpha。0.7→0.8 でカスタム描画APIが移動しており、APIは動いている最中

## P11 訂正 — 「元寸45枚が通る」は**全部映る意味では通っていない**（2026-08-15、C6実施時に判明）

C6実装のPOSITIVE ORACLEを実走させて判明。**再現も独立に取れている**。

```
items=45 frames=1187 atlas_images=30 elapsed=20.0s   ← panicなし。だが 30/45
items=20 frames=...  atlas_images=20                 ← 対照。20枚なら全部載る
```

**45枚のうち15枚はアトラスに載らず、カードが空のまま描かれます。**
panicしないので「PASS」と読めてしまうのが罠です。

shelf packing の算数と一致します。

```
4096 / 1480 = 2列,  4096 / 1400 = 2段  →  1面4枚 × 8面 = 32枚が天井（実効30）
```

**面積では収まるのに枚数で頭打ち**になります（93M / 134M px）。
`width:124px` を指定しても元解像度で載るため（本書の既出項目）、CSSでは回避できません。

**したがって Browser は現状 30枚で頭打ちであり、このままでは製品として使えません。**
回避には virtualization（可視域だけ載せて解放する）が要りますが、これは
「スクロールとは何か」というUI文法の決定を含むため未着手。**次の一粒の候補**。

> 元のP11記録（「元寸1480×1400×45枚 PASS」）は、panicしない事実としては正しく、
> **全項目が表示される主張としては誤り**でした。実測PASSの語が何を指すかを取り違えた例として残します。

## P12 — 透過合成（2026-08-15追測。採択時の未了4）

`spikes/blitz-probe/src/bin/alpha_composite.rs`。macOS / Metal / wgpu 29.0.4 / `vello_hybrid 0.0.9`。

```bash
cd spikes/blitz-probe && cargo run --release --bin alpha_composite
```

| 問い | 結果 |
|---|---|
| アルファを保持するか | **PASS。出力はプリマルチプライド済み** |
| 別の絵の上へ重ねて期待どおりか | **PASS。全320×200pxで最大誤差 0** |
| 一部だけ透過（パネル不透明・間は完全透過） | **PASS** |

`rgba(255,0,0,0.5)` が `[128,0,0,128]` で出る（straightなら R=255）。
コード側も一致 — `vello_hybrid/src/render/wgpu.rs:1370` が `BlendState::PREMULTIPLIED_ALPHA_BLENDING` 固定、
クリアは `LoadOp::Clear(Color::TRANSPARENT)`。
角丸AAの中間α画素168個を検査し、**下地より暗くなる画素は0**（縁が暗くなる現象は出ていない）。

**判別力の確認**: 誤設定 `(SrcAlpha, OneMinusSrcAlpha)` を対照として同時に測り、
最大誤差64 / 16,296px が外れることを確認済み。「何を書いても通る試験」にはなっていない。

### P12で出た「効いているつもりで効いていない」2件

**(a) 合成先のtexture formatをsRGBにすると壊れる。**
`Rgba8UnormSrgb` にすると最大誤差73、不透明パネルが `45→117` に浮く。
**UI・下地・合成先を `Rgba8Unorm` で揃えること。**Stage側のformatが違うとここで事故る。

**(b) `body` に背景色を置くと面全体が不透明になる。**

| 書き方 | 外周の実測値 |
|---|---|
| `html, body { background: transparent }` | `[0,0,0,0]` |
| 背景指定を一切書かない | `[0,0,0,0]`（既定が透明） |
| `body { background: rgb(24,24,24) }` だけ | `[24,24,24,255]` — **viewport全面を塗り潰す** |

`html` が透明のとき `blitz-paint` は body の背景色を拾って viewport 全面を塗る
（`blitz-paint-0.3.0-beta.1/src/render.rs:127-160`）。**パネル色は body ではなく個々の要素に置く。**

### P12の未測定（推測と実測の切り分け）

実物の Rerun Stage との合成（本プローブの下地はCPU生成テクスチャ）、窓表示・surface format
（すべてオフスクリーン読み戻し）、HDR/wide-gamut、`mix-blend-mode` 等のCSS合成モード、`hidpi_scale≠1`。

## 提案（裁定待ち）

1. ホストとcanvas層は Document と同一プロセスに置く
2. パネル層とTimelineは HTML/CSS で書き、**オフスクリーンでテクスチャへ描いて合成する**
3. 境界を「テクスチャを返す」「イベントを受け取る」の2本に絞り、**UI技術を後から差し替え可能に保つ**

3が成立すれば、ファーストパーティのパネルがサードパーティと同一機構になり、
[plugin-ui-model](../plugin-ui-model.md) が保留している custom panel 契約の実証にもなる。

## 裁定

（未記入）
