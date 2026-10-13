/* (a check, not a scene: motolii.h called from plain C, as dart:ffi would call it: a document, a glass ring, a value, an
   undo, a wrong call, a still)   out/api_check <document> <out.png> */
#include "../motolii.h"
#include <stdio.h>
int main(int argc, char** argv) {
    Motolii* m = argc > 2 ? motolii_open(argv[1], 540, 960) : 0;
    if (!m) return printf("not opened\n"), 1;
    const int ring = motolii_add(m, "solid", "ring");
    printf("a ring: thing %d; its fill is glass: %d\n", ring, motolii_say(m, ring, "fill", "glass"));
    printf("set x 0.4: %d   key y at 1 = 0.5: %d   undo: %d   set wobble: %d (%s)\n", motolii_set(m, ring, "x", 0.4), motolii_key(m, ring, "y", 1.0, 0.5), motolii_undo(m), motolii_set(m, ring, "wobble", 1.0), motolii_trouble(m));
    printf("%sruns %.2f s; still: %d\n", motolii_list(m), motolii_seconds(m), motolii_still(m, argv[2]));
    motolii_close(m);
    return 0;
}
