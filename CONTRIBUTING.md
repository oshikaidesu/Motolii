# 関わり方

*English version: [CONTRIBUTING.en.md](CONTRIBUTING.en.md). English is fine for issues and pull requests. Contributions are dual licensed under Apache-2.0 or MIT, as the project is. Short version: build with the README, run the checks below, and say in your pull request what you borrowed from where. If adding a line to ASSETS.md is too much, say so in the pull request and the maintainer will add it.*

## できること

- **Issue**：不具合、組み立てで止まった所、「この表現を使いたい」という見本（URL と、何が良いのか）。ひな型があります。
- **Pull Request**：受けます。Motolii は Apache-2.0 と MIT のダブルライセンスで、送ってもらった物も、特に断りが無ければ同じ二つのライセンスで受けます。
- **同梱した作品の作者の方**：載せ方の直し・外してほしい時は Issue で。すぐ直すか外します（[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)）。

## 組み立て

[`README.md`](README.md) の「使ってみる」のとおり。コミットする人は、クローンした所で一度だけ：

```bash
git config core.hooksPath .githooks
```

## 確かめ方

| 何を | コマンド | 通った時 |
|---|---|---|
| 台帳（下の決まり 2） | `python3 .githooks/assets_check.py` | `ASSETS.md covers all … files` |
| 中身の試験（まとめて） | `filament/check/all.sh` | `all checks held`（どれかが NO なら exit 1）。1 本ずつは `filament/check/*.sh` |
| 窓の試験 | `cd skin/book && flutter test` | 中身を組んでいない時は、中身を使う試験が飛ばされる |
| 窓の書き方 | `cd skin/book && flutter analyze lib test integration_test` | エラー 0（今ある注意は 97） |

GitHub の Actions は、台帳の確かめを毎回走らせます（`.github/workflows/assets.yml`、知らせるだけで PR は止めない）。組み立て全部と試験は `.github/workflows/build.yml`（Apple Silicon の Mac で、README の手順どおり組んで `check/all.sh`・`flutter analyze`・`flutter test`。コードに触る PR と、Actions の画面から手で）。窓での最初の 1 本の確かめは `cd skin/book && flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/first_piece_test.dart`（`docs/media/window_step1..4.png` を書く）。作りは [`ARCHITECTURE.md`](ARCHITECTURE.md) と [`filament/README.md`](filament/README.md)。行動規範は [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)、脆弱性は [`SECURITY.md`](SECURITY.md)。

## 決まり

### 1. 作る前に探す
新しく書く前に、[`ASSETS.md`](ASSETS.md)（手元に在る物の台帳）と、ネットの既存の実装を探してください。作者のやり方のまま使える物があれば、それを使います。

### 2. 新しいファイルは台帳に 1 行
`ASSETS.md` は「何が在って、どこまで確かめたか」の表です。コミットの前に `.githooks/assets_check.py` が走り、台帳に場所の無いファイルがあると止まります。新しい物は行を足すか、在る行に場所を足してください（場所は `` ` `` で囲む。フォルダを書けば中身は全部）。**難しければ `git commit --no-verify` で進めて、PR にそう書いてください。取り込む時に足します。**

### 3. コミット（PR）の本文に出典
何をどこから借りたか（URL・版かコミット）と、何を確かめて何がまだかを書きます。例：`git show d46f69cac`。

### 4. 他の人の作品を入れる時
- 作者のフォルダごと、なるべく手を入れずに置く。隣に作者のライセンスの文書（`LICENSE`）と、取ってきた所（`SOURCE.txt`：名前と作者／licence／from／page）。
- 作者がライセンスの文書を出していない物は入れない（入れる代わりに、組み立ての時に作者の所から取る：`filament/midi_roll.sh` が例）。
- 手を入れる物は、元に当てる差分（`.patch`）に分けて、元は組み立ての時に作者の配布元から取る（`filament/dawn_probe/tvg_on_dawn.sh` が例）。
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) に行を足す。

## 手を付けやすい所

| 何 | どこから | 難しさ |
|---|---|---|
| 配れる `.app`：中身が読む Homebrew の部品（GStreamer・glib・SQLite・LuaJIT・QuickJS）を `.app` の中に入れる。[dylibbundler](https://github.com/auriamg/macdylibbundler) か GStreamer の macOS 用フレームワーク | `filament/edit.sh`、`skin/book/macos/` | 中 |
| AviUtl2 のスクリプトで足りない呼び出し（`obj.drawpoly`、図形「円」など）を、手本（AviUtl2 の lua.txt、sashimono-edit）どおりに | `reports/aviutl2/converter_plan.md`、`filament/aviutl.cpp` | 中〜難 |
| TSL の表現を編集の棚に出す | `filament/tsl/`、`reports/tsl_collection.md` | 中 |
| 窓の試験の落ちる 3 本（下の「知っている穴」）を直し、まとめて走らせても通るようにする | `skin/book/test/` | 中 |
| 台帳 `ASSETS.md` の見出しに英語を添える | `ASSETS.md` | 易 |
| 窓の結合テストの出力先（いまは `reports/`）を一時フォルダに | `skin/book/integration_test/` | 易 |

## 知っている穴

- 動くのは macOS（Apple Silicon）だけ。
- 窓の試験（`flutter test`）は、まとめて走らせると落ちる物がある（2026-10-27 に 150 本中 125 本が通る）。1 本ずつなら通る物が多く、それだけで落ちるのは 3 本：Parent（親を付けた後の位置）、anime.js の庫（10/26 に anime.js の見本を外した後、試験が古いまま）、Posterize と Boxinator のつまみ。直すのは手を付けやすい所の 1 つ。CI ではこの段は知らせるだけ。
- 中身（`libmotolii.dylib`）は Homebrew の GStreamer などを読むので、作った `.app` はそのままでは他の Mac で動きません。
- `filament/` の直下には、今の中身と前の試しのファイルが並んでいます（前の試しのスクリプトは `filament/sdk.sh` が取る組み立て済みの Filament を使う）。
