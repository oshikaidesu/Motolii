# M5 Render Phase fixture証拠比較 v2

作成日: 2026-07-29

状態: **停止／P2D-RCC2差分不採用**（Grok `REJECT`、P0=0/P1=5/P2=2）

変更許可: 本fileのみ

単一動詞: **比較する**

## 入力

- [Render Contribution証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md) §2〜§3、§7
- [Bevy render phase capsule](2026-07-29-m5-capsule-bevy-render-phases.md)
- [Godot transparency capsule](2026-07-29-m5-capsule-godot-transparency.md)
- [Unreal translucency capsule](2026-07-29-m5-capsule-unreal-translucency.md)

取得済みcapsuleだけを読み、network、公式page再取得、二次資料、engine source探索を行わない。

## 比較質問

親task §7の固定6 fixture候補ごとに、三familyの観察が
`支持する制約 / 露出するfailure / 非証明 / Motoliiで未決`
のどこへ対応するか比較する。

engineのrender graph、material、scene、camera、queue／phase enum、threshold、copy方式を
Motoliiの要件または公開語彙へ転記しない。

## 出力

P2D-RCC2がこの節だけを置換し、次を残す。

- 三family × 親task §7の6 fixture候補の比較matrix
- soft alpha交差とdepth bufferだけでは閉じない順序問題の適用範囲
- scene-color／refractionの入力snapshot、範囲、順序、failureに関する証拠coverage
- 各familyの非証明範囲と、fixtureへ焼いてはならない方式／数値
- 事実／推論／未決の分離

<!-- P2D-RC COMMON NON-GOALS BEGIN -->
- 公開API、Document schema、plugin契約、wire形式、Vism/package/schema、実装コード、fixtureコードを変更しない。
- `RenderContribution`等のRust名、trait signature、serde形、registry多重度、phase enumを決定しない。
- P2Dの初期3 policy、P3 Observation、Host authorityを別設計へ置換しない。
- Rerun／ゲームエンジンの型、状態所有、render graph、package名、material／phase enumをMotolii authorityにしない。
- Host enum、具体provider ID、raw JSON／文字列走査、opaque ID／private type走査、公開raw mutation、
  invented serde default、重複planner/helper、lint抑制、期待値／golden変更で境界を迂回しない。
- leaf単独で採用決定、実装解禁、P2D完了を宣言しない。
<!-- P2D-RC COMMON NON-GOALS END -->

## STOP

- capsule外の資料、別version、数値threshold、具体方式が必要になる。
- engine機能をMotoliiの新要件にする、またはP2D既決要件を削る必要がある。
- 公開API、Document、plugin契約、phase enum、fixture期待値を決める必要がある。
- 本file以外の変更、network、repo archaeologyが必要になる。
