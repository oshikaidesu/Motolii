# Pixelizer

下の言語をクリックすると説明を展開できます。  
点击下面的语言即可展开说明。  
Click a language below to expand the documentation.

<details>
<summary><strong>日本語 - クリックして表示</strong></summary>

## 概要

**Pixelizer** は、AviUtl2 用のリアルタイムドット絵化スクリプトです。  
画像や立ち絵などを、色数や階調を抑えたドット絵風の見た目に加工できます。

ピクセルサイズ、色数、階調数、ディザリング、エッジなどを調整できるほか、  
GameBoy、16-bit、4階調グレースケールのプリセットパレットを使用できます。

また、ドットをオブジェクトに追従させる「オブジェクト固定」と、  
オブジェクトを回転してもドットの向きを画面基準に保つ「画面固定」を切り替えられます。

Pixelizer は、エフェクトメニューの **「加工」** にあります。

## インストール方法

1. `Pixelizer_v1.0.0.au2pkg.zip` を AviUtl2 のプレビュー画面へドラッグ＆ドロップします。
2. 表示される内容を確認してインストールします。

パッケージにはスクリプト本体と、日本語・英語・簡体中国語の表示に必要なファイルが含まれています。  
表示言語は AviUtl2 の言語設定に合わせて切り替わります。

## 基本操作

### 拡張設定

使用するパレット、エッジのスタイル、ドットの固定方法を設定します。

パレットはデフォルトの色処理に加えて、GameBoy、16-bit、4階調グレースケールを選択できます。  
ドット固定を「画面固定」にすると、オブジェクトを回転してもドットの向きが画面に対して固定されます。

### ベース

ドット絵化の基本となる設定です。  
ピクセルサイズでドットの大きさを、色数で使用する色の細かさを、階調数で明るさの段階数を調整します。

階調数を無効にすると、明るさの段階化を行わずに色数のみを制限できます。

### ディザリング

限られた色数の中で中間色を表現するためのディザリングを追加します。  
なし、2x2、4x4から選択でき、強度も調整できます。

### エッジ

色の境界や輪郭を強調します。  
拡張設定のエッジスタイルと組み合わせて、柔らかい強調と、よりはっきりしたエッジ強調を切り替えられます。

### 透過保護

透過画像の輪郭付近で色がにじんだり、不自然な縁が出たりするのを抑えるための機能です。  
立ち絵や透過PNGなどをドット絵化するときに使用します。

</details>

<details>
<summary><strong>中文 - 点击查看</strong></summary>

## 概要

**Pixelizer** 是用于 AviUtl2 的实时像素化脚本。  
可以将图片、立绘等对象加工成限制颜色与明暗层次的像素画风格。

可以调整像素大小、颜色等级、色阶数、抖动、轮廓等参数，  
并可以使用 GameBoy、16位色、灰阶4色等预设调色板。

像素固定方式可以在“对象固定”和“屏幕固定”之间切换。  
使用“屏幕固定”时，即使旋转对象，像素格的方向也会保持以画面为基准。

Pixelizer 可以从效果菜单的 **“加工”** 分类中找到。

## 安装方法

1. 将 `Pixelizer_v1.0.0.au2pkg.zip` 拖放到 AviUtl2 的预览画面中。
2. 确认显示的内容后进行安装。

安装包中包含脚本本体，以及日语、英语和简体中文显示所需的文件。  
显示语言会根据 AviUtl2 的语言设置自动切换。

## 基本操作

### 扩展设置

设置调色板、轮廓风格以及像素的固定方式。

除了默认的颜色处理之外，还可以选择 GameBoy、16位色和灰阶4色调色板。  
将像素固定设为“屏幕固定”后，即使旋转对象，像素格的方向也会保持固定在画面上。

### 基础

用于调整像素化的基本效果。  
可以通过像素大小调整像素块尺寸，通过颜色等级控制使用颜色的细致程度，通过色阶数限制亮度层次。

关闭色阶数后，只限制颜色数量，不对亮度进行额外的分级处理。

### 抖动

通过抖动在有限的颜色数量中表现中间色。  
可以选择关闭、2×2、4×4，并调整抖动强度。

### 轮廓

增强颜色边界与对象轮廓。  
配合扩展设置中的轮廓风格，可以在较柔和的增强与更明显的强描边之间切换。

### 透明保护

用于减少透明图片边缘在像素化后出现的颜色渗出或不自然边缘。  
处理立绘、透明PNG等素材时可以使用。

</details>

<details>
<summary><strong>English - Click to view</strong></summary>

## Overview

**Pixelizer** is a real-time pixel-art stylization script for AviUtl2.  
It can process images, character illustrations, and other objects into a pixel-art look with reduced colors and tone levels.

You can adjust pixel size, color levels, tone steps, dithering, and edges,  
and use GameBoy, 16-bit, and 4-tone grayscale preset palettes.

The pixel grid can be locked either to the object or to the screen.  
With Screen Locked enabled, the pixel grid remains aligned to the screen even when the object is rotated.

Pixelizer can be found under the **Processing (加工)** category in the effect menu.

## Installation

1. Drag and drop `Pixelizer_v1.0.0.au2pkg.zip` onto the AviUtl2 preview window.
2. Review the displayed information and install the package.

The package includes the script itself and the files required for Japanese, English, and Simplified Chinese UI text.  
The displayed language follows the AviUtl2 language setting.

## Basic Usage

### Advanced

Configures the palette, edge style, and pixel-grid locking mode.

In addition to the default color processing, you can choose GameBoy, 16-bit, or 4-tone grayscale.  
Screen Locked keeps the pixel grid aligned to the screen even when the object is rotated.

### Basic

Controls the main pixel-art conversion settings.  
Pixel Size changes the size of the pixel blocks, Color Levels controls the number of available color steps, and Tone Steps limits the number of brightness levels.

Disabling Tone Steps limits the colors without applying additional brightness quantization.

### Dithering

Adds dithering to represent intermediate colors with a limited palette.  
You can choose Off, 2×2, or 4×4 and adjust the dithering strength.

### Edge

Enhances color boundaries and object contours.  
Together with Edge Style in Advanced settings, it can produce either softer enhancement or a more pronounced outline.

### Alpha Guard

Helps reduce color bleeding and unnatural edges around transparent areas after pixelization.  
It is useful for character illustrations, transparent PNGs, and similar assets.

</details>
