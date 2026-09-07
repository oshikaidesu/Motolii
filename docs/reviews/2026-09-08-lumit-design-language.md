# Lumit の設計言語をトンマナの定規にできるか — 調査(2026-09-08)

状態: **比較中**(利用者の実機裁定待ち)。利用者提示 https://lumitlab.com/

## Lumit とは

- AE + Vegas を狙う compositing / retiming editor。**Rust engine + Flutter UI**(`crates/` と `flutter_ui/`、`crates/lumit-bridge` で接続)。Motolii と同じ形
- GitHub `luminalmvm/Lumit`、GPLv3、22 star、2457 commit、活発。Dart 245 file / 16 万行
- 設計文書が正本として揃う: `docs/15-DESIGN.md`(色・型・密度・動き・声、1741 行)、`docs/07-UI-SPEC.md`(panel の構造と挙動、3213 行)、`docs/17-BRIDGE-CONTRACT.md`
- 面の段(surface ramp)の構造は **rerun の re_ui に倣う**と明記。Motolii の描画が rerun なので相性が良い

## 得られる物(値と型 = PATTERN)

`15-DESIGN.md` の主な数値。token 名は彼らのもの。

| 種 | 値 |
|---|---|
| 面 5 段 | surface_0 `#0b0c0e`(地・入力の井戸)/ 1 `#131517`(panel)/ 2 `#1a1d20`(帯・header・行)/ 3 `#212528`(hover・浮く物だけ)/ 4 `#2b3034`。Viewer の周囲は厳密に無彩 `#121212` |
| 文字 4 段 | `#eef1f2` / `#c2c8cb` / `#8b9296` / `#5e666b` |
| 罫線 | hairline `#26292c`、strong `#3c4145`。部品は枠線でなく**塗りの段**で idle→hover→pressed |
| accent | `#35785e` 1 色、仕事は「面に 1 つの塗りボタン・playhead・active tab」だけ。もう 1 つの状態色 `animated` `#d8a24a`(キー・stopwatch・work area・値の focus)、仕事一覧は閉じている |
| 角 | Sharp 形: 部品 2、浮く物 6。Round 形は capsule |
| 密度(Regular / Compact) | header 帯 22、副次行 19/18、layer 行 23/22、property 行 27/26、dropdown の閉じ面 20/18、値の井戸 20、dialog の井戸 22、status 18 |
| 型 | Hanken Grotesk(UI)と Geist Mono(**数字・timecode・容器のラベルは全部 mono**、tnum)、どちらも OFL。本文 11、井戸の数字 11 mono、単位 10 mono、kicker 9 mono caps +0.12em。chrome は 13 を越えない |
| 余白 | 4 / 8 / 12 / 16。panel は罫線 1 本で接し隙間なし |
| 動き | ≤150ms、transform と opacity だけ、3 段(All / Minimal / None)。再生は動きではない |
| 色を唯一の符号にしない | キーは形(◇ □ ○、砂時計)、cache は明度 + 高さ、layer 種は色 + glyph |

部品の型(挙動)も参考になる: `widgets/controls/` の value_field(scrub の modifier 段 Shift×10 / Ctrl×0.1 / Alt×0.01)、dropdowns(閉じ面 = label + caret、ellipsis)、menus(safe-triangle hover、tick 列)、popups(**開いている popup の鎖を 1 つの権限で管理**、Esc 1 回で全部閉じる)、slider(commit on release)。

## 取れない物

- **コード**: GPLv3。Motolii は MIT / Apache-2.0 なので持ち込めない(先例: 2026-08-16 Ardour / LMMS「GPL コードは持ち込まず PATTERN のみ」)。文書の文面も同じ扱い。数値と規則は自分の言葉で写す
- **自前 widget 一式**: Lumit は Material を使わず全部自作(HouseButton / MenuRow / FloatSurface …)。Motolii の憲法は「UI は Flutter の部品」。同じ見た目は theme(2026-09-08 決定)で出す
- **icon set**: 自作 16 grid / 1.5px。glyph は借りない

## Motolii への写像案(裁定待ち、UX 合否は利用者の専権)

現行 `EditorTheme`(app `#292929` / panel `#3c3c3c` / raised `#484848` / hover `#585858`、ink `#dddddd`、accent `#ffaa61`)は Lumit より 2 段明るく、段の差が小さい。Lumit に寄せるなら:

1. 面を 5 段の名前にし(`EditorTheme` の app→surface0 … を値だけ差し替え)、Stage の周囲を無彩の 1 色にする
2. 文字を 4 段(ink / muted の 2 段 → primary / secondary / muted / disabled)
3. 数字を mono に(Geist Mono を bundle、`EditorMetrics` に mono の TextStyle を 1 つ)。Timeline の timecode・Inspector の井戸・Stage 下縁の寸法
4. accent の仕事一覧を閉じる(選択・playhead・active。キーは第 2 の状態色へ)
5. 密度は現行 `row = 20` を維持(Lumit の dropdown 面 20 と一致)、property 行を 27 にするかは実機で

いずれも theme.dart / metrics.dart の値の差し替えで済み、panel は触らない。**先に 1 と 3 を実窓で見て、続けるか決める。**

## 出典

- https://github.com/luminalmvm/Lumit — `docs/15-DESIGN.md` §2 面 / §2.2 文字 / §2.3 罫線 / §3.1 色の役 / §5 icon / §7 密度と型 / §12A.6 寸法表、`flutter_ui/lib/theme/theme.dart`、`flutter_ui/lib/widgets/controls/*.dart`
- https://lumitlab.com/ 、https://docs.lumitlab.com — 実画面 `web/src/assets/shots/workspace.png`、`web-docs/src/assets/shots/settings.png`
