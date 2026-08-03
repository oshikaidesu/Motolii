# M5-D0 stable instance evaluator fixture

製品workspace外のdeterministic fixture。explicit source／parent／depthからslot keyとInstanceIdをstable
mixerで求め、`rand_pcg::Pcg32`でchannelを生成する。count増減、reorder、nested、thread順のidentityと
golden vectorを検証し、OS entropy／時計／GPU順には依存しない。

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-D0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-D0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-D0/Cargo.toml
```
