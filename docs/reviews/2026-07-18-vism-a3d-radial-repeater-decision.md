# VSM-A3D — 決定論的 2D Radial Repeater LayerSource 採用決定

作成日: 2026-07-18

状態: **設計決定／コード変更なし。VSM-A3S 仕様化可。VSM-A3 実装は A3S 完了後。**

本書は[VSM-A3R](2026-07-18-vism-a3-external-expression-survey.md)が推薦した第一候補を、**決定論的 2D Radial Repeater** として採用・縮小・延期を確定する。実装許可、公開 API 変更、Document schema 変更、crate 追加は含まない。調査の正本は A3R を改変せず維持し、採用決定の正本は本書である。

関連正本: [Vism実装計画](2026-07-17-vism-implementation-plan.md)、[UI操作言語 §5.4](../ui-interaction-language.md#54-parameter-panelを表現のホームにする)、[プラグイン作者向け規約](../plugin-authoring.md)、[plugin-ui-v1 境界](2026-07-12-plugin-ui-v1-boundary.md)、[A0S Contract Catalog](2026-07-17-vism-a0s-contract-catalog-spec.md)

## 1. 決定

VSM-A3 の v1 表現として、0-input の決定論的 2D Radial Repeater `LayerSource` を採用する。物理 particle、任意 path、multi-input Composite、汎用 Expression runtime、Blender bridge、custom UI は本 ticket では採用しない。

A3R の候補 parameter 名 `size` は本 v1 では `dot_radius`、`speed` は `angular_speed` に対応する。`spread`／`seed` は採用しない。別名併存や schema 予約は行わない。

## 2. 採用 v1 identity

| 欄 | 値 |
|---|---|
| id | `core.layer_source.radial_repeater` |
| version | `1` |
| kind | `LayerSource` |
| texture inputs | `min_inputs = max_inputs = 0` |

### NodeDesc 投影（v1 inspectability 正本の一部）

| 欄 | 値 |
|---|---|
| `display_name` | `Radial Repeater` |
| `category` | `Generate` |
| `tags` | `["radial", "repeater", "generate"]` |

現行 `ParamDef` には label 欄が無い（`id`／`value_type`／`default`／`f64_domain` のみ）。本 ticket で `ParamDef`／`NodeDesc` へ新 field・新 `ValueType`・widget hint schema を発明しない。上記と §4 の parameter 意味表が v1 の検査可能性正本である。

## 3. 正準意味（canonical meaning）

正準空間（単位なし・原点中央・Y-up・高さ = 1.0）で評価する。

- \(N = \texttt{count}\) 個の同一の塗りつぶし円を描く。
- `LayerSourcePlugin::render` の独立した `t: RationalTime` 引数は正準どおり秒
  \(t_\mathrm{sec} = \mathrm{num}/\mathrm{den}\) と解釈する。各 instance の角度:
  \[
  \theta_i(t) = \texttt{phase} + \texttt{angular\_speed} \cdot t_\mathrm{sec} + \frac{2\pi i}{N}, \quad i = 0,\ldots,N-1
  \]
- 各円の中心:
  \[
  \mathrm{center}_i = (\texttt{radius}\cos\theta_i,\ \texttt{radius}\sin\theta_i)
  \]
- 各円の半径は `dot_radius`（正準）。
- `phase` はラジアン。`phase = 0` のとき最初の instance（\(i = 0\)）の中心は **+X 軸上**（\((\texttt{radius}, 0)\)）に置く。
- 正の `angular_speed` は正準 Y-up 空間で **反時計回り（CCW）** に全体位相を進める。
- 出力形状は各円の **union**。重なり領域で alpha を加算しない（coverage は union の解析的意味に従う。数値実装の詳細は A3S handoff）。
- 円の外側は透明。`color = [r, g, b, a]` は既存契約どおり straight sRGB
  として受け、analytic coverage \(C\) に対する出力は
  **premultiplied** `[r*a*C, g*a*C, b*a*C, a*C]` とする。straight color
  をそのまま coverage 倍して premultiplied と呼ばない。
- **純関数契約**: 同一 `(t, params)` から同一画素を返す。前 frame、再生順、wall clock、内部乱数状態を使わない。
- **Draft／Final**: 現行 `LayerSourcePlugin::render` という同一 entry point を使い、本 v1 は同じ入力なら同じ結果を返す。現行 `LayerSourceContext` に `Quality` は無いため、本決定で架空の Quality 差や引数は宣言しない。Quality 追加要否は A3S で公開 API 拡張を前提にせず審判する。
- **VRAM 常駐**: ピクセルは wgpu テクスチャとして GPU に生成し、製品経路に CPU frame を出さない。

## 4. Parameter 閉集合（これ以外禁止）

| id | type | domain | default | 意味（本書の正本。製品Panel投影は§6） |
|---|---|---|---|---|
| `count` | F64 | integer, inclusive `1..=64` | `12` | 円の個数 |
| `radius` | F64 | `>= 0` | `0.30` | 配置円の半径（正準）。各 instance 中心が乗る円の半径 |
| `dot_radius` | F64 | `>= 0` | `0.04` | 各円の半径（正準） |
| `phase` | F64 | finite, unbounded | `0`（rad） | 全体位相（ラジアン）。0 = +X 軸基準 |
| `angular_speed` | F64 | finite, unbounded | `0`（rad/s） | 角速度（ラジアン／秒）。正 = CCW |
| `color` | Color | 既存 Color 契約（straight sRGB、各成分 `0..=1`） | opaque white = `[1.0, 1.0, 1.0, 1.0]` | 塗り色。出力は straight→premultiply 後に coverage を掛ける（§3） |

A3R の `spread`／`size`／`speed`／`seed` は採用しない。`size`→`dot_radius`、`speed`→`angular_speed` の対応のみ記録し、別名併存はしない。

## 5. `count` 上限 64 — resource／performance 根拠

`count` の inclusive 上限 `64` は **この plugin version（`core.layer_source.radial_repeater` v1）の宣言された expression domain** である。作者が保証する評価域であり、**Host 全体の plugin／LayerSource に対する全球上限ではない**。

根拠:

1. **有界 instance 数**: 固定上限により、1 回の評価で扱う円の数が最大 64 と見積できる。WGSL のループ展開、dispatch グリッド、uniform バッファ、中間テクスチャの Draft コストを実装前に概算できる。
2. **無限 clone の先焼き回避**: Duplicator／InstanceId／stable seed 契約を本 LayerSource に先取りして焼かない。`count` を作品側の複製意味と混同しない。
3. **性能モデルとの整合**: [performance-model.md](../performance-model.md) の「表現ごとにコストを宣言する」規律に沿い、本 plugin が責任を持つのは 64 円までの union 評価域のみと宣言する。他 plugin がより大きな `count` を別 ID で宣言することを妨げない。
4. **Host 全球 ceiling 主張の禁止**: 「Motolii は LayerSource を最大 64 点まで」といった製品全体の上限は本決定では宣言しない。

## 6. Parameter Panel 要求と現行境界

M3 製品 UI は U4a 未着手である。歴史的 Slint 実装は非目標とし、次を要求する。

1. A3 実装は現行 `NodeDesc`／`ParamDef` が持つ表示名、parameter ID、型、
   default、F64 domainを正しく宣言し、Catalogから列挙できることを非UI fixtureで
   審判する。これはContract inspectabilityの審判であり、製品Parameter Panel完成の
   代用ではない。
2. §4 の単位、+X基準、CCW、union、premulは本書を意味正本とする。現行
   `ParamDef`にはこれらの説明metadataを投影する口がないため、A3／A3Sで新fieldや
   ID特例を発明しない。
3. [M3-U4a](../specs/M3-ui-integration.md)再開後、通常Effectと同じ自動生成
   Parameter Panelへ現行Contractとraw recipeの値／値source／診断を投影する。
   単位・軸・結合意味の追加表示が必要なら、GAP-13を含むM3側の仕様決定を先に行う。
   U4aが完了するまで、本書は「製品Panelから§4の全意味を検査可能」とは主張しない。

[UI操作言語 §5.4](../ui-interaction-language.md#54-parameter-panelを表現のホームにする)に従い、Parameter Panel を表現のホームとする。custom UI は [plugin-ui-v1](2026-07-12-plugin-ui-v1-boundary.md) の延期を維持する。

## 7. A3R 候補に対する採否

| A3R 候補 | 判定 |
|---|---|
| 決定論的 2D Radial Repeater LayerSource | **採用**（本決定） |
| seed／particle identity／Particle Field 一般化 | **延期** |
| Analytic Trail／path input | **延期** |
| role 付き Composite | **延期** |
| 物理 particle／Simulation | **延期**（VSM-A6） |
| rig／Authoring Tool／Kit mutation | **延期** |
| 汎用 Expression runtime | **延期**（v2） |
| Blender bridge | **延期**（独立 spike） |
| custom UI | **延期**（plugin-ui-v1） |

## 8. 明示的非目標

- random seed／particle／InstanceId 契約
- 任意 path 入力、texture sprite、multi-input Composite
- physics／隠れ状態／bake／Simulation
- 任意 Expression／JS／Python runtime
- Blender bridge、custom UI／Slint
- 新 `ValueType`、新 trait／公開 API／Document schema／`serde(default)` による意味捏造
- A3S lowering signature の本決定での確定実装仕様化（§11 の質問列挙のみ）
- コード・テスト・golden・crate・`.vism`／Kit／loader
- A3R 本文の書換え、Duplicator／P0I の identity 契約の先焼き
- Host 全球 `count` ceiling の宣言

## 9. 不変条件（docs 境界）

- 本 PR は文書のみ。永続 Document・registry・counter を変えない。
- A3R は歴史的調査として維持し、採用決定の正本は本書（A3D）である。
- 計画・README の相対リンクを切らない。
- analytic coverage のサンプリング詳細、AA、Draft／Final 画素許容差は A3S handoff へ回し、本決定で実装仕様として確定しない。
- A3の非UI Contract列挙fixtureと、M3-U4aの製品Parameter Panel conformanceを
  同じ完了条件として偽装しない。

## 10. STOP 条件（実装中即停止）

次のいずれかに当たったら実装を止め、推測補完しない。

1. 許可外ファイルへの編集が必要に見える。
2. A3R 本文の改変が必要に見える。
3. 新 `ValueType`／`ParamDef` field／trait／schema／migration／allowlist／lint 抑制が必要に見える。
4. `count` 64 を Host 全球上限として書きたくなる、または上限を外したくなる。
5. seed／path／Composite／Expression／Slint／コード／テストへ範囲が膨張する。
6. analytic coverage／AA／Quality の画素契約を A3D で「実装仕様として」確定したくなる → §11 へ回し、ここでは列挙のみ。
7. ユーザー指定の identity／式／parameter 表と矛盾する別名・追加 param を足したくなる。
8. 期待値・golden・fixture の削除／弱体化／書換えで「通す」発想に至る。

## 11. A3S handoff 質問（未決のまま残す）

本 ticket では答えない。VSM-A3S で仕様化する。

1. prepared recipe の kind 検査で、ID allowlist なしに 0-input LayerSource を render step へ下ろす公開 signature は何か。
2. 旧 `core.layer_source.clear` 特例の処分（一般化 vs 一時互換）はどうするか。
3. executor 欠落／contract-only／kind 不一致／未来 version の typed rejection をどう分類するか。
4. raw Document を不変のまま prepared params だけを渡す経路をどう固定するか。
5. built-in `doc.layer_source.rect` を registry plugin へ偽装しない分離をどう維持するか。
6. analytic coverage の定義（AA、カバレッジ結合＝union の数値意味、Draft／Final 同一関数下の許容）は何か。
7. §3のstraight→premul×coverage出力と既存 clear／Filter 色契約との接続をどう確認するか。
8. first-party 外部 crate 化時の contract／executor 登録と ID／version parity（A1／A2 型紙踏襲）をどう審判するか。
9. 現行Contractで列挙可能な表示名／parameter ID／型／default／domainと、raw
   recipeの値／値source／診断を、ID allowlistなしで将来U4aへ渡せる投影modelは
   既存境界だけで足りるか。不足時はA3でAPIを発明せずM3側の仕様改訂へ止める。

## 12. A3R §9 反対側論点への応答（短評）

| A3R 反対側論点 | 本決定での扱い |
|---|---|
| shader demo への縮小 | 幾何意味（円 union・正準座標・純関数）を §3 で固定。画素実装は A3S／A3 へ |
| `count`／`seed` による Duplicator 契約の先焼き | `seed` 不採用。`count` は本 plugin の expression domain のみ（§5） |
| LayerSource context の不足 | `Quality` 等は A3D で宣言せず §11 Q1／Q6 へ |
| Document への kind mirror 再導入 | 非目標。prepared recipe 経路は A3S |
| 大面積 editor の不当禁止 | 表現のホームはParameter Panelとしつつ、A3は現行Contract列挙だけを審判し、製品Panel conformanceはM3-U4aへ分離（§6） |
| Composite 延期による GAP 隠蔽 | §7 で明示延期。A3S GAP は §11 で列挙 |
| proprietary 製品からの過剰推論 | A3R 事実を採否表に限定。本決定は Motolii 契約のみ記述 |

## 13. 後続 ticket

| ID | 依存 | 内容 |
|---|---|---|
| VSM-A3S | 本決定 | §11 の handoff 質問を仕様化。コードは触らない |
| VSM-A3 | VSM-A3S | 本 v1 identity を A3S 一般経路と公開 API だけで外部 crate実装。U4a前は現行Contract列挙fixtureだけを審判し、製品Panel適合はU4aへ分離 |
