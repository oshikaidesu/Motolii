# Motolii Inspector — Icebook panel drafts

このファイルは、Motolii の「hero を作るための Inspector」を Icebook で並べて比較するための草案集である。通常の属性台帳ではなく、**選択した物が次の一手で画面上の主役になる**ことを基準にする。

## Story fixture contract

- 各 story は `SelectionProjection` を入力にし、Document を直接変更しない。
- 既存の意味は `PROP / VALUE / KEY`、`KeyCellState` の3状態、`text_input` の Enter 確定、値セルの drag-to-scrub、provider の `ParameterDescriptor`、link の標準5 property に合わせる。
- Iced の composition は `container` / `column` / `row` / `scrollable` / `text_input` / `text_editor` / `pick_list` / `button` / `toggler` / `mouse_area` / `tooltip` / `rule` を基本語彙にする。
- `reuse-vs-scratch` は、既存 projection・Message・Intent・stock widget へ預ける範囲と、見た目の薄い adapter だけを明記する。新しい Document の真実や別の keyframe 機構は草案に含めない。

## I01 — Hero Transform Stack

- **ID:** I01
- **名前:** Hero Transform Stack
- **解決する問題:** Position、Scale、Rotation、Opacity が同じ重さで並び、最初の一手が見えない問題を解決する。
- **Hero / creation role:** 画面中央の主役をまず置き、拡大し、傾けるための「最初の10秒」を支える。Position と Scale を hero 操作、Opacity 以下を仕上げとして読む。
- **レイアウト / visual hierarchy:** `column![ident_band, container(column![hero_rows]), rule, scrollable(rest)]`。hero card は Position/Scale の2行を `row![label, value cells, key]` で大きくし、下に Rotation/Opacity/Anchor を標準行で置く。`PROP / VALUE / KEY` は hero card の上に小さく残す。
- **Interaction / entry:** value cell の click は `text_input` focus、Enter は `FieldSubmit`。cell の press→window `PointerMoved` は scrub、右端の菱形は `KeyPressed`。ident band の名前入力は常時 visible。
- **Density / scale:** 標準幅320px、hero row 36px、通常 row 28px、section gap 12px。狭幅では hero card の補助説明を消し、値と Key を残す。
- **Reuse-vs-scratch note:** `SelectionProjection.transform`、`transform_row`、`FieldDraft`、`KeyCellState`、既存 tokens を再利用する。新規なのは hero 2行を囲う `container` と背景階層だけで、編集経路は scratch しない。

## I02 — Axis Matrix

- **ID:** I02
- **名前:** Axis Matrix
- **解決する問題:** Position/Scale の X・Y・Z が縦に散り、軸の対応関係と同時編集の意図が読み取りにくい問題を解決する。
- **Hero / creation role:** 3D 的な奥行きや正確な拡大率を作るとき、三軸の差分を一目で比較できる制作台になる。
- **レイアウト / visual hierarchy:** `column![section_header("TRANSFORM"), row![blank, X, Y, Z, KEY], matrix_row(Position), matrix_row(Scale), scalar_rows]`。Position と Scale は各 `ComponentSlot` を4列の grid 風 `row` にし、Rotation/Opacity は通常の一列へ落とす。`container` で matrix 全体を薄く囲う。
- **Interaction / entry:** 各軸の cell は同じ `FieldInput/FieldSubmit` に入り、Position の Key は property 一本に対して一つだけ右端に置く。X/Y の片方を変更しても既存の Vec2 property を書き、Z は `POSITION_Z` の別入口として label を分ける。
- **Density / scale:** 幅340px以上、軸 cell 54px、label 62px、Key 24px。幅が足りない場合は X/Y/Z を縦列へ戻す responsive story を別状態として見せる。
- **Reuse-vs-scratch note:** `RowValue::Vector` と `ComponentSlot` の意味をそのまま使い、grid 用の表示 adapter だけを自前にする。軸を別 property に再設計したり、Z を Vec2 に混ぜたりしない。

## I03 — Stage-Linked Coordinates

- **ID:** I03
- **名前:** Stage-Linked Coordinates
- **解決する問題:** Inspector の数値編集が Stage 上の見た目から切り離され、「値を変えた結果」が予想できない問題を解決する。
- **Hero / creation role:** 画面上の主役を動かしながら、現在値・基準値・変更方向を読み、ポスター的な構図を詰める。
- **レイアウト / visual hierarchy:** 上部に `container(row!["LIVE", current frame, layer label])` のライブ帯、その下に Position/Scale の2×2 hero controls、下段に通常 section。値 cell の左側に小さな方向 glyph を置き、Stage の preview を Inspector 内に複製しない。
- **Interaction / entry:** 値 cell の drag-to-scrub は即時 preview、release で一つの undo。click は text entry。`tooltip` で drag 感度と Shift の fine mode を示し、Key click は現在 playhead の値を記録する。
- **Density / scale:** ライブ帯24px、hero cell32px、通常行26px。横幅300〜360pxで、長い layer 名は ellipsis、数値は右揃えにする。
- **Reuse-vs-scratch note:** `FieldDragState`、`dragged_value`、`finish_field_drag` と Stage の既存 preview を再利用する。ライブ帯は Session の playhead read-only 表示だけで、Inspector 用の二重 preview state は作らない。

## I04 — Creation Command Deck

- **ID:** I04
- **名前:** Creation Command Deck
- **解決する問題:** 数値を読むだけの表に見え、hero 作成の「置く・大きくする・見せる」という順序が伝わらない問題を解決する。
- **Hero / creation role:** 制作動詞を上から `PLACE / FRAME / REVEAL` として提示し、素材をモチベーション映像の一枚へ変える。
- **レイアウト / visual hierarchy:** `column![ident_band, place_card(Position/Anchor), frame_card(Scale/Rotation), reveal_card(Opacity/Blend/Hidden), advanced_sections]`。各 card は小見出し、1〜2行、右端 Key の構成。カード間は `rule` ではなく余白で区切る。
- **Interaction / entry:** card 見出しは collapse button、row は既存の value/key 操作、Hidden は `toggler`、Blend は既存の cycle button。入口の順序だけ変え、Message の意味は変えない。
- **Density / scale:** 360px基準、card padding8px、見出し20px、row28px。折りたたみ時は見出しと現在値の summary だけを残す。
- **Reuse-vs-scratch note:** 既存 section row と `CollapseState` の表現を再利用し、grouping と summary の表示だけを薄く足す。新しい「制作モード」状態や動詞別 store は作らない。

## I05 — Selection Delta Sheet

- **ID:** I05
- **名前:** Selection Delta Sheet
- **解決する問題:** 複数選択時に共通値・ばらつき・編集可能範囲が分からず、hero の統一感を作れない問題を解決する。
- **Hero / creation role:** 複数レイヤーを一つのモーション群として揃え、タイトル・装飾・背景を同じリズムへ寄せる。
- **レイアウト / visual hierarchy:** 上部に `N layers selected` と text 対象数の band。各 transform row は `common value`、不一致は `—` と薄い `mixed` badge、右端に bulk affordance。下に bulk-safe な attrs と text section を置く。
- **Interaction / entry:** 共通値だけを text_input/drag で編集し、不一致 cell は click で「全選択へ適用」の明示 entry を出す。Key は複数対象に同じ `KeyPressed` semantics を渡す。text は全選択が Text の時だけ editable とする。
- **Density / scale:** 300px、band32px、row28px。情報量が多いので section は最初から `scrollable`、mixed badge は8pxの文字で値を圧迫しない。
- **Reuse-vs-scratch note:** `selection_count`、`text_layer_count`、既存 bulk projection と一括 Intent を再利用する。混在値の描画 adapter は自前にするが、単一選択用の fake layer は作らない。

## I06 — Narrow Focus Inspector

- **ID:** I06
- **名前:** Narrow Focus Inspector
- **解決する問題:** 小さい dock 幅でラベル・値・Key が潰れ、どの行を触れるか分からなくなる問題を解決する。
- **Hero / creation role:** 画面を広く Stage に残したまま、今触る一つの数値を確実に決める。hero 作成の集中席である。
- **レイアウト / visual hierarchy:** 上部に layer 名と現在の section 名だけを固定し、`scrollable(column![one expanded section])`。row は `label` を上段、`value + key` を下段にする二段構成。隣接 section は compact disclosure にする。
- **Interaction / entry:** section header を button で切り替え、value cell は focus 時だけ border を強める。drag は value 全幅、Key は同じ row の右端に固定して誤タップを減らす。
- **Density / scale:** 220〜260px、label18px、editor32px、section header30px。数値の小数桁は既存 `field_decimals` に従い、フォントを縮めて詰めない。
- **Reuse-vs-scratch note:** 既存 row の field/key semantics、`value_input_style`、scrollable を再利用する。二段 row の geometry と focus decoration のみ自前で、別 input parser は作らない。

## I07 — Three-State Key Rail

- **ID:** I07
- **名前:** Three-State Key Rail
- **解決する問題:** Key の菱形が「キーなし」「現在時刻にキー」「別時刻に track あり」を伝えきれず、クリックの結果が怖い問題を解決する。
- **Hero / creation role:** 静止画を動きへ変える入口を Inspector の全 animatable row に揃え、hero の開始・中間・終端を作りやすくする。
- **レイアウト / visual hierarchy:** `column![property_header, rows]` の右端に24pxの Key rail。各 row は label/value/Key の同じ baseline、Key state は glyph + 小さな tint だけで表し、追加説明は `tooltip` に逃がす。
- **Interaction / entry:** `KeyCellState::Static` は click で playhead key を作成、`AtPlayhead` は click でそのキーを外し、`Between` は評価値で key を追加する。hover tooltip に3状態を表示し、double click は使わない。
- **Density / scale:** row28px、Key24px、rail の背景は section ごとに一枚。高密度でも click target は24pxを下回らない。
- **Reuse-vs-scratch note:** `key_cell_state`、`toggled_key_track`、`KeyRow`、既存 glyph を完全再利用する。新規は rail の背景/hover 表現だけで、keyframe model や toggle semantics は scratch しない。

## I08 — Animated-Only Property Shelf

- **ID:** I08
- **名前:** Animated-Only Property Shelf
- **解決する問題:** property が増えると静止値の行が keyframe の発見を邪魔し、いま動いている要素へ到達できない問題を解決する。
- **Hero / creation role:** すでに動き始めた hero の「どの値が次の演出を決めるか」だけを見せ、モーションの調整を速くする。
- **レイアウト / visual hierarchy:** section header に `Animated only` toggler と `N animated` count。on の時は track が存在する row、off の時は全 row。animated row の Key rail は常時可視、未アニメート row は低コントラストにする。
- **Interaction / entry:** toggler は Session の表示状態だけを変え、Document を書かない。filtered row の value/key 操作は通常経路、off に戻してもスクロール位置は保持する。
- **Density / scale:** 280〜340px、header32px、row28px。count と toggler を同じ24px帯に置き、フィルタ説明を本文へ増やさない。
- **Reuse-vs-scratch note:** track の有無と projection の既存 row を再利用し、filter は view-only state にする。各 section に別々の「animated」判定や property catalogue を新設しない。

## I09 — Playhead Value Lens

- **ID:** I09
- **名前:** Playhead Value Lens
- **解決する問題:** 現在 playhead で評価された値と、静止値/キーの存在が混ざって見える問題を解決する。
- **Hero / creation role:** 今見えているフレームを基準に、前後のポーズを同じ hero の連続として調整する。
- **レイアウト / visual hierarchy:** panel top に `FRAME 042 · 00:01.400` の lens band、続いて大きな current value、下に `source: static / key / track` の小さな provenance line と通常 row。値は右寄せ、source は色でなく text でも読めるようにする。
- **Interaction / entry:** playhead は Inspector から編集せず read-only。value click/drag と Key click は既存 semantics。`tooltip` で「評価値は現在時刻の値」と説明し、source badge は押せない。
- **Density / scale:** band36px、hero value44px、通常 row28px。数値を大きくするのは lens の一つだけで、縦スクロールを圧迫しない。
- **Reuse-vs-scratch note:** `StoreView::value_at`、key state、既存 playhead/session read を再利用する。source line の表示 adapter だけ自前にし、評価経路を Inspector 独自に複製しない。

## I10 — Property Source Ledger

- **ID:** I10
- **名前:** Property Source Ledger
- **解決する問題:** 値が Track、Slot、Link のどこから来ているか分からず、編集しても期待した場所を変えられない問題を解決する。
- **Hero / creation role:** hero の見た目を「自分のキー」「共有された制御」「別レイヤーの値」のどれで作っているか理解し、後から崩れない制作を促す。
- **レイアウト / visual hierarchy:** property row の value 下に source strip を一行だけ置く。左に `Track / Slot / Link` label、中央に source 名、右に `KEY`/`OPEN` button。source strip は通常 row より薄く、編集値の主役を奪わない。
- **Interaction / entry:** source strip は read-only summary。Link の場合だけ I25/I26 の pick_list entry、Track/Slot は将来の詳細 drawer button を disabled/tooltip で予告する。value/key は今までどおり。
- **Density / scale:** row34px、source strip16px、320px幅。source 名が長い場合は ellipsis と full tooltip。
- **Reuse-vs-scratch note:** `StoreView::property_source`、`PropertySource`、existing Link projection を再利用する。source badge と adapter のみ自前で、Slot/Link の新しい解決規則は作らない。

## I11 — Keyframe Detail Drawer

- **ID:** I11
- **名前:** Keyframe Detail Drawer
- **解決する問題:** Key を押せることは分かっても、どの時刻を編集しているか、どの補間が働くかが見えない問題を解決する。
- **Hero / creation role:** hero の「間」を詰めるため、選択中 property のキー群を Inspector 内で時間順に読めるようにする。
- **レイアウト / visual hierarchy:** 通常 property rows の下に `selected property` drawer。上段に property 名と現在値、下段に横方向の `row` で菱形、時刻 label、selected state を並べ、最後に `Interpolation` summary。グラフ canvas は使わず標準 widget で story 化する。
- **Interaction / entry:** Key glyph click は drawer を開くだけの story entry、キー button は選択/解除、時刻の direct edit は後段として disabled placeholder。既存の Key click は drawer 外の通常 row で維持する。
- **Density / scale:** drawer72px、key marker16px、panel幅360px。キー数が多い時は横 `scrollable` ではなく要約（first/current/last）へ畳む。
- **Reuse-vs-scratch note:** `KeyframeTrack` の既存 read、timeline grammar の time order、既存 key glyph を再利用する。key selection state と graph editor は別機構を増やさず、詳細編集は未接続 story と明示する。

## I12 — Text Style Key Strip

- **ID:** I12
- **名前:** Text Style Key Strip
- **解決する問題:** Text の Size、Line Height、Tracking は animatable なのに、Content/Font/Justify と同じ静止欄に見えて keyframe の入口を失う問題を解決する。
- **Hero / creation role:** タイトルのサイズ、行間、字間を時間で変え、モチベーション動画の「出現」と「余韻」を作る。
- **レイアウト / visual hierarchy:** TEXT section を `Content card` と `Style card` に分け、Style card の Size/Line Height/Tracking だけ `PROP / VALUE / KEY` の三列、Font/Justify は二列。Style card 上部に text specimen を一行置く。
- **Interaction / entry:** Size/Line Height/Tracking は `text_style_key_button` と既存 drag-to-scrub/submit、Font は `pick_list`、Justify は cycle button、Content は `text_editor` と Cmd/Ctrl+Enter commit。異なる入口を見た目でも分ける。
- **Density / scale:** specimen32px、style row30px、content editor 72px。360px幅を基準にし、長い本文は editor 内で scrollable。
- **Reuse-vs-scratch note:** `TextSectionProjection`、`TextStyleField`、style track commit/toggle、`text_editor` を再利用する。style card の grouping と specimen preview だけ自前で、Content を KeyframeTrack に偽装しない。

## I13 — Effect Rack

- **ID:** I13
- **名前:** Effect Rack
- **解決する問題:** Effects が単なる設定行の束に見え、適用順・bypass・parameter の関係が読めない問題を解決する。
- **Hero / creation role:** Glow などの効果を積み、hero の輪郭・発光・余韻を「足す順序」として作る rack である。
- **レイアウト / visual hierarchy:** `EFFECTS` section を effect ごとの `container(column![effect header, params])` にする。header は name、bypass toggler、up/down/remove buttons、params は `indent + label/value/key`。effect 間は薄い `rule`。
- **Interaction / entry:** Bypass は `ToggleEffectBypass`、順序は move up/down、削除は remove。param value/key は Transform row 文法、未知 param は出さない。header collapse は presentation state。
- **Density / scale:** header30px、param28px、indent12px、rack幅340px。effect が多い時は一つだけ展開する story と全展開 story を比較できる。
- **Reuse-vs-scratch note:** `EffectRowProjection`、`effects_with_moved_*`、`toggle_inspector_effect_bypass`、`parameters_for_provider` を再利用する。rack card の見た目のみ自前で、effect param 専用 editor は作らない。

## I14 — Provider Parameter Console

- **ID:** I14
- **名前:** Provider Parameter Console
- **解決する問題:** provider ごとに parameter 数・型・keyframe 可否が違うのに、全 effect を同じ固定フォームで扱おうとする問題を解決する。
- **Hero / creation role:** Glow のような効果を「その provider が宣言した意味」の範囲で調整し、未定義の knob を増やさず hero の質感を作る。
- **レイアウト / visual hierarchy:** effect header の下に provider badge、capability summary、catalog 順の parameter rows。Scalar は数値 cell、Bool は toggler、Enum は compact button/pick list とし、右端 Key は descriptor の `keyframeable` が true の時だけ active にする。
- **Interaction / entry:** `ParameterDescriptor` の label/default/capability から editor を選ぶ。未知の kind は generic read-only row、unknown provider は I15 へ。parameter の編集確定は既存 field draft/Intent に合わせる。
- **Density / scale:** provider badge22px、param28px、最大2列ではなく一列優先。effect 名と provider 名を混ぜず、幅320pxで可読性を守る。
- **Reuse-vs-scratch note:** `InspectorDevice`、`ParameterDescriptor`、`ParameterCapabilities`、provider registry を委託先にする。type-to-editor の最小 adapter 以外は固定 parameter enum や provider-specific scratch を作らない。

## I15 — Unknown Provider Safe Dock

- **ID:** I15
- **名前:** Unknown Provider Safe Dock
- **解決する問題:** 未知 provider に対して UI が勝手に parameter を捏造したり、空白だけを出して原因を隠したりする問題を解決する。
- **Hero / creation role:** 既存 hero を壊さず、読み込んだ効果が「存在するが編集不能」なのかを明確にし、次の復旧判断へ進める。
- **レイアウト / visual hierarchy:** effect header は通常の name/bypass/order を保持。params area は `container(column![warning icon, "Provider unavailable", provider id, "0 declared parameters"])`。下に read-only `preserved tracks` summary と retry/replace の future button を置く。
- **Interaction / entry:** provider id は copyable text、bypass/order/remove は通常どおり。parameter rows/pick_list は表示しない。warning tooltip に「catalog が空なので意味を発明していない」と出す。
- **Density / scale:** warning card64px、header30px、320px幅。警告色だけに依存せず、見出しと provider id を必ず文字で置く。
- **Reuse-vs-scratch note:** `device_for_provider(...)=None` と空 parameter catalog の既存 fallback を再利用する。error card と future action の見た目だけ自前で、未知 provider の schema decoder は Inspector で scratch しない。

## I16 — Effect Order Ladder

- **ID:** I16
- **名前:** Effect Order Ladder
- **解決する問題:** 複数 effect の上下順が小さな up/down button だけでは把握しにくく、見た目の因果を作れない問題を解決する。
- **Hero / creation role:** `Color → Blur → Glow` のような適用順を視覚的な梯子にし、hero の輪郭から発光までを上から下へ組み立てる。
- **レイアウト / visual hierarchy:** 左に細い順序 rail `01/02/03`、中央に effect header、右に bypass/remove。展開した effect だけ params をインデントし、折りたたみ effect は provider/name/one-line summary にする。
- **Interaction / entry:** up/down buttons は既存 move Intent、並べ替え drag は story では disabled として表示し、未実装の別 write path を暗示しない。header click で collapse、param row は通常編集。
- **Density / scale:** order rail24px、header30px、param28px、幅340px。3〜5 effect を一画面で比較できる縦密度を基準にする。
- **Reuse-vs-scratch note:** `effects_with_moved_up/down` と effect list order を再利用する。rail と collapse decoration だけ自前で、drag reorder の意味は追加しない。

## I17 — Bypass A/B Strip

- **ID:** I17
- **名前:** Bypass A/B Strip
- **解決する問題:** effect の enabled が静止 toggle に見え、bypass と remove の違い、時間で切り替わる状態が分からない問題を解決する。
- **Hero / creation role:** 効果あり/なしをすばやく聴き比べ・見比べし、hero に本当に必要な効果だけを残す。
- **レイアウト / visual hierarchy:** effect header に `ON/OFF` label、状態 dot、Key glyph を横並びにし、その下に params。右端に remove を離して配置し、破壊操作と bypass を視覚的に分ける。
- **Interaction / entry:** `toggler`/header button は enabled track の current evaluated value を切り替え、Key glyph は `KeyRow::EffectEnabled` の3状態。remove は別 button + tooltip。状態 dot は decoration であり真実を保持しない。
- **Density / scale:** header32px、A/B strip は24px、params28px。色覚差に備え ON/OFF 文字を常に表示する。
- **Reuse-vs-scratch note:** `EffectRowProjection.enabled`、`enabled_key`、`toggle_inspector_effect_bypass` を再利用する。A/B strip の見た目だけ自前で、bypass を effect list から除去する処理は作らない。

## I18 — Copy-First Text Editor

- **ID:** I18
- **名前:** Copy-First Text Editor
- **解決する問題:** Text layer を選んでも本文入力が目立たず、hero のメッセージを作るまでに設定欄を通過しすぎる問題を解決する。
- **Hero / creation role:** コピーを最初に置き、フォント・サイズ・色を後から整える。モチベーション動画の言葉を作るための primary authoring surface。
- **レイアウト / visual hierarchy:** TEXT section の先頭に `text_editor` を大きな card として置き、下に `Font Family / Size / Justify`、さらに `Line Height / Tracking / color`。editor 上部に character count ではなく layer name と本文の role label を置く。
- **Interaction / entry:** `ContentEditorAction` で編集し Cmd/Ctrl+Enter で commit、blur でも既存規約に従う。Font は `pick_list`、Size/Line Height/Tracking は text field + drag/key、Justify は cycle button。
- **Density / scale:** editor84px、style row30px、320〜380px幅。本文の複数行を優先し、font path の技術情報は tooltip に隠す。
- **Reuse-vs-scratch note:** `text_editor`、`ContentEditorCommit`、`TextSectionProjection`、font pick の既存経路を再利用する。copy card の背景と順番だけ自前で、text document の部分更新モデルを新設しない。

## I19 — Typography Specimen Rail

- **ID:** I19
- **名前:** Typography Specimen Rail
- **解決する問題:** Font、Size、Line Height、Tracking の数値を見ても、実際の見出しの印象を想像しにくい問題を解決する。
- **Hero / creation role:** hero の言葉を「読める」だけでなく、ポスターの声量・呼吸・重心として調整する。
- **レイアウト / visual hierarchy:** 上に実際の `content` を `text` で描く specimen band、下に typography controls を `column`。Font row は family/path summary、Size/Line Height/Tracking は値+Key、Justify は三択を巡回する compact button。
- **Interaction / entry:** controls の変更は既存の commit semantics、specimen は projection の resolved style だけを読む。specimen 自体は編集不可で、editor entry は I18 の content card へ tooltip link とする。
- **Density / scale:** specimen56px、control28px、panel360px。本文が長い時は一行 ellipsis ではなく最初の2行を clip して typographic rhythm を見せる。
- **Reuse-vs-scratch note:** `TextDocumentStyle`、resolved projection、既存 token typography を再利用する。preview text の表示 adapter だけ自前で、別 font renderer や style cache は作らない。

## I20 — Text Motion Workbook

- **ID:** I20
- **名前:** Text Motion Workbook
- **解決する問題:** Text style の keyframeable fields と静止 fields が混ざり、文字の登場演出を設計しにくい問題を解決する。
- **Hero / creation role:** Size/Line Height/Tracking の3つを時間軸の小さな workbook として扱い、タイトルの grow、breath、tracking-in を作る。
- **レイアウト / visual hierarchy:** `text_editor` は折りたたみ、Style workbook を主役にする。各 animatable row に左 label、current value、Key glyph、右に `track present` mini line。Font/Justify は `STATIC` group にまとめる。
- **Interaction / entry:** Size/Line Height/Tracking の Key button と drag/submit は既存 text-style route。mini line は read-only story fixture。Font/Justify に Key を表示しない。
- **Density / scale:** workbook row32px、static row26px、mini line40px。340px幅、縦に3 animatable rows が常に見えることを優先する。
- **Reuse-vs-scratch note:** `TextStyleField`、`toggle_text_style_key`、`commit_text_style_track_field`、`KeyCellState` を再利用する。mini line は read-only 表現に留め、Content/Font/Justify の新しい keyframe 機構は作らない。

## I21 — Missing Font Recovery Card

- **ID:** I21
- **名前:** Missing Font Recovery Card
- **解決する問題:** Font path が壊れていても text layer が空白に見え、本文の問題なのか font の問題なのか分からない状態を解決する。
- **Hero / creation role:** hero のコピーを失わず、代替 font を選んで再び画面上の主役へ戻すための復旧席。
- **レイアウト / visual hierarchy:** specimen の場所に fallback text を描き、上部に `Font unavailable` banner、中央に family/path、下部に `pick_list` と `Use fallback` button。Size/Justify は下段で編集可能なまま残す。
- **Interaction / entry:** `PickFont` は family と path を同時に commit。pick list が空なら path を捏造せず、error text と既存 content editor entry を表示する。fallback は story action として disabled/available を fixture で分ける。
- **Density / scale:** banner32px、specimen56px、font recovery row36px、360px幅。赤一色にせず icon/text/path を併記する。
- **Reuse-vs-scratch note:** `font_family_row`、`commit_text_font_pick`、`TextDocumentProjection` の既存 fallback を再利用する。missing state のカードだけ自前で、font discovery や path resolver を Inspector に作らない。

## I22 — Swatch-First Color Desk

- **ID:** I22
- **名前:** Swatch-First Color Desk
- **解決する問題:** RGBA の数字を先に見せると、hero の色の印象と Fill/Stroke の対象が分からない問題を解決する。
- **Hero / creation role:** タイトルの Fill と Stroke を一目で比べ、画面の温度・コントラスト・ブランド色を決める。
- **レイアウト / visual hierarchy:** Color section 上部に大きな swatch pair `FILL | STROKE`、下に各 target の R/G/B/A channel rows。target label と swatch を左に固定し、8-bit value を右に置く。
- **Interaction / entry:** swatch click は対応 channel input に focus、channel は draft→commit。Fill と Stroke は target を明示し、Stroke None は既定色から promotion される既存意味を helper text で示す。
- **Density / scale:** swatch44px、channel26px、panel320〜360px。hex を新しい入力として増やさず、既存の8-bit channel convention を主にする。
- **Reuse-vs-scratch note:** `color_row`、`ColorFieldDraft`、`ColorTarget`、`parse_color_channel_u8`、既存 label chip formula を再利用する。swatch-first の layout だけ自前で、別 color model は作らない。

## I23 — Fill / Stroke Split

- **ID:** I23
- **名前:** Fill / Stroke Split
- **解決する問題:** Fill と Stroke の channel が縦に混ざり、どちらを編集したか分からない問題を解決する。
- **Hero / creation role:** 文字の面と縁を別々の役割として調整し、背景上で読める hero title を作る。
- **レイアウト / visual hierarchy:** `row![fill_card, stroke_card]` を横に置ける幅では二分割、狭幅では縦 stack。各 card は target swatch、target label、RGBA 4行、Key/animation は表示しない（現行 color module の範囲）。
- **Interaction / entry:** 各 card の channel input は target を payload に持ち、片方の変更で他方を触らない。Stroke None の場合は default base を使う既存 commit を表示し、card 自体は empty にならない。
- **Density / scale:** 広幅360px以上、card最小168px、channel24px。狭幅260pxでは縦 stack とし、左右を無理に潰さない。
- **Reuse-vs-scratch note:** `ColorTarget::Fill/Stroke`、commit の mismatched-draft guard、channel parser を再利用する。二枚 card の layout だけ自前で、color picker/wheel は追加しない。

## I24 — Color Channel Audit

- **ID:** I24
- **名前:** Color Channel Audit
- **解決する問題:** 色の見た目は良くても、alpha や一つの channel の入力だけが意図せず変わり、微調整の履歴が追えない問題を解決する。
- **Hero / creation role:** 最後の色合わせを精密に行い、背景上で文字が沈まない hero finish を作る。
- **レイアウト / visual hierarchy:** 上に swatch と `rgba(… )` summary、下に R/G/B/A の4行。各 row は channel name、8-bit input、thin value bar、reset affordance。summary は read-only。
- **Interaction / entry:** input は既存 draft/submit、bar は decoration。無効な文字列は row 内 error text として留め、Document を書かない。同値 commit は undo step を増やさない。
- **Density / scale:** summary36px、channel28px、320px幅。bar は4px、input は38px以上を確保し、数値の legibility を優先する。
- **Reuse-vs-scratch note:** `parse_color_channel_u8`、commit validation、same-value no-op、RGBA index mapping を再利用する。value bar の表示だけ自前で、色補正計算や別履歴を作らない。

## I25 — Link Matrix

- **ID:** I25
- **名前:** Link Matrix
- **解決する問題:** Position、Scale、Rotation、Opacity、Anchor のどれが別レイヤーに委譲されているかが、個別 pick list を開かないと分からない問題を解決する。
- **Hero / creation role:** hero の主役と補助レイヤーを結び、同じ動きを保ったまま複数素材で構図を作る。
- **レイアウト / visual hierarchy:** LINK section を5行固定の matrix にする。各 row は target property、現在 source（`—`/`Layer (#id) · Property`）、右端 `pick_list`/clear button。リンク済み行には薄い chain glyph を付ける。
- **Interaction / entry:** pick list は `LinkSourceCandidate` をそのまま表示し、candidate の layer id を必ず含める。clear は source を通常 track/slot に戻す既存 intent、cycle/error は link status helper に出す。
- **Density / scale:** row30px、property label68px、320px幅。5行を常時並べ、空の link も `—` として入口を残す。
- **Reuse-vs-scratch note:** `LinkTarget::ALL`、`LinkRowProjection`、`commit_inspector_link`、`clear_inspector_link`、cycle exclusion を再利用する。matrix decoration だけ自前で、effect/mask param link を勝手に拡張しない。

## I26 — Source Picker Spotlight

- **ID:** I26
- **名前:** Source Picker Spotlight
- **解決する問題:** 候補が多い project で、どの layer/property を link 元に選ぶかを確認しにくい問題を解決する。
- **Hero / creation role:** hero の制御元を意図的に選び、後からレイヤー名の重複で壊れない制御関係を作る。
- **レイアウト / visual hierarchy:** 選択中 target の一行だけを上部 spotlight に拡大し、`target → source` の矢印を見せる。下に他の4 target を compact list。spotlight 内に `pick_list` と current source chip、clear button を置く。
- **Interaction / entry:** row click で spotlight target を変える。pick list の label は `name (#id) · property`、循環候補は catalog から除外済みとして出さない。選択確定は既存 `SetPropertyLink` の一回で行う。
- **Density / scale:** spotlight56px、compact row24px、360px幅。候補リストは popup に任せ、panel 内に別検索 state を作らない。
- **Reuse-vs-scratch note:** `LinkSourceCandidate::display_label`、projection の cycle filter、stock `pick_list` を再利用する。spotlight の selection/presentation state だけ自前で、link graph の別解決器は作らない。

## I27 — Link Safety State

- **ID:** I27
- **名前:** Link Safety State
- **解決する問題:** 自己参照・循環・対象消失の link が「選べない」のか「壊れている」のか不明で、制作の因果を追えない問題を解決する。
- **Hero / creation role:** hero の制御関係を安全に保ち、リンクを使った演出が無言でループしたり消えたりしないようにする。
- **レイアウト / visual hierarchy:** link row に state chip `Linked / Unlinked / Blocked`、source label、reason line を置く。Blocked は `not allowed` icon + 明示文、Linked は chain glyph + source、Unlinked は通常の pick list を保持する。
- **Interaction / entry:** blocked candidate は popup に出さず、row tooltip で cycle reason を説明。source missing は read-only broken chip と clear action。成功した link の entry は I25 と同じ。
- **Density / scale:** row38px（reason がある時）、通常30px、320px幅。state は色だけでなく文字を表示する。
- **Reuse-vs-scratch note:** UI 側の `link_would_cycle` 相当の projection、store の最終拒否、既存 clear/commit を再利用する。reason text の adapter だけ自前で、循環判定を二重の新規アルゴリズムにしない。

## I28 — No Selection Landing

- **ID:** I28
- **名前:** No Selection Landing
- **解決する問題:** 選択なしの Inspector が単なる空白になり、次に何をすれば hero creation が始まるか分からない問題を解決する。
- **Hero / creation role:** 最初の選択を促し、空の project から「一枚の主役」を作る導線にする。通常ソフトの設定空間ではなく creation start screen として扱う。
- **レイアウト / visual hierarchy:** `container(column![large selection glyph, "Select a layer", one-line hint, shortcut chip])` を panel 中央に置く。現在の `選択なし — layer を選ぶと Transform / Attrs が並ぶ` の意味を保ち、補助 action は一つだけにする。
- **Interaction / entry:** panel 内の action は Stage/Timeline での選択へ focus を戻す read-only/host message。空白をクリックして勝手に layer を作らない。Esc は host の focus semantics に戻す。
- **Density / scale:** center card 220px、icon48px、headline16px、hint12px。panel幅240〜360pxで余白を多くし、空状態をエラー密度にしない。
- **Reuse-vs-scratch note:** 既存 `empty_state`、`Session` focus、token typography を再利用する。large glyph と shortcut chip の presentation だけ自前で、empty state から Document mutation を発明しない。

## I29 — Mixed Selection Triage

- **ID:** I29
- **名前:** Mixed Selection Triage
- **解決する問題:** Text、Shape、Media が混在した選択で、編集できる section とできない section が分からず、誤って全レイヤーへ適用しそうになる問題を解決する。
- **Hero / creation role:** 異なる素材をまとめて選び、共通のモーション/名前/可視性だけを揃えつつ、固有の hero controls は対象を選んで続ける。
- **レイアウト / visual hierarchy:** 上部に `N layers selected` band と type chips。中央に `Common controls`（bulk-safe attrs/transform の summary）、下に `Text only / Shape only / Media only` の collapsed cards。編集不能 card には対象数と理由を出す。
- **Interaction / entry:** common controls は既存 bulk entry、type card click は selection を変えず filter/read view だけを切り替える。Text card は全選択が Text の時だけ `text_editor`、混在時は read-only summary にする。
- **Density / scale:** band34px、type chip20px、card30px、320〜360px幅。混在理由は一行に限定し、長文 tooltip へ送る。
- **Reuse-vs-scratch note:** `selection_count`、`text_layer_count`、既存 multi-selection view と compatibility rule を再利用する。triage card の grouping だけ自前で、混在値を架空の共通 property に変換しない。

## I30 — Inspector Recovery Envelope

- **ID:** I30
- **名前:** Inspector Recovery Envelope
- **解決する問題:** projection error、unknown provider、missing source、壊れた値が同じ無言の空欄になり、hero を直す入口を失う問題を解決する。
- **Hero / creation role:** 既存の制作状態を隠さず、何が編集可能で何が復旧待ちかを見える化して、途中の hero を安全に救う。
- **レイアウト / visual hierarchy:** panel top に status envelope `Ready / Partial / Blocked`、その下に編集可能な sections、最後に `Recovery` stack。各 issue は severity icon、対象（layer/provider/property）、原因、safe action を一枚の `container` にする。正常な Transform は警告に埋めない。
- **Interaction / entry:** safe action は既存の retry/clear/remove/select-source へ限定し、原因不明の auto-fix button は出さない。編集可能な row は通常経路を維持し、壊れた row は read-only + tooltip。provider catalog が空なら I15 の fallback を使う。
- **Density / scale:** status28px、normal row28px、issue card52px、360px幅。問題がない時は Recovery stack 自体を出さず、警告を常設しない。
- **Reuse-vs-scratch note:** `device_for_provider` fallback、projection の欠損/empty 判定、既存 clear/remove/selection entry を再利用する。envelope の表示と issue grouping のみ自前で、Inspector が修復ロジックや別エラーモデルを所有しない。
