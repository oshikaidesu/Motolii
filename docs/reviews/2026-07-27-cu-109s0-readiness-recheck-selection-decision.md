# CU-109S0 CU-109 readiness / order-boundary recheck 選定

- 日付: 2026-07-27
- 状態: **決定**
- CU-109S0: **DONE**

## 1. 目的

`U3a-2Q-P4` 完了後の rolling 再判定として、VS-1 の未完了 blocker である
`CU-109` の実装を直接起動せず、先に必要な
**Undo / Redo prepared-action 順序の再確認**だけを `CU-109S` として
PRODUCT-ASSET lane の次 `DO` に選定する。

本粒は `CU-109S` の結論、`CU-109` / `CU-110` / `CU-111` の実装順、
prepared-action の型・API、poison の具体実装を決めない。

## 2. authority と現行事実

1. [CU-G03 edit durability / publish 順序決定](2026-07-26-cu-g03-edit-durability-ordering-decision.md)
   §9 は、`CU-109` が同決定を直接 authority にできる一方、
   Undo / Redo prepared-action 順序を再確認するまで自動着手しないとする。
2. [VS-1 blocking table](2026-07-24-m3-vertical-slice-execution-decision.md#4-最初の製品完成線-vs-1)は、
   `CU-G03` を `DONE`、`CU-109` を `WAIT` とし、runtime 配線と prepared-action 順序を未完了とする。
3. [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md)の `CU-109` 行は、
   `CU-G03D`、`CU-G03R`、`U2b`、`D1m` を依存とする。これらの現行完了証跡は
   [implementation ledger](../implementation-ledger.md#発注依存証跡)に揃っている。
4. `U3a-2Q-P4` 完了後の PRODUCT-ASSET `DO` は 0 件であり、
   ledger の運用規則は main 到達後に次 order を rolling に再判定する。
5. `G0-6H` は `U0e-3` と W0b 製品前提を止める人間審判であり、
   `CU-109S` の依存ではない。

## 3. 決定

次 PRODUCT-ASSET `DO` を docs-only `CU-109S` とする。

`CU-109S` は current code facts と既存 authority を照合し、次のどちらか一つだけを決める。

1. `CU-109` 実装を次 `DO` にできる。
2. `CU-109` より前に、`CU-111` が所有する prepared-action 順序だけを閉じる
   docs-only 前提粒が必要である。

どちらも一意に導けなければ、実装順を発明せず `STOP` とする。

## 4. 非目標

- Rust / JS / fixture / guard / golden の変更
- prepared-action の型、公開 API、private API、payload の決定
- Document、serde、journal、plugin 契約、永続形式の変更
- `Healthy / Poisoned` の具体 state、復旧 UI、再 open 規則の実装
- `CU-109` / `CU-110` / `CU-111` の実装または同一粒への束ね
- `U3a-2Q-V`、`CU-106P/F`、製品 window、G0-6H、U4a / VS-2 の状態変更
- code / caller / field / API の不在を順序の肯定証拠にすること

## 5. `CU-109S` entry gate

1. candidate は「`CU-109` 実装を次にする」または
   「prepared-action 順序の docs-only 前提を先にする」の二つだけ。
2. `CU-G03D/R`、D1m、D2、U2b-1、CU-104/E と current code factsを再照合する。
3. 既存の責任分離を変更せず、次 PRODUCT-ASSET `DO` を一件だけ残す。
4. 結論に typed action shape、API、永続 payload が必要なら `STOP` する。
5. 歴史 receipt の既存行を変更しない。

## 6. 必須負例

- **N1**: 本粒だけで `CU-109` 実装を ready と裁定する。
- **N2**: dependency が `DONE` というだけで prepared-action 順序の再確認を省略する。
- **N3**: `CU-111` 所有の prepared-action 内容を `CU-109S` で設計する。
- **N4**: G0-6H または製品 window を `CU-109S` の依存へ追加する。
- **N5**: CU-109/110/111 の複数実装を同時 `DO` にする。
- **N6**: 不在証拠、外部 model の助言、旧粒度化の候補分類だけで順序を決める。

## 7. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-109S` | **DO** | Undo / Redo prepared-action 順序を再確認し、CU-109実装またはdocs-only順序前提のどちらを次にするか一意に裁定 |
| `CU-109` | **WAIT** | `CU-109S` の結論待ち |
| `CU-110` / `CU-111` | **WAIT** | 実装順を本粒で固定しない |
| `U3a-2Q-V` / `CU-106P/F` | **WAIT** | actual consumer surface evidence 待ち（据え置き） |

PRODUCT-ASSET lane の `DO` は `CU-109S` ただ一件とする。
