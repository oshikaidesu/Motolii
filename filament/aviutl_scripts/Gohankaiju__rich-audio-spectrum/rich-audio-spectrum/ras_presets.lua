-- ras_presets.lua : rich-audio-spectrum プリセット定義 (仕様書9章)
-- 単位規約: Hz / ms / dB / px / % を明示 (9.4節)。
-- cfg のフィールドは ras_engine.lua の各run_*が読むUI変数名と一致する。

local presets = {}
presets.VERSION = "0.3.0"

-- 各モードでcfgに必須のキー (SelfTestのスキーマ検査で使用)
presets.REQUIRED = {
    spectrum = { "width", "height", "bands", "freq_lo", "freq_hi", "mapping", "agg", "scale_mode",
        "channel", "gain_db", "autolevel", "attack_ms", "release_ms", "smoothness",
        "disp", "range_db", "bar_ratio", "thickness", "mirror",
        "trail_pct", "col1", "col2", "gradmode", "alpha_pct" },
    waveform = { "width", "height", "disp_pts", "disp", "thickness", "channel",
        "dccut", "gain_db", "autolevel", "smoothness", "col1", "col2", "alpha_pct" },
    meter = { "width", "height", "mode", "meas", "floor_db", "hold_ms",
        "col1", "col2", "alpha_pct" },
    radial = { "radius", "barlen", "thickness", "a0", "a1", "rot", "dir", "disp",
        "bands", "freq_lo", "freq_hi", "channel", "range_db", "gain_db", "autolevel",
        "attack_ms", "release_ms", "smoothness",
        "trail_pct", "col1", "col2", "gradmode", "alpha_pct" },
    scope = { "size", "disp", "points", "thickness", "gain_db", "autolevel",
        "fade_pct", "guide", "col1", "col2", "alpha_pct" },
}

presets.list = {
    {   -- 1: 上下ミラーバー (AE風)
        name = "AE Round Bars",
        mode = "spectrum",
        cfg = {
            width = 1400, height = 500, bands = 64,
            freq_lo = 40, freq_hi = 14000, mapping = 0, agg = 0, channel = 0,
            scale_mode = 0,
            gain_db = 0, autolevel = true, attack_ms = 20, release_ms = 180,
            smoothness = 25, disp = 0, range_db = 40, bar_ratio = 62,
            thickness = 4, mirror = true, trail_pct = 0,
            col1 = 0xffffff, col2 = 0x9fd8ff, gradmode = 0, alpha_pct = 100,
        },
    },
    {   -- 2: 白系の滑らかな横長スペクトラム (AE風)
        name = "AE Clean Spectrum",
        mode = "spectrum",
        cfg = {
            width = 1400, height = 320, bands = 192,
            freq_lo = 30, freq_hi = 16000, mapping = 0, agg = 0, channel = 0,
            scale_mode = 0,
            gain_db = 0, autolevel = true, attack_ms = 25, release_ms = 240,
            smoothness = 50, disp = 1, range_db = 42, bar_ratio = 70,
            thickness = 3, mirror = false, trail_pct = 0,
            col1 = 0xffffff, col2 = 0xdddddd, gradmode = 0, alpha_pct = 100,
        },
    },
    {   -- 3: 円形リング
        name = "Neon Ring",
        mode = "radial",
        cfg = {
            radius = 260, barlen = 190, thickness = 5,
            a0 = 0, a1 = 360, rot = 0, dir = 0, disp = 0,
            bands = 128, freq_lo = 40, freq_hi = 14000, channel = 0,
            range_db = 45, gain_db = 0, autolevel = true,
            attack_ms = 30, release_ms = 200, smoothness = 25,
            trail_pct = 0,
            col1 = 0x00d9ff, col2 = 0xff40ff, gradmode = 0, alpha_pct = 100,
        },
    },
    {   -- 4: Trap Nation風のリングライン+残像
        name = "Trap Halo",
        mode = "radial",
        cfg = {
            radius = 230, barlen = 260, thickness = 4,
            a0 = 0, a1 = 360, rot = 0, dir = 0, disp = 1,
            bands = 160, freq_lo = 30, freq_hi = 9000, channel = 0,
            range_db = 42, gain_db = 0, autolevel = true,
            attack_ms = 18, release_ms = 260, smoothness = 50,
            trail_pct = 35,
            col1 = 0x8a5cff, col2 = 0xffffff, gradmode = 1, alpha_pct = 100,
        },
    },
    {   -- 5: 読みやすい塗り波形
        name = "Podcast Wave",
        mode = "waveform",
        cfg = {
            width = 1500, height = 260, disp_pts = 220, disp = 1,
            thickness = 2, channel = 0, dccut = true,
            gain_db = 0, autolevel = true, smoothness = 25,
            col1 = 0xffffff, col2 = 0xbbbbbb, alpha_pct = 90,
        },
    },
    {   -- 6: 軽量な細線波形 (編集確認向け)
        name = "Editor Minimal",
        mode = "waveform",
        cfg = {
            width = 1200, height = 200, disp_pts = 300, disp = 0,
            thickness = 2, channel = 0, dccut = true,
            gain_db = 0, autolevel = false, smoothness = 0,
            col1 = 0xdddddd, col2 = 0xdddddd, alpha_pct = 100,
        },
    },
    {   -- 7: L/Rリサージュ
        name = "Stereo Scope",
        mode = "scope",
        cfg = {
            size = 520, disp = 0, points = 400, thickness = 1.5,
            gain_db = 0, autolevel = true, fade_pct = 45, guide = true,
            col1 = 0x66ff99, col2 = 0x00ccff, alpha_pct = 100,
        },
    },
    {   -- 8: 低域で脈動する両方向リング
        name = "Bass Pulse",
        mode = "radial",
        cfg = {
            radius = 200, barlen = 220, thickness = 10,
            a0 = 0, a1 = 360, rot = 0, dir = 2, disp = 0,
            bands = 48, freq_lo = 20, freq_hi = 250, channel = 0,
            range_db = 36, gain_db = 0, autolevel = true,
            attack_ms = 25, release_ms = 320, smoothness = 50,
            trail_pct = 35,
            col1 = 0xff5533, col2 = 0xffcc66, gradmode = 1, alpha_pct = 100,
        },
    },
    {   -- 9: 高速反応バー
        name = "EDM Bars",
        mode = "spectrum",
        cfg = {
            width = 1400, height = 520, bands = 48,
            freq_lo = 40, freq_hi = 12000, mapping = 0, agg = 0, channel = 0,
            scale_mode = 0,
            gain_db = 0, autolevel = true, attack_ms = 8, release_ms = 110,
            smoothness = 0, disp = 0, range_db = 38, bar_ratio = 78,
            thickness = 4, mirror = false, trail_pct = 0,
            col1 = 0xffd23f, col2 = 0xff4d6d, gradmode = 1, alpha_pct = 100,
        },
    },
    {   -- 10: VU風の遅い暖色メーター
        name = "Lo-Fi Meter",
        mode = "meter",
        cfg = {
            width = 70, height = 460, mode = 0, meas = 1,
            floor_db = -50, hold_ms = 1200,
            col1 = 0xff9944, col2 = 0xff5555, alpha_pct = 95,
        },
    },
    {   -- 11: 太いリボン+残像の映画的スペクトラム
        name = "Cinematic Ribbon",
        mode = "spectrum",
        cfg = {
            width = 1500, height = 360, bands = 96,
            freq_lo = 30, freq_hi = 8000, mapping = 0, agg = 0, channel = 0,
            scale_mode = 0,
            gain_db = 0, autolevel = true, attack_ms = 60, release_ms = 420,
            smoothness = 75, disp = 1, range_db = 40, bar_ratio = 70,
            thickness = 16, mirror = false, trail_pct = 70,
            col1 = 0x3a6ea5, col2 = 0xcfe8ff, gradmode = 0, alpha_pct = 100,
        },
    },
    {   -- 12: 小点列の軽量ミニマル
        name = "Minimal Dots",
        mode = "spectrum",
        cfg = {
            width = 1200, height = 300, bands = 96,
            freq_lo = 40, freq_hi = 14000, mapping = 0, agg = 0, channel = 0,
            scale_mode = 0,
            gain_db = 0, autolevel = true, attack_ms = 30, release_ms = 200,
            smoothness = 25, disp = 2, range_db = 40, bar_ratio = 70,
            thickness = 3, mirror = false, trail_pct = 0,
            col1 = 0xffffff, col2 = 0xffffff, gradmode = 0, alpha_pct = 100,
        },
    },
}

function presets.get(name)
    for _, p in ipairs(presets.list) do
        if p.name == name then return p end
    end
    return nil
end

-- スキーマ検査: 全プリセットが必須キーを持つか。
-- 戻り: ok(boolean), errors(配列)
function presets.validate()
    local errs = {}
    if #presets.list < 12 then
        errs[#errs + 1] = "preset count < 12"
    end
    for i, p in ipairs(presets.list) do
        if type(p.name) ~= "string" or p.name == "" then
            errs[#errs + 1] = "preset[" .. i .. "]: name missing"
        end
        local req = presets.REQUIRED[p.mode]
        if not req then
            errs[#errs + 1] = tostring(p.name) .. ": unknown mode " .. tostring(p.mode)
        else
            for _, key in ipairs(req) do
                if p.cfg[key] == nil then
                    errs[#errs + 1] = tostring(p.name) .. ": missing cfg." .. key
                end
            end
        end
    end
    return #errs == 0, errs
end

return presets
