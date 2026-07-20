# M3 GPU Preview / Viewport先例調査

作成日: 2026-07-18
状態: **先例調査・技術リスク切り分け / UI shell結論は置換済み**。ownership、lifecycle、display pool、負例は有効だが、Slint維持の結論は2026-07-18の[egui採用判断](2026-07-18-m3-egui-selection.md)で置換された。M3製品実装、公開API、Document schemaの変更を許可しない。

## 0. 結論

M3の主Previewについて、別の動画エンジンやnative child surfaceへ乗り換える理由は見つからなかった。現行の

`Motolii render worker → 同じwgpu device上のRGBA texture → slint::Image`

は、Qt Quickのoffscreen color buffer合成、GodotのViewportTexture、OBSの「描画コアがpreview/finalを所有しfrontendは表示先を持つ」という三系統と整合する。S1/R9も同じownershipをApple M4 / Metalで通した。**当時はSlint shell維持を結論にしたが、その後の比較と実機probeにより、同じownershipのまま投影先だけegui native textureへ変更した。**

ただし「技術的に全部証明済み」ではない。未証明なのは方式ではなく、次のlifecycleとplatform境界である。

1. resize / DPI変更 / minimize / hide-show時のtexture寿命と再生成
2. 実`RenderedFrame`を使う連続更新、古いgeneration破棄、UI応答
3. Stage contentの上に置くoverlay、clip、pointer hit-testの座標一致
4. 別window / 別monitor Preview
5. macOS以外のDX12 / Vulkan実機

これらはUIの見た目や操作設計を決めずに検証できる。M3入場前後の隔離branchで下記PV-1〜PV-4を通せば、主Previewで残る停止要因をUI設計側へほぼ限定できる。

## 1. 調査質問と境界

比較したのは「どのtoolkitを使うか」ではなく、次の接合面である。

- GPU device / render loopの所有者
- UI shellへのtextureまたはsurfaceの受け渡し
- resize / DPI / visibilityとresource寿命
- overlay / input / coordinate transform
- frame pacingとUI vsyncの関係
- multi-window / multi-monitor
- 実運用で確認された停止・互換性事故

GPL/AGPL製品はリポジトリ規約に従い、コードを読んでいない。Kdenlive等は公式manualだけを使用した。MIT/商用frameworkも、転用対象は設計パターンと審判であり、コード移植は本調査の非目標である。

## 2. Motoliiの現在地

### 2.1 コードで確認できる事実

- `spikes/s1-slint/src/main.rs`は`GpuCtx::new_for_ui()`でMotolii側がdeviceを作り、`BackendSelector::require_wgpu_29(WGPUConfiguration::Manual)`でSlintへ渡す。
- render専用threadが`wgpu::Texture`を容量1 channelへ`try_send`し、UI threadの16 ms Timerが`try_recv`して`Image::try_from(texture)`を設定する。
- S1はApple M4 / MetalでManual device共有、1280×720 texture import、30 fps更新、IME、UI操作の共存を合格としている。
- `spikes/r9-preview/src/main.rs`も同じManual共有deviceとtexture importを使い、実`render_frame`経路の静止scrubを結線している。
- 両spikeとも`set_rendering_notifier`を使っていない。

### 2.2 文書とコードの不一致

[M3仕様](../specs/M3-ui-integration.md)の方針節は、device共有を

`require_wgpu_29(WGPUConfiguration) + set_rendering_notifier`

と記述し、S1を実装例に挙げる。しかし実際のS1は**Manual構成だけ**で共有し、notifierを登録しない。Slint公式も、同じdevice/queueを得る方法としてManual構成とrendering notifierを代替経路として説明している。

さらにSlint issue [#12030](https://github.com/slint-ui/slint/issues/12030)は、FemtoVG + wgpu 29でrendering notifierを登録すると、空のnotifierでもpipeline cacheが毎frame消され、macOS / Metalでpipelineとshaderが毎frame再作成された事例を報告している。2026-07-18時点でopenである。

したがって本線は次のように読む。

- device共有: `WGPUConfiguration::Manual`
- frame受け渡し: workerが作ったtextureを`Image::try_from`
- UI更新: event-loop上の非blocking通知または短いpoll
- `set_rendering_notifier`: **主Previewには使わない**。underlay/overlayが不可避になった時だけ別spikeへ戻す

これは新設計ではなく、既に動いたS1/R9と仕様文を一致させる補正候補である。仕様編集は本調査では行わない。

### 2.3 Slint側の固定条件

Slint 1.17の公式文書から、現在の接合条件は明確である。

- `require_wgpu_29`は外部wgpu renderer統合用の公式経路
- `unstable-wgpu-29`は通常のAPI安定保証外で、Slint minor更新時に変更・削除され得る
- Slint自身が意図しないcompile breakを避けるため`~1.17`を推奨
- texture importは同じdevice/queueで作られたtextureを前提とする
- import可能formatはRGBA8系、usageは少なくとも`TEXTURE_BINDING | RENDER_ATTACHMENT`

よってSlint / wgpuのversion固定は製品仕様の固定ではなく、**adapter境界のcompile-time整合**である。更新時はversion bump PR内でPV-1/PV-2を再実行すればよく、Documentやplugin契約へversionを出す必要はない。

## 3. 先例比較

| 先例 | 公式資料で確認できる事実 | Motoliiへ移せる部分 | 移さない部分 |
|---|---|---|---|
| Qt Quick `QQuickRhiItem` | custom rendererがoffscreen color bufferへ描き、Qt Quick sceneと合成する。GUI itemとrendererを分離し、通常はrender threadで描く。同じwindowのRHI/deviceを使い、item size×DPRに応じtextureを再生成する | shell item / render worker分離、同一device、offscreen texture合成、resize/DPRをresource lifecycleとして検査 | QRhi private API、GUI threadを止める`synchronize()`方式 |
| Godot SubViewport | SubViewportをrender targetとしてtextureを取り出し、scene内の別objectへ表示できる。visibilityに連動する既定更新とNever/Once/Always/When Parent Visibleを持つ | Previewのvisible/dirty/update policyを明示し、非表示時に無駄なrenderを続けない | Godot node/resource model |
| OBS/libobs | 専用graphics threadがpreview displayとfinal mixを描く。frontend displayはnative handleとdraw callbackを持つ。main textureをpreviewへ描ける | rendererがpreview/final関数を所有し、UIは表示先と入力を担当する。resize/enable/disableを明示する | native child displayの埋め込み。OBS自身も同一base window内の複数displayがmacOSでpresentation stallを起こすと警告 |
| Kdenlive | monitorはdirect-control overlay、preview resolution、別window化を持つ。一方、公式manualはvideo playbackのGPU支援が限定的で、不足時にpreview renderingを推奨する | overlay操作とcontent previewを概念分離する。別windowは後段の独立機能にする | CPU中心pipelineや別GPU engineの後付け |
| OpenCut | MIT。2026年v0.3.0でWebGL rendererをRust/wgpu WASM compositorへ置換。同じreleaseでPreviewのsnap guide欠落、rotation handleのclip漏れ、canvas sizeに応じたhandle位置ずれを修正 | GPU compositorをUIとは別のcoreへ置く方向。Preview overlayのtransform/clipを独立fixtureで検査する必要性を示す反面事例 | React/WASM UI構造、開発中の未確定API。成熟先例とは数えない |
| Slint 1.17 / issue #12030 | Manual shared WGPUとtexture importは公式経路。wgpu APIはunstable feature。FemtoVG rendering notifierの毎frame pipeline再生成が報告中 | Manual shared device + imported textureを維持し、version bumpをadapter内へ閉じる | notifierを主Previewの常用更新hookにすること |

### 3.1 収束している部分

独立した三系統で、次が共通する。

1. UI toolkitはshell、layout、inputを持つ。
2. domain rendererはcontentの描画とGPU resourceを持つ。
3. 接合物はoffscreen textureまたは表示targetであり、CPU pixel bridgeではない。
4. resize / visibility / device変更は暗黙にせずresource lifecycleとして扱う。
5. UI表示周期を作品時間やFinal評価の正本にしない。

この範囲は既存M3仕様の「コアがdeviceを作る」「UI threadでrenderしない」「同期readback禁止」「音声/Transport主クロック」と一致し、新しい公開境界を必要としない。

### 3.2 収束していない部分

- overlayをcontent textureへ同時描画するか、UI scene上に合成するか
- multi-windowで同じtextureを直接共有するか、各window用表示targetへ再描画するか
- visibility/dirty通知の具体的なSlint callback
- OSごとのpresent modeとDPI移動時の細部

これらを先例の多数決で固定してはならない。PV-1〜PV-4の実測とStage UI設計の結果で選ぶ。

## 4. M3前の隔離branch検証

本節は新しい製品ticketの確定ではない。既存U1a/U1b/U1eへ入る前に、技術的不確実性を小さなfixtureで取り除く提案である。Document、公開API、plugin契約、永続設定は追加しない。

### PV-1: single-window texture lifecycle

目的: S1をSlint 1.17.1と現在のworkspaceで再固定し、静止表示では見えないresource事故を潰す。

最小fixture:

- Manual共有device、`Image::try_from(texture)`、notifierなし
- RGBA8 + `TEXTURE_BINDING | RENDER_ATTACHMENT`
- checkerboardとframe/generation番号だけをGPU描画
- window resize、minimize/restore、hide/show、DPI/monitor移動を記録

必須負例:

- notifierを登録しない
- UI threadの`device.poll(Wait)`、download、render呼出しなし
- 毎frameのtexture/pipeline/shader生成なし
- Slint logical pxをDocument/domainへ渡さない

合格証跡:

- release buildで10分更新し、hang/device lost/import errorなし
- 100回resize後も最新generationを表示
- minimize/hide中のrender policyと復帰時間をraw logへ保存
- pipeline/shader/texture作成counterを保存し、定常frameで増加しない
- Metal / DX12 / VulkanごとにOS、GPU、display scale、window/viewport、Slint/wgpu versionを記録

性能の絶対fpsやlatency閾値はここで発明せず、G0-4手順に従いraw値を残す。

### PV-2: 実`RenderedFrame` + latest-generation

目的: S1の色textureではなく、本番と同じrender関数・専用出力texture寿命でU1bの成立性を先に確認する。

最小fixture:

- requestは最新値置換mailbox
- resultはgeneration付き
- 意図的な遅延注入で完了順を反転
- UI event loopへ戻してから`Image`を設定

合格証跡:

- 100連続seekのsenderがblockしない
- 古いgenerationを一度も表示しない
- render中もwindow resize、button、IMEが応答する
- previewとFinalが同じ`render_frame(..., Quality)`系を通る
- shared deviceの同期readbackゼロ

これは既存U1b-1/U1b-2の設計を変えず、実GPU接合の早期証拠を取るだけである。

### PV-3: Stage overlay / clip / hit-test

目的: 色やhandle意匠を決める前に、Preview textureと編集overlayのz-order、clip、座標変換、input routingだけを証明する。

最小fixture:

- contentはcheckerboard + Output Frame
- overlayは選択矩形1個、anchor 1個、frame外object 1個
- Stage pan/zoomとwindow DPIを変え、同じ共有transformからdraw/hit-testを計算
- pointer capture loss / Escape / window focus lossでCancel

合格証跡:

- overlayがPreview領域外へ漏れない
- DPI差でも同じ正規化gestureが同じdomain deltaになる
- frame外objectを選択できる
- Cancel時にDocument変更ゼロ
- layout/hit-test modelはSlint非依存

GPU passへoverlayを焼くかSlint sceneへ重ねるかは、このfixtureで両案のz-order、input、frame costを比較してから選ぶ。どちらを選んでもselectionやlogical pxをDocumentへ保存しない。

### PV-4: separate-window Preview

目的: 既存U1eを主Editorの成立条件から分離し、multi-monitor機能だけのリスクとして閉じる。

前提: PV-2合格後。

比較する案:

1. 同じimport textureを第二Slint Windowへ投影
2. 同じrender結果からwindow別の表示targetを作る

合格証跡:

- monitor間移動とscale変更でDocument、評価時刻、pixel内容不変
- main/secondaryの片方を閉じても他方とrender workerが継続
- macOSでpresentation stallなし
- window数に比例してrender graph自体を重複評価しない

失敗してもU1a/U1b/U1fを止めない。v1の別window Previewを延期し、単一埋め込みPreviewを出荷可能な縮退先にする。

## 5. 実装停止線

次の兆候が出たら、便利な共通化で迂回せずspikeを止める。

- native child window/surfaceをSlint layout内へ重ねないと成立しない
- `set_rendering_notifier`が主Previewの常時更新に必要
- second device、CPU copy、同期readbackが必要
- Slint/winit/native handle型を`motolii-ui`外の公開型へ出す必要
- resize/DPI/event列をDocumentまたはplugin契約へ保存する必要
- preview専用の第二render関数を作る必要
- 同じbase windowに複数present surfaceを置く必要
- texture/pipeline/shaderを定常frameごとに生成する必要

停止時は「Slintが駄目」と一般化せず、どのOS/backend、どのlifecycle event、どのresource ownershipで破れたかをraw logと最小reproで記録する。

## 6. 既存M3 ticketへの翻訳候補

反対側レビュー後に採択する場合も、ticket構造を大きく変える必要はない。

| 既存ticket | 追加で参照する証拠 | 境界 |
|---|---|---|
| U1a-1 shell/static viewport | PV-1 | notifierなし、表示成立まで。resize/DPI/resource lifetimeの負例は2026-07-20分割後のU1a-1bが持つ。worker/seekは混ぜない |
| U1b-1/2 mailbox/stale result | PV-2 | 実`RenderedFrame`のGPU接合。強制cancelは要求しない |
| U1f Stage View | PV-3 | overlay意匠ではなくtransform/clip/hit-testの成立性だけを先に使う |
| U1e separate window | PV-4 | 主Editorの出荷blockerにしない |
| Slint version bump PR | PV-1の短縮版 | adapter内更新。Document/plugin契約変更なし |

主Previewの技術クリティカルパスは`PV-1 → U1a-1 → PV-2/U1b`で閉じる。PV-3はStage UIと並行、PV-4はその後でよい。

## 7. 調査の限界

- Qt/Godot/OBSは同じ所有原則を示すが、Slint + wgpu 29の代替実装ではない。
- S1の実測はApple M4 / Metalだけで、DX12/Vulkanを証明しない。
- OpenCutのwgpu compositor移行は2026年v0.3.0と新しく、長期運用済みの成熟先例としては扱えない。
- Kdenliveは反面例として有用だが、Motoliiと異なるbackend/歴史を持ち、GPU不足の因果を単独で一般化できない。
- Slint #12030は報告者のreproと分析であり、Motolii現行経路が同じ不具合を踏んだ証拠ではない。現行経路はnotifierを使わないため、監視対象かつnotifier案の負例としてのみ扱う。

## 8. 一次資料

- Slint 1.17 [`BackendSelector::require_wgpu_29`](https://docs.slint.dev/latest/docs/rust/slint/struct.BackendSelector)
- Slint 1.17 [Cargo features / unstable-wgpu-29の安定性とtilde固定](https://docs.slint.dev/latest/docs/rust/slint/docs/cargo_features/)
- Slint 1.17 [wgpu integration source documentation](https://docs.slint.dev/latest/docs/rust/src/slint/lib.rs)
- Slint issue [#12030: rendering notifierで毎frame pipeline再生成](https://github.com/slint-ui/slint/issues/12030)
- Qt 6 [`QQuickRhiItem`](https://doc.qt.io/qt-6/qquickrhiitem.html) / [`QQuickRhiItemRenderer`](https://doc.qt.io/qt-6/qquickrhiitemrenderer.html)
- Godot stable [Using Viewports](https://docs.godotengine.org/en/stable/tutorials/rendering/viewports.html)
- OBS [Backend Design](https://docs.obsproject.com/backend-design) / [Frontends: Displays](https://docs.obsproject.com/frontends)
- Kdenlive 26.04 [Monitors](https://docs.kdenlive.org/en/user_interface/monitors.html)
- OpenCut [repository / architecture status](https://github.com/OpenCut-app/OpenCut) / [v0.3.0 release](https://github.com/OpenCut-app/OpenCut/releases/tag/v0.3.0)
