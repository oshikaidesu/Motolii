# 箱から得られる物 — 自走の記録(提案)

2026-09-15 利用者「箱から得られる効果は?」→ 残りの 8 項目の表 →「今から仕事なんでそれぞれ全部自走していけるか?」→ 形を出して「はい」。
すべて提案(取り消せる)。関係の動きは連続性の物差しで跳び 0 を確かめてから次へ。

## 1. カメラが箱を収める

- 先例: Unity Cinemachine の **Group Framing Size**(対象の箱が画面に占める割合、1 で画面いっぱい)、3D ソフトの Frame Selected
- Camera 層に **Framing Size**(0 は使わない)。Target の層の箱の中心を注視点にし、注視点の面での倍率 = Zoom / Distance から、箱の縦横のきつい方がその割合になる Distance を解く(Orbit は箱の測りに入れない)
- Target を Hold で替えると、Camera 層の **Transition** で、注視点・奥行き・Distance の対数を区間の重みで混ぜて箱から箱へ移る(Camera の欄に Transition Duration / Easing を出す)
- 見本 `camera_frames.js`(格子に並べたカードの板を、カメラが 01 → 03 → 06 → 08 → 05 → 02 と渡り、最後に板全体を収める): 跳び 0・尖り 0(物差しにカメラの注視点と距離を足した)
- 試験: `framing_size_fits_the_target_box_on_screen_and_follows_it`

## 2. 変形の中心を箱の割合で

- 先例: CSS の **transform-origin**(キーワード top left / center / bottom right …)。AE では Motion Tools の「アンカーを角へ」で 1 回きり動かし、中身が変わるとずれる
- すべての物に **Transform Origin**(Anchor / Top Left / Top / Top Right / Left / Center / Right / Bottom Left / Bottom / Bottom Right)。Anchor は書いた px のまま、他は毎コマ層の箱から解く。並ぶ子も同じ(書いていなければ今どおり箱の中心)
- Motolii の Position は中心が居る場所なので、Bottom Left なら左下の角が Position に居続け、文字が伸びても、大きさを変えても、角から伸びる
- 試験: `a_transform_origin_keeps_its_corner_on_the_position_as_the_box_grows`

## 3. 親の箱への制約

- 先例: Figma の **Constraints**(Horizontal: Left / Right / Left & Right / Center / Scale、Vertical: Top / Bottom / Top & Bottom / Center / Scale)。CSS なら absolute の left / right の組
- 並べる Group の流れの外の子(Position Type = Absolute)に **Horizontal Constraint** と **Vertical Constraint**。親の箱の大きさが時刻で変わると、子の箱が付いていく
- **基準は時刻 0 の親の箱**(デザインした時の大きさ)。Right は右の辺からの距離、Left & Right は両方の辺からの距離(伸びる)、Center は中心からのずれ、Scale は割合を保つ
- 置き場所は並ぶ子と同じ Slot(移り方も効く)。伸びは Scale で(線も伸びる、Figma と同じ)
- 見本 `constraints_card.js`(幅と高さに鍵を打ったカードの中で、右上の丸・伸びる帯・中央の星・左上の題が付いていく): 跳び 0・尖り 0
- 試験: `free_children_follow_the_parents_edges_by_their_constraints`

## 4. 箱の輪郭を道に

- 先例: CSS の **offset-path: border-box**(道は包む箱の輪郭)、**offset-distance**(道の上の位置)、**offset-rotate: auto**(道の向きに回る)
- すべての物に **Offset Path**(None / Border Box)、**Offset Distance**(%、100 を越えると回る)、**Offset Rotate**(Auto / None)。道は親の箱(並べる Group の箱、Border Radius の角丸込み)、親が無ければ画面の枠。左上の角の後から時計回り
- Position の代わりに道の上の点を置く(Transform Origin・Transition はそのまま効く)。親の箱が変われば道も変わる
- 見本 `border_path.js`(大きさが呼吸する bento の各カードの縁を、白い札が回る): 跳び 0。尖り 65 は、角を曲がる時に回る札の軸に沿った箱の角が折れる分(位置の跳びではない)
- 試験: `an_object_travels_the_border_box_of_its_parent_and_turns_with_it`

## 5. 箱からの距離で効き方

- 先例: C4D MoGraph の **Plain エフェクタ + Fields の Box**(箱の中で強さ 1、外へ Falloff で 0。エフェクタの Scale・Position に強さを掛ける)
- すべての物に **Field**(層)、**Field Falloff**(px)、**Field Scale**(倍)、**Field Opacity**(倍、0..1)、**Field Push**(箱の中心から離れる向きへ px)。強さは場の箱までの距離(中にいれば 0)からの滑らかな段(smoothstep)
- 画面の見え方だけ(並び・書類の値は変えない)。Repeater の写しは 1 枚ずつの箱で効く
- 見本 `field_lens.js`(漂う半透明の箱の近くで、格子に並べた点が膨らんで退き、白い点は縮んで寄る)
- 試験: `a_field_box_swells_what_is_near_it_and_leaves_the_far_alone`
- 残り: 写しの Delay・色(Tint)に効かせる、場の形を Box 以外(Sphere・Linear)に
