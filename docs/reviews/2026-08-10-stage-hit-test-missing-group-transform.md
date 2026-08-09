# Stage hit-testにグループ変形が入っていない（finding）

日付: 2026-08-10
状態: **finding / 未処分。修理は発注していない**

## 0. この文書の扱い

`AGENTS.md`「**findingは権限ではない**」に従い、**報告と分類だけ**を行う。
**本書を根拠に修理を発注しない。** 処分はsupervisor席が別途決める。

## 1. 要旨

**Stageのhit-test経路が使う `world` に、グループ変形の継承が入っていない。**
描画経路には入っている。したがって**変形を持つグループの中の子は、
描画位置とhit領域がずれる。**

```text
描画   world = world(group_of(child)) * parent_chain(child) * local(child)
Stage  world =                          parent_chain(child) * local(child)
```

`Group` 自体は `StageGeometryUnavailable::Group` で投影対象外だが、
**子は可視列へ再帰的に入り個別にhit-testされる**ため、この差は隠れない。

## 2. 利用者から見える形

コード上の再現条件（実行はしていない）:

1. `RECT_LAYER_SOURCE` のRectangle clipを `Group.children` に置く
2. Rectangleの `transform.parent` は既定の `None` のまま
3. **Groupに非identityの平行移動・回転・scaleを設定**
4. Rectangleとgroupを可視・時刻内にする
5. Stage上で、**描画されたRectangleの位置をクリックする**

領域が重ならない変形なら、**描画されたRectangle上のクリックはmissし、
変形前の見えない位置が子へhitし得る。**

## 3. 発見の経緯

supervisorがグループのtransform bounds決定を2回起草し、2回とも反対側レビューで
`REJECT` された。2回目の敗因が「`StageLayerGeometry.world` はグループ継承済み」という
誤った前提だったため、その前提を実測した結果である。

**取り違えやすかったのは、repoに2つの `world` 合成が存在し、
Stage側にだけグループ継承が無いためである。**

- `crates/motolii-doc/src/spatial_resolve.rs` `ensure_world_affine` — `group_of` を再帰合成
- `crates/motolii-doc/src/affine.rs` `resolve_transform_rec` — `xform.parent` のみ合成
- `crates/motolii-ui/src/stage_geometry_projection.rs` `resolve_layer_world` — 後者を呼ぶ
- `crates/motolii-doc/src/graph.rs` `build_group` — 前者を子へ継承して描画

なお、構造上の親GroupとRectangleの `xform.parent` の両方へ同じGroupを設定した場合も、
Stageは1回、描画側は構造継承とparent鎖の双方から合成するため、一般には一致しない。
**2つの概念は自動同期されない。**

## 4. test

**該当ケースのtestは存在しない。**
構造Group内のRectangle childについて、Groupの変形を含むStage `world` または
実描画位置に対するhit-testを検証する既存testは確認できなかった。

## 5. 意図的な設計である可能性

実測時に「バグと断定する前に意図的な設計である可能性を検討せよ」を条件として与えた。
検討の結果、次の理由から**意図的な設計とは考えにくい**。

- `graph.rs` はグループ変形を子へ継承して**描画位置を変えている**。
  したがってグループ変形は視覚的意味を持つ
- 子は個別にhit-testされるため、Group自体が `Unavailable` であることでは差が隠れない

ただし**これは断定ではない**。処分時に前ownerの意図を確認する余地がある。

## 6. 影響範囲

gizmo、group root選択、group bounds、snap基準は**すべてこのhit経路の上に乗る**。
boundsを決めてもhitがずれていれば掴めない。

## 7. 非目標

- 本書を根拠に修理を発注すること
- 修理方法を指定すること（`ensure_world_affine` の流用可否を含む）
- `xform.parent` と `group_of` の関係そのものを再設計すること

---

## 付録 — 実測の全文

実施: Codex direct `gpt-5.6-sol` medium、`--sandbox read-only`、
`scripts/run-observed-cli.py` 経由、実行command 11件。

## FACTS

1. `resolve_transform_rec` が合成するもの

`resolve_transform_rec` が再帰するのは `Transform2D.parent` だけです。

- `xform.parent` を読み、指定されたレイヤーの `Transform2D` を `lookup` で取得して再帰します。`crates/motolii-doc/src/affine.rs:230` `crates/motolii-doc/src/affine.rs:240`
- 親がなければ identity、最終結果は `parent_m * local` です。`crates/motolii-doc/src/affine.rs:260` `crates/motolii-doc/src/affine.rs:262`
- この関数は Document の `Group.children` 構造を受け取らず、`group_of` も構築・参照しません。したがって、構造上のグループ入れ子は合成しません。`crates/motolii-doc/src/affine.rs:184`
- この経路の `placement_space` は、再帰的に解決された `parent_m` です。`crates/motolii-doc/src/affine.rs:239` `crates/motolii-doc/src/affine.rs:262`
- `placement_space` は通常位置から self の world 位置を求め、LookAt 時には world の from/to を親ローカルへ逆変換するために使われます。`crates/motolii-doc/src/affine.rs:204` `crates/motolii-doc/src/affine.rs:208` `crates/motolii-doc/src/affine.rs:216`

2. `ensure_world_affine` が合成するもの

`ensure_world_affine` は、構造上の `group_of` と `Transform2D.parent` の両方を合成します。ただし別々の鎖として扱います。

式にすると以下です。

```text
resolve_affine(id) = resolve_affine(xform.parent) * local(id)
world_affine(id)   = world_affine(group_of(id)) * resolve_affine(id)
```

根拠:

- `group_of(id)` があれば、そのグループの `ensure_world_affine` を再帰します。`crates/motolii-doc/src/spatial_resolve.rs:78` `crates/motolii-doc/src/spatial_resolve.rs:82`
- それを `ensure_resolve_affine(id)` の結果へ左から合成します。`crates/motolii-doc/src/spatial_resolve.rs:86`
- `ensure_resolve_affine` 側は `xform.parent` を再帰し、`parent_m * local` を返します。`crates/motolii-doc/src/spatial_resolve.rs:108` `crates/motolii-doc/src/spatial_resolve.rs:110` `crates/motolii-doc/src/spatial_resolve.rs:128`
- この経路の `placement_space` は `group_m * parent_m` です。`crates/motolii-doc/src/spatial_resolve.rs:114` `crates/motolii-doc/src/spatial_resolve.rs:118`

3. `group_of` と `xform.parent` の関係

同じものではなく、独立した概念です。グループへ子を入れても `xform.parent` は自動設定されません。

- `group_of` は永続フィールドではなく、Document の `Group.children` を走査して、その場で「直接所属するグループ」を構築します。`crates/motolii-doc/src/spatial_resolve.rs:24` `crates/motolii-doc/src/spatial_resolve.rs:186` `crates/motolii-doc/src/spatial_resolve.rs:197`
- `Transform2D.parent` は別の `Option<LayerId>` フィールドで、identity の既定値は `None` です。`crates/motolii-doc/src/schema.rs:391` `crates/motolii-doc/src/schema.rs:399` `crates/motolii-doc/src/schema.rs:403`
- Group の所属関係は `Group.children` が保持します。`crates/motolii-doc/src/schema.rs:781`
- `ParentLocator::Group` は `AddTrackItem` の挿入先を表します。`crates/motolii-doc/src/command.rs:85`
- `AddTrackItem` apply は対象の children vector に `item.clone()` をそのまま挿入し、transform parent を変更しません。`crates/motolii-doc/src/command.rs:1013` `crates/motolii-doc/src/command.rs:1038`
- `xform.parent` を変更するコマンドは独立した `SetTransformParent` です。`crates/motolii-doc/src/command.rs:299` `crates/motolii-doc/src/command.rs:724`
- 複製は既存の明示的 `transform.parent` を再写像しますが、新たにグループを parent として設定しません。`crates/motolii-doc/src/duplicate.rs:32` `crates/motolii-doc/src/duplicate.rs:208`
- `plan_detach_audio` も、挿入先が Group であっても `ItemEnvelope::new`、すなわち `parent: None` の item を生成します。`crates/motolii-doc/src/audio_edit.rs:97` `crates/motolii-doc/src/audio_edit.rs:108`

4. `StageLayerProjection.world`

`StageLayerProjection.world` は、構造上のグループ継承を含みません。

- Stage は `resolve_document_spaces` の `resolved` だけを保持し、返された `world_affine` map を捨てています。`crates/motolii-ui/src/stage_geometry_projection.rs:93`
- Rectangle の `world` は改めて `resolve_layer_world` で計算されます。`crates/motolii-ui/src/stage_geometry_projection.rs:158`
- `resolve_layer_world` は `resolve_transform`、すなわち `xform.parent` のみを合成する経路を呼びます。`crates/motolii-ui/src/stage_geometry_projection.rs:192` `crates/motolii-ui/src/stage_geometry_projection.rs:200`
- 明示的に `xform.parent = Some(group)` とした場合は、その transform-parent 関係としてグループ変形が入ります。しかし、構造上そのグループの子であることからは入りません。
- 描画側は `resolve_document_spaces` の world map を保存し、clip の `AffinePlace` に直接使用します。`crates/motolii-doc/src/graph.rs:261` `crates/motolii-doc/src/graph.rs:276` `crates/motolii-doc/src/graph.rs:425` `crates/motolii-doc/src/graph.rs:434`
- hit-test は `camera_view * geo.world` の逆変換を使うため、Stage world にないグループ変形は hit-test にも入りません。`crates/motolii-ui/src/stage_hit_test.rs:74` `crates/motolii-ui/src/stage_hit_test.rs:87`

グループ自体が `Unavailable::Group` なのは事実ですが、子は投影対象から除外されません。可視列挙は Group を追加した後、その `children` へ再帰します。`crates/motolii-doc/src/graph.rs:856` `crates/motolii-doc/src/graph.rs:862` Stage 側も子を再帰検索します。`crates/motolii-ui/src/stage_geometry_projection.rs:250`

5. 既存テスト

該当ケースを覆う Stage 投影／hit-test テストは存在しません。

- Stage の parent テストは `xform.parent` を明示したトップレベル二層で、構造 Group ではありません。`crates/motolii-ui/src/stage_geometry_projection.rs:371` `crates/motolii-ui/src/stage_geometry_projection.rs:389`
- Stage の Group テストは空の `children` を持つ Group が typed unavailable になることだけを確認しています。`crates/motolii-ui/src/stage_geometry_projection.rs:608` `crates/motolii-ui/src/stage_geometry_projection.rs:615`
- hit-test fixture は Rectangle を常にトップレベル `track.items` へ挿入します。`crates/motolii-ui/src/stage_hit_test.rs:133` `crates/motolii-ui/src/stage_hit_test.rs:143`
- product-host の Stage pointer fixture も常にトップレベルへ Rectangle を挿入します。`crates/motolii-ui/src/rn_product_host.rs:2380` `crates/motolii-ui/src/rn_product_host.rs:2397`
- Document 空間解決には、変形した Group 内の子を検証するテストがありますが、Stage 投影／hit-test ではありません。`crates/motolii-doc/tests/d3_lookat_resolve.rs:96` `crates/motolii-doc/tests/d3_lookat_resolve.rs:117` `crates/motolii-doc/tests/d3_lookat_resolve.rs:137`

## DISCREPANCY

**YES**

具体的な条件は、Rectangle が `Group.children` 内にあり、その構造上のグループ祖先の world affine が結果へ実質的な変化を与え、かつ同じ変形が Rectangle の明示的な `xform.parent` 鎖だけでは表現されていない場合です。

- 描画 world: `world(group_of(child)) * resolve_parent_chain(child) * local(child)`
- Stage world: `resolve_parent_chain(child) * local(child)`

なお、構造上の親 Group と `xform.parent` の両方を同じ Group に設定した場合、Stage はその Group を transform-parent として1回合成しますが、描画側は構造継承と transform-parent 鎖の双方から合成します。両概念が自動同期されないため、この形も一般には一致しません。

## USER_VISIBLE

**YES**

コード上の再現条件:

1. `RECT_LAYER_SOURCE` の Rectangle clip を `Group.children` に置く。
2. Rectangle の `transform.parent` は既定の `None` のままにする。
3. Group に非identityの平行移動、回転、または scale を設定する。
4. Rectangle と Group を可視・時刻内にする。
5. Stage 上で、描画された Rectangle の位置をクリックする。

描画は `resolve_document_spaces` のグループ継承込み world を使いますが、hit-test は子自身の parent-only world を使います。したがって、描画位置と hit 領域がずれます。領域が重ならない変形なら、描画された Rectangle 上のクリックは miss し、変形前の見えない位置が child へ hit し得ます。

グループ自体が hit 対象外であることだけではこの差は隠れません。子 Rectangle は可視列へ再帰的に入り、個別に hit-test されるためです。

## TEST_COVERAGE

**該当ケースなし。**

構造 Group 内の Rectangle childについて、Group の変形を含む Stage `world` または実描画位置に対する hit-test を検証する既存テストは確認できませんでした。

## EVIDENCE_GAP

コード上の判定に必要な未確認点はありません。テスト実行は行っておらず、判定は現行ソースと全 `project_stage_geometry` / `hit_test_projected_layers` 呼び出し・テスト所在の静的確認に基づきます。コード変更はありません。