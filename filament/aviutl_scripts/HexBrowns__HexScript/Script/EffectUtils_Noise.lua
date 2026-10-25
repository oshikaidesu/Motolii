--[[
EffectUtils_Noise.lua
Shared noise utilities for AviUtl2 effect scripts.
]]

local EffectUtils_Noise = {}

local math_floor = math.floor
local math_max = math.max
local math_min = math.min

function EffectUtils_Noise.as_bool(value, default)
  if type(value) == "boolean" then
    return value
  elseif type(value) == "number" then
    return value ~= 0
  end
  return default
end

function EffectUtils_Noise.hash(seed, index)
  local x = math_floor(seed) * 374761393 + math_floor(index) * 668265263
  x = x % 2147483647
  x = (x * 1103515245 + 12345) % 2147483647
  return (x % 1000003) / 1000003
end

function EffectUtils_Noise.seed_random(seed, frame)
  if obj and obj.rand1 then
    return obj.rand1(seed, frame)
  end
  if rand1 then
    return rand1(seed, frame)
  end
  return EffectUtils_Noise.hash(seed, frame)
end

function EffectUtils_Noise.frame_seed(seed, frame, animated)
  if not EffectUtils_Noise.as_bool(animated, true) then
    frame = 0
  end
  return EffectUtils_Noise.seed_random(tonumber(seed) or 0, frame or 0)
end

function EffectUtils_Noise.clamp01(value)
  return math_max(0, math_min(1, value))
end

function EffectUtils_Noise.ensure_simplex()
  if simplexnoise_utl and simplexnoise_utl.noise2D then
    return simplexnoise_utl
  end
  local ok = pcall(require, "simplexnoise_utl")
  if ok and simplexnoise_utl then
    return simplexnoise_utl
  end
  return nil
end

function EffectUtils_Noise.simplex2(x, y)
  local mod = EffectUtils_Noise.ensure_simplex()
  if mod then
    return mod.noise2D(x, y)
  end
  return EffectUtils_Noise.hash(x * 17.13 + y * 31.7, x * 0.71 + y * 1.13) * 2 - 1
end

function EffectUtils_Noise.simplex3(x, y, z)
  local mod = EffectUtils_Noise.ensure_simplex()
  if mod and mod.noise3D then
    return mod.noise3D(x, y, z)
  end
  return EffectUtils_Noise.hash(x * 11.1 + y * 23.3 + z * 37.7, z * 0.17) * 2 - 1
end

function EffectUtils_Noise.perlin2(x, y, seed)
  seed = seed or 0
  local x0 = math_floor(x)
  local y0 = math_floor(y)
  local tx = x - x0
  local ty = y - y0
  local function fade(t)
    return t * t * t * (t * (t * 6 - 15) + 10)
  end
  local function grad(ix, iy)
    local h = EffectUtils_Noise.hash(seed + ix * 928371 + iy * 689287, ix + iy * 17)
    return h * 2 - 1
  end
  local a = grad(x0, y0)
  local b = grad(x0 + 1, y0)
  local c = grad(x0, y0 + 1)
  local d = grad(x0 + 1, y0 + 1)
  local u = fade(tx)
  local v = fade(ty)
  return (a * (1 - u) + b * u) * (1 - v) + (c * (1 - u) + d * u) * v
end

function EffectUtils_Noise.fbm2(x, y, opts)
  opts = opts or {}
  local octaves = opts.octaves or 4
  local lacunarity = opts.lacunarity or 2
  local gain = opts.gain or 0.5
  local seed = opts.seed or 0
  local amplitude = 1
  local frequency = opts.frequency or 1
  local sum = 0
  local norm = 0
  for i = 0, octaves - 1 do
    sum = sum + amplitude * EffectUtils_Noise.perlin2(x * frequency, y * frequency, seed + i * 131)
    norm = norm + amplitude
    amplitude = amplitude * gain
    frequency = frequency * lacunarity
  end
  if norm > 0 then
    return sum / norm
  end
  return 0
end

function EffectUtils_Noise.noise_offset(seed, frame, animated)
  return EffectUtils_Noise.frame_seed(seed, frame, animated)
end

return EffectUtils_Noise
