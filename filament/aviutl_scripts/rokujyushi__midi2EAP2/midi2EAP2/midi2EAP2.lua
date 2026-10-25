local P = {}

-- ハンドラー名（必須）
P.name = "midiファイルをEAP2オブジェクトに変換"

-- 優先度（省略時は 1000）
-- 数値が小さいほど先に実行されます
P.priority = 1000

P.exts = {
    "mid",
    "midi"
}

local ini = require("ini")

function P.drag_enter(files, state)
    -- ドラッグ開始時の処理
    for index, file in ipairs(files) do
        local ext = file.filepath:match("[^.]+$"):lower()
        for key, value in pairs(P.exts) do
            if ext == value then
                return true
            end
        end
    end
    return false
end

function P.drag_leave()
    -- ドラッグがタイムラインから離れたときの処理
end

local midi = require("midi")

-- LuaJIT は Lua5.1 互換のため bit32 が無い。bit ライブラリを使う。
local bit = bit or require("bit")

local function midi_duration(path)
    local tempos = { { tick = 0, bpm = 120 } } -- デフォルトBPM
    local trackTicks = {}
    local division
    local curTrack, curTick = 0, 0

    local f = assert(io.open(path, "rb"))
    midi.process(f, function(ev, a1, a2, a3)
        if ev == "header" then
            division = select(3, a1, a2, a3) -- division を取得
        elseif ev == "track" then
            curTrack, curTick = a1, 0
        elseif ev == "deltatime" then
            curTick = curTick + a1
        elseif ev == "setTempo" then
            if curTrack == 1 then -- テンポトラック前提
                table.insert(tempos, { tick = curTick, bpm = a1 })
            end
        elseif ev == "endOfTrack" then
            trackTicks[curTrack] = curTick
        end
    end)
    f:close()

    local maxTick = 0
    for _, t in pairs(trackTicks) do if t > maxTick then maxTick = t end end
    if not division then return nil, "division missing" end

    table.sort(tempos, function(x, y) return x.tick < y.tick end)

    local function ticks_to_seconds(maxTick, tempos, division)
        if division < 0 then
            -- SMPTE timing: division is a signed 16-bit value
            local div16 = bit.band(division, 0xFFFF)
            local fps = -bit.rshift(div16, 8)
            local ticksPerFrame = bit.band(div16, 0xFF)
            return maxTick / (fps * ticksPerFrame)
        end
        local tpq = division
        local sec = 0
        for i = 1, #tempos do
            local t0 = tempos[i].tick
            local t1 = tempos[i + 1] and tempos[i + 1].tick or maxTick
            if t1 > t0 then
                local usPerQ = 60e6 / tempos[i].bpm
                sec = sec + (t1 - t0) * usPerQ / (1e6 * tpq)
            end
        end
        return sec
    end

    return ticks_to_seconds(maxTick, tempos, division)
end

local function round(n)
    n = tonumber(n) or 0
    if n >= 0 then
        return math.floor(n + 0.5)
    end
    return math.ceil(n - 0.5)
end


function P.drop(files, state)
    local remaining_files = {}
    for index, file in ipairs(files) do
        local ext = file.filepath:match("[^.]+$"):lower()
        local is_midi = false
        for key, value in pairs(P.exts) do
            if ext == value then
                is_midi = true
                break
            end
        end
        if not is_midi then
            -- 対象外はスキップ
            remaining_files[#remaining_files + 1] = file
        else
            -- 応答停止を避けるため、1ファイルだけ処理してすぐ返す
            local data = gcmz.get_project_data()
            local total_time = midi_duration(file.filepath)
            local duration_sec = 1
            if total_time then
                duration_sec = total_time
            end
            local frame = round(duration_sec * (data.rate / data.scale))
            local obj = ini.new()
            obj:set("Object", "layer", "0")
            obj:set("Object", "frame", "0," .. tostring(frame))

            if not state.alt then
                obj:set("Object.0", "effect.name", "External Audio Processing 2 MIDI Visualizer")
                obj:set("Object.0", "MIDI File", file.filepath)
                obj:set("Object.1", "effect.name", "標準描画")
            else
                obj:set("Object.0", "effect.name", "External Audio Processing 2 (Media)")
                obj:set("Object.0", "MIDI File", file.filepath)
                obj:set("Object.1", "effect.name", "音声再生")
            end

            local temp_path = gcmz.create_temp_file("mid2obj_" .. index .. ".object")
            local temp_file = io.open(temp_path, "w")
            if temp_file then
                temp_file:write(tostring(obj))
                temp_file:close()
            else
                debug_print("一時ファイルの作成に失敗しました: " .. temp_path)
                return false
            end
            -- ファイルに何か処理を行う
            remaining_files[#remaining_files + 1] = {
                filepath = temp_path,
                mimetype = "",
                temporary = true
            }
        end
    end
    for i = 1, #remaining_files do
        files[i] = remaining_files[i]
    end
    return false
end

return P
