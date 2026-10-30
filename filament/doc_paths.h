// doc_paths.h: part of doc.h (a path shape's plane and points). Read only through doc.h.
// A path shape's own plane as the picture shows it: its middle and its two ways, seen; a point of the picture on it
// (own: its height 1, from its middle, down as SVG goes) and back (at).
struct DocPlane {
    math::double2 o, ex, ey;
    double det;
    bool seen;
    DocPlane(Doc& d, int thing, double now) {
        const math::mat4 at = doc_shown_at(d, thing, now);
        o = doc_seen(d, (at * math::double4{0, 0, 0, 1}).xyz), ex = doc_seen(d, (at * math::double4{1, 0, 0, 1}).xyz) - o, ey = doc_seen(d, (at * math::double4{0, 1, 0, 1}).xyz) - o;
        det = ex.x * ey.y - ex.y * ey.x, seen = std::fabs(det) > 1e-9;
    }
    math::double2 own(double px, double py) const { const double u = px - o.x, v = py - o.y; return {(u * ey.y - v * ey.x) / det, -(ex.x * v - ex.y * u) / det}; }
    math::double2 at(double fx, double fy) const { return o + ex * fx - ey * fy; }
};
int sk_svg_anchors(const char* data, std::vector<float>& xy);
int sk_svg_ends(const char* data, float& fx, float& fy, int& stretches);
bool sk_svg_move(const char* data, int nth, float dx, float dy, std::string& out);
int sk_svg_handles(const char* data, std::vector<float>& xy, std::vector<float>* of = nullptr);
bool sk_svg_move_handle(const char* data, int nth, float dx, float dy, std::string& out);
int sk_svg_edit(const char* data, float x, float y, float reach, std::string& out);
// A path shape's points as the picture shows them (x, y two a point, points of the picture from the top left; its handles not).
// handles: its curves' handles instead; of: and the point each is the handle of (of[2i], of[2i+1]).
static int doc_points(Doc& d, int thing, double now, double* xy, int room, bool handles = false, double* of = nullptr) {
    if (!doc_knows(d, thing) || doc_word(d, thing, "shape") != "path") return 0;
    std::vector<float> own, owner;
    const std::string path = doc_word(d, thing, "path");
    const int n = handles ? sk_svg_handles(path.c_str(), own, &owner) : sk_svg_anchors(path.c_str(), own);
    const DocPlane plane(d, thing, now);
    for (int i = 0; i < n && i < room; ++i) {
        const math::double2 p = plane.at(own[2 * i], own[2 * i + 1]);
        xy[2 * i] = p.x, xy[2 * i + 1] = p.y;
        if (of) { const math::double2 o = plane.at(owner[2 * i], owner[2 * i + 1]); of[2 * i] = o.x, of[2 * i + 1] = o.y; }
    }
    return n;
}
// A pen on the stage: a point put where the hand is (x, y: points of the picture from its top left), its handle drawn
// out by hx, hy (points; 0, 0: a corner), on a path shape's own path (its `path`, SVG's path data): a line to it, or a
// curve through it as Illustrator's pen makes one (SVG's S: the point before smooth, its handle turned through). On
// its first point again: closed; then on one of its points that point is taken away, on its outline a point is put
// there, elsewhere another is begun (Illustrator's pen adds and deletes so). thing 0: a new path shape, begun there, one step with its
// first point. Its number (0: not).
static int doc_pen(Doc& d, int thing, double x, double y, double hx, double hy) {
    const double now = d.stage->now;
    doc_act(d);
    if (thing && (!doc_knows(d, thing) || doc_word(d, thing, "shape") != "path")) thing = 0;   // (not a path: the pen begins one)
    if (const std::string was = thing ? doc_word(d, thing, "path") : ""; [&] { float fx, fy; int n; return sk_svg_ends(was.c_str(), fx, fy, n) == 4; }()) {   // (closed: on one of its points, it is taken away; on its outline, one is put there; else the pen begins another)
        const DocPlane plane(d, thing, now);
        const math::double2 p = plane.own(x, y);
        std::string edited;
        if (plane.seen && sk_svg_edit(was.c_str(), (float)p.x, (float)p.y, (float)(8.0 / std::max(1e-9, length(plane.ex))), edited)) {
            if (!doc_say(d, thing, "path", edited.c_str())) d.act_ok = false;
            return doc_acted(d) ? thing : 0;
        }
        thing = 0;
    }
    if (!thing) thing = doc_add(d, "shape", "path");
    std::string path = thing ? doc_word(d, thing, "path") : "";
    const DocPlane plane(d, thing, now);
    const math::double2 o = plane.o, ex = plane.ex, ey = plane.ey;
    if (thing && !plane.seen) d.trouble = "the shape is seen edge on", thing = 0;
    const auto own = [&](double px, double py) { return plane.own(px, py); };
    char said[200] = "";
    if (thing) {
        const math::double2 p = own(x, y), in = own(x - hx, y - hy);
        float first_x = 0.f, first_y = 0.f;
        int stretches = 0;
        const int last = sk_svg_ends(path.c_str(), first_x, first_y, stretches);   // (Skia's reading of it: skpath.cpp)
        const double fx = first_x, fy = first_y;
        const bool smooth = last == 3, handle = hx != 0.0 || hy != 0.0;
        const bool closes = stretches > 0 && last != 4 && length(o + ex * fx - ey * fy - math::double2{x, y}) < 8.0;   // (back on its first point, as the hand sees it)
        if (path.empty()) std::snprintf(said, sizeof said, "M%.4f %.4f", p.x, p.y);
        else if (closes) std::snprintf(said, sizeof said, smooth || handle ? " S%.4f %.4f %.4f %.4f Z" : " Z", handle ? own(o.x + ex.x * fx - ey.x * fy - hx, o.y + ex.y * fx - ey.y * fy - hy).x : fx,
                                       handle ? own(o.x + ex.x * fx - ey.x * fy - hx, o.y + ex.y * fx - ey.y * fy - hy).y : fy, fx, fy);
        else if (!smooth && !handle) std::snprintf(said, sizeof said, " L%.4f %.4f", p.x, p.y);
        else std::snprintf(said, sizeof said, " S%.4f %.4f %.4f %.4f", in.x, in.y, p.x, p.y);
        if (!doc_say(d, thing, "path", (path + said).c_str())) thing = 0;
    }
    if (!thing) d.act_ok = false;
    return doc_acted(d) ? thing : 0;
}
// A film's sound taken out to a thing of its own (Alight Motion's Extract Audio): a copy of it, hidden, that is heard
// (a hidden film's sound is: doc_match.h sounding), named so; the film itself no longer heard (volume 0). One step; the
// answer is the copy.
static int doc_extract_sound(Doc& d, int thing) {
    if (!doc_knows(d, thing) || doc_word(d, thing, "kind") != "film") return d.trouble = "only a film's sound is taken out", 0;
    doc_act(d);
    const int made = doc_duplicate(d, thing);
    const bool ok = made && doc_set(d, made, "visible", 0) && doc_say(d, made, "name", (doc_name(d, thing) + " (sound)").c_str()) && doc_set(d, thing, "volume", 0);
    return doc_acted(d) && ok ? made : 0;
}
// A path drawn by hand: the outline perfect-freehand makes round the hand's way (MIT, Steve Ruiz; its Dart port, Adil
// Hanney: the window's), in points of the picture, made a closed path of its own, each point to the next by a quadratic
// curve through their middle, as its example draws the outline. One step; the answer is the new shape.
static int doc_freehand(Doc& d, const char* points) {
    std::vector<double> xy;
    std::istringstream said(points);
    for (double v; said >> v;) xy.push_back(v);
    if (xy.size() < 6) return d.trouble = "a stroke of fewer than three points", 0;
    doc_act(d);
    int thing = doc_add(d, "shape", "path");
    const DocPlane plane(d, thing, d.stage->now);
    if (thing && !plane.seen) d.trouble = "its plane is seen edge on", thing = 0;
    if (thing) {
        std::string path;
        char one[160];
        const size_t n = xy.size() / 2;
        for (size_t i = 0; i < n; ++i) {
            const math::double2 p = plane.own(xy[2 * i], xy[2 * i + 1]), q = plane.own(xy[2 * ((i + 1) % n)], xy[2 * ((i + 1) % n) + 1]);
            if (!i) std::snprintf(one, sizeof one, "M%.4f %.4f", p.x, p.y), path += one;
            std::snprintf(one, sizeof one, " Q%.4f %.4f %.4f %.4f", p.x, p.y, (p.x + q.x) / 2, (p.y + q.y) / 2);
            path += one;
        }
        if (!doc_say(d, thing, "path", (path + " Z").c_str())) thing = 0;
    }
    if (!thing) d.act_ok = false;
    return doc_acted(d) ? thing : 0;
}
// A brush's stroke as the window drew it (perfect-freehand's points, each with the pressure its radius comes from): a
// path shape whose stroke is kept as drawn (`drawing`: brush, how wide, each point and its pressure, in the plane's own
// units) and whose outline AbsentMotion makes of it (absent_drawing: Alight Motion's brush, a band of round pieces, its
// drawing effects on it as time goes: doc_moment). tool: AbsentMotion's (0 a pen, as wide all along; 1 a brush, as its
// pressure says; 2 a fill, the way gone round filled; 3 an eraser: a stroke of `into`'s own, taking from what it has
// drawn, as AbsentMotion's eraser clears within its drawing layer); size: how wide, in points of the picture; then x y
// pressure each. The shape it made or took from.
static int doc_brush(Doc& d, const char* said, int into = 0) {
    std::istringstream in(said);
    int tool = 1;
    double size = 0.0;
    in >> tool >> size;
    tool = std::clamp(tool, 0, 3);
    std::vector<double> xyp;
    for (double v; in >> v;) xyp.push_back(v);
    if (!(size > 0.0) || xyp.size() < 6) return d.trouble = "a stroke of fewer than two points", 0;
    const std::string was = tool == 3 ? doc_word(d, into, "drawing") : std::string();
    if (tool == 3 && was.empty()) return d.trouble = "an eraser takes from a drawing: pick one", 0;
    doc_act(d);
    int thing = tool == 3 ? into : doc_add(d, "shape", "path");
    const DocPlane plane(d, thing, d.stage->now);
    if (thing && !plane.seen) d.trouble = "its plane is seen edge on", thing = 0;
    if (thing) {
        const double wide = length(plane.own(xyp[0] + size, xyp[1]) - plane.own(xyp[0], xyp[1]));
        std::string kept = was.empty() ? "" : was + " ; ";
        char one[96];
        std::snprintf(one, sizeof one, "%d %.5f", tool, wide), kept += one;
        for (size_t i = 0; i + 2 < xyp.size(); i += 3) {
            const math::double2 p = plane.own(xyp[i], xyp[i + 1]);
            std::snprintf(one, sizeof one, " %.5f %.5f %.3f", p.x, p.y, xyp[i + 2]), kept += one;
        }
        const std::string outline = absent_drawing(kept, {}, thing, 0.0, 1000.0, 0.0, true);
        if ((outline.empty() && tool != 3) || !doc_say(d, thing, "drawing", kept.c_str()) || !doc_say(d, thing, "path", outline.c_str())) thing = 0;
    }
    if (!thing) d.act_ok = false;
    return doc_acted(d) ? thing : 0;
}
// A flat shape made a path shape of the outline it shows (Alight Motion's Convert to Outline): its form drawn again as
// it is drawn (flat_path, its corners rounded, puckered, zigzagged and offset by ThorVG's own path operators: tvg_bend,
// as cut_at bends it), its points then its own, in its height from its middle (SVG's way down); those operators then
// none, made part of its path. Its fill, edge and all else as they were.
static bool doc_to_path(Doc& d, int thing) {
    const std::string form = doc_word(d, thing, "shape");
    if (doc_word(d, thing, "kind") != "shape" || form.empty() || form == "path" || form == "null" || form == "solid") return d.trouble = "only a shape of a kind of its own is made a path (not a path, a null or a solid)", false;
    const bool line = form == "line", wave = form == "wave";
    const double high = line ? 0.04 : wave ? 0.3 : 1.0, wide = doc_number(d, thing, "wide", 0, line ? 25 : wave ? 10 : 1);   // (as doc_match makes it)
    const int h = 1024, w = std::max(2, (int)std::lround(1024.0 * wide));
    Drawn made;
    tvg::Shape* shape = tvg::Shape::gen();
    flat_path(&made, shape, form.c_str(), w, h, 0.f, (int)doc_number(d, thing, "points", 0, form == "star" ? 5 : form == "arc" ? 270 : form == "squircle" ? 4 : form == "cog" ? 12 : form == "spiral" ? 3 : wave ? 4 : 6),
              (float)doc_number(d, thing, "inner", 0, form == "cog" ? 0.2 : form == "arc" ? 0.3 : form == "spiral" ? 0.1 : form == "chamfer" ? 0.3 : 0.5), 0.f, 0.f);
    const auto num = [&](const char* name, double usual) { return (float)doc_number(d, thing, name, d.stage->now, usual); };
    tvg_bend(shape, made.plain.data(), (uint32_t)made.plain.size(), made.plainAt.data(), (uint32_t)made.plainAt.size(), line ? 0.f : std::clamp(num("round", 0), 0.f, 0.5f) * made.plainSide,
             100.f * std::clamp(num("bloat", 0), -1.f, 1.f), num("zigzag", 0) * (float)h, (int)std::clamp(num("zigzag_ridges", 5), 0.f, 100.f), num("zigzag_smooth", 0) > 0.5f, num("offset", 0) * (float)h, 0.f);
    const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
    shape->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
    std::string path;
    char one[96];
    const auto at = [&](const tvg::Point& p) { std::snprintf(one, sizeof one, "%.5f %.5f", (p.x - 0.5 * w) / h * high, (p.y - 0.5 * h) / h * high); return std::string(one); };
    for (uint32_t c = 0, p = 0; c < cmdsCnt; ++c) switch (cmds[c]) {
        case tvg::PathCommand::MoveTo: path += (path.empty() ? "M" : " M") + at(pts[p++]); break;
        case tvg::PathCommand::LineTo: path += " L" + at(pts[p++]); break;
        case tvg::PathCommand::CubicTo: path += " C" + at(pts[p]) + " " + at(pts[p + 1]) + " " + at(pts[p + 2]), p += 3; break;
        case tvg::PathCommand::Close: path += " Z"; break;
    }
    tvg::Paint::rel(shape);
    if (path.empty()) return d.trouble = "it has no outline to make a path of", false;
    doc_act(d);
    bool ok = doc_say(d, thing, "shape", "path") && doc_say(d, thing, "path", path.c_str()) && doc_set(d, thing, "wide", wide * high);
    for (const char* name : {"round", "bloat", "zigzag", "offset"}) if (ok && doc_number(d, thing, name, 0, 0) != 0.0) ok = doc_set(d, thing, name, 0.0);
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}
// What of a thing is seen at a moment: the upright box of its outline on the picture (its own points where they are at
// hand, else its box's corners), left, top, right, bottom in points of the picture. false: it is not shown.
static bool doc_seen_box(Doc& d, int thing, double now, double* box) {
    double around[128], cover[4];
    if (!d.shown[thing].in) return false;
    const int n = doc_outline(d, thing, now, around);
    if (n > 2) {
        box[0] = box[2] = around[0], box[1] = box[3] = around[1];
        for (int i = 1; i < n; ++i) box[0] = std::min(box[0], around[2 * i]), box[2] = std::max(box[2], around[2 * i]), box[1] = std::min(box[1], around[2 * i + 1]), box[3] = std::max(box[3], around[2 * i + 1]);
        return true;
    }
    if (!doc_covers(d, thing, now, cover)) return false;
    box[0] = cover[0], box[1] = cover[1], box[2] = cover[0] + cover[2], box[3] = cover[1] + cover[3];
    return true;
}
// A value changed by so much where it is said, every key of it too (moved or sized, its own way of moving kept): its
// value as it comes, if none is said, changed so and said.
static bool doc_value_by(Doc& d, int thing, const char* name, double add, double times) {
    int rows = 0;
    doc_rows(d, sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q", thing, name), [&](sqlite3_stmt* q) { rows = sqlite3_column_int(q, 0); });
    if (!rows) return doc_set(d, thing, name, doc_usual(d, name) * times + add);
    return doc_change(d, sqlite3_mprintf("update fact set value = value * %!.17g + %!.17g where thing = %d and name = %Q", times, add, thing, name));
}
// A thing moved by so many points of the picture (as a hand carries it): its x and y in the plane it hangs in.
static bool doc_move_seen(Doc& d, int thing, double dx, double dy, double now) {
    const math::mat4 above = doc_above(d, thing, now);
    const math::double2 o = doc_seen(d, (above * math::double4{0, 0, 0, 1}).xyz), ex = doc_seen(d, (above * math::double4{1, 0, 0, 1}).xyz) - o, ey = doc_seen(d, (above * math::double4{0, 1, 0, 1}).xyz) - o;
    const double det = ex.x * ey.y - ex.y * ey.x;
    if (std::fabs(det) < 1e-12) return d.trouble = "it is seen edge on", false;
    return doc_value_by(d, thing, "x", (dx * ey.y - dy * ey.x) / det, 1) && doc_value_by(d, thing, "y", (ex.x * dy - ex.y * dx) / det, 1);
}
// Things lined up on the picture as Keynote and Figma line them up (Arrange > Align): one against the picture, more
// against the box round them all; left, centre, right, top, middle, bottom; across and down spread three or more so
// the gaps between them are even. One step.
static bool doc_align(Doc& d, const std::vector<int>& things, const std::string& how) {
    const double now = d.stage->now;
    std::vector<std::array<double, 4>> boxes;
    for (const int t : things) {
        std::array<double, 4> b{};
        if (!doc_seen_box(d, t, now, b.data())) return d.trouble = "a thing not seen now", false;
        boxes.push_back(b);
    }
    if (boxes.empty()) return d.trouble = "nothing to line up", false;
    std::array<double, 4> all = things.size() == 1 ? std::array<double, 4>{0, 0, (double)d.stage->w, (double)d.stage->h} : boxes[0];
    for (const auto& b : boxes) if (things.size() > 1) all = {std::min(all[0], b[0]), std::min(all[1], b[1]), std::max(all[2], b[2]), std::max(all[3], b[3])};
    std::vector<std::pair<double, double>> by(things.size(), {0.0, 0.0});
    if (how == "across" || how == "down") {   // (in the order they lie along the way: the first and last stay, the gaps between made even)
        const int a = how == "across" ? 0 : 1;
        if (things.size() < 3) return d.trouble = "three or more are spread", false;
        std::vector<size_t> order(things.size());
        for (size_t i = 0; i < order.size(); ++i) order[i] = i;
        std::sort(order.begin(), order.end(), [&](size_t p, size_t q) { return boxes[p][a] + boxes[p][a + 2] < boxes[q][a] + boxes[q][a + 2]; });
        double sizes = 0;
        for (const auto& b : boxes) sizes += b[a + 2] - b[a];
        const double gap = (boxes[order.back()][a + 2] - boxes[order.front()][a] - sizes) / (double)(things.size() - 1);
        double at = boxes[order.front()][a];
        for (const size_t i : order) {
            (a ? by[i].second : by[i].first) = at - boxes[i][a];
            at += boxes[i][a + 2] - boxes[i][a] + gap;
        }
    } else
        for (size_t i = 0; i < things.size(); ++i) {
            const auto& b = boxes[i];
            if (how == "left") by[i].first = all[0] - b[0];
            else if (how == "centre") by[i].first = (all[0] + all[2] - b[0] - b[2]) / 2;
            else if (how == "right") by[i].first = all[2] - b[2];
            else if (how == "top") by[i].second = all[1] - b[1];
            else if (how == "middle") by[i].second = (all[1] + all[3] - b[1] - b[3]) / 2;
            else if (how == "bottom") by[i].second = all[3] - b[3];
            else return d.trouble = "lined up left, centre, right, top, middle, bottom, across or down", false;
        }
    doc_act(d);
    bool ok = true;
    for (size_t i = 0; i < things.size() && ok; ++i) ok = doc_move_seen(d, things[i], by[i].first, by[i].second, now);
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}
// A thing made to the picture (Alight Motion's Fit, Fill and Stretch): as large as it can be inside it, or covering it,
// or stretched to it; then its middle at the picture's. One step.
static bool doc_fit(Doc& d, int thing, const std::string& how) {
    const double now = d.stage->now;
    double b[4];
    if (d.act && d.act_unmatched) {   // (within a larger act, what it has made so far shown first, as its end does (doc_acted): a picture placed and fitted in one step, golf ㊱)
        d.act_unmatched = false;
        doc_forget(d);
        if (d.act_ok && !doc_match(d)) d.act_ok = false;
    }
    if (!doc_seen_box(d, thing, now, b) || b[2] - b[0] < 1e-6 || b[3] - b[1] < 1e-6) return d.trouble = "it is not seen now", false;
    const double wx = (double)d.stage->w / (b[2] - b[0]), wy = (double)d.stage->h / (b[3] - b[1]);
    if (how != "fit" && how != "fill" && how != "stretch") return d.trouble = "made to fit, fill or stretch", false;
    doc_act(d);
    bool ok = how == "stretch" ? doc_value_by(d, thing, "scale_x", 0, wx) && doc_value_by(d, thing, "scale_y", 0, wy) : doc_value_by(d, thing, "size", 0, how == "fit" ? std::min(wx, wy) : std::max(wx, wy));
    if (ok) {
        doc_moment(d, now);   // (seen as it is now, to put its middle at the picture's)
        ok = doc_seen_box(d, thing, now, b) && doc_move_seen(d, thing, d.stage->w / 2.0 - (b[0] + b[2]) / 2, d.stage->h / 2.0 - (b[1] + b[3]) / 2, now);
    }
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}
// The upright box of the picture a thing covers at a moment, however it is turned or leans: left, top, wide, high.
static bool doc_covers(Doc& d, int thing, double now, double* box) {
    float2 half{0.f};
    float deep = 0.f;
    if (!doc_half(d, thing, &half, &deep)) return false;
    const math::mat4 place = doc_shown_at(d, thing, now);
    math::double2 low{1e12}, high{-1e12};
    for (int corner = 0; corner < 8; ++corner) {
        const math::double2 at = doc_seen(d, (place * math::double4{corner & 1 ? half.x : -half.x, corner & 2 ? half.y : -half.y, corner & 4 ? deep : -deep, 1.0}).xyz);
        low = min(low, at), high = max(high, at);
    }
    box[0] = low.x, box[1] = low.y, box[2] = high.x - low.x, box[3] = high.y - low.y;
    return true;
}

// A thing's outline on the picture at a moment: its own points as the stage's eye sees them (a shape the kit made:
// a ball, a gem, a ring), or else the corners of its box (a file's, a flat thing's), and the polygon round them all
// (a convex hull, Andrew's way): a ball's round edge, a cube's own outline, a flat thing's tilted four corners; nothing
// of it lies outside. Points of the picture from the top left, in order round it; how many (0: not shown).
static int doc_outline(Doc& d, int thing, double now, double* xy) {
    float2 half{0.f};
    float deep = 0.f;
    if (!doc_half(d, thing, &half, &deep) || !d.shown[thing].in) return 0;
    const math::mat4 place = doc_shown_at(d, thing, now);
    std::vector<math::double2> p;
    if (const Mesh* shape = d.shown[thing].shape; shape && !shape->pos.empty()) {   // (a shape whose points are at hand, the kit's: its own points as seen, every so many)
        const size_t step = std::max<size_t>(1, shape->pos.size() / 600);
        for (size_t i = 0; i < shape->pos.size(); i += step) p.push_back(doc_seen(d, (place * math::double4{math::double3(shape->pos[i]), 1.0}).xyz));
    } else
        for (int corner = 0; corner < (deep > 0.f ? 8 : 4); ++corner)
            p.push_back(doc_seen(d, (place * math::double4{corner & 1 ? half.x : -half.x, corner & 2 ? half.y : -half.y, corner & 4 ? deep : -deep, 1.0}).xyz));
    std::sort(p.begin(), p.end(), [](const math::double2& a, const math::double2& b) { return a.x < b.x || (a.x == b.x && a.y < b.y); });
    const auto turn = [](const math::double2& o, const math::double2& a, const math::double2& b) { return (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x); };
    std::vector<math::double2> hull(2 * p.size());
    size_t k = 0;
    for (size_t i = 0; i < p.size(); ++i) { while (k >= 2 && turn(hull[k - 2], hull[k - 1], p[i]) <= 0) --k; hull[k++] = p[i]; }
    for (size_t i = p.size() - 1, t = k + 1; i > 0; --i) { while (k >= t && turn(hull[k - 2], hull[k - 1], p[i - 1]) <= 0) --k; hull[k++] = p[i - 1]; }
    const size_t all = k > 0 ? k - 1 : 0;
    const int n = (int)std::min<size_t>(all, 64);   // (64 at most, evenly among them: round enough)
    for (int i = 0; i < n; ++i) { const size_t j = (size_t)i * all / (size_t)n; xy[2 * i] = hull[j].x, xy[2 * i + 1] = hull[j].y; }
    return n;
}

static double doc_usual(Doc& d, const char* name) {
    if (!strncmp(name, "js_", 3))   // (a knob of a script: as its head says it comes)
        for (const auto& [file, sc] : doc_scripts(d))
            for (size_t i = 0; i < sc.names.size(); ++i)
                if (sc.names[i] == name) return sc.knobs[i].usual;
    return doc_among("parent_rotate parent_size scale_x scale_y scale_z size speed volume opacity wide tall visible distance zoom copies copy_size copy_wide blur_level letter_spacing line_spacing contrast saturation vibrance environment gradient_scale wave_speed slope power strength spread", name) ? 1.0 : !strcmp(name, "duration") ? -1.0
         : !strcmp(name, "radius") ? 0.5 : !strcmp(name, "gap") ? 0.2   // (a relation as it comes: what it works by when nothing is said)
         : !strcmp(name, "metallic") ? 0.08 : !strcmp(name, "roughness") ? 0.6 : !strcmp(name, "reflectance") ? 0.5 : !strcmp(name, "thickness") ? 0.5 : !strcmp(name, "sun_size") ? 1.09 : !strcmp(name, "sun_halo") ? 10.0   // (a solid's surface as it comes)
         : !strcmp(name, "intensity") ? 1.0 : !strcmp(name, "falloff") ? 10.0 : !strcmp(name, "cone") ? 30.0   // (a light as it comes)
         : !strcmp(name, "fog_near") ? 2.0   // (where fog begins, as it came before it could be said)
         : !strcmp(name, "bounce") ? 0.3 : !strcmp(name, "friction") ? 0.5 : !strcmp(name, "gravity") ? 9.81   // (what falls and strikes, as it comes)
         : !strcmp(name, "ior") ? 1.5 : !strcmp(name, "dispersion") ? 0.1 : !strcmp(name, "swim") ? 0.5 : !strcmp(name, "dark") ? 4.0 : !strcmp(name, "vivid") ? 1.3 : 0.0;   // (the kit's glass as it comes)
}
static bool doc_value(Doc& d, int thing, const char* name, double at, double* is) { return doc_knows(d, thing, name) && (*is = doc_number(d, thing, name, at, doc_usual(d, name)), true); }
// A thing held by a hand, carried, and let go: the one form of everything a hand drags (a thing on the stage, its bar
// on the timeline, a number). What is carried shows at once and is not the document's until it is let go, and then
// it is one step; let go without keeping, nothing happened. The reckoning is here, once: a shell says where the hand
// is, never what the document is to hold.
//   by "place"  the thing carried on the stage: a, b = how far the hand has gone since it took hold, in points of the
//               picture (to the right, down); the thing stays under the hand at its own depth
//   by "bar"    its bar slid on the timeline: a = how many seconds later (when it comes on)
//   by "head", "tail"  its bar cut at an end: a = how many seconds later that end is (a film cut at its head goes on
//               showing at each moment what it showed there)
//   by "point"  a path shape's point or handle nearest where the hand took hold ("grip" first), moved as far as the
//               hand has gone (a, b: points of the picture); a point's handles with it
//   by "scale"  sized about its pivot as far as the hand has gone from where it took hold ("grip" first); "side" the
//               same, the side or corner across from the hand staying where it is
//   by a value's name  that number carried: a = what it is now
static bool doc_hold(Doc& d, int thing) {
    if (!doc_knows(d, thing)) return false;
    if (d.writing) return d.trouble = "the document is being written as a film: nothing is held until that is done", false;
    if (d.held.thing != thing) d.held = Doc::Held{}, d.held.thing = thing;   // (held already: the same hold goes on)
    return true;
}
static bool doc_carry(Doc& d, const char* by, double a, double b) {
    Doc::Held& h = d.held;
    if (h.thing < 0) return d.trouble = "nothing is held", false;
    const double now = d.stage->now;
    const auto was = [&](const char* name) {   // (as it was when the hand first carried it)
        auto found = h.from.find(name);
        if (found == h.from.end()) found = h.from.emplace(name, doc_number(d, h.thing, name, now, doc_usual(d, name))).first;
        return found->second;
    };
    const auto tries = [&](const char* name, double is) { return doc_knows(d, h.thing, name, &is) && (h.tried[name] = is, true); };
    bool ok = true;
    if (!strcmp(by, "place")) {
        if (!h.from.count(" point")) h.from[" point"] = doc_point(d, h.thing, now);   // (remembered with the hold: one unit of the stage in points of the picture, where the thing stood)
        const double x = was("x"), y = was("y"), point = h.from[" point"];
        // (the hand's travel is the stage's; a thing that goes with another is moved in that one's own turned and sized way)
        const math::double3 by = inverse(doc_above(d, h.thing, now).upperLeft()) * math::double3{a / point, -b / point, 0.0};
        ok = tries("x", x + by.x) && tries("y", y + by.y);
    } else if (!strcmp(by, "grip")) {   // (where on the picture the hand took hold, points from the top left: what scale, turn and pivot reckon from)
        double frame[8];
        if (!doc_frame(d, h.thing, now, frame)) return d.trouble = "it has no frame on the picture", false;
        h.from[" grip x"] = a, h.from[" grip y"] = b, h.from[" pivot x"] = frame[5], h.from[" pivot y"] = frame[6], h.from[" turned"] = frame[4];
        h.from[" middle x"] = frame[0], h.from[" middle y"] = frame[1], h.from[" wide"] = frame[2], h.from[" high"] = frame[3];
        h.from[" point"] = doc_point(d, h.thing, now);
        was("scale_x"), was("scale_y"), was("rotate"), was("anchor_x"), was("anchor_y"), was("x"), was("y");
    } else if (!strcmp(by, "scale") || !strcmp(by, "turn") || !strcmp(by, "pivot")) {   // (a, b: how far the hand has gone since it took hold)
        if (!h.from.count(" grip x")) return d.trouble = "the hand has not said where it took hold (grip)", false;
        const double px = h.from[" pivot x"], py = h.from[" pivot y"], gx = h.from[" grip x"] - px, gy = h.from[" grip y"] - py, hx = gx + a, hy = gy + b;
        const double turned = h.from[" turned"] * (double)TAU / 360.0, c = std::cos(-turned), s = std::sin(-turned);   // (into the thing's own upright: as the picture counts, clockwise)
        if (!strcmp(by, "scale")) {   // (each way as far from the point it turns about as the hand now is, against where it took hold)
            const double g0 = c * gx - s * gy, g1 = s * gx + c * gy, n0 = c * hx - s * hy, n1 = s * hx + c * hy;
            ok = (std::fabs(g0) < 2.0 || tries("scale_x", was("scale_x") * n0 / g0)) && (std::fabs(g1) < 2.0 || tries("scale_y", was("scale_y") * n1 / g1));
        } else if (!strcmp(by, "turn")) {   // (round the point it turns about as far as the hand has gone round it; the picture counts clockwise, the stage the other way)
            const double by = std::atan2(hy, hx) - std::atan2(gy, gx);
            ok = tries("rotate", was("rotate") - by * 360.0 / (double)TAU);
        } else {   // (the point it turns about moved under the hand, the thing staying where it is seen: After Effects' Pan Behind)
            float2 half{0.5f};
            doc_half(d, h.thing, &half);
            const double point = h.from[" point"], size = was("size") == 0.0 ? 1.0 : doc_number(d, h.thing, "size", now, 1);
            const double l0 = (c * a - s * b) / point, l1 = -(s * a + c * b) / point;   // (the hand's travel in the thing's own units, y up)
            const double sx = was("scale_x"), sy = was("scale_y");
            ok = tries("anchor_x", was("anchor_x") + l0 / (size * (sx == 0.0 ? 1.0 : sx) * std::max(half.x, 1e-6f))) &&
                 tries("anchor_y", was("anchor_y") + l1 / (size * (sy == 0.0 ? 1.0 : sy) * std::max(half.y, 1e-6f)));
            const math::double3 move = inverse(doc_above(d, h.thing, now).upperLeft()) * math::double3{a / point, -b / point, 0.0};
            ok = ok && tries("x", was("x") + move.x) && tries("y", was("y") + move.y);
        }
    } else if (!strcmp(by, "side")) {   // (sized, the side or corner across from where the hand took hold staying where it is: Figma's and Keynote's handles; "scale" sizes it about its pivot, as with Alt)
        if (!h.from.count(" grip x")) return d.trouble = "the hand has not said where it took hold (grip)", false;
        const double turned = h.from[" turned"] * (double)TAU / 360.0, c = std::cos(-turned), s = std::sin(-turned);
        const auto own = [&](double x, double y) { return math::double2{c * x - s * y, s * x + c * y}; };   // (the picture's way into the thing's own upright, from its middle)
        const double mx = h.from[" middle x"], my = h.from[" middle y"], w = h.from[" wide"], hi = h.from[" high"];
        const math::double2 grip = own(h.from[" grip x"] - mx, h.from[" grip y"] - my);
        const math::double2 o{std::fabs(grip.x) > w / 4.0 ? -std::copysign(w / 2.0, grip.x) : 0.0, std::fabs(grip.y) > hi / 4.0 ? -std::copysign(hi / 2.0, grip.y) : 0.0};   // (the side across, or the corner; an edge's middle: that way unsized)
        const math::double2 pivot = own(h.from[" pivot x"] - mx, h.from[" pivot y"] - my), g = grip - o, n = own(h.from[" grip x"] + a - mx, h.from[" grip y"] + b - my) - o;
        const double k0 = std::fabs(g.x) < 2.0 ? 1.0 : n.x / g.x, k1 = std::fabs(g.y) < 2.0 ? 1.0 : n.y / g.y;
        ok = (k0 == 1.0 || tries("scale_x", was("scale_x") * k0)) && (k1 == 1.0 || tries("scale_y", was("scale_y") * k1));
        const math::double2 back{(1.0 - k0) * (o.x - pivot.x), (1.0 - k1) * (o.y - pivot.y)};   // (sized about its pivot, the side across moved: so much back, in its own upright)
        const double dx = c * back.x + s * back.y, dy = -s * back.x + c * back.y, point = h.from[" point"];
        const math::double3 move = inverse(doc_above(d, h.thing, now).upperLeft()) * math::double3{dx / point, -dy / point, 0.0};
        ok = ok && tries("x", was("x") + move.x) && tries("y", was("y") + move.y);
    } else if (!strcmp(by, "point")) {   // (a path shape's point nearest where the hand took hold (grip), moved as far as the hand has gone: a, b)
        if (!h.from.count(" grip x")) return d.trouble = "the hand has not said where it took hold (grip)", false;
        if (!h.said.count(" path")) {   // (which point, and the path as it was: once, with the hold)
            int nearest = -1;
            double near = 12.0;
            for (const bool handles : {false, true}) {   // (its points first: a handle lying on its point is the point)
                std::vector<double> xy(2048);
                const int n = std::min(1024, doc_points(d, h.thing, now, xy.data(), 1024, handles));
                for (int i = 0; i < n; ++i)
                    if (const double far = std::hypot(xy[2 * i] - h.from[" grip x"], xy[2 * i + 1] - h.from[" grip y"]); far < near - 0.5) near = far, nearest = i, h.from[" handle"] = handles;
            }
            if (nearest < 0) return d.trouble = "no point of its path where the hand took hold", false;
            h.from[" point"] = nearest, h.said[" path"] = doc_word(d, h.thing, "path");
        }
        const DocPlane plane(d, h.thing, now);
        const math::double2 from = plane.own(h.from[" grip x"], h.from[" grip y"]), to = plane.own(h.from[" grip x"] + a, h.from[" grip y"] + b);
        std::string moved;
        if (!plane.seen || !(h.from[" handle"] != 0.0 ? sk_svg_move_handle : sk_svg_move)(h.said[" path"].c_str(), (int)h.from[" point"], (float)(to.x - from.x), (float)(to.y - from.y), moved)) return d.trouble = "its point could not be moved", false;
        h.said["path"] = moved;
        ok = doc_match(d);   // (made again with it now, as a value it is made by)
    } else if (!strcmp(by, "bar")) {
        h.from[" bar"] = 1.0;   // (the whole layer moved in time: its keys go with it when it is let go)
        ok = tries("start", std::max(0.0, was("start") + a));
    } else if (!strcmp(by, "head")) {
        const double start = was("start"), lasts = was("duration"), to = std::clamp(start + a, 0.0, lasts > 0.0 ? start + lasts - 1e-3 : 1e6);
        ok = tries("start", to) && (lasts <= 0.0 || tries("duration", start + lasts - to));
        if (ok && doc_word(d, h.thing, "kind") == "film") ok = tries("begin", std::max(0.0, was("begin") + (to - start) * std::fabs(was("speed"))));
    } else if (!strcmp(by, "tail")) {
        const double start = was("start"), lasts = was("duration");
        ok = tries("duration", std::max(1e-3, (lasts > 0.0 ? lasts : doc_seconds(d) - start) + a));
    } else {
        was(by);   // (as it was: whether it comes on or goes off, carried)
        ok = tries(by, a);
        if (ok && strstr(doc_making, ("'" + std::string(by) + "'").c_str())) ok = doc_match(d);   // (a value it is made by: made again with it now, not when let go)
        else if (ok && strstr(doc_fxing, ("'" + std::string(by) + "'").c_str())) ok = doc_match(d);   // (an effect's: made again only if it now reaches past its room)
    }
    doc_moment(d, now);
    return ok;
}
// A thing's keys moved by so many seconds, every value's that has keys (as After Effects moves a layer's).
static bool doc_keys_by(Doc& d, int thing, double by) {
    return doc_change(d, sqlite3_mprintf("update fact set at = at + %!.17g where thing = %d and name in (select name from fact where thing = %d group by name having count(*) > 1 or max(at) <> 0 or max(way) is not null);"
                                         "update fact set at = at - 1e7 where thing = %d and at >= 5e6", 1e7 + by, thing, thing, thing));   // (by way of far off: no key lands on one not yet moved)
}
static double doc_lasts(Doc& d, int thing);
// A thing cut in two at a moment (Ableton Live's Split, Cmd+E; AbsentMotion's Scene.splitAt): it ends there, and a copy
// of it (every fact, its keys staying where they are on the stage) begins there, a film's copy as far on in the film as
// the head's trim takes it (above), lying just above it. One step to undo; the answer is the copy, as duplicate's.
static int doc_split(Doc& d, int thing, double at) {
    if (doc_knows(d, thing) && doc_number(d, thing, "lock", 0, 0) > 0.5) return d.trouble = "it is locked", 0;
    const double start = doc_number(d, thing, "start", 0, 0), lasts = doc_number(d, thing, "duration", 0, -1), end = lasts > 0.0 ? start + lasts : doc_seconds(d);
    if (!(at > start + 1e-3 && at < end - 1e-3)) return d.trouble = "the moment is not inside it", 0;
    doc_act(d);
    const int made = doc_duplicate(d, thing);
    bool ok = made && doc_set(d, thing, "duration", at - start) && doc_set(d, made, "start", at) && doc_set(d, made, "duration", end - at);
    if (ok && doc_word(d, thing, "kind") == "film") ok = doc_set(d, made, "begin", doc_number(d, thing, "begin", 0, 0) + (at - start) * std::fabs(doc_number(d, thing, "speed", 0, 1)));
    if (ok) {   // (the part after lies just above the part before, as After Effects' Split Layer puts the new layer: a copy is made on top of all)
        doc_forget(d);
        const std::vector<int> lie = doc_laid(d);
        const long from = std::find(lie.begin(), lie.end(), made) - lie.begin(), under = std::find(lie.begin(), lie.end(), thing) - lie.begin();
        if (from != under + 1) ok = doc_raise(d, made, (int)(under + (from > under ? 1 : 0) - from));
    }
    return doc_acted(d) && ok ? made : 0;
}
// The work area taken out and the gap closed (After Effects' Edit > Extract Work Area; Premiere Pro's Extract, DaVinci
// Resolve's Ripple Delete): every thing across one of its edges cut there (doc_split, a film's part after as far on in
// the film), the parts within it taken away, and all after it brought back by its length, their keys with them; the
// marks within it taken away and those after it brought back too, as Premiere Pro's Extract does with Ripple Sequence
// Markers on (its markers after an edit move with the ripple; from 25.3 those in the part extracted are cleared). One
// step (golf (72), P18: 20 seconds made 15 by taking 5 out of the middle, the hits' marks kept on their hits).
static bool doc_extract(Doc& d) {
    double in = 0.0, out = 0.0;
    doc_work(d, &in, &out);
    const double gap = out - in, all = doc_seconds(d);
    if (!(gap > 1e-3) || (in < 1e-3 && out > all - 1e-3)) return d.trouble = "the work area is all of it: set it to the part to take out", false;
    std::vector<int> things;
    doc_rows(d, sqlite3_mprintf("select thing from fact where thing <> 0 and name = 'kind' and value <> 'behaviour' order by thing"), [&](sqlite3_stmt* row) { things.push_back(sqlite3_column_int(row, 0)); });   // (a relation has no time of its own)
    doc_act(d);
    bool ok = true;
    for (const int t : things) {
        const double s = doc_number(d, t, "start", 0, 0), lasts = doc_number(d, t, "duration", 0, -1), e = lasts > 0.0 ? s + lasts : all;
        int after = s >= out - 1e-3 ? t : 0;   // (the part after the work area, brought back)
        if (!after && e > out + 1e-3) ok = ok && (after = doc_split(d, t, out)) != 0;
        if (s < in - 1e-3 && e > in + 1e-3) { const int within = doc_split(d, t, in); ok = ok && within && doc_remove(d, within); }
        else if (s >= in - 1e-3 && s < out - 1e-3) ok = ok && doc_remove(d, t);
        if (after) ok = ok && doc_set(d, after, "start", doc_number(d, after, "start", 0, 0) - gap) && doc_keys_by(d, after, -gap);
        if (!ok) break;
    }
    ok = ok && doc_change(d, sqlite3_mprintf("delete from fact where thing = 0 and name in ('mark', 'mark_colour') and at >= %!.17g and at < %!.17g;"
                                             "update fact set at = at + %!.17g where thing = 0 and name in ('mark', 'mark_colour') and at >= %!.17g;"
                                             "update fact set at = at - 1e7 where thing = 0 and name in ('mark', 'mark_colour') and at >= 5e6", in - 1e-9, out - 1e-9, 1e7 - gap, out - 1e-9));   // (by way of far off, as doc_keys_by: none lands on one not yet moved)
    if (!ok) d.act_ok = false;
    return doc_acted(d);
}
static bool doc_let_go(Doc& d, bool keep) {
    const Doc::Held h = d.held;
    d.held = Doc::Held{};
    if (h.thing < 0) return d.trouble = "nothing is held", false;
    if (!keep || (h.tried.empty() && h.said.empty())) return doc_match(d), doc_moment(d, d.stage->now), true;   // (made again as the document has it)
    doc_act(d);   // (all that was carried is one step: a value that takes keys by the rule of a value changed at a moment, any other as the value it is)
    for (const auto& [name, is] : h.tried) {
        bool moves = false;
        for (const DocValue& one : doc_values) moves = moves || (one.moves && name == one.name);
        if (!moves && name.rfind("js_", 0) == 0)   // (a knob of one of its scripts, an effect's, as doc_knows finds it: one that takes keys as its file says, a track;
            //  else it was set, unkeyed, every key it had taken away: golf_noise.md 44)
            for (const DocScript* sc : doc_scripts_on(d, h.thing))
                for (const DocValue& one : sc->values) moves = moves || (one.moves && name == one.name);
        if (!(moves ? doc_put(d, h.thing, name.c_str(), d.stage->now, is) : doc_set(d, h.thing, name.c_str(), is))) break;
    }
    for (const auto& [name, is] : h.said) if (name[0] != ' ' && !doc_say(d, h.thing, name.c_str(), is.c_str())) break;   // (' path': what it was, kept with the hold)
    const auto bar = h.from.find(" bar"), start = h.tried.find("start");
    if (bar != h.from.end() && start != h.tried.end() && start->second != h.from.at("start")) doc_keys_by(d, h.thing, start->second - h.from.at("start"));
    return doc_acted(d);
}
// Sequence Layers (After Effects' Keyframe Assistant): the things that have a length and stand on their own (shown,
// not locked, going with no other, not a camera or a light), from the front of the stack, each beginning where the
// one before ends less `overlap` seconds, on a frame of `rate`, never before the one before; the first where it is.
// Each moved whole, its keys with it. One step.
static bool doc_sequence(Doc& d, double overlap, double rate) {
    std::vector<int> order;
    const std::vector<int> laid = doc_laid(d);
    for (auto t = laid.rbegin(); t != laid.rend(); ++t) {
        const std::string kind = doc_word(d, *t, "kind");
        if (kind != "camera" && kind != "light" && doc_number(d, *t, "visible", 0, 1) > 0.5 && doc_number(d, *t, "lock", 0, 0) <= 0.5 && doc_number(d, *t, "parent", 0, 0) == 0.0 && doc_lasts(d, *t) > 0.0) order.push_back(*t);
    }
    if (order.size() < 2) return d.trouble = "two things that have a length are needed to follow one another", false;
    const auto frame = [&](double s) { return std::round(s * rate) / rate; };
    double at = frame(doc_number(d, order[0], "start", 0, 0));
    doc_act(d);
    bool ok = true;
    for (const int t : order) {
        const double was = doc_number(d, t, "start", 0, 0), lasts = doc_lasts(d, t);
        if (std::fabs(at - was) > 1e-9) ok = ok && doc_set(d, t, "start", at) && doc_keys_by(d, t, at - was);
        at = std::max(at, frame(at + lasts - overlap));
    }
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}
// The picked things lined up in time (Alight Motion's timing: Extend Start, Move to Playhead, Align Left, Stagger,
// Align Right): starts to the earliest; ends to the latest; each to begin at the playhead; one after another in the
// order picked; or its start brought to the playhead either way, its end staying (Final Cut Pro's Extend Edit; a film from as far in it as the head's
// trim takes it). A thing moved takes its keys with it, as Sequence moves them. One step.
static bool doc_time_align(Doc& d, const std::vector<int>& things, const std::string& how, double now) {
    if (things.empty()) return d.trouble = "nothing to line up", false;
    double first = 1e18, last = -1e18;
    for (const int t : things) first = std::min(first, doc_number(d, t, "start", 0, 0)), last = std::max(last, doc_number(d, t, "start", 0, 0) + doc_lasts(d, t));
    doc_act(d);
    bool ok = true;
    double after = doc_number(d, things[0], "start", 0, 0);
    for (const int t : things) {
        if (!ok) break;
        const double was = doc_number(d, t, "start", 0, 0), lasts = doc_lasts(d, t);
        double to = was;
        if (how == "start") to = first;
        else if (how == "end") to = last - lasts;
        else if (how == "playhead") to = now;
        else if (how == "stagger") to = after, after += lasts;
        else if (how == "extend") {
            if (now >= was + lasts - 1e-3) continue;   // (the playhead past its end: nothing to draw out to)
            ok = doc_set(d, t, "start", std::max(0.0, now)) && doc_set(d, t, "duration", was + lasts - std::max(0.0, now));
            if (ok && doc_word(d, t, "kind") == "film") ok = doc_set(d, t, "begin", std::max(0.0, doc_number(d, t, "begin", 0, 0) + (std::max(0.0, now) - was) * std::fabs(doc_number(d, t, "speed", 0, 1))));
            continue;
        } else if (how == "trim_end") {   // (Final Cut Pro's Trim End: its end to the playhead, the playhead inside it)
            if (now > was + 1e-3 && now < was + lasts - 1e-3) ok = doc_set(d, t, "duration", now - was);
            continue;
        } else { ok = false, d.trouble = "lined up by start, end, playhead, stagger, extend or trim_end"; break; }
        to = std::max(0.0, to);
        if (std::fabs(to - was) > 1e-9) ok = doc_set(d, t, "start", to) && doc_keys_by(d, t, to - was);
    }
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}
// A thing made to go at another speed, held in place at its start, at a moment, or at its end (After Effects' Time
// Stretch: Hold in place Layer In-point, Current Frame, Layer Out-point; Pinnacle Studio's speed anchor): it lasts as
// much longer as it goes slower, and its keys are stretched with it round that place; a film plays the same stretch of
// itself. One step.
static bool doc_time_stretch(Doc& d, int thing, double speed, const std::string& hold, double now) {
    const double was = std::fabs(doc_number(d, thing, "speed", 0, 1)), to = std::fabs(speed);
    if (to < 1e-3) return d.trouble = "a speed of nothing goes nowhere", false;
    if (hold != "start" && hold != "now" && hold != "end") return d.trouble = "held at start, now or end", false;
    const double start = doc_number(d, thing, "start", 0, 0), lasts = doc_lasts(d, thing) > 0.0 ? doc_lasts(d, thing) : std::max(1e-3, doc_seconds(d) - start), f = was / to;   // (no length said: to the document's end, as doc_split counts it)
    const double at = hold == "start" ? start : hold == "end" ? start + lasts : std::clamp(now, start, start + lasts);
    doc_act(d);
    bool ok = doc_set(d, thing, "speed", speed) && doc_set(d, thing, "start", std::max(0.0, at + (start - at) * f)) && doc_set(d, thing, "duration", lasts * f)
           && doc_change(d, sqlite3_mprintf("update fact set at = 1e7 + %!.17g + (at - %!.17g) * %!.17g where thing = %d and name in (select name from fact where thing = %d group by name having count(*) > 1 or max(at) <> 0 or max(way) is not null);"
                                            "update fact set at = at - 1e7 where thing = %d and at >= 5e6", at, at, f, thing, thing, thing));   // (by way of far off, as doc_keys_by: no key lands on one not yet moved)
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}
// A thing's named values as they come (After Effects' Reset on a group of values): what was said of them and their keys
// forgotten; one step.
static bool doc_reset(Doc& d, int thing, const std::string& names) {
    doc_act(d);
    bool ok = true;
    std::istringstream said(names);
    for (std::string n; ok && said >> n;) ok = doc_knows(d, thing, n.c_str()) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q", thing, n.c_str()));
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}
static bool doc_undo(Doc& d) { return doc_step(d, true); }
static bool doc_redo(Doc& d) { return doc_step(d, false); }
