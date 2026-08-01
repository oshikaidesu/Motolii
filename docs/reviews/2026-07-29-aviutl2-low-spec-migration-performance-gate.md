# AviUtl／AviUtl2層の最低スペック移行性能ゲート（2026-07-29）

状態: **比較中／実機未測定**

## 1. 結論

AviUtl／AviUtl2利用者にとって軽さは追加価値ではなく、制作を始めるための入場条件である。
Motoliiは、高度な合成、3D、Vism、Freeze Groupを差別化として提示する前に、同じPCで行っていた
素材配置、cut、文字修正、seek、短い再生を待機儀式なしで成立させなければならない。

従って現時点の比較上は、**旧AviUtlの成熟した完成拡張スタック**と**AviUtl2固定版**を別々の
暫定基準旗とする。旧AviUtlはpluginにより入力、decode、memory、Timeline、effect、script、
RAM preview、波形、編集補助を獲得したsoftwareであり、本体／拡張編集だけを既存利用者の製品能力と
みなさない。構成、設定、責任分解、音MAD映像fixtureは
[AviUtl完成拡張スタックの性能観察](2026-07-29-aviutl-completed-plugin-stack-performance-observation.md)
を正本とする。

これは両AviUtl profileの全負荷での性能勝利や、Motoliiの敗北を実測済みとする表現ではない。
Motoliiの製品合格を「AEより速い」という曖昧な比較から切り離し、既存利用者が失ってはならない
体験へ結び付けるための基準である。

Motoliiが上積みを狙う場所は、軽いsceneだけでなく、負荷が増えた後も次を維持することである。

- UI、Timeline、Document編集をrender／decode待ちで止めない。
- cache完成や緑のbarを操作の前提にしない。
- 変更したbranchとそのconsumerだけを再評価する。
- 古いgenerationの画素を「一瞬の応答」として表示しない。
- 容量不足ではcache／先読み／作業解像度を制御し、再生期限超過ではDraft品質／解像度／表示frameを
  別loopで制御する。
- 停止後とFinalでは、同じ評価関数から正しい高品質像へ収束する。

要約すると、**旧AviUtl完成拡張スタックとAviUtl2の両旗それぞれでL0体験を退行させないことが
移行資格であり、重くなった後の粘りがMotoliiの差別化候補**である。

## 2. 比較対象について確定している事実

旧AviUtlについては、本体と拡張編集の最小構成を比較対象の全体にしない。少なくともL-SMASH Works、
InputPipePlugin、patch.aul、LuaJIT、拡張編集RAMプレビュー、波形／代表編集補助を責任別に固定した
完成拡張スタックprofileを用意し、pluginごとのversion、content hash、優先順位、設定を測定票へ残す。
全利用者が同じ構成という意味ではなく、成熟した利用環境が獲得した上限を過小評価しないためのprofileである。
AviUtl2とはABIも内部設計も異なるため、結果を混ぜない。

2026-07-29時点のAviUtl ExEdit2 2.1.2同梱文書は、Windows 10 64-bit以降、AVX2、
DirectX 11.3以上、ROV対応GPUを動作条件とし、作者の動作確認環境をWin10＋GTX 1650としている。
内部formatはpremultiplied RGBA16Fである。従って「画面を出せる任意の古いPCで動く」とは扱わない。

更新履歴には、previewのframe drop／buffering調整、画像先読みtask、映像cacheの破棄、
VRAMへの画像data cacheが記録されている。これはAviUtl2もGPU、cache、先読み、負荷時制御を持つ
現行の比較対象であり、「旧AviUtlが軽かった」という評判だけをoracleにできないことを示す。

- [AviUtl2同梱文書のcommunity mirror — 動作環境／内部format](https://docs.aviutl2.jp/)
- [AviUtl2同梱更新履歴のcommunity mirror](https://docs.aviutl2.jp/changelog)

公開文書だけでは、特定の旧世代iGPU、8GB RAM、多数MP4、個別effect stackにおけるp50／p95、
decoder surface数、memory使用量を証明できない。AviUtl2の旗もMotoliiの旗も、同一実機／素材／操作列の
比較までは製品性能保証へ昇格しない。

## 3. 最低スペックpersonaとfixture

最低スペックpersonaは二群を混同しない。直接比較群は、CapCut／Alight Motion等からPC制作へ進む人や
既存AviUtl利用者のうち、AviUtl2自身のAVX2／DirectX 11.3／ROV条件を満たすWindows notebookを使う人である。
旧AviUtlだけが動く非対応機も移行需要として残るが、AviUtl2との同一実機比較群には数えず、Motoliiの
capability admissionと実機matrixで別に観察する。ただし製品下限を特定GPU名で先に固定せず、まず次の
能力帯を実機候補として扱う。

- 2017〜2020年前後の4 core級CPU、8GB RAM、SSD。
- hardware H.264 decodeを持つintegrated GPU、または同時期のentry dGPU。
- Motoliiの必須wgpu feature／limitを満たすadapterとdriver。
- 他applicationと共有するため、GPU／RAMを全量使用可能とはみなさない。

このpersona表現は配布保証ではない。実際の最低条件、除外driver、Auto予算はINF-3と実機測定後に
別途決める。AviUtl2のROV／AVX2条件とMotoliiのwgpu条件を同一と推定しない。

比較fixtureは野心的な40動画caseから分離する。

| 段 | scene | 守る体験 |
|---|---|---|
| L0-V video日常編集 | 1080p H.264動画1本、音声1本、文字／図形、cut／seek／parameter変更 | 操作結果を待機儀式なしで表示し、軽いsceneで縮退を意識させない |
| L0-T 字幕重timeline | 動画1本、10分超のtimeline、多数の短いtext／図形object | pixel帯域だけでなくDocument／Timeline／UI応答の退行を検出する |
| L0-M 音MAD映像timeline | 完成WAV 3分、同一動画1000分割、静止画／text／図形を数百、各時刻active 1〜3 | 総Clip数とactive set、同一source再利用、編集密度を分け、移動／trim／複製／Undoを即時に返す |
| L1 overlay | 1080p動画2〜4本、transform／opacity／軽い色・key・blur | UIを止めず、必要なら自動Draft縮退して音声時刻を守る |
| L2 heavy edit | 動画8本前後、branchごとの異なるeffect | 変更branch以外を再利用し、古いgenerationを表示しない |
| L3 stress | 40 tile、40 full-screen overlap、個別effect、group effectを別々に測る | realtime保証でなく、decode／coverage／effect／memoryの破綻原因を分離する |

L0-V／L0-T／L0-Mを通らずL3の最大throughputだけが高くても移行合格にしない。逆にL3が実時間でないことだけで
L0〜L2の製品価値を棄却しない。

video素材は[Decode→Composite前提監査 §7](2026-07-29-decode-to-composite-premise-audit.md#7-原因分離fixture)
と同じ母集団を使う。少なくともlong-GOP CFRをseek／連続再生の基準にし、VFRを正しいframe identityの
別variantとして記録する。all-intra／編集proxyだけでL0合格を作らず、codec、profile、bit depth、GOP、
fps／VFR、bitrate、duration、audio、file hashを測定票へ残す。素材formatや個数の製品既定値はここで決めない。

## 4. 二段の合格面

### Gate A: 移行資格

同一の最低スペック実機、素材、project条件で、旧AviUtl最小構成、旧AviUtl完成拡張スタック、
AviUtl2固定版、Motoliiを操作し、次を別々に記録する。旧AviUtlでは全plugin／script／runtime／設定を
manifest化し、各責任を一つずつ無効化した差分測定も行う。素の本体との差だけで完成環境を代表させない。

ただし旧AviUtl／AviUtl2はWindows用比較対象であり、現行Motolii v1の動作保証はmacOS開発主機に限られる。
MotoliiのWindows製品経路と同一操作fixtureが成立する前は、両AviUtl profileとの同一実機Gate Aを
実行・公表しない。
それまではmacOS上のMotolii L0絶対値、P2aの合成budget profileによるadmission／縮退分岐、少数実GPUの
raw値だけを先行記録し、AviUtl2比較勝利とは呼ばない。
最初のmacOS自己参照値はmemory-model P2の開発主機Apple M4／16GBで取得し、OS build、電源状態、
display条件、background負荷を併記する。これは最低スペックbaselineでも配布保証でもなく、後続実機値の
比較口を作るdiagnostic baselineに限る。

- cold／warm起動、idle RSS／GPU resident bytes。
- importから最初の正しいframeまでの時間。
- seek、cut、文字変更、parameter dragから**対応generationの正しい画素**までのp50／p95。
- UI event処理のstall、audio underrun、表示frame drop。
- cold cache、warm cache、Undo／Redo直後。
- hardware decode成立、software fallback理由、decoder／surface resident bytes。
- 両製品のversion／build、preview／cache／buffer設定、導入plugin、OS／driver、内部pixel format／精度。
- 全platformで電源plan、AC／battery、測定開始時と連続run後の熱状態、測定順序、thermal throttleの
  観測可否。旧notebookのp95を冷間1回だけで代表させない。
- L0-Mの操作列と移行固有記録列は
  [完成拡張スタック観察 §5](2026-07-29-aviutl-completed-plugin-stack-performance-observation.md#5-音mad映像fixture)
  を参照し、InputPipe handle cache、patch.aul、LuaJIT、RAM preview、波形解析の有無をmanifestで固定する。
- L0-Vのcutだけで未使用の高度composite／3D moduleが発生させたper-frame work数。Motoliiは0を
  [編集／合成製品境界のM4 guard](../specs/M4-cache-and-analysis.md#実装ガード先行ツールの失敗ユーザー不満クロスチェック-2026-07-11)
  で審判する。

採択前に一つの総合scoreへ畳まない。Motoliiは少なくともL0で、既存利用者が待機、手動cache、
手動proxy、頻繁なpreview停止を新たに覚えなければならない退行を残さない。
「即時」の固定ms、許容stall、idle memory上限はG0-4／INF-3の測定後に決める。

### Gate B: 負荷後の粘り

L1〜L3では最高fpsだけでなく、負荷増加時の形を審判する。

1. 不要要求のcoalesceと古いgeneration破棄。
2. source-frame共有とbounded decoder concurrency。
3. decoder surfaceを単発利用する時は直接import候補、保持／共有時はGPU内copy後にsurfaceを即返す
   **copy-on-retain**。
4. **容量loop**はcache降格→decode先読み削減→許可時のDraft解像度降格→型付き拒否、
   **deadline loop**はDraft effect品質→自動時の解像度降格→最新frameだけの表示として分ける。
   frame dropを容量対策にせず、cache evictionをdeadline対策にしない。
5. Freeze Group／K7、全曲Draft／K8が成立した後の二周目と、cacheなし初回の分離。

zero-copy単体、VRAM cache単体、Freeze単体を勝利条件にしない。decoder pool starvation、共有memory帯域、
全画面effectのpixel走査は残るため、総frame time、memory、操作応答を同時に見る。

## 5. M3／M4への接続

この比較はM4を任意の後段最適化から、M3の移行体験を支える製品基盤として位置付ける。ただしM3とM4を
一つの実装ticketへ束ねない。

| 責任 | 接続先 |
|---|---|
| UI／Timelineを待たせない、最新値mailbox、対応generationだけを表示 | M3 Preview／GR-UI |
| ResourceLedger、外部decoder memory、hard budget、capacity縮退 | M4 K1a〜K1d |
| persistent reader、demand scheduler、source-frame共有、proxy | M4 K4前段／decode比較spike |
| 完全key、branch invalidation、Undo／Redo freshness | M4 K0／K1 |
| Freeze Group、全曲Draft coverage | M4 K7／K8 |
| audio clock、DRS、frame drop | D5／memory-model P3a |
| 実機最低条件と性能回帰 | INF-3／G0-4 |

resource cacheを外しても同じ`f(t, input, Quality)`へ収束すること、Preview／Finalで別の意味経路を
作らないことは維持する。
P2aの合成profileは速度を証明しないが、L0操作列でcapacity admissionと縮退順を守るCI fixtureへ再利用する。
K1dのpressure snapshotは、理由、解像度scale、予算使用量をGate A／Bの記録へ再利用し、別の測定正本を作らない。

## 6. 非目標と停止線

- AviUtl2の内部実装を推測してMotoliiの仕様にしない。
- 評判、単一fps、作者確認機だけでAviUtl2またはMotoliiの最低スペック勝利を宣言しない。
- L0合格のためにCPU pixel処理、同期readback、古いgeneration表示、色変換分散を許さない。
- 軽いsceneでも常時1/4 Draftにすることで合格値を作らない。
- 40本full-screen＋個別重effectの60fpsを最低スペック保証にしない。
- hardware decoderを40本常駐させることをpersistent readerの意味にしない。decoder同時数と
  surface pool値は実測前に固定しない。
- AviUtl2と異なるOS、codec plugin、出力plugin、project意味を混ぜた比較を同一条件と呼ばない。
- 旧AviUtl本体だけへの勝利を、成熟した完成拡張スタックへの移行合格と呼ばない。
- plugin構成を記録せず「AviUtl」と一括表示しない。旧AviUtl完成環境とAviUtl2を同一profileへ混ぜない。
- 旧AviUtlだけに画像処理間引き、低解像度、RAM preview済み、手動中間codecを許し、Motoliiをcold／
  full-qualityだけで測る非対称比較をしない。逆方向も同様で、各設定と準備時間を別列にする。
- Windows製品経路成立前のmacOS絶対値をAviUtl2比較値として公表しない。
- all-intra／warm cache／短尺・少objectだけをL0代表にして合格を作らない。
- この文書からDocument、plugin公開API、永続cache format、製品下限GPUを新設しない。

## 7. 現在の判定

| 軸 | 現在の判定 |
|---|---|
| 最低スペックの日常編集 | **旧AviUtl完成拡張スタックとAviUtl2を別々の暫定基準旗とする。Motolii未測定** |
| 音MAD映像の高編集密度 | L0-Mを追加。完成拡張スタックの寄与分解、Motoliiとも未測定 |
| 40動画初回の最高性能 | 両者とも同一fixtureの公開実測なし |
| 負荷時の自動縮退とgeneration freshness | Motoliiは設計済み部分あり、製品経路未成立 |
| branch cache／Freeze／全曲Draft | Motoliiの差別化候補、未実装／未測定 |
| 移行合格 | L0同一実機比較とL1以降の粘りを通すまで未成立 |

## 8. Fable 5 read-only助言の処分

初回レビューは`VERDICT: REVISE`、P0=0／P1=3だった。二段のgate、単一score拒否、未実測表示、
M3／M4分離は維持し、次を採用した。

- Gate AがMotoliiのWindows製品経路待ちであることと、それまでの自己参照測定を明記。
- capacityとdeadlineを別loopへ修正。
- decode監査と素材母集団を共有し、long-GOP／VFR／設定／精度を記録。
- 字幕重timelineをL0-Tとして追加。
- P2a合成profileとK1d pressure snapshotを既存の審判／記録機構として再利用。

助言は恒久仕様の根拠とせず、数値閾値、製品下限、decoder数、公開API、永続formatは未決のまま維持する。
再審査は`VERDICT: ACCEPT`、P0/P1=0。残った非blocking P2二件も、decode fixture manifestの属性正本共有と
macOS diagnostic baselineの機種／条件明記として回収した。
