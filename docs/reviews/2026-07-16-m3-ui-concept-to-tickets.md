# M3 UIコンセプトから実装チケットへの分解

日付: 2026-07-16
状態: **条件付き実装発注の正本**。意味は[UI操作言語](../ui-interaction-language.md)と[UI視覚言語](../ui-visual-language.md)、境界は[M3 UI境界汚染の予防](2026-07-14-m3-ui-boundary-prevention.md)に従う。[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み。U0a入場後、各枝番は依存どおり個別発注できる（枝番の完了主張はしない）

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
   ├─ U0b-1 状態所有fixture ─ U0b-2 domain intent
   │                          ├─ U0c-1 Command registry
   │                          │   └─ U0c-2 input router/IME gate
   │                          │       └─ U0d-1 keymap resolver
   │                          │           └─ U0d-2 JSON codec/migration
   │                          │               └─ U0d-3 全command適合検査
   │                          └─ U2a-1 gesture→D2 adapter
   │                              └─ U2b-1 edit E2E
   │                                  ├─ U2c-1 interaction state machine
   │                                  │   ├─ U2c-2 entry conformance
   │                                  │   └─ U2c-3 semantic feedback parts
   │                                  │       └─ U2c-5 diagnostic projection
   │                                  ├─ U2c-4 diagnostic envelope
   │                                  └─ U4a-1 parameter mapping
   │                                      └─ U4a-2 generated panel
   └─ U0e-1 token生成基盤 ─ U0e-2 reference fixture ─ G0-6H 人間審判
                                                      └─ U0e-3 product token/component

U0a + D3 ─ U1a-1 shell/static viewport ─ U1b-1 mailbox ─ U1b-2 stale result E2E
```

`U0b-1`と`U0e-1`は同じ製品コードへ触れないため並行可能。PR #184は製品PRとして丸ごとmergeせず、`U0e-1/2`の証拠・抽出元にする。`U1a-1`は静止画表示までで、workerや連続seekを混ぜない。

## 3. 発注可能な実装粒

### 3.1 状態・入力・キーマップ

| ID | 1チケットの成果物 | 依存 | 自動審判 | STOP / 非目標 |
|---|---|---|---|---|
| U0b-1 | 代表UI状態をDocument / User settings / Workspace-session候補 / Transientへ分類するtoolkit非依存の型とfixture | U0a, G0-2, D2 | 全fixtureが所有区分を持ち、Document外状態の変更でDocument serialize不変 | workspace永続形式、画面component、shortcutを作らない |
| U0b-2 | UI由来の操作をtoolkit非依存のdomain intentへ変換する最小公開境界 | U0b-1 | 代表操作の型付き生成、公開型のegui/eframe/winit依存走査、未知intentの型付き拒否 | key、mouse、px、DPI、egui eventをdomain型へ入れない |
| U0c-1 | 安定`CommandId` registryとcommand metadata | U0b-2 | ID重複、空ID、登録漏れを拒否。表示名変更でID不変 | keybinding、OS予約判定、UI設定画面を含めない |
| U0c-2 | press/release/click/dragを正規化するinput routerとIME preedit gate | U0c-1 | preedit中のshortcut抑止、同じ正規入力から同じintent、機能crateのraw key分岐拒否 | TextInput自体の実機IME合否はU1dへ送る |
| U0d-1 | builtin baseへuser deltaを重ねる純粋keymap resolver | U0c-2 | 追加/置換/複数割当/無効化、競合とOS予約の型付き診断 | ファイルI/Oと設定画面を含めない |
| U0d-2 | version付きJSON codec、migration、原本保全 | U0d-1 | roundtrip、migration冪等、未知`CommandId`と移行前原本保持 | 保存場所、GUI import/exportを決めない |
| U0d-3 | 全登録commandの再割当conformanceとraw key監査 | U0d-2 | 既定無効化→別bindingで同じintent。登録commandの例外ゼロ | 一部commandだけ固定する例外を認めない |

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
| U1a-1 | egui shell、既存device共有、静止viewport native texture、組み込み既定layout preset | U0a, G0-1, D3 | 同じDocument frameをCPU copyなしで表示。UI threadから`render_frame`を直接呼べず、native texture登録をframe loop内で行わない | layout永続化、再生、連続seek、CPU readbackを入れない |
| U1a-2 | toolkit非依存panel layout intentとegui_tiles runtime投影 | U1a-1, U0b-2 | split/tab/resize/hide/restore/resetを操作でき、同じlayout modelから決定的なruntime treeを作る。操作でDocument serialize不変 | `egui_tiles::Tree`/`TileId`/serde形を保存しない。panel固有domain intentを作らない |
| U1a-3 | panel layoutの所有層・保存寿命を決めたUser settings/workspace model | U1a-2, U0b-1 | version付きroundtrip、未知field保全、欠落panel fallback、別monitor消失時の安全復帰、既定preset reset。Document/journal/Undo不変 | 所有層未決のまま形式を焼かない。egui/egui_tiles型を保存形式へ入れない |
| U1b-1 | 最新値置換mailboxと単調generationを持つrender worker | U1a-1 | 100連続送信がblockせず、共有deviceの同期readbackなし | UI componentをworkerから直接更新しない |
| U1b-2 | 完了順反転fixtureと古いresult破棄のE2E | U1b-1 | 最新generationだけをevent-loopへ投影 | GPU work強制cancelを要件にしない |

### 3.4 編集境界と共通操作文法

| ID | 1チケットの成果物 | 依存 | 自動審判 | STOP / 非目標 |
|---|---|---|---|---|
| U2a-1 | domain intentをD2 commandへ変換するgesture macro/merge adapter | U0b-2, D2 | 1 gesture=1 Undo、異target/異gesture非merge、Cancel変更ゼロ | 新transaction APIを発明しない |
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

## 5. M3再入場後の発注候補順

次の順序は着手許可ではなく依存の目安である。U0a入場後、採択した項目は各行依存に従って発注する。

1. `U0b-1`を最初の実装Issue候補にする
2. PR #184から生成機構だけを`U0e-1`へ抽出し、具体token値とshellを製品へ持ち込まない
3. `U0b-1`完了後に`U0b-2`、続いて`U0c-1`と`U2a-1`を別レーンで起票する
4. `U0e-1`→`U0e-2`までは機構として進め、G0-6Hで必ず人間へ戻す
5. `U1a-1`はM3入場時の最新mainから起票し、静止viewport以外を混ぜない

この順序なら、配色やpxを先に発明せず、UIをUXの投影として作れる。同時に、共通部品から漏れた実装を後で見つけるのではなく、registry・state machine・conformanceで入口から拒否できる。
