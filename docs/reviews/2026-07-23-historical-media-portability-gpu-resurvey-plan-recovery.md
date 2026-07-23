# メディア可搬性／GPUベンダ差再調査計画の価値回収（Unit 5F、2026-07-23）

状態: **未実施計画を狭い再入場gateとして維持**

対象: [再調査ラウンド起案](2026-07-16-media-portability-gpu-resurvey-plan.md) cutoff全1版（6,684 bytes）

関連: [backlog](../backlog.md)、[M4仕様](../specs/M4-cache-and-analysis.md)、[Unit 4C D1仕様穴回収](2026-07-23-historical-d1-spec-holes-lineage-recovery.md)、[Unit 5A GPU safety回収](2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

歴史版は調査結果ではなく、先行調査で検証を生き残った主張がゼロだった二領域の**再調査計画**である。AE、Premiere、FCPX、Resolve、wgpu、Blenderについて列挙した仮説を、製品事実や採択済み先例として引用してはならない。この非証明境界を維持する。

計画は未実施のままで、受け手も残っている。ただし「M4ゲート本文の起草前」という旧い一括期日は現行の段階入場と合わない。ラウンド1はGAP-3／7とM4-K4の恒久な指紋、再リンク、package意味を決める前のgateとする。K0のRoD／RoI spikeや、可搬性と独立なK1内部cache契約までは止めない。ラウンド2はINF-3の実機Final再現性方針を閉じる前のgateであり、既存のGPU health検出やGAP-27のtyped分類まで巻き戻さない。

## 2. 歴史版の主張別処分

| 主張 | 処分 |
|---|---|
| 先行調査では可搬性とGPUベンダ差の検証済みclaimがゼロ | **維持**。問題不存在でなく証拠不足を意味する |
| 独立一次資料3票だけを検証済みと呼ぶ | **縮小維持**。再調査の品質gateであり、既存Motoliiコード事実の確認を外部票へ置き換えない |
| メディア6問をGAP-3／7とK4へ渡す | **維持**。指紋、探索、再リンク、packageを一つのpath fieldで代用しない |
| GPU5問をINF-3へ渡す | **維持**。実機差、driver issue、Final再現性、ユーザー診断を分けて調べる |
| M2-D8成立を理由にM4全体を調査待ちにする | **撤回**。現行はtaskごとの段階入場で、可搬性の恒久面だけを止める |
| K0 spikeは再調査を待たない | **維持**。Document指紋やGPU再現性を焼かない独立契約である |
| 調査本体と反対側レビューを別セッションにする | **維持**。ただし両者の存在だけを採択証拠にせず、Motolii正本へ個別転記して初めて決定となる |

## 3. 現行コード事実

### 3.1 メディア可搬性

- `Asset`にはabsolute／project-relative path、file name、`content_hash`、size、head／tail hashの席があり、pathは`/`区切りへ正規化される。
- `resolve_asset_path`はproject-relative、absolute、project直下の同名、`.motolii/media/<content_hash>`の順に**実在ファイル**を探す。
- この関数は候補ファイルの内容、size、head／tail hashを照合しない。同名候補を別素材として拒否する審判、一括再リンク、offline状態、探索結果のD2 command、package manifestも無い。
- 指紋欄は生文字列と任意値のままで、algorithm、version、chunk長、encoding、collision時のfull hash照合が未締結である。M4 `source_id`と別形式を作らない停止線はGAP-3に残る。

よって、path fallbackがあることを「再リンク完成」や「可搬package完成」の証拠にしない。

### 3.2 GPUベンダ差

- `GpuCtx`はadapter info、最低limit、optional timestamp query、device-lost／uncaptured callback、`check_health`を持つ。
- 現行のhealth状態はmutex poisonでpanicし得、uncaptured errorを文字列へ潰す。recoverable frame failure、fatal OOM、device再生成の境界はGAP-27／INF-4に残る。
- lavapipeを使うGPU試験と共通toleranceはあるが、実機ベンダ／backend／driverごとのFinal出力差を収集し、許容または非再現を宣言するINF-3成果物は無い。
- adapter名やdriver issueの存在をブラックリスト、永続設定、plugin契約へ直結する実装は無い。これは未実装であることが正しく、再調査前に追加しない。

## 4. 再入場gate

### 4.1 GAP-3／7・M4-K4

1. まずMotoliiの現行Asset、path解決、D2、K4 `source_id`の未成立点を固定する。
2. 歴史版R1-1〜R1-6を一次資料で再調査し、外部製品の故障例とMotolii要件を別欄に置く。
3. 指紋同一性、候補探索、ユーザー確認、永続path更新、package収集を別責任に分ける。
4. schemaへ席を予約する判断とv1製品実装を分離し、GR-PVを通す。

STOP: XXH3、chunk長、hash encoding、相対path基準、package layout、同名自動採択を調査や意味決定なしにdefault化しない。

### 4.2 INF-3

1. 現行shader、tolerance、CI adapter、実機Final fixtureを先に固定する。
2. 歴史版R2-1〜R2-5を、ベンダ×backend×driver×症状と、外部製品の運用判断に分けて調べる。
3. 同一マシン内の再現、異機種間の許容、bit一致を公約しない範囲、device-lost後の診断を別々に決める。
4. 方針はdocsとfixtureへ閉じ、Document、plugin API、golden値を同時変更しない。

STOP: lavapipe greenを全実機同一の証明にせず、単一driver issueから恒久blacklistを作らず、差が出たGPUへ合わせてsemantic golden／thresholdを変更しない。

## 5. 復活させないもの

- 歴史版の調査質問や落とし穴仮説を、検証済みの外部製品事実として引用すること。
- path候補が存在するだけで内容同一性を成立扱いすること。
- `content_hash`、head／tail hash、M4 `source_id`へ互いに異なる未version形式を焼くこと。
- 「全部内包」または「全部外部参照」を比較なしでpackage正解にすること。
- 可搬性調査を理由にK0／K1など独立したM4内部契約を停止すること。
- device-lost検出の部分実装をINF-3実機再現性方針の完成証拠にすること。
- lavapipeを実機品質の唯一oracle、実機をsemantic golden更新のoracleにすること。
- adapter名、vendor ID、backend型をDocumentや公開plugin契約へ出すこと。

## 6. 固定証跡とcoverage

処分対象は`c68d5662b641e87923336bc684e3a7acc94135ad`の1 blob、6,684 bytes。receiptは`05f-media-portability-gpu-resurvey-plan.tsv`に固定する。

本Unit後のstrict progressは358 / 1,797（19.9%）、未処分1,439である。
