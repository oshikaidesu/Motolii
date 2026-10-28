// Each line read, "<s> <name>=<v>[,<v>] ...": the path AbsentMotion's own ShapePath makes of a shape so (its s and its
// properties, as its shapes.xml says them), written as SVG by Skia's SkParsePath, one line each.
#include <iostream>
#include <sstream>
#include "scene.h"
#include "include/core/SkString.h"
#include "include/utils/SkParsePath.h"

int main() {
    for (std::string line; std::getline(std::cin, line);) {
        std::istringstream words(line);
        am::Layer layer;
        words >> layer.shape;
        for (std::string said; words >> said;) {
            const size_t eq = said.find('=');
            std::istringstream numbers(said.substr(eq + 1));
            for (std::string n; std::getline(numbers, n, ',');) layer.values[said.substr(0, eq)].push_back(std::stof(n));
        }
        std::cout << SkParsePath::ToSVGString(am::ShapePath(layer, 0)).c_str() << "\n";
    }
}
