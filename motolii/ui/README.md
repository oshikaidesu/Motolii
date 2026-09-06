# Motolii UI — Stage 5

[開発入口](../../docs/stage5/README.md) · [モジュール責任](../../docs/stage5/modules.md)

`lib/main.dart`は起動、`app`は窓のライフサイクル、`workspace`はDock、`input`は共通操作、`foundation`は部品、`panels`は各面、`session`はEditorSession、`bridge`は通信。作品は本体doc/renderが持つ。

Rust編集層は`native/src/editor`。旧`motolii/src/ui`は履歴参照で二重保守しない。内部runtimeはEditorRuntime。既存のC symbolと`motolii/probe`通信名はwire互換で残しており、別の検証用入口を意味しない。
