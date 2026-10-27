# Motolii

**ネットで見つけたかっこいい表現を、自分の動画にすぐ使う。** 新しい表現のために、開発者の対応を待たない。

Motolii は MV（絵が主役のモーショングラフィックス）を作る、macOS のアプリです。ネットで配られているシェーダーの効果（ISF・WGSL）や AviUtl2 のスクリプトを、**ファイルを置くだけで**自分の動画に掛けられます。2D と 3D を一つの場面で扱います。

- **いまの段階**：作っている途中（このリポジトリは 2026-10-12 から。前の作り [rust_ae](https://github.com/oshikaidesu/rust_ae) は 7 月から。作者 1 人）。配れるアプリ（リリース）はまだ無く、試すにはソースから組みます。
- **いま試すと良い人**：Mac（Apple Silicon）があり、ターミナルでコマンドを貼って組める人。シェーダーの効果や AviUtl2 のスクリプトを動画に使ってみたい人。
- **まだ待つと良い人**：普段の制作にすぐ使いたい人、Windows の人、アプリを入れるだけで使いたい人。

[English](README.en.md) ・ [質問・相談（Discussions）](https://github.com/oshikaidesu/motolii/discussions) ・ [不具合（Issues）](https://github.com/oshikaidesu/motolii/issues)　どちらも日本語・英語のどちらでも。

| Motolii で作った 8 秒（文字・3D・効果） | 窓 |
|---|---|
| ![光る文字、回るピンクの輪、線の模様、色のずれ](docs/media/mv.gif) | ![Motolii の窓：左に素材と棚、真ん中に場面、下にタイムライン、右に値](docs/media/window.jpg) |

*左の 8 秒は、レシピ（線の模様）・3D の輪・文字・光（Glow）・RGB のずれを置いて書き出した物。作り方は [`docs/examples/mv.txt`](docs/examples/mv.txt)（下の「最初の 1 本」と同じやり方で走らせる）。*

![配られている ISF の効果 8 本を写真と文字に掛けた所](docs/media/effects.jpg)

*配られている ISF の効果 8 本（CMYK Halftone・Bad TV・Duotone・Edges・Dither-Bayer・Kaleidoscope Tile・Glow-Fast・Pixellate）を、写真（パブリックドメイン）と文字に掛けた所。どれも作者のファイルのまま。*

## 何ができるか（いま）

| 置ける物 | 何か | いまの様子 |
|---|---|---|
| ISF の効果（`.fs`） | VJ の道具で配られている、画面に掛けるシェーダーの効果（[ISF-Files](https://github.com/Vidvox/ISF-Files) など） | 効果のフォルダに置けば棚に出る。ISF-Files の 327 本中 318 本が通る（`filament/check/all.sh` で数えられる）。同梱 115 本 |
| WGSL の効果（`.wgsl`） | WebGPU のシェーダー。three.js などの画面効果を移した物 | 画面全体に掛かる。同梱 18 本 |
| AviUtl2 のスクリプト（`.anm2`・`.obj2`） | 日本の動画編集ソフト AviUtl2 の効果。中の HLSL を WGSL に変えて動かす | 同梱 172 パッケージ。DLL の要らない物を中心に、まだ一部 |
| TSL（three.js のシェーダー言語） | ネットの TSL の集まり 48 か所を、three.js 自身の変換器で WGSL に | WGSL まで。編集の棚にはまだ無い |
| 動画・写真・文字・図形・3D のモデル（glTF）・点群（`.ply`） | 素材 | 動く（動画は GStreamer、文字と図形は ThorVG、3D は Filament） |
| After Effects の書き出し（Lottie の `.json`） | アニメーションの絵 | 動く（ThorVG） |
| MIDI（`.mid`） | 音符を場面の部品として置く（ピアノロール、音に合わせて値にキーを打つ） | 動く。音はまだ鳴らない |
| 自分のスクリプト（`.js`） | 1 回で絵を作るレシピ、物の動き | 置けば棚に出る。同梱のレシピ 44 本 |

ほか：キーフレームとイーズ（anime.js のイーズもそのまま使える）、物理（Jolt）、ガラスの屈折、mp4 への書き出し。何が在ってどこまで確かめたかの全部は、作者の台帳 [`ASSETS.md`](ASSETS.md) にあります。

## 他の道具との違い

| | Motolii | After Effects | Cavalry | Alight Motion | Blender | VDMX・TouchDesigner | Motion Canvas・Rive |
|---|---|---|---|---|---|---|---|
| 新しい表現の足し方 | 配られているファイルを置く（ISF・WGSL・AviUtl2・JS） | プラグインを買う・作る | 自分で組む | アプリの更新を待つ | ノードや Python で組む | ISF（VDMX）や GLSL を組む | コードを書く・エディタで作る |
| 主な使い道 | MV の映像を作って書き出す | 映像の合成 | 2D のモーショングラフィックス | スマホで映像 | 3D 全般 | ライブの映像（演奏） | コードで映像（Motion Canvas）、アプリの中のアニメーション（Rive） |
| 2D と 3D | 一つの場面（Filament） | 2D と 3D のレイヤー | 2D が中心 | 2D が中心 | 3D が中心 | 両方（組み方による） | 2D |
| 動く所 | macOS（Apple Silicon）だけ | Windows・macOS | Windows・macOS | iOS・Android | Windows・macOS・Linux | VDMX は macOS、TouchDesigner は Windows・macOS | ブラウザ・各種 |
| 成熟度 | このリポジトリで 2 週間、作者 1 人、リリース無し | 長く使われている | 製品 | 製品 | 長く使われている | 製品 | 使われている |
| 値段・ライセンス | 無料、Apache-2.0 か MIT | 有料 | 有料（無料版あり） | 有料（無料版あり） | 無料、GPL | 有料（無料版あり） | Motion Canvas は無料・MIT、Rive は無料版あり |

**Motolii がまだ負けている所**：出来上がった道具としての安定と機能の数（どれも上の道具が上）、配れるアプリが無い事、Windows で動かない事、ライブで演奏する使い方は考えていない事。

## 作り

```mermaid
flowchart LR
  W["窓（アプリの画面）<br/>Flutter（skin/book）"] -- "C の呼び出し<br/>motolii.h" --> E["中身（エンジン）libmotolii.dylib<br/>（filament/）"]
  E --> F["Filament<br/>3D・最後の合成"]
  E --> T["ThorVG<br/>2D・文字"]
  E --> G["GStreamer・ffmpeg<br/>動画を読む・書く"]
  E --> S["SQLite<br/>プロジェクトファイル（.motolii）"]
  F --> D["Dawn（WebGPU）"]
  T --> D
  E -- "描いた絵をコピーせずに" --> W
```

窓は操作だけで、絵は中身が描きます。中身のフォルダ名が `filament/` なのは歴史的な理由で、Google の Filament そのものではありません（Filament は組み立ての時に取ってきます）。詳しくは [`ARCHITECTURE.md`](ARCHITECTURE.md)（入口のファイル、効果を WGSL にする道、この repo の言葉とふつうの言い方の対応表）。

## いまの状態

- 動く：上の表の「動く」の物、書類の保存と取り消し、再生、mp4 への書き出し、窓の Timeline・Inspector・棚。
- 一部：AviUtl2 のスクリプト、Alight Motion の操作の写し（手本の操作の表 649 行のうち、同じに動くのは 14%、無いのは 63%：[`reports/overview.md`](reports/overview.md)）。
- 知っている不具合：コピーで作るレシピ（Iris・Weave など）に画面用の効果（`.wgsl`）を物として掛けると絵が崩れる（場面全体に掛けるのは大丈夫）。
- まだ：Windows、配れる `.app`（中身が Homebrew の部品を読むので、作った Mac の外では動かない）、作業中の音の一部。

## 使ってみる

### 要る物

| 物 | 版・大きさ | 入れ方 |
|---|---|---|
| Mac | Apple Silicon（確かめたのは macOS 15.5） | — |
| Xcode | App Store の Xcode（窓を組むのに要る。Command Line Tools だけでは足りない）。入れると約 5 GB〜（版による） | App Store。入れたら一度開いて追加の部品を入れ、`sudo xcodebuild -license accept` |
| Homebrew | — | [brew.sh](https://brew.sh/ja/) の 1 行 |
| Flutter | 3.47.2（stable） | [flutter.dev の macOS の手順](https://docs.flutter.dev/get-started/install/macos)。入れたら `flutter doctor` で Xcode と macOS の行に ✓ が付くか見る |
| 空き容量 | 約 10 GB（組み立てた後で 8.1 GB。Xcode と Flutter は別） | — |
| 取ってくる量 | 約 1 GB（Filament のソース 635 MB、Skia 65 MB、Jolt 16 MB ほか、Homebrew の物は別） | — |
| 時間 | 初回 約 10 分（M4。Filament 約 5 分、中身 約 1〜2 分、窓 約 1〜3 分）に、取ってくる時間 | — |

### 組み立てて開く

ターミナルで 1 つずつ貼ります。2 つ目からは、`motolii` のフォルダの中で（`cd motolii` した後で）走らせます。

```bash
git clone https://github.com/oshikaidesu/motolii.git
```

```bash
cd motolii
```

```bash
brew bundle
```

`Brewfile` の物（GStreamer・SQLite・ffmpeg・cmake など）が入ります。最後に `Homebrew Bundle complete!` と出れば成功。

```bash
filament/dawn_probe/build.sh
```

Filament（WebGPU）を取ってきて組みます（M4 で約 5 分）。最後に `dawn_probe: 4096 triangles …` と数字の行が出れば成功。

```bash
filament/edit.sh build.motolii < /dev/null
```

中身を組みます。ThorVG・Skia・libtess2・Jolt などは、ここで初回だけ取ってきます。`edit.sh` は書類を開いて命令を待つ道具なので、`< /dev/null` で「命令なし」を渡し、組み立てて空の書類を作ったらすぐ終わらせています。`filament/out/libmotolii.dylib` ができれば成功（`build.motolii` は消してかまいません）。

```bash
skin/book/live.sh
```

窓が開きます。窓を閉じると終わります。

### 最初の 1 本（窓で）

![1：空の書類](docs/media/window_step1.jpg)

1. 窓が開いたら、左上のボタンの並びの 5 つ目（四角に ＋、**Create**）を押し、検索に `Iris` と打つ。
2. **Iris** のタイルを押し、下の **Create** を押す。場面に色の輪が置かれ、下の Timeline に帯が 1 本できる。
   ![2：レシピ Iris を置いた所](docs/media/window_step2.jpg)
3. 輪を選んだまま、ボタンの並びの 2 つ目（きらめき、**Effects**）に替えて `Glow` を探し、押して下の **Add** を押す。輪が光る（右下の Inspector にも効果が出る）。
   ![3：Glow を掛けた所](docs/media/window_step3.jpg)
4. 右上の **Render** を押し、名前（例 `first`）を入れてもう一度 **Render**。書類の隣に `first.mp4` ができ、上に `Written: first.mp4` と出る。
   ![4：書き出した所](docs/media/window_step4.jpg)

この手順は試験としても走ります（`skin/book/integration_test/first_piece_test.dart`。上の絵はこの試験が撮った物。4 手目は、試験では Render の小窓を通さずに同じ書き出しを呼ぶ）。

### 最初の 1 本（コマンドで）

窓を使わずに、レシピで絵を作り、光（Glow）と画面全体の RGB のずれ（`rgbshift.wgsl`）を掛けて、8 秒の mp4 に書き出す例です。`filament/edit.sh` は書類を開いて、1 行ずつの命令を受けます（`help` で一覧）。

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
- `say 0 script …`：物 0（場面そのもの）に画面全体の効果を置く。
- `write …`：mp4 に書き出す。`Movies` に `first.mp4` ができる。

### 窓の見方

- 左：Media と棚（Create はレシピと図形・文字、Effects は効果）。
- 真ん中：Stage（中身が描いた絵）。下は Timeline（物ごとの帯とキー）と Ease（キーの間の動き方）。
- 右：Inspector（選んだ物の値。数字の上で横に引くと変わる）。右上の Render で書き出す。
- 自分の効果やレシピは `~/Library/Application Support/Motolii/` の `effects/`・`recipes/`・`ways/` に置くと、その場で棚に出ます。

言葉の対応（この README の言葉 → ふつうの言い方）：窓 → アプリの画面、中身 → エンジン、物 → レイヤー、場面・Stage → コンポジション、書類 → プロジェクトファイル（`.motolii`）、棚 → 効果やレシピの一覧。

### 困ったら

| 出た事 | 直し方 |
|---|---|
| `flutter: command not found` | Flutter を PATH に置く（flutter.dev の手順）。別の場所なら `FLUTTER=/path/to/flutter skin/book/live.sh` |
| `brew: command not found` | [brew.sh](https://brew.sh/ja/) の 1 行で Homebrew を入れる |
| `xcrun: error` や、Xcode が無いと言われる | App Store で Xcode を入れ、一度開いて追加の部品を入れ、`sudo xcodebuild -license accept` |
| `No space left on device` | 空きを 10 GB 以上にしてから、止まったコマンドをもう一度 |
| `git clone` や `build.sh` がネットの途中で止まった | 同じコマンドをもう一度（取ってきた分は残る）。直らなければ `filament/third_party/` の中の止まった物のフォルダを消してからもう一度 |
| `ld: library 'webgpu_dawn' not found` | `filament/dawn_probe/build.sh` を先に走らせる |
| 窓は開くが Stage が黒い | 中身を組んだか確かめる（`filament/out/libmotolii.dylib` があるか）。無ければ `filament/edit.sh build.motolii < /dev/null` |

それでも止まったら [Issue](https://github.com/oshikaidesu/motolii/issues) に、走らせたコマンドと出た文をそのまま貼ってください（ひな型があります）。使い方の質問は [Discussions](https://github.com/oshikaidesu/motolii/discussions) へ。日本語・英語のどちらでも。

### 新しくする・消す

- 新しくする：`git pull` の後、`filament/dawn_probe/build.sh` と `filament/edit.sh build.motolii < /dev/null` をもう一度（変わった所だけ組み直す）。窓は `skin/book/live.sh` が組み直す。
- 消す：`motolii` のフォルダを消す（取ってきた物と組んだ物は全部この中）。作った書類と書き出した動画は、置いた所に残ります。`brew bundle` で入れた物は `brew uninstall` で。

## 開発に加わる

[`CONTRIBUTING.md`](CONTRIBUTING.md)（[English](CONTRIBUTING.en.md)）にあります（組み立て、試験の走らせ方、決まり、手を付けやすい所）。行動規範は [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)、脆弱性の知らせ方は [`SECURITY.md`](SECURITY.md)。

### 考え方

- **自作は負債。** まず既存の実装を探し、作者のやり方のまま使う。無ければ周りの技術やコミュニティまで探す。
- 他の人の作品はなるべく手を入れずに同梱し、隣に作者のライセンスと出所を置く。手を入れる時は、元に当てる差分（`.patch`）に分けて元は作者の配布元から取ってくるか、ファイルの 1 行目に元と改変を書く。
- 今は自分で書いたコードも多い（窓の Flutter など）。借りられる物が見つかれば、自分のコードを捨てて取り替える。
- コミットごとに、何をどこから借りたか（出典）を書く。

### 並び

| 場所 | 中身 |
|---|---|
| `filament/` | 中身（エンジン）。入口と、今の組み立てが使うファイルの一覧は [`filament/README.md`](filament/README.md) |
| `filament/effects/`・`recipes/`・`animators/`・`ways/` | 同梱の効果・レシピ・動き・イーズ（利用者のフォルダと同じ形） |
| `filament/aviutl_scripts/` | 同梱の AviUtl2 のスクリプト（作者ごと、LICENSE と SOURCE.txt つき） |
| `filament/check/` | 中身の試験（`all.sh` でまとめて） |
| `skin/book/` | 窓（Flutter、macOS）。試験は `test/` と `integration_test/` |
| `thorvg/` | ThorVG を wgpu に載せた前の結合（前の試しのスクリプトが使う） |
| `old/` | 前の土台の実験（今の作りからは使っていない） |
| `docs/` | この README の絵と作例（`media/`・`examples/`）、作者の作業メモ（`notes/`） |

### 文書の地図

| 外向け | 作者の作業メモ（日本語、そのまま公開） |
|---|---|
| `README.md`・`README.en.md`・`ARCHITECTURE.md`・`CONTRIBUTING.md`・`CONTRIBUTING.en.md`・`THIRD_PARTY_NOTICES.md`・`filament/README.md` | `ASSETS.md`（在る物の台帳。コミットの見張りが読むので一番上）・`docs/notes/TASKS.md`（やること）・`docs/notes/AE_RUBRIC.md`（採点表）・`reports/`（調べた記録。絵と動画は他の人のアプリや素材が映る物が多いので git に入れていない） |

## ライセンス

Motolii 自身の物は [Apache License 2.0](LICENSE-APACHE) と [MIT](LICENSE-MIT) のどちらでも、使う人が選べます（ダブルライセンス）。送ってもらった貢献も、特に断りが無ければ同じ二つのライセンスで受けます。同梱した他の人の作品は、それぞれのライセンスに従います（[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)）。

*Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions. Bundled third-party works keep their own licences.*
