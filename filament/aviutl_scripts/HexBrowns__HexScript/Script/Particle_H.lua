-- Particle_H.lua — 拡張パーティクル_H の計算モジュール（obj に触らない純粋な計算）
--
-- 呼び手: Script/@拡張パーティクル_H.anm2 の「本体」。require("Particle_H")
-- 計画:   AI/specifications/20261002_拡張パーティクル_H_spec.md（5.3 時間と乱数 / 5.4 描画）
-- 検証器: AI/scripts/拡張パーティクル_H/verify_core.py（この本文を luaJIT.dll でそのまま走らせる）
--
-- 時間はすべて「シミュレーション時刻」（秒）。オブジェクトの時刻との対応（開始時間・逆再生）は呼び手が持つ。
-- 粒子は「時刻を与えれば決まる」形で計算する。状態を持たないので、シークしても同じ絵になる。

local bit = require("bit")
local ffi = require("ffi")
local floor, ceil, sqrt, sin, cos, atan2 = math.floor, math.ceil, math.sqrt, math.sin, math.cos, math.atan2
local abs, max, min, pi, huge = math.abs, math.max, math.min, math.pi, math.huge
local RAD = pi / 180

local M = {}
M.VERSION = "0.19.1"      -- v0.19.1: 終了時に消えるを、寿命を縮める作りから「終わりまでに消え終わらない粒子は生まない」に直した
                          -- v0.19.0:ノイズ場の元のノイズ（勾配ノイズ・ブロック）、積分法（陽的オイラー・RK4）と刻みの倍率を足した（既定は今までの計算）
                          -- v0.18.0: 色の変化・軌跡（帯）の補間に色空間と色相経路、出た所の色の近い色選びに色差を足した（既定は今までの計算）
-- 焼き付けのシグネチャに入れる版。同じ設定で動き（位置と回転）が変わるか、ファイルの形が変わる版でだけ上げる。
-- 機能を足しただけの版で M.VERSION を入れると、既定のまま動きが同じでも焼き付けが全部「設定と違う」になり、
-- 読むだけ（計算しない）の焼き付けでは粒が描かれなくなる（v0.19.0 で気づいた。v0.19.0 の既定は v0.18.0 と同じ動き）
M.BAKE_VERSION = "0.18.0"
M.MAX_PARTICLES = 10000   -- 生きている粒子の上限（実機 PRP-12 / 18 で決めた）
M.INTER_MAX = 3000        -- 粒子どうし（v0.12.0）で一緒に進める粒子の上限（超えた分は力を受けず、今までどおり粒子ごとに進める）
M.INTER_CP = 64           -- 粒子どうしのチェックポイントの数の上限（間隔はオブジェクトの長さ ÷ この数、1 秒より短くしない）
M.ALPHA_LEVELS = 16       -- まとめ描きは四角形ごとの透明度を持てないので、段に分けて描く
M.MAX_RUNS = 256          -- 描く順を保つための呼び出しの上限。超えたら段ごとにまとめる
M.MAX_INSTANCES = 60000   -- 描くもの（粒子・軌跡の点・ファンネル・円環）の上限
M.MAX_SWITCHES = 64       -- 素材を読み込み直す回数（鍵の変わり目）の上限。超えたら鍵ごとにまとめる
M.MAX_DELAUNAY = 2000     -- 三角形で塗るときに使う粒子の上限（新しい粒子から）
M.AUDIO_REF = 1000        -- 音の帯の値（obj.getaudio の spectrum）がこの大きさで最大とみなす（実機 EPR-99: 振幅 0.8 の 100Hz で 1260）
M.LUM_REF = 64            -- 力場「画像の明るさ」: この距離（px）で明るさが 0 から 1 に変わる勾配で、強さいっぱいに押す

-- 乱数の用途（同じ粒子でも用途ごとに独立した値にする）
local CH = {
  dir = 1, zdir = 2, speed = 3, accel = 4, life = 5, gx = 6, gy = 7, gz = 8,
  rx0 = 9, ry0 = 10, rz0 = 11, vrx = 12, vry = 13, vrz = 14, rev = 15,
  alpha = 16, zoom = 17, birth = 18, pos1 = 19, pos2 = 20, pos3 = 21, pos4 = 22,
  swing_phase = 23, swing_speed = 24, swing_amp = 25, racc = 26, rtime = 27,
  disp1 = 28, disp2 = 29, sus = 30, att = 33, irr = 34, wind = 36, noise = 37, tdisp = 38, tstop = 39,
  tatt = 40, attxy = 41, attz = 42, dspeed = 43, jit = 44, wob = 50, wobs = 60, wobp = 61, orbw = 62,
  orbvr = 63, orbc = 64, orbth = 67, mat = 70, col = 80, filt = 81, fan = 83, cust = 100, fld = 120, blink = 200, look = 210, prev = 220, path = 230, gat = 240, gst = 250, pace = 260, child = 270, shard = 280,
  dlink = 290, trace = 291,
}
M.CH = CH

----------------------------------------------------------------------------- 乱数

-- 32bit の xorshift だけで組む（乗算の桁あふれで精度を落とさない）
local function mix(x)
  x = bit.bxor(x, bit.lshift(x, 13))
  x = bit.bxor(x, bit.rshift(x, 17))
  return bit.bxor(x, bit.lshift(x, 5))
end

-- シードを混ぜる値。シードが 0 以上ならレイヤーも混ぜる（負ならレイヤーに依らない。原作と同じ意味）
function M.seed_key(seed, layer)
  seed = floor(seed)
  if seed >= 0 then
    return bit.tobit(seed * 1009 + (layer or 0) + 1)
  end
  return bit.tobit(-seed * 1009 + 0x5BD1E995)
end

-- [0,1) の一様乱数。k = 粒子（または組）の番号、ch = 用途、sk = seed_key
local function rnd(k, ch, sk)
  local x = mix(bit.tobit(k + 0x6E5A3B1))
  x = mix(bit.bxor(x, bit.tobit(ch * 40503 + 0x1B873593)))
  x = mix(bit.bxor(x, sk))
  x = mix(bit.tobit(x + 0x2545F491))
  return (x % 0x1000000) / 0x1000000
end
M.rnd = rnd

----------------------------------------------------------------------------- 小道具

-- 個別微調整: 正の値は元の値の %、負の値は絶対幅（原作の説明書 09 の式）
local function vary(base, p, r)
  if not p or p == 0 then return base end
  local w = p > 0 and abs(base) * p / 100 or -p
  return base + (r * 2 - 1) * w
end
M.vary = vary

-- ∫_0^d clamp(m0 + acc*s, lo, hi) ds（回転の加速を閉じた式で積分する）
local function ramp_integral(m0, acc, lo, hi, d)
  if d <= 0 then return 0 end
  if acc == 0 then return min(max(m0, lo), hi) * d end
  if acc > 0 and m0 >= hi then return m0 * d end
  if acc < 0 and m0 <= lo then return m0 * d end
  local target = acc > 0 and hi or lo
  local ts = (target - m0) / acc
  if ts >= d then return m0 * d + 0.5 * acc * d * d end
  return m0 * ts + 0.5 * acc * ts * ts + target * (d - ts)
end
M.ramp_integral = ramp_integral

----------------------------------------------------------------------------- 寿命に沿った変化のカーブ（6.2）

-- 補間の種類（UI の「補間」の番号）: 0 直線 / 1 瞬間 / 2 反復 / 3 加速 / 4 減速 / 5 加減速 / 6 バウンス / 7 弾性 / 8 バック / 9 カーブエディタ_H
M.CURVE_NAMES = { [0] = "直線", "瞬間", "反復", "加速", "減速", "加減速", "バウンス", "弾性", "バック", "カーブエディタ_H" }

local function bounce_out(x)
  local n1, d1 = 7.5625, 2.75
  if x < 1 / d1 then return n1 * x * x end
  if x < 2 / d1 then x = x - 1.5 / d1; return n1 * x * x + 0.75 end
  if x < 2.5 / d1 then x = x - 2.25 / d1; return n1 * x * x + 0.9375 end
  x = x - 2.625 / d1
  return n1 * x * x + 0.984375
end

-- 0..1 の位置 x を 0..1 の割合にする関数。直線と瞬間は nil（呼び手がそのまま扱う）。
-- n = 反復の回数（奇数は終わりの値、偶数は始めの値で終わる。原作の移動タイプ 2 以上と同じ）、fn = カーブエディタ_H のスロットの関数
function M.curve(kind, n, fn)
  kind = floor(tonumber(kind) or 0)
  if kind == 2 then
    n = max(floor(tonumber(n) or 2), 1)
    return function(x)
      local y = min(max(x, 0), 1) * n
      local k = floor(y)
      if k >= n then return n % 2 == 1 and 1 or 0 end
      local f = y - k
      return k % 2 == 0 and f or 1 - f
    end
  elseif kind == 3 then
    return function(x) return x * x end
  elseif kind == 4 then
    return function(x) return 1 - (1 - x) * (1 - x) end
  elseif kind == 5 then
    return function(x) if x < 0.5 then return 2 * x * x end return 1 - 2 * (1 - x) * (1 - x) end
  elseif kind == 6 then
    return function(x) return bounce_out(min(max(x, 0), 1)) end
  elseif kind == 7 then
    return function(x)
      if x <= 0 then return 0 elseif x >= 1 then return 1 end
      return 2 ^ (-10 * x) * sin((x * 10 - 0.75) * (2 * pi / 3)) + 1
    end
  elseif kind == 8 then
    return function(x)
      local c1 = 1.70158
      local y = x - 1
      return 1 + (c1 + 1) * y * y * y + c1 * y * y
    end
  elseif kind == 9 and fn then
    return function(x)
      local ok, v = pcall(fn, x)
      if ok and type(v) == "number" and v == v then return v end
      return x
    end
  end
  return nil
end

-- 速度倍率（寿命の位置 x → 倍率）。始・終が 100% で直線なら nil（掛けない）
function M.speed_curve(z)
  if not z or ((z.vm0 or 100) == 100 and (z.vm1 or 100) == 100) then return nil end
  local m0, m1 = z.vm0 / 100, z.vm1 / 100
  local sh = M.curve(z.vmode, z.vrep, z.vfn)
  if (tonumber(z.vmode) or 0) == 1 then
    return function(x) return x >= 1 and m1 or m0 end
  end
  if sh then return function(x) return m0 + (m1 - m0) * sh(x) end end
  return function(x) return m0 + (m1 - m0) * x end
end

-- 速度倍率つきの閉じた式。自分の速さの項だけに倍率を掛け、∫ 倍率(a/L) × 速さ(a) da を 16 区間のシンプソンで求める
-- （加速度が負なら止まったところで止める。cf_pos と同じ）
local function cf_pos_mul(px, py, pz, ux, uy, uz, v0, ac, gx, gy, gz, tau, L, vm)
  local function f(a)
    local v = v0 + ac * a
    if ac < 0 and v0 > 0 and v < 0 then v = 0 end
    return vm(a / L) * v
  end
  local s = 0
  if tau > 0 then
    local N = 16
    local hh = tau / N
    local sum = f(0) + f(tau)
    for i = 1, N - 1 do sum = sum + f(i * hh) * (i % 2 == 1 and 4 or 2) end
    s = sum * hh / 3
  end
  local v = f(tau)
  local hx = 0.5 * tau * tau
  return px + ux * s + gx * hx, py + uy * s + gy * hx, pz + uz * s + gz * hx, ux * v + gx * tau, uy * v + gy * tau
end
M.cf_pos_mul = cf_pos_mul

-- 中間点つきの折れ線の値。始点は (0, v0)、終点は (Lend, v1)、中間点は mids（時刻の昇順の {t, v}）。
-- 範囲外（0 以下・Lend 以上）の中間点は使わない。mode: 1=瞬間（次の点まで前の値を保つ）/ それ以外は直線。
-- shape: 区間の中の位置（0..1）を割合にする関数（M.curve。nil なら直線）
local function eval_keys(mids, v0, v1, Lend, x, mode, ri, rt, di, dt, shape)
  local pt, pv = 0, v0
  for i = 1, #mids do
    local t, v = mids[i][1], mids[i][2]
    if i == ri then t = rt elseif i == di then t = dt end
    if t > 0 and t < Lend then
      if x < t then
        if mode == 1 then return pv end
        local f = (x - pt) / (t - pt)
        if shape then f = shape(f) end
        return pv + (v - pv) * f
      end
      pt, pv = t, v
    end
  end
  if x >= Lend then return v1 end
  if mode == 1 or Lend <= pt then return pv end
  local f = (x - pt) / (Lend - pt)
  if shape then f = shape(f) end
  return pv + (v1 - pv) * f
end
M.eval_keys = eval_keys

-- 「拡大率と透過率」の設定を整える（中間点を時刻順に並べる）
function M.prepare_zoal(z)
  if not z then return nil end
  local function mids(ts, vs)
    local list = {}
    for i = 1, min(#ts, #vs) do list[#list + 1] = { ts[i], vs[i] } end
    table.sort(list, function(a, b) return a[1] < b[1] end)
    return list
  end
  z.zm = mids(z.zmid_t or {}, z.zmid_v or {})
  z.am = mids(z.amid_t or {}, z.amid_v or {})
  z.zshape = M.curve(z.zmode, z.zrep, z.zfn)
  z.ashape = M.curve(z.amode, z.arep, z.afn)
  z.vmul = M.speed_curve(z)
  return z
end

----------------------------------------------------------------------------- 放出の数

-- 頻度（個/秒）の時系列から、累積の放出回数 N(t) とその逆関数を作る。
-- samples[i] = 時刻 t0 + (i-1)*dt の頻度。区間の中は一定とみなす（累積は折れ線）
function M.rate_table(samples, dt)
  local cum = { [1] = 0 }
  local n = #samples
  for i = 1, n - 1 do
    cum[i + 1] = cum[i] + max(samples[i], 0) * dt
  end
  local tab = { samples = samples, cum = cum, dt = dt, n = n }
  function tab.count(t)            -- t までの累積の放出回数
    if t <= 0 then return 0 end
    local i = floor(t / dt) + 1
    if i >= n then return cum[n] + max(samples[n], 0) * (t - (n - 1) * dt) end
    return cum[i] + max(samples[i], 0) * (t - (i - 1) * dt)
  end
  function tab.time_of(e)          -- 累積が e になる時刻（e 回目の放出の時刻）
    if e <= 0 then
      -- 最初の 1 個は頻度が 0 より大きくなった時刻に出す（頻度が 0 のままなら出さない）
      for i = 1, n do if samples[i] > 0 then return (i - 1) * dt end end
      return huge
    end
    if e >= cum[n] then
      local r = max(samples[n], 0)
      if r <= 0 then return huge end
      return (n - 1) * dt + (e - cum[n]) / r
    end
    local lo, hi = 1, n
    while hi - lo > 1 do
      local mid = floor((lo + hi) / 2)
      if cum[mid] <= e then lo = mid else hi = mid end
    end
    local r = max(samples[lo], 0)
    if r <= 0 then return (lo - 1) * dt end
    return (lo - 1) * dt + (e - cum[lo]) / r
  end
  function tab.rate(t)
    local i = min(max(floor(t / dt) + 1, 1), n)
    return max(samples[i], 0)
  end
  return tab
end

function M.rate_const(r)
  r = max(r, 0)
  return {
    count = function(t) return t <= 0 and 0 or r * t end,
    time_of = function(e) if r <= 0 then return huge end; return max(e, 0) / r end,
    rate = function() return r end,
  }
end

-- ポアソン分布の数（平均 lam）。乱数は番号 f と鍵 sk で決まる（大きい平均は正規分布で近づける）
local function poisson(lam, f, sk, ch)
  if lam <= 0 then return 0 end
  local u = rnd(f, ch, sk)
  if lam < 30 then
    local kk, p = 0, math.exp(-lam)
    local F = p
    while u > F and kk < 200 do
      kk = kk + 1
      p = p * lam / kk
      F = F + p
    end
    return kk
  end
  local u2 = max(rnd(f, ch + 1, sk), 1e-12)
  local z = sqrt(-2 * math.log(u2)) * cos(2 * pi * u)
  return max(floor(lam + sqrt(lam) * z + 0.5), 0)
end
M.poisson = poisson

-- 出し方（v0.8.0）: 元の放出の数 R を、フレームごとに乱数で決めた数にする。mode 1 まばら（ポアソン）/ 2 まとめ撃ち（まとめる数 nclu ずつ、
-- ばらつき var %）。長い目で見た数は元と同じ。フレームの中では等間隔に出す（rate_table）。数は時刻と鍵だけで決まる
function M.rate_paced(R, mode, nclu, var, sk, dt, now)
  local nS = floor(max(now, 0) / dt) + 2
  local smp = {}
  nclu = max(floor(nclu or 1), 1)
  for f = 0, nS - 1 do
    local lam = max(R.count((f + 1) * dt) - R.count(f * dt), 0)
    local cnt
    if mode == 1 then
      cnt = poisson(lam, f, sk, CH.pace)
    else
      local nc = poisson(lam / nclu, f, sk, CH.pace)
      cnt = 0
      for c = 1, nc do
        cnt = cnt + max(floor(nclu * (1 + (rnd(f * 64 + c, CH.pace + 3, sk) * 2 - 1) * (var or 0) / 100) + 0.5), 1)
      end
    end
    smp[f + 1] = cnt / dt
  end
  return M.rate_table(smp, dt)
end

----------------------------------------------------------------------------- 他レイヤーの形（アルファの粗い格子）

-- 画像（RGBA 32bit。obj.getpixeldata の並び。アルファはストレート）から、粗い格子のアルファと明るさを作る。
-- 1 マスの大きさ cs は、長い辺が 256 マス以下になるように決める。マスの値は中の画素（最大 4x4 点）の平均。
-- 明るさ（Rec.709。0..1）はアルファで重み付けした平均。thr（0..1）以上のマスを「形の中」とみなす。座標は画像の中心から（px）
function M.build_mask(data, w, h, thr)
  local p = ffi.cast("const uint8_t*", data)
  thr = max(thr or 0.5, 1 / 255)
  local cs = max(1, ceil(max(w, h) / 256))
  local gw, gh = ceil(w / cs), ceil(h / cs)
  local a = ffi.new("float[?]", gw * gh)
  local lum = ffi.new("float[?]", gw * gh)
  local col = ffi.new("int32_t[?]", gw * gh)
  local st = max(1, floor(cs / 4))
  local cells, nc = {}, 0
  for gy = 0, gh - 1 do
    local y0, y1 = gy * cs, min(gy * cs + cs, h) - 1
    for gx = 0, gw - 1 do
      local x0, x1 = gx * cs, min(gx * cs + cs, w) - 1
      local sum, cnt, ls, sr, sg, sb = 0, 0, 0, 0, 0, 0
      for y = y0, y1, st do
        local row = y * w
        for x = x0, x1, st do
          local o = (row + x) * 4
          local al = p[o + 3]
          sum = sum + al
          cnt = cnt + 1
          if al > 0 then
            ls = ls + (0.2126 * p[o] + 0.7152 * p[o + 1] + 0.0722 * p[o + 2]) * al
            sr, sg, sb = sr + p[o] * al, sg + p[o + 1] * al, sb + p[o + 2] * al
          end
        end
      end
      local i = gy * gw + gx
      local v = cnt > 0 and sum / (cnt * 255) or 0
      a[i] = v
      lum[i] = sum > 0 and ls / (sum * 255) or 0
      -- マスの色（アルファで重みを付けた平均。0xRRGGBB）
      if sum > 0 then col[i] = floor(sr / sum + 0.5) * 65536 + floor(sg / sum + 0.5) * 256 + floor(sb / sum + 0.5) end
      if v >= thr then
        nc = nc + 1
        cells[nc] = i
      end
    end
  end
  return { w = w, h = h, cs = cs, gw = gw, gh = gh, a = a, lum = lum, col = col, thr = thr, cells = cells, nc = nc, picks = {} }
end

-- 点 (lx, ly)（画像の中心から）のマスの値。画像の外は 0
function M.mask_at(Mk, lx, ly)
  local gx, gy = floor((lx + Mk.w / 2) / Mk.cs), floor((ly + Mk.h / 2) / Mk.cs)
  if gx < 0 or gy < 0 or gx >= Mk.gw or gy >= Mk.gh then return 0 end
  return Mk.a[gy * Mk.gw + gx]
end

-- 出すマスの選び方。area: 0 形の中 / 1 形の縁（形の中で、上下左右のどれかが形の外か画像の外）。
-- weight: 0 なし / 1 明るい所ほど多く / 2 暗い所ほど多く。重みの累積を持つ（二分探索で選ぶ）
function M.mask_pick(Mk, area, weight)
  area, weight = floor(area or 0), floor(weight or 0)
  local key = area .. "/" .. weight
  if Mk.picks[key] then return Mk.picks[key] end
  local gw, gh, a, thr = Mk.gw, Mk.gh, Mk.a, Mk.thr
  local function inside(x, y) return x >= 0 and y >= 0 and x < gw and y < gh and a[y * gw + x] >= thr end
  local cells, cum, total = {}, {}, 0
  for _, i in ipairs(Mk.cells) do
    local x, y = i % gw, floor(i / gw)
    if area == 0 or not (inside(x - 1, y) and inside(x + 1, y) and inside(x, y - 1) and inside(x, y + 1)) then
      local wgt = 1
      if weight == 1 then wgt = Mk.lum[i] elseif weight == 2 then wgt = 1 - Mk.lum[i] end
      if wgt > 0 then
        total = total + wgt
        cells[#cells + 1] = i
        cum[#cum + 1] = total
      end
    end
  end
  local pk = { cells = cells, cum = cum, total = total, n = #cells }
  Mk.picks[key] = pk
  return pk
end

-- 形の中のマスから 1 点を選ぶ（r1 でマス、r2・r3 でマスの中の位置）。pk を渡すとその選び方（縁・重み）。形が無ければ nil
function M.mask_point(Mk, r1, r2, r3, pk)
  if not Mk then return nil end
  local c
  if pk then
    if pk.n <= 0 then return nil end
    local target = r1 * pk.total
    local lo, hi = 1, pk.n
    while lo < hi do
      local mid = floor((lo + hi) / 2)
      if pk.cum[mid] > target then hi = mid else lo = mid + 1 end
    end
    c = pk.cells[lo]
  else
    if Mk.nc <= 0 then return nil end
    c = Mk.cells[min(floor(r1 * Mk.nc), Mk.nc - 1) + 1]
  end
  local gx, gy = c % Mk.gw, floor(c / Mk.gw)
  return min((gx + r2) * Mk.cs, Mk.w) - Mk.w / 2, min((gy + r3) * Mk.cs, Mk.h) - Mk.h / 2, c
end

-- 形の縁までの符号付き距離（px。形の中が負）。マスの中心どうしの面取り距離（縦横 1・斜め √2）を 2 回なめて求める。
-- 画像の外は形の外とみなす
function M.mask_sdf(Mk)
  if Mk.sdf then return Mk.sdf end
  local gw, gh, a, thr = Mk.gw, Mk.gh, Mk.a, Mk.thr
  local n = gw * gh
  local BIG, R2 = 1e9, 1.41421356
  local function pass(want_inside)
    local d = ffi.new("float[?]", n)
    for i = 0, n - 1 do d[i] = ((a[i] >= thr) == want_inside) and 0 or BIG end
    local outside = want_inside and BIG or 0
    local function at(x, y)
      if x < 0 or y < 0 or x >= gw or y >= gh then return outside end
      return d[y * gw + x]
    end
    for y = 0, gh - 1 do
      for x = 0, gw - 1 do
        local i = y * gw + x
        d[i] = min(d[i], at(x - 1, y) + 1, at(x, y - 1) + 1, at(x - 1, y - 1) + R2, at(x + 1, y - 1) + R2)
      end
    end
    for y = gh - 1, 0, -1 do
      for x = gw - 1, 0, -1 do
        local i = y * gw + x
        d[i] = min(d[i], at(x + 1, y) + 1, at(x, y + 1) + 1, at(x + 1, y + 1) + R2, at(x - 1, y + 1) + R2)
      end
    end
    return d
  end
  local din, dout = pass(true), pass(false)
  local s = ffi.new("float[?]", n)
  for i = 0, n - 1 do
    if a[i] >= thr then s[i] = -(dout[i] - 0.5) * Mk.cs else s[i] = (din[i] - 0.5) * Mk.cs end
  end
  Mk.sdf = s
  return s
end

-- 点 (lx, ly) の符号付き距離と、その勾配（外へ向く。長さはおよそ 1）。マスの中心の値を直線で補う。
-- 格子の外の点は、いちばん近い格子の端の値に、そこからの距離を足す（勾配は端から点への向き）
local function sdf_bilinear(s, gw, gh, cs, fx, fy)
  local x0, y0 = floor(fx), floor(fy)
  if x0 > gw - 2 then x0 = gw - 2 end
  if y0 > gh - 2 then y0 = gh - 2 end
  if x0 < 0 then x0 = 0 end
  if y0 < 0 then y0 = 0 end
  local x1, y1 = min(x0 + 1, gw - 1), min(y0 + 1, gh - 1)
  local tx, ty = fx - x0, fy - y0
  local v00, v10 = s[y0 * gw + x0], s[y0 * gw + x1]
  local v01, v11 = s[y1 * gw + x0], s[y1 * gw + x1]
  local d = (v00 * (1 - tx) + v10 * tx) * (1 - ty) + (v01 * (1 - tx) + v11 * tx) * ty
  local gx = ((v10 - v00) * (1 - ty) + (v11 - v01) * ty) / cs
  local gy = ((v01 - v00) * (1 - tx) + (v11 - v10) * tx) / cs
  return d, gx, gy
end

function M.sdf_at(Mk, lx, ly)
  local s = Mk.sdf or M.mask_sdf(Mk)
  local gw, gh, cs = Mk.gw, Mk.gh, Mk.cs
  local fx, fy = (lx + Mk.w / 2) / cs - 0.5, (ly + Mk.h / 2) / cs - 0.5
  local cx_, cy_ = min(max(fx, 0), gw - 1), min(max(fy, 0), gh - 1)
  if cx_ ~= fx or cy_ ~= fy then
    local ex, ey = (fx - cx_) * cs, (fy - cy_) * cs
    local dist = sqrt(ex * ex + ey * ey)
    local d0 = sdf_bilinear(s, gw, gh, cs, cx_, cy_)
    return d0 + dist, ex / dist, ey / dist
  end
  return sdf_bilinear(s, gw, gh, cs, fx, fy)
end

-- 点 (lx, ly) にいちばん近い形の中の点（形の中なら点のまま。形が無ければ点のまま）。
-- 距離の場（面取り距離は斜めで長めに出る）の値を探す半径の上限にし、その中の縁のマスを 8 マス四方の桶から探す（v0.7.0）
function M.nearest_inside(Mk, lx, ly)
  local d = M.sdf_at(Mk, lx, ly)
  if d <= 0 then return lx, ly end
  local B, cs, gw, gh = 8, Mk.cs, Mk.gw, Mk.gh
  local E = Mk.edges
  if not E then
    local pk = M.mask_pick(Mk, 1, 0)
    E = { nbx = ceil(gw / B), nby = ceil(gh / B), b = {}, n = pk.n }
    for j = 1, pk.n do
      local c = pk.cells[j]
      local key = floor(floor(c / gw) / B) * E.nbx + floor((c % gw) / B)
      local t = E.b[key]
      if not t then t = {}; E.b[key] = t end
      t[#t + 1] = c
    end
    Mk.edges = E
  end
  if E.n <= 0 then return lx, ly end
  local R = d * 1.1 + 2 * cs
  local fx, fy = (lx + Mk.w / 2) / cs, (ly + Mk.h / 2) / cs
  local bx0, bx1 = max(floor((fx - R / cs) / B), 0), min(floor((fx + R / cs) / B), E.nbx - 1)
  local by0, by1 = max(floor((fy - R / cs) / B), 0), min(floor((fy + R / cs) / B), E.nby - 1)
  local qx, qy, best = lx, ly, huge
  for by = by0, by1 do
    for bx = bx0, bx1 do
      local t = E.b[by * E.nbx + bx]
      if t then
        for _, c in ipairs(t) do
          local cx_ = (c % gw + 0.5) * cs - Mk.w / 2
          local cy_ = (floor(c / gw) + 0.5) * cs - Mk.h / 2
          local d2 = (cx_ - lx) * (cx_ - lx) + (cy_ - ly) * (cy_ - ly)
          if d2 < best then qx, qy, best = cx_, cy_, d2 end
        end
      end
    end
  end
  return qx, qy
end

-- 明るさの格子を、マスの中心の値から直線で補って読む（格子の外は 0）
local function lum_at(Mk, fx, fy)
  local gw, gh = Mk.gw, Mk.gh
  local x0, y0 = floor(fx), floor(fy)
  local tx, ty = fx - x0, fy - y0
  local L = Mk.lum
  local function v(x, y)
    if x < 0 or y < 0 or x >= gw or y >= gh then return 0 end
    return L[y * gw + x]
  end
  return (v(x0, y0) * (1 - tx) + v(x0 + 1, y0) * tx) * (1 - ty) + (v(x0, y0 + 1) * (1 - tx) + v(x0 + 1, y0 + 1) * tx) * ty
end

-- 点 (lx, ly)（画像の中心から）の明るさの勾配（1px あたり）。1 マス離れた 2 点の差で求める
function M.lum_grad(Mk, lx, ly)
  local cs = Mk.cs
  local fx, fy = (lx + Mk.w / 2) / cs - 0.5, (ly + Mk.h / 2) / cs - 0.5
  local gx = (lum_at(Mk, fx + 1, fy) - lum_at(Mk, fx - 1, fy)) / (2 * cs)
  local gy = (lum_at(Mk, fx, fy + 1) - lum_at(Mk, fx, fy - 1)) / (2 * cs)
  return gx, gy
end

-- 他レイヤーの画像の点 (mx, my)（画像の中心から）を、本体から見た座標にする。
-- X..cy は at(t) の戻り値（位置・Z軸回転・拡大率・中心）。画面の点 = 位置 + 回転(拡大率 × (点 − 中心))
local function layer_point(X, Y, Z, rz, sx, sy, cx, cy, mx, my)
  local c, s = cos(rz * RAD), sin(rz * RAD)
  local ux, uy = (mx - cx) * sx, (my - cy) * sy
  return X + ux * c - uy * s, Y + ux * s + uy * c, Z
end
M.layer_point = layer_point

local function layer_unpoint(X, Y, rz, sx, sy, cx, cy, px, py)
  local c, s = cos(rz * RAD), sin(rz * RAD)
  local ux, uy = px - X, py - Y
  return (ux * c + uy * s) / (sx ~= 0 and sx or 1e-9) + cx, (-ux * s + uy * c) / (sy ~= 0 and sy or 1e-9) + cy
end
M.layer_unpoint = layer_unpoint

----------------------------------------------------------------------------- 自作関数（出力位置の xyz / xyzd、挙動の vector、カスタム描画の particle_obj）

local func_cache = {}

local function text_hash(s)
  local h = 0x2545F491
  for i = 1, #s do h = mix(bit.bxor(h, s:byte(i) + i * 256)) end
  return #s .. ":" .. bit.tohex(h)
end

-- ファイルの Lua を読み、定義された関数の入った表（環境）を返す。中身が変わったときだけ読み直す。
-- 関数の中からは obj・math などの全体の値が見える。戻り値: 環境（失敗したら nil）、エラーの文字列、中身の目印
function M.load_func(path)
  if not path or path == "" then return nil, nil, "" end
  local text = M.read_text(path)
  if not text then return nil, "読めない: " .. tostring(path), "" end
  local c = func_cache[path]
  if c and c.text == text then return c.env, c.err, c.hash end
  local env = setmetatable({}, { __index = _G })
  local name = path:match("[^/" .. string.char(92) .. "]+$") or path
  local chunk, err = loadstring(text, "=" .. name)
  if chunk then
    setfenv(chunk, env)
    local ok, e2 = pcall(chunk)
    if not ok then chunk, err = nil, tostring(e2) end
  end
  c = { text = text, env = chunk and env or nil, err = err, hash = text_hash(text) }
  func_cache[path] = c
  return c.env, c.err, c.hash
end

-- 自作関数を呼ぶ。エラーなら holder.err に最初のエラーを残して nil を返す
function M.call(holder, fn, ...)
  local r = { pcall(fn, ...) }
  if r[1] then return unpack(r, 2) end
  holder.err = holder.err or tostring(r[2])
  return nil
end

----------------------------------------------------------------------------- カーブエディタ_H のスロット（補間 = カーブエディタ_H）

-- 式の前置き（関数群）は @カーブエディタ_H.tra2 の _PRELUDE を、Live ファイルの場所は同じ tra2 の _bfile を読み取って使う。
-- CurveEditor_H の定義を写さないため。tra2 の書き方が変わって読めなければ nil とエラーを返す（呼び手は直線にする）。
-- スロットの切り出しは tra2 の _get_fn と同じ（@.live.N の節。無い・空なら return t）
local ce2_cache = {}

local function script_dir()
  local p = package.searchpath and package.searchpath("Particle_H", package.path or "")
  if p then return p:match("^(.*[/\\])") end
  -- require の検索先から分からなければ、tra2 自身と同じ既定のフォルダ（tra2 も Live の場所を C:\ProgramData\aviutl2 で持つ）
  local d = "C:\\ProgramData\\aviutl2\\Script\\"
  if M.read_text(d .. "Particle_H.lua") then return d end
  return nil
end

-- 戻り値: 関数（0..1 → 割合）か nil、エラーの文字列、中身の目印。opt.tra2 / opt.live で読むファイルを替えられる（検証器用）
function M.ce2_slot(slot, opt)
  opt = opt or {}
  slot = floor(tonumber(slot) or 1)
  local BS = string.char(92)
  local tra2 = opt.tra2
  if not tra2 then
    local d = script_dir()
    if not d then return nil, "スクリプトのフォルダが分からない", "" end
    tra2 = d .. "CurveEditor_H" .. BS .. "@カーブエディタ_H.tra2"
  end
  local ttext = M.read_text(tra2)
  if not ttext then return nil, "カーブエディタ_H の tra2 が無い: " .. tra2, "" end
  ttext = ttext:gsub("\r\n", "\n")
  local prelude = ttext:match("local _PRELUDE = %[==%[\n(.-\n)%]==%]")
  -- Live ファイルの場所: v1.10 までは `local _bfile = "…"`、それより後は `local _bfile = _C.live_path` の後の
  -- 既定の場所 `_bfile = "…"`（同じファイル）。どちらも最初の `_bfile = "…"` で読める
  local live = opt.live or ttext:match('_bfile = "([^"]*)"')
  if not prelude or not live then return nil, "tra2 から式の前置きか Live ファイルの場所を読めない", "" end
  live = (live:gsub(BS .. BS, BS))
  local ltext = (M.read_text(live) or ""):gsub("\r\n", "\n")
  local c = ce2_cache
  if c.prelude ~= prelude or c.live ~= ltext then
    c = { prelude = prelude, live = ltext, fns = {}, errs = {}, hash = text_hash(prelude .. ltext) }
    ce2_cache = c
  end
  local fn = c.fns[slot]
  if fn == nil then
    local code = "return t"
    local mk = "@.live." .. tostring(slot) .. "\n"
    local cs
    if ltext:sub(1, #mk) == mk then
      cs = #mk + 1
    else
      local si = ltext:find("\n" .. mk, 1, true)
      if si then cs = si + 1 + #mk end
    end
    if cs then
      local ns = ltext:find("\n@", cs, true)
      local s = ns and ltext:sub(cs, ns - 1) or ltext:sub(cs)
      if s:match("%S") then code = s end
    end
    code = code:gsub("%s+$", "")
    local chunk, err = loadstring(prelude .. code .. "\nend", "=CurveEditor_H")
    fn = false
    if chunk then
      local ok, made = pcall(chunk)
      if ok and type(made) == "function" then fn = made else c.errs[slot] = tostring(made) end
    else
      c.errs[slot] = "構文エラー: " .. tostring(err)
    end
    c.fns[slot] = fn
  end
  if not fn then return nil, "スロット " .. slot .. ": " .. tostring(c.errs[slot]), c.hash end
  local f = fn
  return function(x) return f(x, obj, 0, 1, 0, 2, slot) end, nil, c.hash
end

----------------------------------------------------------------------------- 自分の画像を並べる・文字の並び（v0.7.0）

-- 画像（RGBA 32bit。obj.getpixeldata の並び）を間隔 cell の格子に分け、アルファのあるマスを左上から並べる。
-- マスの中を最大 4x4 点で見て、どれかの点のアルファが 0 より大きければ並べる。
-- 戻り値の x, y はマスの中心（画像の左上から px）、cw, ch はマスの大きさ（右端・下端は画像の端まで）
function M.image_cells(data, w, h, cell)
  local p = ffi.cast("const uint8_t*", data)
  cell = max(floor(cell or 20), 1)
  local gw, gh = ceil(w / cell), ceil(h / cell)
  local C = { w = w, h = h, cell = cell, x = {}, y = {}, cw = {}, ch = {}, col = {}, n = 0 }
  local st = max(1, floor(cell / 4))
  for gy = 0, gh - 1 do
    local y0, y1 = gy * cell, min(gy * cell + cell, h)
    for gx = 0, gw - 1 do
      local x0, x1 = gx * cell, min(gx * cell + cell, w)
      local hit = false
      for y = y0, y1 - 1, st do
        local row = y * w
        for x = x0, x1 - 1, st do
          if p[(row + x) * 4 + 3] > 0 then hit = true; break end
        end
        if hit then break end
      end
      if hit then
        local n = C.n + 1
        C.n = n
        C.x[n], C.y[n], C.cw[n], C.ch[n] = (x0 + x1) / 2, (y0 + y1) / 2, x1 - x0, y1 - y0
        C.col[n] = M.pixel_avg(p, w, h, x0, y0, x1, y1)
      end
    end
  end
  return C
end

-- 四角 [x0, x1) × [y0, y1) の色の平均（アルファで重み付け。最大 4x4 点。0xRRGGBB。透明なら 0）
function M.pixel_avg(p, w, h, x0, y0, x1, y1)
  local stx, sty = max(1, floor((x1 - x0) / 4)), max(1, floor((y1 - y0) / 4))
  local sr, sg, sb, sa = 0, 0, 0, 0
  for y = max(floor(y0), 0), min(ceil(y1), h) - 1, sty do
    local row = y * w
    for x = max(floor(x0), 0), min(ceil(x1), w) - 1, stx do
      local o = (row + x) * 4
      local al = p[o + 3]
      sr, sg, sb, sa = sr + p[o] * al, sg + p[o + 1] * al, sb + p[o + 2] * al, sa + al
    end
  end
  if sa <= 0 then return 0 end
  return floor(sr / sa + 0.5) * 65536 + floor(sg / sa + 0.5) * 256 + floor(sb / sa + 0.5)
end

-- 多角形（{ {x, y}, ... }）を半平面 (p − m)・n ≤ 0 で切る（サザーランド・ホッジマン）
local function clip_half(poly, mx, my, nx, ny)
  local out = {}
  local np = #poly
  for i = 1, np do
    local a, b = poly[i], poly[i % np + 1]
    local da = (a[1] - mx) * nx + (a[2] - my) * ny
    local db = (b[1] - mx) * nx + (b[2] - my) * ny
    if da <= 0 then out[#out + 1] = a end
    if (da < 0 and db > 0) or (da > 0 and db < 0) then
      local t = da / (da - db)
      out[#out + 1] = { a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t }
    end
  end
  return out
end
M.clip_half = clip_half

-- 破片（v0.9.0）: 画像 w×h を、およそ n 個のボロノイの領域に割る。種は格子の点を ±rand/2 マスずらす（鍵 sk で決まる）。
-- 領域は画像の四角を、近くの種（±3 マス）との垂直二等分線で切って求める。形は画像の大きさ・数・不ぞろい・鍵が同じなら取っておく
local shard_cache = {}
function M.voronoi(w, h, n, rand, sk)
  local key = table.concat({ w, h, n, rand, sk }, "|")
  local V = shard_cache[key]
  if V then return V end
  n = max(floor(n), 2)
  local nx = max(floor(sqrt(n * w / h) + 0.5), 1)
  local ny = max(floor(n / nx + 0.5), 1)
  local cw, ch = w / nx, h / ny
  local sx, sy = {}, {}
  for j = 0, ny - 1 do
    for i = 0, nx - 1 do
      local id = j * nx + i
      sx[id] = (i + 0.5 + (rnd(id, CH.shard, sk) - 0.5) * rand) * cw
      sy[id] = (j + 0.5 + (rnd(id, CH.shard + 1, sk) - 0.5) * rand) * ch
    end
  end
  V = { w = w, h = h, n = nx * ny, poly = {}, cx = {}, cy = {}, area = {} }
  for j = 0, ny - 1 do
    for i = 0, nx - 1 do
      local id = j * nx + i
      local poly = { { 0, 0 }, { w, 0 }, { w, h }, { 0, h } }
      local px, py = sx[id], sy[id]
      for jj = max(j - 3, 0), min(j + 3, ny - 1) do
        for ii = max(i - 3, 0), min(i + 3, nx - 1) do
          local o = jj * nx + ii
          if o ~= id and #poly > 0 then
            local qx, qy = sx[o], sy[o]
            poly = clip_half(poly, (px + qx) / 2, (py + qy) / 2, qx - px, qy - py)
          end
        end
      end
      -- 重心と面積
      local A, Cx, Cy = 0, 0, 0
      for t = 1, #poly do
        local a, b = poly[t], poly[t % #poly + 1]
        local cr = a[1] * b[2] - b[1] * a[2]
        A, Cx, Cy = A + cr, Cx + (a[1] + b[1]) * cr, Cy + (a[2] + b[2]) * cr
      end
      A = A / 2
      local k = id + 1
      if abs(A) > 1e-9 then Cx, Cy = Cx / (6 * A), Cy / (6 * A) else Cx, Cy = px, py end
      V.poly[k], V.cx[k], V.cy[k], V.area[k] = poly, Cx, Cy, abs(A)
    end
  end
  shard_cache[key] = V
  return V
end

-- 自分の画像を破片に割る: アルファのある破片だけを、重心を中心にした多角形（px）で並べる
function M.image_shards(data, w, h, n, rand, sk)
  local p = ffi.cast("const uint8_t*", data)
  local V = M.voronoi(w, h, n, rand, sk)
  local C = { w = w, h = h, x = {}, y = {}, cw = {}, ch = {}, col = {}, poly = {}, n = 0, shards = true }
  for k = 1, V.n do
    local poly = V.poly[k]
    if #poly >= 3 then
      local cx, cy = V.cx[k], V.cy[k]
      -- 重心と、頂点を重心へ半分寄せた点のどれかにアルファがあれば並べる
      local hit = false
      local function at(x, y)
        local xi, yi = min(max(floor(x), 0), w - 1), min(max(floor(y), 0), h - 1)
        return p[(yi * w + xi) * 4 + 3] > 0
      end
      if at(cx, cy) then hit = true end
      for t = 1, #poly do
        if hit then break end
        if at((poly[t][1] + cx) / 2, (poly[t][2] + cy) / 2) then hit = true end
      end
      if hit then
        local m = C.n + 1
        C.n = m
        local rel = {}
        local x0, y0, x1, y1 = huge, huge, -huge, -huge
        for t = 1, #poly do
          rel[t] = { poly[t][1] - cx, poly[t][2] - cy }
          x0, y0, x1, y1 = min(x0, poly[t][1]), min(y0, poly[t][2]), max(x1, poly[t][1]), max(y1, poly[t][2])
        end
        C.x[m], C.y[m], C.cw[m], C.ch[m], C.poly[m] = cx, cy, 1, 1, rel
        C.col[m] = M.pixel_avg(p, w, h, x0, y0, x1, y1)
      end
    end
  end
  return C
end

-- 素材のテキストの、粒子 k の文字の番号（0 から）。並び 0 入力順 / 1 逆順 / 2 ランダム（素材の鍵と同じ割り当て）
function M.text_index(Mt, k, sk)
  local nu = Mt.count
  if Mt.order == 0 then return k % nu end
  if Mt.order == 1 then return nu - 1 - (k % nu) end
  return min(floor(rnd(k, CH.mat, sk) * nu), nu - 1)
end

-- テキストを、テキストオブジェクトのように並べる。粒子に使う文字（split_units と同じ順）の中心の位置を返す。
-- measure(s) → 幅, 高さ（px。文字列をまとめて測る）。一行なら 1 行を 1 つとして並べる。
-- opt: size 文字の大きさ / space 文字の間隔 / lspace 行の間隔 / align 0 左 / 1 中央 / 2 右。全体の中心が原点
function M.text_layout(text, by_line, measure, opt)
  opt = opt or {}
  local size = opt.size or 40
  local space, lspace, align = opt.space or 0, opt.lspace or 0, opt.align or 1
  text = (text or ""):gsub("\r\n", "\n")
  local lines, nu = {}, 0
  local cur = { items = {}, w = 0, h = 0 }
  local function add(u, w, h)
    if #cur.items > 0 then cur.w = cur.w + space end
    cur.items[#cur.items + 1] = { u = u, x = cur.w, w = w }
    cur.w = cur.w + w
    if h > cur.h then cur.h = h end
  end
  local function newline()
    if cur.h <= 0 then cur.h = size end
    lines[#lines + 1] = cur
    cur = { items = {}, w = 0, h = 0 }
  end
  if by_line then
    for line in (text .. "\n"):gmatch("(.-)\n") do
      if line:find("%S") then
        nu = nu + 1
        local w, h = measure(line)
        add(nu, w or 0, h or size)
      end
      newline()
    end
    while #lines > 1 and #lines[#lines].items == 0 do lines[#lines] = nil end
  else
    -- 1 文字の位置は「行頭からその文字までをまとめて測った幅 − その文字の幅」（1 文字ずつ足すと字間の分だけ詰まる。実機 EPR-178 で 8 文字 8px）
    local line, recs, nch = "", {}, 0
    local function flush()
      local L = { items = {}, w = 0, h = 0 }
      if line ~= "" then
        L.w = (measure(line) or 0) + space * max(nch - 1, 0)
        for _, r in ipairs(recs) do
          local wp = measure(r.prefix) or 0
          local wc, hc = measure(r.ch)
          wc, hc = wc or 0, hc or size
          L.items[#L.items + 1] = { u = r.u, x = wp - wc + space * (r.pos - 1), w = wc }
          if hc > L.h then L.h = hc end
        end
      end
      if L.h <= 0 then L.h = size end
      lines[#lines + 1] = L
      line, recs, nch = "", {}, 0
    end
    for ch in text:gmatch("[%z\1-\127\194-\244][\128-\191]*") do
      if ch == "\n" then
        flush()
      else
        line = line .. ch
        nch = nch + 1
        if ch:find("%S") then
          nu = nu + 1
          recs[#recs + 1] = { u = nu, prefix = line, ch = ch, pos = nch }
        end
      end
    end
    if line ~= "" or #lines == 0 then flush() end
  end
  local W, H = 0, 0
  for i, L in ipairs(lines) do
    if L.w > W then W = L.w end
    H = H + L.h + (i > 1 and lspace or 0)
  end
  local T = { n = nu, x = {}, y = {}, w = W, h = H }
  local top = -H / 2
  for _, L in ipairs(lines) do
    local x0 = align == 0 and -W / 2 or (align == 2 and W / 2 - L.w or -L.w / 2)
    for _, it in ipairs(L.items) do
      if it.u then T.x[it.u], T.y[it.u] = x0 + it.x + it.w / 2, top + L.h / 2 end
    end
    top = top + L.h + lspace
  end
  return T
end

-- 色の並び（0xRRGGBB）と重みから、メディアンカットで n 色を選ぶ（いちばん幅の広い色の軸の重み付きの真ん中で分ける）
function M.median_cut(cols, wts, n)
  local items = {}
  for i = 1, #cols do
    local c = cols[i]
    items[#items + 1] = { bit.band(bit.rshift(c, 16), 255), bit.band(bit.rshift(c, 8), 255), bit.band(c, 255), wts and wts[i] or 1 }
  end
  if #items == 0 then return { 0xffffff } end
  local boxes = { items }
  while #boxes < n do
    local bi, bax, brange = nil, 1, -1
    for i, bx in ipairs(boxes) do
      if #bx > 1 then
        for ax = 1, 3 do
          local lo, hi = 255, 0
          for _, it in ipairs(bx) do lo, hi = min(lo, it[ax]), max(hi, it[ax]) end
          if hi - lo > brange then bi, bax, brange = i, ax, hi - lo end
        end
      end
    end
    if not bi or brange <= 0 then break end
    local bx = boxes[bi]
    table.sort(bx, function(a, b) if a[bax] ~= b[bax] then return a[bax] < b[bax] end return a[4] < b[4] end)
    -- 値の変わり目のうち、重みの累積が半分にいちばん近い所で分ける（同じ値の並びの途中では分けない）
    local tot = 0
    for _, it in ipairs(bx) do tot = tot + it[4] end
    local acc, cut, best = 0, 1, huge
    for i2 = 1, #bx - 1 do
      acc = acc + bx[i2][4]
      if bx[i2][bax] ~= bx[i2 + 1][bax] then
        local d = abs(acc - tot / 2)
        if d < best then cut, best = i2, d end
      end
    end
    local a, b = {}, {}
    for i2, it in ipairs(bx) do if i2 <= cut then a[#a + 1] = it else b[#b + 1] = it end end
    boxes[bi] = a
    boxes[#boxes + 1] = b
  end
  local cen = {}
  for _, bx in ipairs(boxes) do
    local r, g, bl, wsum = 0, 0, 0, 0
    for _, it in ipairs(bx) do r, g, bl, wsum = r + it[1] * it[4], g + it[2] * it[4], bl + it[3] * it[4], wsum + it[4] end
    if wsum <= 0 then wsum = 1 end
    cen[#cen + 1] = { r / wsum, g / wsum, bl / wsum }
  end
  -- 4 回だけ寄せ直す（各色をいちばん近い代表へ配り、重み付きの平均を取り直す。中央で分けると塊が割れることがある）
  for _ = 1, 4 do
    local acc = {}
    for j = 1, #cen do acc[j] = { 0, 0, 0, 0 } end
    for _, it in ipairs(items) do
      local bj, bd = 1, huge
      for j, c in ipairs(cen) do
        local dd = (c[1] - it[1]) ^ 2 + (c[2] - it[2]) ^ 2 + (c[3] - it[3]) ^ 2
        if dd < bd then bj, bd = j, dd end
      end
      local a = acc[bj]
      a[1], a[2], a[3], a[4] = a[1] + it[1] * it[4], a[2] + it[2] * it[4], a[3] + it[3] * it[4], a[4] + it[4]
    end
    for j, a in ipairs(acc) do
      if a[4] > 0 then cen[j] = { a[1] / a[4], a[2] / a[4], a[3] / a[4] } end
    end
  end
  local pal = {}
  for j, c in ipairs(cen) do pal[j] = floor(c[1] + 0.5) * 65536 + floor(c[2] + 0.5) * 256 + floor(c[3] + 0.5) end
  return pal
end

local function nearest_pal(pal, c)
  local r, g, b = bit.band(bit.rshift(c, 16), 255), bit.band(bit.rshift(c, 8), 255), bit.band(c, 255)
  local best, bd = 0, huge
  for i = 1, #pal do
    local q = pal[i]
    local dr, dg, db = bit.band(bit.rshift(q, 16), 255) - r, bit.band(bit.rshift(q, 8), 255) - g, bit.band(q, 255) - b
    local d = dr * dr + dg * dg + db * db
    if d < bd then best, bd = i - 1, d end
  end
  return best
end
M.nearest_pal = nearest_pal

----------------------------------------------------------------------------- 出力位置

local function poly_len(pts, n)
  local acc, total = { 0 }, 0
  for i = 1, n - 1 do
    local a, b = (i - 1) * 3, i * 3
    local dx, dy, dz = pts[b + 1] - pts[a + 1], pts[b + 2] - pts[a + 2], pts[b + 3] - pts[a + 3]
    total = total + sqrt(dx * dx + dy * dy + dz * dz)
    acc[i + 1] = total
  end
  return acc, total
end

-- 折れ線の上の、長さの割合 u（0..1）の点
local function poly_at(pts, n, acc, total, u)
  if n <= 1 or total <= 0 then return pts[1] or 0, pts[2] or 0, pts[3] or 0 end
  local d = u * total
  for i = 1, n - 1 do
    if d <= acc[i + 1] or i == n - 1 then
      local seg = acc[i + 1] - acc[i]
      local f = seg > 0 and (d - acc[i]) / seg or 0
      f = min(max(f, 0), 1)
      local a, b = (i - 1) * 3, i * 3
      return pts[a + 1] + (pts[b + 1] - pts[a + 1]) * f,
             pts[a + 2] + (pts[b + 2] - pts[a + 2]) * f,
             pts[a + 3] + (pts[b + 3] - pts[a + 3]) * f
    end
  end
  return pts[(n - 1) * 3 + 1], pts[(n - 1) * 3 + 2], pts[(n - 1) * 3 + 3]
end

-- 出力位置（emit が nil なら原点）。key は乱数の番号（粒子または組）、b は誕生時刻。
-- 戻り値: x, y, z[, 出力方向, Z出力方向, 足す速度 x, y, z]（他レイヤー・関数のとき）。出せないときは nil
local function emit_pos(emit, key, b, sk)
  if not emit or emit.shape == 0 then return 0, 0, 0 end
  local pts, n = emit.pts, emit.n
  local r1, r2, r3 = rnd(key, CH.pos1, sk), rnd(key, CH.pos2, sk), rnd(key, CH.pos3, sk)
  if emit.shape == 1 then                                   -- 線
    if emit.from_points then
      local i = min(floor(r1 * n), n - 1)
      return pts[i * 3 + 1], pts[i * 3 + 2], pts[i * 3 + 3]
    end
    return poly_at(pts, n, emit.acc, emit.total, r1)
  elseif emit.shape == 2 then                               -- 線を走る
    local T = max(emit.run_time, 0.001)
    local u = b / T
    if emit.loop then u = u - floor(u) else u = min(max(u, 0), 1) end
    return poly_at(pts, n, emit.acc, emit.total, u)
  elseif emit.shape == 3 then                               -- 箱（アンカー 2 点が対角）
    local x0, y0, z0, x1, y1, z1 = pts[1], pts[2], pts[3], pts[4], pts[5], pts[6]
    local cx, cy, cz = (x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2
    local hx, hy, hz = abs(x1 - x0) / 2, abs(y1 - y0) / 2, abs(z1 - z0) / 2
    local hole = min(max(emit.hollow / 100, 0), 0.999)
    local u, v, w = r1 * 2 - 1, r2 * 2 - 1, r3 * 2 - 1
    if hole > 0 then
      -- 内側の箱を除く: 一番外に近い軸を [hole, 1] へ押し出す（決定的。分布は一様ではない）
      local m = max(abs(u), abs(v), abs(w))
      if m < hole then
        local s = (hole + (1 - hole) * rnd(key, CH.pos4, sk)) / max(m, 1e-9)
        u, v, w = u * s, v * s, w * s
      end
    end
    return cx + u * hx, cy + v * hy, cz + w * hz
  elseif emit.shape == 4 then                               -- 球（1 点目が中心、2 点目までの距離が半径）
    local cx, cy, cz = pts[1], pts[2], pts[3]
    local dx, dy, dz = pts[4] - cx, pts[5] - cy, pts[6] - cz
    local R = sqrt(dx * dx + dy * dy + dz * dz)
    local hole = min(max(emit.hollow / 100, 0), 0.999)
    local zz = r1 * 2 - 1
    local phi = r2 * 2 * pi
    local rr = sqrt(max(1 - zz * zz, 0))
    local h3 = hole * hole * hole
    local rad = R * (h3 + (1 - h3) * r3) ^ (1 / 3)
    return cx + rad * rr * cos(phi), cy + rad * rr * sin(phi), cz + rad * zz
  elseif emit.shape == 5 or emit.shape == 6 then            -- 他レイヤーを追う / 他レイヤーの形
    -- lay(b): 誕生時刻 b の対象の位置・Z軸回転・拡大率・中心（本体から見た座標）。対象が無ければ nil
    local L = emit.lay
    if not L then return nil end
    -- 追う位置（v0.14.0。原作の出力タイプ 1 のオプ2）: 1 = 0〜生まれた時刻、2 = オブジェクト全体の、でたらめな時刻の位置
    local tq = b
    if emit.shape == 5 and (emit.trace or 0) ~= 0 then
      local lo = emit.t0 or 0
      local hi = emit.trace == 1 and b or (emit.t1 or b)
      if hi < lo then hi = lo end
      tq = lo + (hi - lo) * rnd(key, CH.trace, sk)
    end
    local X, Y, Z, rz, sx, sy, cx, cy = L(tq)
    if X == nil then return nil end
    local x, y, z = X, Y, Z
    if emit.shape == 6 then
      local mx, my, mc = M.mask_point(emit.mask, r1, r2, r3, emit.pick)
      if not mx then return nil end
      x, y, z = layer_point(X, Y, Z, rz, sx, sy, cx, cy, mx, my)
      emit.last_cell = mc
    end
    if not emit.use_dir and (emit.add_vel or 0) == 0 then return x, y, z end
    -- 対象の動き（前後の時刻の差）。動く向きに出す・動きの速さを足す
    local dt = emit.dt or (1 / 30)
    local b0 = max(tq - dt, 0)
    local X0, Y0, Z0 = L(b0)
    local X1, Y1, Z1 = L(tq + dt)
    if X0 == nil or X1 == nil then return x, y, z end
    local span = tq + dt - b0
    local vx, vy, vz = (X1 - X0) / span, (Y1 - Y0) / span, (Z1 - Z0) / span
    local sp = sqrt(vx * vx + vy * vy + vz * vz)
    local odir, ozd
    if emit.use_dir and sp > 1e-9 then
      odir = atan2(vx, vy) / RAD
      ozd = math.asin(max(min(vz / sp, 1), -1)) / RAD
    end
    local f = (emit.add_vel or 0) / 100
    return x, y, z, odir, ozd, vx * f, vy * f, vz * f
  elseif emit.shape == 8 then                               -- 自分の画像を並べる（key 番目のマスの中心）
    local C = emit.cells
    if not C or C.n <= 0 then return nil end
    local i = key % C.n + 1
    return C.x[i] - C.w / 2, C.y[i] - C.h / 2, 0
  elseif emit.shape == 10 then                              -- 渡された粒子（誕生時刻に渡されていた粒子の 1 つの位置）
    local A = emit.shared and emit.shared(b)
    if not A or A.n <= 0 then return nil end
    local i = min(floor(r1 * A.n), A.n - 1) + 1
    local x, y, z = A.x[i] - emit.sbx, A.y[i] - emit.sby, A.z[i] - emit.sbz
    if not emit.use_dir and (emit.add_vel or 0) == 0 then return x, y, z end
    local vx, vy = A.vx[i], A.vy[i]
    local odir
    if emit.use_dir and vx * vx + vy * vy > 1e-18 then odir = atan2(vx, vy) / RAD end
    local f = (emit.add_vel or 0) / 100
    return x, y, z, odir, nil, vx * f, vy * f, 0
  elseif emit.shape == 9 then                               -- 文字の並び（素材のテキストの、粒子の文字の位置）
    local T, Mt = emit.tlay, emit.tmat
    if not T or not Mt or (Mt.count or 0) <= 0 then return nil end
    local u = M.text_index(Mt, key, sk) + 1
    if not T.x[u] then return nil end
    return T.x[u], T.y[u], 0
  else                                                      -- 関数: xyzd(t) か xyz(t)（t はオブジェクトの時刻。角度はラジアン）
    local F = emit.fn
    if not F then return 0, 0, 0 end
    local tb = emit.obj_time and emit.obj_time(b) or b
    if F.xyzd then
      local ok, x, y, z, dxy, dz = pcall(F.xyzd, tb)
      if not ok then
        if emit.errs then emit.errs.emit = emit.errs.emit or tostring(x) end
        return 0, 0, 0
      end
      dxy, dz = tonumber(dxy), tonumber(dz)
      return tonumber(x) or 0, tonumber(y) or 0, tonumber(z) or 0, dxy and dxy / RAD, dz and dz / RAD
    end
    local ok, x, y, z = pcall(F.xyz, tb)
    if not ok then
      if emit.errs then emit.errs.emit = emit.errs.emit or tostring(x) end
      return 0, 0, 0
    end
    return tonumber(x) or 0, tonumber(y) or 0, tonumber(z) or 0
  end
end
M.emit_pos = emit_pos

-- 出力位置の設定を整える（折れ線の長さを先に測る）。rt: 本体が用意した、時刻と画像に依る値（他レイヤー・関数）
function M.prepare_emit(emit, rt, cfg)
  if not emit then return nil end
  if emit.shape >= 5 then
    -- 拡張の表は積分の目印に使うので書き換えない（写しに足す）
    local E = {}
    for k2, v in pairs(emit) do E[k2] = v end
    rt = rt or {}
    E.lay, E.mask, E.fn = rt.lay, rt.emask, rt.efn
    E.cells, E.tlay, E.tmat = rt.cells, rt.tlayout, rt.tmat
    E.shared = rt.shared
    E.sbx, E.sby, E.sbz = 0, 0, 0
    if rt.shared_b then E.sbx, E.sby, E.sbz = rt.shared_b[1], rt.shared_b[2], rt.shared_b[3] end
    if E.mask and ((E.oarea or 0) ~= 0 or (E.oweight or 0) ~= 0) then E.pick = M.mask_pick(E.mask, E.oarea, E.oweight) end
    E.errs = cfg and cfg.errs
    -- 関数の t はオブジェクトの時刻（開始時間の分はマイナスになる。原作の説明書 19 の t0 と同じ）
    E.obj_time = cfg and (cfg.obj_time_raw or cfg.obj_time)
    E.t0, E.t1 = cfg and cfg.t_start or 0, cfg and cfg.t_end
    E.dt = 1 / ((cfg and cfg.fps) or 30)
    return E
  end
  local need = (emit.shape == 3 or emit.shape == 4) and 2 or max(emit.n or 0, 1)
  local pts = emit.pts or {}
  if #pts < need * 3 then return nil end
  emit.n = need
  if emit.shape == 1 or emit.shape == 2 then
    emit.acc, emit.total = poly_len(pts, need)
  end
  return emit
end

----------------------------------------------------------------------------- 回転

-- 1 軸ぶんの角度（度）。rot は「回転」セクションの設定（無ければ nil）
local function axis_angle(init, spin, age, b, axis, rot, key, sk, obj_birth)
  local a
  if rot then
    local acc = rot.acc[axis]
    local L = rot.limit[axis]
    local ta = rot.acc_time
    if rot.indiv_acc ~= 0 then acc = vary(acc, rot.indiv_acc, rnd(key, CH.racc + axis * 100, sk)) end
    if rot.indiv_time ~= 0 then ta = vary(ta, rot.indiv_time, rnd(key, CH.rtime, sk)) end
    if not rot.relative then ta = ta - obj_birth end        -- オブジェクトの時刻で数える
    ta = max(ta, 0)
    local sgn = spin < 0 and -1 or 1
    local m0 = abs(spin)
    local hi = (L and L > 0) and L or huge
    local lo = rot.no_reverse and 0 or -hi
    if acc == 0 or age <= ta then
      a = spin * age
    else
      a = spin * ta + sgn * ramp_integral(m0, acc, lo, hi, age - ta)
    end
    a = a + rot.inc[axis] * obj_birth
    local amp = rot.swing[axis]
    if amp ~= 0 then
      if rot.indiv_amp ~= 0 then amp = vary(amp, rot.indiv_amp, rnd(key, CH.swing_amp + axis * 100, sk)) end
      local spd = rot.swing_speed
      if rot.indiv_speed ~= 0 then spd = vary(spd, rot.indiv_speed, rnd(key, CH.swing_speed, sk)) end
      local ph0 = rot.phase_random and rnd(key, CH.swing_phase + axis * 100, sk) * 360 or 0
      local t = rot.sync and (b + age) or age
      a = a + amp * sin((spd * t + ph0) * RAD)
    end
  else
    a = spin * age
  end
  return init + a
end

----------------------------------------------------------------------------- 時間の窓（時間の付け替え）

-- win = {始, 終, 始, 終, ...}（秒。終が負か無ければ終わりなし）
local function win_end(win, i)
  local e = win[i + 1]
  if e == nil or e < 0 then return huge end
  return e
end

-- [a0, a1] と窓が重なる長さ
local function window_measure(win, a0, a1)
  if a1 <= a0 then return 0 end
  local total = 0
  for i = 1, #win, 2 do
    local lo, hi = max(a0, win[i]), min(a1, win_end(win, i))
    if hi > lo then total = total + (hi - lo) end
  end
  return total
end
M.window_measure = window_measure

local function in_window(win, t)
  for i = 1, #win, 2 do
    if t >= win[i] and t < win_end(win, i) then return true end
  end
  return false
end
M.in_window = in_window

----------------------------------------------------------------------------- ノイズ場

-- 格子点の値（-1..1）。整数の座標と種から決まる
local function h3(ix, iy, iz, s)
  local x = bit.bxor(bit.tobit(ix * 73856093), bit.tobit(iy * 19349663), bit.tobit(iz * 83492791), s)
  x = mix(bit.tobit(x + 0x27D4EB2D))
  return (x % 65536) / 32768 - 1
end

-- 3 次元の値ノイズと、x・y の偏微分（なめらかさは 3t^2 - 2t^3）
local function noise3(x, y, z, s)
  local ix, iy, iz = floor(x), floor(y), floor(z)
  local fx, fy, fz = x - ix, y - iy, z - iz
  local ux, uy, uz = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy), fz * fz * (3 - 2 * fz)
  local dux, duy = 6 * fx * (1 - fx), 6 * fy * (1 - fy)
  local a, b = h3(ix, iy, iz, s), h3(ix + 1, iy, iz, s)
  local c, d = h3(ix, iy + 1, iz, s), h3(ix + 1, iy + 1, iz, s)
  local e, f = h3(ix, iy, iz + 1, s), h3(ix + 1, iy, iz + 1, s)
  local g, hh = h3(ix, iy + 1, iz + 1, s), h3(ix + 1, iy + 1, iz + 1, s)
  local k1, k2, k3 = b - a, c - a, e - a
  local k4, k5, k6 = a - b - c + d, a - c - e + g, a - b - e + f
  local k7 = -a + b + c - d + e - f - g + hh
  local n = a + k1 * ux + k2 * uy + k3 * uz + k4 * ux * uy + k5 * uy * uz + k6 * ux * uz + k7 * ux * uy * uz
  local nx = dux * (k1 + k4 * uy + k6 * uz + k7 * uy * uz)
  local ny = duy * (k2 + k4 * ux + k5 * uz + k7 * ux * uz)
  return n, nx, ny
end
M.noise3 = noise3

-- 勾配ノイズ（v0.19.0。Perlin 型）の格子点の勾配: 立方体の辺の中点への 12 方向（長さ √2）。番号は h3 と同じハッシュから取る
local GRAD12 = { 1, 1, 0, -1, 1, 0, 1, -1, 0, -1, -1, 0, 1, 0, 1, -1, 0, 1, 1, 0, -1, -1, 0, -1, 0, 1, 1, 0, -1, 1, 0, 1, -1, 0, -1, -1 }
local function g3(ix, iy, iz, s, fx, fy, fz)
  local x = bit.bxor(bit.tobit(ix * 73856093), bit.tobit(iy * 19349663), bit.tobit(iz * 83492791), s)
  x = mix(bit.tobit(x + 0x27D4EB2D))
  local o = ((x % 65536) % 12) * 3
  local gx, gy, gz = GRAD12[o + 1], GRAD12[o + 2], GRAD12[o + 3]
  return gx * fx + gy * fy + gz * fz, gx, gy
end
-- 3 次元の勾配ノイズと、x・y の偏微分（なめらかさは 6t^5 - 15t^4 + 10t^3）。値は格子点で 0、勾配は格子点でも 0 にならない。
-- 値ノイズと同じ強さで流れるよう、偏微分を GRAD_K 倍して返す（verify_noise_integ.py で、ランダムな点の勾配の二乗平均をそろえた値）
local GRAD_K = 0.946      -- 値ノイズ 0.941 / 勾配ノイズ 0.995（偏微分 x・y の二乗平均の平方根。20 万点）
local function noise3g(x, y, z, s)
  local ix, iy, iz = floor(x), floor(y), floor(z)
  local fx, fy, fz = x - ix, y - iy, z - iz
  local ux, uy, uz = fx * fx * fx * (fx * (fx * 6 - 15) + 10), fy * fy * fy * (fy * (fy * 6 - 15) + 10), fz * fz * fz * (fz * (fz * 6 - 15) + 10)
  local dux, duy = 30 * fx * fx * (fx - 1) * (fx - 1), 30 * fy * fy * (fy - 1) * (fy - 1)
  local a, ax, ay = g3(ix, iy, iz, s, fx, fy, fz)
  local b, bx, by = g3(ix + 1, iy, iz, s, fx - 1, fy, fz)
  local c, cx, cy = g3(ix, iy + 1, iz, s, fx, fy - 1, fz)
  local d, dx, dy = g3(ix + 1, iy + 1, iz, s, fx - 1, fy - 1, fz)
  local e, ex, ey = g3(ix, iy, iz + 1, s, fx, fy, fz - 1)
  local f, fx2, fy2 = g3(ix + 1, iy, iz + 1, s, fx - 1, fy, fz - 1)
  local g, gx, gy = g3(ix, iy + 1, iz + 1, s, fx, fy - 1, fz - 1)
  local hh, hx, hy = g3(ix + 1, iy + 1, iz + 1, s, fx - 1, fy - 1, fz - 1)
  local k1, k2, k3 = b - a, c - a, e - a
  local k4, k5, k6 = a - b - c + d, a - c - e + g, a - b - e + f
  local k7 = -a + b + c - d + e - f - g + hh
  local n = a + k1 * ux + k2 * uy + k3 * uz + k4 * ux * uy + k5 * uy * uz + k6 * ux * uz + k7 * ux * uy * uz
  -- 角の値そのものも位置で変わる（勾配との内積）ので、角の勾配を同じ重みで補間して足す
  local uxy, uyz, uxz, uxyz = ux * uy, uy * uz, ux * uz, ux * uy * uz
  local nx = dux * (k1 + k4 * uy + k6 * uz + k7 * uyz)
    + ax + (bx - ax) * ux + (cx - ax) * uy + (ex - ax) * uz + (ax - bx - cx + dx) * uxy + (ax - cx - ex + gx) * uyz
    + (ax - bx - ex + fx2) * uxz + (-ax + bx + cx - dx + ex - fx2 - gx + hx) * uxyz
  local ny = duy * (k2 + k4 * ux + k5 * uz + k7 * uxz)
    + ay + (by - ay) * ux + (cy - ay) * uy + (ey - ay) * uz + (ay - by - cy + dy) * uxy + (ay - cy - ey + gy) * uyz
    + (ay - by - ey + fy2) * uxz + (-ay + by + cy - dy + ey - fy2 - gy + hy) * uxyz
  return n, nx * GRAD_K, ny * GRAD_K
end
M.noise3g = noise3g

-- 元のノイズを値ノイズ以外にしたときのカール（v0.19.0。ノイズ場の「元のノイズ」）。N.kind: 1 = 勾配ノイズ / 2 = ブロック（補間なし）
-- ブロック: 升（大きさ = ノイズの大きさ）の中では力が一定。升の真ん中の値ノイズのカールを、縦か横の強い方の向きへそろえる。
-- 粒子は升の境目で直角に折れ、格子に沿って進む（発散は 0 ではない）
local function curl_alt(px, py, pz, t, N)
  local x, y, z = px * N.inv + N.ox, py * N.inv + N.oy, pz * N.inv + t * N.speed
  if N.kind == 1 then
    local _, gx_, gy_ = noise3g(x, y, z, N.seed)
    local gz_ = 0
    if N.z then gz_ = noise3g(x + 31.7, y - 17.3, z, N.seed2) end
    return gy_, -gx_, gz_
  end
  local cx_, cy_ = floor(x) + 0.5, floor(y) + 0.5
  local _, bx_, by_ = noise3(cx_, cy_, z, N.seed)
  local rx, ry = by_, -bx_
  if abs(rx) >= abs(ry) then ry = 0 else rx = 0 end
  local bz_ = 0
  if N.z then bz_ = noise3(cx_ + 31.7, cy_ - 17.3, z, N.seed2) end
  return rx, ry, bz_
end

-- カールノイズの力（XY）。ノイズの勾配を 90 度回したもので、発散が無い（渦を巻く流れになる）
-- 座標は端数 ox, oy だけずらす。値ノイズは格子点で勾配が 0 なので、ずらさないと原点（粒子の出る所）で力が 0 になる
function M.curl(px, py, pz, t, N)
  if N.kind then return curl_alt(px, py, pz, t, N) end
  local _, nx, ny = noise3(px * N.inv + N.ox, py * N.inv + N.oy, pz * N.inv + t * N.speed, N.seed)
  local fz = 0
  if N.z then fz = noise3(px * N.inv + N.ox + 31.7, py * N.inv + N.oy - 17.3, pz * N.inv + t * N.speed, N.seed2) end
  return ny, -nx, fz
end

----------------------------------------------------------------------------- 向きを変える

-- 単位ベクトルを、XY の角度で da 度回し、Z の仰角を de 度足す
local function turn(ux, uy, uz, da, de)
  if da ~= 0 then
    local c, s = cos(da * RAD), sin(da * RAD)
    ux, uy = ux * c - uy * s, ux * s + uy * c
  end
  if de ~= 0 then
    local hl = sqrt(ux * ux + uy * uy)
    local el = atan2(uz, hl) + de * RAD
    local ce = cos(el)
    if hl > 1e-12 then ux, uy = ux / hl * ce, uy / hl * ce else ux, uy = ce, 0 end
    uz = sin(el)
  end
  return ux, uy, uz
end
M.turn = turn

----------------------------------------------------------------------------- 跳ね返り

-- 面: 0=両面 / 1=小さい側だけ / 2=大きい側だけ / 3=なし
local function plane1(t, lo, hi, p, v, e)
  local hit = false
  if (t == 0 or t == 1) and p < lo then p = 2 * lo - p; if v < 0 then v = -v * e end; hit = true end
  if (t == 0 or t == 2) and p > hi then p = 2 * hi - p; if v > 0 then v = -v * e end; hit = true end
  return p, v, hit
end

-- 範囲の中だけの面（v0.14.0。原作の領域内反射のオフ）: ほかの 2 軸が範囲の中（ok）で、1 刻み前（p − v × h）は面の内側だったときだけ跳ね返す。
-- 範囲の外を通った粒子は、面の延長をすり抜ける（後で範囲に入っても引き戻さない）
local function plane1f(t, lo, hi, p, v, e, ok, h)
  if not ok then return p, v, false end
  local prev = p - v * h
  local hit = false
  if (t == 0 or t == 1) and p < lo and prev >= lo then p = 2 * lo - p; if v < 0 then v = -v * e end; hit = true end
  if (t == 0 or t == 2) and p > hi and prev <= hi then p = 2 * hi - p; if v > 0 then v = -v * e end; hit = true end
  return p, v, hit
end

-- 最後に当たった面の向き（粒子の側を向く単位の向き。XY だけ。転がる回転に使う。v0.15.0）
local bnx, bny = 0, -1

-- t = シミュレーション時刻、age = 粒子の年齢（面を変える時刻を粒子ごとに数えるとき）
local function bounce(B, px, py, pz, vx, vy, vz, t, age)
  local h1, h2, h3_, hs, hshape
  local bx, by, bz, sph = B.bx, B.by, B.bz, B.sph
  -- 面を変える時刻（v0.14.0。原作のタイプ変更時間）: その時刻より後は、変えた後の面（0 はそのまま）
  if (B.ctime or 0) > 0 then
    local tt
    if B.crel then tt = age else tt = t and (B.obj and B.obj(t) or t) end
    if tt and tt >= B.ctime then
      if B.cbx > 0 then bx = B.cbx - 1 end
      if B.cby > 0 then by = B.cby - 1 end
      if B.cbz > 0 then bz = B.cbz - 1 end
      if B.csph > 0 then sph = B.csph - 1 end
    end
  end
  if B.finite then
    local h = B.h or 0
    local inx, iny, inz = px >= B.xmin and px <= B.xmax, py >= B.ymin and py <= B.ymax, pz >= B.zmin and pz <= B.zmax
    px, vx, h1 = plane1f(bx, B.xmin, B.xmax, px, vx, B.ex, iny and inz, h)
    py, vy, h2 = plane1f(by, B.ymin, B.ymax, py, vy, B.ey, inx and inz, h)
    pz, vz, h3_ = plane1f(bz, B.zmin, B.zmax, pz, vz, B.ez, inx and iny, h)
  else
    px, vx, h1 = plane1(bx, B.xmin, B.xmax, px, vx, B.ex)
    py, vy, h2 = plane1(by, B.ymin, B.ymax, py, vy, B.ey)
    pz, vz, h3_ = plane1(bz, B.zmin, B.zmax, pz, vz, B.ez)
  end
  if h1 then bnx, bny = (abs(px - B.xmin) < abs(px - B.xmax)) and 1 or -1, 0 end
  if h2 then bnx, bny = 0, (abs(py - B.ymin) < abs(py - B.ymax)) and 1 or -1 end
  local kf = 1 - (B.fric or 0) / 100
  if kf < 1 then
    if h1 then vy, vz = vy * kf, vz * kf end
    if h2 then vx, vz = vx * kf, vz * kf end
    if h3_ then vx, vy = vx * kf, vy * kf end
  end
  if sph ~= 0 then
    local cx, cy, cz, R = B.cx, B.cy, B.cz, B.srad
    local dx, dy, dz = px - cx, py - cy, pz - cz
    local d = sqrt(dx * dx + dy * dy + dz * dz)
    if d > 1e-9 then
      local nx, ny, nz = dx / d, dy / d, dz / d
      local outside = sph == 1 or (sph == 3 and py <= cy)
      if outside and d < R then
        px, py, pz = cx + nx * R, cy + ny * R, cz + nz * R
        local vn = vx * nx + vy * ny + vz * nz
        if vn < 0 then
          local kk = (1 + B.se) * vn
          vx, vy, vz = vx - kk * nx, vy - kk * ny, vz - kk * nz
        end
        if kf < 1 then
          local v2 = vx * nx + vy * ny + vz * nz
          vx, vy, vz = nx * v2 + (vx - nx * v2) * kf, ny * v2 + (vy - ny * v2) * kf, nz * v2 + (vz - nz * v2) * kf
        end
        hs = true
        bnx, bny = nx, ny
      elseif sph == 2 and d > R then
        px, py, pz = cx + nx * R, cy + ny * R, cz + nz * R
        local vn = vx * nx + vy * ny + vz * nz
        if vn > 0 then
          local kk = (1 + B.se) * vn
          vx, vy, vz = vx - kk * nx, vy - kk * ny, vz - kk * nz
        end
        if kf < 1 then
          local v2 = vx * nx + vy * ny + vz * nz
          vx, vy, vz = nx * v2 + (vx - nx * v2) * kf, ny * v2 + (vy - ny * v2) * kf, nz * v2 + (vz - nz * v2) * kf
        end
        hs = true
        bnx, bny = -nx, -ny
      end
    end
  end
  local Sh = B.shape
  if Sh and t then
    -- 他レイヤーの形: 対象の画像の座標へ戻して、形の縁までの距離で当たりを見る（Z は見ない）
    local X, Y, _, rz, sx, sy, cx, cy = Sh.at(t)
    if X ~= nil and sx ~= 0 and sy ~= 0 then
      local c, s = cos(rz * RAD), sin(rz * RAD)
      local dx, dy = px - X, py - Y
      local lx, ly = (dx * c + dy * s) / sx + cx, (-dx * s + dy * c) / sy + cy
      local Mk = Sh.mask
      local far = not Sh.inv and (abs(lx) > Mk.w / 2 + Mk.cs or abs(ly) > Mk.h / 2 + Mk.cs)
      local d, gx, gy = 1, 0, 0
      if not far then d, gx, gy = M.sdf_at(Mk, lx, ly) end
      if Sh.inv then d, gx, gy = -d, -gx, -gy end
      local gl = sqrt(gx * gx + gy * gy)
      if d < 0 and gl > 1e-9 then
        gx, gy = gx / gl, gy / gl
        lx, ly = lx - gx * d, ly - gy * d
        local ux, uy = (lx - cx) * sx, (ly - cy) * sy
        px, py = X + ux * c - uy * s, Y + ux * s + uy * c
        -- 面の向き（画面）: 勾配に拡大率の逆数を掛けて回す
        local nx, ny = gx / sx, gy / sy
        nx, ny = nx * c - ny * s, nx * s + ny * c
        local nl = sqrt(nx * nx + ny * ny)
        nx, ny = nx / nl, ny / nl
        local vn = vx * nx + vy * ny
        if vn < 0 then
          local kk = (1 + Sh.e) * vn
          vx, vy = vx - kk * nx, vy - kk * ny
        end
        if kf < 1 then
          local v2 = vx * nx + vy * ny
          vx, vy, vz = nx * v2 + (vx - nx * v2) * kf, ny * v2 + (vy - ny * v2) * kf, vz * kf
        end
        hshape = true
        bnx, bny = nx, ny
      end
    end
  end
  return px, py, pz, vx, vy, vz, (h1 or h2 or h3_ or hs or hshape) and true or false
end
M.bounce = bounce

----------------------------------------------------------------------------- 力場（v0.6.0）

-- 力場のセクションが書いた 1 つ分を整える。種類: 0 渦 / 1 引力・斥力 / 2 衝撃波 / 3 双極 / 4 浮力
function M.prepare_field(F)
  local p = F.pos or {}
  local G = {
    type = floor(tonumber(F.type) or 0), cx = p[1] or 0, cy = p[2] or 0, cz = p[3] or 0,
    qx = p[4] or 200, qy = p[5] or 0, qz = p[6] or 0,
    str = F.str or 0, range = F.range or 0, i_str = F.i_str or 0, spin = F.spin or 0, up = F.up or 0,
    t0 = F.t0 or 0, wave = F.wave or 0, width = max(F.width or 1, 1e-3), follow = F.follow or 0,
    cool = F.cool or 0, heavy = F.heavy or 0, base = F.base or 0, grad = floor(tonumber(F.grad) or 0),
  }
  -- 渦の軸: Z（画面の奥）を X・Y の傾きで回す
  local ax, ay, az = 0, 0, 1
  local tx, ty = (F.ax or 0) * RAD, (F.ay or 0) * RAD
  ay, az = ay * cos(tx) - az * sin(tx), ay * sin(tx) + az * cos(tx)
  ax, az = ax * cos(ty) + az * sin(ty), -ax * sin(ty) + az * cos(ty)
  G.ax, G.ay, G.az = ax, ay, az
  return G
end

-- 種類: 5 画像の明るさ（v0.7.0。G.img = { mask = 明るさの格子, lay = オブジェクトの時刻 → 対象の位置・回転・拡大率・中心 }）

-- 届く距離での弱まり（中心から 届く距離 で半分。0 は弱めない）
local function falloff(G, d)
  if G.range <= 0 then return 1 end
  local u = d / G.range
  return 1 / (1 + u * u)
end

-- 力（px/秒²）。f = 粒子ごとの強さの倍率、tobj = オブジェクトの時刻、age = 年齢、vx..vz = 粒子の速度（衝撃波で使う）。
-- 双極は力ではなく向きを寄せる（M.dipole_dir）
function M.field_force(G, f, px, py, pz, tobj, age, vx, vy, vz)
  local ty = G.type
  if ty == 0 then
    -- 渦: 軸の周りを回し（回す強さ）、軸へ寄せ（強さ。負なら外へ）、軸に沿って押す（軸に沿う強さ）
    local rx, ry, rz = px - G.cx, py - G.cy, pz - G.cz
    local da = rx * G.ax + ry * G.ay + rz * G.az
    rx, ry, rz = rx - G.ax * da, ry - G.ay * da, rz - G.az * da
    local rl = sqrt(rx * rx + ry * ry + rz * rz)
    if rl < 1e-9 then return G.ax * G.up * f, G.ay * G.up * f, G.az * G.up * f end
    local w = falloff(G, rl) * f
    local ux, uy, uz = rx / rl, ry / rl, rz / rl
    local tx, ty_, tz = G.ay * uz - G.az * uy, G.az * ux - G.ax * uz, G.ax * uy - G.ay * ux
    return (tx * G.spin - ux * G.str + G.ax * G.up) * w, (ty_ * G.spin - uy * G.str + G.ay * G.up) * w,
           (tz * G.spin - uz * G.str + G.az * G.up) * w
  elseif ty == 1 then
    -- 引力・斥力: 中心へ 強さ（負なら外へ）
    local dx, dy, dz = G.cx - px, G.cy - py, G.cz - pz
    local d = sqrt(dx * dx + dy * dy + dz * dz)
    if d < 1e-9 then return 0, 0, 0 end
    local k = G.str * falloff(G, d) * f / d
    return dx * k, dy * k, dz * k
  elseif ty == 2 then
    -- 衝撃波: 半径 R = 広がる速さ × (t − 起きる時刻) の、幅 の帯の中を外へ押す。
    -- 力を（広がる速さ − 粒子の外向きの速さ）/ 幅 に比例させると、帯が通り過ぎる間に速さがちょうど 強さ だけ増える
    -- 積分の刻み G.h があれば、その刻みの間に帯（波の後ろからの距離 0..幅）と重なる長さの割合で掛ける（刻みの数え落とし・数えすぎを無くす）
    if G.wave <= 0 then return 0, 0, 0 end
    local R = G.wave * (tobj - G.t0)
    local dx, dy, dz = px - G.cx, py - G.cy, pz - G.cz
    local d = sqrt(dx * dx + dy * dy + dz * dz)
    if d < 1e-9 then return 0, 0, 0 end
    local vr = ((vx or 0) * dx + (vy or 0) * dy + (vz or 0) * dz) / d
    local rel = max(G.wave - vr, 0)
    local k
    if G.h then
      -- 位置は押された後の速さで進む（半陰的オイラー法）ので、重なりを押した後の速さで 2 回求め直す
      local u0 = R - d
      local w = falloff(G, d) * f
      local ov = 0
      for _ = 1, 3 do
        local dv = G.str * w * max(ov, 0) / G.width
        ov = min(u0 + max(rel - dv, 0) * G.h, G.width) - max(u0, 0)
      end
      if ov <= 0 or tobj + G.h < G.t0 then return 0, 0, 0 end
      k = G.str * ov / (G.width * G.h) * w / d
    else
      if tobj < G.t0 or d <= R - G.width or d > R then return 0, 0, 0 end
      k = G.str * rel / G.width * falloff(G, d) * f / d
    end
    return dx * k, dy * k, dz * k
  elseif ty == 4 then
    -- 浮力: 上へ 強さ × 冷め具合（年齢で 1 → 0）。冷めた分だけ下へ 冷めた後の重さ。基準の高さより上では弱める
    local c = 1
    if G.cool > 0 then c = max(1 - age / G.cool, 0) end
    local w = 1
    if G.range > 0 and py < G.base then w = falloff(G, G.base - py) end
    return 0, (-G.str * c * w + G.heavy * (1 - c)) * f, 0
  elseif ty == 5 then
    -- 画像の明るさ: 明るさの勾配の向き（明るい方へ / 暗い方へ / 等高線に沿う）へ、強さ × 勾配の大きさ（M.LUM_REF で 1）で押す
    local I = G.img
    if not I or not I.mask or not I.lay then return 0, 0, 0 end
    local X, Y, _, rz, sx, sy, cx, cy = I.lay(tobj)
    if X == nil then return 0, 0, 0 end
    local lx, ly = layer_unpoint(X, Y, rz, sx, sy, cx, cy, px, py)
    local gx, gy = M.lum_grad(I.mask, lx, ly)
    gx, gy = gx / (sx ~= 0 and sx or 1e-9), gy / (sy ~= 0 and sy or 1e-9)
    local c, s = cos(rz * RAD), sin(rz * RAD)
    local bx, by = gx * c - gy * s, gx * s + gy * c
    local gl = sqrt(bx * bx + by * by)
    if gl < 1e-12 then return 0, 0, 0 end
    local dx, dy = bx / gl, by / gl
    if G.grad == 1 then dx, dy = -dx, -dy elseif G.grad == 2 then dx, dy = -dy, dx end
    local k = G.str * min(gl * M.LUM_REF, 1) * f
    return dx * k, dy * k, 0
  end
  return 0, 0, 0
end

-- 双極の場の向き（1 点目 = N 極から出て、2 点目 = S 極へ入る。長さ 1）と、N 極からの距離での弱まり
function M.dipole_dir(G, px, py, pz)
  local ax, ay, az = px - G.cx, py - G.cy, pz - G.cz
  local bx, by, bz = px - G.qx, py - G.qy, pz - G.qz
  local la2, lb2 = ax * ax + ay * ay + az * az, bx * bx + by * by + bz * bz
  if la2 < 1e-12 or lb2 < 1e-12 then return 0, 0, 0, 0 end
  local da, db = la2 * sqrt(la2), lb2 * sqrt(lb2)
  local fx, fy, fz = ax / da - bx / db, ay / da - by / db, az / da - bz / db
  local l = sqrt(fx * fx + fy * fy + fz * fz)
  if l < 1e-30 then return 0, 0, 0, 0 end
  return fx / l, fy / l, fz / l, falloff(G, sqrt(la2))
end

-- 乱数を変える粒子の鍵（個別微調整の「乱数を変える番号」）
function M.reseed_key(sk, salt)
  return bit.bxor(sk, mix(bit.tobit(floor(salt) * 7919 + 13)))
end

-- 指定の時刻（シミュレーション時刻の並び）に n 個ずつ一斉に出す。放出の数の表に段を足す
function M.add_bursts(R, times, n)
  n = floor(n or 0)
  if not times or #times == 0 or n <= 0 then return R end
  local cs = {}
  for i = 1, #times do cs[i] = n end
  return M.add_bursts_var(R, times, cs)
end

-- 時刻 times[i] に counts[i] 個ずつ一斉に出す（同じ時刻はまとめる。数が 0 以下の時刻は使わない）。
-- 数え方は「その時刻ちょうどで出る」（count(t) は ts[i] <= t の分を足す）。探すのは二分探索
function M.add_bursts_var(R, times, counts)
  if not times or #times == 0 then return R end
  local pairs_ = {}
  for i = 1, #times do
    local c = floor(counts[i] or 0)
    if c > 0 then pairs_[#pairs_ + 1] = { times[i], c } end
  end
  if #pairs_ == 0 then return R end
  table.sort(pairs_, function(a, b) return a[1] < b[1] end)
  local ts, cum = {}, { [0] = 0 }
  for _, p in ipairs(pairs_) do
    local m = #ts
    if m > 0 and ts[m] == p[1] then
      cum[m] = cum[m] + p[2]
    else
      ts[m + 1] = p[1]
      cum[m + 1] = cum[m] + p[2]
    end
  end
  local nb = #ts
  -- t までに一斉に出た数（ts[i] <= t の i の数の和）
  local function upto(t)
    local lo, hi = 0, nb
    while lo < hi do
      local mid = floor((lo + hi + 1) / 2)
      if ts[mid] <= t then lo = mid else hi = mid - 1 end
    end
    return cum[lo], lo
  end
  return {
    count = function(t) return R.count(t) + (upto(t)) end,
    rate = R.rate,
    time_of = function(e)
      -- 一斉の時刻 ts[i+1] の時点で e を超える最初の区間 i（0..nb）を探し、その区間の中で元の放出だけで届く時刻を返す
      local lo, hi = 0, nb
      while lo < hi do
        local mid = floor((lo + hi) / 2)
        if R.count(ts[mid + 1]) + cum[mid + 1] > e then hi = mid else lo = mid + 1 end
      end
      local i = lo
      local t = R.time_of(e - cum[i])
      local top = i < nb and ts[i + 1] or huge
      if t < top then return max(t, i == 0 and -huge or ts[i]) end
      return top
    end,
  }
end

----------------------------------------------------------------------------- パス（v0.7.0）

-- アンカーを通る道を細かい折れ線にし、道のりの表を作る。pa = { pts, n, curve = 0 折れ線 / 1 曲線 }。
-- 曲線は一様なキャットマル・ロム（端は端点の向こうへ同じ向きに延ばした点を使う）。1 区間を 32 に分ける
function M.prepare_path(pa)
  local pts, n = pa.pts or {}, floor(pa.n or 0)
  if n < 2 or #pts < n * 3 then return nil end
  local function P3(i)
    if i < 1 then
      local ax, ay, az = P3(1)
      local bx, by, bz = P3(2)
      return 2 * ax - bx, 2 * ay - by, 2 * az - bz
    elseif i > n then
      local ax, ay, az = P3(n)
      local bx, by, bz = P3(n - 1)
      return 2 * ax - bx, 2 * ay - by, 2 * az - bz
    end
    local b = (i - 1) * 3
    return pts[b + 1], pts[b + 2], pts[b + 3]
  end
  local xs, ys, zs, m = {}, {}, {}, 0
  local function add(x, y, z) m = m + 1; xs[m], ys[m], zs[m] = x, y, z end
  if (pa.curve or 0) == 1 then
    local SUB = 32
    for i = 1, n - 1 do
      local x0, y0, z0 = P3(i - 1)
      local x1, y1, z1 = P3(i)
      local x2, y2, z2 = P3(i + 1)
      local x3, y3, z3 = P3(i + 2)
      for j = 0, SUB - 1 do
        local t = j / SUB
        local t2, t3 = t * t, t * t * t
        local function cr(a, b, c, d) return 0.5 * (2 * b + (c - a) * t + (2 * a - 5 * b + 4 * c - d) * t2 + (3 * b - a - 3 * c + d) * t3) end
        add(cr(x0, x1, x2, x3), cr(y0, y1, y2, y3), cr(z0, z1, z2, z3))
      end
    end
    add(P3(n))
  else
    for i = 1, n do add(P3(i)) end
  end
  local acc = { 0 }
  for i = 2, m do
    local dx, dy, dz = xs[i] - xs[i - 1], ys[i] - ys[i - 1], zs[i] - zs[i - 1]
    acc[i] = acc[i - 1] + sqrt(dx * dx + dy * dy + dz * dz)
  end
  return { x = xs, y = ys, z = zs, m = m, acc = acc, total = acc[m] }
end

-- 道のり d（0..全長に切る）の点と、進む向き（長さ 1）
local function path_at(PT, d)
  local acc, m = PT.acc, PT.m
  if d <= 0 then d = 0 elseif d >= PT.total then d = PT.total end
  local lo, hi = 1, m - 1
  while lo < hi do
    local mid = floor((lo + hi + 1) / 2)
    if acc[mid] <= d then lo = mid else hi = mid - 1 end
  end
  local i = lo
  local seg = acc[i + 1] - acc[i]
  local f = seg > 0 and (d - acc[i]) / seg or 0
  local dx, dy, dz = PT.x[i + 1] - PT.x[i], PT.y[i + 1] - PT.y[i], PT.z[i + 1] - PT.z[i]
  local l = sqrt(dx * dx + dy * dy + dz * dz)
  if l > 0 then dx, dy, dz = dx / l, dy / l, dz / l else dx, dy, dz = 1, 0, 0 end
  return PT.x[i] + (PT.x[i + 1] - PT.x[i]) * f, PT.y[i] + (PT.y[i + 1] - PT.y[i]) * f, PT.z[i] + (PT.z[i + 1] - PT.z[i]) * f, dx, dy, dz
end
M.path_at = path_at

-- 点にいちばん近い道の上の道のり
function M.path_nearest(PT, px, py, pz)
  local best, bd = 0, huge
  for i = 1, PT.m - 1 do
    local ax, ay, az = PT.x[i], PT.y[i], PT.z[i]
    local dx, dy, dz = PT.x[i + 1] - ax, PT.y[i + 1] - ay, PT.z[i + 1] - az
    local l2 = dx * dx + dy * dy + dz * dz
    local f = l2 > 0 and ((px - ax) * dx + (py - ay) * dy + (pz - az) * dz) / l2 or 0
    f = min(max(f, 0), 1)
    local ex, ey, ez = ax + dx * f - px, ay + dy * f - py, az + dz * f - pz
    local d2 = ex * ex + ey * ey + ez * ez
    if d2 < bd then bd, best = d2, PT.acc[i] + (PT.acc[i + 1] - PT.acc[i]) * f end
  end
  return best
end

-- 粒子ごとの値（出る所の道のり・速さ・横のずれ）を q に置く
local function path_setup(PA, q, k, ik, sk, px, py, pz)
  local PT = PA.pt
  local v = vary(PA.speed, PA.i_speed, rnd(ik, CH.path, sk))
  q.pv = v
  q.pox = nil
  if PA.start == 1 then
    q.ps0 = rnd(k, CH.path + 1, sk) * PT.total
  elseif PA.start == 2 then
    -- 出力位置からいちばん近い所から。道からのずれはそのまま保つ（生まれた所から跳ばない）
    local d = M.path_nearest(PT, px, py, pz)
    local x, y, z = path_at(PT, d)
    q.ps0, q.pox, q.poy, q.poz = d, px - x, py - y, pz - z
  else
    q.ps0 = v >= 0 and 0 or PT.total
  end
  q.poff = (rnd(k, CH.path + 2, sk) - 0.5) * PA.width
end

-- 年齢 tau の位置と速度。端に着いたら 0 消える（dead = true）/ 1 止まる / 2 最初へ戻る / 3 折り返す
local function path_pos(PA, q, tau)
  local PT = PA.pt
  local T = PT.total
  local d = q.ps0 + q.pv * tau + 0.5 * PA.accel * tau * tau
  local spd = q.pv + PA.accel * tau
  local dead, sgn = false, 1
  if PA.pend == 0 then
    if d < 0 or d > T then dead = true end
  elseif PA.pend == 1 then
    if d <= 0 or d >= T then spd = 0 end
  elseif PA.pend == 2 then
    d = d % T
  else
    local u = d % (2 * T)
    if u > T then u, sgn = 2 * T - u, -1 end
    d = u
  end
  local x, y, z, tx, ty, tz = path_at(PT, d)
  if q.pox then
    x, y, z = x + q.pox, y + q.poy, z + q.poz
  else
    local nx, ny = -ty, tx
    local nl = sqrt(nx * nx + ny * ny)
    if nl > 1e-9 then nx, ny = nx / nl, ny / nl else nx, ny = 1, 0 end
    x, y = x + nx * q.poff, y + ny * q.poff
  end
  return x, y, z, tx * spd * sgn, ty * spd * sgn, dead
end

----------------------------------------------------------------------------- 積分

pcall(ffi.cdef, [[
typedef struct {
  int32_t key, steps, flags, nb, tgt;
  double px, py, pz, dx, dy, dz, s, wx, wy, wz, tb, td, ac, bb;
  double e1x, e1y, e1z, e1vx, e1vy, e2x, e2y, e2z, e2vx, e2vy, tk;
  double roll, om;
} PRH_State9;
]])

-- 状態の旗
local F_DISP, F_STOP, F_DEAD, F_PARK, F_BNC = 1, 2, 4, 8, 16
M.F_DEAD = F_DEAD
M.F_BNC = F_BNC
M.F_DISP = F_DISP

-- 位置 p・速度 v・時刻 t（年齢 ta）での力（px/秒²）。RK4 の 2〜4 段目に使う（v0.19.0。1 段目は advance の中で同じ式を直接計算する）。
-- 正本は advance の中の 重力・風・ノイズ場・力場・子粒子の空気抵抗。直すときは両方を直す
local function force_at(P, q, px, py, pz, vx, vy, vz, t, ta)
  local W, N, FL = P.wind, P.noise, P.fields
  local fx, fy, fz = q.gx, q.gy, q.gz
  if W then
    local f = q.fwind
    if W.point then
      local ex, ey, ez = px - W.px, py - W.py, pz - W.pz
      f = f * math.exp(-sqrt(ex * ex + ey * ey + ez * ez) / W.range)
    end
    local ax_, ay_, az_ = W.cx, W.cy, W.cz
    if W.moving then ax_, ay_, az_ = W.val("wx", t), W.val("wy", t), W.val("wz", t) end
    fx, fy, fz = fx + ax_ * f - W.drag * vx, fy + ay_ * f - W.drag * vy, fz + az_ * f - W.drag * vz
  end
  if N then
    local amp = (N.moving and N.val("str", t) or N.cstr) * q.fnoise
    if N.fade > 0 then amp = amp * min(max((ta - N.start) / N.fade, 0), 1)
    elseif ta < N.start then amp = 0 end
    if amp ~= 0 then
      local cx, cy, cz = M.curl(px, py, pz, t, N)
      fx, fy, fz = fx + cx * amp, fy + cy * amp, fz + cz * amp
    end
  end
  if FL then
    local tobj = P.obj_time and P.obj_time(t) or t
    for fi = 1, #FL do
      local G = FL[fi]
      if G.type ~= 3 then
        local ax_, ay_, az_ = M.field_force(G, q.ff[fi], px, py, pz, tobj, ta, vx, vy, vz)
        fx, fy, fz = fx + ax_, fy + ay_, fz + az_
      end
    end
  end
  if q.drag then fx, fy, fz = fx - q.drag * vx, fy - q.drag * vy, fz - q.drag * vz end
  return fx, fy, fz
end

-- 粒子 1 個を from 刻みから to 刻みまで進める（S は状態、q は粒子ごとの値、P は全体の設定）。
-- 積分法（v0.19.0。P.integ）: nil = 半陰的オイラー（速度を先に更新し、その速度で位置を進める。今までの計算）/
-- 1 = 陽的オイラー（位置は刻みの始めの速度で進める）/ 2 = RK4（力を 4 回求め、位置と速度を重み付きの平均で進める）。
-- 集結点・双極の向きの寄せと跳ね返りは、どの方法でも刻みの終わりに 1 回だけ掛ける
-- 状態は刻みの境目だけに置く。同じ刻みを同じ順で積むので、前のフレームから進めても誕生から計算し直しても同じ値になる
local function advance(P, q, S, from, to)
  local h = P.h
  local px, py, pz = S.px, S.py, S.pz
  local dx, dy, dz, s = S.dx, S.dy, S.dz, S.s
  local wx, wy, wz = S.wx, S.wy, S.wz
  local ac, flags, nb, tgt, tb, td = S.ac, S.flags, S.nb, S.tgt, S.tb, S.td
  local tk = S.tk
  local roll, om = S.roll, S.om
  local KP = P.keep
  local k, sk = q.k, q.sk
  local W, N, A, B, D, U, T = P.wind, P.noise, P.att, P.bnc, P.disp, P.sus, P.act
  local FL = P.fields
  for i = from, to - 1 do
    if bit.band(flags, F_DEAD + F_PARK) ~= 0 then break end
    local ta = i * h
    local t = q.b + ta
    local nb0 = nb
    if not T or in_window(T.win, T.rel and ta or T.obj(t)) then
      -- 分散してから止める: 止める時刻は 分散した時刻 ＋ つなぐ時間（分散するまでは止めない）
      local tstop = q.t_stop
      if q.link1 then tstop = bit.band(flags, F_DISP) ~= 0 and (td + q.link_t) or 0 end
      if tstop > 0 and bit.band(flags, F_STOP) == 0 and ta >= tstop then
        s, wx, wy, wz = 0, 0, 0, 0
        flags = bit.bor(flags, F_STOP)
      end
      if D and bit.band(flags, F_DISP) == 0 and
         ((D.by_bounce and nb > 0) or (not D.by_bounce and q.t_disp > 0 and ta >= q.t_disp)) then
        local vx, vy, vz = dx * s + wx, dy * s + wy, dz * s + wz
        local sp = sqrt(vx * vx + vy * vy + vz * vz)
        local ux, uy, uz = dx, dy, dz
        if sp > 1e-9 then ux, uy, uz = vx / sp, vy / sp, vz / sp end
        ux, uy, uz = turn(ux, uy, uz, (rnd(k, CH.disp1, sk) * 2 - 1) * D.xy, (rnd(k, CH.disp2, sk) * 2 - 1) * D.z)
        if D.speed_set or q.link2 then sp = q.disp_speed end
        dx, dy, dz, s, wx, wy, wz = ux, uy, uz, sp, 0, 0, 0
        if D.acc_set then ac = D.acc end
        flags = bit.bor(flags, F_DISP)
        td = ta
        S.e2x, S.e2y, S.e2z, S.e2vx, S.e2vy = px, py, pz, ux * sp, uy * sp
      end
      if U then
        local j1 = floor((ta + h) / U.interval)
        if j1 > floor(ta / U.interval) and rnd(k * 131 + j1, CH.sus, sk) * 100 < U.prob then
          local ix, iy, iz = U.x, U.y, U.z
          if U.random then
            local zz = U.z3 and (rnd(k * 131 + j1, CH.sus + 1, sk) * 2 - 1) or 0
            local ph = rnd(k * 131 + j1, CH.sus + 2, sk) * 2 * pi
            local rr = sqrt(max(1 - zz * zz, 0))
            ix, iy, iz = rr * cos(ph), rr * sin(ph), zz
          end
          wx, wy, wz = wx + ix * U.speed, wy + iy * U.speed, wz + iz * U.speed
        end
      end
      local vx, vy, vz
      local fx, fy, fz = q.gx, q.gy, q.gz
      if W then
        vx, vy, vz = dx * s + wx, dy * s + wy, dz * s + wz
        local f = q.fwind
        if W.point then
          local ex, ey, ez = px - W.px, py - W.py, pz - W.pz
          f = f * math.exp(-sqrt(ex * ex + ey * ey + ez * ez) / W.range)
        end
        local ax_, ay_, az_ = W.cx, W.cy, W.cz
        if W.moving then ax_, ay_, az_ = W.val("wx", t), W.val("wy", t), W.val("wz", t) end
        fx = fx + ax_ * f - W.drag * vx
        fy = fy + ay_ * f - W.drag * vy
        fz = fz + az_ * f - W.drag * vz
      end
      if N then
        local amp = (N.moving and N.val("str", t) or N.cstr) * q.fnoise
        if N.fade > 0 then amp = amp * min(max((ta - N.start) / N.fade, 0), 1)
        elseif ta < N.start then amp = 0 end
        if amp ~= 0 then
          local cx, cy, cz = M.curl(px, py, pz, t, N)
          fx, fy, fz = fx + cx * amp, fy + cy * amp, fz + cz * amp
        end
      end
      if FL then
        -- 力場（双極は下で向きを寄せる）
        local tobj = P.obj_time and P.obj_time(t) or t
        for fi = 1, #FL do
          local G = FL[fi]
          if G.type ~= 3 then
            local ax_, ay_, az_ = M.field_force(G, q.ff[fi], px, py, pz, tobj, ta, dx * s + wx, dy * s + wy, dz * s + wz)
            fx, fy, fz = fx + ax_, fy + ay_, fz + az_
          end
        end
      end
      if q.drag then
        -- 子粒子の空気抵抗（v0.15.0。子に動きの拡張を掛けるとき。閉じた式の子と同じ、速さに比例する抵抗）
        fx, fy, fz = fx - q.drag * (dx * s + wx), fy - q.drag * (dy * s + wy), fz - q.drag * (dz * s + wz)
      end
      -- 積分法（v0.19.0）: 位置を進める速さ（bx0..）を方法ごとに決め、RK4 は力を 4 段の重み付きの平均にする
      local IM, bx0, by0, bz0 = P.integ, 0, 0, 0
      if IM == 1 then
        bx0, by0, bz0 = dx * s + wx, dy * s + wy, dz * s + wz
      elseif IM == 2 then
        local hh = 0.5 * h
        local s2, s4 = s, s
        if ac ~= 0 then
          s2, s4 = s + ac * hh, s + ac * h
          if ac < 0 then s2, s4 = max(s2, 0), max(s4, 0) end
        end
        local v1x, v1y, v1z = dx * s + wx, dy * s + wy, dz * s + wz
        local v2x, v2y, v2z = dx * s2 + wx + fx * hh, dy * s2 + wy + fy * hh, dz * s2 + wz + fz * hh
        local a2x, a2y, a2z = force_at(P, q, px + v1x * hh, py + v1y * hh, pz + v1z * hh, v2x, v2y, v2z, t + hh, ta + hh)
        local v3x, v3y, v3z = dx * s2 + wx + a2x * hh, dy * s2 + wy + a2y * hh, dz * s2 + wz + a2z * hh
        local a3x, a3y, a3z = force_at(P, q, px + v2x * hh, py + v2y * hh, pz + v2z * hh, v3x, v3y, v3z, t + hh, ta + hh)
        local v4x, v4y, v4z = dx * s4 + wx + a3x * h, dy * s4 + wy + a3y * h, dz * s4 + wz + a3z * h
        local a4x, a4y, a4z = force_at(P, q, px + v3x * h, py + v3y * h, pz + v3z * h, v4x, v4y, v4z, t + h, ta + h)
        bx0, by0, bz0 = (v1x + 2 * v2x + 2 * v3x + v4x) / 6, (v1y + 2 * v2y + 2 * v3y + v4y) / 6, (v1z + 2 * v2z + 2 * v3z + v4z) / 6
        fx, fy, fz = (fx + 2 * a2x + 2 * a3x + a4x) / 6, (fy + 2 * a2y + 2 * a3y + a4y) / 6, (fz + 2 * a2z + 2 * a3z + a4z) / 6
      end
      if ac ~= 0 then
        s = s + ac * h
        if ac < 0 and s < 0 then s = 0 end
      end
      wx, wy, wz = wx + fx * h, wy + fy * h, wz + fz * h
      vx, vy, vz = dx * s + wx, dy * s + wy, dz * s + wz
      -- 位置を進める速さと、刻みの終わりの速さの差（向きの寄せの後も保つ）
      if IM then bx0, by0, bz0 = bx0 - vx, by0 - vy, bz0 - vz end
      if A then
        local on
        if A.rel then on = ta >= q.t_att else on = A.obj(t) >= q.t_att end
        if on then
          local ti
          if A.mode == 2 then
            local since = A.rel and (ta - q.t_att) or (A.obj(t) - q.t_att)
            ti = floor(max(since, 0) / A.interval) % A.n
          elseif tgt >= 0 then
            ti = tgt
          elseif A.mode == 1 then
            local best = huge
            ti = 0
            for j = 0, A.n - 1 do
              local ex, ey, ez = A.pts[j * 3 + 1] - px, A.pts[j * 3 + 2] - py, A.pts[j * 3 + 3] - pz
              local dd = ex * ex + ey * ey + ez * ez
              if dd < best then best, ti = dd, j end
            end
          else
            ti = min(floor(rnd(k, CH.att, sk) * A.n), A.n - 1)
          end
          tgt = ti
          local ex, ey, ez = A.pts[ti * 3 + 1] - px, A.pts[ti * 3 + 2] - py, A.pts[ti * 3 + 3] - pz
          local dist = sqrt(ex * ex + ey * ey + ez * ez)
          if A.arrive > 0 and dist <= A.radius then
            flags = bit.bor(flags, A.arrive == 1 and F_DEAD or F_PARK)
            if A.arrive ~= 1 then tk = ta end
            s, wx, wy, wz = 0, 0, 0, 0
            break
          end
          if dist > 1e-9 then
            local ux, uy, uz = turn(ex / dist, ey / dist, ez / dist, q.att_xy, q.att_z)
            local sp = A.speed > 0 and A.speed or sqrt(vx * vx + vy * vy + vz * vz)
            local kk = min(A.strength * h, 1)
            vx, vy, vz = vx + (ux * sp - vx) * kk, vy + (uy * sp - vy) * kk, vz + (uz * sp - vz) * kk
            local spd = sqrt(vx * vx + vy * vy + vz * vz)
            if spd > 1e-12 then dx, dy, dz = vx / spd, vy / spd, vz / spd end
            s, wx, wy, wz = spd, 0, 0, 0
          end
        end
      end
      if P.dipole then
        -- 双極: 速度の向きを場の向きへ寄せる（強さが正なら、場に沿う速さもその値にする）
        for fi = 1, #FL do
          local G = FL[fi]
          if G.type == 3 then
            local bx_, by_, bz_, w = M.dipole_dir(G, px, py, pz)
            if w > 0 then
              local sp = sqrt(vx * vx + vy * vy + vz * vz)
              if G.str > 0 then sp = G.str * q.ff[fi] end
              local kk = min(G.follow * w * h, 1)
              vx, vy, vz = vx + (bx_ * sp - vx) * kk, vy + (by_ * sp - vy) * kk, vz + (bz_ * sp - vz) * kk
              local spd = sqrt(vx * vx + vy * vy + vz * vz)
              if spd > 1e-12 then dx, dy, dz = vx / spd, vy / spd, vz / spd end
              s, wx, wy, wz = spd, 0, 0, 0
            end
          end
        end
      end
      local mx_, my_, mz_ = vx, vy, vz
      if IM then mx_, my_, mz_ = vx + bx0, vy + by0, vz + bz0 end
      if P.vmul then
        -- 速度倍率（拡大率と透過率）: 自分の速さの項だけに掛ける。状態の速さは変えず、位置にだけ使う
        local mm = P.vmul((ta + 0.5 * h) / q.L) - 1     -- 刻みの真ん中の倍率（端の値だと刻み幅に比例してずれる）
        mx_, my_, mz_ = mx_ + dx * s * mm, my_ + dy * s * mm, mz_ + dz * s * mm
      end
      if P.vec then
        -- 挙動の速度の関数（粒子が生まれてからの秒）。状態の速度には入れず、位置にだけ足す
        local fx_, fy_, fz_ = P.vec(ta)
        mx_, my_, mz_ = mx_ + fx_, my_ + fy_, mz_ + fz_
      end
      px, py, pz = px + mx_ * h, py + my_ * h, pz + mz_ * h
      if B then
        local hit
        px, py, pz, vx, vy, vz, hit = bounce(B, px, py, pz, vx, vy, vz, t + h, ta + h)
        if hit then
          if B.irr and rnd(k * 131 + nb, CH.irr, sk) * 100 < B.irrp then
            local sp = sqrt(vx * vx + vy * vy + vz * vz)
            if sp > 1e-12 then
              local ux, uy, uz = turn(vx / sp, vy / sp, vz / sp, (rnd(k * 131 + nb, CH.irr + 1, sk) * 2 - 1) * B.irra, 0)
              vx, vy, vz = ux * sp, uy * sp, uz * sp
            end
          end
          nb = nb + 1
          if bit.band(flags, F_BNC) == 0 then
            flags = bit.bor(flags, F_BNC)
            tb = ta + h
            S.e1x, S.e1y, S.e1z, S.e1vx, S.e1vy = px, py, pz, vx, vy
          end
          local spd = sqrt(vx * vx + vy * vy + vz * vz)
          if spd > 1e-12 then dx, dy, dz = vx / spd, vy / spd, vz / spd end
          s, wx, wy, wz = spd, 0, 0, 0
          -- 転がる（v0.15.0）: 当たった後の面に沿う速さ ÷ 半径 で回る（宙では同じ速さで回り続ける）
          if B.roll then om = (bnx * vy - bny * vx) / B.rrad end
        end
        if B.roll then roll = roll + om * h end
      end
      -- 止まったら残す（v0.8.0）: 跳ね返った直後か急停止した後に、速さがしきい値を下回ったら、その場に止める
      -- （跳ね返った後のいつでも見ると、小さく跳ねた頂点で宙に止まる）
      if KP and (nb > nb0 or bit.band(flags, F_STOP) ~= 0) then
        local vx2, vy2, vz2 = dx * s + wx, dy * s + wy, dz * s + wz
        if vx2 * vx2 + vy2 * vy2 + vz2 * vz2 < KP.speed * KP.speed then
          flags = bit.bor(flags, F_PARK)
          tk = ta + h
          s, wx, wy, wz = 0, 0, 0, 0
          om = 0
          if P.hist2 and (i + 1) % P.hstride2 == 0 then
            local base2 = q.slot * P.hm2 + ((i + 1) / P.hstride2) % P.hm2
            P.hist2[base2 * 3], P.hist2[base2 * 3 + 1], P.hist2[base2 * 3 + 2] = px, py, pz
            P.hstep2[base2] = i + 1
          end
          break
        end
      end
    end
    if P.hist and (i + 1) % P.hstride == 0 then
      local base = q.slot * P.hm + ((i + 1) / P.hstride) % P.hm
      P.hist[base * 3], P.hist[base * 3 + 1], P.hist[base * 3 + 2] = px, py, pz
      P.hstep[base] = i + 1
    end
    if P.hist2 and (i + 1) % P.hstride2 == 0 then
      local base2 = q.slot * P.hm2 + ((i + 1) / P.hstride2) % P.hm2
      P.hist2[base2 * 3], P.hist2[base2 * 3 + 1], P.hist2[base2 * 3 + 2] = px, py, pz
      P.hstep2[base2] = i + 1
    end
  end
  S.px, S.py, S.pz, S.dx, S.dy, S.dz, S.s = px, py, pz, dx, dy, dz, s
  S.wx, S.wy, S.wz, S.ac, S.flags, S.nb, S.tgt, S.tb, S.td = wx, wy, wz, ac, flags, nb, tgt, tb, td
  S.tk = tk
  S.roll, S.om = roll, om
  S.steps = to
end
M.advance = advance

--[[
粒子どうし（v0.12.0）: 世界の粒子 i（1..n）の、ラウンドの始めの位置から求める加速度と、重ならないための位置・速さの直し。
W = 世界、IN = P.inter。戻り値: ax, ay, az, cx, cy, cz（位置の直し）, dvx, dvy, dvz（速さの直し）の並び（表）
近所は升の大きさ = 届く距離 の格子から探す（奥行きも使うなら 3 次元）。並びは世界の番号順（表を pairs で回さない）
]]
-- 状態 S の速さを (vx, vy, vz) にする（向き・速さ・風の分に分けて持つ形に直す。集結点と同じ）
local function set_velocity(S, vx, vy, vz)
  local sp = sqrt(vx * vx + vy * vy + vz * vz)
  if sp > 1e-12 then S.dx, S.dy, S.dz = vx / sp, vy / sp, vz / sp end
  S.s, S.wx, S.wy, S.wz = sp, 0, 0, 0
end

local function inter_forces(W, IN)
  local n, st = W.n, W.st
  local R, R2 = IN.r, IN.r * IN.r
  local d3 = IN.d3
  -- 置き場（ラウンドごとに作らず使い回す）。格子は升の番号のハッシュの連結リスト（head = 先頭、nxt = 次。-1 で終わり）
  local B = W.ibuf
  if not B or B.cap < n then
    local cap = 256
    while cap < n do cap = cap * 2 end
    local hs = 1
    while hs < cap * 2 do hs = hs * 2 end
    B = { cap = cap, hs = hs, head = ffi.new("int32_t[?]", hs), nxt = ffi.new("int32_t[?]", cap + 1),
          cx = ffi.new("int32_t[?]", cap + 1), cy = ffi.new("int32_t[?]", cap + 1), cz = ffi.new("int32_t[?]", cap + 1),
          f = ffi.new("double[?]", (cap + 1) * 9) }
    W.ibuf = B
  end
  local mask = B.hs - 1
  local head, nxt, CX, CY, CZ, Fb = B.head, B.nxt, B.cx, B.cy, B.cz, B.f
  ffi.fill(head, B.hs * 4, 0xff)
  ffi.fill(Fb, (n + 1) * 9 * 8, 0)
  local band, bxor = bit.band, bit.bxor
  for i = 1, n do
    if not W.frozen[i] then
      local S = st[i - 1]
      local cx, cy, cz = floor(S.px / R), floor(S.py / R), d3 and floor(S.pz / R) or 0
      CX[i], CY[i], CZ[i] = cx, cy, cz
      local hb = band(bxor(cx * 73856093, cy * 19349663, cz * 83492791), mask)
      nxt[i] = head[hb]
      head[hb] = i
    end
  end
  local zr = d3 and 1 or 0
  local vb = {}
  for i = 1, n do
    local ax, ay, az, kx, ky, kz, ux, uy, uz = 0, 0, 0, 0, 0, 0, 0, 0, 0
    local touch = 0
    local Si = st[i - 1]
    local parked = bit.band(Si.flags, F_PARK + F_DEAD) ~= 0
    if not W.frozen[i] and not parked then
      local pix, piy, piz = Si.px, Si.py, Si.pz
      local vix, viy, viz = Si.dx * Si.s + Si.wx, Si.dy * Si.s + Si.wy, Si.dz * Si.s + Si.wz
      if not d3 then viz = 0 end
      local vl = sqrt(vix * vix + viy * viy + viz * viz)
      local cnt, sx, sy, sz, avx, avy, avz, mx, my, mz = 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
      -- 見る升（隣と自分）のハッシュ。同じハッシュの升は 1 回だけ見る（別の升の粒子は距離で外れる）
      local nb = 0
      for gz = CZ[i] - zr, CZ[i] + zr do
        for gy = CY[i] - 1, CY[i] + 1 do
          for gx = CX[i] - 1, CX[i] + 1 do
            local hb = band(bxor(gx * 73856093, gy * 19349663, gz * 83492791), mask)
            local dup = false
            for c = 1, nb do if vb[c] == hb then dup = true; break end end
            if not dup then nb = nb + 1; vb[nb] = hb end
          end
        end
      end
      for c = 1, nb do
        local j = head[vb[c]]
        while j >= 0 do
          if j ~= i then
            local Sj = st[j - 1]
            local dx, dy, dz = pix - Sj.px, piy - Sj.py, d3 and (piz - Sj.pz) or 0
            local dd = dx * dx + dy * dy + dz * dz
            if dd < R2 and dd > 1e-12 then
              local d = sqrt(dd)
              local nx, ny, nz = dx / d, dy / d, dz / d
              local f = 1 - d / R
              -- 反発（近いほど強く）・引き合い（届く距離の中で、遠いほど強く寄る）
              local a = IN.rep * f * f - IN.att * (d / R)
              ax, ay, az = ax + nx * a, ay + ny * a, az + nz * a
              -- 重ならない: 粒子の大きさより近いと、位置を押し戻し、近づく速さを消す（止まった相手は動かない）
              if IN.size > 0 and d < IN.size then
                local jpark = bit.band(Sj.flags, F_PARK + F_DEAD) ~= 0 or W.frozen[j]
                local share = jpark and 1 or 0.5
                local push = (IN.size - d) * share
                touch = touch + 1
                kx, ky, kz = kx + nx * push, ky + ny * push, kz + nz * push
                local vjx, vjy, vjz = Sj.dx * Sj.s + Sj.wx, Sj.dy * Sj.s + Sj.wy, Sj.dz * Sj.s + Sj.wz
                if jpark then vjx, vjy, vjz = 0, 0, 0 end
                if not d3 then vjz = 0 end
                local vn = (vix - vjx) * nx + (viy - vjy) * ny + (viz - vjz) * nz
                if vn < 0 then ux, uy, uz = ux - nx * vn * share, uy - ny * vn * share, uz - nz * vn * share end
                -- 粒子どうしの摩擦: 触れている相手とのずれる速さ（法線に直角な分）を弱める（砂が山になる）
                if IN.fric > 0 then
                  local rx, ry, rz = vix - vjx - vn * nx, viy - vjy - vn * ny, viz - vjz - vn * nz
                  ux, uy, uz = ux - rx * IN.fric * share, uy - ry * IN.fric * share, uz - rz * IN.fric * share
                end
              end
              -- 群れ: 見える角度の中の仲間
              if IN.sep ~= 0 or IN.ali ~= 0 or IN.coh ~= 0 then
                local seen = true
                if IN.view > -1 and vl > 1e-9 then
                  seen = -(dx * vix + dy * viy + dz * viz) >= d * vl * IN.view
                end
                if seen then
                  cnt = cnt + 1
                  sx, sy, sz = sx + nx * f, sy + ny * f, sz + nz * f
                  local vjx, vjy, vjz = Sj.dx * Sj.s + Sj.wx, Sj.dy * Sj.s + Sj.wy, Sj.dz * Sj.s + Sj.wz
                  avx, avy, avz = avx + vjx, avy + vjy, avz + (d3 and vjz or 0)
                  mx, my, mz = mx + Sj.px, my + Sj.py, mz + (d3 and Sj.pz or piz)
                end
              end
            end
          end
          j = nxt[j]
        end
      end
      if cnt > 0 then
        -- 離れる: 近い仲間ほど強く（最大 2000 px/秒²）。そろえる: 仲間の平均の速さへ 1 秒に 4 倍の割合で寄せる。寄る: 仲間の中心へばねで寄る
        ax, ay, az = ax + sx * IN.sep * 2000, ay + sy * IN.sep * 2000, az + sz * IN.sep * 2000
        ax = ax + (avx / cnt - vix) * IN.ali * 4
        ay = ay + (avy / cnt - viy) * IN.ali * 4
        az = az + (avz / cnt - viz) * IN.ali * 4
        ax = ax + (mx / cnt - pix) * IN.coh * 4
        ay = ay + (my / cnt - piy) * IN.coh * 4
        az = az + (mz / cnt - piz) * IN.coh * 4
      end
    end
    -- 重なりの直し: 位置は足し合わせて大きさの半分まで、速さは触れている相手の数で割る（何粒も重なった所で足し合わせると飛び出す）
    if touch > 0 then
      local kl = sqrt(kx * kx + ky * ky + kz * kz)
      local lim = IN.size * 0.5
      if kl > lim then kx, ky, kz = kx / kl * lim, ky / kl * lim, kz / kl * lim end
      if touch > 1 then ux, uy, uz = ux / touch, uy / touch, uz / touch end
    end
    if not d3 then az, kz, uz = 0, 0, 0 end
    local o = i * 9
    Fb[o], Fb[o + 1], Fb[o + 2], Fb[o + 3], Fb[o + 4], Fb[o + 5], Fb[o + 6], Fb[o + 7], Fb[o + 8] = ax, ay, az, kx, ky, kz, ux, uy, uz
  end
  return Fb
end
M.inter_forces = inter_forces

M.INTER_ITERS = 4   -- 重ならないための位置の直しを、1 ラウンドにくり返す回数

--[[
当たる相手（v0.13.0）: A = 相手の粒子（{ n, x, y, z, vx, vy }。こちらの本体から見た座標）を動く球として、世界の粒子を押し出す（一方向）。
相手の大きさ（中心どうしの距離）より近ければ位置を外へ出し、相手へ近づく速さを消す。相手の大きさの 1.5 倍の中では、相手から押される強さで押す
]]
function M.inter_obstacles(W, IN, A)
  if not A or (A.n or 0) <= 0 then return end
  local R0 = IN.orad
  local R1 = R0 * 1.5
  local d3 = IN.d3
  local grid = {}
  local OFF = 1048576
  for o = 1, A.n do
    local cx, cy = floor(A.x[o] / R1), floor(A.y[o] / R1)
    local key = (cx + OFF) + (cy + OFF) * 2097152
    local cell = grid[key]
    if not cell then cell = {}; grid[key] = cell end
    cell[#cell + 1] = o
  end
  local st, h = W.st, 0
  for i = 1, W.n do
    local S = st[i - 1]
    if not W.frozen[i] and bit.band(S.flags, F_PARK + F_DEAD) == 0 then
      local cx, cy = floor(S.px / R1), floor(S.py / R1)
      local kx, ky, kz, ax, ay = 0, 0, 0, 0, 0
      local vx, vy, vz = S.dx * S.s + S.wx, S.dy * S.s + S.wy, S.dz * S.s + S.wz
      local dvx, dvy, dvz, hit = 0, 0, 0, 0
      for gy = cy - 1, cy + 1 do
        for gx = cx - 1, cx + 1 do
          local cell = grid[(gx + OFF) + (gy + OFF) * 2097152]
          if cell then
            for c = 1, #cell do
              local o = cell[c]
              local dx, dy, dz = S.px - A.x[o], S.py - A.y[o], d3 and (S.pz - (A.z[o] or 0)) or 0
              local dd = dx * dx + dy * dy + dz * dz
              if dd < R1 * R1 and dd > 1e-12 then
                local d = sqrt(dd)
                local nx, ny, nz = dx / d, dy / d, dz / d
                local f = 1 - d / R1
                ax, ay = ax + nx * IN.orep * f * f, ay + ny * IN.orep * f * f
                if d < R0 then
                  kx, ky, kz = kx + nx * (R0 - d), ky + ny * (R0 - d), kz + nz * (R0 - d)
                  local vn = (vx - (A.vx[o] or 0)) * nx + (vy - (A.vy[o] or 0)) * ny + vz * nz
                  if vn < 0 then dvx, dvy, dvz = dvx - nx * vn, dvy - ny * vn, dvz - nz * vn end
                  hit = hit + 1
                end
              end
            end
          end
        end
      end
      if hit > 1 then dvx, dvy, dvz = dvx / hit, dvy / hit, dvz / hit end
      if kx ~= 0 or ky ~= 0 or kz ~= 0 then S.px, S.py, S.pz = S.px + kx, S.py + ky, S.pz + kz end
      -- 押す力は 1 刻み分の速さにして足す（次のラウンドの advance を待たない）
      local hh = W.h or 0
      if dvx ~= 0 or dvy ~= 0 or dvz ~= 0 or ax ~= 0 or ay ~= 0 then
        set_velocity(S, vx + dvx + ax * hh, vy + dvy + ay * hh, vz + dvz)
      end
      h = h + hit
    end
  end
  W.obstacle_hits = h
end

-- 重ならない（粒子の大きさ）: 位置だけを直す。触れている相手から重なりの半分ずつ離し（止まった相手からは全部）、
-- 相手の数で割った分だけ動かす。これを INTER_ITERS 回（重なりが無くなれば途中で止める）くり返し、動かした向きに逆らう速さを消す（上に積んだ重さで沈まない）
local function inter_resolve(W, IN)
  local n, st = W.n, W.st
  local size = IN.size
  if size <= 0 or n == 0 then return end
  local B = W.ibuf
  local R = max(size, 1)
  local d3 = IN.d3
  local mask = B.hs - 1
  local head, nxt, CX, CY, CZ, Fb = B.head, B.nxt, B.cx, B.cy, B.cz, B.f
  local band, bxor = bit.band, bit.bxor
  local zr = d3 and 1 or 0
  local vb = {}
  local moved = {}
  -- 1 回目は全部の粒子、2 回目からは前の回で触れていた粒子だけを見直す
  local cand, nc = nil, n
  for it = 1, M.INTER_ITERS do
    local any = false
    local next_c, nn = {}, 0
    ffi.fill(head, B.hs * 4, 0xff)
    for i = 1, n do
      if not W.frozen[i] then
        local S = st[i - 1]
        local cx, cy, cz = floor(S.px / R), floor(S.py / R), d3 and floor(S.pz / R) or 0
        CX[i], CY[i], CZ[i] = cx, cy, cz
        local hb = band(bxor(cx * 73856093, cy * 19349663, cz * 83492791), mask)
        nxt[i] = head[hb]
        head[hb] = i
      end
    end
    for ci = 1, nc do
      local i = cand and cand[ci] or ci
      local kx, ky, kz, touch = 0, 0, 0, 0
      local Si = st[i - 1]
      if not W.frozen[i] and bit.band(Si.flags, F_PARK + F_DEAD) == 0 then
        local nb = 0
        for gz = CZ[i] - zr, CZ[i] + zr do
          for gy = CY[i] - 1, CY[i] + 1 do
            for gx = CX[i] - 1, CX[i] + 1 do
              local hb = band(bxor(gx * 73856093, gy * 19349663, gz * 83492791), mask)
              local dup = false
              for c = 1, nb do if vb[c] == hb then dup = true; break end end
              if not dup then nb = nb + 1; vb[nb] = hb end
            end
          end
        end
        for c = 1, nb do
          local j = head[vb[c]]
          while j >= 0 do
            if j ~= i then
              local Sj = st[j - 1]
              local dx, dy, dz = Si.px - Sj.px, Si.py - Sj.py, d3 and (Si.pz - Sj.pz) or 0
              local dd = dx * dx + dy * dy + dz * dz
              if dd < size * size and dd > 1e-12 then
                local d = sqrt(dd)
                local jpark = bit.band(Sj.flags, F_PARK + F_DEAD) ~= 0 or W.frozen[j]
                local push = (size - d) * (jpark and 1 or 0.5) / d
                kx, ky, kz = kx + dx * push, ky + dy * push, kz + dz * push
                touch = touch + 1
              end
            end
            j = nxt[j]
          end
        end
      end
      local o = i * 9
      if touch > 1 then kx, ky, kz = kx / touch, ky / touch, kz / touch end
      if touch > 0 then any = true; nn = nn + 1; next_c[nn] = i end
      Fb[o + 3], Fb[o + 4], Fb[o + 5] = kx, ky, d3 and kz or 0
    end
    if not any then break end   -- 重なりが 1 つも無ければ、くり返さない
    for ci = 1, nc do
      local i = cand and cand[ci] or ci
      local o = i * 9
      local kx, ky, kz = Fb[o + 3], Fb[o + 4], Fb[o + 5]
      if kx ~= 0 or ky ~= 0 or kz ~= 0 then
        local S = st[i - 1]
        S.px, S.py, S.pz = S.px + kx, S.py + ky, S.pz + kz
        local m = moved[i]
        if not m then moved[i] = { kx, ky, kz } else m[1], m[2], m[3] = m[1] + kx, m[2] + ky, m[3] + kz end
      end
    end
    cand, nc = next_c, nn
  end
  local FR = (IN.fric > 0) and W.fpos or nil
  for i, m in pairs(moved) do
    local S = st[i - 1]
    local ml = sqrt(m[1] * m[1] + m[2] * m[2] + m[3] * m[3])
    if ml > 1e-12 then
      local nx, ny, nz = m[1] / ml, m[2] / ml, m[3] / ml
      local vx, vy, vz = S.dx * S.s + S.wx, S.dy * S.s + S.wy, S.dz * S.s + S.wz
      local vn = vx * nx + vy * ny + vz * nz
      if vn < 0 then vx, vy, vz = vx - vn * nx, vy - vn * ny, vz - vn * nz; set_velocity(S, vx, vy, vz) end
      -- 静止摩擦（v0.15.0）: このラウンドに相手に沿ってずれた分を、押し戻した量 × 摩擦 まで打ち消す（位置で解く摩擦。
      -- 押し合うほどずれにくい）。ずれを打ち消した割合だけ、面に沿う速さも消す
      if FR then
        local o = i * 3
        local tx, ty, tz = S.px - FR[o], S.py - FR[o + 1], d3 and (S.pz - FR[o + 2]) or 0
        local tn = tx * nx + ty * ny + tz * nz
        tx, ty, tz = tx - tn * nx, ty - tn * ny, tz - tn * nz
        local tl = sqrt(tx * tx + ty * ty + tz * tz)
        if tl > 1e-12 then
          local lim = IN.fric * ml
          local f = tl <= lim and 1 or lim / tl
          S.px, S.py, S.pz = S.px - tx * f, S.py - ty * f, S.pz - tz * f
          local vn2 = vx * nx + vy * ny + vz * nz
          local wtx, wty, wtz = vx - vn2 * nx, vy - vn2 * ny, vz - vn2 * nz
          set_velocity(S, vx - wtx * f, vy - wty * f, vz - wtz * f)
        end
      end
    end
  end
end


--[[
世界を 1 ラウンド（シミュレーション時刻 [r h, (r+1) h)）進める。init(e, j) は粒子の始めの値（呼び手 = simulate が作る）。
births: このラウンドに生まれる粒子（誕生時刻がラウンドの中）を足す → 粒子どうしの力 → 全員を 1 刻み進める → 速さを収める → 寿命を見る
]]
local function world_round(P, W, r, init)
  local h = P.h
  local IN = P.inter
  local t0, t1 = r * h, (r + 1) * h
  -- 生まれる粒子
  -- 下限は t0 のほんの少し前で数える（ちょうど t0 に一斉に出した分を、もう出たものとして飛ばさない。v0.16.0 で直した）
  local e0 = max(floor(W.rate.count(t0 - W.wmax - h * 1e-6)) - 1, 0)
  local e1 = floor(W.rate.count(t1 + W.wmax)) + 1
  for e = e0, e1 do
    for j = 0, W.sync - 1 do
      local q, px, py, pz, ux, uy, uz, v0, ac = init(e, j)
      if q and q.b >= t0 and q.b < t1 and W.idx[q.k] == nil then
        if W.active >= W.cap then
          W.over = W.over + 1
        else
          local i = W.n + 1
          if i > W.stcap then
            local cap2 = W.stcap * 2
            local st2 = ffi.new("PRH_State9[?]", cap2)
            ffi.copy(st2, W.st, ffi.sizeof("PRH_State9") * W.n)
            W.st, W.stcap = st2, cap2
          end
          W.n = i
          W.active = W.active + 1
          W.ks[i], W.es[i], W.js[i], W.q[i], W.frozen[i] = q.k, e, j, q, false
          W.idx[q.k] = i
          local S = W.st[i - 1]
          S.key, S.steps, S.flags, S.nb, S.tgt, S.bb = q.k, 0, 0, 0, -1, q.b
          S.px, S.py, S.pz, S.dx, S.dy, S.dz, S.s = px, py, pz, ux, uy, uz, v0
          S.wx, S.wy, S.wz, S.tb, S.td, S.ac, S.tk = 0, 0, 0, -1, -1, ac, -1
          S.roll, S.om = 0, 0
          if P.hist2 then
            for jj = 0, P.hm2 - 1 do P.hstep2[q.slot * P.hm2 + jj] = -1 end
          end
          if P.hist then
            for jj = 0, P.hm - 1 do P.hstep[q.slot * P.hm + jj] = -1 end
            local b0 = q.slot * P.hm
            P.hist[b0 * 3], P.hist[b0 * 3 + 1], P.hist[b0 * 3 + 2] = px, py, pz
            P.hstep[b0] = 0
          end
        end
      end
    end
  end
  if W.n == 0 then return end
  -- 粒子どうしの力（ラウンドの始めの位置から）
  local Fb = inter_forces(W, IN)
  local st = W.st
  -- 静止摩擦（v0.15.0）: 進める前の位置（位置の直しで、このラウンドのずれを測る）
  local FR
  if IN.size > 0 and IN.fric > 0 then
    if not W.fpos or W.fposn < (W.n + 1) * 3 then W.fposn = (W.n + 1) * 3 * 2; W.fpos = ffi.new("double[?]", W.fposn) end
    FR = W.fpos
  end
  for i = 1, W.n do
    if not W.frozen[i] then
      local S = st[i - 1]
      if bit.band(S.flags, F_PARK + F_DEAD) == 0 then
        local o = i * 9
        local kx, ky, kz, ux, uy, uz = Fb[o + 3], Fb[o + 4], Fb[o + 5], Fb[o + 6], Fb[o + 7], Fb[o + 8]
        if kx ~= 0 or ky ~= 0 or kz ~= 0 then S.px, S.py, S.pz = S.px + kx, S.py + ky, S.pz + kz end
        if ux ~= 0 or uy ~= 0 or uz ~= 0 then
          set_velocity(S, S.dx * S.s + S.wx + ux, S.dy * S.s + S.wy + uy, S.dz * S.s + S.wz + uz)
        end
        local q = W.q[i]
        if FR then FR[i * 3], FR[i * 3 + 1], FR[i * 3 + 2] = S.px, S.py, S.pz end
        local gx, gy, gz = q.gx, q.gy, q.gz
        local fx_, fy_, fz_ = Fb[o], Fb[o + 1], Fb[o + 2]
        q.gx, q.gy, q.gz = gx + fx_, gy + fy_, gz + fz_
        advance(P, q, S, S.steps, S.steps + 1)
        q.gx, q.gy, q.gz = gx, gy, gz
        P.steps = P.steps + 1
        -- 群れの速さの範囲
        if (IN.vmax > 0 or IN.vmin > 0) and bit.band(S.flags, F_PARK + F_DEAD) == 0 then
          local vx, vy, vz = S.dx * S.s + S.wx, S.dy * S.s + S.wy, S.dz * S.s + S.wz
          local sp = sqrt(vx * vx + vy * vy + vz * vz)
          local sp2 = sp
          if IN.vmax > 0 and sp2 > IN.vmax then sp2 = IN.vmax end
          if sp2 < IN.vmin then sp2 = IN.vmin end
          if sp2 ~= sp and sp > 1e-9 then set_velocity(S, vx / sp * sp2, vy / sp * sp2, vz / sp * sp2) end
        end
      end
    end
  end
  -- 重ならない: 進めた後の位置を直す
  inter_resolve(W, IN)
  -- 当たる相手（v0.13.0）: 進めた後の位置で、相手の粒子（このラウンドの時刻の位置）から押し出す
  if IN.other and IN.orad > 0 then M.inter_obstacles(W, IN, IN.other(t1)) end
  -- 寿命: 寿命に届いた粒子は止める（凍らせて、出来事のために残す）。凍った粒子は窓を過ぎたら消す
  local keep_after = W.keep_after
  local removed = false
  for i = 1, W.n do
    local S = st[i - 1]
    local q = W.q[i]
    if not W.frozen[i] then
      if bit.band(S.flags, F_DEAD) ~= 0 or (S.steps * h >= q.L and bit.band(S.flags, F_PARK) == 0) then
        W.frozen[i] = true
        W.active = W.active - 1
      end
    end
    if W.frozen[i] and t1 > q.b + q.L + keep_after then removed = true end
  end
  -- 止まって残る粒子が多すぎれば、古いものから凍らせる
  if W.keep_max then
    local parked = 0
    for i = W.n, 1, -1 do
      if not W.frozen[i] and bit.band(st[i - 1].flags, F_PARK) ~= 0 then
        parked = parked + 1
        if parked > W.keep_max then W.frozen[i] = true; W.active = W.active - 1 end
      end
    end
  end
  if removed then
    local m = 0
    W.idx = {}
    for i = 1, W.n do
      local q = W.q[i]
      if not (W.frozen[i] and t1 > q.b + q.L + keep_after) then
        m = m + 1
        if m ~= i then
          ffi.copy(st + (m - 1), st + (i - 1), ffi.sizeof("PRH_State9"))
          W.ks[m], W.es[m], W.js[m], W.q[m], W.frozen[m] = W.ks[i], W.es[i], W.js[i], W.q[i], W.frozen[i]
        end
        W.idx[W.ks[m]] = m
      end
    end
    for i = m + 1, W.n do W.ks[i], W.es[i], W.js[i], W.q[i], W.frozen[i] = nil, nil, nil, nil, nil end
    W.n = m
  end
end

-- 世界をラウンド R の手前（シミュレーション時刻 R h）まで進める。戻るなら、R より前の一番近いチェックポイントか、最初から
local function world_advance(P, W, R, init)
  if W.r > R then
    local best
    for _, C in ipairs(W.cps) do
      if C.r <= R and (not best or C.r > best.r) then best = C end
    end
    W.n, W.active, W.idx, W.ks, W.es, W.js, W.q, W.frozen = 0, 0, {}, {}, {}, {}, {}, {}
    W.r, W.over = 0, 0
    if best then
      if best.n > W.stcap then W.st, W.stcap = ffi.new("PRH_State9[?]", best.n), best.n end
      ffi.copy(W.st, best.st, ffi.sizeof("PRH_State9") * best.n)
      W.n, W.r = best.n, best.r
      for i = 1, best.n do
        local q = init(best.es[i], best.js[i])
        local fz = best.frozen[i]
        if not q then
          -- 始めの値を求め直せない（他レイヤーの形が変わったなど）: 凍らせて残す
          local S = W.st[i - 1]
          q = { k = best.ks[i], b = S.bb, L = 0, gx = 0, gy = 0, gz = 0, slot = bit.band(best.ks[i], P.cache.cap - 1) }
          fz = true
        end
        W.ks[i], W.es[i], W.js[i], W.q[i], W.frozen[i] = best.ks[i], best.es[i], best.js[i], q, fz
        W.idx[best.ks[i]] = i
        if not fz then W.active = W.active + 1 end
      end
      W.restored = (W.restored or 0) + 1
    else
      W.restarts = (W.restarts or 0) + 1
    end
  end
  while W.r < R do
    world_round(P, W, W.r, init)
    W.r = W.r + 1
    if W.r % W.cp_every == 0 and #W.cps < M.INTER_CP then
      local have = false
      for _, C in ipairs(W.cps) do if C.r == W.r then have = true; break end end
      if not have then
        local C = { r = W.r, n = W.n, st = ffi.new("PRH_State9[?]", max(W.n, 1)), ks = {}, es = {}, js = {}, frozen = {} }
        ffi.copy(C.st, W.st, ffi.sizeof("PRH_State9") * W.n)
        for i = 1, W.n do C.ks[i], C.es[i], C.js[i], C.frozen[i] = W.ks[i], W.es[i], W.js[i], W.frozen[i] end
        W.cps[#W.cps + 1] = C
      end
    end
  end
end
M.world_advance = world_advance

-- 分散と急停止をつなぐ: 粒子ごとの つなぐ時間 と、止まってから分散する時刻（q.t_stop と q.t_disp を決めた後に呼ぶ）
-- v0 = 生まれたときの速さ。止まってから分散するとき、分散の速さを決めていなければこの速さで動き出す（止まった粒子は速さ 0 なので）
local function disp_link(P, q, ik, sk, v0)
  q.link1, q.link2, q.link_t = false, false, 0
  local D = P.dlink
  if not D then return end
  q.link_t = max(vary(D.t, D.i, rnd(ik, CH.dlink, sk)), 0.001)
  if D.mode == 1 then
    q.link1 = true
  elseif D.mode == 2 and not D.by_bounce and q.t_stop > 0 then
    q.t_disp = q.t_stop + q.link_t
    if not P.disp.speed_set then q.link2, q.disp_speed = true, v0 or 0 end
  end
end
M.disp_link = disp_link

-- 最初に跳ね返る年齢の先読み（v0.14.0。フィルター・色の「跳ね返るまで」）: 状態 S の写しを、跳ね返るか寿命 L まで 1 刻みずつ進める。
-- 軌跡の履歴は汚さない（先読みの間は書かない）。跳ね返らなければ -1
function M.first_bounce(P, q, S, L)
  if bit.band(S.flags, F_BNC) ~= 0 and S.tb >= 0 then return S.tb end
  local T = ffi.new("PRH_State9")
  ffi.copy(T, S, ffi.sizeof("PRH_State9"))
  local h1, h2 = P.hist, P.hist2
  P.hist, P.hist2 = nil, nil
  local last = floor(L / P.h)
  local out = -1
  for i = T.steps, last - 1 do
    advance(P, q, T, i, i + 1)
    if bit.band(T.flags, F_BNC) ~= 0 then out = T.tb; break end
    if bit.band(T.flags, F_DEAD + F_PARK) ~= 0 then break end
  end
  P.hist, P.hist2 = h1, h2
  return out
end

-- 状態の置き場。呼び手がオブジェクトごとに持つ表 cache に入れる。設定（sig）が変われば作り直す
function M.prepare_cache(cache, sig, alive_hint, hist_m, hist2_m)
  local cap = 64
  while cap < alive_hint * 2 + 64 do cap = cap * 2 end
  hist_m, hist2_m = hist_m or 0, hist2_m or 0
  if cache.sig ~= sig or not cache.st or cache.cap < cap or (cache.hm or 0) ~= hist_m or (cache.hm2 or 0) ~= hist2_m then
    cache.st = ffi.new("PRH_State9[?]", cap)
    for i = 0, cap - 1 do cache.st[i].key = -1 end
    cache.cap = cap
    cache.sig = sig
    cache.world = nil
    cache.fb = nil
    cache.child = nil
    cache.resets = (cache.resets or 0) + 1
    cache.hm = hist_m
    if hist_m > 0 then
      cache.hist = ffi.new("double[?]", cap * hist_m * 3)
      cache.hstep = ffi.new("int32_t[?]", cap * hist_m)
      for i = 0, cap * hist_m - 1 do cache.hstep[i] = -1 end
    else
      cache.hist, cache.hstep = nil, nil
    end
    cache.hm2 = hist2_m
    if hist2_m > 0 then
      cache.hist2 = ffi.new("double[?]", cap * hist2_m * 3)
      cache.hstep2 = ffi.new("int32_t[?]", cap * hist2_m)
      for i = 0, cap * hist2_m - 1 do cache.hstep2[i] = -1 end
    else
      cache.hist2, cache.hstep2 = nil, nil
    end
  end
  return cache
end

-- 閉じた式の位置と速度（加速度が負なら止まったところで止める。逆には進まない）
local function cf_pos(px, py, pz, ux, uy, uz, v0, ac, gx, gy, gz, tau)
  local s, v
  if ac < 0 and v0 > 0 then
    local ts = v0 / -ac
    local a2 = min(tau, ts)
    s = v0 * a2 + 0.5 * ac * a2 * a2
    v = tau < ts and v0 + ac * tau or 0
  else
    s = v0 * tau + 0.5 * ac * tau * tau
    v = v0 + ac * tau
  end
  local hx = 0.5 * tau * tau
  return px + ux * s + gx * hx, py + uy * s + gy * hx, pz + uz * s + gz * hx, ux * v + gx * tau, uy * v + gy * tau
end
M.cf_pos = cf_pos

-- 円運動（閉じた式）。中心のまわりを、平面の中で回る
local function orbit_pos(O, q, tau)
  local w, al = q.ow, O.acc
  local th
  if O.nonneg and al ~= 0 and w ~= 0 and (w > 0) ~= (al > 0) then
    local tt = min(tau, -w / al)
    th = q.th0 + (w * tt + 0.5 * al * tt * tt) * RAD
  else
    th = q.th0 + (w * tau + 0.5 * al * tau * tau) * RAD
  end
  local r = max(q.r0 + q.ovr * tau, 0)
  if O.lim > 0 then r = min(r, O.lim) end
  local x, y, z = r * cos(th), r * sin(th), 0
  if O.ax ~= 0 then
    local c, s = cos(O.ax * RAD), sin(O.ax * RAD)
    y, z = y * c - z * s, y * s + z * c
  end
  if O.ay ~= 0 then
    local c, s = cos(O.ay * RAD), sin(O.ay * RAD)
    x, z = x * c + z * s, -x * s + z * c
  end
  return q.cx + x, q.cy + y, q.cz + z
end
M.orbit_pos = orbit_pos

-- ゆらぎ（挙動）の位置のずれ。進行方向基準なら X を進む向きと直角、Y を進む向きにする
function M.wobble(WB, spd, fa, p1, p2, p3, a, vx, vy)
  local ox = WB.x * fa * sin((spd * a + p1) * RAD)
  local oy = WB.y * fa * sin((spd * a + p2) * RAD)
  local oz = WB.z * fa * sin((spd * a + p3) * RAD)
  if WB.rel then
    local vl = sqrt(vx * vx + vy * vy)
    local ax_, ay_ = 0, 1
    if vl > 1e-9 then ax_, ay_ = vx / vl, vy / vl end
    ox, oy = -ay_ * ox + ax_ * oy, ax_ * ox + ay_ * oy
  end
  return ox, oy, oz
end

-- 位置のずれ（挙動）: 粒子の番号の並びに沿って、間隔ごとに置いた乱数をつないだ値（-1..1 前後）
local function jitter(J, k, axis, sk)
  local n = J.n
  local seg = floor(k / n)
  local f = (k - seg * n) / n
  local function node(i) return rnd(i, CH.jit + axis, sk) * 2 - 1 end
  if J.curve then
    local p0, p1, p2, p3 = node(seg - 1), node(seg), node(seg + 1), node(seg + 2)
    local f2, f3 = f * f, f * f * f
    local c = 0.5 * ((2 * p1) + (-p0 + p2) * f + (2 * p0 - 5 * p1 + 4 * p2 - p3) * f2 + (-p0 + 3 * p1 - 3 * p2 + p3) * f3)
    -- 滑らかさ（v0.14.0。原作のパターン 0〜101）: 1 で曲線のまま、0 で折れ線、間は 2 つを混ぜる
    local sm = J.smooth or 1
    if sm >= 1 then return c end
    local lin = p1 + (p2 - p1) * f
    return lin + (c - lin) * sm
  end
  local a = node(seg)
  return a + (node(seg + 1) - a) * f
end
M.jitter = jitter

----------------------------------------------------------------------------- 本体

-- 拡張（風・ノイズ場・集結点・跳ね返り・挙動・分散と停止・円運動・時間の付け替え）の設定をまとめる。
-- 戻り値: P（積分の設定。積分が要らなければ nil）、O（円運動）、J（位置のずれ）、WB（ゆらぎ）、TW（時間の窓）
-- 円運動は閉じた式で動かすので、円運動があるときは積分しない（風などは効かない）
function M.prepare_motion(cfg)
  local X = cfg.ext or {}
  local ev = cfg.ext_val or function(kind, var) return X[kind][var] end
  local wind, nz, att, bnc, beh, disp, orb, tim = X.wind, X.noise, X.att, X.bnc, X.beh, X.disp, X.orbit, X.time
  local inter = X.inter
  local O, J, WB, TW
  if orb then
    local pt = orb.pos or {}
    O = { center = orb.center, px = pt[1] or 0, py = pt[2] or 0, pz = pt[3] or 0, w = orb.w, acc = orb.acc,
          nonneg = orb.nonneg, vr = orb.vr, lim = orb.lim, ax = orb.ax, ay = orb.ay, i_w = orb.i_w, i_vr = orb.i_vr,
          rx = orb.rx, ry = orb.ry, rz = orb.rz }
  end
  -- パス（v0.7.0）: 円運動と同じく動きを置き換える。両方あればパスを使う（本体が警告する）
  local PA
  if X.path then
    local pt = M.prepare_path(X.path)
    if pt and pt.total > 0 then
      local pa = X.path
      PA = { pt = pt, speed = pa.speed or 0, accel = pa.accel or 0, width = pa.width or 0, pend = floor(pa.pend or 0),
             start = floor(pa.start or 0), i_speed = pa.i_speed or 0 }
      O = nil
    end
  end
  if beh then
    if beh.jx ~= 0 or beh.jy ~= 0 or beh.jz ~= 0 then
      J = { n = max(floor(beh.jn), 1), curve = beh.jpat == 1, x = beh.jx, y = beh.jy, z = beh.jz,
            smooth = min(max((beh.jsmooth or 100) / 100, 0), 1) }
    end
    if beh.yx ~= 0 or beh.yy ~= 0 or beh.yz ~= 0 then
      WB = { speed = beh.ys, x = beh.yx, y = beh.yy, z = beh.yz, rel = beh.yrel, i_speed = beh.i_ys, i_amp = beh.i_ya }
    end
  end
  if tim then
    TW = {}
    if tim.emit_win and #tim.emit_win >= 1 then TW.emit_win = tim.emit_win end
    if tim.act_win and #tim.act_win >= 1 then TW.act_win, TW.act_rel = tim.act_win, tim.act_rel end
    if not TW.emit_win and not TW.act_win then TW = nil end
  end
  local sus = beh and beh.sp > 0
  local dispon = disp and (disp.dt > 0 or disp.dbounce or ((disp.link or 0) == 2 and disp.stop > 0))
  local stopon = disp and disp.stop > 0
  local RT = cfg.rt or {}
  local vec = beh and RT.vfn
  local fld = X.field and X.field.list
  if fld and #fld == 0 then fld = nil end
  if O or PA or not (wind or nz or att or bnc or dispon or stopon or sus or vec or fld or inter) then return nil, O, J, WB, TW, PA end
  local P = { h = 1 / (cfg.fps * max(floor(cfg.precision or 2), 1)), steps = 0 }
  -- 積分法と刻みの倍率（v0.19.0。本体の「積分法」「刻みの倍率」）。既定（半陰的オイラー・1 倍）は今までと同じ計算
  local im = floor(tonumber(cfg.integ) or 0)
  if im == 1 or im == 2 then P.integ = im end
  local hmul = tonumber(cfg.hmul) or 1
  if hmul > 1 then P.h = P.h * min(hmul, 20) end
  if fld then
    P.fields = {}
    for i, F in ipairs(fld) do
      P.fields[i] = M.prepare_field(F)
      P.fields[i].h = P.h
      P.fields[i].img = RT.fimg and RT.fimg[i]
      if P.fields[i].type == 3 then P.dipole = true end
    end
    P.obj_time = cfg.obj_time
  end
  if vec then
    local errs = cfg.errs
    P.vec = function(a)
      local ok, x, y, z = pcall(vec, a)
      if not ok then
        if errs then errs.vector = errs.vector or tostring(x) end
        return 0, 0, 0
      end
      return tonumber(x) or 0, tonumber(y) or 0, tonumber(z) or 0
    end
  end
  if wind then
    local pt = wind.pt or {}
    P.wind = { point = wind.use_point, px = pt[1] or 0, py = pt[2] or 0, pz = pt[3] or 0, range = max(wind.range, 1),
               drag = wind.drag, indiv = wind.i_wind, val = function(var, t) return ev("wind", var, t) end,
               moving = next(wind.mv or {}) ~= nil, cx = wind.wx, cy = wind.wy, cz = wind.wz }
  end
  if nz then
    P.noise = { inv = 1 / max(nz.scale, 1), speed = nz.speed, z = nz.z, start = nz.start, fade = nz.fade, indiv = nz.i_noise,
                seed = bit.tobit(floor(nz.seed) * 7919 + 17), seed2 = bit.tobit(floor(nz.seed) * 7919 + 977),
                ox = 0.1 + 0.8 * rnd(floor(nz.seed), CH.noise + 100, 0), oy = 0.1 + 0.8 * rnd(floor(nz.seed), CH.noise + 101, 0),
                val = function(var, t) return ev("noise", var, t) end,
                moving = next(nz.mv or {}) ~= nil, cstr = nz.str }
    -- 元のノイズ（v0.19.0）: 0 = 値ノイズ（今まで）/ 1 = 勾配ノイズ / 2 = ブロック（補間なし）
    local nk = floor(tonumber(nz.ntype) or 0)
    if nk == 1 or nk == 2 then P.noise.kind = nk end
  end
  if att then
    local np = min(max(floor(att.n), 1), floor(#att.pts / 3))
    if np >= 1 then
      P.att = { mode = att.mode, n = np, pts = att.pts, rel = att.rel, start = att.start, i_start = att.i_start,
                strength = att.strength, speed = att.speed, arrive = att.arrive, radius = att.radius,
                interval = max(att.interval, 0.001), xy = att.xy, exy = att.exy, z = att.z, ez = att.ez, obj = cfg.obj_time }
    end
  end
  if bnc then
    local sp = bnc.spos or {}
    P.bnc = { bx = bnc.bx, by = bnc.by, bz = bnc.bz, xmin = bnc.xmin, xmax = bnc.xmax, ymin = bnc.ymin, ymax = bnc.ymax,
              zmin = bnc.zmin, zmax = bnc.zmax, ex = bnc.ex, ey = bnc.ey, ez = bnc.ez, sph = bnc.sph,
              cx = sp[1] or 0, cy = sp[2] or 0, cz = sp[3] or 0, srad = max(bnc.srad, 1), se = bnc.se,
              irr = bnc.irr, irrp = bnc.irrp, irra = bnc.irra, shape = RT.bshape, fric = bnc.fric or 0,
              finite = (bnc.finite or 0) == 1, h = P.h, ctime = bnc.ctime or 0, crel = bnc.crel, obj = cfg.obj_time,
              cbx = bnc.cbx or 0, cby = bnc.cby or 0, cbz = bnc.cbz or 0, csph = bnc.csph or 0,
              roll = bnc.roll or nil, rrad = max(bnc.rrad or 10, 1e-3) }
  end
  if dispon then
    P.disp = { by_bounce = disp.dbounce, time = disp.dbounce and 0 or disp.dt, i_time = disp.i_dt, xy = disp.dxy, z = disp.dz,
               speed_set = disp.dspd == 1, speed = disp.dv, acc_set = disp.dacc_on, acc = disp.dac }
  end
  if stopon then P.stop = { time = disp.stop, i_time = disp.i_stop } end
  -- 分散と急停止をつなぐ（v0.14.0。原作の分散停止リンク）: 1 = 分散してから つなぐ時間 で止める / 2 = 止まってから つなぐ時間 で分散する
  if disp and (disp.link or 0) ~= 0 and dispon then
    P.dlink = { mode = disp.link, t = max(disp.link_t or 1, 0.001), i = disp.i_link or 0, by_bounce = disp.dbounce }
  end
  if sus then
    local x, y, z = beh.sdx, beh.sdy, beh.sdz
    local l = sqrt(x * x + y * y + z * z)
    if l > 1e-9 then x, y, z = x / l, y / l, z / l else x, y, z = 0, -1, 0 end
    P.sus = { interval = max(beh.si, 0.001), prob = beh.sp, speed = beh.ssp, random = beh.sdir == 1, z3 = beh.sz3,
              x = x, y = y, z = z }
  end
  if TW and TW.act_win then P.act = { win = TW.act_win, rel = TW.act_rel, obj = cfg.obj_time } end
  if inter then
    P.inter = {
      r = max(inter.r, 1), rep = inter.rep, size = max(inter.size, 0), att = inter.att, fric = min(max((inter.fric or 0) / 100, 0), 1),
      sep = inter.sep / 100, ali = inter.ali / 100, coh = inter.coh / 100, view = cos(min(max(inter.view, 0), 180) * RAD),
      vmin = max(inter.vmin, 0), vmax = max(inter.vmax, 0), d3 = inter.d3,
      other = RT.inter_other, orad = max(inter.orad or 0, 0), orep = inter.orep or 0,
      max = max(floor(inter.imax or M.INTER_MAX), 1),
    }
  end
  local TR = cfg.trail
  local hm, hstride = 0, 1
  if TR and TR.n > 0 then
    hstride = max(floor(TR.dt / P.h + 0.5), 1)
    hm = TR.n + 1
  end
  -- 止まったら残す（跳ね返りか分散と停止のどちらかで選ぶ）
  local kp = (bnc and bnc.keep_stop) and bnc or ((disp and disp.keep_stop) and disp or nil)
  if kp then P.keep = { speed = max(kp.keep_speed or 20, 0), max = max(floor(kp.keep_max or 500), 1) } end
  -- 子粒子「動いている間」: 親の少し前の位置の履歴（間隔ごと、子の生存時間ぶん）
  local CD = cfg.child
  local hm2, hstride2 = 0, 1
  if CD and CD.event == 3 then
    hstride2 = max(floor(CD.interval / P.h + 0.5), 1)
    hm2 = min(ceil(CD.life / (hstride2 * P.h)) + 2, 64)
  end
  P.cache = M.prepare_cache(cfg.cache or {}, (cfg.sig or "") .. "|h" .. hm .. "/" .. hstride .. "|c" .. hm2 .. "/" .. hstride2,
                            cfg.alive_hint or 1000, hm, hm2)
  if hm > 0 then P.hist, P.hstep, P.hm, P.hstride = P.cache.hist, P.cache.hstep, hm, hstride end
  if hm2 > 0 then P.hist2, P.hstep2, P.hm2, P.hstride2 = P.cache.hist2, P.cache.hstep2, hm2, hstride2 end
  return P, O, J, WB, TW
end


--[[
拡大率・透過率（v0.16.0 に simulate から切り出した。焼き付けを読んだ後に、今の設定で計算し直すのにも使う）。
tb・td = 最初に跳ね返った年齢・分散した年齢（無ければ huge）。戻り値: 拡大率（倍）, 不透明度（0..1）
]]
local function zoom_alpha_setup(cfg, zoal, indiv, track, now)
  local TC_ = cfg.track_const or {}
  return { zoal = zoal, indiv = indiv, track = track, obj_now = zoal and zoal.by_object and cfg.obj_time(now) or 0,
           z0 = TC_.zoom0, z1 = TC_.zoom1, a0 = TC_.alpha0, a1 = TC_.alpha1 }
end
local function zoom_alpha(ZA, k, ik, sk, b, L, age_e, tb, td)
  local zoal, indiv, track = ZA.zoal, ZA.indiv, ZA.track
  local z0, z1, a0, a1
  if zoal then
    z0, z1, a0, a1 = zoal.zoom0, zoal.zoom1, zoal.alpha0, zoal.alpha1
  else
    z0, z1, a0, a1 = (ZA.z0 or track("zoom0", b)), (ZA.z1 or track("zoom1", b)), (ZA.a0 or track("alpha0", b)), (ZA.a1 or track("alpha1", b))
  end
  if indiv then
    local rz_ = rnd(ik, CH.zoom, sk)
    local ra_ = rnd(ik, CH.alpha, sk)
    z0, z1 = vary(z0, indiv.zoom, rz_), vary(z1, indiv.zoom, rz_)
    a0, a1 = vary(a0, indiv.alpha, ra_), vary(a1, indiv.alpha, ra_)
  end
  local zm, tr
  if zoal then
    local Lend, xk = L, age_e
    if zoal.by_object then Lend, xk = zoal.total, ZA.obj_now end
    zm = eval_keys(zoal.zm, z0, z1, Lend, xk, zoal.zmode, zoal.zb, tb, zoal.zd, td, zoal.zshape)
    tr = eval_keys(zoal.am, a0, a1, Lend, xk, zoal.amode, zoal.ab, tb, zoal.ad, td, zoal.ashape)
  else
    local f = age_e / L
    zm = z0 + (z1 - z0) * f
    tr = a0 + (a1 - a0) * f
  end
  return max(zm, 0) / 100, min(max(1 - tr / 100, 0), 1)
end

--[[
焼き付けを読んだ粒子の拡大率・透過率を、今の設定で計算し直す（v0.16.0。見た目だけを変えたときも焼き付けを使い続けるため）。
cfg は simulate に渡すものと同じ。位置・回転・年齢・寿命・跳ね返りの年齢は焼き付けのまま。見た目の調整・音の大きさ・格子の型抜きも掛け直す
]]
function M.relook(res, cfg)
  local zoal = M.prepare_zoal(cfg.zoal)
  local indiv = cfg.indiv
  local same_group = indiv and indiv.same_group
  local track = cfg.track
  if cfg.track_const then
    local TC, track0 = cfg.track_const, track
    track = function(name, t)
      local v = TC[name]
      if v ~= nil then return v end
      return track0(name, t)
    end
  end
  local ZA = zoom_alpha_setup(cfg, zoal, indiv, track, cfg.now or 0)
  local CD = cfg.child
  local sk0, rsk = cfg.sk, res.rsk
  for i = 0, res.n - 1 do
    local k, b, L, age = res.k[i], res.b[i], res.life[i], res.age[i]
    if CD and k >= 0x30000000 then
      res.zoom[i] = CD.zoom / 100
      res.alpha[i] = min(max(1 - CD.alpha1 / 100 * age / CD.life, 0), 1)
    else
      local sk = (res.rs and res.rs[i] == 1 and rsk) or sk0
      local ik = same_group and res.e[i] or k
      local tb = (res.tb and res.tb[i] >= 0) and res.tb[i] or huge
      local td = (res.td and res.td[i] >= 0) and res.td[i] or huge
      res.zoom[i], res.alpha[i] = zoom_alpha(ZA, k, ik, sk, b, L, min(age, L), tb, td)
    end
  end
  M.post(res, cfg, true)
end
M.zoom_alpha = zoom_alpha

--[[
cfg（呼び手が作る）:
  now       シミュレーション時刻（秒）
  t_end     シミュレーション時刻での終わり
  t_vanish  終了時に消えるの終わり（シミュレーション時刻。無ければ t_end。ループでは今の時刻からオブジェクトの残りの時間だけ先）
  sk        M.seed_key(...)
  sync      同時発生数（整数）
  rate      M.rate_table / M.rate_const
  life_max  生存時間の上限（窓の計算に使う。個別微調整の幅を含めて呼び手が出す）
  track(name, t)  トラックバーの値（放出時の値を使うなら t = 誕生時刻、使わないなら t = now を呼び手が選ぶ）
                  name: speed dir spread zdir zspread life accel gx gy gz rx0 ry0 rz0 vrx vry vrz zoom0 zoom1 alpha0 alpha1
  rot_random  0=指定どおり / 1=Z軸だけランダム / 2=XYZ ランダム
  revrot      逆回転
  vanish_at_end  終了時に消える: 生まれてから消えるまで（子粒子を出すなら、子が消えるまで）が終わりに収まる粒子だけを生む。
                 寿命は縮めない。止まったら残すの粒子も寿命で消す（残さない）
  obj_time(t) シミュレーション時刻 → オブジェクトの時刻（回転の「放射時角度増減」と「オブジェクト基準の拡大率・透過率」に使う）
  emit / rot / zoal / indiv   拡張の設定（無ければ nil）
]]
function M.simulate(cfg)
  local now, sk, sync = cfg.now, cfg.sk, max(floor(cfg.sync or 1), 1)
  local track, rate = cfg.track, cfg.rate
  -- 動かないトラックバーの値は表から直接読む（呼び手の track を毎回呼ばない。v0.16.0）
  if cfg.track_const then
    local TC, track0 = cfg.track_const, track
    track = function(name, t)
      local v = TC[name]
      if v ~= nil then return v end
      return track0(name, t)
    end
  end
  local indiv, rot, zoal = cfg.indiv, cfg.rot, M.prepare_zoal(cfg.zoal)
  local vmul = zoal and zoal.vmul
  local emit = M.prepare_emit(cfg.emit, cfg.rt, cfg)
  local same_group = indiv and indiv.same_group
  local p_rate = indiv and indiv.rate or 0
  local P, O, J, WB, TW, PA = M.prepare_motion(cfg)
  if P then P.vmul = vmul end
  -- 個別微調整の「乱数を変える番号」の粒子は、乱数の鍵を変える
  local RS, RSK
  if indiv and indiv.reseed and #indiv.reseed > 0 then
    RS = {}
    for _, v in ipairs(indiv.reseed) do RS[floor(tonumber(v) or -1)] = true end
    RSK = M.reseed_key(sk, indiv.salt or 1)
  end
  local emit_win = TW and TW.emit_win
  local q = {}

  -- 窓: 今生きうる粒子の誕生時刻は [now - life_max - ゆらぎ, now]
  local jitter = 0
  if p_rate ~= 0 then
    local r = max(rate.rate(now), 1e-6)
    jitter = p_rate > 0 and (1 / r) * p_rate / 100 or -p_rate
  end
  -- 下限の時刻が 0 以下なら最初から数える（count(0) は時刻 0 の一斉に出す分を含むので、それを飛ばさない）
  -- 子粒子は親が消えた後も子の生存時間だけ残るので、その分の親も見る。止まったら残すは、窓より前の 残す数 個も見る
  local CD = cfg.child
  -- 終了時に消える（v0.19.1）: 誕生時刻 ＋ 寿命 ＋ 子の寿命 が VT を越える粒子は生まない。止まったら残すは効かせない
  local VT = cfg.vanish_at_end and (cfg.t_vanish or cfg.t_end) or nil
  local VTAIL = CD and CD.life or 0
  local KP = P and not VT and P.keep or nil
  local LOOPE = cfg.loop_e
  -- 子粒子に動きの拡張を掛ける（v0.15.0）: 親と同じ風・ノイズ場・力場・跳ね返りで、子も積分する。集結点・分散と停止・
  -- 時間の窓・止まったら残す・挙動不審・速度倍率・速度の関数・履歴は子には掛けない。状態は子の番号ごとに取っておく
  local CP, cq, cgen
  if CD and CD.ext and P and not O and not PA then
    CP = {}
    for kk, vv in pairs(P) do CP[kk] = vv end
    CP.att, CP.disp, CP.act, CP.keep, CP.sus, CP.vmul, CP.vec, CP.hist, CP.hist2, CP.stop, CP.dlink = nil, nil, nil, nil, nil, nil, nil, nil, nil, nil, nil
    -- 子の設定が変われば、取っておいた子の状態を捨てる（親の設定が変われば置き場ごと作り直される）
    local csig = table.concat({ CD.event, CD.n, CD.interval, CD.speed, CD.spread, CD.inherit, CD.life, CD.gy, CD.drag }, ",")
    if not P.cache.child or P.cache.child.csig ~= csig then P.cache.child = { st = {}, seen = {}, gen = 0, csig = csig } end
    CP.cst = P.cache.child
    CP.cst.gen = CP.cst.gen + 1
    cgen = CP.cst.gen
    local ff = {}
    if P.fields then for fi = 1, #P.fields do ff[fi] = 1 end end
    cq = { fwind = 1, fnoise = 1, ff = ff, t_stop = 0, t_disp = 0, disp_speed = 0, link1 = false, link2 = false, slot = 0,
           gx = 0, gy = CD.gy, gz = 0, drag = CD.drag > 0 and CD.drag or nil }
  end
  local t_lo = now - cfg.life_max - jitter - (CD and CD.life or 0)
  local e_lo = t_lo <= 0 and 0 or max(floor(rate.count(t_lo)) - 1, 0)
  if KP then e_lo = max(e_lo - ceil(KP.max / sync), 0) end
  local e_hi = floor(rate.count(now + jitter))
  local cev = {}

  -- 粒子どうし（v0.12.0）: 粒子 (e, j) の始めの値（下の段の書き方と同じ。積分の値だけ）。出せない粒子は nil
  local WORLD
  if P and P.inter then
    local function init(e, j)
      local te = rate.time_of(e)
      if te == huge then return nil end
      local ew = e
      if LOOPE then ew = e % LOOPE end
      local k = ew * sync + j
      local ik = same_group and ew or k
      local skp = (RS and RS[k]) and RSK or sk
      local b = te
      if p_rate ~= 0 then
        local r = max(rate.rate(te), 1e-6)
        local w = p_rate > 0 and (1 / r) * p_rate / 100 or -p_rate
        b = max(te + (rnd(ew, CH.birth, skp) * 2 - 1) * w, 0)
      end
      if emit_win and not in_window(emit_win, cfg.obj_time(b)) then return nil end
      local L = track("life", b)
      if indiv and indiv.life ~= 0 then L = vary(L, indiv.life, rnd(ik, CH.life, skp)) end
      L = max(L, 0.001)
      if VT and b + L + VTAIL > VT + 1e-9 then return nil end
      local px, py, pz, odir, ozd, avx, avy, avz = emit_pos(emit, (emit and emit.same_pos) and ew or k, b, skp)
      if px == nil then return nil end
      local dir = (odir or track("dir", b)) + (rnd(k, CH.dir, skp) * 2 - 1) * track("spread", b)
      local zd = (ozd or track("zdir", b)) + (rnd(k, CH.zdir, skp) * 2 - 1) * track("zspread", b)
      local dxy, dz = cos(zd * RAD), sin(zd * RAD)
      local ux, uy, uz = sin(dir * RAD) * dxy, cos(dir * RAD) * dxy, dz
      local v0 = track("speed", b)
      local ac = track("accel", b)
      local gx, gy, gz = track("gx", b), track("gy", b), track("gz", b)
      if indiv then
        v0 = vary(v0, indiv.speed, rnd(ik, CH.speed, skp))
        ac = vary(ac, indiv.accel, rnd(ik, CH.accel, skp))
        gx = vary(gx, indiv.gx, rnd(ik, CH.gx, skp))
        gy = vary(gy, indiv.gy, rnd(ik, CH.gy, skp))
        gz = vary(gz, indiv.gz, rnd(ik, CH.gz, skp))
      end
      if avx and (avx ~= 0 or avy ~= 0 or avz ~= 0) then
        local wx_, wy_, wz_ = ux * v0 + avx, uy * v0 + avy, uz * v0 + avz
        local sp = sqrt(wx_ * wx_ + wy_ * wy_ + wz_ * wz_)
        if sp > 1e-9 then ux, uy, uz, v0 = wx_ / sp, wy_ / sp, wz_ / sp, sp end
      end
      if J then
        px = px + M.jitter(J, k, 1, skp) * J.x
        py = py + M.jitter(J, k, 2, skp) * J.y
        pz = pz + M.jitter(J, k, 3, skp) * J.z
      end
      local q = { k = k, sk = skp, b = b, L = L, slot = bit.band(k, P.cache.cap - 1), gx = gx, gy = gy, gz = gz }
      if P.fields then
        q.ff = {}
        for fi = 1, #P.fields do q.ff[fi] = vary(100, P.fields[fi].i_str, rnd(ik, CH.fld + fi, skp)) / 100 end
      end
      q.fwind = P.wind and vary(100, P.wind.indiv, rnd(ik, CH.wind, skp)) / 100 or 1
      q.fnoise = P.noise and vary(100, P.noise.indiv, rnd(ik, CH.noise, skp)) / 100 or 1
      q.t_disp = (P.disp and P.disp.time > 0) and max(vary(P.disp.time, P.disp.i_time, rnd(ik, CH.tdisp, skp)), 0) or 0
      q.t_stop = (P.stop and P.stop.time > 0) and max(vary(P.stop.time, P.stop.i_time, rnd(ik, CH.tstop, skp)), 1e-9) or 0
      q.disp_speed = P.disp and P.disp.speed or 0
      disp_link(P, q, ik, skp, v0)
      if P.att then
        q.t_att = vary(P.att.start, P.att.i_start, rnd(ik, CH.tatt, skp))
        q.att_xy = P.att.xy + (rnd(k, CH.attxy, skp) * 2 - 1) * P.att.exy
        q.att_z = P.att.z + (rnd(k, CH.attz, skp) * 2 - 1) * P.att.ez
      end
      return q, px, py, pz, ux, uy, uz, v0, ac
    end
    local W = P.cache.world
    if not W then
      local cpr = max(ceil(max(cfg.t_end or 0, 1) / M.INTER_CP / P.h), ceil(1 / P.h))
      W = { r = 0, n = 0, active = 0, ks = {}, es = {}, js = {}, q = {}, frozen = {}, idx = {}, cps = {}, over = 0,
            st = ffi.new("PRH_State9[?]", 256), stcap = 256, cp_every = cpr }
      P.cache.world = W
    end
    W.rate, W.sync, W.cap, W.h = rate, sync, P.inter.max or M.INTER_MAX, P.h
    W.wmax = jitter
    W.keep_after = (CD and CD.life or 0) + 2 * P.h
    W.keep_max = KP and KP.max or nil
    -- 進めるのは今の 1 つ手前のラウンドまで（次の刻みの始めが今より前に収まる。端数は下の段で粒子ごとに進める）
    world_advance(P, W, floor(now / P.h) - 1, init)
    WORLD = W
  end

  local cap = M.MAX_PARTICLES
  local n = 0
  local out = {
    x = ffi.new("double[?]", cap), y = ffi.new("double[?]", cap), z = ffi.new("double[?]", cap),
    rx = ffi.new("double[?]", cap), ry = ffi.new("double[?]", cap), rz = ffi.new("double[?]", cap),
    vx = ffi.new("double[?]", cap), vy = ffi.new("double[?]", cap),
    zoom = ffi.new("double[?]", cap), alpha = ffi.new("double[?]", cap),
    k = ffi.new("int32_t[?]", cap), age = ffi.new("double[?]", cap),
    b = ffi.new("double[?]", cap), life = ffi.new("double[?]", cap), e = ffi.new("int32_t[?]", cap),
  }
  if cfg.audio or (cfg.gather and cfg.gather.src == 1) then
    out.ex, out.ey, out.ez = ffi.new("double[?]", cap), ffi.new("double[?]", cap), ffi.new("double[?]", cap)
  end
  -- 自分の画像を並べる: 粒子ごとのマス（切り抜く所 = 出た所 に使う）
  local CELLS = emit and emit.shape == 8 and emit.cells
  if CELLS and CELLS.n > 0 then
    out.cu, out.cv = ffi.new("double[?]", cap), ffi.new("double[?]", cap)
    out.cw, out.ch = ffi.new("double[?]", cap), ffi.new("double[?]", cap)
    out.ci = ffi.new("int32_t[?]", cap)
    out.cells = CELLS
  end
  -- 出た所の色（v0.9.0）: 出た所（他レイヤーの形のマス・自分の画像のマス）の色を、段の色（メディアンカット）のどれかにする
  local QSRC, QPAL
  if cfg.colsrc and emit and ((emit.shape == 6 and emit.mask) or (CELLS and CELLS.n > 0)) then
    QSRC = emit.shape == 6 and emit.mask or CELLS
    local nl = max(floor(cfg.colsrc), 1)
    QSRC.qpal = QSRC.qpal or {}
    QPAL = QSRC.qpal[nl]
    if not QPAL then
      local cols, wts = {}, {}
      if emit.shape == 6 then
        for _, c in ipairs(QSRC.cells) do cols[#cols + 1] = QSRC.col[c]; wts[#wts + 1] = QSRC.a[c] end
      else
        for i = 1, QSRC.n do cols[#cols + 1] = QSRC.col[i]; wts[#wts + 1] = QSRC.cw[i] * QSRC.ch[i] end
      end
      QPAL = M.median_cut(cols, wts, nl)
      QSRC.qpal[nl] = QPAL
    end
    out.cl = ffi.new("uint8_t[?]", cap)
    -- 出た所の色そのもの（v0.18.0。-1 = 出た所が無い）。色差を選んだとき、描く側で近い色を選び直す（color_level）
    out.csrc = ffi.new("int32_t[?]", cap)
    out.qpal = QPAL
  end
  if RS then
    out.rs = ffi.new("uint8_t[?]", cap)
    out.rsk = RSK
  end
  -- フィルター・色の「跳ね返ってから / 跳ね返るまで」（v0.14.0）: 最初に跳ね返った年齢と、その先読み（-1 は跳ね返らない）
  if cfg.need_tb then out.tb, out.tbp = ffi.new("double[?]", cap), ffi.new("double[?]", cap) end
  if cfg.need_td then out.td = ffi.new("double[?]", cap) end
  local TR = cfg.trail
  local TN = (TR and TR.n > 0) and TR.n or 0
  if TN > 0 then
    out.tn = TN
    out.tx = ffi.new("double[?]", cap * TN)
    out.ty = ffi.new("double[?]", cap * TN)
    out.tz = ffi.new("double[?]", cap * TN)
    out.tok = ffi.new("uint8_t[?]", cap * TN)
  end
  local overflow = 0
  -- 動かないトラックバーの値（ループの中で毎回 track を呼ばない。v0.16.0）
  local TC_ = cfg.track_const or {}
  local TC_life, TC_dir, TC_spread, TC_zdir, TC_zspread = TC_.life, TC_.dir, TC_.spread, TC_.zdir, TC_.zspread
  local TC_speed, TC_accel, TC_gx, TC_gy, TC_gz = TC_.speed, TC_.accel, TC_.gx, TC_.gy, TC_.gz
  local TC_rx0, TC_ry0, TC_rz0, TC_vrx, TC_vry, TC_vrz = TC_.rx0, TC_.ry0, TC_.rz0, TC_.vrx, TC_.vry, TC_.vrz
  local TC_zoom0, TC_zoom1, TC_alpha0, TC_alpha1 = TC_.zoom0, TC_.zoom1, TC_.alpha0, TC_.alpha1
  local ZA = zoom_alpha_setup(cfg, zoal, indiv, track, now)
  -- 新しい粒子から数えて上限で打ち切る（古い粒子を落とす）。描く順は後で並べ直す
  for e = e_hi, e_lo, -1 do
    local te = rate.time_of(e)
    if te <= now + jitter then
      -- ループ: 乱数の番号を 1 周期に出る数で巻く（e と e ＋ 周期の数 が同じ粒子になる）
      local ew = LOOPE and (e % LOOPE) or e
      for j = sync - 1, 0, -1 do
        local k = ew * sync + j
        local ik = same_group and ew or k          -- 個別微調整の乱数の番号
        local sk = (RS and RS[k]) and RSK or sk
        local b = te
        if p_rate ~= 0 then
          local r = max(rate.rate(te), 1e-6)
          local w = p_rate > 0 and (1 / r) * p_rate / 100 or -p_rate
          b = max(te + (rnd(ew, CH.birth, sk) * 2 - 1) * w, 0)
        end
        local age = now - b
        if age >= 0 then
          local L = (TC_life or track("life", b))
          if indiv and indiv.life ~= 0 then L = vary(L, indiv.life, rnd(ik, CH.life, sk)) end
          L = max(L, 0.001)
          -- 終了時に消える: 終わりまでに（子も）消え終わらない粒子は生まない
          local born = not VT or b + L + VTAIL <= VT + 1e-9
          -- 寿命を過ぎた粒子も、子粒子を出すか止まって残るなら計算する（年齢は寿命で止める）
          local over = age >= L
          if born and (not over or CD or KP) and (not emit_win or in_window(emit_win, cfg.obj_time(b))) then
            if n >= cap then
              overflow = overflow + 1
            else
              local n0 = n
              local age_e = over and L or age
              local now_e = b + age_e
              local ob = cfg.obj_time(b)
              -- 出力位置（他レイヤーの形が空なら出さない）。他レイヤー・関数は出力方向と足す速度も返す
              local px, py, pz, odir, ozd, avx, avy, avz = emit_pos(emit, (emit and emit.same_pos) and ew or k, b, sk)
              local noemit = px == nil
              if noemit then px, py, pz = 0, 0, 0 end
              -- 向きと速さ
              local dir = (odir or (TC_dir or track("dir", b))) + (rnd(k, CH.dir, sk) * 2 - 1) * (TC_spread or track("spread", b))
              local zd = (ozd or (TC_zdir or track("zdir", b))) + (rnd(k, CH.zdir, sk) * 2 - 1) * (TC_zspread or track("zspread", b))
              local dxy, dz = cos(zd * RAD), sin(zd * RAD)
              local ux, uy, uz = sin(dir * RAD) * dxy, cos(dir * RAD) * dxy, dz
              local v0 = (TC_speed or track("speed", b))
              local ac = (TC_accel or track("accel", b))
              local gx, gy, gz = (TC_gx or track("gx", b)), (TC_gy or track("gy", b)), (TC_gz or track("gz", b))
              if indiv then
                v0 = vary(v0, indiv.speed, rnd(ik, CH.speed, sk))
                ac = vary(ac, indiv.accel, rnd(ik, CH.accel, sk))
                gx = vary(gx, indiv.gx, rnd(ik, CH.gx, sk))
                gy = vary(gy, indiv.gy, rnd(ik, CH.gy, sk))
                gz = vary(gz, indiv.gz, rnd(ik, CH.gz, sk))
              end
              if avx and (avx ~= 0 or avy ~= 0 or avz ~= 0) then
                -- 対象の動きの速さを足す（向きと速さを合わせ直す。加速度は新しい向きに掛かる）
                local wx_, wy_, wz_ = ux * v0 + avx, uy * v0 + avy, uz * v0 + avz
                local sp = sqrt(wx_ * wx_ + wy_ * wy_ + wz_ * wz_)
                if sp > 1e-9 then ux, uy, uz, v0 = wx_ / sp, wy_ / sp, wz_ / sp, sp end
              end
              if out.ex then out.ex[n], out.ey[n], out.ez[n] = px, py, pz end
              if out.cu and not noemit then
                local ci = ((emit.same_pos and ew or k) % CELLS.n) + 1
                out.cu[n], out.cv[n] = CELLS.x[ci] / CELLS.w, CELLS.y[ci] / CELLS.h
                out.cw[n], out.ch[n] = CELLS.cw[ci], CELLS.ch[ci]
                out.ci[n] = ci
              end
              if out.cl and not noemit then
                local c = 0
                if emit.shape == 6 then c = QSRC.col[emit.last_cell or 0]
                else c = CELLS.col[((emit.same_pos and ew or k) % CELLS.n) + 1] end
                out.cl[n] = nearest_pal(QPAL, c)
                out.csrc[n] = c
              elseif out.csrc then
                out.csrc[n] = -1
              end
              if J then
                px = px + M.jitter(J, k, 1, sk) * J.x
                py = py + M.jitter(J, k, 2, sk) * J.y
                pz = pz + M.jitter(J, k, 3, sk) * J.z
              end
              -- 動いた時間（時間の付け替えの「活動時間」の外では止まる）
              local tau = age_e
              if TW and TW.act_win then
                if TW.act_rel then tau = window_measure(TW.act_win, 0, age_e)
                else tau = window_measure(TW.act_win, cfg.obj_time(b), cfg.obj_time(now_e)) end
              end
              local x, y, z, vx, vy
              local tb, td = huge, huge
              local tbp = -1
              local rroll = 0
              local alive = true
              local S_
              if O then
                q.k, q.sk = k, sk
                if O.center == 0 then q.cx, q.cy, q.cz = px, py, pz
                elseif O.center == 1 then q.cx, q.cy, q.cz = O.px, O.py, O.pz
                else
                  q.cx = (rnd(k, CH.orbc, sk) * 2 - 1) * O.rx
                  q.cy = (rnd(k, CH.orbc + 1, sk) * 2 - 1) * O.ry
                  q.cz = (rnd(k, CH.orbc + 2, sk) * 2 - 1) * O.rz
                end
                local ex, ey = px - q.cx, py - q.cy
                q.r0 = sqrt(ex * ex + ey * ey)
                q.th0 = q.r0 > 1e-9 and atan2(ey, ex) or rnd(k, CH.orbth, sk) * 2 * pi
                q.ow = vary(O.w, O.i_w, rnd(ik, CH.orbw, sk))
                q.ovr = vary(O.vr, O.i_vr, rnd(ik, CH.orbvr, sk))
                x, y, z = orbit_pos(O, q, tau)
                local x2, y2 = orbit_pos(O, q, tau + 0.01)
                vx, vy = (x2 - x) / 0.01, (y2 - y) / 0.01
              elseif PA then
                path_setup(PA, q, k, ik, sk, px, py, pz)
                local dead
                x, y, z, vx, vy, dead = path_pos(PA, q, tau)
                if dead then alive = false end
              elseif P then
                -- 積分: 状態の置き場から続きを進める（無ければ誕生から）
                local h = P.h
                local full = floor(age_e / h)
                local S = P.cache.st[bit.band(k, P.cache.cap - 1)]
                -- 粒子どうし: 世界にいる粒子は、世界の状態から続ける（端数の刻みだけ、この下で粒子ごとに進める）
                if WORLD then
                  local wi = WORLD.idx[k]
                  if wi and WORLD.st[wi - 1].bb == b then ffi.copy(S, WORLD.st + (wi - 1), ffi.sizeof("PRH_State9")) end
                end
                S_ = S
                local slot = bit.band(k, P.cache.cap - 1)
                if S.key ~= k or S.steps > full or S.bb ~= b then
                  S.key, S.steps, S.flags, S.nb, S.tgt, S.bb = k, 0, 0, 0, -1, b
                  S.px, S.py, S.pz, S.dx, S.dy, S.dz, S.s = px, py, pz, ux, uy, uz, v0
                  S.wx, S.wy, S.wz, S.tb, S.td, S.ac, S.tk = 0, 0, 0, -1, -1, ac, -1
                  S.roll, S.om = 0, 0
                  if P.hist2 then
                    for jj = 0, P.hm2 - 1 do P.hstep2[slot * P.hm2 + jj] = -1 end
                  end
                  if P.hist then
                    for jj = 0, P.hm - 1 do P.hstep[slot * P.hm + jj] = -1 end
                    local b0 = slot * P.hm
                    P.hist[b0 * 3], P.hist[b0 * 3 + 1], P.hist[b0 * 3 + 2] = px, py, pz
                    P.hstep[b0] = 0
                  end
                end
                q.slot = slot
                q.k, q.sk, q.b, q.L = k, sk, b, L
                if P.fields then
                  local ff = q.ff or {}
                  q.ff = ff
                  for fi = 1, #P.fields do ff[fi] = vary(100, P.fields[fi].i_str, rnd(ik, CH.fld + fi, sk)) / 100 end
                end
                q.gx, q.gy, q.gz = gx, gy, gz
                q.fwind = P.wind and vary(100, P.wind.indiv, rnd(ik, CH.wind, sk)) / 100 or 1
                q.fnoise = P.noise and vary(100, P.noise.indiv, rnd(ik, CH.noise, sk)) / 100 or 1
                q.t_disp = (P.disp and P.disp.time > 0) and max(vary(P.disp.time, P.disp.i_time, rnd(ik, CH.tdisp, sk)), 0) or 0
                q.t_stop = (P.stop and P.stop.time > 0) and max(vary(P.stop.time, P.stop.i_time, rnd(ik, CH.tstop, sk)), 1e-9) or 0
                q.disp_speed = P.disp and P.disp.speed or 0
                disp_link(P, q, ik, sk, v0)
                if P.att then
                  q.t_att = vary(P.att.start, P.att.i_start, rnd(ik, CH.tatt, sk))
                  q.att_xy = P.att.xy + (rnd(k, CH.attxy, sk) * 2 - 1) * P.att.exy
                  q.att_z = P.att.z + (rnd(k, CH.attz, sk) * 2 - 1) * P.att.ez
                end
                if S.steps < full then
                  P.steps = P.steps + (full - S.steps)
                  advance(P, q, S, S.steps, full)
                end
                if bit.band(S.flags, F_DEAD) ~= 0 then alive = false end
                if out.tb and cfg.need_fb then
                  -- 跳ね返るまで: 跳ね返った後はその年齢、まだなら先読み（粒子の番号ごとに 1 回）
                  if S.tb >= 0 then tbp = S.tb
                  else
                    P.cache.fb = P.cache.fb or {}
                    local v = P.cache.fb[k]
                    if v == nil or v ~= v then v = M.first_bounce(P, q, S, L); P.cache.fb[k] = v end
                    tbp = v
                  end
                end
                local svx, svy, svz = S.dx * S.s + S.wx, S.dy * S.s + S.wy, S.dz * S.s + S.wz
                local r = age_e - full * h
                if bit.band(S.flags, F_PARK) ~= 0 then r = 0 end
                if P.act and not in_window(P.act.win, P.act.rel and age_e or P.act.obj(now_e)) then r = 0 end
                rroll = r
                x, y, z = S.px + svx * r, S.py + svy * r, S.pz + svz * r
                if P.vmul and r > 0 then
                  local mm = P.vmul((full * h + 0.5 * r) / L) - 1
                  local ox_, oy_, oz_ = S.dx * S.s * mm, S.dy * S.s * mm, S.dz * S.s * mm
                  x, y, z = x + ox_ * r, y + oy_ * r, z + oz_ * r
                  svx, svy = svx + ox_, svy + oy_
                end
                if P.vec and r > 0 then
                  local fx_, fy_, fz_ = P.vec(full * h)
                  x, y, z = x + fx_ * r, y + fy_ * r, z + fz_ * r
                end
                -- 刻みの間の外挿でも面や球を越えないようにする（位置だけ直す。状態は変えない）
                if P.bnc and r > 0 then x, y, z = bounce(P.bnc, x, y, z, svx, svy, svz, now_e, age_e) end
                vx, vy = svx, svy
                if S.tb >= 0 then tb = S.tb end
                if S.td >= 0 then td = S.td end
              elseif vmul then
                x, y, z, vx, vy = cf_pos_mul(px, py, pz, ux, uy, uz, v0, ac, gx, gy, gz, tau, L, vmul)
              else
                x, y, z, vx, vy = cf_pos(px, py, pz, ux, uy, uz, v0, ac, gx, gy, gz, tau)
              end
              -- 止まったら残す: 止まった粒子は寿命の後も描く（見た目は寿命の終わり、回転は止まった時刻で止める）
              local kept = KP and S_ and bit.band(S_.flags, F_PARK) ~= 0 and S_.tk >= 0
              if over and not kept then alive = false end
              -- 子粒子の出来事（親の年齢 a・位置・速度）
              local nev = 0
              if CD then
                local function add(a, ex_, ey_, ez_, evx, evy)
                  nev = nev + 1
                  local t_ = cev[nev] or {}
                  cev[nev] = t_
                  t_[1], t_[2], t_[3], t_[4], t_[5], t_[6] = a, ex_, ey_, ez_, evx, evy
                end
                if CD.event == 0 then
                  if over and not kept and not noemit then add(L, x, y, z, vx, vy) end
                elseif CD.event == 1 then
                  if S_ and bit.band(S_.flags, F_BNC) ~= 0 and S_.tb >= 0 then add(S_.tb, S_.e1x, S_.e1y, S_.e1z, S_.e1vx, S_.e1vy) end
                elseif CD.event == 2 then
                  if S_ and bit.band(S_.flags, F_DISP) ~= 0 and S_.td >= 0 then add(S_.td, S_.e2x, S_.e2y, S_.e2z, S_.e2vx, S_.e2vy) end
                elseif not noemit then
                  -- 動いている間: 年齢 = 間隔 × m の位置ごと（子が生きている m だけ）
                  -- 積分のときは履歴の間隔（刻みの整数倍）に合わせる
                  local ci = (P and P.hist2) and P.hstride2 * P.h or CD.interval
                  local m0 = max(ceil((age - CD.life) / ci), 1)
                  local m1 = floor(age_e / ci + 1e-9)
                  for mm = m1, m0, -1 do
                    local a = mm * ci
                    local hx, hy, hz
                    if O then hx, hy, hz = orbit_pos(O, q, a)
                    elseif PA then
                      local qx_, qy_, qz_, _vx, _vy, dead_ = path_pos(PA, q, a)
                      if not dead_ then hx, hy, hz = qx_, qy_, qz_ end
                    elseif P then
                      if P.hist2 and S_ then
                        local stp = floor(a / P.h + 0.5)
                        if stp % P.hstride2 == 0 then
                          local hb = q.slot * P.hm2 + (stp / P.hstride2) % P.hm2
                          if P.hstep2[hb] == stp then hx, hy, hz = P.hist2[hb * 3], P.hist2[hb * 3 + 1], P.hist2[hb * 3 + 2] end
                        end
                      end
                    elseif vmul then hx, hy, hz = cf_pos_mul(px, py, pz, ux, uy, uz, v0, ac, gx, gy, gz, a, L, vmul)
                    else hx, hy, hz = cf_pos(px, py, pz, ux, uy, uz, v0, ac, gx, gy, gz, a) end
                    if hx then add(a, hx, hy, hz, 0, 0) end
                  end
                end
              end
              -- ゆらぎ（挙動）
              local w_spd, w_fa, w_p1, w_p2, w_p3
              if WB then
                w_spd = vary(WB.speed, WB.i_speed, rnd(ik, CH.wobs, sk))
                w_fa = vary(100, WB.i_amp, rnd(ik, CH.wob, sk)) / 100
                w_p1, w_p2, w_p3 = rnd(k, CH.wobp, sk) * 360, rnd(k, CH.wobp + 1, sk) * 360, rnd(k, CH.wobp + 2, sk) * 360
              end
              -- 軌跡の点（少し前の位置）
              if TN > 0 then
                local base = n * TN
                for j = 1, TN do
                  local ok, tx, ty, tz, tvx, tvy = false
                  local aj = age - j * TR.dt
                  if O then
                    if aj >= 0 then
                      local tj = aj
                      if TW and TW.act_win then
                        if TW.act_rel then tj = window_measure(TW.act_win, 0, aj)
                        else tj = window_measure(TW.act_win, cfg.obj_time(b), cfg.obj_time(b + aj)) end
                      end
                      tx, ty, tz = orbit_pos(O, q, tj)
                      tvx, tvy, ok = vx, vy, true
                    end
                  elseif PA then
                    if aj >= 0 then
                      local tj = aj
                      if TW and TW.act_win then
                        if TW.act_rel then tj = window_measure(TW.act_win, 0, aj)
                        else tj = window_measure(TW.act_win, cfg.obj_time(b), cfg.obj_time(b + aj)) end
                      end
                      local dead
                      tx, ty, tz, tvx, tvy, dead = path_pos(PA, q, tj)
                      ok = not dead
                    end
                  elseif P then
                    if P.hist then
                      local full = floor(age / P.h)
                      local r = (ceil(full / P.hstride) - j) * P.hstride
                      if r >= 0 then
                        local hb = q.slot * P.hm + (r / P.hstride) % P.hm
                        if P.hstep[hb] == r then
                          tx, ty, tz = P.hist[hb * 3], P.hist[hb * 3 + 1], P.hist[hb * 3 + 2]
                          aj = r * P.h
                          tvx, tvy, ok = vx, vy, true
                        end
                      end
                    end
                  elseif aj >= 0 then
                    local tj = aj
                    if TW and TW.act_win then
                      if TW.act_rel then tj = window_measure(TW.act_win, 0, aj)
                      else tj = window_measure(TW.act_win, cfg.obj_time(b), cfg.obj_time(b + aj)) end
                    end
                    if vmul then
                      tx, ty, tz, tvx, tvy = cf_pos_mul(px, py, pz, ux, uy, uz, v0, ac, gx, gy, gz, tj, L, vmul)
                    else
                      tx, ty, tz, tvx, tvy = cf_pos(px, py, pz, ux, uy, uz, v0, ac, gx, gy, gz, tj)
                    end
                    ok = true
                  end
                  if ok and WB then
                    local ox, oy, oz = M.wobble(WB, w_spd, w_fa, w_p1, w_p2, w_p3, aj, tvx, tvy)
                    tx, ty, tz = tx + ox, ty + oy, tz + oz
                  end
                  out.tok[base + j - 1] = ok and 1 or 0
                  if ok then out.tx[base + j - 1], out.ty[base + j - 1], out.tz[base + j - 1] = tx, ty, tz end
                end
              end
              if WB then
                local ox, oy, oz = M.wobble(WB, w_spd, w_fa, w_p1, w_p2, w_p3, age, vx, vy)
                x, y, z = x + ox, y + oy, z + oz
              end
              out.x[n], out.y[n], out.z[n] = x, y, z
              out.vx[n], out.vy[n] = vx, vy
              -- 回転
              local mode = cfg.rot_random or 1
              local rx0 = mode == 2 and rnd(k, CH.rx0, sk) * 360 or (TC_rx0 or track("rx0", b))
              local ry0 = mode == 2 and rnd(k, CH.ry0, sk) * 360 or (TC_ry0 or track("ry0", b))
              local rz0 = mode >= 1 and rnd(k, CH.rz0, sk) * 360 or (TC_rz0 or track("rz0", b))
              local vrx, vry, vrz = (TC_vrx or track("vrx", b)), (TC_vry or track("vry", b)), (TC_vrz or track("vrz", b))
              if indiv then
                vrx = vary(vrx, indiv.vrx, rnd(ik, CH.vrx, sk))
                vry = vary(vry, indiv.vry, rnd(ik, CH.vry, sk))
                vrz = vary(vrz, indiv.vrz, rnd(ik, CH.vrz, sk))
              end
              if cfg.revrot then
                if rnd(k, CH.rev, sk) < 0.5 then vrx = -vrx end
                if rnd(k, CH.rev + 100, sk) < 0.5 then vry = -vry end
                if rnd(k, CH.rev + 200, sk) < 0.5 then vrz = -vrz end
              end
              local age_r = (KP and S_ and bit.band(S_.flags, F_PARK) ~= 0 and S_.tk >= 0) and min(age, S_.tk) or age
              out.rx[n] = axis_angle(rx0, vrx, age_r, b, 1, rot, k, sk, ob)
              out.ry[n] = axis_angle(ry0, vry, age_r, b, 2, rot, k, sk, ob)
              out.rz[n] = axis_angle(rz0, vrz, age_r, b, 3, rot, k, sk, ob)
              -- 転がる（v0.15.0）: 転がった角度（刻みの間は転がる速さで外挿）
              if S_ and P.bnc and P.bnc.roll then out.rz[n] = out.rz[n] + (S_.roll + S_.om * rroll) / RAD end
              -- 拡大率・透過率（透過率は 0=不透明。AviUtl2 の標準と同じ向き）
              out.zoom[n], out.alpha[n] = zoom_alpha(ZA, k, ik, sk, b, L, age_e, tb, td)
              out.k[n] = k
              if out.rs then out.rs[n] = (RS and RS[k]) and 1 or 0 end
              out.age[n] = age
              out.b[n], out.life[n], out.e[n] = b, L, ew
              if out.tb then out.tb[n], out.tbp[n] = (S_ and S_.tb >= 0) and S_.tb or -1, tbp end
              if out.td then out.td[n] = (S_ and S_.td >= 0) and S_.td or -1 end
              if alive and not noemit then n = n + 1 end
              -- 子粒子（速さ・重力・空気抵抗だけの閉じた式。親の後ろに並べる）
              for ie = 1, nev do
                local ev = cev[ie]
                local a_ev, ex0, ey0, ez0, pvx, pvy = ev[1], ev[2], ev[3], ev[4], ev[5], ev[6]
                local th = (pvx ~= 0 or pvy ~= 0) and atan2(pvx, pvy) or 0
                for c = 0, CD.n - 1 do
                  local ca = age - a_ev
                  if ca >= 0 and ca < CD.life then
                    if n >= cap then overflow = overflow + 1; break end
                    local ck = bit.tobit(0x30000000 + k * 101 + floor(a_ev * 1000 + 0.5) * 7 + c)
                    local ang = th + (rnd(ck, CH.child, sk) * 2 - 1) * CD.spread * RAD
                    local sp = CD.speed * (0.6 + 0.4 * rnd(ck, CH.child + 1, sk))
                    local f_ = CD.inherit / 100
                    local cvx, cvy = sin(ang) * sp + pvx * f_, cos(ang) * sp + pvy * f_
                    local kd, g = CD.drag, CD.gy
                    local cx_, cy_, cvx2, cvy2
                    local cz_ = ez0
                    local cdead = false
                    if CP then
                      -- 積分（子の番号ごとの状態から続ける。無ければ出来事の時刻から）
                      local h_ = CP.h
                      local cfull = floor(ca / h_)
                      local b_c = b + a_ev
                      local CS = CP.cst.st[ck]
                      if not CS or CS.steps > cfull or CS.bb ~= b_c then
                        CS = CS or ffi.new("PRH_State9")
                        CP.cst.st[ck] = CS
                        local sp0 = sqrt(cvx * cvx + cvy * cvy)
                        CS.key, CS.steps, CS.flags, CS.nb, CS.tgt, CS.bb = ck, 0, 0, 0, -1, b_c
                        CS.px, CS.py, CS.pz = ex0, ey0, ez0
                        if sp0 > 1e-12 then CS.dx, CS.dy = cvx / sp0, cvy / sp0 else CS.dx, CS.dy = 0, 1 end
                        CS.dz, CS.s = 0, sp0
                        CS.wx, CS.wy, CS.wz, CS.tb, CS.td, CS.ac, CS.tk = 0, 0, 0, -1, -1, 0, -1
                        CS.roll, CS.om = 0, 0
                      end
                      CP.cst.seen[ck] = cgen
                      cq.k, cq.sk, cq.b, cq.L = ck, sk, b_c, CD.life
                      if CS.steps < cfull then
                        P.steps = P.steps + (cfull - CS.steps)
                        advance(CP, cq, CS, CS.steps, cfull)
                      end
                      if bit.band(CS.flags, F_DEAD) ~= 0 then cdead = true end
                      local r_ = ca - cfull * h_
                      local svx, svy, svz = CS.dx * CS.s + CS.wx, CS.dy * CS.s + CS.wy, CS.dz * CS.s + CS.wz
                      cx_, cy_, cz_ = CS.px + svx * r_, CS.py + svy * r_, CS.pz + svz * r_
                      if CP.bnc and r_ > 0 then cx_, cy_, cz_ = bounce(CP.bnc, cx_, cy_, cz_, svx, svy, svz, b_c + ca, ca) end
                      cvx2, cvy2 = svx, svy
                    elseif kd > 0 then
                      local ek = math.exp(-kd * ca)
                      local fm = (1 - ek) / kd
                      cx_ = ex0 + cvx * fm
                      cy_ = ey0 + cvy * fm + g * (ca - fm) / kd
                      cvx2, cvy2 = cvx * ek, cvy * ek + g * (1 - ek) / kd
                    else
                      cx_ = ex0 + cvx * ca
                      cy_ = ey0 + cvy * ca + 0.5 * g * ca * ca
                      cvx2, cvy2 = cvx, cvy + g * ca
                    end
                    if cdead then goto next_child end
                    out.x[n], out.y[n], out.z[n] = cx_, cy_, cz_
                    out.vx[n], out.vy[n] = cvx2, cvy2
                    out.rx[n], out.ry[n], out.rz[n] = 0, 0, rnd(ck, CH.child + 2, sk) * 360
                    out.zoom[n] = CD.zoom / 100
                    out.alpha[n] = min(max(1 - CD.alpha1 / 100 * ca / CD.life, 0), 1)
                    out.k[n], out.age[n], out.b[n], out.life[n], out.e[n] = ck, ca, b + a_ev, CD.life, ew
                    if out.tb then out.tb[n], out.tbp[n] = -1, -1 end
                    if out.td then out.td[n] = -1 end
                    if out.rs then out.rs[n] = 0 end
                    if out.ex then out.ex[n], out.ey[n], out.ez[n] = ex0, ey0, ez0 end
                    if out.cu then
                      out.cu[n], out.cv[n], out.cw[n], out.ch[n], out.ci[n] = out.cu[n0], out.cv[n0], out.cw[n0], out.ch[n0], out.ci[n0]
                    end
                    if out.cl then out.cl[n] = out.cl[n0] end
                    if out.csrc then out.csrc[n] = out.csrc[n0] end
                    if TN > 0 then for jj = 0, TN - 1 do out.tok[n * TN + jj] = 0 end end
                    n = n + 1
                  end
                  ::next_child::
                end
              end
            end
          end
        end
      end
    end
  end
  out.n = n
  out.overflow = overflow
  if CP then
    local cs = CP.cst
    for kk, g in pairs(cs.seen) do
      if g ~= cgen then cs.seen[kk] = nil; cs.st[kk] = nil end
    end
  end
  if WORLD then out.inter_n, out.inter_over = WORLD.active, WORLD.over end
  out.steps = P and P.steps or 0
  out.resets = P and P.cache.resets or 0
  if cfg.gather then M.gather(out, cfg) end
  M.post(out, cfg)
  return out
end

----------------------------------------------------------------------------- 格子と音（計算し終えた粒子に掛ける）

-- 位置を格子にそろえる。G.mode 0 = 直交（間隔 a, b, c は X, Y, Z）/ 1 = 極座標（a = 半径、b = 角度、c = 仰角。度）。間隔 0 はそろえない
function M.snap(G, x, y, z)
  if G.mode == 0 then
    if G.a > 0 then x = floor(x / G.a + 0.5) * G.a end
    if G.b > 0 then y = floor(y / G.b + 0.5) * G.b end
    if G.c > 0 then z = floor(z / G.c + 0.5) * G.c end
    return x, y, z
  end
  local r = sqrt(x * x + y * y + z * z)
  if r < 1e-9 then return 0, 0, 0 end
  local th = atan2(y, x)
  local ph = math.acos(max(min(z / r, 1), -1))
  if G.a > 0 then r = floor(r / G.a + 0.5) * G.a end
  if G.b > 0 then local st = G.b * RAD; th = floor(th / st + 0.5) * st end
  if G.c > 0 then local st = G.c * RAD; ph = floor(ph / st + 0.5) * st end
  local sp = sin(ph)
  return r * sp * cos(th), r * sp * sin(th), r * cos(ph)
end

----------------------------------------------------------------------------- 形へ集まる（v0.7.0）

--[[
cfg.gather（呼び手が作る）:
  src    行き先 0 他レイヤーの形 / 1 出た所 / 2 文字の並び / 3 アンカー
  pick   他レイヤーの形の選び方 0 ランダム / 1 近い所 / 2 均等に
  mask, lay    他レイヤーの形（M.build_mask）と、今の対象の位置・回転・拡大率・中心（lay() → X, Y, Z, rz, sx, sy, cx, cy）
  tlay, tmat   文字の並び（M.text_layout）と素材の設定（文字の番号の割り当て）
  pts, np      アンカー（文字の並びは 1 点目が中心）
  radius       行き先の広がり（px。円の中に散らす）
  start, dur, base（0 粒子ごと / 1 オブジェクト）, spread（ずらす幅・秒）, order（0 番号順 / 1 ランダム / 2 左から / 3 中心から）
  curve, rep, fn   補間（M.curve と同じ。1 = 瞬間）
  align        回転を 0・拡大率を 100% へ同じ進みで寄せる
  after, leave 着いたら 0 とどまる / 1 離れる（離れ始める時刻から、かかる時間で元へ戻る）
  jitter       着いた後のゆらぎ（px）
]]
local function wrap180(a) return (a + 180) % 360 - 180 end

function M.gather(out, cfg)
  local GA = cfg.gather
  local n = out.n
  if n <= 0 or not GA then return end
  local sk0, now = cfg.sk, cfg.now or 0
  local tobj = cfg.obj_time and cfg.obj_time(now) or now
  local src = GA.src or 0
  local Mk = GA.mask
  local X, Y, Z, rz, sx, sy, cx, cy
  if src == 0 then
    if not Mk or Mk.nc <= 0 or not GA.lay then return end
    X, Y, Z, rz, sx, sy, cx, cy = GA.lay()
    if X == nil then return end
  elseif src == 1 then
    if not out.ex then return end
  elseif src == 2 then
    if not GA.tlay or not GA.tmat or (GA.tmat.count or 0) <= 0 then return end
  end
  local pts = GA.pts or { 0, 0, 0 }
  local np = max(min(floor(GA.np or 1), floor(#pts / 3)), 1)
  -- 行き先
  local tx, ty, tz, ok = {}, {}, {}, {}
  local kmin, kmax = huge, -huge
  for i = 0, n - 1 do
    local k = out.k[i]
    local sk = (out.rs and out.rs[i] == 1) and out.rsk or sk0
    if k < kmin then kmin = k end
    if k > kmax then kmax = k end
    local x, y, z
    if src == 0 then
      local mx, my
      if GA.pick == 1 then
        -- 近い所: 形の外の点は、形の縁のマスのうちいちばん近いものへ（形の中の点は動かさない）
        mx, my = layer_unpoint(X, Y, rz, sx, sy, cx, cy, out.x[i], out.y[i])
        mx, my = M.nearest_inside(Mk, mx, my)
      elseif GA.pick == 2 then
        -- 均等に: 粒子の番号を黄金比でならしてマスを選ぶ（数によらず形の全体に散る）
        local u = (k * 0.6180339887498949) % 1
        local c = Mk.cells[min(floor(u * Mk.nc), Mk.nc - 1) + 1]
        mx = min((c % Mk.gw + 0.5) * Mk.cs, Mk.w) - Mk.w / 2
        my = min((floor(c / Mk.gw) + 0.5) * Mk.cs, Mk.h) - Mk.h / 2
      else
        mx, my = M.mask_point(Mk, rnd(k, CH.gat, sk), rnd(k, CH.gat + 1, sk), rnd(k, CH.gat + 2, sk))
      end
      if mx then x, y, z = layer_point(X, Y, Z, rz, sx, sy, cx, cy, mx, my) end
    elseif src == 1 then
      x, y, z = out.ex[i], out.ey[i], out.ez[i]
    elseif src == 2 then
      local u = M.text_index(GA.tmat, k, sk) + 1
      if GA.tlay.x[u] then x, y, z = pts[1] + GA.tlay.x[u], pts[2] + GA.tlay.y[u], pts[3] end
    else
      local j = k % np
      x, y, z = pts[j * 3 + 1], pts[j * 3 + 2], pts[j * 3 + 3]
    end
    if x and (GA.radius or 0) > 0 then
      local r = sqrt(rnd(k, CH.gat + 3, sk)) * GA.radius
      local a = rnd(k, CH.gat + 4, sk) * 2 * pi
      x, y = x + r * cos(a), y + r * sin(a)
    end
    ok[i] = x ~= nil
    tx[i], ty[i], tz[i] = x, y, z
  end
  -- ずらす順の割合（0..1）
  local order, spread = GA.order or 0, GA.spread or 0
  local x0, x1, mcx, mcy, rmax = huge, -huge, 0, 0, 0
  if spread > 0 and (order == 2 or order == 3) then
    local cnt = 0
    for i = 0, n - 1 do
      if ok[i] then
        x0, x1 = min(x0, tx[i]), max(x1, tx[i])
        mcx, mcy, cnt = mcx + tx[i], mcy + ty[i], cnt + 1
      end
    end
    if cnt > 0 then mcx, mcy = mcx / cnt, mcy / cnt end
    for i = 0, n - 1 do
      if ok[i] then rmax = max(rmax, sqrt((tx[i] - mcx) ^ 2 + (ty[i] - mcy) ^ 2)) end
    end
  end
  local shape = M.curve(GA.curve, GA.rep, GA.fn)
  local function ease(p)
    if GA.curve == 1 then return p >= 1 and 1 or 0 end
    return shape and shape(p) or p
  end
  local dur = max(GA.dur or 1, 1e-6)
  local TN = out.tn or 0
  for i = 0, n - 1 do
    if ok[i] then
      local k = out.k[i]
      local sk = (out.rs and out.rs[i] == 1) and out.rsk or sk0
      local u = 0
      if spread > 0 then
        if order == 1 then u = rnd(k, CH.gst, sk)
        elseif order == 2 then u = x1 > x0 and (tx[i] - x0) / (x1 - x0) or 0
        elseif order == 3 then u = rmax > 0 and sqrt((tx[i] - mcx) ^ 2 + (ty[i] - mcy) ^ 2) / rmax or 0
        else u = kmax > kmin and (k - kmin) / (kmax - kmin) or 0 end
      end
      local t = (GA.base == 1) and tobj or out.age[i]
      local p = min(max((t - GA.start - spread * u) / dur, 0), 1)
      local e = ease(p)
      if GA.after == 1 then
        local p2 = min(max((t - GA.leave - spread * u) / dur, 0), 1)
        e = e * (1 - ease(p2))
      end
      if e ~= 0 then
        local gx, gy, gz = tx[i], ty[i], tz[i]
        if (GA.jitter or 0) > 0 then
          local ph1, ph2 = rnd(k, CH.gst + 1, sk), rnd(k, CH.gst + 2, sk)
          gx = gx + sin(2 * pi * (0.73 * t + ph1)) * GA.jitter
          gy = gy + cos(2 * pi * (0.91 * t + ph2)) * GA.jitter
        end
        local x, y, z = out.x[i], out.y[i], out.z[i]
        out.x[i], out.y[i], out.z[i] = x + (gx - x) * e, y + (gy - y) * e, z + (gz - z) * e
        out.vx[i], out.vy[i] = out.vx[i] * (1 - e), out.vy[i] * (1 - e)
        for j = 0, TN - 1 do
          local b = i * TN + j
          if out.tok[b] == 1 then
            out.tx[b] = out.tx[b] + (gx - out.tx[b]) * e
            out.ty[b] = out.ty[b] + (gy - out.ty[b]) * e
            out.tz[b] = out.tz[b] + (gz - out.tz[b]) * e
          end
        end
        if GA.align then
          out.rx[i] = wrap180(out.rx[i]) * (1 - e)
          out.ry[i] = wrap180(out.ry[i]) * (1 - e)
          out.rz[i] = wrap180(out.rz[i]) * (1 - e)
          out.zoom[i] = out.zoom[i] + (1 - out.zoom[i]) * e
        end
      end
    end
  end
end

--[[
cfg.audio: { level = 0..1, zoom = %, alpha = %, spread = % }（level が 1 のときの変化）
cfg.grid:  { mode, a, b, c, mask = M.build_mask(...)（本体の中心に重ねる。無ければ nil）, inv }
]]
function M.post(out, cfg, look_only)
  local A, G, V = cfg.audio, cfg.grid, cfg.look
  if not A and not G and not V then return end
  local TN = out.tn or 0
  local sk, now = cfg.sk, cfg.now or 0
  if V and (V.aspect ~= 0 or V.i_aspect ~= 0 or V.shear ~= 0 or V.i_shear ~= 0) then
    local m = max(out.n, 1)
    out.asx, out.asy, out.shr = ffi.new("double[?]", m), ffi.new("double[?]", m), ffi.new("double[?]", m)
  end
  for i = 0, out.n - 1 do
    if V then
      -- 見た目の調整: 速さ・奥行き・点滅で拡大率と透過率を変え、縦横比とゆがみを決める
      local k = out.k[i]
      if V.sref > 0 and (V.szoom ~= 0 or V.salpha ~= 0) then
        local f = min(sqrt(out.vx[i] * out.vx[i] + out.vy[i] * out.vy[i]) / V.sref, 10)
        out.zoom[i] = out.zoom[i] * max(1 + V.szoom / 100 * f, 0)
        out.alpha[i] = out.alpha[i] * min(max(1 - V.salpha / 100 * f, 0), 1)
      end
      if (V.dzoom ~= 0 or V.dalpha ~= 0) and V.dz1 ~= V.dz0 then
        local f = min(max((out.z[i] - V.dz0) / (V.dz1 - V.dz0), 0), 1)
        out.zoom[i] = out.zoom[i] * max(1 + V.dzoom / 100 * f, 0)
        out.alpha[i] = out.alpha[i] * min(max(1 - V.dalpha / 100 * f, 0), 1)
      end
      if V.blink_depth > 0 and V.blink_hz > 0 then
        local ph = V.blink_rand and rnd(k, CH.blink, sk) or 0
        local m
        if V.blink_shape == 0 then
          m = 1 - V.blink_depth / 100 * (0.5 - 0.5 * cos(2 * pi * (V.blink_hz * now + ph)))
        else
          -- きらめき: 1 / 点滅の速さ 秒ごとに、15% の確率で強く光る（それ以外は 深さ だけ暗い）
          local tick = floor(now * V.blink_hz + ph)
          m = rnd(k * 7919 + tick, CH.blink + 1, sk) < 0.15 and 1 or (1 - V.blink_depth / 100)
        end
        out.alpha[i] = out.alpha[i] * min(max(m, 0), 1)
      end
      if out.asx then
        local a = min(max(V.aspect + (rnd(k, CH.look, sk) * 2 - 1) * V.i_aspect, -99), 99)
        out.asx[i] = a > 0 and 1 - a / 100 or 1
        out.asy[i] = a < 0 and 1 + a / 100 or 1
        out.shr[i] = (V.shear + (rnd(k, CH.look + 1, sk) * 2 - 1) * V.i_shear) / 100
      end
    end
    if A then
      local L = A.level
      out.zoom[i] = out.zoom[i] * max(1 + A.zoom / 100 * L, 0)
      out.alpha[i] = out.alpha[i] * min(max(1 - A.alpha / 100 * L, 0), 1)
      if A.spread ~= 0 and out.ex and not look_only then
        -- 出た所からの距離を広げる
        local f = 1 + A.spread / 100 * L
        local ex, ey, ez = out.ex[i], out.ey[i], out.ez[i]
        out.x[i], out.y[i], out.z[i] = ex + (out.x[i] - ex) * f, ey + (out.y[i] - ey) * f, ez + (out.z[i] - ez) * f
        for j = 0, TN - 1 do
          local b = i * TN + j
          if out.tok[b] == 1 then
            out.tx[b], out.ty[b], out.tz[b] = ex + (out.tx[b] - ex) * f, ey + (out.ty[b] - ey) * f, ez + (out.tz[b] - ez) * f
          end
        end
      end
    end
    if G then
      local x, y, z = out.x[i], out.y[i], out.z[i]
      if not look_only then
        x, y, z = M.snap(G, x, y, z)
        out.x[i], out.y[i], out.z[i] = x, y, z
        for j = 0, TN - 1 do
          local b = i * TN + j
          if out.tok[b] == 1 then out.tx[b], out.ty[b], out.tz[b] = M.snap(G, out.tx[b], out.ty[b], out.tz[b]) end
        end
      end
      if G.mask then
        local inside = M.mask_at(G.mask, x, y) >= G.mask.thr
        if inside == (G.inv and true or false) then out.zoom[i], out.alpha[i] = 0, 0 end
      end
    end
  end
end

----------------------------------------------------------------------------- 描画用の四角形

local basis   -- 描くものの面の向き（下の「まとまりの頂点」で定める。光源が build_draw の中で使う）

-- 回転の順番: 1=X→Y→Z 2=X→Z→Y 3=Y→X→Z 4=Y→Z→X 5=Z→X→Y 6=Z→Y→X（左から順に、固定した軸で回す）
local ORDERS = { { 1, 2, 3 }, { 1, 3, 2 }, { 2, 1, 3 }, { 2, 3, 1 }, { 3, 1, 2 }, { 3, 2, 1 } }
M.STANDARD_ORDER = 6       -- 「標準」= obj.draw と同じ順（Z→Y→X と推定。実機 EPR-30〜33 で確かめる）

local function rot_axis(axis, c, s, x, y, z)
  if axis == 1 then return x, y * c - z * s, y * s + z * c end
  if axis == 2 then return x * c + z * s, y, -x * s + z * c end
  return x * c - y * s, x * s + y * c, z
end

-- 四角形の 4 頂点（中心からの相対）を回す
local function rotate4(vs, rx, ry, rz, order)
  local ang = { rx * RAD, ry * RAD, rz * RAD }
  for _, axis in ipairs(ORDERS[order]) do
    local a = ang[axis]
    if a ~= 0 then
      local c, s = cos(a), sin(a)
      for i = 0, 3 do
        local o = i * 3
        vs[o + 1], vs[o + 2], vs[o + 3] = rot_axis(axis, c, s, vs[o + 1], vs[o + 2], vs[o + 3])
      end
    end
  end
end
M.rotate4 = rotate4


----------------------------------------------------------------------------- ファイル（素材の画像フォルダ・テキスト）

pcall(ffi.cdef, [[
typedef struct {
  uint32_t attr; uint32_t ft[6]; uint32_t size_hi, size_lo, r0, r1;
  uint16_t name[260]; uint16_t alt[14];
} PRH_FINDDATA;
void* FindFirstFileW(const uint16_t* path, PRH_FINDDATA* fd);
int FindNextFileW(void* h, PRH_FINDDATA* fd);
int FindClose(void* h);
int MultiByteToWideChar(unsigned cp, unsigned flags, const char* s, int n, uint16_t* w, int wn);
int WideCharToMultiByte(unsigned cp, unsigned flags, const uint16_t* w, int wn, char* s, int n, const char* d, int* u);
void* CreateFileW(const uint16_t* name, uint32_t access, uint32_t share, void* sa, uint32_t disp, uint32_t flags, void* tmpl);
int ReadFile(void* h, void* buf, uint32_t n, uint32_t* got, void* ov);
int CloseHandle(void* h);
uint32_t GetFileSize(void* h, uint32_t* hi);
]])
local k32
local function kernel() if not k32 then k32 = ffi.load("kernel32") end return k32 end

local function to_w(s)
  local K = kernel()
  local n = K.MultiByteToWideChar(65001, 0, s, -1, nil, 0)
  local w = ffi.new("uint16_t[?]", n)
  K.MultiByteToWideChar(65001, 0, s, -1, w, n)
  return w
end

local function from_w(w)
  local K = kernel()
  local buf = ffi.new("char[1024]")
  local n = K.WideCharToMultiByte(65001, 0, w, -1, buf, 1024, nil, nil)
  return ffi.string(buf, max(n - 1, 0))
end

-- フォルダの中のファイル（名前順）。exts = { png = true, ... }（小文字の拡張子）。日本語のパスも読める
function M.list_dir(dir, exts)
  local list = {}
  if not dir or dir == "" then return list end
  local BS = string.char(92)
  if dir:sub(-1) ~= BS and dir:sub(-1) ~= "/" then dir = dir .. BS end
  local K = kernel()
  local fd = ffi.new("PRH_FINDDATA")
  local h = K.FindFirstFileW(to_w(dir .. "*"), fd)
  if ffi.cast("intptr_t", h) == -1 then return list end
  repeat
    if bit.band(fd.attr, 0x10) == 0 then
      local name = from_w(fd.name)
      local ext = (name:match("%.([^%.]+)$") or ""):lower()
      if exts[ext] then list[#list + 1] = dir .. name end
    end
  until K.FindNextFileW(h, fd) == 0
  K.FindClose(h)
  table.sort(list)
  return list
end

-- テキストファイルを丸ごと読む（UTF-8。先頭の BOM は外す）。日本語のパスも読める。失敗したら nil
function M.read_text(path)
  if not path or path == "" then return nil end
  local K = kernel()
  local h = K.CreateFileW(to_w(path), 0x80000000, 1, nil, 3, 0x80, nil)
  if ffi.cast("intptr_t", h) == -1 then return nil end
  local size = K.GetFileSize(h, nil)
  local buf = ffi.new("uint8_t[?]", size + 1)
  local got = ffi.new("uint32_t[1]")
  K.ReadFile(h, buf, size, got, nil)
  K.CloseHandle(h)
  local s = ffi.string(buf, got[0])
  if s:sub(1, 3) == "\239\187\191" then s = s:sub(4) end
  return s
end

-- ファイルを丸ごと読む（バイト列）。日本語のパスも読める。戻り値: buf（uint8_t*）, 大きさ。失敗したら nil
function M.read_bytes(path)
  if not path or path == "" then return nil end
  local K = kernel()
  local h = K.CreateFileW(to_w(path), 0x80000000, 1, nil, 3, 0x80, nil)
  if ffi.cast("intptr_t", h) == -1 then return nil end
  local size = K.GetFileSize(h, nil)
  local buf = ffi.new("uint8_t[?]", size + 1)
  local got = ffi.new("uint32_t[1]")
  K.ReadFile(h, buf, size, got, nil)
  K.CloseHandle(h)
  return buf, got[0]
end

----------------------------------------------------------------------------- 焼き付け（v0.13.0）

pcall(ffi.cdef, [[
int WriteFile(void* h, const void* buf, uint32_t n, uint32_t* done, void* ov);
int SetFilePointerEx(void* h, int64_t dist, int64_t* newpos, uint32_t method);
int GetFileSizeEx(void* h, int64_t* size);
int SetEndOfFile(void* h);
]])

--[[
ファイルの形（リトルエンディアン）:
  先頭  "PRHBAKE1"(8) / フレームの数 N(int32) / シグネチャの長さ L(int32) / シグネチャ(L) / 目次: N × { 場所(int64), 大きさ(int32) }
  ブロック（1 フレーム）: 欄の数(int32) / 欄ごとに { 名前の長さ(uint8) / 名前 / 型(uint8: 1 = float32, 2 = int32, 3 = uint8, 4 = 数 1 つ(double)) / 個数(int32) / 値 }
目次の場所 0 は「そのフレームは無い」。書くときはファイルの終わりにブロックを足し、目次だけを書き換える（どの順で描いても上書きできる）
]]
local BAKE_MAGIC = "PRHBAKE1"
-- 粒子ごとの欄（無い欄は書かない）。軌跡の欄は 粒子の数 × 軌跡の点の数
local BAKE_F32 = { "x", "y", "z", "rx", "ry", "rz", "vx", "vy", "zoom", "alpha", "age", "b", "life",
                   "ex", "ey", "ez", "cu", "cv", "cw", "ch", "asx", "asy", "shr", "tb", "tbp", "td" }
local BAKE_I32 = { "k", "e", "ci", "csrc" }
local BAKE_U8 = { "cl", "rs" }
local BAKE_TRAIL = { "tx", "ty", "tz" }
local BAKE_SCALAR = { "overflow", "steps", "resets", "inter_n", "inter_over", "rsk", "tn" }

local function bake_open(path, write)
  local K = kernel()
  local access = write and 0xC0000000 or 0x80000000
  local h = K.CreateFileW(to_w(path), access, 1, nil, write and 4 or 3, 0x80, nil)
  if ffi.cast("intptr_t", h) == -1 then return nil end
  return h
end

local function bake_read_at(h, pos, n)
  local K = kernel()
  if K.SetFilePointerEx(h, pos, nil, 0) == 0 then return nil end
  local buf = ffi.new("uint8_t[?]", max(n, 1))
  local got = ffi.new("uint32_t[1]")
  if n > 0 and (K.ReadFile(h, buf, n, got, nil) == 0 or got[0] ~= n) then return nil end
  return buf
end

local function bake_write_at(h, pos, buf, n)
  local K = kernel()
  if pos >= 0 then
    if K.SetFilePointerEx(h, pos, nil, 0) == 0 then return false end
  else
    if K.SetFilePointerEx(h, 0, nil, 2) == 0 then return false end
  end
  local done = ffi.new("uint32_t[1]")
  return K.WriteFile(h, buf, n, done, nil) ~= 0 and done[0] == n
end

local function file_size(h)
  local sz = ffi.new("int64_t[1]")
  if kernel().GetFileSizeEx(h, sz) == 0 then return -1 end
  return tonumber(sz[0])
end

-- 先頭を読む。戻り値: { N, sig, idx_pos }（形が違えば nil）
local function bake_header(h)
  local b = bake_read_at(h, 0, 16)
  if not b or ffi.string(b, 8) ~= BAKE_MAGIC then return nil end
  local hdr = ffi.cast("int32_t*", b + 8)
  local N, L = hdr[0], hdr[1]
  if N < 1 or L < 0 or L > 1000000 then return nil end
  local sb = bake_read_at(h, 16, L)
  if not sb then return nil end
  return { N = N, sig = ffi.string(sb, L), idx = 16 + L }
end

-- res を 1 フレームのブロックにする（Lua の文字列の並び → 1 つにつなぐ）
local function bake_pack(res)
  local parts, nf = {}, 0
  local n = res.n
  local function field(name, ty, cnt, src)
    local nb = #name
    local size = ty == 1 and 4 or (ty == 2 and 4 or (ty == 3 and 1 or 8))
    local buf = ffi.new("uint8_t[?]", 1 + nb + 1 + 4 + cnt * size)
    buf[0] = nb
    ffi.copy(buf + 1, name, nb)
    buf[1 + nb] = ty
    ffi.cast("int32_t*", buf + 2 + nb)[0] = cnt
    local p = buf + 6 + nb
    if ty == 1 then
      local d = ffi.cast("float*", p)
      for i = 0, cnt - 1 do d[i] = src[i] end
    elseif ty == 2 then
      local d = ffi.cast("int32_t*", p)
      for i = 0, cnt - 1 do d[i] = src[i] end
    elseif ty == 3 then
      for i = 0, cnt - 1 do p[i] = src[i] end
    else
      ffi.cast("double*", p)[0] = src
    end
    nf = nf + 1
    parts[nf] = ffi.string(buf, 6 + nb + cnt * size)
  end
  field("n", 4, 1, n)
  for _, k in ipairs(BAKE_SCALAR) do if type(res[k]) == "number" then field(k, 4, 1, res[k]) end end
  for _, k in ipairs(BAKE_F32) do if res[k] then field(k, 1, n, res[k]) end end
  for _, k in ipairs(BAKE_I32) do if res[k] then field(k, 2, n, res[k]) end end
  for _, k in ipairs(BAKE_U8) do if res[k] then field(k, 3, n, res[k]) end end
  if res.tn and res.tx then
    local m = n * res.tn
    for _, k in ipairs(BAKE_TRAIL) do field(k, 1, m, res[k]) end
    field("tok", 3, m, res.tok)
  end
  if res.qpal then
    local q = ffi.new("int32_t[?]", max(#res.qpal, 1))
    for i = 1, #res.qpal do q[i - 1] = res.qpal[i] end
    field("qpal", 2, #res.qpal, q)
  end
  local head = ffi.new("int32_t[1]", nf)
  return ffi.string(head, 4) .. table.concat(parts)
end

local function bake_unpack(buf, size)
  local p = 4
  local nf = ffi.cast("int32_t*", buf)[0]
  local res = {}
  local raw = {}
  for _ = 1, nf do
    if p >= size then return nil end
    local nb = buf[p]
    local name = ffi.string(buf + p + 1, nb)
    local ty = buf[p + 1 + nb]
    local cnt = ffi.cast("int32_t*", buf + p + 2 + nb)[0]
    local q = buf + p + 6 + nb
    local sz = ty == 1 and 4 or (ty == 2 and 4 or (ty == 3 and 1 or 8))
    if p + 6 + nb + cnt * sz > size then return nil end
    if ty == 4 then
      res[name] = ffi.cast("double*", q)[0]
    else
      local arr
      if ty == 1 then
        arr = ffi.new("double[?]", max(cnt, 1))
        local src = ffi.cast("float*", q)
        for i = 0, cnt - 1 do arr[i] = src[i] end
      elseif ty == 2 then
        arr = ffi.new("int32_t[?]", max(cnt, 1))
        ffi.copy(arr, q, cnt * 4)
      else
        arr = ffi.new("uint8_t[?]", max(cnt, 1))
        ffi.copy(arr, q, cnt)
      end
      raw[name] = { arr, cnt }
    end
    p = p + 6 + nb + cnt * sz
  end
  for name, v in pairs(raw) do
    if name == "qpal" then
      local t = {}
      for i = 0, v[2] - 1 do t[i + 1] = v[1][i] end
      res.qpal = t
    else
      res[name] = v[1]
    end
  end
  res.n = floor(res.n or 0)
  if res.tn then res.tn = floor(res.tn) end
  return res
end

-- 書く: フレーム f（0 から）の res を path に置く。N = フレームの数、sig = 設定の目印。戻り値: true か nil, 理由
function M.bake_write(path, sig, N, f, res)
  if f < 0 or f >= N then return nil, "フレームが範囲の外" end
  local h = bake_open(path, true)
  if not h then return nil, "焼き付けのファイルを開けない" end
  local K = kernel()
  local H = bake_header(h)
  if not H or H.N ~= N or H.sig ~= sig then
    -- 作り直す（設定かオブジェクトの長さが変わった）
    K.SetFilePointerEx(h, 0, nil, 0)
    K.SetEndOfFile(h)
    local L = #sig
    local head = ffi.new("uint8_t[?]", 16 + L + N * 12)
    ffi.copy(head, BAKE_MAGIC, 8)
    local hp = ffi.cast("int32_t*", head + 8)
    hp[0], hp[1] = N, L
    ffi.copy(head + 16, sig, L)
    if not bake_write_at(h, 0, head, 16 + L + N * 12) then K.CloseHandle(h); return nil, "焼き付けのファイルに書けない" end
    H = { N = N, sig = sig, idx = 16 + L }
  end
  local blk = bake_pack(res)
  local pos = file_size(h)
  local ok = pos >= 0 and bake_write_at(h, -1, blk, #blk)
  if ok then
    local e = ffi.new("uint8_t[12]")
    ffi.cast("int64_t*", e)[0] = pos
    ffi.cast("int32_t*", e + 8)[0] = #blk
    ok = bake_write_at(h, H.idx + f * 12, e, 12)
  end
  K.CloseHandle(h)
  if not ok then return nil, "焼き付けのファイルに書けない" end
  return true
end

-- 読む: フレーム f の res。戻り値: res か nil, 理由（"無い" = ファイルが無い / "違う" = 設定が違う / "抜け" = そのフレームが無い）
--[[
CSV（v0.16.0）: dir の 名前_csv フォルダに、フレームごとに 00012.csv（フレームの番号）を書く。1 行 1 粒、番号の小さい順。
列: k,x,y,z,rx,ry,rz,zoom,alpha,age,life（座標は本体から見た px、角度は度、zoom は %、alpha は 0..1 の不透明度）
]]
pcall(ffi.cdef, [[
int CreateDirectoryW(const uint16_t* path, void* sa);
]])
function M.csv_write(dir, name, f, res)
  local BS = string.char(92)
  if dir:sub(-1) ~= BS and dir:sub(-1) ~= "/" then dir = dir .. BS end
  local folder = dir .. name .. "_csv"
  local K = kernel()
  K.CreateDirectoryW(to_w(folder), nil)
  local idx = {}
  for i = 0, res.n - 1 do idx[i + 1] = i end
  M.sort_by_k(idx, res)
  local lines = { "k,x,y,z,rx,ry,rz,zoom,alpha,age,life" }
  local fmt = "%d,%.4f,%.4f,%.4f,%.4f,%.4f,%.4f,%.4f,%.6f,%.6f,%.6f"
  for t, i in ipairs(idx) do
    lines[t + 1] = string.format(fmt, res.k[i], res.x[i], res.y[i], res.z[i], res.rx[i], res.ry[i], res.rz[i],
                                 res.zoom[i] * 100, res.alpha[i], res.age[i], res.life[i])
  end
  local text = table.concat(lines, "\r\n") .. "\r\n"
  local h = K.CreateFileW(to_w(folder .. BS .. string.format("%05d.csv", f)), 0x40000000, 1, nil, 2, 0x80, nil)
  if ffi.cast("intptr_t", h) == -1 then return nil, "CSV を書けない: " .. folder end
  local done = ffi.new("uint32_t[1]")
  local ok = K.WriteFile(h, text, #text, done, nil) ~= 0 and done[0] == #text
  K.CloseHandle(h)
  if not ok then return nil, "CSV を書けない: " .. folder end
  return true
end

function M.bake_read(path, sig, N, f)
  local h = bake_open(path, false)
  if not h then return nil, "無い" end
  local K = kernel()
  local H = bake_header(h)
  if not H or H.N ~= N or H.sig ~= sig then K.CloseHandle(h); return nil, "違う" end
  if f < 0 or f >= N then K.CloseHandle(h); return nil, "抜け" end
  local e = bake_read_at(h, H.idx + f * 12, 12)
  if not e then K.CloseHandle(h); return nil, "抜け" end
  local pos, size = tonumber(ffi.cast("int64_t*", e)[0]), ffi.cast("int32_t*", e + 8)[0]
  if pos <= 0 or size <= 0 then K.CloseHandle(h); return nil, "抜け" end
  local b = bake_read_at(h, pos, size)
  K.CloseHandle(h)
  if not b then return nil, "抜け" end
  local res = bake_unpack(b, size)
  if not res then return nil, "抜け" end
  return res
end
M.text_hash = text_hash

-- WAV を読み、チャンネルを混ぜたモノラルの float の並びにする（PCM 16 / 24 / 32 bit・float 32 bit・EXTENSIBLE）。
-- 戻り値: { rate, n, x = float[n]（-1..1） }。読めなければ nil, 理由
function M.wav_decode(buf, size)
  if not buf or size < 44 then return nil, "WAV ではない" end
  local function u32(o) return buf[o] + buf[o + 1] * 256 + buf[o + 2] * 65536 + buf[o + 3] * 16777216 end
  local function u16(o) return buf[o] + buf[o + 1] * 256 end
  if ffi.string(buf, 4) ~= "RIFF" or ffi.string(buf + 8, 4) ~= "WAVE" then return nil, "WAV ではない" end
  local o = 12
  local fmt, ch, rate, bits, data_o, data_n
  while o + 8 <= size do
    local id, len = ffi.string(buf + o, 4), u32(o + 4)
    if id == "fmt " then
      fmt, ch, rate, bits = u16(o + 8), u16(o + 10), u32(o + 12), u16(o + 22)
      if fmt == 0xFFFE and len >= 26 then fmt = u16(o + 32) end
    elseif id == "data" then
      data_o, data_n = o + 8, min(len, size - (o + 8))
      break
    end
    o = o + 8 + len + (len % 2)
  end
  if not fmt or not data_o then return nil, "WAV の fmt か data が無い" end
  local bps = floor(bits / 8)
  if ch < 1 or bps < 1 or not ((fmt == 1 and (bits == 16 or bits == 24 or bits == 32)) or (fmt == 3 and bits == 32)) then
    return nil, "対応していない WAV（形式 " .. fmt .. "・" .. bits .. " bit）"
  end
  local frame = bps * ch
  local n = floor(data_n / frame)
  local x = ffi.new("float[?]", max(n, 1))
  local p = buf + data_o
  if fmt == 3 then
    local f = ffi.cast("float*", p)
    for i = 0, n - 1 do
      local a = 0
      for c = 0, ch - 1 do a = a + f[i * ch + c] end
      x[i] = a / ch
    end
  elseif bits == 16 then
    local q = ffi.cast("int16_t*", p)
    for i = 0, n - 1 do
      local a = 0
      for c = 0, ch - 1 do a = a + q[i * ch + c] end
      x[i] = a / ch / 32768
    end
  elseif bits == 32 then
    local q = ffi.cast("int32_t*", p)
    for i = 0, n - 1 do
      local a = 0
      for c = 0, ch - 1 do a = a + q[i * ch + c] end
      x[i] = a / ch / 2147483648
    end
  else
    for i = 0, n - 1 do
      local a = 0
      for c = 0, ch - 1 do
        local b = i * frame + c * 3
        local v = p[b] + p[b + 1] * 256 + p[b + 2] * 65536
        if v >= 8388608 then v = v - 16777216 end
        a = a + v
      end
      x[i] = a / ch / 8388608
    end
  end
  return { rate = rate, n = n, x = x }
end

--[[
mp3・m4a など（v0.15.0）: Windows の Media Foundation で PCM（float 32 bit）に直し、wav_decode と同じ形にする。
COM の関数は、インターフェースの関数表（vtable）の番号で呼ぶ（IMFAttributes: GetUINT32 7・SetGUID 24 / IMFSourceReader:
SetStreamSelection 4・GetCurrentMediaType 6・SetCurrentMediaType 7・ReadSample 9 / IMFSample: ConvertToContiguousBuffer 41 /
IMFMediaBuffer: Lock 3・Unlock 4 / IUnknown: Release 2）。MFStartup と MFShutdown は読み込み 1 回ごとに対で呼ぶ。
最初の音の時刻が 0 より後なら、その分を無音で埋める（ファイルの先頭 = 0 秒にそろえる）
]]
pcall(ffi.cdef, [[
typedef struct { uint32_t d1; uint16_t d2, d3; uint8_t d4[8]; } PRH_GUID;
typedef struct { void** vt; } PRH_COM;
typedef uint32_t (*PRH_FnRelease)(PRH_COM*);
typedef int32_t (*PRH_FnSetGUID)(PRH_COM*, const PRH_GUID*, const PRH_GUID*);
typedef int32_t (*PRH_FnGetUINT32)(PRH_COM*, const PRH_GUID*, uint32_t*);
typedef int32_t (*PRH_FnStreamSel)(PRH_COM*, uint32_t, int32_t);
typedef int32_t (*PRH_FnGetCurMT)(PRH_COM*, uint32_t, PRH_COM**);
typedef int32_t (*PRH_FnSetCurMT)(PRH_COM*, uint32_t, uint32_t*, PRH_COM*);
typedef int32_t (*PRH_FnReadSample)(PRH_COM*, uint32_t, uint32_t, uint32_t*, uint32_t*, int64_t*, PRH_COM**);
typedef int32_t (*PRH_FnToContig)(PRH_COM*, PRH_COM**);
typedef int32_t (*PRH_FnLock)(PRH_COM*, uint8_t**, uint32_t*, uint32_t*);
typedef int32_t (*PRH_FnUnlock)(PRH_COM*);
int32_t MFStartup(uint32_t version, uint32_t flags);
int32_t MFShutdown(void);
int32_t MFCreateMediaType(PRH_COM** pp);
int32_t MFCreateSourceReaderFromURL(const uint16_t* url, void* attr, PRH_COM** pp);
int32_t CoInitializeEx(void* reserved, uint32_t coinit);
void CoUninitialize(void);
]])

local function guid(d1, d2, d3, b)
  local g = ffi.new("PRH_GUID")
  g.d1, g.d2, g.d3 = d1, d2, d3
  for i = 1, 8 do g.d4[i - 1] = b[i] end
  return g
end

local MF
local function mf_lib()
  if MF == nil then
    local ok, plat, rw, ole = pcall(function() return ffi.load("mfplat"), ffi.load("mfreadwrite"), ffi.load("ole32") end)
    MF = ok and {
      plat = plat, rw = rw, ole = ole,
      MAJOR = guid(0x48eba18e, 0xf8c9, 0x4687, { 0xbf, 0x11, 0x0a, 0x74, 0xc9, 0xf9, 0x6a, 0x8f }),
      SUBTYPE = guid(0xf7e34c9a, 0x42e8, 0x4714, { 0xb7, 0x4b, 0xcb, 0x29, 0xd7, 0x2c, 0x35, 0xe5 }),
      AUDIO = guid(0x73647561, 0x0000, 0x0010, { 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71 }),
      FLOAT = guid(0x00000003, 0x0000, 0x0010, { 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71 }),
      CH = guid(0x37e48bf5, 0x645e, 0x4c5b, { 0x89, 0xde, 0xad, 0xa9, 0xe2, 0x9b, 0x69, 0x6a }),
      RATE = guid(0x5faeeae7, 0x0290, 0x4c31, { 0x9e, 0x8a, 0xc5, 0x34, 0xf6, 0x8d, 0x9d, 0xba }),
    } or false
  end
  return MF or nil
end

local function vcall(o, i, T, ...) return ffi.cast(T, o.vt[i])(o, ...) end
local function release(o) if o ~= nil then vcall(o, 2, "PRH_FnRelease") end end

function M.mf_decode(path)
  local L = mf_lib()
  if not L then return nil, "Media Foundation を読めない" end
  local FIRST_AUDIO, ALL = 0xFFFFFFFD, 0xFFFFFFFE
  local hr_co = L.ole.CoInitializeEx(nil, 0)
  local hr = L.plat.MFStartup(0x00020070, 1)
  if hr < 0 then
    if hr_co == 0 or hr_co == 1 then L.ole.CoUninitialize() end
    return nil, string.format("Media Foundation を始められない（0x%08x）", bit.band(hr, 0xffffffff))
  end
  local rd, mt, cur = ffi.new("PRH_COM*[1]"), ffi.new("PRH_COM*[1]"), ffi.new("PRH_COM*[1]")
  local result, err
  local ok, perr = pcall(function()
    if L.rw.MFCreateSourceReaderFromURL(to_w(path), nil, rd) < 0 or rd[0] == nil then err = "音声ファイルを開けない"; return end
    local R = rd[0]
    vcall(R, 4, "PRH_FnStreamSel", ALL, 0)
    vcall(R, 4, "PRH_FnStreamSel", FIRST_AUDIO, 1)
    if L.plat.MFCreateMediaType(mt) < 0 then err = "Media Foundation の型を作れない"; return end
    vcall(mt[0], 24, "PRH_FnSetGUID", L.MAJOR, L.AUDIO)
    vcall(mt[0], 24, "PRH_FnSetGUID", L.SUBTYPE, L.FLOAT)
    if vcall(R, 7, "PRH_FnSetCurMT", FIRST_AUDIO, nil, mt[0]) < 0 then err = "音を float に直せない（音の無いファイルか、読めない形式）"; return end
    if vcall(R, 6, "PRH_FnGetCurMT", FIRST_AUDIO, cur) < 0 then err = "音の形を読めない"; return end
    local u = ffi.new("uint32_t[1]")
    vcall(cur[0], 7, "PRH_FnGetUINT32", L.CH, u)
    local ch = max(tonumber(u[0]), 1)
    vcall(cur[0], 7, "PRH_FnGetUINT32", L.RATE, u)
    local rate = tonumber(u[0])
    if rate <= 0 then err = "音の形を読めない"; return end
    local cap, n = 1048576, 0
    local x = ffi.new("float[?]", cap)
    local idx, flags, ts, smp = ffi.new("uint32_t[1]"), ffi.new("uint32_t[1]"), ffi.new("int64_t[1]"), ffi.new("PRH_COM*[1]")
    local bufp, pp, mx, cl = ffi.new("PRH_COM*[1]"), ffi.new("uint8_t*[1]"), ffi.new("uint32_t[1]"), ffi.new("uint32_t[1]")
    local first = true
    while true do
      smp[0] = nil
      if vcall(R, 9, "PRH_FnReadSample", FIRST_AUDIO, 0, idx, flags, ts, smp) < 0 then break end
      local S = smp[0]
      if S ~= nil then
        if first then
          -- 最初の音の時刻（100ns）が 0 より後なら、その分を無音で埋める
          local pad = floor(tonumber(ts[0]) * rate / 1e7 + 0.5)
          if pad > 0 and pad < rate * 10 then
            while n + pad > cap do local x2 = ffi.new("float[?]", cap * 2); ffi.copy(x2, x, n * 4); x, cap = x2, cap * 2 end
            ffi.fill(x + n, pad * 4, 0)
            n = n + pad
          end
          first = false
        end
        if vcall(S, 41, "PRH_FnToContig", bufp) >= 0 and bufp[0] ~= nil then
          local Bf = bufp[0]
          if vcall(Bf, 3, "PRH_FnLock", pp, mx, cl) >= 0 then
            local m = floor(tonumber(cl[0]) / (4 * ch))
            while n + m > cap do local x2 = ffi.new("float[?]", cap * 2); ffi.copy(x2, x, n * 4); x, cap = x2, cap * 2 end
            local f = ffi.cast("float*", pp[0])
            for i = 0, m - 1 do
              local a = 0
              for c = 0, ch - 1 do a = a + f[i * ch + c] end
              x[n + i] = a / ch
            end
            n = n + m
            vcall(Bf, 4, "PRH_FnUnlock")
          end
          release(Bf)
        end
        release(S)
      end
      if bit.band(flags[0], 2) ~= 0 then break end     -- MF_SOURCE_READERF_ENDOFSTREAM
    end
    if n == 0 then err = "音が無い"; return end
    result = { rate = rate, n = n, x = x }
  end)
  release(cur[0]); release(mt[0]); release(rd[0])
  L.plat.MFShutdown()
  if hr_co == 0 or hr_co == 1 then L.ole.CoUninitialize() end
  if not ok then return nil, "Media Foundation: " .. tostring(perr) end
  if not result then return nil, err or "読めない" end
  return result
end

--[[
Media Foundation が取り除かない先頭の分（サンプル）。実測（2026-10-03、ffmpeg で作った 44.1kHz / 48kHz）で、
CBR の mp3 は 1729 サンプル（= 先頭の Info フレーム 1152 ＋ LAME タグのエンコーダー遅延 576 ＋ 1）、VBR の mp3 は 577
（Xing フレームは Media Foundation が飛ばす）、Info フレームの無い mp3 は 577、m4a は 1024（edit list の media_time）遅れた。
mp3: ID3v2 の後の最初のフレームが Info なら そのフレームの長さ ＋ LAME タグの遅延、Xing なら遅延だけ（タグが無ければ 0）。
mp4 / m4a: 音のトラック（hdlr が soun）の elst の最初の media_time ÷ mdhd の timescale
]]
local function be32(b, o) return b[o] * 16777216 + b[o + 1] * 65536 + b[o + 2] * 256 + b[o + 3] end
local function mp3_skip(b, size)
  local o = 0
  if size >= 10 and ffi.string(b, 3) == "ID3" then
    o = 10 + b[6] * 2097152 + b[7] * 16384 + b[8] * 128 + b[9]
    if bit.band(b[5], 0x10) ~= 0 then o = o + 10 end
  end
  while o + 4 < size and not (b[o] == 0xFF and bit.band(b[o + 1], 0xE0) == 0xE0) do o = o + 1 end
  if o + 200 > size then return 0 end
  local ver, layer = bit.band(bit.rshift(b[o + 1], 3), 3), bit.band(bit.rshift(b[o + 1], 1), 3)
  if layer ~= 1 then return 0 end
  local mono = bit.band(bit.rshift(b[o + 3], 6), 3) == 3
  local spf = ver == 3 and 1152 or 576
  local side = ver == 3 and (mono and 17 or 32) or (mono and 9 or 17)
  local x = o + 4 + side
  local tag = ffi.string(b + x, 4)
  if tag ~= "Xing" and tag ~= "Info" then return 0 end
  local fl = be32(b, x + 4)
  local l = x + 8 + (bit.band(fl, 1) ~= 0 and 4 or 0) + (bit.band(fl, 2) ~= 0 and 4 or 0) + (bit.band(fl, 4) ~= 0 and 100 or 0) +
            (bit.band(fl, 8) ~= 0 and 4 or 0)
  local delay = 0
  if l + 24 <= size then delay = b[l + 21] * 16 + bit.rshift(b[l + 22], 4) end
  return (tag == "Info" and spf or 0) + delay
end
local function mp4_skip(b, size)
  local skip
  local function walk(o, e, in_trak)
    local media_t, scale, sound
    while o + 8 <= e do
      local sz, ty = be32(b, o), ffi.string(b + o + 4, 4)
      local hd = 8
      if sz == 1 then sz, hd = be32(b, o + 8) * 4294967296 + be32(b, o + 12), 16 elseif sz == 0 then sz = e - o end
      if sz < hd or o + sz > e then break end
      if ty == "moov" or ty == "mdia" or ty == "edts" then
        local m, s_, so = walk(o + hd, o + sz, in_trak)
        media_t, scale, sound = media_t or m, scale or s_, sound or so
      elseif ty == "trak" then
        local m, s_, so = walk(o + hd, o + sz, true)
        if so and m and s_ and s_ > 0 and not skip then skip = m / s_ end
      elseif in_trak and ty == "elst" and o + hd + 8 <= e then
        local v, cnt = b[o + hd], be32(b, o + hd + 4)
        local p = o + hd + 8
        for _ = 1, min(cnt, 64) do
          local mt
          if v == 1 then
            if b[p + 8] >= 128 then mt = -1 else mt = be32(b, p + 8) * 4294967296 + be32(b, p + 12) end
            p = p + 20
          else
            mt = be32(b, p + 4)
            if mt >= 2147483648 then mt = mt - 4294967296 end
            p = p + 12
          end
          if mt >= 0 then media_t = media_t or mt; break end
        end
      elseif in_trak and ty == "mdhd" then
        scale = b[o + hd] == 1 and be32(b, o + hd + 20) or be32(b, o + hd + 12)
      elseif in_trak and ty == "hdlr" then
        sound = ffi.string(b + o + hd + 8, 4) == "soun"
      end
      o = o + sz
    end
    return media_t, scale, sound
  end
  walk(0, size, false)
  return skip or 0
end
function M.audio_skip(buf, size, rate)
  if size < 12 then return 0 end
  if ffi.string(buf + 4, 4) == "ftyp" then return floor(mp4_skip(buf, size) * rate + 0.5) end
  return mp3_skip(buf, size)
end

-- 音声ファイルを PCM にする（WAV はそのまま読み、それ以外は Media Foundation）。直した PCM はファイル（パスと大きさ）ごとに 2 つまで取っておく
local pcm_cache = {}
function M.audio_decode(path)
  local buf, size = M.read_bytes(path)
  if not buf then return nil, "音声ファイルを開けない" end
  local key = path .. "|" .. size
  for _, c in ipairs(pcm_cache) do if c.key == key then return c.W, nil, key end end
  local W, err
  if size >= 12 and ffi.string(buf, 4) == "RIFF" and ffi.string(buf + 8, 4) == "WAVE" then
    W, err = M.wav_decode(buf, size)
  else
    W, err = M.mf_decode(path)
    if W then
      -- Media Foundation が取り除かない先頭の分を落とす（元の配列は W.base で持ち続ける）
      local sk_ = min(M.audio_skip(buf, size, W.rate), W.n - 1)
      if sk_ > 0 then W.base, W.x, W.n = W.x, W.x + sk_, W.n - sk_ end
    end
  end
  buf = nil
  if not W then return nil, err end
  table.insert(pcm_cache, 1, { key = key, W = W })
  pcm_cache[3] = nil
  return W, nil, key
end

--[[
拍（音の立ち上がり）の時刻。W = wav_decode の戻り値、band = 0 全体 / 1 低い音（150Hz より下）/ 2 高い音（2kHz より上）、
sens = 拍の感度 0..100（大きいほど小さな立ち上がりも拾う）。
5ms ごとの音の大きさ（二乗平均）の増えた分 d を作り、前後 0.5 秒の平均 + (2.5 − 2 × 感度) × 標準偏差 を超え、
前後 15ms の中で一番大きく、音の大きさが直前 50ms の一番大きい値を超えた所を拍とする。拍と拍は 0.1 秒以上離す（近ければ大きい方）。
戻り値: 時刻（秒。ファイルの先頭から）の並び, 強さ（一番強い拍が 1）の並び
]]
function M.onsets(W, band, sens)
  local rate, n, x = W.rate, W.n, W.x
  local hop = max(floor(rate / 200), 1)
  local nh = floor(n / hop)
  if nh < 3 then return {}, {} end
  local a1 = 1 - math.exp(-2 * pi * 150 / rate)
  local a2 = 1 - math.exp(-2 * pi * 2000 / rate)
  local lp1, lp2 = 0, 0
  local e = ffi.new("double[?]", nh)
  for k = 0, nh - 1 do
    local acc = 0
    for i = k * hop, k * hop + hop - 1 do
      local v = x[i]
      lp1 = lp1 + a1 * (v - lp1)
      lp2 = lp2 + a2 * (v - lp2)
      local y = v
      if band == 1 then y = lp1 elseif band == 2 then y = v - lp2 end
      acc = acc + y * y
    end
    e[k] = sqrt(acc / hop)
  end
  local d = ffi.new("double[?]", nh)
  local dmax = 0
  d[0] = 0
  for k = 1, nh - 1 do
    local v = e[k] - e[k - 1]
    d[k] = v > 0 and v or 0
    if d[k] > dmax then dmax = d[k] end
  end
  if dmax <= 1e-9 then return {}, {} end
  -- 前後 0.5 秒（100 個）の平均と標準偏差（累積和）
  local c1, c2 = ffi.new("double[?]", nh + 1), ffi.new("double[?]", nh + 1)
  c1[0], c2[0] = 0, 0
  for k = 0, nh - 1 do c1[k + 1] = c1[k] + d[k]; c2[k + 1] = c2[k] + d[k] * d[k] end
  local W2 = 100
  local m = 2.5 - 2 * min(max(sens, 0), 100) / 100
  local times, str = {}, {}
  local last_t, last_v = -huge, 0
  local gap = 0.1
  for k = 1, nh - 1 do
    local v = d[k]
    if v > dmax * 0.02 then
      local lo, hi = max(k - W2, 0), min(k + W2, nh - 1)
      local cnt = hi - lo + 1
      local mu = (c1[hi + 1] - c1[lo]) / cnt
      local var = (c2[hi + 1] - c2[lo]) / cnt - mu * mu
      local thr = mu + m * sqrt(max(var, 0))
      local peak = v > thr
      -- 減っていく途中の揺れ（低い音は 5ms の中で位相がずれて大きさが揺れる）を拾わない: 直前 50ms の一番大きい値を超えた所だけ
      if peak then
        for j = max(k - 10, 0), k - 1 do
          if e[j] >= e[k] then peak = false; break end
        end
      end
      if peak then
        for j = max(k - 3, 0), min(k + 3, nh - 1) do
          if j ~= k and (d[j] > v or (d[j] == v and j < k)) then peak = false; break end
        end
      end
      if peak then
        local t = k * hop / rate
        if t - last_t < gap then
          if v > last_v then times[#times], str[#str], last_t, last_v = t, v, t, v end
        else
          times[#times + 1], str[#str + 1] = t, v
          last_t, last_v = t, v
        end
      end
    end
  end
  local smax = 0
  for _, v in ipairs(str) do smax = max(smax, v) end
  for i = 1, #str do str[i] = smax > 0 and str[i] / smax or 1 end
  return times, str
end

-- 音声ファイルの拍（ファイル・帯・感度ごとに取っておく）。戻り値: 時刻の並び, 強さの並び, 理由（読めないとき）
local beat_cache = {}
function M.file_beats(path, band, sens)
  local W, err, fkey = M.audio_decode(path)
  if not W then return nil, nil, err end
  local key = fkey .. "|" .. band .. "|" .. sens
  local hit = beat_cache[key]
  if hit then return hit[1], hit[2] end
  local t, s_ = M.onsets(W, band, sens)
  beat_cache[key] = { t, s_ }
  return t, s_
end

-- 音の大きさの包絡（v0.15.0）: 10ms ごとの二乗平均の平方根を、一番大きい値で割った並び（0..1）。band は onsets と同じ
function M.loudness(W, band)
  local rate, n, x = W.rate, W.n, W.x
  local hop = max(floor(rate / 100), 1)
  local nh = floor(n / hop)
  local a1 = 1 - math.exp(-2 * pi * 150 / rate)
  local a2 = 1 - math.exp(-2 * pi * 2000 / rate)
  local lp1, lp2 = 0, 0
  local e = ffi.new("double[?]", max(nh, 1))
  local emax = 0
  for k = 0, nh - 1 do
    local acc = 0
    for i = k * hop, k * hop + hop - 1 do
      local v = x[i]
      lp1 = lp1 + a1 * (v - lp1)
      lp2 = lp2 + a2 * (v - lp2)
      local y = v
      if band == 1 then y = lp1 elseif band == 2 then y = v - lp2 end
      acc = acc + y * y
    end
    e[k] = sqrt(acc / hop)
    if e[k] > emax then emax = e[k] end
  end
  if emax > 1e-12 then for k = 0, nh - 1 do e[k] = e[k] / emax end end
  return { dt = hop / rate, n = nh, e = e }
end

-- 時刻 t（秒。ファイルの先頭から）の音の大きさ（区間の間は直線でつなぐ。範囲の外は 0）
function M.loud_at(Lo, t)
  if not Lo or Lo.n == 0 then return 0 end
  local u = t / Lo.dt - 0.5
  if u < 0 or u > Lo.n - 1 then return 0 end
  local i = floor(u)
  local f = u - i
  local a = Lo.e[i]
  return i + 1 < Lo.n and a + (Lo.e[i + 1] - a) * f or a
end

local loud_cache = {}
function M.file_loudness(path, band)
  local W, err, fkey = M.audio_decode(path)
  if not W then return nil, err end
  local key = fkey .. "|" .. band
  local hit = loud_cache[key]
  if hit then return hit end
  local Lo = M.loudness(W, band)
  loud_cache[key] = Lo
  return Lo
end

-- 音の大きさで出す数を変える（v0.15.0）: 元の放出の数 R の、フレームごとの数に 1 ＋ gain / 100 × 音の大きさ を掛けた表にする。
-- 音の大きさは、そのフレームの真ん中の時刻をファイルの時刻（ft(シミュレーション時刻)。オブジェクトの始まり = 0 秒）に直して読む
function M.rate_loud(R, Lo, gain, dt, now, ft)
  local nS = floor(max(now, 0) / dt) + 2
  local smp = {}
  for f = 0, nS - 1 do
    local lam = max(R.count((f + 1) * dt) - R.count(f * dt), 0)
    smp[f + 1] = lam / dt * (1 + gain / 100 * M.loud_at(Lo, ft((f + 0.5) * dt)))
  end
  return M.rate_table(smp, dt)
end

-- BPM のグリッドの拍（シーンの秒）。list = obj.getinfo("bpm_list")、t0..t1 の範囲、div = 1 拍をいくつに分けるか。
-- 強さ: 小節の頭 1 / 拍 0.7 / 拍の間 0.4。区間 i は start_i から次の区間の start まで、拍は start_i + offset_i + k × 60 / tempo
function M.bpm_beats(list, t0, t1, div)
  local times, str = {}, {}
  if not list then return times, str end
  div = max(floor(div or 1), 1)
  for i, B in ipairs(list) do
    local tempo, beat = tonumber(B.tempo) or 0, max(floor(tonumber(B.beat) or 4), 1)
    if tempo > 0 then
      local s0 = tonumber(B.start) or 0
      local s1 = list[i + 1] and tonumber(list[i + 1].start) or huge
      local step = 60 / tempo / div
      local base = s0 + (tonumber(B.offset) or 0)
      local lo, hi = max(s0, t0), min(s1, t1)
      local k = ceil((lo - base) / step - 1e-9)
      while true do
        local t = base + k * step
        if t >= hi - 1e-9 then break end
        if t >= lo - 1e-9 then
          times[#times + 1] = t
          if k % (beat * div) == 0 then str[#str + 1] = 1 elseif k % div == 0 then str[#str + 1] = 0.7 else str[#str + 1] = 0.4 end
        end
        k = k + 1
      end
    end
  end
  return times, str
end

-- 文字列を、1 文字ずつ（UTF-8）か 1 行ずつに分ける。空白・改行だけの単位は除く
function M.split_units(text, by_line)
  local units = {}
  text = (text or ""):gsub("\r\n", "\n")
  if by_line then
    for line in (text .. "\n"):gmatch("(.-)\n") do
      if line:find("%S") then units[#units + 1] = line end
    end
  else
    for ch in text:gmatch("[%z\1-\127\194-\244][\128-\191]*") do
      if ch:find("%S") then units[#units + 1] = ch end
    end
  end
  return units
end

----------------------------------------------------------------------------- 見た目の段（色の変化・フィルター）

local function lerp_color(c0, c1, f)
  local r0, g0, b0 = bit.band(bit.rshift(c0, 16), 255), bit.band(bit.rshift(c0, 8), 255), bit.band(c0, 255)
  local r1, g1, b1 = bit.band(bit.rshift(c1, 16), 255), bit.band(bit.rshift(c1, 8), 255), bit.band(c1, 255)
  local r = floor(r0 + (r1 - r0) * f + 0.5)
  local g = floor(g0 + (g1 - g0) * f + 0.5)
  local b = floor(b0 + (b1 - b0) * f + 0.5)
  return r * 65536 + g * 256 + b
end

-- 色空間の共有部品（v0.18.0）。番号は Script/EffectUtils_ColorSpace.lua と同じ
-- （補間: 0 sRGB / 1 リニア / 2 Oklab / 3 HSV / 4 HSL / 5 Lab / 6 LCh / 7 Oklch / 9 HWB、色差: 0 RGB / 1 YCbCr / 2 Oklab / 3 CIEDE2000）。
-- 既定（どちらも 0）は今までの式のままで、共有部品も読まない。require は Script/ 直下を探す（このモジュールと同じ場所）
local CS
local function colorspace()
  if not CS then CS = require("EffectUtils_ColorSpace") end
  return CS
end

-- 色の段 l（0 始まり）の色
-- 色空間（C.space、v0.18.0）が 0 以外なら、色の並びを色空間で補間する（色相のある色空間は C.hue == 1 で長い方を回る）。
-- 段は 64 まで・色は 64 まで（パレット）なので、並びの変換と段ごとの色は 1 フレームに 1 回だけ求めて C.memo に置く
-- （C は色の変化の節が毎フレーム作り直す。粒子ごとの呼び出しは表を引くだけ）
function M.level_color(C, l)
  local cols, nc = C.cols, C.n
  if C.mode == 5 then return (C.qpal and C.qpal[l + 1]) or cols[1] end
  if C.mode == 1 or C.mode == 2 then return cols[l + 1] or cols[1] end
  if nc <= 1 or C.levels <= 1 then return cols[1] end
  local space = C.space or 0
  if space == 0 then
    -- sRGB: 今までの式（8bit の値の直線補間）
    local f = l / (C.levels - 1) * (nc - 1)
    local i = min(floor(f), nc - 2)
    return lerp_color(cols[i + 1], cols[i + 2], f - i)
  end
  local hue = C.hue or 0
  local memo = C.memo
  if not (memo and memo.cols == cols and memo.n == nc and memo.levels == C.levels and memo.space == space and memo.hue == hue) then
    local cs = colorspace()
    local used = {}
    for i = 1, nc do used[i] = cols[i] end
    memo = { cols = cols, n = nc, levels = C.levels, space = space, hue = hue, stops = cs.prepare(used, space), at = {} }
    C.memo = memo
  end
  local c = memo.at[l]
  if not c then
    local cs = colorspace()
    c = cs.pack(cs.sample(memo.stops, l / (C.levels - 1), hue == 1))
    memo.at[l] = c
  end
  return c
end

-- 出た所の色を、色差（metric、v0.18.0）でパレット pal のいちばん近い色の番号（0 始まり）にする。
-- パレットと色差ごとに、パレットの変換と「色 → 番号」の答えを覚えておく（覚えるパレットは 4 つまで。古いものから捨てる）。
-- 1 フレームに粒子の数だけ呼ばれるが、同じ色は表を引くだけ（新しい色だけ パレットの色の数 回比べる）
local NEAR_CACHE, NEAR_ORDER = {}, {}
local function near_table(pal, metric)
  local key = metric .. ":" .. table.concat(pal, ",")
  local t = NEAR_CACHE[key]
  if t then return t end
  local cs = colorspace()
  local P = {}
  for i = 1, #pal do
    local r, g, b = cs.unpack(pal[i])
    if metric == 2 then
      r, g, b = cs.linear_to_oklab(cs.to_linear(r, g, b))
    elseif metric == 3 then
      r, g, b = cs.xyz_to_lab(cs.linear_to_xyz(cs.to_linear(r, g, b)))
    end
    P[i] = { r, g, b }
  end
  t = { metric = metric, P = P, map = {}, count = 0 }
  NEAR_CACHE[key] = t
  NEAR_ORDER[#NEAR_ORDER + 1] = key
  if #NEAR_ORDER > 4 then NEAR_CACHE[table.remove(NEAR_ORDER, 1)] = nil end
  return t
end

local function nearest_metric(t, c)
  local best = t.map[c]
  if best then return best end
  local cs = colorspace()
  local r, g, b = cs.unpack(c)
  local m, P = t.metric, t.P
  local bd = huge
  best = 0
  if m == 3 then
    local L, A, B = cs.xyz_to_lab(cs.linear_to_xyz(cs.to_linear(r, g, b)))
    for i = 1, #P do
      local q = P[i]
      local d = cs.ciede2000(L, A, B, q[1], q[2], q[3])
      if d < bd then best, bd = i - 1, d end
    end
  elseif m == 2 then
    local L, A, B = cs.linear_to_oklab(cs.to_linear(r, g, b))
    for i = 1, #P do
      local q = P[i]
      local d = (q[1] - L) ^ 2 + (q[2] - A) ^ 2 + (q[3] - B) ^ 2
      if d < bd then best, bd = i - 1, d end
    end
  else
    for i = 1, #P do
      local q = P[i]
      local d = cs.distance(r, g, b, q[1], q[2], q[3], m)
      if d < bd then best, bd = i - 1, d end
    end
  end
  -- 覚える色の数に上限を置く（画像の色が多くても表が膨らみ続けないように）
  if t.count < 65536 then
    t.map[c] = best
    t.count = t.count + 1
  end
  return best
end
M.nearest_metric = function(pal, c, metric) return nearest_metric(near_table(pal, metric), c) end

-- フィルターの段 l の値
function M.level_value(F, l)
  if F.levels <= 1 then return F.v0 end
  return F.v0 + (F.v1 - F.v0) * l / (F.levels - 1)
end

----------------------------------------------------------------------------- 切り抜きの形（中心からの単位の多角形。時計回り）

local SHAPES = {}
local function regular(nv, r_in)
  local t = {}
  for i = 0, nv - 1 do
    local a = (i / nv) * 2 * pi - pi / 2
    local r = (r_in and i % 2 == 1) and r_in or 1
    t[#t + 1] = { r * cos(a), r * sin(a) }
  end
  return t
end
SHAPES[0] = { { -0.5, -0.5 }, { 0.5, -0.5 }, { 0.5, 0.5 }, { -0.5, 0.5 } }   -- 四角（辺の長さ 1）
SHAPES[1] = regular(24)          -- 円
SHAPES[2] = regular(3)           -- 三角
SHAPES[3] = regular(5)           -- 五角
SHAPES[4] = regular(6)           -- 六角
SHAPES[5] = regular(10, 0.45)    -- 星
for i = 1, 5 do for _, v in ipairs(SHAPES[i]) do v[1], v[2] = v[1] * 0.5, v[2] * 0.5 end end   -- 直径 1 にそろえる
M.SHAPES = SHAPES

----------------------------------------------------------------------------- 描くものの並び

--[[
D（呼び手が作る）:
  mat   素材 { kind = 0 元の画像 / 1 切り抜き / 2 図形 / 3 画像フォルダ / 4 テキスト / 5 動画, ... }
  color 色の変化（無ければ nil）{ mode, cols, n, levels, trail, fan }
  filt  フィルター（無ければ nil）{ base, levels, particle, trail, fan, v0, v1, i_v }
  trail 軌跡（無ければ nil）{ mode = 0 並べる / 1 帯 / 2 速度で伸ばす, n, alpha_end, zoom_end, col, width, stretch }
  fan   ファンネル（無ければ nil）
  mesh  メッシュ（無ければ nil）
  old_front, sk, obj_now（オブジェクトの時刻）, total
戻り値 R: R.runs = { { key, vn, alpha, items = { 描くものの番号… } }, ... }、R.cq4 / R.cq3 = 頂点色の頂点の並び、
         R.I = 描くものの表（x y z rx ry rz zoom alpha key shape cu cv csz len ang vx vy）
]]
local function new_inst()
  return { x = {}, y = {}, z = {}, rx = {}, ry = {}, rz = {}, zoom = {}, alpha = {}, key = {}, shape = {},
           cu = {}, cv = {}, cw = {}, ch = {}, poly = {}, csz = {}, len = {}, vx = {}, vy = {}, seed = {}, solid = {}, sk = {}, asx = {}, asy = {}, shr = {}, n = 0 }
end

local function push(I, x, y, z, rx, ry, rz, zoom, alpha, key, shape, vx, vy)
  local n = I.n + 1
  I.n = n
  I.x[n], I.y[n], I.z[n], I.rx[n], I.ry[n], I.rz[n] = x, y, z, rx, ry, rz
  I.zoom[n], I.alpha[n], I.key[n], I.shape[n] = zoom, alpha, key, shape or -1
  I.vx[n], I.vy[n], I.len[n] = vx or 0, vy or 0, 0
  -- 今の粒子の乱数の鍵・縦横比・ゆがみ（軌跡・ファンネルも同じ値）
  I.sk[n], I.asx[n], I.asy[n], I.shr[n] = I.cur_sk, I.cur_asx or 1, I.cur_asy or 1, I.cur_shr or 0
  -- 切り抜く所 = 出た所（自分の画像を並べる）のマス
  I.cu[n], I.cv[n], I.cw[n], I.ch[n], I.poly[n] = I.cur_cu, I.cur_cv, I.cur_cw, I.cur_ch, I.cur_poly
  return n
end

-- 素材の鍵（粒子 i）
local function mat_key(D, res, i, sk)
  local Mt = D.mat
  local k, age = res.k[i], res.age[i]
  local kind = Mt.kind
  if kind == 0 or kind == 1 then return "o" end
  if kind == 2 then return "f" .. (1 + min(floor(rnd(k, CH.mat, sk) * Mt.fignum), Mt.fignum - 1)) end
  if kind == 3 then
    local nf = Mt.count
    if nf <= 0 then return "o" end
    local idx
    if Mt.order == 0 then idx = k % nf
    elseif Mt.order == 1 then idx = min(floor(rnd(k, CH.mat, sk) * nf), nf - 1)
    else idx = floor(age * Mt.fps) % nf end
    return "i" .. (idx + 1)
  end
  if kind == 4 then
    local nu = Mt.count
    if nu <= 0 then return "o" end
    local idx = M.text_index(Mt, k, sk)
    if Mt.switch > 0 then idx = (idx + floor(age / Mt.switch)) % nu end
    return "t" .. (idx + 1)
  end
  if kind == 6 then
    -- カスタム描画: 種類（粒子ごとにでたらめ）と、寿命に沿った時間の段
    local var = min(floor(rnd(k, CH.cust, sk) * Mt.cvar), Mt.cvar - 1)
    local f = min(max(age / max(res.life[i], 1e-6), 0), 1)
    local lv = Mt.clv > 1 and floor(f * (Mt.clv - 1) + 0.5) or 0
    return "u" .. var .. "_" .. lv
  end
  -- 動画: 粒子ごとの再生時刻（あとで段に分ける）
  return "m"
end

local function movie_time(Mt, res, i, sk)
  local k, age = res.k[i], res.age[i]
  local dur = max(Mt.duration, 1e-6)
  local t0 = Mt.randpos and rnd(k, CH.mat + 1, sk) * dur or Mt.pos / 100 * dur
  local spd = vary(Mt.speed, Mt.i_speed, rnd(k, CH.mat + 2, sk)) / 100
  local t = Mt.sync and (Mt.obj_now * spd + t0) or (t0 + age * spd)
  t = t % dur
  return t
end

-- 色の段（粒子 i）
-- 跳ね返りに合わせた進み（v0.14.0。原作のフィルター1リンク）。after = true: 最初に跳ね返ってから寿命の終わりまでで 0 → 1（跳ね返らなければ 0）。
-- false: 生まれてから最初に跳ね返る（先読みの）年齢までで 0 → 1、跳ね返った後は 1（跳ね返らなければ 0）
function M.bounce_progress(res, i, after)
  local age = res.age[i]
  if after then
    local tb = res.tb and res.tb[i] or -1
    if tb < 0 then return 0 end
    return (age - tb) / max(res.life[i] - tb, 1e-6)
  end
  local tb = res.tbp and res.tbp[i] or -1
  if tb < 0 then return 0 end
  if tb <= 0 then return 1 end
  return age / tb
end

local function color_level(D, res, i, sk)
  local C = D.color
  local l
  if C.mode == 5 then
    -- 色差（v0.18.0）: RGB（0）以外なら、出た所の色からパレットの近い色を選び直す（焼き付けに出た所の色が無ければ今までの段）
    local m = C.metric or 0
    local c = (m ~= 0 and res.csrc and C.qpal) and res.csrc[i] or -1
    if c >= 0 then
      if not (C.near and C.near_pal == C.qpal and C.near.metric == m) then C.near, C.near_pal = near_table(C.qpal, m), C.qpal end
      l = nearest_metric(C.near, c)
    else
      l = res.cl and res.cl[i] or 0
    end
  elseif C.mode == 0 or C.mode == 3 or C.mode == 4 or C.mode == 6 or C.mode == 7 then
    -- 寿命で変える / 速さで変える（基準の速さで最後の色）/ 奥行きで変える（奥行きの範囲）/ 跳ね返ってから / 跳ね返るまで
    local f
    if C.mode == 0 then f = res.age[i] / max(res.life[i], 1e-6)
    elseif C.mode == 6 or C.mode == 7 then f = M.bounce_progress(res, i, C.mode == 6)
    elseif C.mode == 3 then f = sqrt(res.vx[i] * res.vx[i] + res.vy[i] * res.vy[i]) / max(C.ref or 1, 1e-6)
    else f = (C.z1 or 1) ~= (C.z0 or 0) and (res.z[i] - (C.z0 or 0)) / ((C.z1 or 1) - (C.z0 or 0)) or 0 end
    f = min(max(f, 0), 1)
    if C.ckind == 1 then f = f >= 1 and 1 or 0
    elseif C.shape then f = min(max(C.shape(f), 0), 1) end
    l = floor(f * (C.levels - 1) + 0.5)
  elseif C.mode == 1 then l = min(floor(rnd(res.k[i], CH.col, sk) * C.n), C.n - 1)
  else l = min(floor(rnd(res.e[i], CH.col, sk) * C.n), C.n - 1) end
  return l
end
M.color_level = color_level

-- 色・フィルターの段の文字（無ければ ""）
--[[
光源（v0.11.0）。L（呼び手が整える）: { kind = 0 平行光 / 1 点光源, dx, dy, dz = 光の進む向き（長さ 1）, px, py, pz = 光の位置,
  amb = 環境光 0..1, pow = 光の強さ 0..2, spec = ハイライト 0..1, levels = 段数, back = 裏も照らす }
表の向き（裏を向いていない側の法線）は -(横 × 縦)。明るさ v = 環境光 + 強さ × max(0, 表の向き · 光へ向かう向き) を 1 で止め、
ハイライト = spec × d^8 を足す（0..2）。段 = round(v × (段数 − 1))。段 / (段数 − 1) が 1 未満なら暗く、1 を超えれば光の色へ寄せる
]]
function M.light_value(L, nx, ny, nz, px, py, pz)
  local lx, ly, lz = L.dx, L.dy, L.dz
  if L.kind == 1 then
    lx, ly, lz = px - L.px, py - L.py, pz - L.pz
    local l = sqrt(lx * lx + ly * ly + lz * lz)
    if l < 1e-9 then lx, ly, lz = 0, 0, 1 else lx, ly, lz = lx / l, ly / l, lz / l end
  end
  local d = -(nx * lx + ny * ly + nz * lz)
  if L.back then d = abs(d) elseif d < 0 then d = 0 end
  local v = min(L.amb + L.pow * d, 1) + L.spec * d ^ 8
  return min(max(v, 0), 2)
end

function M.light_level(L, v)
  return floor(v * (L.levels - 1) + 0.5)
end

-- 四角形の粒子の表の向き（長さ 1）。軸は run_verts と同じ求め方（速度で伸ばす・進行方向を向く）
local function quad_normal(I, ii, opt)
  local rz = I.rz[ii]
  if I.len[ii] > 0 then
    local vx, vy = I.vx[ii], I.vy[ii]
    if vx ~= 0 or vy ~= 0 then rz = atan2(vx, -vy) / RAD end
  elseif opt.facing == 1 then
    local vx, vy = I.vx[ii], I.vy[ii]
    if vx ~= 0 or vy ~= 0 then rz = rz + atan2(vx, -vy) / RAD end
  end
  local axx, axy, axz, ayx, ayy, ayz = basis(I, ii, rz, opt)
  local nx, ny, nz = -(axy * ayz - axz * ayy), -(axz * ayx - axx * ayz), -(axx * ayy - axy * ayx)
  local l = sqrt(nx * nx + ny * ny + nz * nz)
  if l < 1e-12 then return 0, 0, -1 end
  return nx / l, ny / l, nz / l
end
M.quad_normal = quad_normal

-- 立体物の三角形（頂点の並び {x,y,z,u,v} が 3 つずつ）を、光の段ごとに分ける。戻り値は { { lv, 頂点の並び }, ... }（段の小さい順）。
-- 表の向きは -(v1 - v0) × (v2 - v0)（lua.txt「頂点0から3が時計回りになる面が表面」）。
-- closed = 閉じた立体（六面体・球・錐体・双錐体）: 裏の面は見えないので「裏も照らす」を使わない（光に背を向けた面は環境光だけ）
function M.light_split(L, verts, closed)
  if closed and L.back then
    local L2 = {}
    for k, v in pairs(L) do L2[k] = v end
    L2.back = false
    L = L2
  end
  local by, order = {}, {}
  for t = 1, #verts - 2, 3 do
    local a, b, c = verts[t], verts[t + 1], verts[t + 2]
    local ux, uy, uz = b[1] - a[1], b[2] - a[2], b[3] - a[3]
    local wx, wy, wz = c[1] - a[1], c[2] - a[2], c[3] - a[3]
    local nx, ny, nz = -(uy * wz - uz * wy), -(uz * wx - ux * wz), -(ux * wy - uy * wx)
    local l = sqrt(nx * nx + ny * ny + nz * nz)
    if l > 1e-12 then nx, ny, nz = nx / l, ny / l, nz / l else nx, ny, nz = 0, 0, -1 end
    local lv = M.light_level(L, M.light_value(L, nx, ny, nz, (a[1] + b[1] + c[1]) / 3, (a[2] + b[2] + c[2]) / 3, (a[3] + b[3] + c[3]) / 3))
    local list = by[lv]
    if not list then list = {}; by[lv] = list; order[#order + 1] = lv end
    list[#list + 1] = a; list[#list + 1] = b; list[#list + 1] = c
  end
  table.sort(order)
  local out = {}
  for _, lv in ipairs(order) do out[#out + 1] = { lv, by[lv] } end
  return out
end

local function look_suffix(D, res, i, sk, use_color, use_filt)
  local s = ""
  local C, F = D.color, D.filt
  if C and use_color then
    s = s .. "c" .. color_level(D, res, i, sk)
  end
  if F and use_filt then
    local f
    if F.base == 0 then f = res.age[i] / max(res.life[i], 1e-6)
    elseif F.base == 1 then f = D.obj_now / max(D.total, 1e-6)
    elseif F.base == 3 or F.base == 4 then f = M.bounce_progress(res, i, F.base == 3)
    else f = rnd(res.k[i], CH.filt, sk) end
    if F.i_v ~= 0 then f = f + (rnd(res.k[i], CH.filt + 1, sk) * 2 - 1) * F.i_v / 100 end
    f = min(max(f, 0), 1)
    s = s .. "x" .. floor(f * (F.levels - 1) + 0.5)
  end
  return s
end

-- 粒子の並び idx（res の番号の表）を、粒子の番号 k の小さい順にする（v0.16.0）。
-- 比べる関数を渡す table.sort は 10,000 粒で数 ms かかるので、k × 65536 ＋ 並びの番号 の数にして既定の比べ方で並べる
-- （k が同じなら並びの番号の順。k < 2^37・粒子 < 65536 で、double の整数の範囲に収まる）。もう並んでいれば何もしない
function M.sort_by_k(idx, res)
  local n = #idx
  local K = res.k
  local up, down = true, true
  for t = 2, n do
    local a, b = K[idx[t - 1]], K[idx[t]]
    if a > b then up = false elseif a < b then down = false end
    if not up and not down then break end
  end
  if up then return idx end
  if down then
    -- 本ループは新しい粒子（番号の大きい方）から数えるので、子粒子が無ければ逆順に並んでいる
    for t = 1, floor(n / 2) do idx[t], idx[n + 1 - t] = idx[n + 1 - t], idx[t] end
    return idx
  end
  if n >= 65536 then
    table.sort(idx, function(a, b) return K[a] < K[b] end)
    return idx
  end
  for t = 1, n do idx[t] = K[idx[t]] * 65536 + idx[t] end
  table.sort(idx)
  for t = 1, n do idx[t] = idx[t] % 65536 end
  return idx
end

function M.build_draw(res, D)
  local I = new_inst()
  local sk = D.sk
  local n = res.n
  local idx = {}
  for i = 0, n - 1 do idx[i + 1] = i end
  M.sort_by_k(idx, res)
  if D.old_front then
    for i = 1, floor(#idx / 2) do idx[i], idx[#idx + 1 - i] = idx[#idx + 1 - i], idx[i] end
  end
  local Mt, C, F, T, FN = D.mat, D.color, D.filt, D.trail, D.fan
  if C and C.mode ~= 1 and C.mode ~= 2 and C.shape == nil then C.shape = M.curve(C.ckind, C.crep, C.cfn) or false end
  if C and C.mode == 5 then C.qpal = res.qpal end
  -- 動画: 再生時刻を段に分ける（1 フレームに読む動画の絵の数の上限）
  local mframe = {}
  if Mt.kind == 5 then
    local tmin, tmax, ts = huge, -huge, {}
    for _, i in ipairs(idx) do
      local sk = (res.rs and res.rs[i] == 1) and res.rsk or sk
      local t = movie_time(Mt, res, i, sk)
      ts[i] = t
      tmin, tmax = min(tmin, t), max(tmax, t)
    end
    local fps = max(Mt.fps, 1)
    local f0, f1 = floor(tmin * fps), floor(tmax * fps)
    local bin = max(ceil((f1 - f0 + 1) / max(Mt.levels, 1)), 1)
    for i, t in pairs(ts) do mframe[i] = f0 + floor((floor(t * fps) - f0) / bin) * bin end
  end
  local cap = M.MAX_INSTANCES
  local over = 0
  local SH = D.shadow
  if SH then
    -- 影（v0.9.0）: ずらす = 画面の上でずらす / 床に写す = 光の向きで床の高さへ写し、床に寝かせる。鍵の終わりに h を付ける
    local dark = SH.dark / 100
    for _, i in ipairs(idx) do
      local sk = (res.rs and res.rs[i] == 1) and res.rsk or sk
      I.cur_sk = sk
      if res.asx then I.cur_asx, I.cur_asy, I.cur_shr = res.asx[i], res.asy[i], res.shr[i] end
      if res.cu and Mt.kind == 1 and Mt.cwhere == 1 then
        I.cur_cu, I.cur_cv, I.cur_cw, I.cur_ch = res.cu[i], res.cv[i], res.cw[i], res.ch[i]
        I.cur_poly = res.cells and res.cells.poly and res.cells.poly[res.ci[i]] or nil
      else
        I.cur_cu, I.cur_cv, I.cur_cw, I.cur_ch, I.cur_poly = nil, nil, nil, nil, nil
      end
      local mk = mat_key(D, res, i, sk)
      if mk == "m" then mk = "m0" end
      local shape = -1
      if Mt.kind == 1 then
        shape = Mt.shape
        if shape == 6 then shape = 1 + min(floor(rnd(res.k[i], CH.mat + 3, sk) * 5), 4) end
      end
      local x, y, z, rx, ry, rz = res.x[i], res.y[i], res.z[i], res.rx[i], res.ry[i], res.rz[i]
      local a = res.alpha[i] * dark
      local ok_ = true
      if SH.mode == 0 then
        x, y = x + SH.dx, y + SH.dy
      else
        local t_ = SH.floor - y
        if t_ < 0 then ok_ = false end
        x, y, z = x + SH.lx * t_, SH.floor, z + SH.lz * t_
        rx, ry = 90, 0
        if SH.fade > 0 then a = a * max(1 - t_ / SH.fade, 0) end
      end
      if ok_ and a > 0 and I.n < cap then
        local ii = push(I, x, y, z, rx, ry, rz, res.zoom[i], a, mk .. "/" .. look_suffix(D, res, i, sk, false, false) .. "h", shape,
                        res.vx[i], res.vy[i])
        I.seed[ii] = res.k[i]
      end
    end
  end
  for _, i in ipairs(idx) do
    local sk = (res.rs and res.rs[i] == 1) and res.rsk or sk
    I.cur_sk = sk
    if res.asx then I.cur_asx, I.cur_asy, I.cur_shr = res.asx[i], res.asy[i], res.shr[i] end
    if res.cu and Mt.kind == 1 and Mt.cwhere == 1 then
      I.cur_cu, I.cur_cv, I.cur_cw, I.cur_ch = res.cu[i], res.cv[i], res.cw[i], res.ch[i]
      I.cur_poly = res.cells and res.cells.poly and res.cells.poly[res.ci[i]] or nil
    else
      I.cur_cu, I.cur_cv, I.cur_cw, I.cur_ch, I.cur_poly = nil, nil, nil, nil, nil
    end
    local mk = mat_key(D, res, i, sk)
    if mk == "m" then mk = "m" .. (mframe[i] or 0) end
    local k = res.k[i]
    local shape = -1
    if Mt.kind == 1 then
      shape = Mt.shape
      if shape == 6 then shape = 1 + min(floor(rnd(k, CH.mat + 3, sk) * 5), 4) end
    end
    local key = mk .. "/" .. look_suffix(D, res, i, sk, true, not F or F.particle)
    local a, zm = res.alpha[i], res.zoom[i]
    -- 軌跡（粒子より先に描く。古い点から）
    if T and T.mode ~= 1 and res.tn then
      local tkey = mk .. "/" .. look_suffix(D, res, i, sk, not C or C.trail, F and F.trail)
      if T.mode == 0 then
        for j = res.tn, 1, -1 do
          local b = i * res.tn + j - 1
          if res.tok[b] == 1 and I.n < cap then
            local f = j / res.tn
            local ii = push(I, res.tx[b], res.ty[b], res.tz[b], res.rx[i], res.ry[i], res.rz[i],
                            zm * (1 + (T.zoom_end / 100 - 1) * f), a * (1 - T.alpha_end / 100 * f), tkey, shape, res.vx[i], res.vy[i])
            I.seed[ii] = k
            if D.solid then I.solid[ii] = true end
          end
        end
      end
    end
    if I.n < cap then
      local ii = push(I, res.x[i], res.y[i], res.z[i], res.rx[i], res.ry[i], res.rz[i], zm, a, key, shape, res.vx[i], res.vy[i])
      I.seed[ii] = k
      if D.solid then I.solid[ii] = true end
      if T and T.mode == 2 then
        local sp = sqrt(res.vx[i] ^ 2 + res.vy[i] ^ 2)
        I.len[ii] = sp * T.stretch
      end
    else
      over = over + 1
    end
    -- ファンネルと円環
    if FN then
      local fkey = (FN.same and mk or "s") .. "/" .. look_suffix(D, res, i, sk, C and C.fan, F and F.fan)
      local age = res.age[i]
      local tiltx, tilty = 0, 0
      if FN.d3 then tiltx, tilty = rnd(k, CH.fan, sk) * 360, rnd(k, CH.fan + 1, sk) * 360 end
      local nfan = FN.n
      for sidx = 0, nfan - 1 do
        if I.n >= cap then over = over + 1; break end
        local ph = FN.even and (sidx * 360 / nfan) or rnd(k * 64 + sidx, CH.fan + 2, sk) * 360
        local ang = (ph + FN.rev * age) * RAD
        local ox, oy, oz = cos(ang) * FN.rad * zm, sin(ang) * FN.rad * zm, 0
        if FN.d3 then
          local c1, s1 = cos(tiltx * RAD), sin(tiltx * RAD)
          oy, oz = oy * c1 - oz * s1, oy * s1 + oz * c1
          local c2, s2 = cos(tilty * RAD), sin(tilty * RAD)
          ox, oz = ox * c2 + oz * s2, -ox * s2 + oz * c2
        end
        local fz = FN.same and (zm * FN.size / 100) or zm
        push(I, res.x[i] + ox, res.y[i] + oy, res.z[i] + oz, 0, 0, FN.rot * age + ph, fz, a, fkey, FN.same and shape or -1)
      end
      for r = 0, FN.rings - 1 do
        if I.n >= cap then over = over + 1; break end
        local rx_, ry_ = 0, 0
        if FN.d3 then rx_, ry_ = rnd(k * 16 + r, CH.fan + 3, sk) * 180, rnd(k * 16 + r, CH.fan + 4, sk) * 180 end
        push(I, res.x[i], res.y[i], res.z[i], rx_, ry_, 0, zm, a, "r/", -1)
      end
    end
  end
  I.over = over
  -- 光源: 平らな描くもの（粒子・軌跡・ファンネル）の鍵の終わりに光の段 l<段> を足す。影（h）と円環（r/）は足さない
  local LT = D.light
  if LT then
    local lopt = D.lopt or { order = M.STANDARD_ORDER }
    for ii = 1, I.n do
      local key = I.key[ii]
      if not I.solid[ii] and key:sub(-1) ~= "h" and key:sub(1, 2) ~= "r/" then
        local nx, ny, nz = quad_normal(I, ii, lopt)
        I.key[ii] = key .. "l" .. M.light_level(LT, M.light_value(LT, nx, ny, nz, I.x[ii], I.y[ii], I.z[ii]))
      end
    end
  end
  -- まとめる: 鍵・形（四角形か三角形か）・透明度の段が同じものが続く所
  local L = M.ALPHA_LEVELS
  local runs, cur = {}, nil
  for ii = 1, I.n do
    local lv = floor(I.alpha[ii] * L + 0.5)
    if lv > 0 and I.zoom[ii] > 0 then
      local vn = I.solid[ii] and 5 or (I.shape[ii] >= 0 and 3 or 4)
      if not cur or cur.key ~= I.key[ii] or cur.lv ~= lv or cur.vn ~= vn then
        cur = { key = I.key[ii], lv = lv, alpha = lv / L, vn = vn, items = {} }
        runs[#runs + 1] = cur
      end
      cur.items[#cur.items + 1] = ii
    end
  end
  -- 呼び出しや素材の読み込みが多すぎるときは、描く順をあきらめて鍵ごと・段ごとにまとめる
  local switches, last = 0, nil
  for _, r in ipairs(runs) do if r.key ~= last then switches = switches + 1; last = r.key end end
  if #runs > M.MAX_RUNS or switches > M.MAX_SWITCHES then
    local by, order = {}, {}
    for _, r in ipairs(runs) do
      local g = r.key .. "#" .. r.lv .. "#" .. r.vn
      local t = by[g]
      if not t then
        t = { key = r.key, lv = r.lv, alpha = r.alpha, vn = r.vn, items = {} }
        by[g] = t
        order[#order + 1] = g
      end
      for _, it in ipairs(r.items) do t.items[#t.items + 1] = it end
    end
    table.sort(order, function(a, b)
      local ka, kb = by[a], by[b]
      if ka.key ~= kb.key then return ka.key < kb.key end
      if ka.vn ~= kb.vn then return ka.vn < kb.vn end
      return ka.lv < kb.lv
    end)
    runs = {}
    for _, g in ipairs(order) do runs[#runs + 1] = by[g] end
    runs.merged = true
  end
  -- 頂点色で描くもの（帯・メッシュ）
  local cq4, cq3 = {}, {}
  if T and T.mode == 1 and res.tn then M.ribbons(res, idx, T, cq4) end
  if D.mesh then M.mesh(res, idx, D.mesh, cq4, cq3, D) end
  return { I = I, runs = runs, cq4 = cq4, cq3 = cq3, over = over }
end

----------------------------------------------------------------------------- 頂点色の四角形（帯・メッシュ）

local function color_rgb(col)
  return bit.band(bit.rshift(col, 16), 255) / 255, bit.band(bit.rshift(col, 8), 255) / 255, bit.band(col, 255) / 255
end

-- p→q の線を幅 w の四角形にする（画面の XY で直角に広げる）。a0・a1 は両端の不透明度
-- r1, g1, b1 を渡すと、終わりの側をその色にする（帯のグラデーション）
local function line_quad(list, x0, y0, z0, x1, y1, z1, w, r, g, b, a0, a1, r1, g1, b1)
  local dx, dy = x1 - x0, y1 - y0
  local l = sqrt(dx * dx + dy * dy)
  if l < 1e-9 then return end
  r1, g1, b1 = r1 or r, g1 or g, b1 or b
  local nx, ny = -dy / l * w / 2, dx / l * w / 2
  list[#list + 1] = { x0 + nx, y0 + ny, z0, r * a0, g * a0, b * a0, a0 }
  list[#list + 1] = { x1 + nx, y1 + ny, z1, r1 * a1, g1 * a1, b1 * a1, a1 }
  list[#list + 1] = { x1 - nx, y1 - ny, z1, r1 * a1, g1 * a1, b1 * a1, a1 }
  list[#list + 1] = { x0 - nx, y0 - ny, z0, r * a0, g * a0, b * a0, a0 }
end
M.line_quad = line_quad

function M.ribbons(res, idx, T, list)
  local r, g, b = color_rgb(T.col)
  local r2, g2, b2 = r, g, b
  if T.grad == 1 and T.col2 then r2, g2, b2 = color_rgb(T.col2) end
  local tn = res.tn
  -- 色空間（T.space、v0.18.0）: 0（sRGB）は今までの式。ほかは尻尾の区切り（tn + 1 か所。32 まで）の色を 1 フレームに 1 回だけ求める
  local GR, GG, GB
  if T.grad == 1 and T.col2 and (T.space or 0) ~= 0 then
    local cs = colorspace()
    local stops = cs.prepare({ T.col, T.col2 }, T.space)
    GR, GG, GB = {}, {}, {}
    for j = 0, tn do GR[j], GG[j], GB[j] = cs.sample(stops, j / tn, T.hue == 1) end
  end
  for _, i in ipairs(idx) do
    local px, py, pz = res.x[i], res.y[i], res.z[i]
    local a0 = res.alpha[i]
    local w0 = T.width * res.zoom[i]
    for j = 1, tn do
      local bi = i * tn + j - 1
      if res.tok[bi] ~= 1 then break end
      local f1 = j / tn
      local a1 = res.alpha[i] * (1 - T.alpha_end / 100 * f1)
      local w1 = T.width * res.zoom[i] * (1 + (T.zoom_end / 100 - 1) * f1)
      if GR then
        line_quad(list, px, py, pz, res.tx[bi], res.ty[bi], res.tz[bi], (w0 + w1) / 2,
                  GR[j - 1], GG[j - 1], GB[j - 1], a0, a1, GR[j], GG[j], GB[j])
      else
        local f0, f1 = (j - 1) / tn, j / tn
        line_quad(list, px, py, pz, res.tx[bi], res.ty[bi], res.tz[bi], (w0 + w1) / 2,
                  r + (r2 - r) * f0, g + (g2 - g) * f0, b + (b2 - b) * f0, a0, a1,
                  r + (r2 - r) * f1, g + (g2 - g) * f1, b + (b2 - b) * f1)
      end
      px, py, pz, a0, w0 = res.tx[bi], res.ty[bi], res.tz[bi], a1, w1
    end
  end
end

-- メッシュ: 生まれた順の隣、または距離 d 以内の粒子を線で結ぶ（格子に分けて近いものだけ調べる）
-- 点のドロネー三角形分割（ボウヤー・ワトソン法）。戻り値は三角形の頂点番号の 3 つ組の並び。
-- 点を X の小さい順に入れ、外接円が今の点より左で終わる三角形は「済み」に移す（後の点はもっと右なので、もう壊れない）。
-- 辺は番号の組を数にして数える（1 フレームに作り直すので、文字列の鍵を作らない）
function M.delaunay(px, py, n)
  if n < 3 then return {} end
  local x0, y0, x1, y1 = huge, huge, -huge, -huge
  for i = 1, n do x0, y0, x1, y1 = min(x0, px[i]), min(y0, py[i]), max(x1, px[i]), max(y1, py[i]) end
  local d = max(x1 - x0, y1 - y0, 1) * 20
  local mx, my = (x0 + x1) / 2, (y0 + y1) / 2
  local X, Y = {}, {}
  for i = 1, n do X[i], Y[i] = px[i], py[i] end
  X[n + 1], Y[n + 1] = mx - d, my - d
  X[n + 2], Y[n + 2] = mx + d, my - d
  X[n + 3], Y[n + 3] = mx, my + d
  local W = n + 4
  local function make(a, b, c)
    local ax, ay, bx, by, cx, cy = X[a], Y[a], X[b], Y[b], X[c], Y[c]
    local dd = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by))
    if abs(dd) < 1e-12 then return { a, b, c, 0, 0, huge, huge } end
    local a2, b2, c2 = ax * ax + ay * ay, bx * bx + by * by, cx * cx + cy * cy
    local ux = (a2 * (by - cy) + b2 * (cy - ay) + c2 * (ay - by)) / dd
    local uy = (a2 * (cx - bx) + b2 * (ax - cx) + c2 * (bx - ax)) / dd
    local r2 = (ax - ux) ^ 2 + (ay - uy) ^ 2
    return { a, b, c, ux, uy, r2, ux + sqrt(r2) }
  end
  local order = {}
  for i = 1, n do order[i] = i end
  table.sort(order, function(a, b) if X[a] ~= X[b] then return X[a] < X[b] end return Y[a] < Y[b] end)
  local active, done = { make(n + 1, n + 2, n + 3) }, {}
  for _, p in ipairs(order) do
    local x, y = X[p], Y[p]
    local edges, keyl, keep = {}, {}, {}
    for _, t in ipairs(active) do
      if t[7] < x then
        done[#done + 1] = t
      elseif (x - t[4]) ^ 2 + (y - t[5]) ^ 2 < t[6] then
        for e = 1, 3 do
          local a, b = t[e], t[e % 3 + 1]
          local key = a < b and a * W + b or b * W + a
          local cur = edges[key]
          if cur == nil then
            edges[key] = a * W + b
            keyl[#keyl + 1] = key
          else
            edges[key] = false
          end
        end
      else
        keep[#keep + 1] = t
      end
    end
    active = keep
    for _, key in ipairs(keyl) do
      local ab = edges[key]
      if ab then active[#active + 1] = make(floor(ab / W), ab % W, p) end
    end
  end
  for _, t in ipairs(active) do done[#done + 1] = t end
  local out = {}
  for _, t in ipairs(done) do
    if t[1] <= n and t[2] <= n and t[3] <= n then out[#out + 1] = { t[1], t[2], t[3] } end
  end
  return out
end

-- 粒子 i の色（色の変化があればその段の色、無ければ c0）
local function particle_rgb(D, res, i, sk, c0)
  if D.color then return M.level_color(D.color, color_level(D, res, i, sk)) end
  return c0
end

function M.mesh(res, idx, Ms, list4, list3, D)
  local r, g, b = color_rgb(Ms.col)
  if Ms.rule == 2 then
    -- 三角形で塗る（v0.9.0）: 画面の XY でドロネー分割し、三角形を塗る（新しい粒子から M.MAX_DELAUNAY 個まで）
    local use = {}
    for t = #idx, 1, -1 do
      local i = idx[t]
      if res.alpha[i] > 0 or not Ms.mul then use[#use + 1] = i end   -- 粒子の透過率を掛けないなら、見えない粒子も頂点に使う
      if #use >= M.MAX_DELAUNAY then break end
    end
    local px, py = {}, {}
    for t, i in ipairs(use) do px[t], py[t] = res.x[i], res.y[i] end
    local tris = M.delaunay(px, py, #use)
    local base_a = 1 - Ms.alpha / 100
    local me2 = (Ms.maxedge or 0) > 0 and Ms.maxedge * Ms.maxedge or huge
    local sk = D and D.sk or 0
    local drawn_e = {}
    for _, t in ipairs(tris) do
      local i, j, k = use[t[1]], use[t[2]], use[t[3]]
      local function d2(a, c) return (res.x[a] - res.x[c]) ^ 2 + (res.y[a] - res.y[c]) ^ 2 end
      if d2(i, j) <= me2 and d2(j, k) <= me2 and d2(k, i) <= me2 then
        local a = base_a * (Ms.mul and min(res.alpha[i], res.alpha[j], res.alpha[k]) or 1)
        if a > 0 then
          local cs = {}
          if Ms.fill == 1 and Ms.cimg then
            -- 他レイヤーの画像の色: 三角形の重心の色（形の格子のマス）。重心が画像の外か透明な所なら、その三角形は塗らない
            local gx = (res.x[i] + res.x[j] + res.x[k]) / 3
            local gy = (res.y[i] + res.y[j] + res.y[k]) / 3
            local c
            local X, Y, _, rz, sx, sy, cx, cy = Ms.cimg.lay()
            if X then
              local Mk = Ms.cimg.mask
              local lx, ly = layer_unpoint(X, Y, rz, sx, sy, cx, cy, gx, gy)
              local fx, fy = floor((lx + Mk.w / 2) / Mk.cs), floor((ly + Mk.h / 2) / Mk.cs)
              if fx >= 0 and fy >= 0 and fx < Mk.gw and fy < Mk.gh and Mk.a[fy * Mk.gw + fx] > 0 then c = Mk.col[fy * Mk.gw + fx] end
            end
            if not c then a = 0 end
            cs[1], cs[2], cs[3] = c, c, c
          elseif Ms.fill == 0 then
            cs[1], cs[2], cs[3] = particle_rgb(D, res, i, sk, Ms.col), particle_rgb(D, res, j, sk, Ms.col), particle_rgb(D, res, k, sk, Ms.col)
          else
            cs[1], cs[2], cs[3] = Ms.col, Ms.col, Ms.col
          end
          if a > 0 then
            for v, p in ipairs({ i, j, k }) do
              local cr, cg, cb = color_rgb(cs[v])
              list3[#list3 + 1] = { res.x[p], res.y[p], res.z[p], cr * a, cg * a, cb * a, a }
            end
          end
          if Ms.edge then
            for _, ed in ipairs({ { i, j }, { j, k }, { k, i } }) do
              local p1, p2 = ed[1], ed[2]
              local key = p1 < p2 and (p1 .. ":" .. p2) or (p2 .. ":" .. p1)
              if not drawn_e[key] then
                drawn_e[key] = true
                line_quad(list4, res.x[p1], res.y[p1], res.z[p1], res.x[p2], res.y[p2], res.z[p2], Ms.width, r, g, b, base_a, base_a)
              end
            end
          end
        end
      end
    end
    return
  end
  local base_a = 1 - Ms.alpha / 100
  local cnt = {}
  local nn = #idx
  local function alpha_of(i, j, d)
    local a = base_a
    if Ms.mul then a = a * min(res.alpha[i], res.alpha[j]) end
    if Ms.fade and Ms.rule == 1 then a = a * max(1 - d / Ms.dist, 0) end
    return a
  end
  local function same_group(i, j) return not Ms.group or res.e[i] == res.e[j] end
  if Ms.rule == 0 then
    for t = 1, nn - 1 do
      local i, j = idx[t], idx[t + 1]
      if same_group(i, j) then
        local d = sqrt((res.x[i] - res.x[j]) ^ 2 + (res.y[i] - res.y[j]) ^ 2 + (res.z[i] - res.z[j]) ^ 2)
        local a = alpha_of(i, j, d)
        if a > 0 then line_quad(list4, res.x[i], res.y[i], res.z[i], res.x[j], res.y[j], res.z[j], Ms.width, r, g, b, a, a) end
      end
    end
    if Ms.face then
      for t = 1, nn - 2 do
        local i, j, k = idx[t], idx[t + 1], idx[t + 2]
        if same_group(i, j) and same_group(j, k) then
          local a = base_a * (Ms.mul and min(res.alpha[i], res.alpha[j], res.alpha[k]) or 1) * 0.5
          for _, p in ipairs({ i, j, k }) do
            list3[#list3 + 1] = { res.x[p], res.y[p], res.z[p], r * a, g * a, b * a, a }
          end
        end
      end
    end
    return
  end
  local cs = max(Ms.dist, 1)
  local grid = {}
  local function cell(x, y, z) return floor(x / cs) .. "," .. floor(y / cs) .. "," .. floor(z / cs) end
  local pos = {}
  for t, i in ipairs(idx) do
    pos[i] = t
    local c = cell(res.x[i], res.y[i], res.z[i])
    local lst = grid[c]
    if not lst then lst = {}; grid[c] = lst end
    lst[#lst + 1] = i
  end
  local d2max = Ms.dist * Ms.dist
  for t, i in ipairs(idx) do
    if (cnt[i] or 0) < Ms.max then
      local cx, cy, cz = floor(res.x[i] / cs), floor(res.y[i] / cs), floor(res.z[i] / cs)
      local cand = {}
      for dx = -1, 1 do for dy = -1, 1 do for dz = -1, 1 do
        local lst = grid[(cx + dx) .. "," .. (cy + dy) .. "," .. (cz + dz)]
        if lst then
          for _, j in ipairs(lst) do
            if pos[j] > t and same_group(i, j) then
              local d2 = (res.x[i] - res.x[j]) ^ 2 + (res.y[i] - res.y[j]) ^ 2 + (res.z[i] - res.z[j]) ^ 2
              if d2 <= d2max then cand[#cand + 1] = { j, d2 } end
            end
          end
        end
      end end end
      table.sort(cand, function(p, q) if p[2] ~= q[2] then return p[2] < q[2] end return pos[p[1]] < pos[q[1]] end)
      for _, c in ipairs(cand) do
        if (cnt[i] or 0) >= Ms.max then break end
        local j = c[1]
        if (cnt[j] or 0) < Ms.max then
          local d = sqrt(c[2])
          local a = alpha_of(i, j, d)
          if a > 0 then line_quad(list4, res.x[i], res.y[i], res.z[i], res.x[j], res.y[j], res.z[j], Ms.width, r, g, b, a, a) end
          cnt[i], cnt[j] = (cnt[i] or 0) + 1, (cnt[j] or 0) + 1
        end
      end
    end
  end
end


----------------------------------------------------------------------------- ガイドの線・時計・編集中の間引き（v0.6.0）

-- ガイドの線（頂点の色の四角形。obj.drawpoly(表, 4) で描く）。X = 拡張の表、fields = 整えた力場の並び
function M.guide(X, fields, width)
  local list = {}
  local w = width or 2
  local function seg(x0, y0, z0, x1, y1, z1, r, g, b) line_quad(list, x0, y0, z0, x1, y1, z1, w, r, g, b, 0.85, 0.85) end
  local function cross(x, y, z, s, r, g, b)
    seg(x - s, y, z, x + s, y, z, r, g, b)
    seg(x, y - s, z, x, y + s, z, r, g, b)
  end
  local function circle(x, y, z, R, r, g, b)
    for i = 0, 63 do
      local a0, a1 = i / 64 * 2 * pi, (i + 1) / 64 * 2 * pi
      seg(x + R * cos(a0), y + R * sin(a0), z, x + R * cos(a1), y + R * sin(a1), z, r, g, b)
    end
  end
  local L = 4000
  local B = X.bnc
  if B then
    if B.bx ~= 3 then
      if B.bx ~= 2 then seg(B.xmin, -L, 0, B.xmin, L, 0, 0.2, 0.8, 1) end
      if B.bx ~= 1 then seg(B.xmax, -L, 0, B.xmax, L, 0, 0.2, 0.8, 1) end
    end
    if B.by ~= 3 then
      if B.by ~= 2 then seg(-L, B.ymin, 0, L, B.ymin, 0, 0.2, 0.8, 1) end
      if B.by ~= 1 then seg(-L, B.ymax, 0, L, B.ymax, 0, 0.2, 0.8, 1) end
    end
    if B.sph ~= 0 then
      local sp = B.spos or {}
      circle(sp[1] or 0, sp[2] or 0, sp[3] or 0, B.srad, 0.2, 0.8, 1)
    end
  end
  for _, G in ipairs(fields or {}) do
    cross(G.cx, G.cy, G.cz, 16, 1, 0.6, 0.1)
    if G.type == 3 then
      cross(G.qx, G.qy, G.qz, 16, 1, 0.6, 0.1)
      seg(G.cx, G.cy, G.cz, G.qx, G.qy, G.qz, 1, 0.6, 0.1)
    elseif G.type == 2 then
      circle(G.cx, G.cy, G.cz, max(G.width, 4), 1, 0.6, 0.1)
    end
    if G.range > 0 then circle(G.cx, G.cy, G.cz, G.range, 1, 0.6, 0.1) end
  end
  local A = X.att
  if A and A.pts then
    for j = 0, floor(#A.pts / 3) - 1 do cross(A.pts[j * 3 + 1], A.pts[j * 3 + 2], A.pts[j * 3 + 3], 16, 0.4, 1, 0.4) end
  end
  local Wd = X.wind
  if Wd and Wd.use_point then
    local pt = Wd.pt or {}
    cross(pt[1] or 0, pt[2] or 0, pt[3] or 0, 16, 1, 1, 0.3)
    circle(pt[1] or 0, pt[2] or 0, pt[3] or 0, Wd.range, 1, 1, 0.3)
  end
  -- パス（紫）と、形へ集まるのアンカー（紫の十字）
  if X.path then
    local PT = M.prepare_path(X.path)
    if PT then
      for i = 1, PT.m - 1 do seg(PT.x[i], PT.y[i], PT.z[i], PT.x[i + 1], PT.y[i + 1], PT.z[i + 1], 0.8, 0.5, 1) end
    end
  end
  local GA = X.gather
  if GA and (GA.src == 2 or GA.src == 3) and GA.pts then
    local np = GA.src == 2 and 1 or max(min(floor(GA.np or 1), floor(#GA.pts / 3)), 1)
    for j = 0, np - 1 do cross(GA.pts[j * 3 + 1], GA.pts[j * 3 + 2], GA.pts[j * 3 + 3], 16, 0.8, 0.5, 1) end
  end
  return list
end

pcall(ffi.cdef, "int QueryPerformanceCounter(int64_t* c); int QueryPerformanceFrequency(int64_t* f);")
local qpc_f
-- 秒（状態の表示で計算と描画の時間を測る）
function M.clock()
  local K = kernel()
  if not qpc_f then
    local f = ffi.new("int64_t[1]")
    K.QueryPerformanceFrequency(f)
    qpc_f = tonumber(f[0])
  end
  local c = ffi.new("int64_t[1]")
  K.QueryPerformanceCounter(c)
  return tonumber(c[0]) / qpc_f
end

-- 編集中の粒子の割合: 粒子の番号から決まる割合だけを描く（計算した値は変えず、描かない粒子の拡大率と透過率を 0 にする）
function M.thin(res, pct, sk)
  if pct >= 100 then return end
  for i = 0, res.n - 1 do
    if rnd(res.k[i], CH.prev, sk) * 100 >= pct then res.zoom[i], res.alpha[i] = 0, 0 end
  end
end

----------------------------------------------------------------------------- まとまりの頂点

-- 描くもの ii の面の横（ax）と縦（ay）の向き（長さ 1）。点 (lx, ly) は 中心 + lx*ax + ly*ay に置く
basis = function(I, ii, rz, opt)
  if opt.facing == 2 and opt.cam then
    local c = opt.cam
    local ca, sa = cos(rz * RAD), sin(rz * RAD)
    return c.rx * ca + c.ux * sa, c.ry * ca + c.uy * sa, c.rz * ca + c.uz * sa,
           -c.rx * sa + c.ux * ca, -c.ry * sa + c.uy * ca, -c.rz * sa + c.uz * ca
  end
  local ang1, ang2, ang3 = I.rx[ii] * RAD, I.ry[ii] * RAD, rz * RAD
  local axx, axy, axz, ayx, ayy, ayz = 1, 0, 0, 0, 1, 0
  for _, axis in ipairs(ORDERS[opt.order]) do
    local a = axis == 1 and ang1 or (axis == 2 and ang2 or ang3)
    if a ~= 0 then
      local c, s_ = cos(a), sin(a)
      axx, axy, axz = rot_axis(axis, c, s_, axx, axy, axz)
      ayx, ayy, ayz = rot_axis(axis, c, s_, ayx, ayy, ayz)
    end
  end
  return axx, axy, axz, ayx, ayy, ayz
end

----------------------------------------------------------------------------- 立体物

-- 切り抜く四角（素材の中の中心 cu,cv と、貼る範囲 cw,ch px）。run_verts の切り抜きと同じ乱数
local function cut_rect(I, ii, w, h, opt)
  local csz = opt.cut_size
  local cw, ch = min(csz, w), min(csz, h)
  local cu, cv = 0.5, 0.5
  if not opt.cut_center then
    local mu, mv = cw / 2 / w, ch / 2 / h
    cu = mu + (1 - 2 * mu) * rnd(I.seed[ii], CH.mat + 5, I.sk[ii] or opt.sk)
    cv = mv + (1 - 2 * mv) * rnd(I.seed[ii], CH.mat + 6, I.sk[ii] or opt.sk)
  end
  return cu, cv, cw, ch
end

--[[
S（呼び手が作る）: { type = 0 六面体 / 1 球 / 2 錐体 / 3 双錐体 / 4 曲面 / 5 厚み, div = 分割数, size = 大きさ（0 は素材の大きさ）,
                    bh = 横の曲がり %, bv = 縦の曲がり %, depth = 奥行き px }
型紙: 三角形の頂点の並び { x, y, zw, zh, zd, u, v }。中心が原点で、幅・高さ・奥行きが 1。
      描くときの座標は (x × 幅, y × 高さ, zw × 幅 + zh × 高さ + zd × 奥行き)。+Z が奥
]]
local function solid_template(S)
  local T = {}
  local ty, n = S.type, max(floor(S.div or 8), 3)
  local function V(x, y, zw, zh, zd, u, v) return { x, y, zw, zh, zd, u, v } end
  local function tri(a, b, c, convex)
    if convex then
      -- 閉じた凸の形: 外から見て時計回りを表にする（lua.txt の obj.drawpoly「頂点0から3が時計回りになる面が表面」）。
      -- 画面は Y が下・Z が奥なので、時計回りの面の外積は外向きと逆を向く
      local ax_, ay_, az_ = a[1], a[2], a[3] + a[4] + a[5]
      local bx_, by_, bz_ = b[1], b[2], b[3] + b[4] + b[5]
      local cx_, cy_, cz_ = c[1], c[2], c[3] + c[4] + c[5]
      local ux_, uy_, uz_ = bx_ - ax_, by_ - ay_, bz_ - az_
      local vx_, vy_, vz_ = cx_ - ax_, cy_ - ay_, cz_ - az_
      local nx_, ny_, nz_ = uy_ * vz_ - uz_ * vy_, uz_ * vx_ - ux_ * vz_, ux_ * vy_ - uy_ * vx_
      if nx_ * (ax_ + bx_ + cx_) + ny_ * (ay_ + by_ + cy_) + nz_ * (az_ + bz_ + cz_) > 0 then b, c = c, b end
    end
    T[#T + 1] = a
    T[#T + 1] = b
    T[#T + 1] = c
  end
  if ty == 0 then
    -- 六面体: 6 面それぞれに画像全体を貼る。{ 面の中心, 右, 下 }（各 -1..1 の向き）
    local F = {
      { 0, 0, -1, 1, 0, 0, 0, 1, 0 }, { 0, 0, 1, -1, 0, 0, 0, 1, 0 },
      { 1, 0, 0, 0, 0, 1, 0, 1, 0 }, { -1, 0, 0, 0, 0, -1, 0, 1, 0 },
      { 0, -1, 0, 1, 0, 0, 0, 0, 1 }, { 0, 1, 0, 1, 0, 0, 0, 0, -1 },
    }
    for _, f in ipairs(F) do
      local function at(s, t)
        return V(0.5 * (f[1] + s * f[4] + t * f[7]), 0.5 * (f[2] + s * f[5] + t * f[8]), 0, 0,
                 0.5 * (f[3] + s * f[6] + t * f[9]), (s + 1) / 2, (t + 1) / 2)
      end
      local p00, p10, p11, p01 = at(-1, -1), at(1, -1), at(1, 1), at(-1, 1)
      tri(p00, p10, p11, true)
      tri(p00, p11, p01, true)
    end
  elseif ty == 1 then
    -- 球: 経度 n・緯度 n/2 に分ける。画像の中心が正面
    local m = max(floor(n / 2), 2)
    local function at(i, j)
      local th, ph = (i / n - 0.5) * 2 * pi, j / m * pi
      return V(0.5 * sin(ph) * sin(th), -0.5 * cos(ph), 0, 0, -0.5 * sin(ph) * cos(th), i / n, j / m)
    end
    for j = 0, m - 1 do
      for i = 0, n - 1 do
        local a, b, c, d = at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)
        if j > 0 then tri(a, b, c, true) end
        if j < m - 1 then tri(a, c, d, true) end
      end
    end
  elseif ty == 2 or ty == 3 then
    -- 錐体（上が尖る。底は円）/ 双錐体（上下が尖る）。側面は画像を一周させて貼る
    local ring_y = ty == 2 and 0.5 or 0
    local vr = ty == 2 and 1 or 0.5
    local function ring(k)
      local th = (k / n - 0.5) * 2 * pi
      return 0.5 * sin(th), -0.5 * cos(th)
    end
    for k = 0, n - 1 do
      local x0, z0 = ring(k)
      local x1, z1 = ring(k + 1)
      local um = (k + 0.5) / n
      tri(V(0, -0.5, 0, 0, 0, um, 0), V(x0, ring_y, 0, 0, z0, k / n, vr), V(x1, ring_y, 0, 0, z1, (k + 1) / n, vr), true)
      if ty == 3 then
        tri(V(0, 0.5, 0, 0, 0, um, 1), V(x1, 0, 0, 0, z1, (k + 1) / n, 0.5), V(x0, 0, 0, 0, z0, k / n, 0.5), true)
      else
        tri(V(0, 0.5, 0, 0, 0, 0.5, 0.5), V(x1, 0.5, 0, 0, z1, 0.5 + x1, 0.5 + z1), V(x0, 0.5, 0, 0, z0, 0.5 + x0, 0.5 + z0), true)
      end
    end
  elseif ty == 4 then
    -- 曲面: 横（縦）の曲がり 100% で、幅（高さ）を円周とする筒に巻き付く。正なら縁が奥へ曲がる
    local ah, av = (S.bh or 0) / 100 * 2 * pi, (S.bv or 0) / 100 * 2 * pi
    local function at(i, j)
      local u, v = i / n, j / n
      local x, zw = u - 0.5, 0
      if abs(ah) > 1e-6 then
        local f = (u - 0.5) * ah
        x, zw = sin(f) / ah, (1 - cos(f)) / ah
      end
      local y, zh = v - 0.5, 0
      if abs(av) > 1e-6 then
        local f = (v - 0.5) * av
        y, zh = sin(f) / av, (1 - cos(f)) / av
      end
      return V(x, y, zw, zh, 0, u, v)
    end
    for j = 0, n - 1 do
      for i = 0, n - 1 do
        local a, b, c, d = at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)
        tri(a, b, c, false)
        tri(a, c, d, false)
      end
    end
  else
    -- 厚み: 同じ絵を奥行きの方向に 分割数 枚重ねる
    for s = 0, n - 1 do
      local zd = s / (n - 1) - 0.5
      local a, b = V(-0.5, -0.5, 0, 0, zd, 0, 0), V(0.5, -0.5, 0, 0, zd, 1, 0)
      local c, d = V(0.5, 0.5, 0, 0, zd, 1, 1), V(-0.5, 0.5, 0, 0, zd, 0, 1)
      tri(a, b, c, false)
      tri(a, c, d, false)
    end
  end
  return T
end
M.solid_template = solid_template

-- 立体物のまとまりの頂点（三角形の頂点の並び {x,y,z,u,v}。u,v は 0..1）。obj.drawpoly(表, 3, 透明度) で描く。
-- 六面体・球・錐体・双錐体は閉じた凸の形なので、呼び手が裏面を表示しない（culling）にする。
-- 曲面・厚みは裏も見えるので、粒子ごとに奥の三角形から並べる（opt.eye = 目の位置）
--[[
pmd モデル（v0.17.0）: 静止した形だけを読む（頂点の位置・UV・面・材質。ボーン・モーフ・表情は読まない）。
形（リトルエンディアン）: "Pmd" / 版 float / 名前 20 / コメント 256 → 頂点の数 uint32 と 1 頂点 38 バイト（位置 float×3・法線 float×3・UV float×2・
ボーン uint16×2・重み uint8・輪郭 uint8）→ 面の頂点の数 uint32 と uint16 の並び → 材質の数 uint32 と 1 材質 70 バイト
（拡散色 float×3・不透明度 float・反射の強さ float・反射色 float×3・環境色 float×3・トゥーン uint8・輪郭 uint8・面の頂点の数 uint32・テクスチャ char[20]）。
テクスチャの名前は Shift-JIS（UTF-8 に直す）。「a.bmp*b.sph」の * より後と、.sph / .spa だけの名前は使わない（スフィアマップ）。
戻り値: { nv, pos（float[nv×3]）, uv（float[nv×2]）, ni, idx（uint16[ni]）, mats = { { r, g, b, a, first, count, tex（フルパスか nil） } } } か nil, 理由
]]
local function sjis_to_utf8(p, n)
  local len = 0
  while len < n and p[len] ~= 0 do len = len + 1 end
  if len == 0 then return "" end
  local K = kernel()
  local wn = K.MultiByteToWideChar(932, 0, ffi.cast("const char*", p), len, nil, 0)
  if wn <= 0 then return ffi.string(p, len) end
  local w = ffi.new("uint16_t[?]", wn + 1)
  K.MultiByteToWideChar(932, 0, ffi.cast("const char*", p), len, w, wn)
  w[wn] = 0
  return from_w(w)
end
local model_cache = {}
function M.pmd_load(path)
  if not path or path == "" then return nil, "モデルファイルが選ばれていない" end
  local buf, size = M.read_bytes(path)
  if not buf then return nil, "モデルファイルを開けない" end
  local key = path .. "|" .. size
  local hit = model_cache[key]
  if hit then return hit end
  if size < 283 + 4 or ffi.string(buf, 3) ~= "Pmd" then return nil, "pmd ではない（pmx などは読めない）" end
  local o = 283
  local function u32(at) return buf[at] + buf[at + 1] * 256 + buf[at + 2] * 65536 + buf[at + 3] * 16777216 end
  local nv = u32(o); o = o + 4
  if nv <= 0 or o + nv * 38 + 4 > size then return nil, "pmd の頂点が壊れている" end
  local pos, uv = ffi.new("float[?]", nv * 3), ffi.new("float[?]", nv * 2)
  for i = 0, nv - 1 do
    local f = ffi.cast("float*", buf + o + i * 38)
    pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2] = f[0], f[1], f[2]
    uv[i * 2], uv[i * 2 + 1] = f[6], f[7]
  end
  o = o + nv * 38
  local ni = u32(o); o = o + 4
  if ni % 3 ~= 0 or o + ni * 2 + 4 > size then return nil, "pmd の面が壊れている" end
  local idx = ffi.new("uint16_t[?]", max(ni, 1))
  ffi.copy(idx, buf + o, ni * 2)
  for i = 0, ni - 1 do
    if idx[i] >= nv then return nil, "pmd の面が範囲の外の頂点を指している" end
  end
  o = o + ni * 2
  local nm = u32(o); o = o + 4
  if o + nm * 70 > size then return nil, "pmd の材質が壊れている" end
  local BS = string.char(92)
  local dir = path:match("^(.*[" .. BS .. "/])") or ""
  local mats, first = {}, 0
  for m = 0, nm - 1 do
    local b = o + m * 70
    local f = ffi.cast("float*", buf + b)
    local cnt = u32(b + 46)
    local name = sjis_to_utf8(buf + b + 50, 20)
    name = name:match("^([^*]*)") or ""
    local ext = (name:match("%.([^%.]+)$") or ""):lower()
    local tex
    if name ~= "" and ext ~= "sph" and ext ~= "spa" then tex = dir .. name end
    mats[#mats + 1] = { r = f[0], g = f[1], b = f[2], a = f[3], first = first, count = cnt, tex = tex }
    first = first + cnt
  end
  if first > ni then return nil, "pmd の材質の面の数が面の数より多い" end
  local Md = { nv = nv, pos = pos, uv = uv, ni = ni, idx = idx, mats = mats, buf = buf }
  model_cache[key] = Md
  return Md
end

--[[
モデルの材質 mi の三角形を、ラン run の粒子ごとに置いた頂点の並び（{x,y,z,u,v}。u,v は 0..1）。
モデルの 1 単位 = 10px × モデルの拡大率 × 粒子の拡大率。Y は上向き（MMD）なので反転する。画面も Y が下なので、見た目の上では同じ絵になり、
三角形の表（外から見て時計回り）もそのまま保たれる（頂点の順は入れ替えない）。
モデルの (0, 中心の高さ, 0) を粒子の位置に置く。三角形は奥から順に並べる（材質の中だけ）。tex が無い材質は u,v を 0.5 にする（拡散色の四角を貼る）
]]
function M.model_verts(R, run, Md, mi, opt)
  local Mt = Md.mats[mi]
  local S = opt.solid
  local I = R.I
  -- 材質の三角形の頂点を、材質ごとに 1 回だけ並べておく（速さのため。v0.17.0）
  if not Mt.lx then
    local pos, uv, idx = Md.pos, Md.uv, Md.idx
    local lx, ly, lz, uu, vv = {}, {}, {}, {}, {}
    local has_tex = Mt.tex ~= nil
    local m = 0
    for t = Mt.first, Mt.first + Mt.count - 3, 3 do
      for o = 0, 2 do
        local j = idx[t + o]
        m = m + 1
        lx[m], ly[m], lz[m] = pos[j * 3], pos[j * 3 + 1], pos[j * 3 + 2]
        uu[m], vv[m] = has_tex and uv[j * 2] or 0.5, has_tex and uv[j * 2 + 1] or 0.5
      end
    end
    Mt.lx, Mt.ly, Mt.lz, Mt.uu, Mt.vv, Mt.nvx = lx, ly, lz, uu, vv, m
  end
  local LX, LY, LZ, UU, VV, NV = Mt.lx, Mt.ly, Mt.lz, Mt.uu, Mt.vv, Mt.nvx
  local ntri = NV / 3
  local base = (S.mscale or 100) / 100 * 10
  local cy0 = S.my or 0
  local eye = opt.eye or { 0, 0, -1024 }
  local ex, ey, ez = eye[1], eye[2], eye[3]
  -- 頂点の表は使い回す（drawpoly に渡した後は使わない）
  local pool = Md.pool or {}
  Md.pool = pool
  local np = 0
  local out, no = {}, 0
  local keys, tv = {}, {}
  for _, ii in ipairs(run.items) do
    local zm = I.zoom[ii] * base
    local rz = I.rz[ii]
    if opt.facing == 1 then
      local vx, vy = I.vx[ii], I.vy[ii]
      if vx ~= 0 or vy ~= 0 then rz = rz + atan2(vx, -vy) / RAD end
    end
    local axx, axy, axz, ayx, ayy, ayz = basis(I, ii, rz, opt)
    local azx, azy, azz = axy * ayz - axz * ayy, axz * ayx - axx * ayz, axx * ayy - axy * ayx
    local cx, cy, cz = I.x[ii], I.y[ii], I.z[ii]
    local sx, sy = zm * (I.asx[ii] or 1), zm * (I.asy[ii] or 1)
    local np0 = np
    for m = 1, NV do
      local lx, ly, lz = LX[m] * sx, -(LY[m] - cy0) * sy, LZ[m] * zm
      np = np + 1
      local v = pool[np]
      if not v then v = { 0, 0, 0, 0, 0 }; pool[np] = v end
      v[1], v[2], v[3] = cx + lx * axx + ly * ayx + lz * azx, cy + lx * axy + ly * ayy + lz * azy, cz + lx * axz + ly * ayz + lz * azz
      v[4], v[5] = UU[m], VV[m]
    end
    -- 奥から順に並べる: 重心までの距離の 2 乗（整数に丸める）× 65536 ＋ 三角形の番号 を、既定の比べ方で並べて逆から読む
    for t = 0, ntri - 1 do
      local a, b, c = pool[np0 + t * 3 + 1], pool[np0 + t * 3 + 2], pool[np0 + t * 3 + 3]
      local gx = (a[1] + b[1] + c[1]) / 3 - ex
      local gy = (a[2] + b[2] + c[2]) / 3 - ey
      local gz = (a[3] + b[3] + c[3]) / 3 - ez
      keys[t + 1] = floor(min(gx * gx + gy * gy + gz * gz, 1e10)) * 65536 + t
    end
    for t = ntri + 1, #keys do keys[t] = nil end
    if ntri < 65536 then
      table.sort(keys)
      for q = ntri, 1, -1 do
        local t = keys[q] % 65536
        local b0 = np0 + t * 3
        out[no + 1], out[no + 2], out[no + 3] = pool[b0 + 1], pool[b0 + 2], pool[b0 + 3]
        no = no + 3
      end
    else
      for q = 1, NV do out[no + q] = pool[np0 + q] end
      no = no + NV
    end
  end
  return out
end

function M.solid_verts(R, run, w, h, opt)
  local S = opt.solid
  if not S.tpl then S.tpl = solid_template(S) end
  local T = S.tpl
  local I = R.I
  local out = {}
  local nt = #T / 3
  local sorted = S.type >= 4
  local eye = opt.eye or { 0, 0, -1024 }
  for _, ii in ipairs(run.items) do
    local zm = I.zoom[ii]
    local rz = I.rz[ii]
    if opt.facing == 1 then
      local vx, vy = I.vx[ii], I.vy[ii]
      if vx ~= 0 or vy ~= 0 then rz = rz + atan2(vx, -vy) / RAD end
    end
    local axx, axy, axz, ayx, ayy, ayz = basis(I, ii, rz, opt)
    local azx, azy, azz = axy * ayz - axz * ayy, axz * ayx - axx * ayz, axx * ayy - axy * ayx
    local cx, cy, cz = I.x[ii], I.y[ii], I.z[ii]
    local W, H, u0, v0, du, dv = w, h, 0, 0, 1, 1
    if I.shape[ii] >= 0 then
      local cu, cv, cw, ch = cut_rect(I, ii, w, h, opt)
      W, H = opt.cut_size, opt.cut_size
      u0, v0, du, dv = cu - cw / 2 / w, cv - ch / 2 / h, cw / w, ch / h
    end
    local D = S.type == 5 and (S.depth or 20) or (W + H) / 2
    if (S.size or 0) > 0 and S.type <= 3 then W, H, D = S.size, S.size, S.size end
    W, H, D = W * zm * (I.asx[ii] or 1), H * zm * (I.asy[ii] or 1), D * zm
    local tris = sorted and {} or nil
    for t = 0, nt - 1 do
      local a = {}
      for v = 1, 3 do
        local q = T[t * 3 + v]
        local lx, ly, lz = q[1] * W, q[2] * H, q[3] * W + q[4] * H + q[5] * D
        a[v] = { cx + lx * axx + ly * ayx + lz * azx, cy + lx * axy + ly * ayy + lz * azy, cz + lx * axz + ly * ayz + lz * azz,
                 u0 + q[6] * du, v0 + q[7] * dv }
      end
      if sorted then
        local gx = (a[1][1] + a[2][1] + a[3][1]) / 3 - eye[1]
        local gy = (a[1][2] + a[2][2] + a[3][2]) / 3 - eye[2]
        local gz = (a[1][3] + a[2][3] + a[3][3]) / 3 - eye[3]
        a.d = gx * gx + gy * gy + gz * gz
        tris[#tris + 1] = a
      else
        out[#out + 1] = a[1]
        out[#out + 1] = a[2]
        out[#out + 1] = a[3]
      end
    end
    if sorted then
      table.sort(tris, function(p, q) return p.d > q.d end)
      for _, a in ipairs(tris) do
        out[#out + 1] = a[1]
        out[#out + 1] = a[2]
        out[#out + 1] = a[3]
      end
    end
  end
  return out
end

-- まとまり run の頂点。w,h は今の素材の大きさ（px）。
-- 四角形（run.vn == 4）: 1 枚 1 表の {x0,y0,z0,…,x3,y3,z3,u0,v0,…,u3,v3}（u,v は px）。obj.drawpoly(表, 透明度) で描く
-- 三角形（run.vn == 3）: 頂点の並び {x,y,z,u,v}（u,v は 0..1）。obj.drawpoly(表, 3, 透明度) で描く
-- 立体物（run.vn == 5）: 三角形と同じ形（M.solid_verts）
function M.run_verts(R, run, w, h, opt)
  if run.vn == 5 then return M.solid_verts(R, run, w, h, opt) end
  local I = R.I
  local out = {}
  local hw, hh = w / 2, h / 2
  for _, ii in ipairs(run.items) do
    local zm = I.zoom[ii]
    local rz = I.rz[ii]
    local len = I.len[ii]
    if len > 0 then
      -- 速度で伸ばす: 進む向きだけに合わせる（粒子の回転は使わない）
      local vx, vy = I.vx[ii], I.vy[ii]
      if vx ~= 0 or vy ~= 0 then rz = atan2(vx, -vy) / RAD end
    elseif opt.facing == 1 then
      local vx, vy = I.vx[ii], I.vy[ii]
      if vx ~= 0 or vy ~= 0 then rz = rz + atan2(vx, -vy) / RAD end
    end
    local axx, axy, axz, ayx, ayy, ayz = basis(I, ii, rz, opt)
    local cx, cy, cz = I.x[ii], I.y[ii], I.z[ii]
    local shape = I.shape[ii]
    local asx, asy, shr = I.asx[ii] or 1, I.asy[ii] or 1, I.shr[ii] or 0
    local isk = I.sk[ii] or opt.sk
    if shape < 0 then
      -- 四角形（素材の大きさ × 拡大率 × 縦横比。速度で伸ばすときは後ろ（来た向き）へ長くする）。
      -- 点 (lx, ly) は 中心 + (lx + ゆがみ × ly) × 横 + ly × 縦
      local qw, qh = hw * zm * asx, hh * zm * asy
      local qb = qh + len
      local s0, s1 = -qw - shr * qh, qw - shr * qh
      local s2, s3 = qw + shr * qb, -qw + shr * qb
      local x0, y0, z0 = cx + s0 * axx - qh * ayx, cy + s0 * axy - qh * ayy, cz + s0 * axz - qh * ayz
      local x1, y1, z1 = cx + s1 * axx - qh * ayx, cy + s1 * axy - qh * ayy, cz + s1 * axz - qh * ayz
      local x2, y2, z2 = cx + s2 * axx + qb * ayx, cy + s2 * axy + qb * ayy, cz + s2 * axz + qb * ayz
      local x3, y3, z3 = cx + s3 * axx + qb * ayx, cy + s3 * axy + qb * ayy, cz + s3 * axz + qb * ayz
      local keep, mirror = true, false
      if opt.face and opt.face > 0 and opt.facing ~= 2 then
        local front = ((x1 - x0) * (y3 - y0) - (y1 - y0) * (x3 - x0)) > 0
        if opt.face == 1 then keep = front else keep = not front; mirror = opt.face == 2 end
      end
      if keep then
        local u0, u1 = 0, w
        if mirror then u0, u1 = w, 0 end
        out[#out + 1] = { x0, y0, z0, x1, y1, z1, x2, y2, z2, x3, y3, z3, u0, 0, u1, 0, u1, h, u0, h }
      end
    else
      -- 切り抜き: 元の画像の一部を多角形で貼る（中心からの扇）
      local csz = opt.cut_size
      local poly = SHAPES[shape] or SHAPES[0]
      if I.poly[ii] then
        -- 破片: 重心からの頂点（px）。切り抜く範囲 cw = ch = 1 で、頂点の px がそのまま画像の px になる
        poly = I.poly[ii]
      elseif shape == 7 then
        poly = {}
        for c = 0, 3 do
          local a = (c / 4) * 2 * pi - 3 * pi / 4
          local rr = 0.35 + 0.35 * rnd(I.seed[ii] * 4 + c, CH.mat + 4, isk)
          poly[#poly + 1] = { rr * cos(a), rr * sin(a) }
        end
      end
      -- 切り抜く場所（素材の中の中心。0..1）。素材より大きく切り抜くときは、素材の大きさまでを貼る
      local cw, ch, cu, cv, szx, szy
      if I.cu[ii] then
        -- 切り抜く所 = 出た所: 粒子のマス（画像の端のマスは端まで）を、その大きさで貼る
        cw, ch, cu, cv = I.cw[ii], I.ch[ii], I.cu[ii], I.cv[ii]
        szx, szy = cw * zm, ch * zm
      else
        cw, ch = min(csz, w), min(csz, h)
        cu, cv = 0.5, 0.5
        if not opt.cut_center then
          local mu, mv = cw / 2 / w, ch / 2 / h
          cu = mu + (1 - 2 * mu) * rnd(I.seed[ii], CH.mat + 5, isk)
          cv = mv + (1 - 2 * mv) * rnd(I.seed[ii], CH.mat + 6, isk)
        end
        szx, szy = csz * zm, csz * zm
      end
      local nv = #poly
      for t = 1, nv do
        local a1, a2 = poly[t], poly[t % nv + 1]
        local l1y, l2y = a1[2] * szy * asy, a2[2] * szy * asy
        local l1x, l2x = a1[1] * szx * asx + shr * l1y, a2[1] * szx * asx + shr * l2y
        out[#out + 1] = { cx, cy, cz, cu, cv }
        out[#out + 1] = { cx + l1x * axx + l1y * ayx, cy + l1x * axy + l1y * ayy, cz + l1x * axz + l1y * ayz,
                          cu + a1[1] * cw / w, cv + a1[2] * ch / h }
        out[#out + 1] = { cx + l2x * axx + l2y * ayx, cy + l2x * axy + l2y * ayy, cz + l2x * axz + l2y * ayz,
                          cu + a2[1] * cw / w, cv + a2[2] * ch / h }
      end
    end
  end
  return out
end

----------------------------------------------------------------------------- 画面に貼り付ける・粒子を外へ渡す（v0.10.0）

M.SCREEN_D0 = 1024   -- カメラ制御の外（既定のカメラ）の、目からスクリーンまでの距離

--[[
画面に貼り付ける: 既定のカメラ（目 (0,0,-1024)・前 +Z・下 +Y）で見た絵を、今のカメラでも同じに写す変換。
点 P（本体からの座標）に本体の位置 B を足した (X, Y, Z) を、W = 目 + 前 × d × (1 + Z / 1024) + 右 × X + 下 × Y へ移す。
今のカメラで見ると奥行きは d × (1 + Z / 1024)、画面の位置は (X, Y) × 1024 / (1024 + Z) で、既定のカメラと同じになる。
右・下は前と上向き（ux, uy, uz）から作り、傾き rz（度）で前の軸のまわりに回す
（@Camera.cam2 の「座標」と同じ向きの前提。実機で確かめる）。
戻り値 S: 本体からの座標 (x, y, z) を O + 右 × x + 下 × y + 前 × (d / 1024) × z へ移す係数。前が決まらなければ nil
]]
function M.screen_basis(c, bx, by, bz)
  local fx, fy, fz = c.tx - c.x, c.ty - c.y, c.tz - c.z
  local fl = sqrt(fx * fx + fy * fy + fz * fz)
  if fl < 1e-9 then return nil end
  fx, fy, fz = fx / fl, fy / fl, fz / fl
  local ux, uy, uz = c.ux or 0, c.uy or -1, c.uz or 0
  local ax, ay, az = fy * uz - fz * uy, fz * ux - fx * uz, fx * uy - fy * ux     -- 右 = 前 × 上
  local al = sqrt(ax * ax + ay * ay + az * az)
  if al < 1e-9 then return nil end
  ax, ay, az = ax / al, ay / al, az / al
  local dx, dy, dz = fy * az - fz * ay, fz * ax - fx * az, fx * ay - fy * ax     -- 下 = 前 × 右
  local t = (c.rz or 0) * RAD
  if t ~= 0 then
    local ct, st = cos(t), sin(t)
    ax, ay, az, dx, dy, dz = ax * ct - dx * st, ay * ct - dy * st, az * ct - dz * st,
                             ax * st + dx * ct, ay * st + dy * ct, az * st + dz * ct
  end
  local d = (c.d and c.d > 0) and c.d or M.SCREEN_D0
  local k = d / M.SCREEN_D0
  return {
    ox = c.x + fx * d + ax * bx + dx * by + fx * k * bz - bx,
    oy = c.y + fy * d + ay * bx + dy * by + fy * k * bz - by,
    oz = c.z + fz * d + az * bx + dz * by + fz * k * bz - bz,
    ax = ax, ay = ay, az = az, dx = dx, dy = dy, dz = dz, fx = fx * k, fy = fy * k, fz = fz * k,
  }
end

local function screen_pt(S, x, y, z)
  return S.ox + S.ax * x + S.dx * y + S.fx * z, S.oy + S.ay * x + S.dy * y + S.fy * z, S.oz + S.az * x + S.dz * y + S.fz * z
end
M.screen_pt = screen_pt

-- 頂点の並びに screen_basis の変換を掛ける（その場で書き換える）。
-- quad = true: 1 表に 4 頂点（{x0,y0,z0,…,x3,y3,z3,…}）。false: 1 表に 1 頂点（{x,y,z,…}）
function M.screen_apply(S, list, quad)
  if quad then
    for _, q in ipairs(list) do
      q[1], q[2], q[3] = screen_pt(S, q[1], q[2], q[3])
      q[4], q[5], q[6] = screen_pt(S, q[4], q[5], q[6])
      q[7], q[8], q[9] = screen_pt(S, q[7], q[8], q[9])
      q[10], q[11], q[12] = screen_pt(S, q[10], q[11], q[12])
    end
  else
    for _, v in ipairs(list) do v[1], v[2], v[3] = screen_pt(S, v[1], v[2], v[3]) end
  end
end

--[[
粒子を外へ渡す: _G.Particle_H_share[鍵] に、今のフレームの粒子を生まれた順に置く。鍵 = シーン:名前。
座標は本体の位置 B を足した値（受け取る側が自分の位置を引く）。frame はシーン基準のフレーム（obj.originframe）。
戻り値: 同じフレームに別のオブジェクト（id）が同じ鍵で書いていたら true（後から書いた方が勝つ）
]]
function M.share_put(key, res, bx, by, bz, frame, id, hist_n)
  local T = _G.Particle_H_share or {}
  _G.Particle_H_share = T
  local old = T[key]
  local dup = old ~= nil and old.frame == frame and old.id ~= id
  local S = { frame = frame, id = id, n = 0, x = {}, y = {}, z = {}, rx = {}, ry = {}, rz = {}, zoom = {}, alpha = {},
              k = {}, age = {}, life = {}, vx = {}, vy = {} }
  local idx = {}
  for i = 0, res.n - 1 do idx[i + 1] = i end
  M.sort_by_k(idx, res)
  for t, i in ipairs(idx) do
    S.x[t], S.y[t], S.z[t] = res.x[i] + bx, res.y[i] + by, res.z[i] + bz
    S.rx[t], S.ry[t], S.rz[t] = res.rx[i], res.ry[i], res.rz[i]
    S.zoom[t], S.alpha[t], S.k[t], S.age[t], S.life[t] = res.zoom[i], res.alpha[i], res.k[i], res.age[i], res.life[i]
    S.vx[t], S.vy[t] = res.vx[i] or 0, res.vy[i] or 0
  end
  S.n = #idx
  T[key] = S
  -- 出力位置「渡された粒子」のための履歴（フレームごと。位置と速さだけ、多ければ間引いて SHARE_HIST_N 個まで）。
  -- 今のフレームから SHARE_HIST_F フレームより離れたものは捨てる
  local HT = _G.Particle_H_share_hist or {}
  _G.Particle_H_share_hist = HT
  local H = HT[key] or {}
  HT[key] = H
  -- 履歴は float の配列（添字は 1 から。v0.16.0）。粒子の番号 k も持つ（当たる相手の位置を 2 つのフレームから補うため）
  local cap = max(floor(hist_n or M.SHARE_HIST_N), 1)
  local step = S.n > cap and S.n / cap or 1
  local m_ = min(S.n, cap)
  local A = { n = 0, x = ffi.new("float[?]", m_ + 1), y = ffi.new("float[?]", m_ + 1), z = ffi.new("float[?]", m_ + 1),
              vx = ffi.new("float[?]", m_ + 1), vy = ffi.new("float[?]", m_ + 1), k = ffi.new("int32_t[?]", m_ + 1) }
  local t = 1
  while t <= S.n and A.n < cap do
    local u = floor(t)
    local m = A.n + 1
    A.x[m], A.y[m], A.z[m], A.vx[m], A.vy[m], A.k[m] = S.x[u], S.y[u], S.z[u], S.vx[u], S.vy[u], S.k[u]
    A.n = m
    t = t + step
  end
  H[frame] = A
  for f in pairs(H) do
    if abs(f - frame) > M.SHARE_HIST_F then H[f] = nil end
  end
  return dup
end

M.SHARE_HIST_N = 500   -- 履歴に残す粒子の数（1 フレームあたり）
M.SHARE_HIST_F = 300   -- 履歴を残すフレームの幅（今のフレームから前後に）

--[[
渡された粒子の履歴: frame（シーン基準）の粒子（{ n, x, y, z, vx, vy }）。座標は渡した側の本体の位置を足した値。
そのフレームが無ければ、前後 3 フレームまでの近いもの（前を先に）、それも無ければ一番近いもの。履歴が無ければ nil
]]
function M.share_hist(key, frame)
  local HT = _G.Particle_H_share_hist
  local H = HT and HT[key]
  if not H then return nil end
  if H[frame] then return H[frame] end
  for d = 1, 3 do
    if H[frame - d] then return H[frame - d] end
    if H[frame + d] then return H[frame + d] end
  end
  local best, bd = nil, huge
  for f, A in pairs(H) do
    local d = abs(f - frame)
    if d < bd or (d == bd and f < frame) then best, bd = A, d end
  end
  return best
end

--[[
渡された粒子の、フレーム ft（シーン基準。端数あり）の位置（v0.16.0）: ft をはさむ 2 つのフレームから、同じ番号の粒子の位置を直線で補う。
片方のフレームにしか無い粒子は、そのフレームの位置。どちらかのフレームが無いか、番号の無い古い形の履歴なら、近いフレーム（share_hist）
]]
function M.share_at(key, ft)
  local HT = _G.Particle_H_share_hist
  local H = HT and HT[key]
  if not H then return nil end
  local fa = floor(ft)
  local u = ft - fa
  local A, B = H[fa], H[fa + 1]
  if A and u < 1e-9 then return A end
  if not (A and B and A.k and B.k) then return M.share_hist(key, floor(ft + 0.5)) end
  local map = {}
  for j = 1, B.n do map[B.k[j]] = j end
  local L = { n = 0, x = {}, y = {}, z = {}, vx = {}, vy = {}, k = {} }
  local used = {}
  local m = 0
  for i = 1, A.n do
    local j = map[A.k[i]]
    m = m + 1
    if j then
      used[j] = true
      L.x[m], L.y[m], L.z[m] = A.x[i] + (B.x[j] - A.x[i]) * u, A.y[i] + (B.y[j] - A.y[i]) * u, A.z[i] + (B.z[j] - A.z[i]) * u
      L.vx[m], L.vy[m] = A.vx[i] + (B.vx[j] - A.vx[i]) * u, A.vy[i] + (B.vy[j] - A.vy[i]) * u
    else
      L.x[m], L.y[m], L.z[m], L.vx[m], L.vy[m] = A.x[i], A.y[i], A.z[i], A.vx[i], A.vy[i]
    end
    L.k[m] = A.k[i]
  end
  for j = 1, B.n do
    if not used[j] then
      m = m + 1
      L.x[m], L.y[m], L.z[m], L.vx[m], L.vy[m], L.k[m] = B.x[j], B.y[j], B.z[j], B.vx[j], B.vy[j], B.k[j]
    end
  end
  L.n = m
  return L
end

-- 受け取る: 鍵の表と、受け取る側のフレーム − 渡した側のフレーム。表が無ければ nil
function M.share_get(key, frame)
  local T = _G.Particle_H_share
  local S = T and T[key]
  if not S then return nil end
  return S, frame - S.frame
end

--[[
受け取って描く: 渡された粒子の位置に、今の画像（w × h px）の四角を置く。
bx, by, bz = 受け取る側の位置（本体からの座標に直す）。zoomf = 拡大率の倍率。use_alpha = 粒子の透過率を使う。use_rot = 粒子の回転を使う。
戻り値: { [段] = 四角の並び }。段は 1..64 で、不透明度 = 段 / 64（drawpoly の alpha に渡す）
]]
function M.receive_quads(S, w, h, bx, by, bz, zoomf, use_alpha, use_rot, order)
  local groups = {}
  local hw, hh = w / 2, h / 2
  local opt = { order = order or M.STANDARD_ORDER }
  for t = 1, S.n do
    local a = use_alpha and S.alpha[t] or 1
    local lv = floor(a * 64 + 0.5)
    local zm = S.zoom[t] * zoomf
    if lv > 0 and zm > 0 then
      local axx, axy, axz, ayx, ayy, ayz = 1, 0, 0, 0, 1, 0
      if use_rot then axx, axy, axz, ayx, ayy, ayz = basis(S, t, S.rz[t], opt) end
      local cx, cy, cz = S.x[t] - bx, S.y[t] - by, S.z[t] - bz
      local qw, qh = hw * zm, hh * zm
      local list = groups[lv]
      if not list then list = {}; groups[lv] = list end
      list[#list + 1] = {
        cx - qw * axx - qh * ayx, cy - qw * axy - qh * ayy, cz - qw * axz - qh * ayz,
        cx + qw * axx - qh * ayx, cy + qw * axy - qh * ayy, cz + qw * axz - qh * ayz,
        cx + qw * axx + qh * ayx, cy + qw * axy + qh * ayy, cz + qw * axz + qh * ayz,
        cx - qw * axx + qh * ayx, cy - qw * axy + qh * ayy, cz - qw * axz + qh * ayz,
        0, 0, w, 0, w, h, 0, h,
      }
    end
  end
  return groups
end

return M
