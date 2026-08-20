# R8: Unity Timeline / Unreal Sequencer / Spine — Timeline操作意味論の逆算

裁定149。一次資料はすべて公式ドキュメント(docs.unity3d.com / dev.epicgames.com / esotericsoftware.com)。コードは読んでいない。
突き合わせ対象: `/Users/member_ottoto/rust_ae/Motolii/next/reference/timeline-grammar.md`(以下「正典」)。

判定4値: **既載**(正典 §番号)/ **抜け**(Motolii に意味がありそう)/ **対象外**(不採用理由つき)/ **保留**。

---

## 1. Unity Timeline window(クリップ操作)

### 1-1 Edit Mode 切替(Mix / Ripple / Replace)
- 起動条件(既定割当): ツールバーのモード切替、または一時切替キー **1**(Mix)/ **2**(Ripple)/ **3**(Replace)を押している間だけ切替
- ドラッグ中の意味: Mix=隣接に影響しない・重なりで blend 生成(白矢印)。Ripple=後続を巻き込み隙間を保存(黄矢印・黄線)。Replace=重なりを切って置換(赤矢印・赤線、重なり部が半透明化)
- 確定: マウスアップ。キャンセル: 明記なし
- フィードバック: モードごとに矢印色とカーソルが変わる
- 出典: https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/clip-overview.html
- **判定: 対象外**。Ripple/Replace は正典 拘束1 が明示的に不採用とした trim family(ripple/…)そのもの。Mix の「重なり=blend 生成」は §1-5 で個別に扱う。

### 1-2 move(本体ドラッグ)
- 起動条件: Mix モードでクリップ中央をドラッグ
- ドラッグ中: 黒線で選択表示、ルーラに開始/終了時刻表示。複数選択も同一ルールで一括移動。白ゴースト=着地可、赤ゴースト=不可(空きトラック外など)
- 確定: マウスアップ。Inspector の Start 値でも直接編集可
- 出典: https://docs.unity3d.com/Packages/com.unity.timeline@1.7/manual/clp_position.html
- **判定: 既載**(正典 §2 move)。Motolii は絶対値再計算・root のみ移動・塊制約など egui 実測から精緻化済みで、Unity 側は「複数選択も同一ルール」レベルの粗さ。追加知見なし。

### 1-3 trim(端 8px 相当)
- 起動条件: Mix モードで clip 端にホバー→trim カーソル→ドラッグ。または playhead をクリップ内に置き右クリック→Editing > Trim Start / Trim End
- 特徴: Ripple/Replace 以外は隣接非連動。Inspector で Start/End/Duration/Clip In を数値編集可。ループクリップには Complete Last Loop / Trim Last Loop
- 出典: https://docs.unity3d.com/2018.4/Documentation/Manual/TimelineTrimmingClips.html , https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/clip-trim.html
- **判定**: 基本 trim は**既載**(正典 §2 trim)。「playhead へ trim」(右クリックメニュー起動、AE の Trim Start/End と同型)は**抜け候補**——正典は split(Cmd+K)しか playhead 起点の編集を持たない。「trim するが分割しない」操作は MV 編集で頻出(素材の頭出し合わせ)。Complete/Trim Last Loop はクリップ内ループという Motolii 未対応の概念に依存するため**対象外**(現行データモデルに「クリップ内ループ」なし)。

### 1-4 blend(Mix モードでの重なり=クロスフェード)
- 起動条件: Mix モードで clip を隣接クリップへ重なるまでドラッグ、または Ctrl/Cmd を押しながら端を反対側へドラッグして ease-out/ease-in を追加
- 意味: 重なった領域が自動的に crossfade 区間になる。デフォルトは自動イージング曲線、Inspector で Auto→Manual に切替えて Blend Curves を個別編集(Curve Editor)
- 確定: マウスアップ。フィードバック: 重なり部が特殊表示
- 出典: https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/clip-ease.html
- **判定: 対象外**。正典拘束1「自由な絶対時間配置・重なり自由」は明示的に gapless/trim family を退けているが、blend は別の懸念——「重なり=自動的に何らかの合成処理が起動する」という**暗黙のクリップ間相互作用**そのものが、Motolii の「クリップは独立・重なりは意味を持たない(ただの視覚的重なり)」という設計と衝突する。Unity の blend は Mix モードという状態を介した副作用であり、Motolii には「モード」という状態自体がない(1文法で全操作を賄う設計、正典冒頭)。**タスク指定の要点**: Motolii で「重なり」に自動合成の意味を持たせるなら、それは compositing レイヤーの仕事(opacity/blend mode という別のパラメータ系)であって timeline 操作文法の仕事ではない、という切り分けで対象外とするのが妥当。ただし単一クリップの ease-in/ease-out 自体(他クリップとの重なり不要、Ctrl+drag で自クリップの終端減衰を作る)は下記1-4bで別掲。

### 1-4b ease-in/ease-out(単一クリップの減衰、Ctrl/Cmd+端ドラッグ)
- 起動条件: Ctrl(Cmd)を押しながらクリップ端を内側へドラッグ
- 意味: Ease In/Out Duration が Inspector にセットされる。Manual切替でBlend Curveをカーブエディタ編集
- 出典: 同上(clip-ease.html)
- **判定: 保留**。1-5 の blend(2クリップ間合成)とは独立して、「クリップ自身の頭/尻を滑らかに立ち上げ/下げる」操作は Motolii の param 系(opacity キーフレーム)で代替可能とも読めるが、「専用ジェスチャ(端をCtrl+ドラッグ)」という**操作文法としての省略記法**がある点は他社に共通する省力化パターン。正典 §7 未決の穴のどれにも該当しない新規論点のため保留に落とす。

### 1-5 snapping(Edge Snap / Snap to Frame)
- 既定: 両方 ON。Edge Snap のしきい値=**画面 10px**(Motolii SNAP_PX=7px と同系だが値が違う)。スナップ対象=Timeline Playhead / 同トラック他クリップ端 / 他トラッククリップ端 / Timeline 全体の開始・終了
- フィードバック: スナップした start/end guide が白で再描画
- 出典: https://docs.unity3d.com/2018.4/Documentation//Manual/TimelineSettings.html
- **判定: 既載**(正典 §1 SNAP_PX, §2 move のスナップ対象列挙)。Unity は「Edge Snap を切って正確な ease を作る」という運用助言があり、正典の「Alt でスナップ一時無効」の存在理由(精密操作のため)を裏付ける追加証拠。

### 1-6 markers
- 起動条件: Marker track を右クリックしてマーカー種別を選択
- 出典: https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/wf-custom-marker.html
- **判定: 既載**(正典 §5 locator と同系機能)。Unity のドキュメントは起動ジェスチャの細部(ドラッグ移動・クリックジャンプの有無)を明記しておらず、正典の locator 仕様(M タップ即置き・ドラッグ移動・右クリック削除)の方が具体的。追加知見なし。

---

## 2. Unity Animation window(ドープシート/カーブ)

### 2-1 モード切替(Dopesheet / Curves)
- 起動条件: 下部ボタンで切替
- 出典: https://docs.unity3d.com/Manual/animeditor-UsingAnimationEditor.html
- **判定: 対象外**。Motolii は正典冒頭で「1文法」志向、bar上にキーを重ねて表示する設計(§1.5 クリップ面)であり、専用カーブ画面への切替という発想自体を持たない(カーブ編集は将来の別パネル候補で、この台帳の対象外)。

### 2-2 キー追加(ダブルクリック)
- 起動条件: カーブ上をダブルクリック、または右クリック→Add Key
- 出典: https://docs.unity3d.com/Manual/EditingCurves.html
- **判定: 対象外**。正典拘束4「ダブルクリック不使用」に抵触する操作そのもの。Unity はダブルクリックをキー追加に使うが Motolii は明示的にこの語彙を禁じている——良い対比事例。

### 2-3 キードラッグ+グリッドスナップ(Cmd/Ctrl押下)
- 意味: 通常ドラッグは自由移動、**Cmd/Ctrl押下でグリッドスナップが有効化**(=スナップは既定OFFで修飾キーがONにする、AEと同じ極性)
- 出典: 同上
- **判定: 既載/補強**。正典 §2 は「Alt でスナップ一時無効」(常時ON→Altで切る)を明記の上で AE と逆極性であることも自覚している。Unity のキー編集はさらに別の極性(既定OFF→Cmd/CtrlでON)を採用しており、**3社目の前例**として「各社バラバラ」という正典の見立て(§3 キー群リタイムの記述と同型の観察)を補強する。結論を変える証拠ではないので既載。

### 2-4 選択(単独クリック/Shift追加/Ctrl個別解除/box select/Shift+box追加)
- 出典: 同上
- **判定: 既載**(正典 §3 選択、§4 marquee と対応)。Shift+box select で既存選択に追加、という挙動の明記は正典にはない粒度だが、意味論としては marquee の一般則(空白面から/追加選択)で吸収可能——新規性なし。

### 2-5 Tangent Types(Clamped Auto既定/Auto/Free Smooth/Flat/Broken系Free/Linear/Constant)
- 出典: 同上
- **判定: 既載**(正典 §3 イージング欄、および §7 未決2「イージングの現代化」)。Unity のタンジェント語彙は正典 §7-4(Rive の Cubic Value 相当をどこまで踏むか)の検討材料として既に想定内。

### 2-6 Filter by selection / **「Show: Animated」トグル**
- 起動条件: 左下のトグルボタン。ON で「Animation Curve を持たないプロパティ」を非表示。加えて Hierarchy で子 GameObject を選択→Filter by selectionボタンでその配下だけに絞る
- 出典: https://learn.unity.com/course/game-design-curricular-framework-resources/tutorial/2-1-animation-window-features-and-settings
- **判定: 既載/補強・重要**。正典 §5「候: Show Only Animated / 選択のみ表示」がまさにこの Unity 機能を指している。**これは公式ドキュメントで確認できた最も直接的な一次資料**——「今アニメートしているものだけ見せる」が Unity では実装名まで一致する形(Show: Animated)で存在する。正典の候補記述を「Unity 実装名: Show: Animated、対象=Animation Curve を持つプロパティのみ、既定OFF」まで具体化できる。

### 2-7 フレーム移動(,/.、Alt+,/.でキー間ジャンプ)
- 出典: https://docs.unity3d.com/Manual/animeditor-UsingAnimationEditor.html
- **判定: 既載**(正典 §5 矢印キー は playhead ±1/10フレームのみ規定。「次/前のキーへジャンプ」という語彙は正典に明記なし)。**抜け候補(小)**: 正典 §5 矢印キーは「playheadを動かす」とだけ書き、キーへのジャンプ(次のキーフレーム位置へワープ)を持たない。M タップと違い既存キーを辿るナビゲーションで、MV編集でテンポ確認に有用。ただし優先度は低い(既存の locator/矢印キーで代替可能)。

---

## 3. Unreal Engine Sequencer

### 3-1 Section の move/trim/blend、Key Bars / Layer Bars
- move: ドラッグで移動(スナップ規則は3-2)。trim: 端をドラッグ、または **Alt+]** / **Alt+[** で playhead へ trim(Unity の Trim Start/Endと同型、既定キー割当つきの点で Unity よりドキュメントが具体的)
- **Key Bars**: 隣接する2キー間の線をドラッグすると両キーをまとめてリタイム(区間内の曲線形状を保ったまま)
- **Layer Bars**: 中央ドラッグで子キー全部を平行移動、端ドラッグでスケール
- 出典: https://dev.epicgames.com/documentation/en-us/unreal-engine/creating-animation-keyframes-in-unreal-engine , https://dev.epicgames.com/documentation/en-us/unreal-engine/sequencer-editor-reference
- **判定**: move/trim は**既載**(正典 §2)。**Alt+]/Alt+[ で playhead へ trim** は Unity 1-3 と同じ**抜け候補**(playhead 起点 trim の2社目の裏付け——優先度が上がる)。Key Bars/Layer Bars は正典 §3「キー群のリタイム(裁定146 Cmd+ドラッグ)」と**同じ問題への別解**——Unreal は「隣接2キー間の線」「グループの端」という**専用の掴み場所**を作ることで対応しており、Motolii が Cmd+ドラッグという**修飾キー方式**を採ったことと対照的。正典裁定146は確定済みなので既載だが、「掴み場所を分けず修飾キーで区別する」という Motolii の選択の妥当性を裏付ける比較材料として記録。

### 3-2 Snapping(粒度別トグル)
- Snapping ドロップダウンに: Snap to the Interval / Snap to Keys and Sections / Snap Keys and Sections to the Playback Range / Snap to the Interval While Scrubbing / Snap to Keys While Scrubbing / Snap to the Pressed Key / Snap to the Dragged Key
- 出典: https://dev.epicgames.com/documentation/en-us/unreal-engine/sequencer-toolbar-reference
- **判定: 保留**。正典 §1/§2 のスナップは「対象を全部まとめてON/OFF」という単一トグル(Alt で一時無効)。Unreal は7種の粒度別トグルを持つ——将来 Motolii の利用者から「スクラブ中はスナップしたくないが移動中はしたい」等の要望が出た場合の設計余地として保留記録。現時点で正典を変える理由にはならない(拘束6のキーマップ層抽象化があれば後付け可能とも読める)。

### 3-3 キー選択(marquee/複数トラック横断)・複数キー相対移動
- marquee は個別クリックと共存、複数トラックを跨いで一括選択可。複数選択キーの相対移動は間隔を保持
- 範囲選択: **Ctrl+]** で playhead 右側全キー選択、**Ctrl+[** で左側全キー選択
- 出典: creating-animation-keyframes-in-unreal-engine
- **判定**: marquee・複数移動は**既載**(正典 §4 marquee、§3 選択)。**Ctrl+]/[ の「playhead から片側全部」選択は抜け候補**——正典 §3 選択は クリック/Cmd トグル/Shift 範囲の3種のみで、「playhead を境に全選択」という粒度を持たない。編集点の前後で一括リタイムしたい場面(MVのテンポ変更に伴う後続キー一斉シフト)に効く可能性がある。

### 3-4 Interpolation types + 数字ホットキー(1-5)で切替
- Cubic(Auto既定)/Cubic(User)/Cubic(Break)/Linear/Constant。右クリックまたは 1〜5 キーで即切替
- 出典: sequencer-toolbar-reference, creating-animation-keyframes-in-unreal-engine
- **判定: 既載**(正典 §3 イージング欄・§7未決1&4)。「全 param に統一入口を開く」という正典の推奨(§7-1)を後押しする実例。数字キーでの即時切替という省力化パターンは §7-4(イージングの現代化)の検討時に参照可。

### 3-5 コピー/ペースト/複製(Ctrl+D、**Alt+ドラッグ複製**)
- Ctrl+X/C/V でキーの切取/コピー/貼付(貼付は最左キーが playhead に揃う)。Ctrl+D または**Alt+ドラッグ**で複製
- 出典: creating-animation-keyframes-in-unreal-engine
- **判定: 既載**(正典 §4 Duplicate 候「Alt+ドラッグ複製」——Unreal がまさにこの割当。LottieFiles/Cavalry/Lottielab に続く**4社目の一致**で、候補の優先度が実質的に上がる証拠)。ただし正典 §7-3 が指摘する「Alt=スナップ無効(bar)と複製(キー)の文脈分離」の論点はUnrealでも同じ Alt を複製に単独使用しており(Unreal の bar 側スナップ無効キーは別途 Shift 系)、**文脈分離という Motolii の設計仮説を否定しない**追加サンプル。

### 3-6 Selection Range / Playback Range / Marks
- Selection Range: 右クリックで Start(i)/End(o)設定、範囲内キー・セクションの一括選択に使う
- Playback Range: 緑/赤マーカー、[ / ] で Start/End 設定、ロック可
- Marks: 右クリック→Add Mark(M)、ラベル・色・フレーム番号編集、Determinism Fence として使用可
- 出典: sequencer-editor-reference
- **判定**: Marks は**既載**(正典 §5 locator と同型、M という既定キーまで一致——裁定146以前からの locator 設計を裏付ける独立証拠)。Playback Range は正典のループ帯(§5)と機能的に近いが、Unreal は「ループ再生用の帯」と「実際の再生範囲」が同じ概念に統合されている点が Motolii(ループ帯とプロジェクト全体時間は別概念)と異なる——**対象外**(Motolii は既にループ帯を持ち再定義不要)。Selection Range(範囲内一括選択)は3-3の Ctrl+]/[ と同系の**抜け候補**として記録(範囲選択→一括操作という語彙)。

### 3-7 Track Filters: Mute / Solo / Pin / Isolate
- 右クリックで Mute(非表示・評価除外)/ Solo(他を全ミュートして単独可視化)。Pin は1シーケンスにつき1トラックのみ。Alt+W 非表示・Alt+Q 分離表示・Ctrl+Alt+W 再表示・Ctrl+Alt+Q 分離解除
- 出典: sequencer-track-list、unrealdirective.com(ショートカット表、非公式だが Epic 公式ドキュメントのホットキー一覧と一致するため参考採用)
- **判定: 対象外**。M/S/L(正典 §6)は Motolii の Mute/Solo/Lock で概念としては既載だが、Unreal の Solo は「他を全ミュートする一時プレビュー状態」でありトラック単位の恒久フラグではない。Motolii の M/S/L は Document 上の恒久状態(§6「状態は Document から読む」)と定義されており、Unreal 型の一時 Solo プレビューは**性質が異なる別機能**——採用するなら新規に裁定が要る話であり、今回の既決を上書きしない。Pin(1シーケンス1トラックのみ)は Motolii に対応概念がなく**対象外**(用途がUnreal特有のシーケンス切替ワークフロー)。

### 3-8 Blend curve のイージング変更(カーブにホバー→黄変→右クリック)
- 出典: (WebSearch要約、Epic公式フォーラム言及+ toolbar reference の Cubic 既定)
- **判定: 対象外**。1-4 と同じ理由(クリップ間 blend という概念自体を Motolii が採らない)。

---

## 4. Spine(esotericsoftware.com/spine-user-guide)

### 4-1 Dopesheet: 選択・移動・複製・削除
- 単独クリック選択 / Ctrl(Cmd)クリックでトグル / Ctrl+A で行→全体と段階的に全選択
- ドラッグでキー移動。**Ctrl+Shift を押しながらドラッグ開始で複製**
- 削除: ダブルクリック(正典の「ダブルクリック不使用」原則と真逆の割当)
- スナップ無効化: **Shift 押下**(=Motolii と真逆の極性——Motolii は Alt でスナップ無効、Shift は範囲選択用)
- 出典: http://en.esotericsoftware.com/spine-dopesheet
- **判定**: 移動/選択は**既載**(正典 §3)。「ダブルクリックで削除」は拘束4があるため**対象外**(正典は「削除は Delete キー、キー選択が層選択より優先」で既に定義済み・ダブルクリック語彙を使わない)。**Shift=スナップ無効の極性はメモとして記録**——正典 §7-3 は Alt の役割整理を未決としており、Spine の Shift 極性は「Shift をスナップ無効に使う社もある」という反例(LottieFiles=リタイムにShift、正典はリタイムを裁定146でCmd確定済みなので直接の抵触ではないが、Shift の別用途競合可能性として §7-3 の検討材料に追加)。

### 4-2 Box select による**スケール**(端をドラッグで区間圧縮/伸長、逆転可)
- 空白ドラッグで矩形選択→**選択矩形の端をドラッグすると選択キー群の時間間隔が比例伸縮**。左端を右端より右へ持っていくと**キー順序が反転**する
- 出典: http://en.esotericsoftware.com/spine-dopesheet
- **判定: 抜け(重要)**。正典 §3 の裁定146 RetimeSelection(Cmd+ドラッグで範囲端リタイム)と同種の機能だが、Spine は「矩形選択の可視ハンドル」という**専用UIオブジェクト**を経由し、かつ**順序反転(逆再生化)という正典が触れていない挙動**を明示的にサポートしている。正典裁定146はリタイムの起動方法(Cmd+ドラッグ)は確定しているが、**縮めきって0を超えたらどうなるか(反転か・クランプか)を規定していない**——これは正典に追加すべき未決点。§7 に「RetimeSelectionが0または反転をまたぐ時の挙動」を追記する価値がある。

### 4-3 Offset Mode(ループ用ラップ)/ Shift Mode(後続キーを道連れ)
- **Offset Mode**: 有効時、キーをアニメーション終端より先にドラッグすると**アニメーション長で折り返して(wrap)**そのまま保持——ループアニメ制作用
- **Shift Mode**: 有効時、1キーの移動が**後続の全キーを同じ量だけ引き連れて動かす**(=キー単位のripple)
- 出典: http://en.esotericsoftware.com/spine-keys (要約)
- **判定**:
  - Offset Mode(ループ wrap)は**抜け候補**。Motolii は「clip の 0秒/終端で塊が止まる」(§2 move の塊制約)というクランプ思想を持つが、これは clip 移動の話であり、**キーフレームが loop 境界で折り返す**という概念は正典のどこにも無い。Motolii が「クリップ内ループ」を将来持つ場合(1-3で対象外にした Unity の Loop 関連機能と地続き)、このラップ挙動は同時に要検討になる——現時点では該当データモデルがないため**保留**(データモデル未定のため抜けと言い切れない)。
  - Shift Mode(キー単位ripple)は正典拘束1(trim familyの不採用理由=「詰め保証を持たない」という設計)を**キーフレーム粒度に拡張した時にどうなるか**という論点。clip の trim family は明確に不採用だが、単一 param 曲線内でのキー ripple はまた別次元の判断が要る(たとえば「後続キーとの間隔を保ったままテンポを詰める」操作はMV編集で需要がありうる)。**保留**として §7 へ追記候補に。

### 4-4 Graph view: 曲線タイプ(Stepped/Linear/Bezier)+ハンドルプリセット(Automatic/Separate/Flat/Bounce/Ease out/Ease in)
- 縦スナップ(フレーム位置維持)と横スナップ(値維持)を分離、他キー値へのスナップあり、**Shiftでスナップ無効**(4-1と同じ極性)
- 出典: http://en.esotericsoftware.com/spine-graph
- **判定: 既載寄り**(正典 §3 イージング・§7-1/§7-4 の検討材料)。「Bounce」プリセットの存在は Rive の Cubic Value(overshoot可)相当への具体的な参照になり、§7-4 の裁定素材として有用。「縦スナップ/横スナップの分離」自体は Motolii がカーブエディタ画面を持たない前提のため直接対応なし——**対象外**(2-1と同じ理由、専用カーブ画面という前提が違う)。

### 4-5 Tree/Dopesheet の Filter(bones/slots/attachments の種別フィルタ、赤ボタン=フィルタ有効の視覚、右クリックでフィルタON/OFF切替)
- **重要な否定的知見**: Spine のフィルタは「要素の**種別**(ボーン/スロット/アタッチメント)」で絞るものであり、Unity の Show: Animated のような「**キーを持つ行だけ**」という絞り込みではない(ドキュメント上、そのものずばりの「show only animated」相当語は見当たらなかった)
- 出典: http://esotericsoftware.com/spine-tree , http://en.esotericsoftware.com/spine-dopesheet
- **判定: 対象外(参考データとして記録)**。正典 §5 候「Show Only Animated」の裏付けとしては**弱い**——3社中はっきり同義の機能を持つのは Unity のみ(2-6)。Spine のフィルタは種別軸で直交する別機能であり、混同して正典へ書き込まないよう注意。判定としては「Spineはこの点で参考にならない」という否定的事実の記録。

### 4-6 Key ボタンの色状態(緑=キーなし/オレンジ=変更未キー/赤=キー済み)
- 出典: http://en.esotericsoftware.com/spine-keys
- **判定: 保留**。正典 §5.5「カーソル形状は意味の予告」と同系の「状態を色で予告する」思想だが、Spine のこれは**プロパティごとの永続的な色状態インジケータ**であり、Motolii の cursor 系(操作中のみ)とは別の常時表示 UI。パラメータパネル(Motolii の◇◆キー描画)に近い概念で、正典 §1.5/§3 の◇◆キー菱形の描画意味論を拡張する検討材料になりうるが、現時点で正典に対応箇所がないため保留。

---

## まとめ表(件数)

| 分類 | 件数 |
|---|---|
| 既載(補強含む) | 13 |
| 抜け | 6 |
| 対象外 | 12 |
| 保留 | 6 |

## 抜け 上位(優先度順・簡略)

1. **playhead へ trim**(Unity「Editing > Trim Start/End」・Unreal「Alt+]/Alt+[」)— 2社が独立に持つ。分割(Cmd+K)しか playhead 起点編集を持たない正典への追加候補。
2. **RetimeSelection が反転/0点を跨ぐ時の挙動未規定**(Spine の box-select端ドラッグで判明)— 裁定146の実装細部として §7 に追記価値あり。
3. **playhead を境に片側キー全選択**(Unreal Ctrl+]/Ctrl+[)— 正典 §3 選択(単独/Cmd/Shift)に無い粒度。テンポ変更時の一括リタイムに効きうる。
4. **Selection Range による範囲内一括選択**(Unreal)— 3と同系、範囲指定→操作対象確定という語彙自体が正典に薄い。
5. **次/前のキーへ playhead ジャンプ**(Unity Alt+,/.)— 優先度低いが矢印キー(§5)の隣接候補。
6. (保留寄りだが記録)**キー単位ripple(Spine Shift Mode)/ループ wrap(Spine Offset Mode)** — クリップ内ループという未定のデータモデルに依存するため確定抜けとまでは言えないが、将来のループ機能裁定時に必ず再訪すべき論点。

## blend/crossfade の写像(タスク指定論点への回答)

Unity(Mix モードの重なり自動 blend)・Unreal(セクション重なりの crossfade、blend curve handle)ともに、「クリップが時間的に重なる」ことに**自動的な合成処理の起動**という意味を持たせている。Motolii は拘束1で「自由な絶対時間配置・重なり自由」を既決にしているが、これは「詰め保証(ripple/roll等)を持たない」という trim family の否定であり、blend とは別の設計論点である。今回の調査で切り分けが明確になった: **Motolii では重なりはただの視覚的重なりであり、自動合成の意味を持たない**(対象外)。理由は次の2点——(a) Unity/Unreal の blend は「モード」または「専用ハンドル」という状態・UIオブジェクトを介した副作用であり、正典が目指す「1操作=1意味・モードを増やさない」設計(拘束6のアクション直接主義とも整合)と相性が悪い。(b) 合成(opacity/blend mode)は本来 compositing パラメータの仕事であり、それを「クリップの重なり」という配置操作に暗黙で結びつけると、trim/move という時間操作とレンダリング設定が結合してしまい、正典 §1.5(面の分業)の思想に反する。ただし単一クリップの ease-in/out(1-4b、Ctrl+drag)は「他クリップとの重なりを前提としない自クリップの減衰」であり、これは blend とは独立した論点として保留に格納した。
