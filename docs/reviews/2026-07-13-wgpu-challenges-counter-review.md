# 反対側レビュー: Rust+wgpu技術的課題調査の二重補正(2026-07-13)

ステータス: **独立批判レビュー**(一次資料を確認できた範囲だけで判定)

対象: 外部LLMによる「Rust+wgpuの主な技術的課題」一般調査(2026-07-13、チャット持ち込み・未ファイル)と、それに対する初回反対側レビュー(同日チャット内)。**初回レビュー自体にも誤りがあった**ため、本文書は二重の補正 — 元調査の古い批判を退けつつ、初回レビューの誤記と、Motolii自身の文書の過大記述も退ける — を記録する。範囲の限定: 元調査は未ファイルのため、本文書は**本文で列挙した主張に限る再判定記録**であり、元調査全文の正本化ではない。同日の追補として、同じ外部LLMによる「失敗露呈タイムライン」調査への判定を§Cに収録する(こちらも判定側の初稿に誤りがあり、再補正済み)。

## 結論

- 「Motoliiの軽量コンポジット路線はwgpuの弱点を避けやすい」という方向性の結論は**維持**できる
- ただしマルチスレッド・bindless・WGSLの3項目の正しい判定は「問題なし」ではなく「**Motoliiの負荷形状では優先度が低い**」である
- 初回レビューが挙げた「抜け5点」のうち、**文書と実装の不一致が確認できたのはGPU→CPUリードバック**(バッファ再利用のみでin-flight重畳は未実装)のみ。VRAM予算・プラグイン資源・色管理は既に設計へ入っており、「抜けている」は過大だった。リードバック重畳の**優先度は原因分離ベンチ(§C-2)未実施のため未確定** — 本文書は採用提案でありP0確定ではない
- 逆に、[memory-model.md](../memory-model.md) P1の「非同期・パイプライン化」記述は実装([motolii-export](../../crates/motolii-export/src/lib.rs))と食い違う過大記述だった(本レビューで訂正済み)
- シェーダー初回コンパイルのヒッチは実在する未解決ギャップ(計測後の優先度比較対象)

## 判定方法

規律6点([README](README.md))に従い、各主張を (1)一次資料で確認できるか (2)同じ失敗条件がMotoliiにあるか (3)より小さい対策はないか、の順で反対尋問した。一次資料は2026-07-13にdocs.rs・gfx-rs公式ブログ・リポジトリ実物で再確認したものに「確認済」を付す。

判定語: **採用** / **縮小** / **延期** / **棄却**(README準拠)

## A. 主張別の判定(事実補正)

| # | 主張(出所) | 判定 | 根拠 |
|---|---|---|---|
| A-1 | arcanizationはwgpu 0.17(初回レビュー) | **誤り・訂正** | 公式記録では**0.19、2024-01リリース**(trunkマージは2023-11-20)。[wgpu公式ブログ](https://gfx-rs.github.io/2023/11/24/arcanization.html)(確認済) |
| A-2 | 60fps→10fps台は2022〜23年の古い事例(初回レビュー) | **誤りに近い・訂正** | 該当報告は**2024年、wgpu 0.20更新後**=arcanization後。負荷は「複数スレッドでリソース生成しながら描画」であり、コマンドエンコード並列化の問題ではない。[wgpu Discussion #5525](https://github.com/gfx-rs/wgpu/discussions/5525) |
| A-3 | v22〜23でグローバルロック大規模削除済み(初回レビュー) | **一次資料で確認できず・撤回** | 0.19でロック保持時間の短縮は確認できるが、ID/Registry完全撤去の追跡Issueは2026年時点でもOpen。[wgpu #5121](https://github.com/gfx-rs/wgpu/issues/5121) |
| A-4 | コマンドエンコードは並列可能 | **確認** | WebGPU自体が独立コマンドバッファの複数スレッド構築を設計に含む。[WebGPU Explainer](https://gpuweb.github.io/gpuweb/explainer/#multithreading) |
| A-5 | Queueは1本だから遅い(元調査) | **縮小** | Deviceがprimary Queueを1つ持つのは事実だが、フレームあたり少数の大パスを流すコンポジタで律速である証拠はなく(未実測)、採用阻害要因へ昇格しない |
| A-6 | Bindless不在(元調査) | **縮小** | wgpu nativeに`TEXTURE_BINDING_ARRAY`等はあるが、非一様index・partially-boundは別featureで対応差とfallbackが要る([FeaturesWGPU](https://docs.rs/wgpu/29.0.4/wgpu/struct.FeaturesWGPU.html))。ただしbindlessが効くのはGPU駆動の大量マテリアル描画で、レイヤー合成の帯域律速とは故障面が異なる |
| A-7 | naga経由でSPIR-V/GLSLへ逃げられる(初回レビュー) | **API上は確認、Motoliiでは現状不可・訂正** | wgpuはfeature有効時に受理する([ShaderSource](https://docs.rs/wgpu/29.0.4/wgpu/enum.ShaderSource.html))が、Motoliiはwgpu 29を既定feature(WGSLのみ)で使用し([Cargo.toml](../../Cargo.toml)、確認済)、公開契約もWGSLに固定している |

マルチスレッド問題の正しい結論: 「古いから無視」ではなく、**Motoliiが既に守っている「レンダ中・ループ内でGPU資源を生成しない」(performance-model原則3)が主要な回避策**である。将来アセットストリーミング(デコーダ先読みの並列アップロード等)を入れると再浮上するため、その時点でA-2の失敗条件を再確認する。

## B. 「抜け5点」の再判定

### B-1. GPU→CPUリードバックのパイプライン化 — **延期**(計測・採択判断待ち。提案優先度: 高)

**文書と実装が不一致**だった(確認済、2026-07-13リポジトリ実物)。優先度の「P0」確定は§C-2未計測のため行わない。

- [`RgbaDownloader`](../../crates/motolii-gpu/src/transfer.rs)はステージングバッファを再利用するが、copyをsubmitした直後に`map_async`を呼び、**同一関数内で完了までpollする同期実装**
- [書き出しループ](../../crates/motolii-export/src/lib.rs)は各フレームで `render → download完了待ち → CPUコピー → ffmpeg書き込み` を直列実行
- `map_async`は呼び出しが即時復帰するだけで、非同期化には**バッファを複数持ちin-flightを重畳させる**必要がある([wgpu Buffer](https://docs.rs/wgpu/29.0.4/wgpu/struct.Buffer.html#method.map_async))
- 一方[memory-model.md](../memory-model.md) P1は「非同期・パイプライン化を実証済み」と読める記述だった。実証済みなのは**バッファ再利用まで**であり、複数フレームのin-flight化ではない → **本レビューでP1の記述を訂正した(2026-07-13)**

必要な作業(候補): 複数本のbounded staging ringで `GPU render N+1 / GPU→staging copy N / CPU encode N-1` を重畳し、in-flight本数は計測と採択判断で決める。dGPU・ユニファイド双方で測る。P1例外(確定出力の非同期コピーアウトによるキャッシュ充填)もこの実装が前提。

既存タスクとの関係(2026-07-13追記): [M1実装ガードG7](../specs/M1-vertical-slice.md)は**bounded channelによるバックプレッシャ**(増強チケット候補・未実装)であり、**GPU stagingリングによるコピー重畳とは別物**。混同しない。本レビューの提案は (1)**GPU stagingリングによる重畳**と (2)**性能SLO**(§C-2)の2点。正式タスク化は未了 — 採択判断はbacklog/仕様への反映時に、判定語併記で行う。

先例調査(2026-07-13): 「同期待ちを避け、完了までのレイテンシを許容する」原則まではUnity `AsyncGPUReadback`・OBS staging surface・Unreal `FRHIGPUTextureReadback`の公式文書が共通に裏付ける。**N本のbounded staging ring自体はMotolii向けの設計推論**であり、先例が共通に公式規定するものではない — [readback先例調査](2026-07-13-readback-pipelining-prior-art.md)§1(調査文書。採択時は併読)。

### B-2. VRAM予算 — **棄却(設計済み)。ただし事実記述の更新を採用**

LRU・RAM/ディスク降格・ハード予算は[memory-model.md](../memory-model.md) P3と[M4 K1](../specs/M4-cache-and-analysis.md)に既にある。「Motoliiから抜けている」は不正確。

ただし「wgpuには現在のVRAM使用量APIが無い」はwgpu 29では言い切れない(確認済、docs.rs 2026-07-13):

- [`Device::generate_allocator_report()`](https://docs.rs/wgpu/29.0.4/wgpu/struct.Device.html)でwgpu管理下の割当量を取得可能(backend依存で`None`)
- [`MemoryBudgetThresholds`](https://docs.rs/wgpu/29.0.4/wgpu/struct.MemoryBudgetThresholds.html)でD3D12と一部Vulkanの予算比率によるOOM/device-loss閾値を設定可能
- ただしMetalを含む全環境で空きVRAM・レジデンシを統一的に取れるものではない

正しい表現: 「**ポータブルで信頼できる空きVRAM APIは無い。allocator reportは診断補助、正本は自前台帳**」 → **P3の記述を更新した(2026-07-13)**。結論(自前予算管理)は不変。

### B-3. プラグイン境界とGPUリソース — **棄却(現在地には当たらない)。v2スパイクは既存バックログどおり**

v1は静的リンクRust traitで[`TextureRef`が`&wgpu::Texture`を直接渡す](../../crates/motolii-plugin/src/lib.rs)(確認済)。「C ABIを越えられず今すぐハンドル間接化必須」は現在地に当たらない。動的C ABIは[backlog V2-1](../backlog.md)で明示的にv2送り。プロセス隔離では整数ハンドルでは足りず、IOSurface / DXGI shared texture / dma-bufと同期primitiveが要る — これは**v2着手時のスパイク対象**であり、M2へ前倒しすべき穴ではない。

### B-4. シェーダー初回コンパイルのヒッチ — **延期**(INF-8ゲート判定待ち。提案優先度: 中)

実在する未解決ギャップ。ホスト所有PipelineCacheは同一実行中の再コンパイルを防ぎ、ホットリロード時のlast-good維持も[dev-experience.md](../dev-experience.md)§4にあるが、以下は未解決:

- wgpu 29のpipeline作成は同期API
- wgpuの永続[`PipelineCache`](https://docs.rs/wgpu/29.0.4/wgpu/struct.PipelineCache.html)は**現状Vulkanのみ**(確認済。Metal/DX12はドライバ内部キャッシュ任せ)
- アプリ独自のPipelineCacheはパイプラインオブジェクトのメモ化であって、cold-startコンパイルの保証ではない
- プラグイン追加後、最初にノードを表示した瞬間のヒッチを測る完了条件が無い

推奨: INF-8へ「cold cacheで代表プラグインN本を初表示した最大停止時間のSLO」「起動時prewarm」「compile中はlast-goodまたはpass-through」の受け入れ条件を追加する(仕様編集は本レビューではしない — ゲート採用時に判定語併記で)。この形はUnreal PSO Precaching(起動時prewarm+未完時は描画スキップ/デフォルト代替)と同型の出荷済みパターン。転移の2部品にも一般解の先例がある(非同期コンパイル=Bevy 0.13がwgpu上での実現可能性を出荷済み、列挙=Fossilize型/Unreal型の2系統)。ただし**Motolii側の接合条件**が残る: パイプライン生成の捕捉面統一(Overlay/Composite系はPipelineCache外で直接生成しており、現行キーも完全なpipeline記述ではない)と、非同期コンパイル結果の`&mut PipelineCache`への合流設計。方式選択とあわせてゲート判定の対象 — [readback先例調査](2026-07-13-readback-pipelining-prior-art.md)§3。

### B-5. カラーマネジメント — **棄却(方向は設計済み)。HDR意味論の前倒しはしない**

色変換一元化・`Rgba16Float`の型予約・Quality/FrameDescのキャッシュキー算入は[M4仕様](../specs/M4-cache-and-analysis.md)に既にある。現在の書き出し・レンダターゲットは実質RGBA8/sRGB([motolii-export](../../crates/motolii-export/src/lib.rs)、確認済)で、M5のリニアFP16は推奨案の段階。ここでHDR意味論まで恒久化するのは「意味が先」に反する。M4 K1では (1)実フォーマットから占有バイト数を計算 (2)PixelFormat・色空間・premul・Qualityをキーへ含める (3)RGBA8/FP16を決め打ちしない、までを保証し、HDR/OCIOの意味論採択([backlog V2-4](../backlog.md))は独立レビューにする。

## C. 失敗露呈タイムラインと「wgpu採用失敗」の判定基準(2026-07-13追記)

同日の追加調査「技術の失敗が露呈する瞬間」への判定。大原則: **「wgpu採用の失敗」と「Motolii実装・スケジューリングの未完成」を分離する**。挙げられた露呈点の大半は後者であり、wgpuそのものの失敗ではない。

### C-1. 露呈タイミングの正確な並び

元調査のタイムラインはMotoliiの実マイルストーン定義([pitfalls-and-roadmap.md Part 2](../pitfalls-and-roadmap.md))とズレている(M1書き出しは実装済み・通過済み、マルチOSはv1スコープ外=V2-6)。正しくは:

| 時期 | 露呈し得るもの | 既存ガード |
|---|---|---|
| 現在〜M3 | 数分尺書き出しの実測、cold shader compile | 前者は未計測(§C-2)、後者はB-4(INF-8完了条件の提案) |
| M4 | VRAM予算・キャッシュ降格・長時間平衡 | memory-model P3退避はしご、M4 K1平衡/ストレステスト(設計済み) |
| M5 | fp16・3D・多パスによる帯域増加 | performance-model試算はエフェクト帯域を注記済み(下限として読む) |
| v2 | Windows dGPU、HWデコード(V2-3)、HWエンコード(候補・未台帳)、外部共有テクスチャ(B-3) | P2でdGPUを設計基準に据え済み。実測はWindows対応時 |
| 1.0後 | 多様なドライバ・長時間安定性 | 一般論。v2以降 |

### C-2. 書き出し性能の審判(SLOとベンチ設計)

「ffmpegエンコード単体+N%以内」を主SLOにするのは**誤り**(本レビュー初稿の提案を自己訂正): CPUエンコードが遅いほどリードバック問題が隠れ、審判にならない。

- **ユーザー向け主SLO**: 「3〜5分の代表MV(1080p)を実時間の何倍で書き出せるか」
- **原因分離ベンチ(副指標)**: 次を分離計測する — (1)GPUレンダのみ (2)GPU→CPUリードバックのみ (3)decode→encodeパススルー (4)全経路。sinkは**高速・無圧縮相当**と**通常配布codec**の両方で取る(エンコード律速でリードバックが隠れるのを防ぐ)
- `ffmpeg単体比`は原因分析用の副指標に格下げ

### C-3. 早期警告サインの補正

元調査の「書き出し時のCPU高使用率」は警告ではない — ソフトウェアエンコードなら正常。警告にすべきは**リソースが遊んでいるのに進まない**状態:

- GPU・CPU・encoderのいずれも遊んでいるのにフレームが進まない
- 各フレームでpoll待ちが支配的
- in-flight数が常時0(重畳が効いていない)または上限張り付き(バックプレッシャ律速)

また、allocator reportに自動の「警告」は無い — Motolii側で閾値を設けた場合だけ警告になる(B-2の診断補助の枠内)。

### C-4. 「wgpu採用が技術的失敗」と判定できる条件

遅い書き出しだけではwgpu採用失敗にならない。GPU→CPU境界はMetal/Vulkan/DX12を直接使っても存在する。まず疑うのは直列待ち・ステージング設計・ffmpegバックプレッシャ(=Motolii側)。判定手順:

1. 同期待ち・ループ内資源生成などMotolii側の実装欠陥を除去
2. bounded ring・prewarm・VRAM予算管理を実装
3. 代表的な3〜5分MVで定量SLO(§C-2)を測定
4. (§C-2の分離ベンチと直列待ち除去後も原因がwgpu抽象側に残る場合の最終手段)同一GPU・同一処理の最小Metal/Vulkan/DX12試作と比較
5. 直接APIではSLOを満たすのに、**wgpu固有**の抽象・ロック・変換・interop制約で一貫して満たせないことを確認

手順1〜3(実装欠陥除去・ring/prewarm/予算・§C-2計測)で足りないときだけ4〜5へ進む。ネイティブ比較を常設の入場条件にしない(費用が本体開発に匹敵し得、失敗判定を永久先送りする装置になるため)。なお「一部CPUフォールバック」はVRAM常駐原則(memory-model)を壊し、多くの場合さらに遅くなるため、**早期リカバリー策にならない**(誤診による最悪の対処)。

### C-5. Alight Motion参照の扱い

AM内部実装に関する言説(事前計算多用・モバイルGPU最適化等)は出典が無く、規律3により仮説に留める。実機で観測できるのは**プロジェクト制限・Draft時の画質変化・初回エフェクト追加時の停止・export倍率・メモリ逼迫時の挙動**まで — 内部アーキテクチャはそこからの推測と明記する。AM挙動が設計論点になったら実機確認を一次資料とする。

## 優先順位(未完了アクションのみ、2026-07-13再改訂)

実施済み(履歴): memory-model P1過大記述の訂正、P3のVRAM API記述更新(allocator reportは診断補助)。

1. **提案・高(採択判断は計測後)**:
   - 原因分離ベンチマーク(§C-2の4分割+2 sink) — 優先度確定の前提
   - 3〜5分代表MVのユーザー向けexport SLO(実時間の何倍か)
   - GPU stagingリング(重畳)の正式採択判断 — G7(channel)とは別物と明記した上でbacklog/仕様へ(判定語併記)
   - SLO未達かつ分離ベンチでwgpu抽象が疑わしいときだけ§C-4手順4〜5
2. **提案・中**: cold shader compilationのSLOとprewarm/fallback完了条件をINF-8へ(B-4、ゲート判定待ち)
3. **低**: bindlessは要求が発生するまで不採用。採用時はfeature matrixとfallback必須(A-6)
4. **v2**: 動的C ABI・プロセス外共有テクスチャ境界の専用スパイク(B-3、backlog V2-1の枠内)

この形なら「遅かったから即CPUフォールバック」という誤診を防ぎつつ、計測後に優先度を確定できる。v1の実測は開発主機(Apple M4/ユニファイド)のみで行い、dGPU実測はWindows対応時の再計測項目として予約する。

## 併読拘束

元調査・初回反対側レビューは未ファイルのため、**本文書単体を仕様・ゲートの設計根拠にしない**(規律1・6)。ゲート採用時は判定語併記に加え、併読可能な一次資料またはファイル化された元調査が揃うまで外向き化しない。

## 教訓(規律への追記事項ではなく確認)

初回反対側レビュー自体がバージョン・年代の誤り(A-1〜A-3)を含んでいた。「反対側レビューも一次資料で再確認するまで設計根拠にしない」は規律6点の運用注(出典は再確認可能な公開恒久文書に限定)がそのまま適用される — 反対側レビューだからといって免除されない。
