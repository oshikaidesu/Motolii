# 音声を「楽曲1本」から一般メディアへ拡張する設計(2026-07-14)

ステータス: **【設計決定】**。M2の実装完了条件は増やさない。現行`Soundtrack`を壊さず、音付き動画・音声のみ素材・複数同時音声へ追加的に拡張する意味論と実装順を固定する。

## 1. 結論

「MVでは完成済み楽曲1本」を**既定ワークフロー**として残すが、恒久的なDocument制約にはしない。

採るモデル:

- Assetはコンテナであり、video/audio等の複数streamを持ち得る
- 1つのtimeline Clipが、選択されたvideo/audio componentを所有する
- Clipの`start` / `duration` / `TimeMap`は全componentで共有する
- 音付き動画を移動・trim・retimeすると、映像と音声は常に一緒に動く
- 独立編集が必要な時だけ「音声を分離」コマンドでaudio-only Clipを作る
- `Soundtrack`はプロジェクト全体のmusic bedという簡易入口として存続し、clip audioと同じmixerへ入る
- MV向け／一般向けの違いはimport時の既定選択だけ。Documentの別形式やproject modeにはしない

新しい音声機能をM2へ割り込ませない。先に意味を決め、実装は§9の独立レーンで行う。

## 2. なぜ現行のままでは足りないか

現行は次を恒久決定のように書いている。

- `Document.soundtrack: Option<Soundtrack>`が音声の全て
- 動画内蔵音声は常にmute
- 楽曲をPCMへ全展開し、単一bufferとしてTransportへ渡す
- exportは元楽曲のstream copyを優先する

これはMVの最短出口としては正しいが、音付き動画を普通にtrim・移動するだけでも音声が追従できない。一般化を後回しにしても、現在の禁止を恒久意味論にはしない必要がある。

一方、既存の次の境界はそのまま使える。

- 一般`Asset`とcontent hash
- `RationalTime`、半開区間、Clipの`TimeMap`
- 単一writer、command、macro/Undo
- 音声device clockを主とする単一Transport
- preview/exportで同じ評価意味を使う規律
- version/min_reader_version/D1e migration

## 3. 先例

### GStreamer Editing Services

[GESClip](https://gstreamer.freedesktop.org/documentation/gst-editing-services/gesclip.html)は1つのClipがaudio/video等の複数`TrackElement`を制御し、子のstart/durationをClipと同一に保つ。Clipの時刻変更は全componentへ伝播する。Motoliiはこの意味を採る。

ただしGESの内部Track/Layer階層やGObject APIは輸入しない。Motoliiの`Track`は既存どおりユーザーが項目を並べるlaneであり、Clipからvideo render入力とaudio mix入力を別々に投影する。

### OpenTimelineIO

[OTIO](https://opentimelineio.readthedocs.io/en/latest/)はClipを外部media参照+`source_range`として扱い、Trackにaudio/videoのkindを持てる。Motoliiも素材そのものをDocumentへ埋めず、Asset参照と有理数の編集判断だけを保存する。

OTIO互換のため、audio componentも独自のframe indexやsample indexをDocumentへ焼かず、既存の`RationalTime`で時刻を表す。

### FCPXML

[FCPXML](https://developer.apple.com/documentation/professional-video-applications/creating-fcpxml-documents)は1つのasset clipでmedia assetと編集上のstart/durationを表し、audio roleやaudio filterを同じclip系要素へ持たせる。音付き素材を最初から別ファイル扱いにしない点を採る。role/bus/高度なaudio effectは初期一般化へ入れない。

## 4. 恒久意味論

### 4.1 Assetとstream

Assetはファイルではなく、content hashで同一性を持つmedia containerである。1 Assetは0個以上のvideo streamと0個以上のaudio streamを持ち得る。

永続するstream選択は次の意味を持つ。

| 項目 | 意味 |
|---|---|
| `kind` | `video` / `audio`。未知kindへ黙って縮退しない |
| `ordinal` | 同じkindのstreamをcontainer順に0から数えた番号 |
| Asset同一性 | content hashが同じ限り同じordinalを同じstreamとみなす |
| 欠落 | 指定streamが無ければtyped error。別streamへ自動fallbackしない |

旧`ClipSource::Asset { asset }`にstream選択が無い場合は、**video ordinal 0のみ、audioなし**として読む。既存projectを開いただけで音が出始める意味変更は禁止する。

新規importはprobe結果から選択を明示して保存する。複数audio streamがある場合、UIはstream名・言語・channel数を表示して選ばせる。表示metadataは再probe可能なcacheであり、Documentの正本はkind+ordinalだけとする。

### 4.2 Clipは複数componentの時刻所有者

Asset Clipは次のcomponent集合を持てる。

| component | 個数 | 出力 |
|---|---:|---|
| video | 0または1 | RGBA texture。無ければvisual render graphへ参加しない |
| audio | 0以上 | canonical PCM。無ければaudio mixerへ参加しない |

全componentはClipの`start`、`duration`、`TimeMap`を共有する。componentごとの独立start/durationは持たせない。

共有するのはTimeMapのsource時刻写像である。写像後の時刻がsource範囲外だった場合、既存`TimeMap.overrun_mode`はvideo componentへ適用し、audio componentは§5の`out_of_range`を適用する。`Freeze`/`Black`というvideo語彙をaudioへ解釈し直さない。

これにより:

- video+audio = 普通の音付き動画
- videoのみ = MV背景素材、静止画等
- audioのみ = 効果音、ナレーション、独立した楽曲clip
- video+複数audio = container内の複数streamを明示的に同時利用

Vector/Plugin sourceは初期状態ではaudio componentを持たない。将来Generatorが音声を出す場合は別の公開契約を先に決め、Asset streamを装ってはならない。

### 4.3 分離は永続リンクではなくmaterialize command

import直後のvideo/audioは1 Clipなので、リンクID・隠れNull・同期controllerを必要としない。

「音声を分離」は次の1 macro commandとして定義する。

1. 元Clipと同じAsset/start/duration/TimeMapを持つaudio-only Clipを作る
2. 元Clipのaudio componentを無効化する
3. 新Clipへ新しい非再利用IDを割り当てる
4. Undo 1回で分離前の1 Clipへ戻す

分離後は2 Clipが独立する。同期を維持する隠れlinkは作らない。必要なら通常の複数選択・group・macro操作で同時編集する。

### 4.4 Trackの意味は変えない

Documentの`Track`は配置laneであり、media kindではない。`audio track`/`video track`という別スキーマへ既存Trackを再解釈しない。

- 同一Track内の時間重なり禁止は維持
- 別Trackのaudio componentは同時に鳴り、mixerで加算される
- visual componentは既存のlayer/composite順へ参加する
- audio-only Clipはvisual layer順へ影響しない
- UIがaudio laneだけを抽出表示することはprojectionであり、Documentに別の複製を作らない

## 5. Audio componentの最小パラメータ

初期一般化で保存するのは次だけ。

| 項目 | 型/意味 | 既定 |
|---|---|---|
| stream | kind=`audio`+ordinal | 必須 |
| enabled | bool | true |
| gain | `DocParam<F64>`、linear、有限、0以上 | 1.0 |
| out_of_range | `silence` / `loop` | silence |

pan、fade、role、bus、audio effect stack、pitch preserveは初期一般化へ入れない。必要になった時に追加的field/variantとして意味論表とともに足す。

Audioのsource範囲外でvideoの`Freeze`を流用して最終sampleを保持してはならない。長いDC値を生成するためである。Audioは既定`silence`、明示時だけ`loop`とする。

`gain`はcanonical sample時刻ごとに既存DocParam評価器で評価する。初期実装が最適化のためbuffer端点評価やramp化を行う場合も、sampleごとの参照結果と許容誤差内で一致させる。

ClipのTimeMap速度が1以外の場合、初期実装はvarispeedとし、速度に応じてpitchも変わる。pitch-preserving stretchは別variantを追加するまで未対応。黙って高品質stretchを名乗らない。

## 6. MixerとTransport

### 6.1 canonical mix境界

初期一般化の内部mix形式は **48,000 Hz / stereo / interleaved f32** とする。各source streamはproducer workerでdecode/resample/channel-mapし、PCM cacheへ置く。音声callback内でdecode、I/O、allocation、lock待ちを行わない。

評価順は次で固定する。

1. `Soundtrack` music bed
2. DocumentのTrack順
3. Track内item順
4. Clip内audio component ordinal順

各sourceをf32で加算し、clip gain、最後にmaster gainを適用する。自動normalize/limiterは行わない。内部mixは`[-1,1]`へ毎source clampせず、device/encoderの最終sample形式へ変換する境界だけで範囲処理する。

gapとsource範囲外はsilence。mix結果とunderflowを区別し、underflow時は既存規約どおり無音を出すがカウンタへ記録する。

### 6.2 Transportは作り直さない

Transportの入力を「単一Soundtrack PCM」から「AudioProgramのmixed PCM」へ置き換える。クロック規約は不変。

- 通常: audio device clockが主、videoが追従
- rendererが実時間未満: render進捗が主、mixed audioが適応resamplingで追従
- 再生ヘッド所有者は常に1つ

Clip retimeのvarispeedと、重いpreview時のTransport varispeedは別段であり、混同しない。

### 6.3 preview/export同一意味

previewとexportは同じ`mix_audio(range, format)`意味を使う。device buffer sizeやencode chunk sizeでsample結果が変わらないことを審判する。

exportでstream copyしてよいのは、次を全て満たす時だけ。

- audible sourceが`Soundtrack` 1本だけ
- gain=1、offset以外の加工なし
- container/codecが出力と互換

clip audio、複数source、gain automation、retimeのどれかがあればmixed PCMをencodeする。加工済み音声を元素材のstream copyで黙って置き換えない。

## 7. UIの単純化

動画import時の選択肢は2つに限定する。

- **Video + Audio**: 検出したvideo/audioを1 Clipとして配置
- **Video Only**: audio componentなしで配置

前回選択はUser settingsに保存してよいが、Documentの意味にはしない。MV向けworkspace/presetはVideo Onlyを初期選択にできるが、project fileの別modeは作らない。

Clipは既定では1行。必要時だけaudio部分を展開し、waveform、mute、gain、「音声を分離」を表示する。audio-only Clipも同じtimeline itemとして扱う。

音声streamが無い素材では選択肢を出さない。複数streamではAdvancedで選択できるが、通常導線はordinal 0を選んだVideo + Audioの1操作とする。

## 8. 互換性

### 8.1 現行project

- 現行`Soundtrack`は削除・再解釈しない
- 現行Asset Clipはstream指定欠落=video ordinal 0/audioなし
- 既存projectはpixel・音声とも変更前と同一
- 新しいnested fieldを実装する版では`min_reader_version`を上げ、旧readerによる再保存消失を防ぐ

`Soundtrack`は将来もmusic bedとしてAudioProgramへ入力できるため、強制migrationは不要。ユーザーがtimeline clipへ変換したい場合だけ、media probe後の明示commandでaudio-only Clipを作る。

### 8.2 欠落と未対応

- 指定stream欠落: typed error
- unsupported codec/channel layout: typed error
- audioを持てないPlugin/Vector sourceへaudio設定: validate error
- pitch preserve指定を未知のままvarispeedへ縮退: 禁止
- mixer未実装のreaderがaudio component付きprojectを開く: `min_reader_version`で拒否

## 9. 実装フェーズ

| ID | フェーズ | 内容 | 完了条件 |
|---|---|---|---|
| AG-0 | M2終了前・仕様のみ | 本文とM2の「楽曲1本=恒久制約」を改訂 | コード/serde変更なし。現行M2タスクを増やさない |
| AG-1 | v1.x schema/media | probeを全stream列挙へ。stream selector+Asset Clip componentを追加。旧欠落default=video only。min_reader_version/D1e規律 | 旧project意味不変、roundtrip、欠落stream拒否、video/audio/audio-only fixture |
| AG-2 | v1.x audio engine | per-stream PCM cache、canonical変換、deterministic mixer、AudioProgram→Transport | 2 source mix、seek、10分A/V drift、callback非blocking、chunk size不変 |
| AG-3 | M3 UI | Video+Audio/Video Only import、waveform展開、mute/gain、音声分離macro | move/trim/retime追従、分離Undo 1回、保存再読込一致 |
| AG-4 | export | 単一bed stream-copy fast pathとmixed encode path | fast path sample一致、mix時にstream-copyしない、preview/export PCM一致 |
| AG-5 | later | fade/pan/role/bus/audio effect/pitch preserve | 各意味論表と需要確認後。初期一般化のblockerにしない |

依存: AG-1→AG-2→AG-3/AG-4。AG-1はDocument解凍手続き(理由・追加的変更・旧project審判)を独立PRで通す。M2 Wave4へ割り込ませない。

## 10. 必須受け入れコーパス

1. video only MP4
2. video+AAC MP4
3. audio only WAV
4. video+audio 2stream/language container
5. sample rate 44.1kHzと48kHzの同時mix
6. mono+stereoの同時mix
7. 10分素材のmove/trim/seek/retime
8. source audio範囲を越えるClip(silence/loop)
9. missing stream / unsupported codec
10. 旧Soundtrack-only project

審判:

- video+audio Clipは任意のmove/trim後も開始sampleとframe PTSの対応が有理数式どおり
- 100回seekでaudio callback/UI送信がblockしない
- 同一Documentのpreview/export mixがchunk分割によらず一致
- 音声分離前後でPCMが一致し、分離後の移動だけが独立する
- 旧projectの映像goldenとSoundtrack export sampleが不変

## 11. 非目標

この一般化はMotoliiをDAWや汎用NLEにする宣言ではない。

- 録音
- MIDI
- VST/Audio Unitホスト
- 高度なbus routing
- surround/immersive audio
- dialogue cleanup/mastering
- multicam/sync by waveform

これらはpluginまたは将来判断。初期一般化の目的は「音付き動画を置き、切り、動かしたら音も正しく付いてくる」と「複数素材を最低限mixできる」までとする。

## 12. 既存決定への影響

| 既存決定 | 判定 |
|---|---|
| 楽曲1本のMV導線 | 維持。既定UI/`Soundtrack` music bedとして残る |
| 動画音声は常時mute | **恒久決定からM2実装範囲へ縮小** |
| 単一Transport/audio主clock | 維持。入力だけmixed AudioProgramへ一般化 |
| Asset一般化 | そのまま利用 |
| TimeMap | 全componentで共有。audio out-of-rangeだけ別語彙 |
| preview/export同一関数 | audio mixerにも適用 |
| M2締結 | AG-0文書だけ反映し、実装タスクは増やさない |
