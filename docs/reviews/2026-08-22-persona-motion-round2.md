# ペルソナ「モーショングラフィックス制作者」第2周 — 壁の向こうを掘る

日付: 2026-08-22 / 状態: **調査**(read-only・書き込みは本ファイルのみ、コード変更ゼロ) / 起点: [`2026-08-22-persona-motion.md`](2026-08-22-persona-motion.md)(第1周)の続き — 「シェイプが画に出ない」の先を最後まで辿る発注

基点: `git merge main` 済み(fast-forward、`fcd72f30` まで)。以降の grep は全てこの時点の `next/` に対して行った。

## 0. 前提の検査(先出し)

発注書は「SHAPE-RENDER レーンと裁定205が着地した前提で」と指示していたが、**実際に `main` へ着地しているのは裁定205(方針決定)だけ**で、その実行(engine の描画結線・`create_from_card` の `SetShapes` 配線・Stage の描画ツール)は**まだ `main` に無い**。grep で確認:

- `engine/motolii-engine/src/lib.rs:926` は今も `LayerSource::Null | LayerSource::Shape | LayerSource::Group => Ok((None, [0.0, 0.0]))` のまま(第1周と1文字も変わっていない、doc コメント「shape はまだ描画に繋いでいない」も残存)
- `shell/motolii-shell/src/lib.rs:1548` `create_from_card` も `Intent::AddLayer`+`SetMeta`+`SetAttrs` の3つだけで `Intent::SetShapes` は無い(第1周と同一)
- Stage の描画ツール(`shape_tool.rs`、コミット `eabe1c59` 「シェイプ作成ツール(B28)」)は**別 worktree ブランチ(`worktree-agent-a7f76304b1786a2fe`)にのみ存在し、`main` に未マージ**。`git log --all --grep` で "SHAPE-RENDER" 系の着地コミットも見つからない
- 裁定205(`next/DECISIONS.md` 193行目)は着地済み — 「追加する」意図の家を Browser に統一する方針そのものは確定している

**この文書は発注どおり「壁1が塞がった前提」で以降を掘るが、実際にはまだ壁1自体が現在進行形で開いている**ことをまず報告する(逸脱として詳細は§5)。以下の壁2以降は、**壁1の有無に関係なく既に grep 0件が出る**(UI/shell 側にそもそも呼び手が無い)ため、壁1の着地タイミングとは独立に成立する調査結果である。

## 1. 壁の順序リスト(壁1から続く直列)

判定は実装(grep で識別子を示す)に対してのみ行う。

| # | 壁 | 判定 | 根拠 |
|---|---|---|---|
| 1 | (第1周から継続)図形が画に出ない | **未解消** | §0参照。`texture_for` 未結線・`create_from_card` に `SetShapes` 無し・Stage 描画ツール未マージ |
| 2 | 図形を作り込む(頂点編集・角丸・trim-path・repeater 等 modifier) | **詰まる**(意味は完備) | `motolii-vector` に `OpKind::{TrimPath,Repeater,RoundedCorners,PuckerBloat,ZigZag,OffsetPath,Twist}` の7演算子が全実装済み(`engine/motolii-vector/src/lib.rs:193-265`、実処理は `ops.rs`)。編集口も `stack_edit.rs`(`insert_op`/`remove_op`/`move_op`/`set_kind`/`set_hidden`)と `edit.rs`(`insert_vertex`/`remove_vertex`/`move_vertex`/`set_handles`/`close_path`/`open_path`/`split_segment`)に完備。**しかし `ui/` `shell/` のどちらにもこれら識別子の呼び手が1つも無い**(`grep -rl "ShapeOp\|OpKind\|TrimPath\|Repeater\|RoundedCorners\|PuckerBloat\|ZigZag\|OffsetPath\|insert_vertex\|insert_op" next/ui next/shell` = 0件)。原因は Inspector の `SelectionProjection`(`ui/motolii-inspector-pane/src/projection.rs:138-158`)に `shape` フィールドが無いこと1つに収束する — MASK/EFFECTS/TEXT と同型の SHAPE section が丸ごと未着手 |
| 3 | 塗りと線(色・グラデーション・線幅・破線) | **詰まる**(壁2と同根) | `Fill`/`Stroke`/`Brush::{Solid,Gradient}`/`Gradient`/`Dash`/`LineCap`/`LineJoin` が全実装済み(`lib.rs:316-463`)だが SHAPE section が無いので触れない(壁2と同一原因)。加えて**色ピッカー系ウィジェットが `next/ui` 全体に1つも無い**(`grep -rn "color_picker" next/ui next/shell` = 0件・`iced` 依存も core/wgpu/advanced/image/canvas の4 feature のみで color_picker 系は含まれない)— SHAPE section を最小構成で作っても「色をつまむ」部品自体が新規実装になる(iced_aw 等サードパーティ crate の有無は未確認、逸脱) |
| 4 | パス自体をキーフレームで動かす(シェイプモーフ) | **詰まる**(track として未接続) | `motolii_eval::Value::Path`(`core/motolii-eval/src/value.rs:44` 台)は「頂点数が同じ時だけ頂点ごとに線形補間」という補間規則を既に実装済み — だが**この規則を使っているのは mask の形状トラック(`mask.{id}.shape`)だけ**(`grep -rl "Value::Path" next` で mask.rs/view.rs のみヒット)。shape 自身の `PathSource` は `Intent::SetShapes` という「丸ごと静的差し替え」のみで、`TransformField` 相当の「キー可能フィールド」としての配線(diamond トグル・`commit_inspector_field`)が無い。仮に壁1〜3が全部解決しても、**パスの形をタイムラインでアニメーションさせる口が別途要る** |
| 5 | イージングを調整する(カーブ視認・数値入力) | **半分通る**(第1周と不変) | Hold/Linear/Bezier の5固定プリセットのみ、Graph Editor 無し、x1/y1/x2/y2 の数値入力 UI 無し。`ui/motolii-timeline-pane/src/write.rs:285-289` の `EASY_EASE`/`EASY_EASE_IN`/`EASY_EASE_OUT` 定数は不変(merge で触られていない) |
| 6 | 親子とグループ | **通る**(第1周と不変) | ⌘G/⌘⇧G・`Document::group_layers` 健在 |
| 7 | マスクで抜く | **詰まる**(第1周より詳細判明) | (a) 「マスクを追加する」ボタン/メニューが今も無い(`grep -rn "AddMask\|add_mask" next/ui next/shell` = 0件、第1周と不変)。(b) **新知見**: 仮にボタンだけ足して `Intent::SetMasks` で `Mask{id,mode,inverted}` を push しても、同じ apply_all で `mask.{id}.shape` の property track(`Value::Path`)を書かないと `StoreView::resolved_masks` が `Err("マスク {id} に形状が無い")` を返す(`core/motolii-store/src/view.rs:671-676`)— **「形の無いマスク」は仕様上のエラー状態であって空の絵ではない**。最小実装は Mask push と `Intent::SetTrack`(既定矩形パス)を同一操作にする必要がある |
| 8 | マスクの膨張(Expand) | **詰まる**(store は書けるが engine が読まない・二重の壁) | `PropertyId::mask_expansion`(`core/motolii-store/src/document.rs:89`)は2026-08-22 に追加済みで `Intent::SetTrack` で書ける・保存できる・undo も効く。しかし `ResolvedMask` 構造体自身がまだ `expansion` フィールドを持たず、`StoreView::resolved_masks` も読んでいない(`core/motolii-store/src/mask.rs` 冒頭の自己申告「未完(次のレーンへ)」節)— **store↔engine 間の壁**。さらに UI 側にも `TransformField::MaskExpansion` 相当のフィールドが無い(`TransformField` は `MaskOpacity` はあるが `MaskExpansion` は無い、`ui/motolii-inspector-pane/src/transform.rs:47-70`)— **UI↔store 間の壁も別に存在**。2段の壁が重なっている |
| 9 | アルファマット(track matte) | **詰まる**(2026-08-22 に新規発生した壁 — 第1周は未検出) | `MatteMode`(Alpha/InvertedAlpha/Luma/InvertedLuma)と `Matte{layer,mode}`(`core/motolii-store/src/attrs.rs:52-66`)は**この merge で初めて engine が実消費するようになった**(`engine/motolii-engine/src/lib.rs:1133` `translate_matte_mode`、コミット `679540c5`/`769a34a3`「matte を render_frame へ結線」「ゼロコピー経路にも matte を結線」、いずれも2026-08-22)。つまり**意味の実装(合成)は今日完成した**。だが `LayerAttrs.matte` へ書き込む UI・「このレイヤーを下のレイヤーのマットにする」ボタン・matte-mode 選択 UI は `ui/` `shell/` のどこにも無い(`grep -rn "matte" next/ui next/shell` がヒットするのは `split.rs`/`clipboard.rs` の「複製時に既存の matte 設定を保持する」処理のみ — **新規に設定する**経路ではない)。壁7/8と同じ形(意味は着地済み・入口が無い)がこの2日で3件目、増加傾向 |
| 10 | エフェクトを掛ける | **詰まる**(第1周と不変、種類も1つのみ) | Browser の effects タブは Glow カードを含め全カード `creates: None`(第1周と不変)。`AddEffect` 相当の Message は grep 0件。加えて**実装済みの effect 種は `EffectPass::Glow` の1種のみ**(`engine/motolii-compositor/src/effects/mod.rs:24-38`)— 「複数掛け」を試す土台(`Vec<EffectInstance>`、`core/motolii-store/src/effect.rs:31`)は複数インスタンス対応で作られているが、種類が1つしか無いので「違う effect を2つ重ねる」という組み合わせ自体が現状再現不能 |
| 11 | 繰り返しと複製(同じ動きを何個も・時間差) | **半分通る**(手作業のみ) | レイヤーの Duplicate(⌘D、`shell/motolii-shell/src/clipboard.rs`)自体は健在 — 複製して個別に in/out point や position キーを手でずらせば「時間差の反復」は原理上作れる(AE の素の Duplicate と同型、Sequence Layers 相当の自動アシスタントは無い)。**shape 単位の Repeater(`OpKind::Repeater`、コピーごとに変換・時間差ではなく空間差だが「同じ形が並ぶ」用途の主要語彙)は壁2の中にいて触れない** |
| 12 | プレビューの往復 | **通る**(第1周で未評価・今回確認) | Space で `Message::TogglePlayback`(`shell/motolii-shell/src/lib.rs:5671`)、`PlaybackTick` によるスクラブ/自動再生が実装済み |
| 13 | 書き出し(透過・ループ) | **詰まる**(第1周と不変) | MP4/H.264 の1種のみ(`ui/motolii-export-pane/src/lib.rs:132`)。alpha 付き書き出し・GIF/APNG ループ書き出し・Lottie JSON 書き出しのいずれも経路ゼロ(`grep -rln "lottie\|gif\|apng" next/engine/motolii-export` = 0件) |

## 2.「意味はあるのに触れない」完全一覧(Lottie 地図の行つき)

`next/reference/lottie-coverage.tsv` の行番号(1始まり、ヘッダ含む)を付す。**採用済み = store/engine に実在する識別子がある行**。

| # | Lottie 語彙(group/object/field) | 地図行 | 実装識別子 | 触れない理由(壁) |
|---|---|---|---|---|
| 1 | shapes/rectangle s/ty | 432-433 | `PathSource::Rectangle` | 壁1(`SetShapes` 未配線・未描画) |
| 2 | shapes/ellipse s/ty | 376-377 | `PathSource::Ellipse` | 壁1 |
| 3 | shapes/path ks/ty(フリーハンドベジェ) | 414-415 | `PathSource::Bezier` | Stage にペン系ツール入口が無い(`shape_tool.rs` は別ブランチ未マージ) |
| 4 | shapes/polystar pt/or/ir/sy/ty(星・多角形) | 417,419,422,424,425 | `PathSource::PolyStar` | **`CreateKind` に Star/Polygon が無い**(第1周未検出。`Rectangle/Ellipse/Solid/Null` の4種のみ、`ui/motolii-browser-pane/src/model.rs:341-350`) |
| 5 | shapes/trim-path s/e/o/m/ty | 461-465 | `OpKind::TrimPath` | 壁2(SHAPE section 不在)。地図注記「MV で最も使う語彙」 |
| 6 | shapes/repeater c/o/tr/m/ty | 435-439 | `OpKind::Repeater` | 壁2。地図注記「trim-path と並ぶ MV の主語彙」 |
| 7 | shapes/rounded-corners r/ty | 444-445 | `OpKind::RoundedCorners` | 壁2 |
| 8 | shapes/pucker-bloat a/ty | 427-428 | `OpKind::PuckerBloat` | 壁2 |
| 9 | shapes/zig-zag s/r/pt/ty | 472-475 | `OpKind::ZigZag` | 壁2 |
| 10 | shapes/offset-path a/lj/ml/ty | 409-412 | `OpKind::OffsetPath` | 壁2 |
| 11 | shapes/twist a/c/ty | 467-469 | `OpKind::Twist` | 壁2 |
| 12 | values/bezier(頂点単位の追加/削除/移動/ハンドル) | 414 と同じ行(`path.ks` の中身) | `edit::{insert_vertex,remove_vertex,move_vertex,set_handles,split_segment,close_path,open_path}` | 壁2。Stage 上の頂点ドラッグ UI も gizmo.rs に無い |
| 13 | shapes/fill c/r/ty | 379-381 | `Fill`/`Brush::Solid`/`FillRule` | 壁2・壁3(色ピッカー部品も無い) |
| 14 | shapes/stroke ty + base-stroke d/lj/w/lc/ml2 | 452-453, 368,370 | `Stroke`/`Dash`/`LineCap`/`LineJoin` | 壁2・壁3 |
| 15 | shapes/base-gradient s/e/g/t + gradient-fill/-stroke ty | 363,364,366,367,384-385,389 | `Brush::Gradient`/`Gradient`/`GradientStop` | 壁2・壁3 |
| 16 | helpers/mask mode/inv/pt | 194,193,196 | `Mask`/`MaskMode`/`mask_shape` | 壁7(追加入口なし) |
| 17 | helpers/mask o(Opacity) | 195 | `PropertyId::mask_opacity`・`TransformField::MaskOpacity` | 配線自体は完成しているが、壁7で mask が1枚も作れないため到達不能 |
| 18 | helpers/mask x(Expand) | 197(旧「不採用」注記だが2026-08-22裁定で採用済みへ回収) | `PropertyId::mask_expansion` | 壁8(store は書けるが `ResolvedMask` 未消費+UI フィールド無しの二重壁) |
| 19 | constants/matte-mode + layers/visual-layer 相当の matte 参照 | 71 | `MatteMode`/`Matte{layer,mode}`/`translate_matte_mode` | 壁9(2026-08-22 に engine 消費が着地した直後の新規ギャップ) |
| 20 | layers/text-layer(テキストレイヤー新規作成) | (第1周 `docs/reviews/2026-08-22-persona-motion.md` §2 既出) | `LayerSource::Text`/`TextDocument` | `CreateKind::Text` 未追加。裁定205 で「create タブに Text を足す」方針は確定済みだが未実行 |
| 21 | effects/all-effects(Glow 適用) | (Glow は Document 非型化のため地図に行を持たない、裁定70) | `EffectPass::Glow`/`GlowParam` | Browser effects タブ `creates: None`(第1周既出)。加えて実装 effect 種は1つのみ |

## 3. この1本が通るための最小実装(順序つき、第1周の1〜7に続く形)

第1周の1〜7(engine の Shape 描画結線・`create_from_card` の `SetShapes`・SHAPE section 新設・Text 作成・Mask 追加ボタン・Glow `creates` 実体化・alpha 書き出し)を前提に、**その先で「1本のモーショングラフィックス」を最後まで通すため**の続き:

1. **SHAPE section を fill 色 + RoundedCorners 数値の2項目で出す**(第1周案そのまま)。ただし色ピッカー部品が皆無なので、最小は「16進テキスト入力+プレビュー矩形」程度に縮小して部品コストを避けるのが現実的(逸脱として§5に記載)
2. **Mask 追加ボタンは「Mask push」と「`mask.{id}.shape` に既定矩形パスを書く `SetTrack`」を同一 `apply_all`(1 undo)にする**(§1 壁7の新知見 — 分離すると `resolved_masks` が壊れた Document エラーを返す)
3. **CreateKind::Text**(裁定205 で方針確定済み、実行のみ残っている)
4. **Browser effects タブの Glow カードを `creates: Some` へ**(`AddEffect` 相当 Message 1本)
5. **TrimPath / Repeater を SHAPE section に1項目ずつ追加**(`stack_edit::insert_op`/`set_kind` を書き口にする)。地図が「MV で最も使う」「MV の主語彙」と明記する2つを最優先に — ここまでで「回る線」「並ぶ複製」という MV の定番表現が開通する
6. **Matte の最小 UI**: 対象レイヤーを選ぶ pick_list(または Timeline での「1つ下を matte 元にする」トグル)+ mode 4値巡回ボタン(`next_mask_mode` と同型の巡回文法を流用 — 新しい編集文法を発明しない)。**意味側(engine)は既に完成している**ので、この1本の書き口だけで壁9が開通する
7. **頂点編集・グラデーション・破線・残り4 modifier(PuckerBloat/ZigZag/OffsetPath/Twist)**: SHAPE section に operator 一覧(追加/削除/並べ替え = `stack_edit::insert_op/remove_op/move_op`)を足せば、意味側は全部揃っているので UI コスト以外の障壁が無い
8. **mask_expansion の `ResolvedMask` 消費**(`view.rs`/compositor 側の幾何演算)+ `TransformField::MaskExpansion` の追加。優先度は中 — Lottie/AE 実機でも「マスクを触った」体験の主要部分ではない(feather と違い Expand は地図に実在するが、mask 自体が使えるようになった後でよい)
9. (優先度低・別軸、第1周から変わらず)alpha 付き書き出し・ループ書き出し・Lottie JSON 書き出し

1〜6で「矩形を描いて塗って動かしてマスクで抜いてグローを掛けて書き出す」という**AE の入門チュートリアル1本分**が通る。7〜9は表現の幅を広げる投資で、後回しでよい。

## 4. 逸脱

- **前提不成立**(§0): 発注は「SHAPE-RENDER レーンと裁定205が着地した前提」だったが、実際に `main` へ着地しているのは裁定205(方針決定)のみで、実行(engine 結線・`create_from_card` 配線・Stage 描画ツール)は別 worktree ブランチ(`worktree-agent-a7f76304b1786a2fe` のコミット `eabe1c59` 等)にあり `main` 未マージ。壁2以降の判定(UI/shell 側の呼び手ゼロ)は壁1の状態に関わらず成立するため調査の結論には影響しないが、「壁1が最短で足りない物」という第1周の結論は**今この瞬間も継続して真**である
- 色ピッカー部品の完全な不在(`iced` 標準/サードパーティ問わず)は `grep -rn "color_picker"` の0件と `Cargo.toml` の feature 一覧だけで判定した — iced_aw 等の外部 crate を意図的に避けている設計判断なのか単に手が回っていないだけなのかは DECISIONS.md を全文検索した限り明記が見当たらず、未確認(判断不能)
- 254本ある agent/worktree ブランチを総当たりで確認してはいない(時間配分の判断) — §0 の「壁1未着地」は `main` の実ファイル grep と、コミットメッセージによる横断検索(`git log --all --grep`)で確認した範囲であり、どこかのブランチで既に壁1が解決済みで統合待ちの可能性は排除できない
- マスクの feather(羽根づけ)は Lottie `helpers/mask` に対応キーが無いため意味の正本が無く、「意味はあるのに触れない」一覧には含めていない(`mask.rs` 冒頭 doc が明記済みの既存判断を踏襲)
- 壁9(track matte)の UI 最小実装案(§3 手順6)は「対象レイヤーを選ぶ pick_list」と書いたが、`next/` に pick_list 型 UI の前例が無いこと(第1周 §5 で「pick_list は next/ に前例が無い」との言及が `mask.rs` 内にあった)を踏まえると、既存の「巡回ボタン」文法だけでは「どのレイヤーをマット元にするか」を選べない — 参照先レイヤー選択の UI 文法自体が未確定という、より小さいがもう1つの壁が中に隠れている可能性がある(深追いしていない、逸脱)
