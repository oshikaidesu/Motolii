# CU-G04SC0 VS-1 edit runtime product path handoff 選定

- 日付: 2026-07-27
- 状態: **決定**
- CU-G04SC0: **DONE**

## 1. 目的

`CU-109` 再発注orderの事前審査で判明した、製品binaryからsession-backed
shell entryへのproject path carrier未決だけを `CU-G04SC` として次のdocs-only粒へ選定する。
本粒はcarrier、entry公開境界、argv grammar、failure処分を決めない。

## 2. 事実

- `CU-G04S` は、実session-backed製品shell entryが呼出側から明示的な既存project pathを
  要求し、`ProjectSession`をopen・保持することをD2で決定した。
- 現行の唯一の非test callerは、引数なしで公開`run_shell()`を呼ぶ
  `motolii_ui_shell` binaryである。製品用project path carrierは存在しない。
- `MOTOLII_TEST_U2B1_DOCUMENT`はboolean smoke flagであり、CU-G04S D6は
  test所有project pathへの再係留を事前承認したが、このflagを製品path sourceへ変更していない。
- Opus 5は、test flagをpath sourceにするorder案をCodexが却下した後、既存の非test carrierが
  無いことと、argv・product env・新公開entryのいずれも恒久製品境界の新設になることを確認し
  `ORDER: STOP`とした。
- Fable 5のread-only助言は、0/1 positional argvを最小の実caller候補としつつ、
  carrierと公開entryをdocs-only前提粒で先に閉じることを推奨した。
- `CU-G04SC`行は現行ledgerに無く、自分のorderで自分を`DONE`へ作ることは
  発注gateを迂回するため、主担当Codexが親task選定として本粒を記録する。

## 3. 選定

1. CU-G04親は`SPEC / DECIDE`、`CU-G04S`は`DONE`のまま維持し、D2を実行可能にする
   product path handoffだけをdocs-only `CU-G04SC`へ分割する。
2. 次のPRODUCT-ASSET `DO`は`CU-G04SC`ただ一件とする。
3. `CU-109`は`WAIT`へ戻し、`CU-G04SC`完了後に同じdurability runtime scopeで再選定する。
4. `CU-G04SC`の依存は`CU-G04S`、`CU-G03D`、`CU-G03R`、`CU-109SP`、
   `CU-109SP-R1`、`D1m`の完了証跡とする。

## 4. CU-G04SCが閉じる問い

以下は次粒の問いであり、本粒の回答ではない。

1. 製品binaryからsession-backed entryへ既存project pathを運ぶ唯一のcarrierを何にするか。
2. zero-path diagnostic `run_shell()`を変えずに、session-backed entryをどの公開境界へ置くか。
3. path欠落、複数引数、未知flag、open/recover失敗をどうtyped failureへ閉じるか。
4. `MOTOLII_TEST_U2B1_DOCUMENT`をboolean test evidenceだけに保ち、real-binary
   Apply roundtripをtest-only production seamなしでどう到達させるか。

## 5. 非目標

- 上記4問の裁定
- Rust、test、fixture、script、package、public APIの変更
- New/Open chooser、Save/Save As、Unsaved Changes、read-only newer、recovery UX
- checkpoint policy、project path codec、OS file association、CLI一般化
- 永続形式、Document、journal、plugin契約、Healthy/Poisoned具体型の変更
- CU-109、CU-110、CU-111の実装

## 6. 必須負例

- `MOTOLII_TEST_*`、ambient env、cwd、recent/default pathを製品path sourceとして即興採択する。
- testだけから到達するsession entryを製品callerとして数える。
- temp project、auto-initialize、optional durability、live-only product edit fallbackを採択する。
- CU-G04親または`CU-109`を完了したことにする。
- PRODUCT-ASSET `DO`を複数にする。
- 過去decisionまたは発注依存証跡の既存行を書き換える。

## 7. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-G04SC0` | **DONE** | product path carrier未決をCU-G04側の独立docs粒へ選定 |
| `CU-G04SC` | **DO** | binary→session-backed entryのcarrier・entry境界・failure処分だけを決定 |
| `CU-109` | **WAIT** | CU-G04SC完了後にruntime配線を再発注 |
| `CU-110` / `CU-111` | **WAIT** | 据え置き |
