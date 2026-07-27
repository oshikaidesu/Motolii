# U3a-2Q-P4 playhead 五層 state owner 採択

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2Q-P4: **DONE**

## 1. 目的と非目標

docs-only の一粒として、[U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §3 の五層閉集合に対し、
[U3a-2Q-P2](2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md) §4（現行 fresh-open no-restore）と
[U3a-2Q-P3](2026-07-27-u3a-2q-p3-playhead-future-restore-posture-decision.md) §4（将来 best-effort reopen 復元は延期・追加可能）を
正の寿命証拠として当て、五行の一意導出テスト `T2` で editor playhead の **state owner** を一層だけ採択する。

非目標は closed order §5 と同義である。playhead の具体値、初期位置の具体値、state shape、default、
serialization、serde default、永続 workspace / session 形式、version、未知 field 原本保全規則、reset 手順、
破損 fallback の実装、配送経路、公開 API、`DomainIntent`、Document 意味、journal、Undo / history、
plugin 契約、Transport の seek / seed、`PlaybackCounters`、`FramePlan.timeline_time`、scrub 中の中間値寿命、
Cancel 意味、visible range owner、`U3a-2Q-V` の状態変更・結論の先取り、製品 surface、product window、
production pointer 入力、`TimelineHit` production caller、`CU-106P` / `CU-106F` / `U2h-1P` の実装または `DO` 昇格、
semantic zoom 段階の中身、renderer 再判定、egui baseline / fixture / spike の削除、
`U3a-2P` §3 / §4 / §6、`U3a-2Q` §3、`U3a-2Q-P` E1〜E4 / `T1`、`U3a-2Q-P2` §4〜§5、
`U3a-2Q-P3` §4〜§5、G0-2 §2.2 の意味・行・番号・語の変更、歴史 receipt の改変、Rust / UI / fixture / golden の変更。

## 2. authority から引いた事実

1. **C4 / G0-2 §2.2 五層表**（[M3着手前決定 G0-2](2026-07-16-m3-preflight-decisions.md#22-状態の持ち場と寿命)）は、Document 行の寿命を「project保存と同じ」、User settings 行を「user単位、projectをまたいで保存」、Workspace profile 行を「user単位。壊れた場合に既定へ全reset可能」、Project session 行を「project identity単位のbest-effort cache。欠落・破損時は安全な既定へ戻せる」、Transient 行の寿命を「event/session内だけ」、同行の Undo/Document 欄を「保存しない。Cancel時変更ゼロ」とする。同節末尾は「U0bでは分類とdomain型だけを作り、永続化形式を発明しない」とする。
2. **C5 / U3a-2Q-P2 §4** は、fresh な Host coordinator instance が project identity を open した時、editor playhead は以前の値を復元せず、観測値はその project から決定的に定まる安全な初期位置とする。同 §4 は detach / re-dock / close / 再表示 / 同一 coordinator 内の surface 再生成を再 open とみなさない。同 §5 は将来復元時も安全な初期位置を欠落・破損時 fallback として保つとする。
3. **C6 / U3a-2Q-P3 §4** は、best-effort reopen 復元を延期・追加可能とし恒久棄却しないと決定した。同 §5 は五層各行を P2 §4 と本決定 §4 の二時点の寿命へ照らし `T2` で一層だけが残るかを検証し、`T2` で一層に定まらなければ owner を裁定せず STOP する。同 §6 が本粒の entry gate である。
4. **C7 / U3a-2Q-P §5** の `T1` は、P2 / P3 以前の証拠面（E1〜E4）だけでは owner を一意に導けないと判定した。本粒はこの判定を書き換えない。
5. **C8 / U3a-2P §6** は、単一 owner、read-only 投影、window 複製禁止、React 非正本、surface owner ≠ state owner、semantic zoom 前後保持、`timeline_projection` は caller 注入、D2 single writer の 8 不変規則を owner 未割当のまま既に明示している。

## 3. `T2` 五行表

| 層 | G0-2 §2.2 寿命欄の語 | (a) `U3a-2Q-P2` §4 現行 fresh-open no-restore との整合 | (b) `U3a-2Q-P3` §4 将来 best-effort 復元の追加可能性との整合 | 採否と根拠節 |
|---|---|---|---|---|
| Document | project保存と同じ | 矛盾（寿命「project保存と同じ」は project 保存周期で値が読み戻される意味であり、P2 §4 の「以前の値を復元しない」と両立しない） | 矛盾（寿命「project保存と同じ」は作品保存と一体の読み戻しを意味し、P3 §4 の層定義を変えずに足す best-effort reopen 復元と同義にできない） | 除外。G0-2 §2.2 Document 行 |
| User settings | user単位、projectをまたいで保存 | 矛盾（寿命は user 単位で project をまたぐ保存であり、P2 §4 の「その project から決定的に定まる安全な初期位置」と両立しない） | 矛盾（寿命「user単位、projectをまたいで保存」は project identity をまたぐ保持であり、P3 §4 の project 単位 best-effort 復元を層定義を変えず追加する意味と一致しない） | 除外。G0-2 §2.2 User settings 行 |
| Workspace profile | user単位。壊れた場合に既定へ全reset可能 | 矛盾（寿命「user単位」は user 単位の保持であり、P2 §4 の project identity open 時にその project から決定的に定まる安全な初期位置と両立しない） | 矛盾（寿命は user 単位で project identity 横断の保持であり、P3 §4 の project 単位 best-effort reopen 復元を層定義を変えず追加できない） | 除外。G0-2 §2.2 Workspace profile 行 |
| Project session | project identity単位のbest-effort cache。欠落・破損時は安全な既定へ戻せる | 整合（P2 §4 の fresh open で以前値を復元しないことは、best-effort cache の欠落・破損時に安全な既定へ戻す寿命と矛盾しない） | 整合（P3 §4 の将来 best-effort 復元は、層定義を変えず Project session の best-effort cache として追加可能） | 残存。`U3a-2Q-P2` §4、`U3a-2Q-P3` §4 |
| Transient | event/session内だけ（保存しない。Cancel時変更ゼロ） | 整合（P2 §4 の fresh open で以前値を復元しないことは、寿命「event/session内だけ」「保存しない」と両立する） | 矛盾（寿命「event/session内だけ」「保存しない」は reopen をまたぐ best-effort 復元を層定義を変えず追加できない） | 除外。`U3a-2Q-P3` §4 |

## 4. `T2` 判定手順と結果

1. 候補集合が [U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §3 の五層閉集合ちょうどであることを確認した（第六層を足さない、統合しない、削らない）。
2. 各行について (a) を判定した。判定根拠は G0-2 §2.2 寿命欄の語と `U3a-2Q-P2` §4 の 2 文だけとした。
3. 各行について (b) を判定した。判定根拠は G0-2 §2.2 寿命欄の語と `U3a-2Q-P3` §4 の 2 文、および `U3a-2Q-P3` §8 N4（層定義の変更禁止）だけとした。
4. `残存` 行数を数えた → **1 行**（`Project session` のみ）。
5. 残存が厳密に 1 行であるため、§5 でその層を採択する。

## 5. 決定

editor playhead の **state owner** は **Project session** 層とする。

本決定は値・初期位置・state shape・serialization・永続形式・version・公開 API・scrub / Cancel 意味・
visible range owner・製品 surface・caller を決めない。

## 6. 境界と論理的帰結

[U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §6 の不変規則 1〜8 をそのまま維持する（本文は書き写さない）。

surface owner（native Rust/wgpu module が time rail / bar / key / playhead の描画 surface を所有）は、
本粒が採択した **Project session** state owner とは別概念である（U3a-2P §6.4）。

[U3a-2Q-P2](2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md) §4 の現行 fresh-open no-restore と、
[U3a-2Q-P3](2026-07-27-u3a-2q-p3-playhead-future-restore-posture-decision.md) §4 の将来 best-effort 復元の延期・追加可能 posture の両方を、
**Project session** 採択は同時に満たす。現行 v1 は P2 どおり fresh Host coordinator の project open で以前の playhead を復元せず、
将来の best-effort 復元は P3 どおり層定義を変えず追加可能な延期事項として残る。本採択は復元機能の採択・実装予約ではない。

## 7. STOP 条件

1. `T2` の残存候補が 0 層になった。
2. `T2` の残存候補が 2 層以上になった。
3. 上記 1 または 2 の状況で、なお owner を採択するには authority 外の意味補完が必要になった。
4. 除外根拠を書くために code / caller / field / schema / API の不在を使う必要が出た。
5. 採択を書くために値、初期位置、state shape、serialization、永続形式、version、公開 API、restore codec、scrub / Cancel 意味を決める必要が出た。
6. `U3a-2P` §3 / §4 / §6、`U3a-2Q` §3、`U3a-2Q-P` E1〜E4 / `T1`、`U3a-2Q-P2` §4〜§5、`U3a-2Q-P3` §4〜§5、G0-2 §2.2 の意味・行・番号・語を変更しないと文章が成立しない。
7. `docs/implementation-ledger.md`「発注依存証跡」の既存行、または他 decision 文書の判定語・状態・決定内容・順序という歴史 receipt を書き換えないと整合しない。
8. 新 ID を採番しないと §9 handoff が書けない、または PRODUCT-ASSET lane の `DO` が 1 件以上必要になった。
9. `U3a-2Q-V` を `WAIT` のままにできない、または visible range owner に触れる必要が出た。
10. `crates/**` / `ui/**` / `docs/mocks-ui/**` / `docs/mocks/**` / `spikes/**` / fixture / bench / golden / lockfile / package.json を変更したくなった。
11. `./scripts/check-docs.sh` が緑にならず、索引・期待値・guard 側を書き換えれば通ると見えた。
12. `cargo test --locked --workspace` が赤で、docs-only 差分と因果を説明できない。
13. ALLOWED_FILE 外の file を変更する必要が出た。
14. 会話履歴、外部製品、または別 model の助言が無いと order を実行できないと見えた。

## 8. 必須負例

- **N1**: `T2` の残存が 1 層でないのに owner を採択する、または「第一候補」「暫定既定」「有力」「事実上」と書いて実質採択する。
- **N2**: code / caller / field / schema / API の不在を層の排除証明にする。
- **N3**: runtime owner（Host coordinator）または surface owner（native Rust/wgpu module）を五層の state owner と同一視する、あるいは第六層として表へ足す。
- **N4**: Transport の再生中クロック owner を paused / scrub / editor playhead の state owner へ外挿する。
- **N5**: 既決の `Timeline scroll/zoom = Project session` を playhead へ外挿して根拠にする、または別層へ移す。
- **N6**: G0-2 §2.2 の層定義（特に Project session と Transient の寿命欄）を書き換える、要約で置き換える、第六層を足す。
- **N7**: 値、初期位置の具体値、state shape、default、serialization、serde default、永続 workspace / session 形式、version、restore codec、公開 API、`DomainIntent` を書く。
- **N8**: 採択を「将来復元機能の採択」「実装予約」と書く、または現行 v1 が以前の playhead を復元すると書いて `U3a-2Q-P2` §4 を上書きする。
- **N9**: detach / re-dock / close / 再表示 / surface 再生成を project 再 open と同一視する。
- **N10**: `docs/implementation-ledger.md`「発注依存証跡」の既存行を書き換える、mirror 9 面のいずれかだけを更新して他を古いまま残す。
- **N11**: 新 ID を採番する、`U3a-2Q-V` を `DO` にする、PRODUCT-ASSET lane の `DO` を 1 件以上にする。
- **N12**: 外部製品または外部 model の助言を根拠にする、guard 側の期待値・固定 hash・golden・fixture を書き換える。

## 9. 次の最小粒

| ID | 状態 | 内容 |
|---|---|---|
| `U3a-2Q-V` | `WAIT` | visible range owner。actual consumer surface evidence 待ち（据え置き） |
| `U3a-2` 本体 | `WAIT` | 製品 window / consumer 入力待ち（据え置き） |
| `CU-106P` / `CU-106F` / `U2h-1P` | `WAIT` | 実 consumer surface 待ち（据え置き） |
| `CU-0A08BT` / `CU-0A08IT` / `U2c-2` | `WAIT` | 既存依存待ち（据え置き） |

PRODUCT-ASSET lane の `DO` は本粒完了後 0 件である（`U3a-2Q-V` は `WAIT` のまま）。新 ID を本粒で採番しない。
