# FilmPrintLUT



<details open>
<summary>日本語　▲ 🌐 Switch Language</summary>

## 概要

**FilmPrintLUT** は、`.cube` 形式の 3D LUT を直接読み込み、AviUtl2 上の映像や画像の色味を調整するスクリプトです。

LUT ファイルを選び、強度を調整するだけで使用できます。設定項目は **CUBEファイル・強度・アルファを保持** の3つです。

**このスクリプトは、基本的に `.cube` 形式の LUT ファイルを読み込むためのものです。**
オリジナル LUT を3種類同梱していますが、お手持ちの LUT を使用することをおすすめします。

## インストール方法

1. `FilmPrintLUT\\\_v1.2.0.au2pkg.zip` を **解凍せず、AviUtl2 のプレビュー画面へ直接ドラッグ＆ドロップ** します。
2. 表示された内容を確認してインストールします。再起動を求められた場合は、AviUtl2 を再起動してください。
3. 対象のオブジェクトへ効果を追加し、**色調整 → FilmPrintLUT** を選択します。

パッケージにはスクリプト本体、LUT 読み込み用モジュール、3種類の LUT、英語・簡体中国語表示用の言語ファイルが含まれています。表示言語は AviUtl2 の言語設定に合わせて切り替わります。

手動でインストールする場合は、パッケージ内の `Script` と `Language` を対応するフォルダーへ配置してください。`FilmPrintLUT.anm2` と `FilmPrintLUTCube.mod2` は同じフォルダーに置く必要があります。

インストール後、同梱の3種類の LUT は `ProgramData\\aviutl2\\Script\\FilmPrintLUT\\LUTs` にあります。

## 基本操作

`.cube` ファイルを読み込むだけで使用できます。

## 注意事項

* 同梱 LUT は **Rec.709 SDR・Gamma 2.4・フルレンジ（0～1）** を想定しています。Log 素材は、適切な色変換を行ってから適用してください。
* 外部 LUT を読み込む場合は、素材の色空間と LUT が要求する入力形式に合わせて、適切な LUT を選択してください。
* **これらの違いがよくわからない場合は、通常の用途では Rec.709 LUT を使用してください。**
* **LUTファイルが正しい書式で記述されていることを確認してください。**
* スクリプトは色域や Gamma を自動変換しません。ほかの LUT を使用する場合も、素材がその LUT の入力要件を満たしていることを確認してください。
* 独立した 3D `.cube` に対応しています。1D LUT、1D＋3D の複合 LUT、UTF-16 のファイルには対応していません。
* LUT の格子の一辺のサイズは2～90に対応しています。実際に使用できるサイズは AviUtl2 側の制限にも依存します。
* LUT はプロジェクトに埋め込まれません。プロジェクトを移動する場合は LUT ファイルも一緒に保管し、必要に応じてパスを選び直してください。
* 読み込み時に元の LUT を変更したり、PNG を生成・保存したりすることはありません。
* `.cpp` をコンパイルするには、AviUtl2 SDK の `module2.h` が必要です。

</details>

<details>
<summary>中文</summary>

## 简介

**FilmPrintLUT** 是一个 AviUtl2 的调色脚本，可以直接读取 `.cube` 格式的 3D LUT，为视频或图片调整色彩。

选择 LUT 文件后，调整强度即可使用。界面仅 **读取 CUBE、强度、保留 Alpha** 三项设置。

**此脚本本质是`.cube`格式的LUT文件读取脚本。**
附带了三个原创 LUT，不过我建议使用你持有的 LUT。

## 安装方法

1. 将 `FilmPrintLUT\\\_v1.2.0.au2pkg.zip` **直接拖入 AviUtl2 的预览画面，无需解压**。
2. 确认安装内容并完成安装。如果提示重启，请重新启动 AviUtl2。
3. 为目标对象添加效果，在 **色調整 → FilmPrintLUT** 中选择本脚本。

安装包包含脚本、LUT 读取模块、三个 LUT，以及英文和简体中文语言文件。界面语言随 AviUtl2 的语言设置切换。

如果手动安装，请将包内的 `Script` 和 `Language` 放入对应目录。`FilmPrintLUT.anm2` 与 `FilmPrintLUTCube.mod2` 必须放在同一个文件夹。

安装后，三个附赠 LUT 在 `ProgramData\\aviutl2\\Script\\FilmPrintLUT\\LUTs` 中。

## 基本操作

读取`.cube` 就可以了。

## 注意事项

* 附赠 LUT 面向 **Rec.709 SDR、Gamma 2.4、全范围（0～1）**。Log 素材请先完成适当的色彩转换，再应用这些 LUT。
* 读取外部 LUT 时，请根据素材的色彩空间以及 LUT 所要求的输入格式，选择合适的 LUT。
* **如果不清楚这些区别，普通情况下直接使用 Rec.709 LUT 即可。**
* **确保你的LUT书写是规范的。**
* 脚本不会自动转换色域或 Gamma。使用其他 LUT 时，也需要确认素材符合该 LUT 的输入要求。
* 支持独立的 3D `.cube`，不支持 1D LUT、1D＋3D 混合 LUT 或 UTF-16 文件。
* 支持 2～90 的 LUT 网格边长，实际可用尺寸还受 AviUtl2 的限制。
* LUT 不会嵌入工程。移动工程时请一并保留 LUT 文件，必要时重新选择路径。
* 读取时不会修改原 LUT，也不会生成或保存 PNG。
* `.cpp`需要 AviUtl2 SDK 的 `module2.h` 才能编译。

</details>

<details>
<summary>English</summary>

## Overview

**FilmPrintLUT** is a color-grading script for AviUtl2 that reads `.cube` 3D LUT files directly to adjust the colors of videos and images.

Choose a LUT file and adjust its strength to get started. The interface has just three controls: **CUBE file, Strength, and Preserve Alpha**.

**This script is essentially a loader for LUT files in `.cube` format.**
Three original LUTs are included, but I recommend using your own LUTs.

## Installation

1. Drag `FilmPrintLUT\\\_v1.2.0.au2pkg.zip` **directly onto the AviUtl2 preview window without extracting it**.
2. Review the package contents and complete the installation. Restart AviUtl2 if prompted.
3. Add an effect to the target object and select **Color adjustment (色調整) → FilmPrintLUT**.

The package includes the script, LUT loading module, three LUTs, and English and Simplified Chinese language files. The interface language follows the language setting in AviUtl2.

For manual installation, place the package's `Script` and `Language` contents in the corresponding directories. Keep `FilmPrintLUT.anm2` and `FilmPrintLUTCube.mod2` in the same folder.

After installation, the three included LUTs are located in `ProgramData\\aviutl2\\Script\\FilmPrintLUT\\LUTs`.

## Basic Usage

Simply load a `.cube` file.

## Notes

* The included LUTs are designed for **Rec.709 SDR, Gamma 2.4, full range (0–1)**. Apply an appropriate color conversion to Log footage before using these LUTs.
* When loading an external LUT, choose one that matches the footage's color space and the input format required by the LUT.
* **If you are unfamiliar with these differences, use a Rec.709 LUT for typical use cases.**
* **Make sure your LUT file is properly formatted.**
* The script does not automatically convert color gamut or Gamma. When using other LUTs, also make sure your footage meets their input requirements.
* Standalone 3D `.cube` files are supported. 1D LUTs, combined 1D＋3D LUTs, and UTF-16 files are not supported.
* Supported LUT lattice sizes range from 2 to 90 per axis. The sizes you can actually use also depend on AviUtl2's limits.
* LUTs are not embedded in projects. When moving a project, keep the LUT files with it and select their paths again if necessary.
* Loading a LUT does not modify the original file or generate or save a PNG.
* Compiling the `.cpp` file requires `module2.h` from the AviUtl2 SDK.

</details>

