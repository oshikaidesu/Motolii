# ペルソナ「モーショングラフィックス制作者」で最短の欠落を洗い出す

日付: 2026-08-22 / 状態: **調査**(read-only・書き込みは本ファイルのみ、コード変更ゼロ) / 起点: 利用者の危機感「空中分解しかけそう。足りてない機能を最短で洗い出す」

対象: `next/`(正本 workspace)。ペルソナは**素材を撮らず図形と文字で作る**制作者 — この製品の意味の正本が Lottie(モーショングラフィックス交換形式)である以上、本来いちばん噛み合うはずの利用者。ワークフローは直列なので、最初に壊れる工程がそのまま「最短で足りない物」の答えになる。

## 0. 結論(先出し)

**工程2「図形を描く」の手前で全系統が同時に壊れている。** 原因は1つではなく3つが重なっている:

1. Stage に**そもそも図形/パスを描く手が無い**(Pen/Rectangle/Ellipse ツールがgizmoに存在しない — 選択・移動・拡縮・回転のハンドルのみ)
2. Browser の create タブに Rectangle/Ellipse カードは実在し `LayerSource::Shape` の layer を生成する**が、パス本体(`Layer:shapes` component)を一度も書かない**(`shell/motolii-shell/src/lib.rs:1548` `create_from_card` に `Intent::SetShapes` が無い)
3. たとえパスがあっても、**engine が shape layer を描画経路に一切繋いでいない**(`engine/motolii-engine/src/lib.rs:926` — `LayerSource::Shape` は `Null`/`Group` と同じ「テクスチャを焼かない」枝)

この3つのどれか1つを直しても、残り2つが効いて「何も見えない」が続く。**「作ったつもりの図形が画面に何も出ない」が最初の致命傷**であり、この時点でペルソナのワークフローは工程2で完全停止する。工程3〜8(整える/マスク/エフェクト)はこの上に積む工程なので実質未着手のまま連鎖して詰まる。

もう1つの独立した致命傷: **テキストレイヤーを作る入口が UI に存在しない**(`CreateKind` に `Text` variant が無い、メニューにも `New Text Layer` が無い)。テキストの**中身**(`LayerSource::Text`・`TextDocument`・cosmic-text 実描画・Inspector TEXT section)は裁定190で実装済みで、既存プロジェクトを開いて既存テキストレイヤーを複製すれば動く — が、**新規プロジェクトからゼロで始めるペルソナには到達不能**。

## 1. 工程表(10段)

判定は実装(grep で識別子を示す)に対してのみ行う。「通る」= 到達できる実在の入口がある。「詰まる」= 入口が無い/繋がっていない/黙って空振りする。

| # | 工程 | 判定 | 根拠 |
|---|---|---|---|
| 1 | コンポジションを作る(解像度・fps・尺) | **通る** | `Composition{width,height,fps,duration_frames}` が Document 直下(`core/motolii-store/src/lib.rs:230` 台の `Composition` 構造体)。Settings パネルの COMPOSITION 節が W/H/FPS/Frames の4フィールドを編集し `Intent::SetComposition` で確定(`ui/motolii-settings-pane/src/sections.rs:100-134` `CompField::{Width,Height,Fps,DurationFrames}`)。値は Enter で確定、undo も効く(`edit` timeline 上の通常 entity、裁定40) |
| 2 | 図形を描く(矩形・円・ペンでパス) | **致命的に詰まる**(★最初の詰まり) | (a) Stage に描画ツール無し — `ui/motolii-stage-pane/src/gizmo.rs` は選択レイヤーの bbox+8ハンドル+回転ハンドルのみ実装(冒頭 doc「選択レイヤーの bbox+ハンドル8点…drag = move/scale/rotate」)、Pen/Rect/Ellipse ツール相当のモードは存在しない。(b) Browser create タブの Rectangle/Ellipse カードは `CreateKind::Rectangle`/`Ellipse` → `LayerSource::Shape` の layer は作るが(`ui/motolii-browser-pane/src/model.rs:341-350`)、`shell/motolii-shell/src/lib.rs:1548` `create_from_card` は `Intent::AddLayer`+`SetMeta`+`SetAttrs` の3つだけを `apply_all` し、**`Intent::SetShapes` を一度も呼ばない**(同関数のdoc自身が「図形の中身を書き分ける差はこの波の範囲外」と明記)。結果、生成される layer の `Layer:shapes` component は未設定 → `view.rs:617-629` `shapes()` は「無ければ空 `Vec`」を返す設計なので**中身が空のシェイプレイヤー**になる。(c) たとえ (b) を埋めても、`engine/motolii-engine/src/lib.rs:926` `texture_for` は `LayerSource::Null | LayerSource::Shape | LayerSource::Group => Ok((None, [0.0, 0.0]))` — **shape layer は演算子/組版から RGBA を焼く経路が未実装**という同関数の doc コメントどおり、常に「絵を持たない」枝に落ちる。3つの独立した欠落が直列に並んでおり、どれか1つの修理では図形は見えるようにならない |
| 3 | 図形を整える(塗り・線・角丸・パスの頂点編集) | **詰まる**(工程2の帰結+単独でも欠落) | Inspector の section は TRANSFORM(常設)/MASK/EFFECTS/TEXT(`LayerSource::Text`限定)/AUDIO(`LayerSource::Media`限定)の5種のみ(`ui/motolii-inspector-pane/src/projection.rs` の `SelectionProjection` 構成、`lib.rs:120-133` の `mod` 一覧)。**SHAPE section が存在しない** — fill/stroke/角丸/頂点編集のどれも UI 入口が無い。頂点編集用の Stage 上の drag(頂点ハンドル)も gizmo.rs に無い |
| 4 | 変形にキーを打つ(位置・スケール・回転・不透明度) | **通る** | `TransformField::{PositionX,PositionY,PositionZ,ScaleX,ScaleY,Rotation,Opacity,AnchorX,AnchorY}`(`ui/motolii-inspector-pane/src/transform.rs:44-54`)。diamond トグルで track 作成、drag-to-scrub、AE 作法の playhead upsert が実装済み(`key_cell_state`/`toggled_key_track`/`commit_inspector_field`)。Stage gizmo からの move/scale/rotate drag も同じ経路で1 commit=1 undo(`gizmo.rs` 冒頭 doc) |
| 5 | イージングを調整する(Easy Ease・補間切替・カーブ確認) | **半分通る** | `Interp::{Hold,Linear,Bezier{x1,y1,x2,y2}}`(`core/motolii-eval/src/track.rs:34-45`)。Edit メニューに `Interpolation: Hold/Linear/Easy Ease/Easy Ease In/Easy Ease Out` の5項目(`shell/motolii-shell/src/menu.rs:129-159`、定数は `ui/motolii-timeline-pane/src/write.rs:249-253`)。**カーブの確認手段(Graph Editor)は無い** — 数値入力での代替も無い(x1/y1/x2/y2 を直接打鍵する UI が存在せず、5個の固定プリセットのみ選べる)。選択キー全体への一括適用のみで個別カーブ調整不可 |
| 6 | テキストを作って動かす(文字・フォント・色・キーフレーム) | **入口が無く詰まる**(内容は実装済み) | **新規作成が不可能**: `CreateKind` は `Rectangle/Ellipse/Solid/Null` の4種のみで `Text` が無い(`ui/motolii-browser-pane/src/model.rs:341-350`)。Layer メニューにも `New Text Layer` は無い(`shell/motolii-shell/src/menu.rs:162-183` の Layer メニューは New Layer/Group/Ungroup/Freeze/Unfreeze のみ)。`Message::AddLayer`(旧 "+Layer" ボタン)は `LayerSource::Solid` 固定(`shell/motolii-shell/src/lib.rs:1409-1440`)。一方で中身は裁定190で実装済み: cosmic-text→swash→motolii-vector の字形描画ルート、`engine::texture_for` の `Text` 枝、Inspector TEXT section(`projection.rs:154-228`、`inspector-pane/src/text.rs`)。**既存プロジェクトを開いて既存テキストレイヤーを Duplicate(⌘D)すれば動く**が、ゼロから始める新規プロジェクトには到達不能 |
| 7 | マスクで抜く(マスクを描く・feather・mode) | **詰まる**(創建入口が無い+パスを描く手も無い) | MASK section は mode 巡回(Add/Subtract/Intersect/Lighten/Darken/Difference の6値、`ui/motolii-inspector-pane/src/mask.rs:32-41`)・inverted トグル・opacity 編集を実装済みだが、**「マスクを追加する」ボタン/メニューが存在しない**(`AddMask`/`add_mask`/`Mask::new` の呼び出しをグレップしても `mask.rs` 内のテスト以外ヒットなし)。工程2と同根: マスクのパス(`PathSource`)を描く Stage 上の手段が無い。feather の UI 有無は未確認(MASK section の4行に feather は現状見当たらない) |
| 8 | エフェクトを掛ける(Glow 等・パラメータにキー) | **詰まる**(創建入口が無い、パラメータ編集は実装済み) | `GlowParam::{Threshold,Intensity,Radius}`(`ui/motolii-inspector-pane/src/effects.rs:40-54`)は既存 effect のパラメータ編集・キーフレーム化に対応。Browser の effects タブに Glow カードは実在するが `creates: None`(`ui/motolii-browser-pane/src/model.rs:410-416`)— ダブルクリックしても何も起きない(`creates: Some` の場合だけ `on_double_click(Message::CreateFromCard)` が配線される、`browser-pane/src/lib.rs:1572`)。`AddEffect` 相当の Message は存在しない(grep 0件) |
| 9 | グループ化して親子で動かす | **通る** | ⌘G/⌘⇧G が `Message::GroupLayers`/`UngroupLayers` で実装済み(`shell/motolii-shell/src/menu.rs:171-176`)。`Document::group_layers` が選択レイヤーを新規 `LayerSource::Group` の子にし、`attrs.parent` 経由でtransform階層が合成される(裁定173)。任意レイヤーを任意レイヤーの子に直接指定する pick_list 型の parenting UI は無いが、グルーピングによる「まとめて動かす」目的は満たす |
| 10 | プレビューと書き出し(ループ・透過) | **半分通る** | 書き出しは **MP4/H.264 の1種のみ**(`ui/motolii-export-pane/src/lib.rs:132` `CONTAINER_CODEC_LABEL`)、品質(Normal/Lossless)と範囲(全体/作業範囲)のみ選択可。**alpha 付き書き出しは現経路で不可**(DECISIONS.md 裁定16「alpha 付き書き出しは現経路では出せない」、KNOWN.md「shell プレビューの alpha 合成(市松)は未実装」)。ループ書き出し(GIF/APNG等)や Lottie JSON 書き出しの経路自体が無い(`engine/motolii-export` に lottie 系識別子はゼロ) |

## 2. 「意味はあるのに触れない」一覧(この製品固有の非対称)

Lottie 語彙のうち、**store/engine 側の意味が実装済みなのに UI から到達できない**もの:

| 語彙 | 意味の実装場所 | UI から触れない理由 |
|---|---|---|
| Shape layer の中身 | `motolii-vector`(`ShapeNode`/`ops.rs`/`geom::rect`/`geom::ellipse`)、`LayerSource::Shape`(`core/motolii-store/src/lib.rs:208`) | 生成時に `SetShapes` を呼ばない(工程2参照)。仮に呼んでも engine が描かない |
| Text layer 作成 | `LayerSource::Text`/`TextDocument`/cosmic-text 実描画(裁定190)/Inspector TEXT section | `CreateKind` に `Text` が無い・Layer メニューに New Text Layer が無い(工程6参照) |
| Mask 追加 | `Mask`/`MaskMode`(6値)/mode 巡回・inverted・opacity 編集(`inspector-pane/src/mask.rs`) | 「マスクを足す」ボタン/メニューが存在しない(工程7参照) |
| Effect 追加(Glow) | `GlowParam`(Threshold/Intensity/Radius)・effect pass 合成(`EffectPass::padding()` 込みで実装済み、KNOWN.md) | Browser の Glow カードが `creates: None`。`AddEffect` 相当 Message が存在しない(工程8参照) |
| alpha 付き compositing | `blend_with_background: Premultiplied` 一行で readback に実 alpha が乗ることを実測済み(KNOWN.md) | 書き出し・プレビュー(市松)の出口側が未接続(裁定16) |

## 3. 最初の致命的な詰まり(★)

**工程2「図形を描く」。** 3つの独立した欠落(Stage に描画ツールが無い/`create_from_card` が `SetShapes` を呼ばない/engine が `LayerSource::Shape` を描画経路に繋いでいない)が直列に重なっており、現状は「Rectangle カードをダブルクリックしても画面に何も出ない」という**利用者からは"壊れている"としか見えない**状態。ペルソナの主素材(図形)が工程2で止まるため、直列ワークフローの以降(整える/キー/マスク/エフェクト/グループ)は理論上動く部品があっても実演できない。

## 4. この1本が通るための最小実装(順序つき)

小さい修理から積む。1〜3を通すだけで「矩形を置いて動かして書き出す」という最小のモーショングラフィックスが1本通る。

1. **engine: `texture_for` の `LayerSource::Shape` 枝を描画経路に繋ぐ**(`engine/motolii-engine/src/lib.rs:926`)。マスクで既に使っている `motolii_vector` のラスタライズ経路(`engine/motolii-engine/src/mask.rs` が使う `motolii_vector::render`/`Raster`)と同じ仕組みを shape layer の fill/stroke へ転用する。新規依存ゼロ(同じ crate の別入口を使うだけ)
2. **shell: `create_from_card` に `Intent::SetShapes` を追加**(`shell/motolii-shell/src/lib.rs:1548`)。`engine/motolii-vector/src/geom.rs:137/149` に既にある `rect()`/`ellipse()` ビルダー(現状 `pub(crate)`)を `pub` へ昇格し、デフォルトの fill(単色)を添えて Rectangle/Ellipse カードごとに実体を書く
3. **inspector: SHAPE section を新設**(MASK/TEXT/EFFECTS と同じ「型別 section」の型を踏襲、`projection.rs` の `SelectionProjection` に `shape: Option<ShapeSectionProjection>` を足す形)。最小は fill 色+角丸(数値)の2項目でよい — Easy Ease が「プリセットのみ・数値入力なし」で最初に出荷できた前例と同じ縮小合意を踏襲できる
4. **browser+menubar: Text layer 作成の入口を追加**。`CreateKind::Text` を足し、`create_from_card` で `Intent::SetTextDocument` にデフォルト値(空文字+既定スタイル)を書く。中身(TEXT section・描画)は既に実装済みなので、この1本だけで工程6が丸ごと開通する
5. **inspector: MASK section に「マスクを追加」ボタン**。`geom::rect()` を再利用したデフォルト矩形マスクを `Intent::SetMasks` で push する最小実装(Pen tool 相当は後回しでよい — 数値/デフォルト形状で最初の1本を通す方針は3と同じ)
6. **browser: Glow カードの `creates` を実体化**。`Intent::SetEffects` でデフォルトパラメータの Glow を push する `AddEffect` 相当の Message を1本追加するだけで工程8が開通する(パラメータ UI は既存)
7. (優先度落ち・工程10向け)alpha 付き書き出し。DECISIONS 裁定16 が特定済みの fork seam を埋める — 他の6項目より工数が大きく、ペルソナの「最初の1本」には必須ではない

1〜4で工程2・3(最小)・6が同時に開通し、5〜6で工程7・8が開通する。7は別軸の投資として後回しでよい。

## 5. 逸脱

- 「カーブの確認手段が無いことが致命傷か」への回答: **工程2の致命傷の方が先に効くため、現状は致命傷かどうかを実演で確かめる段階にすら到達しない**。ただし工程2〜4が直ったと仮定した場合、5個の固定プリセット(Hold/Linear/Easy Ease/In/Out)は「イージングを一切調整できない」よりは遥かに強く、Rive/Lottie 系の多くの入門編集がプリセットベースであることを踏まえると**致命傷ではなく劣化(数値入力すら無いのは今後の課題)**と判定した。これは実装を読んだ上での判断であり、実機確認はしていない(逸脱)
- feather(mask の羽根づけ)の UI 有無は MASK section の4行(mode/inverted/opacity/…)を読んだ限りでは確認できなかったが、mask 追加自体が無いため深追いしていない(逸脱・時間配分の判断)
- 任意レイヤーへの自由な parent 指定(Group を介さない pick_list 型)UI の有無は完全には調べ切れていない(`set_parent` はテストヘルパーのみ確認、view 層の pick_list を全 pane 走査してはいない)。Group/Ungroup で工程9の主目的は満たされるため優先度を下げた(逸脱)
