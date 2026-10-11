# caustics

床に落ちる、ガラスの分光コースティクス(虹色の光の網)。wgpu + WGSL。Vulkan / Metal / DX12 で動く。

```sh
cargo run --release
```

`floor-caustics.png`(床を真上から見た絵)と、GPU の実時間(タイムスタンプ、41 フレームの中央値)が出る。

## 借りたもの(自作の光学は無い)

| 何 | どこから | ライセンス |
|---|---|---|
| 6 波長の屈折、出口の探索、色の重み(CIE XYZ)、分散の式 | [visgl/luma.gl](https://github.com/visgl/luma.gl) `SpectralCausticsRenderer` | MIT(`third_party/luma.gl-LICENSE`) |
| 屈折した三角形の面積比で明るさを決める考え方 | Evan Wallace の WebGL Water、`axlindt/threejs-refractive-caustics` | MIT |

`src/shaders/trace_hits.wgsl` は luma.gl のシェーダーの派生。光の粒を撒く代わりに、光源から見たテクセルごとに「屈折後に床のどこへ着くか」を記録する。`cells.wgsl` がその隣り合うテクセルの三角形を、面積比(明るさ)つきで床へ描く。粒を撒く方式は、干渉縞(モアレ)が出て、数を増やしても消えなかった。

## 設定(環境変数)

`CAUSTIC_CAPTURE`(光源側の格子、既定 256)、`CAUSTIC_MAP`(床の絵、1024)、`CAUSTIC_DISPERSION`(分散、0.03)、`CAUSTIC_INTENSITY`(1 が太陽と同じ)、`CAUSTIC_FLOOR`(床の一辺 m)、`CAUSTIC_EXPOSURE`。

## 状態(M4、2 物体)

- 見た目: なめらかな連続した光の筋、尖った焦点(カスプ)、縁の分散の色づき。粒の雑音は無い
- GPU 実時間: 約 4.4〜6 ms。内訳は、撮影 4 回で約 3 ms、床へ描く部分で約 2.6 ms、計算で約 0.5 ms。ガラスの予算 1〜2 ms には届いてない
- 解像度を下げても時間はほとんど変わらない(固定費が支配)

## できないこと

- 閉じた凸に近い物体 1 つにつき、屈折は入口と出口の 2 面だけ。内部反射、全反射は無い
- 自分自身と重なるチューブ(輪が交差する所)と、凹んだ部分
- ガラスの足元の影は、格子の段差が見える(隠すにはぼかしが要る)
