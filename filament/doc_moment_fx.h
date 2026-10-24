// doc_moment_fx.h: part of doc.h (a thing with effects of its own, told a moment: thing_fx.h). Read only through doc.h.
// Each drawn alone into a picture of the box it is seen in, its effects run on it, before the stage's own view.
static void doc_things_fx(Doc& d) {
    Stage& s = *d.stage;
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
}
