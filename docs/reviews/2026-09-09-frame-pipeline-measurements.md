# 操作・状態生成・描画経路の分離計測 — 2026-09-09

## 条件

保存済み `light-in-form-observed-20260909.rrd`（15レイヤー、1600×1000）を別プロセスのEditorRuntimeへロード。現在のDebug native libraryをコピーして固定した。作品ファイルは保存・変更していない。ガラスなし条件ではメモリ内のコピーから `motolii.glass` を3件（layer 3/4/5、effect 0）削除した。

- library SHA-256: `b077143d403db7ad37539407e948a4aa96fa798ddd66271a351294b97d363873`
- document SHA-256: `43f614f7d82612638b46bbd6e19645c2008508d28350e3ede063c17145f1d785`
- 手順・生データ: `/tmp/motolii-pipeline-audit-20260909/bench.py`、`results.json`、`profile.py`、`selection.py`、`selection-results.json`、sampleテキスト。
- 既存のnative FFI、Swift hostと同じIOSurface生成属性、Xcode SDKのCoreFoundation/IOSurfaceヘッダを参照した。製品コードへの計測用変更や全buildは行っていない。
- 状態問い合わせ30回。画像はwarmup 5回後に20回。画像ごとに新しいIOSurfaceを作り、native render完了後にreleaseする。静止比較はframe20、移動比較はframe20以降。
- Flutter・CVPixelBuffer wrapping・画面合成・入力待ち行列を含まない。native renderはCPU評価とGPU完了待ちを含む。GPU単体実行時間ではない。

## 画像を一度も描かない問い合わせ

中央値。native requestはRustの処理と返答文字列生成まで。Pythonによる返信コピーとJSONデコードは別に測った。

| 条件 | 停止中status | 再生状態status | 選択要求＋status | renderInfo |
|---|---:|---:|---:|---:|
| 元の15レイヤー | 12.774ms | 4.189ms | 11.516ms | 0.005ms |
| ガラス3件削除 | 9.850ms | 3.568ms | 10.036ms | 0.005ms |
| 空の書類 | 0.624ms | 0.018ms | 対象なし | 0.005ms |

選択要求を30回実行してもnativeのrenderCount増分は0。この約10msは画像描画の費用ではない。ガラスなし選択返信のコピー中央値は0.017ms、Python JSON decodeは0.768ms。従って「言語間のコピーが約10ms掛かっている」という説明も成立しない。選択操作をアプリで実行すると、この後DartのrequiresRenderにより別途renderが呼ばれる。

## 返信に載っているもの

| 項目 | 元の書類の値のサイズ | 空の書類の値のサイズ |
|---|---:|---:|
| backgrounds | 158,618 B | 158,618 B |
| assets | 55,685 B | なし |
| layers | 40,574 B | 空 |
| fontFamilies | 13,132 B | 13,132 B |
| easeKinds | 13,118 B | 13,118 B |

空の書類でも停止中返信全体は188,532 B。15レイヤー・ガラスなしでは約284,531 B。背景サムネイル、フォント一覧、補間見本など低頻度の情報が通常の状態返信に含まれる。大きい返信だけで全遅延を説明するものではない。空の書類のnative生成時間は0.624msであり、レイヤーに関連した繰り返しの問い合わせ・変換も大きい。

## 状態生成のCPUサンプル

ガラスなし・停止中statusの連続取得を5秒採取。main thread 4116標本。inclusiveな関数の滞在標本には、`resolved_layers` 540、`authored_signature` 368、`inspector_data_from_doc` 255が含まれる。入れ子があるため合算しない。

コード照合:

- snapshot冒頭でresolved_layers。boundsの取得にも関連評価がある。
- 保存済みとの差を判定するauthored_signatureが、通常の状態取得で作品全体を文字列化する。
- 全レイヤーのInspector用データとJSON表現を作る。
- サムネイルには既存キャッシュがあり、毎回PNGを再生成すると断定してはいけない。既存文字列を返答へ再び含める処理は残る。

判断: 問い合わせのたび作品の広い投影を組み立てる設計が、レンダリングと独立した実測費用を持つ。

## 画像生成と、その後の状態生成

最初のrunの中央値。各列の中央値の和はtotal中央値と一致するとは限らない。

| 条件（再生状態） | native render | 描画後status等 | 合計（寸法・surface含む） |
|---|---:|---:|---:|
| 元の書類・同じframe | 16.717ms | 5.635ms | 23.448ms |
| ガラスなし・同じframe | 13.704ms | 4.106ms | 17.857ms |
| 元の書類・進むframe | 21.808ms | 4.979ms | 26.896ms |
| ガラスなし・進むframe | 15.279ms | 4.569ms | 20.824ms |
| 空・同じframe | 1.397ms | 0.036ms | 1.472ms |

ガラスなし画像生成だけの別サンプルではmain thread 3425標本のうち2958がnative renderのdevice pollに滞在した（約86%）。これはGPU計算時間やGPU使用率ではない。CPU側でGPU完了を待つことに時間が使われている。実際のGPU仕事、driver/queueの待ち、timerの遅れをこのsampleだけでは分けられない。

## 一般操作の連鎖と交絡

現在の呼び出し順序をheadlessで再現した: select＋status → renderInfo＋surface → native render → status。作品のframeを固定し、選択だけ交代する。

original / without_glass / without_glass / originalの順に計測したtotal中央値は45.0 / 64.5 / 53.1 / 61.8ms。計測中にはMetalトレースの保存、別の描画実験、ビルドが同時稼働しており、負荷の揺れが大きい。**この4値をガラスの有無の因果比較やユーザー画面の実遅延として使わない。** 確認できたのは、通常操作が状態生成を前後に含む直列経路を通ること。その経路内のどの部分が変動しているかを記録する必要がある。

## GPUトレース

Metal System Traceを試行。launch方式の最初のtraceは対象encoder行が0件で、GPU時間の根拠に使えない。初期化済みの別プロセス（PID70178）へattachした再採取は取得できた。Apple M4上でガラスなし・frame20固定を反復した。

- finalize passの297回を基準にすると、Metal command-buffer submissionsは4158件、平均14本/論理フレーム。
- 実encoderは1485件、5個/論理フレーム。内訳はsequential main pass 2回、finalize 1回、composite 1回、blit 1回。
- CPUのencoder構築時間中央値: sequential 0.108ms（各回）、finalize 0.050ms、composite 0.040ms、blit 0.035ms。GPU実行時間ではない。
- 内部コマンドバッファのラベルにはTransit（1485件）、Pre Pass（1188件）、Signal等がある。これらを不要として削除できるという証拠ではない。
- アプリ側 `sequential.rs::flush_pending` は既に複数のwgpu CommandBufferをbatchにしてsubmitする。14本という観測は14回のwgpu Queue::submitを意味しない。下位Metalの分割・同期と、上位のview/command-buffer境界の対応を調べる必要がある。
- 対象に対応付けたGPUの純粋な実行時間は今回確定していない。opaqueな実行イベントコードやdriver intervalをGPU shading時間へ読み替えない。

集計JSONは [metal-summary.json](assets/2026-09-09-frame-pipeline/metal-summary.json)。元のtraceとXMLは上記/tmpディレクトリ。

## 結論と修正の境界

- 確認済み: ガラスを外して画像を描かなくても、選択の状態返信に約10msのnative処理がある。
- 確認済み: 通常の状態返信に変更していない広い情報が含まれ、選択→描画後に状態生成が繰り返される。
- 確認済み: 描画経路でCPUがGPU完了待ちに長く滞在する。
- 未確定: GPUの純粋な実行時間、ガラスの単体費用、Flutter込みの入力→表示遅延、その支配要因の割合。

修正の優先候補は、statusを必要なconsumerと変更範囲に応じた投影にすること、作品の変更がないときの評価・保存差分判定を再利用すること、画素入力が不変の操作で作品画像を生成し直さないこと。GPU同期の削除や画質低下はこの結果からは正当化できない。


## 保存した小さい証拠

[分離計測値](assets/2026-09-09-frame-pipeline/results.json)、[返信内訳](assets/2026-09-09-frame-pipeline/payload-sizes.json)、[交絡を含む選択連鎖の計測](assets/2026-09-09-frame-pipeline/selection-results.json)。元書類のSHA-256は調査後も一致した。製品コード、開いている作品、他の作業プロセスは変更・終了していない。終了したのはこの調査で起動した計測プロセスのみ。
