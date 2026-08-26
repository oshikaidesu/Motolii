# Motolii Timeline panel — Icebook design drafts

Icebook で1案を1 story として並べられるよう、各案に固定 ID と同じ観測項目を持たせた。これは実装順ではなく、Timeline の見せ方・密度・制作上の役割を比較するための草案である。

共通前提:

- Timeline の意味は `next/reference/timeline-grammar.md` に従う。レーンバー、ルーラ、クリップ面、ナビゲータの仕事を混ぜない。
- `M5` は move / trim / snap / release / Esc 復元、`M8` は再生・scrub・音・playhead の同期、`M19` は property 単位の keyframe 操作を観測対象にする。
- 既存の `transport.rs`、`ruler.rs`、`work_area.rs`、`rail.rs`、`canvas.rs`、`key_rows.rs`、`graph_editor.rs`、`markers.rs`、`waveform_view.rs` を再利用する前提で、見た目の差だけを草案化する。

## T01 — 一本の背骨 / Single Spine

- **ID**: `T01`
- **Name**: 一本の背骨 / Single Spine
- **Problem solved**: 再生・現在時刻・編集面の入口が散らばり、Timeline を開いても次に何をすべきか分からない問題を解く。
- **Hero / creation role**: 作品を置いた直後に再生し、Stage の結果を見て次の編集へ入るための基準パネル。hero の良し悪しを最短で判断する。
- **Layout / visual hierarchy**: 上端全幅に先頭・step・play/pause・step・末尾・loop・timecode。直下に loop 帯、locator 帯、目盛、playhead。中央は左レーンバー＋右クリップ面、下端はナビゲータ。transport はクリップ面より一段明るく、playhead と timecode だけ accent にする。
- **Interaction / entry**: Space は既存の再生、transport は同じ意味 action、timecode は確定時だけ playhead を移動、ルーラ押下維持で scrub、`M` で locator、ドラッグ中の `Esc` は復元。
- **Density / scale**: 標準密度。8〜12レーン、0.25〜30秒の制作範囲を一画面に置き、狭くなったら行領域だけ縦スクロールする。
- **Reuse vs scratch note**: `transport_spec`、`ruler`、`work_area`、`rail`、`canvas` の既存責務をそのまま再利用。スクラッチは story 用の配置と token 差分だけにし、新しい再生状態や時間モデルは作らない。

## T02 — 時刻ビーコン / Timecode Beacon

- **ID**: `T02`
- **Name**: 時刻ビーコン / Timecode Beacon
- **Problem solved**: フレーム単位でどこにいるか、次のキーやマーカーまで何フレームかを読めず、編集の確定に自信が持てない問題を解く。
- **Hero / creation role**: 文字・ロゴ・点群の出現フレームを正確に決めるためのパネル。hero の「一瞬」を作る。
- **Layout / visual hierarchy**: 上端中央に大きい frame/timecode、左右に前後の意味点を小さく表示。transport は左、loop は右。ルーラには現在フレームから次の key / locator / loop 端までの短い縦 tick を置き、下のトラックはやや暗くする。
- **Interaction / entry**: timecode をクリックして frame input、Enter で確定、Esc で下書きを破棄。前後キー移動は `JumpToPrevious/NextMeaningPoint`、矢印は playhead だけを1フレーム動かす。J/K/L は候補入口として表示せず、既存 keymap の候補に留める。
- **Density / scale**: 1〜5秒の近接編集向け。timecode の高さを優先し、行は18〜22px、1画面のレーン数は6〜10。
- **Reuse vs scratch note**: `transport` の frame draft、`nav::nearest_meaning_point`、`ruler` の tick 投影を再利用。大きな数字と次点表示だけを新規の見た目として作り、別の playhead owner は持たない。

## T03 — スクラブ・リボン / Scrub Ribbon

- **ID**: `T03`
- **Name**: スクラブ・リボン / Scrub Ribbon
- **Problem solved**: ルーラを掴んでも動いている感覚が弱く、狙った瞬間を探すだけで制作が止まる問題を解く。
- **Hero / creation role**: 音の落ち・カットの頭・文字の出現点を手触りで探すための creation 面。再生より「触って見つける」ことを主役にする。
- **Layout / visual hierarchy**: パネル高の上半分を大きなルーラ／scrub 面にし、playhead を太い縦線と現在時刻ラベルで表示。transport は左上に小さく固定し、トラックは下半分に集約。クリップ面はスクラブ面と同じ時刻軸を共有する。
- **Interaction / entry**: ルーラ押下維持で scrub、掴んだ瞬間に再生停止、フレーム丸め。Cmd+wheel はカーソル下時刻を保持した zoom、Shift+wheel は横パン。ダブルクリックは使わない。
- **Density / scale**: 0.5〜8秒を大きく見せる近接モード。トラックは4〜8行、スクラブ帯は通常の2倍高。
- **Reuse vs scratch note**: `ruler`、`frame_at_x`、`viewport_canvas`、既存の `input` の drag-to-scrub を再利用。太い playhead、現在時刻のミニラベル、面の余白だけがスクラッチで、scrub の意味は発明しない。

## T04 — ビート・パルス Transport / Beat Pulse Transport

- **ID**: `T04`
- **Name**: ビート・パルス Transport / Beat Pulse Transport
- **Problem solved**: 音に合わせたいのに、再生ボタンと波形だけでは拍の基準が見えず、hero の勢いを判断できない問題を解く。
- **Hero / creation role**: 音楽同期のある motivational video で、落ち・アクセント・反復を視覚的に揃えるためのパネル。
- **Layout / visual hierarchy**: transport の下に細い waveform／peak ribbon、その下に locator と拍の縦線。通常トラックはさらに下。再生中だけ現在 beat の pulse が薄く光り、playhead は常に最上層に出る。
- **Interaction / entry**: Space 再生、`M` で現在 playhead に locator、locator クリックでジャンプ。beat metadata がある場合は snap 候補にするが、無い場合は locator と clip/key の既存 snap へ戻す。
- **Density / scale**: 2〜16秒の音楽フレーズ向け。波形 ribbon は12〜20px、beat tick は細く、レーンは標準18〜24px。
- **Reuse vs scratch note**: `waveform_view`、`markers`、`transport`、`ruler` を再利用。新しい beat 検出器や PCM 生成は作らず、上流から得た peak／beat があれば投影し、無ければ既存 locator を使う。

## T05 — トラック台帳 / Track Ledger

- **ID**: `T05`
- **Name**: トラック台帳 / Track Ledger
- **Problem solved**: クリップが増えると、どのレイヤーが何を担当しているか、lock・mute・solo・親子関係が読めなくなる問題を解く。
- **Hero / creation role**: hero を構成する素材の役割を整理し、背景・主役・文字・音を同時に見渡すための arrangement 面。
- **Layout / visual hierarchy**: 左36%をレーンバーに確保し、swatch・名前・depth・M/S/L・fold を恒久列にする。右64%は名前を描かないクリップ面。選択行だけ accent、非選択行は zebra を静かにする。
- **Interaction / entry**: 行クリック、Cmd トグル、Shift 範囲、fold、行間への並べ替え、Enter の inline rename。lock 中の書き込みは理由つき拒否。クリップ上の名前は入口にしない。
- **Density / scale**: 24px 行高を基準に20〜40レーン。レーンバー幅は狭めず、溢れた時間方向だけを横 scroll する。
- **Reuse vs scratch note**: `rail`、`projection::tree`、`rows`、`lane_bar` の現在の行責務を再利用。台帳の見出し装飾と depth guide だけを作り、独自の layer tree や選択状態を持たない。

## T06 — クリップ・リズム盤 / Clip Rhythm Board

- **ID**: `T06`
- **Name**: クリップ・リズム盤 / Clip Rhythm Board
- **Problem solved**: bar の長さ・空白・重なりが弱く見え、時間のリズムを素材名の探索でしか判断できない問題を解く。
- **Hero / creation role**: 「素材を並べた」から「時間にリズムを置いた」へ視線を移し、hero のカットテンポを作るための面。
- **Layout / visual hierarchy**: クリップ面を広く取り、時間方向の明暗帯と行方向の zebra を薄く敷く。bar は角丸と長さで主役にし、名前は左レーンバーにだけ置く。playhead と選択 bar の境界は他より明るくする。
- **Interaction / entry**: bar 本体は move、端8pxは trim、Alt+drag は複製、Cmd は一時 snap 無効、release で1回確定、Esc で元へ戻る。ドラッグ中は preview のみを動かす。
- **Density / scale**: 0.25〜10秒の短尺 hero 向け。bar の最小幅が24px未満になる zoom では端 trim を出さず、本体移動だけにする。
- **Reuse vs scratch note**: `canvas`、`hit`、`clip_gesture`、`projection::geometry` を再利用。明暗 rhythm と選択の ink だけを token で調整し、ripple／roll 等の独自編集は作らない。

## T07 — フォーカス・レーン / Focus Lane

- **ID**: `T07`
- **Name**: フォーカス・レーン / Focus Lane
- **Problem solved**: keyframe が全レイヤーの下に埋もれ、どの property が動いているか見えない問題を解く。
- **Hero / creation role**: 選択した主役レイヤーの position・opacity・scale などを時間で育てる、M19 の中心面。
- **Layout / visual hierarchy**: 選択 layer の直下に key を持つ property 行だけを展開し、他の layer は通常行のままにする。property 名はレーンバー側、菱形と時間はクリップ面側。選択 row の左端に細い focus rail を置く。
- **Interaction / entry**: layer を選択すると自動で展開。◇で params を開閉、key クリックは単独、Cmd はトグル、Shift は範囲、Delete は key を優先。キー端の Cmd+drag は選択群の比例 retime。
- **Density / scale**: 1 layer・4〜12 property 行・1〜30秒。property が多い場合はその帯だけ縦 scroll し、全レイヤーを同時展開しない。
- **Reuse vs scratch note**: `property_rows`、`key_rows`、`key_gesture`、`projection::properties` の既存設計をそのまま再利用。focus rail と story の property 名配色だけをスクラッチにする。

## T08 — グループ・ツリー / Group Tree Stage

- **ID**: `T08`
- **Name**: グループ・ツリー / Group Tree Stage
- **Problem solved**: 親子・precomp 相当のまとまりを開閉すると、自分がどの階層を編集しているか見失う問題を解く。
- **Hero / creation role**: hero の背景・主役・補助要素をグループ単位で整え、複雑な構成でも意図を保つための arrangement 面。
- **Layout / visual hierarchy**: 左のレーンバーを tree として強調し、depth ごとに薄い guide。右の時間面は全階層で同じ x 軸。上部に「root / group / selected layer」の breadcrumb を置くが、時刻面は増やさない。
- **Interaction / entry**: ▸▾ で fold、子孫への drop は禁止カーソル、行間へ drag して並べ替え。Group 選択は親が揃わない場合に理由つき拒否。property 行は focus layer の直下だけ。
- **Density / scale**: depth 0〜4、15〜30行。レーンバーの横幅を固定し、深い階層でもクリップ面の可視時間を削らない。
- **Reuse vs scratch note**: `projection::tree::rows`、`rail`、`layer_row_at_y` を再利用。breadcrumb と depth guide は表示専用で、別の木・parent state・並び順ルールは作らない。

## T09 — Timing Truth / 時間の真実

- **ID**: `T09`
- **Name**: Timing Truth / 時間の真実
- **Problem solved**: 0秒・composition終端・source終端・clip start/end の関係が曖昧で、trim 後に何が残るか判断できない問題を解く。
- **Hero / creation role**: hero の登場・退場を安全に決めるための、M4/M5 を視覚化した timing 面。
- **Layout / visual hierarchy**: ルーラの0とcomp終端を太い pin にし、clip の start/end に細い vertical guides。選択 bar の両端だけ frame label を表示し、source outside は背景色のまま描かない。
- **Interaction / entry**: 端8pxの trim と本体 move を明確にカーソルで予告。snap 対象は0・終端・playhead・loop端・他 clip/key。trim では key 時刻を動かさず、move では key を追従させる。
- **Density / scale**: 0〜comp全体の境界が常に読める fit 寄り。最大でも12行、境界 pin と time label に余白を与える。
- **Reuse vs scratch note**: `frame_to_x`、`bar_span_x`、`classify_bar_part`、`LayerTiming` の既存意味を再利用。境界 guide とラベルだけを追加し、trim family や別の尺計算は作らない。

## T10 — Work Area Loop Deck / 作業範囲デッキ

- **ID**: `T10`
- **Name**: Work Area Loop Deck / 作業範囲デッキ
- **Problem solved**: ループ区間が一時的な選択に見え、反復再生や範囲書き出しの対象を失う問題を解く。
- **Hero / creation role**: 4小節のモーション、音の落ち、短い title animation を繰り返し磨くための creation deck。
- **Layout / visual hierarchy**: 最上段を厚い loop band にし、locator band をその下、目盛と playhead をさらに下に置く。transport の loop button は帯の状態を反映するだけで、帯が唯一の区間面になる。
- **Interaction / entry**: 空白から左右どちらへ引いても loop を作る。端8pxは resize、中は平行移動、L は on/off。再生中の M tap は止めずに locator を置く。drag 中は端を追い越して区間を畳まない。
- **Density / scale**: 1フレーム〜20秒。loop band 24〜32px、クリップ行は18〜22px。短い範囲を作る時だけ帯を自動的に目立たせる。
- **Reuse vs scratch note**: `work_area`、`ruler`、`transport`、`input` の既存 hit/preview を再利用。帯の色と loop endpoint label だけを作り、Document 外の別 loop state は導入しない。

## T11 — Trim Surgery / トリム手術台

- **ID**: `T11`
- **Name**: Trim Surgery / トリム手術台
- **Problem solved**: bar の端を掴んだのか本体を動かしたのかが分からず、意図と違う編集を確定する問題を解く。
- **Hero / creation role**: 主役素材の入り・抜けを1フレームずつ整えるための close-up editing 面。
- **Layout / visual hierarchy**: 選択 clip を中央で大きく表示し、左右の trim edge に frame label と ResizeHorizontal cursor の見本を出す。元位置の ghost を薄く残し、他の行は背景へ退避する。
- **Interaction / entry**: 端8pxは trim、中央は move、掴んだ瞬間の座標を基準に絶対値で preview。release で1 undo、Esc で ghost 位置へ戻る。後続 clip は ripple せずその場に残る。
- **Density / scale**: 1〜2 clip、0.5〜6秒。bar の幅を最低24px以上に保ち、細すぎる時は fit を優先して誤操作を防ぐ。
- **Reuse vs scratch note**: `clip_gesture`、`hit`、`projection::preview`、`TRIM_EDGE` を再利用。ghost と edge label だけがスクラッチで、編集意味や transient の保存先は増やさない。

## T12 — Keyframe Constellation / キーフレーム星座

- **ID**: `T12`
- **Name**: Keyframe Constellation / キーフレーム星座
- **Problem solved**: 小さな菱形が bar の装飾に見え、key の存在・選択・時刻が読めない問題を解く。
- **Hero / creation role**: 主役の動きの節を星座のように配置し、静止画の並べ替えではなく時間変化を作るためのM19面。
- **Layout / visual hierarchy**: 選択 layer の property 行を縦に並べ、菱形をクリップ面の最前面へ出す。未選択 bar は低彩度、selected key は塗り、未選択 key は輪郭。playhead が通る key は一段強調する。
- **Interaction / entry**: クリック単独、Cmd トグル、Shift 範囲、Delete、キー時刻 drag。複数キー移動は方向順に書き、0〜終端へ clamp。key の当たりは描画より大きい12×12px。
- **Density / scale**: 1 property 4〜24 key、0.5〜20秒。菱形の描画は8×8pxでも hit は12×12pxを維持する。
- **Reuse vs scratch note**: `key_rows`、`key_gesture`、`keys2`、`key_order` を再利用。形・選択 ink・focus の見た目だけを調整し、keyframe storage や property traversal を複製しない。

## T13 — Keyframe Loupe / キーフレーム・ルーペ

- **ID**: `T13`
- **Name**: Keyframe Loupe / キーフレーム・ルーペ
- **Problem solved**: key が密集した時に12×12pxの hit を見つけられず、別の key を動かす問題を解く。
- **Hero / creation role**: 細かい表情・scale pulse・文字の easing point を拡大して調整する精密 creation 面。
- **Layout / visual hierarchy**: 通常 Timeline の右下またはポインタ近くに、時刻軸を共有した拡大 loupe を表示。loupe 内だけ key・frame label・snap guide を大きくし、元の面は context として残す。
- **Interaction / entry**: pointer hover または選択 key で loupe。loupe 内の drag は元の selector へ戻り、edge pan も共通化する。外側をクリックしても別の意味を発火させない。
- **Density / scale**: 本体は0.25〜30秒、loupe は周辺±6〜12フレームを3〜4倍。多数 key の時だけ出し、空の時は表示しない。
- **Reuse vs scratch note**: `projection::geometry`、`key_rows`、`hit`、`EDGE_PAN` を再利用。拡大 overlay はスクラッチだが、別座標系を永続化せず frame↔x の既存逆写像を使う。

## T14 — Retime Bracket / リタイム・ブラケット

- **ID**: `T14`
- **Name**: Retime Bracket / リタイム・ブラケット
- **Problem solved**: 複数 key の比例伸縮が、どの範囲に効いているか分からず、意図しない動きになる問題を解く。
- **Hero / creation role**: モーションの間を伸ばす・詰めることで、hero の呼吸や溜めを作るための専用面。
- **Layout / visual hierarchy**: 選択 key の最初と最後を bracket で囲み、両端に retime handle。範囲内の key は明るく、外側の key と他 property は薄くする。中央に倍率と元/新しいフレームを表示する。
- **Interaction / entry**: まず通常のクリック/Cmd/Shift でキー群を選択し、範囲端を Cmd+drag。プレビューは全選択 key に比例適用、0長 clamp、反転不可。release は1 undo、Esc は選択前の時刻へ戻す。
- **Density / scale**: 1〜3 property、選択範囲をパネルの50〜70%に拡大。外側の時間は圧縮しても、0と終端の境界は残す。
- **Reuse vs scratch note**: `key_gesture::retime`、`key_order`、`projection::properties` と正典の `RetimeSelection` を再利用。bracket、倍率表示だけを新規描画し、独自の retime semantics は作らない。

## T15 — Property Matrix / プロパティ行列

- **ID**: `T15`
- **Name**: Property Matrix / プロパティ行列
- **Problem solved**: 同じ layer の position・opacity・scale の key を横断比較できず、動きの因果が読みづらい問題を解く。
- **Hero / creation role**: hero の複数の変化を同じ時刻軸に揃え、文字の移動とopacityの立ち上がりを一緒に設計するための面。
- **Layout / visual hierarchy**: 左に property 名、右に共通 ruler を持つ行列。各行の playhead と time band は一致し、選択行だけ高彩度。キーがある property だけを既定表示し、全行表示は明示 action にする。
- **Interaction / entry**: 行クリックで focus property、Cmd で複数 key 選択、Shift で時間範囲、右クリックで easing。property 行を跨ぐ選択でも selector は layer/property/frame の3点を失わない。
- **Density / scale**: 4〜12 property、横方向は0.5〜30秒。property 名列は固定、時間面だけを横 scroll。行高は18〜22px。
- **Reuse vs scratch note**: `property_rows`、`key_order`、`rail`、`keys2` を再利用。行列の罫線と選択表現だけを作り、property を新しい集約モデルへ移さない。

## T16 — Graph Underlay / 下段グラフ

- **ID**: `T16`
- **Name**: Graph Underlay / 下段グラフ
- **Problem solved**: key の位置は置けても、Ease in/out や Bezier の変化量を時間面だけで判断できない問題を解く。
- **Hero / creation role**: 主役の加速・減速・overshoot のニュアンスを、通常の編集と同じ playhead で磨くためのM19/P2面。
- **Layout / visual hierarchy**: 上65%を通常 Timeline、下35%を Graph Editor。両方に同じ x 軸と1本の playhead を通し、選択 property の curve だけを明るくする。graph の数値欄は右端に小さく置く。
- **Interaction / entry**: key または property 行から graph を開く。curve handle drag、Hold/Linear/Ease preset、scrub は上下を同時更新。下段を閉じても key 編集の意味は変わらない。
- **Density / scale**: 2〜6 property、上は標準密度、下は1曲線を読み取れる高さを確保。小窓では graph を折り畳み、偽の操作入口を残さない。
- **Reuse vs scratch note**: `graph_editor`、`keys2`、`transport`、`ruler` の既存 projection と曲線値を再利用。上下 split の layout と同期線だけをスクラッチにし、曲線評価を別実装しない。

## T17 — Graph Focus Sheet / グラフ集中シート

- **ID**: `T17`
- **Name**: Graph Focus Sheet / グラフ集中シート
- **Problem solved**: 下段グラフでは制御点が小さく、複雑な easing を読めない問題を解く。
- **Hero / creation role**: hero の動きの質感を一曲線ずつ彫るための集中編集。タイムラインは入口と文脈に退く。
- **Layout / visual hierarchy**: パネルの70%をGraph Editor、左20%を選択 layer/property のリスト、上10%を mini ruler と transport にする。curve のゼロ・1・playheadを強い guide で示す。
- **Interaction / entry**: property 行の graph icon、または選択 key の context action から入る。Bezier handle drag、preset、frame input、Esc cancel。property の切替はリストクリックだけで、曲線面の空白クリックは選択解除にする。
- **Density / scale**: 1 propertyずつ、制御点を24〜32pxの visual hit に拡大。長い時間軸でも曲線の精度を優先し、通常の多数レーンは表示しない。
- **Reuse vs scratch note**: `graph_editor` の projection/evaluation、`transport`、`ruler` を再利用。集中シートの枠と property list だけを作り、Bezier計算や新しい key owner は持たない。

## T18 — Easing Palette / イージング・パレット

- **ID**: `T18`
- **Name**: Easing Palette / イージング・パレット
- **Problem solved**: easing を毎回数値で作る必要があり、確立した動きの語彙へ入れない問題を解く。
- **Hero / creation role**: 文字の登場、ロゴの停止、音のアクセントなど、motivation video に必要な速度感を素早く試すための creation 面。
- **Layout / visual hierarchy**: 選択 key の直下に Linear / Hold / Ease In / Ease Out / Ease In-Out の preset chips。各 chip に小さな curve thumbnail を置き、右端に「Graph を開く」を一つだけ置く。
- **Interaction / entry**: key 選択後に palette を開き、選択範囲へ一括適用。適用前に曲線の薄い preview、クリックで1 undo。graph は高度編集への一方向の入口にし、palette 自体で新しい曲線を発明しない。
- **Density / scale**: 1〜12 key、chip は32px以上の hit。通常の行高を崩さず、palette は選択行の下に一段だけ出す。
- **Reuse vs scratch note**: `EASY_EASE` 群、`keys2` の interpolation action、`graph_editor` を再利用。chip と thumbnail だけをスクラッチにし、既存の preset/curve 値を二重管理しない。

## T19 — Curve Stack / カーブ・スタック

- **ID**: `T19`
- **Name**: Curve Stack / カーブ・スタック
- **Problem solved**: position・scale・opacity の曲線が同時に動くと、どれがheroの印象を作っているか比較できない問題を解く。
- **Hero / creation role**: 主役の動きと透明度、スケールの同期を比較し、過剰な演出を減らしながら意図的なピークを作るための面。
- **Layout / visual hierarchy**: 共通 ruler の下に property ごとの mini curve を縦積み。選択 curve は大きく、他は薄く表示し、同じ playhead を全段に通す。左には property 名と solo/focus の表示だけを置く。
- **Interaction / entry**: property 行または graph focus から開く。curve の選択、key range、solo、playhead scrub を共有。複数 curve を同時編集する入口は置かず、focus を1本ずつ切り替える。
- **Density / scale**: 4〜8曲線、各36〜54px。長い曲線を正確に見るより、複数のピークの位相差を比較するスケールにする。
- **Reuse vs scratch note**: 各曲線は既存 `graph_editor` と `property_rows` を再利用。スタックへの配置と比較用の薄い合成 ink だけをスクラッチにし、曲線を新しいデータへ焼き込まない。

## T20 — Marker Storyboard / マーカー絵コンテ

- **ID**: `T20`
- **Name**: Marker Storyboard / マーカー絵コンテ
- **Problem solved**: 「ここで落とす」「ここで文字を出す」という制作意図が時間上に残らず、再生のたびに探し直す問題を解く。
- **Hero / creation role**: hero の beat、台詞、視線の切り替えを locator にして、時間構成を先に作るための planning 面。
- **Layout / visual hierarchy**: ルーラ下にmarker帯を十分な高さで置き、色・短い名前・小さな icon を表示。clip/key はその下で、marker の縦線だけが全時間面へ通る。playhead が marker に近い時は名前を表示する。
- **Interaction / entry**: `M` tap は即追加、同一 frame は畳む。marker drag は時刻移動、クリックはジャンプ、右クリックは削除、メニュー追加だけ rename mode に入る。再生中も追加できる。
- **Density / scale**: 5〜30 marker、0.5〜60秒。名前は短縮表示し、重なった場合は marker rail の高さを増やすのではなく数をまとめて表示する。
- **Reuse vs scratch note**: `markers`、`ruler`、`nav` の既存動詞と frame geometry を再利用。色タグと名前の ink だけを作り、marker を別の timing owner にしない。

## T21 — Chapter Rail / チャプター・レール

- **ID**: `T21`
- **Name**: Chapter Rail / チャプター・レール
- **Problem solved**: 長い hero video で intro・build・drop・outro の位置関係を失い、局所編集だけになる問題を解く。
- **Hero / creation role**: motivational video 全体の起伏を俯瞰し、各章の長さと間を整えるための macro creation 面。
- **Layout / visual hierarchy**: 上端に広い chapter rail。marker の集合を章ラベルとして横長の淡い blocks で表し、下に通常のルーラとトラックを置く。章は時刻の説明であって新しい clip ではない。
- **Interaction / entry**: marker の複数選択から「章として表示」を選ぶ草案。章クリックでその範囲へ jump、loop、Fit。名前変更は marker rename の既存入口に戻す。未接続の chapter action は disabled にせず story では候補として明示する。
- **Density / scale**: 30秒〜10分、通常トラックは圧縮、chapter rail は24px以上。局所編集へ戻ると T03/T12 相当の近接表示に切り替える。
- **Reuse vs scratch note**: `markers`、`work_area`、`nav`、`projection` を再利用。章の見た目は marker の派生表示としてスクラッチし、永続的な章モデルや新しい時間計算は作らない。

## T22 — Waveform Anchor / 波形アンカー

- **ID**: `T22`
- **Name**: Waveform Anchor / 波形アンカー
- **Problem solved**: 音声レイヤーが普通のbarに見え、playhead と音の立ち上がりがずれているか判断できない問題を解く。
- **Hero / creation role**: beat drop、息継ぎ、効果音の頭に映像・文字を合わせるための M8 の中心面。
- **Layout / visual hierarchy**: video rows の下に audio row を固定し、中央に waveform。playhead は video/audio を貫き、waveform の peak だけを弱い accent、locator と現在 peak を強い accent にする。mute/solo はレーンバーに置く。
- **Interaction / entry**: Space で音と映像を同時再生、scrub で Stage と波形 cursor が同時追従。audio row のクリックは時刻 jump、M は現在時刻へ marker。decode 失敗時もrowと最後の正常状態を残す。
- **Density / scale**: 2〜30秒、audio row 42〜64px、video row は18〜24px。波形の細部より playhead と大 peak の読みやすさを優先する。
- **Reuse vs scratch note**: `audio_rows`、`waveform_view`、`transport`、`ruler` を再利用。新しい PCM 生成や audio clock は作らず、既存の上流 waveform と将来のPlaybackClockを同じ playhead へ投影する。

## T23 — Beat Grid Canvas / ビート・グリッド

- **ID**: `T23`
- **Name**: Beat Grid Canvas / ビート・グリッド
- **Problem solved**: waveform の山だけでは細かい拍の間隔を揃えにくく、映像の動きが音楽からずれる問題を解く。
- **Hero / creation role**: 反復する pulse、点群の跳ね、文字の一拍表示を作るための音楽同期専用の close-up。
- **Layout / visual hierarchy**: audio waveform の背後に拍ごとの細い grid、bar ごとに一段濃い guide、locator は太い diamond。video clip/key は同じ x 軸で上に重ね、beat grid は装飾ではなく snap の候補を示す視覚にする。
- **Interaction / entry**: Cmd+wheel の anchored zoom、beat/locator への snap、M tap、playback。beat metadata が無い story では grid を locator から仮生成し、未実装の自動解析ボタンは置かない。
- **Density / scale**: 1〜8秒、1/4〜1/16拍の切り替えを想定するが、story では2段階だけ見せる。audio row 56px、他は圧縮。
- **Reuse vs scratch note**: `waveform_view`、`markers`、`time_band_segment_frames`、既存 snap geometry を再利用。beat grid の線だけをスクラッチにし、検出・テンポ推定・別の音楽モデルは外部技術へ委託する。

## T24 — A/V Sync Split / A/V同期スプリット

- **ID**: `T24`
- **Name**: A/V Sync Split / A/V同期スプリット
- **Problem solved**: 映像・音声を別々に見ていると、同じ playhead でも音の先行／遅延を発見しづらい問題を解く。
- **Hero / creation role**: spoken title、music video、sound effect の impact を、画と音の到達点として同時に評価するための診断兼 creation 面。
- **Layout / visual hierarchy**: 上半分を video tracks、下半分を audio tracks に分け、中央に一本の共通 playhead。両方の ruler は同じ x 軸、audio の peak と video の key/marker を水平に比較できる。offset warning は該当 row にだけ出す。
- **Interaction / entry**: scrub、再生、marker jump、audio/video row の選択を同じ時刻系で行う。link/unlink は候補 action として story に注記するが、現在の未決意味を勝手に固定しない。nudge は通常の clip/key の既存 move 文法へ戻す。
- **Density / scale**: 2〜6 audio/video rows、0.5〜20秒。上下の境界を24px程度確保し、波形を潰さない。
- **Reuse vs scratch note**: `audio_rows`、`waveform_view`、`projection::geometry`、`transport` を再利用。offset warning と上下 split はスクラッチ、A/V link の永続モデルや専用時計は作らない。

## T25 — Navigator Panorama / ナビゲータ・パノラマ

- **ID**: `T25`
- **Name**: Navigator Panorama / ナビゲータ・パノラマ
- **Problem solved**: 見たい範囲へ zoom/pan するだけで視線が失われ、M18 の移動摩擦が高い問題を解く。
- **Hero / creation role**: 全体のどこを作っているかを保ったまま、局所のhero timingへ入るための navigation 面。
- **Layout / visual hierarchy**: 下端に全comp幅の navigator。layer の色を細い積層として示し、現在 viewport を半透明 knob で囲む。main canvas は近接編集、navigator は全体の位置のためだけに使う。
- **Interaction / entry**: navigator 中央 drag は pan、端6pxは片側固定 zoom、Cmd+wheel はカーソル下時刻を固定、Fit は全体表示。main と navigator の playhead は同じ値を読む。
- **Density / scale**: navigator は0.25秒〜comp終端、main は選択範囲の2〜20秒。縦行数に応じて navigator の高さは24〜36pxで固定。
- **Reuse vs scratch note**: `nav`、`viewport_canvas`、`frame_to_x`、`tick_steps` を再利用。overview ink と knob の見た目だけを作り、Session の zoom/pan owner を二重化しない。

## T26 — Long-form Minimap / 長尺ミニマップ

- **ID**: `T26`
- **Name**: Long-form Minimap / 長尺ミニマップ
- **Problem solved**: 5〜20分の作品で、全体のどこに素材・音・chapterがあるか把握できず、局所を往復できない問題を解く。
- **Hero / creation role**: motivational video の長い build-up と複数の drop を全体設計し、局所編集の位置を失わないための macro面。
- **Layout / visual hierarchy**: 左端または下端に横長 minimap。layer ごとの bar は1〜2pxの集約、選択 layer と chapter marker だけ強調。main timeline は8〜20行に絞り、minimap の viewport rectangle を常時表示する。
- **Interaction / entry**: minimap の任意位置をクリックして jump、viewport rectangle を drag して pan、端を drag して zoom、Fit で戻る。main の選択や playhead は minimap に即時反映する。
- **Density / scale**: 5〜20分、50+レーンを想定。main は0.5〜30秒、minimap は全comp。集約で1px未満になる bar は存在ではなく密度として描く。
- **Reuse vs scratch note**: `rows`、`projection::geometry`、`nav`、`markers` を再利用。read-only の集約描画だけをスクラッチにし、minimap 用の第二Documentや別の編集判定は作らない。

## T27 — Empty Start Pad / 空プロジェクトの発射台

- **ID**: `T27`
- **Name**: Empty Start Pad / 空プロジェクトの発射台
- **Problem solved**: 空のTimelineが「何もできない画面」に見え、最初の素材を置くまで制作が始まらない問題を解く。
- **Hero / creation role**: 空から最初の hero を立ち上げる入口。drop、browse、scrub、keymap が空でも生きていることを示す。
- **Layout / visual hierarchy**: レーン領域中央に大きな drop target と「最初の素材を置く」説明、背景に薄い ruler・playhead・transport。未実装のボタンを並べず、drop target と1つのBrowse入口だけを主役にする。
- **Interaction / entry**: Finder drop または Browse で素材を受ける。拒否時はファイル名と理由をその場に残す。素材が無くても ruler scrub、frame input、M locator が動き、空を無理に仮clipで埋めない。
- **Density / scale**: 0 rows、広い余白、transport 32px、ruler 28px。1素材追加後は T01 または T05 へ自然に遷移する。
- **Reuse vs scratch note**: `rows` の空結果、`transport`、`ruler`、OS drop の既存入口を再利用。drop illustration とCTAだけをスクラッチにし、empty専用のDocument状態やダミーレイヤーは作らない。

## T28 — Recoverable Error Bay / 復旧可能なエラー帯

- **ID**: `T28`
- **Name**: Recoverable Error Bay / 復旧可能なエラー帯
- **Problem solved**: import・waveform・render の失敗が無言で消え、次に何を直せばよいか分からず作品全体を失う問題を解く。
- **Hero / creation role**: 失敗しても制作の文脈と最後の正常なhero frameを残し、直して続行できる信頼面。M13/M16をTimelineで観測する。
- **Layout / visual hierarchy**: 上端transportは通常のまま、直下に一行 status strip。失敗したrowだけ赤い reason badge、最後に成功した frame は通常表示、壊れたwaveformは空白ではなく「取得失敗」と表示する。全画面 modal は使わない。
- **Interaction / entry**: badge クリックで詳細、Retry、素材を外す、別素材へ置換の入口。inspectだけではDocumentを変えない。再生・scrub・undo はエラーrow以外で継続し、拒否は理由を返す。
- **Density / scale**: 通常の行高を維持し、status は28〜40px。エラーが複数でも縦に積まず、件数＋選択中1件の詳細にする。
- **Reuse vs scratch note**: `waveform_view::WaveformState`、row projection、transport、既存 shell の拒否理由経路を再利用。reason badge と retry card だけをスクラッチにし、エラー専用の隠れた状態所有者は作らない。

## T29 — Hero Beatline / ヒーロー・ビートライン

- **ID**: `T29`
- **Name**: Hero Beatline / ヒーロー・ビートライン
- **Problem solved**: 通常の動画ソフトのように素材を横へ並べるだけでは、主役・動機・個性を時間上で立ち上げにくい問題を解く。
- **Hero / creation role**: Motolii固有の「motivational video を作る」視点をTimelineで見せる。主役の登場・加速・ピーク・余韻を、utility tracksより強く読む。
- **Layout / visual hierarchy**: 選択したhero layerを中央の32px以上の beatline として表示し、impact marker、keyframe cluster、音のpeakを同じ縦線へ寄せる。背景・補助レーンは18pxで低彩度。transport と ruler は通常位置を保つ。
- **Interaction / entry**: hero layerを選択すると focus。Mでimpact marker、keyの範囲選択とCmd retime、playhead jump、playback。hero専用の新しいlayer種別は作らず、通常選択の強調として扱う。
- **Density / scale**: hero 1行＋support 6〜12行、2〜30秒。ピーク前後の±1〜2秒を少し広く見せるが、時間軸の値は歪めない。
- **Reuse vs scratch note**: `rail`、`markers`、`key_rows`、`transport`、既存 selection/fold を再利用。hero accent と peak alignment の表示だけをスクラッチにし、Motolii専用の意味モデルを増やさない。

## T30 — Adaptive Two-Mode / Compose・Inspect

- **ID**: `T30`
- **Name**: Adaptive Two-Mode / Compose・Inspect
- **Problem solved**: 一つの固定レイアウトでは、素材を並べる時の広さとkey/easingを詰める時の精度を同時に満たせない問題を解く。
- **Hero / creation role**: Compose でheroの時間構成を作り、Inspect で主役のkey/easingを仕上げる。通常編集とMotoliiの表現編集を一つの制作 loop に戻す。
- **Layout / visual hierarchy**: `Compose` は広いクリップ面・複数レーン・navigatorを優先し、transport/rulerを常設。`Inspect` は選択 layer のproperty行とGraph Editorを広げ、他の行を縮める。切替後も同じplayhead・selection・Documentを映す。
- **Interaction / entry**: mode tab または意味 action で切替、選択行のgraph入口でInspectへ入る。Cmd+wheel、fold、key選択、Esc、playbackは両モードで同じ文法。mode切替は編集の確定ではなくSession状態として扱う。
- **Density / scale**: 3段階のUI scaleを想定。Composeは18〜24px行高で20〜40行、Inspectは24〜32pxのproperty/graphで1〜6行。狭い窓ではInspectのgraphを畳み、触れない入口を残さない。
- **Reuse vs scratch note**: 現在の `TimelinePane` builder、`Session`、`rail`、`canvas`、`key_rows`、`graph_editor` を再利用。modeのレイアウト切替だけをスクラッチにし、モードごとのDocument・playhead・selection・keyframe ownerは作らない。
