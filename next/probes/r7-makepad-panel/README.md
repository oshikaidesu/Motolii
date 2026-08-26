# R7 Makepad panel probe

これは製品 front の Makepad ホストです(裁定251/252)。
意味の正本は`motolii-store` / `motolii-shell-state` / `motolii-engine`。
`motolii-shell`は凍結icedアセンブラであり、製品核ではありません(裁定253)。
旧世界のモックを正本にしません。standalone workspace なのは next/ への折り込みが未了なだけです。

## 正本

- 構造・状態・余白: `next/reference/mocks/` の現行モック
- 寸法: `next/ui/motolii-tokens-rs/tokens/dimensions.json`
- 実装面: `panel.splash`
- 再読込ホスト: `src/main.rs` の `HotPanel`
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

Chrome の付け替え: 上で起動し、`src/chrome/gallery.rs` か `src/chrome/mod.rs` / `src/chrome/parts/*.rs` を保存すると窓が更新される。

ウィンドウ起動後に `panel.splash` を編集すると、ホストが 120ms 間隔で変更を検知し、
`HotPanel` がメイン VM 内で面だけを再評価します。Stage は
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

`Splash` の標準ローダーはサンドボックス化されたモジュールで `mod.res` を取り除くため、
外部 SVG を `crate_resource("self://resources/icons/...")` から安定して参照できません。
この probe では `HotPanel` が probe 自身の manifest path を持つ `ScriptMod` を作り、
SVG リソース解決とホットリロードを同じ面で成立させています。SVG は
`resources/icons/` に集約し、regular / bold / code のフォント役割を `panel.splash` に明示しています。

Makepad は [oshikaidesu/makepad](https://github.com/oshikaidesu/makepad/tree/motolii-magnify) の
git revisionへ固定しています。fork差分は意味を持たないgesture transformイベントとplatform producer
だけに限定し、Timeline固有の判断は`gesture_input.rs`より下流へ置きます。候補の依存を製品 workspace
へ混ぜないため、この probe 自体も standalone workspace のままにします。製品 front は `motolii-shell` を引きません。

## Ableton比較ループ

Abletonから借りるのはArrangementの外観ではなく、transport・時間面・色付きの対象列・
密度・選択結果の即時性です。各反映でAbleton公式資料と実窓を見比べ、`panel.splash` と
SVGだけを更新します。

- 参照: [Live](https://www.ableton.com/en/live/)、[Live Concepts](https://www.ableton.com/en/live-manual/12/live-concepts/)、[First Steps / Info View](https://www.ableton.com/en/live-manual/12/first-steps/)
- 軸: `macro_layout`, `transport_density`, `time_surface_rhythm`, `semantic_color`, `focus_and_feedback`, `copy_minimization`
- 低スペックLLMの「Abletonと判別しにくいか」は視覚の煙検知として使う。合否は実窓の操作・意味・Document接続で決める。
- 公式画面の画像はリポジトリへ複製せず、上記URLを参照元として記録する。

## 検収

- `cargo run` が実窓を開き、Browser・Stage・Inspector・Timeline が同じ面に表示される
- Stage に fixture の `Document → Engine → re_renderer` 由来フレームが表示される
- playheadをドラッグすると`Session.playhead`とStage frameが同じframeへ更新される
- laneを上下へドラッグすると`LayerMeta.order`が1 undoで変わり、Stage重なりとlane順が一致する
- Timeline上の二本指horizontal scrollでtime panし、Option-scrollで時間軸だけがzoomして目盛り間隔が表示尺へ追随する
- 斜めtrackpad入力のaxisがgesture中に変わらず、OS momentumが次のtouchで停止する
- macOS pinchでpointer anchorを保つtime zoomが連続動作する
- Windows/Linuxはnative producer着地までAlt/Option-scroll fallbackを使う
- `panel.splash` の文字または寸法を編集し、再起動なしで表示が更新される
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
