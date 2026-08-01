# CU-205E Opacity通常製品route E2E receipt

- 日付: 2026-08-01
- 状態: **E2E DONE**
- 対象HEAD: `7901e51e998ecf114ee43151625b7faf4a0ba24e`
- 対象binary SHA-256: `ed9e338a6a7f9fb9451c5fc2adfdc20c9c9e70f8dceb5ae8e2a9077f6316239c`
- product bundle id: `com.motolii.cu205e`
- executable: `/tmp/MotoliiCU205E.app/Contents/MacOS/motolii_ui_shell`

## 1. 実行面

fixture-only Host、headless helper、diagnostic routeではなく、通常のnative top-level windowと
product Browser / Inspector WebView、native Stage / Timelineを使った。検証用`.app` wrapperは
上記binaryをそのまま起動する一時物であり、製品source、Document、journal形式は変更していない。

## 2. 同一Documentの操作列

1. empty projectを通常windowで開く
2. Create BrowserのRectangleをStageへ配置する
3. Effects BrowserでOpacity cardへfocusし、Enterでprimary Rectangleへ追加する
4. Inspectorの`amount`を`1.0 → 0.89`へ変更してreleaseする
5. Undo 1回で`amount = 1.0`、もう1回でEffect Useをdetachする
6. Redo 1回で同じEffect Use / Definitionをattachし、もう1回で`amount = 0.89`を戻す
7. windowを閉じ、同じprojectを通常windowで再度開く

source seedは`/tmp/motolii-cu205e-final-20260801.json`、SHA-256は
`dc758e46300ddd19cc68d30ee1bb1fded8b71236a13b982a352dbdfd621a8176`。
これは意図どおりempty projectのまま保ち、操作結果は
`/tmp/motolii-cu205e-final-20260801.json.motolii/journal.wal`へ記録した。
journal SHA-256は`328ce0552797fbf08c5958bb1ec34933f0a6a11f9e2f9b26287069db2a27e7b5`。
記録順は`AddTrackItem → CreateEffect → SetProperty(1.0→0.89) →
SetProperty(0.89→1.0) → UndoCreateEffect → CreateEffect → SetProperty(1.0→0.89)`である。

## 3. reopen receipt

2026-08-01 08:44 JSTに同じbinary / projectを再起動し、通常windowのaccessibility treeで次を確認した。

- Browserは`Effects`を表示し、結果1件の`Opacity` cardが存在する
- Stage headerは`STAGE / RECTANGLE`を表示する
- native Timelineとproduct Inspector WebViewが同じwindowに存在する
- Inspectorはactive Effect controlを表示せず、安全な未選択へ戻る

recovery Documentは
`/tmp/motolii-cu205e-final-20260801.recovered-1785541452243567000.json`、SHA-256は
`6a316c045665996afe1946652089cd94d07cd88a27d8cd07201ac6db41d56108`。
typed内容はLayer `0`、Effect Use `0`、Definition `1`、plugin
`core.filter.opacity` version `1`、`amount = F64(0.89)`である。UIのactive Effect Useは
Document / recovery JSONへ存在しない。

再現時は`open -a /tmp/MotoliiCU205E.app --args /tmp/motolii-cu205e-final-20260801.json`
でseedと隣接WALを通常製品runtimeへ渡す。runtimeがrecovery確認を経て上記
`*.recovered-1785541452243567000.json`を出力し、同じnormal windowを開いた。

同時刻の通常window captureは
`/var/folders/b7/y9y9r4qx5y12lnsbzmb29mvh0000gn/T/com.openai.sky.CUAService/Motolii CU-205E Screenshot 2026-08-01 at 8.44.39 AM.jpeg`、
SHA-256は`7de039ad45dfff78e542bb213e8e6560e8cd8d7d8881c1863ae0400c2ceec6fb`。
この一時pathをauthorityにはせず、hashと上記accessibility観測を本receiptへ固定する。

## 4. blocker修復と自動試験

- `581876e0`: exact empty preview graphをtransparent black 1 stepへlower
- `7901e51e`: CreateEffect Redo時だけ、live primaryと一致するEffect Useをactiveへ復元
- `cargo test -p motolii-render`: 37 unit + 16 integration pass
- `cargo test -p motolii-ui`: 168 unitを含む全test pass
- 最終独立read-only code review: P0/P1/P2=0

commitだけをUI E2E証跡とは扱わず、§2と§3の通常window receiptと合わせてCU-205Eを判定する。

## 5. 非主張

- CU-204P、親CU-205、U4a-2の完了
- active Effect Useの永続化
- fixture/headlessだけによるE2E代用
- diagnostic-only routeの通常製品接続扱い
