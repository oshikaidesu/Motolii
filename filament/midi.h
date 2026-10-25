// midi.h: a MIDI file's notes, as GStreamer's own midiparse reads it (a buffer an event, its time the event's): each
// note's moment on and off, its channel, pitch and velocity. Made keys as om_midi makes After Effects keyframes of
// them (github.com/otomad/om_midi, Core.ts: a note's velocity keyed where it comes on, HOLD till the next key).
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

struct MidiNote { double on = 0, off = -1; int channel = 0, pitch = 0, velocity = 0; };

static std::vector<MidiNote> midi_notes(const char* file) {
    static const bool started = (gst_init(nullptr, nullptr), true);
    (void)started;
    std::vector<MidiNote> notes;
    bool pulling = true;
    GstElement* reads = gst_parse_launch("filesrc name=in ! midiparse ! appsink name=out sync=false", nullptr);
    if (!reads) return notes;
    gst_read_from(reads, file);
    GstElement* out = gst_bin_get_by_name(GST_BIN(reads), "out");
    if (gst_element_set_state(reads, GST_STATE_PLAYING) == GST_STATE_CHANGE_FAILURE) pulling = false;   // (no such file: no end would come)
    std::map<std::pair<int, int>, size_t> sounding;   // (channel and pitch: the note of them that is on)
    while (GstSample* got = pulling ? gst_app_sink_pull_sample(GST_APP_SINK(out)) : nullptr) {
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
