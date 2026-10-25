// midi.h: a MIDI file's notes, as GStreamer's own midiparse reads it (a buffer an event, its time the event's): each
// note's moment on and off, its channel, pitch and velocity. Made keys as om_midi makes After Effects keyframes of
// them (github.com/otomad/om_midi, Core.ts: a note's velocity keyed where it comes on, HOLD till the next key).
#pragma once
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include <cmath>
#include <map>
#include <mutex>
#include <string>
#include <utility>
#include <vector>
#include "script.h"

struct MidiNote { double on = 0, off = -1; int channel = 0, pitch = 0, velocity = 0; };

static std::vector<MidiNote> midi_notes(const char* file) {
    static const bool started = (gst_init(nullptr, nullptr), true);
    (void)started;
    std::vector<MidiNote> notes;
    gchar* path = g_strescape(file, nullptr);
    const std::string said = std::string("filesrc location=\"") + path + "\" ! midiparse ! appsink name=out sync=false";
    g_free(path);
    GstElement* reads = gst_parse_launch(said.c_str(), nullptr);
    if (!reads) return notes;
    GstElement* out = gst_bin_get_by_name(GST_BIN(reads), "out");
    gst_element_set_state(reads, GST_STATE_PLAYING);
    std::map<std::pair<int, int>, size_t> sounding;   // (channel and pitch: the note of them that is on)
    while (GstSample* got = gst_app_sink_pull_sample(GST_APP_SINK(out))) {
        GstBuffer* b = gst_sample_get_buffer(got);
        GstMapInfo map;
        if (b && GST_BUFFER_PTS_IS_VALID(b) && gst_buffer_map(b, &map, GST_MAP_READ)) {
            const double at = (double)GST_BUFFER_PTS(b) / (double)GST_SECOND;
            if (map.size >= 3) {
                const int status = map.data[0] & 0xf0, channel = map.data[0] & 0x0f, pitch = map.data[1], velocity = map.data[2];
                const auto key = std::make_pair(channel, pitch);
                if ((status == 0x80 || (status == 0x90 && velocity == 0)) && sounding.count(key)) notes[sounding[key]].off = at, sounding.erase(key);
                else if (status == 0x90) {
                    if (sounding.count(key)) notes[sounding[key]].off = at;
                    sounding[key] = notes.size();
                    notes.push_back({at, -1, channel, pitch, velocity});
                }
            }
            gst_buffer_unmap(b, &map);
        }
        gst_sample_unref(got);
    }
    gst_element_set_state(reads, GST_STATE_NULL);
    gst_object_unref(out), gst_object_unref(reads);
    return notes;
}

// A MIDI file's notes read once (the file as it is).
static const std::vector<MidiNote>& midi_of(const std::string& file) {
    static std::map<std::string, std::vector<MidiNote>> read;
    static std::mutex one;
    std::lock_guard<std::mutex> hold(one);
    auto found = read.find(file);
    if (found == read.end()) found = read.emplace(file, midi_notes(file.c_str())).first;
    return found->second;
}
// Its notes as a thing's copies, as ALMaMPlayer shows them (github.com/almam72/ALMAMPlayer: a note a sprite of the shape
// it is given, an effect where it plays): copy i is note i, across by its moment (now in the middle, the notes going by
// to the left), up by its pitch, large while it sounds, paler once it has. Lengths in points of the document (`unit`).
static std::vector<ScriptCopy> midi_copies(const std::vector<MidiNote>& notes, int n, double t, double unit) {
    std::vector<ScriptCopy> all((size_t)std::max(n, 0));
    if (notes.empty()) return all;
    double low = 127, high = 0;
    for (const MidiNote& m : notes) low = std::min(low, (double)m.pitch), high = std::max(high, (double)m.pitch);
    const double middle = 0.5 * (low + high), across = 360.0, up = 22.0;   // (points a second, points a semitone)
    for (int i = 0; i < n && i < (int)notes.size(); ++i) {
        const MidiNote& m = notes[(size_t)i];
        const double off = m.off > m.on ? m.off : m.on + 0.2;
        ScriptCopy& c = all[(size_t)i];
        c.x = (m.on - t) * across * unit, c.y = (m.pitch - middle) * up * unit;
        const bool sounding = t >= m.on && t < off;
        const double since = t - m.on;
        c.size = sounding ? 1.0 + 0.6 * (m.velocity / 127.0) * std::exp(-6.0 * since) + 0.25 : 0.8;
        c.opacity = t >= off ? 0.35 : 1.0;
    }
    for (int i = (int)notes.size(); i < n; ++i) all[(size_t)i].opacity = 0.0;   // (more copies than notes: not seen)
    return all;
}
