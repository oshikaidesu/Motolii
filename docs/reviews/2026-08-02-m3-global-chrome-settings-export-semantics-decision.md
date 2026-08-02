# M3 global chrome / Settings / recovery / Export 接続決定

状態: **決定（接続意味）／source closure一部不足・実装dispatch禁止**（2026-08-02）

## 1. 位置づけ

本決定は新しいSettings、autosave、Export機構を設計しない。
[M3縦slice実行方針](2026-07-24-m3-vertical-slice-execution-decision.md)の「M3はM0〜M2で成立した能力を通常製品UIへ接続するphase」と、
[M3既知技術採択・並列実装地図](../m3-parallel-implementation-map.md) §2の「Motolii codeは薄いadapter、製品policy、fixtureに限定する」をそのまま適用する。

Motoliiが今回所有するのは、入口の配置、状態owner、既存能力へのtyped connection、未対応能力を捏造しない負例だけである。

## 2. 利用者成果と既知実装への接続

| 利用者成果 | 既知実装・既存資産 | 採用形 | Motoliiに残す薄い接続 | M3で作らないもの |
|---|---|---|---|---|
| window上部からSettings、Save、Export、activityへ入れる | fixed React commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`の`mock-titlebar`、既存ProductApp projection、Stage host message route | `ADOPT` / `REMAP` | source assetの直接移管、projection、typed intent、一回だけのdispatch | 別header component、第二state、別command bus |
| Settingsで現在実在するuser/workspace設定を変更できる | `UiStateOwner`、`EnableReduceMotion`、`ResetWorkspaceProfile`、既存keymap codec。固定SHAのSettings本体は`archive/settings` legacy bridgeだけで製品sourceではない | `REUSE` / `SOURCE GAP` | source成立後の既存field・command・codecへのprojectionとintent | legacy bridge昇格、Settings store、新しい永続codec、UI都合のfield、keymap codec複製 |
| crash後もaccepted editを失いにくく、復旧候補を選べる | `ProjectSession`、journal/WAL、non-destructive recovery、`persist.rs`のatomic publish | `REUSE` | 既存durable stateのcandidate/blocked投影、restore/discard intent | autosave database、第二journal、独自lock、別atomic writer |
| window上部から既存の書き出しを開始し状態を追える | current productの`StageTransportCandidate`、`motolii-export::ExportJob`、headless worker、ffmpeg/encoder route、`ProductEvent::ExportFinished`。固定SHAにExport sheet sourceはない | `REUSE` / `REMAP` / `SOURCE GAP` | 現行Start/status routeのglobal header接続。sheetはsource成立後 | 別Export sheet、別Export model、job queue、encoder、架空のprogress/cancel |

## 3. 接続時の意味

### 3.1 Global header

- fixed React titlebarをproduct packageへ直接所有移管する。見た目だけ似せたcomponentを新設しない。
- 左側は既存projectionで取得できるMotolii、project名、dirty/read-only/recovery状態だけを表示する。
- 右側はSettings、Save、Export、compact activity indicatorを置く。
- Reactは表示とtyped intentだけを所有し、ProductAppが状態ownerである。
- global route接続後にだけStageのSave/Export重複入口を降格する。一操作を二重dispatchしない。

### 3.2 Settings

- fixed SHAの`AllSurfacesScreen`にSettingsボタンはあるが、製品移管可能なSettings sheet sourceはない。`archive/settings`の`LegacySettings`はlegacy HTML境界でありsource assetにしない。
- SettingsはUserSettingsまたはWorkspaceProfile ownerの既存能力だけを表示・変更する。
- 最初に接続する能力は`EnableReduceMotion`、既存keymap route、`ResetWorkspaceProfile`である。
- Document、Undo、selection、ProjectSession、ExportJobはSettingsのownerではない。
- UIに項目が存在しても、対応する既存field・command・codecが無ければdisabled/非表示とし、React local stateや新しいRust fieldで補わない。
- 新しい設定意味が必要ならM3接続粒を止め、その意味を所有する上流specへ`REMAP / REDUCE`する。
- Settings sheetの実装dispatchは、独立React source、CSS/model/test closureが成立するまで行わない。

### 3.3 Recovery autosave

- autosaveは常時有効とする。ただし実体は既存journal/WALのdurable editとnon-destructive recoveryを再利用し、別autosave機構を作らない。
- accepted Document commandの既存durable publishだけを復旧対象とし、transient previewを保存しない。
- autosaveはexplicit Save、dirty、canonical project、sidecar lineageを変更しない。
- read-onlyまたはlock未取得なら既存routeどおり書かず、blocked状態をheaderへ投影する。lock stealingやsilent fallbackは禁止する。
- 起動時は既存recovery結果をcandidateとして提示し、restore/discardを明示選択させる。canonical projectを無言で置換しない。
- 世代数、物理配置、pruneは既存journal/session実装に従う。M3 UIから別policyを追加しない。

### 3.4 Export

- fixed SHAにはExportボタンだけがあり、Export sheet sourceはない。現行製品sourceはStage上のStart/statusだけである。
- Export sheetは現行`ExportJob`とproviderが実在して持つ入力だけをdraftとして扱う。
- Startはvalidated jobを既存headless workerへ渡し、Documentとdirtyを変更しない。
- 現行Jobに無いrange、fps override、resolution、codec/container、audio mux等をM3で追加しない。必要ならexport契約の上流grainへ戻す。
- sheetを閉じてもstarted jobをcancelしない。activityは既存provider stateだけを投影する。
- providerがprogress/cancel capabilityを供給しない場合、percentage、time remaining、Cancel controlを表示しない。
- failure時は既存typed failureを投影し、final artifactを成功物として扱わない。
- したがって現時点でdispatch可能なのは既存Start/status routeのglobal headerへの`REMAP`までであり、sheet自体はsource成立待ちとする。

## 4. 製品先例の使い方

外部製品はMotoliiのauthorityや移植元ではなく、接続後の利用者期待を照合する比較材料である。

| 製品先例 | 確認する利用者期待 | Motoliiへ持ち込まないもの |
|---|---|---|
| [Adobe Premiere Pro crash recovery](https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/recover-projects-after-a-crash.html) | crash後に復旧候補を提示し、復元を明示選択できる | folder配置、interval設定、保存形式 |
| [DaVinci Resolve 20 project backups](https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-20-Editors-Guide.pdf) | backupを選んで復元し、現在projectを無条件に上書きしない | project database、backup機構、設定UI |
| [Final Cut Pro background tasks](https://support.apple.com/en-ie/guide/final-cut-pro/ver64e71609/mac) | share等のbackground activityを専用面で追跡し、能力があるtaskだけを制御する | queue、pause規則、percentage計測 |

## 5. 監督接続

field単位には分けず、既存ownerと供給routeが一つになる三つの契約境界で扱う。

### GRAIN M3-GLOBAL-CHROME-CONNECTION

- `REACT AUTHORITY`: global titlebar、React直接移管契約、ProductApp projection、M3-P01/P09。Settings sheetはsource gap
- `SOURCE ASSET`: commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`の`docs/mocks-ui/src/screens/AllSurfacesScreen.jsx::AllSurfacesScreen` titlebar、`all-surfaces.css`、`AllSurfacesScreen.stories.jsx`、`visual-parity.spec.js`
- `PRESERVE`: titlebar DOM/class、wordmark/project slot、Settings/Export button順、visual state
- `REPLACE`: fixture projectを既存ProductApp projectionへ、Settings/Export buttonを既存intentへ交換。Settings sheetは交換対象に含めない
- `STATE OWNER`: UserSettings / WorkspaceProfile / ProjectSession / Transient。React local persistenceなし
- `DIAGNOSTIC ROUTE`: 通常製品windowを成果とし、development確認面を代替成果にしない
- `NEGATIVE ORACLE`: source copy、legacy Settings昇格、存在しない設定field、二重state、二重Save/Export、接続前のStage入口削除を拒否
- `STOP`: Settings sheetを含める、対応する既存field/command/codec不在、owner変更、公開契約追加が必要なら当該能力だけ`REMAP / REDUCE`

### GRAIN M3-RECOVERY-CONNECTION

- `AUTHORITY`: 本決定 §3.3、`ProjectSession`、journal/WAL、non-destructive recovery、CU-211
- `INTERNAL TARGET`: accepted durable edit、existing recovery result、ProductApp header projection
- `OWNER`: ProjectSession/recovery provider。Document、dirty、canonical projectは変更しない
- `WRITE ROUTE`: existing durable journal → existing recovery → candidate/blocked projection → restore/discard intent
- `REUSE TARGET`: `motolii-doc` journal/session/persist。新しい保存機構なし
- `NEGATIVE ORACLE`: transient保存、dirty clear、lock stealing、silent canonical replacement、第二journalを拒否
- `STOP`: 既存recovery routeで常時保護を表現できず、新しい永続形式・lock・世代policyが必要なら上流へ戻す

### GRAIN M3-EXPORT-CONNECTION

- `AUTHORITY`: 本決定 §3.4、M3-P08、現行`motolii-export::ExportJob`
- `INTERNAL TARGET`: current ExportJob、headless worker、`ProductEvent::ExportFinished`
- `OWNER`: draftはTransient、job/activityは既存provider、Documentはread-only
- `WRITE ROUTE`: current Stage Start/status → global header intent/projection → current validated ExportJob → existing worker。sheetはsource gap
- `REUSE TARGET`: `motolii-export`、既存ffmpeg/encoder、既存atomic final publish
- `NEGATIVE ORACLE`: parallel Export model、queue、架空progress/cancel、Document変更、失敗artifact公開を拒否
- `STOP`: Export sheetを実装する、UI項目に対応する現行Job/provider capabilityが無い場合は隠すか上流へ戻し、M3で型やproviderを新設しない

## 6. 非目標

- Save-Asのdialog、identity切替、sidecar transaction
- 新しいUserSettings field・codec・store
- 新しいautosave/recovery storage、retention、prune policy
- ExportJob semantic expansion、preset、queue、provider cancel/progress実装
- React source assetの縮約再実装
