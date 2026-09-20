# 最小コア化 — 2026-09-19進捗・引き継ぎ

この文書は本作業の到達点を固定した記録。作成前のmainは `4b1ba7a87`、作業ツリーはクリーン。以下の実装はmainへ反映済み。責任の現行定義は[modules.md](modules.md)、機械検査の対象は[modules.json](modules.json)。

## 結論

**小さい読み取り・再生基盤に、編集と表現を拡張として載せる方向へ進んだ。ただし最小コア化全体は未完。**

利用者のAviUtl的な意図は、プレイヤー以外の機能を捨てることではなく、機能追加で中央の実装を肥大化させず、責任と接続先を見通せる構造にすること。行数・ファイル数・crate数だけで完成を判定しない。進捗率は、全体の分母を定義できていないため算出しない。

大きな到達点は次の3つ。

1. **作品と閲覧の責任分離**：Document/Intent/UndoとViewerStateを分離し、UIの破棄を作品の終了と切り離した。
2. **編集なしのビルド**：描画とjobsは、編集Document・Intent・Undoを含めずにビルドできる。読み取り用Recordingが実経路に接続された。
3. **効果の計算を外へ**：配置プログラムの入力・出力を定義し、Repeater・Mirror・Blobの配置計算、Motionのサンプル方針を拡張側へ寄せた。

## 定規

- 作品の値・構造・キー・時間軸・UndoはDocumentが唯一の正本。変更はIntent経由。
- 選択・現在見ている時刻・観測カメラ・表示範囲・フォーカスは閲覧側。パネルは第二のDocumentにならない。
- UIの終了、閲覧接続の終了、作品の終了を同一視しない。
- 拡張へは必要な値・読み取り・操作要求の口を渡す。編集権限を丸ごと渡さない。
- 保存形式・既存機能を維持する。未決定の「idと時刻だけ」案は採用していない。
- CPUの常時pollや不変入力の再計算を既定にしない。機能を足すたびに全体を再構築する構造を避ける。
- 静的検査・ビルド成功と実窓の操作成功を区別する。検査の上限を緩めて成功扱いにしない。

## 機能と責任の現在地

以下の実装パスは `motolii/` 以下。

| 機能・責任 | 所有者／接続先 | 到達点・限界 |
|---|---|---|
| 作品の編集・Undo・保存 | `crates/motolii-doc/src/store/document.rs`、Intent、persist | 唯一の書き込み経路を維持。読み取りから編集命令の解釈を除去 |
| 保存作品の読み取り | `store/recording.rs` のRecording | RRD decoderはDocumentと共有。編集・Undoを公開せず、確定した編集位置を保持。一時編集は含めない |
| プレビュー投影・読取cache | 編集側projection → StoreView/read | 入力変更時に読み取り値へ投影。同件数の異なるプレビューも区別。一時値を除くViewでのlayout cache混入を修正 |
| 選択・閲覧時刻・観測・ポインタ | `ui/native/src/viewer.rs` | Documentから分離。製品ではまだ一つの連動ViewerStateを使用 |
| 再生時計・音声transport | `crates/motolii-render/src/playback.rs` | StoreViewを読み、編集Document・Flutter・パネルに依存しない |
| 窓・UIの接続寿命 | Swift WindowAttachment、Dart EditorSession | 古いdetachが後継接続を壊さない。UI破棄で共有作品をclose/pauseしない。共有描画の駆動は主窓 |
| 新しいランタイムへの表示接続 | runtimeEpoch、Stage | 同寸法の作品を開き直した際の描画欠落を修正。接続IDと作品の版は別物 |
| JS実行 | `ui/extensions/script` | Hostのcommand/query経由。JS VMを編集本体から分離 |
| 書き出し・Freeze | `ui/extensions/jobs` | 受付検証後に編集側からRecordingを受け取る。入口・workerともDocument/Intentを直接受け取らない |
| 配置効果 | PlacementProgram／PlacementInput／PlacementOutput | 静的なRust関数をViewへ渡せる。同梱のRepeater・Mirror・Blobも同じ口を使用 |
| Blob Track | 画像解析はrender、配置計算は`extensions/blob.rs` | 解析データを借りて配置を返す。飛び番IDと配列順序を分離。グループの余分な子の表示・無効化時の隠蔽を修正 |
| Motion Blur | `extensions/motion.rs` のSampling | サンプル選択方針を分離。コアは変換を読み、平均合成用の結果へ適用。汎用的な外部Motion登録は未実装 |
| Text Morph・WGSL | render側extensions／vism | Text Morphは保存コアの外。WGSLは既存manifestとstageの契約を利用 |
| UIの構成 | app/workspace/input/foundation/panels/session/bridge | 依存方向を検査。棚の選択で不変の内容・タグ・入力欄を再利用。既存のMaterial祖先エラーを原因修正 |

旧実装1,770ファイルを履歴へ退役させた整理も反映済み。復元先はGit履歴であり、新しい作業で旧UIを再び正本にしない。

## ビルド待ちについて

`motolii-doc`は既定で`editing`を有効にする。読み取り用途は`default-features = false`を指定する。

- 編集なしのdoc、描画、jobsのビルド成功を確認した。
- compiler dep-infoでも、読み取り専用構成から`document.rs`・`persist.rs`・`text_edit.rs`が外れることを確認した。
- 描画・jobsのテスト用fixtureだけはdev-dependencyとして編集機能を使う。
- Cargo featureは加算される。編集アプリ全体のビルドでは編集機能を含む。
- **物理的なcrate分割の完了、ビルド時間短縮の実測、拡張変更で常にコア再ビルド不要になったことは、まだ証明していない。**
- UI変更はhot reload、Rust変更だけnative build。小さい変更で全検査・全構成を毎回ビルドしない。

## 検証結果

結果は各対象の直近の検証時点。全項目を最新mainに対して一括実行した結果ではない。

| 対象 | 確認した結果 |
|---|---|
| 最新のdoc単体／編集transaction／配置・Motion投影 | **129／19／4件成功**。確定値layout cacheの再利用も追加assertionで確認 |
| Recordingの権限制約 | Undoを呼べないcompile-fail test成功。読み込み同値・Undo位置・一時値除外を確認 |
| jobs | **5件成功**。無効な受付でコピーしない、snapshot失敗、保存ゼロとキャンセルを区別 |
| 読み取り専用ビルド | doc成功。描画・jobsも専用入口で成功を確認 |
| Flutter全体 | **177成功・3失敗**。残りは全体layout 2件、作品選択変更時layout 1件 |
| 棚の選択 | build **199→87**、layout **43→20**。既存上限を通過。入力保持・現在の操作先・狭幅を検査 |
| 構成ルール | 通常検査成功。依存ルール9ケースをCIへ追加。明示的editing/default要求を拒否 |
| Freeze GPU検査 | 埋め込みshader構成で形・動画の**2件成功**。画像化変更を外す対照で形の保存失敗を再現 |
| 画像解析GPU検査 | **4成功・1失敗**。Track Overlayの枠線位置。同変更を外した対照でも失敗 |

### 実窓で確認済み

- Dockの赤字`No Material widget found`の解消、タブ操作・ドラッグ。
- Inspectorの別窓表示、別窓を閉じても主窓の再生が継続、UI再起動後の作品・選択・表示保持。
- 同寸法の作品再open後のStage/Camera描画。
- Fonts棚の選択枠・名前・タグ更新と狭幅表示。
- Recording経由のexport：**640×360、30fps、300フレーム、10秒のH.264**。出力画像の矩形も確認。
- 修正後のShape Freeze：**300画像＋300metadata**を生成。別フレームの表示、解除後の表示保持とcache削除を確認。
- 検証後に元の保全作品を開き直し、保全ファイルのハッシュ不変を確認。

これらは全素材・全効果・全操作の検収ではない。

### 配置／Blob／Motion分離後の実窓検収（完了）

`5d5c08e42` のnativeを実窓へ反映して確認した。検証作品は[minimal_core_read_views.js](../../motolii/ui/native/src/editor/script/examples/minimal_core_read_views.js)。

- Blobは Frame 15 でも白い素材へ追従。効果を無効化すると元の子が戻る。
- Edit メニューの Undo で追従配置が復元。
- Motion Blur のサンプルは移動方向の前後両側へ伸びる。中心合わせの方針（[motion.rs:49](../../motolii/crates/motolii-render/src/extensions/motion.rs:50) の `sample_times` が `-0.5 → +0.5`）どおりで、先例の Alight Motion・AE の既定と一致する。
- 保留していた Cmd+Z は**実装の問題ではなかった**。手では undo・redo・cut・copy・paste・select all すべて動く。反応しなかったのは自動操作側の事情。`MainMenu.xib` の First Responder 宛て key equivalent は無効時に鍵を消費しないため、衝突していない。

## core内の評価組み立てを切った(残件1の一部)

`store/layout.rs` は2,837行の1塊で、`impl StoreView` 1,400行に5つの責任が同居し、`thread_local!` が5つ隠れていた。隠れた大域を外してから、責任ごとに切った。

### 隠れた大域(5つとも撤去)

| 大域 | 正体 | 始末 |
|---|---|---|
| `PHYSICS_SHIFTS` | 物理がGPUのblockへ移った時(`4bcf5bb22`)に、書き手が空のmapしか置かなくなった。読みは常に0 | 削除 |
| `DEPTH` | 配置を解く入れ子の深さ | viewの`Scratch`へ |
| `ANCHORING`・`ROUTING` | 付き合い・線の輪の巡り止め | viewの`Scratch`へ |
| `KIDS` | 箱の子の順のcache。捨てる口が無く版ごとに溜まる | viewの`Scratch`へ(viewと共に死ぬ) |

止め具とcacheがthread_localだったため、同じthreadの**別のview同士が共有**していた。片方の巡り止めが他方の巡りを止め得る形で、これは版でも時刻でも区別されない。

### 責任ごとの分割

| 責任 | 実装 | 行 | 外への口 |
|---|---|---|---|
| 時刻(順番の札・Loop・移り方) | `store/layout/time.rs` | 185 | 0 |
| 道(Offset Path・角丸の閉じた道) | `store/layout/path.rs` | 87 | 0 |
| 文字(落ちる場所・折り返し・回り込み) | `store/layout/text.rs` | 128 | 2 |
| 並べる(flex・grid、解き手はtaffy) | `store/layout/flow.rs` | 557 | 5 |
| 箱と座標(素材の箱・奥行き・切り・付いて置くずれ) | `store/layout/boxes.rs` | 484 | 4(+text宛て1) |
| 入口と契約(`layout_frame`・`Frame`/`Slot`/`Scratch`・欄の表) | `store/layout.rs` | 562 | — |

分割の良し悪しは**file数ではなく継ぎ目の口の数**で測る。timeとpathは0口で閉じた(元から独立していた責任)。flow→boxesは最初8口で、これは線が本当の境目の手前にあった印。奥行き・回し方・アンカーを部品で訊いていたのを`Extent`1つに畳んで4口にした。

### 何を描くかの組み立て(`store/view/resolve.rs`)

同じ手順で 1,850 行を切った。

| 責任 | 実装 | 行 | 外への口 |
|---|---|---|---|
| 観測(効いている Stage と Camera、その姿勢) | `resolve/camera.rs` | 314 | 0 |
| 文字の書類 | `resolve/text.rs` | 127 | 0 |
| 切り(自分のマスク・Matte・祖先の箱) | `resolve/mask.rs` | 314 | 1 |
| 効果の列(自分・グループから降りる・子/板) | `resolve/effects.rs` | 168 | 2 |
| 整える手(背景 → 面 → 線 → 型紙 → 格子 → Field) | `resolve/settle.rs` | 341 | 1 |
| 写しを積む(配置・Motion Blur・Split・Ghost) | `resolve/copies.rs` | 335 | 2 |
| 変換 | `resolve/transform.rs` | 377 | 既存 |
| 入口と組み立て(`resolve`・`resolved_layers`・solo/隠し/重ね方) | `resolve.rs` | 357 | — |

切る時に 2 度畳んだ。切りは `resolved_masks` と `clipped_masks` を呼ぶ側が合成していた(順番を間違えると祖先の箱が自分のマスクの前に掛かる)ので、`masks_of` 1 口へ。整える手は 7 つの pass の**順番が法**なのに、その順番が `resolved_layers` の記憶にあったので、`settle` 1 口へ移した。

solo・隠し・重ね方・Matte も切りかけたが**取り消した**。5 つの口に対して中身が 5 つで、全部が外から呼ばれる — それは境界ではなく file を増やしただけになる。複数の module が共有する語彙なので、入口に残す。

### 機能の名前をコアから抜く(2026-09-19 夜)

「最小コア」を行数で測るのをやめ、**コアが機能の名前を呼んでいる箇所の数**で測った。開始時は 9 本(うち `use super::*` の陰に隠れていた 2 本を含め、実数はもっと多かった)。

| 逆流 | 正体 | 始末 |
|---|---|---|
| `placement::noise` (3 箇所) | 拡張ではなく種から決まる乱数。形の縁の揺らぎと粒の散りが拡張の戸を通って読んでいた | `core/noise.rs` へ戻す |
| `motion::is_motion_blur` / `motion::sampling` | Motion Blur を名指しして、ぼかすかどうかと時刻の刻みを訊いていた | `SamplingProgram` + `Shutter`(開く欄・幅・枚数・刻み)。道のりを測るのはコア、枚数と刻みは効果 |
| `overlay::is_track_overlay` / `edge_lines` ほか | どの辺が同じ格子の線に乗るかを拡張の関数で解いていた | `SnapProgram` + `Snapping`(強さ・まとめ具合・線を立てる関数) |
| `placement::picks` / `TRANSFORM_WHOLE` | どの子をどの写しが引くか、全体が 1 つとして動くか | `PlacementProgram` の `pick` と `moves_whole` |
| `extensions::placement_program` ほか 3 本の lookup | コアが同梱の一式を名指しして view へ渡していた | `Programs` 1 枚の表にまとめ、`Document`/`Recording` が持つ。**既定は `Programs::NONE`(効果ゼロ)** |

結果、**コアの実装から同梱の効果への参照は 0**。効果は書類を開く側が渡す:

```rust
// motolii/ui/native/src/lib.rs
Document::load(path)?.with_programs(doc::extensions::bundled())
```

この逆転で、同梱一式に黙って寄りかかっていた test が 4 件露見した(見本の `blank_project()` を含む)。どれも必要な効果を自分で登録する形へ直した。

### 同梱の効果をコアの外へ(物理)

参照が 0 になったので、`motolii-doc/src/extensions/` を `motolii-render/src/extensions/` へ移した。行き先は「contracts live in doc; implementations are selected here」と既に書いてあった場所で、**家は増えていない**(doc・render・ui・vism・tests のまま)。

| | 前 | 後 |
|---|---|---|
| `motolii-doc` | 77 file / 22,374 行 | **71 file / 21,259 行** |
| 同梱の効果 | doc の中 | `motolii-render/src/extensions/` 10 file / 1,687 行 |

**コアは効果を 1 つも compile しない。** 契約(`store::kind` の `PlacementProgram`・`SamplingProgram`・`SnapProgram`・`Programs`)だけが残る。

移動で露見した物:

- 見本の `blank_project()` は効果ゼロになった。効果を使う検査は `with_programs` で自分が要る物を登録する。
- 同梱の振る舞いを見ていた結合 test 5 件(Repeater の Transform、Group の子の引き、Motion のサンプル、Blob の飛び番)は `motolii-render/tests/bundled_effects.rs` へ移した。効果の検査は効果の家に置く。
- コアに残った 1 件(写しの重なり順)は、必要な配置効果を**その test 自身が定義する**形にした。何を試しているかが test に書いてある。

### 再 build の範囲(実測)

進捗文書が「まだ証明していない」と書いていた「拡張変更でコア再 build 不要」を測った。同じ機械・warm な状態で `cargo build -p motolii-ui`。

| 触った物 | 再 build される crate | 時間 |
|---|---|---|
| 同梱の効果(移設前 = コアの中) | **doc** → render → jobs → ui | 31.4s |
| 同梱の効果(移設後 = render の中) | render → jobs → ui | **24.9s** |
| コア(`store/layout/flow.rs`) | doc → render → jobs → ui | 29.2s |
| 拡張 crate(`ui/extensions/jobs`) | jobs → ui | 14.6s |

**効果を触ってもコアは再 build されない。** これは移設の前は成り立っていなかった。ただし render が太いので時間は 31.4 → 24.9s にしか縮んでいない — 「コアを触らない」は達成したが、「速くなった」はまだ小さい。次に効くのは render 側の分割で、そこは今回の範囲外。

### 今夜の検証(回帰は無し)

| 対象 | 結果 |
|---|---|
| doc 単体 | **111 成功** |
| doc 編集 transaction | **16 成功** |
| doc 配置 program(効果の受け渡しを含む) | **3 成功** |
| 同梱の効果(`motolii-render/tests/bundled_effects.rs`) | **5 成功** |
| doc `owned_budget` | 失敗(既知、残件7) |
| render 単体 | 218 成功・**13 失敗** |
| ui 単体 | 86 成功・**5 失敗** |
| 構成検査・native build | PASS |

失敗 18 件は**全て今夜の作業前から同じ**ことを確認した。render の 13 件は `dec0eddbb` で同じ名前・同じ数が失敗。ui の 5 件は、2 件が baseline で同一の diff、2 件は単独実行では成功(全 suite 同時実行の GPU の取り合い)、残る `the_cage_follows_the_drawn_text_under_an_orbited_camera` は構造の作業前の `1dd075511` でも失敗する。

途中、効果の移設で render の contract 24 件が倒れたが、これは**移設の仕事が終わっていなかったため**(render が自分の効果を登録していなかった)。登録を足して 13 = 元からの数に戻した。

### 渡し忘れは黙って効果を消す(構造で塞いだ)

コアの既定が効果ゼロになったので、`with_programs` を通さずに作った作品は**効果が黙って効かない**。実際に踏んだ:

- `File ▸ New` と空起動が効果ゼロの作品を開く状態を作ってしまった(`b1b6eb34a` で修正)。
- render の contract 24 件と examples が同様に倒れた(Repeater が 3 枚ではなく 1 枚)。効果を実装する家が自分で登録する形にした(`78f5a15e8`)。

編集機が作品を作る口を `motolii/ui/native/src/lib.rs` の `work()` 1 つに絞り、**他の file が作品を作れば検査が FAIL** するようにした(`effectRegistration`)。偽の違反で FAIL することを確認済み。

保存形式は変えていない。今夜の `store.rs` の差分は註釈 1 行だけで、**以前の作品はそのまま開く**。

### 次の境界: 層の種類(未着手、朝の相談待ち)

効果の次に大きい結合は**層の種類**。コアは `LayerSource::{Shape, Text, Group, File, Camera, Stage, Null, Particles}` で **44 箇所**分岐している(test を除く)。

| 箇所 | 実装 |
|---|---|
| 10 | `store/layout/boxes.rs` — 素の箱が種類ごと(形は輪郭、文字は文字の箱、素材は寸法、Group は並べた大きさ) |
| 7 | `store/view/resolve/settle.rs` |
| 5 | `store/view/resolve.rs` |
| 4 | `store/view/resolve/copies.rs` |
| 18 | その他 11 file |

ここを効果と同じ形(種類が自分の箱と輪郭を申告する)にすれば、`vector/`(3,814行)がコアを離れられる。`vector::` への参照はコアの 8 file に 43 箇所で、偏りは `layout/boxes.rs` 13・`store/shape_props.rs` 10・`store/connect.rs` 7 — 形と文字と線に集中している(＝種類ごとに固まっている)。

**ただし `LayerSource` は保存形式に載る enum。** 振る舞いだけを表へ移すなら形式は変わらないが、AviUtl のように**外の人が種類を足せる**ようにするなら plugin id の文字列へ開くことになり、これは意味の決定なので勝手にやらない。今夜はここで止める。

### 構造で守る(検査)

- coreに`thread_local!`・`static mut`・契約が名指ししない可変staticがあればFAIL(名指しは`docs/stage5/modules.json`の`coreProcessState`。今日の時点では文字の`SYSTEM`・`KNOWN`だけ)。
- taffyの名を`store/layout/flow.rs`以外が口にすればFAIL(`coreSolvers`)。文字のmeasureはtaffyの`Size`・`AvailableSpace`で話していたので、平の数へ直した。
- どちらも偽の違反を入れてFAILすることを確認済み。

**まだ物理的なcrate分割ではない。** 5つのmoduleは同じcrateの中で、`impl StoreView` を分け合っている。

## 残件・注意点

1. **最小コアの完成は未証明**：次の塊は `store/document/group.rs`(1,249行)と `vector.rs`+`vector/`(約2,700行)、`eval/track.rs`(647行)。ファイル分割やfeature分離を、完全な物理分離と呼ばない。
2. 独立した選択・閲覧時刻を窓ごとに持つ製品接続は未実装。別窓からの再生開始は実窓未検収。
3. ~~直近のBlob／Motion分離の実窓検収~~ **完了**（後述）。
4. Flutterのlayout上限超過3件。上限は据え置き。
5. Track Overlayの枠線位置の既存失敗。今回のFreeze修正を外しても再現した。
6. debug用shader監視のFSEvents開始待ち。devを閉じた単独テストでも再現。スタックは`FileServer::watch → notify → FSEventStreamStart`で、GPU競合と断定しない。
7. `owned_budget`未合格：compute pipeline **2/0**、shader module **2/0**、bind-group layout **2/0**、render pass **4/3**、GPU無期限待機 **18/4**、naga parser **4/3**（実数/上限）。上限を引き上げていない。
8. 第三者コードの動的ロード・権限・sandbox・障害隔離、GUI登録、配布ABI管理は未実装。今回の関数型や接続IDはセキュリティ境界ではない。
9. payload/snapshotの完全な型付け、Session永続化、全機能の検収も残る。既存監査[#481](https://github.com/oshikaidesu/Motolii/issues/481)全件を解決したものではない。

## 再開時の順序

1. `motolii/AGENTS.md`、この文書、modules.json、Git状態を確認。未コミット変更を保護する。
2. 残るcore内の評価組み立てを依存単位で切る。新しい汎用registryやsandboxを先回りして作らない。
3. layout 3件、Track Overlay、監視待ち、実装量上限を、対象と根拠を分けて処理する。
4. 完了時は目的ごとの証拠を確認し、未検証を未完として残す。ビルド速度は再ビルド範囲を確認してから必要な測定を行う。

軽い検査・対象別の入口：

```sh
python3 scripts/check-stage5.py --self-test
scripts/motolii-ui.sh check
git diff --check
cargo test -p motolii-doc --test placement_programs
cargo test -p motolii-doc --lib --test edit_transactions
scripts/motolii-ui.sh check-read-only
```

Rustの依存環境は[CONTRIBUTING](../../CONTRIBUTING.md)と開発スクリプトに従う。`IS_IN_RERUN_WORKSPACE=no`のGPUテストは上流の埋め込みshader構成であり、debug監視を修復した証拠ではない。監視待ちを隠すため常用設定を書き換えない。

## 主要な追跡点

文書化後の追記：配置programの`needs_position`を表示の識別値へ含める修正。修正前は同一関数の入力条件を変えても識別値が衝突することを再現し、修正後は位置入力による配置の変化と識別の分離を確認した。対象の投影3テスト成功。GPU・実窓の追加検証は行っていない。

続く追記：一時編集を読むViewと除外するViewの表示識別を分離。除外した一時値・プレビューの更新では確定値Viewの識別を変えず、確定値layoutの同一cacheを再利用する。修正前の識別衝突を再現し、修正後の値・識別・cache再利用を検査済み。保存形式は変更していない。

| commit | 内容 |
|---|---|
| `7ca2d8436` | ViewerStateをDocumentから分離 |
| `3704f6f8c` | UI接続寿命・共有描画・runtimeEpoch |
| `37dc3b8bf` | 棚の選択と不変内容の再構築を分離 |
| `11371b29c` | 確定値Viewへの一時layout cache混入を修正 |
| `9de838126` | 読み取り専用Recording |
| `d36608692` | 編集なしのdoc／描画ビルド |
| `9b835de95` | jobs入口から編集Document依存を除去 |
| `858368ab3` | 素材画像取得・Freeze保存ゼロの修正 |
| `de7c0372d`・`7132f8aa4` | 配置programとBlobの分離 |
| `e78290e09` | Motionのサンプル方針を分離 |
| `4b1ba7a87` | 読み取り専用依存の明示feature抜け道を検査 |
| `524de0ea7` | scrub中のヘッドをtimeline全体の再構築から外す |
| `0b9886c91` | 物理の死んだ大域を撤去 |
| `f3055fb62` | 止め具・cacheをviewの`Scratch`へ、隠れた大域を検査で禁止 |
| `851a14b97` | layoutを時刻・文字・並べる・道・箱へ分割、解き手の名は1箇所 |
| `4e55c0377` | flow→boxesの継ぎ目を`Extent`1つへ(8口→4口) |
| `c8dbf2be6` | Motion Blur を `SamplingProgram`/`Shutter` で受け取る |
| `eb0e60b84` | 格子寄せと子の引きを効果の申告へ |
| `10c35273f` | 3 本の lookup を `Programs` 1 枚へ |
| `e0dc63908` | 効果は書類を開く側が渡す(既定は効果ゼロ) |
| `94351a6cd` | 同梱の効果を `motolii-render/src/extensions/` へ移設 |
| `d85909621` | `push_placements`の説明をapply_fieldsの上から戻す(元からの取り違え) |
| `8351ecc68` | resolveを観測・文字・切り・効果列・整える手・写しへ分割 |
