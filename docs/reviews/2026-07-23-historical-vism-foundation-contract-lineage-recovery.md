# Vism基礎契約lineageの価値回収（Unit 9C、2026-07-23）

状態: **処分完了**（10 historical blob、現行D1m時点補正を含む）

対象: `vism-ready-counter-review-disposition`、`vism-a0d-contract-migration-ownership-decision`、`vism-a0s-contract-catalog-spec`、`vism-a2-legacy-project-migration-decision`、`vism-a7-bpm-datatrack-spike`のcutoff全版。

関連: [A0D](2026-07-17-vism-a0d-contract-migration-ownership-decision.md)、[A0S](2026-07-17-vism-a0s-contract-catalog-spec.md)、[A2](2026-07-17-vism-a2-legacy-project-migration-decision.md)、[A7](2026-07-17-vism-a7-bpm-datatrack-spike.md)、[D1m](2026-07-16-m2-project-sidecar-session-decision.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このlineageが固定したのは「Vismを早く増やす手順」ではなく、外部表現をHostへ入れても作品の保存意味を壊さない基礎境界である。

```text
Document raw recipe（保存正本）
        │
        ▼
immutable Contract Catalog（宣言的に検証・cloneをmigration）
        │
        ▼
prepared recipe（runtime-only）
        │
        ▼
Executor Registry（このHostで実行可能か）
```

したがって、次の不等号を維持する。

```text
保存できる ≠ 意味を検証できる ≠ このHostで実行できる
```

Plugin作者はContractと隣接version migrationの意味を所有し、Hostはimmutable Catalogを組み立て、Documentはraw recipeを未知のままでも保持する。Project openはinstall、network、build、任意plugin code実行を起こさない。Executorの有無は保存可能性と別に判定する。

## 2. 五文書の処分

| lineage | 歴史上の役割 | 現在の判定 |
|---|---|---|
| Vism-ready反対側レビュー 1版 | Opacity／Sine／BPM／Kitを短絡して「Vism-ready」と呼ぶ案を止め、A0→A7→A0D→A0S→A0I→A1/A2→B0/B1/B2へ戻した | **現行の停止線**。二本のpluginやGesture MacroだけでPhase A、consumer、Kit atomicityを証明しない |
| A0D 2版 | raw recipe、作者Contract、Host Catalog、Executor Registryの所有を分離し、A2へのlegacy処分リンクを追加 | **現行決定**。実行codeとcontractの可用性を同一視しない |
| A0S 5版 | 公開型、prepared resolution、状態行列、負例、A0I完了、D1m ownership追補 | **現行仕様**。版差は実装進捗とD1m追補であり、競合する別仕様ではない |
| A2 1版 | 旧CLI `ProjectV1`のSine `amp`互換をprivate declarative adapterへ閉じた | **互換bridgeとして現行**。Document migrationや公開汎用frameworkへ昇格しない |
| A7 1版 | `beat_position(t)=t*bpm/60`を既存DataTrack→DocParamで決定的に評価できることをfixture化 | **意味証拠**。BPM Vism、BeatEvents、tempo map、consumer port、Kitの決定ではない |

A0Sには、同じ初期仕様から分かれた簡潔なA0I完了版と、試験詳細を多く残した完了版があった。現行本文は完了SHAと必須fixtureを保持し、実コード／試験も三段境界を実装しているため、別枝の冗長な進捗記録を新しい規範として重ねない。

## 3. D1m時点補正

歴史版A0Sの2026-07-18追補は、作成当時「docs only、コード未追随」と正しく記録していた。しかし現行正本にもその時制が残り、D1mが未実装であるように読めた。

現行コードでは次を確認した。

- `ProjectSession`は非`Clone`のsession capabilityとしてpath mutationを所有する。
- `open_project_resolved`は`ProjectSession::open`を通り、`ResolvedOpenProjectOutcome`が`session`を保持する。
- `ProjectSession::save_document`、`save_with_journal`、`migrate_legacy_sidecar`が`&mut self`を要求する。
- D1m public API closure、session lock、sidecar path、legacy migrationの専用試験が存在する。

このためA0S、D1m決定、M2 task表、docs入口を「追補当時はdocs-only、その後D1m実装済み」へ補正した。これは契約変更ではなく、歴史時点と現在状態の混同を解消する修正である。

## 4. 復活させない短絡

- OpacityとSineを別crateへ出しただけでthird-party runtime、package、distributionまで成立したとしない。
- 現行`ParamDriver`へ入力portを追加せず、A7をprovider→consumer一般接続と呼ばない。
- 同一Gestureへ複数commandを積むことを、途中失敗時変更ゼロのatomic batchと呼ばない。
- A0D/A0Sから`.vism` manifest、container、loader、install store、version solver、署名方式を逆算しない。
- Contractへ`DocParam`、UI widget hint、任意Rust callback、WASM実行を混ぜない。
- migration成功時もraw recipe、revision、Undo、保存bytesを自動更新しない。
- A2の旧CLI adapterをDocument migration APIやplugin公開façadeへ一般化しない。
- A7から0〜1 phase、BeatEvents、tempo map、meter、Kit schemaを発明しない。

## 5. 固定歴史出典とcoverage

5 pathの初版を全文で読み、後続版は親との差分と変更後の節を確認した。A0Dの後続版はA2処分へのリンク追加、A0Sの後続版はA0I完了・A2リンク・D1m所有追補、および同一初版からの別完了枝だった。ready反対側レビューとA7は別branchに同内容のcommitがあるが、Git blobは各1件である。

処分した10 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/09c-vism-foundation-contract.tsv`を正本とする。cutoff総数1,797のうち処分済みは201、未処分は1,596である。A3表現lineageとinstall/load/trust残余はUnit 9D/9Eへ送り、本Unitの完了でVism全歴史完了とはしない。
