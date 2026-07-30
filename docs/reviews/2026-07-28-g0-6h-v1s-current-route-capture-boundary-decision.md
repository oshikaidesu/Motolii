# G0-6H-V1S 現行route capture境界の裁定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-V1S: **DONE**

## 目的

`G0-6H-V0` が明示的に見送った4件を、責任境界だけに絞って裁定し `G0-6H-V1` の開始条件を満たす。

## B-1

### 裁定

- screen2〜5 は `#plugin-browser-candidate` の同一 route 上にある既存 interaction だけで到達する。
- screen1 は `G0-6H-A` が要件化した empty Project + Project外 Starter Media の成立条件を確認するため、B-2 の development 専用 typed fixture projection で到達する。

### 停止線

- route名変更はしない。
- 新しい route を追加しない。
- 追加の hash fixture key を生やさない。
- query/search param を追加しない。

### 根拠authority

- [G0-6H-S](2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)
- [G0-6H-M](2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)
- [G0-6H-V0](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)

## B-2

### 裁定

- `G0-6H-A` A-1〜A-3 の空状態・`Starter Media` 状態条件を成立させる入力は、development 専用の typed fixture projection に閉じる。
- その境界はプロダクト runtime、Document、公開API、永続形式へ到達しない。

### 停止線

- 開発用 projection の外側へ `Starter Media` を project asset として持ち込まない。
- `screen 1` を `empty project + Project 外 fixture-only Starter Media` 以外の状態証拠に再解釈しない。

### 根拠authority

- [G0-6H-A](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)
- [G0-6H-AF](2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md)
- [G0-6H-AG0](2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md)
- [G0-6H-S](2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)

## B-3

### 裁定

- 5画面の中の `Starter Media` 証拠カプセル byte は、`#plugin-browser-candidate` の Browser `Media` surface 上で表示される意味としてのみ扱う。
- `Project asset` / production Registered folder 正本化はしない。

### 停止線

- 画面表示意味を `Project` へ接続しない。
- `Starter Media` を製品 runtime・Document・plugin契約・永続形式・公開APIの正本にしない。

### 根拠authority

- [G0-6H-V0](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)
- [G0-6H-A](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)
- [G0-6H-AF](2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md)
- [G0-6H-AG0](2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md)

## B-4

### 裁定

- capture 環境9軸（viewport / scale / locale / timezone / theme / reduced motion / browser version / browser revision / font fixture）は、capture 実施時の literal 値を generation manifest が記録する。
- 9軸すべての記録責任は generation manifest の一面だけに固定する。

### 停止線

- `G0-6H-V0` V-8 の human session 記録項目の閉集合はすべて維持し、human sessionを9軸の記録責任者にはしない。
- 9軸のうちいずれかを manifest 以外の面へ同時に移譲しない。

### 根拠authority

- [G0-6H-V0](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)
- [reference handoff](../mocks-ui/reference-handoff.md)
- [G0-6H-S](2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)

## 確定しないこと

- `G0-6H-M` §5 の状態語や `G0-6H-M` の要件、または `G0-6H-V1` の具体 algorithm / schema / command は確定しない。
- `G0-6H-V0` で未決のままの capture 環境値の具体値は本粒で確定しない。

## 非目標

- `G0-6H-V1` の実装、画像・variant・generation・manifest schema・hash algorithm・check command の確定。
- route実装・route名変更・`screenRegistry` 変更・`docs/mocks-ui/src/**` や `ui/motolii-web/**` の変更。
- `Starter Media` の media byte / path / schema の再生成または変更。

## 次の一粒（ちょうど1件）

**`G0-6H-V1`** — `B-1` `B-2` `B-3` `B-4` が同時に成立した時点で、`#plugin-browser-candidate` の 5状態 current-route capture を実装化する bounded implementation 粒へ進める。

## handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-S` | **DONE** | 前提。route裁定 |
| `G0-6H-M` | **DONE** | 前提。semantic gap map |
| `G0-6H-A` | **DONE** | 前提。scenario / fixture 所有契約 |
| `G0-6H-AF` | **DONE** | 前提。媒体源・provenance class 裁定 |
| `G0-6H-AG0` | **DONE** | 前提。generator責任処分 |
| `G0-6H-V0` | **DONE** | 前提。current-route evidence 契約 |
| `G0-6H-V1` | **DO** | 裁定済み境界の下で実装粒 |
| `G0-6H` | **DO / HUMAN** | 据え置き |

## 関連

- [G0-6H-S human judgment route決定](2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)
- [G0-6H-M route semantic gap map](2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)
- [G0-6H-A scenario / fixture契約](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)
- [G0-6H-AF media source provenance 裁定](2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md)
- [G0-6H-AG0 generator責任処分](2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md)
- [G0-6H-V0 現行route variant evidence契約](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)
- [reference handoff](../mocks-ui/reference-handoff.md)
