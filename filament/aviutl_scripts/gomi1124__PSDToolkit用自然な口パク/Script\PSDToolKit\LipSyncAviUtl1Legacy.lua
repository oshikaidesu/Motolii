-- Exact AviUtl1-compatible lip sync state detection.

local LipSyncAviUtl1Legacy = {}
LipSyncAviUtl1Legacy.__index = LipSyncAviUtl1Legacy

local PSDToolKit = require("PSDToolKit")

local function get_shared_voice(id)
	return PSDToolKit.get_shared_voice(id)
end

local pattern_keys = {
	"閉じ~ptkl",
	"ほぼ閉じ~ptkl",
	"半開き~ptkl",
	"ほぼ開き~ptkl",
	"開き~ptkl",
}

local service = PSDToolKit.__aviutl1_legacy_lipsync_service
if not service then
	service = {
		native_module = nil,
		native_error = nil,
	}
	PSDToolKit.__aviutl1_legacy_lipsync_service = service
end

local function validate_patterns(patterns)
	if #patterns < 2 then
		error("LipSyncAviUtl1Legacy requires at least 2 patterns (closed and open)")
	end
end

local function get_native_module()
	if service.native_module then
		return service.native_module
	end

	local ok, native = pcall(obj.module, "LipSyncAviUtl1")
	if not ok or not native then
		local detail = ok and "module not found" or tostring(native)
		service.native_error = detail
		error("LipSyncAviUtl1.mod2 is unavailable: " .. detail)
	end

	service.native_module = native
	service.native_error = nil
	return native
end

local function get_native_state(voice, frame_rate, lipsync)
	local native = get_native_module()
	local get_state = native.get_state
	if not get_state then
		service.native_error = "native.get_state is unavailable"
		error("LipSyncAviUtl1.mod2 does not provide AviUtl1-compatible state detection")
	end
	local ok, state = pcall(
		get_state,
		voice.audio,
		voice.time,
		frame_rate,
		lipsync.locut,
		lipsync.hicut,
		lipsync.threshold,
		lipsync.sensitivity,
		lipsync.speed,
		#lipsync.patterns
	)
	if not ok then
		service.native_error = tostring(state)
		error("LipSyncAviUtl1 compatible state detection failed: " .. tostring(state))
	end

	state = tonumber(state)
	if not state then
		service.native_error = "native module returned a non-numeric state"
		error("LipSyncAviUtl1 compatible state detection returned an invalid value")
	end
	state = math.floor(state)
	if state < 0 or state >= #lipsync.patterns then
		service.native_error = "native module returned an out-of-range state"
		error("LipSyncAviUtl1 compatible state detection returned an out-of-range value")
	end

	service.native_error = nil
	return state
end

function LipSyncAviUtl1Legacy.clear()
	service.native_error = nil
end

function LipSyncAviUtl1Legacy.new(opts)
	if not opts then
		error("opts cannot be nil")
	end

	local patterns = {}
	for _, key in ipairs(pattern_keys) do
		local value = opts[key]
		if value and value ~= "" then
			table.insert(patterns, value)
		end
	end
	validate_patterns(patterns)

	local sensitivity = math.floor(tonumber(opts["感度"]) or 1)
	if sensitivity < 1 then
		sensitivity = 1
	end

	local speed = tonumber(opts["速さ"]) or 1
	if speed < 0 then
		speed = 0
	end

	return setmetatable({
		patterns = patterns,
		locut = tonumber(opts["ローカット"]) or 100,
		hicut = tonumber(opts["ハイカット"]) or 1000,
		threshold = tonumber(opts["しきい値"]) or 20,
		sensitivity = sensitivity,
		speed = speed,
		alwaysapply = (tonumber(opts["発声がなくても有効"]) or 0) ~= 0,
	}, LipSyncAviUtl1Legacy)
end

function LipSyncAviUtl1Legacy:getstate(ctx)
	validate_patterns(self.patterns)
	local obj = ctx.obj
	local voice_id
	if ctx.psd and ctx.psd.character_id and ctx.psd.character_id ~= "" then
		voice_id = ctx.psd.character_id
	else
		voice_id = obj.layer
	end

	local voice = get_shared_voice(voice_id)
	if not voice or not voice.audio or voice.audio == "" then
		voice = ctx:get_voice(voice_id)
	end
	if not voice or not voice.audio or voice.audio == "" then
		if self.alwaysapply then
			return self.patterns[1]
		end
		return ""
	end

	local state = get_native_state(voice, obj.framerate, self)
	return self.patterns[state + 1]
end

function LipSyncAviUtl1Legacy.get_diagnostics()
	local version = nil
	if service.native_module then
		local ok, value = pcall(service.native_module.get_version)
		if ok then
			version = value
		end
	end
	return {
		backend = "LipSyncAviUtl1.mod2 AviUtl1-compatible spectral detection",
		native_version = version,
		native_error = service.native_error,
	}
end

return LipSyncAviUtl1Legacy
