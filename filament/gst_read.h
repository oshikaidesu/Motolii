// gst_read.h: a file handed to a made pipeline's filesrc (named "in") as its location property, as GStreamer's own
// tutorials hand it (g_object_set), not written into the pipeline's words: a path with letters past ASCII (曲/…) would
// come back escaped (\346…) and not be found.
#pragma once
#include <gst/gst.h>

static void gst_read_from(GstElement* pipeline, const char* file) {
    if (GstElement* in = gst_bin_get_by_name(GST_BIN(pipeline), "in")) g_object_set(in, "location", file, nullptr), gst_object_unref(in);
}
