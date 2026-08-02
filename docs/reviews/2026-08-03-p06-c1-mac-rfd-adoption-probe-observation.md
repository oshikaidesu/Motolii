# P06-C1-MAC rfd採択probe観察

状態: **ACTIVE / EXTERNAL_GATE_PENDING**。これは製品import接続ではなく、固定Macのnative file dialog境界を既知実装で採択できるかを確認する隔離probeである。

## 利用者成果と現行gap

- 成果は、Macの親window付きfile dialogで素材を選択またはCancelし、typed failureを既存read-only media probeへ渡せること。
- 現行`crates/motolii-media/src/probe.rs`には`probe_container`とtyped `MediaError`があるが、`motolii-ui`にはfile-dialog依存とparent-window接続がない。
- このprobeはDocument、Undo、journal、project persistence、公開API、React state、共有writerを変更しない。

## 主担当preflight

| 項目 | 裁定 |
|---|---|
| `MECHANISM CLASS` | native file dialog + parent/thread/cancel boundary |
| `KNOWN IMPLEMENTATION` | `rfd` 0.17.2 `FileDialog` / `AsyncFileDialog`、既存`motolii-media::probe_container` |
| `ADOPTION ROUTE` | `ADOPT`（API／platform probeのみ。製品依存追加は外部gate後） |
| `THIN MOTOLII RESIDUAL` | file-kind admission、typed missing/corrupt/unsupported mapping、read-only probe |
| `RETIREMENT` | custom dialog、filesystem watcher、UI-owned path persistenceを作らない |
| `BUILD` | `FORBIDDEN` |

## 現在の証拠

- 隔離した最小API probeでhost `cargo check` が成功した。
- 同じprobeは `aarch64-apple-darwin` target compileも成功した。
- 実Macのnative dialogを起動していないため、parent window、実ファイル選択、Cancel、main-thread実行の外部gateは未完了である。
- Computer Useはconstruction中に反復しない。製品routeが実施可能になった最後に、名前付き外部gateとして一回だけ実施する。

## 次の一手と停止線

1. product Cargoやruntime接続を変更せず、外部gateの実施可能条件だけを確認する。
2. 外部gateがPASSした場合のみ、P06-C2をvideo-only配置へ縮小し、既存`probe_container`へ接続する。
3. parent/selection/Cancelまたはtyped failureのどれかが既知targetへ閉じなければ、`EXTERNAL_GATE_PENDING`を維持し、CU-201Pを迂回する新しいdialog実装を作らない。

## 非目標

- soundtrack、Import UI、Project Save、Export、Document schema、Undo、独自dialog、watcher、pathの永続化。
- `CU-201P`のTimeline gesture owner/codec/terminal targetをこのprobeで埋めること。
