# M5 Glow multi-pass／HDR intermediate／Host transient proof

日付: 2026-08-10
状態: **縮小採用 / PROBE ONLY**

## 1. 結論

既存M5-R0のprivate wgpu fixtureで、投影後RGBAを`Rgba16Float`のlinear中間へ置き、
`bright-pass → separable blur → additive composite`を同じcommand encoderへ直列化できた。
source／bright／blur ping／output texture、pipeline、bind group、readback bufferはfixture生成時に
Host役が一度だけ所有し、同じfixtureの連続評価で再利用する。

これにより、Glowが単一passの近傍sampleではなく、1.0超highlightと複数pass／一時textureを必要とする
表現であること、その最小GPU機構がwgpu上で成立することを確認した。製品Stage、Vism公開API、
Document schema、最終色変換、M4 resource budgetへは接続していない。

## 2. 既知実装preflight

```text
MECHANISM CLASS: linear HDR textureをbright抽出、有限半径blur、加算合成へ流す複数GPU passとHost transient lifetime
KNOWN IMPLEMENTATION SEARCH: docs/specs/M5-3d-and-post.md、docs/reviews/2026-07-23-first-party-vism-expression-demand-survey.md §2.7、M5-R0、M5-P0、docs/references.md
CANDIDATES: M5-R0 wgpu 29.0.4 offscreen/pipeline/readback、M5-P0 separable blur contract
ADOPTION ROUTE: REUSE (wgpu/R0) + PATTERN (P0 blur)
REJECTED CANDIDATES: Vello blur／scene-engine post stack／新依存 :: private GPU成立性oracleに不要でownerを増やす
THIN MOTOLII SEAM: M5-R0内のHost役fixtureが固定texture/pipeline/bind groupを所有してpassをencode
THIN MOTOLII RESIDUAL: 32x32 FP16 source、threshold 1.0、radius 2のprobe parameterとreadback oracle
RETIREMENT: KEEP / probe-only。製品Host transient ownerへ接続する時にfixture型を製品APIへ移植せず退役
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

## 3. 自動oracle

- bright／blur中間は`Rgba16Float`で、readback上のredが`1.0`を超える。
- additive composite後はsourceのred `4.0`を超え、bright結果が元RGBAへ戻る。
- source alphaが0のpixelにもradius 2以内ではhalo alphaが現れ、元の幾何extentを広げる。
- radius 2のpadding外ではRGBAが全て0で、透明領域のRGBが漏れない。
- 一つの`GlowFixture`を2回評価した出力は同一で、render内にtexture／pipeline／bind group生成はない。
- adapter／limit不足は既存R0のtyped refusalを維持する。

## 4. 境界

これは固定半径・full-resolutionの最小Glowであり、downsample pyramid、複数scale bloom、tone mapping、
mask port、RoI scheduler、Draft／Final縮退、resource budget、device loss、3 OSを証明しない。
FP16はこのfixtureの成立条件であり、M5仕様に残る製品中間format未決を解消しない。
最終色変換をshaderへ混ぜず、linear値のままreadbackするため、色変換一元化の製品ownerも変更しない。
