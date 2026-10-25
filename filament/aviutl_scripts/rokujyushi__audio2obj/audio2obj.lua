local P = {}

local ini = require("ini")

local function round(n)
    n = tonumber(n) or 0
    if n >= 0 then
        return math.floor(n + 0.5)
    end
    return math.ceil(n - 0.5)
end

local function basename_without_ext(path)
    local name = tostring(path or ""):match("([^/\\]+)$") or ""
    return (name:gsub("%.[^%.]+$", ""))
end

local function is_audio_file(filepath)
    local ext = tostring(filepath or ""):match("[^.]+$")
    ext = ext and ext:lower() or ""
    for _, value in ipairs(P.audio_exts) do
        if ext == value then
            return true
        end
    end
    return false
end

-- ハンドラー名（必須）
P.name = "AudioFileをオブジェクトに変換"

-- 優先度（省略時は 1000）
-- 数値が小さいほど先に実行されます
P.priority = 1000

-- 設定項目
P.settings = {
    -- 設定項目の例
    use_alt_key = true, -- Altキーを押下したときのみ有効にする
    set_character_id = {
        switch = false,
        splitstr = "_" -- ファイル名からキャラクターIDを設定する際の区切り文字
    },                 -- キャラクターIDをファイル名に設定する
}
-- 設定項目

P.audio_exts = {
    "m4a",
    "wav",
    "aiff",
    "flac",
    "wv",
    "alac",
    "aac",
    "mp3",
    "ogg",
    "opus",
    "ac3",
    "wma"
}

function P.drag_enter(files, state)
    -- ドラッグ開始時の処理
    for index, file in ipairs(files) do
        local ext = file.filepath:match("[^.]+$"):lower()
        for key, value in pairs(P.audio_exts) do
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

function P.drop(files, state)
    if P.settings.use_alt_key and not state.alt then
        return false
    end

    local data = gcmz.get_project_data()
    local obj = ini.new()

    local totalframes = 0
    local obj_idx = 0
    local group_idx = 1
    local converted_any = false
    local remaining_files = {}

    for _, file in ipairs(files) do
        if is_audio_file(file.filepath) then
            converted_any = true

            local info = gcmz.get_media_info(file.filepath)
            local duration_sec = 1
            if info and info.total_time then
                duration_sec = info.total_time
            end

            local length_frames = round(duration_sec * (data.rate / data.scale))
            if length_frames < 1 then
                length_frames = 1
            end
            local start_frame = totalframes
            local end_frame = totalframes + length_frames - 1

            local text = basename_without_ext(file.filepath)
            if P.settings.set_character_id.switch then
                local parts = {}
                for part in string.gmatch(text, "([^" .. P.settings.set_character_id.splitstr .. "]+)") do
                    parts[#parts + 1] = part
                end
                if #parts > 1 then
                    text = table.concat(parts, " ", 2)
                end
            end

            -- 音声
            obj:set(tostring(obj_idx), "layer", "0")
            obj:set(tostring(obj_idx), "frame", tostring(start_frame) .. "," .. tostring(end_frame))
            obj:set(tostring(obj_idx), "group", tostring(group_idx))
            obj:set(tostring(obj_idx) .. ".0", "effect.name", "音声ファイル")
            obj:set(tostring(obj_idx) .. ".0", "ファイル", tostring(file.filepath))
            obj:set(tostring(obj_idx) .. ".1", "effect.name", "音声再生")
            obj_idx = obj_idx + 1

            -- セリフ準備
            obj:set(tostring(obj_idx), "layer", "1")
            obj:set(tostring(obj_idx), "frame", tostring(start_frame) .. "," .. tostring(end_frame))
            obj:set(tostring(obj_idx), "group", tostring(group_idx))
            obj:set(tostring(obj_idx) .. ".0", "effect.name", "セリフ準備@PSDToolKit")
            obj:set(tostring(obj_idx) .. ".0", "キャラクターID", "file_name_0")
            obj:set(tostring(obj_idx) .. ".0", "テキスト", tostring(text))
            obj:set(tostring(obj_idx) .. ".0", "音声ファイル", tostring(file.filepath))
            obj:set(tostring(obj_idx) .. ".1", "effect.name", "標準描画")
            obj_idx = obj_idx + 1

            totalframes = end_frame + 1
            group_idx = group_idx + 1
        else
            remaining_files[#remaining_files + 1] = file
        end
    end

    if not converted_any then
        return false
    end
    local temp_path = gcmz.create_temp_file("wav2obj.object")
    local temp_file = io.open(temp_path, "wb")
    if not temp_file then
        debug_print("一時ファイルの作成に失敗しました: " .. temp_path)
        return false
    end
    temp_file:write(tostring(obj))
    temp_file:close()

    -- 音声ファイルを除外し、生成 object を追加
    remaining_files[#remaining_files + 1] = {
        filepath = temp_path,
        mimetype = "",
        temporary = true
    }

    -- files を置き換え（音声ファイルは削除済み）
    for i = 1, #remaining_files do
        files[i] = remaining_files[i]
    end
    for i = #remaining_files + 1, #files do
        files[i] = nil
    end

    return true
end

return P
