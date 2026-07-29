# M5 Render Contribution境界比較 v2

作成日: 2026-07-29

状態: **停止／P2D-RCA2差分不採用**（Grok有効判定なし）

変更許可: 本fileのみ

単一動詞: **比較する**

## 入力

- [Render Contribution証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md) §2〜§3、§5
- M5、semantic seat、Controlled Microkernelの元authorityは親task §2の固定anchorからだけ辿る
- 旧`P2D-RCA`差分、Rerun／engine capsule、本file以外の探索は入力にしない

## 比較質問

親task §5の6論点を、次の固定列で比較する。

`論点 / Host既存責任 / 表現側要求 / admission前後 / failure / 追加進化 / 負例 / 未決`

公開trait、Rust名、phase enum、Document、plugin契約、wire、First Vismの具体表現は決めない。
ここでの出力は最終契約でも実装解禁でもなく、後続`P2D-RCI`が元authorityへ再照合する比較表である。

## 出力

P2D-RCA2がこの節だけを置換し、次を残す。

- 親task §5の6論点を一対一で覆う比較表
- opaque／cutout／soft alpha／scene-color-refractionの能力、順序、alpha保証、fallback、診断の比較
- unknown contributionと第二の未知表現に対する負例
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

- 親taskの固定語彙外に規範語を足す必要がある。
- 公開境界、永続意味、First Vismの具体表現を決めないと比較不能に見える。
- 現行コードに無い能力を成立済みとして扱う必要がある。
- 本file以外の変更、network、repo archaeology、旧RCA差分が必要になる。
