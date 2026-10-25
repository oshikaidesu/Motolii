// What easing_equations.h asks of Godot's core/math/math_funcs.h: real_t and Math's PI, sin, cos, pow, sqrt, as the C++ library has them.
#pragma once
#include <cmath>
using real_t = double;
namespace Math {
inline constexpr double PI = 3.1415926535897932384626433833;
using std::cos, std::pow, std::sin, std::sqrt;
}
