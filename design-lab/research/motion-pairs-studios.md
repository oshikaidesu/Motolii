# 意図 → 動きの対 (スタジオ事例の調査)

基準は 1 つだけ: 「利用者がしたいこと」と「画面で動く物」がどれだけ直接一致するか。

## 制約と証拠の水準 (先に読む)
- 動画は見ていない。全行 **text-described**。still-seen は 0 件(画像は 1 枚も見ていない)。
- 本文は WebFetch(小モデルによる要約)経由。つまり「ページの言葉」の二次要約であり、原文の逐語ではない。要約が盛った可能性は排除できない。
- 多くのスタジオ頁(Dunk & Loop / Snacpac / MHF / Samim)は哲学と案件名のみで、具体的な「意図→動き」を書いていない。そのため具体対の大半は Impractical・advids・Tubik の記述から取った。スタジオ名がある行はそれ単体では動きの記述が薄い。
- 直接度 1-5: 5 = 意図そのものが物の動きになっている / 1 = 意味上の必然なし。点数は調査者の判断。

## (1) 対の表

| # | 意図(動詞) | 画面の動き | 出典(URL / 制作者・作品) | 直接度と理由 | 証拠 |
|---|---|---|---|---|---|
| 1 | 散らばりを集める (Group / Gather) | 散乱ファイル・チームが「混沌と飛び回り」、1 つのハブに集約、承認フローが収束 | https://advids.co/blog/saas-product-animation-video / WoodWing | 5: 集める操作そのものが集まる動き | text-described |
| 2 | 複数データを統合 (Group) | 複数のデータ流が中央ハブに漏斗状に流れ込み、行き先ごとの容器へ | 同上 / Segment | 4: 合流の動き。ただし「容器」は比喩 | text-described |
| 3 | 断片を統合 (Group) | 分断データフィードが 1 つの系に畳み込まれる | https://advids.co/blog/animated-interface-ui-videos / ITRS Geneos | 4: 集約=収束。対象が抽象的な「フィード」 | text-described |
| 4 | 別々のデータ源を 1 画面に (Group) | 散在するデータ流が 1 つのダッシュボードに収束 | 同上 / FactSet | 4: 同上 | text-described |
| 5 | 混乱を整頓 (Sort/Arrange) | スプレッドシート・紙の混沌が溶けてきれいな UI になり、資格情報が現れる | 同上 / AG5 Skills Management | 3: 「溶ける→現れる」は変換の比喩。操作の結果が動きになるのは半分 | text-described |
| 6 | 手動を自動に (Arrange) | 手作業の混沌 → 自動スケジュール格子、空き時間が最適配置へ動く | 同上 / Petal | 4: 物が所定位置へ入っていく=配置という意図そのもの | text-described |
| 7 | 接続 (Connect) | 光るデータ線が機器から銀行系へ延びて繋ぐ | https://advids.co/blog/saas-product-animation-video / Deposita | 4: 線が延びて繋がる=接続。ただし操作の UI ではなく概念図 | text-described |
| 8 | 関係を辿る (Connect/Trace) | ベクターが接点間の経路を描いて辿る | https://advids.co/blog/saas-ui-video / Mediahawk | 4: 辿る線が経路そのもの | text-described |
| 9 | 関係を示す (Connect) | グラフ技術が管轄間の関係をマップ化 | animated-interface-ui-videos / PwC Pillar Two | 3: 線で関係を描くが、操作ではなく表示 | text-described |
| 10 | 拡散・展開 (Spread) | 位置ピンが地球儀上で増えて広がる | 同 / Antom | 3: 広がる=対応業種が広がる。操作ではない | text-described |
| 11 | 分解・分類 (Split) | フィードバック文が話題カテゴリに分解、感情色が各コメントに広がる | 同 / SurveySparrow CogniVue | 4: 分ける=バラけて別の場所へ行く | text-described |
| 12 | 流れ込む (Send/Sync) | 家庭機器のバイタルが医師ダッシュボードへ流れる | 同 / NextGen HealthBridge | 4: 送るは流れる。ただし概念図 | text-described |
| 13 | 生成・構築 (Build) | AI が作った UI ブロックが次々と「連続して組み上がる」(cascade) | https://impractical.ai/launch-videos/notion-3 / Notion 3.0 | 4: 「作る」が組み上がる動き。ただし 1 要素ずつではなく一括演出 | text-described |
| 14 | 入力する (Type) | プロンプトにタイプライター表示 | 同 / Notion | 3: 打つ=文字が出る。意図そのものではなく人間らしさ演出 | text-described |
| 15 | 注目させる (Focus) | UI の一部へゆるい寄り(punch-in) | 同 / Notion、https://impractical.ai/launch-videos/cognition-mac-vm-integration / Devin | 2: カメラ操作であり、機能の意図ではない | text-described |
| 16 | 因果を示す (Cause→Effect) | コード生成から Slack 通知へハードカット | 同 / Devin Gets a Mac | 2: カットの編集技。物の動きですらない | text-described |
| 17 | 待ち時間を消す (Skip) | アイドル区間を削除 | 同 / Devin | 1: 動きが無い(削除) | text-described |
| 18 | バネ設定を見せる (Ease) | 要素がオーバーシュートで入り、拍で収まる。「ユーザーが調整できるバネ値そのもの」を実演 | https://impractical.ai/launch-videos/figma-motion / Figma Motion | 5: Ease の意図=動きの質そのもの(イージングを見せる=イージングで動く) | text-described |
| 19 | 静から動へ変換 (Animate) | 静的部品が一瞬で動く版へマッチカット | 同 | 4: 変化の差分そのものが見える | text-described |
| 20 | 新パネルを示す (Reveal) | 新しいタイムライン/プロパティの行が 1 行ずつ順に現れる(stagger) | 同 | 3: 順次=視線誘導。「新しい行が増えた」意味との対応は中 | text-described |
| 21 | 設定と結果を同時に示す | 動くデザインの横に「Motion Tokens」パネル(Brand: Linear / Snappy / Playful)を置く | https://www.moonb.io/blog/product-launch-video / Figma Motion | 2: 数値パネルを並べるだけで、動きの意味との必然は薄い(Strength→スライダーに近い) | text-described |
| 22 | 時刻を選ぶ (Pick time) | 空・星・太陽/月が回転、選んだ時刻の空に画面がなる | https://blog.tubikstudio.com/case-study-toonie-ui-animation-development/ / Toonie Alarm (Tubik) | 5: 値(時刻)がそのまま景色の動きになる。数値は 2 番目 | text-described |
| 23 | 有効/無効を切る (Toggle) | スイッチが太陽になり、ON で光線が回る | 同 | 4: 状態=太陽が出ているか。意味が絵に入っている | text-described |
| 24 | ランダム化 (Random) | 端末を振ると ステッカーがシャッフル/散らばる | https://blog.tubikstudio.com/creative-motion-12-concepts-of-interface-animation/ / Toonie Alarm Shake | 5: 振る→ばらばらになる。意図と動きが同一 | text-described |
| 25 | 完了/削除 | タスク完了/削除演出、統計カウンタ更新 | 同 / Upper App | 3: 具体の動きの記述が無く、対の中身は不明(記述不足) | text-described |
| 26 | 選択→展開 (Expand) | カードが広がって詳細になる(shared element) | 同 / Night in Berlin; 検索結果 https://blog.tubikstudio.com/creative-motion-12-concepts-of-interface-animation/ | 4: 「詳しく見る」=広がる | text-described |
| 27 | 見つける (Search/filter) | 資格のある人が検索結果でハイライトされる | animated-interface-ui-videos / AG5 | 3: ハイライトは結果表示で、絞る動き(不適合が退く)の記述は無い | text-described |
| 28 | 探索・発見 (Discover) | 既知/未知の資産がネットワークマップに動的に現れる | 同 / Mandiant ASM | 3: 現れる=見つかる。操作は不在 | text-described |
| 29 | 強さ・危険度 (Strength) | 深刻度指標が脈動、警告がダッシュボードに連鎖 | 同 / Mandiant, Telesoft | 2: 脈動は一般的な警報記号。強さそのものの動きではない | text-described |
| 30 | 成長 (Grow) | ロケットが上昇 | Tubik 12 concepts / Referanza; advids / Giffy | 1: 比喩の常套。必然性なし | text-described |
| 31 | 速さを伝える (Fast) | 拍打ちのハードカットと「加速が速く減速が長い」シャープなイージング | https://impractical.ai/launch-videos/supabase-jev-ai / Jev AI | 2: 速度=動きの質だが、機能の意図(何をしたいか)とは別 | text-described |
| 32 | 製品が動く (Intro) | キネティック・タイポが機能名を導入 | 同 / Jev AI、Unily https://mhf-creative.com/work/unily-futures-product-launch-video/ | 1: 文字が踊るだけで意味の対応なし(Unily 頁は具体的な動きの記述自体が無い) | text-described |

## (2) 一撃の対 (5/5)
- 集める: 散乱ファイル → 1 つのハブへ集約(WoodWing, #1)
- 乱す(Random): 端末を振る → ステッカーが散る/混ざる(Toonie, #24)
- 値を景色にする: 時刻を選ぶ → 空と天体がその時刻になる(Toonie, #22)
- イージングを見せる: バネ値の話 → 要素がそのバネで動く(Figma Motion, #18)
(いずれも text-described。動画は未確認)

## (3) 派手だが恣意的 (直接度 1-2)
- ロケット上昇 = 成長(#30): 比喩の常套句。意味上の必然は無い。
- キネティック・タイポ(#32): 機能と文字の動きに対応無し。
- punch-in(#15)・ハードカット(#16)・待ち時間削除(#17): 編集技で、物の動きではない。
- 脈動する深刻度(#29)・Motion Tokens パネル(#21): 「強さ→スライダー/数値表示」型に近い。
- 注: これらの根拠は Impractical・advids の要約文のみ(動き自体は未視認)。

## (4) モーショングラフィックスエディタの自操作に関わるもの
正直に対応づけできる範囲のみ。多くは「製品 UI の演出」であり、エディタ操作の動きではない。
- Random: #24 (振る→散る)。ほぼ唯一の素直な対応。
- Ease: #18 (バネ値を動きで見せる)。Figma Motion はエディタ自身の機能。
- Stagger: #20 / #13 (行・ブロックが順次現れる)。ただし「見せ方の stagger」であり、操作の意図としての Stagger との対応は未確認。
- Group: #1-#4 (収束)。直接。ただし出典はデータ統合の概念図で、ユーザーが複数オブジェクトをグルーピングする操作の記述ではない。
- Connect: #7, #8 (線が延びて繋ぐ/辿る)。同上、概念図。
- Search/filter: #27 の弱い対応のみ(ハイライト)。
- Wave / Repeat / Step / Range / Copy-one-aspect: 該当なし。

## (5) 開けなかった/中身が薄かった出典
- 開けない: whatstudio.digital (404、/work も 404)、impractical.ai/launch-videos/raycast-new-raycast (404、スラグ推測違い)。
- 開けたが具体の動きの記述なし: dunkandloop.com (哲学のみ。Reely/Oppo は案件名だけ)、snacpac.biz (案件ごとの概念文のみ)、mhf-creative.com SaaS 頁と Unily 案件頁(動きの記述なし)、samimstudios.com LocAI (環境の言葉のみ)。
- 検索で見つからない: COSMOTIONS、Riser PV、SaaS Animate UI film (特定できず)。
- Impractical 一覧頁は題名とカテゴリのみ。個別頁 3 本(Notion 3.0 / Devin / Figma Motion)と Jev AI のみ取得。Linear Agent、Framer 3.0 等は未取得。
- 取得は全て二次要約。画像(still)は 1 枚も見ていないため、still-seen の項目は無い。
