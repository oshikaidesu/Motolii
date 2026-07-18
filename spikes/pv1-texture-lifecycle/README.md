# pv1-texture-lifecycle (PV-1)

Slint 1.17 + wgpu 29 上で、`GpuCtx::new_for_ui` の device を `WGPUConfiguration::Manual` で Slint と共有し、**保持 RGBA8 texture** を `Image::try_from` で単一 window に継続表示したときの lifecycle を観測する隔離 spike。

- UI thread は render しない（worker が保持 `TextureView` へ GPU render-pass Clear）
- resize / 明示 regenerate 時のみ `create_texture`
- Worker→UI 配送は `LatestSlot`（Texture / Status 分離）の最新値 mailbox
- 製品 `crates/`・Document・公開 API には触れない

**状態: 自動振る舞い試験 landed / 人間実機審判 pending**

## 実行

```bash
export CARGO_TARGET_DIR=/private/tmp/motolii-pv1-target
cd spikes/pv1-texture-lifecycle
cargo test
cargo build --release
cargo run --release
# 任意: PV1_EVIDENCE_DIR=../../docs/spikes/pv1-texture-lifecycle-evidence cargo run --release
# （証跡 JSON は Shutdown→worker join 後に 1 回だけ書込）
```

## H3 / H4（単一 window 自動往復）

1. **Hide**: ボタンで `window.hide()` → 約 400ms 後に自動 `show()` + lifecycle `Show`（手動 Show 不要）
2. **Minimize**: ボタンで `set_minimized(true)` → 約 400ms 後に自動 `set_minimized(false)` + lifecycle `Restore`

Hide 中も `run_event_loop_until_quit()` で event loop を維持し、window の close 操作で明示的に終了する。Show / Restore 手動ボタンも残すが、審判手順の正は上記自動往復。

## 自動層 (CI / ヘッドレス)

| 検査 | 内容 |
|---|---|
| `cargo test` | 状態機械・resource counter・generation bind・resize/hide 非再生成・mailbox poison 負例 |
| `cargo build --release` | release ビルド可能 |

GPU 無し環境では `motolii_testkit::unavailable_dep` により該当テストは **skip**（`MOTOLII_REQUIRE_GPU=1` では silent skip 禁止）。

## 人間層 (開発主機・release)

`docs/spikes/pv1-texture-lifecycle.md` の H1–H6 を実測し、証跡 JSON を手更新する。未実測は **pending** のまま。overall を自動で pass にしない。

| ID | 内容 |
|---|---|
| H1 | 単一 window 継続表示 ≥10分 |
| H2 | resize 100 回（OS 物理 resize 含む） |
| H3 | hide → 自動 show 往復 |
| H4 | minimize → 自動 restore 往復 |
| H5 | 明示 texture 再生成後の復帰 |
| H6 | DPI/monitor 移動（可能な環境のみ） |

Backend: Metal 実測 / DX12・Vulkan は未実測なら **pending**。

## レビュー用 rg

禁止 API の走査コマンドは `docs/spikes/pv1-texture-lifecycle.md` を参照（spike README には禁止語を載せない）。

## 参照

- 先行: `spikes/s1-slint`（Manual 共有 + `try_from`）
- 仕様メモ: `docs/spikes/pv1-texture-lifecycle.md`
