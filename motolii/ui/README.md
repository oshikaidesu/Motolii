# Motolii UI — Stage 5

[開発入口](../../docs/stage5/README.md) · [モジュール責任](../../docs/stage5/modules.md)

`lib/main.dart`は起動、`app`は窓のライフサイクル、`workspace`はDock、`input`は共通操作、`foundation`は部品、`panels`は各面、`session`はEditorSession、`bridge`は通信。作品は本体doc/renderが持つ。

Rust編集層は`native/src/editor`。旧実装は[Git履歴](../../docs/stage5/history/retired-source.md)へ退役済み。内部runtimeはEditorRuntime。既存のC symbolと`motolii/probe`通信名はwire互換で残しており、別の検証用入口を意味しない。

寸法・余白・文字サイズは`lib/foundation/metrics.dart`の`EditorMetrics`から取る。生の数字は`tool/motolii_lints`(analyzer plugin)がIDEで止め、`scripts/motolii-ui.sh test`の`bin/check.dart`が一式で止める。quick fixは同じ値のtokenへ置き換える。Dockの側面幅は`panel_catalog.dart`の`Extent`。

Inspectorは部品版に一本化。[操作契約と検証](../../docs/stage5/inspector.md)。

## UI Gallery

`lib/gallery_main.dart`は製品とは別の起動入口だが、UIの別実装ではない。`EditorApp → EditorWindow → buildPanel` の本番経路をそのまま起動し、Gallery側が持つのはdocumentのfixtureとnative hostの代役だけ。

- ローカル: `flutter run -d chrome -t lib/gallery_main.dart`
- Flutter Widget Previewer: `flutter widget-preview start`
- Pagesの状態: `?story=default` / `?story=dense` / `?story=text`

Galleryから `foundation/`・`panels/`・`workspace/` を直接importしてUIを組み直すのは禁止で、CIが止める。Web buildは同時に、UI経路へFFI実装が漏れていないことも検査する。
