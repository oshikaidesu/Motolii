# Stage の既定カメラを document camera(正対)にした証跡

2026-08-18。`docs/reviews/2026-08-18-first-real-run-observations.md` 欠陥(3)
「Stage が斜め3D視点(グリッド床+暗赤背景に遠近付きの板)」に対する実装の前後。

裁定は `docs/reviews/2026-08-18-rerun-as-composition-foundation.md` —
「編集時の既定表示も document camera(未定義なら正対)とし、現在の斜め固定視点は
view camera の初期値バグとして扱う」。

## 撮り方

同じ project(1920x1080 / 16:9 / clip 1枚 + soundtrack)を同じ口で開いたもの。

```sh
motolii-blitz-shell --project <project.json> --screenshot shell.png
```

## 中身

| file | 何 |
| --- | --- |
| `shell-default-camera-before.png` | 実装前。Rerun の既定 eye。板が斜めに潰れ、グリッド床と暗赤の背景が出ている |
| `shell-document-camera.png` | 実装後。comp 平面へ正対し、画枠の横いっぱいに載る。グリッド床も遠近も消えた |
| `red.txt` | 実装前に取った `cargo test -p motolii-ui --lib rerun_stage::adapter` の落ちる出力 |

## 後の絵の読み方

- pane は 16:9 より僅かに縦長なので、**横合わせ**が効いて上下に細い帯が残る。
  帯は Rerun の背景で、グリッドは出ていない(実装前と見比べる場所はここ)。
- 帯の際にある薄紫の線は layer の輪郭(`FIXTURE_RECT_STROKE_COLOR`)で、
  comp の縁とちょうど重なっている。

導出と審判は `crates/motolii-ui/src/rerun_stage/document_camera.rs`。
