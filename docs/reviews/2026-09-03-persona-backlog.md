# persona の違和感 backlog — しらみ潰し(2026-09-03 利用者: 最大公約数でなく全部)

状態: ☐ 未 / ☑ 済 / ✗ 見送り(理由)。番号は persona の報告順。根拠の file:line は報告時点。

## 第 1 波(AE 10 年・NLE 編集者・Figma・初めての人) — §9 に記録済み。残り:
- ☐ Stage の文字を直接ダブルクリックで打つ
- ☐ 書体の欄(font family / weight)
- ☐ 印を掴んで動かす
- ✗ 履歴の行に操作名(Document に操作名が無い)、J/K/L(Clock に速度が無い) — model 側の口が先

## 第 2 波 K: キーボードだけの人
- ☑ K1 Tab / Shift+Tab: `aim_keystrokes` が焦点を `#app` へ奪い返す
- ☑ K2 焦点のある button を Enter / Space で押せない
- ☑ K3 menu をキーボードで開けない・歩けない — Tab + Enter で開けて中も押せる。開いている間は鍵を引かない(↑↓ の巡回は未)
- ☑ K4 Escape の層: Inspector の選択肢・机の引き出し・Browser の rail
- ☐ K5 数値・本文・名前の欄を開く鍵(焦点の行へ Enter)
- ☑ K6 Cmd+= / Cmd+- / Cmd+0 の UI 倍率
- ☑ K7 IME: 欄が閉じたら Disable、composing 中は Enter を送らない
- ☑ K8 焦点の輪が無い面(Timeline 行・Inspector の値・Browser の札)
- ☐ K9 素材を鍵で置く
- ☐ K10 Cmd+C / V / X
- ☑ K11 Shift+↑↓ で選択を伸ばす
- ☐ K12 Cmd+←/→ = 印跳びは macOS の予約鍵(裁定 489 の preset 待ち)
- ☐ K13 複数選択で Enter が先頭だけ
- ✗ K14 Delete と Backspace 同義(害が小さい)
- ☐ K15 F9 は OS が食う(案内)
- ☑ K16 menu に加速鍵の表記
- ☑ K17 forget_modifiers が欄を開くたび走る
- ☐ K18 auto-repeat の加速
- ☐ K19 harness の鍵の道(1・2 が構造上捕まらない)

## 第 2 波 S: 日本語の歌詞デザイナー
- ☑ S1 印に吸い付く
- ☑ S2 層を動かすと歌詞の切替が置き去り
- ☑ S3 縁取りを足せる(行は常に出す、幅 0 なら級数の 5%)
- ☐ S4 縦書き(model から)
- ☑ S5 行間・字送りの行(Justify は未)
- ☐ S6 書体・ウェイト
- ☐ S7 文字単位アニメーション(Range Selector: model は在り、描画も UI も無い)
- ☐ S8 複数スタイル(runs)
- ☐ S9 ルビ
- ☐ S10 折り返し(wrap_size 未使用)
- ☑ S11 書き置きを層へ(To layer)
- ☐ S12 印を掴む・名前
- ☐ S13 BPM / 拍
- ☐ S14 IME の確定文字と単一行欄の Enter(K7 と同根)
- ☑ S15 文字色の α

## 第 2 波 V: 視覚デザイン(styles.css / tokens.rs)
- ☑ V1 面の分離(panel / raised / hover の値)
- ☑ V2 ink3 のコントラスト、t-micro 8px
- ☑ V3 未定義 token(--border, --sp5/6, --fg)
- ☑ V4 Inspector の値に hover が無い
- ☑ V5 机の chip が素の文字(selector が #stagefoot 限定)
- ☑ V6 Colors の左袖で文字が溢れる
- ☑ V7 Timeline の層名列が色の壁
- ☐ V8 時間軸に目盛の数字が無い(timeline widget に文字を描く口が無い — vello の text が要る)
- ☑ V9 窓の頭が二重(title + appname)
- ☑ V10 罫線 #555 のコントラスト
- ☑ V11 数字の桁が行ごとに違う
- ☑ V12 スカラ値が Z 列
- ☐ V13 見出し 3 段
- ☑ V14 Key 列の見出しだけ accent
- ☑ V15 .ptab の cursor: grab
- ☑ V16 色相環が円板で赤が 3 時
- ☑ V17 hex が表示専用
- ☐ V18 空状態の文体
- ☑ V19 カササギが drop 領域に被る、色が palette 外
- ☑ V20 status bar が痩せている、尺表記の不一致
- ☑ V21 focus ring が 1px 内側
- ☐ V22 級数の段
- ☐ V23 間隔の律
- ☑ V24 .vgrip:hover が全面 accent
- ☐ V25 Stage の作品枠が外と同色(custom widget が自分で塗る。CSS では効かない)
- ☑ V26 .btab 死に selector、.tgrid 2 列固定
- ☐ V27 stagefoot の ◎ が無枠
- ☐ V28 常設ヒント

## 第 2 波 Q: QA の状態 bug
- ☑ Q1 Undo/Redo が窓側の id を掃除しない
- ☑ Q2 replace_project が scrub / panel_ask を残す
- ☑ Q3 印の本文が index 指し(並べ替えでずれる)
- ☑ Q4 窓外 release の受け口が別窓に無い
- ☑ Q5 panel_ask が別窓から届かない
- ☑ Q6 錠が編集経路で見られていない
- ☑ Q7 複数選択で同じ値を打つと他が更新されない
- ☑ Q8 ダブルクリックが擦りの transient を残す
- ☑ Q9 set_typing が欄より長生き(閉じた直後の 1 打鍵)
- ☑ Q10 保存の revision の取り直し、Cmd+S 連打
- ☑ Q11 layout.json の古い panel 名
- ☑ Q12 主窓を閉じても別窓が残る
- ☑ Q13 アンカー升が前の層の箱で押される
- ☑ Q14 style が空の文字層
- ☐ Q15 harness と窓のずれ(click_super、IME の試験)
- ☐ Q16 落とした物・drag の後始末

## 第 3 波 D: 書き出しの人
- ☐ D1 comp の設定(寸法・fps・尺・縦)を窓から変えられない — 9:16 が作れない
- ☐ D2 書き出しの選択肢がゼロ(preset・寸法・fps・範囲)
- ☐ D3 素材の欠落が窓に出ず relink の口も無い。描画が Asset でなく生 path を見る
- ☐ D4 書き出し前の確認(1 行のサマリ)
- ☑ D5 進捗の母数が最初 0 / 0、支度中に止められない
- ☑ D6 完了通知が status bar の 1 行だけ(Reveal・×・通知)
- ☑ D7 出力名が comp.mp4 固定 → 作品名(前回の出力先の記憶は未)
- ☐ D8 バッチ書き出し
- ☐ D9 snapshot 経由の書き出しが相対 path を壊す
- ☐ D10 白紙と ffmpeg 無しの無表示
- ☑ D11 エラー文言が日本語

## 第 3 波 C: 色と Blend の VJ
- ☐ C1 効果の bypass・並べ替え・複製(EffectInstance に enabled が無い)
- ☐ C2 同じ効果を二度足せる、番号無し
- ☐ C3 効果の preset の器
- ☐ C4 Blend のサムネイルが空の四角
- ☐ C5 hover preview が鍵(focus)とタッチに無い
- ☑ C6 色の書き戻しが gradient を潰す
- ☑ C7 線形光の一言
- ☐ C8 Add が mix の列に混じる
- ☐ C9 色を保存する口(palette、裁定 244)
- ☐ C10 スポイト
- ☑ C11 α が drag できず 1 押し 1 手
- ☐ C12 効果の色 param
- ☐ C13 効果カードに絵・検索・分類
- ☑ C14 BLEND 行で机が前に出る
- ☑ C15 hex / HSV / RGB を打つ口

## 第 3 波 A: 音と拍の編集者
- ☐ A1 スクラブに音が無い(seek が device を捨てる)
- ☐ A2 Document を書くたび音が止まる(sync_document)
- ☐ A3 ループ区間・イン/アウト
- ☑ A4 印を打つと机が毎回開く → 机が Follow の時だけ
- ☑ A5 印へ跳んでも画面が付いてこない
- ☑ A6 S(solo)が音に効かない、音だけの mute が無い
- ☐ A7 メーターが死んでいる・音量の口が無い
- ☐ A8 波形がモノラル
- ☑ A9 波形の解像度が Retina で半分
- ☐ A10 波形が線形振幅
- ☑ A11 印が目盛の中にしか描かれない → 全 track を貫く線
- ☑ A12 吸い付きを切れない
- ☑ A13 再生位置のスクラブが吸い付かない
- ☑ A14 目盛の刻みが 1 秒固定 → 倍率で段
- ☐ A15 空きを押すと再生位置が動かない
- ☑ A16 印の名前が採番 → タイムコード
- ☐ A17 波形の生成中が無表示
- ☑ A18 尺の表記が秒丸め
- ☐ A19 動画の音は 1 本目だけ

## 第 3 波 H: macOS HIG
- ☐ H1 NSMainMenu が無い(⌘Q ⌘H ⌘M ⌘W ⌘, が全滅)— objc2-app-kit で App / Edit / Window / Help の 4 本
- ☐ H2 Services・標準 Edit(日本語の入力ソース・Emoji)— H1 と同時
- ☐ H3 App menu(About・⌘,)
- ☐ H4 フルスクリーン・Window menu
- ☑ H5 未保存で閉じる alert に Save が無い(project.rs の 3 択へ寄せる)
- ☑ H6 rfd の dialog に set_parent — Import・Export・Open は sheet(Save は put_away が窓を持たず未)
- ☑ H7 title bar が書類を指さない(set_title・representedFilename・documentEdited)
- ☐ H8 Open Recent
- ☐ H9 Revert to Saved
- ☐ H10 自動保存・版
- ☑ H11 ⌘N ⌘O ⇧⌘S の加速鍵
- ☐ H12 ⌘W と閉じる意味(窓を閉じる ≠ 終了)
- ☐ H13 窓の位置・大きさの復元
- ☐ H14 別窓に menu が無い(H1 で解決)
- ☑ H15 save panel に種別の絞り
- ✗ H16 DragEntered で欄が確定してしまう — 読み違い(focus_lost は掴みと menu を畳むだけで、欄の確定は Focused(false) のみ)
- ☐ H17 drop の受け入れ可否が視覚に出ない
- ☐ H18 dark 固定(Appearance の設定)
- ☐ H19 accent が OS 設定を無視
- ☐ H20 Reduce Motion を起動時にしか読まない
- ☑ H21 menubar の role / aria-controls
- ☐ H22 scale_factor 変化(外部 display)の再 layout — 要実窓
- ☑ H23 alert の文面(host.rs と project.rs を HIG の文に統一)

## 第 4 波 ST: Stage / 空間の人
- ☑ ST1 ⌘0 ⌘= ⌘− が UI 倍率に取られ、view の Fit / 100% の入口が無い
- ☑ ST2 倍率の表示が無い(#stagefoot)
- ☑ ST3 中ボタン=パン、右ボタンは何もしない(Space+drag は未)
- ☑ ST4 複数選択を一括で動かす(marquee は未)
- ☐ ST5 アンカーを Stage で掴めない
- ☑ ST6 Shift の軸拘束(スナップは未)
- ☑ ST7 矢印で 1px ナッジ(Stage focus 時)
- ☑ ST8 修飾を離してから放すと確定値がプレビューと食い違う
- ☑ ST9 書き出し枠のドラッグが無条件でカメラにキーを打つ
- ☐ ST10 hover / cursor / 変形中の数値が無い
- ☑ ST11 描いた取っ手と掴める取っ手の大きさが違う
- ☑ ST12 ◎ chip が 3D の状態を読めない
- ☑ ST13 奥行きドラッグが下=奥、取っ手が回転帯と食い合う
- ☑ ST14 orbit が触っていない軸にもキーを書く
- ☐ ST15 市松・セーフエリア・グリッド・定規・ガイドが無い
- ☐ ST16 整列・分布
- ☐ ST17 Stage で文字を直接打てない(第 1 波の残り)
- ☐ ST18 output_only が到達不能
- ☑ ST19 素のホイールが拡縮(先例は パン / ⌘+ホイール=拡縮)
- ☐ ST20 fit が paint でしか更新されない

## 第 4 波 M: 媒体の司書
- ☑ M1 置いた素材の尺が必ず comp の終わりまで(probe の nb_frames を渡す)
- ☑ M2 取り込みが窓を止める(SHA-256 を UI thread で doc.lock を握ったまま)
- ☑ M3 フォルダを取り込めない
- ☑ M4 重複が黙って消える(Imported 1 files と出て札は増えない)
- ☐ M5 札の情報が種別文字列だけ(尺・fps・解像度・容量)
- ☐ M6 札の絵を描画の最中に作る(ffmpeg 同期 spawn、失敗を永久に憶える)
- ☐ M7 hover scrub / 下見が無く、押すと即座に層が生まれる
- ☐ M8 Browser から Stage / Timeline へ引けない
- ☑ M9 差し替えが Alt+click の隠し技
- ☑ M10 Create の既定値 — "Text"・四角は短辺の 1/4・採番(書体の path と Null/Solid は未)
- ☑ M11 family の袖が空になっても残る
- ☑ M12 Library の下帯が先頭の素材名
- ☑ M13 札の下地色が並び順で回る
- ☑ M14 素材を library から外せない / Reveal in Finder
- ☐ M15 格子を鍵で歩けない(K9)
- ☐ M16 落とす先の可否・役目が覆いに出ない(H17)

## 第 4 波 X: 支援技術と文言
- ☑ X1 再生ボタンに名前が無い
- ☑ X2 書き出しの進捗・status bar に live region が無い
- ☑ X3 M/S/L の aria_label が可視文字と違う
- ☑ X4 ◎ の title と aria_label が食い違う / "To layer" → "Send to layer"
- ☑ X5 面タブが span で focus も role も無い
- ☑ X6 disabled な札の理由が届かない
- ☑ X7 "Imported 1 files"
- ☑ X8 書き出し error の文体(小文字始まり・回復手段無し)
- ☑ X9 "Wrote {fullpath}" など完了文が経路まる出し
- ☑ X10 未保存 dialog の文面が 2 種類
- ☑ X11 View menu の ✓ が字(menuitemcheckbox へ)
- ☑ X12 "Window" が 8 個同名
- ☑ X13 倍率・調光の値が名前に紐づかない
- ☑ X14 状態が色だけ(.lit .on)
- ☑ X15 OBJECT / rows / layer の呼び名
- ☑ X16 空状態の文体が揃わない
- ☑ X17 命令文と説明文の混在
- ☑ X18 参考画像の名前が 3 回読まれる
- ☑ X19 見出しの全大文字を CSS へ
- ☑ X20 Freeze を chip に・"Blend preview · linear light"・"Attached"

## 第 4 波 F: キーを打つ人
- ☑ F1 Delete がキーでなく層を消す(selected_keys があればキー削除)
- ☑ F2 同じトラックの複数キーを掴むと 1 つしか動かない(SetTrack が最後勝ち)
- ☑ F3 分割が shape / text を落とす・切り口の Interp を split_at しない
- ☑ F4 帯の時刻が 30fps 決め打ち(fixture FPS)
- ☐ F5 キー選択が index(行が組み替わると別のキー)
- ☑ F6 掴み終わりに publish_keys を呼ばない(F9 が空振り)
- ☑ F7 キーを 1 つだけ消す口が無い(◆ navigator)
- ☐ F8 既存のキーへ落とすと黙って上書き
- ☑ F9 Hold が棚に無い
- ☑ F10 錠が Timeline の掴みに効かない
- ☐ F11 層の行のキーへ形を当てると全属性に乗る / 値グラフが無い
- ☑ F12 Easy Ease In / Out が AE と裏返し
- ☑ F13 キーを鍵で動かせない(Alt+←→)
- ☐ F14 spatial(モーションパス)が窓から触れない
- ☐ F15 タイムストレッチ(speed)の UI が無い
- ☑ F16 スリップ(Alt+drag)が無い
- ☑ F17 囲い選択で層も選ぶ(Alt の除外は未)
- ☑ F18 右に天井(zoom to fit は未)
- ☑ F19 キーの当たり半径が UI 倍率に付いてこない
- ☐ F20 親付けがピックウィップでない
- ☑ F21 掴んでいる間フレームに丸まらない
- ☑ F22 複製の重ね順が衝突する
