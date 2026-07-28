# G0-6H-A empty Project + local Starter Media scenario / fixture 所有契約

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-A: **DONE**

## 目的

`G0-6H-A0`でユーザーが採択したempty Project + local `Starter Media`方向を、docs-onlyのscenario意味とfixture所有の停止線として裁定する。素材byte、path、schema、route、生成手段は本粒では閉じない。

## 裁定（A-1〜A-9）

### A-1 scenario semantic（Project側）

本scenarioにおいてProject assetsは**0件**であり、Stage / Inspector / Timelineに作品内容（object、layer、key、選択、再生位置以外の制作状態）を含まない。

### A-2 scenario semantic（Browser側）

Browserは非表示にせず、`Media` surface上に表示され続ける。Browserは「Projectが空である」ことと「Projectの外に参照可能なmediaがある」ことを同時に示す。

### A-3 source分離

Browserが表示するsampleは`Starter Media`という**Project外のregistered-folder源**として現れ、Project assetへ変換しない。`Starter Media`の項目はProject登録済み・Project所有として表示しない。[ui-interaction-language.md](../ui-interaction-language.md)のProject Asset / Registered folders統合（同文書:77）およびRegistered FoldersのWorkspace参照定義（同文書:154）を新設・変更・上書きしない。

### A-4 sample集合の許容範囲

sample setは静止画、短い動画、音声、SVGを**含んでよい**（許容であって必須構成の確定ではない）。件数、file名、拡張子、codec、解像度、尺、byte数を本粒で決めない。

### A-5 offline要求

capture時とtest時に外部networkを要求しない。実行環境のnetwork到達性を前提にした取得を成立条件に置かない。

### A-6 provenance要求

byte provenanceは固定かつ監査可能であること（同一byteが再現でき、由来が文書から追えること）を要求する。具体のhash algorithm、manifest schema、file配置、check commandは本粒で決めない。

### A-7 fixture所有の停止線

`Starter Media` fixtureは、Document、製品runtime、公開API、plugin契約、永続形式、**production Registered folderの正本**のいずれにもならない。fixture-only資産であり、製品のregistered-folder機能の意味決定に使わない。

### A-8 非推測規律

label（`Starter Media`、`PROJECT`、`AUDIO LIBRARY`等の表示文字）およびopaque ID（`data-file-root-select`値、`data-asset-source`値、asset名）から欠落意味を推測して補完しない。

### A-9 `G0-6H-V0`の扱い

`G0-6H-V0`は、本契約がCodexによって統合されるまで`WAIT`のままである。本粒はimplementation ledgerの状態語を変更しない。

## React / Browser authority（参照のみ・本粒は差分0）

- **対象面**: product-owned React module `DiscoveryBrowserCandidate` の `Media` surface（`ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`）。移管契約は [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)。UI runtime境界は [ui-runtime-architecture.md](../ui-runtime-architecture.md)（Browserはbundled first-party Host module）。
- **SOURCE ASSET**: 固定commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`（`ui/motolii-web/source-provenance.json`）。本粒はclosureを読むだけで1 byteも変更しない。
- **STATE OWNER（記録）**: Project assetsと配置確定はDocument（D2 single writer）。Registered foldersの登録等はWorkspace参照。`Starter Media` fixture byteはfixture-onlyで状態正本ではない。

## 確定しないこと

具体path、file名、manifest schema、route / query shape、adapter API、codec、npm依存、外部asset、token値、threshold、golden、生成command、画像byte。

## 非目標

- sample mediaの生成・取得・追加・commit。
- 決定的生成と pinned vendoring の**採択**（推奨記載のみ。裁定は`G0-6H-AF`）。
- asset path、file名、manifest schema、hash algorithm、route / query shape、adapter API、公開API、codec、npm package、外部素材、生成toolの決定。
- token値、製品theme、threshold、golden、期待値、component、iconの選定・変更。
- 画像capture、variant生成、人間審判の実施。
- `Project` および production `Registered folders` の意味の新設・変更・拡張。
- 現行route実装、route名、入場条件、`docs/mocks-ui/README.md`、`src/main.jsx`、hash fixtureの変更。
- 隣接チケット（`CU-107*` / `CU-110*` / `CU-111` / `U3a-*` / `U2h-*` / `G0-9*` / `U0e-*` / `CU-0B0*`）への波及。

## 必須負例

- §ALLOWED_FILE 以外のfileを変更する。
- 4 fileのうち一部だけを変更する部分適用、またはTODOスタブで契約を置き換える。
- `reference-handoff.md` の既存節（固定証拠、再現コマンド、自動report、5秒課題表、Decision template、既存4注記）を変更・削除・並べ替えする。
- Decision template の `未記入` または checklist の `[ ]` を本粒で埋める。
- `Starter Media` を Project asset、Document、公開API、plugin、永続形式、production Registered folderの正本として扱う記述。
- 具体path、file名、manifest schema、hash algorithm、route / query、adapter API、codec、npm package、外部URL、token値、threshold、golden、生成commandを1つでも確定する。
- 決定的生成または pinned vendoring のどちらかを**採択**として書く（`(i) 決定的生成`の推奨表記を超える）。
- labelまたはopaque IDから欠落意味を推測して補う。
- 承認済みnormal 5画面、`check-reference` 成功、Git ancestryを、visual parity・人間承認・route同一性・empty-project成立の根拠とする。
- `G0-6H` / `G0-6H-V0` / `CU-0B01` / `CU-0B02` / `U0e-3` の状態語を本粒で変更する。
- `implementation-ledger.md` を本ticket差分に含める。
- reviews索引未登録のまま新規文書を置く、または相対リンク切れを残す。
- lint / test 抑制、期待値・golden・threshold・fixture special-caseの追加・変更。
- `G0-6H-AF` 以外の後続粒を新設する、または次の一粒を2件以上起票する。

## 次の一粒（ちょうど1件）

docs-only **`G0-6H-AF`** — `Starter Media` fixtureのmedia源とprovenance方式を、**(i) 決定的生成** と **(ii) pinned vendoring** の二択で1件だけ裁定する粒。byteは生成しない。本粒は **(i) 決定的生成を推奨** として記載するにとどめ、採択は`G0-6H-AF`が行う。

## handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-M` | **DONE** | 前提。element-level gap mapと人間裁定一点 |
| `G0-6H-A0` | **DONE** | 前提。選択肢(a)とStarter Media方向の受領 |
| `G0-6H-A` | **DONE** | 本粒。scenario / fixture所有契約 |
| `G0-6H-AF` | **DO** | fixture media源とprovenance方式の二択裁定（byteなし） |
| `G0-6H-V0` | **WAIT** | 本契約のCodex統合まで維持 |
| `G0-6H` | **DO / HUMAN** | 据え置き |

## 関連

- [G0-6H-A0 選定](2026-07-28-g0-6h-a0-empty-project-starter-media-selection.md)
- [G0-6H-M gap map](2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)
- [G0-6H-S route裁定](2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)
- [reference handoff](../mocks-ui/reference-handoff.md)
- [ui-interaction-language](../ui-interaction-language.md)
- [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)
- [ui-runtime-architecture](../ui-runtime-architecture.md)
