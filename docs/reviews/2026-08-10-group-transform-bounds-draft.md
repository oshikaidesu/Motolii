# グループのtransform boundsをどこから採るか（起草・未採択）

日付: 2026-08-10
状態: **起草 / 未採択。反対側レビュー未実施**

## 0. この文書の扱い

**決定ではない。** supervisorが起草した案であり、反対側レビューを通すまで採択しない。
`decision-index` へも未登録である。**本書を根拠に実装を発注しない。**

## 1. 問い

[グループrootを掴んで動かす鎖](2026-08-10-group-drag-call-site-sketch.md)の `???` #7
`???_evaluated_group_bounds_for_picking` を閉じる。

**グループを掴むとき、handleが囲む矩形は何か。**

この1点が確定しないと、同鎖が示すとおり次の6つが決まらない。
group自身のhit領域、root候補、pivot、handle位置とhit領域、snap基準、preview dirty領域。

## 2. 利用者の要求（一次情報）

利用者（2026-08-10）:

> プリコンプに直すとギズモが全て1920pになるのが本当に嫌いで、
> そのためにオートクロッププラグインがあるぐらい

After Effectsのprecompは自前canvasを持つ入れ子compであるため、
レイヤーのboundsが**中身と無関係にcomp寸法**になる。
サードパーティのauto-crop pluginが存在するのは、この既定を実務で戻す需要があるためである。

## 3. Motolii側の既決（引用）

- **precompを作らない。** グループ化（再帰可）+仮出力（ベイク）で置換する（`concept.md:153` / `:198`）
- **グループはクリップと同じ項目エンベロープを持ち、
  グループのエフェクトは子を合成したフラットな1枚に適用する**（`concept.md:199`）
- Hostが**bounds／picking参加境界**を所有する（`specs/M5-3d-and-post.md:113`）
- **Unknown boundsをsort/cull根拠にせずFinalを切らない**（同 P2D oracle）
- `M4-P01-REGION` が RoD / RoI / tile extent / unknown propagation を扱い、
  OpenFXを`PATTERN`とする。**K0でRoD/RoIの契約意味はtest-only spikeとして凍結済み**
  （`implementation-ledger.md:50`）
- `P01-C2`: **output要求から各input RoIへ逆伝播し、非対応nodeはfull RoDへfallbackする**
- `P01`oracle: **Unknown/Infiniteは過小評価0、empty/overflow安全**

## 4. 案

**グループのtransform handleが囲む矩形は、
そのグループが合成した子の範囲であり、composition寸法ではない。**

### 4.1 canvas由来を採らない

Motoliiにprecompは無く、グループはcanvasを持たない。
`concept.md:199` が定めるのは「子を合成したフラットな1枚」であって、
「composition寸法のcanvasへ子を描いたもの」ではない。
**canvas由来はMotoliiのグループ意味論に対応物を持たない。**

利用者要求とも一致するが、**要求だけを根拠にしない**。既決から導ける。

### 4.2 グループ自身のエフェクト適用**前**の範囲を採る

`concept.md:199` の順序は「子を合成 → フラット1枚 → グループのエフェクト」である。
transformはそのフラット1枚に対する操作なので、
**handleが囲むのはエフェクト適用前の合成範囲**とする。

blur等でRoDが外へ広がっても掴む枠は動かない。逆に、
掴む枠を広げるとエフェクトを足すたびにhandleが動き、操作対象が変わって見える。

**これはpickingと同一ではない。** 見えている画素をclickして選べるべきかは別の問い（`???` #8側）であり、
本書では決めない。

### 4.3 `Unknown` を一級の値として扱う

子にRoDが確定しないものが含まれる場合、**推測で有限矩形を作らない**。
`Unknown` を伝播させ、消費側は既決どおり処理する。

- `Unknown` を sort / cull の根拠にせず Final を切らない（P2D oracle）
- 過小評価0（P01 oracle）。**過小に見積もるくらいなら全域へ倒す**

handleについては、`Unknown` のときcomposition全域へ倒す。
**これはcanvas由来を既定にすることではなく、fallbackである。**

### 4.4 空グループ

子が0のグループは範囲が空である。空矩形にhandleを出さない。
**空をcomposition全域へ倒さない**（fallbackはUnknownのためのものであって、空のためではない）。

## 5. 決めていないこと

- **picking領域**（見えている画素で選べるか）。`???` #8 と同じ層で別途決める
- 座標系と時刻の正確な指定。transformが時間変化する子を含む場合の評価時刻
- mask / clipping が範囲へ与える影響
- `Unknown` の具体的な判定条件と、どのnodeが `Unknown` を生むか
- RoD実装そのもの（`M4-P01-REGION` は `CONTRACT CLOSED / IMPLEMENTATION NOT STARTED`）

## 6. 非目標

- 本書を根拠に実装を発注すること
- gizmo採択の裁定（`BUILD JUSTIFICATION` は `NOT NONE` のまま）
- RoDの実装順序を変えること
- 利用者の選好だけを根拠に既決を覆すこと

## 7. 反証してほしい点

反対側レビューへ渡す。**特に次を疑うこと。**

1. §4.2「transformはフラット1枚に対する操作」は `concept.md:199` から本当に導けるか。
   エフェクト適用前後のどちらが対象かを、あの1文は決めているか
2. RoDは**描画のための定義域**であって、**掴むための矩形**ではない。
   §4.3でRoDのoracleを流用しているのは飛躍ではないか
3. §4.1「canvas由来は対応物を持たない」は正しいか。
   グループの仮出力（ベイク）は寸法を持つのではないか
4. `Unknown` fallbackを全域にすると、利用者が嫌う1920p挙動が別経路で復活しないか
