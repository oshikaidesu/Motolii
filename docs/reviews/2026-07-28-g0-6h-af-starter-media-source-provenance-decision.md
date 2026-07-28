# G0-6H-AF Starter Media 媒体源・provenance class 裁定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-AF: **DONE**

## 目的

`G0-6H-A` A-5 / A-6 が残した媒体源と provenance class を、二択でちょうど1件裁定する。byte、path、schema、command は閉じない。

## 比較（ちょうど2案）

| 案 | 内容 | 採否 |
|---|---|---|
| (i) 決定的生成 | fixture byte を repo 内の決定的手続きで生成する | **採択** |
| (ii) pinned vendoring | 第三者 media を固定版として repo または lock 経由で持ち込む | **本fixtureに限り棄却** |

## AF-1 採択（決定的生成）

本 bounded fixture の媒体源は決定的生成とする。理由は次の4点だけである。

(a) 第三者 media の licensing 責任を持ち込まない、(b) capture 時・test 時の remote 供給を成立条件にしない（`G0-6H-A` A-5 と一致）、(c) byte が再現可能かつ監査可能になる（`G0-6H-A` A-6 と一致）、(d) npm または外部 media を runtime authority にしない。

## AF-2 棄却の範囲限定

(ii) pinned vendoring の棄却は **本 `Starter Media` fixture に限る**。repo全体で pinned vendoring を禁止しない。既存の vendoring 判断、`references.md` の候補、依存優先ゲートの `ADOPT / EXTERNAL` 処分を本粒で撤回・変更しない。

## AF-3 後続生成物への要求（要求のみ。実現手段は決めない）

後続粒が生成する出力は次をすべて満たすこと。

(1) 固定された local fixture-only byte であること、(2) provenance が文書から追える監査可能な形で固定されること、(3) read-only の完全性検査を持つこと（検査の失敗時は何も変更せず停止する）、(4) 生成・検証・capture・test のいずれの時点でも network を要求しないこと。

## AF-4 決定論の主張範囲

本粒は **cross-platform の byte 決定性を主張しない**。同一 byte の再現可能範囲は、generator / toolchain 契約が成立した後の粒で初めて宣言できる。本粒で OS 横断・toolchain 横断の byte 一致を約束する記述を書かない。

## AF-5 責任処分の位置づけ

本粒は依存優先・責任最小化ゲートの `RESPONSIBILITY DISPOSITION` を確定しない。generator の `REUSE / ADOPT / WRAP / EXTERNAL / BUILD / REJECT` と `RETIREMENT` の裁定は `G0-6H-AG0` が行う。「決定的生成を採る」ことを「自作 generator を BUILD する」と読み替えない。

## AF-6 停止線の継承

`G0-6H-A` A-3 / A-7 / A-8 をそのまま継承する。`Starter Media` は Project 外 fixture-only 源であり、Document / 製品runtime / 公開API / plugin契約 / 永続形式 / production Registered folder の正本にならない。label と opaque ID から欠落意味を補完しない。

## AF-7 `G0-6H-V0` の扱い

`G0-6H-V0` は `WAIT` のまま。本粒は implementation ledger の状態語を変更しない。

## React / Browser authority（参照のみ・本粒は差分0）

- **対象面**: product-owned React module `DiscoveryBrowserCandidate` の `Media` surface。移管契約は [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)。UI runtime境界は [ui-runtime-architecture.md](../ui-runtime-architecture.md)（Browser は bundled first-party Host module）。
- **SOURCE ASSET**: 固定 commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`（`ui/motolii-web/source-provenance.json`）。本粒は closure を読むだけで1 byteも変更しない。

## 確定しないこと

具体path、file名、codec、寸法、尺、byte数、manifest schema、hash algorithm、生成command、package / tool の正確なversion、route / query shape、adapter API、media byte、実装file。

## 非目標

- media byte、file、fixture、asset の生成・取得・追加・commit。
- 具体path、file名、codec、寸法、尺、byte数、manifest schema、hash algorithm、生成command、package / tool の正確なversion、route / query shape、adapter API、公開APIの決定。
- generator の実装、script、CI 配線、npm script 追加。
- `G0-6H-AG0` が所有する tooling 分類・`RESPONSIBILITY DISPOSITION`・`RETIREMENT` の先取り。
- repo全体での pinned vendoring 禁止、既存 vendoring / 依存採否の撤回。
- token値、theme、threshold、golden、期待値、component、iconの選定・変更。
- `Project` および production `Registered folders` の意味の新設・変更・拡張。
- 現行route実装、route名、入場条件、`docs/mocks-ui/README.md`、`src/main.jsx`、hash fixture の変更。
- `docs/implementation-ledger.md` の変更（Codex統合が所有する）。
- 隣接チケット（`CU-107*` / `CU-110*` / `CU-111` / `U3a-*` / `U2h-*` / `G0-9*` / `U0e-*` / `CU-0B0*` / `G0-6H-V0`）への波及。

## 必須負例

- §ALLOWED_FILE 以外の file を変更・追加・削除する。
- 4 file のうち一部だけを変更する部分適用、または TODO スタブで裁定を置き換える。
- 第3案（hybrid、両採択、保留、later決定）を書く、または二択のどちらも採択しない。
- pinned vendoring を repo 全体で禁止する、既存の vendoring / 依存採否を撤回する。
- 具体path、file名、codec、寸法、尺、byte数、manifest schema、hash algorithm、生成command、package / tool version、route / query、adapter API、media byte を1つでも確定する。
- generator の実装file、script、npm script、CI job を作る、または生成物を commit する。
- cross-platform / OS横断 / toolchain横断の byte 決定性を主張する。
- `G0-6H-AG0` の tooling 分類、`RESPONSIBILITY DISPOSITION`、`RETIREMENT` を先取りして書く。
- `Starter Media` を Project asset、Document、公開API、plugin、永続形式、production Registered folder の正本として扱う記述。
- label または opaque ID から欠落意味を推測して補う。
- `reference-handoff.md` の既存節（固定証拠、再現コマンド、自動report、5秒課題表、Decision template、既存5注記）を変更・削除・並べ替えする。
- Decision template の `未記入` または checklist の `[ ]` を本粒で埋める。
- 承認済みnormal 5画面、`check-reference` 成功、Git ancestry を、visual parity・人間承認・route同一性・empty-project成立の根拠とする。
- `G0-6H` / `G0-6H-V0` / `CU-0B01` / `CU-0B02` / `U0e-3` の状態語を本粒で変更する。
- `docs/implementation-ledger.md` を本ticket差分に含める。
- reviews索引未登録のまま新規文書を置く、相対リンク切れを残す、`decision-index.md` に未定義の状態語彙を書く。
- lint / test 抑制、期待値・golden・threshold・fixture special-case の追加・変更、生JSON/文字列走査による型付き境界の迂回、公開raw API、重複planner/helper の新設。
- `G0-6H-AG0` 以外の後続粒を新設する、または次の一粒を2件以上起票する。

## 次の一粒（ちょうど1件）

docs-only **`G0-6H-AG0`** — **generator / output closure inventory**。既存 repo tooling を分類し、依存優先・責任最小化ゲートの `REUSE / ADOPT / WRAP / EXTERNAL / BUILD / REJECT` から1つと `RETIREMENT` を選ぶ docs-only 粒。media byte を生成しない。

## handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-A` | **DONE** | 前提。scenario / fixture所有契約 |
| `G0-6H-A0` | **DONE** | 前提。選択肢(a)とStarter Media方向の受領 |
| `G0-6H-AF` | **DONE** | 本粒。媒体源・provenance class の二択裁定（byteなし） |
| `G0-6H-AG0` | **DO** | generator / output closure inventory と責任処分（byteなし） |
| `G0-6H-V0` | **WAIT** | 本契約のCodex統合まで維持 |
| `G0-6H` | **DO / HUMAN** | 据え置き |

## 関連

- [G0-6H-A scenario / fixture契約](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)
- [G0-6H-A0 選定](2026-07-28-g0-6h-a0-empty-project-starter-media-selection.md)
- [依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)
- [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)
- [ui-runtime-architecture](../ui-runtime-architecture.md)
- [reference handoff](../mocks-ui/reference-handoff.md)
