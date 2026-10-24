// doc_moment.h: part of doc.h (the stage told a moment). Read only through doc.h.
static void doc_moment(Doc& d, double now) {
    Stage& s = *d.stage;
    d.halves.clear(), d.talls.clear();   // (made again since: a flat thing's sheet is as large as its picture now)
    const std::vector<int> lie = doc_laid(d);
    // Each thing as it is now, in the order they lie: whether it is on, and whether it is a solid thing.
    struct Lies { int thing; Doc::Shown* shown; bool on, solid; double since; };
    std::vector<Lies> lies;
    bool solo = false;   // (while any thing is solo, only those are seen)
    for (int thing : lie) solo = solo || doc_number(d, thing, "solo", 0, 0) > 0.5;
    for (int thing : lie) {
        const auto found = d.shown.find(thing);
        if (found == d.shown.end() || found->second.camera) continue;
        const double since = now - doc_number(d, thing, "start", 0, 0), lasts = doc_number(d, thing, "duration", 0, -1);
        Lies one{thing, &found->second, since >= 0.0 && (lasts < 0.0 || since < lasts) && doc_number(d, thing, "visible", 0, 1) > 0.5 && (!solo || doc_number(d, thing, "solo", 0, 0) > 0.5), found->second.space != 2, since};   // (a sheet in the world settles depth with the solid things)
        lies.push_back(one);
    }
    // The camera: the nearest one in the order of the things that is on (the past repository's choice); none, the
    // stage's own. It looks at where it is, from round about it as it is turned (y: about, x: up and down, z: its lean),
    // `distance` times the stage's own way off, `zoom` times as near.
    {
        int camera = 0;
        for (int thing : lie)
            if (d.shown.count(thing) && d.shown[thing].camera) {
                const double since = now - doc_number(d, thing, "start", 0, 0), lasts = doc_number(d, thing, "duration", 0, -1);
                if (since >= 0.0 && (lasts < 0.0 || since < lasts) && doc_number(d, thing, "visible", 0, 1) > 0.5) camera = thing;
            }
        const auto turned = [&](const char* name, math::double3 about) { return camera ? math::mat4::rotation(doc_number(d, camera, name, now, 0) * (double)TAU / 360.0, about) : math::mat4{}; };
        const math::mat4 round = turned("rotate_y", {0, 1, 0}) * turned("rotate_x", {1, 0, 0}) * turned("rotate", {0, 0, 1});
        const math::double3 look = camera ? (doc_place(d, camera, now) * math::double4{0, 0, 0, 1}).xyz : math::double3{0.0};
        const double far = camera ? doc_number(d, camera, "distance", now, 1) : 1.0, near = camera ? doc_number(d, camera, "zoom", now, 1) : 1.0;
        const math::double3 eye = look + (round * math::double4{0, 0, 6.0 * far, 0}).xyz, up = (round * math::double4{0, 1, 0, 0}).xyz;
        s.camera->lookAt(eye, look, up);
        const double fov = 2.0 * std::atan(std::tan(15.0 * (double)TAU / 360.0) / near);
        if (camera && doc_word(d, camera, "projection") == "orthographic") {   // (as large at the point it looks at as the perspective one shows it there)
            const double high = 6.0 * far * std::tan(0.5 * fov), wide = high * double(s.w) / double(s.h);
            s.camera->setProjection(Camera::Projection::ORTHO, -wide, wide, -high, high, 0.05, 60.0 * std::max(1.0, far));
        } else s.camera->setProjection(fov * 360.0 / (double)TAU, double(s.w) / double(s.h), 0.05, 60.0 * std::max(1.0, far));
        s.camera->setShift(camera ? math::double2{doc_number(d, camera, "shift_x", now, 0), doc_number(d, camera, "shift_y", now, 0)} : math::double2{0.0});
        d.rig = s.camera->getModelMatrix() * math::mat4::translation(math::double3{0, 0, -6});   // (from the stage's own camera to this one)
        // (its lens, Filament's own: sharp at the focus, as blurred off it as the f-number says; the shutter goes with the
        //  f-number so the picture is as bright as at the stage's own f/16 and 1/125 s)
        const double said = camera ? doc_number(d, camera, "aperture", now, 0) : 0.0, focus = camera ? doc_number(d, camera, "focus", now, 0) : 0.0;
        d.blurred = said > 0.0;
        const double stop = std::max(said, 0.5);   // (Filament's widest)
        View::DepthOfFieldOptions lens = s.view->getDepthOfFieldOptions();
        if (camera && lens.cocScale != (float)doc_number(d, camera, "blur_level", now, 1)) lens.cocScale = (float)doc_number(d, camera, "blur_level", now, 1), s.view->setDepthOfFieldOptions(lens);
        s.camera->setExposure(d.blurred ? (float)stop : 16.f, d.blurred ? (float)(stop * stop / (16.0 * 16.0 * 125.0)) : 1.f / 125.f, 100.f);
        s.camera->setFocusDistance((float)(focus > 0.0 ? focus : length(eye - look)));
        d.facing = math::mat4(d.rig.upperLeft());
    }
    doc_look(d);
    d.round = doc_seconds(d);
    // Who is before whom (the past repository's law of 2026-09-12: a flat thing is not in the world). Flat things take
    // no part in depth and lie by the order of the things alone; solid things that follow one another settle depth
    // among themselves; a run of the one sort lies whole over the run before it. All in the stage's own view, the runs
    // kept in order by Filament's channels; within a run of flat things, the blend order. A flat thing's looks neither
    // ask nor write depth.
    std::vector<int> runOf(lies.size(), -1), sorts;
    for (size_t i = 0; i < lies.size(); ++i) {
        if (!lies[i].on || lies[i].shown->camera || lies[i].shown->behaviour || !lies[i].shown->light.isNull()) continue;   // (a camera, a behaviour or a light is in no run: nothing of it is drawn)
        const int sort = lies[i].solid ? 1 : 0;
        if (sorts.empty() || sorts.back() != sort) sorts.push_back(sort);
        runOf[i] = (int)sorts.size() - 1;
    }
    const int base = sorts.size() > 1 && sorts[0] == 0 && sorts[1] == 1 ? 1 : 0;   // (the last run in the stage's own view: flat things under the first solid ones)
    for (size_t i = 0; i < lies.size(); ++i) {
        if (runOf[i] < 0) continue;
        const int run = runOf[i];
        Doc::Shown& t = *lies[i].shown;
        const bool own = doc_fx_files(doc_fx_list(d, lies[i].thing));   // (with effects of its own: drawn alone, its picture laid here: thing_fx.h)
        Scene* into = own ? thing_fx_made(s, d.thingFx[lies[i].thing]).view->getScene() : s.scene;   // (one view: every other run in the stage's own scene, kept in order by Filament's channels)
        Scene* const was = t.scene ? t.scene : s.scene;
        if (was != into) {   // (into the scene of its run; one not seen goes in when it is seen)
            if (t.in) doc_entities(s, t, [&](utils::Entity e) { was->remove(e), into->addEntity(e); });
            t.scene = into;
        }
        if (const int channel = std::min(run, 7); channel != t.channel) {
            doc_parts(s, t, [&](RenderableManager& faces, RenderableManager::Instance one) { faces.setChannel(one, (uint8_t)channel); });
            t.channel = channel;
        }
        if (t.run < 0 && !lies[i].solid) doc_looks(s, t, [](MaterialInstance* look) { look->setDepthCulling(false), look->setDepthWrite(false); });
        if (t.lies != (int)i && !lies[i].solid)
            doc_parts(s, t, [&](RenderableManager& faces, RenderableManager::Instance one) {
                for (size_t part = 0; part < faces.getPrimitiveCount(one); ++part) faces.setBlendOrderAt(one, part, (uint16_t)(i * 128)), faces.setGlobalBlendOrderEnabledAt(one, part, true);   // (128 places for each: its copies lie among themselves by how far off they are)
            });
        t.run = run, t.lies = (int)i;
        if (own) {   // (its sheet where it lies: the same channel and the same blend order as it has)
            auto& faces = s.engine->getRenderableManager();
            for (const Stage::Made& made : s.made)
                if (made.of == d.thingFx[lies[i].thing].sheet) {
                    const auto one = faces.getInstance(made.thing);
                    faces.setCulling(one, false), faces.setChannel(one, (uint8_t)std::min(run, 7)), faces.setBlendOrderAt(one, 0, (uint16_t)(i * 128)), faces.setGlobalBlendOrderEnabledAt(one, 0, true);   // (its points are the screen's corners: never culled, as image.mat's sheet is not)
                }
        }
    }
    for (auto e = d.thingFx.begin(); e != d.thingFx.end();) {   // (no longer with effects of its own, or gone: let go)
        const auto at = std::find_if(lies.begin(), lies.end(), [&](const auto& l) { return l.thing == e->first; });
        if (at == lies.end() || runOf[(size_t)(at - lies.begin())] < 0 || !doc_fx_files(doc_fx_list(d, e->first))) thing_fx_off(s, e->second), e = d.thingFx.erase(e);
        else ++e;
    }
    s.before.clear();
    for (auto& [thing, e] : d.thingFx) {   // (each drawn alone into a picture, its effects run on it, before the stage's own view: thing_fx.h)
        thing_effects(s, e.st, doc_fx_join(doc_fx_list(d, thing)));
        const ScreenNumbers numbers = [&d, thing = thing, &e = e](const std::string& file, const std::string& name, double usual) {
            std::lock_guard<std::recursive_timed_mutex> hold(d.lock);   // (asked while the picture is drawn)
            if (name == "origin_x" || name == "origin_y" || name == "turn" || name == "extent") {   // (where the thing is on the picture now, how turned, how large: an effect's pattern stuck to it)
                double frame[8];
                if (!doc_frame(d, thing, d.stage->now, frame)) return usual;
                return name == "origin_x" ? frame[0] - e.box[0] : name == "origin_y" ? frame[1] - e.box[1] : name == "turn" ? frame[4] : std::max(frame[2], frame[3]);   // (on its own picture: the box it is seen in)
            }
            return doc_number(d, thing, doc_screen_knob(file, name).c_str(), d.stage->now, usual);
        };
        const auto thorFx = [&d, thing = thing] {   // (ThorVG's own among its steps, as they are now: lengths in points of the stage)
            std::lock_guard<std::recursive_timed_mutex> hold(d.lock);
            DrawnFx fx;
            std::string why;
            doc_fx(d, thing, d.stage->now, false, 1.0, fx, why, true);
            double frame[8];
            const auto t = d.shown.find(thing);
            if (t != d.shown.end() && t->second.drawn && t->second.drawn->fx.overHigh > 0.f && doc_frame(d, thing, d.stage->now, frame)) {
                const float k = (float)(frame[3] / (t->second.units * t->second.drawn->fx.overHigh));
                fx.blur *= k, fx.fall *= k, fx.soft *= k;
            }
            return fx;
        };
        s.before.push_back([&s, &d, thing = thing, &e = e, numbers, thorFx] {
            double box[4] = {0, 0, (double)s.w, (double)s.h}, seen[4];
            {
                std::lock_guard<std::recursive_timed_mutex> hold(d.lock);   // (asked while the picture is drawn)
                if (doc_seen_box(d, thing, d.stage->now, seen))   // (where it is seen, as much of it as is on the stage: its picture, After Effects' layer)
                    box[0] = std::max(0.0, seen[0]), box[1] = std::max(0.0, seen[1]), box[2] = std::min((double)s.w, seen[2]), box[3] = std::min((double)s.h, seen[3]);
                if (box[2] - box[0] < 1 || box[3] - box[1] < 1) box[0] = box[1] = 0, box[2] = s.w, box[3] = s.h;   // (none of it on the stage: the whole)
            }
            thing_fx_aim(s, e, box);
            thing_effects_draw(s, e.st, e.view, numbers, thorFx);
            if (e.st.seen && e.look) e.look->setParameter("image", e.st.seen, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
        });
    }
    {   // (a glow: Filament's own, a thing past 1 bright taken up by the stage's bloom (its threshold), as emissive is)
        float most = 0.f;
        for (size_t i = 0; i < lies.size(); ++i)
            if (runOf[i] >= 0 && !doc_word(d, lies[i].thing, "glow").empty()) most = std::max(most, (float)doc_number(d, lies[i].thing, "glow_opacity", now, 1));
        if (most > 0.f) {
            View::BloomOptions glow = s.view->getBloomOptions();
            glow.enabled = true, glow.threshold = true, glow.strength = std::max(glow.strength, std::clamp(0.35f * most, 0.f, 1.f));
            s.view->setBloomOptions(glow);
        }
    }
    View::DepthOfFieldOptions lens = s.view->getDepthOfFieldOptions();   // (the camera's lens, on the stage's own view when solid things are in it)
    if (lens.enabled != (d.blurred && (base == 1 || (!sorts.empty() && sorts[0]))))
        lens.enabled = !lens.enabled, s.view->setDepthOfFieldOptions(lens);
    for (size_t i = 0; i < lies.size(); ++i) {
        const int thing = lies[i].thing;
        Doc::Shown& t = *lies[i].shown;
        const bool in = lies[i].on;
        const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.shape;
        if (in != t.in) {   // (in or out of the scene of its run)
            Scene* sc = t.scene ? t.scene : s.scene;
            doc_entities(s, t, [&](utils::Entity e) { in ? sc->addEntity(e) : sc->remove(e); });
            t.in = in;
        }
        if (!in) continue;
        if (t.film) {   // (the film at its moment: on a live stage once the document is let go)
            if (s.later) s.later->push_back([&s, film = t.film, since = (float)lies[i].since] { video_at(s, film, since); });
            else video_at(s, t.film, (float)lies[i].since);
        }
        if (t.film && t.film->look) {   // (its own look, in its material, as ThorVG's effects give a picture its: blurred (in points of the film), tinted, tritoned)
            DrawnFx fx;
            std::string why;
            doc_fx(d, thing, now, false, 1.0, fx, why);
            const std::string way = doc_word(d, thing, "blur_direction");
            const float blur = (float)doc_number(d, thing, "blur", now, 0);
            const float4 grade[6] = {{way == "vertical" ? 0.f : blur / (float)t.film->w, way == "horizontal" ? 0.f : blur / (float)t.film->h, fx.tint, fx.tritone},
                                     {fx.tints[0], 0.f}, {fx.tints[1], 0.f}, {fx.tones[0], 0.f}, {fx.tones[1], 0.f}, {fx.tones[2], 0.f}};
            t.film->look->setParameter("grade", grade, 6);
        }
        if (t.solid && !t.solid->moves.empty()) solid_at(t.solid, doc_word(d, thing, "animation"), lies[i].since * doc_number(d, thing, "speed", 0, 1));   // (its own movement, from when it comes on)
        if (t.drawn && t.drawn->seconds > 0.f) picture_at(s, t.drawn, std::fmod((float)lies[i].since, t.drawn->seconds));
        if (t.drawn && !t.drawn->effects.empty()) {   // (its effects as they are now: those that move are drawn again)
            DrawnFx fx;
            std::string why;
            if (doc_fx(d, thing, now, false, t.units, fx, why)) fx_at(s, t.drawn, fx);
        }
        if (t.drawn && t.drawn->shape) trim_at(s, t.drawn, (float)doc_number(d, thing, "trim", now, 0), (float)doc_number(d, thing, "trim_begin", now, 0), (float)doc_number(d, thing, "trim_offset", now, 0));
        if (t.drawn && t.drawn->cut) {   // (a shape in triangles: bent in its look, and its copies' own looks, nothing drawn again)
            const auto num = [&](const char* name, double otherwise) { return (float)doc_number(d, thing, name, now, otherwise); };
            const float deg = (float)(TAU / 360.0), since = (float)lies[i].since;
            bool driven = false;   // (a bend given each copy by a behaviour: each copy's own look told its own)
            float4 region{0.f};   // (a bend's behaviour's falloff, point by point in the look: a round where its shape lies, as large as it is)
            const auto bendsOf = [&](int copy, int n, float4* bends, const math::double3* where = nullptr) {
                const auto at = [&](const char* name, double otherwise) {
                    if (const int by = doc_driven_by(d, thing, name)) {
                        driven = true;
                        if (const int f = (int)doc_number(d, by, "falloff", 0, 0); f && d.shown.count(f) && d.shown[f].drawn) {   // (weighed point by point, not copy by copy)
                            const math::mat4 there = doc_place(d, f, now, 0, 0, false);
                            const math::double4 middle = there * math::double4{0, 0, 0, 1};
                            const double r = length((there * math::double4{0.5 * d.shown[f].drawn->sheetHigh, 0, 0, 0}).xyz), soft = doc_number(d, by, "falloff_soft", now, 0);
                            region = float4{(float)middle.x, (float)middle.y, (float)r, (float)(doc_number(d, by, "falloff_invert", 0, 0) > 0.5 ? -std::max(soft, 1e-4) : soft)};
                            return (float)doc_behaviour_at(d, by, copy, n, now);
                        }
                        return (float)doc_given(d, by, copy, n, now, num(name, otherwise), where);
                    }
                    return num(name, otherwise);
                };
                const float4 all[7] = {{at("warp", 0), at("warp_width", 0.5), at("warp_speed", 1) * since, at("warp_angle", 0) * deg},
                                       {at("bend", 0) * deg, doc_word(d, thing, "bend_direction") == "vertical" ? 1.f : 0.f, at("twist", 0) * deg, at("pinch", 0)},
                                       {at("skew", 0) * deg, at("skew_axis", 0) * deg, at("squash", 0), 0.5f * t.drawn->wideBy},
                                       {at("wiggle", 0), at("wiggle_detail", 3), at("wiggle_speed", 1) * since, num("seed", 0) * 7.31f},
                                       {0.f, 0.f, num("stretch", 0), 0.f},
                                       {at("pin_tl_x", 0), at("pin_tl_y", 0), at("pin_tr_x", 0), at("pin_tr_y", 0)},
                                       {at("pin_bl_x", 0), at("pin_bl_y", 0), at("pin_br_x", 0), at("pin_br_y", 0)}};
                std::copy(all, all + 7, bends);
                bends[7] = region;
            };
            float4 bends[8];
            bendsOf(0, 1, bends);
            if (bends[4].z != 0.f) {   // (how it moves: where its middle was a sixtieth of a second ago, seen as it is now turned and sized)
                const math::mat4 was = doc_place(d, thing, now - 1.0 / 60.0), is = doc_place(d, thing, now);
                const math::float4 back = inverse(is) * was * math::float4{0, 0, 0, 1};
                bends[4].x = (float)(-60.0 * back.x), bends[4].y = (float)(-60.0 * back.y);
            }
            t.drawn->look->setParameter("bends", bends, 8);
            if (t.drawn->edgeLook) t.drawn->edgeLook->setParameter("bends", bends, 8);
            // (another shape's path as it is now, where it lies, in this one's points: as shape_cut lays them; and each of its copies, `all`)
            const auto other = [&](int with, bool all) {
                Drawn::Other o;
                const auto found = d.shown.find(with);
                if (with == thing || found == d.shown.end() || !found->second.drawn || !found->second.drawn->cut) return o;
                const Drawn* a = t.drawn; const Drawn* b = found->second.drawn;
                const math::mat4 back = inverse(doc_place(d, thing, now));
                const tvg::PathCommand* cmds; const tvg::Point* pts; uint32_t cmdsCnt = 0, ptsCnt = 0;
                b->outline->path(&cmds, &cmdsCnt, &pts, &ptsCnt);
                const int many = all ? doc_members(d, with) : 1;
                for (int copy = 0; copy < many; ++copy) {   // (a plate cut by a whole grid of them)
                    const math::mat4 to = back * (many > 1 ? doc_place(d, with, now, 0, copy, true) : doc_place(d, with, now));
                    o.by.insert(o.by.end(), cmds, cmds + cmdsCnt);
                    for (uint32_t k = 0; k < ptsCnt; ++k) {
                        const math::double4 on = to * math::double4{(pts[k].x - 0.5 * b->cutW) / b->plainHigh * b->sheetHigh, (0.5 * b->plainHigh - pts[k].y) / b->plainHigh * b->sheetHigh, 0.0, 1.0};
                        o.at.push_back({(float)(on.x / a->sheetHigh * a->plainHigh + 0.5 * a->cutW), (float)(0.5 * a->plainHigh - on.y / a->sheetHigh * a->plainHigh)});
                    }
                }
                return o;
            };
            Drawn::Other join, morph;   // (put together with another, Skia's op in skpath.cpp's order; turned into another, so far)
            const std::string how = doc_word(d, thing, "boolean");
            for (int k = 0; k < 4; ++k)
                if (how == std::array<const char*, 4>{"subtract", "intersect", "union", "exclude"}[(size_t)k]) join = other((int)doc_number(d, thing, "boolean_with", 0, 0), true), join.how = k;
            if (const float much = num("morph", 0); much > 0.f) morph = other((int)doc_number(d, thing, "morph_to", 0, 0), false), morph.much = much;
            if (const int steps = (int)doc_number(d, thing, "morph_steps", 0, 0); steps > 1) morph = other((int)doc_number(d, thing, "morph_to", 0, 0), false), morph.much = 1.f, morph.how = std::min(steps, 64);   // (a blend: every step at once)
            Drawn::Other trace;   // (another's track over the last so many seconds, its middle where it was, as Cavalry's Trails: this one's path)
            if (const int with = (int)doc_number(d, thing, "trail", 0, 0); with && with != thing && d.shown.count(with)) {
                const math::mat4 back = inverse(doc_place(d, thing, now));
                const double length = std::max(1e-3, doc_number(d, thing, "trail_length", now, 1));
                const Drawn* a = t.drawn;
                // (on a grid of 64 ticks over its length, fixed in time: a tick worked out once is kept, so a moment only works out
                //  the ticks new since; its ends where the other was then and is now)
                const double step = length / 64.0, last = std::floor(now / step);
                if (d.tracks.size() > 200000) d.tracks.clear();
                const auto was = [&](double t, long long tick) {
                    if (tick < 0) return (doc_shown_at(d, with, t) * math::double4{0, 0, 0, 1}).xyz;
                    auto found = d.tracks.find({with, step, tick});
                    if (found == d.tracks.end()) found = d.tracks.emplace(std::make_tuple(with, step, tick), (doc_shown_at(d, with, t) * math::double4{0, 0, 0, 1}).xyz).first;
                    return found->second;
                };
                const auto put = [&](math::double3 world) {
                    const math::double4 on = back * math::double4{world, 1.0};
                    trace.by.push_back(trace.by.empty() ? tvg::PathCommand::MoveTo : tvg::PathCommand::LineTo);
                    trace.at.push_back({(float)(on.x / a->sheetHigh * a->plainHigh + 0.5 * a->cutW), (float)(0.5 * a->plainHigh - on.y / a->sheetHigh * a->plainHigh)});
                };
                put(was(now - length, -1));
                for (long long k = (long long)last - 63; k <= (long long)last; ++k) if ((double)k * step > now - length) put(was((double)k * step, k));
                put(was(now, -1));
            }
            cut_at(s, t.drawn, float3{num("trim", 0), num("trim_begin", 0), num("trim_offset", 0)},   // (its path anew, if it is told otherwise: cut again)
                   {num("round", 0), num("bloat", 0), num("zigzag", 0), num("zigzag_ridges", 5), num("zigzag_smooth", 0), num("offset", 0), num("rough", 0), num("rough_detail", 0.03), std::round(num("resample", 0)), num("voxel", 0)}, join, morph, trace);
            t.drivenLooks = driven;
            const int members = driven ? doc_members(d, thing) : 1;
            auto& faces = s.engine->getRenderableManager();
            for (const Stage::Made& made : s.made) {
                if (made.of != t.drawn->sheet || made.matte) continue;
                if (driven && !made.many) {   // (each copy its own bends, in its own look and its edge's)
                    float4 own[8];
                    const math::double3 where{made.place[3].x, made.place[3].y, made.place[3].z};   // (where this copy lies: what a falloff weighs)
                    bendsOf(std::max(0, made.nth), members, own, &where);
                    if (own[4].z == 0.f) own[4] = bends[4];
                    const auto one = faces.getInstance(made.thing);
                    MaterialInstance*& look = d.copyLooks[made.thing.getId()];
                    if (!look) look = MaterialInstance::duplicate(t.drawn->look), faces.setMaterialInstanceAt(one, 0, look);
                    look->setParameter("bends", own, 8);
                    if (t.drawn->edgeLook && faces.getPrimitiveCount(one) > 1) {
                        MaterialInstance*& edge = d.copyEdges[made.thing.getId()];
                        if (!edge) edge = MaterialInstance::duplicate(t.drawn->edgeLook), faces.setMaterialInstanceAt(one, 1, edge);
                        edge->setParameter("bends", own, 8);
                    }
                } else if (const auto own = d.copyLooks.find(made.thing.getId()); own != d.copyLooks.end()) own->second->setParameter("bends", bends, 8);
            }
        }
        if (t.drawn && t.drawn->shape)
            bend_at(s, t.drawn, (float)doc_number(d, thing, "round", now, 0), (float)doc_number(d, thing, "bloat", now, 0), (float)doc_number(d, thing, "zigzag", now, 0),
                    (float)doc_number(d, thing, "zigzag_ridges", now, 5), (float)doc_number(d, thing, "zigzag_smooth", now, 0), (float)doc_number(d, thing, "offset", now, 0));
        const mat4f place = mat4f(doc_place(d, thing, now));
        const bool staggered = doc_staggered(d, thing, now);   // (each copy, the first too, as late as its order says, waving as its place in the wave says)
        const std::vector<math::mat4> joins = doc_number(d, thing, "connect", 0, 0) != 0.0 ? doc_joins(d, thing, now) : std::vector<math::mat4>{};   // (a joining thing: its copies lie between another's)
        if (t.grains && t.look) t.look->setParameter("dissolve", (float)doc_number(d, thing, "dissolve", now, 0)), t.look->setParameter("grain", (float)doc_number(d, thing, "grain", now, 1));
        if (t.grains && t.look && t.splatFx) {   // (its effect's uniforms at its own time: the knobs as the document says them, then the effect's updateEffect)
            const auto said = t.splatFx->uniforms(std::max(0.0, now - doc_number(d, thing, "start", 0, 0)), 1.0 / 60.0, [&](const std::string& knob, double usual) { return doc_number(d, thing, doc_screen_knob(t.splatFxFile, knob).c_str(), now, usual); });
            for (const auto& [n, f] : said) {
                if (f.size() == 1) t.look->setParameter(n.c_str(), f[0]);
                else if (f.size() == 2) t.look->setParameter(n.c_str(), float2{f[0], f[1]});
                else if (f.size() == 3) t.look->setParameter(n.c_str(), float3{f[0], f[1], f[2]});
                else if (f.size() >= 4) t.look->setParameter(n.c_str(), float4{f[0], f[1], f[2], f[3]});
            }
            if (t.splatPlain && t.splatFx->finished() != t.splatDone) {   // (finished: its plain look, as PlayCanvas takes the shader off; back before it: the effect's again)
                t.splatDone = t.splatFx->finished();
                doc_parts(s, t, [&](RenderableManager& faces, RenderableManager::Instance one) { faces.setMaterialInstanceAt(one, 0, t.splatDone ? t.splatPlain : t.look); });
            }
        }
        if (t.solid) solid_to(s, t.solid, place);
        else if (doc_number(d, thing, "connect", 0, 0) != 0.0) move(s, m, [&](int nth, const mat4f&) { return (size_t)nth < joins.size() ? mat4f(joins[(size_t)nth]) : mat4f::scaling(float3{0.f}); });
        else {
            std::vector<float> far;   // (how far off each copy is from the eye: those further off drawn first, within the thing's own place)
            const mat4f eye = mat4f(s.camera->getViewMatrix());
            const std::string fills = doc_word(d, thing, "copy_fill");
            const bool flatThing = !lies[i].solid && (fills.empty() || fills == "none");   // (coloured along their order: drawn in it, not by how far off)
            move(s, m, [&](int nth, const mat4f&) {
                const mat4f at = nth || staggered ? mat4f(doc_place(d, thing, now, 0, nth, staggered)) : place;
                if ((size_t)nth >= far.size()) far.resize((size_t)nth + 1, 0.f);
                far[(size_t)nth] = -(eye * at[3]).z;
                return at;
            }, flatThing ? &eye : nullptr);   // (copies drawn in one go: those further off first, as each copy's own order does it below)
            if (far.size() > 1 && !lies[i].solid && t.lies >= 0) {
                std::vector<size_t> order(far.size());
                for (size_t k = 0; k < order.size(); ++k) order[k] = k;
                std::stable_sort(order.begin(), order.end(), [&](size_t a, size_t b) { return far[a] > far[b]; });
                std::vector<uint16_t> place_of(far.size());
                for (size_t k = 0; k < order.size(); ++k) place_of[order[k]] = (uint16_t)(k * 127 / std::max<size_t>(1, order.size() - 1));
                auto& faces = s.engine->getRenderableManager();
                for (const Stage::Made& made : s.made) {
                    if (made.of != m || made.nth < 0 || (size_t)made.nth >= place_of.size()) continue;
                    const auto one = faces.getInstance(made.thing);
                    for (size_t part = 0; part < faces.getPrimitiveCount(one); ++part) faces.setBlendOrderAt(one, part, (uint16_t)(t.lies * 128 + place_of[(size_t)made.nth]));
                }
            }
        }
        if (MaterialInstance* look = t.film ? t.film->look : t.drawn ? t.drawn->look : nullptr) look->setParameter("opacity", (float)doc_number(d, thing, "opacity", now, 1));
        if (t.drawn && t.drawn->edgeLook) t.drawn->edgeLook->setParameter("opacity", (float)doc_number(d, thing, "opacity", now, 1));
        if (MaterialInstance* main = t.film ? t.film->look : t.drawn ? t.drawn->look : nullptr) {   // (its copies coloured each their own, along a ramp in their order: a look of its own each, told what the thing's is)
            std::vector<float3> ramp;
            std::istringstream said(doc_word(d, thing, "copy_fill"));
            for (std::string one; said >> one;) { if (one == "none") break; const long rgb = std::strtol(one.c_str(), nullptr, 16); ramp.push_back(Color::toLinear<ACCURATE>(sRGBColor{float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f})); }
            if (ramp.size() == 1) ramp.push_back(ramp[0]);   // (one colour: every copy in it)
            const std::vector<float2>& letters = t.drawn ? t.drawn->letters : std::vector<float2>{};
            const auto scripted = d.scripted.find(thing);   // (a script's copies each as clear as it says)
            const bool clears = scripted != d.scripted.end() && std::any_of(scripted->second.copies.begin(), scripted->second.copies.end(), [](const ScriptCopy& c) { return c.opacity != 1.0; });
            const bool clearEach = doc_driven_by(d, thing, "opacity") != 0, pickEach = doc_driven_by(d, thing, "copy_fill_at") != 0;   // (given each copy by a behaviour)
            if (ramp.size() < 2 && letters.empty() && !clears && !t.drivenLooks && !clearEach && !pickEach) doc_copy_looks_off(d, m, main);
            else {
                auto& faces = s.engine->getRenderableManager();
                const int n = doc_members(d, thing);
                for (const Stage::Made& made : s.made) {
                    if (made.of != m || made.matte) continue;
                    MaterialInstance*& own = d.copyLooks[made.thing.getId()];
                    if (!own) own = MaterialInstance::duplicate(main), faces.setMaterialInstanceAt(faces.getInstance(made.thing), 0, own);
                    if (made.many) {   // (drawn with others in one go: the ramp and where this draw's first copy lies along it, flat.mat colours each)
                        float4 colours[8] = {};
                        for (size_t k = 0; k < ramp.size() && k < 8; ++k) colours[k] = float4{ramp[k], 1.f};
                        own->setParameter("ramp", colours, 8);
                        own->setParameter("rampOf", ramp.size() >= 2 ? float4{(float)made.nth, (float)n, (float)std::min<size_t>(ramp.size(), 8), doc_word(d, thing, "copy_fill_way") == "cycle" ? 1.f : 0.f} : float4{0.f});
                        if (ramp.size() < 2) own->setParameter("wash", t.drawn ? t.drawn->wash : float4{0.f});
                    }
                    else if (ramp.size() >= 2 && doc_word(d, thing, "copy_fill_way") == "cycle") own->setParameter("wash", float4{ramp[(size_t)made.nth % ramp.size()], 1.f});   // (one each in turn)
                    else if (ramp.size() >= 2) {   // (along the ramp in their order; or where a behaviour says, copy_fill_at, 0 its first colour, 1 its last)
                        const math::double3 where{made.place[3].x, made.place[3].y, made.place[3].z};
                        const double order = n > 1 ? (double)made.nth / (double)(n - 1) : 0.0;
                        const double pick = pickEach ? std::clamp(doc_given(d, doc_driven_by(d, thing, "copy_fill_at"), made.nth, n, now, order, &where), 0.0, 1.0) : order;
                        const float along = (float)pick * (float)(ramp.size() - 1);
                        const size_t i = std::min((size_t)along, ramp.size() - 2);
                        own->setParameter("wash", float4{mix(ramp[i], ramp[i + 1], along - (float)i), 1.f});
                    } else own->setParameter("wash", t.drawn ? t.drawn->wash : float4{0.f});   // (its own: a shape cut in triangles is filled by it)
                    if ((size_t)made.nth < letters.size()) own->setParameter("window", float4{letters[(size_t)made.nth].x, letters[(size_t)made.nth].y, 1.f, 0.f});   // (its own letter)
                    const double clear = scripted != d.scripted.end() && (size_t)made.nth < scripted->second.copies.size() ? scripted->second.copies[(size_t)made.nth].opacity : 1.0;
                    const math::double3 here{made.place[3].x, made.place[3].y, made.place[3].z};
                    const int clearBy = doc_driven_by(d, thing, "opacity");
                    const double opacity = !clearEach ? doc_number(d, thing, "opacity", now, 1)
                                         : doc_among(doc_relations, doc_word(d, clearBy, "file")) ? std::clamp(doc_literal(d, thing, "opacity", now, 1), 0.0, 1.0)   // (a relation's: by the thing's place among its things)
                                         : std::clamp(doc_given(d, clearBy, made.nth, n, now, doc_number(d, thing, "opacity", now, 1), &here), 0.0, 1.0);
                    own->setParameter("opacity", (float)(opacity * std::clamp(clear, 0.0, 1.0)));
                    if (t.drawn && t.drawn->edgeLook && !made.many) {   // (its edge's own look: a frame alone (filled with nothing) coloured each as its fill would be; each as clear)
                        const auto one = faces.getInstance(made.thing);
                        if (faces.getPrimitiveCount(one) > 1) {
                            MaterialInstance*& edge = d.copyEdges[made.thing.getId()];
                            if (!edge) edge = MaterialInstance::duplicate(t.drawn->edgeLook), faces.setMaterialInstanceAt(one, 1, edge);
                            if (t.drawn->hollow && ramp.size() >= 2) {
                                edge->setParameter("wash", own->getParameter<float4>("wash"));
                            }
                            edge->setParameter("opacity", (float)(opacity * std::clamp(clear, 0.0, 1.0)));
                        }
                    }
                    own->setParameter("glowing", (float)doc_number(d, thing, "emissive", now, 0));
                }
            }
        }
        const float shine = (float)doc_number(d, thing, "emissive", now, 0) + (doc_word(d, thing, "glow").empty() ? 0.f : 0.35f * (float)doc_number(d, thing, "glow_opacity", now, 1));   // (how much it lights itself now: a flat thing's sheet, a solid's own faces; a glow past 1, for the bloom)
        doc_looks(s, t, [&](MaterialInstance* look) { if (look->getMaterial()->hasParameter("glowing")) look->setParameter("glowing", shine); });
    }
    // Soft shadows (Filament's PCSS) on the stage's own view while any light that is on says so; else its crisp ones.
    {
        bool soft = false;
        for (const Lies& one : lies) soft = soft || (one.on && !one.shown->light.isNull() && doc_number(d, one.thing, "shadow_softness", 0, 0) > 0.0);
        s.view->setShadowType(soft ? View::ShadowType::PCSS : View::ShadowType::PCF);
    }
    // The lights: in every scene of the stage while each is on (the stage's own and every picture laid over it), where
    // it is, shining the way it is turned (ahead of it), as strong as it is now; 1 as bright as the stage's own light.
    for (const Lies& one : lies) {
        const utils::Entity e = one.shown->light;
        if (e.isNull()) continue;
        std::vector<Scene*> all{s.scene};
        for (Scene* sc : all) one.on ? (sc->hasEntity(e) ? void() : sc->addEntity(e)) : sc->remove(e);
        if (!one.on) continue;
        auto& lights = s.engine->getLightManager();
        const auto it = lights.getInstance(e);
        const math::mat4 place = doc_place(d, one.thing, now);
        lights.setPosition(it, math::float3((place * math::double4{0, 0, 0, 1}).xyz));
        lights.setDirection(it, normalize(math::float3((place * math::double4{0, 0, -1, 0}).xyz)));
        const float strong = (float)doc_number(d, one.thing, "intensity", now, 1);
        if (lights.getType(it) == LightManager::Type::SUN) lights.setIntensity(it, 60000.f * strong);   // (lux, as the room's own light)
        else lights.setIntensityCandela(it, 60000.f * 36.f * strong);   // (as bright as that six units off: where the stage's own eye stands)
        if (lights.getType(it) == LightManager::Type::SUN)
            lights.setSunAngularRadius(it, (float)doc_number(d, one.thing, "sun_size", now, 1.09) / 2.f), lights.setSunHaloSize(it, (float)doc_number(d, one.thing, "sun_halo", now, 10));
        if (lights.getType(it) == LightManager::Type::FOCUSED_SPOT) {
            const float cone = (float)(doc_number(d, one.thing, "cone", now, 30) * (double)TAU / 360.0);
            lights.setSpotLightCone(it, 0.8f * cone, cone);
        }
    }
    // A clipped thing is seen only inside the frame of the thing before it: Filament's own scissor on its looks, told
    // the rectangle of the picture that the thing before it covers now (turned or leaning: the upright box around it).
    int before = 0;
    for (int thing : lie) {
        const auto found = d.shown.find(thing);
        if (found == d.shown.end()) continue;
        Doc::Shown& t = found->second;
        double box[4] = {0, 0, 0, 0};
        const bool clipped = t.in && before && doc_number(d, thing, "clip", 0, 0) > 0.5 && doc_covers(d, before, now, box);
        // (clipped by a flat thing ThorVG draws, it is cut by that thing's own picture: the kit's matte, After Effects'
        //  alpha matte; by anything else, by the upright box of what that covers)
        const Doc::Shown* const a = clipped ? &d.shown[before] : nullptr;
        const bool shaped = a && a->drawn && (t.drawn || t.film || t.look);   // (a solid too, by its own look: the kit's glass, or its plain look made to be cut)
        Doc::Cut& cut = d.cuts[thing];
        MaterialInstance* const own = t.film ? t.film->look : t.drawn ? t.drawn->look : t.look;
        const std::string how = shaped ? doc_word(d, thing, "clip_matte") + " " + std::to_string(doc_number(d, thing, "clip_invert", 0, 0)) + " " + std::to_string(doc_number(d, thing, "clip_feather", 0, 0)) : std::string();   // (said only for one that is cut)
        if (shaped && (cut.from != before || cut.by != a->drawn || cut.how != how)) {   // (its matte takes that picture's shape, at that thing's place, cut as it is said)
            if (cut.look) {   // (what cut its look before is let go of first)
                std::erase_if(s.cuts, [&](const Stage::Cut& c) { return c.look == cut.look; });
                cut.look->setParameter("matteUse", float4{0.f});
            }
            if (cut.matte >= 0) {
                Scene* sc = s.mattes[(size_t)cut.matte].scene;
                doc_unshape(s, [sc](const Stage::Made& made) { return sc->hasEntity(made.thing); });
            }
            cut.matte = matte(s, a->drawn->sheet, {mat4f(doc_place(d, before, now))}, a->drawn->texture ? a->drawn->texture : white_point(s), doc_word(d, thing, "clip_matte") == "luma", 1.f, cut.matte);
            cut.from = before, cut.by = a->drawn, cut.look = nullptr, cut.how = how;
        }
        if (shaped && cut.look != own && own && cut.matte >= 0) {
            matted(s, own, cut.matte, doc_number(d, thing, "clip_invert", 0, 0) < 0.5, (float)doc_number(d, thing, "clip_feather", 0, 0)), cut.look = own;
            if (t.drawn && t.drawn->edgeLook) matted(s, t.drawn->edgeLook, cut.matte, doc_number(d, thing, "clip_invert", 0, 0) < 0.5, (float)doc_number(d, thing, "clip_feather", 0, 0));   // (its edge too)
        }   // (its look, or its look made again, is cut by it)
        if (!shaped && cut.look) {   // (no longer cut so: the matte is let go of)
            std::erase_if(s.cuts, [&](const Stage::Cut& c) { return c.look == cut.look; });
            if (cut.look == own) cut.look->setParameter("matteUse", float4{0.f});
            cut.look = nullptr, cut.from = 0, cut.by = nullptr;
            Scene* sc = s.mattes[(size_t)cut.matte].scene;
            doc_unshape(s, [sc](const Stage::Made& made) { return sc->hasEntity(made.thing); });
        }
        if (shaped) { if (t.clipped) doc_looks(s, t, [](MaterialInstance* look) { look->unsetScissor(); }), t.clipped = false; }
        else if (clipped || t.clipped) {
            const uint32_t left = (uint32_t)std::clamp(box[0], 0.0, (double)s.w), bottom = (uint32_t)std::clamp((double)s.h - (box[1] + box[3]), 0.0, (double)s.h);
            doc_looks(s, t, [&](MaterialInstance* look) { if (clipped) look->setScissor(left, bottom, (uint32_t)std::lround(std::max(box[2], 0.0)), (uint32_t)std::lround(std::max(box[3], 0.0))); else look->unsetScissor(); });
            t.clipped = clipped;
        }
        if (t.in) before = thing;
    }
    // An After Effects picture's 3D layers: each at the moment its picture is at, where its pose puts it over that
    // picture's sheet as the stage has it now (the composition's corner at the sheet's top left, a pixel a share of its height).
    for (auto& [thing, t] : d.shown) {
        if (t.deep.empty() || !t.drawn || !t.motion) continue;
        const double since = now - doc_number(d, thing, "start", 0, 0);
        const float sec = t.drawn->seconds > 0.f ? std::fmod((float)std::max(since, 0.0), t.drawn->seconds) : 0.f;
        const float frame = t.drawn->fromFrame + std::min(sec / std::max(t.drawn->seconds, 1e-6f) * t.drawn->frames, std::max(t.drawn->frames - 1.f, 0.f));
        motion_at(*t.motion, frame);
        mat4f sheetAt;
        bool found = false;
        auto& places = s.engine->getTransformManager();
        for (const Stage::Made& made : s.made)
            if (made.of == t.drawn->sheet && !made.matte && made.nth == 0) { sheetAt = places.getWorldTransform(places.getInstance(made.thing)); found = true; break; }
        const float px = 1.f / t.motion->h;
        const float3 corner{-0.5f * t.motion->w * px, 0.5f, 0.f}, right{px, 0.f, 0.f}, down{0.f, -px, 0.f};
        for (const auto& deep : t.deep) {
            layer_at(s, deep.sheet, frame);
            tvg::Matrix flat;
            float seen = 0.f;
            const bool there = t.in && found && motion_flat(*t.motion, deep.id, &flat, &seen) && seen > 0.f;
            const mat4f at = motion_built(*t.motion, deep.id);
            move(s, deep.sheet->sheet, [&](int, const mat4f&) {
                return there ? sheetAt * motion_in_scene(at, corner, right, down) * mat4f::translation(float3{(deep.box.x + 0.5f * deep.box.z) * px, (deep.box.y + 0.5f * deep.box.w) * px, 0.f}) * mat4f::scaling(float3{1, -1, 1})
                             : mat4f::scaling(0.f);
            });
        }
    }
}

// The stage made to match the document: a thing the document has and the stage has not is made (by the kit's own
// call for its kind), one the stage has and the document has not is taken off, one whose making is told otherwise now
// is made again. false: a thing could not be made (d.trouble says which, and why).
// All that a thing's making goes by: one changed, the thing is made again (doc_match), carried by a hand too (doc_carry).
static const char* const doc_making = "'kind','file','shape','path','blend','fx','keyed','space','begin','duration','speed','text','fill','fill_rule','wide','points','inner','round','bloat','zigzag','offset','warp','clip','stroke','stroke_width','align','copies','letter_spacing','line_spacing','italic','cap','join','heads','head_size','head_shape','dash','dash_offset','metallic','roughness','reflectance','clear_coat','clear_coat_roughness','falloff','shadows','shadow_softness','contact_shadows','sheen','anisotropy','shading','thickness','light_group','script','copy_fill','gradient','gradient_angle','gradient_scale','gradient_spread','wrap','wrap_at','split','ior','dispersion','swim','dark','vivid','marker','mosh','mosh_with','mosh_at'";
// A flat thing's effects (ThorVG's, doc_fx): told again at each moment (fx_at), never a reason to make it again. What
// they ask of its making is only which are on and how much room round it they reach (doc_fx_room).
static const char* const doc_fxing = "'blur','shadow','shadow_distance','shadow_angle','shadow_blur','shadow_opacity','glow','glow_blur','glow_opacity','tint','tint_black','tint_white','tritone','tritone_shadow','tritone_midtone','tritone_highlight'";
// How far a thing's effects reach round it at their most (what is carried too), in the document's units, rounded up to
// a power of two: the room its picture is made with, so a number changed within it is drawn again without making it
// again (as a texture atlas keeps power-of-two room). 0: none reach.
static double doc_fx_room(Doc& d, int thing) {
    const auto most = [&](const char* name, double otherwise) { return std::max(doc_most(d, thing, name, otherwise), doc_number(d, thing, name, 0, otherwise)); };
    double r = 3.0 * most("blur", 0);
    if (!doc_word(d, thing, "shadow").empty()) r = std::max(r, most("shadow_distance", 10) + 3.0 * most("shadow_blur", 10));
    return r <= 0.0 ? 0.0 : std::exp2(std::ceil(std::log2(std::max(r, 1.0))));
}
