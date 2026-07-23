# G0-10 multi-surface window spike

同一画面上にEditorとdetached Previewの2つのtop-level window / wgpu Surfaceを作り、Previewの
close/reopen、片側だけのsurface-lost注入、fullscreen/focus/scale/layoutを計測する隔離spike。
Document、D2、公開plugin API、永続window layoutは一切定義しない。

```bash
cargo test --manifest-path spikes/g0-10-multi-surface-window/Cargo.toml
cargo run --manifest-path spikes/g0-10-multi-surface-window/Cargo.toml -- --auto
```

`--auto`は同一GPU device上に2 Surfaceを作り、Previewだけへ疑似`Lost`を1回注入して再configureし、
fullscreen往復、Preview close、Editorの継続present、Preview reopenを順に実行してreportを書き出す。
report pathは`G0_10_REPORT`で変更でき、既定は
`/tmp/motolii-g0-10-multi-surface-window-report.json`。

疑似`Lost`は実driver障害の再現ではなく、`Surface::get_current_texture`がLost/Outdatedを返した後と同じ
再configure分岐を決定的に通すfault injectionである。
