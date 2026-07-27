# U3a-2P playhead / visible range owner 判断の範囲決定

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2P: **DONE**

## 1. 目的と非目標

`U3a-2`（windowed native Timeline）に残る未決点「playhead / visible range を**どの層が所有するか**」について、**owner を決めずに**、後続の owner 採択粒が閉じた境界の内側で動くための docs 範囲だけを確定する。本粒は docs-only である。

本粒は **playhead / visible range の state owner を決めない**。state shape、default、復元規則、serialization も決めない。

非目標（§9 STOP・§10 負例と同義）:

- playhead / visible range の owner 決定、第一候補・推奨・暫定既定の提示。
- state shape、default、復元規則、serialization、serde default、永続 workspace / session 形式。
- 公開 API、`DomainIntent`、Document 意味、journal、Undo / history、plugin 契約の変更。
- semantic zoom の段階の中身・閾値・切替条件・描画内容。
- renderer 再判定、`direct_vello` / `egui_vello` の再評価、egui baseline / fixture / spike の削除、絶対性能閾値・合否基準・製品公約の新設。
- production pointer 入力、`TimelineHit` production caller、`CU-106P` / `CU-106F` / `U2h-1P` の実装または `DO` 昇格。
- W0b / H1b / Motolii Studio Preview / 通常製品 window 結合 / Distribution Ready / **G0-9D** の解禁または閉集合変更。
- Rust / JS / JSX / CSS / fixture / bench / golden / visual 期待値 / spike raw / manifest / lockfile / package.json / `docs/mocks-ui/**` / `crates/**` / `ui/**` / `spikes/**` の変更、`npm install` の実行。
- `U3a-2S` / `U3a-2S-R2` / `U3a-2S-R3` / `U3a-2R` / `U3a-2Z` / `U3a-2A` / `CU-105R` / `CU-106S` / `U2h-1PR` の決定内容・状態・順序・負例の書き換え（current mirror 1 行の同期のみ許可）。
- `U3a-2Q` 以外の新 ID 採番、親名 `U3a-2` / `CU-105` / `CU-106` での closed order 化。
- 外部製品（Rerun を含む）を根拠・再利用箇所・変更案に含めること。
- 隣接チケットへの拡張、TODO stub、部分適用（mirror を片方だけ更新して終える）。

## 2. authority から引いた事実

### B1 五層分類（G0-2 §2.2）

[M3着手前決定 G0-2](2026-07-16-m3-preflight-decisions.md#22-状態の持ち場と寿命) §2.2 は、代表状態を Document / User settings / Workspace profile / Project session / Transient の 5 層へ分類する。`Timeline scroll/zoom` は **Project session** 行に、`hover` / `focus` / `drag preview` 等は **Transient** 行に置く。同節は「U0b では分類と domain 型だけを作り、永続化形式を発明しない」とする。

### B2 playhead / range は owner 未決（CU-105R §3、U3a-2Z §3・§5、U3a-2A §8）

[CU-105R](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) §3 表の最終行は **playhead / range 不変 = `STOP`（owner 未決）** とする。
[U3a-2Z](2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md) §3 (5) 行は「selection / playhead / range | Core 意味 + Host coordinator（selection）；playhead / range owner は未決 | `STOP`」とする。同 §5 は playhead / visible range の owner と不変規則を **未決** のまま残す。
[U3a-2A](2026-07-27-u3a-2a-renderer-adoption-decision.md) §8 は playhead / visible range owner、viewport 値 shape / default / 復元規則を未決とし、§11 は次の `DO` を `U3a-2P` だけとする。

### B3 単一 owner と read-only 投影（detachable 契約、decision-index）

[detachable panel / multi-window 契約](2026-07-22-m3-detachable-panel-window-contract.md) は selection / playhead / Graph / Timeline channel を window ごとに複製せず、全 window が Host の同じ revision 付き snapshot を read-only 投影するとする（本文・§「両windowが同じsnapshot revision、stable selection、playheadを読む」）。
[decision-index](../decision-index.md) `panel detach re-dock multi-window …` 行は「Host snapshot/selection/**playhead は単一 owner**で全 window が read-only 投影。window/DPI/monitor は Document 外」とする。

### B4 surface owner と React 非正本（decision-index UI runtime 行、ui-runtime-architecture）

[decision-index](../decision-index.md) `UI runtime責任境界` 行と [UI runtime 責任境界](../ui-runtime-architecture.md) は、time/Z 軸へ row 同期する rail・bar・key・**playhead**、高頻度 scrub を **native Rust/wgpu module** が所有する。React 製品 package は playback / playhead / selection の**正本を持たない**（typed command intent のみ）。Document は D2 single writer、Transient selection/session は Host coordinator だけが所有する。

### B5 visible range layout は bundled Host module（軸分離 §3）

[軸分離](2026-07-22-m3-surface-extension-axis-separation.md) §3 は、**bundled Host module** = visible range **layout**、semantic zoom、hit-test、native 描画、gesture adapter とする。**Core** = RationalTime、identity、projection 入力、selection 意味、typed intent、D2 command、Undo/Cancel とする。

### B6 headless projection は caller 注入（U3a-2Z §3 (3)、U3a-1 分割 §7）

[U3a-2Z](2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md) §3 (3) 行は `motolii-ui::timeline_projection` を caller 注入 viewport・metrics とし、selection / playhead / owned range を所有しないとする。
[U3a-1 owner/visibility分割](2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md#7-完了証跡u3a-1i) §7 は headless `U3a-1I` を `DONE` とする。

### B7 selection consumer 到達性（CU-106S §1〜§3、§5）

[CU-106S](2026-07-27-cu-106-selection-consumer-split-decision.md) §1〜§3、§5 は、`TimelineHit` の production caller 0、pointer 入力不在。`CU-106P` / `CU-106F` は実 consumer surface まで `WAIT` とする。

### B8 G0-9 停止継続（G0-9 段階化 §7）

[G0-9 段階化](2026-07-23-m3-g0-9-staged-platform-gates.md) §7 は W0b / H1b / Motolii Studio Preview / 通常製品 window 結合 / **G0-9D** / egui baseline と fixture の削除 / Document・journal・plugin ABI・永続 layout 形式を引き続き停止とする。

### B9 処分語彙（依存優先ゲート §3）

[依存優先ゲート](2026-07-24-dependency-first-responsibility-gate.md) §3 の処分語彙は `PASS / REDUCE / STOP` と `FROZEN / DELETE-LATER / KEEP-AS-EVIDENCE` のみとする。

### B10 台帳事実（implementation-ledger）

[implementation-ledger](../implementation-ledger.md)「現在の並列レーン」の `U3a-2P` 行は `DO`、「発注依存証跡」の `U3a-2A` 行は `DONE` とする（BASE_SHA 時点の台帳事実。本粒完了後に mirror 同期する）。

## 3. candidate 五層閉集合表

「Host coordinator」「native module」「React」は**層ではなく配送・描画の責任**である（B3〜B5）。下表は candidate **state owner** の閉集合として五層だけを列挙する。採否印・推奨・第一候補は付けない。

| 層 | 定義（G0-2 §2.2 の語のまま） | 該当する既決事実 |
|---|---|---|
| Document | layer、clip、parameter、接続、camera等の作品意味。project保存と同じ。D2 command、Undo対象 | G0-2 §2.2 表1行目。作品意味は D2 single writer（B4） |
| User settings | keymap delta、UI scale、theme、reduce motion、resource policy。user単位、projectをまたいで保存。対象外。Document/journalへ入れない | G0-2 §2.2 表2行目 |
| Workspace profile | panel開閉・幅、Timeline density等の作業配置。user単位。壊れた場合に既定へ全reset可能。対象外。projectの作品意味にしない | G0-2 §2.2 表3行目。P48/P49 は Workspace-session **候補**（interaction prototype P48・P49 行） |
| Project session | Stage View pan/zoom/fit、**Timeline scroll/zoom**、選択中panel等。project identity単位のbest-effort cache。対象外。export/evalへ寄与しない | G0-2 §2.2 表4行目。`Timeline scroll/zoom` の層分類は既決（B1）。値 shape / default / 復元規則は未決（U3a-2Z §5） |
| Transient | hover、focus、drag preview、connection picking、popup、IME preedit。event/session内だけ。保存しない。Cancel時変更ゼロ | G0-2 §2.2 表5行目。Transient selection/session は Host coordinator だけが所有（B4） |

playhead / visible range の **state owner** は上記のどの行にも authority が割り当てていない（B2）。

## 4. 証拠 admissibility 表

| 証拠 | owner 判断へ使ってよい | owner 判断へ使ってはならない |
|---|---|---|
| G0-2 §2.2 五層表 | はい（候補閉集合の出典） | 特定層への owner 割当の導出 |
| detachable / multi-window 契約（単一 owner + read-only 投影） | はい（不変規則） | どの層が単一 owner かの決定 |
| decision-index UI runtime 行 / `ui-runtime-architecture.md` | はい（**surface owner** と React 非正本） | surface owner を **state owner** と同一視すること |
| 軸分離 §3（visible range layout は bundled Host module） | はい（**layout** 責任） | visible range **値の state owner** の決定 |
| CU-105R §3 / U3a-2Z §5 / U3a-2A §8 | はい（未決である事実） | 未決を「既定」で埋める根拠 |
| M3 仕様 U0b / interaction prototype P48・P49（Timeline scroll/zoom = Project session） | はい（既決分類の事実） | visible range の値 shape / default / 復元規則の決定、playhead への外挿 |
| `timeline-bench` 1k / 100k、visual parity spike、G0-9L manifest、CU-0G02B raw | いいえ | owner 判断の合格証拠 |
| React モック（`docs/mocks-ui`）の literal・catalog ID・label | いいえ | 欠落意味の推測 |
| 外部製品（Rerun を含む） | いいえ | Motolii authority 外 |

## 5. 判断 owner

[U3a-2R](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) §5(a) を踏襲する。

### (a) 判断 owner

PRODUCT-ASSET lane の `U3a-2` 系 docs 粒（主担当 Codex）。実装粒 / spike / Grok 検収 / 外部 model へ委譲しない。

**surface owner**（native Rust/wgpu module が rail / bar / key / playhead の描画 surface を所有）は、**state owner**（五層のどれが playhead / visible range の値を正本として持つか）とは別概念である（B4・B5）。

## 6. owner 未割当のまま既に明示されている不変規則

新しい不変規則は発明しない。authority が既に明示しているものだけを列挙する。

1. playhead は**単一 owner**であり、全 window / 全 surface はその revision 付き snapshot の **read-only 投影**である（detachable 契約、decision-index `panel detach …` 行）。
2. selection / playhead / Graph / Timeline channel を window ごとに**複製しない**（detachable 契約本文）。
3. React 製品 package は playback / playhead / selection の**正本を持たない**（typed command intent のみ）（decision-index UI runtime 行、`ui-runtime-architecture.md`）。
4. time rail / bar / key / playhead / 高頻度 scrub の**描画 surface** は `motolii-ui` 内 native Rust/wgpu module が所有する（state owner の決定ではない）（decision-index UI runtime 行）。
5. window / DPI / monitor は Document 外（decision-index `panel detach …` 行）。
6. semantic zoom の段階境界前後で selection identity・playhead・visible range を保つ。density pixel を Document object identity にしない（U3a-2Z §2 A1、interaction prototype ledger §2 LD-8 / §5 U3a 行を A1 が引用）。
7. `motolii-ui::timeline_projection`（headless、U3a-1I `DONE`）は viewport を **caller 注入**で受け取り、selection / playhead / owned range を所有しない（U3a-2Z §3 (3) 行）。
8. Document 書き込みは D2 single writer のみ。Transient selection / session は Host coordinator だけが所有する（decision-index UI runtime 行、`ui-runtime-architecture.md`）。

## 7. owner 採択粒（`U3a-2Q`）の entry gate

次を**すべて**満たす場合に限り `U3a-2Q` を起票する。一つでも欠けたら起票しない。

1. 本決定（`U3a-2P`）が `DONE` であり、§3 五層閉集合・§4 admissibility・§6 不変規則が BASE_SHA 事実として参照可能。
2. `docs/implementation-ledger.md`「発注依存証跡」に `U3a-2P` の一意な `DONE` 行が存在する。
3. PRODUCT-ASSET lane の `DO` が `U3a-2Q` ただ 1 件である。
4. playhead と visible range を**同一粒で扱うか分割するか**を、§3 の 5 層と §6 の 8 不変規則だけで判定でき、production pointer 入力・`TimelineHit` production caller・製品 window 結合を前提にしない。
5. owner を書くために新しい serde 面、永続 workspace/session 形式、公開 API を必要としない。

## 8. 未決として残す点

- playhead の state owner、visible range の state owner、両者を同一層に置くか否か。
- 値 shape / default / 復元規則 / serialization、scrub 中の中間値の寿命。
- production pointer 入力と `TimelineHit` production caller、CU-106P/F・U2h-1P 入場。
- semantic zoom 段階の中身、G0-9D。

## 9. STOP 条件

1. 範囲を閉じるために playhead / visible range の owner、値 shape、default、復元規則、serialization を決める必要が出た。
2. 五層閉集合のどれかに「もっともらしい既定」「第一候補」「暫定」を付けないと §3 が閉じない。
3. authority の**節番号または表の行**で裏づけられない事実を書く必要が出た。
4. surface owner（native Rust/wgpu）を state owner と同一視しないと文章が成立しない。
5. 既決の `Timeline scroll/zoom = Project session` を別層へ移す、または playhead へ外挿する必要が出た。
6. 新しい恒久 workspace / session 形式、serde default、公開 API、`DomainIntent`、Document / journal / Undo / plugin 契約の追加・変更が要る。
7. production caller、pointer 入力、lint 抑制、`#[allow]`、dummy caller、test-only accessor の新設が要る。
8. `crates/**` / `ui/**` / `docs/mocks-ui/**` / `docs/spikes/**` / fixture / bench / golden / lockfile を変更したくなった、または `npm install` を実行したくなった。
9. renderer 勝者・絶対閾値・egui baseline 削除・G0-9D 閉集合・W0b / H1b / 製品 window / Distribution Ready の解禁に触れる必要が出た。
10. `./scripts/check-docs.sh` が緑にならず、**索引・期待値・guard 側・golden を書き換えれば通る**と見えた。
11. PRODUCT-ASSET lane の `DO` が 2 件以上になる、または親名 `U3a-2` / `CU-105` / `CU-106` で closed order を作れると書きたくなった。
12. `U3a-2Q` 以外の新 ID を採番しないと handoff が書けないと見えた。
13. mirror の一部だけを更新して残りを古いまま残すことになった、または既存決定の意味・状態・順序を書き換えないと整合しない。
14. 外部製品を根拠・再利用箇所・変更案に含めたくなった。
15. ALLOWED_FILE 外の file を 1 byte でも変更する必要が出た。

## 10. 必須負例 N1〜N12

- **N1**: 五層のいずれかを playhead または visible range の owner として決める、推奨する、暫定既定・第一候補と書く。
- **N2**: state shape、default、復元規則、serialization、serde default、永続 workspace/session 形式を書く。
- **N3**: 五層閉集合の行を増やす・減らす・統合する、または「Host coordinator」「native module」「React」を第 6 層以降として表へ足す。
- **N4**: **surface owner**（native Rust/wgpu が rail / bar / key / playhead を描画所有）を **state owner の決定**として書く、または time surface を React 製品 package の責任へ移す。
- **N5**: 既決の `Timeline scroll/zoom = Project session` を別層へ移す、または同分類を playhead / visible range へ外挿して既決と書く。
- **N6**: `timeline-bench` 1k / 100k、`g0-9-timeline-visual-parity`、`g0-10-multi-surface-window`、G0-9L manifest、CU-0G02B raw を owner 判断の合格証拠へ昇格させる。
- **N7**: fps / ms / MB / 件数の合否閾値、有意性判定、製品公約を新規に書く、または renderer 勝者・egui baseline 削除・G0-9D 閉集合変更を書く。
- **N8**: production pointer 入力、`TimelineHit` production caller、`CU-106P` / `CU-106F` / `U2h-1P` の実装または `DO` 昇格を書く。
- **N9**: mirror 8 面（reviews/README、docs/README、decision-index、implementation-ledger、M3 spec、縦 slice、CU-106S、U2h-1P）のいずれかだけを更新して他を古いまま残す、または既存決定の意味・状態・順序・歴史 receipt を書き換える。
- **N10**: PRODUCT-ASSET lane の `DO` を 2 件以上にする、`U3a-2Q` 以外の ID を採番する、または親名 `U3a-2` / `CU-105` / `CU-106` で closed order を作れると書く。
- **N11**: `docs/mocks-ui` を現行実装として更新する、`npm install` を実行する、guard 側の期待値・固定 hash・golden を書き換える、`docs/reviews/README.md` の索引検査を迂回する。
- **N12**: 外部製品（Rerun を含む）を根拠・再利用箇所・変更案に含める、または `docs/mocks-ui` の literal / catalog ID / label から欠落意味を推測する。

## 11. 次の最小粒

| ID | 状態 | 内容 |
|---|---|---|
| `U3a-2Q` | **DO** | playhead / visible range **owner 採択** docs 粒。本決定 §3 五層閉集合・§4 admissibility・§6 不変規則・§7 entry gate の内側だけで owner を決める。state shape / default / serialization は束ねない |
| `U3a-2` 本体 | WAIT | windowed 実装は範囲・責任・採択・owner の docs 閉包後、かつ製品 window / consumer 入力の成立後 |
| `CU-106P` / `CU-106F` / `U2h-1P` | WAIT | 実 consumer surface 待ち（据え置き） |
| `CU-0A08BT` / `CU-0A08IT` / `U2c-2` | WAIT | 既存依存待ち（据え置き） |

## 12. 建設的所見（非拘束）

1. §3 の五層表を **G0-2 §2.2 の語をそのまま**使う 1 列＋既決事実 1 列に固定しておくと、`U3a-2Q` は「どの行に印を付けるか」だけの判断になり、層の再定義圧力を先に断てる。
2. **surface owner と state owner を §5 で明示的に分離**しておくことが本粒の最大の価値である。decision-index の UI runtime 行は「native が playhead を所有する」と読めるため、この 1 文がないと `U3a-2Q` が「native module = state owner」と短絡する事故が起きやすい。
3. §6 の不変規則 8 項目は、playhead と visible range を**同一粒で扱うか分割するか**の判定材料そのものになる。番号を固定しておけば `U3a-2Q` は「N 番だけで決まる」形で書ける。
4. §4 admissibility を U3a-2R §4 と同じ 3 列にしておくと、将来 CU-106P / CU-106F の consumer 粒が**行追加だけ**で再利用できる。
5. `U3a-2Z` §3 の 5 列責任所在表は本粒で壊さないこと。`U3a-2Q` は同表 (5) 行の「未決」を書き換える 1 行差分で閉じられる形が最小である。
