// sound.h: how loud a file's sound is through its length, as GStreamer's own level element measures it (decoded as
// fast as it goes, not played), and the moments it strikes: where it grows louder by so much at once (a drum, a clap).
// After Effects' Convert Audio to Keyframes reads the same loudness; the strikes are where its curve jumps.
#pragma once
#include <gst/gst.h>

#include <algorithm>
#include <string>
#include <vector>

// The loudest channel's loudness (dB, 0 at full) every 1/rate seconds of the file's own time; empty: no sound read.
static std::vector<float> sound_loudness(const char* file, double rate) {
    static const bool started = (gst_init(nullptr, nullptr), true);
    (void)started;
    std::vector<float> db;
    gchar* path = g_strescape(file, nullptr);
    const std::string said = std::string("filesrc location=\"") + path + "\" ! decodebin ! audioconvert ! audio/x-raw ! level name=level post-messages=true interval=" +
                             std::to_string((guint64)(1e9 / rate)) + " ! fakesink sync=false";
    g_free(path);
    GError* wrong = nullptr;
    GstElement* reads = gst_parse_launch(said.c_str(), &wrong);
    if (wrong) g_error_free(wrong);
    if (!reads) return db;
    GstBus* bus = gst_element_get_bus(reads);
    gst_element_set_state(reads, GST_STATE_PLAYING);
    for (bool going = true; going;) {
        GstMessage* heard = gst_bus_timed_pop_filtered(bus, 10 * GST_SECOND, (GstMessageType)(GST_MESSAGE_ELEMENT | GST_MESSAGE_EOS | GST_MESSAGE_ERROR));
        if (!heard) break;
        if (GST_MESSAGE_TYPE(heard) != GST_MESSAGE_ELEMENT) going = false;
        else if (const GstStructure* s = gst_message_get_structure(heard); s && gst_structure_has_name(s, "level")) {
            const GValue* rms = gst_structure_get_value(s, "rms");
            float loudest = -100.f;
            G_GNUC_BEGIN_IGNORE_DEPRECATIONS
            if (rms && G_VALUE_HOLDS(rms, G_TYPE_VALUE_ARRAY)) {
                const GValueArray* each = (const GValueArray*)g_value_get_boxed(rms);
                for (guint c = 0; c < each->n_values; ++c) loudest = std::max(loudest, (float)g_value_get_double(g_value_array_get_nth((GValueArray*)each, c)));
            }
            G_GNUC_END_IGNORE_DEPRECATIONS
            db.push_back(std::max(loudest, -100.f));
        }
        gst_message_unref(heard);
    }
    gst_element_set_state(reads, GST_STATE_NULL);
    gst_object_unref(bus);
    gst_object_unref(reads);
    return db;
}

// Where it strikes, in the file's own seconds: it grows `rise` dB louder within 30 ms, and is loud enough to be heard
// (above -40 dB); one strike, then none for 120 ms.
static std::vector<double> sound_strikes(const std::vector<float>& db, double rate, double rise) {
    std::vector<double> at;
    const int near = std::max(1, (int)std::lround(0.03 * rate)), apart = std::max(1, (int)std::lround(0.12 * rate));
    int last = -apart;
    for (int i = 0; i < (int)db.size(); ++i) {
        if (i - last < apart || db[(size_t)i] < -40.f) continue;
        float low = i < near ? -100.f : db[(size_t)i];   // (before the file begins: silence)
        for (int j = std::max(0, i - near); j < i; ++j) low = std::min(low, db[(size_t)j]);
        if (db[(size_t)i] - low >= rise) at.push_back(i / rate), last = i;
    }
    return at;
}
