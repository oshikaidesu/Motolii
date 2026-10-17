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
/* The document on a stage (0: not a document). A document says its own size: one that is there is opened at it; a
   new one is made `wide` by `high` points. motolii_size: the document says another size (a step of undo); the stage it
   is open on stays the size it was opened at. motolii_size_of: the size of that stage. motolii_fits: 1 if the stage
   is the size the document says now; 0 (after motolii_size, or an undo or redo of one): close it and open it again. */
Motolii* motolii_open(const char* document, int wide, int high);
int motolii_size(Motolii* m, int wide, int high);
int motolii_size_of(Motolii* m, int* wide, int* high);
int motolii_fits(Motolii* m);
/* The work area: the stretch played round and written as a film (not said: all of it); a change is a step. The marks:
   moments with a name; motolii_mark with "" takes the one there away; motolii_marks: a line each, in time, "moment name"
   (until the next call). */
int motolii_work(Motolii* m, double from, double to);
int motolii_loop(Motolii* m, int on);   /* play goes round the work area (1, as a document is opened) or stops at its end (0); below 0: only asks */
int motolii_work_of(Motolii* m, double* from, double* to);
int motolii_mark(Motolii* m, double at, const char* word);
const char* motolii_marks(Motolii* m);
/* The guides: lines across the picture, x upright and y level, at so many points from its left or top; a change is a
   step. motolii_guides: a line each, "x 150" (until the next call). */
int motolii_guide(Motolii* m, const char* axis, double at, int on);
const char* motolii_guides(Motolii* m);
void motolii_close(Motolii* m);
const char* motolii_trouble(Motolii* m);                           /* why the last call that was not done was not */

const char* motolii_kind_of(const char* file);                     /* what kind of thing a file makes ("": none) */
int motolii_sets(const char* font, const char* words);             /* 1 if a text in that font file can be those words (any thread, while a document is open) */
int motolii_thumbnail(const char* film, const char* png, int wide);   /* a film's first picture written small, as a .png (asks nothing of a document: callable from any thread) */
int motolii_add(Motolii* m, const char* kind, const char* file);   /* a new thing (kind "": what the file makes), on from the stage's present moment: its number (0: not made) */
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
/* The effects a thing may be given, "name\tvalue" one to a line (until the next call); one put on a thing at a moment,
   or taken off if it is on (its keys too), one step. Its card shows while it is on (motolii_cards). */
const char* motolii_effect_names(Motolii* m);
int motolii_effect(Motolii* m, int thing, const char* name, double at);
const char* motolii_way_of(Motolii* m, int thing, const char* name, double at);   /* the way of the stretch that begins at that key, as it is said ("": no key there; until the next call) */
const char* motolii_ways(Motolii* m);
const char* motolii_way_names(Motolii* m);                         /* the names alone, with spaces between: the nine kinds, then the ways defined here (until the next call) */
double motolii_way_at(Motolii* m, const char* way, double u);      /* how a way goes, one of the nine kinds or one defined in the document: how far (0 at its beginning, 1 at its end) at u of the way through */                              /* the names of the ways the document knows (until the next call) */
int motolii_value(Motolii* m, int thing, const char* name, double at, double* is);   /* a value at a moment, as the stage is told it */
/* What a shell reads to draw its lists. motolii_things: the numbers of the things there are (as many as there is room
   for; it returns how many there are). motolii_word: a thing's word by name (kind, file, shape, blend, text, fill; ""
   if it has none; until the next call), or by "name" what it is called where things are listed (a text by its
   words, a shape by its shape, a file by its name). motolii_lasts: how long a thing is on, in seconds (below 0: for good).
   motolii_key_moments: the moments of a value's keys (none if the value stays). */
int motolii_things(Motolii* m, int* things, int room);
const char* motolii_word(Motolii* m, int thing, const char* name);
double motolii_lasts(Motolii* m, int thing);
int motolii_key_moments(Motolii* m, int thing, const char* name, double* at, int room);
int motolii_thing_key_moments(Motolii* m, int thing, double* at, int room);   /* every key moment of any of a thing's values, each once, in time (as many as there are; room: how many at holds) */
/* A thing's frame on the picture at the stage's moment: frame[0..4] = its middle (x, y), how wide, how high (points
   of the picture, from the top left), how far it is turned (degrees, clockwise as seen); frame[5..6] = the point it turns about (x, y); frame[7] = how deep it is (0: flat). 0: there is no such thing. */
int motolii_frame(Motolii* m, int thing, double* frame);
/* How a thing moves over a while, as its copy is placed (its wave, scattering, copies' rows and keys all told), n moments
   from `from` to before `to`: path[4i..4i+3] = where it is (x, y, units of the stage), how far it is turned (degrees) and
   how large (a share of its own size). For a shell's small preview that plays by itself. 0: there is no such thing. */
int motolii_path(Motolii* m, int thing, int copy, double from, double to, int n, double* path);
/* A thing held by a hand, carried, and let go: the one form of everything a hand drags. What is carried shows at once
   and is not the document's until it is let go; then it is one step (keep 0: nothing happened). The reckoning is the
   inside's: a shell says where the hand is, never what the document is to hold.
     by "place"          carried on the stage: a, b = how far the hand has gone since it took hold, in points of the
                         picture (right, down); the thing stays under the hand at its own depth
     by "bar"            its bar slid on the timeline: a = how many seconds later it comes on
     by "head", "tail"   its bar cut at an end: a = how many seconds later that end is (a film cut at its head goes on
                         showing at each moment what it showed there)
     by a value's name   that number carried: a = what it is now
   motolii_hold on the thing already held goes on with the same hold. */
int motolii_hold(Motolii* m, int thing);
int motolii_carry(Motolii* m, const char* by, double a, double b);
int motolii_let_go(Motolii* m, int keep);
double motolii_point(Motolii* m);                                  /* points of the picture to one unit of the stage, at z 0 */
const char* motolii_reading(Motolii* m, int thing, const char* name);  /* how a hand reads a value: "times from" (shown = value x times + from) */
/* What a hand does to a value at a moment, by the document's own rules. motolii_put: the value changed there (see
   motolii_keying for what that is to a value with keys and to one without). motolii_key_here: a key at that moment holding
   what the value is there. motolii_stay: its keys go, it stays what it is at that moment. motolii_keyed: 0 the value has no keys, 1 keys but none at that moment, 2 a key at that
   moment. motolii_moving: the names of a thing's values that take keys, with spaces between (until the next call). */
int motolii_put(Motolii* m, int thing, const char* name, double at, double number);
int motolii_keying(Motolii* m, int on);                            /* on: every value a hand changes or carries gets a key at that moment, and its first key leaves what it was as a key at the moment this was turned on, so it travels from there; off (as a document is opened): a value with keys is moved whole, every key by as much; below 0: only asks. Returns whether it is on now */
int motolii_key_here(Motolii* m, int thing, const char* name, double at);
int motolii_stay(Motolii* m, int thing, const char* name, double at);
/* A thing brought `places` places nearer among the things (below 0: further back). motolii_things lists them in the
   order they lie, the one furthest back first; at the same depth a nearer thing is seen in front. */
int motolii_raise(Motolii* m, int thing, int places);
int motolii_keyed(Motolii* m, int thing, const char* name, double at);
const char* motolii_moving(Motolii* m, int thing);
/* motolii_values: the names of all a thing's values, with spaces between (until the next call). motolii_range: a number
   value's lowest and highest (0: it has no such number). */
const char* motolii_values(Motolii* m, int thing);
int motolii_range(Motolii* m, int thing, const char* name, double* low, double* high);
/* The cards a maker sees a thing's values on, besides where it is and its fill: "title: name name; title: name" (until
   the next call). A value added to the document's cards is shown by a shell that reads this, with nothing of its own. */
const char* motolii_cards(Motolii* m, int thing);
/* How a value is shown to a hand: "look\tunit\tplain name\tlow\thigh\tfold" (empty: as its range says; until the next
   call). The look is count, onoff, angle, stage (a length of the stage, shown in points of the picture), down (one up the stage, shown down it), share (0 to 1,
   shown in hundredths), colour, colours, numbers, thing, seed or none; low and high the span its knob covers, in the
   document's own units; fold the name of the fold it lies in on its card (empty: not folded). */
const char* motolii_shown(Motolii* m, const char* name);
/* Pictures in one call: motolii_recipes says each recipe's name and what it makes ("name\twhat", one to a line, until
   the next call); motolii_make makes one at the stage's present moment, as one step, and says the new thing (0: not). */
const char* motolii_recipes(Motolii* m);
int motolii_make(Motolii* m, const char* recipe);
/* The whole-screen effects on the shelf ("name\tpath", one to a line, until the next call): the stage's script is the
   paths of those on it, with spaces between, the first nearest the picture. */
const char* motolii_effects(Motolii* m);
/* The motions a thing may be given (Cavalry's Animators: "name\tpath", one to a line, until the next call): a thing's
   script is the path of the one it has (a text given one moves letter by letter). */
const char* motolii_animators(Motolii* m);
/* What the inside's hand has spent its time on since it was opened (a line of words, for measuring). */
const char* motolii_spent(Motolii* m);
/* Whether the inside is telling the stage a moment just now (the document held): a shell that reads each picture may
   leave this picture's reading out rather than wait (it is asked without the document, at once). */
int motolii_busy(Motolii* m);
/* What the document's steps changed since last asked, a line each ("things" when a thing came or went, then "thing
   name" per value; asking forgets them), and the names of the values that change what cards a thing has. */
const char* motolii_changes(Motolii* m);
const char* motolii_heard(Motolii* m);   /* each thing's sound as heard now: "file wanted is playing" a line (seconds in its file; -1 not heard): for checks */
const char* motolii_shaping(Motolii* m);
/* A change asked without waiting for the hand (say, set, let_go, undo, redo, together: number is let_go's keep and
   together's begin 1 / end 0): queued in order behind all others, a ticket said at once. motolii_answer: -1 not done
   yet, 0 refused (motolii_answer_why says why, until the next answer), 1 done; an answer is said once. */
int motolii_later(Motolii* m, const char* call, int thing, const char* name, const char* word, double number);
int motolii_answer(Motolii* m, int ticket);
const char* motolii_answer_why(Motolii* m);
const char* motolii_words(Motolii* m, int thing, const char* name);   /* the words a value of a thing may be, with spaces between ("": any, or it is not a word; until the next call) */
/* One act that is several calls (a value changed and its key put): the changes made from motolii_together(m, 1) to
   motolii_together(m, 0) are one step of undo, and if one of them is refused none of them is made (0 comes back).
   Acts may lie one within another: the outermost is the step. */
int motolii_together(Motolii* m, int on);
int motolii_undo(Motolii* m);
int motolii_redo(Motolii* m);

double motolii_seconds(Motolii* m);                                /* how long the document runs */
int motolii_at(Motolii* m, double seconds);                        /* the stage's moment */
int motolii_still(Motolii* m, const char* png);                    /* the stage as it is now, as a picture */
int motolii_write(Motolii* m, const char* mp4);                    /* the whole document as a film, with its films' sound; other calls are answered meanwhile (from another thread), but the document is not changed and nothing is held until it is done */
int motolii_rate(Motolii* m);                                      /* how many pictures a second that film has (a shell that counts in frames counts in these) */
const char* motolii_list(Motolii* m);                              /* every fact of the document, a line each (until the next call) */
/* The stage's picture as it is drawn, for a shell to show without a copy. The inside keeps its own clock: it draws
   when something has changed or time is going on, and the shell asks only for the last finished picture.
   motolii_front: a CVPixelBufferRef on macOS, held for the caller (who lets it go); it and motolii_frames (how many
   pictures have been finished) may be called from any thread. */
int motolii_live(Motolii* m);
void* motolii_front(Motolii* m);
/* One thing alone, for a card's small picture of it (After Effects' solo, in a picture of its own: the stage is not
   changed): drawn with the stage's own pictures, side by side points, square, the eye on the thing's frame. thing 0:
   none. 1 if it can be shown (the stage is live). Its picture and count as motolii_front and motolii_frames give the
   stage's. */
int motolii_solo(Motolii* m, int thing, int side);
void* motolii_solo_front(Motolii* m);
unsigned long long motolii_solo_frames(Motolii* m);
/* The desk: the editing window's whole stage pane, w x h pixels, the picture drawn there instead of on the stage's own
   buffer (motolii_front keeps its last), the stage lying with its top left at x, y, `scale` pixels a point of it, and
   what lies off the stage seen round it. w 0: off. Made again when its size changes. 1: on. */
int motolii_desk(Motolii* m, int w, int h, double x, double y, double scale);
void* motolii_desk_front(Motolii* m);
unsigned long long motolii_desk_frames(Motolii* m);
unsigned long long motolii_frames(Motolii* m);
int motolii_play(Motolii* m, int playing);                         /* time goes on by the clock (and round again at the end), or stands */
double motolii_now(Motolii* m);                                    /* the stage's moment */
void motolii_timed(Motolii* m, int pictures);                      /* played by the clock for that many pictures: says on its output what it cost and what was left out */
#ifdef __cplusplus
}
#endif
#endif
