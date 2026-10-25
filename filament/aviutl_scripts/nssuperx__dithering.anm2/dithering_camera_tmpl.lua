@単一カメラ
--label:加工
--check@is_offscrean:オフスクリーン描画,1
--track@start_dist:開始距離,0,100000,512,1
--track@end_dist:終了距離,0,100000,100,1
--track@fineness:細かさ,0,100,100,1
--[[pixelshader@dithering_camera:
{{.DitheringCamera}}
]]

if is_offscrean == 1 then
    obj.effect("オフスクリーン描画")
end
local cam = camera_param
obj.pixelshader("dithering_camera", "object", {"object"}, {start_dist, end_dist, obj.x, obj.y, obj.z, cam.x, cam.y, cam.z, fineness}, "copy")

@複数カメラ
--label:加工
--check@is_offscrean:オフスクリーン描画,1
--value@cam_layer:カメラ制御レイヤー,1
--track@start_dist:開始距離,0,100000,512,1
--track@end_dist:終了距離,0,100000,100,1
--track@fineness:細かさ,0,100,100,1
--[[pixelshader@dithering_camera:
{{.DitheringCamera}}
]]

if is_offscrean == 1 then
    obj.effect("オフスクリーン描画")
end
local cam = camera_param_table[cam_layer]
obj.pixelshader("dithering_camera", "object", {"object"}, {start_dist, end_dist, obj.x, obj.y, obj.z, cam.x, cam.y, cam.z, fineness}, "copy")
