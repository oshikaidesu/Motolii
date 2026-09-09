# GPUインスタンスを主ビューと反射ビューで共有する

状態: **縮小採用**。利用者「gpuベースでいきますよ、進んでくれ」。基準root `e8bc63ff` / fork `530bdf86907e6e3e0599938c12879815e0a012dc`。別タスクの計画と3文書の未commit差分は保持する。

## 先例と施工単位

[QualiArtsの描画最適化](https://speakerdeck.com/qualiarts/idoly-pride-graphics-optimization)、既存re_rendererのMeshDrawData/instance-stepped vertex bufferと、[wgpu 29 draw_indexedのinstance range](https://docs.rs/wgpu/29.0.0/wgpu/struct.RenderPass.html#method.draw_indexed)を確認した。現在は6面内でdraw dataを共有するが、2地点のcaptureと主ビュー間では同じmesh instanceの組立・normal matrix計算・uploadを再実行している。

初版は1評価sceneのmesh群を一度GPUへ上げ、各ビューが元instanceの選択範囲だけを変えて同じbufferを読む。構築時の入力indexとGPU並び順の対応をre_rendererが保持し、Motoliiは作者の配置と対象範囲を渡す。Repeatの意味評価をWGSLへ複写しない。

不透明・Normal・背景透過不要・clipなしのmesh群で比較する。条件外は既存経路を維持。反射の自己receiver除外、異なるmain runの部分集合、材質/field、Picking/Outlineの値と描画順を保存する。2Dは既存の共通surfaceのままで、今回はrectangle用GPU batchingを増やさない。GPU culling/ComputeによるRepeat評価/平面反射/SSRはこの実装単位に含めない。

## 検証と採否

同じDocument・時刻の旧経路と新経路を全画素で比較。10/100/1000個について静止・動的を測り、CPU意味評価/層組立/mesh準備、総時間、instance upload数とbytesを分離する。instance bufferの削減をdrawcall削減やGPU単独時間と呼ばない。GPU encoder timestampの逆転問題は前回記録を踏襲し、採否には使わない。

反射cacheは両modeで同条件。静止hit時も性能悪化がないか確認。既存52件と追加の共有/除外/順序/条件外fallbackを検証する。実窓は既存Flutter AccessibilityBridgeのクラッシュを切り分け、未完を完了扱いしない。

## 比較結果

fork `a97e03916c7bf6f03307b3cf2f9710ecebf2642b`。共有は既定で有効、不透明等の適用条件を満たす反射撮影時に使う。静止の反射cache hitでは従来通り主ビュー分だけを準備する。

Apple M4（8 GPU cores）/ Metal / 1280×720 / dev build。各modeを交互順に実行、warmup 3回＋20 sampleを独立2回（各mode合計40 sample）。反射cacheは双方有効。warmup込み**276組で全画素一致**。

| mesh個数 | 動的・個別転送 → GPU共有（総時間中央値） | 描画データ準備CPU | instance転送 bytes/frame |
| --- | ---: | ---: | ---: |
| 10 | 10.200 → 9.984 ms | 0.190 → 0.071 ms | 4,368 → 1,560 |
| 100 | 10.994 → 11.001 ms | 0.422 → 0.160 ms | 46,488 → 15,600 |
| 1000 | 22.644 → 21.065 ms | 2.809 → 1.030 ms | 467,688 → 156,000 |

1000個では総時間約7.0%、描画データ準備約63%、instance転送約66.6%の削減。転送個体数は2998→1000（旧方式は2地点でそれぞれreceiver 1個を除外し、主ビューも別途上げる）。GPU共有の効果は反射を撮り直すフレームにある。

静止では両modeとも1000個分156,000 bytesを1回上げる。総時間は10個9.120→8.665ms、100個9.141→9.191ms、1000個16.258→16.476msで、対応frame差の参考bootstrap 95%区間はいずれも0を含む。静止の性能改善・悪化はこの測定では確定しない。動的1000個の対応frame差は−4.399〜−0.340ms、10/100個は0を含む。小さい個数の総時間改善を主張しない。独立起動間・待機時間に変動があるため、前回の別実行の総時間と横比較せず、同じ実行の対で判断する。

1000個の動的なDocument評価は共有後も中央値4.796ms、層組立は0.774ms。これらはCPUに残っている。ComputeでRepeatの配置式を実行したものではなく、**GPU上の同じinstance bufferを反射・主ビューで読む**実装である。

[集計JSON](assets/2026-09-09-gpu-instance-sharing/summary.json)・[再集計script](assets/2026-09-09-gpu-instance-sharing/summarize.py)。同directoryにraw sampleを保存。総時間はCPU準備・submit・GPU完了待ち・readback回収を含み、GPU単独時間や実窓FPSではない。描画データ準備の計時はmeshに加え、同じdraw-data組立関数内のrect/cloud準備も含む。

```sh
cargo build -p motolii-render --example reflection_compare
motolii/target/debug/examples/reflection_compare /path/to/repeater-1000.rrd /path/to/result.json gpu-sharing
python3 docs/reviews/assets/2026-09-09-gpu-instance-sharing/summarize.py
```

比較作品は[既存の生成器](assets/2026-09-09-shared-reflections/create_scene.py)を使用。JSONの`variant_enabled`と`gpu_instance_sharing`が比較のswitchで、`cache`は両modeでtrue。

## 検証結果

- 全体54件を実行して53件通過。追加testのClipをRepeat後へ置くと集合への効果になるため、個体へのClipを検収する配置（Repeat前）へ修正し、残る1件を再実行して通過。**54件の確認が揃った**。runtime修正でテストの画を都合よく変えていない。
- 追加検証は10/100/1000個のupload数と全画素一致、半透明時のfallback、fieldとClip、2Dを挟む複数main run、個別surface parameterの往復。既存の屈折順序・coverage・clip・preview/exportも通過。
- re_rendererのdraw phaseはstable sortであり、分割rangeには同じ代表位置を使い元のrange順を維持する。source-index対応はGPUに詰めた順と分離し、receiverを全submeshから除外する。
- 最終clippy（all-features/tests/examples）とnative build完走（warningあり）。owned-budget全pattern一致、Stage5検査通過。
- 最終nativeの実窓で1000個を表示し、カメラ後方の反射元をY回転すると全体の映り込みが更新されることを確認した。Cmd+Z/Control+Zでは今回の操作で値の復帰を確認できなかった。既存のFlutter AccessibilityBridgeクラッシュがあるメニュー経路は再試行せず、UIのUndo検収は残す。UI/SDKを変更したものではない。テスト作品は保存せず終了した。
- 元作品は前回同様`~/.local/state/motolii-stage5/checkpoints/shared-reflection-2026-09-09/motolii-ux-current-2026-09-05.rrd`から復帰し、frame 0の元作品と未変更状態を実窓で確認した。Documentsの原本は変更していない。
- hygieneは旧UIの最大2804行・長大file23件に加え、build後のincremental session 93（上限60）で不通過。live build cacheを削除して数だけを合わせていない。docs checkerの既存「比率 aspect」の状態`未決`も別タスクの裁定として保持する。

## 実装上の費用と制約

forkのMeshDrawDataが元input indexと中心位置の対応をCPU側に保持する（この機器ではinstanceあたり32 bytes）。選択後のdraw dataは同じGPU instance bufferを参照する。選択に穴があればdraw rangeが分かれるため、drawcallが減ると主張しない。比較の両modeは同じforkであり、この対応表の追加費用も両方に含む。

静止の反射cache hitでは共有sceneを組み立てず、従来の主ビュー1回の準備を使う。capture miss時だけ共有GPUデータを準備する。合算bufferがdevice上限を超える場合も既存経路へ戻す。

forkの指定formatコマンド`pixi run rs-fmt`はpixi未導入で実行できなかった。編集対象だけをrustfmt（edition 2024）で整形した。rootはclippyを先行実行し、GPU画像比較を行う。

CPU区間の内訳はこの平坦な比較作品での値。入れ子の合成では描画データ準備が層組立の中で行われる場合もあるため、各内訳を無条件に足し合わせない。
