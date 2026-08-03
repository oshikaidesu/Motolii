# M5-T0 text stack comparison

製品workspace外のstandalone probe。現行Fontique＋HarfRust単一font shapingと、Parley 0.11の
font fallback／BiDi／cluster layoutを同じCJK＋Latin＋emoji＋RTL fixtureで比較する。
Parleyはprivate比較対象であり、MotoliiのP6公開APIやVello接続をこのprobeから発明しない。

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-T0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-T0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-T0/Cargo.toml
```

fixtureはCJK、Latin合字／結合文字、RTL、emoji ZWJ、未知scalarを含む。itemize／BiDiは手書きせず、
現行経路はHarfRust、比較経路はParley＋ICU4Xへ委ねる。variation settingもleafへ渡すが、検証hostの
選択fontにvariable axisがなければ`variation_runs=0`を不足の診断として扱う。
