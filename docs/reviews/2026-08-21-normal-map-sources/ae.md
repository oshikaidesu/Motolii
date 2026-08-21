出典メモ:
- MENU_PDF = https://www.provideocoalition.com/wp-content/uploads/AECC_MenuIDs_v1_1_1-1.pdf (David Torno作, AE CC 12.2.1x5からExtendScriptで抽出したメニューコマンドID一覧。Adobe公式ではないが実機メニュー構造を機械的に採取した一次資料に近い技術文書)
- SHORTCUT_URL = https://helpx.adobe.com/after-effects/desktop/get-started/keyboard-shortcuts/keyboard-shortcuts-reference.html (Adobe公式、r.jina.ai経由で取得)
- PREFS_URL = https://helpx.adobe.com/after-effects/using/preferences.html (Adobe公式、WebSearch要約経由で内容確認。直接fetchはhelpx.adobe.comへのタイムアウトで不可)
- WORKSPACE_URL = https://helpx.adobe.com/after-effects/using/workspaces-panels-viewers.html (Adobe公式、直接fetch不可・WebSearch要約経由)
- PROPS_URL = https://helpx.adobe.com/after-effects/using/properties-panel.html (Adobe公式)
- SCRIPTPREF_URL = https://community.adobe.com/t5/after-effects-discussions/scripting-option-amp-expresions-is-not-displayed-within-the-after-effects-preferences-menu/m-p/12134869 (Scripting & Expressions環境設定の項目詳細。2019年AE 16.0で新設、MENU_PDF(2016年)には未収録)

種別	パス	項目名(英)	意味1行(日)	出典URL

## 1. menu (メニュー木)
menu	After Effects	About After Effects…	バージョン情報	MENU_PDF
menu	After Effects > Preferences	General…	その他一般設定	MENU_PDF
menu	After Effects > Preferences	Previews…	プレビュー動作設定	MENU_PDF
menu	After Effects > Preferences	Display…	モーションパス等の表示設定	MENU_PDF
menu	After Effects > Preferences	Import…	読み込み時の既定動作設定	MENU_PDF
menu	After Effects > Preferences	Output…	出力時の既定動作設定	MENU_PDF
menu	After Effects > Preferences	Grid & Guides…	グリッド・ガイド・セーフマージン設定	MENU_PDF
menu	After Effects > Preferences	Labels…	ラベル色の設定	MENU_PDF
menu	After Effects > Preferences	Media & Disk Cache…	メディア・ディスクキャッシュ設定	MENU_PDF
menu	After Effects > Preferences	Video Preview…	外部ビデオプレビュー出力設定	MENU_PDF
menu	After Effects > Preferences	Appearance…	UI外観・ハイライト色設定	MENU_PDF
menu	After Effects > Preferences	Auto-Save…	自動保存の間隔・世代数設定	MENU_PDF
menu	After Effects > Preferences	Memory & Multiprocessing…	メモリ・マルチプロセス設定	MENU_PDF
menu	After Effects > Preferences	Audio Hardware…	オーディオデバイス設定	MENU_PDF
menu	After Effects > Preferences	Audio Output Mapping…	オーディオ出力チャンネル割当	MENU_PDF
menu	After Effects > Preferences	Sync Settings…	Creative Cloud経由の設定同期	MENU_PDF
menu	After Effects	Hide After Effects	アプリを隠す(macOS)	MENU_PDF
menu	After Effects	Hide Others	他アプリを隠す(macOS)	MENU_PDF
menu	After Effects	Show All	全アプリ表示(macOS)	MENU_PDF
menu	After Effects	Quit After Effects	終了	MENU_PDF
menu	File > New	New Project	新規プロジェクト	MENU_PDF
menu	File > New	New Folder	新規フォルダ	MENU_PDF
menu	File > New	Adobe Photoshop File…	新規Photoshopファイル作成	MENU_PDF
menu	File > New	MAXON CINEMA 4D File…	新規C4Dファイル作成	MENU_PDF
menu	File	Open Project…	プロジェクトを開く	MENU_PDF
menu	File	Browse in Bridge…	Bridgeで参照	MENU_PDF
menu	File	Close	現在のビューを閉じる	MENU_PDF
menu	File	Close Project	プロジェクトを閉じる	MENU_PDF
menu	File	Save	保存	MENU_PDF
menu	File > Save As	Save As…	名前を付けて保存	MENU_PDF
menu	File > Save As	Save a Copy…	コピーを保存	MENU_PDF
menu	File > Save As	Save a Copy As XML…	XML形式でコピー保存	MENU_PDF
menu	File > Save As	Save a Copy As CS6…	CS6形式でコピー保存	MENU_PDF
menu	File	Increment and Save	バージョン番号を上げて保存	MENU_PDF
menu	File	Revert	直前保存状態に戻す	MENU_PDF
menu	File > Import	File…	ファイルを読み込む	MENU_PDF
menu	File > Import	Multiple Files…	複数ファイルを読み込む	MENU_PDF
menu	File > Import	Adobe Premiere Pro Project…	Premiereプロジェクトを読み込む	MENU_PDF
menu	File > Import	Pro Import After Effects…	Premiere用インポータ	MENU_PDF
menu	File > Import	Vanishing Point (.vpe)…	Vanishing Pointデータ読み込み	MENU_PDF
menu	File > Import	Placeholder…	プレースホルダーを読み込む	MENU_PDF
menu	File > Import	Solid…	単色を読み込む	MENU_PDF
menu	File > Export	Add to Adobe Media Encoder Queue…	AMEキューに追加	MENU_PDF
menu	File > Export	Add to Render Queue	レンダーキューに追加	MENU_PDF
menu	File > Export	Adobe Flash Player (SWF)…	SWF書き出し	MENU_PDF
menu	File > Export	Adobe Premiere Pro Project…	Premiereプロジェクトへ書き出し	MENU_PDF
menu	File > Export	MAXON CINEMA 4D Exporter…	C4D形式で書き出し	MENU_PDF
menu	File	Find…	プロジェクト内検索	MENU_PDF
menu	File	Add Footage to Comp	素材をコンポへ追加	MENU_PDF
menu	File	New Comp from Selection…	選択素材から新規コンポ作成	MENU_PDF
menu	File > Dependencies	Collect Files…	依存ファイルを収集	MENU_PDF
menu	File > Dependencies	Consolidate All Footage	重複素材を統合	MENU_PDF
menu	File > Dependencies	Remove Unused Footage	未使用素材を削除	MENU_PDF
menu	File > Dependencies	Reduce Project	プロジェクトを選択コンポのみに縮小	MENU_PDF
menu	File > Dependencies	Find Missing Effects	見つからないエフェクトを検索	MENU_PDF
menu	File > Dependencies	Find Missing Fonts	見つからないフォントを検索	MENU_PDF
menu	File > Dependencies	Find Missing Footage	見つからない素材を検索	MENU_PDF
menu	File	Watch Folder…	監視フォルダを設定	MENU_PDF
menu	File > Scripts	Run Script File…	スクリプトファイルを実行	MENU_PDF
menu	File > Scripts	Open Script Editor	スクリプトエディタを開く	MENU_PDF
menu	File > Create proxy	Still…	静止画プロキシを作成	MENU_PDF
menu	File > Create proxy	Movie…	動画プロキシを作成	MENU_PDF
menu	File > Set Proxy	File…	プロキシファイルを指定	MENU_PDF
menu	File > Interpret Footage	Main…	メイン解釈設定を開く	MENU_PDF
menu	File > Interpret Footage	Proxy…	プロキシ解釈設定を開く	MENU_PDF
menu	File > Interpret Footage	Remember Interpretation	解釈設定を記憶	MENU_PDF
menu	File > Interpret Footage	Apply Interpretation	記憶した解釈設定を適用	MENU_PDF
menu	File > Replace Footage	File…	素材をファイルで置換	MENU_PDF
menu	File > Replace Footage	With Layered Comp	レイヤー化コンポで置換	MENU_PDF
menu	File > Replace Footage	Placeholder…	プレースホルダーで置換	MENU_PDF
menu	File > Replace Footage	Solid…	単色で置換	MENU_PDF
menu	File	Reload Footage	素材を再読み込み	MENU_PDF
menu	File	Reveal in Finder	Finderで表示	MENU_PDF
menu	File	Reveal in Bridge	Bridgeで表示	MENU_PDF
menu	File	Project Settings…	プロジェクト設定	MENU_PDF
menu	Edit	Undo	元に戻す	MENU_PDF
menu	Edit	Redo	やり直す	MENU_PDF
menu	Edit	History	操作履歴	MENU_PDF
menu	Edit	Cut	切り取り	MENU_PDF
menu	Edit	Copy	コピー	MENU_PDF
menu	Edit	Copy with Property Links	プロパティリンク付きコピー	MENU_PDF
menu	Edit	Paste	貼り付け	MENU_PDF
menu	Edit	Clear	削除	MENU_PDF
menu	Edit	Duplicate	複製	MENU_PDF
menu	Edit	Split Layer	レイヤーを分割	MENU_PDF
menu	Edit	Lift Work Area	ワークエリアをリフト	MENU_PDF
menu	Edit	Extract Work Area	ワークエリアを抽出	MENU_PDF
menu	Edit	Select All	すべて選択	MENU_PDF
menu	Edit	Deselect All	選択解除	MENU_PDF
menu	Edit > Label	Select Label Group	同ラベルをまとめて選択	MENU_PDF
menu	Edit > Label	None	ラベルなし	MENU_PDF
menu	Edit > Label	Red	赤ラベル	MENU_PDF
menu	Edit > Label	Yellow	黄ラベル	MENU_PDF
menu	Edit > Label	Aqua	水色ラベル	MENU_PDF
menu	Edit > Label	Pink	ピンクラベル	MENU_PDF
menu	Edit > Label	Lavender	ラベンダーラベル	MENU_PDF
menu	Edit > Label	Peach	ピーチラベル	MENU_PDF
menu	Edit > Label	Sea Foam	シーフォームラベル	MENU_PDF
menu	Edit > Label	Blue	青ラベル	MENU_PDF
menu	Edit > Label	Green	緑ラベル	MENU_PDF
menu	Edit > Label	Purple	紫ラベル	MENU_PDF
menu	Edit > Label	Orange	オレンジラベル	MENU_PDF
menu	Edit > Label	Brown	茶ラベル	MENU_PDF
menu	Edit > Label	Fuchsia	フクシアラベル	MENU_PDF
menu	Edit > Label	Cyan	シアンラベル	MENU_PDF
menu	Edit > Label	Sandstone	サンドストーンラベル	MENU_PDF
menu	Edit > Label	Dark Green	ダークグリーンラベル	MENU_PDF
menu	Edit > Purge	All Memory & Disk Cache…	メモリとディスクキャッシュを全消去	MENU_PDF
menu	Edit > Purge	All Memory	全メモリを消去	MENU_PDF
menu	Edit > Purge	Undo	アンドゥ履歴を消去	MENU_PDF
menu	Edit > Purge	Image Cache Memory	画像キャッシュメモリを消去	MENU_PDF
menu	Edit > Purge	Snapshot	スナップショットを消去	MENU_PDF
menu	Edit	Edit Original…	元アプリで編集	MENU_PDF
menu	Edit	Edit in Adobe Audition	Auditionで編集	MENU_PDF
menu	Edit > Templates	Render Settings…	レンダー設定テンプレート	MENU_PDF
menu	Edit > Templates	Output Module…	出力モジュールテンプレート	MENU_PDF
menu	Edit	Paste mocha mask	mochaマスクを貼り付け	MENU_PDF
menu	Composition	New Composition…	新規コンポジション	MENU_PDF
menu	Composition	Composition Settings…	コンポジション設定	MENU_PDF
menu	Composition	Set Poster Time	ポスターフレーム時刻を設定	MENU_PDF
menu	Composition	Trim Comp to Work Area	ワークエリアにコンポをトリム	MENU_PDF
menu	Composition	Crop Comp to Region of Interest	関心領域にコンポをクロップ	MENU_PDF
menu	Composition	Add to Adobe Media Encoder Queue…	AMEキューに追加	MENU_PDF
menu	Composition	Add to Render Queue	レンダーキューに追加	MENU_PDF
menu	Composition	Add Output Module	出力モジュールを追加	MENU_PDF
menu	Composition	Cache Work Area in Background	バックグラウンドでワークエリアをキャッシュ	MENU_PDF
menu	Composition	Cancel Caching Work Area in Background	バックグラウンドキャッシュを中止	MENU_PDF
menu	Composition > Preview	RAM Preview	RAMプレビュー	MENU_PDF
menu	Composition > Preview	Audio	オーディオプレビュー	MENU_PDF
menu	Composition > Preview	Audio Preview (Here Forward)	現在時刻以降のオーディオプレビュー	MENU_PDF
menu	Composition > Preview	Audio Preview (Work Area)	ワークエリアのオーディオプレビュー	MENU_PDF
menu	Composition > Save Frame As	File…	フレームをファイル保存	MENU_PDF
menu	Composition > Save Frame As	Photoshop Layers…	フレームをPSDレイヤーとして保存	MENU_PDF
menu	Composition	Pre-render…	プリレンダー	MENU_PDF
menu	Composition	Save RAM Preview…	RAMプレビューを保存	MENU_PDF
menu	Composition	Composition Flowchart	コンポジションのフローチャート表示	MENU_PDF
menu	Composition	Composition Mini-Flowchart	ミニフローチャート表示	MENU_PDF
menu	Layer > New	Text	新規テキストレイヤー	MENU_PDF
menu	Layer > New	Solid…	新規単色レイヤー	MENU_PDF
menu	Layer > New	Light…	新規ライトレイヤー	MENU_PDF
menu	Layer > New	Camera…	新規カメラレイヤー	MENU_PDF
menu	Layer > New	Null Object	新規ヌルオブジェクト	MENU_PDF
menu	Layer > New	Shape Layer	新規シェイプレイヤー	MENU_PDF
menu	Layer > New	Adjustment Layer	新規調整レイヤー	MENU_PDF
menu	Layer > New	Adobe Photoshop File…	新規Photoshopレイヤー	MENU_PDF
menu	Layer > New	MAXON CINEMA 4D File…	新規C4Dレイヤー	MENU_PDF
menu	Layer	Layer Settings…	レイヤー設定	MENU_PDF
menu	Layer	Open Layer	レイヤーパネルを開く	MENU_PDF
menu	Layer	Open Layer Source	ソースを開く	MENU_PDF
menu	Layer	Reveal in Finder	Finderで表示	MENU_PDF
menu	Layer > Mask	New Mask	新規マスク	MENU_PDF
menu	Layer > Mask	Mask Shape…	マスク形状	MENU_PDF
menu	Layer > Mask	Mask Feather…	マスクフェザー	MENU_PDF
menu	Layer > Mask	Mask Opacity…	マスク不透明度	MENU_PDF
menu	Layer > Mask	Mask Expansion…	マスク拡張	MENU_PDF
menu	Layer > Mask	Reset Mask	マスクをリセット	MENU_PDF
menu	Layer > Mask	Remove Mask	マスクを削除	MENU_PDF
menu	Layer > Mask	Remove All Masks	全マスクを削除	MENU_PDF
menu	Layer > Mask > Mode	None	マスクモード:なし	MENU_PDF
menu	Layer > Mask > Mode	Add	マスクモード:加算	MENU_PDF
menu	Layer > Mask > Mode	Subtract	マスクモード:減算	MENU_PDF
menu	Layer > Mask > Mode	Intersect	マスクモード:交差	MENU_PDF
menu	Layer > Mask > Mode	Lighten	マスクモード:比較(明)	MENU_PDF
menu	Layer > Mask > Mode	Darken	マスクモード:比較(暗)	MENU_PDF
menu	Layer > Mask > Mode	Difference	マスクモード:差	MENU_PDF
menu	Layer > Mask	Inverted	マスクを反転	MENU_PDF
menu	Layer > Mask	Locked	マスクをロック	MENU_PDF
menu	Layer > Mask > Motion Blur	Same As Layer	レイヤーに準拠	MENU_PDF
menu	Layer > Mask > Motion Blur	On	モーションブラーON	MENU_PDF
menu	Layer > Mask > Motion Blur	Off	モーションブラーOFF	MENU_PDF
menu	Layer > Mask > Feather Falloff	Smooth	フェザー減衰:滑らか	MENU_PDF
menu	Layer > Mask > Feather Falloff	Linear	フェザー減衰:線形	MENU_PDF
menu	Layer > Mask	Unlock All Masks	全マスクのロック解除	MENU_PDF
menu	Layer > Mask	Lock Other Masks	他マスクをロック	MENU_PDF
menu	Layer > Mask	Hide Locked Masks	ロック済マスクを隠す	MENU_PDF
menu	Layer > Mask and Shape Path	RotoBezier	ロトベジエ	MENU_PDF
menu	Layer > Mask and Shape Path	Closed	パスを閉じる	MENU_PDF
menu	Layer > Mask and Shape Path	Set First Vertex	開始頂点を設定	MENU_PDF
menu	Layer > Mask and Shape Path	Free Transform Points	ポイントを自由変形	MENU_PDF
menu	Layer > Quality	Best	画質:最高	MENU_PDF
menu	Layer > Quality	Draft	画質:下書き	MENU_PDF
menu	Layer > Quality	Wireframe	画質:ワイヤーフレーム	MENU_PDF
menu	Layer > Quality	Bilinear	サンプリング:バイリニア	MENU_PDF
menu	Layer > Quality	Bicubic	サンプリング:バイキュービック	MENU_PDF
menu	Layer > Switches	Hide Other Video	他のビデオを隠す	MENU_PDF
menu	Layer > Switches	Show All Video	全ビデオを表示	MENU_PDF
menu	Layer > Switches	Unlock All Layers	全レイヤーのロック解除	MENU_PDF
menu	Layer > Switches	Shy	シャイ切替	MENU_PDF
menu	Layer > Switches	Lock	ロック切替	MENU_PDF
menu	Layer > Switches	Audio	オーディオ切替	MENU_PDF
menu	Layer > Switches	Video	ビデオ表示切替	MENU_PDF
menu	Layer > Switches	Solo	ソロ切替	MENU_PDF
menu	Layer > Switches	Effect	エフェクト有効切替	MENU_PDF
menu	Layer > Switches	Collapse	コラップス切替	MENU_PDF
menu	Layer > Switches	Motion Blur	モーションブラー切替	MENU_PDF
menu	Layer > Switches	Adjustment Layer	調整レイヤー切替	MENU_PDF
menu	Layer > Transform	Reset	トランスフォームをリセット	MENU_PDF
menu	Layer > Transform	Anchor Point…	アンカーポイント	MENU_PDF
menu	Layer > Transform	Position…	位置	MENU_PDF
menu	Layer > Transform	Scale…	スケール	MENU_PDF
menu	Layer > Transform	Orientation…	方向	MENU_PDF
menu	Layer > Transform	Rotation…	回転	MENU_PDF
menu	Layer > Transform	Opacity…	不透明度	MENU_PDF
menu	Layer > Transform	Flip Horizontal	水平反転	MENU_PDF
menu	Layer > Transform	Flip Vertical	垂直反転	MENU_PDF
menu	Layer > Transform	Center in View	ビュー中央に配置	MENU_PDF
menu	Layer > Transform	Center Anchor Point in Layer Content	アンカーポイントをコンテンツ中央へ	MENU_PDF
menu	Layer > Transform	Fit to Comp	コンポにフィット	MENU_PDF
menu	Layer > Transform	Fit to Comp Width	コンポ幅にフィット	MENU_PDF
menu	Layer > Transform	Fit to Comp Height	コンポ高さにフィット	MENU_PDF
menu	Layer > Transform	Auto-Orient…	自動方向	MENU_PDF
menu	Layer > Time	Enable Time Remapping	タイムリマップを有効化	MENU_PDF
menu	Layer > Time	Time-Reverse Layer	レイヤーを逆再生	MENU_PDF
menu	Layer > Time	Time Stretch…	タイムストレッチ	MENU_PDF
menu	Layer > Time	Freeze Frame	フレームを静止	MENU_PDF
menu	Layer > Frame Blending	Off	フレームブレンドOFF	MENU_PDF
menu	Layer > Frame Blending	Frame Mix	フレームミックス	MENU_PDF
menu	Layer > Frame Blending	Pixel Motion	ピクセルモーション	MENU_PDF
menu	Layer	3D Layer	3Dレイヤー化	MENU_PDF
menu	Layer	Guide Layer	ガイドレイヤー化	MENU_PDF
menu	Layer	Environment Layer	環境レイヤー化	MENU_PDF
menu	Layer	Add Marker	マーカーを追加	MENU_PDF
menu	Layer	Preserve Transparency	透明度を保持	MENU_PDF
menu	Layer > Blending Mode	Normal	通常	MENU_PDF
menu	Layer > Blending Mode	Dissolve	ディゾルブ	MENU_PDF
menu	Layer > Blending Mode	Dancing Dissolve	ダンシングディゾルブ	MENU_PDF
menu	Layer > Blending Mode	Multiply	乗算	MENU_PDF
menu	Layer > Blending Mode	Screen	スクリーン	MENU_PDF
menu	Layer > Blending Mode	Overlay	オーバーレイ	MENU_PDF
menu	Layer > Blending Mode	Soft Light	ソフトライト	MENU_PDF
menu	Layer > Blending Mode	Hard Light	ハードライト	MENU_PDF
menu	Layer > Blending Mode	Linear Light	リニアライト	MENU_PDF
menu	Layer > Blending Mode	Vivid Light	ビビッドライト	MENU_PDF
menu	Layer > Blending Mode	Pin Light	ピンライト	MENU_PDF
menu	Layer > Blending Mode	Hard Mix	ハードミックス	MENU_PDF
menu	Layer > Blending Mode	Darken	比較(暗)	MENU_PDF
menu	Layer > Blending Mode	Multiply	乗算	MENU_PDF
menu	Layer > Blending Mode	Color Burn	焼き込みカラー	MENU_PDF
menu	Layer > Blending Mode	Classic Color Burn	焼き込みカラー(クラシック)	MENU_PDF
menu	Layer > Blending Mode	Linear Burn	焼き込み(リニア)	MENU_PDF
menu	Layer > Blending Mode	Darker Color	カラー比較(暗)	MENU_PDF
menu	Layer > Blending Mode	Lighten	比較(明)	MENU_PDF
menu	Layer > Blending Mode	Color Dodge	覆い焼きカラー	MENU_PDF
menu	Layer > Blending Mode	Classic Color Dodge	覆い焼きカラー(クラシック)	MENU_PDF
menu	Layer > Blending Mode	Linear Dodge	覆い焼き(リニア)	MENU_PDF
menu	Layer > Blending Mode	Lighter Color	カラー比較(明)	MENU_PDF
menu	Layer > Blending Mode	Add	加算	MENU_PDF
menu	Layer > Blending Mode	Difference	差	MENU_PDF
menu	Layer > Blending Mode	Classic Difference	差(クラシック)	MENU_PDF
menu	Layer > Blending Mode	Exclusion	除外	MENU_PDF
menu	Layer > Blending Mode	Subtract	減算	MENU_PDF
menu	Layer > Blending Mode	Divide	除算	MENU_PDF
menu	Layer > Blending Mode	Hue	色相	MENU_PDF
menu	Layer > Blending Mode	Saturation	彩度	MENU_PDF
menu	Layer > Blending Mode	Color	カラー	MENU_PDF
menu	Layer > Blending Mode	Luminosity	輝度	MENU_PDF
menu	Layer > Blending Mode	Stencil Alpha	ステンシルアルファ	MENU_PDF
menu	Layer > Blending Mode	Stencil Luma	ステンシル輝度	MENU_PDF
menu	Layer > Blending Mode	Silhouette Alpha	シルエットアルファ	MENU_PDF
menu	Layer > Blending Mode	Silhouette Luma	シルエット輝度	MENU_PDF
menu	Layer > Blending Mode	Alpha Add	アルファ加算	MENU_PDF
menu	Layer > Blending Mode	Luminescent Premul	光有りプリマル	MENU_PDF
menu	Layer	Next Blending Mode	次のブレンドモードへ	MENU_PDF
menu	Layer > Track Matte	No Track Matte	トラックマットなし	MENU_PDF
menu	Layer > Track Matte	Alpha Matte	アルファマット	MENU_PDF
menu	Layer > Track Matte	Alpha Inverted Matte	反転アルファマット	MENU_PDF
menu	Layer > Track Matte	Luma Matte	輝度マット	MENU_PDF
menu	Layer > Track Matte	Luma Inverted Matte	反転輝度マット	MENU_PDF
menu	Layer > Layer Styles	Show All	全レイヤースタイルを表示	MENU_PDF
menu	Layer > Layer Styles	Remove All	全レイヤースタイルを削除	MENU_PDF
menu	Layer > Layer Styles	Drop Shadow	ドロップシャドウ	MENU_PDF
menu	Layer > Layer Styles	Inner Shadow	シャドウ(内側)	MENU_PDF
menu	Layer > Layer Styles	Outer Glow	光彩(外側)	MENU_PDF
menu	Layer > Layer Styles	Inner Glow	光彩(内側)	MENU_PDF
menu	Layer > Layer Styles	Bevel and Emboss	ベベルとエンボス	MENU_PDF
menu	Layer > Layer Styles	Satin	サテン	MENU_PDF
menu	Layer > Layer Styles	Color Overlay	カラーオーバーレイ	MENU_PDF
menu	Layer > Layer Styles	Gradient Overlay	グラデーションオーバーレイ	MENU_PDF
menu	Layer > Layer Styles	Stroke	境界線	MENU_PDF
menu	Layer	Group Shapes	シェイプをグループ化	MENU_PDF
menu	Layer	Ungroup Shapes	シェイプのグループ解除	MENU_PDF
menu	Layer > Arrange	Bring Layer to Front	最前面へ	MENU_PDF
menu	Layer > Arrange	Bring Layer Forward	前面へ	MENU_PDF
menu	Layer > Arrange	Send Layer Backward	背面へ	MENU_PDF
menu	Layer > Arrange	Send Layer to Back	最背面へ	MENU_PDF
menu	Layer	Convert to Editable Text	編集可能なテキストに変換	MENU_PDF
menu	Layer	Create Shapes from Text	テキストからシェイプ作成	MENU_PDF
menu	Layer	Create Masks from Text	テキストからマスク作成	MENU_PDF
menu	Layer	Create Shapes from Vector Layer	ベクターレイヤーからシェイプ作成	MENU_PDF
menu	Layer > Camera	Create Stereo 3D Rig	ステレオ3Dリグ作成	MENU_PDF
menu	Layer > Camera	Create Orbit Null	オービットヌル作成	MENU_PDF
menu	Layer > Camera	Link Focus Distance to Point of Interest	注視点にフォーカス距離をリンク	MENU_PDF
menu	Layer > Camera	Link Focus Distance to Layer	レイヤーにフォーカス距離をリンク	MENU_PDF
menu	Layer > Camera	Set Focus Distance to Layer	レイヤーへフォーカス距離を設定	MENU_PDF
menu	Layer	Auto-trace…	オートトレース	MENU_PDF
menu	Layer	Pre-compose…	プリコンポーズ	MENU_PDF
menu	Effect	Effects Controls	エフェクトコントロールを表示	MENU_PDF
menu	Effect	Last Effect	直前のエフェクトを再適用	MENU_PDF
menu	Effect	Remove All	全エフェクトを削除	MENU_PDF
menu	Animation	Save Animation Preset…	アニメーションプリセット保存	MENU_PDF
menu	Animation	Apply Animation Preset…	アニメーションプリセット適用	MENU_PDF
menu	Animation	Recent Animation Presets	最近使ったプリセット	MENU_PDF
menu	Animation	Browse Presets…	プリセットを参照(Bridge)	MENU_PDF
menu	Animation	Add Keyframe	キーフレームを追加	MENU_PDF
menu	Animation	Toggle Hold Keyframe	ホールドキーフレーム切替	MENU_PDF
menu	Animation	Keyframe Interpolation…	キーフレーム補間法	MENU_PDF
menu	Animation	Keyframe Velocity…	キーフレーム速度	MENU_PDF
menu	Animation > Keyframe Assistant	Convert Audio to Keyframes	オーディオをキーフレームに変換	MENU_PDF
menu	Animation > Keyframe Assistant	Convert Expression to Keyframes	エクスプレッションをキーフレームに変換	MENU_PDF
menu	Animation > Keyframe Assistant	Easy Ease	イージーイーズ	MENU_PDF
menu	Animation > Keyframe Assistant	Easy Ease In	イージーイーズイン	MENU_PDF
menu	Animation > Keyframe Assistant	Easy Ease Out	イージーイーズアウト	MENU_PDF
menu	Animation > Keyframe Assistant	Exponential Scale	指数スケール	MENU_PDF
menu	Animation > Keyframe Assistant	RPF Camera Import	RPFカメラ読み込み	MENU_PDF
menu	Animation > Keyframe Assistant	Sequence Layers…	レイヤーをシーケンス化	MENU_PDF
menu	Animation > Keyframe Assistant	Time-Reverse Keyframes	キーフレームを時間反転	MENU_PDF
menu	Animation > Animate Text	Enable Per-character 3D	文字ごとの3Dを有効化	MENU_PDF
menu	Animation > Animate Text	Anchor Point	アンカーポイント	MENU_PDF
menu	Animation > Animate Text	Position	位置	MENU_PDF
menu	Animation > Animate Text	Scale	スケール	MENU_PDF
menu	Animation > Animate Text	Skew	スキュー	MENU_PDF
menu	Animation > Animate Text	Rotation	回転	MENU_PDF
menu	Animation > Animate Text	Opacity	不透明度	MENU_PDF
menu	Animation > Animate Text	All Transform Properties	全トランスフォームプロパティ	MENU_PDF
menu	Animation > Animate Text	Fill Color	塗り色	MENU_PDF
menu	Animation > Animate Text	Stroke Color	線色	MENU_PDF
menu	Animation > Animate Text	Stroke Width	線幅	MENU_PDF
menu	Animation > Animate Text	Tracking	トラッキング(字送り)	MENU_PDF
menu	Animation > Animate Text	Line Anchor	行のアンカー	MENU_PDF
menu	Animation > Animate Text	Line Spacing	行間	MENU_PDF
menu	Animation > Animate Text	Character Offset	文字オフセット	MENU_PDF
menu	Animation > Animate Text	Character Value	文字値	MENU_PDF
menu	Animation > Animate Text	Blur	ぼかし	MENU_PDF
menu	Animation > Add Text Selector	Range	範囲セレクタ	MENU_PDF
menu	Animation > Add Text Selector	Wiggly	ウィグリーセレクタ	MENU_PDF
menu	Animation > Add Text Selector	Expression	エクスプレッションセレクタ	MENU_PDF
menu	Animation	Remove All Text Animators	全テキストアニメーターを削除	MENU_PDF
menu	Animation	Add Expression	エクスプレッションを追加	MENU_PDF
menu	Animation	Separate Dimensions	次元を分割	MENU_PDF
menu	Animation	Track Camera	カメラトラッキング	MENU_PDF
menu	Animation	Track in mocha AE	mocha AEでトラッキング	MENU_PDF
menu	Animation	Warp Stabilizer VFX	ワープスタビライザーVFX	MENU_PDF
menu	Animation	Track Motion	モーショントラッキング	MENU_PDF
menu	Animation	Track Mask	マスクトラッキング	MENU_PDF
menu	Animation	Track this Property	このプロパティをトラック	MENU_PDF
menu	Animation	Reveal Properties with Keyframes	キーフレーム付きプロパティを表示	MENU_PDF
menu	Animation	Reveal Properties with Animation	アニメーション付きプロパティを表示	MENU_PDF
menu	Animation	Reveal All Modified Properties	変更済み全プロパティを表示	MENU_PDF
menu	View	New Viewer	新規ビューアー	MENU_PDF
menu	View	Zoom In	ズームイン	MENU_PDF
menu	View	Zoom Out	ズームアウト	MENU_PDF
menu	View > Resolution	Full	解像度:フル	MENU_PDF
menu	View > Resolution	Half	解像度:1/2	MENU_PDF
menu	View > Resolution	Third	解像度:1/3	MENU_PDF
menu	View > Resolution	Quarter	解像度:1/4	MENU_PDF
menu	View > Resolution	Custom…	解像度:カスタム	MENU_PDF
menu	View	Use Display Color Management	ディスプレイカラーマネジメントを使用	MENU_PDF
menu	View > Simulate Output	No Output Simulation	出力シミュレーションなし	MENU_PDF
menu	View > Simulate Output	HDTV (Rec. 709)	HDTV Rec.709でシミュレート	MENU_PDF
menu	View > Simulate Output	SDTV NTSC	SDTV NTSCでシミュレート	MENU_PDF
menu	View > Simulate Output	SDTV PAL	SDTV PALでシミュレート	MENU_PDF
menu	View > Simulate Output	Legacy Macintosh RGB (Gamma 1.8)	旧Mac RGBでシミュレート	MENU_PDF
menu	View > Simulate Output	Internet Standard RGB (sRGB)	sRGBでシミュレート	MENU_PDF
menu	View > Simulate Output	Custom…	カスタムでシミュレート	MENU_PDF
menu	View	Show Rulers	定規を表示	MENU_PDF
menu	View	Show Guides	ガイドを表示	MENU_PDF
menu	View	Snap to Guides	ガイドにスナップ	MENU_PDF
menu	View	Lock Guides	ガイドをロック	MENU_PDF
menu	View	Clear Guides	ガイドを消去	MENU_PDF
menu	View	Show Grid	グリッドを表示	MENU_PDF
menu	View	Snap to Grid	グリッドにスナップ	MENU_PDF
menu	View	View Options…	表示オプション	MENU_PDF
menu	View	Show Layer Controls	レイヤーコントロールを表示	MENU_PDF
menu	View	Reset 3D View	3Dビューをリセット	MENU_PDF
menu	View > Switch 3D View	Active Camera	アクティブカメラ視点	MENU_PDF
menu	View > Switch 3D View	Front	正面視点	MENU_PDF
menu	View > Switch 3D View	Left	左視点	MENU_PDF
menu	View > Switch 3D View	Top	上視点	MENU_PDF
menu	View > Switch 3D View	Back	背面視点	MENU_PDF
menu	View > Switch 3D View	Right	右視点	MENU_PDF
menu	View > Switch 3D View	Bottom	下視点	MENU_PDF
menu	View > Switch 3D View	Custom View 1	カスタム視点1	MENU_PDF
menu	View > Switch 3D View	Custom View 2	カスタム視点2	MENU_PDF
menu	View > Switch 3D View	Custom View 3	カスタム視点3	MENU_PDF
menu	View	Switch to Last 3D View	直前の3Dビューへ	MENU_PDF
menu	View	Look at Selected Layers	選択レイヤーを見る	MENU_PDF
menu	View	Look at All Layers	全レイヤーを見る	MENU_PDF
menu	View	Go to Time…	指定時刻へ移動	MENU_PDF
menu	Window > Workspace	All Panels	全パネル表示ワークスペース	MENU_PDF
menu	Window > Workspace	Animation	アニメーション用ワークスペース	MENU_PDF
menu	Window > Workspace	Effects	エフェクト用ワークスペース	MENU_PDF
menu	Window > Workspace	Minimal	最小構成ワークスペース	MENU_PDF
menu	Window > Workspace	Motion Tracking	モーショントラッキング用ワークスペース	MENU_PDF
menu	Window > Workspace	Paint	ペイント用ワークスペース	MENU_PDF
menu	Window > Workspace	Standard	標準ワークスペース	MENU_PDF
menu	Window > Workspace	Text	テキスト用ワークスペース	MENU_PDF
menu	Window > Workspace	Undocked Panels	パネル切り離しワークスペース	MENU_PDF
menu	Window > Workspace	New Workspace…	新規ワークスペース	MENU_PDF
menu	Window > Workspace	Delete Workspace…	ワークスペースを削除	MENU_PDF
menu	Window	Go to Time…	該当なし(Viewメニューと重複確認用)	MENU_PDF
menu	Composition	Take Snapshot	該当なし(参考: PDFにはComposition関連のSnapshotは明記なし、Edit>Purge>Snapshotのみ確認)	MENU_PDF
menu	Help	After Effects Help…	ヘルプを開く	MENU_PDF
menu	Help	Scripting Help…	スクリプトヘルプ	MENU_PDF
menu	Help	Expression Reference…	エクスプレッションリファレンス	MENU_PDF
menu	Help	Effect Reference…	エフェクトリファレンス	MENU_PDF
menu	Help	Animation Presets…	アニメーションプリセットヘルプ	MENU_PDF
menu	Help	Keyboard Shortcuts…	キーボードショートカット一覧	MENU_PDF
menu	Help	Welcome Screen…	ようこそ画面	MENU_PDF
menu	Help	Adobe Product Improvement Program…	製品改善プログラム	MENU_PDF
menu	Help	Adobe Crash Reporter…	クラッシュレポーター	MENU_PDF
menu	Help	Enable Logging	ログ記録を有効化	MENU_PDF
menu	Help	Reveal Logging File	ログファイルを表示	MENU_PDF
menu	Help	After Effects Support Center…	サポートセンター	MENU_PDF
menu	Help	Online Users Forums…	オンラインフォーラム	MENU_PDF
menu	Help	Send Feedback…	フィードバック送信	MENU_PDF
menu	Help	Complete/Update Adobe ID Profile…	Adobe IDプロフィール更新	MENU_PDF
menu	Help	Updates…	アップデート確認	MENU_PDF

## 2. shortcut (ショートカット表、Timeline系はR6既採集のため除外)
shortcut	General	Select all	すべて選択	SHORTCUT_URL
shortcut	General	Deselect all	選択解除	SHORTCUT_URL
shortcut	General	Rename selected layer, composition, folder, effect, group, or mask	選択項目の名前変更	SHORTCUT_URL
shortcut	General	Open selected layer, composition, or footage item	選択項目を開く	SHORTCUT_URL
shortcut	General	Move selected items down/up in stacking order	選択項目を重ね順で上下に移動	SHORTCUT_URL
shortcut	General	Move selected items to bottom/top of stacking order	選択項目を最上/最下へ移動	SHORTCUT_URL
shortcut	General	Extend selection to next item in Project/Render Queue/Effect Controls panel	次項目まで選択範囲を拡張	SHORTCUT_URL
shortcut	General	Extend selection to previous item	前項目まで選択範囲を拡張	SHORTCUT_URL
shortcut	General	Duplicate selected items	選択項目を複製	SHORTCUT_URL
shortcut	General	Quit	終了	SHORTCUT_URL
shortcut	General	Undo	元に戻す	SHORTCUT_URL
shortcut	General	Redo	やり直す	SHORTCUT_URL
shortcut	General	Purge All Memory	全メモリをパージ	SHORTCUT_URL
shortcut	General	Interrupt running script	実行中スクリプトを中断	SHORTCUT_URL
shortcut	General	Display filename in Info panel	Infoパネルにファイル名を表示	SHORTCUT_URL
shortcut	Keyboard Shortcut Editor	Open Visual Keyboard Shortcut Editor	ビジュアルショートカットエディタを開く	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Project panel	Projectパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Render Queue panel	Render Queueパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Tools panel	Toolsパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Info panel	Infoパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Preview panel	Previewパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Audio panel	Audioパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Effects & Presets panel	Effects & Presetsパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Character panel	Characterパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Paragraph panel	Paragraphパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Paint panel	Paintパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Brushes panel	Brushesパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open or close Effect Controls panel for selected layer	Effect Controlsパネル開閉	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Open Flowchart panel for project	Flowchartパネルを開く	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Switch to workspace	ワークスペース切替	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Close active viewer or panel	アクティブビューア/パネルを閉じる	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Close active panel or all viewers of type	同種の全ビューアを閉じる	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Split frame and create viewer with opposite locked/unlocked state	フレーム分割し反対ロック状態のビューアを作成	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Maximize or restore panel under pointer	ポインタ下パネルを最大化/復元	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Resize application window to fit screen	アプリウィンドウを画面に合わせてリサイズ	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Move application window to main monitor and resize	メインモニタへ移動しリサイズ	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Toggle between Composition and Timeline panels	Composition/Timelineパネル切替	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Cycle to previous or next item in active viewer	アクティブビューア内の前後項目を巡回	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Cycle to previous or next panel in active frame	アクティブフレーム内の前後パネルを巡回	SHORTCUT_URL
shortcut	Panels, Viewers, Workspaces, and Windows	Activate view in multi-view layout	マルチビューでビューを有効化	SHORTCUT_URL
shortcut	Tools	Cycle through tools	ツールを巡回	SHORTCUT_URL
shortcut	Tools	Activate Selection tool	選択ツール	SHORTCUT_URL
shortcut	Tools	Activate Hand tool	手のひらツール	SHORTCUT_URL
shortcut	Tools	Temporarily activate Hand tool	一時的に手のひらツール	SHORTCUT_URL
shortcut	Tools	Activate Zoom In tool	ズームインツール	SHORTCUT_URL
shortcut	Tools	Activate Zoom Out tool	ズームアウトツール	SHORTCUT_URL
shortcut	Tools	Activate Rotation tool	回転ツール	SHORTCUT_URL
shortcut	Tools	Activate Roto Brush tool	ロトブラシツール	SHORTCUT_URL
shortcut	Tools	Activate Refine Edge tool	エッジ調整ツール	SHORTCUT_URL
shortcut	Tools	Activate and cycle Camera tools	カメラツールを巡回	SHORTCUT_URL
shortcut	Tools	Activate Pan Behind tool	アンカーポイントツール	SHORTCUT_URL
shortcut	Tools	Activate and cycle mask and shape tools	マスク/シェイプツールを巡回	SHORTCUT_URL
shortcut	Tools	Activate and cycle Type tools	文字ツールを巡回	SHORTCUT_URL
shortcut	Tools	Activate and cycle Pen and Mask Feather tools	ペン/マスクフェザーツールを巡回	SHORTCUT_URL
shortcut	Tools	Temporarily activate Selection tool with pen selected	ペン選択中に一時的に選択ツール	SHORTCUT_URL
shortcut	Tools	Temporarily activate Pen tool with Selection selected	選択ツール中に一時的にペンツール	SHORTCUT_URL
shortcut	Tools	Activate and cycle Brush, Clone Stamp, and Eraser tools	ブラシ/コピースタンプ/消しゴムを巡回	SHORTCUT_URL
shortcut	Tools	Activate and cycle Puppet tools	パペットツールを巡回	SHORTCUT_URL
shortcut	Tools	Temporarily convert Selection to Shape Duplication tool	一時的にシェイプ複製ツールへ	SHORTCUT_URL
shortcut	Tools	Temporarily convert Selection to Direct Selection tool	一時的にダイレクト選択ツールへ	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to specific time	指定時刻へ移動	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to beginning or end of work area	ワークエリアの先頭/末尾へ	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to previous or next visible item in time ruler	タイムルーラーの前後の項目へ	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to beginning of composition, layer, or footage	コンポ/レイヤー/素材の先頭へ	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to end of composition, layer, or footage	コンポ/レイヤー/素材の末尾へ	SHORTCUT_URL
shortcut	Compositions and Work Area	Go forward 1 frame	1フレーム進む	SHORTCUT_URL
shortcut	Compositions and Work Area	Go forward 10 frames	10フレーム進む	SHORTCUT_URL
shortcut	Compositions and Work Area	Go backward 1 frame	1フレーム戻る	SHORTCUT_URL
shortcut	Compositions and Work Area	Go backward 10 frames	10フレーム戻る	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to layer In point	レイヤーIn点へ	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to layer Out point	レイヤーOut点へ	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to previous In or Out point	前のIn/Out点へ	SHORTCUT_URL
shortcut	Compositions and Work Area	Go to next In or Out point	次のIn/Out点へ	SHORTCUT_URL
shortcut	Compositions and Work Area	Scroll to current time in Timeline panel	現在時刻までスクロール	SHORTCUT_URL
shortcut	Preview	Start or stop preview	プレビュー開始/停止	SHORTCUT_URL
shortcut	Preview	Reset preview settings	プレビュー設定をリセット	SHORTCUT_URL
shortcut	Preview	Preview only audio from current time	現在時刻からオーディオのみプレビュー	SHORTCUT_URL
shortcut	Preview	Preview only audio in work area	ワークエリアのオーディオのみプレビュー	SHORTCUT_URL
shortcut	Preview	Manually preview video	ビデオを手動プレビュー	SHORTCUT_URL
shortcut	Preview	Manually preview audio	オーディオを手動プレビュー	SHORTCUT_URL
shortcut	Preview	Preview specified number of frames	指定フレーム数をプレビュー	SHORTCUT_URL
shortcut	Preview	Toggle Mercury Transmit video preview	Mercury Transmitプレビュー切替	SHORTCUT_URL
shortcut	Preview	Take snapshot	スナップショットを取得	SHORTCUT_URL
shortcut	Preview	Display snapshot in active viewer	スナップショットを表示	SHORTCUT_URL
shortcut	Preview	Purge snapshot	スナップショットをパージ	SHORTCUT_URL
shortcut	Preview	Fast Previews > Off	高速プレビュー:オフ	SHORTCUT_URL
shortcut	Preview	Fast Previews > Adaptive Resolution	高速プレビュー:適応解像度	SHORTCUT_URL
shortcut	Preview	Fast Previews > Draft	高速プレビュー:ドラフト	SHORTCUT_URL
shortcut	Preview	Fast Previews > Fast Draft	高速プレビュー:ファストドラフト	SHORTCUT_URL
shortcut	Preview	Fast Previews > Wireframe	高速プレビュー:ワイヤーフレーム	SHORTCUT_URL
shortcut	Display Color Management	Turn display color management on or off	ディスプレイカラーマネジメント切替	SHORTCUT_URL
shortcut	Display Color Management	Show red, green, blue, or alpha channel as grayscale	各チャンネルをグレースケール表示	SHORTCUT_URL
shortcut	Display Color Management	Show colorized red, green, or blue channel	各チャンネルを色付き表示	SHORTCUT_URL
shortcut	Display Color Management	Toggle showing straight RGB color	ストレートRGB表示切替	SHORTCUT_URL
shortcut	Display Color Management	Show alpha boundary in Layer panel	Layerパネルでアルファ境界表示	SHORTCUT_URL
shortcut	Display Color Management	Show alpha overlay in Layer panel	Layerパネルでアルファオーバーレイ表示	SHORTCUT_URL
shortcut	Display Color Management	Show Refine Edge X-ray	エッジ調整X線表示	SHORTCUT_URL
shortcut	Zooming and Panning	Center composition in panel	パネル内でコンポを中央に	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom in in Composition, Layer, or Footage panel	各パネルでズームイン	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom out in Composition, Layer, or Footage panel	各パネルでズームアウト	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom to 100%	100%表示	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom to fit	フィット表示	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom up to 100% to fit	100%上限でフィット表示	SHORTCUT_URL
shortcut	Zooming and Panning	Set resolution to Full, Half, or Custom	解像度をフル/半分/カスタムに設定	SHORTCUT_URL
shortcut	Zooming and Panning	Open View Options dialog for active Composition panel	表示オプションダイアログを開く	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom in time	時間軸をズームイン	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom out time	時間軸をズームアウト	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom Timeline to single-frame units	1フレーム単位までズーム	SHORTCUT_URL
shortcut	Zooming and Panning	Zoom out Timeline to show entire composition	コンポ全体表示までズームアウト	SHORTCUT_URL
shortcut	Viewer Display Options	Prevent rendering for previews	プレビュー用レンダリングを抑止	SHORTCUT_URL
shortcut	Viewer Display Options	Show or hide safe zones	セーフゾーン表示切替	SHORTCUT_URL
shortcut	Viewer Display Options	Show or hide grid	グリッド表示切替	SHORTCUT_URL
shortcut	Viewer Display Options	Show or hide proportional grid	比例グリッド表示切替	SHORTCUT_URL
shortcut	Viewer Display Options	Show or hide rulers	定規表示切替	SHORTCUT_URL
shortcut	Viewer Display Options	Show or hide guides	ガイド表示切替	SHORTCUT_URL
shortcut	Viewer Display Options	Turn snapping to grid on or off	グリッドスナップ切替	SHORTCUT_URL
shortcut	Viewer Display Options	Turn snapping to guides on or off	ガイドスナップ切替	SHORTCUT_URL
shortcut	Viewer Display Options	Lock or unlock guides	ガイドロック切替	SHORTCUT_URL
shortcut	Viewer Display Options	Show or hide layer controls	レイヤーコントロール表示切替	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Import one file or image sequence	1ファイル/連番読み込み	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Import multiple files or image sequences	複数ファイル読み込み	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Open movie in After Effects Footage panel	Footageパネルで動画を開く	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Add selected items to most recently activated composition	直近コンポへ選択項目を追加	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Replace selected source footage for selected layers	選択レイヤーのソースを置換	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Replace source for selected layer	選択レイヤーのソースを置換(ドラッグ)	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Delete footage item without warning	警告なしで素材を削除	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Open Interpret Footage dialog	素材の解釈ダイアログを開く	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Remember footage interpretation	素材の解釈を記憶	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Edit selected footage in associated application	関連アプリで素材を編集	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Replace selected footage item	選択素材を置換	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Reload selected footage items	選択素材を再読み込み	SHORTCUT_URL
shortcut	Importing and Replacing Footage	Set proxy for selected footage item	選択素材にプロキシを設定	SHORTCUT_URL
shortcut	Layers	New solid layer	新規単色レイヤー	SHORTCUT_URL
shortcut	Layers	New null layer	新規ヌルレイヤー	SHORTCUT_URL
shortcut	Layers	New adjustment layer	新規調整レイヤー	SHORTCUT_URL
shortcut	Layers	Select layer by number (1-999)	番号でレイヤー選択	SHORTCUT_URL
shortcut	Layers	Toggle selection of layer by number (1-999)	番号でレイヤー選択切替	SHORTCUT_URL
shortcut	Layers	Select next layer in stacking order	次のレイヤーを選択	SHORTCUT_URL
shortcut	Layers	Select previous layer in stacking order	前のレイヤーを選択	SHORTCUT_URL
shortcut	Layers	Extend selection to next layer	次レイヤーまで選択拡張	SHORTCUT_URL
shortcut	Layers	Extend selection to previous layer	前レイヤーまで選択拡張	SHORTCUT_URL
shortcut	Layers	Deselect all layers	全レイヤー選択解除	SHORTCUT_URL
shortcut	Layers	Scroll topmost selected layer to top of Timeline	選択最上位レイヤーを先頭へスクロール	SHORTCUT_URL
shortcut	Layers	Show or hide Parent column	Parent列表示切替	SHORTCUT_URL
shortcut	Layers	Show or hide Layer Switches and Modes columns	スイッチ/モード列表示切替	SHORTCUT_URL
shortcut	Layers	Set sampling method to Best/Bilinear	サンプリング:最高/バイリニア	SHORTCUT_URL
shortcut	Layers	Set sampling method to Best/Bicubic	サンプリング:最高/バイキュービック	SHORTCUT_URL
shortcut	Layers	Turn off all other solo switches	他の全ソロを解除	SHORTCUT_URL
shortcut	Layers	Turn Video (eyeball) switch on or off	ビデオ表示切替	SHORTCUT_URL
shortcut	Layers	Open settings dialog for selected layer	選択レイヤーの設定ダイアログを開く	SHORTCUT_URL
shortcut	Layers	Paste layers at current time	現在時刻にレイヤーを貼り付け	SHORTCUT_URL
shortcut	Layers	Split selected layers	選択レイヤーを分割	SHORTCUT_URL
shortcut	Layers	Precompose selected layers	選択レイヤーをプリコンポーズ	SHORTCUT_URL
shortcut	Layers	Open Effect Controls panel	Effect Controlsパネルを開く	SHORTCUT_URL
shortcut	Layers	Open layer in Layer panel	Layerパネルでレイヤーを開く	SHORTCUT_URL
shortcut	Layers	Open source in Footage panel	Footageパネルでソースを開く	SHORTCUT_URL
shortcut	Layers	Reverse selected layers in time	選択レイヤーを時間反転	SHORTCUT_URL
shortcut	Layers	Enable time remapping	タイムリマップを有効化	SHORTCUT_URL
shortcut	Layers	Move selected layers so In/Out point is at current time	In/Out点を現在時刻へ移動	SHORTCUT_URL
shortcut	Layers	Trim In or Out point to current time	現在時刻でIn/Outをトリム	SHORTCUT_URL
shortcut	Layers	Add or remove expression for property	プロパティにエクスプレッションを追加/削除	SHORTCUT_URL
shortcut	Layers	Add effect to selected layers	選択レイヤーにエフェクトを追加	SHORTCUT_URL
shortcut	Layers	Set In or Out point by time-stretching	タイムストレッチでIn/Outを設定	SHORTCUT_URL
shortcut	Layers	Move layers so In point at composition beginning	In点をコンポ先頭に	SHORTCUT_URL
shortcut	Layers	Move layers so Out point at composition end	Out点をコンポ末尾に	SHORTCUT_URL
shortcut	Layers	Lock selected layers	選択レイヤーをロック	SHORTCUT_URL
shortcut	Layers	Unlock all layers	全レイヤーのロック解除	SHORTCUT_URL
shortcut	Layers	Set Quality to Best, Draft, or Wireframe	画質を最高/下書き/ワイヤーフレームに設定	SHORTCUT_URL
shortcut	Layers	Cycle through blending modes	ブレンドモードを巡回	SHORTCUT_URL
shortcut	Layers	Find in Timeline panel	Timelineパネル内検索	SHORTCUT_URL
shortcut	Modifying Layer Properties	Modify property value by default increments	既定の増減幅でプロパティ変更	SHORTCUT_URL
shortcut	Modifying Layer Properties	Modify property value by 10x increments	10倍の増減幅でプロパティ変更	SHORTCUT_URL
shortcut	Modifying Layer Properties	Modify property value by 1/10 increments	1/10の増減幅でプロパティ変更	SHORTCUT_URL
shortcut	Modifying Layer Properties	Open Auto-Orientation dialog	自動方向ダイアログを開く	SHORTCUT_URL
shortcut	Modifying Layer Properties	Open Opacity dialog	不透明度ダイアログを開く	SHORTCUT_URL
shortcut	Modifying Layer Properties	Open Rotation dialog	回転ダイアログを開く	SHORTCUT_URL
shortcut	Modifying Layer Properties	Open Position dialog	位置ダイアログを開く	SHORTCUT_URL
shortcut	Modifying Layer Properties	Center selected layers in view	選択レイヤーをビュー中央に	SHORTCUT_URL
shortcut	Modifying Layer Properties	Center anchor point in visible content	アンカーポイントを表示内容の中央に	SHORTCUT_URL
shortcut	Modifying Layer Properties	Move selected layers 1 pixel	選択レイヤーを1px移動	SHORTCUT_URL
shortcut	Modifying Layer Properties	Move selected layers 10 pixels	選択レイヤーを10px移動	SHORTCUT_URL
shortcut	Modifying Layer Properties	Move selected layers 1 frame earlier or later	選択レイヤーを1フレーム前後移動	SHORTCUT_URL
shortcut	Modifying Layer Properties	Move selected layers 10 frames earlier or later	選択レイヤーを10フレーム前後移動	SHORTCUT_URL
shortcut	Modifying Layer Properties	Increase or decrease Rotation by 1°	回転を1度増減	SHORTCUT_URL
shortcut	Modifying Layer Properties	Increase or decrease Rotation by 10°	回転を10度増減	SHORTCUT_URL
shortcut	Modifying Layer Properties	Increase or decrease Opacity by 1%	不透明度を1%増減	SHORTCUT_URL
shortcut	Modifying Layer Properties	Increase or decrease Opacity by 10%	不透明度を10%増減	SHORTCUT_URL
shortcut	Modifying Layer Properties	Increase Scale by 1%	スケールを1%増加	SHORTCUT_URL
shortcut	Modifying Layer Properties	Decrease Scale by 1%	スケールを1%減少	SHORTCUT_URL
shortcut	Modifying Layer Properties	Increase Scale by 10%	スケールを10%増加	SHORTCUT_URL
shortcut	Modifying Layer Properties	Decrease Scale by 10%	スケールを10%減少	SHORTCUT_URL
shortcut	Modifying Layer Properties	Modify Rotation in 15° increments	回転を15度刻みで変更	SHORTCUT_URL
shortcut	Modifying Layer Properties	Modify Scale constrained to aspect ratio	縦横比を保ってスケール変更	SHORTCUT_URL
shortcut	Modifying Layer Properties	Reset Rotation to 0°	回転を0度にリセット	SHORTCUT_URL
shortcut	Modifying Layer Properties	Reset Scale to 100%	スケールを100%にリセット	SHORTCUT_URL
shortcut	Modifying Layer Properties	Scale and reposition layers to fit composition	コンポにフィットさせスケール・再配置	SHORTCUT_URL
shortcut	Modifying Layer Properties	Scale and reposition to fit composition width	コンポ幅にフィット	SHORTCUT_URL
shortcut	Modifying Layer Properties	Scale and reposition to fit composition height	コンポ高さにフィット	SHORTCUT_URL
shortcut	3D Views	Switch to 3D view 1 (Front)	3Dビュー1(正面)へ	SHORTCUT_URL
shortcut	3D Views	Switch to 3D view 2 (Custom View 1)	3Dビュー2(カスタム1)へ	SHORTCUT_URL
shortcut	3D Views	Switch to 3D view 3 (Active Camera)	3Dビュー3(アクティブカメラ)へ	SHORTCUT_URL
shortcut	3D Views	Return to previous view	直前のビューへ戻る	SHORTCUT_URL
shortcut	3D Views	New light	新規ライト	SHORTCUT_URL
shortcut	3D Views	Switch to Orbit camera control	オービットカメラ操作へ	SHORTCUT_URL
shortcut	3D Views	Switch to Pan camera control	パンカメラ操作へ	SHORTCUT_URL
shortcut	3D Views	Switch to Dolly camera control	ドリーカメラ操作へ	SHORTCUT_URL
shortcut	3D Views	New camera	新規カメラ	SHORTCUT_URL
shortcut	3D Views	Switch to Universal gizmo	ユニバーサルギズモへ	SHORTCUT_URL
shortcut	3D Views	Switch to Position gizmo	位置ギズモへ	SHORTCUT_URL
shortcut	3D Views	Switch to Scale gizmo	スケールギズモへ	SHORTCUT_URL
shortcut	3D Views	Switch to Rotation gizmo	回転ギズモへ	SHORTCUT_URL
shortcut	3D Views	Move camera to look at selected 3D layers	選択3Dレイヤーを見るようカメラ移動	SHORTCUT_URL
shortcut	3D Views	With camera tool, move to selected 3D layers	カメラツールで選択3Dレイヤーへ移動	SHORTCUT_URL
shortcut	3D Views	With camera tool, move to all 3D layers	カメラツールで全3Dレイヤーへ移動	SHORTCUT_URL
shortcut	3D Views	Turn Casts Shadows on or off	影を落とす切替	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Toggle between Graph Editor and layer bar modes	グラフエディタ/レイヤーバー切替	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Select all keyframes for property	プロパティの全キーフレームを選択	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Select all visible keyframes and properties	表示中の全キーフレーム・プロパティを選択	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Deselect all keyframes, properties, and groups	全キーフレーム・プロパティ・グループの選択解除	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Move keyframe 1 frame later or earlier	キーフレームを1フレーム前後移動	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Move keyframe 10 frames later or earlier	キーフレームを10フレーム前後移動	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Set interpolation for keyframes	キーフレーム補間法を設定	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Set keyframe interpolation to hold or Auto Bezier	補間をホールド/自動ベジェに設定	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Set interpolation to linear or Auto Bezier	補間をリニア/自動ベジェに設定	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Set interpolation to linear or hold	補間をリニア/ホールドに設定	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Easy ease selected keyframes	選択キーフレームにイージーイーズ	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Easy ease selected keyframes in	イージーイーズイン	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Easy ease selected keyframes out	イージーイーズアウト	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Set velocity for selected keyframes	選択キーフレームの速度を設定	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Add or remove keyframe at current time	現在時刻でキーフレームを追加/削除	SHORTCUT_URL
shortcut	Keyframes and Graph Editor	Reverse paste copied keyframes	コピーしたキーフレームを逆順貼り付け	SHORTCUT_URL
shortcut	Text	New text layer	新規テキストレイヤー	SHORTCUT_URL
shortcut	Text	Align horizontal text left, center, or right	水平テキストの左/中央/右揃え	SHORTCUT_URL
shortcut	Text	Align vertical text top, center, or bottom	垂直テキストの上/中央/下揃え	SHORTCUT_URL
shortcut	Text	Extend or reduce selection by one character in horizontal text	1文字ずつ選択範囲を拡大縮小(横書き)	SHORTCUT_URL
shortcut	Text	Extend or reduce selection by one word in horizontal text	1単語ずつ選択範囲を拡大縮小(横書き)	SHORTCUT_URL
shortcut	Text	Extend or reduce selection by one line in horizontal text	1行ずつ選択範囲を拡大縮小(横書き)	SHORTCUT_URL
shortcut	Text	Extend or reduce selection by one character in vertical text	1文字ずつ選択範囲を拡大縮小(縦書き)	SHORTCUT_URL
shortcut	Text	Extend or reduce selection one word in vertical text	1単語ずつ選択範囲を拡大縮小(縦書き)	SHORTCUT_URL
shortcut	Text	Select text to beginning or end of line	行頭/行末まで選択	SHORTCUT_URL
shortcut	Text	Move cursor to beginning or end of line	行頭/行末へカーソル移動	SHORTCUT_URL
shortcut	Text	Select all text on layer	レイヤー内全テキストを選択	SHORTCUT_URL
shortcut	Text	Select text to beginning or end of frame	フレーム先頭/末尾まで選択	SHORTCUT_URL
shortcut	Text	Select text from cursor to click point	カーソルからクリック位置まで選択	SHORTCUT_URL
shortcut	Text	Move cursor in horizontal text	横書きテキスト内でカーソル移動	SHORTCUT_URL
shortcut	Text	Move cursor in vertical text	縦書きテキスト内でカーソル移動	SHORTCUT_URL
shortcut	Text	Select word, line, paragraph, or entire text frame	単語/行/段落/全体を選択	SHORTCUT_URL
shortcut	Text	Turn All Caps on or off	すべて大文字切替	SHORTCUT_URL
shortcut	Text	Turn Small Caps on or off	スモールキャップ切替	SHORTCUT_URL
shortcut	Text	Turn Superscript on or off	上付き文字切替	SHORTCUT_URL
shortcut	Text	Turn Subscript on or off	下付き文字切替	SHORTCUT_URL
shortcut	Text	Set horizontal scale to 100%	水平比率を100%に	SHORTCUT_URL
shortcut	Text	Set vertical scale to 100%	垂直比率を100%に	SHORTCUT_URL
shortcut	Text	Auto leading for selected text	選択テキストを自動行送りに	SHORTCUT_URL
shortcut	Text	Reset tracking to 0	トラッキングを0にリセット	SHORTCUT_URL
shortcut	Text	Justify paragraph; left align last line	両端揃え(最終行左)	SHORTCUT_URL
shortcut	Text	Justify paragraph; right align last line	両端揃え(最終行右)	SHORTCUT_URL
shortcut	Text	Justify paragraph; force last line	強制両端揃え	SHORTCUT_URL
shortcut	Text	Decrease or increase font size by 2 units	フォントサイズを2単位増減	SHORTCUT_URL
shortcut	Text	Decrease or increase font size by 10 units	フォントサイズを10単位増減	SHORTCUT_URL
shortcut	Text	Increase or decrease leading by 2 units	行送りを2単位増減	SHORTCUT_URL
shortcut	Text	Increase or decrease leading by 10 units	行送りを10単位増減	SHORTCUT_URL
shortcut	Text	Decrease or increase baseline shift by 2 units	ベースラインシフトを2単位増減	SHORTCUT_URL
shortcut	Text	Decrease or increase baseline shift by 10 units	ベースラインシフトを10単位増減	SHORTCUT_URL
shortcut	Text	Decrease or increase kerning/tracking 20 units	カーニング/トラッキングを20単位増減	SHORTCUT_URL
shortcut	Text	Decrease or increase kerning/tracking 100 units	カーニング/トラッキングを100単位増減	SHORTCUT_URL
shortcut	Text	Toggle paragraph composer	段落コンポーザ切替	SHORTCUT_URL
shortcut	Masks	New mask	新規マスク	SHORTCUT_URL
shortcut	Masks	Select all points in mask	マスクの全ポイントを選択	SHORTCUT_URL
shortcut	Masks	Select next or previous mask	次/前のマスクを選択	SHORTCUT_URL
shortcut	Masks	Enter free-transform mask editing mode	自由変形マスク編集モードへ	SHORTCUT_URL
shortcut	Masks	Exit free-transform mask editing mode	自由変形マスク編集モードを終了	SHORTCUT_URL
shortcut	Masks	Scale around center point in Free Transform	中心点基準で拡大縮小	SHORTCUT_URL
shortcut	Masks	Move selected path points 1 pixel	選択パスポイントを1px移動	SHORTCUT_URL
shortcut	Masks	Move selected path points 10 pixels	選択パスポイントを10px移動	SHORTCUT_URL
shortcut	Masks	Toggle between smooth and corner points	スムーズ/コーナーポイント切替	SHORTCUT_URL
shortcut	Masks	Redraw Bezier handles	ベジェハンドルを再描画	SHORTCUT_URL
shortcut	Masks	Invert selected mask	選択マスクを反転	SHORTCUT_URL
shortcut	Masks	Open Mask Feather dialog	マスクフェザーダイアログを開く	SHORTCUT_URL
shortcut	Masks	Open Mask Shape dialog	マスク形状ダイアログを開く	SHORTCUT_URL
shortcut	Masks	Subtract mode	マスクモード:減算	SHORTCUT_URL
shortcut	Masks	Darken mode	マスクモード:比較(暗)	SHORTCUT_URL
shortcut	Masks	Difference mode	マスクモード:差	SHORTCUT_URL
shortcut	Masks	Add mode	マスクモード:加算	SHORTCUT_URL
shortcut	Masks	Intersect mode	マスクモード:交差	SHORTCUT_URL
shortcut	Masks	None	マスクモード:なし	SHORTCUT_URL
shortcut	Paint	Swap paint background and foreground colors	背景色と前景色を入替	SHORTCUT_URL
shortcut	Paint	Set foreground to black, background to white	前景を黒、背景を白に	SHORTCUT_URL
shortcut	Paint	Set foreground to color under pointer	ポインタ下の色を前景色に	SHORTCUT_URL
shortcut	Paint	Set foreground to average color under pointer	ポインタ周辺の平均色を前景色に	SHORTCUT_URL
shortcut	Paint	Set brush size	ブラシサイズを設定	SHORTCUT_URL
shortcut	Paint	Set brush hardness	ブラシ硬さを設定	SHORTCUT_URL
shortcut	Paint	Join current stroke to previous	現在のストロークを前と結合	SHORTCUT_URL
shortcut	Paint	Set Clone Stamp starting sample point	コピースタンプの開始点を設定	SHORTCUT_URL
shortcut	Paint	Momentarily activate Eraser with Last Stroke Only	直前ストロークのみ消しゴム化	SHORTCUT_URL
shortcut	Paint	Show and move Clone Stamp overlay	コピースタンプオーバーレイを表示・移動	SHORTCUT_URL
shortcut	Paint	Activate specific Clone Stamp preset	コピースタンプのプリセットを呼び出す	SHORTCUT_URL
shortcut	Paint	Duplicate Clone Stamp preset	コピースタンププリセットを複製	SHORTCUT_URL
shortcut	Paint	Set opacity for paint tool	ペイントツールの不透明度を設定	SHORTCUT_URL
shortcut	Paint	Set opacity to 100%	不透明度を100%に	SHORTCUT_URL
shortcut	Paint	Set flow for paint tool	ペイントツールのフローを設定	SHORTCUT_URL
shortcut	Paint	Set flow to 100%	フローを100%に	SHORTCUT_URL
shortcut	Paint	Move earlier or later by stroke duration	ストローク長単位で前後移動	SHORTCUT_URL
shortcut	Shapes	Group selected shapes	選択シェイプをグループ化	SHORTCUT_URL
shortcut	Shapes	Ungroup selected shapes	選択シェイプのグループ解除	SHORTCUT_URL
shortcut	Shapes	Enter free-transform path editing mode	自由変形パス編集モードへ	SHORTCUT_URL
shortcut	Shapes	Increase star inner roundness	星の内側の丸みを増加	SHORTCUT_URL
shortcut	Shapes	Decrease star inner roundness	星の内側の丸みを減少	SHORTCUT_URL
shortcut	Shapes	Increase points for star/polygon; increase roundness for rounded rectangle	星/多角形の頂点数、角丸長方形の丸みを増加	SHORTCUT_URL
shortcut	Shapes	Decrease points for star/polygon; decrease roundness for rounded rectangle	星/多角形の頂点数、角丸長方形の丸みを減少	SHORTCUT_URL
shortcut	Shapes	Reposition shape during creation	作成中にシェイプを再配置	SHORTCUT_URL
shortcut	Shapes	Set rounded rectangle to sharp; decrease outer roundness	角丸長方形をシャープに/外側の丸みを減少	SHORTCUT_URL
shortcut	Shapes	Set rounded rectangle to maximum; increase outer roundness	角丸長方形を最大丸みに/外側の丸みを増加	SHORTCUT_URL
shortcut	Shapes	Constrain shapes proportionally	シェイプを縦横比固定で作成	SHORTCUT_URL
shortcut	Shapes	Change outer radius of star	星の外側半径を変更	SHORTCUT_URL

## 3. panel (Windowメニューのパネル一覧)
panel	Window	Project	素材・コンポの管理パネル	MENU_PDF
panel	Window	Composition	コンポジションのプレビュー・編集パネル	MENU_PDF
panel	Window	Timeline	レイヤーと時間軸の編集パネル	MENU_PDF
panel	Window	Effect Controls	選択レイヤーのエフェクト設定パネル	MENU_PDF
panel	Window	Layer	個別レイヤーのプレビューパネル	MENU_PDF
panel	Window	Footage	素材のプレビューパネル	MENU_PDF
panel	Window	Flowchart	コンポ・レイヤー依存関係の図示パネル	MENU_PDF
panel	Window	Render Queue	レンダーキュー管理パネル	MENU_PDF
panel	Window	Align	整列パネル	MENU_PDF
panel	Window	Audio	オーディオレベル操作パネル	MENU_PDF
panel	Window	Brushes	ブラシ設定パネル	MENU_PDF
panel	Window	Character	文字設定パネル	MENU_PDF
panel	Window	Effects & Presets	エフェクト・プリセット一覧パネル	MENU_PDF
panel	Window	Info	選択項目の情報パネル	MENU_PDF
panel	Window	Mask Interpolation	マスク補間設定パネル	MENU_PDF
panel	Window	Media Browser	外部メディアブラウズパネル	MENU_PDF
panel	Window	Metadata	メタデータ表示パネル	MENU_PDF
panel	Window	Motion Sketch	モーションスケッチ記録パネル	MENU_PDF
panel	Window	Paint	ペイントツールパネル	MENU_PDF
panel	Window	Paragraph	段落設定パネル	MENU_PDF
panel	Window	Preview	プレビュー再生制御パネル	MENU_PDF
panel	Window	Progress	処理進捗表示パネル	MENU_PDF
panel	Window	Smoother	キーフレームスムージングパネル	MENU_PDF
panel	Window	Tools	ツールバーパネル	MENU_PDF
panel	Window	Tracker	モーション/カメラトラッカーパネル	MENU_PDF
panel	Window	Wiggler	キーフレームランダム化パネル	MENU_PDF
panel	Window	Essential Graphics	Motion Graphicsテンプレート作成パネル(近年追加)	https://helpx.adobe.com/after-effects/desktop/motion-graphics/work-with-motion-graphics-templates/creating-motion-graphics-templates.html
panel	Window	Libraries	Creative Cloud Librariesパネル(近年追加)	WORKSPACE_URL
panel	Window	Lumetri Color	カラーグレーディングパネル(近年追加)	https://www.provideocoalition.com/effects-nab-2017-update/
panel	Window	Lumetri Scopes	波形・ベクトルスコープパネル(近年追加)	https://www.provideocoalition.com/effects-nab-2017-update/
panel	Window	Properties	主要プロパティを集約したパネル(2022年AE 22.3で追加)	PROPS_URL
panel	Window	Content-Aware Fill	オブジェクト除去用パネル(2020年AE 17.5で追加)	https://helpx.adobe.com/after-effects/using/content-aware-fill.html

## 4. pref (環境設定カテゴリと主要項目)
pref	Preferences	General	その他一般設定(アンドゥ回数、ツールチップ表示、OS標準ショートカット使用等)	PREFS_URL
pref	Preferences	Previews	プレビュー全般の設定(オーディオプレビュー長、適応解像度品質、ハードウェアアクセラレーション等)	PREFS_URL
pref	Preferences	Display	モーションパス表示、Projectパネルサムネイル、Infoパネルのレンダリング表示等	PREFS_URL
pref	Preferences	Import	静止画の既定表示時間、埋め込みアルファの扱い、ドラッグ&ドロップ読み込みの挙動	PREFS_URL
pref	Preferences	Output	出力時のファイル分割、既定ファイル名/フォルダの使用可否	PREFS_URL
pref	Preferences	Grid & Guides	グリッド色・サイズ、ガイド色、セーフマージン(Action Safe / Title Safe / Center-Cut)	PREFS_URL
pref	Preferences	Labels	ラベル色とその割当の設定	PREFS_URL
pref	Preferences	Media & Disk Cache	キャッシュ・メディアフォルダの場所、Enable Disk Cache、Maximum Disk Cache Size、Enable Compressed Frames等	PREFS_URL
pref	Preferences	Video Preview	外部プレビュー出力デバイス(FireWire等)の設定	PREFS_URL
pref	Preferences	Appearance	UI外観・ラベル/マスクのハイライト色設定	PREFS_URL
pref	Preferences	Auto-Save	自動保存の間隔と保持世代数	PREFS_URL
pref	Preferences	Memory & Multiprocessing	メモリ割当・マルチフレームレンダリング・他アプリ用CPU予約	PREFS_URL
pref	Preferences	Audio Hardware	サウンドデバイス・ドライバ・サンプルレートの選択	PREFS_URL
pref	Preferences	Audio Output Mapping	Left/Right Channel割当、Audio Block Duration	PREFS_URL
pref	Preferences	Sync Settings	Creative Cloud経由でショートカット・設定を同期(Audio Hardware/Output Mapping/一部Media & Disk Cache設定は同期対象外)	PREFS_URL
pref	Preferences > Scripting & Expressions	Allow Scripts to Write Files and Access Network	スクリプトによるファイル書込・ネットワークアクセスを許可	SCRIPTPREF_URL
pref	Preferences > Scripting & Expressions	Enable JavaScript Debugger	ExtendScriptデバッガを有効化	SCRIPTPREF_URL
pref	Preferences > Scripting & Expressions	Warn User When Executing Files	スクリプトファイル実行時に警告	SCRIPTPREF_URL
pref	Preferences > Scripting & Expressions	Show Scripting Progress Dialog	スクリプト実行中の進捗ダイアログを表示	SCRIPTPREF_URL
pref	Preferences > Scripting & Expressions	Expression Pick Whip Writes Compact English	ピックウィップが簡潔な英語表記のエクスプレッションを生成	SCRIPTPREF_URL
pref	Preferences > Scripting & Expressions	Show Warning Banner When Project Contains Expression Errors	エクスプレッションエラー時に警告バナー表示	SCRIPTPREF_URL

## 集計
menu: 417件
shortcut: 315件
panel: 32件
pref: 21件(カテゴリ15 + Scripting & Expressions内訳6)

## 未列挙: 理由
- helpx.adobe.com への直接WebFetch/curlが全面的にタイムアウト(sandbox環境からの接続不良と思われる)。r.jina.ai経由のリーダープロキシで一部ページ(ショートカット表)のみ取得成功。Preferences本体ページ・Workspaces/Panelsページ・General UI Itemsページはr.jina.ai経由でも422エラーで本文取得不可となり、WebSearchのAI要約(=helpx.adobe.com記載内容の間接引用)で代替した。よって環境設定・パネルの一部項目は「Adobe公式ページの直接引用」ではなく「検索結果に現れた要約からの再構成」であることに留意。
- Effect メニューはインストール済みプラグインで動的に生成されるサブメニュー(3D Channel/Audio/Blur & Sharpen/Channel/Color Correction/Distort/Expression Controls/Generate/Keying/Matte/Noise & Grain/Obsolete/Perspective/Simulation/Stylize/Text/Time/Transition/Utility/Immersive Video/Synthetic Aperture等、数百エフェクト)であり、静的なMENU_PDFには"Effects Controls / Last Effect / Remove All"の固定項目しか記録されていない。エフェクト個別名は本タスクのメニュー木採取範囲外と判断し未列挙。
- MENU_PDFはAE CC 12.2.1x5(2015年)時点の抽出であるため、後年追加されたメニュー項目(例: Content-Aware Fill関連コマンド、Master Properties関連、Boundary Box、最新のAdd to Adobe Media Encoder Queue以外の書き出し形式等)は本リストに含まれていない可能性がある。Window/Preferencesは別途WebSearchで後年追加分(Essential Graphics, Libraries, Lumetri Color/Scopes, Properties, Content-Aware Fill / Scripting & Expressions)を補ったが、File/Edit/Composition/Layer/Effect/Animation/View/Help メニューの後年差分までは追いきれていない。
- ショートカット表のうち「Markers」専用カテゴリは、公式ショートカット参照ページに独立した見出しとして存在せず(Layer > Add Markerというメニュー項目のみ確認)、単独カテゴリとしては未列挙。
- ショートカット表のうち「Timeline Panel Properties and Groups」(A/S/T/P/R/U/UU等のプロパティ表示ショートカット)は、指示によりR6で既採集と判断し本リストから意図的に除外した。重複の要否はsupervisor判断に委ねる。
- Preferences各カテゴリのうち General/Import/Output/Display/Video Preview/Appearance/Auto-Save/Memory & Multiprocessing/Audio Hardware/Audio Output Mapping/Sync Settingsはカテゴリ概要は確認したが、Scripting & Expressionsほどの項目単位の完全な内訳は取得できていない(WebSearch要約の粒度に依存)。
