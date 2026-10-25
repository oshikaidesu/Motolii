--[[
EffectUtils_ColorGrade.lua
Color space conversion and grading helpers.
]]

local EffectUtils_ColorGrade = {}

local math_max = math.max
local math_min = math.min
local math_floor = math.floor

function EffectUtils_ColorGrade.clamp01(v)
  return math_max(0, math_min(1, v))
end

function EffectUtils_ColorGrade.lerp(a, b, t)
  return a + (b - a) * t
end

function EffectUtils_ColorGrade.lerp3(a, b, t)
  return {
    EffectUtils_ColorGrade.lerp(a[1], b[1], t),
    EffectUtils_ColorGrade.lerp(a[2], b[2], t),
    EffectUtils_ColorGrade.lerp(a[3], b[3], t),
  }
end

function EffectUtils_ColorGrade.rgb_to_hsl(r, g, b)
  local maxv = math_max(r, math_max(g, b))
  local minv = math_min(r, math_min(g, b))
  local h, s, l = 0, 0, (maxv + minv) * 0.5
  if maxv ~= minv then
    local d = maxv - minv
    s = l > 0.5 and d / (2 - maxv - minv) or d / (maxv + minv)
    if maxv == r then
      h = (g - b) / d + (g < b and 6 or 0)
    elseif maxv == g then
      h = (b - r) / d + 2
    else
      h = (r - g) / d + 4
    end
    h = h / 6
  end
  return h, s, l
end

function EffectUtils_ColorGrade.hsl_to_rgb(h, s, l)
  if s <= 0 then
    return l, l, l
  end
  local function hue2rgb(p, q, t)
    if t < 0 then t = t + 1 end
    if t > 1 then t = t - 1 end
    if t < 1 / 6 then return p + (q - p) * 6 * t end
    if t < 1 / 2 then return q end
    if t < 2 / 3 then return p + (q - p) * (2 / 3 - t) * 6 end
    return p
  end
  local q = l < 0.5 and l * (1 + s) or l + s - l * s
  local p = 2 * l - q
  return hue2rgb(p, q, h + 1 / 3), hue2rgb(p, q, h), hue2rgb(p, q, h - 1 / 3)
end

function EffectUtils_ColorGrade.linear_to_srgb(c)
  if c <= 0.0031308 then
    return 12.92 * c
  end
  return 1.055 * (c ^ (1 / 2.4)) - 0.055
end

function EffectUtils_ColorGrade.srgb_to_linear(c)
  if c <= 0.04045 then
    return c / 12.92
  end
  return ((c + 0.055) / 1.055) ^ 2.4
end

function EffectUtils_ColorGrade.rgb_to_xyz(r, g, b)
  r = EffectUtils_ColorGrade.srgb_to_linear(r)
  g = EffectUtils_ColorGrade.srgb_to_linear(g)
  b = EffectUtils_ColorGrade.srgb_to_linear(b)
  return
    0.4124 * r + 0.3576 * g + 0.1805 * b,
    0.2126 * r + 0.7152 * g + 0.0722 * b,
    0.0193 * r + 0.1192 * g + 0.9505 * b
end

function EffectUtils_ColorGrade.xyz_to_rgb(x, y, z)
  local r = 3.2406254 * x - 1.537208 * y - 0.4986286 * z
  local g = -0.9689307 * x + 1.875756 * y + 0.041517522 * z
  local b = 0.055710122 * x - 0.20402105 * y + 1.056996 * z
  return
    EffectUtils_ColorGrade.linear_to_srgb(r),
    EffectUtils_ColorGrade.linear_to_srgb(g),
    EffectUtils_ColorGrade.linear_to_srgb(b)
end

function EffectUtils_ColorGrade.luminance(r, g, b)
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
end

function EffectUtils_ColorGrade.vignette_factor(dist, intensity, falloff)
  local v = 1 - EffectUtils_ColorGrade.clamp01(dist * falloff)
  return 1 - intensity * (1 - v * v)
end

function EffectUtils_ColorGrade.color_from_hex(hex)
  hex = math_floor(tonumber(hex) or 0xffffff) % 0x1000000
  return
    math.floor(hex / 0x10000) / 255,
    math.floor((hex / 0x100) % 0x100) / 255,
    math.floor(hex % 0x100) / 255
end

function EffectUtils_ColorGrade.white_balance_rgb(r, g, b, gain_r, gain_g, gain_b, rate)
  rate = EffectUtils_ColorGrade.clamp01(rate or 1)
  gain_r = gain_r ^ rate
  gain_g = gain_g ^ rate
  gain_b = gain_b ^ rate
  return r * gain_r, g * gain_g, b * gain_b
end

function EffectUtils_ColorGrade.lut_sample_1d(lut, index)
  index = EffectUtils_ColorGrade.clamp01(index)
  local n = #lut
  if n <= 1 then
    return lut[1] or {0, 0, 0}
  end
  local pos = index * (n - 1)
  local i0 = math_floor(pos) + 1
  local i1 = math_min(i0 + 1, n)
  local t = pos - math_floor(pos)
  return EffectUtils_ColorGrade.lerp3(lut[i0], lut[i1], t)
end

return EffectUtils_ColorGrade
