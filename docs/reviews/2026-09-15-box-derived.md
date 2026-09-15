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

## 6. 箱から落ちる影

- 先例: CSS の **box-shadow**(offset-x offset-y blur spread color、ぼかしは標準偏差 blur / 2 のガウス)、Figma の Drop shadow(X・Y・Blur・Spread・Color)
- 並べる Group に **Shadow Color**(α 0 で無し)、**Shadow Offset**、**Shadow Blur**、**Shadow Spread**。背景と同じ形の書類の中で、背景の前に描く(背景が透明でも影は出る)
- ぼかしは効果を使わず、箱を広げて薄くした角丸の矩形 32 枚を外から内へ重ね、重なった濃さが外の 0 から内の α まで滑らかな段で上がるよう輪ごとの α を解く(描く道は形の道のまま)
- 影が箱の外へ出ると形の画布の原点がずれるので、Group の置き場所をその分戻す(背景と子が揃う。3D の札でも)
- 見本 `box_shadow.js`(机の上のカードが順に持ち上がり、影が遠くやわらかくなって、降りると戻る): 跳び 0・尖り 0
- 試験: `a_box_shadow_falls_below_the_box_and_leaves_the_box_in_place`
- 残り: inset、複数の影、形の層・文字の層の影(今は並べる Group の箱だけ)

## 7. 箱の大きさを単位に(container query)— 案だけ、相談待ち

実装しない(意味の範囲が広い。値の型と書類の形を変えるので、法を先に相談する)。

### 先例

| 先例 | 形 |
|---|---|
| CSS `container-type: size` と `@container (min-width: 600px) { … }` | 箱が「問われる容器」だと宣言し、中の規則をその箱の大きさで切り替える(画面の大きさの media query を置き換えた) |
| CSS の単位 `cqw` `cqh` `cqi` `cqb` `cqmin` `cqmax` | 容器の幅・高さの 1% を単位に、余白・文字・大きさを書く |
| Framer の Breakpoints | 1 つのページに幅の段(Desktop / Tablet / Phone)ごとの差分 |
| Figma の Variants + Auto Layout | 大きさ違いを別の型として持つ |
| AE | 無い(比率ごとに作り直すのが普通) |

### 意図 → 札

| 押す人 | 意図 | 札 |
|---|---|---|
| SNS の運用 | 16:9 で作った物を 9:16・1:1 で書き出したら、横並びが縦並びに組み直る | 並べる Group の段(幅が細ければ Flex Direction が Column) |
| リリック | 文字の大きさを枠の幅の割合で | 箱の単位の値 |
| 商品紹介 | カードが細い時だけ説明を隠す | 段ごとの差分(Opacity・Display) |

### 法の案(聞くこと)

1. **段(Breakpoint)は並べる Group が宣言する**: `(最小の幅, 欄の差分の組)` の並び。今の箱の幅が越えた一番大きい段の差分が、書いた値に重なる(Framer の Breakpoints、CSS の @container の書き方の順)。差分は欄の値だけ(鍵は段ごとに持たない)
2. **箱の単位は値の型にする**: Width・Height・Gap・Padding・文字の大きさなどに `12cqw` のような「親の箱の幅の %」の値を許す。式言語は作らない(2026-08-31 / 09-14 の改定)ので、単位 1 つの値として持つ
3. **比率違いの書き出し**は comp の大きさを替えて書き出すだけで、箱が Fill の作品は組み直る(今でも)。段と単位が入ると、崩れずに組み直る

聞くこと:
- 段の差分を書類に持つ形(欄の値の上書きの組)でよいか、Figma の Variants のように別の型(写し)で持つか
- 単位の値を持つ欄の範囲(全部か、大きさと余白だけか)
- 段が切り替わる瞬間の見え方(離散。移り方 Transition で滑らせるのが既定でよいか)

## 8. 編集中の Cmd 吸い付き(提案、実装)

利用者「cmdで適用かな」。先例 AE の Snapping(既定は切、Cmd を押している間だけ吸い付く)。宿題の案 2(既定は入)はこれで取り消し。

- 相手: 動かす層と祖先・子孫でない、描かれる層(Camera・Stage・Null を除く)の回っていない箱の左・中・右 / 上・中・下、と comp の枠
- 動かす側も箱の 3 本ずつ。軸ごとに一番近い 1 本へ、しきい値は画面の 6px(view の倍率で comp 座標へ)
- 吸い付いた x / y の線を Stage に comp の枠を横切って描く(離すと消える)
- 対象: 平面ケージの移動で、傾いていない時だけ。拡大の掴み・3D の軸・Flex / Grid の席の中は今は吸い付かない
- 試験: `a_moved_box_snaps_its_edges_and_centre_to_the_nearest_line_within_the_threshold`。手触りは実窓で

聞くこと: 相手にアンカー点を含めるか(AE)、等間隔の吸い付きを足すか(Figma)、拡大の掴みにも効かせるか。

## 宿題(2026-09-15 利用者「宿題にしておいてくれ」)

| # | 問い | 出所 |
|---|---|---|
| 1 | 容器の段の差分を欄の上書きの組で持つか、Variants のような写しで持つか | 7 |
| 2 | 箱の単位(cqw など)を持てる欄の範囲 | 7 |
| 3 | 段が切り替わる瞬間の見え方(Transition で滑らせるのが既定か) | 7 |
| 4 | Cmd 吸い付きの相手にアンカー点を含めるか(AE)/ 等間隔(Figma)/ 拡大の掴みにも | 8 |
| 5 | 抜き出しの効果(Extract・Color Key)を棚へ | [Stencil](2026-09-15-stencil.md) |
| 6 | 既定の 3D の札で、同じ面の層の重なりが order でなく距離で決まる | [百合の分解](2026-09-15-lilium-clone-anatomy.md) |
| 7 | Group の時刻が子の時刻を切らない(AE のプリコンプの感覚と違う) | 同上 |
| 8 | 実窓の検収: Cmd 吸い付きの手触りと線、棚の Found Grid、Inspector の新しい欄 | 1〜8 |

判定語: 宿題。

## 見本「渋谷の窓」で分かったこと(2026-09-15)

スクラッチ `tokyo_windows.js`(Pexels の渋谷 3 本を 6 枚の窓に、10 秒): 散らばった窓の辺から Found Grid が線を立て、窓が揃うほど線が減る → 吊った破線で順に結ぶ → カメラが Framing Size で窓から窓へ、最後の窓をくぐる。跳び 0・尖り 0。

- **Found Grid は並べる Group を 1 つの箱として読む(提案)**。今までは Group を飛ばしていたので、切り抜いた窓(Overflow Clip の Group)が格子に効かなかった。線は立てるが寄せない — 寄せた位置は画面の変換だけで、子と、箱を読むつなぐ線・札に届かない
- 宿題に足す: **寄せた位置がつなぐ線・札に届かない**(葉を寄せても角の掴みとロープは寄せる前の箱を指す)。**映像の層の箱は解析の大きさに頼る**(解析の無い読み方ではつなぐ線・札が壊れる)

## 格子は奥行きに立つ(提案 2026-09-15)

利用者「グリッドはオブジェクト関係に付与されるものであって、カメラではなくない?」「根本原因は奥行きを考えきれていないこと」「3d のグリッド表現」。

- 平面の見本でズレた訳: 窓も格子の層も既定の 2.5D で、2.5D は層ごとに自分の中心を基準にカメラへ写す。中心の違う格子の層だけ寄った時にずれた。格子は画面ではなく物の関係に付くので、物と同じ空間に置き、同じカメラで写す
- **3D の Found Grid は格子を空間の線で描く**: 物の箱の辺から x・y の面、物の奥行きから z の面を立て(どれも近い物を 1 つに寄せる)、面どうしの交線を線にする。奥行きの面の上に x・y の線(画面の幅・高さいっぱい)、x と y の交点に一番手前から一番奥までの柱。線は寄り合っても消さず重みで薄める(出たり消えたりしない)。奥行きが 1 つなら柱は無く平面の格子と同じ
- 描く道は粒子の Plexus と同じ re_renderer の 3D の線分(焼かない)。初版は奥行きごとに平面の格子の写しを立てたが、柱が描けないので置き換えた(利用者「なぜ奥行きの線は出せない?安いのに」— 道は既にあった)
- 3D の Found Grid では Box / Marker はまだ描かない(格子だけ)
- 試験: `a_lattice_stands_sheets_at_the_depths_and_pillars_between_them`
- 見本 `tokyo_depth.js`(スクラッチ): 跳び 0・尖り 0
- 同時に直した物: なぞる形・つなぐ線の `Margin`(箱からの間合い)が間合いの法の押し合いにも効き、角の掴みが奥行きに押されて跳んだ → つなぐ線となぞる形は押し合わない(付いて置く札と同じ)。試験 `a_trace_keeps_its_margin_to_its_box_and_is_not_pushed`
- 宿題に足す: 奥行きを跨ぐつなぐ線(今は線の層の平面に描くので、奥行きの違う箱をつなげない)。3D の層の重なりが order でなく中心の距離で決まり、遠い窓の札が近い窓の上に出ることがある
