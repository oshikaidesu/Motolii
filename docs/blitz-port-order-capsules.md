# Blitz移植の発注capsule

[Blitz移行起案](reviews/2026-08-15-blitz-ui-runtime-adoption-proposal.md)が**採択された場合**に
外部実装へ渡すためのclosed order capsule集。形式は[AGENTS.md](../AGENTS.md)の
`BASE / AUTHORITY / CURRENT STATE / OWNER / EXACT TARGET / ALLOWLIST / READ SET / ORACLES / NON-GOALS / RETURN`。

> **2026-08-15の[利用者裁定](reviews/2026-08-15-blitz-ui-runtime-adoption-proposal.md#裁定)により発注凍結は解除された。**
> C1〜C6は発注してよい。ただし本書のNON-GOALSとORACLEは裁定後も全て有効であり、
> **裁定が担保しているのは採否だけで、移植が意味を変えないことは担保していない。**

## この文書が存在する理由

移植作業は「意味は既に決まっており、置き場所だけが変わる」種類の仕事である。
にもかかわらず外部LLMは、**判断材料が目の前にあると設計者として振る舞い始める**
（色を選ぶ、間隔を決める、命名を変える、「より良い」構造を提案する、意味を新設する）。
それは移植ではなく再設計であり、[絶対規律](../AGENTS.md)の`自己発注禁止`に反する。

そこで本書は全capsuleに共通の拘束を先に置き、**各値の出所をfile:lineで固定する**。
実装担当は値を**決めるのではなく写す**。

## 全capsule共通の NON-GOALS（設計者化の禁止）

| # | 禁止 | 代わりにすること |
|---|---|---|
| 1 | **色・寸法・間隔・字送りを決める** | `crates/motolii-ui/src/timeline_egui/theme.rs` と `geometry.rs` から**写す**。定数を新設しない |
| 2 | **操作の意味を決める**（何を掴めるか、何が起きるか） | `timeline_egui/clip_band.rs`、`timeline_skia/hit.rs`、`docs/ui-interaction-language.md` から写す |
| 3 | `DomainIntent` / `Document` schema / 公開plugin契約を拡張する | 足りなければ`RETURN`。**enumに1つ足す判断も設計判断である** |
| 4 | `docs/decision-index.md` と `docs/reviews/**` を編集する | 読むだけ。矛盾を見つけたら`RETURN` |
| 5 | oracle・test・golden・閾値を通すために変更する | 落ちたら止める。testが誤りに見えるなら`RETURN`（[AGENTS.md](../AGENTS.md)) |
| 6 | 「より良い」構造・命名・抽象を提案して適用する | 現行の構造と命名を保つ。提案は`RETURN`のnoteへ書き、実装しない |
| 7 | `timeline_skia/` を削除する | **意味/hit/oracleの源として残す**（2026-08-15 egui裁定と同じ扱い） |
| 8 | 複数capsuleを1PRにまとめる | 1 capsule = 1 branch = 1 PR |

**判断が要ると感じたら、それは`RETURN`の合図である。** 判断してはならない。

## 全capsule共通の NEGATIVE ORACLE（Blitzはブラウザではない）

HTML/CSSはLLMとの相性が良く、それが本移植を成立させている。
**同時にそれが最大の罠でもある。** LLMのCSSはブラウザで訓練されており、
Blitzで効かないCSSを書いても**エラーにならず、silentに違う絵になる**。
[実測プローブ](reviews/2026-08-15-blitz-ui-runtime-probe.md)で既に2件踏んでいる。

| # | 禁止 | 理由 / 代わりにすること |
|---|---|---|
| 9 | **ブラウザで動くはずのCSSを、Blitzで確認せずに使う** | 画像は`width`指定でも**元解像度でアトラスへ載る**（probe実測）。使うCSSは`spikes/blitz-probe/`で実際に効いたものに限る |
| 10 | **JSに依存する構造を書く**（`<script>`、インラインhandler、JS前提のライブラリ） | **BlitzはJSエンジンを持たない。**挙動はRust側に書く |
| 11 | **性能問題をCSSで解こうとする** | メモ化はCSSではなく**Dioxus側**（probe実測）。CSSで直らないものはCSSの問題ではない |
| 12 | 効くか不明なCSSプロパティを「たぶん効く」で入れる | `spikes/blitz-probe/` に最小再現を足して**確かめてから**使う。確かめられないなら`RETURN` |

**判定法**: `timeline_blitz/` 等に書いたCSSプロパティのうち、
`spikes/blitz-probe/` で一度も使われていないものが**残っていないこと**。

## RETURN の形式

`STOP` / `TARGET_MISSING` を状態語だけで返さない。次を揃える（[AGENTS.md](../AGENTS.md)）。

```
RETURN:
  探索範囲:      どのファイル・どの決定文書を読んだか
  候補:          考えられる選択肢
  不適合理由:    なぜどれも選べないか
  exact gap:     何が決まっていないのか（1文）
  再入場条件:    誰が何を決めれば続行できるか
  安全に続けられるedge: 同じcapsule内で止まらずに進める部分
```

---

## C1 — Timeline描画をBlitzへ（意味は移さない）

| 項目 | 内容 |
|---|---|
| **BASE** | 裁定後の最新 main |
| **AUTHORITY** | [Blitz移行起案](reviews/2026-08-15-blitz-ui-runtime-adoption-proposal.md)の裁定 |
| **CURRENT STATE** | Timeline描画は `crates/motolii-ui/src/timeline_egui/`（egui）。`ui/motolii-rn/native-renderer/src/timeline_skia/`（rust-skia）が並存 |
| **OWNER** | UI描画層のみ |
| **EXACT TARGET** | 現行Timelineの見た目をBlitz(HTML/CSS)で描き、自前wgpuテクスチャへ出す。**入力・意味・Documentは触らない** |
| **ALLOWLIST** | 新規 `crates/motolii-ui/src/timeline_blitz/**` のみ |
| **READ SET** | `timeline_egui/theme.rs`（色・帯高）、`geometry.rs`（座標変換）、`ruler.rs`、`clip_band.rs`、`rows.rs`、`spikes/blitz-probe/src/bin/ui_mock.rs`（再現済みモック） |
| **POSITIVE ORACLE** | `ui_mock.rs` と同一のHTML/CSS構造で、`theme.rs` の全定数が**リテラルで一致**すること（色・帯高・sidebar幅・行高） |
| **NEGATIVE ORACLE** | `timeline_blitz/` に**新しい色定数・寸法定数が1つも無い**こと。全て `theme.rs` / `geometry.rs` からの写し |
| **NON-GOALS** | 共通NON-GOALS全部 + 入力処理を書かない + `timeline_egui/` を削除しない |
| **RETURN** | `theme.rs` に無い色が必要になった時点で即`RETURN`。**自分で選ばない** |

## C2 — 入力ルーティング（Motolii側が持つ）

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | winitイベントを `blitz_dom::EventDriver::handle_ui_event` へ流し、`BaseDocument::hit(x,y)` でBlitz面かcanvas面かを振り分ける |
| **ALLOWLIST** | `crates/motolii-ui/src/timeline_blitz/input.rs`（新規） |
| **READ SET** | `crates/motolii-input/src/input_router.rs`（`InputPhase` / `SafetyInterrupt`）、`spikes/blitz-probe/src/bin/texture_mode.rs`（実証済み経路） |
| **POSITIVE ORACLE** | `InputPhase::Cancel` と `SafetyInterrupt::{PointerCaptureLost, WindowFocusLost}` が**必ず経由する**こと（`ui-friction-ledger` F17の再発防止） |
| **NEGATIVE ORACLE** | `enqueue` 系の戻り値を `let _ =` で捨てている箇所が**0件**（F18の再発防止） |
| **NON-GOALS** | 共通NON-GOALS全部 + **`DomainIntent` に variant を足さない**（足りなければRETURN） |
| **RETURN** | Timeline操作を表す `DomainIntent` が無いことに気づいた時 → **足さずにRETURN**。これは[F18](ui-friction-ledger.md)が指す未決事項 |

## C3 — key帯を custom widget へ（密な面の1ノード化）

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | keyframe描画を `blitz_dom::Widget` 実装へ移す。clip とラベルはDOMのまま |
| **ALLOWLIST** | `crates/motolii-ui/src/timeline_blitz/key_widget.rs`（新規） |
| **READ SET** | `spikes/blitz-probe/src/bin/custom_widget.rs`、`timeline_skia/hit.rs`（当たり判定の意味） |
| **POSITIVE ORACLE** | key数を10倍にしても `resolve` が増えないこと（実測: `spikes/blitz-probe` P8） |
| **NEGATIVE ORACLE** | widget内で**文字を描かない**（`draw_glyphs` を使わない）。文字はDOM側 |
| **NON-GOALS** | 共通NON-GOALS全部 + hit半径を変えない（`F7` で5.6px視覚一致が既決） |
| **RETURN** | 間引き（`(+N)`）の見た目を決める必要が出たら`RETURN`。**これはUI文法の決定であり実装判断ではない** |

## C4 — ドッキング移植 — **退役（2026-08-15 利用者裁定）**

**ドッキングは egui 側の責任とする。Blitzへは移植しない。**

理由（利用者の言葉）: 「簡単そうに見えて大変」。`egui_tiles` の split/tab/resize/hide/reset は
**見た目ではなく状態機械**であり、CSS flex/grid で描けるのは配置結果だけで、
移植コストの本体は写せない側にある。ホストは既に `eframe(egui)` で
（[P7](reviews/2026-08-15-blitz-ui-runtime-probe.md#p7--提案構成そのものを窓で動かし上限を測る)）、
Blitzパネルはテクスチャとして pane へ合成される
（[P12 透過合成 PASS](reviews/2026-08-15-blitz-ui-runtime-probe.md#p12--透過合成2026-08-15追測採択時の未了4)）。
`egui_tiles` を移植せず**そのまま使う**構成が、この2つの実測と一致する。

したがって本capsuleは**発注しない**。以下は退役時点の原文で、履歴として残す。

- 既に存在する `crates/motolii-ui/src/blitz_ui/`（dock skin）と `blitz_dump` の dock パネルは、
  **製品の役割を持たない**。C1/C6/C7 と同じ器で描けることを示した証拠として残す
- 合成先 format は `Rgba8Unorm` で揃える（[P12(a)](reviews/2026-08-15-blitz-ui-runtime-probe.md#p12で出た効いているつもりで効いていない2件)）。
  egui 側の pane texture format がここに効く
- **未確認**: `egui_tiles` の pane 内へ他所のテクスチャを出す口の形

<details>
<summary>退役した原文</summary>

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | `egui_tiles` のツリーとD&D状態機械を移植し、配置結果をCSS flex/gridで描く |
| **ALLOWLIST** | `crates/motolii-ui/src/dock/**`（新規） |
| **READ SET** | `egui_tiles`(MIT OR Apache-2.0) の `tiles.rs` / `container/mod.rs` / `tile.rs` / `tree.rs` / `grid.rs` / `linear.rs` / `tabs.rs`、`docs/ui-interaction-language.md:75`（製品要件） |
| **POSITIVE ORACLE** | 分割・tab化・resize・表示/非表示・既定presetへのreset が動くこと（`ui-interaction-language.md:75` の文言そのもの） |
| **NEGATIVE ORACLE** | `egui` への依存が `dock/` に**0件**であること |
| **NON-GOALS** | 共通NON-GOALS全部 + **ドッキングの操作感を「改善」しない**（`egui_tiles` の挙動を写す） + ライセンス表記を落とさない |
| **RETURN** | `egui::Id`/`Context` のメモリストア置換で意味が変わりそうな箇所 → `RETURN` |

</details>

## C6 — Browserパネル（フォルダ参照でメディア入口を開く）

`N-MEDIA-PICK` / `N-PROJECT-ENTRY`（ファイルを選ぶ入口が無い）への代替。
**完成条件を直接動かす唯一のcapsule**。

### 計測（2026-08-15。C7と同じ構図であることが後から判明した）

| | 行数 | |
|---|---|---|
| `Browser.tsx` | 544 | うち **JSXマークアップ45行 / style 41行** |
| `browser_host.rs` + `browser_host_runtime.rs` + `media_library.rs` | **1,899** | **意味とメディア管理は既にRust側にある** |

C7(Inspector)より偏りが大きい。**フォルダ走査・メディア一覧・サムネイル管理を新規に実装する仕事ではない。**
既にあるRust runtimeの投影をHTML/CSSで描く仕事である。
同等の機能を `browser_blitz/` に書いていたら、それは**やりすぎのサイン**。

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | フォルダを参照してサムネイル格子を出し、項目を掴んでドラッグできる |
| **ALLOWLIST** | `crates/motolii-ui/src/browser_blitz/**`（新規） |
| **READ SET** | `spikes/blitz-probe/src/bin/browser_panel.rs`（実証済み）、`docs/ui-interaction-language.md`（Browserの役割）、**`browser_host.rs`(661) / `browser_host_runtime.rs`(685) / `media_library.rs`(553) — 意味とメディア管理は既にRust側にある。繋ぐ先** |
| **POSITIVE ORACLE** | 元寸PNG 45枚を表示して60秒 panic しないこと |
| **NEGATIVE ORACLE** | `ImageManager` の `cache` と `texture_bindings` を**フレーム内で新規生成していない**こと（下記の罠）／`browser_host*.rs` と `media_library.rs` を**編集していない**こと／`browser_blitz/` にそれら3ファイルが既に持つ機能の**再実装が無い**こと |
| **NON-GOALS** | 共通NON-GOALS全部 + **配置intentの意味を決めない**（drop先で何が起きるかは未決。RETURN） + native file dialog を作らない |
| **RETURN** | drop したとき Document に何が起きるべきかを決める必要が出たら`RETURN`。**これは製品意味の決定** |

### この2つは必ず踏む（実測で踏んだ）

1. **`ImageManager` の `cache` はフレームを跨いで保持する。** 毎フレーム新規生成すると
   画像が atlas へ再確保され続け、数秒で `AtlasLimitReached` で **panic** する。
   しかも `vello_hybrid/src/render/wgpu.rs:596` の `.unwrap()` なので**捕捉できない**
2. **`blitz_net::Provider` は Tokio reactor を要求する。** 無いと panic

## C5 — RN退役（**最後**）

C1〜C3・C6〜C8がmainで動いた後にのみ着手する（C4は退役）。

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | `rn_product_host/`（実装8,247 + テスト7,273）と `host_bridge/`(4,488) の削除 |
| **NON-GOALS** | 共通NON-GOALS全部 + **他のcapsuleと同時に発注しない** + `timeline_skia/` を巻き込んで消さない |
| **RETURN** | 削除すると失われるoracleが見つかったら`RETURN`。**代替を自分で書かない** |

---

## C7 — Inspector を RN から Blitz へ

C5(RN退役)の**前提**。Inspectorの移植先が無いままRNを消すと製品が消える。

### 計測（2026-08-15、この capsule の形を決めた根拠）

| | 行数 | |
|---|---|---|
| `Inspector.tsx` | 1,596 | うち **JSXマークアップ149行 / style定義85行 / host呼び出し14行** |
| `inspector_host_runtime.rs` | **1,965** | **意味と状態機械は既にRust側にある**（TSXより大きい） |

したがってこれは「ロジックをRustへ書き直す」仕事では**ない**。
**マークアップとスタイルを写し、既にあるRust runtimeへ繋ぐ**仕事である。
Rust側にロジックを新規で書き足していたら、それは**やりすぎのサイン**。

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | `Inspector.tsx` の見た目をBlitzで描き自前wgpuテクスチャへ出す。表示は `inspector_host_runtime.rs` の既存投影を読む。**入力配線は含まない**（C2の担当） |
| **ALLOWLIST** | 新規 `crates/motolii-ui/src/inspector_blitz/**` のみ |
| **READ SET** | `Inspector.tsx`(構造)、**`productStyles.ts`(147行 — 色・寸法の唯一の出所。C1における`theme.rs`に相当)**、`inspector_host_runtime.rs`(繋ぐ先)、`spikes/blitz-probe/src/bin/ui_mock.rs` |
| **POSITIVE ORACLE** | 各セクションが同じ構造で存在し、`productStyles.ts` の値が**リテラルで一致**すること |
| **NEGATIVE ORACLE** | 新しい色・寸法定数が0件 / probe未使用のCSSプロパティが0件 / **`inspector_host_runtime.rs` を編集していない** |
| **NON-GOALS** | 共通NON-GOALS全部 + `PanResponder`(ドラッグ)を移植しない(C2) + `ui/motolii-rn/`を削除・改変しない(**まだ製品正本**) |
| **RETURN** | `productStyles.ts` に無い値が要る時点で即`RETURN`。Rust側にロジックを足したくなったら`RETURN` |

### 実装は EXACT TARGET より狭く返ってきた（2026-08-15、利用者裁定で容認）

上表は「表示は `inspector_host_runtime.rs` の既存投影を読む」と書いているが、
実装（`crates/motolii-ui/src/inspector_blitz/`）は `mod.rs:8` で
**`inspector_host_runtime` を参照しないと明記し、表示値は `sample.rs` の固定値**にしている。

利用者裁定（2026-08-15）: **仮の値で良い。** よって差分は欠陥ではなく**繰り延べ**として扱う。
ただし C7 を「済」と数えると投影配線が誰の担当でもなくなるため、ここに残余として置く。

- **残余**: `inspector_host_runtime.rs` の既存投影 → `sample.rs` の置換。入力配線はC2のまま別
- 移植面（マークアップ・スタイル・寸法）は完了しており、この残余は写す仕事ではなく繋ぐ仕事

## C8 — chrome と panels を RN から Blitz へ

C5の前提。C7と同型で、対象が小さい（計252行）。

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | `chrome.tsx`(172)、`panels/registry.tsx`(24)、`panels/AssetTaggingPanel.tsx`(56) の見た目をBlitzで描く。**入力配線は含まない** |
| **ALLOWLIST** | 新規 `crates/motolii-ui/src/chrome_blitz/**` のみ |
| **READ SET** | 上記3ファイル、`productStyles.ts`、`spikes/blitz-probe/src/bin/ui_mock.rs`、`docs/ui-interaction-language.md`(**読むだけ。新要件を発明しない**) |
| **POSITIVE ORACLE** | 各要素が同じ構造で存在し `productStyles.ts` の値がリテラル一致 |
| **NEGATIVE ORACLE** | 新しい色・寸法定数が0件 / probe未使用のCSSが0件 / **パネル登録の仕組みを新設しない**(`registry.tsx`の構造を写す) |
| **NON-GOALS** | 共通NON-GOALS全部 + chromeに何を置くかを決めない + `ui/motolii-rn/`を削除・改変しない |
| **RETURN** | `productStyles.ts` に無い値が要る時点で即`RETURN` |

### 実装済み（2026-08-15）— `crates/motolii-ui/src/chrome_blitz/`

`export` / `settings` / `panels` / `parts` の4枚を `motolii-blitz-dump` から出せる。専用 bin は作らず、
Timeline / Browser / dock と同じ道具に足した（4枚を並べて見るため）。

値の出所が**3つ**ある点が C7 と違う。`registry.tsx:20-24` と `AssetTaggingPanel.tsx:39-56` は
`productStyles.ts` ではなく**自前のローカル `StyleSheet`** を持つ。`theme.rs` は3つの表に分け、
それぞれの原文と逐語で突き合わせる。両方が `root` という名前を持って衝突するため、
CSSは `registry.tsx:16-17` の id で名前空間を切って出す（`.asset-tags .root` / `.export-notes .root`）。

**残余**

- `input` の `font-size` が写せない（上の共通罠の表を見ること）。決定が要る
- `parts` の並びは**製品レイアウトではない**。`chrome.tsx` が1枚の画面ではなく部品の集まりなので、
  見るために並べただけ。ここに意味を読まないこと
- `.parts-row` だけが移植元に無い構造クラス（幅しか持たない `vSplitter` に交差軸の高さを与えるため）
- 面は 980x650 固定。`.shell` が `productStyles.ts:4` で `minWidth: 980, minHeight: 650` を持つので、
  これより小さい面を渡すと最小値が勝って `chromeModalScrim` の矩形とPNGの矩形がずれる

### C7 / C8 共通 — React Native は HTML ではない

`View`/`Text`/`Pressable`/`ScrollView`/`TextInput` は div/span/button ではない。機械的に置換しないこと。

| 罠 | 内容 |
|---|---|
| **`flexDirection` の既定が逆** | **RNは`column`、CSSは`row`。**写し間違えるとsilentに崩れる |
| カスケードと継承 | RNの`StyleSheet`には**無い**。CSSには**ある**。意図しない継承が起きていないか確認する |
| **`fontSize` を書いていない葉は写せない** | C8実測。`AssetTaggingPanel.tsx:43-52` の `input` は `fontSize` を持たないため、CSS側はUA既定の16pxを継承し、`heading`(14px)より大きく出る。RNは継承が無く、`TextInput` の既定はプラットフォーム側にある。**その値はリポジトリのどこにも書かれていないので写せない。** 埋めるには決定が要る |
| 単位 | RNの数値は密度非依存ピクセル |
| `FlatList` | 仮想化リスト。BlitzのDOM天井は1,500〜3,000ノードなので作り直しが要る（C6の担当） |
| **`zIndex` を写すと座標が1フレーム遅れる** | Blitz側の順序の問題であってCSSの書き方の問題ではない。静止画1枚だと原点に落ちて見える（[P13](reviews/2026-08-15-blitz-ui-runtime-probe.md#p13--z-index-を持つ要素は間違った場所に描かれるのではなく座標が1レイアウト遅れる2026-08-15追測)）。**絵が崩れてもCSSを書き換えて直そうとしないこと** |

---

## 発注順

```
C1 ──── C2 ──── C3 ──┐
C6 ──────────────────┤
C7 ──┬───────────────┴── C5(最後)
C8 ──┘

C4(ドッキング) は退役。egui側の責任。
```

**C6 は C1〜C3 と独立**（file-disjoint）で、かつ**完成条件を直接動かす**ため優先度が高い。

**C7・C8 は C1/C6 と file-disjoint なので並列可**で、かつ**C5の前提**である
（Inspectorとchromeの移植先が無いまま`rn_product_host`を消すと製品が消える）。
C5は全部の後。
同じshared seatを触る複数PRを同時発注しない（[AGENTS.md](../AGENTS.md)）。
