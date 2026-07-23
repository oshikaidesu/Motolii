# wgpu readback／cold compile lineageの価値回収（Unit 5C、2026-07-23）

状態: **延期判断維持／現行gap分離**（cutoff 4 historical blobの処分完了）

対象: [wgpu課題反対側レビュー](2026-07-13-wgpu-challenges-counter-review.md)全3版と[readback／cold compile先例](2026-07-13-readback-pipelining-prior-art.md)全1版。

関連: [memory model](../memory-model.md)、[M1仕様](../specs/M1-vertical-slice.md)、[M4仕様](../specs/M4-cache-and-analysis.md)、[backlog](../backlog.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

初版はGPU readback ringをP0採用候補、cold shader compileをP1昇格候補としたが、第2版がどちらも**計測／gate判断まで延期**へ訂正した。第3版はUnity／OBS／Unreal／Bevy／Fossilize等の先例を接続しながら、先例が証明する一般原則とMotolii固有の接合工事を分離した。この自己訂正を最新版として維持する。

2026-07-23の現行コードでも、exportは1本のstaging bufferを再利用する同期readbackであり、cold product pipeline生成は一部しかHost cacheへ集約されていない。二つは同じ「GPU性能問題」にまとめず、readbackの原因分離／採択gateをGAP-29、product cold compile admissionをGAP-30へ置く。

VRAM予算、M4 cache、static first-party plugin境界、色管理は既存設計に席があり、「設計から抜けている」へ戻さない。遅いという観察だけでwgpu不採用、native API再実装、CPU合成fallbackへ飛ばない。

## 2. 全4版の差分

| 版 | 主要差分 | 処分 |
|---|---|---|
| 反対側レビュー1 | readbackをP0採用候補、cold compileをP1候補、ring 2〜4本を例示 | 問題発見は維持。優先度／本数の先決めは後続版で撤回 |
| 反対側レビュー2 | readback／cold compileを延期へ縮小、ring本数を計測事項化、native API比較を最終手段化、併読拘束追加 | **現行判断として維持** |
| 反対側レビュー3 | readback／cold compile先例とMotolii接合条件を追記 | 一般解の存在と接合未完を分離して維持 |
| 先例1 | async readback、GPU texture直encode、async compile、pipeline列挙の出荷先例 | 仮説整合事例。方式採択、ring本数、Metal挙動の証明には使わない |

第1→2版は単なる文言修正ではなく、実測なしのP0/P1断定を撤回した意味変更である。第3版はその延期を解除していない。

## 3. Readbackの現行事実

- `RgbaDownloader`は必要byte数が同じ間、`MAP_READ` bufferを1本だけ再利用する。
- 各`download`はtexture→buffer copyをsubmitし、直後に`map_async`を要求し、同じcall内の`wait_for_map`で完了までpollする。
- exportのoverlay経路とDocument経路はいずれも、frameごとにrender→download完了待ち→CPU `Vec` copy→encoder writeを直列実行する。
- M1 G7の現行同期1-frame経路は滞留数を実質boundedにするが、GPU copy N／render N+1／encode N-1の重畳を実装しない。
- M4 K1cとK7aは確定出力の非同期copy-outを要求するが、implementation ledgerではWAITである。現在のexport downloaderをcache充填基盤として完成扱いしない。

したがって「buffer再利用」「`map_async`というAPI名」「G7 green」「K1c仕様あり」のどれも、非同期copy-out成立の証拠ではない。

## 4. GAP-29: 計測から採択する

最初の成果はring実装ではなく原因分離fixtureである。

1. 代表3〜5分MVの1080p export倍率をユーザー向け指標として測る。具体SLO値は測定と製品判断前に発明しない。
2. GPU renderのみ、GPU→CPU readbackのみ、decode→encode passthrough、full pathを分ける。
3. 高速sinkと通常配布codecの両方を使い、遅いsoftware encoderがreadback stallを隠す場合を検出する。
4. map待ち、GPU、CPU、encoder、in-flight数、backpressureを観測し、export copyとM4 cache copy-outのQueue／帯域競合は別負荷として測る。
5. 重畳を採択した後だけ有限in-flight数、順序、cancel、timeout、map failure、encoder failure時の全buffer回収を契約する。

Unity／OBS／Unrealの一次資料が共通に支えるのは「同期を避け、完了レイテンシを許容する」までである。N本ringはMotolii向け推論で、本数を2〜4へ固定する根拠ではない。Metal unified memoryもCPU／GPUが同じsystem memoryを使うことを示すだけで、copyと同期消滅を証明しない。

HW encoderへのGPU texture直渡しは上位候補だが、software encode経路が残る限りreadbackの代替ではない。V2 payload／OS interop／encoder選択と同時に先行実装しない。

## 5. Cold pipeline compileの現行事実

- Host `PipelineCache`は`PipelineCacheKey { id: &'static str, wgsl: &'static str }`で、`fullscreen_uniform16`と`tex_sample_uniform4`の2定型を同期get-or-createする。
- keyは完全pipeline descriptorでなく、layout、target、blend、entry point等を再生できる永続CreateInfoではない。
- nodesのOverlay、Composite、Mask、AffinePlace系とYUV変換等にcache外の直接`create_render_pipeline`が残る。歴史版の「直接2箇所」という個数は古く、核心は**捕捉面未統一**である。
- `RenderSession::new`はcore node pipelineを同期生成する。製品`RenderWorker::spawn`はworkerの実行closureを作る前、caller側でこれを呼ぶため、worker分離だけでは初期compile停止を消さない。
- plugin cache missはrender worker内でも初回結果を遅らせる。product全pipelineのprewarm、非同期compile結果合流、last-good／pending、cold SLOは存在しない。
- INF-8は作者向けdev WGSL hot reload／restart計測で、product cold cache初表示の完成条件ではない。

## 6. GAP-30: product cold admissionの再入場条件

1. product pipelineを全数inventoryし、生成owner、caller／worker thread、layout、target、blend、entry point、使用surface、初回要求地点を記録する。
2. cold／warmの起動、代表Document初表示、代表first-party plugin初表示を測り、UI／caller停止のSLOを採択する。
3. 捕捉面統一とdescriptor closureはHost内部契約として先に決める。現行`id+wgsl`を永続replay、Document、公開`NodeDesc`、third-party plugin契約へ昇格しない。
4. prewarm／別thread compileを採る場合は、結果の世代、重複要求、失敗、device loss、shutdownとの合流を型付きにする。
5. dev reload失敗時はlast-known-goodを維持する。cold product previewのpending／代替は明示表示し、Final／exportは代替pixelを成果物にせず正規compile完了またはtyped failureを待つ。
6. warm／cold、cache hit／missで最終pixelを変えず、steady frameでpipeline生成0を審判する。

Bevyはwgpu上の非同期compile可能性、Unrealはprecache／代替、Fossilizeはdescriptor記録という部品の先例である。Motoliiの捕捉面、完全descriptor、結果合流を無改修で与えない。

## 7. 維持する棄却／延期

- マルチスレッド、bindless、WGSLを「問題なし」とは言わず、現行の負荷形状では低優先度とする。将来streaming／大量materialが同じ失敗条件を作った時に再測定する。
- VRAM budgetは設計済みだがK1a〜c未実装。allocator reportは診断補助、自前hard capが正本という決定を維持する。
- static Rust pluginの現状へC ABIハンドル問題を前倒ししない。外部runtime／共有textureはVism／v2のpayload別判断へ残す。
- HDR／bindless／HW encode／native API比較をreadback／compile対策へ束ねない。
- AM内部実装の無出典推測を根拠にしない。
- 遅いだけでwgpu採用失敗とせず、Motolii側の同期、資源生成、backpressure、計測不足を先に処分する。

## 8. 復活させないもの

- 固定2〜4本ringや特定in-flight数を計測前の公開契約にすること。
- M1 G7のbounded channel／同期1-frame経路をstaging overlap完成と呼ぶこと。
- K1c／K7aの仕様記述をcache copy-out実装済み証拠にすること。
- HW direct encodeをsoftware encode readbackの全面代替とすること。
- Metal shared memoryからcopy／sync cost 0を推論すること。
- 歴史の「直接pipeline生成2箇所」という個数を現行inventoryへ戻すこと。
- `PipelineCacheKey{id,wgsl}`を完全descriptor／永続PSO replay契約へ昇格すること。
- compile中のpass-through／default pixelをFinal成果物へ混ぜ、cache warm／coldで作品を変えること。
- UI thread同期readback、CPU合成fallback、native API比較を通常の入場条件にすること。

## 9. 固定歴史出典とcoverage

反対側レビュー初版`d2f087d0`を全文で読み、後続2版の全差分と最終版`18712fce`を確認した。先例単一版`998a4e95`も全文で読んだ。処分した4 unique blob（66,680 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/05c-wgpu-readback-cold-compile.tsv`を正本とする。cutoff総数1,797のうち処分済みは352、未処分は1,445である。
