# Motolii の窓の画面（10/25、コードから数えた。窓では動かしていない）

画面ごとに「何が載っていて、どれが変えられるか」。（表示）＝見るだけ、（変える）＝変えられる。場所は skin/book/lib/ の下。

## 数

約 79 画面：窓の枠と起動 5、メニュー 6、上の帯 4、棚 12、ステージ 7、タイムライン 5、下のパネル 9、インスペクタ 20。開いて閉じる画面は 4 つ（書き出し、待ちの一覧、選び札の一覧、MIDI のファイル選び）と、その場で開く 2 つ（値の行の右クリック、MIDI のキーの欄）。**画面そのものが無いもの 14 種類**。

## 画面そのものが無いもの（grep で確かめた）

| 無いもの | 今どうなっているか |
|---|---|
| 設定（環境設定） | 無い。View ▸ Skin、上の帯の倍率、手で書き換える ~/.motolii/keybindings.json・tags.json・window.xml だけ |
| 新しい文書の大きさ・コマの速さ・長さ | File ▸ New は untitled_N を作るだけ。大きさは後で上の帯の札で変えられる。**コマの速さと長さはどこからも変えられない**（文書の設定の欄はあるが隠してある） |
| 文書の設定・書き出しの設定 | 書き出しの画面の範囲・大きさ・速さ・形式は表示だけ |
| 好きな場所から開く・保存・別名で保存・閉じる・最近使った文書 | 開くは同じフォルダの .motolii だけ。保存は自動 |
| 読み込みの画面 | メディアの棚からだけ |
| このアプリについて・ヘルプ・ショートカットの一覧と設定・取り消しの履歴 | 無い |
| ヒント（Tooltip） | 窓のどこにも無い |
| 消す時の確認 | 無い |
| 層の名前を変える | 名前を書き換える画面が無い |
| 層やキーのコピーと貼り付け | 無い |
| パネルを戻す・並びを戻す | パネルは閉じられず、並びは保存されない |
| エンジンが読めなかった時の画面 | 灰色のまま何も出ない |
| プリセットを保存する | 無い（Presets の棚は「No presets yet」だけ） |
| インスペクタからイーズのパネルへ | つなぎ（LinkTo.ease）はあるが使われていない |

## 作りかけで止まっている所（見つけたもの）

- カメラのカードの「Target」の選び札が、文書の層ではなく、決め打ちの見本の名前（Backdrop、Hero title…）を出す（sets/inspector/inspector_camera_layout.dart:35,58）
- ペンの「Close path」は保存されるがステージに渡っていない（workspace/stage.dart:337）
- 作るの棚の札にショートカットの文字が出ているが、キーにつながっていない（workspace/dock_make_2.dart:328）
- 書き出しの画面で Esc が効かない（workspace/top_bar.dart:246）
- 上の帯の「fps」と「長さ」の札は表示だけ（workspace/top_bar.dart:17）

## 画面の一覧

### 起動と窓
- **起動**：一時フォルダに Untitled.motolii を作って開く。エンジンが開くまで灰色。エンジンが読めないと灰色のまま何も言わない。倍率（88%）と見た目は ~/.motolii/window.xml から（main.dart:94、workspace/workspace.dart:72,314）
- **パネルの並び**：左に棚とステージ、その下にタイムラインとイーズ。右にインスペクタ、その下にカメラ・デスク・関係。タブは動かせる・分けられる・境目を動かせる。閉じる・最大化はできない。並びは保存されない（workspace/workspace.dart:335,352、workspace/panes.dart:118）
- **窓が小さい時**：全体が縮む（何も言わない）（workspace/workspace.dart:293）

### メニュー（Mac のメニューバー）
- lab_book：Quit だけ
- File：New Document（⌘N）、Open ▸（同じフォルダの .motolii だけ）、Render（⌘R）
- Edit：Undo・Redo、Cut・Copy・Paste（文字の欄だけ）、Select All、Duplicate（⌘D）、Remove
- View ▸ Skin：見た目の一覧、Show Skins Folder
- Animation：Sequence Layers
- メニューに無いキー：0、Space、← →、⇧← ⇧→、Home・End、V M R E P T（workspace/keys.dart:13、workspace/stage.dart:61）

### 上の帯（workspace/top_bar.dart:17,366）
- 「M Motolii」（表示）、取り消し・やり直し（変える）、文書の名前（表示）、選んだ層の名前（表示）、大きさの札（変える：横にこする、ダブルクリックで WxH を打つ）、fps の札（表示）、長さの札（表示）、倍率（変える：50〜200%）、明るい・暗い（変える）、待ちの数、状態の文字、Render
- **状態の文字**：「Saved HH:MM」、または「Writing X …」「Written: X」「Not written: 理由」「Not taken: 理由」（workspace/top_bar.dart:218）
- **待ちの一覧**：赤い点を押す → 最近 40 件の待ち（表示）（workspace/top_bar_2.dart:4）
- **書き出し**：ファイル名（変える）、フォルダ・範囲・大きさ・速さ・形式（表示）、Cancel・Render、Enter で書き出す、外を押すと閉じる、**Esc は効かない**（workspace/top_bar.dart:246）

### 棚
- どの棚も：見出しと数、検索と ✕、タグの補い、絞り込みの札、選んだ物の下の帯。何も無い時「Nothing matches “q”」
- **メディア**：絵と一覧、下のフォルダも、木の位置。フォルダの木（Project と Home）。札：押すと選ぶ、ダブルクリックで置く、ステージに落とすと置く、動画は乗ると流し見・▶■、使っている印（workspace/dock_files_browser.dart、dock_files_contents.dart）
- **効果**：目的の札、層の効果（層を選んだ時だけ）、画面の効果（順番の数）、動き。ダブルクリックか落とすで付け外し（workspace/dock_effects.dart:12）
- **字体**：全部・使用中・星・タグ。字体そのもので書いた行か札。ダブルクリックで選んだ文字の字体に、無ければ新しい文字（workspace/dock_fonts.dart:62）
- **色**：素材（Glass）、輪、使っている色と「Save as palette」、色の組と「From photo」、グラデーション、見つけた色、見本 60（workspace/dock_make.dart、dock_make_2.dart）
- **グラデーションを作る**：見本、CAM16 と sRGB、点（押すと消える）、Add stop（8 まで）・Save・Clear（workspace/dock_make_2.dart:6）
- **作る**：Recipes、Layers（Text、Camera、Null）、Shapes（Rect、Ellipse、Polygon、Star、Line）、Copies（Repeater）、3D（Ring、Gem、Cube、Ball）、Light（Sun、Point、Spot）。ショートカットの文字は出ているがつながっていない（workspace/dock_make_2.dart:328）
- **リピーター**：数 1〜40、段、曲がり 0〜360°（workspace/dock_make_3.dart:4）
- **プリセット**：「No presets yet」だけ（workspace/dock.dart:181）

### ステージ
- 見出し：Fit・50%・100%、Safe、Grid、Camera・Free。棚の札の落とし先（workspace/stage.dart:73）
- ツールの列：Select V、Move M、Rectangle R、Ellipse E、Pen P、Text T。塗りと線の見本（表示）（workspace/stage.dart:22,212,266）
- ツールの帯：Select（Layer・Group、Snap、Pivots）、Move（Free・X・Y、Snap）、Rect（Fill・Stroke・Radius は表示、Shape・Mask は変える）、Ellipse（Corner・Centre）、Pen（Path・Mask、Close path はつながっていない）、Text（Font・Size は表示、L・C・R は変える）（workspace/stage.dart:337）
- 読み取り：「Nothing selected」「N picked」、層の名前と P x,y、S %、R °（表示）（workspace/stage.dart:447）
- 本体：押す・⇧ ⌘、ドラッグ、取っ手、投げ縄、図形・ペン・文字、定規からガイド、ホイール・2 本指で動かす、ピンチで段（workspace/stage_view_2.dart:393）
- 定規・倍率と XY・全体図（workspace/stage_view_3.dart:348,380）
- 関係の帯：「N picked」と Around・Row・Follow・Push・Group、関係を選ぶと目・Bake・Delete、取っ手・✕・王冠（workspace/stage_view_3.dart:9,117,129）

### タイムライン
- 見出し：文書の名前（表示）、先頭・戻る・再生・進む・ループ、時刻と長さ（表示）、fps（表示）、選んだ層（表示）、拡大の −・スライダー・＋・Fit（workspace/timeline_header.dart:4）
- 層の名前：番号、種類、名前（表示）、目・ソロ・鍵。押す・⇧・⌘、上下にドラッグ（workspace/timeline_labels.dart、timeline_rows.dart）
- ルーラー：こする、作業範囲、ダブルクリックでマーカー、マーカーをドラッグ・外へで消す、拍の目盛り・溜めたコマの帯（表示）（workspace/timeline.dart:387）
- マーカーの名前：旗をダブルクリック → その場で打つ（workspace/timeline.dart:465）
- 帯：本体で動かす、端で縮める、キー、囲み、下を押すと外す、ホイール・2 本指・⌘ホイール・ピンチ（workspace/timeline.dart:405）
- 全体の帯（workspace/timeline_overview.dart:4）

### 下のパネル
- **イーズ**：区間の帯（「No keys picked」など）、曲線の取っ手、影響と速度（表示）、x1 y1 x2 y2（変える）、見本（Linear、Ease、In、Out、In-out、Back、Hold、エンジンのほかのイーズ）（workspace/desk_ease*.dart）
- **カメラ（深さ）**：mm・f/・視野（表示）、ピント、上から見た図（ピントの線・板・円錐の縁）、Focal・Aperture・Dist（変える）、被写界深度・Near・Far（表示）。カメラの層が無いと保存されない（workspace/desk_depth.dart:73,148）
- **デスク**：「Follows selection」「Pinned · Follow」、Tools。いつも一覧から始まる（workspace/desk.dart）
- **デスク ▸ Blend**：層ごとに名前・混ぜ方・不透明度・下で切る（workspace/desk_blend.dart:12）
- **デスク ▸ Align**：6 つのボタン（画面に対して）（workspace/desk_align.dart:11）
- **関係**：層・カメラ・関係の 3D の図。回す・寄る・軸の玉・選ぶ（workspace/desk_relations.dart:16）

### インスペクタ
- 関係の重ね（選んだ層が関係に入っている時）（workspace/stage_view_3.dart:204）
- 何も選んでいない時：「Nothing selected」と、ステージのカード。文書の大きさ・速さの欄は隠してある（sets/inspector/inspector_choosers_header.dart:340）
- 見出し：種類、名前（表示）、Animate all（変える）（sets/inspector/inspector_choosers_header.dart:231）
- カード：題を押してたたむ、たたむと要約とキーの点
- Transform（2D・2.5D・3D、小さな図、位置・大きさ・回転）、Anchor（27 点）、Layer（不透明度、親）、Camera（Target は見本の名前、Orbit、Framing、Roll）、Text（中身、字体、大きさ、揃え）、Fill and stroke、Blending、Light・Copies・Drop Shadow・Trim Paths（図で触る）、Wave・Physics・Scatter・Field・Connect（動きの見本）、ほかのエンジンのカード（filament/doc_values.h:286 の一覧）
- 値の札の種類：オン・オフ、選ぶ、言葉、ファイル、棒、色、角度、%、こする、さいころ、＋−、2D の板
- 数字の欄：こする（⇧×10、⌥×0.1）、矢印、Enter で打つ、「Number required」、「— Mixed」、キーの点
- 選び札の一覧：検索、一覧、「No match」、Esc・外で閉じる
- 値の行の右クリック：Key this frame・Remove key・Remove animation・Keys from MIDI…
- MIDI のキーの欄：Choose…（Mac のファイル選び。窓で唯一のファイル選び）、チャンネル、BPM、種類、Apply、まちがいの言葉
