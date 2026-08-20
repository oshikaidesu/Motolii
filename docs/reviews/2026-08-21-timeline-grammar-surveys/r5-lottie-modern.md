# R5: Lottie圏優先・現代モーションエディタ timeline操作文法 採集

裁定145対応。前提: AE型自由配置・trim family不採用。Ravel除外。Lottie圏公式エディタ最優先。

対象: LottieFiles Creator / Lottielab / Rive / Cavalry / Jitter / Fable(閉鎖済み・史料として) / Linearity Move(発見物・Fable後継として言及される)

---

## 1. LottieFiles Creator (lottiefiles.com/lottie-creator)

出典:
- https://docs.lottiefiles.com/en/creator/01_introduction/key-features
- https://docs.lottiefiles.com/en/creator/07_animation/timeline
- https://docs.lottiefiles.com/en/creator/07_animation/keyframes
- https://docs.lottiefiles.com/en/creator/07_animation/time-stretch
- https://docs.lottiefiles.com/en/creator/07_animation/easing
- https://docs.lottiefiles.com/en/creator/07_animation/graph-editor
- https://docs.lottiefiles.com/en/creator/07_animation/animated-properties
- https://docs.lottiefiles.com/en/creator/07_animation/advanced-duplicator

### 総評
- timelineは **レイヤーバー型+track型のハイブリッド**: 左に36px幅のレイヤーサイドバー(シーン管理・segment・visibility・lock)、右にkeyframe panel(レイヤー行×アニメートされたプロパティ行、キーフレームdiamond表示)。上部に秒+フレーム目盛のtime scale。
- **「最初から揃っている」度は6エディタ中もっとも高い**: keyframe diamond種別による4アイコン表現、Keyframe Assistant、Keyframe Optimizer、Advanced Graph Editor、Advanced Duplicator(6パターン+stagger)まで標準搭載。AEのプロ機能をほぼそのまま踏襲しつつブラウザネイティブに再実装している。
- キーフレームは**プロパティ単位**(P/S/R/Tショートカット)。グループ単位の集約キーフレームは今回の情報からは確認できず(Lottielabは明示的に廃止、Creatorは個別プロパティ表示が基本)。

### 操作採集

**1-1. タイムラインのズーム/パン**
- 起動条件: Cmd/Ctrl+`+`/`-`、time scale上でのスクロールホイール、トラックパッドpinch、zoomスライダー両端ドラッグ
- ドラッグ中の意味: ズームは常にplayhead位置を中心に行われる(フォーカス維持)
- 確定/キャンセル: 即時反映、モードレス
- フィードバック: time scaleの目盛(tick)がズームレベルに応じて動的に変化

**1-2. パン**
- 起動条件: keyframe panel下のスクロールバードラッグ、2本指トラックパッドスワイプ、middle-click+drag、zoomスライダー中央ドラッグ
- フィードバック: 水平スクロール

**1-3. ワークエリア(再生範囲)設定**
- 起動条件: work area開始/終了の可動スライダー、Cmd/Ctrl+Opt/Alt+`[`/`]`、Alt+Drag on time scale
- ドラッグ中の意味: 特定範囲を「フォーカス再生・書き出し範囲」として切り出す
- フィードバック: 右クリックメニューで名前付きセグメント作成・シーンtrim・境界reset

**1-4. playheadスクラブ**
- 起動条件: time scaleクリックでジャンプ、playheadハンドルドラッグ、Home/End/PageUp/PageDown(+Shiftで10フレーム単位)
- ドラッグ中の意味: **Shift+Drag playhead → 可視キーフレームへスナップ**(閾値の明記なし、「visible keyframes」への吸着)
- フィードバック: オレンジの垂直線

**1-5. キーフレーム作成**
- 起動条件: キーボードショートカット P(Position)/S(Scale)/R(Rotation)/T(Opacity)/K(全プロパティをモーダルで)、property panel右サイドバーのキーフレームボタン、timeline左サイドバーのコントロール
- フィードバック: 初回キーフレーム追加時「現在値が保持される」形でプロパティがアニメート対象化

**1-6. キーフレームUI形状(4種)**
- Arrow Right = イージングinのみ(開始キーフレーム)
- Arrow Left = イージングoutのみ
- Diamond = 標準(イージングなし)
- Circle = イージングin/out両方
→ **形状そのものがイージング状態を語る**、AEの単一ダイヤモンド表示より情報密度が高い。

**1-7. 選択**
- 単一: diamondクリック
- 複数: Shift+Click(プロパティ/レイヤー横断で追加)、マーキー(矩形ドラッグ)選択、Cmd/Ctrl+A(選択レイヤー内の全キーフレーム)

**1-8. 移動**
- 起動条件: diamondをクリック保持しドラッグ
- ドラッグ中の意味: フレームグリッドにスナップ。複数選択時は全選択キーフレームが相対タイミングを保ったまま一緒に移動

**1-9. リタイム(キー群の時間伸縮)**
- 起動条件: 複数キーフレーム選択後、選択範囲の最初/最後のキーフレームをShift+Dragドラッグ
- ドラッグ中の意味: **選択範囲内の全キーフレームが比例スケール**(いわゆるtime stretchのキー版)

**1-10. コピー&ペースト・複製**
- Cmd/Ctrl+C / Cmd/Ctrl+V、右クリックメニュー
- **Alt/Option+Drag = 元を残したまま複製**(ドラッグ量がそのまま複製先のオフセット)

**1-11. Keyframe Assistant**
- 起動条件: 2つ以上のキーフレーム選択後、右クリック
- 提供操作: リタイム、均等配置(even distribution)、ミラー/リバース

**1-12. Keyframe Optimizer**
- 冗長キーフレームをアニメーション形状を保ったまま削減。許容誤差スライダー+リアルタイムプレビュー付きで削除前に確認できる

**1-13. 削除**
- Delete、または右クリック→削除。単一・複数選択とも即座に削除

**1-14. Time Stretch(シーン単位)**
- 起動条件: canvas or timelineでネストされたシーンを選択
- 操作: 伸縮率を%で入力(100%=元の長さ、超過で延長、未満で短縮)。**シーン内の全キーフレームが比例的に再スケールされ相対タイミング関係は保持**
- Hold in Place(アンカー): Layer In-point(既定)/ Current Frame / Layer Out-point の3択で「固定する基準点」を選べる
- 注記: 個別キーフレーム範囲でなく**ネストシーン単位**の操作である点が1-9(選択範囲のリタイム)と役割分担している

**1-15. イージング(per-keyframe in/outタンジェント)**
- 種別: Linear(角ばった/一定速度)、Smooth=Bezier(丸みを帯びた/可変速度)
- キーフレームUIが選択されるとproperty panelにcubic-bezier数値(x1,y1,x2,y2)+「Edit Easing」ボタンが出現、bezierエディタが開く
- **in/outタンジェントは個別トグルで独立制御**(片方だけlinear、片方だけbezierも可)

**1-16. Advanced Graph Editor**
- フレームレベルでbezierハンドルを直接ドラッグしカーブ編集
- **複数プロパティ(position/scale/rotationなど)のカーブを同時に重ねて表示**(単一プロパティに閉じない)
- ネイティブブラウザ機能として外部ツール不要

**1-17. Animated Properties パネル**
- アニメート中の全プロパティ(Position/Scale/Rotation/Opacity/Skew/Fill Color/Stroke Color/Stroke Width/Trim Pathほか)を一覧表示。timelineとの直接連動の詳細は文書からは未確認

**1-18. Advanced Duplicator**
- 起動条件: 右クリック、Shift+Cmd/Ctrl+D、サイドバーアイコン、Editメニュー
- パターン: Linear/Grid/Circular/Spiral/Confetti/Random の6種
- transform(scale/rotation/opacity/position)の**段階的補間**(Linear/Ease In-Out選択可)
- **Animation offset**: 複製ごとにフレーム遅延をつけてstagger演出(方向: Forward/Backward/Center Out/Random)。**アニメーションタイミングと複製パターンが統合されている**点がAEのduplicate+manual offsetより一段モダン

---

## 2. Lottielab (lottielab.com)

出典:
- https://docs.lottielab.com/editor/animating/timeline
- https://docs.lottielab.com/editor/animating/timeline/keyframes
- https://docs.lottielab.com/editor/animating/timeline/keyframes/add-a-keyframe
- https://docs.lottielab.com/editor/animating/timeline/keyframes/duplicate-keyframes
- https://docs.lottielab.com/editor/animating/timeline/keyframes/keyframe-thumbnails
- https://docs.lottielab.com/editor/animating/timeline/transition-bar
- https://docs.lottielab.com/editor/animating/timeline/layers-and-properties
- https://docs.lottielab.com/editor/animating/timeline/playhead-and-controls
- https://docs.lottielab.com/editor/animating/timeline/duration-and-playback
- https://docs.lottielab.com/editor/animating/easing
- https://docs.lottielab.com/editor/animating/easing/add-easing
- https://docs.lottielab.com/editor/animating/easing/custom-easing (未公開・準備中)
- https://docs.lottielab.com/editor/animating/easing/easing-presets
- https://docs.lottielab.com/editor/canvas/layer-controls-huds/motion-path
- https://docs.lottielab.com/editor/organising-layers/layer-actions/select-multi-select-layers

### 総評
- timelineは**レイヤー行の中にプロパティ行がネストする階層disclosure構造**。「未アニメートのレイヤーはtimelineに表示しない」がデフォルトで、常に「アニメーションされているものだけ見える」引き算的UI。
- 明示的に**グループ/レイヤーレベルの集約キーフレーム表示を廃止**した経緯が見える("Aggregated keyframes are no longer visible on a layer/group level anymore")。プロパティ個別編集に一本化。
- 「最初から揃っている」度はLottieFiles Creatorよりやや軽量。Graph Editor/Custom Easingは**執筆時点でまだ準備中("check this page again soon")**——プリセットイージング止まりで、bezierハンドルを直接操作するUIはまだ文書化されていない(Add EasingページはTransition bar選択→右パネルでの数値/プリセット操作を示唆するのみ)。
- **Motion Path(canvas直接編集)**が特徴的: timelineでなくcanvas上でオブジェクトの移動軌跡を直線的に見せ、そのパス自体をハンドルでカーブさせられる。AEのモーションパス(位置キーフレーム連結線)をより一等市民として押し出した設計。

### 操作採集

**2-1. キーフレーム作成**
- 前提: Animate mode(エディタ上部中央のトグル)である必要がある
- 方法A(auto-animate、既定): playheadをキーフレーム未存在地点に置き、プロパティを変更 → 自動でその時刻にキーフレーム生成
- 方法B(+ボタン): playheadをドラッグして未存在地点に来ると playhead上に`+`ボタンが出現 → クリックでプロパティ変更なしにキーフレーム設置
- フィードバック: `+`ボタンの出現がドラッグ位置=「キーフレームなし」の可視サイン

**2-2. キーフレーム複製**
- 起動条件: **⌥Option+Drag(Mac) / Alt+Drag(Win)** on keyframe
- ドラッグ中の意味: ドラッグ量がそのまま複製先の時間オフセットになる(推定、LottieFiles Creatorと同型)

**2-3. Transition bar(2キーフレーム間の区間)**
- 定義: 「レイヤーが1つのキーフレームから次へ"遷移"する区間」を指す帯
- ドラッグ: 帯自体をドラッグすると紐づくキーフレームセットごと移動
- クリック: 帯をクリックすると右パネルにイージングオプションが出現(粒度の細かい制御への入口)

**2-4. Layers and Properties(階層表示)**
- 未アニメートレイヤーはtimelineに出現しない(明示選択時のみ表示)
- 矢印クリックでネストしたレイヤー/プロパティを展開(標準的なdisclosure tree)

**2-5. イージングプリセット**
- Linear(一定速度)/ Natural(摩擦・重力を模した加減速)/ Slow down(速→減速)/ Accelerate(遅→加速)/ Bounce in / Bounce out の6種
- 適用方法の詳細UIは文書未記載(プリセット一覧の存在のみ確認)

**2-6. Add Easing(操作エントリポイント)**
- 方法A: Transition bar(2キーフレーム間の区間)を選択 → 右パネルにイージングプロパティ表示
- 方法B: Motion path segment を選択 → 右パネルでイージング調整
- **timeline経由とcanvas(motion path)経由の2つの入り口が用意されている**のが特徴

**2-7. Custom Easing / Speed Graph**
- 執筆時点で**未リリース・ドキュメント準備中**("We're currently working on releasing custom easing")。Anticipation/Overshootページも同様にプレースホルダー。→ グラフエディタ相当機能はLottielabではまだ発展途上と判断できる

**2-8. Motion Path(canvas上の軌跡編集)**
- キーフレーム化された位置間の移動軌跡をcanvas上に線として表示
- ハンドル操作で直接カーブを付けられる(Create a motion path / Curving a motion path の専用ページあり)

**2-9. 複数選択(レイヤー)**
- Mac: Cmd+Select、Win: Ctrl+Select(個別追加選択)
- Shift+Select: 範囲選択(間の全レイヤー)
- Canvas上: Shift/Cmd/Ctrlは同じ機能(複数選択への追加)。ドラッグでラバーバンド選択も可

**2-10. Playhead & Controls**
- Timeline ruler上でクリック/ドラッグしてスクラブ。ruler自体が精密な目盛を提供

**2-11. Duration & Playback**
- Duration表示(再生ボタン脇)をクリックして直接時間値を入力
- 再生/一時停止はtimeline左上の再生ボタン

**2-12. Keyframe Thumbnails**
- 各キーフレーム時点でのレイヤーのスナップショットをプレビュー表示
- 右クリックで表示/非表示切替、または**Shift+T**ショートカット
- 注: LottieFiles Creator側では「Keyframe Thumbnailsは恒久的に無効化された」という変更履歴があった(視覚的ノイズ削減のため) — 同じ機能をLottielabは維持、Creatorは廃止、という設計判断の分岐が確認できる

---

## 3. Rive (rive.app)

出典:
- https://rive.app/docs/editor/animate-mode/timeline
- https://rive.app/docs/editor/animate-mode/keys
- https://rive.app/docs/editor/animate-mode/interpolation-easing
- https://rive.app/docs/editor/fundamentals/design-vs-animate-mode

### 総評
- timelineは**アニメーション単位の一覧(左)+その中のkeyed object行**という構造。トラック型に近いが、Rive独自の**State Machine(状態遷移の論理レイヤー)がTimelineの外側にもう一段ある**のが最大の差異——「Timelineは値とキーフレームを持つ完全にステートレスなコンテナ」、「State Machineはtimelineをラップして実行フロー・ブレンド・条件分岐を制御する論理監督者」という二層構造。
- **Design mode / Animate modeの明示分離**がAEとの決定的違い。AEは常時ライブ(timelineがアクティブなら編集は即キーフレーム化されうる)なのに対し、Riveは「構造変更(Design mode)」と「振る舞い変更(Animate mode)」を意図的にモード分離し、**Animate modeでもState Machineが選択されている間は自動キー化されない**という安全策を持つ。「AEの自動キー化という核メカニズムは残しつつ、事故防止のガードレールを追加した」のがRiveの立ち位置。
- キーフレームは「Key」と呼称。**grey key(グループ化された複数プロパティの集約)とblue key(個別プロパティ)の2階層**を持ち、折りたたみ時はgrey keyが代表する。LottieFiles Creatorの「常にプロパティ単位」路線とは逆に、AEの「まとめて動くグループプロパティ」という発想をRiveは残している。

### 操作採集

**3-1. Timelineの構成**
- 左側にアニメーション一覧、各アニメーションにOne-Shot(終端で停止)/ Ping-Pong(往復ループ)/ Loop(先頭に戻ってループ)の3種の再生タイプ
- Work Areaで再生範囲を絞れる
- **Show Only Selected**トグル: 選択中のオブジェクトがキー化された行だけに絞り込み表示

**3-2. ナビゲーション**
- 水平スクロールバーでのズーム、右クリックドラッグ or スペースバードラッグでパン
- 再生速度は負の値も可(逆再生)。key snap間隔の表示設定あり

**3-3. Key作成(3方法)**
- Stage上で直接オブジェクトを変形(position/rotation/scale)→自動でkey生成
- Inspectorでプロパティ変更→key生成
- プロパティ横の専用キーボタンをクリック→現在値でkey設置
- **キーボタンの3状態フィードバック**: グレー枠=キーなし、青枠=そのプロパティはアニメート中(ただし現在フレームにキーなし)、青塗り=現在のplayhead位置にキーあり

**3-4. Key表示の階層(grey/blue)**
- blue塗り = 個別プロパティのkey
- grey key = プロパティが折りたたまれているときの集約表示(そのオブジェクトの全キー化済み属性をまとめて1本の行として見せる)

**3-5. Keyの移動**
- クリックして保持しドラッグでタイミング調整
- grey keyをドラッグ = オブジェクトの全プロパティが一緒に移動
- blue keyをドラッグ = 個別プロパティのみ移動

**3-6. Keyのリサイズ(範囲伸縮)**
- 起動条件: 複数key選択範囲の始端/終端を**Alt+Drag**
- 意味: アニメーション全体の長さを伸長/圧縮(LottieFiles CreatorのShift+Drag retimeと同系統の操作だが修飾キーが異なる=エディタ間で統一された規約が存在しない)

**3-7. コピー&ペースト**
- 標準のコピペ操作でプロパティキーをオブジェクト間に転写可能

**3-8. Data Binding**
- キーを外部データソースに紐付け、固定値でなくデータ駆動で値を決定できる(モーションGr系エディタとしては珍しい機能)

**3-9. イージング/補間の4種**
- **Linear**(既定・一定速度)
- **Cubic**: カーブ補間、ドラッグ可能な2ハンドル、始終端が滑らかに減速/加速(既定形状)
- **Cubic Value**: AEのbezierシステムに相当。Graph Editor上で直接ハンドル操作。**overshoot可能**(始終端値が同一でなくても、値がキー値を超えて戻る=バウンスや予備動作を作れる)
- **Hold**: 補間せず次keyまで現在値を保持(AEのhold keyframeと同じ)

**3-10. Interpolation Panel**
- timeline上でkey選択時に出現。x軸=時間、y軸=プロパティ変化のグラフを表示。4つの数値(0-1想定)でハンドル位置を手入力可能

**3-11. Graph Editor**
- timeline近くのボタンでトグル、**timeline表示を置き換える形で出現**(併存ではなく切り替え)
- 選択中のオブジェクトのみ表示される(全体表示ではなく焦点を絞る設計)
- Cubic Valueのハンドルをここで直接ドラッグして細かく調整

---

## 4. Cavalry (cavalry.scenegroup.co)

出典:
- https://cavalry.studio/docs/user-interface/menus/window-menu/scene-window/time-editor/
- https://cavalry.studio/docs/user-interface/menus/window-menu/scene-window/graph-editor/
- https://cavalry.studio/docs/user-interface/menus/window-menu/scene-window/keyframe-layers/
- https://cavalry.studio/docs/user-interface/menus/window-menu/scene-window/timeline/
- https://cavalry.studio/docs/user-interface/menus/animation-menu/
- Magic Easingプリセット一覧: https://note.com/tourmalinism/n/n519a9f95985c (二次資料、プリセット名列挙のみ参照)

### 総評
- 6エディタ中もっとも**プロ向け・AE後継色が強い**。Time Editor(タイムライン型キーフレーム編集)とGraph Editor(カーブ編集)が明確に分離しており、AEの操作文法をほぼ継承しつつ独自拡張を足す方針。
- **Keybars**という独自概念: 2つのキーフレームを結ぶ線自体を選択対象にでき、「範囲選択せずにカーブ全体を移動」できる。AEにはない中間UIオブジェクト。
- **Keyframe Layers**は他5エディタに存在しない独自機能: 複数のキーフレームセット/アニメーションカーブを**strength(0-100)とNormal/Overwriteブレンドモード**で重ね合わせ、走る⇄歩くのような**状態間のブレンド**をコピペなしで実現する。AEの「グループ化されたプロパティ」とは似て非なる——こちらは独立したアニメーショントラックの合成であり、Riveのstate machine的な発想に近い(ただしtimeline側の機能として実装されている点が異なる)。
- **Magic Easing**: 右クリック一発でSlowIn/SlowOut/VerySlow系/Spring系/Anticipate/Overshoot/Bounce等のプリセットを適用できる、数式駆動のイージング体系。カスタム式(Edit Custom Expression)にも対応。

### 操作採集

**4-1. Timeline window**
- playhead(teal色)をクリック+ドラッグでスクラブ、timeline上の任意点クリックでplayheadジャンプ
- **Shift+Drag = 10フレーム単位 or Time Markerへスナップ**
- Playback Range(work area)の両端"teal bookends"をクリック+ドラッグで範囲設定、範囲全体はバードラッグで移動可
- キーボード: Cmd/Ctrl+矢印=1フレーム、Shift+Cmd/Ctrl+矢印=5フレーム、Option/Alt+Cmd/Ctrl+矢印=次/前のキーフレームへジャンプ
- **Qキー押しながらどのウィンドウからでもドラッグでスクラブ可能**(ワークフロー全体を跨ぐ操作)

**4-2. Time Editor: Clips**
- レイヤーが画面に存在する時間範囲を示す帯。端をクリック+ドラッグしてViewportに出現するフレーム範囲を調整
- 右クリックメニューでsplit/merge/extend

**4-3. Time Editor: Keyframes**
- 作成・編集・移動・削除・色分け可能
- **ダブルクリックでインラインpopupが開き数値を直接編集**

**4-4. Time Editor: Keybars**
- キーフレームペアを結ぶ線。**「範囲選択せずにアニメーションカーブ全体を移動できる」**独自UI要素
- 実線=値の変化あり、破線=静止値(変化なし)を意味する(状態の可視化)

**4-5. 選択ロジック**
- マーキー選択は「キーフレームが存在すればキーフレーム優先、なければKeybar選択」という優先順位ルール
- Option/Alt修飾でクリップのみに選択対象をフィルタ

**4-6. 複数選択・移動**
- Cmd/Ctrl+Click またはマーキーで複数キーフレーム選択、属性・レイヤー横断でまとめてドラッグ移動

**4-7. 複製・変形**
- Option/Alt+Click+Drag = 複製
- Transform tool: 選択範囲を**緑ハンドルでスケール**(時間軸方向のリタイムに相当)

**4-8. スナップ**
- キーフレーム、クリップ端、time markerへのスナップ。**Pacing markers**(拍・秒単位のガイド)も用意

**4-9. キーフレームアラインメント**
- 選択キーフレームをleft/center/rightで整列(Animation menuから)

**4-10. Graph Editor: 補間状態**
- Linear(直線)/ Bezier(カーブ+可動ハンドル)/ Step(階段状)の3種
- **Option/Alt+Click = linear⇔bezier変換**
- **Option/Alt+Click+Drag on linear keyframe = その場でbezier化**
- **Shift+Option/Alt+Drag = 連結タンジェントの重み(weight)調整**
- **X+Click = ハンドル折りたたみ、X+Click+Drag = 復元**
- **Option/Alt+Click+Drag = joined⇔brokenハンドルの切替**

**4-11. Graph Editorナビゲーション**
- ズーム: スクロールホイール / Option+Alt+スクロールで水平ズーム / Shift+スクロールで垂直ズーム
- パン: Space+Drag
- 移動制約: Shift+Dragでキーフレーム/ハンドルの動きを軸拘束
- Fキー = 選択範囲にフレーミング(フォーカス)

**4-12. Magic Easing**
- 右クリック→Magic Easing→プリセット選択(SlowIn/SlowOut/SlowInSlowOut/VerySlowIn/VerySlowOut/VerySlowInVerySlowOut/SpringIn/SpringOut/SpringInSpringOut/SmallSpringIn/SmallSpringOut/SmallSpringInSmallSpringOut/AnticipateIn/OvershootOut/AnticipateInOvershootOut/BounceIn/BounceOut/BounceInBounceOut)
- Edit Custom Expression...でカスタム数式イージングも定義可能

**4-13. Keyframe Layers**
- 複数のアニメーションカーブ/キーフレームセットを層として重ね、**Strength(0-100、アニメート可能)**でブレンド強度を制御
- ブレンドモード: Normal(加算的)/ Overwrite
- 用途例: 「走る」と「歩く」のアニメーションをStrengthのアニメーションだけで滑らかに切り替え(コピペや手動キー再構築が不要)
- Scene Tree上に読み取り専用の合成値表示あり

**4-14. Animation menu 総括**
- Keyframe管理(position/rotation/scale/all transforms一括設定)
- Next/Previous Keyframeジャンプ、nudge(前後シフト)
- クリップのin/out点調整・現在フレームへの移動
- カーブのreverse、キーフレーム削除
- **Bake**: プロシージャルアニメーションをキーフレームアニメーションへ変換

---

## 5. Jitter (jitter.video)

出典:
- https://help.jitter.video/en/articles/14111802-work-with-the-timeline
- https://help.jitter.video/en/articles/14136797-create-advanced-animations
- https://help.jitter.video/en/articles/12089209-what-is-jitter

### 総評
- 6エディタの中で**もっともAEから遠い**——「キーフレーム」という語彙・UIをそもそも前面に出さず、**「アクション(Move/Scale/Rotate/Opacity/Color/Shadow/Layer Blur/Background Blur/Glass/Hide-Show/Resize/Corner Radius/Stroke、テキスト用にChange Text/Letter Spacing/Line Height/Counter、シェイプ用にMorph/Arc/Star)」という意味単位のブロック**をtimeline上に並べる設計。
- timelineはlayer-property行の集合ではなく、**「1アニメーション(またはグループ)=1セグメント」の帯**として表示される。AEのレイヤーバー型に近いが、プロパティ粒度まで降りない(ズーム段階が違う)。
- **Stagger(複数アクションへ均等な時間差を一括付与)**が右クリック一発で使えるのが特徴——AEでは手作業(expressionかnudgeの繰り返し)が必要な操作を一級の名前付き操作として持つ。
- イージングは「グラフのハンドルをドラッグ、または下の数値を編集」で調整可能、Move用にBezierパスの直接編集も追加された(2025年3月changelog系機能)。プリセット: Slow down / Accelerate / Elastic / Bounce / Overshoot ほか、強度(intensity)ノブつき。

### 操作採集

**5-1. Timeline構造**
- artboard選択+Animateタブを開くとエディタ下部に出現
- **1アニメーション or 1グループ = 1セグメント**として表示、操作の並び順・関係性が一目でわかる設計

**5-2. セグメントの移動**
- クリック+ドラッグで左右移動(開始/終了タイミング調整)
- ←/→キーでミリ秒単位の微調整

**5-3. Stagger(段差付け)**
- 起動条件: Ctrl/Cmd+Clickで複数アニメーションを選択 → 右クリック → Stagger
- 効果: 選択範囲内の各アニメーション開始時刻に**均一な時間差**を一括設定

**5-4. Duration調整**
- セグメント端にホバーしてドラッグ、伸ばすと遅く/縮めると速く見える

**5-5. グルーピング**
- 複数アニメーション選択 → Ctrl/Cmd+G またはコンテキストメニュー

**5-6. Scene Duration(全体尺)**
- グレー領域端が書き出し尺の境界を表す。ドラッグで最終尺を設定。境界外のアニメーションは書き出しに含まれない

**5-7. ズーム**
- Ctrl/Cmd+スクロール、またはトラックパッドジェスチャ

**5-8. アクション(Actions)モデル**
- AEの「プロパティ+キーフレーム」でなく、**「レイヤーに何をさせるか」を宣言する意味単位ブロック**が基本単位
- 汎用: Move/Scale/Rotate/Opacity/Color/Shadow/Layer Blur/Background Blur/Glass/Hide-Show/Resize/Corner Radius/Stroke
- テキスト専用: Change Text/Letter Spacing/Line Height/Counter
- シェイプ専用: Morph/Arc/Star

**5-9. イージング**
- グラフ上のハンドルをドラッグ、または下部の数値入力で調整
- 「左ハンドル=開始の加減速を制御、平らなほど滑らか・急なほど唐突な始まり」という言語化がされている
- プリセット: Slow down / Accelerate / Elastic / Bounce / Overshoot ほか + 強度(intensity)コントロール

**5-10. Move用Bezierパス編集**
- Move actionに対して、パス自体をBezierで直接編集できる機能を追加(precise Bezier controls for the path)

---

## 6. Fable (fable.app) — 2024年11月に事業終了・史料として記録

出典:
- https://amxmln.com/blog/2023/animating-with-fable/ (著者による実機レビュー)
- https://amxmln.com/blog/2024/goodbye-fable/ (終了後の追悼記事)
- https://news.ycombinator.com/item?id=41850573 (終了告知の反応)
- https://www.linearity.io/blog/linearity-move-the-alternative-to-the-post-fable-world-of-motion-graphics/ (後継文脈)

### 総評
- 「Figma for motion」を標榜したブラウザベースの共同編集モーションツール。**2024年11月に会社ごとwind down**——「AIの発展がソフトウェアという営みそのものの前提を揺るがした」という理由を公表して解散(閉鎖理由自体がAI時代のツール戦略を考える上で参考になる)。
- 公式ドキュメントサイトが機能しておらず(`fable.app`/`www.fable.app`ともDNS消滅)、一次資料はほぼ入手不能。二次資料(ユーザーブログ)からの断片情報のみ。
- ユーザー評: 「AEに似ているが、よりシンプルで直感的」「レイヤーの混雑をカラーコーディングと区間限定表示で解消していた」。イージングカーブはキーフレーム選択→timeline上に視覚的に表現される、という記述はあるが具体的なUI操作の詳細(ドラッグ方式・修飾キー等)は入手できず。
- 著者(amxmln)は終了後、Rive・Phase(phase.com、未調査)を代替候補として試し「Fableには及ばない」と評している——**「最初から揃っている」を極めた先行例が失われた**という事実自体が、この裁定145のリサーチにとって重要な記録。

---

## 7. 発見物: Linearity Move (linearity.io/move) — Fable後継として言及される

出典:
- https://www.linearity.io/move/
- WebSearch要約(https://www.linearity.io/blog/ui-animation-guide/ ほか)

### 総評
- iPad/macOSネイティブアプリ(ブラウザでない点が他6件と異なる)。業界では「post-Fable時代の代替」として名指しされている。
- **キーフレームは基本的に手動で打たず、canvas上でオブジェクトを動かすと自動記録される**("auto-records keyframes as you move")——AEの「キーフレームボタンを押してから動かす」の逆で、「動かせば記録される」が既定。Rive/LottieFiles Creatorの明示的キー打ちとは対照的。
- イージングはtimeline上のキーフレームハンドルをドラッグしてbezierカーブを作る方式+数値入力の精密指定を両立。既定でease、Inspectorパネルからlinearへ変更可能。
- **Lottie JSON書き出しを「ほぼ唯一の正しいハンドオフ形式」と位置づけている**点がLottie圏との接続性として一致(モーションキーフレーム・イージング・マスク・モーフィングをフルフィデリティで保持すると明記)。
- 詳細な修飾キー・ドラッグ閾値等は一次docsが手薄で未確認(公式ヘルプセンター未探索、今回はマーケティングページとレビュー記事止まり)。

---

## AEから捨てられた要素の共通パターン(横断観察)

1. **プロパティ単位の手動キー打ち(P/S/R/Tキー→ダイヤモンド)自体は生き残っている** — LottieFiles Creator、Rive、Cavalryは形を変えつつ維持。ただしLottielabは「グループ/レイヤーレベルの集約キーフレーム表示」を明示的に廃止し、Linearity Moveは「動かせば自動記録」でキー打ちという操作自体を消した。**「キーフレームを打つ」という動詞そのものを隠す方向と、残しつつ整理する方向の二極化**。

2. **常時ライブ編集(timelineがアクティブなら何をしてもキー化されうる)という前提が疑われている** — RiveはDesign mode/Animate modeを明示分離し、State Machine選択時は自動キー化を止める安全策を導入。AEの「うっかり打ってしまう」事故を構造的に防ぐ設計判断が複数エディタで共有されている。

3. **グラフエディタ(bezierハンドルによるカーブ直接編集)は「あるが最上位ではない」** — Cavalryはプロ向けにフル機能のGraph Editorを持つが、LottieFiles Creator/Rive/Jitterはまず**プリセット(Linear/Natural/Slow down/Elastic/Bounceなど名前付きイージング)を主UIに置き、グラフ編集は詳細設定として後段に回す**。Lottielabに至ってはグラフエディタ自体がまだ未リリース。AEの「グラフエディタがデフォルトで開いて当然」という前提は捨てられている。

4. **キーフレーム群のリタイム(範囲選択→端をドラッグしてスケール)は全エディタが個別に再発明しており、修飾キーが統一されていない** — LottieFiles Creator=Shift+Drag、Rive=Alt+Drag、Cavalryは専用Transform tool(緑ハンドル)。AEのグラフエディタ内でのマーキー+ドラッグ拡縮という操作の「意味」だけが継承され、キー割当は各社バラバラ。Motoliiが独自に決める余地がある領域。

5. **AEの「莫大な数のプロパティを個別にexpand/collapseして探す」手間が、複数エディタで能動的に潰されている** — Lottielabの「未アニメートレイヤーは表示しない」、LottieFiles Creatorの「Show Only Selected」相当機能(Rive)、Jitterの「セグメント=意味単位」でそもそもプロパティ粒度まで降りない設計。**「今アニメートしているものだけを見せる」がデフォルト思想として共有されている**。

6. **AEのdurationはコンポジション全体の固定尺という発想だが、複数エディタは「ネストされた入れ子の時間伸縮」を一級機能にしている** — LottieFiles CreatorのTime Stretch(ネストシーン単位、Hold in Place基準点選択付き)、Riveのwork area、CavalryのPlayback Range。**入れ子構造の時間管理**がAE以降のモーションツールの共通装備になっている。

7. **AEにない「意味づけされた複合操作」が新規に追加されている** — Cavalryの Keyframe Layers(状態ブレンド)、LottieFiles Creatorの Advanced Duplicator(複製+staggerの統合)、JitterのStagger単体操作、RiveのState Machine。いずれも「AEでは手作業の組み合わせで実現していたパターン」を**名前を与えて一級操作に格上げ**している。

8. **Fableの消失(2024-11)自体が一つのデータ点** — 「最初から一番揃っていた」と評されたツールが、AIによるソフトウェアという営みの前提の揺らぎを理由に事業終了した。ツールの機能的完成度だけでは生存を保証しない、という文脈情報として記録しておく価値がある。
