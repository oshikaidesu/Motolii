/* (a check: a recipe as code. The sunflower made from recipes/sunflower.js; its knobs on its own card, with their
   ranges, labels and words; a knob changed changes the picture; a recipe file put in recipes/ is on the shelf at once)
   out/script_check <document> <dir for pictures> */
#include "../motolii.h"
#include <stdio.h>
#include <string.h>
#include <unistd.h>
static int same(const char* a, const char* b) {
    FILE *x = fopen(a, "rb"), *y = fopen(b, "rb");
    int s = x && y, cx = 0, cy = 0;
    while (s && (cx = fgetc(x)) == (cy = fgetc(y)) && cx != EOF) {}
    s = s && cx == cy;
    if (x) fclose(x);
    if (y) fclose(y);
    return s;
}
int main(int argc, char** argv) {
    Motolii* m = argc > 2 ? motolii_open(argv[1], 540, 960) : 0;
    if (!m) return printf("FAIL not opened\n"), 1;
    int bad = 0;
#define SAY(ok, ...) (printf("%s ", (ok) ? "ok  " : (bad++, "FAIL")), printf(__VA_ARGS__), printf("\n"))
    const int f = motolii_make(m, "Sunflower");
    SAY(f > 0, "made the sunflower: thing %d %s", f, f ? "" : motolii_trouble(m));
    const char* cards = motolii_cards(m, f);
    SAY(strstr(cards, "Script: script js_spread js_grow js_turn js_pulse js_order") != 0, "its cards: %s", cards);
    double low = 0, high = 0, at = 0;
    SAY(motolii_range(m, f, "js_spread", &low, &high) && low == 0 && high == 5, "js_spread goes %g to %g", low, high);
    SAY(motolii_value(m, f, "js_spread", 0, &at) && at == 1.3, "js_spread as it comes: %g", at);
    SAY(strstr(motolii_shown(m, "js_spread"), "Spread") != 0, "shown as: %s", motolii_shown(m, "js_spread"));
    SAY(!strcmp(motolii_words(m, f, "js_order"), "in out"), "js_order may be: %s", motolii_words(m, f, "js_order"));
    char a[512], b[512], c[512];
    snprintf(a, sizeof a, "%s/script_a.png", argv[2]), snprintf(b, sizeof b, "%s/script_b.png", argv[2]), snprintf(c, sizeof c, "%s/script_c.png", argv[2]);
    motolii_still(m, a);
    SAY(motolii_set(m, f, "js_spread", 0.6), "js_spread set to 0.6");
    motolii_still(m, b);
    SAY(!same(a, b), "the picture changed with it");
    SAY(motolii_say(m, f, "js_order", "out"), "js_order said out");
    motolii_still(m, c);
    SAY(!same(b, c), "and with the word");
    SAY(!motolii_set(m, f, "js_wobble", 1), "a knob it has not: %s", motolii_trouble(m));
    const char* put = "../../recipes/zz_check_put.js";   /* (beside out/, where the library reads them) */
    FILE* w = fopen(put, "w");
    fprintf(w, "// Put here: a recipe dropped in\n// make shape circle\n// set copies 3\n// --track@far:Far,0,2,1,0.1\nfunction each(i, n, t, k) { return { x: (i - 1) * k.far }; }\n");
    fclose(w);
    const int p = motolii_make(m, "Put here");
    SAY(p > 0 && strstr(motolii_cards(m, p), "js_far") != 0, "a file put in recipes/ made at once: thing %d, cards %s", p, p ? motolii_cards(m, p) : motolii_trouble(m));
    unlink(put);
    motolii_close(m);
    printf(bad ? "FAIL %d\n" : "PASS\n", bad);
    return bad != 0;
}
