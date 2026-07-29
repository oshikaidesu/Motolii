# M5 Render Contribution設計締結 — Fable 5最終反対側レビュー

状態: **観察**。2026-07-29に`claude-fable-5`をread-onlyで直接呼び、
[設計締結地図](2026-07-29-m5-render-contribution-design-closure-map.md)、M5仕様、
implementation ledger、各decisionの横断整合を審査した。Fable出力はauthorityではなく、
主担当Codexが現行文書へ再照合して処分した。

## 1. 審判範囲

- 文書間の矛盾と`DONE`／`WAIT`分類
- M4からM5への依存とSTOP線
- 永続schema、公開API、plugin契約への意図しない焼き込み
- 意味設計完了とGPU／public boundary evidence待ちの分離
- 実装粒の直列依存と並列可能範囲

証拠待ちのformat、copy method、budget、public API形を、未決であることだけを理由に欠陥扱いしない。
将来のoptional featureをv1の必須条件へ昇格しない。

## 2. 初回判定

- `VERDICT: REVISE`
- P0=0 / P1=1 / P2=4

P1は`P2D-RCBUD1`の依存同期漏れだった。締結地図はM4-K1に加えて
`P2D-RCFP1F`／`P2D-RCP1`の実byte証拠を要求していたが、M5仕様、
implementation ledger、統合decisionにはM4-K1までしか記載していなかった。
このままでは締結地図以外から、format／methodの測定前にbudgetを閉じられるように読めた。

P2は次の非blockingな文言差だった。

1. `P2D-RCD2I`の`WAIT`と「今すぐ準備可能」の関係。
2. RCD1締結時点の「後続8件はWAIT」が現在形で残っていたこと。
3. RCR1が使うM4-K0既決RoD／RoI意味と、未merge runtime spikeの区別。
4. RCF1I-ALPHAのRCO1依存とv1審判scopeの表現差。

## 3. Codex処分

- RCBUD1の依存を全入口で
  `P2D-RCD1 + M4-K1 + P2D-RCFP1F + P2D-RCP1`へ同期した。
- RCD2Iは`WAIT`を維持し、発注前準備だけが可能だと明記した。
- 「後続8件はWAIT」をRCD1締結時点の歴史記述へ直した。
- RCR1はM4-K0の既決RoD／RoI意味だけを使い、runtime spike mergeへ依存しないと明記した。
- RCF1I-ALPHAはRCO1のv1 dispositionを依存に含め、
  cutout pixelとsoft-alpha typed unsupportedだけをv1 oracleとした。

ticket状態、恒久schema、公開API、Rust item形、plugin契約は変更していない。

## 4. bounded再審査

- `VERDICT: ACCEPT`
- P0=0 / P1=0 / P2=0

Fableは5件すべての解消と、状態変更・恒久面追加・公開API追加が無いことを確認した。
非blockingなP3 nitとして、harness decision §4 F3の「対応時のpixel保証はRCO1後」は、
RCO1がv1方式非採択で閉じた現在、将来scope改訂後と読む方が正確だとした。
同文書§7とM5仕様がv1のsoft-alpha pixelを非目標に固定しているため、現行契約の矛盾ではなく、
Codexが「将来scope改訂と方式採択後。v1は非目標」へ文言だけを同期した。

## 5. 結論

M5 Render Contributionの意味設計締結は、Fable最終反対側レビューでも
P0/P1なしとなった。これはpublic seam、concrete scene-color format、copy method、
resource budget、実装完了を証明しない。それらは締結地図の`WAIT`とSTOP線を維持する。
