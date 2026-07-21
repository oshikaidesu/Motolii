# G0-9 native Timeline visual-parity spike

React `TimelineCandidate`を見た目と情報密度のoracleにし、同じsemantic fixtureをnative wgpuで描く隔離spike。
Document、D2、selection正本、製品theme token、公開APIは変更しない。

このnative fixtureが描くのはtime ruler、row同期S/M rail、bar、key、playheadである。Z軸Timeline / depth railも
同じnative所有に含む。React oracleにある`KEYS / LAYERS`切替とAlign等のtool panelはReact chromeに残し、
native fixtureへ複製しない。

```bash
cargo test --manifest-path spikes/g0-9-timeline-visual-parity/Cargo.toml
cargo run --manifest-path spikes/g0-9-timeline-visual-parity/Cargo.toml -- --auto
```

`--auto`は実windowで120 frame present後、計測reportを書いて終了する。
