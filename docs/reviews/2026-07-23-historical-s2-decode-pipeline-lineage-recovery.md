# S2 decode pipelineの価値回収（Unit 5I、2026-07-23）

状態: **自前ffmpeg process採択維持／VFR・長尺・cancel保証未成立**

対象: [S2 decode spike](../spikes/s2-decode.md) cutoff全6版（11,586 bytes）

関連: [M0仕様](../specs/M0-spikes.md)、[GAP-26](../backlog.md)、[Rerun学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

S2の採択は、ffmpegを外部processとして使うことと、Rustの`ffmpeg-sidecar`クレートへ依存することを分けた点に価値がある。Motoliiは後者を不採用とし、ffprobe JSONとffmpeg rawvideo pipeを自前管理する。現行`motolii-media`もこの判断を維持し、raw YUVを`motolii-gpu`で変換し、CFR素材のframe正確seekを成立させている。

ただしS2の完了は製品decode lifecycle全体の完成ではない。現行VFR判定は`r_frame_rate`と`avg_frame_rate`の差が0.5%を超える素材を拒否するheuristicで、timestamp正本や正規化ではない。長尺／4K throughput、bounded decoder pool、prefetch、block中readの確実なkill、continuous stderr drainも未成立である。VFR／source正規化はK4、process停止とstderrはGAP-26へ分離する。

## 2. 6版の差分と処分

6版は本文の採否を反転していない。S1／S3状況、関連link、表現の追補を含む同一lineageとして一括処分する。

| 歴史主張 | 処分 |
|---|---|
| `ffmpeg-sidecar`クレートを採用しない | **維持**。referencesとRerun文書の「本命」を訂正する |
| ffprobe JSON＋固定長rawvideo pipeを自前実装 | **維持**。process管理まで含む万能完成とは呼ばない |
| raw YUVをGPU色変換へ渡す | **維持**。CPU rgba変換を並行正本にしない |
| 半frame手前の入力`-ss`でCFR seek | **維持**。CFR fixtureの成立範囲に限定する |
| VFR挙動はM4へ先送り | **維持**。現行heuristic拒否をVFR対応と数えない |
| 長尺／高解像度throughput未検証 | **維持**。小fixtureのgreenを性能保証へ外挿しない |
| hardware decodeはv2 | **維持**。native decoder／zero-copyを現在の公開契約へ足さない |

## 3. 現行コードとの照合

- `probe_media`はffprobe JSONをtyped `MediaInfo`へ変換し、rotation、color metadata、frame rateを読む。
- VFRは`r_frame_rate`と`avg_frame_rate`の差が0.5%超ならtyped errorにする。差が小さいVFR、timestamp列、edit timelineへの正規化を証明しない。
- `FrameReader`はffmpegへ`-f rawvideo -pix_fmt yuv420p`を指定し、frame sizeだけ`read`して`CpuFrame`を作る。色変換はGPU側へ残る。
- `next_frame`は`Arc<Mutex<Child>>`を保持したままstdout readでblockし得る。`FrameReaderKillHandle::kill`も同じmutexを必要とするため、stalled readを別threadから必ず解放できるというcomment／testの意図はコード構造だけでは成立しない。
- stderrはstream終端の`check_child_exit`で読む。長時間decode中のstderr floodとcancel teardownは未証明である。
- `framereader_cancel` fixtureは通常のslow producerを止めるが、stdoutが停止した状態でlock競合を再現する負例ではない。

## 4. 再入場条件

1. GAP-26ではstdout ownershipとprocess controlを分け、readが停止中でもbounded timeでkill／waitできるfixtureを先に作る。
2. stderrを継続drainし、decoder error、cancel、EOF、DropでFD／zombieを残さない。
3. decoder pool／prefetchは最大process数、byte量、generation破棄、pressureとdeadlineを分けた測定後だけ採択する。
4. K4では実VFR timestamp fixture、seek、preview／exportの同じ時刻写像、CFR normalization要否を閉じる。rate差だけを正本にしない。
5. 長尺／1080p／4Kはdecode-only、GPU upload／color、full previewを分離計測し、hardware decodeを同じ発注へ束ねない。

## 5. 復活させないもの

- `ffmpeg-sidecar`を「B-2本命」へ戻すこと。
- Rerunが同crateを使うことからMotoliiの依存、公開API、decode意味を逆算すること。
- heuristic VFR拒否をVFR support、small CFR roundtripを長尺／4K完成と表示すること。
- cancel flagだけ、または同じChild mutexを取るkillだけでblocked read停止を証明したとすること。
- decoder数、queue長、prefetch距離を測定前に恒久化すること。
- ffmpegの暗黙CPU rgba変換をpreviewの別経路として戻すこと。
- hardware decode、native GPU interop、K4 source identity、GAP-26 artifact installを一発注へ束ねること。

## 6. 固定証跡とcoverage

6 blobの完全SHAはreceipt `05i-s2-decode-pipeline.tsv`を正本とする。合計11,586 bytes。

本Unit後のstrict progressは370 / 1,797（20.6%）、未処分1,427である。
