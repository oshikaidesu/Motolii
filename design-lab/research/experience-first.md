# Experience-first view: 代表的な制作の流れ・手癖を土台にした追加・完成条件・試作

2026-10-02(改訂 2 版)。DESIGN.md §0: 目的は「考えた絵を、探す・操作で止まらずに最短で作品にする体験を Widgetbook の実操作で見つけ、証明する」事。
**方針(利用者の裁定)**: 利用者の手癖(AE/Ableton/Blender/Figma の筋肉記憶)を最優先する。Browser / Inspector / Stage / Timeline の分割は原則として壊さない(壊すには実操作での強い証拠が要る)。改善は **手癖の上への「追加」**(ショートカット、手癖の近道、求めた時だけ出る補助)で入れる。出荷された道具と同じ drag/click の意味を既定にする。
判断の順: 利用者の便利さ → 実制作 → 先例 → Motolii 内の一貫。実装コストでは決めない。
(前の版の「境界を壊す案 H1-H12」は取り下げ。分割を壊す事だけが取り柄の案は落とし、残す案は「手癖への追加」として書き直した。)

## A. 代表的な制作の流れ(10)

各流れ = **手癖の基準線**(AE 等のユーザーが今そのままやる動き。これが Motolii の既定)+ **追加**(求めた時だけ出る・手癖を壊さない)。手数は 1 動作=1 の見込みで、試作で実測する。

### E1 文字に look を付けて出す
- 手癖の基準: Browser で font/effect を探す(Ableton 型の検索欄と絞り)→ drag か dbl-click で置く/付ける → Inspector で値を触る(AE の P/S/R/T と label scrub)。
- 追加: ① Cmd-K / Tab のコマンド検索で Browser を開かずに effect を付ける(Houdini Tab/VS Code palette の手癖)。② 付けた直後、その effect の 2-3 個の主要値が Inspector の先頭に来る(AE の Effect Controls の先頭展開と同じ場所)。③ 候補は選んだ layer の自分のコマで試写(B5-c)。
- 得る物: 面を移動する手数の減少。失う手癖: 無し(基準線は全て残る)。

### E2 既存 layer の look を選ぶ
- 手癖の基準: Browser の tile を dbl-click で付け、気に入らなければ Cmd-Z。Ableton の Hot-Swap(Q)が音の世界での手癖。
- 追加: Q(Ableton と同じ)で Browser を layer に結び、↑↓ で着せ替え、Enter 確定、Esc で解除(B9-a)。試着は確定まで履歴に残らない。
- 得る物: 20 候補を連続で見る速さ。

### E3 多数を動かす(Scatter→広げる→key→再生)
- 手癖の基準: Inspector の Property → Relations(既決の入口)→ gadget の handle → stopwatch で key → Space で再生(AE の流儀)。
- 追加: gadget の handle を掴んで広げている間、値が Stage に live で出る。key を打つ標(stopwatch)は gadget の脇にも出る(同じ stopwatch)。
- 得る物: 広げた量を数値を見ずに決められる(C4)。

### E4 動きの癖(ease)を決める
- 手癖の基準: 2 key を選ぶ → Ease の面(preset の行+ handle)。AE の F9(Easy Ease)に相当する 1 キー。
- 追加: preset に hover=曲線を plot に出す(観察だけ)/ 矢印で進める間は Stage の動きに重ねる / Esc で戻る。
- 得る物: 選ぶ速さ。手癖(preset 先 → handle で微調整)は不変。

### E5 1 つの値で多数を動かす(連動)
- 手癖の基準: AE の pick-whip と同じ意味の「掴んで結ぶ」(I9-a)。値の行の標から相手へ drag。1 対 N は Relations の lasso(既決)。
- 追加: 掴んでいる間、型の合う相手だけが強調される(Cavalry の dim)。
- 得る物: 間違った相手に結ばない安心(C9)。

### E6 自分の look を他へ運ぶ
- 手癖の基準: effect を copy → 他の layer を選ぶ → paste(AE/Blender の Ctrl-C/V)。
- 追加: 「ease だけ」「色だけ」「effect 群だけ」を貼る側面コピー(Mt. Mograph の発想)。drag で Browser の「自分」に落とすと保存(Blender の Mark as Asset の手癖)。
- 得る物: 丸ごとコピーしか無かった所に部分コピーが入る(C7)。

### E7 大きな案件(layer 200・素材 3000・effect 800)で探して直す
- 手癖の基準: Browser の検索+絞り、Timeline の検索/ solo / shy、Inspector の折畳(AE/Ableton の手癖)。
- 追加: 「変えた物だけ」「動く物だけ」の絞りを、求めた時に Timeline と Inspector へ同時に効かせる(AE の U/UU の手癖の延長。Mt. Mograph Focus Group 型)。Cmd-K で property 名を打って飛ぶ。
- 得る物: 規模での探す速さ(C1, C6, C8)。

### E8 素材を取り込み、差し替える
- 手癖の基準: OS/Browser から drag で Stage/Timeline へ。layer 上は **既定は追加、Alt で置換**(AE の手癖)。落とす前に outline で結果が見える。
- 追加: 欠損の印・Locate、フォルダを生きた入力に(Cavalry)。
- 得る物: 取り違えない(C9)。

### E9 「なぜ動く/何を変えたか」を調べる(状態・差分・relation の可視)
- 手癖の基準: Timeline の twirl、U/UU(動く物/変えた物だけ)、Inspector の stopwatch・ label の状態。
- 追加: 求めた時だけ(キー押下中・欄のつまみ)、選んだ layer の「動く理由(key・relation・親)」と「既定との差」を 1 つに並べる一覧が出る。標準の面は変えない。
- 得る物: 見落としの無さ(C6)。

### E10 キーボードだけで走る
- 手癖の基準: AE の J/K・P/S/R/T/A、Space、Home/End(P10 の 3 道具一致の最小集合を既定に)。
- 追加: Cmd-K 1 本の欄に、検索・適用・property への移動・値入力(`opacity 50`)を受ける第二段(Raycast の手癖)。
- 得る物: 熟練の速さ(C10)。

## B. 完成条件 10 と、実操作で見える受入

(DESIGN.md §0 の条件。静止画でなく手で走らせて Y/N。)

| 条件 | 見える受入 |
|---|---|
| C1 速く見つかる | 名前を知らない effect を 10 秒以内に選べるか(E1/E2)。3000 素材でも同じか(E7) |
| C2 置く前後に試せる | 20 候補を連続で試して履歴が 1 行も増えないか。Esc 1 回で試す前に戻るか。確定が 1 手の undo か |
| C3 次の操作へ速く届く | 付けた直後・広げた直後の「次にやる事」へ、面を探さず 2 手以内で届くか |
| C4 直接触れる | 位置・散らし・ease・色を数値を見ずに決められるか。数値が要る時だけ数値行を使うか |
| C5 多数をまとめて | 30 layer への同じ変更とずらしが各 3 手以内か。違う値が「混在」と言えているか |
| C6 隠れない | 選んだ layer で「動く理由・結ばれた相手・既定との差・locked」が、追加操作 1 回以内に全て見えるか。見落とす値が 0 か |
| C7 一部を運ぶ・使い回す | ease だけ/色だけ/effect 群だけを他 3 layer へ 1 動作で運べるか。1 動作で棚に保存できるか |
| C8 規模で保つ | layer 200・素材 3000・effect 800 の mock で、検索・絞り・複数選択・scroll が手の速さで返るか |
| C9 予測でき戻すのが怖くない | 「確定していない」が語で見えるか。Cmd-Z 1 回で試す前に戻るか。複数 layer の変更が 1 手で戻るか |
| C10 初見で分かり、慣れると速い | 説明なしで E1 を完走できるか。3 回目に手数と時間が減るか。キーボードだけで走れるか |
| **手癖(全走行に共通)** | AE/Ableton/Blender/Figma に慣れた人が、**追加機能を知らなくても**、その道具と同じ gesture で走行を完走できるか。同じ gesture の意味が棚や面で割れていないか |

走行の記録は手書きで足りる: 手数・秒・面の移動回数・迷った箇所・誤操作。基準線(手癖のみ)と「基準線+追加」を同じ人が同じ課題で走り、追加が手数と迷いを減らし、**基準線を邪魔していない事**を確かめる。

## C. 手癖の台帳(各案の置き場)

options-browser.md / options-inspector.md の案を 4 種に分ける。本文の案名に `[習慣]` `[追加]` `[外れる]` と読み替える。

### C.1 手癖のまま(既定にする)
B1 置き場+Ableton 型の絞り(B2-a)/ B3-a 常設検索 / B5-b hover scrub(Resolve・Premiere)/ B5-d preview 面 / B7-a 単=選ぶ・dbl/Enter=使う・drag=置く / B8-a 追加+Alt 置換(AE)/ B9-a Q の Hot-Swap(Ableton)/ B9-b 効果 card への drop 置換(Alt 追加)/ B10-a 数字キーで collection(色は使わず名前で)/ B10-b 星 / B11-a 棚へ drop(Blender)/ B12-a list↔thumb / I2-a・b 数値の drag・入力・既定(規則は一つ)/ I3-a Stage の gizmo+数値行 / I4-a 行ごとの stopwatch / I4-b Animate / I4-d key へ飛ぶ / I5-a preset+handle / I6-b 差分だけ見る(AE の U/UU)/ I6-c group reset(Cavalry)/ I7-a Advanced 折畳 / I7-b 検索 / I8-a card の積み / I8-d solo / I9-a pick-whip / I11-a 側面 copy・paste / I11-b link コピー(AE)/ I12-a 複数選択で相対 drag・絶対入力(Figma・Cavalry)。

### C.2 手癖の上への追加(求めた時だけ出る・既定の動きは変えない)
B3-b Tab/Cmd-K の検索(Houdini・palette)/ B3-c 先頭 1 文字で領域切替 / B3-d 空入力の並び(最近→選択に効く)/ B2-b 絞りの chip / B4 結果帯と Clear(Ableton)/ B5-c 自分のコマでの試写(Q・矢印の間だけ)/ B7-d Browser 上端の対象表示 / B9-c card 上の ↑↓ / B9-d 候補と今の A/B(求めた時)/ B10-c 使った物の積み / B11-c 履歴の snapshot / B12-b 似た物 / B12-d 生きたフォルダ / B13-a・b・c 選択で絞る・理由つき淡色・後から対象を選ぶ / I2-c ladder / I2-d ± / I3-b 小さな instrument(利用者の既決)/ I3-c 展開で出す / I5-b 強さ 1 つ / I5-c 動きの言葉で選ぶ / I5-d ease の copy / I6-d 戻す先を選ぶ / I7-c 星で止める / I9-b lasso(既決)/ I9-c macro 取っ手 / I11-c スポイト / I11-d 側面の棚 / I12-b ずらし / I12-c Falloff 図 / I12-d Grab / E9 の「動く理由の一覧」/ E7 の全面の同時絞り(U/UU の延長)。

### C.3 手癖から外れる案(印を明記。外す手癖と得る物)
| 案 | 外す手癖 | 得る物 |
|---|---|---|
| B2-c 絞りを文章で言う見出し | Ableton の結果帯(件数+Clear) | 一目で何に絞られているかが読める |
| B5-a tile が常に動く | 静止画の thumbnail(Ableton・Resolve) | 見た瞬間に動きが分かる |
| B7-c 単クリック=使う | Ableton・Premiere の「単=選ぶ」 | 1 手少ない(誤爆の恐れ。D-3 で不採用) |
| B8-b layer 上は置換が既定 | AE の「追加が既定」 | Ableton 風で置換が速い(D-2 で不採用) |
| B8-c layer 上へは落とせない | AE/Premiere の layer 上 drop | 取り違えが起きない |
| B12-c 地図を 1 軸の並びに | 過去の Explore 地図 | 規模での実用 |
| I2-a の label 掴み scrub と I2-b の値掴みの併存 | AE の値/label どちらも scrub | 無し(併存でよい) |
| I6-a label の濃さで差を言う | AE の無印(U 絞りで見る) | 一目で差が分かる(追加でなく既定の見た目を変える) |
| I7-d 触った順に並ぶ | 位置固定の並び(全道具) | 手が覚えられなくなる。不採用 |

### C.4 取り下げ(境界を壊す事だけが取り柄だった案)
B13-c の Stage 直選択は残すが、次は取り下げ: I3-d「掴むと Stage に手が出る」、I4-c「その場の mini lane」、I8-b「effect の flow 鎖」、前版の H1(Stage 上で打つ)・H2(+スロット)・H4(bar 上の試着)・H5(Stage に relation を描く)・H7(obj→obj の線)・H10(Browser を経由せず落とす)・H11 の常時 overlay・H12 の「全面を置換するコマンド線」。
残した H3 は B9-a(Q)、H6 は I5-a(Ease 面)、H9 は E7 の追加(U/UU の延長)、H12 の核は Cmd-K の第二段(E10)に吸収。

## D. 次の試作: 流れ単位(1 走行を最初から最後まで、Widgetbook だけで)

各試作は、**まず手癖の基準線だけで走れる**(合格条件の第一)。次に追加を入れ、同じ人・同じ課題で手数と迷いを比べる。部品の出来では合否を付けない。

| 順 | 走行 | 開始 → 終了 | 基準線 | 入れる追加 | 条件 |
|---|---|---|---|---|---|
| R1 | タイトルを作って出す | 空の Stage → 文字+look+イン・アニメ再生(E1+E4) | Browser 検索→drag→Inspector→Space | Cmd-K 付け・主要値先頭・自分のコマ試写・ease の hover | C1 C2 C3 C4 C10 手癖 |
| R2 | 着せ替え選び | 5 layer の look を各 10 候補から選ぶ(E2) | tile を dbl-click→Cmd-Z | Q+↑↓ の Hot-Swap | C1 C2 C9 |
| R3 | 群れを動かす | shape → Scatter → 広げる → key → 再生(E3+E5) | Inspector の Relations→gadget→stopwatch | live の Stage 反映・型の合う相手だけ強調 | C3 C4 C5 C6 |
| R4 | 3 layer を同じ look に | 1 つの look を他 3 つへ、混在を直し、棚へ保存(E6) | copy/paste、棚に drop | 側面コピー(ease だけ等) | C5 C7 C9 |
| R5 | 大きな案件の整理 | mock 200 layer: 「Blur が付いて動く物」を見つけ全員直す(E7) | 検索・折畳・U/UU | 同時絞り・Cmd-K 飛び | C1 C5 C6 C8 |
| R6 | 素材を入れて差し替え | OS から落とす→追加→Alt で置換、key が残る(E8) | AE の drop/Alt | 落とす前の outline | C2 C3 C9 |
| R7 | なぜ動く?を探る | 他人の案件でその layer が動く理由を全て答える(E9) | twirl・U/UU・stopwatch | 「動く理由の一覧」 | C6 |
| R8 | キーボードだけ | R1 をマウス無しで(E10) | J/K・P/S/R/T・Space | Cmd-K の第二段 | C10 C3 |
| R9 | 初見走行 | 説明なしの人が R1 を走る | 手癖のみ | 追加は見せない | C10 手癖 |
| R10 | 戻せる安心 | R1-R4 の途中で試して Esc/Cmd-Z | 全部 | 「確定していない」の表示 | C9 |

R1-R3 を先に作る。理由: 目的(アイデア→絵)に最も直結し、基準線と追加の両方を含み、残りがこの骨格に依存する。R9 は R1 の直後に回す(手癖だけで走れるかが最初の合否)。

### 各走行の受入(実操作で見る Y/N)
- **R1**: (1) 基準線だけで、AE/Ableton に慣れた人が完走できる。(2) 追加あり版で面の移動が減る(回数を数える)。(3) 10 個の look を試して履歴が増えない。(4) 付けた直後に数値を見ずに主要値を 1 つ触れる。(5) ease を hover で見て選べ、Esc で戻る。
- **R2**: (1) 基準線: tile の dbl-click と Cmd-Z だけで完走できる。(2) Q+↑↓ で候補の切替が 1 手。(3) 試す間「元」が分かる。(4) Esc で 5 layer 全てが試す前に戻る。
- **R3**: (1) 基準線の手順(Relations→gadget→stopwatch)だけで完走。(2) 広げた量を数値を見ず決められる。(3) key の標が広げた直後に見える。(4) 連動が付いている事が選んだだけで layer 上に見える。(5) 取消 1 手。
- **R4**: (1) 丸ごと copy/paste が手癖どおり動く。(2) 「ease だけ」を 3 layer へ 1 動作。(3) 混在が見える。(4) 棚へ drop 保存・再利用。
- **R5**: (1) 200 layer での検索・U/UU が手の速さで返る。(2) 同時絞りで Timeline と Inspector が揃う。(3) 絞り中と分かり 1 手で解除。(4) 30 個を一括で直せる。
- **R6**: (1) AE と同じく追加が既定、Alt で置換。(2) 落とす前に結果が見える。(3) 置換後 bar と key が残る。(4) Esc で何も増えない。
- **R7**: (1) 基準線(twirl・U/UU)で答えられる。(2) 追加の一覧で 5 項目(動く理由・key・relation・親・既定との差)を各 2 手以内に答えられる。(3) 見落とし 0。
- **R8**: (1) マウス無しで完走。(2) J/K・P/S/R/T が AE と同じ意味。(3) Cmd-K の第二段で手数が減る。
- **R9**: (1) 説明無しで完走。(2) つまずきを記録し次の版で潰せる。
- **R10**: (1) 試した全ての変更が Esc/Cmd-Z で戻る。(2) 「確定していない」が常に語で見える。(3) 戻った後の見た目が走行前と一致する。
