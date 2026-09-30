# Motolii UI — Stage 5

[開発入口](../../docs/product/README.md) · [モジュール責任](../../docs/product/modules.md)

`lib/main.dart`がアプリの入口(製品の窓)、`app`はその窓の本体(`window.dart`)、`workspace`はDock、`stage` `timeline` `inspector` `browser` `desks` `effects` `colors` `fonts`は各面、`controls`は共通部品、`theme`はtoken、`input`は共通操作、`session`はEditorSession、`bridge`は通信。作品は本体doc/renderが持つ。

Rust編集層は`native/src/editor`。旧実装は[Git履歴](../../docs/product/history/retired-source.md)へ退役済み。内部runtimeはEditorRuntime。既存のC symbolと`motolii/probe`通信名はwire互換で残しており、別の検証用入口を意味しない。

寸法・余白・文字サイズは`lib/theme/metrics.dart`の`Surface`・`Dn`(共有部品とStageの枠だけ値名の`Step`)から取る。生の数字は`tool/motolii_lints`(analyzer plugin)がIDEで止め、`scripts/motolii-ui.sh test`の`bin/check.dart`が一式で止める。quick fixは同じ値のtokenへ置き換える。

[操作契約と検証](../../docs/product/inspector.md)。
