--label:アニメーション効果
--track@sliceAngle:角度,-360,360,10,1
--track@shiftAmount:移動量,-100,100,10,0.1
--track@shiftAmountTimes:移動量倍率,0,100,1,0.01
--track@shiftBias:移動方向偏り,-1,1,0,0.01
--track@sliceCount:分割数,0,100,10,1
--check@sliceWidthRandom:幅ランダム化,0
--track@sliceWidth:幅拡張,-100,300,0,1
--check@fadeEnable:フェード,0
--check@colorEnable:色付け,0
--color@color:色,0xffffff
--track@randSeed:乱数シード,0,10000,0,1
--[[pixelshader@objectSlicer:
{{.ObjectSlicer}}
]]

local r, g, b = RGB(color)
local angleRad = math.rad(sliceAngle or 0)

obj.pixelshader("objectSlicer", "object", {"object"}, {angleRad, shiftAmount, shiftAmountTimes, shiftBias, sliceCount, sliceWidthRandom, sliceWidth, fadeEnable, colorEnable, r, g, b, randSeed}, "copy", "clip")
