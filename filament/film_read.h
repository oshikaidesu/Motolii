// film_read.h: a film's pictures one after another on the CPU, small, for a sketch to read: GStreamer's own parts (appsink,
// as its tutorial "Short-cutting the pipeline" pulls samples). video.h keeps a film on the GPU; a sketch asks for numbers.
#pragma once
#include <gst/app/gstappsink.h>
#include <gst/gst.h>
#include <cstring>
#include <string>
#include <vector>

struct FilmRead {
    GstElement* reader = nullptr;
    GstAppSink* out = nullptr;
    int w = 0, h = 0;
    std::vector<uint8_t> last;   // (the picture last read: at its end it stays)
    ~FilmRead() { if (reader) gst_element_set_state(reader, GST_STATE_NULL), gst_object_unref(reader); if (out) gst_object_unref(out); }
    bool open(const std::string& file, int wide) {
        gst_init(nullptr, nullptr);
        const std::string line = "filesrc name=in ! decodebin ! videoconvert ! videoscale ! videorate ! video/x-raw,format=RGBA,width=" + std::to_string(wide) + ",pixel-aspect-ratio=1/1,framerate=60/1 ! appsink name=out sync=false max-buffers=8";
        reader = gst_parse_launch(line.c_str(), nullptr);
        if (!reader) return false;
        GstElement* in = gst_bin_get_by_name(GST_BIN(reader), "in");
        g_object_set(in, "location", file.c_str(), nullptr);
        gst_object_unref(in);
        out = GST_APP_SINK(gst_bin_get_by_name(GST_BIN(reader), "out"));
        gst_element_set_state(reader, GST_STATE_PLAYING);
        return next();
    }
    bool next() {   // (the picture 1/60 s on)
        GstSample* one = gst_app_sink_try_pull_sample(out, 5 * GST_SECOND);
        if (!one) return !last.empty();
        GstStructure* said = gst_caps_get_structure(gst_sample_get_caps(one), 0);
        gst_structure_get_int(said, "width", &w), gst_structure_get_int(said, "height", &h);
        GstMapInfo map;
        GstBuffer* b = gst_sample_get_buffer(one);
        if (gst_buffer_map(b, &map, GST_MAP_READ)) {
            const size_t row = (map.size / (size_t)h), want = 4 * (size_t)w;
            last.resize(want * (size_t)h);
            for (int y = 0; y < h; ++y) std::memcpy(last.data() + want * y, map.data + row * y, want);
            gst_buffer_unmap(b, &map);
        }
        gst_sample_unref(one);
        return true;
    }
};
