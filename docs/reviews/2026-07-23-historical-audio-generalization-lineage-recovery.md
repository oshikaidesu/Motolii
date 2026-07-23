# Audio generalization lineageの価値回収（Unit 5B、2026-07-23）

状態: **設計維持／到達表示訂正**（cutoff 6 historical blobの処分完了）

対象: [音声一般化設計](2026-07-14-audio-generalization-design.md)のcutoff全6版。

関連: [M2仕様](../specs/M2-document-model.md)、[M3仕様](../specs/M3-ui-integration.md)、[D5先例](2026-07-14-d5-transport-prior-art.md)、[UI runtime責任境界](../ui-runtime-architecture.md)、[backlog](../backlog.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

6版で恒久意味は変わっていない。初版がAsset stream、Clip component共有時間、音声分離、Track非media-kind、48 kHz stereo f32 mix、preview/export同一意味、AG-5非目標を決め、後続5版はAG-0、AG-1、AG-2、AG-4、AG-3 domainの到達表示だけを更新した。

現行コードはAG-1、AG-2のmixer core、AG-3 domain、AG-4 exportを実装している。しかし製品`motolii-transport::PlaybackSession`は今も単一`Arc<PcmCache>`から`AudioProducer`を起動し、`AudioProgram`／`MixProducer`を受けない。`AudioProgram`の利用はtestとexportに限られる。したがって歴史版の「AG-2完了」をcore成立と製品接続へ分け、後者をGAP-28へ戻す。

また初版§6.2の「rendererが遅い時はrender進捗を主にしてmixed audioを適応resampling」は、同日の後続D5が明示棄却した。現行正本はaudio device clock常時主、video frame／古い要求のdropである。旧Slint UI名も現在のReact chrome＋native Timeline所有へ置換する。

## 2. 全6版の差分

| 版 | 差分 | 処分 |
|---|---|---|
| 1 | 全設計とAG-0〜5の初期状態 | 本文意味の正本。後続決定と現行コードで再判定 |
| 2 | AG-0とAG-1を完了、#156を記録 | 歴史到達。現行schema／media fixtureでも成立 |
| 3 | AG-2を完了、#157を記録 | mixer coreの成立として維持。製品Transport接続の証明には使わない |
| 4 | AG-4を完了、#159を記録 | 現行mixed encode／fast path／preview-export PCM fixtureで成立 |
| 5 | AG-3をdomain完了＋UI follow-up、AG-5を追跡のみへ変更 | domain／UI分離を維持。Slint固有語彙は撤回 |
| 6 | AG-3を進行中へ戻し、別lane拒否とordinal一意を追記 | 「UIまで完了していない」訂正を維持。現行surface ownerへ再翻訳 |

中間版に本文field、mix順、互換default、非目標の追加・削除はない。進捗checkboxを設計意味の変更として水増ししない。

## 3. 生存する恒久意味

- 「完成済み楽曲1本」は既定ワークフローであって恒久Document制約ではない。
- Assetはcontainerで、stream identityはkind＋同kind内ordinal。欠落時はtyped errorで別streamへfallbackしない。
- 旧Asset Clipのselector欠落はvideo ordinal 0／audioなし。既存projectを開いただけで音を出さない。
- Asset Clipは0/1 videoと0+ audio componentの`start`、`duration`、`TimeMap`を一つだけ所有する。
- 音声分離は新しいaudio-only Clipをmaterializeし元audioを無効化する1 macroで、fresh IDを使い、隠れ同期linkを残さない。
- Trackは配置laneのままで、audio／video別schemaへ再解釈しない。同一lane overlap禁止の実装未到達は既存GAP-18の責任であり、本Unitで別規則を発明しない。
- audio component初期面はstream、enabled、linear nonnegative gain、silence／loopだけ。Freezeを音声へ流用しない。
- mix境界は48,000 Hz／stereo／interleaved f32。Soundtrack→Track→item→component ordinal順で加算し、sourceごとのclamp／自動normalize／limiterを入れない。
- previewとexportは同じ`mix_audio`意味を使う。stream copyは無加工の単一Soundtrackだけに限定する。

## 4. 現行コード到達

| 面 | 2026-07-23 live判定 |
|---|---|
| AG-1 schema／probe | **実装済み**。kind＋ordinal、legacy default、component所有、typed拒否がある |
| AG-2 canonical decode／mix | **実装済み**。`AudioProgram`、`MixSource`、`mix_audio`、`MixProducer`と決定論fixtureがある |
| AG-2 製品Transport | **未実装**。`PlaybackSession::open_*`は`Arc<PcmCache>`を取り`AudioProducer`を起動する。`MixProducer`の製品callerが無い |
| AG-3 domain | **実装済み**。import source構築、waveform peaks、mute／gain command、別laneへのdetach macroと拒否fixtureがある |
| AG-3 product UI | **未実装**。React import／formとnative Timeline audio row／waveformを同一Host projection／intentへ接続する成果は無い |
| AG-4 export | **実装済み**。Documentから`AudioProgram`を作りmixed PCMをencodeし、限定fast pathと一致fixtureを持つ |
| AG-5 | **延期**。候補一覧であり一括実装の承認ではない |

core testで`MixProducer`がringへ供給できることは、製品`PlaybackSession`がmixed programを再生する証拠ではない。逆に製品接続不足を理由にmixer、schema、exportの成立を巻き戻さない。

## 5. Transport訂正

D5の採択はaudio device clock常時主、video drop、timestamp queryが使える場合だけの適応解像度である。device sample rateとcanonical 48 kHzの固定比変換、Clip retimeとしてユーザーが明示したvarispeedは別責任で、許される。

重いrendererを理由に再生速度やpitchを自動変更する案は復活させない。GAP-28は既存D5のclock owner、`PlaybackCounters`、`DeviceWaitLatency`、non-blocking callbackを保ったまま、producer入力だけをmixed `AudioProgram`へ一般化する。製品入口でSoundtrack-only、複数Clip audio、seek、10分drift、preview/export意味を審判する。

## 6. UI再翻訳

歴史版のSlint import dialog／waveform follow-upは画面意味の候補であってtoolkit契約ではない。現行ownerは次である。

- React chrome: Video + Audio／Video Only import dialog、stream選択、form型gain編集、説明と診断。
- native Timeline: 行と同じscroll／zoom／selectionへ同期するaudio component展開、waveform、mute、detachの直接操作。
- Host: revision付きread-only projection、typed intent、D2 macro、single writer、Undo、selection正本。

Reactとnativeへaudio state、selection、Undoを二重保存しない。DOM／CSS／waveform geometry／pixel値をDocumentやplugin契約へ焼かない。AG-3はこの製品接続まで未完了とするが、現行Selected U seriesやG0-9停止線を追い越さない。

## 7. GAP-28の再入場条件

1. `PlaybackSession`の入力ownerをDocument snapshot→`AudioProgram`へ閉じ、UIからraw cacheやmixerを操作させない。
2. callbackはringから読むだけに保ち、decode、I/O、allocation、blocking lockを持ち込まない。
3. audio device clock常時主、video generation drop、device waitだけの補償を維持する。
4. seek時は古いproducer／generationを破棄し、同時に第二clockや第二再生headを作らない。
5. Soundtrack-only互換、2 source mix、Clip trim／retime、100 seek、10分drift、underflow、preview/export sample意味を製品入口で固定する。
6. Document schema、mix順、canonical format、AG-5候補、UI surface契約へ範囲を拡張しない。

## 8. 復活させないもの

- renderer遅延を理由にrender進捗をmaster clockへ昇格し、音声を自動varispeedすること。
- Slint／eguiを現在の製品UI正本として戻すこと。
- audio／video別Track schema、別project mode、componentごとの独立時間、隠れ同期link。
- 指定stream欠落時のordinal fallback、audio Freeze、sourceごとのclamp／normalize／limiter。
- core `MixProducer` fixtureだけで製品Transport接続と10分A/V driftを完了扱いすること。
- AG-3 UI未実装を理由にdomain command／waveform peaksの成立を撤回すること。
- fade、pan、role、bus、audio effect、pitch preserveをAG-5という一括機能として実装すること。

## 9. 固定歴史出典とcoverage

初版`6787365f`を全文で読み、後続5版の全差分と最終版`d7c30673`の本文を確認した。処分した6 unique blob（90,564 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/05b-audio-generalization.tsv`を正本とする。cutoff総数1,797のうち処分済みは348、未処分は1,449である。
