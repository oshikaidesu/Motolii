-- 参考画像の構図。文字は描かない。座標は画面左上を 0、右下を 1 とする。
local M = {}

local function box(nx, ny, nw, nh, sw, sh, extra)
    extra.kind = "rect"
    extra.x = (nx + nw * 0.5 - 0.5) * sw
    extra.y = (0.5 - (ny + nh * 0.5)) * sh
    extra.w = nw * sw
    extra.h = nh * sh
    extra.rot = extra.rot or 0
    return extra
end

local function push(parts, item)
    parts[#parts + 1] = item
end

function M.parts(screen_w, screen_h)
    local sw = tonumber(screen_w) or 1920
    local sh = tonumber(screen_h) or 1080
    local parts = {}

    push(parts, box(0, 0, 1, 1, sw, sh, {
        role = "ground",
        drive = "none",
        pulse = 0,
        bg = true,
    }))
    push(parts, box(0.11, 0.03, 0.105, 0.66, sw, sh, {
        role = "bar",
        drive = "high",
        pulse = 0.22,
        axis = "y",
    }))
    push(parts, box(0.225, 0.275, 0.175, 0.016, sw, sh, {
        role = "staff",
        drive = "mid",
        pulse = 0.28,
        axis = "x",
    }))
    push(parts, box(0.225, 0.355, 0.175, 0.016, sw, sh, {
        role = "staff",
        drive = "mid",
        pulse = 0.28,
        axis = "x",
    }))

    local body = {
        { 0.42, 0.20, 0.26, 0.155 },
        { 0.69, 0.20, 0.075, 0.155 },
        { 0.42, 0.375, 0.22, 0.145 },
        { 0.65, 0.375, 0.07, 0.145 },
        { 0.42, 0.54, 0.105, 0.12 },
        { 0.535, 0.54, 0.09, 0.12 },
    }
    for i = 1, #body do
        local b = body[i]
        push(parts, box(b[1], b[2], b[3], b[4], sw, sh, {
            role = "body",
            drive = "beat",
            pulse = 0.06,
            axis = "both",
        }))
    end

    local key_x = 0.30
    local key_y = 0.46
    local kw = 0.0165
    local gap = 0.0035
    for col = 0, 6 do
        for row = 0, 1 do
            local h = row == 0 and 0.055 or 0.07
            local y = key_y + (row == 0 and 0 or 0.012)
            push(parts, box(key_x + col * (kw + gap), y, kw, h, sw, sh, {
                role = "key",
                drive = "mid",
                pulse = 0.18,
                axis = "y",
            }))
        end
    end

    local head = 0.105 * sw
    push(parts, {
        kind = "circle",
        role = "head",
        drive = "beat",
        pulse = 0.12,
        axis = "both",
        x = (0.835 - 0.5) * sw,
        y = (0.5 - 0.255) * sh,
        size = head,
        w = head,
        h = head,
        rot = 0,
    })
    local mark = 0.20 * sh
    push(parts, {
        kind = "triangle",
        role = "mark",
        drive = "bass",
        pulse = 0.14,
        axis = "both",
        x = (0.80 - 0.5) * sw,
        y = (0.5 - 0.60) * sh,
        size = mark,
        w = mark,
        h = mark,
        rot = 90,
    })
    return parts
end

return M
