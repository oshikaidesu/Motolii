# LLM にアニメーションを「見せる・測る・教える」— Web 分野の到達点の索引(2026-09-18 取得)

課題: 語彙を渡す型(gsap-skills、Remotion Agent Skills、Motion for AI)は既知。その先、**LLM は間のコマを見ていないから動きの良し悪しが分からない**という弱点を Web 分野がどう埋めているか。一次資料(公式 docs・GitHub README/SKILL.md・arXiv・作者本人)だけを当日取得。取れなかった物・又聞きしか無い物は「未検証」と書いた。

## 0. 要約(5 行)

1. **見せる**は「コマの格子(contact sheet)1 枚を vision に渡す」が事実上の標準になった。web-motion-skill(24 コマ格子 → 直して録り直し)、pneuma-skills の contact sheet(2026-09-14)、Chrome DevTools MCP / Playwright MCP の screencast(動画は agent が直接読めないので ffmpeg でコマ出し)。
2. **測る**は 2 系統。(a) 参照ありの類似度 — SSIM/CLIP を 5fps のコマ列に DTW で当てる(ManimBench)、appearance と temporal を分けた指標(Animation2Code)。(b) 参照なしの CV 診断 — 2fps のコマから重なり・はみ出し・**運動エネルギーの跳ね(中央値の 10 倍)・flicker** を数で拾う(OmniManim, 2026-05)。Web の CLS/INP は「ずれ・応答」であって「動きの質」ではない。Smoothness(dropped frames %)は 2021 の提案のまま。
3. **教える**は「原則の書き直し + 禁止リスト」の型に収束。Emil Kowalski 本人の skills(review-animations 等 12 本、38.5k★)、LottieFiles の motion-design-skill(Disney 12 原則の UI 版、40+ agent)、vercel-labs の web-animation-design。**どれも code を読む審査で、描画結果は見ない**。
4. **失敗の列挙**: 一様な fade-in、全部同時(stagger 抜け)、scale(0) から出す、ease-in を UI に、300ms 超、脈打つ表示、blur 乱用、bouncy spring を実用操作に、mount 時に静的物まで動く(kylezantos の anti-slop 一覧、Emil の hard block)。研究側: 群/個の取り違え、delay と duration の噛み合わせ失敗(Keyframer)、最終コマしか見ないので途中の重なりを見逃す、loop の残差(LogoMotion)。
5. **VLM 自体の限界**(AniMINT, ACL 2026 Findings): 10fps のコマ列で「動いた種類」は 9 model 中 5 が満点、しかし「何のための動きか」は最高 64%、画面の 15% 未満の小さな動き・短い揺れ(shake)は落とす、最終コマの文字に引きずられる(static frame bias)。動きの残像(transparency decay の合成)+ 文脈文 + 動きの字幕で 3.15 → 3.52。

## 1. 見せる — agent に動きを見せる仕組み

| 名前 | 何をする | 一次資料 | Motolii に写せるか |
|---|---|---|---|
| Chrome DevTools MCP(Google 公式) | `screencast_start/stop`(webm/mp4)、`take_screenshot`(png/jpeg/webp、要素単位)、`performance_start_trace`(LCP/INP/CLS の insight)、`emulate`(CPU 絞り・回線)。**動画をコマにする道具は無い** | https://github.com/ChromeDevTools/chrome-devtools-mcp · https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/main/docs/tool-reference.md | 録画 → コマ出しは自前。trace 相当は無い |
| Playwright MCP(公式) | `browser_start_video/stop_video`、`browser_video_chapter`(章の札)、`browser_start_tracing`(Trace Viewer 用)、`browser_take_screenshot`、`--caps=vision`。既定は accessibility tree なので動きは見えない | https://playwright.dev/mcp/introduction | 同上 |
| web-motion-skill(Schmandarine、MIT) | headless Chromium で約 9 秒 scroll しながら録画 → 25fps 約 220 コマ → **24 コマの contact sheet(コマ番号焼き込み)1 枚**を Claude が読む → entrance/dwell/exit の窓、stagger の向き、跳び・切れを目で判定 → code 修正 → 録り直し。数値指標は無し | https://github.com/Schmandarine/web-motion-skill | 写せる(scroll を時刻 scrub に置換) |
| pneuma-skills PR #149「look before sampling」(2026-09-14) | `sprite-sheet.mjs contact <clip>` で時刻付き格子 PNG + 数値(stillStart/stillEnd、loop 検出、コマ間 silhouette 差)。「agent の目のための作業 file、asset ではない」 | https://github.com/pandazki/pneuma-skills/pull/149 | 写せる(格子 + 差分数値の 2 本立て) |
| Remotion Agent Skills(公式) | `/remotion-render` = "Invoke a render into a video or a still"、`/remotion-studio` で preview。**公式 skill に「コマを見て確かめろ」の規則は無い** | https://www.remotion.dev/docs/ai/skills · https://github.com/remotion-dev/skills | still 出力の口は既にある |
| claude-remotion-skill(haidrrrry、MIT、第三者) | 10 の hard rule の 10 番目が "Render → inspect frames → fix → re-render. Never ship unverified."(ffmpeg でコマ抽出。何コマ・どの時刻かは未記載) | https://github.com/haidrrrry/claude-remotion-skill | 規則文としてそのまま |
| Lottie Creator MCP(LottieFiles 公式、2026-09-11 更新) | scene の inspect、**playhead を指定コマへ移動**、easing/timing の読み書き、「motion speed・contrast の点検」。描画像を返す tool は明記無し。WebMCP でも接続可 | https://docs.lottiefiles.com/en/creator/13_ai-tools/lottie-creator-mcp | playhead を動かす口 = scrub API の先例 |
| Rive MCP(公式、desktop Editor のみ) | 6 分類(file、scene inspect、shape、animation/state machine、data binding、Luau/WGSL)。**描画・preview・screenshot の tool は無い** | https://rive.app/docs/editor/ai/mcp | 反面教師(見せる口が無い) |
| Rive Console MCP(saralobo、第三者) | 「headless preview engine が frame を抽出し motion quality を検証」と紹介。**本体 403 で未検証** | https://lobehub.com/mcp/saralobo-rive-console-mcp(未検証) | — |
| MotionLens(MCP、商用) | 画面録画から timing・easing を推定し "Motion Blueprint" と code を返す。**本体 402 で未検証** | https://motionlens.dev/(未検証) | — |
| Figma Make / v0 / Lovable / Bolt | アニメーションの検証手順は**公開情報が見つからず**(v0 は Framer Motion を出す、以上) | — | 見つからず |
| Chrome DevTools Animations panel を agent が読む例 | CDP Animation domain(seek、pause、playbackRate、timing 変更)は存在するが、MCP には露出無し。実例**見つからず** | https://chromedevtools.github.io/devtools-protocol/tot/Animation/ | 「時刻 scrub の口」の仕様参考 |

## 2. 測る — 動きの質を数にする物

| 名前 | 何をする | 一次資料 | Motolii に写せるか |
|---|---|---|---|
| OmniManim「See Before You Code」(2026-05-15) | render 後に **2fps のコマから CV 診断**: 重なり(HSV mask・bbox IoU・文字行の交差)、はみ出し、**運動の不連続(energy jump > 中央値 ×10)、flicker(連結成分数の急変)**、palette shift。VLM は旗が立った区間だけ意味審査。Vision Agent が疎な keyframe の bbox を先に置き、**補間で生まれる中間コマの失敗**を目的関数に入れる | https://arxiv.org/abs/2605.15585 | 診断 4 種はそのまま写せる(GPU 読み戻し無しで可) |
| ManimBench / RITL(2026-04、改 2026-08-31) | Visual Similarity = SSIM(gray)+ CLIP ViT-L/14(RGB)を **5fps のコマ列に DTW 整列**して幾何平均。RITL の feedback は**render error log の末尾 10 行だけ**、コマは見せない。RSR 94% / VS 85.7%(Qwen3 Coder 30B + GRPO + RITL-DOC) | https://arxiv.org/abs/2604.18364 | 参照ありの比較(束の証明の画素差 0 と同族) |
| Animation2Code(2026-06-26) | 1,069 本の web animation 動画 ↔ HTML/CSS/JS。**appearance similarity と temporal similarity を分離**。結論: 「見た目は合うのに時間が合わない」が finetune でも反復でも残る | https://arxiv.org/abs/2606.28593 | 2 軸分離の考え方 |
| PRISM(2026-05-19) | 10,372 の指示↔code。実行可 → 空間的に正しい への落差 **約 41%**(Execution-Spatial Gap)。Temporal Density で時間の活動量を診る | https://arxiv.org/abs/2605.19382 | 「走った ≠ 正しい」の数字 |
| AniMINT「Beyond Screenshots」(ACL 2026 Findings、2026-04-28) | 300 本の UI animation を **60fps → 10fps** に落として VLM に。効果分類は満点、目的分類は最高 64%、解釈 3.47/5。失敗: 最終コマの文字に引きずられる、画面 15% 未満の小さな動き、短い shake、文脈盲。**残像合成 + 文脈文 + 動きの字幕**で改善 | https://arxiv.org/abs/2604.26148 | 見せ方の設計指針(残像・字幕) |
| LogoMotion(2024-05、改 2025-02) | 目標 layout と**最終コマ**の bbox 差で位置(30.4% の走行)・scale(18.4%)を検出 → VLM に 2 枚見せて修正。明記された穴: **途中コマの一時的な重なりは見ない**、色や影の誤りは通る、loop の残差 | https://arxiv.org/abs/2405.07065 | 最終コマ検査は最低線、途中を見るには上の CV 診断 |
| Chrome DevTools MCP の trace | LCP/INP/CLS の insight。**滑らかさの指標は無い** | https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/main/docs/tool-reference.md | 動きの質には使えない |
| web.dev Smoothness(Percent Dropped Frames) | 平均/1 秒窓の最悪/95 パーセンタイルの 3 案。**2021-11-03 更新のまま提案段階**。LoAF API(Chrome 123〜、W3C FPWD 2026-04-28)は 50ms 超のコマを列挙するだけ | https://web.dev/articles/smoothness · https://developer.chrome.com/docs/web-platform/long-animation-frames | 「落ちたコマ %」は実装が軽い |
| 動画生成側の LLM-as-judge(VBench++、Video-Bench CVPR 2025) | motion smoothness(補間 based)+ dynamics degree(optical flow)。対象は生成動画で、UI/モーショングラフィックス向けではない | https://openaccess.thecvf.com/content/CVPR2025/papers/Han_Video-Bench_Human-Aligned_Video_Generation_Benchmark_CVPR_2025_paper.pdf | 借りる程では無い |
| timing/spacing の自動評価(ease の良し悪しを数で) | **見つからず**。全て「参照との一致」か「破綻の検出」で、「良い ease」を数にした物は無い | — | 見つからず |

## 3. 語彙以外の教え方

| 名前 | 何をする | 一次資料 | Motolii に写せるか |
|---|---|---|---|
| emilkowalski/skills(本人、38.5k★) | 12 本: animate / review-animations / improve-animations / find-animation-opportunities / animation-vocabulary / apple-design ほか。review は **code を diff で審査**(描画は見ない)。hard block: UI に ease-in、300ms 超、scale(0) 入場、`transition: all`、layout 属性、全部同時(30〜80ms の stagger)、reduced-motion 無視、100 回/日の操作に動き | https://github.com/emilkowalski/skills · https://github.com/emilkowalski/skills/blob/main/skills/review-animations/SKILL.md | 審査規則の型。数値は UI 向け |
| LottieFiles/motion-design-skill(公式、MIT) | 「Disney 12 原則の UI 版」+ 感情→動きの対応 + 8 段の checklist + duration/easing 表 + quality checklist。40+ agent。自動検証は無し | https://github.com/lottiefiles/motion-design-skill | 原則の書き直しの先例 |
| kylezantos/design-motion-principles | create / audit の 2 mode。audit は **code** の gap と anti-slop 一覧を HTML 報告に。Emil・Krehel・Jhey の公開文を蒸留(本人非公認) | https://github.com/kylezantos/design-motion-principles | anti-slop 一覧(§4) |
| vercel-labs/open-agents web-animation-design | 100〜150 / 150〜250 / 200〜300ms、exit は entrance の 20% 速く、transform と opacity だけ、ease-in 禁止、blur 20px 超禁止。検証の指示は無し | https://github.com/vercel-labs/open-agents/blob/main/.agents/skills/web-animation-design/SKILL.md | 同上 |
| Keyframer(Apple、2024-02、改 2025-08) | 入力は **SVG code だけ**(絵は渡さない)。設計者は「分解して少しずつ」prompt し、描画を横に並べて目で比べる。CSS の誤り 6.7% | https://arxiv.org/abs/2402.06071 | 「分解 prompt + 並べて比べる」 |
| Theatre.js / Framer | Framer 3.0(2026-06-16)は canvas 内 agent + MCP plugin、Theatre.js は第三者 skill のみ。**動きの検証の記述は見つからず** | https://www.framer.com/marketplace/plugins/mcp/ | 見つからず |
| eval 駆動で動きを改善するループ | 実例は web-motion-skill(§1)と OmniManim(§2)。Web 製品側の公開実例は**見つからず** | — | — |

## 4. 失敗の報告 — LLM が典型的に間違える所

| 出所 | 列挙 | 一次資料 |
|---|---|---|
| kylezantos anti-slop checklist | 脈打つ表示、blur で全部入場、hover で全部 scale、**stagger 乱発**、実用操作に bouncy spring、**一様な fade-in**、静的物まで mount で動く | https://github.com/kylezantos/design-motion-principles |
| Emil Kowalski review-animations | ease-in を UI に、300ms 超、scale(0) から("Nothing appears from nothing")、**全部同時**、popover の transform-origin: center、押す/離すの対称 timing、割り込めない keyframes、`transition: all` | https://github.com/emilkowalski/skills/blob/main/skills/review-animations/SKILL.md |
| Keyframer(研究) | **群と個の取り違え**(3 つの星を 1 塊として動かす)、**delay と duration の噛み合わせ失敗**(一緒に動くべき物がずれる)、「言った通りにやったが言うべき事が違った」 | https://arxiv.org/html/2402.06071v1 |
| LogoMotion(研究) | 位置ずれ 30.4%、scale 18.4%、loop の残差、**途中コマの重なりは検出器が見ない**、色・影の誤りは通る | https://arxiv.org/html/2405.07065v1 |
| OmniManim(研究) | 「**個々には正しい 2 つの keyframe が、補間された途中で重なる・隠れる・関係が壊れる**」、flicker、palette shift | https://arxiv.org/html/2605.15585 |
| Animation2Code / PRISM(研究) | 見た目は合うが時間が合わない(反復しても直らない)、実行可と空間正しさの落差 41% | https://arxiv.org/abs/2606.28593 · https://arxiv.org/abs/2605.19382 |
| AniMINT(研究、VLM の目の側) | 最終コマの文字に引きずられる、小さい・短い動きを落とす、直前の操作との文脈を繋げない | https://arxiv.org/html/2604.26148v1 |

## 5. Web が解いた事 / 解いていない事

**解いた**
- 「見せる」の形: 動画ではなく **時刻を焼いたコマの格子 1 枚**(24 コマ前後)+ 必要なら個別コマへ drill。ffmpeg で自前。動画そのものを LLM に渡す道は主流に無い。
- 「破綻の検出」: 重なり・はみ出し・不連続(energy jump)・flicker・落ちたコマ %、参照があれば SSIM/CLIP + DTW。全部 CV で、VLM は旗の立った所だけ見る。
- 「語彙より先の教え」: 原則の書き直し(Disney → UI)+ **禁止リスト + 数値の上限**(300ms、stagger 30〜80ms)+ code 審査の skill。本人(Emil)が出した事で標準化した。
- 「途中のコマの失敗は code からは見えない」を研究が言語化し(OmniManim、LogoMotion の自己申告)、keyframe の間を目的関数に入れ始めた。

**解いていない**
- **良い ease・良い spacing を数にする物は無い**。あるのは「参照と一致」か「壊れていない」だけ。timing/spacing の美醜は依然 人の目。
- VLM は「何が動いたか」は分かるが「何のための動きか」は 64%、小さく短い動きは見えない。残像合成・字幕で少し上がる程度。
- Web の計測(CLS/INP/LoAF)は応答とずれの指標で、モーショングラフィックスの質とは別物。Smoothness は 5 年前の提案のまま。
- 商用の生成 UI 道具(Figma Make / v0 / Lovable / Bolt)、Rive 公式 MCP、Remotion 公式 skill は **agent に動きを見せる口を持たない**か公開していない。
- 公式 skill はどれも code 審査で止まり、描画結果を見て直すループは第三者の skill と論文にしか無い。

## 6. Motolii への含意(判断は書かない)

- 見せる口: 時刻を焼いた contact sheet(24 コマ程度 + コマ間差分の数値)を render crate の still 出力から作る形が Web の共通解。
- 測る口: OmniManim の 4 診断(重なり・はみ出し・energy jump・flicker)と「落ちたコマ %」は参照無しで動き、GPU 読み戻し無しに近い。
- 教える口: Emil / LottieFiles の型は「原則 + 禁止 + 上限値」だが数値は UI 向け。Motolii の台本の語彙に同じ型を被せる先例になる。
- 見せ方の設計: VLM に渡す時は残像合成 + 動きの字幕 + 文脈文(AniMINT)、最終コマだけでは途中の重なりを見逃す(LogoMotion)。
- 未解決のまま残る物: ease の美醜の数値化。ここは Web にも無い(利用者の目の専権と一致)。

## Sources(取得 2026-09-18、一次のみ)

- 公式 tool: github.com/ChromeDevTools/chrome-devtools-mcp(README、docs/tool-reference.md)· playwright.dev/mcp/introduction · remotion.dev/docs/ai/skills · github.com/remotion-dev/skills · docs.lottiefiles.com/en/creator/13_ai-tools/lottie-creator-mcp · rive.app/docs/editor/ai/mcp · chromedevtools.github.io/devtools-protocol/tot/Animation · web.dev/articles/smoothness · developer.chrome.com/docs/web-platform/long-animation-frames · framer.com/marketplace/plugins/mcp
- skill(作者本人・公式): github.com/emilkowalski/skills(README、skills/review-animations/SKILL.md)· github.com/LottieFiles/motion-design-skill · github.com/vercel-labs/open-agents(web-animation-design/SKILL.md)
- skill(第三者): github.com/Schmandarine/web-motion-skill · github.com/pandazki/pneuma-skills/pull/149 · github.com/haidrrrry/claude-remotion-skill · github.com/kylezantos/design-motion-principles · github.com/nano-step/rive-playground
- 論文: arxiv.org/abs/2605.15585(OmniManim)· 2604.18364(ManimBench/RITL)· 2606.28593(Animation2Code)· 2605.19382(PRISM)· 2604.26148(AniMINT)· 2405.07065(LogoMotion)· 2402.06071(Keyframer)· 2605.30174(LiveSVG、参考)· CVPR 2025 Video-Bench
- 未検証(本体取得不可): lobehub.com/mcp/saralobo-rive-console-mcp(403)· motionlens.dev(402)
