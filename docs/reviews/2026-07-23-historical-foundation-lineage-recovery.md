# Historical-only foundation lineageの価値回収（Unit 2A、2026-07-23）

状態: **決定**（歴史文書11 blobの処分、D1n external revisionの再採択、multi-key Graph Viewの再入場状態）

対象: M2/M3入場、keymap、project外部差替え、preview texture lifecycle、workspace、Graph Viewのhistorical-only 7 path。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[負けた仕様の価値回収](2026-07-23-losing-specification-value-recovery.md)、[M2仕様](../specs/M2-document-model.md)、[M3仕様](../specs/M3-ui-integration.md)

## 1. 結論

7 path / 11 blobを全文または初版全文+版間diffで処分した。大半は現在の正本へ吸収済みだが、二つは現行へ戻す価値がある。

1. **D1n external revisionは未回収の採択済み設計だった。** PR #207相当のcommit `580a0c1c`でM2仕様と再締結ゲートへ接続されたが、現在のlineageには文書もtaskも無い。現行`ProjectSession`はMotolii process間lockを持つ一方、main/journal/catalog/generationの保存直前revision preconditionを持たない。非協調sync clientや別editorによる差替えを検出せず上書きし得るため、D1nをM2の独立follow-up決定へ再採択する。
2. **multi-key Graph Viewは未採択である。** 歴史文書の最終版は「製品採択・M3 task化は未決」へ訂正され、現行M3仕様に`U4e`は存在しない。`ui-reference-map.md`の「U4b/U4e正式タスク化済み」を訂正し、区間Easing Graph `U4b`とは別の再入場候補として残す。

M2再締結済みという現在の記録を全面撤回しない。D1nは再締結時に実際に審判した閉集合の外から回収された保存hardeningであり、未実装の間は「非協調external changeを保存前に検出する」「cloud-safe」を公約しない。D1n実装は公開errorとsession mutation境界を変えるため、closed orderと独立レビューを必要とする。

## 2. 個別処分

| 歴史path / blob | 分類 | 判定 | 現在の回収先 |
|---|---|---|---|
| `2026-07-12-M2-order-gate-halt.md` / `8afd4645` | **成立理由 + archiveのみ** | 入場条件と仕様確定条件を混同しない規律、journal/Undo/audio等を一行taskで閉じない規律は現行再締結ゲートとAGENTSへ吸収済み。当時の停止対象・順序・未完了表示は再発効しない | [M2再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)、[恒久焼き込み予防](2026-07-12-m2-permanence-prevention.md) |
| `2026-07-15-keymap-schema.md` / `35aa050e` | **現行規範 + 負例 + archiveのみ** | 全shortcut再割当、Document分離、不変base+delta、version、未知入力保全は現行G0-2/U0dへ精密化済み。旧`gesture_id`、`ContextId`、command単位`replace`、Slint/AccessKit前提、設定path案は戻さない | [M3着手前決定 §2](2026-07-16-m3-preflight-decisions.md#2-g0-2-inputとui状態の意味)、[keymap codec](2026-07-20-m3-keymap-codec-contract.md) |
| `2026-07-15-m3-entry-gate.md` / `a7e84aed`,`5f2bbfda`,`8ede6290` | **成立理由 + archiveのみ** | UIはpixel比較だけで閉じず、依存方向、性能、GPU寿命、cancel、keymapを先に審判する考えは現行M3へ吸収済み。旧Slint、M3E番号、PR #178〜#189のpending状態は発注根拠にしない | [M2再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)、[M3仕様](../specs/M3-ui-integration.md) |
| `2026-07-16-m2-external-revision-decision.md` / `540ca4f9` | **現行規範へ再採択 + 負例** | D1nとして再採択する。正確なbounded bytesのtransient revision、mutation直前precondition、typed conflict、write 0を採る。Document field、watch/mtime権威、自動merge、分散lock保証は採らない | 本書§3、[M2仕様 D1n](../specs/M2-document-model.md) |
| `2026-07-18-m3-preview-lifecycle-disposition.md` / `12ee0882` | **成立理由 + archiveのみ** | GPU保持texture、loop内resource生成禁止、latest generation、nonblocking mailbox、resize/minimize復帰の審判は現行U1a/U1bへ吸収済み。Slint Manual deviceと`Image::try_from`のroute、Metal単一機の合格を現行native/React topologyへ移植しない | [U1a-1契約](2026-07-21-m3-u1a-1-static-viewport-contract.md)、[M3仕様 U1b](../specs/M3-ui-integration.md) |
| `2026-07-18-m3-workspace-customization-decision.md` / `32e4ed85` | **現行規範 + 再入場候補** | Workspace profile所有、toolkit非依存layout、Document/Undo不変、reset、px/monitor ID非保存はG0-2/U1a-2/U1a-3へ吸収済み。名前付きworkspace、複製、import/exportは現行U1a-3完成条件ではなく、codec成立後の追加候補。floating/別monitorはU1eと分離する | [G0-2 §2.2](2026-07-16-m3-preflight-decisions.md#22-状態の持ち場と寿命)、[U1a-2契約](2026-07-21-m3-u1a-2-layout-projection-contract.md) |
| `2026-07-19-graph-view-reference-decision.md` / `d832f1fa`,`ad688be2`,`e433c87c` | **再入場候補 + 負例** | 初版の「Graph View採択」は最終版で明示撤回。実時間×実値のmulti-key俯瞰、Frame Selected、snapshot、de Casteljau key追加、drag/Cancel規律は候補として保持する。現行区間Easing Graphと同名・同task・同状態にしない | 本書§4、[UI参照地図](../ui-reference-map.md)、[M3 U4b](../specs/M3-ui-integration.md) |

## 3. D1n external revision再採択

### 3.1 現行コード事実

`crates/motolii-doc/src/journal/session.rs`の`ProjectSession`は`document_path`、exclusive `lock_file`、`limits`だけを所有する。`save_document`、`save_with_journal`、`migrate_document_file`は保存開始時にopen時のmain/journal/catalog/generation bytesと比較しない。D1mは同じMotoliiの別processとpath aliasを排他するが、advisory lockへ参加しないcloud sync、network peer、別editorを排他しない。

現在の`detect_cloud_sync()`はpath名のhintであり、差替え検出ではない。`DocumentWriter.revision: u64`はprocess内編集世代で、disk familyのexact byte revisionではない。この二つをD1nの代用にしない。

### 3.2 採択する契約

1. `ProjectSession::open/acquire`後の確定時点で、main、journal、catalogと、catalogが参照するgenerationの**正確なbounded bytes**からtransient observed revisionを得る。
2. main/journal/catalogは全project mutationの直前に比較する。generationはcheckpoint、rotate、pin/unpin、recovery/migration等、そのoperationのread/write setに入る時だけ比較する。
3. journal digestはD1dのbounded scanが既に読んだ全bytesを使い、invalid/ignored tailも含める。revision計算のためだけに第二のjournal readを足さない。
4. exact match時だけ既存durability sequenceを実行し、durable resultを再読してsession revisionを更新する。
5. replace、appear、disappear、同一size/mtimeの中間byte変更はcomponent付きtyped conflictとして拒否し、そのoperationのwriteを0にする。malformed、limit、I/O errorは既存の構造化errorを保持する。
6. write後にpeer-wins差替えを検出した場合は別のtyped after-write conflictとし、回復artifactを保持する。

### 3.3 公約しないもの

- non-cooperating peerがprecondition後、atomic replace前へ割り込むTOCTOUを完全に防ぐこと。
- vendor/OS固有の分散lock、自動merge、自動reload、上書き、read-only fallback、Save As UI。
- mtime、inode、file ID、watch event、provider名をrevision authorityにすること。
- SHA-256等のrevisionをDocument、journal record、catalog schemaへ永続化すること。

### 3.4 実装STOP線

D1nは未実装である。次のいずれかが必要なら実装を止め、M2仕様改訂へ戻す。

- public `ProjectSession`/error契約を生文字列や汎用I/O errorへ潰す。
- D1d scan bytesを再利用できず、journal二重readまたは無制限readが必要。
- operationごとのread/write setを閉じられず、全generation全操作hashへ広げる。
- 既存D1d recovery、D1m lock、atomic saveの順序または意味を変更する。
- Motolii-wins raceも検出できるという虚偽のCAS保証が必要。

必須負例はsame-size/same-mtime main差替え、journal middle/ignored-tail変更、catalog/generationのreplace/appear/disappear、precondition malformed/limit/I/O、local連続save、watch無しである。各precondition失敗でmain/sidecar全bytes不変を検査する。

## 4. multi-key Graph Viewの再入場条件

現在採択済みなのは、隣接key 1区間を編集するM3 `U4b`である。multi-key Graph Viewは次を満たす独立taskが仕様へ追加されるまで比較中とする。

1. U4bの区間補間editorと、実時間×実値のmulti-key editorを名前、入口、対象、状態所有で分ける。
2. focus channelとcontext channel、実時間/実値range、selection、snapshotの投影元を既存Document/keyframe意味から定める。
3. pan/zoom/filter/snapshotはDocument・Undo不変。key/handle dragはTransient、release 1 Undo、Escape/capture lossは変更ゼロ。
4. curve上へのkey追加が既存curve形状を保つ意味を、現行`Interp`に対するproperty testで証明する。歴史prototypeのJS modelを製品正本にしない。
5. px/DPI/view range/display normalizationをDocument、評価、plugin契約へ入れない。
6. multi-key同時編集、tangent link、snapshot、Frame Selectedのうち、初回taskの閉集合と非目標を明示する。

`U4e`という未定義IDを参照だけで復活させない。採択時はM3 task表、依存、完了条件、UI参照地図、decision indexを同じ仕様PRで更新する。

## 5. 復活させない旧具体

- Slint 1.17、`unstable-wgpu-29`、`Image::try_from(wgpu::Texture)`を現行UI routeにすること。
- keymapの`ContextId`、`gesture_id`、logical `Ctrl/Alt/Meta`表、command単位replace、具体config path。
- M3E-1〜9や旧PR番号を現在のtask状態へ再利用すること。
- workspaceへSlint component path、window handle、physical px、monitor固有IDを保存すること。
- Graph prototypeのReact/SVG/JS型、palette、viewBox、snapshotをDocument/public APIにすること。
- external revisionをDocument fingerprint、mtime、watch event、provider名で代用すること。

## 6. 固定歴史出典

| lineage | 読み方 |
|---|---|
| M2 order gate | `git cat-file -p 8afd46458d119497876b2f579b99d10581a011a0` |
| keymap schema | `git cat-file -p 35aa050e525dc337d7efbefaba65499b06e6f442` |
| M3 entry gate | 初版`a7e84aed`を全文、`a7e84aed..5f2bbfda`と`5f2bbfda..8ede6290`をdiffで確認 |
| external revision | `git cat-file -p 540ca4f9963c4fe69a2d6233263ebbd2632c33ca`、接続commit `580a0c1c` |
| preview lifecycle | `git cat-file -p 12ee088234546bc499949f65fce6e8dfb3f92845` |
| workspace | `git cat-file -p 32e4ed85afbb84bd6ffcb977d7704db4d67b8cf0` |
| Graph View | 初版`d832f1fa`を全文、format版`ad688be2`、採択撤回版`e433c87c`までdiffで確認 |

これら11 blobは本書でDISPOSITIONEDとする。旧文書を現行正本として直接参照せず、本書の変換とリンク先を経る。
