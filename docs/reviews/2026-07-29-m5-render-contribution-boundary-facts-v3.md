# M5 Render Contribution境界比較 v3

作成日: 2026-07-29

状態: **登録済み／P2D-RCA3未実行**

変更許可: 本fileの`転記欄`だけ

単一動詞: **比較する**

## 入力

- [Render Contribution証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md) §2〜§3、§5
- 親task §2が固定するM5、semantic seat、Controlled Microkernelの元authority

旧RCA/RCA2差分、Rerun／engine capsule、network、repo archaeologyは入力にしない。

## 固定比較軸

各行は`事実 / 比較上の含意 / 未決`を分ける。`事実`は入力authorityの既決または現行code factだけ、
`比較上の含意`は既決を変更しない整理だけ、`未決`は後続`P2D-RCI`へ戻す事項だけを書く。

| ID | 論点 | Host既存責任 | 表現側の型付き要求 | admission前後 | failure | 追加進化 | 負例 | 未決 |
|---|---|---|---|---|---|---|---|---|---|
| Q1 | 要求とcontributionの分離 | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> |
| Q2 | contributionが所有しないHost状態 | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> |
| Q3 | opaque/cutout/soft alpha/scene-color-refraction | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> |
| Q4 | 未知能力と追加的進化 | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> |
| Q5 | First Vismのconformance fixture役割 | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> |
| Q6 | 第二の未知表現 | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> | <!-- 転記欄 --> |

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

- 固定比較軸、入力authority、固定語彙の追加または変更が必要になる。
- 公開境界、永続意味、First Vismの具体表現を決めないと比較不能に見える。
- 現行コードに無い能力を成立済みとして扱う必要がある。
- 本fileの`転記欄`以外の変更が必要になる。
