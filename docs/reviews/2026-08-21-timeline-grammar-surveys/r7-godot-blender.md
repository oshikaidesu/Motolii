# Godot / Blender Timeline 操作意味論調査(裁定149 / R7)

対 `/Users/member_ottoto/rust_ae/Motolii/next/reference/timeline-grammar.md`(裁定144正典)の網羅性材料。判定は正典本文を通読した上での **既載/抜け/対象外/保留** の4値。

## 出典

### Godot(MIT — コード引用・移植候補の指摘まで可)
- リポジトリ: `github.com/godotengine/godot`, `master` ブランチ, HEAD `9ba32b09e0dfa4a6c1b82312554894615c716cce`(取得時刻 2026-08-21 相当のAPI応答、`editor/animation/animation_track_editor.cpp` 自体の最終更新コミットは `b4ebc1cd682cd8ac08d5c2b87a0cab4beefa570a` 2026-07-18)
- 取得方法: `gh api repos/godotengine/godot/contents/<path>?ref=master` でファイル単位に取得(リポジトリ全体は巨大なため個別ファイル取得、フルclone代替)。ローカル保存先: `scratchpad/timeline-grammar/godot-src/{animation_track_editor.cpp, animation_track_editor.h, animation_bezier_editor.cpp, animation_bezier_editor.h}`
- 対象ファイル: `editor/animation/animation_track_editor.cpp`(9,984行, クラス `AnimationTrackEdit` / `AnimationTrackEditor` / `AnimationMarkerEdit` / `AnimationTrackKeyEdit` / `AnimationMultiTrackKeyEdit`)、`editor/animation/animation_bezier_editor.cpp`(2,745行, クラス `AnimationBezierTrackEdit`)
- 補助: 公式 docs([docs.godotengine.org/en/stable/tutorials/animation/introduction.html](https://docs.godotengine.org/en/stable/tutorials/animation/introduction.html))は概説のみでドラッグ/スナップ/矩形選択の正確な意味論を欠くため、本調査の一次資料はソースコードが主

### Blender(GPL — 意味の記述のみ、コード引用禁止)
- 公式 manual `docs.blender.org/manual/en/latest/`(ページ内表記は「Blender 5.2 LTS Manual」)。取得は `curl` + BeautifulSoup でナビゲーション部を除去し本文のみ抽出(ローカル保存: `scratchpad/timeline-grammar/blender_*.txt`)
- 参照ページ: `editors/dope_sheet/{introduction,editing}.html`、`editors/graph_editor/{introduction,fcurves/editing,channels/editing}.html`、`editors/nla/{introduction,strips,editing/strip,editing/track}.html`、`video_editing/edit/montage/{editing,selecting}.html`、`animation/keyframes/editing.html`、`interface/{selecting,operators}.html`、`scene_layout/object/editing/transform/move.html`
- 以下はすべて意味論の記述であり、コード・原文の逐語転記は行っていない(見出し語・ショートカット表記は固有名詞として引用)

---

## 総論: Blenderの「モーダル操作」文法とAE/Motolii系「ドラッグ文法」の対比(最重要論点)

Blenderの変形操作(移動 `G` / 回転 `R` / 拡大縮小 `S` / Extend `E` / Slide `Shift-T` など)は **2つの起動経路が同じ1個のモーダルオペレータに合流する** という構造を持つ。

1. **キー起動(モードに入る)**: `G` を押す → ボタンを押していなくても選択物がマウス位置に追従し続ける「移動モード」に入る → 移動が終わったら **明示的にクリック(LMB)または Enter で確定、Esc または RMB でキャンセル**(出典: [`interface/operators.html`](https://docs.blender.org/manual/en/latest/interface/operators.html) "Modal Operators": `Esc`/`RMB` = Cancels a modal operator. `Return`/`LMB` = Confirms the action of a modal operator。および [`scene_layout/object/editing/transform/move.html`](https://docs.blender.org/manual/en/latest/scene_layout/object/editing/transform/move.html): "Pressing G activates "Move" transformation mode. The selected object...moves freely according to the mouse pointer's location...To confirm the action, press LMB.")
2. **ドラッグ起動(押している間だけ)**: キーフレームを直接クリックしたままドラッグする — こちらはマウスボタンを離した時点で暗黙に確定する、AE/Motolii と同じ「押している間だけ」文法(出典: [`editors/dope_sheet/introduction.html`](https://docs.blender.org/manual/en/latest/editors/dope_sheet/introduction.html) "Keyframes can be selected by clicking and moved by dragging."。VSE strip も同様: [`video_editing/edit/montage/editing.html`](https://docs.blender.org/manual/en/latest/video_editing/edit/montage/editing.html) "It is possible to move strips using mouse by dragging them while holding LMB.")

**この2経路は同一のオペレータに帰着する**(同じ Move/Scale/Slip の内部処理を共有し、どちらの起動でも同じ Shift=precision / Ctrl=coarse や Snap の挙動が適用される)。つまりBlenderは「押している間だけ」と「モードに入る」を**同じ操作の2つの入口として両立**させている。これはMotoliiの拘束(「掴んだ瞬間に単独選択へ差し替え」「確定はマウスアップ」§2、「1ドラッグ=1確定単位」拘束2)が暗黙に前提する「操作=ボタンを押している区間」というモデルとは異なる。Motoliiがキー起動モーダル(オプション1)を採らないのは(egui版に無い・拘束6でショートカットは意味アクションに後接続する設計のため経路自体は塞がっていない)自然だが、**確定/キャンセルの語彙(LMB/Enter=確定・Esc/RMB=キャンセル)がBlenderの全モーダル操作に共通する一枚岩の規約になっている点**は正典の「Esc=ドラッグ中キャンセル」(§2)が clip 専用記述になっていて他面(キー・並べ替え・ループ帯)に明記されていないのと対照的 — 下記 判定 B2 参照。

もう一つの「モードに入る」実例が **矩形選択(`B`)** と **Circle Select(`C`)**: `B` はキーを押した後にドラッグを始める2段階起動で、Circle Select に至っては **ドラッグを終えてもモードが持続し、次のドラッグを`C`を押し直さず開始できる**(出典: [`interface/selecting.html`](https://docs.blender.org/manual/en/latest/interface/selecting.html) "Once activated, Circle Select stays active: you can release the mouse button and start dragging somewhere else without having to press C again...To deactivate the tool again, press RMB, Return, or Esc.")。これは「1ジェスチャ=1確定」というMotoliiの前提そのものが通用しない領域で、対象外として明記する価値がある(判定 B14)。

---

## Part A: Godot AnimationPlayer / Track Editor

Godotの `Animation` リソースには **clip(bar)という単位が存在しない**。トラック上に生キーフレームが直接並ぶだけで、開始・終了時刻を持つ「素材の一部を切り出した矩形」という概念自体がない。したがって正典 §2(クリップ/bar操作: move・trim・split・Esc取消)の**大半はGodotに構造的に対応物がない**(対象外、拘束1文脈というより構造差)。Godotが厚いのは §3(キーフレーム)・§4(選択)・§1.6(折りたたみ)・§5(マーカー・ズーム)相当の層。

### 採集した操作

**A1. キー時刻ドラッグ(移動)**
起動: キーの当たり矩形(表示アイコン幅の**2倍**、"Make it a big easier to click" — `animation_track_editor.cpp:2574-2582` `get_key_rect()`)を押下。未選択キーなら即選択+ドラッグ準備(`moving_selection_attempt=true`)。**既に選択済みのキーを掴んだ場合は選択を変えずそのままドラッグ準備に入る**が、実際に動かさず放した場合のみ単独選択に絞り込む(`select_single_attempt`、`animation_track_editor.cpp:3286-3298` と `3416-3444`)。
ドラッグ中の意味: 横方向のみ(時刻)。`snap_time()` で丸め、Shift 押下中は丸め粒度を0.25倍に細かくする(`animation_track_editor.cpp:7940-7957`)。
確定: マウスアップ。**キャンセル: ドラッグ中の右クリックで `move_selection_cancel`**(`animation_track_editor.cpp:3300-3304`)。Escでのキャンセルは同関数内に見当たらない(Godotの取消はRMB限定)。
出典: `editor/animation/animation_track_editor.cpp:3096-3379`, `3382-3456`

**A2. 矩形選択(キー)**
起動: スクロール領域(`_scroll_input`)内で左ボタン押下(`animation_track_editor.cpp:6446-6519`)。**明示的な最小ドラッグ距離の閾値は無い** — 最初の mousemove イベントで即座に box が可視化される。可視化された瞬間、Ctrl/Shiftが押されていなければ既存選択を即座にクリアする(= 経路によって「クリックのみ」と「選択枠になる」の分岐点が"最初の1px動いた瞬間")。
確定: マウスアップ時、可視状態だった場合のみ各トラックへ `append_to_selection` を委譲(**折りたたまれた=非表示トラックはスキップ**)。
出典: `editor/animation/animation_track_editor.cpp:6446-6519`

**A3. スナップの二重トグル+一時反転**
Godotはスナップを **timeline(スクラブ)用とkeys(キー/ストリップ)用の2つの独立ボタン**に分けている。`is_snap_timeline_enabled()` / `is_snap_keys_enabled()` はそれぞれ「トグルボタンの状態 **XOR** Ctrl押下」で決まる(`animation_track_editor.cpp:5144-5151`)。既定状態がONならCtrlで一時OFF、既定がOFFならCtrlで一時ONになる — Motoliiの「Alt常時有効→Altで切る」非対称ではなく **対称なXOR反転**。さらにドラッグ中Shift押下でスナップ粒度そのものを0.25倍に細かくする(無効化ではなく解像度変更、A1参照)。
スナップ対象は秒/フレームの**固定グリッドのみ**(`snap_time()` は `Math::snapped(value, snap_unit)` — 他キー・マーカー・playheadへの吸着スキャンは行わない、`animation_track_editor.cpp:7940-7957`)。秒⇔フレーム切替はヘッダの `OptionButton`(項目 "Seconds"/"FPS"、`animation_track_editor.cpp:8373-8375`)。フレーム未満の秒刻みを選んだ場合に「実際のFPSに丸め直す」独立トグル `fps_compat` もある(`_update_snap_unit()`, `animation_track_editor.cpp:7914-7937`)。
出典: `editor/animation/animation_track_editor.cpp:5144-5151, 7914-7957, 8329-8375`

**A4. ベジエ・ハンドルモード(4種)とドラッグ**
起動: ベジエトラックの各キーには in/out 独立の当たり矩形があり(`edit_points[i].in_rect` / `out_rect`)、ドラッグでハンドル角度・長さを変更(`animation_bezier_editor.cpp:1668-1690`)。モードは **Free(独立)/ Linear(直線=ゼロ相当)/ Balanced(対向方向・長さは各ハンドル自前)/ Mirrored(対向方向・長さも同一)** の4種(`animation_track_editor.cpp:636`, 実装 `animation_bezier_editor.cpp:2242-2291`)。
Ctrl+空白クリックで**その場に新規ベジエキーを即挿入し、間髪入れずドラッグ移動待機状態に入る**(`animation_bezier_editor.cpp:1737-1770` — 挿入と移動が1ジェスチャに連結)。
出典: `editor/animation/animation_track_editor.cpp:520-636`, `editor/animation/animation_bezier_editor.cpp:1660-1770, 2242-2291`

**A5. 選択矩形の拡縮ハンドル(ベジエエディタ限定)**
複数キー選択時、その包絡矩形(時間×値)の四隅・四辺に**ドラッグ可能な拡縮ハンドル**が現れ、ピボットを基準に選択キー群の時刻と値を**同時にスケール**できる(`animation_bezier_editor.cpp:1693-1721`)。RetimeSelectionのような修飾キー+端キードラッグではなく、**可視のバウンディングボックスUI**によるスケール。
出典: `editor/animation/animation_bezier_editor.cpp:1657-1734`

**A6. トラック(グループ)折りたたみ + 折りたたみ時のキー概観オーバーレイ**
起動: 同一ノードパスを共有する複数トラックがグループヘッダ行にまとめられ、左端の矢印アイコン領域クリックで開閉(`animation_track_editor.cpp:3968-3990`、状態は `Animation::editor_set_group_folded` でアニメーションリソースにエディタメタデータとして永続化)。
**フィードバック**: 折りたたむと、子孫全トラックの全キーが**半透明のアイコンとして折りたたみ行の上に元の時刻位置のまま重ね描画**される(`animation_track_editor.cpp:3938-3955`)。「今どこにキーがあるか」を1行に集約して見せる。
出典: `editor/animation/animation_track_editor.cpp:3916-3990`

**A7. 複製(Ctrl+D)= playhead 位置へ再配置**
ショートカット `Cmd/Ctrl+D`(`animation_track_editor.cpp:8434`)。複製先の基準時刻は**現在のplayhead位置**(選択キー群の最も早い時刻をplayheadに合わせ、他は相対距離を保って追従)。ドラッグでの再配置は伴わない即時操作。
出典: `editor/animation/animation_track_editor.cpp:6617-6724`

**A8. 複数トラック横断編集(Inspector連携)**
複数トラック・複数キーを選択すると `AnimationMultiTrackKeyEdit` が Inspector に**共通編集可能なプロパティだけ**を出す(時刻は常に共通、`easing` は値系トラックのみ共通提示、`TYPE_ANIMATION` 系は複数選択時は非表示、など型ごとの積集合ロジック)。
出典: `editor/animation/animation_track_editor.cpp:1200-1290`(クラス定義 `animation_track_editor.h:98`)

**A9. ダブルクリックの意味重複(反面教師)**
トラック行内でのダブルクリックは、それが偶然キーに当たっていようといまいと **常に** playhead シークを追加発火する(`if (mb->is_double_click() && !moving_selection...) { emit_signal("timeline_changed", ...) }`、キー選択判定より前に評価される、`animation_track_editor.cpp:3096-3103`)。1回目の押下がキー選択、2回目が「選択+シーク」の意味を持つため、2回目の押下の意味が文脈依存になる。
出典: `editor/animation/animation_track_editor.cpp:3095-3103`

**A10. ズームのアンカー方式がホイールとスライダで異なる**
マウスホイールでのズームはカーソルのX位置をアンカー(`zoom_scroll_origin`)。ヘッダのズームスライダをドラッグした場合は**playheadが可視範囲内にあればplayheadをアンカー**、範囲外または中央なら画面中央をアンカーにする、という異なる規則(`animation_track_editor.cpp:1288-1330` 付近 `AnimationTimelineEdit::_zoom_changed`)。
出典: `editor/animation/animation_track_editor.cpp:1275-1335`

**A11. Inspectorからの直接キー打ち**
Inspectorの各プロパティ行に小さな鍵アイコンが出て、クリックでトラック新設+現在値でのキー追加ができる(公式docs [`tutorials/animation/introduction.html`](https://docs.godotengine.org/en/stable/tutorials/animation/introduction.html))。Timelineペイン自体の操作ではなくInspector連携。

### Godot 判定

| # | 操作 | 判定 |
|---|---|---|
| A0 | clip/bar概念自体が無い(生キーフレームのみ) | **対象外**(拘束1文脈=構造差。§2のmove/trim/split/Esc取消はGodotに直接対応物なし) |
| A1 | キー時刻ドラッグ・押下座標での即選択差し替えなし(離した時に単独化) | **抜け**(§2「掴んだ瞬間に単独選択へ差し替え」はclipのみの記述で、既選択キーを掴んだ場合の扱いが正典に無い。Godotは「離すまで選択を変えない」= AE/Photoshop系の一般的な複数選択ドラッグ規約) |
| A1 | ドラッグ中の右クリックキャンセル | **抜け**(§2はEscのみを規定。GodotはRMBでも移動を取り消せる。Blender側B2と合わせて2ソース独立に確認できた候補) |
| A2 | 矩形選択の最小閾値=事実上ゼロ | **保留**(§7-6の開問に対する一次資料。Godotは明示閾値を持たない側の実例) |
| A3 | timeline用/keys用の独立スナップトグル | **抜け**(正典はスナップを単一のクリップ面の仕組みとして扱い、スクラブ用スナップの区別が無い) |
| A3 | Alt/Ctrlの一時反転がXOR(対称) | **抜け**(正典はAlt常時無効化の非対称。GodotのXOR設計は「既定OFFでもCtrlで一時ON」ができる点で拡張的) |
| A3 | Shiftドラッグ中でスナップ粒度を細かくする(無効化ではない) | **抜け**(正典に無い第3の軸。無効化キーAltと共存できる設計) |
| A4 | ベジエハンドル4モード(Free/Linear/Balanced/Mirrored) | **抜け**(§7-1の開問「イージング変更が全paramへ開くか」に効く具体語彙。§3の「候: 菱形の形でイージング状態を語る」に接続可能) |
| A5 | 選択の包絡矩形に可視スケールハンドル(時刻+値を同時スケール) | **抜け**(§3 RetimeSelectionは時間のみ・修飾キードラッグ。可視ハンドルUIで値まで同時操作する発想は正典に無い) |
| A6 | 折りたたみ行への子孫キー概観オーバーレイ | **既載**(§1.5「クリップ上の余白は将来のキーフレームオーバーレイのために空けておく」の意図に一致する実装済み先例。折りたたみ時の集約表示という具体形は§1.5の想定より進んでいるため、実装時の参考として太字候補) |
| A7 | Ctrl+D複製はplayhead位置へ再配置 | **抜け**(§4 Duplicateは配置位置を規定していない。決定漏れの穴を埋める具体候補の1つ) |
| A8 | 複数トラック横断選択時のプロパティ積集合編集 | **保留**(Inspector連携でTimelineペイン外。Motoliiが同種パネルを持つ場合の参考) |
| A9 | ダブルクリックが常にシークも兼ねる | **対象外**(拘束4の反例そのもの。採用しないことの正しさを補強する実例として記録) |
| A10 | ホイールズームとスライダズームでアンカー規則が違う | **候**(§5のCmd+ホイールアンカーズームは既載。ナビゲータ帯knobドラッグのズームにplayhead優先アンカーを足すかは未検討) |
| A11 | Inspectorからのキー追加ボタン | **対象外**(Timelineペインの外側の操作) |

---

## Part B: Blender Dope Sheet / Graph Editor / NLA / VSE

### 採集した操作

**B1. モーダル移動(G)とドラッグ移動の二重起動** — 総論参照。
出典: [`scene_layout/object/editing/transform/move.html`](https://docs.blender.org/manual/en/latest/scene_layout/object/editing/transform/move.html)、[`editors/dope_sheet/introduction.html`](https://docs.blender.org/manual/en/latest/editors/dope_sheet/introduction.html)、[`video_editing/edit/montage/editing.html`](https://docs.blender.org/manual/en/latest/video_editing/edit/montage/editing.html)

**B2. 全モーダル操作共通の確定/取消キー**
`Esc` / `RMB` = キャンセル、`Return` / `LMB` = 確定。これは Move・Scale・Extend・Slide・Blend系・Smooth系・Slip Strip Contents など**ほぼ全てのモーダルオペレータに共通する一枚岩の規約**(出典: [`interface/operators.html`](https://docs.blender.org/manual/en/latest/interface/operators.html) "Modal Operators")。個別ページでも都度 "press LMB to confirm (or RMB to cancel)" と明記される(例: [`editors/graph_editor/fcurves/editing.html`](https://docs.blender.org/manual/en/latest/editors/graph_editor/fcurves/editing.html) Extend / Blend / Smooth 各項)。

**B3. Snap: 絶対/相対の切替を持つ2段構え**
トグルボタンで有効/無効。ドロップダウンで **Snap To**(Frame / Second / Nearest Marker)を選択。加えて **Absolute Time Snap** という第3のチェックボックスがあり、OFFなら「ドラッグ量をSnap To単位の**増分**で丸める(元のサブフレームオフセットは保持)」、ONなら「**絶対値**としてSnap To単位の倍数へ丸める(サブフレームオフセットは消える)」という質的に異なる2つの丸めモードになる(出典: [`editors/dope_sheet/editing.html`](https://docs.blender.org/manual/en/latest/editors/dope_sheet/editing.html) "Snap" 節、同型の記述が [`editors/nla/introduction.html`](https://docs.blender.org/manual/en/latest/editors/nla/introduction.html) "Snap" 節にもある)。

**B4. Box Select / Circle Select のツール二重人格**
ツールバーで選ぶ変種(`Select Box`, `W`)はドラッグそのものが選択矩形になる。メニュー変種(`Box Select`, `B`)はキー押下後にドラッグを始める2段階起動で、既定動作が「追加選択」(既存選択を消さない)という点でツールバー変種と挙動が違う。Circle Selectの変種はドラッグ終了後もモードが持続し、`Esc`/`RMB`/`Return`で明示的に抜けるまで有効(出典: [`interface/selecting.html`](https://docs.blender.org/manual/en/latest/interface/selecting.html))。

**B5. Extend(E)— playhead基準の片側一括移動**
選択後、playheadの左右どちらかにマウスを置いて `E` を押すと、**その側にある選択キー/ストリップだけ**がマウス追従で移動する(Dope Sheet: [`editors/graph_editor/fcurves/editing.html`](https://docs.blender.org/manual/en/latest/editors/graph_editor/fcurves/editing.html) "Extend"、NLA: [`editors/nla/editing/strip.html`](https://docs.blender.org/manual/en/latest/editors/nla/editing/strip.html) "Extend"、VSE: [`video_editing/edit/montage/editing.html`](https://docs.blender.org/manual/en/latest/video_editing/edit/montage/editing.html) "Move/Extend from Current Frame")。ストリップがplayheadを跨ぐ場合は跨ぐ側の端点だけが動く。

**B6. Dope Sheet Slide(Shift+T)— 中央分割・両側伸縮**
3つ以上のキーを選択し、選択範囲の中間にカーソルを置いて `Shift-T`。カーソル位置で選択範囲を一時的に2分割し(破線で表示)、マウス移動で片方を伸ばしもう片方を縮める(**合計の長さは不変**)。確定LMB/取消RMB(出典: [`editors/dope_sheet/editing.html`](https://docs.blender.org/manual/en/latest/editors/dope_sheet/editing.html) "Slide")。

**B7. VSE Overlap Mode(Expand / Overwrite / Shuffle)**
ストリップ移動でオーバーラップが生じた時の挙動を3択で規定: Expand=右側を押し出す(リップル相当)、Overwrite=重なった相手をトリム/分割して上書き、Shuffle=重ならない最寄りの空きへ自動退避(出典: [`video_editing/edit/montage/editing.html`](https://docs.blender.org/manual/en/latest/video_editing/edit/montage/editing.html) "Overlap Mode")。

**B8. VSE Slip Strip Contents(S)**
ストリップの位置・長さは変えず、内部で参照する素材の開始点だけをずらす。Shift押下でサブフレーム精度、`C`でクランプ切替、確定はLMB/Return/Space、取消Esc/RMB(出典: 同上 "Slip Strip Contents")。

**B9. VSE Retiming Keys**
ストリップ内部に打てる専用キー群で、キーをドラッグすると**そのセグメントだけ**時間の伸縮(≒速度変更)が起きる。ストリップの開始・終了の境界にあるキーは常に存在必須。ボックス選択は「既に1つキーが選択されている場合のみキーを拾い、そうでなければストリップを拾う」という文脈依存の優先順位を持つ(出典: 同上 "Retiming Keys")。

**B10. Graph Editor Blend系(値領域の一括加工)**
Breakdown / Blend to Neighbor / Ease / Push Pull / Shear Keys / Scale Average / Scale from Neighbor / Time Offset — いずれも「メニュー項目をクリック→マウス左右でファクタ調整→LMB確定/RMB取消」というスライダーモーダルの型を共有する値(Y軸)側の一括編集群(出典: [`editors/graph_editor/fcurves/editing.html`](https://docs.blender.org/manual/en/latest/editors/graph_editor/fcurves/editing.html) "Blend" 以下)。

**B11. Channel Grouping(Ctrl+G / Ctrl+Alt+G)**
選択チャンネルを名前付きの折りたたみ可能な集合にまとめる。ダブルクリックでリネーム。ノードの親子構造とは無関係な、**純粋にエディタ上の整理用**のグルーピング(出典: [`editors/graph_editor/channels/editing.html`](https://docs.blender.org/manual/en/latest/editors/graph_editor/channels/editing.html) "Un/Group Channels")。

**B12. NLA Track Move とStrip Move Trackの同一ショートカット・文脈分岐**
`Track ‣ Move`(トラック自体をPageUp/PageDownで上下入れ替え)と `Strip ‣ Transform ‣ Move Up/Down`(ストリップを別トラックへ移す、同じPageUp/PageDown)が**マウスカーソルがトラック領域の上にあるかストリップ領域の上にあるかだけ**で意味を変える(出典: [`editors/nla/editing/track.html`](https://docs.blender.org/manual/en/latest/editors/nla/editing/track.html) "When using the keyboard shortcuts, make sure the mouse cursor is hovering over the track region...")。

**B13. VSE Box Select (Include Handles)(Ctrl+B)**
通常の Box Select(ストリップ本体を拾う)と違い、矩形内にストリップの**トリムハンドル**が入っていればハンドル単体を選択できる。片側ハンドルだけ選択されればそのハンドルをドラッグしてトリム、両ハンドルならストリップ全体移動になる(出典: [`video_editing/edit/montage/selecting.html`](https://docs.blender.org/manual/en/latest/video_editing/edit/montage/selecting.html) "Box Select (Include Handles)")。

**B14. Tweak / Select Box ツールの二者択一**
既定ツールが "Tweak" のときは要素上のドラッグは**移動**になり、"Select Box" ツールに切り替えると同じドラッグが**矩形選択**になる。ヒットテストによる暗黙分岐(AE/Motolii型)ではなく、**左のツールバーで選んだ現在のツールが全体の解釈を決める**モード切替方式(出典: [`interface/selecting.html`](https://docs.blender.org/manual/en/latest/interface/selecting.html) "Toolbar Selection Tools")。

**B15. Keyframe Insert の3経路**
`I`(既定チャンネルまたは Keying Set に従って自動挿入)/ プロパティ上で `I` または右クリック→ Insert Keyframe / `I` 長押しでパイメニュー(Location/Rotation/Scale/Available を選択)。Auto Keyframe(record ボタン)は値変更を検知した時だけ自動追加(出典: [`animation/keyframes/editing.html`](https://docs.blender.org/manual/en/latest/animation/keyframes/editing.html))。

### Blender 判定

| # | 操作 | 判定 |
|---|---|---|
| B1 | モーダル移動(Gキー起動、押していなくても追従・クリックで確定) | **対象外**(新規区分=モーダル操作様式そのもの。拘束2/5が前提する「ドラッグ=ボタンを押している区間」という設計方針と根本的に別系統。総論参照) |
| B1 | 直接ドラッグ(押している間だけ追従・離して確定) | **既載**(§2 move/§3 時刻ドラッグと同型。AE/Motolii系と一致する側の入口) |
| B2 | Esc/RMB=取消、Return/LMB=確定という全モーダル共通規約 | **抜け**(§2はEscのみをclip dragに限定して明記。RMBでも取消できる点、かつ「全ての面で同じ確定/取消語彙を使う」という一般化がGodot A1でも独立に確認できた=2ソースで補強される具体候補) |
| B3 | Absolute Time Snap(相対増分 vs 絶対グリッドの二値) | **抜け**(正典のスナップは常に絶対位置への吸着を前提。「元のサブフレームオフセットを保ったまま増分だけ丸める」という第3のスナップ挙動は正典に無い軸) |
| B4/B14 | ツール依存の入力解釈切替(Tweak⇄Select Box、Circle Selectの持続モード) | **対象外**(構造差。Motolii/AEはヒット位置による暗黙分岐でツール切替を要求しない設計であり、その設計の正しさを補強する反例として記録) |
| B5 | Extend(E)— playhead基準の片側一括移動 | **候**(拒否済みのtrim family=ripple/roll/slip/slide/insert/overwrite/lift/extract/sync lockのいずれとも厳密には一致しない。「明示的に呼び出す・時間のみ・自動伝播ではない」点でGroup Extend系の§2候補「Quick Offset」に近い第4の案として検討価値あり。拘束1に抵触するかは要裁定) |
| B6 | Dope Sheet Slide(中央分割・両側逆伸縮で合計時間を保つ) | **抜け**(§3 RetimeSelection=範囲端キーの比例伸縮とは別物。「カーソル位置で分割点を選べる・両側が逆方向に伸縮する」という構造は正典に無い) |
| B7 | VSE Overlap Mode(Expand/Overwrite/Shuffle) | **対象外**(拘束1で名指しされたripple/overwrite/insert-shuffle系そのもの。採用しないことの妥当性を補強する実例) |
| B8 | VSE Slip Strip Contents | **対象外**(拘束1が明示的に列挙する"slip"そのもの) |
| B9 | VSE Retiming Keys(ストリップ内部の可変速キー群) | **抜け**(§7-5「入れ子の時間伸縮」の開問に効く具体案。単一の一様スケールでなく、ストリップ内で複数キーによる区間ごとの可変速という形は正典未検討) |
| B10 | Graph Editor Blend系(値域の一括加工、スライダーモーダル) | **保留**(Timelineペインではなく値カーブ編集面の操作。Motoliiが同種の値カーブ編集面を持つか未定のため判定を保留) |
| B11 | Channel Grouping(構造と無関係な整理用グループ) | **抜け**(§1.6の「グループ階層」はegui版のレイヤー構造=実データの親子を参照する想定。ノード階層と無関係な「見た目だけの整理用フォルダ」という発想は正典に無い) |
| B12 | Track MoveとStrip Move Trackがマウス位置文脈で同一ショートカットの意味を変える | **対象外**(§5.5「場所で意味が変わるホイールは予告できない」の精神をキーボードショートカットにも一般化して適用すべき反例として記録。Motoliiが既に拒否した設計の追加証拠) |
| B13 | Box Select (Include Handles) — 矩形内のハンドルを個別に拾う | **候**(§4 marqueeは現状「clipを選ぶ」専用。矩形選択がハンドル単体を拾える設計は§2 trimとの接続点になり得るが、「1ジェスチャ1意味」の簡潔さとのトレードオフ要検討) |
| B15 | Keyframe Insertの3経路(既定チャンネル/プロパティ個別/パイメニュー) | **対象外**(Timelineペイン外、プロパティパネル連携の話) |

---

## 集計

- Godot: 採集14項目 → 既載1・抜け7・対象外3・保留3
- Blender: 採集15項目 → 既載1・抜け5・対象外6・保留2(B1は「モーダル起動」を対象外、「直接ドラッグ」を既載として1項目内で2値評価しているため、行数と一致しない)
- 合計(単純合算): 既載2 / 抜け12 / 対象外9 / 保留5(29項目、B1の二重評価を含む)

## 抜け 上位5件(短報)

1. **A6: 折りたたみ行での子孫キー概観オーバーレイ**(Godot) — 正典§1.5が「将来のキーフレームオーバーレイ」として空けている余白の、折りたたみ時ケースにおける実装済み先例。半透明アイコンで子の全キー時刻を1行に集約表示する。
2. **A3+B3: スナップの多層化**(Godot/Blender共通で別軸) — Godotは timeline用/keys用トグル分離+Ctrl一時反転(XOR)+Shiftで粒度を細かくする(無効化ではない)。Blenderは絶対グリッド吸着と相対増分丸めの二値(Absolute Time Snap)。どちらも正典の「Alt常時無効化」単軸より表現力が高い。
3. **A1+B2: ドラッグ中キャンセルの右クリック対応**(Godot・Blender独立に確認) — 正典§2はEscのみをclip dragについて明記。2ソース独立にRMBキャンセルが標準搭載されており、キーフレーム面・並べ替え面等への一般化が推奨候補。
4. **A5: 選択包絡矩形の可視スケールハンドル**(Godot ベジエエディタ) — RetimeSelection(時間のみ・修飾キードラッグ)と別に、時間と値を同時にスケールする可視ハンドルUI。§3の候補群に無い新規語彙。
5. **B9: VSE Retiming Keys**(Blender) — ストリップ内部に置く可変速キー群による区間ごとの時間伸縮。§7-5(入れ子の時間伸縮)の開問に効く具体案で、単一の一様スケールより表現力が高い代替案。
