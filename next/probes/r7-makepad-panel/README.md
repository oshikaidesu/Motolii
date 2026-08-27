# R7 Makepad panel probe

これは製品 front の Makepad ホストです(裁定251/252)。
意味の正本は`motolii-store` / `motolii-shell-state` / `motolii-engine`。
`motolii-shell`は凍結icedアセンブラであり、製品核ではありません(裁定253)。
旧世界のモックを正本にしません。standalone workspace なのは next/ への折り込みが未了なだけです。

## 正本

- 構造・状態・余白: `next/reference/mocks/` の現行モック
- 寸法: `next/ui/motolii-tokens-rs/tokens/dimensions.json`
- 実装面: `src/main.rs` の `script_mod!`(面の骨格)と `src/*_surface.rs`(各パネル)
- 機械可読な結線: `probe.json`

現在の面は Browser | Stage | Inspector | Timeline です。Browser の 26/30/26px、
Stage の 720x405 comp、Inspector の 300px pane と 26/28/46/21/26px、Timeline の
150px rail / 22px ruler / 18px key row は現行の密度パスです。通常レーン高は固定値ではなく、
Timelineの利用可能高から全レーン数とkey行を引いて毎フレーム算出します。
Timeline 下の Play / frame / slider は意味論モック内の導出案であり、採用しません。

## 再現

リポジトリのルートから実行します。

```bash
cargo run --locked --manifest-path next/probes/r7-makepad-panel/Cargo.toml -- --hot
```

`--hot` は makepad 本体の live reload です。`script_mod!` を持つ `src/*.rs` を保存すると、
再ビルドなしで窓が更新されます(`browser_surface.rs` / `chrome/*.rs` / `main.rs` すべて)。
自前の再読込ホストは持ちません。かつて `HotPanel` + `panel.splash` で二重に実装しており、
**どのパネルの .rs を触っても窓が空白になる**欠陥の原因になっていたため撤去しました
(裁定なし・2026-08-27 実測)。makepad は FileChange で `script_mod!` を再実行し
`Event::LiveEdit` を投げるので、面を宣言で持つ限りこれだけで足ります。Stage は
`motolii-fixture::build` と共有面（`create_presentable_texture` → import → `render_into`）へ接続します。Makepad側はDocumentを
所有せず、`TimelineSurface action → App::handle_actions → BackendBridge →
Document::apply_all / Session` へ書きます。playhead dragはSession時刻、
lane dragはDocumentの`LayerMeta.order`へ入り、StageとTimelineが同じ正本から再投影されます。
横pan/zoomだけは表示窓なのでMakepadの一時状態ですが、縦方向のscaleは持ちません。二本指の
horizontal scrollはtime pan、Option-scrollはpointer anchorを保つtime zoom、Shift-wheelは
horizontal panです。trackpadのaxisと動詞はgesture開始後に固定し、OS momentumは同じownerへ
継続、次のtouchで停止します。native gestureは`gesture_input.rs`の汎用transform sampleへ変換し、
macOS pinchはscaleだけをTimeline policyがtime zoomとして解釈します。
Browser / Inspector の操作と Export はまだ接続していません。

同期timeline操作は `Document` / `Session` へ直接書く。iced `Task` は製品 front に無い。

SVG は `resources/icons/` に集約し、`crate_resource("self://resources/icons/...")` で参照します。
面が `script_mod!` に宣言で載っているので、リソース解決は makepad の通常経路です
(かつて必要だった `ScriptMod` の手組みは `HotPanel` ごと不要になりました)。
regular / bold / code のフォント役割は `src/main.rs` の `script_mod!` に明示しています。

Makepad は [oshikaidesu/makepad](https://github.com/oshikaidesu/makepad/tree/motolii-magnify) の
git revisionへ固定しています。fork差分は意味を持たないgesture transformイベントとplatform producer
だけに限定し、Timeline固有の判断は`gesture_input.rs`より下流へ置きます。候補の依存を製品 workspace
へ混ぜないため、この probe 自体も standalone workspace のままにします。製品 front は `motolii-shell` を引きません。

## Ableton比較ループ

Abletonから借りるのはArrangementの外観ではなく、transport・時間面・色付きの対象列・
密度・選択結果の即時性です。各反映でAbleton公式資料と実窓を見比べ、`script_mod!` と
SVGだけを更新します。

- 参照: [Live](https://www.ableton.com/en/live/)、[Live Concepts](https://www.ableton.com/en/live-manual/12/live-concepts/)、[First Steps / Info View](https://www.ableton.com/en/live-manual/12/first-steps/)
- 軸: `macro_layout`, `transport_density`, `time_surface_rhythm`, `semantic_color`, `focus_and_feedback`, `copy_minimization`
- 低スペックLLMの「Abletonと判別しにくいか」は視覚の煙検知として使う。合否は実窓の操作・意味・Document接続で決める。
- 公式画面の画像はリポジトリへ複製せず、上記URLを参照元として記録する。

## タブ行(裁定265)

各パネルのタブ行は**レイアウトの高さを取りません**。セルの左上に触れた時だけ、中身の上へ
overlay で浮きます。タブは切り替えであると同時にレイアウト組み替えのハンドルなので、
消さずに浮かせています(`hide_tab_bar` は drop target から外れるので採りません)。

機構は makepad fork 側(`DockTabs{float_tab_bar: true}`)、**いつ明かすかは `main.rs` の
`reveal_tab_bars_under`**。開く引き金はセル左上の隅だけ、開いた後は帯の全幅で保持します。

fork は未 push なので、`Cargo.toml` の `[patch]` がローカル checkout を指しています。
push したら rev を上げてその節を外してください。

## アイコンの規則(裁定266)

**箱はグリフより小さくしてはならない。** 切れたアイコンは操作の意味を壊すので、
これは見た目の問題ではありません。

- アイコンを持つボタンは `padding: 0` / `margin: 0`。左右だけ 0 にすると
  `ButtonFlat` 既定の `theme.mspace_1` が上下に残り、親に潰された時にグリフが切れます
- 容器を中身と同寸にしない(`Fit` か明示の余裕)
- 踏面 24 の `ChromeButton` は行高 16 の `ChromeRow` に**入りません**。
  組み合わせられない部品を見本で組まない

正規化は **makepad 側**が持ちます(fork `4d7b6ddd0`)。SVG は中心線の境界ではなく
**インクの境界**(= 境界 + `stroke/2`)で箱にフィットします。これが無いと、はみ出す量が
アイコンごとに違うので同じ箱でも大きさが揃いません。**新しいアイコンは自動で揃います** —
1個ずつ測って正規化するのは規則ではなく作業だからです。

### 検出

```bash
python3 next/probes/r7-makepad-panel/tools/clip_oracle.py <remote-port>
```

窓の実画素だけを見ます(widget の内部を覗かない)。箱の縁にインクが乗っていたら、
そのグリフは箱の外へ続いていた、と読みます。縁の**ほぼ全部**がインクの時は
箱の境界が丸めで隣の色に読めているだけなので偽陽性として除きます。
切れている箱が1つでもあれば exit 1。

## 目盛り(全パネル共通)

寸法・字・線・面の値は `src/tokens.rs` の `mod.tokens` にだけ在ります。Tailwind と同じ考えで、
値は数直線から選ぶものであってその場で書くものではありません(`14` と `13` が並ぶ理由を
後から誰も説明できないため)。`space` / `text` / `size` / `rule` / `face` / `ink` の6族。

`mod.tokens.scale` が唯一の可変値で、寸法系は全部これに掛かります。よって **UI 全体を
1% 刻みで拡縮できます** — `Cmd -` / `Cmd +` で 1%、`Shift` 併用で 10%、`Cmd 0` で 100%。
scale は Rust の atomic に在り、`script_mod!` の式へ焼き込まれるので、変更後は
`cx.request_live_edit()` で焼き直します(makepad が iOS の safe-area inset に使う経路と同じ)。

窓の `dpi_override` でも同じ絵は作れますが、実行時に差し替えると `--remote` の grab が
Metal のアサーションで落ちます(drawable と grab テクスチャの寸法不一致、2026-08-27 実測)。
検証手段を壊さない方を採りました。

現在トークンに載っているのは `main.rs` の骨格と Browser です。Stage / Inspector / Export /
Settings / Timeline はまだ数値が直書きで、拡縮に追随しません。Dock の pane 幅は利用者が
掴んで動かす物なので、意図的に scale へ追随させていません。

## 検収

- `cargo run` が実窓を開き、Browser・Stage・Inspector・Timeline が同じ面に表示される
- Stage に fixture の `Document → Engine → re_renderer` 由来フレームが表示される
- playheadをドラッグすると`Session.playhead`とStage frameが同じframeへ更新される
- laneを上下へドラッグすると`LayerMeta.order`が1 undoで変わり、Stage重なりとlane順が一致する
- Timeline上の二本指horizontal scrollでtime panし、Option-scrollで時間軸だけがzoomして目盛り間隔が表示尺へ追随する
- 斜めtrackpad入力のaxisがgesture中に変わらず、OS momentumが次のtouchで停止する
- macOS pinchでpointer anchorを保つtime zoomが連続動作する
- Windows/Linuxはnative producer着地までAlt/Option-scroll fallbackを使う
- `src/*_surface.rs` の文字または寸法を編集し、再ビルド・再起動なしで表示が更新される
- Browser / Stage / Inspector / Timeline の操作記号が SVG アイコンで表示され、補助説明ラベルは空にできる
- 実窓の基準画像は [evidence/makepad-panel.png](evidence/makepad-panel.png)
- 密度パスの画像は [evidence/makepad-panel-iteration-02.png](evidence/makepad-panel-iteration-02.png)

利用者裁定(2026-08-26、裁定251/254): Makepadが製品 front。意味は store / session / engine。
`motolii-shell`は凍結icedアセンブラであり、製品 front の依存グラフに入らない。

## Stage ゼロコピー

切り方の正本は
[Stage ゼロコピー Makepad fork 台帳](../../../docs/reviews/2026-08-26-stage-zero-copy-makepad-fork-seam.md)
(裁定256)。Host / Makepad fork / r7 の3室。既存 `SharedBGRAu8` は触らない。

通常経路は Shared。失敗はエラー画面。FallbackCpu は通常表示に使わない。
