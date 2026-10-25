# 窓の動き：どのアプリの動きか・出典なしか（10/25）

窓（`skin/book/lib/workspace/` と `sets/inspector/` のうち窓に出ているもの）の、ツール・クリック・右クリック・ドラッグ・キーを1つずつ。
「出典」はコードのコメントか ASSETS.md にアプリ名が書いてあるもの。書いていなければ **なし**。「似ている（推測）」は調べた者の見立てで、確かめていない。
Widgetbook だけの試作（Host・Idea・Cards・Plans の 48 行、うち出典あり 4）は窓に出ていないので外した。

## 数

| 場所 | 動き | 出典あり | なし |
|---|---|---|---|
| ステージ・ツール・上の帯・キー・パネルの並び | 89 | 22 | 67 |
| タイムライン・下のパネル（イーズ・ブレンド・カメラ・関係）・取り消し | 92 | 17 | 75 |
| 左の棚（メディア・効果・字体・色・作る）・インスペクタ | 129 | 27 | 102 |
| **計** | **310** | **66（21%）** | **244（79%）** |

出典ありの内訳（多い順）：After Effects がいちばん多い。ほかに Ableton、Blender、Figma、Photoshop、Cavalry、AEViewer 2、om_midi、VS Code（キーの書き換えファイル）、ライブラリ（package:docking、flutter_colorpicker、file_selector、material_color_utilities）。

## 動いていない・おかしい所（先に）

- ペンの「Close path」の切り替え：押せるが何も読んでいない（`stage.dart:361`）
- 表示の拡大・縮小のキーが無い（`zoom.dart:21` の WsZoom.by はどこからも呼ばれない）
- ⌘X・⌘C・⌘V：文字の欄でしか効かない。物の切り取り・コピー・貼り付けは無い（`workspace.dart:143`）
- 塗り・線・角丸・字体・大きさの欄（ツールの帯）：表示だけで変えられない（`stage.dart:417`）
- 最初に窓の並びが変わって見えたのは、文書が開くまで空の箱を出し、開いてから並びを組むため（`workspace.dart:314`→`352`）。押したせいではない

## キー（いまの割り当て、`workspace/keys.dart`）

| キー | 動き | 出典 |
|---|---|---|
| Space | 再生・止める | After Effects |
| Home / End | 最初・最後のコマ | After Effects |
| ← / → | 1コマ戻る・進む | なし（似ている：Premiere） |
| ⇧← / ⇧→ | 10コマ | なし（似ている：Premiere） |
| ⌘D | 複製して複製を選ぶ | After Effects（Ableton も ⌘D） |
| 0 | 見える・見えないを切り替え | Ableton（クリップのオン・オフ） |
| ⌘A | 全部選ぶ | Ableton |
| ⌘N | 新しい文書 | なし（Mac の決まり） |
| ⌘R | 書き出しの画面 | なし |
| ⌘Z / ⌘⇧Z | 取り消し・やり直し | なし（Mac の決まり） |
| ⌘X / ⌘C / ⌘V | 文字の欄だけ | なし |
| Delete / Backspace | 選んだキー、無ければ物を消す | なし |
| V M R E P T | 選択・移動・四角・楕円・ペン・文字（ステージにいる時） | なし（似ている：Figma・Illustrator） |
| Tab / ⇧Tab / 矢印 | 次・前の層（コメントだけ。割り当ては見つからず） | なし |
| Esc | 選択を外す・ドラッグを取り消す | なし（似ている：Blender） |
| ⇧（ドラッグ中） | 細かく（0.1 倍） | なし（似ている：Blender） |
| ⇧ / ⌥（数字をこする時） | 10 倍・細かく | After Effects |
| `~/.motolii/keybindings.json` | キーを書き換える | VS Code |

## ステージ・ツール

| 動き | 出典 | 場所 |
|---|---|---|
| ツールの列を押す → そのツール | なし（似ている：Photoshop・Illustrator） | stage.dart:234 |
| Fit / 50% / 100% | なし（似ている：After Effects） | stage.dart:90 |
| Safe（タイトルセーフ）の表示 | なし（似ている：After Effects） | stage.dart:92 |
| グリッド 12×7 | なし（似ている：After Effects） | stage.dart:94 |
| カメラ越し・自由 の切り替え | なし | stage.dart:96 |
| 棚の札をステージに落とす → 札のダブルクリックと同じ | Ableton（デバイスをトラックに落とす） | stage.dart:76 |
| 物に乗る → 札の色で縁どり、XY と定規に線 | Ableton（クリップごとの色） | stage_view_2.dart:374 |
| ホイール → 拡大時だけ動かす。Fit では何もしない。つまんで拡大は無い | なし（似ている：Figma） | stage_view_2.dart:358 |
| 拡大時に右下へ小さな全体図 | なし（似ている：Photoshop のナビゲーター） | stage_view_2.dart:453 |
| 物を押す → 選ぶ／何も無い所 → 外す | なし | stage_view_2.dart:389 |
| ⇧・⌘ を押しながら → 足す・外す | なし（似ている：Figma・Illustrator） | stage_view_2.dart:390 |
| 選択を「グループ」にして押す → いちばん外の親を選ぶ | なし（似ている：Illustrator・Figma） | stage_view_2.dart:85 |
| 物をドラッグ → 選んで動かし、再生を止める | なし | stage_view_2.dart:318 |
| 移動ツールを X / Y にしてドラッグ → 片方の軸だけ | なし（似ている：Illustrator の ⇧） | stage_view_2.dart:343 |
| スナップ → 6 px で縁・真ん中・ガイドにくっつき、線が出る | After Effects | stage_view_2.dart:99 |
| 角をドラッグ → 大きさ／上のつまみ → 回転／十字 → 回転の中心 | なし（似ている：After Effects・Illustrator・Figma） | stage_view_2.dart:300 |
| 線の点・取っ手をドラッグ | なし（似ている：Illustrator のダイレクト選択） | stage_view_2.dart:302 |
| 何も無い所を自由に描く → 投げ縄 | なし（似ている：Photoshop・Blender） | stage_view_2.dart:196 |
| 何も無い所をまっすぐドラッグ → 四角で囲んで選ぶ | Figma・After Effects | stage_view_2.dart:204 |
| 定規から引き出す → ガイド／定規に戻す → 消す | なし（似ている：Photoshop・Illustrator） | stage_view_2.dart:158 |
| 四角・楕円をドラッグで描く／1回押す → 決まった大きさで置く | なし（似ている：Illustrator・Figma） | stage_view_2.dart:211, 388 |
| 楕円を「真ん中から」 | なし（似ている：Illustrator の ⌥） | stage_view_2.dart:217 |
| 四角・ペンを「マスク」 → 選んだ物の後ろで切り抜く | After Effects（マット） | stage_view_2.dart:235 |
| ペン：押す → 角の点／ドラッグ → 取っ手つき | なし（似ている：Illustrator） | stage_view_2.dart:258 |
| ペン：閉じた線の上 → 点を足す・消す／ほか → 新しい線 | なし（似ている：Illustrator） | stage_view_2.dart:273 |
| ペン中にツールを変える → 次は新しい線 | なし | stage_view_2.dart:20 |
| 文字の左・中・右揃え | なし | stage.dart:365 |
| 塗りと線の見本（列の下） | Photoshop | stage.dart:264 |
| 2つ以上選ぶ → 上に「Around / Row / Follow / Push / Group」 | なし | stage_view_3.dart:138 |
| Group → 下から順に出てくる動き | なし | relate.dart:12 |
| 関係の十字・矢印をドラッグ → 真ん中・半径・間 | なし | stage_view_3.dart:43 |
| 関係の × → 外す／王冠 → 先頭・真ん中を変える | なし | stage_view_3.dart:117 |
| 関係の目 → オン・オフ | After Effects（エクスプレッションのスイッチ） | stage_view_3.dart:141 |
| 関係の Bake / Delete を並べる | Blender（Apply と Delete） | stage_view_3.dart:142 |

## 上の帯・メニュー・窓

| 動き | 出典 | 場所 |
|---|---|---|
| 取り消し・やり直しの矢印 | なし | top_bar.dart:118 |
| 画面の大きさの札を横にこする → 幅を変える／ダブルクリック → 打つ | なし | top_bar.dart:430 |
| 表示の倍率を横にこする | Ableton（Zoom Display）・ブラウザの ⌘+ | zoom.dart:41 |
| 倍率をダブルクリック → 88% に戻す | なし | zoom.dart:47 |
| 半分の丸 → 明るい・暗い | なし | top_bar.dart:173 |
| Render → 書き出しの画面 | なし（似ている：After Effects・Premiere） | top_bar.dart:271 |
| 赤い点 → 待ちの一覧 | なし | top_bar_2.dart:55 |
| メニュー File / Edit / View | なし | workspace.dart:191 |
| Animation › Sequence Layers | After Effects（キーフレーム補助） | workspace.dart:245 |
| 文書なしで開く → 新しい文書 | After Effects | workspace.dart:72 |
| 1280×800 より小さい → 全体を縮める | なし | workspace.dart:294 |
| パネルのタブを動かす・分ける・境目を動かす | package:docking | panes.dart:120 |
| パネルを閉じる・最大化はできない | package:docking（切ってある） | panes.dart:111 |
| よそから棚を呼ぶ → タブが前に出て並びを組み直す | なし | panes.dart:72 |

## タイムライン

| 動き | 出典 | 場所 |
|---|---|---|
| 先頭へ・1コマ・再生・ループ のボタン | なし | timeline_header.dart:38 |
| − / + / スライダー / Fit → 拡大縮小（再生位置を中心に） | なし | timeline_header.dart:78 |
| ⌘ホイール → 拡大／⇧ホイール → 時間を動かす／ホイール → 段を上下 | なし | timeline.dart:87 |
| 再生が端を越える → 画面がついていく | なし | timeline.dart:76 |
| キーを押す → 選ぶ／⇧ → 足す | なし（似ている：After Effects） | timeline_hand.dart:27 |
| 帯を押す → 層を選ぶ／空いた所 → キーを外す | なし | timeline_hand.dart:31 |
| いちばん下の下を押す → 全部外す | After Effects | timeline_hand.dart:10 |
| 乗った所で矢印の形が変わる | なし | timeline_hand.dart:37 |
| キーをドラッグ → コマ単位で動く | After Effects | key_drag.dart:1 |
| 帯をドラッグ → 時間をずらす／端 → 縮める | なし（似ている：After Effects） | timeline.dart:149 |
| 空いた所をドラッグ → 四角で囲んでキーか層を選ぶ | Ableton・After Effects | timeline.dart:108 |
| 定規を押す・こする → 再生位置 | なし（似ている：After Effects） | timeline_ruler.dart:35 |
| 作業範囲の端・まん中をドラッグ | なし（似ている：After Effects） | timeline_ruler.dart:26 |
| 定規をダブルクリック → マーカー（M1, M2…）／旗をダブルクリック → 名前 | After Effects | timeline.dart:175 |
| 旗をドラッグ → 動かす／定規の外へ → 消す | After Effects | timeline.dart:240 |
| 全体の帯を押す → そこへ | なし | timeline_overview.dart:21 |
| 層の名前を押す → 選ぶ／⇧ → 間を全部 | After Effects（⇧） | timeline_rows.dart:62 |
| ⌘ を押しながら → 1つ足す・外す | なし（似ている：Finder・Figma） | timeline_rows.dart:63 |
| 層を上下にドラッグ → 前後を変える | なし（似ている：After Effects） | timeline_rows.dart:47 |
| 目 → 見える・見えない | なし（似ている：After Effects） | timeline_rows.dart:12 |
| ソロ・鍵 | After Effects | timeline_rows.dart:19 |
| キーを打つ・消す（再生位置） | なし（似ている：After Effects） | session_playhead.dart:40 |
| キーのある物を変える → 自動でキー | なし（似ている：After Effects） | session_undo.dart:162 |
| 複数を選んで変える → 全部に同じ変化、打てば全部同じ値 | なし（似ている：Figma の Mixed） | session_undo.dart:78 |
| 取り消し → どのパネルの操作も1つずつ | なし | session_undo.dart:46 |

## 下のパネル（デスク）

| 動き | 出典 | 場所 |
|---|---|---|
| 選んだ物でパネルの中身が勝手に変わる（キー → イーズ、カメラ → 深さ、ほか → 道具） | なし | desk.dart:1 |
| 固定・ついていく の切り替え／道具の札を押す → 開いて固定 | なし | desk.dart:47 |
| 数字を横にこする | なし（似ている：After Effects） | desk_kit.dart:242 |
| 整列 6 個 | なし（似ている：Figma・Illustrator） | desk_align.dart:22 |
| 混ぜ方を選ぶ（並びは After Effects の組） | After Effects | desk_blend.dart:107 |
| 不透明度の帯 | なし | desk_blend.dart:136 |
| Clip → 下の層の形で切る | なし（似ている：Photoshop のクリッピングマスク） | desk_blend.dart:68 |
| 深さ：板を押す → ピント／線を上下 → ピントの距離 | なし | desk_depth.dart:258 |
| 深さ：横 → 焦点距離 | 「Transform」（どのアプリか分からない） | desk_depth.dart:269 |
| イーズ：取っ手をドラッグ | なし（似ている：CSS の cubic-bezier） | desk_ease_graph.dart:50 |
| イーズ：影響と速さの数（表示） | After Effects | desk_ease_numbers.dart:9 |
| イーズ：見本の札（Linear, Ease, In, Out…）／反転 | なし（似ている：CSS） | desk_ease_presets.dart:29 |
| 関係：ドラッグで回す／ホイールで寄る | なし（似ている：Blender） | desk_relations.dart:93 |
| 関係：軸の玉 → その向きから見る | Blender | desk_relations.dart:98 |
| 関係：選んだ物の道筋だけ | After Effects | desk_relations.dart:291 |

## 左の棚

| 動き | 出典 | 場所 |
|---|---|---|
| メディア：並べ方（絵・一覧）、フォルダの木、木の高さをドラッグ | AEViewer 2 | dock.dart:111 |
| 検索・タグの補い・絞り込みの札・星 | なし | dock_parts.dart:361 |
| 札に乗る → 光る | なし | dock_parts.dart:75 |
| 札を押す → 選ぶ／ダブルクリック → 置く・付ける／ステージに落とす → 同じ | なし（似ている：After Effects のプロジェクトパネル） | dock_parts.dart:81 |
| 下のボタン（Place / Apply / Add…） | なし | dock_parts.dart:285 |
| パンくず | なし（似ている：Finder） | dock_files_browser.dart:200 |
| 動画の札の上を横に動かす → 8 コマ流し見 | なし（似ている：Final Cut Pro のスキミング） | dock_files_contents.dart:176 |
| 効果・字体：選ぶ・ダブルクリック・落とす・星 | なし | dock_effects.dart:78 |
| 色：輪と帯 | flutter_colorpicker | dock_make.dart:456 |
| 色：見本を押す・ダブルクリック・落とす | なし | dock_make_2.dart:157 |
| 色：写真から色の組 | material_color_utilities（Material You） | dock_make.dart:322 |
| グラデーションを作る（CAM16 と sRGB、点の足し引き） | なし | dock_make_2.dart:22 |
| 作る：札を落とす・ダブルクリック → 作る | なし（作る中身は Cavalry のチュートリアル） | dock_make_2.dart:411 |
| リピーター | After Effects | dock_make_2.dart:300 |
| 数・段・曲がりのスライダー | なし | dock_make_3.dart:24 |

## インスペクタ

| 動き | 出典 | 場所 |
|---|---|---|
| 値の行を右クリック → キーを打つ・消す・MIDI から・関係… | om_midi（MIDI のみ）。ほかはなし | inspector_rows_prop_row.dart:201 |
| MIDI のパネル（チャンネル・BPM・種類・Apply） | om_midi（Portal.ts） | inspector_rows_midi.dart:86 |
| ファイル選び | file_selector（OS のもの） | inspector_rows_midi.dart:35 |
| 行の印を横にこする | Figma | inspector_rows_prop_row.dart:124 |
| 数字を押してこする／⇧ ⌥ で速さ／ダブルクリック | After Effects | inspector_state.dart:53 |
| こすりながら上下 → 細かさを変える | なし | num_field_state.dart:107 |
| ⇧ を押して欄を選ぶ → 四角く複数 | 表計算 | inspector_state_2.dart:312 |
| 菱形 → キーを打つ・消す／矢印 → 元に戻す | なし（似ている：After Effects のストップウォッチ） | inspector_rows_prop_row.dart:239 |
| 横スクロール → 値を動かす／矢印キー・Enter・Delete・Esc | なし | num_field_state.dart:196 |
| ＋−、さいころ、角度の輪、オン・オフ、選び札、2D の板、棒の並び | なし | inspector_param_controls*.dart |
| 文字の欄：離れたら決まる、Esc で戻す | Cavalry | inspector_param_controls_tile.dart:127 |
| 色・字体の値 → 色・字体の棚を開く | なし | inspector_panel_link.dart:68 |
| カードの名前 → たたむ／Advanced | なし | inspector_rows_sect.dart:147 |
| カードの ▲▼ | After Effects | inspector_effects.dart:128 |
| 効果の頭を長押ししてドラッグ → 順番 | なし（似ている：After Effects） | inspector_effects_effect_card.dart:73 |
| 効果の More（前へ・後へ・戻す・ランダム・消す）、オン・オフ | なし | inspector_effects_effect_card.dart:94 |
| 選び札 → 検索つき一覧 | なし | inspector_choosers_header.dart:156 |
| 親の一覧 | After Effects | inspector_transform_rows.dart:439 |
| 「Animate all」 → 何を変えてもキー | なし（似ている：Blender の自動キー） | inspector_choosers_header.dart:327 |
| 2D / 2.5D / 3D、大きさの鎖、回転の中心の 9 点 | なし | inspector_transform_rows.dart:56 |
| 小さな図で動かす・大きさ・回転・中心 | なし | inspector_direct_transform_canvas.dart:112 |
| 影の図をドラッグ | Photoshop（レイヤースタイル） | inspector_shadow.dart:44 |
| トリムの図の端・まん中 | After Effects | inspector_trim.dart:48 |
| 光の図のつまみ | Ableton のフィルター・OP-1 | inspector_light.dart:60 |
| カードに乗る → 動きの見本が動く／見本をドラッグ → 2 つの値 | Good Boy Ninja の Wiggle | inspector_motion_preview.dart:132 |
| 複製の図の ＋ とつまみ | なし | inspector_copies.dart:39 |
| 関係の行、層の札 | なし | inspector_relations.dart:28 |

## 次に足す列（まだ空）

Ableton を土台にする話（キーは Ableton、Ableton に無いものは Illustrator・Figma、拡張がもっと良く直していれば拡張）に合わせて、この表の行ごとに次を埋める：

| いまの動き | お手本（本体） | 拡張が直したこと | マニュアルの出典 | 借りられるコード |
|---|---|---|---|---|
