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
    motolii_key(m, ring, "x", 0.0, 0.0), motolii_key(m, ring, "x", 1.0, 1.0), motolii_key(m, ring, "y", 0.0, 0.0), motolii_key(m, ring, "y", 1.0, 1.0);
    motolii_together(m, 1);   /* (two changes by one act of a hand: one step back) */
    const int both = motolii_way(m, ring, "x", 0.0, "Hold") + motolii_way(m, ring, "y", 0.0, "Bezier 0.42 0 0.58 1");
    motolii_together(m, 0);
    printf("two ways told together: %d; x goes %s,", both, motolii_way_of(m, ring, "x", 0.0));
    printf(" y goes %s;", motolii_way_of(m, ring, "y", 0.0));
    printf(" one undo: %d; x goes %s,", motolii_undo(m), motolii_way_of(m, ring, "x", 0.0));
    printf(" y goes %s;", motolii_way_of(m, ring, "y", 0.0));
    printf(" one redo: %d;", motolii_redo(m));
    printf(" x goes %s,", motolii_way_of(m, ring, "x", 0.0));
    printf(" y goes %s;", motolii_way_of(m, ring, "y", 0.0));
    printf(" no key at 0.5: \"%s\"\n", motolii_way_of(m, ring, "x", 0.5));
    printf("a way defined in the document: %d; half way through it is at %.3f (the square: 0.250), a kind of the nine at %.3f\n", motolii_define(m, "square", "a + (b - a) * u * u"), motolii_way_at(m, "square", 0.5), motolii_way_at(m, "Linear", 0.5));
    printf("%sruns %.2f s; still: %d\n", motolii_list(m), motolii_seconds(m), motolii_still(m, argv[2]));
    motolii_close(m);
    return 0;
}
