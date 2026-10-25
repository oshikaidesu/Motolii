local P = {}

local ini = require("ini")

local function round(n)
    n = tonumber(n) or 0
    if n >= 0 then
        return math.floor(n + 0.5)
    end
    return math.ceil(n - 0.5)
end

local function get_ext(filepath)
    return tostring(filepath or ""):match("%.([^./\\]+)$") and
        tostring(filepath):match("%.([^./\\]+)$"):lower() or ""
end

local function change_ext(filepath, new_ext)
    local newfilepath = tostring(filepath or ""):gsub("%.[^./\\]+$", "")
    return newfilepath .. "." .. new_ext
end

local function package_key(filepath)
    return tostring(filepath or ""):gsub("%.[^./\\]+$", ""):lower()
end

local function is_supported_file(file)
    if type(file) ~= "table" or type(file.filepath) ~= "string" then
        return false
    end
    local ext = get_ext(file.filepath)
    return ext == "wav" or ext == "txt"
end

-- ハンドラー名（必須）
P.name = i18n({
    ja_JP = "テキスト、音声ファイルをVariable Font Textオブジェクトに変換",
    en_US = "Convert text and audio files to Variable Font Text objects",
    zh_CN = "将文本和音频文件转换为Variable Font Text对象",
})

-- 優先度（省略時は 1000）
-- 数値が小さいほど先に実行されます
P.priority = 1000

function P.drag_enter(files, state)
    for _, file in ipairs(files) do
        if is_supported_file(file) then
            return true
        end
    end
    return false
end

function P.drag_leave()
    -- ドラッグがタイムラインから離れたときの処理
end

local function files_package(files)
    local packages_by_key = {}
    local package_order = {}

    for _, file in ipairs(files) do
        if is_supported_file(file) then
            local key = package_key(file.filepath)
            local package_file = packages_by_key[key]
            if not package_file then
                package_file = {
                    audio_file = nil,
                    text_file = nil,
                }
                packages_by_key[key] = package_file
                package_order[#package_order + 1] = package_file
            end

            if get_ext(file.filepath) == "wav" then
                package_file.audio_file = file.filepath
            else
                package_file.text_file = file.filepath
            end
        end
    end

    for _, package_file in ipairs(package_order) do
        if package_file.audio_file and not package_file.text_file then
            local txt_path = change_ext(package_file.audio_file, "txt")
            local txt_file = io.open(txt_path, "rb")
            if txt_file then
                txt_file:close()
                package_file.text_file = txt_path
            end
        end
    end

    return package_order
end

local function text_from_file(file_path)
    local file = io.open(file_path, "rb")
    if not file then
        return ""
    end
    local content = file:read("*a")
    file:close()
    return content
end

local function parse_bool(v)
    if type(v) == "boolean" then
        return v
    end
    local s = tostring(v or ""):lower()
    return s == "1" or s == "true" or s == "on" or s == "yes"
end

local function is_conversion_enabled()
    local script_dir = gcmz.get_script_directory()
    local ini_path = script_dir .. "/VariableFont.ini"

    local exists = io.open(ini_path, "rb")
    if not exists then
        return true
    end
    exists:close()

    local ok, conf = pcall(ini.load, ini_path)
    if not ok or type(conf) ~= "table" then
        debug_print(i18n({
            ja_JP = "VariableFont.ini の読み込みに失敗しました",
            en_US = "Failed to load VariableFont.ini",
            zh_CN = "无法加载VariableFont.ini",
        }) .. ": " .. tostring(conf))
        return true
    end
    local value = conf:get("Switch", "Handle", nil)
    if value == nil then
        return true
    end

    return parse_bool(value)
end

function P.drop(files, state)
    if not is_conversion_enabled() then
        debug_print(i18n({
            ja_JP = "オブジェクト変換は無効です。",
            en_US = "Object conversion is disabled.",
            zh_CN = "对象转换已禁用。",
        }))
        return
    end
    if state and state.alt then
        return
    end

    local data = gcmz.get_project_data()
    if not data or not data.rate or not data.scale or data.scale == 0 then
        debug_print(i18n({
            ja_JP = "プロジェクト情報の取得に失敗しました。",
            en_US = "Failed to get project information.",
            zh_CN = "无法获取项目信息。",
        }))
        return
    end

    local packages = files_package(files)
    if #packages == 0 then
        return
    end

    local obj = ini.new()

    local totalframes = 0
    local obj_idx = 0
    local group_idx = 1

    for _, file in ipairs(packages) do
        local duration_sec = 1
        if file.audio_file then
            local info, err = gcmz.get_media_info(file.audio_file)
            if info and info.total_time then
                duration_sec = info.total_time
            elseif not info then
                debug_print(i18n({
                    ja_JP = "メディア情報の取得に失敗しました",
                    en_US = "Failed to get media information",
                    zh_CN = "无法获取媒体信息",
                }) .. ": " .. tostring(err))
            end
        end

        local length_frames = round(duration_sec * (data.rate / data.scale))
        if length_frames < 1 then
            length_frames = 1
        end
        local start_frame = totalframes
        local end_frame = totalframes + length_frames - 1

        if file.audio_file then
            -- 音声
            obj:set(tostring(obj_idx), "layer", "0")
            obj:set(tostring(obj_idx), "frame", tostring(start_frame) .. "," .. tostring(end_frame))
            obj:set(tostring(obj_idx), "group", tostring(group_idx))
            obj:set(tostring(obj_idx) .. ".0", "effect.name", "音声ファイル")
            obj:set(tostring(obj_idx) .. ".0", "ファイル", tostring(file.audio_file))
            obj:set(tostring(obj_idx) .. ".1", "effect.name", "音声再生")
            obj_idx = obj_idx + 1
        end

        if file.text_file then
            -- セリフ準備
            obj:set(tostring(obj_idx), "layer", "1")
            obj:set(tostring(obj_idx), "frame", tostring(start_frame) .. "," .. tostring(end_frame))
            obj:set(tostring(obj_idx), "group", tostring(group_idx))
            obj:set(tostring(obj_idx) .. ".0", "effect.name", "Variable Font Text")
            obj:set(tostring(obj_idx) .. ".0", "テキスト", tostring(text_from_file(file.text_file)))
            obj:set(tostring(obj_idx) .. ".1", "effect.name", "標準描画")
            obj_idx = obj_idx + 1
        end

        totalframes = end_frame + 1
        group_idx = group_idx + 1
    end

    if obj_idx == 0 then
        return
    end

    local temp_path, temp_err = gcmz.create_temp_file("variable-font.object")
    if not temp_path then
        debug_print(i18n({
            ja_JP = "一時ファイルの作成に失敗しました",
            en_US = "Failed to create a temporary file",
            zh_CN = "无法创建临时文件",
        }) .. ": " .. tostring(temp_err))
        return
    end

    local temp_file = io.open(temp_path, "wb")
    if not temp_file then
        debug_print(i18n({
            ja_JP = "一時ファイルを開けませんでした",
            en_US = "Failed to open the temporary file",
            zh_CN = "无法打开临时文件",
        }) .. ": " .. temp_path)
        return
    end

    local write_ok, write_err = temp_file:write(tostring(obj))
    temp_file:close()
    if not write_ok then
        debug_print(i18n({
            ja_JP = "一時ファイルの書き込みに失敗しました",
            en_US = "Failed to write the temporary file",
            zh_CN = "无法写入临时文件",
        }) .. ": " .. tostring(write_err))
        return
    end

    local replacement = {
        filepath = temp_path,
        mimetype = "application/aviutl-object",
        temporary = true,
    }
    local output_files = {}
    local replacement_added = false
    for _, file in ipairs(files) do
        if is_supported_file(file) then
            if not replacement_added then
                output_files[#output_files + 1] = replacement
                replacement_added = true
            end
        else
            output_files[#output_files + 1] = file
        end
    end

    for index = #files, 1, -1 do
        files[index] = nil
    end
    for index, file in ipairs(output_files) do
        files[index] = file
    end
end

return P
