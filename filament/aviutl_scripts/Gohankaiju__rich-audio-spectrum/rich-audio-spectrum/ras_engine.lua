-- ras_engine.lua : rich-audio-spectrum 実行エンジン
-- 各モードの解析→DSP→描画パイプラインを cfg テーブル入力で実行する。
-- 専用オブジェクト(@rich-audio-spectrum_Spectrum等)と @rich-audio-spectrum_Main の両方から呼ばれる
-- 唯一の実装 (ロジック重複禁止、仕様書17.3節)。
-- cfg のフィールド名は各オブジェクトのUI変数名と一致させる。

local engine = {}
engine.VERSION = "0.3.0"

local core = require("ras_core")
local drawm = require("ras_draw")


-- スケール%適用対象 (Mainのスケール調整で乗算するキー)
engine.SCALE_KEYS = {
    spectrum = { "width", "height", "height_x", "thickness" },
    waveform = { "width", "height", "thickness" },
    meter    = { "width", "height" },
    radial   = { "radius", "barlen", "thickness" },
    scope    = { "size", "thickness" },
}

-- ============================================================
-- spectrum
-- ============================================================

function engine.run_spectrum(cfg)
    local q = core.quality(cfg.quality)
    -- scale_mode: 0=Linear(DTMer互換、既定) / 1=dB(従来・プリセット用)
    local dtmer = (cfg.scale_mode or 1) == 0
    -- DTMerモードのbandsは補間描画の分割数(コストは描画のみ)なので上限を緩める
    local nb = dtmer and math.min(cfg.bands, q.bands * 4) or math.min(cfg.bands, q.bands)
    local st = core.state(cfg.state_kind or "spec")
    core.check_sig(st, table.concat({ "spec", nb, cfg.freq_lo, cfg.freq_hi,
        cfg.mapping, cfg.agg, cfg.channel, cfg.disp, cfg.quality, cfg.scale_mode or 1 }, "/"))
    local chname = ({ [0] = "mix", [1] = "left", [2] = "right" })[cfg.channel] or "mix"
    local af = core.get_audio{ type = "fourier", channel = chname }
    if not af then
        core.error_text("音声が取得できません。")
        return
    end
    local dt = core.dt(st)
    local vals
    if dtmer then
        -- DTMer AudioSpectrum_01互換: リニア振幅・線形bin軸・相対ゲート・
        -- ガウシアン・エルミート補間。スケールは spectrum_scale=400000 相当。
        vals = core.build_bands_dtmer(af.values, af.n, af.rate, {
            freq_lo = cfg.freq_lo,
            freq_hi = cfg.freq_hi,
            bands = nb,
        })
        -- 値は16bit正規化(0..1)のまま扱う。DTMerの÷400000スケールは
        -- ジオメトリ側のKH(高さ係数)にのみ含める(二重適用しない)
        local amp = core.db_to_gain(cfg.gain_db or 0) * ((cfg.react_pct or 100) / 100)
        if cfg.autolevel then
            local pk = 0
            for i = 1, nb do
                local x = vals[i] * amp
                if x > pk then pk = x end
            end
            amp = amp * core.auto_level(pk, st, dt, { enabled = true, target = 0.6, max_gain = 6 })
        end
        for i = 1, nb do
            local v = vals[i] * amp
            if v < 0 then v = 0 end
            vals[i] = core.soft_clip(v)
        end
    else
        vals = core.build_bands(af.values, af.n, af.rate, {
            bands = nb,
            freq_lo = cfg.freq_lo,
            freq_hi = cfg.freq_hi,
            mapping = (cfg.mapping == 1) and "linear" or "log",
            agg = ({ [0] = "peak", [1] = "rms", [2] = "avg" })[cfg.agg] or "peak",
            min_db = -3 - (cfg.range_db or 45),
            max_db = -3,
            gain_db = cfg.gain_db,
        })
        local pk = 0
        for i = 1, nb do
            if vals[i] > pk then pk = vals[i] end
        end
        local amp = core.auto_level(pk, st, dt, { enabled = cfg.autolevel, target = 0.85, max_gain = 6 })
            * ((cfg.react_pct or 100) / 100)
        if amp ~= 1 then
            for i = 1, nb do vals[i] = core.soft_clip(vals[i] * amp) end
        end
    end
    st.env = st.env or {}
    core.envelope(vals, nb, st.env, dt, { attack_ms = cfg.attack_ms, release_ms = cfg.release_ms })
    -- 弾み%(バネ慣性) → 落下%(重力)
    core.spring(vals, nb, st, dt, { bounce_pct = cfg.bounce_pct, attack_ms = cfg.attack_ms })
    core.fall(vals, nb, st, dt, cfg.fall_pct)
    -- 滑らかさ%(0-100)をバンド配列へのガウシアンシグマにマッピング
    core.smooth_sigma(vals, nb, ((cfg.smoothness or 0) / 100) * nb / 12)
    core.commit(st)

    -- 高さ: 波形高さ倍率(DTMer互換: 表示値1.0 = 32767/400000×画面高×倍率) or 明示px
    local KH = 32767 / 400000 * (obj.screen_h or 1080) * (cfg.height_x or 4)
    -- DTMer互換モードのバッファ高は画面高で固定 (波形高さを変えても
    -- オブジェクト枠が変わらない。DTMerと同じ挙動。はみ出しは上下クリップ)
    local Hpx = cfg.height or (dtmer and (obj.screen_h or 1080) or math.floor(KH * 2))

    -- 個別オブジェクト(試験): バー/点を独立オブジェクトとして描画して終了
    if cfg.multi_obj and (cfg.disp == 0 or cfg.disp == 2) then
        local mn = math.min(nb, 256)   -- 負荷上限
        local mv = vals
        if mn < nb then mv = core.resample(vals, nb, mn, "catmull") end
        engine.render_spectrum_multi(cfg, mv, mn, dtmer, KH, Hpx)
        return
    end

    local geo = {
        w = cfg.width,
        h = Hpx,
        hpx = dtmer and KH or (Hpx / 2),
        mirror = cfg.mirror,
        col1 = drawm.rgb(cfg.col1),
        col2 = drawm.rgb(cfg.col2),
        grad_by_amp = (cfg.gradmode == 1),
        alpha = (cfg.alpha_pct or 100) / 100,
        bar_ratio = (cfg.bar_ratio or 70) / 100,
        thickness = cfg.thickness,
        bw = cfg.thickness,
    }
    local function build(arr, alpha_mul)
        local g2 = {}
        for k2, v2 in pairs(geo) do g2[k2] = v2 end
        g2.alpha = geo.alpha * (alpha_mul or 1)
        local vv
        if dtmer then
            -- 中央基線・上方向、ミラーで下側追加 (DTMerと同じ座標系)
            -- 振幅スケールはKH基準 (バッファ高=画面高とは独立)
            if cfg.disp == 1 then
                g2.mirror = true   -- line()の中央基線モード
                g2.h = KH * 2
                local pts = core.resample(arr, nb, math.min(nb * 2, 1024), "catmull")
                vv = drawm.line(pts, g2)
                if cfg.mirror then drawm.append_mirrored(vv) end
            elseif cfg.disp == 2 then
                g2.mirror = true
                g2.h = KH * 2
                g2.dot_size = cfg.thickness * 2
                vv = drawm.dots(arr, g2)
                if cfg.mirror then drawm.append_mirrored(vv) end
            else
                vv = drawm.bars_center(arr, g2)
            end
        else
            if cfg.disp == 1 then
                local pts = core.resample(arr, nb, math.min(nb * 4, 768), "catmull")
                vv = drawm.line(pts, g2)
                if cfg.mirror then drawm.append_mirrored(vv) end
            elseif cfg.disp == 2 then
                g2.dot_size = cfg.thickness * 2
                vv = drawm.dots(arr, g2)
                if cfg.mirror then drawm.append_mirrored(vv) end
            else
                vv = drawm.bars(arr, g2)
            end
        end
        return vv
    end
    local verts = engine.with_trail(st, vals, nb, cfg.trail_pct, q, build)
    -- 上下反転 (yを反転し、面の表裏を保つため頂点順も反転)
    if cfg.flip then
        for i = 1, #verts, 4 do
            local a, b, c, d = verts[i], verts[i + 1], verts[i + 2], verts[i + 3]
            a[2] = -a[2]; b[2] = -b[2]; c[2] = -c[2]; d[2] = -d[2]
            verts[i], verts[i + 1], verts[i + 2], verts[i + 3] = d, c, b, a
        end
    end
    local margin = 16 + math.min(cfg.thickness or 4, 64) * 2
    local ssaa = ((q.name == "High" or q.name == "Export") and cfg.disp ~= 0) and 2 or 1
    drawm.render_pipeline(verts, cfg.width + margin, Hpx + margin, {
        ssaa = ssaa,
    })
end

-- ============================================================
-- waveform
-- ============================================================

function engine.run_waveform(cfg)
    local st = core.state(cfg.state_kind or "wave")
    core.check_sig(st, table.concat({ "wave", cfg.disp_pts, cfg.channel, cfg.disp, cfg.quality }, "/"))
    local chname = ({ [0] = "mix", [1] = "left", [2] = "right", [3] = "diff" })[cfg.channel] or "mix"
    local af = core.get_audio{ type = "pcm", channel = chname, size = 4096 }
    if not af then
        core.error_text("音声が取得できません。")
        return
    end
    local dt = core.dt(st)
    local vals, n = af.values, af.n
    if cfg.dccut then core.dc_cut(vals, n) end
    local g = core.db_to_gain(cfg.gain_db)
    if cfg.autolevel then
        g = g * core.auto_level(af.peak, st, dt, { enabled = true, target = 0.7, max_gain = 8 })
    end
    for i = 1, n do
        vals[i] = core.soft_clip_signed(vals[i] * g)
    end
    core.commit(st)

    -- 前置ガウシアン: 間引き率連動の自動アンチエイリアス + 滑らかさ%(0-100)
    -- 生データに先にかけるのが滑らかさの核 (DTMer方式)
    local m = math.min(cfg.disp_pts, math.max(n, 2))
    local sigma_aa = (n / m) * 0.5
    local sigma_user = ((cfg.smoothness or 0) / 100) ^ 2 * 60
    core.smooth_sigma(vals, n, sigma_aa + sigma_user)
    local pts = core.resample(vals, n, m, "catmull")

    local geo = {
        w = cfg.width,
        h = cfg.height,
        signed = true,
        col1 = drawm.rgb(cfg.col1),
        col2 = drawm.rgb(cfg.col2),
        alpha = (cfg.alpha_pct or 100) / 100,
        thickness = cfg.thickness,
    }
    local verts
    if cfg.disp == 1 then
        verts = drawm.wave_fill(pts, geo)
    elseif cfg.disp == 2 then
        local gf = {
            w = cfg.width, h = cfg.height, signed = true,
            col1 = geo.col1, col2 = geo.col2,
            alpha = geo.alpha * 0.35, thickness = cfg.thickness,
        }
        verts = drawm.wave_fill(pts, gf)
        local lv = drawm.line(pts, geo)
        for i = 1, #lv do verts[#verts + 1] = lv[i] end
    else
        verts = drawm.line(pts, geo)
    end
    local margin = 16 + cfg.thickness * 2
    drawm.render_to_object(verts, cfg.width + margin, cfg.height + margin)
end

-- ============================================================
-- meter
-- ============================================================

function engine.run_meter(cfg)
    local st = core.state(cfg.state_kind or "meter")
    core.check_sig(st, table.concat({ "meter", cfg.mode, cfg.meas, cfg.floor_db }, "/"))
    local af = core.get_audio{ type = "pcm", channel = "dual", size = 4096 }
    if not af then
        core.error_text("音声が取得できません。")
        return
    end
    local dt = core.dt(st)

    local function measure(arr, n)
        if cfg.meas == 1 then
            local s = 0
            for i = 1, n do
                local x = arr[i] or 0
                s = s + x * x
            end
            return math.sqrt(s / math.max(n, 1))
        end
        local mx = 0
        for i = 1, n do
            local a = math.abs(arr[i] or 0)
            if a > mx then mx = a end
        end
        return mx
    end

    local vl, vr
    if af.left then
        vl = measure(af.left, af.n)
        vr = measure(af.right, af.n)
    else
        vl = measure(af.values, af.n)
        vr = vl
    end
    if cfg.mode == 1 then
        vl = (vl + vr) / 2
        vr = nil
    end

    local function to_disp(v)
        return core.clamp((core.db(v) - cfg.floor_db) / (0 - cfg.floor_db), 0, 1)
    end

    st.chL = st.chL or {}
    st.chR = st.chR or {}
    local dl = to_disp(vl)
    local hl = core.peak_hold(dl, st.chL, dt, cfg.hold_ms / 1000, 0.6)
    local dr, hr
    if vr then
        dr = to_disp(vr)
        hr = core.peak_hold(dr, st.chR, dt, cfg.hold_ms / 1000, 0.6)
    end
    core.commit(st)

    local gap = 6
    local nbar = vr and 2 or 1
    local W = cfg.width * nbar + gap * (nbar - 1)
    local H = cfg.height
    local c1 = drawm.rgb(cfg.col1)
    local c2 = drawm.rgb(cfg.col2)
    local a = (cfg.alpha_pct or 100) / 100
    local verts = {}

    local function bar(ix, dv, hv)
        local x0 = -W / 2 + (ix - 1) * (cfg.width + gap)
        local x1 = x0 + cfg.width
        -- 背景とホールド線もパレット由来にする(色上書きが全要素に効くように)
        drawm.push_quad(verts, x0, -H / 2, x1, H / 2,
            c1[1] * 0.14, c1[2] * 0.14, c1[3] * 0.14 + 0.02, a * 0.8)
        if dv > 0.002 then
            local rt, gt, bt = drawm.grad(c1, c2, dv)
            drawm.push_quad(verts, x0, H / 2 - dv * H, x1, H / 2,
                c1[1], c1[2], c1[3], a, rt, gt, bt, a)
        end
        if hv and hv > 0.01 then
            local y = H / 2 - hv * H
            drawm.push_quad(verts, x0, y - 1.5, x1, y + 1.5,
                (c2[1] + 1) / 2, (c2[2] + 1) / 2, (c2[3] + 1) / 2, a)
        end
        if dv >= 0.999 then
            drawm.push_quad(verts, x0, -H / 2, x1, -H / 2 + 8, 1, 0.15, 0.15, a)
        end
    end

    bar(1, dl, hl)
    if vr then bar(2, dr, hr) end
    drawm.render_to_object(verts, W + 12, H + 12)
end

-- ============================================================
-- radial
-- ============================================================

function engine.run_radial(cfg)
    local q = core.quality(cfg.quality)
    local nb = math.min(cfg.bands, q.bands)
    local st = core.state(cfg.state_kind or "radial")
    core.check_sig(st, table.concat({ "radial", nb, cfg.freq_lo, cfg.freq_hi,
        cfg.channel, cfg.disp, cfg.dir, cfg.quality }, "/"))
    local chname = ({ [0] = "mix", [1] = "left", [2] = "right" })[cfg.channel] or "mix"
    local af = core.get_audio{ type = "fourier", channel = chname }
    if not af then
        core.error_text("音声が取得できません。")
        return
    end
    local dt = core.dt(st)
    local vals = core.build_bands(af.values, af.n, af.rate, {
        bands = nb,
        freq_lo = cfg.freq_lo,
        freq_hi = cfg.freq_hi,
        mapping = "log",
        agg = "peak",
        min_db = -3 - cfg.range_db,
        max_db = -3,
        gain_db = cfg.gain_db,
    })
    local pk = 0
    for i = 1, nb do
        if vals[i] > pk then pk = vals[i] end
    end
    -- 自動音量 × 反応の深さ (react_pct: 表示振幅の倍率)
    local amp = core.auto_level(pk, st, dt, { enabled = cfg.autolevel, target = 0.85, max_gain = 6 })
        * ((cfg.react_pct or 100) / 100)
    if amp ~= 1 then
        for i = 1, nb do vals[i] = core.soft_clip(vals[i] * amp) end
    end
    st.env = st.env or {}
    core.envelope(vals, nb, st.env, dt, { attack_ms = cfg.attack_ms, release_ms = cfg.release_ms })
    -- 弾み%(バネ慣性) → 落下%(重力)
    core.spring(vals, nb, st, dt, { bounce_pct = cfg.bounce_pct, attack_ms = cfg.attack_ms })
    core.fall(vals, nb, st, dt, cfg.fall_pct)
    -- 滑らかさ%(0-100)をバンド配列へのガウシアンシグマにマッピング
    core.smooth_sigma(vals, nb, ((cfg.smoothness or 0) / 100) * nb / 12)
    core.commit(st)

    -- 個別オブジェクト(試験): バー/点を独立オブジェクトとして描画して終了
    if cfg.multi_obj and (cfg.disp == 0 or cfg.disp == 2) then
        local mn = math.min(nb, 256)   -- 負荷上限 (ミラーでも総本数=分割数)
        local mv = vals
        if mn < nb then mv = core.resample(vals, nb, mn, "catmull") end
        engine.render_radial_multi(cfg, mv, mn)
        return
    end

    local geo = {
        radius = cfg.radius,
        barlen = cfg.barlen,
        thickness = cfg.thickness,
        a0_deg = cfg.a0,
        a1_deg = cfg.a1,
        rot_deg = cfg.rot,
        dir = cfg.dir,
        inner_min = math.max(cfg.thickness, 2),
        col1 = drawm.rgb(cfg.col1),
        col2 = drawm.rgb(cfg.col2),
        grad_by_amp = (cfg.gradmode == 1),
        alpha = (cfg.alpha_pct or 100) / 100,
    }
    local function build(arr, alpha_mul)
        local g2 = {}
        for k2, v2 in pairs(geo) do g2[k2] = v2 end
        g2.alpha = geo.alpha * (alpha_mul or 1)
        local a2, n2 = arr, nb
        if cfg.mirror then
            -- EDM風ミラー: 半周に波形、残り半周に左右反転を配置。
            -- 分割数=最終描画本数を維持するため、スペクトラムを半分に畳んでから
            -- 対称配置する(バー1本の角度幅が非ミラー時と同じ=塊感を保つ)
            local half = math.max(math.floor(nb / 2), 4)
            local src = core.resample(arr, nb, half, "catmull")
            a2 = {}
            for i = 1, half do a2[i] = src[i] end
            for i = 1, half do a2[half + i] = src[half + 1 - i] end
            n2 = half * 2
            g2.grad_mirror = true
        end
        if cfg.disp == 1 then
            local pts = core.resample(a2, n2, math.min(n2 * 3, 1024), "catmull")
            return drawm.ring_line(pts, g2)
        elseif cfg.disp == 2 then
            g2.dot_size = cfg.thickness * 2
            return drawm.ring_dots(a2, g2)
        end
        return drawm.ring_bars(a2, g2)
    end
    local verts = engine.with_trail(st, vals, nb, cfg.trail_pct, q, build)
    local D = 2 * (cfg.radius + cfg.barlen + cfg.thickness) + 24
    local ssaa = (q.name == "High" or q.name == "Export") and 2 or 1
    drawm.render_pipeline(verts, D, D, {
        ssaa = ssaa,
    })
end

-- ============================================================
-- scope (stereo)
-- ============================================================

function engine.run_scope(cfg)
    local q = core.quality(cfg.quality)
    local st = core.state(cfg.state_kind or "scope")
    core.check_sig(st, table.concat({ "scope", cfg.disp, cfg.points, cfg.quality }, "/"))
    local af = core.get_audio{ type = "pcm", channel = "dual", size = 4096 }
    if not af then
        core.error_text("音声が取得できません。")
        return
    end
    local dt = core.dt(st)
    local g = core.db_to_gain(cfg.gain_db)
    if cfg.autolevel then
        g = g * core.auto_level(af.peak, st, dt, { enabled = true, target = 0.65, max_gain = 8 })
    end
    core.commit(st)
    local L, R = {}, {}
    for i = 1, af.n do
        L[i] = core.soft_clip_signed((af.left and af.left[i] or af.values[i] or 0) * g)
        R[i] = core.soft_clip_signed((af.right and af.right[i] or af.values[i] or 0) * g)
    end
    local half = cfg.size / 2
    local xs, ys, m = drawm.lissajous_points(L, R, af.n, cfg.points, half * 0.95)
    local geo = {
        col1 = drawm.rgb(cfg.col1),
        col2 = drawm.rgb(cfg.col2),
        alpha = (cfg.alpha_pct or 100) / 100,
        thickness = cfg.thickness,
        dot_size = cfg.thickness * 2.5,
    }
    local verts = {}
    if cfg.guide then
        local ga = geo.alpha * 0.18
        local c = geo.col1
        drawm.push_quad(verts, -half, -half, half, -half + 1, c[1], c[2], c[3], ga)
        drawm.push_quad(verts, -half, half - 1, half, half, c[1], c[2], c[3], ga)
        drawm.push_quad(verts, -half, -half, -half + 1, half, c[1], c[2], c[3], ga)
        drawm.push_quad(verts, half - 1, -half, half, half, c[1], c[2], c[3], ga)
        drawm.push_quad(verts, -0.5, -half, 0.5, half, c[1], c[2], c[3], ga)
        drawm.push_quad(verts, -half, -0.5, half, 0.5, c[1], c[2], c[3], ga)
    end
    if cfg.disp == 2 then
        st.history = st.history or {}
        local layers = math.min(q.history, 6)
        for k = math.min(layers, #st.history), 1, -1 do
            local h = st.history[k]
            if h then
                local g2 = {
                    col1 = geo.col1, col2 = geo.col2,
                    alpha = geo.alpha * ((cfg.fade_pct / 100) ^ k),
                    dot_size = geo.dot_size,
                }
                local vv = drawm.xy_dots(h.xs, h.ys, h.m, g2)
                for i = 1, #vv do verts[#verts + 1] = vv[i] end
            end
        end
        table.insert(st.history, 1, { xs = xs, ys = ys, m = m })
        while #st.history > layers do table.remove(st.history) end
    else
        st.history = nil
    end
    local mv
    if cfg.disp == 0 then
        mv = drawm.polyline(xs, ys, m, geo)
    else
        mv = drawm.xy_dots(xs, ys, m, geo)
    end
    for i = 1, #mv do verts[#verts + 1] = mv[i] end

    local margin = 16
    local ssaa = (q.name == "High" or q.name == "Export") and 2 or 1
    drawm.render_pipeline(verts, cfg.size + margin, cfg.size + margin, {
        ssaa = ssaa,
    })
end

-- ============================================================
-- 共通: 残像 (数値履歴の再描画方式、仕様書8.6節)
-- ============================================================

-- st.historyへ値配列を蓄積し、古い層を減衰αで重ねた頂点列を返す。
-- trail_pct: 0=OFF、1-100で残像の強さを連続制御 (層数と減衰の両方に効く)。
-- builder(arr, alpha_mul) -> verts
function engine.with_trail(st, vals, nb, trail_pct, q, builder)
    local verts = {}
    local tp = math.max(trail_pct or 0, 0)
    local layers = 0
    if tp > 0 then
        layers = math.max(1, math.floor(q.history * tp / 100 + 0.5))
    end
    local fade = 0.30 + 0.55 * tp / 100
    if layers > 0 then
        st.history = st.history or {}
        for k = math.min(layers, #st.history), 1, -1 do
            local old = st.history[k]
            if old then
                local vv = builder(old, 0.85 * (fade ^ k))
                for i = 1, #vv do verts[#verts + 1] = vv[i] end
            end
        end
        local cp = {}
        for i = 1, nb do cp[i] = vals[i] end
        table.insert(st.history, 1, cp)
        while #st.history > layers do table.remove(st.history) end
    else
        st.history = nil
    end
    local mv = builder(vals, 1)
    for i = 1, #mv do verts[#verts + 1] = mv[i] end
    return verts
end

-- ============================================================
-- 個別オブジェクト描画 (試験実装)
-- 各バー/点を obj.multiobject() で独立オブジェクトとして描画し、
-- 後段の個別オブジェクト対応アニメーション効果をバー単位に適用できるようにする。
-- このモードでは内蔵の残像/SSAAは適用されない(装飾は後段効果に委ねる)。
-- ============================================================

local function grad_color(c1, c2, t)
    t = core.clamp(t, 0, 1)
    local r = c1[1] + (c2[1] - c1[1]) * t
    local g = c1[2] + (c2[2] - c1[2]) * t
    local b = c1[3] + (c2[3] - c1[3]) * t
    return RGB(math.floor(r * 255 + 0.5), math.floor(g * 255 + 0.5), math.floor(b * 255 + 0.5))
end

function engine.render_spectrum_multi(cfg, vals, nb, dtmer, KH, Hpx)
    local slot = cfg.width / nb
    local bw = math.max(1, math.floor(math.min(cfg.thickness or 5, slot + 1) + 0.5))
    local c1 = drawm.rgb(cfg.col1)
    local c2 = drawm.rgb(cfg.col2)
    local alpha = (cfg.alpha_pct or 100) / 100
    local dots = (cfg.disp == 2)
    obj.multiobject(nb, function()
        local i = obj.index + 1
        local val = core.clamp(vals[i] or 0, 0, 1)
        if val <= 0.003 then
            obj.load("figure", "四角形", 0x000000, 1)
            obj.alpha = 0
            return
        end
        local t = (cfg.gradmode == 1) and val or ((i - 1) / math.max(nb - 1, 1))
        local col = grad_color(c1, c2, t)
        local cx = -cfg.width / 2 + (i - 0.5) * slot
        if dots then
            local dsize = math.max(math.floor((cfg.thickness or 5) * 2 + 0.5), 2)
            local ty
            if dtmer then
                ty = -val * KH
            elseif cfg.mirror then
                ty = -val * Hpx / 2
            else
                ty = Hpx / 2 - val * Hpx
            end
            if cfg.flip then ty = -ty end
            obj.load("figure", "四角形", col, dsize)
            obj.ox = cx
            obj.oy = ty
            obj.alpha = alpha
        else
            local h, cy
            if dtmer then
                h = math.max(val * KH, 1)
                if cfg.mirror then
                    cy = 0
                    h = h * 2
                else
                    cy = -h / 2
                end
            else
                h = math.max(val * Hpx, 1)
                cy = cfg.mirror and 0 or (Hpx / 2 - h / 2)
            end
            if cfg.flip then cy = -cy end
            -- clamp: 拡大時に端が透明と補間されて先端が薄くなるのを防ぐ
            obj.setoption("sampler", "clamp")
            obj.load("figure", "四角形", col, bw)
            -- 一括描画と同じ整数スナップ(バー端をピクセル境界に揃える)
            obj.ox = math.floor(cx - bw / 2 + 0.5) + bw / 2
            obj.oy = cy
            obj.sy = h / bw
            obj.alpha = alpha
        end
    end)
end

function engine.render_radial_multi(cfg, vals, nb)
    local a2, n2 = vals, nb
    if cfg.mirror then
        -- 分割数=最終描画本数を維持 (半分に畳んで対称配置)
        local half = math.max(math.floor(nb / 2), 4)
        local src = core.resample(vals, nb, half, "catmull")
        a2 = {}
        for i = 1, half do a2[i] = src[i] end
        for i = 1, half do a2[half + i] = src[half + 1 - i] end
        n2 = half * 2
    end
    local c1 = drawm.rgb(cfg.col1)
    local c2 = drawm.rgb(cfg.col2)
    local alpha = (cfg.alpha_pct or 100) / 100
    local bw = math.max(1, math.floor((cfg.thickness or 6) + 0.5))
    local a0d, a1d = cfg.a0 or 0, cfg.a1 or 360
    local dots = (cfg.disp == 2)
    obj.multiobject(n2, function()
        local i = obj.index + 1
        local val = core.clamp(a2[i] or 0, 0, 1)
        if val <= 0.003 then
            obj.load("figure", "四角形", 0x000000, 1)
            obj.alpha = 0
            return
        end
        local t
        if cfg.gradmode == 1 then
            t = val
        else
            t = (i - 1) / math.max(n2 - 1, 1)
            if cfg.mirror then t = 1 - math.abs(2 * t - 1) end
        end
        local col = grad_color(c1, c2, t)
        local deg = a0d + (a1d - a0d) * (i - 0.5) / n2 + (cfg.rot or 0)
        local ang = math.rad(deg - 90)
        local len = math.max(val * cfg.barlen, 1)
        if dots then
            local dsize = math.max(bw * 2, 2)
            local rtip = (cfg.dir == 1) and math.max(cfg.radius - len, 2) or (cfg.radius + len)
            obj.load("figure", "四角形", col, dsize)
            obj.ox = math.cos(ang) * rtip
            obj.oy = math.sin(ang) * rtip
            obj.rz = deg
            obj.alpha = alpha
        else
            local rc
            if cfg.dir == 1 then
                rc = math.max(cfg.radius - len / 2, len / 2 + 2)
            elseif cfg.dir == 2 then
                rc = cfg.radius
            else
                rc = cfg.radius + len / 2
            end
            -- clamp: 拡大時に端が透明と補間されて先端が薄くなるのを防ぐ
            obj.setoption("sampler", "clamp")
            obj.load("figure", "四角形", col, bw)
            obj.ox = math.cos(ang) * rc
            obj.oy = math.sin(ang) * rc
            obj.sy = len / bw
            obj.rz = deg
            obj.alpha = alpha
        end
    end)
end

-- ============================================================
-- dispatcher
-- ============================================================

local RUNNERS = {
    spectrum = function(cfg) engine.run_spectrum(cfg) end,
    waveform = function(cfg) engine.run_waveform(cfg) end,
    meter    = function(cfg) engine.run_meter(cfg) end,
    radial   = function(cfg) engine.run_radial(cfg) end,
    scope    = function(cfg) engine.run_scope(cfg) end,
}

function engine.run(mode, cfg)
    local fn = RUNNERS[mode]
    if not fn then
        core.error_text("プリセット定義が不正です。(mode=" .. tostring(mode) .. ")")
        return
    end
    fn(cfg)
end

return engine
