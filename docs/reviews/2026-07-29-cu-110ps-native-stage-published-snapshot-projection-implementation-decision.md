# CU-110PS native Stage published snapshot投影 実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-110P`、`CU-110`

## 1. 実装

通常製品Hostが`CU-110`成功時に採用したpublished snapshotを、既存のGPU経路へ再投入した。

- direct product lifetimeに既存`RenderWorker`を一つ保持し、同じ`GpuCtx`で実行する。
- D2 publish後に同じ`current_document`をlatest-only workerへsubmitする。
- result generationがlatest acceptedと一致し、以前のdisplayed generationより新しい時だけ採用する。
- `RenderedFrame`を既存`DisplaySlot::copy`で同じVRAM textureへcopyする。
- render完了cameraをProduct Hostの表示中cameraとして更新し、後続Place変換も表示内容と一致させる。
- worker signalは既存`ProductEvent::Wake`へ接続し、UI threadで同期waitしない。
- event loop終了後はworkerをclose / joinする。

新しいrenderer、display slot、CPU readback、Document clone正本、generation counterは追加していない。
既存Surface bind groupは同じslot viewを参照するため再生成しない。

## 2. 負例

- stale result / duplicate generationはslotへcopyしない。
- worker render失敗、submit失敗、display descriptor不一致はtyped errorで通常製品runtimeを停止する。
- `download_rgba`、buffer map、CPU pixel pathは通常製品sourceに無い。
- Timeline / Inspector / selection / Undo、公開API、Document、serde、journal、plugin契約は変更しない。

## 3. 実Mac証跡

MacBook内蔵画面の通常製品windowを新規project
`/private/tmp/motolii-stage-110ps-project.json`で起動した。

1. drop前のStageは緑一色。
2. Browser CreateのRectangleをnative Stageへdrop。
3. 再起動せず白いRectangleがStageへ出現。
4. session journalは998 bytesでAddTrackItemを保持。

これはjournalだけでなく、同じ起動中のpublished snapshotがrender workerとVRAM display slotへ
到達した証拠である。

## 4. 検証

```text
cargo test -p motolii-ui --test cu110ps_product_stage_projection
1 passed

cargo test -p motolii-ui --lib product_runtime
14 passed

cargo clippy -p motolii-ui --all-targets -- -D warnings
passed
```

## 5. handoff

`CU-110PS`は`DONE`。次の唯一のPRODUCT-ASSET `DO`は`CU-110PT`。
同じpublished snapshotを既存headless Timeline projectionへ渡し、native viewportにbarを描く。
