// sound.h: how loud a file's sound is through its length, as GStreamer's own level element measures it (decoded as
// fast as it goes, not played), and the moments it strikes: where it grows louder by so much at once (a drum, a clap).
// After Effects' Convert Audio to Keyframes reads the same loudness; the strikes are where its curve jumps.
#pragma once
#include <gst/gst.h>
#include <gst/app/gstappsink.h>

#include <algorithm>
#include <map>
#include <memory>
#include <mutex>
#include <thread>
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

// Its sound as ISF's audio inputs take it (ISF_Spec: an image, a row a channel; "audio" a column a sample of the wave,
// centred on 0.5; "audioFFT" a column a value of the FFT): one channel, all of its length, read once.
struct SoundRead {
    std::vector<float> wave;    // its samples, sound_rate a second (-1 to 1)
    std::vector<float> fft;     // sound_bands a row, sound_rows rows a second (0 silent to 1 loudest)
};
static constexpr int sound_rate = 24000, sound_bands = 256, sound_rows = 60;
// The spectrum, as GStreamer's own spectrum element posts it (its documentation's example: "magnitude", a band each, dB
// above its threshold), the wave as its appsink hands the samples on (GStreamer's basic tutorial 8).
static SoundRead sound_read(const std::string& file) {
    static const bool started = (gst_init(nullptr, nullptr), true);
    (void)started;
    SoundRead r;
    gchar* path = g_strescape(file.c_str(), nullptr);
    const std::string from = std::string("filesrc location=\"") + path + "\" ! decodebin ! audioconvert ! audioresample ! audio/x-raw,format=F32LE,channels=1,rate=" + std::to_string(sound_rate);
    g_free(path);
    const double threshold = -80.0;
    {
        GstElement* reads = gst_parse_launch((from + " ! spectrum name=s bands=" + std::to_string(sound_bands) + " threshold=" + std::to_string((int)threshold) + " interval=" + std::to_string((guint64)(1e9 / sound_rows)) + " post-messages=true message-magnitude=true ! fakesink sync=false").c_str(), nullptr);
        if (!reads) return r;
        GstBus* bus = gst_element_get_bus(reads);
        gst_element_set_state(reads, GST_STATE_PLAYING);
        for (bool going = true; going;) {
            GstMessage* heard = gst_bus_timed_pop_filtered(bus, 10 * GST_SECOND, (GstMessageType)(GST_MESSAGE_ELEMENT | GST_MESSAGE_EOS | GST_MESSAGE_ERROR));
            if (!heard) break;
            if (GST_MESSAGE_TYPE(heard) != GST_MESSAGE_ELEMENT) going = false;
            else if (const GstStructure* st = gst_message_get_structure(heard); st && gst_structure_has_name(st, "spectrum"))
                if (const GValue* mags = gst_structure_get_value(st, "magnitude"))
                    for (int i = 0; i < sound_bands; ++i)
                        r.fft.push_back((float)std::clamp((g_value_get_float(gst_value_list_get_value(mags, (guint)i)) - threshold) / -threshold, 0.0, 1.0));
            gst_message_unref(heard);
        }
        gst_element_set_state(reads, GST_STATE_NULL);
        gst_object_unref(bus), gst_object_unref(reads);
    }
    GstElement* reads = gst_parse_launch((from + " ! appsink name=out sync=false").c_str(), nullptr);
    if (!reads) return r;
    GstElement* out = gst_bin_get_by_name(GST_BIN(reads), "out");
    gst_element_set_state(reads, GST_STATE_PLAYING);
    while (GstSample* got = gst_app_sink_pull_sample(GST_APP_SINK(out))) {
        GstMapInfo map;
        if (GstBuffer* b = gst_sample_get_buffer(got); b && gst_buffer_map(b, &map, GST_MAP_READ)) {
            const float* f = (const float*)map.data;
            r.wave.insert(r.wave.end(), f, f + map.size / sizeof(float));
            gst_buffer_unmap(b, &map);
        }
        gst_sample_unref(got);
    }
    gst_element_set_state(reads, GST_STATE_NULL);
    gst_object_unref(out), gst_object_unref(reads);
    return r;
}
// A file's sound read once, aside (it takes as long as decoding it does): none until it is (silence meanwhile).
static std::shared_ptr<const SoundRead> sound_of(const std::string& file) {
    static std::map<std::string, std::shared_ptr<const SoundRead>> read;
    static std::mutex one;
    std::lock_guard<std::mutex> hold(one);
    auto found = read.find(file);
    if (found != read.end()) return found->second;
    read[file] = nullptr;
    std::thread([file] {
        auto done = std::make_shared<const SoundRead>(sound_read(file));
        std::lock_guard<std::mutex> hold(one);
        read[file] = done;
    }).detach();
    return nullptr;
}
// What an ISF audio input reads at a moment of the file's own (seconds): `wide` values, as its MAX says (none said:
// the samples of one picture's time, the bands as they are). fft: the spectrum's row then; else the wave about then.
static std::vector<float> sound_at(const SoundRead* r, bool fft, int wide, double at) {
    std::vector<float> v((size_t)std::max(wide, 1), fft ? 0.f : 0.5f);
    if (!r || at < 0) return v;
    if (fft) {
        const long row = std::lround(at * sound_rows);
        if (row < 0 || (row + 1) * sound_bands > (long)r->fft.size()) return v;
        for (int i = 0; i < (int)v.size(); ++i) v[(size_t)i] = r->fft[(size_t)(row * sound_bands + (long)i * sound_bands / (long)v.size())];
        return v;
    }
    const long first = std::lround(at * sound_rate);
    for (int i = 0; i < (int)v.size(); ++i) {
        const long k = first + (long)i * (sound_rate / sound_rows) / (long)v.size();
        if (k >= 0 && k < (long)r->wave.size()) v[(size_t)i] = 0.5f + 0.5f * std::clamp(r->wave[(size_t)k], -1.f, 1.f);
    }
    return v;
}
