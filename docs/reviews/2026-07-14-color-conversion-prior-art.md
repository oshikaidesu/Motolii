# 色変換(プレビュー/書き出し不一致)の既知解調査メモ(2026-07-14)

ステータス: **調査メモ → 判定済み**(2026-07-14 ユーザー判定: **反対側レビューは免除** — 主要主張が一次資料(ASWF公式ガイドライン・OBS本体リポジトリ)で直接確認できる既知解であり、色タグの埋め込みを含め業界確立事項のため。規律6の判定語は以下)

> 2026-07-23歴史監査: cutoff全1版を[Unit 5E回収](2026-07-23-historical-color-export-lineage-recovery.md)で処分した。GPU RGB→YUVの採択は維持するが、後発のShared Effect lifecycleと衝突した旧`GAP-14`を`GAP-31`へ改番する。現行はdecode方向だけGPU化済みで、exportはRGBA readback＋ffmpeg swscaleのまま。色意味はGAP-31、readback overlap／ring本数はGAP-29、TRC実測はGAP-5へ分離する。

| 節 | 判定 | 補足 |
|---|---|---|
| §1 定番原因チェックリスト | **採用** | GAP-5実測レポートへ照合欄として転記する |
| §2 trcタグの選択 | **延期(実測待ち)** | 事実は既知だが、タグの選択自体はプレビュー一致 vs 入力素材一致の**プロジェクト固有の二者択一**で既知解では決まらない。GAP-5の実測を審判として維持 |
| §3 書き出しRGB→YUVのGPU化 | **採用**(=GAP-31、2026-07-23にID衝突訂正) | ユーザー方針+一次資料先例 |
| §4 sws_flags暫定対策 | **棄却** | §3採用に伴い不要 |

## これは何か

「書き出したら色が違う」(B-3/B-4/GAP-5)に対する**業界の既知解**を調査した記録。あわせてユーザー方針(2026-07-14)「**書き出しの色変換はGPU資産を使うべき**」の先例を調査した。各項目は「穴 → 先例 → 提案文言」の形。

## 現状整理(コード上の事実)

不一致の原因になり得る変換ポイントと現状:

| 変換ポイント | 実装 | 状態 |
|---|---|---|
| ソースYUV→RGBA(デコード) | **GPUシェーダ** `motolii-gpu/src/yuv.wgsl`(係数は`ColorParams::for_color_space`、CPU参照と共有) | B-3対策済。`swscale_reference.rs`で外部正解と照合済 |
| 合成・レンダ | プレビューと書き出しが**同一関数**(`render_export_frame_rgba`相当、`Quality::FINAL`) | B-4対策済。`verify_b4.rs`で数値照合(許容8) |
| RGBA→YUV(再エンコード) | **CPU(ffmpeg/swscale)** `encode.rs:50-52`の`-vf scale=out_color_matrix=bt709:out_range=tv` | **GPU資産の外にある唯一の色変換**(本メモ§3) |
| 色タグ | `-colorspace/-color_primaries/-color_trc bt709 -color_range tv`(`encode.rs:53-61`) | タグ付け自体は対策済。**trcの値**が未決(GAP-5、本メモ§2) |
| 表示 | Slintが`Rgba8Unorm`のバイト列をそのまま表示(=BT.709ガンマのバイトをsRGBとして表示する近似) | GAP-5の「内部sRGB近似」側 |

つまりB-3/B-4の主要対策は完了しており、残る不一致源は**(1) trcタグと内部sRGB近似のズレ(GAP-5)**、**(2) 再エンコードのRGB→YUVだけがswscale(CPU)** の2点に絞れている。

---

## 1. 「書き出したら色が違う」の定番原因チェックリスト

**穴**: 不一致の原因は複数あり、1つ潰しても別の原因で同じ症状が出る。網羅リストで現状を照合しておかないと、GAP-5実測時に原因の切り分けができない。

**先例**: Academy Software Foundation (ASWF) の[Encoding Guidelines / Color Preservation](https://academysoftwarefoundation.github.io/EncodingGuidelines/ColorPreservation.html)と[Canvaのffmpeg色空間記事](https://www.canva.dev/blog/engineering/a-journey-through-colour-space-with-ffmpeg/)が定番原因を体系化している:

| # | 定番原因 | Motoliiの状態 |
|---|---|---|
| 1 | swscaleは色空間未指定だと**BT.601を仮定**する(HD素材で「緑が明るく赤が沈む」) | 対策済: エンコードで`out_color_matrix=bt709`強制、デコードは生YUV読み+自前シェーダ |
| 2 | **タグ無し出力**はプレイヤーごとに解釈が割れる | 対策済: 4タグ明示+`roundtrip.rs`で読み戻し検証 |
| 3 | **limited/full range**の取り違え(白飛び/眠い黒) | 対策済: `out_range=tv`+黒レベルY=16±2のテスト |
| 4 | **transfer(ガンマ)タグ**とコンテンツ実体の不一致 → カラマネするプレイヤーだけ明暗シフト | **未決 = GAP-5**(§2) |
| 5 | swscaleの**低精度高速経路**(`accurate_rnd`/`full_chroma_int`無しのRGB→YUV) | **未対応**(§4。GPU化(§3)なら穴ごと消える) |
| 6 | **クロマ配置(siting)** の不一致 | デコード側は明示(水平left-cosited/垂直mid、swscale照合済)。エンコード側は現状swscale任せ(§3で自前化) |

**提案文言**: GAP-5実測レポートには本表を照合欄として転記し、「どの原因は排除済みか」を明記する。

## 2. GAP-5(内部sRGB近似 vs 出力bt709タグ)の既知解

**穴**: [backlog GAP-5] プレビューはBT.709ガンマのバイトをsRGBとして表示する近似。出力ファイルには`color_trc=bt709`を書く。この既知ズレの実測と許容範囲の線引きが未着手。

**先例**: これは業界で「QuickTime/Resolveガンマシフト問題」として知られる定番のズレそのもの。

- **プレイヤーの解釈が割れるのはまさにtrcタグ**: [ffmpeg-tests(ASWFガイドライン著者)のWeb Color Preservation検証](https://richardssam.github.io/ffmpeg-tests/WebColorPreservation.html)によると、`color_trc=bt709`はChromeでは事実上sRGB扱い、Safari/QuickTime系ではカメラOETF(≈γ1.95)扱いになり、**同じファイルがブラウザ間で明暗シフト**する。sRGBで作業したコンテンツには`-color_trc iec61966-2-1`(sRGBタグ)が最も一貫した表示になると報告されている。ASWFの[H.264推奨コマンド](https://academysoftwarefoundation.github.io/EncodingGuidelines/Encodeh264.html)も、sRGB作業前提のレビュー用途でmatrix/primaries=bt709のまま`-color_trc iec61966-2-1`を採用している(matrixとtrcは独立に選べる)。
- 一方、**カラーマネジメントしないプレイヤー(多数派)はtrcを無視**するため、trcをどちらにしてもズレは出ない。ズレが観測されるのはQuickTime/Safari等のカラマネ系のみ。

**転移条件の注意(Motolii固有の事情)**: sRGBタグ一択と即断できない対抗事実がある。

- パススルー素材(bt709タグ入りの実写)を「ガンマ保持」で通して**bt709タグのまま出す現状は、入力ファイルとの整合では正しい**。trcをsRGBに変えると、カラマネ系プレイヤーで「元ファイル」と「Motolii出力」の見えが変わる方向のズレに置き換わる。
- つまりこれは「プレビュー(sRGB表示)に合わせる」か「入力素材のタグに合わせる」かの**二者択一**であり、どちらを選んでもカラマネ系プレイヤーではもう片方とズレる。これがGAP-5が「許容範囲の線引き」を要求している理由と整合する。

**提案文言(GAP-5着手時の測定マトリクス)**: 結論をスキーマや契約に焼く前に、`trc=bt709` / `trc=iec61966-2-1`の2出力 ×(QuickTime/Safari/Chrome/VLC/mpv)で同一フレームを実測し、判定基準を「**プレビュー(Slint sRGB表示)との一致**」と「**入力素材との一致**」の2軸で別々に記録する(規律5: 別々に評価)。sRGB≒BT.709ガンマの差は中間調で最大になるため、測定パターンにはカラーバーに加えグレイランプを含める。

## 3. 書き出しRGB→YUVをGPU資産で行う(ユーザー方針の先例)

**穴**: 現状、パイプライン中で唯一swscale(CPU)に残っている色変換が再エンコードのRGB→YUV(`encode.rs:52`)。係数・siting・丸めがGPU資産(`ColorParams`+WGSL+CPU参照)の管理外にあり、F-5ガード「色変換をmotolii-gpuの共通関数1箇所に閉じる」の例外になっている。ユーザー方針(2026-07-14): せっかくGPU資産があるので書き出しもそれを使うべき。

**先例(OBS Studio / libobs)**: 本番実績のある同型構造が存在する。

- OBSはエンコーダへ渡す前の**RGB→YUV変換をGPUシェーダで実行**する(`libobs/obs-video.c`の`render_convert_texture()`、I420/NV12等へのpackシェーダ)。[PR #1978](https://github.com/obsproject/obs-studio/pull/1978)でこの変換をスケールシェーダから独立したYUV packシェーダに再編している。
- 変換後のプレーナYUVを**ステージングテクスチャ+ダブルバッファの非同期リードバック**でCPUへ写し、そのままエンコーダに渡す。
- 教訓(逆側): ハードウェアエンコーダに**RGBを直渡しすると内部変換の行列が不透明**で、NVENC系はBT.601相当の変換・タグになるという報告がある([NVENC API自体はRGB入力時に内部変換を行う](https://docs.nvidia.com/video-technologies/video-codec-sdk/13.0/nvenc-video-encoder-api-prog-guide/index.html)。行列不一致の観測は[フォーラム報告](https://forum.videohelp.com/threads/384118-Hardware-encoders(Quicksync-NVENC)-colormatrix-behavior)レベルの弱い出典)。**変換は自前GPUで済ませ、エンコーダには常にYUVを渡す**構造にしておけば、将来hwエンコーダ対応時もこの穴を踏まない。

**転移条件(Motoliiに揃っている資産)**:

1. **係数の単一ソース**: `ColorParams::for_color_space`(kr=0.2126/kb=0.0722)の**逆行列**を同じ場所に足すだけで、デコードとエンコードが同一係数ソースを共有する。F-5ガードの例外が消え、B-4検証(`verify_b4.rs`)が「往路も復路も同じ係数」の閉ループになる。
2. **sitingの明示化**: デコードシェーダのsiting(水平left-cosited、垂直mid = `in_h_chr_pos=0:in_v_chr_pos=128`)を、エンコード側の4:2:0ダウンサンプルがミラーする。現状はswscaleの既定値任せだったものが自前宣言になる。
3. **テスト資産の再利用**: `swscale_reference.rs`(デコードのswscale照合)と対称に、GPUエンコード変換をswscaleを**外部正解**として照合するテストが同じ道具立てで書ける。CPU参照(`cpu_reference/yuv.rs`)にも逆変換を足し、`tol`モジュールの既存許容(max 1 / mean 0.5)で守る。
4. **転送量の削減(副次)**: リードバックがRGBA 4byte/px → yuv420p 1.5byte/pxになり62.5%減。変換もGPU並列になる(性能は主目的ではなく副次効果として記録)。

**提案文言(実装スケッチ、1PR粒度)**:

- `motolii-gpu`に`RgbaToYuv`(RGBA8→yuv420pプレーナ、compute shader)を追加。係数は`ColorParams`の逆変換として同ファイルに定義、sitingはデコードと同一宣言。limited固定(v1)。
- `encode.rs`の入力を`-pix_fmt rgba`+`-vf scale=...`から**`-pix_fmt yuv420p`の生YUV入力**に変更(`-vf scale`の色変換を削除。4色タグは維持)。qp0検証パスはyuv444p(サブサンプル無し)で変換のみを検証できる。
- テスト: (a) GPU逆変換 vs CPU参照逆変換(`tol::GPU_RASTER`)、(b) GPU逆変換 vs swscale外部正解(`swscale_reference.rs`と同形式・同許容根拠)、(c) GPU encode→GPU decodeの往復ゴールデン(qp0)。往復が同一係数になるためB-4許容(現在8)を締められる可能性があるが、締め幅は実測後に決める(規律4: 審判が揃うまで公約しない)。
- lavapipe(CI)で決定的に走ること(INF-3の方針と整合)を入場条件にする。

## 4. 短期の小さい対策(swscale継続の場合のみ)

**穴**: §3を実施するまでの間、`encode.rs`のswscale変換は精度フラグ無しで低精度経路に入る可能性がある。

**先例**: [ASWFのswscale検証](https://academysoftwarefoundation.github.io/EncodingGuidelines/EncodeSwsScale.html)は`-sws_flags spline+accurate_rnd+full_chroma_int`を推奨(`accurate_rnd`/`full_chroma_int`無しのRGB→YUVはクロマが低精度)。

**提案文言**: §3(GPU化)を採用するなら本項は**不要になる**ため着手しない。§3を延期する判定になった場合のみ、`encode.rs`に1行追加+ゴールデン更新の最小PRとする(より小さい対策の原則)。

## 提案の優先順位(2026-07-14判定済み — 冒頭の判定表参照)

| 提案 | 対応する穴 | 粒度 |
|---|---|---|
| §3 GPU RGB→YUVエンコードパス(採用=GAP-31) | F-5ガードの例外解消+ユーザー方針 | 色意味／export接続を順に閉じる。readback overlapはGAP-29 |
| §2 GAP-5測定マトリクス(延期=実測待ち) | 「書き出したら色が違う」の最終境界 | 実測レポート(backlog GAP-5そのもの、P2) |
| §4 sws_flags(棄却) | §3採用に伴い不要 | — |

## 出典

再確認可能な公開恒久文書(運用注):

- [ASWF Encoding Guidelines: Color Preservation](https://academysoftwarefoundation.github.io/EncodingGuidelines/ColorPreservation.html) / [H264 Encoding](https://academysoftwarefoundation.github.io/EncodingGuidelines/Encodeh264.html) / [SwsScale検証](https://academysoftwarefoundation.github.io/EncodingGuidelines/EncodeSwsScale.html)
- [Canva Engineering: A journey through colour space with FFmpeg](https://www.canva.dev/blog/engineering/a-journey-through-colour-space-with-ffmpeg/)
- [obs-studio PR #1978: Rework RGB to YUV conversion](https://github.com/obsproject/obs-studio/pull/1978)(libobsのGPU変換再編)
- [NVENC Video Encoder API Programming Guide](https://docs.nvidia.com/video-technologies/video-codec-sdk/13.0/nvenc-video-encoder-api-prog-guide/index.html)
- [ffmpeg-tests: Web Color Preservation](https://richardssam.github.io/ffmpeg-tests/WebColorPreservation.html)(ASWFガイドライン著者の検証ページ。ブラウザ別trc解釈の観測)
- 弱い出典(観測報告のみ、根拠にしない): [VideoHelp: Hardware encoders colormatrix behavior](https://forum.videohelp.com/threads/384118-Hardware-encoders(Quicksync-NVENC)-colormatrix-behavior)
