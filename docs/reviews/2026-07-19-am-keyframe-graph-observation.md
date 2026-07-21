# Alight Motionキーフレームグラフ観察台帳

日付: 2026-07-19
状態: **公式事実確認済み／既決の区間補間をReact fixtureへ反映／MultiEase比較中**

## 目的と証拠

Alight Motion（AM）をキーフレームUXの参考にする時、AMの事実、Motoliiへの採否、現行React fixtureとの差分を分離する。AM画面の一括模倣や、旧HTMLへの機能追加を許可する資料ではない。

一次資料:

- [Alight Motion Help Center — Animation Easing Curves](https://support.alightmotion.com/hc/en-us/articles/10536934703889-Animation-Easing-Curves)（2026-07-19再確認）
- [concept.md](../concept.md)「AM式の高度イージング型を採用」（2026-07-10決定、commit `97d934e`）。Bounce / Elastic / Steps / Elastic Stepsを式やParamDriverでなく区間補間として持つ
- ユーザー撮影スクリーンショット34枚（IMG_4933〜IMG_4966、2026-07-19閲覧・幾何抽出済み。リポジトリへの画像取込は未実施）。取込規約は[evidence README](evidence/am-keyframe-graph/README.md)
- 現行比較画面: `http://127.0.0.1:5173/#plugin-browser-candidate`

出典等級:

- `公式`: AM公式ヘルプ本文・添付画像で直接確認
- `実機`: 版、OS、操作列を記録したユーザースクリーンショットで確認
- `Motolii判断`: AMを現行Document、Undo、UI境界へ翻訳した採否

未取込スクリーンショットの記憶だけで`実機`を埋めない。

## 観察、採否、現行差分

| ID | AMで確認した操作面 | 出典 | Motolii判断 | `#plugin-browser-candidate`の現状 |
|---|---|---|---|---|
| AM-KG-01 | Curve Editorは専用iconから開く | 公式 Step 2 | 採用。Preview直下のGraph icon | bridge fixtureに存在 |
| AM-KG-02 | playheadを隣接key間へ置くと、その区間のcurveが見える | 公式 Step 3 / Multiple Keyframes | 採用。key単体や任意選択集合でなく1区間を対象 | bridge fixtureに存在 |
| AM-KG-03 | 現在propertyのkeyは白いdiamond＋dark border、他propertyのkeyは薄くborderなし | 公式 Step 1 | 採用。fill、stroke、opacityでcurrent/contextを区別 | **React候補へ反映・操作試験あり** |
| AM-KG-04 | shape presetと2本のhandleでcurveを編集する | 公式 Step 4 | 採用。形状thumbnail＋Bezier handle | bridge fixtureに存在 |
| AM-KG-05 | Xは元時間、Yはremapped time、傾きが速度 | 公式 Step 4 | 意味だけ採用。常設説明を増やさずaccessible descriptionへ | **React候補のaccessible nameへ反映** |
| AM-KG-06 | 隣接key pairごとに別curveを持つ | 公式 Multiple Keyframes | 採用。左keyのoutgoing interpolation | UI試験が不足 |
| AM-KG-07 | Overshootは既定で点線内に拘束し、overflowで明示ON/OFF | 公式 Overshoot | 採用。既定OFF。ON時だけmanual handleを範囲外へ動かせる。Elastic / Elastic Steps選択は型の意味としてONへ変わったことを明示し、**非overshoot型(Bounce / Cyclic / Random / Steps)への切替では明示OFFへ戻す**。非overshoot型はtoggle状態に関わらず曲線を0〜1へ拘束する | **React候補へ反映・既定OFF/型切替試験あり** |
| AM-KG-08 | CurveをCopyし別区間へPasteできる | 公式 Copying and Pasting Curves | 採用。CopyはTransient、Paste currentは1 Undo | **React候補へ反映・操作試験あり** |
| AM-KG-09 | 選択propertyの全keyframe pairへPasteでき、他property／layerは変えない | 公式 Copying and Pasting Curves | 対象件数つき`Paste all in current channel`、1 macro／1 Undoとして採用 | **React候補へ反映・操作試験あり** |
| AM-KG-10 | Bounce、Elastic、Cyclic、Random、Steps、Elastic Stepsの高度補間型 | 公式 Advanced Easing Types／Motolii 2026-07-10決定 | 採用済み。すべて既存key pairの**区間補間**であり、適用してもkeyの個数・時刻・値を変えない。CyclicはSine波として識別可能にする | **React候補へ反映・非破壊試験あり** |
| AM-KG-11 | Overshoot OFF時の既存範囲外curve処分 | 公式では未確認 | parameterを黙って書き換えない。比較fixtureではOFFが出力表示をkey値範囲へ拘束し、元parameterは保持する。Document意味へは未昇格 | **React候補へ反映** |
| AM-KG-12 | 数値文字列copy、favorite即適用、User curve library | AM当該記事では未確認 | Flow／Motolii側の判断として分離 | AM由来と表記しない |

## 実装境界

`#plugin-browser-candidate`のEasing Graph viewは`src/candidates/EasingGraphCandidate.jsx`へ置換した。旧HTMLは変更せず、`#all-surfaces`等のparity sourceとして維持する。区間導出、Bezier handle更新、Undo表示のfixture adapterはまだlegacy scriptに依存するため、製品状態modelの完了扱いにはしない。

### egui移植時に保持する操作契約

AMスクリーンショットから移す対象はDOM構造、CSS、px位置ではなく、各補間型の**操作意味**である。React fixtureのbutton、SVG node、`data-*`属性を製品componentまたは公開APIへ移植しない。

- toolkit非依存のeditor stateは、対象区間、補間型、型ごとの編集parameter、drag開始時snapshot、現在previewだけを持つ。eguiの`Response`、pointer event列、`Rect`、DPI値は持たない。
- handleは補間型ごとに`handle_id / 対応parameter / 表示条件 / 正規化座標 / 可動域・連動規則`を宣言する。**本数、形、拘束、連動、表示条件はスクリーンショット証拠を確認してから埋め、Bezierの2 handleを全型へ流用しない**。
- `BeginHandleDrag → Preview → Commit / Cancel`を共通Intentにする。drag中はTransient preview、pointer-upで既存D2 commandを1回だけ発行し、Escape／capture lossではdrag開始時snapshotへ戻してDocument変更ゼロにする。
- graph座標は`u`と補間出力の正規化値で保持し、egui描画時だけpanel `Rect`へ写像する。px、DPI、screen座標をDocument、評価器、Undo payloadへ入れない。
- handle dragと数値入力は同じparameter Intentへ正規化する。keyboard操作や直接入力を別の意味・別commandにしない。
- 高度補間型の切替でもkeyframeは生成しない。変更対象は選択区間のoutgoing interpolationだけであり、handle操作も同じ1区間／1 Undo境界を守る。
- Overshootは既定OFFである。OFF時のmanual handleはkey値範囲へ拘束し、ONを選んだ時だけ範囲外へ出せる。Elastic系のように型自体が終点越えを意味する場合も、選択時に状態変化を明示して黙って許可しない。

React側でこの契約をまだ満たさず、DOMを直接変更している箇所は比較fixture用adapterである。egui実装へ転用せず、U4bでtoolkit非依存state／Intentを先に実装してから両者を投影する。

既知のadapter競合（2026-07-19修正）: legacy fixture scriptは`#interval-easing`と`[data-curve]`ボタンへ直接handlerを張り、`renderEasingGraph()`が旧座標系（0..100）で`#graph-curve`・`#graph-handle-a/b`・`#easing-values`等を上書きする。旧HTMLは不変のまま、React候補は該当クリック直後にworkspace subtreeをkey差し替えで再マウントして描画所有権を取り戻す（Playwright回帰試験あり）。これはfixture adapter限定の措置であり、egui実装では単一所有なので発生しない。候補に存在しない`#curve-shelf`を参照するlegacyの`#open-curve-shelf` handlerは初期化後に無効化する。

### advanced handle mapping

公式記事はBezierの2 handleを明示する一方、advanced各型のhandleまでは説明していない。2026-07-19にユーザー撮影スクリーンショット34枚（IMG_4933〜IMG_4966、iPad版AM、curve editor実機）を8並列の幾何抽出（プロット枠px・0..1ボックスpx・曲線18点以上サンプル・handle座標・装飾端点）にかけ、全6型の**閉形式モデル**まで特定した。ここでのparameter名は比較fixture内の呼称であり、永続schema、`Interp` field名、製品既定値ではない。

**グラフ描画の根本仕様（全型共通、実機確認）:**

- AM実機の観察では、プロット領域は固定（473:499、ほぼ正方形）だが、**0..1ガイドボックスはコンテンツに合わせて縦方向へ自動フィット**する。実測: Bounce/Random/Steps系はボックス高294px（上下余白≒±0.35v）、Cyclicはレバー用に余白拡大（v≈1.64まで）、Elasticはovershoot分をさらに確保（v≈2.21まで）。
- **Motolii採用差分（2026-07-19決定）**: 動的フィットはhandle操作中の座標写像を変えて不具合源になるため採らない。Overshoot OFFは標準固定範囲、ONはmanual handleとElastic limitの最大可動域を最初から収める固定範囲へ一度に切り替え、curve・parameter変更では範囲を変えない。
- ガイドは上下2本の点線（v=0/v=1）のみで左右の縦線は無い。緑のendpoint dotが(0,0)と(1,1)を示し、playhead位置に点線の縦線が立つ。
- 曲線は太い丸端線。0..1ボックス外へも描かれ、クリップはプロット枠端のみで起きる。
- handleは大きな塗り円（実機黄=parameter、白=補助）。装飾はstem・点線limit線・envelope線で、すべてparameterから導出される。数字バッジ等の注釈overlayは無い。

| 補間型 | 実機の閉形式とhandle | 状態 |
|---|---|---|
| Bounce | **自己相似バウンド**: handle=(a,h)は最初の谷の頂点。d=1-h、T=a/(1+d)。立ち上がり(u/T)²、以後は幅2Td^k・深さd^kの放物線弧（振幅も持続もd倍ずつ。弾道則√dではない）。頂点はv=1に接するcusp、弧が入り切らなくなったらv=1で平坦保持。h=0で無減衰の全振幅弧＝「ほぼcos」 | **実機確認**（3状態を±0.01で照合） |
| Elastic | v=1−(A−1)(1−u)ⁿcos(2πu/p)。**クランプではなく振幅スケーリング**（A=1で攻め上がり後の平坦保持、undershoot無しを確認）。limit handle=点線天井線の右端（縦=A）。波handle=曲線から垂下するstemの先端（x=p=最初の谷位置、縦=減衰n、曲線上=平坦） | **実機確認**（n≈(1−v_h)⁻²のみ中確度） |
| Cyclic / Sine | v=f+(1−f)·W(frac(u/T))、f=E·u。谷はenvelope線、頂はv=1に接する。W: 頂点位相s（top guide上のhandle x=s·T）と平滑度c（白handle、v≈1.5線上をx=c·Tで滑る。c=0→cosine、c=1→linear）。s=0下りsaw、0.5でsine/三角、1上りsaw。周期handleは基線上(T,0)。終端は位相途中でも切る（snapしない） | **実機確認**（cの連続blendのみ外挿） |
| Random | v=u+amp·env·noise+bias·Ψ。noiseは滑らかなvalue noise（折れ線でない）、0..1ボックスへ**クランプされない**。左端の縦scrub=seed再抽選、上辺=粒度（左=最細）、自由handle=振幅（null線v≈0.47からの距離）＋エネルギー中心（x）、下辺=帯全体を上下へ押すbias（Ψ=(1−e^{−u/τ})(1−e^{−(1−u)/τ})）。白丸(1,1)は終端keyアンカー | **実機確認**（振幅飽和則・粒度写像は中確度） |
| Steps | 対角線のsample-and-hold量子化 v=w·floor(u/w)。白anchor（基線上）のx=段幅w（連続値、1/wは非整数可）、黄satellite（上辺guide上）のanchorからの水平offset=平滑幅s。平滑rampは段時刻に**到着**する（easeが段に先行）。端数は終端(1,1)へのジャンプ | **実機確認** |
| Elastic Steps | v=P·(k−1+S_E(τ))、遷移は段時刻kPで**開始**（Stepsと逆向き）。黄handle（基線上）x=P=段幅かつ段高。白handle（左端）の高さE=弾性: E=0で幅0.45Pの滑らかなS、E=1で瞬間ジャンプ＋減衰リング（初回+36%、周期0.112τ、半周期×0.72）。stemは白handleから曲線上の(2P,P)角へ | **実機確認**（E→overshoot量の中間則は中確度） |

横／縦はReactとeguiの物理座標契約ではない。toolkit非依存側ではparameterの正規化値だけを扱い、各rendererが画面上のhandle位置へ写像する。

実装は`src/candidates/easing-graph-model.js`（純関数のみ: 評価器・handle anchor/apply・装飾導出・Overshoot状態からの固定ビュー選択）へ分離し、React componentは射影だけを行う。node実行で抽出実測値と直接照合でき、egui移植時はこのmoduleの写像をそのままcustom paintへ渡す。数字バッジのような注釈overlayはAM画面に存在しないため置かない。

修正順は次とする。

1. `component-map.json`でReact viewとlegacy state adapterの所有を分けて追跡する。
2. current/context key、Overshoot、Copy/Paste/Paste allはReact componentとPlaywright操作試験を同時に維持する。
3. 区間導出、curve編集state、Undo adapterをReact candidate stateへ移した後、同領域のlegacy selector／script依存を削除する。
4. 高度補間型の製品実装は`concept.md`の追加的`Interp` variantへ接続する。React候補は既決の意味を比較するモックであり、未実装の永続schemaやparameter既定値を発明しない。
5. Stepを含む高度補間の適用前後で、keyframeの個数・時刻・値が完全一致し、変更されるのは選択区間の左keyが持つoutgoing interpolationだけであることを試験する。
6. スクリーンショットからhandleの本数・拘束・連動・表示条件を証拠化し、toolkit非依存handle宣言とIntent試験を先に閉じてからegui custom paintへ接続する。

## MultiEase比較観察（2026-07-19）

参照: [MultiEase（BOOTH）](https://booth.pm/ja/items/3924629)

状態は**比較中**。これは採択済みの区間Easing Graph Viewを撤回する決定ではなく、3点以上のカーブ構築用途を別の操作面として扱う必要があるかを調べる観察である。

掲載説明から直接確認できる範囲では、MultiEaseは単一区間の3点Bezierではない。複数カーブを使う3点以上のイージング、カーブごとの点追加・削除とhandle操作、選択カーブのコピー、作成カーブのpreset保存を持つ。ポイントが1つ以上あるカーブでは各キーフレーム間へキーフレームを自動生成するため、既決の「1区間の補間だけを変え、keyの個数・時刻・値を変えない」高度補間型とは編集結果が異なる。

ただし、MultiEaseの利用実績は「3点以上の恒久機能が本質的に必要」という証拠にはしない。AE Graph Editorの区間編集・複数区間編集が扱いにくいため、その回避策として中間key生成とpreset化が発明された可能性がある。これはユーザーの仕事と先行製品の制約を分けていないため、現時点では**因果未確認**である。

Motoliiでの第一候補を複数点モードの先行実装から、既決のFlow / AM式**単一区間Graph Viewを十分に操作可能にすること**へ戻す。その固定fixtureで複合的な動きを作れない実例が残った場合だけ、既存Graph View内の独立した複数点カーブ構築モードを比較する。

- 1区間のEasing編集は現行どおり左keyのoutgoing interpolationだけを変更する。
- 複数点構築は**実装停止**。必要性が残った場合の候補は、適用前をTransient previewとし、明示Apply時だけ中間点を通常のkeyframeへmaterializeする。全体を1 macro／1 Undoにし、非選択keyと外側区間を完全に保存する。
- 中間点を「1区間内の恒久knot」として保持する案は、新しい`Interp` variant、制御点配列、評価・migration・plugin契約を要求するため本比較の非目標とする。
- 作成curveのpresetはDocumentへ入れない。User libraryの保存形式、共有、import/exportは所有境界が決まるまで未決とする。
- 現行React fixtureでGraph入口がAutomation channelの新しい投影へ追従せず無効化されている現象は、Graph View不要の根拠ではなくstate adapterの接続不良として分離する。

再開前に満たす停止線:

1. 単一区間Graph Viewで実行できず、複数点でなければ解けない具体的な制作操作を再現する。
2. その不足が操作回数や一覧性だけの問題なら、区間の複数選択・copy/paste・preset改善で解けないことを示す。
3. 点が通常keyframeとして確定するのか、補間内部のknotなのか。
4. time/valueをどの範囲へ正規化してcopy/pasteするのか。
5. 複数channel、異なる次元、部分的に無効な選択をどう拒否するか。
6. Apply、Cancel、Undo後にkey個数・時刻・値・外側補間がどう不変になるか。
7. presetの所有先と、presetをDocumentへ適用した後の自己完結性。

非目標は、モックから公開API、Document field、preset file format、script runtime、式言語を発明すること、また旧HTML archiveへ比較UIを追加することである。

## U4b受け入れへの追補候補

- current channel keyとcontext-only keyを通常／grayscaleで区別できる。
- Overshoot状態と操作入口を読め、OFFで既存範囲外curveを黙ってclampしない。
- CopyはDocument／Undo不変。Paste currentは1区間だけ、Paste allは現在channelの全区間だけを1 macro／1 Undoで変更する。
- Copy前Paste、対象0、別channel／別layer混入、bulk途中失敗を負例にする。
- 高度補間型の適用でkeyframeを追加・削除・移動せず、key値も変更しない。1区間の補間変更を1 Undoにする。
- handleの操作意味が確認済みでも、この比較fixture内のparameter名・数値域・既定値は、恒久形式の審査が終わるまでDocument、公開API、製品`Interp` fieldへ昇格させない。

## スクリーンショット取込後

ユーザー撮影資料を受領したら、evidence manifestへ版、OS、撮影日、操作状態を追記する。公式記事と現行アプリ版が異なる場合は、React fixtureを先に直さず本表の観察と採否を再審査する。
