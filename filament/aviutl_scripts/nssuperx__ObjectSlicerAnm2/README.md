# ObjectSlicerAnm2

![objectslicer](./imgs/objectslicer.png)

AviUtl2スクリプト オブジェクトをスライスする

Respectrum93様の [MultiSlicer_P](https://www.dropbox.com/scl/fo/x4l4mepoe5w7d9mtt332g/ACON0AjzXe3vxxIT0-4Hl6k?dl=0&e=1&preview=MultiSlicer_P.anm&rlkey=w17rfsb61pyhdbws5dfo7jqc9) がAviUtl2で使えなくなっていたため、その移植を試みていました。HLSLで書き直しています。

本家の [Multi-Slicer](https://www.dropbox.com/scl/fo/x4l4mepoe5w7d9mtt332g/ACON0AjzXe3vxxIT0-4Hl6k?dl=0&e=1&file_subpath=%2F%40MultiSlicer.anm&preview=old_script_etc.zip&rlkey=w17rfsb61pyhdbws5dfo7jqc9) は使えるため、特殊な事情がない限りは本家のスクリプトを使用してください。

## 使い方

> [!WARNING]
> **幅ランダム化** と **幅拡張** は未実装

> [!IMPORTANT]
> フィルタオブジェクトの **領域拡張** をこのスクリプトより前に適用することを強く推奨します

[Releases](https://github.com/nssuperx/ObjectSlicerAnm2/releases)から objectSlicer.anm2 をダウンロード

フィルタのラベルは**アニメーション効果**

|パラメータ|値|説明|
|---|---|---|
|角度|-360~360|スライスの角度|
|移動量|-100~100|移動量|
|移動量倍率|0~100|移動量の倍率|
|移動方向偏り|-1.0~1.0|スライスの移動方向の割合を変える。-1だと全部負方向, 0は半々, 1は全部正方向|
|分割数|0~100|スライスの数|
|幅ランダム化|bool|スライス幅をランダムにするか|
|幅拡張|-100~300|スライス幅|
|フェード|bool|移動量に応じてフェードさせるか|
|色付け|bool|移動量に応じて色を変えるか|
|色|色|色付けがオンだとここで指定した色を付ける|
|乱数シード|0~10000|乱数シード|

## 謝辞

このスクリプトは、Respectrum93様の[MultiSlicer_P](https://www.dropbox.com/scl/fo/x4l4mepoe5w7d9mtt332g/ACON0AjzXe3vxxIT0-4Hl6k?dl=0&e=1&preview=MultiSlicer_P.anm&rlkey=w17rfsb61pyhdbws5dfo7jqc9), [Multi-Slicer](https://www.dropbox.com/scl/fo/x4l4mepoe5w7d9mtt332g/ACON0AjzXe3vxxIT0-4Hl6k?dl=0&e=1&file_subpath=%2F%40MultiSlicer.anm&preview=old_script_etc.zip&rlkey=w17rfsb61pyhdbws5dfo7jqc9)にインスパイアされて作りました
