-- browser.lua  AviUtl2Browser_H（aviutl_browser の AviUtl2 移植）の互換モジュール
--
-- 原作 oov/aviutl_browser の browser.lua と同じ呼び方で使える。
-- 原作は bridge.dll（require("bridge").call）で aviutl_browser.exe を呼んでいたが、
-- ここでは AviUtl2Browser_H.mod2 を呼ぶ。exe との通信と画素の受け渡しは mod2 が持つ。
--
--   obj.setoption("drawtarget", "tempbuffer", w, h)
--   obj.load("tempbuffer")
--   local ok, ret = require("browser").execute({
--     abc = "mycontent.abc", -- contents フォルダー内の .abc（dir とどちらか）
--     dir = "mycontent",     -- contents フォルダー内のフォルダー
--     tabid = "",            -- 同じコンテンツを別タブで開くときの識別文字列（省略可）
--     param = "",            -- JavaScript に渡す文字列（省略可）
--     userfile = file,       -- ファイル参照で選んだファイル（省略可。HTML からは /userfile）
--     dev = false,           -- DevTools を出すなら true
--   })
--
-- 画像は呼び出し時点のオブジェクトの大きさ（obj.w × obj.h）で描かれ、オブジェクトの画像を置き換える。
-- 失敗したとき（exe が無い・JS が完了を返さない等）は画像を書き換えず、false と理由を返す。

local P = {}

-- 文字列は 16 進にして渡す。旧形式（.obj / .anm）の SJIS や NUL を、本体の文字列変換に通さずに mod2 へ届けるため
local HEX = {}
for i = 0, 255 do
  HEX[string.char(i)] = string.format("%02x", i)
end

local function hex(s)
  if s == nil then
    return ""
  end
  return (tostring(s):gsub(".", HEX))
end

local MODULE_NAMES = { "AviUtl2Browser_H", "AviUtl2Browser_H@browser" }
local mod

local function load_module(o)
  if mod then
    return mod
  end
  for _, name in ipairs(MODULE_NAMES) do
    local ok, m = pcall(o.module, name)
    if ok and m then
      mod = m
      return mod
    end
  end
  return nil
end

-- 同じ理由のエラーは 1 度だけログに出す（毎フレーム呼ばれるため）
local reported = {}

local function warn_once(msg)
  if not reported[msg] then
    reported[msg] = true
    print("@warn", "[AviUtl2Browser_H] " .. msg)
  end
end

-- dev は原作では真偽値。--check@ の 0 / 1 で渡されても 0 を偽として読む
local function truthy(v)
  if type(v) == "number" then
    return v ~= 0
  end
  return v and true or false
end

--- ブラウザーでコンテンツを表示し、内容をオブジェクトの画像にする
-- @return bool, string 成功したかどうかと、JavaScript から渡された文字列
function P.execute(params)
  -- 呼び出し元のスクリプトの obj を使う（require したモジュールの環境に obj が無い場合に備える）
  local caller = getfenv and getfenv(2) or nil
  local o = (caller and caller.obj) or obj
  params = params or {}

  local path, use_dir
  if params.abc ~= nil then
    path, use_dir = params.abc, false
  elseif params.dir ~= nil then
    path, use_dir = params.dir, true
  else
    return false, "abc or dir is required"
  end

  local m = load_module(o)
  if not m then
    local msg = "AviUtl2Browser_H.mod2 is not found (Script/browser/)"
    warn_once(msg)
    return false, msg
  end

  local w, h = math.floor(o.w), math.floor(o.h)
  if w < 1 or h < 1 then
    return false, "image is empty (call obj.load(\"tempbuffer\") first)"
  end

  local data, ok, ret = m.execute(
    hex(path), use_dir, truthy(params.dev),
    hex(params.userfile), hex(params.tabid), hex(params.param), w, h)
  if data then
    o.putpixeldata("object", data, w, h, "bgra")
  end
  if not ok then
    warn_once(tostring(ret))
  end
  return ok, ret
end

return P
