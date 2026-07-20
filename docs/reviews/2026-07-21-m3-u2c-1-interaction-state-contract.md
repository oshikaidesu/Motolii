# M3 U2c-1 共通interaction state machine契約

作成日: 2026-07-21
状態: **決定 / U2c-1実装待ち**

## 1. 目的

U2c-1は、[UI操作言語](../ui-interaction-language.md)で決定済みの
`Discover / Target / Preview / Commit / Cancel / Inspect`を、
機能別widgetやtoolへ分裂しないtoolkit非依存の共通状態機械として固定する。

この状態機械は操作進行を説明するTransientな投影であり、Document、D2 command、
target、診断、UI event列の新しい正本ではない。CommitはU2b-1のsingle writerへ
受理済みrequestを渡す不可逆点を表すだけで、状態機械自身はDocumentを変更しない。

## 2. 正本と現行コード事実

- [M3仕様 U2c](../specs/M3-ui-integration.md)は、入口意味同値、
  Cancel変更ゼロ、診断の段階投影、UI event列と診断の非保存を要求する
- [G0-7](2026-07-16-m3-preflight-decisions.md#6-g0-7-操作文法を共通部品の契約にする)は
  6状態の共通遷移と、preview/error/Cancel/capture lossを含むstate matrixを決定済みである
- UI操作言語§4は、Previewを候補が存在する時だけ通り、Target/PreviewからのCancelは
  変更ゼロで待機へ戻ること、Undoは6状態の外にある通常のD2 commandであることを明記する
- [U2b-1契約](2026-07-21-m3-u2b-1-single-writer-e2e-contract.md)は、
  完成済みrequestをqueueへ受理した後の取消や公開transaction lifecycleを認めていない
- 現行`InputRouter`はEscape、capture loss、focus lossを
  `CancelInFlightGesture`へ正規化するが、機能横断の操作進行状態はまだ無い
- 現行`motolii-ui`にはtoolkit非依存の`DomainIntent`と`UiStateOwner::Transient`がある。
  新しいDocument field、serde codec、egui stateを追加する必要はない

## 3. 固定する状態と遷移

### 3.1 状態の意味

| 状態 | 意味 | Documentへの権限 |
|---|---|---|
| `Discover` | 操作開始前。入口を提示または探索できる | なし |
| `Target` | subject、型、scopeを解決中または解決済み | なし |
| `Preview` | 確定候補を一時投影中 | なし |
| `Commit` | prepared requestがU2b-1のqueueへ受理された不可逆点 | 状態機械自身はなし |
| `Cancel` | 未commit操作を破棄したことを1遷移だけ観測できる | なし |
| `Inspect` | 成功または拒否の結果を既存snapshot/診断から検査する | なし |

`Target`はpicker画面の存在を意味しない。現在選択などからsubjectが即時解決する操作も、
同じ状態を通る。`Preview`は任意であり、表示可能な中間結果を持たない操作は
`Target → Commit`を使う。投影可能な確定候補があるのに、実装都合だけでPreviewを
省略してはならない。入口の違いと具体target型はU2c-2以降で扱う。

### 3.2 許可遷移

| 現在 | 次 | 条件 |
|---|---|---|
| `Discover` | `Target` | 操作を開始する |
| `Target` | `Preview` | 一時投影可能な候補ができた |
| `Target` | `Commit` | preview不要のprepared requestが受理された |
| `Target` | `Cancel` | request受理前に取消した |
| `Preview` | `Commit` | prepared requestが受理された |
| `Preview` | `Cancel` | request受理前に取消した |
| `Commit` | `Inspect` | writerの成功snapshotまたはtyped rejectionを検査可能になった |
| `Cancel` | `Discover` | transientな操作資源を破棄して待機へ戻る |
| `Inspect` | `Discover` | 結果検査を終えて次の操作を待つ |

表に無い遷移はすべてtyped rejectionとし、現在状態を変えない。同一状態への遷移も
暗黙no-opにせず拒否する。`Commit → Cancel`は適用後Cancelを発明するため拒否する。
`Discover → Commit`はtarget/scope確認を迂回するため拒否する。
Undoは6状態の遷移ではなく、Commit後のDocument意味を戻すU2b/D2側の別commandである。

### 3.3 API境界

`motolii-ui`に次のtoolkit非依存公開型だけを置く。

- 6 variantの`InteractionState`
- 現在状態を所有する`InteractionStateMachine`
- `transition(next)`が返す`InteractionTransitionError { from, to }`

machineの初期状態は`Discover`である。成功時だけ状態を更新し、失敗時は
errorに`from/to`を構造化して保持する。型にはDocument、command、target ID、
DomainIntent、entry kind、widget、pointer座標、px/DPI、表示文言、serde実装を持たせない。

状態機械はcallback、writer、render worker、診断componentを呼ばない。
各adapterは副作用の結果と状態遷移を対応させる責務を持つが、そのE2E配線は後続チケットで行う。

## 4. Cancel不変条件

`Target`または`Preview`から`Cancel`へ遷移する前後で、少なくとも次を不変にする。

- Document serialize、revision、Undo/Redo
- U2b-1 edit queue件数と発行snapshot
- render request generation
- 状態機械以外のUser settings / Workspace-session

状態機械単体はこれらを所有しないため、fixtureでは実`DocumentEditRuntime`へrequestを
queueしない経路と組み合わせて不変を証明する。Escape、capture loss、focus lossは
既存`InputRouter`が同じCancel intentへ正規化した後、この取消経路へ接続する。
物理eventやwidgetごとのCancel分岐を状態機械へ追加しない。

## 5. 自動審判

1. 全許可遷移と、6×6から許可表を除いた全invalid遷移を総当たりする。
   invalid時はtyped errorの`from/to`と現在状態不変を検査する
2. `Discover → Target → Preview → Commit → Inspect → Discover`と、
   preview無しの`Discover → Target → Commit → Inspect → Discover`を通す
3. `Target`および`Preview`から`Cancel → Discover`を通し、
   Document serialize、revision、Undo/Redo、queue、snapshot、render generationが不変
4. `Commit → Cancel`、`Discover → Commit`、同一状態遷移を明示的に拒否する
5. source/trait検査で`InteractionState`、`InteractionStateMachine`、
   `InteractionTransitionError`の全3型にserde実装、toolkit型、Document、command、
   target、DomainIntent、entry kind、pointer座標、px/DPI、表示文言が無いことを確認する
6. button、whip、tool、picker別の状態enumまたは遷移表を追加しない
7. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
   `./scripts/check-ui-toolkit-deps.sh`、
   `cargo clippy --workspace --all-targets -- -D warnings`、
   `cargo test --workspace`を通す

## 6. 非目標

- Direct / Tool / Advanced入口の同値fixture（U2c-2）
- target型、scope、selection、hit-test、connection pickerの具体状態
- Transient Diagnostic Envelope、reason code、recoverability（U2c-4）
- feedbackの色、icon、cursor、文言、Brief/Context/Inspect component（U2c-3/U2c-5）
- command preflight、request生成、writer lifecycle、適用後Cancel
- Document schema、journal、Undo形式、plugin契約、永続設定形式
- feature別の状態機械、局所popup、隠れhelper

## 7. STOP条件

次のいずれかが必要に見えた時点で実装を止める。

- 6状態以外、または機能別variantを共通machineへ足す
- target、scope、entry kind、diagnostic envelopeの未決型を先に発明する
- state transitionからDocument、writer、render worker、UI callbackを直接呼ぶ
- Commit後のCancel、rollback、公開transaction lifecycleを追加する
- UI event列またはinteraction stateをserializeする
- egui/eframe/winit型、px/DPI、物理入力を公開状態へ入れる
- 既存D2 API、Document意味、公開plugin契約、永続形式の変更が必要になる
