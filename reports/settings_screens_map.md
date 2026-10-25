# 新しい文書・文書の設定・環境設定の画面の照らし合わせ（10/25）

Motolii に画面そのものが無い所のうち、A：新しい文書を作る画面、B：あとから文書の設定を変える画面、C：設定（環境設定）の画面を、お手本 10 本の公式から集めた（Adobe は比べるだけで数に入れない）。数は「その項目を持っているお手本の数」。○ は作る窓ではなく、既定・見本・ひな形で決まるもの。

お手本の略：Cav＝Cavalry（cavalry.studio/docs）、Bl＝Blender 5.2（docs.blender.org/manual）、Ji＝Jitter（help.jitter.video）、Ri＝Rive（rive.app/docs）、Fi＝Figma（help.figma.com）、DR＝DaVinci Resolve 20（リファレンスマニュアル 96〜152 ページ）、FCP＝Final Cut Pro（support.apple.com/guide/final-cut-pro）、Live＝Ableton Live 12（ableton.com/en/live-manual/12）、CSP＝クリスタ（help.clip-studio.com）、Sp＝Spline（docs.spline.design）。Adobe は helpx の検索結果の抜き出しだけ。

## Motolii いま

- **A 新しい文書**：File ▸ New は untitled_N を作るだけ。大きさ・速さ・長さ・背景を選ぶ画面が無い。
- **B 文書の設定**：大きさだけ、上の帯の札で変えられる（こする・WxH を打つ）。**コマの速さと長さはどこからも変えられない**（インスペクタの「何も選んでいない時」の欄に、文書の幅・高さ・fps はあるが隠してある：sets/inspector/inspector_choosers_header.dart:340）。
- **C 設定**：無い。見た目（View ▸ Skin）、倍率（上の帯）、手で書く keybindings.json・tags.json・window.xml だけ。

## A：新しい文書を作る画面

| 項目 | お手本 | 数 | Adobe | Motolii |
|---|---|---|---|---|
| 大きさの見本の一覧 | Fi・Ri・Ji・CSP・FCP・Bl○・DR○・Cav○ | 8 | AE・Pr | 無い |
| 自分で幅・高さ | Fi・Ri・CSP・FCP・Ji・Bl○・DR○・Cav○ | 8 | AE・Pr | 無い（後で札） |
| ひな形 | Bl・Ji・Live・CSP・DR○・Cav○ | 6 | – | 無い |
| コマの速さ | FCP・CSP・Cav○・Bl○・DR○ | 5 | AE・Pr | 無い |
| SNS・縦長の見本 | Fi・Ji（Instagram 4:5 など）・FCP（縦横比で選ぶ）・DR○ | 4 | – | 無い |
| 自分の見本を保存・既定にする | Cav・DR・CSP・Live | 4 | AE | 無い |
| 色空間 | FCP・DR○・Cav○ | 3 | Pr | 無い |
| 長さ | CSP・Ji（4 秒）・Bl○ | 3 | AE | 無い |
| 背景色・紙の色 | CSP・Cav○ | 2 | AE | 無い |
| 開始タイムコード | FCP・DR | 2 | AE | 無い |
| 縦横比の鍵 | Fi・Ji | 2 | AE | 無い |
| 縦と横の入れ替え | CSP・Bl | 2 | – | 無い |
| 作るものの種類（イラスト・アニメ…） | CSP・Bl | 2 | – | 無い |
| 前の設定を覚える・最初の素材に合わせる | FCP | 1 | AE | 無い |
| 音のチャンネル・サンプルレート | FCP | 1 | Pr | 無い |
| 安全な枠・余白 | CSP | 1 | – | 画面に Safe の表示はある |

## B：あとから変える画面

| 項目 | お手本 | 数 | Adobe | Motolii |
|---|---|---|---|---|
| 幅・高さ | Cav・Bl・Ji・Ri・Fi・DR・FCP・CSP | 8 | AE・Pr | ある（上の帯の札） |
| 大きさの見本 | Cav・Bl・Fi・DR・FCP | 5 | AE | 無い |
| コマの速さ | Cav・Bl・DR（素材を入れたら固定）・FCP（空の時だけ）・CSP | 5 | AE | **無い（隠してある）** |
| 長さ・コマの範囲 | Cav・Bl・Ji・Ri | 4 | AE | **無い** |
| 背景色・透明 | Cav・Ri・Fi・Sp | 4 | AE | 無い |
| 色管理・作業色空間 | Cav・Bl・DR・FCP | 4 | Pr | 無い |
| 再生範囲・作業範囲を別に持つ | Cav・Ri | 2 | AE | 作業範囲はある |
| n コマおき | Cav・Bl | 2 | – | 無い |
| 速さを変えた時に長さを保つ | Bl・CSP | 2 | – | 無い |
| ピクセル縦横比 | Bl・DR | 2 | AE・Pr | 無い |
| 解像度の％・プロキシ | Bl・DR | 2 | AE | 無い |
| はみ出しを隠す | Ri・Fi | 2 | – | 無い |
| 見本の保存・読み込み | Cav・DR | 2 | AE | 無い |
| 置き場所（素材・書き出し・控え） | Cav・DR | 2 | – | 無い |
| モーションブラー | Cav | 1 | AE | 無い |
| 8・16 bit | Cav | 1 | AE | 無い |
| 繰り返しの種類・速さ | Ri | 1 | – | ループはある |

## C：設定の画面

| 項目 | お手本 | 数 | Adobe | Motolii |
|---|---|---|---|---|
| 見た目・テーマ | Bl・Live・CSP・DR・Sp・Fi・Cav | 7 | AE・Pr | View ▸ Skin と明暗の切り替え |
| メモリ・GPU・速さ | Cav・Bl・DR・FCP・CSP | 5 | AE・Pr | 無い |
| キーボードショートカット | Bl・DR・Fi・Ri・Live | 5 | AE・Pr | 手で書く keybindings.json |
| UI の大きさ・文字の大きさ | Bl・Live・CSP・Cav | 4 | AE・Pr | 上の帯の倍率 |
| 言語 | Bl・Live・DR・CSP | 4 | – | 無い |
| 自動保存と間隔 | Cav・Bl・DR・CSP | 4 | AE・Pr | 無い（いつも自動） |
| キャッシュの大きさ・置き場所 | Cav・Bl・DR・Live | 4 | AE・Pr | 無い（~/Library/Caches/Motolii に溜まる） |
| 音の機器 | Bl・DR・Live・FCP | 4 | AE・Pr | 無い |
| ペン・タッチ・入力 | Bl・CSP・Live・Cav | 4 | – | 無い |
| 控えの数 | Bl・DR・Cav | 3 | AE | 無い |
| 読み込みの既定 | FCP・DR・Live | 3 | AE・Pr | 無い |
| 時間の見せ方（タイムコード・コマ・秒） | Bl・FCP・Cav | 3 | AE | 無い |
| ずらす量 | Cav・Fi・DR | 3 | – | 無い |
| プラグイン・スクリプトの場所 | Bl・Live・DR | 3 | AE | 無い（効果のフォルダは決め打ち） |
| ファイルの既定の場所 | Bl・Live・Cav | 3 | – | 無い |
| 既定の長さ（静止画・トランジション） | FCP（静止画 4 秒）・DR（5 秒） | 2 | AE・Pr | 無い |
| **既定のイーズ・補間** | Bl・Cav | 2 | – | 無い |
| 取り消しの段数 | Bl・CSP | 2 | – | 無い |
| 書き出しの既定 | Cav・FCP | 2 | AE | 無い |
| 警告を戻す | FCP・Live | 2 | – | 無い |
| 試作中の機能を見せる | Cav・Bl | 2 | – | 無い |
| 動きを減らす | Bl | 1 | – | 無い |

## 公式で確かめられなかった所

Rive の設定の窓の中身、Spline の新しいシーンの大きさの窓、Jitter のアートボードの見本の全部の寸法、Blender の既定の 1920×1080・24fps（ページに書いていない）、Adobe は全部検索結果の抜き出しだけ。
