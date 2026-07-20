# M3 egui / Rerun pattern mock

既存React fixture `#plugin-browser-candidate`を、製品統合前にRust/eguiで比較する実コードprototype。
このcrateはroot workspaceから隔離し、Document、公開API、plugin契約、永続形式を定義しない。
Reactとの固定寸法と比較方法は[PARITY.md](PARITY.md)を参照。

## Rerun転移記録

1. `MOTOLII AUTHORITY`: `M3 U0e / U1a / U3a / U4a / U4d / U6`。画面と操作の参照はReact fixture、状態所有と完成条件はM3仕様を正とする。
2. `CODE FACT GAP`: 現行`motolii-ui`はU0a〜U0dの境界骨格までで、Browser / Stage / Inspector / Timelineを一画面で操作できるegui shellは未成立。
3. `RERUN EVIDENCE`: Rerun `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`の`re_viewport_blueprint`による`egui_tiles::Tree`投影と、`re_ui`のdense list / selection / property row / fixture testingを参照する。Rerunの製品意味、Blueprint、store、Entity、Time、theme値を証明しない。
4. `TRANSFER CLASS`: `DEPEND` = 採択済み`egui_tiles 0.16`、`PATTERN` = dense list / selection / property row / fixture testing。
5. `TRANSFER LIMIT`: このspike配下だけを変更する。`re_*`依存、Rerun font/icon/theme/schema、Rerun `Tree`/`TileId`の保存、Document・公開API・plugin契約への型追加を禁止する。
6. `MOTOLII ORACLE`: React `#plugin-browser-candidate`の1440×900 fixtureと同じ情報階層を表示し、Browser検索・選択、Inspector値変更、Timeline選択、panel resizeが操作可能であること。状態変更はmock内部だけでDocument/Undoへ接続しない。

## 実行

```sh
cargo run --manifest-path spikes/m3-egui-rerun-mock/Cargo.toml
```

比較撮影（test-only helper）:

```sh
MOTOLII_KITTEST_CAPTURE=/tmp/motolii-egui-mock.png \
  cargo test --manifest-path spikes/m3-egui-rerun-mock/Cargo.toml \
  capture_full_mock -- --ignored
```
