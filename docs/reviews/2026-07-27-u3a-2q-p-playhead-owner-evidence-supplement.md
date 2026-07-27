# U3a-2Q-P playhead owner 判断の admissibility / evidence 補遺

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2Q-P: **DONE**

## 1. 目的と非目標

docs-only の一粒として、[U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §4 admissibility 表の**意味を一切変更せず**、playhead owner 判断へ追加で使える証拠面と、使ってはならない推論を、同形 3 列の**新しい表**として本補遺に足す。[U3a-2Q](2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md) §3 表の **playhead 行**へ接続する。§5 の一意導出テスト `T1` を実行し、その結果に従って owner 採択を行うか未決維持かを決める。

本粒は **owner を決めるとは約束しない**。

非目標は本 order §7 と同義である。

- playhead / visible range の state owner の決定、推奨、第一候補、暫定既定、候補層の絞り込み・除外の宣言。
- state shape、default、lifetime、保存、復元規則、reopen policy、serialization、serde default、永続 workspace / session 形式。
- 公開 API、`DomainIntent`、Document 意味、journal、Undo / history、plugin 契約、product surface、pointer caller の追加・変更。
- visible range owner の取り扱い、`U3a-2Q-V` の状態変更・結論の先取り。
- `U3a-2P` §3 五層閉集合・§4 admissibility 表・§6 不変規則 8 項目の意味・行・番号・語の変更。
- semantic zoom の段階の中身、renderer 再判定、egui baseline / fixture / spike の削除、絶対性能閾値・合否基準・製品公約の新設。
- production pointer 入力、`TimelineHit` production caller、`CU-106P` / `CU-106F` / `U2h-1P` の実装または `DO` 昇格。
- W0b / H1b / Motolii Studio Preview / 通常製品 window 結合 / Distribution Ready / G0-9D の解禁または閉集合変更。
- Rust / JS / JSX / CSS / fixture / bench / golden / guard / lockfile / package.json / `docs/mocks-ui/**` / `crates/**` / `ui/**` / `spikes/**` の変更、`npm install` の実行。
- `U3a-2Q-P2` 以外の新 ID 採番、親名 `U3a-2` / `CU-105` / `CU-106` での closed order 化。
- 外部製品（Rerun を含む）または外部 model の助言を根拠・再利用箇所・変更案として引くこと。
- 隣接チケットへの拡張、TODO stub、部分適用。

## 2. authority から引いた事実

- **F4**（[快適利用 work-map](2026-07-22-m3-comfortable-use-work-map.md) §7「状態所有の再確認」表、playhead 行）: 「**playhead | runtime ownerはHost coordinator。再open時の永続化は未決であり、仕様判断前にProject sessionへ焼かない**」。同表の他行は `Timeline scroll/zoom、作業中のview | Project session` 等であり、playhead 行はそれらと別行である。
- **F5**（[detachable panel / multi-window 契約](2026-07-22-m3-detachable-panel-window-contract.md) §1〜§4）: §1 は Host coordinator を revision 付き snapshot / selection / focus intent の唯一の owner とし、detach した top-level が Document、Undo、selection、playhead、Graph/Timeline channel を複製しない、全 window は Host の同じ revision 付き snapshot を read-only 投影する。§2 は Document revision / selection / playhead を Host coordinator 状態として `WindowId` / DPI / monitor 等の OS session 状態と分ける。§3 は detach を placement と projection target の切り替えとし、re-dock で同じ panel を二つの writer として残さない。§4 の合格条件は両 window が同じ snapshot revision、stable selection、playhead を読む、projection owner 1 を含む。
- **F6**（`crates/motolii-transport/src/lib.rs`）: module doc は再生位置の正本 = デバイスへ供給済みサンプル数、映像は常に最新の聴感時刻のみレンダする。`FramePlan.timeline_time` は聴感タイムライン時刻。`Transport` は単一の再生ヘッドでクロック所有者はここだけ。`perceptual_time()` → `next_frame_plan()` が `timeline_time` を導く。同 crate 配下で `seek` / `scrub` / `paused` の出現は 0 件。
- **F7**（[M3-ui-integration](../specs/M3-ui-integration.md) L194、U2b 行）: U2b-2 Place の GUI command 意味として top-level compatible selection、first Track top fallback、canonical Y-up、Rect size 0.2、**playhead〜composition end** を固定する。同行は playhead を Place の**入力**として使うだけで、playhead の state owner を決めていない。
- **F8**（[M3着手前決定 G0-2](2026-07-16-m3-preflight-decisions.md) §2.2 表）: Project session 行の寿命欄は project identity 単位の best-effort cache。Transient 行は**保存しない**、Cancel 時変更ゼロ。同節末尾は U0b では分類と domain 型だけを作り、永続化形式を発明しない。
- **F9**（[U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §3 / §4 / §6）: §3 は五層 candidate 閉集合を採否印なしで列挙し、playhead / visible range の state owner はどの行にも authority が割り当てていない。§4 は 3 列 admissibility 表。§6 は owner 未割当のまま既に明示されている不変規則 8 項目。
- **F10**（[U3a-2Q](2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md) §3 / §6 / §10.1）: §3 表 playhead 行は §6.1・§6.2 を state owner 形の不変として持ち、五層のどの行が state owner かは §6 が与えていないもの列に置く。§6 は `U3a-2Q-P` を `DO`、`U3a-2Q-V` を `WAIT` とする。§10.1 は将来の `U3a-2Q-P` が `U3a-2P` §4 を直接書き換えず補遺 admissibility 表へ行を足すだけで本 §3 playhead 行と接続できるとする。
- **F11**（[UI runtime 責任境界](../ui-runtime-architecture.md)）: L58 は React control が typed command intent だけを Host へ送り、playback、playhead、selection の正本を持たない。L188 は time/Z 軸に同期する rail・bar・key・playhead は native。L198 は Transient selection / session は Host coordinator だけが所有する。

## 3. 追加 admissibility 表（新規 4 行）

[U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §4 の既存行は写経・再掲・改変しない。

| 証拠 | owner 判断へ使ってよい | owner 判断へ使ってはならない |
|---|---|---|
| E1: work-map §7 playhead 行 | **runtime owner = Host coordinator** という配送・協調責任の既決事実、および「再open 時の永続化は未決」「仕様判断前に Project session へ焼かない」という**明示的な停止**の事実 | runtime owner を五層のどれかの **state owner** と同一視すること。playhead を Project session へ割り当てること |
| E2: detachable 契約 §1〜§4 | 単一 owner・全 window read-only 投影・window ごと複製禁止・projection owner 1 という**不変**（`U3a-2P` §6.1 / §6.2 と同一内容） | どの層が単一 owner かの決定。Host coordinator 状態という語から五層割当を導くこと |
| E3: `crates/motolii-transport/src/lib.rs` | **再生中**のクロック正本が Transport の供給済み audio frame であり `FramePlan.timeline_time` を導く、という code 事実。同 crate に paused seek / scrub / reopen / 永続 / Undo owner が**存在しない**という不在事実 | Transport clock owner を paused / editor playhead の五層 state owner へ外挿すること。不在を「その層では持たない」証明として使うこと |
| E4: M3 仕様 U2b 行（`playhead〜composition end`） | Rectangle Place が playhead を **入力**として読む、という既決事実 | playhead が Document である／Document でない、の決定。Place が読むことから owner を導くこと |

## 4. `U3a-2Q` §3 playhead 行への接続

[U3a-2Q](2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md) §3 の既存 2 行（playhead / visible range）は書き換えない。下表は playhead 行へ**行追加**する接続だけを示す。

| 追加証拠 | `state owner 形の不変` 列へ足す内容 | `その他の事実` 列へ足す内容 | `§6 が与えていないもの` 列へ足す内容 |
|---|---|---|---|
| E1 | （列の既存 §6.1 / §6.2 不変は変更しない） | runtime owner = Host coordinator（F4）。再 open 時永続化は未決、Project session へ焼かない停止（F4） | 五層 state owner 割当は依然 §3 未割当（F9）。E1 は runtime 配送責任のみで層を名指ししない |
| E2 | §6.1 / §6.2 と同内容の不変を detachable §1〜§4 で裏づけ（F5） | detachable §2〜§4 の projection / re-dock 事実（F5） | 五層のどの行が単一 state owner かは E2 だけでは決まらない（F5・F9） |
| E3 | （§6.1 / §6.2 の playhead 不変は Transport 再生中 clock とは別軸） | 再生中 clock 正本 = Transport・`timeline_time` 導出（F6）。paused / scrub / reopen / 永続 / Undo owner の code 不在（F6） | paused / editor playhead の五層 owner は E3 から導けない（F6） |
| E4 | （列の既存不変は変更しない） | Place が playhead を入力として読む（F7） | Place 入力事実から五層 owner は導けない（F7・F9） |

## 5. 一意導出テスト `T1` と結果

手順（closed order §5）:

1. `U3a-2P` §3 の五層のうち、authority が playhead に**行として**割り当てているものを数える → **0 件**（F9）。
2. `U3a-2P` §6 と E1〜E4 のうち、**特定の 1 層**を名指しで playhead の state owner とするものを数える → **0 件**。E1 が名指しするのは Host coordinator であり、五層に含まれない配送・協調責任（F4・F11）。E2 は不変のみ。E3 は再生中 clock のみ。E4 は consumer 入力のみ（F5〜F7）。
3. E1〜E4 はいずれも五層の特定行を state owner として名指ししないことを確認 → **成立**。
4. F4 が「**再open 時の永続化は未決**」「仕様判断前に Project session へ焼かない」と明示していることを確認 → **成立**。
5. 判定: 五層への直接割当 0 件、追加証拠にも特定層の名指しなし、永続化は明示的に未決。特定層の採択・除外は authority 外の意味を補うことになり、[U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §9.1・§10 N1 に抵触する。

**結果: owner 採択は authority だけからは一意に導けない。** 本粒は owner を採択せず、未決を維持する。

## 6. 明示的に禁止する推論（4 件）

- **N1 同義**（F4・F9・F11）: runtime owner（Host coordinator）を `U3a-2P` §3 五層の **state owner** と同一視する、または Host coordinator を第 6 層として表へ足す。
- **N2 同義**（F6）: Transport の再生中クロック owner を、paused / scrub / editor playhead の五層 state owner へ外挿する。
- **N3 同義**（F8・F9）: 「現行 Document schema に field が無い」を Document 恒久不採用の証明として書く、または任意の不在事実を層の排除証明へ昇格させる。
- **N4 同義**（F4・F8・F9）: 既決の `Timeline scroll/zoom = Project session` を playhead へ外挿する、または別層へ移す。

## 7. 本粒で決めないこと

- playhead / visible range の state owner の決定、推奨、第一候補、暫定既定、候補層の絞り込み・除外の宣言。
- state shape、default、lifetime、保存、復元規則、reopen policy、serialization、serde default、永続 workspace / session 形式。
- 公開 API、`DomainIntent`、Document 意味、journal、Undo / history、plugin 契約、product surface、pointer caller の追加・変更。
- visible range owner の取り扱い、`U3a-2Q-V` の状態変更・結論の先取り。
- `U3a-2P` §3 五層閉集合・§4 admissibility 表・§6 不変規則 8 項目の意味・行・番号・語の変更。
- semantic zoom の段階の中身、renderer 再判定、egui baseline / fixture / spike の削除、絶対性能閾値・合否基準・製品公約の新設。
- production pointer 入力、`TimelineHit` production caller、`CU-106P` / `CU-106F` / `U2h-1P` の実装または `DO` 昇格。
- W0b / H1b / Motolii Studio Preview / 通常製品 window 結合 / Distribution Ready / G0-9D の解禁または閉集合変更。
- Rust / JS / JSX / CSS / fixture / bench / golden / guard / lockfile / package.json / `docs/mocks-ui/**` / `crates/**` / `ui/**` / `spikes/**` の変更、`npm install` の実行。
- `U3a-2Q-P2` 以外の新 ID 採番、親名 `U3a-2` / `CU-105` / `CU-106` での closed order 化。
- 外部製品（Rerun を含む）または外部 model の助言を根拠・再利用箇所・変更案として引くこと。
- 隣接チケットへの拡張、TODO stub、部分適用。

## 8. STOP 条件

1. §5 の `T1` を実行した結果が「一意に導ける」となり、owner を採択できると見えた。
2. 本粒を閉じるために playhead の owner、値 shape、default、lifetime、reopen policy、復元規則、serialization を決める必要が出た。
3. authority の**節番号または表の行**で裏づけられない事実を書く必要が出た。
4. E1〜E4 の 4 行だけでは §4 の `U3a-2Q` §3 playhead 行への接続が書けず、5 行目以降の新証拠面を authority 外から持ち込む必要が出た。
5. `U3a-2P` §4 / §3 / §6 の意味・行・番号・語を変更しないと文章が成立しない。
6. `U3a-2Q` §3 の既存 2 行、または §8 で保護した 6 文書の本文を変更しないと整合が取れない。
7. 「発注依存証跡」の既存行、または歴史 receipt を書き換えないと整合が取れない。
8. `U3a-2Q-P2` 以外の ID を採番しないと handoff が書けない、または完了条件が 2 件以上必要になった。
9. `U3a-2Q-V` を `WAIT` のままにできない、または PRODUCT-ASSET lane の `DO` が 2 件以上になる。
10. 公開 API、`DomainIntent`、Document / journal / Undo / plugin 契約、serde 面、永続形式の追加・変更が要る。
11. lint 抑制、`#[allow]`、dummy caller、test-only accessor、production caller、pointer 入力の新設が要る。
12. `crates/**` / `ui/**` / `docs/mocks-ui/**` / `docs/spikes/**` / fixture / bench / golden / lockfile / package.json を変更したくなった、または `npm install` を実行したくなった。
13. `./scripts/check-docs.sh` または reference guard が緑にならず、**索引・期待値・guard 側・golden・固定 hash を書き換えれば通る**と見えた。
14. `cargo test --locked --workspace` が赤で、docs-only 差分と因果が説明できない。
15. ALLOWED_FILE 外の file を 1 byte でも変更する必要が出た。
16. 会話履歴、別 worktree、repo 横断の歴史調査、複数仕様の意味判断、未指定の公開境界探索、または外部 model の助言が無いと order を実行できないと見えた。

## 9. 必須負例 N1〜N14

- **N1**: runtime owner（Host coordinator）を `U3a-2P` §3 五層の **state owner** と同一視する、または Host coordinator を第 6 層として表へ足す。
- **N2**: Transport の再生中クロック owner を、paused / scrub / editor playhead の五層 state owner へ外挿する。
- **N3**: 「現行 Document schema に field が無い」を Document 恒久不採用の証明として書く、または任意の不在事実を層の排除証明へ昇格させる。
- **N4**: 既決の `Timeline scroll/zoom = Project session` を playhead へ外挿する、または別層へ移す。
- **N5**: 五層のいずれかを playhead の owner として決める、推奨する、第一候補・暫定既定・有力と書く、あるいは特定層を「除外済み」と断定して候補集合を狭める。
- **N6**: state shape、default、lifetime、保存、reopen policy、復元規則、serialization、serde default、永続 workspace / session 形式を書く。
- **N7**: `U3a-2P` §4 の既存行を書き換える・削除する・語を変える、`U3a-2P` §3 五層閉集合の行を増減統合する、`U3a-2P` §6 の 8 項目の番号・語を書き換える、または §6 に無い不変規則を新設する。
- **N8**: `U3a-2Q` §3 の既存 2 行、または `U3a-2P` / `U3a-2Z` / `U3a-2A` / `U3a-2R` / `U3a-2S` / `CU-105R` の本文を変更する。
- **N9**: `docs/implementation-ledger.md`「発注依存証跡」の既存行を書き換える（追加のみ許可）、または各 decision 文書の判定語・状態・決定内容・順序という歴史 receipt を改変する。
- **N10**: mirror 8 面のいずれかだけを更新して他を古いまま残す、`docs/reviews/README.md` の既存索引行を並べ替える・削除する、または索引検査を迂回する。
- **N11**: `U3a-2Q-V` を `DO` にする、その結論を先取りする、`U3a-2Q-P2` 以外の ID を採番する、PRODUCT-ASSET lane の `DO` を 2 件以上にする、または親名 `U3a-2` / `CU-105` / `CU-106` で closed order を作れると書く。
- **N12**: fps / ms / MB / 件数の合否閾値、有意性判定、製品公約、renderer 勝者、egui baseline 削除、G0-9D 閉集合変更、production pointer 入力、`TimelineHit` production caller、`CU-106P` / `CU-106F` / `U2h-1P` の実装または `DO` 昇格を書く。
- **N13**: `docs/mocks-ui` を現行実装として更新する、`npm install` を実行する、guard 側の期待値・固定 hash・golden・fixture を書き換える、または reference guard 実行後に `docs/mocks-ui/node_modules` symlink を残す。
- **N14**: 外部製品（Rerun を含む）または外部 model の助言を根拠・再利用箇所・変更案として引く、`docs/mocks-ui` の literal / catalog ID / label から欠落意味を推測する、`docs/reviews/README.md` 規則 3 の固定語彙外の状態語を使う、`docs/implementation-ledger.md` の状態語固定集合外の語を lane 表へ書く。

## 10. 次の最小粒と完了条件

| ID | 状態 | 内容 |
|---|---|---|
| `U3a-2Q-P2` | `DECIDE` | playhead の **reopen 時 lifetime** の仕様判断 docs 粒。work-map §7 playhead 行が「未決」と宣言した一点だけを閉じる。owner 採択・state shape / default / serialization・製品 surface は束ねない |
| `U3a-2Q-V` | `WAIT` | visible range owner。解除条件は actual consumer surface evidence の成立（[U3a-2Q](2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md) §6 のまま） |
| `U3a-2` 本体 | `WAIT` | 据え置き（`U3a-2P` §11 のまま） |
| `CU-106P` / `CU-106F` / `U2h-1P` | `WAIT` | 実 consumer surface 待ち（据え置き） |
| `CU-0A08BT` / `CU-0A08IT` / `U2c-2` | `WAIT` | 既存依存待ち（据え置き） |

**完了条件（`U3a-2Q-P2`、1 件のみ）**: 主担当 Codex（[U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §5(a) の判断 owner）が、playhead を再 open 時に復元するか安全な既定へ戻すかを、owner・state shape・default・serialization を同時決定せずに一意に裁定し、根拠・反対側負例・後続 owner 採択の entry gate を decision 文書へ記録すること。

PRODUCT-ASSET lane の `DO` は本粒完了後 **0 件**である（`U3a-2Q-V` は `WAIT` のまま）。
