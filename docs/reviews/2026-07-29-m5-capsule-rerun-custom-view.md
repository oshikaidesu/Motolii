# Rerun custom view証拠capsule

状態: **観察** / `FROZEN / DELETE-LATER` / 製品import禁止

- source: Rerun commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`
- file: `examples/rust/custom_view/src/main.rs`
- license: `MIT OR Apache-2.0`
- 削除条件: P2D-RCIで元sourceへの直接引用へ置換後

## 観察

- `App::add_view_class`が新しいView classをHost appへ登録する。
- 同じapp組立でarchetype reflection、component UI、data receiverも登録される。
- 既存Viewの能力追加と、新View追加は別の登録動線である。

## 非証明範囲

RerunのApp／ViewClass／Blueprint／component UI責任はMotoliiのHost／plugin／Document境界を証明しない。
View完成品、UI state、store型、登録APIを転記する根拠にしない。
