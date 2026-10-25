local util = require("rs_util")

local M = {}

M.CIRCLE_SEGMENTS = 16
M.KINDS = { "circle", "square", "triangle", "polygon", "line" }

function M.kind_for(shape, index)
    if shape ~= "mixed" then return shape or "circle" end
    local i = index or 1
    return M.KINDS[(i - 1) % #M.KINDS + 1]
end

local function vertex(x, y, r, g, b, a)
    return { x, y, 0, r, g, b, a }
end

local function add_tri(dst, x0, y0, x1, y1, x2, y2, r, g, b, a)
    dst[#dst + 1] = vertex(x0, y0, r, g, b, a)
    dst[#dst + 1] = vertex(x1, y1, r, g, b, a)
    dst[#dst + 1] = vertex(x2, y2, r, g, b, a)
end

local function rotate(x, y, deg)
    local a = math.rad(deg or 0)
    local c = math.cos(a)
    local s = math.sin(a)
    return x * c - y * s, x * s + y * c
end

local function place(x, y, deg, cx, cy)
    local rx, ry = rotate(x, y, deg)
    return cx + rx, cy + ry
end

local function polygon_points(sides, radius)
    local pts = {}
    for i = 0, sides - 1 do
        local a = -math.pi / 2 + i * math.pi * 2 / sides
        pts[#pts + 1] = { math.cos(a) * radius, math.sin(a) * radius }
    end
    return pts
end

local function add_fan(dst, pts, deg, cx, cy, r, g, b, a)
    local n = #pts
    if n < 3 then return end
    local c0x, c0y = place(0, 0, deg, cx, cy)
    for i = 1, n do
        local p0 = pts[i]
        local p1 = pts[i % n + 1]
        local x0, y0 = place(p0[1], p0[2], deg, cx, cy)
        local x1, y1 = place(p1[1], p1[2], deg, cx, cy)
        -- Y-up で時計回りになるよう、中心・次頂点・現頂点の順にする。
        add_tri(dst, c0x, c0y, x1, y1, x0, y0, r, g, b, a)
    end
end

local function add_edge(dst, x0, y0, x1, y1, width, r, g, b, a)
    local dx = x1 - x0
    local dy = y1 - y0
    local len = math.sqrt(dx * dx + dy * dy)
    if len < 1e-6 then return end
    local nx = -dy / len * width * 0.5
    local ny = dx / len * width * 0.5
    add_tri(dst, x0 + nx, y0 + ny, x1 + nx, y1 + ny, x1 - nx, y1 - ny, r, g, b, a)
    add_tri(dst, x0 + nx, y0 + ny, x1 - nx, y1 - ny, x0 - nx, y0 - ny, r, g, b, a)
end

local function add_outline(dst, pts, deg, cx, cy, width, r, g, b, a)
    local placed = {}
    for i = 1, #pts do
        local x, y = place(pts[i][1], pts[i][2], deg, cx, cy)
        placed[i] = { x, y }
    end
    for i = 1, #placed do
        local p0 = placed[i]
        local p1 = placed[i % #placed + 1]
        add_edge(dst, p0[1], p0[2], p1[1], p1[2], width, r, g, b, a)
    end
end

function M.build(spec)
    spec = spec or {}
    local kind = spec.kind or "circle"
    local cx = spec.x or 0
    local cy = spec.y or 0
    local size = tonumber(spec.size) or 32
    if size < 1 then size = 1 end
    local radius = size * 0.5
    local deg = spec.rotation or 0
    local alpha = spec.alpha
    if alpha == nil then alpha = 1 end
    local line_width = tonumber(spec.line_width) or 2
    if line_width < 0.5 then line_width = 0.5 end
    local fill = spec.fill
    if fill == nil then fill = true end
    local outline = spec.outline
    if outline == nil then outline = true end
    local r, g, b, a = util.premul(spec.color or 0xffffff, alpha)
    local dst = {}
    if a <= 0.001 then return dst end

    if kind == "rect" then
        local w = tonumber(spec.w) or size
        local h = tonumber(spec.h) or size
        if w < 0 then w = 0 end
        if h < 0 then h = 0 end
        local hw = w * 0.5
        local hh = h * 0.5
        -- Y-up で反時計回り。add_fan と同じく中心・次・現在で時計回りにする。
        local corners = { { -hw, -hh }, { hw, -hh }, { hw, hh }, { -hw, hh } }
        for i = 1, 4 do
            local cur = corners[i]
            local nxt = corners[i % 4 + 1]
            local x0, y0 = place(cur[1], cur[2], deg, cx, cy)
            local x1, y1 = place(nxt[1], nxt[2], deg, cx, cy)
            add_tri(dst, cx, cy, x1, y1, x0, y0, r, g, b, a)
        end
        return dst
    end

    if kind == "line" then
        local x0, y0 = place(-radius, 0, deg, cx, cy)
        local x1, y1 = place(radius, 0, deg, cx, cy)
        add_edge(dst, x0, y0, x1, y1, math.max(line_width, 1), r, g, b, a)
        return dst
    end

    local sides = math.floor(tonumber(spec.sides) or 6)
    if kind == "square" then
        sides = 4
    elseif kind == "triangle" then
        sides = 3
    elseif kind == "circle" then
        sides = M.CIRCLE_SEGMENTS
    end
    if sides < 3 then sides = 3 end
    if sides > 32 then sides = 32 end
    local pts = polygon_points(sides, radius)
    if fill then
        if sides == 3 then
            local p = {}
            for i = 1, 3 do
                local x, y = place(pts[i][1], pts[i][2], deg, cx, cy)
                p[i] = { x, y }
            end
            add_tri(dst, p[1][1], p[1][2], p[3][1], p[3][2], p[2][1], p[2][2], r, g, b, a)
        else
            add_fan(dst, pts, deg, cx, cy, r, g, b, a)
        end
    end
    if outline then
        add_outline(dst, pts, deg, cx, cy, line_width, r, g, b, a)
    end
    return dst
end

return M
