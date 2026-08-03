# CU-204A diagnostic projection adapter 実装決定

- 日付: 2026-07-31
- 状態: **決定 / DONE**
- commit: 本文と同一commit

## 1. 結論

`DiagnosticEnvelope`をBrief / Context / Inspect / Assistiveへ変換する純粋adapterを
`motolii-ui`のprivate moduleとして実装した。

全密度はreason、action、subjects、facts、recoverability、recovery candidatesを
同じ順序で保持する。密度差は表示copyの量だけであり、Briefは結果+原因、Contextは
回復説明、Inspectはtyped詳細、Assistiveは同じ情報の完全文を追加する。

## 2. 境界

- `crates/motolii-ui/src/diagnostic_projection.rs`
  - `DiagnosticDensity`
  - `DiagnosticProjection`
  - `project_diagnostic`
- moduleはcrate rootから公開re-exportしない
- `DiagnosticEnvelope`へserde、表示文、constructorを追加しない
- errorの`Display` / `Debug`文字列を解析しない
- Document、writer、queue、render、toolkit、callbackを受け取らない
- 現行5 envelopeのcandidateは全件空のまま。actionを作らない

`Inspect`のdetailsはsubjectsを入力順に並べ、その後factsを入力順に並べる。
同じ数値や表示名からidentityを逆算しない。

## 3. 証跡

- `cargo test -p motolii-ui diagnostic_projection`: 3 pass
- `cargo test -p motolii-ui --test diagnostic`: 5 pass
- `cargo clippy -p motolii-ui --all-targets -- -D warnings`: pass
- `cargo fmt --all -- --check`: pass

試験は5 reason × 4 densityのtyped identity、表示量の包含関係、definition/use順序、
candidate空を固定する。source guardはserde/toolkit/Document writer/render worker/
function callback型の混入を拒否する。

## 4. 後続

親CU-204は`SPLIT`、CU-204S/Aは`DONE`。CU-204Pは、現行通常操作から到達可能な
diagnostic sourceとlifetime/clear規則が閉じるまで`WAIT`を維持する。
diagnostic-only routeやunknown command注入を通常製品証拠にしない。
