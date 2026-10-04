# Browser / Inspector: 道具に依らない「問題」の一覧と、見る角度(レンズ)

2026-10-02。調査のみ(コードなし)。読んだ物: ideas-ecosystem / ideas-old-motolii / ideas-survey / precedents-w6-w9-w10 / intents/*.md / old-motolii-complaints / workflows / DESIGN.md。
利用者の方針(採用): 制作者が Browser(ライブラリ・素材・effect・preset・検索)と Inspector(property・relation・transform・effect・key・curve)で何に困るかを **道具から切り離した「問題」** として書き、複数のレンズで眺め、UX が違う案を多く出す。

## 0. 扱い方(守ること)
- 過去の Motolii は **「どんな問題に気付いたか」「何が便利/不便だったか」** の参照だけ。その実装・仕様・約束・契約(「書かない」「Esc は送らない」等)は、新しい決定の基礎でも制約でもない(実装の都合だった可能性がある)。見た目は反面教師(DESIGN.md §4 末尾)。ここで違うのは **操作モデル**。
- 利用者の発言は材料であって仕様ではない(DESIGN.md §1)。決め方は (1) 実制作での便利さ・速さ・手触り (2) 実際のワークフロー (3) 良い先例 (4) Motolii 内の一貫。**実装コストで案を選ばない・並べない**。
- 先例が無い挙動は `[新]` と書く(先例で確かめられないので試作で最初に触る)。裁定は options 側で **先例とワークフローから自分で決め、理由を書いた**。利用者に残したのは「事実か規模の前提」で同価値な物だけ。
- 相対/絶対・clamp・対象決め・複数 write・undo 区切りは host が意味を持つ領域なので、契約には「host が決める」と書き、UI 案の比較対象にしない(既存の分担裁定)。
- 記号は増やさない(DESIGN.md §4)。案は新しい glyph を要求しない。状態は位置・灰の段・言葉で出す。
- 出所の略号: **IO** = ideas-old-motolii.md、**IE** = ideas-ecosystem.md、**IS** = ideas-survey.md、**W#** = workflows.md、**P6/P9/P10** = precedents-w6-w9-w10.md、**BF** = intents/browser-filters.md、**IN** = intents/inspector.md、**EA** = intents/ease.md、**TL** = intents/timeline-expansion.md、**CMP** = old-motolii-complaints.md。
- 「出荷済みの道具が解いたか」: Y = 先例で十分解けている / partly = 一部 / N = どこも解いていない(または出荷物で確認できず)。「Motolii の裏付け」は未確認(IS は机上調査で [seen] 無し)。

---

## 1. レンズ(12)
問題を同じ向きから見ない為の角度。options の各案は「どのレンズに有利/不利か」を書く。

| ID | レンズ | 問い |
|---|---|---|
| L1 | 探す (find) | 名前を知らない・覚えていない時に、どう辿り着くか |
| L2 | 使う前に見る (preview) | 付ける前に何が起きるか分かるか。書かずに見られるか |
| L3 | 使う (apply) | 一手で、曖昧さ無く、狙った所に効くか。置換か追加か |
| L4 | 整理 (organize) | 自分の分類・順序・出所を、ファイルを動かさず持てるか |
| L5 | 再利用・コピー (reuse) | 一部(ease だけ、色だけ)を持ち運べるか。自分の物を棚にできるか |
| L6 | つなぐ (link) | 値が別の値を読む・動かす関係が、見えて作れて外せるか |
| L7 | 一括 (bulk) | 多数に同じ・ずらして効かせられるか。「違う」を正直に言えるか |
| L8 | 状態・差分・戻す (state/diff/reset) | 今どうなっている/既定から何を変えた/元に戻せるか。絞り込み中か |
| L9 | 初心者↔熟練 (beginner-expert) | 初見で足りるか。覚えた手が速くなるか(同じ手が育つか) |
| L10 | 速さ・見つけやすさ (speed/discoverability) | 手数・移動距離は。隠れた力に手掛かりがあるか |
| L11 | 少数↔大量 (few/many) | 項目が 5 個でも 5000 個でも崩れないか(密度・仮想化・折り畳み) |
| L12 | 感覚↔数値・時間↔空間 (feel/number, time/space) | 耳・手触りで決められるか、数値で言い切れるか。時間軸と空間のどちらで見せるか |

---

## 2. 問題一覧(38)

凡例: 誰が = 初(初心者)/ 熟(熟練)/ 大(素材が多い)/ 長(長時間の作業)。

### 2.1 Browser 側 (B1-B15)

| ID | 一行の必要 | 誰が | 証拠 | 解けた? | 過去の Motolii が試した事 |
|---|---|---|---|---|---|
| B1 | 今どの置き場(project/内蔵/自分のフォルダ)を見ているかが分かり、戻れる | 初 大 | BF §1.1-1.3, IO B1 | Y(Ableton の Places) | 「SOURCES」チップ列。「·off」で消えない。が、場所・保存視点・動詞が同じ重みで並び、狭いと 3-4 行が積もった |
| B2 | 置き場を知らなくても、「どんな物か」を言って絞れる(種類・性格) | 熟 大 | BF §3.1-3.2, IO B2, IS A1 | partly(Ableton filter、Resolve smart bin) | 派生 descriptor + 1 軸の family 帯。AND の token 検索。**組合せ不能**(Blur かつ Favorites)、kind は単一選択のみ |
| B3 | 名前を知らない/曖昧にしか覚えていない物を、打って辿り着く | 熟 長 | IE §1, IS A6-A7, BF §4.1 | Y(Houdini Tab・palette)/ Adobe の Effects & Presets は N | `/` と Cmd+F、Esc で消去→離脱。検索が 4 棚で虫眼鏡の裏、Media だけ常時表示(二通り) |
| B4 | 今「絞られている」ことと件数が見え、一手で全解除できる | 全員 | BF §3.4, IO B2 | Y(Ableton Results bar) | 絞りの合算表示も一括 Clear も無し。「Nothing matches.」は行き止まり |
| B5 | 付ける前に、それが何をするか画で分かる(出来れば自分のコマで) | 初 熟 | IO B4, IE §2, BF §9.1 | partly(固定サンプルは Y、自分のコマは N) | 効果を固定サンプル画に描いた tile。自分のコマには無し |
| B6 | 動きのある物(動画・動く preset)を開かず・押さず中身まで覗く | 熟 大 | BF §7.2, IE §2 Animation Composer | partly(Resolve/Premiere の hover scrub、AC の hover 動画) | 押下中のみ 2-4px の strip で scrub。hover では動かない |
| B7 | クリック・ダブルクリック・Enter・ドラッグが、全棚で一つの意味を持つ | 全員 | BF §8.2, IO B5, W6 | partly | 効果 tile は単クリック=適用、media は単クリック=選択。反応しない tile が淡色 48% のまま押せた(8/25) |
| B8 | 既にある物の上に落とした時、置換か追加かが予測でき、選べる | 熟 | P6, W6, BF §8.1 | partly(AE: Alt=置換、Premiere: 既定が overwrite、Ableton: preset は置換) | media ドラッグは ghost 85%・元 40%。置換は「Replace selected layer」メニューのみ |
| B9 | いま付いている effect/preset を、別の候補に差し替えて、その場で見比べる | 熟 長 | BF §8.3, IO B4「未解決」 | partly(Ableton Hot-Swap は音、映像は N) | 無し |
| B10 | よく使う物・最近使った物に戻れる | 長 大 | BF §5.1-5.2, IO B3 | Y(Ableton Collections、frecency) | 7 色 Collections+Recent(上限 12)。Ableton の 1-7/0 キーは写さず、コンテキストメニューだけ。class 帯に混ぜた |
| B11 | 自分が作った物(curve・色・effect の組・layer)を棚の一員にできる | 熟 長 | EA §C1, IE §4 Layer Library, IS A2「mark as asset」 | partly(Blender・Cavalry・Procreate は Y) | Ease の保存 curve と保存 gradient だけ。名前無し・個別削除無し・Presets 棚無し |
| B12 | 数千点の素材から、見て探す/似た物を探す/並べ替える | 大 | BF §6.1-6.3, IS A9 Splice「similar」 | partly(thumbnail+並替は Y、「似た物」は N) | List↔Thumbnail の同じ顔が動く、size 44-140、列は幅で落ちる。Explore 地図は 300 点上限で大規模に使えず、関係線は一部空 |
| B13 | 今の選択に効かない物は最初から出ない/理由が分かる(選択で絞る) | 初 | BF §8.4, IS A6 Houdini Tab、Cavalry の dim | partly(Houdini は Y) | 非対応 tile を 48% 淡色で残し、説明無し |
| B14 | 外の素材を取り込み、欠けたら分かり、フォルダを生きた入力にできる | 大 長 | BF §10, IS A5 Cavalry Smart Folder, IE §3 Overlord | partly | drop veil・欠損バッジ+Locate は良好。フォルダ=生きた源は無し |
| B15 | 狭い dock に入れても探して使える | 長 | IO B7, BF §11 | partly | 幅・高さ別の 3 形態。ただし 240px の切りが 3 箇所に複製、dock で書体が崩れた(CMP#2) |

### 2.2 Inspector 側 (I1-I15)

| ID | 一行の必要 | 誰が | 証拠 | 解けた? | 過去の Motolii が試した事 |
|---|---|---|---|---|---|
| I1 | 今 何の property を見ているか分かる。複数選択では正直に「違う」と言う | 全員 | IN A1,A6,A7, IO I1 | Y(Figma "Mixed") | subject 3 種、"N layers"、混在「—」、drag は相対・typed は絶対 |
| I2 | 数値を、速度の予測が付くまま変える(scrub / 入力 / wheel / 矢印 / 既定へ) | 全員 | IN C1-C10, CMP#1, IO I3 | partly(AE: Shift x10・Ctrl x0.1) | 二つの数値部品が **逆の scrub 向き・逆の Shift** を持っていた。値の大きさに比例する step |
| I3 | 空間の値(位置・scale・回転・anchor)を手触りで、数値は二の次で | 全員 | IN B3+Owner decision, DESIGN §1, IO I2 | partly(Figma/AE は数値中心、Blender は gizmo) | Transform instrument(pad+dial+handles)。4 モード+モード別色=道具の中の道具。Stage と重複 |
| I4 | 「これは時間で変わる」を値の所で言える(状態が 3 つ) | 全員 | IN D1-D4, IO I4 | Y(AE stopwatch)/ 状態の見分けは partly | 行ごとの diamond 3 状態を色調のみ・7.5px で |
| I5 | 二つの key の間の動きの癖(ease)を、graph editor 無しで決める | 熟 | EA, P9, IO I5 | partly(AE/Flow/Figma。**hover で書かずに覗く**は先例無し) | Ease desk: 9 family・peek(書かない)・handle drag は release で 1 undo。保存 curve 名無し。狭いと意味が消えた |
| I6 | 既定から変えた物が分かり、一手で戻せる(値・group・layer 単位) | 熟 長 | IN E4,C7, IS B10-B14 | Y(Unreal・Houdini・Unity) | off-default の点+reset 矢印+diamond+relation 錠が 1 行に混み合った |
| I7 | 長い parameter 一覧から、触る物へ速く着く(絞る・畳む・探す) | 熟 | IN F2,F3,H3, IS B11-B13 | partly | hero+「Advanced n」、section 折畳、検索(live 結線は未確認) |
| I8 | effect の積み(順序・on/off・差替・まとめて reset)を扱う | 熟 | IN F1,F5, IS B10 | Y(AE) | card(on/off・fold・menu・drag 並替)。header が fold と drag grip を兼ね誤爆 |
| I9 | ある値が別の値に従う/別の値を動かす関係を、見て作って外す | 熟 | IO U1,U2, IE §7, IS B14 Cavalry | partly(Cavalry connection・Ableton Map は良) | Relations v0: lasso→Set→normalize→1:N。badge は 2 箇所に重複。Scatter 等の gadget は未実装 |
| I10 | 色・font・blend・ease を小さな部品でなく専門道具で、その slot に向けて | 全員 | IN I1, IO I6 | partly | route 行+pill が既存の語を繰り返す。Fonts は family 名のみ |
| I11 | 「一側面だけ」(ease・色・key・property 群)を他へ持ち運ぶ | 熟 | IE §6, §9 | N(host は丸ごとコピーのみ。Motion/Blender アドオンが補う) | 無し |
| I12 | 多数 layer に同じ変更を、ずらして/相対で/一括で入れる | 熟 大 | IE §8, W5, IN A7 | partly(Blender Alt・Cavalry Alt。ずらしは Motion が補う) | 相対・絶対の規則は良。stagger は Relations 側 |
| I13 | 値を変えて試し、元と見比べて戻せる(A/B・snapshot) | 熟 | IE §12 Snapshot/Re-Key, IS B12 Unity override | N(出荷物は個別 reset と undo のみ) | 無し |
| I14 | 「何を指すか」(parent・参照 layer・source)を、数百 layer でも選べる | 熟 大 | IN G3 | Y(pick-whip + 検索 dropdown) | prev/next 矢印で全 layer を巡る(数百で不能) |
| I15 | 深い構造から少数の操作面を作る(必要な 5 つだけ露出・macro) | 熟 | IE §3,§10, IS A3 component properties | Y(Essential Properties・Houdini promote・Unreal) | hero 宣言(host 側)のみ。利用者側に作る手段は無し |

### 2.3 横断 (X1-X8)

| ID | 一行の必要 | 誰が | 証拠 | 解けた? | 過去の Motolii が試した事 |
|---|---|---|---|---|---|
| X1 | どの panel で選んでも、他 panel が同じ物を指す(選択・hover の共有) | 全員 | W1, IN I2 | Y | Stage↔Inspector は結線。lab の panel は未結線(W 0 節) |
| X2 | 試す→書かずに見る→確定/取消が、全箇所で同じ作法 | 全員 | EA A1, P9, IN C3, U5 | partly | Esc/blur/cancel を一級の相に。peek は Ease のみ |
| X3 | キーボードだけで、選ぶ・探す・動かす・実行まで | 熟 長 | W10, P10 | partly | Inspector のみ(P/S/R/T・矢印)。palette・Timeline・Browser は未実装 |
| X4 | drag・Shift・Alt・wheel・矢印が、アプリ中で一つの文法 | 全員 | IN C4,J-1,J-2, CMP#1, P10 | N(先例同士が割れる) | 設計文には一文法と書いたが実装が二つに割れた |
| X5 | 効かせる「対象」(選択 layer・key・区間・active)が操作の前に分かる | 全員 | BF §8.3, W9, TL C6 | partly | 対象決めの規則が panel ごと。複数選択の scope も panel ごと |
| X6 | 空・無結果・失敗・禁止を、理由と次の一手つきで言う | 初 | BF §10, IN H2 | partly | 行き止まりの文言、淡色のみの禁止、色だけの Locked |
| X7 | dock 化・縮小時にも書体・線・主張が揃う(声量) | 全員 | CMP#2,#3, IO B7 | N(判定の基準にまだ入っていない) | 反面教師。lab は T.* に集約、例外 5 |
| X8 | Browser で選んだ物を、置いた後の Inspector で「そのまま調整」へ繋ぐ | 初 熟 | IE §3 Animation Composer Edit, Premiere Graphics Templates | partly(AC は良) | 無し(Browser→置く、で止まる) |

---

## 3. 問題 × レンズ 行列
● = そのレンズで最も強く現れる(解き方の違いがそこで割れる)。○ = 副次的に関わる。空白 = 薄い。

| 問題 | L1 find | L2 preview | L3 apply | L4 org | L5 reuse | L6 link | L7 bulk | L8 state | L9 b/e | L10 speed | L11 few/many | L12 feel/time |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| B1 | ○ |  |  | ● |  |  |  | ○ | ○ | ○ | ● |  |
| B2 | ● |  |  | ● |  |  |  | ○ | ○ | ○ | ● |  |
| B3 | ● |  | ○ |  |  |  |  |  | ● | ● | ○ |  |
| B4 | ○ |  |  |  |  |  |  | ● | ○ | ○ |  |  |
| B5 | ○ | ● | ○ |  |  |  |  |  | ● | ○ |  | ● |
| B6 |  | ● |  |  |  |  |  |  |  | ○ | ● | ● |
| B7 |  |  | ● |  |  |  |  |  | ● | ● |  |  |
| B8 |  | ○ | ● |  |  |  |  | ○ | ○ |  |  |  |
| B9 |  | ● | ● |  |  |  |  | ● | ○ | ○ |  | ● |
| B10 | ○ |  |  | ● | ○ |  |  |  |  | ● | ○ |  |
| B11 |  |  |  | ● | ● |  |  |  | ○ |  | ○ |  |
| B12 | ● | ○ |  | ○ |  |  |  |  |  |  | ● |  |
| B13 | ● |  | ○ |  |  |  |  | ○ | ● |  | ○ |  |
| B14 |  |  | ○ | ○ | ○ | ● |  | ● |  |  | ● |  |
| B15 | ○ |  |  |  |  |  |  |  |  | ○ | ● |  |
| I1 |  |  |  |  |  |  | ● | ● | ● |  | ○ |  |
| I2 |  | ○ | ○ |  |  |  | ○ |  | ● | ● |  | ● |
| I3 |  | ○ |  |  |  |  | ○ | ○ | ● |  |  | ● |
| I4 |  |  |  |  |  |  |  | ● | ○ | ○ |  | ● |
| I5 |  | ● | ● |  | ● |  |  | ○ | ● |  |  | ● |
| I6 |  |  |  |  |  |  | ○ | ● | ○ | ● | ○ |  |
| I7 | ● |  |  | ● |  |  |  | ○ | ● | ● | ● |  |
| I8 |  | ○ | ● | ● | ○ |  |  | ● |  |  | ● |  |
| I9 |  | ○ | ● |  |  | ● | ● | ● | ● |  | ○ | ● |
| I10 |  | ○ | ● |  |  |  |  |  | ● |  |  | ● |
| I11 |  |  | ● |  | ● | ○ | ○ |  | ○ |  |  |  |
| I12 |  | ○ | ● |  |  | ○ | ● | ○ | ○ |  | ● | ● |
| I13 |  | ● |  |  | ○ |  |  | ● | ○ |  |  | ○ |
| I14 | ● |  | ● |  |  | ● |  |  |  |  | ● |  |
| I15 |  |  |  | ● | ● | ○ |  | ○ | ● |  | ● |  |
| X1 |  |  |  |  |  | ○ | ○ | ○ |  | ○ |  |  |
| X2 |  | ● | ○ |  |  |  |  | ● | ● |  |  |  |
| X3 | ● |  | ○ |  |  |  |  |  | ● | ● |  |  |
| X4 |  |  | ○ |  |  |  | ○ |  | ● | ● |  | ○ |
| X5 |  |  | ● |  |  |  | ● | ○ |  |  |  |  |
| X6 |  |  |  |  |  |  |  | ● | ● |  |  |  |
| X7 |  |  |  |  |  |  |  | ○ |  |  | ● |  |
| X8 |  | ○ | ● |  | ○ | ○ |  |  | ● | ○ |  | ○ |

読み方:
- 列合計が大きいレンズ(L8 state / L9 beginner-expert / L10 speed)は **どの問題の案も、この三つで勝ち負けが割れる**。案を比べる時の第一の物差し。
- L2 preview と L8 state が重なる所(B9, I13, X2)は、試す作法(options-browser.md D-1)がそのまま効く。
- L11 few/many が強い所(B1, B2, B12, B15, I7, I8, I12, I14, I15)は、案を 5 個と 5000 個の両方で試す必要がある(試作の確認項目に入れた)。

---

## 4. 見えてきた事(次の文書への橋)
1. 先例が手を付けていない所(解けた? = N): **I11 一側面コピー、I13 A/B・snapshot、X4 一つの文法、X7 dock での声量、B12 の「似た物」**。Motolii が差を作れる所だが、「意味を発明しない」の縛りで、案の前に利用者の「普通」を聞く。
2. 先例が実制作で通用している(Y)のに過去の Motolii が崩した所: **B4(絞りの合算+一括 Clear)、B10(数字キーで即付与)、I2(一規則)、I14(検索付きの選択)**。ここは先例を土台にし、案の分岐は小さい。
3. 「確定まで履歴を汚さずに試す」(peek)は B5 B6 B9 I5 I13 X2 を貫く 1 本の作法で、候補を連続して見る実制作の速さから決める(options-browser.md D-1)。
4. 選ぶ基準(options で 10 問題ずつ): 証拠が多い(複数の文書に出る)+ 制作で毎日起きる + 先例が割れているか未解決、を優先。Relation(I9)は「Relation は操作」の既存裁定(入口は Inspector Property → Relations パネル)を壊さないよう、**Browser の棚にはしない**前提で扱う。

---

## 5. Experience-first view(目的に合わせた見方)
DESIGN.md §0: 目的は「アイデアを最短で絵・作品にする体験を、Widgetbook の実操作で見つけて証明する」事。利用者の手癖(AE/Ableton/Blender/Figma の筋肉記憶)を尊重し、Browser/Inspector/Stage/Timeline の分割は原則壊さない(壊すには実操作での強い証拠が要る)。
上の 38 問題・12 レンズ・options は材料として残す。**代表的な制作の流れ(E1-E10)、各流れの「手癖の基準線+追加」、完成条件 10(+手癖)と実操作の受入、手癖の台帳(手癖のまま/追加/外れる/取り下げ)、流れ単位の試作 R1-R10 は experience-first.md。** 試作の順は experience-first.md §D が優先(prototypes-next.md の部品単位の旧リストは各走行の部品の参考)。
