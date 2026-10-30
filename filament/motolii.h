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
/* A film's sound: its loudness (dB, 0 at full) 100 times a second of its own, as many as there are (room: how many db
   holds; 0: no sound, or not a film); and the moments of the document where it strikes (grows `rise` dB louder within
   30 ms), as many as there are, where its film shows them. Read once a file, as fast as it decodes. */
int motolii_loudness(Motolii* m, int thing, float* db, int room);
int motolii_strikes(Motolii* m, int thing, double rise, int own, double* at, int room);
int motolii_loudness_over(Motolii* m, int thing, double from, double to, int n, float* db);   /* loudness at n moments of the document, where the film shows them (-100: silent or not shown); -1: not read yet, 0: no sound */   /* own: in the film's own seconds instead */
const char* motolii_marks(Motolii* m);
const char* motolii_mark_colours(Motolii* m);   /* a line each, in time, "moment colour" (1 to 16, the label colours): the marks that have one */
const char* motolii_settings(Motolii* m);   /* the window's choices kept with the document, not steps (asked later: setting): a line each, "name word" */
/* The words a hand types against a shelf's items (`items`: "id<TAB>its name and label<TAB>what it does" a line), nearest
   first, the best `limit` (5 when 0): "id<TAB>score" a line, the score 0 to 100. RapidFuzz (MIT): typos, parts, abbreviations. */
const char* motolii_find(Motolii* m, const char* query, const char* items, int limit);
/* The guides: lines across the picture, x upright and y level, at so many points from its left or top; a change is a
   step. motolii_guides: a line each, "x 150" (until the next call). */
int motolii_guide(Motolii* m, const char* axis, double at, int on);
const char* motolii_guides(Motolii* m);
void motolii_close(Motolii* m);
int motolii_work_out(Motolii* m, const char* sum, double* is);     /* a sum typed in a number field (+ - * / and brackets), as After Effects', Figma's and Blender's take one, worked out by SQLite as the ways are; 0: it is not one */
const char* motolii_trouble(Motolii* m);                           /* why the last call that was not done was not */

const char* motolii_kind_of(const char* file);                     /* what kind of thing a file makes ("": none) */
int motolii_sets(const char* font, const char* words);             /* 1 if a text in that font file can be those words (any thread, while a document is open) */
int motolii_thumbnail(const char* film, const char* png, int wide);   /* a film's first picture written small, as a .png (asks nothing of a document: callable from any thread) */
int motolii_thumbnail_at(const char* film, const char* png, int wide, double fraction);   /* the same, a picture that far along the film (0 to 1) */
int motolii_thumbnail_seconds(const char* film, const char* png, int wide, double seconds);   /* the same, the picture so many seconds in */
const char* motolii_media_info(const char* file);   /* what a media file is, GStreamer's discoverer: "wide high rate seconds format" ("": cannot be read; until the next call on this thread) */
const char* motolii_font_tags(const char* path);   /* a font file's kinds as CoreText reads them: serif sans script mono bold light italic symbols ornaments, and the languages beyond Latin it has (japanese korean chinese russian greek arabic hebrew thai hindi) */
int motolii_add(Motolii* m, const char* kind, const char* file);   /* a new thing (kind "": what the file makes), on from the stage's present moment: its number (0: not made) */
int motolii_remove(Motolii* m, int thing);
int motolii_set(Motolii* m, int thing, const char* name, double number);          /* a value that stays */
int motolii_say(Motolii* m, int thing, const char* name, const char* word);
/* A pen: a point where the hand is (points of the picture from the top left), its handle drawn out by hx, hy (0, 0: a
   corner), on a path shape's path; on its first point again, closed (the next point begins another). thing 0: a new path shape begun there. Its number (0: not). */
int motolii_pen(Motolii* m, int thing, double x, double y, double hx, double hy);
/* A path shape's points where the picture shows them (xy[2i], xy[2i+1]: points from the top left; its handles not), as
   many as there are (room: how many xy holds). One is moved by a hand: motolii_hold, carry "grip" where it took hold,
   carry "point" by how far it has gone, motolii_let_go. */
int motolii_points(Motolii* m, int thing, double* xy, int room);
int motolii_handles(Motolii* m, int thing, double* xy, double* of, int room);   /* and its curves' handles, the same way, of: the point each is the handle of (may be null); carry "point" takes whichever lies nearest */       /* a word */
int motolii_key(Motolii* m, int thing, const char* name, double at, double number);   /* a key at a moment */
int motolii_key_many(Motolii* m, int thing, const char* name, int n, const double* at, const double* number);   /* n keys of one value in one call and one step (at[i], number[i]) */
int motolii_midi(Motolii* m, int thing, const char* name, const char* file, const char* kind, int channel, double bpm);   /* a MIDI file's notes as keys of a value, held, as om_midi keys them: kind velocity (0..1), pitch, count, on (1/0) or bool; channel -1: any; bpm: the file's beats played at it (0: the file's own) */
double motolii_midi_bpm(const char* file);   /* the BPM a MIDI file says (120 if it says none: om_midi's BPM box at first) */
int motolii_unkey(Motolii* m, int thing, const char* name, double at);
int motolii_keys(Motolii* m, int thing, const char* name);         /* how many keys a value has */
/* The stretch from the key at a moment to the next key is itself defined: how the value goes there. A way is a name
   and up to four numbers ("smooth", "hold", "wiggle 3 0.2"). A way of one's own is an expression (SQLite's) in u (how
   far through the stretch, 0 to 1), t (seconds into it), d (its length), a and b (the two keys' values), p1 to p4
   (the numbers said with it); it is taken only if it begins at a and ends at b. */
int motolii_way(Motolii* m, int thing, const char* name, double at, const char* way);
int motolii_define(Motolii* m, const char* way, const char* expression);
/* The effects a thing may be given, "name\tvalue\tlabel" one to a line (label: what it is for, the shelf's kind) (until the next call); one put on a thing at a moment,
   or taken off if it is on (its keys too), one step. Its card shows while it is on (motolii_cards). */
const char* motolii_effect_names(Motolii* m);
int motolii_effect(Motolii* m, int thing, const char* name, double at);
const char* motolii_way_of(Motolii* m, int thing, const char* name, double at);   /* the way of the stretch that begins at that key, as it is said ("": no key there; until the next call) */
const char* motolii_thing_stretches(Motolii* m, int thing);   /* every stretch of a thing's keyed values, a line each: "from\tto\tway" (until the next call) */
const char* motolii_ways(Motolii* m);
const char* motolii_way_names(Motolii* m);                         /* the names alone, with spaces between: the nine kinds, then the ways defined here (until the next call) */
const char* motolii_way_numbers(Motolii* m, const char* kind);   /* a kind's numbers as a hand takes them: name, lowest, highest for each ("seed 1 999 grain 0 0.5 ..."); "" for a kind with none or a way defined in the document */
double motolii_way_at(Motolii* m, const char* way, double u);      /* how a way goes, one of the nine kinds or one defined in the document: how far (0 at its beginning, 1 at its end) at u of the way through */                              /* the names of the ways the document knows (until the next call) */
int motolii_value(Motolii* m, int thing, const char* name, double at, double* is);   /* a value at a moment, as the stage is told it */
int motolii_shows(Motolii* m, int thing, const char* name, double at, double* is);   /* as the stage shows it: 2 driven (by a relation, a behaviour), 1 its own, 0 none */
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
int motolii_seen(Motolii* m, int thing, double* box);   /* what of it is seen: the upright box of its outline on the picture, box[0..3] = left, top, right, bottom (points from the top left). 0: not shown */
/* How a thing moves over a while, as its copy is placed (its wave, scattering, copies' rows and keys all told), n moments
   from `from` to before `to`: path[4i..4i+3] = where it is (x, y, units of the stage), how far it is turned (degrees) and
   how large (a share of its own size). For a shell's small preview that plays by itself. 0: there is no such thing. */
int motolii_path(Motolii* m, int thing, int copy, double from, double to, int n, double* path);
int motolii_path3(Motolii* m, int thing, int copy, double from, double to, int n, double* path);   /* the same in the round: x, y, z (units of the stage) and how large */
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
const char* motolii_cards_more(Motolii* m, int thing);   /* the cards it may be given and is not shown: asked for by the maker (title: names; ...) */
/* How a value is shown to a hand: "look\tunit\tplain name\tlow\thigh\tfold" (empty: as its range says; until the next
   call). The look is count, onoff, angle, stage (a length of the stage, shown in points of the picture), down (one up the stage, shown down it), share (0 to 1,
   shown in hundredths), colour, colours, numbers, thing, seed or none; low and high the span its knob covers, in the
   document's own units; fold the name of the fold it lies in on its card (empty: not folded). */
const char* motolii_shown(Motolii* m, const char* name);
/* Pictures in one call: motolii_recipes says each recipe's name and what it makes ("name\twhat", one to a line, until
   the next call); motolii_make makes one at the stage's present moment, as one step, and says the new thing (0: not). */
const char* motolii_recipes(Motolii* m);
/* A thing's look (its values, effects and keys, all but what it is, holds and where it lies) kept as a file of its
   own (motolii_look_save), given to another thing as one step (motolii_look_put); motolii_looks says the shelf's
   ("name\tpath\tlabel\tpicture", one to a line, until the next call). */
int motolii_look_save(Motolii* m, int thing, const char* path);
int motolii_look_put(Motolii* m, int thing, const char* path);
const char* motolii_looks(Motolii* m);
int motolii_make(Motolii* m, const char* recipe);
/* The whole-screen effects on the shelf ("name\tpath\tlabel\tpicture\thome\tsame\tabout\ttags", one to a line, tags what it is read from it (isf.h isf_tags), label as the file's "// --label:" says;
   picture its tile ("" none yet), home where one drawn for it goes, same "same" when it is seen to change nothing (doc_recipes.h doc_tile); until the next call): the stage's script is the
   paths of those on it, with spaces between, the first nearest the picture. AviUtl2's scripts after them (aviutl_scripts/,
   to any depth; an @ file's each "file#name"), each for a thing's fx. */
const char* motolii_effects(Motolii* m);
/* A file or folder the shelves read from was written, put in or taken out (the window's own watch says so, or a hand
   asks): the shelves, effects, scripts and recipes ask their files again at their next use, and not before. */
void motolii_files_changed(void);
/* The motions a thing may be given (Cavalry's Animators: "name\tpath\tlabel", one to a line, until the next call): a thing's
   script is the path of the one it has (a text given one moves letter by letter). */
const char* motolii_animators(Motolii* m);
/* What the inside's hand has spent its time on since it was opened (a line of words, for measuring). */
const char* motolii_spent(Motolii* m);
/* Whether the inside is telling the stage a moment just now (the document held): a shell that reads each picture may
   leave this picture's reading out rather than wait (it is asked without the document, at once). */
/* What the document's steps changed since last asked, a line each ("things" when a thing came or went, then "thing
   name" per value; asking forgets them), and the names of the values that change what cards a thing has. */
const char* motolii_heard(Motolii* m);   /* each thing's sound as heard now: "file wanted is playing" a line (seconds in its file; -1 not heard): for checks */
const char* motolii_shaping(Motolii* m);
/* A change asked without waiting for the hand (say, set, let_go, undo, redo, together: number is let_go's keep and
   together's begin 1 / end 0; thing -1: the thing the last add made; set with word "made": the number is that thing,
   as a value naming a thing; relate: the things in word (numbers) put in the relation `thing` as they show now; free: the thing out of the
   relation `number`, as if at strength 0 for good: back to the relation under it, or its own value; lead: the relation `thing` led by the thing `number`, nothing moving; release: the relation
   `thing` let go, each of its things where it shows now; key: a key of `name` at the moment in word holding number): queued in order behind all others, a ticket said at once. motolii_answer: -1 not done
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
int motolii_at(Motolii* m, double seconds);                        /* the stage's moment: sent, not waited for (a later one replaces it); what was drawn, and when, motolii_drawn_at */
int motolii_pack(Motolii* m, const char* folder, const char* name);   /* the document (as name.motolii) and the files its things are made of, copied into a folder of their own, the copy's words told their names there: how many files (-1: not) */
int motolii_poster(Motolii* m, const char* png);                   /* the document's own picture (at its poster_at, else now), 256 on its longer side, written to png and kept in the document with its size and length */
const char* motolii_doc_facts(const char* path, const char* png);  /* a document read without opening it: "wide high seconds rate" as it was last kept, its picture written to png (null: not); "": none kept (any thread) */
int motolii_still(Motolii* m, const char* png);                    /* the stage as it is now, as a picture */
int motolii_write(Motolii* m, const char* mp4);                    /* the whole document as a film (.mp4 H.264, .mov HEVC, .gif, .png a picture each: kit_draw.h), with its films' sound; other calls are answered meanwhile (from another thread), but the document is not changed and nothing is held until it is done */
int motolii_rate(Motolii* m);                                      /* how many pictures a second that film has (a shell that counts in frames counts in these) */
char* motolii_picture(Motolii* m, int which, long since, long* version);   /* the window's pictures of the document (skin/BRIDGE.md), read without the document: 1 the document's (its own part and each thing's part newer than since), 2 the moment's; JSON, a copy freed with free(); *version: as of which */
void motolii_tell(Motolii* m, void (*told)(int which));   /* the window's one listener, called (from the hand) when pictures are new: 1 the document's, 2 the moment's, 4 the kept pictures (motolii_kept), 8 an answer to a change asked later (motolii_answer), or'd */
const char* motolii_hand_step(Motolii* m);   /* where the inside's hand is in its round now ("waiting for a change or a beat", "doing the changes asked", "drawing", "putting the films at the moment"...), read without waiting: why a shell waits long */
int motolii_asked(Motolii* m);   /* how many changes asked later are not begun yet */
void motolii_watch(Motolii* m, int on);   /* on from the opening; while on, the window's (main) thread may not wait on the hand: a change it waits for, or a read waiting on the document over 2 ms, is kept */
const char* motolii_broken(Motolii* m);   /* what was kept since motolii_watch(1): one to a line, named and timed; "" none */
long motolii_broken_count(Motolii* m);   /* how many times the window's thread waited since it was watched (watched from the opening), read without waiting */
int motolii_sequence(Motolii* m, double overlap);   /* Sequence Layers: the things that have a length and stand on their own, from the front, one after another (less `overlap` seconds), on frames, keys moved with them; one step */
const char* motolii_list(Motolii* m);                              /* every fact of the document, a line each (until the next call) */
/* The stage's picture as it is drawn, for a shell to show without a copy. The inside keeps its own clock: it draws
   when something has changed or time is going on, and the shell asks only for the last finished picture.
   motolii_front: a CVPixelBufferRef on macOS, held for the caller (who lets it go); it and motolii_frames (how many
   pictures have been finished) may be called from any thread. */
int motolii_live(Motolii* m);
void* motolii_front(Motolii* m);
/* The desk: the editing window's whole stage pane, w x h pixels, the picture drawn there instead of on the stage's own
   buffer (motolii_front keeps its last), the stage lying with its top left at x, y, `scale` pixels a point of it, and
   what lies off the stage seen round it, the picked thing marked over it (`point`: pixels a point of the screen the
   marks are sized in). w 0: off. Made again when its size changes. 1: on. */
int motolii_desk(Motolii* m, int w, int h, double x, double y, double scale, double point);
int motolii_pick(Motolii* m, int thing, int hover);   /* the thing picked in the window (0: none) and the one under its hand: marked on the desk */
void* motolii_desk_front(Motolii* m);
unsigned long long motolii_desk_frames(Motolii* m);
unsigned long long motolii_frames(Motolii* m);
/* Told on the hand's thread whenever a picture (the stage's or the desk's) is finished: `tell(with)`; then motolii_front
   or motolii_desk_front has it (Thermion's markTextureFrameAvailable: the shell asks nothing on a clock of its own). */
void motolii_finished(Motolii* m, void (*tell)(void*), void* with);
/* The stage's pictures kept on disk for playing (stage_kept.h): as stretches of frames at motolii_rate, "from to from to"
   ("" none), and how many times that has changed (read without waiting, as motolii_revision). */
const char* motolii_kept(Motolii* m);
/* A thing's effect cards' titles in the order its effects are laid, ";" between ("" none): ThorVG's own and files' alike.
   Moved with motolii_later "fx_move" (word: a title, number: places, up less than 0). */
const char* motolii_fx_titles(Motolii* m, int thing);
int motolii_play(Motolii* m, int playing);                         /* time goes on by the clock (and round again at the end), or stands */
double motolii_now(Motolii* m);                                    /* the stage's moment */
void motolii_timed(Motolii* m, int pictures);                      /* played by the clock for that many pictures: says on its output what it cost and what was left out */
#ifdef __cplusplus
}
#endif
#endif
