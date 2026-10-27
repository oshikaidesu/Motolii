# 関わり方

*English is fine for issues. The notes below are in Japanese; the short version: build with the README, add a line to ASSETS.md for every new file, and say in each commit what you borrowed from where.*

## 今できること

- **Issue**：不具合、組み立てで止まった所、「この表現を使いたい」という見本（URL と、何が良いのか）。
- **同梱した作品の作者の方**：載せ方の直し・外してほしい時は Issue で。すぐ直すか外します（[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)）。
- **Pull Request は、まだ受けられません。** Motolii 自身のコードのライセンスが決まっていないためです。決まったら、PR は同じライセンスで受ける（inbound = outbound）形にします。

## 組み立て

[`README.md`](README.md) の「組み立て」のとおり。クローンした所で一度だけ `git lfs install` と `git config core.hooksPath .githooks` をしてください。

## 決まり

### 1. 作る前に探す
新しく書く前に、[`ASSETS.md`](ASSETS.md)（手元に在る物の台帳）と、ネットの既存の実装を探してください。「作者のやり方のまま使える物」があれば、それを使います。

### 2. 新しいファイルは台帳に 1 行
コミットの前に `.githooks/assets_check.py` が走り、`ASSETS.md` に場所の無いファイルがあるとコミットが止まります。新しい物は行を足すか、在る行に場所を足してください（場所は `` ` `` で囲む。フォルダを書けば中身は全部）。消した物は台帳からも消します。

### 3. コミットの本文に出典
何をどこから借りたか（URL・版かコミット）、自分で書いた行数、何を確かめて何がまだか、を書きます。例：`git show 94759bc5`。

### 4. 他の人の作品を入れる時
- 作者のフォルダごと、なるべく手を入れずに置く。隣に作者のライセンスの文書（`LICENSE`）と、取ってきた所（`SOURCE.txt`：名前と作者／licence／from／page）。
- 配ってよいと書かれていない物は入れない。
- 手を入れる物は、元に当てる差分（`.patch`）に分けて、元は組み立ての時に作者の配布元から取る（`filament/dawn_probe/tvg_on_dawn.sh` が例）。
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) に行を足す。

## 知っている穴

- 動くのは macOS（Apple Silicon）だけ。
- 窓の結合テスト（`skin/book/integration_test/`）の一部は、作者の Mac の道を出力先にしています。
- 中身（`libmotolii.dylib`）は Homebrew の GStreamer などを `/opt/homebrew` から読むので、作った `.app` はそのままでは他の Mac で動きません。
