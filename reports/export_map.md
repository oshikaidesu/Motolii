# 書き出しの画面の照らし合わせ（10/25）

Motolii の書き出しの画面（skin/book/lib/workspace/top_bar.dart:246）で変えられるのはファイル名だけ。フォルダ・範囲（作業範囲）・大きさ・コマの速さ・形式（H.264 .mp4、音つき）は表示だけ。お手本 12 本の書き出しの画面を公式から集めて比べた。

## 先に分かったこと：書き出しの中身の持ち主

書き出しは GStreamer ではなく、外の `ffmpeg` コマンドを文字列で組み立てて呼んでいる（filament/kit_draw.h:41）。

- コマの速さは **60 で決め打ち**（`-r 60`、`time_is(s, i / 60.0)`）。画面の「Rate」の表示とは別に、いつも 60。
- 画質は libx264 の CRF 17、キーフレーム 60 ごと、yuv420p、faststart。音は AAC 160k。
- 音を混ぜる手順（48 kHz にそろえる、切り出す、速さ、逆、繰り返し、遅らせる、混ぜる、リミッター）を ffmpeg のフィルタの文字列として自分で書いている（kit_draw.h:25-40）。
- `ffmpeg` はリポジトリにも窓の中にも入っていない。使う人の Mac に入っていなければ書き出せない。libx264 は GPL。

**持ち主にできるもの：GStreamer**（構成表の「動画：GStreamer」）。
- GStreamer 自身が書き出しのプリセット（encoding target、`.gep`）を持ち、この Mac にも入っている：/opt/homebrew/share/gstreamer-1.0/encoding-profiles/ に mp4・webm・mkv・mov（mp4）・avi・ogv・ts・mp3・oga・flv・dvd・youtube。`gst_encoding_list_all_targets` で一覧、ユーザーの分も同じ形で保存できる。
  https://gstreamer.freedesktop.org/documentation/pbutils/encoding-target.html
- 画質のプリセット（`.prs`）：x264 の Profile Baseline・Main・High、Quality Low・Normal・High、Profile YouTube（大きさで決まる速さ：1080p は 8000 kbps など）。
  https://github.com/GStreamer/gstreamer/blob/main/subprojects/gst-plugins-ugly/ext/x264/GstX264Enc.prs
- Mac の機械の書き出し `vtenc_h264`・`vtenc_h265a`（透明つき HEVC）・`vtenc_prores`（ProRes）、AV1 の `svtav1enc` も入っている（gst-inspect-1.0 で確かめた、1.28.8）。
- `encodebin` にプリセットを渡せば、入れ物と符号化の部品は GStreamer が選ぶ。
  https://gstreamer.freedesktop.org/documentation/encoding/encodebin.html

画面の「形式」と「画質」の選択肢は、自分で並べずに GStreamer のプリセットの一覧から出せる。

## 数

| 印 | 行 |
|---|---|
| Motolii にある | 2（名前とフォルダ、出来上がりの要約） |
| Motolii にあるが決め打ち | 4（範囲、大きさ、速さ、形式） |
| 無い | 61 |

## 照らし合わせ

お手本の略：Rs＝DaVinci Resolve 20、FCP＝Final Cut Pro、Ab＝Ableton Live 12、Bl＝Blender、Cv＝Cavalry、Ji＝Jitter、Rv＝Rive、Fg＝Figma、Sp＝Spline、CSP＝クリスタ、HB＝HandBrake（GPLv2、形だけ）、AE＝After Effects・Media Encoder（比べるだけ。Adobe のヘルプは取れず、一部は第三者）。

### 範囲・時間
| 項目 | お手本 | Motolii |
|---|---|---|
| 全部か、作業範囲・In/Out か | Rs・Ab・Bl・Cv・CSP・AE | 作業範囲（決め打ち） |
| 始まりと終わりを決める | Rs・Ab・Bl・Cv・CSP・AE | 無い |
| 秒で長さ | Rv・Sp | 無い |
| 今のコマだけ（静止画） | Cv（FCP は「今のコマを保存」） | 無い |
| コマを飛ばす（ステップ） | Bl・Cv・AE | 無い |
| 書き出す時に長さを変える | Bl・Cv | 無い |
| ループ用（終わりの余韻を頭に） | Ab | 無い |

### 大きさ
| 項目 | お手本 | Motolii |
|---|---|---|
| 大きさを選ぶ（プリセット・自由） | Rs・FCP・Bl・Rv・Ji・Sp・CSP・HB | 文書の大きさ（決め打ち） |
| 倍率（%・2x） | Bl・Cv・Fg・Ji | 無い |
| 縦長に切り替え（TikTok など） | Rs | 無い |
| 一部だけ（切り抜き） | Bl・AE・CSP | 無い |
| 元より大きくしない | FCP・HB | 無い |

### コマの速さ
| 項目 | お手本 | Motolii |
|---|---|---|
| 速さを選ぶ（8〜240、Custom） | Rs・Bl・Cv・Ji・Rv・Sp・CSP・HB | 表示は文書の fps、中身は 60 で決め打ち |

### 形式
| 項目 | お手本 | Motolii |
|---|---|---|
| 入れ物と符号化を選ぶ | 全部 | H.264 .mp4（決め打ち） |
| 連番画像（PNG・EXR など） | Rs・Bl・Cv・Rv・Sp・CSP・AE | 無い |
| GIF・APNG・WebP | Cv・Ji・Rv・Sp・CSP | 無い |
| Lottie・SVG（動く絵として） | Cv・Ji・Rv・Fg | 無い |
| ProRes（透明つき） | Rs・FCP・Bl・Cv・Ji | 無い |
| 音だけ・絵だけ | Rs・FCP・Cv・Ab | 無い |
| 色の深さ 8・10・12 | Rs・Bl・Cv | 無い |

### 画質
| 項目 | お手本 | Motolii |
|---|---|---|
| 画質のプリセット（高・中・低） | Bl・Cv・Ji・Rs・HB | 無い（CRF 17 決め打ち） |
| CRF・RF の数 | Bl・HB | 無い |
| ビットレート | Rs・Bl・Cv・Rv・HB | 無い |
| 1 回か 2 回か | Rs・FCP・HB | 無い |
| 速さ（符号化の速さ） | Bl・HB | 無い |
| キーフレームの間 | Rs・Bl | 無い（60 決め打ち） |
| 描く質（アンチエイリアス） | Cv・AE | 無い |
| 分けて速く書き出す | FCP | 無い |

### 透明・色
| 項目 | お手本 | Motolii |
|---|---|---|
| 透明の背景（アルファ） | Rs・Bl・Cv・Ji・CSP・AE | 無い |
| ストレートかプリマルチか | Rs・AE | 無い |
| 色空間・ガンマ・プロファイル | Rs・FCP・Bl・Fg・CSP | 無い |
| ビデオレベルかフルか | Rs | 無い |
| ディザ | Bl・Cv・CSP | 無い |

### 音
| 項目 | お手本 | Motolii |
|---|---|---|
| 音を入れる・入れない | Rs・Cv・Bl・Ab | いつも入れる |
| 音の符号化・サンプルレート・ビット深度 | Rs・Bl・Cv・Ab・CSP | 無い（AAC 160k 決め打ち） |
| チャンネル（モノ・ステレオ・5.1） | Bl・FCP・Ab・CSP | 無い |
| 音量をそろえる（ラウドネス） | Rs・Ab | 無い（リミッターだけ） |
| トラックごとに別のファイル | Ab・FCP・Rs | 無い |

### 名前と場所
| 項目 | お手本 | Motolii |
|---|---|---|
| 名前とフォルダ | 全部 | ある（フォルダは表示だけ） |
| 名前の差し込み（文書名・日付・大きさ・fps） | Cv・HB・Bl・AE・Rs | 無い |
| 頭・後ろの文字、番号 | Fg・CSP・Rs・Cv | 無い |
| 名前の `/` でフォルダ | Cv・Fg | 無い |
| 上書きしないように名前を変える | Rs・Cv・HB | 無い |
| 上書きする・しない | Bl・AE | 無い（いつも上書き） |
| 出来上がりの場所の見本 | Cv・Rs | 無い |

### プリセットと行き先
| 項目 | お手本 | Motolii |
|---|---|---|
| 用意されたプリセット | Rs・HB・Bl・FCP | 無い（GStreamer のプリセットが使える） |
| 自分のプリセットを保存・既定にする | Rs・Cv・HB・CSP・AE | 無い |
| 前の設定を覚える | FCP・Cv | 無い |
| SNS 向け（YouTube・TikTok・Vimeo）・そのまま上げる | Rs・Cv・FCP | 無い |
| 題・説明 | Rs・FCP | 無い |
| マーカーから章 | Rs・FCP・HB | 無い |
| 字幕（焼き込み・別ファイル） | Rs・FCP・HB | 無い |
| 情報を焼き込む（コマ番号など） | Bl・Rs | 無い |

### まとめて・続けて
| 項目 | お手本 | Motolii |
|---|---|---|
| 書き出しの列（キュー） | Rs・Cv・Rv・HB・AE | 無い |
| まとめて書き出す | FCP・Fg・Ji・Cv | 無い |
| 1 回で別の形式もいっしょに | Rs・AE・Fg・FCP | 無い |
| 裏で書き出して作業を続ける | FCP・Cv・Rv | 無い（書き出し中の扱いは要確認） |

### 進み具合
| 項目 | お手本 | Motolii |
|---|---|---|
| 進み具合 | 全部 | 上の帯に「Writing X …」 |
| 残り時間 | Rs・AE | 無い |
| 出来上がりの大きさの見込み（書き出す前） | FCP | 無い |
| 出来上がりの要約（大きさ・fps・音・長さ） | FCP・Rs | ある（表示だけ） |
| Dock のアイコンに進み具合 | Rs | 無い |
| やめたら作りかけのファイルを消す | Ab | 無い |

### 終わった後
| 項目 | お手本 | Motolii |
|---|---|---|
| 知らせる（通知・音） | FCP・HB・AME・AE・Cv | 上の帯に「Written: X」 |
| プレイヤーで開く・Finder で見せる | FCP・Rs | 無い |
| 書き出した物を文書に戻す | AE・Rs | 無い |
| 終わった後に動く手順 | Rs・Cv・HB | 無い |
| 書き出しの履歴 | FCP・Rv | 無い |

## 窓の中で今すぐ分かるまちがい

- Esc で閉じない（Apple の決まりでは Esc で取り消し）
- 「Rate」に文書の fps を出しているが、中身はいつも 60
