# Stage panel — Icebook design drafts

Icebookでそのままstory fixtureへ落とせるよう、各案に安定したstory IDを付けたStageパネル草案。
これは実装仕様でも実窓の合否でもなく、視線・密度・主役・操作入口を比較するための候補である。

現行の意味を基線にする。Stageは出力カメラとUser Viewを分け、Composition.cameraは一つ、
世界はz=0を既定とする2.5D、透視投影はzの差でインパクトを生む。点群・深度クラウド・
3Dメッシュは将来のadditiveなstory fixtureとして扱い、自由なカメラ姿勢を現行機能とは主張しない。
各案は一般的なNLEの交通整理ではなく、heroを見て次の一手へ進むcanvasを主役にする。

## ST-01 — stage-camera-truth / Camera Truth

- **problem**: いま見ている絵が書き出される絵なのか分からず、heroの判断を止める。
- **hero / creation role**: Camera viewを「完成物と同じ視点」と一目で伝え、安心して主役の配置と動きを試せる。
- **layout / visual hierarchy**: canvasを全面の主役にし、上縁にCamera / User Viewの二択だけを置く。compのletterboxを暗幕、作品を最も明るい面、下縁を小さな状態帯にする。
- **interaction / entry**: Cameraタブで出力視点へ戻る。User Viewからも同じタブへ戻れる。絵の上には余計な常設ボタンを置かない。
- **density / scale**: sparse。1920×1080の単一テキスト＋背景、1600px幅以上の大きなcanvasを想定。
- **reuse-vs-scratch**: 視点タブ、letterbox、render_frame、状態帯へ委託する。自前はstory用の内容とCameraの意味を示す最小ラベルだけ。別のpreview計算は作らない。

## ST-02 — stage-user-view-return / User View Return

- **problem**: 世界を眺める途中で出力枠を見失い、探索がそのまま誤った構図になる。
- **hero / creation role**: 少し引いた視点から空間の勢いを探し、1操作で完成画角へ戻って試行錯誤を続ける。
- **layout / visual hierarchy**: User Viewを選択状態の面塗りで示し、canvas内に出力カメラの枠だけを細く重ねる。枠外は暗く、枠内のheroを明るくする。
- **interaction / entry**: ホイールで観測zoom、中ボタンドラッグでpan。下縁の「Return to Camera」とShift+Fは同じ動詞へ向ける。
- **density / scale**: sparse-to-balanced。zの異なる3層を置き、カメラ枠と視差が読める900×600以上のcanvas。
- **reuse-vs-scratch**: ObservationCamera、zoom_at_screen_point、pan_by_screen_delta、frame cornersを再利用する。自前は枠外暗幕と復帰ラベルの見せ方だけ。観測値をDocumentの第二カメラにしない。

## ST-03 — stage-point-cloud-observatory / Point Cloud Observatory

- **problem**: 点群が単なる密な画像に見え、奥行きと主役の位置を発見できない。
- **hero / creation role**: 点群の密度と奥行きから、静止画では出ない空間的なheroの核を見つける。
- **layout / visual hierarchy**:黒に近いcanvasに点群を大きく置き、選択した密度塊だけを淡いhaloで囲む。上縁にはCamera / User View、右下には小さなDepth range readoutだけ。
- **interaction / entry**: 点群の塊をクリックして選択。wheelで観測zoom、middle dragでpan。選択の焦点化は既存のStage選択入口から入る。
- **density / scale**: dense。数万点、前景・中景・背景の3密度層を想定。点が小さくても主役のhaloが読める1200px幅。
- **reuse-vs-scratch**: point_cloud / depth_cloudの描画はre_rendererへ、視点はComposition.cameraとObservationCameraへ委託する。自前は密度塊の選択haloだけ。クラスタリングや点群rendererを新規実装しない。

## ST-04 — stage-parallax-ladder / Parallax Ladder

- **problem**: zを変えれば視差が出るのに、どの層が前後にあるかを画面上で推測しにくい。
- **hero / creation role**: 文字・画像・点群を奥行きの階段へ置き、最小の操作でheroらしい前後感を作る。
- **layout / visual hierarchy**: canvasを左85%、右に細いDepth ladderを置く。選択層の点だけ明るくし、pinned層は梯子の外に小さく固定表示する。
- **interaction / entry**: 選択層の点を上下へドラッグしてzを調整する案。0位置へのresetは既存のproperty操作へ戻す。Camera姿勢のorbitは提供しない。
- **density / scale**: balanced。5層まで。canvasは1000×650以上、ladderは視線を奪わない48px程度。
- **reuse-vs-scratch**: LayerPlacementのz、pinned、既存のcamera projectionへ委託する。自前はzの可視化と入力変換の薄い継ぎ目だけ。レイヤー専用の3D世界や第二カメラは作らない。

## ST-05 — stage-depth-scan / Depth Scan

- **problem**: 点群や3Dメッシュの前後関係が一枚の塊になり、どこを主役にすべきか判断できない。
- **hero / creation role**: 深度の帯を観察して、視差が最も強く出る部分をheroの焦点へ昇格させる。
- **layout / visual hierarchy**: canvasの下辺にだけ、前景・中景・背景を示す三段のdepth scan stripを置く。主役の点群は中央、帯は小さく半透明にする。
- **interaction / entry**: scan stripの帯をクリックして対応する深度群をhighlight。wheelとpanはUser Viewの既存操作。Camera viewでは出力枠を保つ。
- **density / scale**: balanced。3深度帯、選択前後の差が分かる程度の点数。横幅1200px、帯は32px。
- **reuse-vs-scratch**: point/depth rendererとObservationCameraへ委託し、表示専用のhighlightだけを自前にする。深度解析・再配置の正本をStage内に持たない。

## ST-06 — stage-fixed-axis-perspective / Fixed-axis Perspective

- **problem**: 2.5Dの奥行き表現は欲しいが、自由orbitを入れると世界・書き出し・操作の意味が増殖する。
- **hero / creation role**: 固定軸のままzとzoomだけで透視のインパクトを体験し、Motoliiの一つの世界という考え方を直感化する。
- **layout / visual hierarchy**: canvas中央に薄いvanishing depth guide、上縁に「Perspective / fixed axis」の状態ラベルだけを置く。操作器具は増やさず作品を最優先する。
- **interaction / entry**: User Viewのpan/zoom、選択層のz調整だけ。cameraの向き変更入口は作らない。Cameraタブではguideを消す。
- **density / scale**: sparse。z=0の平面2枚とzを持つ1枚を比較できる、1000×560以上のcanvas。
- **reuse-vs-scratch**: Projection::Perspective、Composition.camera、glamの既存数学へ委託する。自前は固定軸であることの説明guideだけ。自由orbit、handedness、pose propertyを先に発明しない。

## ST-07 — stage-frame-lock / Composition Frame Lock

- **problem**: letterboxの外へ配置した素材が「壊れた」のか、意図したcropなのか分からない。
- **hero / creation role**: 画角の外側を恐れずに主役を大きくし、最後に出力枠へ戻して構図を決められる。
- **layout / visual hierarchy**: comp rectangleを明るい境界、枠外を均一な暗幕、選択layerをその上に描く。枠線は作品を囲う装飾ではなく、出力範囲の意味として一重にする。
- **interaction / entry**: Camera viewでframe lockを表示。User Viewではframeを観測しながらpan/zoom。Fitは既存のview操作へ委託し、枠をdragしてcameraを複製しない。
- **density / scale**: sparse。単一のhero titleが枠を一部はみ出すfixture。800×500から成立。
- **reuse-vs-scratch**: letterboxed_rect、render camera corners、Camera/User Viewを再利用する。自前は暗幕の階調と枠の状態表現だけ。新しいcrop stateは持たない。

## ST-08 — stage-thirds-tension / Thirds Tension

- **problem**: 主役を中央に置くだけになり、視線の流れや余白を試せない。
- **hero / creation role**: 文字や点群を三分割の交点へ置き、最初の一枚から意図のあるhero構図を作る。
- **layout / visual hierarchy**: canvas全体に9分割を薄く敷き、4つの交点だけをhover時に少し明るくする。grid名や説明文は常設しない。
- **interaction / entry**: Stageのsheet toggleからGridを選択。gizmoで主役を交点へ移す。snapは見せず、必要なら補助点を一時表示するだけ。
- **density / scale**: sparse。主役1、支え2、背景1。1280×720以上で線が作品を覆わないことを想定。
- **reuse-vs-scratch**: SheetOverlay、GizmoOverlay、既存のposition propertyへ委託する。自前は交点のhover cueだけ。独自のlayout engineやsnap正本を作らない。

## ST-09 — stage-golden-flow / Golden Flow

- **problem**: 静止したheroの中で視線の進行方向を設計できず、画面が平板になる。
- **hero / creation role**: 黄金比の流れへ文字の起点と点群の密度を沿わせ、動きの出発点を決める。
- **layout / visual hierarchy**: 黄金比guideを一枚だけcanvasへ重ね、螺旋の終点に小さなfocus markerを置く。guideは低コントラスト、作品は常に前面。
- **interaction / entry**: SheetsのGoldenを選び、gizmoでheroをguide上へ置く。markerは表示専用で、押して新状態を作らない。
- **density / scale**: sparse-to-balanced。文字1と点群1の二主役を想定。1000×600以上。
- **reuse-vs-scratch**: 既存のgolden sheetとcamera-to-screen変換へ委託する。自前は終点markerとfixtureの初期配置だけ。黄金比から自動レイアウトを発明しない。

## ST-10 — stage-safe-title / Safe Title

- **problem**: 文字MVの主役がsafe areaを越え、見栄えの良い構図と発信できる構図が衝突する。
- **hero / creation role**: 文字を大きく動かしながら、タイトルの可読範囲を守る。
- **layout / visual hierarchy**: title-safeを内側の細線、action-safeを外側の点線、文字のbboxを最も明るい線で表示する。作品外の説明は一行のlegendだけ。
- **interaction / entry**: Safe areasのsheet toggleから表示。gizmoで文字を動かすとbboxだけ追従する。guide自体は編集対象にしない。
- **density / scale**: balanced。3行の日本語タイトル＋背景画像。900×600以上で日本語の輪郭がsafe線に埋もれない。
- **reuse-vs-scratch**: SheetToggles、text layerのbbox、GizmoOverlayへ委託する。自前はsafeの二重線とlegendだけ。text layoutやexport safe判定をStageで二重化しない。

## ST-11 — stage-roll-horizon / Roll Horizon

- **problem**: camera rollで画面に勢いを与えたとき、水平の基準を失って調整が偶然になる。
- **hero / creation role**: 文字や点群の斜めの力を意図的に作り、rollをheroの演出として扱う。
- **layout / visual hierarchy**: canvas中央に短いhorizon lineを薄く置き、作品の角度とCameraのrollを同時に読めるようにする。Camera viewではlineを消し、User Viewでのみ出す。
- **interaction / entry**: camera roll propertyの既存入力へ移る。Stage上のhorizonは基準線であり、回転handleの代わりにしない。
- **density / scale**: sparse。大きな一文字＋3層の点。1000×560以上。
- **reuse-vs-scratch**: Composition.cameraのroll、既存camera property、ObservationCameraへ委託する。自前はhorizonの表示だけ。camera専用の新しい状態所有者は作らない。

## ST-12 — stage-overlay-stack / Overlay Stack

- **problem**: grid、safe、frame、selectionが同時に出ると、heroの輪郭より補助線が目立つ。
- **hero / creation role**: 必要なguideだけを瞬時に残し、作品の主役へ視線を戻す。
- **layout / visual hierarchy**: canvasの左上に縦積みの小さなstate chipsを置く。activeなsheetだけがcanvasへ描画され、inactiveなものは薄い文字で存在だけ示す。
- **interaction / entry**: chipをクリックしてGrid、Thirds、Golden、Safe、Frameを個別にtoggle。Escapeでchipsを畳むが、canvasの意味は変えない。
- **density / scale**: balanced。guideが5種類、作品は3層まで。chipsは28px幅の縦列で、canvasを覆わない。
- **reuse-vs-scratch**: SheetToggles、frame overlay、既存viewer stateの表示専用状態へ委託する。自前は複数toggleを一つの視覚的入口へ束ねる薄い器だけ。新しいoverlay計算は作らない。

## ST-13 — stage-gizmo-bounds / Transform Bounds

- **problem**: 変形値をInspectorで探す間に、作品の見た目と操作の因果が切れる。
- **hero / creation role**: 文字・画像・点群を直接つかみ、位置・拡縮・回転を見た結果のまま試せる。
- **layout / visual hierarchy**: 選択layerのbboxを中太線、8つのscale handleを小さな面、中心のanchorを別形状で描く。非選択layerの輪郭は消す。
- **interaction / entry**: bbox内はmove、8 handleはscale、外周handleはrotate、anchorはpan-behind。dragはStart→Move→Commit/Cancelの一周にする。
- **density / scale**: balanced。選択1、周辺layer4。canvas 900×600以上、handleは高DPIでも指で拾える大きさ。
- **reuse-vs-scratch**: GizmoTarget、gizmo hit-test、GizmoDragの契約、PropertyIdへ委託する。自前はstory fixtureと色の状態差だけ。Stage内でDocumentを書かない。

## ST-14 — stage-anchor-compensation / Anchor Compensation

- **problem**: anchorを動かすとlayerの見た目まで動き、回転中心を試すことが怖くなる。
- **hero / creation role**: 文字の中心や点群の重心をずらして、回転・拡縮の感情的な軸を作る。
- **layout / visual hierarchy**: anchor markerをbbox中心から離して強調し、元の中心と補償後のpositionを細いghost lineで結ぶ。ghostはdrag中だけ出す。
- **interaction / entry**: anchorをdragすると見た目を保ったままpositionを補償。Escでtransientを捨て、releaseで一つの編集として確定。
- **density / scale**: sparse。大きな単語1つ、補償線が読める1200×700。
- **reuse-vs-scratch**: GizmoValue::Anchorのanchor＋position対書き、transient、1 gesture＝1 commitへ委託する。自前はghost lineだけ。anchor用の第二historyや別transform modelは作らない。

## ST-15 — stage-rotation-ring / Rotation Ring

- **problem**: kinetic typeを回すとき、pivotと角度の関係が見えず、勢いを偶然に頼る。
- **hero / creation role**: 文字の回転軌道を目で読み、静止画でも動きの予感があるheroを作る。
- **layout / visual hierarchy**: bboxは低コントラスト、外側のrotation ringとanchorだけを高コントラストにする。cursor近くに一時的な角度値を置く。
- **interaction / entry**: ringをdragしてrotation propertyを更新。releaseでcommit、Escでcancel。bodyやscale handleとのhit-test順は既存gizmoに従う。
- **density / scale**: sparse-to-balanced。大きな文字1、補助図形2。ring外周に余白を取れる1000×650。
- **reuse-vs-scratch**: 既存Rotate handle、GizmoPhase、rotation propertyへ委託する。自前はdrag中の角度readoutだけ。別のrotation editorや常設numeric panelは作らない。

## ST-16 — stage-selection-constellation / Selection Constellation

- **problem**: 複数の素材を同じheroの群れとして扱うと、何が選択されているか見失う。
- **hero / creation role**: 文字、点、背景の役割を一つの視覚的なまとまりとして試し、群れの重心を作る。
- **layout / visual hierarchy**: active layerは明るいbbox、他の選択layerは細い点線、非選択layerは通常表示。画面端に選択数を一行だけ出す。
- **interaction / entry**: marquee、Shift追加、空クリック解除。shared boxを出す案では、1 gesture＝1 commitのDocument batchへ戻す。
- **density / scale**: dense。選択8、非選択10まで。canvas 1200×700以上、bboxが重なってもactiveを判別できること。
- **reuse-vs-scratch**: MarqueeOverlay、Session selection、Document::apply_allへ委託する。自前はshared selection outlineを足す場合の薄い表示だけ。選択正本をStageに複製しない。

## ST-17 — stage-path-sculpt / Path Sculpt

- **problem**: ベジェ形状の輪郭をInspectorへ往復しながら作ると、heroのシルエットを失う。
- **hero / creation role**: ペンの軌跡そのものを主役の形として試し、文字や点群の背後に固有の輪郭を置く。
- **layout / visual hierarchy**: pathの頂点を小さな点、選択頂点を明るいring、control handleを細線で描く。完成したfillは薄く、輪郭を優先する。
- **interaction / entry**: 頂点drag、control handle drag、path close。releaseで確定、Escapeでキャンセル。shape toolのSelect中は他のoverlayへイベントを渡す。
- **density / scale**: balanced。12頂点以内、1000×600以上。全頂点を常時ラベル付けしない。
- **reuse-vs-scratch**: PathEditOverlay、camera-to-screen変換、既存の確定messageへ委託する。自前は選択頂点のfocus表現だけ。path geometryやDocument write口を二重化しない。

## ST-18 — stage-mask-reveal / Mask Reveal

- **problem**: matteの境界が素材の色に埋もれ、何が隠れ何が見えるか判定できない。
- **hero / creation role**: 文字や点群を切り抜く境界を意図として調整し、出現・消失の演出を作る。
- **layout / visual hierarchy**: checkerboardを背景にし、mask pathを高コントラスト、reveal結果を通常の明るさ、外側を少し暗くする。maskの用途を一枚の小さなstatus labelで示す。
- **interaction / entry**: mask pathの頂点をdrag、checkerboardをstate bandからtoggle、channel表示でAlphaへ切り替える。
- **density / scale**: balanced。複雑度20頂点以内、900×600以上。alphaの境界が一目で読めること。
- **reuse-vs-scratch**: MaskPathEditOverlay、checkerboard、ChannelDisplay、既存mask propertyへ委託する。自前は外側dimとlabelだけ。matte合成やalpha計算をStageで実装しない。

## ST-19 — stage-shape-first-spark / Shape First Spark

- **problem**: 空のStageで最初の結果までが遠く、作る気持ちが立ち上がらない。
- **hero / creation role**: rectangle、ellipse、penの一筆から、すぐに動かせるheroの種を作る。
- **layout / visual hierarchy**: 空のcanvas中央に一つの薄いseed preview、上縁またはcanvas端にShape / Ellipse / Penの三つだけを置く。既存素材一覧は出さない。
- **interaction / entry**: toolを選んでdrag、Penは点を置いてclose。Enterで確定、Escapeで一時形状を破棄。生成後はseed案内が消える。
- **density / scale**: sparse。空projectまたは背景1枚＋新shape1。800×500から成立。
- **reuse-vs-scratch**: ShapeTool、ShapeToolOverlay、既存create intentへ委託する。自前はempty状態のseed previewと一行のentry hintだけ。Stage専用のshape modelは作らない。

## ST-20 — stage-object-focus / Object Focus

- **problem**: 多層のheroを調整すると、選択対象の輪郭と背景の情報量が競合する。
- **hero / creation role**: 一つの主役を深く磨き、周辺のlayerを文脈として残したまま視線を整理する。
- **layout / visual hierarchy**: active objectは通常彩度、周辺layerは明度を一段落とし、activeのbboxとanchorだけを表示する。画面上部にFocus: layer nameを短く出す。
- **interaction / entry**: click、marquee、Inspectorから選択。空クリックで通常表示へ戻る。Focus表示はselectionから導出し、別モードとして保持しない。
- **density / scale**: balanced-to-dense。主役1、周辺15まで。1200×700以上。
- **reuse-vs-scratch**: Session selection、GizmoOverlay、compositorの既存frameへ委託する。自前は周辺dimの表示変換だけ。Stageにfocus selectionの第二正本を作らない。

## ST-21 — stage-channel-truth / Channel Truth

- **problem**: alphaやRGBの破綻が完成画の色に隠れ、heroの素材感を誤って判断する。
- **hero / creation role**: 透明の縁、色の欠落、マスクの効き方を一瞬で確認して、表現の試行を安全に続ける。
- **layout / visual hierarchy**: canvas全体を診断画像にし、下縁にRGB / Alpha / R / G / Bの短い選択列を置く。Alpha時だけcheckerboardを背面にする。
- **interaction / entry**: ChannelDisplayを直接選び、RGBへ戻る。診断状態はDocumentへ保存しない。
- **density / scale**: sparse。単一の半透明文字＋点群。800×500以上、グレースケールの差が読めること。
- **reuse-vs-scratch**: apply_channel_display、ChannelDisplay、checkerboard、viewer barへ委託する。自前はfixtureのRGBA入力だけ。別のrender pathや色補正を作らない。

## ST-22 — stage-resolution-budget / Resolution Budget

- **problem**: previewが粗いとき、作品の問題と表示負荷の問題を取り違えて制作を止める。
- **hero / creation role**: 画質を意図的に落としても、heroの構図とタイミングを先に試せる。
- **layout / visual hierarchy**: canvasを崩さず、下縁にAuto / ½ / ¼と実効倍率を小さく表示する。粗さの説明はcanvas外の一行だけにする。
- **interaction / entry**: viewer barのquality menuで既知の値へ直接移動し、state bandのcycleでも辿れる。Export品質は変えない。
- **density / scale**: balanced。点群＋ぼかし背景のような負荷差が出るfixture。1000×600以上。
- **reuse-vs-scratch**: PreviewResolutionCap、effective_preview_scale、resolution_quality_viewへ委託する。自前は「preview budget」の短い状態文だけ。別の品質設定正本を作らない。

## ST-23 — stage-playback-pulse / Playback Pulse

- **problem**: 再生中に映像・playhead・音の同期が分からず、heroの勢いを評価できない。
- **hero / creation role**: 画面の変化を音の拍と一つの動きとして感じ、最初のMV的な気持ちよさを判断する。
- **layout / visual hierarchy**: canvasを全面にし、再生中だけ下縁に細いpulse line、現在frame、audio lockの状態を表示する。Timelineの目盛りはStageへ持ち込まない。
- **interaction / entry**: SpaceとTimelineのplayheadを主入口にし、Stageは同じSessionを読む。pause状態ではpulseを消し、現在frameだけ残す。
- **density / scale**: sparse。3秒の音＋文字のscale変化＋背景1枚。1200×675以上。
- **reuse-vs-scratch**: Session playhead、Engine::render_frame、Timeline transport、AudioProgramへ委託する。自前は再生中のpulse overlayだけ。Stageに第二のclockを置かない。

## ST-24 — stage-scrub-trace / Scrub Trace

- **problem**: scrub中のフレーム更新が一瞬途切れると、render失敗や操作ミスに見える。
- **hero / creation role**: 動きの前後関係を短い試行で読み、どの瞬間がheroになるかを見つける。
- **layout / visual hierarchy**: 現在frameを最も明るく、直前のframeの輪郭を薄いghost、次frameの輪郭をさらに薄く表示する。常時表示せずscrub中だけ出す。
- **interaction / entry**: Timelineのplayhead dragから入る。Stage自身は時刻を変更せず、Escで通常表示へ戻る。
- **density / scale**: balanced。1秒に3つの文字位置が変わるfixture。1000×600以上、ghostは輪郭だけ。
- **reuse-vs-scratch**: Session/playheadと同じrender_frameへ委託する。自前は一時的なonion outlineで、保存・exportに通さない。履歴や第二時刻を作らない。

## ST-25 — stage-beat-impact / Beat Impact

- **problem**: 音楽の拍と映像の変化が別々に見え、MVの起点を失う。
- **hero / creation role**: 一拍ごとの文字の出現、点群の密度変化、camera rollの意図を視覚的に試す。
- **layout / visual hierarchy**: canvas中央のheroを保ち、beat時だけ中心から短いradial pulseを出す。beat番号は小さく、Timelineのmarker列は表示しない。
- **interaction / entry**: playbackとscrubでbeat pulseを観測する。markerの作成・編集はTimelineへ返し、Stageは結果を読むだけ。
- **density / scale**: balanced。8拍、文字3レイヤー、点群1。1200×675以上、pulseは数フレームで消える。
- **reuse-vs-scratch**: Timeline marker、audio clock、Engine renderへ委託する。beat検出そのものは未確定の外部入力として扱い、自前は表示pulseだけに限定する。

## ST-26 — stage-empty-first-spark / Empty First Spark

- **problem**: 空projectを開いたとき、Stageが壊れているのか何をすればよいのか分からない。
- **hero / creation role**: 最初のdropかshape作成へ数秒で到達し、空白を制作開始の余白へ変える。
- **layout / visual hierarchy**: comp frameを保った静かなcanvas中央に「Drop media」または「Create shape」の二つの入口だけを置く。Camera/User Viewや不要な設定は残す。
- **interaction / entry**: Browserからdrop、ShapeToolからshape作成。最初のlayerが立った瞬間にempty copyは消える。
- **density / scale**: sparse。0 layer、comp情報1行、entry 2個。800×500から成立。
- **reuse-vs-scratch**: Documentのempty判定、Browser drag、ShapeTool、Composition frameへ委託する。自前はempty illustrationと文言だけ。空状態専用のDocument stateは作らない。

## ST-27 — stage-empty-composition-ready / Composition Ready

- **problem**: 空のStageでcomp設定が無いように見え、作品の置き場所を想像できない。
- **hero / creation role**: まず画角と余白を理解してから、文字や点群を主役として置ける。
- **layout / visual hierarchy**: 暗いcanvasの中央に空のcomp rectangle、右下に「1920×1080 · 0 layers」、枠内に小さなfocus crossを置く。CTAは枠の外に出さない。
- **interaction / entry**: Cameraで枠を見る、Browser dropまたはCreate shapeで最初の内容を入れる。Fitはview操作として扱う。
- **density / scale**: sparse。0 layerの構図確認に特化。900×560以上、余白を主役にする。
- **reuse-vs-scratch**: CompSpec、letterbox、empty Document、既存create入口へ委託する。自前はcomp metadataとfocus crossだけ。新しいsetup wizardをStageに埋め込まない。

## ST-28 — stage-render-recovery / Render Recovery

- **problem**: render errorで画面が空になると、直前までのheroも原因も同時に失われる。
- **hero / creation role**: エラーの瞬間にも作品の状態を見失わず、修正して再び試す意欲を保つ。
- **layout / visual hierarchy**: last valid frameまたはcheckerboardをcanvasに残し、隅に理由を一枚のstatus cardとして重ねる。cardは作品を覆わず、原因文を主、操作は一つに絞る。
- **interaction / entry**: エラー文を読み、可能ならReturn to Cameraまたは表示を閉じて同じframeへ戻る。Stageからrender設定を増やさない。
- **density / scale**: sparse。hero一枚＋error card一枚。1000×600以上、赤い警告面で作品を塗りつぶさない。
- **reuse-vs-scratch**: observation_preview_sourceの理由付きErr、Shell status、既存のsafe fallbackへ委託する。自前はerror cardの配置だけ。engineのretryやerror分類をStageで再実装しない。

## ST-29 — stage-input-rejection / Rejection With Context

- **problem**: 読み込めない素材や観測視点の失敗が、ただの空Stageや無反応に見える。
- **hero / creation role**: 失敗の理由を理解して別素材・別視点へすぐ切り替え、制作ループから離脱しない。
- **layout / visual hierarchy**: 直前の有効な絵を保持し、下縁state bandの直上に短い理由帯を置く。ファイル名や視点名を理由の冒頭に残し、modalでcanvasを塞がない。
- **interaction / entry**: reason bandをクリックして詳細statusへ移る。再試行や素材交換の実体はBrowser・Shellの既存入口へ返す。
- **density / scale**: balanced。通常frame、理由帯、現在viewの3要素。900×560以上。
- **reuse-vs-scratch**: Shell status、Browser rejection、observationのErr伝播へ委託する。自前はStage内での表示位置と一行要約だけ。Importやdecodeの責任をStageへ移さない。

## ST-30 — stage-hero-focus-lab / Hero Focus Lab

- **problem**: パネルを便利機能で埋めるほど、最初に見たい「作品が立ち上がる瞬間」が遠くなる。
- **hero / creation role**: 文字、点群、奥行き、音同期を一枚の大きなcanvasで試し、作った結果が次の制作を誘う状態を作る。
- **layout / visual hierarchy**: canvasを約85%にし、中央にhero fixture、上縁はCamera/User View、下縁はresolution・channel・checkerboardの状態帯だけ。gizmoやsheetは選択時に現れるcontextual overlayとする。
- **interaction / entry**: dropまたはCreate shapeでseedを入れ、gizmoで主役を動かし、User Viewで奥行きを観測し、Spaceでpulseを確認する。各操作は既存のpane・Session・Documentへ戻る。
- **density / scale**: balanced。text 1、point/depth cluster 1、背景1、beat 4拍。1440×900の大きなIcebook storyを基準にする。
- **reuse-vs-scratch**: Camera/User View、gizmo、sheets、channel display、resolution state、render_frame、playheadへ全面的に委託する。自前はhero fixtureの初期Documentとstoryの見せ順だけ。第二runtime、第二評価経路、常設NLE chromeは作らない。
