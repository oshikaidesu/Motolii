# PR #476 構造懐疑監査 — 「見た目が通る」と「正しく作った」の乖離

- 制定: 2026-08-13(利用者裁定「見た目だけ通ればいい話ではない。拡張性を考えた実装になっていない。今作成したもの全てを懐疑的に見るべき。既にある技術の採択で適切なUXが産まれない限り批判し続ける」)
- 対象: PR #476(codex/timeline-editing-ux-20260812)で作った全成果物+その検収体系
- 位置づけ: oracle green・PNG sha不変・実機smokeは「壊していない」証明であって「正しく作った」証明ではない。本書は**採択済み資産の迂回**と**拡張性の負債**を、言い訳なしで列挙する台帳である。各itemは「何を短絡したか/なぜUXと拡張性を裏切るか/正しい形」の3点で書く

## S1. Stage gizmoの迂回(最重症・実UX劣化が既に起きている)

- **短絡**: `transform_gizmo` crateは採択済みで、`RerunStage`に`Gizmo`実体・`GizmoConfig`・Inspector連携(一時値共有)・move+rotateのtest(`performance_gizmo_transform_moves_and_rotates_fixture_vertices`)まで存在する(rerun_stage.rs:66,123,543-593)。しかしhost接続時、`stage_pointer`(renderer_core.rs:850)は「host投影が正本の間はfixture gizmo probeへ送らず」と**gizmoを丸ごと迂回**し、move専用の自前gesture(`stage_host_move_pointer`)を新設した
- **裏切り**: 実Documentの操作性がfixture probe**より退化**(ハンドル・回転・scaleなし)。一般編集ソフトの反射(bounding box+ハンドル)にも不一致。`SetProperty{Scale/Rotation/Anchor/Opacity}`はD2に既存なのに、UI側の受け皿を自前moveで塞いだ
- **正しい形**: 既存Gizmoをhost幾何(stage_geometry)へ接続し、gizmo interaction→`move_layer_by`/`SetProperty`系dispatchへ写像する。自前move gestureはgizmoのtranslateハンドルに吸収して撤去

## S2. キーフレーム体系のposition特化

- **短絡**: Documentモデルは`DocParam::Keyframes`で**全paramが汎用にキーフレーム可能**なのに、wire投影(`WireTimelinePositionKey`)・D2 command鏡映(SetPositionKeyValue/Time/Interp、Add/RemovePositionKey)・UI(exact-on-key editor、key菱形)の全層をpositionへ焼き付けた
- **裏切り**: scale/rotation/opacity/effect paramへ広げる度に「command族×wire×UI」の三重複製が必要。一般編集ソフトの「全プロパティにstopwatch/キー」に対し、複製地獄が拡張の税になる
- **正しい形**: `param_path`(target+プロパティ経路)を持つ汎用key command族+汎用wire keys投影へ設計し直し、positionは退化形として乗せる。既存position系commandはjournal互換のため残置(migrate方針は別粒)

## S3. Rect→OverlayRectのベクタ資産迂回

- **短絡**: place_rectangleは`ClipSource::Vector→StandardShape::Rect`(モデルは正しくパス)だが、graph降下(graph.rs:515)はRect+modifiersなしだけを`RenderStep::OverlayRect`(パラメトリック矩形)へ特例で降ろし、Ellipse/PathOp/SvgAsset/TextPath/Groupを`UnsupportedVectorSource`で門前払い。**既存資産**=pathgeom.rs(1289行、M2 PathOp)・Path2D coverage(M5-FILTERMASK裁定でmask実用済み)・vello/vello_svg(M4-P13縮小採用)を一切通らない
- **裏切り**: 図形が増える度に特例が増える。シェイプツール・SVG・テキストへの道が全部塞がっている
- **正しい形**: StandardShape降下を既存Path2D coverage経路へ寄せ、Rectをパスの退化形として同経路に乗せる。SVGは採択済みvello_svg Sceneを既存GPU合成へ。新規tessellation実装・新規依存は不要

## S4. Timeline時間領域が「bar」(音楽格子)

- **短絡**: fixtureの見た目(1bar=2s、song_bars)を守るため、製品Timelineの座標系・snap(整数bar吸着)・i128固定小数点換算まで音楽格子で作った
- **裏切り**: 一般動画編集の格子はframe/timecode。frame step・timecode表示・playback・マーカー等、今後の全機能がbar⇄frame換算税を払う。snapが「小節」に吸着するのは動画編集の反射と不一致
- **正しい形**: Timeline内部座標をtime(RationalTime)へ、表示格子をfps由来(frame/秒/timecode)へ。barはfixture表示の互換層として隔離

## S5. accepted:true無変化のsilent no-op

- **短絡**: AddPositionKey等のprimary-gated commandは、target≠primaryだと`Ok(None)`=**accepted扱いで何も起きない**(document_edit_runtime.rs:629)。文書化なし(wave B stamp契約test作成中に偶然発見)
- **裏切り**: 利用者原則Q3「全操作に報酬・沈黙禁止」のwire層違反。呼び手は成功と区別できず、UI側で「触ったのに変化なし」を作り込む温床
- **正しい形**: 非primary targetはtyped拒否(reason付き)か、target明示commandとしてprimary gate自体を外すかの二択に裁定して統一

## S6. 単一primary選択の焼き付き

- **短絡**: 選択モデル=単一primaryが、wire(`primary_layer_id`)・command gate(S5)・UI(Layer席)全層の前提
- **裏切り**: 複数選択(文法地図Tier 2)へ行く時、選択の運搬・gate・UIを全層作り直し
- **正しい形**: wireを`selection: [layer_id]`+primaryへ先に広げる(additive)。commandのgateはS5裁定と同時に

## S7. caps+全量JSON snapshotの構造

- **短絡**: 16 layers/64 keys/8 effectsのcap、全変更で全snapshot再serialize(最大128KiB)+RN全parse、host_bridgeは手書きescape-aware parser(serdeはworkspace採択済みなのに)
- **裏切り**: 17個目のlayerが**UIから永久に触れない**(台帳F13)。field追加ごとに手書きparserの保守。projectが実規模になった瞬間に全部詰む
- **正しい形**: 差分投影(dirty range/layer単位)か、少なくともserde化+cap撤廃の設計粒。wave BのstampゲートはこのS7の痛み止めであって治療ではない

## S8. gesture識別のindexベース

- **短絡**: gesture対象を(band_index, clip_idx, key_idx)で識別し、scene差し替えとの競合をguard(`gesture_indices_valid`)で対症療法
- **裏切り**: stale-index事故クラス(嵐testが実際に踏んだOOB)が構造的に残る。layer/key追加削除と並行するgestureで常に再発リスク
- **正しい形**: gesture対象をstable ID(layer_id/key_id)で持ち、描画時にindexへ解決

## S9. Timeline renderer二重化

- **短絡**: vello製`native_timeline_renderer`(egui時代、motolii-ui内で現存・コンパイル対象)が生きたまま、rust-skia製`timeline_skia`を新設(main 26439b44系譜)
- **裏切り**: 2D vectorラスタスタックが2本(vello+skia)。依存重量・知識の分散・「どちらに機能を足すか」の恒常的な迷い
- **正しい形**: どちらかへの統一裁定(vello採択の先行裁定 vs skiaの現行実装量)。少なくとも旧rendererの生死を裁定して台帳へ

## S10. プロセスグローバルsingleton

- **短絡**: host slot単一・`TIMELINE_INTERACTING` AtomicBool(wave AのF5で**私が同じ穴に足した**)
- **裏切り**: multi-document/multi-window(F16)で全部詰む
- **正しい形**: host handle単位のstateへ。少なくとも新規追加分をsingletonにしない規律

## S11. warm-upは部分warm

- **短絡**: mount時warm-upはfixture scene/空geometryを温める。実contentの初回評価(host frame graph初回)は温まっていない(review指摘を実測値だけで棄却したのは監督の早計。実機Stage max 20.7msに現れたのがそれ)
- **裏切り**: 実projectを開いた初回に規模依存のスパイクが残る
- **正しい形**: host接続後の最初のUnchanged tickで実contentの先行評価を1回(または初回評価の分割)

## S12. 検収体系そのものの欠陥(根本原因)

- **短絡**: 監督(Fable)のoracle設計が「PNG sha不変」「挙動不変」= **見た目保存を報酬**にし、構造の正しさを検収する目を持たなかった。orderに既存資産調査欄がなく、decision-index grepの自己規律を発注時に落とした(S1/S3はこれで生まれた)
- **正しい形**(即日施行):
  1. 全orderに **PRIOR ART欄**を必須化 — tree内の既存資産・採択裁定(decision-index)・borrowできるRerun/vendored資産を発注前に列挙し、READ SETへ固定
  2. NEGATIVE ORACLEに「**採択済み資産が存在する領域での新実装**」を標準搭載
  3. oracleへ**構造検収**を追加 — 「このseamは既存Xを経由している」ことをdiff上で示させる(見た目不変だけでは合格させない)

## 是正の順番(UX即効順の監督案)

1. **S1 gizmo接続**(退化の回復+scale/rotate開通。D2は既存、接続のみ)
2. **S5+S6 selection/no-op裁定**(小さいが全操作の誠実さに直結)
3. **S2 keyframe汎用化**(仕様粒→実装の二段。Inspector全プロパティのキー化=「普通の編集ソフト」の核)
4. **S3 ベクタ降下の既存経路化**(シェイプ/SVG/テキストへの道)
5. **S4 time domain移行**(playback campaign前に必須)
6. S7/S8/S9/S10/S11は各自の粒で(S9は裁定のみ先行可)

本書の各itemは焼却されるまで削除しない。焼却はorder着地のみ。
