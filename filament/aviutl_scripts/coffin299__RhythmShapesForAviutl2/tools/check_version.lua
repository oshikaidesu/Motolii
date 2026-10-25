local version = dofile("script/RhythmShapes/version.lua")
if type(version) ~= "string" or not version:match("^%d+%.%d+%.%d+$") then
    io.stderr:write("version.lua is not MAJOR.MINOR.PATCH\n")
    os.exit(1)
end

local function read_all(path)
    local f = assert(io.open(path, "rb"))
    local text = f:read("*a")
    f:close()
    return text
end

local obj = read_all("script/RhythmShapes/RhythmShapes.obj2")
local marker = "--information:RhythmShapes v" .. version
if not obj:find(marker, 1, true) then
    io.stderr:write("obj2 information does not match " .. version .. "\n")
    os.exit(1)
end
if not obj:find("--track@count:個数,1,64,24,1", 1, true) then
    io.stderr:write("shape count track does not match the host cap\n")
    os.exit(1)
end

local changelog = read_all("CHANGELOG.md")
if not changelog:find("## [" .. version .. "]", 1, true) then
    io.stderr:write("CHANGELOG.md is missing [" .. version .. "]\n")
    os.exit(1)
end

local required = {
    "script/RhythmShapes/RhythmShapes.obj2",
    "script/RhythmShapes/version.lua",
    "script/RhythmShapes/rs_util.lua",
    "script/RhythmShapes/rs_audio.lua",
    "script/RhythmShapes/rs_beat.lua",
    "script/RhythmShapes/rs_grid.lua",
    "script/RhythmShapes/rs_motion.lua",
    "script/RhythmShapes/rs_layout.lua",
    "script/RhythmShapes/rs_figure.lua",
    "script/RhythmShapes/rs_shape.lua",
    "script/RhythmShapes/rs_preset.lua",
    "script/RhythmShapes/rs_host.lua",
    "README.md",
    "INSTALL.md",
    "CHANGELOG.md",
    "LICENSE",
}
for i = 1, #required do
    local f = io.open(required[i], "rb")
    if not f then
        io.stderr:write("missing " .. required[i] .. "\n")
        os.exit(1)
    end
    f:close()
end

io.write("version " .. version .. " ok\n")
