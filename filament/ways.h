// Ways by name: Cavalry's Magic Easing names, each an expression of u (how far through the stretch, 0 to 1) that
// gives how far the value has gone. One table for both: a key's stretch (doc.h, worked out by SQLite) and a script's
// ease(name, u) (script.cpp, the same text run as JavaScript), so they cannot drift apart. Only what both have:
// pow, exp, cos, abs, pi().
#pragma once
static const struct { const char* name; const char* goes; } named_ways[] = {
    {"SlowIn", "pow(u, 3)"},
    {"SlowOut", "1 - pow(1 - u, 3)"},
    {"SlowInSlowOut", "u * u * u * (u * (6 * u - 15) + 10)"},
    {"VerySlowIn", "pow(u, 5)"},
    {"VerySlowOut", "1 - pow(1 - u, 5)"},
    {"SpringOut", "1 - exp(-5 * u) * cos(3 * pi() * u) * (1 - u)"},
    {"SmallSpringOut", "1 - exp(-7 * u) * cos(2 * pi() * u) * (1 - u)"},
    {"BounceOut", "1 - abs(cos(3.5 * pi() * u)) * pow(1 - u, 2)"},
    {"Overshoot", "1 + 2.70158 * pow(u - 1, 3) + 1.70158 * pow(u - 1, 2)"},
};
