# CU-203 共通feedback source / ownership分割決定

作成日: 2026-07-31
状態: **決定 / CU-203S DONE / CU-203M DONE / CU-203P DONE**

## 1. 利用者成果

U2c-3の共通feedback componentを、個別pickerやpopupが独自のhover、focus、Cancel、
error stateを持たない一つの製品資産として成立させる。操作候補がinvalidまたはdisabledの時は、
色やdimだけで終えず、同じcomponentから「何ができないか」「なぜできないか」
「どうすれば進めるか」へ到達できるようにする。

CU-203は診断の意味を新設するticketではない。既存のU2c-1 interaction stateとU2c-4
`DiagnosticEnvelope`を正本にし、表示文言への段階投影とrecovery Intent配線はCU-204へ残す。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [UI操作言語 §8](../ui-interaction-language.md#8-失敗を教えるのではなく次の一手を返す)、[U2c-4契約](2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md)、[M3 U2c](../specs/M3-ui-integration.md)、[React直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md) |
| `INTERNAL TARGET` | product-owned primitive 7 exportとcomponent state CSS、既存`DiagnosticEnvelope::{reason, action, subjects, facts, recoverability, recovery_candidates}` getter。feedbackの独立React sourceは現存しない |
| `OWNER` | feedback modelはHost Transient projection、Reactはlocal presentationだけ。Document、selection、Undo、journal、pluginはownerにしない |
| `WRITE ROUTE` | CU-203はread-only表示だけ。recovery選択はCU-204で既存`DomainIntent`がある候補だけを通常Intent→D2 single writerへ渡す |
| `GAP` | fixed commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`と現行productに、U2c-3を満たす独立React source、CSS、state matrix、interaction/accessibility oracleが無い。legacy recovery dialogはraw legacy bridgeでありsourceにできない |
| `RESOLUTION ROUTE` | `REUSE`: primitives/token/U2c-1/U2c-4を再利用。`REMAP`: legacy dialogは不可。`REDUCE`: 表示shellだけをM→Pで確立し、診断投影とIntent配線はCU-204へ残す |
| `DISPOSITION` | `RESOLVE`: source不在を一粒のproduct新規実装で埋めず、CU-203MとCU-203Pへ分ける |

## 3. 分割

| ID | 種類 / 状態 | 一成果 | 依存 | 完了条件 |
|---|---|---|---|---|
| CU-203S | `SPEC / DONE` | source、owner、state matrix、停止線の裁定 | CU-0B02C-P、U2c-1/U2c-4 | 本文書と台帳が同じ分割を指す |
| CU-203M | `PRODUCT-ASSET / DONE` | mock側で独立feedback JSX/CSSとdevelopment-only matrixを確立 | CU-203S | fixed SHA。9 fixture、guard 3件、Playwright 4件、current-route 30 PNG byte同一 |
| CU-203P | `PRODUCT / DONE` | Mの固定bytesをproduct ownerへ直接移管 | CU-203M | mock consumer反転、copy 0、package export、ownership guard、matrix不変 |

MとPを一commitへ束ねない。正しい独立sourceが無い状態でproductへ縮約componentを先に作ることは、
React直接移管契約の停止線に該当する。

## 4. CU-203Mの閉じたcomponent契約

mock側の新規sourceは次の2 pathだけをownerとする。

- `docs/mocks-ui/src/feedback/Feedback.jsx`
- `docs/mocks-ui/src/feedback/feedback.css`

CU-203Mで確立した固定SHA-256は次とする。

- JSX: `459fdd6120fd369b78d4a9784d98ac2b29fbb553afb35522f8f680fdfe4e4cd1`
- CSS: `7e22e2a183796732c4f77c4bb018eb2342ecb812181e46f348e1aa3aa827ef50`
- matrix JSX: `da7c7f8b71c1518675ee23d247883d4aac299f370177a21a3cd02c671f73c4ca`
- matrix CSS: `3545caf57d23b99647ffd8e6c15fec1f3c96f21e3801ce3c6a6ef23f0477ef1e`

current-route publicationはgeneration `1632212201a0-ead41d4d6562`へ進み、
先行`ee3c1a2d44fd-ead41d4d6562`との30 PNG byte同一を確認した。

componentは既存product primitivesとgenerated product tokenだけを使う。新しいtoken、icon system、
global store、callback、timer、portal、popover、picker state machineを作らない。入力はCU-204が後で
Host projectionへ接続できるread-only presentation modelに限定する。

- `placement`: `inline / target / badge / cursor`
- `tone`: `neutral / valid / warning / error / loading / disabled`
- `label`: 何が起きているかを示す可視文
- `reason`: `{ code, text }`。stable codeと人間向け文を分離し、codeから文言を推測しない
- `recovery`: `{ kind, text }`。`retry-with-changed-input / requires-another-action / unrecoverable`
`invalid`は別toneを増やさず、`target + warning/error`とtyped `reason/recovery`の組で表す。
`disabled / warning / error`は`reason`と`recovery`を必須とし、欠落fixtureをfail closedする。
`loading`は`aria-busy`と動きに依存しないshapeを持つ。全fixtureは色と文字のほかに
component-localな幾何markerまたはborder pattern、roleまたはARIA state、
stable `data-feedback-*` hookを持つ。幾何markerはCSSだけの非再利用status cueであり、
glyph、inline SVG、Unicode、共通`Icon` exportを追加せず、未決のCU-0B02I icon systemを先取りしない。
Assistive専用の別文言も作らず、可視`label / reason.text / recovery.text`を同じ
`aria-describedby`関係で順に読ませる。

development-only `#diagnostics/feedback-states`に次の9 fixtureを固定する。

1. inline neutral
2. target valid
3. target invalid
4. disabled action
5. warning
6. error / unrecoverable
7. loading
8. semantic badge
9. cursor context

matrixは通常製品navigationへ登録せず、CU-203の製品成果そのものにも数えない。CU-203P後も
同じfixtureがproduct exportのconsumerとして残り、CU-204が通常製品routeへ接続する。

## 5. CU-203Pの直接移管

M完了時にJSX/CSS SHA-256、export、fixture、test closureを固定する。Pはbytesを
`ui/motolii-web/src/feedback/`へ移し、mock側をproduct export consumerへ反転する。
DOM、class、stable hook、ARIA、keyboard focus order、9 fixtureのstateを変えない。

product runtimeから`docs/mocks-ui`、legacy、archive、fixtureへのimportは0、
mock側の独立component/CSS copyも0とする。product exportはpresentation componentだけであり、
`DiagnosticEnvelope` wire codec、翻訳辞書、Document command、recovery callbackを公開しない。

CU-203Pでは固定JSX/CSSを次へSHA-256同一で移管した。

- `ui/motolii-web/src/feedback/Feedback.jsx`
- `ui/motolii-web/src/feedback/feedback.css`

mock側は`@motolii/motolii-web`の`Feedback`だけをre-exportするconsumerへ反転し、旧CSSを削除した。
package rootはpresentation component `Feedback`だけを公開し、内部validation helperは公開しない。
product ownership provenanceと専用guard 3件を追加し、9 fixtureのDOM、class、ARIA、focus order、
state hook、matrix JSX/CSS bytesは変更していない。CU-204のHost projection、表示密度、翻訳、
recovery Intent配線は未着手のまま残す。current-route publicationはgeneration
`0d972253d868-ead41d4d6562`へ進み、先行`1632212201a0-ead41d4d6562`との30 PNG byte同一を確認した。

## 6. 自動審判

1. 9 fixtureが一意なplacement/tone/state hookを持ち、全て幾何markerまたはborder patternと可視文を併用する
2. invalid/disabled/warning/errorはreason code、reason text、recovery kind、recovery textの
   いずれか欠落を拒否する
3. reason code、表示文、CSS class、labelからDocument identityやrecovery Intentを推測しない
4. reason/recoveryはhoverだけでなくkeyboard focusとaccessible descriptionから到達できる
5. loadingは`aria-busy`、errorはalert相当、badge/cursorは同じcomponent DOMを再利用する
6. theme外raw color、独自spacing、独自font、visual threshold、golden更新は0
7. U2c-1のstate machine、U2c-4のenum/adapter、Document serialize、journal、Undo、selection、
   render generationは変更0
8. Mでは通常route変更0。Pではmock copy/import 0とproduct ownership provenanceを固定する

## 7. STOP

- legacy recovery dialog、`InspectorCandidate`内のDOM、archive HTMLをsourceへ昇格する
- feedback表示のためにU2c-4 enum、既存error、Document、plugin/community契約、永続形式を変える
- 存在しないrecovery `DomainIntent`、Connection/Drop rejection、翻訳IDを発明する
- componentがDocument、writer、Undo、selection、Host transportを所有または直接変更する
- 個別picker/popup/toolのstate machine、hover、focus、CancelをCU-203へ持ち込む
- source不在のままproduct側へ別componentを作る、M/Pに独立copyを残す
- diagnostic routeだけを通常製品接続またはCU-204完了の証拠にする
- raw color、visual threshold、既存goldenの更新が必要になる
