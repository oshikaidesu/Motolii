# iced エコシステム採掘(第1弾+第2弾)

日付: 2026-08-19
状態: **観察**(決定を含まない。採否は supervisor / 利用者)
経緯: UIトンマナ統一 campaign([同日文書](2026-08-19-ui-tone-unification-campaign.md))の機構材料として
利用者指示「icedのエコシステムも検索」「COSMICというOS採用があるくらいだから色んなソフトから深掘れるはず」を受けた2段調査。

## 第1弾(機構の表層)

- **正攻法は iced 本体**: `iced_core::theme::Base` trait + widget ごとの `Catalog` trait(MIT)。
  アプリ全体の style 一括差し替えの口は本体にある。
- **cosmic-theme(MPL-2.0)の4層導出**: `CosmicPalette`(プリミティブ)→ `Container` → `Component`
  (base/hover/pressed/disabled)をコードで導出。DTCG token → iced Style 変換層の設計手本。
- iced_aw(MIT): tab_bar / context_menu は使える。split と tooltip は **iced core 標準**
  (`pane_grid` / `tooltip`)。number_input にドラッグスクラブは無い(Motolii 自前実装を維持)。
- Sniffnet(Apache-2.0): 6 role 最小 palette から状態色を自動導出する構造。全面等幅フォントの極端例。
- **Halloy は GPLv3 — コード参照不可**。「TOML テーマ1枚で全 widget 差し替え」という UX 発想のみ借用可。

## 第2弾(COSMIC 採掘+横断)

### COSMIC / libcosmic

- widget カタログ(`pop-os/libcosmic src/widget/`): segmented_button, context_drawer, header_bar,
  nav_bar, settings, spin_button, table, toaster, color_picker, dnd_destination/source ほか多数。
- **ただし upstream iced では動かない**: libcosmic は `pop-os/iced` フォークを submodule で抱え、
  widget は `cosmic::Theme`/`cosmic::Renderer` 依存。**単体切り出しは構造模写以外は不可**。
- **状態色導出の実数値**(`cosmic-theme src/model/derivation.rs`, `src/util.rs`、MPL-2.0=コード流用可):
  - 合成は Porter-Duff over(straight alpha)
  - hover = base に neutral α**0.1** を over / pressed・selected = α**0.2** / disabled = 本体 α**0.5**
  - divider = on 色 α0.2(高コントラスト時 0.5)
  - dark 系は base に `neutral_10` α**0.08** の下地を先に over(light 系は 0.75/0.9/1.0 の非対称)
  - 色付き widget(destructive/warning/accent)は α 0.05/0.1/0.2 刻み、focus は accent 固定
- cosmic-files 等のアプリ本体は **GPL-3.0**(構造模写のみ)。高密度リストは
  `settings::item::builder(label).control(...)` の「ラベル+コントロール」1行ビルダーパターン。

### DAW / 映像系

- **iced_audio**(MIT、活発): knob / h_slider / v_slider / ramp / xy_pad / mod_range_input。
  upstream `iced_core 0.14` 依存 — Motolii の fork(`oshikaidesu/iced` 0.15.0-dev)との差分検証が前提。
- OctaSine(**AGPL-3.0** — 流用不可、旧世代 API)。widget ごとに style ファイルを分割する構成の参考のみ。
- **iced_video_player**(MIT OR Apache-2.0): GStreamer デコード → `iced_wgpu::primitive::Pipeline` の
  カスタム primitive で YUV(NV12)を wgpu テクスチャへ直接アップロード、WGSL で YUV→RGB。
  CPU の image widget 経由を避ける設計。preview 再生の GPU パス化の参照実装。

### 横断

- **Liana**(wizardsardine/liana、**BSD-3-Clause** — 最大の発見): `liana-ui` という専用 theme crate が
  `src/theme/` 配下で widget ごとにファイル分割(button/card/pill/notification/…)+ `styles!` マクロで
  palette→Style のボイラープレートを畳む。**許諾ライセンスで直接流用可能な唯一の per-widget theme crate 実例**。
  Liana も iced を独自 fork に patch している。
- Neothesia / XMODITS / Furtherance はいずれも GPL-3.0(参考のみ)。
- **傾向**: 密度の高い UI を作る iced 製品は大抵 iced 自体を fork する — Motolii の現路線(fork 使用)と整合。

## ライセンス早見

| 出所 | ライセンス | 扱い |
|---|---|---|
| iced 本体 / iced_aw / iced_audio | MIT | コード流用可 |
| iced_video_player | MIT OR Apache-2.0 | コード流用可 |
| cosmic-theme / libcosmic | MPL-2.0 | 流用可(ファイル単位開示)。libcosmic widget は fork 依存で実質構造のみ |
| Liana / liana-ui | BSD-3-Clause | コード流用可 |
| Sniffnet | Apache-2.0 | 流用可(表示義務) |
| Halloy / cosmic アプリ / Neothesia / XMODITS / Furtherance | GPL-3.0 | **コード参照不可**、構造・発想のみ |
| OctaSine | AGPL-3.0 | **コード参照不可** |

## 次波への含意(提案。決定ではない)

1. hover/pressed/disabled の状態色を cosmic-theme の α ladder(0.1/0.2/0.5、dark 下地 0.08)で
   `theme::Tokens` から機械導出するヘルパを足す — 手触り裁定([ui-hand-feel-direction] 無反応ゼロ・
   hover/press 即応)の実装コストを widget ごとの発明から定数1組へ畳める
2. `theme/` の将来形は liana-ui 型(widget ごとの style ファイル+マクロ)を手本にする
3. iced_video_player の primitive パイプラインは preview 再生の GPU 化検討時の READ SET
