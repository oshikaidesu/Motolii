# 現行 GUI の装置の棚卸し(2026-09-19)

対象: `motolii/ui/lib`(HEAD `6cfb7b8ff` + 作業樹の未 commit 分)と、それを説明する docs(`ui-visual-language.md`、`stage5/README.md`・`workspace.json`・`product-contract.md`・`panel-placement.md`、`wiki/window.md`、`decision-index.md` の 2026-09 行、`gesture-tests.md`、`reviews/2026-09-19-flutter-audit.md`)。読むだけ、コードは触っていない。作り直しの前に「意図して置いた物」を全部名指ししておくための台帳。行番号は作業樹の今の状態。

## 要約 5 行

1. 装置は **「1 本の出所 → 鍵ごとの購読 → 掴む・見せる・確定」の 3 層**に整理できる。theme/metrics + lint が見た目の出所、`DocumentSlice`・`Picked`・row pulse が「軽さ」の出所、`EditorPreviewQueue` + `previewProperties/commitPreview/cancelPreview` が「触ると先に絵が動く」の出所。
2. **利用者裁定に紐づく装置が 40 超**(2026-09-02〜09-17)。Animate 1 本・A キー・補色の差し色、Advanced 畳み・点、OP-1 の主役 4 つ、Ableton の dice、机 = レンズ、色の原子 = 見本 1 個、Fonts/Colors = 棚、Live 12 の Filter View / Collections、Stage = Boxcam の Original Comp、Sequence = ゴースト、GSAP の ease 文字列。
3. **文書が言って code に無い物**は少ないが重要: 高コントラスト・Light・font/UI 言語の User settings(視覚言語)、reduce motion の全面適用(Ease だけ)、Stage の 2.5D「札」の絵(Inspector の Space 3 択のみ)、Web の Pinterest API、Timeline の inertia は「同じ慣性処理」の文言どおりだが ruler 以外の検証が薄い。
4. **code にあって文書に無い物**も多い: `EditorInk`(ThemeExtension、audit は「無し」と書いたが今は在る)、`raw_color` lint(未 commit)、`FittedName` の読み流し、`_SampleShelf` の LIFO/呼吸、`ViewportMotion` の 8000 px/s 上限、numeric field の Figma 段ばしご、`EditorLamp` の 4 状態、History の rail、Files 棚の crumbs、`fontFacts` 札、`owner` による well の据え置き。
5. 音・触覚・隠し要素は **無い**(grep 0 件)。遊びは「Ableton の dice」(`_roll`)と「OP-1 の主役 4 つ」(hero)だけで、どちらも裁定(2026-09-08)の写し。

## 装置の表

列: 名前(code/docs の呼び名)・効き(利用者に何をするか)・場所(file:line)・決定/日付・理由(docs の引用 ≤25 語)。path は `motolii/ui/lib/` からの相対。

### A. 出所を 1 本にする(見た目)

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| `EditorMetrics` | 行 20 / control 24 / 文字 11 / 余白 s2〜s48 の定規 1 枚。全 panel が名前で寸法を読む(681 箇所) | foundation/metrics.dart:7-17 | 2026-09-08(decision 597) | 「寸法は `EditorMetrics`、色は `EditorTheme` の名前だけ」product-contract |
| off-scale token `s5…s280`(負債台帳) | 2026-09-06 に panel から拾った生値を名前にして scale の中へ。「値を変えて試し、使われなくなったら名前を消す」 | foundation/metrics.dart:19-26 | 2026-09-06 | 「Each is a snap candidate」 |
| `raw_dimension` lint + `use_metric` quick fix | 寸法に生の数を書くと IDE と `check.dart` が警告。0/.5/1 だけ通す。fix が同値の token に置換 | tool/motolii_lints/lib/src/raw_dimension.dart:10-36, use_metric.dart:9-13 | 2026-09-06 commit 655490d | 「生数字を IDE と test で拒否」flutter-audit |
| `raw_color` lint | `Color(0x…)` を theme 外で拒否。計算色(`fromARGB`・`\|`)は通す | tool/motolii_lints/lib/src/raw_color.dart:10-45(**未 commit**) | 2026-09-19(audit の「色にも同じ型で」を受けた物) | 「寸法には lint があり色には無い、の差」flutter-audit |
| `EditorTheme` + `ThemeData.dark().copyWith` 1 本 | menu / menuButton / slider / dialog / textButton / input の component theme を 1 度だけ書き、panel は素の部品を置く | foundation/theme.dart:151, :267-330 | 2026-09-08(597) | 「トンマナを構造で強制 → Flutter 本体の ThemeData に一本化」 |
| `EditorInk`(ThemeExtension) | 面ごとの ink(headerInk・easePaper 等)を extension で持つ | foundation/theme.dart:14 | 記録なし(audit 2026-09-19 は「ThemeExtension 無し」と書いており、それ以後) | — |
| M3 state layer の「持ち上がり」 | hover 8% / pressed 10% / chosen 16%。灰の階段 app→panel→raised→hover は app を 8% ずつ持ち上げた値。選択は枠で囲まず面が上がる | foundation/theme.dart:152-164(`hoverLift/pressedLift/draggedLift`)、reference/ui-lift.tsv | 2026-09-10(621) | 「新しい数字を目分量で置かない」 |
| `keyAccent`(補色の差し色) | Animate 中は key lamp・Animate スイッチ・playhead・キーだけ accent を色相の補色へ振る | foundation/theme.dart:173-176 | 2026-09-10(620) | 「窓全体の反転は選択の色と混線し注意が散るので却下」 |
| 操作中 panel だけの明灰枠 | `#ACACAC` 1px、非アクティブには付けない | workspace/workspace_view.dart(`_Leaf` の active 枠) | 2026-09-05〜06 product-contract | 「非アクティブのパネルへ同じ枠を付けない」 |
| `EditorChoice`(MenuAnchor) | `DropdownButton` の行高 48 を避け、行高 `EditorMetrics.row` の選択肢 | foundation/panel_controls.dart:125 | 2026-09-08(597) | 「DropdownButton は行高 48 未満を拒むので使わず」 |
| `EditorTooltip` / `EditorButton` / `EditorSection` / `EditorFold` / `EditorSwitch` / `EditorCard` / `EditorBar` | 共通部品。tooltip は hover 領域 + long-press を 1 つに畳む | foundation/theme.dart:393-576、panel_controls.dart:89-284, 983, 1377 | — | 「同じ役割は同じ surface、radius、spacing、stroke」ui-visual-language |
| tabular figures | 数値欄・readout は等幅数字で幅が揺れない | panel_controls.dart:743, :830 | ui-visual-language「数値と timecode の文字」 | 「値の変化で幅が揺れないこと」 |
| 窓の文字は英語、値だけ文字 | 全 label 英語。道具は形(◇◆ M S L ↳、glyph) | AGENTS.md:14、docs/wiki/window.md:32、実装は全 panel | 憲法 | 「値が意味を持つ物だけが文字で、道具は形で見せる」 |
| `EditorScale` / `EditorScaledViewport` / Settings の ± | 窓の文字の倍率は Settings だけ。⌘0/1/=/− は視点 | panel_controls.dart:1719-1755、input/editor_shortcuts.dart:97-110 | 2026-09-03(547) | 「AE・Figma・Nuke・Blender の全先例でこの指はビューの倍率」 |

### B. 軽さの出所(購読と導出)

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| `EditorSession.document` 1 本 + `take`(変わった鍵だけ差し替え) | 返信のたび全鍵を `sameValue` で深比較し、動いた鍵だけ通知 | session/editor_session.dart:56, :129-166, :33-51 | — | 「Flutter の ValueListenable 型そのもの」flutter-audit 残す物 |
| `DocumentSlice`(鍵ごとの購読、`derived` 指紋) | panel は読む鍵を名指し。Desk は `deskIdentity` の文字列が変わる時だけ build | editor_session.dart:13-30; desk.dart:26, :232; inspector.dart:85-99; timeline.dart:951-965; stage.dart:991 | 2026-09-12(23) | 「導出は build の外… slice は initState で listen」 |
| `Picked<T>`(選択は信号) | 数百 tile の棚で pick が触った 2 枚だけ描き直す。browser・notes・inspector の fold・blend が共用 | panel_controls.dart:8-30; browser.dart:185; notes_desk.dart:28; inspector.dart:1529 | 2026-09-12(23) | 「選択・ホバーは信号(`ValueNotifier` を見る側が自分の出入りだけで描き直す)」 |
| Inspector row pulse(`_pulse` + `_Live`) | 行ごとに `ValueNotifier` を 1 本。status が来たら動いた行の well だけ rebuild、枠は `_stampShape` が変わった時だけ。**視覚の「息」ではなく配線**(点滅・アニメ無し) | inspector.dart:78-83, :176-184, :197, :2096-2108 | 2026-09-12(23) | 「a number that changes rebuilds its own well and nothing else」 |
| `_stampDeclared`(形は「何か」だけ) | 枠の指紋に値を入れない。`'list n'`/`'number'` に潰す | inspector.dart:248 | — | 「what it is, never what it says」 |
| 押下は arena の下(`Listener.onPointerDown`) | double-tap と同居する tap の 100 ms 遅れを避け、押した frame に枠 | browser/tile.dart:152; timeline 概観 | 2026-09-12(23) | 「押した瞬間に選ぶは `Listener.onPointerDown`」 |
| `shouldRepaint` は field 比較 | painter 8 本全部 `identical`→`sameValue`/`listEquals` | timeline.dart:2003-2024; stage.dart:1540; depth_desk.dart:372; ease painter 3 本 | 2026-09-12(23) | 「`shouldRepaint => true` は書かない」 |
| `panel_layout_cost_test`(定規が先) | 素材 500 で 1 クリック builds ≤ 100 / layouts ≤ 20 | test/panel_layout_cost_test.dart, panel_rebuild_scope_test.dart | 2026-09-12(23) | 「予算は測った数で置き、上げるときは理由を予算のコメントに」 |
| `_SampleShelf`(native 見本の待ち行列) | 画面にある見本だけ、scroll が落ち着いてから 3 枚ずつ、1 frame 呼吸。LIFO(最新の願い先)、LRU 2048 | panels/native_visual_sample.dart:10-24, :63-83 | — | 「waits for the scroll to settle, and lets go of the worker between small bursts」 |
| `_serial` / generation + `Ticker` 再生 | 命令を 1 本に直列化、render は 1 つ飛行、polling 無し | editor_session.dart:367-390, :585-646 | — | flutter-audit 残す物 |
| `owner`(well の据え置き) | 選択が変わっても同じ位置の well は render object を保ち、他人の下書きは捨てる | panel_controls.dart:431-437 | — | 「a draft left open belongs to the owner it was typed for」 |
| Desk `_inside`(作業中は追従しない) | Desk の中で触っている間は選択が動いても道具を奪わない | desk.dart:89-101, :189-194 | 2026-09-06 panel-placement | 「selection changes cannot unmount a Depth interaction」 |
| `_drawer` 名だけ見る枠 | 名前が同じ間は枠も中の道具も build しない | desk.dart:59 | — | (code 内コメントのみ) |

### C. 掴む → 見せる → 確定(drag→preview→commit)

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| `previewProperties` → `commitPreview` / `cancelPreview` | 掴んでいる間は下書き、離した時に 1 undo、Esc・窓離脱・app 非活性で取り消し | inspector.dart:408-500; ease_desk.dart:379-429; depth_desk.dart:28-56; gradient_inspector.dart:42-62; colors_shelf.dart:452-585; rich_text_editor.dart:147-315 | 2026-09-01〜02(gesture-tests §2) | 「書かずに手放すので、掴む前の値がそのまま残る」 |
| `EditorPreviewQueue`(最新の願いが勝つ) | pointer move ごとに送らず、飛行中は最新 1 つだけ保持 | panel_controls.dart:1165; ease_desk.dart:156; blend_panel.dart:144; depth_desk.dart:30 | — | 「a sweep leaves one query with the port, not one per pointer move」 |
| `EditorDragSession` mixin | 「drag → preview → commit, once, for every continuous control」。field・dial・pad・gradient stop が mix in。`watchesWindow`(窓を失っても field だけは取り消さない)、`discardsPreview`(bar から外した stop) | panel_controls.dart:1092-1163; gradient_inspector.dart:42-56 | audit 2026-09-19 判定 4 → 作業樹で統合済 | 「1 つ直せば同族全部」AGENTS |
| tooltip は全 off(`EditorApp.noHover`) | hover で何も出ない。`EditorTooltip` は tooltip が on の時だけ hover 領域・long-press・semantics を持つ | app/editor_app.dart:10-12; theme.dart:391-395; test/hover_and_tap_target_test.dart | — | 「Hovering a control shows nothing: every Tooltip below is off」 |
| 掴んでいる間に見せた値 = 確定値 | 離す時の修飾は見ない | panel_controls.dart(`_end`)、stage `_finish` | 2026-09-03(550) | 「離す直前に Shift を放すと見た目と違う値が書かれていた」 |
| touch slop 3 px、Esc = 取り消し、focus loss = 取り消し | 押しただけで値が変わらない。宙に浮く掴みを作らない | stage.dart, timeline.dart, panel_controls.dart:1160-1163; docs/gesture-tests.md | 2026-09-01〜02 | 「認識器は Recognized / Failed / Cancelled / Ended のどれかに必ず到達する」 |
| drag は相対、打ち込みは絶対(複数選択) | 複数層を掴むと各層が自分の差分を保つ | inspector.dart:406-437 | 2026-09-02〜03(51, 61) | 「相対 scrub・絶対入力・一手一 Undo を維持」 |
| Blend hover preview(90 ms、latest wins、1 click = 1 undo) | tile を撫でると Stage が変わる。`setAttrs` が preview を先に消すので履歴に残らない | blend_panel.dart:144-221 | 2026-09-02(inspector-and-desk §4) | 「一覧を hover するだけで絵が変わる」 |
| Anchor grid の hover preview | 9 点を撫でると Stage に anchor の仮線 | inspector.dart:1148-1161(`c.anchorPreview`) | — | — |
| Sequence の下書き(`_previewSequence`) | 複数層でカーブを掴むと Stage のゴーストがその場で動く、放して 1 undo | ease_desk.dart:219-290 | 2026-09-07(589 ゴースト)+ 自走 | 「AE の Sequence Layers を in 点を動かさず(壊さず)やる」 |
| `▶ Preview motion`(1.2 s の玉)/ hover 試聴(`_peek`) | 欄の意味を言葉無しで見せる。Reduce Motion では終点へ跳ぶ | ease_desk.dart:301, :470-492 | 2026-09-08(602 規則 4)、09-09(617) | 「hover は native サンプルの一回再生で書き込み 0」 |

### D. 数値の顔(Inspector と numeric field)

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| `EditorNumericField`(≈520 行の 1 部品) | drag scrub、Figma の段ばしご(×10/×1/×0.1/×0.01 を縦の移動で)、Shift ×10、左押しホイール 1 刻み、trackpad 横 2 本指、300 ms settle、点線下線 = 触れる数字、既定値で静かな ink | panel_controls.dart:383-900(:572-580 wheel、:627-638 ladder、:822-846 ink) | 2026-09-08(602 規則 2・3) | 「触れる数字は触れる顔(Tangle)」 |
| `TrackStyle`(fill / level / steps / ruler)+ `defaultValue` の目盛 + 中央ゼロ | 欄が自分の効き(量・閾値・段・長さ)を背に描く。既定値に tick | panel_controls.dart:1631(`_TrackPainter`); inspector_property_style.dart:142-160 | 2026-09-08(602 規則 3・5) | 「基準と尺度を必ず添える」 |
| `_Character` 表(spatial/amount/time/count/seed/angle) | 宣言名(Blender subtype 語彙が優先)から色相・単位・control を決める。新 Vism が宣言だけで顔を得る | inspector_property_style.dart:26-125 | 2026-09-08(601) | 「One table, so a new Vism gets its glyph, unit and control the moment it is declared」 |
| hero(主役 4 つ) | 効果ごと `HERO` 宣言か先頭 4 つを大きく前に、残りは Advanced | inspector.dart:1401-1429, :1698-1708 | 2026-09-08(602 規則 1) | 「主役は 4 つまで(OP-1)」 |
| Advanced 畳み + 点 | 一見変化の無い欄を畳み、中に非既定かキーがあれば ▸ の脇に点。開閉は層×効果で記憶 | inspector.dart:1311, :1390-1400, :1529-1548 | 2026-09-07(569) | 「畳んだまま『何か入っている』が分かる」 |
| dice `_roll` / seed の 🎲 / `_rest`(戻す) | 範囲内 ±20% で振る、seed は全振り、1 undo。全部既定へ | inspector.dart:633, :661, :1834-1857 | 2026-09-08(602 規則 6) | 「束ねる・振る・戻す(Ableton)」 |
| `EditorLamp`(◇◆ の 4 状態) | unlit = キー無し / lit = 有り / bright = 今キー / ember = Animate 無しで触った。unlit は hover でだけ出る。押す = 打つ/消す | panel_controls.dart:890-980; inspector.dart:562-569 | 2026-09-07(567) | 「欄ごとの ◇ は押せない印 → 今は押せる灯」(Ableton の慣習) |
| Animate 1 本(識別 bar)+ A キー + Key the start too | キーを打つ入口は Animate だけ。A で入切(Shift+A = Anchor)。入れた時刻を覚え、別時刻の最初の触りで起点・終点 2 キー | inspector.dart:1938-1947; input/editor_shortcuts.dart:75-95, :163-172; panel_settings.dart:56-68 | 2026-09-07(567)、09-10(620) | 「値の面と時間の面・印・レーン・書き込みを切り離している」 |
| card の固定席 | Transform → World → 物固有。選択で Position が上下しない | inspector.dart:1988-1992 | 2026-09-13(633) | 「共通の欄は固定席、物に固有の欄はその下」 |
| `_fit` 3 列 / `InspectorCell` 88〜200 / `EditorZoomBar` | 狭いと語を glyph に、広いと 1 行 1 数。幅は desk key `inspectorCell` | inspector.dart:742-776, :2081-2114 | 2026-09-08(599) | 「値を収める秩序はあるが、人が触って編集する寸法と手掛かりが弱い」への答え |
| Space 2D / 2.5D / 3D(World card) | 投影の札は利用者が押す 3 択。Depth 行は 2D で隠す。切替は姿を保って 1 undo | inspector.dart:1113-1115, :1173-1190; panel_settings.dart:83-96(New layers 既定) | 2026-09-06(565)、09-11(27)、09-12(24) | 「ユーザーは必ず意図で札を選ぶ」= 記憶「札は自動で変えない」 |
| World の switch(blocksLight / environment / ghost / clipToBelow / Freeze) | 自分を説明する label。Freeze は旗でなく状態(DAW の Freeze Track)、凍ると効果が 0.45 で触れない | inspector.dart:1228-1302, :2037-2074 | 2026-09-10(624)、09-13(630) | 「Freeze は投影の前をコマごとに固めて 3D のまま」 |
| 効果の頭(順 = パイプライン、glyph 7 → 1) | drag で順を変える、7 つ並んでいた glyph は `more_horiz` の裏へ | inspector.dart:1470-1515 | — | 「The order is the pipeline」 |
| `_effectMenu` の散文 label | "Throw every number within its reach" / "Back to where the numbers rest" / "Apply earlier/later" | inspector.dart:1315 | — | — |
| Text card は鍵の打てる値だけ | 書体・文字群・揃えは Fonts 棚。px は Scale で動かす | inspector.dart:907-957 | 2026-09-13(637, 638) | 「文字の大きさ(px)は鍵を打つ値ではない」 |
| Parent = ピックウィップ、Matte = 直上クリッピング | 関係の行は選択肢でなく手つき | inspector.dart:815, :1585; timeline の ↳ | 2026-09-02(61) | 「既に手慣れた作法へ認知負荷を預ける」 |

### E. Desk 家族

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| `DeskPanel`(机 = 焦点を読むレンズ) | 誰にも呼ばれず、選択・キー・焦点で Ease / Depth / Blend / History を出す。★ で idle 既定、Tools は手動 | panels/desk.dart:10-216; foundation/panel_catalog.dart:36, :81-120 | 2026-09-02(62)、09-06(44, 45) | 「机は常設の一枚で、Document を覗くレンズ。誰にも呼ばれず」 |
| 引き出しは浮かせず机の中 | 開けた間だけ机が広がる。右下に浮かす案は実窓で却下 | desk.dart(inline)、product-contract | 2026-09-02(60, 62) | 「浮かせる層は座標測りが要るため却下」 |
| Settings で Desk ↔ Tab ↔ Window ↔ Hidden | 補助道具を常設へ昇格、戻せる。Desk は全 panel の管理者でない | panel_settings.dart:107-161; docs/stage5/panel-placement.md | 2026-09-06 | 「Desk is not the owner or manager of all panels」 |
| **Ease desk**: 9 種の kind(Bounce / Elastic / Cyclic / Random / Steps / ElasticSteps…)+ 1 文の意味 | AE の graph editor に無い: 曲線ごとの英語 1 文、正方形の固定 plot、overshoot 枠、preset の絵 | ease_desk.dart:73-82, :680, :1029-1065 | 2026-07-10(AM 式高度イージング)、09-09(617) | 「正方形の編集グラフ+名前/短い説明/動きのプレビュー+読める標準 9 種」 |
| Ease の区間帯 `EaseIntervalPainter` + 曲線上の playhead | 「どの区間か」を帯で、「区間の中のどこか」を縦線と玉で。外なら矢印 | ease_desk.dart:1450-1527, :1310-1344, :1131-1146 | — | 「帯は『どの区間』、縦線は『区間の中のどこ』」 |
| Use for new keys / Copy curve / Save preset / Mixed / Read only | 新しいキーの形を先に決める。既定 Easy Ease。preset は人の物 | ease_desk.dart:977-999, :531-556 | 2026-09-10(622) | 「キーを打つたびに区間を選んで ease を打つ手数を消す」 |
| 生まれた対の自動選択(`born_spans`)+ Shift+Cmd+D(Reselect) | Animate From で対が出来たらその 2 キーが選ばれ Ease が出る | native port.rs; editor_shortcuts.dart:66 | 2026-09-10(622) | 「Desk を自動で開くのは流れを切る」→ 追従だけ |
| GSAP の ease 文字列 | `power2.out` / `back.out(1.7)` / `steps(5)` を既存の型への名前の表で受ける(評価器は足さない) | doc 側(motolii-doc)、Timeline/Ease は既存 kind を表示 | 2026-09-17(664) | 「一度足した `Interp::Gsap` は取り消し」 |
| `_EaseIcon`(Ink 無し) | Reduce Motion で走る animation が残らない | ease_desk.dart:24-58 | — | 「静かな道具の粒」 |
| **Depth desk**(上から見た X/Z の平面図) | 注視点を原点、camera + frustum + 全層の点。eye を掴むと yaw/distance、層を掴むと親空間で移動。目盛は中身から | depth_desk.dart:19-42, :140-221, :307-372 | 2026-09-10(32 カメラ POI) | 「author するのは注視点・orbit・distance で、eye は導出」 |
| **Blend desk**(見本の格子) | tile が見本そのもの(層の色 × `blend_preview.rs` の地)、W3C 順、Stencil/Silhouette alpha | blend_panel.dart:14-52 | 2026-09-02(§4)、09-15(657) | 「文字で選ばない。机のサムネイル格子」 |
| **History**(クリスタ一覧 + Git Graph の縦線) | 点を押すと `historyGoto`、保存・起動・異常終了は同じ列に別印、到達済みは明線・redo 尾は暗 | history_records.dart:8-10, :37, :199-258 | 2026-09-09(619) | 「クリスタの一覧 + VS Code Git Graph の縦一本線」 |
| **Notes**(OneNote 型の canvas) | 空所を押して書く、上の取っ手で動かす、角で伸ばす、画像 drop/paste、reference block(層 · start–end を押すと選択+seek) | notes_desk.dart:98-148, :391-409, :531-559, :646-653 | 2026-09-06(45) product-contract | 「OneNote note-container interaction」 |
| **Web**(URL を覚えて外のブラウザ) | Pinterest 既定、`openWeb` | web_panel.dart:16-31 | 2026-09-06(45) | 「Web は通常ブラウザで開く」 |

### F. Browser(枠 + 棚)

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| `BrowserShelf` / `BrowserHost`(棚 1 tab = 1 file) | 枠が tab・検索・rail・grid・選択・鍵を持ち、棚は口だけで話す。tab を足す = 棚を足す | browser/shelf.dart:6-50, :98-200; browser.dart:46 | 2026-09-12(31) | 「Browser は枠 1 つ + 棚 1 tab = 1 file」 |
| Grid / List / Thumbnails + `BrowserSize`(72〜240)+ 件数 label | 3 表示、tile は 1 枚の絵として拡縮(文字だけ床)、件数は「選択 / 表示 / 全部」 | browser.dart:732-754, :916-998, :1108; browser/parts.dart:67 | 2026-09-09 browser-ui-phases 3.2 | 「thumbnail-only / thumbnail+name / list を選択可能に」visual-language |
| tile の解剖(1 行 caption、format badge、印は絵の上、枠は選択だけ) | 名前が幅を独占、拡張子は badge、使用中は淡点・欠落は警告、枠 = 選択 | browser/tile.dart:41-108 | 2026-09-09 phases 3.2 | 「selection and use cannot be read for each other」 |
| `FittedName`(読み流し) | 収まらない名前は縮めず、hover で 90 px/s で左へ流れて端を見せ、離れると戻る | browser/parts.dart:153-252 | — | 「so nothing is ever shrunk to fit」 |
| `_ShapeMark` / `ShapeMark` | 見本の無い mesh を file 名(sphere / torus / cube…)で線画に | browser/create_shelf.dart:124-234; media_shelf.dart:401 | 2026-09-09 phases 3.2 | 「torus.obj と sphere.obj が同じ icon だった」 |
| **Create** = 名詞のレシピ | Text・Rectangle・Rounded Rectangle・Ellipse・Star・Polygon・Null・Camera・Particles・Stage・Line・Bezier + Rust の同梱 3D | create_shelf.dart:16-71 | 2026-09-10(28) | 「Create は名詞のレシピ、輪郭を変える演算は効果の棚の Path 族」 |
| **Effects** = 札は画像 1 枚(VST3 snapshot)、宣言から群 | Seat / Applies to / Time / Inputs / Parameters / Origin、Reload + 1 行の失敗理由、選んだ組を一度に掛ける | effects_shelf.dart:13-192 | 2026-09-12(30) | 「効果の札は普通の画像 1 枚。VST3 の Plug-in Snapshot の型」 |
| **Media** = 絵だけで開く、Timeline へ drag、同梱 HDRI | 既定 Thumbnails、行へ落とすとその位置(`aimRowDrop` 共用)、Relink・Extract palette、drop hint | media_shelf.dart:14-47, :62-100, :165-300 | 2026-09-07(573, 593)、09-01(67) | 「メディアインポートは普通の動画ソフトと一緒」 |
| **Files** = 実 folder の窓(AEViewer) | Home/Desktop/…の rail、crumbs、Back/Forward/Up、同期 read(1 ms) | files_shelf.dart:11-60, :159-321 | 2026-09-09 phases 3 | 「the way AEViewer sits beside AE」 |
| **Colors** = 常設の輪 + 色の原子 | 輪は Browser に常設し焦点に付いて行く。値の原子 = 見本 1 個(押すと輪が向く、hex も輪も持たない)。棚 = Used here / Saved / Starter、From image(k-means)、eyedropper(Esc)、Square/Triangle、alpha | colors_shelf.dart:20-44, :155-220, :412-585, :639-736, :927-982; foundation/color_field.dart:49 | 2026-09-03(544)、09-13(634) | 「カラーが机に出ると視線がごちゃつく」 |
| `_FillDefinitions`(自分の stop で描いた tile) | solid/linear/radial/angular/diamond と Blend(rgb / linear / oklab 既定 / oklch)を、今の色で描いて見せてから選ぶ | colors_shelf.dart:785-906 | 2026-09-13(634) | 「モードと Blend は書類にハードで書く定義なので棚(1 手で当たる)」 |
| `GradientInspector`(bar 1 本) | stop のつまみ = 見本、押すと輪、drag で位置、外へ drag で消す(32 px、.35 で予告)、bar を押すと増える、32 個上限 | gradient_inspector.dart:11-14, :38-56, :100-225 | 2026-09-13(634) | 「Inspector の Fill card は bar 1 本」 |
| `ColorWheel`(hit と絵が同じ式) | 7 stop の sweep、0° 上、三角/四角、白黒で色相を失わない(`rememberedHue`) | browser/color_wheel.dart:11-209; colors_shelf.dart:497-503 | — | 「Sampling and painting both read this, so they cannot disagree」 |
| **Fonts** = 棚(Colors と同じ型) | 見本 = Flutter 自身の文字描画で層の本文 1 行(無選択なら書体名)。click = 着せる、double-click = その書体で文字層を 1 手。`fontFacts` の札(A あ 漢 한 Я ع・N styles・axes・Mono・Color)。scope(Hiragana/Katakana/Kanji/Latin/Upper/Lower)+ 文字群ごとの大きさ + `_Justify` 3 マス | fonts_shelf.dart:16-49, :92-120, :178-206, :250-300, :307-444 | 2026-09-07(591)、09-13(637, 639, 642, 643) | 「書体の見本は Flutter 自身の文字描画」「書体そのものが持つ事実を小さな札で」 |
| Live 12 の Filter View / Collections / Labels / Quick Tags | 群は縦に積み 1 行に札を流す、群内 OR・群間 AND、Cmd で足す、群の 3 種(宣言/実値/範囲 `min-max`)、Collections 7 色(1〜7 / 0、行を掴んで rail へ drop、double-click で改名)、右 menu に 7 行 + Remove、Label = 保存した絞り、Cmd+E | browser/filters.dart:13-160, :232-430, :575-871; browser.dart:81-185, :425-489, :999-1066 | 2026-09-13(644, 645, 646, 647)、09-14(649) | 「左 rail を Filter View にはしない。Collections やユーザー意図分類にする、Ableton がそうだから」 |
| 取り込みは Media へ飛んで選ぶ(`_revealImported`) | File > Import / Finder drop の後、絞りを外して新しい物を選択 | browser.dart:342 | 2026-09-01(67) | 「取り込んだ物は Media に居る。開いて、絞り込みを外して、選んでおく」 |
| rail の折り畳み(48 以下で縦 tab、grip の double-tap 0↔96) | 狭くしても今の分類が読める | browser.dart:674-713 | — | — |
| `ValueKey('browser:…')` の試験の継ぎ目 | 全操作要素に安定 key | browser 全域、ease(`ease-*`) | — | (慣習、文書なし) |

### G. Stage

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| Stage / Camera の 2 tab(Boxcam) | Stage = 既定カメラで置いた comp そのもの(作中カメラは箱だけ動かす)、Camera = 箱の中身 = 出力。両方同時に生きる。見せる tab だけ native に窓を渡す | stage.dart:15-17(`view`)、:89; docs/stage5/product-contract.md「User Stage / Camera View」 | 2026-09-12(627)、09-07(596) | 「意味を発明すんな、AE と Boxcam が欲しいだけだ」 |
| 枠の外はグレーの被せ(切らない) | tab の大きさがそのまま絵、Output Frame は参照 | stage.dart(scrim)、Settings「枠の外の暗さ」 | 2026-09-12(627)、visual-language G0-6 画面 4 | 「枠外を不透明塗潰しせず半透明 scrim で見せ」 |
| Fit / 100% / 段階(⌘0 / ⌘1 / ⌘= / ⌘−) | 視点の倍率。Fit は作業範囲(Stage 層)へ | stage.dart:126-136, :278; editor_shortcuts.dart:97-110 | 2026-09-03(547)、09-09(604) | 「Fit moves the observer to the active working area」 |
| ホイール = 滑る、⌘/Ctrl+ホイール = 寄る、中ボタン = パン、右 = orbit | trackpad 2 本指で倍率が飛ばない | stage.dart(gesture)、docs decision 548 | 2026-09-03(548)、09-07(596) | 「パンのつもりで倍率が飛んでいた」 |
| 籠(cage)= mask の 4 角、線無し | 2D/2.5D は `_handles` の平面籠(body / spatial / rotation、rotation は 7 px 許容)、3D は native が当たり判定した三角形そのもの(`_SpatialMesh`、identity で cache) | stage.dart:155-195, :567-600 | 2026-09-11(625) | 「線は要らない、縁取り effect の邪魔」 |
| Camera gizmo(Blender 式) | 正面 = frustum と comp 面の交わりの箱(辺・角・取っ手)、Depth desk で四角錐 | stage.dart:338-465(`_selectedCamera`, `_onCameraEdge`, `_moveCamera`) | 2026-09-13(629) | 「回転に耐えられるようカメラギズモを Blender 式に」 |
| Stage 層(作業範囲 4 辺)+ Extend switch | comp を動かさず作業範囲だけ広げる。Extend を入れて Stage 層が無ければ作る | stage footer; create_shelf.dart:52 | 2026-09-09(604, 605) | 「出力の比率は作る前に決めなくてよい」 |
| marquee、Shift/⌘ の選択、層のナッジ Alt+矢印(Shift 10px) | 素の矢印は時間のまま | stage.dart:501, :688; editor_shortcuts.dart:128-145 | 2026-09-03(549) | 「素の矢印は NLE の指(コマ送り)」 |
| `_state`(document + rendered を 1 回) | hover や scrub frame で overlay を作り直さない | stage.dart:28-31 | 2026-09-12(23) | — |
| 2.5D の札(投影)は Stage で推定しない | Stage は `projection` を読むだけ(:574-575, :194)。宣言は Inspector の Space、既定は Settings の New layers | stage.dart:160, :190-194 | 2026-09-12(24) | 記憶「2D/2.5D/3D を操作から推定して切り替える UX は禁止」 |
| Texture(IOSurface → Metal → FlutterTexture、コピー無し) | 絵の受け渡しに Dart のコピーが無い | stage.dart:1200; native/src/lib.rs:152-180; macos MainFlutterWindow.swift:199-235 | Stage 5 技術境界 | flutter-audit 残す物 |

### H. Timeline

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| `_TrackRow` / `_LaneContainer` / `_LaneLayout`(寸法だけで建つ帯) | 行は widget で積まず CustomPaint、`_relane()` は LayoutBuilder の外 | timeline_layout.dart:3-80; timeline.dart:1254 | 2026-09-12(23) | 「帯は寸法だけで建つ」 |
| 実レーンだけ横線、名前欄まで、空領域は縦グリッド、行 20 px | AE の詰め込みでなく Ableton の密度 | timeline painter:1377-2020 | product-contract「Timeline と階層」 | 「実レーンだけに横線を引き、名前欄まで境界を通す」 |
| ◇/◆ の twirl、M S L ↳ | ◇ = 全層共通のキーレーン開閉、M = mute、S = solo、L = lock、↳ = 直上へクリップ | timeline.dart:1905-1930 | product-contract | 「意味のある記号を装飾の丸へ置換しない」 |
| キーの間の線(Linear 破線 / 形が付けば実線 / 両端選択で差し色) | 線を押せば両端を選ぶ、Shift で足す。ease の履歴一覧は作らない | timeline.dart:1716-1740 | 2026-09-10(622) | 「見せる・選ぶは Timeline の線に委ねる」 |
| 畳んだ行の `allKeys`(AE の集約キー) | 層行に全 property/effect のキーを出し、まとめて動かす | timeline_layout.dart:22-30 | product-contract | 「本文キー、個別プロパティキー、集約キー選択は別の対象」 |
| Marker(`addMarker`、本文は `Marker.body`) | Timeline は名前だけ、Notes/Desk は全文。Export の「Marker to marker」 | timeline.dart:1379-1381, :1787-1793; export_controls.dart:34-68 | 2026-09-02(62) | 「書き置きはマーカーの本文」 |
| `ViewportMotion`(同じ慣性処理、8000 px/s 上限) | 通常移動・scroll zoom・pinch が同じ変換と release を通る。離した後減速、次の操作で止まる | input/viewport_motion.dart:7-90; timeline.dart:37-38; test/viewport_motion_test.dart | product-contract「入力と反復」 | 「変換と release の処理を別々に追加して慣性を落とさない」 |
| 名前欄の幅 drag、階層で伸びる、drag で並べ替え(挿入線 / 囲い) | 離した時だけ確定、自分の子孫へは拒否 | timeline.dart(:428-433 locked 判定含む) | product-contract | 「境界は挿入線、グループ中央は中へ入れる囲い」 |
| 尺の無い物の既定長 = 見えている幅の 3/4 | 終端 trim が画面内に見える | native create; timeline 表示 | 2026-09-07(579) | 「comp の終わりまで伸ばさず」 |
| preview は Ableton(止めるのは人だけ) | 尺の終わりで止めず、ループせず、越えて進む | editor_session.dart:585-646 | 2026-09-07(595) | 「止めるのは人だけ」 |
| 速度欄(負 = 逆再生) | 書類は受ける、UI の確認は pending | workspace.json pending | — | — |

### I. 窓・入力・記録

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| `panelCatalog` / `PanelSpec` / `Extent`(panel を data で) | 名前・icon・使える寸法・drawer 可否を 1 表に。registry が widget を作る | foundation/panel_catalog.dart:1-130; panels/registry.dart | 2026-09-06 panel-placement | 「The common catalog owns identities, icons, usable sizes and drawer eligibility」 |
| `_Split` / `_Leaf` の dock、仕切り drag、tab drag、別窓 | 4 置き場、tab を掴んで移す、外へ出せば別窓、配置は次回も残る | workspace/layout.dart:19-70; workspace/workspace_view.dart | 2026-09-12(23 全体 layout 867→626) | 「タブを掴んで別の置き場へ落とせば移る」wiki/window |
| dock json の移行(`Test`→`Inspector`、Files 棚の追加、Camera tab の追加、`offset` 無しの旧 layout) | 古い配置を読んでも panel が消えない | workspace/layout.dart:98-123; test/dock_extent_test.dart | — | (code のみ) |
| `requiresPause` / `requiresRender` の表 | 編集で再生を止めるのは書類の差し替えだけ、再描画が要らない op は飛ばす | bridge/protocol.dart:89-93 | 2026-09-07(595) | 「playback behaves like Ableton — keeps looping and stays editable」 |
| `EditorShortcuts` 1 表 | Esc(掴み中は取り消し、それ以外は選択解除)、A / Shift+A、⌘D 複製、Shift+⌘D Reselect、⌘0/1/=/−、Alt+矢印 | input/editor_shortcuts.dart:6-208 | 2026-09-03(547-549)、09-10(620, 622) | 「入力欄のフォーカスと編集ショートカットを取り合わない」 |
| 欄は許可制(`EditorDraftField`) | 押すまで無く、Enter/Esc/外/窓離脱で消える。`pendingEditors` を移動・保存前に flush | panel_controls.dart:284-380; notes_desk.dart:487; editor_window.dart | 2026-09-02(60) | 「閉じ損ねは鍵の全喪失」 |
| 300 ms の自動保存 debounce、閉じる時の save/as | 保存を忘れさせない | app/editor_window.dart:67, :155-163, :321-347 | — | — |
| Freeze の裏仕事・棚の失敗理由を status bar へ 1 行 | 走っていなければ出ない、失敗は理由 | editor_window.dart:19, :37 | — | (code 内コメントのみ) |
| Composition controls(比率 preset、背景色 4 段 + hex、fps 有理数) | 16:9 / 9:16 / 1:1 / 4K、Black/Dark/Grey/White、23.976 = 24000/1001 | composition_controls.dart:34-123 | 2026-09-09(606) | 「AE の Composition Settings → Background Color」 |
| Export controls(All / Marker to marker、MP4 H.264 + AAC、300 ms poll) | 進捗 done/total、Cancel、空範囲の guard | export_controls.dart:34-100 | — | — |
| History 台帳(Rust `history.jsonl` 200 点、異常終了 1 行) | 起動時に前回の `end` が無ければ異常終了を印 | native/src/editor/history.rs | 2026-09-09(619) | 「記録などクラッシュなどのログも観れると嬉しい」 |
| 再生中の UI 再初期化で native 時計を保つ | hot restart しても frame が飛ばない(frame 1749 で継続) | editor_session.dart 再接続; test/session_reconnect_test.dart | 2026-09-06(561) | 「native の時計をリセットせず、main session が更新要求を再開する」 |
| 試験 60 file(装置ごとの番人) | numeric_field_{drag,typing,wheel,trackpad_repro}、drag_session、inspector_row_watch、panel_rebuild_scope、panel_layout_cost、desk_{context,scope,depth_*}、ease_interaction、browser_{filters,tile,files,…}、fonts_shelf、color_{wheel,swatch,palette}、stage_{observer,spatial_gizmo,ground}、timeline_{layer_drag,multi_move,navigation,summary_keys}、viewport_motion、visual_selection、hover_and_tap_target、animate_mode、blend_panel、history_panel、notes_canvas、rich_text_editor、settings_flat_projection、session_reconnect | motolii/ui/test/ | — | docs/gesture-tests.md(W3C Pointer Events / UIKit / Android slop の必須 5 項) |

### J. 音・触覚・遊び

| 名前 | 効き | 場所 | 決定 | 理由 |
|---|---|---|---|---|
| 音 / 触覚 | **無し**(`HapticFeedback`・`SystemSound`・audioplayers 0 件) | — | — | visual-language「motion は装飾であって唯一の手掛かりにしない」の延長 |
| 隠し要素 | **無し**(easter/konami 0 件) | — | 2026-08-08 の 3 条件 → 「カササギは机に居る」(§5)は code に無い | 「鳥は何も示さない」 |
| 遊びに当たる物 | dice 🎲、OP-1 の主役、FittedName の読み流し、Ease の玉 | 上記 D・F・E | 2026-09-08(602) | 「ポップさやケレン味で UX を代替しない」の範囲内 |

## 文書と code のズレ

| 文書が言う | code | 扱い |
|---|---|---|
| 高コントラスト表示・Light・font/言語の User settings(ui-visual-language) | `EditorTheme` は static const、`Theme.of` 0、切替無し(audit) | M3 時代の設計基準。Stage 5 は dark 固定の裁定。**作り直しで持ち込むなら `EditorInk` の上に** |
| UI motion の token 化・reduce motion の尊重(ui-visual-language) | duration token 無し。`disableAnimationsOf` は Ease の玉(ease_desk.dart:470)だけ。裁定 66(direct 0 / hover 100 / selection 150 ms)は数値として散在 | 半分だけ。lint の対象にする候補 |
| 「hover/focus 中の操作の『何をするか』『shortcut』を出す口(Blender の Info)」(ui-visual-language) | 無し。しかも tooltip は `EditorApp.noHover` で全 off(editor_app.dart:10)。文言は tooltip の message に書いてあるが今は誰にも見えない | 未実装 + 眠っている文言 |
| 「数値の ▶ で範囲を 1 秒往復 preview」(decision 602 規則 4) | Inspector の欄に無し。Ease の玉(`Preview motion`)だけ | 未実装 |
| 「数値は横スクロールで変えられる」(利用者の言う新概念) | code に在る(panel_controls.dart:572-587、test 2 本)が decision-index・product-contract に行が無い | **code にあって docs に無い概念**。decision に 1 行要る |
| Media の 16:9 thumbnail、波形、VFR/HDR の札(product-contract、記憶「全ての素材を第一線に」) | thumbnail は 9:16 grid、波形無し、HDR は format 文字だけ、VFR の表示無し | 未実装 |
| 「Desk は Depth / Ease / Blend / History」(panel-placement) と 「顔は書き置き・参考画像・カササギ」(inspector-and-desk §3) | 顔 = Tools の棚(★)。書き置き・参考画像は Notes(別 panel)へ移った。カササギ無し | 09-02 → 09-06 で意味が移った。inspector-and-desk §3・§5 は履歴 |
| Utility panel(wiki/window「寄せる・並べる・写す」)、Output panel | catalog に Utility 無し(Inspector へ、62)、Output = Camera tab(627) | wiki/window.md が古い |
| Inspector の row pulse を「値が息する」と読む向き | 配線であり視覚効果は無い(inspector.dart:78-83) | 名前が誤解を呼ぶ。作り直し時に「row watch」等へ |
| Timeline「トラックパッドは縦横移動とピンチ、同じ慣性処理」(product-contract) | `ViewportMotion` は ruler の scrub zoom と pan を通す。lane 上の pinch は要確認 | 検証薄 |
| Web の Pinterest API / 埋め込み WebKit | 無し(product-contract も「not part of this change」) | 一致(未実装と明記) |
| `EditorDragSession` で 4 写しを 1 つに(audit 判定 4) | gradient だけ mixin、panel_controls の 3 写し(:655-668 / :1144-1155 / :1466-1477)は残る | 進行中 |
| `colors_shelf.dart:71-111` の貼り間違い 29 error(audit) | 作業樹では `classification()`(:74)と `groups`(:80)が別 method に戻っている | 直った(未 commit) |
| flutter-audit「ThemeExtension 無し」「色の lint 無し」 | `EditorInk extends ThemeExtension`(theme.dart:14)、`raw_color.dart` 在る(未 commit) | audit の後に足された。commit されれば一致 |
| Files 棚の「Cannot read this folder」 | `listingError` に入るが描かれない(files_shelf.dart:309) | 死んだ装置 |

## 説明の無い装置(code にあって docs に理由が無い)

- `EditorInk` ThemeExtension(theme.dart:14)— 面ごとの ink。visual-language の「面の意味 role」に対応するが名指し無し。
- `owner` による well の据え置き(panel_controls.dart:431-437)— 選択切替で render object を保つ。code コメントのみ。
- Figma 段ばしご(panel_controls.dart:627-638)、左押しホイール 1 刻み・trackpad 横 2 本指(:572-580)、300 ms settle(:622)— decision 602 は「触れる顔」までで、指の割り当ては未記録。
- `EditorLamp` の ember 状態と「unlit は hover でだけ」(panel_controls.dart:890-927)— 567 の ◇◆ 3 状態から 4 状態に増えた経緯なし。
- `ViewportMotion` の 8000 px/s clamp(viewport_motion.dart:84)。
- `_SampleShelf` の 2048 / 3 枚 / 120 ms / 16 ms(native_visual_sample.dart:20-24)。
- `FittedName` の 90 px/s・4096 cache(parts.dart:171, :221)。
- Blend の 90 ms hover(blend_panel.dart:185)、Export の 300 ms poll(export_controls.dart:93)、autosave 300 ms(editor_window.dart:163)、History の 160 px で時刻(history_records.dart:112)。
- gradient の 32 stop 上限・32 px の drop 距離・.35 の予告(gradient_inspector.dart:38, :113, :304)。
- seed = `microsecondsSinceEpoch % max`(inspector.dart:1842-1846)。
- Inspector の scroll 0 へ戻し(inspector.dart:130-133)、narrow well の decimals 0(:588)。
- `_Justify` の並びが left / center / right で値は 0 / 2 / 1(fonts_shelf.dart:419)。
- Desk `_drawer`(名前だけ見る枠、desk.dart:59)、Ease `_epoch`(遅い返信を捨てる、ease_desk.dart:124)、`_labelHeight`(幅ごと 1 回、:694-714)、Ease の paper が theme に依らず暗い(`EditorInk.dark.easePaper`, :619)。
- Depth の hit 13 px、Ease handle 12 px、Stage rotation 7 px — 全部生数字で lint の対象外(hit 半径は `measuredNames` に無い)。
- `ValueKey('browser:…')` / `ease-*` の試験の継ぎ目の慣習。
- Notes の Delete が EditableText focus 中は効かない(notes_desk.dart:216-220)、canvas が +300 px ずつ育つ(:198-211)。
- Web の Pinterest 既定(web_panel.dart:18)— product-contract に 1 行あるのみ。
- 初回起動で Notes が別窓(editor_window.dart:140)、Outside dim の既定 .75(:68)、sheet の x offset s85/s155/s200(:526-531)、drop zone の 18/82・22/78%(workspace_view.dart:245-257)、`initialDock` の比 .68/.205/.775/.76(layout.dart:132-152)。
- Timeline painter の `fontFamily: 'Arial'`(timeline.dart:1456)と tabular 無し(「$frame / $duration」を固定位置で描いて揺れを避けている :1863-1870)、1/2/5/10 の目盛段と 12 px の副目盛抑制(:1491-1504)、名前欄 114〜162 + indent、double-tap で 138(:1207-1244)、zoom clamp .1〜40 / Stage .02〜16。
- Stage の `_grabTolerance` 25% 上限(:596-610)、凸包 hit(:1635-1669)、回転取っ手 22 px、`−`/`+` が比でなく 1% 刻み(:1059-1077)。
- 数値の横スクロール(panel_controls.dart:572-587)— K-1 参照。decision に無い最大の物。
- 効果 menu の散文 label(inspector.dart:1315)。
- rail の縦 tab 化 48 px / grip double-tap 0↔96(browser.dart:674-713)。

## K. UX の概念(操作の発明)— 仕草ごとに 1 行

列: 概念・仕草・何が新しいか(AE / Figma / Ableton と比べて)・場所・決定日。値の触り方は全部別行。

### K-1. 数値の触り方(`EditorNumericField`、Inspector の well)

| 概念 | 仕草 | 何が新しいか | 場所 | 決定日 |
|---|---|---|---|---|
| 横 drag で scrub | 数字を左右に掴んで値を変える(cursor `resizeLeftRight`、slop 3 px) | AE の scrub と同じ。Figma は drag 無し(label drag のみ) | panel_controls.dart:648, :768 | gesture-tests 2026-09-01 |
| **横スクロール(trackpad 2 本指)で値が動く** | 押さずに横へ 2 本指で流すと drag と同じに動き、指が止まって 300 ms で確定。縦は周りの list のまま(arena で横だけ取る)。natural scroll を鏡映 | AE・Figma・Ableton に無い。「数値は横スクロールで変えられる」= Motolii の発明 | panel_controls.dart:572-587, :622-625; test/numeric_field_wheel_test.dart:59,73 | (code コメントのみ、decision 無し) |
| 左ボタン押しながらホイール = 1 刻み | 押している間だけ notch ごとに 1 単位(Shift ×10)。押さないホイールは list へ | Ableton の「hover でホイール」より安全側(誤操作を防ぐ) | panel_controls.dart:572-574; test/numeric_field_wheel_test.dart:142 | — |
| Figma の段ばしご(rung) | drag 中に縦へ離すと ×10 / ×1 / ×0.1 / ×0.01 の段に入る。段を変えても値は飛ばない。tooltip に `×rung` | Figma の precision ladder の写し。AE には無い(Shift/⌘ の 2 段だけ) | panel_controls.dart:627-640, :786-790 | 2026-09-08(602 「触れる顔」の実装) |
| Shift = ×10 | drag・ホイール・↑↓ 全部で Shift は 10 倍 | AE と同じ(AE は Shift ×10、⌘ ×0.1)。⌘ の細かさは段ばしごへ | panel_controls.dart:582, :604, :715 | 2026-09-03(550 「離す時の修飾は見ない」) |
| ↑ / ↓ で 1 刻み確定 | 欄に focus があれば矢印で ±speed(Shift ×10)、1 手 1 undo | Figma/Ableton と同じ | panel_controls.dart:686-722 | — |
| double-click / Enter で打ち込み | 表示中の文字列そのままを編集開始(`0.00` が `0.0` に化けない、`-19` が `-18.99…` にならない) | AE は click で編集、Motolii は click = 選択・drag、double = 打ち込み(Figma 型) | panel_controls.dart:515-518, :772 | 2026-09-02(60 「欄は許可制」) |
| **double-click は既定へ戻さない** | 既定へ戻すのは右クリック menu の `Reset`(欄 1 つ)、`Back to where the numbers rest`(効果 1 枚)、既定値は track の tick で見える | Ableton は double-click/Delete で既定へ戻る。Motolii は double = 打ち込みに割り当てたので reset は menu | inspector.dart:1668-1685, :661; panel_controls.dart:424-426 | 2026-09-08(602 規則 6) |
| Esc = cancel | drag 中は掴む前の値へ、打ち込み中は下書きを捨てる、IME 合成中は合成だけ戻す | AE は drag 中の Esc が効かない(確定される) | panel_controls.dart:307-318, :686-690; gesture-tests §2 | 2026-09-01 |
| 窓を離れても field は取り消さない(`watchesWindow => false`) | ⌘Tab で別 app へ行っても打ちかけの数字は残る。dial/pad/stop は取り消す | AE 無し(field は modal)。Motolii は「field は 1 つだけ例外」 | panel_controls.dart:453-456 | — |
| 掴んでいる間に見せた値 = 確定値 | 離す瞬間の修飾は見ない | AE は release 時の修飾で値が変わることがある(利用者の実害) | panel_controls.dart `_end`; decision 550 | 2026-09-03 |
| 既定値で静か、動かすと ink、触れる数字は点線 | 値の顔で「触れる / 既定 / 変えた」を読む | Tangle(Bret Victor)の写し。AE は全部同じ青文字 | panel_controls.dart:822-856 | 2026-09-08(602 規則 2) |
| 単位の席は空でも残す | `px` `%` `°` の rider が無くても列の桁が揃う | — | panel_controls.dart:860-874 | — |
| 複数選択の drag は相対、打ち込みは絶対 | 3 層を掴めば各層が自分の差分、数字を打てば全部その値、mixed は `—` | AE は複数選択で drag も打ち込みも絶対(全部同じ値になる) | inspector.dart:406-437, :528-537 | 2026-09-02〜03(51, 61) |
| 角度は dial、点は pad | 角度はつまみを回す(preview → commit)、`name_x`/`name_y` は 1 つの点として掴む(距離で速度、範囲不要) | AE は全部数字。Vital / Lightroom の「欄が自分の効果を描く」 | panel_controls.dart:1069-1280, :1472-1575; inspector.dart:1435-1453 | 2026-09-08(602 規則 5) |
| `▶` で範囲を 1 秒往復 preview | (裁定は「規則 4」だが Inspector の欄には未実装。Ease の玉だけ) | Desmos の写し | — | 2026-09-08(602)未実装 |
| 🎲 = 範囲内 ±20% で振る、1 undo | 効果 1 枚の数を一度に振る、seed は全振り | Ableton の randomize の写し。AE 無し | inspector.dart:633, :1834-1857 | 2026-09-08(602 規則 6) |
| lamp を押す = 今のコマにキー / 消す | ◇ の押下がキーの打ち消し | AE の stopwatch(全キー消去)と違い、1 コマ分だけ | panel_controls.dart:902-950; inspector.dart:562-569 | 2026-09-07(567) |

### K-2. キーを打つ・時間

| 概念 | 仕草 | 何が新しいか | 場所 | 決定日 |
|---|---|---|---|---|
| Animate 1 本(Rive の Design/Animate) | Animate が入っている間だけ触った値がキーになる。切 + キー有りは全キーへ同じ差(形を保つ) | AE の stopwatch(property ごと)を捨てた。AE 式「勝手にキーが増える」を止めた | inspector.dart:1938-1947; native | 2026-09-07(567) |
| A 単体で Animate 入切(Anchor は Shift+A) | Ableton の `a`(automation arm)と同じ指 | AE に無い | editor_shortcuts.dart:163-172 | 2026-09-10(620) |
| Key the start too(Animate From) | Animate を入れた時刻を覚え、別時刻で最初に触った値は起点と終点の 2 キーになる | AE は 1 キーで「動かない」→ 2 キー目で動く。Motolii は 1 手で対 | panel_settings.dart:56-68; editor_session.dart:62 | 2026-09-10(620) |
| 生まれた対を自動選択 → Ease が出る | 対が出来た瞬間その 2 キーが選ばれ、Desk が Ease に向く。Desk を勝手に開かない | 「キーを打つたびに区間を選んで ease」の手数を消す | native port.rs `born_spans`; desk.dart:47-50 | 2026-09-10(622) |
| Use for new keys | 新しいキーの ease を先に決める(既定 Easy Ease) | AE は Linear 固定 → 後から F9 | ease_desk.dart:986-990 | 2026-09-10(622) |
| キーの間の線を押す = 両端を選ぶ | Linear 破線・形が付けば実線・両端選択で差し色。Shift で足す | AE は 2 キーを個別に選ぶ。線が「区間」という 1 単位 | timeline.dart:225-235, :1716-1742 | 2026-09-10(622) |
| Shift+⌘D = 直前のキー選択に戻る | Photoshop の Reselect(1 つ前だけ) | AE 無し | editor_shortcuts.dart:66; editor_session.dart:117-122 | 2026-09-10(622) |
| F9 / Shift+F9 / ⌘Shift+F9 = Easy Ease 3 種 | AE と同じ指 | 同じ | editor_shortcuts.dart:48-60 | — |
| 補色の差し色(Animate 中) | 入っている間、キーが生まれる所だけ accent が補色へ | Ableton の record-arm 赤に当たる物を「色相の反転」で | theme.dart:171-176 | 2026-09-10(620) |
| 素の矢印 = コマ送り、Alt+矢印 = ナッジ(Shift 10 px) | NLE の指を素に、Figma の指を修飾に | Figma と衝突するので分けた | editor_shortcuts.dart:128-145 | 2026-09-03(549) |
| ↑↓ = 層の一覧を歩く、Home/End、M = marker、U = keyed only、P/S/R/T/A = Inspector のその欄を出す | AE の property letter を「Inspector で露出」に写した | AE は Timeline に露出、Motolii は Inspector | editor_shortcuts.dart:117-179 | — |
| P/R/S を押している間だけ gizmo が絞られる | Stage で P を押したまま掴むと位置だけ、R で回転、S で拡縮。同じ字が Inspector でも同じ欄 | Blender の G/R/S(押して離す)でなく「押している間」。AE 無し | stage.dart:834-871 | — |
| 尺は壁ではない | 頭は 0 で止め、先は自由。再生は尺の終わりで止まらず越えて進む、ループしない。Timeline は今のコマが端を越えたら 2 倍ずつ伸びる | Ableton の arrangement と同じ。AE は work area で止まる | timeline.dart:132-142, :158-168, :211-222; editor_session.dart:585-646 | 2026-09-07(595) |
| 止めるのは人だけ(操作で再生を止めない) | 編集しても再生は回り続ける(`requiresPause` は書類の差し替えだけ) | Ableton 型。AE は編集で止まる | bridge/protocol.dart:89-91 | 2026-09-07(595) |
| scrub は絵を待たない | 掴んでいる間のヘッドは native の返事を待たず描く、pending は最新だけ | — | timeline.dart:64-83 | — |
| 尺の無い物の既定長 = 見えている幅の 3/4 | 静止画・文字・図形を置くと終端が画面内に来る | AE は comp の終わりまで | editor_session.dart:226, :581-584 | 2026-09-07(579) |
| 層の bar: Alt = slip、両端 6 px = trim、他 = move、複数は 0 で揃って止まる | AE と同じ語彙、slip が Alt | 同じ(Premiere の slip も Alt) | timeline.dart:439-445, :474-484 | — |
| 名前欄 drag = 並べ替え(挿入線 / 囲い)、Media の札も同じ落とし先 | 行と Media の drop が `aimRowDrop` 1 本 | AE の Project→Timeline drop と同型だが「inside(グループへ)」を囲いで見せる | timeline.dart:495-534; media_shelf.dart:245 | 2026-09-07(573) |
| ゴースト = 別の時刻で見た行 | 同じ行に遅れ分ずれた掴めない帯。はみ出しだけ灰 | AE の Echo と違い「層の属性」。Sequence がこれを配る | timeline.dart:1617-1661; inspector.dart:1259 | 2026-09-07(589) |
| 畳んだ行の集約キー | 層行に全 property/effect のキーを 1 つの ◇ で、grip の一部だけ動かせる | AE の集約キーと同じ、split-grip は AE より細かい | timeline_layout.dart:22-43; timeline.dart:1694-1707 | product-contract |
| Freeze / Unfreeze は右クリック | DAW の Freeze Track。中を固めて軽く、配置は生きたまま | AE の pre-compose + render でなく Ableton の Freeze | timeline.dart:884-892; inspector.dart:1279-1302 | 2026-09-13(630) |
| Marker to marker で書き出し | playhead の前後の marker が範囲 | AE の work area でなく marker | export_controls.dart:34-68 | — |

### K-3. Stage の仕草

| 概念 | 仕草 | 何が新しいか | 場所 | 決定日 |
|---|---|---|---|---|
| 素のホイール = 滑る、⌘/Ctrl+ホイール = 寄る(pointer 中心)、中ボタン = パン、Space+drag = パン、右 drag = orbit | trackpad の 2 本指がそのまま移動 | AE は素のホイール = 縦 scroll、Figma は素 = 滑る・⌘ = zoom(Figma 型)。右 = orbit は rerun/Blender | stage.dart:720-727, :770-775, :1114-1126, :957-970 | 2026-09-03(548)、09-07(596) |
| ⌘0 / ⌘1 / ⌘= / ⌘− = Fit / 100% / 段階(×1.2) | 視点の倍率。UI の文字倍率は Settings | AE/Figma/Nuke/Blender と同じ | stage.dart:122-138; editor_shortcuts.dart:97-116 | 2026-09-03(547) |
| double-click = 注視(object)/ 視点を戻す(背景)、`Front` ボタン | rerun 3D view の写し。間隔は入力時刻で測る | AE 無し | stage.dart:479-492, :1041-1046 | 2026-09-07(596) |
| 掴む優先順(eyedropper → 右 orbit → double → 作業範囲の縁 → camera 取っ手 → camera 辺 → 中/Space パン → handle → 3D mesh → 層 → marquee) | 1 つの `_down` に順を書いた | — | stage.dart:708-816 | — |
| 籠 = mask の 4 角、線無し。角 = 拡縮、辺 = 片側、上の取っ手 = 回転(7 px、25% 上限) | 縁取り効果の邪魔をしない | AE の bounding box に近いが「線を引かない」 | stage.dart:567-610, :189-195 | 2026-09-11(625) |
| 3D は native が当たり判定した三角形そのもの、白 = 休み・黒 = hover | 「見えている物を掴む」— 描く物と当たる物が同じ data | transform-gizmo 型 crate でなく fork の hit をそのまま | stage.dart:158-219, :1593-1605 | 2026-08-18 概念地図 → 09-11 |
| ⌘を押しながら移動 = 他の箱の辺・中心に snap(guide 線) | AE の snapping(⌘)の写し | 同じ | stage.dart:288-311, :628-642 | 2026-09-15 snap-to-grid.md |
| Shift / ⌘ / Ctrl = 追加選択、marquee は silhouette(凸包)で当たる | 8 角の 3D 箱も見た目の輪郭で選ぶ | — | stage.dart:532-535, :1635-1669 | — |
| click = 選択、drag は slop(mouse 3 px / touch kTouchSlop)の後 | 押しただけで値が変わらない | gesture-tests §4 | stage.dart:810, :923-929 | 2026-09-01 |
| Esc = drag だけ取り消し(掴んでいない時は選択解除 → sheet → drawer の順) | Esc の梯子 | — | stage.dart:1098-1106; editor_shortcuts.dart:34-43 | 2026-09-01 |
| Camera の箱(Boxcam): 辺 = Center、角 = Zoom、上の取っ手 = Roll。回した camera は eye + frustum だけ | 正面を向いた自前 camera だけ箱で author できる | AE の camera は 3D ギズモ。Boxcam の「箱を掴む」を写した | stage.dart:337-360, :424-463 | 2026-09-12(627)、09-13(629) |
| 作業範囲(Stage 層)の縁を掴んで広げる(`Extend` switch) | comp を動かさず余白を書く。初回に Stage 層を作る | AE 無し(comp を広げるしかない) | stage.dart:252-276, :398-422 | 2026-09-09(604) |
| 地の透明 switch(市松) | alpha < 1 なら書き出しは alpha、Stage は市松。1 gesture = 1 undo | AE の transparency grid に「書き出し alpha」を結んだ | stage.dart:228-244, :1249-1262 | — |
| Eyedropper: 次の Stage click が色を読む、Esc で止める | 読んだ色は焦点の色 slot か選択へ | Photoshop 型、AE 無し | stage.dart:696-706; colors_shelf.dart:666, :728 | 2026-09-13(634) |
| Anchor grid を撫でると Stage に仮の pivot | 9 点 hover で位置を見せてから押す | AE 無し(anchor は Pan Behind で掴む) | inspector.dart:1148-1161; stage.dart:1238-1245 | — |
| 再生中は gizmo を消す | 絵を見る時は籠を出さない | — | stage.dart:1230 | — |
| Stage / Camera の 2 tab が同時に生きる、Stage は枠の外を灰の被せ | 「枠は参照で crop ではない」 | AE の comp viewer + Boxcam。Figma 無し | stage.dart:15-18, :1385-1425 | 2026-09-12(627) |
| 2D / 2.5D / 3D は Inspector の Space で押す、Stage は読むだけ。切替は姿を保って 1 undo | 投影の意図は利用者が宣言、操作から推定しない | AE の 3D switch に近いが「絵が変わらない」保証付き | inspector.dart:1173-1190; stage.dart:194, :574-576; test/stage_spatial_gizmo_test.dart | 2026-09-06(565)、09-12(24) |

### K-4. Desk / Ease / Depth / Blend

| 概念 | 仕草 | 何が新しいか | 場所 | 決定日 |
|---|---|---|---|---|
| 机は焦点に付いて行き、中で触っている間は奪われない | キー → Ease、Camera → Depth、Blend 行 → Blend。★ で idle 既定 | Ableton の Clip/Device View(選択で切替)+「作業中は動かない」 | desk.dart:47-113, :189-194 | 2026-09-02(62)、09-06 |
| Ease の handle drag → 曲線が live、Esc で戻す、preset の hover で試聴(`_peek`)、矢印で preset を歩く + Enter | AE の graph editor に無い: 意味の 1 文、動きの玉、hover 試聴 | Flow / Alight Motion の「区間 → popup」を常設に | ease_desk.dart:478-492, :571-578, :725-757 | 2026-09-09(617) |
| Overshoot 枠(−0.5〜2.2)を preset が宣言すれば自動で開く | Bounce/Elastic が箱を出る | AE は 0..1 固定 | ease_desk.dart:360, :1029-1065, :1232-1235 | — |
| 複数層を選んで Ease の曲線を掴む = Sequence(Stagger)を配る、Stage のゴーストが live、放して 1 undo | in 点を動かさずに遅れを曲線で | AE の Sequence Layers・Motion Tools Pro の Sequence は in 点を動かす | ease_desk.dart:219-290 | 2026-09-07(589)+ 自走 |
| Depth: eye を掴んで orbit(yaw + distance)、層を掴んで親空間で移動、空所 = 選択解除、locked は選べるが動かない | 上から見た平面図で奥行きを author | AE の Top view に近いが「注視点が原点、目盛は中身から」 | depth_desk.dart:140-221 | 2026-09-10(32) |
| Blend: tile を撫でると Stage が変わる(90 ms)、押すと 1 undo、Esc/離脱で戻る、keyboard focus でも preview | 見本の格子で hover preview | Photoshop / Figma / Procreate 型。AE は dropdown | blend_panel.dart:144-221, :345-350 | 2026-09-02(§4) |
| History: 点を押すと undo/redo で往復、保存・起動・異常終了も同じ列 | クリスタの履歴 + Git Graph | AE の History panel 無し(Photoshop 型) | history_records.dart:83-114 | 2026-09-09(619) |
| Notes: 空所を押して書く、上の取っ手で動かす、角で伸ばす、drop/paste で画像、`Link selection` で層 · 区間の札(押すと選択 + seek) | OneNote の note container + 時間への link | AE 無し | notes_desk.dart:98-148, :531-559, :646-653 | 2026-09-06 |

### K-5. Browser の仕草

| 概念 | 仕草 | 何が新しいか | 場所 | 決定日 |
|---|---|---|---|---|
| click = 着せる、double-click = 作る(層が増える物は double) | 迷い click で層が増えない。Create・Media・Fonts(無選択) | AE は drag でしか置けない | shelf.dart:124; fonts_shelf.dart:178-206; media_shelf.dart:14-16 | 2026-09-13(639) |
| 押した瞬間に選ぶ(`Listener.onPointerDown`、arena の下) | double-tap 併存でも 100 ms 待たない | — | tile.dart:152 | 2026-09-12(23) |
| Shift = 範囲、⌘/Ctrl = toggle、⌘A、Enter = 当てる、矢印/Home/End、Delete、Esc = 検索 → 選択の順に消す | 棚の keyboard 一式 | Finder / Live の混合 | browser.dart:394-489 | — |
| 1〜7 = Collection に入れる、0 = 外す、行を掴んで rail の Collection へ drop、double-click で改名、右 menu に 7 行 | Live 12 の Collections そのまま + drag&drop | Ableton と同じ(AE 無し) | browser/filters.dart:575-700; browser.dart:459, :1026 | 2026-09-13(644, 647)、09-14(649) |
| Filter View: 群内 OR・群間 AND、⌘click で群内に足す、範囲 `min-max` を自分で切る、Label = 保存した絞り、⌘E = Quick Tag | Live 12 の Filter View、範囲は Motolii の追加 | Ableton には範囲群が無い | filters.dart:164-260, :488; browser.dart:134-185, :455 | 2026-09-13(644, 645, 646) |
| Colors: 押す = 当てる、右 = 直す、Shift+押す = gradient、空きを押す = 今の色を置く、drop = 抽出(From image) | Good Boy Ninja の Colors/Swatches の語彙 | AE 無し(Figma の styles に近い) | colors_shelf.dart:155, :268-300; decision 634 | 2026-09-13(634) |
| 色の原子 = 見本 1 個、押すと Browser の輪が向く(hex も輪も持たない) | Inspector・Composition・gradient の stop 全部同じ | AE は各所に picker。Motolii は picker 1 つ | color_field.dart:7-30; gradient_inspector.dart:100-110 | 2026-09-13(634) |
| gradient bar: つまみ drag = 位置、外へ drag = 消す(32 px、.35 で予告)、bar を押す = stop を増やす、Delete = 消す(最低 2) | Figma の gradient bar に「外へ drag で消す」を足した | AE の gradient editor は別窓 | gradient_inspector.dart:38-56, :128-143, :176-225 | 2026-09-13(634) |
| 色の輪: 白黒でも色相を失わない、三角/四角、輪の外れた pointer は最寄りの辺へ | — | — | color_wheel.dart:66-79; colors_shelf.dart:497-503 | — |
| Fonts: 文字群(Hiragana/Katakana/Kanji/Latin/Upper/Lower)を選んで着せる、見本は層の本文 | 「書体を跨ぐ和欧混植」を棚で | AE 無し(1 層 1 書体)。Illustrator の合成フォントに近い | fonts_shelf.dart:52-76, :307-400 | 2026-09-13(637) |
| 名前は縮めず、hover で読み流す(90 px/s) | 省略記号でなく流れる | — | parts.dart:153-252 | — |
| rail を 48 px 以下に畳むと縦 tab、grip の double-tap で 0↔96 | — | — | browser.dart:674-713 | — |
| Import は Media へ飛んで新しい物を選ぶ | File > Import / Finder drop 後に迷子にならない | Premiere 型 | browser.dart:342 | 2026-09-01(67) |

### K-6. 窓・欄・panel

| 概念 | 仕草 | 何が新しいか | 場所 | 決定日 |
|---|---|---|---|---|
| 欄は許可制 | 押すまで欄は無く、Enter / Esc / 外 / 窓離脱で消える。open drafts は保存・移動の前に flush | 「窓に input が在れば打鍵は全部そこへ行く」問題を構造で消した | panel_controls.dart:284-380; editor_session.dart:189-194 | 2026-09-02(60) |
| tab を掴んで 4 置き場へ、外へ出せば別窓、右 menu で Detach/Close、空の leaf は「Drop a panel here」 | 端 18/82・22/78% の drop zone | VS Code / Ableton の dock と同型 | workspace_view.dart:183-194, :245-281 | 2026-09-06 |
| 仕切り drag は node の状態(panel は build されない) | 引いても他の panel が組み直らない | — | workspace_view.dart:44-52 | 2026-09-12(23) |
| Settings で Desk / Tab / Window / Hidden を panel ごとに | 補助道具の昇格・降格 | Blender の editor area に近い | panel_settings.dart:107-161 | 2026-09-06 |
| Esc の梯子: cancel preview → sheet → drawer → 選択解除 | 1 つの Esc で「一番近い物」から閉じる | — | editor_shortcuts.dart:34-43 | — |
| Space = 再生/停止(Ease の preset に focus があっても通る) | — | AE と同じ | editor_shortcuts.dart:44; test/ease_interaction_test.dart | — |
| ⌘K = split、⌘Alt+K = composition、⌘G/⇧⌘G = group/ungroup、⌘D = 複製(切れる) | 複製は「切れる」が既定、まとめて増やすは配置効果 | AE の ⌘D と同じ、「繋げて増やす」は Repeater | editor_shortcuts.dart:61-116 | 2026-08-31(72) |
| UI 倍率(Settings の ±、.5〜2)と Outside dim(±.05) | 窓の都合は Settings、作品には入らない | — | editor_window.dart:556-606 | 2026-09-03(547) |
| 初回起動は Notes が別窓 | (理由未記録) | — | editor_window.dart:140 | — |
| 支援技術の押下 = 打鍵の Enter/Space と同じ関数 | test と読み上げが同じ道 | — | (旧 Blitz 世代の裁定 555、Flutter では Semantics に委譲) | 2026-09-03 |

## 守る順(作り直しで失うと痛い 5 つ)

1. **drag→preview→commit + `EditorPreviewQueue` + Esc/focus loss の取り消し**(C)— gesture-tests の必須 5 項の実体。掴んでいる間の絵が確定値、履歴は 1 undo。
2. **`DocumentSlice` / `Picked` / row watch / `shouldRepaint` field 比較**(B)— 「ビューが超軽量」の根。型付き model を足すなら上に乗せる。
3. **`EditorMetrics` + `EditorTheme`/`EditorInk` + `raw_dimension`/`raw_color` lint**(A)— 見た目の出所 1 本。lift は M3 の数、accent は補色。
4. **Inspector の数値の顔 + Animate 1 本**(D)— `_Character` 表、TrackStyle、hero、Advanced の点、dice、`EditorLamp`、Animate/A/補色。裁定 567・569・602・620 の束。
5. **Browser の枠/棚契約 + Live 12 の Filter/Collections + Colors/Fonts の「棚」型**(F)— 棚 1 tab = 1 file、色の原子 = 見本 1 個、書体は Flutter 自身の描画で見本。裁定 31・544・634・637〜647。

次点: Desk = レンズ(E)、Stage = Boxcam の 2 tab(G)、Timeline のキー間の線と ◇◆ M S L ↳(H)、`ViewportMotion` の 1 本化、History の縦線。

UX の概念で特に守る物(K): 数値の横スクロール・段ばしご・Esc cancel・掴んでいる値 = 確定値(K-1)、Animate 1 本 + A + 対の自動選択 + 線を押して区間(K-2)、尺は壁ではない(K-2)、素のホイール = 滑る / ⌘ = 寄る / 右 = orbit(K-3)、click = 着せる・double = 作る(K-5)、欄は許可制(K-6)。

## Sources

- `motolii/AGENTS.md`(窓の文字は英語、決定は decision-index)
- `docs/ui-visual-language.md`(M3 設計基準: 読む前に分かる、情報を隠さない、ケレン味で代替しない、tabular、motion token、高コントラスト)
- `docs/stage5/README.md`、`docs/stage5/workspace.json`(pending)、`docs/stage5/product-contract.md`(画面構成・見た目・Timeline・入力・2.5D・Panels/Settings/Desk・Notes/Web/Ease)、`docs/stage5/panel-placement.md`
- `docs/wiki/window.md`(置き場・文字の規則)
- `docs/decision-index.md` 2026-09 行: 23, 24, 27, 28, 30, 31, 32, 44, 45, 50, 51, 60, 61, 62, 66, 67, 544, 547-550, 561, 565, 567, 569, 573, 579, 591, 595-597, 599-606, 617, 619-622, 624, 625, 627, 629-634, 637-639, 642-647, 649, 657, 664
- `docs/reviews/2026-09-19-flutter-audit.md`(残す物、判定 5 つ)、`2026-09-12-ui-derive-outside-build.md`(軽さの法 7 つ)、`2026-09-02-inspector-and-desk.md`(机 = レンズ、blend は見本)、`2026-09-08-number-affordance-prior-art.md`(規則 6 つ)、`2026-09-09-browser-ui-phases.md`(tile の解剖)、`docs/gesture-tests.md`(W3C Pointer Events / UIKit / Android slop)
- code: `motolii/ui/lib/**`(48 file)、`motolii/ui/tool/motolii_lints/**`、`motolii/ui/test/*.dart`(60 file)。行番号は 2026-09-19 の作業樹。

## 変更(2026-09-19 午後、利用者の裁定)

- **籠と取っ手は意図で出す**(Stage と Camera の両方、常に): pointer がその物の上にある時だけ、その物の籠を描く。掴んで動かしている間は選択の籠。選んでいるだけでは絵の上に何も出ない。Inspector の anchor pad を触った時の印は、選択に出す(pad が意図)。`stage.dart` `_touched` / `touching` / `selectedTouched`。決定行の案: `2026-09-19 | 籠と取っ手は触ろうとした物にだけ。選択だけでは出さない | stage.dart _touched`
