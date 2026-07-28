# G0-6H-S 人間審判入力routeの裁定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-S: **DONE**

## 目的

G0-6Hの人間審判入力routeを一本化し、採択内容を文書に確定する。旧証拠はrequired human-judgment inputから外して保持し、現行候補のcurrent-route evidence contractは次粒で要求化する。

## 確認した事実

- `docs/implementation-ledger.md`の現在の並列レーンでは`G0-6H-S`が`DO`として存在する。
- 依存する`G0-6H-E0`/`G0-6H-E`/`G0-6H-R0`/`G0-6H-R`が`DONE`として記録されている。
- `docs/reviews/2026-07-28-g0-6h-r-reference-authority-role-reconciliation-decision.md`は`R-1`〜`R-5`を確定し、`G0-6H-S`を前提扱いとしてhandoffしている。
- `docs/reviews/2026-07-28-g0-6h-e-candidate-approval-observation.md`は現行候補の5画面normal承認が環境未提供・派生variant未取得であること、旧30 PNGと派生25枚が未承認であることを明示している。
- `docs/mocks-ui/reference-handoff.md`の固定証拠は`eb16d06f...`と`u0e2-08f96cbd7754-85c0fc529ab1`を不変で保持し、現行candidateを`/`または`#plugin-browser-candidate`で開く現状を記録している。
- `ui/motolii-web/source-provenance.json`の`fixedSourceCommit`は`56c318edcddab7cf95d263cc2f7dd2b4e6791134`、authorityは`docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md`である。

## 決定

- **S-1**: G0-6Hの人間審判入力routeとして候補(B)を採択し、以後のforward-lookingなG0-6H人間審判入力は、product-owned React source authority `56c318edcddab7cf95d263cc2f7dd2b4e6791134`が裏付ける`#plugin-browser-candidate`**だけ**とする。
- **S-2**: 旧generation `u0e2-08f96cbd7754-85c0fc529ab1`とsource authority `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`は**不変**として保持し、(a) 固定generationの再現証拠、(b) normal→lightness/grayscale/Machado CVD派生生成手順のderivation-regression証拠として維持する。required human-judgment inputには含めない。削除・再生成・移動・期待値変更をしない。
- **S-3**: `G0-6H-E`が記録した現行候補normal色5画面の承認は**partial evidence**に留める。`G0-6H` / `CU-0B01` / `U0e-3`を解禁せず、`docs/mocks-ui/reference-handoff.md`のDecision templateとchecklistの充足には代替しない。
- **S-4**: 後続の**current-route evidence contract**は以下5点を閉じる要求としてのみ規定する。
  1. 現行候補5状態（mixed Timeline / Browser検索0件 / Interval Easing / Hand / Relative Move）と`ui-visual-language.md`「## G0-6の審判」の5画面意図との、推測なしの決定的semantic対応を固定する。
  2. viewport, scale, locale, timezone, theme, reduced motion, browser version/revision, font fixtureを固定するcapture環境を記録する。
  3. normalに加えlightness / grayscale / Machado CVD（protanopia / deuteranopia / tritanopia）派生を、normal RGBAから再計算して揃える。
  4. generationをimmutableとして置き、manifest（path＋SHA-256閉包）とread-only照合手段を持つ。
  5. 判定者、実施日、表示環境（OS / display / scale / ambient）、5秒課題結果、採否と理由を正本へ残すhuman sessionを記録する。
  具体値・閾値・token・component・画像・script・route実装は本粒で決めない。
- **S-5**: `G0-6H-R`の`R-4`/`R-5`を維持する。route裁定は`#reference/*`と`#plugin-browser-candidate`の間でvisual parityを主張しない。Git ancestry成立や`check-reference`成功をroute横断のparity・人間承認・route同一性の根拠にしない。
- **S-6**: 本裁定はroute選択のみに限定する。`#reference/*` routeの削除、入場条件変更、実装変更は含めない。旧`#reference/*`は`docs/mocks-ui/README.md`:44の規則で引き続き開く。
- **S-7**: 具体token値、製品theme、閾値、golden、期待値、画像、fixtureの選定・生成・変更は行わない。`G0-6H` / `CU-0B01` / `U0e-3` / `CU-0B02`の状態語を変更しない。
- **S-8**: 次の一粒はdocs-only `G0-6H-V0`のみとし、現行routeに対するvariant evidence contractの要求を閉じるのみとする。

## 確定しないこと

- 旧30 PNG（`u0e2-08f96cbd7754-85c0fc529ab1`）および派生25枚の人間採否。
- 現行候補派生variantの採否。
- 具体token値、製品theme、閾値、golden、画像、fixtureの選定。
- `G0-6H` / `CU-0B01` / `U0e-3` の状態変更。
- 現行候補と旧referenceのvisual parity成立可否。
- `G0-6H-V0`でのevidence contractの具体形式・script・command。

## 非目標

- 画像・variant・generation・`CURRENT`・`reference-provenance.json`の生成・再生成・変更・移動・削除。
- route実装、route名、入場条件、`docs/mocks-ui/README.md`、`src/main.jsx`、hash fixtureの変更。
- React / CSS / Rust / fixture / test / guard / JSON / script の変更。
- 具体token値、製品theme、閾値、golden、期待値、component、iconの選定・変更。
- `reference-handoff.md` のDecision template / 5秒課題checklistの記入。
- `G0-6H` / `CU-0B01` / `CU-0B02` / `U0e-3` / `U2c-3` / `U2c-5` の状態変更・完了・解禁。
- `docs/implementation-ledger.md` の変更。
- 公開API、Document意味、plugin契約、永続形式、serde defaults の変更・新設。
- 隣接チケット（`CU-107*`、`CU-110*`、`CU-111`、`U3a-*`、`U2h-*`、`G0-9*`）への波及。
- `G0-6H-V0`の内容を先取りした裁定や2件以上の次一粒を起票。

## 必須負例

- `docs/mocks-ui/reference-handoff.md` の `React source authority: \`eb16d06f...\`` 行、capture generation行、manifest SHA-256行、images行を変更・削除・置換する。
- Decision template の `未記入` または checklist の `[ ]` が1つでも埋まる。
- Git ancestry または `check-reference` 成功を、visual parity・人間承認・route同一性の根拠として書く。
- 現行候補normal色5画面承認を、旧30 PNG・派生25枚・現行派生variant・具体token・`U0e-3`解禁へ拡張して書く。
- 現行候補と旧referenceのvisual parityを主張、または暗黙に前提とする。
- `G0-6H` / `CU-0B01` / `CU-0B02` / `U0e-3` の状態語を変更する。
- `docs/implementation-ledger.md` が差分に含まれる。
- §ALLOWED_FILE 以外のファイル（React / CSS / Rust / fixture / test / guard / JSON / script / 画像）を差分に含める。
- `docs/decision-index.md` に固定語彙外の状態語を入れる。
- `docs/specs/M3-ui-integration.md` の `G0-6` 行の状態cell、または `U0e` 行を変更する。
- `docs/ui-visual-language.md` の既存5画面定義・自動審判・人間審判項目を1文字でも変更する。
- `docs/ui-reference-map.md` の既存表cellを変更する。
- `G0-6H-V0` 以外の後続粒を新設、または `G0-6H-V0` の具体semantic mapping、capture値、variant algorithm、manifest形式、session書式を先取り裁定する。
- evidence contractの5要件を実装・script追加・fixture追加で満たそうとする。
- TODOスタブ、部分適用（7 fileのうち一部だけ変更）、lint/test抑制、期待値・golden・threshold・fixture special-caseの追加。
- serde default、公開API、Document意味、plugin契約、永続形式に触れる記述の新設。
- 新しいguard script、新しいplanner、既存helperと重複するhelperの新設。

## STOP条件

- 裁定を書くために`#reference/*`と`#plugin-browser-candidate`間のvisual parity主張が必要になった。
- 5状態semantic mappingが、authorityにない意味の発明なしには書けないと判断した。
- 画像・golden・threshold・token値の生成または変更が必要になった。
- 公開API、Document意味、plugin契約、永続形式、route実装、React/CSS/Rust/fixture/testの変更が必要になった。
- 旧generationのmanifest・PNG・`CURRENT`・`reference-provenance.json`を変更しないと整合が取れない。
- `docs/implementation-ledger.md` を変えないと整合が取れないと判断した。
- AUTHORITY行SHA-256と作業時file hashが一致しない。
- `docs/implementation-ledger.md`「現在の並列レーン」の`G0-6H-S`行が`DO`でない、または発注依存証跡の4 DEPENDENCY行のいずれかが`DONE`でない。
- 既存guard baseline（`118 pass / check-docs OK / reference-guard 0 fail`）が着手前から赤い。

## handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-R` | **DONE** | 前提。二commitのauthority役割分類 |
| `G0-6H-E` | **DONE** | 前提。現行候補normal色5画面承認の限定観察 |
| `G0-6H-S` | **DONE** | 本粒。人間審判入力routeを候補(B)へ裁定 |
| `G0-6H-V0` | **DO** | docs-only。現行route用variant evidence contractの要求を閉じる次の一粒 |
| `G0-6H` | **DO / HUMAN** | 据え置き（未完了） |
