# 縫い目調査: 観測カメラ/レンダリングカメラ分離(裁定156)

読むだけの調査。設計判断はしない — 事実と選択肢のみ。

## 0. 前提の確認

- 裁定113/115/116 で実装済みなのは**レンダリングカメラ**(`Composition.camera` の
  center/zoom/roll)だけ。**観測カメラは現状どこにも無い** — Stage は常にレンダリング
  カメラの絵をそのまま CPU で貼っている(下記1〜3で実証)。
- 裁定156 の「レンダリングカメラ=Document のカメラレイヤー」は、新しい層概念ではなく
  **既存の `Composition.camera`(裁定113/115/116)がそれに相当する**と読める —
  読む対象1「Document 側カメラの現在の型」がまさにこれ。

## 1. 現状の縫い目地図(file:line)

### 1-1. Document 側(camera track・resolve)

- `next/core/motolii-store/src/document.rs:222-235`
  `PropertyId::camera(name)` — layer とは別 entity(`/composition`)、
  component 名前空間 `Composition:{name}`(layer 側 `Layer:{name}` と衝突しない)。
- `next/core/motolii-store/src/document.rs:340-345` `Intent::SetCameraTrack`、
  `:346-350` `Intent::SetCameraPropertySlot`、`:940` `:967` で `apply_intent` が処理、
  `:1036` `set_camera_transient` / `:1054` `clear_camera_transient`(drag-to-scrub overlay)。
- `next/core/motolii-store/src/view.rs:287-289` `StoreView::camera_track(property)` —
  `/composition` entity の track を読む(S1 の `resolved_effects`/`track` と同型の読み口)。
- `next/core/motolii-store/src/view.rs:397-439` `StoreView::resolve_camera(t)` —
  `camera.center`/`camera.zoom`/`camera.roll` を評価し `motolii_core::ResolvedCamera` を返す。
  track が無い property は既定値(裁定20 の静止値ルールをカメラにもそのまま適用)。

### 1-2. 投影の正本(GPU 非依存の数学)

- `next/core/motolii-core/src/camera.rs`(全体が正本、1箇所だけで組む設計)。
  - `ResolvedCamera{center,zoom,roll_degrees}`(:22-42) — **向きは固定軸**
    (常に world +Z を向く、裁定115「向きの表現はまだ開けない」)。
  - `camera_projection(comp, camera) -> CameraProjection{eye,rotation,vertical_fov,...}`
    (:85-118) — 投影の唯一の組み立て箇所。
  - `camera_screen_from_world_z0(comp, camera)`(:174-176) — z=0 平面でのカメラの写像
    (アフィン)。pinned layer 打ち消し用、**かつ画枠投影にも再利用できる**(1-4 参照)。

### 1-3. engine → compositor(消費)

- `next/engine/motolii-engine/src/lib.rs:114-120` `Engine::render_frame(view, t)` —
  **camera を引数に取らない**。`:157-159` で `view.resolve_camera(t)` を内部で読む
  (裁定40 の「comp も引数で取らない」と同じ規律:「preview/export が違うカメラを
  渡せないように」と doc comment に明記、:154-156)。
- `next/engine/motolii-engine/src/lib.rs:132-138`
  `Engine::render_frame_without_background(view, t)` — **同じ規律**で camera も
  view から読む。第二の render パスではなく同一合成器への入力差分(裁定141 と同型)。
  この precedent(「同じ合成器・入力だけ変える第二エントリ」)がそのまま
  観測カメラの実装パターンとして使える(§3 参照)。
- `next/engine/motolii-engine/src/lib.rs:261`
  `self.compositor.render_with_effects(comp, camera, &layers)` — camera はここで
  compositor へ渡るのが最後。
- `next/engine/motolii-compositor/src/lib.rs:310-318` `Compositor::render` /
  `:321-430` `render_with_timing(comp, camera, layers)` — `camera_projection`
  (:332)と `camera_screen_from_world_z0`(:335、pinned 用)を呼び、
  `TargetConfiguration{view_from_world, projection_from_view, ...}`(:397-427)
  へ詰め替えるだけ(compositor 自身は投影を計算し直さない)。

### 1-4. render_frame の呼び手は2箇所だけ(export と Stage 更新)

- `next/engine/motolii-export/src/lib.rs:123` `engine.render_frame(view, t)`。
- `next/shell/motolii-shell/src/lib.rs:1588` `self.engine.render_frame(&self.doc.view(), t)`
  (`refresh_frame` 内)。
- **この2箇所以外に `render_frame`/`render_frame_without_background` の呼び手は無い**
  (grep で確認)。両方とも `(view, t)` の2引数固定 — camera を渡す口がそもそも無い。

### 1-5. Stage 表示(観測カメラに相当する物は無い)

- `next/shell/motolii-shell/src/lib.rs:9` 「Stage は **CPU 経路**」。
- `next/shell/motolii-shell/src/lib.rs:1735-1768` `stage_pane` — `image(handle)`
  を `Length::Fill` で貼るだけ。pan/zoom/orbit の状態も入力ハンドラも無い。
- `next/shell/motolii-shell/src/lib.rs:1665-1681` `build_stage_handle` —
  `stage_handle_rgba`(縮小)+ 市松合成のみ。カメラ変換は無い。
- `next/shell/motolii-shell/src/lib.rs:118-140` `Session{playhead, selection,
  selected_keys, key_anchor}` — カメラ関連フィールドは無い。
- `next/shell/motolii-shell/src/screenshot.rs` の検分器具も、Stage が実際に貼る絵
  (市松合成後の `RenderedFrame::rgba`/`checkerboard_preview_rgba`)をそのまま
  貼り直すだけ — 独自視点は持たない。
- **結論**: 「Stage は常にレンダリングカメラの絵に正対している」— 裁定156 が
  観測カメラの不在を前提にしているのはこの実装事実と一致する。

## 2. 旧資産の意味(next/ には引かない前提の確認)

- `crates/motolii-ui/src/rerun_stage/document_camera.rs` +
  `docs/reviews/evidence/stage-document-camera/README.md`(2026-08-18): 旧
  egui/Rerun Stage の既定カメラを「document camera(正対)」に直した実装。
  旧正本の「view camera」と「document camera」という**呼称の起点**がここ。
  旧 crate のコードは 裁定3/CANON により next/ に引かない。
- `docs/reviews/2026-08-18-stage-interaction-concept-map.md` §4「2台カメラ」
  (未決の**観察**文書、決定ではない): 「視点カメラ = camera seat(fork済み、
  `SpatialStage::set_camera`)。書き出しに影響しない」/「書き出しカメラ =
  compositionの寸法・枠。document概念であり、export経路(Rerunを通らない)が
  実行する」。Blender viewport camera vs scene camera、AE comp viewport と
  同型と明記。**next/ の現状(1-4 の「export は render_frame だけを呼ぶ」)は
  この「export経路がビューアを通らない」という担保をそのまま体現している**
  — Rerun という第三者ビューアが無くなった今も、担保の構造(呼び出し経路を
  分ける)は同じ形で使える。
- `docs/reviews/2026-08-18-rerun-e0-composition-probe.md` §4(b)「カメラ注入は
  不成立」: Rerun fork へ外からカメラを注入しようとして seam が閉じていた
  (`injected_document_camera_maps_the_layer_onto_the_frame` 系の実測)。
  これが `wrapper-over-hack`(2026-08-18)の初出 — 「ハック的迂回を強いられたら
  自分の境界に素直な口を1本作る方を採る」。**観測カメラも同じ教訓が効く**:
  既存の `render_frame` へ後から camera 引数をこじ入れる(迂回)のではなく、
  裁定141 と同型の**別エントリ**を1本作るのが素直な口(§3 の切片案はこれを採る)。
- `next/core/motolii-core/src/lib.rs:7-13`: 旧 `crates/motolii-core/src/camera.rs`
  (561行、Spatial 変種)は「next/ から参照が0」で意図的に落としてある。裁定115が
  「向きの表現・handedness・特異点」を未解決枠として残した実体がここ。観測カメラで
  自由な向き(orbit)が要る設計になった場合の一次参照先だが、**今回は持ち込まない**
  (§4 で軽量案を優先する理由)。

## 3. 観測カメラの置き場の選択肢

| 選択肢 | 判定 | 根拠 |
| --- | --- | --- |
| Document(`Composition.camera` の隣に第二カメラ track) | 不採用の理由が明確 | 裁定156 の定義そのもの(「Document に乗らない」)に反する。undo/redo・書き出しJSON にも乗ってしまい、レンダリングカメラとの区別が型でなく「どっちの track か」という運用にしかならない。 |
| `Session`(`playhead`/`selection` と同格) | 弱い | `Session` は「編集の作業状態だが Document ではない」層だが、既存メンバ(再生位置・選択)は**編集意味を持つ**。観測カメラは編集意味を一切持たない純表示状態なので、`Session` に混ぜると「Session=編集状態」の一貫性が崩れる。 |
| `Shell` 直下のフィールド(`checkerboard: bool` と同格) | **既存 precedent と完全一致** | `next/shell/motolii-shell/src/lib.rs:388-390` の `checkerboard` が「表示専用 — Document には一切乗らない(書き出しに影響しない)」を doc comment で明言し、`Session` でも `Document` でもない置き場として既に実例がある。観測カメラもこの並びに置くのが最小の逸脱。 |

**推し**: `Shell` 直下(`checkerboard` と同じ並び)。理由1行: 「表示専用・
Document 非搭載」の precedent が repo に既にあり、観測カメラはその型を
そのまま複製できる。

### 書き出し経路が汚染されない構造の担保案

1-4 で確認した通り、`render_frame`/`render_frame_without_background` は
`(view, t)` の2引数固定で、呼び手は export と shell の2箇所だけ。**担保は
「camera を運ぶ第三の引数を足さない」ことだけで成立する** — 観測カメラを
`Engine` の**新しい別メソッド**(裁定141 と同型: 同じ compositor・同じ層構築、
入力だけ差し替える第二エントリ)として実装し、そのメソッドは shell の
`refresh_frame` からしか呼ばれない形にすれば、export クレート
(`next/engine/motolii-export/`)のコードは観測カメラの存在自体を知らずに
`render_frame` を呼び続けられる。grep 1本(「`render_frame` 系の呼び手が
export と refresh_frame 以外に増えていないか」)が回帰の柵になる
(`render_pipeline_fence.rs` に類する試験として追加できる)。

## 4. カメラビュー系 UI の挿入点

- **フレーム枠(セーフエリア含む)**: `camera_screen_from_world_z0(comp, camera)`
  (`next/core/motolii-core/src/camera.rs:174-176`)を**レンダリングカメラ**で呼べば
  comp の4隅の画面座標(観測カメラ視点での投影後座標)が出る — 新しい数学は要らない、
  既存の「投影の正本」を枠の描画にも再利用するだけ。挿入点は `stage_pane`
  (`next/shell/motolii-shell/src/lib.rs:1735-1768`)— `image` の上に重ねる overlay。
- **グリッド**: 同上の写像を格子点に適用するだけ(新規数学なし)。密度・色は意匠判断
  (このドキュメントの範囲外)。
- **「カメラを通して見る」トグル**: `ToggleSettingsPanel`/`ToggleCheckerboard`
  (`next/shell/motolii-shell/src/lib.rs:247-251`)と同型の `Message` variant を
  1本足すだけ。表示専用トグルの precedent がここにも既にある。
- **観測カメラの操作(pan/zoom/orbit の入力)**: `inspector_pointer_event`
  (`next/shell/motolii-shell/src/lib.rs:1692-1728`)が window 全体の pointer/keyboard
  event を拾っている唯一の subscription — 新しい drag ジェスチャ(Stage 上の
  orbit/pan)もここに合流させるのが既存動線と一致する。

## 5. 実装切片の割り案

観測カメラの「向き」は**固定軸のまま**(既存 `ResolvedCamera{center,zoom,roll}` を
そのまま複製・第二インスタンスとして使う)前提で切る。自由な orbit(向きの回転)が
要る設計になった場合は §2 の旧 `camera.rs`(561行、Spatial 変種)が別系統の切片に
なる — その判断は意味裁定側に委ねる(このドキュメントでは決めない)。

| # | 切片 | 領域(write-set) | 推定行数 | 判断の重さ |
| --- | --- | --- | --- | --- |
| S0 | `Engine` に観測カメラ専用の第二エントリを足す(裁定141 と同型: `render_frame`と同じ compositor・層構築、camera だけ`Option<ResolvedCamera>`で上書き可能にする。`render_frame`自体の署名は変えない) | `next/engine/motolii-engine/src/lib.rs` のみ | 40〜70行 | 低(precedent 踏襲。唯一の論点は「`render_frame`の2引数固定を絶対に崩さない」という柵をどう書くか) |
| S1 | `Shell` に観測カメラの状態(`checkerboard`と同格の非 Document フィールド)+ 「カメラを通して見る」トグルの `Message`/`update` 分岐を足す(レンダリングへの配線はまだしない) | `next/shell/motolii-shell/src/lib.rs`(新フィールド・新 Message のみ、既存関数は触らない) | 60〜100行 | 低〜中(precedent 踏襲だが、`ResolvedCamera`をどう複製するか=専用型かエイリアスかの小さな型判断) |
| S2 | `refresh_frame` を S0+S1 で配線する: トグル OFF の間は S0 の観測カメラで Stage 表示用 handle を作り、`RenderedFrame::rgba`(export 真値)は**従来通り無改造の `render_frame` から取り続ける**。cache スキップ条件(`frame.revision == revision && frame.playhead == playhead`、:1554)に観測カメラの等価性チェックを追加する(`checkerboard`が既にこの形で入っている、:1555 参照) | `next/shell/motolii-shell/src/lib.rs`(`refresh_frame`・`RenderedFrame`のみ) | 100〜150行 | 高(a. 毎フレーム二重 render のコスト判断 b. cache 無効化条件の見落としは「観測カメラを動かしても絵が変わらない」という無反応ゼロ違反に直結 c. `render_pipeline_fence.rs`が指摘した iced 2MB同期アップロード予算の flicker 再発リスク — drag 中に毎回 Handle 再生成すると同じ症状が起きうる) |
| S3 | Stage overlay: フレーム枠・セーフエリア・グリッドの描画 + トグルボタンの header 配置 | `next/shell/motolii-shell/src/lib.rs`(`stage_pane`・header 周辺) | 80〜150行 | 中(密度・配色は意匠判断だが、投影数学は§4の通り既存流用で新規判断ゼロ) |

**依存順**: S0 と S1 は互いに独立(並行可)→ S2(S0+S1 に依存)→ S3(S1 のトグル状態と、
理想的には S2 の絵に依存)。

**write-set の正直な注記**: S1・S2・S3 は全て `next/shell/motolii-shell/src/lib.rs`
(1800行超の単一ファイル)を触る — ファイル単位での完全な互いに素は成立しない
(既存の `checkerboard`/`settings_panel_open` などの機能もすべて同じファイルに同居
している、このファイルの既存の姿がそう)。関数単位では素(S1=struct/Message定義、
S2=`refresh_frame`/`RenderedFrame`、S3=`stage_pane`/header)だが、直列化(S1→S2→S3)
が安全側。S0 のみ別クレート(`motolii-engine`)なので完全に独立して進められる。

---

## 最終報告用の要約

- **縫い目1行要約**: レンダリングカメラは `Composition.camera`
  (`motolii-store::view::resolve_camera` → `motolii-core::camera_projection`
  → `motolii-compositor::render_with_timing`)として実装済みで、`Engine::render_frame(view,t)`
  という2引数固定の唯一経路(呼び手は export と shell の2箇所だけ)を通るため
  export は構造的に守られている一方、Stage 表示は CPU 経路の単純な image 貼り
  (`stage_pane`)で観測カメラに相当する状態・変換は現状どこにも存在しない。
- **観測カメラ置き場の推し**: `Shell` 直下のフィールド(`checkerboard: bool` と
  同格)— 「表示専用・Document 非搭載」の precedent が repo に既に実例として
  存在するため。
- **切片数と依存順**: 4切片。S0(engine 第二エントリ)∥ S1(shell 状態+トグル)→
  S2(配線)→ S3(枠/グリッド UI)。
