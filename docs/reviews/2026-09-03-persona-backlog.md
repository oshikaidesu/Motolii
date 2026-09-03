# persona の違和感 backlog — しらみ潰し(2026-09-03 利用者: 最大公約数でなく全部)

状態: ☐ 未 / ☑ 済 / ✗ 見送り(理由)。番号は persona の報告順。根拠の file:line は報告時点。

## 第 1 波(AE 10 年・NLE 編集者・Figma・初めての人) — §9 に記録済み。残り:
- ☐ Stage の文字を直接ダブルクリックで打つ
- ☐ 書体の欄(font family / weight)
- ☐ 印を掴んで動かす
- ✗ 履歴の行に操作名(Document に操作名が無い)、J/K/L(Clock に速度が無い) — model 側の口が先

## 第 2 波 K: キーボードだけの人
- ☐ K1 Tab / Shift+Tab: `aim_keystrokes` が焦点を `#app` へ奪い返す
- ☐ K2 焦点のある button を Enter / Space で押せない
- ☐ K3 menu をキーボードで開けない・歩けない(↑↓ が SelectStep に食われる)
- ☐ K4 Escape の層: Inspector の選択肢・机の引き出し・Browser の rail
- ☐ K5 数値・本文・名前の欄を開く鍵(焦点の行へ Enter)
- ☐ K6 Cmd+= / Cmd+- / Cmd+0 の UI 倍率
- ☐ K7 IME: 欄が閉じたら Disable、composing 中は Enter を送らない
- ☐ K8 焦点の輪が無い面(Timeline 行・Inspector の値・Browser の札)
- ☐ K9 素材を鍵で置く
- ☐ K10 Cmd+C / V / X
- ☐ K11 Shift+↑↓ で選択を伸ばす
- ☐ K12 Cmd+←/→ = 印跳びは macOS の予約鍵(裁定 489 の preset 待ち)
- ☐ K13 複数選択で Enter が先頭だけ
- ✗ K14 Delete と Backspace 同義(害が小さい)
- ☐ K15 F9 は OS が食う(案内)
- ☐ K16 menu に加速鍵の表記
- ☐ K17 forget_modifiers が欄を開くたび走る
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
- ☐ S15 文字色の α

## 第 2 波 V: 視覚デザイン(styles.css / tokens.rs)
- ☐ V1 面の分離(panel / raised / hover の値)
- ☐ V2 ink3 のコントラスト、t-micro 8px
- ☐ V3 未定義 token(--border, --sp5/6, --fg)
- ☐ V4 Inspector の値に hover が無い
- ☐ V5 机の chip が素の文字(selector が #stagefoot 限定)
- ☐ V6 Colors の左袖で文字が溢れる
- ☐ V7 Timeline の層名列が色の壁
- ☐ V8 時間軸に目盛の数字が無い
- ☐ V9 窓の頭が二重(title + appname)
- ☐ V10 罫線 #555 のコントラスト
- ☐ V11 数字の桁が行ごとに違う
- ☐ V12 スカラ値が Z 列
- ☐ V13 見出し 3 段
- ☐ V14 Key 列の見出しだけ accent
- ☐ V15 .ptab の cursor: grab
- ☐ V16 色相環が円板で赤が 3 時
- ☐ V17 hex が表示専用
- ☐ V18 空状態の文体
- ☐ V19 カササギが drop 領域に被る、色が palette 外
- ☐ V20 status bar が痩せている、尺表記の不一致
- ☐ V21 focus ring が 1px 内側
- ☐ V22 級数の段
- ☐ V23 間隔の律
- ☐ V24 .vgrip:hover が全面 accent
- ☐ V25 Stage の作品枠が外と同色
- ☐ V26 .btab 死に selector、.tgrid 2 列固定
- ☐ V27 stagefoot の ◎ が無枠
- ☐ V28 常設ヒント

## 第 2 波 Q: QA の状態 bug
- ☐ Q1 Undo/Redo が窓側の id を掃除しない
- ☐ Q2 replace_project が scrub / panel_ask を残す
- ☐ Q3 印の本文が index 指し(並べ替えでずれる)
- ☐ Q4 窓外 release の受け口が別窓に無い
- ☐ Q5 panel_ask が別窓から届かない
- ☐ Q6 錠が編集経路で見られていない
- ☐ Q7 複数選択で同じ値を打つと他が更新されない
- ☐ Q8 ダブルクリックが擦りの transient を残す
- ☐ Q9 set_typing が欄より長生き(閉じた直後の 1 打鍵)
- ☐ Q10 保存の revision の取り直し、Cmd+S 連打
- ☐ Q11 layout.json の古い panel 名
- ☐ Q12 主窓を閉じても別窓が残る
- ☐ Q13 アンカー升が前の層の箱で押される
- ☐ Q14 style が空の文字層
- ☐ Q15 harness と窓のずれ(click_super、IME の試験)
- ☐ Q16 落とした物・drag の後始末
