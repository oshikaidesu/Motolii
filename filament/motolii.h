/* motolii.h: the editing inside, as C calls: all that a shell on a screen (Flutter, by dart:ffi) calls, and nothing
   else. The shell holds no document: it holds this handle and the numbers of things. One call = one change = one step
   of undo, in the document's file as the call returns. A call that is not done returns 0, changes nothing, and
   motolii_trouble() says why.
   kinds: film, picture (an After Effects file, a .jpg, a .png), text (a font, .ttf), solid (.glb, .gltf, or one of ring, gem, cube)
   A solid's fill is what it is made of: a colour, or glass. The names of a thing's values are ThorVG's own where it has one (x, y, rotate, size, blend,
   text, fill, begin): doc.h lists them. */
#ifndef MOTOLII_H
#define MOTOLII_H
#ifdef __cplusplus
extern "C" {
#endif
typedef struct Motolii Motolii;

void motolii_home(const char* folder);                             /* where the inside's own files are (its materials, its room): said once, before a document is opened */
Motolii* motolii_open(const char* document, int wide, int high);   /* the document (made if it is not there) on a stage of that many points; 0: not a document */
void motolii_close(Motolii* m);
const char* motolii_trouble(Motolii* m);                           /* why the last call that was not done was not */

int motolii_add(Motolii* m, const char* kind, const char* file);   /* a new thing, on from the stage's present moment: its number (0: not made) */
int motolii_remove(Motolii* m, int thing);
int motolii_set(Motolii* m, int thing, const char* name, double number);          /* a value that stays */
int motolii_say(Motolii* m, int thing, const char* name, const char* word);       /* a word */
int motolii_key(Motolii* m, int thing, const char* name, double at, double number);   /* a key at a moment */
int motolii_unkey(Motolii* m, int thing, const char* name, double at);
int motolii_keys(Motolii* m, int thing, const char* name);         /* how many keys a value has */
/* The stretch from the key at a moment to the next key is itself defined: how the value goes there. A way is a name
   and up to four numbers ("smooth", "hold", "wiggle 3 0.2"). A way of one's own is an expression (SQLite's) in u (how
   far through the stretch, 0 to 1), t (seconds into it), d (its length), a and b (the two keys' values), p1 to p4
   (the numbers said with it); it is taken only if it begins at a and ends at b. */
int motolii_way(Motolii* m, int thing, const char* name, double at, const char* way);
int motolii_define(Motolii* m, const char* way, const char* expression);
const char* motolii_ways(Motolii* m);                              /* the names of the ways the document knows (until the next call) */
int motolii_value(Motolii* m, int thing, const char* name, double at, double* is);   /* a value at a moment, as the stage is told it */
/* What a shell reads to draw its lists. motolii_things: the numbers of the things there are (as many as there is room
   for; it returns how many there are). motolii_word: a thing's word by name (kind, file, shape, blend, text, fill; ""
   if it has none; until the next call). motolii_lasts: how long a thing is on, in seconds (below 0: for good).
   motolii_key_moments: the moments of a value's keys (none if the value stays). */
int motolii_things(Motolii* m, int* things, int room);
const char* motolii_word(Motolii* m, int thing, const char* name);
double motolii_lasts(Motolii* m, int thing);
int motolii_key_moments(Motolii* m, int thing, const char* name, double* at, int room);
/* A thing's frame on the picture at the stage's moment: frame[0..4] = its middle (x, y), how wide, how high (points
   of the picture, from the top left), how far it is turned (degrees, clockwise as seen). 0: there is no such thing. */
int motolii_frame(Motolii* m, int thing, double* frame);
/* While a hand is still on a thing (a drag): a value shown and not written. The document is not changed and no step
   of undo is made; motolii_set (or any change) settles it, motolii_try_drop puts the document's own value back. */
int motolii_try(Motolii* m, int thing, const char* name, double number);
int motolii_try_drop(Motolii* m);
double motolii_point(Motolii* m);                                  /* points of the picture to one unit of the stage, at z 0 */
int motolii_undo(Motolii* m);
int motolii_redo(Motolii* m);

double motolii_seconds(Motolii* m);                                /* how long the document runs */
int motolii_at(Motolii* m, double seconds);                        /* the stage's moment */
int motolii_still(Motolii* m, const char* png);                    /* the stage as it is now, as a picture */
int motolii_write(Motolii* m, const char* mp4);                    /* the whole document as a film, with its films' sound */
const char* motolii_list(Motolii* m);                              /* every fact of the document, a line each (until the next call) */
/* The stage's picture as it is drawn, for a shell to show without a copy. The inside keeps its own clock: it draws
   when something has changed or time is going on, and the shell asks only for the last finished picture.
   motolii_front: a CVPixelBufferRef on macOS, held for the caller (who lets it go); it and motolii_frames (how many
   pictures have been finished) may be called from any thread. */
int motolii_live(Motolii* m);
void* motolii_front(Motolii* m);
unsigned long long motolii_frames(Motolii* m);
int motolii_play(Motolii* m, int playing);                         /* time goes on by the clock (and round again at the end), or stands */
double motolii_now(Motolii* m);                                    /* the stage's moment */
void motolii_timed(Motolii* m, int pictures);                      /* played by the clock for that many pictures: says on its output what it cost and what was left out */
#ifdef __cplusplus
}
#endif
#endif
