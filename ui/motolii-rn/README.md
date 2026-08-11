# Motolii RN product source

通常作業の入口はこのdirectoryだけ。macOS appはrepository rootから
`scripts/build-macos-app.sh`でbuildする。旧worktree、DerivedData、リポジトリ外artifact、
過去appへfallbackしない。current buildが無ければ旧appを開かず、未buildと報告する。

Motoliiの唯一のRN製品source。`App.tsx` 660行 — Browser 3タブ(`MEDIA`/`EFFECTS`/`CREATE`)、
Inspector/Extensions、Timeline 3モード、effect一覧、panel registry、
native `MotoliiGpuComponentView.mm`、Fabric spec `MotoliiGpuView`/`MotoliiTimelineView`。

## UI version

UIの版番号の正本は`package.json`の`version`だけである。起動した画面のtitle barにも
`UI <version>`として同じ値を表示する。利用者に見えるUIを変更するコミットでは、同じ
commitでこのversionを更新し、確認時はsourceの`package.json`と起動画面の表示が一致することを
先に確かめる。native rendererのbuild identityは別の確認対象であり、UI versionから推測しない。

旧名`MotoliiRnProbe`の資産を `~/Documents/Codex/2026-08-06/ui-rust-ui-c-react/work/MotoliiRnProbe/` から
[2026-08-10回収監査](../../docs/reviews/2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)により
回収し、2026-08-11にこのpathと`MotoliiRn` app identityへcut overした
(node_modules/Pods/build/target/vendor/.yarn除外。`yarn install`で復元可)。RN標準のREADMEは`README.upstream.md`へ退避。

**状態境界**: このdirectoryがRN／Rerun／Skiaを束ねる製品source兼write targetである。
別RN appや移植先、過去版を保持する新しい凍結copyを作らない。旧R0 UI sourceはmainから削除済みで、
必要な歴史証拠はGit履歴からだけ読む。
固定fixtureは製品意味の正本にせず、Document／D2入力へ接続できた箇所から置き換える。

## Rerun Stage共通評価probe

`native-renderer/src/renderer_core.rs`の`encode_rerun_stage_shapes`は、B001のRect／Circleを
一つのRerun評価関数から既存wgpu textureへencodeする。実StageとGPU testは同じ関数を使う。
testは同じdevice／queue上の2出力へ同じ入力を描き、直接readbackのbyte完全一致と、clear色以外の
画素があることを確認する。

```sh
cargo test --manifest-path ui/motolii-rn/native-renderer/Cargo.toml \
  renderer_core::chroma_tests::rerun_stage_shapes_are_identical_across_two_output_targets \
  -- --exact
```

これはRerun由来のVism visualizerをPreview／Exportの共通評価へ挟める技術的な足場だけを確認する。
製品の正準評価`build_document_frame_graph`→`render_graph_cached`、Document投影、Quality差、動画、
Skia overlayまでのPreview／Export同一性を置換・証明しない。
