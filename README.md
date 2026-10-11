# Motolii

ThorVG と Filament だけを置いた、履歴のない置き場です。`rust_ae` のリポジトリからはコピーで、元の木は動かしていません。

| 場所 | 中身 |
|---|---|
| `thorvg/` | ThorVG 1.1 を wgpu の Metal デバイスへ載せる結合。`src/thor.rs`、`src/native_device.c`、ライブラリ |
| `filament/` | ガラスと三次元の光路の実験。ソースと Filament の事前ビルド。描画結果と依存パッケージは含まない |

この二つは、まだ一つのプロセスには繋がっていません。ガラスのプリズムは Filament 側です。

```sh
filament/prism.sh
```

`filament/out/prism.png` に出ます。

ThorVG のパスから三次元のチューブを作る実験は `link.sh` です。パスは ThorVG が持ち、毎フレーム読み戻して Filament の頂点バッファにします。

```sh
filament/link.sh
```

`filament/out/link-sheet.png` ほかに出ます。`thorvg/` の ThorVG は WebGPU 専用のビルドでソフトウェア描画を持たないため、二次元の比較の段は描かずに飛ばします。

`filament/gpu_tube.mm`(`link.sh gpu_tube` で動く)は、チューブの展開を GPU でやる版です。同じ制御点から CPU で展開したガラスのチューブと画素を比べ、ずれが大きければ `FAIL` で終わります。

頂点処理の `material.worldPosition` は、ユーザーの世界座標ではなくカメラ位置ぶんずれた Filament 内部の座標です。絶対座標で値を入れるときは `getUserWorldFromWorldMatrix()` で変換してから戻します。

持ってきていないもの: 編集の文書と窓、`v1` の計画、`work` の他の実験、画像の出力、`drei-ref`。
