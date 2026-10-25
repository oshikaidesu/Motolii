--[[
EffectUtils_ColorSpace.lua v1.01 by HexBrowns
色空間の変換・補間・色相回転・色差。

  local cs = require("EffectUtils_ColorSpace")

HLSL の同じ式: AI/tools/assets/colorspace.hlsl（関数名は cs_ を外したもの）。
机上検証: AI/scripts/色空間拡張/verify_colorspace.py

色はすべてストレート（乗算済みでない）の sRGB（0〜1）で受け渡す。
色空間の番号（全スクリプト共通。@図形グラデーション_H と同じ）
  0 sRGB / 1 リニア / 2 Oklab / 3 HSV / 4 HSL / 5 Lab / 6 LCh / 7 Oklch / 8 XYZ / 9 HWB
色相のある色空間（3 / 4 / 6 / 7 / 9）は 1 番目に色相（0〜1 周）を置く。
  HSV (h, s, v) / HSL (h, s, l) / HWB (h, w, b) / LCh (h, C, L) / Oklch (h, C, L)
Lab は D65 の白（CSS の lab() は D50 なので値は少し違う）。
色差の種類: 0 RGB / 1 YCbCr（色差だけ）/ 2 Oklab / 3 CIEDE2000。黒と白の差がおよそ 1。

require は呼んだスクリプトのフォルダしか探さないので、Script/ 直下のスクリプトから使う。
]]

local M = {}

local abs, max, min, floor = math.abs, math.max, math.min, math.floor
local sqrt, sin, cos, atan2, exp = math.sqrt, math.sin, math.cos, math.atan2, math.exp
local TAU = math.pi * 2

M.SRGB, M.LINEAR, M.OKLAB, M.HSV, M.HSL, M.LAB, M.LCH, M.OKLCH, M.XYZ, M.HWB = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9

local function clamp01(v)
  if v < 0 then return 0 end
  if v > 1 then return 1 end
  return v
end

local function frac(x)
  return x - floor(x)
end

-- 0xRRGGBB → r, g, b（0〜1）
function M.unpack(col)
  col = floor(tonumber(col) or 0)
  return floor(col / 65536) % 256 / 255, floor(col / 256) % 256 / 255, col % 256 / 255
end

-- r, g, b（0〜1）→ 0xRRGGBB（範囲外は切り詰める）
function M.pack(r, g, b)
  return floor(clamp01(r) * 255 + 0.5) * 65536 + floor(clamp01(g) * 255 + 0.5) * 256 + floor(clamp01(b) * 255 + 0.5)
end

local function lin1(c)
  if c <= 0.04045 then return c / 12.92 end
  return ((abs(c) + 0.055) / 1.055) ^ 2.4
end

local function srgb1(c)
  if c <= 0.0031308 then return c * 12.92 end
  return 1.055 * abs(c) ^ (1 / 2.4) - 0.055
end

function M.to_linear(r, g, b)
  return lin1(r), lin1(g), lin1(b)
end

-- リニア → sRGB。範囲外は切り詰める
function M.to_srgb(r, g, b)
  return srgb1(clamp01(r)), srgb1(clamp01(g)), srgb1(clamp01(b))
end

local function cbrt(x)
  if x < 0 then return -((-x) ^ (1 / 3)) end
  return x ^ (1 / 3)
end

-- Oklab（Björn Ottosson, 2020）
function M.linear_to_oklab(r, g, b)
  local l = cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
  local m = cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
  local s = cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
  return 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s
end

function M.oklab_to_linear(L, a, b)
  local l = L + 0.3963377774 * a + 0.2158037573 * b
  local m = L - 0.1055613458 * a - 0.0638541728 * b
  local s = L - 0.0894841775 * a - 1.2914855480 * b
  l, m, s = l * l * l, m * m * m, s * s * s
  return 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
end

-- sRGB（D65）のリニア ⇔ XYZ
function M.linear_to_xyz(r, g, b)
  return 0.4124564 * r + 0.3575761 * g + 0.1804375 * b,
    0.2126729 * r + 0.7151522 * g + 0.0721750 * b,
    0.0193339 * r + 0.1191920 * g + 0.9503041 * b
end

function M.xyz_to_linear(x, y, z)
  return 3.2404542 * x - 1.5371385 * y - 0.4985314 * z,
    -0.9692660 * x + 1.8760108 * y + 0.0415560 * z,
    0.0556434 * x - 0.2040259 * y + 1.0572252 * z
end

-- CIE 15: ε = 216/24389、κ = 24389/27
local function lab_f(t)
  if t > 0.008856452 then return cbrt(t) end
  return (903.2963 * t + 16) / 116
end

local function lab_finv(f)
  local f3 = f * f * f
  if f3 > 0.008856452 then return f3 end
  return (116 * f - 16) / 903.2963
end

function M.xyz_to_lab(x, y, z)
  local fx, fy, fz = lab_f(x / 0.95047), lab_f(y), lab_f(z / 1.08883)
  return 116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)
end

function M.lab_to_xyz(L, a, b)
  local fy = (L + 16) / 116
  local fx = fy + a / 500
  local fz = fy - b / 200
  return lab_finv(fx) * 0.95047, lab_finv(fy), lab_finv(fz) * 1.08883
end

-- (L, a, b) → (色相 0〜1 周, C, L)
function M.lab_to_polar(L, a, b)
  return frac(atan2(b, a) / TAU + 1), sqrt(a * a + b * b), L
end

function M.polar_to_lab(h, c, L)
  return L, c * cos(h * TAU), c * sin(h * TAU)
end

local function hue_of(r, g, b, mx, mn)
  local d = mx - mn
  local h = 0
  if d > 0.000001 then
    if mx == r then
      h = (g - b) / d
    elseif mx == g then
      h = (b - r) / d + 2
    else
      h = (r - g) / d + 4
    end
  end
  return frac(h / 6 + 1)
end

local function hue_rgb(h)
  local function ch(o)
    return clamp01(abs(frac(h + o) * 6 - 3) - 1)
  end
  return ch(1), ch(2 / 3), ch(1 / 3)
end

function M.rgb_to_hsv(r, g, b)
  local mx, mn = max(r, g, b), min(r, g, b)
  local s = mx > 0.000001 and (mx - mn) / mx or 0
  return hue_of(r, g, b, mx, mn), s, mx
end

function M.hsv_to_rgb(h, s, v)
  local hr, hg, hb = hue_rgb(h)
  s = clamp01(s)
  return v * (1 + (hr - 1) * s), v * (1 + (hg - 1) * s), v * (1 + (hb - 1) * s)
end

function M.rgb_to_hsl(r, g, b)
  local mx, mn = max(r, g, b), min(r, g, b)
  local l = (mx + mn) * 0.5
  local den = 1 - abs(2 * l - 1)
  local s = den > 0.000001 and (mx - mn) / den or 0
  return hue_of(r, g, b, mx, mn), s, l
end

function M.hsl_to_rgb(h, s, l)
  local chroma = (1 - abs(2 * l - 1)) * clamp01(s)
  local hr, hg, hb = hue_rgb(h)
  return (hr - 0.5) * chroma + l, (hg - 0.5) * chroma + l, (hb - 0.5) * chroma + l
end

function M.rgb_to_hwb(r, g, b)
  local mx, mn = max(r, g, b), min(r, g, b)
  return hue_of(r, g, b, mx, mn), mn, 1 - mx
end

function M.hwb_to_rgb(h, w, b)
  w, b = max(w, 0), max(b, 0)
  local sum = w + b
  if sum > 1 then
    w, b = w / sum, b / sum
  end
  local hr, hg, hb = hue_rgb(h)
  local k = 1 - w - b
  return hr * k + w, hg * k + w, hb * k + w
end

local function in_gamut(r, g, b)
  local e = max(abs(r), abs(g), abs(b)) * 0.000002 + 0.000000001
  return r >= -e and g >= -e and b >= -e and r <= 1 + e and g <= 1 + e and b <= 1 + e
end

-- Oklch (h, C, L) をリニアへ。sRGB に無い色は、明るさと色相を保ったまま C を下げて収める（二分探索 16 回）。
-- 範囲の判定は float32 の誤差の分だけ、値の大きさに比例して許す（決まった幅で許すと、暗い色で切り詰めたときに色相がずれる。v1.01）
function M.oklch_to_linear_fit(h, c, L)
  local r, g, b = M.oklab_to_linear(M.polar_to_lab(h, c, L))
  if not in_gamut(r, g, b) then
    local lo, hi = 0, max(c, 0)
    for _ = 1, 16 do
      local mid = (lo + hi) * 0.5
      if in_gamut(M.oklab_to_linear(M.polar_to_lab(h, mid, L))) then
        lo = mid
      else
        hi = mid
      end
    end
    r, g, b = M.oklab_to_linear(M.polar_to_lab(h, lo, L))
  end
  return r, g, b
end

-- sRGB → 色空間の値
function M.to_space(r, g, b, space)
  if space > 8.5 then
    return M.rgb_to_hwb(r, g, b)
  elseif space > 7.5 then
    return M.linear_to_xyz(M.to_linear(r, g, b))
  elseif space > 6.5 then
    return M.lab_to_polar(M.linear_to_oklab(M.to_linear(r, g, b)))
  elseif space > 5.5 then
    return M.lab_to_polar(M.xyz_to_lab(M.linear_to_xyz(M.to_linear(r, g, b))))
  elseif space > 4.5 then
    return M.xyz_to_lab(M.linear_to_xyz(M.to_linear(r, g, b)))
  elseif space > 3.5 then
    return M.rgb_to_hsl(r, g, b)
  elseif space > 2.5 then
    return M.rgb_to_hsv(r, g, b)
  elseif space > 1.5 then
    return M.linear_to_oklab(M.to_linear(r, g, b))
  elseif space > 0.5 then
    return M.to_linear(r, g, b)
  end
  return r, g, b
end

-- 色空間の値 → sRGB（0〜1）。Oklch だけは C を下げて sRGB に収め、ほかは切り詰める
function M.from_space(x, y, z, space)
  local r, g, b
  if space > 8.5 then
    r, g, b = M.hwb_to_rgb(x, y, z)
  elseif space > 7.5 then
    return M.to_srgb(M.xyz_to_linear(x, y, z))
  elseif space > 6.5 then
    return M.to_srgb(M.oklch_to_linear_fit(x, y, z))
  elseif space > 5.5 then
    return M.to_srgb(M.xyz_to_linear(M.lab_to_xyz(M.polar_to_lab(x, y, z))))
  elseif space > 4.5 then
    return M.to_srgb(M.xyz_to_linear(M.lab_to_xyz(x, y, z)))
  elseif space > 3.5 then
    r, g, b = M.hsl_to_rgb(x, y, z)
  elseif space > 2.5 then
    r, g, b = M.hsv_to_rgb(x, y, z)
  elseif space > 1.5 then
    return M.to_srgb(M.oklab_to_linear(x, y, z))
  elseif space > 0.5 then
    return M.to_srgb(x, y, z)
  else
    r, g, b = x, y, z
  end
  return clamp01(r), clamp01(g), clamp01(b)
end

function M.is_polar(space)
  return (space > 2.5 and space < 4.5) or (space > 5.5 and space < 7.5) or space > 8.5
end

-- 色相を持たない（無彩色の）値か
function M.no_hue(y, z, space)
  if space > 8.5 then return y + z > 0.9999 end
  if space > 6.5 then return y < 0.0002 end
  if space > 5.5 then return y < 0.05 end
  return y < 0.0001
end

-- 色空間の値どうしの補間。a0 / a1 は不透明度で、乗算済みのまま混ぜる（透明へ向かっても黒ずまない）。
-- 色相のある色空間では色相だけは乗算せず、無彩色の側は相手の色相に合わせる。hue_long が真で長い方を回る。
-- 戻り値: 色空間の値 x, y, z と不透明度
function M.mix_values(x0, y0, z0, a0, x1, y1, z1, a1, u, space, hue_long)
  local a = a0 + (a1 - a0) * u
  if M.is_polar(space) then
    if M.no_hue(y0, z0, space) then
      x0 = x1
    elseif M.no_hue(y1, z1, space) then
      x1 = x0
    end
    local dh = x1 - x0
    if hue_long then
      if dh > 0 and dh < 0.5 then
        dh = dh - 1
      elseif dh <= 0 and dh > -0.5 then
        dh = dh + 1
      end
    else
      if dh > 0.5 then
        dh = dh - 1
      elseif dh < -0.5 then
        dh = dh + 1
      end
    end
    local y, z
    if a > 0.000001 then
      y = (y0 * a0 + (y1 * a1 - y0 * a0) * u) / a
      z = (z0 * a0 + (z1 * a1 - z0 * a0) * u) / a
    else
      y = y0 + (y1 - y0) * u
      z = z0 + (z1 - z0) * u
    end
    return frac(x0 + dh * u + 1), y, z, a
  end
  if a > 0.000001 then
    return (x0 * a0 + (x1 * a1 - x0 * a0) * u) / a, (y0 * a0 + (y1 * a1 - y0 * a0) * u) / a,
      (z0 * a0 + (z1 * a1 - z0 * a0) * u) / a, a
  end
  return x0 + (x1 - x0) * u, y0 + (y1 - y0) * u, z0 + (z1 - z0) * u, a
end

-- ストレートの sRGB 2 色を色空間で補間する。戻り値は r, g, b, a
function M.mix(r0, g0, b0, a0, r1, g1, b1, a1, u, space, hue_long)
  local x0, y0, z0 = M.to_space(r0, g0, b0, space)
  local x1, y1, z1 = M.to_space(r1, g1, b1, space)
  local x, y, z, a = M.mix_values(x0, y0, z0, a0, x1, y1, z1, a1, u, space, hue_long)
  local r, g, b = M.from_space(x, y, z, space)
  return r, g, b, a
end

-- 0xRRGGBB の 2 色を補間して 0xRRGGBB を返す（不透明どうし）
function M.mix_color(col0, col1, u, space, hue_long)
  local r0, g0, b0 = M.unpack(col0)
  local r1, g1, b1 = M.unpack(col1)
  local r, g, b = M.mix(r0, g0, b0, 1, r1, g1, b1, 1, u, space, hue_long)
  return M.pack(r, g, b)
end

-- 色の並び（0xRRGGBB の配列）を先に色空間へ変換しておく。sample で何度も引くとき用
function M.prepare(cols, space)
  local stops = { space = space }
  for i = 1, #cols do
    local r, g, b = M.unpack(cols[i])
    local x, y, z = M.to_space(r, g, b, space)
    stops[i] = { x, y, z }
  end
  return stops
end

-- prepare した並びの t（0〜1）の色を r, g, b で返す。並びは等間隔とみなす
function M.sample(stops, t, hue_long)
  local n = #stops
  if n == 0 then return 0, 0, 0 end
  if n == 1 then return M.from_space(stops[1][1], stops[1][2], stops[1][3], stops.space) end
  local p = clamp01(t) * (n - 1)
  local i = min(floor(p), n - 2)
  local s0, s1 = stops[i + 1], stops[i + 2]
  local x, y, z = M.mix_values(s0[1], s0[2], s0[3], 1, s1[1], s1[2], s1[3], 1, p - i, stops.space, hue_long)
  return M.from_space(x, y, z, stops.space)
end

-- 色相を turns 周だけ回す。space: 3 HSV / 4 HSL / 6 LCh / 7 Oklch / 9 HWB（それ以外は HSV）
function M.hue_shift(r, g, b, turns, space)
  local sp = M.is_polar(space) and space or 3
  local x, y, z = M.to_space(r, g, b, sp)
  return M.from_space(frac(x + turns + 1), y, z, sp)
end

-- 鮮やかさを k 倍にする。space: 0 sRGB（Rec.709 の輝度との間で伸ばす）/ 3 HSV / 4 HSL / 6 LCh / 7 Oklch
function M.scale_chroma(r, g, b, k, space)
  if M.is_polar(space) and space < 8.5 then
    local x, y, z = M.to_space(r, g, b, space)
    return M.from_space(x, max(y * k, 0), z, space)
  end
  local l = 0.2126 * r + 0.7152 * g + 0.0722 * b
  return clamp01(l + (r - l) * k), clamp01(l + (g - l) * k), clamp01(l + (b - l) * k)
end

-- 明るさ（0〜1）。space: 0 sRGB（Rec.709 をガンマのまま）/ 1 リニア（相対輝度 Y）/ 2・7 Oklab の L / 5・6 Lab の L / 3 HSV の V / 4 HSL の L
function M.lightness(r, g, b, space)
  if (space > 6.5 and space < 7.5) or (space > 1.5 and space < 2.5) then
    return (M.linear_to_oklab(M.to_linear(r, g, b)))
  elseif space > 4.5 and space < 6.5 then
    return (M.xyz_to_lab(M.linear_to_xyz(M.to_linear(r, g, b)))) / 100
  elseif space > 3.5 and space < 4.5 then
    return (max(r, g, b) + min(r, g, b)) * 0.5
  elseif space > 2.5 and space < 3.5 then
    return max(r, g, b)
  elseif space > 0.5 and space < 1.5 then
    local lr, lg, lb = M.to_linear(r, g, b)
    return 0.2126729 * lr + 0.7151522 * lg + 0.0721750 * lb
  end
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
end

-- CIEDE2000（Sharma, Wu, Dalal 2005）。Lab（D65）の値どうし
function M.ciede2000(L1, a1, b1, L2, a2, b2)
  local c1 = sqrt(a1 * a1 + b1 * b1)
  local c2 = sqrt(a2 * a2 + b2 * b2)
  local cb7 = ((c1 + c2) * 0.5) ^ 7
  local g = 0.5 * (1 - sqrt(cb7 / (cb7 + 6103515625)))
  local a1p, a2p = a1 * (1 + g), a2 * (1 + g)
  local c1p = sqrt(a1p * a1p + b1 * b1)
  local c2p = sqrt(a2p * a2p + b2 * b2)
  local h1p = (abs(a1p) + abs(b1)) > 0 and atan2(b1, a1p) or 0
  local h2p = (abs(a2p) + abs(b2)) > 0 and atan2(b2, a2p) or 0
  if h1p < 0 then h1p = h1p + TAU end
  if h2p < 0 then h2p = h2p + TAU end
  local dl = L2 - L1
  local dc = c2p - c1p
  local dh = 0
  local hbp = h1p + h2p
  if c1p * c2p > 0 then
    dh = h2p - h1p
    if dh > math.pi then
      dh = dh - TAU
    elseif dh < -math.pi then
      dh = dh + TAU
    end
    if abs(h1p - h2p) > math.pi then
      hbp = (h1p + h2p < TAU) and (h1p + h2p + TAU) * 0.5 or (h1p + h2p - TAU) * 0.5
    else
      hbp = (h1p + h2p) * 0.5
    end
  end
  local dhh = 2 * sqrt(c1p * c2p) * sin(dh * 0.5)
  local lbp = (L1 + L2) * 0.5
  local cbp = (c1p + c2p) * 0.5
  local t = 1 - 0.17 * cos(hbp - math.pi / 6) + 0.24 * cos(2 * hbp)
    + 0.32 * cos(3 * hbp + math.pi / 30) - 0.20 * cos(4 * hbp - 63 * math.pi / 180)
  local dtheta = math.pi / 6 * exp(-((hbp * 180 / math.pi - 275) / 25) ^ 2)
  local cbp7 = cbp ^ 7
  local rc = 2 * sqrt(cbp7 / (cbp7 + 6103515625))
  local l50 = (lbp - 50) ^ 2
  local sl = 1 + 0.015 * l50 / sqrt(20 + l50)
  local sc = 1 + 0.045 * cbp
  local sh = 1 + 0.015 * cbp * t
  local rt = -sin(2 * dtheta) * rc
  local tl, tc, th = dl / sl, dc / sc, dhh / sh
  return sqrt(max(tl * tl + tc * tc + th * th + rt * tc * th, 0))
end

-- 2 色の差。metric: 0 RGB / 1 YCbCr（BT.709 の色差だけ）/ 2 Oklab / 3 CIEDE2000。黒と白の差がおよそ 1
function M.distance(r0, g0, b0, r1, g1, b1, metric)
  if metric > 2.5 then
    local L1, a1, b1_ = M.xyz_to_lab(M.linear_to_xyz(M.to_linear(r0, g0, b0)))
    local L2, a2, b2_ = M.xyz_to_lab(M.linear_to_xyz(M.to_linear(r1, g1, b1)))
    return M.ciede2000(L1, a1, b1_, L2, a2, b2_) / 100
  elseif metric > 1.5 then
    local L1, a1, b1_ = M.linear_to_oklab(M.to_linear(r0, g0, b0))
    local L2, a2, b2_ = M.linear_to_oklab(M.to_linear(r1, g1, b1))
    return sqrt((L1 - L2) ^ 2 + (a1 - a2) ^ 2 + (b1_ - b2_) ^ 2)
  elseif metric > 0.5 then
    local dr, dg, db = r0 - r1, g0 - g1, b0 - b1
    local cb = -0.1146 * dr - 0.3854 * dg + 0.5 * db
    local cr = 0.5 * dr - 0.4542 * dg - 0.0458 * db
    return sqrt(cb * cb + cr * cr) * 1.41421356
  end
  return sqrt((r0 - r1) ^ 2 + (g0 - g1) ^ 2 + (b0 - b1) ^ 2) * 0.57735027
end

return M
