# Motolii

**ネットで見つけたかっこいい表現を、自分の動画にすぐ使う。** 新しい表現のために、開発者の対応を待たない。

Motolii は MV（絵が主役のモーショングラフィックス）を作る道具です。Filament（Dawn＝WebGPU の上）と ThorVG の上に薄い皮を張り、2D と 3D を一つの世界で扱います。触り方はまず Alight Motion のふるまいを全部なぞり、そこから先へ行きます。

*Motolii is a motion-graphics tool for music videos: use a cool look you found on the net in your own video right away, without waiting for a developer. A thin skin over Filament (on Dawn/WebGPU) and ThorVG, 2D and 3D in one world.*

- 状態：作っている途中（リリースはまだ）。動くのは macOS（Apple Silicon）だけ。Windows はこれから。
- 何が在って何が動くかの台帳：[`ASSETS.md`](ASSETS.md)（作る前にここを見る）
- やること：[`TASKS.md`](TASKS.md)
- 同梱した他の人の作品と、そのライセンス：[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
- 関わり方：[`CONTRIBUTING.md`](CONTRIBUTING.md)

## 考え方

- **自作は負債。** まず既存の実装を探し、作者のやり方のまま使う。無ければ周りの技術やコミュニティまで探す。
- 他の人の作品はなるべく手を入れずに同梱し、隣に作者のライセンスと出所を置く。手を入れる時は、元に当てる差分（`.patch`）に分けて元は作者の配布元から取ってくるか、ファイルの 1 行目に元と改変を書く。
- 今は自分で書いたコードも多い（窓の Flutter など）。借りられる物が見つかれば、自分のコードを捨てて取り替える。
- コミットごとに、何をどこから借りたか（出典）と、自分で書いた行数を書く。
- 書いた行数は成果ではなく負債。

## 並び

| 場所 | 中身 |
|---|---|
| `filament/` | 中身（エンジン）：描く・動かす・書類（SQLite）・動画（GStreamer）・効果・物理。C++ / Objective-C++。組み立てのスクリプトもここ |
| `filament/aviutl_scripts/` | 同梱した AviUtl2 のスクリプト 180 パッケージ（作者ごと、手を入れずに） |
| `filament/effects/` | 効果のファイル（ISF・WGSL・PlayCanvas の点群の効果） |
| `filament/third_party/` | 取ってくる依存の置き場（git の外）。一部の小さい物だけ同梱 |
| `skin/book/` | 窓（Flutter、macOS）。操作だけで、絵は中身が描く |
| `reports/` | 調べた記録・測った数字・見本の絵と動画 |
| `thorvg/` | ThorVG を wgpu に載せた前の結合（組み立てのスクリプトが一部を使う） |
| `old/` | 前の土台の実験（今の作りからは使っていない） |
| `.githooks/` | コミットの前に `ASSETS.md` が正しいか確かめる見張り、Git LFS のフック |

## 組み立て（macOS）

1. Xcode と、Homebrew の物を入れる。
   ```bash
   brew bundle
   ```
2. Flutter 3.47.2（stable）を入れて PATH に置く（別の場所なら `FLUTTER=…/bin/flutter`）。
3. クローンした所で一度だけ：
   ```bash
   git lfs install
   git config core.hooksPath .githooks
   ```
4. Filament（WebGPU）を組む。初回は Filament を取ってきて差分を当て、組んで入れる（M4 で約 5 分）。
   ```bash
   filament/dawn_probe/build.sh
   ```
5. 中身を組む。ThorVG・Skia・libtess2・Jolt は初回にそれぞれのスクリプトが取ってきて組む。
   ```bash
   filament/edit.sh out/first.motolii < /dev/null
   ```
6. 窓を開く。
   ```bash
   skin/book/live.sh
   ```

取ってくる物とその版は [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) の 2 節にあります。

## ライセンス

Motolii 自身のコードのライセンスは**まだ決めていません**。同梱した他の人の作品は、それぞれのライセンスに従います（[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)）。
