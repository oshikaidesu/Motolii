# G0-9 native Multi-key Graph View spike

固定React `GraphViewCandidate`の3 channel fixtureを、Blender利用者に馴染むheader / channel list / graph /
status配置へdirect wgpuで投影する。Blender GPL source、icon、shader、定数、内部型は使用しない。

```bash
cargo test --manifest-path spikes/g0-9-graph-view/Cargo.toml
cargo run --manifest-path spikes/g0-9-graph-view/Cargo.toml
```

操作は次のとおり。

- key / tangentを左drag: 4 logical pxを越えてからTransient更新、releaseで1 commit、Escで開始値へ戻す
- 空白を左drag: marquee選択。Shiftを押した開始は既存選択へ加算する
- wheel: cursor-anchor zoom
- middle drag: pan
- `Home`: Fit All、`F`: Fit Selection、`S`: fixture用frame snap切替

pan / zoom / fit / selectionはDocument相当fixtureとsemantic commitを変更しない。座標変換だけを
`understory_view2d 0.1.0`へ委ね、stable ID、選択意味、snap、gesture lifecycleはfixtureのheadless核が所有する。
