# Cavalry への文句 = Motolii の要件(2026-09-16)

利用者「多分、Cavalry にはもう十分に意味があるんですよね、だからこういうかっこいい画が見れる」
「次に話すべきなのは、意味を探すんじゃなくて、Cavalry がなんであんなめんどくさくて難しいのかという文句です」

意味は借りる(Scheduling Group・Stagger・Distribution・Level Mode・Fields・Collision Events・Ground Mode)。
面倒は消す。この一覧は「何を消すか」の定規。文句は文句の形をしていない事も多い(「難しいですが…」「Cavalry Brain が要る」)。

## 取説から出る文句(docs.cavalry.scenegroup.co / cavalry.studio/docs)
1. **意味が配線** — 時間差 = Stagger を作り `stagger.id` を Duplicator の `Shape Time Offset` に繋ぐ。意図 1 つがノード 2 個 + 線 1 本 + Id の理解
2. **内部語が前提** — Id・sub-mesh。「最初の Id に Minimum、最後の Id に Maximum」。人は字と升目を見ている
3. **道具の都合を人が回避** — 「0 コマ目より前に始まらないよう Minimum を負に」が取説の Tips
4. **物理の儀式** — Make Dynamic → Solver Shape → Bodies → Active Solver → Add Field → Body Settings(Density/Friction/Bounce/Damping×2/Iterations×2/World Scale/Time Step)。硬さ 1 つに欄 15
5. **収束しない** — 入門記事の教え「一度に 1 つ、少しずつ、混沌なら半分に」。人が二分探索
6. **cache を人が管理** — Cache Solver / Use File Cache / Cache File Path / Remove Cache
7. **舞台が欄** — Ground Mode: Off/Bottom/Edges。構図を見れば分かる事を聞く
8. **粒度が物理の奥** — 字を 1 字ずつは Body Settings の Level Mode: Characters
9. **既定が無い** — 空の comp は何もしない。全部ノードを足す
10. **画とグラフが別の場所** — Graph 欄・Attribute Editor。絵を触って絵が変わらない

## 利用者の文句(2026-09-16)
11. **直観的でない** — やりたい事から入れない。語彙(Behaviour / Utility / Distribution)が先
12. **ピクセル操作が重い** — ベクターの道具にラスターは後付け。fx を重ねると落ちる
13. **令和のソフトに見えない** — ノードの箱と線、数字欄、ファイル cache、ドック。Figma 以後の型が無い
14. 12・13 の根は同じ — 描画の境目が浅い(ベクター・ラスター・物理が別の道)

## 検索で出る文句(2026-09-16、文句の形をしていない物も)
15. 「it's so hard to use and doesn't make any sense … no user guide」(一般利用者の声、検索結果より)
16. 「procedural thinking takes time」「Cavalry Brain が数プロジェクト後に click する」(elements.envato.com/learn/cavalry-motion-graphics、motioncircles.com) — 数週間の脳の切替を要求する
17. 「node interface は Houdini より浅いが AE の direct manipulation より steep」(schoolofmotion.com/blog/cavalry-houdini-of-2d-after-effects)
18. 「複雑な node graph と real-time preview は VRAM を食う、6GB 未満で遅くなる」(superrendersfarm.com review 2026)
19. **最終合成は AE に戻す** — 「Cavalry の出力を AE に重ねて色・fx・仕上げ」(同上)。画を完成させる場所ではない
20. 小さいコミュニティ、チュートリアルが少ない(同上)

## 裏返し = Motolii の答え
| 文句 | 答え | 今 |
|---|---|---|
| 1, 2, 9 | **箱が配線**。Group = Scheduling Group、Grid = Distribution、部屋 = Solver。順番は層の順。何もしなくても構図が動く | 箱 ✓、順番の札 **無し** |
| 3, 5 | 終わりは構図(必ず着く)、箱から出ない(嘘) | ✓(2026-09-16) |
| 4 | 硬さ・重さ・ばらつきの 3 つ、場は 1 効果 | ✓ |
| 6 | 尺が有限なので黙って焼く | ✓ |
| 7 | 構図が環境 | ✓ |
| 8 | Text の 1 札(Level は借りる) | **無し** |
| 10, 11 | 棚に置けば効く、関係は絵の上に出る(ケージ・升目・場) | 棚 ✓、Trace は物理だけ |
| 12, 14, 18 | 描画の境目は深く 1 本、全部 GPU、読み戻さない | ✓(memory: 天井を作らない) |
| 13 | Flutter、dev + reload、台本、法は vism の text | 実窓の検収が残る |
| 16, 17 | 脳の切替を要求しない: 語彙は CSS の箱と AE の層 | 11 と同じく未検収 |
| 19 | **画はここで完成する**(fx・色・合成が同じ家) | vism の棚 ✓ |

最大の未達は 1・2・9(順番)と 10・11(絵の上で触る・見える)。次の 1 手は箱に順番の札(Sequence / Overlap / From End / Order)。
