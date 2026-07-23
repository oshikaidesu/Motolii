# D5 Transport lineage全4版の価値回収（Unit 5D）

状態: **決定／現行再照合済み**

対象: `docs/reviews/2026-07-14-d5-transport-prior-art.md` のcutoff全4版（103,177 bytes）

関連: [M2仕様](../specs/M2-document-model.md)、[M3仕様](../specs/M3-ui-integration.md)、[performance model](../performance-model.md)、[音声一般化回収](2026-07-23-historical-audio-generalization-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

4版は、旧「rendererが遅い時はTransport clockをrender進捗へ交代し、音声を自動varispeedする」案を、一次資料との不一致から撤回した系譜である。生き残る決定は次の一組であり、現行M2仕様と`motolii-transport`骨格はこれに一致する。

1. audio device clockを常時主とし、videoは古い時刻を初めから手掛けず最新の聴感時刻へdropで追従する。
2. 実時間回復はGPU timestamp queryを正本にしたDraft 1/2→1/4のDRSで行い、計測不能時は自動DRSだけを無効化してdropを続ける。
3. 補償で引くのはdevice waitだけで、ring充填量を二重に引かない。
4. device rate変換はD4-FU producer側で閉じ、algorithm latencyをTransportへ持ち出さない。

ただしD5は製品完成ではない。現行コードにはcontroller、simulation、共有counter／device wait、headless render接続がある一方、本番preview loopからのGPU計測、U5接続、実機10分E2E、mixed `AudioProgram`製品接続が未成立である。既存D5 pendingとGAP-28を維持し、新しいGAPは増やさない。

## 2. 全4版の変遷

| 版 | 当時の状態 | 現在の処分 |
|---|---|---|
| 1 | 先例調査、採択待ち | 調査結果と撤回理由を維持。提案段階を現行決定と誤読しない |
| 2 | ユーザー採択を記録 | clock固定、video drop、自動varispeed撤回を決定として維持 |
| 3 | 反対側レビューでdevice wait、D4-FU、timestamp非対応縮退、機械審判を補正 | 補正後だけを現行規範とする |
| 4 | resampler algorithm latencyをproducer pre-roll／先頭trimへ閉じた | 現行D4-FU責任として維持し、Transport補償へ再混入させない |

最初の全文と各親子差分を確認した。版2は採択状態だけを変え、版3が実質的な境界修正、版4がresampler latencyの最終責任を確定している。

## 3. 成立理由として残すもの

### 3.1 二つ目のclockを作らない

mpv／ffplay等の音声rate補正はdisplay同期や非audio-master構成の微調整であり、重いrendererを理由に0.5x級へ自動低速化する先例ではない。Motoliiではframeが時刻`t`の純関数なので、遅れたframeを順に消化せず、現在のaudio時刻だけを次に評価できる。この構造が、音のtempo／pitchを嘘にせずvideoだけを省略できる理由である。

### 3.2 DRSは作品意味でなくTransientな縮退

DRSの段階、閾値、実表示fpsはDocument、journal、Undo、plugin parameter、Final出力へ入れない。GPU計測が無ければwall timeを代用せず固定Draft＋dropへ縮退する。これは「対応GPUだけ正しい」のではなく、全環境でclock意味を保ち、可能な環境だけ表示品質を自動調整する境界である。

### 3.3 latency責任を一度だけ数える

供給済みsampleを時計起点にした時、ring内sampleは未来であり、device waitとring充填量を両方引くと二重補償になる。固定比resamplerの内部遅延もD4-FU producerが開始／seek時に吸収し、Transportの引き算へ加えない。

## 4. 現行コード事実

| 面 | 現在の事実 | 判定 |
|---|---|---|
| clock | `Transport::perceptual_frames`は`frames_supplied - device_wait`だけを使う | 骨格成立 |
| drop | `next_frame_plan`は最新時刻をfloorし、前回との差を`dropped_frames`へ記録する | 骨格成立 |
| DRS | `DrsController`はmeasured GPUだけを見て、CPU-bound／unmeasuredを降格理由にしない | controller成立 |
| render接続 | headless testが`FramePlan`のtime／qualityを`render_frame`へ渡す | test接続のみ |
| product計測 | render passの`timestamp_writes`は未配線で、製品loopから`record_render_timing`へ実測を渡す経路が無い | 未実装 |
| product playback | `PlaybackSession`は単一`PcmCache`＋`AudioProducer`を所有する | GAP-28 |
| UI | U5／U1gの製品再生、drop表示、最新generation投影はpending | 未実装 |
| E2E | simulationの10分driftはあるが、実device＋product previewの10分審判ではない | 完了根拠にしない |

したがって#144を「D5完了」と再表示しない。骨格の存在は、製品計測・製品loop・実機clockの閉包を証明しない。

## 5. 維持する再入場条件

- 明示的なscrub／shuttle／play-every-frameを将来追加する場合だけ、pitch変化またはpitch保存time stretchを別機能として比較する。renderer overloadの自動fallbackへ戻さない。
- DRSを製品へ接続する時は、実GPU timestamp、CPU-bound対照、非対応縮退、最小滞留内の再復帰0、audio underrun増加0を同じfixtureで審判する。
- U5／U1gではTransport時刻、project fps、audio pitchを不変にし、古いgenerationだけを捨てる。drop indicatorはTransient投影で、Documentへ保存しない。
- GAP-28はproducer入力をmixed `AudioProgram`へ一般化しても、同じcounter、device wait、non-blocking callback、単一clockを維持する。

## 6. 復活させない負例

- rendererが遅い時にclock ownerをrender進捗へ交代する。
- 自動0.5x／0.6x varispeedをmpv／ffplayの確立手法と呼ぶ。
- ring充填量、resampler latency、device waitをTransportで合算して引く。
- GPU timestampが無い時にwall timeで自動DRSを続ける。
- DRSの閾値や段階をDocument／plugin契約／cache identityへ焼く。
- simulation green、headless render接続、`drs_available`だけで製品DRSを完了扱いする。
- dropをproject fps変更、Final frame省略、音声clock停止として実装する。
- D5を理由にnative API、CPU fallback、第二clockを公開契約へ追加する。

## 7. 固定sourceとcoverage

| blob | bytes | lineage上の意味 |
|---|---:|---|
| `f4dddc3d3325d7268410e83bf535bd57b3769c95` | 26,138 | 調査完了・採択待ち |
| `47be9e0b72b4815bbad6c50fbf3b466fd81066c5` | 26,075 | ユーザー採択記録 |
| `3ef1a6862817b226765796a3643d116d61eedd5d` | 25,448 | レビュー境界修正 |
| `d504a96064f1120ddfdfa24d22b21426bfa6b536` | 25,516 | D4-FU latency責任確定・現行版 |

本Unitで4 blobをDISPOSITIONEDへ移す。厳密進捗は356 / 1,797、残り1,441である。
