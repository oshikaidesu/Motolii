local util = require("rs_util")

local M = {}

M.SHAPES = { "circle", "square", "triangle", "polygon", "line", "mixed" }
M.LAYOUTS = { "scatter", "grid", "orbit", "radial", "figure" }
M.EASES = { "linear", "outQuad", "outCubic", "outExpo" }
M.MAPS = { "beat", "downbeat", "rms", "bass", "mid", "high", "transient" }

local function profile(extra)
    local p = {
        shape = "circle",
        layout = "scatter",
        ease = "outQuad",
        map_scale = "beat",
        map_rotate = "beat",
        map_move = "beat",
        map_alpha = "beat",
        map_color = "beat",
        map_width = "beat",
        map_count = "none",
        spin_dps = 0,
        gain = 1,
    }
    for k, v in pairs(extra) do
        p[k] = v
    end
    return p
end

M.profiles = {
    [0] = profile({}),
    [1] = profile({
        shape = "mixed",
        layout = "scatter",
        ease = "outExpo",
        map_scale = "transient",
        map_rotate = "transient",
        map_move = "beat",
        map_alpha = "transient",
        map_color = "high",
        map_width = "transient",
        map_count = "transient",
        gain = 1.15,
    }),
    [2] = profile({
        shape = "circle",
        layout = "orbit",
        ease = "outQuad",
        map_scale = "bass",
        map_rotate = "beat",
        map_move = "beat",
        map_alpha = "bass",
        map_color = "mid",
        map_width = "beat",
        spin_dps = 18,
    }),
    [3] = profile({
        shape = "square",
        layout = "grid",
        ease = "outCubic",
        map_scale = "beat",
        map_rotate = "downbeat",
        map_move = "beat",
        map_alpha = "beat",
        map_color = "downbeat",
        map_width = "beat",
    }),
    [4] = profile({
        shape = "circle",
        layout = "radial",
        ease = "outQuad",
        map_scale = "bass",
        map_rotate = "bass",
        map_move = "bass",
        map_alpha = "bass",
        map_color = "bass",
        map_width = "bass",
    }),
    [5] = profile({
        shape = "triangle",
        layout = "grid",
        ease = "linear",
        map_scale = "beat",
        map_rotate = "beat",
        map_move = "beat",
        map_alpha = "beat",
        map_color = "beat",
        map_width = "beat",
        gain = 0.35,
    }),
    [6] = profile({
        shape = "mixed",
        layout = "figure",
        ease = "outQuad",
        map_scale = "beat",
        map_rotate = "beat",
        map_move = "beat",
        map_alpha = "beat",
        map_color = "beat",
        map_width = "beat",
        map_count = "none",
        gain = 1,
    }),
}

M.names = {
    [0] = "カスタム",
    [1] = "Beat Burst",
    [2] = "Geometric Orbit",
    [3] = "Rhythm Grid",
    [4] = "Bass Pulse",
    [5] = "Minimal Motion",
    [6] = "フラット図形",
}

M.required_keys = {
    "shape", "layout", "ease",
    "map_scale", "map_rotate", "map_move", "map_alpha", "map_color", "map_width", "map_count",
    "spin_dps", "gain",
}

local function pick(list, index, fallback)
    index = math.floor(tonumber(index) or 0)
    return list[index + 1] or fallback
end

-- トラックバーと色は常に ui の値。プリセットは配置・反応・イージングを決める。
function M.apply(ui)
    ui = ui or {}
    local id = math.floor(tonumber(ui.preset) or 0)
    local base = M.profiles[id] or M.profiles[0]
    local cfg = util.copy(base)
    cfg.preset = id
    cfg.count = tonumber(ui.count) or 24
    cfg.size = tonumber(ui.size) or 48
    cfg.line_width = tonumber(ui.line_width) or 3
    cfg.alpha = tonumber(ui.alpha) or 90
    cfg.seed = tonumber(ui.seed) or 1
    cfg.sides = tonumber(ui.sides) or 6
    cfg.fill = ui.fill
    if cfg.fill == nil then cfg.fill = true end
    cfg.outline = ui.outline
    if cfg.outline == nil then cfg.outline = true end
    cfg.color = tonumber(ui.color) or 0xffffff
    cfg.bpm = tonumber(ui.bpm) or 120
    cfg.offset_ms = tonumber(ui.offset_ms) or 0
    cfg.sensitivity = tonumber(ui.sensitivity) or 50
    cfg.min_interval_ms = tonumber(ui.min_interval_ms) or 120
    cfg.weight_bass = tonumber(ui.weight_bass) or 100
    cfg.weight_mid = tonumber(ui.weight_mid) or 40
    cfg.weight_high = tonumber(ui.weight_high) or 20
    cfg.scale_amt = tonumber(ui.scale_amt) or 80
    cfg.rotate_amt = tonumber(ui.rotate_amt) or 25
    cfg.move_amt = tonumber(ui.move_amt) or 30
    cfg.decay_ms = tonumber(ui.decay_ms) or 180
    cfg.accent = (tonumber(ui.accent) or 40) / 100
    cfg.subdivision = tonumber(ui.subdivision) or 1
    cfg.sync_mode = math.floor(tonumber(ui.sync_mode) or 0)
    if id == 0 then
        cfg.shape = pick(M.SHAPES, ui.shape, "circle")
        cfg.layout = pick(M.LAYOUTS, ui.layout, "scatter")
        cfg.ease = pick(M.EASES, ui.ease, "outQuad")
        cfg.map_scale = pick(M.MAPS, ui.map_scale, "beat")
        cfg.map_rotate = pick(M.MAPS, ui.map_rotate, "beat")
        cfg.map_move = pick(M.MAPS, ui.map_move, "beat")
        cfg.map_alpha = pick(M.MAPS, ui.map_alpha, "beat")
        cfg.map_color = pick(M.MAPS, ui.map_color, "beat")
        cfg.map_width = pick(M.MAPS, ui.map_width, "beat")
    end
    return cfg
end

return M
