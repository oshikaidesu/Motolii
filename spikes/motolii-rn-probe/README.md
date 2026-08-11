# MotoliiRnProbe(2026-08-10回収)

RN製品sourceへその場で繰り上げ中の接続probe。`App.tsx` 660行 — Browser 3タブ(`MEDIA`/`EFFECTS`/`CREATE`)、
Inspector/Extensions、Timeline 3モード、effect一覧、panel registry、
native `MotoliiGpuComponentView.mm`、Fabric spec `MotoliiGpuView`/`MotoliiTimelineView`。

元所在は `~/Documents/Codex/2026-08-06/ui-rust-ui-c-react/work/MotoliiRnProbe/` で、
[2026-08-10回収監査](../../docs/reviews/2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)により
リポジトリへ移管した(node_modules/Pods/build/target/vendor/.yarn除外。`yarn install`で復元可)。
以後の正本はこのディレクトリ。RN標準のREADMEは`README.upstream.md`へ退避。

**状態境界**: このdirectoryが現在のRN／Rerun／Skia接続targetである。接続成功時はcodeを別targetへ
copyせず、同じartifactの状態を`PROBE`から`PRODUCT_SOURCE`へ繰り上げる。directory名は由来を示すだけで、
製品化前のrename／移植を要求しない。`ui/motolii-rn`は利用者が明示的に解凍するまでread-onlyの旧shellである。
固定fixtureは製品意味の正本にせず、Document／D2入力へ接続できた箇所から置き換える。

## Rerun Stage共通評価probe

`native-renderer/src/renderer_core.rs`の`encode_rerun_stage_shapes`は、B001のRect／Circleを
一つのRerun評価関数から既存wgpu textureへencodeする。実StageとGPU testは同じ関数を使う。
testは同じdevice／queue上の2出力へ同じ入力を描き、直接readbackのbyte完全一致と、clear色以外の
画素があることを確認する。

```sh
cargo test --manifest-path spikes/motolii-rn-probe/native-renderer/Cargo.toml \
  renderer_core::chroma_tests::rerun_stage_shapes_are_identical_across_two_output_targets \
  -- --exact
```

これはRerun由来のVism visualizerをPreview／Exportの共通評価へ挟める技術的な足場だけを確認する。
製品の正準評価`build_document_frame_graph`→`render_graph_cached`、Document投影、Quality差、動画、
Skia overlayまでのPreview／Export同一性を置換・証明しない。
