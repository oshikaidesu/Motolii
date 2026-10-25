# ReflectionOnPane

<details>
<summary><strong>日本語 - クリックして表示</strong></summary>

## 概要

**ReflectionOnPane** は、画像や動画をガラスの反射・映り込みのように合成する AviUtl2 用アニメーション効果です。  
色調補正、露光、スクリーン合成、左右反転、ぼかし、位置をずらした二重反射、エッジ減衰を一つの効果として設定できます。

暗い部分を目立たせず、照明、看板、人物などの明るい部分を中心に背景へ重ねる用途を想定しています。

ReflectionOnPane は、フィルタ効果追加メニューの **「加工」** から選択できます。

## インストール方法

1. `ReflectionOnPane_v.x.x.x.au2pkg.zip` を解凍せず、AviUtl2 のプレビュー画面へ直接ドラッグ＆ドロップします。
2. 表示された内容を確認してインストールします。
3. 反射として使用する画像または動画へアニメーション効果を追加し、`ReflectionOnPane` を選択します。

パッケージにはスクリプト本体と、英語・簡体中国語表示用の言語ファイルが含まれています。  
表示言語は AviUtl2 の言語設定に合わせて自動的に切り替わります。

## 基本操作

背景を下のレイヤーへ、反射として使用する画像または動画を上のレイヤーへ配置してください。  
ReflectionOnPane は反射素材を調整した後、スクリーンモードで背景へ合成します。

### 露光・色調補正

露光は反射素材の RGB を乗算し、暗い素材の反射を強めます。`100` が等倍です。  
コントラスト、ガンマ、彩度、色相は AviUtl2 標準の「色調補正」へ渡されます。

初期値は、露光 `100`、コントラスト `150`、ガンマ `-100`、彩度 `100`、色相 `0` です。

### 反射強度

反射全体の不透明度を設定します。初期値は `65` です。  
明るすぎる場合は、色調補正より先に反射強度を下げると調整しやすくなります。

### ぼかし

反射素材をぼかし、ガラス面から少し離れた像や焦点の外れた映り込みを表現します。初期値は `3` です。

### 重影

Xずれ（重影）または Yずれ（重影）を 0 以外にすると、元の反射を残したまま、指定位置へ 50% の二重反射を追加します。  
両方とも 0 の場合、重影は描画されません。

### エッジ減衰

エッジ方向で、なし、左から右、右から左、上から下、下から上を選択できます。  
エッジ範囲は減衰が画像へ入る距離、エッジ柔らかさは減衰曲線を設定します。

### ミラー

有効にすると、反射素材を Y 軸で 180 度回転し、左右反転します。

## 注意事項

- スクリーン合成では黒が透明に近い働きをするため、非常に暗い素材は見えにくい場合があります。
- 明るい人物と暗い背景が同居する素材では、一つの全体調整だけで両方を適正化できない場合があります。
- ぼかしと重影を併用すると GPU 負荷が増加します。
- 後段に別の描画系効果を追加した場合、重影の描画順が変わることがあります。

</details>

<details>
<summary><strong>中文 - 点击查看</strong></summary>

## 概要

**ReflectionOnPane** 是用于 AviUtl2 的玻璃反射动画效果。  
它将色调校正、曝光、滤色合成、左右镜像、模糊、偏移重影和边缘衰减整合在一个效果中。

它主要用于压低反射素材中的暗区，并将灯光、招牌和人物等较亮内容叠加到背景上。

ReflectionOnPane 可以从滤镜效果菜单的 **“加工”** 分类中选择。

## 安装方法

1. 不要解压 `ReflectionOnPane_v.x.x.x.au2pkg.zip`，直接拖放到 AviUtl2 的预览画面。
2. 确认显示内容后进行安装。
3. 为用作反射的图片或视频添加动画效果，然后选择 `ReflectionOnPane`。

安装包包含脚本本体，以及英文和简体中文界面所需的语言文件。  
显示语言会根据 AviUtl2 的语言设置自动切换。

## 基本操作

将背景放在下方图层，将用作反射的图片或视频放在上方图层。  
ReflectionOnPane 会先处理反射素材，再通过滤色（Screen）模式与背景合成。

### 曝光与色调校正

曝光以乘法增强反射素材的 RGB；`100` 表示原始强度。  
对比度、伽马、饱和度和色相会传递给 AviUtl2 标准“色調補正”。

默认值为曝光 `100`、对比度 `150`、伽马 `-100`、饱和度 `100`、色相 `0`。

### 反射强度

控制整个反射层的不透明度，默认值为 `65`。效果过亮时，通常先降低反射强度更容易调整。

### 模糊

模糊反射素材，用于表现焦点之外或与玻璃表面存在距离的反射，默认值为 `3`。

### 重影

将重影偏移 X 或 Y 设为非零后，会保留主反射，并在指定偏移处增加一层 50% 强度的反射。  
两个偏移都为 0 时不会绘制重影。

### 边缘衰减

边缘方向可以选择关闭、从左到右、从右到左、从上到下或从下到上。  
边缘范围控制衰减进入图像的距离，边缘柔和度控制衰减曲线。

### 镜像

启用后会将反射素材绕 Y 轴旋转 180 度，实现左右镜像。

## 注意事项

- 滤色合成中的黑色接近透明，因此极暗素材可能不明显。
- 当素材同时包含明亮人物和极暗背景时，一组全局参数可能无法同时得到理想曝光。
- 同时使用较高模糊和重影会增加 GPU 负载。
- 在该效果后方添加其他绘制类效果，可能改变重影的处理顺序。

</details>

<details>
<summary><strong>English - Click to view</strong></summary>

## Overview

**ReflectionOnPane** is an AviUtl2 animation effect for compositing an image or video as a reflection on glass.  
It combines color correction, exposure, Screen blending, horizontal mirroring, blur, an offset ghost reflection, and directional edge fading.

It is intended to suppress unobtrusive dark areas while reflecting brighter content such as lights, signs, and people over a background.

ReflectionOnPane is available under **Processing (加工)** in the filter-effect menu.

## Installation

1. Do not extract `ReflectionOnPane_v.x.x.x.au2pkg.zip`. Drag and drop it directly onto the AviUtl2 preview window.
2. Review the displayed contents and install the package.
3. Add an animation effect to the image or video used as the reflection, then select `ReflectionOnPane`.

The package includes the script and the language files required for English and Simplified Chinese UI text.  
The displayed language follows the AviUtl2 language setting automatically.

## Basic Usage

Place the background on a lower layer and the image or video used as the reflection on a layer above it.  
ReflectionOnPane processes the reflection source and composites it over the background with Screen blending.

### Exposure and Color Correction

Exposure multiplies the RGB values of the reflection source; `100` is unity gain.  
Contrast, Gamma, Saturation, and Hue are passed to AviUtl2's standard Color Correction effect.

Defaults are Exposure `100`, Contrast `150`, Gamma `-100`, Saturation `100`, and Hue `0`.

### Reflection Strength

Controls the overall opacity of the reflection. The default is `65`. Reducing Reflection Strength is usually the easiest first adjustment when the result is too bright.

### Blur

Softens the reflection to simulate an out-of-focus image or a reflection separated from the glass surface. The default is `3`.

### Ghost Reflection

Setting Ghost Offset X or Y to a non-zero value preserves the main reflection and adds a second reflection at 50% strength.  
No ghost is drawn when both offsets are zero.

### Edge Fade

Edge Direction supports None, Left to Right, Right to Left, Top to Bottom, and Bottom to Top.  
Edge Range controls how far the fade extends into the image, while Edge Softness controls its curve.

### Mirror

When enabled, the reflection source is rotated 180 degrees around the Y axis to create a horizontal mirror image.

## Notes

- Black contributes almost nothing in Screen blending, so extremely dark sources can be difficult to see.
- A single global adjustment may not balance a bright subject and a very dark background in the same source.
- Combining high blur values with the ghost reflection increases GPU load.
- Adding another drawing effect after ReflectionOnPane may change the ghost-reflection processing order.

</details>
