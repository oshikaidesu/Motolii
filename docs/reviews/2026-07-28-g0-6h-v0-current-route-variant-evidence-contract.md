# G0-6H-V0 現行route variant evidence契約

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-V0: **DONE**

## 目的

`G0-6H-S` S-4 の5要求を docs-only の契約として閉じる。画像・script・fixture・byte・capture・token・threshold は閉じない。

## V-1 契約の適用範囲

本契約は `#plugin-browser-candidate`（product-owned React source authority `56c318edcddab7cf95d263cc2f7dd2b4e6791134`）を入力 route とする current-route evidence にだけ適用する。旧 `#reference/*` generation `u0e2-08f96cbd7754-85c0fc529ab1` は `G0-6H-S` S-2 のとおり不変の再現・派生回帰証拠であり、本契約の適合対象にしない。

## V-2 5状態 semantic mapping

`G0-6H-M` §3 の5対応を、承認済み normal 5状態と `docs/ui-visual-language.md`「## G0-6の審判」5画面の間の決定的対応として固定する。

名称対応のみであり、element parity の証拠とはしない。

- mixed Timeline ↔ screen 2
- Browser検索0件 ↔ screen 1
- Interval Easing ↔ screen 3
- Hand ↔ screen 4
- Relative Move ↔ screen 5

`G0-6H-M` §5 の総合判定:

- screen 1 = `対応なし`（`G0-6H-M0`の確定事実で、`Browser検索0件`画面に`night_drive`のStage / Inspector / Timelineが残るため`empty project`ではないため）
- screen 2 = `partial`
- screen 3 = `partial`
- screen 4 = `partial`
- screen 5 = `partial`

element 単位の `未確認` は本粒で `対応` へ格上げしない。

## V-3 screen 1 の充足条件

screen 1（`empty project + asset browser` / `Browser検索0件`）が `対応` になる条件は、`G0-6H-A` A-1〜A-3 の scenario を満たす capture が存在することとする。すなわち Project assets 0、Stage / Inspector / Timeline に作品内容なし、Browser は `Media` surface 上に表示され続け、Project 外 fixture-only `Starter Media` を示す。本粒では capture が存在しないため screen 1 は `対応なし` のままであり、状態語を変更しない。

## V-4 Starter Media 束縛

本契約における `Starter Media` の唯一の fixture-only 源は commit `e4ad5c9f` の証拠カプセル（`docs/mocks-ui/starter-media/media/` の4 byte と `docs/mocks-ui/starter-media/starter-media-provenance.json`）とする。`G0-6H-A` A-7 / `G0-6H-AF` AF-6 / `G0-6H-AG0` の停止線を継承し、Project asset / Document / 製品runtime / 公開API / plugin契約 / 永続形式 / production Registered folder の正本にしない。カプセルは `RESPONSIBILITY DISPOSITION: WRAP` / `RETIREMENT: FROZEN / DELETE-LATER` のまま製品 runtime から到達させない。byte・path・schema・hash値を本粒で新設・変更しない。

## V-5 capture 環境の閉集合

記録必須の軸をちょうど次の9件の閉集合として固定する: viewport、scale、locale、timezone、theme、reduced motion、browser version、browser revision、font fixture。各軸の**具体値は実際の capture から literal に記録する**ものとし、本粒では1件も確定しない。`G0-6H-E` が記録した `1440×900` / dark は現行候補 normal 承認時の観察事実に留め、本契約が要求する値として採択しない。

## V-6 variant の閉集合

1 screen あたり variant はちょうど6件（`normal` / `lightness` / `grayscale` / `protanopia` / `deuteranopia` / `tritanopia`）とし、5 screen × 6 variant = 30 とする。派生5件は当該 screen の `normal` RGBA から再計算して揃える。variant 種別の追加・削除、算法、閾値、差分許容量は本粒で決めない。

## V-7 immutability と read-only 照合

generation は immutable として置く。manifest は path と SHA-256 の閉包を持つ。照合手段は read-only であり、実行前後で Git 可視 file と status を変えず、失敗時は何も変更せず停止する。生成・照合・capture・test のいずれの時点でも network を要求しない（`G0-6H-A` A-5 / `G0-6H-AF` AF-3 を継承）。`G0-6H-AF` AF-4 / `G0-6H-AG0` を継承し、cross-platform / OS横断 / toolchain横断の byte 決定性を主張しない。manifest schema、file 配置、hash algorithm 名、check command は本粒で決めない。

## V-8 human session 記録項目の閉集合

記録必須項目を、`docs/mocks-ui/reference-handoff.md` の既存 Decision template と同一の閉集合として固定する: 判定者 / 実施日 / 表示環境（OS / display / scale / ambient）/ 使用generation / 5秒課題の結果 / 採否（`ACCEPT` / `REVISE`）/ 採否理由 / 修正要求（screen / semantic role / observed problem）/ 採択する具体token候補 / 棄却する具体token候補 / 次に解凍する粒。項目の新設・改名・削除をせず、本粒では1件も記入しない。

## V-9 状態の非変更

本粒は `G0-6H` / `G0-6H-V0` / `CU-0B01` / `CU-0B02` / `U0e-3` / `U2c-3` / `U2c-5` の状態語を変更せず、`docs/implementation-ledger.md` を差分に含めない（Codex 統合が所有する）。現行候補 normal 5画面の承認は partial evidence のままであり、`reference-handoff.md` の Decision template / checklist の充足に代替しない。

## React / Browser authority（参照のみ・本粒は差分0）

- **対象面**: product-owned React module `DiscoveryBrowserCandidate` の `Media` surface（route `#plugin-browser-candidate`、`ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`）。移管契約は [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)。UI runtime 境界は [ui-runtime-architecture.md](../ui-runtime-architecture.md)（Browser は bundled first-party Host module）。対応 spec ID は M3 / VS-1 / G0-6H 系列（`G0-6H-S` route 裁定、`G0-6H-M` gap map、`G0-6H-A` scenario 契約、`G0-6H-AF` 媒体源裁定、`G0-6H-AG0` 責任処分、固定 evidence カプセル commit `e4ad5c9f`、本粒 `G0-6H-V0`）。
- **SOURCE ASSET**: 固定 commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`（`ui/motolii-web/source-provenance.json` の `fixedSourceCommit`）。対象 export は `DiscoveryBrowserCandidate` at `ui/motolii-web/src/index.js`。
- **PRESERVE / REPLACE / DIAGNOSTIC ROUTE / NEGATIVE ORACLE**: 本粒は React source を1 byteも変更しない。Motolii Studio Preview は未実装。旧 `#reference/*` generation route と guard test は development 専用の契約確認に限る。`Starter Media` 証拠カプセルは製品 runtime から到達しない `FROZEN / DELETE-LATER` 証拠面に閉じる。

## 確定しないこと

- 旧30 PNG（`u0e2-08f96cbd7754-85c0fc529ab1`）および派生25枚の人間採否。
- 現行候補派生variantの採否、capture 実施、具体 token 値。
- capture 環境9軸の具体値、manifest schema、file 配置、hash algorithm 名、check command、variant 算法・閾値・差分許容量。
- 画像・media byte・golden・route 実装・script・fixture の生成または変更。

## 非目標

- 画像・variant・generation・`CURRENT`・`reference-provenance.json`・`starter-media/**` の生成・再生成・変更・移動・削除。
- script、fixture、guard test、npm script、CI job、依存、`docs/mocks-ui/package.json` / `package-lock.json` / `node_modules` の追加・変更。
- 具体 token 値、製品 theme、製品 font、threshold、golden、期待値、component、icon の選定・変更。
- manifest schema、hash algorithm 名、file 配置、route / query shape、adapter API、check command、tool version、codec、寸法、尺、byte 数の確定。
- route 実装、route 名、入場条件、`docs/mocks-ui/README.md`、`docs/mocks-ui/src/main.jsx`、`ui/motolii-web/**`、hash fixture の変更。
- React / CSS / Rust / test / JSON / 画像 / media byte の変更。
- 公開 API、Document 意味、plugin 契約、永続形式、serde default の変更・新設。
- `docs/implementation-ledger.md` / `docs/specs/M3-ui-integration.md` / `docs/ui-visual-language.md` / `docs/ui-reference-map.md` / `docs/README.md` の変更。
- `Project` および production `Registered folders` の意味の新設・変更・拡張。
- 隣接チケット（`CU-107*` / `CU-110*` / `CU-111` / `U3a-*` / `U2h-*` / `G0-9*` / `U0e-*` / `CU-0B0*`）への波及。
- `G0-6H-V1` の内容（capture 実施、具体値、算法、schema、command、画像）の先取り裁定。

## 必須負例

1. §ALLOWED_FILE 以外の file を変更・追加・削除する（`git diff --name-only` が4 path の部分集合でない）。
2. 4 file のうち一部だけを変更する部分適用、または TODO スタブ・「後で判断」で契約を置き換える。
3. `docs/mocks-ui/reference-handoff.md` の既存節（固定証拠 `:9-16`、再現コマンド、自動report、5秒課題表、Decision template、既存6注記）を1文字でも変更・削除・並べ替えする（append-only 違反）。
4. Decision template の `未記入` または checklist の `[ ]` を1つでも埋める。
5. `G0-6H-M` §5 の総合判定（screen 1 = `対応なし`、screen 2〜5 = `partial`）または §4 の `未確認` を、本粒で `対応` / `partial` へ書き換える。
6. capture 環境9軸のいずれかに具体値（viewport 実寸、scale 数値、locale、timezone、browser version / revision、font package version 等）を確定して書く。
7. variant を6件以外にする、種別を新設・改名・削除する、派生の算法・閾値・差分許容量を書く。
8. human session 記録項目を `reference-handoff.md` の Decision template 集合から追加・改名・削除する。
9. manifest schema、hash algorithm 名、file 配置、check command、route / query、adapter API、codec、byte 数、tool version を1つでも確定する。
10. cross-platform / OS 横断 / toolchain 横断の byte 決定性を主張する。
11. `Starter Media` またはカプセル byte を Project asset、Document、製品runtime、公開API、plugin、永続形式、production Registered folder の正本として扱う記述。
12. label（`Starter Media` / `PROJECT` / `AUDIO LIBRARY` 等）または opaque ID（`data-file-root-select` 値、`data-asset-source` 値、asset 名）から欠落意味を推測して補う。
13. 承認済み normal 5画面、`check-reference` 成功、Git ancestry を、visual parity・人間承認・route 同一性・empty-project 成立の根拠として書く。
14. `#reference/*` と `#plugin-browser-candidate` の visual parity を主張、または暗黙に前提とする。
15. 旧 generation `u0e2-08f96cbd7754-85c0fc529ab1` / source authority `eb16d06f...` を本契約の適合対象・required human-judgment input として扱う。
16. `G0-6H` / `G0-6H-V0` / `CU-0B01` / `CU-0B02` / `U0e-3` / `U2c-3` / `U2c-5` の状態語を変更する。
17. `docs/implementation-ledger.md` が差分に含まれる。
18. reviews 索引未登録のまま新規文書を置く、相対リンク切れを残す、`docs/decision-index.md` に固定語彙外（`決定 / 縮小採用 / 延期 / 棄却 / 撤回 / 未統一 / 観察 / 比較中 / 停止線` 以外）の状態語を書く。
19. lint / test 抑制、期待値・golden・threshold・snapshot・fixture special-case の追加・変更、生 JSON / 文字列走査による型付き境界の迂回、公開 raw API、invented serde default、重複 planner / helper の新設。
20. 新しい guard script、npm script、CI job、文書 template を作る。
21. `G0-6H-V1` 以外の後続粒を新設する、次の一粒を0件または2件以上にする、または `G0-6H-V1` の具体値・算法・schema・command・画像を先取り裁定する。

## STOP条件

- `docs/implementation-ledger.md`「現在の並列レーン」の `G0-6H-V0` 行の状態語が `DO` でない、または一意でない。
- 「発注依存証跡」の6 DEPENDENCY 行のいずれかが `DONE` でない、または一意でない。
- AUTHORITY 行の SHA-256 と作業時の file hash が1件でも一致しない。
- 契約を書くために、authority に無い意味（empty-project の表示意味、variant 算法、threshold、token、manifest schema、route parity）の発明が必要になった。
- 画像・media byte・golden・threshold・token 値・script・fixture の生成または変更が必要になった。
- 公開 API、Document 意味、plugin 契約、永続形式、serde default、route 実装、React / CSS / Rust / test の変更が必要になった。
- `docs/mocks-ui/reference-handoff.md` の既存節を変更しないと整合が取れない。
- `docs/implementation-ledger.md` / `docs/ui-visual-language.md` / `docs/specs/M3-ui-integration.md` を変更しないと整合が取れないと判断した。
- 着手前から既存 baseline が赤い（pre-edit 実行で失敗が出る）。
- 会話履歴、他 worktree、repo 横断の歴史調査、複数仕様の意味判断、未指定の公開境界探索が必要になった。

## 次の一粒（ちょうど1件）

**`G0-6H-V1`** — bounded implementation 粒。本契約 V-2〜V-8 に適合する current-route evidence generation（`#plugin-browser-candidate` からの5状態 capture、6 variant、immutable generation、SHA-256 manifest、read-only 照合）を作る。人間 session の実施、token 採択、`U0e-3` 解禁は含めない。2件目の後続粒を起票しない。

## handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-AG` | **DONE** | 前提。固定 evidence カプセル |
| `G0-6H-S` | **DONE** | 前提。route 裁定 |
| `G0-6H-V0` | **DONE** | 本粒。current-route variant evidence 要求契約 |
| `G0-6H-V1` | **DO** | bounded implementation。V-2〜V-8 適合 evidence generation |
| `G0-6H` | **DO / HUMAN** | 据え置き |

## 関連

- [G0-6H-S 人間審判入力routeの裁定](2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)
- [G0-6H-M 現行route element-level semantic gap map](2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)
- [G0-6H-A empty Project + local Starter Media scenario / fixture 所有契約](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)
- [G0-6H-AF Starter Media 媒体源・provenance class 裁定](2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md)
- [G0-6H-AG0 Starter Media generator / output closure 棚卸しと責任処分](2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md)
- [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)
- [reference handoff](../mocks-ui/reference-handoff.md)
- [ui-visual-language](../ui-visual-language.md)
- [ui-runtime-architecture](../ui-runtime-architecture.md)
