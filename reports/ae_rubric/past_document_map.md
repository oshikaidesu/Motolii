# 過去の倉庫の文書の形と名前（今の文書をそれに揃えるための地図）

読んだだけ。何も作らず、何も走らせていない。過去 = `~/rust_ae/Motolii`（以下、道は `motolii/` から。`crates/motolii-doc/src/` は `doc/`、`crates/motolii-edit/src/` は `edit/`、`ui/native/src/` は `native/`、`ui/lib/` は `lib/` と略す）。今 = `~/motolii/filament`（`doc.h`、`motolii.h`、`interp.h`）。
「読んだ」= そのファイルのその行に書いてある。「推測」= 書いてある物から私が導いた。

## 1. まとめ

1. 悪い知らせ：過去のガワは Rust の文書を直には呼ばない。間に `ui/native/src/`（port.rs・snapshot.rs、約 9000 行）が居て、ガワの呼び出しは「`request` に JSON 1 個（`{"op":...}`）、返事は status の JSON 1 個」。今の `motolii.h`（C の関数）とは口の形そのものが違う。名前を替えるだけでは繋がらない。
2. 悪い知らせ：過去の時刻は全部「コマ」（整数）か分数（`RationalTime{num,den}`）。今は秒（実数）。ガワに出るのはコマだけ（`frame`、`start`、`duration`、`sourceIn`）。
3. 悪い知らせ：過去は「物」を layer と呼び、種類は `LayerSource` の 8 つ（File / Null / Camera / Stage / Shape / Text / Group / Particles）。今の film・picture・solid は全部 `File` 1 つ（ガワには拡張子から Video / Image / Mesh と出る）。glass に当たる種類は無い。
4. 値：過去は `position`（2 つ組）＋ `position.z`、`scale`（2 つ組、倍率）、`rotation`（度）、`opacity`、`anchor`。今は x・y・z・rotate・size（高さ）。`position.x`／`position.y` の分けた形も過去にある（`position` があればそちらが勝つ）。
5. キーは同じ考え：`Keyframe{t, value, interp, spatial}`。今の「行き方（way）」= 過去の `interp`（そのキーから次のキーまで）。ただし新しいキーの既定は過去 `Linear`、今 `Bezier`。
6. 今の「いつもの数」は 2 つ過去と違う：Bounce（今 1.5/2.75, 0.75 は `gsap.rs` 由来。過去の自前は 0.27, 0.2）と Random（今 0, 0.25, 0.5, 0.8, 0.5。過去 500, 0.15, 0.5, 0.75, 0.5）。過去の自前の表は `native/editor/ease_kinds.rs:5-15`。
7. 今の `define`（式で行き方を足す）は、過去の決め事「属性ごとに式を書く AE の Expression 型は作らない」に当たる（`docs/design/rationale/script-mouth.md:23`）。
8. 保存：過去は Rerun の `.rrd`（履歴は保存しない、開くと取り消しは空）。今は SQLite（取り消しもファイルの中）。中身は今のままでよいと決まっているので、ここは揃えなくてよい所。

## 2. 表「過去 → 今」

列「関係」: 同名 / 別名（同じ物が別の名前）/ 過去だけ / 今だけ / 意味が違う。

### 種類

| 今 | 過去の名前と形 | 所 | 関係 |
|---|---|---|---|
| （物）thing、番号 `int` | layer。`LayerId(pub u64)`、置き場は `/layer/<id>` | `doc/store/ids.rs:10,26` | 別名 |
| film | `LayerSource::File{path, fingerprint}`。ガワへは `"kind":"Video"` | `doc/store.rs:229-234`、`native/snapshot.rs:48-56` | 別名（種類ではなく File の拡張子） |
| picture（写真） | 同じ `File`。ガワへは `"Image"` | 同上 | 別名 |
| picture（AE のファイル） | 層の種類としては見つけていない（`render/src/export/lottie.rs` は書き出し側） | — | 今だけ（推測。読み込み側は読めていない） |
| text | `LayerSource::Text` ＋ `TextDocument`（`Layer:text`） | `doc/store.rs:239`、`doc/store/text.rs:239-247` | 同名（頭が大文字） |
| solid（.glb） | 同じ `File`。ガワへは `"Mesh"`。同梱の立体は `create` の `NewKind::Primitive{path,name}` | `native/snapshot.rs:51`、`native/editor/create.rs:22` | 意味が違う：過去の Solid は「AE の平面」（`doc/store.rs:58`）と「平らな物を立体に起こす効果の族」（`doc/store/kind.rs:70-71`） |
| glass | `LayerSource` に無い | `doc/store.rs:229-243` | 今だけ |
| — | Null、Camera、Stage、Shape、Group、Particles | `doc/store.rs:235-242` | 過去だけ |

### 値の名前

| 今 | 過去の名前と形 | 所 | 関係 |
|---|---|---|---|
| x, y（数 2 つ、別々にキー） | `position`：`Value::Vec2([f64;2])` 1 本。別に `position.x`・`position.y`（`F64`）もあり、`position` があればそちらが勝つ | `doc/store.rs:144-146`、`crates/motolii-render/src/picture/resolve/transform.rs:184-204` | 別名＋持ち方が違う |
| z | `position.z`（`F64`） | `doc/store.rs:158` | 別名 |
| rotate（度） | `rotation`（`F64`、度）。傾きは `rotation.x`・`rotation.y` | `doc/store.rs:148,159-160`、`doc/core/frame.rs:305-331` | 別名 |
| size（高さ、1 = いつもの高さ） | `scale`：`Vec2`、既定 `[1.0,1.0]`（倍率）。奥行きは `scale.z` | `doc/store.rs:147,161`、`edit/document/edit.rs:251-252` | 意味が違う（高さ ↔ 倍率、1 つ ↔ 2 つ） |
| start（秒） | `LayerTiming.start: i64`（comp のコマ）。ガワ `"start"` | `doc/store.rs:299-304`、`native/snapshot.rs:552` | 同名、単位が違う |
| duration（秒。言わなければずっと） | `LayerTiming.duration: i64`（comp のコマ）。必ず在る（`LayerTiming::place` が決める）。ガワ `"duration"` | `doc/store.rs:301,324-337` | 同名、単位が違う、「言わない」が無い |
| begin（秒） | `LayerTiming.source_in: i64`（素材のコマ）。ガワ `"sourceIn"` | `doc/store.rs:302`、`native/port.rs:243` | 別名、単位が違う |
| speed（1 = そのまま、負 = 逆） | `LayerTiming.speed: Speed{num,den}`（分数）。別にキーの打てる `speed` の欄もある | `doc/store.rs:303,468-498,157`、`doc/store/attrs.rs:237` | 同名、意味が違う：「comp が 1 コマ進む間に素材が何コマ進むか」（comp 30 / 素材 24 なら 0.8 が等速）`docs/decision-index.md:522` |
| keyed | 層の値としては無い（効果なら `applyEffect`。緑抜きの効果が在るかは見ていない） | — | 今だけ（推測） |
| blend（normal add multiply screen） | `LayerAttrs.blend_mode: BlendMode`（19 個：Normal, Add, Multiply, Screen, Overlay, …, StencilAlpha, SilhouetteAlpha）。ガワ `"blendMode"`、言葉は頭が大文字 | `doc/store/attrs.rs:7-30,168`、`native/port.rs:30` | 別名、今の 4 つは過去の中に在る |
| file | `LayerSource::File.path` ＋ 素材の表 `Asset{id, name, asset_type, content_hash, path_absolute, path_project_relative, …}` | `doc/store.rs:232`、`doc/store/asset.rs:79-101` | 別名（過去は素材の表を通す） |
| file（text の：フォント） | `TextDocumentStyle.font: FontRef{path, fingerprint, family, style}` | `doc/store/text.rs:10-14,113` | 別名（層の file ではない） |
| shape（ring gem cube） | 無い。過去の shape は線画（`Layer:shapes`、`shape.points`、`shape.size` …） | `doc/store.rs:126-134` | 意味が違う（名前がぶつかる） |
| text | `TextDocument.content: ContentTrack`（`ContentKeyframe{t, content}`）。欄の名前は `content` | `doc/store/text.rs:44-50,240`、`doc/store/names.rs:63` | 別名（`text` は過去では予約語：`doc/store.rs:113-115`） |
| fill（6 桁） | `TextDocumentStyle.fill: [f64;4]`（RGBA、0〜1）。欄は `text_style.<id>.fill_color` | `doc/store/text.rs:115`、`doc/store/names.rs:67` | 別名、持ち方が違う |
| — | `anchor`、`opacity`、`skew`、`skew_axis`、`depth`、`level`、`pan`、`fade_in`、`fade_out`、`time_remap`、`hidden`、`solo`、`locked`、`name`、`parent`、`matte`、`order` | `doc/store.rs:143-163`、`doc/store/attrs.rs:165-194` | 過去だけ |

### キー

| 今 | 過去の名前と形 | 所 | 関係 |
|---|---|---|---|
| key（at 秒, value 数） | `Keyframe{t: RationalTime, value: Value, interp: Interp, spatial: Option<SpatialTangent>}` | `doc/eval/track.rs:449-456` | 同じ物。時刻が分数、値が型つき |
| 値の全部のキー | `KeyframeTrack{keys: Vec<Keyframe>}`（時刻は必ず増える順） | `doc/eval/track.rs:458-506` | 別名（今は名前が無い） |
| stretch / way（そのキーから次まで） | `Keyframe.interp`（区間は始まりのキーの `interp` で進む） | `doc/eval/track.rs:598-603` | 別名 |
| 9 種類の名前と数 | `Interp`：Hold, Linear, Bezier{x1,y1,x2,y2}, Bounce{first_dip,dip}, Elastic{limit,period,damp}, Cyclic{period,peak,linear,envelope_end}, Random{seed,grain,center_u,center_v,bias}, Steps{width,smooth}, ElasticSteps{width,elasticity} | `doc/eval/track.rs:36-60` | 同名 |
| 言わない時 = Bezier | 新しいキーは `Interp::Linear` | `edit/document/edit.rs:22-26`、`edit/document.rs:349` | 意味が違う |
| way の表（式で足す）、`define` | 無い | — | 今だけ（G の決め事に当たる） |
| — | `SpatialTangent{out_tangent, in_tangent}`（位置の道を曲げる） | `doc/eval/track.rs:443-447` | 過去だけ |
| — | 動かない値 `PropertyBase::Constant(Value)`、共有 `Slot`、他の値への `PropertyLink` | `doc/store/slot.rs:27-36` | 動かない値は今も在る（キー 1 つ）。他は過去だけ |

### 呼び出し（`motolii.h`）

| 今 | 過去（ガワが送る op と中身 → Rust の編集） | 所 | 関係 |
|---|---|---|---|
| `motolii_open(document, wide, high)` | チャンネルの `open {path}`／`attach`。新規は op `new`。大きさは op `composition {width,height,fpsNum,fpsDen,durationFrames,background}` | `lib/session/session_files.dart:20,57`、`native/port.rs:282,301` | 別名 |
| `motolii_trouble` | 返事の `error`（文字） | `native/snapshot.rs:353`、`lib/session/session_native.dart:54` | 別名 |
| `motolii_add(kind, file)` | op `create {kind, family?, visibleFrames?}`（kind: text, rectangle, roundedRectangle, ellipse, star, polygon, null, camera, particles, stage, line, bezier）。ファイルは op `import {paths}` → `placeAsset {id, start?, placement?, target?}`。中では `Intent::AddLayer(LayerId)` ほか | `native/editor/create/catalog.rs:7-22`、`native/port.rs:217,285-286`、`edit/document/intent.rs:7` | 意味が違う（ファイルは 2 段） |
| `motolii_remove(thing)` | op `delete`（引数なし。選んである層を消す）。`Intent::RemoveLayer(LayerId)` | `native/port.rs:229`、`edit/document/intent.rs:8` | 別名、相手の言い方が違う（選択） |
| `motolii_set(thing, name, number)` | op `setProperty {layer, property, value}`（value は数／2 つ組／4 つ組）→ `Document::place_checked` → `Intent::SetConstant` か `SetTrack` | `native/port.rs:69-79,198`、`edit/document/edit.rs:38-132` | 意味が違う：キーのある値に置くと、過去は全部のキーを同じだけずらす（`edit.rs:75-86`）。今は全部消す |
| `motolii_say(thing, name, word)` | 文は op `setText {layer, content}`、重ね方は op `setAttrs {layers, patch:{blendMode}}`、色は op `setColor {rgba, …}` | `native/port.rs:208,209,258,26-37` | 別名（3 つに分かれる） |
| `motolii_key(thing, name, at, number)` | op `toggleKey {layer, property}`（時刻は言わない：今の再生位置。値は今出ている値）。または Animate を入れて `setProperty` | `native/port.rs:244-254,263` | 意味が違う（時刻を言わない） |
| `motolii_unkey` | 同じ op `toggleKey`（在れば消す） | `native/port.rs:244` | 別名 |
| `motolii_keys`（数） | status の `layers[].properties[].keys[]`（`{frame, value, interp}`） | `native/snapshot.rs:43-46` | 別名 |
| `motolii_way(thing, name, at, way)` | op `ease {kind, 数…, keys:[{layer, property, frame}]}`（keys が無ければ選んだキー） | `lib/desks/ease/desk.dart:274-277`、`native/port.rs:405-424` | 別名 |
| `motolii_define`、`motolii_ways` | `define` は無い。一覧は status の `easeKinds`（9 つ、いつもの数つき） | `native/snapshot.rs:353` | `define` は今だけ |
| `motolii_value(thing, name, at)` | status の `properties[].value`（今のコマの値）。Rust では `StoreView::value_at(layer, &property, t)` | `native/snapshot.rs:36,46`、`doc/store/view.rs:529` | 別名 |
| `motolii_undo`／`redo` | op `undo`／`redo`／`historyGoto`。`Document::undo()`／`redo()` | `native/port.rs:302-305`、`edit/document/undo.rs:24-42` | 同名 |
| `motolii_seconds` | status の `durationFrames` と `fps`（comp に書いてある長さ。物から数えない） | `native/snapshot.rs:353`、`doc/store.rs:276-283` | 意味が違う |
| `motolii_at(seconds)` | op `seek {frame}` | `native/port.rs:318` | 別名、単位が違う |
| `motolii_still(png)` | op としては見つけていない | — | 今だけ（推測） |
| `motolii_write(mp4)` | op `export {path, start, end}`、`exportStatus`、`cancelExport` | `native/port.rs:293-294` | 別名 |
| `motolii_list` | op `status` | `native/snapshot_cache.rs:129` | 別名 |
| `motolii_timed` | status の `renderMs`、`framesSkipped`、`playbackHealth` | `native/snapshot.rs:353` | 別名（近い物） |
| — | op `save {path}`、`play`、`pause`、`select`、`group`、`ungroup`、`moveLayers`、`setTimings`、`duplicate`、`split`、`copy`／`cut`／`paste`、マーカー、効果、色 ほか（全 90 個） | `lib/bridge/protocol.dart:3-92` | 過去だけ |

## 3. 事実

### A. 文書の形

- 物は **layer**。番号は `LayerId(pub u64)`（`doc/store/ids.rs:10`）。次の番号は `StoreView::next_layer_id()`（`doc/store/view.rs:215`）。
- 種類は `LayerSource`：`File{path, fingerprint}`（旧名 Media、PointCloud も読む）、`Null`、`Camera`、`Stage`、`Shape`、`Text`、`Group`、`Particles`（`doc/store.rs:228-243`）。種類の名前は `kind()` が返す（`doc/store.rs:248-259`）。
- 注意：頼まれた `store/kind.rs` は層の種類ではない。効果の欄の宣言（`Param`、`ParamKind`、`Family{Placement, Path, Solid, Text, Output}`、`Kind{plugin_id, label, params, family}`）と配置（`Placement`）（`doc/store/kind.rs:6-85,229-241`）。
- 層 1 つが持つ記録：`LayerMeta{source: LayerSource, order: i16, timing: LayerTiming}`（`doc/store.rs:500-505`）と `LayerAttrs`（`doc/store/attrs.rs:165-194`）。
- 重ね順は `LayerMeta.order: i16`。入れ子は `LayerAttrs.parent: Option<LayerId>`（親は Group など。子の順は order：`doc/store/layout/time.rs:48-62`）。
- 一番上の入れ物は **Composition** 1 つ：`Composition{width: u32, height: u32, fps: Fps, duration_frames: i64, background: [f32;4]}`（`doc/store.rs:275-283`）。project という型は読んだ範囲に無い（`blank_project()` は Document を返すだけ：`edit/lib.rs:24-36`、1920×1080、30fps、1800 コマ）。
- Composition に付く物：`Composition:settings`、`markers`、`slots`、`assets`、`notebook`、カメラの欄（`doc/store/components.rs:168-198,240-242`、`doc/store/ids.rs:182-188`）。
- 層に付く物：`Layer:meta`、`Layer:attrs`、`Layer:present`、`Layer:masks`、`Layer:effects`、`Layer:shapes`、`Layer:text`、そして値 1 つにつき `Layer:<欄の名前>`（`doc/store/components.rs:144-238`、`doc/store/ids.rs:57`）。
- 欄の名前に使えない言葉：`meta, present, masks, attrs, effects, shapes, text`（`doc/store.rs:113-115`）。

### B. 値

- 値の型は `Value`：`F64(f64)`、`Vec2([f64;2])`、`Color([f64;4])`、`Bool(bool)`、`Path(Path)`、`Enum(i64)`、`LayerId(u64)`（`doc/eval/value.rs:16-25`）。3 つ組は無い。
- 値の名前は `PropertyId`（中身は文字）。書くときは名前の文字そのまま（`doc/store/ids.rs:200-217`）。
- 名前の表（`doc/store.rs:143-163`）：`anchor`、`position`、`position.x`、`position.y`、`position.z`、`scale`、`scale.z`、`rotation`、`rotation.x`、`rotation.y`、`opacity`、`skew`、`skew_axis`、`depth`、`level`、`pan`、`fade_in`、`fade_out`、`time_remap`、`speed`。
- 既定（`edit/document/edit.rs:249-265`）：`position`・`anchor` = `Vec2([0,0])`、`scale` = `Vec2([1,1])`、`scale.z` = 1、`opacity` = 1、`rotation` ほか = 0。
- 単位：`rotation` は度（`doc/core/frame.rs:309,327`）。`scale` は倍率。`opacity` は 0〜1（推測：既定 1.0 から）。`position` は comp の px（推測：`Composition.width/height` が px で、Stage の余白の説明が「comp px」：`doc/store.rs:209`）。
- 組み立ての順：`translation(position) * rotation * skew * scale * translation(-anchor)`（`doc/core/frame.rs:326-330`）。
- 窓に出る名前（英語）は 1 つの表：`doc/store/names.rs:10-51`（Position、Position X、Scale、Rotation、Tilt X、Opacity …）。
- 番号つきの名前：`mask.<id>.shape|opacity|expansion`、`effect.<id>.param.<name>`、`effect.<id>.enabled`、`text_style.<id>.axis.<tag>`、`text_range.<id>.selector.start` など（`doc/store/ids.rs:64-180`、`doc/store.rs:117-123`）。
- スイッチ類は値ではなく `LayerAttrs`：`hidden, parent, blend_mode, matte, clip_to_below, name, auto_orient, projection("2D"|"2.5D"|"3D"), solo, locked, label_color, frozen, flatten, environment, ghost`（`doc/store/attrs.rs:165-194`）。
- **書いた形とガワの形の違い**：Rust の中は snake_case（`blend_mode`、`clip_to_below`、`source_in`、`auto_orient`）。ガワとの JSON は camelCase（`blendMode`、`clipToBelow`、`sourceIn`、`autoOrient`）（`native/port.rs:26-37`、`native/snapshot.rs:552`）。値の名前（`position` など）は両方同じ。
- 文字：`TextDocument{content: ContentTrack, justify, wrap_size, styles: Vec<TextDocumentStyle>, slot_id, ranges, alignment, runs}`、`TextDocumentStyle{id, font: FontRef, size: f32, fill: [f64;4], line_height, tracking, axes, features}`（`doc/store/text.rs:111-119,239-247`）。
- 素材：`Asset{id: AssetId(u64), name, asset_type, content_hash, role, path_absolute, path_project_relative, file_name, size_bytes, head_hash, tail_hash, duration: Option<RationalTime>}`（`doc/store/asset.rs:79-101`）。

### C. 時刻

- `RationalTime{num: i64, den: i64}`（約分して持つ。書くと `{"num":..,"den":..}`）（`doc/core/time.rs:6-10,20-34`）。`try_from_frame(frame, fps)`、`try_to_frame_round(fps)`、`as_seconds_f64()`（`doc/core/time.rs:55-78`）。
- `Fps{num, den}`（`doc/core/time.rs:235-238`）。
- 層の時間 `LayerTiming{start: i64, duration: i64, source_in: i64, speed: Speed}`（`doc/store.rs:298-304`）。出ているのは `start <= コマ < start + duration`（`doc/store.rs:318-320`）。素材のコマ = `source_in + (コマ - start) × speed`（`doc/store.rs:339-344`）。
- `Speed{num, den}`、等速は `Speed::NORMAL = 1/1`、分母は正（`doc/store.rs:467-483`）。
- 決め事：`start`・`duration` は comp のコマ、`source_in` は素材のコマ、`speed` が 2 つを繋ぐ（`docs/decision-index.md:522`）。
- キーの時刻だけが `RationalTime`。層の in／out はコマの整数。「out」という名前は無い（start + duration）。trim はガワの動かし方の名前：`setTimings {changes:[{layer, mode: "move"|"trimStart"|"trimEnd"|"slip", delta}]}`（`native/port.rs:359-366`、`native/editor/functions/verb.rs:12-17`）。
- 繰り返しとずらし：`layout.loop_duration`、`layout.loop_direction`、`layout.stagger`、`layout.stagger_from`、`layout.from_end`（秒の実数）（`doc/store/layout.rs:32,101-106`、`doc/store/layout/clock.rs:98-119`）。今の「duration を言わなければ頭から繰り返す」に近いのは `layout.loop_duration`。
- マーカー：`Marker{name, time: RationalTime, duration: RationalTime, body}`（`doc/store/marker.rs:6-14`）。

### D. キー

- `Keyframe{t, value, interp, spatial}`、`KeyframeTrack{keys}`、`SpatialTangent{out_tangent:[f64;2], in_tangent:[f64;2]}`（`doc/eval/track.rs:443-462`）。
- 読み方：最初のキーより前はその値、最後より後はその値、間は始まりのキーの `interp`（Hold はそのまま）（`doc/eval/track.rs:582-604`）。
- 断る数：Bezier の x1・x2 は 0〜1、Bounce の first_dip は正、Elastic・Cyclic の period は正、Steps・ElasticSteps の width は正、Steps の smooth は 0 以上（`doc/eval/track.rs:507-577`）。
- 値への付け方：値 1 つに `PropertySource{base: Option<PropertyBase>, modulators: Vec<PropertyLink>}`。`PropertyBase` は `Track(KeyframeTrack)`／`Slot(SlotId)`／`Constant(Value)`（`doc/store/slot.rs:21-36`）。
- 置き場：動くものは JSON の文字（`TrackJson`）を `Layer:<名前>` に。動かない数は型のまま（`ValueF64`、`ValueVec2` など）（`doc/store/components.rs:13-14,144-150`、`doc/store/value_components.rs:158-198`）。
- **書かれた形の実例**：ファイルに書き出された見本は見つからなかった（`"interp"` を探して出たのは `native/snapshot.rs` だけ）。下は serde の付け方（`track.rs:36,449`、`value.rs:16`、`time.rs:6`、`slot.rs:27-28`）から私が導いた形（推測。`ease_kinds.rs:280` が `{kind: params}`／`"kind"` の形で読み戻しているので interp の形は裏が取れている）：
  ```json
  {"keys":[
    {"t":{"num":0,"den":1},"value":{"F64":0.0},"interp":{"Bezier":{"x1":0.42,"y1":0.0,"x2":0.58,"y2":1.0}}},
    {"t":{"num":1,"den":2},"value":{"F64":1.0},"interp":"Linear"}
  ]}
  ```
- **ガワへ渡す形**（読んだ：`native/snapshot.rs:22-33,43`）：`{"frame": 15, "value": 1.0, "interp": {"kind":"Bezier","x1":0.42,"y1":0.0,"x2":0.58,"y2":1.0,"handles":[[u,v],…],"overshoots":false,"samples":[[u,v],…]}}`。時刻はコマ、種類は `kind`、数は同じ名前で平らに。
- 見本の組み方は `edit/fixture.rs:21-22`（`RationalTime::try_new(frame, FPS_NUM)`）と `:231-332`（`Interp::Linear`、`Hold`、`Bezier{…}`）。

### E. 編集

- 変更は全部 `Intent`（`edit/document/intent.rs:6-120`）：
  `AddLayer(LayerId)`、`RemoveLayer(LayerId)`、`SetTrack{layer, property, track}`、`SetConstant{layer, property, value}`、`SetPropertySlot{layer, property, slot}`、`SetPropertyLink{layer, property, link}`、`SetPropertyModulators{layer, property, modulators}`、`SetCameraPropertyModulators{property, modulators}`、`SetMeta{layer, meta}`、`SetSource{layer, source}`、`SetOrder{layer, order}`、`SetMasks{layer, masks}`、`AddMask{layer, mask, shape}`、`SetTiming{layer, timing}`、`SetAttrs{layer, patch: LayerAttrsPatch}`、`SetEffects{layer, effects}`、`SetShapes{layer, shapes}`、`SetTextDocument{layer, document}`、`SetComposition(Composition)`、`SetNotebook{notebook}`、`SetMarkers{markers}`、`SetCameraConstant{property, value}`、`SetCameraTrack{property, track}`、`SetCameraPropertySlot{property, slot}`、`SetSlots{slots}`、`AdmitAsset{draft}`、`RemoveAsset{asset}`、`RelinkAsset{asset, path_absolute, project_root}`、`Freeze{group}`、`Unfreeze{group}`。
- 入口：`Document::apply(intent)`、`apply_all(intents)`、`apply_then(first, then)`（何個でも取り消し 1 段。失敗したら全部残らない）（`edit/document.rs:151-239`）。
- キーが生まれる決まりは 1 つ：`Document::place(layer, property, value, t)`（キーが無ければ `SetConstant`、在ればその時刻へ）と `place_checked(…, animate: Animate)`（`edit/document.rs:328-359`、`edit/document/edit.rs:38-132`）。`Animate{Off, Now{interp}, From{origin, interp}}`（`edit/document/edit.rs:14-19`）。
- まとめの操作：`group_layers(&[LayerId])`、`ungroup_layers`、`move_layers(layers, target, placement: &str, at)`、`set_projection`（`edit/document/group.rs:12,61`、`edit/document/group/moving.rs:13-19`、`edit/document/projection.rs:15`）。
- 断り：親の輪、リンクの輪、形の無いマスク、鍵のかかった層、凍った層（`edit/document/validate.rs:12,45,78,102,112`）。
- 試し置き（確定前）：`begin_preview`、`preview_edits(owner, &[Intent])`、`clear_preview_edits`、`set_transient`（`edit/document/edit.rs:134-226`、`edit/document.rs:406`）。
- 取り消し：`head`／`tip`／`floor` の 3 つの数。`apply` で head+1、`undo` は head-1、`redo` は head+1、`history_depth()`（`edit/document.rs:35-37,186`、`edit/document/undo.rs:4-42`）。Rerun の `edit` という並び（`EDIT_TIMELINE`）の上の位置（`doc/store.rs:96`、`edit/document.rs:108-110`）。
- 保存：`Document::save(path)` は平らにして（`flattened()`：今の姿だけ、履歴なし）Rerun の rrd（`PROTOBUF_UNCOMPRESSED`）に書き、`.名前.tmp` から置き換える。`load(path)` の後は取り消しが空（`mark_undo_floor`）。`auto_save` は「<名前> auto-save」の中に世代で（`edit/persist.rs:10-45,47-95,97-140`）。拡張子：ガワの保存の窓は `Untitled.rrd`（`lib/session/session_files.dart:69`）。
- `stagger.rs` は GSAP の `distribute` の写し（`Stagger{amount, each, from, wrap_at, axis, ease: Option<Interp>, seed}`、`From{Start, Center, End, Edges, Random, Index}`）（`edit/stagger.rs:1-41`）。GSAP 由来なので、要らない側かもしれない（持ち主の判断）。

### F. ガワとの約束

- 仕組み：Flutter の MethodChannel `'motolii/probe'`（`lib/bridge/native_bridge.dart:8`）。編集は全部 `invoke('request', {'command': <JSON の文字>, …})`、JSON は `{"op": 名前, …引数}`（`lib/bridge/protocol.dart:112-115`、`native_bridge.dart:40-45`）。
- op の一覧（90 個）は `lib/bridge/protocol.dart:3-92` と `native/port.rs:8`（`CAPABILITIES`）。Rust 側の振り分けは `native/port.rs:179-338`。
- 引数の形（読んだ分）：
  - `setProperty {layer, property, value}`、`previewProperties {edits:[…]}`、`commitPreview`、`cancelPreview`（`port.rs:198-201`）
  - `setAttrs {layers?, patch:{hidden, solo, locked, flatten, environment, clipToBelow, autoOrient, name, parent, blendMode, projection, matte, ghost}}`（`port.rs:26-37,209`）
  - `create {kind, family?, visibleFrames?}`、`delete`、`duplicate`、`group`、`ungroup`、`moveLayers {layers, target, placement}`、`reorder {delta}`、`split`（`port.rs:217-234`）
  - `setTiming {layer, start, duration, sourceIn}`、`setTimings {changes:[{layer, mode, delta}]}`（`port.rs:235,243,359-366`）
  - `toggleKey {layer, property | properties}`、`moveKeys {deltaFrames}`、`ease {kind, 数…, keys?, selection?}`、`animate {enabled|toggle, from, shape}`（`port.rs:244-257,263`）
  - `setText {layer, content}`、`setFont {layer, family, preview?}`、`setColor {rgba, …}`（`port.rs:207-208,258`）
  - `composition {width, height, fpsNum, fpsDen, durationFrames, background}`、`seek {frame}`、`play`、`pause`、`transport {loop, in, out}`（`port.rs:282,312-318`）
  - `import {paths}`、`placeAsset {id, start?, placement?, target?}`、`save {path}`、`new`、`undo`、`redo`、`historyGoto`、`export {path, start, end}`（`port.rs:285-305`）
- op でない呼び出し（チャンネルの名前）：`attach`、`open`、`detach`、`render`、`flushEditors`、`easeModel`、`windowInfo`、`pickOpen`、`pickSave`、`pickImport`、`pickExport`、`catalog` ほか（`lib/session/session_native.dart:10-21`、`session_files.dart:10-75`、`session_render.dart:10,31`、`lib/desks/ease/desk.dart:210`）。
- Rust からガワへ：`documentChanged`、`playbackFrame`、`effectsChanged`、`catalogChanged`、`documentClosed`、`confirmClose` ほか（`lib/session/session_native.dart:126-188`）。
- 返事：包み `{status | statusJson, textureId, textureIds, runtimeEpoch, attachmentId, frameReady, …}`（`lib/bridge/native_bridge.dart:13-23`、`lib/session/session_native.dart:41-67`）。
- status の一番上の名前（`native/snapshot.rs:353`）：`observer, cameraGizmos, width, height, stageWindow, fps, fpsNum, fpsDen, durationFrames, background, frame, playing, playbackHealth, waveforms, undo, redo, path, dirty, layers, selectedId, selectedIds, selectedKeys, selectedBounds, x, y, assets, catalog, catalogErrors, palette, markers, colorTarget, capabilities, easeKinds, documentRevision, deviceId, renderCount, framesSkipped, renderMs, interopCopies, readbacks, error, preview, export, freeze, relink`（行の続きは読み切っていない。`easeIntervals`、`blendTargets` は `:318` に出る）。
- 層 1 行（`native/snapshot.rs:552`）：`{id, name, kind, ghost, ghostable, parent, order, hidden, solo, locked, clipToBelow, clipBase, projection, flatten, environment, frozen, blendMode, matte, start, duration, sourceIn, properties, text, effects, contentKeys}`。
- 値 1 行（`native/snapshot.rs:46`）：`{id, label, link, kind: "number"|"vec2"|"color"|"enum"|"text", value, min, max, keyedNow, keys:[{frame, value, interp}]}`。
- 絵：Flutter の `Texture(textureId: …)`。番号は返事の `textureId`／`textureIds`（見え方の名前 → 番号）（`lib/stage/overlay.dart:6-20`、`lib/session/session_native.dart:57-64`）。
- **Ease の机**：
  - 区間 1 つを直す時：`command('ease', {kind, …数, keys:[{layer, property, frame}]})`。1 回で取り消し 1 段（`lib/desks/ease/desk.dart:272-277`）。
  - 形は `{'kind': 名前, Bezier なら x1,y1,x2,y2、他は model の数の欄全部}`（`desk.dart:199-205`）。
  - 机は自分の数を持たない。種類の並びといつもの数は Rust が status の `easeKinds` で渡す（`desk.dart:9,52`、`native/snapshot.rs:353`）。取っ手を動かした後の形も Rust に訊く（`easeModel {shape, handle, point}`：`desk.dart:210-213`、`native/editor/ease_kinds.rs:284-287`）。
  - **過去の自前の「いつもの数」**（`native/editor/ease_kinds.rs:4-15`、「実機で確かめた既定値(2026-07-19 観察台帳)」）：
    - `Bezier {0.42, 0.0, 0.58, 1.0}`
    - `Bounce {first_dip: 0.27, dip: 0.2}`
    - `Elastic {limit: 1.5, period: 0.3, damp: 0.35}`
    - `Cyclic {period: 2/7, peak: 0.5, linear: 0.0, envelope_end: 0.0}`
    - `Random {seed: 500, grain: 0.15, center_u: 0.5, center_v: 0.75, bias: 0.5}`
    - `Steps {width: 0.178, smooth: 0.0}`
    - `ElasticSteps {width: 0.2, elasticity: 0.5}`
  - 今（`filament/interp.h:20-26`）との食い違い：**Bounce**（今 `1.5/2.75, 0.75`。この数は過去では `doc/eval/gsap.rs:102` にだけ在る = 要らないと言われた側）と **Random**（今 `0, 0.25, 0.5, 0.8, 0.5`）。Cyclic と ElasticSteps と Steps と Elastic と Bezier は過去の表と同じ。
  - 取っ手の動く幅も同じファイル：Bounce first_dip 0.06〜0.7・dip 0〜0.9、Elastic limit 1.02〜2.0・period 0.12〜0.7・damp 0〜0.97、Cyclic period 0.08〜0.95・peak 0.02〜0.98（`ease_kinds.rs:83-84,93,105-106,117,130`）。
  - 注意：`ease` の `kind` が 9 つに無い名前だと `gsap::interp_for` に渡る（`ease_kinds.rs:268-272`）。GSAP の写しを外すならここも外れる。`EasyEase`／`EasyEaseIn`／`EasyEaseOut` は別扱い（`native/port.rs:418-420`）。
  - 机だけで動く時の見本（Rust なし）：Linear、Ease `[.42,0,.58,1]`、Bezier `[.25,.1,.78,.92]`、Spring、Bounce（`lib/desks/ease/face.dart:61-66`）。

### G. 名前と形についての決め事（過去の文書から）

- `docs/design/rationale/script-mouth.md:8`：「縛りは **実物に無いパラメータは無い** — スクリプトの名前は窓に出る名前だけ。」
- 同 `:21`：「**名前は属性の表から作ったものだけ**(`store/kind.rs` の `Param`、効果の ISF manifest の inputs)。窓の英語 label をそのまま名前にする。」
- 同 `:23`：「Document の中で属性ごとに式を書く AE の Expression 型は、引き続き作らない。」（今の `define` に当たる）
- 同 `:47`：「スクリプトは **port の op を呼ぶだけ**…窓でできない操作はスクリプトでもできない。」
- `doc/store/names.rs:1-3`：「名前は窓が既に出している語が先、無ければ AE の語(Mask Path・Tracking Amount)、字の軸は OpenType の登録名。」
- `doc/store/names.rs:54`：「裁定: 欄の名前は CSS の語」（並べ方の欄は `layout.flex_direction` など CSS の名前：`doc/store/layout.rs:23-128`）。
- `doc/store/slot.rs:32-34`：動かない値は「**キーを持たずに値だけ在る**状態(AE の形)」。
- `edit/document/intent.rs:14-15`：「キーは作らない —— 利用者が ◇ を押すまで時間の世界へ入れない(根底3)。」
- `docs/decision-index.md:522`：`start`／`duration` は comp のコマ、`source_in` は素材のコマ、`speed` が繋ぐ。
- `docs/decision-index.md:320`：組み込みの `CommandId` は「`motolii.`+`.`区切りの小文字ASCII意味segment」。

## 4. 揃える時に決めないといけない事

1. **口の形**。過去：JSON の op 1 個 → status 1 個。今：C の関数。ガワをほぼそのまま繋ぐなら、今の中に「op を受けて status を返す」口が要る（`native/port.rs`＋`snapshot.rs` の役）。値段：一番大きい。status の層 1 行・値 1 行の形（F）を今の事実から組む仕事。
2. **選択と再生位置を誰が持つか**。過去の op は「選んである層」「今のコマ」に効く（`delete`、`toggleKey`、`group`）。今は番号と時刻を毎回言う。値段：今の側に選択と今のコマを持たせるか、op を番号つきに読み替えるか。
3. **時刻の書き方**。過去：コマ（整数）と分数。今：秒。値段：文書に fps を持たせる（下の 4）＋ start／duration／begin／キーの時刻の全部の書き直し。秒のまま持って口でコマに直すだけ、という手もある（その時は丸めの決まりが要る）。
4. **Composition**。過去は幅・高さ・fps・長さ・地の色を文書が持つ。今は舞台 1 つで、長さは物から数え、fps は無い。値段：事実を 5 つ足す。`motolii_seconds` の意味が「書いてある長さ」に変わる。
5. **位置を 1 本か 3 本か**。過去：`position`（2 つ組 1 本、キーも 1 本、道を曲げる `spatial` つき）＋ `position.z`。分けた `position.x`／`position.y` も過去に在るので、今の 3 本をこの名前に替えるだけなら安い。2 つ組 1 本にするなら「値が数 1 つ」という今の持ち方を変える事になる。
6. **座標の物差し**。過去：comp の px（推測。上下の向きは確かめていない）。今：高さ 3.2 の世界、真ん中 0、上が正。値段：どちらで書くかと、直す式。
7. **size と scale**。過去：倍率の 2 つ組（縦横別）。今：高さ 1 つ。値段：名前だけ `scale` にすると意味が違ってしまう。倍率に変えるか、名前を変えないか。
8. **種類の分け方**。過去：File 1 つ（拡張子で Video／Image／Mesh）と Text。今：film／picture／solid／text／glass。値段：`add` を「import してから place」の 2 段にするか、op `create`／`placeAsset` に今の 1 段を被せるか。glass は過去に席が無い（足すなら新しい種類か、`create` の kind 1 つ）。「solid」「shape」は過去で別の意味なので、そのままは使えない。
9. **素材の表**。過去はファイルを `Asset` の表に入れてから層が指す。今は層がファイルの名前を直に持つ。値段：表を足すか、ガワの Media の棚を使わないか。
10. **文字**。過去：`content` はキーの打てる列、色・フォント・大きさは style ごと（RGBA 0〜1）。今：`text`・`fill`（6 桁）・file（フォント）。値段：名前は `content` に替えられる。色は 4 つ組にするかどうか。style の番号を持つかどうか。
11. **speed の意味**。過去：素材のコマ ÷ comp のコマ（分数、fps の差もここ）。今：1 = そのまま。値段：fps を持つなら過去の意味に、持たないなら今のまま（名前は同じなので取り違えやすい）。
12. **duration を言わない**。過去は必ず在る。今は「言わなければずっと、繰り返す」。値段：置く時に長さを決める（過去の `LayerTiming::place`）＋繰り返しは `layout.loop_duration` に分ける、か、今のまま。
13. **キーのある値に置いた時**。過去：全部のキーを同じだけずらす。今：全部消す。どちらかに決める。
14. **新しいキーの行き方**。過去 `Linear`、今 `Bezier`（ゆるく）。ガワは Animate の `shape` で選べる。
15. **いつもの数**。Bounce と Random を過去の自前の表（`ease_kinds.rs:5-15`）に替えるか。今の Bounce の数は GSAP の写しから来ている。
16. **`define`（式の行き方）**。過去の決め事は「作らない」。残すなら過去のガワには席が無い（`ease` の kind は 9 つ＋GSAP の名前だけ）。
17. **重ね順と親**。過去：`order: i16` と `parent`。今：z と置いた順。値段：事実を 2 つ足す。ガワの Timeline は order で並ぶ。
18. **スイッチ類**（hidden、solo、locked、name、blend_mode、matte）。過去は値ではなく attrs 1 かたまり、キーは打てない。今の blend はこの形に近い。名前を `blend_mode`／`blendMode` に替えるだけで済む。

## 5. 読めなかった所

- `native/port.rs` の細かい所（1 行が長い。op ごとの引数は上に書いた分だけ読んだ。`select`、`stageGesture`、`nudge`、`relate`、色、効果、マーカーは引数を追っていない）。status の一番上の行は途中まで。
- `doc/store/view.rs` の中身（名前の一覧だけ）、`doc/store/layout.rs` の表、`doc/store/particles.rs`、`mask.rs`、`effect.rs`、`doc/vector/*`、`doc/core/camera.rs`。
- `edit/document/apply.rs`、`group.rs`／`moving.rs` の中身（名前と引数だけ）、`text_edit.rs`。
- `crates/motolii-glue`、`motolii-render`、`motolii-script`（transform の所と数行だけ）。
- `lib/timeline/`、`lib/inspector/`、`lib/stage/`（ガワのどの画面が status のどの名前を読むか）。`lib/desks/ease/face.dart` は見本の表と名前だけ。
- `MANIFESTO.ja.md`、`VISION.ja.md`、`v1/`（`v1/core`、`v1/skin`、`v1/PLAN/*` という別の代が在る。開いていない）、`docs/decision-index.md` は名前で引っかかった行だけ。
- comp の座標の向き（上が正か下が正か）、opacity の幅、AE／Lottie のファイルを層として読めるか、緑抜きの効果が在るか、絵 1 枚を書き出す op が在るか。
- 書き出された rrd の中身の実物（走らせていないので、D の JSON は導いた形）。
