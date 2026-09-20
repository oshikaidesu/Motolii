# UI の手触りの台帳 — プロ道具が気持ちいい理由を数と挙動で(2026-09-19)

状態: **観察**(一次資料で確認できた規則と、未確認の規則を分けて置く。採否は DESIGN.md の Do/Don't に写す時に判定語を付ける)

目的: Ableton Live 12・Figma・Vital(JUCE)・FabFilter・Apple HIG・Material 3・NN/g・WCAG の**利用者向け文書と公開ソース**から、「値の触り方」「応えの速さ」「色と地」「間隔と大きさ」「選択と focus」「文字」「音」の細則を数と挙動で拾い、各行に出典 URL と 25 語以内の引用を付ける。Motolii 側の適用先(Inspector/帯の値ドラッグ、hover、パネル開閉、ステージ上の選択、スナップ、タイムライン、棚のカード)を注記する。

規律: [reviews/README.md](README.md) に従い、一次資料で確認できなかった行は削らず **未確認** と印を付ける。二次記事(Medium 等)だけで裏取りした行も未確認扱い。

凡例: ✔ = 一次資料で確認、△ = 一次資料の周辺(公開ソース・実装)で確認、**未確認** = 引用を取れなかった。

---

## 1. 値の触り方(ドラッグ・修飾キー・リセット)

| # | 規則(数・挙動) | 出典 + 引用 | 合意する道具 | Motolii の適用先 |
|---|---|---|---|---|
| 1-1 ✔ | **Shift = 細かい解像度**。ドラッグ中も矢印キーも Shift で細かくなる | Ableton Live 12 Keyboard Shortcuts https://www.ableton.com/en/manual/live-keyboard-shortcuts/ — "Shift: Finer Resolution When Dragging" / "Shift up and down arrow keys: Decrement/Increment in Octaves or Fine Adjustments" | Ableton, FabFilter(1-6), Vital は Cmd(1-4) | Inspector/帯の数値、タイムラインのキー移動。修飾キーは **Shift 1 本**に統一(Vital の Cmd は採らない) |
| 1-2 ✔ | **Delete = 既定値へ戻す**(選んだ値・制御を消す = 初期化) | 同上 — "Delete: Return to Default" | Ableton(Pan は double-click でも) | Inspector の値、帯の値。Delete と double-click の両方を持つ |
| 1-3 ✔ | **double-click = 既定値へ戻す**(Pan) | Ableton Live 12 Mixing https://www.ableton.com/en/manual/mixing/ — "Double-clicking the Pan control resets it to its default value." | Ableton, Vital(1-5: JUCE の setDoubleClickReturnValue を default_value で有効化) | 円形・帯の値。テキスト入力は Alt-click(Vital)か既存の入力欄で |
| 1-4 △ | **ノブ 1 周 = 200 px のドラッグ**。Cmd で ×0.1(1 周 = 2000 px) | Vital synth_slider.h https://raw.githubusercontent.com/mtytel/vital/main/src/interface/editor_components/synth_slider.h — `kDefaultRotaryDragLength = 200.0f`, `kSlowDragMultiplier = 0.1f`, synth_slider.cpp `setMouseDragSensitivity(kDefaultRotaryDragLength / sensitivity_)` | Vital。JUCE 既定は **250 px**(1-7) | 帯の値ドラッグ: 全域 = 200〜250 px、Shift で ×0.1。これが「重すぎず軽すぎず」の実測値 |
| 1-5 △ | **直線スライダは自分の長さ = 全域**(mouse position に snap しない)、変調リングは半径の 26%、ノブの開き角は ±0.8π(=288°) | 同上 — `kRotaryModulationControlPercent = 0.26f`, `kLinearWidthPercent = 0.26f`, `kRotaryAngle = 0.8f * vital::kPi`; cpp `setSliderSnapsToMousePosition(false); setMouseDragSensitivity(max(w,h)/sensitivity_)` | Vital | 帯(横スライダ)は押した所へ飛ばさず相対ドラッグ。円形の値表示は 288° 開き、外周リングは太さ 26% |
| 1-6 ✔ | **Shift = 微調整(ドラッグもホイールも)、Alt = 軸の拘束、ホイール = 第 3 の値(Q)** | FabFilter Pro-Q 4 Display https://www.fabfilter.com/help/pro-q/using/eqdisplay — "Hold down Shift while dragging (or while using the mouse wheel) to fine-tune the settings." / "Hold down Alt while dragging to constrain to horizontal adjustments (frequency) or vertical adjustments." / "Move the mouse wheel to adjust the Q setting" | FabFilter, Ableton(1-1) | ステージ上の点ドラッグ(位置 xy + ホイールで回転/大きさ)。Alt で軸拘束。Figma は Shift 拘束なので **Alt/Shift は要裁定**(4-4 と衝突) |
| 1-7 △ | **JUCE の既定: 全域 250 px、速度モードは sensitivity 1.0 / threshold 1 px、hover の値吹き出しは 2000 ms で消える、double-click 戻しは既定 off** | JUCE juce_Slider.cpp https://raw.githubusercontent.com/juce-framework/JUCE/master/modules/juce_gui_basics/widgets/juce_Slider.cpp — `int pixelsForFullDragExtent = 250;` / `velocityModeSensitivity = 1.0 … velocityModeThreshold = 1;` / `int popupHoverTimeout = 2000;` / `bool doubleClickToValue = false;` | JUCE(= Vital, FabFilter 以外の多数の VST) | 帯の値: 全域 250 px 基準。hover の値表示は 2 s で自動消灯。double-click 戻しは **明示的に on** にする(既定 off のままの道具は触ると外れる) |
| 1-8 △ | **hover で値の吹き出しを即出す(遅延 0)、離れたら即消す** | Vital synth_slider.cpp — `mouseEnter … hovering_ = true; redoImage();` / `mouseExit … hidePopup(true); hovering_ = false;` / `if (shouldShowPopup()) parent_->showPopupDisplay(...)` | Vital。JUCE は mouseMove で出し timeout で消す(1-7) | 帯・円形の hover: 値は遅延なしで出す、離れたら即消す。tooltip 的な 500 ms 待ちは **値には使わない** |
| 1-9 ✔ | **double-click で数値をキー入力**。"2k"・"A4"・"C#2+13" のような略記を受ける | FabFilter Band controls https://www.fabfilter.com/help/pro-q/using/bandcontrols — "Double-click any knob to enter the value directly using the keyboard." | FabFilter(Vital は Alt-click) | Inspector の数値: double-click = 入力欄。Motolii では 1-3 の「double-click = 既定へ」と衝突 → **裁定**: 帯は既定へ、数字テキストは入力 |
| 1-10 ✔ | **Alt-click = bypass/一時無効**(消さずに切る) | 同上 — "you can also bypass an EQ band by Alt-clicking its dot in the display" | FabFilter | ブロック/fx の一時 off。削除と区別する |
| 1-11 ✔ | **既定は 1 px、Shift で 10 px の nudge** | Figma Adjust alignment https://help.figma.com/hc/en-us/articles/360039956914 — "By default, small nudge is set to 1 and big nudge set to 10." | Figma | ステージの矢印キー移動: 1 / Shift 10(単位は comp の px) |
| 1-12 ✔ | **Ctrl(Cmd)押しっぱなしでスナップを一時解除** | 同上 — "If you have Snap to geometry or Snap to objects enabled, hold Control to temporarily disable them." | Figma | ステージのスナップ: 修飾キーで一時 off、設定で恒久 off |
| 1-13 ✔ | **複数選択で 1 つ動かすと全部動く** | Ableton Mixing — "With multiple tracks selected, adjusting the volume of one of them will adjust the others as well." / FabFilter Display — "Click and drag a selected dot to adjust the frequency and gain of all selected bands." | Ableton, FabFilter | 帯・Inspector で複数選択時は相対で全部に効かせる(「1 つが全体に干渉する」の UI 側) |
| 1-14 **未確認** | Ableton の数値欄は **縦ドラッグ**で値が変わる(横ではない) | Live 12 manual の Live Concepts / Clip View / Mixing の各章に該当文を見つけられず。挙動は実機で既知だが文書の引用が取れない | — | 帯は縦ドラッグを基本にするか要裁定。Vital/JUCE は RotaryHorizontalVerticalDrag(縦横どちらも) |

## 2. 応えの速さ(ms)

| # | 規則 | 出典 + 引用 | 合意 | Motolii |
|---|---|---|---|---|
| 2-1 ✔ | **0.1 s 以内 = 自分がやった感じ、1 s = 思考の流れが切れない限界、10 s = 注意の限界** | NN/g Response Times https://www.nngroup.com/articles/response-times-3-important-limits/ — "0.1 second is about the limit for having the user feel that the system is reacting instantaneously" / "1.0 second is about the limit for the user's flow of thought to stay uninterrupted" | NN/g, Apple(2-4) | 値ドラッグ → 絵の更新は **100 ms 以内**(理想は 1 フレーム)。物理・解析の再計算が 100 ms を越すなら代理表示、1 s を越すなら進捗 |
| 2-2 ✔ | **メニュー展開が 0.1 s 未満なら「自分が開けた」と感じる** | NN/g Powers of 10 https://www.nngroup.com/articles/powers-of-10-time-scales-in-ux/ — "If you click on an expandable menu and see the expanded version in less than 0.1 seconds, then it feels as if you made the menu open up." | NN/g | パネル開閉・棚の展開は 100 ms 未満で完了(アニメを付けるなら短い側) |
| 2-3 ✔ | **Material 3 の時間 token: short 50/100/150/200、medium 250/300/350/400、long 450/500/550/600、extra-long 700/800/900/1000 ms。easing emphasized = cubic-bezier(0.2,0,0,1)、emphasized-decelerate (0.05,0.7,0.1,1)、emphasized-accelerate (0.3,0,0.8,0.15)、standard-decelerate (0,0,0,1)、standard-accelerate (0.3,0,1,1)** | material-web tokens v0_192 https://raw.githubusercontent.com/material-components/material-web/main/tokens/versions/v0_192/_md-sys-motion.scss — 値はそのまま(引用は数値) | Material。Flutter の既定もここから | hover/press の色変化は **short(≤200 ms)**、パネル開閉は medium(250〜300 ms)、画面遷移だけ long。**ease は 1 本**(emphasized-decelerate を入り、accelerate を出)。Motolii の「決め切り」の ease と揃える |
| 2-4 ✔ | **フィードバックのアニメは短く正確に。待たせない、途中で取り消せる** | Apple HIG Motion https://developer.apple.com/design/human-interface-guidelines/motion — "Aim for brevity and precision in feedback animations." / "don't make people wait for an animation to complete before they can do anything" | Apple, NN/g | 全アニメは入力で即中断可能(次の入力がアニメを待たない)。ドラッグ中はアニメ無し(値に直結) |
| 2-5 ✔ | **Reduce Motion: ズーム・拡縮・周辺の動きを減らし、遷移はフェードに置き換える** | Apple HIG Accessibility https://developer.apple.com/design/human-interface-guidelines/accessibility — "reducing automatic and repetitive animations, including zooming, scaling, and peripheral motion" / "Replacing transitions in x-, y-, and z-axes with fades" | Apple, Material(2-3 の token を 0 に) | UI 側のアニメ(パネル・棚)は OS の Reduce Motion に従いフェードへ。作品側の動きは対象外 |
| 2-6 △ | **hover の値吹き出しは 2000 ms で自動消灯** | JUCE 1-7 `popupHoverTimeout = 2000` | JUCE | 帯の hover 値。動かさず 2 s で消す |
| 2-7 **未確認** | Figma の "Speed is a feature"(操作は即時、キャンバスは 60 fps) | Figma blog Our approach to designing UI3 https://www.figma.com/blog/our-approach-to-designing-ui3/ — "Speed is a feature"(数値なし) | Figma | 数値の裏取りなし。2-1 で代替 |

## 3. 色と地(dark UI)

| # | 規則 | 出典 + 引用 | 合意 | Motolii |
|---|---|---|---|---|
| 3-1 ✔ | **文字は 4.5:1、大きい文字(18 pt / 14 pt bold)は 3:1** | WCAG 2.2 1.4.3 https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html — "text and images of text has a contrast ratio of at least 4.5:1" / "Large-scale text … at least 3:1" / "at least 18 point or 14 point bold or font size that would yield equivalent size for … CJK" | WCAG, Apple(3-2) | UI 文字(帯の値・棚の名)は 4.5:1。日本語は CJK 等価サイズで判定 |
| 3-2 ✔ | **Apple も同じ表: 17 pt まで 4.5:1、18 pt から 3:1、bold は 3:1** | Apple HIG Accessibility(同上)— 表 "Up to 17 pts: 4.5:1 / 18 pts: 3:1 / Bold: 3:1" | Apple = WCAG | 同上 |
| 3-3 △ | **dark の面は黒でなく #121212、高さは白の overlay で表す: 1dp 5%、2dp 7%、3dp 8%、4dp 9%、6dp 11%、8dp 12%、12dp 14%、16dp 15%、24dp 16%(式 4.5·ln(dp+1)+2 %)** | Flutter elevation_overlay.dart https://raw.githubusercontent.com/flutter/flutter/master/packages/flutter/lib/src/material/elevation_overlay.dart — `opacity = (4.5 * log(elevation + 1) + 2) / 100.0`、コメント "matches the values in the spec"(material.io dark-theme #properties)。m2.material.io 本体は JS 描画で引用を取れず、表の値は式から再計算 | Material(Flutter 実装) | パネル/棚/帯の重なりは影でなく **白 8〜16% の overlay**。手前ほど明るい。Flutter の ThemeData でそのまま使える(標準の仕組みが先) |
| 3-4 **未確認** | Material dark の文字不透明度: 高 87%、中 60%、無効 38%。推奨コントラスト 15.8:1。accent は彩度を落とす(200 tone) | m2.material.io/design/color/dark-theme.html — 1 回目の取得で要約は返ったが、2 回目で本文が取れず**逐語引用なし** | Material | UI 文字の 3 段(値・ラベル・無効)を 87/60/38% で持つ案。裁定前に本文を目で確認 |
| 3-5 ✔ | **テーマは OS の light/dark に追従、色調(warm/cool/neutral)・コントラスト・グリッド不透明度・明るさを利用者が調整できる** | Ableton Live 12 First Steps https://www.ableton.com/en/live-manual/12/first-steps/ — "You can choose a color scheme from the Theme section, or have Live follow the light/dark mode settings from your OS." | Ableton, Figma(UI3 も OS 追従) | テーマは OS 追従を既定。**グリッド不透明度**は設定で(タイムラインの升目) |
| 3-6 ✔ | **選択は青の枠 1 色、hover も青の箱** | Figma Select layers https://help.figma.com/hc/en-us/articles/360040449873 — "All selected objects are wrapped in a blue bounding box." / "a blue box will highlight that layer's location on the canvas" | Figma | ステージの選択色は 1 色。hover は同色の細い枠、選択は太い枠 + ハンドル。作品の色と混ざらない UI 色を 1 つ決める |
| 3-7 ✔ | **スナップの案内線は赤 1 色** | Figma Adjust alignment(1-11 と同頁)— "A red guide appears on the canvas as a visual indicator." | Figma | スナップ線は選択色(青)と別の 1 色 |

## 4. 間隔と大きさ

| # | 規則 | 出典 + 引用 | 合意 | Motolii |
|---|---|---|---|---|
| 4-1 ✔ | **macOS の当たり判定: 既定 28×28 pt、最小 20×20 pt**(iOS は 44/28) | Apple HIG Accessibility(同上)— 表 "macOS: 28x28 pt default, 20x20 pt minimum" | Apple | Flutter デスクトップの帯・ボタン・ハンドルは **28 pt 基準、20 pt を下限**。密な帯でも 20 pt を割らない |
| 4-2 ✔ | **ボタンの当たり判定は原則 44×44 pt** | Apple HIG Buttons https://developer.apple.com/design/human-interface-guidelines/buttons — "a button needs a hit region of at least 44x44 pt … whether they use a fingertip, a pointer, their eyes, or a remote." | Apple(タッチ) | タッチ対応時のみ。デスクトップは 4-1 |
| 4-3 ✔ | **ポインタの当たり判定は見た目より広く: 縁あり要素は約 12 pt、縁なしは約 24 pt の余白** | Apple HIG Pointing devices https://developer.apple.com/design/human-interface-guidelines/pointing-devices — "~12 points of padding" / "~24 points of padding around the element's visible edges" | Apple | ステージのハンドル(見た目 6〜8 px)の当たりは 24 pt。帯の細い区切りも同じ |
| 4-4 ✔ | **WCAG: ポインタ標的は 24×24 CSS px 以上、または 24 px 円が隣と重ならない** | WCAG 2.2 2.5.8 https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html — "The size of the target for pointer inputs is at least 24 by 24 CSS pixels" | WCAG, Apple(4-1 の 28 と整合) | 4-1 と合わせて **下限 24 px**(Apple の 20 より厳しい方を取る) |
| 4-5 ✔ | **画像ボタンは画像と縁の間に約 10 px** | Apple HIG Buttons(同上)— "Include about 10 pixels of padding between the edges of the image and the button edges." | Apple(macOS) | 棚のカードのアイコン・帯のアイコンボタン |
| 4-6 ✔ | **パネルは幅を変えられる、UI は全部隠せる、ツールバーは細く下に** | Figma blog Inside the Redesigned Figma https://www.figma.com/blog/behind-our-redesign-ui3/ — "We've made the panel resizable" / "You can hide the UI completely, with panels only appearing when needed" | Figma, Ableton(Info View の表示切替) | Inspector と棚は幅可変・全隠し可。ステージが主 |
| 4-7 ✔ | **ピクセル格子は 400% 以上でだけ見せる** | Figma Zoom https://help.figma.com/hc/en-us/articles/360041065034 — "The pixel grid is only visible at zoom levels of 400% or higher." | Figma | ステージの px 格子は拡大時だけ。タイムラインの升目も密度に応じて間引く |
| 4-8 ✔ | **ズームは Shift+ / Shift−、全体 = Shift 1、選択 = Shift 2** | 同上 — "Zoom in: Shift +, Zoom out: Shift −, Zoom to fit: Shift 1, Zoom to selection: Shift 2" | Figma。Ableton は Ctrl± で窓全体、± で時間軸 | ステージ: fit と selection の 2 つを 1 打鍵で。タイムラインの時間軸ズームは別キー |
| 4-9 ✔ | **主窓の拡大率(zoom)を設定で持つ(第 2 窓も別に)** | Ableton First Steps(同上)— "set the zoom amount for Live's main window (as well a second window, if open)" | Ableton, Figma UI3 | UI 全体の倍率設定(Flutter の textScale でなく画面倍率)。文字の pt は固定せず倍率で |
| 4-10 **未確認** | Figma UI3 の本文 11 px、パネル 240 px 等の具体値 | Figma blog 2 本に数値なし(4-6)。Pattern Library 記事も token 数(680+)のみ | — | 裁定前に UI3 実物を測るか、Flutter の既定(bodySmall 12 / labelSmall 11)を採る |

## 5. 選択と focus

| # | 規則 | 出典 + 引用 | 合意 | Motolii |
|---|---|---|---|---|
| 5-1 ✔ | **hover = 青い箱、選択 = 青い枠、Shift-click で追加、Cmd/Ctrl-click で深い選択、空き地からドラッグで矩形選択** | Figma Select layers(3-6 と同頁)— "Hold Shift and click on another object." / "Hold down the modifier key to select … a nested layer or object by clicking it on the canvas" / "Click and hold on an empty part of the canvas. Drag" | Figma, FabFilter(5-2) | ステージ: 同じ 5 動作。箱(mask/comp)の中の物は Cmd-click で深く |
| 5-2 ✔ | **Ctrl/Cmd-click で複数、Shift-click で連続範囲** | FabFilter Band controls(同上)— "Hold down Ctrl (Command on macOS) and click another dot to select multiple bands. Hold down Shift and click a dot to select a consecutive range" | FabFilter。Figma は Shift = 追加(5-1) | **衝突**: Shift の意味(追加 vs 範囲)。Motolii はステージ = Figma 型、タイムライン/棚(順序あり) = FabFilter 型で分ける案 |
| 5-3 ✔ | **選ぶと制御が対象の直下に浮く(選択に付いて来る)** | FabFilter Band controls — "the floating band controls will automatically appear, right under the selected bands at the bottom of the display" | FabFilter | ステージで物を選ぶと、その物の帯が近くに出る(Inspector まで目を飛ばさない) |
| 5-4 ✔ | **hover で「押したらどうなるか」の予告(曲線の薄い下書き)** | FabFilter Display — "When you hover anywhere in the display, a subtle curve preview will appear, indicating the exact curve." | FabFilter | スナップ先・追加位置・ブロック挿入の予告を薄く出す |
| 5-5 ✔ | **hover した要素の名と役目を常設の枠(Info View)に出す、tooltip でなく** | Ableton First Steps — "The Info View displays the name and function of whatever element of the UI that you hover over with the mouse." Shortcuts — "Shift ?: Hide/Show Info View" | Ableton | Status/Info の 1 行を常設。tooltip の待ちを無くす |
| 5-6 ✔ | **hover で 1 つ選ぶと Status Bar に正確な位置** | Ableton Live Concepts https://www.ableton.com/en/live-manual/12/live-concepts/ — "When hovering over an insert marker …, the Status Bar displays the marker's precise location." | Ableton | タイムライン hover で時刻、ステージ hover で座標を Status に |
| 5-7 ✔ | **ポインタ効果は控えめ、無駄な効果を作らない** | Apple HIG Pointing devices — "Avoid creating gratuitous pointer and content effects." / "The subtle highlighting and movement bring focus to the control without distracting" | Apple | hover の拡大・影は付けない。色 1 段の変化だけ |
| 5-8 ✔ | **ラベルは切り替え可(新人は on、慣れたら off)** | Figma blog(4-6 と同頁)— "Turn on labels to quickly understand what each control does, or turn them off to focus on your work" | Figma | 帯のラベル表示を設定で。段差を消す入口 |

## 6. 文字

| # | 規則 | 出典 + 引用 | 合意 | Motolii |
|---|---|---|---|---|
| 6-1 ✔ | **動く数字・縦に並ぶ数字は等幅数字(tabular)、本文は proportional** | Practical Typography https://practicaltypography.com/alternate-figures.html — "tabular figures are essential for one purpose: vertically aligned columns, like you find in grids of numbers." | Butterick。CSS/OpenType(6-2) | 帯の値・タイムコード・Inspector の数値は tnum。ラベルは pnum |
| 6-2 ✔ | **OpenType `tnum` = 全数字が同じ幅、`pnum` = 幅が違う。CSS は font-variant-numeric: tabular-nums** | MDN font-variant-numeric https://developer.mozilla.org/en-US/docs/Web/CSS/font-variant-numeric — "numbers are all of the same size, allowing them to be easily aligned like in tables. It corresponds to the OpenType values tnum." | W3C/MDN | Flutter: `FontFeature.tabularFigures()`。フォント選定で tnum を持つ物を条件に |
| 6-3 ✔ | **値の表示は 5 桁・小数 5 桁が上限、文字高さは箱の 70%** | Vital synth_slider.h — `kDefaultFormatLength = 5; kDefaultFormatDecimalPlaces = 5;` / `kDefaultTextHeightPercentage = 0.7f` / `kDefaultTextEntryHeightPercent = 0.35f` | Vital | 帯の値は 5 桁で丸める、文字は帯高の 70%、入力欄は 35% |
| 6-4 ✔ | **数値入力は略記を受ける("2k"、"A4")** | FabFilter Band controls(1-9) | FabFilter | 時間は "1s"・"12f"、角度は "45d"、色は "#fff" を受ける |
| 6-5 **未確認** | Ableton の UI フォントサイズ(pt)の明記 | Live 12 manual に記載なし。zoom 設定のみ(4-9) | — | 固定 pt を決めず倍率で(4-9) |
| 6-6 **未確認** | Refactoring UI の「大きさでなく色と太さで階層」「余白は多めから」 | https://www.refactoringui.com/ は販売頁で章題のみ("Balance weight and contrast" / "Start with too much white space")。本文の引用なし | — | 章題のみ参照。DESIGN.md には書かない |

## 7. 音

| # | 規則 | 出典 + 引用 | 合意 | Motolii |
|---|---|---|---|---|
| 7-1 ✔ | **音だけで大事な情報を伝えない。無音モードでは利用者が始めた音だけ鳴らす** | Apple HIG Playing audio https://developer.apple.com/design/human-interface-guidelines/playing-audio — "avoid communicating important information using only sound" / "When a device is in silent mode, it plays only the audio that people explicitly initiate, like media playback" | Apple | UI 効果音は付けない(DAW を再発明しない)。鳴るのは作品の音と再生だけ。エラーは Status の文字 |
| 7-2 **未確認** | Ableton はメトロノーム以外に UI 効果音を持たない | 文書上の「UI 音なし」の明言は見つからず(挙動として周知) | Ableton, Figma | 7-1 で足りる |

---

## 採用した寸法(2026-09-20、修正)

利用者裁定: Motolii の芯は Ableton の高密度で、カード(写真 + 名札)の薄い外箱のシルエットは意図した形。**行を広げる変更は取り消した。**

| 値 | 根拠 | 確認 |
|---|---|---|
| **row 20 / control 24 / section 26 / bar 28 / tall 30**(元の寸法表) | Apple HIG Accessibility の表 — macOS の当たり判定は既定 28×28 pt、**最小 20×20 pt**。row 20 は Apple の下限に一致 | 台帳 4-1 の記録どおり。2026-09-20 は JS 描画で再取得できず、**未再確認** |
| WCAG 2.2 SC 2.5.8 の 24 px | 原文は 2026-09-20 に確認済み(4-4)。例外の「間隔」は、行が隣と接する密な帯では成り立たない | **密な帯は満たさない。既知の逸脱として受ける** |

押せる範囲は、行の高さ(20)を下限とする。検査(`craft_sweep_test`)の基準は 20 pt。24 を満たすには、見た目を変えずに当たりだけ広げる設計が要る(未着手)。

## 衝突(裁定待ち)

1. **Shift の意味**: Ableton/FabFilter/JUCE = 細かい(1-1, 1-6)、Figma = 大きい nudge(1-11)・範囲/拘束。→ 値ドラッグでは細かい、ステージの矢印では 10 px、が両方の先例通り。文書に両方書く
2. **double-click の意味**: Ableton/Vital = 既定へ(1-3)、FabFilter = 入力(1-9)。→ 帯・円形は既定へ、数字テキストは入力
3. **軸拘束の修飾キー**: FabFilter = Alt(1-6)、Figma = Shift(周知、未引用)。→ ステージは Figma 型(Shift)で、帯は Alt 不要
4. **選択追加**: Figma = Shift(5-1)、FabFilter = Ctrl + Shift 範囲(5-2)。→ 場所で分ける(5-2 の注)
5. **当たり判定の下限**: Apple macOS 20 pt、WCAG 24 px。→ 24 を取る(4-4)

## DESIGN.md Do/Don't 草案(20 行)

Do
1. 値ドラッグは全域 200〜250 px、Shift で ×0.1、押した所へ飛ばさず相対で動かす(1-4, 1-5, 1-7)
2. Delete と double-click で既定値へ戻す。数字テキストは double-click で入力欄(1-2, 1-3, 1-9)
3. hover の値は遅延 0 で出し、離れたら即消す、動かなければ 2 s で消す(1-8, 2-6)
4. 入力 → 絵は 100 ms 以内。越えるなら代理表示、1 s を越えるなら進捗(2-1)
5. UI アニメは short(≤200 ms)で色、medium(250〜300 ms)で開閉、ease は 1 本、入力で即中断(2-3, 2-4)
6. Reduce Motion で UI の動きはフェードに落とす(2-5)
7. 文字は 4.5:1(CJK 等価サイズで判定)、重なりは影でなく白 8〜16% の overlay(3-1, 3-3)
8. 選択は 1 色(青)、スナップ線は別の 1 色(赤)、hover は同色の細枠(3-6, 3-7)
9. 当たり判定は見た目より広く: 下限 24 px、基準 28 pt、ハンドルの縁なし余白 24 pt(4-1, 4-3, 4-4)
10. 動く数字は tnum、5 桁で丸め、略記入力を受ける(6-1, 6-3, 6-4)
11. 選んだ物の帯は物の近くに出す。hover の名と役目は常設の Info 行へ(5-3, 5-5)
12. パネルは幅可変・全隠し可、ラベルは切替可、格子は拡大時だけ(4-6, 4-7, 5-8)

Don't
13. Shift 以外の修飾キーで「細かく」を作らない(Vital の Cmd は採らない)(1-1)
14. tooltip の待ち時間を値表示に使わない(1-8)
15. ドラッグ中にアニメを挟まない。値は直結(2-4)
16. hover で拡大・影・ばねを付けない。色 1 段だけ(5-7)
17. 純黒 #000 の地、影による重なり表現を使わない(3-3)
18. UI 効果音を付けない。鳴るのは作品と再生だけ(7-1)
19. 24 px を割る標的、20 pt を割るボタンを作らない(4-1, 4-4)
20. 未確認の行(1-14, 3-4, 4-10, 6-5, 6-6, 7-2)を DESIGN.md に写さない — 引用が取れてから

## 取得できなかった一次資料(再挑戦の手順)

- m3.material.io の motion token 頁と m2 dark theme 頁は JS 描画で WebFetch では本文が来ない → ブラウザで開くか、material-web の tokens(取得済)/Flutter 実装(取得済)を一次資料の代わりに使った
- Ableton Live 12 manual の「数値欄の縦ドラッグ」の章は見つからず(Live Concepts・Clip View・Mixing を確認)。PDF 版で "drag" を検索する
- Figma UI3 の具体値(px)は blog 2 本と Pattern Library 記事に無い
