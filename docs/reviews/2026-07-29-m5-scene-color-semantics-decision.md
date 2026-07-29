# M5 scene-color semantics decision

作成日: 2026-07-29

状態: **決定／P2D-RCFP1S DONE**

## 1. Authorityと分割

本decisionはM5のscene-color／refraction capability、色変換一元化、Preview／Export同一評価を正本とする。
現行render targetが`Rgba8Unorm`／`Srgb`であること、`Rgba16Float`／`LinearRgb`型が存在することを
新経路の成立証拠にしない。

`P2D-RCFP1`を次へ分ける。

- `P2D-RCFP1S`: scene-colorの色／alpha意味。本decisionでDONE。
- `P2D-RCFP1F`: concrete GPU format、precision、usage、extent別byte量。実機証拠までWAIT。

## 2. canonical scene-color意味

- scene-color snapshotは**linear-light RGB**として解釈する。
- render intermediateのalphaは**premultiplied**を正規形とする。
- alphaはcoverage／transmittanceのscalarであり、RGBのEOTF／OETFを適用しない。
- display-encoded sRGB、BT.709 YUV、final output tag、tone-mapped像ではない。
- intermediateで暗黙sRGB encode、tone map、8-bit quantize、0〜1 clampを行わない。

永続straight-alpha sRGB色はHost所有render ingressで、sRGB EOTF→linear RGB→premultiplyの順に
変換する。premultiply後にRGBをlinear化しない。

## 3. 単一変換authority

Hostが保存色／decoded sourceをcanonical scene-colorへ正規化し、final display／Exportの
encoding／色変換もHostの単一terminal boundaryが所有する。

contribution、effect、providerは個別EOTF／OETF、straight／premultiplied切替、tone mapping、
output encodingを所有しない。すでに正規化されたlinear-premultiplied inputだけを読む。

linear意味を提供できない環境ではdescriptorを偽装したsRGB／RGBA8へfallbackせず、能力不足を
型付き拒否する。

## 4. truthful FrameDesc

Hostはscene-color resourceと実意味に一致する`FrameDesc`を対で扱う。

- `color_space = LinearRgb`
- `premultiplied = true`
- `format`は後続RCFP1Fで採択した実format
- width／heightは`Quality`とK0／RCR1の要求範囲に従う

`FrameDesc`はsnapshot point、logical RoI、lifetime、ordering、resource ownershipを表さない。
`stride`をGPU row pitch、allocation alignment、総byte量とみなさない。

descriptorとresource意味の不一致は型付き拒否し、label、wgpu metadata、provider IDから推測しない。

## 5. QualityとPreview／Export

`Quality`はresolution、sample数、format、許容誤差を変え得るが、linear-light／premultipliedの意味を
変えない。DRAFTだけsRGB値を`LinearRgb`と称さない。

Preview／Exportは同じscene-color評価関数を通り、final display／file encodingのterminal差だけを
Host boundaryへ残す。

## 6. semantic oracle

- sRGB 0.5をlinear 0.5と扱わず、共通sRGB EOTF期待値へ変換する。
- linear blendとsRGB値直接blendを区別する中間調fixture。
- straight sRGB→linear→premultiplyと誤順序を区別する半透明fixture。
- alpha 0のRGB寄与が0で、後続blur／refractionへ色漏れしない。
- scene-color read前後でEOTF／OETFを二重適用しない。
- Preview／Exportが同じlinear意味を持つ。
- `Srgb`なのにlinear値、`LinearRgb`なのにsRGB値、`premultiplied=false`を型付き拒否する。
- 1.0超highlightをterminal変換前に暗黙clampしない。

semantic oracle artifactだけを`classification.tsv`へ登録し、harness／runtime配線と分離する。
期待値やtolerance変更で合格させない。

## 7. RCFP1Fの必須証拠

`Rgba16Float`を第一候補、`Rgba32Float`をreference／必要時候補にできるが、採択ではない。
RCFP1Fは少なくとも次を実測する。

- render attachment、blend、sample、filter、copyの対象backend対応。
- black／white／中間調／near-zero alpha、複数over、1.0超highlight、blur、subpixel scene sample。
- fp32 referenceに対するDRAFT／FINALのabsolute／relative error、banding、NaN／Inf／subnormal。
- formatごとのsample／mip／alignment／同時live resourceを含むbyte量。
- K0 `Finite / Infinite / Unknown`とHost clamp後extent。Unknownを空allocationにしない。
- unsupported deviceのtyped refusal。RGBA8／sRGBへ黙示fallbackしない。

hard budget、admission、cache keyの最終ownerはM4-K1／`P2D-RCBUD1`である。
RCFP1Fはformat別必要量を正しく提示できるところまでを所有する。

## 8. 非目標とSTOP

本decisionはHDR output／mastering metadata、OCIO、final TRC、BT.709 YUV、copy／subpass、
snapshot range／order、resource lifetime、public plugin API、Document schemaを決めない。

次で停止する。

- linear／premultiplied意味を閉じるため保存`Color`意味の変更が必要になる。
- `FrameDesc`へsnapshot／range／lifetime fieldを追加する。
- concrete formatを実機usage／precision／extent証拠なしで採択する。
- fp16非対応時にRGBA8／sRGB fallbackしか成立しない。
- hard budget責任をRCFP1へ移す。
- final output color policyを同時に決める必要がある。
