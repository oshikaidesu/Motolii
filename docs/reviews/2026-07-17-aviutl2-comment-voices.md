# AviUtl2動画コメント欄 — 統一できない利用者の声

日付: 2026-07-17

状態: **一次声の観察台帳。反対側レビュー前。設計根拠にしない。** 本文書は[レビュー文書の規律](README.md)に従い、公開コメントから観測した事実とMotoliiへの仮説を分離する。コメントは編集・削除・並び替えされ得る非恒久資料であり、公式仕様や再現試験の代わりにしない。

対象: [「AviUtl2はなぜ素晴らしいか、ミニマリズムの観点で｜映像学区」](https://www.youtube.com/watch?v=tyhPv4i0Q8s)

## 1. なぜ記録するか

動画本編は「軽快さ」「必要十分」「起動の速さ」「小さな本体+拡張」という一つの主張に編集されている。コメント欄には、その主張へ回収できない実利用者の声が同時にある。

- 軽快だから短い録画をすぐ残せる人と、4K・多重動画・低スペック機ではpreviewが止まる人。
- 一つでほどほどに何でもできることを好む人と、DaVinci Fusion、Premiere、YMM4等の専門性から戻れない人。
- plugin catalogで導入障壁が下がった人と、管理者実行、初期MP4非対応、安定版plugin待ち、旧資産非互換で移れない人。
- 新UIやframe単位audio scrubbingで初めて理解できた人と、旧AviUtlのtimeline意味から変わったため戻った人。
- AIなしの必要十分を評価する人と、歌詞と絵からAIが即座にMVを作ることを望む人。

この不一致はnoiseではない。制作物、素材、既存資産、習熟、OS、機材、支払済みtoolchainが異なるため、同じ製品特性が利点にも移行障壁にもなることを示す一次声である。

## 2. 取得範囲と限界

- 2026-07-17にYouTubeの「新しい順」で表示された親コメント34件を取得した。画面の総数表示は80件で、残りには返信が含まれる。
- 返信は表示できたthreadを読み、悩みが解消したか、回避策・別tool・将来updateへ送られたかを確認した。YouTubeの遅延読み込みと折り畳みのため、全返信の完全取得は保証しない。
- `like`数は時点依存なので、声の重要度や母集団比率へ変換しない。
- 動画の視聴者が自発的に書いたコメントであり、AviUtl2利用者全体の代表標本ではない。好意的動画のコメント欄というselection biasがある。
- 発言の技術的正しさは未検証である。「GPU支援が中途半端」「互換がある」等は利用者認識として記録し、実装事実として扱わない。

### 2.1 Motoliiへの転移フィルタ

一次声として貴重であることと、Motoliiが解くべき要求であることは別である。Motoliiは[concept.md](../concept.md)で**汎用NLEではない**と決定済みであり、次の声は観察台帳には残すが、Motoliiの機能要求・性能目標・UI導線へ転移しない。

- 友人との録画を素早く切り抜く、Vlogをcutする等、**単純なcut編集そのもの**の快適さ。
- 字幕主体の実況を一つのtoolで完結する、台詞入力を高速化する等、**YMM4等の専用toolが正面から解く用途**。
- 多数の実写素材、BGM、効果音を差し替えながら番組を組む等、**Premiere / Resolve等のNLEが正面から解くworkflow**。
- 「何でも一つでできる」こと自体を目的とする要求。Motoliiは専門toolを置換する万能編集ソフトを目指さない。

転移候補に残すのは、同じ発言が次のMotolii本来の経路へ直接重なる場合だけである。

- MV・motion graphicsの多層合成、keyframe、easing、音同期、scrub、preview。
- 表現pluginの発見・導入・version・安定性・欠落診断・作品再現。
- MV制作資産の持続性、外部toolからの素材handoff、OS/GPU/解像度をまたぐ可搬性。
- 「思いついた表現を試す」までの起動・操作反映・比較・Undoの摩擦。

たとえば「録画のcutがすぐ終わる」は**不採用**だが、その発言中の「起動が速い」は、MV制作者が最初の表現を試すまでの摩擦として別の一次声・実測で再確認できた時だけ転移候補になる。用途の異なる成功談から性能要求だけを都合よく切り出さない。

## 3. 声の台帳

### V1. 軽さは一つの数値に統一できない

| 観測した声 | 同時に残る反対側 | 一次声 |
|---|---|---|
| 起動の速さと手軽さにより、友人との録画をすぐ切り抜いて残せる | **Motoliiへの転移は不採用**。単純cutはNLE領域であり、短い軽作業の成功をMV合成性能の根拠にしない | [短い録画をすぐ編集](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzkB-orOvZyIcTXqLV4AaABAg) |
| 音MADには非常に使いやすい | 動画を多数重ねるとPremiereよりpreviewが重いという体験が同じ発言にある | [用途で軽重が逆転](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgyWZcMcp5F0hrhUwoZ4AaABAg) |
| RAM preview相当を求める | 返信では標準のcache再生手順が案内されたが、正常表示しない場合もあると注記された | [重い作品でcache再生を探索](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=Ugyd-YIcYbsW3kPpAg14AaABAg) |
| 4K編集が重い | 「AviUtl2は軽い」という総称では、この条件差を説明できない | [4K編集の重さ](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzuaQAtggvOtsDSYzZ4AaABAg) |
| 低スペック機ではpreviewが固まる | 返信は撮影時bitrateを下げる回避策で、editor側の解消ではない | [機材条件で停止](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzWdv8-QFOdspp4HBJ4AaABAg) |

**観測**: 「軽い」は少なくとも起動、空project、scrub、短い合成、多層動画、4K、cache再生、素材codec/bitrateに分解しないと、互いに真である声を一つへ潰す。

### V2. 一体型の身軽さと、専門toolchainの強さ

| 観測した声 | 同時に残る反対側 | 一次声 |
|---|---|---|
| Adobeの複数アプリを選び分けず「とりあえず起動」で広い作業を始められる | **万能一体型要求は不採用**。同じ発言のtimeline比較も、MVの多層合成・音同期へ重なる論点だけを分離して再審判する | [timeline比較の長文観察](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzIPnRxYKtUBr7TUBp4AaABAg) |
| ほどほどに何でもできることを評価 | 従来transitionを失い、DaVinci Fusionの能力を得たことで戻れない | [AviUtl2からDaVinciへ移行](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=Ugy4KTzYUTiV__OiZhZ4AaABAg) |
| YMM4からAviUtl2へ移る価値を問う | **実況編集要求は不採用**。字幕をYMM4、motion部分を別toolとする分業例は、Motoliiが専用toolを置換しない境界の観察に限って使う | [YMM4との使い分け相談](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgywYr2uTr0S3IEPlV14AaABAg) |
| YMM4→旧AviUtlという10年のworkflowを維持したい | **YMM workflow互換は不採用**。ただし長期資産を捨てられず移行しない構造は、Motolii自身が将来schemaを変える際の反例としてのみ残す | [長期workflowと互換障壁](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgwxBQ9hdH87VyDpKR54AaABAg) |
| 英語利用者がAviUtl2へ移り始めた | motion blurはVegas ProやAviSynthで補うと述べ、単体完結ではない | [HitFilmからの移行と外部補完](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgyRCFfLbryckIs_buV4AaABAg) |

**観測**: 「一つで完結」は全員の目標ではない。単体の能力だけでなく、既存toolchainのどこへ入り、何を往復させられるかが移行可能性を決める。

### V3. 拡張性は自由と管理負荷を同時に生む

| 観測した声 | 同時に残る反対側 | 一次声 |
|---|---|---|
| catalogを知ったことが導入の決め手 | catalogが解くのは発見・導入・更新であり、pluginの安定性やproject互換ではない | [catalogで導入決定](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=Ugygagdb2sop8XDecJp4AaABAg) |
| 使う機能だけ入れられるため無駄がない | 本人も初期の必須plugin導入に時間が掛かると述べる | [選択導入と初期負担](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgwrCJcTDcXujeVDVZJ4AaABAg) |
| pluginが増え、旧版より万能になったと感じる | 旧plugin・script・projectをそのまま使えるかは別問題 | [plugin増加で再評価](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgyhGlFJqynqLTsHmLp4AaABAg) |
| UIがよいので旧版から移りたい | 旧script群への依存とproject作り直しが移行を止める。返信も「移植は進んだがprojectは再利用不可」とする | [旧資産互換への切実さ](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=Ugwlwojbpoj4yj1zPrx4AaABAg) |
| PSD対応pluginは存在するという返信 | 利用者は「存在」ではなく安定版であることを移行条件にしており、回答者間でも成熟度認識が割れる | [PSD安定性がボトルネック](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzMCqdEzgMX9nUVmxx4AaABAg) |
| 管理者実行でplugin errorを回避 | 初心者を弾くのもよいという返信と、regressionなので直してほしいという返信が衝突する | [管理者実行問題](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgyJBQpRDZNo4Szhklh4AaABAg) |
| 初期状態でMP4非対応なのを旧来性込みで面白がる | 新規利用者にとっては入力導線の欠落であり、愛着ある冗談だけでは評価できない | [初期MP4非対応](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgxHW2kBxP-MNawVPVR4AaABAg) |

**観測**: pluginの「有無」「導入可能」「更新可能」「安定」「作品で再現可能」「将来も開ける」は別の保証である。catalogだけで拡張後の身軽さを語らない。

### V4. Timelineの合理性はworkflowごとに異なる

| 観測した声 | 同時に残る反対側 | 一次声 |
|---|---|---|
| AEはlayerが増え、ノートPCでは表示量が問題になるという驚き | AE側はprecomposeで整理するが、別timelineへの分断を伴う | [AEのlayer増加への反応](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzNvxZjRxMJWxs7ET94AaABAg) |
| 旧AviUtlでは同一layerに別objectを置けた | AviUtl2の意味変更に慣れず旧版へ戻った。新しさが常に移行改善ではない | [同一layer意味の断絶](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=Ugxl3eU6vWDp_z8lgEd4AaABAg) |
| Premiereのaudio/video分離は差し替えやBGM変更が容易 | 時刻範囲の項目をまとめて選ぶ操作では統合timelineが有利という同一話者の比較 | [分離と統合の得失](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzIPnRxYKtUBr7TUBp4AaABAg) |

**観測**: 「優れたtimeline」を一つの階層・一つのpacking規則で証明できない。素材交換、時間範囲編集、Z順、表示密度、旧版からの筋肉記憶を別々に審判する必要がある。

### V5. 学習、言語、OS、既存投資は機能表の外にある

| 観測した声 | 同時に残る反対側 | 一次声 |
|---|---|---|
| 旧AviUtlの使いづらさで動画編集自体をやめた | 新版が改善したかを試す前に、過去体験が参入を止めている | [旧版体験による離脱](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgyMh-aC-ydRPMiaK8V4AaABAg) |
| easingを理解できず、解説を見ても挫折中 | 無料・軽快でも「図形を意図どおり緩急付きで動かす」First Resultへ届かなければ参入障壁は残る | [easing学習の挫折](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=Ugxu3rqEhwT4oKAVekx4AaABAg) |
| 英語利用者は新版UIでmidpointを初めて理解し、frame単位audio scrubbingで移行を決めた | localizationだけでなく、時間編集の概念表現と音の操作感が移行条件 | [英語利用者の理解改善](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgyRCFfLbryckIs_buV4AaABAg) |
| Macなので使えない | Wine案は不安定という返信があり、理論上の起動可能性は制作環境の成立を意味しない | [Mac/Windows境界](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgwrODV59eKl3hdQT3R4AaABAg) |
| 日本中心のtoolでは世界標準へ接続できないという批判 | 同じ欄には英語利用者の移行例もあり、「日本語圏のみ」と即断もできない | [国際性への批判](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzuHL177-oDUyUn5uJ4AaABAg) |
| 高価なAE plugin群を背負い、精神的に重い | 最大能力は高く、支払済み資産とworkflowがあるため、身軽さだけでは移行できない | [有料plugin資産の精神的負担](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgygXVlOIlGotKophdN4AaABAg) |

### V6. 「必要十分」の中身も一致しない

| 観測した声 | 同時に残る反対側 | 一次声 |
|---|---|---|
| AI高機能toolを使わない本人には満足でき、無料が大きい | 別の利用者は歌詞とillustrationからAIが即座にMVを作るtoolを最も欲しいとする | [AIなしで満足](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=Ugz46XhxTsKgS3_CT7F4AaABAg) / [AI生成を希望](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgwErr84e8yRyIelGDN4AaABAg) |
| motion graphicsを調べれば洒落たものも作れ、初心者の入口になる | 「作れないことはない」は、発見可能性や標準導線が十分という意味ではない | [入口としての可能性](https://www.youtube.com/watch?v=tyhPv4i0Q8s&lc=UgzAQJhTpgd9Mp3-szR4AaABAg) |

**観測**: 必要十分は製品が一方的に固定する機能集合ではない。作品種別と「自分で制御したい範囲」により、automationは不要機能にも必須機能にもなる。

## 4. Motoliiへ転移する前の問い

以下は設計決定ではない。一次声から生じた、既存審判へ追加で尋ねるべき問いである。

1. **性能条件を分解したか**: 起動、scrub、単一動画、多層動画、vector、4K、低VRAM、codec/bitrate、cache warm/coldを「軽い」に畳んでいないか。
2. **First Beatの次を測るか**: 最初の成功だけでなく、easing、複数案比較、重い区間、既存作品の持込みで離脱しないか。
3. **plugin保証を分解したか**: 発見、導入、権限、互換、安定channel、欠落診断、project再現、update/revertを別々に審判できるか。
4. **移行はimportだけか**: 旧projectの完全変換が無理でも、素材、timing、easing、preset、plugin設定、外部toolとの往復のどこまでを救えるか。
5. **対象外workflowを混ぜていないか**: timeline審判の中心はMV・motion graphicsとする。字幕主体、実写多素材、短い切り抜きで高評価でも採用理由にせず、逆にそこで不利でもMotoliiの失敗とは数えない。
6. **単体完結を強制していないか**: YMM4、DAW、Blender、NLE等と分業する人を「未移行」と誤認せず、明示的なhandoffとして扱えるか。
7. **存在と信頼を混同していないか**: pluginがあることと、制作本番で安定版を採用できることを同じ完了にしていないか。
8. **理論上対応と制作可能を混同していないか**: Wine等で起動できること、低bitrateなら動くこと、回避手順があることを製品側の解決として数えていないか。
9. **非目標を正直に言えるか**: MotoliiよりPremiere、YMM4、DaVinci等が適する条件を、製品失敗ではなく用途境界として説明できるか。

## 5. 現時点の扱い

- [concept.md](../concept.md)の「軽快さは創作の試行回数を守る」と整合する声はあるが、それだけを抽出しない。重い条件と移行不能の声を同じ重さで残す。
- [ui-concept.md](../ui-concept.md)のFirst Beat仮説には、easingを理解できず挫折する声と、audio scrubbingが移行決定になる声を反例・補助観察として渡せる。ただし数値KPIやUI仕様へ直接焼かない。
- [extensible-core-model.md](../extensible-core-model.md)の拡張後の管理負荷には、catalog成功例だけでなく、管理者権限、安定版待ち、旧資産非互換、初期入力対応を併読させる。
- 単純cut、Vlog、実況字幕、番組型の実写多素材編集に関する声は、Motoliiのbacklog・M3操作・性能KPIへ転記しない。競合へ譲る用途境界の確認にだけ使う。
- 性能、timeline、plugin lifecycle、portabilityへ採用する場合は、公式仕様・実機再現・独立した反対側レビューで事実と転移条件を確認する。

## 6. 失ってはいけないもの

このコメント欄の価値は多数決ではない。「AviUtl2は軽い」「catalogが解決した」「一つで完結する」という綺麗な結論へ寄せるほど、一次声としての価値は下がる。

残すべきなのは、同じtoolを前にしても次が一致しないという事実である。

- 何を軽いと感じるか。
- どこまで一つのtoolで行いたいか。
- どの旧資産を捨てられないか。
- pluginの何を信頼条件にするか。
- timelineの何を合理的と感じるか。
- automationを助けと感じるか、不要と感じるか。
- 回避策を解決とみなすか、製品側の宿題とみなすか。

Motoliiへの転移で守るべき態度は、これらを一つのpersonaや一つの「正しいworkflow」に統一しないことである。
