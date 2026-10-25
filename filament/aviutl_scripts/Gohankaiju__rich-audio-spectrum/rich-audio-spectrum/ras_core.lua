-- ras_core.lua : rich-audio-spectrum 中核 (utils / state / audio / dsp)
-- 仕様書 aviutl2-rich-audio-waveform-spec.md v4.1 に準拠。
-- dsp系関数は obj 非依存のテーブル入出力(SelfTestから直接呼べる)。
-- audio/state/quality/error は obj API を利用する。

local core = {}
core.VERSION = "0.1.0"

local floor, ceil, abs, exp, log, max, min, sqrt =
    math.floor, math.ceil, math.abs, math.exp, math.log, math.max, math.min, math.sqrt

-- ============================================================
-- utils (obj非依存)
-- ============================================================

function core.clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

function core.lerp(a, b, t)
    return a + (b - a) * t
end

-- Catmull-Rom補間 (obj.interpolationは実測でCR不一致のため自前実装を使う)
function core.catmull(t, p0, p1, p2, p3)
    return 0.5 * ((2 * p1) + (-p0 + p2) * t
        + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t * t
        + (-p0 + 3 * p1 - 3 * p2 + p3) * t * t * t)
end

function core.safe_num(v, def)
    if type(v) == "number" and v == v and v ~= math.huge and v ~= -math.huge then
        return v
    end
    return def or 0
end

-- dBFS変換 (ref=1.0の線形値を受ける)
function core.db(v)
    return 20 * math.log10(max(abs(v), 1e-9))
end

function core.db_to_display(db, min_db, max_db)
    return core.clamp((db - min_db) / (max_db - min_db), 0, 1)
end

function core.db_to_gain(db)
    return 10 ^ (db / 20)
end

-- ソフトクリップ (0..1系、0.85まで素通し)
function core.soft_clip(x)
    if x <= 0.85 then return x end
    return 0.85 + 0.15 * math.tanh((x - 0.85) / 0.15)
end

function core.soft_clip_signed(x)
    if x < 0 then return -core.soft_clip(-x) end
    return core.soft_clip(x)
end

-- ============================================================
-- state (仕様書10章)
-- ============================================================

-- 種別ごと・オブジェクトごとの状態スライスを返す。
-- フレーム戻り/大ジャンプで動的状態をリセットする。
function core.state(kind)
    _G.RAS_STATE = _G.RAS_STATE or {}
    local root = _G.RAS_STATE
    local key = kind .. ":" .. tostring(obj.effect_id or obj.id or obj.layer or 0)
    local st = root[key]
    if not st then
        st = { sig = "", last_frame = -1, last_time = -1 }
        root[key] = st
    end
    local f = obj.frame
    if st.last_frame >= 0 and (f < st.last_frame or f > st.last_frame + 30) then
        core.reset_dynamic(st)
    end
    return st
end

function core.reset_dynamic(st)
    st.env = nil
    st.history = nil
    st.level = nil
    st.gain = nil
    st.ph = nil
    st.pht = nil
    st.clip_t = nil
    st.pos = nil      -- 弾み% (バネ系)
    st.vel = nil
    st.fpos = nil     -- 落下%
    st.fvel = nil
    st.last_time = -1
end

-- 設定署名が変わったら動的状態をリセット
function core.check_sig(st, sig)
    if st.sig ~= sig then
        st.sig = sig
        core.reset_dynamic(st)
    end
end

-- 前フレームからの経過秒。初回/巻き戻し後は1フレーム分で代用。
function core.dt(st)
    if st.last_time and st.last_time >= 0 and obj.time >= st.last_time then
        local d = obj.time - st.last_time
        if d < 10 then return d end
    end
    return 1 / max(obj.framerate or 60, 1)
end

function core.commit(st)
    st.last_frame = obj.frame
    st.last_time = obj.time
end

-- ============================================================
-- audio (仕様書7章 / 実測2.3節)
-- ============================================================

-- req: { source="audiobuffer"|path, type="pcm"|"fourier"|"spectrum",
--        channel="mix"|"left"|"right"|"diff"|"dual", size=n }
-- 戻り: { values / left,right, n, rate, peak, is_silent } または nil
-- 正規化: pcm=-1..1 (16bit/32768基準), fourier=0..1 (32767クリップ実測)
function core.get_audio(req)
    local src = req.source or "audiobuffer"
    local t = req.type or "fourier"
    local ch = req.channel or "mix"
    local size = req.size or 1024

    local function grab(spec, sz)
        local buf = {}
        local okk, n, rate
        if sz then
            okk, n, rate = pcall(obj.getaudio, buf, src, spec, sz)
        else
            okk, n, rate = pcall(obj.getaudio, buf, src, spec)   -- fourierはsize指定不要
        end
        if not okk or type(n) ~= "number" or n <= 0 then return nil end
        return buf, n, rate
    end

    local fr = { type = t, channel = ch }

    if t == "pcm" then
        local SCALE = 32768
        if ch == "left" or ch == "right" then
            local b, n, rate = grab(ch == "left" and "pcm.l" or "pcm.r", size)
            if not b then return nil end
            local v = {}
            for i = 1, n do v[i] = core.safe_num(b[i], 0) / SCALE end
            fr.values, fr.n, fr.rate = v, n, rate
        else
            -- mix / diff / dual: .l/.r から自前合成 (生"pcm"はL+Rの和のため。実測7.1節)
            local bl, nl, rate = grab("pcm.l", size)
            local br, nr = grab("pcm.r", size)
            if bl and br then
                local n = min(nl, nr)
                if ch == "dual" then
                    local L, R = {}, {}
                    for i = 1, n do
                        L[i] = core.safe_num(bl[i], 0) / SCALE
                        R[i] = core.safe_num(br[i], 0) / SCALE
                    end
                    fr.left, fr.right, fr.n, fr.rate = L, R, n, rate
                else
                    local s = (ch == "diff") and -1 or 1
                    local v = {}
                    for i = 1, n do
                        v[i] = (core.safe_num(bl[i], 0) + s * core.safe_num(br[i], 0)) / (2 * SCALE)
                    end
                    fr.values, fr.n, fr.rate = v, n, rate
                end
            else
                -- フォールバック: 生pcm (L+Rの和、±65536スケール)
                local b, n, rate = grab("pcm", size)
                if not b then return nil end
                local v = {}
                for i = 1, n do v[i] = core.safe_num(b[i], 0) / 65536 end
                fr.values, fr.n, fr.rate = v, n, rate
            end
        end
    else
        -- fourier / spectrum: magnitudeは32767でクリップ(実測)
        local spec = t
        if ch == "left" then spec = t .. ".l"
        elseif ch == "right" then spec = t .. ".r" end
        local b, n, rate
        if t == "fourier" then
            b, n, rate = grab(spec, nil)   -- size指定不要
        else
            b, n, rate = grab(spec, size)
        end
        if not b then return nil end
        local v = {}
        for i = 1, n do
            local x = core.safe_num(b[i], 0) / 32767
            if x > 1 then x = 1 elseif x < 0 then x = 0 end
            v[i] = x
        end
        fr.values, fr.n, fr.rate = v, n, rate
    end

    -- 無音判定
    local mx = 0
    if fr.values then
        for i = 1, fr.n do
            local a = abs(fr.values[i])
            if a > mx then mx = a end
        end
    elseif fr.left then
        for i = 1, fr.n do
            local a = max(abs(fr.left[i]), abs(fr.right[i]))
            if a > mx then mx = a end
        end
    end
    fr.peak = mx
    fr.is_silent = mx < 0.0005
    return fr
end

-- ============================================================
-- dsp (obj非依存)
-- ============================================================

-- fourier値(0..1配列) → 表示バンド(0..1配列)
-- cfg: { bands, freq_lo, freq_hi, rate, mapping="log"|"linear",
--        agg="peak"|"rms"|"avg", min_db, max_db, gain_db, gate_db }
function core.build_bands(values, n, rate, cfg)
    local N = max(cfg.bands or 64, 1)
    local out = {}
    local f_lo = max(cfg.freq_lo or 40, 1)
    local f_hi = min(cfg.freq_hi or 15000, rate / 2)
    if f_hi <= f_lo then f_hi = f_lo * 2 end
    local log_ratio = log(f_hi / f_lo)
    local bin_per_hz = 2048 / rate  -- bin_hz(i)=rate*i/2048 (実測確定)
    local min_db = cfg.min_db or -72
    local max_db = cfg.max_db or -6
    for i = 1, N do
        local t0, t1 = (i - 1) / N, i / N
        local h0, h1
        if cfg.mapping == "linear" then
            h0 = f_lo + (f_hi - f_lo) * t0
            h1 = f_lo + (f_hi - f_lo) * t1
        else
            h0 = f_lo * exp(log_ratio * t0)
            h1 = f_lo * exp(log_ratio * t1)
        end
        local b0 = core.clamp(floor(h0 * bin_per_hz + 0.5), 1, n)
        local b1 = core.clamp(ceil(h1 * bin_per_hz - 0.5), 1, n)
        if b1 < b0 then b1 = b0 end
        local v
        if cfg.agg == "rms" then
            local s = 0
            for b = b0, b1 do
                local x = values[b] or 0
                s = s + x * x
            end
            v = sqrt(s / (b1 - b0 + 1))
        elseif cfg.agg == "avg" then
            local s = 0
            for b = b0, b1 do s = s + (values[b] or 0) end
            v = s / (b1 - b0 + 1)
        else
            v = 0
            for b = b0, b1 do
                local x = values[b] or 0
                if x > v then v = x end
            end
        end
        local db = core.db(v) + (cfg.gain_db or 0)
        if cfg.gate_db and db < cfg.gate_db then db = -999 end
        out[i] = core.db_to_display(db, min_db, max_db)
    end
    return out
end

-- DTMer AudioSpectrum_01互換のバンド生成 (リニア振幅・線形bin軸)。
-- 相対ノイズゲート(全域最大の10%) → bin範囲切り出し(DC除去) → 端部パディング
-- → ガウシアン(σ2.5) → エルミート(Catmull)補間で分割数へアップ/ダウンサンプル。
-- 戻り値は 0..1 (16bit正規化のままのリニア振幅)。
function core.build_bands_dtmer(values, n, rate, cfg)
    local mx = 0
    for i = 2, n do
        local v = values[i] or 0
        if v > mx then mx = v end
    end
    local gate = mx * (cfg.gate_ratio or 0.1)
    local b0 = core.clamp(floor(cfg.freq_lo * 2048 / rate) + 1, 1, n)
    local b1 = core.clamp(ceil(cfg.freq_hi * 2048 / rate), b0, n)
    local range = {}
    for i = b0, b1 do
        local v = (i <= 1) and 0 or (values[i] or 0)   -- DC bin除去 (DTMer: audio_data[1]=0)
        if v < gate then v = 0 end
        range[#range + 1] = v
    end
    local m = #range
    local pad = 3
    if m >= 2 then
        for k = 1, pad do
            table.insert(range, 1, max(range[1] * exp(-k), 0))
            table.insert(range, max(range[#range] * exp(-k), 0))
        end
    else
        pad = 0
    end
    core.smooth_sigma(range, #range, cfg.sigma or 2.5, true)   -- 端ゼロ埋め(DTMer方式)
    -- パディングも描画対象に含める(DTMerと同じく端が0へ落ちる山なりになる)
    local out = core.resample(range, #range, cfg.bands, "catmull")
    for i = 1, cfg.bands do
        if out[i] < 0 then out[i] = 0 end   -- 補間オーバーシュート除去
    end
    -- 端フェード: 低域端に巨大なbinがあるとガウシアンの裾が端を持ち上げるため、
    -- パディング幅相当をsmoothstepで確実に0へ落とす(山なり保証)
    local ew = max(2, ceil(cfg.bands * (pad + 1.5) / max(#range, 1)))
    for i = 1, min(ew, cfg.bands) do
        local t = (i - 0.5) / ew
        local f = t * t * (3 - 2 * t)
        out[i] = out[i] * f
        local j = cfg.bands - i + 1
        out[j] = out[j] * f
    end
    return out
end

-- attack/release (仕様書7.6節: alpha = exp(-dt/tau))
-- values を破壊的に更新し、env に保存する。
function core.envelope(values, n, env, dt, cfg)
    local ta = max(cfg.attack_ms or 30, 1) / 1000
    local tr = max(cfg.release_ms or 200, 1) / 1000
    local aa = exp(-dt / ta)
    local ar = exp(-dt / tr)
    for i = 1, n do
        local x = values[i] or 0
        local p = env[i]
        if p == nil then p = x end
        if x >= p then
            values[i] = x + (p - x) * aa
        else
            values[i] = x + (p - x) * ar
        end
        env[i] = values[i]
    end
end

-- 弾み% (バネ-ダンパー慣性)。envelope後の値に2次系で追従させ、
-- オーバーシュートと揺り戻しを加える。時間方向の「波うち」。
-- attack_msが応答速度の基準(遅いattack=ゆったり揺れる)。
function core.spring(values, n, st, dt, cfg)
    local b = core.clamp((cfg.bounce_pct or 0) / 100, 0, 1.5)
    if b <= 0 then
        st.pos = nil
        st.vel = nil
        return values
    end
    local pos, vel = st.pos, st.vel
    if not pos or #pos ~= n then
        pos, vel = {}, {}
        for i = 1, n do
            pos[i] = values[i] or 0
            vel[i] = 0
        end
        st.pos = pos
        st.vel = vel
    end
    local tau = max(cfg.attack_ms or 35, 8) / 1000
    local omega = min(2 * math.pi / (tau * 3), 45)   -- 上限≈7Hz (フリッカー防止)
    -- 減衰比: 0-100%は1.0→0.15、100-150%は0.15→0.03の低減衰域
    local zeta
    if b <= 1 then
        zeta = 1.0 - 0.85 * b
    else
        zeta = max(0.15 - 0.24 * (b - 1), 0.03)
    end
    local k = omega * omega
    local c = 2 * zeta * omega
    local steps = max(1, ceil(dt * omega / 0.9))
    local h = dt / steps
    for i = 1, n do
        local x = pos[i]
        local v = vel[i]
        local target = values[i] or 0
        for s = 1, steps do
            v = v + ((target - x) * k - c * v) * h
            x = x + v * h
        end
        if x < 0 then
            x = 0
            if v < 0 then v = 0 end
        end
        pos[i] = x
        vel[i] = v
        values[i] = core.soft_clip(x)
    end
    return values
end

-- 落下% (戻る速さ): 下降を重力落下に置き換える。値が下がるとき、
-- 一定加速度で落ちる物体のように振る舞う(指数減衰と根本的に見た目が違う)。
-- 高%ほどゆっくり落ちる。0=OFF。上昇は制限しない。
function core.fall(values, n, st, dt, fall_pct)
    local p = core.clamp((fall_pct or 0) / 100, 0, 1)
    if p <= 0 then
        st.fpos = nil
        st.fvel = nil
        return values
    end
    local fpos, fvel = st.fpos, st.fvel
    if not fpos or #fpos ~= n then
        fpos, fvel = {}, {}
        for i = 1, n do
            fpos[i] = values[i] or 0
            fvel[i] = 0
        end
        st.fpos = fpos
        st.fvel = fvel
    end
    -- 全高(1.0)をT秒で落ち切る重力: T = 0.15秒(速い)〜1.6秒(ふわり)
    local T = 0.15 + 1.45 * p
    local g = 2 / (T * T)
    for i = 1, n do
        local v = values[i] or 0
        if v >= fpos[i] then
            fpos[i] = v
            fvel[i] = 0
        else
            fvel[i] = fvel[i] + g * dt
            fpos[i] = math.max(v, fpos[i] - fvel[i] * dt)
        end
        values[i] = fpos[i]
    end
    return values
end

-- ピークホールド (メーター用)。stに ph/pht を持つ。
function core.peak_hold(value, st, dt, hold_s, decay_per_s)
    st.ph = st.ph or 0
    st.pht = st.pht or 0
    if value >= st.ph then
        st.ph = value
        st.pht = 0
    else
        st.pht = st.pht + dt
        if st.pht > (hold_s or 0.8) then
            st.ph = max(value, st.ph - (decay_per_s or 0.8) * dt)
        end
    end
    return st.ph
end

-- 自動音量 (仕様書7.7節簡易版)。display域(0..1)のピークを追従し倍率を返す。
-- cfg: { enabled, target=0.85, max_gain=6, slew=2.0 }
function core.auto_level(peak, st, dt, cfg)
    if not cfg.enabled then
        st.gain = 1
        return 1
    end
    local lv = st.level or max(peak, 0.001)
    if peak > lv then
        lv = lv + (peak - lv) * (1 - exp(-dt / 0.12))
    else
        lv = lv + (peak - lv) * (1 - exp(-dt / 1.2))
    end
    lv = max(lv, 0.001)
    st.level = lv
    if peak < 0.003 then
        return st.gain or 1   -- 無音時はgainをfreeze
    end
    local g = core.clamp((cfg.target or 0.85) / lv, 0.2, cfg.max_gain or 6)
    local prev = st.gain or g
    local slew = 1 + (cfg.slew or 2.0) * dt
    if g > prev then
        g = min(prev * slew, g)
    else
        g = max(prev / slew, g)
    end
    st.gain = g
    return g
end

-- 任意シグマのガウシアン平滑化 (生データ用)。破壊的更新。
-- 間引き前の生データにかけることでエイリアシングを除去する(DTMer方式)。
-- カーネルはシグマから自動決定しキャッシュする。
-- zero_edge=true: 範囲外を0として畳み込む(再正規化なし、DTMer方式)。
-- 端の値が自然に沈み、山なりの立ち上がりになる。falseは端値の複製。
function core.smooth_sigma(values, n, sigma, zero_edge)
    if not sigma or sigma < 0.3 then return values end
    local radius = min(math.ceil(sigma * 3), 96)
    core._KCACHE = core._KCACHE or {}
    local key = radius .. ":" .. math.floor(sigma * 100)
    local k = core._KCACHE[key]
    if not k then
        k = {}
        local s2 = 2 * sigma * sigma
        local sum = 0
        for i = -radius, radius do
            local w = exp(-(i * i) / s2)
            k[i] = w
            sum = sum + w
        end
        for i = -radius, radius do k[i] = k[i] / sum end
        core._KCACHE[key] = k
    end
    local out = {}
    for i = 1, n do
        local s = 0
        for j = -radius, radius do
            local idx = i + j
            if idx >= 1 and idx <= n then
                s = s + (values[idx] or 0) * k[j]
            elseif not zero_edge then
                s = s + (values[idx < 1 and 1 or n] or 0) * k[j]
            end
        end
        out[i] = s
    end
    for i = 1, n do values[i] = out[i] end
    return values
end

-- 空間ガウシアン平滑化 (radius 1..3)。破壊的更新。
function core.smooth(values, n, radius)
    if not radius or radius < 1 then return values end
    local tmp = {}
    local sigma = radius * 0.6
    for i = 1, n do
        local s, w = 0, 0
        for k = -radius, radius do
            local j = core.clamp(i + k, 1, n)
            local wt = exp(-(k * k) / (2 * sigma * sigma))
            s = s + (values[j] or 0) * wt
            w = w + wt
        end
        tmp[i] = s / w
    end
    for i = 1, n do values[i] = tmp[i] end
    return values
end

-- 配列を out_n 点へ再サンプル。mode="catmull"|"linear"
function core.resample(values, n, out_n, mode)
    local out = {}
    if out_n <= 1 or n <= 1 then
        local v = values[1] or 0
        for i = 1, max(out_n, 1) do out[i] = v end
        return out
    end
    for i = 1, out_n do
        local pos = (i - 1) / (out_n - 1) * (n - 1) + 1
        local i1 = floor(pos)
        local t = pos - i1
        if mode == "catmull" then
            local p0 = values[core.clamp(i1 - 1, 1, n)] or 0
            local p1 = values[core.clamp(i1, 1, n)] or 0
            local p2 = values[core.clamp(i1 + 1, 1, n)] or 0
            local p3 = values[core.clamp(i1 + 2, 1, n)] or 0
            out[i] = core.catmull(t, p0, p1, p2, p3)
        else
            local a = values[core.clamp(i1, 1, n)] or 0
            local b = values[core.clamp(i1 + 1, 1, n)] or 0
            out[i] = a + (b - a) * t
        end
    end
    return out
end

-- DCオフセット除去 (平均を引く)。破壊的更新。
function core.dc_cut(values, n)
    if n < 1 then return values end
    local s = 0
    for i = 1, n do s = s + (values[i] or 0) end
    local m = s / n
    for i = 1, n do values[i] = (values[i] or 0) - m end
    return values
end

-- ============================================================
-- quality (仕様書12章)
-- ============================================================

core.QUALITY = {
    { name = "Preview",  bands = 128,  history = 2 },
    { name = "Balanced", bands = 256,  history = 4 },
    { name = "High",     bands = 512,  history = 6 },
    { name = "Export",   bands = 1024, history = 8 },
}

-- sel: 0=Auto, 1=Preview, 2=Balanced, 3=High, 4=Export
function core.quality(sel)
    if sel and sel >= 1 and sel <= 4 then
        return core.QUALITY[sel]
    end
    local saving = false
    local okv, sv = pcall(obj.getinfo, "saving")
    if okv and sv then saving = true end
    if saving then
        return core.QUALITY[4]   -- 出力中はExport (仕様書12.2節)
    end
    return core.QUALITY[2]
end

-- ============================================================
-- error表示 (仕様書13章)
-- ============================================================

function core.error_text(msg)
    pcall(function()
        obj.setfont("メイリオ", 22, 1, 0xffdddd, 0x201010)
        obj.load("text", "rich-audio-spectrum: " .. tostring(msg))
    end)
end

return core
