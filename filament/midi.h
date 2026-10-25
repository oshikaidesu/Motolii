// midi.h: a MIDI file's notes, as GStreamer's own midiparse reads it (a buffer an event, its time the event's): each
// note's coming on and going off, its channel, pitch and velocity. Made keys as om_midi makes After Effects keyframes
// of them (github.com/otomad/om_midi, Core.ts; GPL: its rules, not its code).
#pragma once
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include "gst_read.h"
#include "script.h"
#include <fstream>
#include <iterator>
#include <cmath>
#include <map>
#include <mutex>
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

// A note: when it comes on and goes off (an on with none: as long as the next), its pitch and velocity.
struct MidiNote { double on = 0, off = 0; int pitch = 0, velocity = 0; };

// A MIDI file's notes, read once (the file as it is), in the order they come on.
static const std::vector<MidiNote>& midi_of(const std::string& file) {
    static std::map<std::string, std::vector<MidiNote>> read;
    static std::mutex one;
    std::lock_guard<std::mutex> hold(one);
    auto found = read.find(file);
    if (found != read.end()) return found->second;
    std::vector<MidiNote> notes;
    std::map<std::pair<int, int>, size_t> sounding;   // (channel and pitch: the note of them that is on)
    for (const MidiEvent& e : midi_events(file.c_str())) {
        const auto key = std::make_pair(e.channel, e.pitch);
        if (auto was = sounding.find(key); was != sounding.end()) notes[was->second].off = e.at, sounding.erase(was);
        if (e.on) sounding[key] = notes.size(), notes.push_back({e.at, -1, e.pitch, e.velocity});
    }
    for (MidiNote& n : notes) if (n.off < n.on) n.off = n.on + 0.1;
    return read.emplace(file, std::move(notes)).first->second;
}

// Its notes as a thing's copies, as ALMaMPlayer shows them with a picture for a note (github.com/almam72/ALMAMPlayer,
// GPL-3: its numbers, not its code; a 1920 x 1080 screen, `unit` a pixel of it in the stage's lengths). Copy 2i is note
// i: going left 300 a second (speed), struck in the middle; its pitch up from the lowest, the whole range 688 high
// (vertical_offset + top_margin), the middle of it the screen's; struck it grows to 1.03 over 0.2 s, back over 0.27 s
// once it goes off (note.gd's tweens). Copy 2i + 1 is its burst where it was struck (note_effect.tscn): 0.8, 1.16 at
// 0.027 s, 1 at 0.096 s; from 0.025 s before it goes off, smaller to none by 0.167 s and gone between 0.083 and 0.146 s.
static std::vector<ScriptCopy> midi_copies(const std::vector<MidiNote>& notes, int n, double t, double unit) {
    std::vector<ScriptCopy> all((size_t)std::max(n, 0));
    for (ScriptCopy& c : all) c.opacity = 0.0;   // (more copies than notes: not seen)
    if (notes.empty()) return all;
    int low = 127, high = 0;
    for (const MidiNote& m : notes) low = std::min(low, m.pitch), high = std::max(high, m.pitch);
    const double speed = 300.0, spacing = 688.0 / std::max(1, high - low);
    const auto ease = [](double a, double b, double x) { x = std::clamp(x, 0.0, 1.0); return a + (b - a) * (0.5 - 0.5 * std::cos(x * 3.14159265358979)); };   // (Tween.TRANS_SINE, EASE_IN_OUT)
    const auto keys = [](const std::vector<std::pair<double, double>>& k, double x) {   // (Godot's animation track: linear between its keys)
        if (x <= k.front().first) return k.front().second;
        for (size_t i = 1; i < k.size(); ++i)
            if (x <= k[i].first) return k[i - 1].second + (k[i].second - k[i - 1].second) * (x - k[i - 1].first) / (k[i].first - k[i - 1].first);
        return k.back().second;
    };
    for (int i = 0; i < (int)notes.size() && 2 * i < n; ++i) {
        const MidiNote& m = notes[(size_t)i];
        const double y = (m.pitch - low - 0.5 * (high - low)) * spacing * unit;
        ScriptCopy& note = all[(size_t)(2 * i)];
        note.x = (m.on - t) * speed * unit, note.y = y, note.opacity = 1.0;
        note.size = t < m.on ? 1.0 : t < m.off ? ease(1.0, 1.03, (t - m.on) / 0.2) : ease(1.03, 1.0, (t - m.off) / 0.27);
        if (2 * i + 1 >= n || t < m.on) continue;
        ScriptCopy& burst = all[(size_t)(2 * i + 1)];
        const double end = m.off - m.on > 0.09 ? m.off - 0.025 : m.on + 0.065;   // (note_effect.gd's Timer)
        burst.y = y;
        if (t < end) burst.size = keys({{0, 0.8}, {0.0268592, 1.16}, {0.0956246, 1.0}}, t - m.on), burst.opacity = 1.0;
        else burst.size = keys({{0, 1.0}, {0.166667, 0.0}}, t - end), burst.opacity = keys({{0.0833333, 1.0}, {0.145833, 0.0}}, t - end);
    }
    return all;
}
