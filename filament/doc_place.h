// doc_place.h: part of doc.h (where a thing lies, its look, its effects). Read only through doc.h.
static math::mat4 doc_place(Doc& d, int thing, double now, int deeper, int copy, bool member) {
    const double said = now;   // (the moment itself: a wave goes on at it, whatever the copy's delay)
    struct Placing { Doc& d; int copy, n; bool hasAt; math::double3 at; ~Placing() { d.ctxCopy = copy, d.ctxN = n, d.ctxHasAt = hasAt, d.ctxAt = at; } } was{d, d.ctxCopy, d.ctxN, d.ctxHasAt, d.ctxAt};   // (which copy, as it was before: told back after)
    {
        const int many = member ? doc_members(d, thing) : 1;
        d.ctxCopy = member ? copy : 0, d.ctxN = many, d.ctxHasAt = false;
    }
    if (member) now -= doc_delay(d, thing, copy);   // (a member of its copies: as it was so much earlier)
    const auto turned = [&](const char* name, math::double3 about) { return math::mat4::rotation(doc_literal(d, thing, name, now, 0) * (double)TAU / 360.0, about); };
    float2 half{0.5f};
    float deep = 0.f;
    doc_half(d, thing, &half, &deep);
    const math::double3 anchor{doc_literal(d, thing, "anchor_x", now, 0) * half.x, doc_literal(d, thing, "anchor_y", now, 0) * half.y, doc_literal(d, thing, "anchor_z", now, 0) * deep};
    // A member of its copies is moved, scattered and waved in units of the stage (so a larger thing is not further
    // apart), its copies laid about the thing's own place (its middle where the rows' middle is); then turned and sized
    // as its copy, its scattering and its wave say, about its anchor.
    const int n = member ? doc_members(d, thing) : 1;
    math::double3 shift{0.0};
    math::mat4 own;
    if (member) {
        const int columns = (int)doc_literal(d, thing, "copy_columns", 0, 0);
        const double along = columns > 0 ? copy % columns : copy, down = columns > 0 ? copy / columns : copy;
        const double alongMid = columns > 0 ? (std::min(n, columns) - 1) * 0.5 : (n - 1) * 0.5, downMid = columns > 0 ? (std::ceil((double)n / columns) - 1) * 0.5 : (n - 1) * 0.5;
        shift += math::double3{(along - alongMid) * doc_literal(d, thing, "copy_x", now, 0), (down - downMid) * doc_literal(d, thing, "copy_y", now, 0), (copy - (n - 1) * 0.5) * doc_literal(d, thing, "copy_z", now, 0)};
        if (doc_connects(d, thing)) {   // (where this copy lies, laid out in its rows, on the stage: what a falloff weighs; its own values, unconnected)
            d.ctxAt = (doc_above(d, thing, now, deeper) * math::double4{doc_number(d, thing, "x", now, 0) + shift.x, doc_number(d, thing, "y", now, 0) + shift.y, doc_number(d, thing, "z", now, 0) + shift.z, 1.0}).xyz;
            d.ctxHasAt = true;
        }
        // (each copy turned and sized more than the one before; given by a behaviour, each copy its own turn and size as it gives)
        const bool turnsEach = doc_driven_by(d, thing, "copy_rotate"), sizesEach = doc_driven_by(d, thing, "copy_size");
        own = math::mat4::rotation((copy * doc_literal(d, thing, "copy_turn", now, 0) / n + (turnsEach ? 1 : copy) * doc_literal(d, thing, "copy_rotate", now, 0)) * (double)TAU / 360.0, math::double3{0, 0, 1}) *
              math::mat4::scaling(sizesEach ? doc_literal(d, thing, "copy_size", now, 1) : std::pow(doc_literal(d, thing, "copy_size", now, 1), copy)) *
              math::mat4::scaling(math::double3{n > 1 ? std::pow(doc_literal(d, thing, "copy_wide", now, 1), (double)copy / (n - 1)) : 1.0, 1.0, 1.0});
        if (const double ty = doc_literal(d, thing, "copy_rotate_y", now, 0), tx = doc_literal(d, thing, "copy_rotate_x", now, 0); tx != 0.0 || ty != 0.0)   // (turned out of the picture's plane too, each more than the one before)
            own = math::mat4::rotation(copy * ty * (double)TAU / 360.0, math::double3{0, 1, 0}) * math::mat4::rotation(copy * tx * (double)TAU / 360.0, math::double3{1, 0, 0}) * own;
        // (scattered: each copy its own way, from the seed and its number; the same each time)
        const double sx = doc_literal(d, thing, "scatter_x", now, 0), sy = doc_literal(d, thing, "scatter_y", now, 0), sr = doc_literal(d, thing, "scatter_rotate", now, 0), ss = doc_literal(d, thing, "scatter_size", now, 0);
        const uint32_t seed = (uint32_t)doc_literal(d, thing, "seed", 0, 0);
        const auto roll = [&](uint32_t k) {   // (-1 to 1: a hash of the seed, the copy and which roll)
            uint32_t h = seed * 0x9E3779B9u ^ ((uint32_t)copy + 1u) * 0x85EBCA6Bu ^ k * 0xC2B2AE35u;
            h ^= h >> 16, h *= 0x7FEB352Du, h ^= h >> 15, h *= 0x846CA68Bu, h ^= h >> 16;
            return (double)h / 2147483647.5 - 1.0;
        };
        if (const double sz = doc_literal(d, thing, "scatter_z", now, 0); sz != 0.0) shift.z += sz * roll(5);
        if (sx != 0.0 || sy != 0.0 || sr != 0.0 || ss != 0.0) {
            const auto round = [](double v, double edge) { return edge > 0.0 ? v - 2.0 * edge * std::floor((v + edge) / (2.0 * edge)) : v; };   // (flowing past the edge: back at the other)
            const auto whole = [&](double drift, double edge) { const double k = edge > 0.0 ? std::round(drift * d.round / (2.0 * edge)) : 0.0; return k != 0.0 ? k * 2.0 * edge / d.round : drift; };   // (round the scattering a whole number of times in the document)
            shift += math::double3{round(sx * roll(1) + whole(doc_literal(d, thing, "drift_x", 0, 0), sx) * now, sx), round(sy * roll(2) + whole(doc_literal(d, thing, "drift_y", 0, 0), sy) * now, sy), 0.0};
            own = own * math::mat4::rotation(sr * roll(3) * (double)TAU / 360.0, math::double3{0, 0, 1}) * math::mat4::scaling(std::max(0.0, 1.0 + ss * roll(4)));
        }
        // (a wave through it: each copy so far behind the one before, at its own moment, a copy staggered so much later)
        const double said_speed = doc_literal(d, thing, "wave_speed", said, 1), noisy = doc_literal(d, thing, "wave_noise", said, 0);
        const double turns = std::round(said_speed * d.round), speed = turns > 0.0 ? turns / d.round : said_speed;   // (a whole number of times round in the document: its film loops)
        double w = std::sin((double)TAU * (speed * now - doc_literal(d, thing, "wave_phase", said, 0) * copy / n));
        if (noisy > 0.0) {   // (a smooth noise: random values at the corners of a lattice, eased between; its own for each copy, or, with a scale, read where the copy lies)
            const double scale = doc_literal(d, thing, "noise_scale", said, 0);
            const auto at = [&](int64_t i, int64_t j, int64_t k) {
                const int64_t ring = turns > 0.0 ? (int64_t)turns * 2 : 0;   // (round in time as the wave is: the lattice's last row is its first)
                if (ring > 0) k = ((k % ring) + ring) % ring;
                uint32_t h = seed * 0x9E3779B9u ^ (uint32_t)i * 0x85EBCA6Bu ^ (uint32_t)j * 0x27d4eb2du ^ (uint32_t)k * 0xC2B2AE35u ^ (scale > 0.0 ? 0u : ((uint32_t)copy + 1u) * 0x165667B1u);
                h ^= h >> 16, h *= 0x7FEB352Du, h ^= h >> 15, h *= 0x846CA68Bu, h ^= h >> 16;
                return (double)h / 2147483647.5 - 1.0;
            };
            const math::double3 p{scale > 0.0 ? shift.x / scale : 0.0, scale > 0.0 ? shift.y / scale : 0.0, speed * now * 2.0};
            const math::double3 f{std::floor(p.x), std::floor(p.y), std::floor(p.z)}, u = p - f, e = u * u * (3.0 - 2.0 * u);
            double sum = 0.0;
            for (int c = 0; c < 8; ++c)
                sum += at((int64_t)f.x + (c & 1), (int64_t)f.y + ((c >> 1) & 1), (int64_t)f.z + ((c >> 2) & 1)) * ((c & 1) ? e.x : 1.0 - e.x) * ((c & 2) ? e.y : 1.0 - e.y) * ((c & 4) ? e.z : 1.0 - e.z);
            w = w * (1.0 - noisy) + noisy * sum;
        }
        shift += math::double3{doc_literal(d, thing, "wave_x", said, 0), doc_literal(d, thing, "wave_y", said, 0), 0.0} * w;
        shift.z += doc_literal(d, thing, "wave_z", said, 0) * std::cos((double)TAU * (speed * now - doc_literal(d, thing, "wave_phase", said, 0) * copy / n));   // (a quarter round after: round and round with up and down)
        own = own * math::mat4::rotation(w * doc_literal(d, thing, "wave_rotate", said, 0) * (double)TAU / 360.0, math::double3{0, 0, 1}) *
              math::mat4::scaling(std::max(0.0, 1.0 + w * doc_literal(d, thing, "wave_size", said, 0)));
    }
    if (const std::string file = doc_literal_word(d, thing, "file"); std::filesystem::path(file).extension() == ".mid") {   // (a MIDI file's roll going by: its start at the thing at its first moment, 300 points a second, ALMaMPlayer's track.gd)
        const double speed = doc_literal(d, thing, "speed", 0, 1), point = doc_point(d, thing, said);
        std::string why;
        double w = 0, h = 0;
        if (point > 0 && midi_roll_box(midi_roll_file(file, speed == 1.0 ? 0.0 : midi_bpm(file.c_str()) * speed, why), w, h))
            shift.x += (0.5 * w - 300.0 * (said - doc_literal(d, thing, "start", 0, 0))) / point;
    }
    if (const std::string notes = member ? doc_literal_word(d, thing, "script") : std::string(); std::filesystem::path(notes).extension() == ".mid") {   // (a MIDI file's notes, copies: midi.h)
        Doc::Scripted& kept = d.scripted[thing];
        if (kept.at != said || (int)kept.copies.size() != n)   // (its speed: the file's beats faster or slower, as a film's)
            kept.at = said, kept.copies = midi_copies(midi_of(notes), n, (said - doc_literal(d, thing, "start", 0, 0)) * doc_literal(d, thing, "speed", 0, 1), doc_number(d, 0, "high", 0, 1080) / 1080.0 * (double)d.stage->h / std::max(1.0, doc_number(d, 0, "high", 0, 1080) * doc_point(d, thing, said)));
        const ScriptCopy c = copy < n ? kept.copies[(size_t)copy] : ScriptCopy{};
        shift += math::double3{c.x, c.y, c.z};
        own = own * math::mat4::scaling(c.size);
    } else if (member)   // (a recipe as code: each copy where the thing's script says, on top of what its copies' values say)
        if (DocScript* sc = doc_script_of(d, doc_literal_word(d, thing, "script")); sc && sc->run && sc->run->ok()) {
            Doc::Scripted& kept = d.scripted[thing];
            if (kept.at != said || kept.made != doc_scripts_made(d) || (int)kept.copies.size() != n) {   // (every copy at once, the knobs read once)
                std::map<std::string, double> numbers;
                std::map<std::string, std::string> words;
                for (size_t i = 0; i < sc->knobs.size(); ++i) {
                    if (sc->knobs[i].kind == "select") {   // (not said yet: its first word)
                        const std::string w = doc_word(d, thing, sc->names[i].c_str());
                        words[sc->knobs[i].name] = w.empty() ? sc->knobs[i].words.substr(0, sc->knobs[i].words.find(' ')) : w;
                    }
                    else numbers[sc->knobs[i].name] = doc_number(d, thing, sc->names[i].c_str(), said, sc->knobs[i].usual);
                }
                kept.at = said, kept.made = doc_scripts_made(d);
                const double begins = doc_literal(d, thing, "start", 0, 0);
                numbers["duration"] = doc_literal(d, thing, "duration", 0, -1);   // (how long the thing lasts: an Out ends before it)
                kept.copies = sc->run->copies(n, said - begins, numbers, words, (double)d.stage->h / std::max(1.0, doc_number(d, 0, "high", 0, 1080) * doc_point(d, thing, said)));   // (a pixel of the document, where the thing stands)
            }
            const ScriptCopy c = copy < n ? kept.copies[(size_t)copy] : ScriptCopy{};
            shift += math::double3{c.x, c.y, c.z};
            own = own * math::mat4::rotation(c.rotate * (double)TAU / 360.0, math::double3{0, 0, 1}) * math::mat4::rotation(c.rotate_y * (double)TAU / 360.0, math::double3{0, 1, 0}) *
                  math::mat4::rotation(c.rotate_x * (double)TAU / 360.0, math::double3{1, 0, 0}) * math::mat4::scaling(c.size);
        }
    double tall = doc_literal(d, thing, "tall", now, 1);   // (taller, its anchor staying: a copy as its number in copy_tall says too)
    if (member) {
        auto found = d.talls.find(thing);
        if (found == d.talls.end()) {   // (read once a moment)
            std::istringstream said(doc_literal_word(d, thing, "copy_tall"));
            std::vector<double> all;
            for (std::string one; said >> one && one != "none";) all.push_back(std::atof(one.c_str()));
            found = d.talls.emplace(thing, std::move(all)).first;
        }
        const std::vector<double>& each = found->second;
        if (!each.empty()) tall *= each[(size_t)copy % each.size()];
    }
    own = own * math::mat4::scaling(math::double3{1.0, tall, 1.0});
    if (member)   // (a split text's letter turns and grows about its own middle, not the whole line's)
        if (const double middle = doc_letter_middle(d, thing, copy); middle != 0.0) own = math::mat4::translation(math::double3{middle, 0, 0}) * own * math::mat4::translation(math::double3{-middle, 0, 0});
    if (!d.phys_keyed && !deeper) {   // (a thing that falls and strikes, or a letter of one: where the world has its middle, turned as it has it)
        PhysPose pose;
        if (doc_physics(d, thing, now, &pose, member ? copy : -1)) {
            const math::quat q{pose.turn[3], pose.turn[0], pose.turn[1], pose.turn[2]};
            return math::mat4::translation(math::double3{pose.at[0], pose.at[1], pose.at[2]}) * math::mat4(math::mat3(math::quat(q))) *
                   math::mat4::scaling(doc_literal(d, thing, "size", now, 1)) *
                   math::mat4::scaling(math::double3{doc_literal(d, thing, "scale_x", now, 1), doc_literal(d, thing, "scale_y", now, 1), doc_literal(d, thing, "scale_z", now, 1)}) *
                   math::mat4::translation(math::double3{-doc_letter_middle(d, thing, member ? copy : -1), 0, 0});
        }
    }
    const math::mat4 heading = math::mat4::rotation(doc_heading(d, thing, now), math::double3{0, 0, 1});
    if (member)   // (laid on another shape's outline, evenly, each turned the way it goes there, as Cavalry's Duplicator on a shape)
        if (const int on = (int)doc_literal(d, thing, "copy_on", 0, 0); on && on != thing && doc_number(d, on, "copy_on", 0, 0) == 0.0) {
            const auto found = d.shown.find(on);
            const Drawn* b = found == d.shown.end() ? nullptr : found->second.drawn;
            if (b && b->cut && b->outline) {
                const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
                b->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
                const double from = doc_literal(d, thing, "copy_on_offset", said, 0);
                Doc::Along& kept = d.along[thing];
                const double key = (double)ptsCnt + (ptsCnt ? pts[0].x * 1e-3 + pts[ptsCnt - 1].y * 1e-6 : 0.0) + n * 1e3 + from;
                if (kept.at != said || kept.key != key || kept.of != b) kept.at = said, kept.key = key, kept.of = b, sk_along(cmds, cmdsCnt, pts, n, (float)from, kept.each);
                if ((size_t)copy * 3 + 2 < kept.each.size()) {
                    const float* e = &kept.each[(size_t)copy * 3];
                    const math::mat4 there = doc_place(d, on, now);   // (its outline's place on the stage: as shape_cut lays it)
                    const math::double4 world = there * math::double4{(e[0] - 0.5 * b->cutW) / b->plainHigh * b->sheetHigh, (0.5 * b->plainHigh - e[1]) / b->plainHigh * b->sheetHigh, 0.0, 1.0};
                    const math::double3 way = there.upperLeft() * math::double3{std::cos(e[2]), -std::sin(e[2]), 0.0};
                    const math::mat4 frame = doc_above(d, thing, now, deeper) * math::mat4::translation(math::double3{doc_literal(d, thing, "x", now, 0), doc_literal(d, thing, "y", now, 0), doc_literal(d, thing, "z", now, 0)}) *
                                             heading * turned("rotate", {0, 0, 1}) * turned("rotate_y", {0, 1, 0}) * turned("rotate_x", {1, 0, 0});
                    const math::mat4 back = inverse(frame);
                    shift = (back * world).xyz;   // (where it lies, in the thing's own turned frame: its rows and scatter left out)
                    const math::double3 w = back.upperLeft() * way;
                    own = math::mat4::rotation(std::atan2(w.y, w.x), math::double3{0, 0, 1}) * own;
                }
            }
        }
    math::mat4 place = doc_above(d, thing, now, deeper) * math::mat4::translation(math::double3{doc_literal(d, thing, "x", now, 0), doc_literal(d, thing, "y", now, 0), doc_literal(d, thing, "z", now, 0)}) *
                       heading * turned("rotate", {0, 0, 1}) * turned("rotate_y", {0, 1, 0}) * turned("rotate_x", {1, 0, 0}) * math::mat4::translation(shift) * math::mat4::scaling(doc_literal(d, thing, "size", now, 1)) *
                       math::mat4::scaling(math::double3{doc_literal(d, thing, "scale_x", now, 1), doc_literal(d, thing, "scale_y", now, 1), doc_literal(d, thing, "scale_z", now, 1)}) *
                       own * math::mat4::translation(-anchor);
    const double reach = member ? doc_literal(d, thing, "field_radius", said, 0) : 0.0;
    if (reach > 0.0) {   // (within the field: larger and turned about where it is seen, its picture's middle, as near the field's middle as it is)
        const math::double3 at = (place * math::double4{0.0, 0.0, 0.0, 1.0}).xyz;
        const math::double3 middle{doc_literal(d, thing, "field_x", said, 0), doc_literal(d, thing, "field_y", said, 0), doc_literal(d, thing, "field_z", said, 0)};   // (a ball: near and far measured too)
        const double from = length(at - middle), band = doc_literal(d, thing, "field_band", said, 0);
        const double near = std::clamp(band > 0.0 ? 1.0 - std::fabs(from - reach) / band : 1.0 - from / reach, 0.0, 1.0), k = near * near * (3.0 - 2.0 * near);
        const math::double3 away = at - middle;
        const double r = length(away), by = doc_literal(d, thing, "field_push", said, 0);   // (pushed out as if from a hole that wide: the nearest ones all end about as far out, the hole round however they lay)
        const math::double3 push = r > 1e-9 ? normalize(away) * (by > 0.0 ? k * (std::sqrt(r * r + by * by) - r) : k * by) : math::double3{0.0};
        place = math::mat4::translation(push) * math::mat4::translation(at) * math::mat4::rotation(k * doc_literal(d, thing, "field_rotate", said, 0) * (double)TAU / 360.0, math::double3{0, 0, 1}) *
                math::mat4::scaling(std::max(0.0, 1.0 + k * doc_literal(d, thing, "field_size", said, 0))) * math::mat4::translation(-at) * place;
    }
    // (the user's three tags: 2D sticks to the camera; 2.5D stays in the world, smaller far, but keeps the angle it
    //  was given on the screen; 3D, and a solid, is filmed as it is)
    const auto shown = d.shown.find(thing);
    if (deeper || shown == d.shown.end() || shown->second.space == 3) return place;
    if (shown->second.space == 2) return d.rig * place;
    const math::double3 at = (place * math::double4{anchor, 1.0}).xyz;   // (where it stands)
    return math::mat4::translation(at) * d.facing * math::mat4::translation(-at) * place;
}
// Where a joining thing's copies lie (its connect): one from each copy of the thing it joins to another, stretched
// along the way between them and as fine as they are far apart; those left over shown nowhere. Only where its copies
// are placed, as any thing's: the lines are its own copies, in its own fill.
static std::vector<math::mat4> doc_joins(Doc& d, int thing, double now) {
    const int one = (int)doc_number(d, thing, "connect", 0, 0), two = (int)doc_number(d, thing, "connect_to", 0, 0);
    if (!one || one == thing || !d.shown.count(one) || (two && (two == thing || !d.shown.count(two)))) return {};
    const auto where = [&](int other) {   // (where each copy of it is seen, its picture's middle, before its space is taken into account)
        std::vector<math::double3> at((size_t)doc_members(d, other));
        for (size_t i = 0; i < at.size(); ++i) at[i] = (doc_place(d, other, now, 1, (int)i, true) * math::double4{0.0, 0.0, 0.0, 1.0}).xyz;
        return at;
    };
    const std::vector<math::double3> a = where(one), b = two ? where(two) : a;
    const double reach = doc_number(d, thing, "connect_reach", now, 0);
    std::vector<std::pair<size_t, size_t>> pairs;
    const size_t most = (size_t)doc_members(d, thing);
    if (two) for (size_t i = 0; i < std::min(a.size(), b.size()); ++i) pairs.push_back({i, i});
    else if (reach <= 0.0) for (size_t i = 0; i + 1 < a.size(); ++i) pairs.push_back({i, i + 1});
    else for (size_t i = 0; i < a.size() && pairs.size() < most; ++i) for (size_t j = i + 1; j < a.size() && pairs.size() < most; ++j) if (length(a[i] - a[j]) < reach) pairs.push_back({i, j});
    float2 half{0.5f};
    float deep = 0.f;
    doc_half(d, thing, &half, &deep);
    const auto shown = d.shown.find(thing);
    const int space = shown == d.shown.end() ? 3 : shown->second.space;
    std::vector<math::mat4> lines;
    for (size_t k = 0; k < pairs.size() && k < most; ++k) {
        const math::double3 from = a[pairs[k].first], to = b[pairs[k].second], mid = (from + to) * 0.5, way = to - from;
        const double far = length(way), fine = reach > 0.0 && !two ? 0.35 + 0.65 * (1.0 - far / reach) : 1.0;   // (never finer than about a third: thinner than a dot of the screen it breaks up)
        // (along the way between them however deep it goes; its width across, as level as it can lie, facing the eye)
        const math::double3 along = far > 1e-9 ? way / far : math::double3{1, 0, 0}, flat = cross(along, math::double3{0, 0, 1});
        const math::double3 across = length(flat) > 1e-6 ? normalize(flat) : math::double3{0, 1, 0}, up = cross(along, across);
        const double wide = doc_number(d, thing, "size", now, 1) * fine;
        math::mat4 line;
        line[0] = math::double4{along * (far / std::max(1e-6, 2.0 * half.x)), 0.0};
        line[1] = math::double4{-across * wide, 0.0};
        line[2] = math::double4{up, 0.0};
        line[3] = math::double4{mid, 1.0};
        lines.push_back(space == 2 ? d.rig * line : line);
    }
    return lines;
}

// The shapes the kit's mattes are cut by that `which` names, taken out of their matte's scene and done away with,
// their looks too (a look holds the picture it reads: Filament will not let that picture go while a look holds it).
static void doc_unshape(Stage& s, const std::function<bool(const Stage::Made&)>& which) {
    auto& faces = s.engine->getRenderableManager();
    for (size_t i = s.made.size(); i-- > 0;) {
        if (!s.made[i].matte || !which(s.made[i])) continue;
        const utils::Entity e = s.made[i].thing;
        MaterialInstance* look = faces.getMaterialInstanceAt(faces.getInstance(e), 0);
        for (Stage::Matte& mt : s.mattes) mt.scene->remove(e);
        s.engine->destroy(e);
        s.engine->destroy(look);
        s.made.erase(s.made.begin() + (long)i);
    }
}
// Every thing told its moment: whether it is on (from its start, for its duration if it has one), what of itself it
// shows (it goes round when it comes to its end), and where it is.
// The stage's own looks, Filament's, as the document says them (thing 0): its grade made again only when it is told
// otherwise; the glow, flare, darkened corners and fog on the stage's own view; the room's light and the colour behind.
static bool doc_knows(Doc& d, int thing, const char* name, const double* number, const char* word, bool key);
static bool doc_frame(Doc& d, int thing, double now, double* frame);
// An ISF audio input's values now (screen.h sounds): the sound of the thing its knob names (by number, on the card as
// a picture's is), at that thing's own moment as its film shows it (from its start, from where it begins, at its
// speed: motolii_loudness_over's way), read once aside (sound.h); none named, or not read yet: none (silence).
static std::vector<float> doc_sound_now(Doc& d, int thing, const std::string& file, const std::string& name, bool fft, int wide) {
    std::lock_guard<std::recursive_timed_mutex> hold(d.lock);   // (asked while the picture is drawn)
    const double now = d.stage->now;
    const int t = (int)doc_number(d, thing, doc_screen_knob(file, name).c_str(), now, 0);
    if (t <= 0 || !doc_knows(d, t, nullptr, nullptr, nullptr, false)) return {};
    const std::string heard = doc_word(d, t, "file");
    if (heard.empty()) return {};
    const SoundRead* read = sound_of(heard);
    if (!read) return {};
    const double start = doc_number(d, t, "start", 0, 0), begin = doc_number(d, t, "begin", 0, 0), speed = doc_number(d, t, "speed", 0, 1);
    return sound_at(read, fft, wide, now < start ? -1.0 : begin + (now - start) * speed);
}
static void doc_look(Doc& d) {
    Stage& s = *d.stage;
    const auto hex = [&](const char* name, float3 otherwise) { const std::string w = doc_word(d, 0, name); return w.size() == 6 ? float3{(float)((std::strtol(w.c_str(), nullptr, 16) >> 16) & 255), (float)((std::strtol(w.c_str(), nullptr, 16) >> 8) & 255), (float)(std::strtol(w.c_str(), nullptr, 16) & 255)} / 255.f : otherwise; };
    const std::string mapper = doc_word(d, 0, "tone_mapper");
    char grade[300];
    const float slope = (float)doc_number(d, 0, "slope", 0, 1), offset = (float)doc_number(d, 0, "offset", 0, 0), power = (float)doc_number(d, 0, "power", 0, 1);
    std::snprintf(grade, sizeof grade, "%s %g %g %g %g %g %g %g %g", mapper.c_str(), doc_number(d, 0, "exposure", 0, 0), doc_number(d, 0, "contrast", 0, 1), doc_number(d, 0, "saturation", 0, 1), doc_number(d, 0, "vibrance", 0, 1), doc_number(d, 0, "temperature", 0, 0), slope, offset, power);
    const auto tint = [&](const char* name) { const float3 c = hex(name, float3{1.f}); std::strncat(grade, (" " + doc_word(d, 0, name)).c_str(), 20); return float4{Color::toLinear<ACCURATE>(sRGBColor{c}), 0.f}; };
    const float4 low = tint("shadows_color"), mid = tint("midtones_color"), high = tint("highlights_color");
    if (d.graded != grade) {
        static LinearToneMapper linear; static ACESToneMapper aces; static FilmicToneMapper filmic; static AgxToneMapper agx; static PBRNeutralToneMapper neutral;
        const ToneMapper* tm = mapper == "aces" ? (const ToneMapper*)&aces : mapper == "filmic" ? (const ToneMapper*)&filmic : mapper == "agx" ? (const ToneMapper*)&agx : mapper == "pbr_neutral" ? (const ToneMapper*)&neutral : (const ToneMapper*)&linear;
        ColorGrading* was = const_cast<ColorGrading*>(s.view->getColorGrading());
        ColorGrading* now = ColorGrading::Builder().toneMapper(tm).exposure((float)doc_number(d, 0, "exposure", 0, 0)).contrast((float)doc_number(d, 0, "contrast", 0, 1))
                                .saturation((float)doc_number(d, 0, "saturation", 0, 1)).vibrance((float)doc_number(d, 0, "vibrance", 0, 1)).whiteBalance((float)doc_number(d, 0, "temperature", 0, 0), 0.f)
                                .slopeOffsetPower(float3{slope}, float3{offset}, float3{power})
                                .shadowsMidtonesHighlights(low, mid, high, float4{0.f, 0.333f, 0.55f, 1.f}).build(*s.engine);   // (the ranges as Filament's own)
        s.view->setColorGrading(now);
        if (was) s.engine->destroy(was);
        d.graded = grade;
    }
    View::BloomOptions glow = s.view->getBloomOptions();
    glow.enabled = doc_number(d, 0, "bloom", 0, 0) > 0.0, glow.strength = (float)doc_number(d, 0, "bloom", 0, 0);
    glow.lensFlare = glow.starburst = doc_number(d, 0, "lens_flare", 0, 0) > 0.0;
    glow.enabled = glow.enabled || glow.lensFlare;   // (a flare is drawn by the glow's own passes)
    glow.chromaticAberration = 0.005f * (float)doc_number(d, 0, "lens_flare", 0, 0), glow.ghostThreshold = glow.haloThreshold = 1.f;
    s.view->setBloomOptions(glow);
    const std::string smooth = doc_word(d, 0, "anti_aliasing");   // (on every picture laid: each smooths its own edges)
    for (View* v : std::vector<View*>{s.view}) {
        v->setAntiAliasing(smooth.empty() || smooth == "fxaa" ? View::AntiAliasing::FXAA : View::AntiAliasing::NONE);
        View::TemporalAntiAliasingOptions taa = v->getTemporalAntiAliasingOptions();
        taa.enabled = smooth == "taa";
        v->setTemporalAntiAliasingOptions(taa);
    }
    View::MultiSampleAntiAliasingOptions many = s.view->getMultiSampleAntiAliasingOptions();
    many.enabled = smooth == "msaa", many.sampleCount = 4;
    s.view->setMultiSampleAntiAliasingOptions(many);
    View::VignetteOptions corners = s.view->getVignetteOptions();
    corners.enabled = doc_number(d, 0, "vignette", 0, 0) > 0.0, corners.midPoint = 1.f - 0.9f * (float)doc_number(d, 0, "vignette", 0, 0), corners.color = LinearColorA{hex("vignette_color", float3{0.f}), 1.f};
    s.view->setVignetteOptions(corners);
    View::FogOptions haze = s.view->getFogOptions();
    haze.enabled = doc_number(d, 0, "fog", 0, 0) > 0.0, haze.density = 0.5f * (float)doc_number(d, 0, "fog", 0, 0), haze.distance = 2.f, haze.height = -1e3f, haze.heightFalloff = 0.f, haze.color = hex("fog_color", float3{0.85f, 0.85f, 0.9f}), haze.fogColorFromIbl = false;
    s.view->setFogOptions(haze);
    View::AmbientOcclusionOptions folds = s.view->getAmbientOcclusionOptions();
    folds.enabled = doc_number(d, 0, "ambient_occlusion", 0, 0) > 0.0, folds.intensity = 2.f * (float)doc_number(d, 0, "ambient_occlusion", 0, 0);
    s.view->setAmbientOcclusionOptions(folds);
    View::ScreenSpaceReflectionsOptions mirror = s.view->getScreenSpaceReflectionsOptions();
    mirror.enabled = doc_number(d, 0, "reflections", 0, 0) > 0.0;
    s.view->setScreenSpaceReflectionsOptions(mirror);
    if (d.roomLight <= 0.f && s.scene->getIndirectLight()) d.roomLight = s.scene->getIndirectLight()->getIntensity();
    std::string which = doc_word(d, 0, "room");
    if (which.empty()) which = "small_studio";   // (as the stage was opened)
    if (which != d.roomIs) {   // (another room: made once from its file, then kept; every picture laid over the stage's own is lit by it too)
        static std::map<std::string, IndirectLight*> made;
        if (!d.roomIs.empty() || made.empty()) made.emplace(d.roomIs.empty() ? "small_studio" : d.roomIs, s.scene->getIndirectLight());
        const auto found = made.find(which);
        if (found != made.end()) s.scene->setIndirectLight(found->second);
        else if (room(s, ("out/" + which + "/" + which + "_ibl.ktx").c_str(), d.roomLight)) made[which] = s.scene->getIndirectLight();
        d.roomIs = which;
    }
    if (IndirectLight* room = s.scene->getIndirectLight()) {
        room->setIntensity(d.roomLight * (float)doc_number(d, 0, "environment", 0, 1));
        room->setRotation(math::mat3f::rotation((float)(doc_number(d, 0, "environment_rotate", 0, 0) * (double)TAU / 360.0), math::float3{0, 1, 0}));
    }
    d.screen.sounds = [&d](const std::string& file, const std::string& name, bool fft, int wide) { return doc_sound_now(d, 0, file, name, fft, wide); };
    screen_effects(s, d.screen, doc_word(d, 0, "script"), [&d](const std::string& file, const std::string& name, double usual) {
        std::lock_guard<std::recursive_timed_mutex> hold(d.lock);   // (asked while the picture is drawn, when the hand does not hold it)
        // (a mouse's knob, where the followed thing is on the picture: pointer as the web says it, -1 to 1 and up; the
        //  others from the top left, 0 to 1, as the effects' own uv)
        static const char* const mice[] = {"pointer", "center", "mouse", "point"};
        if (const int t = (int)doc_number(d, 0, "follow", 0, 0); t > 0 && name.size() > 2 && name[name.size() - 2] == '_')
            for (const char* mouse : mice)
                if (name.compare(0, name.size() - 2, mouse) == 0) {
                    double frame[8];
                    if (!doc_knows(d, t, nullptr, nullptr, nullptr, false) || !doc_frame(d, t, d.stage->now, frame)) break;
                    const bool across = name.back() == 'x';
                    const double at = across ? frame[0] / d.stage->w : frame[1] / d.stage->h;
                    return !strcmp(mouse, "pointer") ? (across ? 2.0 * at - 1.0 : 1.0 - 2.0 * at) : at;
                }
        return doc_number(d, 0, doc_screen_knob(file, name).c_str(), d.stage->now, usual);
    });
    const std::string behind = doc_word(d, 0, "background");
    if (doc_word(d, 0, "behind") == "room") {   // (the room itself behind everything, as bright as it lights the stage: its own picture, made once a room)
        static std::map<std::string, Skybox*> shown;
        Skybox*& sky = shown[d.roomIs];
        if (!sky) {
            static std::vector<std::vector<uint8_t>> kept;
            kept.push_back(load_file(("out/" + d.roomIs + "/" + d.roomIs + "_skybox.ktx").c_str()));
            if (!kept.back().empty())
                if (Texture* tex = ktxreader::Ktx1Reader::createTexture(s.engine, new image::Ktx1Bundle(kept.back().data(), (uint32_t)kept.back().size()), false))
                    sky = Skybox::Builder().environment(tex).intensity(d.roomLight).showSun(true).build(*s.engine);   // (a sun among the lights: its disk in it)   // (Filament's sky takes its brightness as it is made)
        }
        if (sky) sky->setLayerMask(0xFF, 0x01);
        s.scene->setSkybox(sky);
    } else s.scene->setSkybox(nullptr);
    // A colour behind everything: a flat colour as large as the stage, stuck to the eye, drawn first in the stage's own
    // view and writing no depth, so every thing is drawn over it (Filament's sky is drawn after the things it holds, and
    // a flat thing writes no depth, so a sky painted over them). It is opaque: Filament draws blended sheets after every
    // solid thing whatever their priority, so a blended backdrop was painted over the solids.
    const std::string colour = doc_word(d, 0, "behind") == "room" ? "" : behind;
    if (colour != d.backdropIs) {
        if (d.backdrop) picture_off(s, d.backdrop), d.backdrop = nullptr;
        if (!colour.empty()) {
            const math::double4 up = s.camera->getProjectionMatrix() * s.camera->getViewMatrix() * math::double4{0, 1, 0, 1};
            if ((d.backdrop = flat(s, "rect", (float)s.w / (float)s.h, mat4f{}, (float)(2.0 * up.w / up.y), hex("background", float3{0.f}), "opaque"))) {   // (opaque: Filament draws it with the solid things, first by its priority, not over them with the sheets)
                auto& faces = s.engine->getRenderableManager();
                for (const Stage::Made& made : s.made)
                    if (made.of == d.backdrop->sheet) faces.setPriority(faces.getInstance(made.thing), 0), faces.setChannel(faces.getInstance(made.thing), 0);
                d.backdrop->look->setDepthWrite(false), d.backdrop->look->setDepthCulling(false);
            }
        }
        d.backdropIs = colour;
    }
    if (d.backdrop) move(s, d.backdrop->sheet, [&](int, const mat4f&) { return mat4f(d.rig); });
}

static int doc_keys(Doc& d, int thing, const char* name, double at);
// A value at its most: the highest of its keys, or what it is said to be (how far a moving effect may reach).
static double doc_fx_room(Doc& d, int thing);
static bool doc_on(Doc& d, int thing, const char* name);
// A thing's effects in the order they are laid, After Effects' stack: its "fx" names them in turn, files (.wgsl) and
// ThorVG's own alike (doc_thor_names), any order; one of ThorVG's that is on and not named (a document from before)
// lies first, in the old order (colours, shadow, glow, blur).
static std::vector<std::string> doc_fx_list(Doc& d, int thing) {
    std::vector<std::string> said, all;
    for (const std::string& w : script_files(doc_word(d, thing, "fx"))) if (w != "none" && w != "glow") said.push_back(w);   // (a glow is no step of the stack: Filament's bloom, doc_moment)
    for (const char* n : doc_thor_names) if (std::find(said.begin(), said.end(), n) == said.end() && doc_on(d, thing, n)) all.push_back(n);
    for (const std::string& w : said) if (!doc_thor_letter(w) || doc_on(d, thing, w.c_str())) all.push_back(w);
    return all;
}
// Whether a thing's effects are laid on it as the camera sees it (one of them a file: then all of them, in turn, there:
// screen.h thing_effects_draw), not in its own picture as ThorVG draws it.
static bool doc_fx_files(const std::vector<std::string>& list) {
    return std::any_of(list.begin(), list.end(), [](const std::string& w) { return !doc_thor_letter(w) && std::filesystem::path(w).extension() != ".mjs"; });   // (a splat effect is in the cloud's own look)
}
static std::string doc_fx_join(const std::vector<std::string>& list) {
    std::string all;
    for (const std::string& w : list) all += (all.empty() ? "" : " ") + w;
    return all;
}
static double doc_most(Doc& d, int thing, const char* name, double otherwise) {
    double most = doc_number(d, thing, name, 0, otherwise);
    if (doc_keys(d, thing, name, -1.0)) doc_rows(d, sqlite3_mprintf("select max(value) from fact where thing = %d and name = %Q", thing, name), [&](sqlite3_stmt* row) { most = sqlite3_column_double(row, 0); });
    return most;
}
// The effects of a flat thing at a moment (or, `most`, as far as each reaches at its most), ThorVG's own: in shares of
// its height from points of the picture (`units` to a unit of the stage, as it was made). false: a colour is not
// six digits (why says which).
static bool doc_fx(Doc& d, int thing, double at, bool most, double units, DrawnFx& fx, std::string& why, bool camera = false) {
    const auto num = [&](const char* name, double otherwise) { return most ? doc_most(d, thing, name, otherwise) : doc_number(d, thing, name, at, otherwise); };
    bool ok = true;
    const auto hue = [&](const char* name, const char* otherwise, float3* to) {   // (a colour said as six digits; not said: as it comes)
        std::string said = doc_word(d, thing, name);
        if (said.empty()) said = otherwise;
        const long rgb = said.size() == 6 && said.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(said.c_str(), nullptr, 16) : -1;
        if (rgb < 0) return ok = false, why = std::string("a ") + name + " is six digits, as ff3366", void();
        *to = float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f;
    };
    fx.blur = (float)(num("blur", 0) / units);
    if (most) fx.room = (float)(doc_fx_room(d, thing) / units);   // (the room it is made with: a power of two, so a change within it makes nothing again)
    const std::string way = doc_word(d, thing, "blur_direction");
    fx.blurWay = way == "horizontal" ? 1 : way == "vertical" ? 2 : 0;
    if (!doc_word(d, thing, "shadow").empty()) {   // (After Effects' own drop shadow, as it comes: half of it, 135 degrees, 10 points away, 10 soft)
        float3 dark{0.f};
        hue("shadow", "000000", &dark);
        fx.shadow = float4{dark, (float)num("shadow_opacity", 0.5)};
        fx.fall = (float)(num("shadow_distance", 10) / units), fx.way = (float)num("shadow_angle", 135), fx.soft = (float)(num("shadow_blur", 10) / units);
    }
    if ((fx.tint = (float)num("tint", 0)) > 0.f) hue("tint_black", "1b1f5c", &fx.tints[0]), hue("tint_white", "ff8a65", &fx.tints[1]);   // (a deep blue and a warm pink, as it comes)
    if ((fx.tritone = (float)num("tritone", 0)) > 0.f) hue("tritone_shadow", "1b1f5c", &fx.tones[0]), hue("tritone_midtone", "c44569", &fx.tones[1]), hue("tritone_highlight", "ffd166", &fx.tones[2]);
    {   // (in the order its list lays them; with a file among them all are laid as the camera sees it, none in its picture)
        const std::vector<std::string> list = doc_fx_list(d, thing);
        int n = 0;
        for (const std::string& w : list) {
            if (doc_thor_letter(w)) continue;
            if (!std::filesystem::is_regular_file(w)) ok = false, why = "there is no such file: " + w;   // (as a picture's own file: doc_match)
            else if (isf_file(w).empty()) ok = false, why = "the ISF effect could not be made WGSL: " + w;   // (made once and kept, as the stage reads it: isf.cpp)
        }
        if (!doc_fx_files(list))
            for (const std::string& w : list) if (const char k = doc_thor_letter(w); k && n < 5) fx.order[n++] = k;
        fx.order[n] = 0;
        if (!n && !camera) fx.blur = 0.f, fx.shadow.w = 0.f, fx.tint = 0.f, fx.tritone = 0.f;   // (camera: as a step of its stack lays them, screen.h thing_thor)
    }
    fx.reach = (float)(std::max(0.0, num("offset", 0)) + std::fabs(num("zigzag", 0)) + 0.5 * std::max(0.0, num("bloat", 0)) + std::fabs(num("warp", 0)));
    return ok;
}
