# Stage 5 — UIと操作の採用事項

根拠は2026-09-05〜06の利用者の指示と訂正。目標・採用した操作・現在の実装制限を分ける。この文書は再編で意味を落とさないための基準であり、全項目の実装完了を宣言しない。根本は[concept](../concept.md)。

## 画面の構成

| 領域 | 意味 |
|---|---|
| 左 Browser | 素材・作成・効果・色への入口。分類→絞り込み→結果→プレビュー |
| 中央 Stage | 作品の結果を見る／直接操作する空間 |
| 右 Inspector / Desk | 選択対象の調整と詳細。Inspectorは意味の所有者ではなく窓 |
| 下 Timeline | 長さ・時間・変化の編集。再生や時間の共通操作はここへ集約 |

大枠は採用済み。残すべき入口を消して密度を上げない。Browser分類の詳細な分類体系は宿題だが、分類自体は結果を絞り込む機能として残す。映像・画像・結果プレビューは16:9など内容に適したサムネイルを使う。原始図形を含む全項目を一律に巨大な画像カードへする意味ではない。

色の輪はBrowserのColorsに常設し、Inspectorで触った色の対象へ追従させる。Deskへ移して視線を分散させない。InspectorのTransformの位置や並びは安定させ、選択に応じて毎回組み替えない。

## 見た目

- Abletonから借りるのは配色だけでなく、意味のまとまり、揃った外端、内部の小さなグリッド、罫線と近接の階層。
- 操作中パネルだけに明灰色の外枠。採色した`#ACACAC`、現在採用幅1 logical px。非アクティブのパネルへ同じ枠を付けない。
- 現在の反復基準は行20px、名前11px、レーン境界2px、色面・隣接ボタンの上下余白2px。参考製品の内部仕様の断定ではなく、実画面で再調整できる基準として扱う。
- タブ・行は低く、文字は行内でY中央。名前は左、数値は右に揃える。大見出しや重複見出しで作業面を削らない。
- 所属色は名前欄・クリップ・対象識別へ不透明色として使う。選択、有効状態、所属を混同しない。色だけに依存せず名前・記号・位置を残す。
- 右クリックは暗い面、明るい細枠、低い行、水色ホバー。操作名は左、既存ショートカットは右。参照画像にある未実装機能を動く項目として偽装しない。
- 仮置きUIは許可されているが、実装済みと区別できる状態にする。今回使った寸法や自動色パレットの全てが永久仕様になったわけではない。

## Timelineと階層

- 実レーンだけに横線を引き、名前欄まで境界を通す。空領域は縦グリッド。クリップ内の重複名札は不要。
- 選択表示はユーザーが選んだレーンが単位。親レイヤー参照と、プロパティ行の選択を分ける。親を選んだから全プロパティを選択色にしない。
- Mはミュート、Sはソロ、Lはロック。意味のある記号を装飾の丸へ置換しない。
- Mの左の◇が、全レイヤー共通のキーフレームレーン開閉。通常はキーがある行だけ。右クリックから全プロパティ表示／非表示も選べる。
- グループの開閉は子レイヤーの表示。グループ自身のキーフレーム開閉とは独立する。
- グループは再帰的な容器。閉じた時は子の色と時間範囲を要約し、開いた時は同じ配置構造から背景・境界・子の配置・ヒット領域を作る。行へ後付けの線を重ねて帳尻を合わせない。
- 深い階層では名前欄全体を伸ばし、操作列・時間軸・ヒット判定も追従する。ユーザーが境界をドラッグして幅を調整できる。最小は通常レイヤー基準、最大は階層分を加えた幅、両端に多少の遊び。閉じる時も下限を守る。
- 名前行のドラッグは並べ替え。境界は挿入線、グループ中央は中へ入れる囲い。離した時だけ確定し、複数選択の順と子孫を保つ。自分の子孫への移動は拒否。

## 親子・キー・編集の所有

- 親削除は全子孫と所属するキーをまとめて削除し、Undoでまとめて戻す。グループ解除は子を新しい親へ付け直して残す。
- コピー／複製は通常レイヤーを挟んだ孫も含める。内部の親子関係を保持する。
- 本文キー、個別プロパティキー、集約キー選択は別の対象。本文キーの削除や移動が、同時刻のPosition等へ波及してはいけない。
- ClipboardはDocumentとLayerの両方を識別する。別作品の同番号Layerを同じ対象と見なさない。
- 親変更はドロップ時の姿を保ち、その後は新しい親に従う。アニメーションする親を一律禁止しない。
- 現実装は平行移動ならアニメーション子の全Positionキーを一様補正し、時刻・補間・接線を保つ。回転／拡縮を伴う一般のアニメーション子補償は宿題であり、製品の「あえての制限」ではない。
- マスクの編集入口はTimelineのクリッピングマスク。Photoshop／CLIP STUDIO型の親子・隣接関係として扱い、通常の切り抜き入口をエフェクト扱いに戻さない。[詳細](../reviews/2026-09-05-timeline-clipping.md)。

## 入力と反復

- 入力欄のフォーカスと編集ショートカットを取り合わない。標準入力部品を使い、見えない状態で同じ操作の意味を変えない。
- レーン上のトラックパッドは縦横移動とピンチ。時間目盛り上の上下スクロールは時刻を中心にズーム。通常移動・スクロールズーム・ピンチは同じ慣性処理を通す。
- 指を離した後は減速し、次の操作で止まる。表示移動は作品・キー・再生位置を書き換えない。変換とreleaseの処理を別々に追加して慣性を落とさない。
- UIはhot reloadで反復し、Document/GPU資源を保つ。Rust更新・初回buildは必要な時だけ。hot restartで失う表示状態は隠さない。
- 薄く広く制作可能にすることを優先する。試験の量・型分割・ビルド成功だけを完成とせず、ユーザーが触った結果を受けて修正する。

## 2.5Dの比較基準 — 2026-09-06

利用者の比較指示により、カメラ位置とレイヤー中心の方向から姿勢を変える自動回転を外した。2.5Dは当面、通常の3Dと同じ透視投影へ authored world geometryを渡す。位置変更だけで姿勢を自動補正せず、奥行き視差とカメラの回転は投影で表れる。これは立体を画像へ畳む処理ではない。2Dのフレーム基準の変換は維持する。

2.5Dという独立モードの最終的な意味を固定した宣言ではなく、余計な補正なしで使い心地を比較するための基準。フレーム固定・カメラ追従・合成時の畳みを同じスイッチへ混ぜない。

### 2Dの位置不変性

2Dはフレーム基準。Positionだけを変えた場合、輪郭・傾き・大きさは保ち、同じ絵が移動する。傾いた面や奥行きのある頂点でも、画面全体の消失点から位置依存の変形を受けないよう、レイヤー中心の投影とフレーム上の配置を分ける。変換は非退化なままで、Zを潰す平面化は行わない。描画・ギズモ・ヒット判定は同じ補正関数を利用する。

### Camera layer

Create → Camera adds a non-rendering camera layer with the current Center, Zoom and Roll. These are ordinary layer properties and use the same keys, preview, lifetime, save and Undo routes. At each frame, the highest visible camera within its duration is active; solo cameras take priority without hiding artwork. With no active camera, the existing composition camera remains the fallback. This entry exposes the current camera model; orbit/target cameras and camera export to Lottie are not implemented.

### User Stage / Camera View

Stage switches between User Stage and Camera View. User Stage uses a runtime-only observation camera (right-drag orbit); it does not author camera properties or change export. Camera View uses the active document camera. Camera bodies and frusta are selectable wire overlays only in User Stage. Both modes share the renderer and IOSurface route; layer projection uses the authored camera, while bounds and pointer projection use the displayed camera.

### Panels, Settings and Desk

Panels share one catalog, body implementation and placement route. Settings owns the complete panel list and placement controls. Normal panels support tab, separate window and hidden; auxiliary tools additionally support Desk. Stage, Timeline, Inspector, browsing collections, Notes and Web remain independent surfaces. Desk contains Depth, Ease, Blend and History: tools used for a while, between a popup and a permanent panel. A user can promote these tools to permanent surfaces through Settings. Desk is not the owner or manager of all panels.

Desk owns its idle default as well as contextual presentation. Settings changes placement only; returning a tool to Desk does not force it open. Desk reads selection/focus to present tools still assigned to it: keys → Ease, Blend focus → Blend, camera → Depth. It never duplicates a tool made permanent or unmounts one because of an internal interaction. Normal docking remains available. The main workspace owns physical placement; native windows receive identities without duplicating the catalog. Document content and saved workspace drafts are independent of hosting. Notes pages, positioned text/images and layer/time references belong to `Document::SetNotebook`, with Undo and save/reopen. Image bytes are embedded. Text and Reference are unified in Notes; prior saved data can be imported. Editors flush before save, move and window close.

Notes follows the [OneNote note-container interaction](https://support.microsoft.com/en-us/onenote/onenote-help-and-learning/work-with-note-containers): click empty space and type, drag a block by its top handle, resize its corner, paste/insert/drop images, and switch named pages. Web opens a regular browser tab from a persisted URL; Pinterest is the initial address. Pinterest API integration and embedded WebKit are not part of this change.

Ease follows `motolii/src/ui/ease.rs`, `ease_model.rs`, and `ease_widget.rs`: nine interpolation kinds, native handle semantics/canonical samples, square fixed-range plot, playhead, target/mixed state, curve copy and keyboard preset navigation. Release/preset activation is one grouped edit; cancellation, target changes and foreign pointers cannot commit a gesture. A terminal key alone has no outgoing interval. Target-free curves remain workspace data.

Validation (2026-09-06): all 13 catalog panels share tab/window/hidden placement without duplicates; the auxiliary subset additionally traverses Desk. Desk-scope tests keep normal panels out of the drawer catalog and exercise promotion through Settings. Document tests cover Notes Undo, invalid layout rejection and save/reopen. Real-window Notes checks cover text, image insertion/movement and page switching. Real-window Ease checks cover Bounce application, handle changes and one-step Undo. Depth retains its view when selecting a contained object. See [panel placement](panel-placement.md) for the shared model.
