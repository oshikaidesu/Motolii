# G0-10 detached Preview multi-Surface spike（2026-07-21）

状態: **同一画面・macOS自動fixture合格／異DPI・第二monitor・HDRは未証明**。

公開API、Document、D2、永続window layoutを変更せず、Editorとdetached Previewを別top-level
window / wgpu Surfaceとして動かせるかを[`spikes/g0-10-multi-surface-window/`](../../spikes/g0-10-multi-surface-window/)
で検証した。両Surfaceは1つのwgpu device / queueを共有し、window固有のSurface、config、render pipelineを持つ。
Host stateはwindow mapの外に置き、Preview windowの破棄対象へ含めない。

## 自動fixture

`--auto`は同一画面で次を順に実行する。

1. Editorとdetached Previewを生成し、両Surfaceでpresentする
2. Previewだけへ疑似surface lostを1回注入し、Previewだけを再configureする
3. Previewをfullscreenへ移し、resize event観測後に元へ戻す
4. Preview windowを閉じ、Editorがその後もpresentしたことを確認する
5. Previewを再生成し、同じHost snapshotを表示対象として維持する
6. Editorへfocusを戻し、最終reportを書いて終了する

疑似surface lostはdriver障害の再現ではない。`get_current_texture`の`Lost / Outdated`と同じ
再configure分岐を、対象Surfaceだけに決定的に通すfault injectionである。実driver、device lost、異GPU adapterは
このfixtureの合格範囲に含めない。

## 2026-07-21 macOS実測

環境はmacOS 15.5、Apple M4、内蔵2560×1664 Retina、scale factor 2.0。追加monitorは接続していない。
保存したreportは[`g0-10-multi-surface-window-evidence/report.json`](g0-10-multi-surface-window-evidence/report.json)。

- 2 top-level window / 2 Surface、共有device 1個
- Preview疑似lost 1回、Preview再configure 1回、Editor再configure 0回
- Preview close 1回、reopen 1回、window generation 2
- close後もEditor present継続、Host stable ID・選択・Shape数は不変
- fullscreen enter / exitを各1回、focus gained / lostを両windowで観測
- Editor layout epoch 4、Preview layout epoch 12、最終scale factorはいずれも2.0
- 最終状態はEditor / Previewともopen、`status=complete`

## 合格範囲と停止線

成立したのは、同一画面におけるtop-levelごとのSurface所有、片側だけの再configure、Preview close/reopenと
Host transient snapshot保持、fullscreen/focus/scale/layout計装である。window位置、DPI、fullscreen状態、focusは
Document・公開plugin契約へ流していない。

異DPI monitor間移動、第二monitor、HDR/SDR差、実surface/device lost、Windows WebView2は別実機審判のまま残す。
このspikeの`HostSnapshot`、window role、report JSON、fault injection型を製品公開API、Document、D2、永続形式へ
昇格しない。製品統合はM3入場と既存UI境界規律に従う。

## 再現コマンド

```bash
cargo test --manifest-path spikes/g0-10-multi-surface-window/Cargo.toml
cargo run --manifest-path spikes/g0-10-multi-surface-window/Cargo.toml -- --auto
jq . /tmp/motolii-g0-10-multi-surface-window-report.json
```
