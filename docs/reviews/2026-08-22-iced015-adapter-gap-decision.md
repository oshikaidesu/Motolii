# 裁定170 — iced 0.15 移行の採択と adapter gap の解法(rerun fork 案B・iced fork 無傷)

日付: 2026-08-22(深夜継続) / 状態: **決定** / 起点: 利用者裁定「29対応を今やる」+ ω 調査(`2026-08-22-iced-015-wgpu29-migration-survey.md`)

## 1. 採択

- **iced を fork `motolii/host-seams`(`73e686ee`、= upstream master + seam1/seam2)へ pin し、wgpu を 29.0.4 単一へ統一する**。ω の解決実験で wgpu 系 14 パッケージが単一 29.0.4 へ畳まれることは実測済み。API 差分は我々の使用面 338 箇所で実質ゼロ(ω §2)
- 段階は ω §5 の M0〜M4 を採用。**M0+M1 は1レーンに束ねる**(pin 交換はビルドしないと合否が出ない — 「ビルド不可レーン」を挟む意味が無い)
- `iced_test` は本体と同 rev に同梱(2ライン共存事故の防止、ω §2.4)

## 2. adapter gap(EVIDENCE_GAP-1)の裁定 = 案B強化版

**iced fork には一切足さない。rerun fork へ device 駆動の第二コンストラクタを1本足す**(BL1b・裁定161 と同型の最小口):

- 実測根拠: `RenderContext::new` が adapter に求めるのは `DeviceCaps::from_adapter`(features/limits/downlevel)と `adapter.get_info()`(ログ)のみ。**wgpu 29.0.4 の `Device` は `features()`/`limits()`/`adapter_info()` を公開している**(`wgpu-29.0.4/src/api/device.rs:105-120` 実測)— adapter の実物なしで大半が導出できる
- 唯一 device から取れないのは `get_downlevel_capabilities()`。ここは `DeviceCaps::from_device(device)` を fork に追加し、**tier 判定を `device.adapter_info().backend` で分岐**する: `Backend::Gl` のみ保守的に `Limited`、native backend(Metal/Vulkan/DX12)は WebGPU min-spec 充足として `FullWebGpuSupport`(downlevel flags の欠けは実質 GL 系のみ、という re_renderer 自身の doc 注記と整合)。limits 由来の `max_texture_dimension2d`/`max_buffer_size` は `device.limits()` から — **共有 device では device の実効 limits が正**(adapter の理論値より正しい)
- fork 追加: `DeviceCaps::from_device` + `RenderContext::new_from_device(device, queue, output_format_color, config_provider)`(既存 `new()` の adapter 依存2呼び出しを置換しただけの姉妹関数)。既存呼び手は無改変・バイト一致
- 案A(iced の `Pipeline::new` trait 改変)は棄却: 上流公開 trait のシグネチャ差分は rebase のたびに複利で効く。seam 台帳の思想(「fork に足すのは seam のみ、概念は足さない」)にも反する

## 3. レーン割り(cargo 同時4本の新上限下)

| レーン | write-set | 中身 |
|---|---|---|
| **M01** | `next/Cargo.toml`・`next/shell/motolii-shell/Cargo.toml`・`Cargo.lock`+(必要時)PNG oracle 閾値 | pin 交換→full workspace 緑。font スタック更新で PNG oracle が赤くなったら**意匠不変を目視確認の上で許容誤差のみ再較正**(ε 前例)。instrument は文字を描かないため無風の可能性大(未測定) |
| **M2** | `next/engine/motolii-compositor/src/lib.rs` のみ | `Compositor::with_device`(不活性・配線ゼロ)。M01 と並走可(write-set 互いに素・lockfile 不変) |
| **M3** | rerun fork(`device_caps.rs`/`context.rs`)+ rev pin bump + compositor glue | 本裁定 §2 の実装+常設 oracle(wgpu 直建て device で `new_from_device` の pipeline 成立を headless 審判)。**M01 merge 後**(lockfile 衝突回避) |
| **M4** | Stage presenter | readback 撤去・`Primitive::render` で直接 blit。実窓 fps 実測+絵の同一性。最終審判=利用者実窓 |

## 4. 既知のリスク(ω から継承)

- font スタック大幅更新(cosmic-text 0.15→0.19 等)の実窓の文字の見え方変化 — **利用者の目でしか判定できない**(M01 着地時の窓差し替えで確認を依頼)
- 0.15-dev は動く線 — rev pin+seam 台帳(rerun と同運用)。iced-rs/winit・cryoglyph の2 git fork が依存グラフに加わる(ω §2.4 item 5)
- `frames()`/shader trait の実挙動は M3/M4 で初めて実使用検証(EVIDENCE_GAP-4 はそこで閉じる)
