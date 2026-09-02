# UI motion — 因果をつなぐ最小の動き

日付: 2026-09-02

目的は賑やかさではなく、0msで切り替わっていた状態の前後関係を目で追えるようにすること。
Stage、Timeline、splitter、scrub、drag等の直接操作は補間せず、入力へ同じframeで追従させる。

## 現窓baseline

- `styles.css`に`transition`／`animation`／motion tokenは0件
- hover、pressed、selected、menu、Inspector choice、panel切替、drop guideが全て瞬時に切り替わる
- playhead、Stage、Timeline、splitter、dock ghostは既に入力／作品時刻から直接描く。ここへUI補間を足すと意味が遅れる
- pin済みBlitzはStyloのCSS animation/transitionを持ち、active中はwindow frame loopが`resolve(time)`を継続する。別timerは不要

## 検索receipt

| field | 内容 |
|---|---|
| `NEED` | 触った結果は出るが、hover／selection／menu／panelの前後が0msで飛び、操作全体がカクついて見える |
| `SEARCHED` | 現行CSS／semantic component、品質バーQ3/Q4、pin済みBlitz animation loop、Apple HIG Motion／Accessibility、Material 3 motion token、W3C reduced motion、AppKit `NSWorkspace` |
| `DISPOSITION` | **REUSE_REMAP**。animation runtime、timer、spring、状態機械を作らず、Blitz CSS transitionへ外部tokenを写す |
| `OWNER/ROUTE` | `tokens.rs motion token → styles.css共通state family → SemanticMenu/Button、tab、card、choice、drop feedback`。Document／Undoへ書かない |
| `RULER` | [Apple Motion](https://developer.apple.com/design/human-interface-guidelines/motion)は目的的・短く精密・頻出操作を待たせないことを要求。[Material Motion](https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md)は100/150/200msとstandard curveをtoken化。[W3C C39](https://www.w3.org/WAI/WCAG21/Techniques/css/C39.html)はsystem reduced-motionでinteraction motionを止める |
| `ORACLE` | controlled animation clockでmenu opacityが0→中間→1。reduced時は全motion token 0ms。直接操作property(left/top/width/height)をtransition対象にしない。実窓でmenu/tab/card/choiceを連続操作して入力待ち・残像・click-through 0 |

## 採る契約

| family | duration | easing | 対象 |
|---|---:|---|---|
| `direct` | 0ms | linear | Stage、Timeline、scrub、splitter、scroll、drag ghost座標、playhead |
| `fast` | 100ms | standard | hover、pressed、focus、drop target |
| `state` | 150ms | standard | selected、tab、card、key／mute状態、Inspector choice |
| `enter` | 200ms | standard-decelerate | menu、panel、file-drop feedbackの一度だけの出現 |

Material 3の`short2/short3/short4`とstandard curve
`cubic-bezier(0.2, 0, 0, 1)`、enterのstandard-decelerate
`cubic-bezier(0, 0, 0, 1)`をそのまま使う。

動かしてよいのは`opacity`、`background-color`、`color`、`border-color`、`box-shadow`と、
出現時だけの2px以下のtranslate。layout寸法、left/top、scroll、作品のpixelは動かさない。

## Reduce Motion

macOS製品窓ではAppKit `NSWorkspace.accessibilityDisplayShouldReduceMotion`を読む。
trueなら同じtokenを0msへ差し替える。状態の色・border・selected表現は残すので、motionだけを消しても意味は消えない。
headless testにはWindow contextが無いためAppKitへ触れず、明示した`reduced`入力で同じCSS tokenを検査する。

## 受入

1. token値とreduced=0をunitで固定
2. headless animation clockでmenu enterに中間frameが実在
3. GUI既存62 testを維持。menu dismiss、click-through、dock drag、gesture stormを壊さない
4. 実窓でFile/View、panel tab、Browser card、Inspector choiceを素早く往復し、操作受付は即時、見た目だけ短く接続
5. 全93 test、docs、diff check

## 実装結果

- `tokens.rs`へ`direct / fast / state / enter`と2 easingを一度だけ定義
- `styles.css`のbutton、menu、tab、card、selection、Inspector choice、drop feedbackへ
  color／border／opacity／shadowだけを共通適用
- menu／panelはmount時だけ2px以下のtranslate + fade。dock ghostのleft/top、Stage、Timeline、splitterは補間0
- macOS製品窓はAppKit `NSWorkspace.accessibilityDisplayShouldReduceMotion`を読み、trueなら全duration tokenを0msにする
- headless／他OSはAppKitへ触れず、同じCSS contractを維持

## 検証結果

| oracle | 結果 |
|---|---|
| Material token写像／Reduce Motion | `100 / 150 / 200ms`とreduced時`0 / 0 / 0ms` PASS |
| 直接操作の除外 | `transition: all`、left/top/width/height/transform transition 0件 PASS |
| animation clock | menuはcommand直後にDOMへ存在。1/60秒刻み12frameのうち6frame以上でopacityが単調増加し200msで1.0 PASS |
| 既存GUI | menu dismiss/click-through、dock drag、focus loss、Mask、gesture stormを含むlib 65/65 PASS |
| 実窓 | File/View高速往復、Media/Create/Effects高速往復、Create card、Inspector choiceに残像／blank／click-through 0。Stage dragは即時にPositionへ反映 |
| 全回帰 | `cargo test --locked --no-fail-fast` 96/96 PASS |

採らなかった物: spring、bounce、blur、layout morph、playhead補間、Stage／Timelineの見せかけの追従。
これらは美しさでなく意味遅延を作るため、最低限のmotionへ入れない。

現在判定: **PASS**。
