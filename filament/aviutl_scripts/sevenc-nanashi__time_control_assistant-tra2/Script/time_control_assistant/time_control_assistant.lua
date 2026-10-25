local M = {}

---@param name string
---@param value unknown
---@return number
function M.assert_positive(name, value)
	assert(
		type(value) == "number" and value > 0 and value < math.huge,
		("%sは0より大きい有限の数である必要があります"):format(name)
	)
	return value
end

---@param obj_getpoint function
---@return number
local function get_total_duration(obj_getpoint)
	return M.assert_positive("時間制御の長さ", obj_getpoint("time", obj_getpoint("num")))
end

---@param obj_getpoint function
---@return number
local function get_current_time(obj_getpoint)
	local left_section_index, section_frac = math.modf(obj_getpoint("index"))
	local left_time = obj_getpoint("time", left_section_index)
	local right_time = obj_getpoint("time", left_section_index + 1)
	return left_time + section_frac * (right_time - left_time)
end

---@param obj_getpoint function
---@param duration number
---@param offset number
---@param position number
---@param reverse_output? boolean
---@return number
local function apply_position(obj_getpoint, duration, offset, position, reverse_output)
	M.assert_positive("周期", duration)

	---@type number
	local left = obj_getpoint(0)
	---@type number
	local right = obj_getpoint(1)
	local total_duration = get_total_duration(obj_getpoint)
	local left_value = duration / total_duration * left + (offset / total_duration * 100)
	local right_value = duration / total_duration * right + (offset / total_duration * 100)
	local timecontrol_value = obj_getpoint("timecontrol", "value", position * total_duration)

	if reverse_output then
		return right_value + (left_value - right_value) * timecontrol_value
	end
	return left_value + (right_value - left_value) * timecontrol_value
end

---@param obj_getpoint function
---@param duration number
---@param offset number
---@return number
function M.do_repeat(obj_getpoint, duration, offset)
	local current_repeating_position = ((get_current_time(obj_getpoint) - offset) / duration) % 1
	return apply_position(obj_getpoint, duration, offset, current_repeating_position)
end

---@param obj_getpoint function
---@param duration number
---@param offset number
---@return number
function M.do_reverse_forthback(obj_getpoint, duration, offset)
	local position = ((get_current_time(obj_getpoint) - offset) / duration) % 2
	if position > 1 then
		position = 2 - position
	end

	return apply_position(obj_getpoint, duration, offset, position)
end

---@param obj_getpoint function
---@param duration number
---@param offset number
---@return number
function M.do_repeat_forthback(obj_getpoint, duration, offset)
	local position = ((get_current_time(obj_getpoint) - offset) / duration) % 2
	local reverse_output = position >= 1
	return apply_position(obj_getpoint, duration, offset, position % 1, reverse_output)
end

---@class time_control_assistant.bpm
---@field tempo number
---@field beat number
---@field start number
---@field offset number

---@param bpm time_control_assistant.bpm
---@param index integer
---@return number
local function get_measure_duration(bpm, index)
	local tempo = M.assert_positive(("BPMグリッド[%d]のBPM"):format(index), bpm.tempo)
	local beat = M.assert_positive(("BPMグリッド[%d]の拍数"):format(index), bpm.beat)
	return 60 * beat / tempo
end

---@param bpm_list time_control_assistant.bpm[]
---@param project_time number
---@return number measure_position
---@return number measure_duration
local function get_measure_position(bpm_list, project_time)
	assert(type(bpm_list) == "table" and #bpm_list > 0, "BPMグリッドが設定されていません")

	local first_bpm = bpm_list[1]
	assert(type(first_bpm.start) == "number", "BPMグリッド[1]の開始位置が不正です")
	assert(type(first_bpm.offset) == "number", "BPMグリッド[1]の拍子オフセットが不正です")

	local measure_duration = get_measure_duration(first_bpm, 1)
	if project_time < first_bpm.start then
		return (project_time - first_bpm.start - first_bpm.offset) / measure_duration, measure_duration
	end

	local measure_position = -first_bpm.offset / measure_duration
	local segment_start = first_bpm.start

	for index = 2, #bpm_list do
		local bpm = bpm_list[index]
		assert(type(bpm.start) == "number", ("BPMグリッド[%d]の開始位置が不正です"):format(index))
		assert(
			type(bpm.offset) == "number",
			("BPMグリッド[%d]の拍子オフセットが不正です"):format(index)
		)
		assert(bpm.start >= segment_start, "BPMグリッドの開始位置は昇順である必要があります")

		if project_time < bpm.start then
			break
		end

		measure_position = measure_position + (bpm.start - segment_start) / measure_duration
		measure_duration = get_measure_duration(bpm, index)
		local measure_fraction = -bpm.offset / measure_duration
		local measure_index = math.floor(measure_position - measure_fraction + 0.5)
		measure_position = measure_index + measure_fraction
		segment_start = bpm.start
	end

	measure_position = measure_position + (project_time - segment_start) / measure_duration
	return measure_position, measure_duration
end

---@param bpm_list time_control_assistant.bpm[]
---@param project_time number
---@return number
local function get_next_measure_time(bpm_list, project_time)
	local measure_position, measure_duration = get_measure_position(bpm_list, project_time)
	local next_measure_time =
		project_time + (math.ceil(measure_position - 1e-9) - measure_position) * measure_duration

	for _, bpm in ipairs(bpm_list) do
		if bpm.start > project_time and bpm.start <= next_measure_time then
			measure_position, measure_duration = get_measure_position(bpm_list, bpm.start)
			next_measure_time =
				bpm.start + (math.ceil(measure_position - 1e-9) - measure_position) * measure_duration
		end
	end

	return next_measure_time
end

---@param obj_getpoint function
---@param obj_getinfo function
---@param divisions number
---@param units_per_cycle number
---@param divisions_offset number
---@param repeat_from_measure_start boolean
---@return number duration
---@return number offset
---@return number position
local function get_bpm_grid_cycle(
	obj_getpoint,
	obj_getinfo,
	divisions,
	units_per_cycle,
	divisions_offset,
	repeat_from_measure_start
)
	M.assert_positive("分割数", divisions)
	M.assert_positive("分割後の個数", units_per_cycle)

	local framerate = M.assert_positive("フレームレート", obj_getpoint("framerate"))
	local object_start_time = obj_getpoint("frame_s") / framerate
	local project_time = object_start_time + get_current_time(obj_getpoint)
	local bpm_list = obj_getinfo("bpm_list")
	local measure_position, measure_duration = get_measure_position(bpm_list, project_time)
	local divided_duration = measure_duration / divisions
	local duration = divided_duration * units_per_cycle
	local position = (measure_position * divisions - divisions_offset) / units_per_cycle
	local offset = divisions_offset * divided_duration

	if repeat_from_measure_start then
		offset = offset + get_next_measure_time(bpm_list, object_start_time) - object_start_time
	end

	return duration, offset, position
end

---@param obj_getpoint function
---@param obj_getinfo function
---@param divisions number
---@param units_per_cycle number
---@param divisions_offset number
---@param repeat_from_measure_start boolean
---@return number
function M.do_repeat_by_bpm_grid(
	obj_getpoint,
	obj_getinfo,
	divisions,
	units_per_cycle,
	divisions_offset,
	repeat_from_measure_start
)
	local duration, offset, position = get_bpm_grid_cycle(
		obj_getpoint,
		obj_getinfo,
		divisions,
		units_per_cycle,
		divisions_offset,
		repeat_from_measure_start
	)
	return apply_position(obj_getpoint, duration, offset, position % 1)
end

---@param obj_getpoint function
---@param obj_getinfo function
---@param divisions number
---@param units_per_cycle number
---@param divisions_offset number
---@param repeat_from_measure_start boolean
---@return number
function M.do_reverse_forthback_by_bpm_grid(
	obj_getpoint,
	obj_getinfo,
	divisions,
	units_per_cycle,
	divisions_offset,
	repeat_from_measure_start
)
	local duration, offset, position = get_bpm_grid_cycle(
		obj_getpoint,
		obj_getinfo,
		divisions,
		units_per_cycle,
		divisions_offset,
		repeat_from_measure_start
	)
	position = position % 2
	if position > 1 then
		position = 2 - position
	end
	return apply_position(obj_getpoint, duration, offset, position)
end

---@param obj_getpoint function
---@param obj_getinfo function
---@param divisions number
---@param units_per_cycle number
---@param divisions_offset number
---@param repeat_from_measure_start boolean
---@return number
function M.do_repeat_forthback_by_bpm_grid(
	obj_getpoint,
	obj_getinfo,
	divisions,
	units_per_cycle,
	divisions_offset,
	repeat_from_measure_start
)
	local duration, offset, position = get_bpm_grid_cycle(
		obj_getpoint,
		obj_getinfo,
		divisions,
		units_per_cycle,
		divisions_offset,
		repeat_from_measure_start
	)
	position = position % 2
	local reverse_output = position >= 1
	return apply_position(obj_getpoint, duration, offset, position % 1, reverse_output)
end

---@param obj_getpoint function
---@param count number
---@param count_offset number
---@return number
function M.do_repeat_by_count(obj_getpoint, count, count_offset)
	M.assert_positive("回数", count)
	local duration = get_total_duration(obj_getpoint) / count
	return M.do_repeat(obj_getpoint, duration, count_offset * duration)
end

---@param obj_getpoint function
---@param count number
---@param count_offset number
---@return number
function M.do_reverse_forthback_by_count(obj_getpoint, count, count_offset)
	M.assert_positive("回数", count)
	local duration = get_total_duration(obj_getpoint) / count
	return M.do_reverse_forthback(obj_getpoint, duration, count_offset * duration)
end

---@param obj_getpoint function
---@param count number
---@param count_offset number
---@return number
function M.do_repeat_forthback_by_count(obj_getpoint, count, count_offset)
	M.assert_positive("回数", count)
	local duration = get_total_duration(obj_getpoint) / count
	return M.do_repeat_forthback(obj_getpoint, duration, count_offset * duration)
end

---@param obj_getpoint function
---@param start_frame number
---@param end_frame number
---@return number
function M.do_absolute(obj_getpoint, start_frame, end_frame)
	local obj_start_frame = obj_getpoint("frame_s")
	local obj_end_frame = obj_getpoint("frame_e")
	local obj_frame_duration = obj_end_frame - obj_start_frame

	local target_0_value = (start_frame - obj_start_frame) / obj_frame_duration * 100
	local target_100_value = (end_frame - obj_start_frame) / obj_frame_duration * 100

	local index, frac = math.modf(obj_getpoint("index"))
	local frac_timecontrol = obj_getpoint("timecontrol", "value", frac * obj_getpoint("time", obj_getpoint("num")))

	local current_section_left = obj_getpoint(index)
	local current_section_right = obj_getpoint(index + 1)

	local current_section_value = current_section_left
		+ (current_section_right - current_section_left) * frac_timecontrol

	return target_0_value + (target_100_value - target_0_value) * (current_section_value / 100)
end

return M
