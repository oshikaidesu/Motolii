--[[
MIT License

Copyright (c) 2026 sigma-axis

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
-- v1.10
--

-- https://github.com/sigma-axis/aviutl2_script_Region_S

local obj, print, type, tonumber, unpack, loadstring, pcall, bit, ffi = obj, print, type, tonumber, unpack, loadstring, pcall, require("bit"), require("ffi");
local math_abs, math_floor, math_ceil, math_min, math_max = math.abs, math.floor, math.ceil, math.min, math.max;
local bit_band, bit_bor = bit.band, bit.bor;
local ptr_int32_t = ffi.typeof("int32_t*");

if obj.getinfo("version") < tonumber("2010800") then
	error([[AviUtl ExEdit2 version 2.1.8 以降が必要です！]], 2);
end

-- `dir` shall mean as, for each opaque pixel,
-- a boundary path lies on and goes to the direction as the followings:
--       (+1,0)→
--   (0,-1)↑□↓(0,+1)
--       (-1,0)←
-- □ represents a focused opaque pixel,
-- arrows represent sections of the path weaving between pixels,
-- numbers are the values for the variables `dx` and `dy`.

local function is_opaque(x, y, buf, w, h)
	-- assuming the top 8 bits, which is the alpha channel, is either 0x00 or 0xff.
	return 0 <= x and x < w and 0 <= y and y < h and buf[x + y * w] < 0;
end

-- boundary-tracking function.
local function advance(X, Y, dx, dy, buf, w, h, conn_corner)
	local x1, y1, x2, y2 = X + dx, Y + dy, X + (dy + dx), Y + (dy - dx);
	local X1, Y1, X3, Y3 = X, Y, x2, y2;
	local dx1, dy1, dx3, dy3 = -dy, dx, dy, -dx;
	if conn_corner then
		x1, y1, x2, y2 = x2, y2, x1, y1;
		X1, Y1, X3, Y3 = X3, Y3, X1, Y1;
		dx1, dy1, dx3, dy3 = dx3, dy3, dx1, dy1;
	end

	if is_opaque(x1, y1, buf, w, h) == conn_corner then return X1, Y1, dx1, dy1 end
	if is_opaque(x2, y2, buf, w, h) == conn_corner then return X + dx, Y + dy, dx, dy end
	return X3, Y3, dx3, dy3;
end

-- `path flags` is a marker that indicates whether the path is already tracked,
-- and if so, it is a left side and/or right side of the path.
-- it's coded in the lowest two bits of each pixel as follows:
--   0: unmarked.
--   1: marked that the pixel is the left side of the path.
--   2: marked that the pixel is the right side of the path.
-- they are combined with the bitwise-or operation.
-- note that top and bottom bounds are not marked,
-- as the use of these flags is limited only to left and right sides.

-- closes the path with the given starting point.
local function close_path(X0, Y0, dx0, dy0, buf, w, h, conn_corner)
	-- follow the path until it closes.
	local X, Y, dx, dy = X0, Y0, dx0, dy0;
	local Lx, Ly, Rx, Ry, Tx, Ty, Bx, By = X, Y, X, Y, X, Y, X, Y;
	local cycle = 0;
	repeat
		X, Y, dx, dy = advance(X, Y, dx, dy, buf, w, h, conn_corner);

		-- mark the path flag.
		if dx == 0 then buf[X + Y * w] = bit_bor(buf[X + Y * w], dy > 0 and 2 or 1) end

		-- update min/max.
		if X < Lx then Lx, Ly = X, Y end
		if X > Rx then Rx, Ry = X, Y end
		if Y < Ty then Tx, Ty = X, Y end
		if Y > By then Bx, By = X, Y end

		-- update cycle.
		if dx == 0 and Y == Y0 then
			cycle = cycle + (X == X0 and 1 or X < X0 and -dy or dy);
		end
	until X == X0 and Y == Y0 and dx == dx0 and dy == dy0;

	return Lx, Ly, Rx, Ry, Tx, Ty, Bx, By, cycle > 0;
end

-- searches the outer boundary that contains the given point.
-- returns the bounding box of the boundary, or an empty box if the boundary was already marked.
-- assumes the given point is on the opaque side, otherwise nothing is marked and returns an empty box.
local function search_outer_boundary(X0, Y0, buf, w, h, conn_corner)
	if not is_opaque(X0, Y0, buf, w, h) then return 0, -1, 0, -1 end

	local X, Y = X0, Y0;
	while true do
		-- find a point on a path nearby.
		while X + 1 < w and buf[X + 1 + Y * w] < 0 do X = X + 1 end

		-- when a boundary is already marked, so is the outer boundary. exit the loop.
		if bit_band(buf[X + Y * w], 2) ~= 0 then return 0, -1, 0, -1 end

		-- a boundary is found. close that path.
		local Lx, _, Rx, Ry, _, Ty, _, By, c = close_path(X, Y, 0, 1, buf, w, h, conn_corner);
		if c then
			-- that path contsins (X, Y) inside.
			return Lx, Rx, Ty, By;
		end

		-- skip to the right-most of the path.
		X, Y = Rx, Ry;
	end
end

---erases alpha values of all pixels outside the connected components specified by `points`. then returns the bounding box of those components.
---@param buff_src string the buffer name to get pixel data from. all pixels are assumed to be either of the form `float4(0, 0, 0, 0)` or `float4(0, 0, 0, 1)`.
---@param buff_dst string the buffer name to put pixel data onto. may be same as `buff_src`.
---@param conn_corner boolean specifies if two adjacent corners are recognized as connected.
---@param union_left boolean specifies if all left-most pixels belong to the virtual "outer component".
---@param union_right boolean specifies if all right-most pixels belong to the virtual "outer component".
---@param union_top boolean specifies if all top-most pixels belong to the virtual "outer component".
---@param union_bottom boolean specifies if all bottom-most pixels belong to the virtual "outer component".
---@param num_points integer the number of points in the array `points`.
---@param points number[] an array of points of the form `{ x1, y1, x2, y2, ... }` that the extracted components contain, in anchor coordinate (origin is at the center of the image).
---@return integer L, integer R, integer T, integer B the bounding box of all components. `L` and `T` are inclusive, while `R` and `B` are exclusive.
local function extract_components(buff_src, buff_dst, conn_corner, union_left, union_right, union_top, union_bottom, num_points, points)
	if num_points <= 0 or #points < 2 then return -1, -1, -1, -1 end
	local data, w, h = obj.getpixeldata(buff_src);
	local buf = ptr_int32_t(data);

	local L, R, T, B, outer = w, -1, h, -1, false;
	for i = 1, num_points do
		local X, Y = tonumber(points[2 * i - 1]), tonumber(points[2 * i]);
		if X and Y then
			X, Y = math_floor(w / 2 + X), math_floor(h / 2 + Y);
			if 0 <= X and X < w and 0 <= Y and Y < h then
				local l, r, t, b = search_outer_boundary(X, Y, buf, w, h, conn_corner);

				if l <= r and t <= b then
					L, R, T, B = math_min(L, l), math_max(R, r), math_min(T, t), math_max(B, b);
				end
			else outer = true end
		end
	end

	-- include all pixels on the specified edges.
	if outer or
		(union_left and L == 0) or (union_right and R == w - 1) or
		(union_top and T == 0) or (union_bottom and B == h - 1) then
		if union_left then
			for y = 0, h - 1 do -- include all left-most pixels.
				local v = buf[y * w];
				if v < 0 and bit_band(v, 1) == 0 then
					local l, _, r, _, _, t, _, b, _ = close_path(0, y, 0, -1, buf, w, h, conn_corner);
					L, R, T, B = math_min(L, l), math_max(R, r), math_min(T, t), math_max(B, b);
				end
			end
		end
		if union_right then
			for y = 0, h - 1 do -- include all right-most pixels.
				local v = buf[w - 1 + y * w];
				if v < 0 and bit_band(v, 2) == 0 then
					local l, _, r, _, _, t, _, b, _ = close_path(w - 1, y, 0, 1, buf, w, h, conn_corner);
					L, R, T, B = math_min(L, l), math_max(R, r), math_min(T, t), math_max(B, b);
				end
			end
		end
		if union_top then
			local was_in = false;
			for x = 0, w - 1 do -- include all top-most pixels.
				local v = buf[x];
				if not was_in and v < 0 and bit_band(v, 1) == 0 then
					local l, _, r, _, _, t, _, b, _ = close_path(x, 0, 0, -1, buf, w, h, conn_corner);
					L, R, T, B = math_min(L, l), math_max(R, r), math_min(T, t), math_max(B, b);
				end
				was_in = v < 0;
			end
		end
		if union_bottom then
			local was_in = false;
			for x = 0, w - 1 do -- include all bottom-most pixels.
				local v = buf[x + (h - 1) * w];
				if not was_in and v < 0 and bit_band(v, 2) == 0 then
					local l, _, r, _, _, t, _, b, _ = close_path(x, h - 1, 0, -1, buf, w, h, conn_corner);
					L, R, T, B = math_min(L, l), math_max(R, r), math_min(T, t), math_max(B, b);
				end
				was_in = v < 0;
			end
		end
	end
	if L > R or T > B then return -1, -1, -1, -1 end

	-- erase pixels that cannot be reached from the given points.
	for y = T, B do
		local is_in = false;
		for x = L, R do
			local v = buf[x + y * w];
			if v < 0 then
				if not is_in and bit_band(v, 1) == 0 then buf[x + y * w] = 0;
				else is_in = bit_band(v, 2) == 0 end
			elseif is_in then
				-- new path found.
				close_path(x - 1, y, 0, 1, buf, w, h, conn_corner);
				is_in = false;
			end
		end
	end

	-- send back the pixel data.
	obj.putpixeldata(buff_dst, data, w, h);

	-- then return the bounding box.
	return L, R + 1, T, B + 1;
end

---Lua のエラーメッセージを，AviUtl2 が標準で出力する形式を真似て出力する．
---@param err_mes string Lua からのエラーメッセージ．
---@param source string エラー元となった Lua スクリプトのソースコード．
local function print_script_error(err_mes, source)
	local n, err_desc = err_mes:match("%]:(%d+):%s(.-)$");
	n = tonumber(n);
	if n and err_desc then
		-- collect three lines containing the one that caused the error.
		n = math_max(n - 1, 1);
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

---wrapper function to execute script from string.
---@param script string the body of the script to execute.
---@param header string? script piece to be added at the head of `script` (optional).
---@param ... any arguments to pass to the script.
local function execute_script(script, header, ...)
	if not script:find("%S") then return end

	local f, c, e;
	f, e = loadstring((header or "")..script, script);
	if f then c, e = pcall(f, ...) end
	if not (f and c) then
		print_script_error(tostring(e), script);
		obj.setoption("draw_state", true);
		return;
	end
end

---apply clipping using the left, right, top and bottom coordinates.
---@param L integer left coordinate, 0 at the left edge of the image.
---@param R integer right coordinate, 0 at the left edge of the image.
---@param T integer top coordinate, 0 at the top edge of the image.
---@param B integer bottom coordinate, 0 at the top edge of the image.
---@param move_center boolean passed to the clipping effect whether to move the center.
local function crop_image(L, R, T, B, move_center)
	local w, h = obj.w, obj.h;
	local max_clip = 4000;
	if L >= R or T >= B then
		if h > max_clip then obj.clearbuffer("object", 1, 1) end
		w, h, L, R, T, B = 1, 1, 0, 1, 0, 0;
	end
	R, B = w - R, h - B;
	while L > 0 or R > 0 or T > 0 or B > 0 do
		local l, r, t, b = math_min(max_clip, L), math_min(max_clip, R), math_min(max_clip, T), math_min(max_clip, B);
		obj.effect("クリッピング", "左", l, "右", r, "上", t, "下", b, "中心の位置を変更", move_center and 1 or 0);
		L, R, T, B = L - l, R - r, T - t, B - b;
	end
end

local function PI_as_bool(pi_value, gui_value)
	if type(pi_value) == "boolean" then return pi_value;
	elseif type(pi_value) == "number" then return pi_value ~= 0;
	else return gui_value end
end

local PI_choose_blend_mode do
	local blend_name2num, blend_num2codename = {
		["通常"] = 0, ["加算"] = 1, ["減算"] = 2, ["乗算"] = 3, ["スクリーン"] = 4, ["オーバーレイ"] = 5,
		["比較(明)"] = 6, ["比較(暗)"] = 7, ["輝度"] = 8, ["色差"] = 9,
		["陰影"] = 10, ["明暗"] = 11, ["差分"] = 12,
		["alpha_add"] = 100, ["alpha_max"] = 101, ["alpha_sub"] = 102, ["alpha_add2"] = 103, ["rgba_add"] = 104,
	}, {
		[0] = "none", "add", "sub", "mul", "screen", "overlay",
		"light", "dark", "brightness", "chroma", "shadow", "light_dark", "diff",
		[100] = "alpha_add", [101] = "alpha_max", [102] = "alpha_sub", [103] = "alpha_add2", [104] = "rgba_add",
	};
	function PI_choose_blend_mode(pi_value, gui_value)
		if type(pi_value) == "string" then
			gui_value = blend_name2num[pi_value] or gui_value;
		end
		return blend_num2codename[gui_value] or "none";
	end
end

local function save_obj_props()
	return { obj.ox, obj.oy, obj.oz, obj.cx, obj.cy, obj.cz, obj.rx, obj.ry, obj.rz, obj.sx, obj.sy, obj.sz, obj.alpha };
end
local function restore_obj_props(props)
	obj.ox, obj.oy, obj.oz, obj.cx, obj.cy, obj.cz, obj.rx, obj.ry, obj.rz, obj.sx, obj.sy, obj.sz, obj.alpha = unpack(props);
end

local function inflate_mask(L, R, T, B, inflation, union_left, union_right, union_top, union_bottom, w, h)
	if L < R and T < B and math_abs(inflation) >= 1 then
		local infl_i, dl, dr, dt, db = math_ceil(inflation), 0, 0, 0, 0;
		if infl_i < 0 then
			if union_left and L == 0 then dl = -infl_i end
			if union_right and R == w then dr = -infl_i end
			if union_top and T == 0 then dt = -infl_i end
			if union_bottom and B == h then db = -infl_i end
			L, R, T, B = L - dl, R + dr, T - dt, B + db;
		end
		if math_min(R - L, B - T) + 2 * infl_i > 0 then
			crop_image(L, R, T, B, true);
			if dl > 0 or dr > 0 or dt > 0 or db > 0 then
				obj.effect("領域拡張", "左", dl, "右", dr, "上", dt, "下", db, "塗りつぶし", 1);
			end
			obj.effect("アウトラインσ", "距離", inflation, "ライン幅", -4000, "縁色", 0x000000, "方式", "2値化");
			if infl_i < 0 then
				-- note that outline-sigma does not reduce the size, and that `-infl_i` never exceeds 4000.
				obj.effect("クリッピング", "左", -infl_i, "右", -infl_i, "上", -infl_i, "下", -infl_i);
			end
			return L - infl_i, R + infl_i, T - infl_i, B + infl_i, L - infl_i, T - infl_i;
		end
		return -1, -1, -1, -1, 0, 0;
	end
	return L, R, T, B, 0, 0;
end

local function mask_and_combine(inflation, conn_corner,
	union_left, union_right, union_top, union_bottom,
	invert, mode_draw, blend, alpha_eff, alpha_src, fixed_size,
	num_points, points, bin_func, filter_func, ...)

	local cache_name_ori = "cache:region_s/obj#"..obj.effect_id;
	local w, h, cx, cy, ofs_x, ofs_y, alpha = obj.w, obj.h, obj.cx, obj.cy, 0, 0, obj.alpha;

	-- extract the connected components.
	obj.copybuffer("tempbuffer", "object");
	bin_func("tempbuffer", "object");
	local L, R, T, B = extract_components(
		"object", "object",
		conn_corner, union_left, union_right, union_top, union_bottom,
		num_points, points);
	L, R, T, B, ofs_x, ofs_y = inflate_mask(L, R, T, B, inflation, union_left, union_right, union_top, union_bottom, w, h);

	if filter_func == nil or alpha_eff <= 0 or (not invert and (L >= R or T >= B)) then
		-- trivial case where no effect is applied.
		if L >= R or T >= B or alpha_eff == alpha_src then
			obj.pixelshader("const_alpha@連結成分塗りつぶし@Region_S", "tempbuffer", nil, { invert and alpha_eff or alpha_src }, "mask");
		else
			obj.pixelshader("bounded_mask@連結成分塗りつぶし@Region_S", "tempbuffer", "object", {
				L, T, R, B; ofs_x, ofs_y;
				invert and alpha_eff or alpha_src, invert and alpha_src or alpha_eff;
			}, "mask");
		end
	else
		-- extract and save the outer image.
		if alpha_src > 0 then
			obj.copybuffer(cache_name_ori, "tempbuffer");
			obj.pixelshader("bounded_mask@連結成分塗りつぶし@Region_S", cache_name_ori, "object", {
				L, T, R, B; ofs_x, ofs_y;
				invert and 0 or alpha_src, invert and alpha_src or 0;
			}, "mask");
		end

		-- extract the inner and apply effects.
		obj.pixelshader("bounded_mask@連結成分塗りつぶし@Region_S", "tempbuffer", "object", {
			L, T, R, B; ofs_x, ofs_y;
			invert and 1 or 0, invert and 0 or 1;
		}, "mask");
		obj.copybuffer("object", "tempbuffer");
		filter_func(...);
		if obj.w <= 0 or obj.h <= 0 then return end -- subsequent filter already drew.
		local cx1, cy1, alpha1 = obj.cx, obj.cy, obj.alpha;
		obj.cx, obj.cy, obj.alpha = cx, cy, alpha;

		-- combine the effected and original buffers.
		local prev_blend = obj.getoption("blend");
		if mode_draw == 0 then
			-- original is back, effected is front.
			if alpha_src > 0 then obj.copybuffer("tempbuffer", cache_name_ori);
			else obj.clearbuffer("tempbuffer", w, h) end
			obj.setoption("drawtarget", "tempbuffer");
			obj.setoption("blend", blend);
			obj.draw(cx - cx1, cy - cy1, 0, 1, alpha_eff * alpha1 / alpha);
		else
			-- effected is back, original is front.
			obj.setoption("drawtarget", "tempbuffer", w, h);
			obj.draw(cx - cx1, cy - cy1, 0, 1, alpha_eff * alpha1 / alpha);
			if alpha_src > 0 then
				obj.copybuffer("object", cache_name_ori);
				obj.setoption("blend", blend);
				obj.draw();
			end
		end
		obj.setoption("blend", prev_blend);
	end
	obj.copybuffer("object", "tempbuffer");

	-- crop the outside if necessary.
	if not fixed_size then crop_image(L, R, T, B, false) end
end

local function fill_anchored_blank(thresh, inflation, conn_corner,
	union_left, union_right, union_top, union_bottom,
	color, invert, mode_draw, blend, alpha_eff, alpha_src,
	num_points, points, filter_func, ...)

	local cache_name_ori = "cache:region_s/obj#"..obj.effect_id;
	local w, h, cx, cy, ofs_x, ofs_y, alpha = obj.w, obj.h, obj.cx, obj.cy, 0, 0, obj.alpha;

	-- extract the connected components.
	obj.copybuffer("tempbuffer", "object");
	obj.pixelshader("bin_by_alpha@連結成分塗りつぶし@Region_S", "object", "tempbuffer", { thresh, 1, 0 });
	local L, R, T, B = extract_components(
		"object", "object",
		conn_corner, union_left, union_right, union_top, union_bottom,
		num_points, points);
	L, R, T, B, ofs_x, ofs_y = inflate_mask(L, R, T, B, inflation, union_left, union_right, union_top, union_bottom, w, h);

	if alpha_eff <= 0 or (not invert and (L >= R or T >= B)) then
		-- trivial case where no effect is applied.
		obj.pixelshader("const_alpha@連結成分塗りつぶし@Region_S", "tempbuffer", nil, { alpha_src }, "mask");
	else
		-- save the original image.
		if alpha_src > 0 then
			obj.copybuffer(cache_name_ori, "tempbuffer");
			if alpha_src < 1 then
				obj.pixelshader("const_alpha@連結成分塗りつぶし@Region_S", cache_name_ori, nil, { alpha_src }, "mask");
			end
		end

		-- extract the filling area.
		obj.clearbuffer("tempbuffer", color);
		obj.pixelshader("bounded_mask@連結成分塗りつぶし@Region_S", "tempbuffer", "object", {
			L, T, R, B; ofs_x, ofs_y;
			invert and 1 or 0, invert and 0 or 1;
		}, "mask");
		obj.copybuffer("object", "tempbuffer");

		-- apply effect.
		if filter_func ~= nil then
			filter_func(...);
			if obj.w <= 0 or obj.h <= 0 then return end -- subsequent filter already drew.
		end
		local cx1, cy1, alpha1 = obj.cx, obj.cy, obj.alpha;
		obj.cx, obj.cy, obj.alpha = cx, cy, alpha;

		-- combine the effected and original buffers.
		local prev_blend = obj.getoption("blend");
		if mode_draw == 0 then
			-- original is back, effected is front.
			if alpha_src > 0 then obj.copybuffer("tempbuffer", cache_name_ori);
			else obj.clearbuffer("tempbuffer", w, h) end
			obj.setoption("drawtarget", "tempbuffer");
			obj.setoption("blend", blend);
			obj.draw(cx - cx1, cy - cy1, 0, 1, alpha_eff * alpha1 / alpha);
		else
			-- effected is back, original is front.
			obj.setoption("drawtarget", "tempbuffer", w, h);
			obj.draw(cx - cx1, cy - cy1, 0, 1, alpha_eff * alpha1 / alpha);
			if alpha_src > 0 then
				obj.copybuffer("object", cache_name_ori);
				obj.setoption("blend", blend);
				obj.draw();
			end
		end
		obj.setoption("blend", prev_blend);
	end
	obj.copybuffer("object", "tempbuffer");
end

local function anchored_transparent_key(thresh, inflation, conn_corner,
	union_left, union_right, union_top, union_bottom,
	invert, num_points, points, key_func, ...)

	-- backup the original image.
	local cache_name_ori = "cache:region_s/obj";
	local w, h = obj.w, obj.h;
	obj.copybuffer(cache_name_ori, "object");

	-- apply function.
	key_func(...);

	-- extract the connected components.
	local ofs_x, ofs_y = 0, 0;
	obj.copybuffer("tempbuffer", "object");
	obj.pixelshader("bin_by_alpha@連結成分塗りつぶし@Region_S", "object", "tempbuffer", { thresh, 1, 0 });
	local L, R, T, B = extract_components(
		"object", "object",
		conn_corner, union_left, union_right, union_top, union_bottom,
		num_points, points);
	L, R, T, B, ofs_x, ofs_y = inflate_mask(L, R, T, B, inflation, union_left, union_right, union_top, union_bottom, w, h);

	-- combine the keyed image and the original.
	obj.pixelshader("bounded_mask@連結成分塗りつぶし@Region_S", "tempbuffer", "object", {
		L, T, R, B; ofs_x, ofs_y;
		invert and 1 or 0, invert and 0 or 1;
	}, "mask");
	obj.pixelshader("bounded_mask@連結成分塗りつぶし@Region_S", cache_name_ori, "object", {
		L, T, R, B; ofs_x, ofs_y;
		invert and 0 or 1, invert and 1 or 0;
	}, "mask");
	obj.copybuffer("object", cache_name_ori);
	obj.setoption("drawtarget", "tempbuffer");
	obj.draw();
	obj.copybuffer("object", "tempbuffer");
end

return {
	extract_components = extract_components,

	print_script_error = print_script_error,
	execute_script = execute_script,
	crop_image = crop_image,

	PI_as_bool = PI_as_bool,
	PI_choose_blend_mode = PI_choose_blend_mode,

	save_obj_props = save_obj_props,
	restore_obj_props = restore_obj_props,

	mask_and_combine = mask_and_combine,
	fill_anchored_blank = fill_anchored_blank,
	anchored_transparent_key = anchored_transparent_key,
};

