// An ISF effect file (.fs) read as a .wgsl effect is (isf.cpp): the file to read for it, made once.
#pragma once
#include <filesystem>
#include <regex>
#include <string>

// The .wgsl to read for an effect file: itself if not ISF; for a .fs the one made of it ("" if it could not be).
std::string isf_file(const std::string& file);

// What an ISF effect is, read from it, nobody asked to write it (the user's Mercari: a seller is not asked, the item is
// read; Shadertoy flags its shaders from what they define and filters by it): its tags with spaces between, of
// transition generator sound multipass feedback time image ("" none, or not ISF).
std::string isf_tags(const std::string& file);

// An effect put on a thing again (an effect card's Duplicate, doc_fx_duplicate: "Glow~2.fs" beside "Glow.fs"): the file
// it is a copy of, read as that one is; its values its own, a value's name being its file's (doc_screen_knob). Any
// other: the file as it is.
inline std::string fx_source(const std::string& f) {
    std::error_code wrong;
    if (std::filesystem::exists(f, wrong)) return f;
    static const std::regex copy(R"((.*)~[0-9]+(\.[^./]+)$)");
    std::smatch m;
    return std::regex_match(f, m, copy) ? m[1].str() + m[2].str() : f;
}
