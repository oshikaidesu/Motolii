# 歴史価値回収の意味グラフ補助境界

日付: 2026-07-23
状態: **決定**
対象: HVR-G01 / HVR-D01〜D04

## 1. 問題

[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)はcutoff時点の
Markdown blobを1,797件へ固定している。処分済み／未処分の現在値はreceipt正本から`check-historical-docs-recovery.sh`が算出し、coverage台帳に単位ごとのsnapshotを記録する。
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

### 4.1 HVR-D02 runner契約

repo-local runnerはPython標準ライブラリだけで実装し、次のCLIだけを持つ。

```text
python3 scripts/historical_semantic_index.py index \
  --repo-root <Motolii> --projection <HVR-D01出力> --state <生成state>
python3 scripts/historical_semantic_index.py search \
  --repo-root <Motolii> --state <生成state> --query <UTF-8文字列> [--page-size N]
```

`--projection`と`--state`は絶対pathかつrepo外で、互いの内側ではならない。`index`は
`manifest.tsv`、`edges.tsv`、`nodes/`を要求し、開始前後の全file path＋bytes SHA-256が
一致しなければ失敗する。Basic Memoryへprojectionを書かせないため、全subprocessへ次を固定する。
`PATH`と`HOME`だけは起動元から継承する。`HOME`は`uvx`の依存buildが利用者のtoolchain managerを
解決するためであり、Basic Memory、embedding、uvの設定・cache先は次の専用stateへ上書きする。

- `BASIC_MEMORY_CONFIG_DIR=<state>/config`
- `BASIC_MEMORY_AUTO_UPDATE=false`
- `BASIC_MEMORY_SYNC_CHANGES=false`
- `BASIC_MEMORY_ENSURE_FRONTMATTER_ON_SYNC=false`
- `BASIC_MEMORY_SEMANTIC_SEARCH_ENABLED=true`
- `BASIC_MEMORY_SEMANTIC_EMBEDDING_PROVIDER=fastembed`
- `BASIC_MEMORY_SEMANTIC_EMBEDDING_MODEL=sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2`
- `BASIC_MEMORY_SEMANTIC_EMBEDDING_CACHE_DIR=<state>/models`
- `UV_CACHE_DIR=<state>/uv-cache`

起動commandは常に
`uvx --from basic-memory==0.22.1 basic-memory`とし、任意の`--offline`指定時だけ
`uvx --offline`を加える。project名は`motolii-historical-recovery`へ固定する。`index`は
`project add <name> <projection> --local --default`、`reindex --project <name> --full`、
`status --project <name> --wait --timeout 300 --json --local`の順で成功を要求する。
同じstate/projectの再登録が「already exists」でexit 0となる0.22.1のblack-box挙動は許容する。

成功後だけ`<state>/hvr-index.json`へ、schema `1`、Basic Memory version、embedding model、
project名、projection tree hashを記録する。時刻やrepo pathは記録しない。`search`はこのmarkerを
照合し、`tool search-notes <query> --hybrid --project <name> --local --page-size <N>`のstdoutを
そのまま返す。`page-size`は1〜100、空queryは拒否する。外部CLIのJSON/DBをMotoliiが解釈して
coverageや裁定へ変換しない。

unit testはfake `uvx`で引数・環境・失敗伝播・projection不変・marker・offlineを固定する。
実Basic Memoryとmodel取得を伴う日本語smokeはopt-inで別実行し、CIへ入れない。smokeは
日本語fixtureをindexし、hybrid searchが候補を返すことだけを確認する。順位・score・先頭一致を
期待値にしない。

### 4.2 HVR-D03 repo-local skill契約

repo-local skillは`skills/historical-recovery/SKILL.md`を入口とし、skill名を
`historical-recovery`へ固定する。「昔の案」「負けた仕様」「消えた構想」「以前の判断」など、
現行正本だけでは答えられない設計考古学と再評価を依頼された時に使う。現行codeや正本だけの検索、
receipt coverageの全件監査、未処分履歴の一括裁定、通常の実装作業には起動しない。

skillは次の順序を変えない。

1. `docs/decision-index.md`、本書、主題に対応する現行正本とcode事実を先に読む。
2. HVR-D01 projectionをrepo外の一時directoryへ新規生成するか、同じ契約で生成済みの
   repo外projectionを検証して使う。tracked file、既存projection、receiptは変更しない。
3. 利用者がrepo外のHVR-D02 stateを明示し、有効な`hvr-index.json`がある場合だけ
   `scripts/historical_semantic_index.py search`を使う。skill自身はBasic Memoryのinstall、
   model取得、全量`index`を暗黙に開始しない。
4. 意味索引が無い、失敗した、候補が弱い場合は、query語とその表記揺れを明示した上で
   `rg`によるprojection本文検索と`manifest.tsv`のSHA/path参照へ縮退する。縮退をcoverage失敗や
   「該当なし」の証明にしない。
5. 候補を最大20 blobへ重複排除し、repo外の一時Markdownへcandidate packetを作る。各候補は
   `blob SHA`、当時path、projection node path、取得経路、候補にした理由、既存receipt処分の有無を
   持つ。score、順位、要約を採否や処分順へ変換しない。
6. packetに入れた候補だけ本文を全文で読み、現行正本・code事実と比較する。各候補を
   `観察 / 比較中 / 決定 / 棄却 / 停止`のいずれとして提案し、根拠SHA/path、現行との関係、
   未読範囲と非証明範囲を併記する。
7. packetをユーザーへ返して止まる。receipt、decision index、spec、ledger、codeを自動変更せず、
   採択・再回収はHVR-D04以降の通常の単位別裁定へ渡す。

packet冒頭にはtopic、query語、取得mode（`semantic / lexical / mixed`）、projection tree hash、
corpus総数、処分済み数、未処分数、候補数を記録する。候補0件でもpacketを作り、検索語、検索範囲、
意味索引の有無を残す。「全履歴を読んだ」「網羅した」「価値が無い」とは書かない。たとえば
「AviUtl catalogとhostless配布案を探す」「single camera / 2.5Dの負けた仕様を再評価する」
「Vism Kit構想から消えた価値を探す」は対象である。

projection生成がHVR-D01審判を通らない、候補SHAをGit objectとして読めない、現行正本が未統一、
receipt処分と候補本文が矛盾する、または候補から公開API・永続形式・Document・plugin契約を
直接更新したくなった場合は`停止`として返す。意味索引の再構築、候補上限超過、receipt更新、
正本改訂をskill内の便利な自動処理で補わない。

## 5. 機械審判

HVR-D01の実corpus検査は少なくとも次を満たす。

- nodeが1,797件で重複SHAがない。
- 全node本文が対応するGit blob bytesと一致する。
- manifestの処分済み／未処分件数が、その時点の既存checker出力と一致する。
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
| HVR-D01 | `DONE` | 決定的projection | fixture負例、実corpus 1,797件、現在のreceipt件数がcheckerと一致 |
| HVR-D02 | `DONE` | Basic Memory opt-in runner | HVR-D01後。隔離設定、日本語smoke、障害時の縮退を確認 |
| HVR-D03 | `DONE` | repo-local回収skill | HVR-D01後。候補packetだけを作りreceiptを自動変更しない |
| HVR-D04 | `ACTIVE` | Unit 5N以降の運用 | Unit 8A完了。候補packetを従来の単位別裁定へ渡し、残る単位は1件ずつ処分 |

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
