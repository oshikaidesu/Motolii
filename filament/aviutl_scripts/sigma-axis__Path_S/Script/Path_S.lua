--[[
MIT License

Copyright (c) 2025-2026 sigma-axis

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

https://mit-license.org/
]]

--
-- v2.42
--

-- https://github.com/sigma-axis/aviutl2_script_Path_S

local obj, print, math, tonumber, type, ipairs, unpack, bit, ffi, buffer = obj, print, math, tonumber, type, ipairs, unpack, bit, require("ffi"), require("string.buffer");

if obj.getinfo("version") < tonumber("2011000") then
	error([[AviUtl ExEdit 2.1.10 以降が必要です！]], 2);
end

---PI で `boolean|number|nil` での指定を適用する．number の場合，0 を false 扱い，0 以外を true 扱いとする．
---@param pi_value any PI に渡ってきた値．
---@param gui_value boolean 実際にスクリプトのパラメタとして渡ってきた値．
---@return boolean
local function PI_as_bool(pi_value, gui_value)
	if type(pi_value) == "boolean" then return pi_value;
	elseif type(pi_value) == "number" then return pi_value ~= 0;
	else return gui_value end
end

local PI_choose_blend_mode do
	local name2num, num2code = {
		["通常"] = 0, ["加算"] = 1, ["減算"] = 2, ["乗算"] = 3, ["スクリーン"] = 4, ["オーバーレイ"] = 5,
		["比較(明)"] = 6, ["比較(暗)"] = 7, ["輝度"] = 8, ["色差"] = 9,
		["陰影"] = 10, ["明暗"] = 11, ["差分"] = 12,
		["alpha_add"] = 100, ["alpha_max"] = 101, ["alpha_sub"] = 102, ["alpha_add2"] = 103, ["rgba_add"] = 104,
	}, {
		[0] = "none", "add", "sub", "mul", "screen", "overlay",
		"light", "dark", "brightness", "chroma", "shadow", "light_dark", "diff",
		[100] = "alpha_add", [101] = "alpha_max", [102] = "alpha_sub", [103] = "alpha_add2", [104] = "rgba_add",
	};

	---PI で「合成モード」指定を適用して，`obj.setoption("blend", ...)` に渡せる文字列に変換する．
	---このパラメタは次の形式で指定されているものとする:
	---
	---`--select@blend:合成モード=0,通常=0,加算=1,減算=2,乗算=3,スクリーン=4,オーバーレイ=5,比較(明)=6,比較(暗)=7,輝度=8,色差=9,陰影=10,明暗=11,差分=12`
	---
	---この他にも `"alpha_add"` などの仮想バッファ専用合成モードも `pi_value` として有効．
	---@param pi_value any PI に渡ってきた値．
	---@param gui_value integer 実際にスクリプトのパラメタとして渡ってきた値．
	---@return string # 合成モードの名前．
	function PI_choose_blend_mode(pi_value, gui_value)
		if type(pi_value) == "string" then
			gui_value = name2num[pi_value] or gui_value;
		end
		return num2code[gui_value] or "none";
	end
end

---@alias path_type # パスの種類．
---| 0 # 折れ線
---| 1 # 補間移動
---| 2 # 2次 Bezier
---| 3 # 3次 Bezier
local PI_choose_path_type do
	local name2num = {
		["折れ線"] = 0, ["補間移動"] = 1, ["2次ベジェ曲線"] = 2, ["3次ベジェ曲線"] = 3,
	};
	---PI で「パスの種類」指定を適用する．
	---このパラメタは次の形式で指定されているものとする:
	---
	---`--select@path_type:線タイプ=3,折れ線=0,補間移動=1,2次ベジェ曲線=2,3次ベジェ曲線=3`
	---@param pi_value any
	---@param gui_value path_type
	---@return path_type
	function PI_choose_path_type(pi_value, gui_value)
		if type(pi_value) == "string" then
			gui_value = name2num[pi_value] or gui_value;
		end
		return math.min(math.max(math.floor(0.5 + gui_value), 0), 3);
	end
end

---@alias mode_anchor_base # アンカー基準．
---| 0 # 回転中心
---| 1 # 左上
---| 2 # 上
---| 3 # 右上
---| 4 # 左
---| 5 # 中央
---| 6 # 右
---| 7 # 左下
---| 8 # 下
---| 9 # 右下
local PI_choose_mode_anchor_base do
	local name2num = {
		["回転中心"] = 0, ["左上"] = 1, ["上"] = 2, ["右上"] = 3, ["左"] = 4, ["中央"] = 5, ["右"] = 6, ["左下"] = 7, ["下"] = 8, ["右下"] = 9,
	};
	---PI で「アンカー基準」指定を適用する．
	---このパラメタは次の形式で指定されているものとする:
	---
	--`--select@mode_anchor_base:アンカー基準=5,回転中心=0,左上=1,上=2,右上=3,左=4,中央=5,右=6,左下=7,下=8,右下=9`
	---@param pi_value any
	---@param gui_value mode_anchor_base
	---@return mode_anchor_base
	function PI_choose_mode_anchor_base(pi_value, gui_value)
		if type(pi_value) == "string" then
			gui_value = name2num[pi_value] or gui_value;
		end
		return math.min(math.max(math.floor(0.5 + gui_value), 0), 9);
	end
end

---@alias mode_fill # 塗りつぶし範囲の指定．
---| 0 # 通常
---| 1 # 奇偶
---| 2 # 反転
---| 3 # 奇偶反転
local PI_choose_mode_fill do
	local name2num = {
		["内側"] = 0, ["奇偶"] = 1, ["内側反転"] = 2, ["奇偶反転"] = 3,
	};
	---PI で塗りつぶしの「範囲」指定を適用する．
	---このパラメタは次の形式で指定されているものとする:
	---
	---`--select@mode_fill:範囲=0,内側=0,奇偶=1,内側反転=2,奇偶反転=3`
	---@param pi_value any
	---@param gui_value mode_fill
	---@return mode_fill
	function PI_choose_mode_fill(pi_value, gui_value)
		if type(pi_value) == "string" then
			gui_value = name2num[pi_value] or gui_value;
		end
		return math.min(math.max(math.floor(0.5 + gui_value), 0), 3);
	end
end

---@alias end_shape # 端の形状．
---| 0 # 円
---| 1 # 四角
---| 2 # 平坦
---| 3 # 三角
---| 4 # リボン
local PI_choose_end_shape do
	local name2num = {
		["円"] = 0, ["四角"] = 1, ["平坦"] = 2, ["三角"] = 3, ["リボン"] = 4,
	};
	---PI で塗りつぶしの「範囲」指定を適用する．
	---このパラメタは次の形式で指定されているものとする:
	---
	---`--select@end_shape:端の形状=0,円=0,四角=1,平坦=2,三角=3,リボン=4`
	---@param pi_value any
	---@param gui_value end_shape
	---@return end_shape
	function PI_choose_end_shape(pi_value, gui_value)
		if type(pi_value) == "string" then
			gui_value = name2num[pi_value] or gui_value;
		end
		return math.min(math.max(math.floor(0.5 + gui_value), 0), 4);
	end
end

---@alias join_shape # 線結合の形状．
---| 0 # ラウンド
---| 1 # ベベル
---| 2 # マイター
---| 3 # ブランク
---| 4 # ブランク+ラウンド
---| 5 # ブランク+ベベル
local PI_choose_join_shape do
	local name2num = {
		["ラウンド"] = 0, ["ベベル"] = 1, ["マイター"] = 2, ["ブランク"] = 3, ["ブランク+ラウンド"] = 4, ["ブランク+ベベル"] = 5,
	};
	---PI で塗りつぶしの「範囲」指定を適用する．
	---このパラメタは次の形式で指定されているものとする:
	---
	---`--select@join_shape:線結合の形状=0,ラウンド=0,ベベル=1,マイター=2,ブランク=3,ブランク+ラウンド=4,ブランク+ベベル=5`
	---@param pi_value any
	---@param gui_value join_shape
	---@return join_shape
	function PI_choose_join_shape(pi_value, gui_value)
		if type(pi_value) == "string" then
			gui_value = name2num[pi_value] or gui_value;
		end
		return math.min(math.max(math.floor(0.5 + gui_value), 0), 5);
	end
end

local line_inflation do
	---端の形状による影響を考慮して，ラインの領域サイズの上下左右拡張幅を計算．
	---@param end_shape end_shape 端の形状．
	---@param line_width number ライン幅
	---@param antialias number ぼかし幅．
	---@return number # 上下左右方向の拡大幅．
	local function end_inflation(end_shape, line_width, antialias)
		if end_shape == 1 then -- 四角
			return 2 ^ 0.5 * (line_width / 2 + antialias);
		elseif end_shape == 4 then -- リボン
			return 2 ^ 0.5 * line_width / 2 + antialias;
		else -- 円 / 平坦 / 三角
			return line_width / 2 + antialias;
		end
	end
	---線結合の形状による影響を考慮して，ラインの領域サイズの上下左右拡張幅を計算．
	---@param join_shape join_shape 端の形状．
	---@param line_width number ライン幅
	---@param antialias number ぼかし幅．
	---@param miter_limit number マイター限界．
	---@return number # 上下左右方向の拡大幅．
	local function join_inflation(join_shape, line_width, antialias, miter_limit)
		if join_shape == 2 then -- マイター
			return miter_limit * (line_width / 2 + antialias);
		elseif join_shape == 1 or join_shape == 5 then -- ベベル
			return ((line_width / 2 + antialias) ^ 2 + antialias ^ 2) ^ 0.5;
		else -- ラウンド / ブランク
			return line_width / 2 + antialias;
		end
	end

	---端の形状や線結合の形状による影響を考慮して，ラインの領域サイズの上下左右拡張幅を計算．
	---@param end_shape end_shape 端の形状．
	---@param dash_end_shape end_shape 破線の端の形状．
	---@param join_shape join_shape 線結合の形状．
	---@param line_width number ライン幅．
	---@param antialias number ぼかし幅．
	---@param miter_limit number マイター限界．
	---@param dash_pat number[]? 破線パターン．`dash_end_shape` を考慮に入れるか決定する．省略時は常に考慮に入れる．
	---@return number # 上下左右方向の拡大幅．
	function line_inflation(end_shape, dash_end_shape, join_shape, line_width, antialias, miter_limit, dash_pat)
		local m = math.max(
			end_inflation(end_shape, line_width, antialias),
			join_inflation(join_shape, line_width, antialias, miter_limit));
		if dash_pat then
			for i = 2, #dash_pat, 2 do
				if dash_pat[i] > 0 then goto has_dash end
			end
			return m;
		end
		::has_dash::
		return math.max(m, end_inflation(dash_end_shape, line_width, antialias));
	end
end

---@alias dither_pattern # ディザリング
---| 0 # なし
---| 1 # Checker
---| 2 # Bayer 2x2
---| 3 # Bayer 4x4
---| 4 # Bayer 256x256
---| 5 # IGN
---| 6 # White Noise
---| 7 # Halftone
---| 8 # Halftone Honeycomb
---| 9 # Halftone Square
---| 10 # Stripe
local PI_choose_dither_pattern do
	local name2num = {
		["なし"] = 0,
		["Checker"] = 1,
		["Bayer 2x2"] = 2,
		["Bayer 4x4"] = 3,
		["Bayer 256x256"] = 4,
		["IGN"] = 5,
		["White Noise"] = 6,
		["Halftone"] = 7,
		["Halftone Honeycomb"] = 8,
		["Halftone Square"] = 9,
		["Stripe"] = 10,
	};
	---PI で「ディザリング」指定を適用する．
	---このパラメタは次の形式で指定されているものとする:
	---
	---`--select@dither_pattern:ディザリング=0,なし=0,Checker=1,Bayer 2x2=2,Bayer 4x4=3,Bayer 256x256=4,IGN=5,White Noise=6,Halftone=7,Halftone Honeycomb=8,Halftone Square=9,Stripe=10`
	---@param pi_value any
	---@param gui_value dither_pattern
	---@return dither_pattern
	function PI_choose_dither_pattern(pi_value, gui_value)
		if type(pi_value) == "string" then
			gui_value = name2num[pi_value] or gui_value;
		end
		return math.min(math.max(math.floor(0.5 + gui_value), 0), 10);
	end
end

---@class dither_setting ディザ設定．
---@field width number ディザリングの幅．ピクセル単位．0.0 以上．
---@field pattern dither_pattern ディザパターン．
---@field seed integer ノイズのシード．
---@field rate number ディザリングの影響の強さ．0.0 -- 1.0.
---@field cx number パターンの原点の X 座標．バッファ左上からのピクセル単位の相対座標．
---@field cy number パターンの原点の Y 座標．バッファ左上からのピクセル単位の相対座標．
---@field size number ディザリングのドットの拡大率．1.0 以上の実数.
---@field rot number ディザリングの回転角．時計回りのラジアン単位．

---@alias antialias # アンチエイリアス幅，またはディザ設定．
---| number # アンチエイリアス幅．ピクセル単位．0.0 以上．
---| dither_setting # ディザリングする場合の設定．

---アンチエイリアス設定の型をディザ設定の型に矯正する．
---@param antialias antialias
---@return dither_setting
local function wrap_antialias(antialias)
	if type(antialias) ~= "number" then
		if antialias.width > 0 and antialias.rate > 0 and antialias.pattern ~= 0 then return antialias end
		antialias = antialias.width;
	end
	return {
		width = antialias,
		pattern = 0, seed = 0, rate = 0,
		cx = 0, cy = 0, size = 1, rot = 0,
	};
end

---「アンカー基準」の設定項目から，実際のアンカーの基準となる座標を取得する．
---@param mode_anchor_base mode_anchor_base アンカー基準．
---@return number cx, number cy アンカーの原点の X, Y 座標．
local function anchor_offset(mode_anchor_base)
		local cx, cy = 0, 0;
		if mode_anchor_base == 0 then
			cx, cy = obj.getvalue("center");
			cx, cy = cx + obj.cx, cy + obj.cy;
		else
			cx = (((mode_anchor_base - 1) % 3) - 1) * obj.w / 2;
			cy = (math.floor((mode_anchor_base - 1) / 3) - 1) * obj.h / 2;
		end
	return cx, cy;
end

---@alias snap_angle # 角度単位のスナップ設定．
---| 0 # なし
---| 1 # 90°
---| 2 # 45°
---| 3 # 30°
---| 4 # 15°
---| 5 # 5°

---@alias snap_prev_ctrl # 制御点ハンドルのスナップ設定．
---| 0 # なし
---| 1 # 角度
---| 2 # 長さ
---| 3 # 角度と長さ

local snap_anchors do
	local tau, sin, cos, atan2, abs, min, max, floor = 2 * math.pi, math.sin, math.cos, math.atan2, math.abs, math.min, math.max, math.floor;
	local num_angles = { 4, 8, 12, 24, 72 };

	local function pt(pts, i) return tonumber(pts[i]) or 0 end
	local function norm(x, y) return x ^ 2 + y ^ 2 end
	local function closer(x0, y0, x1, y1, x2, y2)
		if not x1 or norm(x1 - x0, y1 - y0) > norm(x2 - x0, y2 - y0) then return x2, y2 end
		return x1, y1;
	end
	local function snap_cossin(dx, dy, N)
		local a = atan2(dy, dx);
		a = floor(0.5 + N * a / tau) / N * tau;
		local c, s = cos(a), sin(a);
		if abs(c) < 2 ^ -10 then c = 0 end
		if abs(s) < 2 ^ -10 then s = 0 end
		return c, s;
	end

	-- for position points.
	local function snap_pos(x, y, x1, y1, x2, y2, grid_x, grid_y, n_angles, thresh)
		-- grid.
		local X0, Y0;
		if grid_x > 0 then X0 = floor(0.5 + x / grid_x) * grid_x end
		if grid_y > 0 then Y0 = floor(0.5 + y / grid_y) * grid_y end

		-- angles.
		local c1, s1, c2, s2;
		if n_angles > 0 then
			if x1 and y1 then c1, s1 = snap_cossin(x - x1, y - y1, n_angles) end
			if x2 and y2 then c2, s2 = snap_cossin(x - x2, y - y2, n_angles) end
		end

		-- co-dimension of 2 (i.e. points).
		local X, Y;

		-- grid.
		if X0 and Y0 then X, Y = X0, Y0 end

		-- two angles.
		if c1 and c2 then
			local det = c1 * s2 - c2 * s1;
			if det ~= 0 then
				local u, v = -s1 * x1 + c1 * y1, -s2 * x2 + c2 * y2;
				X, Y = closer(x, y, X, Y,
					(c2 * u - c1 * v) / det, (s2 * u - s1 * v) / det);
			end
		end

		-- angle and grid.
		if c1 then
			if X0 and c1 ~= 0 then
				X, Y = closer(x, y, X, Y, X0, (X0 - x1) / c1 * s1 + y1);
			end
			if Y0 and s1 ~= 0 then
				X, Y = closer(x, y, X, Y, (Y0 - y1) / s1 * c1 + x1, Y0);
			end
		end
		if c2 then
			if X0 and c2 ~= 0 then
				X, Y = closer(x, y, X, Y, X0, (X0 - x2) / c2 * s2 + y2);
			end
			if Y0 and s2 ~= 0 then
				X, Y = closer(x, y, X, Y, (Y0 - y2) / s2 * c2 + x2, Y0);
			end
		end

		-- check the threshold.
		if X and Y and norm(X - x, Y - y) <= thresh ^ 2 then return X, Y end

		-- co-dimension of 1 (i.e. lines).
		X, Y = nil, nil;

		-- grid.
		if X0 then X, Y = X0, y end
		if Y0 then X, Y = closer(x, y, X, Y, x, Y0) end

		-- angle.
		if c1 then
			local d = (x - x1) * c1 + (y - y1) * s1;
			X, Y = closer(x, y, X, Y, d * c1 + x1, d * s1 + y1);
		end
		if c2 then
			local d = (x - x2) * c2 + (y - y2) * s2;
			X, Y = closer(x, y, X, Y, d * c2 + x2, d * s2 + y2);
		end

		-- check the threshold.
		if X and Y and norm(X - x, Y - y) <= thresh ^ 2 then return X, Y end

		return x, y;
	end

	-- for control points of quadratic bezier.
	local function snap_ctrl_2(x, y, x1, y1, x2, y2, dx1, dy1, dx2, dy2, n_angles, snap_angle, snap_length, thresh)
		-- from position points.
		local c01, s01, c02, s02;
		if n_angles > 0 then
			c01, s01 = snap_cossin(x - x1, y - y1, n_angles);
			c02, s02 = snap_cossin(x - x2, y - y2, n_angles);
		end

		-- from control points.
		local c1, s1, r1, c2, s2, r2;
		if snap_angle or snap_length then
			if dx1 and dy1 then
				r1 = (dx1 ^ 2 + dy1 ^ 2) ^ 0.5;
				c1, s1 = dx1 / r1, dy1 / r1;
				if c1 * (x - x1) + s1 * (y - y1) < 0 then c1, s1 = -c1, -s1 end
			end
			if dx2 and dy2 then
				r2 = (dx2 ^ 2 + dy2 ^ 2) ^ 0.5;
				c2, s2 = dx2 / r2, dy2 / r2;
				if c2 * (x - x2) + s2 * (y - y2) < 0 then c2, s2 = -c2, -s2 end
			end
			if not snap_angle then c1, s1, c2, s2 = nil, nil, nil, nil end
			if not snap_length then r1, r2 = nil, nil end
		end

		-- co-dimension of 2 (i.e. points).
		local X, Y;

		-- angles from position points.
		if c01 then
			local det = c01 * s02 - c02 * s01;
			if det ~= 0 then
				local u, v = -s01 * x1 + c01 * y1, -s02 * x2 + c02 * y2;
				X, Y = (c02 * u - c01 * v) / det, (s02 * u - s01 * v) / det;
			end
		end

		-- control point.
		if r1 then
			if c1 then
				X, Y = closer(x, y, X, Y,
					r1 * c1 + x1, r1 * s1 + y1);
			end
			X, Y = closer(x, y, X, Y, x1, y1);
		end
		if r2 then
			if c2 then
				X, Y = closer(x, y, X, Y,
					r2 * c2 + x2, r2 * s2 + y2);
			end
			X, Y = closer(x, y, X, Y, x2, y2);
		end

		-- mixed conditions from control points.
		if c1 and c2 then -- two angles.
			local det = c1 * s2 - c2 * s1;
			if det ~= 0 then
				local u, v = -s1 * x1 + c1 * y1, -s2 * x2 + c2 * y2;
				X, Y = closer(x, y, X, Y,
					(c2 * u - c1 * v) / det, (s2 * u - s1 * v) / det);
			end
		end
		if r1 and r2 then -- two radii.
			local l = ((x1 - x2) ^ 2 + (y1 - y2) ^ 2) ^ 0.5;
			if l > 0 and abs(r1 - r2) <= l and l <= r1 + r2 then
				local u = (l ^ 2 + r1 ^ 2 - r2 ^ 2) / (2 * l);
				local v, c, s = (r1 ^ 2 - u ^ 2) ^ 0.5, (x2 - x1) / l, (y2 - y1) / l;
				X, Y = closer(x, y, X, Y, u * c - v * s, u * s + v * c);
				X, Y = closer(x, y, X, Y, u * c + v * s, u * s - v * c);
			end
		end
		if c1 and r2 then -- angle and length.
			local d = -(x1 - x2) * s1 + (y1 - y2) * c1;
			if abs(d) <= r2 then
				local v = (r2 ^ 2 - d ^ 2) ^ 0.5;
				local x12, y12 = -s1 * d + x2, c1 * d + y2;
				X, Y = closer(x, y, X, Y, v * c1 + x12, v * s1 + y12);
				X, Y = closer(x, y, X, Y, -v * c1 + x12, -v * s1 + y12);
			end
		end
		if c2 and r1 then
			local d = -(x2 - x1) * s2 + (y2 - y1) * c2;
			if abs(d) <= r1 then
				local v = (r1 ^ 2 - d ^ 2) ^ 0.5;
				local x21, y21 = -s2 * d + x1, c2 * d + y1;
				X, Y = closer(x, y, X, Y, v * c2 + x21, v * s2 + y21);
				X, Y = closer(x, y, X, Y, -v * c2 + x21, -v * s2 + y21);
			end
		end

		-- mixed conditions from position and control points.
		if c01 then
			if r1 then
				X, Y = closer(x, y, X, Y, r1 * c01 + x1, r1 * s01 + y1);
			end
			if c2 then
				local det = c01 * s2 - c2 * s01;
				if det ~= 0 then
					local u, v = -s01 * x1 + c01 * y1, -s2 * x2 + c2 * y2;
					X, Y = closer(x, y, X, Y,
						(c2 * u - c01 * v) / det, (s2 * u - s01 * v) / det);
				end
			end
		end
		if c02 then
			if r2 then
				X, Y = closer(x, y, X, Y, r2 * c02 + x2, r2 * s02 + y2);
			end
			if c1 then
				local det = c1 * s02 - c02 * s1;
				if det ~= 0 then
					local u, v = -s1 * x1 + c1 * y1, -s02 * x2 + c02 * y2;
					X, Y = closer(x, y, X, Y,
						(c02 * u - c1 * v) / det, (s02 * u - s1 * v) / det);
				end
			end
		end
		if c01 and r2 then
			local d = -(x1 - x2) * s01 + (y1 - y2) * c01;
			if abs(d) <= r2 then
				local v = (r2 ^ 2 - d ^ 2) ^ 0.5;
				local x12, y12 = -s01 * d + x2, c01 * d + y2;
				X, Y = closer(x, y, X, Y, v * c01 + x12, v * s01 + y12);
				X, Y = closer(x, y, X, Y, -v * c01 + x12, -v * s01 + y12);
			end
		end
		if c02 and r1 then
			local d = -(x2 - x1) * s02 + (y2 - y1) * c02;
			if abs(d) <= r1 then
				local v = (r1 ^ 2 - d ^ 2) ^ 0.5;
				local x21, y21 = -s02 * d + x1, c02 * d + y1;
				X, Y = closer(x, y, X, Y, v * c02 + x21, v * s02 + y21);
				X, Y = closer(x, y, X, Y, -v * c02 + x21, -v * s02 + y21);
			end
		end

		-- check the threshold.
		if X and Y and norm(X - x, Y - y) <= thresh ^ 2 then return X, Y end

		-- co-dimension of 1 (i.e. lines).
		X, Y = nil, nil;

		-- angle from a position point.
		if c01 then
			local d = (x - x1) * c01 + (y - y1) * s01;
			X, Y = closer(x, y, X, Y, d * c01 + x1, d * s01 + y1);
		end
		if c02 then
			local d = (x - x2) * c02 + (y - y2) * s02;
			X, Y = closer(x, y, X, Y, d * c02 + x2, d * s02 + y2);
		end
		-- in-between the position points.
		if c01 then
			local l = ((x1 - x2) ^ 2 + (y1 - y2) ^ 2) ^ 0.5;
			if l > 0 then
				local c, s = (x1 - x2) / l, (y1 - y2) / l;
				local d = (x - x1) * c + (y - y1) * s;
				X, Y = closer(x, y, X, Y, d * c + x1, d * s + y1);
			end
		end
		-- angle from a control point.
		if c1 then
			local d = (x - x1) * c1 + (y - y1) * s1;
			X, Y = closer(x, y, X, Y, d * c1 + x1, d * s1 + y1);
		end
		if c2 then
			local d = (x - x2) * c2 + (y - y2) * s2;
			X, Y = closer(x, y, X, Y, d * c2 + x2, d * s2 + y2);
		end
		-- length from a control point.
		if r1 then
			local l = ((x - x1) ^ 2 + (y - y1) ^ 2) ^ 0.5;
			if l > 0 then
				X, Y = closer(x, y, X, Y, (x - x1) / l * r1 + x1, (y - y1) / l * r1 + y1);
			end
		end
		if r2 then
			local l = ((x - x2) ^ 2 + (y - y2) ^ 2) ^ 0.5;
			if l > 0 then
				X, Y = closer(x, y, X, Y, (x - x2) / l * r2 + x2, (y - y2) / l * r2 + y2);
			end
		end

		-- check the threshold.
		if X and Y and norm(X - x, Y - y) <= thresh ^ 2 then return X, Y end

		return x, y;
	end

	-- for control points of cubic bezier.
	local function snap_ctrl_3(dx, dy, dx1, dy1, n_angles, snap_angle, snap_length, thresh)
		-- from position points.
		local c0, s0;
		if n_angles > 0 then c0, s0 = snap_cossin(dx, dy, n_angles) end

		-- from control points.
		local c1, s1, r1;
		if (snap_angle or snap_length) and dx1 and dy1 then
			r1 = (dx1 ^ 2 + dy1 ^ 2) ^ 0.5;
			c1, s1 = dx1 / r1, dy1 / r1;
			if c1 * dx + s1 * dy < 0 then c1, s1 = -c1, -s1 end
			if not snap_angle then c1, s1 = nil, nil end
			if not snap_length then r1 = nil end
		end

		-- co-dimension of 2 (i.e. points).
		local dX, dY;

		-- control point.
		if r1 then
			if c1 then
				dX, dY = r1 * c1, r1 * s1;
			end
			dX, dY = closer(dx, dy, dX, dY, 0, 0);
		end

		-- mixed conditions from position and control points.
		if c0 and r1 then
			dX, dY = closer(dx, dy, dX, dY, r1 * c0, r1 * s0);
		end

		-- check the threshold.
		if dX and dY and norm(dX - dx, dY - dy) <= thresh ^ 2 then return dX, dY end

		-- co-dimension of 1 (i.e. lines).
		dX, dY = nil, nil;

		-- position point.
		if c0 then
			local d = dx * c0 + dy * s0;
			dX, dY = d * c0, d * s0;
		end

		-- control point.
		if c1 then
			local d = dx * c1 + dy * s1;
			dX, dY = closer(dx, dy, dX, dY, d * c1, d * s1);
		end
		if r1 then
			local l = (dx ^ 2 + dy ^ 2) ^ 0.5;
			if l > 0 then
				dX, dY = closer(dx, dy, dX, dY, dx / l * r1, dy / l * r1);
			end
		end

		-- check the threshold.
		if dX and dY and norm(dX - dx, dY - dy) <= thresh ^ 2 then return dX, dY end

		return dx, dy;
	end

	---アンカー位置にスナップを適用する．指定したグリッドや角度に十分近い場合，その位置に点を矯正してテーブルを書き換える．各パラメタは型や範囲のチェックを受け，型が合わない場合は何もせずエラーにもならない．
	---@param path_type path_type|any # パスの種類．
	---@param pts any[]|any # 点列．`{ x1, y1, x2, y2, ... }` の形式．このテーブルの中身が書き換えられる．number 型以外の項目は書き換えない．
	---@param n_pos_pts integer|any # `pts` 内の位置を指定する点 (制御点でない点) の個数．
	---@param loop boolean|any # 閉じたパスかどうか．
	---@param grid_x number|any # 制御点でない点をスナップするグリッドの X 間隔．0 でスナップしない．
	---@param grid_y number|any # 制御点でない点をスナップするグリッドの Y 間隔．0 でスナップしない．
	---@param angle_pos snap_angle|any # 制御点でない点の，直前の点との位置から角度でスナップする設定．ループの場合はループ終端直前と，ループ始点との関係でもスナップする．
	---@param angle_ctrl snap_angle|any # 制御点のハンドルを角度でスナップする設定．
	---@param prev_ctrl snap_prev_ctrl|any # 制御点を，同じ点から伸びる直前の制御点と揃えるためのスナップ設定．角度や長さが一致するようにスナップする．ループの場合，ループ終点から伸びる制御点は，ループ始点から伸びる制御点とも揃えるようにもスナップする．
	---@param threshold number|any # スナップ先に矯正する際の最大許容移動距離．ピクセル単位．
	---@param show_grid boolean|any # `grid_x` や `grid_y` によるスナップ先のグリッドを `obj.setanchor()` を使って表示するかどうか指定．
	---@param mode_anchor_base mode_anchor_base|any # `show_grid` が `true` のときのみ有効．グリッド表示の基準位置を指定．
	function snap_anchors(path_type, pts, n_pos_pts, loop,
		grid_x, grid_y, angle_pos,
		angle_ctrl, prev_ctrl, threshold,
		show_grid, mode_anchor_base)
		-- parameter checks.
		path_type = min(max(floor(0.5 + (tonumber(path_type) or 0)), 0), 3);
		n_pos_pts = floor(0.5 + (tonumber(n_pos_pts) or 0));
		grid_x = tonumber(grid_x) or 0;
		grid_y = tonumber(grid_y) or 0;
		local num_angle_pos = num_angles[floor(0.5 + (tonumber(angle_pos) or 0))] or 0;
		local num_angle_ctrl = num_angles[floor(0.5 + (tonumber(angle_ctrl) or 0))] or 0;
		local prev_angle_ctrl = prev_ctrl == 1 or prev_ctrl == 3;
		local prev_length_ctrl = prev_ctrl == 2 or prev_ctrl == 3;
		threshold = tonumber(threshold) or 0;
		if type(pts) ~= "table" or (n_pos_pts <= 0) or
			(grid_x <= 0 and grid_y <= 0 and num_angle_pos <= 0 and
				(path_type < 2 or (num_angle_ctrl <= 0 and not prev_angle_ctrl and not prev_length_ctrl))) or
			(threshold <= 0) then return end

		local pts_per_seg =
			path_type == 0 and 1 or
			path_type == 1 and 1 or
			path_type == 2 and 2 or 3;

		-- position points.
		do local x, y = tonumber(pts[1]), tonumber(pts[2]);
			if x and y then pts[1], pts[2] = snap_pos(x, y, nil, nil, nil, nil, grid_x, grid_y, num_angle_pos, threshold) end
		end
		for i = 1, n_pos_pts - (loop and 2 or 1) do
			local x1, y1, x, y =
				tonumber(pts[2 * pts_per_seg * (i - 1) + 1]), tonumber(pts[2 * pts_per_seg * (i - 1) + 2]),
				tonumber(pts[2 * pts_per_seg * i + 1]), tonumber(pts[2 * pts_per_seg * i + 2]);
			if x and y then
				pts[2 * pts_per_seg * i + 1], pts[2 * pts_per_seg * i + 2] =
					snap_pos(x, y, x1, y1, nil, nil, grid_x, grid_y, num_angle_pos, threshold);
			end
		end
		if loop then
			local x1, y1, x, y, x2, y2 =
				tonumber(pts[2 * pts_per_seg * (n_pos_pts - 2) + 1]), tonumber(pts[2 * pts_per_seg * (n_pos_pts - 2) + 2]),
				tonumber(pts[2 * pts_per_seg * (n_pos_pts - 1) + 1]), tonumber(pts[2 * pts_per_seg * (n_pos_pts - 1) + 2]),
				tonumber(pts[1]), tonumber(pts[2]);
			if x and y then
				pts[2 * pts_per_seg * (n_pos_pts - 1) + 1], pts[2 * pts_per_seg * (n_pos_pts - 1) + 2] =
					snap_pos(x, y, x1, y1, x2, y2, grid_x, grid_y, num_angle_pos, threshold);
			end
		end

		-- control points.
		if path_type == 2 then
			for i = 1, n_pos_pts - 1 do
				local x1, y1, x, y, x2, y2 =
					tonumber(pts[4 * i - 3]), tonumber(pts[4 * i - 2]),
					tonumber(pts[4 * i - 1]), tonumber(pts[4 * i]),
					tonumber(pts[4 * i + 1]), tonumber(pts[4 * i + 2]);
				if x1 and y1 and x and y and x2 and y2 then
					local dx1, dy1;
					if i > 1 then
						dx1, dy1 = tonumber(pts[4 * i - 5]), tonumber(pts[4 * i - 4]);
						if dx1 and dy1 then dx1, dy1 = dx1 - x1, dy1 - y1 end
					end
					pts[4 * i - 1], pts[4 * i] =
						snap_ctrl_2(x, y, x1, y1, x2, y2, dx1, dy1, nil, nil,
							num_angle_ctrl, prev_angle_ctrl, prev_length_ctrl, threshold);
				end
			end
			if loop then
				local x1, y1, x, y, x2, y2 =
					tonumber(pts[4 * n_pos_pts - 3]), tonumber(pts[4 * n_pos_pts - 2]),
					tonumber(pts[4 * n_pos_pts - 1]), tonumber(pts[4 * n_pos_pts]),
					tonumber(pts[1]), tonumber(pts[2]);
				if x1 and y1 and x and y and x2 and y2 then
					local dx1, dy1, dx2, dy2;
					if n_pos_pts > 1 then
						dx1, dy1 = tonumber(pts[4 * n_pos_pts - 5]), tonumber(pts[4 * n_pos_pts - 4]);
						if dx1 and dy1 then dx1, dy1 = dx1 - x1, dy1 - y1 end
					end
					dx2, dy2 = tonumber(pts[3]), tonumber(pts[4]);
					if dx2 and dy2 then dx2, dy2 = dx2 - x2, dy2 - y2 end
					pts[4 * n_pos_pts - 1], pts[4 * n_pos_pts] =
						snap_ctrl_2(x, y, x1, y1, x2, y2, dx1, dy1, dx2, dy2,
							num_angle_ctrl, prev_angle_ctrl, prev_length_ctrl, threshold);
				end
			end
		elseif path_type == 3 then
			for i = 1, n_pos_pts - (loop and 0 or 1) do
				local dx1, dy1, x1, y1, dx, dy = nil, nil,
					tonumber(pts[6 * i - 5]), tonumber(pts[6 * i - 4]),
					tonumber(pts[6 * i - 3]), tonumber(pts[6 * i - 2]);
				if x1 and y1 and dx and dy then
					if i > 1 then
						dx1, dy1 = tonumber(pts[6 * i - 7]), tonumber(pts[6 * i - 6]);
						if dx1 and dy1 then dx1, dy1 = dx1 - x1, dy1 - y1 end
					end
					dx, dy = dx - x1, dy - y1;
					dx, dy = snap_ctrl_3(dx, dy, dx1, dy1,
						num_angle_ctrl, prev_angle_ctrl, prev_length_ctrl, threshold);
					pts[6 * i - 3], pts[6 * i - 2] = dx + x1, dy + y1;
				end

				if not loop or i < n_pos_pts then
					dx1, dy1, x1, y1 = nil, nil,
						tonumber(pts[6 * i + 1]), tonumber(pts[6 * i + 2]);
				else
					dx1, dy1, x1, y1 =
						tonumber(pts[3]), tonumber(pts[4]),
						tonumber(pts[1]), tonumber(pts[2]);
				end
				dx, dy = tonumber(pts[6 * i - 1]), tonumber(pts[6 * i]);
				if x1 and y1 and dx and dy then
					if dx1 and dy1 then dx1, dy1 = dx1 - x1, dy1 - y1 end
					dx, dy = dx - x1, dy - y1;
					dx, dy = snap_ctrl_3(dx, dy, dx1, dy1,
						num_angle_ctrl, prev_angle_ctrl, prev_length_ctrl, threshold);
					pts[6 * i - 1], pts[6 * i] = dx + x1, dy + y1;
				end
			end
		end

		-- show grid.
		if show_grid == true and (grid_x > 0 or grid_y > 0) then
			local color, max_grid = 0xffffff55, 128;
			local l, t = pt(pts, 1), pt(pts, 2); local r, b = l, t;
			for i = 3, 2 * (n_pos_pts * pts_per_seg - (loop and 0 or pts_per_seg - 1)), 2 do
				local x, y = pt(pts, i), pt(pts, i + 1);
				l, r, t, b = min(l, x), max(r, x), min(t, y), max(b, y);
			end
			l, r, t, b = l - threshold, r + threshold, t - threshold, b + threshold;
			local cx, cy = anchor_offset(PI_choose_mode_anchor_base(nil, tonumber(mode_anchor_base) or 5));

			local setanchor = obj.setanchor;
			if r - l < max_grid * grid_x then
				for x = -floor(-l / grid_x), floor(r / grid_x) do
					setanchor({ x * grid_x, t, x * grid_x, b }, 2, "line", "rgba", color, "offset", cx, cy);
				end
			end
			if b - t < max_grid * grid_y then
				for y = -floor(-t / grid_y), floor(b / grid_y) do
					setanchor({ l, y * grid_y, r, y * grid_y }, 2, "line", "rgba", color, "offset", cx, cy);
				end
			end
		end
	end
end

local anchor, poll do
	local function pt(pts, i) return tonumber(pts[i]) or 0 end
	local function unpack1(x) if x ~= nil then return x end end

	---places anchors for the path.
	---@param var_name string 点列を受け取る変数の名前．
	---@param path_type path_type パスの種類．
	---@param pts any[] 点列の配列， `{ x1, y1, x2, y2, x3, y3, ... }` の形式．
	---@param n_segs integer パスの分割区間の個数．
	---@param loop boolean 閉じたパスかどうか．
	---@param alt_points nil|fun(path_type: path_type, pts: any[], n_segs: integer, loop: boolean): number[] # 想定されるアンカーの個数が `pts` の点の個数と異なる場合に呼ばれる関数．代替となる点列テーブルを返す．省略時は独自の方法で代替の点列を構築する．
	---@param mode_anchor_base mode_anchor_base? アンカー基準．省略時は `5` (中央).
	---@return integer n_anchors 設定したアンカーの個数．
	---@return table pts_corrected 足りない点や余剰な点を補正した点列．補正の必要がない場合は `pts` そのもの．
	function anchor(var_name, path_type, pts, n_segs, loop, alt_points, mode_anchor_base)
		local pts_per_seg =
			path_type == 0 and 1 or
			path_type == 1 and 1 or
			path_type == 2 and 2 or 3;
		local n_anchors = (loop and 0 or 1) + pts_per_seg * n_segs;
		local alt_pts = nil;
		if 2 * n_anchors ~= #pts then
			if alt_points then
				alt_pts = alt_points(path_type, pts, n_segs, loop);
				if 2 * n_anchors ~= #alt_pts then alt_pts = nil end
			end
			if alt_pts == nil then
				alt_pts = {};
				local n = math.min(n_anchors, math.floor(#pts / 2));
				for i = 1, 2 * n do alt_pts[i] = pt(pts, i) end

				if n < n_anchors then
					-- find a suitable placeholder point.
					local k = 1 + math.floor((n - 1) / pts_per_seg) * pts_per_seg;
					local X, Y = pt(pts, 2 * k - 1), pt(pts, 2 * k); -- last point.
					local dx, dy;
					if loop then
						dx, dy = (pt(pts, 1) - X) / 2, (pt(pts, 2) - Y) / 2;
					else
						k = math.max(1, k - pts_per_seg);
						dx, dy = (X - pt(pts, 2 * k - 1)) / 2, (Y - pt(pts, 2 * k)) / 2;
					end
					local l = dx ^ 2 + dy ^ 2;
					if l > 100 ^ 2 then
						l = 100 / l ^ 0.5; -- at most 100 pixel far.
						dx, dy = l * dx, l * dy;
					end
					X, Y = X + dx, Y + dy;

					-- fill the rest with that placeholder.
					for i = n + 1, n_anchors do
						alt_pts[2 * i - 1], alt_pts[2 * i] = X, Y;
					end
				end
			end
		end

		local cx, cy = anchor_offset(mode_anchor_base or 5);
		if path_type >= 2 then
			obj.setanchor(var_name, n_anchors, "offset", cx, cy, unpack1(alt_pts));

			-- draw handles.
			local pts2, pts3 = alt_pts or pts, {};
			for i = 1, n_segs do
				local I, J = 2 * pts_per_seg * i, 2 * pts_per_seg * (loop and (i % n_segs) or i);
				obj.setanchor({
					pt(pts2, I - 2 * pts_per_seg + 1), pt(pts2, I - 2 * pts_per_seg + 2),
					pt(pts2, I - 2 * pts_per_seg + 3), pt(pts2, I - 2 * pts_per_seg + 4),
					pt(pts2, I - 1), pt(pts2, I - 0),
					pt(pts2, J + 1), pt(pts2, J + 2),
				}, 4, "offset", cx, cy, "line", "inout");
				pts3[2 * i - 1], pts3[2 * i] = pt(pts2, I - 2 * pts_per_seg + 1), pt(pts2, I - 2 * pts_per_seg + 2);
			end
			if not loop then
				pts3[2 * n_segs + 1], pts3[2 * n_segs + 2] =
					pt(pts2, 2 * pts_per_seg * n_segs + 1), pt(pts2, 2 * pts_per_seg * n_segs + 2);
			end
			obj.setanchor(pts3, n_segs + (loop and 0 or 1),
				"offset", cx, cy, loop and "loop" or "line");
		else
			obj.setanchor(var_name, n_anchors, "offset", cx, cy, unpack1(alt_pts));
			obj.setanchor(alt_pts or pts, n_segs + (loop and 0 or 1),
				"offset", cx, cy, loop and "loop" or "line");
		end
		return n_anchors, alt_pts or pts;
	end

	local interpolation = obj.interpolation;
	local function bezier2_1d(t, a0, a1, a2)
		return (1 - t) ^ 2 * a0 + 2 * t * (1 - t) * a1 + t ^ 2 * a2;
	end
	local function bezier2(t, x0, y0, x1, y1, x2, y2)
		return bezier2_1d(t, x0, x1, x2), bezier2_1d(t, y0, y1, y2);
	end
	local function bezier3_1d(t, a0, a1, a2, a3)
		return (1 - t) ^ 3 * a0 + 3 * t * (1 - t) ^ 2 * a1 + 3 * t ^ 2 * (1 - t) * a2 + t ^ 3 * a3;
	end
	local function bezier3(t, x0, y0, x1, y1, x2, y2, x3, y3)
		return bezier3_1d(t, x0, x1, x2, x3), bezier3_1d(t, y0, y1, y2, y3);
	end

	local pts2, n_pts2, stack, r2 = {}, 0, {}, 0;
	local function poll_section(curve, apriori, ...)
		local rem = 0;
		-- firstly, collect certain number of mid-points.
		for i = 1, apriori do
			local t = 1 - (i - 1) / apriori;
			rem = rem + 1;
			stack[3 * rem - 2], stack[3 * rem - 1], stack[3 * rem] = t, curve(t, ...);
		end

		-- iterate until the stack clears up.
		local s = 0;
		while rem > 0 do
			local x, y, t, X, Y =
				pts2[2 * n_pts2 - 1], pts2[2 * n_pts2],
				stack[3 * rem - 2], stack[3 * rem - 1], stack[3 * rem];
			if (X - x) ^ 2 + (Y - y) ^ 2 < r2 then
				-- point near enough.
				rem = rem - 1;
				n_pts2 = n_pts2 + 1;
				s, pts2[2 * n_pts2 - 1], pts2[2 * n_pts2] = t, X, Y;
			else
				-- too far, calculate the intermediate.
				local u = (s + t) / 2;
				rem = rem + 1;
				stack[3 * rem - 2], stack[3 * rem - 1], stack[3 * rem] = u, curve(u, ...);
			end
		end
	end

	---曲線を表すパスを折れ線の列に変換する．
	---@param path_type path_type パスの種類．折れ線の場合は実質 shallow copy が取られる．
	---@param pts any[] 点列の配列， `{ x1, y1, x2, y2, x3, y3, ... }` の形式．
	---@param n_segs integer パスの分割区間の個数．1 以上．
	---@param loop boolean 閉じたパスかどうか．
	---@param prec number 折れ線の許容最長距離．
	---@return number[] pts2 結果の折れ線の頂点を表す点列．`loop` が true の場合は末尾には最初と同じ点が格納される．
	---@return integer n_pts2 `pts2` に含まれる点の個数．
	function poll(path_type, pts, n_segs, loop, prec)
		local ret, n_ret;
		if path_type == 0 then
			ret, n_ret = {}, n_segs + 1;
			-- essentially a shallow copy of `pts`.
			for i = 1, 2 * n_ret do ret[i] = pt(pts, i) end
			if loop then
				-- place a copy of the first point at the end.
				ret[2 * n_ret - 1], ret[2 * n_ret] = ret[1], ret[2];
			end
		else
			-- replace the curve into a sequence of secants.
			-- prepare poll_section() function.
			pts2, n_pts2, stack, r2 = { pt(pts, 1), pt(pts, 2) }, 1, {}, prec ^ 2;

			if path_type == 1 then
				-- built-in interpolation function.
				for i = 1, n_segs do
					local x0, y0, x2, y2, x3, y3 =
						pt(pts, 2 * i - 3), pt(pts, 2 * i - 2),
						pt(pts, 2 * i + 1), pt(pts, 2 * i + 2),
						pt(pts, 2 * i + 3), pt(pts, 2 * i + 4);
					if loop then
						if i == 1 then x0, y0 = pt(pts, 2 * n_segs - 1), pt(pts, 2 * n_segs) end
						if i >= n_segs - 1 then
							local j = i - n_segs;
							x3, y3 = pt(pts, 2 * j + 3), pt(pts, 2 * j + 4);
							if j >= 0 then x2, y2 = pt(pts, 2 * j + 1), pt(pts, 2 * j + 2) end
						end
					else
						if i == 1 then x0, y0 = pts2[1], pts2[2] end
						if i >= n_segs then x3, y3 = x2, y2 end
					end
					local x1, y1 = pts2[2 * n_pts2 - 1], pts2[2 * n_pts2];
					if x0 ~= x1 or x0 ~= x2 or x0 ~= x3 or y0 ~= y1 or y0 ~= y2 or y0 ~= y3 then
						poll_section(interpolation, 4,
							x0, y0, x1, y1, x2, y2, x3, y3);
					end
				end
			elseif path_type == 2 then
				-- quadratic Bezier curve.
				for i = 1, n_segs do
					local j = (i >= n_segs and loop) and 0 or i;
					local x1, y1, x2, y2 =
						pt(pts, 4 * i - 1), pt(pts, 4 * i - 0),
						pt(pts, 4 * j + 1), pt(pts, 4 * j + 2);
					local x0, y0 = pts2[2 * n_pts2 - 1], pts2[2 * n_pts2];
					if x0 ~= x1 or x0 ~= x2 or y0 ~= y1 or y0 ~= y2 then
						poll_section(bezier2, 2,
							x0, y0, x1, y1, x2, y2);
					end
				end
			else
				-- cubic Bezier curve.
				for i = 1, n_segs do
					local j = (i >= n_segs and loop) and 0 or i;
					local x1, y1, x2, y2, x3, y3 =
						pt(pts, 6 * i - 3), pt(pts, 6 * i - 2),
						pt(pts, 6 * i - 1), pt(pts, 6 * i - 0),
						pt(pts, 6 * j + 1), pt(pts, 6 * j + 2);
					local x0, y0 = pts2[2 * n_pts2 - 1], pts2[2 * n_pts2];
					if x0 == x1 and y0 == y1 and x2 == x3 and y2 == y3 then
						if x1 ~= x2 or y1 ~= y2 then
							n_pts2 = n_pts2 + 1;
							pts2[2 * n_pts2 - 1], pts2[2 * n_pts2] = x3, y3;
						end
					else
						poll_section(bezier3, 4,
							x0, y0, x1, y1, x2, y2, x3, y3);
					end
				end
			end

			if n_pts2 < 2 then
				-- ensure to have at least two points.
				n_pts2 = n_pts2 + 1;
				pts2[2 * n_pts2 - 1], pts2[2 * n_pts2] = pts2[1], pts2[2];
			end

			-- store the result.
			ret, n_ret, pts2, n_pts2, stack, r2 = pts2, n_pts2, {}, 0, {}, 0;
		end

		return ret, n_ret;
	end
end

---折れ線の bounding box と長さを計算．
---@param pts number[] 折れ線の頂点を表す点列．
---@param n_pts integer `pts` に含まれる点の個数．1 以上．
---@return number L, number R, number T, number B bounding box の座標．
---@return number length 折れ線の長さ．
local function measure(pts, n_pts)
	local x0, y0 = pts[1], pts[2];
	local L, R, T, B, length = x0, x0, y0, y0, 0;
	for i = 3, 2 * n_pts - 1, 2 do
		local x, y = pts[i], pts[i + 1];
		L = math.min(L, x); R = math.max(R, x);
		T = math.min(T, y); B = math.max(B, y);
		length = length + ((x - x0) ^ 2 + (y - y0) ^ 2) ^ 0.5;
		x0, y0 = x, y;
	end
	return L, R, T, B, length;
end

---折れ線を表す配列で，始点から距離 `pos` だけ離れた位置にある点の前後にある頂点のインデックスを二分法で検索．
---@param pos number 始点からの距離．0 以上の場合はピクセル数で指定，負の場合は `pos >= -1.0` の必要があり，パスの総長との比を絶対値で指定．
---@param tbl number[] `n_pts` を指定した場合，点列を `{ x1, y1, x2, y2, ... }` の形式で指定．`n_pts` が `nil` の場合，各頂点の始点からの累計距離の配列を `{ 0, l1, l2, ... }` の形式で指定．
---@param n_pts integer? `tbl` に点列を指定した場合， `tbl` に含まれる点の個数． 累計距離の配列の場合は `nil`.
---@return integer int_part `pos` の前後位置にある頂点のインデックスのうち小さいほう．
---@return number frac_part `int_part` から次の頂点までの相対位置を表す，0.0 から 1.0 までの数値．
---@return number[] lengths 各頂点の始点からの累計距離の配列．2回目以降の呼び出しで使うと一部計算を省略できる．
local function find_index(pos, tbl, n_pts)
	if n_pts then
		-- construct an array of accumulated lengths.
		local lengths = { 0.0 };
		local x0, y0 = tbl[1], tbl[2];
		for i = 2, n_pts do
			local x, y = tbl[2 * i - 1], tbl[2 * i];
			lengths[i] = lengths[i - 1] + ((x - x0) ^ 2 + (y - y0) ^ 2) ^ 0.5;
			x0, y0 = x, y;
		end
		tbl = lengths;
	else n_pts = #tbl end
	if pos < 0 then pos = tbl[n_pts] * (-pos) end

	-- find the secant where the `pos` is placed on.
	local i, j = 1, n_pts;
	while i + 1 < j do
		local k = math.floor((i + j) / 2);
		if tbl[k] <= pos then i = k else j = k end
	end

	-- calculate the relative position between indices.
	j = tbl[i + 1] - tbl[i];
	j = j > 0 and (pos - tbl[i]) / j or 0.5;

	-- return an integer-fraction pair.
	return i, j, tbl;
end

---点列に対して拡縮回転平行移動を適用する．
---@param pts number[] 適用先の点列．このテーブルの内容を書き換える．
---@param n_pts integer `pts` に含まれる点の個数．0 以上．
---@param scale number 拡縮の比率．0 以上の実数．
---@param rotate number ラジアン単位の回転角．
---@param dx number? X方向の平行移動量．省略時は 0. 拡縮回転の後に適用される．
---@param dy number? Y方向の平行移動量．省略時は 0. 拡縮回転の後に適用される．
local function transform(pts, n_pts, scale, rotate, dx, dy)
	dx, dy = dx or 0, dy or 0;
	if scale == 1 and rotate == 0 and dx == 0 and dy == 0 then return end
	local c, s = scale * math.cos(rotate), scale * math.sin(rotate);
	for i = 1, 2 * n_pts - 1, 2 do
		local x, y = pts[i], pts[i + 1];
		pts[i], pts[i + 1] =
			(c * x - s * y) + dx,
			(s * x + c * y) + dy;
	end
end

local rotation_count do
	local function quadrant(x, y)
		if x >= 0 then return y >= 0 and 0 or 3;
		else return y >= 0 and 1 or 2 end
	end
	---ループする点列に対して，回転回数を計算する．時計回りに正．
	---@param pts number[] ループする点列．`{ x1, y1, x2, y2, ... }` の形式．始点と終点は同じ点である必要がある．
	---@param n_segs integer `pts` に含まれる線分の個数．3 以上．
	---@return number # 回転回数．時計回りに正．
	function rotation_count(pts, n_segs)
		local x0, y0, x1, y1 = pts[2 * n_segs - 1], pts[2 * n_segs], pts[1], pts[2];
		local dx0, dy0 = x1 - x0, y1 - y0;
		local q0 = quadrant(dx0, dy0);
		local count = 0;
		for i = 1, n_segs do
			local x2, y2 = pts[2 * i + 1], pts[2 * i + 2];
			local dx1, dy1 = x2 - x1, y2 - y1;
			local q1 = quadrant(dx1, dy1);
			if dx0 * dy1 - dy0 * dx1 >= 0 then
				if q1 < q0 then count = count + 1 end
			else
				if q1 > q0 then count = count - 1 end
			end
			x0, y0, x1, y1 = x1, y1, x2, y2;
			dx0, dy0, q0 = dx1, dy1, q1;
		end
		return count;
	end
end

local disk_rand do
	local setmetatable, tau, cos, sin, rand1 =
		setmetatable, 2 * math.pi, math.cos, math.sin, obj.rand1;
	local mt_disk_rand = {
		__call = function (self)
			self[1] = self[1] + 2;
			local a, r =
				tau * rand1(self[2], self[1] - 1),
				self[3] * rand1(self[2], self[1]) ^ 0.5;
			return r * cos(a), r * sin(a);
		end
	};
	-- PRNG uniform on a disk with the specified radius.
	function disk_rand(seed, radius)
		return setmetatable({ 1, seed, radius }, mt_disk_rand);
	end
end
---パスに対してランダム移動を適用する．乱数器は `obj.rand1()`．
---@param pts number[] 適用先の点列．このテーブルの内容は書き換わらない．
---@param n_pts integer `pts` に含まれる点の個数．2 以上．
---@param period number ランダムを適用する距離周期，ピクセル単位．
---@param rand_range number 各点がランダム移動する最大距離，ピクセル単位．
---@param end_mode 0|1|2 両端での特別扱いを指定．0: 特になし, 1: 両端は固定, 2: 両端のランダム移動量を揃える (ループ用).
---@param seed number `obj.rand1()` へ渡す乱数シード値．
---@return number[] pts2 ランダム移動で得られた新しい点列．
---@return integer n_pts2 `pts2` に含まれる点の個数．
local function randomize(pts, n_pts, period, rand_range, end_mode, seed)
	local rng = disk_rand(seed, rand_range);
	local x_r0, y_r0 = rng();
	local x_r1, y_r1 = rng();
	local p = 0;

	-- for fixed ends or a loop, adjust the period and the first random value.
	local rand_rest, x_r2, y_r2 = -1, 0, 0;
	if end_mode == 1 or end_mode == 2 then
		local _, _, _, _, len = measure(pts, n_pts);
		rand_rest = math.max(math.floor(0.5 + len / period), 2);
		period = len / rand_rest;
		rand_rest = rand_rest - 2;

		if end_mode == 1 then x_r0, y_r0 = 0, 0 end
		x_r2, y_r2 = x_r0, y_r0;
	end

	local n_ret, x0, y0 = 1, pts[1], pts[2];
	local ret = { x0 + x_r0, y0 + y_r0 };
	for i = 3, 2 * n_pts - 1, 2 do
		local x1, y1 = pts[i], pts[i + 1];
		local q = p + ((x1 - x0) ^ 2 + (y1 - y0) ^ 2) ^ 0.5 / period;
		while q >= 1 do
			local t = (1 - p) / (q - p);
			n_ret, x0, y0 = n_ret + 1, (1 - t) * x0 + t * x1, (1 - t) * y0 + t * y1;
			x_r0, y_r0, x_r1, y_r1 = x_r1, y_r1, rng();
			p, q = 0, q - 1;
			ret[2 * n_ret - 1], ret[2 * n_ret] = x0 + x_r0, y0 + y_r0;

			-- manipulate the random values for fixed ends or a loop.
			if rand_rest == 0 then x_r1, y_r1 = x_r2, y_r2 end
			rand_rest = rand_rest - 1;
		end
		p = q;

		if p > 0 then
			n_ret, x0, y0 = n_ret + 1, x1, y1;
			ret[2 * n_ret - 1], ret[2 * n_ret] =
				x0 + (1 - p) * x_r0 + p * x_r1, y0 + (1 - p) * y_r0 + p * y_r1;
		end
	end

	return ret, n_ret;
end

local send, retrieve do
	local function encode_float(x)
		local d, m, M = 2 ^ 8, -2 ^ 23, 2 ^ 23 - 1;
		return bit.bor(0xff000000, math.min(math.max(math.floor(x * d + 0.5), m), M));
	end
	local function decode_float(c)
		local d = 2 ^ 16;
		return bit.lshift(c, 8) / d;
	end
	local buf, intptr_t, uint32_t_array, uint32_t_ptr =
		buffer.new(9), ffi.typeof("intptr_t"), ffi.typeof("uint32_t[?]"), ffi.typeof("uint32_t*");
	-- converts a pointer to a light userdata.
	local function to_userdata(ptr)
		-- 0x05: lightud64. (https://luajit.org/ext_buffer.html)
		buf:reset():encode(ffi.cast(intptr_t, ptr)):ref()[0] = 0x05;
		return buf:decode();
	end

	---点列 `pts` の情報を `target` で指定したバッファに転送する．転送したデータはシェーダーで点列として読み取れるようになる．
	---@param pts number[] 折れ線の頂点を表す点列．
	---@param n_pts integer `pts` に含まれる点の個数．
	---@param dx number? X方向の平行移動量．省略時は 0.
	---@param dy number? Y方向の平行移動量．省略時は 0.
	---@param target string? `"tempbuffer"` あるいは `"cache:..."` の形式でバッファを指定．省略時は `"tempbuffer"`.
	function send(pts, n_pts, dx, dy, target)
		dx, dy = dx or 0, dy or 0;
		local max_width = 2 ^ 12;
		local w, h =
			math.min(2 * n_pts, max_width),
			math.ceil(2 * n_pts / max_width)
		local data = uint32_t_array(w * h);
		for i = 1, 2 * n_pts, 2 do
			data[i - 1] = encode_float(pts[i] + dx);
			data[i] = encode_float(pts[i + 1] + dy);
		end
		obj.putpixeldata(target or "tempbuffer", to_userdata(data), w, h);
	end

	---`target` で指定したバッファから点列情報を復元する．
	---@param n_pts integer バッファに含まれている点の個数．
	---@param target string? `"tempbuffer"` あるいは `"cache:..."` の形式でバッファを指定．省略時は `"tempbuffer"`.
	---@return number[]? pts 求める点列を表す配列．もしバッファのサイズが `n_pts` と想定されない場合は `nil`.
	function retrieve(n_pts, target)
		local max_width = 2 ^ 12;
		local w, h =
			math.min(2 * n_pts, max_width),
			math.ceil(2 * n_pts / max_width);
		local data, W, H = obj.getpixeldata(target or "tempbuffer");
		if H < h or (h > 1 and W ~= w) or W < w then return nil end
		local ret, ptr = {}, uint32_t_ptr(data);
		for i = 1, 2 * n_pts do ret[i] = decode_float(ptr[i - 1]) end
		return ret;
	end
end

local function mask_uniform(alpha, target)
	if alpha <= 0 then obj.clearbuffer(target);
	elseif alpha < 1 then
		obj.pixelshader("const_alpha@パスマスクσ@Path_S", target, nil, { alpha }, "mask");
	end
end

---パスマスクσ をバッファに送った点列データを元に適用する．
---@param alpha_outer number パス外側のマスクのアルファ値を 0.0 から 1.0 で指定．
---@param alpha_inner number パス内側のマスクのアルファ値を 0.0 から 1.0 で指定．
---@param mode_fill mode_fill 塗りつぶし範囲の指定．
---@param inflation number 「追加幅」をピクセル単位で指定．0 以上の実数．
---@param antialias antialias 「ぼかし幅」をピクセル単位で指定，0.0 以上．またはディザ設定のテーブル．
---@param buffer_name string 折れ線の頂点データのあるバッファ名．
---@param num_points integer バッファに含まれる頂点数．3 以上．
---@param target_buffer { name: string, w: integer, h: integer }? マスク適用先のバッファ名 (e.g. "object", "cache:foo") とその幅と高さを指定．省略時は `{ name = "object", w = obj.w, h = obj.h }`.
local function path_mask_area_buffered(
	alpha_outer, alpha_inner, mode_fill,
	inflation, antialias,
	buffer_name, num_points,
	target_buffer)
	-- normalize parameters.
	local tgt_name = target_buffer and target_buffer.name or "object";
	antialias = wrap_antialias(antialias);

	-- handle the trivial case.
	if alpha_outer == alpha_inner then mask_uniform(alpha_outer, tgt_name); return end

	-- mask with the path.
	local dc, ds = math.cos(antialias.rot) / antialias.size, math.sin(antialias.rot) / antialias.size;
	obj.pixelshader("carve@パスマスクσ@Path_S", tgt_name, buffer_name,
	{
		alpha_inner - alpha_outer, alpha_outer;
		num_points, mode_fill, inflation;
		math.max(antialias.width, 1 / 1024);
		antialias.cx, antialias.cy;
		dc, -ds, 0, 0,
		ds, dc;
		math.max(1 - antialias.rate, 1 / 1024), antialias.seed; antialias.pattern;
	}, "mask");
end

---パスマスクσ を点列を元に適用する．
---@param alpha_outer number パス外側のマスクのアルファ値を 0.0 から 1.0 で指定．
---@param alpha_inner number パス内側のマスクのアルファ値を 0.0 から 1.0 で指定．
---@param mode_fill mode_fill 塗りつぶし範囲の指定．
---@param inflation number 「追加幅」をピクセル単位で指定．0 以上の実数．
---@param antialias antialias 「ぼかし幅」をピクセル単位で指定，0.0 以上．またはディザ設定のテーブル．
---@param path_type path_type|nil 「線タイプ」(パスの種類) を指定．`nil` を指定した場合，折れ線への変換や点列のコピーを省略する．このとき `pts` は既に折れ線の前提で，テーブルの内容も書き換わる．
---@param pts any[] 点列の配列， `{ x1, y1, x2, y2, x3, y3, ... }` の形式．
---@param n_segs integer パスの分割区間の個数．1 以上．
---@param prec number 「曲線精度」(折れ線の許容最長距離) を指定．
---@param scale number 「拡大率」を指定．0 以上の実数．
---@param rotate number 「回転」を指定．
---@param dx number 「移動X」を指定．拡縮回転の後に適用．省略時は 0.
---@param dy number 「移動Y」を指定．拡縮回転の後に適用．省略時は 0.
---@param target_buffer { name: string, w: integer, h: integer }? マスク適用先のバッファ名 (e.g. "object", "cache:foo") とその幅と高さを指定．省略時は `{ name = "object", w = obj.w, h = obj.h }`.
---@param temp_buffer_name string? 頂点データを転送する先のバッファ名 (e.g. "tempbuffer", "cache:foo"). 省略時は "tempbuffer".
local function path_mask_area(
	alpha_outer, alpha_inner, mode_fill,
	inflation, antialias,
	path_type, pts, n_segs, prec,
	scale, rotate, dx, dy,
	target_buffer, temp_buffer_name)
	-- normalize parameters.
	local tgt_name, tgt_w, tgt_h = "object", obj.w, obj.h;
	if target_buffer then
		tgt_name, tgt_w, tgt_h = target_buffer.name, target_buffer.w, target_buffer.h;
	end
	antialias = wrap_antialias(antialias);

	-- handle the trivial case.
	if alpha_outer == alpha_inner then mask_uniform(alpha_outer, tgt_name); return end

	-- make the curve into the sequence of secants.
	local points, num_points = pts, n_segs + 1;
	if path_type then
		points, num_points = poll(path_type, pts, n_segs, true,
			prec / math.min(math.max(scale, 1 / 64), 1));
	end

	-- apply translation / scaling / rotation.
	transform(points, num_points, scale, rotate, dx, dy);
	local L, R, T, B = measure(points, num_points);
	local th = inflation + antialias.width;
	L, R, T, B = L - th, R + th, T - th, B + th;

	-- check if the path overlaps this object.
	if L >= tgt_w / 2 or R <= -tgt_w / 2 or T >= tgt_h / 2 or B <= -tgt_h / 2 then
		mask_uniform(mode_fill >= 2 and alpha_inner or alpha_outer, tgt_name);
		return;
	end

	-- send the coordinates to tempbuffer.
	temp_buffer_name = temp_buffer_name or "tempbuffer";
	send(points, num_points, tgt_w / 2, tgt_h / 2, temp_buffer_name);

	path_mask_area_buffered(
		alpha_outer, alpha_inner, mode_fill,
		inflation, antialias, temp_buffer_name, num_points,
		target_buffer);
end

---パスマスク(ライン)σ をバッファに送った点列データを元に適用する．
---@param alpha_outer number パス外側のマスクのアルファ値を 0.0 から 1.0 で指定．
---@param alpha_inner number パス内側のマスクのアルファ値を 0.0 から 1.0 で指定．
---@param line_width number 「ライン幅」をピクセル単位で指定．0 以上の実数．
---@param antialias antialias 「ぼかし幅」をピクセル単位で指定，0.0 以上．またはディザ設定のテーブル．
---@param buffer_name string 折れ線の頂点データのあるバッファ名．
---@param num_points integer バッファに含まれる頂点数．2 以上．
---@param len_path number バッファに含まれる折れ線の累計長さ．
---@param loop boolean 閉じたパスかどうか．
---@param start_pos number 「開始位置」を 0.0 から 1.0 に正規化した数値で指定．ただしループの場合はこの範囲を超えることもある．
---@param end_pos number 「終了位置」を 0.0 から 1.0 に正規化した数値で指定．ただしループの場合はこの範囲を超えることもある．
---@param end_shape end_shape 「端の形状」を指定．
---@param join_shape join_shape 「線結合の形状」を指定．
---@param miter_limit number 「マイター限界」を指定．1.0 以上の実数．
---@param dash_pat number[] 「破線パターン」を `{ opaque_len1, blank_len1, opaque_len2, blank_len2, ... }` の形式で指定．
---@param dash_pos number 「破線位置」をピクセル単位で指定．
---@param dash_adj boolean 「破線周期補正」を指定．
---@param dash_end_shape end_shape 破線の「端の形状」を指定．
---@param target_buffer { name: string, w: integer, h: integer }? マスク適用先のバッファ名 (e.g. "object", "cache:foo") とその幅と高さを指定．省略時は `{ name = "object", w = obj.w, h = obj.h }`.
local function path_mask_line_buffered(
	alpha_outer, alpha_inner, line_width, antialias,
	buffer_name, num_points, len_path, loop,
	start_pos, end_pos, end_shape, join_shape, miter_limit,
	dash_pat, dash_pos, dash_adj, dash_end_shape,
	target_buffer)
	-- normalize parameters.
	local tgt_name = target_buffer and target_buffer.name or "object";
	antialias = wrap_antialias(antialias);

	-- handle the trivial case.
	if alpha_outer == alpha_inner then mask_uniform(alpha_outer, tgt_name); return end

	if line_width < 1 then
		-- fade out the line as the width approaches zero.
		alpha_inner = line_width * alpha_inner + (1 - line_width) * alpha_outer;
	end

	local phase_whole0, phase_whole1, phase_whole2;
	if end_pos < start_pos then
		phase_whole0, phase_whole1, phase_whole2 = 2, 0, 0;
	elseif loop then
		start_pos, end_pos = start_pos % 1, end_pos - math.floor(start_pos);
		if end_pos - start_pos >= 1 then
			phase_whole0, phase_whole1, phase_whole2 = 0, 2, 0;
		elseif end_pos <= 1 then
			phase_whole0, phase_whole1, phase_whole2 = start_pos, end_pos - start_pos, 2;
		else
			phase_whole0, phase_whole1, phase_whole2 = 0, end_pos - 1, 1 + start_pos - end_pos;
		end
	else
		start_pos = math.min(math.max(start_pos, 0), 1);
		end_pos = math.min(math.max(end_pos, 0), 1);
		phase_whole0 = math.max(start_pos, 0);
		phase_whole1 = math.max(end_pos - phase_whole0, 0);
		phase_whole2 = 2;
	end

	-- dash pattern.
	local dash_len0, dash_idx0, sum_dash_len = 0, 0, 0;
	if #dash_pat <= 256 then
		for i,v in ipairs(dash_pat) do
			v = math.max(tonumber(v) or 0, 0);
			sum_dash_len = sum_dash_len + v;
			dash_pat[i] = v;
		end
	end
	if sum_dash_len > 0 then
		-- normalize so zero does not occur other than the head.
		local j, pat, len_rest = 1, { 0 }, (-dash_pos) % sum_dash_len;
		for i, v in ipairs(dash_pat) do
			if v ~= 0 then
				if (i - j) % 2 ~= 0 then
					pat[j] = pat[j] + v;
				else
					j = j + 1;
					pat[j] = v;
				end

				if len_rest >= 0 then
					len_rest = len_rest - v;
					if len_rest < 0 then
						dash_len0 = -len_rest;
						dash_idx0 = j;
					end
				end
			end
		end
		if j % 2 == 1 then
			pat[1] = pat[j];
			pat[j] = nil;
			j = j - 1;
		end

		dash_pat = pat; -- replace with the normalized one.
		if j <= 2 then
			if j < 2 or pat[1] == 0 then
				sum_dash_len = 0; -- all opaque.
			end
		end
	end
	if sum_dash_len <= 0 then
		dash_len0, dash_idx0 = len_path * 2, 2;
		dash_pat = { 0, len_path * 2 };
	elseif loop and dash_adj then
		local adj = len_path / sum_dash_len;
		adj = adj / math.max(math.floor(0.5 + adj), 1);
		sum_dash_len = adj * sum_dash_len;
		dash_len0 = adj * dash_len0;
		for i, v in ipairs(dash_pat) do dash_pat[i] = adj * v end
	end

	-- mask with the path.
	local dc, ds = math.cos(antialias.rot) / antialias.size, math.sin(antialias.rot) / antialias.size;
	if end_pos - start_pos >= 1 and sum_dash_len <= 0 then
		obj.pixelshader("carve@パスマスク(ライン)σ@Path_S", tgt_name, buffer_name,
		{
			alpha_inner - alpha_outer, alpha_outer;
			num_points, loop and 1 or 0;
			math.max(line_width - 1, 0) / 2, math.max(antialias.width, 1 / 1024);

			antialias.cx, antialias.cy;
			dc, -ds, 0, 0,
			ds, dc;
			math.max(1 - antialias.rate, 1 / 1024); antialias.seed; antialias.pattern;

			end_shape; join_shape; 1 - 2 / miter_limit ^ 2;
		}, "mask");
	else
		obj.pixelshader("carve_dash@パスマスク(ライン)σ@Path_S", tgt_name, buffer_name,
		{
			alpha_inner - alpha_outer, alpha_outer;
			num_points, loop and 1 or 0;
			math.max(line_width - 1, 0) / 2, math.max(antialias.width, 1 / 1024);

			antialias.cx, antialias.cy;
			dc, -ds, 0, 0,
			ds, dc;
			math.max(1 - antialias.rate, 1 / 1024); antialias.seed; antialias.pattern;

			#dash_pat, dash_len0, dash_idx0 - 1;

			end_shape; join_shape; dash_end_shape; 1 - 2 / miter_limit ^ 2;

			len_path * phase_whole0, len_path * phase_whole1, len_path * phase_whole2, len_path * 2;
			unpack(dash_pat)
		}, "mask");
	end
end

---パスマスク(ライン)σ を点列を元に適用する．
---@param alpha_outer number パス外側のマスクのアルファ値を 0.0 から 1.0 で指定．
---@param alpha_inner number パス内側のマスクのアルファ値を 0.0 から 1.0 で指定．
---@param line_width number 「ライン幅」をピクセル単位で指定．0 以上の実数．
---@param antialias antialias 「ぼかし幅」をピクセル単位で指定，0.0 以上．またはディザ設定のテーブル．
---@param path_type path_type|nil 「線タイプ」(パスの種類) を指定．`nil` を指定した場合，折れ線への変換や点列のコピーを省略する．このとき `pts` は既に折れ線の前提で，テーブルの内容も書き換わる．
---@param pts any[] 点列の配列， `{ x1, y1, x2, y2, x3, y3, ... }` の形式．
---@param n_segs integer パスの分割区間の個数．1 以上．
---@param loop boolean 閉じたパスかどうか．
---@param prec number 「曲線精度」(折れ線の許容最長距離) を指定．
---@param start_pos number 「開始位置」を 0.0 から 1.0 に正規化した数値で指定．ただしループの場合はこの範囲を超えることもある．
---@param end_pos number 「終了位置」を 0.0 から 1.0 に正規化した数値で指定．ただしループの場合はこの範囲を超えることもある．
---@param end_shape end_shape 「端の形状」を指定．
---@param join_shape join_shape 「線結合の形状」を指定．
---@param miter_limit number 「マイター限界」を指定．1.0 以上の実数．
---@param dash_pat number[] 「破線パターン」を `{ opaque_len1, blank_len1, opaque_len2, blank_len2, ... }` の形式で指定．
---@param dash_pos number 「破線位置」をピクセル単位で指定．
---@param dash_adj boolean 「破線周期補正」を指定．
---@param dash_end_shape end_shape 破線の「端の形状」を指定．
---@param scale number 「拡大率」を指定．0 以上の実数．
---@param rotate number 「回転」を指定．
---@param dx number? 「移動X」を指定．拡縮回転の後に適用．省略時は 0.
---@param dy number? 「移動Y」を指定．拡縮回転の後に適用．省略時は 0.
---@param target_buffer { name: string, w: integer, h: integer }? マスク適用先のバッファ名 (e.g. "object", "cache:foo") とその幅と高さを指定．省略時は `{ name = "object", w = obj.w, h = obj.h }`.
---@param temp_buffer_name string? 頂点データを転送する先のバッファ名 (e.g. "tempbuffer", "cache:foo"). 省略時は "tempbuffer".
local function path_mask_line(
	alpha_outer, alpha_inner, line_width, antialias,
	path_type, pts, n_segs, loop, prec,
	start_pos, end_pos, end_shape, join_shape, miter_limit,
	dash_pat, dash_pos, dash_adj, dash_end_shape,
	scale, rotate, dx, dy,
	target_buffer, temp_buffer_name)
	-- normalize parameters.
	local tgt_name, tgt_w, tgt_h = "object", obj.w, obj.h;
	if target_buffer then
		tgt_name, tgt_w, tgt_h = target_buffer.name, target_buffer.w, target_buffer.h;
	end
	antialias = wrap_antialias(antialias);

	-- handle the trivial case.
	if alpha_outer == alpha_inner then mask_uniform(alpha_outer, tgt_name); return end

	-- make the curve into the sequence of secants.
	local points, num_points = pts, n_segs + 1;
	if path_type then
		points, num_points = poll(path_type, pts, n_segs, loop,
			prec / math.min(math.max(scale, 1.0 / 64), 1));
	end

	-- apply translation / scaling / rotation.
	transform(points, num_points, scale, rotate, dx, dy);
	local L, R, T, B, len = measure(points, num_points);
	local th =line_inflation(end_shape, dash_end_shape, join_shape,
		line_width, antialias.width, miter_limit, dash_pat);
	L, R, T, B = L - th, R + th, T - th, B + th;

	-- check if the path overlaps this object.
	if L >= tgt_w / 2 or R <= -tgt_w / 2 or T >= tgt_h / 2 or B <= -tgt_h / 2 then
		mask_uniform(alpha_outer, tgt_name);
		return;
	end

	-- send the coordinates to tempbuffer.
	temp_buffer_name = temp_buffer_name or "tempbuffer";
	send(points, num_points, tgt_w / 2, tgt_h / 2, temp_buffer_name);

	path_mask_line_buffered(
		alpha_outer, alpha_inner, line_width, antialias,
		temp_buffer_name, num_points, len, loop,
		start_pos, end_pos, end_shape, join_shape, miter_limit,
		dash_pat, dash_pos, dash_adj, dash_end_shape,
		target_buffer);
end

local push_context, pop_context do
	local key_name_stack, key_name_cxt do
		local g_key_stack, g_key_cxt = "path_s/cxt_man/cxt_stack#", "path_s/cxt_man/cxt#";
		function key_name_stack(id) return g_key_stack..id end
		function key_name_cxt(id, effect_id) return g_key_cxt..id.."&"..effect_id end
	end
	---後続フィルタに伝達する情報のテーブルを `global` に登録する．`obj.effect()` 経由で後続フィルタからも参照できる．
	---@param cxt table 登録情報のテーブル．
	function push_context(cxt)
		local g_key_stack = key_name_stack(obj.id);
		local stack do
			local str_stack = global[g_key_stack];
			if str_stack then
				stack = buffer.decode(str_stack);
				if type(stack) ~= "table" then stack = {} end
			else stack = {} end
		end
		stack[#stack + 1] = obj.effect_id;
		local g_key_cxt = key_name_cxt(obj.id, obj.effect_id);
		global[g_key_cxt] = buffer.encode(cxt);
		global[g_key_stack] = buffer.encode(stack);
	end
	---登録した後続フィルタに伝達する情報のテーブルを `global` から抜き出す．
	---@param effect_id integer? 抜き出す対象の `obj.effect_id`. 省略時は stack top のみを抜き出して，そのテーブルを返す．stack が空の場合は `nil` を返す．指定時はこの id とそれ以降を stack から抜き出して，指定 id のテーブルを返す．id が見つからない場合は `nil` を返す．
	---@return table? cxt 登録情報のテーブル，または nil．
	function pop_context(effect_id)
		local g_key_stack = key_name_stack(obj.id);
		local stack do
			local str_stack = global[g_key_stack];
			if str_stack then
				stack = buffer.decode(str_stack);
				if type(stack) ~= "table" then stack = {} end
			else stack = {} end
		end
		local idx = -1 if #stack > 0 then
			if effect_id then
				for i = #stack, 1, -1 do
					if stack[i] == effect_id then idx = i; break end
				end
			else idx = #stack end
		end
		local cxt = nil if idx >= 0 then
			for i = #stack, idx + 1, -1 do
				local g_key_cxt = key_name_cxt(obj.id, stack[i]);
				stack[i] = nil;
				global[g_key_cxt] = nil;
			end
			local g_key_cxt = key_name_cxt(obj.id, stack[idx]);
			stack[idx] = nil;
			local str_cxt = global[g_key_cxt]; global[g_key_cxt] = nil;
			if str_cxt then cxt = buffer.decode(str_cxt) end
		end
		global[g_key_stack] = #stack > 0 and buffer.encode(stack) or nil;

		-- check the restored object.
		if type(cxt) ~= "table" then return nil end
		return cxt;
	end
end

local partial_filter_check_cxt, partial_filter_make_cxt, partial_filter_combine do
	---@alias partial_filter_context { [1]: "part", [2]: integer, [3]: path_type, [4]: number[], [5]: number, [6]: mode_fill, [7]: number, [8]: dither_setting, [9]: boolean, [10]: number, [11]: number, [12]: number, [13]: number, [14]: integer, [15]: integer, [16]: number, [17]: number, [18]: string } # パス部分フィルタσ で後続フィルタに伝達できる形でパスやフィルタ元の状態の情報を保持するテーブル．
	---与えられたテーブルが `partial_filter_context` であるかどうかを確認する．
	---@param t table? # 確認対象のテーブル値．
	---@return partial_filter_context? # `partial_filter_context` である場合 `t` と同じ値，そうでないなら `nil`.
	function partial_filter_check_cxt(t)
		if not t then return nil end

		-- header
		if t[1] ~= "part" then return nil end
		-- num_points
		if type(t[2]) ~= "number" or t[2] % 1 > 0 or t[2] < 3 then return nil end
		-- path_type
		if type(t[3]) ~= "number" or t[3] ~= PI_choose_path_type(nil, t[3]) then return nil end
		-- points
		if type(t[4]) ~= "table" then return nil end
		-- precision
		if type(t[5]) ~= "number" or t[5] < 1 then return nil end
		-- mode_fill
		if type(t[6]) ~= "number" or t[6] ~= PI_choose_mode_fill(nil, t[6]) then return nil end
		-- inflation
		if type(t[7]) ~= "number" or t[7] < 0 then return nil end
		-- antialias
		if type(t[8]) ~= "table" then return nil;
		else
			local width, pattern, seed, rate, cx, cy, size = t[8].width, t[8].pattern, t[8].seed, t[8].rate, t[8].cx, t[8].cy, t[8].size;
			if type(width) ~= "number" or type(pattern) ~= "number"  or type(rate) ~= "number" or type(seed) ~= "number"
				or type(cx) ~= "number" or type(cy) ~= "number" or type(size) ~= "number"then return nil;
			elseif width < 0 then return nil;
			elseif rate < 0 or rate > 1 then return nil;
			elseif size < 1 then return nil end
		end
		-- invert
		if type(t[9]) ~= "boolean" then return nil end
		-- X
		if type(t[10]) ~= "number" then return nil end
		-- Y
		if type(t[11]) ~= "number" then return nil end
		-- zoom
		if type(t[12]) ~= "number" or t[12] <= 0 then return nil end
		-- rotate
		if type(t[13]) ~= "number" then return nil end
		-- obj.w
		if type(t[14]) ~= "number" or t[14] % 1 > 0 or t[14] <= 0 then return nil end
		-- obj.h
		if type(t[15]) ~= "number" or t[15] % 1 > 0 or t[15] <= 0 then return nil end
		-- obj.cx
		if type(t[16]) ~= "number" then return nil end
		-- obj.cy
		if type(t[17]) ~= "number" then return nil end
		-- cache_name
		if type(t[18]) ~= "string" or not t[18]:find("^cache:.") then return nil end
		return t;
	end
	---パス部分フィルタσ で後続フィルタに伝達する情報を登録し，登録情報のテーブルを作成する．`partial_filter_push_cxt()` や `partial_filter_combine()` で利用する．
	---@param num_points integer # 頂点数．
	---@param path_type path_type # 線タイプ．
	---@param points number[] # 点リスト．
	---@param precision number # 曲線精度．
	---@param mode_fill mode_fill # 範囲．
	---@param inflation number # 追加幅．
	---@param antialias antialias # ぼかし幅．
	---@param invert boolean # 反転．
	---@param X number # 移動X．
	---@param Y number # 移動Y．
	---@param zoom number # 拡大率．1.0 で等倍．
	---@param rotate number # 回転．ラジアン単位．
	---@return partial_filter_context # 登録情報のテーブル．
	function partial_filter_make_cxt(
		num_points, path_type, points, precision,
		mode_fill, inflation, antialias, invert,
		X, Y, zoom, rotate)
		local cache_name = "cache:path_s/part/ori#"..obj.effect_id;
		assert(obj.copybuffer(cache_name, "object"));
		return {
			"part",
			num_points, path_type, points, precision,
			mode_fill, inflation, wrap_antialias(antialias), invert,
			X, Y, zoom, rotate,
			obj.w, obj.h, obj.cx, obj.cy, cache_name,
		};
	end
	---登録情報のテーブルをもとにフィルタ加工前と加工後の 2 つの画像を合成する．
	---@param cxt partial_filter_context 登録情報のテーブル．
	function partial_filter_combine(cxt)
		local num_points, path_type, points, precision,
			mode_fill, inflation, antialias, invert,
			X, Y, zoom, rotate,
			w0, h0, cx0, cy0, cache_name =
			cxt[2], cxt[3], cxt[4], cxt[5],
			cxt[6], cxt[7], cxt[8], cxt[9],
			cxt[10], cxt[11], cxt[12], cxt[13],
			cxt[14], cxt[15], cxt[16], cxt[17], cxt[18];

		-- adjust the size and center.
		local w1, h1, cx1, cy1 = obj.w, obj.h, obj.cx, obj.cy;
		local w, h, cx, cy do
			local L, R, T, B =
				math.min(-w0 / 2 - cx0, -w1 / 2 - cx1),
				math.max(w0 / 2 - cx0, w1 / 2 - cx1),
				math.min(-h0 / 2 - cy0, -h1 / 2 - cy1),
				math.max(h0 / 2 - cy0, h1 / 2 - cy1);
			w, h = math.ceil(R - L), math.ceil(B - T);
			cx, cy = -(2 * L + w) / 2, -(2 * T + h) / 2;
		end

		-- create the shape of the path.
		local cache_name_mask, cache_name_tmp =
			"cache:path_s/part/mask", "cache:path_s/part/tmp";
		obj.clearbuffer(cache_name_mask, w, h, 0x000000);
		obj.cx, obj.cy = cx, cy;
		antialias.cx, antialias.cy = antialias.cx + (w - w0) / 2 + (cx - cx0), antialias.cy + (h - h0) / 2 + (cy - cy0);
		path_mask_area(
			invert and 1 or 0, invert and 0 or 1, mode_fill, inflation, antialias,
			path_type, points, num_points, precision,
			zoom, rotate, X + (cx - cx0), Y + (cy - cy0),
			{ name = cache_name_mask, w = w, h = h }, cache_name_tmp);

		-- interpolate the original and effected buffers by that shape.
		obj.clearbuffer(cache_name_tmp, w, h);
		obj.pixelshader("interpolate@パス部分フィルタσ@Path_S", cache_name_tmp, { cache_name, "object", cache_name_mask }, {
			-(w - w0) / 2 - (cx - cx0), -(h - h0) / 2 - (cy - cy0);
			-(w - w1) / 2 - (cx - cx1), -(h - h1) / 2 - (cy - cy1);
		});
		assert(obj.copybuffer("object", cache_name_tmp));
	end
end

local post_effect_check_cxt, post_effect_make_cxt,post_effect_combine do
	---@alias post_effect_context { [1]: "eff", [2]: 0|1, [3]: blend_mode, [4]: number, [5]: number, [6]: integer, [7]: integer, [8]: number, [9]: number, [10]: string } # 後続フィルタに伝達できる形で処理の途中状態を保持するテーブル．
	---与えられたテーブルが `post_effect_context` であるかどうかを確認する．
	---@param t table? # 確認対象のテーブル値．
	---@return post_effect_context? # `post_effect_context` である場合 `t` と同じ値，そうでないなら `nil`.
	function post_effect_check_cxt(t)
		if not t then return nil end

		-- header
		if t[1] ~= "eff" then return nil end
		-- mode_draw
		if type(t[2]) ~= "number" or t[2] % 1 > 0 or t[2] < 0 or t[2] > 1 then return nil end
		-- blend_mode
		if type(t[3]) ~= "string" then return nil end
		-- orig_alpha
		if type(t[4]) ~= "number" or t[4] < 0 or t[4] > 1 then return nil end
		-- ext_alpha
		if type(t[5]) ~= "number" or t[5] < 0 or t[5] > 1 then return nil end
		-- w0
		if type(t[6]) ~= "number" or t[6] % 1 > 0 or t[6] < 1 or t[6] >= 2 ^ 14 then return nil end
		-- h0
		if type(t[7]) ~= "number" or t[7] % 1 > 0 or t[7] < 1 or t[7] >= 2 ^ 14 then return nil end
		-- cx0
		if type(t[8]) ~= "number" then return nil end
		-- cy0
		if type(t[9]) ~= "number" then return nil end
		-- cache_name
		if type(t[10]) ~= "string" or not t[10]:find("^cache:.") then return nil end
		return t;
	end
	---輪郭パスσ などで後続フィルタに伝達する情報を登録し，登録情報のテーブルを作成する．`post_effect_push_cxt()` や `post_effect_combine()` で利用する．
	---@param mode_draw 0|1 # モード (前方から合成 or 後方から合成).
	---@param blend_mode blend_mode # 合成モードの名前．
	---@param orig_alpha number # 元画像のアルファ値．
	---@param ext_alpha number # 合成画像のアルファ値．
	---@param w0 integer # 元画像の幅．
	---@param h0 integer # 元画像の高さ．
	---@param cx0 number # 元画像の回転中心の X 座標．
	---@param cy0 number # 元画像の回転中心の Y 座標．
	---@param cache_name string # 元画像のキャッシュ名．
	---@return post_effect_context
	function post_effect_make_cxt(
		mode_draw, blend_mode, orig_alpha, ext_alpha,
		w0, h0, cx0, cy0, cache_name)
		return {
			"eff",
			mode_draw, blend_mode, orig_alpha, ext_alpha,
			w0, h0, cx0, cy0, cache_name,
		};
	end
	---登録情報のテーブルをもとにフィルタ加工後と元画像の 2 つを合成する．
	---@param cxt post_effect_context 登録情報のテーブル．
	function post_effect_combine(cxt)
		local mode_draw, blend_mode, orig_alpha, ext_alpha,
			w0, h0, cx0, cy0, cache_name =
			cxt[2], cxt[3], cxt[4], cxt[5],
			cxt[6], cxt[7], cxt[8], cxt[9], cxt[10];
		local cx1, cy1, L, R, T, B = obj.cx, obj.cy,
			math.floor(math.min(cx0 - obj.cx + (w0 - obj.w) / 2, 0)),
			math.ceil(math.max(cx0 - obj.cx + (w0 + obj.w) / 2, w0)),
			math.floor(math.min(cy0 - obj.cy + (h0 - obj.h) / 2, 0)),
			math.ceil(math.max(cy0 - obj.cy + (h0 + obj.h) / 2, h0));
		local W, H = R - L, B - T;
		obj.cx, obj.cy = cx0 - (L + R - w0) / 2, cy0 - (T + B - h0) / 2;

		local prev_dst = obj.getoption("drawtarget");
		if mode_draw == 0 then
			local line_cache = "cache:path_s/track/line";
			assert(obj.copybuffer(line_cache, "object"));
			if W > w0 or H > h0 or obj.cx ~= cx0 or obj.cy ~= cy0 or orig_alpha < 1 then
				assert(obj.copybuffer("object", cache_name));
				obj.setoption("drawtarget", "tempbuffer", W, H);
				obj.draw(obj.cx - cx0, obj.cy - cy0, 0, 1, orig_alpha);
			else
				assert(obj.copybuffer("tempbuffer", cache_name));
				obj.setoption("drawtarget", "tempbuffer");
			end
			assert(obj.copybuffer("object", line_cache));
			local prev_blend = obj.getoption("blend");
			obj.setoption("blend", blend_mode);
			obj.draw(obj.cx - cx1, obj.cy - cy1, 0, 1, ext_alpha);
			obj.setoption("blend", prev_blend);
		else
			if W > obj.w or H > obj.h or obj.cx ~= cx1 or obj.cy ~= cy1 or ext_alpha < 1 then
				obj.setoption("drawtarget", "tempbuffer", W, H);
				obj.draw(obj.cx - cx1, obj.cy - cy1, 0, 1, ext_alpha);
			else
				assert(obj.copybuffer("tempbuffer", "object"));
				obj.setoption("drawtarget", "tempbuffer");
			end
			assert(obj.copybuffer("object", cache_name));
			local prev_blend = obj.getoption("blend");
			obj.setoption("blend", blend_mode);
			obj.draw(obj.cx - cx0, obj.cy - cy0, 0, 1, orig_alpha);
			obj.setoption("blend", prev_blend);
		end
		obj.setoption("drawtarget", prev_dst);
		assert(obj.copybuffer("object", "tempbuffer"));
	end
end

---Lua のエラーメッセージを，AviUtl2 が標準で出力する形式を真似て出力する．
---@param err_mes string Lua からのエラーメッセージ．
---@param source string エラー元となった Lua スクリプトのソースコード．
local function print_script_error(err_mes, source)
	local n, err_desc = err_mes:match("%]:(%d+):%s(.-)$");
	n = tonumber(n);
	if n and err_desc then
		-- collect three lines containing the one that caused the error.
		n = math.max(n - 1, 1);
		local k = 0;
		for l in (source.."\n"):gmatch("(.-)\n") do
			k = k + 1;
			if k >= n then
				err_desc = err_desc.."\n> "..l;
				if k >= n + 2 then break end
			end
		end
	else err_desc = err_mes end
	print(err_desc); -- easy-to-read message.
	print("@warn", err_mes); -- raw message.
end

local find_boundaries, find_all_boundaries do
	local tau, math_min, math_max, math_ceil, math_atan, math_atan2 = 2 * math.pi, math.min, math.max, math.ceil, math.atan, math.atan2;
	local bit_rshift, bit_band, bit_bor = bit.rshift, bit.band, bit.bor;
	local ptr_uint32, arr_uint8 = ffi.typeof("uint32_t*"), ffi.typeof("uint8_t[?]");

	-- `dir` shall mean as, for each opaque pixel,
	-- a boundary path lies on and goes to the direction as the followings:
	--       (+1,0)→
	--   (0,-1)↑□↓(0,+1)
	--       (-1,0)←
	-- □ represents a focused opaque pixel,
	-- arrows represent sections of the path weaving between pixels,
	-- numbers are the values for the variables `dx` and `dy`.

	local function is_opaque(x, y, buf, w, h, thresh)
		return 0 <= x and x < w and 0 <= y and y < h and buf[x + y * w] >= thresh;
	end

	-- boundary-tracking function.
	local function advance(X, Y, dx, dy, buf, w, h, thresh, conn_corner)
		local x1, y1, x2, y2 = X + dx, Y + dy, X + (dy + dx), Y + (dy - dx);
		local X1, Y1, X3, Y3 = X, Y, x2, y2;
		local dx1, dy1, dx3, dy3 = -dy, dx, dy, -dx;
		if conn_corner then
			x1, y1, x2, y2 = x2, y2, x1, y1;
			X1, Y1, X3, Y3 = X3, Y3, X1, Y1;
			dx1, dy1, dx3, dy3 = dx3, dy3, dx1, dy1;
		end

		if is_opaque(x1, y1, buf, w, h, thresh) == conn_corner then return X1, Y1, dx1, dy1 end
		if is_opaque(x2, y2, buf, w, h, thresh) == conn_corner then return X + dx, Y + dy, dx, dy end
		return X3, Y3, dx3, dy3;
	end

	local function pt_cand_anchor(X, Y, dx, dy, buf, w, h, thresh)
		local x, y = X + dy, Y - dx;
		local a0, a1 = bit_rshift(buf[X + Y * w], 24), 0;
		if 0 <= x and x < w and 0 <= y and y < h then
			a1 = bit_rshift(buf[x + y * w], 24) end

		local t = (thresh - a1) / (a0 - a1);
		return t * X + (1 - t) * x + 0.5, t * Y + (1 - t) * y + 0.5;
	end

	---指定したバッファの指定した点に近い境界を囲うパスを生成する．点は複数個指定できる．
	---@param target string # バッファの名前．
	---@param points integer[] # 点の配列を，`{ x1, y1, x2, y2, ... }` の形式で指定．
	---@param num_points integer # `points` に含まれる点の個数．
	---@param thresh number # 境界を決定するアルファ値のしきい値．0.0 -- 1.0.
	---@param conn_corner boolean # 角で隣接するかどうか．
	---@param prec number # パスのラインの許容ぶれ幅．ピクセル単位．
	---@return { points: number[], num_segments: integer }[] # 生成されたパスの配列．ループ端点は末尾に含まれている．
	function find_boundaries(target, points, num_points, thresh, conn_corner, prec)
		local th, th_i =
			math_min(math_max(255 * thresh, 0.125), 255),
			math_min(math_max(math_ceil(255 * thresh), 1), 255) * 2 ^ 24;
		local d, w, h = obj.getpixeldata(target);
		local buf = ptr_uint32(d);

		local ret, n_paths = {}, 0;
		for i = 1, num_points do
			local X, Y = points[2 * i - 1], points[2 * i];

			-- the first pixel must be opaque, and advance left/up/right/down until the boundary.
			if is_opaque(X, Y, buf, w, h, th_i) then
				local dx0, dy0 = 0, -1;
				for j = 1, math_max(w, h) do
					if not is_opaque(X - j, Y, buf, w, h, th_i) then
						X, dx0, dy0 = X - j + 1, 0, -1;
						break;
					end
					if not is_opaque(X, Y - j, buf, w, h, th_i) then
						Y, dx0, dy0 = Y - j + 1, 1, 0;
						break;
					end
					if not is_opaque(X + j, Y, buf, w, h, th_i) then
						X, dx0, dy0 = X + j - 1, 0, 1;
						break;
					end
					if not is_opaque(X, Y + j, buf, w, h, th_i) then
						Y, dx0, dy0 = Y + j - 1, -1, 0;
						break;
					end
				end

				-- track the boundary until it closes.
				local ax, ay = pt_cand_anchor(X, Y, dx0, dy0, buf, w, h, th);
				local bx, by, t_min, t_max, path, n = ax, ay, -tau, tau, { ax, ay }, 1;
				local x, y, dx, dy = advance(X, Y, dx0, dy0, buf, w, h, th_i, conn_corner);
				while x ~= X or y ~= Y or dx ~= dx0 or dy ~= dy0 do
					local cx, cy = pt_cand_anchor(x, y, dx, dy, buf, w, h, th);

					-- update the valid range of angle.
					local t, dt =
						math_atan2(cy - ay, cx - ax),
						math_atan(0.5 * prec / ((cx - ax) ^ 2 + (cy - ay) ^ 2) ^ 0.5);
					if t_min > t + dt then t_min, t_max = t_min - tau, t_max - tau;
					elseif t_max < t - dt then t_min, t_max = t_min + tau, t_max + tau end
					t_min, t_max = math_max(t_min, t - dt), math_min(t_max, t + dt);

					if t_min > t_max then
						-- the angle will be invalid if the previous anchor is skipped.
						path[2 * n + 1], path[2 * n + 2] = bx, by;
						n = n + 1;
						t, dt =
							math_atan2(cy - by, cx - bx),
							math_atan(prec / ((cx - bx) ^ 2 + (cy - by) ^ 2) ^ 0.5);
						ax, ay, t_min, t_max = bx, by, t - dt, t + dt;
					elseif t_min >= tau / 2 then t_min, t_max = t_min - tau, t_max - tau;
					elseif t_max <= -tau / 2 then t_min, t_max = t_min - tau, t_max - tau end

					bx, by = cx, cy;
					x, y, dx, dy = advance(x, y, dx, dy, buf, w, h, th_i, conn_corner);
				end

				-- append the loop end point to the path.
				path[2 * n + 1], path[2 * n + 2] = path[1], path[2];

				-- append the path.
				ret[n_paths + 1] = { points = path, num_segments = n };
				n_paths = n_paths + 1;
			end
		end

		return ret;
	end

	---指定したバッファの境界を囲うパスを，バッファ全体から探して生成する．
	---@param target string # バッファの名前．
	---@param thresh number # 境界を決定するアルファ値のしきい値．0.0 -- 1.0.
	---@param conn_corner boolean # 角で隣接するかどうか．
	---@param prec number # パスのラインの許容ぶれ幅．ピクセル単位．
	---@return { points: number[], num_segments: integer}[] # 生成されたパスの配列．ループ端点は末尾に含まれている．
	function find_all_boundaries(target, thresh, conn_corner, prec)
		local th, th_i =
			math_min(math_max(255 * thresh, 0.125), 255),
			math_min(math_max(math_ceil(255 * thresh), 1), 255) * 2 ^ 24;
		local d, w, h = obj.getpixeldata(target);
		local buf, marks = ptr_uint32(d), arr_uint8(w * h);

		local ret, n_paths = {}, 0;
		for j = 0, h - 1 do
			local prev_opaque, prev_mark = false, 0;
			for i = 0, w - 1 do
				local curr_opaque, curr_mark =
					is_opaque(i, j, buf, w, h, th_i), marks[i + w * j];
				if prev_opaque ~= curr_opaque and
					(prev_opaque and bit_band(prev_mark, 2) or bit_band(curr_mark, 1)) == 0 then
					local X, Y, dx0, dy0 = i, j, 0, -1;
					if prev_opaque then X, dy0 = i - 1, 1 end
					marks[X + w * Y] = bit_bor(marks[X + w * Y], dy0 > 0 and 2 or 1);

					-- track the boundary until it closes.
					local ax, ay = pt_cand_anchor(X, Y, dx0, dy0, buf, w, h, th);
					local bx, by, t_min, t_max, path, n = ax, ay, -tau, tau, { ax, ay }, 1;
					local x, y, dx, dy = advance(X, Y, dx0, dy0, buf, w, h, th_i, conn_corner);
					while x ~= X or y ~= Y or dx ~= dx0 or dy ~= dy0 do
						if dx == 0 then
							marks[x + w * y] = bit_bor(marks[x + w * y], dy > 0 and 2 or 1);
						end
						local cx, cy = pt_cand_anchor(x, y, dx, dy, buf, w, h, th);

						-- update the valid range of angle.
						local t, dt =
							math_atan2(cy - ay, cx - ax),
							math_atan(0.5 * prec / ((cx - ax) ^ 2 + (cy - ay) ^ 2) ^ 0.5);
						if t_min > t + dt then t_min, t_max = t_min - tau, t_max - tau;
						elseif t_max < t - dt then t_min, t_max = t_min + tau, t_max + tau end
						t_min, t_max = math_max(t_min, t - dt), math_min(t_max, t + dt);

						if t_min > t_max then
							-- the angle will be invalid if the previous anchor is skipped.
							path[2 * n + 1], path[2 * n + 2] = bx, by;
							n = n + 1;
							t, dt =
								math_atan2(cy - by, cx - bx),
								math_atan(prec / ((cx - bx) ^ 2 + (cy - by) ^ 2) ^ 0.5);
							ax, ay, t_min, t_max = bx, by, t - dt, t + dt;
						elseif t_min >= tau / 2 then t_min, t_max = t_min - tau, t_max - tau;
						elseif t_max <= -tau / 2 then t_min, t_max = t_min - tau, t_max - tau end

						bx, by = cx, cy;
						x, y, dx, dy = advance(x, y, dx, dy, buf, w, h, th_i, conn_corner);
					end

					-- append the loop end point to the path.
					path[2 * n + 1], path[2 * n + 2] = path[1], path[2];

					-- append the path.
					ret[n_paths + 1] = { points = path, num_segments = n };
					n_paths = n_paths + 1;
					curr_mark = marks[i + w * j]; -- update the marker.
				end
				prev_opaque, prev_mark = curr_opaque, curr_mark;
			end
		end

		return ret;
	end
end

-- return the table containing the exported functions.
return {
	PI = {
		as_bool = PI_as_bool,
		blend_mode = PI_choose_blend_mode,
		path_type = PI_choose_path_type,
		mode_anchor_base = PI_choose_mode_anchor_base,
		mode_fill = PI_choose_mode_fill,
		end_shape = PI_choose_end_shape,
		join_shape = PI_choose_join_shape,
		dither_pattern = PI_choose_dither_pattern,
	},

	snap_anchors = snap_anchors,
	anchor_offset = anchor_offset,
	anchor = anchor,
	poll = poll,
	measure = measure,
	rotation_count = rotation_count,
	line_inflation = line_inflation,
	find_index = find_index,
	transform = transform,
	randomize = randomize,
	send = send,
	retrieve = retrieve,

	path_mask_area_buffered = path_mask_area_buffered,
	path_mask_area = path_mask_area,
	path_mask_line_buffered = path_mask_line_buffered,
	path_mask_line = path_mask_line,

	context_manager = {
		push = push_context,
		pop = pop_context,
	},
	partial_filter = {
		check_cxt = partial_filter_check_cxt,
		make_cxt = partial_filter_make_cxt,
		combine = partial_filter_combine,
	},

	boundary = {
		find = find_boundaries,
		find_all = find_all_boundaries,
	},

	post_effect = {
		check_cxt = post_effect_check_cxt,
		make_cxt = post_effect_make_cxt,
		combine = post_effect_combine,
	},

	print_script_error = print_script_error,

	VERSION = "v2.42",
};

