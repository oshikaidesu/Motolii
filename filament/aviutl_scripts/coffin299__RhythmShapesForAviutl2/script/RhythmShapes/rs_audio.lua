local util = require("rs_util")

local M = {}

M.PCM_SCALE = 32768
-- lua.txt: fourier は元周波数の 1/2048 から 1/2 まで 1/2048 刻みの 1024 個。
M.FOURIER_DENOM = 2048

M.DEFAULT_BANDS = {
    bass = { 20, 250 },
    mid = { 250, 2000 },
    high = { 2000, nil },
}

function M.normalize_pcm(samples)
    local out = {}
    local n = samples and #samples or 0
    for i = 1, n do
        out[i] = (tonumber(samples[i]) or 0) / M.PCM_SCALE
    end
    return out
end

function M.rms(samples, i0, i1)
    if not samples then return 0 end
    i0 = i0 or 1
    i1 = i1 or #samples
    if i1 < i0 then return 0 end
    local acc = 0
    local n = 0
    for i = i0, i1 do
        local v = samples[i] or 0
        acc = acc + v * v
        n = n + 1
    end
    if n == 0 then return 0 end
    return math.sqrt(acc / n)
end

function M.fourier_freq(index, rate)
    return (rate or 0) * index / M.FOURIER_DENOM
end

function M.spectrum_freq(index, count, rate)
    if not count or count <= 0 then return 0 end
    return (rate or 0) * 0.5 * index / count
end

local function maybe_scale(bins)
    local maxv = 0
    for i = 1, #bins do
        local v = math.abs(tonumber(bins[i]) or 0)
        if v > maxv then maxv = v end
    end
    local scale = 1
    if maxv > 4 then scale = M.PCM_SCALE end
    return scale
end

function M.band_mean(bins, rate, lo, hi, freq_fn)
    if not bins or #bins == 0 then return 0 end
    local scale = maybe_scale(bins)
    local acc, n = 0, 0
    local nyquist = (rate or 0) * 0.5
    hi = hi or nyquist
    for i = 1, #bins do
        local freq = freq_fn(i, rate, #bins)
        if freq >= lo and freq < hi then
            acc = acc + math.abs(tonumber(bins[i]) or 0) / scale
            n = n + 1
        end
    end
    if n == 0 then return 0 end
    return acc / n
end

function M.lowpass_rms(samples, rate, cutoff)
    local n = samples and #samples or 0
    if n == 0 or not rate or rate <= 0 then return 0 end
    cutoff = cutoff or 120
    local dt = 1 / rate
    local rc = 1 / (2 * math.pi * cutoff)
    local a = dt / (rc + dt)
    local y = 0
    local acc = 0
    for i = 1, n do
        y = y + a * ((samples[i] or 0) - y)
        acc = acc + y * y
    end
    return math.sqrt(acc / n)
end

local function unit(v)
    return util.clamp(v or 0, 0, 1)
end

-- pcm は 16bit スケールの生値。fourier / spectrum はホストが返した列。
function M.analyze(req)
    req = req or {}
    local rate = tonumber(req.rate) or 0
    local pcm = M.normalize_pcm(req.pcm)
    local bands = req.bands or M.DEFAULT_BANDS
    local rms = M.rms(pcm, 1, #pcm)
    local bass, mid, high = 0, 0, 0
    local source = "silence"

    if req.fourier and #req.fourier > 0 and rate > 0 then
        local function freq(i, rt)
            return M.fourier_freq(i, rt)
        end
        bass = M.band_mean(req.fourier, rate, bands.bass[1], bands.bass[2], freq)
        mid = M.band_mean(req.fourier, rate, bands.mid[1], bands.mid[2], freq)
        high = M.band_mean(req.fourier, rate, bands.high[1], bands.high[2], freq)
        source = "fourier"
    elseif req.spectrum and #req.spectrum > 0 and rate > 0 then
        local function freq(i, rt, count)
            return M.spectrum_freq(i, count, rt)
        end
        bass = M.band_mean(req.spectrum, rate, bands.bass[1], bands.bass[2], freq)
        mid = M.band_mean(req.spectrum, rate, bands.mid[1], bands.mid[2], freq)
        high = M.band_mean(req.spectrum, rate, bands.high[1], bands.high[2], freq)
        source = "spectrum"
    elseif #pcm > 0 and rate > 0 then
        bass = M.lowpass_rms(pcm, rate, 120)
        local rest = math.max(0, rms - bass)
        mid = rest * 0.65
        high = rest * 0.35
        source = "pcm"
    end

    return {
        pcm = pcm,
        rate = rate,
        rms = unit(rms),
        bass = unit(bass),
        mid = unit(mid),
        high = unit(high),
        source = source,
    }
end

return M
