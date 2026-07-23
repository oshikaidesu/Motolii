# 先例調査: D5 Transport低速時戦略 — クロックオーナー交代 vs 音声主固定+適応解像度(2026-07-14)

ステータス: **【採択】2026-07-14**(ユーザー決定)。**レビュー修正**(レイテンシ=デバイス待ちのみ / D4-FU切り出し / DRS縮退 / 完了条件閾値)を仕様へ追記済み。

> 2026-07-23歴史監査: cutoff全4版を[Unit 5D回収](2026-07-23-historical-d5-transport-lineage-recovery.md)で処分した。audio device clock常時主、video drop、GPU timestamp計測時だけのDRS、device waitだけの補償を維持する。現行コードはTransport／DRS骨格とheadless審判までで、本番Preview、実GPU timestamp配線、10分実機E2E、mixed `AudioProgram`接続は未完了である。

## これは何か

D5(Transport)発注前の既製品サーチ。動機: 現行仕様[「音声トランスポート設計」](../specs/M2-document-model.md)3項の**バリスピードモード**(低速時=レンダ進捗がクロック所有者に交代し、音声が適応リサンプリングでピッチごと低速化。0.6xなら音も0.6x)が、ユーザーがAEで最悪と体験した挙動(ドロップアウト時に音が低速化し音ハメ不能)と同型ではないかという懸念(2026-07-14ユーザー提起)。調査は4レーン(動画プレイヤー/NLE/適応制御則/Rust部品)を一次資料(公式マニュアル・設計文書・実装ソース・原典論文)で実施した。

## 結論の先出し

1. **「映像が間に合わない→音声ごと自動低速化」を持つ既製品は、調査した全ツール中AEのみ**。しかもAdobe自身が環境設定「Mute Audio When Preview Is Not Real-Time」を用意するほどの既知の構造的不満点であり、ユーザー苦情が定番化している。
2. プレイヤー界(mpv/ffplay/VLC/GStreamer/Chromium)とNLE界(Premiere/Resolve/FCP/Blender)の一致した答えは**「音声クロック主+映像フレームドロップ」**。NLEはさらに**「解像度/処理品質を下げて実時間を回復」**を併用する。GStreamerは設計文書に「音声の欠落は映像フレームのドロップより耳障り」と明文化している。
3. 現行仕様3項が根拠に挙げる「mpv/ffplayのadaptive resamplingと同じ確立された手法」という帰属は**一次資料と一致しない**(規律2の因果・帰属チェックで棄却対象)。mpvのdisplay-resampleは音声変化を**±0.125%**に制限したジャダー除去用途で、レンダが追いつかない場合はmpvも結局フレームドロップに戻る。ffplayの±10%音声補正はデフォルト無効(非オーディオマスター構成のみ)。**0.6x級の自動音声低速化の先例は存在しない**(あるのはAEの悪評のみ)。
4. **採択案**: D5からクロックオーナー交代・適応リサンプリングを外し、「音声クロック常時主+映像フレームドロップ+**適応解像度降格**(Draft 1/2→1/4)」へ。デバイス≠素材レートの固定比変換は**D4-FU**。DRSはtimestamp query非対応時に自動無効。レイテンシ補償はcpalデバイス待ちのみ。

## A. プレイヤー界: 音声マスターが唯一の主流、音声レート操作は±1%未満の世界

- **mpv**: デフォルト`--video-sync=audio`(音声マスター+映像ドロップ/リピート)。音声を触るのはdisplay-sync系モードのみで、目的は「レンダが遅いから」ではなく「ディスプレイのリフレッシュレートに映像を完全同期させジャダーを消すため」。制限は`--video-sync-max-video-change`=1%、`--video-sync-max-audio-change`=**0.125%**。23.976→25fpsの+4.27%ですら「ピッチ変化が極端すぎる(too extreme)」としてデフォルト拒否。公式wikiは1%+0.125%を「訓練された耳でもほとんど気付かない」と記述。範囲を満たせない場合はdisplay-sync自体が無効化されaudioモードへフォールバック。フレームドロップは`--framedrop=vo`(デコードは全フレーム、表示のみスキップ)が推奨で、実効10fpsを下回るとドロップを停止する安全弁を持つ。`decoder`ドロップは「予測不能で見るに堪えない」と非推奨。[mpv options.rst](https://github.com/mpv-player/mpv/blob/master/DOCS/man/options.rst) / [mpv wiki Display-synchronization](https://github.com/mpv-player/mpv/wiki/Display-synchronization)
- **ffplay**: デフォルトはオーディオマスター(`AV_SYNC_AUDIO_MASTER`)。映像側は`sync_threshold = clamp(delay, 0.04, 0.1)`秒で待ち時間調整+期限切れフレーム破棄。音声サンプル補正(±10% `SAMPLE_CORRECTION_PERCENT_MAX`)は**オーディオが非マスターの時のみ**動く。[fftools/ffplay.c](https://github.com/FFmpeg/FFmpeg/blob/master/fftools/ffplay.c)
- **VLC**(4.0新クロック設計): 「ノミナルケースではオーディオがマスタークロック」。旧設計の欠点として「ローカルファイルでも音声リサンプルを要求してしまう」ことを挙げ、新設計では「**オーディオマスターモードではもう音声をリサンプルしない**」。同期回復は遅延側に合わせる(映像=新しい画を出さない、音声=無音)。[doc/clock.md](https://github.com/videolan/vlc/blob/master/doc/clock.md)
- **GStreamer**: パイプラインクロックはオーディオシンク優先。シンクが遅延を測定し上流へQoSイベント(`proportion`/jitter)→デコーダがフレームスキップ・品質低下で応答(=「上流の品質を落として追いつく」の一般形。適応解像度の同型)。ビデオシンクは`max-lateness`≈20msでドロップ。設計文書に「**通常QoSはビデオパイプラインでのみ有効。音声の欠落は映像フレームのドロップより耳障り**」と明記。[QoS design](https://gstreamer.freedesktop.org/documentation/additional/design/qos.html)
- **Chromium**: プルベースでサウンドカードが再生を駆動し、音声供給量がグローバルクロックを更新。映像レンダラはvsyncごとにクロックをポーリングし期限切れフレームをドロップ。WSOLA(`AudioRendererAlgorithm`)はユーザー指定playbackRate用であり、A/V同期のために音声速度を変える機構ではない。[Chromium design doc](https://www.chromium.org/developers/design-documents/video/)
- **可聴性の実効コンセンサス**: ±0.1〜0.5%=安全圏(DAC発振器の自然公差と同水準)、1%=実用上限、4%以上=明確に可聴(PALスピードアップ≈0.7半音として知られる)。**現行仕様の0.6x追従は−40%であり、mpvが「extreme」と拒否する4.3%の約10倍、実用上限の40倍**。

## B. NLE界: 「音声ごと遅くする」のはAEだけ、それはAE最大級の不満点

| ツール | 間に合わない瞬間の挙動 | 解像度低減 | 音声が遅くなるか |
|---|---|---|---|
| Premiere Pro | フレームドロップ(黄インジケータ表示)。「60秒のシーケンスはフレームをスキップしてでも60秒で再生する」(Adobe公式FAQ) | 手動 Full〜1/16 | 決してない |
| DaVinci Resolve | フレームドロップ(デフォルト) | Timeline Proxy手動 + Performance Modeが処理品質を**自動**調整 | 「Show All Video Frames」を**明示ON**にした時のみ音声が犠牲(マニュアルに"audio quality is compromised"と注記) |
| Final Cut Pro | フレームドロップ+「If frames drop, warn after playback」で事後警告 | 手動(Better Performance)+バックグラウンドレンダで予防 | 決してない |
| **After Effects** | **全フレーム描画のため実時間より遅く再生**(フレーム正確性優先のコンポジタ出自) | 手動(Half/Quarter) | **なる(悪名高い)**: スロー/低ピッチ/スタッター/デシンク。対策として「Mute Audio When Preview Is Not Real-Time」設定と「Cache Before Playback」が存在 |
| Blender | ユーザー選択制: Sync to Audio=音声主+ドロップ(音付き作業の推奨)/ Frame Dropping / Play Every Frame(映像が遅くなり音とズレる) | — | Play Every Frameでデシンク |
| Alight Motion | プレビュー解像度低減が主戦略(公式パフォーマンスガイド: 1080p→480p/360p、書き出し非影響) | 手動 | 情報なし |
| Cavalry / Rive | 単純にfps低下(リアルタイムレンダラ型)。Cavalryはビューポート品質設定(High〜Lowest)+再生キャッシュ | Cavalry品質設定 | — |

主要出典: [Adobe公式FAQ(AE vs Premiere実時間再生)](https://community.adobe.com/t5/after-effects-discussions/faq-why-doesn-t-after-effects-preview-in-real-time-like-premiere-pro/m-p/10690804) / [Premiere再生解像度](https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/set-display-quality-for-the-source-and-program-monitors.html) / [Resolve 18マニュアル "Prioritizing Audio or Video Playback"](https://www.steakunderwater.com/VFXPedia/__man/Resolve18-6/DaVinciResolve18_Manual_files/part232.htm) / [FCP再生設定](https://support.apple.com/guide/final-cut-pro/playback-settings-verb8e60ab7/mac) / [AEプレビュー統合(CC 2015)](https://helpx.adobe.com/after-effects/kb/all-about-previews-in-after-effects-cc-2015--13-5-.html) / [Blenderマニュアル Timeline](https://docs.blender.org/manual/en/3.4/editors/timeline.html) / [Alight Motion Performance Guide](https://support.alightmotion.com/hc/en-us/articles/10537513451793-Performance-Guide)

AE音声低速化のユーザー苦情実例: [Creative COW "After Effects Audio Slowing Down"](https://creativecow.net/forums/thread/after-effects-audio-slowing-down/) / [Adobe Community "Slow audio playback in After Effects preview"](https://community.adobe.com/t5/after-effects/slow-audio-playback-in-after-effects-preview-can-anyone-help/td-p/10184556) — [ae-pain-points.md](../ae-pain-points.md)の系譜に連なる。

## C. 適応制御の制御則(輸入候補の中身)

### C-1. Dynamic Rate Control(エミュレータの音声微調整 — 参考、v1不採用)

- 原典: [Arntzen "Dynamic Rate Control for Retro Game Emulators" (2012)](https://docs.libretro.com/guides/ratecontrol.pdf)。音声バッファ充填率を目標50%に保つ**純粋P制御**(`ratio × (1 + d·正規化偏差)`)で、1次系として指数収束が保証される。最大ピッチ偏差dは**0.2〜0.5%**([RetroArch実装](https://github.com/libretro/RetroArch/blob/master/audio/audio_driver.c)の既定`DEFAULT_RATE_CONTROL_DELTA = 0.005`)。実測の実効偏差は0.062%。
- これが解くのは「映像リフレッシュと音声レートが同一発振器に連動したレトロ機を、公差のあるPCの2クロック(モニタ/DAC)へ写像する」問題。**Motoliiには2クロック問題が存在しない**: 映像はvsyncに同期させない方針が確定済み([M3仕様ガード3](../specs/M3-ui-integration.md): 「主クロックは音声。vsyncが暴走してもフレームペースと音声同期が崩れない」)なので、追従すべき第二のクロックがない。
- Dolphinは大幅スローダウン対応としてピッチ保存の「Audio Stretching」(SoundTouch、レイテンシ80ms、既定OFF)も持っていたが、現行masterでは撤去済み。

### C-2. Dynamic Resolution Scaling(ゲームの適応解像度 — 採択案の中身)

- **フィードバック信号**: GPUフレーム時間(タイムスタンプクエリ)。CPUスレッド時間も記録し、**CPUバウンドのフレームはGPU超過とみなさない**(GPUに責任のない超過で解像度を下げない)— [UE DynamicResolution.cpp](https://github.com/chenyong2github/UnrealEngine/blob/c865e168d0935b8e5f4bd865ddcc1c733c8ce7cf/Engine/Source/Runtime/Engine/Private/DynamicResolution.cpp) / [UE公式doc](https://dev.epicgames.com/documentation/en-us/unreal-engine/dynamic-resolution-in-unreal-engine)
- **UEの発振防止4段構え**(既定値): ①解像度を**上げる方向のみ**ブレンド係数0.9で償却(下げは速く上げは慎重の非対称制御) ②±2%未満の変化は棄却(不感帯) ③変更間隔は最低8フレーム ④16フレームの指数加重履歴平均。加えて**パニック機構**: 2連続でGPU予算超過なら慎重さをバイパスして即時降格+履歴リセット。目標は予算の90%(ヘッドルーム10%)。下限クランプ50%。
- **id Tech(DOOM 2016)の簡約形**: 二重閾値によるヒステリシス — 昇格閾値(`rs_raiseMilliseconds 14.5ms`)をフレーム予算(16.7ms)より低く置く(mod由来の報告値で公式文書なし。構造の参考に留める)。
- 系譜の原点は[Intel "Dynamic Resolution Rendering" (Binks, GDC 2011)](https://www.intel.com/content/dam/develop/external/us/en/documents/dynamicresolutionrendering-183334.pdf)。VSync有効時はフレーム間隔が同期レートに張り付くため**GPU内部時間の計測が必須**(UEと同じ問題意識。Motoliiではwgpuタイムスタンプクエリが対応物)。

### C-3. Rust部品(将来席の在庫確認)

- **rubato**(v4.0、ストリーミングリサンプルのデファクト): `set_resample_ratio(_, ramp: true)`で次チャンク内を線形ランプ=クリックフリーのレート変更。±0.5%微調整から0.5x大変速まで公式サポート範囲(`max_resample_ratio_relative`)。RT安全(`process_into_buffer`はアロケーション無し)。配置の定石はプロデューサスレッド側(PCMキャッシュ読み出し→リサンプル→リング書き込み)。[rubato](https://github.com/HEnquist/rubato)
- **signalsmith-stretch**(Rustバインディング有り、MIT): ピッチ保存タイムストレッチ。最良品質域は0.75x〜1.5x、レイテンシ既定〜120ms。rubberbandはGPL/商用デュアル+公開バインディング無しで不適。[signalsmith-stretch](https://github.com/Signalsmith-Audio/signalsmith-stretch)
- **cpal**: `OutputCallbackInfo::timestamp()`の`playback - callback`で**デバイス待ちレイテンシ**を取得可能(ALSA/CoreAudio/WASAPI等ホスト別実装あり、PulseAudio/ASIOは粗い)。採択後の6項はこれのみを補償に使う。時計起点が「デバイスへ供給済み」のとき、リング充填量は未来の未供給音声なので**さらに引くと二重計上** — リング量を時計に使う構成は採らない。[cpal timestamp.rs](https://github.com/RustAudio/cpal/blob/master/src/timestamp.rs)
- 現行D4の`pick_output_config`(`crates/motolii-audio/src/device.rs`)はデバイスが素材のサンプルレートを直接サポートしない場合に型付きエラーで拒否する。固定比リサンプルのフォールバックは**D4-FU**へ切り出し済み(下記判定表)。

## 現行仕様との衝突点(採択時に書き換えが要る場所)

1. **[M2「音声トランスポート設計」](../specs/M2-document-model.md)3項**(バリスピードモード): 根拠に挙げた「mpv/ffplayの確立された手法」の帰属が一次資料と不一致(前掲)。置換対象。1/2/4/5/6項は全先例と一致しており不変。
2. **D5行**: 内容「低速時=レンダ進捗主+適応リサンプリング追従」、完了条件「0.5x律速で音声がピッチ同調して追従」「可聴アーティファクトが無いことを人間の耳で確認」。
3. **[M3 U5行](../specs/M3-ui-integration.md)**: 完了条件「低速シーンでバリスピード動作が確認できる」。
4. **[performance-model.md](../performance-model.md)プレビュー品質モード**: 「Draft 1/2(重い時1/4に自動段階降格)」— **既に適応解像度の席が存在する**(衝突ではなく接続点)。採択案はこの「自動段階降格」の制御則を与える実装。空間パラメータの解像度相対規律(落とし穴F-1)が「降格しても見た目の分岐なし」を保証する前提も宣言済み。

## 採択案(**【採択】2026-07-14** — 以下は仕様へ反映済み)

### 提案1: クロックオーナー交代を廃止し、音声クロック常時主に固定する

再生位置=供給済みサンプル数由来(現行2項のまま)。映像は現在位置に最も近いフレームを表示し、間に合わなければドロップ。フレーム=時刻tの純関数(B-4)なので「ドロップ」は特別な機構ではなく**「レンダループが常に最新の現在時刻だけをレンダする」**で自然に成立する(mpvの`framedrop=vo`と同型: 作りかけを捨てるのではなく、古い時刻を最初から手掛けない)。旧3項が守ろうとした不変条件「映像だけ遅れて音が先行する状態が構造的に存在しない」は**この構成でも維持される**: 表示されるフレームは常に「いま聞こえている音」の時刻に対応し、劣化はfps低下としてのみ現れる。

### 提案2: 低速時=適応解像度降格(DRS)

- 段階: Draft 1/2 → 1/4(performance-model.mdの既定段階をそのまま使う。連続スケールはTAA的アップスケーラを持たないv1では過剰)
- 制御則はid Tech型二重閾値+UEの安全弁に縮小して輸入:
  - **降格**: フレーム時間が予算(1/fps)超過をN連続(UE既定は2)→即時降格(パニック則)
  - **昇格**: フレーム時間が予算×(1−ヘッドルーム、目安10〜20%)を下回る状態がMフレーム+最小滞留時間継続した時のみ1段階
  - 計測の正本はGPUフレーム時間(wgpu timestamp query)。CPUバウンド判定を最初から席として持つ(UEの教訓: CPUが犯人の時に解像度を下げても無意味)
  - **縮退**: timestamp query非対応GPUでは自動DRSを無効化し、フレームドロップのみ継続(`motolii_gpu::required_features`のoptional併設方針と整合)
- 発振(解像度パンピング)しないことを、最小滞留時間内の再復帰0回として自動テストする

### 提案3: 1/4でも間に合わない場合は音声正速のままフレームドロップ継続

Premiere/FCP/Resolveと同じ「スライドショー化しても時間軸は嘘をつかない」。音ハメ用途では音楽のタイミングが正であることが絶対条件であり、fps低下は許容できるがテンポ・ピッチの嘘は許容できない。ドロップ発生の可視化(Premiereの信号機型インジケータ)はM3の席として記録。

### 提案4: バリスピード(ピッチ変化を伴う再生)は「自動フォールバック」から「明示ユーザー操作」へ再定義

自動挙動としては廃止。JKLシャトル・音声スクラブ等の**明示操作**の機能としてD5外に席を残す(Resolveの「Show All Video Frames」と同じ「明示オプトイン+注記」構造)。その時の部品はrubato(ピッチ変化型、ramp付き、レイテンシ〜3ms)またはsignalsmith-stretch(ピッチ保存型、レイテンシ〜120ms)で、D4のプロデューサ/リング構造に差し込める(C-3)。

### D5行の書き換え案(レビュー修正後の正本はM2タスク表)

> | D5 | …レイテンシ補償=cpalデバイス待ちのみ。依存 D3,D4,**D4-FU**。完了条件はドリフト≤1フレーム長 / 変換前PCMビット同一 / パンピング0 / 切替時アンダーラン増加0(いずれも自動) |

完了条件の「人間の耳による可聴アーティファクト審査」は、自動フォールバックの音声経路に可変リサンプラが入らなくなるため**縮小**される。固定比変換(D4-FU)がある場合もビット同一は変換前PCM境界に限定。

### 波及: M3 U5の完了条件

「低速シーンでバリスピード動作が確認できる」→「低速シーンで音声無傷のまま解像度降格+フレームドロップが確認できる(ドロップインジケータ表示)」へ書き換えが必要(U5発注前に)。

## 判定表(規律6: 判定語併記)

| 所見 | 判定 | 根拠 |
|---|---|---|
| 音声クロック常時主+映像フレームドロップ | **採用** | プレイヤー・NLE全業界一致。反例=AEのみで、それが本プロジェクトの回避対象そのもの |
| 適応解像度降格(DRS) | **採用(縮小)** | NLE慣行(Resolve自動品質調整・AM解像度低減)+performance-model.md既定の実装。制御則はUE完全形でなくid Tech型二重閾値+パニック則+CPUバウンド判定に縮小 |
| クロックオーナー交代+自動適応リサンプリング(現行3項) | **棄却** | 帰属誤り(mpv/ffplayは±0.125〜1%のジャダー除去であり0.6x級の先例ではない)。実先例はAEの悪評のみ |
| DRC(±0.5%音声P制御) | **棄却(v1不要)** | 解く問題(2クロック公差)がMotoliiに存在しない(vsync非同期方針: M3ガード3)。将来display-resample的ジャダー除去を望む時の席としてのみ記録 |
| ピッチ保存タイムストレッチ(signalsmith-stretch) | **延期** | 明示スクラブ/シャトル機能(M3以降)の部品。D5の範囲外 |
| rubato固定比変換(デバイスレート≠素材レート) | **D4-FUへ切り出し【決定】** | D5依存に置く。ビット同一は変換前PCM境界。**アルゴリズム遅延はproducer pre-roll/先頭trimで吸収**しTransportへ持ち出さない |
| DRS timestamp query非対応 | **縮退規約【決定】** | 自動DRS無効+フレームドロップ継続 |
| ドロップ可視化インジケータ | **延期(M3席)** | Premiere/FCPの事後警告・信号機方式。U5と同時に |

## 注意(規律2/3)

- 反対側レビューは未実施。ただし本調査の中心的主張(「音声ごと低速化の先例は無い」)は**反例探索そのもの**(全主流プレイヤー・NLEを走査してAE以外に見つからなかった)であり、「仮説と整合する事例集め」ではない。残る検証点は「適応解像度の自動切替をNLEが再生中に行う先例は薄い(Resolve Performance Modeの自動調整は処理品質でありレンダ解像度そのものではない。Premiere/FCP/AMは手動)」こと — つまり**自動DRSはゲーム界からの輸入であり、NLE界では手動が主流**。この点は「うちはゲームと同じリアルタイムGPUレンダラ型(Cavalry/Rive系)である」という設計位置づけで正当化するが、切替の視覚的違和感が実害になるかはU5実装時に実物で判定する(だめなら手動固定に縮退可能 — スキーマに焼かない運用パラメータであり不可逆性は無い)。
- 数値既定値(ヘッドルーム%・連続超過N・滞留時間)は先例の既定値を出発点とする運用調整値であり、永続スキーマ・プラグイン契約に焼かない(GR-PV-2)。

## 出典一覧

プレイヤー: [mpv options.rst](https://github.com/mpv-player/mpv/blob/master/DOCS/man/options.rst) / [mpv wiki](https://github.com/mpv-player/mpv/wiki/Display-synchronization) / [ffplay.c](https://github.com/FFmpeg/FFmpeg/blob/master/fftools/ffplay.c) / [VLC clock.md](https://github.com/videolan/vlc/blob/master/doc/clock.md) / [GStreamer QoS](https://gstreamer.freedesktop.org/documentation/additional/design/qos.html) / [Chromium video design](https://www.chromium.org/developers/design-documents/video/)
NLE: [Adobe FAQ](https://community.adobe.com/t5/after-effects-discussions/faq-why-doesn-t-after-effects-preview-in-real-time-like-premiere-pro/m-p/10690804) / [Premiere playback resolution](https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/set-display-quality-for-the-source-and-program-monitors.html) / [Premiere dropped frame indicator](https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/enable-dropped-frame-indicator.html) / [Resolve 18 manual part232/234/235](https://www.steakunderwater.com/VFXPedia/__man/Resolve18-6/DaVinciResolve18_Manual_files/part232.htm) / [FCP playback](https://support.apple.com/guide/final-cut-pro/playback-settings-verb8e60ab7/mac) / [AE previews 13.5](https://helpx.adobe.com/after-effects/kb/all-about-previews-in-after-effects-cc-2015--13-5-.html) / [Blender Timeline](https://docs.blender.org/manual/en/3.4/editors/timeline.html) / [Alight Motion](https://support.alightmotion.com/hc/en-us/articles/10537513451793-Performance-Guide) / [Cavalry viewport](https://docs.cavalry.scenegroup.co/user-interface/menus/window-menu/viewport/)
制御則: [Arntzen 2012 (libretro)](https://docs.libretro.com/guides/ratecontrol.pdf) / [RetroArch audio_driver.c](https://github.com/libretro/RetroArch/blob/master/audio/audio_driver.c) / [bsnes DRC記事(アーカイブ)](https://github.com/higan-emu/emulation-articles) / [Dolphin 5.0 Mixer](https://github.com/dolphin-emu/dolphin/blob/5.0/Source/Core/AudioCommon/Mixer.cpp) / [UE Dynamic Resolution doc](https://dev.epicgames.com/documentation/en-us/unreal-engine/dynamic-resolution-in-unreal-engine) / [Intel DRR (GDC 2011)](https://www.intel.com/content/dam/develop/external/us/en/documents/dynamicresolutionrendering-183334.pdf)
Rust部品: [rubato](https://github.com/HEnquist/rubato) / [signalsmith-stretch](https://github.com/Signalsmith-Audio/signalsmith-stretch) / [signalsmith-stretch-rs](https://github.com/colinmarc/signalsmith-stretch-rs) / [cpal timestamp.rs](https://github.com/RustAudio/cpal/blob/master/src/timestamp.rs) / [fixed-resample](https://codeberg.org/Meadowlark/fixed-resample)
知覚: [audio pull up/down 解説](https://javierzumer.com/blog/2019/4/28/figuring-out-audio-pull-updown)(4%=顕著、0.1%=不可知)
