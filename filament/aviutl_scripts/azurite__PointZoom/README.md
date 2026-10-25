# AviUtl2 ポイントズーム

アンカーで指定したポイントにズームする [AviUtl2](https://spring-fragrance.mints.ne.jp/aviutl/) 用スクリプトです。

## 動作環境

- [AviUtl ExEdit2](https://spring-fragrance.mints.ne.jp/aviutl/)  
`beta18` 以降必須（`beta40a` で動作確認済み）。

## 導入方法

次のいずれかの方法でインストールできます。

### AviUtl2 カタログを使う（推奨）

本スクリプトは [aviutl2-catalog](https://github.com/Neosku/aviutl2-catalog) に登録済みです。
メインメニュー ＞ パッケージ一覧 ＞ スクリプト ＞ ポイントズーム からインストールしてください。

### 手動インストール

[Releases](https://github.com/azurite581/AviUtl2-PointZoom/releases) から `PointZoom_v{version}.au2pkg.zip` をダウンロードし、AviUtl2 のプレビューにドラッグ&ドロップしてください。

## 使い方

1. フィルタ効果を追加 → 変形 → ポイントズームを選択します。
2. ズームしたい箇所にアンカーを移動させ、拡大率を変更してください。

## パラメーター

### ズーム

#### 拡大率

  拡大率。上限は 10000 です。

#### 補間なし

  チェックを入れると補間​しなくなります。

### 目標座標

#### X

  ズームする X 座標。

#### Y

  ズームする Y 座標。

### 領域外

#### モード

  縮小したときに生じる画像の領域外のサンプリングーモードを指定します。

- 指定色：指定した色
- 引き伸ばし：境界の​色
- ループ：元画像と同じ向きでループ
- ミラー：元画像を​反転しながらループ

#### 色

[`領域外`](#領域外-1) が `指定色` のときの色。

#### 透明度

領域外の透明度。

### PI

パラメーターインジェクション用の入力欄です。以下の形式に沿って値を入力することで、各種パラメーターの値を上書きできます（実際に入力するときは波括弧は不要です）。
範囲を超える値が入力された場合は自動で丸められます。

```lua
{ zoom, no_interp, x, y, mode, color, transparency }
```

| | 説明 | 型 | 範囲 |
| :--- | :--- | :--- | :--- |
| zoom | 拡大率 | number | [0, 10000] |
| no_interp | 補間なしを有効にするかどうか | number または boolean | [0, 1] または false/true |
| x | ズームする X 座標 | number | |
| y | ズームする Y 座標 | number | |
| mode | 領域外のサンプリングモード | number | [1, 4] |
| color | 領域外の色（16進数カラーコード） | number | |
| transparency | 領域外の透明度 | number | [0, 100] |

## ライセンス

[MIT License](LICENSE.txt) に基づくものとします。

## クレジット

### 使用したツール

</details>

#### [aviutl2-cli](https://github.com/sevenc-nanashi/aviutl2-cli)

<details>
<summary>MIT License</summary>

```text
MIT License

Copyright (c) 2026 Nanashi. <sevenc7c.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

</details>

## 更新履歴

[CHANGELOG](CHANGELOG.md) をご確認ください。
