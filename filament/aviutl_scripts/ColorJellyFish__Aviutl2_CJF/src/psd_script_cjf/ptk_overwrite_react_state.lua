-- パーツ上書き連動の状態（require キャッシュ）
local M = {
	by_id = {},
	last_log_frame = {},
	clips = {},
	part_bank = {},
	part_bank_end = {},
	-- フェード@psd 用: [key]={data,w,h,values}
	fade_snap = {},
	fade_last_scene = {},
	fade_force_open = {},
}
return M
