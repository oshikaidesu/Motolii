# UI視覚言語(M3)

ステータス: **設計基準**(意味役割・情報密度・禁止事項は決定。具体token値とreference screenはM3視覚確定(G0-6)で確定)

この文書は操作モデルではなく、Motoliiの見た目と視覚認知の基準を定める。UI toolkit、Document意味論、入力契約は変更しない。操作トポロジーと共通component契約は[UI操作言語](ui-interaction-language.md)、実装境界は[M3 UI境界汚染の予防](reviews/2026-07-14-m3-ui-boundary-prevention.md)、タスクと審判は[M3仕様](specs/M3-ui-integration.md)を正本とする。

視覚構成の基準は[高密度メインUIモック](mocks/m3-main-ui-v1.html)とする。HTMLは設定画面からライト/ダークを実際に切り替えられ、[ライト静止画](mocks/m3-main-ui-v1-light.png)と[ダーク静止画](mocks/m3-main-ui-v1-dark.png)は回収時点の構成比較証拠として固定する。モックにある未決機能は実装決定ではなく、画面密度のfixtureである。

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

## 情報を隠さず、一覧で所在を示す

Motoliiは制作ツールなので、一般消費者向けアプリのように機能を大きな余白や段階的開示の奥へ隠さない。画面を軽く見せることより、どこに何の情報があるかを一目で把握できることを優先する。

- asset、preview、property、effect stack、driver、timeline、transportの領域を常設の見出し・罫線・位置で識別できるようにする
- timelineは波形、階層、項目種別、keyframe、bake/cache状態、beat gridを同じoverviewで確認できる密度を許容する
- 密度の審判はAbleton比較だけに依存しない。Abletonを唯一の既知例とする単一参照バイアスを避けるため、G0-6の同一fixture比較にはAbleton以外の制作ツール(Resolve / Blender等)を1つ以上並べる
- 頻用値と状態は折り畳みの奥へ隠さない。折り畳みは詳細値へ使い、機能の存在と現在状態は閉じた状態でも残す
- 右下またはstatus領域に、hover/focus中の操作について「何をするか」「shortcut」「現在の制約」を短く表示できる口を持つ。Blenderはこの文脈ヘルプだけの先例であり、panel数、情報量、全体レイアウトは模倣しない
- panelを増やす時は、既存の常設領域へ収まるかを先に検討する。一覧性を壊すmodal、別window、深いnavigationを既定導線にしない
- 説明を増やしても、カードの羅列にはしない。領域見出し、1行summary、必要時のcontext説明で情報階層を作る

「すっきりしている」は合格条件にしない。固定fixtureで、初見の人がasset、effect、driver、keyframe、bake状態、出力操作の所在を指せることを審判する。

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
- 余白は装飾や高級感の手段にしない。領域の分離は罫線と明度差で行い、空白による分離を既定にしない。「整って見える」は余白の量ではなく間隔の一貫性で作る
- spacing、radius、行高もcolor同様にtoken scaleから取り、raw値の場当たり指定を禁止する。「見にくい」への処方は余白追加ではなく、間隔をscaleへ揃えること
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

track/clipごとの任意色をDocumentへ保存するかは未決である。M3の見た目を理由にschema fieldを追加しない。最初の実装は安定Object IDからtheme paletteのslotを決定的に導出し、再描画や起動順で色を変えない。同じObjectの離れたbarは同色にし、同じカテゴリの別Objectは別色になり得る。これにより、mp4やshapeだけが大量に並ぶworkflowでもカテゴリ色一色へ潰さない。任意色の永続化が必要ならGR-PVを通す。

### 場所を覚えるwayfinding color

制作中に文字を読み直さず固定領域へ戻れるよう、Project、Files/Inbox、Plugins、Stage、Inspector、Timelineへ小面積の位置色を割り当ててよい。Abletonの色による再発見から借りるのは、色を作品装飾にせず反復認知へ使う点である。

- 同じ領域では見出し先頭の同形marker、選択tab下辺、領域内の主要入口へ同じ色を反復する
- 領域のsurface全体は塗らず、色を外しても位置、見出し、icon、境界で領域を識別できること
- wayfinding colorは選択、active、warning、error、項目状態を表さない。状態色と同じ視覚channelで競合させない
- Timeline itemはObject単位の単色面、選択はoutline、keyframeはdiamond、warningは固有icon/線種へ分離する。種類は色へ持たせず、icon・名称・bar形状で読む
- 色割当はthemeのsemantic tokenで決定し、Document、Undo、project固有設定へ保存しない。利用者が任意に領域色を変更する機能は別途意味を決めるまで作らない

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

- theme tokenはDesign Tokens Community Group format v2025.10準拠のJSONを正本とし、Rust / egui adapter側は決定的に生成する。v2025.10はDTCG初のstable reportだが、W3C Standards Trackの標準ではない
- 「raw color literalをtheme file外で拒否する」審判は、この単一正本があって初めて機械検査になる。tokenをコード内定数へ複製しない
- generatorのversion、入力hash、生成先を固定し、生成物の手編集と未生成差分をCIで拒否する。DTCGが定めるのは交換formatであり、Motolii内のrole命名や階層は本書が定める

### テーマ選択と拡張

- 組み込みテーマとして `Motolii Dark` と `Motolii Light` を同格で提供し、初回起動の既定値は `Motolii Dark` とする。土台dark neutralの面規約と一致させ、映像制作ツールの先例(AE / Premiere / Resolve / Final Cut / Blenderはいずれもdark既定。自発光コンテンツの色・露出判断は暗い順応状態で行う)に従う。Ableton Liveのlight既定はaudioツール(色判断が存在しない)の運用であり、既定値の根拠には借用しない
- 選択は設定画面の「外観 > テーマ」で行い、ユーザー設定として保存する。projectやDocumentへ焼き込まず、次回起動時に復元する
- custom themeも組み込みテーマと同じsemantic token schemaで読み込む。Ableton LiveのTheme運用を先例に、componentや機能ごとの例外分岐を作らない
- themeは色値の束であり、role名、component state、contrast条件、意味色の用途は変更できない。未知token、欠落token、型違い、contrast違反は読み込み時に診断し、安全な組み込みテーマへfallbackする
- UI実装は `light` / `dark` の名称や具体色を参照せず、解決済みsemantic tokenだけを読む。テーマ追加のたびにRust / egui componentコードを変更する設計を不合格とする

### dark面の明度階層

- 土台は原則として純黒(`#000000`)を使わず、dark grayから始める。Material 2の`#121212`は歴史的先例であり、Motoliiの固定値にはしない
- 階層は`canvas / panel / raised / overlay`等の意味roleとして必要最小数を定義する。一定率の機械的加算やMaterialのelevation overlayを移植せず、隣接境界・文字・iconのcontrastを満たす値をG0-6で個別に固定する

### 数値とtimecodeの文字

- timecode、フレーム番号、パラメータ値、ドラッグ中に変化する数値は、tabular(等幅)数字で表示し、値の変化で幅が揺れないことをG0-6の検査に含める
- 等幅が必要な列(timecode等)はmonospaced書体またはtabular lining機能を使う(SF Mono / SF Proのtabular liningが先例)
- egui既定fontだけではCJKを表示できないことを実機確認済み。数値readout用の等幅fontとCJK fallbackは、再配布可能なlicenseの同梱fontまたはOS別system font resolverで`FontDefinitions`へ明示登録する。SF Monoは同梱不可

### 多言語と文字幅

- v1の対応言語はLTRに絞る。出荷基準は日本語・英語、多くの企業/スタートアップが採用する言語(簡体中文・韓国語・独・仏・西等)は翻訳ファイル(.po)の追加だけで増やせることを機構要件とする。UIコード差分が要る設計は不合格
- RTL(アラビア語・ヘブライ語)と複雑scriptはv1スコープ外と明記する。将来対応の妨げになる決め打ち(左右の意味を持つhard-coded座標での文字配置)は避けるが、mirror layoutは作らない
- 密度と列幅は英語基準で設計し、言語で伸縮させない(開発者コミュニティはReddit等の英語圏を想定。英語は日本語より長い側なので下限基準としても安全)。日本語を含む各言語は同一fixtureで表示検査する。UI labelは原則1行固定で折り返さず、幅を超えた部分をelideする。全文や詳しい説明は下部Info / tooltip / focusへ委ね、文字列長で行高やpanel配置を変えない。G0-6の疑似長文locale(pseudo-locale)fixtureで機械検査する
- CJKは偶然のOS fallbackへ依存せず、日本語・英語・簡体中文・韓国語fixtureでglyph欠落、baseline、caret、IME preeditを検査する。具体font、subset、license、binary sizeはG0-6で固定する

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

初期の構成比較には[高密度メインUIモック](mocks/README.md)(および過去モックの台帳)を使う。モック内の色値はtoken候補ではなく、構成と意味roleの比較専用とする。

1. empty project + asset browser
2. video/audio/shape/text/groupを含むtimeline
3. 選択項目のparameter panel + keyframe/easing popup + warning/disabled状態
4. Stage + Output Frame + frame内外object + Camera tool / Hand(Stage View) tool。枠外を不透明塗潰しせず半透明scrimで見せ、Output Frame境界、出力外、選択を文字だけ・色だけにせず識別する
5. 非隣接layer 3つが同じEffect Definitionを異なるstack位置で使うtimeline。固定connection gutterの常時線、from/outとuse/in、折畳みstub+件数、通常drag/Relative drag HUDを同時に検証する

自動審判:

- screenshot goldenをlightness差分と通常差分で比較する
- DTCG token JSONをschema検証し、Rust/egui adapter生成物の手編集・生成漏れ・theme外raw color literalを拒否する
- 組み込みlight/darkとcustom theme fixtureを同じschema・component state matrixへ通し、テーマ追加時にUIコード差分が発生しないことを検査する
- clean profileの初回起動はdark、設定でlightを選んだprofileの再起動はlight、theme fileが壊れたprofileは診断付きでdarkへfallbackすることを統合テストする
- 通常文字4.5:1、大文字/太字3:1を下回るtoken pairを拒否する
- 意味を持つUI境界/icon/indicatorの隣接contrast 3:1と、focusの面積・状態間contrast・隣接contrastを検査する
- gradientは許可listにあるデータ表現だけに限定する
- component state matrixで選択/hover/focus/disabled/errorの欠落を拒否する
- timecode/数値表示の幅不変fixture、motion token外のduration/easing literal、reduce-motion時の状態欠落を検査する
- 通常/grayscale/protan/deutan/tritanのreference imageを同じfixtureから生成する

人間審判:

- labelを読まず、5秒以内に選択項目・項目種別・mute/disabled・keyframe・warningを指せるか
- 初見でasset、effect stack、driver、timeline、transport、context説明の所在を指せるか
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
- [Apple — Sufficient Contrast evaluation criteria](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/sufficient-contrast-evaluation-criteria)
- [Apple — Differentiate Without Color Alone evaluation criteria](https://developer.apple.com/help/app-store-connect/manage-app-accessibility/differentiate-without-color-alone-evaluation-criteria)
- [Apple Human Interface Guidelines — Icons](https://developer.apple.com/design/human-interface-guidelines/icons)
- [Apple Human Interface Guidelines — Typography](https://developer.apple.com/design/human-interface-guidelines/typography)(tabular lining数字、SF Mono)
- [WCAG 2.2 — Understanding SC 1.4.11 Non-text Contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html)
- [WCAG 2.2 — Understanding SC 2.4.13 Focus Appearance](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html)
- [Design Tokens Format Module 2025.10(DTCG)](https://www.designtokens.org/tr/2025.10/format/)
- [DTCG FAQ](https://www.designtokens.org/faq/)(stable範囲とW3C Standards Track外の明記)

設計先例(値をそのまま規範化しない):

- [Adobe Spectrum — Color system](https://spectrum.adobe.com/page/color-system/)
- [Adobe Spectrum — Using color](https://spectrum.adobe.com/page/using-color/)
- [Adobe Spectrum — Color fundamentals](https://spectrum.adobe.com/page/color-fundamentals/)
- [Material Design — Dark theme](https://design.google/library/material-design-dark-theme)(純黒を避けたdark grayと低contrast階層)
- [Material Design 3 — Easing and duration tokens](https://m3.material.io/styles/motion/easing-and-duration/tokens-specs)
- [Okabe-Ito palette(Color Universal Design)](https://siegal.bio.nyu.edu/color-palette/)
- [Lucide — Icon Design Guide](https://lucide.dev/contribute/icon-design-guide)

「実装時に発明しない領域」の各基準は2026-07-14のweb調査に基づく。docs/reviews/README.mdの規律に従い、G0-6で具体値を固定する前に反対側レビューの対象とする。規格、製品基準、設計先例を混同して値を追加しない。
