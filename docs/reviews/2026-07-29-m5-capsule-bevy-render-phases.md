# Bevy render phase証拠capsule

状態: **観察** / `FROZEN / DELETE-LATER` / 製品import禁止

- source: Bevy `0.19.0`公式API docs、取得日2026-07-29
- URLs: <https://docs.rs/bevy/latest/bevy/render/render_phase/index.html>,
  <https://docs.rs/bevy/latest/bevy/render/render_phase/struct.SortedRenderPhase.html>,
  <https://docs.rs/bevy/latest/bevy/core_pipeline/core_3d/>
- license: API docs記載のcrate licenseに従う。本文転載なし
- 削除条件: P2D-RCIで公式URLへの直接引用へ置換後

## 観察

- render phaseはqueue、prepare、sort、drawを分けるmodular abstractionと説明される。
- opaque／alpha-maskはbinned、transparentはback-to-frontを要するsorted phaseとして分かれる。
- phase分離理由にはsorting／batching差と、前phaseのrendered textureを読むscreen-space effectが挙がる。
- core pipelineにはprepassとOIT moduleが別責任として存在する。

## 非証明範囲

Bevyの型、schedule、render graph、OIT方式、phase名をMotolii契約へ転記する根拠にしない。
