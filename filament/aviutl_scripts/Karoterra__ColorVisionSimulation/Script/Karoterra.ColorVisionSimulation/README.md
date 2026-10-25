# AviUtl2 スクリプト - 色覚シミュレーションKR

![GitHub Release](https://img.shields.io/github/v/release/karoterra/aviutl2-ColorVisionSimulation)
[![AviUtl2 Catalog](https://aviutl2-catalog-badge.sevenc7c.workers.dev/badge/v/Karoterra.ColorVisionSimulation)](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/Karoterra.ColorVisionSimulation)

自分と違う色覚を持つ人はどのように色が見えているのかシミュレーションする
[AviUtl2](http://spring-fragrance.mints.ne.jp/aviutl/) スクリプトです。

[AviUtl1 用の色覚シミュレーションKR](https://github.com/karoterra/aviutl-ColorVisionSimulation) を AviUtl2 用に移植したものです。

## 注意事項

このスクリプトは、P型、D型、T型色覚の人はどのように色が見えているか一般的な色覚の人が体験するためのものです。
ただし、各色覚の人がどのように色が見えているか厳密にシミュレーションするものではありません。
あくまでも、どのような色の区別が困難であったり容易であったりするか確認する程度に留めてください。

## 動作環境

AviUtl ExEdit2 version 2.0beta42 にて動作確認しました。

## 導入方法

### AviUtl2 カタログからインストール

[AviUtl2 カタログ](https://github.com/Neosku/aviutl2-catalog) がセットアップ済みであれば以下のリンクからインストールできます。

[![AviUtl2 Catalog](https://aviutl2-catalog-badge.sevenc7c.workers.dev/badge/v/Karoterra.ColorVisionSimulation)](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/Karoterra.ColorVisionSimulation)

### パッケージファイルからインストール

1. [Releases](https://github.com/karoterra/aviutl2-ColorVisionSimulation/releases/) から最新版のパッケージファイル `.au2pkg.zip` をダウンロードしてください。
2. ダウンロードしたパッケージファイルを AviUtl2 のプレビュー画面にドラッグアンドドロップしてください。

## 使い方

お好きなオブジェクトにアニメーション効果「色覚シミュレーションKR」を適用してください。
フィルタオブジェクトとしても利用できます。

デフォルトでは「色調整」カテゴリの中にあります。

- Type: 色覚の種類
- Severity: シミュレーションの程度
  - 0 -> 変化なし
  - 100 -> 完全に変化

動画で確認したい場合は[AviUtl1のときの紹介動画](https://www.nicovideo.jp/watch/sm44847041)を参照してください。
ただし一部導入手順が異なるのでご注意ください。


## License

このソフトウェアは Unlicense ライセンスのもとで公開されます。
詳細は [LICENSE](LICENSE) を参照してください。

## Credits

### libDaltonLens

このスクリプトは一部 libDaltonLens を参考にして実装しました。

https://github.com/DaltonLens/libDaltonLens

> This is free and unencumbered software released into the public domain.
> 
> Anyone is free to copy, modify, publish, use, compile, sell, or
> distribute this software, either in source code form or as a compiled
> binary, for any purpose, commercial or non-commercial, and by any
> means.
> 
> In jurisdictions that recognize copyright laws, the author or authors
> of this software dedicate any and all copyright interest in the
> software to the public domain. We make this dedication for the benefit
> of the public at large and to the detriment of our heirs and
> successors. We intend this dedication to be an overt act of
> relinquishment in perpetuity of all present and future rights to this
> software under copyright law.
> 
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
> EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
> MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
> IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
> OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
> ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
> OTHER DEALINGS IN THE SOFTWARE.
> 
> For more information, please refer to <https://unlicense.org>

## Change Log

### v1.1.0 (2026-04-19)
- フィルタオブジェクトに対応
- 配布形式を au2pkg に変更

### v1.0.0 (2025-07-09)
- 初版公開
