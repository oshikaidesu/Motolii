# CU-0B04N native Stage Surface / layout epoch実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-0B04P`、`CU-0B04NA`

## 1. 完了した境界

macOSの通常project sessionをegui比較baselineからdirect product Hostへ切り替え、
一つのtop-level wgpu Surfaceへnative Stage / Timeline viewportを描画した。

- `GpuCtx::new_for_ui`の同じdevice / queueで既存VRAM display slotを直接sampleする。
  previewのCPU readback、viewport別Surface、別render pathは追加しない。
- 既存`PanelLayout::built_in`と共有する比率からlogical layout epochを生成し、Browserの
  opaque child WebView bounds、aspect-fit Stage image、Timeline viewportを同じepochから
  投影する。resize / scale変更は新epochを作り、Document意味を変えない。
- Host platform captureのrelease候補は最新epochのStage imageだけでhit-testし、
  top-left logical座標をcanonical Y-up NDCへ変換する。本粒ではTransientな
  `PendingStageDrop`までとし、D2 / journal / Undoへ配送しない。
- window lifecycleのraw `WindowEvent`は`CU-0B04NA`のprivate adapterへ閉じ、
  Close / Resize / Scale / Occluded / Redraw以外を製品inputへ昇格しない。
- wgpu 29のmacOS Surfaceは前面化直後にも`CurrentSurfaceTexture::Occluded`を返し得る。
  これを永久`Wait`へ落とさず50ms後に一回再試行し、実際のwinit
  `Occluded(true)`中は停止、`false`遷移で再描画する。busy pollは行わない。
- drop順はSurface、WebView、Windowとし、zero-size / timeout / occludedをskip、
  lost / outdatedをreconfigureする。`pre_present_notify`はpresent直前に置く。
- repo内の同型surface Host spikeに合わせ、top-level windowはopaque・hiddenで生成し、
  visibleなcontent viewへ切り替えてからMetal Surfaceを作る。

## 2. 負例と非目標

React terminal / coordinate、HTML5 DataTransfer終端、default center、egui product
terminal、native viewport別Surface、CPU readback、React semantic state、Document /
selection / Undoの二重正本は追加していない。

Browser focus / reload / crashからのsnapshot再投影、Inspector WebView island、D2への
exactly-once配送、Timeline / InspectorのDocument projection、Undo / Redoは本粒の
成果に数えない。これらは`CU-0B04R`以降へ残す。

## 3. 検証

```text
cargo clippy -p motolii-ui --lib --bins -- -D warnings
passed

cargo test -p motolii-ui native_host_layout --lib
2 passed

cargo test -p motolii-ui --test raw_input_boundary
5 passed

cargo test -p motolii-ui
全test / doc-test passed

cargo test --workspace
全workspace test / doc-test passed

./scripts/check-docs.sh
OK: docs整合チェック全項目通過

MacBook実機 / MotoliiNativeProduct.app
opaque Browser child + green native Stage preview + native Timelineを同一windowで目視確認
```

同梱wryのwgpu統合例、wgpu 29 Metal backend、repo内の同型surface Host spikeを
再照合し、初回表示停止はpipeline / device不整合でなく、on-screen化順とmacOS
occlusion回避後の再試行契機欠落だと分類した。Claude Opus 5のread-only相談は
完全model ID `claude-opus-5`、`--effort low`で完了し、adapter移設を棄却、
既存spikeのopaque・visible-before-Surface順を採用した。通常作業の直列barrierには
しなかった。

次PRODUCT-ASSET `DO`は`CU-0B04R`。本粒で表示したBrowser一島を起点に、
opaque child WebViewのfocus / geometry epochとHost snapshot再投影の入口を閉じる。
