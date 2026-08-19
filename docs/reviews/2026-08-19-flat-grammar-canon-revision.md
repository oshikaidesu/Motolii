# フラット文法の正本改定と評価軸の定数化(第3波設計)

日付: 2026-08-19
状態: **設計**(利用者裁定「正本改定を土台とし、Ableton系の余白を作らないフラットUIへ」
「評価軸の定数化をゴールとする」の実行設計。値の根拠は全て同日の実測3本)

根拠実測: [Ableton密度実測](2026-08-19-ableton-density-measurements.md) /
Abletonフラット文法画素実測(本文書に転記) / モック余白の器具実測(css-metrics、本文書に転記)。

## 実測が示した3つの事実

1. **Abletonの分離文法**: 隣接面の背景色は同一(実測 209=209 / 165=165)で、分離は
   1〜2px論理のヘアライン**だけ**。gap 0。行内上下padding≈2px、リスト左インデント≈5.5px、
   角丸0、トグルは塗り反転のみ、影・グラデーション無し。セクション区切りのみ行ピッチ1.3倍。
   独立ドック境界だけ約9pxの帯を持つが、それは**リサイズグリップ**(機能)であって余白ではない
2. **現行モックは既に大半フラット**(境界1px・角丸実質0・余白は最大27px)だが、
   **分散が大きい**: 面色が inspector 3段 / browser 8〜9段(token未接続) / timeline 6段、
   文字基準が 9 / 8 / 11 px でパネルごとにズレる
3. Ableton の可読性の正体は装飾でなく**分散の小ささ**(文字1バンド・行格子・chrome無彩色・
   部品文法の反復)。分散は測れる → 定数化できる

## 定数表(正本。第3波の全レーンはこの値だけを使う)

### 面(surface ladder) — 3段固定

| role | 値 | 備考 |
|---|---|---|
| surface-app | `#141414` | 最深(window地・timeline溝) |
| surface-panel | `#1a1a1a` | パネル地(全パネル同一。**隣接パネルを別色にしない** — 分離は線) |
| surface-raised | `#222222` | 持ち上がり(入力欄・チップ地・hover前面) |

inspector の現行3段をそのまま全面の正本へ昇格。browser の 8〜9 段・timeline の 6 段は
この3段+状態ティント(hover/pressed の α overlay)へ畳む。
状態は cosmic-theme 実測の α ladder を借用: **hover = neutral α0.10 / pressed・selected = α0.20 /
disabled = α0.50**([icedエコシステム採掘](2026-08-19-iced-ecosystem-mining.md)、MPL-2.0)。

### 線(border) — 2role + hairline

| role | 値 | 用途 |
|---|---|---|
| hairline | `#111111` 1px | パネル間・ドック間の分離(gap 0 で線1本) |
| border-default | `#3b3b3b` 1px | 部品の枠・行内の区切り |
| border-strong | `#686868` 1px | 強調枠(focus以外) |

角丸 = **0**(チップ・ドット等の識別記号のみ 50%/8px 系を維持)。影・グラデーション禁止(既定どおり)。

### 文字 — 1バンド3段

| role | 値 | 用途 |
|---|---|---|
| text-base | **11px** | 本文・行ラベル(Ableton実測の主文字帯と同値) |
| text-dense | **9px** | 補助・メタ行 |
| text-micro | **8px** | 単位・添字(これ未満は禁止) |

timeline は現行11で一致。inspector 9→基準11/補助9へ、browser 8→基準11/補助9へ
(browser の 6.5/6px は 8px へ引き上げ — 下限フロア)。panel title は 12px(現行14から1段圧縮)。
数値は tabular/monospace(既定どおり)。

### 格子と余白

| 定数 | 値 | 根拠 |
|---|---|---|
| 行高(list/track) | **20px**(行高の種類 ≤2: 20/畳み行18) | Ableton 19-20実測・2026-08-08決定の下限 |
| 行内上下padding | **≤2px** | Ableton実測 |
| インデント単位 | **6px** | Ableton実測5.5の丸め |
| セクション区切り | **26px**(=行ピッチ1.3倍)。それ以外の呼吸余白禁止 | Ableton実測 |
| spacing scale | **{2, 4, 6, 8}**(scale外の場当たり値禁止) | モック実測の主要山(3-8px帯)の整列 |
| pane間 gap | **0**+hairline 1本 | Ableton実測 |
| リサイズグリップ | **8px**(resize可能な境界のみ。装飾余白としては使用不可) | Ableton実測≈9の丸め |

## 改定手順(レーン分割)

| レーン | 中身 | 順序 |
|---|---|---|
| G | **モックcss改定**: 3枚(`{inspector,browser,timeline}-library.css`)を上の定数表へ。browser パレットの token 接続(`mock-candidates.css` 経由)。器具 `motolii-css-metrics` で再抽出し新計算値を evidence 化 | 先行 |
| I | **iced追随**: browser_pane `colors`/`font_size`・inspector_pane・timeline を新計算値へ。type_scale を {11,9,8}+title12 へ改定、spacing scale {2,4,6,8} を theme::space へ、全ベタ書きを scale 参照へ置換 | Gの後 |
| H | **トンマナoracle実装**(=定数化のゴール): (1) css-metrics ベース — モック3枚の面色集合⊆ladder3段・文字サイズ集合⊆バンド・border色⊆3role を assert、(2) ソース走査 fence — iced 内 `.size(`/`.spacing(`/raw hex が scale/palette 外なら fail(intent_gateway_fence と同型)、(3) token ペアの contrast 計算(文字4.5:1・境界3:1)、(4) 行高の種類数≤2 | Iの後(最終状態をpin) |

H が green になった時点で campaign のゴール(随時の目視確認からの解放)達成。
以後の意匠変更は oracle の red で検出される。第4波(UIスケール%)は H の上に載せる。

## 非目標

- 機能変更・hit target の縮小(WCAG 24px 級の押下領域は行高と別に維持)
- ライトテーマ同時改定(ladder の role 構造は共通化するが値の確定は dark 先行)
- ゲシュタルト審判の自動化(Q0 5秒テストは人間審判のまま)
