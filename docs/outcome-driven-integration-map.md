# 成果駆動統合地図 — M3を主軸にM4/M5を需要側から引く

状態: **起草 / 2026-08-07 node survey 62項目を実測根拠とする**

## 0. この地図が置き換えるもの

| 文書 | 本地図後の扱い |
|---|---|
| [m3-rn-runtime-execution-map.md](m3-rn-runtime-execution-map.md) | **dispatch authorityを本地図へ移す**。旧node ID、oracle、失敗例は検索資料として保持 |
| [m4-known-implementation-adoption-map.md](m4-known-implementation-adoption-map.md) | **技術採択は正本のまま維持**。実装順だけ本地図の需要側へ従属させる |
| [m5-known-implementation-adoption-map.md](m5-known-implementation-adoption-map.md) | 同上 |

**M4/M5の技術採択（`foyer-memory` REMAP、`cacache` REJECT、`rend3` REJECT、`glam` private leaf、
`renderling` 非gate 等）は既決であり本地図で再判定しない。**変わるのは*いつ引くか*だけである。

## 1. なぜ作り直すか

2026-08-07に62項目をcurrent codeへ read-only 照合した結果:

| | 件数 |
|---|---|
| 製品routeから到達可能 | **7** |
| 実在するが旧route／probeにのみ到達 | **30** |
| 部分的に実在 | **23** |
| **本当に存在しない** | **11** |
| コード読解では決着しない | 7 |

**M3は「作る工程」ではなく「先に作った資産を接続する工程」である。**
旧M3地図はこれを`TARGET_MISSING`（＝存在しない）と表現していたため、実際には実在するものを
不在として扱い、見積もりと発注順を歪めていた。M4/M5の地図は同種の誤りを持たない
（「probeを通したか」という検証可能な事実で書かれているため）。

## 2. 状態語彙（旧地図の最大の欠陥を修正する）

旧`TARGET_MISSING`は次の3つを1語へ潰していた。本地図では分ける。

| 状態 | 意味 | 次の一手 |
|---|---|---|
| `WIRED` | 通常製品routeから到達可能 | 受入と回帰だけ |
| `BUILT_UNWIRED` | 実在するが旧route／probe／testにのみ到達点がある | **接続**。再実装しない |
| `PARTIAL` | 一部だけ実在 | 不足部分を特定してから接続 |
| `ABSENT` | 本当に無い | 既知実装調査 → 採択 → 実装 |
| `UNDECIDED` | 意味が未決（旧`SPEC_ONLY`） | 仕様粒で先に閉じる |
| `EXTERNAL` | 実機・人間・配布審判 | syntheticで代用しない |

`BUILT_UNWIRED`と`ABSENT`を混同しない。**これが旧地図の誤りの本体である。**

## 3. 不可分な直列核と、薄くできる部分

[Controlled Microkernel決定](reviews/2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)は
「Coreを極小のtyped protocol kernelへ収束させ、Host capabilityを並列に実装・検収できる」と
既に定めている。**未実装の既決である。**

### 3.1 不可分（既決規律）

- Document mutationの順序 — 絶対規律4 single writer
- snapshot coherence — 全consumerが同一revisionを読む
- GPU device/queue/surfaceの単一owner
- atomic commit arbitration

### 3.2 薄くできる（現在のコード形状にすぎない）

2026-08-07の実測: 背骨3粒（pointer / time / selection）は**意味的にdisjointなのに**
`WireIntentEnvelope`（15 field union）と単一`match intent.kind`を共有するため直列化した。

仮コードでper-kind分解を試したところ**書けた**。kernelに残るのは`runtime`と
`projection_generation`だけで、**15 fieldのうち13がcapability側へ落ちる**。

→ `N-ABI-SPLIT`（下記）を並列度の前提nodeとして扱う。

## 4. 新規に発見したnode（旧地図のどこにも存在しない）

### `N-OVERLAY` — 2D overlay renderer seat

**`PROBE_ONLY`**（2026-08-08訂正。当初`ABSENT`としたのは誤り）。

（2026-08-10更新）`skia-safe`は`ed9024fc`でworkspace依存として`crates/motolii-ui/Cargo.toml`へ
追加済みだが、**`skia_safe`を使うRustコードはmainに0行**のまま。リポジトリ外の隔離probe
`skia-timeline-probe`で`skia-safe 0.99.0` + `wgpu 29`が実動しており(bin 15本、depth-rail v4〜v14)、
Windows target checkも実施済みである。
詳細は[リポジトリ外資産の棚卸し](reviews/2026-08-08-out-of-repository-asset-inventory.md)と
[2026-08-10回収監査](reviews/2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)。

したがって次手は**既知実装調査ではなく移管・接続**である。

再基線決定はrust-skiaをTimeline／Curve／Stage overlayの標準に定めたが、実装コードは未移管で、
旧地図はこれをnodeとして持っていない。にもかかわらず仮コードで背骨を書くと
**`draw_stage_overlay`と`draw_timeline`の2箇所で必ず現れる**。

`R2-STAGE-GIZMO` / `R2-TL-NAV` / `R2-CURVE-READ` / `R2-STAGE-VIEW`の全てがこれを待つ。
[依存優先・責任最小化ゲート](reviews/2026-07-24-dependency-first-responsibility-gate.md)を先に通す。

### `N-ABI-SPLIT` — Host ABI の per-kind 分解

`ABSENT`（既決原則の未実装分）。per-kind payload、module別snapshot寄与、intent routingの登録制。
背骨の粒ではないが、**以降の全粒の並列度を決める**。

### `N-GIZMO-SURVEY` — gizmo機構の既知実装調査

`ABSENT`。`references.md`にgizmoの項目が無く、
[既知実装採択モデル](known-implementation-adoption-model.md)は
「picking／gizmo／bounds を機構classとして調べる」と指定しているが調査recordが存在しない。
Blenderは GPL のため `PATTERN` 限定。非GPL候補は未調査。
AGENTS.mdの既知実装preflightにより、**調査前にgizmo実装を発注しない**。

## 5. 利用者outcomeと node（需要側から引く）

### 5.1 outcome A — 表示中objectを選び、動かし、時間に固定する【最初の人間応答地点】

| 段 | 状態 | 根拠 |
|---|---|---|
| projectを開き同一revisionを表示 | `WIRED` | `R0-HOST` / `R0-STAGE-LIFECYCLE` survey |
| 幾何投影（LayerId + 正準rect + world/camera） | `WIRED` | 2026-08-07 `7851e3d0` |
| pointer transport | `WIRED` | `0eb2a3c0` |
| transient評価時刻（frame wire） | `WIRED` | `11c8d012` |
| primary selection producer | `WIRED` | `68546b8d` |
| gesture baseline / transient preview | `BUILT_UNWIRED` | `PositionGestureBaseline`（旧route） |
| **gizmo handle描画** | `ABSENT` | → `N-OVERLAY` + `N-GIZMO-SURVEY` |
| release → Position key書き込み | `BUILT_UNWIRED` | `SetPositionKeyValue` 実在・journal/Undo込み |
| Timeline へ同一identity投影 | `BUILT_UNWIRED` + `ABSENT` | `project_timeline`実在 / 描画は`N-OVERLAY` |
| visible-range owner | `UNDECIDED` | U3a-2Q-V |
| Easing適用 | `BUILT_UNWIRED` | `SetPositionKeyInterp` 実在 |

**M4/M5の呼び出しは一度も現れない**（仮コードで確認）。現在時刻1frameしか評価せず、
再生も書き出しもしないため。

### 5.2 outcome B — mediaを入れる

| 段 | 状態 | 根拠 |
|---|---|---|
| file選択dialog | `ABSENT` | motolii-uiにfile dialog接続なし。rfdはP06-C1固定Mac隔離probeのみ |
| thumbnail生成 | `ABSENT` | `thumbnail` grepが`docs/`のみ、`crates/`に実装なし |
| metadata / probe | `BUILT_UNWIRED` | `motolii-media::probe` / FrameReader |
| media place intent | `ABSENT` | `document_edit_runtime.rs:83-128`のpush_*にplace_media相当が無い |
| D2 AddTrackItemでcommit | `BUILT_UNWIRED` | 既存command |

### 5.3 outcome C — 保存して開き直す

| 段 | 状態 | 根拠 |
|---|---|---|
| project open / lock | `PARTIAL` | `ProjectSession::acquire`はlock取得のみ |
| **OpenMode admission** | `PARTIAL` | `opened.open_mode`が`shell.rs:58-66`で破棄。→ **FINDING-1** |
| New（新規作成経路） | `ABSENT` | product向け経路なし。test helperのみ |
| Save As destination | `UNDECIDED` | P12-C1のconnection residual |
| 再open後のUndo履歴復元 | `ABSENT` | `UndoHistory::from_restored`は定義のみ呼び出し元ゼロ |

### 5.4 outcome D — 再生する

| 段 | 状態 | 根拠 |
|---|---|---|
| audio mixer core | `BUILT_UNWIRED` | AG-2成立、製品`PlaybackSession`接続はGAP-28 |
| mixed AudioProgram接続 | `ABSENT` | 現行は`PcmCache`/`AudioProducer`経路 |
| transport seek | `ABSENT` | `Transport`にseek/set_origin相当のmethodなし。`timeline_origin`はconstructorのみ |
| playhead進行 | `PARTIAL` | 本地図 §5.1 の時刻席は`WIRED`だが自動進行を持たない |

### 5.5 outcome E — 書き出す

| 段 | 状態 | 根拠 |
|---|---|---|
| headless export本体 | `BUILT_UNWIRED` | `motolii-export`実在 |
| progress通知 | `ABSENT` | `lib.rs:254-324`が`frames_written`をローカルに積むだけで外部経路なし |
| cancel | `ABSENT` | `ExportJob`にcancel token / flag のfieldなし |
| 完成artifactの公開通知 | `ABSENT` | `ExportReport`は戻り値のみ、CLIは`println!`のみ |
| export設定UI | `ABSENT` | RN側に実装なし |

### 5.6 outcome F — 日常操作

| 段 | 状態 | 根拠 |
|---|---|---|
| delete | `BUILT_UNWIRED` | 既存`RemoveTrackItem` / DeleteTargetedItems |
| duplicate | `PARTIAL` | `duplicate.rs`のprepareは実在、`DomainIntent::ALL`(7件)にduplicateが無い |
| rename | `ABSENT` | `Command`列挙にdisplay_name変更variantなし |
| copy / paste | `ABSENT` | clipboard実装なし（`CopyLocalEffect`は別物） |
| keymap | `BUILT_UNWIRED` | `keymap.rs` / `keymap_codec.rs` |
| **menu → CommandId** | `PARTIAL` | 唯一のmenu実装は`LayoutAction`を直接生成しCommandIdを経由しない（**当初survey`EXISTS_OLD_ONLY`から格下げ**） |

### 5.7 outcome G — panel配置とdiagnostics

| 段 | 状態 | 根拠 |
|---|---|---|
| layout意味 | `BUILT_UNWIRED` | `layout_authority.rs` / `layout_runtime.rs` |
| layout保存・復元 | `ABSENT` | `LayoutAuthority::built_in()`が唯一の構築子。永続load経路なし |
| diagnostic投影 | `PARTIAL` | `adapt_*`は3種のerrorのみ対応、Layout系は非対応 |
| 通常操作からのdiagnostic source | `ABSENT` | production source callが0（CU-204P再確認） |
| a11y tree | `ABSENT` | RN側実装なし |
| activity / telemetry | `ABSENT` | typed snapshot owner不在 |

### 5.8 outcome B〜Gから引かれるM4/M5

- **M4**: outcome D（再生）とE（書き出し）が preview pressure / cache へ触れうるが、
  現時点の呼び出し側には現れない。`R3-PREVIEW-PRESSURE requires=[M4-PROVIDER-TARGET]`が唯一の明示edge
- **M5**: outcome B〜Gのどの呼び出し側にも現れない

→ **M4/M5は現時点のどのoutcomeからも需要が立っていない。** milestone順の先行実装をしない根拠とする。

## 5.9 仮コード合成テストが検出した決定間の合成失敗（2026-08-07）

`Use → Tune → Compose → Inspect → Fork → Author → Publish → Reuse`の呼び出し鎖を書いて
決定同士の同時成立を検査したところ、**14件の合成失敗**を検出した。

[価値観更新](reviews/2026-08-07-m3-integration-zone-value-update.md)§5のsort key
（`concept.md`の完成条件＝3〜5分MVを音楽同期で書き出す）で仕分ける。

### 完成条件を塞ぐ 3件 — 本地図のnodeとして扱う

| # | 種別 | 内容 | 影響するoutcome |
|---|---|---|---|
| T1 | 順序不能 | RN routeでparameterを編集する経路が**どの決定順でも成立しない**。parameter編集はWebView側にしか実装が無く、RN側は read-only decode のみ（`decodeInspectorInitialRead.ts`にgesture型が皆無）。再基線は新規WebView実装を凍結している | A（制作の中心） |
| T2 | 矛盾 | `decision-index`はInspectorを読取専用と要約するが、3日後の正本と現行コード（`InspectorCandidate.jsx:485-577`）はwrite route（`SetPositionKeyValue`/`AddPositionKey`）を実装済みとして閉じている。**索引更新義務が5文書で未履行** | A |
| T3 | 断絶 | `CommandKind::SetEffectEnabled`（`command.rs:302`）は実装済みだが、`ui/`配下に呼び出しが**0件**。「effectをenable/disableする」呼び出し側が存在しない | A |

**T2は記録層の欠陥**であり、実装ではなく索引の是正で閉じる。

### 作者・配布枝 11件 — 本地図のnodeにしない

Fork 4 / Author 3 / Publish・Reuse 4。詳細は
[仮コード合成失敗の記録](reviews/2026-08-07-call-site-sketch-composition-failures.md)。

11件のうち Author / Publish・Reuse の7件は**同一の欠落へ収束する** —
runtime identity と installation path が存在せず、実在するのは compile-time 静的登録のみ。

`concept.md`は解析駆動生成を「優先度=最終フェーズ、M1〜M5完成後」と定めており、
作者・配布枝の後置は既決の踏襲である。**本地図はこれらを実装nodeへ昇格させない。**

## 5.10 鎖のgateが検出した「完成条件を塞ぐ8件」の node 化（2026-08-10登録）

[鎖のgate 6区間](reviews/2026-08-09-chain-gate-results-and-audio-path.md)は完成条件
（3〜5分・音楽同期のMVを1本書き出す）を塞ぐ8件を確定したが、**本地図にも
[implementation-ledger](implementation-ledger.md)にもnodeとして登録されていなかった。**
§5.9の3件は登録され、こちらは登録されなかった — 記録層の欠陥であり、
2026-08-10に本節で是正する。

**登録にあたり、8件すべてを現行mainのコードで再確認した。**
仮コードは[器具境界決定](reviews/2026-08-07-provisional-call-site-sketch-instrument-decision.md)により
非authorityであり、gate結果をそのまま転記するとauthorityへ昇格させたことになる。
下表の判定根拠は仮コードではなく**再確認したコードの実在**である。

| node | gate #番 | 状態 | 現行mainでの再確認 |
|---|---|---|---|
| — | 1 | **解決済み** | CLI subcommandは`ExportOverlay` / `ExportProject` / **`ExportDocument`** / `VerifyB4` / `Help`。`ExportDocument`が`export_document_file`→`export_document_video`（`resolve_audio_export`→`mux_soundtrack` / `mux_mixed_pcm`）へ到達する。commit `97830975`（2026-08-10）で配線済み |
| `N-IMPORT-AUDIO` | 2 | **`ABSENT`**（2026-08-10訂正） | 当初`BUILT_UNWIRED`とし「製品importは常に音声を落とす」と書いたのは**誤り**。`ClipSource::asset_video_only`の8箇所は全て`crates/motolii-ui/src/document_edit_runtime.rs:1135`以降の`#[cfg(test)]`内で、**製品呼び出しは0件**。落としているのではなく**製品import経路が無い**。`build_import_clip_source` + `ImportAvMode::VideoAndAudio`（`crates/motolii-doc/src/audio_edit.rs:21`、`lib.rs:54`でpub）は実在するが呼ぶのは`tests/ag3_audio_commands.rs`だけ。したがって音声の有無は独立nodeではなく**`N-MEDIA-PLACE`施工時の選択事項**である |
| `N-MEDIA-PICK` | 3 | `ABSENT` | 素材を選ぶ入口が`crates/` `ui/`のどこにも無い（file dialog／`rfd`／`pick_media`の実装0件）。リポ外probeのrfd実装は[回収監査](reviews/2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)により**disposable probe debt・製品転記禁止として意図的に非コミット**であり、`UNKNOWN_OUTSIDE_REPO`ではなく`ABSENT`で確定 |
| `N-MEDIA-PLACE` | 4 | **`BUILT_UNWIRED`**（2026-08-10訂正） | `Command::AdmitAsset`（`command.rs:392`）は型としてCommand境界に実在しUndo可能だが、`prepare_admit_asset`を呼ぶのは`tests/m2_asset_lifecycle_commands.rs`だけで、**`crates/motolii-ui`配下に`AdmitAsset`という文字列が0件**。当初`PARTIAL`として「欠けているのは挿入位置とtarget trackのintent」と書いたが、実際は**admission自体が製品から一度も呼ばれていない**。RN側の既存intentは`BrowserPlaceRectangleIntent`（`ui/motolii-rn-legacy/App.tsx:32`）のみで、Browserの品目も`{id:'rectangle'}`固定1件（`ui/motolii-rn-legacy/src/browser/BrowserPanel.tsx:19`） |
| `N-PROJECT-ENTRY` | 5 | **`ABSENT`**（2026-08-10改称・訂正） | 当初`N-PROJECT-NEW` `PARTIAL`としたが範囲が狭すぎた。**NewだけでなくOpenにも入口が無い**。既存projectを開く実装は実在する（`crates/motolii-ui/src/shell.rs:58` `open_project_runtime` → `ProjectSession::open`）が、pathの供給元は`--motolii-project <path>`かenv `MOTOLII_PROJECT_PATH`だけである（`ui/motolii-rn-legacy/macos/MotoliiRn-macOS/AppDelegate.mm:23`）。人間がprojectを選ぶ経路が製品に無い |
| （既存 `T1`） | 6 | 既登録 | §5.9 `T1`と同一。RN Inspectorに編集routeが無い（`ui/motolii-rn-legacy/src/inspector/InspectorInitialReadPanel.tsx`にgesture／onChange 0件）。write routeは`ui/motolii-web/src/candidates/InspectorCandidate.jsx`側にあり再基線が凍結している。**重複nodeを立てない** |
| （既存 `T3`） | 7 | 既登録 | §5.9 `T3`と同一。`Command::SetEffectEnabled`（`command.rs:319`）は実在するが`ui/`配下の呼び出しは0件（`diagnostic_projection.rs:261`の表示名のみ）。**重複nodeを立てない** |
| `N-SOUNDTRACK-WRITE` | 8 | `ABSENT` | `Document.soundtrack: Option<Soundtrack>`（`crates/motolii-doc/src/lib.rs:137`）は実在し、validate・`AudioProgram`（`crates/motolii-audio/src/program.rs:39`）・mux側は揃っている。**`command.rs`にsoundtrackを設定するvariantが1件も無い** — 楽曲bedを作品へ据える編集操作が存在しない |

### 2026-08-10 実装なぞりによる訂正と収束

登録直後に5 nodeの実装を呼び出し側からなぞり、**3 nodeの記述が誤りだった**ため上表を訂正した
（`N-IMPORT-AUDIO` / `N-MEDIA-PLACE` / `N-PROJECT-NEW`→`N-PROJECT-ENTRY`）。
誤りの型はすべて同じで、**型・Command・関数が実在することを「製品にある」と読んだ**ことによる。
`#[cfg(test)]`の内側か、呼び出し元が0件かを確認していなかった。

その結果、media鎖は**端から端まで製品コードが0**であると分かった。
素材を選ぶ → Assetを受け入れる → Timelineへ置く → 音声を含める、のどこにも製品呼び出しが無い。
実在するのは`Command`型層とtest fixtureだけである。

さらに`N-MEDIA-PICK`・`N-PROJECT-ENTRY`（New/Open）・Save Asは
**「native file dialogの席が無い」1件へ収束する**。§5.9でAuthor/Publish 7件が
runtime identity 1つへ収束したのと同じ構造であり、別nodeとして数えると4件、実体は1件である。
`rfd`はgit史全体（`git log --all -S"rfd"`）で0件、リポ外にも製品資産としては無い。
**この収束を確認してからdialog系のnodeを個別発注しない。**

### この節が変えないもの

- **実装許可でも発注でもない。** 状態語彙（§2）に載せただけである
- 仮コード6区間の`NEEDS_REVISION`は未解消のまま。本節は**gate結果のうちコードで再確認できた事実だけ**を昇格させたもので、鎖本体を通過扱いにしない
- 優先度・実装順は§6の原則に従う。`BUILT_UNWIRED`（`N-IMPORT-AUDIO`）は接続、`PARTIAL`は不足特定、`ABSENT`は既知実装調査からであり、本節はこれを繰り上げない

## 6. 実装順の原則

1. `WIRED`は受入だけ。再実装しない
2. `BUILT_UNWIRED`は**接続**。旧routeの意味・command・oracleを再利用し、新実装を起こさない
3. `ABSENT`だけが既知実装調査 → 採択 → 実装の対象。
   ただし**`ABSENT`と判定する前にリポジトリ外の隔離成果を確認する**。
   確認範囲を判定へ併記し、未確認なら`ABSENT`と書かず`UNKNOWN_OUTSIDE_REPO`とする
   （[リポジトリ外資産の棚卸し](reviews/2026-08-08-out-of-repository-asset-inventory.md)§4）。
   **本地図の`ABSENT`のうち外部確認済みは`N-OVERLAY`と`R1-BROWSER`の2件のみである**
   （2026-08-10注: この2件はその後repo内へ進んだ。`R1-BROWSER`は`5b6e6c56`でRN実装がmain到達、
   `N-OVERLAY`は依存のみ着地でコード未移管）
4. `UNDECIDED`は仕様粒で先に閉じる。UIから発明しない
5. M4/M5は**outcome側の呼び出しが現れた時にだけ**引く。milestone順で先行実装しない

## 7. 非目標

- M4/M5の技術採択の再判定
- `BUILT_UNWIRED`を再実装すること
- 背骨が要求しない領域の先行実装
- 旧地図のnode ID・oracle・失敗例の破棄
- `N-OVERLAY`未成立のままoverlay依存nodeを発注すること
