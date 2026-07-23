# 歴史価値回収の意味グラフ補助境界

日付: 2026-07-23  
状態: **決定**  
対象: HVR-G01 / HVR-D01〜D04

## 1. 問題

[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)はcutoff時点の
Markdown blobを1,797件へ固定し、現時点で420件を処分済み、1,377件を未処分としている。
この全量性は必要だが、path名と手作業のtopic分割だけで関連主張を探すと、同じ意味の改名、
撤回、後続実装への一般化を見落としやすい。

一方、embeddingやLLMの類似判定をcoverageへ数えると、「似ている」と「全文を読み、現行へ
処分した」が混ざる。よって意味検索は回収候補を作る補助へ限定し、完了判定には使わない。

## 2. 決定

回収系を次の4層へ分ける。

1. **Git固定corpus**: `cutoff-refs.tsv`、`corpus.tsv`、Git objectが入力正本。
2. **処分receipt**: `disposition-receipts/`と
   `check-historical-docs-recovery.sh`だけがcoverageの正本。
3. **決定的projection**: Git blob本文と機械的な来歴だけを、可搬Markdownとmanifestへ
   losslessに投影する。生成物は派生物でありGitへcommitしない。
4. **任意の意味索引**: projectionを外部のローカル索引へ読ませ、候補packetを作る。
   順位、要約、推論edgeは非規範であり、receiptを自動更新しない。

意味検索後も、LLMまたは人が候補blobの全文、現行docs、現行コードを読み、
[「負けた仕様」の価値回収](2026-07-23-losing-specification-value-recovery.md)の分類を行う。
採択・撤回・再入場・archiveの裁定は従来どおり単位別文書とreceiptで閉じる。

## 3. 決定的projection契約

HVR-D01はPython標準ライブラリとGit CLIだけで実装する。入力は既存の固定corpusとreceipt、
出力先は明示されたrepo外directoryとし、同じcommit・同じ入力からbyte-identicalな出力を作る。
時刻、絶対path、実行環境、embedding、LLM出力を含めない。

出力treeを次へ固定する。`<aa>`はblob SHAの先頭2文字である。

```text
<out>/
├── nodes/<aa>/<40-hex-sha>.md
├── manifest.tsv
└── edges.tsv
```

`<out>`はrepo rootの外側にある、実行開始時に存在しないpathだけを受け付ける。実装は同じ
parentの一時directoryへ全件を書き、検査成功後にatomic renameする。失敗時に`<out>`や
成功扱いできる部分treeを残さない。

blobごとに本文をそのまま持つMarkdown nodeを1件出力する。nodeはUTF-8のYAML front matter、
区切り`---\n`、Git blob bytesの順で連結し、区切り後に改行を追加しない。blobがUTF-8で
なければ拒否する。YAMLの文字列値はJSON string literalとしてescapeする（JSON文字列は
YAML double-quoted scalarのsubsetとして使う）。front matterのkeyと順序を次へ固定する。

- `title`
- `type: historical_blob`
- `permalink`
- `motolii_blob_sha`
- `motolii_bytes`
- `motolii_observed_path`
- `motolii_cutoff_manifest_sha256`
- 処分済みの場合だけ、次の順で
  `motolii_receipt_file`、`motolii_receipt_source_scope`、
  `motolii_disposition_document`、`motolii_publication`

`title`は`<observed_path> @ <SHA先頭12文字>`、`permalink`は
`motolii-history/blob/<40-hex-sha>`とする。`motolii_cutoff_manifest_sha256`はcommitted
`cutoff-refs.tsv` bytesのSHA-256である。receipt用4値は、receipt fileの
repo-relative pathと同じ行の`source_scope / disposition_document / publication`を
exact copyする。

`manifest.tsv`のheaderと列順は次へ固定し、blob SHA昇順で1,797行を出す。値にtab、CR、LFが
あれば曖昧化せず拒否する。処分済み行の`coverage`は`disposed`、それ以外は`remaining`、
処分されていないreceipt 4列は空文字とする。`node_path`はout rootからのPOSIX相対pathである。

```text
blob_sha	bytes	observed_path	node_path	coverage	receipt_file	source_scope	disposition_document	publication
```

`edges.tsv`のheaderと列順は次へ固定し、全列のbytewise辞書順でsortする。

```text
source_id	relation	target_kind	target_id
```

各blobは`blob:<sha> / observed_path / path / <observed_path>`を1行持つ。処分済みblobだけ、
`receipt / receipt / <receipt_file>`、`disposition_document / document /
<disposition_document>`、`publication / publication / <publication>`を各1行持つ。
空のpublicationはedgeを出さない。

本文は`git cat-file`のblob bytesと一致しなければならない。構造edgeは以上の閉集合とする。
`adopts`、`rejects`、`supersedes`、`implements`、spec ID、価値評価などの意味edgeは生成しない。

## 4. Basic Memoryの位置

[Basic Memory 0.22.1](https://github.com/basicmachines-co/basic-memory)を、HVR-D02で検証する
**任意の外部CLI**候補に採る。AGPL-3.0のため、Motoliiへvendor、link、source copyせず、
製品runtime、通常build、CI、coverage検査の必須依存にしない。公式文書とCLIのblack-box挙動だけを
参照し、Basic Memory固有schemaをHVR-D01の可搬projection契約へ入れない。

検証時はversionを`0.22.1`へ固定し、専用の`BASIC_MEMORY_CONFIG_DIR`を使い、
`BASIC_MEMORY_AUTO_UPDATE=false`とする。日本語を含むためembedding候補は
`sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2`へ固定し、日本語fixtureで
関連候補が返ることだけを確認する。rankやscoreは合否・処分順・coverageへ使わない。
installと初回model取得はnetworkを要し得るためopt-inとし、cache済み環境での索引・検索だけを
local実行として扱う。「完全offline install」は主張しない。

Basic Memoryが無い、起動しない、model取得に失敗する場合も、projection、手動検索、
receipt coverageは成立し続けなければならない。

## 5. 機械審判

HVR-D01の実corpus検査は少なくとも次を満たす。

- nodeが1,797件で重複SHAがない。
- 全node本文が対応するGit blob bytesと一致する。
- manifestの処分済み420件、未処分1,377件が既存checkerの結果と一致する。
- receipt ID/pathは既存receiptからexact copyされ、未知SHA・二重処分を拒否する。
- 同じ入力へ2回実行したdirectory treeのhashが一致する。
- 日本語、空本文、front matter風本文を改変しないfixtureを持つ。
- TSV制御文字、非UTF-8 blob、既存またはrepo内の出力先を拒否し、失敗時に部分treeを残さない。

意味索引の検索品質はcoverage審判へ入れない。HVR-D02のsmoke testは外部CLIが利用可能な時だけ
別に行い、skipをcoverage成功へ言い換えない。

## 6. 発注単位と並列化

| ID | 状態 | 境界 | 完了条件 |
|---|---|---|---|
| HVR-G01 | `DONE` | 本決定 | projection、外部索引、receiptの責任が分離されている |
| HVR-D01 | `DO` | 決定的projection | fixture負例と実corpus 1,797/420/1,377が一致 |
| HVR-D02 | `WAIT` | Basic Memory opt-in runner | HVR-D01後。隔離設定、日本語smoke、障害時の縮退を確認 |
| HVR-D03 | `WAIT` | repo-local回収skill | HVR-D01後。候補packetだけを作りreceiptを自動変更しない |
| HVR-D04 | `WAIT` | Unit 5N以降の運用 | HVR-D01〜D03後。候補packetを従来の単位別裁定へ渡す |

Composer 2.5とGrokのread-only調査は論点抽出として並列に行える。実装は各行をclosed orderへ
分け、Terra実装と別担当Grok検収を経る。HVRは履歴回収の独立tooling laneであり、
選択中のM3 Uシリーズの意味・所有境界や順序を変更しない。同じHVR laneでは`DO`を1件だけにする。

## 7. STOPと非目標

次に遭遇したら実装を止めて本決定へ戻す。

- corpus、receipt、Git objectのどれを正本とするか変更が必要。
- embedding、LLM要約、類似rankをcoverageや処分済み判定へ使いたくなった。
- Basic Memoryのsource、内部DB schema、AGPL codeのcopy/vendor/linkが必要。
- 自動生成edgeから現行spec、公開API、Document意味、plugin契約を更新したくなった。
- 生成物、SQLite、embedding、model cacheをGitへcommitしたくなった。

Motolii製品への意味検索機能、中央server、共有vector DB、自動裁定、全履歴の一括PR化、
既存receipt形式の置換は非目標とする。

## 8. 外部確認

- [Basic Memory semantic search](https://docs.basicmemory.com/concepts/semantic-search)
- [Basic Memory configuration](https://docs.basicmemory.com/reference/configuration)
- [Basic Memory technical information](https://docs.basicmemory.com/reference/technical-information)
- [FastEmbed supported models](https://qdrant.github.io/fastembed/examples/Supported_Models/)
