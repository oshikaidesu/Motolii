@通常
--label:加工
--track@fade_percentage:透明度,0,100,0,0.1
--track@fineness:細かさ,0,100,100,1
--[[pixelshader@dithering:
{{.Dithering}}
]]

obj.pixelshader("dithering", "object", {"object"}, {fade_percentage, fineness}, "copy")

@Z座標
--label:加工
--check@is_offscrean:オフスクリーン描画,1
--track@start_z:開始Z座標,-1024,100000,-512,1
--track@end_z:終了Z座標,-1024,100000,-768,1
--track@fineness:細かさ,0,100,100,1
--[[pixelshader@dithering_z:
{{.DitheringZ}}
]]

if is_offscrean == 1 then
    obj.effect("オフスクリーン描画")
end
obj.pixelshader("dithering_z", "object", {"object"}, {start_z, end_z, obj.z, fineness}, "copy")
