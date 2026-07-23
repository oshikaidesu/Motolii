# Group Source Pool / Cloner概念吸収

作成日: 2026-07-23

状態: **比較中／概念先行**。M5-P0Iへ渡す意味fixtureであり、Document schema、公開API、Group評価方式、UI実装の許可ではない。

関連正本: [Relative Move / Duplicator決定](2026-07-15-relative-scope-duplicator-decision.md)、[M5仕様](../specs/M5-3d-and-post.md)、[小さなCoreと探索可能な拡張](../extensible-core-model.md)、[first-party Vism表現需要調査](2026-07-23-first-party-vism-expression-demand-survey.md)

## 1. 吸収する製品命題

複数の映像ObjectをClonerへ渡すために、平面レイヤーへeffectを付け、別レイヤーのShapeを一つずつ参照接続する手順を標準作法にしない。MotoliiのGroup所有境界を利用し、**一つのGroupの直接の子をprototype poolとして扱い、子ごとの出現割合を同じ場所で編集できるCloner入力モード**を比較する。

これはxClonerをCoreへ写す話ではない。既決の`Input Shapes`、`Prototype Id`、stable `InstanceId`、Distribution、Behaviourを、制作者がObject tree上で直接理解できる所有形へ投影する話である。Group名、子の表示名、Timeline順を文字列走査して暗黙接続しない。

## 2. 二つの入力モードを混ぜない

| 入力モード候補 | 意味 | 主用途 |
|---|---|---|
| **Whole Prototype** | 一つのObjectまたはGroupを一つの原型として複製する | 従来型Cloner、完成した複合Objectの反復 |
| **Group Source Pool** | 指定Groupの直接の子を独立prototypeとして扱い、各instanceへ一つを割り当てる | 複数キャラ、図形、映像断片を一つの生成集合から混在出力 |

`Whole Prototype`を既定の従来経路として残す。Groupを入力しただけで自動的に子へ分解せず、`Group Source Pool`への切替を明示操作にする。切替は同じCloner／Distributionを維持し、layout、count、seed、Behaviourを作り直さない。

Group Source Poolは最初の比較では**直接の子だけ**を列挙する。nested Groupを再帰flattenすると、所有境界、transform、child order、欠落診断、cache依存が見えなくなる。nested Groupは一つのWhole Prototypeとして扱うか、明示的に別poolへ展開する後続比較へ残す。

## 3. Groupは配線の短縮ではなく所有境界になる

第一候補は、Clonerが外部Groupの表示名や現在の選択を追跡する形ではなく、Groupとその子の安定IDを型付きに参照する形である。

```text
Group A
  ├─ Prototype Red   weight 50%
  ├─ Prototype Blue  weight 30%
  └─ Prototype Star  weight 20%
          ↓ typed membership / stable child identity
Cloner: Group Source Pool
          ↓ Distribution + InstanceId + seed
addressable instance set
```

ただし、次の所有方式はまだ未決である。

1. 通常Groupを外部からClonerが参照し、Group自身も通常どおり描画できる。
2. Groupの明示`Output Mode`を`Composite Children / Clone Children`で切り替え、後者では子をprototypeとして所有し、Group出力をCloner結果にする。
3. Clonerがowned source scopeを持ち、Object tree上はGroupと同じ折り畳み操作を提供する。

2は配線と二重描画を最も減らせる一方、汎用Groupの評価意味を増やす。3は意味をClonerへ閉じられるが、Groupとほぼ同じ所有UIを別概念として増やす。P0Iでは1〜3を同じfixtureで比較し、見た目だけ同じ隠れGroup、source移動、visibility切替を実装しない。

## 4. Percentageは表示と意味を分ける

子ごとのUIは百分率表示を第一候補にするが、内部意味を曖昧な`percentage`一語へ固定しない。比較対象は二つある。

| 配分方式 | 意味 | 利点 | 代償 |
|---|---|---|---|
| **Weighted Stable** | 非負weightを合計100%へ正規化し、`hash(seed, InstanceId, prototype_channel)`を累積区間へ写す | count増減後も残存InstanceIdのprototypeが変わりにくい。空間へ自然に混ざる | 小countでは表示割合と実個数が厳密一致しない |
| **Exact Quota** | count×percentageを最大剰余法等で整数個数へ確定し、合計をcountへ一致させる | 10個の30%を必ず3個にできる | count・割合・並び変更で既存instanceのPrototype Idが再割当され得る |

初期推薦は`Weighted Stable`である。Motoliiの`InstanceId != index`、count増減時のidentity維持、seed決定性と最も整合する。UIは設定割合に加えて現在countでの実出現数を投影できる。厳密な内訳が制作意味として必要だとfixtureで確認できた場合だけ、`Exact Quota`を別の明示モードとして比較する。

共通規則候補:

- weightは有限の非負値。全sourceが0なら型付き拒否し、先頭sourceへ黙ってfallbackしない。
- 0%はsourceをpoolから外す明示値。通常の表示／非表示状態をweightへ流用しない。
- 合計が100でなくても相対weightとして正規化し、UIが正規化後の割合を表示する。保存値をUI操作の丸めで毎回書き換えない。
- child renameとObject tree上の表示順変更だけではprototype identityを変えない。
- source欠落を残りへ黙って再配分しない。Projectを保持したまま評価／export不能理由を識別する。
- prototype割当は`Prototype Id` channelの一つであり、Position、Time Offset、Visibility等のBehaviourと同じく純関数評価する。

## 5. Motoliiらしい操作形

比較する最小操作は次である。

1. 複数Objectを選択し、`Group as Source Pool`で一つの所有scopeへまとめる。
2. 同じ操作でClonerを作るか、既存ClonerのInput Modeを`Group Source Pool`へ切り替える。
3. Object tree／Inspectorに子のthumbnail、名前、割合、現在の実出現数を一列で表示する。
4. 子の追加・削除・並べ替え・割合変更を各1 gesture = 1 D2 macroで確定する。
5. Stageでは生成結果を直接確認し、元sourceとの接続はGroup境界からClonerへ辿れる。
6. `Whole Prototype`へ戻してもlayout、count、seed、Behaviourは保持し、source解釈だけを切り替える。

Cloner作成の裏でNull、controller、expression、平面layer、非表示source copyを生成しない。通常Groupのchild visibilityをprototype有効化の隠れスイッチにしない。

## 6. P0Iへ追加する反証fixture

1. 3 sourceを50/30/20で100 instanceへ配分し、同じDocument／seed／時刻でPrototype Id列が一致する。
2. count 100→120→100で、共通slotのInstanceIdと`Weighted Stable`のPrototype Idが一致する。
3. child rename、thumbnail変更、Object tree表示順変更でidentityと割当が変わらない。
4. weight変更はPrototype Id channelだけを再評価し、Distribution slot identity、Position、seedを再生成しない。
5. 0/0/0、非有限、負weight、空Group、source欠落、循環を型付き診断する。
6. Whole PrototypeとGroup Source Poolの切替をUndo/Redoし、layout、count、Behaviour、seedが往復一致する。
7. nested Groupを暗黙flattenせず、一つのprototypeまたはunsupportedとして識別する。
8. 1,000 instanceでもTimeline row／Document Objectを1,000件生成せず、UI thread同期readbackを行わない。
9. 2D/3D／Depth policyを切り替えてもGroup ownershipとprototype assignmentを作り直さない。
10. source poolの所有方式1〜3を同じ画と操作で比較し、hidden copy、名前検索、二重描画、二重stateが0である。

## 7. 停止線

- `Group Source Pool`を理由に現行Group schemaの解釈を黙って変える。
- P0I前に`percentage`、`source_mode`、child weightをDocumentへ追加する。
- exact quotaとweighted selectionを同じ設定値で状況依存に切り替える。
- Group childのTimeline順、表示名、thumbnailからprototype identityを導出する。
- source欠落時に残りsourceへ再正規化してFinal exportを成功扱いする。
- Cloner専用のGroup copy、Undo、resource pool、LayerSource backdoorを作る。
- Group内の全子を個別texture／Timeline layerへ展開し、GPU instance共有を失う。

## 8. Group LayerをMotoliiの意味的シャーシとみなす仮説

Group Source Poolから、より大きい製品仮説が見える。MotoliiのGroupは単なるfolder、transform parent、precomposeの代用品ではなく、**子のidentityと編集可能性を保ったowned setに、型付きの評価方針を適用する意味的シャーシ**になり得る。

```text
Group = owned children + stable identity + visible scope
                         ↓ typed interpretation
          Composite / Prototype Pool / Depth Participation
                         ↓
              texture / instance set / shared pass
```

この仮説の独自性は、Groupへ機能を直書きすることではない。「複数Objectを対象にする機能」が平面layer、Null、名前検索、隠れ接続、専用source copyを要求せず、同じ可視の所有境界を入力にできる点にある。

既に別々に存在するMotolii判断とも接続する。

| 現行判断 | Groupを通した読み方 | 変えてはいけないもの |
|---|---|---|
| 通常Group composite | 子をauthoring orderで合成し、Group effectを子合成後に1回適用 | 現行M2画素、effect順、未知保持 |
| Group Depth | 同じ子集合を共有depth passの参加scopeとして読む | 座標、子順、通常`Layer Order`の意味 |
| Group Source Pool | 同じ子集合を独立prototypeとして読む | child identity、Distribution slot identity、seed |
| 将来のBake／cache | Group子合成直後を再生成可能な成果物境界として読む | Document意味、Undo、通常評価の正しさ |
| Selector／Behaviour | Groupを対象集合の明示scopeとして参照する候補 | property path文字列、名前検索、独自Undo |

したがってGroup Coreが普遍的に所有する候補は狭く保つ。

- 子のownership、stable ID、順序、追加／削除／複製lifecycle
- scopeの型付き参照、循環拒否、欠落診断
- 通常のtransform／selection／UndoとObject tree上の可視性
- 評価方針が要求するcapabilityと、対応不能理由の投影

一方、ClonerのDistribution、Glow、Simulation、時間変換、export、任意custom UIをGroupの巨大enumへ集積しない。それらはVism、Host capability、Simulation、Delivery等の責任に残し、Groupを**入力scopeとして読む**。現行M2の「Groupにretimeを持たせない」「Group肥大化禁止」は維持する。

### 8.1 「Groupが主役」と「万能Group」は別である

Groupを製品思想の中心に置く場合も、すべてを`GroupMode`へする必要はない。第一候補の分離は次である。

```text
Group owns children
Capability interprets children
Host schedules resources and lifecycle
Vism supplies expression-specific policy
UI projects the same scope and causality
```

これならCloner、Depth、Effect scope、Bakeが同じGroupを利用しても、各機能のparameter、payload、resource、failureをGroup schemaへ混ぜずに済む。第三者Vismも「Timelineを走査して対象layerを探す」のではなく、Hostから型付きGroup scopeを受け取れる方向を比較できる。

### 8.2 GroupはPrecomposeではなく、Composite Viewを必要時に作る

Group化とPrecomposeを同義にしない。Group化はnode ownershipとscopeを作る構造操作であり、子をRGBA一枚の2D layerへ変換しない。子は引き続きObject、prototype、depth participant、addressable instance sourceとして評価できる。

| 境界 | 正本 | 時間／子identity | 出力 |
|---|---|---|---|
| **Group / Node Scope** | owned childrenと構造 | 親作品時間のまま。子identityを保持 | consumerに応じたtyped child set |
| **Composite View** | Groupを指定時刻に読む派生投影 | Group／子を置換しない | premultiplied RGBA texture |
| **CompositionClip / Precompose** | 独立Compositionへの明示参照 | 独立時間境界を持ち得る | Compositionの評価結果 |
| **Bake／cache artifact** | 同じDocumentから再生成可能な成果物 | Document／Undoの意味にしない | 検証済み一時成果物 |

既存M2の「Groupにretimeを持たせず、必要ならCompositionClip／precompを明示追加する」と整合する。時間境界が欲しい時にGroupを肥大化させず、構造整理だけで時間・座標・画素意味を変えない。

既存D3eの「Group effectは子合成後に1回」も、Groupそのものが常時2D layerだという意味にしない。texture 1→1のFilterをGroupへ適用する地点で、HostがGroupの**Composite View**を作り、そのviewへeffectを一回適用するというconsumer固有の評価である。Group Depth、Prototype Pool、object queryが同じGroupを読む時は、先にRGBAへflattenしない。

```text
Group children ── object/depth consumer ──→ typed objects
       │
       ├──────── prototype consumer ─────→ instance source pool
       │
       └──────── texture consumer ───────→ Composite View ─→ Filter
```

Composite Viewの生成地点は評価グラフで識別可能にし、隠れprecomp、Document上の2D layer、source copyを生成しない。texture化で失われるdepth、個体問い合わせ、子別mask等を後段が要求する場合は、対応するtyped capabilityを使うか構造化診断し、似た画へ黙ってflattenしない。

Group／Ungroupの基本操作は、Group固有transform／effect／評価方針を追加しない限り見かけ、子identity、時間、depth意味を保つ方向とする。world transformを保つための局所値変換が必要ならD2 macroで明示的かつUndo可能に行い、grouping操作からprecompose、Bake、cache生成を起動しない。

### 8.3 競合未到達は未検証の比較仮説

AEのprecomp／folder、CavalryのGroup／Duplicator、BlenderのCollection／Geometry Nodes、Cinema 4DのCloner、Unreal Motion Design等に部分的な同形がある可能性は高い。現時点では「どのcomposite softwareも切り込んでいない」を決定根拠にしない。

後続調査で問うのはGroupという名前の有無ではなく、次の組合せが一つの製品文法として成立しているかである。

1. 子identityを失わず、同じowned setを複数の型付き評価方針で読めるか。
2. source用の隠れlayer、Null、名前検索、expression、二重copyを要求しないか。
3. Composite／Instance／Depth／Bakeを切り替えても子を作り直さないか。
4. 第三者拡張が内部scene treeを走査せず同じscopeを利用できるか。
5. 欠落、循環、未対応能力をGroup単位で診断し、無関係な子編集を続行できるか。

この五点が既存製品で一体化されていないことを確認できた時、「Group LayerはMotolii独自の思想」という市場上の主張へ昇格する。

## 9. 現時点の処分

- **決定方向**: 従来のWhole Prototypeを残し、Groupの子をprototype poolとして読む明示モードをP0Iへ先行入力する。
- **推薦**: 百分率UIの初期意味は`Weighted Stable`。厳密個数は同じpercentageの隠れ挙動にせず、需要が証明された場合の別モードとする。
- **比較中**: Group参照、Group Output Mode、Cloner-owned source scopeのどれが所有正本か。
- **上位仮説**: Groupをowned setの意味的シャーシとし、Composite／Prototype Pool／Depth等を型付き評価方針として接続する。Group巨大enumにはしない。
- **決定方向**: Groupはnode ownership／scopeであり、Group化だけではPrecompose、2D layer化、retime、Bakeを起こさない。RGBAはtexture consumerが要求する明示Composite Viewで派生する。
- **未検証**: 他composite softwareが同じ五条件へ到達していないという市場比較。
- **非目標**: xCloner互換、nested自動flatten、Linked clone、per-source time offset、package／Vism schema、実装着手。
