# CU-0A08SSCI Browser Place source seam 前提順序 裁定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08SSCI: **WAIT**

## 1. 目的

最小コード実装粒`CU-0A08SSCI`は前提3件が未決のため成立しない。本粒はコードを一切書かず、(P)(I)(T)の依存順と「最初に閉じる唯一の一問」だけをdocsで裁定し、`CU-0A08SSCI`を`WAIT`として証跡化し、次の唯一の`DO`を1件だけ選定する。

## 2. 事実

- `ui/motolii-web/guard-tests/browser-ownership.test.mjs`の`validatePostPromotionChanges`は、`provenance.postPromotionChanges`が存在する場合`changes.length !== 1`でthrowする（entryは厳密に1件固定）。
- 同guardはentryの`task`を文字列`"G0-6H-V1ETB"`、`file`を`"ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx"`、`reason`を`"development-only Starter Media projection"`、`fixedSourceSha256`を`4edb3dfc…d5b8`にliteral固定し、`currentSha256`は当該fileの実SHA-256と一致することを要求する。key集合は5個で過不足をthrowする。
- `ui/motolii-web/source-provenance.json`の`postPromotionChanges`は上記条件を満たす1件のみで、`currentSha256` = `866124a69caaa168fa19c67e6c723db97fec67a61071bdbe66973576266c42f4`は当該`.jsx`の現行hashと一致する。
- `DiscoveryBrowserCandidate.jsx`（1276行）で`CandidateCreateBrowser`（537行）はmodule-private。同fileのexportは1198行`export function DiscoveryBrowserCandidate`の1つだけ。
- `CandidateCreateBrowser()`はpropsを受けず、747行で`CandidateBrowserTabs`から引数なしで呼ばれる。`developmentProjection`が定義済みの場合は`CandidateBrowserTabsProjectionOnly`（717行）が代わりに描画され、`CandidateCreateBrowser`は描画されない。`decodeProjection`の出力は`CandidateProjectBrowser`へしか渡らない。
- `elementProps(itemId)`（559行）は`{itemId, selected, tags, tagVisible, onSelect}`を返し、`itemId`はbare文字列（`"rectangle"`他）。
- `docs/mocks-ui/node_modules`は本worktreeに存在しない。`@babel/parser`を要するguard（`browser-ownership.test.mjs`、`inspector-read-model-inventory.test.mjs`、`npm run test:reference-guard`）は本worktreeで実行できない。
- 「現在の並列レーン」節で完全一致 `` `DO` `` の行は、本粒着手前はPRODUCT-ASSET `CU-0A08SSCI`の1件だった。前回のOpus 5 order draft判定は`ORDER: STOP`（発注不成立）だった。

## 3. 裁定

三前提:

- **(P)** provenance/ownership guard: post-promotion変更の多entry受理と改竄拒否が未決。
- **(I)** decode済みscoped identityを`CandidateCreateBrowser`へ供給する口が未決。
- **(T)** module-private componentを公開export追加・新依存追加なしで検証するharnessが未決。

候補 **(P-first)** と候補 **(I/T-first)** を並記する。(I)と(T)はいずれも`DiscoveryBrowserCandidate.jsx`のbyte変更を伴う。現行guard下でそのbyte変更が取り得る表現は2つに限られる。(i) 正当な2件目のentryを追加する → guardはthrow。(ii) entryを1件のまま`currentSha256`だけ更新する → guardは通るが、別粒の変更を`task: G0-6H-V1ETB` / `reason: development-only Starter Media projection`に帰属させるprovenance改竄になる。したがって(P)は(I)(T)の厳密な先行条件であり、**P-first**をVS-1 Rectangleに限って採択する。

最初に閉じる唯一の一問は(P)である。次の唯一の`DO`はORACLE-GUARD `CU-0A08SSCI-P`（1件）とし、PRODUCT-ASSETの完全一致`DO`は0件とする。

## 4. 変わらないもの

- 公開API、React source byte（`ui/`配下すべて）、guardの期待値・literal・hash、`source-provenance.json`。
- Document / journal / serde / 永続形式 / Undo単位 / plugin契約。
- Place ownerと`CU-101` / `CU-102` / `CU-110`の意味。
- bare `itemId` drag payloadとJSX `identity` literalの`S`。
- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V`の状態と依存セル。
- W0/W1表、`U4a-2`、M3仕様本文、`docs/mocks-ui`配下、`docs/mocks/`。

## 5. 非目標

- (P)(I)(T)のいずれかを実装すること。
- 型・callback・event・payload・props名・module・export・wire・transport・decoder名を決めるか命名すること。
- guardの期待値・literal・hash・fixtureを書き換えること。`postPromotionChanges`を編集すること。
- `ui/`のbyteを1つでも変えること。
- (I)(T)にIDを与えてlane行を追加すること（本粒は`CU-0A08SSCI-P`のみ追加）。
- 「発注依存証跡」へ行を追加すること。
- bare IDの`S`を解消済みと書くこと。

## 6. 必須負例

- (I/T-first)を採択し(P)を後回しにする。
- `CU-0A08SSCI`を`DO`のまま残す、またはPRODUCT-ASSETの完全一致`DO`を2件以上にする。
- provenance改竄（1 entryのまま`currentSha256`だけ更新して別粒変更をG0-6H-V1ETBに帰属）を正当化する。
- `CU-0A08SSCI`を発注依存証跡へ追加する。
- allowlist外のstale mirrorを本粒で書き換える。

## 7. 同期した current mirror

同じ7箇所を、実装粒 `CU-0A08SSCI` はOpus 5 order判定 `STOP` により `WAIT`。次の唯一の `DO` はORACLE-GUARD `CU-0A08SSCI-P`（1件）で、PRODUCT-ASSETの完全一致 `DO` は0件へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCI` | **WAIT** | Opus 5 order判定`STOP`（前提3件未決）。decode済みidentity／VS-1 Rectangle／非目標は[CU-0A08SSCSD裁定](2026-07-29-cu-0a08sscsd-browser-place-source-seam-implementation-scope-decision.md)のまま |
| `CU-0A08SSCI-P` | **DO** | (P) post-promotion provenance多entry受理と改竄拒否をoracle/guard前提として先に閉じる |
| (I) / (T) | 未採番前提 | (P)完了後に順に選定する未採番前提（IDを与えない） |

## 10. STOP条件

1. 型・event・payload・props名・decoder名・module pathを先に決めないと文が閉じない。
2. `ui/`のbyte、guard期待値、`source-provenance.json`を変えないとoracleが緑にならない。
3. 公開API・Document・serde・永続形式・plugin契約・Place ownerの変更が必要になる。
4. laneの完全一致 `` `DO` `` を1件（`CU-0A08SSCI-P`）に保てない。
5. allowlist外のfileを変更する必要が生じた。
