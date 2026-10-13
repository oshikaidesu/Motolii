/* motolii.h: the editing inside, as C calls: all that a shell on a screen (Flutter, by dart:ffi) calls, and nothing
   else. The shell holds no document: it holds this handle and the numbers of things. One call = one change = one step
   of undo, in the document's file as the call returns. A call that is not done returns 0, changes nothing, and
   motolii_trouble() says why.
   kinds: film, picture (an After Effects file, a .jpg, a .png), type (a font, .ttf), solid (.glb, .gltf), glass
   (ring, gem, cube). The names of a thing's values: doc.h. */
#ifndef MOTOLII_H
#define MOTOLII_H
#ifdef __cplusplus
extern "C" {
#endif
typedef struct Motolii Motolii;

Motolii* motolii_open(const char* document, int wide, int high);   /* the document (made if it is not there) on a stage of that many points; 0: not a document */
void motolii_close(Motolii* m);
const char* motolii_trouble(Motolii* m);                           /* why the last call that was not done was not */

int motolii_add(Motolii* m, const char* kind, const char* file);   /* a new thing: its number (0: not made) */
int motolii_remove(Motolii* m, int thing);
int motolii_set(Motolii* m, int thing, const char* name, double number);          /* a value that stays */
int motolii_say(Motolii* m, int thing, const char* name, const char* word);       /* a word */
int motolii_key(Motolii* m, int thing, const char* name, double at, double number);   /* a key at a moment */
int motolii_unkey(Motolii* m, int thing, const char* name, double at);
int motolii_keys(Motolii* m, int thing, const char* name);         /* how many keys a value has */
int motolii_undo(Motolii* m);
int motolii_redo(Motolii* m);

double motolii_seconds(Motolii* m);                                /* how long the document runs */
int motolii_at(Motolii* m, double seconds);                        /* the stage's moment */
int motolii_still(Motolii* m, const char* png);                    /* the stage as it is now, as a picture */
int motolii_write(Motolii* m, const char* mp4);                    /* the whole document as a film, with its films' sound */
const char* motolii_list(Motolii* m);                              /* every fact of the document, a line each (until the next call) */
void motolii_timed(Motolii* m, int pictures);                      /* played by the clock for that many pictures: says on its output what it cost and what was left out */
#ifdef __cplusplus
}
#endif
#endif
