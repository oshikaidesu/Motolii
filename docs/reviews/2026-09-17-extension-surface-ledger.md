# 拡張の面の台帳(6)— AE と AviUtl が拡張に見せる物・書かせる物

2026-09-17 調査(取説から: AE C++ SDK Guide / Expression / Scripting の mirror、拡張編集 0.92 の写しと AviUtl plugin SDK の原文)。利用者「最小単位のコアを考えるべき。役に立つのは AE の拡張機能、AviUtl の拡張機能」。核を中から決めると家が 4 つ残るので、**拡張が見る物 / 書く物**で外から決める定規。結論は [最小の核(提案)](2026-09-17-minimal-core.md)。

---

# 拡張の定規 — After Effects が拡張に見せる物・書かせる物(2026-09-17、読むだけ)

取説から: Adobe After Effects C++ SDK Guide(ae-plugins.docsforadobe.dev)、Expression Reference(ae-expressions.docsforadobe.dev)、Scripting Guide(ae-scripting.docsforadobe.dev)。旧 aenhancers.com は DNS 不通、helpx.adobe.com は 403(r.jina.ai 経由も不可)。
目的: AE が拡張に切った口(効果 PF / 汎用 AEGP / 式)を定規にして、Motolii のホストの最小の芯を測る。Motolii 側は `crates/motolii-render/vism/*.wgsl` の頭の JSON、`compositor/effects/block_program.rs`、`engine/blocks.rs`、`compositor/effects/isf/mod.rs`、`motolii-doc/src/store/names.rs` を読んだ。

## 1. Effect plug-in(PF)— 「自分の層 1 枚に掛ける」口

- **PF_InData(ホスト → 効果、読むだけ)**: `current_time` `time_step` `time_scale`(時刻は**層の時間系**の整数 / scale。Time Stretch が負なら time_step も負)、`total_time`、`local_time_step`(時間リマップに影響されない)、`width` `height` `pixel_aspect_ratio`、`downsample_x/y`、`extent_hint`(入力と出力の見える所の交わり。6.0 以降は層より 20% 大きい)、`output_origin_x/y` `pre_effect_source_origin_x/y`、`field`、`quality`、`shutter_angle` `shutter_phase`、`sequence_data` `frame_data` `global_data`(効果自身の記憶)、`effect_ref` `appl_id` `pica_basicP`。**層の変換・親・番号・in/out 点・comp の時刻は無い**(欲しければ AEGP 経由、下の 3D)。
- **欄(PF_Param_*)**: LAYER(comp の層を選ぶ pull-down。`dephault` = MYSELF / NONE、`params[0]` は自分の入力)/ FLOAT_SLIDER(SLIDER・FIX_SLIDER は旧)/ ANGLE(360° 超可)/ CHECKBOX(CANNOT_INTERP 強制)/ COLOR / POINT(出力層の座標、原点は左上)/ POINT_3D / POPUP(選択肢は固定)/ PATH(**同じ層**のマスクへの参照、PF_PathQuerySuite)/ GROUP_START・END / BUTTON(値を持たない、SUPERVISE で押下を受ける)/ ARBITRARY_DATA(型・補間・比較・平坦化を効果が書く)/ CUSTOM・NO_DATA。**鍵は既定で打てる**(`PF_ParamFlag_CANNOT_TIME_VARY` で禁止、`CANNOT_INTERP` で補間禁止)。**補間はホスト**: 効果は毎コマ「その時刻の値」だけを受け取り、鍵の形は見えない。
- **別の層・別の時刻**: `PF_CHECKOUT_PARAM(index, what_time, time_step, time_scale)` で欄の値も層の絵も**別の時刻**で取れる(`PF_OutFlag_WIDE_TIME_INPUT` を宣言、`PF_CHECKIN_PARAM` 必須)。層の絵は `PF_LayerDef`(= PF_EffectWorld: `data` `rowbytes` `width` `height` `extent_hint`(不透明画素の外接)`pix_aspect_ratio`)。`<none>` の層は 0 の buffer。音は `PF_CHECKOUT_LAYER_AUDIO`(読むだけ)。
- **PF_OutData(効果 → ホスト)**: `out_flags` / `out_flags2` = 自分の性格の宣言(`NON_PARAM_VARY` 欄が同じでも時刻で変わる / `WIDE_TIME_INPUT` / `I_EXPAND_BUFFER` `I_SHRINK_BUFFER` `USE_OUTPUT_EXTENT` / `PIX_INDEPENDENT` / `I_USE_SHUTTER_ANGLE` / `I_USE_AUDIO` / `I_USE_3D_CAMERA` `I_USE_3D_LIGHTS` / `SUPPORTS_SMART_RENDER` `FLOAT_COLOR_AWARE` `SUPPORTS_GPU_RENDER_F32` / `REVEALS_ZERO_ALPHA` / `DEPENDS_ON_UNREFERENCED_MASKS`)、`width` `height` `origin`(FRAME_SETUP で出力の大きさを変える)、`sequence_data` `frame_data` `global_data`、`return_msg`。**書けるのは出力の絵・自分の記憶・旗だけ。**
- **SmartFX(extent の交渉)**: `PF_Cmd_SMART_PRE_RENDER` で `PF_RenderRequest`(`rect` `field` `channel_mask` `preserve_rgb_of_zero_alpha`)を受け、`checkout_layer(checkout_id, what_time…)` で要る入力を宣言 → `PF_CheckoutResult`(`result_rect` `max_result_rect` `par` `ref_width/height`)。返すのは `result_rect` `max_result_rect` `solid` `pre_render_data`。`PF_Cmd_SMART_RENDER` で `checkout_layer_pixels` / `checkout_output` / `checkin_layer_pixels`。GPU は `PF_Cmd_SMART_RENDER_GPU`(+ `GPU_DEVICE_SETUP/SETDOWN`)。
- **3D(カメラ・光)**: `AEGP_PFInterfaceSuite`(`AEGP_GetEffectLayer` `AEGP_GetEffectCamera` `AEGP_GetEffectCameraMatrix` `AEGP_ConvertEffectToCompTime`)、`AEGP_LayerSuite::AEGP_GetLayerToWorldXform(layer, comp_time)`、`AEGP_CameraSuite` `AEGP_LightSuite`、`AEGP_GetLayerStreamValue`(ZOOM・INTENSITY・COLOR・CONE_ANGLE…)、光は `AEGP_GetCompNumLayers` + `AEGP_GetLayerObjectType == AEGP_ObjectType_LIGHT` で数える。**読むだけ**。取説:「Effects cannot allow the results of AEGP queries to control what is rendered, without appropriately storing those query results (usually in sequence data), cancelling their own render, and forcing a re-render」「This should not be interpreted to mean that effects can use any AEGP suite calls」。
- **PF_Cmd_* の一生**: ABOUT → GLOBAL_SETUP / SETDOWN → PARAM_SETUP → SEQUENCE_SETUP / RESETUP / FLATTEN / SETDOWN → (FRAME_SETUP → RENDER → FRAME_SETDOWN | SMART_PRE_RENDER → SMART_RENDER) → AUDIO_SETUP / RENDER / SETDOWN。UI: EVENT・USER_CHANGED_PARAM・UPDATE_PARAMS_UI・DO_DIALOG。他: ARBITRARY_CALLBACK・QUERY_DYNAMIC_FLAGS・GET_EXTERNAL_DEPENDENCIES・COMPLETELY_GENERAL(AEGP からの呼び)・GPU_DEVICE_SETUP / SETDOWN・SMART_RENDER_GPU。
- **効果に出来ない事**: 層の変換・親・in/out を書く(読むのも AEGP 経由)/ 層を作る・消す / 他の層の欄を書く / comp の構造を見て絵を変える(上の引用)/ 自分の欄すら render 中には書けない(書けるのは UI 事件の中: `PF_ParamFlag_SUPERVISE` + `change_flags`)。出力は**常に自分の層 1 枚の絵**(大きくは出来る)。

## 2. AEGP(General Plug-in)— 書類そのものの口(1 行 1 suite: 読む / 書く)

| Suite | 読む | 書く |
|---|---|---|
| Proj / Item / Footage | project 名・bit 深度、item の名前・寸法・種類、footage の解釈・solid の色 | NewProject / Open / Save、SetItemName / SetItemParentFolder / DeleteItem、NewFootage / SetFootageInterpretation |
| Comp | comp の寸法・尺・fps・背景色、item ↔ comp | CreateComp、CreateSolidInComp / CreateCameraInComp / …、SetCompDuration |
| Layer | 番号・名前・source・親・in / offset / stretch / 転送モード・旗・2D/3D・`GetLayerMaskedBounds(time)`、`GetLayerCurrentTime`、`GetLayerToWorldXform(layer, time)`、`ConvertCompToLayerTime` | AddLayer / DuplicateLayer / DeleteLayer / ReorderLayer、SetLayerParent / SetLayerInPointAndDuration / SetLayerOffset / SetLayerStretch / SetLayerTransferMode / SetLayerFlag / SetLayerName、SetTrackMatte |
| Stream / DynamicStream | 層と効果の欄(`AEGP_LayerStream_ANCHORPOINT … TIME_REMAP … SOURCE_TEXT`)の値・型・単位・時変か、`GetLayerStreamValue(time)`、式の文字列と有効 | `SetStreamValue`、`SetExpression` / `SetExpressionState`、動的 stream の追加・削除・並べ替え |
| Keyframe | 鍵の数・時刻・値・補間・temporal ease・spatial tangent・旗 | InsertKeyframe / DeleteKeyframe、SetKeyframeValue / Interpolation / TemporalEase / SpatialTangents、AddKeyframes(まとめて) |
| Mask / MaskOutline | 層のマスクの数・モード・羽・不透明度、輪郭の頂点と接線 | マスクの追加・削除、頂点の書き換え |
| TextDocument / TextLayer | 文字列、文字の輪郭 | SetNewText |
| Camera / Light | camera の型・film size、light の型(値は stream) | SetCameraType / SetCameraFilmSize、SetLightType |
| Marker | comp / 層 marker の時刻・comment・duration | 追加・削除・書き換え |
| Effect | 層の効果の数・名前・欄の stream、`EffectCallGeneric`(効果へ PF_Cmd_COMPLETELY_GENERAL を送る) | ApplyEffect / DeleteLayerEffect |
| Render / RenderOptions / LayerRenderOptions | `RenderAndCheckoutFrame` / `RenderAndCheckoutLayerFrame`(SetTime・SetDownsampleFactor・SetRegionOfInterest・SetWorldType・SetMatteMode)= **どの item・層の、どの時刻の絵でも読める** | (絵を書く口は無い。画素を作るのは World / Composite) |
| RenderQueue / RQItem / OutputModule / RenderQueueMonitor | 列の項目・状態・出力設定 | 項目の追加・削除・設定 |
| World / Composite / Iterate / Math | AEGP_World の画素、行列 | NewWorld、転送モード・track matte・copy、複数 CPU で回す |
| Canvas / QueryXform(**Artisan** 用) | `GetCompToRender`、`GetNumLayersToRender` / `GetNthLayerContextToRender`、`GetLayerTexture`、`GetCompRenderTime`、`GetRenderLayerBounds`、`GetRenderLayerToWorldXform`、`GetROI`、`GetRenderDownsampleFactor`、`GetCompShutterAngle`、`GetTrackMatteContext`、`QueryXformGet*`(camera・時刻・変換) | `GetCompDestinationBuffer` へ 3D 層の合成結果を書く。Artisan は **3D 層だけ**を描き 2D は AE が合成、comp に 1 つ。取説「not recommended for anyone without a strong need」 |
| Utility / PersistentData / Command / Register / Collection / Memory / SoundData / ColorSettings / ItemView / Guide / FIM | 版・undo・prefs・選択・音・色設定・ガイド | ReportInfo、StartUndoGroup、menu command と hook(command / death / idle)の登録、AEIO の登録、ガイドの追加 |
| PFInterface | 効果 → AEGP の橋(`GetEffectLayer` / `GetEffectCamera` / `GetEffectCameraMatrix` / `ConvertEffectToCompTime`) | — |
| AEIO(形式) | 入力: `AEIO_InitInSpecFromFile` / `GetInSpecInfo` / `GetDimensions` / `GetDuration` / `GetTime` / `DrawSparseFrame` / `DrawFrame` / `GetSound` / `InqNextFrameTime`(ホストが「この時刻のコマをくれ」) | 出力: `InitOutputSpec` / `StartAdding` / `AddFrame` / `AddSoundChunk` / `EndAdding` / `OutputFrame`(ホストが描いたコマを渡す) |

要点: **書類を書けるのは AEGP と ExtendScript だけ**。AEGP は何でも読めて何でも書ける代わりに、描く途中(render)では書かない。効果(PF)は描く途中に居るので書けない。

## 3. Expressions / ExtendScript — 「どの値からでも、どの時刻の値でも」

- **式が読める物**(全部「値」。絵は `sampleImage` の平均だけ): Global `time`(comp の秒)、`value`、`thisComp` `thisLayer` `thisProperty` `thisProject`、`comp(name)` `footage(name)`、`colorDepth`、`posterizeTime(fps)`。Comp `layer(index | name | otherLayer, relIndex)` `layerByComment`、`numLayers`、`width` `height` `duration` `frameDuration` `pixelAspect` `shutterAngle` `shutterPhase` `bgColor` `displayStartTime` `activeCamera` `marker`。Layer `index` `name` `parent` `hasParent` `inPoint` `outPoint` `startTime` `width` `height` `active` `enabled` `hasVideo` `hasAudio` `audioActive`、`source`、`sourceTime(t)`、`sourceRectAtTime(t, includeExtents)`(top/left/width/height)、`sampleImage(point, radius=[.5,.5], postEffect=true, t=time)`、`effect(name|index)`、`mask(name|index)`、`toComp / fromComp / toWorld / fromWorld(point, t=time)` と Vec 版、`fromCompToSurface`、`transform.*`(anchorPoint / position / scale / rotation / opacity / orientation / rotationX.. / timeRemap / audioLevels)、camera(`zoom` `pointOfInterest` `focusDistance` `aperture` `depthOfField` …)、light(`intensity` `color` `coneAngle` …)。
- Property `value` `valueAtTime(t)` `velocity` `velocityAtTime(t)` `speed` `speedAtTime(t)` `numKeys` `key(i).time / value / index` `nearestKey(t)` `nextKey` `previousKey` `propertyGroup` `propertyIndex`、`wiggle(freq, amp, octaves, amp_mult, t)` `temporalWiggle` `smooth(width, samples, t)` `loopIn / loopOut(type, n)` `loopInDuration / loopOutDuration`。Path `points(t)` `inTangents(t)` `outTangents(t)` `isClosed()` `pointOnPath(u, t)` `tangentOnPath` `normalOnPath`、`createPath(points, inTangents, outTangents, isClosed)`(返せる唯一の「形」)。補間 `linear / ease / easeIn / easeOut(t, tMin, tMax, v1, v2)`。乱数 `seedRandom(offset, timeless=false)` `random` `gaussRandom` `noise(v)`(種は「層の一意 id・欄・現在時刻・offset」の関数。`timeless=true` で時刻を外す)。音は `audioLevels` と、Keyframe Assistant「Convert Audio to Keyframes」が作る Null 層「Audio Amplitude」の Left / Right / Both Channels の Slider の鍵(音 → 鍵。式は鍵を読む)。
- **式が書ける物**: 自分の property の値 1 つ(各時刻の評価結果)。他の property・層・構造は読めるが書けない。記憶も無い(毎時刻の純関数。`posterizeTime` は時刻の丸め)。
- **ExtendScript**: AEGP と同じ側。`Property.setValue / setValueAtTime / setValuesAtTimes / addKey / removeKey / setInterpolationTypeAtKey / setTemporalEaseAtKey / setSpatialTangentsAtKey / expression / expressionEnabled`、`valueAtTime(time, preExpression)`、`LayerCollection.add / addNull / addSolid / addText / addBoxText / addCamera / addLight / addShape / precompose`。一度組む側(Motolii の台本と同型)。

## 4. 交差表 — 拡張が見る物 / 書く物

| 項目 | PF(効果) | AEGP | Expression | 時刻の純関数か | Motolii の対応 |
|---|---|---|---|---|---|
| 今の時刻 | `current_time` `time_step` `time_scale`(層の時間系) | `GetLayerCurrentTime` `GetCompRenderTime` | `time` `frameDuration` | ○ | block `host.time`(block_program.rs PRELUDE、comp の秒)、pass `TIME / TIMEDELTA / FRAMEINDEX`(vism.rs CLOCK_KEYS)、欄 `SUBTYPE: TIME`(turbulent_displace の Evolution) |
| 層の時刻 ↔ comp の時刻 | `current_time` は層の時間、`local_time_step` | `ConvertCompToLayerTime` `GetLayerInPoint / Offset / Stretch` | `sourceTime(t)` `inPoint` `outPoint` `startTime` `timeRemap` | ○ | `layout.rs layer_time`(親の順番の札 = Stagger / schedule でずれる)、`looped_time`、`object_frames`(blocks.rs:448)、札 Time Remap / Speed / Fade In / Fade Out(names.rs FIXED) |
| 自分の層の絵 | `params[0]` の PF_LayerDef | `RenderAndCheckoutLayerFrame` | `sampleImage(p, r, postEffect, t)` | ○ | INPUTS `"TYPE": "image"`(source / inputImage) |
| 他の層の絵 | `PF_Param_LAYER` + `PF_CHECKOUT_PARAM` | `RenderAndCheckoutLayerFrame` | `thisComp.layer("x").sampleImage(…)` | ○ | `"TYPE": "layer"` + image の `"LAYER"`(set_matte.fs)、`SOURCE: below / group / comp`(background_delay.fs、isf/mod.rs TimeSource) |
| 別の時刻の絵・値 | `PF_CHECKOUT_PARAM(what_time)` + `WIDE_TIME_INPUT` | `RenderOptions::SetTime` | `valueAtTime(t)` `sampleImage(…, t)` | ○(過去も未来も) | `TIME_OFFSET` / `TIME_AT` / `TIME_OFFSET_FRAMES`(hold.fs、isf/mod.rs TimeOffset)。ホストが t′ の層を引き直して渡す = 純のまま(plugin-resources.md §6) |
| 層の大きさ・PAR・縮小 | `width` `height` `pixel_aspect_ratio` `downsample_x/y` | `GetItemDimensions` `GetLayerMaskedBounds` | `width` `height` `sourceRectAtTime` | ○ | `render_size` `material_origin`(vism.rs)、`objects[k].lo / hi`(block)、`FieldIn.frame_position`(subtype.rs) |
| 見える範囲(extent) | `extent_hint`、SmartFX `output_request` → `result_rect` `max_result_rect` | `GetROI` `GetRenderLayerBounds` | — | ○ | PASSES の WIDTH / HEIGHT(vism.rs:446)、`REACH`(近くの升目の幅、blocks.rs run_blocks) |
| 層の変換(位置・回転・拡大・anchor) | 読む: AEGP 経由 `GetLayerToWorldXform` のみ。**書く: ×** | 読む・書く(`GetLayerStreamValue` / `SetStreamValue` POSITION…) | 読む: `transform.position` `toWorld(p, t)`。書く: 自分の 1 つだけ | ○ | 読む: `objects[k]`(comp 座標の箱)。**書く: `Offset{translate, rotate, scale}` → state → motion**(block_program.rs WorldPass)。AE に無い口 |
| 住む箱・親 | × | `GetLayerParent` / `SetLayerParent` | `parent` `hasParent` `toComp` | ○ | `room_lo / room_size` `group`、`SCOPE: room`、`PHYSICS.ROOM ancestor / parent`(field.wgsl) |
| 番号・数・隣 | × | `GetLayerIndex` `GetCompNumLayers` | `index` `numLayers` `layer(other, rel)` | ○ | `k` `host.members / objects / round` `neighbor(k, i)`(CSR、全員を回らない) |
| 欄(数・角度・色・点・選択・真偽) | `PF_Param_*`、値は毎コマホストが補間して渡す | `GetLayerStreamValue(time)` / `SetStreamValue` | `effect("x")("y")` `valueAtTime` | ○(鍵の補間) | INPUTS float / long / bool / color / point2D / point3D、`effect.<id>.param.<name>` は書類の property で鍵可(view/resolve.rs:423、`value_at(t)`) |
| 鍵の形(ease・接線) | ×(値しか見えない) | Keyframe suite で読み書き | `key(i)` `velocity` `speed` `ease()` `loopOut` | ○ | 書類の鍵と ease 名。効果には値だけ渡す(AE と同じ切れ目) |
| 経路(mask path) | `PF_Param_PATH`(同じ層のマスクだけ) | MaskOutline | `mask(1).maskPath.points(t)` `pointOnPath` `createPath` | ○ | 札 `mask.<id>.shape`(names.rs)、pathop(Offset / Trim / Oscillator)。効果へ点列を渡す口は無し |
| 任意データ | `PF_Param_ARBITRARY_DATA`(補間・比較・平坦化を自分で) | — | — | ○(効果が定義) | 無し(欄は float / long / bool / point / color / image / layer だけ) |
| カメラ・光 | `I_USE_3D_CAMERA / LIGHTS` + AEGP で読む | Camera / Light suite + stream | `activeCamera` `camera.zoom` `light.intensity` | ○ | `camera_value_at`(comp の経路、view.rs)、`SurfaceIn.view_dir / world_position`、`frame.sun_color`(surface_program.rs) |
| 音 | `PF_CHECKOUT_LAYER_AUDIO`(読む)。音の効果は別の旗 | SoundData / `RenderNewItemSoundData` | `audioLevels`、Convert Audio to Keyframes の鍵 | ○(標本) | 札 Level / Pan / Fade のみ。効果に音を渡す口は無し(DAW を再発明しない) |
| 乱数の種 | 無し(自前) | `GetLayerDancingRandValue` | `seedRandom(offset, timeless)` `random` `noise` | ○(層・欄・時刻の関数) | 欄 `seed` + `dice(k)`(arrive.wgsl)、台本 `random(seed)` |
| 記憶(状態) | `sequence_data` `frame_data`、`NON_PARAM_VARY` | PersistentData | 無し(毎時刻の純関数) | × | `PHYSICS`(Rapier の焼き、field.wgsl)、`ROUNDS` の state(1 コマの中だけ)、PERSISTENT feedback は予約(plugin-resources.md §6) |
| 出力の絵 | 出力 PF_LayerDef / `checkout_output`、`width / height / origin`(I_EXPAND_BUFFER) | Artisan: `GetCompDestinationBuffer` | ×(値のみ) | — | pass の出力 / `PASSES TARGET`、`OUTPUT_FLOAT`、surface の `vec3f`、field の `FieldOut.offset` |
| 自分の欄を書く | UI 事件の中だけ(`SUPERVISE` + `change_flags`) | `SetStreamValue` `InsertKeyframe` `SetExpression` | × | — | 効果は書けない。書くのは Intent(AGENTS.md「書き込みは Intent 経由のみ」) |
| 層・構造を作る | × | `AddLayer` `CreateComp` `DuplicateLayer` `DeleteLayer` `SetLayerParent` | × | — | 台本(`api.*` で一度組む)= AEGP / ExtendScript の側 |
| 他の物を動かす | × | ○(何でも) | ×(自分の 1 property) | — | ○ `SCOPE: room` の Field が同じ箱の全員を動かす、`FollowPass`(付いて行く)、`set_physics_shifts`(線が追う、layout.rs:1833) |
| 尺(始まり・終わり) | `total_time` | `GetLayerDuration` `GetCompDuration` | `duration` `outPoint` | ○ | `last_frame`(Gather = 終わりは構図)、層の in / out |

## 5. 見立て(8 行)

1. AE の拡張の口を定規にすると、ホストの最小の芯は **6 つ**: 時刻(層の時間系 + comp との換算)、層の絵(自分と、名指した他の層)、鍵の欄(ホストが補間して「その時刻の値」だけ渡す)、層の大きさ・変換(読むだけ)、別の時刻の絵と値(what_time / valueAtTime)、出力の絵 1 枚。Motolii はこの 6 つを全部持っている(host.time / image 欄 / INPUTS + value_at / objects[k] + render_size / TIME_OFFSET / pass の出力)。
2. **効果は変換を書けない**。これが AE の最大の縫い目: 式は自分の 1 property を書くために「他の全部を読む」口になり、Trapcode の類は層の絵の上に粒子を**描く**事で動きを作り(層は動かない)、Cavalry / MASH は「動かす側を node にする」事でこの縫い目を消した。Motolii の block(`Offset` → motion)と `SCOPE: room` の Field は、AE なら AEGP か式でしか出来ない「他の物を動かす」を効果の棚に載せた物 = AE の切れ目を 1 つだけ跨いだ所が Motolii の芯。
3. **記憶の置き場は AE も 1 つ持つ**(`sequence_data` + `NON_PARAM_VARY`)。物理の焼きはそこに当たる。Motolii は `PHYSICS`(Rapier)と PERSISTENT の予約で同じ 1 箇所に閉じ込めている。増やさない。
4. **鍵の補間はホスト、効果は値だけ**(PF)。鍵の形(velocity / key(i))を見るのは式と AEGP だけ。Motolii も効果には値だけ渡す — ease や鍵の形を効果の欄に持ち込まない。
5. **extent の交渉**(SmartFX の result_rect / max_result_rect、I_EXPAND_BUFFER)は「絵の大きさは効果が決める」口。Motolii の PASSES WIDTH / HEIGHT と REACH で足りる。
6. **他の層は「名指し」で読む**(PF_Param_LAYER、`layer("x")`)。数・番号・隣を読むのは式と AEGP だけ。Motolii の `k` / `neighbor(k, i)` は式の `index` を GPU に置いた物 — AE では式でしか書けない「並びの関係」を効果が読む。
7. **式が読める物は全部「値」**(絵は sampleImage の平均だけ)。Motolii の台本も同じ(値の関係を一度組む)。絵の関係は効果(image 欄・LAYER・SOURCE)。
8. 足りない口は 2 つだけ: (a) 音を値として読む口(AE は Convert Audio to Keyframes = 鍵に焼く。Motolii なら「音 → 鍵」の一手で足り、効果に音は渡さない)、(b) 経路の点列を効果へ渡す口(PF_Param_PATH / `points(t)`)。どちらも「値の形」で足り、ホストの芯は増えない。

## Sources

- AE C++ SDK Guide — PF_InData https://ae-plugins.docsforadobe.dev/effect-basics/PF_InData/ 、PF_OutData(旗の一覧)https://ae-plugins.docsforadobe.dev/effect-basics/PF_OutData/ 、Parameters https://ae-plugins.docsforadobe.dev/effect-basics/parameters/ 、PF_ParamDef(旗)https://ae-plugins.docsforadobe.dev/effect-basics/PF_ParamDef/ 、PF_EffectWorld / PF_LayerDef https://ae-plugins.docsforadobe.dev/effect-basics/PF_EffectWorld/ 、Command Selectors https://ae-plugins.docsforadobe.dev/effect-basics/command-selectors/ 、Interaction Callback Functions(PF_CHECKOUT_PARAM)https://ae-plugins.docsforadobe.dev/effect-details/interaction-callback-functions/ 、SmartFX https://ae-plugins.docsforadobe.dev/smartfx/smartfx/ 、Accessing Camera & Light Information https://ae-plugins.docsforadobe.dev/effect-details/accessing-camera-light-information/ 、Cheating Effect Usage of AEGP Suites(引用 2 文)https://ae-plugins.docsforadobe.dev/aegps/cheating-effect-usage-of-aegp-suites/ 、AEGP Suites https://ae-plugins.docsforadobe.dev/aegps/aegp-suites/ 、Artisans https://ae-plugins.docsforadobe.dev/artisans/artisans/ 、AEIOs https://ae-plugins.docsforadobe.dev/aeios/aeios/
- AE Expression Reference — Global https://ae-expressions.docsforadobe.dev/general/global/ 、Comp https://ae-expressions.docsforadobe.dev/objects/comp/ 、Layer General https://ae-expressions.docsforadobe.dev/layer/general/ 、Layer Properties https://ae-expressions.docsforadobe.dev/layer/properties/ 、Layer Sub-objects https://ae-expressions.docsforadobe.dev/layer/sub-objects/ 、Layer Space Transforms https://ae-expressions.docsforadobe.dev/layer/layer-space-transforms/ 、Property https://ae-expressions.docsforadobe.dev/objects/property/ 、Key https://ae-expressions.docsforadobe.dev/objects/key/ 、Path Property https://ae-expressions.docsforadobe.dev/objects/path-property/ 、Camera https://ae-expressions.docsforadobe.dev/objects/camera/ 、Interpolation https://ae-expressions.docsforadobe.dev/general/interpolation/ 、Random Numbers https://ae-expressions.docsforadobe.dev/general/random-numbers/
- AE Scripting Guide — Property https://ae-scripting.docsforadobe.dev/property/property/ 、LayerCollection https://ae-scripting.docsforadobe.dev/layer/layercollection/
- Convert Audio to Keyframes(helpx は 403。二次資料)— macProVideo「Animating with Sound in After Effects, Part 1」https://www.macprovideo.com/article/after-effects/animating-with-sound-in-after-effects-part-1 、Surfaced Studio https://www.surfacedstudio.com/blog/vfx-vlog/after-effects-audio-to-keyframes/
- Motolii — motolii/crates/motolii-render/src/compositor/effects/block_program.rs(PRELUDE: Item / Offset / BlockHost、WorldPass、FollowPass)、src/engine/blocks.rs(:448 object_frames、run_blocks、set_physics_shifts)、src/compositor/effects/isf/mod.rs(IsfInputType、TimeOffset / TimeBase / TimeSource)、src/compositor/effects/vism.rs(CLOCK_KEYS、render_size、:446 PASSES)、src/compositor/effects/subtype.rs(FieldIn / FieldOut / SurfaceIn)、vism/{arrive,field,wave,hang,bounce,push_apart,turbulent_displace,glass,clip}.wgsl、vism/{hold,background_delay,set_matte}.fs、motolii/crates/motolii-doc/src/store/names.rs(FIXED)、store/layout.rs(layer_time / looped_time / :1833 set_physics_shifts)、store/view/resolve.rs:423(effect.<id>.param.)、docs/plugin-resources.md §6(CompLookbehind)、docs/reviews/2026-09-17-creative-coding-ledger.md


---

# AviUtl の拡張が見る物・書く物 — 最小の芯を測る定規(2026-09-17、読むだけ)

定規は取説: 拡張編集 0.92 の lua.txt(原本は取れず — scrapbox の lua.txt 頁は空、fandom の写しは Cloudflare。以下「写し」= scrapbox /aviutl の obj 頁群と「スクリプトファイル フォーマット」頁、exedit.txt の引用)、AviUtl 0.99k plugin SDK の filter.h / input.h / output.h(kbinani mirror、原文)、AviUtl2 の Lua 取説(後継、差分の確認用)。
Motolii 側は vism/*.wgsl の manifest、block_program.rs / engine/blocks.rs のブロック契約、names.rs の札、Split の写し。

## 1. 拡張編集の台本(.anm / .obj / .scn / .cam / .tra)

種類と置き場(写し・exedit.txt): `.anm` アニメーション効果(フィルタ効果の 1 段)、`.obj` カスタムオブジェクト(素材)、`.scn` シーンチェンジ(obj = 前の絵、framebuffer = 後の絵、進捗は `obj.getvalue("scenechange")`)、`.cam` カメラ効果(描画関数は効かない、`camera_param` を読んで書く)、`.tra` トラックバー変化方法。全部 exedit.auf 隣の `script/`(1 段下まで)。Shift-JIS・CRLF。1 file 複数は `@名前` 行で区切る(表示 `名前@file`)。

頭の宣言 = ホストが持つ欄(写し「スクリプトファイル フォーマット」):
- `--track0:名前,始点,終点,既定値[,間隔 1|0.1|0.01]`(0..3、.scn は 0,1)→ `obj.track0..3`。トラックバーは拡張編集の物: 中間点ごとにキーが打て、移動の法(移動無し/直線/加減速/曲線/瞬間/中間点無視/移動量指定/ランダム/反復 + .tra)で補間される。台本は毎コマの値だけ受け取る
- `--check0:名前,既定値` → `obj.check0`(boolean)。`--color:` → 色ボタン → global `color`(数値)。`--file:` → ファイル選択 → global `file`(文字列)
- `--dialog:表示名,変数=初期値;…`(16 個まで、`/chk` `/col` `/fig` で部品)→ `変数=初期値` がそのまま Lua に流れる(global。`local` と書けば local)。`--param:` パラメータ設定ダイアログ(例 `--param:p={-100,-100,…};`)
- `--value:` `--information:` は写し 2 つに無い(利用者の列挙。AviUtl2 の `--value@` / `--information` が該当。0.92 での有無は未確認)
- .tra だけ: `--twopoint`(中間点を見ない)、`--speed:1,1`(加速/減速チェック)、`--param:42`(整数 1 つ = 「設定」ダイアログ)

obj の変数(写し「obj」):
- 書ける(描画前に変えるとホストの描画に効く): `obj.ox/oy/oz` 相対座標、`obj.rx/ry/rz` 回転(度)、`obj.cx/cy/cz` 中心、`obj.zoom` 拡大率(1 = 等倍。getvalue の "zoom" は 100 = 等倍)、`obj.alpha` [0,1]、`obj.aspect` [-1,1]。`obj.x/y/z` は基準座標(中間点の値)
- 読むだけ(書いても反映されない): `obj.w/h`(拡張描画の拡大率・縦横比を受けた後の大きさ)、`obj.screen_w/h`、`obj.framerate`、`obj.frame`(0 から)、`obj.time`(オブジェクト基準の秒)、`obj.totalframe/totaltime`(オブジェクトの長さ)、`obj.layer`、`obj.index`(何番目の個別オブジェクトか)、`obj.num`(個別オブジェクトの総数)、`obj.track0..3`、`obj.check0`
- `obj.id / obj.color / obj.file / obj.figure` は 0.92 に無い(`color`/`file` は global。AviUtl2 に `obj.id`、`--color@`/`--figure@`)

| 関数(写し) | 形 | 何を見る / 書く |
|---|---|---|
| `obj.draw` | `([ox,oy,oz,zoom,alpha,rx,ry,rz])` 既定 0/1/1/0 | 今の obj 画像を framebuffer へ 1 枚。何度でも(複製は draw を回す)。draw すればホストは自分では描かない(`draw_state` がその印、setoption で上書き) |
| `obj.drawpoly` | `(x0,y0,z0,…,x3,y3,z3[,u0,v0,…,u3,v3[,alpha]])` | 4 点の板。uv は (0,0)–(obj.w,obj.h) |
| `obj.load` | `("movie",file[,time=obj.time,flag])`→秒,コマ数 / `("image",file)` / `("text",text[,speed,time])` / `("figure",name[,color,size,line,aspect])` / `("framebuffer"[,x,y,w,h])` 下の層の合成結果 / `("tempbuffer"[,x,y,w,h])` / `("layer",no[,effect])` 他層の絵 / `("before")` 直前 obj(.obj のみ) / 無指定は自動判別 | obj の画像を差し替える(w/h が変わる) |
| `obj.effect` / `obj.filter` | `("クリッピング","上",30,"右",10)` 名無しなら「以降の効果を先に実行」 | 内蔵のフィルタ効果を obj 画像に掛ける / filter は framebuffer に掛ける |
| `obj.setfont` / `obj.mes` | `(name,size[,type 0..4,col1,col2])` / `(str)` | text の書体(以後の台本へ伝播 = 状態)/ テキストの文字バッファへ追記 |
| `obj.rand` | `(a,b[,seed=0,frame=obj.frame])` | 整数。同じ seed と frame なら同じ値(純)。seed 省略はオブジェクト毎、負なら全オブジェクト同じ(AviUtl2 写し) |
| `obj.getvalue` | `(target[,time=obj.time,section=0])` target: `0..3` 自分のトラック、`"x" "y" "z" "rx" "ry" "rz" "zoom" "alpha" "aspect" "time"`(オブジェクトの設定値。後の基本効果は反映されない)、`"layer<番号>.<種類>"` 他層、`"scenechange"`。time は秒、section は区間の番号 | 例 `obj.getvalue("layer"..(obj.layer-1)..".x")`、`obj.getvalue(0,0,i-1)` = 区間 i-1 の頭の track0 |
| `obj.getoption` | `"track_mode"`(n)→0 移動なし 1 直線 2 曲線 3 瞬間 4 中間点無視 5 移動量指定 6 ランダム 7 加減速 8 反復 15 スクリプト、`"section_num"` 中間点+1、`"script_name"`、`"gui"`、`"camera_mode"`、`"camera_param"`→{x,y,z,tx,ty,tz,rz,ux,uy,uz,d}、`"multi_object"` 個別オブジェクトか | 読む |
| `obj.setoption` | `"culling" "billboard" "shadow" "antialias" "blend"(合成モード) "drawtarget"(framebuffer|tempbuffer,w,h) "draw_state" "focus_mode" "camera_param"` | 描き方とカメラを書く |
| `obj.getpoint` | .tra のみ: `(n)` 中間点 n の値、`"index"` 今の区間(小数 = 区間内の割合)、`"totalindex"`、`"num"`、`"time"[,section]`、`"accelerate"/"decelerate"`、`"param"`、`"link"`→(番号,総数)、`"default"` | 補間の材料 |
| `obj.interpolation` | `(t,x0..x3)` / 2D / 3D | P1–P2 間の距離重み 3 次(exedit.tra の補間移動が使う) |
| `obj.setanchor` | `(name,num[,line|loop|star|arm, color c, inout, xyz])` name = --param/--dialog の global table か `"track"`(track0=x, track1=y[, track2=z]) | 台本が画面に掴める点を出し、動かした座標がホストの欄へ書き戻る |
| `obj.getaudio` | `(buf,file,type,size)` file = `"audiobuffer"`(拡張編集の音)か path、type `"pcm"/"spectrum"/"fourier"`(+`.l`/`.r`) → (数, サンプルレート[, table]) | 音を読む(型の一覧は AviUtl2 写し) |
| `obj.copybuffer` | `(dst,src)` `obj / tmp / cache:名 / image:file / frm`。obj と tmp はどこからでも、cache へは obj/tmp から、image・frm は読むだけ | cache はコマを跨いで残る(状態) |
| `obj.getpixel/putpixel/copypixel/pixeloption` | `(x,y[,type])` 型 `col/rgb/yc`、`pixeloption("get"|"put", "obj"|"frm")`、`("blend",mode)` | 1 画素ずつ |
| `obj.getpixeldata/putpixeldata` | `([work|alloc])`→(BGRA userdata,w,h) / `(data)` | 画像を丸ごと DLL/FFI へ |
| `obj.getinfo` | `"script_path" "saving" "image_max"` | 環境 |

.tra(写し): 描画関数は効かず、**戻り値 = そのコマのトラックの値**。exedit の移動 9 種と同じ棚に .tra が並ぶ = **イージングが拡張の単位**。材料は `obj.getpoint` + `obj.interpolation`(`return obj.getpoint("default")` で直線)。

個別オブジェクトと書き戻し(exedit.txt「文字毎に個別オブジェクト: 文字毎に個別オブジェクトとして処理します。フェード効果等が文字毎に処理される」): 台本は文字ごとに 1 回走り、`obj.index`(0 から)/`obj.num`、`obj.x/y` はその文字の位置。書き戻しは 2 通り — (a) 描画前に `obj.ox/rz/zoom/alpha` を変えてホストに描かせる(.anm の基本形、テキストの制御文字 `<?obj.rz=obj.time*360?>` も同じ)、(b) `obj.draw/drawpoly` で自分で描く(何枚でも)。

## 2. フィルタプラグイン(.auf)・入力(.aui)・出力(.auo)— SDK 原文、1 行ずつ

- `.auf` FILTER_DLL: 宣言 = `flag`、`name`、`track_n/track_name/track_default/track_s/track_e`(整数の欄、NULL なら 0..256。**キーは無い**)、`check_n/check_name/check_default`、`func_proc(fp,fpip)`、`func_init/exit/update(status)/WndProc`、`func_save_start/end`、`func_is_saveframe`、`func_project_load/save`(project に残す data)、`func_modify_title`、`ex_data_ptr/size/def`(保存される拡張データ)、`information`。`fp->track[]/check[]` に今の値(AviUtl が入れる)、`fp->exfunc`
- FILTER_PROC_INFO(func_proc が見る): `ycp_edit`(YC48 の絵、**書き換える**。`ycp_temp` と入れ替え可)、`w,h`(**変えられる**)、`max_w/max_h`、`frame`(0 から)/`frame_n`、`org_w/org_h`、`audiop/audio_n/audio_ch`(音声フィルタ時 PCM16)、`editp`、`yc_size/line_size`、`flag`(フィールド順・インターレース)
- EXFUNC 読む: `get_ycp_source_cache(editp,n,ofs)`(**任意のコマ**のフィルタ前の絵)、`get_ycp/get_ycp_ofs`、`get_pixelp`(RGB24)/`get_pixel_source(format)`、`get_audio(n,buf)`、`get_ycp_filtering_cache_ex(fp,editp,n,&w,&h)`(自分の手前までフィルタ済みの任意コマ。`set_ycp_filtering_cache_size` で cache)、`get_audio_filtering`、`get_frame/get_frame_n/get_frame_size/get_select_frame`、`get_file_info/get_source_file_info/get_source_video_number`、`get_sys_info`、`get_config_name`、`is_editing/is_saving/is_keyframe/is_recompress/is_filter_active`、`get_frame_status(_table)`、`get_disp_pixelp`(表示フィルタ)
- EXFUNC 書く(編集そのもの): `set_frame/set_frame_n/set_select_frame`、`copy_frame/copy_video/copy_audio`、`set_frame_status`、`copy_clip/paste_clip`、`filter_window_update`、`ini_load/save_int/str`、`dlg_get_load/save_name`、`add_menu_item`。道具: `rgb2yc/yc2rgb`、`create_yc/delete_yc`、`load_image`、`resize_yc`、`copy_yc`、`draw_text`、`avi_file_open/read_video/read_audio(_sample)/close`、`exec_multi_thread_func`
- `.aui` INPUT_PLUGIN_TABLE: `func_open(file)`→handle、`func_info_get`→INPUT_INFO(`rate/scale`、`n`、BITMAPINFOHEADER、`audio_n`、WAVEFORMATEX)、`func_read_video(ih,frame,buf)`、`func_read_audio(ih,start,length,buf)`、`func_is_keyframe`、`func_config` = **コマ番号 → 絵/音の純関数**(cache はホスト)
- `.auo` OUTPUT_PLUGIN_TABLE: `func_output(oip)` が `oip->func_get_video(frame)/func_get_video_ex(frame,format)/func_get_audio(start,length,&readed)` を引いて書く。`w,h,rate/scale,n,audio_*,savefile`、`func_is_abort/func_rest_time_disp/func_update_preview/func_get_flag(frame)` = ホストへは何も書かない
- 拡張編集(exedit.auf)自身が 1 本の .auf: タイムライン・オブジェクト・Lua は全部フィルタプラグインの中。ホスト本体が持つのはコマ・YC48 の絵・欄・project だけ

## 3. 交差表「拡張が見る物 / 書く物」

| 項目 | anm/obj | .auf | 時刻の純関数か | Motolii の対応 |
|---|---|---|---|---|
| 時刻 | `obj.time/frame/totaltime/totalframe/framerate`(オブジェクト基準) | `fpip->frame/frame_n`(comp 基準) | 純: 毎コマ白紙で走る。ただし Lua の global・`cache:名`・setfont はコマを跨ぐ(台本が状態を持てる) | ISF `TIME/TIMEDELTA/FRAMEINDEX/DATE`(isf/mod.rs:733)。block は `host.time` = comp の秒(blocks.rs:634、層の入点基準でも Stagger 後でもない。Stagger は block が `k` で自作 arrive.wgsl:29)。絵は `TIME_OFFSET/TIME_AT`(isf/mod.rs:126) |
| 自分の絵 | obj の画像(`load` で差し替え、`w/h`)、`getpixel/getpixeldata` | `ycp_edit`(w,h 可変)、`ycp_temp` | 純 | pass: image 入力 → 出力 1 枚(PASSES で中間 target、blur.wgsl)。block は絵を見ない(箱だけ) |
| 変換・大きさ・不透明度 | `ox/oy/oz rx/ry/rz cx/cy/cz zoom alpha aspect` を**読み書き**、`x/y/z screen_w/h` | 無し(座標の概念が無い) | 純 | block が読む: `objects[k].lo/hi`(箱)、`room_lo/room_size`、`radius`、`margin/weight/group`(block_program.rs:109)。書く: `Offset(translate,rotate,scale)` だが WorldPass は translate+rotate だけ motion へ(block_program.rs:425、**scale は落ちる**)。alpha・色・aspect・中心は書けない |
| 番号 | `obj.index/obj.num`(個別オブジェクト)、`obj.layer` | 無し | 純 | block: `k`(効果の列が掛かった順の物の番号、blocks.rs:431)、`host.members/objects`、`neighbor(k,i)`。**Split の写し(copy>0)は物にならない**(blocks.rs:235 `copy == 0`、resolve.rs:1260 push_split)= 文字単位に index 無し。Repeater は Each/Random の行(placement.rs:80-90) |
| 種の乱数 | `obj.rand(a,b,seed,frame)` | 無し(自前) | 純 | vism の `seed` 欄(turbulent_displace.wgsl)、block の `dice(k)`(arrive.wgsl:20)、Repeater `Seed`、台本 `random(seed)` |
| 欄(ホストのつまみ) | `--track0..3`(移動の法でキー)、`--check0`、`--color/--file/--dialog/--param`、`setanchor` で画面から書き戻し | `track_n/check_n`(整数、キー無し)、`ex_data` | 純(値はホストが補間して渡す) | `INPUTS`: float/long/bool/point2D/point3D/color/image/layer(isf/mod.rs:54)+ MIN/MAX/DEFAULT/LABELS/SUBTYPE/ADVANCED/HERO、鍵は Timeline。block は float 24 個まで(block_program.rs:13)。画面で掴む点(setanchor)は無し |
| 他の値を他の時刻で | `getvalue(target,time,section)`(自分のトラック・x/y/z/zoom/alpha/aspect/time・`layerN.x`)、`load("movie",file,time)`、`load("layer",no)`、`load("framebuffer")` | `get_ycp_source_cache(n,ofs)`、`get_ycp_filtering_cache_ex(n)`、`get_audio(n)` | 純(時刻を引数で引く) | 絵だけ: `TIME_OFFSET`(層の絵を別の時刻で)、`LAYER`(他層の絵、isf/mod.rs:130)、Field の層参照(layout.rs:250)。**数値を他層・他時刻から引く口は無し** |
| 音 | `getaudio(audiobuffer|file, pcm/spectrum/fourier)` | `audiop`、`get_audio(_filtering)` | 純 | render/audio に波形はあるが INPUTS に音の型は無い |
| 出す側 | `draw/drawpoly`(何枚でも)、`load`(素材の差し替え)、`effect/filter`(内蔵効果を呼ぶ)、`setoption`(blend/drawtarget/camera) | `ycp_edit` を書き換える 1 枚、`w/h` 変更 | 純 | pass は出力 1 枚、block は Offset のみ。draw を回す複製・他効果の呼び出し・素材の読み込みは無し(複製は Repeater の札、素材は層 = 意図的) |
| カメラ | `.cam`: `get/setoption("camera_param")` | 無し | 純 | Camera rows(names.rs `CAMERA_ROWS`)。効果からは触れない |
| 補間の拡張 | `.tra`: `getpoint` + 戻り値、`--twopoint/--speed/--param` | 無し | 純 | ease は固定名(Linear/Bezier/Elastic/Steps…)。拡張の口無し |
| 状態 | Lua global、`cache:名`、setfont、tempbuffer | plugin の static、`ex_data`、`project_save` | × | `PHYSICS`(Rapier、`TIME: gather` = 終わりから読む、field.wgsl)、feedback 効果(hold/background_delay)。他は純 |

## 4. 見立て — AviUtl の面を定規にした最小の芯

1. 芯 = **時刻**(obj.time)・**自分の絵**(w/h)・**変換 8 つ**(ox oy oz rx ry rz zoom alpha、+ aspect cx cy cz)・**番号**(index/num)・**種の乱数**・**欄**(track/check/color/file/dialog、鍵と移動の法はホスト)・**他の値と他の時刻を引く口**(getvalue/load)・**出す側**(draw/load/effect)。ホストが持つのはこれと移動の法だけ。.auf はそのうち「絵と時刻と欄」だけで、拡張編集ごと 1 本の .auf に収まった
2. 「手軽」の芯は、拡張が obj 1 個の読み書き + 頭 1 行の欄で窓が勝手に出る事。Motolii の INPUTS と札(names.rs FIXED)は同じ形(欄は manifest、鍵はホスト)= 既に同型
3. 「fx もレイヤーベース」は、効果の列の中で台本が走り自分の絵と変換を触る事。Motolii の pass/block も列の 1 段 = 同型。「拡張も容易」は .anm 1 file を script/ に置くだけ = vism/ に 1 file と同じ
4. 欠け 1: block は translate+rotate しか書けない(scale は WorldPass で捨てられ、alpha・色・aspect 無し)。AviUtl の `obj.zoom/obj.alpha` 書き戻しに当たる物が無い → Offset に scale と opacity を通すか、色は pass に任せるかは法なので先に相談
5. 欠け 2: Split の単位に番号が無い(写しは `copy>0` で除外、block は層単位)。「文字毎に個別オブジェクト」に当たるのは Stagger の時刻ずらしだけ(schedule_shift、layout.rs:764)
6. 欠け 3: `getvalue(target,time,section)`。Motolii は絵の TIME_OFFSET と LAYER はあるが、数値を層・時刻で引く口が無い(Field の層参照だけ)。値は時刻の純関数なので口を開けても純のまま
7. 欠け 4: `.tra` = イージングが拡張の単位。Motolii の ease は固定名。ISF の型で `STAGE: ease`(t と区間の値列を受け、値 1 つを返す)を足せば同じ棚に載る
8. 要らない物も定規が言う: getpixel 系・copybuffer・DLL(pass shader が肩代わり)、`obj.effect` で内蔵効果を呼ぶ(列に置けば足りる)、カメラ台本(Camera rows)、Lua global(状態は焼くか閉形式)。AviUtl の自由で残す価値があるのは「draw を何度でも」= 複製だが、Motolii はそれを Repeater の札に畳んだ — 意図的な差

## Sources

- scrapbox /aviutl(lua.txt の写し): obj https://scrapbox.io/aviutl/obj 、スクリプトファイル フォーマット https://scrapbox.io/aviutl/%E3%82%B9%E3%82%AF%E3%83%AA%E3%83%97%E3%83%88%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB_%E3%83%95%E3%82%A9%E3%83%BC%E3%83%9E%E3%83%83%E3%83%88 、トラックバーの変化方法(exedit.txt 引用)https://scrapbox.io/aviutl/%E3%83%88%E3%83%A9%E3%83%83%E3%82%AF%E3%83%90%E3%83%BC%E3%81%AE%E5%A4%89%E5%8C%96%E6%96%B9%E6%B3%95 、テキスト(exedit.txt「文字毎に個別オブジェクト」)https://scrapbox.io/aviutl/%E3%83%86%E3%82%AD%E3%82%B9%E3%83%88 、obj.getvalue / obj.getpoint / obj.interpolation / obj.setanchor / obj.getoption("track_mode") / obj.getoption("camera_param") / obj.setoption / obj.load(movie,text,figure,framebuffer,tempbuffer,layer,before) / obj.draw / obj.drawpoly / obj.effect / obj.filter / obj.rand / obj.copybuffer / obj.pixeloption / obj.getpixeldata / obj.getinfo / obj.setfont / obj.mes(各 https://scrapbox.io/aviutl/obj.〜、API https://scrapbox.io/api/pages/aviutl/〜/text で取得)。lua.txt 頁 https://scrapbox.io/aviutl/lua.txt は空(404)
- AviUtl2 Lua 取説(後継、getaudio の型・rand の seed・index/num・旧書式の互換の確認)https://docs.aviutl2.jp/lua/
- AviUtl plugin SDK(0.99k、kbinani mirror): filter.h https://raw.githubusercontent.com/kbinani/aviutl_plugin_sdk/master/filter.h 、input.h …/input.h 、output.h …/output.h(Shift-JIS、原文を scratchpad/filter.h.utf8 に変換)
- 読めなかった物: ocraviutl.fandom.com「Lua/独自関数」(Cloudflare 403)、aviutl.memo.wiki「スクリプトの作り方」(文字化け、要点のみ WebFetch)
- Motolii: motolii/crates/motolii-render/src/compositor/effects/block_program.rs:13,18,31,109,409-425 、motolii/crates/motolii-render/src/engine/blocks.rs:235,431,634 、motolii/crates/motolii-render/src/compositor/effects/isf/mod.rs:26,54,118-130,733 、motolii/crates/motolii-render/vism/{arrive,wave,bounce,hang,field,push_apart,turbulent_displace,blur}.wgsl 、motolii/crates/motolii-doc/src/store/names.rs:10,60 、motolii/crates/motolii-doc/src/store/layout.rs:178-180,250,735,764,785 、motolii/crates/motolii-doc/src/store/view/resolve.rs:1260-1295 、motolii/crates/motolii-doc/src/store/placement.rs:70-100 、docs/reviews/2026-09-14-script-mouth.md 、docs/reviews/2026-09-17-creative-coding-ledger.md

