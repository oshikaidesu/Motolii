local P = {}

-- ハンドラー名（必須）
P.name = "File Rename '-'to'_'"

-- 優先度（省略時は 1000）
-- 数値が小さいほど先に実行されます
P.priority = 1000

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

local function debug_print_(message)
    debug_print(P.name .. ": " .. message)
end

local function exists(path)
    local file = io.open(path, "r")
    if file then
        file:close()
        return true
    end
    return false
end

local function basename_without_ext(path)
    local name = tostring(path or ""):match("([^/\\]+)$") or ""
    return (name:gsub("%.[^%.]+$", ""))
end

local function replace_extension(path, ext)
    local base = tostring(path or ""):match("(.+)%.[^%.]+$")
    if not base then
        return nil
    end
    return base .. ext
end

local function dirname(path)
    return tostring(path or ""):match("^(.*[/\\])") or ""
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

local function find_matching_text_file(files, path)
    local base_name = basename_without_ext(path)

    for _, file in ipairs(files) do
        if not is_audio_file(file.filepath) and basename_without_ext(file.filepath) == base_name then
            debug_print_(base_name .. " matches " .. base_name)
            return file.filepath, file
        end
    end
    return nil
end

local function search_txt(files, path)
    return find_matching_text_file(files, path) ~= nil
end

local function get_txt_file_name(files, path)
    return find_matching_text_file(files, path)
end

local function should_rename_audio_file(path)
    return basename_without_ext(path):find("-", 1, true) ~= nil
end

local function build_renamed_paths(original_path)
    local file_dir = dirname(original_path)
    local renamed_audio_base = original_path:match("([^/\\]+)$"):gsub("-", "_")
    local audio_file_name = file_dir .. renamed_audio_base
    local text_file_name = replace_extension(audio_file_name, ".txt")

    return audio_file_name, text_file_name
end

local function rename_text_file(txt_path, txt_file, text_file_name)
    if not txt_path then
        return
    end

    debug_print_("Renaming text file to: " .. txt_path .. " -> " .. text_file_name)
    local text_ok, text_err = os.rename(txt_path, text_file_name)
    if not text_ok then
        debug_print_("Failed to rename text file: " .. tostring(text_err))
    elseif txt_file then
        txt_file.filepath = text_file_name
    end
end

function P.drag_enter(files, state)
    -- ドラッグ開始時の処理
    for index, file in ipairs(files) do
        if is_audio_file(file.filepath) and search_txt(files, file.filepath) then
            return true
        end
    end
    return false
end

function P.drag_leave()
    -- ドラッグがタイムラインから離れたときの処理
end

function P.drop(files, state)
    for _, file in ipairs(files) do
        if is_audio_file(file.filepath) and search_txt(files, file.filepath) then
            debug_print_("Processing file: " .. file.filepath)

            if should_rename_audio_file(file.filepath) then
                -- ファイルリネーム
                -- 例: "01-TrackName.wav" -> "01_TrackName.wav"
                local original_path = file.filepath
                local txt_path, txt_file = get_txt_file_name(files, original_path)
                local audio_file_name, text_file_name = build_renamed_paths(original_path)

                if not text_file_name then
                    debug_print_("Skipping rename because the audio file has no extension: " .. tostring(original_path))
                else
                    debug_print_("Renaming audio file to: " .. file.filepath .. " -> " .. audio_file_name)
                    local audio_ok, audio_err = os.rename(original_path, audio_file_name)
                    if not audio_ok then
                        debug_print_("Failed to rename audio file: " .. tostring(audio_err))
                    else
                        file.filepath = audio_file_name
                        rename_text_file(txt_path, txt_file, text_file_name)
                    end
                end
            end
        end
    end

    return true
end

return P
