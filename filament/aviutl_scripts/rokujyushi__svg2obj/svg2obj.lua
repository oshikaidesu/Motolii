local P = {}

local ini = require("ini")

-- ハンドラー名（必須）
P.name = i18n({
    ja_JP = [=[SVGファイルをカスタムオブジェクトに変換]=],
    en_US = [=[Convert SVG files to custom objects]=],
    zh_CN = [=[将SVG文件转换为自定义对象]=],
})

-- 優先度（省略時は 1000）
-- 数値が小さいほど先に実行されます
P.priority = 1000

-- 設定項目
P.settings = {
    -- 設定項目の例
    use_alt_key = false, -- Altキーを押下したときのみ有効にする
}
-- 設定項目

function P.drag_enter(files, state)
    -- ドラッグ開始時の処理
    for _, file in ipairs(files) do
        local ext = file.filepath:match("[^.]+$"):lower()
        if ext == "svg" then
            return true
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

    local obj = ini.new()

    for _, file in ipairs(files) do
        if file.filepath:match("[^.]+$"):lower() == "svg" then
            -- SVG
            obj:set("Object", "frame", "0,200")
            obj:set("Object.0", "effect.name", "SVG")
            obj:set("Object.0", "ファイル", tostring(file.filepath))
            obj:set("Object.1", "effect.name", "標準描画")

            local temp_path = gcmz.create_temp_file("svg2obj.object")
            local temp_file = io.open(temp_path, "wb")
            if not temp_file then
                debug_print(i18n({
                    ja_JP = [=[一時ファイルの作成に失敗しました: ]=],
                    en_US = [=[Failed to create a temporary file: ]=],
                    zh_CN = [=[临时文件创建失败: ]=],
                }) .. temp_path)
                return false
            end
            temp_file:write(tostring(obj))
            temp_file:close()
            file.filepath = temp_path
            file.mimetype = ""
            file.temporary = true
        end
    end
    return true
end

return P
