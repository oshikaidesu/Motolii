--[[
EffectUtils_Budget.lua by HexBrowns
cache: と tempbuffer の枠（system.conf の TemporaryImageCacheSize、既定 16,000,000 ドット）に収まるかを、
確保する前に見積もるための共有モジュール。require("EffectUtils_Budget") で読む（Script/ 直下に置く）。

枠を超えると、最も長く使っていないバッファが黙って捨てられ、後で読むと invalid buffer type になる。
1 枚で枠を超える確保は CreateRenderTargetView() failed の例外になる（ルール au2-filter-object）。
ほかのオブジェクトの cache も同じ枠を使うので、目安は 1600 万ではなく 1400 万にする。

使い方
  local budget = require("EffectUtils_Budget")
  -- 縮めない（作業の解像度が決まっている）なら、合計を見て収まらなければ素通し
  if not budget.check("効果名", w, h, w * h + 2 * lw * lh) then return end
  -- 作業の解像度を下げて収める（倍率 s。面積は s^2 倍）
  local s = budget.scale(固定のドット数, s = 1 のときの縮むドット数)
  if not s then budget.refuse("効果名", w, h); return end

仕様と経緯: AI/scripts/issues/20261013_cache_budget_4k_wide_materials.md
]]

local M = {}

M.LIMIT = 16000000   -- 本体の既定の枠（本番は既定のまま）
M.BUDGET = 14000000  -- 自作の効果が 1 回の実行で使ってよい合計の目安

-- dots が枠の目安に収まるか
function M.fits(dots, budget)
  return dots <= (budget or M.BUDGET)
end

-- 作業の解像度の倍率 s（0 < s <= 1）。fixed は縮まないドット数、scaled は s = 1 のときの縮むドット数（s^2 倍になる）。
-- 縦横の切り上げで少し増える分は、3% 小さめに見積もった s で吸う。収まらないなら nil
function M.scale(fixed, scaled, budget, min_s)
  budget = budget or M.BUDGET
  local room = budget - (fixed or 0)
  if room <= 0 then return nil end
  if scaled <= room then return 1 end
  local s = math.floor(math.sqrt(room * 0.97 / scaled) * 1024) / 1024
  if s < (min_s or 1 / 64) then return nil end
  return s
end

-- 同じ理由の警告を 1 回だけ出す（global に印を残す。毎フレーム出さない）
function M.warn_once(tag, key, message)
  local k = "EffectUtils_Budget:" .. tag .. ":" .. key
  if global == nil then return end
  if global[k] == nil then
    global[k] = "1"
    print("@warn", "[" .. tag .. "] " .. message)
  end
end

-- 収まらないので効果を掛けないことを、画像の大きさごとに 1 回だけ知らせる
function M.refuse(tag, w, h)
  w, h = math.floor(w), math.floor(h)
  M.warn_once(tag, w .. "x" .. h,
    "画像（" .. w .. "x" .. h .. "）が大きく、作業用のバッファが枠（cache と tempbuffer の合計 "
    .. math.floor(M.LIMIT / 10000) .. " 万ドット）に収まらないので、効果を掛けません")
end

-- 合計 dots が収まれば true。収まらなければ警告を 1 回出して false（呼び出し側は素通しで return する）
function M.check(tag, w, h, dots, budget)
  if M.fits(dots, budget) then return true end
  M.refuse(tag, w, h)
  return false
end

return M
