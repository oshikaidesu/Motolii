# 参考リポジトリ・ライブラリ一覧

作成日: 2026-07-07(Web検索で所在・ライセンスを確認済み)

用途別に「コードを流用/依存してよいもの」と「設計参考のみ(ライセンス・状況的にコード流用不可)」を明確に区別する。**GPL系はコードを読んで書き写すだけでも汚染リスクがあるため、設計・データ構造の考え方の参考に留める。**

## コード流用・依存候補(ライセンス上安全)

| リポジトリ | ライセンス | 何を参考/利用するか |
|---|---|---|
| [OpenCut](https://github.com/OpenCut-app/OpenCut) | MIT | タイムラインUIの**操作仕様の参考のみ。コード流用は不可**(D-3改訂2026-07-08。UIはSlintで自前実装するためReactコンポーネントは流用対象外)。Rustコア(GPU compositor/effects/masks)は設計思想の参考 |
| [ffmpeg-sidecar](https://github.com/nathanbabcock/ffmpeg-sidecar) | MIT | **B-2対策の本命**。ffmpegバイナリをサイドカープロセスとして起動しrawvideoフレームをIterator APIで受け取るRustクレート。M0-S2スパイクはまずこれを評価し、足りなければ自前パイプ実装 |
| [wgpu](https://github.com/gfx-rs/wgpu) | Apache-2.0/MIT | レンダリングコアの土台(採用決定済み) |
| [Vello](https://github.com/linebender/vello) | Apache-2.0/MIT | **採用決定(2026-07-10、S3スパイク合格)**。vello 0.9=wgpu29依存で本体と同一デバイス同居を実測確認。条件: Renderer長寿命保持(初期化~900ms)・出力straight alpha→境界でpremul化・**vello_svgは使わず**usvg→vello変換は自前(スパイクに雛形)。バージョン結合がSlint↔wgpu↔velloの三者になる点に注意(A-3)。詳細は[spikes/s3-vello.md](spikes/s3-vello.md) |
| [Symphonia](https://github.com/pdeljanov/Symphonia) | MPL-2.0 | Pure Rust音声デコード(MP3/AAC/FLAC/WAV等)。音声インポート(B-1)の第一候補。MPLはファイル単位コピーレフトなので依存利用は安全 |
| [resvg / usvg](https://github.com/linebender/resvg) | MPL-2.0 | SVGパーサ(usvg: 参照解決済みの正規化ツリーを返す)。SVG読み込み(コンセプト決定でコア機能)の第一候補。linebender管理下で保守中。Vello描画と接続する(M4-K6) |
| [rubato](https://github.com/HEnquist/rubato) | MIT | 音声リサンプリング。バリスピード再生(M2音声トランスポート設計の適応リサンプリング)の候補 |
| [Slint](https://github.com/slint-ui/slint) | GPL / **Royalty-Free(デスクトップ無償)** / 商用 | **UI基盤に採用決定(2026-07-08)**。公式wgpu統合(`unstable-wgpu-29`、`Image::try_from(wgpu::Texture)`でゼロコピー埋め込み)+日本語IME実績([2025年調査](https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html)で合格)。[slint-interpreter](https://docs.rs/slint-interpreter)で.slintの実行時ロード可(プラグインパネル用)。Royalty-Freeライセンスの条文(帰属表示等)は配布前に精査 |
| [slint-off-thread-rendering](https://github.com/tronical/slint-off-thread-rendering) | MIT | Slint公式関係者の実験リポジトリ。**`require_wgpu_29(WGPUConfiguration::Manual)` を使う場合に、`default-features = false` でレンダラfeatureを明示固定する実例**。OpenGL/WGPUの混在ミスマッチを避ける設定の参照先 |
| [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) | Apache-2.0 | Zed製Rust GPU UIフレームワーク。crates.ioに単体公開済み(v0.2系、pre-1.0でAPI変動あり)。IME実績あり。Slint不合格時の検討候補。[gpui-component](https://github.com/longbridge/gpui-component)(既製コンポーネント集)も存在 |
| [Theatre.js](https://github.com/theatre-js/theatre) | core: Apache-2.0 / **studio: AGPL-3.0** | キーフレーム編集UI(シーケンスエディタ+グラフエディタ)の操作仕様の参考。**studio側はAGPLなのでコード流用禁止**、coreのデータモデル(JSON書き出し形式)は参考可 |

## 設計参考のみ(GPL系・特殊ライセンス、コード流用不可)

| リポジトリ | ライセンス | 何を学ぶか |
|---|---|---|
| [Olive](https://github.com/olive-editor/olive) | GPL-3.0 | ノードベースNLEをゼロから作った先行例。float中間パイプライン・カラーマネジメント(OCIO統合)・ディスクキャッシュの設計判断。「アルファのまま長期化した」経緯自体がスコープ管理(D-4)の教材。**死因分析(F群)**: 0.1→0.2フルリライトの理由は技術的負債+OCIO/OTIOの入れ直し。作者談「アーキが固まるまで性能はずっと最悪だった」(F-3/F-5の根拠) |
| [Natron](https://github.com/NatronGitHub/Natron) | GPL-2.0 | ノードグラフ評価・タイル/領域ベースのレンダリング要求伝播(RoI)。B-5のキャッシュキー設計の参考。**死因分析(F群)**: 開発停止の要因=2人プロジェクトの人的バス係数・長尺メモリ問題・マルチスレッドrace/deadlock・キャッシュ(trimap)デッドロック→作り直し未完(F-2の根拠) |
| [Remotion](https://github.com/remotion-dev/remotion) | 独自(企業は有償) | プレビュー/書き出し分離(B-4)の設計思想、「時刻t→決定的フレーム」の純関数モデル。コード流用は不可と考えること |
| [OpenFX (OFX)](https://openfx.readthedocs.io/) / After Effects SDK | OFX=BSD系仕様 / AE=独自 | **プラグイン拡張が業界標準の枯れた手法である裏付け**。エフェクト=「画像(テクスチャ)in+パラメータ→out」、ネイティブ(ピクセル/GPU)+スクリプト(パラメータ/ロジック)の二層、動的C ABIロード — いずれもうちの設計(concept: プラグインファースト/5-1隔離方針)と同型。Nuke/DaVinci/Natron/FlameがOFX採用。**トレードオフの明文化**: v1は独自Rust trait API(単純境界=LLMで書きやすさ最優先)を採り**OFX非互換**のため、既存OFXプラグイン資産は継承しない(自前エコシステムを育てる前提)。「なぜOFXにしないか」を蒸し返さないための記録。動的ロード(C ABI/`abi_stable`)はv2 |
| [Reco video-stitcher](https://github.com/reco-project/video-stitcher) | **AGPL-3.0**(+CLA) | 2カメラ映像のパノラマ合成ツール。**Slint GUI + wgpu(28)GPUパイプライン + ffmpegゼロコピーHWデコード(NVDEC/VideoToolbox)という、うちと同型の構成が実運用品質で成立している実証**(活発、1100+コミット)。学ぶ点: HWデコード→wgpuのゼロコピー統合(うちのv2スコープB-2の先行例)、push型フレーム投入APIと厳格な依存方向のクレート階層。wgpu 28使用 = Slintバージョン連動の現実の傍証 |
| [rs-wgpu-video-player](https://github.com/singh-ps/rs-wgpu-video-player) | GPL-2.0(表記曖昧、GPL扱い) | Slint UI + ffmpeg + cpalの動画プレイヤー(小規模・初期段階)。**「音声サンプル消費クロックを主、遅れた映像フレームはドロップ」というM2トランスポート設計と同一の結論に独立到達**している点が裏付けとして貴重。一方で反面教師も明確: フレームは`SharedPixelBuffer`(CPU経路)でSlintに渡しており(うちが避けたコピー路線)、libswscaleでCPU色変換(レビュー指摘#2で排除した経路)。既知の限界(シーク未実装・長時間でA/Vドリフト・1スロットキューでフレーム落ち)は、うちの有理数時間・有界キュー設計が対処すべき点のリスト |

## その他の依存候補(定番、必要時に評価)

- 音声出力: [cpal](https://github.com/RustAudio/cpal)(音声主クロック実装の土台)
- WASMランタイム: [wasmtime](https://github.com/bytecodealliance/wasmtime)(5-1のWASMパラメータプラグイン、v2)
- プラグインのクラッシュ隔離(設計思想の参考): **Bitwig Studio** — 別プロセスサンドボックス+5段階ホスティングモードで「1プラグインの異常が本体を落とさない/再生を止めない/自動再ロード」の模範(Abletonはネイティブ隔離なし=1個で全体が落ちる、が反面教師)。ただし音声バッファ前提でIPCが安いため、GPU(MB級・VRAM常駐)へは階層別に輸入する(concept.md「クラッシュ隔離を階層化」参照)
- オープンプラグイン標準(設計思想の参考): [CLAP](https://github.com/free-audio/clap)(Bitwig+u-he、**MIT**、C-ABI、明快なスレッドモデル、プロセス外ホスティング対応、WASM版=WCLAPあり)。OFX/VSTと違いオープンで、我々のOSS思想と親和。そのまま採用ではなく境界設計の参考
- dylib ABI: [abi_stable](https://github.com/rodrimati1992/abi_stable_crates) / stabby(動的ロード導入時=v2に評価)
- 色管理: [OpenColorIO](https://github.com/AcademySoftwareFoundation/OpenColorIO)(BSD-3。v1は自前の最小色空間タグで済ませ、HDR対応時に検討。パイプラインをOCIO-shapedに保つ規律はperformance-model参照=F-5)
- テキスト基盤(F-6、M5-P6): [rustybuzz](https://github.com/harfbuzz/rustybuzz)(MIT、HarfBuzzのRust移植=日本語シェーピング)/ [fontdb](https://github.com/RazrFalcon/fontdb)(MIT、フォント列挙・読み込み)。いずれもresvg/linebender系で実績あり
- タイムライン交換: [OpenTimelineIO](https://github.com/AcademySoftwareFoundation/OpenTimelineIO)(Apache-2.0。v2の書き出し候補。v1はスキーマの素性を寄せるのみ=F-5。座標系提案「単位なし・単一原点・Y-up」はF-1の正準座標系の参考元)
- 並行テスト: [loom](https://github.com/tokio-rs/loom)(MIT。M4-K1のキャッシュ並行契約の検証候補=F-2)

## プラットフォーム母数の出典(2026-07-10。E章ターゲットOS/F-9の根拠)

クロスプラットフォームを設計レベルの恒久要件とする母数根拠。クリエイティブ層のOS比率に信頼できる一次統計は存在しない(Adobeは非公表)ため、傾向の裏取りとして使う。

- [StatCounter — Desktop OS Market Share Worldwide](https://gs.statcounter.com/os-market-share/desktop/worldwide/) — 2026-06: Windows 56.55%(初の60%割れ)、macOS系合算 約16%("OS X"と"macOS"に分かれて計上される点に注意)、Linux 4.39%。Web利用ベースの推計であり導入台数ではない。[報道(linuxiac)](https://linuxiac.com/windows-drops-under-60-in-global-desktop-os-share-for-the-first-time-in-years/)
- [Adobe — After EffectsのGPU機能](https://helpx.adobe.com/after-effects/using/basics-gpu-after-effects.html) — **macOSはCUDA非サポート**(Metal/OpenCLのみ)の一次ソース。AEプラグイン圏がWindows限定化する技術的根拠(F-9の1)
- [School of Motion — Mac vs PC for MoGraph](https://www.schoolofmotion.com/blog/mac-vs-pc-motion-design-value) — モーションデザイン業界のMac/PC選択の論点整理(コスパ vs エコシステム)
- [Mac vs PC for Graphic Design 2025(Alibaba insights)](https://www.alibaba.com/product-insights/mac-vs-pc-for-graphic-design-in-2025-which-pros-prefer.html) — エージェンシー系デザイナーMac 58%・UI/UX Mac 67%・3D/VFXワークステーションWindows 72%等の断片。**SEO寄り記事で精度は粗い**。「デザイン系=Mac優勢、GPU系=Windows優勢」の傾向確認のみに使う

## 検索ソース

- OpenCut: [公式サイト](https://opencut.dev/), [GitHub org](https://github.com/OpenCut-app/)
- ffmpeg-sidecar: [crates.io](https://crates.io/crates/ffmpeg-sidecar), [docs.rs](https://docs.rs/ffmpeg-sidecar/latest/ffmpeg_sidecar/)
- Theatre.js: [README(ライセンス記載)](https://github.com/theatre-js/theatre/blob/main/README.md)
- GPUI: [crates.io](https://crates.io/crates/gpui), [gpui.rs](https://www.gpui.rs/)
- Symphonia: [GitHub](https://github.com/pdeljanov/Symphonia)
- Remotion: [ライセンス](https://www.remotion.dev/docs/license)
- Vello: [GitHub](https://github.com/linebender/vello)
- Olive: [GitHub](https://github.com/olive-editor/olive) / Natron: [GitHub](https://github.com/NatronGitHub/Natron)
