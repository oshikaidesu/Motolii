# UI視覚言語(M3)

ステータス: **設計基準**(意味役割と禁止事項は決定。具体token値とreference screenはM3 G0-6で確定)

この文書は操作モデルではなく、Motoliiの見た目と視覚認知の基準を定める。UI toolkit、Document意味論、入力契約は変更しない。操作トポロジーと共通component契約は[UI操作言語](ui-interaction-language.md)、実装境界は[M3 UI境界汚染の予防](reviews/2026-07-14-m3-ui-boundary-prevention.md)、タスクと審判は[M3仕様](specs/M3-ui-integration.md)を正本とする。

## 参照範囲を混ぜない

| 対象 | 参照するもの | 参照しないもの |
|---|---|---|
| 操作動線 | OpenCut、Flow / Alight Motion、一般的なトラック型UI。Abletonは一画面・固定役割・選択→詳細・Info View・評価順と配置の一致という操作トポロジーだけ | Ableton固有のDAW意味論、Session View |
| タイムラインの視覚言語 | AbletonのTimeline Viewに見られる密度、階層、色による識別 | Ableton Arrangement Viewの画面構成をそのまま移植すること |
| 全体の仕上げ | Ableton / Appleの抑制された面、整った余白、明確な階層、一貫したicon | glassmorphism、装飾gradient、生成物ごとに揺れるスタイル |
| keyframe easing | Flow / Alight Motionの区間選択→popup編集 | AEの値グラフを主画面へ常設すること |

OpenCut等は操作と配置、Abletonは制作ソフトの操作トポロジーと視覚言語、Appleは抑制された視覚階層の先例である。Abletonの参照範囲は[UI操作言語](ui-interaction-language.md#21-主参照はプロ用ソフトの操作トポロジー)に限定し、「Ableton風」を理由にArrangement Viewの画面構成やDAWの概念を持ち込まない。

## 設計目標: 読む前に分かる

AEのように無彩色と文字ラベルへ識別を寄せすぎない。ユーザーが文字を読む前に、少なくとも次を位置・形・icon・意味色の組み合わせで判別できることを目標にする。

- 選択中 / hover / keyboard focus
- 映像・音声・shape・text・group等の項目種別
- 再生中、record/arm相当、solo、mute、disabled、warning/error
- keyframe、automation/easing、link/reference、cache/bake状態
- 実行できる操作、現在値、変更済み/既定値

色だけには依存しない。意味色にはicon、形、線種、pattern、位置のいずれかを必ず併用する。文字は正確な名称、数値、tooltip、screen reader用に残すが、主要状態の唯一の識別手段にしない。

## ポップさやケレン味でUXを代替しない

Motoliiの親しみやすさは、彩度、丸み、animation、キャラクター性、賑やかなempty stateから作らない。これらは必要な意味を補助する場合だけ使え、操作の因果が弱いことを隠す用途には使えない。

- 目立たせる前に、対象、scope、順序、状態、次の操作が配置と形で分かるようにする。
- 楽しさを演出する前に、入力への応答、preview、Cancel、Undo、error理由を確実に返す。
- 高機能に見せる前に、同じ目的のcontrolとmodeを減らし、残した語彙を全画面で再利用する。
- 新機能の存在感を独自色や大きなcardで作らず、既存のInspector、Timeline、Stage、badgeへ投影する。
- 装飾を無効化し、motionを0にし、grayscaleにしても、情報階層と主要状態が成立しなければ不合格とする。

抑制は無個性や無彩色を意味しない。意味色、直接操作中のghost、接続仮線、warning等、理解を速める表現には十分な面積とcontrastを与える。削る対象は情報ではなく、情報を装って注意を奪う演出である。

## 面・色・文字・iconの規約

### 面

- 土台はdark neutral。階層は小さな明度差と境界線で作り、カードやshadowを増殖させない
- 装飾だけのgradient、neon glow、glassmorphism、全controlのpill化を禁止する。gradientがデータそのものを表す場合(color ramp等)は例外
- panelごとに別の意匠を発明しない。同じ役割は同じsurface、radius、spacing、strokeを使う
- 情報密度は制作ツールとして保ち、Apple風を「大きな余白と巨大control」の意味にしない
- popさを出すためだけの高彩度面、過剰な丸角、sticker/emoji、celebration、bounceを共通componentへ入れない

### 意味色

具体色値はG0-6でcontrast測定後に固定し、実装中に都合のよい色を足さない。最低限のtoken roleは次とする。

| role | 意味 | 併用する手掛かり |
|---|---|---|
| `accent.selection` / `accent.focus` | 選択とkeyboard focus | outline、focus ring |
| `item.video/audio/shape/text/group` | timeline項目種別 | icon、clip形状/stripe |
| `state.active/play` | 有効・再生中 | icon、位置 |
| `state.solo` / `state.mute` | solo / mute | 固有icon、明度/線種 |
| `state.keyframe` / `state.automation` | keyframe・easing | diamond/curve icon |
| `state.linked` / `state.cached` | 参照・cache/bake | chain/cache iconまたはpattern |
| `feedback.warning` / `feedback.error` | 注意・失敗 | warning/error icon、文言 |
| `content.primary/secondary/muted/disabled` | 文字とcontrolの重要度 | font weight、opacity、操作可否 |

1 roleを複数の意味へ流用せず、同じ意味を画面ごとに別色へしない。項目種別色と状態色が競合する場合、項目種別は面/stripe、状態はoutline/iconのようにchannelを分ける。

track/clipごとの任意色をDocumentへ保存するかは未決である。M3の見た目を理由にschema fieldを追加しない。最初の実装は既存の項目種別と状態から決定的に色を導出できるが、任意色の永続化が必要ならGR-PVを通す。

### 文字とicon

- 通常文字は背景とのcontrast 4.5:1以上、大きい文字または太字は3:1以上をG0-6の初期基準とする
- primary / secondary / muted / disabledをtoken化し、raw opacityの場当たり指定を禁止する
- iconは単一のgrid、stroke、サイズ系列、角の語彙へ揃える。emoji、異なるicon setの混在、画面ごとの即席生成を禁止する
- iconだけで曖昧な操作には短いlabelまたはtooltipを付ける。ただし状態識別をlabelだけへ戻さない

### 接続操作の視覚文法

型付き接続はiconだけで開始・方向・結果を推測させない。平常時は接続controlの短いlabelまたはtooltipで開始方法を示し、接続mode中はカーソル近傍へ「何を・何へ・どう繋ぐか」の短文を常時追従表示する。valid targetはoutline+形、invalid targetはdim+拒否理由、hover targetは仮線または同等のfrom/to手掛かりで示し、色だけへ依存しない。確定後は接続元と接続先を読めるsemantic badgeをInspector/Timelineの既存語彙へ残す。

この文法はLookAt / Follow / Parent / DataTrack / Effect Use等で共通化し、機能ごとに別のウィップ外観やdrag規則を発明しない。接続説明、hover、仮線はTransientなUI投影でありDocument意味ではない。Advanced表示も同じ接続を詳しく検査するだけで、説明のない`force connect`や型不一致を色だけで警告する入口を作らない。操作意味と例外の採用条件は[操作単純化モデル S-3a](interaction-simplicity-model.md#s-3a-接続操作はカーソル自身が意味を説明する)を正本とする。

## 実装時に発明しない領域

以下はLLM実装者が指定なしだと場当たりに発明しやすい領域である。ここに基準がある事項は、実装中に独自判断で置き換えない。具体値の確定はG0-6で行う。

WCAGはWeb content向けの規格であり、native desktop UIであるMotoliiがWCAG適合を名乗る根拠には使わない。以下のcontrast/focus値は、測定方法が公開されている先例をMotoliiの製品基準として借用するものとする。

### 非テキストcontrastとfocus外観

- 文字だけでなく、UI部品の境界・状態を示すindicator・意味を持つicon/グラフィックも、隣接色と3:1以上のcontrastを初期基準とする(WCAG 2.2 SC 1.4.11)
- focus indicatorは「2 logical pxの周囲線に相当する面積以上」「focus有無の同一pixel間の差が3:1以上」をWCAG 2.2 Focus Appearance(AAA)から参考値として借り、G0-6のfocus ring仕様に使う。加えてfocus中のindicatorは隣接色とも3:1以上を満たす。focusを面の微妙な明度変化だけで表さない

### 色覚多様性

- grayscale審判に加え、protanopia / deuteranopia / tritanopiaのCVDシミュレーション画像をG0-6で自動生成する。simulation単独の合否判定はせず、意味の衝突を人間審判する
- 項目種別色の候補選定はOkabe-Ito 8色(Color Universal Design由来)を色相間隔の出発点にし、hex値をそのまま移植しない。dark背景上の面色・線色として再調整し、通常表示と3種simulationの両方で測定する

### tokenの格納形式

- theme tokenはDesign Tokens Community Group format v2025.10準拠のJSONを正本とし、Rust / Slint側は決定的に生成する。v2025.10はDTCG初のstable reportだが、W3C Standards Trackの標準ではない
- 「raw color literalをtheme file外で拒否する」審判は、この単一正本があって初めて機械検査になる。tokenをコード内定数へ複製しない
- generatorのversion、入力hash、生成先を固定し、生成物の手編集と未生成差分をCIで拒否する。DTCGが定めるのは交換formatであり、Motolii内のrole命名や階層は本書が定める

### dark面の明度階層

- 土台は原則として純黒(`#000000`)を使わず、dark grayから始める。Material 2の`#121212`は歴史的先例であり、Motoliiの固定値にはしない
- 階層は`canvas / panel / raised / overlay`等の意味roleとして必要最小数を定義する。一定率の機械的加算やMaterialのelevation overlayを移植せず、隣接境界・文字・iconのcontrastを満たす値をG0-6で個別に固定する

### 数値とtimecodeの文字

- timecode、フレーム番号、パラメータ値、ドラッグ中に変化する数値は、tabular(等幅)数字で表示し、値の変化で幅が揺れないことをG0-6の検査に含める
- 等幅が必要な列(timecode等)はmonospaced書体またはtabular lining機能を使う(SF Mono / SF Proのtabular liningが先例)

### iconの物理仕様

- 単一grid・単一stroke幅・統一されたcap/joinを最初に固定する。Lucideの24×24 grid、2px stroke、round cap/join、単純なpath/shape要素は具体的な候補であり、Motoliiの高密度timelineで20px/24px表示を比較してG0-6で採否を決める
- 採択した物理仕様には自作iconも従い、既成setと自作の混在で線の太さ・角・optical volume・detail量が揺れないことを検査する。Lucide採用を決めるまでは、そのicon assetを製品へ取り込まない

### UI motion

- トランジションのdurationとeasingもtoken化し、componentごとに即席の数値を書かない(Material 3のduration / easing token体系が先例)
- OSのreduce motion設定を尊重し、motionを0にしても状態変化が判別できること(= motionは装飾であって唯一の手掛かりにしない)を規約とする
- 再生・スクラブ等のデータ由来の動きはUI motionと別扱いで、reduce motionで殺さない

## 既存UIへ馴染ませる規約

新規機能は「目立つ新部品」として置かず、既存のtrack row、clip、toolbar、inspector、popupの語彙へ割り当てる。新componentを作る前に次の順で判断する。

1. 既存componentの状態variantで表せるか
2. 既存componentの組み合わせで表せるか
3. 新componentが必要なら、同じspacing・radius・stroke・icon grid・意味色で作れるか
4. それでも独自の見た目が必要なら、理由と比較fixtureをG0-6の判断記録へ残す

「違和感がない」は目視だけで完了にしない。既存機能と新規機能を同じscreenへ置いたreference screenを作り、theme token外のraw color、独自spacing、独自iconを機械検査する。

## G0-6の審判

次の5画面を同一の固定Document fixtureから作る。

初期の構成比較には[M3 timeline v0 mock](mocks/README.md)を使う。モック内の色値はtoken候補ではなく、構成と意味roleの比較専用とする。

1. empty project + asset browser
2. video/audio/shape/text/groupを含むtimeline
3. 選択項目のparameter panel + keyframe/easing popup + warning/disabled状態
4. Stage + Output Frame + frame内外object + Camera tool / Hand(Stage View) tool。枠外を不透明塗潰しせず半透明scrimで見せ、Output Frame境界、出力外、選択を文字だけ・色だけにせず識別する
5. 非隣接layer 3つが同じEffect Definitionを異なるstack位置で使うtimeline。固定connection gutterの常時線、from/outとuse/in、折畳みstub+件数、通常drag/Relative drag HUDを同時に検証する

自動審判:

- screenshot goldenをlightness差分と通常差分で比較する
- DTCG token JSONをschema検証し、Rust/Slint生成物の手編集・生成漏れ・theme外raw color literalを拒否する
- 通常文字4.5:1、大文字/太字3:1を下回るtoken pairを拒否する
- 意味を持つUI境界/icon/indicatorの隣接contrast 3:1と、focusの面積・状態間contrast・隣接contrastを検査する
- gradientは許可listにあるデータ表現だけに限定する
- component state matrixで選択/hover/focus/disabled/errorの欠落を拒否する
- timecode/数値表示の幅不変fixture、motion token外のduration/easing literal、reduce-motion時の状態欠落を検査する
- 通常/grayscale/protan/deutan/tritanのreference imageを同じfixtureから生成する

人間審判:

- labelを読まず、5秒以内に選択項目・項目種別・mute/disabled・keyframe・warningを指せるか
- grayscale表示でも選択、focus、disabled、warningを区別できるか
- protan/deutan/tritan simulationでも項目種別と状態色の意味が衝突しないか
- Stage上でOutput Frame内外と選択状態を区別でき、Camera操作とHand/Stage View操作のどちらが作品を変えるかを5秒以内に指せるか
- 非選択状態でもEffectのfrom/outと各layerのuse/inを追え、Group合成後1回とExplicit個別適用を5秒以内に区別できるか。線がclip/keyframe読取りを妨げていないか
- 通常dragとmodifier+drag Relative Moveを、HUDとmotion path ghostから操作前に区別できるか
- 既存componentと追加componentのうち、追加分だけが別製品のように浮いていないか
- Timeline View案とMotolii案を同じ情報量・viewportで比較し、Arrangement Viewで感じた一覧性の低さを再生産していないか
- 彩度を落としmotionを無効にした状態でも、target、選択、評価順、warning、Commit/Cancel結果を説明できるか。説明できない場合、派手さで因果不足を覆っていると判定する

5秒は認知速度の科学的な普遍値ではなく、M3内で比較を揃えるための製品基準である。単独案の印象評ではなく、同じfixture・viewport・taskで比較する。

## 根拠の強さ

- **製品決定**: semantic colorを増やし、文字を読む前の識別を優先する。装飾gradientを使わない。新規UIを既存語彙へ馴染ませる
- **先例**: Abletonはclip/track colorと一貫した意味色を使う。Apple HIGは明確な階層、一貫性、認識可能なcontrol、色以外の手掛かり併用を求める
- **現時点の個人評価**: Ableton Arrangement Viewは一覧性が低く、参照対象にしない。この評価を一般事実とは扱わず、上記の同一fixture比較でMotolii案を審判する

規範として借用する一次資料:

- [Ableton Live Manual — Clip View](https://www.ableton.com/en/manual/clip-view/)
- [Ableton Live Manual — Live Concepts](https://www.ableton.com/en/manual/live-concepts/)
- [Apple Human Interface Guidelines — Design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles)
- [Apple Human Interface Guidelines — Color](https://developer.apple.com/design/human-interface-guidelines/color)
- [Apple Human Interface Guidelines — Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility)
- [Apple Human Interface Guidelines — Icons](https://developer.apple.com/design/human-interface-guidelines/icons)
- [Apple Human Interface Guidelines — Typography](https://developer.apple.com/design/human-interface-guidelines/typography)(tabular lining数字、SF Mono)
- [WCAG 2.2 — Understanding SC 1.4.11 Non-text Contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html)
- [WCAG 2.2 — Understanding SC 2.4.13 Focus Appearance](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html)
- [Design Tokens Format Module 2025.10(DTCG)](https://www.designtokens.org/tr/2025.10/format/)
- [DTCG FAQ](https://www.designtokens.org/faq/)(stable範囲とW3C Standards Track外の明記)

設計先例(値をそのまま規範化しない):

- [Material Design — Dark theme](https://design.google/library/material-design-dark-theme)(純黒を避けたdark grayと低contrast階層)
- [Material Design 3 — Easing and duration tokens](https://m3.material.io/styles/motion/easing-and-duration/tokens-specs)
- [Okabe-Ito palette(Color Universal Design)](https://siegal.bio.nyu.edu/color-palette/)
- [Lucide — Icon Design Guide](https://lucide.dev/contribute/icon-design-guide)

「実装時に発明しない領域」の各基準は2026-07-14のweb調査に基づく。docs/reviews/README.mdの規律に従い、G0-6で具体値を固定する前に反対側レビューの対象とする。規格、製品基準、設計先例を混同して値を追加しない。
