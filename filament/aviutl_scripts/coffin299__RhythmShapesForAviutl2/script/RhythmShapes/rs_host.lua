local util = require("rs_util")
local audio = require("rs_audio")
local beatdet = require("rs_beat")
local grid = require("rs_grid")
local motion = require("rs_motion")
local layout = require("rs_layout")
local shape = require("rs_shape")
local preset = require("rs_preset")
local figure = require("rs_figure")

local M = {}

-- 実測前の上限。プローブの script_time を見てから変える。
M.MAX_SHAPES = 64
M.MAX_PCM = 8192
M.MAX_BUFFER = 3840

function M.scene_time(frame_s, frame, framerate)
    local fps = tonumber(framerate) or 0
    if fps <= 0 then fps = 60 end
    return ((tonumber(frame_s) or 0) + (tonumber(frame) or 0)) / fps
end

function M.even(n)
    n = math.floor(tonumber(n) or 0)
    if n < 2 then n = 2 end
    if n % 2 == 1 then n = n + 1 end
    return n
end

local function outward(x, y, amount)
    local len = math.sqrt(x * x + y * y)
    if len < 0.001 then
        return x, y + amount
    end
    return x + x / len * amount, y + y / len * amount
end

function M.build_frame(input)
    input = input or {}
    local ui = input.ui or {}
    local cfg = preset.apply(ui)
    local max_shapes = math.floor(tonumber(input.max_shapes) or M.MAX_SHAPES)
    if max_shapes < 1 then max_shapes = 1 end
    local count = math.floor(cfg.count or 1)
    if count < 1 then count = 1 end
    if count > max_shapes then count = max_shapes end

    local analyzed = audio.analyze({
        pcm = input.pcm,
        fourier = input.fourier,
        spectrum = input.spectrum,
        rate = input.rate,
    })
    local detected = beatdet.detect(analyzed.pcm, analyzed.rate, {
        sensitivity = cfg.sensitivity,
        min_interval = (cfg.min_interval_ms or 120) / 1000,
        weights = {
            bass = cfg.weight_bass,
            mid = cfg.weight_mid,
            high = cfg.weight_high,
        },
    })

    local list = nil
    if cfg.sync_mode == 0 then
        list = input.bpm_list
    end
    local scene = tonumber(input.scene_time) or 0
    local clock = grid.query(scene, list, {
        tempo = cfg.bpm,
        beat = 4,
        offset = (cfg.offset_ms or 0) / 1000,
        subdivision = cfg.subdivision,
    })

    local sw = tonumber(input.screen_w) or 0
    local sh = tonumber(input.screen_h) or 0
    if sw < 2 then sw = 1920 end
    if sh < 2 then sh = 1080 end
    if sw > M.MAX_BUFFER then sw = M.MAX_BUFFER end
    if sh > M.MAX_BUFFER then sh = M.MAX_BUFFER end
    sw = M.even(sw)
    sh = M.even(sh)
    local margin = (cfg.size or 32) * 0.5 + (cfg.line_width or 2)
    local placed = {}
    if cfg.layout ~= "figure" then
        placed = layout.positions(cfg.layout, count, cfg.seed, sw, sh, margin)
    end

    local decay = (cfg.decay_ms or 180) / 1000
    local function signal(source, index)
        return motion.driver(source, {
            audio = analyzed,
            beat = detected,
            grid = clock,
            decay = decay,
            accent = cfg.accent,
            stagger = (index - 1) / count,
        }, cfg.ease)
    end

    if cfg.layout == "figure" then
        local parts = figure.parts(sw, sh)
        local total = #parts
        local gain = cfg.gain or 1
        local amt = util.clamp((cfg.scale_amt or 0) / 100, 0, 3) * gain
        local alpha = (cfg.alpha or 90) / 100
        local verts = {}
        for i = 1, total do
            local p = parts[i]
            local drv = 0
            if p.drive and p.drive ~= "none" then
                drv = util.clamp(motion.driver(p.drive, {
                    audio = analyzed,
                    beat = detected,
                    grid = clock,
                    decay = decay,
                    accent = cfg.accent,
                    stagger = (i - 1) / math.max(total - 1, 1),
                }, cfg.ease), 0, 1)
            end
            local extra = (p.pulse or 0) * amt * drv
            local sx = 1
            local sy = 1
            if p.axis == "x" then
                sx = 1 + extra
            elseif p.axis == "y" then
                sy = 1 + extra
            elseif p.axis == "both" then
                sx = 1 + extra
                sy = 1 + extra
            end
            local piece = shape.build({
                kind = p.kind,
                x = p.x,
                y = p.y,
                size = (p.size or cfg.size) * ((sx + sy) * 0.5),
                w = (p.w or 0) * sx,
                h = (p.h or 0) * sy,
                rotation = p.rot or 0,
                color = p.bg and cfg.color or 0xffffff,
                alpha = alpha,
                fill = true,
                outline = false,
            })
            for v = 1, #piece do
                verts[#verts + 1] = piece[v]
            end
        end
        return {
            vertices = verts,
            shown = total,
            count = total,
            audio = analyzed,
            beat = detected,
            grid = clock,
            cfg = cfg,
            buffer_w = sw,
            buffer_h = sh,
        }
    end

    local shown = count
    if cfg.map_count and cfg.map_count ~= "none" then
        local drv = util.clamp(signal(cfg.map_count, 1), 0, 1)
        shown = math.floor(count * (0.3 + 0.7 * drv) + 0.5)
        if shown < 1 then shown = 1 end
        if shown > count then shown = count end
    end

    local hw = sw / 2
    local hh = sh / 2
    local verts = {}
    for i = 1, shown do
        local p = placed[i]
        if p then
            local scale_d = signal(cfg.map_scale, i)
            local rot_d = signal(cfg.map_rotate, i)
            local move_d = signal(cfg.map_move, i)
            local alpha_d = util.clamp(signal(cfg.map_alpha, i), 0, 1)
            local color_d = util.clamp(signal(cfg.map_color, i), 0, 1)
            local width_d = util.clamp(signal(cfg.map_width, i), 0, 1)
            local gain = cfg.gain or 1
            local draw_size = cfg.size * (1 + (cfg.scale_amt / 100) * gain * scale_d)
            local rotation = p.rot + (cfg.spin_dps or 0) * scene + cfg.rotate_amt * gain * rot_d
            local x, y = outward(p.x, p.y, cfg.move_amt * gain * move_d)
            local limit = math.max(8, draw_size * 0.5 + cfg.line_width)
            x = util.clamp(x, -hw + limit, hw - limit)
            y = util.clamp(y, -hh + limit, hh - limit)
            local color = util.shift_color(cfg.color, color_d * 48)
            local piece = shape.build({
                kind = shape.kind_for(cfg.shape, i),
                x = x,
                y = y,
                size = draw_size,
                rotation = rotation,
                color = color,
                alpha = (cfg.alpha / 100) * (0.2 + 0.8 * alpha_d),
                line_width = cfg.line_width * (0.45 + 0.55 * width_d),
                fill = cfg.fill,
                outline = cfg.outline,
                sides = cfg.sides,
            })
            for v = 1, #piece do
                verts[#verts + 1] = piece[v]
            end
        end
    end

    return {
        vertices = verts,
        shown = shown,
        count = count,
        audio = analyzed,
        beat = detected,
        grid = clock,
        cfg = cfg,
        buffer_w = sw,
        buffer_h = sh,
    }
end

function M.fetch_audio(getaudio, max_pcm)
    local function grab(kind, size)
        local buf = {}
        local ok, n, rate
        if size then
            ok, n, rate = pcall(getaudio, buf, "audiobuffer", kind, size)
        else
            ok, n, rate = pcall(getaudio, buf, "audiobuffer", kind)
        end
        if not ok or type(n) ~= "number" or n <= 0 then
            return nil, nil
        end
        return buf, rate
    end

    max_pcm = max_pcm or M.MAX_PCM
    local pcm, rate = grab("pcm", max_pcm)
    if not pcm then
        pcm, rate = grab("pcm.l", max_pcm)
    end
    local fourier, frate = grab("fourier")
    if frate and not rate then rate = frate end
    local spectrum = nil
    if not fourier then
        spectrum, frate = grab("spectrum", 64)
        if frate and not rate then rate = frate end
    end
    return {
        pcm = pcm,
        fourier = fourier,
        spectrum = spectrum,
        rate = rate or 0,
    }
end

function M.fetch_bpm(getinfo)
    local ok, list = pcall(getinfo, "bpm_list")
    if ok and type(list) == "table" and #list > 0 then
        return list
    end
    local ok2, tempo, beat, offset = pcall(getinfo, "bpm")
    if ok2 and type(tempo) == "number" and tempo > 0 then
        return {
            { tempo = tempo, beat = beat or 4, start = 0, offset = offset or 0 },
        }
    end
    return nil
end

-- 同梱 script.obj2 の多角形と同じ順。画像なしの drawpoly は避ける。
function M.draw(obj, vertices, screen_w, screen_h)
    local w = M.even(screen_w)
    local h = M.even(screen_h)
    if w > M.MAX_BUFFER then w = M.even(M.MAX_BUFFER) end
    if h > M.MAX_BUFFER then h = M.even(M.MAX_BUFFER) end
    obj.load("figure", "四角形", 0xffffff, 2)
    obj.setoption("drawtarget", "tempbuffer", w, h)
    if vertices and #vertices > 0 then
        obj.drawpoly(vertices, 3)
    end
    obj.setoption("drawtarget", "framebuffer")
    local loaded = obj.load("tempbuffer")
    return loaded, w, h
end

function M.show_error(obj, message)
    pcall(function()
        obj.setfont("メイリオ", 28, 0, 0xffdddd, 0x111111)
        obj.load("text", tostring(message))
    end)
end

return M
