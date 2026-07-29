# M5 Render Contribution並列Waveの教訓

作成日: 2026-07-29

状態: **観察**

対象: `P2D-RC0`から`P2D-RCD1`までの証拠取得、比較、統合、private spike、typed seam decision。

この文書は実行結果から得た運用上の観察を残す。Render Contributionの意味は
[統合decision](2026-07-29-m5-render-contribution-integration-decision.md)と
[typed seam decision](2026-07-29-m5-render-contribution-typed-seam-decision.md)が正本であり、
本書はそれらを変更しない。

## 1. 実際に成立したWave

| 段 | task | 責任 | 並列性 | 結果 |
|---|---|---|---|---|
| 証拠A | `P2D-RCA8` | Motolii authorityと現行境界の固定配置 | 証拠B/Cと独立 | Grok `ACCEPT` P0/P1/P2=0 |
| 証拠B | `P2D-RCB6` | Rerun固定assetの観察配置 | 証拠A/Cと独立 | Grok `ACCEPT` P0/P1/P2=0 |
| 証拠C | `P2D-RCC5` | provider横断fixture軸の比較配置 | 証拠A/Bと独立 | Grok `ACCEPT` P0/P1/P2=0 |
| 統合 | `P2D-RCI` | 三証拠の採否とMotolii意味の決定 | 三証拠ACCEPT後に直列 | 意味、負例、後続停止線を決定 |
| 実機反例 | `P2D-RCS1` | private opaque Group Depthの成立性 | RCI後に直列 | F1/F6、非波及、共通評価関数を実証 |
| seam決定 | `P2D-RCD1` | 型付き要求とcontributionのseam意味 | RCS1後に直列 | typed seamを決定、後続8件は`WAIT` |

有効だった並列単位は「同じ契約を三者が別々に決めること」ではなく、同じ比較軸へ戻せる
独立証拠を三方向から作ることだった。意味の統合、実機反例、公開観測上のseam決定は一本の
直列背骨に残った。

## 2. 並列化できたものと、できなかったもの

並列化できたのは、入力authority、出力path、観測項目、非証明範囲が互いに交差しないleafである。
RCA8、RCB6、RCC5は同じ結論を競うのではなく、Motolii内部、Rerun、provider familyという異なる
証拠所有面を担当した。この分割により、一つの外部先例がMotolii要件を上書きする経路を作らずに済んだ。

並列化できなかったのは次である。

- 三証拠の採否と語彙失効を決める`P2D-RCI`
- RCIの意味が実GPU depthで成立するかを確かめる`P2D-RCS1`
- private spikeを公開契約の根拠へ昇格させず、公開観測だけを閉じる`P2D-RCD1`

これらを同時に走らせると、証拠が揃う前の採否、実装形からの意味逆算、P3やschemaの先取りが起きる。
したがってWaveの速度はleaf数ではなく、**独立証拠を早く揃え、単一ownerの統合点へ短く渡せるか**で決まった。

## 3. 最初の広いgrainが失敗した理由

旧`P2D-RCA/B/C`では、authority mapping、外部資料取得、転記、比較、裁定を一つのgrainへ寄せた。
結果はRCAのauthority誤引用と概念境界不足によるREJECT、RCB/RCCのcontext枯渇だった。

後続Waveでは次の縮小が効いた。

- 共通authority、固定語彙、code fact hash、非目標を親taskで一度だけ固定する。
- 一leafを`取得 / 転記 / 比較 / 配置 / 裁定`の一動詞へ狭める。
- 外部資料を短いevidence capsuleへ固定し、比較leafでnetworkとrepo archaeologyを繰り返さない。
- 判断を含まないfragmentと、Motoliiへの採否を別成果物にする。
- 失敗したIDとworktreeを再利用せず、新IDをclean baseから開始する。

contextを増やすことより、leafから意味決定と探索責任を取り除くことの方が収束へ効いた。

## 4. shared authorityは一度だけ作り、leafへ決めさせない

三つのleafがそれぞれHost所有、型付き要求、contribution、First Vismを再定義すると、並列数だけ
微妙に異なる正本が生まれる。今回安定したのは、主担当Codexが次を先に所有したためである。

- Motolii仕様と既決decisionの優先順位
- 現行コードの成立／未成立事実とhash
- 比較軸、固定語彙、共通非目標
- Rerun assetの転移裁定
- 最終採否、語彙失効、後続の停止線

leafは固定された席へ証拠を配置するだけにした。これにより外部engineのphase名、registry形、
provider identityをMotoliiの公開契約へ持ち込まずに比較できた。

## 5. REJECTを局所化するとWave全体を捨てずに済む

このWaveでは、authority誤引用、重複行、cardinality、order不成立、表列数、親authorityの古い状態、
固定本文末尾の欠落が独立検収で検出された。いずれも差分を採用せず、原因を一つへ狭めてclean retryした。

有効だった点は、失敗を「ほぼ完成」として統合しなかったことと、無関係なACCEPT済みlaneを巻き戻さなかった
ことである。REJECTはWave全体の失敗ではなく、該当leafの出力契約または施工精度の不足として局所化できた。

一方、同じ意味を保ったまま版を重ねると、モデルレビューが転記・表形・状態同期の検査へ消費される。
意味レビューの前に機械化できる不変条件を落とす必要がある。

## 6. 意味レビューの前へ出すべき機械gate

今回の失敗から、次の検査はGrokより前に置けると分かった。

| 対象 | 機械検査候補 |
|---|---|
| 固定本文 | marker間payloadと成果物のbyte-for-byte比較、末尾改行を含むsize/hash |
| 固定fragment配置 | keyの一意性、期待cardinality、欠落・重複 |
| Markdown表 | 対象行の列数、挿入位置、表内外、次見出し前の空行 |
| ticket完了 | ticket IDのdocs横断検索、親decision・spec・ledger・索引の`DO/DONE/WAIT`整合 |
| allowlist | clean baseからの変更path閉集合、新規file数 |
| authority | manifest fingerprintと実file hash |
| M5 P2D動線 | 締結地図への入口4文書+AGENTS到達、停止済み7 templateの`再発注禁止`とstale状態語不在 |

`check-docs.sh`は索引、link、状態語彙を検査したが、固定本文の一文欠落や複数文書間の
`DO/DONE`矛盾までは検出しなかった。`P2D-RCD1`では主担当がmarker間payloadを抽出して4,430 bytesの
完全一致をGrok前に確認し、再検収はP0/P1/P2=0でACCEPTした。

このbyte比較と横断状態照合は本Waveで有効性を確認した**標準化候補**であり、現時点で
`delegate-cursor-supervised.sh`や`check-docs.sh`へ一般実装済みとは数えない。
2026-07-29に`check-docs.sh`へ追加したのは上表のM5 P2D動線だけであり、
ticket状態の横断推論、byte比較、一般的なworkflow gateを実装済みとは数えない。

## 7. 検収者と主担当の役割

役割分離は次の形で価値を出した。

- Opus 5はclosed order化と、最終の横断authority矛盾検出に有効だった。
- Sparkは意味を決めず、狭いallowlistへ施工する役割に限ると速かった。
- Grokは実diffをread-onlyで検収し、親authorityの古い`DO`、壊れた文、末尾欠落を検出した。
- 主担当Codexは元authority、採否、clean retry、最終統合を所有した。

GrokのREJECTを避けるために審判を弱めるのではなく、機械検査可能な施工不良を前段で除き、
Grokには契約回避、authority矛盾、意味の極性、非目標漏れを見せる方が利益が大きい。

## 8. 実装順から得た教訓

このWaveでは次の順序が効いた。

1. Motolii、Rerun、provider familyの証拠を独立に固定する。
2. 外部先例の多数決を使わず、Motolii authorityで意味と負例を統合する。
3. 公開APIを作る前に、private spikeで最小の実機反例を確認する。
4. spike内部形を公開根拠にせず、公開観測上のseam意味だけを別decisionで閉じる。
5. schema、fixture、alpha、refraction、copy、budgetを同時解禁せず、別契約境界として`WAIT`に残す。

これは「調査後すぐ汎用traitを作る」経路より遅く見えるが、Document、serde、plugin契約、P3型へ
未決を焼かずに、後続が依存できる狭い意味を得られた。

## 9. 今後の利益

- 外部証拠取得は並列化しつつ、製品意味のownerを一つに保てる。
- 広いgrainのcontext枯渇を、情報圧縮ではなく責任分割で避けられる。
- 一つのleafがREJECTされても、ACCEPT済み証拠と無関係laneを保持できる。
- private feasibilityと公開契約を分け、実装都合の恒久焼き込みを避けられる。
- 後続ticketは同じ正本と負例を共有し、engine別APIやFirst Vism専用口を再発明しにくくなる。
- 機械検査を前段化すれば、独立レビューを意味・authority・契約の問題へ集中できる。

次のWaveへ再利用する最小形は、**独立証拠leaf群 → 単一統合decision → private反例spike →
公開観測decision → 後続契約を個別解禁**である。並列数は固定値にせず、入力authority、出力path、
状態ownerが交差しないleaf数だけにする。

## 10. 非目標

- 本書をRender Contributionの新しい意味正本にしない。
- 後続8件を`WAIT`から動かさない。
- Opus／Spark／Grok監督ループやrunner仕様を本書だけで改訂しない。
- byte比較や横断状態照合をrepo-wideに実装済みと宣言しない。
- 特定modelの恒久的な能力・速度評価、固定並列数、一般的な成功率を結論にしない。
- 停止済み差分を採用証拠、ACCEPT済み成果を製品実装完了の証拠にしない。
