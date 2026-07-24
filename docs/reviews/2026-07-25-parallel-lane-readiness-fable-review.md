# 並列レーン着手地図 — Fable 5反対側レビュー（2026-07-25）

状態: **初回REVISEを採用し、再審査ACCEPT**。レビュー入力は候補レーン地図と現行の実装台帳、
M3/M4/M5仕様、backlog、Vism計画、Human Response Frontier決定。read-onlyで実施した。

## 判定

- `VERDICT: REVISE`
- P0: 0
- P1: 2

## P1と処分

| 指摘 | 根拠 | 処分 |
|---|---|---|
| PRODUCT-ASSETが現在`DO`の`CU-0A05A / R2A`とWAIT中の`CU-0A05B / R2B`を混同 | R2Aはproduct差分0のmock-side extraction。product ownershipはR2B | [着手地図](2026-07-25-parallel-lane-readiness-map.md)をR2Aの同形React化/parityへ限定し、product比較をR2B後へ移した |
| K0/P0Iの`READY`が台帳の「Uシリーズ直列選択中は同時着手しない」と衝突 | M4/M5仕様は独立spikeを許す一方、発注順の台帳が旧運用を維持 | 着手地図と同じ変更で台帳をlane-local直列へ改訂した |

## P2と処分

- GAP-23とGAP-25はgate script/workflowの変更pathを起動前に照合し、重複時は直列化する。
- K0/P0I fixtureではlegacy/deprecated constructorを使わない。
- `VSM-A4S`/`VSM-A4I`をVism計画へ登録し、A4Iに§8.1全体レビューgateを追加する。
- PRODUCT-ASSETとVISUAL-RESPONSEの`docs/mocks-ui`近接pathを起動前に照合する。
- Human Response FrontierをR2A mock-side parityとR2B product面比較の二段に訂正する。

## 採否

Fableの助言はauthorityではない。Codexが各指摘を現行文書へ再照合し、上記P1/P2を採用した。
訂正後のbounded再審査は`VERDICT: ACCEPT`、P0=0/P1=0/P2=0。R2Aをmock-side parityへ
限定したこと、旧全体直列文言が残っていないこと、4件のP2 guard、M3 lane-local直列性、
後続`WAIT`の維持を確認した。訂正後地図は未成立の実装粒を`WAIT`に保つ。
