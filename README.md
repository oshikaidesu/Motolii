# Motolii

**ネットで見つけたかっこいい表現を、自分の動画にすぐ使う。** 新しい表現のために、開発者の対応を待たない。

Motolii は MV（絵が主役のモーショングラフィックス）を作る、macOS のアプリです。ネットで配られているシェーダーの効果（ISF・WGSL）や AviUtl2 のスクリプトを、**ファイルを置くだけで**自分の動画に掛けられます。2D と 3D を一つの場面で扱います。いまは作っている途中で、配れるアプリ（リリース）はまだありません。

[English](README.en.md)

![Motolii の窓：左に素材と棚、真ん中に場面、下にタイムライン、右に値](docs/media/window.jpg)

![配られている ISF の効果 8 本を写真と文字に掛けた所](docs/media/effects.jpg)

*配られている ISF の効果 8 本（CMYK Halftone・Bad TV・Duotone・Edges・Dither-Bayer・Kaleidoscope Tile・Glow-Fast・Pixellate）を、写真（パブリックドメイン）と文字に掛けた所。どれも作者のファイルのまま。*

## 何ができるか（いま）

| 置ける物 | 何か | いまの様子 |
|---|---|---|
| ISF の効果（`.fs`） | VJ の世界で配られているシェーダーの形式（[ISF-Files](https://github.com/Vidvox/ISF-Files) など） | 効果のフォルダに置けば棚に出る。ISF-Files の 327 本中 318 本が通る。同梱 115 本 |
| WGSL の効果（`.wgsl`） | WebGPU のシェーダー。three.js などの画面効果を移した物 | 画面全体にも、物 1 つだけにも掛かる。同梱 18 本 |
| AviUtl2 のスクリプト（`.anm2`・`.obj2`） | 日本の動画編集ソフト AviUtl2 のスクリプト。中の HLSL を WGSL に変えて動かす | 同梱 172 パッケージ。DLL の要らない物を中心に、まだ一部 |
| TSL（three.js のシェーダー言語） | ネットの TSL の集まり 48 か所を、three.js 自身の変換器で WGSL に | WGSL まで。編集の棚にはまだ無い |
| 動画・写真・文字・図形・3D のモデル（glTF）・点群（`.ply`） | 素材 | 動く（動画は GStreamer、文字と図形は ThorVG、3D は Filament） |
| After Effects の書き出し（Lottie の `.json`） | アニメーションの絵 | 動く（ThorVG） |
| MIDI（`.mid`） | 音符を場面の部品として置く（ピアノロール、音に合わせて値にキーを打つ） | 動く。音はまだ鳴らない |
| 自分のスクリプト（`.js`） | 1 回で絵を作るレシピ、物の動き | 置けば棚に出る。同梱のレシピ 44 本 |

ほか：キーフレームとイーズ（anime.js のイーズもそのまま使える）、物理（Jolt）、ガラスの屈折、mp4 への書き出し。何が在ってどこまで確かめたかの全部は、作者の台帳 [`ASSETS.md`](ASSETS.md) にあります。

## 他の道具との違い

| | Motolii | After Effects | Cavalry | Alight Motion | Motion Canvas |
|---|---|---|---|---|---|
| 新しい表現の足し方 | 配られているファイルを置く（ISF・WGSL・AviUtl2・JS） | プラグインを買う・作る | 自分で組む | アプリの更新を待つ | コードを書く |
| 2D と 3D | 一つの場面（Filament） | 2D と 3D のレイヤー | 2D が中心 | 2D が中心 | 2D |
| 触り方 | 窓。操作の手本は Alight Motion | 窓 | 窓 | スマホ | コード |
| 値段・ライセンス | 無料、Apache-2.0 か MIT | 有料 | 有料（無料版あり） | 有料（無料版あり） | 無料、MIT |

## 作り

```mermaid
flowchart LR
  W["窓<br/>Flutter（skin/book）"] -- "C の呼び出し<br/>motolii.h" --> E["中身 libmotolii.dylib<br/>（filament/）"]
  E --> F["Filament<br/>3D・最後の合成"]
  E --> T["ThorVG<br/>2D・文字"]
  E --> G["GStreamer<br/>動画・音"]
  E --> S["SQLite<br/>書類（.motolii）"]
  F --> D["Dawn（WebGPU）"]
  T --> D
  E -- "描いた絵をコピーせずに" --> W
```

窓は操作だけで、絵は中身が描きます。中身のフォルダ名が `filament/` なのは歴史的な理由で、Google の Filament そのものではありません（Filament は組み立ての時に取ってきます）。

## いまの状態

- 動く：上の表の「動く」の物、書類の保存と取り消し、再生、mp4 への書き出し、窓の Timeline・Inspector・棚。
- 一部：AviUtl2 のスクリプト、Alight Motion の操作の写し（手本の操作の表 649 行のうち、同じに動くのは 14%、無いのは 63%：[`reports/overview.md`](reports/overview.md)）。
- まだ：Windows、配れる `.app`（中身が Homebrew の部品を読むので、作った Mac の外では動かない）、作業中の音の一部。

## 使ってみる

### 要る物

| 物 | 版 | 入れ方 |
|---|---|---|
| Mac | Apple Silicon（確かめたのは macOS 15.5） | — |
| Xcode | App Store の Xcode（窓を組むのに要る。Command Line Tools だけでは足りない） | App Store |
| Homebrew | — | [brew.sh](https://brew.sh/ja/) |
| Flutter | 3.47.2（stable） | [flutter.dev の macOS の手順](https://docs.flutter.dev/get-started/install/macos)、`flutter` を PATH に |
| 空き容量 | 約 10 GB（組み立てた後で 8.1 GB） | — |
| 時間 | 初回 約 10 分（M4。Filament 約 5 分、中身 約 1〜2 分、窓 約 1〜3 分。取ってくる時間は別） | — |

### 組み立てて開く

```bash
git clone https://github.com/oshikaidesu/motolii.git
```

```bash
cd motolii
```

```bash
brew bundle
```

`Brewfile` の物（GStreamer・SQLite・cmake など）がまとめて入ります。

```bash
filament/dawn_probe/build.sh
```

Filament（WebGPU）を取ってきて組みます（M4 で約 5 分）。

```bash
filament/edit.sh out/first.motolii < /dev/null
```

中身を組みます。ThorVG・Skia・libtess2・Jolt などは、ここで初回だけ取ってきます。

```bash
skin/book/live.sh
```

窓が開きます。窓を閉じると終わります。

### 最初の 1 本（コマンドで）

窓を使わずに、レシピで絵を作り、物に光（Glow）を、画面全体に RGB のずれ（`rgbshift.wgsl`）を掛けて、8 秒の mp4 に書き出す例です。`filament/edit.sh` は書類を開いて、1 行ずつの命令を受けます（`help` で一覧）。

```bash
filament/edit.sh $HOME/Movies/first.motolii <<EOF
make Weave
effect 1 Glow
say 0 script $PWD/filament/effects/rgbshift.wgsl
write $HOME/Movies/first.mp4
EOF
```

![最初の 1 本：線の模様が波打ち、色がずれる](docs/media/first.gif)

- `make Weave`：レシピ「Weave」で物 1 を作る（`recipes` で一覧）。
- `effect 1 Glow`：物 1 に効果を掛ける（`effects` で一覧）。
- `say 0 script …`：物 0（場面そのもの）に画面全体の効果を置く。ISF の `.fs` は `say 1 fx …` で物に置ける。
- `write …`：mp4 に書き出す。

### 窓の見方

- 左：Media と棚（Create はレシピと図形・文字、Effects は効果）。タイルを押すと場面に置かれる。
- 真ん中：Stage（中身が描いた絵）。下は Timeline（物ごとの帯とキー）と Ease（キーの間の動き方）。
- 右：Inspector（選んだ物の値。数字の上で横に引くと変わる）。右上の Render で書き出す。
- 自分の効果やレシピは `~/Library/Application Support/Motolii/` の `effects/`・`recipes/`・`ways/` に置くと、その場で棚に出ます。

### 困ったら

| 出た事 | 直し方 |
|---|---|
| `flutter: command not found` | Flutter を PATH に置く（flutter.dev の手順）。別の場所なら `FLUTTER=/path/to/flutter skin/book/live.sh` |
| `brew: command not found` | [brew.sh](https://brew.sh/ja/) の 1 行で Homebrew を入れる |
| `xcrun: error` や、Xcode が無いと言われる | App Store で Xcode を入れ、一度開いて追加の部品を入れる |
| `ld: library 'webgpu_dawn' not found` | `filament/dawn_probe/build.sh` を先に走らせる |
| 窓は開くが Stage が黒い | 中身を組んだか確かめる（`filament/out/libmotolii.dylib` があるか）。無ければ `filament/edit.sh out/first.motolii < /dev/null` |

それでも止まったら [Issue](https://github.com/oshikaidesu/motolii/issues) に、走らせたコマンドと出た文をそのまま貼ってください。

## 開発に加わる

[`CONTRIBUTING.md`](CONTRIBUTING.md) にあります（組み立て、試験の走らせ方、決まり、手を付けやすい所）。

### 考え方

- **自作は負債。** まず既存の実装を探し、作者のやり方のまま使う。無ければ周りの技術やコミュニティまで探す。
- 他の人の作品はなるべく手を入れずに同梱し、隣に作者のライセンスと出所を置く。手を入れる時は、元に当てる差分（`.patch`）に分けて元は作者の配布元から取ってくるか、ファイルの 1 行目に元と改変を書く。
- 今は自分で書いたコードも多い（窓の Flutter など）。借りられる物が見つかれば、自分のコードを捨てて取り替える。
- コミットごとに、何をどこから借りたか（出典）を書く。

### 並び

| 場所 | 中身 |
|---|---|
| `filament/` | 中身（エンジン）。C++ / Objective-C++。組み立てのスクリプトもここ |
| `filament/effects/`・`recipes/`・`animators/`・`ways/` | 同梱の効果・レシピ・動き・イーズ（利用者のフォルダと同じ形） |
| `filament/aviutl_scripts/` | 同梱の AviUtl2 のスクリプト（作者ごと、LICENSE と SOURCE.txt つき） |
| `filament/check/` | 中身の試験（`*.sh` と期待する出力） |
| `skin/book/` | 窓（Flutter、macOS）。試験は `test/` と `integration_test/` |
| `thorvg/` | ThorVG を wgpu に載せた前の結合（前の試しのスクリプトが使う） |
| `old/` | 前の土台の実験（今の作りからは使っていない） |
| `docs/media/` | この README の絵（Motolii で作った物と、パブリックドメインの写真に掛けた物） |

### 文書の地図

| 外向け | 作者の作業メモ（日本語、そのまま公開） |
|---|---|
| `README.md`・`README.en.md`・`CONTRIBUTING.md`・`THIRD_PARTY_NOTICES.md` | `ASSETS.md`（在る物の台帳）・`TASKS.md`（やること）・`AE_RUBRIC.md`（採点表）・`reports/`（調べた記録。絵と動画は他の人のアプリや素材が映る物が多いので git に入れていない） |

## ライセンス

Motolii 自身の物は [Apache License 2.0](LICENSE-APACHE) と [MIT](LICENSE-MIT) のどちらでも、使う人が選べます（ダブルライセンス）。送ってもらった貢献も、特に断りが無ければ同じ二つのライセンスで受けます。同梱した他の人の作品は、それぞれのライセンスに従います（[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)）。

*Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions. Bundled third-party works keep their own licences.*
