--track@fade_percentage:透明度,0,100,0,0.1
--track@fineness:細かさ,0,100,100,1
--[[pixelshader@{{.psmain}}:
{{.hlsl}}
]]

obj.pixelshader("{{.psmain}}", "object", {"object"}, {fade_percentage, fineness}, "copy")
