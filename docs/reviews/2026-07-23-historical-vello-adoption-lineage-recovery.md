# Vello採否スパイクの価値回収（Unit 5G、2026-07-23）

状態: **局所renderer採択を維持／製品統合は未実装**

対象: [R8軽量レビュー](2026-07-10-R8-vello-review.md)と[S3(R8)結果報告](../spikes/s3-vello.md)のcutoff全2版（4,712 bytes）

関連: [M1仕様](../specs/M1-vertical-slice.md)、[M4仕様](../specs/M4-cache-and-analysis.md)、[M5仕様](../specs/M5-3d-and-post.md)、[native renderer再選定](2026-07-21-native-surface-renderer-reselection.md)、[第一コード監査回収](2026-07-23-historical-first-code-audit-lineage-recovery.md)

## 1. 結論

2026-07-10のisolated spikeは、Vello 0.9系とwgpu 29系を同じdeviceで使い、CPU pixel bridgeなしにVello出力をwgpu textureへ置けるという**成立性**を示した。この採択は後続判断で、direct wgpu primitive batchを主とし、Velloを複雑path／textだけへ局所利用する形に縮小されて生きている。

一方、現行workspaceはVello／usvgへ依存せず、M4-K6、M5-P6、M3-U3a-2はいずれも製品統合前である。使い捨てspikeの成功、Apple M4での時間値、Cargo.tomlのversion rangeを、現行lock、CI、product lifetime、SVG全機能、text描画の完成証拠にしない。

## 2. 歴史主張の処分

| 主張 | 処分 |
|---|---|
| Velloと本体wgpuが同じdevice／textureで同居できる | **維持**。固定時点のheadless成立性。window surface、device lost、product cacheまでは証明しない |
| Rendererを長寿命化する | **維持**。loop内／exportごとの再生成を拒否する。実ownerとcold compile admissionは製品利用者側で閉じる |
| Vello出力をstraight alphaとして一度だけpremultiplyする | **維持**。K6とP6が別adapterを作らず、CQ-7の単一GPU境界を先に閉じる |
| usvgから小さな自前adapterで変換する | **縮小維持**。当時のpath＋単色fill成立性だけ。stroke、gradient、clip、image、text、resource、error意味は未証明 |
| `vello_svg`を永久に使わない | **時点限定へ訂正**。当時のversion skewは事実だが、将来の依存採否は固定versionとclosureを再照合する |
| Velloをベクター描画基盤として全面採用する | **後続決定で縮小**。Timeline大量primitiveやUI scene graphはdirect wgpu、Velloは局所pass |
| Apple M4の約895 ms／18 msを製品性能値にする | **棄却**。単一機、readback込み、one-shot測定でSLOではない |
| Vello移行時に既存goldenを一斉regenerateする | **撤回**。rendererへoracleを合わせない。意味を維持できない変更は独立したoracle migration判断へ戻す |

## 3. 現行コードとの照合

- root workspaceのCargo manifest／lockと製品crateにVello／usvg依存や呼び出しは無い。
- `spikes/vello-eval`のsourceは残るが、tracked Cargo.lockは無い。現在の依存解決を当時のexact closureと呼べない。
- `motolii-core`にはCPU色入力のpremultiply helperがあるが、Vello textureを受けるGPU単一adapterは無い。CQ-7はK6／P6の先行条件として未実装のままである。
- M4-K6はSVG import＋vector source、M5-P6はshape済みglyph run描画、M3-U3a-2はwindowed native surface比較であり、同じ「Vello統合」名で一発注へ束ねない。
- native UIの現行第一候補はdirect wgpu primitive batch＋局所Vello passであり、Velloへinput、layout、selection、scene ownershipを渡さない。

## 4. 再入場条件

### 4.1 共通境界

1. 採るVello／wgpu／usvgのexact versionとlicense、feature、backend closureを固定する。
2. Renderer／resource／pipeline cacheのownerを製品session寿命で一つにし、steady frameで再生成0を審判する。
3. straight textureからpremultiplied正規形へ入るGPU境界を一つにし、二重変換と変換漏れの独立oracleを置く。
4. device lost、surface reconfigure、cold start、VRAM budgetを各利用者のfixtureで閉じる。

### 4.2 利用者別

- **M4-K6**: path、group、fill、stroke、transform、clip、unsupported resourceの意味表とSVG goldenを先に固定する。
- **M5-P6**: fontique／harfrustのshape結果とcluster対応を正本にし、Velloはglyph runのdrawだけを担当する。
- **M3-U3a-2**: 同じTimeline fixtureでdirect wgpuのみと局所Vello passをwindow present／input／WebView同居まで比較する。

## 5. 復活させないもの

- spike成功をK6、P6、U3a-2の実装完了と呼ぶこと。
- VelloをTimeline primitive、DOM chrome、layout、hit-test、Document、plugin契約のownerにすること。
- `Renderer::new`をframe、panel、window、exportごとに作ること。
- K6とP6が別々のstraight→premul変換を持つこと。
- 当時の`vello_svg`不整合や自前60行adapterを恒久API判断にすること。
- Metal一機の結果をlavapipe、Windows、他adapter、Final再現性へ外挿すること。
- AAやraster差に合わせてsemantic golden、許容閾値、期待値を書き換えること。
- K6、P6、U3a-2を「Vello共通化」として一つの実装単位へ束ねること。

## 6. 固定証跡とcoverage

処分対象は`e710f6522f299d348fba66f147468d10559b631f`と`cd27770dc6667da1e6aafdc4ed2699365247aba9`の2 blob、4,712 bytes。receiptは`05g-vello-adoption.tsv`に固定する。

本Unit後のstrict progressは360 / 1,797（20.0%）、未処分1,437である。
