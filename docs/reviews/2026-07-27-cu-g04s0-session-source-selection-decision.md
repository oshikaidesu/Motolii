# CU-G04S0 VS-1 edit runtime session source 選定

- 日付: 2026-07-27
- 状態: **決定**
- CU-G04S0: **DONE**

## 1. 目的

`CU-109` 実装orderの事前調査で判明した、製品edit runtimeの
`ProjectSession` source未決だけを `CU-G04S` として次のdocs-only粒へ選定する。
本粒はsession source、no-project時の処分、Undo/Redoの暫定処分、smokeの意味を決めない。

## 2. 事実

- `CU-109` は現在の唯一のPRODUCT-ASSET `DO`として実装発注を開始した。
- Opus 5は、`motolii_ui_shell`がpathを受け取らずin-memory bootstrap
  `DocumentWriter`だけを構築する一方、CU-G03 §3.1がrecover可能なbaseを持つ
  `ProjectSession`を要求し、§7がbase作成をCU-G04側lifecycleへ割り当てているため
  `ORDER: STOP`とした。
- CU-G04親は`SPEC / DECIDE`であり、製品session sourceはまだ裁定されていない。
- Fable 5のread-only助言は、CU-109 familyへ意味を移さず、CU-G04を分割して
  session sourceだけをdocsで先に閉じる案を推奨した。
- `CU-G04S`行は現行ledgerに無く、自分のorderで自分を`DONE`へ作ることは
  発注gateを迂回するため、主担当Codexが親task選定として本粒を記録する。

## 3. 選定

1. CU-G04親は`SPEC / DECIDE`のまま維持し、VS-1 edit runtimeのsession sourceだけを
   docs-only `CU-G04S`へ分割する。
2. 次のPRODUCT-ASSET `DO`は`CU-G04S`ただ一件とする。
3. `CU-109`は`WAIT`へ戻し、`CU-G04S`完了後に同じ実装scopeで再選定する。
4. `CU-G04S`の依存は`CU-G03D`、`CU-G03R`、`CU-109S`、`CU-109SP`、
   `CU-109SP-R1`、`D1m`の完了証跡とする。

## 4. CU-G04Sが閉じる問い

以下は次粒の問いであり、本粒の回答ではない。

1. session-backed product entryへ既存project pathをどう渡し、誰が
   `ProjectSession`をopen/保持するか。
2. path欠落、base不在、recovery失敗時にedit runtimeを構築するか。
3. CU-111前のUndo/Redoをどうtyped rejectし、CU-109所有を維持するか。
4. 既存real-binary U2b-1 smokeをsession-backed Apply roundtrip / reopen証拠へ
   どう再配置するか。

## 5. 非目標

- 上記4問の裁定
- Rust、test、fixture、script、package、public APIの変更
- New/Open chooser、Save/Save As、Unsaved Changes、read-only newer、recovery UX
- checkpoint policy、project path codec、永続形式、Document、journal、plugin契約の変更
- `Healthy / Poisoned`具体型、CU-110、CU-111の実装

## 6. 必須負例

- `CU-109`を実装したことにする。
- temp project、optional durability、live-only product edit fallbackを採択する。
- CU-G04親全体を`DONE`へする。
- PRODUCT-ASSET `DO`を複数にする。
- 過去decisionまたは発注依存証跡の既存行を書き換える。

## 7. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-G04S0` | **DONE** | session source未決をCU-G04側の独立docs粒へ選定 |
| `CU-G04S` | **DO** | VS-1 edit runtime session source / no-session / interim action / smoke dispositionだけを決定 |
| `CU-109` | **WAIT** | CU-G04S完了後にruntime配線を再発注 |
| `CU-110` / `CU-111` | **WAIT** | 据え置き |
