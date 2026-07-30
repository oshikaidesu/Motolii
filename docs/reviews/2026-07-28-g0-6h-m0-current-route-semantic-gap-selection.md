# G0-6H-M0 現行route semantic gap確認粒の選定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-M0: **DONE**

## 1. 目的

`G0-6H-S`は人間審判入力を`#plugin-browser-candidate`へ一本化し、次粒を
`G0-6H-V0`とした。しかしV0着手前照合で、承認済み「Browser検索0件」画面が
G0-6正本の「empty project + asset browser」を満たさないことが判明した。
意味を推測で埋めず、現行5状態とG0-6必須表示要素のgapだけを記録する
docs-only `G0-6H-M`をV0の前へ選定する。

## 2. 確認した事実

- 承認済み2枚目はBrowser検索結果が0件だが、Stage、Inspector、Timelineには
  `night_drive`の作品内容が残り、empty projectではない。
- `docs/ui-visual-language.md`のG0-6 screen 1は
  `empty project + asset browser`を要求する。
- 現行`#plugin-browser-candidate`にempty-project scenarioは登録されていない。
- 旧`#reference/empty-browser`はcapture専用投影を持つが、G0-6H-Sにより
  required human-judgment inputではなく、現行routeへleafを複製してよい根拠にもならない。
- `G0-6H-S`が要求した決定的5状態mappingは、このgapを解消するまで閉じない。

## 3. 次粒が閉じる一成果

`G0-6H-M`は承認5状態をG0-6の5画面意図・必須表示要素へ照合し、
`対応 / partial / 対応なし / 未確認`で記録する。特に
`Browser検索0件 != empty project`を固定し、現行route上のempty-project意味を
新設するか、G0-6 screen 1を独立spec改訂するかという人間裁定一点へ返す。

## 4. 非目標

- empty-projectの表示意味、scenario API、adapter、route、fixtureを決める。
- 旧reference leaf、DOM、semantic ID、sceneを現行routeへ複製する。
- 画像、variant、manifest、script、React / CSS / Rust / test / guardを変更する。
- G0-6の5画面定義、token、threshold、G0-6H/U0e-3状態を変更する。

## 5. STOP条件

1. 画像で見えない必須要素を「対応」と推測する必要がある。
2. empty projectの現行route上の意味をagentが決める必要がある。
3. archived HTML、Document、公開API、plugin契約、永続形式の変更が必要になる。

## 6. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-S` | **DONE** | 現行routeを人間審判入力へ裁定 |
| `G0-6H-M0` | **DONE** | semantic gap確認粒を選定 |
| `G0-6H-M` | **DO** | 承認5状態と必須表示要素の対応/gapを非推測で記録 |
| `G0-6H-V0` | **WAIT** | mappingとscenario意味の人間裁定待ち |
| `G0-6H` | **DO / HUMAN** | 据え置き |
