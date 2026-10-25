# AviUtl2 スクリプト - 油絵KR

![GitHub Release](https://img.shields.io/github/v/release/karoterra/aviutl2-OilPainting)
[![AviUtl2 Catalog](https://aviutl2-catalog-badge.sevenc7c.workers.dev/badge/v/Karoterra.OilPainting)](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/Karoterra.OilPainting)

Kuwahara filter により油絵風の加工を施す
[AviUtl2](http://spring-fragrance.mints.ne.jp/aviutl/) スクリプトです。

[AviUtl1 用の油絵スクリプト](https://github.com/karoterra/aviutl-OilPainting) を AviUtl2 用に移植したものです。

## 動作環境

AviUtl ExEdit2 version 2.0beta1 にて動作確認しました。

## 導入方法

### AviUtl2 カタログからインストール

[AviUtl2 カタログ](https://github.com/Neosku/aviutl2-catalog) がセットアップ済みであれば以下のリンクからインストールできます。

[![AviUtl2 Catalog](https://aviutl2-catalog-badge.sevenc7c.workers.dev/badge/v/Karoterra.OilPainting)](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/Karoterra.OilPainting)

### パッケージファイルからインストール

1. [Releases](https://github.com/karoterra/aviutl2-OilPainting/releases/) から最新版のパッケージファイル `.au2pkg.zip` をダウンロードしてください。
2. ダウンロードしたパッケージファイルを AviUtl2 のプレビュー画面にドラッグアンドドロップしてください。

## 使い方

お好きなオブジェクトにアニメーション効果「油絵KR」を適用してください。
フィルタオブジェクトとしても利用できます。

デフォルトでは「加工」カテゴリの中にあります。

- ぼかし: ぼかしの強さ(0だと変化なし)
- 輝度モード: ピクセルの輝度の計算方法
  - スクリプトの内部処理で輝度を計算する箇所があり、その計算方法を選択します
  - 輝度モードによってアニメーション効果適用後の雰囲気が変わる場合があります。

## License

このソフトウェアは MIT ライセンスのもとで公開されます。
詳細は [LICENSE](LICENSE) を参照してください。

## Credits

このスクリプトは高橋直哉氏の[油絵スクリプト](https://www.nicovideo.jp/watch/sm35695116)の処理速度向上を目的として開発したものです。

## Change Log

### v1.1.0 (2026-04-19)
- フィルタオブジェクトに対応
- 配布形式を au2pkg に変更

### v1.0.0 (2025-07-12)
- 初版公開
