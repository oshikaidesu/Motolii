# blind evidence envelope反例観察

状態: **観察**（外部reviewer共通方式の実測根拠。provider固有性能の一般保証ではない）

日付: 2026-08-03

## 問い

意味を要約せず同じ原文rangeを一artifactへ機械連結したblind evidence envelopeは、複数Readよりturn、wall、costを減らしながら
判定を維持できるか。また、選択範囲外に競合authorityがある反例で、全hit inventoryと`EVIDENCE_GAP`は監督の選択バイアスを
局所化できるか。

## 速度比較

完全model ID `claude-fable-5`、effort `low`、fresh session、Read-only、同じ判定問いと原文rangeを固定した。Aは7 rangeを個別Read、
Bは同じrangeを意味要約なしで機械連結した一つのenvelopeを1回Readした。各2回の有効試行はすべて`ACCEPT`、P0/P1=0だった。

| 条件 | 反復 | natural turn | 平均wall | 平均cost | 判定 |
|---|---:|---:|---:|---:|---|
| A: 7 range個別Read | 2 | 各8 | 17.771秒 | $0.427977 | 2/2 ACCEPT、P0/P1=0 |
| B: 単一blind envelope | 2 | 各2 | 11.782秒 | $0.304853 | 2/2 ACCEPT、P0/P1=0 |

このfixtureではturn 75.0%、wall 33.7%、cost 28.8%を削減した。単発の速度差を全providerや全粒へ外挿しない。worktree外packetの
Read permissionで停止したBの初回1試行は内容未読のため無効として除外した。

## 選択バイアス反例

合成authorityへ、`CLOSED=low / WIDE=high / incomplete packetはSolへ戻す`というCandidate Aと、範囲外に`全final reviewをlowへ固定し、
不足はrepo自由探索で補う`という競合Candidate Bを置いた。repoや現行Motolii policyは変更せず、一時fixtureだけを使った。

1. Candidate Aだけを収録し、その選択をmanifestへ正直に記録したnaive envelopeは`ACCEPT`せず`EVIDENCE_GAP`を返した。ただし要求は
   「Candidate A以外全部」と広かった。2 turn、8.249秒、$0.068560だった
2. 同じ原文にliteral query scope内の全hit inventoryとsource hashを加えると、`EVIDENCE_GAP`は正確にsource 17–22行だけを要求した。
   2 turn、7.031秒、$0.068090だった
3. 要求されたexact原文だけを追加したfresh sessionはCandidate A/Bの競合を`REJECT`し、P0=2/P1=1/P2=0だった。2 turn、
   12.024秒、$0.070524だった

期待した`EVIDENCE_GAP → exact range追加 → REJECT`が成立した。naive envelopeでも収録範囲を隠さなければ誤ACCEPTを避けられ、
全hit inventoryは不足要求を局所化した。

## 採用できる解釈

- 意味要約でなくexact原文を一つへ機械連結すれば、reviewerの複数Readを減らせる
- manifestは収録範囲を正直に示し、完全性を主張しない
- literal query／symbol／anchorとそのscope内の全hit inventoryをcoverage witnessにできる。ただしquery外の意味的完全性は証明しない
- 未収録の関連hitがあれば`ACCEPT`を禁止し、exact rangeの`EVIDENCE_GAP`へする
- Solがsource/range/hash一致とquery選択を所有し、追加原文はfreshな短waveへ渡す

## provider適用

envelope、manifest、hash、hit inventory、`EVIDENCE_GAP`、fresh waveはprovider固有toolに依存しないため、FableだけでなくOpus／Grokを
含む外部LLM reviewerの共通方式にできる。Fable lowでは動線と反例捕捉を実証した。Opus／Grokで未較正なのはnatural turn、cost、
schema遵守率、permission、final event位置等の効果量であり、方式の適用可否ではない。初回数粒で自然観測し、問題が出たproviderだけ
補正する。

## 限界と非目標

- 合成反例1件と同一policy reviewの2反復で、未知の全選択バイアスや将来provider品質を保証しない
- hit inventoryは記録したquery scopeだけを証明し、semantic searchの完全性を主張しない
- reviewer自身のhash一致確認に依存せず、Solの起動前preflightを必要とする
- 自由repo探索、全file読込、receipt DB、model score、新しい専用runnerを復活させない
- Opus／Grokの未較正を固定fallback、適用停止、過大effortの理由にしない
