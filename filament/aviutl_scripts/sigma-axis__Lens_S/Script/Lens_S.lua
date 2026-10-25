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

-- https://github.com/sigma-axis/aviutl2_script_Lens_S

local obj, tonumber, type, assert = obj, tonumber, type, assert;

if obj.getinfo("version") < tonumber("2011000") then
	error([[AviUtl ExEdit version 2.1.10 以降が必要です！]], 2);
end

local basic_s = require("Basic_S");
if (tonumber((basic_s.VERSION or ""):match("^v(.*)$")) or 0) < tonumber("2.88") then
	error([[Basic_S の v2.88 以降が必要です！]], 2);
end
local math_min, math_max, math_floor, math_ceil, math_cos, math_sin, math_log, math_tau = math.min, math.max, math.floor, math.ceil, math.cos, math.sin, math.log, 2 * math.pi;
local image_max_w, image_max_h = obj.getinfo("image_max");

---@alias mat3x3_index # 3x3 行列のテーブルのインデックスと，行列成分の位置との対応を記述．
---|`1` m11
---|`2` m12
---|`3` m13
---|`4` m21
---|`5` m22
---|`6` m23
---|`7` m31
---|`8` m32
---|`9` m33
---@alias mat3x3 table<mat3x3_index, number> 3x3 行列を表すテーブル．

---行列と縦ベクトルとの積を計算する．
---@param M mat3x3 積に使う行列．
---@param x number 縦ベクトルの X.
---@param y number 縦ベクトルの Y.
---@param z number 縦ベクトルの Z.
---@return number x, number y, number z # 結果のベクトルの 3 成分．
local function mat3x3_mul_col_vec(M, x, y, z)
	return
		M[1] * x + M[2] * y + M[3] * z,
		M[4] * x + M[5] * y + M[6] * z,
		M[7] * x + M[8] * y + M[9] * z;
end

---行列と横ベクトルとの積を計算する．
---@param x number 横ベクトルの X.
---@param y number 横ベクトルの Y.
---@param z number 横ベクトルの Z.
---@param N mat3x3 積に使う行列．
---@return number x, number y, number z # 結果のベクトルの 3 成分．
local function mat3x3_mul_row_vec(x, y, z, N)
	return
		x * N[1] + y * N[4] + z * N[7],
		x * N[2] + y * N[5] + z * N[8],
		x * N[3] + y * N[6] + z * N[9];
end

---行列の積を計算して一番左のテーブルに格納する．
---@param L mat3x3
---@param R mat3x3?
---@param ... mat3x3?
---@return mat3x3 # 結果の積．引数の一番左のテーブルと同一．
local function mat3x3_mul_to_left(L, R, ...)
	if not R then return L end
	L[1], L[2], L[3] = mat3x3_mul_row_vec(L[1], L[2], L[3], R);
	L[4], L[5], L[6] = mat3x3_mul_row_vec(L[4], L[5], L[6], R);
	L[7], L[8], L[9] = mat3x3_mul_row_vec(L[7], L[8], L[9], R);
	return mat3x3_mul_to_left(L, ...);
end

---行列の積を計算して一番右のテーブルに格納する．
---@param L mat3x3
---@param R mat3x3?
---@param ... mat3x3?
---@return mat3x3 # 結果の積．引数の一番右のテーブルと同一．
local function mat3x3_mul_to_right(L, R, ...)
	if not R then return L end
	R = mat3x3_mul_to_right(R, ...);
	R[1], R[4], R[7] = mat3x3_mul_col_vec(L, R[1], R[4], R[7]);
	R[2], R[5], R[8] = mat3x3_mul_col_vec(L, R[2], R[5], R[8]);
	R[3], R[6], R[9] = mat3x3_mul_col_vec(L, R[3], R[6], R[9]);
	return R;
end

local function PI_choose_color_opt(pi_value, gui_value)
	if pi_value == false then return nil end
	local value = tonumber(pi_value) or gui_value;
	if not value then return end
	return math_floor(0.5 + value) % 2 ^ 24;
end

local transform_camera_param, transform_billboard, transform_object_scene do
	local function normal3(x, y, z)
		local l = (x ^ 2 + y ^ 2 + z ^ 2) ^ 0.5;
		if l > 0 then return x / l, y / l, z / l;
		else return 0, 0, 0 end
	end
	local function cross3(x1, y1, z1, x2, y2, z2)
		return
			y1 * z2 - z1 * y2,
			z1 * x2 - x1 * z2,
			x1 * y2 - y1 * x2;
	end
	local gv = obj.getvalue;

	---@class camera_transform カメラの変換行列などを保持するテーブル．
	---@field x number カメラ位置の X 座標．
	---@field y number カメラ位置の Y 座標．
	---@field z number カメラ位置の Z 座標．
	---@field T mat3x3 シーン座標からカメラ座標系への変換行列．
	---@field T_i mat3x3 `T` の逆行列．

	---カメラ制御のパラメタから，カメラ座標系との変換行列などを保持するテーブルを計算する．
	---@param camera_param camera_param `obj.getoption("camera_param")` の戻り値．
	---@return camera_transform # カメラ座標系との変換行列などを保持するテーブル．
	function transform_camera_param(camera_param)
		local x, y, z = camera_param.x, camera_param.y, camera_param.z;
		local dx, dy, dz = normal3( -- depth-ward vector (Z).
			camera_param.tx - x, camera_param.ty - y, camera_param.tz - z);
		if dx == 0 and dy == 0 and dz == 0 then dx, dy, dz = 0, 0, 1 end
		local rx, ry, rz = normal3(cross3( -- right-ward vector (X).
			-camera_param.ux, -camera_param.uy, -camera_param.uz, dx, dy, dz));
		local bx, by, bz = normal3(cross3( -- bottom-ward vector (Y).
			dx, dy, dz, rx, ry, rz));

		local scale, rot = camera_param.d / 1024, math_tau * ((camera_param.rz / 360) % 1);
		local c, s = math_cos(rot), math_sin(rot);
		local M = {
			c * rx - s * bx, c * ry - s * by, c * rz - s * bz;
			s * rx + c * bx, s * ry + c * by, s * rz + c * bz;
			dx, dy, dz;
		}; -- orthogonal matrix.

		return {
			x = x, y = y, z = z;
			T = {
				M[1] * scale, M[2] * scale, M[3] * scale;
				M[4] * scale, M[5] * scale, M[6] * scale;
				M[7], M[8], M[9];
			},
			T_i = {
				M[1] / scale, M[4] / scale, M[7];
				M[2] / scale, M[5] / scale, M[8];
				M[3] / scale, M[6] / scale, M[9];
			};
		};

		--[[
			cam = obj.getoption("camera_param");
			.x   : カメラの座標X
			.y   : カメラの座標Y
			.z   : カメラの座標Z
			.tx  : カメラの目標座標X
			.ty  : カメラの目標座標Y
			.tz  : カメラの目標座標Z
			.rz  : カメラの傾き
			.ux  : カメラの上方向単位ベクトルX
			.uy  : カメラの上方向単位ベクトルY
			.uz  : カメラの上方向単位ベクトルZ
			.d   : カメラからスクリーンまでの距離(焦点距離)

			note:
				- (tx, ty, tz) == (x, y, z) のとき，カメラの向きは (0, 0, 1) 方向 ((tx, ty, tz) = (x, y, z + 1) と修正されたのと同等).
				- (ux, uy, uz) はカメラ方向と垂直とは限らない．これが画面の上方向になるようにまず回転する (rz の影響前). 通常は (0, -1, 0).
				- (ux, uy, uz) がカメラ方向と平行だと正しく描画されない．
				- rz の影響は，(ux, uy, uz) の後に回転する形．正の値で描画結果が時計回りに動く．
				- d はカメラ制御の深度ぼけや焦点レイヤーの設定には無関係．視野角に依存して変化．距離 d にあるオブジェクトが等倍で描画される．
				- カメラ制御の視野角が "既定値" (0 値) のときの d は 1024.
				- d が 0 や負の場合の挙動は未確認．
			default_cam = {
				x = 0, y = 0, z = -1024,
				tx = 0, ty = 0, tz = 0, rz = 0,
				ux = 0, uy = -1, uz = 0,
				d = 1024,
			};
		]]
	end

	---「カメラ制御オプション」の「オブジェクトの向き」で指定した向きに合わせるための変換行列を計算する．
	---@param billboard 0|1|2|3 `obj.getoption("billboard")` の戻り値．0: 向かない, 1: 横方向のみ, 2: 縦横方向のみ, 3: 向く．
	---@param cam camera_transform `transform.camera_param()` の戻り値．
	---@return mat3x3? # `nil` だと単位行列の意味．オブジェクト固有の各種回転の「前」にかかることに注意．
	function transform_billboard(billboard, cam)
		if billboard == 0 then return nil;
		elseif billboard == 3 then
			-- yaw-pitch-roll の 3 成分．そのまま `cam` の行列を使ったほうが早い．
			local x, y, z = cam.T_i[1], cam.T_i[4], cam.T_i[7];
			local t = (x ^ 2 + y ^ 2 + z ^ 2) ^ -0.5;
			return {
				t * x, t * cam.T_i[2], cam.T_i[3];
				t * y, t * cam.T_i[5], cam.T_i[6];
				t * z, t * cam.T_i[8], cam.T_i[9];
			};
		else
			local x, y, z = cam.T_i[3], cam.T_i[6], cam.T_i[9];
			local L = (x ^ 2 + z ^ 2) ^ 0.5;
			local Z, X = z / L, x / L;
			if billboard == 1 then
				-- yaw のみ．
				return {
					 Z, 0, X;
					 0, 1, 0;
					-X, 0, Z;
				};
			else
				-- yaw と pitch のみ．
				return {
					 Z, -y * X, x;
					 0,      L, y;
					-X, -y * Z, z;
				};
			end
		end
	end

	---オブジェクト座標とシーン座標の変換に必要な座標や行列を取得・計算する．カメラ制御に関する情報を恣意的に与えることができるが，省略も可能．ただし前提として，グループ制御が 2 階層以上の入れ子になっていないことが必要．
	---@param billboard 0|1|2|3 `obj.getoption("billboard")` の戻り値．0: 向かない, 1: 横方向のみ, 2: 縦横方向のみ, 3: 向く．カメラ制御下ではない場合は `0` を指定．
	---@param cam camera_transform? `transform.camera_param()` の戻り値．省略時はカメラ制御下ではないとして計算をする．
	---@return number tx, number ty オブジェクトのスクリーン座標補正 (立体射影後に加算, カメラ制御下では 0).
	---@return number ox, number oy, number oz オブジェクトのシーン内座標位置 (回転中心の変換先).
	---@return number cx, number cy, number cz オブジェクトの回転中心座標．
	---@return mat3x3 T オブジェクト座標からシーン座標への変換行列．
	---@return mat3x3 T_i `T` の逆行列．
	function transform_object_scene(billboard, cam)
		local ox, oy, oz = gv("pos");
		local cx, cy, cz = gv("center");
		local sx, sy, sz = gv("scale");
		local rx, ry, rz = gv("angle");
		ox, oy, oz = ox + obj.ox, oy + obj.oy, oz + obj.oz;
		cx, cy, cz = cx + obj.cx, cy + obj.cy, cz + obj.cz;
		sx, sy, sz = sx * obj.sx, sy * obj.sy, sz * obj.sz;
		rx, ry, rz = rx + obj.rx, ry + obj.ry, rz + obj.rz;
		rx, ry, rz = math_tau * ((rx / 360) % 1), math_tau * ((ry / 360) % 1), math_tau * ((rz / 360) % 1);
		local rx_c, rx_s, ry_c, ry_s, rz_c, rz_s =
			math_cos(rx), math_sin(rx),
			math_cos(ry), math_sin(ry),
			math_cos(rz), math_sin(rz);

		local M = {
			-- product of: rot(rx) rot(ry) rot(rz); column vectors are multiplied from right.
			 ry_c * rz_c, -ry_c * rz_s, ry_s;
			 rx_s * ry_s * rz_c + rx_c * rz_s, rx_c * rz_c - rx_s * ry_s * rz_s, -rx_s * ry_c;
			-rx_c * ry_s * rz_c + rx_s * rz_s, rx_s * rz_c + rx_c * ry_s * rz_s,  rx_c * ry_c;
		};

		local tx, ty = ox, oy; ox, oy = 0, 0;
		for i = 0, obj.layer - 2 do
			local group_layer = obj.getoption("group_info", i);
			if group_layer <= 0 then break end
			local gx, gy, gz =
				tonumber(gv(group_layer, "グループ制御", "X")) or 0,
				tonumber(gv(group_layer, "グループ制御", "Y")) or 0,
				tonumber(gv(group_layer, "グループ制御", "Z")) or 0;
			local grx, gry, grz =
				math_tau * (((tonumber(gv(group_layer, "グループ制御", "X軸回転")) or 0) / 360) % 1),
				math_tau * (((tonumber(gv(group_layer, "グループ制御", "Y軸回転")) or 0) / 360) % 1),
				math_tau * (((tonumber(gv(group_layer, "グループ制御", "Z軸回転")) or 0) / 360) % 1);
			local gzm = (tonumber(gv(group_layer, "グループ制御", "拡大率")) or 100) / 100;
			local grx_c, grx_s, gry_c, gry_s, grz_c, grz_s =
				math_cos(grx), math_sin(grx),
				math_cos(gry), math_sin(gry),
				math_cos(grz), math_sin(grz);
			local N = {
				 gry_c * grz_c, -gry_c * grz_s, gry_s;
				 grx_s * gry_s * grz_c + grx_c * grz_s, grx_c * grz_c - grx_s * gry_s * grz_s, -grx_s * gry_c;
				-grx_c * gry_s * grz_c + grx_s * grz_s, grx_s * grz_c + grx_c * gry_s * grz_s,  grx_c * gry_c;
			};

			mat3x3_mul_to_right(N, M);
			ox, oy, oz = mat3x3_mul_col_vec(N,
				gzm * (ox + tx), gzm * (oy + ty), gzm * oz);
			tx, ty, oz = gx, gy, oz + gz;
			sx, sy, sz = gzm * sx, gzm * sy, gzm * sz;
		end

		if cam ~= nil then
			ox, oy, tx, ty = ox + tx, oy + ty, 0, 0;
			mat3x3_mul_to_left(M, transform_billboard(billboard, cam));
		end

		-- note that M is an orthogonal matrix.
		return
			tx, ty,
			ox, oy, oz,
			cx, cy, cz,
			{
				M[1] * sx, M[2] * sy, M[3] * sz;
				M[4] * sx, M[5] * sy, M[6] * sz;
				M[7] * sx, M[8] * sy, M[9] * sz;
			}, {
				M[1] / sx, M[4] / sx, M[7] / sx;
				M[2] / sy, M[5] / sy, M[8] / sy;
				M[3] / sz, M[6] / sz, M[9] / sz;
			};
	end
end

---オブジェクトが描画される予定の位置にあるフレームバッファの部分を切り出し，指定したバッファにコピーする．カメラ制御に関する情報を恣意的に与えることができるが，省略も可能．
---@param buff_target string コピー先のバッファ名 ("tempbuffer" や "cache:***" など).
---@param w integer オブジェクトの想定ピクセル幅．`obj.w` と一致する必要はない．
---@param h integer オブジェクトの想定ピクセル高さ．`obj.h` と一致する必要はない．
---@param back_col_r number 背景の透明部分や，消失点の外側を補う背景色．R 成分．
---@param back_col_g number 背景の透明部分や，消失点の外側を補う背景色．G 成分．
---@param back_col_b number 背景の透明部分や，消失点の外側を補う背景色．B 成分．
---@param back_col_a number 背景の透明部分や，消失点の外側を補う背景色．A 成分．
---@param move_x number 背景の平行移動量．オブジェクト座標でのピクセル単位．
---@param move_y number 背景の平行移動量．オブジェクト座標でのピクセル単位．
---@param scale number 背景の拡大率．1.0 で等倍．中心は回転中心．
---@param rotate number 背景の回転角．ラジアン単位．中心は回転中心．
---@param anchor_screen boolean 背景の平行移動拡縮回転の起点を，オブジェクトの回転中心ではなく，画面の中央にするかどうか．
---@param billboard 0|1|2|3|nil `obj.getoption("billboard")` の戻り値．省略時は `cam` を含めて現在の状態を取得する．0: 向かない, 1: 横方向のみ, 2: 縦横方向のみ, 3: 向く．カメラ制御下でない場合は `0` を指定すること．
---@param cam camera_transform? `transform.camera_param()` の戻り値．省略時はカメラ制御下ではないものとして計算をする．`billboard` の省略時は無視される．
local function clip_framebuffer(buff_target, w, h,
	back_col_r, back_col_g, back_col_b, back_col_a,
	move_x, move_y, scale, rotate, anchor_screen, billboard, cam)
	-- complete optional parameters.
	if billboard == nil then
		billboard, cam = 0, nil;
		if obj.getoption("camera_mode") ~= 0 then
			billboard, cam = obj.getoption("billboard"),
				transform_camera_param(obj.getoption("camera_param"));
		end
	end

	-- calculate overall composite transform.
	local tx, ty, ox, oy, oz, cx, cy, cz, T, _ = transform_object_scene(billboard, cam);

	-- object-local transforms.
	local M = mat3x3_mul_to_right(T, {
		1, 0, -w / 2 - cx;
		0, 1, -h / 2 - cy;
		0, 0, -cz;
	});
	M[3], M[6], M[9] = M[3] + ox, M[6] + oy, M[9] + oz;

	-- transforms by camera control.
	if cam ~= nil then
		M[3], M[6], M[9] = M[3] - cam.x, M[6] - cam.y, M[9] - cam.z;
		mat3x3_mul_to_right(cam.T, M);
		M[9] = M[9] - 1024;
	end

	-- project.
	M[7], M[8], M[9] = M[7] / 1024, M[8] / 1024, M[9] / 1024;
	M[9] = M[9] + 1;
	mat3x3_mul_to_right({
		1, 0, tx;
		0, 1, ty;
		0, 0, 1;
	}, M);

	-- extra transform.
	if not (move_x == 0 and move_y == 0 and scale == 1 and rotate == 0) then
		local scr_cx, scr_cy, anch_x, anch_y;

		-- calculate the destinations of the center and the anchor.
		if anchor_screen then
			scr_cx, scr_cy = 0, 0;
			anch_x, anch_y = move_x, move_y;
		else
			local x, y, z = mat3x3_mul_col_vec(M, w / 2 + cx, h / 2 + cy, 1);
			scr_cx, scr_cy = x / z, y / z;
			x, y, z = mat3x3_mul_col_vec(M, w / 2 + cx + move_x, h / 2 + cy + move_y, 1);
			anch_x, anch_y = x / z, y / z;
		end

		-- append transform.
		local c, s = math_cos(-rotate) / scale, math_sin(-rotate) / scale;
		mat3x3_mul_to_right({
			1, 0, scr_cx;
			0, 1, scr_cy;
			0, 0, 1;
		}, {
			c, -s, 0;
			s, c, 0;
			0, 0, 1;
		}, {
			1, 0, -anch_x;
			0, 1, -anch_y;
			0, 0, 1;
		}, M);
	end

	-- normalize.
	mat3x3_mul_to_right({
		1 / obj.screen_w, 0, 0.5;
		0, 1 / obj.screen_h, 0.5;
		0, 0, 1;
	}, M);

	-- apply shader.
	obj.clearbuffer(buff_target, w, h);
	obj.pixelshader("back_proj@アクリルσ@Lens_S", buff_target, "framebuffer", {
		back_col_r, back_col_g, back_col_b, back_col_a;
		M[1], M[4], M[7], 0,
		M[2], M[5], M[8]; 0;
		M[3], M[6], M[9];
	}, "copy", "clamp");
end

---アクリルσの実体関数．値の型や範囲チェックは行われないので，事前に指定範囲内の保証をしておくこと．
---@param base_alpha number 基準透明度によるアルファ値，0.0 -- 1.0.
---@param base_color integer? 基準色．
---@param mul_luma number 透過輝度，0 以上．通常は 0.0 -- 1.0.
---@param add_luma number 輝度補正，nan, infty 以外．通常は -1.0 -- 1.0.
---@param mul_chroma number 透過彩度，0 以上．通常は 0.0 -- 1.0.
---@param tint_color integer 着色．
---@param tint_luma number 着色輝度，0 以上．通常は 0.0 -- 1.0.
---@param tint_chroma number 着色彩度，0 以上．通常は 0.0 -- 1.0.
---@param blur_x number X 方向のぼかし範囲，0 -- 1000.
---@param blur_y number Y 方向のぼかし範囲，0 -- 1000.
---@param blur_luma_weight number 光の強さ，0 -- 60.
---@param noise_intensity number ノイズ，0 以上．通常は 0.0 -- 1.0.
---@param noise_seed integer シード．0 -- 2^20 - 1. オブジェクトごとの違いなどは考慮しない．
---@param noise_size number ドットサイズ．1.0 以上．
---@param move_x number 移動X，nan, infty 以外．
---@param move_y number 移動Y，nan, infty 以外．
---@param scale number 拡大率，0.01 -- 100. 1.0 で等倍．
---@param rotate number 回転，nan, infty 以外．ラジアン単位．
---@param anchor_screen boolean 画面基準で配置．
---@param back_color integer|nil 背景色．
local function apply_acryl(
	base_alpha, base_color,
	mul_luma, add_luma, mul_chroma,
	tint_color, tint_luma, tint_chroma,
	blur_x, blur_y, blur_luma_weight,
	noise_intensity, noise_seed, noise_size,
	move_x, move_y, scale, rotate,
	anchor_screen, back_color)
	-- further calculations.
	add_luma = add_luma + (1 - mul_luma) / 2;
	local blur_xi, blur_yi = math_ceil(blur_x), math_ceil(blur_y);
	local cx, cy = 0, 0;
	if noise_intensity > 0 then
		cx, cy = obj.getvalue("center");
		cx, cy = cx + obj.cx, cy + obj.cy;
	end
	local base_col_r, base_col_g, base_col_b, base_col_a = basic_s.rgba_color_opt(base_color);
	local tint_col_r, tint_col_g, tint_col_b = basic_s.rgba_color_opt(tint_color);
	local back_col_r, back_col_g, back_col_b, back_col_a = basic_s.rgba_color_opt(back_color);

	-- backup the original image.
	local cache_name_orig, cache_name_back =
		"cache:lens_s/acryl/orig#"..obj.effect_id,
		"cache:lens_s/acryl/back#"..obj.effect_id;
	local w, h = obj.w, obj.h;
	assert(obj.copybuffer(cache_name_orig, "object"));

	-- prepare clipped background.
	clip_framebuffer((blur_x > 0 or blur_y > 0) and "object" or cache_name_back,
		w + 2 * blur_xi, h + 2 * blur_yi,
		back_col_r, back_col_g, back_col_b, back_col_a,
		move_x, move_y, scale, rotate, anchor_screen);

	-- apply blur to the background.
	if blur_x > 0 or blur_y > 0 then
		basic_s.effect.prec_blur(blur_x, blur_y, blur_luma_weight, true, 1);
		obj.effect("クリッピング", "左", blur_xi, "右", blur_xi, "上", blur_yi, "下", blur_yi);
		assert(obj.copybuffer(cache_name_back, "object"));
	end

	-- apply shader.
	if obj.w ~= w or obj.h ~= h then obj.clearbuffer("object", w, h) end
	obj.pixelshader("combine@アクリルσ@Lens_S",
		"object", { cache_name_orig, cache_name_back }, {
		base_col_r, base_col_g, base_col_b, base_col_a; base_alpha;
		tint_col_r, tint_col_g, tint_col_b; tint_luma; tint_chroma;
		mul_luma; mul_chroma; add_luma;
		noise_intensity; noise_seed; 1 / noise_size;
		cx + w / 2, cy + h / 2;
	});
end

---レンズσの実体関数．値の型や範囲チェックは行われないので，事前に指定範囲内の保証をしておくこと．
---@param mul_luma number 透過輝度，0 以上．通常は 0.0 -- 1.0.
---@param add_luma number 輝度補正，nan, infty 以外．通常は -1.0 -- 1.0.
---@param mul_chroma number 透過彩度，0 以上．通常は 0.0 -- 1.0.
---@param face_luma number 着色輝度，0 以上．通常は 0.0 -- 1.0.
---@param face_chroma number 着色彩度，0 以上．通常は 0.0 -- 1.0.
---@param lift number 背景距離，nan, infty 以外．
---@param edge_amplify number 輪郭起伏，ピクセル単位，nan, infty 以外．
---@param edge_round number 輪郭丸み，ピクセル単位，0 以上．
---@param luma_amplify number 輝度起伏，ピクセル単位，nan, infty 以外．
---@param luma_round number 輝度丸み，ピクセル単位，0 -- 1000.
---@param col_amplify number 色境界起伏，ピクセル単位，nan, infty 以外．
---@param col_round number 色境界丸み，ピクセル単位，0 以上．
---@param col_thresh number 色境界しきい値，0.0 -- 1.0.
---@param smooth number 平滑化，ピクセル単位，0 -- 1000.
---@param refr_idx number 屈折率，1.0 以上．
---@param chrm_abrr number 色収差，，-1.0 -- 1.0.
---@param light_angle number 光角度，nan, infty 以外．ラジアン単位．
---@param refl_power number 光沢，約 1.5 以上．小さすぎるとアーティファクトの原因．
---@param refl_light number ハイライト，0 以上．通常は 0.0 -- 1.0.
---@param refl_shadow number シャドウ，0 以上．通常は 0.0 -- 1.0.
---@param light_screen boolean 光角度を画面基準．
---@param blur_x number X 方向のぼかし範囲，0 -- 1000.
---@param blur_y number Y 方向のぼかし範囲，0 -- 1000.
---@param blur_luma_weight number 光の強さ，0 -- 60.
---@param noise_intensity number ノイズ，0 以上．通常は 0.0 -- 1.0.
---@param noise_seed integer シード．0 -- 2^20 - 1. オブジェクトごとの違いなどは考慮しない．
---@param noise_size number ドットサイズ．1.0 以上．
---@param move_x number 移動X，nan, infty 以外．
---@param move_y number 移動Y，nan, infty 以外．
---@param scale number 拡大率，0.01 -- 100. 1.0 で等倍．
---@param rotate number 回転，nan, infty 以外．ラジアン単位．
---@param anchor_screen boolean 画面基準で配置．
---@param back_color integer|nil 背景色．
---@param chrm_abrr_order 0|1|2 色収差順序．0 -> 赤緑青，1 -> 緑青赤，2 -> 青赤緑．
---@param visualize_shape boolean 形状可視化．
local function apply_lens(
	mul_luma, add_luma, mul_chroma, face_luma, face_chroma,
	lift, edge_amplify, edge_round, luma_amplify, luma_round, col_amplify, col_round, col_thresh, smooth,
	refr_idx, chrm_abrr,
	light_angle, refl_power, refl_light, refl_shadow, light_screen,
	blur_x, blur_y, blur_luma_weight,
	noise_intensity, noise_seed, noise_size,
	move_x, move_y, scale, rotate, anchor_screen,
	back_color, chrm_abrr_order,
	visualize_shape)
	-- further calculations.
	add_luma = add_luma + 0.5 - (mul_luma + face_luma) / 2;
	lift = lift + math_max(-edge_amplify, 0)
		+ math_max(-luma_amplify, 0)
		+ math_max(-col_amplify, 0);
	local max_height = math_max(lift + math_max(edge_amplify, 0) + math_max(luma_amplify, 0) + math_max(col_amplify, 0), 0);
	local blur_xi, blur_yi, margin = math_ceil(blur_x), math_ceil(blur_y),
		math_ceil(2 * math_min(max_height, math_max(edge_round, luma_round, col_round)));
	local cx, cy = 0, 0;
	if noise_intensity > 0 then
		cx, cy = obj.getvalue("center");
		cx, cy = cx + obj.cx, cy + obj.cy;
	end
	margin = math_floor(math_min(margin,
		(image_max_w - obj.w) / 2 - blur_xi,
		(image_max_h - obj.h) / 2 - blur_yi));
	local chrm_abrr_arr = { 1 - chrm_abrr, 1, 1 + chrm_abrr};
	if chrm_abrr_order == 1 then
		chrm_abrr_arr[2], chrm_abrr_arr[3], chrm_abrr_arr[1] = chrm_abrr_arr[1], chrm_abrr_arr[2], chrm_abrr_arr[3];
	elseif chrm_abrr_order == 2 then
		chrm_abrr_arr[3], chrm_abrr_arr[1], chrm_abrr_arr[2] = chrm_abrr_arr[1], chrm_abrr_arr[2], chrm_abrr_arr[3];
	end
	local light_angle_x, light_angle_y = math_sin(light_angle), -math_cos(light_angle);
	local billboard, cam = 0, nil;
	if obj.getoption("camera_mode") ~= 0 then
		billboard, cam = obj.getoption("billboard"),
			transform_camera_param(obj.getoption("camera_param"));
	end
	if light_screen and (refl_light > 0 or refl_shadow > 0) then
		-- rotate the light angle to follow the camera.
		-- calculate overall composite transform.
		local tx, ty, ox, oy, oz, _, _, cz, T, _ = transform_object_scene(billboard, cam);
		local M = mat3x3_mul_to_right(T, {
			1, 0, 0;
			0, 1, 0;
			0, 0, -cz;
		});
		M[3], M[6], M[9] = M[3] + ox, M[6] + oy, M[9] + oz;
		if cam ~= nil then
			M[3], M[6], M[9] = M[3] - cam.x, M[6] - cam.y, M[9] - cam.z;
			mat3x3_mul_to_right(cam.T, M);
			M[9] = M[9] - 1024;
		end
		M[7], M[8], M[9] = M[7] / 1024, M[8] / 1024, M[9] / 1024;
		M[9] = M[9] + 1;
		mat3x3_mul_to_right({
			1, 0, tx;
			0, 1, ty;
			0, 0, 1;
		}, M);

		-- transform the origin and each unit vector.
		local x, y, x1, y1, x2, y2, z;
		x, y, z = mat3x3_mul_col_vec(M, 0, 0, 1); x, y = x / z, y / z;
		x1, y1, z = mat3x3_mul_col_vec(M, 1, 0, 1); x1, y1 = x1 / z, y1 / z;
		x2, y2, z = mat3x3_mul_col_vec(M, 0, 1, 1); x2, y2 = x2 / z, y2 / z;

		-- calculate matrices and a vector.
		x1, y1 = x1 - x, y1 - y;
		x2, y2 = x2 - x, y2 - y;
		x1, x2, y1, y2 = y2, -x2, -y1, x1;
		x, y =
			x1 * light_angle_x + x2 * light_angle_y,
			y1 * light_angle_x + y2 * light_angle_y;
		if x1 * y2 < x2 * y1 then x, y = -x, -y end

		-- normalize.
		local q = x ^ 2 + y ^ 2;
		if q <= 0 then light_angle_x, light_angle_y = 0, -1;
		else light_angle_x, light_angle_y = x * q ^ -0.5, y * q ^ -0.5 end
	end
	local back_col_r, back_col_g, back_col_b, back_col_a = basic_s.rgba_color_opt(back_color);

	-- backup the original image.
	local cache_name_orig, cache_name_back, cache_name_temp1, cache_name_temp2 =
		"cache:lens_s/lens/orig#"..obj.effect_id,
		"cache:lens_s/lens/back#"..obj.effect_id,
		"cache:lens_s/lens/temp1#"..obj.effect_id,
		"cache:lens_s/lens/temp2#"..obj.effect_id;
	local w, h = obj.w, obj.h;
	assert(obj.copybuffer(cache_name_orig, "object"));

	-- prepare blurred luma map if necessary.
	if luma_round > 0 and luma_amplify ~= 0 then
		basic_s.effect.prec_blur(luma_round, luma_round, 0, true, 1);
		assert(obj.copybuffer(cache_name_back, "object"));
	end

	-- create reposition maps.
	obj.pixelshader("dist_init@レンズσ@Lens_S", "object", cache_name_orig, {
		math_max(col_thresh, 2 ^ -24)
	});
	assert(obj.copybuffer(cache_name_temp2, "object"));
	for i = math_ceil(math_max(math_log(math_max(w, h) / 2, 2), -1)), 0, -1 do
		obj.pixelshader("dist_step@レンズσ@Lens_S",
			(i % 2 ~= 0) and "object" or cache_name_temp2,
			(i % 2 == 0) and "object" or cache_name_temp2, {
			w, h; 2 ^ i;
		});
	end
	obj.pixelshader("dist_fin@レンズσ@Lens_S", "object", cache_name_temp2, { w, h });

	-- edit-purpose visualization.
	if visualize_shape then
		local ess_lift, ess_height =
			-math_min(edge_amplify, 0) - math_min(luma_amplify, 0) - math_min(col_amplify, 0),
			math.abs(edge_amplify) + math.abs(luma_amplify) + math.abs(col_amplify);
		obj.computeshader("visualize@レンズσ@Lens_S", "object",
			(luma_round > 0 and luma_amplify ~= 0) and cache_name_back or cache_name_orig, {
			w, h; 1 / (ess_height + 0.5);
			ess_lift; edge_amplify; edge_round; luma_amplify; col_amplify; col_round;
		}, math_ceil(w / 8), math_ceil(h / 8));
		return;
	end

	-- make reflect, refract and highlight maps.
	local ht_adj_factor = math_max(max_height, 0.5);
	obj.clearbuffer(cache_name_temp2, w, h);
	obj.computeshader("calc_lens@レンズσ@Lens_S", { "object", cache_name_temp2 },
		(luma_round > 0 and luma_amplify ~= 0) and cache_name_back or cache_name_orig, {
		w, h; 1 / ht_adj_factor;
		lift; edge_amplify; edge_round; luma_amplify; col_amplify; col_round;
		refr_idx;
		light_angle_x, light_angle_y; refl_power;
	}, math_ceil(w / 8), math_ceil(h / 8));
	if smooth > 0 then
		basic_s.effect.prec_blur(smooth, smooth, 0, true, 1);
		assert(obj.copybuffer(cache_name_temp1, "object"));
		assert(obj.copybuffer("object", cache_name_temp2));
		basic_s.effect.prec_blur(smooth, smooth, 0, true, 1);
		assert(obj.copybuffer(cache_name_temp2, "object"));
	else assert(obj.copybuffer(cache_name_temp1, "object")) end

	-- prepare clipped background.
	clip_framebuffer((blur_x > 0 or blur_y > 0) and "object" or cache_name_back,
		w + 2 * (blur_xi + margin), h + 2 * (blur_yi + margin),
		back_col_r, back_col_g, back_col_b, back_col_a,
		move_x, move_y, scale, rotate, anchor_screen, billboard, cam);

	-- apply blur to the background.
	if blur_x > 0 or blur_y > 0 then
		basic_s.effect.prec_blur(blur_x, blur_y, blur_luma_weight, true, 1);
		obj.effect("クリッピング", "左", blur_xi, "右", blur_xi, "上", blur_yi, "下", blur_yi);
		assert(obj.copybuffer(cache_name_back, "object"));
	end

	-- combine.
	if obj.w ~= w or obj.h ~= h then obj.clearbuffer("object", w, h) end
	obj.pixelshader("combine@レンズσ@Lens_S", "object",
		{ cache_name_orig, cache_name_back, cache_name_temp1, cache_name_temp2 }, {
		w, h; margin; ht_adj_factor;
		chrm_abrr_arr[1], chrm_abrr_arr[2], chrm_abrr_arr[3];
		refl_power; refl_light, refl_shadow;
		mul_luma; add_luma; mul_chroma;
		face_luma; face_chroma;
		noise_intensity; noise_seed; 1/ noise_size;
		cx + w / 2, cy + h / 2;
	}, "copy", "clamp");
end

return {
	transform = {
		camera_param = transform_camera_param,
		billboard = transform_billboard,
		object_scene = transform_object_scene,
	},

	clip_framebuffer = clip_framebuffer,

	PI = {
		color_opt = PI_choose_color_opt,
		as_bool = basic_s.PI.as_bool,
	},

	effect = {
		["アクリルσ"] = apply_acryl, acryl = apply_acryl;
		["レンズσ"] = apply_lens, lens = apply_lens;
	},

	VERSION = "v1.10",
};

