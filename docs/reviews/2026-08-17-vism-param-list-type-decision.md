# Vism parameter に同種の並びを足す

作成日: 2026-08-17

状態: **決定**（型と規則）＋ **型・規則は実装済み（2026-08-17）／GPU受け渡し(2.4)とUIウィジェットは未実装**

関連: [Vismプラグインカタログ](../vism-plugin-catalog.md)、[VSM-A4I外部作者経路の実測と汎用化](2026-08-17-vsm-a4i-external-author-path-measurement.md)、[プラグイン作成](../plugin-authoring.md)

## 0. なぜ要るか

SINGLE lane 12件を外部作者へ渡せる状態にした（A4I の汎用化）。当てて確かめると、**11件は現行の公開境界に収まり、Gradient Ramp だけが収まらない**。

公開されている pipeline は 2形しかない（`crates/motolii-gpu/src/pipeline_cache.rs:56,72`）。

- `get_or_create_fullscreen_uniform16` — 入力なし + 16 float（LayerSource）
- `get_or_create_tex_sample_uniform4` — 入力1枚 + sampler + 4 float（Filter）

Gradient Ramp は stop 1つあたり color 4 + 位置 1 = 5 float であり、16 float では **3 stop が限界**である。

ただし本質は容量ではない。`ValueType`（`crates/motolii-plugin/src/contract.rs:31`）に **同種のものが並ぶ型が無い**。`F64 / Vec2 / Vec3 / Color / AssetRef` はあるが、可変個を宣言する語彙が plugin 契約に存在しない。uniform を大きくしても N stop は N stop で頭打ちになる。**制限ではなく欠落である。**

## 1. 天井は残す。形だけ直す

固定長・型付きという制約は無償の制限ではない。次を支えている。

- **keyframe** — 全パラメータがスカラ／ベクトルで domain を持つのでキーが打てる
- **UI 自動生成** — `ParamDef` の `ValueType` + domain から Host がエディタを出せる
- **journal 互換** — 型が閉じているので replay と migration が追える
- **純関数契約** — 不透明なデータ入口が無く、前フレームを覚える抜け道が構造的に作れない
- **組み換えの必然性** — 任意のデータを受け取れると作者は一体型を作る。スカラしか受け取れないから、複雑なものは部品＋型付き接続で組むことになる

したがって次を分ける。

- **不透明・任意のデータ** — 入口を作らない。上の5つを壊す
- **既存の型の、可変個の並び** — 何も壊さない。純関数のまま、キーが打て、UI は既知のウィジェット、journal も型付きで追える

**後者だけを足す。**

## 2. 決めたこと

### 2.1 型

```rust
pub enum ValueType { F64, Vec2, Vec3, Color, AssetRef, List(Box<ValueType>) }
pub enum Value     { ..., List(Vec<Value>) }
```

- 要素は既存の型に限る
- **`List` の入れ子は許さない**（`List<List<T>>` は検証で弾く）
- 不透明なバイト列は入れない

Gradient Ramp は `positions: List<F64>` と `colors: List<Color>` の2本になる。長さの一致は plugin 側が検査する。構造体型を足せば1本にできるが、それは別の拡張であり本決定に含めない。

### 2.2 補間

`Value::lerp`（`crates/motolii-eval/src/value.rs:16`）へ `List` の規則を足す。

- 長さが同じ → **要素ごとに lerp**
- 長さが違う → 補間せず `a` を返す（バリアント不一致時の既存挙動と揃える）

### 2.3 keyframe の粒度 — **list 全体で1キー**

キーフレームは list を丸ごと保存する。補間は 2.2 により要素ごとに走る。

これで**どの要素でもアニメーションできる**。stop 3 の位置だけを動かすなら、その要素だけが違う list を2つ打てばよい。

できないのは**要素ごとに独立したタイミングを持つこと**である（stop 1 が t=0,5、stop 3 が t=2,7 という打ち方）。

**採らなかった案**: 要素ごとのキー。要素の安定 identity が要る。添字を identity にすると並べ替え・削除で壊れる（Blender の `color_ramp.elements[1].position` が既知の例）。安定 ID を持たせるなら Document・Undo・複製・journal を横断し、`VSM-B0` と同じ形の問いになる。After Effects は Gradient Ramp をグラデーション全体1プロパティで扱っており、stop ごとのキーを持たない。

**この決定は要素ごとのキーを塞がない。** 条件は一つ、**添字を identity として焼き込まないこと**。全体キーは要素を名前で指さないので、後から要素 ID を導入しても既存のキーと共存できる。逆順は不可能である（壊れる identity が Document に入る）。

### 2.4 GPU への渡し方

`PipelineCache` に形を1つ足す。storage buffer（`var<storage, read>`）が素直である。

`VSM-A3S-F1` が公開 `PipelineCache` 境界の欠落（0-input + uniform64 定型なし）を境界訂正として足した前例があり、同じ route に乗る。

**MULTIPASS の停止線には触れない。** カタログ lane 表の「`VSM-A8G0〜G1` 前に専用API、自前pool、loop内resource生成をしない」が禁じているのは transient texture・pool・loop 内 resource 生成であり、storage buffer の binding はどれにも当たらない。

### 2.5 検証規則

- `f64_domain` は `ValueType::F64` 以外で `None`（既存規則、`contract.rs:86`）。`List(F64)` でも `None` とする。要素の domain は本決定では持たない
- `List` の入れ子を弾く
- `default` の `Value` が `value_type` と一致すること（既存検証の延長）

## 3. 波及範囲（実測）

```
Value::      558 箇所
ValueType::   97 箇所
Value::AssetRef を含む網羅 match  11ファイル以上
  motolii-eval/src/track.rs, motolii-plugin/src/contract.rs,
  motolii-doc/src/{doc_value,param_eval,param_expect,plugin_resolution,
                   position_key_prepare}.rs, motolii-doc/src/validate/{params,asset_uses}.rs,
  motolii-ui/tests/u4a1_parameter_control.rs ほか
```

`Value` は `Serialize, Deserialize` 付き（`value.rs:4`）であり、Document 直列化と journal 互換が動く。

**移行の手順は既存の規律に従う。** 台帳のキー編集 API の行が定める形である — 足す → 旧は replay 専用 → 新規は汎用版。既存 variant は削除しない。

## 4. これで何が変わるか

- Gradient Ramp が SINGLE lane のまま成立する（カタログの「SINGLE、後に PORTS」を PORTS へ送らずに済む）
- 今日作れる11件は**作り直しにならない**。型の追加であって既存 variant は変わらない
- 残る天井は2枚。**2枚目のテクスチャ（mask/field）は `VSM-B2`**、**中間テクスチャと複数 pass は `VSM-A8G0/G1`**。どちらも本決定では触らない

## 5. 未実装

型・規則・粒度は決まっており、設計判断は残っていない。実装は網羅 match の追随と直列化・journal の追加であり、機械的である。ただし11ファイル以上を同時に動かすため、半端に始めるとビルドが割れた状態が残る。一括で入れること。

## 6. 実装で分かったこと（2026-08-17）

### 6.1 `List(Box<ValueType>)` は入らなかった — 要素型を別の型にした

**実装した形は §2.1 と違う。**

```rust
pub enum ElementType { F64, Vec2, Vec3, Color, AssetRef }
pub enum ValueType   { F64, Vec2, Vec3, Color, AssetRef, List(ElementType) }
```

根拠は推論ではなく、`List(Box<ValueType>)` を実際に入れて `cargo check -p motolii-plugin` を回した結果である。

```
error[E0204]: the trait `Copy` cannot be implemented for this type
error[E0004]: non-exhaustive patterns: `ValueType::List(_)` not covered
```

`ValueType` は `Copy` で、`as_str(self) -> &'static str` を持ち、`ParamDef`・`ParamConstraints` は `Copy` かつ `const fn` で組まれている。`Box` を入れると `Copy` が落ち、`as_str` も `&'static str` を返せなくなる（入れ子の型名は静的文字列にならない）。これは網羅 match の追随ではなく、公開境界の作り直しになる。

`ElementType` を別に置くと、`Copy`・`const fn`・`&'static str` がすべて残り、加えて **§2.1 の「`List` の入れ子は許さない」が検証規則ではなく型で保証される**。決めた中身（要素は既存型に限る／入れ子なし／不透明データなし）は一つも変えていない。

副作用として **§2.5 の「`List` の入れ子を弾く」は弾く対象が構文上作れない**ため、検証項目としては消えた。`Value::List(vec![Value::List(..)])` は値としては書けるが、`value_matches_type` がどの `ValueType` にも一致させないので受け口に入らない。

### 6.2 決定に書かれていなかったが必要だった型が2つある

§2.1 は `ValueType` と `Value` だけを挙げているが、keyframe を保存し検証するには次の2本も並びを持つ必要があった。どちらも決定文書に記載がない。

- `DocValue`（`crates/motolii-doc/src/doc_value.rs`）— 永続層の値。**keyframe は list 全体で1キー（§2.3）なので、保存されるのはこの型である**
- `ExpectedValueType`（`crates/motolii-doc/src/param_expect.rs`）— doc 受け口の期待型。`Copy` + `const fn` なので `ExpectedElementType` を同じ理由で対にした

評価層の `Value` と永続層の `DocValue` を分ける設計は既存のもの（`doc_value.rs` の冒頭に理由が書かれている）であり、本実装はその分離をまたいでいない。

### 6.3 空 list の扱い（決定に無い規則を1つ足した）

`List` の要素型は値からは先頭要素でしか分からず、空 list は要素型を名乗れない。次のようにした。

- `value_matches_type(List(T), Value::List([]))` → **一致する**。長さは plugin 側の関心であって型の関心ではない（§2.1「長さの一致は plugin 側が検査する」に合わせた）
- 型名表示は空なら `List`、要素があれば `List<F64>` 等

### 6.4 要素ごとの domain は入れていない

`validate_param` の `unit_interval` / `min` / `max` / `integer` は**スカラのみに掛かったまま**にした。§2.5 の「要素の domain は本決定では持たない」に従う。`List(F64)` に `f64_domain` を付けると既存の `NonF64Parameter` が弾く（`value_type != ValueType::F64` のため）ので、§2.5 の「`List(F64)` でも `None`」は追加のコードなしに成立している。

ただし **`default` の有限性と Color の 0..=1 は要素へ降りて検査する**。これは domain ではなく値の健全性なので分けた。

### 6.5 まだ無いもの

- **§2.4 の storage buffer 形**（`PipelineCache`）— 未実装。使う plugin がまだ無い
- **UI ウィジェット** — `map_parameter_control` は `List` を `AssetRef` と同じく不支持で返す。Host 側の受け口が未決
- **Gradient Ramp 本体** — 型が入っただけで、plugin は書いていない
- **journal の往復試験** — `DocValue` は `Serialize`/`Deserialize` を継いでいるが、`List` を含む文書の replay は試していない
