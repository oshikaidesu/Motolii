// midi.h: a MIDI file's notes, as GStreamer's own midiparse reads it (a buffer an event, its time the event's): each
// note's coming on and going off, its channel, pitch and velocity. Made keys as om_midi makes After Effects keyframes
// of them (github.com/otomad/om_midi, Core.ts; GPL: its rules, not its code).
#pragma once
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include "gst_read.h"
#include <fstream>
#include <iterator>
#include <map>
#include <string>
#include <utility>
#include <vector>

// A note's coming on or going off, in the file's order (status 0x90 on, 0x80 off; an on of velocity 0 is an off).
struct MidiEvent { double at = 0; bool on = false; int channel = 0, pitch = 0, velocity = 0; };

static std::vector<MidiEvent> midi_events(const char* file) {
    static const bool started = (gst_init(nullptr, nullptr), true);
    (void)started;
    std::vector<MidiEvent> events;
    bool pulling = true;
    GstElement* reads = gst_parse_launch("filesrc name=in ! midiparse ! appsink name=out sync=false", nullptr);
    if (!reads) return events;
    gst_read_from(reads, file);
    GstElement* out = gst_bin_get_by_name(GST_BIN(reads), "out");
    if (gst_element_set_state(reads, GST_STATE_PLAYING) == GST_STATE_CHANGE_FAILURE) pulling = false;   // (no such file: no end would come)
    while (GstSample* got = pulling ? gst_app_sink_pull_sample(GST_APP_SINK(out)) : nullptr) {
        GstBuffer* b = gst_sample_get_buffer(got);
        GstMapInfo map;
        if (b && GST_BUFFER_PTS_IS_VALID(b) && gst_buffer_map(b, &map, GST_MAP_READ)) {
            const int status = map.size >= 3 ? map.data[0] & 0xf0 : 0;
            if (status == 0x80 || status == 0x90)
                events.push_back({(double)GST_BUFFER_PTS(b) / (double)GST_SECOND, status == 0x90 && map.data[2] > 0, map.data[0] & 0x0f, map.data[1], map.data[2]});
            gst_buffer_unmap(b, &map);
        }
        gst_sample_unref(got);
    }
    gst_element_set_state(reads, GST_STATE_NULL);
    gst_object_unref(out), gst_object_unref(reads);
    return events;
}

// The keys of one value as om_midi keys its null object's sliders (Core.ts, its chords not keyed apart: a note at the
// moment of the one before is skipped, but for an on just after an off), each held: velocity (0 to 1: where a note
// comes on, and its letting go 0.5 ms before it goes off), pitch (the note's number), count (1, 2, 3 ... a note),
// on (1 while one sounds, 0 where it goes off), bool (count's odd or even: 1, 0, 1 ...). At 0: velocity, on and count
// 0. Empty: a kind it does not know, or no note of the channel (-1: any).
static std::map<double, double> midi_keys(const std::vector<MidiEvent>& events, const std::string& kind, int channel) {
    std::map<double, double> keys;
    if (kind != "velocity" && kind != "pitch" && kind != "count" && kind != "on" && kind != "bool") return keys;
    const double min_interval = 5e-4;   // (om_midi's MIN_INTERVAL)
    int count = 0;
    double last = -1;
    bool last_off = true;
    for (const MidiEvent& e : events) {
        if (channel >= 0 && e.channel != channel) continue;
        if (e.at <= last && !(last_off && e.on)) continue;
        if (e.on) {
            ++count;
            const double is = kind == "velocity" ? e.velocity / 127.0 : kind == "pitch" ? e.pitch : kind == "count" ? count : kind == "on" ? 1 : count % 2;
            keys[e.at] = is;
        } else if (kind == "velocity") keys[std::max(0.0, e.at - min_interval)] = e.velocity / 127.0;
        else if (kind == "on") keys[e.at] = 0;
        last = e.at, last_off = !e.on;
    }
    if (keys.empty()) return keys;
    if ((kind == "velocity" || kind == "on" || kind == "count") && !keys.count(0.0)) keys[0.0] = 0;
    return keys;
}

// The tempo a MIDI file says first (Standard MIDI File 1.0: the meta event FF 51 03 and microseconds a quarter note),
// as BPM; one that says none is at 120 (the same spec's default; om_midi's Midi.ts DEFAULT_BPM).
static double midi_bpm(const char* file) {
    std::ifstream in(file, std::ios::binary);
    const std::string b((std::istreambuf_iterator<char>(in)), std::istreambuf_iterator<char>());
    const size_t at = b.find("\xff\x51\x03");
    if (at == std::string::npos || at + 6 > b.size()) return 120.0;
    const double us = (double)(((unsigned char)b[at + 3] << 16) | ((unsigned char)b[at + 4] << 8) | (unsigned char)b[at + 5]);
    return us > 0 ? 60e6 / us : 120.0;
}
