# persona の違和感 backlog — しらみ潰し(2026-09-03 利用者: 最大公約数でなく全部)

状態: ☐ 未 / ☑ 済 / ✗ 見送り(理由)。番号は persona の報告順。根拠の file:line は報告時点。

## 第 1 波(AE 10 年・NLE 編集者・Figma・初めての人) — §9 に記録済み。残り:
- ☐ Stage の文字を直接ダブルクリックで打つ
- ☐ 書体の欄(font family / weight)
- ☑ 印を掴んで動かす
- ✗ 履歴の行に操作名(Document に操作名が無い)、J/K/L(Clock に速度が無い) — model 側の口が先

## 第 2 波 K: キーボードだけの人
- ☑ K1 Tab / Shift+Tab: `aim_keystrokes` が焦点を `#app` へ奪い返す
- ☑ K2 焦点のある button を Enter / Space で押せない
- ☑ K3 menu をキーボードで開けない・歩けない — Tab + Enter で開けて中も押せる。開いている間は鍵を引かない(↑↓ の巡回は未)
- ☑ K4 Escape の層: Inspector の選択肢・机の引き出し・Browser の rail
- ☐ K5 数値・本文・名前の欄を開く鍵(焦点の行へ Enter)
- ✗ K6 Cmd+= / Cmd+- / Cmd+0 の UI 倍率 — ⌘= ⌘− ⌘0 は視点(ST1)が取った。窓の文字は Settings の ±
- ☑ K7 IME: 欄が閉じたら Disable、composing 中は Enter を送らない
- ☑ K8 焦点の輪が無い面(Timeline 行・Inspector の値・Browser の札)
- ☐ K9 素材を鍵で置く
- ☑ K10 Cmd+C / V / X(blitz-shell の clipboard feature)
- ☑ K11 Shift+↑↓ で選択を伸ばす
- ☐ K12 Cmd+←/→ = 印跳びは macOS の予約鍵(裁定 489 の preset 待ち)
- ✗ K13 複数選択で Enter が先頭だけ — Finder も 1 つずつ。一括の名前付けは別の手
- ✗ K14 Delete と Backspace 同義(害が小さい)
- ☑ K15 F9 の代替 ⌘⌥E 一族、効かなければ "Select keyframes first"
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
- ☑ S12 印を掴んで動かす(名前の表示は V8 と同根で未)
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
- ☑ D1 Composition menu で寸法・fps・尺・9:16
- ☑ D2 範囲と fps・寸法は Composition から(format preset は X1)
- ☐ D3 素材の欠落が窓に出ず relink の口も無い。描画が Asset でなく生 path を見る
- ☑ D4 Export sheet の 1 行サマリ
- ☑ D5 進捗の母数が最初 0 / 0、支度中に止められない
- ☑ D6 完了通知が status bar の 1 行だけ(Reveal・×・通知)
- ☑ D7 出力名が comp.mp4 固定 → 作品名(前回の出力先の記憶は未)
- ☐ D8 バッチ書き出し
- ☐ D9 snapshot 経由の書き出しが相対 path を壊す
- ☑ D10 白紙では Export が押せず理由が出る(ffmpeg 無しは未)
- ☑ D11 エラー文言が日本語

## 第 3 波 C: 色と Blend の VJ
- ☐ C1 効果の bypass・並べ替え・複製(EffectInstance に enabled が無い)
- ☐ C2 同じ効果を二度足せる、番号無し
- ☐ C3 効果の preset の器
- ☑ C4 Blend の札は式で混ざった色(暗・明・青・暖の下地)
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
- ☑ A2 Document を書くたび音が止まる — 音の指紋が同じなら再投影しない
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
- ☑ H17 drop の受け入れ可否が覆いに出る
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
- ☑ ST18 View ▸ Output Only で Stage を出す物だけに
- ☑ ST19 素のホイールが拡縮(先例は パン / ⌘+ホイール=拡縮)
- ☐ ST20 fit が paint でしか更新されない

## 第 4 波 M: 媒体の司書
- ☑ M1 置いた素材の尺が必ず comp の終わりまで(probe の nb_frames を渡す)
- ☑ M2 取り込みが窓を止める(SHA-256 を UI thread で doc.lock を握ったまま)
- ☑ M3 フォルダを取り込めない
- ☑ M4 重複が黙って消える(Imported 1 files と出て札は増えない)
- ☐ M5 札の情報が種別文字列だけ(尺・fps・解像度・容量)
- ☑ M6 札の絵は取り込みの糸で先に作る(失敗の記憶と re_video 化は R6)
- ☐ M7 hover scrub / 下見が無く、押すと即座に層が生まれる
- ☐ M8 Browser から Stage / Timeline へ引けない
- ☑ M9 差し替えが Alt+click の隠し技
- ☑ M10 Create の既定値 — "Text"・四角は短辺の 1/4・採番(書体の path と Null/Solid は未)
- ☑ M11 family の袖が空になっても残る
- ☑ M12 Library の下帯が先頭の素材名
- ☑ M13 札の下地色が並び順で回る
- ☑ M14 素材を library から外せない / Reveal in Finder
- ☐ M15 格子を鍵で歩けない(K9)
- ☑ M16 覆いは受け付けられる数と机の役目を言う(H17)

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
- ☑ F5 キー選択は行の差し替えで(層・属性・秒)を追い直す
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

## 営業 R: rerun で賄える物(→ [vendor-pitches](2026-09-03-vendor-pitches.md))
- ☐ R1 時間↔画素と目盛を re_time_ruler / re_format へ(V8・A15・F18)
- ☐ R2 当たり判定を PickingLayerProcessor へ
- ☐ R3 選択の縁取りを OutlineMaskProcessor へ
- ☐ R4 Fit / orbit を RectTransform / eye.rs の関数へ
- ☐ R5 グリッドを WorldGridConfiguration で
- ☐ R6 札の絵と素性を re_video へ(M5・M6・M7・A19)
- ✗ R7 mesh 読み込みの二重化 — importer は RenderContext(GPU)を要し、取り込み時の寸法取りは CPU 経路が正当
- ☐ R8 取っ手の大きさを re_renderer::Size へ
- ☑ R9 (CPU の式で代替)Blend / 効果のサムネイルを headless の ScreenshotProcessor で(C4・C13)

## 営業 B: Dioxus / Blitz で賄える物(→ [vendor-pitches](2026-09-03-vendor-pitches.md))
- ☑ B1 `accessibility` feature を立てる(X1〜X20 が platform へ)
- ☑ B2 `clipboard` feature を立てる(K10)
- ☐ B3 realise() と hotreload の複製を DioxusNativeApplication へ(上流 PR: pending_window を Vec に)
- ☐ B4 place_ime を上流の focus/blur へ(上流 1 行: ImeCapabilities に cursor_area)
- ☐ B5 thumbnail を NetProvider / ImageHandler へ(M6)
- ☐ B6 custom widget が ComputedStyles を読む(V25・V22・V23)
- ☐ B7 harness の合成 event を pointer_event / press_with / ime へ(Q15・K19)
- ☐ B8 dioxus-dnd を使い切る(M8・K9・M15・Q16・C1・F20)
- ☐ B9 欄の確定を onblur へ
- ☐ B10 H22 は 0 行(試験のみ)、H18 は @media prefers-color-scheme
- ☐ B11 scrollbars / svg feature
- ☐ BU1〜BU8 上流 PR 待ち(button の Enter/Space、gesture/drag event、TextInputData の選択 API、widget の文字、widget の a11y、menu の roving focus、窓 blur、NSMainMenu は範囲外)

## 第 5 波 N: 初めての 10 分
- ☑ N1 白紙で起動しない(fixture の完成品が毎回立つ)— blank_project を既定に、fixture は環境変数
- ☑ N2 空状態の案内が無い(.zhint が未使用)、既定の前面タブを Create に
- ☑ N3 tmeta の説明・◇ の title・既定パレット(Desk の名前は未)
- ☑ N4 履歴の一覧の入口が無い — Edit ▸ History…
- ☑ N5 Delete の後に "Deleted … · ⌘Z to undo"(Shift+click の範囲選択は未)
- ☑ N6 既定名 Untitled.rrd・filter "Motolii Project"(Save の sheet は Z11)
- ☑ N7 層が無くても Export が押せて真っ黒が出る
- ☑ N8 ⌘Q が無い(File ▸ Quit を自前 menubar に)
- ☑ N9 文言(No layer yet · select one / Nothing of this kind yet / Write what happens here)、.hint が hover の間しか出ない(V28)
- ☑ N10 Reset Layout の確認・報告・名前
- ☐ N11 title 属性が tooltip として描かれるか未確認(隠し技の唯一の伝達路)

## 第 5 波 Z: QA の再監査(今日の差分)
- ☑ Z1 K6(⌘= の UI 倍率)は ST1 に取られて消えている — ✗ 理由を書くか ⌥⌘ へ
- ☑ Z2 錠が新しい書く経路を素通り(キー削除・Alt+←→・Rename・色/α/hex・Replace・Stage の gizmo・marquee の選択)— Session::writable を 1 本
- ☐ Z3 別窓で開いた欄が主窓の押しで閉じず鍵が死ぬ / 印が動くと Note の欄が固着 — Field の use_drop・commit_field_outside の保険・storm の assert
- ☑ Z4 replace_project が view_request / imports / project_notice / saving を取りこぼす
- ☑ Z5 Alt+←→ がキーと層の二役(Escape でキー選択が落ちない)
- ☑ Z6 Escape が机の引き出しと Browser の rail を閉じない(K4 の詰め残し)
- ☑ Z7 blend の hover 下見の transient が mouseleave 無しで残る(選択変更・焦点・引き出し閉じ・窓の焦点喪失)— 全 transient を落とす 1 本
- ☑ Z8 札の上の Replace / Finder / × が hover のみ(:focus-within を足す)
- ☑ Z9 α の帯は縦にぶれた瞬間に確定して追従が止まる(color と同じ move の保険)
- ☑ Z10 tablist / tabpanel(aria-controls と roving tabindex は未)
- ☐ Z11 仕舞う経路が 2 本(host.rs save_now と project.rs put_away_inner)で文言と後始末が食い違う
- ☑ Z12 select_inside の層選択が ⌘/⇧ を無視して置き換える
- ☐ Z13 試験の穴(storm が新しい状態を見ない、印の drag が move_marker を直接呼ぶ、Note の欄が死なない)
- ☑ Z14 set_typing(false) と handles の重複(note_key_down は未)

## 第 5 波 L: 歌詞動画を 1 本作り切る(致命 3 つが先)
- ☑ L1 Composition menu(⌥⌘K)に preset / 寸法 / fps / 尺 / Fit to layers(background・Export の範囲と preset は未 = D2・D4)
- ☑ L2 Content の ◇/◆ で今の時刻に本文のキーを立てる・外す、Timeline に菱形(複数行の貼り付けを行ごとに配るのは未)
- ☑ L3 音に関わる変更だけで再投影(指紋で gate、A2)— スクラブの音(A1)と BPM(S13)は未
- ☐ L4 空の comp に最初の音・動画を入れたら尺を素材に伸ばす(place が切った事を status に)
- ☐ L5 Import に ⌘I
- ☑ L6 M の toggle が連打で直前の印を消す
- ☐ L7 Text 節に family / weight / justify(S6・S5 の残り)
- ☐ L8 文字送り(Range Selector を描く側から、S7)、出入りの preset(C3)
- ☐ L9 Create に Solid、新しい素材は下・文字は上(新層が必ず一番上)
- ☐ L10 ループ区間 I / O(A3)
- ☑ L11 Timeline の帯 drag を選択している全層へ(finish_drag が drag.layer 1 枚)
- ☐ L12 層の Copy / Paste(行の入れ替え)

## 第 6 波 E: AE 10 年の再点検
- ☐ E1 P/S/R/T/A の単打で属性の行を絞る(U の同族)、UU
- ☐ E2 J/K で前後のキーへ、I/O で層の頭/尻へ
- ☑ E3 ⌘⇧D で分割(⌘K は切る手のまま、枠は ⌥⌘K)
- ☐ E4 Y の pan-behind = Stage でアンカーを掴む(ST5)
- ☐ E5 ◆ の ⌥ が AE と逆(停止時計を別に置き、◇/◆ は今の 1 つだけ)
- ☐ E6 数値 drag の ⇧=10x ⌘=0.1x
- ☑ E7 ⇧+click が帯と Stage で効かない(キーと囲いは効く)
- ☑ E8 帯とキーの drag が ⌘ で吸い付きを切れない(再生位置だけ)
- ☑ E9 端の ⌥drag が Trim に食われる(⌥ は常に Slip)
- ☑ E10 F9 の代替鍵(⌘⌥E 一族)と、0 track の時の報せ
- ☐ E11 複数選択の拡縮が主の層だけ(Move は配る)
- ☐ E12 Pre-compose(時間を持つ入れ物)、shy、Work Area(B/N)、Motion Blur、Frame Blending、Continuously Rasterize — 無い
- ☐ E13 RAM preview(描画のフレーム cache と緑帯)
- ☐ E14 印の番号打ち
- ☐ E15 AE を超える芽: blend の式を効果カードの絵へ(C13)、キー同定子を Undo 後の選択復元と ⌘C/V へ、◇ で貼り付けを行ごとに配る、印の BPM 格子、音の指紋を描画 cache へ

## 第 6 波 P: 規模と性能(層 200・7200 コマ・4K×3・キー 2000・印 400・素材 50)
- ☐ P1 【致命】view.attrs()/meta() が毎回 latest_at + serde_json、rows_nested が O(層²)— StoreView に revision 付き cache、親表を 1 回で
- ☐ P2 【致命】text の texture が comp 解像度 × 無制限(200 層で 1.6GB、4K で 6.6GB)— 実バウンディング + LRU 上限、鍵を u64 に
- ☑ P3 Stage は変化があった時だけ描き、層は paint で 1 回だけ解く
- ☑ P4 行の投影は Document の revision が動いた時だけ(面の再構築の分割は未)
- ☐ P5 【致命】Timeline 左列に仮想化が無い(6,000 node)
- ☐ P6 音の指紋が O(層)の String、from_view が同期 decode(92MB/本、evict 無し)、doc.lock の中
- ☐ P7 保存済み project を開くと札の decode / ffmpeg が描画の糸(MADE は process 内)— cache に無い物は別の糸へ、data URI の clone
- ☐ P8 複数選択の Inspector が O(選択 × 層)
- ☐ P9 Undo 履歴が伸びっぱなし、SetTrack が track 丸ごと(帯 1 拍で 1,000 chunk)
- ☐ P10 保存が UI の糸で全同期(flattened + encode)
- ☐ P11 Timeline paint が selected の線形探索、waveform_tracks の to_vec、snap_targets の再生成
- ☑ P12 select_inside が行ごとに doc.lock
- ☐ P13 Desk が再生位置ごとに再構築(印 400 の parse、参考画像の data URI clone)
- ☑ P14 天井は帯・印・作品の尺の最遠(MIN_PPS の動的下限と sfac は未)
- ☑ P15 別窓の CSS 倍率が 100% 固定(150% で左右がずれる)
- ☑ P16 1/3000 量子化が truncate で ◆ の一致が外れる — clock.current_time() を使う
- ☐ P17 小物: has_layer が layers()、can_export が毎 render、used / used_colors が毎 render、擦りの transient × 選択数、blend の enter/leave が revision

## 第 6 波 VO: VoiceOver(adapter が立った後)
- ☑ VO1 名前の路は可視の文字だけ — aria_label を見えない span(.a11y)として置く(SemanticButton / SemanticControl / Field)
- ☐ VO2 node に矩形が無い(VO の枠が動かない)— 上流 B13
- ☑ VO3 Inspector の升と層名に焦点が届く — tabindex / spinbutton / option、Enter で欄(K5)
- ☑ VO4 tab の roving tabindex・aria-controls・tabpanel の名前(ptools を tablist の外へは未)
- ☑ VO5 menuitemcheckbox の状態を文字で(on / off)
- ☐ VO6 disabled が届かない — 理由を可視の文言へ
- ☐ VO7 status / live が届かない — 上流 B14
- ☑ VO8 input / textarea に名前(Field の label)
- ☑ VO9 hover だけの物(.tacts を opacity へ)
- ☑ VO10 見出し(.sec → h3、.sh → h3、b → h2)、img の alt
- ☑ VO11 起動直後の一言(h1 Motolii)
- ☐ VO12 DOM 順(MenuDismiss が先頭)、stagehint の role
- ☐ VO13 custom widget の代替路の穴: Ease、層の並べ替え
- ☑ VO14 焦点の輪(content の欄の outline:none を撤回)
- ☑ VO15 コントラスト(INK3 0x92→0xa0、BORDER 0x63→0x74)
- ☐ B12 上流: aria-label / presentation / disabled / aria-* を a11y へ
- ☐ B13 上流: a11y node に bounds
- ☐ B14 上流: live region の TreeUpdate

## 第 6 波 X: 納品(書き出し)
- ☑ X1 Export sheet(範囲 All / 印から印、1 行サマリ)— format(ProRes4444 / PNG / 連番 / Lottie)は未
- ☑ X2 書き込み失敗が "Could not read the frame" と嘘をつく(Desc を read / write に割る)
- ☑ X3 disk full が "Broken pipe"(ffmpeg の stderr を読む)
- ☑ X4 ffmpeg 無しの文言と、tools_available が未使用(can_export で先に断る)
- ☐ X5 色: 中身 sRGB・タグ bt709(in_range=full を明示 / transfer)、PNG が premultiplied のまま
- ☑ X6 29.97 / 23.976 / 59.94 が窓から選べない
- ☑ X7 Choosing phase、start の Err は status へ
- ☑ X8 終わった状態から Idle へ戻る道が無い(Dismiss)
- ☐ X9 音の支度中に進捗が 0 のまま(Mixing audio…)

## 第 6 波 KB: 鍵だけの再点検
- ☐ KB1 【致命】Shift+Tab が前へ行かない(上流 keyboard.rs は Tab のみ)— keys.rs に focus_prev
- ☑ KB2 【致命】面タブを ←→ で切り替えられない(Colors へ届かない)
- ☑ KB3 【致命】Content の span に焦点が届かない(tabindex / textbox / Enter)
- ☑ KB4 【致命】層名で Enter / Space が飲まれる(onkeydown が無い)
- ☑ KB5 【致命】Composition の生 input が窓中の鍵を人質に(FIELD が生 input を拾う)
- ☐ KB6 欄を確定すると焦点が #app へ飛ぶ(開いた時の焦点へ返す)
- ☑ KB7 ⌥ を伴う文字の binding が macOS で発火しない疑い(⌥⌘E・⌥⌘K)— code→文字の表を alt 全域へ
- ☐ KB8 Tab 順: Position まで 16、層 16 枚で 80 停止(roving・仮想化)
- ☑ KB9 menu の ↑↓ が Composition / Settings で効かない(SemanticButton が MenuItems に名乗らない)
- ☑ KB10 ↓ が property 行に落ちると選択が消える
- ☑ KB11 .csheet-in の輪(#app と custom widget は未)
- ☐ KB12 menu を開けている間は全 Intent が死ぬ(⌥⌘K で閉じられない)、Browser の rail は Escape で閉じない
