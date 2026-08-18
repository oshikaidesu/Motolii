# iced 視覚再現レーン(2026-08-18)証拠

## 事実

利用者が実機で「UI も html や egui、skia の再現が全くできていない」と報告。
実物は素のシステムフォント "Inspector" 見出し + 平文一行だった。原因は
発注側の穴 — M-4 系のどの capsule にも「見た目の再現」が受入条件として無く、
theme レーンは色 token を M-1 画面へ当てただけだった。

作業途中で正本が訂正された: 再現目標は egui shell の絵ではなく、
`docs/mocks-ui/public/{inspector,browser,timeline}-library.html` + 同 `.css`
そのもの(egui は写しにすぎない)。このディレクトリの `mock-*-reference.png`
は、その html を実ブラウザで撮った基準画像(利用者から支給)。

## 並び絵

| 面 | 基準(mocks-ui) | iced(このレーン後) |
|---|---|---|
| Inspector | `mock-inspector-reference.png` | `iced-inspector-crop.png` |
| Browser | `mock-browser-reference.png` | `iced-browser-crop.png` |
| Timeline | `mock-timeline-reference.png` | `iced-timeline-crop.png` |
| (窓ぜんたい) | — | `iced-seated-wgpu.png`(1960×1300 = 980×650 論理 ×2 scale) |
| (座席なし) | — | `iced-start-screen-wgpu.png` |

iced 側は `--screenshot` ではなく `iced_test::Simulator::snapshot` で撮った
(`--project` に相当する「既存プロジェクトを CLI から開く」口が iced 側にまだ
無いため — 残差節参照)。撮り方の再現手順は本 README 末尾。

## 自分の目で見た差分(正直に列挙)

### Inspector(最も近づいた)

- 一致: panel header の緑 accent bar + 太字見出し、identity 行の raised 面 +
  M/S ボタン、列見出し行(Property/X/Y)、TRANSFORM の凹んだ section 帯、
  property 行の左 3px 色帯(Position=data/Rotation=action-active/
  Scale=shape、`inspector-library.html` の `--property-color` 割当をそのまま
  写した)、key 列の ◇/◆、値の等幅右寄せ
- 差: 基準は **X/Y/Z の3列**(3D transform)。この製品の read-model
  (`inspector_model::ParamRow`)は **X/Y の2列**(2D)しか持たない — Z 列は
  発明していない(Q0)。基準の Effect/Custom タブ、shape thumbnail の実色
  swatch(fill color)、kind icon(丸/角/#/≡ の記号)、section の折り畳み
  chevron は**足していない** — この製品の read-model がタブ/fill色/折り畳み
  状態を持たないため(死に chrome を作らない)
- 差: 基準は横幅 496px 前提。この製品の Inspector pane は 300px 固定
  (`INSPECTOR_PANE_W`)なので、値セルは基準と同じ 64px を使うと Y 列で
  ほぼ埋まる。X/Y 以外(Z・kind icon 列)を切って収めた
- **実測で踏んだ罠**: section chrome を足した分だけ縦が伸び、980×650 窓の
  上段(高さ 1:1 の片側 ≈325px)には TRANSFORM の 4 param 全部 + EFFECTS が
  収まらない。当初 `column![]` に直に積んだら、収まりきらない行が
  0 高さへ押し潰されて **押せないボタン**になった(`an_effect_toggle_
  writes_the_shared_definition` が red で発覚)。修正は
  `inspector-library.css:108 .tableScroller{overflow:auto}` と同じ発想 —
  TRANSFORM/EFFECTS を `scrollable` に包んだ(`inspector_pane.rs::inspector`)。
  現状、1つの selection で FX が1つでもあると **スクロールしないと ON/OFF
  ボタンに届かない**。行の padding をもう少し詰めるか、pane 幅を広げるかの
  余地がある(次レーンへの引き継ぎ)

### Browser

- 一致: header(太字見出し + 右に library root 名)、rail の選択強調、
  grid card(thumbnail 枠 + 2行 caption、name=太字/meta=muted)、
  selection tray の帯
- 差: 基準の色は `--motolii-color-*` token に対応が無い raw hex
  (`#111315` 地、`#b9a660` accent 等 — 21 token を総当たりしても一致が
  無いことを確認済み)。**独自 hex を発明しない柵**により、この製品は
  引き続き `theme::Tokens` の意味の近い role(`surface_panel`/
  `surface_raised`/`action_active` 等)で塗っている。色そのものは基準と
  一致しない(色相・彩度が違う)— 直すには新しい token を正本
  (`ui/motolii-tokens/sources/`)へ足す decision が要る、という認識で
  このレーンでは止めた
- 差: 基準にある検索欄・Filters/Tags ボタン・Effects/Create/Panels タブ・
  view 切替(grid/list/details)は**置いていない** — この製品にはまだ検索/
  フィルタ機能そのものが無い(Q0: 機能の無いボタンを見た目のためだけに
  置かない、`lib.rs` の「ここに無いもの」に明記済みの非目標)

### Timeline

- 一致: 全体の配色(`timeline/canvas.rs` の BG/RULE/INK/DIM/GOLD accent は
  元々 `timeline-library.css` の値と一致 — `#292929` 地、`#111` 罫線、
  `#8d8d8d` ruler 字、`#e9cf72` accent)。ruler の 0:00/0:02/… 目盛、行の
  縞、選択 bar の白枠
  - **1件修正した**: playhead 本体は元コードで accent(gold #e9cf72)を
    塗っていたが、`timeline-library.css:93` の `.playhead` は白
    (`#e8e8e8`)で、gold は別要素 `.timeGuide`(drag 中の snap 案内線)の
    色だった。既存の `SELECTED`(#f2f2f2、白系)へ差し替えた
- 差: 基準にある **ARRANGEMENT 俯瞰帯**(全体の縮小表示)、行ヘッダの
  M/S ボタンと kind アイコン、**KEY TOOLS パネル**(ALIGN/INTERPOLATION)、
  ヘッダの Snap/Fit ボタンと LAYERS ラベルは**置いていない** — これは
  見た目の話ではなく、この製品にまだ無い機能(俯瞰スクロール・行ごとの
  M/S・ALIGN/INTERPOLATION 操作)そのもの。Q0 により、動かないボタンを
  見た目のためだけに足すことはしていない
- 差: `snap_candidates`/`snapped` は当たり判定だけを返し、**案内線を描いて
  いない**。timeGuide の gold 線を描く先が無かったので `ACCENT` 定数ごと
  削った(次に描くときは同じ #e9cf72 を使えば足りる、と comment に残した)

## token へ追加した値

**0件。** 全ての新しい塗りは既存 21 role(`theme::Tokens`)の再利用で足りた:

| 用途 | 使った role | 出所 |
|---|---|---|
| Inspector panel header accent bar | `way_inspector` | `inspector-library.css:39` |
| Position 行の帯 | `data` | `inspector-library.html:29`(`--property-color: var(--mock-role-data)`) |
| Rotation 行の帯 | `action_active` | `inspector-library.html:36` |
| Scale 行の帯 | `shape` | `inspector-library.html:43` |
| Opacity 行の帯 | `text_secondary` | `inspector-library.html:59` |
| FX badge / pill / 左帯 | `way_plugins` | `inspector-library.css:199`(`--effect-color` 既定値) |
| FX param 行の帯(F64/Vec2,3) | `data` / `way_inspector` | `inspector-library.css:297-302`(scalar/vector) |
| Timeline playhead | `SELECTED`(既存の canvas 内定数、token とは別家) | `timeline-library.css:93` |

Browser の raw hex(`#b9a660` 等)は token に対応が無く、**追加もしていない**
(上の「差」節に理由を記載)。

## `--screenshot` の使い方(このレーンで足した常設器具)

```
cargo run -p motolii-shell-iced -- --screenshot out/shell.png [frames]
```

egui shell(`motolii-blitz-shell`)と同じ引数の形。`frames`(既定 10)フレーム
描いてから 1 枚 PNG にして窓を閉じる。実装は `crates/motolii-shell-iced/src/
main.rs` の `Host`/`Capture`/`HostMessage` — 撮影の都合は `Message`(製品の
意味の列)に一切混ぜていない(`HostMessage::App` で包むだけ)。

**現状の制約**: iced Launch にはまだ `--project` が無い(`launch.rs`)ので、
`--screenshot` は常に**座席なしのスタート画面**を撮る。Inspector/Browser/
Timeline を実窓で撮るには、egui 側のように「実プロジェクトを開いてから撮る」
経路が要る — このレーンでは代わりに `iced_test::Simulator::snapshot`
(テスト運転席と同じ、窓を開かない撮影)でシード済み状態を撮って
`iced-seated-wgpu.png` を作った(生成に使った使い捨てコードは
`crates/motolii-shell-iced/examples/dump_evidence.rs` に一時的に置いて実行し、
コミット前に削除済み — 再生成したい場合は同じ手順を
`tests/inspector_drive.rs::an_effect_toggle_writes_the_shared_definition` の
fixture 構築部を参考に書き直せる)。

## テスト集計

`cargo test -p motolii-shell-iced` — 22 test binary、全 green(既存分含め
1件も落としていない)。今回の変更で新しく踏んだ罠(scrollable が要る・
`iced_test::Simulator` のホイールは `point_at` を呼ばないと
`cursor_over_scrollable` を得られない)は
`tests/common/mod.rs::scroll_then_click` と
`tests/inspector_drive.rs::an_effect_toggle_writes_the_shared_definition`
に直接残してある。

## 手動確認事項(利用者への引き継ぎ)

1. 実機で `cargo run -p motolii-shell-iced` を開き、`Cmd+N` → 適当な素材を
   window へ drop → layer を選んで Inspector を見る。FX を持つ layer では
   EFFECTS まで見るのに **スクロールが要る**(トラックパッド/ホイールで
   下へ)。これが「触れそうで触れない」に見えないか確認してほしい
   (Q0 の主観判定は実機でないと確定しない、との既存決定どおり)
2. Browser の色相が基準(mock)と違って見えるはず(token 対応が無い raw hex
   だったため、意味の近い role で代替した)。ここを直す価値があるなら、
   次の一手は「新しい token を正本へ足す」decision
3. Timeline の ARRANGEMENT 俯瞰帯・KEY TOOLS パネルは**未着手の機能**(見た目
   ではない)。見た目レーンの範囲外だが、実機で見て「無いと物足りない」か
   どうかは利用者の目で見てほしい
