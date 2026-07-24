# 先例調査: GPU→CPUリードバック重畳とcold shader compileの解決例(2026-07-13)

ステータス: **調査文書**(規律3: 仮説と整合する事例の収集。反例未探索。設計採用時は反対側レビュー併読=規律6)

> 2026-07-23歴史監査: 本版を[Unit 5C回収](2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md)で処分した。先例が支えるのは「同期を避け、完了までのレイテンシを許容する」と一般解の存在までである。bounded ring本数、HW direct encode、prewarm列挙、非同期結果合流、Metal staging実体はMotoliiで未裁定のままGAP-29／30へ渡す。

対象仮説: 「[wgpu課題反対側レビュー](2026-07-13-wgpu-challenges-counter-review.md)のB-1(リードバック重畳)とB-4(cold shader compile)は既知の問題であり、先行事例による解決例に溢れている」(ユーザー提起、2026-07-13)

## 結論(仮説と整合)

両方とも出荷ソフトが同じ形で解いている定番問題だった。リードバックは「**数フレームのレイテンシと引き換えにストールを消す**」が業界共通の設計原則で、Motoliiの**オフライン書き出し**はレイテンシ許容経路なので理想的な適用先。キャッシュ充填(memory-model P1例外)もレイテンシ許容だが、**対話プレビューとGPU帯域・Queueを競合し得るため書き出しとは分けて評価する**。さらに「読み戻し自体を消す」上位解(GPUテクスチャ直エンコード)はOBSの導入コミットで**一次確認済み**の出荷パターンであり、V2-3の対になる未台帳のHWエンコード候補に接続する。cold shader compileも非同期化(Bevy)と列挙(Fossilize/Unreal)に出荷済みの一般解が実在する(§3)。総括: **一般解の存在は全件確認できた。ただしMotoliiへの接合条件 — パイプライン生成の捕捉面の統一、非同期コンパイル結果の合流設計(いずれも§3)、Metalステージングの実体(実測待ち)— は未確認**であり、方式選択とあわせて採択判断の対象。先例の部品と接合工事を混同しない。

## 1. リードバック重畳の先例

| 先例 | 方式 | 出典 |
|---|---|---|
| **Unity `AsyncGPUReadback`** | 「GPU/CPUどちらもストールさせずにGPU→CPUコピーする。**代わりに数フレームのレイテンシが加わる**」と公式APIが明記。リクエストはフレームごとに自動進行し、完了後1フレームだけ結果へアクセス可能 | [Unity公式Scripting API](https://docs.unity3d.com/ScriptReference/Rendering.AsyncGPUReadback.html)(公式) |
| **OBS Studio staging surface** | `gs_stage_texture`公式ドキュメントが「**ストールを防ぐため、処理に1フレーム与えるのが理想**」と明記 — copyとmapを同一フレームでやらない、という本件と同型の指針 | [OBS公式graphics APIリファレンス](https://docs.obsproject.com/reference-libobs-graphics-graphics)(公式) |
| **Unreal `FRHIGPUTextureReadback`** | フェンスベースの非同期readback API(`IsReady`ポーリング) | [UE公式APIリファレンス](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/RHI/FRHIGPUTextureReadback)(公式) |
| **wgpu自体** | `map_async`は即時復帰でありポーリングで進行、が公式契約 — 「複数バッファでin-flightを重ねる」のはこのAPI形状が想定する使い方 | [wgpu Buffer 29.0.4](https://docs.rs/wgpu/29.0.4/wgpu/struct.Buffer.html#method.map_async)(公式) |

二次資料注記: Unrealの同期版(`ReadPixels`)がRHIコマンドキューをflushしゲームスレッドをmsオーダーでブロックするという定番知識は[フェンス解説記事](https://nicholas477.github.io/blog/2023/reading-rt/)(二次)によるもので、一次資料未確認。

**一次資料が共通に裏付ける原則**: 同期待ちを避け、完了はコールバック/フェンスのポーリングで検知し、完了までのレイテンシを許容する(UnityとOBSはこのトレードオフを公式文書に明記)。

**Motolii向けの設計推論(先例の公式規定ではない)**: 「ステージングN本のbounded ringでcopy発行と消費を別フレームにずらし、in-flight数を有限にする」は、上記原則をMotoliiのpull型書き出しループへ落とすための**妥当な実装候補**であって、三者が共通にリング構造を公式規定しているわけではない。反対側レビューB-1の提案はこの区分で読む。

**Motoliiへの転移条件**: 書き出しは(ゲームと違い)フレーム落ちの概念が無くレイテンシ完全許容なので、先例より条件が緩い。ただし先例はゲームエンジン(毎フレーム進行が保証される)であり、Motoliiの書き出しはpull型ループなのでポーリング駆動を自前で書く点だけ異なる。

## 2. 上位解: 読み戻し自体を消す(HWエンコーダへのテクスチャ直渡し)

OBSはハードウェアエンコーダに対し**GPUテクスチャを直接渡す**経路を持つ。一次資料(2026-07-13確認): [texture-based NVENC導入コミット ed0c7bc](https://github.com/obsproject/obs-studio/commit/ed0c7bcd6a7a7ad844975beda5ec72aa9cc8fcf4)が「NV12出力テクスチャを**GPUから降ろさずに**NVENCへ直接渡し、性能を大幅に向上」と明記。実装は[jim-nvenc.c(SHA固定)](https://github.com/obsproject/obs-studio/blob/a249d26eaa2ff708b4b5540295abc15030111137/plugins/obs-ffmpeg/jim-nvenc.c)がテクスチャ配列+bitstream配列を添字で循環させる構造(循環キューの設計解説自体は二次資料[DeepWiki](https://deepwiki.com/obsproject/obs-studio/4.4.2-hardware-video-encoders)を参照)。

Motoliiへの含意: V2-3(HWデコードゼロコピー)の対になる**HWエンコード直渡し(候補・未台帳)**として台帳化を検討できる。macOSならVideoToolbox。出荷実績は上記コミットで一次確認済み。ただしこれはリング化の代替ではなく補完 — ソフトウェアエンコード(配布品質のx264等)が要る限りリードバック経路は残る。

## 3. cold shader compileの先例(B-4の傍証)

| 先例 | 方式 | 出典 |
|---|---|---|
| **Unreal PSO Precaching** | 使われ得るPSOを自動収集し**非同期に先行コンパイル**。Global Shader PSOは「初回使用ヒッチを起こすため**起動時にコンパイル**」。コンパイル未完のオブジェクトは**描画をスキップするかデフォルトマテリアルで代替** | [UE公式ドキュメント](https://dev.epicgames.com/documentation/en-us/unreal-engine/pso-precaching-for-unreal-engine)(公式)、[Epic技術ブログ「Game engines and shader stuttering」](https://www.unrealengine.com/tech-blog/game-engines-and-shader-stuttering-unreal-engines-solution-to-the-problem)(公式) |
| 同ブログの一般知見 | PSOコンパイルは数ms〜数百ms。ハードが既知のコンソールでは事前コンパイルできるがPCではGPU依存で不可 — **PC向けは実行時の先行コンパイル+フォールバックが正解**という業界結論 | 同上 |

B-4の提案(起動時prewarm+compile中はlast-goodまたはpass-through+初表示ヒッチのSLO)は、Unrealの出荷済み解(precache+描画スキップ/デフォルト代替)と同型。転移に必要な2つの部品も、それぞれ出荷済みの解が実在する:

**(1) 非同期コンパイル — 実現可能性はwgpuエコシステム内で出荷済み**。wgpu 29のpipeline作成APIは同期だが(反対側レビューB-4)、[Bevy 0.13公式リリースノート](https://bevy.org/news/bevy-0-13/)がwgpu 0.19(arcanization)への更新により「**シェーダーを非同期にコンパイルしてコンパイルスタッターを回避**できるようになった」と明記(公式)。ただしこの出典が支えるのは**wgpu上で非同期コンパイルが実現可能なこと**まで — 具体方式(別スレッドからの同期API呼び出し等)はこの出典だけでは特定できず、Motoliiでの利用には**ホスト側の改修が必要**(現行は`&mut PipelineCache`への同期的get-or-create。非同期コンパイル結果をどう合流させるかの設計)。実現可能性=確認済み、無改修転移=不可、として分けて扱う。

**(2) パイプライン列挙 — 出荷済みの解が複数あり、方式選択の問題**:

- **記録・再生型**: [Fossilize](https://github.com/ValveSoftware/Fossilize)(Valve公式リポジトリ)はVulkanパイプラインのCreateInfoを記録し、「**手動宣言ではなくロード時に自動生成**」するために再生する(README明記)
- **宣言収集型**: Unreal PSO Precachingはマテリアル等の静的宣言から使用PSOを収集する(公式ドキュメント、§3表)
- **Motoliiの現物との接続(接合条件あり)**: 現行[NodeDesc](../../crates/motolii-plugin/src/lib.rs)にshader/pipeline宣言は無い。Filter系は[ホスト所有PipelineCache](../../crates/motolii-gpu/src/pipeline_cache.rs)へ`PipelineCacheKey{id, wgsl}`で要求する(F-10実証済み)が、**捕捉は全面ではない** — [motolii-nodes](../../crates/motolii-nodes/src/lib.rs)のOverlay/Composite系はPipelineCacheを介さず直接`create_render_pipeline`している(2026-07-13確認: 直接生成2箇所)。また現行キーは固定レイアウト1形式(texture+sampler+uniform4)の識別子であり、Fossilizeが記録するlayout・render pass等を含む**完全なpipeline記述ではない**([Fossilize公式README](https://github.com/ValveSoftware/Fossilize)は`CreateInfo`一式を記録する)。したがってキー列の記録だけではFossilize型replayにならない

つまり正確な状態は「**一般解は2系統実在。Motolii側には、パイプライン生成の捕捉面をPipelineCacheへ統一しキーを完全記述へ拡張する接合工事が転移条件として残り、その上で方式選択**」。プラグイン数がゲームのPSO数より桁違いに少ない規模感の利は効く。

## 追調査の成果(2026-07-13、一次確認)

- ~~OBS循環テクスチャキューの一次資料~~ → **解決**: [導入コミット ed0c7bc](https://github.com/obsproject/obs-studio/commit/ed0c7bcd6a7a7ad844975beda5ec72aa9cc8fcf4)+[jim-nvenc.c(SHA固定)](https://github.com/obsproject/obs-studio/blob/a249d26eaa2ff708b4b5540295abc15030111137/plugins/obs-ffmpeg/jim-nvenc.c)(§2)
- ~~非同期コンパイルのwgpuでの可否~~ → **実現可能性のみ解決**: Bevy 0.13が出荷済み(§3)。Motolii側の結果合流設計は接合条件として残る
- ~~パイプライン列挙手段~~ → **一般解2系統を確認**: Fossilize型/Unreal型(§3)。捕捉面の統一が接合条件として残る
- Appleユニファイドメモリのreadback特性 → **アーキテクチャ前提のみ確認**: [MTLStorageModeShared公式](https://developer.apple.com/documentation/metal/mtlstoragemode/shared)はCPU/GPUのシステムメモリ共有を確認する資料であって、`copy_texture_to_buffer`のメモリ内コピーとGPU/CPU同期は消えない。wgpuのMetal上のstagingバッファ割当も未確認 — **Motolii経路は実測待ち**(§C-2ベンチに含める)

## 未確認の接合条件(採択判断・実装前に潰す)

- **捕捉面の統一**: Overlay/Composite系の直接`create_render_pipeline`をPipelineCache経由へ寄せ、キーを完全なpipeline記述へ拡張する(§3)
- **非同期コンパイル結果の合流**: `&mut PipelineCache`の同期get-or-create構造への非同期結果の合流設計(§3)
- **Metalステージングの実体**: wgpuのstorage mode割当の確認と開発主機での実測(上記)

## 未調査(反対側レビューで当たるべき点)

- Unity/OBS/Unrealの方式の**失敗事例**(レイテンシ起因のバグ、リング本数の選定ミス等)は未探索
- Unreal `ReadPixels`の同期ブロック挙動の一次資料(現状は二次記事のみ。本文からは分離済みで結論に影響しない)
- Fossilize型 vs Unreal型 vs ロード時フックの**Motoliiでの方式選択**(採択判断。一般解の存在は確認済み)
