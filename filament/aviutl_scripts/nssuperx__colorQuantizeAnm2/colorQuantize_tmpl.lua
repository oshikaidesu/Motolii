@通常
--label:色調整
--track@color_num:色数,2,255,2,1
--track@threshold_shift:しきい値ずれ量,-0.5,0.5,0.0,0.01
--track@curve:しきい値曲線,-1.0,1.0,0.0,0.01
--color@startColor:開始色,0xffffff
--color@endColor:終了色,0x000000
--[[pixelshader@colorQuantize:
{{.ColorQuantize}}
]]

sr, sg, sb = RGB(startColor)
er, eg, eb = RGB(endColor)

obj.pixelshader("colorQuantize", "object", {"object"}, {color_num, threshold_shift, curve, sr, sg, sb, er, eg, eb}, "copy")

@グレースケール
--label:色調整
--track@color_num:色数,2,255,2,1
--track@threshold_shift:しきい値ずれ量,-0.5,0.5,0.0,0.01
--track@curve:しきい値曲線,-1.0,1.0,0.0,0.01
--[[pixelshader@grayScaleQuantize:
{{.GrayScaleQuantize}}
]]

obj.pixelshader("grayScaleQuantize", "object", {"object"}, {color_num, threshold_shift, curve}, "copy")

@bit数
--label:色調整
--track@color_bits:bit数,1,8,4,1
--[[pixelshader@bitQuantize:
{{.BitQuantize}}
]]

obj.pixelshader("bitQuantize", "object", {"object"}, {color_bits}, "copy")

@bit数（RGB別）
--label:色調整
--track@color_bits_r:赤bit数,1,8,4,1
--track@color_bits_g:緑bit数,1,8,4,1
--track@color_bits_b:青bit数,1,8,4,1
--[[pixelshader@bitRGBQuantize:
{{.BitRGBQuantize}}
]]

obj.pixelshader("bitRGBQuantize", "object", {"object"}, {color_bits_r, color_bits_g, color_bits_b}, "copy")
