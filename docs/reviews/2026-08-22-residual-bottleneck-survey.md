# 残コスト構造調査(裁定175 保留14行向け・軽レーン)

日付: 2026-08-22 / 状態: **調査**(read-only・cargo 不使用) / 起点: 裁定175「Fast Previews(844-846)+Memory/Disk Cache prefs(11行)は保留 — ゼロコピー下で何が低解像度・キャッシュを要求するのか軽く調べる」

対象コード: `next/engine/motolii-compositor`・`next/engine/motolii-engine`・`next/engine/motolii-media`・`next/shell/motolii-shell`(M4 = 裁定171 v2)。upstream/fork 側は既存 KNOWN・[構造解決台帳](2026-08-22-structural-solutions-ledger.md)の当日付き一次資料を再利用(軽レーンなので再検索していない — 差分なしと判断)。

## 0. 結論(先出し)

保留14行は**性質が違う2群**で、判定も割れる。

| 群 | 行数 | 判定 |
|---|---|---|
| Fast Previews > Draft/Fast Draft/Off(844-846) | 3 | **(b) 実測待ち**。理論的な条件(多層・高解像度素材の同時プレビュー)は r1 probe で実証済みだが、typical project でその条件に当たるかは未測定。かつ**現行の解像度 cap 配線がその条件下でも効かない**(§1-4) |
| Memory & Disk Cache 系(1147/1148/1159/1183/1194/1195/1197/1198/1199/1207/1236) | 11 | **(a) ありそう**。ゼロコピー(GPU 合成)とは無関係な**素材デコード側**に、具体的で日常的に踏む重コストが実在する(§1-2) |

## 1. 現アーキテクチャの残コスト構造(コード実測)

### 1-1. GPU 合成そのもの — 律速は fragment 数でも MSAA でもなく素材帯域(既存実測の追認)

`next/probes/r1-frame-throughput/tests/r1.rs`(既存 probe、`#[ignore]`・単独実行専用)が M4 以前から実測済み・`next/DECISIONS.md` #21 に記録済み:

- 1080p・40層(等倍)= 約40ms(30fps 予算 33.3ms を超過)。MSAA off でも 9% しか変わらない
- comp 解像度だけ 1/2 にしても 16〜36ms とブレる(効果不安定)
- **素材(source texture)を 1/4 に縮める(proxy)と 5.9ms** — 60fps 予算に確実に収まる
- 結論(probe 自身のコメント): 「律速は raster の面積ではなく素材のバンド幅。40枚の 1080p 素材(332MB)を毎フレーム舐める」

→ 支配的コストは **layer 数 × 素材解像度**(帯域)であって、comp 出力解像度でも MSAA でもない。これは M4 のゼロコピー化(readback 撤去)とは独立の律速要因 — ゼロコピーは「読み戻しを消す」だけで、GPU が毎フレーム触る texel 数は変えない。

### 1-2. 素材デコード — ffmpeg サイドカーの**プロセス起動はキャッシュmiss毎**(re_video ではない)

`next/engine/motolii-media/src/lib.rs` のモジュール doc が明記する通り、Motolii の動画層デコードは **re_video を使わず ffmpeg サイドカー**(裁定24、4点の契約不一致が理由)。

`next/engine/motolii-engine/src/lib.rs:602` (`texture_for` の `LayerSource::Media` 分岐):

```rust
let cpu = read_frame_at(path, &info, frame)?;
```

`read_frame_at`(`next/engine/motolii-media/src/decode.rs:232`)の実体:

```rust
pub fn read_frame_at(path, info, frame_index) -> Result<CpuFrame> {
    let mut reader = FrameReader::open(path, info, frame_index)?; // ffmpeg 新規 spawn + -ss seek
    reader.next_frame()?.ok_or(...)
}
```

`FrameReader::open` は **`Command::new("ffmpeg")` を毎回起動**し、`-ss` でシークしてから1フレームだけ読む(順方向連続読みに使える `FrameReader` 型自体は存在するが、`texture_for` のホットパスは1フレームごとに使い捨てる `read_frame_at` を呼ぶ)。

キャッシュは `Engine::frames`(`(path, frame)` → GPU texture の LRU、`FRAME_CACHE_LIMIT = 64`、`lib.rs:517`)。**エンジン1個・全レイヤー共有の64枚**であり、ディスクにも永続しない(プロセス再起動で消える)。64枚を超える範囲を scrub する、または動画層を複数同時に置くと、cache miss = **新規 ffmpeg プロセス起動 + seek + 1フレームデコード**が毎回走る。

→ これは GPU ゼロコピーと**完全に独立したコスト**(CPU 側・プロセス境界越え)。M4 が読み戻しを消してもここは一切変わらない。

### 1-3. Effect pass — 現状は実害小さいが、ゼロコピー経路が**新しい非再利用コスト**を持ち込んでいる

`translate_effect_passes`(`motolii-engine/src/lib.rs:711`)は `motolii.glow` だけを実 pass に変換し、他は無音 skip。[構造解決台帳](2026-08-22-structural-solutions-ledger.md) 節2-4 の通り「effect の複数 pass は連鎖しない(最後勝ち)、現在単一 pass のみなので実害なし」。**したがって現時点で effect pass 数がボトルネックになる実例はまだ存在しない**。

ただし M4 の `Compositor::render_to_texture`(`motolii-compositor/src/lib.rs:991`)には zero-copy 経路固有の設計上の代償がコードコメントで明記されている:

- `effect_scratch.acquire(...)` は呼ぶ(`lib.rs:1044`)が **`.release()` を一度も呼ばない**(readback 経路の `render_with_effects` は `device.poll` で GPU 読了確認後に release してプールへ戻す — zero-copy 経路は poll をしないため安全に戻せず、doc が明示的にそう書いている)
- 結果: **glow などの effect を持つ layer が動くフレームは毎回 scratch texture を新規確保**(次フレームでまた新規生成、プールに戻らない)

→ 「effect pass 数」自体は今のところ小さいが、**ゼロコピー化それ自体が「effect 付き layer の毎フレーム新規テクスチャ確保」という新しいコストを導入した**——複数 effect が連鎖するようになった時(構造解決台帳 節2-4 が「vism 第2号以降で直列化が要る」と予告)にここが効いてくる。

### 1-4. 解像度 cap(½/¼)— **今の配線はゼロコピーの高速路を無効化する側**

裁定163 の `resolution_cap`(Auto/½/¼、normal-map 1450「Fast Previews > Adaptive Resolution」の実装先とされている)を実コードで追うと:

- `motolii-shell/src/lib.rs:2255-2258`: M4 の GPU 高速路(`render_to_texture` 経由、readback ゼロ)が通る条件に **`resolution_cap == Auto` が必須**。½/¼ を選ぶと**自動的にこの分岐から外れ**、下の「フル再計算」(`engine.render_frame` = CPU readback を伴う旧経路)へフォールスルーする(コード中コメント: 「`resolution_cap != Auto`: ½/¼ は CPU 側の縮小に依存しており、GPU 側の縮小はまだ実装していない」)
- しかもその CPU 経路ですら、`resolution_cap` は **compositor の実描画解像度を一切変えない**。`engine.render_frame` は常に comp 解像度でフルレンダーし、その結果(`full_rgba`)を `build_stage_presenter_rgba` → `stage_presenter_rgba`(`lib.rs:2525`)が**レンダー後に CPU で縮小**しているだけ(`motolii-shell/src/lib.rs:2298-2317` 周辺)

→ 現状、½/¼ cap を選ぶことは「表示用コピーを縮める」効果しかなく、**GPU 側の実描画コスト(§1-1 の律速)を一切減らさない**上に、**M4 のゼロコピー高速路を自ら手放して readback 経路に戻る**。つまり「速くするための cap」が、今の配線では Auto(等倍・ゼロコピー)より遅くなり得る。normal-map 1450 を「採用済」とした2026-08-22 の [MA 精査](2026-08-22-normal-map-audit.md) 整合修正は、この配線の実態(post-render CPU downsample・zero-copy 無効化)までは検証していない。

## 2. rerun/re_renderer 側の知見(既存一次資料の追認・軽レーン)

- **fork(`oshikaidesu/rerun`, pin `483b855`)は upstream にほぼ素**([rerun fork seam 台帳](2026-08-18-rerun-fork-seam-ledger.md)): 追加2ファイル(embed 用 `SpatialStage`/`stage_camera.rs`)+小さな改変のみ。**Motolii 側が LOD/frame budget 機構を fork に足した形跡は無い**
- **re_renderer に mipmap 自動生成は無い**(TODO のまま、KNOWN 2026-08-20確認・構造解決台帳 節3 で再追認)。GitHub issue 番号は WebSearch で特定できず EVIDENCE_GAP のまま(既存調査で確認済み、今回再検索していない)。**preview 高速化の唯一の買い方は素材側 proxy**(§1-1 の r1 実測と整合)
- **frame budget / scalability(Unreal Niagara 型の品質レベル別間引き)に相当する仕組みは rerun 本体・fork のどちらにも見当たらない**。今回の軽い確認範囲(fork seam 台帳・構造解決台帳・KNOWN)では該当記述なし — 存在しないと断定するには upstream `re_renderer` のレンダーループ自体を読む必要があり、これは「軽く」の範囲外なので確認していない(EVIDENCE_GAP として明記)
- **`re_video::load_mp4_from_reader` は moov 先読みのストリーミング対応**だが、Motolii の動画層はこれを使っていない(§1-2、ffmpeg サイドカー採用は裁定24 のまま有効)。「re_video のストリーミングがフレーム時間を支配する」という当初の問い1の前提は**実装上あたらない** — 支配するのは ffmpeg プロセス起動コストの方

## 3. 判定

### Fast Previews(844-846)= **(b) 実測待ち**

r1 probe は 40×1080p という合成ストレス値での実測であり、typical project(何層の動画/画像が同時に可視か)の分布は未測定。かつ §1-4 の通り、**今の resolution_cap 実装はこの条件下でも効かない**(GPU 描画コストを減らさない・ゼロコピーを無効化する)。「Fast Previews」を意味あるものにするには最低限、(i) 実プロジェクトの層数分布の実測、(ii) resolution_cap を「post-render CPU 縮小」から「render 前の素材側 proxy 生成」へ配線し直すこと、の両方が要る——今 verdict を確定させると、まだ存在しない実装を前提にした判断になる。

probe案(要る場合): `r1-frame-throughput` に **実測 project の典型層数**(現行 timeline サンプル・normal-map の freq 分布ではなく実データ)でのケースを追加し、かつ resolution_cap を `render_to_texture` 前段の素材ダウンスケールとして繋いだ場合の再測定を行う。今回はコード読解のみで numbers を出していないため、cargo 実行が要る(軽レーンの範囲外・次レーンへ)。

### Memory & Disk Cache 系(11行)= **(a) ありそう**

§1-2 が具体的な根拠: 動画層のデコードは **cache miss = ffmpeg プロセス新規起動 + seek**、キャッシュは 64 フレーム・プロセス内・全レイヤー共有で永続化なし。これは GPU ゼロコピーと完全に独立したコストであり、以下の条件で確実に踏む:

- 動画層を複数(64 を超えるユニーク `(path, frame)` の作業集合)同時に扱う timeline
- 同じ区間を繰り返し scrub/再生する(アプリ再起動のたびに再デコード = disk cache の欠如がそのまま効く)

「RAM/Disk cache」prefs(AE の Media & Disk Cache 系そのもの)は、この decode コストを吸収する仕組みとして技術的に意味を持つ——ただし「ゼロコピー下で何が低解像度を要求するのか」という当初の問い方は的が外れており、正しい問いは「**デコード済みフレームの再利用範囲をどこまで永続化するか**」。ここは probe 不要で、コード実測だけで判定できる。

## 4. 副次的な発見(このレーンの scope 外・別レーン向けに記録のみ)

- §1-4 の resolution_cap 配線(½/¼ が M4 zero-copy 経路を無効化し、かつ実描画コストを一切削減しない)は、normal-map 1450「採用済」の実態と裁定175 双方に影響する可能性がある小さな実装ギャップ。修正はこのレーンの ALLOWLIST 外(read-only)なので実施していない
