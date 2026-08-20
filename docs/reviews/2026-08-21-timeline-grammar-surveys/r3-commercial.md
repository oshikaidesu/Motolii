# 商用ソフト Timeline 操作文法 — 公式資料採集(裁定144-d, R3)

調査方法: WebSearch(Adobe公式 helpx.adobe.com へのsite:検索・スニペット抽出)。WebFetchは
helpx.adobe.com / support.alightmotion.com とも接続がタイムアウト・403で直接本文取得は不可だったため、
WebSearchが返す検索エンジン側キャッシュ由来の抜粋(公式ページ本文からの引用)を一次情報として採用した。
出典URLは全て公式ドメインを指す。Alight MotionとCapCutは公式ヘルプセンターにジェスチャ粒度の記述がほぼ無く、
その旨を明記した上で信頼度の低い代替情報源(サードパーティ解説)を「補助」として使った箇所は個別に注記する。

---

## 1. After Effects(最優先)

### 1.1 レイヤーバーの移動(move)
- 操作名: レイヤーデュレーションバーのドラッグ移動
- 起動条件: Timeline panel でレイヤーのデュレーションバー(名前ではなくバー本体)をドラッグ
- ドラッグ中の意味: そのままドラッグすると自由に移動。**Shift を押しながらドラッグ**すると、マーカー・コンポジションの開始/終了など「意味のある時間点」にスナップする
- 確定・キャンセル: マウスアップで確定(公式ページに明記された専用キャンセル操作は見つからず。一般的なEscでのドラッグキャンセルはAE全般の挙動として存在するが本ページでは未言及)
- フィードバック: 記述なし(スナップ時のガイド表示等は本ページのスニペットには現れず)
- 出典: [Selecting and arranging layers in After Effects](https://helpx.adobe.com/after-effects/using/selecting-arranging-layers.html)

引用(検索結果からの抜粋):
> To snap the layer duration bar to significant points in time (such as markers or the start or end of the composition), Shift-drag the layer duration bar.

### 1.2 複数レイヤーの同時移動/トリム — 相対タイミング保持
- 操作名: 複数選択レイヤーの同時move/trim
- 起動条件: 複数レイヤーを選択した状態でデュレーションバーをドラッグ
- ドラッグ中の意味: 最初に選択したレイヤーはその場に留まり、最後に選択したレイヤーがドラッグした距離だけ全体移動、間のレイヤーは均等配分でタイミングが変化する(=レイヤー間の相対間隔ではなく「先頭固定・末尾フルオフセット・線形補間」という特有のモデル)
- 出典: [Selecting and arranging layers in After Effects](https://helpx.adobe.com/after-effects/using/selecting-arranging-layers.html)

### 1.3 Quick Offset(複数レイヤー/複数キーフレームの千鳥配置)
- 操作名: Quick Offset
- 起動条件: 複数レイヤー(またはレイヤー横断で複数キーフレーム)を選択し、**Cmd+Option(mac)/ Ctrl+Alt(Win)を押しながらドラッグ**
- ドラッグ中の意味: カーソルが変化。最初に選択したレイヤーはその場に留まり、最後に選択したレイヤーがドラッグ距離ぶん移動、中間レイヤーは時間的に均等配置される「千鳥(stagger)」を作る。ドラッグ中にTotal Offset(全体の展開幅)とPer Layer(レイヤー1つあたりの間隔)を示すオーバーレイが表示される
- 確定・キャンセル: マウスアップで確定
- フィードバック: カーソル変化+Total Offset/Per Layerのオーバーレイ表示
- 備考: レイヤー移動時にキーフレームも追従し、アニメーションの内容自体は保持される
- 出典: [After Effects feature summary (August 2025 release)](https://helpx.adobe.com/after-effects/using/whats-new/2025-4.html)

### 1.4 In/Out点のトリム(レイヤーバー端のドラッグ)
- 操作名: レイヤーIn/Out点のトリム
- 起動条件: デュレーションバーの左端(In)または右端(Out)をドラッグ
- ドラッグ中の意味: In/Outのみを動かし、レイヤー内部のキーフレームは動かない(キーフレームの相対時間位置は固定されたまま、可視範囲だけが変わる)。これに対し、デュレーションバー本体をドラッグして移動する場合はキーフレームも全て追従する
- 出典: [Selecting and arranging layers in After Effects](https://helpx.adobe.com/after-effects/using/selecting-arranging-layers.html)

引用:
> Moving only the In or Out point of a layer doesn't move keyframes... dragging the layer duration bar moves all keyframes.

- 補足: Slip編集バー(レイヤー内部のコンテンツだけを動かす、In/Outは固定)をドラッグすると、選択済みキーフレームだけが動き、未選択キーフレームは動かない。

### 1.5 In/Out点の設定 — キーボードショートカット
- 操作名: 現在時刻でのIn/Out設定(トリム)
- 起動条件: レイヤー選択 + `Alt+[`(In点を現在時刻にトリム)/ `Alt+]`(Out点を現在時刻にトリム)
- 出典: 検索結果は community.adobe.com 経由の言及も含むが、`Alt+[` / `Alt+]` はAE標準キーボードショートカットとして公式リファレンスに準拠([Preset and customizable keyboard shortcuts in After Effects](https://helpx.adobe.com/after-effects/using/keyboard-shortcuts-reference.html) 掲載範囲)。個別の抜粋テキストはフォーラム由来のため、この項目のみ確度は「公式ショートカット表に基づくが本文引用はフォーラム補助」と明記する。

### 1.6 キーフレームのドラッグ移動
- 操作名: キーフレームのドラッグ
- 起動条件: Timeline panel(レイヤーバーモード)でキーフレームアイコンをドラッグ
- ドラッグ中の意味: 自由に時間移動。**Shiftを押しながらドラッグ**するとマーカー・CTI(現在時刻インジケータ)・In/Out点・コンポジション/ワークエリアの開始終了にスナップする。レイヤーバーモードでは、Shiftを押しながらキーフレームをCTIへドラッグすると、そこにスナップする挙動が明記されている
- 出典: [Editing, moving, and copying keyframes](https://helpx.adobe.com/after-effects/using/editing-moving-copying-keyframes.html)

引用:
> When you drag a keyframe, the current-time indicator, or a layer duration bar in the Timeline panel, hold down Shift to snap these items to markers.
> In layer bar mode, hold Shift after you begin to drag a keyframe icon to the Current Time Indicator.

### 1.7 キーフレームの選択(単一・複数・マーキー)
- 操作名: キーフレーム選択
- 起動条件:
  - 単一: キーフレームアイコンをクリック
  - 複数(非連続含む): Shift+クリックで追加選択
  - マーキー(範囲矩形選択): キーフレームアイコンが無い領域からドラッグして矩形を描き、囲まれたキーフレームを選択
  - 既存選択済みキーフレームの周りをShiftドラッグでマーキーを描くと、それらのキーフレームは選択解除される(トグル的挙動)
- 出典: [Setting, selecting, and deleting keyframes in After Effects](https://helpx.adobe.com/after-effects/using/setting-selecting-deleting-keyframes.html)

### 1.8 Graph Editor でのスナップ
- 操作名: Graph Editorキーフレームドラッグ時のスナップ
- 起動条件: Graph Editor で Snap ボタンが有効な状態でキーフレームをドラッグ
- ドラッグ中の意味: キーフレームの値・時間、現在時刻、In/Out点、マーカー、ワークエリアの開始/終了にスナップ
- 一時トグル: ドラッグ開始後に **Ctrl(Win)/ Cmd(mac)を押す**と、スナップのON/OFFを一時的に切り替えられる
- 出典: [Editing, moving, and copying keyframes](https://helpx.adobe.com/after-effects/using/editing-moving-copying-keyframes.html)

### 1.9 レイヤーの複数選択(marquee/Shift/Ctrl)
- 操作名: レイヤー選択
- 起動条件:
  - Composition panel: 選択ツールでドラッグしてマーキー(選択ボックス)を描き、囲まれたレイヤーを選択
  - Shiftを押しながらクリック/ドラッグで選択の追加・除外
  - Timeline panel: レイヤー名または デュレーションバーをクリックして選択。Composition panelやFlowchart panelでの選択とも連動
  - Ctrl+↓ / Ctrl+↑(mac: Cmd+↓/↑)でスタック順で次/前レイヤーへ選択移動。Ctrl+Shift+↓/↑で選択を拡張
  - テンキーでレイヤー番号を直接入力して選択
  - Edit > Select All / Deselect All
- 出典: [Selecting and arranging layers in After Effects](https://helpx.adobe.com/after-effects/using/selecting-arranging-layers.html)

### 1.10 時間スクラブ(CTIドラッグ)
- 操作名: Current Time Indicator(CTI)のドラッグスクラブ
- 起動条件: Timeline panelの時間ルーラー上のCTI(赤い縦線)をドラッグ
- ドラッグ中の意味: 素のドラッグでフレーム単位のプレビュー。プレビュー再生中にCTIをスクラブすると再生は停止するが、**Option/Altを押しながらスクラブ**すると再生を止めずにスクラブできる。**Shiftを押しながらドラッグ**すると、キーフレーム・マーカー・In/Out点・コンポジション/ワークエリアの開始終了にスナップする
- 出典: [General user interface items in After Effects](https://helpx.adobe.com/after-effects/using/general-user-interface-items.html), [Editing, moving, and copying keyframes](https://helpx.adobe.com/after-effects/using/editing-moving-copying-keyframes.html)

### 1.11 ズーム操作
- 操作名: Timeline panelのズームIn/Out
- 起動条件:
  - Zoom In/Outボタンクリック、またはズームスライダーをドラッグ
  - キーボード: `=`(ズームイン)/ `-`(ズームアウト)
  - マウスホイール: 前方回転でパネル中心へズームイン、後方回転でズームアウト
  - トラックパッド: ピンチイン/アウトでズームアウト/イン(マルチタッチジェスチャ)
  - Shiftを押しながらマウスホイール回転で水平スクロール(時間ルーラー/タイムナビゲータ上ではShift+ホイール後方回転で時間が進む、前方回転で戻る)
  - Time Navigator の開始/終了ブラケットをドラッグして特定区間へズーム
- 出典: [General user interface items in After Effects](https://helpx.adobe.com/after-effects/using/general-user-interface-items.html)

### 1.12 AE 修飾キー表(本タスクの中核成果物)

| キー | コンテキスト | 意味 |
|---|---|---|
| Shift(ドラッグ中) | レイヤーバー移動/トリム、キーフレームドラッグ、CTIドラッグ | マーカー・CTI・In/Out点・コンポジション/ワークエリア境界へスナップ |
| Shift(クリック) | レイヤー選択、キーフレーム選択 | 追加選択・除外(トグル) |
| Shift(マーキードラッグ) | 選択済みキーフレーム群の再ドラッグ | 選択解除(deselect)のマーキーになる |
| Ctrl(Win)/Cmd(mac) | Graph Editorでのキーフレームドラッグ中 | スナップON/OFFの一時トグル |
| Ctrl(Win)/Cmd(mac) | レイヤースタック内ナビゲーション(↓/↑併用) | 次/前レイヤーへ選択移動 |
| Ctrl+Shift / Cmd+Shift | レイヤースタック内ナビゲーション | 選択範囲の拡張 |
| Alt(Win)/Option(mac) | CTIスクラブ中 | プレビュー再生を止めずにスクラブ |
| `Alt+[` / `Alt+]` | レイヤー選択時 | 現在時刻でIn点/Out点をトリム(公式ショートカット表準拠、本文引用はフォーラム補助) |
| Ctrl+Alt(Win)/Cmd+Option(mac)(ドラッグ) | 複数レイヤー or 複数キーフレーム選択時 | Quick Offset — 先頭固定・末尾フルオフセットで千鳥配置。Total Offset/Per Layerオーバーレイ表示 |

→ 上記のうち、公式ページ本文からの直接引用で裏取りできたのは Shift系(スナップ・選択トグル)、Ctrl/Cmd(Graph Editorスナップトグル、レイヤーナビゲーション)、Alt/Option(スクラブ)、Ctrl+Alt/Cmd+Option(Quick Offset)。**`Alt+[` / `Alt+]` のみキーボードショートカット表への参照であり、本文引用はフォーラム由来**。これを除けば表はほぼ公式本文で裏取り済み。

---

## 2. Alight Motion(UX north star)

**公式ヘルプセンター(support.alightmotion.com)はZendesk系サイトで自動クローラー/フェッチをブロックしており(WebFetch: 403)、WebSearchのスニペットにもTimeline/トリムのジェスチャ粒度の記述は現れなかった。** 公式サイトから確認できたのは以下のみ:

- 公式ヘルプセンターの構成: Quick Start Guide / Elements: The Complete Guide / Feature Guides(Preview Pan and Zoom, Camera Objects, Layer Parenting and Null Objects等)が存在する
  出典: [Alight Motion Help Center](https://support.alightmotion.com/hc/en-us)
- レイヤー分割(split)時の親子関係: 親レイヤーを分割すると、子レイヤーは分割後の左半分に紐付いたまま残る
  出典: [Layer Parenting and Null Objects – Alight Motion Help Center](https://support.alightmotion.com/hc/en-us/articles/10536997444369-Layer-Parenting-and-Null-Objects)

以下は**公式資料に情報がないため、サードパーティ解説を補助として採用**(信頼度: 中〜低、公式ドキュメントでの裏取りなし):

- 操作名: クリップのトリム(handle drag)
  起動条件: タイムライン上のクリップをタップして選択→左右端に表示されるハンドルをドラッグ
  意味: 左ハンドルを内側にドラッグで先頭をトリム、右ハンドルを内側にドラッグで末尾をトリム。ドラッグ中はプレビューがリアルタイム更新
  出典(補助・非公式): [How To Add, Trim, Split & Delete Clips In Alight Motion](https://themotionalight.com/trim-split-and-delete-clips-in-alight-motion/)
- 操作名: 分割(split)
  起動条件: ハサミ(Scissors)アイコンをタップ。Razor toolとTrimmer toolが別ツールとして存在(Razorは分割、Trimmerはin/out点の定義)
  出典(補助・非公式): 同上、および [How to Use Alight Motion 2025](https://alightmotionx.com/how-to-use-alight-motion/)
- 操作名: キーフレーム追加
  起動条件: プレイヘッドを目的の時間に移動 → プロパティパネルでダイヤモンド型キーフレームアイコンをタップ → 値を変更すると自動補間される
  出典(補助・非公式): [Alight Motion Keyframe Tutorial](https://filmora.wondershare.com/advanced-video-editing/how-to-add-keyframes-in-alight-motion.html)

→ 修飾キーの概念自体がモバイル(タッチ)UIのため存在しない。デスクトップへの翻訳可能なジェスチャとして拾えるのは「ハンドルドラッグでトリム」「ダイヤモンドタップでキーフレーム設定」程度で、AEのような詳細な修飾キー文法は無い。

---

## 3. Premiere / Resolve(基本操作のみ。trim family=ripple/roll/slip/slideは深掘り禁止・存在記録のみ)

### 3.1 Premiere Pro

- **選択**: Selection toolでクリック選択。Shift+クリックで複数選択(連続・非連続とも可)。マーキー(矩形)ドラッグで範囲内のクリップを連続選択、Shift+マーキードラッグで選択範囲に追加。Lasso Toolでの自由形状選択も可能
  出典: [Select clips in Premiere timeline](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/select-clips.html)
- **スナップ**: Timeline panel左上のSnapボタンを有効にすると、クリップをドラッグした際に他のクリップの端・マーカー・時間ルーラーの開始終了・プレイヘッドに自動整列(スナップ)する。位置が合うと縦のガイド線が表示される
  出典: [Activate snapping in Premiere](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/snap-clips.html)
- **移動(move)**: クリップをドラッグ&ドロップで移動。既定(修飾キーなし)はOverwrite編集(アイコン表示)。**Ctrl(Win)/Cmd(mac)を押しながらドロップ**でInsert編集(既存クリップ群を後方へ押し出す)。**Ctrl+Alt(Win)/Cmd+Option(mac)を押しながらドロップ**でRearrange編集。矢印キーでの微調整(nudge)、Ctrl/Cmd+矢印キーで大きい単位のnudgeも可能
  出典: [Different ways to move clips](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/different-ways-to-move-clips.html)
- **キーフレーム選択**: Shift+クリックで複数選択(Selection/Pen tool)。Effect Controls panelでSelection toolによるドラッグ選択、Shift+ドラッグで既存選択に追加
  出典: [Select keyframes in Premiere Pro](https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/select-keyframes.html)
- **マーカー**: Find, move, and delete markersページに準拠する基本操作あり(詳細は今回未深掘り)
  出典: [Find, move, and delete markers in Premiere Pro](https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/find-move-and-delete-markers.html)
- **trim family(ripple/roll/slip/slide)**: Premiereにはこれらの専用トリムツール(Ripple Edit, Rolling Edit, Slip, Slide)が存在することのみ記録。Motolii側では不採用済みのため意味論の深掘りはしない。

### 3.2 DaVinci Resolve

- **スナップ**: タイムライン上部ツールバーのマグネットアイコン(Snap)をクリックしてON/OFF、またはキーボードショートカット **N**。ONの状態でクリップのIn/Out点・マーカー・プレイヘッドが互いに整列(スナップ)する。境界が吸着すると白い縦線が表示される。プレイヘッドがロックされている場合はプレイヘッドへはスナップしない
  出典: [Snapping — DaVinci Resolve 18.6 Manual](https://www.steakunderwater.com/VFXPedia/__man/Resolve18-6/DaVinciResolve18_Manual_files/part715.htm)(Blackmagic公式マニュアルのミラー掲載)
- **マーカー**: プレイヘッドを目的のフレームへ移動し **M** でマーカーを配置。クリップを選択した状態でクリップ内のフレームへプレイヘッドを合わせ、ツールバーのMarkerボタン(または**M**)でそのクリップ上にマーカーを配置
  出典: 同上マニュアル(Adding Markers to Timelinesページ、[part996.htm](https://www.steakunderwater.com/VFXPedia/__man/Resolve18-6/DaVinciResolve18_Manual_files/part996.htm))
- **trim family**: Resolveも Ripple/Roll等のトリムモードを持つが、本タスクでは不採用済みのため存在記録のみ。

---

## 4. CapCut(既定挙動のみ — スナップ・磁石・ドラッグ)

**CapCut公式ヘルプセンター(capcut.com/help)には「Editing & Exporting」カテゴリに12本の記事があるが、スナップ/磁石/ドラッグの既定挙動を説明する専用記事は見当たらなかった。** capcut.com/resource(公式ドメインだがマーケティング/解説系コンテンツ)由来の記述を採用する:

- 操作名: Track Magnet(磁石トラック)/ Auto Snapping(自動スナップ)
  起動条件: タイムライン上部(ズーム操作近辺)にある磁石アイコンをクリックしてトグル。有効時はアイコンがハイライト表示、無効時はグレーアウト
  ドラッグ中の意味: 有効時、クリップ(映像・音声・テキスト・画像レイヤー)をドラッグすると近傍の基準点(他クリップ端・プレイヘッド等)へ自動的に吸着し、ギャップが生じないよう整列する。無効時はクリップは自由に配置され、他要素へジャンプしない
  出典(公式ドメインcapcut.com、ただしresource/マーケティングページであり専用ヘルプ記事ではない): [CapCut PC version](https://www.capcut.com/resource/pc-professional-video-editor) に近い文脈の記述が複数の非公式解説サイトでも一致して引用されている
- 備考: CapCutは磁石アイコンのON/OFFで挙動が切り替わる点はDaVinci Resolveの操作文法と類似(トグルボタン+視覚状態変化)

→ CapCutは4ソフト中もっとも公式一次資料が薄く、実質的にマーケティングページと第三者解説の一致点のみで既定挙動を裏取りした状態。

---

## まとめ表(採集件数)

| ソフト | 採集した操作項目数 | 公式一次資料での裏取り度 |
|---|---|---|
| After Effects | 12項目(1.1〜1.12の修飾キー表含む) | 高(ほぼ全て helpx.adobe.com 本文引用で裏取り。`Alt+[`/`Alt+]`のみショートカット表参照+フォーラム補助) |
| Alight Motion | 2項目(公式)+ 3項目(補助・非公式) | 低(公式ヘルプセンターがクローラーブロックしておりジェスチャ粒度の記述に到達できず) |
| Premiere Pro | 5項目(選択・スナップ・移動・キーフレーム選択・マーカー)+ trim family存在記録のみ | 高(helpx.adobe.com本文引用で裏取り) |
| DaVinci Resolve | 2項目(スナップ・マーカー)+ trim family存在記録のみ | 中〜高(Blackmagic公式マニュアルのミラー掲載経由) |
| CapCut | 1項目(磁石/自動スナップの既定挙動) | 低(公式ヘルプセンターに該当記事なし、公式resourceページ+第三者解説の一致で代替) |
