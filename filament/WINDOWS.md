# ガラスを Windows の Skin に出す経路

2026-10-12 に書いた。**Windows では何もビルドしていないし、動かしてもいない。** ここにあるのは、ヘッダと公式ソースを読んで決めた道順と、実機で最初に確かめる項目。

Mac で動いている形(`glass_live.h`、`build_live.sh`、`skin/book/lib/sets/glass_live.dart`)と同じ絵、同じ操作、コピーなしを Windows で通すのが目的。

## ひとことで

- Mac: Filament(Metal)が共有バッファ(CVPixelBuffer)に直接描く。Flutter が同じバッファを表示する。
- Windows: Filament(Vulkan)が **D3D11 の共有テクスチャ**に直接描く。Flutter が同じテクスチャを表示する。

形は同じ。違うのは「共有バッファの種類」と「描き終わりの知らせ方」の2点だけ。

## なぜこの形か

- Filament の Windows 版に D3D はない。Vulkan か OpenGL。
- Flutter の Windows 版は D3D11(ANGLE)で描く。外から受け取れる GPU の絵は「DXGI 共有ハンドル」か「ID3D11Texture2D」の2種類だけ。
- なので橋は「Vulkan から D3D11 の共有テクスチャを開く」になる。これは Vulkan の標準拡張(`VK_KHR_external_memory_win32`)。
- Flutter のテクスチャ受け渡しは OS ごとに別の API なので、ここだけは OS 別のつなぎが避けられない。ガラスの絵を作る部分は共通のまま。

## 読んで確かめたこと

| 事実 | どこで |
|---|---|
| スワップチェーンを自前の絵に差し替える口が公式にある。`createSwapChain` / `getSwapChainBundle` / `acquire` / `present` / `destroy` が全部 `virtual` | `third_party/filament/include/backend/platforms/VulkanPlatform.h` 195〜280行 |
| 拡張を足してデバイスを作る口がある(`createVkDevice` が `virtual`)。Filament がデバイスを持ったままでよい | 同 534行 |
| Windows 用の土台クラス `VulkanPlatformWindows` がある | `VulkanPlatformWindows.h` |
| `Engine::createSwapChain(窓, flags)` に渡したポインタは、そのまま自前の `createSwapChain` に届く。幅と高さだけの版(ヘッドレス)だと `present` が呼ばれない | Filament v1.77.2 `VulkanSwapChain.cpp` |
| `present(chain, index, finishedDrawing)` に「描き終わったら立つセマフォ」が渡される | 同上、`VulkanPlatform.h` 218行 |
| 表示用のレイアウト変更は切れる(`Customization::transitionSwapChainImageLayoutForPresent = false`) | `VulkanPlatform.h` 165行 |
| **Mac で使った `setFrameCompletedCallback` は Vulkan では中身が空** | Filament v1.77.2 `VulkanDriver.cpp` |
| Vulkan では `Texture::Builder::import` は使えない(アサートで落ちる) | 同上 |
| Flutter Windows: `GpuSurfaceTexture(kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle, コールバック)`、形式は `BGRA8888` か `RGBA8888`、`RegisterTexture` / `MarkTextureFrameAvailable` | Flutter `flutter_texture_registrar.h`、`client_wrapper/include/flutter/texture_registrar.h` |

先例: Thermion(Flutter + Filament、Apache-2.0)が Windows で同じ橋を渡している。ただしあちらは「Filament の外部画像に描く → GPU 内で1回コピー → D3D11 共有テクスチャ」で、コピーが1回入る。メモは `../research_notes/filament_integrations.md` の3章。

## 作るもの

### 1. Filament の Windows 版と材質

- 公式の配布物 `filament-v1.77.2-windows.tgz` を `third_party/filament` と同じ並びで置く。Vulkan が入っているかは未確認(手順の最初に確かめる)。
- 材質(`.mat` 64枚)を Vulkan 向けに作り直す。`matc -a vulkan -p desktop`。中身は書き換えない。

### 2. 絵を描く本体の持ち運び

OS で違う部分は `glass_door.h` の関数4つに切り出してある(2026-10-12、Mac で確認済み: 切り出しの前後で止め絵が1バイトも変わらず、60 fps)。

| 関数 | 役目 | Mac(`glass_door_mac.mm`、44行) |
|---|---|---|
| `glass_door_engine()` | その OS の方式でエンジンを作る | Metal |
| `glass_door_open(engine, w, h, finished)` | バッファを3枚作る。描き終わるたびに `finished(番号)` を呼ぶ。描画スレッドを時間どおり起こす設定もここ | CVPixelBuffer 3枚、スワップチェーン3本、Filament の描き終わり通知、QoS |
| `glass_door_chain(番号)` | その1枚に描くために `beginFrame` に渡すもの | その1枚のスワップチェーン |
| `glass_door_hand(番号)` | ホストに渡す形にする | 保持した CVPixelBuffer |

**Windows でやることは、この4つを `glass_door_win.cpp` に書くことだけ。** `glass_scene.mm` と `glass_live.mm` は Objective-C を使っていない(`clang++ -x c++ -fsyntax-only -DGLASS_LIB` でエラー 0 を確認)。拡張子は `.mm` のままなので、Windows では `-x c++` を付けてコンパイルする。

残る Apple 寄りの書き方(どれも1行ずつ):

- `glass_live.mm` の `setenv` → Windows は `_putenv_s`。
- `glass_live.mm` の `__attribute__((visibility("default")))` → `__declspec(dllexport)`。
- `glass_scene.mm` 1016行の `(const int[]){0, 0}` という書き方(GNU 拡張。clang-cl なら通るはず、MSVC は通らない)。
- `std::system("mkdir -p …")`(ライブラリ版では通らない道)。

コンパイラは clang-cl を第一候補にする(今のコードは clang でしか通していない)。

### 3. 扉(新しく書くのはここだけ)

`VulkanPlatformWindows` を継いだクラス1つ。見積もり 200行前後。

- `createVkDevice`: Filament が用意した設定に `VK_KHR_external_memory_win32` を足してからデバイスを作る。
- `getCustomization`: 表示用のレイアウト変更を切る。
- `createSwapChain`: D3D11 で BGRA8 の共有テクスチャを3枚作る。同じ3枚を Vulkan の絵として開く。深度の絵も1枚作る。
- `getSwapChainBundle`: その3枚と深度、形式、大きさを返す。
- `acquire`: Flutter が今見せていない1枚の番号を返す。
- `present`: 渡されたセマフォを待つ空の提出を1回出して、フェンスを付ける。別スレッドがフェンスを待って、立ったらその1枚を「表」にして `frames` を1つ進める。
- `hasResized` は常に偽。`destroy` で片付け。

Mac との違いが1つある。Mac はスワップチェーンを3本持った。Windows は1本で絵が3枚。こっちのほうが Vulkan の普通の形。`glass_door_chain(番号)` は「次の `acquire` はこの番号」と覚えてから、その1本を返せばよい。

4つの関数との対応: `glass_door_engine` がこのクラスを `Engine::Builder().backend(VULKAN).platform(...)` に渡す。`glass_door_open` は `engine->createSwapChain(自前の構造体, 0)` を1回呼ぶ。`present` の先のスレッドが `finished(番号)` を呼ぶ。`glass_door_hand` が共有ハンドルを返す。

`present` は Filament の描画スレッドから呼ばれる。そこで提出するので、Filament とキューを取り合わない。Thermion が入れていた「キューに鍵をかける細工」は要らないはず(未確認)。

### 4. Skin 側

- `skin/book` に Windows の入れ物がまだ無い。`flutter create --platforms=windows .` で作る。
- `windows/runner/flutter_window.cpp` に、Mac の `GlassTexture`(`macos/Runner/MainFlutterWindow.swift` の末尾、約70行)と同じものを C++ で書く。
  - チャンネル `motolii/glass` の `start {lib, base, width, height, env}`。`LoadLibrary` で DLL を開いて `glass_env` / `glass_start` を呼ぶ。
  - `GpuSurfaceTexture` を登録。コールバックで `glass_front()` のハンドルを `FlutterDesktopGpuSurfaceDescriptor` に入れて返す。
  - `glass_frames()` が進んだら `MarkTextureFrameAvailable`。
- `glass_live.dart`: ライブラリ名を `libglass_live.dylib` と `glass_live.dll` で切り替える。`GLASS_DIR` の既定値を OS で分ける。ほかはそのまま(ドラッグ、ホイール、つまみ、読み出し)。
- Windows には Mac のようなサンドボックスの設定は無い。

## 実機で最初に確かめる順

小さく切って、1つずつ絵で確かめる。

1. **Filament が Vulkan で立つか。** 配布物に Vulkan が入っているか。`glass_scene` の単体版を Vulkan でビルドして、PNG を1枚出す。Mac の同じ設定の絵と見比べる(1バイト一致は期待しない。GPU が違う)。
2. **速さ。** 単体版の `PACE=1 PACE_HZ=2000 PACE_FRAMES=2500` で1コマ何 ms か。Mac(M4)は 1080p で約 5〜9 ms だった。
3. **扉だけ。** 三角形1枚を D3D11 の共有テクスチャに描いて、D3D11 側で読んで色を確かめる。Flutter はまだ使わない。
4. **Flutter に出す。** 止まった絵 → 動く絵 → 60 fps。
5. **操作。** ドラッグ、ホイール、つまみ、ページを離れて戻る。

## 分かっていない点(実機で決める)

- **共有ハンドルの種類。** D3D11 の共有ハンドルには古い型と新しい型(NT ハンドル)がある。Vulkan 側の指定(`D3D11_TEXTURE_KMT_BIT` か `D3D11_TEXTURE_BIT`)と、Flutter が開ける型が一致する組を探す。Thermion の `flutter_d3d_texture.cpp` と `WindowsVulkanTexture.cpp` が答えを持っている。
- **GPU が2つあるノート PC。** Vulkan、D3D11、Flutter が同じ GPU に乗らないと共有できない。Vulkan のデバイスの LUID で D3D11 のアダプタを選ぶ(Thermion と同じ)。Flutter 側が別の GPU を選んだ場合の扱いは未定。
- **描き終わりの待ち方。** 上の「セマフォ → フェンス」が本線。だめなら Filament の `Fence` を使う。どちらも Mac の通知より一手多い。
- **上下と色。** 絵が逆さになる、赤と青が入れ替わる、明るさがずれる、のどれかは出ると思っておく。3番の三角形で先に潰す。
- **60 Hz の刻み。** Mac ではスレッドの優先度が低いと 45 fps まで落ちた。Windows の同じ罠はタイマーの粗さ(既定で約15 ms)。
- **止め方。** Mac は `Engine::destroy` が落ちるので「止める = 休む」にした。Windows も同じにする。
- **アプリとして配る形。** DLL と材質ファイルを今は絶対パスで読んでいる。見本の窓としてはそのままでよい。

## 退路

扉(3)がどうしても通らないとき: Thermion と同じ「外部画像に描く → GPU 内で1回コピー → 共有テクスチャ」。Filament は無改造のまま、動いた実績がある。コピーは GPU の中で1回、CPU には戻らない。

## これは決定ではない

- これは「Skin の窓にガラスを出す」ための扉。マスクや ThorVG と GPU を分け合う話(`motolii/melt`、`motolii/bridge`)とは別で、あちらの方式はまだ選んでいない。
- Filament を Vulkan で動かすこと自体は、どの方式を選んでも Windows で必要になる。手順の1と2はそのまま使える。
