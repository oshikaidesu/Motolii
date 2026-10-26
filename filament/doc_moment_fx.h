// doc_moment_fx.h: part of doc.h (a thing with effects of its own, told a moment: thing_fx.h). Read only through doc.h.
// Each drawn alone into a picture of the box it is seen in, its effects run on it, before the stage's own view.
// One whose fx names a solid's shape (solids/*.js, Alight Motion's Box, Torus...): drawn alone square to its own face,
// its effects run on that, and the picture laid on the solid's faces where it lies; its sheet not seen.
static void doc_thing_solid(Doc& d, int thing, ThingFx& e, const ScreenNumbers& numbers, const std::function<DrawnFx()>& thorFx) {
    Stage& s = *d.stage;
    math::mat4 place;
    float2 half{0.5f};
    {
        std::lock_guard<std::recursive_timed_mutex> hold(d.lock);   // (asked while the picture is drawn)
        place = doc_place(d, thing, d.stage->now), doc_half(d, thing, &half);
    }
    thing_fx_face(s, e, place, half);
    thing_effects_draw(s, e.st, e.view, numbers, thorFx);
    if (solid_reads(e.solidFile)) thing_fx_cover(s, e, (int)(e.knobs.count("precision") ? e.knobs["precision"] : 160)), thing_fx_solid(s, e, true);
    e.look->setParameter("place", mat4f::scaling(float3{0.f}));   // (its sheet: nowhere)
    if (!e.solid || !e.st.seen) return;
    move(s, e.solid, [&](int, const mat4f&) { return mat4f(place); });
    const auto wrap = e.solidRepeat ? TextureSampler::WrapMode::REPEAT : TextureSampler::WrapMode::CLAMP_TO_EDGE;
    e.solidLook->setParameter("image", e.st.seen, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR, wrap));
}
static void doc_things_fx(Doc& d) {
    Stage& s = *d.stage;
    s.before.clear();
    for (auto& [thing, e] : d.thingFx) {   // (each drawn alone into a picture, its effects run on it, before the stage's own view: thing_fx.h)
        thing_effects(s, e.st, doc_fx_join(doc_fx_list(d, thing)));
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
                if (doc_seen_box(d, thing, d.stage->now, seen)) {   // (where it is seen, as much of it as is on the stage: its picture, After Effects' layer;
                    //  with room around it for what its effects draw outside it: a glow, a shadow, streaks)
                    const double m = doc_number(d, thing, "fx_margin", d.stage->now, 0) * std::max(seen[2] - seen[0], seen[3] - seen[1]);
                    box[0] = std::max(0.0, seen[0] - m), box[1] = std::max(0.0, seen[1] - m), box[2] = std::min((double)s.w, seen[2] + m), box[3] = std::min((double)s.h, seen[3] + m);
                }
                if (box[2] - box[0] < 1 || box[3] - box[1] < 1) box[0] = box[1] = 0, box[2] = s.w, box[3] = s.h;   // (none of it on the stage: the whole)
            }
            if (!e.solidFile.empty()) return doc_thing_solid(d, thing, e, numbers, thorFx);
            thing_fx_aim(s, e, box);
            thing_effects_draw(s, e.st, e.view, numbers, thorFx);
            if (e.st.seen && e.look) e.look->setParameter("image", e.st.seen, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
            if (e.look) {   // (its glow over what its effects made: the stage's bloom, as a thing without effects glows (doc_moment))
                std::lock_guard<std::recursive_timed_mutex> hold(d.lock);
                e.look->setParameter("glowing", (float)doc_number(d, thing, "emissive", d.stage->now, 0) + (doc_word(d, thing, "glow").empty() ? 0.f : 0.35f * (float)doc_number(d, thing, "glow_opacity", d.stage->now, 1)));
            }
        });
    }
}
