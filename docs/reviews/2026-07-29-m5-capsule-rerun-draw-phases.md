# Rerun draw phases証拠capsule

状態: **観察** / `FROZEN / DELETE-LATER` / 製品import禁止

- source: Rerun commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`
- files: `crates/viewer/re_renderer/src/draw_phases/draw_phase_manager.rs`,
  `draw_phases/mod.rs`
- license: `MIT OR Apache-2.0`
- 削除条件: P2D-RCIで元sourceへの直接引用へ置換後

## 観察

- draw dataがphase別drawableを収集し、managerが全draw dataを保持してphase単位にsort／dispatchする。
- opaque系はrenderer／draw-data grouping後near-to-far、transparent系はdistance far-to-nearでsortする。
- transparentはdepth read／no write、opaqueはdepth read／writeと説明される。
- phase集合は固定enumで、source自身がphase abstractionを進行中と注記する。

## 非証明範囲

固定phase enum、sort key、renderer key packing、ViewBuilder責任をMotoliiへ採用する根拠にしない。
soft alpha交差の正解、OIT、scene-color lifetime、Motolii resource budgetを証明しない。
