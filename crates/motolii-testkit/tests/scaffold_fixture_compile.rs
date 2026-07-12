//! M2E-10: new-plugin 生成 testkit テストの配置コンパイル検証ハーネス。
//!
//! `MOTOLII_SCAFFOLD_FIXTURE=1` の nested cargo 時のみ、生成した
//! `target/scaffold-plugin-fixture/in_testkit/mod.rs` を取り込む。
//! Cargo feature ではないので `--all-features` では有効化されない。

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[cfg(motolii_scaffold_fixture)]
#[path = "../../../target/scaffold-plugin-fixture/in_testkit/mod.rs"]
mod generated;
