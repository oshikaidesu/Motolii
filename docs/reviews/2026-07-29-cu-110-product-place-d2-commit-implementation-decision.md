# CU-110 通常製品Place D2 commit接続実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-102`、`CU-107`、`CU-109`

## 1. 実装

通常製品Hostのprivate `PendingStageDrop`を、既存のsession-backed
`DocumentEditRuntime`へ直結した。

- 表示中cameraでStage NDCをcanonical positionへ戻す。
- `PlaceRectangleRequest`を一件だけqueueへ積み、`process_next`を一回だけ呼ぶ。
- 既存plannerがfresh `LayerId`を予約し、`AddTrackItem`一件をjournal commit後に
  `apply_macro`一回でlive Documentへ適用する。
- 成功時は同じpublished snapshot / primary / projection generationをProduct Hostが採用する。
- cancel / outside / capture loss / duplicate / staleはD2へ入らない。

playheadは現行direct productが表示する静止frameと同じ`RationalTime::ZERO`であり、
新しい公開API、汎用transaction、raw ID mint、transport identity保存は追加していない。

## 2. 実Mac証跡

MacBook内蔵画面の通常製品windowでBrowser Rectangleをnative Stageへdropした。
focusはnative Hostへ移り、session journalは404 bytesから1354 bytesへ増加した。
記録は`AddTrackItem`一件、fresh `layer_id: 1`、canonical position、
playhead 0、`Rectangle`名を持つ。

## 3. 次

`CU-110`を`DONE`とする。次PRODUCT-ASSETはselection / 三面projectionと
Undo/Redoの既決依存を再照合し、背骨順を維持して選定する。
