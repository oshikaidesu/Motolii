# M3 UI基盤 egui採用判断（2026-07-18）

ステータス: **歴史的採用決定 / 製品runtime採用は撤回 / 比較・診断baseline限定**。2026-07-24にeguiを標準製品runtimeの候補から外した。本書の測定事実と、現行mainで完了したU0a〜U0e-1、U1a-1/2、U1b-1/2等のegui基準実装は比較・回帰・診断証拠として保持するが、新しい製品shell/panel/Timeline/Stage/theme/componentをeguiへ実装しない。物理撤去は[UI runtime責任境界](../ui-runtime-architecture.md)のG0-9撤去条件成立後に分離して行う。

## 1. 2026-07-24の処分

- eguiを標準製品runtimeへ採用しない
- React所有面をeguiへ再実装せず、native Stage/Timelineをegui widget/callbackで包まない
- 既存shell、native texture preview、layout投影、render worker、IME/lifecycle試験はbaseline/debug・回帰比較として当面保持する
- direct wgpuまたはWebView/native platform受入の失敗だけでeguiを自動的に製品候補へ戻さない。再採用には正本文書とM3仕様の明示改訂を要する
- 既存baselineの撤去は、G0-9のplatform受入と代替診断経路が閉じた後の独立作業とする

## 2. 2026-07-18時点の歴史的決定

- 当時は製品UIへRust nativeの**egui**を使うと決めた。初期統合の検証版は`egui` / `eframe` / `egui-wgpu` / `egui-winit` 0.35、`egui_tiles` 0.16、wgpu 29を使った。この採用結論は2026-07-24に撤回した
- `motolii-gpu`がwgpu device/queueを所有し、UI shellは`egui_wgpu::WgpuSetup::Existing`で借りる。第2deviceやCPU pixel bridgeを正規経路にしない
- previewは同一device上の`Rgba8Unorm` `TextureView`を`egui_wgpu::Renderer::register_native_texture`で登録して表示する。display slot生成時に安定viewを作り、rendererを得られる`eframe::CreationContext`で一度だけ登録する。毎frame、resize、DPI変更、minimize/restoreごとにsampler/bind groupを作らない
- toolkit依存は`motolii-ui`内へ閉じ、domain intent、Document command、render/eval/plugin公開APIへegui/eframe/winit型を出さない
- 可変panelは`egui_tiles`をruntime投影先の第一候補としていた。`Tree`、`TileId`、crateのserde形を製品設定の正本にせず、Motolii所有の安定したlayout modelから投影する境界はtoolkit横断で維持するが、`egui_tiles`を製品runtimeへ採用しない
- v1 plugin UIは従来どおり`NodeDesc`からHostが自動生成するpanelだけを公開境界とする。plugin所有のegui code、native widget、自由wgpu UIは公開しない

## 3. 当時の変更理由

Slint S1は2026-07-11時点のApple M4 / MetalでManual device共有とtexture importを実証したため、失敗ではない。一方、M3着手前の再調査で次を確認した。

1. Slintのwgpu接合は`unstable-wgpu-29`とrenderer featureの組合せへ依存し、minor更新時の変更可能性がある
2. egui-wgpu 0.35は既存の`Instance / Adapter / Device / Queue`を渡す[`WgpuSetup::Existing`](https://docs.rs/egui-wgpu/0.35.0/egui_wgpu/enum.WgpuSetup.html)を通常の公開APIとして持つ
3. 同じrendererは既存`TextureView`を[`register_native_texture`](https://docs.rs/egui-wgpu/0.35.0/egui_wgpu/struct.Renderer.html#method.register_native_texture)でoffscreen imageとして表示できる
4. editor型の高密度UI、Host自動生成parameter panel、Rust/LLMによるcomponent単位の変更、利用者が組み替えられるpanel構成はimmediate modeと相性がよい
5. 判断時点（2026-07-18）の`motolii-ui`はSlint接続確認用の空骨格で、製品画面や`.slint` componentは未実装であった。toolkitを変える費用が最小の時点であった（U0a完了後はegui骨格へ置換済み）

## 4. Apple M4 / Metal実機証拠

製品treeを変更しない隔離scratchで、現行`GpuCtx::new_for_ui()`をそのまま使った。

| 項目 | 結果 |
|---|---|
| GPU | Apple M4 / Metal |
| ownership | `GpuOrigin::UiShared`、Motolii core-first |
| preview | 640×360 `Rgba8Unorm` Textureをnative textureとして登録・表示 |
| device | Motoliiとeguiで同一。第2deviceなし |
| CPU pixel bridge | なし。fixture生成時の初回upload後はTextureViewを直接sample |
| lifecycle | 800×700 resize、minimize、400 ms後restore、960×640復帰に成功 |
| health | 92 frame後も`GpuCtx::check_health()`成功 |
| UI update CPU | 24 asset行、50 slider、preview、簡易timelineでp50 0.597 ms / p95 0.934 ms |
| idle | repaint要求停止後600 ms待機。close処理以外の連続frameなし |
| screenshot | lifecycle前後とも1920×1280 Retina imageを取得しpreview一致 |
| dependency | wgpu 29.0.4 / winit 0.30.13の重複なし |

CPU値は`App::ui`内の計測で、GPU present時間、OS input latency、accessibility処理を含まない。G0-4の製品性能値の代わりにしない。

## 5. 日本語IME実機証拠

egui既定fontは日本語glyphを含まず豆腐表示になった。macOSの日本語fontを`FontDefinitions`へfallback登録すると描画は成功した。したがって製品では、再配布可能なCJK font同梱またはOS別system font resolverを必須とし、偶然のOS fallbackへ依存しない。具体font、subset、license、binary sizeはG0-6で決める。

同じApple M4 / Metalと共有deviceで、egui 0.35 `TextEdit`の単一行・複数行を手動確認した。

| 項目 | 結果 |
|---|---|
| Preedit | 37 event。変換中文字列と`active_range_chars`を取得 |
| Commit | 5 event。「運行中」「社会」等を確定 |
| cancel | 空Preeditによる取消/終了経路を取得 |
| multiline | 確定後のEnterと改行を確認 |
| candidate | caret付近の候補表示を人間確認 |
| shortcut isolation | 変換中のEnter / Esc / Space漏れ0 |
| GPU | errorなし、正常終了 |

`egui-winit`はmacOS固有のPreedit処理を持つが、実装の存在だけで合格とせず上記の製品経路で確認した。Windows MS-IMEの事前実機確認は採用停止線にせず、最初のWindows CI/配布候補での運用確認へ送る。

## 6. 維持する境界

toolkit変更で次を変えない。

- pixelはVRAMに置いたまま処理する
- preview/exportは同じrender関数を使い、差は`Quality`
- UI threadでMotolii frameをrenderせず、最新値mailboxとgeneration破棄を使う
- UI状態はDocument / User settings / Workspace-session候補 / Transientへ分類する
- 永続編集はD2 commandと単一writerだけを通る
- timeline layout/hit-test/render modelはtoolkit非依存とし、大量widgetではなく単一wgpu面を使う
- panel layoutの利用者設定をDocument、egui memory、`egui_tiles::Tree`の生serializeへ焼かない

## 7. 移行停止線（歴史）

U0a完了前に行わなかったもの（本入場でU0a相当は完了）:

- ~~workspaceのSlint依存削除とegui依存追加~~ → **U0a完了**
- ~~`UiDeviceParts`、Slint固有コメント、依存方向テスト名の変更~~ → **U0a完了**（`UiDeviceParts`名・公開形は不変）
- ~~`motolii-ui`製品shell、静止preview、組み込みpanel layout、render worker~~ → **U1a-1/2・U1b-1/2完了**。追加のtoolkit固有panel/TimelineはG0-9待ち
- 公開API、Document schema、plugin ABI、永続設定形式の追加 → **各タスク依存**

U0aで完了した項目:

1. G0-1を本判断と実機証拠へ差し替え
2. Slint固有の依存方向CIを「UI toolkitは`motolii-ui`だけ」へ一般化
3. egui骨格でのリンク確認（当時は窓なし。その後device共有・native textureはU1a-1で成立）

## 8. 歴史資料の扱い

- [S1 Slintスパイク](../spikes/s1-slint.md)は当時の採否証拠として変更・削除しない
- [GPU preview先例調査](2026-07-18-m3-gpu-preview-viewport-prior-art.md)のSlint結論は本判断で置換されたが、ownership、lifecycle、display pool、負例は継承する
- `.slint`実行時ロードを将来候補にした過去文書は歴史記録とする。現在のv1/v1.x active roadmapへplugin所有toolkit codeを戻さない
