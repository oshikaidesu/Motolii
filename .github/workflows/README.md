# CI の方針

**門を作らない。** 2026-08-10 の裁定「main へのマージ段差を全廃・直接 push・
fix-forward」が有効で、**事前検証をマージ条件として再提案しない**。
ここに置くのは **required check ではなく通知**。

## なぜ要るか

2026-08-23、`Asset` に `status` が増えたせいで `motolii-browser-pane` が
壊れたまま main に入り、supervisor が手で `cargo test --workspace` を
回すまで**誰も気づかなかった**(裁定201 の実例)。
**気づく役を人から外す**のがここの目的。

## なぜ台帳の柵だけか

- コードのビルドは iced/wgpu を引くので CI では重く、**いつも落ちる CI は
  無いより悪い**
- 同日の実測で、**腐り15件はすべて台帳側**だった(コードではなく)
- 柵(`owns_justification` / `axis_ledger` / `entries` / `evidence`)は
  Rust + python だけで動き、速い

コードの検収はレーン側の `cargo check --tests`(裁定220)が持つ。
