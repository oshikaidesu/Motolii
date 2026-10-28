// What AbsentMotion's shapes.cpp reads of its scene (its scene.h's Layer), and no more: a layer that is a shape, its
// values by name (as the shape's properties in its shapes.xml say them), no text, no stroke. The shapes.cpp beside it
// is AbsentMotion's own, read as it is (filament/absent_shapes.sh).
#pragma once
#include <map>
#include <string>
#include <vector>
#include "include/core/SkPath.h"
#include "include/core/SkPoint.h"

namespace am {
struct Track { float get(int, float, float fallback) const { return fallback; } };
struct Stroke { bool present = false, enabled = false; Track size; };
struct Layer {
    void* text = nullptr;
    std::string kind = "shape", shape;
    int innerWidth = 0, innerHeight = 0;
    Stroke stroke;
    std::map<std::string, std::vector<float>> values;
    float value(const char* name, int i, float, float fallback) const {
        const auto v = values.find(name);
        return v != values.end() && i < (int)v->second.size() ? v->second[i] : fallback;
    }
};
SkPath ShapePath(const Layer& layer, float t);
SkPath StrokePath(const Layer& layer, const SkPath& path, float t);
inline SkPath OutlineAt(const Layer&, float) { return SkPath(); }
}
