// Does memory grow while many films play live? Films from the folder given, three of each, laid in a grid, played
// round and round (live, muted: MOTOLII_MUTE) for so many seconds; the process's footprint said every 15 seconds.
//   check/films_memory.sh [seconds]
#include "../motolii.h"
#include <dirent.h>
#include <signal.h>
#include <mach/mach.h>
#include <malloc/malloc.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static double footprint_mb(void) {
    task_vm_info_data_t info;
    mach_msg_type_number_t n = TASK_VM_INFO_COUNT;
    return task_info(mach_task_self(), TASK_VM_INFO, (task_info_t)&info, &n) == KERN_SUCCESS ? info.phys_footprint / 1048576.0 : -1;
}

static double malloc_mb(void) {   // (the part of it that is malloc's: the rest is the GPU's and the films' surfaces)
    malloc_statistics_t st;
    malloc_zone_statistics(NULL, &st);
    return st.size_in_use / 1048576.0;
}

int main(int argc, char** argv) {
    Motolii* m = motolii_open(argv[1], 1080, 1920);
    if (!m) return printf("FAIL not opened\n"), 1;
    DIR* dir = opendir(argv[2]);
    struct dirent* e;
    int films = 0;
    motolii_together(m, 1);
    while (dir && (e = readdir(dir)))
        if (strstr(e->d_name, ".mp4"))
            for (int k = 0; k < 3; ++k) {
                char path[1024];
                snprintf(path, sizeof path, "%s/%s", argv[2], e->d_name);
                const int t = motolii_add(m, "film", path);
                if (!t) continue;
                motolii_set(m, t, "x", -1.5 + (films % 4)), motolii_set(m, t, "y", -2.5 + (films / 4) * 0.8), motolii_set(m, t, "size", 0.6);
                ++films;
            }
    motolii_together(m, 0);
    if (!motolii_live(m)) return printf("FAIL not live\n"), 1;
    motolii_play(m, 1);
    const int seconds = argc > 3 ? atoi(argv[3]) : 180;
    printf("%d films playing\n", films);
    double first = -1;
    for (int s = 0; s <= seconds; s += 15) {
        const double mb = footprint_mb();
        if (s == 30) first = mb;   // (counted from half a minute in: the first moments fill what is made once)
        if (getenv("GST_TRACERS") && (s == 30 || s == seconds)) raise(SIGUSR2);   // (GStreamer's leaks tracer: what came and stayed since)
        printf("  %3d s  %.1f MB  malloc %.1f MB  pictures %llu\n", s, mb, malloc_mb(), motolii_frames(m));
        fflush(stdout);
        if (s < seconds) sleep(15);
    }
    const double grew = footprint_mb() - first, minutes = (seconds - 30) / 60.0;
    printf("grew %.1f MB in %.1f minutes after the first half minute: %.2f MB a minute\n", grew, minutes, grew / minutes);
    motolii_play(m, 0);
    motolii_close(m);
    return 0;
}
