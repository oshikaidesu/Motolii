# CU-110PT native Timeline published snapshot投影 実装決定

- 日付: 2026-07-29
- 状態: **決定 / DONE**
- commit: 本文と同一commit

## 1. 結論

通常製品Hostが採用した`current_document`を
[CU-110PT0 envelope](2026-07-29-cu-110pt0-native-timeline-projection-envelope-decision.md)
どおり既存`project_timeline`へ渡し、同じnative wgpu Surface / render passの
Timeline viewportへbarを描画した。

起動時snapshotとPlace成功時published snapshotの両方を同じprivate
`ProductTimelineProjection`へ投影する。別Document snapshot、selection store、
visible-range state、React Timelineは作らない。

## 2. 実装

- `ProductTimelineProjection::from_document`
  - viewport: `ZERO..composition.duration`
  - metrics: durationで正規化する`units_per_second`、unit band
  - 既存`project_timeline`だけを呼び、typed errorを透過
- `ProductApp`
  - 起動時にsession snapshotを投影
  - D2成功時、採用した同じ`current_document`から再投影
  - Stage用render worker、primary、projection generationと別正本を作らない
- `ProductSurface`
  - 既存Timeline backgroundの直後、同じpassでprojection barsをdraw
  - normalized x / band spanを現在のlogical / physical Timeline rectへ写像
  - 1 logical px相当のgapだけをDPIから導出
  - CPU pixel readback、別surface、別renderer、loop内GPU resource作成なし

bar色は既存generated product token `color.way.timeline`
（`rgb(204,149,135)`）と同じ値をnative shaderへ使う。token consumer統合、
visual threshold、golden変更は本粒へ含めない。

## 3. 自動証跡

- `product_runtime` unit
  - composition envelopeがfull-width normalized barになる
  - native Timeline rectへのDPI-aware写像
- `cu110pt_product_timeline_projection`
  - published snapshot採用armが同じDocumentをprojectする
  - native bar draw callerが存在する
  - `TimelineHit`、React Timeline、CPU readbackが同armへ入らない
- 既存`CU-110` / `CU-110PS`構造試験を同時実行し、Place / Stage接続を維持

## 4. 実Mac証跡

- app: `/private/tmp/MotoliiNativeProduct.app`
- project: `/private/tmp/motolii-timeline-110pt-project.json`
- 起動時: 既存1 clipをTimelineの1 barとして表示
- Create Rectangleをnative Stageへdrop
- 再起動なしで:
  - StageへRectangleを表示
  - Timelineがoverlap first-fitの2 barへ更新
- journal:
  `/private/tmp/motolii-timeline-110pt-project.json.motolii/journal.wal`
  は998 bytes

旧バイナリの残存processを検出した最初の画面は証拠から除外し、PIDを限定して終了後、
新バイナリ単独の通常製品windowで起動前後を再確認した。

## 5. 非目標と停止線の維持

- `U3a-2Q-V`は`WAIT`。visible-range state / owner / default /復元を決めない。
- Timeline input、selection、`TimelineHit` caller、focus、playhead、semantic zoom、key描画なし。
- 公開API、Document、serde、journal形式、plugin契約、Undo/history変更なし。
- React `TimelineCandidate`、mock、fixture default、diagnostic route代用なし。

`CU-110PT`は`DONE`。次の唯一のPRODUCT-ASSET `DO`は`CU-110PI`。
