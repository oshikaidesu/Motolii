# Rerun 部品目録調査 — 技術層の再発明を洗う(2026-08-29)

発注書: Stage プレビューのガクつき(利用者指摘)を追ったら `device.poll(wgpu::PollType::wait_indefinitely())`
がライブ経路(`presentable.rs`/`sequential.rs`/`point_cloud.rs`)に見つかった。原因は export 用の
同期コードをライブ経路へ誤って流用したこと。さらに調べると `motolii-media`(FFmpeg サイドカー)の
同期デコードと、`EffectScratch`(独自テクスチャプール)も同型の問題を抱えていた。

利用者の整理: 「AE は操作の文法(レイヤー・タイムライン・キーフレーム)の出典であって、
技術層(GPU・非同期・実時間性)の出典ではない。技術層は Rerun(2023年設計)の方が進んでいる」。
これを一行の裁定で終わらせず、**Rerun 側に既にある部品をMotoliiが気づかず再発明していないか**を
体系的に洗う調査に広げた。

## 0. 前提: Motolii は Rerun のフォークである

`app/Cargo.toml` / `Cargo.toml` の `re_renderer`/`re_chunk`/`re_sdk_types` 等は上流
`rerun-io/rerun` ではなく **`https://github.com/oshikaidesu/rerun`(利用者自身のフォーク)** を指す。
比喩ではなく事実。フォークの commit 履歴には既に「外部ホスト(Makepad)が Rerun の描画経路に
直接繋がるための embedder API」を Motolii 自身が Rerun 本体へ書き足してきた実績がある:

```
346a0b3 feat(re_renderer): add ViewBuilder::new_with_external_resolved
7cca401 feat(re_renderer): device-driven second constructor for RenderContext
856f597 feat(renderer): let embedders read a view's resolved main target directly
483b855 feat(spatial): let embedders place the view camera directly
252c9ce feat(spatial): show embedded GPU frames as transparent 3D layers
037579e feat(spatial): expose embedded eye distance for host fit
71a3127 feat(spatial): accept GPU-resident image frames
```

**役割分担**(2026-08-29 利用者裁定、`AGENTS.md`にも記載): 技術の土台は Rerun フォーク側に足す、
Motolii 本体(`app/`)は足りていない編集体系(AE 的なメンタルモデル)だけを出す。
フォークへの patch は**薄いラッパーに限る**(既存ロジックを書き換えず、新しい口を1つ足す形)。
上流 `rev` を上げた時に merge/rebase で潰れない形を保つため。

## 1. Rerun の製品コンセプト

README(`rerun-io/rerun`)の自己紹介: **「物理AIのためのデータレイヤー」**。
ロボットログ・人間データリグ・シミュレーション・Web動画など多レート・マルチモーダルなデータ
(画像・点群・変換・時系列・関節状態・動画)を、列指向ストレージ(Apache Arrow)へログし、
クエリし(dataframe/SQL)、リアルタイムに可視化し(組み込みビューア)、学習へストリームする。

動画編集ソフトとは製品として別物だが、**「多レートで時間同期したデータをリアルタイムに正しく
扱う」という技術的な土台の問題は、編集エンジンの要求と丸ごと重なる**。`re_tf`(ロボットの
関節=変換階層)・`re_quota_channel`(センサーストリームの背圧)・`re_backoff`(ネットワーク越しの
リトライ)のような一見関係なさそうなクレートが揃っているのはこのため。**「動画編集に関係なさそう」
という理由でクレート名を除外すると、今回のような見落としを繰り返す。**

## 2. 実測: ライブ経路に見つかった同期待ち(修正はまだしていない)

| 箇所 | 種別 | 発火条件 |
|---|---|---|
| `app/engine/motolii-compositor/src/presentable.rs:121-124` | `device.poll(wait_indefinitely())` | 常時(Stage描画のたび) |
| `app/engine/motolii-compositor/src/sequential.rs:705` | 同上 | track matte を含む composition |
| `app/engine/motolii-compositor/src/point_cloud.rs:103` | 同上 | point cloud レイヤーを含む composition |
| `app/engine/motolii-media/src/decode.rs:144-192` | 1フレーム1プロセスffmpeg起動+同期read、UIスレッド | 常時(Stage毎フレーム、キャッシュミス時) |
| `app/engine/motolii-media/src/waveform.rs:79` | ffmpeg全読み、UIスレッド | 素材インポート時 |
| `app/engine/motolii-compositor/src/effects/mod.rs:130-200` | `EffectScratch`(独自テクスチャプール、`release()`が手動pollを前提) | エフェクトパスを持つレイヤーを含む時 |

`render_effects.rs`/`sequential.rs`の他5箇所は「通常のplayhead/再生からは呼ばない」と
自己申告コメント付きのバッチ(export/screenshot)専用経路——ライブ経路の対象外と判定済み。

## 3. Rerun 側の候補部品(概念レベルの棚卸し)

`re_renderer`/`re_video` だけを見て「無い」と判断した1回目の調査は誤り(粒度が狭すぎた)。
Rerun は crates ~100個規模のモノレポ。`ARCHITECTURE.md`の一覧を見て候補を広げ直した。

| クレート | コンセプト | Motoliiとの噛み合わせ | 判定 |
|---|---|---|---|
| `re_renderer` | wgpuベースの高レベルレンダラ | フレーム完了は境界式バックプレッシャー(`MAX_NUM_INFLIGHT_QUEUE_SUBMISSIONS=4`)、`GpuTexturePool`も標準装備 | **委託**(presentable.rs等3箇所の直し先) |
| `re_video` | 動画デコード。`AsyncDecoder`(chunk投入型・非ブロッキング)、`VideoPlayer::frame_at`(フレーム単位シーク) | H.264/H.265は内部でもffmpegサイドカー(排除はできない)。コンテナはMP4のみ、ProRes非対応。probe/waveformはカバー外 | **部分委託**(decode.rsの構造的な直し先。probe/waveformは自前継続) |
| `re_tf` | 時間変化するアフィン変換の木構造(transform frame)+キャッシュ(`transform_resolution_cache`) | 名前は似ているが問題設定が違う | **不採用(確定、2026-08-29検証済み)**。3D限定(2D+skew型が無い)、キーフレーム補間を持たない(`latest-at`離散値取得のみ、`lerp`/`slerp`0件)、階層参照が文字列(entity path)ベースで`LayerId`ポインタ参照と構造が違う、`re_chunk_store`/`re_entity_db`依存が全実装に染み渡り単体利用不可。Rerunは「離散ログ値のlatest-at」、Motoliiは「キーフレーム間の連続補間」——問題そのものが違う |
| `re_quota_channel` | バイトサイズで背圧をかけるmpsc/broadcastチャンネル(tokio)。詰まったら警告 | 「生成側スレッドは進み続け、消費側は最新だけ読む、詰まったら分かる」という今回の設計方針とほぼ同じ実装 | **条件付き採用(2026-08-29検証済み)**。`sync`モジュールはtokio非依存(featureゲート無し、常に使える)。decode.rsのworker化には`Sender<CpuFrame>`/`Receiver<CpuFrame>`としてそのまま使え、フレームのバイトサイズで背圧をかけられる。ただしフレームサイズがほぼ均一なら単純な`crossbeam-channel::bounded()`で足りる可能性もあり、過剰の疑いは残る(フレームサイズのばらつき次第) |
| `re_backoff` | 指数バックオフ+jitter生成器 | `motolii-audio/producer.rs`の自前`thread::sleep`バックオフの直接差し替え候補 | **不採用(確定、2026-08-29検証済み)**。producer.rsの`thread::sleep(1ms)`は「一定間隔の供給待ちポーリング」で、`re_backoff`が想定する「失敗のたびに待ち時間を伸ばすリトライ」とは意味論が違う。指数バックオフを当てるとレイテンシが伸びて逆効果。API形は使えなくもないが目的が違うので無意味。副産物: `Cargo.toml:158`に`tokio`が既に宣言されているのに、どのクレートも実際には引いていない(死んだ依存) |
| `re_mp4_reader` | mp4→Rerunの`Chunk`(ログデータ)変換。内部で`re_video`使用 | 出口がRerunの記録データモデル向けで、Motoliiが欲しい「合成用の生テクスチャ」とは形が違う | **見送り**(`re_video`本体で足りる) |

## 4. 機械可読な部品目録(生成スクリプト)

`scripts/gen-rerun-inventory.sh` — `gen-inventory.sh`(Motolii自身の在庫表)と同じ理屈で、
rustdoc JSON を読むだけの機械生成。`cargo metadata`で毎回チェックアウトパスを引き直すので、
**Cargo.toml の `rev` が動いても再実行するだけで追従する**(パスのハードコードをしない)。
今は `re_renderer`/`re_video` の2クレートのみが対象(`CRATES=(...)`配列に追加していく)。
出力先: `next/reference/generated/rerun-parts.tsv`(2026-08-29時点2,166行)。

## 5. Model面でも同じ構図が確認できた — `Document` は既に `re_chunk_store` の上に建っている

利用者の仮説「MotoliiのDocument自体がRerunの`re_chunk_store`であって、Lottieはそこへの
翻訳層に格下げできるのでは」を検証した。**これは未来の移行先ではなく、既に現在の実装だった**:

- `app/core/motolii-store/src/document.rs:14-15, 351` — `Document`は`re_entity_db::EntityDb`
  (`re_chunk_store::ChunkStore`のラッパー)を`pub(crate) db`として直に持つ。Lottie形式の
  自前ストレージは最初から無かった
- Undo/Redo(`document.rs:449-479`)は値を書き戻すのではなく、`EDIT_TIMELINE`という専用の
  時系列上で時刻カーソル(`head`/`tip`/`floor`)を動かすだけ。再編集時は`drop_time_range()`で
  未来側を捨てる(`document.rs:542-550`)。この設計はコメントで**Rerun自身の
  `re_viewer_context/src/undo.rs`(Blueprintストアのundo機構)を直接引用**しており、
  「編集可能ドキュメントをChunkStoreの上に作る」というパターン自体、Rerunが自分の
  Blueprint用に確立済みだったものを正しく踏襲している

**`motolii-store`の14,181行が実際に足しているのは3つ、いずれもRerun側に対応物が無い**:

1. **セグメントごとに異なる補間方式**(`Hold/Linear/Bezier/Bounce/Elastic/Steps`、
   `app/core/motolii-eval/src/track.rs:55`)。Rerun側の`re_sdk_types::components::InterpolationMode`
   (`interpolation_mode.rs:26-45`)は`Linear/StepAfter/StepBefore/StepMid`の4種のみで、
   `SeriesLines`(プロット線描画)専用——アニメーションカーブの補間とは別物。この
   `KeyframeTrack`一式は`TrackJson`という不透明JSON文字列1本にシリアライズされてChunk
   Storeへ渡る(`components.rs:21`)。ストア自体はセグメント構造を一切見ていない
2. **名指した欄だけ書き換える部分更新**(裁定271)。ストアの機能ではなく、
   `document/apply.rs`の`Intent::SetPropertySlot`(333-410行)が「読む→1フィールドだけ
   Rustで書き換える→JSON全体を再シリアライズして1つの新しい値として書き戻す」を
   自前で行っている
3. **Intentの意味論・検証・バッチ原子性**(`apply_all`, `document.rs:502`)

**結論**: Lottieは最初からストレージモデルではなかった——この点で利用者の仮説は
「これから成立させる」ではなく「既に成立していた事実の言語化」だった。技術基盤
(ストレージ・時系列クエリ・undo機構)はRerunに乗り、Motoliiが足しているのは
正真正銘「AE的な編集の意味」だけ、という役割分担が**View(Stage)だけでなくModel
(Document)でも確認できた**。§0〜1の一般原則(技術層はRerun、Motoliiは編集体系のみ)は
一箇所の思いつきではなく、この repo の実装が既に一貫してその形をしている、という
実測的な裏付けが取れたことになる。

## 6. 次にやること

1. ~~`re_tf` を実際に読んで「委託できるか」を確定させる~~ → **完了(2026-08-29、不採用確定)**。§3参照
2. ~~`re_quota_channel`/`re_backoff` を実際に読んで「委託できるか」を確定させる~~ → **完了
   (2026-08-29)**。`re_quota_channel`(sync module)は条件付き採用、`re_backoff`は不採用確定。§3参照
3. §2の6箇所を、確定した委託先(主に`re_renderer`本体・`re_video`・`re_quota_channel`)へ
   繋ぎ直す(未着手・実装フェーズ)。ただしフォークへの patch は§0の「薄いラッパー限定」の制約を守る
4. `scripts/gen-rerun-inventory.sh` の対象クレートに `re_tf`/`re_quota_channel`/`re_backoff` を追加
   (棚卸し済みなので優先度は下がった。実装フェーズで実際に使う物だけ追加すればよい)
5. **完了**: `app/reference/lottie-coverage.tsv` の採用済み項目のうち `layers`/`effects`/
   `effect-values`/`composition`(39行)から、Rerun側への技術委託を横断的に洗う台帳
   [`next/reference/generated/rerun-technical-delegation.tsv`](../../next/reference/generated/rerun-technical-delegation.tsv)
   を作成した。39行中34行は純粋なDocumentデータ項目/AE編集体系そのもの(Rerunに対応概念なし)で
   対象外。技術判断が要った5行(`ks`/`parent`/`bm`/`effect.ty`/`masksProperties`/`ao`/`text-layer.t`、
   実際は7行)は**全て監査完了**——`ks`/`parent`(`re_tf`)・`bm`(blend.rs)・`masksProperties`
   (mask.rs)・`text-layer.t`(motolii-vector)は委託不可確定(自作継続が正当)、`ao`は両者未実装、
   `effect.ty`は部分配線済み。**Motoliiの技術委託棚卸しはこれで一区切り** — 残るは実装フェーズ
   (presentable.rs/sequential.rs/point_cloud.rs/decode.rsを確定した委託先へ繋ぎ直す)と、
   `exponential_smooth_factor`(emath、§7参照)のUIナビゲーション平滑化への適用検討

## 7. 副産物: `emath::exponential_smooth_factor`(KeyframeTrackとは別問題)

利用者から「Rerunのカメラが滑らかに動くのは何故か」という指摘を受けて`re_view_spatial/src/eye.rs`
を調査した。結論: **KeyframeTrackの代替にはならない**(問題設定が違う)が、**別の用途に転用価値のある
小さな部品**が見つかった。

- `EyeInterpolation`(`eye.rs:158`): 開始姿勢→終了姿勢への一回限りの遷移。`ease_out`
  (`1-(1-t)²`、固定1種類)+`lerp`/`slerp`。KeyframeTrackが要求する「任意個数のキーフレーム・
  セグメントごとに補間方式(Bezier/Bounce/Elastic/Steps)を切替」とは別物 — **不採用**
- `egui::emath::exponential_smooth_factor`(MIT OR Apache-2.0、ライセンス上安全):
  ```rust
  pub fn exponential_smooth_factor(reach_this_fraction: f32, in_this_many_seconds: f32, dt: f32) -> f32 {
      1.0 - (1.0 - reach_this_fraction).powf(dt / in_this_many_seconds)
  }
  // let t = exponential_smooth_factor(0.90, 0.2, dt); value = lerp(value..target, t);
  ```
  リアルタイム入力(WASD/ゲームパッド移動)への**状態追従型平滑化**(exponential smoothing、
  キーフレームという概念を持たない)。velato precedentと同じ「コード査読→式だけ移植」の型で、
  Stage/Timelineのカメラパン・ズーム・スクロール momentum に使える可能性がある(**未適用、
  今の実装が何を使っているか確認してから当てはめる価値があるか判断する。次回に持ち越し**)
