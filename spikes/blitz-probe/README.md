# blitz-probe — Blitz / dioxus-native 採否プローブ

UI基盤候補としての Blitz(dioxus-native) を、**判断に効く点だけ**実測する。
`spikes/` 完結。製品 `Document`/schema には触れない。

**状態: 実測完了 2026-08-15（開発主機 macOS / Metal）**

## 判定結果

| # | 項目 | 結果 |
|---|---|---|
| P2 | 日本語 IME 4項目 | **合格**（利用者審判、2026-08-15） |
| P3 | Timeline形状DOMの毎フレーム更新(dioxus-native経由) | 天井 約1,500〜3,000ノード。**採用構成の値ではない** |
| P4 | 自前wgpu29テクスチャへの描画(ヘッドレス) | **PASS(ピクセル完全一致)** |
| P5 | clip/trim/key/playhead の掴み | **合格**(利用者審判) |
| P6 | 自前ルーティングのイベント→DOM→絵の変化 | **PASS** |
| P7 | **提案構成の実走(eframe host + Blitzテクスチャ)と上限** | **PASS。天井 約3,600ノード** |
| P8 | custom widget で密な面を1ノードに | **PASS。resolveが消える。天井は約20,000プリミティブへ** |
| P9 | 差分更新は再パースを置き換えられるか | **PASS。32.25ms → 8.31ms** |

### P2

判定基準は `spikes/ime-acceptance` の4項目をそのまま流用した(preedit下線 / 候補追従 /
Enter未食い / 長文連続入力)。**Blitz は IME を通した。**
egui 側は同条件未測定のため、この項目で Blitz が不利ということはない。

### P3（release ビルド、自動駆動 300フレーム、playhead と zoom を毎フレーム変更）

| ノード数 | p50 | p95 | 判定 |
|---|---|---|---|
| 424 | 16.63ms | 17.41ms | 60fps |
| 808 | 16.63ms | 16.89ms | 60fps |
| 1,576 | 16.66ms | 17.36ms | 60fps（限界付近） |
| 3,112 | 2.07ms | 18.79ms | **異常値。信用しない** |
| 6,184 | 24.31ms | 35.39ms | 約41fps。破綻 |

p50 が 16.6ms に張り付く区間は vsync 律速であり、余裕の量は本測定では分からない。
3,112 の行は「再レンダーは走ったが描画が伴っていない」種類の値で、本測定法では説明できない。

**製品現行capとの関係**: `ui-friction-ledger` F13 の現行capは 16 layer / 64 key。
これを埋めると概算 1,400〜1,500ノードで、**上表の限界付近にちょうど載る**。
つまり DOM Timeline は「今のcapなら 60fps、ただし余裕ゼロ」であり、
cap を上げる(F13の解消)と破綻する。**virtualization(可視域だけ描画)が前提になる。**

## 併せて確定した事実

- **wgpu の版**: `dioxus-native 0.7.10` は wgpu 26.0.1、`0.8.0-alpha.1` は **29.0.4**。
  Motolii本体 / egui(eframe 0.35) / Rerun fork はいずれも 29.0.4 なので、**0.8系のみ型が繋がる**。
- **カスタム描画APIは alpha 間で動いている**: `use_wgpu` / `CustomPaintSource` /
  `CustomPaintCtx` / `TextureHandle` は 0.7.10 のルートに在るが、0.8.0-alpha.1 のルートには無い。
- **paint callback 内で wgpu queue に触れてはいけない**: `render()` の中で
  `queue.write_texture` を呼ぶと `Encoder is invalid` で落ちる(実測)。
  Rerun を載せる場合、render pass は callback の外で回し、callback は出来上がった
  Texture を返すだけにする必要がある。
- **サムネイルに `register_texture` は不要**: `blitz-net` があるので PNG/JPEG は
  通常の `<img>` 経路で出る。生GPU出力が要るのは Stage / Timeline canvas だけ。
- **ライセンス**: Blitz本体 Apache-2.0 OR MIT、CLAなし。`stylo_taffy` のみ MPL-2.0 が加わるが
  ファイル単位のコピーレフトで製品全体には伝播しない。**fork可能。**

## 未解決

- `0.8.0-alpha.1` は alpha。API が動いている最中。
- ドッキング(panel分割/tab/resize、`ui-interaction-language.md:75` の製品要件)は Blitz に無い。
  `egui_tiles`(rerun-io、MIT OR Apache-2.0) はツリーとD&D状態機械がツールキット非依存なので、
  描画部分を差し替えれば移植できる見込み。**未検証。**
- P1(自前 wgpu::Texture を DOM へ)は 0.7.10 で API 確認まで。0.8 系での実走は未了。

## 実行

```bash
cargo run --release                                   # 既定(424ノード)
BLITZ_PROBE_SCALE=4 cargo run --release               # ノード数を増やす
BLITZ_PROBE_SCALE=4 BLITZ_PROBE_FRAMES=300 cargo run --release
```

`P3 RESULT:` 行が stderr に出る。P2 は GUI で人手審判。
自動駆動は `BLITZ_PROBE_FRAMES` 回で止まるので、**途中でズームが止まるのは仕様。**


## P4〜P7(2026-08-15追記)

詳細と数値は [reviews/2026-08-15-blitz-ui-runtime-probe.md](../../docs/reviews/2026-08-15-blitz-ui-runtime-probe.md) が正本。

```bash
cargo run --release --bin offscreen       # P4 ヘッドレス。テクスチャへ描けるか
cargo run --release --bin texture_mode    # P6 イベント注入→DOM→絵の変化
cargo run --release --bin texture_host    # P7 提案構成そのもの(窓が開く)
cargo run --release --bin timeline_ux     # P5 dioxus-native窓モードの手触り

BLITZ_PROBE_SCALE=16 BLITZ_PROBE_AUTO=300 cargo run --release --bin texture_host   # P7 上限掃引
```

### 採用構成での上限(P7、p50)

| ノード数 | resolve | render | total |
|---|---|---|---|
| 1,320 | 4.65ms | 1.68ms | 6.40ms |
| 2,600 | 9.22ms | 1.90ms | 11.18ms |
| 5,160 | 19.56ms | 2.03ms | 21.66ms |

**60fps天井 約3,600ノード。1ノードあたり resolve 約4.0µs の線形。**
`render`(テクスチャ経路)はノード50倍でも1.25→2.48msでほぼ横ばい。

### 窓モード(dioxus-native)で見つかった制約

- **キーイベントはフォーム要素にしか届かない**(`div`+`tabindex`では発火しない)
- **トラックパッドのピンチが届かない**(winitの`PinchGesture`をDOMイベントへ変換していない)
- `pointer-events: none` は効く

**いずれもテクスチャモードには当てはまらない** — イベントの配り先を決めるのがMotolii側になるため(P6で実証)。


## P8 / P9(2026-08-15追記)

```bash
BLITZ_PROBE_ITEMS=5000 cargo run --release --bin custom_widget    # P8
BLITZ_PROBE_CLIPS=900  cargo run --release --bin diff_update      # P9
BLITZ_PROBE_WIDGET=1 BLITZ_PROBE_TRACKS=14 cargo run --release --bin texture_host  # P8を目視
```

### 罠(必読)

**`BaseDocument::set_style_property` は属性を設定するが再レイアウトを起こさない。**
これで性能を測ると「何もしていない速さ」が出る。正しくは
`doc.mutate().set_attribute(id, qual_name!("style"), ..)`。
`diff_update` には `P9 SANITY` としてピクセル健全性検査を入れてあるので、
**性能を測るときは必ず併せて見ること**。
