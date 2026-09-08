# 生成器は素材の形の中、効果は配置効果の上へ

利用者指示(2026-09-08): Repeat に Gradient を付けると各複製に付かない。「普通にしてくれ」。

## 見つかったこと(2 つ重なっていた)

1. **Gradient が形を無視する**(Repeat と無関係)。`vism/gradient.wgsl` は image 入力の無い生成器で、対象の矩形全面を alpha 1.0 で塗る。円の切り抜きに掛けると四角になる。2026-08-11 の M5-FILTERMASK-P0(gradient は coverage で clip)に現物が追いついていなかった。glass・tri_led・turbulent_displace も同じ型。
2. **Effects から足すと Repeat の下に入る**(Repeat の責務)。新しい効果は末尾 append で、Repeat のある層では末尾 = Repeat の下。9/6 の「下は増えた後の全体に 1 回」が発動し、comp 大の 1 枚に勾配が乗って全面が塗られた(240×80 で 19,200/19,200 px)。

## 直し方

- **生成器は直前の絵の alpha の中に閉じ込める**。`effective_layer_textures` の pass 列で、image 入力が 0 の効果の出力に matte(mode Alpha)を掛ける。効果側に手を入れない 1 箇所で、生成器の族全部に効く。matte は生成器の出力 format ごとに組み直す(`coverage_programs`)。format を跨いで sRGB 型へ置くと次の pass の decode が変わり、既存の「gradient → 半径 0 blur」一致契約が壊れた。
- **普通の効果は配置効果の上へ挿す**(`effect_batch_intents`)。配置効果より下は特別な置き方(1 枚に畳む)なので、普通の手つきで特別な方へ落ちない。下に置きたい時は moveEffect で下げる。配置効果同士は末尾(掛け算)。既存 plugin の二重追加は従来どおり無変更。

AE の Gradient Ramp は矩形全面を塗る(alpha を捨てる)先例だが、Motolii は P0 裁定と利用者の「普通」(Photoshop の塗りは形の中)を採る。

## 検収

- render `placement_contract::a_generator_stays_inside_the_source_shape_above_and_below_the_placement`: 円 ×3 に gradient を上/下どちらに置いても塗られる px = 円 3 つ分。
- ui/native `effect_insert_position`: [blur, Repeat] に gradient → [blur, gradient, Repeat]、Repeat 追加は無変更、Repeat 無しは append。
- 既存 `mixed_format_effects_consume_previous_output_and_recover_after_undo`(gradient→blur 0 の一致・Undo)は据え置きで通過。render 42 本・ui/native 28 本。
