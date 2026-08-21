# Transform hierarchy(グループ/親子/シェイプ階層)縫い目調査

日付: 2026-08-22 / 状態: **調査**(読み取り専用レーン。製品コード変更なし・cargo build/test 未実行)

発端: 利用者裁定(2026-08-22)「グループレイヤーと親子関係(親が動くと子も動く)をそろそろ実装すべき。シェイプのキーフレームにも適用され、グループも親子も再帰的に決まる」。検証対象の仮説:
> グループレイヤー・レイヤー parenting・シェイプ内階層は**単一の再帰的変換木の3つの顔**である。

## 0. 先に見つけた既存の起草文書 — この調査の土台

**[2026-08-20-group-layer-semantics-decision.md](2026-08-20-group-layer-semantics-decision.md)**(状態: 起草、未採番)が、今回の要求にほぼ直接答える設計を既に書いていた。要旨:

| # | 仕事 | 機構 | 現状(本調査で確認) |
|---|---|---|---|
| 1 | 変換の集約 | `parent`(層の一般属性) | **スキーマは実装済み**(`LayerAttrs.parent: Option<LayerId>`、循環拒否込み)。**eval 側は未消費**(§2 参照) |
| 2 | 整理と畳み | グループ(Document、member 列)+ fold(Session) | **未実装**。`LayerSource`/`LayerAttrs` に Group 相当が無い |
| 3 | 集合への不透明度・効果 | 隔離グループ `isolate: bool` | **未実装** |
| 4 | 性能 | フリーズ `frozen: bool` + フラット化(export 経路) | **未実装** |

**m/s/l**(検査系トグル)は解決済みの下位項目として先に着地している — `LayerAttrs.solo`/`.locked` は今回のコード読みで存在を確認(`hidden` は元々あった)。

この起草は「**変換だけが parent を流れる。opacity・効果・可視性は変換の木を流れない**」と明言しており、利用者の 2026-08-22 の言葉(「グループも親子も再帰的に決まる」)と字面は一致するが、**「Group への所属」と「parent」が同じ木か別の木かを明言していない**。これは本調査の最重要な未決点であり、§4 で判定する。

## 1. Store の現状(`next/core/motolii-store/`)

### 1.1 実装済み

- `LayerId(u64)` — 参照は常に安定 ID(裁定65)
- `LayerAttrs.parent: Option<LayerId>`(`next/core/motolii-store/src/attrs.rs:86`) — 循環拒否は書き口にある: `Intent::SetAttrs` → `validate_no_parent_cycle`(`next/core/motolii-store/src/document.rs:1143-1169`)。親鎖を辿って自分自身に戻らないかを `HashSet` でガードしながら検査する、**書き込み時点の唯一のガード**
- `LayerAttrs.parent` の doc コメント自身が明言: 「循環参照は絶対に作れない(layer-meta 束の柵)。作れると、**親を辿って transform を合成する日(未実装、resolve はまだ parent を読んでいない)**に無限ループになる」(`document.rs:1139-1141`) — つまり **schema は先に用意され、eval 消費だけが空席**という状態が、今回の起草文書より前(裁定65 系譜)から自己申告されていた
- `LayerSource::Null` — 「絵を持たず transform だけ持つ(AE の Null Object)。親子の受け皿」(`next/core/motolii-store/src/lib.rs:173-175`) — AE 型の「見えない親」を置く箱は既にある
- `order: i16` — 重ね順のみを持つフラットな値。parent/group とは独立(`meta.order`)。木構造を導入しても順序の正本を兼ねさせるべきではない(奥行きと親子は別次元、§4.2 で後述)

### 1.2 未実装

- Group を表す `LayerSource` variant が無い(`Solid`/`Media`/`Null`/`Shape`/`Text` の5つのみ)
- `LayerAttrs` に `isolate`/`frozen`/`group` 相当のフィールドが無い
- Intent 語彙に Group 関連の口が無い(`AddLayer`/`SetAttrs` の組み合わせで表現可能かは §5 で判定)
- `motolii-vector::Shape` は**1パス源 + 演算子スタック + fill/stroke 1つずつ**のみ(`next/engine/motolii-vector/src/lib.rs:56-65`)。複数 `Shape` を1つの shape-layer が持てる(`Layer:shapes` component が `Vec<Shape>`)が、**shape 間の入れ子・共有 transform・グループ opacity は存在しない**。旧世界の `VectorContent::Group`(§3 参照)に相当する概念が丸ごと無い

## 2. eval/engine の縫い目

**縫い目は1点に集中している**: `next/core/motolii-store/src/view.rs` の `StoreView::resolve_with_solo`(719-849行台)。

```
next/core/motolii-store/src/view.rs:822-829
let transform = LayerPlacement::from_transform(
    vec2(property::ANCHOR, [0.0, 0.0])?,
    self.resolve_position(layer, t)?,
    vec2(property::SCALE, [1.0, 1.0])?,
    scalar(property::ROTATION, 0.0)?,
    scalar(property::SKEW, 0.0)?,
    scalar(property::SKEW_AXIS, 0.0)?,
);
```

この1呼び出しは**その layer 自身の property track だけ**から local `glam::Affine2` を組む。`attrs.parent` は同じ関数内で読まれている(`attrs.hidden`/`attrs.blend_mode`/`attrs.matte`/`attrs.pinned` は使うが `attrs.parent` は未使用)。行列の意味の正本は `next/core/motolii-core/src/frame.rs:405` の `LayerPlacement::from_transform`(Lottie 順: anchor 引く→scale→skew→rotation→position、列ベクトル規約)で、これは**layer-local のみ**を扱う関数として設計されている(親合成はここに足すべきではない — 正本を汚す)。

`resolved_layers()`(`view.rs:908-918`)は `layers()` を1回走査して `resolve_with_solo` を呼ぶだけの**フラットな Vec**を返し、`order` でソートする。木は無い。

**engine/compositor は無改修で済む見込みが高い**(実測済み):

```
next/engine/motolii-compositor/src/lib.rs:394-396 ほか(4箇所同型)
(pinned_cancel * layer.placement.transform, 0.0)
```

`motolii-compositor` は `ResolvedLayer.placement.transform` を「layer 座標 → comp 座標」の**comp 空間アフィンとして直接**カメラ合成に渡すだけで、それが単一 layer 由来か親子合成済みかを一切問わない。つまり **`resolve_with_solo` の中で親を合成した world affine を書き込みさえすれば、compositor・engine のコードは1行も変わらない**。縫い目は `motolii-store` 1 crate に閉じている。

### 2.1 親子合成の挿入点(具体案)

`StoreView` に旧世界(§3)と同じ形の**メモ化・cycle 検出つき事前解決パス**を足す:

- `world_affine(&self, layer, t, memo: &mut HashMap<LayerId, Affine2>, visiting: &mut HashSet<LayerId>) -> Result<Affine2, StoreError>`
- `local = LayerPlacement::from_transform(...)`(既存のまま)
- `parent_m = match attrs.parent { Some(p) => self.world_affine(p, t, memo, visiting)?, None => IDENTITY }`
- `world = parent_m * local`
- `resolved_layers()` は `any_solo` と同じ「1パスで求めて使い回す」パターンに倣い、全 layer 分の `world_affine` を1回のトラバースで埋めてから各 `resolve_with_solo` に渡す(現状の `any_solo` 前計算と同型)

**キーフレームは各ノードローカルのまま、合成だけ再帰**という利用者の仮説どおりの形が成立する — `local` の評価(補間・track 読み)は今のまま touch 不要、`world` だけが新設の再帰関数の責務になる。

### 2.2 未解決(EVIDENCE_GAP)

- **ドラッグ中の overlay**(`transient: HashMap<TransientKey, Value>`)は layer 単位でスコープが切られており、`value_at_path` の中で `transient_value_at` が local property だけに効く(`view.rs:365-372`)。親をドラッグ中、子の world transform は「親の local が overlay で動く」→「子の world 合成にその overlay 値を伝播させる」という経路が今は無い。overlay は評価済みの値を直接持つ設計なので、`world_affine` 計算が overlay 越しの local を読めば自動的に効くはずだが、実装前に軽く検証が要る
- 循環検出は現状「書き込み時(`SetAttrs`)のみ」。旧世界は validate 時(静的)と eval 時(`visiting: HashSet`)の**二重の安全網**を持っていた(§3.4)。新 store も `world_affine` 側に `visiting` ガードを防御的に足すことを推奨(壊れた Document を読んだ時に無限再帰しないため — 書き込みガードは正しい Document を保証するが、旧ファイルの読み込みや将来のバグに対する第二の柵にはならない)

## 3. 旧世界の独自思想(egui 版 — `crates/motolii-doc/` + `crates/motolii-ui/src/timeline_editor/`)

旧世界(裁定 timeline-skia-route-confirmed 以前・2026-08-20 リセット前)は、**今回の仮説をほぼそのまま一度実装し尽くしていた**。以下は実コードからの復元。

### 3.1 スキーマ: 3つの独立した紐帯が1つの合成関数に集まる

`crates/motolii-doc/src/schema.rs`:

```rust
pub enum TrackItem { Clip(Clip), Group(Group) }        // :264-267
pub struct Group { pub envelope: ItemEnvelope, pub children: Vec<TrackItem> }  // :858-861
pub struct Transform2D { position, anchor, scale, rotation, pub parent: Option<LayerId> }  // :469-477
```

`crates/motolii-doc/src/param.rs:25-44`:

```rust
pub enum DocParam {
    Const(DocValue), Keyframes(DocKeyframeTrack), Data { .. },
    LookAt { target: LayerId, axis: LookAtAxis },   // 回転を「target を向く」に置き換える
    Follow { target: LayerId, offset: [f64; 2] },   // 位置を「target + offset」に置き換える
}
```

旧世界は**3つの別の紐帯**を持っていた:

1. **Group containment**(`TrackItem::Group.children: Vec<TrackItem>`) — 構造的な入れ子。正本は親側(Group)が持つ `Vec`
2. **`Transform2D.parent: Option<LayerId>`** — AE 型の参照リンク。Group の入れ子と**独立**(Group の中に居なくても任意の layer を parent にできる)
3. **`DocParam::LookAt`/`DocParam::Follow`** — **property 単位**の制約リンク(全 transform ではなく回転や位置の1成分だけを他 layer に委譲)。AE の expression(pick-whip)が担う仕事のうち最頻出の2パターンだけを型付きで先取りしたもの

さらに shape 側にも別の Group がある(`schema.rs:780-784`):

```rust
pub enum VectorContent {
    StandardShape { .. }, SvgAsset { .. }, TextPath { .. },
    /// パス合成用ネスト(タイムライン `TrackItem::Group` とは別概念)。
    Group { children: Vec<VectorContent> },
}
```

コード自身のコメントが「**タイムライン `TrackItem::Group` とは別概念**」と明言している。これは重要な一次資料 — 旧世界の設計者は「グループ」という同じ単語を2箇所で使いながら、**意図的に統合しなかった**。

### 3.2 eval: 事前解決パス + メモ化再帰(cycle 検出込み)、描画とは分離

`crates/motolii-doc/src/spatial_resolve.rs` が、**新世界の §2.1 で提案した設計とほぼ同型**の実装を持っていた:

```rust
// :78-90 ensure_world_affine — Group 継承込みの world アフィン
fn ensure_world_affine(&mut self, id: LayerId) -> Result<Affine2D, ParamEvalError> {
    if let Some(m) = self.world_affine.get(&id.get()).copied() { return Ok(m); }
    let group_m = match self.group_of.get(&id.get()).copied() {
        Some(g) => self.ensure_world_affine(g)?,   // Group 側の再帰
        None => Affine2D::IDENTITY,
    };
    let local_chain = self.ensure_resolve_affine(id)?;  // parent 側の再帰(下記)
    let world = compose_transform(group_m, local_chain);
    ...
}

// :108-118 resolve_affine_uncached — parent 側の再帰。group_m とは別に parent_m も合成
let parent_m = match xform.parent { Some(p) => self.ensure_resolve_affine(p)?, None => IDENTITY };
let group_m = match self.group_of.get(&id.get()).copied() { .. };
let placement_space = compose_transform(group_m, parent_m);
```

つまり **1つの layer の world 位置は `group_m * parent_m * local` という3項の積**であり、Group 継承(構造)と parent 参照(明示リンク)は**別の変数として計算され、最後に同じ `compose_transform` へ合流する**。circular 検出は `visiting: HashSet<u64>` で行い、`ParentCycle` と `SpatialLinkCycle`(LookAt/Follow 経由の循環)を型で区別する(`ParamEvalError`、`param_eval.rs:12-40`)。**描画順とは完全に独立**した1回の事前解決パスであり、`build_group`/`build_clip`(`graph.rs:345-408`)は解決済みの `world_affine` を引くだけ — 「変形は子へ継承(グループ1枚の事後リサンプルなし)」(`graph.rs:4` のモジュール doc)。

`build_group` の効果適用位置も明記されている(`graph.rs:380-393`): 子合成 → **グループの effect stack を子合成後の1枚に**適用 → clipping mask。これは今回の新世界の起草文書(§0)の「集合への効果=隔離グループ」と同じ発想の先行実装。

### 3.3 UI: `timeline_rows.rs` — 木を持たず毎フレーム平坦化するテスト済みアルゴリズム

`crates/motolii-ui/src/timeline_rows.rs`(206行、テスト込み)は**高品質・低結合・移植候補として最有力**:

- `TimelineRow { layer, kind: Object|Property(ParamRef), depth: u16, has_children, children_open, params_open }`
- `TimelineFoldState { children_open: HashSet<LayerId>, params_open: HashSet<LayerId> }` — **開閉は2軸独立**(Group の子を出す軸とキー行を出す軸)。モジュール doc: 「Document schema には入れない — 開閉は Undo の対象ではなく、Timeline の scroll/zoom と同じ棚(Project session)に置く」
- `rows(document, fold) -> Vec<TimelineRow>` が `Document` の再帰構造(`Group{children}`)を毎フレーム平坦化する。「木を持たない」の doc: 「描画も hit も『行の index』で引ければ済むので、木のまま持たない」
- indent: `crates/motolii-ui/src/timeline_editor/mod.rs:4422` `let indent = 8.0 + row.depth as f32 * 14.0;`
- テストが仕様を凍結している: `group_closed_hides_children_but_keeps_the_group_row`、`group_open_puts_children_directly_after_at_depth_plus_one`、`reopening_a_group_restores_each_child_fold_state`(畳んで開き直しても子孫の fold 状態が保持される)、`param_rows_and_child_rows_open_independently`

この設計原則(「fold は Session、Document には入れない」)は、**新世界の `2026-08-20-timeline-pane-semantics.md`**(現行の正典)が独立に到達した結論と**一致**している — 「Session(undo 対象外): 選択・playhead + scroll_y・zoom/view_span・**fold 開閉**」。旧世界からの連続性が実測で確認できる稀な例。

### 3.4 UI: 構造上の置き場(containment)と parent は別物という発見

`crates/motolii-ui/src/timeline_editor/mod.rs` の `ParentLocator`(`motolii-doc` 定義、`Track(TrackId)` / `Group(LayerId)`)は、**ドラッグ&ドロップで「どこに物理的に住むか」を表す型**であり、`Transform2D.parent`(spatial)とは無関係に存在する。行を掴んでドラッグすると `prepare_reparent_clip(*layer, ParentLocator::Group(group), i, None)` が呼ばれる(`mod.rs:2811`)— これは **containment の移動**であって、spatial `parent` を書き換えるコマンドではない。

その他の実測知見:

- **グループ化は新規 command を増やさない**: 「空の Group を置く」+「選んだものを `ReparentClip` で入れる」の組み合わせ(`crates/motolii-doc/src/command/track_item.rs:18-26` の doc)。逆操作(Ungroup)は既存の `RemoveTrackItem`/`ReparentClip` の逆で閉じる
- **Group の削除は中身ごと、1 Delete = 1 Undo**(`mod.rs:2586` 近辺のコメント、テスト `deleting_a_group_takes_its_children_and_one_undo_puts_them_back`)
- **lock は祖先チェーンから継承**(`effective_lock`、`mod.rs:5613`、コメント「親から受けている分も含める」)— 新世界の 2026-08-20 起草文書の「実効可視性は祖先チェーンの AND 導出」と同型の考え方が m/s/l 全般に及んでいた
- **D2 に「Group を消す」口が無い**という既知の穴: Ungroup は子を出すだけで、空になった Group はそのまま残る(`mod.rs:1658-1689` 近辺のコメント)。新世界で作り直す際に踏まなくてよい穴として記録
- **畳んだ Group は中身をその bar の中に visualize する**(`mod.rs:4727` 近辺のコメント「畳んである Group は、中身をその bar の中に出す」)— 折りたたみ時の視覚表現の具体案として移植候補
- 複製(`prepare_duplicate_track_item`)は Group の子と `VectorContent::Group`(シェイプ内入れ子)の**両方を再帰**するが、コードは別々の再帰関数(2つの木は統合されていない)

### 3.5 生きている思想 vs 死んだ実装

**2026-08-20 リセット裁定**([2026-08-20-reset-to-one-axis.md](2026-08-20-reset-to-one-axis.md))が移植方針を既に宣言していたが、**今日まで未着手**と確認できた:

| リセット文書の指示(§4 移植表) | 対象 | 本調査での実装状況 |
|---|---|---|
| 「`param_eval`/`pathgeom`/keyframe 補間/D2 command の**意味**は捨てる資産ではなく AE 化の中身として `motolii-schema`/`motolii-eval` へ移す」 | `spatial_resolve.rs` の group/parent/LookAt/Follow 事前解決パス | **未着手**(§2 の縫い目は今も空席) |
| 「`timeline_editor/`(9,059行)は**操作カタログの正本として意味関数だけ移す。UI は移さない**」 | `timeline_rows.rs`(rows/fold)、Group 化・削除・並べ替えの操作意味 | **未着手**(`next/ui/motolii-timeline-pane` は完全フラット、fold/indent/tree 皆無、§4 参照) |

つまり今回の利用者裁定は、**リセット文書が22ヶ月前(2日前)に「移す」と決めていて誰も実行していなかった宿題**を掘り起こした形になっている。

### 3.6 移植価値のある資産(優先度つき)

| 資産 | 移植コスト | 価値 | 判定 |
|---|---|---|---|
| `timeline_rows.rs` の `rows()`+`TimelineFoldState` アルゴリズム | 低(206行・外部依存は `motolii_doc` の3型のみ・テスト同梱) | 高(木を持たず毎フレーム平坦化、fold=Session の設計は現行正典と一致済み) | **移植推奨**。型を `Document::layers()`+`attrs.parent` 由来のツリー構築に差し替えるだけで、アルゴリズム本体(`push_item` の depth/fold 分岐)はほぼそのまま使える |
| `spatial_resolve.rs` の「メモ化再帰+cycle 検出+複数紐帯の compose」パターン | 中(直接移植不可 — `Document`/`ResolveCtx` の型が全く違う。**概念だけ**を §2.1 の設計に反映) | 高 | **概念移植推奨**。コードではなく「world_affine をメモ化 HashMap + visiting HashSet で1回だけ求める」という構造 |
| `ParentLocator::{Track,Group}`(containment の型付き置き場) | 低〜中 | 中 | Group を実装する段になったら同型の locator が要る(§4.2 で「二重帳簿にしない」設計と両立するかは要検討) |
| `DocParam::LookAt`/`Follow`(property 単位の制約リンク) | 中(新 `Value`/`PropertyId` 語彙が要る) | 中〜低(v1 優先度低) | 今回のスコープ外。将来、effect_param と同じ「平坦な名前」流儀(`link.{property}.target`)で足せる余地はある(`motolii-eval::Value::LayerId` は既に存在、§1.1 で確認) |
| `TrackItem::Group{envelope,children}` そのもののスキーマ | — | 参考のみ | rerun `EntityDb` はフラットな entity path 空間なので、「親が子の `Vec` を物理的に持つ」形はそのまま移せない(§4.2) |
| `VectorContent::Group`(シェイプ内ネスト) | — | 参考のみ | `motolii-vector::Shape` は現状「1パス源+op stack」のみで、この概念の受け皿が丸ごと無い(§1.2、§6 EVIDENCE_GAP) |

## 4. 単一再帰木仮説の判定

### 4.1 支持される部分: 評価アルゴリズムのレベル

旧世界の実測が示すとおり、「親の evaluated transform を子へ乗算する」という**合成の数学**は、紐帯の発生源(Group containment か、明示 parent か)を問わず**同じ関数**(`compose_transform`)に落とせる。新世界でも `LayerPlacement::from_transform`(local)→ `parent_m * local`(world)という2段構成にすれば、**キーフレーム評価は各ノードローカルのまま、合成だけ再帰**という利用者の仮説の実装形がそのまま成立する(§2.1)。この意味で **仮説は正しい** — 1つの再帰関数、1つの `Affine2` 代数で足りる。

### 4.2 反例: スキーマ(データ構造)のレベルでは3つの顔は同一ではない

- **旧世界は Group containment(`TrackItem::Group.children`)と `Transform2D.parent` を意図的に別フィールドとして持ち続けた**。両者は独立に選べる(Group の外にいる layer にも parent を張れるし、Group の中の子が Group 自身ではなく全く別の layer に spatial-parent することもできる)。もし「単一の木」であれば、この独立性は表現できない
- **AE のコミュニティ不満の内訳がこの区別を裏付ける**(`docs/reviews/2026-07-16-ae-layer-system-disposition.md` §4b): 「プリコンポなしのレイヤーグループ」への要望(2009年〜300票超)は「**グループ化と precompose を分けてほしい**」という不満であり、「parenting と grouping を1つにしてほしい」という不満ではない。AE の parenting(1個の親スロット参照)自体は元々グループ化と独立に機能しており、ユーザーが混同で困っているのは precompose(別タイムライン化という重い機構)の方
- **`VectorContent::Group` はコード自身が「タイムライン `TrackItem::Group` とは別概念」と明言**(§3.1)。シェイプ内階層は**座標系の粒度が違う**(shape の頂点はレイヤー内ローカル座標、op スタックの中に住む)— 利用者の言う「シェイプのキーフレームにも適用され」を素直に読むと、これは「shape 自身が持つ transform が pathの頂点に再帰的に効く」という要求であり、layer 間の親子とは**別の再帰**(木の深さも通貨単位も違う: layer 木は `LayerId` を結び、shape 木は `Shape` の配列内 index を結ぶことになる)
- **2026-08-20 起草文書(§0)自身が「変換だけが parent を流れる。opacity・効果・可視性は変換の木を流れない」と明言**しつつ、Group 所属の伝播対象を「検査系トグル(m/s/l)のみ」としている。これは**Group 木と parent 木がもし同一なら不要な作文**であり、書き分けている以上、起草者も両者を別物として扱っている(ただし前述のとおり「⌘G で同時に parent も設定するか」は明記されておらず、これが本調査最大の未決点)

### 4.3 判定

**「評価の合成アルゴリズムは単一の再帰(1つの `compose` 関数)で統一できる」は真。「データ構造(木そのもの)が単一である」は反例あり — 旧世界・AE 先例・現行起草文書のいずれも、Group 所属(整理目的)と parent(変換目的)を別の紐帯として扱っている。**

実装への含意: **①1つの `world_affine` 解決関数**(§2.1)を作り、**②その関数が読む「親」の出処を1本化する**のが妥当な着地点であって、「Group 木と parent 木を最初から同じフィールドに統合する」のは早すぎる可能性が高い。§5 の推奨案 (c) はこの判定を踏まえ、「Group はただの特殊な Layer であり、Group への所属は**その Layer 自身の `parent` を Group の LayerId にすることで表現する**」という形で**構造は1本(`parent` のみ)にしつつ、Group という『意味』は `LayerSource::Group` という別の印で表す**——これは仮説の「データも完全に1つ」ではなく「1本の参照リンクの上に、Group という特別な種別が乗る」という穏当な統合であり、旧世界が抱えていた二重帳簿の危険(§4.4)を避けられる。

### 4.4 二重帳簿の危険(旧世界が踏んでいない罠だが、新世界で踏みうる)

もし Group を「member 列を持つ独立エンティティ」(2026-08-20 起草文書の字面どおり「Group entity(member 列 + isolate + 検査系トグル)」)として実装すると、**「Group.members に入っている」と「その layer の parent が Group を指している」という2つの正本が同時に存在しうる**。旧世界はこれを「Group は `Vec<TrackItem>` を物理的に所有し、`parent` は別の独立フィールド」という形で**そもそも Group 所属を parent と結びつけていなかった**ので二重帳簿は発生しなかったが、rerun の `EntityDb` はフラットな entity path 空間であり、「親が子を物理的に所有する」というツリー構造をそのまま持ち込むのは不自然(§4.2)。新世界で Group を実装するなら、**正本は常に子側の1フィールド(`parent: Option<LayerId>`)に絞り、Group 側に `members: Vec<LayerId>` を持たせない**(必要なら `layers().filter(|l| attrs(l).parent == Some(group))` で毎フレーム導出、コストが問題になったら `TrackCache` と同じ RefCell キャッシュ層を足す)ことを強く推奨する。

## 5. schema 3案比較(new store 向け)

前提: `LayerAttrs.parent: Option<LayerId>` は既存資産として動かさない(裁定65「参照は LayerId」/循環拒否済み/undo は既存 `SetAttrs` 1発)。

| 案 | 形 | serde 後方互換 | undo 粒 | 循環検出 | Timeline 折りたたみへの適合 | 二重帳簿リスク |
|---|---|---|---|---|---|---|
| **(a) parent のみ**(追加ゼロ) | Group という概念を作らず、`parent` だけで「親が動くと子も動く」を実現。UI 側で「同じ parent を持つ集合」を疑似的に Group として見せる | 変更なし(実装済み) | 既存のまま | 実装済み(§1.1) | **弱い**。Group という明示境界が無いので、Timeline で「これは1つの折りたたみ単位」と言い切れない(parent を共有しない子も後から追加されうる。fold の対象範囲が曖昧) | 無し(単一正本) |
| **(b) Group entity**(旧世界型。member 列を持つコンテナ) | 2026-08-20 起草文書の字面どおり「Group entity(member 列)」。`GroupId` を新設するか、`LayerId` 空間を共有するかは要選択 | 新規 component 追加(additive、既存ファイルは無傷) | Group 作成 + N 件の membership 追加を `apply_all` で1 undo に束ねる必要(既存 `AddLayer`+`SetAttrs` の複合パターンを踏襲すれば可能) | **新設が必要**(member 列側の循環も別途チェックしないと、`parent` の循環検出だけでは Group 木の循環を防げない) | 強い(折りたたみ境界が明示) | **高い**(§4.4)。`members` と各子の「自分がどの Group に属するか」を同期し続けないといけない |
| **(c) parent + LayerSource::Group**(推奨) | Group は「子を持てる」という**印**を持つだけの特殊な Layer(`LayerSource::Group` variant を新設)。所属は既存の `parent` 1本槍で表現し、Group 側に `members` は持たせない | 新規 `LayerSource` variant の追加(additive、`match` の非網羅で**コンパイラが呼び出し側を全部教えてくれる**——構造的強制) | 「N 件選択→⌘G」は `AddLayer(Group) + SetMeta + N×SetAttrs{parent: Some(group)}` を `apply_all` で1 undo にできる。既存の複合 Intent パターンそのまま | **既存の `validate_no_parent_cycle` がそのまま使える**(Group も普通の LayerId なので、既存ロジックに変更不要) | 強い(`attrs.parent == Some(group_id)` で子集合を導出。§4.4 のとおり O(N) 走査+将来キャッシュ) | 無し(正本は `parent` のみ) |

### 推奨: (c)

理由:
1. 既存の cycle 検出・undo 粒・serde を**一切変更せずに**再利用できる(§1.1 のガードは `parent` の値がどの `LayerId` を指すかを問わない、Group もただの `LayerId` になるため無改修で効く)
2. §4.4 の二重帳簿リスクを構造的に避けられる(正本は常に1箇所)
3. §4.3 の判定(「合成アルゴリズムは1本、データ構造は Group という『意味』を持つ特別な parent 先」)と最も整合する
4. 2026-08-20 起草文書の「Group entity(member 列)」という字面とは異なるが、**起草文書自身がまだ「起草」段階(未採番)であり、実装時に一度 supervisor/利用者に this reconciliation を確認する価値がある**(§7 で明示)

非推奨とする (b) の使い道: 将来「1つの layer が複数 Group に同時所属する」(2026-08-20 起草文書は「第1弾は木」と明言し、複数所属を先送りしている)要求が実際に来たら、その時点で (a)/(c) の単純な木では表現できなくなるので (b) 相当の多対多構造を再検討する — 現時点では起草文書自身が単純な木で十分としているので (c) で足りる。

## 6. Timeline UI の縫い目

### 6.1 現状

`next/ui/motolii-timeline-pane/`(4,203行、11ファイル)は**完全にフラット**。`indent`/`fold`/`tree`/`child`/`parent`/`group` のいずれのキーワードもソースに出現しない(実測 grep 0件)。`canvas.rs` は `row_height` を全 layer 均一に積むだけで、行の階層構造という概念そのものが無い。

### 6.2 TL-P1(走行中、worktree)との関係

[2026-08-22-timeline-canvas-widget-survey.md](2026-08-22-timeline-canvas-widget-survey.md) が確度高いと判定した Phase 1(rail widget 化: 名前=native ellipsis、M/S/L=実 button、atlas 可視化)は、write-set が `timeline-pane`+`shell timeline系テスト` に限定されている([lane-board.md](2026-08-21-lane-board.md) 13行目)。**今回の調査(docs/reviews のみ)とはファイルが重ならないので衝突は無い**。

ただし将来、Group/parent の Timeline UI 化(indent 描画・fold トグル・行のツリー化)に着手する段になると、**同じ `timeline-pane` ファイル群を書くことになる**ので、TL-P1 の完了後に着手するのが妥当(逐次化であって衝突ではない)。TL-P1 が確定させる `RAIL_W`(行ヘッダ列幅、`2026-08-20-timeline-pane-semantics.md` の tokens 項目)は、indent の基準幅としてそのまま再利用できる見込み(旧世界の `8.0 + depth * 14.0` は実測値であり、新世界の RAIL_W 確定後に fixture で再較正すべき)。

I-tokens(write-set = `tokens`+`inspector-pane`+`inspector_pixel_fence`)は Timeline と無関係で、衝突なし。

### 6.3 normal-map 該当行

| id | canonical | verdict | 備考 |
|---|---|---|---|
| 455/456/457 | Group / Group selected shapes / Group Shapes | 採用予定 | ⌘G(edit_basic) |
| 468/469/470 | Ungroup / Ungroup selected shapes / Ungroup Shapes | 採用予定 | |
| 957 | Show or hide Parent column | 採用予定 | 理由列「parent実装済み(裁定112d)、UI列表示は未」— schema 実装済みの自己申告と一致(§1.1)。「裁定112d」は decision-index に見つからず、旧世界(pre-reset)の採番と推測(§8 EVIDENCE_GAP) |
| 910 | Precompose selected layers | **不採用** | 理由「GOALS要らないもの『プリコンポ/Nest/Compound clip—グループ化+ベイクへ置換済み』(裁定119)」— AE の3分割のうち precompose を Motolii は最初から作らない方針が既に確定している |
| 1334 | Group clips(timeline, Ctrl+G) | 採用予定 | 「他NLE用語との混同疑い」注記あり。455/456/457 と同一機能かは要確認(§8 EVIDENCE_GAP) |
| 1173 | Expression Pick Whip Writes Compact English | 不採用 | 一般 expression VM は非目標(旧世界の LookAt/Follow のような型付き先取りのみ検討余地) |

## 7. 切片割り案(重み均等・oracle つき)

依存順: 1 → 2 →(3 は多分無改修、確認のみ)→ 4。1 は 5 の実装、2 は 6 の実装、4 は Group を Timeline に可視化する。

1. **store schema**(write-set: `motolii-store` のみ): `LayerSource::Group` variant 追加(既存 match 網羅をコンパイラに教えてもらう)。Group 作成の複合 Intent パターン(`AddLayer`+`SetMeta`+N×`SetAttrs{parent}` を `apply_all` で1 undo)をテストで固定。§4.4 の「members を持たせない」を doc コメントで明示し、将来の実装者が二重帳簿を作らないよう縛る。oracle: 循環拒否(既存流用の回帰確認)・Group 削除時に子が孤児にならない(子の `parent` を `None` へ落とすか、子ごと削除するかの意味決定込み・旧世界は「中身ごと削除」だった §3.4)
2. **eval 再帰**(write-set: `motolii-store::view` のみ): §2.1 の `world_affine` メモ化再帰を実装。`resolve_with_solo` を local 計算(現状のまま)+ world 合成(新設)の2段に分離。oracle: 「親を N フレーム分移動させたキーフレーム track に対し、子の最終 world 位置を手計算した期待値と一致させる」数値証明テスト1本+「循環 parent を書き込もうとしたら書き込み時点で拒否される」既存回帰+「深さ K の親鎖でも O(K) で止まる」(メモ化の効果測定)
3. **engine/compositor 確認**(write-set: 無改修見込み、確認テストのみ `motolii-compositor`): §2 の実測(`layer.placement.transform` を素通しするだけ)を裏付ける統合テスト1本。もし何か想定外の依存が見つかったら独立切片へ格上げ
4. **Timeline UI tree 行**(write-set: `motolii-timeline-pane`+`motolii-shell` timeline 系テスト、**TL-P1 完了後に着手**): 旧 `timeline_rows.rs` の `rows()`+`TimelineFoldState` を概念移植(型を `next/core/motolii-store` 由来に差し替え)。indent は TL-P1 確定後の `RAIL_W` から導出。⌘G/Ungroup の D2 相当を新 Intent の複合パターンへ(旧世界の「新規 command を増やさない」原則を踏襲)。oracle: 旧世界のテスト5本(`group_closed_hides_children_but_keeps_the_group_row` 等)を新 store 向けに書き直して緑化
5. **Stage(親選択時のギズモ影響)**: 利用者裁定で明示的に範囲外(将来)

## 8. EVIDENCE_GAP

1. `motolii-compositor` が `layer.placement.transform` を local/world どちらの前提で消費しているかは4箇所の grep 一致から強く推測したのみで、実行時の統合テストは未実施(§7 切片3で埋める)
2. ドラッグ中の transient overlay が親→子の world 合成に自動で伝播するかは未検証(§2.2)
3. `motolii-vector::Shape` にシェイプ内階層(旧世界の `VectorContent::Group` 相当)を足す設計は本調査のスコープ外 — 利用者の「シェイプのキーフレームにも適用され」の要求のうち、shape 自身の頂点/パスへの再帰継承が必要なら**別の調査**が要る(layer 木とは別の粒度、§4.2)
4. `DocParam::LookAt`/`Follow` 相当(property 単位の制約リンク)は今回の Group/parent 設計に含めていない。将来必要になったら別発注
5. normal-map row 957 の「裁定112d」がどの文書を指すか特定できず(decision-index.md に該当なし、grep でも repo 内に見当たらない)。旧世界(pre-reset)の口頭裁定番号の可能性が高いが未確認
6. normal-map row 1334「timeline Group clips(Ctrl+G)」と row 455-457「Group(edit_basic)」が同一機能の重複計上か、Track グループ化と shape グループ化のような別機能かは未確認
7. 2026-08-20 起草文書の「Group entity(member 列)」という字面(§0)と、本調査 §5 の推奨 (c)(「Group 側に member 列を持たせない」)は**明示的に異なる**。この差分は次の設計判断(または利用者裁定)で解消すべき — 本調査は (c) を推奨するが、起草文書の原案 (b) 相当を supervisor/利用者が意図的に選んでいた可能性は排除できない

## 9. 走行中レーンとの衝突

**衝突なし**。本調査の成果物は `docs/reviews/` の1ファイル+ README 1行のみで、TL-P1(write-set: `timeline-pane`+shell timeline テスト)・I-tokens(write-set: `tokens`+`inspector-pane`+`inspector_pixel_fence`)のいずれとも write-set が交わらない。§7 切片4(Timeline UI tree 行)は TL-P1 と同じファイル群に触れる将来の実装であり、**TL-P1 完了後に着手**すべきという順序の指摘のみ(衝突ではなく逐次化)。
