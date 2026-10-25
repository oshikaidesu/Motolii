# PuniCollision

[![Demo](preview.webp)](https://github.com/user-attachments/assets/df8f9a61-8941-4577-a7d8-ced283a6523d)

<details open>
<summary><strong>日本語　▲ 🌐 Switch Language</strong></summary>

## 概要

**PuniCollision** は、AviUtl2 の画像・図形オブジェクト向けの2Dソフトボディアニメーション効果スクリプトです。

フィルタ効果を追加し、グループIDを同じ数値に設定すると、グループ内のすべてのオブジェクトが衝突したときの押し合いや変形を再現できます。

**「剛体」** にチェックを入れると、そのオブジェクトは変形しない障害物として扱われ、他のソフトボディを押して変形させることができます。

本スクリプトは、オブジェクトの位置移動、自由落下、回転移動を制御するものではありません。

効果の追加メニューの **「変形」→「PuniCollision」** から選択できます。

## インストール方法

1. `PuniCollision_v0.7.0.au2pkg.zip` をダウンロードします。
2. 解凍せずに AviUtl2 のプレビュー画面へドラッグ＆ドロップし、内容を確認してインストールします。
3. 必要に応じて AviUtl2 を再起動し、対象オブジェクトに効果を追加します。

AviUtl2 **2.1.13 以降**が必要です。パッケージには日本語のスクリプト本体と、簡体中国語・英語の `.aul2` 言語ファイルが含まれています。表示言語は AviUtl2 の言語設定に従います。

## 基本操作

1. 衝突させたい各オブジェクトへ PuniCollision を追加します。
2. 同じ **グループID**を設定します。別々の組として扱う場合はグループIDを分けてください。
3. キーフレームで位置を動かし、接触した状態で変形を調整します。
4. 壁や床にするオブジェクトは **剛体**を有効にします。

### ソフトボディ

| 項目 | 動作 |
| --- | --- |
| 変形範囲 | 0～100、初期値60。低いほど輪郭付近、高いほど内部まで変形します。0でも衝突は無効になりません。 |
| 押出量 | 0～100、初期値50。低いほどつぶれやすく、高いほど面積を保とうとして周囲へ膨らみます。 |
| 反発 | 0～100、初期値50。低いほど反発が早く収まり、高いほど長く続きます。0は反発の完全停止ではありません。 |

押出量と反発は独立して調整できます。

### 剛体

剛体を有効にするとソフトボディ用の設定が隠れ、形状の設定が表示されます。

- **自動生成**：透明部分を除いた画像の輪郭を使用します。透明部分がない画像では画像全体が対象になります。
- **手動指定**：2～36点をマウスでドラッグして配置します。2点は線分、3点以上は閉じた多角形です。線を交差させないでください。

制御点はオブジェクトに追従します。点と線は選択して編集中のみ表示され、プレビュー映像や書き出しには含まれません。

## 注意事項

- 個別のオブジェクトに適用してください。グループ制御・画像合成への適用は推奨しません。
- ソフトボディ同士の衝突には滑らかな近似輪郭を使用します。髪飾りなど細い部分や深いくぼみの画素境界とは完全には一致しません。
- 2Dの移動・拡大縮小・Z軸回転と、X/Y軸の180度反転に対応します。X/Y軸の斜め回転には対応していません。
- 同じグループIDのオブジェクトが多いほど処理が重くなります。関係のないオブジェクトには別のグループIDを設定してください。

</details>

<details>
<summary><strong>中文</strong></summary>

## 概要

**PuniCollision** 是用于 AviUtl2 图片和图形对象的2D软体动画效果脚本。

添加效果，组ID设置相同数字后，组内所有对象可相互挤压变形。
如果勾选 **刚体** ，该对象便作为不发生形变的障碍物，可以挤压其他软体。
本脚本不接管位置移动、自由下落或滚动。

从效果菜单 **“变形（変形）”→“PuniCollision”** 中添加。

## 安装方法

1. 下载 `PuniCollision_v0.7.0.au2pkg.zip`。
2. 不要解压，直接拖入 AviUtl2 预览画面，确认内容后安装。
3. 按需重启 AviUtl2，然后给目标对象添加效果。

需要 AviUtl2 **2.1.13 或更新版本**。安装包包含日语脚本，以及简体中文、英语 `.aul2` 语言文件，界面跟随 AviUtl2 的语言设置。

## 基本操作

1. 给参与碰撞的对象分别添加 PuniCollision。
2. 设置相同的 **组ID**。不同碰撞组使用不同组ID。
3. 通过关键帧移动对象，在接触状态下调整形变。
4. 作为墙面、地面的对象勾选 **刚体**。

### 软体

| 参数 | 作用 |
| --- | --- |
| 变形范围 | 0～100，默认60。低值主要影响边缘，高值向内部扩展；0不会关闭碰撞。 |
| 挤出量 | 0～100，默认50。低值容易压瘪，高值更倾向保持面积并向周围撑开。 |
| 反弹 | 0～100，默认50。低值快速平息，高值持续更久；0不是完全关闭回弹。 |

挤出量和反弹可以独立调整。

### 刚体

勾选刚体后，软体参数隐藏，显示刚体形状设置。

- **自动生成**：根据不透明部分的轮廓生成刚体；没有透明背景时以整张图片为对象。
- **手动指定**：设置2～36个点，仅用鼠标拖动。2点为线段，3点以上首尾闭合。请勿交叉连线。

控制点跟随对象移动。点线仅选中编辑时显示，不进入预览画面和输出。

## 注意事项

- 请添加给独立对象，不推荐添加给组控制与画像合成。
- 软体互撞使用平滑的近似碰撞轮廓，不会完全贴合发饰等细小部分或深凹陷的像素边缘。
- 支持2D位移、缩放、Z轴旋转和X/Y轴180°翻转，不支持X/Y轴倾斜旋转。
- 同组ID对象越多，计算量越大。不需要互相碰撞的对象请使用不同组ID。

</details>

<details>
<summary><strong>English</strong></summary>

## Overview

**PuniCollision** is a 2D soft-body animation effect script for image and shape objects in AviUtl2.

Apply the effect and set the same Group ID for multiple objects. All objects within the group can push against and deform each other.

When **Rigid Body** is enabled, the object acts as a non-deformable obstacle that can push and deform other soft bodies.

This script does not control object movement, free fall, or rolling.

Add the effect from **Deformation (変形) → PuniCollision**.

## Installation

1. Download `PuniCollision_v0.7.0.au2pkg.zip`.
2. Drag the package onto the AviUtl2 preview without extracting it, review its contents, and install it.
3. Restart AviUtl2 if needed, then add the effect to your objects.

Requires AviUtl2 **2.1.13 or later**. The package contains the Japanese script and Simplified Chinese and English `.aul2` language files. The display language follows AviUtl2's language setting. 

## Basic Usage

1. Add PuniCollision to each participating object.
2. Give them the same **GroupID**. Use different IDs for separate groups.
3. Animate their positions with keyframes and adjust deformation while they touch.
4. Enable **Rigid Body** for walls and floors.

### Soft Bodies

| Parameter | Behavior |
| --- | --- |
| Deformation | 0–100, default60. Low values affect mainly the edges; high values extend deformation into the interior. 0 does not disable collision. |
| Compression | 0–100, default50. Low values allow flattening; high values favor area preservation and outward bulging. |
| Rebound | 0–100, default50. Low values settle faster; high values retain motion longer. 0 does not completely disable rebound. |

Compression and Rebound are independent controls.

### Rigid Bodies

Enabling Rigid Body hides the soft-body controls and displays the shape settings.

- **Auto** uses the visible image contour. An image without transparency uses the whole image area.
- **Manual** provides 2–36 mouse-dragged points. Two points form a segment; three or more form a closed polygon. Do not cross the edges.

Points follow the object. Handles appear only while selected for editing and are excluded from preview frames and exports.

## Notes

- Apply the effect to individual objects. Applying it to Group Control or Image Composition is not recommended.
- Soft-body collisions use smooth approximate outlines. Thin decorations and deep concavities may not match their exact pixel boundaries.
- Supports 2D translation, scaling, Z rotation, and 180° X/Y flips. Tilted X/Y rotations are not supported.
- More objects sharing an ID increase processing cost. Assign unrelated objects to different IDs.

</details>
