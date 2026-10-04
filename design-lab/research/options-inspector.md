# Inspector 側: 価値の高い 10 問題と、操作モデルの違う案

2026-10-02。調査のみ。問題 ID は problems.md。**案は見た目でなく操作モデルが違う**。見た目は DESIGN.md の現行のまま、記号は増やさない。
書式は options-browser.md と同じ。**機構 / 契約 / 静止時に隠す物 / 最頻の手数 / レンズ(▲有利 ▼不利)/ 出所 / Widgetbook 試作**。`[新]` = 出荷されている道具に無い組合せ(試作で最初に手で判定)。
判断の基準(利用者の方針): (1) 実制作での便利さ・速さ・手触り、(2) 実際のワークフロー、(3) 良い先例、(4) Motolii 内の一貫。**実装の重さで案を選ばない**。過去の Motolii の契約は制約に使わない(何が不便だったかの材料として読んだだけ)。
既存部品の呼び名: `Doc`/`Ctx`(inspector_parts: 1 layer の値・key・複数選択 mixed・reveal・effect 順)、`ValueWell`・`ScrubField`、`inspector-gui`(pad/dial/scale/anchor、Hierarchy options)、`relation_set`(card・chip・ConnectionDemo・Add relation menu・`_Flow`)、Curve picker/`EaseThumb`/mini curve editor(timeline_parts)、`stage_set`(gizmo)、`Session`、Command palette。

**手癖の台帳**(案を「手癖のまま / 手癖への追加 / 手癖から外れる / 取り下げ」に分けた表)は experience-first.md §C。改訂方針: 手癖(AE/Ableton/Blender/Figma)を既定にし、分割は壊さない。I3-d・I4-c・I8-b は取り下げ(境界を壊すだけが取り柄)。

## 選んだ 10 問題とその理由
| 問題 | 選んだ理由 |
|---|---|
| I2 数値の速さ | 毎日一番多く触る操作。過去の不満(1% が 10% に)の実体で、先例同士も割れる。一規則で決めると他の全部が揃う |
| I3 空間値を手触りで | 利用者の既定方針(数値は 4 番目・GUI 主)。主従の配置が未決で、制作で最も触る |
| I4 時間で変わる | キーは motion の本体。状態を色だけでなく形・語で見せたい |
| I5 ease | 3 回作り直された領域(Flow・Ease and Wizz・Motion Easing)。「感覚で決める」が最も出る |
| I6 既定との差・戻す | 長時間作業で「どこをいじったか」が分からなくなる。先例が豊富で案が割れる |
| I7 長い一覧 | effect が増えるほど効く。絞る・畳む・探す・お気に入りの 4 モデルが違う |
| I9 値を結ぶ | AE の expression 嫌いの直接の対抗。Cavalry/Ableton が良い先例。**Relation は操作(Browser の棚にしない)の既存裁定の範囲で** |
| I11 一側面コピー | どの host も未解決(N)で Motion/Blender アドオンが埋めた。差を作れる |
| I8 effect の積み | 制作の骨格。順序・bypass・差替の操作モデルが割れる |
| I12 多数に一括・ずらし | motion graphics で頻出(同じ変更を 30 個に)。ずらし(stagger)の入口が割れる |
外した: I1(複数選択の正直さ)は先例が揃い案の分岐が小さい → 判断 D-I1 に決定だけ書く。I10(専門道具へ渡す)は I5 と X8 の中で扱う。I13(A/B・snapshot)は I6 の案 I6-d と B11-c に吸収。I14(parent 選択)は「検索付き選択+ Stage 上で拾う」で先例が揃う → D-I2。I15(利用者が操作面を絞る)は価値は高いが「Vism が自走」の範囲に近く別文書向け。

---

## I2. 数値を、速度の予測が付くまま変える

共通の下敷き(決定、D-I3): scrub は右で増える、1 目盛り=1 表示単位(固定)、Shift で x10、Alt で x0.1、矢印も同じ規則、wheel も同じ規則。以下は **この規則の上で、操作の入口が違う案**。

**案 I2-a「label を掴んで scrub、値を押して入力」** 機構: 行の label(名前)を左右に drag で scrub、数値部分のクリックで直接入力。
契約: label drag = scrub / 数値クリック(4px 以内)= 入力で全桁選択 / Enter 確定、Esc で入力前の値。
隠す物: 無し。 手数: scrub 1 drag、入力 1 クリック+打つ。
レンズ: ▲L9 L10 ▼L2。 出所: Figma(label scrub)、AE(label/値 drag)。
試作: `ValueWell` の label に drag、値に click 入力。

**案 I2-b「数値そのものを掴む(1 つの面で 3 役)」** 機構: 数値の面で drag=scrub、クリック=入力、dbl-click=既定。
契約: 4px 以上動けば scrub、動かなければ入力/ dbl-click = 既定(入力欄は開かない)。
隠す物: label の役を分けない。 手数: 同上。
レンズ: ▲L11(面が小さくて良い)▼L8(dbl-click と入力の衝突を避ける設計が要る)。 出所: Ableton / Blender。
試作: `ScrubField` の click 遅延(dbl-click 判定)の挙動を手で確かめる。

**案 I2-c「ladder: 掴んだまま上下で細かさが変わる」** 機構: drag 中に pointer を上下すると x10 / x1 / x0.1 / x0.01 の段が切替わり、段が小さく出る。
契約: 段は掴んでいる間だけ見える/ 段を変えても値は保たれる/ 修飾キーと矛盾しない(Shift/Alt は段を 1 つ動かす)。
隠す物: 段表示は drag 中だけ。 手数: 0 追加(掴んだ手で足りる)。
レンズ: ▲L10 L12 ▼L9(発見されにくい)。 出所: Figma 風(IN C6)。トラックパッドでは使いにくい。
試作: `ScrubField` に縦の段、drag 中の小さな表示。

**案 I2-d「数値の脇に ±」** [新(連続 scrub と併用)] 機構: 数値の両脇に隠れた「-1 / +1」。hover で現れ、押しっぱなしで加速。
契約: 1 クリック=±1 単位、Shift で x10、押しっぱなしで連続。
隠す物: hover で現れる。 手数: 1 クリック。
レンズ: ▲L9 L10 ▼L11(行が太る)。 出所: 数値 stepper(一般的)。
試作: `ValueWell` の hover で ± を出す。

---

## I3. 空間値(位置・scale・回転・anchor)を手触りで

(利用者の既定: GUI が主、数値は 4 番目。ここは「主従の配置」と「Stage との役割分け」の違い。)

**案 I3-a「Stage の gizmo が主、Inspector は数値行のみ」** 機構: 手触りは Stage の box で、Inspector は 1 行の数値。pad は持たない。
契約: Stage で動かすと Inspector の数値が即時に変わる/ 数値を打つと Stage の box が動く/ Esc で drag を元に戻す。
隠す物: pad・dial 無し。 手数: Stage で drag 1。
レンズ: ▲L11 L2 ▼L9(Stage が見えていない時に使えない)。 出所: AE/Figma/Blender。
試作: `workflow_set` W2(gizmo と `Doc` を結ぶ)。

**案 I3-b「Inspector に小さな instrument(pad+dial)、Stage と同じ値を双方向」** 機構: panel 内に X/Y pad、回転 dial、scale handle。数値は下に細く。
契約: pad の drag = 数値と Stage が同時に動く/ Shift で細かく/ Esc で戻す/ 同じ値を別の GUI が重複して持たない(1 つの `Doc`)。
隠す物: 3D の Z・X/Y 回転は 3D switch で出る。 手数: pad 1 drag。
レンズ: ▲L12 L9 ▼L11(panel の高さを食う)。 出所: 利用者の既定(pad+dial+handles)、Ableton の XY pad。
試作: `inspector-gui` の Hierarchy options の 1 つ。`Doc` に結ぶ。

**案 I3-c「数値行を開くと instrument が展開(必要な軸だけ)」** 機構: 通常は数値 1 行。Position の行をクリックで展開すると pad が出る。
契約: 展開は 1 行ずつ(accordion)/ 展開中も数値は同じ位置/ 展開状態は layer をまたいで保つ。
隠す物: 通常は数値行のみ。 手数: 展開 1 + drag 1。
レンズ: ▲L11 L9 ▼L10。 出所: Blender の accordion(IS B11)、AE の twirl。
試作: `inspector-gui` の展開行。

**案 I3-d「掴むと Stage に手が出る」** [新] 機構: Inspector の Position 行の label を掴むと、Stage 上の対象に pad/handle が一時的に現れ、同じ drag を Stage に延長できる。離すと消える。
契約: 掴んでいる間だけ Stage に手が出る/ Esc で戻す/ 1 drag=1 undo。
隠す物: panel 内に pad を持たない。 手数: 掴む 1。
レンズ: ▲L12 L11 ▼L9(発見されにくい)。 出所: 出荷物に無し(Procreate の「slider 無しで画面上を scrub」IS B14 に近い発想)。
試作: label の drag で `stage_set` に一時 gizmo を表示。

---

## I4. 「時間で変わる」を値の所で言う(key の状態)

共通の下敷き(決定、D-I4): 状態は **形(輪郭/半/塗り)+ 位置の言葉**で出し、色の濃淡だけに頼らない。diamond は当たり判定を見た目より大きく取る。記号は増やさない(既存の key の記号の塗りの違いだけ)。

**案 I4-a「各行の key 標」(stopwatch 型)** 機構: 値の行ごとに小さな key の標。クリックで現在時刻に key。
契約: 標は 3 状態(動かない/動く・今は key 無し/今 key 有り)が形で見分けられる/ クリック = 現在時刻の key の付け外し。
隠す物: 無し。 手数: 1。
レンズ: ▲L8 L9 ▼L11。 出所: AE stopwatch、Resolve diamond(IS B10/B12)。
試作: `inspector_set` の key mark の 3 状態を形で出す。

**案 I4-b「Animate スイッチ(auto-key)+ 変えた値が自動で key」** 機構: header のスイッチが ON の間、値を変えると key になる。OFF の間は変えても key にならない。
契約: スイッチの状態が語で見える/ ON 中に未 key の値を変えると最初の key ができる/ 今の編集が key になるかを、値を掴む前に行の標で示す。
隠す物: 行ごとの標は変えるまで静か。 手数: 0(変えるだけ)。
レンズ: ▲L10 L12 ▼L8(意図しない key)。 出所: AE auto-key、Cavalry(IN D2)。
試作: `Doc._touch` と Animate スイッチ。

**案 I4-c「その場に時間の小窓(mini lane)」** [新] 機構: 値の行の右に、その property の key を並べた短い時間軸(playhead 付き)が出る。そこで key を付け外し・動かす。
契約: 小窓の playhead は Timeline と同期/ 小窓で key を動かすと Timeline の key も動く。
隠す物: 小窓は animated な行だけ出る。 手数: 1。
レンズ: ▲L12 L8 ▼L11。 出所: 出荷物に無し。Timeline の lane(TL D1-D6)を Inspector へ小さくした形。
試作: 行の脇に `timeline_parts` の lane を縮めて埋める。

**案 I4-d「key 一覧に飛ぶ」** 機構: 行の標をクリックで、その property の次の key へ playhead が飛ぶ。Shift で前の key へ。
契約: 標のクリック=次の key / 付け外しは別の gesture(長押し/右クリック)。
隠す物: 無し。 手数: 1。
レンズ: ▲L10 L12 ▼L9。 出所: AE の key navigator、Blender の ↑↓(P10)。
試作: `Doc.keys` を前後に巡る。

---

## I5. 二つの key の間の ease を、graph editor 無しで決める

共通の下敷き(決定、D-I5): preset と handle が共存、y は範囲外 OK(overshoot)、x は 0..1、Shift=軸固定。preset の hover は curve を plot の中で見せ(観察だけ)、矢印/キーで進めると Stage の動きに重ねる(Hot-Swap と同じ作法、options-browser.md D-1)。

**案 I5-a「名前付き preset の行 + handle(AE/Figma 型)」** 機構: 名前が付いた preset 一覧(Linear・Ease Out・Back…)+ 選ぶと handle で微調整できる。
契約: preset の行に名前が出る/ クリック=その ease を 2 key 間へ書く/ handle drag は release で 1 手。
隠す物: handle は選択中の curve だけ。 手数: 1 クリック。
レンズ: ▲L9 L12 L10 ▼L11。 出所: Figma の easing menu、AE Easy Ease(P9)。
試作: Curve picker+`EaseThumb`+mini curve editor。

**案 I5-b「方向だけ決めて、強さは 1 本のつまみ」** 機構: 「入る/出る/両方」の 3 択と、強さ 1 つ(influence)。形は 1 つの滑らかな族。
契約: 3 択と強さ 1 本で curve が決まる/ 数値は 2 つ。
隠す物: handle・family 無し。 手数: 3 択 1 + つまみ 1。
レンズ: ▲L9 L10 ▼L12(表現の幅)。 出所: AE の Easy Ease + influence を単純化(P9)。
試作: Segmented 3 つ + 1 本の slider で curve を描く。

**案 I5-c「動きの辞書から選ぶ(Bounce・Overshoot・Spring…)」** [新(名前の付け方)] 機構: 数学の名前でなく、動きの言葉(ふわっ・ぴたっ・ばね)の preset を、動く小さな tile で選ぶ。
契約: tile は実際の動きで 1 周 1 秒/ 選ぶと 2 key 間へ書く。
隠す物: 数学名は小さい補足に。 手数: 1。
レンズ: ▲L9 L12 L2 ▼L1(言葉の語彙の設計が要る)。 出所: Figma の spring(Gentle/Bouncy)、Ease and Wizz の名前(IE §9)。
試作: Curve picker の tile を動くものに替える(`kFast` ではなく 1 周)。

**案 I5-d「ease を 1 回測って、他の key にコピー(貼る)」** 機構: ある 2 key 間の ease を copy し、別の 2 key 間(複数選択可)に paste。
契約: copy は ease だけ(値・時間は持たない)/ paste は選択中の区間全てへ 1 手。
隠す物: メニュー内。 手数: 2。
レンズ: ▲L5 L7 ▼L10。 出所: Mt. Mograph Copy/Paste Ease(IE §6, §9)。I11 の一種。
試作: key 2 組の mock。copy/paste の 2 ボタンと一括 paste。

---

## I6. 既定から変えた物が分かり、一手で戻せる

**案 I6-a「label の濃さで差を言う、label を dbl-click で戻す」** 機構: 既定のままの label は薄く、変えた物は通常の濃さ。label の dbl-click で既定へ。
契約: 変えた=濃/ 既定=薄/ dbl-click = その行だけ既定へ(key があれば現在時刻の key だけ既定値に)。
隠す物: 専用の印が無い。 手数: 1(dbl)。
レンズ: ▲L8 L11 ▼L9(「薄い=無効」と誤読)。 出所: Houdini の label 色(IS B14)、Unreal の modified 印の別形。
試作: `inspector_set` の label トーンを `Doc` の差分で切替。

**案 I6-b「差分だけ見るフィルタ」** 機構: 上端の 1 つのつまみで「変えた物だけ」に絞る(U 型)。
契約: つまみ ON で既定のままの行が畳まれる/ 件数を言葉で出す(「変更 7」)。
隠す物: つまみは 1 つ。 手数: 1。
レンズ: ▲L8 L11 L7 ▼L10。 出所: Unreal Show Only Modified、AE の UU(IS B10, B13)。
試作: `Doc` の差分と PillSwitch。

**案 I6-c「group の header に『全部戻す』」** 機構: group(Transform、各 effect)の header に「reset」。押しても key や relation は消さない(値だけ既定)。
契約: group reset = 値を既定に/ key と relation は残る/ 1 undo。
隠す物: header の中。 手数: 1。
レンズ: ▲L7 L8 ▼L10。 出所: Cavalry Reset all Attributes(IN 先例)。
試作: card header に reset を足す。

**案 I6-d「戻す先を選べる(既定 / 前回確定 / snapshot)」** [新] 機構: 戻す操作が「既定」だけでなく「この layer の直前の状態」「保存した snapshot」を選べる。
契約: reset の先を 1 語で選ぶ/ 戻しても履歴に 1 手。
隠す物: 通常は既定へ。 手数: 1(+選ぶ)。
レンズ: ▲L8 L5 ▼L9。 出所: Unity の override apply/revert(IS B13)、Mt. Mograph Snapshot(IE §12)の組合せ。
試作: 戻る先の 3 択メニュー。

---

## I7. 長い parameter 一覧から、触る物へ速く着く

**案 I7-a「重要を前に、残りは Advanced に畳む」** 機構: 宣言された主要 parameter + 「Advanced n」の折畳。
契約: 折畳に件数/ 検索は全部を平らにする/ 折畳状態は layer をまたいで保つ。
隠す物: 長い尾。 手数: 0(普段)。
レンズ: ▲L9 L11 ▼L1。 出所: Unreal/Blender/Cavalry(IN F2)。
試作: `inspector_set` に Advanced fold を足す。

**案 I7-b「欄で打って絞る」** 機構: panel 上端の欄に打つと、一致する行だけが並ぶ(group は平ら)。
契約: 1 文字ごとに更新/ 一致無しは「ここに無い」と、全体へ戻る 1 手を出す。
隠す物: 欄は常設 1 行。 手数: 打つ。
レンズ: ▲L1 L10 ▼L9。 出所: Unreal の property 検索(IS B13)。
試作: `TextBox` と行の filter。

**案 I7-c「よく触る行を上に星で止める(自分の主要)」** 機構: 行に星を付けると、その effect の上位に固定される(利用者の hero)。
契約: 星は effect 種ごとに記憶/ 星の無い行は変わらない。
隠す物: 星の標は hover/選択時のみ。 手数: 1。
レンズ: ▲L10 L9 ▼L5。 出所: Unreal の favorites(IS B13)。
試作: 行の星と上位固定。

**案 I7-d「触った順に並ぶ(最近触った行が上へ)」** [新] 機構: その layer で最近触った行ほど上に並ぶ。
契約: 並びは触った順(最近が上)/ 固定にもできる/ 位置は動かさない選択もある。
隠す物: 並び替えを意識させない。 手数: 0。
レンズ: ▲L10 L11 ▼L9(位置が変わる=記憶に反する)。 出所: 出荷物に無し(frecency の発想を property に)。
試作: `Doc` に touch 履歴を持たせ並びを切替。

---

## I9. 値が別の値に従う・動かす関係を、見て作って外す

(Relation は操作。入口は Inspector の Property → Relations パネル。Browser の棚にはしない。)

**案 I9-a「掴んで結ぶ(pick-whip/connection)」** 機構: 値の行の標から別の行/layer へ線を引く。種類に合う相手だけが残り、合わない物は dim。
契約: 掴んだ間、合う相手が強調され合わない物は動かない/ 離した所が相手/ 結んだ値の行に「← 相手」が出る。
隠す物: 無し。 手数: drag 1。
レンズ: ▲L6 L10 L12 ▼L11(遠い相手)。 出所: Cavalry Connection、AE pick-whip(IS B14)。
試作: `relation_set` の ConnectionDemo を Inspector の行から。

**案 I9-b「lasso で一括(1 対 N)」** 機構: 値の行から Relations 面を開き、Stage 上の点を lasso で囲んで一括。範囲を決める。
契約: lasso で選んだ全員に 1 手で結ぶ/ 範囲の両端を scrub、現在値から「ここを最小/最大」を入れる/ 1 undo。
隠す物: 通常は badge だけ。 手数: 3-4。
レンズ: ▲L7 L6 L12 ▼L10。 出所: 過去の relations-v0 の仕組み(問題として:「N に同じ関係を一度で」)。
試作: `relation_set` の既存 card+lasso。

**案 I9-c「見える取っ手を足す(macro/controller 型)」** 機構: 「つまみ(slider/点)」を 1 つ作り、複数の値をそれに結ぶ。各値に範囲を持たせる。
契約: つまみを動かすと結んだ全値が各自の範囲で動く/ つまみの行が値の持ち主。
隠す物: 通常は結んだ値の行に badge。 手数: 作る 1 + 結ぶ n。
レンズ: ▲L6 L7 L12 ▼L10。 出所: Ableton Macro/Map、AE Expression Controls(IS B14, IE §7)。
試作: 1 つの slider と 3 つの `ValueWell` を結ぶ。

**案 I9-d「関係の面を行の下に一行で」** [新] 機構: 値の行をクリックして展開すると、その行の「従う/動かす」が 1 行の関係式(源 → 範囲 → 先)として出て、そこで直接 scrub できる。
契約: 展開は 1 行ずつ/ 式の端の値をそのまま scrub/ 外すは式の右端の語。
隠す物: 展開前は badge。 手数: 展開 1 + scrub。
レンズ: ▲L6 L8 L10 ▼L11。 出所: 出荷物に無し(Houdini の channel reference を行内に展開する発想)。
試作: 行内展開に源/範囲/先を並べる。

---

## I11. 「一側面だけ」を他へ運ぶ

**案 I11-a「側面を選んで copy/paste(clipboard)」** 機構: 値・ease・色・key・effect 群のいずれかを copy → 対象を選び paste。
契約: copy の面は 1 つ(ease だけ等)/ paste は選択中の全 layer へ 1 手/ 何を持っているかが見える。
隠す物: clipboard の中身が常に 1 語で見える(「ease を持っている」)。 手数: 2。
レンズ: ▲L5 L7 ▼L10。 出所: Mt. Mograph Copy/Paste Ease/Color(IE §6)。
試作: 小さな clipboard 表示+ paste 先の複数選択。

**案 I11-b「copy with link(同じ物を共有)」** 機構: 貼った物が元と繋がり、片方を直すと両方が変わる。
契約: link した物には「共有」の語/ unlink で独立/ 元を消すと貼った方は値が残る。
隠す物: 通常は静か。 手数: 2。
レンズ: ▲L5 L6 ▼L8(誤解)。 出所: AE Copy with Property Links(IE §6)、Ray Dynamic Color。
試作: 2 つの card が同じ値を共有。

**案 I11-c「スポイトで拾って塗る」** [新] 機構: 行や effect card から「スポイト」で他の layer の同名行を拾って、自分に当てる。
契約: スポイト中の対象の値を hover で見せる/ 離すと確定。
隠す物: 無し。 手数: 2。
レンズ: ▲L5 L10 L12 ▼L7。 出所: 出荷物に無し(Figma のスポイトを property 全般へ)。
試作: Stage 上の layer を hover で値を見せて、選ぶと反映。

**案 I11-d「Browser の『自分』棚へ側面ごと保存」** 機構: ease/色/effect 群を B11-a と同じ drag で棚に保存し、他 layer へ drop で当てる。
契約: 側面ごとに別の棚/ drop 先の layer の同名 property にだけ書く。
隠す物: 保存ボタン不要。 手数: drag 1。
レンズ: ▲L5 L4 ▼L10。 出所: Flow の curve library、Layer Library(IE §4,§9)。
試作: B11-a と共通。

---

## I8. effect の積み

**案 I8-a「card の積み(fx 型)」** 機構: effect は card。header で on/off、折畳、並替(drag)、menu。
契約: header の drag 領域と fold の領域を分ける(誤爆しない)/ bypass は濃淡+語/ 順序が見える。
隠す物: menu 内。 手数: 1。
レンズ: ▲L9 L8 ▼L11。 出所: AE Effect Controls(IN F1)。
試作: `inspector_set` の card を drag 並替。

**案 I8-b「flow(左→右)の鎖」** [新] 機構: effect を 1 行の鎖(左から右へ)として見せ、順序が一目で分かる。選んだ物だけ下に展開。
契約: 鎖は順序を示す/ 展開は 1 つずつ/ bypass は鎖の上で 1 クリック。
隠す物: parameter は展開時のみ。 手数: 1。
レンズ: ▲L11 L8 ▼L10。 出所: TouchDesigner/Houdini のネットワークを 1 行化(IS A6)。
試作: card を横に並べ、選んだ物の下に展開。

**案 I8-c「Browser の候補を hot-swap(B9)」** 機構: card を選び Q で Browser に結び、↑↓ で差替。
契約: B9-a と同じ。
レンズ: ▲L3 L2 ▼L10。 出所: Ableton Hot-Swap。
試作: B9-a と共通。

**案 I8-d「まとめて bypass(solo)」** 機構: 1 つの effect を solo にすると他が一時的に無効、元の on/off は覚えている。
契約: solo の間だけ他が淡く無効/ solo を外すと元に戻る(on/off は変わらない)。
隠す物: solo の標は hover/選択時のみ。 手数: 1。
レンズ: ▲L8 L2 ▼L10。 出所: Ableton/DAW の solo、AE の Solo。
試作: card に solo、`Doc.effects` の有効フラグを一時に。

---

## I12. 多数に同じ変更、ずらし

**案 I12-a「複数選択で、drag は相対・入力は絶対」(Figma/Cavalry 型)** 機構: 複数選択中の scrub は各自の値に同じ差を足し、入力は全員を同じ値にする。違う値は「混在」と言う。
契約: 混在の値は専用の語(「混在」)/ scrub = 相対/ 入力 = 絶対/ locked の物は変えない(host が決める)。
隠す物: 無し。 手数: 0 追加。
レンズ: ▲L7 L8 ▼L9。 出所: Figma "Mixed"、Blender/Cavalry Alt(IS B11, B12)。
試作: `Doc` の `kind=several`(既存)。

**案 I12-b「ずらし(Stagger)を値の脇のつまみで」** [新] 機構: 複数選択中、値の行に「ずらし」のつまみが出て、選択順に値/時間を等差でずらす。
契約: つまみ = 隣との差/ 順序の基準は Stage か Timeline の並びを選べる/ 1 undo。
隠す物: 単独選択では出ない。 手数: scrub 1。
レンズ: ▲L7 L12 ▼L10(順序の基準)。 出所: Mt. Mograph Delay/Stagger/Falloff(IE §8)。
試作: 3 つの `ValueWell` の等差 scrub。

**案 I12-c「グラフに点を描いてずらす(Falloff 型)」** [新] 機構: 選択した物の値を x(並び)・y(値)の小さな図に点として出し、図で線を引いて分布を決める。
契約: 図の点を触ると該当 layer が動く/ 図に線(傾き・カーブ)を引くと全員に分布が入る。
隠す物: 図は複数選択時のみ。 手数: 1。
レンズ: ▲L7 L12 L11 ▼L9。 出所: Mt. Mograph Falloff、Cavalry Behaviours(IE §8, §10)。
試作: 6 点の散布 + 線で値を書き戻す。

**案 I12-d「同じ property を全部集めて 1 行に(Grab 型)」** 機構: 選択した全 layer の同じ property を 1 つの作業行に集める(1 つの値の群)。
契約: 集めた行の編集は全員へ/ 個別の値は展開で見える。
隠す物: 個別の行。 手数: 1。
レンズ: ▲L7 L11 ▼L8。 出所: Mt. Mograph Grab(IE §8)。
試作: `Doc.several` の展開/畳み。

---

## 判断(先例・ワークフロー・既決の方向から自分で決めた。理由つき)

| # | 問い | 決めた事 | 理由 |
|---|---|---|---|
| D-I1 | 複数選択の値(I1/I12-a) | **scrub は各自へ相対、入力は全員へ絶対、違う値は「混在」。locked の物は動かさない。意味は host が持つ** | Figma・Blender・Cavalry が同じ向き(IS B11-B12)で、実制作で「30 個を一緒に動かす」に素直。入力を相対にすると、揃えたい時に困る |
| D-I2 | parent/参照の選び方(I14) | **検索付きの一覧+ Stage 上で拾う(pick)の両方。prev/next 巡回は採らない** | AE の pick-whip+一覧が数百 layer でも使える。巡回は数十を超えると詰む |
| D-I3 | 数値の規則(X4/I2) | **右 drag で増える。1 目盛り=1 表示単位で固定(値の大きさに比例しない)。Shift で x10、Alt(Option)で x0.1。矢印・wheel も同じ規則。wheel は押さずに効く。dbl-click(または label dbl)は既定へ、入力欄は開かない** | AE と Figma が Shift=大きく(P10 の「3+ 道具が一致」)。Alt=細かいは AE 系(Ctrl は Mac で別意味)。Ableton/Blender は Shift=細かいが、**Motolii 内で一つに揃える(基準 4)**ことを最優先し、段数が多い機能(arrow・wheel・drag)で全て同じ。過去の不満はこの規則が 2 つに割れた事で、割れた所が不便だった |
| D-I4 | key の状態表示 | **形(輪郭/半/塗り)+ 当たり判定を大きく。色だけに頼らない** | 見た目の細い diamond は掴めない、色だけは暗所で区別できない、という使い心地の不便が過去から分かっている |
| D-I5 | ease の試す作法 | **preset hover = plot の中で curve を見せるだけ。矢印/キーで進める間は Stage の動きに重ねる。click/Enter で 1 手書く。handle は release で 1 手。Esc は drag/重ねを元に戻す。overshoot は y のみ許す(x 0..1)** | 候補を連続して見る速さ(options-browser.md D-1)。AE/Figma/Blender と整合(P9) |
| D-I6 | 既定との差の表示場所 | **1 箇所にする: label の濃さ(I6-a)を標準とし、「変えた物だけ」フィルタ(I6-b)を足す。reset の入口は dbl-click と、focus 中の Backspace** | 点・矢印・diamond・badge が 1 行に重なった反省(IO I4)。位置・灰・語で出す(DESIGN §4 記号は少なく) |
| D-I7 | 長い一覧 | **主要を前に+検索(I7-a+b)を標準。星(I7-c)は試作で比べる。並びが勝手に動く案(I7-d)は採らない** | 触る位置が毎回変わると手が覚えられない(基準 1) |
| D-I8 | effect card の操作 | **header は fold のみ、並替は専用の取っ手(drag grip)に分ける** | header が fold と drag を兼ねて誤爆した不便が分かっている |
| D-I9 | relation の入口 | **I9-a(掴んで結ぶ)を標準、1 対 N は I9-b の lasso。見える取っ手(I9-c)は後続** | Cavalry/Ableton の先例が強く、制作での「1 個の値で多数を動かす」に直結。Relation は Browser に置かない既決を維持 |

## 本当に同じ価値で、利用者の事実か判断が要るもの
| # | 問い | 理由 |
|---|---|---|
| OR-I1 | 空間の手触りを panel 内の instrument(I3-b)に持つか、Stage の gizmo に任せる(I3-a/d)か | どちらも妥当。決め手は「利用者が Stage を常に見ながら作るか」「Inspector を単独で使う場面があるか」という作業様式の事実。**既に Widgetbook の Hierarchy options で比べている**ので、そこで利用者の手で決まる |
| OR-I2 | Animate(auto-key)の初期値を ON にするか OFF にするか | AE は OFF(意図しない key を避ける)、他は ON。作り方(最初は絵を作る人か、最初から動かす人か)の事実で決まる |
