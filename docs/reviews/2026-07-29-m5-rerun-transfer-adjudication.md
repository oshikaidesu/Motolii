# M5 Render Contribution Rerun転移裁定

作成日: 2026-07-29

状態: **決定**（証拠assetの転移分類だけ。Render Contribution契約の採否ではない）

対象: Rerun commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`の固定6 asset

本書は主担当Codexが
[Render Contribution証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md) §6の
固定監査対象を取得し、Motolii仕様、現行コード事実、Rerun先例、Motolii fixtureの順で
`DEPEND / VENDOR / PORT / PATTERN / REJECT`を裁定した記録である。Rerunの型、phase名、
実装、依存を製品へ持ち込む許可ではなく、後続のRerun比較leafは本書の分類を変更しない。

## MOTOLII AUTHORITY

- [M5仕様](../specs/M5-3d-and-post.md)の「方針」、
  「Camera Provider／Observationと空間rendererの分界」、task `P2/P3/P2D`、実装ガード13/15。
- [換装可能な意味の席／Provider決定](2026-07-24-replaceable-semantic-seat-decision.md)の
  「2.3 Hostが所有するもの」「2.4 Providerが所有するもの」「2.5 Provider換装」「6. 停止線」。
- [Controlled Microkernel決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)の
  「3. Coreに残す最小責任」「5. 権限と多重度」「6. pluginという語と信頼境界の分離」。
- 親task §2〜§3の共通Host境界、固定語彙、code fact。Rerunはこれらより下位の先例である。

完成条件は、既存P2Dを置換せず、未知の空間表現をHost enumやfirst-party専用口なしで比較でき、
2D pixel不変、同一world/camera、型付き拒否、Preview/Export同一をfixture候補へ残すことである。

## CODE FACT GAP

親task §3の固定hashで、次を確認済みとする。

- `LayerSourcePlugin::render`は具体`CompCamera`とRGBA outputを持つ0-input sourceであり、
  provider非依存Observation要求、複数draw phase、depth／transparent／refraction contributionは無い。
- `build_source`はprepared LayerSourceを0-input RGBA nodeへloweringし、shared depthへのadmissionは無い。
- `dispatch_plugin`は固定kind分岐でLayerSourceへcameraとRGBA outputを渡し、capability交渉と
  phase resolveは無い。

これはRerunとの差ではなく、Motolii現行コードで未成立の事実である。hashが変わった場合は
leafが再解釈せず主担当Codexへ戻す。

## RERUN EVIDENCE

固定commit: `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`

license: repositoryの`LICENSE-APACHE`／`LICENSE-MIT`とpackage metadataで
`MIT OR Apache-2.0`を確認した。監査済み範囲は次のfile/APIと対応capsuleだけである。

| asset | 監査対象 | capsule |
|---|---|---|
| A1 | `examples/rust/custom_visualizer/src/main.rs`の`App::extend_view_class` | [custom visualizer](2026-07-29-m5-capsule-rerun-custom-visualizer.md) |
| A2 | `examples/rust/custom_visualizer/src/height_field_visualizer.rs`のvisualizerとdraw data生成 | [custom visualizer](2026-07-29-m5-capsule-rerun-custom-visualizer.md) |
| A3 | `examples/rust/custom_visualizer/src/height_field_renderer.rs`のrenderer、draw data、phase参加 | [custom visualizer](2026-07-29-m5-capsule-rerun-custom-visualizer.md) |
| A4 | `examples/rust/custom_view/src/main.rs`の`App::add_view_class` | [custom view](2026-07-29-m5-capsule-rerun-custom-view.md) |
| A5 | `crates/viewer/re_renderer/src/draw_phases/draw_phase_manager.rs`の収集、sort、dispatch責任 | [draw phases](2026-07-29-m5-capsule-rerun-draw-phases.md) |
| A6 | `crates/viewer/re_renderer/src/draw_phases/mod.rs`の固定phase語彙 | [draw phases](2026-07-29-m5-capsule-rerun-draw-phases.md) |

全関数、依存closure、LFS snapshot、shader出力、性能、Motolii適合、Rerun全体のView／store／UI責任は
監査しておらず、本書はそれらを証明しない。

## TRANSFER CLASS

| asset | class | 裁定理由 |
|---|---|---|
| A1 | `PATTERN` | 既存Viewへ能力を追加する動線と、新View追加を別責任にする比較先例だけを使う |
| A2 | `PATTERN` | query／transform等からrenderer固有draw dataを生成する責任分離だけを比較する |
| A3 | `PATTERN` | 表現固有resource／pipelineとHost側phase参加を分ける先例だけを比較する |
| A4 | `PATTERN` | 新しいView登録は既存View拡張と別動線である、という負例比較だけを使う |
| A5 | `PATTERN` | 複数contributionをHost側で収集し、ordering／dispatchする責任分離だけを比較する |
| A6 | `REJECT` | 固定phase enum、phase名、sort keyをMotoliiの公開契約または閉じた能力集合へ転移しない |

`PATTERN`は型、trait、関数、source、shader、依存を再利用する意味ではない。A1〜A5にも
`DEPEND / VENDOR / PORT`を許さず、A6の語彙をA5の一部として復活させない。

## TRANSFER LIMIT

- 後続leafの変更許可は事前登録した単一docs fileだけとする。
- RerunのApp、Entity、ViewClass、Blueprint、store、query、draw-phase enum、sort key、serde、
  shader、dependency、UI stateをMotoliiへ持ち込まない。
- Motoliiの公開API、Document、plugin契約、wire、Vism/package/schema、実装、fixtureコードを変更しない。
- 固定6 asset外が必要なら未裁定として`ORDER: STOP`し、inventoryの候補分類で補わない。
- `PATTERN`をRerun構造への類似要求、実装解禁、採用推奨、完成証拠に読み替えない。

## MOTOLII ORACLE

Rerunへの構造／外観類似ではなく、後続`P2D-RCI`がMotolii authorityから分離するfixture候補で判定する。

1. contribution未使用時に既存2D compositionのpixelが不変である。
2. 全objectが同じworld、active camera／Observation、world transformへ従う。
3. opaque／cutout／soft alphaを一つの黙示fallbackへ潰さず、depth参加と非対応診断を区別する。
4. scene-color／refraction要求が入力snapshot、範囲、順序、failureを宣言し、隠れcopyを作らない。
5. unknown capabilityをHost enum、opaque ID、raw JSON走査で推測せず、型付き拒否する。
6. Preview／Exportが同じ評価経路を使い、差は`Quality`だけである。

本書の裁定はfixture期待値を決めない。公開trait、phase enum、Document field、provider identity、
First Vismの具体表現が必要になった時点で、後続leafは決定を発明せず停止する。
