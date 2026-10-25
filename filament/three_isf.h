// A three.js screen effect (examples/jsm/shaders/*.js) read as an ISF effect is (three_isf.cpp, then isf.cpp).
#pragma once
#include <string>

// Whether the file is one: a .js that says a fragmentShader.
bool three_is(const std::string& file);
// Its ISF text ("" and `why` if it cannot be).
std::string three_isf(const std::string& file, std::string& why);
