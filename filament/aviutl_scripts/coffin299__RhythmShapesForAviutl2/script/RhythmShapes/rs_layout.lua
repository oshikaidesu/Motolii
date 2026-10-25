local util = require("rs_util")

local M = {}

local function push(out, x, y, rot)
    out[#out + 1] = { x = x, y = y, rot = rot or 0 }
end

function M.positions(mode, count, seed, screen_w, screen_h, margin)
    count = math.floor(tonumber(count) or 0)
    if count < 1 then count = 1 end
    screen_w = tonumber(screen_w) or 1920
    screen_h = tonumber(screen_h) or 1080
    margin = tonumber(margin) or 0
    local hw = screen_w / 2 - margin
    local hh = screen_h / 2 - margin
    if hw < 1 then hw = 1 end
    if hh < 1 then hh = 1 end
    local out = {}
    mode = mode or "scatter"

    if mode == "grid" then
        local cols = math.ceil(math.sqrt(count))
        local rows = math.ceil(count / cols)
        local gap_x = 0
        local gap_y = 0
        if cols > 1 then gap_x = (hw * 2) / (cols - 1) end
        if rows > 1 then gap_y = (hh * 2) / (rows - 1) end
        local i = 1
        for r = 1, rows do
            for c = 1, cols do
                if i > count then break end
                local x = 0
                local y = 0
                if cols == 1 then x = 0 else x = -hw + (c - 1) * gap_x end
                if rows == 1 then y = 0 else y = hh - (r - 1) * gap_y end
                push(out, x, y, 0)
                i = i + 1
            end
        end
    elseif mode == "orbit" then
        local rx = hw * 0.82
        local ry = hh * 0.82
        for i = 1, count do
            local a = (i - 1) / count * math.pi * 2 - math.pi / 2
            push(out, math.cos(a) * rx, math.sin(a) * ry, a * 180 / math.pi)
        end
    elseif mode == "radial" then
        local rand = util.rng((tonumber(seed) or 1) + 17)
        for i = 1, count do
            local a = (i - 1) * 2.399963229728653
            local rad = math.sqrt(i / count)
            local jx = (rand() - 0.5) * hw * 0.04
            local jy = (rand() - 0.5) * hh * 0.04
            push(out, math.cos(a) * rad * hw + jx, math.sin(a) * rad * hh + jy, a * 180 / math.pi)
        end
    else
        local rand = util.rng(seed)
        for i = 1, count do
            local x = (rand() * 2 - 1) * hw
            local y = (rand() * 2 - 1) * hh
            local rot = rand() * 360
            push(out, x, y, rot)
        end
    end
    for i = 1, #out do
        out[i].x = util.clamp(out[i].x, -hw, hw)
        out[i].y = util.clamp(out[i].y, -hh, hh)
    end
    return out
end

return M
