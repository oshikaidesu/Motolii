local M = {}

function M.clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

function M.lerp(a, b, t)
    return a + (b - a) * t
end

-- 32bit LCG。同じシードなら呼び出し順に同じ列を返す。
function M.rng(seed)
    local s = tonumber(seed) or 1
    s = s % 4294967296
    if s < 0 then s = s + 4294967296 end
    if s == 0 then s = 1 end
    return function()
        s = (s * 1664525 + 1013904223) % 4294967296
        return s / 4294967296
    end
end

function M.unpack_rgb(color)
    color = tonumber(color) or 0
    if color < 0 then color = 0 end
    local r = math.floor(color / 65536) % 256
    local g = math.floor(color / 256) % 256
    local b = math.floor(color) % 256
    return r / 255, g / 255, b / 255
end

function M.pack_rgb(r, g, b)
    r = math.floor(M.clamp(r, 0, 1) * 255 + 0.5)
    g = math.floor(M.clamp(g, 0, 1) * 255 + 0.5)
    b = math.floor(M.clamp(b, 0, 1) * 255 + 0.5)
    return r * 65536 + g * 256 + b
end

function M.rgb_to_hsv(r, g, b)
    local max = math.max(r, g, b)
    local min = math.min(r, g, b)
    local d = max - min
    local h = 0
    if d > 1e-8 then
        if max == r then
            h = ((g - b) / d) % 6
        elseif max == g then
            h = (b - r) / d + 2
        else
            h = (r - g) / d + 4
        end
        h = h * 60
        if h < 0 then h = h + 360 end
    end
    local s = 0
    if max > 1e-8 then s = d / max end
    return h, s, max
end

function M.hsv_to_rgb(h, s, v)
    h = h % 360
    local c = v * s
    local x = c * (1 - math.abs((h / 60) % 2 - 1))
    local m = v - c
    local r, g, b = 0, 0, 0
    if h < 60 then
        r, g, b = c, x, 0
    elseif h < 120 then
        r, g, b = x, c, 0
    elseif h < 180 then
        r, g, b = 0, c, x
    elseif h < 240 then
        r, g, b = 0, x, c
    elseif h < 300 then
        r, g, b = x, 0, c
    else
        r, g, b = c, 0, x
    end
    return r + m, g + m, b + m
end

function M.shift_color(color, degrees)
    local r, g, b = M.unpack_rgb(color)
    local h, s, v = M.rgb_to_hsv(r, g, b)
    r, g, b = M.hsv_to_rgb(h + (degrees or 0), s, v)
    return M.pack_rgb(r, g, b)
end

function M.premul(color, alpha)
    local r, g, b = M.unpack_rgb(color)
    alpha = M.clamp(alpha or 1, 0, 1)
    return r * alpha, g * alpha, b * alpha, alpha
end

function M.copy(src)
    local dst = {}
    for k, v in pairs(src) do
        dst[k] = v
    end
    return dst
end

return M
