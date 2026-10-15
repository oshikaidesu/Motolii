// heard.h: the things' own sound heard while the stage plays (Stage::sounding: what a written film mixes, kit.h): each
// a GStreamer playbin of its file's sound alone, sought to where the stage's moment puts it in its cut and played at its
// speed; paused when the stage stands or the thing is not on; sought again when it strays (its cut gone round, the
// playhead put elsewhere, a clock apart). MOTOLII_MUTE: everything as it is, at no loudness (checks run on a desk).
#pragma once
// (after kit.h: Stage, Stage::Sound)
#include <gst/gst.h>
#include <cmath>
#include <cstdlib>
#include <map>
#include <string>

struct Heard {
    GstElement* bin = nullptr;
    std::string file;
    bool on = false;          // playing now
    double wanted = -1.0;     // where in its file the stage's moment puts it (seconds), as last worked out
};

// Where in its file a thing's sound is at the stage's moment `now` (seconds), as a written film puts it: its cut, sped,
// gone round; false: not heard then (before it comes on, after it ends).
static bool heard_where(const Stage::Sound& a, double now, double* at) {
    const double t = now - a.on;
    if (t < 0.0 || (a.lasts >= 0.0 && t > a.lasts)) return false;
    const double fast = std::max(1e-3, std::fabs(a.speed)), into = (a.into + t) * fast;   // (seconds of the file gone by since the cut's start)
    const double cut = a.to - a.from;
    *at = a.from + (cut > 0.0 ? std::fmod(into, cut) : into);
    return true;
}

static void heard_off(Heard& h) {
    if (!h.bin) return;
    gst_element_set_state(h.bin, GST_STATE_NULL);
    gst_object_unref(h.bin);
    h.bin = nullptr;
}

// Each picture of the live stage: every sounding thing heard where the stage's moment puts it, or not at all.
static void heard(Stage& s, std::map<const void*, Heard>& all, double now, bool playing) {
    static const bool mute = std::getenv("MOTOLII_MUTE") != nullptr;
    for (auto it = all.begin(); it != all.end();)   // (a thing gone, or no longer sounding: its sound goes too)
        if (!s.sounding.count(it->first)) heard_off(it->second), it = all.erase(it);
        else ++it;
    for (const auto& [who, tells] : s.sounding) {
        const Stage::Sound a = tells();
        Heard& h = all[who];
        if (h.bin && h.file != a.file) heard_off(h);
        if (!h.bin) {   // (made once a thing: ready, standing, its sound alone)
            h.bin = gst_element_factory_make("playbin", nullptr);
            if (!h.bin) continue;
            gchar* uri = gst_filename_to_uri(a.file.c_str(), nullptr);
            g_object_set(h.bin, "uri", uri, "flags", 0x2 /* GST_PLAY_FLAG_AUDIO: no picture */, nullptr);
            if (mute) {   // (heard by nobody, as fast as by the speakers: the Mac's sink takes no loudness from playbin)
                GstElement* none = gst_element_factory_make("fakesink", nullptr);
                g_object_set(none, "sync", TRUE, nullptr);
                g_object_set(h.bin, "audio-sink", none, nullptr);
            }
            g_free(uri);
            GstBus* bus = gst_element_get_bus(h.bin);
            gst_bus_set_flushing(bus, TRUE);   // (nobody reads it: what it is told goes as it comes, not kept)
            gst_object_unref(bus);
            gst_element_set_state(h.bin, GST_STATE_PAUSED);
            h.file = a.file, h.on = false;
        }
        double at = 0.0;
        const bool on = playing && heard_where(a, now, &at);
        h.wanted = on ? at : -1.0;
        if (!on) {
            if (h.on) gst_element_set_state(h.bin, GST_STATE_PAUSED), h.on = false;
            continue;
        }
        gint64 is = -1;
        const bool where = gst_element_query_position(h.bin, GST_FORMAT_TIME, &is);
        if (!h.on || !where || std::fabs((double)is / GST_SECOND - at) > 0.1) {   // (begun, or strayed: sought to where it is to be)
            if (!h.on) gst_element_get_state(h.bin, nullptr, nullptr, 200 * GST_MSECOND);   // (ready to be sought: its first time, a moment)
            gst_element_seek(h.bin, std::max(1e-3, std::fabs(a.speed)), GST_FORMAT_TIME, (GstSeekFlags)(GST_SEEK_FLAG_FLUSH | GST_SEEK_FLAG_ACCURATE),
                             GST_SEEK_TYPE_SET, (gint64)(at * GST_SECOND), GST_SEEK_TYPE_NONE, -1);
        }
        if (!h.on) gst_element_set_state(h.bin, GST_STATE_PLAYING), h.on = true;
    }
}
