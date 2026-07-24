# D1m project sidecar / session lineageの価値回収（Unit 4A、2026-07-23）

状態: **処分完了**（cutoff 6 historical blob）

対象: `docs/reviews/2026-07-16-m2-project-sidecar-session-decision.md`のcutoff全版。

関連: [D1m現行決定](2026-07-16-m2-project-sidecar-session-decision.md)、[D1n再採択](2026-07-23-historical-foundation-lineage-recovery.md#3-d1n-external-revision再採択)、[A0S](2026-07-17-vism-a0s-contract-catalog-spec.md)、[M2仕様](../specs/M2-document-model.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

D1mが閉じたのは単なる保存directory名ではなく、project fileとjournal familyを変更できるcapabilityの所有である。

```text
canonical project path
  ├─ <file-name>.motolii/       project固有journal family
  └─ <file-name>.motolii.lock   sibling OS lock
             │
             ▼
      non-Clone ProjectSession
             │ &mut
             ├─ save / checkpoint / recovery mutation
             ├─ document-file migration
             └─ explicit legacy-sidecar migration
```

- 同一directoryの別projectはsidecar、lock、journal、catalog、generationが衝突しない。
- OSのnonblocking exclusive lockを権威とし、PIDや時刻からstale判定してlockを奪わない。
- path mutationは`&mut ProjectSession`またはcrate-privateだけ。catalog必須の製品openはsessionを保持して返す`open_project_resolved`一つに閉じる。
- legacy親共有`.motolii/`は所有者を推測せず、明示操作だけが既知journal familyをstagingへcopy、verify、fsync、atomic installする。
- `Document` schema、UI state、Save As／rename、read-only fallback、分散lockはD1mへ入れない。

## 2. 六版の処分

| blob | 歴史上の追加／分岐 | 現在の判定 |
|---|---|---|
| `bca147db` | D1m初版。project固有sidecar、sibling OS lock、closed legacy state table、session capability | **現行決定の基礎** |
| `013a4691` | D1mのcloud-sync限界を別D1n decisionへ無条件接続 | **D1n系譜の証拠**。リンク先歴史文書は失われたが、現行コードgap再確認後にUnit 2AでD1nを再採択済み |
| `3ecdd2af` | A0Sとのopen/save所有衝突を解消。`ProjectSession::open`をcapability核、`open_project_resolved`を唯一のcatalog製品façadeへ | **現行決定** |
| `7e4f19dd` | legacy unknown entryのlossless/non-persistent diagnostic report、preflight、return mapping、負例を追加 | **現行決定・実装済み** |
| `5f097a6e` | D1m landedとしたcheckpointだが、直前のdiagnostic report節を落とした枝 | **棄却checkpoint**。診断廃止の決定は無く、実装完了正本にしない |
| `5e68a091` | D1m本体とdiagnostic reportの両方をlandedへ更新した枝 | **採用された実装完了系譜** |

cutoff後、D1nの現行コードgap再確認と再採択、A0S/D1mの「追補当時docs-only」時制補正が加わった。これらは6 blobの処分と矛盾せず、現行D1m本文へ統合済みである。

## 3. Legacy migration診断を落とさない

途中checkpoint`5f097a6e`は、statusをImplementation landedへ変える一方で、`LegacySidecarMigrationReport`の仕様全体を削除した。これはunknown entryを黙って捨ててよいという採択ではない。

現行契約は次を維持する。

- 成功は`Installed`または`AlreadyValid`だけ。typed rejectは`Err`でreportを返さない。
- `untouched_legacy_entries`はlegacy root直下のcopy-set以外を、lossy UTF-8変換せずplatform-native `OsString::cmp`順で返す。
- enumerationはmutation前のread-only preflight。失敗時はfilesystem不変でtyped I/O error。
- reportはin-process return valueだけで、Document、journal、catalog、sidecar、wire formatへ保存しない。
- legacy sourceと`.motolii/media`を削除・copyせず、unknown entryもdisk上で変更しない。

現行`ProjectSession::migrate_legacy_sidecar`とD1m legacy migration試験がこの境界を実装している。枝で節が消えた事実を、意味の撤回と解釈しない。

## 4. D1mとD1nを混同しない

D1mのOS lockは協調するMotolii processとpath aliasの競合を防ぐ。Dropbox/iCloud/別editor等、lockへ参加しないpeerによる外部差替えは防がない。

`013a4691`が参照したD1nは一度現行正本から消えたが、Unit 2Aで現在のコードにcompare-before-mutationが無いことを再確認して再採択した。D1m完了をcloud-safe保証またはD1n完了と読まず、D1nのexact-byte transient revision preconditionを別ticketのまま維持する。

## 5. 復活させない案

- parent共有`<parent>/.motolii/`を通常openから自動帰属しない。
- document fingerprint一致だけでlegacy ownerを推測しない。
- lock file削除、PID、mtime、timeoutからlock stealしない。
- invalid／partial／unknown-only destinationへmerge、overwrite、fallbackしない。
- incomplete stagingをactive familyとして読まず、黙って削除しない。
- root-public raw path save/open/recoverやtuple-splitでsession capabilityを迂回しない。
- `load_document*`のcatalog非依存・unknown保持までsessionへ閉じ込めない。
- diagnostic reportをserde化し、恒久schemaへ焼かない。
- D1mへSave As、rename、read-only fallback、distributed lockを便乗実装しない。

## 6. 固定歴史出典とcoverage

初版`bca147db`を全文で読み、D1n枝、A0S ownership追補、legacy diagnostic追補、二つのlanded checkpointへの全差分を確認した。同内容を別branchへ載せたcommitはblob単位で重複countしていない。cutoff後の現行補正blobは最後のdelta単位へ回し、本receiptへ水増ししない。

処分した6 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/04a-d1m-sidecar-session.tsv`を正本とする。cutoff総数1,797のうち処分済みは224、未処分は1,573である。
