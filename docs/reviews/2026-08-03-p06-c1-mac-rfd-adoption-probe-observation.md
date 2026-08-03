# P06-C1-MAC rfd採択probe観察

状態: **DONE / FIXED_MAC_GATE_PASS**。これは製品import接続ではなく、固定Macのnative file dialog境界を既知実装で採択できることを確認した隔離probeである。P06-C1全体、Linux portal、製品UI統合、P06-C2は未完了である。

## 利用者成果と現行gap

- 成果は、Macの親window付きfile dialogで素材を選択またはCancelし、typed failureを既存read-only media probeへ渡せること。
- 現行`crates/motolii-media/src/probe.rs`には`probe_container`とtyped `MediaError`があるが、`motolii-ui`にはfile-dialog依存とparent-window接続がない。
- このprobeはDocument、Undo、journal、project persistence、公開API、React state、共有writerを変更しない。

## 主担当preflight

| 項目 | 裁定 |
|---|---|
| `MECHANISM CLASS` | native file dialog + parent/event-loop/selection/cancel + typed media probe |
| `KNOWN IMPLEMENTATION SEARCH` | P06決定・依存gate・現行media code・`rfd` 0.17.2 source/APIを照合 |
| `CANDIDATES` | `rfd::FileDialog` / `AsyncFileDialog`、`winit` 0.30.13、`motolii_media::probe_container` / `MediaError` |
| `ADOPTION` | `ADOPT` `rfd` disposable fixed-Mac probe + `REUSE` `motolii-media` |
| `REJECTED` | custom dialog、filesystem watcher、UI-owned path state |
| `THIN MOTOLII SEAM` | actual `winit::Window` parent → selected `PathBuf` → existing `probe_container` |
| `RETIREMENT` | probe、app bundle、generated media、raw local evidenceは`target/`・`/private/tmp`ごと破棄可能 |
| `BUILD JUSTIFICATION` | `NONE`。custom mechanismは`FORBIDDEN` |

## 発注と隔離

- 実装担当はexact model `gpt-5.3-codex-spark`。観測ハーネスは生`stream-json`、heartbeat、終了statusを保存し、exit 0、約89秒、timeoutなしだった。
- probeは`target/p06-rfd-mac-probe`の独立workspaceだけに作成し、製品Cargo、runtime、docsを実装担当に変更させなかった。実装前後のtracked HEADは`f02786ab1e1dcaf2c270eb3277aa1ec09e29192c`、tracked statusはcleanだった。
- `cargo check --manifest-path target/p06-rfd-mac-probe/Cargo.toml --offline`と`cargo build --manifest-path target/p06-rfd-mac-probe/Cargo.toml --offline`はPASS。オンラインcheckのDNS失敗はcrate cacheを用いたoffline再実行で原因分離した。
- probe source SHA-256は`bbf2a2b4f3b816a5f81dd53d34504714a2523a21d56fca648dfcba77662dcbf7`、manifestは`172a8609e384b89cecf620a94ee0ea20bf6a998861aaf73329d2a0eefe016454`。

## 実Mac外部gate

- 実行環境はmacOS 15.5 build 24F74、arm64、`aarch64-apple-darwin`、rustc 1.96.1。
- `FileDialog::set_parent(window).pick_file()`をwinit event-loopのkeyboard callbackから同期実行した。Computer UseのAccessibility treeは同一app `com.motolii.probe.p06-rfd-mac`内に`0 sheet Description: open, ID: open-panel`、`CancelButton`、`OKButton`を返し、親window付きnative sheetを確認した。
- Cancel後は`P06_CANCEL`。生成した1秒・64x64のMP4選択後は`P06_PROBE_OK video_streams=1 audio_streams=0`。22-byteの壊れた`.mp4`選択後は`P06_PROBE_ERR kind=Probe`となり、panicやstring-only flatteningではなく既存`MediaError::Probe`へ到達した。
- window close後は`P06_EXIT`、app processはexit 0。tracked file変更は0だった。

```text
P06_WINDOW_READY
P06_DIALOG_OPEN
P06_CANCEL
P06_DIALOG_OPEN
P06_SELECTED "/private/tmp/p06-rfd-valid-20260803.mp4"
P06_PROBE_OK video_streams=1 audio_streams=0
P06_DIALOG_OPEN
P06_SELECTED "/private/tmp/p06-rfd-corrupt-20260803.mp4"
P06_PROBE_ERR kind=Probe ...
P06_EXIT
```

## 独立検収

- Cursor Grok 4.5 High（exact model `cursor-grok-4.5-high`）をread-only検収者とした。初回はAccessibility receiptがreview packetに無く、親sheetを独立確認できないため`REJECT / P1=1`だった。
- AX receiptを隔離evidenceへ追加した再検収は`VERDICT: ACCEPT / P0=0 / P1=0 / P2=1 / SCOPE: PASS`。P2はdeprecated `EventLoop::create_window` / `EventLoop::run`とunused `Result`で、使い捨てprobe debtとして固定Mac採択gateを妨げないと裁定された。
- deprecated winit APIを製品接続へ転記しない。P06-C2を起票する場合は現行`ProductApp`の`ApplicationHandler` / `run_app`境界へ接続し、probe event loopを再利用しない。

## 結論と次の一手

1. 固定Macでは`rfd` 0.17.2を親window付きnative file selection seamとして採択可能とする。
2. これは製品依存追加の自動許可ではない。P06-C2は別の一契約として、video-only admissionと既存`probe_container`への接続targetを再確認してから起票する。
3. Linux portal、soundtrack、Import UI、placement、Document/Undo/persistenceは本gateで証明していない。P06-C1全体を`DONE`へ上げない。

## 非目標

- soundtrack、Import UI、Project Save、Export、Document schema、Undo、独自dialog、watcher、pathの永続化。
- `CU-201P`のTimeline gesture owner/codec/terminal targetをこのprobeで埋めること。
