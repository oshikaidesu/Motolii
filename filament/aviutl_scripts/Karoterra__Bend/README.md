# AviUtl2 スクリプト - 曲げKR

画像を曲線に沿って曲げる
[AviUtl2](http://spring-fragrance.mints.ne.jp/aviutl/) スクリプトです。

[AviUtl1 用の曲げKR](https://github.com/karoterra/aviutl-Bend) を AviUtl2 用に移植したものです。

## 動作環境

AviUtl ExEdit2 version 2.0beta7 にて動作確認しました。

## 導入方法

1. [Releases](https://github.com/karoterra/aviutl2-Bend/releases/) から最新版の ZIP ファイルをダウンロードしてください。
2. ZIP ファイルを展開し、以下のファイルを `C:\ProgramData\aviutl2\Script\` または `C:\ProgramData\aviutl2\Script\<任意の名前>\` に配置してください。
   - `KaroterraBend.lua`
   - `@曲げKR.anm2`

> [!NOTE]
> `aviutl2.exe` と同じ階層に `data` フォルダを作っている場合はそちらに配置してください。

## 使い方

お好きなオブジェクトにアニメーション効果「円弧@曲げKR」または「ベジェ曲線@曲げKR」を適用してください。
デフォルトでは「変形」カテゴリの中にあります。
使い方はスクリプトファイル内のコメントや[AviUtl1のときの紹介動画](https://www.nicovideo.jp/watch/sm39711773)を参照してください。

## License

このソフトウェアは MIT ライセンスのもとで公開されます。
詳細は [LICENSE](LICENSE) を参照してください。

## Change Log

### v1.0.1 (2025-08-19)
- drawpolyの効率化による描画速度向上 ([#1](https://github.com/karoterra/aviutl2-Bend/pull/1))

### v1.0.0 (2025-07-09)
- 初版公開
