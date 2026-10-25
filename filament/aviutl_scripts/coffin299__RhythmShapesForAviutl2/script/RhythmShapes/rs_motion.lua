local util = require("rs_util")

local M = {}

function M.ease(name, t)
    t = util.clamp(t or 0, 0, 1)
    if name == "outQuad" then
        local u = 1 - t
        return 1 - u * u
    elseif name == "outCubic" then
        local u = 1 - t
        return 1 - u * u * u
    elseif name == "outExpo" then
        if t >= 1 then return 1 end
        if t <= 0 then return 0 end
        return 1 - 2 ^ (-10 * t)
    end
    return t
end

-- age=0 で 1、age>=decay で 0。decay 秒だけで決まる。
function M.envelope(age, decay, name)
    decay = tonumber(decay) or 0
    age = tonumber(age) or 0
    if age < 0 then age = 0 end
    if decay <= 0 then
        if age <= 0 then return 1 end
        return 0
    end
    if age >= decay then return 0 end
    return M.ease(name, 1 - age / decay)
end

local function level(ctx, key)
    local audio = ctx.audio or {}
    return util.clamp(tonumber(audio[key]) or 0, 0, 1)
end

function M.driver(source, ctx, ease_name)
    source = source or "beat"
    if source == "none" then return 1 end
    if source == "rms" or source == "bass" or source == "mid" or source == "high" then
        return level(ctx, source)
    end
    if source == "transient" then
        local beat = ctx.beat or {}
        local age = beat.age
        if age == nil then age = math.huge end
        local env = M.envelope(age, ctx.decay, ease_name)
        return util.clamp(env * (tonumber(beat.strength) or 0), 0, 1)
    end

    local grid = ctx.grid or {}
    local period = grid.sub_period or grid.period or 0.5
    local phase = grid.sub_phase or 0
    if source == "downbeat" then
        phase = grid.beat_phase or 0
        period = grid.period or 0.5
        if not grid.is_downbeat then return 0 end
    end
    local stagger = tonumber(ctx.stagger) or 0
    local age = phase * period + stagger * period
    if period > 0 then age = age % period end
    local env = M.envelope(age, ctx.decay, ease_name)
    local accent = tonumber(ctx.accent) or 0
    if grid.is_downbeat then
        env = env * (1 + accent)
    end
    return env
end

return M
