// doc_moment_fx.h: part of doc.h (a thing with effects of its own, told a moment: thing_fx.h). Read only through doc.h.
// Each drawn alone into a picture of the box it is seen in, its effects run on it, before the stage's own view.
// One whose fx names a solid's shape (solids/*.js, Alight Motion's Box, Torus...): drawn alone square to its own face,
// its effects run on that, and the picture laid on the solid's faces where it lies; its sheet not seen.
static void doc_thing_solid_aim(Doc& d, int thing, ThingFx& e) {   // (before the frame)
    Stage& s = *d.stage;
    math::mat4 place;
    float2 half{0.5f};
    {
        std::lock_guard<std::recursive_timed_mutex> hold(d.lock);   // (asked while the picture is drawn)
        place = doc_place(d, thing, d.stage->now), doc_half(d, thing, &half);
    }
    thing_fx_face(s, e, place, half);
    thing_fx_pictures(s, e.st, e.view);
    e.look->setParameter("place", mat4f::scaling(float3{0.f}));   // (its sheet: nowhere)
    if (!e.solid || !e.solidLook) return;
    move(s, e.solid, [&](int, const mat4f&) { return mat4f(place); });
    const auto wrap = e.solidRepeat ? TextureSampler::WrapMode::REPEAT : TextureSampler::WrapMode::CLAMP_TO_EDGE;
    e.solidLook->setParameter("image", thing_fx_seen(e.st), TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, wrap));
}
static void doc_thing_solid(Doc& d, int thing, ThingFx& e, const ScreenNumbers& numbers, const std::function<DrawnFx()>& thorFx) {   // (in the frame)
    Stage& s = *d.stage;
    thing_effects_draw(s, e.st, e.view, numbers, thorFx);
    if (solid_reads(e.solidFile)) thing_fx_cover(s, e, (int)(e.knobs.count("precision") ? e.knobs["precision"] : 160)), thing_fx_solid(s, e, true);
}
static void doc_things_fx(Doc& d) {
    Stage& s = *d.stage;
    s.before.clear(), s.aim.clear();
    for (auto& [thing, e] : d.thingFx) {   // (each drawn alone into a picture, its effects run on it, before the stage's own view: thing_fx.h)
        thing_effects(s, e.st, doc_fx_join(doc_fx_steps(d, thing)));
        e.solidFile.clear();
        for (const std::string& w : doc_fx_list(d, thing)) if (solid_is(w)) e.solidFile = w;   // (a solid's shape among them: its picture laid on that, thing_fx.h; the last named)
        if (const DocScript* sc = e.solidFile.empty() ? nullptr : doc_script_of(d, e.solidFile)) {   // (its knobs as the document says them now, by their own names; the solid made with them, before the frame)
            e.knobs.clear(), e.words.clear();
            for (size_t i = 0; i < sc->knobs.size(); ++i) {
                const ScriptKnob& k = sc->knobs[i];
                const std::string w = doc_word(d, thing, sc->names[i].c_str());
                if (k.kind != "select") e.knobs[k.name] = doc_number(d, thing, sc->names[i].c_str(), d.stage->now, k.usual);
                else e.words[k.name] = w.empty() ? k.words.substr(0, k.words.find(' ')) : w;
            }
            float2 half{0.5f};
            doc_half(d, thing, &half);
            e.size = 2.f * half;
            thing_fx_solid(s, e, false);
        } else if (e.solid) take_off(s, e.solid), delete e.solid, e.solid = nullptr, e.solidSaid.clear();   // (no longer: its sheet again)
        e.st.sounds = [&d, thing = thing](const std::string& file, const std::string& name, bool fft, int wide) { return doc_sound_now(d, thing, file, name, fft, wide); };
        const ScreenNumbers numbers = [&d, thing = thing, &e = e](const std::string& file, const std::string& name, double usual) {
            std::lock_guard<std::recursive_timed_mutex> hold(d.lock);   // (asked while the picture is drawn)
            if (name == "duration") {   // (AbsentMotion's effects.cpp: the layer's length in seconds. None said: the document's, as a still that does not end)
                const double lasts = doc_number(d, thing, "duration", 0, -1);
                return lasts > 0 ? lasts : doc_seconds(d);
            }
            if (name.rfind("backdropBox_", 0) == 0) {   // (the box its picture is, on the stage's: left, top, width, height, each a share of it: what is behind it read there, effects/from_absent.py)
                const Stage& s = *d.stage;
                const size_t i = std::string("xyzw").find(name.back());
                return i == 0 ? e.box[0] / s.w : i == 1 ? e.box[1] / s.h : i == 2 ? (e.box[2] - e.box[0]) / s.w : (e.box[3] - e.box[1]) / s.h;
            }
            if (name.rfind("layerRect_", 0) == 0) {   // (its own rectangle on its picture, in the picture's pixels: AbsentMotion's layer, where its corner is and how large: effects/from_absent.py)
                double frame[8];
                if (!doc_frame(d, thing, d.stage->now, frame)) return 0.0;
                const Stage& s = *d.stage;
                const double ppp = (double)s.view->getViewport().width / std::max(1, s.w);
                const size_t i = std::string("xyzw").find(name.back());
                return ppp * (i == 0 ? frame[0] - frame[2] / 2 - e.box[0] : i == 1 ? frame[1] - frame[3] / 2 - e.box[1] : i == 2 ? frame[2] : frame[3]);
            }
            if (name == "origin_x" || name == "origin_y" || name == "turn" || name == "extent") {   // (where the thing is on the picture now, how turned, how large: an effect's pattern stuck to it)
                double frame[8];
                if (!doc_frame(d, thing, d.stage->now, frame)) return usual;
                return name == "origin_x" ? frame[0] - e.box[0] : name == "origin_y" ? frame[1] - e.box[1] : name == "turn" ? frame[4] : std::max(frame[2], frame[3]);   // (on its own picture: the box it is seen in)
            }
            if ((name == "blurLength" || name == "angle") && file.ends_with("am_directionalblur.wgsl") && doc_word(d, thing, "fx").find("am_directionalblur.wgsl") == std::string::npos) {
                // (the step its Motion Blur is: as far and that way as its middle goes while its shutter is open, in its picture's pixels, as AbsentMotion's quick smear)
                double open = 0.0, was[8], is[8];
                bool moves = false;
                const double now = d.stage->now;
                if (!doc_shutter(d, thing, now, &open, &moves) || !moves || !doc_frame(d, thing, now - 0.5 * open, was) || !doc_frame(d, thing, now + 0.5 * open, is)) return 0.0;
                const double ppp = (double)d.stage->view->getViewport().width / std::max(1, d.stage->w), dx = is[0] - was[0], dy = is[1] - was[1];
                return name == "blurLength" ? ppp * std::hypot(dx, dy) : std::atan2(-dy, -dx) * 180.0 / M_PI;   // (its blur reads behind it along the way it is told: told back, it trails behind the way it goes)
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
        s.aim.push_back([&s, &d, thing = thing, &e = e] {   // (before the frame: where it is seen, its pictures, what its sheet shows)
            double box[4] = {0, 0, (double)s.w, (double)s.h}, seen[4];
            {
                std::lock_guard<std::recursive_timed_mutex> hold(d.lock);   // (asked while the picture is drawn)
                if (doc_seen_box(d, thing, d.stage->now, seen)) {   // (where it is seen, as much of it as is on the stage: its picture, After Effects' layer;
                    //  with room around it for what its effects draw outside it, sashimono-edit's BAKE_MARGIN: 128 pixels,
                    //  engine/render/script_bake.py; its effects' own reach read from their knobs is not, ISF's not in pixels)
                    const double m = 128.0 * s.w / std::max(1u, s.view->getViewport().width);
                    box[0] = std::max(0.0, seen[0] - m), box[1] = std::max(0.0, seen[1] - m), box[2] = std::min((double)s.w, seen[2] + m), box[3] = std::min((double)s.h, seen[3] + m);
                }
                if (box[2] - box[0] < 1 || box[3] - box[1] < 1) box[0] = box[1] = 0, box[2] = s.w, box[3] = s.h;   // (none of it on the stage: the whole)
            }
            if (!e.solidFile.empty()) return doc_thing_solid_aim(d, thing, e);
            if (e.st.readsBehind) {   // (its sheet on the screen only, not in the picture of what is behind it that its effect reads: as the glass leaves itself out, kit_things.h)
                s.backdropRead = true;
                for (const Stage::Made& made : s.made) if (made.of == e.sheet) s.engine->getRenderableManager().setLayerMask(s.engine->getRenderableManager().getInstance(made.thing), 0xFF, 0x02);
            }
            thing_fx_aim(s, e, box);
            thing_fx_pictures(s, e.st, e.view);
            if (e.look) e.look->setParameter("image", thing_fx_seen(e.st), TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
            if (e.look) {   // (its glow over what its effects made: the stage's bloom, as a thing without effects glows (doc_moment))
                std::lock_guard<std::recursive_timed_mutex> hold(d.lock);
                e.look->setParameter("glowing", (float)doc_number(d, thing, "emissive", d.stage->now, 0) + (doc_word(d, thing, "glow").empty() ? 0.f : 0.35f * (float)doc_number(d, thing, "glow_opacity", d.stage->now, 1)));
            }
        });
        s.before.push_back([&d, thing = thing, &e = e, numbers, thorFx] {   // (in the frame: drawn alone, its effects run)
            if (!e.solidFile.empty()) return doc_thing_solid(d, thing, e, numbers, thorFx);
            thing_effects_draw(*d.stage, e.st, e.view, numbers, thorFx);
        });
    }
}
