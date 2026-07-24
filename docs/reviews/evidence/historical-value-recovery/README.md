# Historical value recovery evidence

状態: **観察**

このdirectoryは[全歴史価値回収coverage台帳](../../2026-07-23-historical-value-recovery-coverage-ledger.md)の機械証跡を置く。

## 固定ファイル

- `cutoff-refs.tsv`: cutoff時の`refs/archive/*`と作業checkpointを含む全Git ref名とcommit SHA。
- `corpus.tsv`: cutoff refsから到達可能な`docs/**/*.md`のunique blob。列は`blob_sha / bytes / observed_path`。
- `paths.tsv`: cutoff refsの履歴へ現れたMarkdown pathの集合。
- `read-receipts/`: 全文または限定topicで読了した既存監査の証跡。これは最終処分完了を意味しない。
- `disposition-receipts/`: 現行規範・成立理由・再入場候補・負例・archiveのみのいずれかへ処分したblob。一つのblobは一つのprimary receiptだけに属する。

`observed_path`は`git rev-list --objects`がblobへ最初に対応づけたpathであり、rename全履歴の正本ではない。renameや同一blobの複数pathは`paths.tsv`と個別回収文書で確認する。

## 検査

```sh
./scripts/check-historical-docs-recovery.sh
./scripts/check-historical-docs-recovery.sh --complete
```

通常実行は重複・corpus外参照を拒否して進捗を表示する。`--complete`は未処分blobが一つでもあれば失敗するため、最終回収宣言でのみ使う。

corpusの再生成方法:

```sh
./scripts/build-historical-docs-corpus.sh \
  docs/reviews/evidence/historical-value-recovery/cutoff-refs.tsv
```

再生成結果がcommitted `corpus.tsv`と一致しない場合、cutoff refが欠落したかGit objectが失われている。新しいrefの後発変更は既存corpusへ混ぜず、最終delta単位で別manifestに固定する。

## 意味索引との境界

[意味グラフ補助境界](../../2026-07-23-historical-semantic-graph-recovery-tooling.md)に従い、
このdirectoryとGit objectだけがcoverageの入力正本である。Markdown projection、SQLite、
embedding、検索rank、候補packetは再生成可能な派生物であり、ここへcommitせず、
処分済み判定や`--complete`の代わりにしない。
