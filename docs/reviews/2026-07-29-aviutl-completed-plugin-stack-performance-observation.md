# AviUtl完成拡張スタックの性能観察（2026-07-29）

状態: **観察／比較中／実機未測定**

## 1. 観察の訂正

AviUtlを移行比較へ使う時、`aviutl.exe`または拡張編集単体を製品能力の全体として扱わない。
AviUtlは小さい本体、拡張編集、入力／出力plugin、binary patch、Lua runtime、script、preview bake、
解析／編集補助を利用者が組み合わせて完成させるsoftwareである。音MAD、字幕、MV等の成熟した制作環境を
比較するなら、利用者が実際に使う**versionと設定を固定した完成拡張スタック**を一つの比較profileとして
記録する。

この観察は全AviUtl利用者が同じpluginを使うという主張ではない。次の構成は、今回の「完成WAVへ数百〜
数千の短い映像Clip／画像／effectを同期配置する」負荷について、責任の違う代表例を揃える測定profileである。
AviUtl2は内部設計とplugin ABIが異なるため、旧AviUtl完成環境と混ぜず別profileにする。

## 2. 本体外で獲得された責任

| 責任 | 代表的な先例 | 観察できること | Motoliiへ直結する問い |
|---|---|---|---|
| 実用media入力 | L-SMASH Works | MP4等のdemux／decode／seekを入力pluginが担う | Hostのdecode境界、source identity、fallbackをどこが所有するか |
| 同素材大量cut | InputPipePlugin | 同じfileをClipごとに再openせずhandleを再利用する。decodeを別processへ出し、画像／音声転送を共有memory化する | Clip identityとdecoder identityを分け、同じsource reader／decoded frameを安全に共有できるか |
| 本体／拡張編集の局所高速化 | patch.aul | Timeline／設定画面の描画、text cache、図形、Glow、Blur、Displacement等をbinary patchで置換する | UI paint、text、effect、decodeを総fpsへ畳まず、責任別に測れるか |
| script実行 | LuaJIT | JITとFFIによりLua effect／generatorの重いloopを置換する | plugin payload overhead、GPU dispatch、Host境界費用を分離できるか |
| 区間preview | 拡張編集RAMプレビュー | 選択区間を事前生成し、無圧縮／通常品質／1/2／1/4で再生負荷を一定化する | K7／K8、Draft coverage、明示Freezeの比較対象は何か |
| 波形／解析 | アイテム内音声波形 | 波形計算を外部／複数processへ出し、再生中や保存中には仕事を止める | analysis workerがTimeline／Transportを待たせず、古いgenerationを捨てられるか |
| 操作圧縮／表現追加 | `.exa`／`.exo`、alias、Lua script、編集補助plugin | 配置、複製、整列、再利用、effect追加の操作回数を減らす | pixel速度だけでなく、同じ結果までの操作数とDocument変異costを測るか |
| Final出力 | 出力plugin群 | codec／container／encoderを本体外で補う | Previewの軽さとFinal throughput／配布能力を混同しないか |

一次資料上、[InputPipePlugin](https://github.com/amate/InputPipePlugin)は、同じ動画を多数cutすると
拡張編集が同じfileへ繰り返しopen要求を送ること、handle cacheで一度開いたhandleを再利用すること、
L-SMASH Worksを別processで動かしてAviUtl側のmemory使用量を減らすことを明記する。共有memoryへ変更した
版の履歴には、旧named pipe経路に対して約6割高速化したという作者記録もある
（観察commit `99c169355562942a33db3efb56ec135803c53439`）。これはMotoliiの同一asset大量cut fixtureで
process起動回数、file open回数、seek回数、decoded frame共有を必ず計器化すべき直接の先例である。

[patch.aul](https://github.com/nazonoSAUNA/patch.aul)は「バグ修正plugin」だけではない。観察commit
`2441800fe81070d437e8eac98d9580104ca7bfb7`の同梱文書は、拡張編集window／設定dialogの描画最適化、
text cache、OpenCL／AVX2によるGlow、極座標変換、Displacement、放射／方向／Lens Blur、縁取り等の
高速化を列挙する。従って「AviUtlはCPU softwareだからeffectが遅い」と一括評価できない。完成環境では
pluginが特定の支配costを置換している。

[拡張編集RAMプレビュー](https://oov.github.io/aviutl_psdtoolkit/plugins.html)は選択区間をmemoryへ
事前生成し、通常品質、1/2、1/4解像度を選べる。これは自動cacheとは別に、利用者が重い区間を明示的に
完成画素へ置換する既存作法である。MotoliiのK7／K8は「RAM previewが存在しない前提」の新規発明として
比較せず、Document不変、部分無効化、hard budget、再起動後再利用、audio clock維持で何を改善するかを測る。

[アイテム内音声波形](https://github.com/hebiiro/AviUtl-Plugin-ShowWaveform)は、外部process化、
multi-process化、zoom-out描画負荷対策、preview中／保存中に何もしない変更を履歴化している
（観察commit `cb12a9aaf385b8c1d645ebc893155bd773c8c511`）。波形表示を単なるUI装飾でなく、解析scheduleと
Timeline paintの独立した性能責任として扱う先例になる。

[sigma_aviutl_scripts](https://github.com/sigma-axis/sigma_aviutl_scripts)はLuaJITのJIT／FFIを推奨し、
patch.aulのcache画像共有memory退避や修正済みeffect意味へ依存するscriptがあることを説明する
（観察commit `79bce1098a69b1e8da7cb77811aa3356a411ee2d`）。一つのscriptだけでなく、
runtime、patch、resource、aliasを含むclosureが実際の能力単位になり得る。

## 3. 「AviUtlが大量の短いClipを許容した」の分解

現時点では、次の要因を一つの「軽いengine」へ畳まない。

1. 現在時刻にactiveなobjectだけを評価する本体／拡張編集の構造。
2. 同一sourceを大量cutした時のhandle reuse。
3. decode memoryと失敗を別processへ隔離した効果。
4. Timeline／設定dialog／text／代表effectをpatch.aulが置換した効果。
5. LuaJITがscript loopを置換した効果。
6. RAM previewによる利用者指定区間の事前Bake。
7. 波形等のbackground workを再生中に停止した効果。
8. alias／script／編集補助により、同じ作品へ到達する操作数を減らした効果。
9. 低解像度、間引き、素材形式、project設定、利用者が受容したpreview品質。

「本体が速い」「cacheが賢い」「古いsoftwareだから単純」のどれか一つで説明せず、下記fixtureで寄与を
個別に測る。

## 4. 完成拡張スタックprofile

実測票は少なくとも次を記録する。profile名だけでplugin構成を推定しない。

- AviUtl、拡張編集、各plugin／script、Lua runtimeのversionまたはcontent hash。
- plugin優先順位、patch.aul switch、L-SMASH／InputPipeのhandle cache・IPC方式。
- cache frame／image cache／video handle数、maximum image size、thread数。
- preview解像度、画像処理間引き、RAM preview品質と対象区間。
- 素材の事前中間codec変換、手動proxy、範囲書き出し→再importの有無、準備時間、変換先codec／解像度。
- input plugin、codec、GOP、VFR／CFR、audio、file hash。
- project解像度／fps、同時active object数、総object数、script／effect closure。
- cold／warm、RAM preview未生成／生成済み、波形未解析／解析済み。

最初のprofile候補は次の三つとする。これは配布推奨一覧やAviUtl利用者の多数派認定ではない。

| profile | 目的 |
|---|---|
| A0: AviUtl＋拡張編集の最小構成 | 本体／拡張編集が担う基礎costを分離する |
| A1: 成熟した完成拡張スタック | L-SMASH Works、InputPipePlugin、patch.aul、LuaJIT、RAM preview、波形／代表編集補助を固定し、実利用に近い上限を測る |
| A2: AviUtl2固定版 | 旧ABI／binary patch資産を混ぜず、現行後継softwareとして別測定する |

## 5. 音MAD映像fixture

完成WAVを音声正本とし、DAW機能は要求しない。時間方向の編集密度と現在frameのactive setを分ける。

| variant | 配置 | 反証する短絡 |
|---|---|---|
| D0 | 3分WAV、同一1080p long-GOP動画1本をsource時刻単調に1000分割、各時刻active 1〜3 | 総Clip数と同時decode数の混同、同一file再open |
| D0r | D0のClip配置をsource時刻乱順にし、playhead順再生で遠距離seekを反復 | handle reuseの利得とGOP pre-roll／seek thrashの混同 |
| D0s | 同一fileの離れた2〜3 source位置を同時activeにする | asset単位の単一reader共有が常に安全／高速という短絡 |
| D1 | D0＋静止画300、text／図形300、軽いtransform／opacity | pixel以外のTimeline／Document／UI costの無視 |
| D2 | D1＋patch.aul高速化対象のGlow／Blur／DisplacementとLua effectを代表区間へ配置 | 素のeffectと完成環境のeffect性能の混同 |
| D3 | D2の一部で10〜20 layer同時active、Group effectを追加 | 編集密度と同時合成負荷の混同 |
| D4 | 1000個が異なるsource file | handle reuseだけで大量素材一般を証明する短絡 |
| D5 | 同じ短いScene／Group相当を20回反復配置 | 平坦Clipだけで構造再利用、K7候補、Scene参照costを評価する短絡 |

数値は初期fixtureの比較入力であり、製品SLOやAviUtl利用実態の代表値ではない。実作品manifestを取得できた
場合は、匿名化したobject数、active set、source再利用率、effect分布を別profileとして追加する。
D0はlong-GOP直編集と、利用者が事前に編集用中間codecへ変換した条件を分け、変換時間と容量を含める。
D5はAviUtl側のScene参照とMotolii側の既存Group／Composition境界を対応付け、K7完成前の通常評価と
K7完成後の二周目を別測定する。K7未実装の現時点で勝利や再利用率を主張しない。

操作列はimport、cold first frame、連続再生、scrub burst、1 Clip移動、trim、複製、分割、parameter drag、
Undo／Redo、project reopen、RAM preview生成／purgeに加え、機械生成した1000 object相当の配置を
一括投入し、一操作としてUndoする経路を固定する。AviUtlの`.exo`とMotoliiのtyped command列は形式を
同一視せず、同じ配置結果への投入時間、Document commit数、UI停止、履歴単位を比較する。
記録値は次を分離する。

- 操作入力から対応generationの正しいframeまでのp50／p95。
- UI event／Timeline paint、Document command、active-set query、render、decodeの時間。Timeline viewportの
  zoom、可視object数、全尺一様配置／局所密集も記録する。
- file open、reader／process生成、seek、decoded／discard frame数、source-frame共有hit、Clip遷移ごとの
  source seek距離分布、同一fileの同時読取position数。
- effect／script別CPU／GPU時間、IPC bytes、CPU upload／GPU copy bytes。
- RSS、shared memory、VRAM、cache／RAM preview bytes、background job数。
- audio underrun、表示frame drop、古いframe表示、Final frame差。
- 同じ結果へ到達する操作数と、補助pluginが生成したobject／command数。
- RAM preview／手動pre-renderの生成時間、生成後の連続再生、1 Clip編集後の失効範囲と再生成時間。

本節をAviUtl移行fixtureの**操作列と移行固有記録列の正本**とする。素材manifestとdecode固有の共通記録列は
[Decode→Composite前提監査 §7](2026-07-29-decode-to-composite-premise-audit.md#7-原因分離fixture)、
machine／OS／電源／熱／製品設定は
[最低スペック移行性能ゲート §4](2026-07-29-aviutl2-low-spec-migration-performance-gate.md#4-二段の合格面)
を正本とし、同じ列を別定義しない。

## 6. Motoliiへ標準装備する責任と拡張へ残す責任

AviUtlがpluginで解決したことを、すべてMotolii plugin APIへ移さない。

### Hostが所有する

- source identity、reader／decoder共有、request coalesce、generation破棄。
- ResourceLedger、hard budget、shared／external memory accounting。
- Timeline active-set query、UI virtualization、single writer、Undo freshness。
- cache key、無効化、eviction、K7／K8、audio clock、background job preemption。
- 波形等の解析jobが失速してもeditorを待たせないこと。
- plugin／Host moduleのfailure isolationとtyped diagnostic。

これらを第三者pluginへ委譲すると、正しさ、memory、Undo、deadlineが拡張の組み合わせで変わるため、
Motoliiの共通保証にできない。

### 締結済みseat上のHost module／pluginへ残せる

- media importer／codec provider、effect、generator、analyzer、editing aid、export provider。
- effectの高速なWGSL実装、script／code payload、alias／preset／Kit。
- Hostが計測、budget、generation、lifecycleを所有した上での表現追加。

media importer／codec providerはrender純関数pluginと同じ状態契約ではない。Hostがreader／process／
cancel／teardown／typed failureのlifecycleを所有し、公開plugin境界を通るcodeは供給元を問わず
非信頼とする。[Controlled Microkernelの信頼境界](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md#6-pluginという語と信頼境界の分離)
と[GAP-26](../backlog.md)を維持し、InputPipeの別process化は完成安全境界でなく隔離比較の先例として扱う。

### 模倣しない

- 未公開memory layoutへのbinary hookを通常拡張点にする。
- pluginごとのglobal cache、独自thread pool、独自GPU／色authority。
- pluginの有無でDocument意味、Undo、Final frameが黙って変わる。
- 必須patchを利用者が手動で集めないと基本性能／正しさが成立しない配布。

AviUtlの価値は、局所課題をcommunityが独立に直せたことである。Motoliiが学ぶべきなのはbinary patchそのもの
ではなく、**閉じたHost責任と、独立に置換可能なseatを同時に持つこと**である。

## 7. M3／M4への接続

| 観察 | 接続先 |
|---|---|
| 同素材大量cutのhandle reuse | Decode→Composite監査のpersistent reader／source-frame共有、M4 K4 |
| 外部process／shared memory | GAP-26、K1a external／shared memory accounting |
| Timeline／設定window最適化 | M3 native Timeline、headless interaction、G0-4 |
| text／effect局所高速化 | M4 K1計測hook、render path、将来Vism |
| RAM preview | K7／K8。ただしDocument不変、部分無効化、hard budgetを追加審判 |
| 波形background work | M3 AG-3、M2-D8、D5 |
| alias／script／編集補助 | creator／developer連続体、Vism／Kit、typed command |

本観察から新しいDocument field、plugin ABI、永続cache format、数値SLOを追加しない。M4は完成AviUtl環境を
一製品として比較する一方、実装判断では各pluginが処理した責任へ分解する。

## 8. 非目標と停止線

- 今回列挙した構成を「標準的な全AviUtl利用者」の統計として扱わない。
- plugin名、作者申告の高速化率、紹介記事だけでMotolii採否を決めない。
- 旧AviUtl、AviUtl2、plugin有無、設定差を一つのfpsへ混ぜない。
- RAM preview済みAviUtlとcold Motolii、または逆を同条件比較と呼ばない。
- 画像処理間引きや精度差を記録せず、見た目が動いたことだけを合格にしない。
- AviUtlのbinary hook／global stateをMotolii公開plugin契約へ移植しない。
- 「AviUtlではpluginだった」を理由に、Hostが持つべき正しさ／所有／budgetを外へ出さない。
- 完成拡張スタックに勝つために、古いgeneration、同期readback、CPU pixel経路、別Final rendererを許さない。

## 9. 現在の判定

旧AviUtlの軽さは、本体だけの性質でも、単一cache機構だけの成果でもない。入力handle reuse、process隔離、
shared memory、局所patch、JIT、区間RAM preview、background解析停止、操作補助が累積した完成環境として
再評価する必要がある。

従ってMotoliiの移行資格は「素のAviUtlより速い」ではなく、**versionと設定を固定した成熟AviUtl完成環境で
成立していた音MAD映像操作を退行させないこと**を旧AviUtl側の合格面とする。移行資格全体の一文定義は
[最低スペック移行性能ゲート §1](2026-07-29-aviutl2-low-spec-migration-performance-gate.md#1-結論)に一元化し、
旧AviUtl完成stackとAviUtl2の両旗を通す。勝敗、寄与率、必要SLOは上記fixtureの実機測定まで未決である。

## 10. Fable 5 read-only助言の処分

初回監査は`VERDICT: REVISE`、P0=0／P1=2だった。移行資格の一文が旧AviUtl完成stackとAviUtl2へ
片側ずつ分裂していた点、D0がsource時刻順序と同一fileの離れた位置の同時参照を固定せず、
handle reuseとseek thrashを分離できない点をP1として採用した。

非blocking助言から、一括配置import、手動中間codec／pre-render作法、操作／記録列の正本分離、
importer lifecycle／信頼境界、Windows notebookの電源／熱条件、Scene／Group反復variantも採用した。
Fableの助言を実測や仕様authorityとせず、新SLO、reader identity形式、plugin ABI、Document field、
永続cache formatは未決のまま維持する。

限定再審査は`VERDICT: ACCEPT`、P0/P1=0。残ったP2二件も、performance modelの移行資格要約を
本gate参照へ弱め、Gate AのL0-M記録列を本節への参照に置換して回収した。
