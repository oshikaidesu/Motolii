# CU-0A08RM0 Browser typed-intent 依存裁定範囲の選定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08RM0: **DONE**

## 1. 目的

`CU-0A08RM` の Opus `ORDER: STOP` が示した Browser typed-intent 依存の authority 未裁定を、docs-only 裁定粒 `CU-0A08RMD` が閉じる一問へ限定する。本粒はその問いに**答えない**。

## 2. 事実

- `CU-0A08RS0` と `CU-0A08RS` は [発注依存証跡](../implementation-ledger.md#発注依存証跡) で `DONE` である。
- `CU-0A08RS` は、VS-1 read-only projection に `U4a-2` が load-bearing ではないと裁定したが、`CU-0A08BT` / `CU-0A08IT` の `WAIT` と依存セルは変更していない。
- `CU-0A08BP`、`CU-101`、`CU-102` は発注依存証跡で `DONE` である。
- `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` は `CU-107N` が定義した閉集合であり、本粒はこれらを発注依存証跡の `DONE` 行として扱わない。
- `CU-0A08RM` は Opus `ORDER: STOP`（Browser typed-intent 依存の authority 未裁定）により `WAIT` である。
- 着手時点の PRODUCT-ASSET lane で状態が完全一致 `DO` の行は `CU-0A08RM0` の1件だけであった。

## 3. CU-0A08RMD が閉じる唯一の問い

以下は次粒の問いであり、本粒の回答ではない。

`CU-0A08BT` の typed-intent 半分は VS-1 Rectangle Place を表し、したがって既決の `CU-101` / `CU-102` と `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-110` の連鎖に依存するのか、それとも `U4a-2` に依存し続ける別の Direct-entry 責任なのか。

## 4. 可能な候補（優劣を付けない）

**(A)** `CU-0A08BT` typed-intent 半分は VS-1 Rectangle Place を表し、既決の `CU-101` / `CU-102` と `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-110` の連鎖に依存すると次粒で裁定する案。

**(B)** それは別の Direct-entry 責任であり、`U4a-2` Direct 製品入口依存が残ると次粒で裁定する案。

## 5. 非目標

- §3 の問いへ答える、候補へ重み付けする、または実質的な選好を示すこと。
- `CU-0A08BT` / `CU-0A08IT` の `WAIT`、依存セル、責任分割を変更すること。
- `U4a-2`、`CU-107*`、`CU-110` の意味、順序、完了条件を変更すること。
- event shape、WebView wire、typed intent の型・名前・payload、drag payload、公開 API、Document、journal、plugin 契約、永続形式を決めること。
- Rust / JS / JSX / CSS / fixture / guard / schema / golden を変更すること。

## 6. 必須負例

- 候補 (A)/(B) の推奨、採用、優劣を記す。
- §3 の一問を増やす。
- `CU-0A08RM` の `WAIT` または STOP 証跡を消す。
- `CU-0A08BT` / `CU-0A08IT` の行を変更する。
- PRODUCT-ASSET `DO` を0件または2件以上にする。
- `CU-0A08RMD` を発注依存証跡へ追加する。
- allowlist 外の file、guard期待値、除外、lint抑制を変更する。

## 7. 同期した current mirror

次の7箇所を、`CU-0A08RM0` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` は docs-only `CU-0A08RMD`（1件）という現在地へ同期した。

1. `docs/implementation-ledger.md` 現在地表 M3 行
2. `docs/implementation-ledger.md` M3への入場判定の運用判断
3. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` journal durability 行
4. 同文書 selection / Undo再投影 行
5. `docs/decision-index.md` M3 VS-1 縦slice 主題行
6. 同文書 CU-110S〜CU-107W 主題行
7. 同文書 CU-0A08RS0〜CU-0A08RMD 主題行

allowlist 外に未同期の current next-DO mirror は残していない。

## 8. STOP 条件

1. §3 の一問へ答えないと選定を記録できない。
2. `CU-0A08BT` / `CU-0A08IT`、`U4a-2`、W0/W1表またはM3仕様を変えないと整合しない。
3. 公開 API、Document、永続形式、journal、plugin 契約、event shape、typed intent への波及が必要になる。
4. allowlist 外に current next-DO mirror が見つかる。
5. PRODUCT-ASSET `DO` を `CU-0A08RMD` の1件に保てない。

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08RM0` | **DONE** | Browser typed-intent 依存の authority 未裁定を §3 の一問へ限定 |
| `CU-0A08RMD` | **DO** | §3 の一問だけを docs-only で閉じる |
| `CU-0A08RM` | **WAIT** | `CU-0A08RMD` 裁定後にのみ mirror 修復を再開 |
