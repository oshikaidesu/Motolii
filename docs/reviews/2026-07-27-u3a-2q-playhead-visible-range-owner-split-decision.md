# U3a-2Q playhead / visible range owner 採択の分割判断

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2Q: **DONE**

## 1. 目的と非目標

`U3a-2P` §7.4 が委ねた「playhead と visible range を**同一粒で扱うか分割するか**」だけを、§3 五層と §6 八不変規則だけで判定し、docs-only で閉じる。本粒は **owner を決めない**。

非目標は本決定 §8 STOP・§9 負例と同義である。[U3a-2P playhead visible range範囲決定](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §1 の非目標一覧を参照する（`U3a-2P` の非目標を緩めない）。

## 2. authority から引いた事実

- **A1**（[U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §7 entry gate 項目 4）: playhead と visible range を同一粒で扱うか分割するかを、§3 の 5 層と §6 の 8 不変規則だけで判定でき、production pointer 入力・`TimelineHit` production caller・製品 window 結合を前提にしない。
- **A2**（同 §12.3）: §6 の不変規則 8 項目は、playhead と visible range を同一粒で扱うか分割するかの判定材料そのものになる。
- **A3**（同 §6）: §6.1 は playhead の単一 owner + read-only 投影。§6.2 は selection / playhead の window ごと非複製。§6.5 は window / DPI / monitor は Document 外。§6.7 は `timeline_projection` は viewport を caller 注入で受け取り owned range を所有しない。§6.6 は両者共通。§6.3 / §6.4 は React 非正本・描画 surface owner（§4 により state owner 判断へ使わない）。
- **A4**（同 §4 admissibility 表）: detachable 契約は不変規則として使ってよいが層 owner の決定には使わない。`Timeline scroll/zoom = Project session` は既決分類の事実として使ってよいが playhead 外挿・visible range 値 shape / default / 復元には使わない。
- **A5**（同 §10 N5）: 既決の `Timeline scroll/zoom = Project session` を別層へ移す、または playhead / visible range へ外挿して既決と書くことを禁じる。
- **A6**（同 §11 表）: `U3a-2Q` を **DO**（owner 採択 docs 粒）としていた。本粒は §7.4 の判定結果として採択を分割へ差し替える。
- **A9**（[docs/reviews/README.md](../reviews/README.md) 登録規則 3）: 状態語彙の固定集合は **決定 / 縮小採用 / 延期 / 棄却 / 撤回 / 未統一 / 観察 / 比較中 / 停止線** のみ。

## 3. 分割判定（§3 五層 + §6 八不変だけ）

| 対象 | §6 で名指しされた state owner 形の不変（単一 owner／非複製） | §6 で名指しされたその他の事実 | §6 が与えていないもの |
|---|---|---|---|
| playhead | §6.1（単一 owner + 全 window revision snapshot read-only 投影）、§6.2（window ごとに複製しない） | §6.3 React 非正本・§6.4 描画 surface owner（**いずれも §4 により state owner 判断へ使わない**）、§6.6、§6.8 | 五層のどの行が state owner かは §3 が未割当 |
| visible range | **なし** | §6.5（window / DPI / monitor は Document 外という一般則）、§6.7（`timeline_projection` は viewport を caller 注入で受け取り owned range を所有しない）、§6.6、§6.8 | 単一 owner／非複製の不変、五層の owner 割当 |

判定材料は [U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §3 五層と §6 八不変だけである（A1）。§3 五層は playhead にも visible range にも owner を割り当てていない（同 §3 末尾）ため、§3 は両者を区別しない。したがって判定は §6 だけで決まる。

§6 において、state owner 形の不変（単一 owner／window 毎に非複製）を名指しで持つのは playhead のみ（§6.1、§6.2）。visible range は §6 のどの項目でも単一 owner／非複製を与えられていない。§6.5 は window / DPI / monitor に関する一般則、§6.7 は viewport の caller 注入と owned range 非所有であり、どちらも visible range の state owner を与えない。§6.6 は両者共通、§6.8 は Document / Transient 一般則で両者を区別しない。

よって §6 の証拠 coverage は非対称であり、同一粒で両者を扱えば visible range 側は §6 の裏づけなしに owner を書くことになり [U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §9.1・§9.2・N1 に抵触する。分割は §3 + §6 だけから一意に導ける。

## 4. 決定

playhead owner 判断と visible range owner 判断を独立した後続粒へ分割する。

## 5. 本粒で決めないこと（未決を維持する）

- playhead の state owner、visible range の state owner、lifetime、保存、値 shape、default、復元規則、serialization、公開 API、`DomainIntent`、製品 surface、production 入力、後続粒の結論。
- visible range を **window-side / session-scoped / Workspace / Project session 等へ暗黙分類しない**。
- [U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §4 admissibility 表の意味を変更しない。

## 6. 後続粒

| ID | 状態 | 内容 |
|---|---|---|
| `U3a-2Q-P` | **DO** | playhead owner の **admissibility / evidence の狭い補遺** docs 粒。`U3a-2P` §3 五層閉集合・§4 admissibility・§6 不変規則の内側だけで、playhead owner 判断に使える証拠面を補う。**owner 自体を決めるとは本粒で約束しない**。state shape / default / serialization / 製品 surface は束ねない |
| `U3a-2Q-V` | **WAIT** | visible range owner。解除条件は **actual consumer surface evidence の成立**（`U3a-2P` §7.4 が前提にしないとした production pointer 入力・`TimelineHit` production caller・製品 window 結合が、別粒で実際に成立すること） |
| `U3a-2` 本体 | WAIT | 据え置き（`U3a-2P` §11 のまま） |
| `CU-106P` / `CU-106F` / `U2h-1P` | WAIT | 実 consumer surface 待ち（据え置き） |
| `CU-0A08BT` / `CU-0A08IT` / `U2c-2` | WAIT | 既存依存待ち（据え置き） |

PRODUCT-ASSET lane の `DO` は `U3a-2Q-P` **ただ 1 件**である。

## 7. 前段 REJECT / STOP の受理記録

(a) 先行の owner 採択 order を主担当 Codex が REJECT した。(b) 再設計 order を Opus 5 が `ORDER: STOP` とした。(c) 理由は [U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §4 が現 admissible evidence から特定層 owner を導出することを禁じ、§10 N5 が `Timeline scroll/zoom` 分類を playhead / visible range へ既決として外挿することを禁じるため。(d) ループ外 Fable 5 への再相談は誤読を訂正しこの STOP と分割案を支持したが、**助言は根拠ではない**（[AGENTS.md](../../AGENTS.md) の外部出力の扱い）。外部 model の助言を authority として引かない。

## 8. STOP 条件

1. 本粒を閉じるために playhead または visible range の owner、値 shape、default、lifetime、復元規則、serialization を決める必要が出た。
2. §3 の非対称表を埋めるために、`U3a-2P` §6 の 8 項目に**無い**不変規則を新設・言い換え・拡大解釈する必要が出た。
3. `U3a-2P` §6.3 / §6.4（React 非正本・描画 surface owner）を **state owner の証拠**として使わないと分割が正当化できない、と見えた。
4. 既決の `Timeline scroll/zoom = Project session` を playhead / visible range へ外挿する、または別層へ移す必要が出た（N5 抵触）。
5. `U3a-2P` §4 admissibility 表の行の意味を変更・追加・削除しないと文章が成立しない。
6. 分割の根拠として authority の**節番号または表の行**で裏づけられない事実を書く必要が出た。
7. 外部 model（Fable 5 / Grok / その他）の助言を authority・根拠として引用しないと結論が立たない。
8. `U3a-2Q-V` を `WAIT` のままにできない、または PRODUCT-ASSET lane の `DO` が `U3a-2Q-P` 以外にも生じる。
9. 本 order で与えた以外の ID を採番しないと handoff が書けない、または親名 `U3a-2` / `CU-105` / `CU-106` で closed order を作れると書きたくなった。
10. `U3a-2P` 本文（§3〜§10・§12 を含む）、`U3a-2Z`、`U3a-2A`、`U3a-2R`、`U3a-2S`、`CU-105R` を変更しないと整合が取れない。
11. 歴史 receipt（各 decision 文書の判定語・状態・決定内容・順序、`docs/implementation-ledger.md` §発注依存証跡の既存行）を書き換えないと整合が取れない。
12. `./scripts/check-docs.sh` が緑にならず、**索引・期待値・guard 側・golden・固定 hash を書き換えれば通る**と見えた。
13. `crates/**` / `ui/**` / `docs/mocks-ui/**` / `docs/spikes/**` / fixture / bench / golden / lockfile を変更したくなった、または `npm install` を実行したくなった。
14. lint 抑制、`#[allow]`、dummy caller、test-only accessor、production caller、pointer 入力の新設が要る。
15. 公開 API、`DomainIntent`、Document / journal / Undo / plugin 契約、serde 面、永続形式の追加・変更が要る。
16. 外部製品（Rerun を含む）を根拠・再利用箇所・変更案に含めたくなった。
17. ALLOWED_FILE 外の file を 1 byte でも変更する必要が出た。
18. mirror 8 面のうち一部だけを更新して残りを古いまま残すことになった（部分適用）。

## 9. 必須負例 N1〜N12

- **N1**: 五層のいずれかを playhead または visible range の owner として決める、推奨する、第一候補・暫定既定と書く。
- **N2**: visible range を window-side / session-scoped / Workspace profile / Project session / Transient 等へ分類する、または「visible range は Document 外だから ◯◯ 層」と推論する。
- **N3**: state shape、default、lifetime、保存、復元規則、serialization、serde default、永続 workspace / session 形式を書く。
- **N4**: `U3a-2P` §6 に無い不変規則を新設する、§6 の 8 項目の番号・語を書き換える、または §3 五層閉集合の行を増減・統合する。
- **N5**: 既決の `Timeline scroll/zoom = Project session` を playhead / visible range へ外挿して既決と書く、または別層へ移す。
- **N6**: surface owner（native Rust/wgpu が rail / bar / key / playhead を描画所有）や React 非正本を **state owner の証拠**として使う。
- **N7**: `U3a-2Q-P` / `U3a-2Q-V` の結論を先取りする、`U3a-2Q-V` を `DO` にする、または PRODUCT-ASSET lane の `DO` を 2 件以上にする。
- **N8**: 本 order で与えた以外の ID を採番する、親名 `U3a-2` / `CU-105` / `CU-106` で closed order を作れると書く。
- **N9**: `U3a-2P` 本文、`U3a-2Z` / `U3a-2A` / `U3a-2R` / `U3a-2S` / `CU-105R` 本文、または `docs/implementation-ledger.md` §発注依存証跡の既存行を書き換える（= 歴史 receipt の改変）。
- **N10**: mirror 8 面のいずれかだけを更新して他を古いまま残す、`docs/README.md` のファイルマップへ行を足す、`docs/reviews/README.md` の既存索引行を並べ替える、または索引検査を迂回する。
- **N11**: fps / ms / MB / 件数の合否閾値、有意性判定、製品公約、renderer 勝者、egui baseline 削除、G0-9D 閉集合変更、production pointer 入力、`TimelineHit` production caller、`CU-106P` / `CU-106F` / `U2h-1P` の実装または `DO` 昇格を書く。
- **N12**: 外部製品（Rerun を含む）または外部 model の助言を根拠・再利用箇所・変更案として引く、`docs/mocks-ui` の literal / catalog ID / label から欠落意味を推測する、`docs/reviews/README.md` 規則 3 の固定語彙外の状態語を使う。

## 10. 建設的所見（非拘束）

1. 将来の `U3a-2Q-P` は `U3a-2P` §4 の意味を直接書き換えず、新しい補遺文書の admissibility 表へ**行を足すだけ**で、本 §3 の playhead 行と接続できる。
2. 将来の `U3a-2Q-V` は actual consumer surface evidence 成立後、本 §3 の visible range 行へ**行を足すだけ**で同型の判定を再利用できる。
3. §3 の非対称表を先に固定しておくと、後続粒が「§6 のどの番号が効くか」だけを議論でき、owner 採択と分割判断の混線を防げる。
4. PRODUCT-ASSET lane の `DO` を `U3a-2Q-P` 1 件に限定した handoff は、§6 coverage 非対称の帰結として機械的に読める。
5. `U3a-2Z` §3 (5) 行の「playhead / range owner は未決」は本粒後も真のまま据え置きである（owner は選ばない）。
