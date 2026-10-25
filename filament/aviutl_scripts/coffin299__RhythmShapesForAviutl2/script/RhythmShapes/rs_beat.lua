local util = require("rs_util")
local audio = require("rs_audio")

local M = {}

local function dft_mags(samples)
    local n = #samples
    local half = math.floor(n / 2)
    local mags = {}
    for k = 0, half - 1 do
        local re, im = 0, 0
        local ang = -2 * math.pi * k / n
        for i = 0, n - 1 do
            local a = ang * i
            local v = samples[i + 1]
            re = re + v * math.cos(a)
            im = im + v * math.sin(a)
        end
        mags[k + 1] = math.sqrt(re * re + im * im) / n
    end
    return mags
end

local function resample(samples, dest_n)
    local n = #samples
    if n == 0 then return {} end
    if dest_n > n then dest_n = n end
    if dest_n < 2 then dest_n = 2 end
    local out = {}
    for i = 0, dest_n - 1 do
        local pos = 1
        if dest_n > 1 then
            pos = 1 + (n - 1) * i / (dest_n - 1)
        end
        local i0 = math.floor(pos)
        local i1 = i0 + 1
        if i1 > n then i1 = n end
        local t = pos - i0
        out[i + 1] = (samples[i0] or 0) * (1 - t) + (samples[i1] or 0) * t
    end
    return out
end

function M.spectral_flux(a, b)
    local na = resample(a, 64)
    local nb = resample(b, 64)
    local ma = dft_mags(na)
    local mb = dft_mags(nb)
    local n = math.min(#ma, #mb)
    if n == 0 then return 0 end
    local acc = 0
    for i = 1, n do
        local d = mb[i] - ma[i]
        if d > 0 then acc = acc + d end
    end
    return acc / n
end

local function envelope(pcm, rate)
    local hop = math.max(1, math.floor((rate or 1) * 0.01))
    local env = {}
    local centers = {}
    local i = 1
    local n = #pcm
    while i <= n do
        local j = math.min(n, i + hop - 1)
        env[#env + 1] = audio.rms(pcm, i, j)
        centers[#centers + 1] = math.floor((i + j) / 2)
        i = j + 1
    end
    return env, centers, hop
end

local function slice(pcm, i0, i1)
    local out = {}
    for i = i0, i1 do
        out[#out + 1] = pcm[i] or 0
    end
    return out
end

function M.estimate_bpm(env, hop_sec)
    local n = #env
    if n < 8 or not hop_sec or hop_sec <= 0 then
        return nil, 0
    end
    local mean = 0
    for i = 1, n do mean = mean + env[i] end
    mean = mean / n
    local var = 0
    for i = 1, n do
        local d = env[i] - mean
        var = var + d * d
    end
    var = var / n
    if var < 1e-6 then
        return nil, 0
    end

    local min_lag = math.floor((60 / 180) / hop_sec)
    local max_lag = math.floor((60 / 70) / hop_sec)
    if min_lag < 1 then min_lag = 1 end
    if max_lag >= n - 1 then
        return nil, 0
    end

    local function corr(lag)
        local s, c = 0, 0
        local last = n - lag
        for i = 1, last do
            s = s + env[i] * env[i + lag]
            c = c + 1
        end
        if c == 0 then return 0 end
        return s / c
    end

    local c0 = corr(0)
    if c0 < 1e-8 then return nil, 0 end
    local best, best_lag = 0, nil
    for lag = min_lag, max_lag do
        local c = corr(lag) / c0
        if best_lag == nil or c > best + 1e-6 or (math.abs(c - best) <= 1e-6 and lag < best_lag) then
            best = c
            best_lag = lag
        end
    end
    if not best_lag or best < 0.55 then
        return nil, best or 0
    end
    return 60 / (best_lag * hop_sec), best
end

-- pcm は -1..1。結果は窓だけから決まり、前フレームを持たない。
function M.detect(pcm, rate, opts)
    opts = opts or {}
    pcm = pcm or {}
    rate = tonumber(rate) or 0
    local sensitivity = util.clamp(tonumber(opts.sensitivity) or 50, 0, 100)
    local min_interval = tonumber(opts.min_interval) or 0.12
    if min_interval < 0 then min_interval = 0 end
    local weights = opts.weights or {}
    local wb = util.clamp((tonumber(weights.bass) or 100) / 100, 0, 1)
    local wm = util.clamp((tonumber(weights.mid) or 40) / 100, 0, 1)
    local wh = util.clamp((tonumber(weights.high) or 20) / 100, 0, 1)

    local result = {
        fired = false,
        age = math.huge,
        strength = 0,
        threshold = 0,
        flux = 0,
        peak_sample = nil,
        estimated_bpm = nil,
        bpm_confidence = 0,
    }
    if #pcm < 2 or rate <= 0 then
        return result
    end

    local env, centers, hop = envelope(pcm, rate)
    local hop_sec = hop / rate
    local bpm, conf = M.estimate_bpm(env, hop_sec)
    result.estimated_bpm = bpm
    result.bpm_confidence = conf or 0

    local mid = math.floor(#pcm / 2)
    local first = slice(pcm, 1, math.max(1, mid))
    local second = slice(pcm, mid + 1, #pcm)
    local flux = M.spectral_flux(first, second)
    result.flux = flux
    local bass_delta = math.max(0, audio.lowpass_rms(second, rate, 120) - audio.lowpass_rms(first, rate, 120))
    local rest_delta = math.max(0, audio.rms(second) - audio.rms(first))

    local baseline = 0
    for i = 1, #env do baseline = baseline + env[i] end
    baseline = baseline / #env
    local factor = 1.15 - sensitivity / 100
    local threshold = (0.02 + baseline * 0.35) * factor
    result.threshold = threshold

    local half = math.floor(#env / 2)
    local last_t = -math.huge
    local kept_i = nil
    for i = 2, #env do
        local rise = env[i] - env[i - 1]
        if rise < 0 then rise = 0 end
        local score = rise
        if i > half then
            score = score + bass_delta * wb + rest_delta * wm * 0.5 + flux * wh
        end
        local is_peak = score >= threshold and (i == #env or env[i] >= env[i + 1])
        if is_peak then
            local t = centers[i] / rate
            if t - last_t >= min_interval then
                kept_i = i
                last_t = t
            end
        end
    end

    if kept_i then
        result.peak_sample = centers[kept_i]
        result.age = (#pcm - centers[kept_i]) / rate
        if result.age < 0 then result.age = 0 end
        local strength = env[kept_i] / (env[kept_i] + threshold + 0.02)
        result.strength = util.clamp(strength, 0, 1)
        local hold = math.max(min_interval, hop_sec)
        result.fired = result.age <= hold
    end
    return result
end

return M
