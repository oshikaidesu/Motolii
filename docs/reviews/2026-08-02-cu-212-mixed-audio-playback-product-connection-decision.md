# CU-212 — mixed AudioProgram playback 製品接続決定

状態: **縮小採用**（2026-08-02）

## 目的

固定Mac Local Alphaの通常製品windowで、Document由来の既存`AudioProgram`／`MixProducer`を`PlaybackSession`へ接続する。音声callbackは従来どおりringから読むだけとし、decode／mixはproducer workerへ置く。

## 接続票

| 項目 | 正本 / 接続 |
| --- | --- |
| AUTHORITY | [音声一般化設計](2026-07-14-audio-generalization-design.md) §6.2、D5。audio device clockを主とし、`AudioProgram`のmixed PCMをTransportへ供給する |
| INTERNAL TARGET | `motolii_audio::AudioProgram::from_document`、`MixProducer::spawn_with_device_rate`、`motolii_transport::PlaybackSession`、`ProductApp::toggle_playback` |
| OWNER | Document由来sourceの構築はProductApp初期化前、decode／mixとring供給は音声producer worker、clock／pause／play／playheadは既存PlaybackSession／Transport |
| WRITE ROUTE | 新しいDocument commandやReact stateは作らず、既存Stage strict `toggle-play` intentからProductAppを起動する |
| GAP | 既存PlaybackSessionが単一`PcmCache`／`AudioProducer`だけを受けていたため、mixed `AudioProgram`をdevice rateへ接続する薄い入口が欠けていた |
| RESOLUTION ROUTE | `REUSE`（AG-2の既存mixer／producer）→ `REMAP`（PlaybackSessionのproducer保持をprivate enumへ交換）→ `REDUCE`（音声UI、continuous scrub、waveformは持ち込まない） |
| DISPOSITION | `PASS` |

## 採用範囲

- `PlaybackSession::open_default_program`／`open_on_device_program`を追加し、既存のdevice negotiation、共有`PlaybackCounters`、Transport origin変換を共通経路へ残す。
- `AudioProgram::pad_to_duration`でcomposition尺までgain 0の正規silence sourceを足し、短い音源でもcpal clockがunderrunで停止しないようにする。既存のsource順、gain、master gain、Document意味は変更しない。
- `ProductApp`初期化時に既存Documentから`AudioProgram`を一度だけ構築し、sourceがある時だけmixed producerを使う。Rectangle／音声sourceなしは既存composition尺の無音`PcmCache` fallbackを維持する。
- startup前のdecode、再生中のmixはevent loop内で実行しない。UI callbackはstrict intentとprojectionだけを扱う。`AudioProgram`はsession startup snapshotとして保持し、音声を含むClipの編集後に同じsessionで再bindingする経路は次粒へ送る。

## 非目標 / 残件

- audio import UI、Soundtrack／Clip audioのauthoring、waveform、audio row、mute／solo／gain UI、device選択、continuous scrub、10分drift測定は別粒。
- 音声を含むClipのmove／trim／Undo後にproducerを非同期再構築して再bindingすることは別粒。現行製品面は音声authoringを公開しておらず、startup snapshotのstalenessをcompletion evidenceへ一般化しない。
- Save／reopen／Exportの完了条件はCU-211のまま。mixed音声をExportの検収へ拡張しない。
- `AudioProgram`を新しい公開Document schemaへ変更しない。runner、AGENTS.md、route契約、model順序も変更しない。

## 必須負例 / 検証

- 既存single-PCM `PlaybackSession::open_default`が同じ共通device／Transport経路でコンパイル・テストを通ること。
- mixed sourceを持たないRectangle projectが既存無音fallbackで再生できること。
- compositionより短い音源はpadding sourceにより終端まで供給され、clockが音源終端で停止しないこと。
- decode／mixの失敗はtyped `AudioError`としてProductApp初期化から返し、event loop内で推測fallbackしないこと。
- `cargo test -p motolii-audio --test mix_program --quiet`、`cargo test -p motolii-transport --quiet`、`cargo test -p motolii-ui --lib --quiet`、対象crate clippy `-D warnings`を通過すること。
- 通常Mac製品windowで10秒WAVを含む固定projectを開き、Stageの`再生`を押すとtraceに`kind=transport source=mixed sources=1`が出て、timecodeが`00:00.1`から`00:10.0`へ進み、buttonが`再生`へ戻ること。

## 検収記録

- 通常製品window（`com.motolii.cu211`）で10秒／48 kHz stereo sine WAVを含む一時projectを使用した。`source=mixed sources=1`、再生開始、終端`kind=transport action=ended time=10s`を確認した。
- `cargo check -p motolii-audio -p motolii-transport -p motolii-ui`、対象crate clippy、audio mix／transport／UI lib tests、`git diff --check`を通過した。
- Opus 5（`claude-opus-5`、effort=low、fresh read-only）はbounded diffを再確認し、video-only分岐、composition尺padding、producer lifetime、clock/rate、typed error、scopeを監査した。`VERDICT: ACCEPT`、`P0=0`、`P1=0`。初回指摘だった音声なしprojectの分岐誤りと尺分zero allocationは修正済み。
- CU-212は、既存のAG-2 coreを製品Transportへ接続する一粒として閉じる。長尺drift、export audio mux、authoring UIの証拠へ一般化しない。

## ルート運用

Opusは速度優先のbounded read-only検収（`claude-opus-5`、low effort、fresh session）を一回行う。runner、AGENTS.md、route契約、receipt形式は変更しない。
