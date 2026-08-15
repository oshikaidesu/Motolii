# Blitz移植の発注capsule

[Blitz移行起案](reviews/2026-08-15-blitz-ui-runtime-adoption-proposal.md)が**採択された場合**に
外部実装へ渡すためのclosed order capsule集。形式は[AGENTS.md](../AGENTS.md)の
`BASE / AUTHORITY / CURRENT STATE / OWNER / EXACT TARGET / ALLOWLIST / READ SET / ORACLES / NON-GOALS / RETURN`。

> **起案の裁定欄が空である間、本書のcapsuleを発注しない。**
> 本書は発注準備であって発注許可ではない。

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

## C4 — ドッキング移植

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | `egui_tiles` のツリーとD&D状態機械を移植し、配置結果をCSS flex/gridで描く |
| **ALLOWLIST** | `crates/motolii-ui/src/dock/**`（新規） |
| **READ SET** | `egui_tiles`(MIT OR Apache-2.0) の `tiles.rs` / `container/mod.rs` / `tile.rs` / `tree.rs` / `grid.rs` / `linear.rs` / `tabs.rs`、`docs/ui-interaction-language.md:75`（製品要件） |
| **POSITIVE ORACLE** | 分割・tab化・resize・表示/非表示・既定presetへのreset が動くこと（`ui-interaction-language.md:75` の文言そのもの） |
| **NEGATIVE ORACLE** | `egui` への依存が `dock/` に**0件**であること |
| **NON-GOALS** | 共通NON-GOALS全部 + **ドッキングの操作感を「改善」しない**（`egui_tiles` の挙動を写す） + ライセンス表記を落とさない |
| **RETURN** | `egui::Id`/`Context` のメモリストア置換で意味が変わりそうな箇所 → `RETURN` |

## C6 — Browserパネル（フォルダ参照でメディア入口を開く）

`N-MEDIA-PICK` / `N-PROJECT-ENTRY`（ファイルを選ぶ入口が無い）への代替。
**完成条件を直接動かす唯一のcapsule**。

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | フォルダを参照してサムネイル格子を出し、項目を掴んでドラッグできる |
| **ALLOWLIST** | `crates/motolii-ui/src/browser_blitz/**`（新規） |
| **READ SET** | `spikes/blitz-probe/src/bin/browser_panel.rs`（実証済み）、`docs/ui-interaction-language.md`（Browserの役割） |
| **POSITIVE ORACLE** | 元寸PNG 45枚を表示して60秒 panic しないこと |
| **NEGATIVE ORACLE** | `ImageManager` の `cache` と `texture_bindings` を**フレーム内で新規生成していない**こと（下記の罠） |
| **NON-GOALS** | 共通NON-GOALS全部 + **配置intentの意味を決めない**（drop先で何が起きるかは未決。RETURN） + native file dialog を作らない |
| **RETURN** | drop したとき Document に何が起きるべきかを決める必要が出たら`RETURN`。**これは製品意味の決定** |

### この2つは必ず踏む（実測で踏んだ）

1. **`ImageManager` の `cache` はフレームを跨いで保持する。** 毎フレーム新規生成すると
   画像が atlas へ再確保され続け、数秒で `AtlasLimitReached` で **panic** する。
   しかも `vello_hybrid/src/render/wgpu.rs:596` の `.unwrap()` なので**捕捉できない**
2. **`blitz_net::Provider` は Tokio reactor を要求する。** 無いと panic

## C5 — RN退役（**最後**）

C1〜C4がmainで動いた後にのみ着手する。

| 項目 | 内容 |
|---|---|
| **EXACT TARGET** | `rn_product_host/`（実装8,247 + テスト7,273）と `host_bridge/`(4,488) の削除 |
| **NON-GOALS** | 共通NON-GOALS全部 + **他のcapsuleと同時に発注しない** + `timeline_skia/` を巻き込んで消さない |
| **RETURN** | 削除すると失われるoracleが見つかったら`RETURN`。**代替を自分で書かない** |

---

## 発注順

```
C1 ──┬── C2 ──┬── C3
     │        │
     └── C4 ──┘
C6 ──────────────┴── C5(最後)
```

**C6 は C1〜C4 と独立**（file-disjoint）で、かつ**完成条件を直接動かす**ため優先度が高い。

C1とC4は独立（file-disjoint）なので並列可。C5は全部の後。
同じshared seatを触る複数PRを同時発注しない（[AGENTS.md](../AGENTS.md)）。
