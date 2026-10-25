# LongShadowN



3種類のロングシャドウを作成できるスクリプト

![IMAGE](image.png)

## 最新 / Latest

**r14**

## 後続効果について

`Following Effects` → `Apply To` で、LongShadowN の後に置いた効果の適用先を選べます。  
初期値の `Whole` は影と元オブジェクトの全体に適用します。  
`Shadow Only` は影だけに適用し、その後で元オブジェクトを描画します。

## 大きい影で描画されない場合

`invalid buffer type` が出る場合は、AviUtl2 の一時画像キャッシュが不足している可能性があります。  
`system.conf` の `[Config]` にある `TemporaryImageCacheSize` を増やし、AviUtl2 を再起動してください。  
極端に大きい影は、設定を増やしても描画できない場合があります。

## 変更履歴 / Change log

- r14
  - Softnessのかかり方を調整
    - Texture1DやLengthが長いときに歪になることがあります
  - 後続効果の適用先を `Whole` / `Shadow Only` から選べるように

- r13
  - パラメータの名称を整えた

- r12
  - ほとんど効果を得られないため、`Supersampling`を削除

- r11
  - 問題がありすぎたので`Blur Shadow`パラメータを削除
  - Directional時限定で`Softness`パラメータを追加
    - Texture1DやLengthが長いときに歪になることがあります
  - TextureグループのOpacityを削除(存在価値が不明)
  - FadeInの数値適応のしかたを反転(0でなし)
  - r8の方式変更により必要なくなったため`Post Smooth`チェックボックスを削除

- r10
  - r9で修正しきれなかったアーティファクトを修正

- r9
  - 影のない方向に縁のようなアーティファクトが発生する問題を修正

- r8
  - 動作方式そのものを変更
  - 軽量化
  - `Inverse Radial`で発生していたバグを修正
  - 元オブジェクトのアンチエイリアス部分がおかしい問題の修正

- r7
  - TextureグループのScale,ScaleX/Yの範囲上限を`4000`に変更

- r6
  - `Blur Shadow` を、影の根元から離れるほどボケ量が強くなる方式に変更
  - `Blur Shadow` の範囲を `0–4000`、刻みを `0.1` に変更

- r5
  - 一部パラメータの有効小数点を変更

- r4
  - 特定状況でしか動作しない一部パラメータが隠されない問題の修正

- r3
  - 一部グループの初期状態をOpenに変更

- r2
  - `Object Color` の `Mix / Opacity` が反映されない問題を修正
  - `Shadow::Opacity` が元オブジェクトの透明度に影響する問題を修正
  - `Fade In / Fade Out` の範囲を `0–100` に変更
- r1
  - 初版
