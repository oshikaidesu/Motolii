# Settings / Chrome panel — Icebook design drafts

Settings と pane 横断 chrome を比較するための、Icebook に渡せる30案。
これは実装仕様でも実窓の合否でもなく、同じ状態を異なる視線・密度・入口で見せる
story 候補である。採用しなかった案を製品機能数として数えない。

## 共通の story 契約

- Project / Composition / Background は Document の読み書き、Undo は既存の
  Document::apply / apply_all を通る。
- UI scale と色・寸法は Tokens / Dimensions、Autosave は AutoSaveConfig、
  Playback や cache は注入された read-only の統計を読む。
- Session と pane の開閉・フォーカスは既存 Shell / widget の状態を使う。
  Settings が同じ値のコピーを持つ案は採らない。
- メニューの操作は既存の menubar Item と Shell の message / Intent へ戻す。
  Icebook の fixture が未実装状態を表示しても、製品側に別の状態正本を増やさない。
- 色は暗い surface、明るい on-surface、意味色、focus 境界の既存 token を使う。
  section header、value input、hairline の代わりの余白、角丸 menu 面を共通文法とする。

## S01 — Project Landing / 制作開始

- **Area:** project / session
- **Problem solved:** 起動後に新規制作か既存作品への復帰かを決められず、最初の一歩で止まる。
- **Hero / creation role:** Create と Open を同じ視界の主役にし、最初の hero frame へ最短で到達させる。
- **Layout / visual hierarchy:** 上段に現在の project identity と保存状態、中央に大きい Create / Open の2カード、下段に最近の project と短いヒント。主役は2つの入口で、状態は補助帯にする。
- **Interaction / entry:** 歯車の Settings、または menubar の File / Window から入る。初期 focus は Create、Tab で Open、最近の行は Enter で開く。
- **Density / scale:** balanced。現在 project 1件、最近の project 5件程度を medium の pane に収める。
- **Reuse vs scratch:** Document / Composition、Shell の recent/open 動線、menubar の message を再利用する。自前は2カードの配置だけで、最近の一覧や project 状態を Settings に複製しない。

## S02 — Session Desk / 今の制作

- **Area:** project / session / status
- **Problem solved:** いま何を編集しているか、保存済みか、どの pane を見ているかが別々で不安になる。
- **Hero / creation role:** 作品名と現在地を安定して見せ、考えを中断せず hero の調整へ戻れるようにする。
- **Layout / visual hierarchy:** 左に project 名・composition 名、中央に編集対象の短い要約、右に Saved / Unsaved と現在 pane。Save は一番目立たせず、現在地を主役にする。
- **Interaction / entry:** Settings を開いた瞬間に active session を read-only 表示。Save は既存 File / Save と同じ message、project 名の編集入口は既存 rename がある場合だけ出す。
- **Density / scale:** sparse。1つの active session と3つの状態だけを大きく読む。
- **Reuse vs scratch:** Shell の既存 session fields、Document の dirty 状態、menubar の Save を読む。自前は identity の並べ替えだけで、Session Desk 用の状態所有者を作らない。

## S03 — Lifecycle Ladder / 開く→作る→保存→書き出し

- **Area:** project / session / recovery
- **Problem solved:** 開始・編集・保存・書き出しが別機能に見え、どこまで進んだか分からない。
- **Hero / creation role:** hero を作る行程を4段に見せ、作品を外へ出すまでの不安を減らす。
- **Layout / visual hierarchy:** 横4段の Open / Create / Save / Export ladder。active step は色だけでなく短い状態文と操作可能な action を持ち、未到達 step は muted にする。
- **Interaction / entry:** 段をクリックすると既存 pane または menubar action へ移動する。Save / Export の状態は read-only の確認で、ladder 自体が処理を実行する新経路にはならない。
- **Density / scale:** balanced。4 step と各1行の結果を横幅 medium で見せる。
- **Reuse vs scratch:** File lifecycle と Export の既存 message、Document / session status を再利用する。自前は順序の投影だけで、進捗を二重に記録しない。

## S04 — Recent Shelf / 最近のプロジェクト

- **Area:** project / session / recovery
- **Problem solved:** 一度閉じた作品を探し直す手間が大きく、制作の勢いが切れる。
- **Hero / creation role:** 直近の hero を一目で再開でき、発想の断絶を小さくする。
- **Layout / visual hierarchy:** 最近順の縦 shelf。各行は project 名、composition、保存時刻、保存状態だけ。最上段に Continue current、最下段に Open… を置く。
- **Interaction / entry:** keyboard で行を上下移動し Enter で既存 Open を呼ぶ。行の削除・pin・タグはこの案では作らず、押せるように見える chrome を増やさない。
- **Density / scale:** dense。8〜12行を scroll なしまたは短い scroll で扱う、小さめの panel。
- **Reuse vs scratch:** OS / Shell の既存 recent source と Document open を再利用する。自前は行の視覚化のみで、Settings 内に recent database を作らない。データが無い story fixture では空状態1行だけを出す。

## S05 — Handoff Card / 再開地点

- **Area:** session / recovery
- **Problem solved:** 再起動すると最後に見ていた時刻・pane・選択対象を失い、作品へ戻るまでに迷う。
- **Hero / creation role:** 前回の思考状態を思い出させ、hero の続きをすぐ試せるようにする。
- **Layout / visual hierarchy:** 大きい read-only preview の横に Last project、playhead、active pane、selection の4項目。下に Resume と Open clean の2入口を置く。
- **Interaction / entry:** Resume は既存の session restore がある場合だけ使用し、無い場合は story で unavailable と表示する。Open clean は既存 Open と同じ入口に戻す。
- **Density / scale:** balanced。preview 1枚と4項目、2 action を large-medium で配置する。
- **Reuse vs scratch:** Session の既存 restore 値と Stage の read-only preview を再利用する。自前はカード構造だけで、再開地点の保存先を Settings に増やさない。

## S06 — Unsaved Guard / 未保存の出口

- **Area:** session / recovery / status
- **Problem solved:** 変更を試すことが怖く、閉じる・切り替える・書き出す前に喪失しそうになる。
- **Hero / creation role:** 失敗を恐れず hero の変化を試せる安全網を前面に出す。
- **Layout / visual hierarchy:** pane 上端に Saved / Unsaved の一行 guard。Unsaved のときだけ差分の要約と Save を出し、通常時は薄い status に畳む。
- **Interaction / entry:** Save は File / Save と同じ message。閉じる動線は既存の終了確認へ戻し、Settings 内に別の保存ボタンや dirty 判定を持たない。
- **Density / scale:** sparse。判断1つを大きくし、説明は2行以内に抑える。
- **Reuse vs scratch:** Document の dirty / save result と Shell の終了確認を再利用する。自前は guard の強調表示だけで、second save state を作らない。

## S07 — Autosave Vault / 自動保存

- **Area:** recovery / project
- **Problem solved:** 自動保存が働いているか、何世代を戻せるか分からず、制作を止めて確認する。
- **Hero / creation role:** 実験の復帰可能性を見える化し、思い切った hero の試行を許す。
- **Layout / visual hierarchy:** 上段に Autosave enabled と最終保存時刻、中央に Save Every / Generations、下段に世代リスト。restore は危険操作として row の末尾に置く。
- **Interaction / entry:** 既存の toggler と数値 input を使い、Enter で確定する。世代の restore は既存 Document recovery が存在する場合だけ表示する。
- **Density / scale:** balanced。設定2行と世代4〜6行を scroll 可能な medium pane に入れる。
- **Reuse vs scratch:** AutoSaveConfig と shell の timer / restore 経路を再利用する。自前は世代の見せ方だけで、backup database や recovery state を Settings に所有させない。

## S08 — Recovery Ladder / 復旧の選択

- **Area:** recovery / status
- **Problem solved:** 失敗後に「手動保存」「自動保存」「現在の未保存」のどれを戻すべきか判断できない。
- **Hero / creation role:** 作品を失わずに安全な分岐を選べるため、hero の試行回数を守る。
- **Layout / visual hierarchy:** source を Manual / Auto / Current の縦3段にし、各段に時刻、状態、失われる変更の有無を出す。存在しない source は描かない。
- **Interaction / entry:** 各段を選ぶと既存の recovery action の確認へ進む。Cancel は元の pane へ戻り、復旧候補の選択を別 Document にしない。
- **Density / scale:** sparse-to-balanced。実在する候補だけで最大3段、各段は大きい row にする。
- **Reuse vs scratch:** File system / AutoSaveConfig / Document replay の既存経路へ委託する。自前は比較レイアウトだけで、復旧コピーを新規に保持しない。

## S09 — Neutral Appearance / 背景と UI scale

- **Area:** appearance / hero
- **Problem solved:** 背景色と UI の大きさが作品の見え方を邪魔し、hero の良し悪しを判断しにくい。
- **Hero / creation role:** Stage の結果を正確に見せ、背景の選択を表現判断へ変える。
- **Layout / visual hierarchy:** 上に背景 swatch と Black / White / Gray 18% / Transparent、下に RGBA value cells と UI Scale。Stage の read-only preview を左端に小さく添える。
- **Interaction / entry:** preset は即時確定、RGBA と UI Scale は text input + Enter。現在の Settings の順序と focus を保つ。
- **Density / scale:** balanced。preset 4個、RGBA 4セル、scale 1行を medium の scroll pane にする。
- **Reuse vs scratch:** Composition.background、BackgroundPreset、Tokens::ui_scale、既存 chrome をそのまま使う。自前は swatch と preview の配置だけで、appearance の別 state owner を作らない。

## S10 — Preset Carousel / Hero mood

- **Area:** appearance / hero
- **Problem solved:** 最初の一枚を作る前に背景の細かい値を決める必要があり、制作の勢いが落ちる。
- **Hero / creation role:** 4つの明確な背景 mood を即座に試し、結果を見て hero の方向を決める。
- **Layout / visual hierarchy:** 大きな4タイルを横または縦に並べ、色面そのものを主役にする。選択中は focus ring と文字ラベルの両方で示し、数値は補助に畳む。
- **Interaction / entry:** tile click / Enter が既存 BackgroundPreset を送る。RGBA の詳細編集は secondary action として同じ pane の下へ置く。
- **Density / scale:** sparse。4択を large tile で見せる narrow-to-medium pane。
- **Reuse vs scratch:** 既存4 preset と Document の undo を再利用する。自前は carousel の表現だけで、mood 名・preset 保存・別の背景モデルを発明しない。

## S11 — Contrast Check / 読みやすさ

- **Area:** appearance / accessibility
- **Problem solved:** 暗い画面で設定値・focus・status が読めず、作品を見る前に chrome と戦う。
- **Hero / creation role:** author が UI の見落としではなく hero の見え方に集中できる。
- **Layout / visual hierarchy:** 左に現在の設定面の小さな preview、右に Text / Control / Focus の3つの read-only check。問題がある行だけ、該当する appearance row への link を持つ。
- **Interaction / entry:** check row は選択して原因を表示するが、独自の色設定を直接変更しない。修正入口は既存 UI scale / token を読む Settings に戻す。
- **Density / scale:** balanced。3 check と各2行の説明を見渡せる medium pane。
- **Reuse vs scratch:** Colors、readability JSON、既存 focus style を参照する。自前は診断表示だけで、accessibility score や別の色設定 state を持たない。

## S12 — Scale Rehearsal / UI scale の予行

- **Area:** appearance / accessibility
- **Problem solved:** UI scale を変えた後に密度や focus の読め方が予想できず、戻すのが怖い。
- **Hero / creation role:** 見たい hero の大きさを保ちながら、制作環境の読みやすさを調整できる。
- **Layout / visual hierarchy:** 上に current scale、中央に Before / After の小さな panel sample、下に1つの UI Scale input と reset。preview は値の比較に限定する。
- **Interaction / entry:** input に入力して Enter、reset は既定値へ既存 save_ui_scale 経路で戻す。preview をクリックして値を書き換えない。
- **Density / scale:** sparse。2 preview と1 inputだけを large readable scale で置く。
- **Reuse vs scratch:** Tokens / Dimensions の既存 ui_scale を正本にする。自前は同じ token で作る sample の並べ方だけで、preview 専用 scale state を作らない。

## S13 — Input Map / ショートカット一覧

- **Area:** input / menus
- **Problem solved:** 入口はあるのに shortcut と menubar の関係が見えず、反復編集の手数が増える。
- **Hero / creation role:** hero の試行を止めず、頻出の編集・保存・pane 移動を覚えられるようにする。
- **Layout / visual hierarchy:** File / Edit / Layer / Window の category rail と、右の Command / Shortcut / Entry の3列。primary は command name、shortcut は muted。
- **Interaction / entry:** category click で同じ list を切り替え、row Enter で対応する既存 message を送る。shortcut の再割当はこの案に含めない。
- **Density / scale:** dense。category 4つ、各5〜8項目を compact text scale で scroll する。
- **Reuse vs scratch:** menubar Item、SHORTCUT_ONLY_REGISTRY、Shell key resolver を一つの読み口として使う。自前はカテゴリ表示だけで、keymap のコピーを Settings に持たない。

## S14 — Gesture Deck / 直接操作の説明

- **Area:** input / hero
- **Problem solved:** どの値が drag でき、どの値が text input なのか分からず、調整が遅い。
- **Hero / creation role:** 数値を素早く試し、hero の動きや見え方を手で探れるようにする。
- **Layout / visual hierarchy:** 左に gesture label、中央に対応する value cell の live sample、右に Type / Drag / Submit の短い説明。実際の値セルを主役にする。
- **Interaction / entry:** label press は既存の mouse_area / value drag、cell は既存 text input、Enter は既存 commit。説明カード自体は操作入口にしない。
- **Density / scale:** balanced。背景 RGBA、Composition、Autosave の代表3行を large-medium で見せる。
- **Reuse vs scratch:** 既存 value_input_style、BackgroundChannelDragPressed、CompFieldDragPressed を再利用する。自前は説明の配置だけで、gesture state を Settings に新設しない。

## S15 — Shortcut Conflict / 衝突の検査

- **Area:** input / accessibility / menus
- **Problem solved:** 同じキーが別の動詞に見えたり、表示 shortcut と実配線がずれて、誤操作を生む。
- **Hero / creation role:** 試行錯誤の速度を落とさず、意図しない編集を避ける。
- **Layout / visual hierarchy:** Command / Key / Source / Conflict の表。Conflict が無い行は muted、実際に衝突する行だけ warning text と原因を出す。
- **Interaction / entry:** row 選択で対応する menubar または shortcut-only entry を highlight する。再割当ボタンは作らず、現在の resolver の事実だけを出す。
- **Density / scale:** dense。15〜20行を scroll する compact table、warning は形と文言でも区別する。
- **Reuse vs scratch:** menubar Item と SHORTCUT_ONLY_REGISTRY、Shell resolver を再利用する。自前は join と表の見せ方だけで、keymap を二重管理しない。

## S16 — Focus Route / キーボードの道

- **Area:** accessibility / input / chrome
- **Problem solved:** Tab の順番と現在 focus が見えず、マウスに戻らないと Settings を完了できない。
- **Hero / creation role:** 視線と手を作品へ戻し、設定変更を制作の流れの中で完了できる。
- **Layout / visual hierarchy:** 左に 1〜8 の focus route、右に各 control の miniature。現在 focus は色だけでなく番号、outline、短い label で示す。
- **Interaction / entry:** Tab / Shift+Tab / Enter だけで全操作を進める。route は実際の widget order の読み取りで、クリック用の別ナビゲーションではない。
- **Density / scale:** sparse-to-balanced。8 target を大きい focus boundary で読める medium pane。
- **Reuse vs scratch:** Iced widget の focus、text_input、toggler、既存 button style を再利用する。自前は focus map の story 表示だけで、focus state を別に所有しない。

## S17 — Large Type / 読み取り優先

- **Area:** accessibility / appearance
- **Problem solved:** 小さな caption と数値欄が読めず、設定を間違えて hero の見え方を損なう。
- **Hero / creation role:** 読み取りの負担を減らし、視覚表現の判断へ集中させる。
- **Layout / visual hierarchy:** label を大きく、value をその次、hint を最小にする一列。アイコンだけの toggler は使わず、On / Off の文言を添える。
- **Interaction / entry:** UI Scale の既存 input から調整し、変更結果は同じ pane 全体に反映する。個別の text-size slider は作らない。
- **Density / scale:** sparse。6〜8行を large text scale で表示し、情報量より可読性を優先する。
- **Reuse vs scratch:** Tokens::ui_scale、Dimensions::theme、readability の閾値を再利用する。自前は並び替えだけで、Settings 専用の font scale を持たない。

## S18 — Reduced Motion / 安定したフィードバック

- **Area:** accessibility / status / hero
- **Problem solved:** hover・open・status の動きが注意を奪い、hero の時間変化と chrome の動きを混同する。
- **Hero / creation role:** 時間表現の動きだけを主役として残し、操作フィードバックを安定させる。
- **Layout / visual hierarchy:** 上に motion policy の read-only summary、中央に hover / open / success の静的 state sample、下に「作品の動きは変えない」という注記。
- **Interaction / entry:** sample は focus / hover の状態比較に使い、設定値の変更は既存 Tokens または将来の明示された accessibility owner へ戻す。未実装なら製品では表示しない。
- **Density / scale:** sparse。3 state sample を大きく見せ、説明を短くする。
- **Reuse vs scratch:** 既存 chrome の base / hover / focus style と Tokens を再利用する。自前は Icebook の状態比較だけで、motion preference の保存先や別設定 state をこの pane に作らない。

## S19 — Status Command Line / 一行の現在地

- **Area:** status / session / recovery
- **Problem solved:** 保存・入力・render の結果が section の間に埋もれ、次に何を直すか分からない。
- **Hero / creation role:** 制作を止める問題だけを一行で明示し、安心して次の hero experiment へ進める。
- **Layout / visual hierarchy:** pane 上端に icon + status sentence + time の一行 status command line。body は既存 Settings、下端に source link を置く。
- **Interaction / entry:** status を押すと原因のある既存 row または menubar action へ移る。dismiss は一時表示に限り、履歴削除の意味を持たせない。
- **Density / scale:** sparse。常時1件、長い内容は2行までに切る。
- **Reuse vs scratch:** Shell の既存 status、Document result、preview cache result を読む。自前は status の固定位置だけで、alert store や通知履歴を Settings に増やさない。

## S20 — Diagnostics Drawer / 現在の健康状態

- **Area:** status / project / chrome
- **Problem solved:** composition の有無、cache の状態、保存状態を確認したいが、常設の情報量で作業面が狭くなる。
- **Hero / creation role:** hero の結果を疑うときだけ内部状態を開き、普段は創作面を広く保つ。
- **Layout / visual hierarchy:** 通常は Settings 本体だけ。右下または下端の Diagnostics disclosure を開いた時だけ、read-only の Composition / Cache / Save 3列が現れる。
- **Interaction / entry:** disclosure は view chrome の開閉、row は source pane への移動。数値を書き換える入口は置かない。
- **Density / scale:** balanced。閉じた状態は sparse、開いた状態は3項目の dense drawer。
- **Reuse vs scratch:** sections::ViewModel の composition / preview_cache、Shell status、Document view を再利用する。自前は drawer layout のみで、diagnostic snapshot を保存しない。

## S21 — Event Log / 操作の復元

- **Area:** status / recovery
- **Problem solved:** 直前に何が起きたか分からず、Undo するか再編集するか判断できない。
- **Hero / creation role:** 試した変化を追跡できるため、hero の反復を安全に続けられる。
- **Layout / visual hierarchy:** 左に時系列の短い event rows、右に選択 event の source と結果。最上段に現在状態、履歴は補助にする。
- **Interaction / entry:** row を選ぶのは read-only。Undo / Redo は既存 Edit menubar に戻し、Event Log 内で独自の履歴再生はしない。
- **Density / scale:** dense。10〜20件を compact row と scroll で見せる。
- **Reuse vs scratch:** Document の既存 undo / redo と Shell status を再利用する。自前は event の投影だけで、第二の操作履歴や復旧モデルを作らない。

## S22 — Command Palette / コマンド入口

- **Area:** menus / input / chrome
- **Problem solved:** menubar の階層を探している間に制作の集中が切れる。
- **Hero / creation role:** 次の hero 操作へ視線を戻すまでの探索を短くする。
- **Layout / visual hierarchy:** Settings の上に一枚の raised surface を重ね、上に入力、中央に最近の commands、右に shortcut。背景は opaque にして背後を誤操作させない。
- **Interaction / entry:** menubar から開き、入力で既存 Item の label を絞り、Enter で message を publish する。ショートカットは既存のものだけ表示し、新しい Cmd+K を発明しない。
- **Density / scale:** dense。12〜20 command rows を compact text で表示し、検索結果を最大8行に抑える。
- **Reuse vs scratch:** motolii-menubar の Item / leaf と既存 message を再利用する。自前は一時的な filter と overlay だけで、command registry や別の action state を持たない。

## S23 — Menu Atlas / menubar の地図

- **Area:** menus / accessibility
- **Problem solved:** File / Edit / Layer / Window / Help のどこに操作があるか予測できない。
- **Hero / creation role:** 操作の地図を覚え、hero の調整を止めずに適切な入口へ進める。
- **Layout / visual hierarchy:** 上段に既存 top-level menu の横並び、中央に選択 menu の leaves、右に shortcut と説明。menubar の素の文字と hover 面をそのまま大きく見せる。
- **Interaction / entry:** root を click / keyboard で選び、leaf Enter で既存 message を送る。menu の open state は widget に任せる。
- **Density / scale:** dense。root 5本、leaf 最大10本を menu width と compact row に合わせる。
- **Reuse vs scratch:** vendored menu_bar、Menu / Item、leaf style を直接再利用する。自前は atlas の story composition だけで、menu action の別定義を作らない。

## S24 — Context Bridge / 選択対象からのメニュー

- **Area:** menus / chrome / input
- **Problem solved:** clip・layer・keyframe の右クリックだけに操作が閉じ、別の入口との対応が分からない。
- **Hero / creation role:** 選択中の対象から Copy / Split / Group / Interpolation などへ直行でき、試行を止めない。
- **Layout / visual hierarchy:** 上に selected object の短い identity、中央に context items、下に「同じ操作は Edit / Layer にもある」という entry hint。
- **Interaction / entry:** context menu は現在の選択へ既存 message を送る。menu item と shortcut の重複は許すが、context 専用の意味や第二 state を作らない。
- **Density / scale:** balanced。対象1件と8項目前後の raised menu を medium で表示する。
- **Reuse vs scratch:** motolii-menubar の context_items、Item、既存選択状態を再利用する。自前は bridge の視覚構造だけで、context 専用 command model を持たない。

## S25 — Chrome Calm / 一列の静かな設定

- **Area:** chrome / appearance / accessibility
- **Problem solved:** 境界線・面・見出しが多く、設定そのものが作品の preview と競合する。
- **Hero / creation role:** chrome を背景へ退かせ、Stage の hero result を視線の主役にする。
- **Layout / visual hierarchy:** SETTINGS header、section header、value rows の一列だけ。面の差と余白を階層にし、常時 border は使わない。warning/status だけ意味色を許す。
- **Interaction / entry:** gear で開閉、section は自然な scroll、value は既存 input。押せない説明を button に見せない。
- **Density / scale:** sparse。Composition / Appearance / Autosave / Playback を各2〜4行に抑え、large-medium の行高にする。
- **Reuse vs scratch:** panel_container_style、section_header、value_input_style、tokens の surface を再利用する。自前は一本の並びだけで、chrome token や状態を増やさない。

## S26 — Chrome Split / master-detail

- **Area:** chrome / project / appearance / input
- **Problem solved:** section が増えると縦 scroll だけでは現在地を失い、設定を探せない。
- **Hero / creation role:** 左の目的選択で右の設定面を絞り、hero へ戻るまでの視線移動を短くする。
- **Layout / visual hierarchy:** 左に Project / Appearance / Input / Recovery / Status の master list、右に1 section の detail。active は text + shape + focus で示す。
- **Interaction / entry:** master click / keyboard で同じ data source の section を切り替える。右側は既存 sections::view の一部を投影し、各 master が独自の state を持たない。
- **Density / scale:** balanced。左5項目、右3〜6行を medium-wide pane で表示する。
- **Reuse vs scratch:** existing section view、scrollable、Tokens の列幅を再利用する。自前は master-detail の layout と一時的な選択だけで、section ごとの状態 owner を増やさない。

## S27 — Chrome Rail / 端の設定レール

- **Area:** chrome / session / menus
- **Problem solved:** Settings を開くたびに Stage の面積が大きく減り、hero を見ながら調整できない。
- **Hero / creation role:** Stage を常に見えるまま、必要な設定だけを端から呼び出せる。
- **Layout / visual hierarchy:** pane の端に細い gear / section glyph rail、選択時だけ raised popover。popover 内は既存の一列 Settings、rail は主役にしない。
- **Interaction / entry:** ToggleSettingsPanel と Window / Settings の既存入口で開く。click outside / Escape は表示を閉じるだけで Document を触らない。
- **Density / scale:** sparse。rail は1列、popover は active section 3〜6行の compact-medium。
- **Reuse vs scratch:** ToggleSettingsPanel、panel_container_style、section_header、menubar Window entry を再利用する。自前は anchor positioning のみで、rail 用の新しい pane state を作らない。

## S28 — Chrome Adaptive Stack / 幅に応じる設定

- **Area:** chrome / accessibility / appearance
- **Problem solved:** narrow pane で Composition の2列値が潰れ、wide pane では余白が余り、密度が不安定になる。
- **Hero / creation role:** どの作業環境でも値の意味と hero の preview を同時に保つ。
- **Layout / visual hierarchy:** wide では Size / Time / Background を2列、narrow では同じ順序の1列へ自然に積む。section header と primary value の順序は変えない。
- **Interaction / entry:** window / pane resize と ui_scale の結果だけで layout が変わる。compact / comfortable の新しい保存 toggle は置かない。
- **Density / scale:** adaptive。wide は balanced、narrow は sparse rows、large UI scale では自動で1列に落とす。
- **Reuse vs scratch:** Dimensions、Taffy transfer、existing comp_cells_row と responsive width を再利用する。自前は配置規則だけで、density preference の第二 state owner を作らない。

## S29 — Hero Launch Console / hero を始める

- **Area:** hero / project / appearance / session
- **Problem solved:** Settings が保守項目の集まりに見え、作品の主役を作る入口になっていない。
- **Hero / creation role:** 「作品を始める」ための Create / Open、背景、Preview の3判断を前面に出し、Motolii の目的を最初に伝える。
- **Layout / visual hierarchy:** 上に短い Hero creation cue、中央に read-only Stage preview、下に Create / Open と4つの BackgroundPreset。詳細な lifecycle / autosave は secondary disclosure。
- **Interaction / entry:** Create / Open は既存 lifecycle、preset は Composition.background、preview は Stage の読み取り。cue や preview は click action を持たない。
- **Density / scale:** sparse。preview 1枚、primary action 2つ、preset 4つを large scale で見せる。
- **Reuse vs scratch:** Document / Composition、Stage preview、BackgroundPreset、Tokens を再利用する。自前は hero-oriented な順番だけで、hero session や別の creation state を導入しない。

## S30 — Confidence Gate / 共有前の信頼確認

- **Area:** status / recovery / project / menus
- **Problem solved:** 今見えている preview が保存済みか、cache が揃っているか、外へ出せる結果か判断できない。
- **Hero / creation role:** hero を共有する前の不安を3〜5件の確認へ縮め、結果への信頼を作る。
- **Layout / visual hierarchy:** 上に read-only preview、中央に Saved / Composition / Preview / Audio-or-Export の status matrix、下に既存 Save / Export action。欠落データの行は出さない。
- **Interaction / entry:** status row は原因のある Settings / Timeline / Export へ移る。Save / Export は menubar の既存 message を呼び、matrix が処理を複製しない。
- **Density / scale:** balanced。確認3〜5件と2 actionを medium pane に置き、warning は文字・形・意味色で示す。
- **Reuse vs scratch:** Document dirty、Engine / preview cache、Export status、menubar action を再利用する。自前は確認matrixの配置だけで、confidence や export result の第二 state owner を作らない。

