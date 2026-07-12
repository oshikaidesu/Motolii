//! M2E-10: nested scaffold コンパイル専用 cfg。
//!
//! Cargo feature にしない(`--all-features` でクリーン checkout が壊れない)。
//! 検証テストだけが `MOTOLII_SCAFFOLD_FIXTURE=1` を立てて有効化する。

fn main() {
    println!("cargo:rerun-if-env-changed=MOTOLII_SCAFFOLD_FIXTURE");
    println!("cargo:rustc-check-cfg=cfg(motolii_scaffold_fixture)");
    if std::env::var_os("MOTOLII_SCAFFOLD_FIXTURE").is_some() {
        println!("cargo:rustc-cfg=motolii_scaffold_fixture");
    }
}
