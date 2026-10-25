# Diffusion_K

![GitHub License](https://img.shields.io/github/license/korarei/AviUtl2_Diffusion_K_Script)
![GitHub Last commit](https://img.shields.io/github/last-commit/korarei/AviUtl2_Diffusion_K_Script)
![GitHub Downloads](https://img.shields.io/github/downloads/korarei/AviUtl2_Diffusion_K_Script/total)
![GitHub Release](https://img.shields.io/github/v/release/korarei/AviUtl2_Diffusion_K_Script)

拡散系エフェクトスクリプト．

[ダウンロードはこちらから](https://github.com/korarei/AviUtl2_Diffusion_K_Script/releases)

## 動作確認

- [AviUtl ExEdit2 beta43b](https://spring-fragrance.mints.ne.jp/aviutl/)

> [!CAUTION]
> beta41以降必須．

## 導入・更新・削除

初期ラベルは`光効果`，`ぼかし`，`加工`である．

`オブジェクト追加メニューの設定`からラベルを変更することで任意の場所に移動可能．

### 導入・更新

ダウンロードした`*.au2pkg.zip`をAviUtl2にD&D．

手動で導入する場合は，`*.anm2`をAviUtl2が認識する場所に設置．

### 削除

パッケージ情報からアンインストールする．

手動で導入した場合は設置した`*.anm2`を削除する．

## 使い方

### Orton

初期ラベル: `光効果`

オートン効果の機能拡張版エフェクト．

#### パラメータ

- Intensity: オートン効果の強さ
- Blurriness: ぼかしの強さ
- Exposure: 露出
- Invert: マスクの反転
- Low: マスクの閾値 (下)
- High: マスクの閾値 (上)
- Softness: マスクの柔らかさ
- Mask::Source: マスクのソース
  - Image: 画像
  - Layer: レイヤー
- Mask::Channel: マスクチャンネル
  - Luminance: 輝度
  - Alpha: アルファ
- Mask::Image: マスク画像
- Mask::Layer: マスクレイヤー
- Brightness: マスク処理後の輝度
- Contrast: マスク処理後のコントラスト
- Rotation: ぼかしの回転
- Scale::X: ぼかしの拡大率X
- Scale::Y: ぼかしの拡大率Y
- Aberration::Channels: 色収差のチャンネル
  - Red & Green: RチャンネルとGチャンネルの色収差
  - Red & Blue: RチャンネルとBチャンネルの色収差
  - Green & Blue: GチャンネルとBチャンネルの色収差
- Aberration::Offset: 色収差のオフセット
- LoCA::Enable: LoCAの有効化
- LoCA::Red: Rチャンネルのぼかし拡大率
- LoCA::Green: Gチャンネルのぼかし拡大率
- LoCA::Blue: Bチャンネルのぼかし拡大率
- LaCA::Enable: LaCAの有効化
- LaCA::Red: Rチャンネルの拡大率
- LaCA::Green: Gチャンネルの拡大率
- LaCA::Blue: Bチャンネルの拡大率
- Blend Mode: 合成モード (暗くするものと明るくするものを選択可能)
  - Replace: 置換
  - Normal: 通常合成
  - Darken: 比較 (暗)
  - Multiply: 乗算
  - Color Burn: 焼き込み (カラー)
  - Linear Burn: 焼き込み (リニア)
  - Darker Color: カラー比較 (暗)
  - Lighten: 比較 (明)
  - Screen: スクリーン
  - Color Dodge: 覆い焼き (カラー)
  - Linear Dodge (Add): 覆い焼き (リニア) - 加算
  - Lighter Color: カラー比較 (明)
- Alpha Mode: アルファモード
  - Alpha Blending: アルファブレンド
  - Alpha Hashed: アルファハッシュ
- Clamp: 描画結果を0-1に飽和
- Glow Only: 光成分のみ表示
- Gamma: 入力画像のガンマ値
- Layer Reference: レイヤー参照
  - Absolute: 絶対参照
  - Relative: 相対参照

> [!NOTE]
> - `Contrast`は`-100.00`より小さい値に設定するとネガポジ反転する．
> - `LoCA::Enable`を有効にするとストレートアルファの都合上，やや暗くなる．
> - `Gamma`は既にリニアな画像に対して適用する場合は`1.0`に設定する．
> - マスク画像はガンマ補正などはされない．
> - 輝度はAEの輝度計算式 (Luminosity) で採用されている (0.3, 0.59, 0.11) を使用している．

### GaussianBlur

初期ラベル: `ぼかし`

ガウシアンブラーエフェクト．

#### パラメータ

- Blurriness: ぼかしの強さ
- Dimensions: ぼかしの方向
  - Horizontal: 水平方向
  - Vertical: 垂直方向
  - Horizontal and Vertical: 水平・垂直方向
- Resize: ぼかした画像のサイズを調整 (フィルタオブジェクトでは無視)

### ChannelBlur

初期ラベル: `ぼかし`

チャンネルごとにガウシアンブラーを適用するエフェクト．

#### パラメータ

- Blurriness::Red: Rチャンネルのぼかしの強さ
- Blurriness::Green: Gチャンネルのぼかしの強さ
- Blurriness::Blue: Bチャンネルのぼかしの強さ
- Blurriness::Alpha: Aチャンネルのぼかしの強さ
- Dimensions: ぼかしの方向
  - Horizontal: 水平方向
  - Vertical: 垂直方向
  - Horizontal and Vertical: 水平・垂直方向
- Resize: ぼかした画像のサイズを調整 (フィルタオブジェクトでは無視)

### Dissolve

初期ラベル: `加工`

ディザ合成エフェクト．

#### パラメータ

- Mode: ディザ合成の種類
  - Dissolve: ディザ合成
  - Dancing Dissolve: ダイナミックディザ合成

### Scatter

初期ラベル: `加工`

画像のピクセルを拡散させるエフェクト．

#### パラメータ

- Amount: 拡散の強さ
- Grain: 拡散の方向
  - Horizontal: 水平方向
  - Vertical: 垂直方向
  - Both: 水平・垂直方向
- Seed: シード値
- Resize: 拡散した画像のサイズを調整 (フィルタオブジェクトでは無視)

### Noise

初期ラベル: `加工`

ノイズエフェクト．

#### パラメータ

- Amount: ノイズの強さ
- Color: ノイズの色
  - BW: 白黒ノイズ
  - RGB: RGBノイズ
  - RGBA: RGBAノイズ
- Seed: シード値
- Clamp: 描画結果を0-1に飽和

> [!NOTE]
> `Color`を`RGBA`に設定した場合，`Clamp`有効無効にかかわらずアルファ値は0-1に飽和する．

### Noise(HSLA)

初期ラベル: `加工`

HSLAノイズエフェクト．

#### パラメータ

- Amount::Hue: Hチャンネルのノイズの強さ
- Amount::Saturation: Sチャンネルのノイズの強さ
- Amount::Lightness: Lチャンネルのノイズの強さ
- Amount::Alpha: Aチャンネルのノイズの強さ
- Seed: シード値

### 共通項目

- PI: パラメータインジェクション

パラメータ値を上書きする．以下のようにエイリアス名と値で記述する．

```lua
{ ["Amount::Hue"] = 10.0 }
```

`{}`は既に入力済みである．

## ビルド方法

[リリース用ワークフロー](./.github/workflows/releaser.yml)を参照されたい．

## ライセンス

本プログラムのライセンスは[LICENSE](./LICENSE)を参照されたい．

また，本プログラムが利用するサードパーティ製ライブラリ等のライセンス情報は[THIRD_PARTY_LICENSES](./THIRD_PARTY_LICENSES.md)に記載している．

## 更新履歴

[CHANGELOG](./CHANGELOG.md)を参照されたい．
