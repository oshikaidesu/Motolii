# Flutter の皮とネイティブの中身：よそはどうつないでいるか（調査 2026-10-14）

調べ方：Web の検索と、リポジトリの実ファイルの読み取りだけ。コードは書いていない。何も入れていない。時間は約 30 分で打ち切った。
「読んだ」と書いたものは末尾の URL を実際に開いた。「覚え」と書いたものは今回は開いていない。

## 6 行のまとめ（悪い知らせから）

1. 悪い知らせ：Flutter 3.47 から Impeller が macOS / Windows / Linux で既定になった（docs.flutter.dev で読んだ）。Windows の `DxgiSharedHandle` の道が Impeller の下でそのまま動くかは、一次資料で確かめられなかった。
2. 悪い知らせ：そのまま使える出来合いの部品は無い。`irondash_texture` は Rust 用。`flutter_wgpu` は wgpu 用で Windows は CPU コピー。C++ の中身には、Runner に小さな橋を自分で置くことになる（Lumit の macOS の橋は Swift 1 ファイル）。
3. 悪い知らせ：操作のたびに「文書まるごとの状態」を返す製品は、調べた中に一つも無かった。どこも「何が変わったか」だけを返す。
4. 良い知らせ：macOS の絵の渡し方は、Lumit・thermion・irondash の 3 つが同じ形だった。`FlutterTexture` の `copyPixelBuffer` で IOSurface 付きの `CVPixelBuffer` を返す。Flutter 側は `CVMetalTextureCacheCreateTextureFromImage` で包むだけで、CPU コピーは無い（エンジンの原典で読んだ）。
5. 良い知らせ：Windows も Lumit と RustDesk が同じ形だった。`flutter::GpuSurfaceTexture` + `kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle` + BGRA。落とし穴は Lumit の注釈に書いてある（RGBA は黙って映らない、登録解除は非同期）。
6. 口（くち）は 2 つの型に分かれる。「型つきの直接呼び出し + 変更の流れ」（Lumit、thermion、RustDesk）と「戸口ひとつ + 直列化した伝言」（AppFlowy、Anthem、Rinf）。うちに一番近い双子は thermion（C++ + C API + `dart:ffi`）。

## 言葉の一覧（広く探す前に集めたもの）

英語
- Flutter external texture / Texture widget / texture registry / TextureRegistrar
- FlutterTexture / copyPixelBuffer / textureFrameAvailable / onTextureUnregistered
- CVPixelBuffer / IOSurface / CVPixelBufferCreateWithIOSurface / IOSurfaceLookup / CVMetalTextureCache
- FlutterDesktopGpuSurfaceDescriptor / GpuSurfaceTexture / PixelBufferTexture / DXGI shared handle / ID3D11Texture2D / ANGLE share handle / MarkTextureFrameAvailable
- zero-copy texture / shared texture / GPU texture renderer / DMA-BUF
- Impeller external texture / Flutter GPU (flutter_gpu) / PlatformView vs Texture
- Flutter desktop native renderer / Flutter embed engine / headless core + Flutter UI / Flutter Rust core
- dart:ffi / ffigen / NativeCallable.listener / Dart_PostCObject / ReceivePort / native assets (build hook)
- flutter_rust_bridge (frb) / StreamSink / rinf signals / event dispatch / protobuf FFI / change stream / scoped change
- frame scheduler / CVDisplayLink / vsync / render thread / staged resize handoff
- drag preview / provisional value / commit on release / latest-wins throttle / coalesce

日本語
- 外部テクスチャ / テクスチャ登録 / ゼロコピー / 共有テクスチャ / ネイティブ描画 / FFI ブリッジ / 変更通知 / ドラッグ中のプレビュー

中国語
- 外接纹理 / 零拷贝 / 共享纹理 / Flutter 桌面 Impeller

## 製品の表

「生き」の列：日付が読めたものだけ書いた。読めなかったものは「日付は読めず」と書いた。

| 製品 | 何か / 生き | 中身の言語 | 命令の入れ方（しくみ・粒・誰が文書と選択と再生位置を持つか） | 変更の戻り方 | 取り消しとドラッグ中の仮の値 | 絵：macOS | 絵：Windows | 誰が時計を持つか / 大きさ変え | ライセンス |
|---|---|---|---|---|---|---|---|---|---|
| **Lumit** | モーショングラフィックスと合成の編集ソフト。early-access。main に 2,771 コミット。日付は読めず | Rust（wgpu） | `flutter_rust_bridge` の生成コード。1 操作 = 1 呼び出し = `lumit_core` の 1 op。値は差分でなく「丸ごとの値」。Dart は文書を持たない。ハンドル（`ProjectReference`、`CompositionReference`、`LayerReference`、`ItemReference`）だけ持つ。再生位置・ズーム・選択は皮が持つ。時計・待ち行列・やり直し・値の切り詰めは中身が持つ。時間は `{num, den}` かフレーム番号で渡す | `ScopedChange` の流れ。どのハンドルに触れたかを名指しする。皮はそこだけ描き直す。進み具合・スコープ・計測は別の「フレーム応答の流れ」に乗る | 取り消し = 逆を持つ命令の日誌。適用のたびに不変のスナップショット（今は `Document` 丸ごと複製、`arc-swap` で差し替え）。ドラッグ = 確定しない。`render_frame_with_preview` が「継ぎを当てた複製」を描く。離したときだけ確定。皮の `PreviewThrottle` は 20 ms 間隔で最新勝ち。中身も `drain_to_newest` で 1 枚に絞る。ドラッグ中は 640x360 相当の小さい絵（`drag_tier`） | Runner の `ViewerTextureBridge.swift`。中身が IOSurface を作り、`IOSurfaceID`（整数）を Dart 経由で渡す。`IOSurfaceLookup` → `CVPixelBufferCreateWithIOSurface` → `FlutterTexture.copyPixelBuffer` で `passRetained`。チャンネル名 `lumit/viewer_texture`、メソッドは `register` / `frameReady` / `unregister` | `viewer_texture_bridge.cpp`。D3D12 の共有テクスチャの NT ハンドル（64 ビット）を渡す。`flutter::GpuSurfaceTexture(kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle, …)`、`kFlutterDesktopPixelFormatBGRA8888`。橋はハンドルを自分で開かない。Flutter が自分の D3D11/ANGLE で開く | 再生は中身の作業スレッドが先回りして描き、期日に見せる（「格子であってストップウォッチではない」）。再生中は音の時計が主。絵が届くたびに Dart が `frameReady` を呼び、橋が `textureFrameAvailable` を呼ぶ。Dart の再生位置は届いた絵で錨を打ち直す（`clockFrame`）。大きさ変え = 登録し直し（`ensureRegistered`）。再生位置を動かすと Viewer の epoch が進み、古い仕事は `Err(Cancelled)` | GPLv3。コードは写せない。やり方は写せる |
| **thermion** | Filament を Dart / Flutter で使う 3D 道具。Filament v1.76.0 を使用。リリース日は読めず | C++（Filament）+ C API | `dart:ffi` の生成バインディング。Filament に触る呼び出しは全部 `*_RenderThread` の shim を通り、描画スレッドの `std::packaged_task` に積まれる。Dart は completer で戻りを待つ。文書という考えは無い（場面は中身） | 呼び出しごとのコールバックだけ。変更の流れは無い | 無い（編集ソフトではない） | `MetalTextureWrapper.swift`。`CVPixelBufferCreate`（`kCVPixelFormatType_32BGRA`、`kCVPixelBufferIOSurfacePropertiesKey: [:]`、`kCVPixelBufferMetalCompatibilityKey`）→ `CVMetalTextureCacheCreateTextureFromImage` → `MTLTexture` のポインタを整数で Dart へ → Filament の Metal バックエンドが `importedTextureHandle` として描く。`FlutterTexture.copyPixelBuffer` は同じ `CVPixelBuffer` を返す | Filament は Vulkan。`Texture` ウィジェットで合成。D3D のハンドルを大きさ変えで差し替える（文書は TODO が多く、API 名までは読めず） | 画面の同期信号が主。`CVDisplayLinkScheduler`（macOS）/ `DXGIFrameScheduler`（Windows）→ `ffi.NativeCallable`（release）か `Dart_PostCObject_DL`（debug）で Dart の `_onFrame` → `render()` → 描画スレッド。描いている最中の遅れた tick は捨てる。`setTargetFramerate` で間引く。大きさ変え（macOS）= 段取りつきの交代：新しいテクスチャを下に置く → 1 枚描く → 見せる → 古いのを外す。Windows = テクスチャ id はそのまま、中のハンドルを差し替え | Apache-2.0。コードも写せる |
| **Rive（rive_native）** | Rive の C++ ランタイムと Rive Renderer の Flutter 版。0.1.11 は「2 か月前」 | C++ | FFI の直接呼び出し。文書は中身（Rive File） | 読めず | 無い | Rive Renderer はテクスチャ（`RenderTexture`）に描く。中の API 名は読めず。changelog：0.1.7 で `textureFrameAvailable` を platform スレッドに限った（終了時の落ち対策） | 同じく名は読めず | 大きさ変え：古いテクスチャを伸ばして使い回し、新しいのは間引いて作り、「描けてから」差し替える（`textureResizeInterval`）。`useSharedTexture` は実験中 | MIT |
| **RustDesk** | リモートデスクトップ。出荷済みで広く使われている（覚え）。日付は読めず | Rust | `flutter_rust_bridge` の関数（`session_*`、`platformFFI.*`）。セッションは中身が持つ | 中身 → 皮は `StreamSink<EventToUI>`。`push_event` が「name + 値」の JSON を流す。全体向けは `GLOBAL_EVENT_STREAM` | 無い | この道は RGBA のバッファ（`texture_rgba_renderer`、`FlutterRgbaRendererPluginOnRgba`）。GPU の道は macOS には無い（`flutter.rs` で読んだ） | `flutter_gpu_texture_renderer`。`flutter::GpuSurfaceTexture` + `kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle`。アプリの `ID3D11Texture2D` を `SetTexture` で受け、同じデバイスに共有用の板を作り `GetSharedHandle`、毎回 `CopyResource` + `Flush`（GPU 内のコピー 1 回）、`MarkTextureFrameAvailable`。アダプタは `view->GetGraphicsAdapter()` の LUID を中身に教える | 絵を作る側（復号）が押す。Flutter は合成のときに引く。大きさが変わったら共有の板を作り直す | 本体は AGPL-3.0（覚え）。プラグインのライセンスは読めず。やり方だけ |
| **AppFlowy** | Notion 風の文書ソフト。出荷済み（覚え）。日付は読めず | Rust | 戸口はひとつ：`ffi.async_event(port, bytes, len)`。中身は protobuf の `FFIRequest`（event 名 + payload）。返事は native port に `FFIResponse`（`Ok` / `Err` / `Internal`）。文書は中身が持つ | 別の port。`RustStreamReceiver` が `RawReceivePort` で受け、`SubscribeObject`（protobuf）にほどいて broadcast する。皮は id で聞く | 読めず | 絵は無い | 絵は無い | 無い | AGPL-3.0（覚え）。やり方だけ |
| **Anthem** | DAW。開発中。「production build はまだ無い」。1,292 コミット | C++（JUCE） | 皮と中身は別プロセス。ローカルの TCP ソケットに JSON。伝言は Dart のクラスで書き、C++ の相方を生成する。要求に id、返事は同じ id。**文書（プロジェクトモデル）は皮（Dart）が持つ。** 中身の写しは生成コードで自動で追う | 返事と、頼まれていない伝言 | 読めず | 絵は無い（音） | 同じ | 無い | GPL-3.0。やり方だけ |
| **Rinf** | Rust + Flutter の枠組み。日付は読めず | Rust | 呼び出しでなく伝言（signal）。型は Rust の derive から Dart を生成。FFI だけで運ぶ | `rustSignalStream` を `StreamBuilder` で聞く | 無い | 無い | 無い | 無い | MIT |
| **irondash_texture** | Flutter の外部テクスチャを Rust から扱う部品。日付は読めず | Rust | 無い（部品）。`Texture::new_with_provider`、`PayloadProvider::get_payload`、`mark_frame_available`、`into_sendable_texture` | 無い | 無い | `BoxedIOSurface`。中で `registerTexture:` → `copyPixelBuffer` のとき `CVPixelBufferCreateWithIOSurface`。`mark_frame_available` が `textureFrameAvailable:` を呼ぶ | `BoxedTextureDescriptor<ID3D11Texture2D>` と `BoxedTextureDescriptor<DxgiSharedHandle>` | 作る側が `mark_frame_available`。Flutter が raster スレッドで引く。`Texture` は platform スレッドで作る | ページでは読めず（覚えでは MIT / Apache-2.0） |
| **media_kit** | 動画プレーヤー（libmpv）。日付は読めず | C（libmpv）+ C++ | FFI + テクスチャ登録 | プレーヤーの流れ | 無い | README は TODO。読めず | ANGLE。図に `ANGLESurfaceManager`、D3D11 デバイス、D3D11 テクスチャ、共有 `HANDLE`。libmpv の OpenGL 描画 API を ANGLE の上で使う。ソフト描画はピクセルバッファ | 作る側（`NotifyRender` / `Render`） | MIT |
| **fvp** | `video_player` の差し替え（libmdk）。日付は読めず | C++ | Dart がプレーヤーのポインタを platform channel で渡す | — | 無い | 「metal for macOS/iOS」（README）。細部は読めず | 「d3d11 for windows」（README）。細部は読めず | 読めず | 包みは BSD-3-Clause。libmdk 本体は別（覚え） |
| **flutter_wgpu（gpux）** | wgpu の絵を Flutter に出す部品。0.1.0 は「6 か月前」 | Rust（wgpu） | `WgpuTextureController` / `WgpuTextureWidget` | — | 無い | IOSurface（ゼロコピー） | **ピクセルバッファのコピー** | 読めず | MIT |
| Flutter 本体 | — | — | — | — | — | `FlutterTextureRegistry`：`registerTexture:`（platform スレッド、失敗は 0）、`textureFrameAvailable:`、`unregisterTexture:`。`copyPixelBuffer` は raster スレッドで呼ばれる。受ける形式は `kCVPixelFormatType_32BGRA` と 2 面の YUV（`420YpCbCr8BiPlanar` の Video / Full）だけ。`FlutterExternalTexture.mm` は `CVMetalTextureCacheCreateTextureFromImage` で包むだけ | `FlutterDesktopGpuSurfaceDescriptor`{`struct_size`, `handle`, `width`, `height`, `visible_width`, `visible_height`, `format`, `release_callback`, `release_context`}。型は `kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle` と `kFlutterDesktopGpuSurfaceTypeD3d11Texture2D`。登録・解除・`MarkExternalTextureFrameAvailable` はどのスレッドからでも可。解除は非同期。コールバックは描画スレッド | Flutter は「引く」側。時計は持たない | BSD-3（覚え） |

読めなかったもの（表に入れていない）：flutter_unity_widget、maplibre / mapbox、Godot や Bevy を Flutter に入れる試み。覚えでは前の 2 つは PlatformView（OS の view を重ねる）で、デスクトップの `Texture` の道ではない。今回は開いていない。

## くらべる

### くり返し出る型（名前をつける）

- **「作る側が知らせ、Flutter が引く」**：絵を作った側が `textureFrameAvailable` / `MarkTextureFrameAvailable` を呼ぶ。Flutter は次の合成で raster スレッドから取りに来る。Lumit、RustDesk、irondash、media_kit、thermion の全部がこれ。Flutter が自分から時計を出す製品は無い。
- **「絵は別の細い道」**：命令の道と絵の道は別。絵の道は「登録 / できた / 解除」の 3 つだけで、渡すのは整数ひとつ（`IOSurfaceID`、NT ハンドル、`MTLTexture` のポインタ）。Lumit、thermion、RustDesk が同じ。
- **「BGRA の板ひとつ」**：macOS も Windows も BGRA 8 ビット。macOS は API が BGRA と YUV しか受けない。Windows は ANGLE が RGBA を黙って断る（Lumit の注釈）。
- **「大きさ変え = 新しい板、描けてから交代」**：thermion（段取りつき交代）、Rive（描けてから差し替え、作り直しは間引く）、Lumit（登録し直し）、RustDesk（共有の板を作り直し）。板をその場で伸ばす製品は無い。
- **「中身が文書、皮は見ているものだけ」**：Lumit は文書と方針を中身、再生位置・ズーム・選択を皮。AppFlowy も文書は中身。例外は Anthem（皮が文書を持つ）。
- **「何が変わったかだけ返す」**：Lumit の `ScopedChange`、AppFlowy の `SubscribeObject`、RustDesk の名前つきイベント、Rinf の signal。皮はそこだけ読み直す。文書まるごとを返す製品は無い。
- **「1 操作 = 1 呼び出し = 取り消し 1 歩、値は丸ごと」**：Lumit が規則として書いている。
- **「ドラッグは確定しない、離して確定、最新勝ちで間引く」**：Lumit（`render_frame_with_preview`、`PreviewThrottle` 20 ms、`drain_to_newest`、ドラッグ中は小さい絵）。編集ソフトで読めたのは Lumit だけ。
- **「中身に触る呼び出しは描画スレッドへ並べる」**：thermion（`*_RenderThread` + `packaged_task`）。Lumit は「project ごとに `RwLock` ひとつ、短く持つ、知らせは鍵を放してから」。

### 違うところと、その理由

- **時計**。thermion は画面の同期信号（`CVDisplayLink` / DXGI）から中身を叩く。3D の覗き窓で、毎回の画面更新に合わせたいから。Lumit は再生中は音の時計が主で、中身の作業スレッドが先に描いて期日に出す。理由は文書に書いてある：「見せる絵は、鳴らした音の関数」「評価が遅れたら絵を落とす。音は待たない」。止まっているときは編集やスクラブの要求ごとに 1 枚。
- **口の形**。Lumit は型つきの直接呼び出し。理由：生成器がメモリを持つ、パニックは越えさせない、両側の内容ハッシュが違えば起動しない。AppFlowy は戸口ひとつに protobuf。Anthem は別プロセスに JSON で、伝言のクラスを生成する。Anthem の理由：手書きの RPC と手書きの更新呼び出しは「highly error-prone」、とくにファイルを開くときに手に負えない。Rinf の理由：出来事で動く形にすると並行・取り消し・待たない処理がしやすい。
- **誰が文書を持つか**。Anthem だけ皮が持つ。音の中身は「文書から音を作るだけ」だから。絵を描く Lumit は逆で、描くのに文書が要る中身が持つ。
- **Windows の板の作り方**。Lumit は D3D12 の共有テクスチャを中身が作り、ハンドルをそのまま渡す（コピー無し）。RustDesk は復号器の D3D11 テクスチャを共有用の板に `CopyResource`（GPU 内 1 回）。thermion は Vulkan で描いて D3D のハンドルを渡す。
- **macOS の板の作り方**。Lumit と irondash は「中身が IOSurface を作る → `CVPixelBufferCreateWithIOSurface`」。thermion は「Swift が `CVPixelBuffer` を作る → `MTLTexture` にして中身に渡す」。向きが逆なだけで、Flutter から見れば同じ。

### いちばんよく使われ、確かめられているやり方

- (1) macOS：`FlutterTexture` + IOSurface 付き BGRA の `CVPixelBuffer` + `textureFrameAvailable`。3 製品が同じ。Flutter 側の原典でもコピー無し。
- (1) Windows：`flutter::GpuSurfaceTexture` + `kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle` + BGRA + `MarkTextureFrameAvailable`。2 製品が同じ（Lumit、RustDesk）。media_kit も ANGLE + 共有 `HANDLE`。
- (2) 口：入りは型つきの呼び出し（frb か `dart:ffi`）が多数。戻りは「何が変わったか」の流れ。これは直接呼び出し派も伝言派も同じ。

## うちへのすすめ

### (1) 生きた絵

macOS（いちばん小さく、確かめられた形）
- Flutter の API は `FlutterTexture` と `FlutterTextureRegistry`。Dart 側は `Texture(textureId: …)` ウィジェットだけ。
- Runner に橋を 1 ファイル置く。形は Lumit の `ViewerTextureBridge.swift` と同じ：メソッドは「登録（IOSurfaceID、幅、高さ）→ テクスチャ id」「できた」「解除」の 3 つ。中では `IOSurfaceLookup` → `CVPixelBufferCreateWithIOSurface`、`copyPixelBuffer` で `passRetained`。Lumit は GPLv3 なので文面は写さない。同じ形のコードは thermion（Apache-2.0）にあり、こちらは写せる。
- 板は BGRA（`kCVPixelFormatType_32BGRA`）。Flutter が受けるのは BGRA と 2 面 YUV だけ。
- 出来合いのパッケージは使えない。`irondash_texture` は Rust の API。`flutter_wgpu` は wgpu 用。うちは既に CVPixelBuffer の道を持っているので、足すのは「できた」の知らせと大きさ変えの段取りだけ。
- 時計は中身が持つ。理由は Lumit のもの：再生は音の時計が主で、絵は音の関数。皮の vsync は時計にしない。中身が 1 枚描き終えたら「できた」を知らせ、Flutter が引く。止まっているときは、値が変わったときとスクラブのときだけ 1 枚描く。
- 「できた」は platform スレッドから呼ぶ。Rive が 0.1.7 でそう直した（終了時の落ち対策）。
- 大きさ変えは「新しい板を作る → 1 枚描く → 登録を替える → 古い板を外す」。thermion と Rive の順番。ドラッグで大きさが毎フレーム変わると登録の競り合いが 60 回 / 秒起きる（Lumit の注釈）。作り直しは間引く（Rive の `textureResizeInterval`）。
- 再生位置は皮が持ち、絵が届くたびに錨を打ち直す（Lumit の `clockFrame`）。再生位置を動かしたら古い描画は捨てる（Lumit の epoch と `Err(Cancelled)`）。
- 費用：Runner の Swift 1 ファイルと、中身の「できた」の呼び戻し。2 枚の板の入れ替え（描いている板を Flutter に読ませない）は自分で持つ。Lumit がここをどうしているかは読めなかった。

Windows の双子
- `flutter::GpuSurfaceTexture` に `kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle`、`FlutterDesktopGpuSurfaceDescriptor` に NT ハンドル・幅・高さ・`kFlutterDesktopPixelFormatBGRA8888`。知らせは `MarkTextureFrameAvailable`。
- 中身（Dawn の D3D12 か D3D11）が共有テクスチャを作り、ハンドルだけ渡す。橋はハンドルを開かない。Flutter が自分の D3D11 / ANGLE で開く（Lumit）。
- 守ること（Lumit の注釈から）：BGRA にする。RGBA は登録は通るが何も映らない。解除は非同期なので、記述子の置き場は完了コールバックで捨てる。大きさ変えは登録し直し。
- アダプタは Flutter と同じものを使う。RustDesk は `GetGraphicsAdapter()` の LUID を中身に教えている。
- 費用と危なさ：Flutter 3.47 で Impeller が既定になった。この道が Impeller の下で動くかは確かめていない。二次の記事は「ANGLE の上の OpenGL ES」と書いているが、一次資料では読めていない。Windows に着手するとき、最初に最小の絵で確かめること。

### (2) 口

すすめ：**「1 操作 = 1 op = 取り消し 1 歩」は残す。「状態まるごとの JSON を返す」はやめ、「何が変わったか」の流れに替える。入りは C API を `dart:ffi`（`ffigen`）で直接呼ぶ。**

調べた製品が挙げている理由
- 粒は今のままで正しい。Lumit の規則と同じ：1 操作 = 1 呼び出し = 1 op = 取り消し 1 歩。値は差分でなく丸ごと。
- 戻りは、直接呼び出し派（Lumit の `ScopedChange`）も伝言派（AppFlowy の `SubscribeObject`、Rinf の signal）も「変わった所の名指し」。まるごと返す製品は無い。Lumit の理由は「そこだけ描き直すため」。
- 入りの道具は、うちの双子の thermion が C API + `dart:ffi` の生成バインディング。`flutter_rust_bridge` は Rust 用で、C API には使えない。
- 文書と方針は中身、再生位置・ズーム・選択は皮（Lumit）。値の切り詰めは中身がやり、皮のは「合わせているだけ」。
- ドラッグは「仮の値で描く呼び出し」と「離したときの確定」を分ける。仮の値は取り消しに入れない。皮は 20 ms の最新勝ちで間引く。中身も最新 1 枚に絞る（Lumit）。
- 時間は有理数かフレーム番号で渡す。小数の秒は渡さない（Lumit）。

それぞれの費用
- JSON の op を入りに残す場合：戸口ひとつで、記録・再生・試験がしやすい。AppFlowy と Anthem はこの形で、Anthem は手書きの RPC より生成した伝言のほうが間違いが少ないと言う。費用は毎回の直列化と、型が実行時まで確かめられないこと。Anthem と AppFlowy は伝言のクラスを生成して埋めている。20 個の呼び出しなら生成器は要らないが、型の検査は自分で持つ。
- `dart:ffi` の直接呼び出しに移る場合：型はコンパイル時に合う。直列化が無い。費用は、スレッドの決まりを自分で持つこと（thermion は全部を描画スレッドに並べる shim を書いている。Lumit は鍵ひとつ・短く）。中身から皮への知らせには `NativeCallable.listener` か `Dart_PostCObject` の port が要る（thermion は両方、AppFlowy は port）。
- 状態まるごとを返し続ける場合：費用は文書の大きさに比例する。ドラッグは 20 ms ごとに来るので、ここがいちばん先に詰まる。調べた製品に前例が無い。
- 変更の流れに替える場合：皮は「名指しされた所を読み直す」読み取りの呼び出しが要る。皮の作りは少し増える。

まとめると：入りの形（JSON か直接か）は製品が割れている。戻りの形（名指しの流れ）は割れていない。先に替えるのは戻りのほう。

## 実際に読んだ URL

Lumit
- https://github.com/luminalmvm/Lumit （README）
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/docs/17-BRIDGE-CONTRACT.md
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/docs/05-ARCHITECTURE.md
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/flutter_ui/macos/Runner/ViewerTextureBridge.swift
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/flutter_ui/windows/runner/viewer_texture_bridge.cpp
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/flutter_ui/lib/state/playback_loop.dart
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/flutter_ui/lib/state/preview_throttle.dart
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/flutter_ui/lib/state/viewer_view.dart
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/flutter_ui/lib/state/project_changes.dart （変更の流れではなかった。保存ファイル 2 つの差分）
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/crates/lumit-bridge/src/playback.rs
- https://raw.githubusercontent.com/luminalmvm/Lumit/main/crates/lumit-bridge/src/realtime.rs
- 一覧だけ：flutter_ui/lib、flutter_ui/lib/state、flutter_ui/lib/panels、flutter_ui/lib/widgets、flutter_ui/macos/Runner、flutter_ui/windows/runner、docs

thermion
- https://github.com/nmfisher/thermion （README）
- https://raw.githubusercontent.com/nmfisher/thermion/develop/thermion_flutter/thermion_flutter/ARCHITECTURE.md
- https://raw.githubusercontent.com/nmfisher/thermion/develop/thermion_flutter/thermion_flutter/darwin/thermion_flutter/Sources/thermion_flutter/MetalTextureWrapper.swift
- https://raw.githubusercontent.com/nmfisher/thermion/develop/thermion_flutter/thermion_flutter/darwin/thermion_flutter/Sources/thermion_flutter/FlutterMetalTextureWrapper.swift

Flutter 本体
- https://api.flutter.dev/macos-embedder/protocol_flutter_texture-p.html
- https://api.flutter.dev/macos-embedder/protocol_flutter_texture_registry-p.html
- https://raw.githubusercontent.com/flutter/flutter/master/engine/src/flutter/shell/platform/darwin/macos/framework/Source/FlutterExternalTexture.mm
- https://api.flutter.dev/windows-embedder/flutter__texture__registrar_8h_source.html
- https://docs.flutter.dev/perf/impeller

そのほか
- https://github.com/irondash/irondash/tree/main/texture
- https://raw.githubusercontent.com/irondash/irondash/main/texture/src/platform/darwin/mod.rs
- https://github.com/rustdesk/rustdesk/blob/master/flutter/lib/models/desktop_render_texture.dart
- https://raw.githubusercontent.com/rustdesk/rustdesk/master/src/flutter.rs
- https://github.com/rustdesk-org/flutter_gpu_texture_renderer
- https://raw.githubusercontent.com/rustdesk-org/flutter_gpu_texture_renderer/main/windows/flutter_gpu_texture_renderer_plugin.cpp
- https://raw.githubusercontent.com/rustdesk-org/flutter_gpu_texture_renderer/main/windows/d3d11_output.cpp
- https://github.com/AppFlowy-IO/AppFlowy/blob/main/frontend/appflowy_flutter/packages/appflowy_backend/lib/dispatch/dispatch.dart
- https://raw.githubusercontent.com/AppFlowy-IO/AppFlowy/main/frontend/appflowy_flutter/packages/appflowy_backend/lib/rust_stream.dart
- https://github.com/SecondFlight/anthem （fork のページに着いた。README）
- https://github.com/anthem-audio/anthem/tree/main/docs
- https://raw.githubusercontent.com/anthem-audio/anthem/main/docs/architecture/state_synchronization.md
- https://raw.githubusercontent.com/anthem-audio/anthem/main/docs/architecture/communication_between_ui_and_engine.md
- https://github.com/cunarist/rinf
- https://github.com/media-kit/media-kit （README の先頭 10 万字）
- https://github.com/wang-bin/fvp
- https://pub.dev/packages/rive_native
- https://rive.app/docs/runtimes/flutter/rive-native.md
- https://pub.dev/packages/flutter_wgpu

検索結果の要約だけ（ページは開いていない）
- rive_native の changelog（0.1.7、0.1.9 の記述）
- flutter/engine PR 26840（`GpuSurfaceTexture` の出どころ、ANGLE の share handle 拡張）
- Flutter 3.47 の Impeller 既定化を伝える二次の記事（startdebugging.net、juejin.cn）
- pub.dev の flutter_rust_3d、pixel_surface、minigpu_view、video_forge

## 読めなかったもの

- Lumit：`ScopedChange` の Dart 側と Rust 側の定義。`frameReady` を呼んでいる Dart のファイル（`ViewerTextureController`）。IOSurface を作る Rust 側。板を何枚持って入れ替えているか。最後のコミットの日付。
- thermion：Windows の API 名（文書が TODO）。最新リリースの日付。
- Rive：Rive Renderer が Flutter に絵を渡す中身のコード。
- media_kit：macOS の実装（パスが 404）。fvp：実装の細部。
- AppFlowy：Rust 側（`set_stream_port`、通知の出し方）。文書サイトのページは 404。取り消しのしくみ。
- RustDesk：`texture_rgba_renderer` の中身。ライセンスの文面。
- flutter_rust_bridge のテクスチャについての案内（当てた URL は 404。覚えでは frb 自体はテクスチャを持たず、irondash を使う人が多い）。
- Rinf の公式サイト（証明書の誤りで開けず）。README だけ読んだ。
- Flutter の Windows 埋め込みが Impeller の下で外部テクスチャをどう扱うか（`external_texture_d3d` の今の版）。flutter_gpu。
- flutter_unity_widget、maplibre / mapbox、Godot / Bevy を Flutter に入れる試み。Flutter の皮を持つ映像編集ソフトは Lumit のほかに見つからなかった（探したのは 1 回の検索だけ）。
- 各製品の最終リリース日は、ほとんど読めなかった。
