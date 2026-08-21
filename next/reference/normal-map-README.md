# normal-map.tsv — 「普通」地図の併合台帳(裁定154)

`docs/reviews/2026-08-21-normal-map-sources/{ae,premiere,resolve,capcut}.md` の4語彙源を機械的に読み込み、単一の TSV 台帳へ併合したもの。Lottie 地図(`lottie-coverage.tsv`, 裁定68)の再現 — 全項目・機械照合可能・未判定が数えられる形を狙う。

併合作業自体は `git`・`cargo`・Web を一切使わず、4本の md をローカルで読んで行った(スクリプトは `/private/tmp/.../scratchpad/{parse_sources,merge,build_output,write_tsv}.py` に残るが、リポジトリには入れていない)。

## 列定義

| 列 | 内容 |
|---|---|
| `id` | 連番(1始まり) |
| `category` | 機能カテゴリ(下記ヒューリスティックで自動分類。誤分類あり、後段で修正想定) |
| `canonical` | 英語の代表名。alias行は統合先の代表表記、非alias行は出典の原文そのまま |
| `意味` | 日本語1行説明(出典の記述をほぼそのまま採用) |
| `ae` / `pr` / `dr` / `cc` | After Effects / Premiere Pro / DaVinci Resolve / CapCut に存在するか(1/0) |
| `freq` | 上記4列の合計。0〜4 |
| `entries(menu:shortcut:panel:pref)` | この行に統合された出典生データの内訳。`menu件数:shortcut件数:panel件数:pref件数` の固定順コロン区切り |
| `quality` | 出典の質。複数出典が統合されている場合は `;` 区切りで列挙(下記「出典の質タグ」参照) |
| `scope` | `out-of-domain?` が付くのは Fusion/Fairlight/Dolby Vision/HDR規格/プロ入出力設定等、製品固有すぎる深部機能(キュレーションではなく機械的キーワード判定。誤検出・見逃し双方あり得る) |
| `verdict` | **全行空**。後段レーンが埋める |
| `理由` | alias行は統合元の生データ一覧(`統合(alias): 製品:種別:パス>項目名 \| ...`)。scope フラグ行は簡潔な理由文。それ以外は空 |

### 出典の質タグ
- `公式(Adobe)` … Adobe公式ヘルプページ(helpx.adobe.com)から直接採取
- `二次(非公式技術抽出/2015時点)` … AE MENU_PDF(2015年、ExtendScriptでの機械抽出。Adobe公式ではないが実機メニュー構造の一次採取に近い技術文書)
- `二次(premiere全体注記)` … premiere.md 全体が「Adobe公式へのWebFetchが全面タイムアウトし、二次資料(スクール記事等)の再構成で代替した」という制約下にある(premiere.md 冒頭の注記どおり)。**Premiere由来の行は全件この注記が付く**
- `公式(BMDマニュアルPDF)` … Blackmagic Design公式リファレンスマニュアル(DR18)のミラーPDF
- `公式(BMD, v11時点)` … Blackmagic公式のキーボードショートカットPDFだが、DaVinci Resolve **11**時点のもの(現行v18〜20の完全な公式キー一覧は本文形式で再発行されていない、と resolve.md 自身が明記)
- CapCutの行は capcut.md 自身が付けた質タグ(`公式` / `単一(...)` / `一致(N件)` / 矛盾ペア等)をそのまま転記

## 束ね規則(rule 1〜4 の適用)

1. **同義束ね(alias)は保守的に**: 誰が見ても同一動作(Undo、Copy、Paste、Split/Razor、New Project/Create Project 等)だけを1行へ束ねた。迷った3系統は**あえて別行のまま残した**:
   - `Clear In` / `Clear Out` / `Clear In and Out` — 3つとも別テーブル行として温存(1つに丸めなかった)
   - `Apply Video Transition` / `Apply Audio Transition` / `Apply Default Transitions to Selection`(一括適用) / `Add Transition(generic)` — トランジション適用系を4行に分けたまま
   - `Link/Unlink`(クリップ単位のA/V同期切替、Premiere) / `Linked Selection`(選択の連動、Resolveのグローバル設定) — 名前は似るが挙動の粒度が違うため別行
2. **行の単位=利用者から見える1能力**: 同一製品内で menu と shortcut に同じ動作が重複登場する場合(例: AEの Undo が Edit メニューと General ショートカット両方に出てくる)は1行に畳み、`entries` 列で内訳を保持した。ただし全ての menu/shortcut 対応を機械的に畳んだわけではない — 判定を誤りやすい(例: AEの `File…` は Import/Set Proxy/Replace Footage/Save Frame As の4文脈で名前が衝突する)ため、**確信の持てる約80概念のみ**を明示的にマッチさせ、それ以外は出典の生行をそのまま1行1行として残した(=under-merge優先)。
3. **頻度=普通度**: `freq` 列 = ae+pr+dr+cc の1の数。CapCutの単一出典行も0.5扱いにせず1のまま、`quality` 列に出典の質を明記(rule 3 準拠)。
4. **製品固有すぎる行**: 削除せず `scope=out-of-domain?` を付けて残した(Fusion/Fairlight/Dolby Vision/HDR10+/HDR Vivid/Printer Light/Resolve Live/Cintel/easyDCP 等をキーワード検出)。判定はしていない — 後段レーンの仕事。

## 件数集計

- 入力4本の生行(データ行のみ、見出し・注記行を除く): **1,733行**(ae 785 / premiere 287 / resolve 594 / capcut 67)。このうち完全一致の重複行が1件(AE `Layer > Blending Mode: Multiply` が原文に2回登場)あり、機械的に1件へ縮約 → **1,732行**を台帳の入力とした。
  - 出典ファイル自身の「件数集計」節の自己申告(premiere 251件、capcut 63件など)は、実際にテーブルへ書かれた行数と一致しない(自己集計の誤差。premiere.md 自身が「要再確認」と書いている)。本台帳は `grep -c '^menu\t'` 等での機械カウントを正とした。
- 台帳の総行数: **1,551行**(捨て行ゼロ。1,732の生行全てがこの1,551行のどこかに対応する)
  - alias行(2製品以上、または同一製品内でmenu/shortcutを統合した行): **68行**。ここに生データ**249行**が畳み込まれている
  - 残り**1,483行**は出典の生行をほぼそのまま1対1で転記した行(passthrough)

### freq 分布

| freq | 該当行数 | 意味 |
|---|---|---|
| 4 | **6行** | 全4製品に存在。「普通」の核: `Copy`(コピー) / `Paste`(貼り付け) / `Import (media/file)`(読み込み) / `Effect Controls / Inspector`(選択要素のプロパティパネル) / `Timeline panel`(タイムラインパネル) / `New Project`(新規プロジェクト) |
| 3 | **23行** | 過半数層。大半は「CapCut側の出典が薄くて未確認」による欠落(capcut.mdが67行しかなく、他3本より1桁少ない)。Undo/Redo/Cut/Select All/Deselect All/Clear/Split/Add Marker/Zoom In/Zoom Out/Snapping/Save/Save As/Quit/Project Settings/Play・Pause 等 |
| 2 | 29行 | 2製品共通 |
| 1 | 1,493行 | 単一製品のみ(その大半はpassthrough行そのもの) |

### category 別内訳(上位)

`clip_edit` 179 / `misc` 179 / `playback` 103 / `color` 85 / `preferences` 77 / `layer_transform` 74 / `audio` 72 / `tool` 67 / `text` 65 / `panel_window` 63 / `timeline` 60 / `workspace` 58 / `mask` 54 / `view_display` 53 / `effects_animation` 50 / `camera_3d` 42 / `edit_basic` 42 / `blend_mode` 33 / `import_export` 32 / `fusion_vfx` 29 / `marker` 28 / `project` 28 / `help` 22 / `export_render` 20 / `label_color` 19 / `caption` 13 / `ai_feature` 2 / `collab` 2

`misc`(179行、全体の約12%)はキーワードヒューリスティックで分類しきれなかった行。誤分類・分類漏れは両方向にあり得る前提で、category 列は**参考情報**として扱うこと。

`scope=out-of-domain?` を付けた行: **70行**(すべて Resolve由来。Fairlight/Color/Fusionの深部機能キーワードに一致した行)。

## 既知の限界

1. **CapCut側の出典が薄い**: capcut.md は67行しかなく(他3本は287〜785行)、しかも右クリックメニュー・ショートカット・pref の大半が「単一の第三者記事(要一次確認)」または「相互に矛盾する2記事」。freq=3止まりの行の大半は「CapCutにも実在するはずだが出典側が未採取」であって「CapCutに存在しない」ではない可能性が高い。次回レーンでCapCut実機確認(過去メモ `am-hands-on-verification.md` の精神と同様)を推奨。
2. **Premiere側は全行「二次資料」**: premiere.md 冒頭の注記どおり、本セッションのWebFetchがhelpx.adobe.comへ到達できず、Adobe公式ページを直接引用できていない。`quality` 列で全premiere行に `二次(premiere全体注記)` を付与したのはこのため。Adobe公式の完全な項目順序・網羅性は未保証。
3. **Resolveのshortcut列はv11時点**: 現行v18〜20の公式キー割当て一覧はBlackmagicが本文形式で再発行しておらず、資料はDaVinci Resolve 11のPDFに依存する。メニュー構造(Trim/Timeline/Clip等の分離)と項目名が一部食い違う版差がある。
4. **AEのmenu列はMENU_PDF(2015年、AE CC 12.2.1x5)が主典拠**: Adobe公式ではなく非公式の機械抽出。後年追加の項目(Essential Graphics/Libraries/Lumetri Color・Scopes/Properties/Content-Aware Fill等)は別途WebSearchで補ったが、File/Edit/Composition/Layer/Effect/Animation/View/Helpメニューの後年差分までは追いきれていない(ae.md自身の注記どおり)。
5. **Effect個別名・動的サブメニューは対象外**: インストール済みプラグインで動的生成されるAEのEffectサブメニュー数百種、Resolveの深部Fusion/Colorノード等は、出典側の時点で既に列挙対象から外れている(=本台帳にも生データが存在しないため、当然この台帳にも現れない)。これは「捨て行」ではなく、出典4本の側で最初から採取していない領域。
6. **within-product の menu/shortcut 統合は約80概念のみ手動確認**: 同一製品内で明らかに同一動作である80弱の概念だけ確信を持って畳んだ。それ以外の同一製品内重複(存在する可能性がある)は未統合のまま出典の生行ごとに別行として残っている。過小併合(under-merge)を優先した意図的な判断。
7. **category / scope は自動分類**: どちらもキーワードヒューリスティックによる一次分類で、誤り得る。判定(verdict)は空のまま後段レーンに委ねる、という裁定の要求どおり。
