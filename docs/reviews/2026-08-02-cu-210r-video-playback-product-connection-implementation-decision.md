# CU-210R video-only playback 製品接続実装決定

## 判定

- `CU-210R`: **DONE（縮小採用）**
- `P07-C1`: mixed `AudioProgram` 接続を含む親の完了ではなく、Rectangle／音声sourceなしの Local Alpha 用 video-only slice として閉じる。
- `GAP-28`: mixed `AudioProgram`、Soundtrack／Clip audio、音声UI、waveform は未完のまま維持する。

## 接続した意味

既存 `motolii-transport::PlaybackSession` と cpal の `PlaybackCounters` を製品windowへ接続した。UI timer や preview 専用clockは作らない。

- `Transport` に seek 起点をデバイス供給レートへ変換して保持させ、非zero seekでもproject時刻へ戻す。`PlaybackSession`はsource起点とsource rateを別に保持し、終端判定もdevice rateへ変換するため44.1 kHz等へ縮退してもrate domainを混ぜない。
- `PlaybackSession` に既存 `OutputStream::pause/play` を委譲し、play stateだけをTransientとして公開する。
- Rectangle fixtureは音声sourceを持たないため、composition尺の48 kHz stereo無音PCMを既存 `PcmCache`／`AudioProducer`へ渡す。これはmixed `AudioProgram`の代替実装ではない。
- Stage React transportの再生buttonはstrict `toggle-play` IPCを発行し、HostがwakeしてProductAppのevent loopへ渡す。
- 再生中のplayheadは`Transport::perceptual_time()`から16 ms単位で最新render requestへ投影する。終端は供給済みframe数またはproject durationで閉じる。

## 許可された変更面

- `crates/motolii-transport/src/lib.rs`
- `crates/motolii-transport/src/playback.rs`
- `crates/motolii-ui/Cargo.toml`
- `crates/motolii-ui/src/product_runtime.rs`
- `crates/motolii-ui/src/product_runtime_adapter.rs`
- `crates/motolii-ui/src/stage_chrome_host_runtime.rs`
- `ui/motolii-web/src/candidates/StageChromeCandidate.jsx`
- `ui/motolii-web/src/host/stage-transport-main.jsx`
- `ui/motolii-web/src/host/stageHostBridge.js`
- 対応する生成済みHost assetとmanifest

runner、AGENTS.md、route契約、Document／journal／plugin公開形式は変更しない。

## 負例と検証

- `Transport` のorigin fixtureで5秒seek後の1秒供給が6秒になることを固定。
- `toggle-play` はunknown fieldを拒否し、既存Easing identity／layout gateを迂回しない。
- pause後約1.2秒の待機でStage timecodeが変化しない。
- source drain後に`action=ended`を発行し、buttonを再生表示へ戻す。
- 通常Mac製品windowで、初期`00:00.0` → 再生 → pause → 終端`00:10.0`、さらにruler seek `00:05.0` → 再生 `00:06.0`を確認。

自動検証は次を通過した。

```text
cargo test -p motolii-transport --quiet
cargo test -p motolii-ui --lib stage_chrome_host_runtime --quiet
cargo check -p motolii-ui --quiet
cargo clippy -p motolii-ui --all-targets -- -D warnings
npm --prefix ui/motolii-web run check:host
node --check generated stage assets
```

Host bundleのVite再生成は環境に`vite`が無いため未実行。既存生成assetを同じentry/hash名のまま更新し、syntaxとasset-manifestを検証した。

## 非目標と次hand-off

- mixed `AudioProgram`／`MixProducer`の製品接続、Soundtrack／Clip audio、continuous scrub、step、audio device選択、waveformは次の独立粒。
- Save／reopen／Exportはこのcommitの完了条件に含めない。
- 次の製品粒は、既存 `ProjectSession`／`ExportJob` の接続票を先に閉じ、共有writer/event loopの責任を一つずつ特定する。
