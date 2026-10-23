// An ISF effect file (.fs) read as a .wgsl effect is (isf.cpp): the file to read for it, made once.
#pragma once
#include <string>

// The .wgsl to read for an effect file: itself if not ISF; for a .fs the one made of it ("" if it could not be).
std::string isf_file(const std::string& file);
