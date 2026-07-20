# M3 UIコンセプトから実装チケットへの分解

日付: 2026-07-16
状態: **条件付き実装発注の正本**。意味は[UI操作言語](../ui-interaction-language.md)と[UI視覚言語](../ui-visual-language.md)、境界は[M3 UI境界汚染の予防](2026-07-14-m3-ui-boundary-prevention.md)に従う。[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み。U0a入場後の初回Uシリーズは、本書§5の順で1枝番ずつ直列発注する（枝番の完了主張はしない）

## 1. なぜ親タスクをそのまま実装しないか

`U0b: 状態所有+domain intent`や`U0e: token+component+icon`は設計上のまとまりであり、1 PRの大きさではない。大きいまま着手すると、型、保存、egui adapter、視覚判断が同時に動き、どの判断を戻すべきか分からなくなる。

今後の実装単位は次を満たす。

1. **1 Issue = 1 commit = 1つの検証可能な境界**とする
2. 各Issueに依存、変更可能範囲、正の完了条件、拒否条件、非目標を書く
3. 未決事項を含む粒は`DECIDE`または`HUMAN`に留め、実装粒と混ぜない
4. 親IDはロードマップ上の能力、枝番IDは実際の発注単位とする
5. 枝番をまとめた巨大PRを作らない。親完了は必要な枝番がmainへ到達した時にだけ記録する
6. [UIアップデート考古学](2026-07-16-ui-update-forensics.md)のAF審判から該当項目だけをIssueの拒否条件へ割り当てる

## 2. 最初の依存グラフ

```text
main済み U0a
  → U0b-1 状態所有fixture
  → U0b-2 domain intent
  → U0c-1 Command registry → U0c-2 input router/IME gate
  → U0d-1 keymap resolver → U0d-2 JSON codec/migration → U0d-3 全command適合検査
  → U2a-0 one-shot atomic macro → U2a-1 gesture→D2 adapter
  → U1a-1 shell/static viewport → U1a-2 layout projection
  → U1b-1 mailbox → U1b-2 stale result E2E
  → U2b-1 edit E2E → U2c-1〜5 common interaction/diagnostic
  → U0e-1 token生成基盤 → U0e-2 reference fixture
  → G0-6H 人間審判 → U0e-3 product token/component
  → U3a timeline foundation
  → U4a-1 parameter mapping → U4a-2 generated panel
```

Uシリーズの初回製品実装は、ファイル競合の有無ではなく意味・所有境界の確定順を優先し、
**1チケットずつ直列に進める**。最初は`U0b-1`、次に`U0b-2`とし、入力・keymapと
`U2a-0`と`U2a-1`までを閉じてから製品shellの`U1a-1`へ入る。PR #184と旧`cursor/m3-u0b-1-night` /
`cursor/m3-u0e-1-night` / `cursor/m3-u1a-1-night`は未検証の証拠・抽出元に限定し、
現行mainへ直接mergeしない。`U1a-1`は静止画表示までで、workerや連続seekを混ぜない。

## 3. 発注可能な実装粒

### 3.1 状態・入力・キーマップ

| ID | 1チケットの成果物 | 依存 | 自動審判 | STOP / 非目標 |
|---|---|---|---|---|
| U0b-1 | **完了**: G0-2で決定済みの代表UI状態をDocument / User settings / Workspace profile / Project session / Transientへ分類するtoolkit非依存の型とfixture | U0a, G0-2, D2 | `UiStateOwner`×5と所有層ごとの`UiStateLifetime`、typed代表fixture、Document層の非Document境界拒否、非Document更新後のDocument serialize不変 | 新しい所有層・寿命、workspace/session永続形式、画面component、shortcutを作らない |
| U0b-2 | **完了**: UI由来の操作をtoolkit非依存のdomain intentへ変換する最小公開境界 | U0b-1 | 5所有層の代表`DomainIntent`、一時adapter kindからの型付き生成、表外kindの`UnknownAdapterKind`拒否、公開型とsourceのtoolkit/物理入力/永続契約監査 | key、mouse、px、DPI、egui eventをdomain型へ入れない |
| U0c-1 | **完了**: G0-2の文字列規則に従う安定`CommandId` registryとcommand metadata | U0b-2 | ID構文、ID重複、空ID、intent欠落/重複を型付き拒否。5代表intentの全単射、builtin ID↔intent対応、表示名変更でID不変 | keybinding、OS予約判定、UI設定画面を含めない |
| U0c-2 | **完了**: press/release/click/dragを正規化するinput routerとIME preedit gate | U0c-1 | 7 phaseの意味差、preedit中の登録済みshortcut抑止、未知ID拒否、Safety Cancel分離、全builtin IDのregistry経由intent対応、同じ正規入力から同じ出力、raw key/toolkit/永続契約監査 | TextInput自体の実機IME合否はU1dへ送る |
| U0d-1 | **完了**: G0-2の閉じたGesture語彙でbuiltin baseへuser deltaを重ねる純粋keymap resolver | U0c-2 | Add、base exact GestureのReplace/Disable、複数割当、順序非依存delta。Primaryのplatform展開とKeyToggleのPress/Release展開後のEffectiveTrigger競合、Primary混在、未登録ID、OS予約、不正target/phaseを型付き診断し該当triggerを実行mapへ載せない | JSON/serde、原本保全、ファイルI/O、Context、設定画面を含めない |
| U0d-2 | **完了**: version付きJSON codec、migration、原本保全 | U0d-1 | roundtrip、migration冪等、未知`CommandId`と移行前原本保持 | 保存場所、GUI import/exportを決めない |
| U0d-3 | **完了**: 全登録commandの再割当conformanceとraw key監査 | U0d-2 | `builtin_command_registry()`全量へ一意な合成base/別Gestureを割り当て、base無効化→別bindingをresolveしたIDを`InputRouter`へ通して同じintent。全workspace memberのproduct sourceを[AST raw input監査](2026-07-16-m3-preflight-decisions.md#23-keymap保存)。U1a-2以後は同節で改訂した単一layout adapterの閉集合を除き、登録command・source fileの例外ゼロ | 製品既定Gesture/presetを推測しない。仕様改訂なしに一部commandやfileだけ固定する例外を認めない |

### 3.2 視覚言語

| ID | 1チケットの成果物 | 依存 | 自動/人間審判 | STOP / 非目標 |
|---|---|---|---|---|
| U0e-1 | DTCG token schemaと決定的Rust/egui adapter generator | U0a。PR #184は証拠・抽出元 | 同じ入力からbyte一致、生成物の手編集差分を拒否 | 色、px、radius等の**製品値を確定しない** |
| U0e-2 | 5 reference screenの固定fixtureと同条件render手順 | U0e-1 | normal/lightness/grayscale/CVDを同じfixtureから生成 | 見た目の良否をpixel testだけで決めない |
| G0-6H | 人間が5画面を見て階層、識別、馴染み、過剰装飾なしを判定し具体token値を固定 | U0e-2 | 判定者、画面、条件、採否理由を記録 | エージェントが目視判断を代行しない |
| U0e-3 | 確定token、共通component state、icon gridを製品へ導入 | G0-6H | contrast、focus、意味色+形、gradient allowlist、raw color/spacing拒否 | 新画面固有の独自componentや装飾を足さない |

この分割により「reference screenを作るにはtokenが要るが、token値を決めるにはreference screenが要る」という循環を解く。生成**機構**とfixtureは先に作り、製品の具体値だけを人間審判後に入れる。

### 3.3 shellと非blocking preview

| ID | 1チケットの成果物 | 依存 | 自動審判 | STOP / 非目標 |
|---|---|---|---|---|
| U1a-1 | **完了**: [決定済み静止viewport契約](2026-07-21-m3-u1a-1-static-viewport-contract.md)に従うegui shell、既存device共有、中央Stageだけの静止viewport native texture | U0a, U0b-2, G0-1, D3 | bootstrapが渡した同じfixture Documentを単一private preparation入口へ通し、独立display slotの期待画素をheadless統合oracleで検査。UiShared readback拒否、CreationContextでregister 1回、event loop開始後render/joinなし。製品binaryの実resize/minimize/restore smokeと、scale-factor adapter不変条件 | 五面preset、layout model/永続化、panel操作、製品token、再生、連続seek、worker/mailbox、CPU bridgeを入れない。実monitor DPI移動はU1e |
| U1a-2 | **完了**: [決定済みlayout投影契約](2026-07-21-m3-u1a-2-layout-projection-contract.md)に従うBrowser左 / Stage中央 / Inspector右 / Timeline下 / status下の組み込みpreset、private panel layout intentとegui_tiles runtime投影 | U1a-1, U0b-2 | 固定3補助paneをsubjectとするsplit/tab/resize/hide/restore/reset proposalを全体検証し、同じintentからTileId非依存の決定的signatureを作る。Stageはsplit anchorだけ。Stage最小幅とDocument全状態不変 | Tree/TileId/serde保存、Stageの移動/tab/hide、panel DomainIntent、Browser内部rail、status診断、自由dock/別window、U1a-3 codecを入れない |
| U1a-3 | panel layoutの所有層・保存寿命を決めたUser settings/workspace model | U1a-2, U0b-1 | version付きroundtrip、未知field保全、欠落panel fallback、別monitor消失時の安全復帰、既定preset reset。Document/journal/Undo不変 | 所有層未決のまま形式を焼かない。egui/egui_tiles型を保存形式へ入れない |
| U1b-1 | **完了**: [決定済みrender worker契約](2026-07-21-m3-u1b-1-render-worker-contract.md)に従う最新値置換request/result mailbox、単調generation、実`RenderedFrame`を返す常駐worker | U1a-1 | barrier中の100連続送信がconsumer/GPUを待たずpending generation 100だけを保持。実行中1の後は100だけを開始。実GPU canonical render、型付きerror/panic、close/drain/join、共有device同期readbackなし | UI component、display copy、TextureId、notifierをworkerから更新しない。event-loop stale結果破棄はU1b-2、GR-1出力poolはM4 |
| U1b-2 | **完了**: [決定済みlatest result投影契約](2026-07-21-m3-u1b-2-latest-projection-contract.md)に従うowner/client分離、repaint signal、既存display slotへのlatest-only copy、配送順反転fixture | U1b-1 | 取得済み古いresultを`2→1`で同じstale gateへ配送してgeneration 2だけを1回copy。実GPU/実windowでlatest generation一致、slot ID不変、register 1、Document不変。`run_native`帰還後join | GPU work強制cancel、複数worker、desc変更pool、seek UI、workerからのUI更新を入れない |

### 3.4 編集境界と共通操作文法

| ID | 1チケットの成果物 | 依存 | 自動審判 | STOP / 非目標 |
|---|---|---|---|---|
| U2a-0 | **完了**: [D2 one-shot atomic macro](2026-07-20-m3-u2a-1-command-adapter-contract.md) | D2 | 全command成功時だけ1 gesture/1 revision/1 Undo。空列・途中失敗はDocument/revision/gesture counter/Undo/Redo不変 | lifecycle transaction、schema/journal変更、既存`apply_command`意味変更をしない |
| U2a-1 | **完了**: [決定済みD2 commandを伴うDocument intentを1 macro requestへ変換するadapter](2026-07-20-m3-u2a-1-command-adapter-contract.md) | U0b-2, U2a-0 | 1 request=1 gesture=1 Undo、異target/異request非merge、初回適用前Cancel変更ゼロ、非Document intent/不一致を型付き拒否 | target/preflightを実装せず、新transaction API・適用後Cancel・配送E2Eを発明しない |
| U2b-1 | UI event→intent→command→single writer→`Arc<Document>`購読E2E | U1a-1, U2a-1 | edit/Undo/Redo往復、UI状態だけの変更でserialize不変 | egui memoryを第2のDocumentにしない |
| U2c-1 | Discover/Target/Preview/Commit/Cancel/Inspectの共通状態機械 | U2b-1, G0-7 | invalid遷移拒否、Cancel変更ゼロ、Transient非保存 | button/whip/tool別の状態機械を作らない |
| U2c-2 | Direct/Tool/Advanced入口の意味同値conformance harness | U2c-1 | 存在する複数入口が同じDocument意味とUndo単位 | 未実装入口をhidden helperで偽装しない |
| U2c-3 | target、error、semantic badge、cursor説明の共通feedback部品 | U2c-1, U0e-3 | state matrix、色だけ/文字だけ依存を拒否。disabled/invalidはtyped reasonと回復方法を必須化 | gray/dimだけのsilent disabled、個別機能固有のhover/focus/Cancelを許さない |
| U2c-4 | 既存の領域固有errorを棚卸しし、共通表示へ渡すTransient Diagnostic Envelopeとadapter境界を実証 | U0b-2 | `DocumentError`等の既存型から代表3系統を選び、stable reason code、action kind、subject ID、typed facts、recoverabilityを投影後も保持。future rejection向けfixtureを固定 | 未実装Connection/Drop型を先に発明しない。Document objectへのUI説明、巨大domain error enum、serialize、egui型、表示文言IDを作らない |
| U2c-5 | 同じ診断をBrief/Context/Inspect/Assistiveへ段階投影する共通componentとrecovery Intent配線 | U2c-3, U2c-4 | 全段階でreason/subjects/facts一致。Preview中の事前拒否、回復不能明示、screen-reader説明。recovery選択は通常Intent経由でUndo規則を守る | 外部検索を通常操作の必須手順にしない。診断componentからDocumentを直接変更しない |

### 3.5 最初の縦切り: 自動Effect panel

| ID | 1チケットの成果物 | 依存 | 自動審判 | STOP / 非目標 |
|---|---|---|---|---|
| U4a-1 | `ValueType → host control → command`対応表とtoolkit非依存model | U2b-1 | 全登録pluginの全保存parameterに対応または型付き拒否 | 新ValueType、plugin所有egui UI、one-knob macroを発明しない |
| U4a-2 | Effect Inspector内の自動生成panelとnonblocking preview | U4a-1, U0e-3, U1b-2, U2c-5 | 全保存param編集可能、100 slider update非blocking、最新preview、1 gesture=1 Undo。invalid/read-only parameterは共通診断で原因と次の一手を表示 | plugin独自panel、custom wgpu UI、grayだけのdisabledを入れない |

U4aを最初の縦切りにする理由は、Host所有の共通操作文法、型付きparameter、非blocking preview、Undoを一度に実証できる一方、自由plugin UIやParam Pipelineへ踏み込まずに済むためである。

## 4. 既存PRの扱い

| PR | 判定 | 次の行動 |
|---|---|---|
| [#181](https://github.com/oshikaidesu/Motolii/pull/181) keymap設計 | **仕様材料** | 現在のG0-2と照合し、未決の保存場所・初期Context・import/export等を実装で埋めない。U0c/U0dの枝番へ再発注する |
| [#184](https://github.com/oshikaidesu/Motolii/pull/184) visual spike | **証拠・抽出元** | generator、fixture、検査の再利用候補。spikeの具体token値やshellを製品決定として丸ごとmergeしない |
| [#137](https://github.com/oshikaidesu/Motolii/pull/137) 高密度UI mock | **履歴参照** | 現行の操作言語・着手前決定より古い。実装baseにせず、必要な観察だけreferenceへ移す |
| [#190](https://github.com/oshikaidesu/Motolii/pull/190) IME checklist | **人間証拠待ち** | U0c-2の自動gateとは分離し、U1dの対象OS実機記録へ使う |

## 5. M3再入場後の直列実行順

次の順序は初回Uシリーズの運用上の必須順である。各行の論理依存がより短い経路や
並走を許しても、現在選択中のUシリーズでは前の枝番がmainへ到達するまで次を起票しない。

1. `U0b-1`を最初の実装Issueにする
2. `U0b-1`完了後に`U0b-2`を単独で起票する
3. `U0b-2`後も初回Uシリーズは並走させず、`U0c-1`→`U0c-2`→`U0d-1`→`U0d-2`→`U0d-3`→`U2a-0`→`U2a-1`の順に意味入口を閉じる
4. `U2a-1`完了後の最新mainから`U1a-1`を起票し、静止viewport以外を混ぜない
5. `U1a-1/2`→`U1b-1/2`→`U2b-1`→`U2c-*`を直列に進める
6. PR #184から生成機構だけを`U0e-1/2`へ抽出し、G0-6Hで必ず人間へ戻す。具体token値と旧shellを製品へ持ち込まない
7. G0-6H後の`U0e-3`、`U3a`、`U4a-*`を最初の自動Effect panel縦切りへ合流させる

この直列順は論理上の依存が並列を許す場合にも初回運用として優先する。外部依存の
G0-6H、G0-8/M4-K1a、M4-K1d/K7/K8、M2-D5、GAP-16、Browser P41等へ
到達したら、その依存を迂回せずSTOPする。
