# motolii-media は全部スクラッチだった — 委譲先の測定

- 日付: 2026-08-30
- 裁定(利用者): **`motolii-media` は捨てる。管理責務を持ってはいけない。**
- 位置づけ: 委譲先の**測定**。どこまで捨てられるかを行数と出所で示す

## 測定

`motolii-media` は 2673 行、うち 6 ファイルが ffmpeg を `Command::new` で直接叩いている。

| ファイル | 行数 | 委譲先 | 出所 |
|---|---|---|---|
| `decode.rs` | 281 | **`re_renderer::video::Video::frame_at` → `GpuTexture2D`** | Rerunフォーク |
| `probe.rs` | 648 | `re_video::demux::VideoDataDescription`(寸法・尺・フレーム時刻) | Rerunフォーク |
| `point_cloud.rs` | 207 | `re_importer` | Rerunフォーク |
| `encode.rs` | 222 | `ffmpeg-sidecar` | **既に樹に居る**(re_videoが連れてくる) |
| `mux.rs` | 476 | `ffmpeg-sidecar` | 同上 |
| `waveform.rs` | 523 | `symphonia` | **既に樹に居る** |

**2357行すべてに相手が居り、新しい依存はゼロ。**

## 監督の誤り

最初「`encode`/`mux`/`waveform` は上流に相手が居ないので残す」と報告した。**誤り。**
**Rerun の中だけを探して、Rerun が連れてきている物を見ていなかった。**
`ffmpeg-sidecar` も `symphonia` も既に `Cargo.lock` に居る。

委譲先を探す時は、**直接の上流だけでなくその依存も見る**。

## 憲法の中に在る(重要)

`re_renderer/src/video/` が在り、`lib.rs:40` で `pub mod video`。フォークに手を入れずに届く。

- `Video::frame_at(render_context, stream_id, time, source) -> FrameDecodingOutput`
  — **時刻を渡すと `GpuTexture2D` が返る**
- `Video::begin_frame()` — シェーダのホットリロードで既に回している
  `RenderContext::begin_frame` と同じ拍に乗る
- `re_video::player::VideoPlayer` — フレーム要求・キーフレーム巻き戻し・デコード遅延の扱い。
  **再生機構を自作しなくてよい**

現行の `texture.rs:486` は**フレームごとに ffmpeg プロセスを起動**し、YUV を手で
アップロードし、自前 LRU に入れている(コメント自身が「暫定実装」と認めている)。
`frame_at` はこの3つを一度に置き換える。

対応コーデックも増える: 現行は ffmpeg CLI 1本、`re_video` は av1/h264/h265/vp8/vp9 の
ネイティブ + ffmpeg フォールバック。

## 境目の見つけ方(一般化)

最初に引いた境目「Rerun はビューアなので出力を持たない」は**間違った軸**だった。
正しい問いは「**保守責務を自分が持つか**」で、答えは常に「持たない道を探す」。
ffmpeg を `Command::new` で叩く自前コードは、それが読む側か書く側かに関係なくスクラッチ。

## 復号ストリームの鍵(踏んだ罠)

`re_renderer::video::Video::frame_at` は `VideoPlayerStreamId` を取る。**この粒度を間違えると
どちらの方向でも壊れる。**

| 鍵 | 症状 |
|---|---|
| **path 単位** | 同じ動画を**別の時刻**で使う層が1つのデコーダを共有し、毎フレーム別々の時刻を要求 → シークのたびに前のキーフレームまで戻って復号し直し、**止まる** |
| **層単位**(現行) | 時刻をずらしても止まらない。ただし**ストリーム1本 = デコーダ1本** |

上流の doc が規則を書いている(`re_video/src/player/mod.rs`):

> `time_track_salt` refers to a unique identifier for **a certain way to play through time**.
> For things following the given entity & component at the play head, use `AT_TIME_CURSOR_SALT`.

**鍵は「表示場所」ではなく「時間の辿り方」。**厳密には `(start, source_in, speed)` が同じ層は
共有すべきだが、**`ResolvedLayer` は `source_frame` しか持たない**(時間の写像は engine から
意図的に隠されている、「engine はもう時間の計算をしない」)。層単位はその近似で、
**同じ辿り方の層が無駄にデコーダを持つ**点だけが厳密解と違う。穴を開けてまで厳密化していない。

**H.264 のデコードは ffmpeg CLI 越し**(`re_video` の feature: `## Decode H.264 using ffmpeg over CLI`。
ネイティブは AV1 のみ)。ストリームを増やすほど重くなる構造なので、
重さを疑う時は `pgrep -c ffmpeg` で数える。

## 監督の観測ミス(3度目)

利用者が「止まる・重い」と報告した窓は、**修正が入る12分前に起動したプロセス**だった。
`stat` でバイナリの mtime、`ps -o lstart` で起動時刻を突き合わせて1分で判明。
[窓は切り取る前に全体を撮る](2026-08-30-overnight-plan.md)の同型 —
**症状を聞いたら、まず「それは今のコードか」を確かめる。**

## 委譲が止まる場所 — `probe.rs`(実測後の訂正)

**当初「`probe.rs` は `re_video::demux` へ委譲できる」と書いたが、誤り。**

値そのものは重なる(`coded_dimensions`=width/height、`duration()`、`num_samples()`=nb_frames)。
**しかし覆う範囲が違う。**

`probe()` は先頭 video stream を要求するので **audio-only ファイルで必ず失敗する**(裁定274)。
そのため `probe_container` が別に在り、video の有無を問わず container 内の全 stream を
列挙して `media_frames`/`media_duration` の正本になっている
(`motolii-engine/src/lib.rs:209` の `containers`、回帰テストは
`engine/motolii-engine/tests/media_frames.rs`)。

**`re_video` は動画専用なのでこの役目を代われない。**`probe.rs` は残す。

### 実際に発注して確かめた(差分は破棄)

`MediaInfo` の4フィールドだけ `re_video` から取るレーンを出したところ、
**648行 → 726行に増えた**。`ContainerInfo` を非目標に切ったせいで ffprobe の解析が居座り、
**同じ値の出所が2つになっただけ**で ffprobe の起動回数も行数も減らなかった(Q5違反)。
発注の家の切り方の失敗(`motolii-dispatch` §2.5、同日3度目)。

### 副産物(残す価値のある実測)

`nb_frames / fps` で尺を導出すると **ffprobe の値と一致しない**。
実素材(sample.mp4)で 1440/24 = 60.0s に対し ffprobe は 60.095s。
edit list / タイムスタンプの不整合で、バグではなく素材の実態。
尺が要る時は**サンプルの提示時刻から計算した値**を使う
(`VideoDataDescription::duration()` はそれをやっている)。

## 委譲の最終状態

| ファイル | 行数 | 結末 |
|---|---|---|
| `decode.rs` | 281 | **削除**(`re_renderer::video` へ) |
| `preview.rs` | 130 | **削除**(decode に依存した二世代前の島) |
| `waveform.rs` | 523 | **削除**(未配線。必要なら `symphonia`) |
| `mux.rs` | 476 | **削除**(未配線。必要なら `ffmpeg-sidecar`) |
| `probe.rs` | 648 | **残す** — audio-only を覆うため上流に相手が居ない |
| `encode.rs` | 222 | 残(書き出し。`ffmpeg-sidecar` 候補、未着手) |
| `point_cloud.rs` | 207 | 残(`re_importer` 候補、未着手) |

2673 → 1251行。**削れたのは「上流に在る物」と「繋がっていない物」で、
残ったのは「上流に無い役目を持つ物」。**
