# React parity baseline

比較対象は `docs/mocks-ui/` の `#plugin-browser-candidate` を 1440×900 で描画した画面。Rerunの外観や内部構造は審判に使わない。

## 固定する軸

| Region | React baseline | egui fixture |
| --- | ---: | ---: |
| title bar | 34 px | 34 px |
| command bar | 32 px | 32 px |
| Browser | 284 px | 284 px initial share |
| Inspector | 326 px | 326 px initial share |
| Timeline | 270 px | 270 px initial height |
| status bar | 24 px | 23 px + 1 px border |
| Output Frame | 16:9, about 680×382 px | 16:9, 680×383 px at 1440×900 |

## 機械審判

```sh
MOTOLII_KITTEST_CAPTURE=/tmp/motolii-egui-mock.png \
  cargo test --manifest-path spikes/m3-egui-rerun-mock/Cargo.toml \
  capture_full_mock -- --ignored
```

画像比較と操作契約は分ける。画像側は上表の境界、16:9、選択枠・軌道・bar位置を確認する。操作側はtoolbar選択、Browser 3 tab・検索・選択、Inspector scrub/blend/automation、Stage transport、Timeline選択、panel resizeがmock-local stateだけを変えることを確認する。scrubはdrag中だけPreviewし、releaseで1回だけ履歴へ積み、Escでは開始値へ戻して履歴を作らない。

Reactのcomputed style採寸は`fixtures/react-layout-1440x900.json`へ正規化して保存する。egui側は描画snapshotとは別に、Taffyが割り当てたBrowser内部矩形をtest-onlyで記録し、`browser_taffy_rects_match_react_manifest`で比較する。これにより、font raster差とlayout差を同じpixel diffへ混ぜない。

共通componentへの移行時は、移行直前の1440×900 captureに対するImageMagick RMSEも確認する。構造改善を理由に見た目を変更せず、意図したReact parity修正だけを別に判定する。

## 意図的な差

- ReactのWeb icon fontは持ち込まず、eguiで確実に描画できる文字またはprimitiveへ置換する。
- macOSではReactのfallbackと同じSF UI / SF Monoをsystem fontから読む。font rasterizer差によるsubpixel anti-aliasingだけはpixel一致の対象外。
- Inspectorの値操作は通常sliderではなく、Reactと同じ24 px高の無限目盛scrub（10 px minor、50 px major、中央指標、右35 px値表示）で判定する。
- Document、製品Undo、plugin host、GPU previewには接続しない。Undo/Redo表示は操作文法を比較するmock-local snapshotだけを対象にする。
