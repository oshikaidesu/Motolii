# Browser panel — Icebook design drafts

現行の `next/ui/motolii-browser-pane` を土台にした比較用の草案。Icebook の1 storyで、空・選択中・大量・絞り込み後の状態を差し替えて見比べる。現在の共通骨格は `Media / Effects / Create / Panels` の4タブ、左 rail、検索・filter shelf、Results、`scrollable` なカード一覧、Grid/List 切替。ここでの30案は実装数ではなく、hero作成の視線と入口を比較する候補である。

## B01 — Hero shelf

- `problem solved`: 素材を探すたびに一覧とStageを往復し、heroに使える一枚を決めるまで止まる。
- `hero/creation role`: 選択した素材を「今回のheroの主役候補」として大きく見せ、置く前の動機を作る。
- `layout/visual hierarchy`: 上に4タブと検索、左に既存rail、右を `column!` の大きな選択カード＋下段 `scrollable` 一覧。主役カードは `container` を一段明るくし、名前・尺・statusを縦積みする。
- `interaction/entry`: Mediaカードの通常clickで主役カードを更新、double-clickは既存 `PreviewMedia`、下段カードは `button`。Clearは既存 `ClearFilters`。
- `density/scale`: balanced。主役1枚＋一覧6〜12枚を基準にし、Grid/Listの切替で量へ対応する。
- `reuse-vs-scratch note`: `pane_view`、`card_body`、`AssetStatus`、`PreviewMedia`、tokensを再利用。選択主役の表示だけをpane-localに薄く足し、素材再生はBrowserへ実装しない。

## B02 — Contact sheet rhythm

- `problem solved`: heroのリズムに合う素材の組み合わせを、1枚ずつ開かず比較したい。
- `hero/creation role`: 画・音・短いcutawayを同じ視野に並べ、モチベーションの源になる組み合わせを先に発見する。
- `layout/visual hierarchy`: Mediaのrailは細く保ち、右は `scrollable(column!(row![card...]))` の接触シート。動画・画像・音声をglyph、色地、尺の3要素で揃え、選択カードだけ下線を太くする。
- `interaction/entry`: Cmd/Ctrlで `SelectCardWithModifiers`、Shiftで範囲選択。選択数をResults横の `text` で表示し、double-clickは個別 `PreviewMedia`。
- `density/scale`: dense。Grid 4列相当の小カードを12〜32枚見せる story と、List 1列の比較 storyを用意する。
- `reuse-vs-scratch note`: 既存のmodifier選択、`ViewMode`、`card_body`、`scrollable`をそのまま使う。選択数のsummaryだけ自前、contact sheet専用のデータ所有者は作らない。

## B03 — Drop runway

- `problem solved`: 新しい素材をどこへ落とせば制作が始まるのか、空のBrowserで分からない。
- `hero/creation role`: ファイルを置いた瞬間にhero候補へ変わるという開始感を作り、最初の一歩を強くする。
- `layout/visual hierarchy`: 台帳が空の時だけ右のcatalogを大きな `container` にし、中央に `text("Drop files here")`、下に薄い `button("Choose media")` の配置候補を示す。取り込み後は同じ面を最近追加カードの `row!` に切り替える。
- `interaction/entry`: 既存 `DropHoverChanged` で面の背景をhover色へ変更。admit後は `RecentlyAdmitted` の光を使い、選択・double-clickは通常Media経路へ戻す。
- `density/scale`: sparseからbalanced。空状態は1つの入口、取り込み後は直近4枚＋残りを `scrollable` にする。
- `reuse-vs-scratch note`: 既存の空状態、drop-hover、recent highlight、`catalog_container`を再利用。ファイル選択APIやadmit処理はShellへ委託し、Browser側に仮のimport stateを作らない。

## B04 — Missing triage

- `problem solved`: 素材の欠落を一覧の中で見落とし、heroを再現できないまま編集を続けてしまう。
- `hero/creation role`: heroの成立条件を壊している素材を先に発見し、表現以前の再現性の不安を取り除く。
- `layout/visual hierarchy`: Media railに `Missing` の候補行を追加し、右は欠落カードだけの `scrollable`。カード上部にstatus badge、中央にglyph、下にファイル名、右端に小さな `button("Relink")` 候補を置く。
- `interaction/entry`: rail選択は既存scope文法へ合わせ、欠落カードclickで選択、Relinkは未接続なら表示しない。接続後のみ `button` のon_pressをShellの復旧入口へ渡す。
- `density/scale`: balanced。欠落0件、1件、10件の3 storyで警告の強さを比較する。
- `reuse-vs-scratch note`: `AssetStatus::Missing`、既存badge、検索、カード選択を再利用。Missing scopeとRelinkの技術経路が未決なら自前UIを先に作らず、store/OSの復旧入口へ預ける。

## B05 — Source monitor cards

- `problem solved`: Mediaカードのdouble-clickが何を起こすか見えず、素材を置く前の確認が弱い。
- `hero/creation role`: heroに使う尺・画・音の適性を、編集対象へ置く前に確かめる。
- `layout/visual hierarchy`: 右上に選択中カードの横長 `container`、左下に既存カード `scrollable`。主役面はglyph、名前、duration、status、Previewの小さな `button` を順に置き、カードgridを補助にする。
- `interaction/entry`: Mediaカードのdouble-clickは既存 `PreviewMedia`、Previewボタンも同じtyped handoffへ畳む。カードの右clickは既存 `OpenContextMenu` とRemoveだけ。
- `density/scale`: sparse。主役1枚＋一覧8枚程度で、動画サムネがまだ代表フレームを持たない状態でも成立させる。
- `reuse-vs-scratch note`: `PreviewMedia`、`media_preview`、context menu、`card_body`を再利用。再生・波形・代表フレームはSource Monitor側とFFmpegへ委託し、Browserにplayerをスクラッチしない。

## B06 — Place-ready stack

- `problem solved`: 選択素材を置く先が決まっているのに、ReplaceやPlaceの入口がカードから見えない。
- `hero/creation role`: heroの主役候補を現在の選択レイヤーへ即座に差し替え、試行錯誤の摩擦を減らす。
- `layout/visual hierarchy`: 右上に `text("Selected layer")` と対象名の細いtarget ribbon、下にMediaカードを `scrollable`。置換可能なカードは `container` 内で `button("Replace")` を名前の右に置く。
- `interaction/entry`: `single_selected_layer` が `Some` の時だけ `can_replace_source` を通して `ReplaceSelectedLayerSource` を発行。複数選択時はtarget ribbonを説明文へ落とし、ボタンを出さない。
- `density/scale`: balanced。カード8〜20枚、target ribbonは常時1行で視線を奪わない。
- `reuse-vs-scratch note`: 既存 `single_selected_layer`、`can_replace_source`、`asset_to_layer_source`、`ReplaceSelectedLayerSource`を再利用。Document書き込みはShellの`Intent::SetSource`へ委託し、Browserはtarget表示だけを持つ。

## B07 — Media evidence list

- `problem solved`: 小さなカードでは名前・種別・尺・欠落状態が同時に読めず、hero候補を選ぶ根拠が薄い。
- `hero/creation role`: 「なぜこの素材を選ぶか」を可視化し、勘だけでなく尺と状態に基づく選択を支える。
- `layout/visual hierarchy`: Mediaのfilter shelf直下を `scrollable` なList専用にし、各行を `row![thumb, column![name, caption], duration, status]`。選択行だけ左端にactive stripを出す。
- `interaction/entry`: 既存 `ViewMode::List` の水平カードを主役にし、名前検索は`text_input`、sortは既存3チップ。double-clickとCmd/Shift選択は変えない。
- `density/scale`: dense。縦20〜40行を基準にし、Gridへ戻った時の視覚差もIcebookで比較する。
- `reuse-vs-scratch note`: `card_body`のList、`SortKey`、status badge、ellipsis、`scrollable`を再利用。列幅はtokensへ委託し、表専用の新しいlayout engineは作らない。

## B08 — Hero palette

- `problem solved`: Mediaの種類は絞れても、heroの色・音・動きの組み合わせを考える視点が一覧にない。
- `hero/creation role`: 素材を単なるファイルではなく、heroの色調・テンポ・空気として選べるようにする。
- `layout/visual hierarchy`: 左railはAll media / Video / Images / Audioを維持。右上に選択中カードのglyph色とdurationをまとめた `container`、下にカードgrid、filter shelfには既存scopeとsortだけを置く。
- `interaction/entry`: クリック選択、Cmd/Ctrl複数選択、double-click Preview。色やテンポの推測を新しいタグとして表示せず、現行の種別と実属性だけを入口にする。
- `density/scale`: balanced。4〜16枚で、単色・混在・音声多めのfixturesを用意する。
- `reuse-vs-scratch note`: 既存のCategory、glyph、duration、selection、PreviewMediaを再利用。AI分類や新しいmood属性は上流/storeへ委託し、見た目だけのタグをスクラッチしない。

## B09 — Effect audition grid

- `problem solved`: Effectsカードを押す前に、heroの画面へ何を足す意図なのか判断しづらい。
- `hero/creation role`: Glow、Mask、shape opを「表現を一段変える候補」として比較し、heroの勢いを止めずに試せる。
- `layout/visual hierarchy`: Effectsタブの左railはColor / Masks / Shape ops、右は `scrollable` のカードgrid。各カードはglyph、名称、`effect · Color` 等のcaption、下端に選択対象のtarget textを置く。
- `interaction/entry`: 単一layer選択時だけカードdouble-clickを既存 `ApplyEffectFromCard` / `AddMaskFromCard` / `ApplyOpFromCard`へ渡す。対象なしではカード選択のみとし、適用ボタンを出さない。
- `density/scale`: balancedからdense。Glow/Mask＋7演算子を6〜12枚のstoryで見せる。
- `reuse-vs-scratch note`: `PreviewCard`、`SelectionAction`、既存mouse_area、`CardHovered`を再利用。効果計算と既定値はcompositor/storeへ委託し、Browserにeffect engineを作らない。

## B10 — Selection-target effect rail

- `problem solved`: Effectsを選んでも、どのlayerへ適用されるかが分からず誤操作が怖い。
- `hero/creation role`: heroの主役を壊さず、現在選んでいるlayerへ安全に表現を足す。
- `layout/visual hierarchy`: 上部に細いtarget ribbon、左railは既存カテゴリ、右はカードを2列。target ribbonの状態を `text` と小さな `container` の面差だけで示し、カードを主役にする。
- `interaction/entry`: `Some(single_selected_layer)`なら「Selected layer」、Noneなら「Select one layer」を表示。カードdouble-clickは既存Message、target不在時の拒否判断はShellへ渡す。
- `density/scale`: sparseからbalanced。カード9枚を一度に見せ、target ribbonは2行以内に抑える。
- `reuse-vs-scratch note`: `single_selected_layer`の値渡し、既存card action、tokens、Iced `container`を再利用。選択レイヤーの正本や拒否規則はSession/Shellへ預ける。

## B11 — Before/after effect card

- `problem solved`: effect名だけでは、heroの見た目がどう変わるか想像しにくい。
- `hero/creation role`: 「試す価値がある表現」を視覚で選び、効果の導入をモチベーションに変える。
- `layout/visual hierarchy`: 各Effectsカードのthumbを `row![before_container, text("→"), after_container]` にし、名称・captionを下へ置く。grid/list切替は既存`ViewMode`のまま。
- `interaction/entry`: single clickでカード選択、double-clickで既存適用Message。実プレビューが無いfixtureではglyphとtoken色だけを使い、偽のレンダー結果を表示しない。
- `density/scale`: balanced。カード6〜9枚、各thumbは小さくてもbefore/afterの左右関係が読める幅を確保する。
- `reuse-vs-scratch note`: `PreviewCard`、`card_body`、`SelectionAction`、既存tokensを再利用。before/afterの実画像生成はEngine/Compositorへ委託し、Browserでは表現構造だけをスクラッチする。

## B12 — Effect chain shelf

- `problem solved`: heroの主役にすでに何を足したかがBrowserから見えず、同じeffectを重ねる不安がある。
- `hero/creation role`: heroの表現を「足していく」感覚と、現在のeffect構成を同時に保つ。
- `layout/visual hierarchy`: Effectsタブ上部に選択layerのchainを横 `scrollable`、下に既存filter shelfとカードgrid。chainは小さな `container` チップをrowで並べ、カタログを主面に残す。
- `interaction/entry`: chainチップは表示専用または既存Inspectorへのhandoff。effectカードdouble-clickは既存適用Message、未選択時はchainを出さない。
- `density/scale`: balanced。chain 0〜5件＋カード9枚を一画面で比較し、10件以上は水平scrollする。
- `reuse-vs-scratch note`: selected layerのread projectionはStoreView/Inspectorへ委託し、既存`PreviewCard`と`SelectionAction`を再利用。Browser独自のeffect listや削除書き込みは作らない。

## B13 — Shape-operator deck

- `problem solved`: 7つのshape opが同じ見た目のカードに埋もれ、形を変える入口が見つからない。
- `hero/creation role`: 形の反復・歪み・切り出しを素早く試し、heroのシルエットを作る。
- `layout/visual hierarchy`: Effects > Shape opsを選んだ状態で、上に7枚の小さなoperator deckを `row!`、下に選択中opの名前と短い説明を `column!`。card gridより「連続して試す」視線にする。
- `interaction/entry`: 既存 `ShapeOpKind` の7カードをsingle clickで選択、double-clickで `ApplyOpFromCard`。既定パラメータの調整はInspectorへhandoffする。
- `density/scale`: dense。7枚すべてを横1列または2段の`row!`で見せ、追加の架空opは置かない。
- `reuse-vs-scratch note`: `ShapeOpKind`、`PreviewTag::ShapeOps`、既存action handoffを再利用。parameter editor・shape evaluationはvector/storeへ委託し、Browserは札と入口だけ持つ。

## B14 — Hero mood effects

- `problem solved`: Effectsの分類が技術名中心で、heroにどんな感情の変化を足すかが見えにくい。
- `hero/creation role`: Color / Masks / Shape opsを「明るさ・焦点・動き」の制作意図として読み替え、表現の方向を選びやすくする。
- `layout/visual hierarchy`: 既存3カテゴリrailを維持し、右上に現在カテゴリの短い`text`、下にカードgrid。カードのcaptionは現行の`effect · Color`等を保ち、hero用の説明は選択時の小さな`container`に限定する。
- `interaction/entry`: railまたはfilter chipでカテゴリを選択、カードsingle/double clickは既存経路。説明文は新しい操作入口を増やさない。
- `density/scale`: balanced。カテゴリごとに2〜9枚、カード間隔を広めにして意味の切替を見せる。
- `reuse-vs-scratch note`: 既存PreviewTag、PreviewScope、filter shelf、SelectionActionを再利用。mood taxonomyは外部先例/製品定義へ委託し、未接続のmoodタグを永続化しない。

## B15 — Creation launchpad

- `problem solved`: CreateタブがMediaと同じカード一覧に見え、「ここから作る」という意図が弱い。
- `hero/creation role`: Rectangle / Ellipse / Star / Solid / Null / Textを、heroの最初の形・面・文字へ直結する発射台にする。
- `layout/visual hierarchy`: Createタブだけ左railを短くし、右を6枚の大きめ `mouse_area` カード `scrollable`。thumb、名前、`shape · Built-in`等のcaption、hover面を明確に分ける。
- `interaction/entry`: single clickは既存選択、double-clickは`CreateFromCard { kind }`。hoverは`CardHovered`で表示し、drag-to-Stageはこの案では入口にしない。
- `density/scale`: sparse。6枚を2列または3列で見せ、空間を「作る」方向へ使う。
- `reuse-vs-scratch note`: `CreateKind`、`CREATE_PREVIEW`、mouse_area、`create_card_face`、既存tokensを再利用。実レイヤー生成と選択はShell/Documentへ委託し、カード側に作成状態を持たせない。

## B16 — Text-first lyric launch

- `problem solved`: lyric/MVの開始時に、Text入口が形カードの中へ埋もれている。
- `hero/creation role`: 文字をheroの主役として最初に置き、音楽・言葉・画の動機をすぐ画面へ出す。
- `layout/visual hierarchy`: Createタブ上部にTextのfeatured `container`、下にRectangle/Ellipse/Star/Solid/Nullの小カード `scrollable`。featured面はglyph `T`、名前、短いcaptionを大きくする。
- `interaction/entry`: Text面も既存`CreateFromCard { kind: Text }`、通常カードは同じdouble-click。featured面をbutton化して別の作成文法を増やさない。
- `density/scale`: sparse。主役Text 1枚＋補助5枚、縦方向に余白を残す。
- `reuse-vs-scratch note`: `CreateKind::Text`と既存カード経路を再利用。文字内容・書体・範囲スタイルはInspector/TextDocumentへ委託し、Browserでtext editorをスクラッチしない。

## B17 — Shape family rail

- `problem solved`: CreateのShapesとBuilt-inが混ざり、形を探すときに補助layerまで視界へ入る。
- `hero/creation role`: heroの輪郭を作る形だけを連続して比較し、最初のシルエット決定を速くする。
- `layout/visual hierarchy`: railのShapesを選ぶとRectangle/Ellipse/Starだけを右へ表示。上は検索・filter shelf、中央は3枚を大きな`row!`、下は説明用の薄い`container`。
- `interaction/entry`: `PreviewScope::Tag(Shapes)`と既存filter chipを使い、カードdouble-clickでCreateFromCard。Built-inへ戻るとSolid/Null/Textを同じcatalog文法で表示する。
- `density/scale`: sparse。Shapes 3枚を大きく、Built-in 6枚ではbalancedへ戻す。
- `reuse-vs-scratch note`: `PreviewTag::Shapes`、`PreviewScope`、`preview_visible`、既存カードを再利用。カテゴリ分割の新しい状態や別catalogは作らず、既存の宣言順へ預ける。

## B18 — Staging kit

- `problem solved`: SolidとNullが単なる追加カードに見え、heroの背景・制御面としての役割が伝わらない。
- `hero/creation role`: Solidを背景/色面、Nullを制御点、Textをメッセージとしてheroの骨格を組む発想を支える。
- `layout/visual hierarchy`: Create > Built-inでSolid/Null/Textを上段の3枚 `row!`、Shapesを下段 `scrollable`。各上段カードにglyph、caption、短いrole textを積む。
- `interaction/entry`: 3枚とも既存double-click作成。role textは説明だけで、選択中layerやcompositionを暗黙に変更しない。
- `density/scale`: balanced。上段3枚＋下段3〜6枚、横幅の狭いBrowserでは下段のみscrollする。
- `reuse-vs-scratch note`: `CreateKind`、既存Create cards、`row`/`column`/`scrollable`を再利用。背景・親子・制御の意味はDocument/Stageへ委託し、Browserにcompositionモデルを作らない。

## B19 — Motif assembly strip

- `problem solved`: heroの形を一つ作った後、次に何を足すかが毎回ゼロからになる。
- `hero/creation role`: Text・Shape・Solid・Nullを一つのmotifとして眺め、制作の勢いを保ったまま次の一手を選ぶ。
- `layout/visual hierarchy`: 上部に「Create」見出しと現在選択カードのmini `container`、下に6枚のカードgrid。mini面は選択表示だけで、作成履歴の永続リストにはしない。
- `interaction/entry`: single clickでfocus、double-clickで既存CreateFromCard。作成後の選択反映はShellから次のrenderで返し、Browserが履歴を推測しない。
- `density/scale`: balanced。mini 1枚＋カード6枚、複数生成後もminiは1枚に固定する。
- `reuse-vs-scratch note`: `CardKey::Preview`、`CreateKind`、既存選択とShellの生成経路を再利用。motif履歴を新しい状態として持たず、選択中layerのprojectionへ委託する。

## B20 — Featured create card

- `problem solved`: 6枚のCreateカードが同じ重さで、heroの最初の一手を決める視線が散る。
- `hero/creation role`: 選択中のstoryで最も重要な創作入口を一枚だけ大きく提示する。
- `layout/visual hierarchy`: 右上にfeatured `container` 1枚、下に残り5枚の`row!`/`scrollable`。featuredは現在のCreateKindをfixtureで差し替え、他カードは既存サイズで揃える。
- `interaction/entry`: featuredも残りも同一mouse_area・single選択/double作成。featured切替はカードsingle clickだけで、別の「おすすめ」機能は作らない。
- `density/scale`: sparseからbalanced。featured 1＋補助5、狭幅ではfeaturedを上、補助を縦scrollする。
- `reuse-vs-scratch note`: `create_card_face`、`card_body`、`CardHovered`、`CreateFromCard`を再利用。featured選定はIcebook fixtureまたは既存選択から与え、推薦エンジンをスクラッチしない。

## B21 — Create-to-effect handoff

- `problem solved`: 形を作るタブと、形へeffectを足すタブの間で次の操作が見失われる。
- `hero/creation role`: 「形を作る→Glow/Mask/shape opを足す」というhero制作の因果を一つの視線で示す。
- `layout/visual hierarchy`: Createカードの下端に小さな`container` ribbon「Next: Effects」、右端に`button("Effects")`候補を置く。Effectsタブでは同じ位置に「Selected layer」ribbonを出す。
- `interaction/entry`: Createカードのdouble-clickは既存生成、Effectsボタンは`SelectTab(Effects)`へ畳む候補。自動適用はせず、次のeffectカード選択を利用者に残す。
- `density/scale`: balanced。Create 6枚またはEffects 9枚を一度に見せ、ribbonは1行だけ。
- `reuse-vs-scratch note`: `SelectTab`、`CreateFromCard`、既存tab帯、target値渡しを再利用。クロスタブの自動workflowや新しいDocument stateは作らず、Shellのtab handoffへ委託する。

## B22 — Utility panel shelf

- `problem solved`: PanelsのAsset tagging / Notes / Export notesが、作るカードと同じに見えて役割が伝わらない。
- `hero/creation role`: hero制作中に「整理・記録・書き出し確認」の補助を必要な時だけ呼び出せる。
- `layout/visual hierarchy`: Panelsタブは左railを残しつつ、右を3枚の横長 `button` shelfにする。各panelはglyph、panel名、短い用途を`row!`で横並びにし、Media/Effects/Createのカードより落ち着いた面にする。
- `interaction/entry`: card clickで選択、double-clickまたはbutton pressで将来のpanel handoff候補。未接続のPanelは選択だけにし、操作可能な顔を出さない。
- `density/scale`: sparse。3枚を同時表示し、余白で「補助面」を表現する。
- `reuse-vs-scratch note`: `PANELS_PREVIEW`、`PreviewTag`、`PreviewScope`、既存card frameを再利用。Notes/Tagging/Exportの実体は各ownerへ委託し、Browserに内容の第二状態を作らない。

## B23 — Shot-prompt notes

- `problem solved`: heroを作る理由や一場面の狙いが、編集途中で消える。
- `hero/creation role`: Notesを「便利なメモ」ではなく、heroの一場面を始める動機の短いpromptとして扱う。
- `layout/visual hierarchy`: Panels > Notesで、上に選択中のNotesカードを大きな`container`、下にAsset tagging/Export notesを小さなbutton shelf。本文編集欄はこのBrowser storyには置かない。
- `interaction/entry`: Notesカードdouble-clickはNotes ownerへのhandoff候補、Media選択は別tabのまま保持。Browser内で入力を確定する操作は増やさない。
- `density/scale`: sparse。主役1枚＋補助2枚、文章は2行ellipsisで止める。
- `reuse-vs-scratch note`: 既存Panelsカード、tab state、ellipsis、tokensを再利用。ノート本文と保存はDocument/Notes ownerへ委託し、Browserにtext_inputを新設しない。

## B24 — Export readiness panel

- `problem solved`: heroが完成したつもりでも、書き出し時に必要な確認をどこで見るか分からない。
- `hero/creation role`: 表現の勢いを保ったまま、完成したheroを外へ出す最後の不安を減らす。
- `layout/visual hierarchy`: Panels > ExportでExport notesをfeatured `container`、下にNotesとAsset tagging。featured内はstatusの読み取り専用`row!`候補（format / range / audio）とし、編集欄は置かない。
- `interaction/entry`: Export notesカードからExport paneへhandoff。Browserは `SelectTab` とカード選択だけを行い、実際のExport操作・進捗・再開はExport ownerへ渡す。
- `density/scale`: sparse。3枚＋確認行3つ、警告があるfixtureではstatus行だけ強調する。
- `reuse-vs-scratch note`: `PANELS_PREVIEW`、既存tab、`container`/`row`を再利用。format/range/audioの真実はExport pane/Documentへ委託し、Browserに複製しない。

## B25 — Asset tagging gate

- `problem solved`: Mediaが増えた後、hero候補の整理を始める入口が見えない。
- `hero/creation role`: 大量素材からheroの候補群を再発見できるようにし、探す時間を制作時間へ戻す。
- `layout/visual hierarchy`: Panels > Asset taggingをfeaturedにし、上部に「current media selection」のmini `container`、下部にMediaへ戻る`button`候補を置く。タグ一覧そのものは描かない。
- `interaction/entry`: Asset taggingカードのdouble-clickはTagging ownerへのhandoff候補、Media buttonは`SelectTab(Media)`。タグ未接続時はcard選択だけで止める。
- `density/scale`: sparse。Panelカード3枚と選択mini 1枚、素材量はMedia側のstoryで別に見せる。
- `reuse-vs-scratch note`: 既存`Asset tagging`カード、`SelectTab`、Media選択projectionを再利用。タグ属性・favorite・collectionはstore/Filesystemへ委託し、見た目だけのタグ編集をスクラッチしない。

## B26 — Panels split view

- `problem solved`: Panelsの補助機能を開くたびに、どのPanelを選んだかと戻り先が消える。
- `hero/creation role`: hero制作の流れを遮らず、補助Panelを一時的に見てすぐ主役の作業へ戻れる。
- `layout/visual hierarchy`: 左を既存Panels rail＋カード棚、右を選択中Panelのread-only `container` に分ける `row!`。選択がない時は「Choose a panel」の最小text、Panelsカードは常に左の主役。
- `interaction/entry`: card clickで右面を更新、double-clickはowner handoff候補、タブ切替で右面をクリア。戻る操作は既存tab buttonへ畳む。
- `density/scale`: balanced。左3枚＋右1面、右の文章は最大4行でscrollしない。
- `reuse-vs-scratch note`: `PreviewScope`、`CardKey::Preview`、既存tab遷移、Iced `row`/`container`を再利用。Panel内容の正本は各paneへ委託し、Browserに汎用detail routerを作らない。

## B27 — Role rail browser

- `problem solved`: 上部の4タブは役割を切り替えるが、狭いBrowserでは今どの制作段階かが弱い。
- `hero/creation role`: Media→Create→Effects→Panelsをhero制作の流れとして見せ、次の意図を選びやすくする。
- `layout/visual hierarchy`: `tab_band_view`を横帯から細い左のrole railへ置き換える候補。左からMedia/Effects/Create/Panelsの`button`、右に現行rail＋catalogを`column!`で置く。
- `interaction/entry`: role buttonは既存`SelectTab`、各tab内部のscope/filter/card messageは不変。active roleは既存active色とunderline相当の面差で示す。
- `density/scale`: balanced。横幅を節約し、カードgrid 2〜4列を保つ。狭幅・広幅の2 storyを比較する。
- `reuse-vs-scratch note`: `LibraryTab`、`SelectTab`、既存rail/catalog、tokensを再利用。タブの意味や状態所有は変えず、差分はレイアウトの薄い自前組み替えだけにする。

## B28 — Focus creation mode

- `problem solved`: Browserのrail・filter・4タブが、heroの一手を選ぶ時には視界を狭める。
- `hero/creation role`: 一時的にカードを大きくして、作る・足すの決断をモチベーションの高い状態で行う。
- `layout/visual hierarchy`: `container` の上に小さなFocus `button`、有効時はrailを`Length::Shrink`相当に畳み、右catalogをFill。タブ帯は残し、カードthumbと名前を主役にする。
- `interaction/entry`: Focus buttonはpane-local表示状態の候補、tab/scope/query/card actionは不変。Escapeまたは同じbuttonで通常layoutへ戻す。
- `density/scale`: sparse。Focusはカード2〜6枚、通常時は既存Grid/List。大量素材ではFocusを解除して検索へ戻る。
- `reuse-vs-scratch note`: 既存catalog、`ViewMode`、search/filter、tokensのFill/spacingを再利用。Focus flagだけを一時状態へ薄く足し、Document/Sessionや別windowを作らない。

## B29 — Selection handoff ribbon

- `problem solved`: Browserで選んだ素材・作成カード・effectカードが、どの外部対象へ渡るか見えない。
- `hero/creation role`: hero制作の「選ぶ→置く→足す」を一枚のhandoff ribbonで認識できるようにする。
- `layout/visual hierarchy`: 4タブ帯の直下に高さ1行の`container` ribbon、下は現行のrail＋catalog。ribbonは `text` で選択カード名、target、次の意味を示し、catalogの主役を奪わない。
- `interaction/entry`: Media選択はPreview/Replace、CreateはCreateFromCard、EffectsはApply/Add、Panelsはowner handoff候補を既存Messageへ接続。未選択・複数選択は説明だけにする。
- `density/scale`: balanced。ribbonは1〜2行、カード量は各tabの現行上限を維持する。
- `reuse-vs-scratch note`: `selected_cards`、`single_selected_layer`、`SelectionAction`、既存typed Messageを再利用。targetの正本と副作用はShell/Document/各ownerへ委託し、ribbonはprojectionに限定する。

## B30 — Hero recipe lane

- `problem solved`: 4タブがそれぞれ便利でも、heroを作る一連の因果が横断して見えない。
- `hero/creation role`: Mediaの主役候補、Createの形、Effectsの一手、Panelsの確認を一つの制作レシピとして思い出せる。
- `layout/visual hierarchy`: Browser上部に薄い `row!` の4セル（Media / Create / Effects / Panels）を置き、各セルは現在選択の小さなglyph＋nameだけを表示。下は選択中tabの現行 `rail + filter shelf + scrollable catalog` をそのまま主面にする。
- `interaction/entry`: 4セルのbuttonは既存`SelectTab`、カード操作は各tabの既存Message。セルは次のtabへ移るだけで、自動作成・自動適用・自動書出しは行わない。
- `density/scale`: balanced。recipe laneは4セル固定、下部カードはGrid/Listと検索で量を受ける。未選択セルは空glyphではなく短いrole labelだけにする。
- `reuse-vs-scratch note`: `LibraryTab`、tab band、選択projection、既存catalog、tokensを再利用。横断recipeの保存や履歴は作らず、各paneの意味とDocument/Sessionの正本へ委託する。
