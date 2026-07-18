# M3 UI基盤 egui採用判断（2026-07-18）

ステータス: **採否決定 / 文書反映のみ**。M3のUI基盤をSlintからeguiへ変更する。これは[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)中の製品実装許可ではない。現行`motolii-ui`骨格、workspace依存、Slint固有コメントとテスト名の移行は、ゲート解除後のM3入場PRで行う。

## 1. 決定

- 製品UIはRust nativeの**egui**を使う。初期統合の検証版は`egui` / `eframe` / `egui-wgpu` / `egui-winit` 0.35、`egui_tiles` 0.16、wgpu 29を使った
- `motolii-gpu`がwgpu device/queueを所有し、UI shellは`egui_wgpu::WgpuSetup::Existing`で借りる。第2deviceやCPU pixel bridgeを正規経路にしない
- previewは同一device上の`Rgba8Unorm` `TextureView`を`egui_wgpu::Renderer::register_native_texture`で登録して表示する。安定したdisplay poolのviewはpool生成時に一度登録し、毎frame sampler/bind groupを作らない
- toolkit依存は`motolii-ui`内へ閉じ、domain intent、Document command、render/eval/plugin公開APIへegui/eframe/winit型を出さない
- 可変panelは`egui_tiles`をruntime投影先の第一候補とする。`Tree`、`TileId`、crateのserde形を製品設定の正本にせず、Motolii所有の安定したlayout modelから投影する。利用者はpanelの分割、tab化、resize、表示/非表示、復帰を選べる
- v1 plugin UIは従来どおり`NodeDesc`からHostが自動生成するpanelだけを公開境界とする。plugin所有のegui code、native widget、自由wgpu UIは公開しない

## 2. 変更理由

Slint S1は2026-07-11時点のApple M4 / MetalでManual device共有とtexture importを実証したため、失敗ではない。一方、M3着手前の再調査で次を確認した。

1. Slintのwgpu接合は`unstable-wgpu-29`とrenderer featureの組合せへ依存し、minor更新時の変更可能性がある
2. egui-wgpu 0.35は既存の`Instance / Adapter / Device / Queue`を渡す[`WgpuSetup::Existing`](https://docs.rs/egui-wgpu/0.35.0/egui_wgpu/enum.WgpuSetup.html)を通常の公開APIとして持つ
3. 同じrendererは既存`TextureView`を[`register_native_texture`](https://docs.rs/egui-wgpu/0.35.0/egui_wgpu/struct.Renderer.html#method.register_native_texture)でoffscreen imageとして表示できる
4. editor型の高密度UI、Host自動生成parameter panel、Rust/LLMによるcomponent単位の変更、利用者が組み替えられるpanel構成はimmediate modeと相性がよい
5. 現行`motolii-ui`はSlint接続確認用の空骨格で、製品画面や`.slint` componentは未実装である。toolkitを変える費用が最小の時点である

## 3. Apple M4 / Metal実機証拠

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

## 4. 日本語IME実機証拠

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

## 5. 維持する境界

toolkit変更で次を変えない。

- pixelはVRAMに置いたまま処理する
- preview/exportは同じrender関数を使い、差は`Quality`
- UI threadでMotolii frameをrenderせず、最新値mailboxとgeneration破棄を使う
- UI状態はDocument / User settings / Workspace-session候補 / Transientへ分類する
- 永続編集はD2 commandと単一writerだけを通る
- timeline layout/hit-test/render modelはtoolkit非依存とし、大量widgetではなく単一wgpu面を使う
- panel layoutの利用者設定をDocument、egui memory、`egui_tiles::Tree`の生serializeへ焼かない

## 6. 移行停止線

M3入場PRまで行わないもの:

- workspaceのSlint依存削除とegui依存追加
- `UiDeviceParts`、Slint固有コメント、依存方向テスト名の変更
- `motolii-ui`製品shell、panel、preview、timeline実装
- 公開API、Document schema、plugin ABI、永続設定形式の追加

M3入場PRでは次を同時に再翻訳する。

1. G0-1を本判断と実機証拠へ差し替える
2. Slint固有の依存方向CIを「UI toolkitは`motolii-ui`だけ」へ一般化する
3. core-first既存device方式でWindow Surface互換adapterを確認する。失敗時だけ、surface-compatible deviceをshellが生成して`GpuCtx::from_device_queue()`へ渡す代替を仕様改訂する
4. `egui_tiles`はruntime projectionに限定し、安定layout modelの所有層と保存寿命を先に決める
5. 0.35で確認した`App::update`→`App::ui`等のAPI churnを製品全体へ漏らさないadapter testを置く

## 7. 歴史資料の扱い

- [S1 Slintスパイク](../spikes/s1-slint.md)は当時の採否証拠として変更・削除しない
- [GPU preview先例調査](2026-07-18-m3-gpu-preview-viewport-prior-art.md)のSlint結論は本判断で置換されたが、ownership、lifecycle、display pool、負例は継承する
- `.slint`実行時ロードを将来候補にした過去文書は歴史記録とする。現在のv1/v1.x active roadmapへplugin所有toolkit codeを戻さない
