# D1仕様穴・TimeMap・Generator先例lineageの価値回収（Unit 4C、2026-07-23）

状態: **観察**（cutoff 12 historical blobの処分完了）

対象: `docs/reviews/2026-07-12-d1-spec-holes-prior-art.md`のcutoff全版。

関連: [D1仕様穴の先例調査](2026-07-12-d1-spec-holes-prior-art.md)、[決定パック](2026-07-13-decision-pack-adoption.md)、[M2仕様](../specs/M2-document-model.md)、[M3 Generator](../specs/M3-ui-integration.md#編集時generator-hookone-shot)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

12版は、D1へ焼く前の穴を列挙しただけでなく、TimeMapの単一正本、操作と評価の分離、編集時Generator、公開コーパスという現在も有効な設計理由を残していた。一方、初期提案をそのまま復活させてはいけない。

- **現行へ残す**: キーフレーム評価へTimeMapを通さない。速度変更がkey時刻を伸縮するならD2編集コマンドで決定済み値を書き、key編集からClip尺を暗黙変更しない。TimeMapは専用fieldのまま、製品面だけ一か所に集約する。
- **現行へ残す**: one-shot Generatorは通常Document編集へmaterializeし、script runtimeをpreview/exportへ常駐させない。公開例、恒久URL、検索可能なMarkdownはcreator/developer連続体の供給資産である。
- **縮小して残す**: p5.jsの豊富なコーパスは価値ある入力だが、canvas履歴・座標・runtimeまで互換化しない。現行採択はShapeScriptの正準object model + SVG adapterであり、p5互換は完成後のsyntax sugar候補に過ぎない。
- **再入場させる**: Asset指紋format、A4の全lane重なりvalidator、A6 Tempo/Meter map、TimeMap authoring製品動線は、決定または候補が文書にあるのに現行コード／ticketへ閉じていない。
- **棄却する**: dangling layer参照を幽霊として保存し静的値へfallbackする初期案、追加fieldなら常に`min_reader_version`を上げないという初期案、毎frame live JSをone-shot Generatorと同じ境界に入れる案。

## 2. 十二版の処分

| blob | 主な変化 | 現在の判定 |
|---|---|---|
| `7ca71b4d` | TimeMap、BPM、Asset指紋、layer参照、reader versionの初版 | **成立理由を保持**。個別提案は後続決定と現行コードで再判定 |
| `b6899531` | 編集時generator / live JS分離、波形低優先化 | Generator分離を**縮小採用**。波形は後続AG-3を拘束しない歴史判断 |
| `67bc8c70` | p5.js互換を公開コーパス戦略として提案 | コーパス問題を採用、API互換は後続で撤回 |
| `8b44fc6f` | Cavalry docs非発見性をrobotsだけで棄却せず訂正 | **成立理由を保持**。伝聞は設計根拠にしない |
| `e5b3b9e7` | Cavalry timeline追調査を停止 | **停止判断を保持**。AE/AMのUXをCavalryで再決定しない |
| `49fccb17` | 色/回転/transform A1〜A3、A4〜A8/B群を棚卸し | A1〜A3と後続採択を現行正本へ接続 |
| `7757d71c` | 第二コード監査S1〜S18との重複整理 | 現行コードで再照合。監査当時のfile:lineを現行証拠にしない |
| `c046807e` | gate停止branchでOverrun等の実装追随を反映 | 実装状態は現行codeを正とし、branch固有の完了表示はarchive |
| `1a8291c6` | D1gのOverrun契約を固定 | **採択済み**。Freeze/Black/Loopを保持し、Black/Loopは現行もtyped unsupported |
| `f7e85ab8` | #103決定先とcritical pathを補正 | 歴史的進捗。決定内容は採択文書を正本にする |
| `b075d69b` | #103/#100決定パックを反映 | **現行決定へ採用** |
| `82abe672` | p5互換をShapeScript/SVG/Feedbackへ再分割 | **現行Generator境界へ採用** |

途中の`c046807e`、`1a8291c6`、`f7e85ab8`、`b075d69b`は単純な一直線の上書きではない。commit日が新しい枝の進捗文言だけで現行採否を上書きせず、意味決定とコード事実を別々に照合した。

## 3. 現行コードとの照合

| 主題 | 現行事実 | 処分 |
|---|---|---|
| TimeMap原点 | `Clip.start`とclip-local→source `TimeMap`へ一本化済み。key評価へTimeMapを通さない | 実装済み |
| Overrun | `Freeze/Black/Loop`を保存。render/D3はBlack/Loopをtyped unsupportedとして拒否 | 予約実装済み、実行は未実装 |
| A1〜A3 | 保存sRGB成分補間、radian、多回転、transform順を仕様とgoldenで固定 | 実装済み |
| A4 overlap | `audio_edit`は別laneを要求するが、`Document::validate`は全siblingの区間重複を検査しない | **仕様／コードgap。GAP-18** |
| A6 Tempo/Meter | `Document.bpm`と拍長変換はある。可変Tempo/Meter mapは無い | **決定／コードgap。GAP-20** |
| A8/B①/B④/B⑤/B⑦/B⑧ | stable ID、重複key拒否、3軸flag、閉BlendMode、fps、離散Holdを既存型・試験へ反映 | 実装済み範囲を維持 |
| Asset指紋 | `content_hash/size_bytes/head_hash/tail_hash`は生文字列・任意欄。algorithm/chunk/encoding/versionの型は無い | **候補未締結。GAP-3へ回収** |
| layer参照 | danglingはvalidateで拒否し、parent/spatial cycleはtyped error | 初期の幽霊参照+fallback案を**棄却** |
| reader policy | stable ID、Asset component、Shared Effect、Camera等は意味を旧readerが理解できないためmin reader floorを上げる | 「追加fieldなら常に据置」を**棄却** |
| Generator | U9a〜c / SCR-1〜3は仕様・backlogのみ。runtime実装は未着手 | 境界決定済み、実装未着手 |

## 4. TimeMap authoringを再入場させる

歴史側で決定され、現行schemaには反映されたが製品ticketから落ちた部分をGAP-19へ戻した。

1. 速度、TimeMap、Overrunの編集入口は製品面の「時間」枠へ集約する。ただし永続schema上は順序付きEffect stackへ移さず、Clipの専用fieldを維持する。
2. 速度変更でkeyを追従させる既定は、対象Clip区間とkey時刻を決定済み値へ変換するD2 macroで実装し、1 gesture = 1 Undoにする。
3. keyの追加・移動はClip duration / TimeMapを暗黙変更しない。区間外へ出たkeyも削除しない。
4. Black/Loopは保存できても現行runtimeで未実装なので、UIがFreeze相当として黙って適用しない。

これは新しい公開Command/API、exact panel layout、速度ランプschemaを本Unitで決めるものではない。それらはD2/M3のclosed contractまで停止する。

## 5. Asset指紋候補の回収

初版が残した価値はXXH3やN MiBという未裁定値ではなく、D1 AssetとM4 `source_id`が別々の恒久formatを作る危険の指摘である。GAP-3へ次の決定項目を戻した。

- format version
- algorithm
- head/tail chunk length
- byte encoding
- file size
- optional full-file hash
- collisionまたは部分一致時の追加照合

現行の文字列fieldを理由に意味を推測せず、再リンクとK4の両利用者を揃えてGR-PVを通すまで新しいformatを焼かない。

## 6. Generatorと公開コーパス

初期p5.js案から残すのは「有名APIなら自動生成しやすい」という一時的能力仮定ではない。残すのは、作者が見つけ、forkし、検証できる公開例を蓄積する供給設計である。

- source/example docsはGitHub上の検索可能な恒久Markdownにする。
- docs siteを作るならstable URL、sitemap、SSRまたは同等の発見性を守る。
- one-shot Generatorはtyped D2 batchへmaterializeし、runtime無しでsave/reload/preview/exportを一致させる。
- ShapeScript固有コーパスはMotolii自身が育て、SVGを既存生成コーパスのadapterとする。
- live JS、非clear canvas、Feedbackは同じ実行境界へ混ぜない。

## 7. 復活させないもの

- p5.js canvasの座標、状態、`draw()`履歴まで互換化すること。
- 毎frame JS runtimeをone-shot Generatorの自然な拡張として入れること。
- TimeMapを画像Effect stackの任意位置へ移し、時間写像と画素Effectの順序を発明すること。
- key移動からClip尺を伸ばす不可視な双方向結合。
- Asset指紋のXXH3、chunk長、hex表現を比較なしにdefault化すること。
- dangling layer参照を保存し、静的値へ黙ってfallbackすること。
- 新field/new variantなら旧readerが常に安全だとみなし、reader floorを据え置くこと。
- 「波形は海苔だから低優先」という2026-07-12判断で、後続のAudio Clip waveform要件を撤回すること。

## 8. 固定歴史出典とcoverage

初版`7ca71b4d`を全文で読み、11個のdistinct blob遷移を分岐込みで確認した。処分した12 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/04c-d1-spec-holes.tsv`を正本とする。cutoff総数1,797のうち処分済みは245、未処分は1,552である。
