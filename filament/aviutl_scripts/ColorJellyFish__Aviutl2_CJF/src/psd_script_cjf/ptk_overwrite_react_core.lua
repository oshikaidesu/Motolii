--- ptk_overwrite_react_core.lua
-- PSDToolKit パーツ上書きの共有検知コア。
-- 対象:
--   - パーツ上書き@PSDToolKit（標準）
--   - anm2 Editor の @*.ptk.obj2（同じ OverwriterStates API）
--
-- 運用前提（終端 peek 含む）:
--   参照レイヤー1本に上書きを隙間なしで並べる。
--   現在値はキャラID優先（OverwriterStates / SubObjectStates）。
--   終端半窓の peek は参照レイヤーの getvalue のみ（IDでは未来が見えない）。
--
-- 状態は require("ptk_overwrite_react_state") に置く。
-- AviUtl2 の global に table を入れない（文字列のみ可）。

local M = {}

local PART_COUNT = 16
local EFFECT_CANDIDATES = {
	"パーツ上書き@PSDToolKit",
	"パーツ上書き@PSD",
	"パーツ上書き",
	"OverwriteSelector@PSDToolKit",
	"OverwriteSelector@PSD",
}

local function get_state()
	local ok, state = pcall(require, "ptk_overwrite_react_state")
	if not ok or type(state) ~= "table" then
		state = { by_id = {}, part_bank = {}, part_bank_end = {} }
	end
	if type(state.by_id) ~= "table" then
		state.by_id = {}
	end
	if type(state.part_bank) ~= "table" then
		state.part_bank = {}
	end
	if type(state.part_bank_end) ~= "table" then
		state.part_bank_end = {}
	end
	return state
end

function M.parse_watch_parts(spec)
	local mask = {}
	if spec == nil or tostring(spec):gsub("%s+", "") == "" then
		for i = 1, PART_COUNT do
			mask[i] = true
		end
		return mask, "all"
	end
	-- 区切りは半角/全角カンマなど可: 4,5 / 4、5 / 4，5
	local s = tostring(spec):gsub("%s+", "")
	s = s:gsub("、", ","):gsub("，", ","):gsub(";", ",")
	local label_bits = {}
	for token in string.gmatch(s, "[^,]+") do
		local n = tonumber(token)
		if n ~= nil then
			n = math.floor(n)
			if n >= 1 and n <= PART_COUNT then
				mask[n] = true
				label_bits[#label_bits + 1] = tostring(n)
			end
		end
	end
	if #label_bits == 0 then
		for i = 1, PART_COUNT do
			mask[i] = true
		end
		return mask, "all(fallback)"
	end
	return mask, table.concat(label_bits, ",")
end

local function watched_changed(prev, curr, mask)
	if prev == nil or curr == nil then
		return false, ""
	end
	local bits = {}
	for i = 1, PART_COUNT do
		if mask[i] and prev[i] ~= curr[i] then
			bits[#bits + 1] = "p" .. i .. "=" .. tostring(prev[i]) .. "->" .. tostring(curr[i])
		end
	end
	return #bits > 0, table.concat(bits, ",")
end

local function normalize_ow(ow)
	if type(ow) ~= "table" then
		return nil, false
	end
	local values = {}
	for i = 1, PART_COUNT do
		local v = ow["p" .. i]
		values[i] = (v ~= nil) and (tonumber(v) or 0) or 0
	end
	return values, true
end

local function framerate()
	local fr = tonumber(obj.framerate) or 30
	if fr < 1 then
		fr = 30
	end
	return fr
end

--- 描画中オブジェクトのシーン時刻（秒, シーン先頭=0）。
local function scene_time_now()
	local fr = framerate()
	return (tonumber(obj.frame_s) or 0) / fr + (tonumber(obj.time) or 0)
end

--- シーン時刻 → 対象レイヤーオブジェクトのローカル時刻。
local function layer_local_time(layer, scene_t)
	local fr = framerate()
	local ok, fs = pcall(obj.getvalue, "layer" .. tostring(layer) .. ".frame_s")
	if ok and fs ~= nil then
		return scene_t - (tonumber(fs) or 0) / fr
	end
	-- フォールバック: 上書きもこのオブジェクトと同じ始端とみなす。
	return scene_t - (tonumber(obj.frame_s) or 0) / fr
end

local function read_via_overwriter_states(layer, want_id)
	local ok, OverwriterStates = pcall(require, "PSDToolKit.OverwriterStates")
	if not ok or OverwriterStates == nil then
		return nil, false, "require_failed:" .. tostring(OverwriterStates)
	end
	local tried = {}
	local ow, how = nil, ""
	if want_id ~= nil and want_id ~= "" then
		ow = OverwriterStates:get(want_id)
		tried[#tried + 1] = "id=" .. tostring(want_id) .. ":" .. tostring(ow ~= nil)
		if ow ~= nil then
			how = "id=" .. tostring(want_id)
		end
	end
	if ow == nil then
		local key = "L" .. tostring(layer)
		ow = OverwriterStates:get(key)
		tried[#tried + 1] = key .. ":" .. tostring(ow ~= nil)
		how = key
	end
	local values, any = normalize_ow(ow)
	if not any then
		return nil, false, table.concat(tried, ",")
	end
	return values, true, how
end

-- getvalue ホットパス用の固定アイテム名（都度生成を避ける）。
local ITEM_NAME_CANDIDATES = {}
for i = 1, PART_COUNT do
	ITEM_NAME_CANDIDATES[i] = {
		"p" .. i,
		"track.p" .. i,
		"select.p" .. i,
		"パーツ" .. i,
	}
end

-- peek は複数スロット必須。1ヒット+ゼロ埋めだと誤って「次クリップ」と判定していた。
local PEEK_MIN_HITS = 4

local function normalize_effect_name(name)
	if name == nil then
		return nil
	end
	local s = tostring(name):gsub("^%s+", ""):gsub("%s+$", "")
	if s == "" then
		return nil
	end
	return s
end

--- getvalue フォールバック用の効果名一覧（指定の @*.ptk.obj2 があれば先頭）。
local function effect_candidates(extra)
	local list = {}
	local seen = {}
	local function push(name)
		name = normalize_effect_name(name)
		if name == nil or seen[name] then
			return
		end
		seen[name] = true
		list[#list + 1] = name
		-- 先頭 @ なしも試す（メニュー名 / getvalue 名の差）。
		if name:sub(1, 1) == "@" then
			local bare = name:sub(2)
			if bare ~= "" and not seen[bare] then
				seen[bare] = true
				list[#list + 1] = bare
			end
		end
	end
	push(extra)
	for _, name in ipairs(EFFECT_CANDIDATES) do
		push(name)
	end
	return list
end

--- レイヤーローカル時刻の p1..p16 を読む（nil = 現在）。
--- 優先: layer.pN / layer.track.pN（@*.ptk.obj2 の select@ でも効果名なしで読める）。
--- min_hits: 成功スロットの下限（既定1 = 旧 tracks フォールバック）。
local function read_parts_at(layer, local_time, overwrite_effect, min_hits)
	min_hits = tonumber(min_hits) or 1
	if min_hits < 1 then
		min_hits = 1
	end

	-- 1) layerN.pN / layerN.track.pN / layerN.select.pN
	do
		local tmp, hits = {}, 0
		local prefix = "layer" .. tostring(layer) .. "."
		for i = 1, PART_COUNT do
			local got = nil
			local names = {
				prefix .. "p" .. i,
				prefix .. "track.p" .. i,
				prefix .. "select.p" .. i,
			}
			for _, name in ipairs(names) do
				local ok, v
				if local_time ~= nil then
					ok, v = pcall(obj.getvalue, name, local_time)
				else
					ok, v = pcall(obj.getvalue, name)
				end
				if ok and v ~= nil then
					got = tonumber(v) or 0
					break
				end
			end
			if got ~= nil then
				tmp[i] = got
				hits = hits + 1
			else
				tmp[i] = 0
			end
		end
		if hits >= min_hits then
			return tmp, true, "layer.p"
		end
	end

	-- 2) getvalue(layer, effect, item, time)
	for _, effect in ipairs(effect_candidates(overwrite_effect)) do
		local tmp, hits = {}, 0
		for i = 1, PART_COUNT do
			local got = nil
			for _, item in ipairs(ITEM_NAME_CANDIDATES[i]) do
				local ok, v
				if local_time ~= nil then
					ok, v = pcall(obj.getvalue, layer, effect, item, local_time)
					if (not ok) or v == nil then
						ok, v = pcall(obj.getvalue, layer, effect, item, local_time, 0)
					end
				else
					ok, v = pcall(obj.getvalue, layer, effect, item)
				end
				if ok and v ~= nil then
					got = tonumber(v) or 0
					break
				end
			end
			if got ~= nil then
				tmp[i] = got
				hits = hits + 1
			else
				tmp[i] = 0
			end
		end
		if hits >= min_hits then
			return tmp, true, "effect:" .. tostring(effect)
		end
	end

	return nil, false, nil
end

--- (now_local, now_local+pre_roll] で監視パーツが変わる最初の時刻。
local function find_param_change_time(layer, present, mask, now_local, pre_roll, overwrite_effect)
	if present == nil or pre_roll <= 0 then
		return nil, nil
	end
	local horizon = now_local + pre_roll
	local future, ok, how = read_parts_at(layer, horizon, overwrite_effect)
	if not ok or future == nil then
		return nil, nil
	end
	if not watched_changed(present, future, mask) then
		return nil, how
	end

	local lo, hi = now_local, horizon
	-- 反復上限: 各ステップがフル read_parts_at（AviUtl2 では重い）。
	for _ = 1, 10 do
		local mid = (lo + hi) * 0.5
		local mid_v, mid_ok = read_parts_at(layer, mid, overwrite_effect)
		if mid_ok and mid_v ~= nil and watched_changed(present, mid_v, mask) then
			hi = mid
		else
			lo = mid
		end
		if (hi - lo) < 1e-4 then
			break
		end
	end
	return hi, how
end

--- 現在の上書きクリップのシーン frame_e（と frame_s）。
--- AviUtl2 の getvalue(layerN.frame_e) は当てにならない。パーツ上書き予告の state を優先。
local function read_clip_span(layer, want_id)
	local fr = framerate()
	local fs, fe = nil, nil

	-- 1) 有効上書き側の予告 anm2 が書いた値。
	do
		local st = get_state()
		if type(st.clips) == "table" then
			local c = st.clips[layer] or st.clips["L" .. tostring(layer)]
			if (c == nil or c.frame_e == nil) and want_id ~= nil and want_id ~= "" then
				c = st.clips["id:" .. tostring(want_id)]
			end
			if c ~= nil then
				fs = tonumber(c.frame_s)
				fe = tonumber(c.frame_e)
				if fe == nil and fs ~= nil then
					local tf = tonumber(c.totalframe)
					if tf ~= nil and tf > 0 then
						fe = fs + tf - 1
					end
				end
				if fe ~= nil then
					return fs, fe
				end
			end
		end
	end

	-- 2) フォールバック: 他レイヤー getvalue（AviUtl2 ではしばしば nil）。
	local prefix = "layer" .. tostring(layer) .. "."
	local ok, v = pcall(obj.getvalue, prefix .. "frame_s")
	if ok and v ~= nil then fs = tonumber(v) end
	ok, v = pcall(obj.getvalue, prefix .. "frame_e")
	if ok and v ~= nil then fe = tonumber(v) end
	if fe == nil and fs ~= nil then
		ok, v = pcall(obj.getvalue, prefix .. "totalframe")
		if ok and v ~= nil then
			local tf = tonumber(v)
			if tf ~= nil and tf > 0 then
				fe = fs + tf - 1
			end
		end
	end
	if fe == nil and fs ~= nil then
		ok, v = pcall(obj.getvalue, prefix .. "totaltime")
		if ok and v ~= nil then
			local tt = tonumber(v)
			if tt ~= nil and tt > 0 then
				fe = fs + math.max(0, math.floor(tt * fr + 1e-6) - 1)
			end
		end
	end

	return fs, fe
end

--- 次クリップ切替時刻（このオブジェクトの obj.time 基準）。
--- 現在上書きの終端が未来なら、その境界を返す。
local function find_clip_boundary_change(layer, now_obj_time, want_id)
	local fr = framerate()
	local fs, fe = read_clip_span(layer, want_id)
	if fe == nil then
		return nil, fs, fe
	end
	local my_scene_frame = (tonumber(obj.frame_s) or 0) + (tonumber(obj.frame) or 0)
	-- 次オブジェクトは通常、このクリップ終端の次フレームから。
	local boundary_frame = fe + 1
	local until_frames = boundary_frame - my_scene_frame
	if until_frames <= 0 then
		return nil, fs, fe
	end
	return now_obj_time + until_frames / fr, fs, fe
end

--- レイヤーローカルの変化時刻 → このオブジェクトのローカル時刻。
local function layer_local_to_obj_time(layer, layer_t)
	local fr = framerate()
	local ok, fs = pcall(obj.getvalue, "layer" .. tostring(layer) .. ".frame_s")
	local layer_fs = (ok and fs ~= nil) and (tonumber(fs) or 0) or (tonumber(obj.frame_s) or 0)
	local scene_t = layer_fs / fr + layer_t
	local my_fs = tonumber(obj.frame_s) or 0
	return scene_t - my_fs / fr
end

--- 同一参照レイヤーの次クリップを peek（終端半窓用）。
--- 軽い経路のみ（ov_tt*）。scene_layer / scene_raw は現在値を返しやすく重いので使わない。
--- clip_flip_gates は part_bank 優先。
--- 戻り: values, ok, how
---   ok+past  = 次クリップっぽい別パラメータ
---   ok+same  = 次も同じとみなせる
---   not ok   = 失敗 / 信用できない
local function peek_next_parts(layer, remain, overwrite_effect, curr_values, clip_t, clip_tt)
	if remain == nil or remain < 0 or layer == nil then
		return nil, false, nil
	end
	local fr = framerate()
	local dt = 1.0 / fr

	local candidates = {}
	local function add(tag, tm)
		if tm == nil then
			return
		end
		candidates[#candidates + 1] = { tag = tag, t = tm }
	end

	-- 上書きクリップ終端直後（同レイヤー隙間なし）。
	if clip_tt ~= nil then
		add("ov_tt", clip_tt + dt)
		add("ov_tt2", clip_tt + 2 * dt)
	elseif clip_t ~= nil then
		add("ov_trem", clip_t + remain + dt)
	end

	local all_mask = {}
	for i = 1, PART_COUNT do
		all_mask[i] = true
	end

	local same_hits = 0
	local last_same_how = nil
	for _, c in ipairs(candidates) do
		local v, ok, how = read_parts_at(layer, c.t, overwrite_effect, PEEK_MIN_HITS)
		if ok and v ~= nil then
			local tag = tostring(c.tag) .. "/" .. tostring(how)
			if curr_values ~= nil then
				local any_diff = watched_changed(curr_values, v, all_mask)
				if any_diff then
					return v, true, "past:" .. tag
				end
				same_hits = same_hits + 1
				last_same_how = "same:" .. tag
			else
				return v, true, tag
			end
		end
	end

	if same_hits >= 1 then
		return curr_values, true, last_same_how or "same"
	end
	return nil, false, nil
end

local function quantize_scene(scene_sec)
	local fr = framerate()
	return math.floor((tonumber(scene_sec) or 0) * fr + 0.5)
end

local function copy_part_values(values)
	local out = {}
	for i = 1, PART_COUNT do
		out[i] = values[i]
	end
	return out
end

local function part_bank_layer(state, root_name, layer)
	local root = state[root_name]
	if type(root) ~= "table" then
		root = {}
		state[root_name] = root
	end
	local bank = root[layer]
	if type(bank) ~= "table" then
		bank = {}
		root[layer] = bank
	end
	return bank
end

local function part_bank_end_clear(end_bank, entry, scene_e)
	if scene_e == nil or entry == nil then
		return
	end
	local qe = quantize_scene(scene_e)
	if end_bank[qe] == entry then
		end_bank[qe] = nil
	end
end

local function part_bank_end_set(end_bank, entry)
	if entry == nil or entry.scene_e == nil then
		return
	end
	end_bank[quantize_scene(entry.scene_e)] = entry
end

--- この上書きクリップの p1..p16 を始端シーン時刻で記憶（後の open/close 比較用）。
--- 値が同じなら書き換えず、終端インデックスだけ更新（直前クリップ検索用）。
local function part_bank_put(layer, scene_s, scene_e, values)
	if layer == nil or scene_s == nil or values == nil then
		return
	end
	local state = get_state()
	local bank = part_bank_layer(state, "part_bank", layer)
	local end_bank = part_bank_layer(state, "part_bank_end", layer)
	local q = quantize_scene(scene_s)
	local prev = bank[q]
	if type(prev) == "table" and type(prev.values) == "table" then
		local same = true
		for i = 1, PART_COUNT do
			if prev.values[i] ~= values[i] then
				same = false
				break
			end
		end
		if same then
			local old_e = prev.scene_e
			prev.scene_e = scene_e
			if old_e ~= scene_e then
				part_bank_end_clear(end_bank, prev, old_e)
				part_bank_end_set(end_bank, prev)
			else
				part_bank_end_set(end_bank, prev)
			end
			return
		end
		part_bank_end_clear(end_bank, prev, prev.scene_e)
	end
	local entry = {
		values = copy_part_values(values),
		scene_s = scene_s,
		scene_e = scene_e,
	}
	bank[q] = entry
	part_bank_end_set(end_bank, entry)
end

--- 始端が scene_sec 付近の bank 済みクリップ（次クリップ / close 用）。
local function part_bank_get_near(layer, scene_sec)
	local state = get_state()
	if type(state.part_bank) ~= "table" then
		return nil
	end
	local bank = state.part_bank[layer]
	if type(bank) ~= "table" then
		return nil
	end
	local q = quantize_scene(scene_sec)
	for dq = 0, 3 do
		local e = bank[q + dq]
		if e ~= nil and e.values ~= nil then
			return e
		end
	end
	return nil
end

--- 終端が scene_sec 付近の bank 済みクリップ（直前クリップ / open 用）。
local function part_bank_get_ending_near(layer, scene_sec)
	local state = get_state()
	if type(state.part_bank_end) ~= "table" then
		return nil
	end
	local end_bank = state.part_bank_end[layer]
	if type(end_bank) ~= "table" then
		return nil
	end
	local q = quantize_scene(scene_sec)
	-- 隙間なし: 直前終端 ≈ 現在始端（1〜3フレーム余裕）。
	for dq = 0, 3 do
		local e = end_bank[q - dq]
		if e ~= nil and e.values ~= nil then
			return e
		end
	end
	return nil
end

--- 監視パーツで始端/終端フリップをゲートする。
--- open : react.changed、または直前クリップ bank との差（スクラブ耐性）。
--- close: まず次クリップ bank。なければ peek。bank-same なら close 武装を解除。
--- opts: react, time, remain, totaltime, open_half, close_half, watch_parts, peek_layer
function M.clip_flip_gates(opts)
	opts = opts or {}
	local react = opts.react
	if type(react) ~= "table" or not react.found or react.values == nil then
		return { open = false, close = false, reason = "no-react" }
	end

	local t = tonumber(opts.time)
	local remain = tonumber(opts.remain)
	local clip_tt = tonumber(opts.totaltime)
	if clip_tt == nil and t ~= nil and remain ~= nil then
		clip_tt = t + remain
	end
	local open_half = tonumber(opts.open_half) or 0
	local close_half = tonumber(opts.close_half) or open_half
	if open_half < 0 then open_half = 0 end
	if close_half < 0 then close_half = 0 end

	local watch_mask, watch_label = M.parse_watch_parts(opts.watch_parts)
	local watch_all = (watch_label == "all" or watch_label == "all(fallback)")
	local peek_layer = math.floor(tonumber(opts.peek_layer) or tonumber(react.ref_layer) or 1)
	if peek_layer < 1 then
		peek_layer = 1
	end

	local state = get_state()
	local key = react.key or (tostring(obj.id) .. "#" .. tostring(obj.effect_id))
	local st = state.by_id[key]
	if type(st) ~= "table" then
		st = {}
		state.by_id[key] = st
	end
	-- 旧ビルドの誤 close 印を捨てる。
	st.watch_bounds = nil

	do
		local scene = (tonumber(obj.frame_s) or 0) + (tonumber(obj.frame) or 0)
		if st.flip_last_scene ~= nil then
			local delta = scene - st.flip_last_scene
			if delta < -1 or delta > 2 then
				st.flip_open = false
				st.flip_close = false
				st.flip_close_armed = false
			end
		end
		st.flip_last_scene = scene
	end

	local in_open = (t ~= nil and open_half > 0 and t <= open_half + 1e-6)
	local in_close = (remain ~= nil and close_half > 0 and remain <= close_half + 1e-6)
	local scene_now = scene_time_now()

	-- このクリップのスナップを更新（変化なしなら実質 no-op）。
	if t ~= nil and remain ~= nil then
		part_bank_put(peek_layer, scene_now - t, scene_now + remain, react.values)
	end

	local do_open, do_close = false, false
	local reason = "idle"
	local open_reason = "open"

	if watch_all then
		do_open = in_open
		do_close = in_close
		if do_open and do_close then
			do_close = false
			reason = "clock-all-open"
		elseif do_open then
			reason = "clock-all-open"
		elseif do_close then
			reason = "clock-all-close"
		end
	else
		if react.changed then
			st.flip_open = true
			open_reason = "open"
		end
		-- スクラブで境界に戻ったとき: prev が現在と同じになり得る（例: B→C で監視が同値）。
		-- その場合は bank の直前クリップと比較する。
		if in_open and not st.flip_open and t ~= nil then
			local prev_clip = part_bank_get_ending_near(peek_layer, scene_now - t)
			if prev_clip ~= nil then
				local will, diff = watched_changed(prev_clip.values, react.values, watch_mask)
				if will then
					st.flip_open = true
					open_reason = "bank-open:" .. tostring(diff)
				end
			end
		end
		if not in_open then
			st.flip_open = false
		end
		do_open = in_open and st.flip_open == true

		if in_close then
			local end_scene = scene_now + remain
			local armed, arm_why = false, nil
			local bank_decided = false

			-- 1) bank 優先: 次クリップが以前描画されていればその実値。
			local nxt = part_bank_get_near(peek_layer, end_scene)
			if nxt ~= nil then
				bank_decided = true
				local will, diff = watched_changed(react.values, nxt.values, watch_mask)
				if will then
					armed, arm_why = true, "bank:" .. tostring(diff)
				else
					arm_why = "bank-same"
				end
			end

			-- 2) bank が無いときだけ peek（getvalue 連打を避ける）。
			if not bank_decided then
				local next_v, ok, how = peek_next_parts(
					peek_layer, remain, react.overwrite_effect, react.values, t, clip_tt
				)
				if ok and next_v ~= nil then
					local will, diff = watched_changed(react.values, next_v, watch_mask)
					if will then
						armed, arm_why = true, "peek:" .. tostring(how) .. ":" .. tostring(diff)
					else
						arm_why = "peek-same:" .. tostring(how)
					end
				else
					arm_why = "peek-miss"
				end
			end

			if armed then
				st.flip_close_armed = true
				st.flip_close = true
				reason = arm_why
			elseif arm_why == "bank-same" then
				-- bank で同値と分かったので古い sticky 武装を消す。
				st.flip_close_armed = false
				st.flip_close = false
				reason = arm_why
			elseif st.flip_close_armed then
				st.flip_close = true
				reason = "sticky:" .. tostring(arm_why or "miss")
			else
				st.flip_close = false
				reason = tostring(arm_why or "idle")
			end
			do_close = st.flip_close == true
		else
			st.flip_close = false
			st.flip_close_armed = false
		end

		if do_open and do_close then
			do_close = false
			reason = open_reason
		elseif do_open then
			reason = open_reason
		end
	end

	return {
		open = do_open,
		close = do_close,
		reason = reason,
		watch_label = watch_label,
		watch_all = watch_all,
		peek_layer = peek_layer,
	}
end

--- 現在上書きクリップのローカル時計（SubObjectStates。標準 / @*.ptk.obj2）。
--- 戻り: time, totaltime, remain, source_tag
function M.read_sub_clock(layer, char_id)
	local ok, Sub = pcall(require, "PSDToolKit.SubObjectStates")
	if not ok or Sub == nil then
		return nil, nil, nil, "no-sub"
	end
	local sub = nil
	if char_id ~= nil and char_id ~= "" then
		sub = Sub:get(char_id)
	end
	if sub == nil then
		sub = Sub:get("L" .. tostring(layer))
	end
	if sub == nil then
		return nil, nil, nil, "miss"
	end

	local t = tonumber(sub.time)
	local tt = tonumber(sub.totaltime)
	local rate = framerate()

	if (t == nil or tt == nil) then
		local fr = tonumber(sub.frame)
		local tf = tonumber(sub.totalframe)
		if fr ~= nil and tf ~= nil and tf > 0 then
			t = fr / rate
			tt = tf / rate
		end
	end
	if t == nil or tt == nil then
		return nil, nil, nil, "bad"
	end

	local remain = tt - t
	if remain < 0 then remain = 0 end
	return t, tt, remain, "sub"
end

--- opts: ref_layer, char_id, watch_parts, key, pre_roll, overwrite_effect
--- overwrite_effect: getvalue フォールバック用の任意効果名（空なら自動）
--- pre_roll: パーツ変化の何秒前から elapsed を進めるか（先読み）。現行アニメは 0 推奨。
function M.update(opts)
	opts = opts or {}
	local layer = math.floor(tonumber(opts.ref_layer) or 1)
	if layer < 1 then
		layer = 1
	end
	local want_id = opts.char_id
	if type(want_id) ~= "string" then
		want_id = tostring(want_id or "")
	end
	local overwrite_effect = normalize_effect_name(opts.overwrite_effect)
	local watch_mask, watch_label = M.parse_watch_parts(opts.watch_parts)
	local pre_roll = tonumber(opts.pre_roll) or 0
	if pre_roll < 0 then
		pre_roll = 0
	end
	local key = opts.key
	if key == nil or key == "" then
		key = tostring(obj.id) .. "#" .. tostring(obj.effect_id)
	end

	local values, found, source = nil, false, "none"
	do
		local v, ok, how = read_via_overwriter_states(layer, want_id)
		if ok then
			values, found, source = v, true, "OverwriterStates:" .. tostring(how)
		else
			source = "OverwriterStates:miss(" .. tostring(how) .. ")"
		end
	end
	if not found then
		local v, ok, how = read_parts_at(layer, nil, overwrite_effect)
		if ok then
			values, found, source = v, true, "tracks:" .. tostring(how)
		end
	end

	local state = get_state()
	local st = state.by_id[key]
	if type(st) ~= "table" then
		st = { prev = nil, change_t = nil, lock_t = nil, source = nil, last_scene = nil }
		state.by_id[key] = st
	end

	local now = obj.time
	-- 再生ヘッドが飛んだら sticky をリセット（特に逆スクラブ）。
	do
		local scene = (tonumber(obj.frame_s) or 0) + (tonumber(obj.frame) or 0)
		if st.last_scene ~= nil then
			local delta = scene - st.last_scene
			if delta < -1 or delta > 2 then
				st.prev = nil
				st.change_t = nil
				st.lock_t = nil
			end
		end
		st.last_scene = scene
	end
	local changed, diff = false, ""
	if found then
		changed, diff = watched_changed(st.prev, values, watch_mask)
		if changed then
			-- デバウンス: ちらつきフレームで change_t を毎回上書きしない（open 半窓を守る）。
			local hold = 0.05
			if st.change_t == nil or (now - st.change_t) > hold then
				st.change_t = now
			else
				changed = false
				diff = ""
			end
		end
		st.prev = values
		st.source = source
	else
		st.source = "missing"
	end

	-- 別オブジェクト切替用の先読み。
	-- OverwriterStates は今フレームのみ。クリップ跨ぎの未来読みは当てにならない。
	-- アニメ窓のあいだ change_t を lock し、次クリップの frame_e に上書きされないようにする。
	local lookahead_src = nil
	local clip_fs, clip_fe = nil, nil
	local post_roll = pre_roll
	if found and pre_roll > 0 then
		local locked = st.lock_t ~= nil
			and now >= (st.lock_t - pre_roll - 1e-6)
			and now <= (st.lock_t + post_roll + 1e-6)

		if locked then
			st.change_t = st.lock_t
			lookahead_src = "locked"
			clip_fs, clip_fe = read_clip_span(layer, want_id)
		else
			st.lock_t = nil

			-- 1) 主経路: 現在上書き終端 → 次オブジェクト始端。
			local T_clip
			T_clip, clip_fs, clip_fe = find_clip_boundary_change(layer, now, want_id)
			if T_clip ~= nil and T_clip > now + 1e-6 then
				st.change_t = T_clip
				lookahead_src = "clip_end"
				if now >= (T_clip - pre_roll - 1e-6) then
					st.lock_t = T_clip
				end
			end

			-- 2) 予備: 同一クリップ内のパラメータ/キー変化（通常運用ではない）。
			if lookahead_src == nil then
				local scene_now = scene_time_now()
				local local_now = layer_local_time(layer, scene_now)
				local time_bases = {
					{ t = local_now, to_obj = function(t) return layer_local_to_obj_time(layer, t) end, tag = "local" },
					{ t = now, to_obj = function(t) return t end, tag = "obj" },
				}
				for _, base in ipairs(time_bases) do
					local present_fx, present_ok = read_parts_at(layer, base.t, overwrite_effect)
					if not present_ok then
						present_fx = values
					end
					local T_read, how = find_param_change_time(
						layer, present_fx, watch_mask, base.t, pre_roll, overwrite_effect
					)
					if T_read ~= nil then
						local T_obj = base.to_obj(T_read)
						if T_obj > now + 1e-6 then
							st.change_t = T_obj
							lookahead_src = "param:" .. tostring(how) .. "/" .. base.tag
							if now >= (T_obj - pre_roll - 1e-6) then
								st.lock_t = T_obj
							end
							break
						end
					end
				end
			end
		end
	end

	-- 予測切替に乗ったフレームでは lock_t を change_t に据え置く。
	if changed and st.lock_t ~= nil then
		st.change_t = st.lock_t
	end

	local elapsed = nil
	local until_change = nil
	if st.change_t ~= nil then
		until_change = st.change_t - now
		local start_t = st.change_t - pre_roll
		if now >= start_t then
			elapsed = now - start_t
			if elapsed < 0 then
				elapsed = 0
			end
		end
	end

	return {
		found = found,
		changed = changed,
		diff = diff,
		values = values,
		source = source,
		watch_label = watch_label,
		ref_layer = layer,
		char_id = want_id,
		overwrite_effect = overwrite_effect,
		key = key,
		change_t = st.change_t,
		elapsed = elapsed,
		until_change = until_change,
		pre_roll = pre_roll,
		lookahead = lookahead_src,
		clip_fs = clip_fs,
		clip_fe = clip_fe,
		lock_t = st.lock_t,
		time = now,
	}
end

M.PART_COUNT = PART_COUNT
M.watched_changed = watched_changed
return M
