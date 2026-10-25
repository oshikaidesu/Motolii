# PixelSorter_K

![GitHub License](https://img.shields.io/github/license/korarei/AviUtl2_PixelSorter_K_Script)
![GitHub Last commit](https://img.shields.io/github/last-commit/korarei/AviUtl2_PixelSorter_K_Script)
![GitHub Downloads](https://img.shields.io/github/downloads/korarei/AviUtl2_PixelSorter_K_Script/total)
![GitHub Release](https://img.shields.io/github/v/release/korarei/AviUtl2_PixelSorter_K_Script)

ピクセルソート (グリッチ表現のひとつ) を行うスクリプト．

[ダウンロードはこちらから．](https://github.com/korarei/AviUtl2_PixelSorter_K_Script/releases)

## 動作確認

- [AviUtl ExEdit2 beta17](https://spring-fragrance.mints.ne.jp/aviutl/)

> [!CAUTION]
> beta17以降必須．

## 導入・削除・更新

初期配置場所は`加工`である．

`オブジェクト追加メニューの設定`から`ラベル`を変更することで任意の場所へ移動可能．

### 導入

1.  同梱の`*.anm2`と`*.mod2`を`%ProgramData%`内の`aviutl2/Script`フォルダまたはその子フォルダに入れる．

`beta4`以降では`aviutl2.exe`と同じ階層内の`data/Script`フォルダ内でも可．

### 削除

1.  導入したものを削除する．

### 更新

1.  導入したものを上書きする．

## 使い方

このスクリプトはメディアオブジェクトに追加することで，対象のピクセルを設定に応じて一方向に昇順で並べる．

- Sort By: ソート対象．以下の7種類から選択可能．

  1.  Luminance (BT.601) (輝度)
  1.  Luminance (BT.709) (輝度)
  1.  Luminance (BT.2020) (輝度)
  1.  RGB Average (RGB平均)
  1.  RGB Product (RGB乗算)
  1.  RGB Minimum (RGB最小)
  1.  RGB Maximum (RGB最大)

- Sort From: ソート開始値．`Sort By`で指定した値がこれより小さいピクセルはソート対象外．

- Sort To: ソート終了値．`Sort By`で指定した値がこれより大きいピクセルはソート対象外．

- Direction: ソート方向．角度 (deg) で指定する．

- Color Space: 入力画像のカラースペース．以下の2種類から選択可能．

  1.  Linear (入力画像をそのまま計算する)
  1.  sRGB (ガンマ補正を解除して計算する)

- Mask: ソート部分のみ描画する．

- PI: パラメータインジェクション．

```lua
{
  sort_by = 1,
  sort_from = 64.0,
  sort_to = 128.0,
  dir = 0.0,
  col_space = 1,
  mask = 0 -- booleanでも可
}
```

`{}`は挿入済みであるため省略して入力する．

## スクリプトモジュール

### sort 関数

メタデータバッファに基づいて画像バッファを横方向昇順にピクセルソートする．

メタデータバッファは`R`にソート対象の値，`A`にソート対象かどうかを入れる．

|位置|名前|型|説明|
|:---|:---:|:---:|:---|
|引数 #1|`px`|userdata|画像バッファ|
|引数 #2|`meta`|userdata|メタデータバッファ|
|引数 #3|`w`|number|画像，メタデータの横幅|
|引数 #4|`h`|number|画像，メタデータの縦幅|
|引数 #5|`mask`|number|マスクフラグ|

## License

LICENSEファイルに記載．

## Credits

### AviUtl ExEdit2 Plugin SDK

https://spring-fragrance.mints.ne.jp/aviutl/

---

The MIT License

Copyright (c) 2025 Kenkun

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

## Change Log
- **v1.0.1**
  - `get_param_boolean`を使用するように変更．
  - `Direction`変更時見切れる可能性があった問題を修正．

- **v1.0.0**
  - Release
