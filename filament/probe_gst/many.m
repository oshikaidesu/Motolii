// N films read at once in ONE program, each by its own GStreamer pipeline with the Mac's hardware decoder; every
// picture is pulled out and checked to be an IOSurface (nothing is copied or looked at). How long, how many threads.
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
#include <mach/mach.h>
#include <pthread.h>
typedef struct { GstMeta meta; CVBufferRef cvbuf; CVPixelBufferRef pixbuf; } CoreVideoMeta;
static int g_most_threads = 0;
static int threads_now(void) { thread_act_array_t l; mach_msg_type_number_t n = 0; task_threads(mach_task_self(), &l, &n); vm_deallocate(mach_task_self(), (vm_address_t)l, n * sizeof(thread_t)); return (int)n; }
typedef struct { GstElement* pipe; GstAppSink* out; int pulled, surfaces; } Film;
static void* pull(void* p) {
    Film* f = p;
    GType api = 0;
    for (GstSample* s; (s = gst_app_sink_pull_sample(f->out)); gst_sample_unref(s), ++f->pulled) {
        if (!api) api = g_type_from_name("GstCoreVideoMetaAPI");   // (the plugin names it when its first picture is made)
        CoreVideoMeta* m = api ? (CoreVideoMeta*)gst_buffer_get_meta(gst_sample_get_buffer(s), api) : NULL;
        if (m && m->pixbuf && CVPixelBufferGetIOSurface(m->pixbuf)) ++f->surfaces;
        if (f->pulled % 60 == 0) { int t = threads_now(); if (t > g_most_threads) g_most_threads = t; }
    }
    return NULL;
}
int main(int argc, char** argv) {
    gst_init(&argc, &argv);
    const int n = atoi(argv[2]);
    Film* films = g_new0(Film, n);
    pthread_t* who = g_new0(pthread_t, n);
    gint64 t0 = g_get_monotonic_time();
    for (int i = 0; i < n; ++i) {
        gchar* line = g_strdup_printf("filesrc location=\"%s\" ! qtdemux ! h264parse ! vtdec_hw ! video/x-raw,format=NV12 ! appsink name=out sync=false max-buffers=4", argv[1]);
        films[i].pipe = gst_parse_launch(line, NULL);
        films[i].out = GST_APP_SINK(gst_bin_get_by_name(GST_BIN(films[i].pipe), "out"));
        gst_element_set_state(films[i].pipe, GST_STATE_PLAYING);
        pthread_create(&who[i], NULL, pull, &films[i]);
    }
    int pulled = 0, surfaces = 0;
    int whole = 0, none = 0, most = 0;
    for (int i = 0; i < n; ++i) { pthread_join(who[i], NULL); pulled += films[i].pulled; surfaces += films[i].surfaces; if (films[i].pulled > most) most = films[i].pulled; }
    for (int i = 0; i < n; ++i) { whole += films[i].pulled == most; none += films[i].pulled == 0; }
    double took = (g_get_monotonic_time() - t0) / 1e6, length = atof(argv[3]);
    g_print("%2d films at once in one program: %d pictures pulled, %d of them GPU surfaces, in %.2f s for %.1f s of film -> %.1f times as fast as needed; %d threads at most; %d films whole, %d gave nothing\n",
            n, pulled, surfaces, took, length, length / took, g_most_threads, whole, none);
    return 0;
}
