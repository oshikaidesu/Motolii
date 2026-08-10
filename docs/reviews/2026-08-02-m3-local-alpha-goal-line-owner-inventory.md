# M3 固定Mac Local Alpha ゴール線 owner 棚卸し

状態: **観察（次粒選定の入力）**（2026-08-02）

## 目的

固定Mac Local Alphaの利用者ゴール線を、既存の完了receipt／実装決定へ一度だけ写像する。
この文書は新しい製品意味、fixture、task ID、公開契約を追加しない。`CU-5A04`の統合fixtureを
前倒し完了扱いにもせず、未接続の意味を既知targetとして推測しない。

## ゴール線と既知target

| ゴール線 | 既存owner／receipt | 現在の判定 |
|---|---|---|
| 起動・通常製品window | `CU-210R`、`CU-211`、`CU-206C` の通常製品window確認 | 個別routeで確認済み。全長fixtureの一回実行は未確認 |
| Rectangle配置 | `CU-108`（Browser→D2→Stage/Timeline/Inspector） | `DONE` |
| Stage・Timeline・Inspector投影 | `CU-108`、`CU-110PS/PT/PIH` | `DONE`（同一snapshot／primary境界） |
| parameter編集 | `CU-205W/E`（既存Opacity amount route） | `DONE`（Opacityに限定。一般parameter UIへ拡張しない） |
| keyframe追加 | `U4b-0P` | `DONE`（Position 0秒／5秒） |
| Easing | `U4b-1P` | `DONE`（既存popup／Smooth／Undo/Redo／reopen） |
| move・trim・snap | `CU-201E` | `DONE`（同じLayerId、Undo/Redo、reopen） |
| seek | `CU-210P` | `DONE`（paused ruler seek） |
| 再生・終端 | `CU-210R`、`CU-212` | `DONE（縮小採用）`（video-only／mixed playback。authoring／waveformは残件） |
| Undo／Redo | `CU-108`、`CU-205E`、`CU-201E`、`U4b-1P` | 各routeで`DONE` |
| Save／reopen | `CU-211`、`U4b-1P` | `DONE（縮小採用）`（Save-As／in-process reopenは非目標） |
| Export | `CU-211` | `DONE（縮小採用）`（atomic final／ffprobe。cancelは非目標） |

## 閉じていない統合境界

上表は部品ごとの接続証拠であり、起動からExportまでを一つのfixtureで通した受領書ではない。
その一作品と負例の正本targetは既存の`CU-5A04`／`CU-5A05`で、`P11-C1`は次の依存が閉じるまで
`WAIT_CONFLICT`のままにする。

- `CU-G06` fixture manifest
- `CU-309` 実素材save/reopen/Export
- `CU-401A`／`CU-402`／`CU-404`〜`CU-406`
- `CU-5A01`〜`CU-5A03`

これらのWAIT粒を満たすための新しいadapter、fixture-only source、Delete/Rename、IME、panel、
activity、recovery機構は本棚卸しから発明しない。`CU-5A04`を名乗るために一部だけを手動で通すことも
しない。

## 次の選定境界

次の実装targetは、この表の空欄を埋めるために新設しない。`CU-5A04`依存の一つが正本・code事実・
oracleまで閉じた時だけ、その既存IDを再選定する。依存が閉じない間に行えるのは、既存receiptの
再検証または正本docsの同期に限り、Local Alpha全長の完了とは報告しない。

## 2026-08-02 監査記録（Fable read-only照会後）

- 現在のbranch `codex/m3-local-alpha-20260801` / `0cb55444`はcleanである。
- 上表のreceiptは各区間の証拠であり、全長を閉じる受領書ではない。各receiptの縮小範囲を読み合わせずに
  9段のcoverageやM3 Local Alpha完了へ集約しない。
- 全長の既知正本targetは`CU-5A04/P11-C1`だけだが、上記依存が未解決のため`WAIT_CONFLICT`を維持する。
- 本記録は監査入力であり、`CU-5A04`の完了・部分完了、実装発注、dispatch変更を意味しない。依存の
  いずれかが正本・owner・原因・oracleまで閉じた時にだけ、その既存IDを再選定する。
