local M = {}

local function floor_div(a, b)
    return math.floor(a / b)
end

-- scene_time 秒。list は {tempo, beat, start, offset} の配列。空なら opt.tempo。
-- 時刻だけで拍位置が決まる。
function M.query(scene_time, list, opt)
    opt = opt or {}
    scene_time = tonumber(scene_time) or 0
    local tempo, beat, start, offset, source
    if list and #list > 0 then
        local seg = list[1]
        for i = 1, #list do
            local item = list[i]
            if (tonumber(item.start) or 0) <= scene_time then
                seg = item
            else
                break
            end
        end
        tempo = tonumber(seg.tempo) or opt.tempo or 120
        beat = tonumber(seg.beat) or opt.beat or 4
        start = tonumber(seg.start) or 0
        offset = tonumber(seg.offset) or 0
        source = "list"
    else
        tempo = tonumber(opt.tempo) or 120
        beat = tonumber(opt.beat) or 4
        start = 0
        offset = 0
        source = "manual"
    end
    if tempo < 1 then tempo = 1 end
    beat = math.floor(beat)
    if beat < 1 then beat = 1 end
    local subdiv = math.floor(tonumber(opt.subdivision) or 1)
    if subdiv < 1 then subdiv = 1 end
    local user_offset = tonumber(opt.offset) or 0

    local period = 60 / tempo
    local beat_pos = (scene_time - start - offset + user_offset) / period
    local beat_index = math.floor(beat_pos)
    local beat_phase = beat_pos - beat_index
    local bar_index = floor_div(beat_index, beat)
    local beat_in_bar = beat_index - bar_index * beat
    local sub_pos = beat_phase * subdiv
    local sub_index = math.floor(sub_pos)
    local sub_phase = sub_pos - sub_index

    return {
        tempo = tempo,
        beat = beat,
        start = start,
        offset = offset,
        source = source,
        period = period,
        beat_index = beat_index,
        beat_phase = beat_phase,
        beat_in_bar = beat_in_bar,
        bar_index = bar_index,
        bar_phase = (beat_in_bar + beat_phase) / beat,
        is_downbeat = beat_in_bar == 0,
        subdivision = subdiv,
        sub_index = sub_index,
        sub_phase = sub_phase,
        sub_period = period / subdiv,
    }
end

return M
