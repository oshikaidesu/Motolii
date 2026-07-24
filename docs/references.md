# 参考リポジトリ・ライブラリ一覧

作成日: 2026-07-07(Web検索で所在・ライセンスを確認済み)

用途別に「コードを流用/依存してよいもの」と「設計参考のみ(ライセンス・状況的にコード流用不可)」を明確に区別する。**GPL系はコードを読んで書き写すだけでも汚染リスクがあるため、設計・データ構造の考え方の参考に留める。**

## コード流用・依存候補(ライセンス上安全)

| リポジトリ | ライセンス | 何を参考/利用するか |
|---|---|---|
| [OpenCut](https://github.com/OpenCut-app/OpenCut) | MIT | タイムラインUIの**操作仕様の参考のみ。コード流用は不可**(Rust/egui UIのためReact componentは流用対象外)。Rustコア(GPU compositor/effects/masks)は設計思想の参考 |
| [ffmpeg-sidecar](https://github.com/nathanbabcock/ffmpeg-sidecar) | MIT | **B-2対策の本命**。ffmpegバイナリをサイドカープロセスとして起動しrawvideoフレームをIterator APIで受け取るRustクレート。M0-S2スパイクはまずこれを評価し、足りなければ自前パイプ実装。Rerun 0.34.1の`re_video`もH.264デコードに同crateを採用しており独立収束の傍証([Rerun先例調査](reviews/2026-07-20-rerun-prior-art-survey.md)) |
| [wgpu](https://github.com/gfx-rs/wgpu) | Apache-2.0/MIT | レンダリングコアの土台(採用決定済み) |
| [Vello](https://github.com/linebender/vello) | Apache-2.0/MIT | **採用決定(2026-07-10、S3スパイク合格)**。vello 0.9=wgpu29依存で本体と同一device同居を実測確認。条件: Renderer長寿命保持(初期化~900ms)・出力straight alpha→境界でpremul化・**vello_svgは使わず**usvg→vello変換は自前。version結合がegui-wgpu↔wgpu↔velloの三者になる点に注意(A-3)。詳細は[spikes/s3-vello.md](spikes/s3-vello.md) |
| [Symphonia](https://github.com/pdeljanov/Symphonia) | MPL-2.0 | Pure Rust音声デコード(MP3/AAC/FLAC/WAV等)。音声インポート(B-1)の第一候補。MPLはファイル単位コピーレフトなので依存利用は安全 |
| [resvg / usvg](https://github.com/linebender/resvg) | MPL-2.0 | SVGパーサ(usvg: 参照解決済みの正規化ツリーを返す)。SVG読み込み(コンセプト決定でコア機能)の第一候補。linebender管理下で保守中。Vello描画と接続する(M4-K6) |
| [rubato](https://github.com/HEnquist/rubato) | MIT | 音声リサンプリング。**明示**スクラブ/シャトルのバリスピード候補(自動フォールバック用途ではない — [D5先例調査](reviews/2026-07-14-d5-transport-prior-art.md))。デバイス≠素材レートの固定比変換にも候補 |
| [egui](https://github.com/emilk/egui) / [egui_tiles](https://github.com/rerun-io/egui_tiles) | MIT OR Apache-2.0 | **UI基盤に採用決定(2026-07-18)**。egui-wgpu 0.35の`WgpuSetup::Existing`とnative textureをApple M4 / Metalで実測。日本語IME、resize/minimize/restore、idle停止も確認。egui_tilesはruntime投影先で、生Tree/TileIdを保存正本にしない。[採用判断](reviews/2026-07-18-m3-egui-selection.md) |
| [Rerun](https://github.com/rerun-io/rerun) | repository: MIT OR Apache-2.0、`re_ui`: `(MIT OR Apache-2.0) AND OFL-1.1` | **egui製品実装の主要先例(2026-07-20決定)**。同世代のegui 0.35 / egui_tiles 0.16 / wgpu 29で、`re_ui`、Viewport/Blueprint、Time Panel/density、selection、egui-wgpu callback、`re_renderer`、parallel View execution、snapshot試験を層別に学ぶ。Reactモックの要求を置換せず、資産ごとに`DEPEND/VENDOR/PORT/PATTERN/REJECT`を裁定する。[先例調査](reviews/2026-07-20-rerun-prior-art-survey.md)、[学習・転移計画](reviews/2026-07-20-rerun-learning-transfer-plan.md) |
| [Slint](https://github.com/slint-ui/slint) | GPL / Royalty-Free / 商用 | **歴史的な採用候補**。2026-07-08採用、S1合格後、2026-07-18にeguiへ置換。Manual wgpu共有、renderer feature、IMEの測定事実は[歴史証拠](spikes/s1-slint.md)として維持する |
| [slint-off-thread-rendering](https://github.com/tronical/slint-off-thread-rendering) | MIT | Slint公式関係者の実験リポジトリ。**`require_wgpu_29(WGPUConfiguration::Manual)` を使う場合に、`default-features = false` でレンダラfeatureを明示固定する実例**。OpenGL/WGPUの混在ミスマッチを避ける設定の参照先 |
| [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) | Apache-2.0 | Zed製Rust GPU UI framework。crates.ioに単体公開済み(v0.2系、pre-1.0でAPI変動あり)。IME実績あり。egui比較時の候補だったが不採用。[gpui-component](https://github.com/longbridge/gpui-component)も存在 |
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
| [Basic Memory](https://github.com/basicmachines-co/basic-memory) | AGPL-3.0 | 歴史価値回収の**任意の外部ローカル索引CLI**候補。0.22.1をblack-box検証し、公式文書だけを参照する。Motoliiへvendor/link/source copyせず、製品runtime・通常build・CI・coverageの必須依存にしない。可搬projection、receipt正本、生成物非commitの境界は[意味グラフ補助境界](reviews/2026-07-23-historical-semantic-graph-recovery-tooling.md)を正本とする |

## その他の依存候補(定番、必要時に評価)

- 音声出力: [cpal](https://github.com/RustAudio/cpal)(音声主クロック実装の土台)
- 大量行table widget: [egui_table](https://github.com/rerun-io/egui_table)(MIT OR Apache-2.0、rerun-io保守、egui 0.35対応。sticky header・"millions of rows"・可変行高。Browser/Inspector大量行の**未評価候補** — 内部結合の`re_dataframe_ui`と違い外部leaf crateなので`DEPEND`比較対象。[Rerun inventory §5.6](reviews/2026-07-20-rerun-source-asset-inventory.md))
- WASMランタイム: [wasmtime](https://github.com/bytecodealliance/wasmtime)(5-1のWASMパラメータプラグイン、v2)
- プラグインのクラッシュ隔離(設計思想の参考): **Bitwig Studio** — 別プロセスサンドボックス+5段階ホスティングモードで「1プラグインの異常が本体を落とさない/再生を止めない/自動再ロード」の模範(Abletonはネイティブ隔離なし=1個で全体が落ちる、が反面教師)。ただし音声バッファ前提でIPCが安いため、GPU(MB級・VRAM常駐)へは階層別に輸入する(concept.md「クラッシュ隔離を階層化」参照)
- 「馬鹿正直にシミュレートしない」の先行実証(設計思想の参考、いずれもクローズド): **[Furikake](https://aescripts.com/furikake/)** — AEの軽量粒子プラグイン。物理を「重力/風=閉形式・乱流=ノイズ変位・バウンス=解析反射」の f(t) に畳める力だけに選定し、粒子間相互作用を持たないことで O(N)・マルチコア・MFR対応の"バカ軽さ"を成立させた(concept.md根本コンセプトの母数)。**Alight Motion** — バウンス/バネ/段階移動を物理シミュでなくパラメトリック補間型([Animation Easing Curves](https://support.alightmotion.com/hc/en-us/articles/10536934703889-Animation-Easing-Curves))に畳み、AEでは`valueAtTime`式が必須だった領域をGUI選択肢化(Interp設計に採用済み)
- グループへのエフェクト適用の先人比較(設計思想の参考、2026-07-10。concept「プリコンポは作らない/項目エンベロープ」決定の母数): **Alight Motion** — グループを1つのレイヤーとして選択し、単体レイヤーと同様にエフェクト/アニメーションを適用できる([レイヤー管理ガイド](https://themotionalight.com/group-and-ungroup-layers-in-alight-motion/))=採用した意味論。**AviUtl** — [グループ制御](https://aviutl.info/guru-puseigyo/)はフィルタを対象オブジェクトへ**個別**適用する意味論のため、「合成結果1枚に掛ける」用途では[フレームバッファ](https://aviutl.info/hure-mubaffa/)(画面全体を掴む)への迂回が定番 — per-child意味論と「画面全体しか掴めない」迂回の両方が反面教師([複数オブジェクトへまとめてフィルタ](https://scrapbox.io/aviutl/%E8%A4%87%E6%95%B0%E3%81%AE%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88%E3%81%AB%E3%81%BE%E3%81%A8%E3%82%81%E3%81%A6%E3%83%95%E3%82%A3%E3%83%AB%E3%82%BF%E3%82%92%E6%8E%9B%E3%81%91%E3%82%8B)も同趣旨)。**Photoshop/クリスタ** — グループ既定の「通過(pass through)」ブレンドは分離合成と排他の二重意味論(通過グループにはエフェクトが定義できない)であり、通過不採用の根拠
- オープンプラグイン標準(設計思想の参考): [CLAP](https://github.com/free-audio/clap)(Bitwig+u-he、**MIT**、C-ABI、明快なスレッドモデル、プロセス外ホスティング対応、WASM版=WCLAPあり)。OFX/VSTと違いオープンで、我々のOSS思想と親和。そのまま採用ではなく境界設計の参考
- dylib ABI: [abi_stable](https://github.com/rodrimati1992/abi_stable_crates) / stabby(動的ロード導入時=v2に評価)
- 色管理: [OpenColorIO](https://github.com/AcademySoftwareFoundation/OpenColorIO)(BSD-3。v1は自前の最小色空間タグで済ませ、HDR対応時に検討。パイプラインをOCIO-shapedに保つ規律はperformance-model参照=F-5)
- テキスト基盤(F-6、M5-P6、**2026-07-10決定**):
  - シェーピング: [harfrust](https://github.com/harfbuzz/harfrust)(MIT、HarfBuzz公式Rust・rustybuzz後継。read-fonts/skrifa系でVelloとフォント解析を一本化)。旧候補[rustybuzz](https://github.com/harfbuzz/rustybuzz)は保守モード(HB≈v10相当)のため採用しない
  - 列挙+フォールバック: [fontique](https://github.com/linebender/parley/tree/main/fontique)(Apache-2.0/MIT、linebender。fontdbは列挙のみでフォールバック未実装のため置き換え)
  - 描画: Vello `Scene::draw_glyphs`(グリフID+位置を渡す。アウトライン化はVello内部のskrifa。自前「グリフ→パス」は不要)。`normalized_coords` / `font_size` / `glyph_transform` でバリアブル軸・サイズ・グリフ単位変形を貫通
  - **コアAPI**: ラン単位の `itemize` / `shape(variations・クラスタ対応表)` / `draw` のみ。一発`draw_text`は禁止([M5-P6](specs/M5-3d-and-post.md))
  - **コアに入れない**: [Parley](https://github.com/linebender/parley)丸ごと — 横書きレイアウト統合層で縦書き未対応。歌詞尖り(縦書き・ルビ)はシェーパ直叩き+プラグイン組版(F-6分界)
- パス演算子(AEシェイプ演算子ファミリー、F-13・concept 2026-07-10決定): [lottie-docs Shapes](https://lottiefiles.github.io/lottie-docs/shapes/) — BodymovinがAEシェイプレイヤーを書き出すため、パンク・膨張(`pb`)/ジグザグ(`zz`)/パスのオフセット(`op`)/角丸(`rd`)/トリムパス(`tm`)/ツイスト(`tw`)/リピーター(`rp`)/パス結合(`mm`)の**意味論とシリアライズが公開文書化済み**(「Adobe独占」はオーサリングUIの話であって数学ではない — スキーマ設計の直接の前例に使う)。数学の参照実装は [lottie-web](https://github.com/airbnb/lottie-web)(MIT)。[Glaxnimate](https://github.com/KDE/glaxnimate)(**GPL-3.0=設計参考のみ・コード流用不可**)はOffset Path/Zig Zag等を独立実装したデスクトップ実例。パスオフセットの幾何は [kurbo](https://github.com/linebender/kurbo)(Apache-2.0/MIT、Vello系列で依存済み)の`offset`/`stroke`モジュールが土台候補
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
