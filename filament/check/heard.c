// Is a film's own sound heard where the live stage's moment puts it? A film with sound, coming on half a second in,
// cut from its second second, twice as fast: played live (muted: MOTOLII_MUTE), its sound must stand within 0.15 s of
// where the stage puts it, go twice as fast as the stage, and stop when the stage stops.   check/heard.sh
#include "../motolii.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int sound(Motolii* m, double* wanted, double* is, int* on) {   // (the first line of motolii_heard)
    char file[512];
    return sscanf(motolii_heard(m), "%511s %lf %lf %d", file, wanted, is, on) == 4;
}

int main(int argc, char** argv) {
    Motolii* m = motolii_open(argv[1], 640, 360);
    if (!m) return printf("FAIL not opened\n"), 1;
    const int film = motolii_add(m, "film", argv[2]);
    if (!film || !motolii_set(m, film, "start", 0.5) || !motolii_set(m, film, "begin", 2) || !motolii_set(m, film, "speed", 2)) return printf("FAIL %s\n", motolii_trouble(m)), 1;
    if (!motolii_live(m)) return printf("FAIL not live\n"), 1;
    motolii_at(m, 0);
    motolii_play(m, 1);
    usleep(2000000);
    double w1, i1, w2, i2;
    int on1, on2;
    if (!sound(m, &w1, &i1, &on1)) return printf("FAIL no sound is heard: %s\n", motolii_heard(m)), 1;
    usleep(1000000);
    sound(m, &w2, &i2, &on2);
    motolii_play(m, 0);
    usleep(300000);
    double w3, i3;
    int on3;
    sound(m, &w3, &i3, &on3);
    const int near = on1 && fabs(i1 - w1) < 0.15 && fabs(i2 - w2) < 0.15, twice = fabs((i2 - i1) - 2.0) < 0.2, stops = !on3;
    printf("playing: where the stage puts it %.3f s, heard at %.3f s; a second later %.3f, heard %.3f (%.2f s of its file in a second); stopped: playing %d\n", w1, i1, w2, i2, i2 - i1, on3);
    printf("the sound goes with the stage: %s\n", near && twice && stops ? "yes" : "NO");
    motolii_close(m);
    return near && twice && stops ? 0 : 1;
}
