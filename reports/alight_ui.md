# Alight Motion を実機で開いて見た記録（10/25）

Alight Motion 6.2.60（Android）を、エミュレーターで 1 画面ずつ開いて記録した。画面ごとに「何がある・UI の形・良い所・悪い所・Motolii ではどうか」を書き、スクショは [alight_ui/](alight_ui/) に入れた。Alight Motion は**下限**で、ゴールではない。

- 見た所：Android Studio のエミュレーター（Pixel Tablet、Android 15）と BlueStacks Air（スマホ）。adb で画面の部品を文字で読み、絵を撮った。
- **見られなかった所**：写真・動画・音のレイヤー（端末の写真への許可を出していない）、有料の機能の中身（カメラは「まず試す」で見た）、書き出しの実行、クラウド、iPad の横の並べ方。
- **横の並べ方は iPad だけ**：Android 版はタブレットでも縦に固定（`SCREEN_ORIENTATION_PORTRAIT`）。Android 15 の縦横比の決まりで無理に横にすると、スマホの並べ方を横に引き伸ばしただけになる（[10_editor](alight_ui/10_editor.jpg)）。Mac の App Store にも無い（対応機種 130 に Mac が無い）。
- 公式ガイドの効果は 169、**アプリの中の効果は 184**（12 分類）。ガイドだけにあるのは Precise Box Blur の 1 つ、アプリだけにあるのは 16（下の「効果の選び画面」）。

## 操作体系の全体（良い所・悪い所）

**良い所**
1. **どの値も同じ形で触る**：パネルを開くと、左の縦の列に「戻る・キーを打つ／消す・イーズ・⋯」が必ず同じ場所にある（[21](alight_ui/21_color_fill.jpg)、[28](alight_ui/28_move_location.jpg)、[76](alight_ui/76_effect_card.jpg)）。色でも位置でも効果のつまみでも、キーの打ち方が 1 つ。
2. **数字はこする目盛り**：細かい刻みの目盛りを横にこすり、右に数字（[30](alight_ui/30_move_scale.jpg)、[76](alight_ui/76_effect_card.jpg)）。指で大きく動かしても値は少しずつ動く。
3. **指で隠れない「こする帯」**：位置と点は、ステージを直接触る代わりに、下の大きな帯をこすって動かせる（「Swipe here to move layer」[28](alight_ui/28_move_location.jpg)、「Swipe to move point」[32](alight_ui/32_edit_points.jpg)）。
4. **押す前に結果が見える**：ブレンドは見本の写真に掛けた絵（[27](alight_ui/27_blend.jpg)）、効果は動く絵（[71](alight_ui/71_effect_category.jpg)）と大きな動く見本（[72](alight_ui/72_effect_preview.jpg)）。
5. **初めての所で吹き出しが指す**：カメラを置くと「この見え方のパネルで Active Camera View を入れる」と、押す場所を指して教える（[61](alight_ui/61_camera_hint.jpg)）。
6. **選んだ数で上の帯が変わる**：複数選ぶと上の帯が緑になり「3 layers selected」と、グループ・マスク・抜く・消す・揃えが出る（[50](alight_ui/50_multiselect.jpg)）。
7. **入れ子にパンくず**：グループに入ると「New Project › Group 1」（[52](alight_ui/52_group_inside.jpg)）。

**悪い所**
1. **有料の壁が操作の途中に出る**：プロジェクトを作る・閉じるたびに課金の画面と広告。カメラ・親・イーズの弾み系・一部の効果は TRY／鍵（[60](alight_ui/60_pro_gate.jpg)）。試験用の広告は閉じるボタンが無く、アプリを止めるしかなかった。
2. **横に並べ直していない**：Android のタブレットでも縦固定。無理に横にすると、効果の見本の「足す」ボタンが画面の外に出て押せない（[73](alight_ui/73_effect_preview_cut.jpg)）。
3. **深い**：値を変えるのに「レイヤーを選ぶ → パネル → タブ → 値」と 3〜4 段。同時に見られる値は 1 パネル分だけ。
4. **見つけにくい操作がある**：複数選ぶのは「コピー貼り付けのメニュー → Select All Layers」か左の丸の印。長押しは何も起きなかった。
5. **キーボード・右クリック・メニューバーが無い**：全部タップ。

## 画面ごと

### 新しいプロジェクト（[01](alight_ui/01_new_project.jpg)）
- **ある**：PROJECT か ELEMENT、名前、縦横比 6 種（16:9・9:16・4:5・1:1・4:3・自由で幅・高さ・つなぎ）、解像度 7 段（2160p〜270p、大きさも表示）、fps 10 段（12・15・18・20・24・25・30・48・50・60）、背景 5 色。
- **UI**：1 枚の小さな箱に全部。縦横比は形の絵で選ぶ。
- **良い**：作る前に全部決まる。縦横比が絵。
- **悪い**：長さを決める欄が無い（レイヤーで決まる）。背景は 5 色だけ。
- **Motolii**：New は untitled を作るだけ。大きさは後で上の帯、**fps と長さはどこからも変えられない**（screens.md）。→ この箱を借りる形が近い。

### プロジェクトの設定（[02](alight_ui/02_project_settings.jpg)）
- **ある**：縦横比・解像度・fps・背景を後から変える、**Total Edit Time**（編集した時間）。
- **良い**：作ったときと同じ箱がそのまま後から開く。
- **Motolii**：文書の設定の欄はあるが隠してある。

### 書き出し（[03](alight_ui/03_export.jpg)、[04](alight_ui/04_export_video.jpg)、[05](alight_ui/05_export_sequence.jpg)）
- **ある**：空き容量と大きさの見積もり。Video（MP4、解像度、H.264 / H.265、fps LOW・REDUCED・FULL、画質 Mbps のつまみ）、Upload to Cloud、Project Package（元の素材ごと、リンクで共有、5MB を超えると有料）、今のコマを PNG、Image Sequence（PNG / JPEG、解像度、fps、画質）、「テンプレートとしても書き出す」。
- **良い**：形式ごとに 1 行、右の矢印で細かい設定。空き容量と見積もりが上に常に出る。
- **悪い**：GIF が見当たらない（宣伝にはある）。透明（アルファ）の動画の選択肢が無い。範囲の指定が無い。
- **Motolii**：外の ffmpeg を決め打ち（60fps・CRF 17）、形式・画質は表示だけ（export_map.md）。

### プロジェクト一覧（[06](alight_ui/06_projects_list.jpg)、[07](alight_ui/07_project_menu.jpg)）
- **ある**：タブ Projects・Your Templates・Elements・Cloud、Upload XML file、並べ替え（名前・日付）、日付ごとの見出し、カードに長さ・縦横比・解像度・大きさ・fps、⋯ で Select・Export・Duplicate・Upload to Cloud・Delete。
- **良い**：カードに作品の仕様が全部出る。
- **Motolii**：開くは同じフォルダの .motolii だけ。一覧の画面が無い。

### 設定（[80](alight_ui/80_settings_menu.jpg)）
- **ある**：Low Quality Preview、Demo Mode、Default Layer Duration（2s）、Audio Resampling（LOW・HIGH・ULTRA）、About・プライバシー・オープンソース・サポート。
- **Motolii**：環境設定の画面が無い。

### 編集画面（[10](alight_ui/10_editor.jpg)）
- **ある**：上に戻る・名前・設定・書き出し。中段に取り消し・やり直し・頭へ・再生・終わりへ・コピー貼り付け・見え方。下にタイムライン（目・選ぶ丸・帯・右端の ≡ で並べ替え）、右下に＋。
- **良い**：取り消しが常に画面にある。
- **悪い**：タイムラインは 1 行に 1 レイヤーで、キーは選んだレイヤーの帯の上にしか出ない。
- **Motolii**：Ableton どおりのタイムライン（ableton_map.md）。

### コピー・貼り付け（[11](alight_ui/11_copy_paste.jpg)）
- **ある**：Select All Layers、Duplicate、Save Preset、Copy（見た目ごと）、Paste Layer、**Paste Style**（見た目だけ、矢印で何を貼るか選ぶ）。
- **良い**：「見た目だけ貼る」がある。
- **Motolii**：レイヤーとキーのコピー貼り付けが無い（screens.md）。

### 見え方（[12](alight_ui/12_view_options.jpg)）
- **ある**：Pixel・Grid・Solo・Camera（カメラ越しに見る）、拡大・縮小と %。
- **Motolii**：Fit・50%・100%・Safe・Grid・Camera/Free（近い）。Pixel（拡大で点を見る）は無い。

### ＋で足す（[13](alight_ui/13_add_shape.jpg)、[14](alight_ui/14_add_shape_p2.jpg)、[15](alight_ui/15_add_object.jpg)）
- **ある**：Shape（図形 54：円・角丸四角・十字・円弧・三角・吹き出し・しずく・雪・盾・扇・六角・花・矢印・台形・三日月・雲・再生・渦・四角・星・線・L・切手・ハート・稲妻・パズル・チェック・旗・いいね・紙飛行機・家・麦・ヨット・虫眼鏡・鍵・四芒星・しおり・指・炎・リボン・月桂樹・太陽・人・ロケット・画鋲・ひし形・飛行機・傘・帯・爆弾・封筒・女の人・車）、Media、Audio、Object / Element（Camera・Null・Empty Group・Element/Project）、Template、Freehand Drawing、Vector Drawing、Text。
- **良い**：足せる物が 1 枚に全部。図形は絵で選ぶ。
- **Motolii**：作るの棚に Text・Camera・Null、図形 5、3D 4、光 3、Repeater、レシピ 45。図形の種類は値の shape で 16。**手描きが無い**。

### レイヤーの ⋯（[20](alight_ui/20_layer_menu.jpg)）
- **ある**：Save to My Elements、左右反転、上下反転、画面に合わせる（Fit・Fill・Stretch）、Create Clipping Mask、ラベルの色 7。
- **Motolii**：反転・画面に合わせる・ラベルの色が無い。

### 色と塗り（[21](alight_ui/21_color_fill.jpg)、[22](alight_ui/22_gradient.jpg)、[23](alight_ui/23_palette.jpg)）
- **ある**：なし・単色・グラデーション（線・円・もう 1 種、2 色）・**メディアで塗る**。見本 14 色、自分の色を足す、スポイト、小さいピッカー、パレット（色相環・RGB、彩度・明るさ・アルファ）。
- **良い**：塗りの種類が 4 つのタブで一目。
- **悪い**：グラデーションは 2 色だけ。
- **Motolii**：色の棚に見本 60・CAM16 のグラデーション（8 点まで）・写真から色。メディアで塗るのは無い。

### 縁と影（[24](alight_ui/24_stroke.jpg)、[25](alight_ui/25_stroke_heads.jpg)、[26](alight_ui/26_shadow.jpg)）
- **ある**：線（色・太さ・端 3・角 3・**始点と終点の形 10 種と大きさ**）、Border（何重にも足す）、影（色・大きさ・アルファ・位置 X・Y）。
- **良い**：矢じりを絵で選ぶ。縁を何重にも。
- **Motolii**：stroke・cap・join・heads・dash・shadow はある。縁を何重にもは無い。

### 重ね方と不透明度（[27](alight_ui/27_blend.jpg)）
- **ある**：不透明度、重ね方 28（Normal／Darken 5／Lighten 5／Contrast 7／Difference 4／Color 4／Mask 2）。
- **UI**：7 分類を畳んで開く、各モードは見本の写真に掛けた絵。
- **良い**：押す前に結果が見える。選んだ名前が見出しに出る。
- **Motolii**：blend 25 と clip。選び方は名前の一覧。

### 移動と変形（[28](alight_ui/28_move_location.jpg)〜[31](alight_ui/31_move_skew.jpg)）
- **ある**：位置 X・Y・Z（こする帯）、回転（丸いダイヤル）、大きさ（幅・高さ・つなぎ、目盛り）、傾き X・Y、⋯ で Reset Skew・Reset Transform。
- **良い**：回転がダイヤル。
- **悪い**：X・Y の回転（3D）がここに無い（効果の Flip Layer）。
- **Motolii**：インスペクタの数字の欄（Ableton どおり）。

### 点の編集（[32](alight_ui/32_edit_points.jpg)）
- **ある**：点を選ぶ細い列（右）、こすって点を動かす帯、閉じる・開く、曲線の種類、点を足す・消す、キーの編集と点の編集の切り替え。
- **良い**：指で隠れない。点を順に送って選べる。
- **Motolii**：ペンで点を足す。点を順に送るのは無い。

### プリセット（[33](alight_ui/33_presets.jpg)）
- **ある**：レイヤーの値とキーをプリセットとして保存、人のを読み込む、Discover Presets。効果ごとのプリセット（Standard Settings）もある。
- **Motolii**：プリセットの棚は「No presets yet」だけ。

### イーズ（[34](alight_ui/34_ease.jpg)、[35](alight_ui/35_ease_menu.jpg)）
- **ある**：ベジェ・弾み系・段々系（TRY）、反転、前・次のカーブへ、Copy Curve・Paste Curve・**Paste Curve to All Keyframes**・Enable Overshoot。
- **悪い**：キーが 2 つ無いと「Keyframes Required」の警告だけで開かない。
- **Motolii**：同じ 9 種（Alight Motion を手本に作った）。カーブのコピー貼り付けは無い。

### 速さ（[36](alight_ui/36_speed.jpg)）
- **ある**：亀とウサギのつまみ（1.00x）、速さを変えるときにどちらの端を止めるかの 4 つ。
- **Motolii**：speed の値はあるが、端を止める選択は無い。

### 文字（[40](alight_ui/40_text_edit.jpg)、[41](alight_ui/41_fonts.jpg)、[42](alight_ui/42_font_filters.jpg)）
- **ある**：揃え・フォント・大きさ・色・決定。フォントは最近使ったもの、View All Fonts（太さごと、星）、絞り込み（Serif・Sans Serif・Display・Handwriting・Monospace・Favorite）、並べ替え（Popular…）、言語、Import Fonts。字送り・行送り・文字ごとの動きは効果（Text Spacing・Text Transform…）。
- **良い**：フォントの分類と言語で絞れる。
- **Motolii**：字体の棚に全部・使用中・星・タグ。言語での絞り込みは無い。

### 手描き（[43](alight_ui/43_freehand.jpg)）
- **ある**：ペン・ブラシ・塗りつぶし・消しゴム、太さ、色。
- **Motolii**：無い。

### 複数選んだとき（[50](alight_ui/50_multiselect.jpg)、[51](alight_ui/51_multiselect_align.jpg)）
- **ある**：グループにする、**グループにしてマスク**、**グループにして抜く**、消す。時間の揃え（始まりを伸ばす・再生位置へ寄せる・左揃え・ずらして並べる・右揃え）。画面の揃え（左・中・右・上・中・下・縦に等間隔・横に等間隔）。
- **良い**：マスクがグループの 1 手。時間を「ずらして並べる」が 1 手。
- **Motolii**：Sequence Layers（ずらして並べる）はある。揃えとグループは無い。

### グループ（[52](alight_ui/52_group_inside.jpg)、[53](alight_ui/53_group_settings.jpg)）
- **ある**：中に入って編集（パンくず）、設定に幅・高さ・背景・Precompose（Dynamic / Fixed Resolution）・Re-timing（Freeze・Stretch・Loop・Loop & Stretch・Blank）。
- **良い**：長さを変えたときの伸び方を選べる（Elements と同じ）。
- **Motolii**：関係の group（並びだけ）。入れ子・伸び方は無い。

### カメラ（[60](alight_ui/60_pro_gate.jpg)〜[64](alight_ui/64_camera_fog.jpg)）
- **ある**：Camera View（遠近・画角 60°・ズーム 1662）、Focus Blur（焦点の距離・被写界深度・ぼけの強さ）、Fog（色・近い所・遠い所）。初めての吹き出し。有料。
- **Motolii**：distance・zoom・focus・aperture・段の fog。

### 効果の選び画面（[70](alight_ui/70_effect_browser.jpg)〜[77](alight_ui/77_effect_menu.jpg)）
- **ある**：検索（**説明文の中まで探す**）、おすすめ 6、分類 12、分類の中は動く絵の並び（有料に鍵）、押すと大きな動く見本・名前・説明・タグ・Guide・効果ごとのプリセット・お気に入り。掛けた効果は畳める札（名前・⋮・消す）、⋮ で Effect Details・Reset to Defaults・Duplicate・Copy Effect。
- **分類と数**：Color & Light 29、Drawing & Edge 17、Blur 18、Distortion/Warp 27、Procedural 23、3D 19、Move/Transform 12、Repeat 7、Matte/Mask/Key 11、Opacity/Visibility 7、Text 6、Other 8（重なりを除いて 184）。
- **アプリだけにある 16**：Advanced Chroma Key、Chromatic Zoom Blur、Color Balance、Energy Beam、Grunge、Key Spill Cleaner、Luma Stamper、Matte Fringe、Page Curl、Parenting Helper、Raster Transform、Ring、Scale Assist、Scatter Field、Streak Strips、Warp Blur。
- **良い**：動く絵・説明文での検索・効果ごとのプリセット。
- **悪い**：横にすると「足す」が画面の外。
- **Motolii**：効果の棚（目的の札、ダブルクリックで付け外し）。絵は止まっている。説明文での検索・効果ごとのプリセットは無い。

## デザイン（見た目）の良い所・悪い所

**良い**
- 暗い地に、選んだ物だけ緑。選んだタブ・つまみの線・選んだつまみの名前が全部同じ緑で、今どこを触っているかが一目。
- レイヤーの種類ごとに帯の色が違う（図形は青、手描きは紫、文字は橙、カメラは赤、グループは朱）。
- アイコンの下に必ず文字。NEW・TRY の小さな札で、新しい物と有料の物が分かる。

**悪い**
- NEW・TRY・鍵の札が多く、どれが今使えるのかが散らかる。
- ホーム・一覧は明るい地、編集は暗い地で、行き来すると眩しい。

## 次に見る所

- 写真・動画・音のレイヤー（端末の写真への許可を出せば見られる）
- iPad の横の並べ方（App Store の iPad 用スクショ、公式の動画）
- 有料の機能の中身（親、イーズの弾み系、鍵の効果）
