// doc.h: a document of the stage, and the editing of it as calls: what a shell on a screen would call, and nothing
// else. The document is a file of facts: a thing's value, by name, at a time (thing, name, at, value). A value with one
// fact stays as it is; with more it moves, and they are its keys. One call = one change = one step of undo (Lumit's
// bridge is so), and the stage shows it as the call returns. A change the stage cannot show is not made.
// Keys, Motolii's own way: a key holds a value, and THE STRETCH FROM A KEY TO THE NEXT IS ITSELF DEFINED: how the value
// goes there. The kinds of stretch are Motolii's own nine, brought from the past repository as they are (interp.h:
// Hold, Linear, Bezier, Bounce, Elastic, Cyclic, Random, Steps, ElasticSteps, each with its numbers). A way of one's
// own can be added by an expression kept in the document (worked out by SQLite's own expressions: u is how far
// through the stretch, 0 to 1; t the seconds into it; d its length; a and b the two keys' values; p1 to p4 numbers
// said with it), taken only if it begins at the one key's value and ends at the other's.
// Borrowed whole: the file, saving (a change is in the file as its call returns) and undo are SQLite's (its session
// extension: what a call changed is kept, and undone by applying it reversed: the steps are in the file too). A
// thing's time is told in GES's words (start, duration).
// The names a thing's facts go by are ThorVG's own, where it has one (translate's x and y, rotate, size, blend and its
// words, text, fill, a segment's begin, add, remove):
//   numbers  x, y, z, rotate, size  where its middle is, turned by how many degrees, how tall
//            rotate_x, rotate_y     turned about its own x and y too (degrees): a thing leaning back, a thing turned to its side
//            opacity                how much of it is there: 1 all, 0 none (a film's, a picture's, a text's)
//            start, duration        when it comes on, and for how long (not said: for good, going round)
//            begin, speed           a film's: from where in itself, how fast (below 0: backwards)
//            order                  where it lies among the things: one with a greater number is nearer (not said: as they were made)
//            visible                0: it is put aside, not seen (ThorVG's word), though it is still in the document
//            clip                   1: it is seen only inside the frame of the thing before it (a 3D thing cut by a rectangle)
//            wide                   a shape's: how many times as wide as it is high
//            keyed                  a film's: 1 if its ground is keyed out (a green screen, of whatever colour: the colour its four corners share), 0 if not; not said: keyed if it has such a ground
//   words    blend                  how it lies on what is behind: normal | add | multiply | screen
//            file, shape            what it was made from (add says them): a file (a text's: its font); or, a solid's, one of the kit's shapes: ring | gem | cube;
//                                   or, a shape's (a flat one, drawn by ThorVG), rect | circle
//            text                   a text's: what it says (to the end of the line)
//            fill                   what its surface is: a text's colour as six digits (ff3366); a solid's: a colour, or the material glass
#include "drawn.h"
#include "motion.h"
#include "interp.h"
#include "physics.h"
#include "script.h"
#include "ways.h"
#include "screen.h"
#include <filesystem>
#include <mutex>
#include <dlfcn.h>
#include <fstream>
#include "solid.h"
#include "video.h"
#include <filament/Skybox.h>
#include <set>
#include <sstream>
#include <sqlite3.h>

struct Doc {
    std::recursive_mutex lock;   // held by whoever reads or changes it: a shell's reads need not wait for a picture being drawn (motolii.mm)
    sqlite3* file = nullptr;
    Stage* stage = nullptr;
    std::string trouble;   // why the last call that was not done was not
    std::vector<int> laid;   // the things in the order they lie, the one furthest back first (read when first asked for, forgotten at every change)
    // One act of a hand: the changes made from its beginning to its end are one step of undo, and if one of them is
    // refused none of them is made. Acts may lie one within another: the outermost one is the step.
    std::string graded;                 // the stage's grade as last made (doc_look): made again only when it is told otherwise
    std::set<std::pair<int, std::string>> changed;   // the values changed since a shell last asked (motolii_changes), each (thing, name), from each step's own changeset
    bool things_changed = false;        // and whether a thing came or went since
    std::map<uint32_t, MaterialInstance*> copyLooks;   // a copy's own look (its thing's look duplicated, by the copy's entity), while its copies are coloured each their own
    float roomLight = 0.f;              // the room's own light, as the stage was opened with it
    std::string roomIs;                 // which room lights it now (doc_look)
    Skybox* sky = nullptr;              // the room behind everything, once it is said
    Drawn* backdrop = nullptr;          // the colour behind everything: a flat colour as large as the stage, drawn before all else in the stage's own view
    std::string backdropIs;             // (the colour it was made in)
    bool keying = false;                // every value a hand changes gets a key at that moment (a shell's "animate all")
    double keying_from = 0.0;           // the stage's moment at which that was turned on: a value's first key then has the value it had, there, before it
    int act = 0;                        // how many acts are open, one within another
    bool act_ok = true;                 // no change of the open act has been refused
    bool act_unmatched = false;         // a change within a larger act not yet shown: the stage is matched once, when the act ends
    std::unique_ptr<PhysWorld> phys;    // the things that fall and strike (Jolt): made when first asked, made again when what it is made of changes
    std::vector<int> phys_of;           // which thing each of its bodies is
    std::vector<int> phys_copy;         // and which of its copies (a split text's letter; -1: the thing whole)
    std::vector<double> phys_until;     // and till when its keys have it
    bool phys_stale = true, phys_any = false;   // (the document changed since the world was last looked at; some thing has physics)
    std::string phys_made;              // all the world was made of, as the document said it then (made again only when that changes)
    struct Scripted { double at = -1; int made = -1; std::vector<ScriptCopy> copies; };
    std::map<int, Scripted> scripted;
    ScreenStack screen;                 // the stage's whole-screen effects, its script .wgsl files one over another (screen.h)   // each thing's copies as its script said them, at one moment (worked out once a picture)
    bool phys_keyed = false;            // reading where a thing's keys put it (not where the world has it)
    bool writing = false;               // it is being written as a film: it is read, not changed, until that is done
    double written_from = 0.0;          // the moment of the document that a film being written begins at (its work area's start)
    double round = 8.0;                 // how long the document is, as last shown: waves, noise and drifts go round in it a whole number of times (a film loops)
    sqlite3_session* acting = nullptr;  // SQLite's own record of what the open act changes
    // A thing held by a hand (doc_hold): what of it is carried shows at once and is not yet the document's.
    struct Held { int thing = -1; std::map<std::string, double> from, tried; } held;   // (-1: nothing held; 0 is the stage, which a hand may hold by a value of its own)
    struct Keys { std::vector<double> at, value; std::vector<Interp> interp; std::vector<std::string> own; };   // (own: a way defined in the document, where the stretch is not one of the nine kinds)
    std::map<std::pair<int, std::string>, Keys> moving;   // a value's keys, and the way of the stretch after each; read from the facts when first asked for, forgotten at every change
    std::map<std::pair<int, std::string>, std::string> words;   // a value's word, the same way
    std::map<std::pair<int, std::string>, std::vector<double>> keyed;   // the moments of a value's keys, the same way
    std::map<std::string, sqlite3_stmt*> named, goes;     // a way by its name, and by its expression: made ready once (SQLite works it out)
    struct Shown { std::string of; Video* film = nullptr; Drawn* drawn = nullptr; Solid* solid = nullptr; Mesh* shape = nullptr; bool in = true, clipped = false, camera = false; int run = -1, lies = -1;
                   utils::Entity light;   // (a light, Filament's own: in every scene of the stage while it is on)
                   double units = 1.0;    // (a flat thing's: points of the picture to a unit of the stage as it was made, which its effects are counted in)
                   int space = 3;
                   Scene* scene = nullptr; int channel = -1;
                   MaterialInstance* look = nullptr;   // (a solid's own look, made by the kit: what a matte cuts)
                   struct Deep { uint32_t id; Drawn* sheet; float4 box; };   // (an After Effects picture's 3D layer: a sheet of its own where it is)
                   std::vector<Deep> deep;
                   std::shared_ptr<Motion> motion; };   // (what says where they are: the same file, read for its poses only)   // (the scene it is in: none said, the stage's own, where it was made; and its channel there)   // (read as it is made: 2 a flat thing on the camera, 25 one in the world, 3 a solid thing or a camera)
    // The camera of the moment, against the stage's own (from 6 units in front, 30 degrees high): how the world is
    // moved for it (rig; a 2D thing goes with it) and only how it is turned (facing; a 2.5D thing keeps its own angle).
    math::mat4 rig, facing;
    bool blurred = false;   // its lens blurs what is off its focus (solid things and things in the world only)
    // A thing clipped by the flat thing before it: the kit's matte it is cut by (its own), cut by which thing's picture,
    // and which of its own looks were told of it.
    struct Cut { int matte = -1, from = 0; const Drawn* by = nullptr; MaterialInstance* look = nullptr; std::string how; };   // (how: by its clearness or its light, the other way about, how soft: as last cut)
    std::map<int, Cut> cuts;
    std::map<int, Shown> shown;   // the things as they are on the stage, and what each was made of
};
// What a thing may be told, by name: which kinds of thing have it, a number between which two (or one of which
// words), and whether it can move (take keys).
static const char* const doc_kinds = "film picture text solid shape camera light";   // (the kinds of thing)
static const char* const doc_drawn = "film picture text solid shape";          // (those that are seen)
static const char* const doc_shapes = "ring gem cube ball";             // (the kit's own shapes a solid may be)
// (whether a value is keyed, in SQLite's words: see doc_key)
#define DOC_KEYED "(select count(*) > 1 or max(at) <> 0 or max(way) is not null from fact where thing = %d and name = %Q)"
struct DocValue { const char* name; const char* kinds; double low, high; bool moves; const char* words; };
static const DocValue doc_values[] = {
    {"x", doc_kinds, -1e4, 1e4, true, nullptr}, {"y", doc_kinds, -1e4, 1e4, true, nullptr}, {"z", doc_kinds, -1e4, 1e4, true, nullptr},
    {"rotate", doc_kinds, -1e6, 1e6, true, nullptr}, {"rotate_x", doc_kinds, -1e6, 1e6, true, nullptr}, {"rotate_y", doc_kinds, -1e6, 1e6, true, nullptr},
    // (how many times as wide, as high and as deep as its size makes it, each on its own as After Effects' Scale: below 0
    //  turned over; a flat thing has no depth to scale yet, its thickness to come as its scale_z)
    {"scale_x", doc_drawn, -1e4, 1e4, true, nullptr},
    {"scale_y", doc_drawn, -1e4, 1e4, true, nullptr},
    {"scale_z", "solid", -1e4, 1e4, true, nullptr},
    // (falls and strikes others: moves, the world has it from its last key on; still, a wall where the document puts it;
    //  how much it bounces, how it rubs; and the stage's pull down, in units of the stage a second each second)
    {"physics", doc_drawn, 0, 0, false, "none moves still"},
    // (a recipe as code: a JavaScript file that says where each copy is (script.h); its head's knobs are values of the thing, named js_ and the knob's name)
    {"script", "picture text shape stage", 0, 0, false, ""},   // (the stage's: a whole-screen effect, a .wgsl file: screen.h)
    {"bounce", doc_drawn, 0, 1, false, nullptr},
    {"friction", doc_drawn, 0, 2, false, nullptr},
    {"gravity", "stage", -100, 100, false, nullptr},
    {"follow", "stage", 0, 1e9, false, nullptr},
    // (the thing the stage's effects follow in place of a mouse: their pointer, center, mouse or point knobs are where it is on the picture; 0: none)
    {"size", doc_drawn, 1e-3, 1e4, true, nullptr}, {"anchor_x", doc_drawn, -100, 100, true, nullptr}, {"anchor_y", doc_drawn, -100, 100, true, nullptr}, {"anchor_z", doc_drawn, -100, 100, true, nullptr},
    // (a camera, the past repository's: it looks at where it is, from round about it as it is turned, so far away, so near)
    {"distance", "camera", 0.01, 100, true, nullptr}, {"zoom", "camera", 0.01, 100, true, nullptr},
    // (and its lens: how far off from it things are sharp, 0 being where it looks; the f-number, 0: all sharp)
    {"focus", "camera", 0, 1e4, true, nullptr}, {"aperture", "camera", 0, 64, true, nullptr}, {"blur_level", "camera", 0, 100, true, nullptr},   // (After Effects' blur level: how many times the lens's own blur)
    {"opacity", "film picture text shape", 0, 1, true, nullptr},
    {"clip", doc_drawn, 0, 1, false, nullptr},
    // (clipped by a flat thing before it, the kit's matte: by how clear that is or how light (After Effects' alpha and
    //  luma mattes), the other way about (inverted), its edge so soft, in points of the screen)
    {"clip_matte", "film picture text shape", 0, 0, false, "alpha luma"}, {"clip_invert", "film picture text shape", 0, 1, false, nullptr}, {"clip_feather", "film picture text shape", 0, 500, false, nullptr},   // (a solid is cut by a box, not a matte)
    {"solo", doc_kinds, 0, 1, false, nullptr}, {"lock", doc_kinds, 0, 1, false, nullptr},   // (After Effects' own: while any thing is solo only those are seen; a locked thing is not changed)
    {"visible", doc_kinds, 0, 1, false, nullptr}, {"order", doc_kinds, -1e9, 1e9, false, nullptr}, {"wide", "shape", 1e-3, 1e3, false, nullptr}, {"shape", "shape", 0, 0, false, "rect circle solid polygon star line wave null"},
    {"points", "shape", 3, 100, false, nullptr}, {"inner", "shape", 0.01, 1, false, nullptr}, {"round", "shape", 0, 0.5, true, nullptr},   // (a polygon's or star's points, a star's inner points, its corners rounded)
    {"start", doc_kinds, 0, 1e6, false, nullptr}, {"duration", doc_kinds, 1e-3, 1e6, false, nullptr},
    {"begin", "film", 0, 1e6, false, nullptr}, {"speed", "film solid", -100, 100, false, nullptr}, {"keyed", "film", 0, 1, false, nullptr},
    {"blend", "film picture text shape", 0, 0, false, "normal add multiply screen"}, {"file", "film picture solid text", 0, 0, false, ""}, {"shape", "solid", 0, 0, false, doc_shapes},
    {"text", "text", 0, 0, false, ""}, {"fill", "text solid shape light", 0, 0, false, ""},
    {"marker", "picture", 0, 0, false, ""},   // (an After Effects file: which of its named stretches it plays, over and over; all: the whole of it)
    {"name", doc_kinds, 0, 0, false, ""},   // (what it is called where things are listed; not said: by what it is)
    {"animation", "solid", 0, 0, false, ""},   // (a 3D file: which of its own movements it plays, round and round, at its speed, from when it comes on)
    // (a light, Filament's own: made a sun, a point or a spot (its file); how strong, 1 as bright as the stage's own
    //  light; how far a point or spot reaches, in units of the stage; how wide a spot's cone is, in degrees; whether
    //  the solid things it lights cast shadows. Its colour is its fill; it shines the way it is turned, ahead of it)
    {"intensity", "light", 0, 100, true, nullptr}, {"falloff", "light", 0.01, 1000, false, nullptr}, {"cone", "light", 1, 89, true, nullptr}, {"shadows", "light", 0, 1, false, nullptr},
    {"shadow_softness", "light", 0, 1, false, nullptr},   // (how soft its shadows are: as from a larger lamp, Filament's PCSS; 0: crisp)
    // (Filament's contact shadows: where things touch, the thin shadow its map misses, found on the picture)
    {"contact_shadows", "light", 0, 1, false, nullptr},
    // (a sun seen in the room behind (behind room): Filament's own disk, its size in degrees across, and the glow around it)
    {"sun_size", "light", 0, 10, true, nullptr},
    {"sun_halo", "light", 0, 100, true, nullptr},
    // (Filament's light channels: a light of group 1 to 7 lights only the solids of that group; 0, as it comes: the solids of group 0 and every other)
    {"light_group", "light solid", 0, 7, false, nullptr},
    {"parent", doc_kinds, 0, 1e9, false, nullptr},   // (the thing it goes with: it is moved, turned and sized as that one is, then as it is itself; 0: none)
    {"space", "film picture text shape", 0, 0, false, "2d 2.5d"},   // (the past repository's three tags: 2d is not in the world and lies by the order alone; 2.5d is a sheet in the world, before or behind by depth; a solid is 3d)
    {"align", "text", 0, 1, false, nullptr},
    // (ThorVG's own scene effects on a thing it draws: a blur, and a shadow of a colour, how far it falls, which way, how
    //  soft, how much of it; lengths in points of the picture at size 1)
    {"blur", "film picture text shape", 0, 1000, true, nullptr}, {"blur_direction", "film picture text shape", 0, 0, false, "both horizontal vertical"}, {"shadow", "picture text shape", 0, 0, false, ""},
    {"shadow_distance", "picture text shape", 0, 1000, true, nullptr}, {"shadow_angle", "picture text shape", -1e6, 1e6, true, nullptr},
    {"shadow_blur", "picture text shape", 0, 1000, true, nullptr}, {"shadow_opacity", "picture text shape", 0, 1, true, nullptr},
    // (and its darks and lights made two colours, how much: ThorVG's tint; or its darks, middles and lights three: its tritone)
    {"glow", "picture text shape", 0, 0, false, ""}, {"glow_blur", "picture text shape", 0, 1000, true, nullptr}, {"glow_opacity", "picture text shape", 0, 1, true, nullptr},
    // (copies of it, each the one before moved, turned about its anchor and sized by as much: After Effects' repeater)
    {"trim", "shape", 0, 1, true, nullptr},   // (how much of its path is cut off the end: After Effects' trim paths, ThorVG's trimpath)
    {"trim_begin", "shape", 0, 1, true, nullptr}, {"trim_offset", "shape", -1e3, 1e3, true, nullptr},   // (and off its beginning; the part left slid along it, in turns)
    // (its path bent, After Effects' path operators, ThorVG's own: pucker & bloat (-1 in, 1 out), zig zag (so high, a share
    //  of its height; so many ridges a side; smooth or sharp), offset paths (out, or in, a share of its height))
    {"bloat", "shape", -1, 1, true, nullptr},
    {"zigzag", "shape", -10, 10, true, nullptr},
    {"zigzag_ridges", "shape", 0, 100, true, nullptr},
    {"zigzag_smooth", "shape", 0, 1, true, nullptr},
    {"offset", "shape", -10, 10, true, nullptr},
    // (and warped as it is drawn, on the GPU: After Effects' wave warp: so high a wave and so long (shares of its height),
    //  running at an angle (degrees), so many waves a second along it)
    {"warp", "shape", -10, 10, true, nullptr},
    {"warp_width", "shape", 0.001, 100, true, nullptr},
    {"warp_angle", "shape", -1e6, 1e6, true, nullptr},
    {"warp_speed", "shape", -1e3, 1e3, true, nullptr},
    // (and its points moved as it is drawn, on the GPU, a shape of one colour only (flat_mesh): Cavalry's bend (degrees
    //  round, across it or up it), After Effects' twist (degrees, most at its middle), Cavalry's pinch (in, or out
    //  when less than 0), skew (degrees, along an axis turned so far), squash and stretch (taller and as much
    //  narrower: its area kept), After Effects' wiggle paths (how far, a share of its height; how fine; how fast))
    {"bend", "shape", -1e4, 1e4, true, nullptr},
    {"bend_direction", "shape", 0, 0, false, "horizontal vertical"},
    {"twist", "shape", -1e5, 1e5, true, nullptr},
    {"pinch", "shape", -1, 1, true, nullptr},
    {"skew", "shape", -89, 89, true, nullptr},
    {"skew_axis", "shape", -1e4, 1e4, true, nullptr},
    {"squash", "shape", -0.95, 10, true, nullptr},
    {"wiggle", "shape", -10, 10, true, nullptr},
    {"wiggle_detail", "shape", 0, 1000, true, nullptr},
    {"wiggle_speed", "shape", -1e3, 1e3, true, nullptr},
    // (and Cavalry's motion stretch: longer the way it moves, and narrower, so much for each unit a second it goes)
    {"stretch", "shape", -10, 10, true, nullptr},
    // (and After Effects' corner pin: how far each corner is moved, shares of its height, up and right; all between follow)
    {"pin_tl_x", "shape", -10, 10, true, nullptr},
    {"pin_tl_y", "shape", -10, 10, true, nullptr},
    {"pin_tr_x", "shape", -10, 10, true, nullptr},
    {"pin_tr_y", "shape", -10, 10, true, nullptr},
    {"pin_bl_x", "shape", -10, 10, true, nullptr},
    {"pin_bl_y", "shape", -10, 10, true, nullptr},
    {"pin_br_x", "shape", -10, 10, true, nullptr},
    {"pin_br_y", "shape", -10, 10, true, nullptr},
    {"copies", "picture text shape", 1, 1000, false, nullptr}, {"copy_x", "picture text shape", -1e4, 1e4, true, nullptr}, {"copy_y", "picture text shape", -1e4, 1e4, true, nullptr},
    {"copy_rotate", "picture text shape", -1e6, 1e6, true, nullptr}, {"copy_size", "picture text shape", 0.01, 100, true, nullptr},
    {"copy_columns", "picture text shape", 0, 100, false, nullptr},   // (so many to a row, then the next row: copy_x along it, copy_y down; 0: one row)
    {"stagger", "picture text shape", 0, 1e3, false, nullptr},   // (how much later the last of its copies moves than the first, in seconds: each as late as its order says)
    {"stagger_from", "picture text shape", 0, 0, false, "start centre end edges"},   // (which of them moves first: the first, the middle one, the last, the outer ones)
    {"copy_fill", "picture text shape", 0, 0, false, ""},   // (its copies coloured from the first colour to the last, in their order: six digits each; one, all in it; none, not coloured each)
    {"copy_fill_way", "picture text shape", 0, 0, false, "ramp cycle"},   // (from the first to the last across them all, or one colour each in turn, round and round)
    {"copy_wide", "picture text shape", 0.01, 100, true, nullptr},   // (how many times as wide its last copy is as its first, those between in step: rows of waves, each with more of them)
    {"copy_z", "picture text shape", -1e4, 1e4, true, nullptr},   // (its copies so far apart towards the eye, in units of the stage: a row going into the picture)
    {"copy_rotate_x", "picture text shape", -1e6, 1e6, true, nullptr},   // (each copy turned so much more than the one before about the level, towards the eye: a fan of sheets in depth)
    {"copy_rotate_y", "picture text shape", -1e6, 1e6, true, nullptr},   // (and about the upright: sheets turning round like pages)
    {"wave_z", "picture text shape", -1e3, 1e3, true, nullptr},   // (and waved so far towards the eye and away, a quarter round after up and down: with wave_y, round and round, a helix)
    {"scatter_z", "picture text shape", 0, 1e4, true, nullptr},   // (scattered so far towards the eye and away)
    {"copy_turn", "picture text shape", -1e5, 1e5, true, nullptr},   // (how far its copies turn in all, spread evenly over them: 360 fills a round; copy_rotate's turn each on top)
    {"tall", "picture text shape", 0, 1e3, true, nullptr},   // (how many times as tall as it is, its anchor staying where it is: a bar growing up from its foot)
    {"copy_tall", "picture text shape", 0, 0, false, ""},   // (and its copies each so many times as tall again, in their order, going round: numbers, a bar chart's data; none: all as tall)
    {"connect", "shape", 0, 1e9, false, nullptr},   // (the thing whose copies its own copies join, one each from one to another: lines between dots; 0: none)
    {"connect_to", "shape", 0, 1e9, false, nullptr},   // (and another: the first's nth copy joined to this one's nth, rungs between two strands; 0: within the first)
    {"connect_reach", "shape", 0, 1e4, true, nullptr},   // (only those nearer than this, in units of the stage, finer the further apart: a plexus; 0: each to the next in their order)
    // (a wave through it and its copies, Cavalry's oscillator: so far across and up (units), so far turned (degrees),
    //  so much larger (a share), so many times a second, each copy so far behind the one before (turns spread over all)
    {"wave_x", "picture text shape", -1e3, 1e3, true, nullptr}, {"wave_y", "picture text shape", -1e3, 1e3, true, nullptr},
    {"wave_rotate", "picture text shape", -1e4, 1e4, true, nullptr}, {"wave_size", "picture text shape", -1, 10, true, nullptr},
    {"wave_speed", "picture text shape", 0, 100, true, nullptr}, {"wave_phase", "picture text shape", -100, 100, true, nullptr},
    // (a field over its copies, Cavalry's falloff: a round place on the stage (its middle, units; how far it reaches); a
    //  copy within it is so much larger and so far turned, most at its middle, none at its edge)
    {"field_x", "picture text shape", -1e4, 1e4, true, nullptr}, {"field_y", "picture text shape", -1e4, 1e4, true, nullptr}, {"field_radius", "picture text shape", 0, 1e4, true, nullptr},
    {"field_size", "picture text shape", -1, 20, true, nullptr}, {"field_rotate", "picture text shape", -1e4, 1e4, true, nullptr},
    {"field_push", "picture text shape", -1e3, 1e3, true, nullptr},   // (and pushed so far away from its middle, in units of the stage; below 0: drawn in)
    {"field_band", "picture text shape", 0, 1e4, true, nullptr},   // (a ring so wide, strongest at its radius from its middle and nothing so far either side: a band of large dots going out; 0: a round, strongest in its middle)
    {"field_z", "picture text shape", -1e4, 1e4, true, nullptr},   // (where its middle is towards the eye: the field is a ball, near and far measured too)
    {"wave_noise", "picture text shape", 0, 1, true, nullptr},   // (how much of the wave is a noise of each copy's own instead of a sine: Cavalry's noise; 1 all noise)
    {"noise_scale", "picture text shape", 0, 1e3, true, nullptr},   // (that noise read where each copy lies, so near ones move alike: how large its blobs are, in units of the stage; 0: each copy its own)
    // (its copies scattered, each its own way by a seed: up to so far across and up (units), so far turned (degrees), so
    //  much larger or smaller (a share); another seed, another scattering)
    {"scatter_x", "picture text shape", 0, 1e4, true, nullptr}, {"scatter_y", "picture text shape", 0, 1e4, true, nullptr},
    {"scatter_rotate", "picture text shape", 0, 1e4, true, nullptr}, {"scatter_size", "picture text shape", 0, 1, true, nullptr},
    {"seed", "picture text shape", 0, 1e6, false, nullptr},
    {"drift_x", "picture text shape", -1e3, 1e3, true, nullptr}, {"drift_y", "picture text shape", -1e3, 1e3, true, nullptr},   // (scattered copies flowing so far a second, units of the stage: past the scattering's edge, back in at the other)
    {"tint", "film picture text shape", 0, 1, true, nullptr}, {"tint_black", "film picture text shape", 0, 0, false, ""}, {"tint_white", "film picture text shape", 0, 0, false, ""},
    {"tritone", "film picture text shape", 0, 1, true, nullptr}, {"tritone_shadow", "film picture text shape", 0, 0, false, ""}, {"tritone_midtone", "film picture text shape", 0, 0, false, ""}, {"tritone_highlight", "film picture text shape", 0, 0, false, ""},   // (how its lines lie one under another, ThorVG's own: 0 at the left, 0.5 about the middle, 1 at the right)
    {"stroke", "text shape", 0, 0, false, ""}, {"stroke_width", "text shape", 0, 1000, false, nullptr},   // (its edge: a colour, and how wide, in points of the picture at size 1)
    // (ThorVG's own: a text's letters and lines so many times as far apart as the font sets them, its slant; an edge's
    //  ends and corners, and dashes along it so long, with gaps as long, begun so far along)
    {"wrap", "text", 0, 0, false, "none character word smart ellipsis"}, {"wrap_at", "text", 0, 1000, false, nullptr},   // (its lines broken in a box so many times as wide as its letters are high: ThorVG's layout and wrap; a text's size is the height of all its lines, so this goes with the letters, not the picture)
    {"split", "text", 0, 0, false, "none letters"},   // (one line of it split into its letters, each a copy of its own: they lie where they are, and move, wave and are coloured each in their order)
    {"letter_spacing", "text", 0, 10, false, nullptr}, {"line_spacing", "text", 0, 10, false, nullptr}, {"italic", "text", 0, 0.5, false, nullptr},
    // (a fill of two colours or more: across, ThorVG's linear gradient, or from the middle out, its radial; turned so far,
    //  in degrees clockwise, 0 from top to bottom; so long, as a share of how far it reaches; past its ends padded,
    //  reflected or repeated)
    {"gradient", "text shape", 0, 0, false, "linear radial"}, {"gradient_angle", "text shape", -1e6, 1e6, false, nullptr},
    {"gradient_scale", "text shape", 0.01, 100, false, nullptr}, {"gradient_spread", "text shape", 0, 0, false, "pad reflect repeat"},
    {"cap", "shape", 0, 0, false, "butt round square"}, {"join", "shape", 0, 0, false, "miter round bevel"},
    {"dash", "shape", 0, 1000, false, nullptr}, {"dash_offset", "shape", -1e4, 1e4, false, nullptr},
    // (Filament's own surface of a solid in a colour: how metal, how rough, how much it reflects, a clear coat over it
    //  and how rough that is, and how much it lights itself: 1 its colour itself on the picture, more brighter still)
    {"metallic", "solid", 0, 1, false, nullptr}, {"roughness", "solid", 0, 1, false, nullptr}, {"reflectance", "solid", 0, 1, false, nullptr},
    // (a solid filled with glass, the kit's own: how much it bends light (index of refraction), how far its colours part,
    //  how much what is behind it swims, how fast a long way through it goes dark, how vivid what is seen through it is)
    {"ior", "solid", 1, 3, false, nullptr}, {"dispersion", "solid", 0, 0.5, false, nullptr}, {"swim", "solid", 0, 1, false, nullptr}, {"dark", "solid", 0, 20, false, nullptr}, {"vivid", "solid", 0, 3, false, nullptr},
    {"clear_coat", "solid", 0, 1, false, nullptr}, {"clear_coat_roughness", "solid", 0, 1, false, nullptr}, {"emissive", "film picture text shape solid", 0, 20, true, nullptr},
    // (Filament's velvet sheen at its rim, in its own colour; its highlights drawn out as on brushed metal, -1 to 1 the other way)
    {"sheen", "solid", 0, 1, false, nullptr},
    // (Filament's shading model: lit, cloth (soft, no sharp highlight) or subsurface (light goes in, as in skin or wax); how thick it is to the light going through)
    {"shading", "solid", 0, 0, false, "lit cloth subsurface"},
    {"thickness", "solid", 0, 1, false, nullptr},
    {"anisotropy", "solid", -1, 1, false, nullptr},
    // (the stage's, thing 0: the whole picture, Filament's own. Its grade, on every picture laid: a tone mapper, an
    //  exposure in stops, contrast, saturation, vibrance, how warm; its glow round bright parts and its lens flare, a
    //  darkening at the corners and a fog in a colour, on the stage's own view; the room's light, how bright and turned
    //  how far, and the colour behind everything)
    // (a camera's lens, Filament's own: drawn without depth (orthographic: as large far as near, as an axonometric
    //  drawing), and shifted across and up, in shares of the picture (a shift lens: uprights stay upright))
    {"projection", "camera", 0, 0, false, "perspective orthographic"}, {"shift_x", "camera", -2, 2, true, nullptr}, {"shift_y", "camera", -2, 2, true, nullptr},
    {"tone_mapper", "stage", 0, 0, false, "linear aces filmic agx pbr_neutral"}, {"exposure", "stage", -10, 10, false, nullptr},
    {"contrast", "stage", 0, 2, false, nullptr}, {"saturation", "stage", 0, 2, false, nullptr}, {"vibrance", "stage", 0, 2, false, nullptr}, {"temperature", "stage", -1, 1, false, nullptr},
    {"bloom", "stage", 0, 1, false, nullptr}, {"lens_flare", "stage", 0, 1, false, nullptr}, {"vignette", "stage", 0, 1, false, nullptr}, {"vignette_color", "stage", 0, 0, false, ""},
    {"fog", "stage", 0, 1, false, nullptr}, {"fog_color", "stage", 0, 0, false, ""},
    // (Filament's screen effects on solid things: shade where they meet and in their folds (SSAO), and what is near mirrored in them (SSR))
    {"ambient_occlusion", "stage", 0, 1, false, nullptr},
    {"reflections", "stage", 0, 1, false, nullptr},
    {"slope", "stage", 0, 4, false, nullptr},
    {"offset", "stage", -1, 1, false, nullptr},
    {"power", "stage", 0.1, 4, false, nullptr},
    // (Filament's shadows, mid-tones and highlights each tinted their colour, six digits: not said, white, as they are)
    {"shadows_color", "stage", 0, 0, false, ""},
    {"midtones_color", "stage", 0, 0, false, ""},
    {"highlights_color", "stage", 0, 0, false, ""},
    // (Filament's ASC CDL grade, the same on red, green and blue: each colour times slope, plus offset, to the power)
    {"anti_aliasing", "stage", 0, 0, false, "fxaa taa msaa none"},   // (how its edges are smoothed: Filament's FXAA as it came, its temporal one (steadier on fine edges in motion), or not)
    {"room", "stage", 0, 0, false, "small_studio studio_small_02_2k lightroom venetian_crossroads_2k graffiti_shelter_2k parking_garage_2k pillars_2k"},   // (which room lights it: the rooms made ready in out/)
    {"environment", "stage", 0, 10, false, nullptr}, {"environment_rotate", "stage", -1e6, 1e6, false, nullptr}, {"background", "stage", 0, 0, false, ""},
    {"behind", "stage", 0, 0, false, "colour room"}};   // (what is behind everything: the background's colour, or the room itself)
// The cards a maker sees a thing's values on, besides where it is and its fill: a title and the values on it, in the
// order shown (a thing is shown those of them its kind has). A value the window has no other seat for goes here.
struct DocCard { const char* title; const char* names; const char* when = nullptr; const char* is = nullptr; };   // (shown only while `when` is `is`; "!...": is not; "~...": has it in it)
// How a value is shown to a hand (a shell draws its knob by this, nothing of its own): its look (count: a whole number;
// onoff; angle; stage: a length of the stage, shown in points of the picture; down: one up the stage, shown down it as the picture counts; share: 0 to 1, shown in hundredths;
// colour, colours, numbers: words of those; words: any, typed; file: a file, given from a shelf; thing: another thing; seed; none: as its range says), the unit it is read
// in, a name in plain words, the span its knob covers (the value itself may go further, as its range says) and
// the name of the fold it lies in on its card, if it is folded away (named by what the fold holds; a card's first
// row holds eight at most, as on Ableton's Push).
struct DocShown { const char* name; const char* look; const char* unit; const char* label; double low, high; const char* fold; };
static const DocShown doc_shown[] = {
    {"text", "words", "", "Text", 0, 0, nullptr},
    {"file", "file", "", "Font", 0, 0, nullptr},
    {"size", "", "px", "Size", 0, 2, nullptr},
    {"align", "share", "%", "Align", 0, 1, nullptr},
    {"fill", "colours", "", "Fill", 0, 0, nullptr},
    {"stroke", "colour", "", "Stroke", 0, 0, nullptr},
    {"stroke_width", "", "px", "Stroke width", 0, 16, nullptr},
    {"split", "", "", "Split", 0, 0, nullptr},
    {"letter_spacing", "", "×", "Tracking", 0, 3, nullptr},
    {"line_spacing", "", "×", "Line gap", 0, 3, nullptr},
    {"italic", "share", "%", "Slant", 0, 0.5, nullptr},
    {"wrap", "", "", "Wrap", 0, 0, nullptr},
    {"wrap_at", "", "×", "Wrap at", 0, 40, nullptr},
    {"gradient", "", "", "Gradient", 0, 0, nullptr},
    {"gradient_angle", "angle", "°", "Angle", -180, 180, nullptr},
    {"round", "share", "%", "Round", 0, 0.5, nullptr},
    {"trim", "share", "%", "Trim end", 0, 1, nullptr},
    {"trim_begin", "share", "%", "Trim start", 0, 1, nullptr},
    {"trim_offset", "share", "%", "Trim offset", -1, 1, nullptr},
    {"bloat", "", "", "Bloat", -1, 1, nullptr},
    {"offset", "", "", "Offset", -0.5, 0.5, nullptr},
    {"zigzag", "", "", "Zig zag", -0.3, 0.3, nullptr},
    {"zigzag_ridges", "count", "", "Ridges", 0, 20, "Zig zag"},
    {"zigzag_smooth", "onoff", "", "Smooth", 0, 1, "Zig zag"},
    {"warp", "", "", "Wave", -0.5, 0.5, nullptr},
    {"warp_width", "", "", "Wave length", 0.05, 2, "Wave"},
    {"warp_angle", "angle", "°", "Wave angle", -180, 180, "Wave"},
    {"warp_speed", "", "/s", "Wave speed", -5, 5, "Wave"},
    {"bend", "angle", "°", "Bend", -360, 360, nullptr},
    {"bend_direction", "", "", "Bend way", 0, 0, "Bend"},
    {"twist", "angle", "°", "Twist", -720, 720, nullptr},
    {"pinch", "", "", "Pinch", -1, 1, nullptr},
    {"skew", "angle", "°", "Skew", -80, 80, nullptr},
    {"skew_axis", "angle", "°", "Skew axis", -180, 180, "Skew"},
    {"squash", "", "", "Squash", -0.9, 2, nullptr},
    {"wiggle", "", "", "Wiggle", 0, 0.3, nullptr},
    {"wiggle_detail", "", "", "Detail", 0.5, 20, "Wiggle"},
    {"wiggle_speed", "", "/s", "Wiggle speed", 0, 10, "Wiggle"},
    {"stretch", "", "", "Stretch", 0, 0.5, nullptr},
    {"pin_tl_x", "", "", "Pin TL x", -0.5, 0.5, "Corner pin"}, {"pin_tl_y", "", "", "Pin TL y", -0.5, 0.5, "Corner pin"},
    {"pin_tr_x", "", "", "Pin TR x", -0.5, 0.5, "Corner pin"}, {"pin_tr_y", "", "", "Pin TR y", -0.5, 0.5, "Corner pin"},
    {"pin_bl_x", "", "", "Pin BL x", -0.5, 0.5, "Corner pin"}, {"pin_bl_y", "", "", "Pin BL y", -0.5, 0.5, "Corner pin"},
    {"pin_br_x", "", "", "Pin BR x", -0.5, 0.5, "Corner pin"}, {"pin_br_y", "", "", "Pin BR y", -0.5, 0.5, "Corner pin"},
    {"gradient_scale", "", "×", "Stretch", 0.1, 4, nullptr},
    {"gradient_spread", "", "", "Past end", 0, 0, nullptr},
    {"copies", "count", "", "How many", 1, 200, nullptr},
    {"copy_columns", "count", "", "Per row", 0, 40, nullptr},
    {"copy_x", "stage", "px", "Gap X", -2, 2, nullptr},
    {"copy_y", "down", "px", "Gap Y", -2, 2, nullptr},
    {"copy_turn", "angle", "°", "Turn all", 0, 720, nullptr},
    {"stagger", "", "s", "Delay", 0, 3, nullptr},
    {"copy_fill", "colours", "", "Colours", 0, 0, nullptr},
    {"copy_z", "stage", "px", "Gap deep", -2, 2, "Each copy"},
    {"copy_rotate_x", "angle", "°", "Tilt", -180, 180, "Each copy"},
    {"copy_rotate_y", "angle", "°", "Swing", -180, 180, "Each copy"},
    {"copy_rotate", "angle", "°", "Turn", -180, 180, "Each copy"},
    {"copy_size", "", "×", "Grow", 0.5, 1.5, "Each copy"},
    {"copy_wide", "", "×", "Wider", 0.2, 5, "Each copy"},
    {"tall", "", "×", "Height", 0, 3, "Each copy"},
    {"copy_tall", "numbers", "", "Heights", 0, 0, "Each copy"},
    {"stagger_from", "", "", "From", 0, 0, nullptr},
    {"copy_fill_way", "", "", "Order", 0, 0, "Each copy"},
    {"wave_y", "stage", "px", "Up/down", -1, 1, nullptr},
    {"wave_x", "stage", "px", "Sideways", -1, 1, nullptr},
    {"wave_z", "stage", "px", "In/out", -1, 1, nullptr},
    {"wave_rotate", "angle", "°", "Swing", -90, 90, nullptr},
    {"wave_size", "share", "%", "Pulse", -1, 2, nullptr},
    {"wave_speed", "", "/s", "Speed", 0, 4, nullptr},
    {"wave_phase", "", "waves", "Ripple", -4, 4, nullptr},
    {"wave_noise", "share", "%", "Random", 0, 1, "Noise"},
    {"noise_scale", "stage", "px", "Blob", 0, 3, "Noise"},
    {"field_x", "stage", "px", "At X", -3, 3, nullptr},
    {"field_y", "down", "px", "At Y", -2, 2, nullptr},
    {"field_radius", "stage", "px", "Size", 0, 4, nullptr},
    {"field_size", "share", "%", "Grow", -1, 5, nullptr},
    {"field_push", "stage", "px", "Push", -2, 2, nullptr},
    {"field_band", "stage", "px", "Ring", 0, 2, nullptr},
    {"field_z", "stage", "px", "At Z", -2, 2, nullptr},
    {"field_rotate", "angle", "°", "Turn", -180, 180, nullptr},
    {"scatter_x", "stage", "px", "Spread X", 0, 4, nullptr},
    {"scatter_y", "stage", "px", "Spread Y", 0, 3, nullptr},
    {"scatter_z", "stage", "px", "Spread Z", 0, 3, nullptr},
    {"scatter_rotate", "angle", "°", "Turn", 0, 180, nullptr},
    {"scatter_size", "share", "%", "Size", 0, 1, nullptr},
    {"seed", "seed", "", "Seed", 0, 9999, nullptr},
    {"drift_x", "stage", "px/s", "Flow X", -2, 2, nullptr},
    {"drift_y", "down", "px/s", "Flow Y", -2, 2, nullptr},
    {"emissive", "", "×", "Light up", 0, 10, nullptr},
    {"marker", "", "", "Part", 0, 0, nullptr},
    {"animation", "", "", "Movement", 0, 0, nullptr},
    {"speed", "", "×", "Speed", -4, 4, nullptr},
    {"clip_matte", "", "", "Cut by", 0, 0, nullptr},
    {"clip_invert", "onoff", "", "Invert", 0, 1, nullptr},
    {"clip_feather", "", "px", "Feather", 0, 100, nullptr},
    {"connect", "thing", "", "Join", 0, 0, nullptr},
    {"x", "", "px", "Position X", -4, 4, nullptr}, {"y", "", "px", "Position Y", -3, 3, nullptr}, {"z", "", "px", "Position Z", -6, 6, nullptr},   // (in points of the picture from its top left: motolii_reading)
    {"scale_x", "share", "%", "Scale X", 0, 4, nullptr}, {"scale_y", "share", "%", "Scale Y", 0, 4, nullptr}, {"scale_z", "share", "%", "Scale Z", 0, 4, nullptr},
    {"rotate", "angle", "°", "Rotation", -180, 180, nullptr}, {"rotate_x", "angle", "°", "X Rotation", -180, 180, nullptr}, {"rotate_y", "angle", "°", "Y Rotation", -180, 180, nullptr},
    {"opacity", "share", "%", "Opacity", 0, 1, nullptr},
    {"blend", "", "", "Blend", 0, 0, nullptr}, {"clip", "onoff", "", "Clip to below", 0, 1, nullptr},
    {"distance", "", "px", "Distance", 0.05, 4, nullptr}, {"zoom", "share", "%", "Zoom", 0.1, 4, nullptr},
    {"anchor_x", "", "%", "Anchor X", -1, 1, "Anchor"}, {"anchor_y", "", "%", "Anchor Y", -1, 1, "Anchor"}, {"anchor_z", "", "%", "Anchor Z", -1, 1, "Anchor"},
    {"parent", "thing", "", "Parent", 0, 0, nullptr},   // (the thing it goes with: shown by its name)
    {"connect_to", "thing", "", "With", 0, 0, nullptr},
    {"follow", "thing", "", "Follow", 0, 0, nullptr},
    {"connect_reach", "stage", "px", "Reach", 0, 2, nullptr},
    {"cap", "", "", "Ends", 0, 0, nullptr},
    {"join", "", "", "Corners", 0, 0, nullptr},
    {"dash", "", "px", "Dash", 0, 100, nullptr},
    {"dash_offset", "", "px", "Offset", -100, 100, nullptr},
    {"metallic", "share", "%", "Metal", 0, 1, nullptr},
    {"roughness", "share", "%", "Rough", 0, 1, nullptr},
    {"reflectance", "share", "%", "Shine", 0, 1, nullptr},
    {"clear_coat", "share", "%", "Varnish", 0, 1, nullptr},
    {"clear_coat_roughness", "share", "%", "Haze", 0, 1, nullptr},
    {"sheen", "share", "%", "Sheen", 0, 1, nullptr},
    {"shading", "", "", "Model", 0, 0, nullptr},
    {"thickness", "share", "%", "Thick", 0, 1, nullptr},
    {"anisotropy", "", "", "Brushed", -1, 1, nullptr},
    {"ior", "", "", "Bend", 1, 3, nullptr},
    {"dispersion", "share", "%", "Rainbow", 0, 0.5, nullptr},
    {"swim", "share", "%", "Wobble", 0, 1, nullptr},
    {"dark", "", "", "Darken", 0, 20, nullptr},
    {"vivid", "", "×", "Colour", 0, 3, nullptr},
    {"intensity", "", "", "Bright", 0, 100, nullptr},
    {"falloff", "stage", "px", "Reach", 0, 20, nullptr},
    {"cone", "angle", "°", "Cone", 1, 89, nullptr},
    {"shadows", "onoff", "", "Shadows", 0, 1, nullptr},
    {"shadow_softness", "share", "%", "Soft", 0, 1, nullptr},
    {"sun_size", "angle", "°", "Disk", 0, 10, nullptr},
    {"sun_halo", "", "×", "Halo", 0, 100, nullptr},
    {"light_group", "", "", "Group", 0, 7, nullptr},
    {"contact_shadows", "onoff", "", "Contact", 0, 1, nullptr},
    {"projection", "", "", "Lens", 0, 0, nullptr},
    {"shift_x", "", "", "Shift X", -2, 2, nullptr},
    {"shift_y", "", "", "Shift Y", -2, 2, nullptr},
    {"tone_mapper", "", "", "Look", 0, 0, nullptr},
    {"exposure", "", "stops", "Bright", -5, 5, nullptr},
    {"contrast", "", "×", "Contrast", 0, 2, nullptr},
    {"saturation", "", "×", "Colour", 0, 2, nullptr},
    {"vibrance", "", "×", "Vibrance", 0, 2, nullptr},
    {"temperature", "", "", "Warmth", -1, 1, nullptr},
    {"bloom", "share", "%", "Bloom", 0, 1, nullptr},
    {"physics", "", "", "Physics", 0, 0, nullptr},
    {"bounce", "share", "%", "Bounce", 0, 1, nullptr},
    {"friction", "", "", "Friction", 0, 2, nullptr},
    {"gravity", "", "", "Gravity", 0, 30, nullptr},
    {"ambient_occlusion", "share", "%", "AO", 0, 1, nullptr},
    {"reflections", "share", "%", "Mirror", 0, 1, nullptr},
    {"vignette", "share", "%", "Vignette", 0, 1, nullptr},
    {"vignette_color", "colour", "", "Vignette", 0, 0, "Tints"},
    {"fog", "share", "%", "Fog", 0, 1, nullptr},
    {"fog_color", "colour", "", "Fog", 0, 0, "Tints"},
    {"lens_flare", "share", "%", "Flare", 0, 1, nullptr},
    {"anti_aliasing", "", "", "Smooth", 0, 0, nullptr},
    {"slope", "", "", "Slope", 0, 4, nullptr},
    {"offset", "", "", "Offset", -1, 1, nullptr},
    {"power", "", "", "Power", 0.1, 4, nullptr},
    {"shadows_color", "colour", "", "Shadows", 0, 0, nullptr},
    {"midtones_color", "colour", "", "Mids", 0, 0, nullptr},
    {"highlights_color", "colour", "", "Highs", 0, 0, nullptr},
    {"room", "", "", "Room", 0, 0, nullptr},
    {"environment", "", "×", "Bright", 0, 10, nullptr},
    {"environment_rotate", "angle", "°", "Turn", -180, 180, nullptr},
    {"behind", "", "", "Behind", 0, 0, nullptr},
    {"background", "colour", "", "Colour", 0, 0, nullptr},
};
// A thing's script (script.h): the knobs its file's head declares, as values of the thing named js_ and the knob's name,
// and its code, made once a file and again when the file changes (a script written and saved shows at once).
struct DocScript {
    long long stamp = -1;
    std::chrono::steady_clock::time_point looked{};   // (when the file was last asked whether it changed: a few times a second at most)
    std::vector<ScriptKnob> knobs;
    std::vector<std::string> names;
    std::vector<DocValue> values;
    std::vector<DocShown> shown;
    std::unique_ptr<Script> run;
};
static std::map<std::string, DocScript>& doc_scripts() { static std::map<std::string, DocScript> all; return all; }
static int& doc_scripts_made() { static int made = 0; return made; }
static const DocShown* doc_shown_of(const char* name) {
    for (const DocShown& one : doc_shown) if (!strcmp(one.name, name)) return &one;
    for (const auto& [file, script] : doc_scripts())
        for (const DocShown& one : script.shown) if (!strcmp(one.name, name)) return &one;
    return nullptr;
}
// A whole-screen effect's knob in the document: its file's name before the knob's, as several lie on one stage and two
// may each have a knob of one name (Posterize's gamma and Boxinator's are not one value).
static std::string doc_screen_knob(const std::string& file, const std::string& knob) { return "js_" + std::filesystem::path(file).stem().string() + "_" + knob; }
static DocScript* doc_script_of(const std::string& file) {
    if (file.empty() || file == "none") return nullptr;
    const bool screen = std::filesystem::path(file).extension() == ".wgsl";   // (a whole-screen effect's knobs: named by the effect too)
    DocScript& s = doc_scripts()[file];
    const auto now = std::chrono::steady_clock::now();
    if (s.stamp >= 0 && now - s.looked < std::chrono::milliseconds(250)) return &s;
    s.looked = now;
    std::error_code wrong;
    const long long stamp = (long long)std::filesystem::last_write_time(file, wrong).time_since_epoch().count();
    if (wrong) return s.stamp >= 0 ? &s : nullptr;
    if (s.stamp != stamp) {
        s = DocScript{};
        s.stamp = stamp, s.looked = now, s.knobs = script_knobs(file);
        doc_scripts_made()++;   // (what was worked out from the old one is forgotten)
        for (const ScriptKnob& k : s.knobs) s.names.push_back(screen ? doc_screen_knob(file, k.name) : "js_" + k.name);
        for (size_t i = 0; i < s.knobs.size(); ++i) {   // (the names are all made first: what points into them stays put)
            const ScriptKnob& k = s.knobs[i];
            s.values.push_back(DocValue{s.names[i].c_str(), "picture text shape stage", k.kind == "select" ? 0.0 : k.low, k.kind == "select" ? 0.0 : k.high, k.kind == "track", k.kind == "select" ? k.words.c_str() : nullptr});
            s.shown.push_back(DocShown{s.names[i].c_str(), k.kind == "check" ? "onoff" : "", "", k.label.c_str(), k.low, k.high, nullptr});
        }
        if (std::filesystem::path(file).extension() != ".wgsl") s.run = std::make_unique<Script>(file);   // (a .wgsl is drawn, not run: screen.h)
    }
    return &s;
}
// A thing's scripts: its one (a path may have spaces), or the stage's effects, one over another (paths with spaces between).
static std::string doc_word(Doc& d, int thing, const char* name);
static std::vector<DocScript*> doc_scripts_on(Doc& d, int thing) {
    const std::string said = doc_word(d, thing, "script");
    std::vector<DocScript*> all;
    if (thing != 0) {
        if (DocScript* sc = doc_script_of(said)) all.push_back(sc);
        return all;
    }
    std::istringstream each(said);
    for (std::string file; each >> file;) if (DocScript* sc = doc_script_of(file)) all.push_back(sc);
    return all;
}
static const DocCard doc_cards[] = {
    {"Text", "text file size align", "kind", "text"},   // (After Effects' Source Text and Character, Cavalry's Text Shape: what it says, in which font, how large, which way it lines up)
    {"Fill", "fill stroke stroke_width"},   // (After Effects' Fill and Stroke, Cavalry's Fill and Stroke: a colour, colours down it, or the material glass; its edge and how wide)
    {"Transform", "space parent anchor_x anchor_y anchor_z x y z scale_x scale_y scale_z rotate_x rotate_y rotate opacity", "kind", "!camera"},   // (After Effects' Transform group: where it is, how large, how turned, how seen; each its own value)
    {"Camera", "parent x y z rotate_x rotate_y rotate distance zoom", "kind", "camera"},   // (After Effects' camera: what it looks at, where it is and how turned, how far its eye is, its zoom)
    {"Blending", "blend clip"},   // (After Effects' layer mode and its track matte, in its simplest: drawn over what lies below as it says, or only inside the one before it)
    {"Type", "split letter_spacing line_spacing italic wrap wrap_at"},
    {"Gradient", "gradient gradient_angle gradient_scale gradient_spread", "fill", "~ "},
    {"Copies", "copies copy_columns copy_x copy_y copy_rotate copy_rotate_x copy_rotate_y copy_turn copy_size copy_wide copy_z tall copy_tall stagger stagger_from copy_fill copy_fill_way"},
    {"Wave", "wave_x wave_y wave_z wave_rotate wave_size wave_speed wave_phase wave_noise noise_scale"},
    {"Field", "field_x field_y field_z field_radius field_band field_size field_rotate field_push"},
    {"Scatter", "scatter_x scatter_y scatter_z scatter_rotate scatter_size seed drift_x drift_y"},
    {"Shine", "emissive", "kind", "!solid"},   // (a flat thing lit from within: past 1, the stage's bloom takes it up where it lies in the stage's own view; laid over a solid thing, a picture of its own is clear round it and keeps no halo: ThorVG's glow does that)
    {"Play", "marker animation speed"},
    {"Matte", "clip_matte clip_invert clip_feather", "clip", "1"},
    {"Connect", "connect connect_to connect_reach"},
    {"Line", "cap join dash dash_offset"},
    {"Path", "round bloat offset zigzag zigzag_ridges zigzag_smooth", "kind", "shape"},   // (its path made anew: After Effects' shape operators, ThorVG's own)
    {"Deform", "warp bend twist pinch skew squash wiggle stretch bend_direction pin_tl_x pin_tl_y pin_tr_x pin_tr_y pin_bl_x pin_bl_y pin_br_x pin_br_y warp_width warp_angle warp_speed skew_axis wiggle_detail wiggle_speed", "kind", "shape"},   // (its points moved as it is drawn: Cavalry's deformers)
    {"Surface", "shading metallic roughness reflectance clear_coat clear_coat_roughness sheen anisotropy thickness emissive", "fill", "!glass"},
    {"Glass", "ior dispersion swim dark vivid", "fill", "glass"},
    {"Lit", "light_group", "kind", "solid"},   // (which lights reach it)
    {"Light", "intensity falloff cone shadows shadow_softness contact_shadows light_group sun_size sun_halo", "kind", "light"},
    {"Projection", "projection shift_x shift_y"},
    {"Grade", "tone_mapper exposure contrast saturation vibrance temperature slope offset power shadows_color midtones_color highlights_color"},
    {"Lens", "bloom ambient_occlusion reflections lens_flare vignette vignette_color fog fog_color anti_aliasing"},
    {"Room", "room environment environment_rotate behind background"},
    {"Physics", "physics bounce friction gravity"},
    {"Trim Paths", "trim trim_begin trim_offset", "trim", "+"},   // (After Effects' shape operator: added from the shelf)
    {"Gaussian Blur", "blur blur_direction", "blur", "+"},   // (the effects (doc_effects), each its card while it is on, as After Effects' Effect Controls)
    {"Drop Shadow", "shadow shadow_distance shadow_angle shadow_blur shadow_opacity", "shadow", "+"},
    {"Glow", "glow glow_blur glow_opacity", "glow", "+"},
    {"Duotone", "tint tint_black tint_white", "tint", "+"},
    {"Gradient Map", "tritone tritone_shadow tritone_midtone tritone_highlight", "tritone", "+"},
};
static bool doc_among(const std::string& words, const std::string& one) { return (" " + words + " ").find(" " + one + " ") != std::string::npos; }

// Rows of the document: `each` is told every row of what is asked (in SQLite's own words).
static bool doc_rows(Doc& d, char* ask, const std::function<void(sqlite3_stmt*)>& each = nullptr) {
    sqlite3_stmt* rows = nullptr;
    bool ok = sqlite3_prepare_v2(d.file, ask, -1, &rows, nullptr) == SQLITE_OK;
    while (ok && sqlite3_step(rows) == SQLITE_ROW)
        if (each) each(rows);
    sqlite3_finalize(rows);
    sqlite3_free(ask);
    return ok;
}
static void doc_forget(Doc& d) {
    d.phys_stale = true, d.scripted.clear();   // (the world made again only if what it is made of changed: doc_physics)
    d.moving.clear(), d.words.clear(), d.keyed.clear();
    d.named.clear();
    d.laid.clear();
}
// The things in the order they lie, the one furthest back first: by `order` where it is said, else as they were made.
static const std::vector<int>& doc_laid(Doc& d) {
    if (d.laid.empty())
        doc_rows(d, sqlite3_mprintf("select f.thing from fact f where f.name = 'kind' order by coalesce((select o.value from fact o where o.thing = f.thing and o.name = 'order' and o.at = 0), f.thing), f.thing"),
                 [&](sqlite3_stmt* row) { d.laid.push_back(sqlite3_column_int(row, 0)); });
    return d.laid;
}

// A way's expression made ready for SQLite to work out (nullptr: SQLite does not take it; d.trouble says why).
static sqlite3_stmt* doc_goes(Doc& d, const std::string& expression) {
    auto ready = d.goes.find(expression);
    if (ready != d.goes.end()) return ready->second;
    sqlite3_stmt* made = nullptr;
    char* asked = sqlite3_mprintf("with v(u, t, d, a, b, p1, p2, p3, p4) as (select ?, ?, ?, ?, ?, ?, ?, ?, ?) select (%s) from v", expression.c_str());
    if (sqlite3_prepare_v2(d.file, asked, -1, &made, nullptr) != SQLITE_OK) d.trouble = sqlite3_errmsg(d.file), made = nullptr;
    sqlite3_free(asked);
    return made ? d.goes[expression] = made : nullptr;
}
static double doc_went(sqlite3_stmt* way, double u, double t, double length, double a, double b, const double* p) {
    const double told[] = {u, t, length, a, b, p[0], p[1], p[2], p[3]};
    for (int i = 0; i < 9; ++i) sqlite3_bind_double(way, i + 1, told[i]);
    const double is = sqlite3_step(way) == SQLITE_ROW ? sqlite3_column_double(way, 0) : a;
    sqlite3_reset(way);
    return is;
}
static sqlite3_stmt* doc_way_named(Doc& d, const std::string& name) {
    auto known = d.named.find(name);
    if (known != d.named.end()) return known->second;
    sqlite3_stmt* way = nullptr;
    doc_rows(d, sqlite3_mprintf("select goes from way where name = %Q", name.c_str()), [&](sqlite3_stmt* row) { way = doc_goes(d, (const char*)sqlite3_column_text(row, 0)); });
    if (!way)   // (not defined here: one of the ways by name, ways.h)
        for (const auto& one : named_ways) if (name == one.name) way = doc_goes(d, std::string("a + (b - a) * (") + one.goes + ")");
    return d.named[name] = way;
}
// A stretch that goes a way defined in the document (`own`: its name, then its numbers): the value at `u` of the way
// through, `t` seconds in, between a key of `a` and a key of `b`, `length` seconds apart (no such way: `a`).
static double doc_went_by(Doc& d, const std::string& own, double u, double t, double length, double a, double b) {
    std::istringstream said(own);
    std::string way;
    double p[4] = {0, 0, 0, 0};
    said >> way;
    for (double& one : p) said >> one;
    sqlite3_stmt* goes = doc_way_named(d, way);
    return goes ? doc_went(goes, u, t, length, a, b, p) : a;
}

// A value at a moment: `otherwise` where the document does not say; where it has keys, the value the stretch it is in
// gives (before the first key and after the last: theirs). A stretch goes the way said for it (not said: Linear).
static double doc_number(Doc& d, int thing, const char* name, double at, double otherwise) {
    if (d.held.thing == thing)   // (what a hand is carrying shows in place of the document's own)
        if (const auto tried = d.held.tried.find(name); tried != d.held.tried.end()) return tried->second;
    auto found = d.moving.find({thing, name});
    if (found == d.moving.end()) {
        Doc::Keys keys;
        doc_rows(d, sqlite3_mprintf("select at, value, coalesce(way, '') from fact where thing = %d and name = %Q order by at", thing, name), [&](sqlite3_stmt* row) {
            keys.at.push_back(sqlite3_column_double(row, 0));
            keys.value.push_back(sqlite3_column_double(row, 1));
            const std::string way = (const char*)sqlite3_column_text(row, 2);
            Interp kind;
            const bool of_the_nine = interp_said(way.empty() ? "Linear" : way, &kind) == 0;   // (not said: Linear, as a new key is in the past repository)
            keys.interp.push_back(kind);
            keys.own.push_back(of_the_nine ? "" : way);
        });
        found = d.moving.emplace(std::make_pair(thing, std::string(name)), keys).first;
    }
    const Doc::Keys& k = found->second;
    if (k.at.empty()) return otherwise;
    if (k.at.size() == 1 || at <= k.at.front()) return k.value.front();
    if (at >= k.at.back()) return k.value.back();
    const size_t i = (size_t)(std::upper_bound(k.at.begin(), k.at.end(), at) - k.at.begin()) - 1;   // (the stretch it is in begins at key i)
    const double u = (at - k.at[i]) / (k.at[i + 1] - k.at[i]);
    if (k.own[i].empty()) return k.value[i] + (k.value[i + 1] - k.value[i]) * interp_ease(k.interp[i], u);
    return doc_went_by(d, k.own[i], u, at - k.at[i], k.at[i + 1] - k.at[i], k.value[i], k.value[i + 1]);
}
static std::string doc_word(Doc& d, int thing, const char* name) {
    if (thing == 0 && !strcmp(name, "kind")) return "stage";   // (thing 0 is the stage: its values are the whole picture's)
    const auto known = d.words.find({thing, name});   // (asked again and again between changes: the document asked once)
    if (known != d.words.end()) return known->second;
    std::string is;
    doc_rows(d, sqlite3_mprintf("select value from fact where thing = %d and name = %Q", thing, name), [&](sqlite3_stmt* row) { is = (const char*)sqlite3_column_text(row, 0); });
    d.words.emplace(std::make_pair(thing, std::string(name)), is);
    return is;
}

// Every look of a thing as it is on the stage (a sheet's one; a kit shape's, as many as it is drawn by; every face of a 3D file).
static void doc_looks(Stage& s, const Doc::Shown& t, const std::function<void(MaterialInstance*)>& each) {
    auto& faces = s.engine->getRenderableManager();
    if (t.film) each(t.film->look);
    if (t.drawn) each(t.drawn->look);
    for (const Stage::Made& made : s.made)
        if (t.shape && made.of == t.shape) each(faces.getMaterialInstanceAt(faces.getInstance(made.thing), 0));
    for (size_t i = 0; t.solid && i < t.solid->made->getRenderableEntityCount(); ++i) {
        const auto one = faces.getInstance(t.solid->made->getRenderableEntities()[i]);
        for (size_t part = 0; part < faces.getPrimitiveCount(one); ++part) each(faces.getMaterialInstanceAt(one, part));
    }
}
// Each drawn part of a thing (Filament's renderables), and the scene it is in (its run's: run 0 or not yet put, the stage's own).
static void doc_entities(Stage& s, const Doc::Shown& t, const std::function<void(utils::Entity)>& each) {
    const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.shape;
    for (const Stage::Made& made : s.made)
        if (m && made.of == m && !made.matte) each(made.thing);
    for (size_t i = 0; t.solid && i < t.solid->made->getRenderableEntityCount(); ++i) each(t.solid->made->getRenderableEntities()[i]);
}
static void doc_parts(Stage& s, const Doc::Shown& t, const std::function<void(RenderableManager&, RenderableManager::Instance)>& each) {
    auto& faces = s.engine->getRenderableManager();
    doc_entities(s, t, [&](utils::Entity e) { each(faces, faces.getInstance(e)); });
}
static bool doc_covers(Doc& d, int thing, double now, double* box);
static bool doc_half(Doc& d, int thing, float2* half, float* deep = nullptr);
// Where a thing is at a moment, as a matrix: moved, turned about z, y and x, made its size, all about its anchor (a
// point of its own box: 0 its middle, 1 its edge on that side; the past repository's `anchor`, keyed as it was).
static math::mat4 doc_place(Doc& d, int thing, double now, int deeper = 0, int copy = 0, bool member = false);
static double doc_point(Doc& d, int thing, double now);
static double doc_seconds(Doc& d);
// A thing's copies given back its own look, their own looks done away with (no longer coloured each, or taken off).
static void doc_copy_looks_off(Doc& d, const Mesh* m, MaterialInstance* main) {
    Stage& s = *d.stage;
    auto& faces = s.engine->getRenderableManager();
    for (const Stage::Made& made : s.made) {
        if (made.of != m) continue;
        const auto found = d.copyLooks.find(made.thing.getId());
        if (found == d.copyLooks.end()) continue;
        if (main) faces.setMaterialInstanceAt(faces.getInstance(made.thing), 0, main);
        s.engine->destroy(found->second);
        d.copyLooks.erase(found);
    }
}
// How many members a thing has in their order: a text split into letters, a letter each; else its copies.
static int doc_members(Doc& d, int thing) {
    const auto t = d.shown.find(thing);
    if (t != d.shown.end() && t->second.drawn && !t->second.drawn->letters.empty()) return (int)t->second.drawn->letters.size();
    return std::max(1, (int)doc_number(d, thing, "copies", 0, 1));
}
// How much later a thing's nth copy moves than it is said to (the past repository's order law, GSAP's stagger: the
// stagger spread over them by their order: from the start, from the centre out, from the end, from the edges in).
static double doc_delay(Doc& d, int thing, int nth) {
    const double stagger = doc_number(d, thing, "stagger", 0, 0);
    const int n = doc_members(d, thing);
    if (stagger <= 0.0 || n < 2) return 0.0;
    const double along = (double)nth / (double)(n - 1), centre = std::fabs(along - 0.5) * 2.0;
    const std::string from = doc_word(d, thing, "stagger_from");
    return stagger * (from == "centre" ? centre : from == "end" ? 1.0 - along : from == "edges" ? 1.0 - centre : along);
}
// Where the thing it goes with is (its parent's place; none, or one not there any more: as the stage is).
static math::mat4 doc_above(Doc& d, int thing, double now, int deeper = 0) {
    const int parent = (int)doc_number(d, thing, "parent", 0, 0);
    return parent && d.shown.count(parent) && deeper < 64 ? doc_place(d, parent, now, deeper + 1) : math::mat4{};   // (64: a document written elsewhere with a ring of parents still ends)
}
// Where a split text's letter has its middle across the thing, in its own units (0: not a letter of one).
static double doc_letter_middle(Doc& d, int thing, int copy) {
    const auto sh = d.shown.find(thing);
    if (copy < 0 || sh == d.shown.end() || !sh->second.drawn || copy >= (int)sh->second.drawn->letters.size()) return 0.0;
    float2 half{0.5f};
    float deep = 0.f;
    doc_half(d, thing, &half, &deep);
    const float2 l = sh->second.drawn->letters[(size_t)copy];
    return ((double)(l.x + l.y) * 0.5 - 0.5) * 2.0 * half.x;
}
// The world of the things that fall and strike, made from the document when first asked: each one's shape (a box as
// large as it is, a ball for a ball, flat things held to their plane), what it bounces and rubs, where its keys put
// it. Where the world has one at a moment (false: the world does not have it then, its keys do).
static bool doc_physics(Doc& d, int thing, double now, PhysPose* pose, int copy = -1) {
    if (d.phys_stale) {   // (all the world is made of: every value and key of the things with physics, and the pull down)
        d.phys_stale = false;
        std::string made;
        doc_rows(d, sqlite3_mprintf("select group_concat(thing || ' ' || name || ' ' || at || ' ' || quote(value) || ' ' || coalesce(way, ''), ';') from (select * from fact where thing in "
                                    "(select thing from fact where name = 'physics' and value in ('moves', 'still')) or (thing = 0 and name = 'gravity') order by thing, name, at)"),
                 [&](sqlite3_stmt* row) { if (sqlite3_column_type(row, 0) == SQLITE_TEXT) made = (const char*)sqlite3_column_text(row, 0); });
        if (made != d.phys_made) d.phys.reset(), d.phys_of.clear(), d.phys_copy.clear(), d.phys_until.clear(), d.phys_made = made;
        d.phys_any = made.find(" physics ") != std::string::npos;
    }
    if (!d.phys_any) return false;   // (no thing has physics: nothing asked of the document at every place)
    const std::string mine = doc_word(d, thing, "physics");
    if (mine != "moves") return false;
    if (!d.phys) {
        std::vector<PhysThing> made;
        d.phys_of.clear(), d.phys_copy.clear(), d.phys_until.clear();
        for (int t : doc_laid(d)) {
            const std::string how = doc_word(d, t, "physics");
            if ((how != "moves" && how != "still") || doc_number(d, t, "parent", 0, 0) != 0.0) continue;
            float2 half{0.5f};
            float deep = 0.f;
            if (!doc_half(d, t, &half, &deep)) continue;
            const double size = doc_number(d, t, "size", 0, 1);
            PhysThing one;
            one.still = how == "still", one.flat = deep == 0.f;
            one.ball = doc_word(d, t, "shape") == "ball";
            one.half[0] = (float)(half.x * size * std::fabs(doc_number(d, t, "scale_x", 0, 1)));
            one.half[1] = (float)(half.y * size * std::fabs(doc_number(d, t, "scale_y", 0, 1)));
            one.half[2] = one.flat ? 0.05f : (float)(deep * size * std::fabs(doc_number(d, t, "scale_z", 0, 1)));
            one.bounce = (float)doc_number(d, t, "bounce", 0, 0.3), one.friction = (float)doc_number(d, t, "friction", 0, 0.5);
            for (const char* name : {"x", "y", "z", "rotate", "rotate_x", "rotate_y"}) {   // (its keys hold it till the last of them)
                doc_rows(d, sqlite3_mprintf("select max(at) from fact where thing = %d and name = %Q and " DOC_KEYED, t, name, t, name), [&](sqlite3_stmt* row) {
                    if (sqlite3_column_type(row, 0) != SQLITE_NULL) one.until = std::max(one.until, sqlite3_column_double(row, 0));
                });
            }
            const auto sh = d.shown.find(t);
            const std::vector<float2>* letters = sh != d.shown.end() && sh->second.drawn && !sh->second.drawn->letters.empty() ? &sh->second.drawn->letters : nullptr;
            const int copies = letters ? (int)letters->size() : 1;   // (a split text: each letter a body of its own, as wide as its ink, a little narrower so neighbours do not start inside each other)
            for (int c = 0; c < copies; ++c) {
                PhysThing each = one;
                if (letters) each.half[0] = std::max(0.01f, 0.92f * ((*letters)[(size_t)c].y - (*letters)[(size_t)c].x) * one.half[0]);
                made.push_back(each);
                d.phys_of.push_back(t), d.phys_copy.push_back(letters ? c : -1), d.phys_until.push_back(one.still ? 1e9 : one.until);
            }
        }
        const auto keyed = [&d](int index, double at) {   // (where its keys put its middle, and how they turn it)
            const int t = d.phys_of[(size_t)index], copy = d.phys_copy[(size_t)index];
            d.phys_keyed = true;
            const math::double3 middle = (doc_place(d, t, at, 0, std::max(copy, 0), copy >= 0) * math::double4{doc_letter_middle(d, t, copy), 0, 0, 1}).xyz;
            d.phys_keyed = false;
            const auto half = [](double degrees, math::double3 about) { const double a = 0.5 * degrees * (double)TAU / 360.0; return math::quat{std::cos(a), about.x * std::sin(a), about.y * std::sin(a), about.z * std::sin(a)}; };
            const math::quat q = half(doc_number(d, t, "rotate", at, 0), {0, 0, 1}) * half(doc_number(d, t, "rotate_y", at, 0), {0, 1, 0}) * half(doc_number(d, t, "rotate_x", at, 0), {1, 0, 0});
            PhysPose p;
            p.at[0] = (float)middle.x, p.at[1] = (float)middle.y, p.at[2] = (float)middle.z;
            p.turn[0] = (float)q.x, p.turn[1] = (float)q.y, p.turn[2] = (float)q.z, p.turn[3] = (float)q.w;
            return p;
        };
        d.phys = std::make_unique<PhysWorld>(made, (float)doc_number(d, 0, "gravity", 0, 9.81), keyed);
    }
    size_t index = d.phys_of.size();
    for (size_t i = 0; i < d.phys_of.size(); ++i) if (d.phys_of[i] == thing && d.phys_copy[i] == copy) { index = i; break; }   // (a thing's copies that are not letters keep to their keys)
    if (index == d.phys_of.size()) return false;
    if (now <= d.phys_until[index]) return false;   // (its keys have it still)
    *pose = d.phys->pose((int)index, (int)std::lround(now * 60.0));
    return true;
}
// Whether the stage places a thing as a member of its copies (each copy, the first too, as late as its order says, waving
// as its place in the wave says), or as itself: the stage and whoever asks how it moves (motolii_path) place it alike.
static bool doc_staggered(Doc& d, int thing, double now) {
    return doc_members(d, thing) > 1 || doc_number(d, thing, "stagger", 0, 0) > 0.0 || doc_number(d, thing, "wave_x", now, 0) != 0.0 || doc_number(d, thing, "wave_y", now, 0) != 0.0 || doc_number(d, thing, "wave_rotate", now, 0) != 0.0 || doc_number(d, thing, "wave_size", now, 0) != 0.0 || doc_number(d, thing, "field_radius", now, 0) > 0.0 || doc_number(d, thing, "scatter_x", now, 0) > 0.0 || doc_number(d, thing, "scatter_y", now, 0) > 0.0 || doc_number(d, thing, "scatter_rotate", now, 0) > 0.0 || doc_number(d, thing, "scatter_size", now, 0) > 0.0;
}
static math::mat4 doc_place(Doc& d, int thing, double now, int deeper, int copy, bool member) {
    const double said = now;   // (the moment itself: a wave goes on at it, whatever the copy's delay)
    if (member) now -= doc_delay(d, thing, copy);   // (a member of its copies: as it was so much earlier)
    const auto turned = [&](const char* name, math::double3 about) { return math::mat4::rotation(doc_number(d, thing, name, now, 0) * (double)TAU / 360.0, about); };
    float2 half{0.5f};
    float deep = 0.f;
    doc_half(d, thing, &half, &deep);
    const math::double3 anchor{doc_number(d, thing, "anchor_x", now, 0) * half.x, doc_number(d, thing, "anchor_y", now, 0) * half.y, doc_number(d, thing, "anchor_z", now, 0) * deep};
    // A member of its copies is moved, scattered and waved in units of the stage (so a larger thing is not further
    // apart), its copies laid about the thing's own place (its middle where the rows' middle is); then turned and sized
    // as its copy, its scattering and its wave say, about its anchor.
    const int n = member ? doc_members(d, thing) : 1;
    math::double3 shift{0.0};
    math::mat4 own;
    if (member) {
        const int columns = (int)doc_number(d, thing, "copy_columns", 0, 0);
        const double along = columns > 0 ? copy % columns : copy, down = columns > 0 ? copy / columns : copy;
        const double alongMid = columns > 0 ? (std::min(n, columns) - 1) * 0.5 : (n - 1) * 0.5, downMid = columns > 0 ? (std::ceil((double)n / columns) - 1) * 0.5 : (n - 1) * 0.5;
        shift += math::double3{(along - alongMid) * doc_number(d, thing, "copy_x", now, 0), (down - downMid) * doc_number(d, thing, "copy_y", now, 0), (copy - (n - 1) * 0.5) * doc_number(d, thing, "copy_z", now, 0)};
        own = math::mat4::rotation(copy * (doc_number(d, thing, "copy_turn", now, 0) / n + doc_number(d, thing, "copy_rotate", now, 0)) * (double)TAU / 360.0, math::double3{0, 0, 1}) *
              math::mat4::scaling(std::pow(doc_number(d, thing, "copy_size", now, 1), copy)) *
              math::mat4::scaling(math::double3{n > 1 ? std::pow(doc_number(d, thing, "copy_wide", now, 1), (double)copy / (n - 1)) : 1.0, 1.0, 1.0});
        if (const double ty = doc_number(d, thing, "copy_rotate_y", now, 0), tx = doc_number(d, thing, "copy_rotate_x", now, 0); tx != 0.0 || ty != 0.0)   // (turned out of the picture's plane too, each more than the one before)
            own = math::mat4::rotation(copy * ty * (double)TAU / 360.0, math::double3{0, 1, 0}) * math::mat4::rotation(copy * tx * (double)TAU / 360.0, math::double3{1, 0, 0}) * own;
        // (scattered: each copy its own way, from the seed and its number; the same each time)
        const double sx = doc_number(d, thing, "scatter_x", now, 0), sy = doc_number(d, thing, "scatter_y", now, 0), sr = doc_number(d, thing, "scatter_rotate", now, 0), ss = doc_number(d, thing, "scatter_size", now, 0);
        const uint32_t seed = (uint32_t)doc_number(d, thing, "seed", 0, 0);
        const auto roll = [&](uint32_t k) {   // (-1 to 1: a hash of the seed, the copy and which roll)
            uint32_t h = seed * 0x9E3779B9u ^ ((uint32_t)copy + 1u) * 0x85EBCA6Bu ^ k * 0xC2B2AE35u;
            h ^= h >> 16, h *= 0x7FEB352Du, h ^= h >> 15, h *= 0x846CA68Bu, h ^= h >> 16;
            return (double)h / 2147483647.5 - 1.0;
        };
        if (const double sz = doc_number(d, thing, "scatter_z", now, 0); sz != 0.0) shift.z += sz * roll(5);
        if (sx != 0.0 || sy != 0.0 || sr != 0.0 || ss != 0.0) {
            const auto round = [](double v, double edge) { return edge > 0.0 ? v - 2.0 * edge * std::floor((v + edge) / (2.0 * edge)) : v; };   // (flowing past the edge: back at the other)
            const auto whole = [&](double drift, double edge) { const double k = edge > 0.0 ? std::round(drift * d.round / (2.0 * edge)) : 0.0; return k != 0.0 ? k * 2.0 * edge / d.round : drift; };   // (round the scattering a whole number of times in the document)
            shift += math::double3{round(sx * roll(1) + whole(doc_number(d, thing, "drift_x", 0, 0), sx) * now, sx), round(sy * roll(2) + whole(doc_number(d, thing, "drift_y", 0, 0), sy) * now, sy), 0.0};
            own = own * math::mat4::rotation(sr * roll(3) * (double)TAU / 360.0, math::double3{0, 0, 1}) * math::mat4::scaling(std::max(0.0, 1.0 + ss * roll(4)));
        }
        // (a wave through it: each copy so far behind the one before, at its own moment, a copy staggered so much later)
        const double said_speed = doc_number(d, thing, "wave_speed", said, 1), noisy = doc_number(d, thing, "wave_noise", said, 0);
        const double turns = std::round(said_speed * d.round), speed = turns > 0.0 ? turns / d.round : said_speed;   // (a whole number of times round in the document: its film loops)
        double w = std::sin((double)TAU * (speed * now - doc_number(d, thing, "wave_phase", said, 0) * copy / n));
        if (noisy > 0.0) {   // (a smooth noise: random values at the corners of a lattice, eased between; its own for each copy, or, with a scale, read where the copy lies)
            const double scale = doc_number(d, thing, "noise_scale", said, 0);
            const auto at = [&](int64_t i, int64_t j, int64_t k) {
                const int64_t ring = turns > 0.0 ? (int64_t)turns * 2 : 0;   // (round in time as the wave is: the lattice's last row is its first)
                if (ring > 0) k = ((k % ring) + ring) % ring;
                uint32_t h = seed * 0x9E3779B9u ^ (uint32_t)i * 0x85EBCA6Bu ^ (uint32_t)j * 0x27d4eb2du ^ (uint32_t)k * 0xC2B2AE35u ^ (scale > 0.0 ? 0u : ((uint32_t)copy + 1u) * 0x165667B1u);
                h ^= h >> 16, h *= 0x7FEB352Du, h ^= h >> 15, h *= 0x846CA68Bu, h ^= h >> 16;
                return (double)h / 2147483647.5 - 1.0;
            };
            const math::double3 p{scale > 0.0 ? shift.x / scale : 0.0, scale > 0.0 ? shift.y / scale : 0.0, speed * now * 2.0};
            const math::double3 f{std::floor(p.x), std::floor(p.y), std::floor(p.z)}, u = p - f, e = u * u * (3.0 - 2.0 * u);
            double sum = 0.0;
            for (int c = 0; c < 8; ++c)
                sum += at((int64_t)f.x + (c & 1), (int64_t)f.y + ((c >> 1) & 1), (int64_t)f.z + ((c >> 2) & 1)) * ((c & 1) ? e.x : 1.0 - e.x) * ((c & 2) ? e.y : 1.0 - e.y) * ((c & 4) ? e.z : 1.0 - e.z);
            w = w * (1.0 - noisy) + noisy * sum;
        }
        shift += math::double3{doc_number(d, thing, "wave_x", said, 0), doc_number(d, thing, "wave_y", said, 0), 0.0} * w;
        shift.z += doc_number(d, thing, "wave_z", said, 0) * std::cos((double)TAU * (speed * now - doc_number(d, thing, "wave_phase", said, 0) * copy / n));   // (a quarter round after: round and round with up and down)
        own = own * math::mat4::rotation(w * doc_number(d, thing, "wave_rotate", said, 0) * (double)TAU / 360.0, math::double3{0, 0, 1}) *
              math::mat4::scaling(std::max(0.0, 1.0 + w * doc_number(d, thing, "wave_size", said, 0)));
    }
    if (member)   // (a recipe as code: each copy where the thing's script says, on top of what its copies' values say)
        if (DocScript* sc = doc_script_of(doc_word(d, thing, "script")); sc && sc->run && sc->run->ok()) {
            Doc::Scripted& kept = d.scripted[thing];
            if (kept.at != said || kept.made != doc_scripts_made() || (int)kept.copies.size() != n) {   // (every copy at once, the knobs read once)
                std::map<std::string, double> numbers;
                std::map<std::string, std::string> words;
                for (size_t i = 0; i < sc->knobs.size(); ++i) {
                    if (sc->knobs[i].kind == "select") {   // (not said yet: its first word)
                        const std::string w = doc_word(d, thing, sc->names[i].c_str());
                        words[sc->knobs[i].name] = w.empty() ? sc->knobs[i].words.substr(0, sc->knobs[i].words.find(' ')) : w;
                    }
                    else numbers[sc->knobs[i].name] = doc_number(d, thing, sc->names[i].c_str(), said, sc->knobs[i].usual);
                }
                kept.at = said, kept.made = doc_scripts_made();
                const double begins = doc_number(d, thing, "start", 0, 0);
                numbers["duration"] = doc_number(d, thing, "duration", 0, -1);   // (how long the thing lasts: an Out ends before it)
                kept.copies = sc->run->copies(n, said - begins, numbers, words, (double)d.stage->h / std::max(1.0, doc_number(d, 0, "high", 0, 1080) * doc_point(d, thing, said)));   // (a pixel of the document, where the thing stands)
            }
            const ScriptCopy c = copy < n ? kept.copies[(size_t)copy] : ScriptCopy{};
            shift += math::double3{c.x, c.y, c.z};
            own = own * math::mat4::rotation(c.rotate * (double)TAU / 360.0, math::double3{0, 0, 1}) * math::mat4::rotation(c.rotate_y * (double)TAU / 360.0, math::double3{0, 1, 0}) *
                  math::mat4::rotation(c.rotate_x * (double)TAU / 360.0, math::double3{1, 0, 0}) * math::mat4::scaling(c.size);
        }
    double tall = doc_number(d, thing, "tall", now, 1);   // (taller, its anchor staying: a copy as its number in copy_tall says too)
    if (member) {
        std::istringstream said(doc_word(d, thing, "copy_tall"));
        std::vector<double> each;
        for (std::string one; said >> one && one != "none";) each.push_back(std::atof(one.c_str()));
        if (!each.empty()) tall *= each[(size_t)copy % each.size()];
    }
    own = own * math::mat4::scaling(math::double3{1.0, tall, 1.0});
    if (member)   // (a split text's letter turns and grows about its own middle, not the whole line's)
        if (const double middle = doc_letter_middle(d, thing, copy); middle != 0.0) own = math::mat4::translation(math::double3{middle, 0, 0}) * own * math::mat4::translation(math::double3{-middle, 0, 0});
    if (!d.phys_keyed && !deeper) {   // (a thing that falls and strikes, or a letter of one: where the world has its middle, turned as it has it)
        PhysPose pose;
        if (doc_physics(d, thing, now, &pose, member ? copy : -1)) {
            const math::quat q{pose.turn[3], pose.turn[0], pose.turn[1], pose.turn[2]};
            return math::mat4::translation(math::double3{pose.at[0], pose.at[1], pose.at[2]}) * math::mat4(math::mat3(math::quat(q))) *
                   math::mat4::scaling(doc_number(d, thing, "size", now, 1)) *
                   math::mat4::scaling(math::double3{doc_number(d, thing, "scale_x", now, 1), doc_number(d, thing, "scale_y", now, 1), doc_number(d, thing, "scale_z", now, 1)}) *
                   math::mat4::translation(math::double3{-doc_letter_middle(d, thing, member ? copy : -1), 0, 0});
        }
    }
    math::mat4 place = doc_above(d, thing, now, deeper) * math::mat4::translation(math::double3{doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0)}) *
                       turned("rotate", {0, 0, 1}) * turned("rotate_y", {0, 1, 0}) * turned("rotate_x", {1, 0, 0}) * math::mat4::translation(shift) * math::mat4::scaling(doc_number(d, thing, "size", now, 1)) *
                       math::mat4::scaling(math::double3{doc_number(d, thing, "scale_x", now, 1), doc_number(d, thing, "scale_y", now, 1), doc_number(d, thing, "scale_z", now, 1)}) *
                       own * math::mat4::translation(-anchor);
    const double reach = member ? doc_number(d, thing, "field_radius", said, 0) : 0.0;
    if (reach > 0.0) {   // (within the field: larger and turned about where it is seen, its picture's middle, as near the field's middle as it is)
        const math::double3 at = (place * math::double4{0.0, 0.0, 0.0, 1.0}).xyz;
        const math::double3 middle{doc_number(d, thing, "field_x", said, 0), doc_number(d, thing, "field_y", said, 0), doc_number(d, thing, "field_z", said, 0)};   // (a ball: near and far measured too)
        const double from = length(at - middle), band = doc_number(d, thing, "field_band", said, 0);
        const double near = std::clamp(band > 0.0 ? 1.0 - std::fabs(from - reach) / band : 1.0 - from / reach, 0.0, 1.0), k = near * near * (3.0 - 2.0 * near);
        const math::double3 away = at - middle;
        const double r = length(away), by = doc_number(d, thing, "field_push", said, 0);   // (pushed out as if from a hole that wide: the nearest ones all end about as far out, the hole round however they lay)
        const math::double3 push = r > 1e-9 ? normalize(away) * (by > 0.0 ? k * (std::sqrt(r * r + by * by) - r) : k * by) : math::double3{0.0};
        place = math::mat4::translation(push) * math::mat4::translation(at) * math::mat4::rotation(k * doc_number(d, thing, "field_rotate", said, 0) * (double)TAU / 360.0, math::double3{0, 0, 1}) *
                math::mat4::scaling(std::max(0.0, 1.0 + k * doc_number(d, thing, "field_size", said, 0))) * math::mat4::translation(-at) * place;
    }
    // (the user's three tags: 2D sticks to the camera; 2.5D stays in the world, smaller far, but keeps the angle it
    //  was given on the screen; 3D, and a solid, is filmed as it is)
    const auto shown = d.shown.find(thing);
    if (deeper || shown == d.shown.end() || shown->second.space == 3) return place;
    if (shown->second.space == 2) return d.rig * place;
    const math::double3 at = (place * math::double4{anchor, 1.0}).xyz;   // (where it stands)
    return math::mat4::translation(at) * d.facing * math::mat4::translation(-at) * place;
}
// Where a joining thing's copies lie (its connect): one from each copy of the thing it joins to another, stretched
// along the way between them and as fine as they are far apart; those left over shown nowhere. Only where its copies
// are placed, as any thing's: the lines are its own copies, in its own fill.
static std::vector<math::mat4> doc_joins(Doc& d, int thing, double now) {
    const int one = (int)doc_number(d, thing, "connect", 0, 0), two = (int)doc_number(d, thing, "connect_to", 0, 0);
    if (!one || one == thing || !d.shown.count(one) || (two && (two == thing || !d.shown.count(two)))) return {};
    const auto where = [&](int other) {   // (where each copy of it is seen, its picture's middle, before its space is taken into account)
        std::vector<math::double3> at((size_t)doc_members(d, other));
        for (size_t i = 0; i < at.size(); ++i) at[i] = (doc_place(d, other, now, 1, (int)i, true) * math::double4{0.0, 0.0, 0.0, 1.0}).xyz;
        return at;
    };
    const std::vector<math::double3> a = where(one), b = two ? where(two) : a;
    const double reach = doc_number(d, thing, "connect_reach", now, 0);
    std::vector<std::pair<size_t, size_t>> pairs;
    const size_t most = (size_t)doc_members(d, thing);
    if (two) for (size_t i = 0; i < std::min(a.size(), b.size()); ++i) pairs.push_back({i, i});
    else if (reach <= 0.0) for (size_t i = 0; i + 1 < a.size(); ++i) pairs.push_back({i, i + 1});
    else for (size_t i = 0; i < a.size() && pairs.size() < most; ++i) for (size_t j = i + 1; j < a.size() && pairs.size() < most; ++j) if (length(a[i] - a[j]) < reach) pairs.push_back({i, j});
    float2 half{0.5f};
    float deep = 0.f;
    doc_half(d, thing, &half, &deep);
    const auto shown = d.shown.find(thing);
    const int space = shown == d.shown.end() ? 3 : shown->second.space;
    std::vector<math::mat4> lines;
    for (size_t k = 0; k < pairs.size() && k < most; ++k) {
        const math::double3 from = a[pairs[k].first], to = b[pairs[k].second], mid = (from + to) * 0.5, way = to - from;
        const double far = length(way), fine = reach > 0.0 && !two ? 0.35 + 0.65 * (1.0 - far / reach) : 1.0;   // (never finer than about a third: thinner than a dot of the screen it breaks up)
        // (along the way between them however deep it goes; its width across, as level as it can lie, facing the eye)
        const math::double3 along = far > 1e-9 ? way / far : math::double3{1, 0, 0}, flat = cross(along, math::double3{0, 0, 1});
        const math::double3 across = length(flat) > 1e-6 ? normalize(flat) : math::double3{0, 1, 0}, up = cross(along, across);
        const double wide = doc_number(d, thing, "size", now, 1) * fine;
        math::mat4 line;
        line[0] = math::double4{along * (far / std::max(1e-6, 2.0 * half.x)), 0.0};
        line[1] = math::double4{-across * wide, 0.0};
        line[2] = math::double4{up, 0.0};
        line[3] = math::double4{mid, 1.0};
        lines.push_back(space == 2 ? d.rig * line : line);
    }
    return lines;
}

// The shapes the kit's mattes are cut by that `which` names, taken out of their matte's scene and done away with,
// their looks too (a look holds the picture it reads: Filament will not let that picture go while a look holds it).
static void doc_unshape(Stage& s, const std::function<bool(const Stage::Made&)>& which) {
    auto& faces = s.engine->getRenderableManager();
    for (size_t i = s.made.size(); i-- > 0;) {
        if (!s.made[i].matte || !which(s.made[i])) continue;
        const utils::Entity e = s.made[i].thing;
        MaterialInstance* look = faces.getMaterialInstanceAt(faces.getInstance(e), 0);
        for (Stage::Matte& mt : s.mattes) mt.scene->remove(e);
        s.engine->destroy(e);
        s.engine->destroy(look);
        s.made.erase(s.made.begin() + (long)i);
    }
}
// Every thing told its moment: whether it is on (from its start, for its duration if it has one), what of itself it
// shows (it goes round when it comes to its end), and where it is.
// The stage's own looks, Filament's, as the document says them (thing 0): its grade made again only when it is told
// otherwise; the glow, flare, darkened corners and fog on the stage's own view; the room's light and the colour behind.
static bool doc_knows(Doc& d, int thing, const char* name, const double* number, const char* word, bool key);
static bool doc_frame(Doc& d, int thing, double now, double* frame);
static void doc_look(Doc& d) {
    Stage& s = *d.stage;
    const auto hex = [&](const char* name, float3 otherwise) { const std::string w = doc_word(d, 0, name); return w.size() == 6 ? float3{(float)((std::strtol(w.c_str(), nullptr, 16) >> 16) & 255), (float)((std::strtol(w.c_str(), nullptr, 16) >> 8) & 255), (float)(std::strtol(w.c_str(), nullptr, 16) & 255)} / 255.f : otherwise; };
    const std::string mapper = doc_word(d, 0, "tone_mapper");
    char grade[300];
    const float slope = (float)doc_number(d, 0, "slope", 0, 1), offset = (float)doc_number(d, 0, "offset", 0, 0), power = (float)doc_number(d, 0, "power", 0, 1);
    std::snprintf(grade, sizeof grade, "%s %g %g %g %g %g %g %g %g", mapper.c_str(), doc_number(d, 0, "exposure", 0, 0), doc_number(d, 0, "contrast", 0, 1), doc_number(d, 0, "saturation", 0, 1), doc_number(d, 0, "vibrance", 0, 1), doc_number(d, 0, "temperature", 0, 0), slope, offset, power);
    const auto tint = [&](const char* name) { const float3 c = hex(name, float3{1.f}); std::strncat(grade, (" " + doc_word(d, 0, name)).c_str(), 20); return float4{Color::toLinear<ACCURATE>(sRGBColor{c}), 0.f}; };
    const float4 low = tint("shadows_color"), mid = tint("midtones_color"), high = tint("highlights_color");
    if (d.graded != grade) {
        static LinearToneMapper linear; static ACESToneMapper aces; static FilmicToneMapper filmic; static AgxToneMapper agx; static PBRNeutralToneMapper neutral;
        const ToneMapper* tm = mapper == "aces" ? (const ToneMapper*)&aces : mapper == "filmic" ? (const ToneMapper*)&filmic : mapper == "agx" ? (const ToneMapper*)&agx : mapper == "pbr_neutral" ? (const ToneMapper*)&neutral : (const ToneMapper*)&linear;
        ColorGrading* was = const_cast<ColorGrading*>(s.view->getColorGrading());
        ColorGrading* now = ColorGrading::Builder().toneMapper(tm).exposure((float)doc_number(d, 0, "exposure", 0, 0)).contrast((float)doc_number(d, 0, "contrast", 0, 1))
                                .saturation((float)doc_number(d, 0, "saturation", 0, 1)).vibrance((float)doc_number(d, 0, "vibrance", 0, 1)).whiteBalance((float)doc_number(d, 0, "temperature", 0, 0), 0.f)
                                .slopeOffsetPower(float3{slope}, float3{offset}, float3{power})
                                .shadowsMidtonesHighlights(low, mid, high, float4{0.f, 0.333f, 0.55f, 1.f}).build(*s.engine);   // (the ranges as Filament's own)
        s.view->setColorGrading(now);
        for (View* v : s.layers) v->setColorGrading(now);   // (the same grade on every picture laid over the stage's own)
        if (was) s.engine->destroy(was);
        d.graded = grade;
    }
    View::BloomOptions glow = s.view->getBloomOptions();
    glow.enabled = doc_number(d, 0, "bloom", 0, 0) > 0.0, glow.strength = (float)doc_number(d, 0, "bloom", 0, 0);
    glow.lensFlare = glow.starburst = doc_number(d, 0, "lens_flare", 0, 0) > 0.0;
    glow.enabled = glow.enabled || glow.lensFlare;   // (a flare is drawn by the glow's own passes)
    glow.chromaticAberration = 0.005f * (float)doc_number(d, 0, "lens_flare", 0, 0), glow.ghostThreshold = glow.haloThreshold = 1.f;
    s.view->setBloomOptions(glow);
    const std::string smooth = doc_word(d, 0, "anti_aliasing");   // (on every picture laid: each smooths its own edges)
    for (View* v : std::vector<View*>{s.view}) {
        v->setAntiAliasing(smooth.empty() || smooth == "fxaa" ? View::AntiAliasing::FXAA : View::AntiAliasing::NONE);
        View::TemporalAntiAliasingOptions taa = v->getTemporalAntiAliasingOptions();
        taa.enabled = smooth == "taa";
        v->setTemporalAntiAliasingOptions(taa);
    }
    View::MultiSampleAntiAliasingOptions many = s.view->getMultiSampleAntiAliasingOptions();
    many.enabled = smooth == "msaa", many.sampleCount = 4;
    s.view->setMultiSampleAntiAliasingOptions(many);
    for (View* v : s.layers) v->setAntiAliasing(s.view->getAntiAliasing()), v->setTemporalAntiAliasingOptions(s.view->getTemporalAntiAliasingOptions()), v->setMultiSampleAntiAliasingOptions(many);
    View::VignetteOptions corners = s.view->getVignetteOptions();
    corners.enabled = doc_number(d, 0, "vignette", 0, 0) > 0.0, corners.midPoint = 1.f - 0.9f * (float)doc_number(d, 0, "vignette", 0, 0), corners.color = LinearColorA{hex("vignette_color", float3{0.f}), 1.f};
    s.view->setVignetteOptions(corners);
    View::FogOptions haze = s.view->getFogOptions();
    haze.enabled = doc_number(d, 0, "fog", 0, 0) > 0.0, haze.density = 0.5f * (float)doc_number(d, 0, "fog", 0, 0), haze.distance = 2.f, haze.height = -1e3f, haze.heightFalloff = 0.f, haze.color = hex("fog_color", float3{0.85f, 0.85f, 0.9f}), haze.fogColorFromIbl = false;
    s.view->setFogOptions(haze);
    View::AmbientOcclusionOptions folds = s.view->getAmbientOcclusionOptions();
    folds.enabled = doc_number(d, 0, "ambient_occlusion", 0, 0) > 0.0, folds.intensity = 2.f * (float)doc_number(d, 0, "ambient_occlusion", 0, 0);
    s.view->setAmbientOcclusionOptions(folds);
    View::ScreenSpaceReflectionsOptions mirror = s.view->getScreenSpaceReflectionsOptions();
    mirror.enabled = doc_number(d, 0, "reflections", 0, 0) > 0.0;
    s.view->setScreenSpaceReflectionsOptions(mirror);
    if (d.roomLight <= 0.f && s.scene->getIndirectLight()) d.roomLight = s.scene->getIndirectLight()->getIntensity();
    std::string which = doc_word(d, 0, "room");
    if (which.empty()) which = "small_studio";   // (as the stage was opened)
    if (which != d.roomIs) {   // (another room: made once from its file, then kept; every picture laid over the stage's own is lit by it too)
        static std::map<std::string, IndirectLight*> made;
        if (!d.roomIs.empty() || made.empty()) made.emplace(d.roomIs.empty() ? "small_studio" : d.roomIs, s.scene->getIndirectLight());
        const auto found = made.find(which);
        if (found != made.end()) s.scene->setIndirectLight(found->second);
        else if (room(s, ("out/" + which + "/" + which + "_ibl.ktx").c_str(), d.roomLight)) made[which] = s.scene->getIndirectLight();
        for (View* v : s.layers) v->getScene()->setIndirectLight(s.scene->getIndirectLight());
        d.roomIs = which;
    }
    if (IndirectLight* room = s.scene->getIndirectLight()) {
        room->setIntensity(d.roomLight * (float)doc_number(d, 0, "environment", 0, 1));
        room->setRotation(math::mat3f::rotation((float)(doc_number(d, 0, "environment_rotate", 0, 0) * (double)TAU / 360.0), math::float3{0, 1, 0}));
    }
    screen_effects(s, d.screen, doc_word(d, 0, "script"), [&d](const std::string& file, const std::string& name, double usual) {
        std::lock_guard<std::recursive_mutex> hold(d.lock);   // (asked while the picture is drawn, when the hand does not hold it)
        // (a mouse's knob, where the followed thing is on the picture: pointer as the web says it, -1 to 1 and up; the
        //  others from the top left, 0 to 1, as the effects' own uv)
        static const char* const mice[] = {"pointer", "center", "mouse", "point"};
        if (const int t = (int)doc_number(d, 0, "follow", 0, 0); t > 0 && name.size() > 2 && name[name.size() - 2] == '_')
            for (const char* mouse : mice)
                if (name.compare(0, name.size() - 2, mouse) == 0) {
                    double frame[8];
                    if (!doc_knows(d, t, nullptr, nullptr, nullptr, false) || !doc_frame(d, t, d.stage->now, frame)) break;
                    const bool across = name.back() == 'x';
                    const double at = across ? frame[0] / d.stage->w : frame[1] / d.stage->h;
                    return !strcmp(mouse, "pointer") ? (across ? 2.0 * at - 1.0 : 1.0 - 2.0 * at) : at;
                }
        return doc_number(d, 0, doc_screen_knob(file, name).c_str(), d.stage->now, usual);
    });
    const std::string behind = doc_word(d, 0, "background");
    if (doc_word(d, 0, "behind") == "room") {   // (the room itself behind everything, as bright as it lights the stage: its own picture, made once a room)
        static std::map<std::string, Skybox*> shown;
        Skybox*& sky = shown[d.roomIs];
        if (!sky) {
            static std::vector<std::vector<uint8_t>> kept;
            kept.push_back(load_file(("out/" + d.roomIs + "/" + d.roomIs + "_skybox.ktx").c_str()));
            if (!kept.back().empty())
                if (Texture* tex = ktxreader::Ktx1Reader::createTexture(s.engine, new image::Ktx1Bundle(kept.back().data(), (uint32_t)kept.back().size()), false))
                    sky = Skybox::Builder().environment(tex).intensity(d.roomLight).showSun(true).build(*s.engine);   // (a sun among the lights: its disk in it)   // (Filament's sky takes its brightness as it is made)
        }
        if (sky) sky->setLayerMask(0xFF, 0x01);
        s.scene->setSkybox(sky);
    } else s.scene->setSkybox(nullptr);
    // A colour behind everything: a flat colour as large as the stage, stuck to the eye, drawn first in the stage's own
    // view and writing no depth, so every thing is drawn over it (Filament's sky is drawn after the things it holds, and
    // a flat thing writes no depth, so a sky painted over them). It is opaque: Filament draws blended sheets after every
    // solid thing whatever their priority, so a blended backdrop was painted over the solids.
    const std::string colour = doc_word(d, 0, "behind") == "room" ? "" : behind;
    if (colour != d.backdropIs) {
        if (d.backdrop) picture_off(s, d.backdrop), d.backdrop = nullptr;
        if (!colour.empty()) {
            const math::double4 up = s.camera->getProjectionMatrix() * s.camera->getViewMatrix() * math::double4{0, 1, 0, 1};
            if ((d.backdrop = flat(s, "rect", (float)s.w / (float)s.h, mat4f{}, (float)(2.0 * up.w / up.y), hex("background", float3{0.f}), "opaque"))) {   // (opaque: Filament draws it with the solid things, first by its priority, not over them with the sheets)
                auto& faces = s.engine->getRenderableManager();
                for (const Stage::Made& made : s.made)
                    if (made.of == d.backdrop->sheet) faces.setPriority(faces.getInstance(made.thing), 0), faces.setChannel(faces.getInstance(made.thing), 0);
                d.backdrop->look->setDepthWrite(false), d.backdrop->look->setDepthCulling(false);
            }
        }
        d.backdropIs = colour;
    }
    if (d.backdrop) move(s, d.backdrop->sheet, [&](int, const mat4f&) { return mat4f(d.rig); });
}

static int doc_keys(Doc& d, int thing, const char* name, double at);
// A value at its most: the highest of its keys, or what it is said to be (how far a moving effect may reach).
static double doc_most(Doc& d, int thing, const char* name, double otherwise) {
    double most = doc_number(d, thing, name, 0, otherwise);
    if (doc_keys(d, thing, name, -1.0)) doc_rows(d, sqlite3_mprintf("select max(value) from fact where thing = %d and name = %Q", thing, name), [&](sqlite3_stmt* row) { most = sqlite3_column_double(row, 0); });
    return most;
}
// The effects of a flat thing at a moment (or, `most`, as far as each reaches at its most), ThorVG's own: in shares of
// its height from points of the picture (`units` to a unit of the stage, as it was made). false: a colour is not
// six digits (why says which).
static bool doc_fx(Doc& d, int thing, double at, bool most, double units, DrawnFx& fx, std::string& why) {
    const auto num = [&](const char* name, double otherwise) { return most ? doc_most(d, thing, name, otherwise) : doc_number(d, thing, name, at, otherwise); };
    bool ok = true;
    const auto hue = [&](const char* name, const char* otherwise, float3* to) {   // (a colour said as six digits; not said: as it comes)
        std::string said = doc_word(d, thing, name);
        if (said.empty()) said = otherwise;
        const long rgb = said.size() == 6 && said.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(said.c_str(), nullptr, 16) : -1;
        if (rgb < 0) return ok = false, why = std::string("a ") + name + " is six digits, as ff3366", void();
        *to = float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f;
    };
    fx.blur = (float)(num("blur", 0) / units);
    const std::string way = doc_word(d, thing, "blur_direction");
    fx.blurWay = way == "horizontal" ? 1 : way == "vertical" ? 2 : 0;
    if (!doc_word(d, thing, "shadow").empty()) {   // (After Effects' own drop shadow, as it comes: half of it, 135 degrees, 10 points away, 10 soft)
        float3 dark{0.f};
        hue("shadow", "000000", &dark);
        fx.shadow = float4{dark, (float)num("shadow_opacity", 0.5)};
        fx.fall = (float)(num("shadow_distance", 10) / units), fx.way = (float)num("shadow_angle", 135), fx.soft = (float)(num("shadow_blur", 10) / units);
    }
    if (!doc_word(d, thing, "glow").empty()) {   // (a warm light all round it, as it comes: soft 40, all of it; ThorVG's shadow spreads narrowly)
        float3 light{0.f};
        hue("glow", "ffd166", &light);
        fx.glow = float4{light, (float)num("glow_opacity", 1)}, fx.glowSoft = (float)(num("glow_blur", 40) / units);
    }
    if ((fx.tint = (float)num("tint", 0)) > 0.f) hue("tint_black", "1b1f5c", &fx.tints[0]), hue("tint_white", "ff8a65", &fx.tints[1]);   // (a deep blue and a warm pink, as it comes)
    if ((fx.tritone = (float)num("tritone", 0)) > 0.f) hue("tritone_shadow", "1b1f5c", &fx.tones[0]), hue("tritone_midtone", "c44569", &fx.tones[1]), hue("tritone_highlight", "ffd166", &fx.tones[2]);
    fx.reach = (float)(std::max(0.0, num("offset", 0)) + std::fabs(num("zigzag", 0)) + 0.5 * std::max(0.0, num("bloat", 0)) + std::fabs(num("warp", 0)));
    fx.warped = doc_most(d, thing, "warp", 0) != 0.0 || doc_number(d, thing, "warp", 0, 0) != 0.0;   // (how far its bent path goes out past it)
    return ok;
}

static void doc_moment(Doc& d, double now) {
    Stage& s = *d.stage;
    const std::vector<int> lie = doc_laid(d);
    // Each thing as it is now, in the order they lie: whether it is on, and whether it is a solid thing.
    struct Lies { int thing; Doc::Shown* shown; bool on, solid; double since; };
    std::vector<Lies> lies;
    bool solo = false;   // (while any thing is solo, only those are seen)
    for (int thing : lie) solo = solo || doc_number(d, thing, "solo", 0, 0) > 0.5;
    for (int thing : lie) {
        const auto found = d.shown.find(thing);
        if (found == d.shown.end() || found->second.camera) continue;
        const double since = now - doc_number(d, thing, "start", 0, 0), lasts = doc_number(d, thing, "duration", 0, -1);
        Lies one{thing, &found->second, since >= 0.0 && (lasts < 0.0 || since < lasts) && doc_number(d, thing, "visible", 0, 1) > 0.5 && (!solo || doc_number(d, thing, "solo", 0, 0) > 0.5), found->second.space != 2, since};   // (a sheet in the world settles depth with the solid things)
        lies.push_back(one);
    }
    // The camera: the nearest one in the order of the things that is on (the past repository's choice); none, the
    // stage's own. It looks at where it is, from round about it as it is turned (y: about, x: up and down, z: its lean),
    // `distance` times the stage's own way off, `zoom` times as near.
    {
        int camera = 0;
        for (int thing : lie)
            if (d.shown.count(thing) && d.shown[thing].camera) {
                const double since = now - doc_number(d, thing, "start", 0, 0), lasts = doc_number(d, thing, "duration", 0, -1);
                if (since >= 0.0 && (lasts < 0.0 || since < lasts) && doc_number(d, thing, "visible", 0, 1) > 0.5) camera = thing;
            }
        const auto turned = [&](const char* name, math::double3 about) { return camera ? math::mat4::rotation(doc_number(d, camera, name, now, 0) * (double)TAU / 360.0, about) : math::mat4{}; };
        const math::mat4 round = turned("rotate_y", {0, 1, 0}) * turned("rotate_x", {1, 0, 0}) * turned("rotate", {0, 0, 1});
        const math::double3 look = camera ? (doc_place(d, camera, now) * math::double4{0, 0, 0, 1}).xyz : math::double3{0.0};
        const double far = camera ? doc_number(d, camera, "distance", now, 1) : 1.0, near = camera ? doc_number(d, camera, "zoom", now, 1) : 1.0;
        const math::double3 eye = look + (round * math::double4{0, 0, 6.0 * far, 0}).xyz, up = (round * math::double4{0, 1, 0, 0}).xyz;
        s.camera->lookAt(eye, look, up);
        const double fov = 2.0 * std::atan(std::tan(15.0 * (double)TAU / 360.0) / near);
        if (camera && doc_word(d, camera, "projection") == "orthographic") {   // (as large at the point it looks at as the perspective one shows it there)
            const double high = 6.0 * far * std::tan(0.5 * fov), wide = high * double(s.w) / double(s.h);
            s.camera->setProjection(Camera::Projection::ORTHO, -wide, wide, -high, high, 0.05, 60.0 * std::max(1.0, far));
        } else s.camera->setProjection(fov * 360.0 / (double)TAU, double(s.w) / double(s.h), 0.05, 60.0 * std::max(1.0, far));
        s.camera->setShift(camera ? math::double2{doc_number(d, camera, "shift_x", now, 0), doc_number(d, camera, "shift_y", now, 0)} : math::double2{0.0});
        d.rig = s.camera->getModelMatrix() * math::mat4::translation(math::double3{0, 0, -6});   // (from the stage's own camera to this one)
        // (its lens, Filament's own: sharp at the focus, as blurred off it as the f-number says; the shutter goes with the
        //  f-number so the picture is as bright as at the stage's own f/16 and 1/125 s)
        const double said = camera ? doc_number(d, camera, "aperture", now, 0) : 0.0, focus = camera ? doc_number(d, camera, "focus", now, 0) : 0.0;
        d.blurred = said > 0.0;
        const double stop = std::max(said, 0.5);   // (Filament's widest)
        View::DepthOfFieldOptions lens = s.view->getDepthOfFieldOptions();
        if (camera && lens.cocScale != (float)doc_number(d, camera, "blur_level", now, 1)) lens.cocScale = (float)doc_number(d, camera, "blur_level", now, 1), s.view->setDepthOfFieldOptions(lens);
        s.camera->setExposure(d.blurred ? (float)stop : 16.f, d.blurred ? (float)(stop * stop / (16.0 * 16.0 * 125.0)) : 1.f / 125.f, 100.f);
        s.camera->setFocusDistance((float)(focus > 0.0 ? focus : length(eye - look)));
        d.facing = math::mat4(d.rig.upperLeft());
    }
    doc_look(d);
    d.round = doc_seconds(d);
    // Who is before whom (the past repository's law of 2026-09-12: a flat thing is not in the world). Flat things take
    // no part in depth and lie by the order of the things alone; solid things that follow one another settle depth
    // among themselves; a run of the one sort is drawn whole over the run before it. By Filament's own means: the
    // stage's own view holds the first run, and with it the first run of solid things when flat things lie under them
    // (a ground; Filament's channels keep the two in order there, and its depth of field, which only an opaque view
    // takes, blurs that ground as far off); each run after those is a view of its own laid over the ones before (kit.h
    // layer_view; flat things there are sharp). The order within a run of flat things is the blend order, and a flat
    // thing's looks neither ask nor write depth.
    std::vector<int> runOf(lies.size(), -1), sorts;
    for (size_t i = 0; i < lies.size(); ++i) {
        if (!lies[i].on || lies[i].shown->camera || !lies[i].shown->light.isNull()) continue;   // (a camera or a light is in no run: nothing of it is drawn)
        if (sorts.empty() || sorts.back() != (int)lies[i].solid) sorts.push_back((int)lies[i].solid);
        runOf[i] = (int)sorts.size() - 1;
    }
    const int base = sorts.size() > 1 && !sorts[0] ? 1 : 0;   // (the last run in the stage's own view)
    for (size_t i = 0; i < lies.size(); ++i) {
        if (runOf[i] < 0) continue;
        const int run = runOf[i];
        Doc::Shown& t = *lies[i].shown;
        Scene* into = run <= base ? s.scene : layer_view(s, (size_t)(run - base - 1), !lies[i].solid)->getScene();
        Scene* const was = t.scene ? t.scene : s.scene;
        if (was != into) {   // (into the scene of its run; one not seen goes in when it is seen)
            if (t.in) doc_entities(s, t, [&](utils::Entity e) { was->remove(e), into->addEntity(e); });
            t.scene = into;
        }
        if (const int channel = run <= base ? run : 2; channel != t.channel) {
            doc_parts(s, t, [&](RenderableManager& faces, RenderableManager::Instance one) { faces.setChannel(one, (uint8_t)channel); });
            t.channel = channel;
        }
        if (t.run < 0 && !lies[i].solid) doc_looks(s, t, [](MaterialInstance* look) { look->setDepthCulling(false), look->setDepthWrite(false); });
        if (t.lies != (int)i && !lies[i].solid)
            doc_parts(s, t, [&](RenderableManager& faces, RenderableManager::Instance one) {
                for (size_t part = 0; part < faces.getPrimitiveCount(one); ++part) faces.setBlendOrderAt(one, part, (uint16_t)(i * 128)), faces.setGlobalBlendOrderEnabledAt(one, part, true);   // (128 places for each: its copies lie among themselves by how far off they are)
            });
        t.run = run, t.lies = (int)i;
    }
    s.layered = sorts.size() > (size_t)base + 1 ? sorts.size() - (size_t)base - 1 : 0;
    View::DepthOfFieldOptions lens = s.view->getDepthOfFieldOptions();   // (the camera's lens, on the stage's own view when solid things are in it)
    if (lens.enabled != (d.blurred && (base == 1 || (!sorts.empty() && sorts[0]))))
        lens.enabled = !lens.enabled, s.view->setDepthOfFieldOptions(lens);
    for (size_t i = 0; i < lies.size(); ++i) {
        const int thing = lies[i].thing;
        Doc::Shown& t = *lies[i].shown;
        const bool in = lies[i].on;
        const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.shape;
        if (in != t.in) {   // (in or out of the scene of its run)
            Scene* sc = t.scene ? t.scene : s.scene;
            doc_entities(s, t, [&](utils::Entity e) { in ? sc->addEntity(e) : sc->remove(e); });
            t.in = in;
        }
        if (!in) continue;
        if (t.film) {   // (the film at its moment: on a live stage once the document is let go)
            if (s.later) s.later->push_back([&s, film = t.film, since = (float)lies[i].since] { video_at(s, film, since); });
            else video_at(s, t.film, (float)lies[i].since);
        }
        if (t.film && t.film->look) {   // (its own look, in its material, as ThorVG's effects give a picture its: blurred (in points of the film), tinted, tritoned)
            DrawnFx fx;
            std::string why;
            doc_fx(d, thing, now, false, 1.0, fx, why);
            const std::string way = doc_word(d, thing, "blur_direction");
            const float blur = (float)doc_number(d, thing, "blur", now, 0);
            const float4 grade[6] = {{way == "vertical" ? 0.f : blur / (float)t.film->w, way == "horizontal" ? 0.f : blur / (float)t.film->h, fx.tint, fx.tritone},
                                     {fx.tints[0], 0.f}, {fx.tints[1], 0.f}, {fx.tones[0], 0.f}, {fx.tones[1], 0.f}, {fx.tones[2], 0.f}};
            t.film->look->setParameter("grade", grade, 6);
        }
        if (t.solid && !t.solid->moves.empty()) solid_at(t.solid, doc_word(d, thing, "animation"), lies[i].since * doc_number(d, thing, "speed", 0, 1));   // (its own movement, from when it comes on)
        if (t.drawn && t.drawn->seconds > 0.f) picture_at(s, t.drawn, std::fmod((float)lies[i].since, t.drawn->seconds));
        if (t.drawn && !t.drawn->effects.empty()) {   // (its effects as they are now: those that move are drawn again)
            DrawnFx fx;
            std::string why;
            if (doc_fx(d, thing, now, false, t.units, fx, why)) fx_at(s, t.drawn, fx);
        }
        if (t.drawn && t.drawn->shape) trim_at(s, t.drawn, (float)doc_number(d, thing, "trim", now, 0), (float)doc_number(d, thing, "trim_begin", now, 0), (float)doc_number(d, thing, "trim_offset", now, 0));
        if (t.drawn && t.drawn->cut) {   // (a shape in triangles: bent in its look, and its copies' own looks, nothing drawn again)
            const auto num = [&](const char* name, double otherwise) { return (float)doc_number(d, thing, name, now, otherwise); };
            const float deg = (float)(TAU / 360.0), since = (float)lies[i].since;
            float4 bends[7] = {{num("warp", 0), num("warp_width", 0.5), num("warp_speed", 1) * since, num("warp_angle", 0) * deg},
                                     {num("bend", 0) * deg, doc_word(d, thing, "bend_direction") == "vertical" ? 1.f : 0.f, num("twist", 0) * deg, num("pinch", 0)},
                                     {num("skew", 0) * deg, num("skew_axis", 0) * deg, num("squash", 0), 0.5f * t.drawn->wideBy},
                                     {num("wiggle", 0), num("wiggle_detail", 3), num("wiggle_speed", 1) * since, num("seed", 0) * 7.31f},
                                     {0.f, 0.f, num("stretch", 0), 0.f},
                                     {num("pin_tl_x", 0), num("pin_tl_y", 0), num("pin_tr_x", 0), num("pin_tr_y", 0)},
                                     {num("pin_bl_x", 0), num("pin_bl_y", 0), num("pin_br_x", 0), num("pin_br_y", 0)}};
            if (bends[4].z != 0.f) {   // (how it moves: where its middle was a sixtieth of a second ago, seen as it is now turned and sized)
                const math::mat4 was = doc_place(d, thing, now - 1.0 / 60.0), is = doc_place(d, thing, now);
                const math::float4 back = inverse(is) * was * math::float4{0, 0, 0, 1};
                bends[4].x = (float)(-60.0 * back.x), bends[4].y = (float)(-60.0 * back.y);
            }
            t.drawn->look->setParameter("bends", bends, 7);
            if (t.drawn->edgeLook) t.drawn->edgeLook->setParameter("bends", bends, 7);
            cut_at(s, t.drawn, float3{num("trim", 0), num("trim_begin", 0), num("trim_offset", 0)},   // (its path anew, if it is told otherwise: cut again)
                   {num("round", 0), num("bloat", 0), num("zigzag", 0), num("zigzag_ridges", 5), num("zigzag_smooth", 0), num("offset", 0)});
            for (const Stage::Made& made : s.made)
                if (made.of == t.drawn->sheet)
                    if (const auto own = d.copyLooks.find(made.thing.getId()); own != d.copyLooks.end()) own->second->setParameter("bends", bends, 7);
        }
        if (t.drawn && t.drawn->shape)
            warp_at(s, t.drawn, (float)doc_number(d, thing, "warp", now, 0), (float)doc_number(d, thing, "warp_width", now, 0.5), (float)doc_number(d, thing, "warp_angle", now, 0),
                    (float)(doc_number(d, thing, "warp_speed", now, 1) * lies[i].since));
        if (t.drawn && t.drawn->shape)
            bend_at(s, t.drawn, (float)doc_number(d, thing, "round", now, 0), (float)doc_number(d, thing, "bloat", now, 0), (float)doc_number(d, thing, "zigzag", now, 0),
                    (float)doc_number(d, thing, "zigzag_ridges", now, 5), (float)doc_number(d, thing, "zigzag_smooth", now, 0), (float)doc_number(d, thing, "offset", now, 0));
        const mat4f place = mat4f(doc_place(d, thing, now));
        const bool staggered = doc_staggered(d, thing, now);   // (each copy, the first too, as late as its order says, waving as its place in the wave says)
        const std::vector<math::mat4> joins = doc_number(d, thing, "connect", 0, 0) != 0.0 ? doc_joins(d, thing, now) : std::vector<math::mat4>{};   // (a joining thing: its copies lie between another's)
        if (t.solid) solid_to(s, t.solid, place);
        else if (doc_number(d, thing, "connect", 0, 0) != 0.0) move(s, m, [&](int nth, const mat4f&) { return (size_t)nth < joins.size() ? mat4f(joins[(size_t)nth]) : mat4f::scaling(float3{0.f}); });
        else {
            std::vector<float> far;   // (how far off each copy is from the eye: those further off drawn first, within the thing's own place)
            const mat4f eye = mat4f(s.camera->getViewMatrix());
            move(s, m, [&](int nth, const mat4f&) {
                const mat4f at = nth || staggered ? mat4f(doc_place(d, thing, now, 0, nth, staggered)) : place;
                if ((size_t)nth >= far.size()) far.resize((size_t)nth + 1, 0.f);
                far[(size_t)nth] = -(eye * at[3]).z;
                return at;
            });
            if (far.size() > 1 && !lies[i].solid && t.lies >= 0) {
                std::vector<size_t> order(far.size());
                for (size_t k = 0; k < order.size(); ++k) order[k] = k;
                std::stable_sort(order.begin(), order.end(), [&](size_t a, size_t b) { return far[a] > far[b]; });
                std::vector<uint16_t> place_of(far.size());
                for (size_t k = 0; k < order.size(); ++k) place_of[order[k]] = (uint16_t)(k * 127 / std::max<size_t>(1, order.size() - 1));
                auto& faces = s.engine->getRenderableManager();
                for (const Stage::Made& made : s.made) {
                    if (made.of != m || made.nth < 0 || (size_t)made.nth >= place_of.size()) continue;
                    const auto one = faces.getInstance(made.thing);
                    for (size_t part = 0; part < faces.getPrimitiveCount(one); ++part) faces.setBlendOrderAt(one, part, (uint16_t)(t.lies * 128 + place_of[(size_t)made.nth]));
                }
            }
        }
        if (MaterialInstance* look = t.film ? t.film->look : t.drawn ? t.drawn->look : nullptr) look->setParameter("opacity", (float)doc_number(d, thing, "opacity", now, 1));
        if (t.drawn && t.drawn->edgeLook) t.drawn->edgeLook->setParameter("opacity", (float)doc_number(d, thing, "opacity", now, 1));
        if (MaterialInstance* main = t.film ? t.film->look : t.drawn ? t.drawn->look : nullptr) {   // (its copies coloured each their own, along a ramp in their order: a look of its own each, told what the thing's is)
            std::vector<float3> ramp;
            std::istringstream said(doc_word(d, thing, "copy_fill"));
            for (std::string one; said >> one;) { if (one == "none") break; const long rgb = std::strtol(one.c_str(), nullptr, 16); ramp.push_back(Color::toLinear<ACCURATE>(sRGBColor{float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f})); }
            if (ramp.size() == 1) ramp.push_back(ramp[0]);   // (one colour: every copy in it)
            const std::vector<float2>& letters = t.drawn ? t.drawn->letters : std::vector<float2>{};
            const auto scripted = d.scripted.find(thing);   // (a script's copies each as clear as it says)
            const bool clears = scripted != d.scripted.end() && std::any_of(scripted->second.copies.begin(), scripted->second.copies.end(), [](const ScriptCopy& c) { return c.opacity != 1.0; });
            if (ramp.size() < 2 && letters.empty() && !clears) doc_copy_looks_off(d, m, main);
            else {
                auto& faces = s.engine->getRenderableManager();
                const int n = doc_members(d, thing);
                for (const Stage::Made& made : s.made) {
                    if (made.of != m || made.matte) continue;
                    MaterialInstance*& own = d.copyLooks[made.thing.getId()];
                    if (!own) own = MaterialInstance::duplicate(main), faces.setMaterialInstanceAt(faces.getInstance(made.thing), 0, own);
                    if (ramp.size() >= 2 && doc_word(d, thing, "copy_fill_way") == "cycle") own->setParameter("wash", float4{ramp[(size_t)made.nth % ramp.size()], 1.f});   // (one each in turn)
                    else if (ramp.size() >= 2) {
                        const float along = n > 1 ? (float)made.nth / (float)(n - 1) * (float)(ramp.size() - 1) : 0.f;
                        const size_t i = std::min((size_t)along, ramp.size() - 2);
                        own->setParameter("wash", float4{mix(ramp[i], ramp[i + 1], along - (float)i), 1.f});
                    } else own->setParameter("wash", t.drawn ? t.drawn->wash : float4{0.f});   // (its own: a shape cut in triangles is filled by it)
                    if ((size_t)made.nth < letters.size()) own->setParameter("window", float4{letters[(size_t)made.nth].x, letters[(size_t)made.nth].y, 1.f, 0.f});   // (its own letter)
                    const double clear = scripted != d.scripted.end() && (size_t)made.nth < scripted->second.copies.size() ? scripted->second.copies[(size_t)made.nth].opacity : 1.0;
                    own->setParameter("opacity", (float)(doc_number(d, thing, "opacity", now, 1) * std::clamp(clear, 0.0, 1.0)));
                    own->setParameter("glowing", (float)doc_number(d, thing, "emissive", now, 0));
                }
            }
        }
        const float shine = (float)doc_number(d, thing, "emissive", now, 0);   // (how much it lights itself now: a flat thing's sheet, a solid's own faces)
        doc_looks(s, t, [&](MaterialInstance* look) { if (look->getMaterial()->hasParameter("glowing")) look->setParameter("glowing", shine); });
    }
    // Soft shadows (Filament's PCSS) on the stage's own view while any light that is on says so; else its crisp ones.
    {
        bool soft = false;
        for (const Lies& one : lies) soft = soft || (one.on && !one.shown->light.isNull() && doc_number(d, one.thing, "shadow_softness", 0, 0) > 0.0);
        s.view->setShadowType(soft ? View::ShadowType::PCSS : View::ShadowType::PCF);
    }
    // The lights: in every scene of the stage while each is on (the stage's own and every picture laid over it), where
    // it is, shining the way it is turned (ahead of it), as strong as it is now; 1 as bright as the stage's own light.
    for (const Lies& one : lies) {
        const utils::Entity e = one.shown->light;
        if (e.isNull()) continue;
        std::vector<Scene*> all{s.scene};
        for (View* v : s.layers) all.push_back(v->getScene());
        for (Scene* sc : all) one.on ? (sc->hasEntity(e) ? void() : sc->addEntity(e)) : sc->remove(e);
        if (!one.on) continue;
        auto& lights = s.engine->getLightManager();
        const auto it = lights.getInstance(e);
        const math::mat4 place = doc_place(d, one.thing, now);
        lights.setPosition(it, math::float3((place * math::double4{0, 0, 0, 1}).xyz));
        lights.setDirection(it, normalize(math::float3((place * math::double4{0, 0, -1, 0}).xyz)));
        const float strong = (float)doc_number(d, one.thing, "intensity", now, 1);
        if (lights.getType(it) == LightManager::Type::SUN) lights.setIntensity(it, 60000.f * strong);   // (lux, as the room's own light)
        else lights.setIntensityCandela(it, 60000.f * 36.f * strong);   // (as bright as that six units off: where the stage's own eye stands)
        if (lights.getType(it) == LightManager::Type::SUN)
            lights.setSunAngularRadius(it, (float)doc_number(d, one.thing, "sun_size", now, 1.09) / 2.f), lights.setSunHaloSize(it, (float)doc_number(d, one.thing, "sun_halo", now, 10));
        if (lights.getType(it) == LightManager::Type::FOCUSED_SPOT) {
            const float cone = (float)(doc_number(d, one.thing, "cone", now, 30) * (double)TAU / 360.0);
            lights.setSpotLightCone(it, 0.8f * cone, cone);
        }
    }
    // A clipped thing is seen only inside the frame of the thing before it: Filament's own scissor on its looks, told
    // the rectangle of the picture that the thing before it covers now (turned or leaning: the upright box around it).
    int before = 0;
    for (int thing : lie) {
        const auto found = d.shown.find(thing);
        if (found == d.shown.end()) continue;
        Doc::Shown& t = found->second;
        double box[4] = {0, 0, 0, 0};
        const bool clipped = t.in && before && doc_number(d, thing, "clip", 0, 0) > 0.5 && doc_covers(d, before, now, box);
        // (clipped by a flat thing ThorVG draws, it is cut by that thing's own picture: the kit's matte, After Effects'
        //  alpha matte; by anything else, by the upright box of what that covers)
        const Doc::Shown* const a = clipped ? &d.shown[before] : nullptr;
        const bool shaped = a && a->drawn && (t.drawn || t.film || t.look);   // (a solid too, by its own look: the kit's glass, or its plain look made to be cut)
        Doc::Cut& cut = d.cuts[thing];
        MaterialInstance* const own = t.film ? t.film->look : t.drawn ? t.drawn->look : t.look;
        const std::string how = doc_word(d, thing, "clip_matte") + " " + std::to_string(doc_number(d, thing, "clip_invert", 0, 0)) + " " + std::to_string(doc_number(d, thing, "clip_feather", 0, 0));
        if (shaped && (cut.from != before || cut.by != a->drawn || cut.how != how)) {   // (its matte takes that picture's shape, at that thing's place, cut as it is said)
            if (cut.look) {   // (what cut its look before is let go of first)
                std::erase_if(s.cuts, [&](const Stage::Cut& c) { return c.look == cut.look; });
                cut.look->setParameter("matteUse", float4{0.f});
            }
            if (cut.matte >= 0) {
                Scene* sc = s.mattes[(size_t)cut.matte].scene;
                doc_unshape(s, [sc](const Stage::Made& made) { return sc->hasEntity(made.thing); });
            }
            cut.matte = matte(s, a->drawn->sheet, {mat4f(doc_place(d, before, now))}, a->drawn->texture ? a->drawn->texture : white_point(s), doc_word(d, thing, "clip_matte") == "luma", 1.f, cut.matte);
            cut.from = before, cut.by = a->drawn, cut.look = nullptr, cut.how = how;
        }
        if (shaped && cut.look != own && own && cut.matte >= 0) {
            matted(s, own, cut.matte, doc_number(d, thing, "clip_invert", 0, 0) < 0.5, (float)doc_number(d, thing, "clip_feather", 0, 0)), cut.look = own;
            if (t.drawn && t.drawn->edgeLook) matted(s, t.drawn->edgeLook, cut.matte, doc_number(d, thing, "clip_invert", 0, 0) < 0.5, (float)doc_number(d, thing, "clip_feather", 0, 0));   // (its edge too)
        }   // (its look, or its look made again, is cut by it)
        if (!shaped && cut.look) {   // (no longer cut so: the matte is let go of)
            std::erase_if(s.cuts, [&](const Stage::Cut& c) { return c.look == cut.look; });
            if (cut.look == own) cut.look->setParameter("matteUse", float4{0.f});
            cut.look = nullptr, cut.from = 0, cut.by = nullptr;
            Scene* sc = s.mattes[(size_t)cut.matte].scene;
            doc_unshape(s, [sc](const Stage::Made& made) { return sc->hasEntity(made.thing); });
        }
        if (shaped) { if (t.clipped) doc_looks(s, t, [](MaterialInstance* look) { look->unsetScissor(); }), t.clipped = false; }
        else if (clipped || t.clipped) {
            const uint32_t left = (uint32_t)std::clamp(box[0], 0.0, (double)s.w), bottom = (uint32_t)std::clamp((double)s.h - (box[1] + box[3]), 0.0, (double)s.h);
            doc_looks(s, t, [&](MaterialInstance* look) { if (clipped) look->setScissor(left, bottom, (uint32_t)std::lround(std::max(box[2], 0.0)), (uint32_t)std::lround(std::max(box[3], 0.0))); else look->unsetScissor(); });
            t.clipped = clipped;
        }
        if (t.in) before = thing;
    }
    // An After Effects picture's 3D layers: each at the moment its picture is at, where its pose puts it over that
    // picture's sheet as the stage has it now (the composition's corner at the sheet's top left, a pixel a share of its height).
    for (auto& [thing, t] : d.shown) {
        if (t.deep.empty() || !t.drawn || !t.motion) continue;
        const double since = now - doc_number(d, thing, "start", 0, 0);
        const float sec = t.drawn->seconds > 0.f ? std::fmod((float)std::max(since, 0.0), t.drawn->seconds) : 0.f;
        const float frame = t.drawn->fromFrame + std::min(sec / std::max(t.drawn->seconds, 1e-6f) * t.drawn->frames, std::max(t.drawn->frames - 1.f, 0.f));
        motion_at(*t.motion, frame);
        mat4f sheetAt;
        bool found = false;
        auto& places = s.engine->getTransformManager();
        for (const Stage::Made& made : s.made)
            if (made.of == t.drawn->sheet && !made.matte && made.nth == 0) { sheetAt = places.getWorldTransform(places.getInstance(made.thing)); found = true; break; }
        const float px = 1.f / t.motion->h;
        const float3 corner{-0.5f * t.motion->w * px, 0.5f, 0.f}, right{px, 0.f, 0.f}, down{0.f, -px, 0.f};
        for (const auto& deep : t.deep) {
            layer_at(s, deep.sheet, frame);
            tvg::Matrix flat;
            float seen = 0.f;
            const bool there = t.in && found && motion_flat(*t.motion, deep.id, &flat, &seen) && seen > 0.f;
            const mat4f at = motion_built(*t.motion, deep.id);
            move(s, deep.sheet->sheet, [&](int, const mat4f&) {
                return there ? sheetAt * motion_in_scene(at, corner, right, down) * mat4f::translation(float3{(deep.box.x + 0.5f * deep.box.z) * px, (deep.box.y + 0.5f * deep.box.w) * px, 0.f}) * mat4f::scaling(float3{1, -1, 1})
                             : mat4f::scaling(0.f);
            });
        }
    }
}

// The stage made to match the document: a thing the document has and the stage has not is made (by the kit's own
// call for its kind), one the stage has and the document has not is taken off, one whose making is told otherwise now
// is made again. false: a thing could not be made (d.trouble says which, and why).
// All that a thing's making goes by: one changed, the thing is made again (doc_match), carried by a hand too (doc_carry).
static const char* const doc_making = "'kind','file','shape','blend','keyed','space','begin','duration','speed','text','fill','wide','points','inner','round','bloat','zigzag','offset','warp','clip','stroke','stroke_width','align','blur','shadow','shadow_distance','shadow_angle','shadow_blur','shadow_opacity','glow','glow_blur','glow_opacity','copies','tint','tint_black','tint_white','tritone','tritone_shadow','tritone_midtone','tritone_highlight','letter_spacing','line_spacing','italic','cap','join','dash','dash_offset','metallic','roughness','reflectance','clear_coat','clear_coat_roughness','falloff','shadows','shadow_softness','contact_shadows','sheen','anisotropy','shading','thickness','light_group','script','gradient','gradient_angle','gradient_scale','gradient_spread','wrap','wrap_at','split','ior','dispersion','swim','dark','vivid','marker'";
static bool doc_match(Doc& d) {
    Stage& s = *d.stage;
    std::map<int, std::string> told;   // every thing of the document, and all that its making goes by
    doc_rows(d, sqlite3_mprintf("select thing, group_concat(name || '=' || value, ' ') from (select * from fact where thing > 0 and name in (%s) order by thing, name) group by thing", doc_making),
             [&](sqlite3_stmt* row) { told[sqlite3_column_int(row, 0)] = (const char*)sqlite3_column_text(row, 1); });
    if (const auto held = told.find(d.held.thing); held != told.end())   // (what a hand carries of its making shows too: it is made again as it would be)
        for (const auto& [name, is] : d.held.tried)
            if (strstr(doc_making, ("'" + name + "'").c_str())) {   // (one that moves is drawn again each moment anyway (doc_moment): made again only as it comes on or goes off)
                bool moves = false;
                for (const DocValue& one : doc_values) moves = moves || (one.moves && name == one.name);
                const auto from = d.held.from.find(name);
                if (!moves) held->second += " " + name + "~" + std::to_string(is);
                else if (from != d.held.from.end() && (from->second != 0.0) != (is != 0.0)) held->second += " " + name + "~" + (is != 0.0 ? "on" : "off");
            }
    for (auto it = d.shown.begin(); it != d.shown.end();) {
        const auto still = told.find(it->first);
        if (still != told.end() && still->second == it->second.of) { ++it; continue; }
        if (it->second.scene && it->second.scene != s.scene)   // (out of the scene of its run first: the kit takes things out of the stage's own only)
            doc_entities(s, it->second, [&](utils::Entity e) { it->second.scene->remove(e); });
        if (const Mesh* m = it->second.film ? it->second.film->sheet : it->second.drawn ? it->second.drawn->sheet : it->second.shape)   // (and the shape it cuts a matte by: done away with first)
            doc_unshape(s, [m](const Stage::Made& made) { return made.of == m; });
        if (it->second.film) video_off(s, it->second.film);
        if (const Mesh* m = it->second.film ? it->second.film->sheet : it->second.drawn ? it->second.drawn->sheet : nullptr)   // (its copies' own looks first)
            doc_copy_looks_off(d, m, it->second.film ? it->second.film->look : it->second.drawn->look);
        if (it->second.drawn) picture_off(s, it->second.drawn);
        for (const auto& deep : it->second.deep) picture_off(s, deep.sheet);
        if (it->second.solid) solid_off(s, it->second.solid);
        if (!it->second.light.isNull()) {   // (out of every scene, and done away with)
            s.scene->remove(it->second.light);
            for (View* v : s.layers) v->getScene()->remove(it->second.light);
            s.engine->getLightManager().destroy(it->second.light), utils::EntityManager::get().destroy(it->second.light);
        }
        if (it->second.shape) take_off(s, it->second.shape);
        it = d.shown.erase(it);
    }
    bool all = true;
    for (const auto& [thing, of] : told) {
        if (d.shown.count(thing)) continue;
        Doc::Shown made{of};
        const std::string kind = doc_word(d, thing, "kind"), file = doc_word(d, thing, "file"), laid = doc_word(d, thing, "blend"), shape = doc_word(d, thing, "shape");
        const char* lay = laid.empty() || laid == "normal" ? nullptr : laid.c_str();
        std::string why = "it could not be read";
        const std::string fill = doc_word(d, thing, "fill");
        // (a fill is a colour, six digits; or, of a text or a flat shape, two colours or more: a ramp down it)
        std::vector<float3> ramp;
        long rgb = -1;
        {
            std::istringstream said(fill);
            for (std::string one; said >> one;) {
                const long c = one.size() == 6 && one.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(one.c_str(), nullptr, 16) : -1;
                if (c < 0) { ramp.clear(), rgb = -1; break; }
                if (ramp.empty()) rgb = c;
                ramp.push_back(float3{(float)((c >> 16) & 255), (float)((c >> 8) & 255), (float)(c & 255)} / 255.f);
            }
        }
        const float3 colour = float3{(float)((rgb >> 16) & 255), (float)((rgb >> 8) & 255), (float)(rgb & 255)} / 255.f;
        // (an edge, ThorVG's stroke: its colour said as a fill is, and its width as a share of the thing's own height,
        //  from points of the picture at size 1 where things stand at first)
        const std::string stroke = doc_word(d, thing, "stroke");
        const long edgeRgb = stroke.size() == 6 && stroke.find_first_not_of("0123456789abcdefABCDEF") == std::string::npos ? std::strtol(stroke.c_str(), nullptr, 16) : -1;
        const math::double4 up = s.camera->getProjectionMatrix() * s.camera->getViewMatrix() * math::double4{0, 1, 0, 1};
        const float4 edge = edgeRgb < 0 ? float4{0.f} : float4{float3{(float)((edgeRgb >> 16) & 255), (float)((edgeRgb >> 8) & 255), (float)(edgeRgb & 255)} / 255.f, (float)(doc_number(d, thing, "stroke_width", 0, 0) / (0.5 * s.h * up.y / up.w))};
        if (!stroke.empty() && edgeRgb < 0) why = "a stroke is six digits, as ff3366";
        const double units = 0.5 * s.h * up.y / up.w;   // (points of the picture to one unit of the stage, where things stand at first)
        DrawnFx fx;
        if (ramp.size() > 1) fx.ramp = ramp;
        fx.radial = doc_word(d, thing, "gradient") == "radial", fx.rampAngle = (float)doc_number(d, thing, "gradient_angle", 0, 0), fx.rampScale = (float)doc_number(d, thing, "gradient_scale", 0, 1);
        fx.spread = doc_word(d, thing, "gradient_spread") == "reflect" ? 1 : doc_word(d, thing, "gradient_spread") == "repeat" ? 2 : 0;
        fx.copies = (int)doc_number(d, thing, "copies", 0, 1);
        fx.wrapWidth = (float)doc_number(d, thing, "wrap_at", 0, 0);
        const std::string split = doc_word(d, thing, "split"), script = doc_word(d, thing, "script");
        fx.split = split == "letters" || (split.empty() && !script.empty() && script != "none");   // (a text given a script moves letter by letter, as Cavalry's are, unless said none)
        { const std::string w = doc_word(d, thing, "wrap"); fx.wrap = w == "character" ? 1 : w == "word" ? 2 : w == "smart" ? 3 : w == "ellipsis" ? 4 : fx.wrapWidth > 0.f ? 2 : 0; }   // (ThorVG's TextWrap; a box said and no way: by words)
        fx.letters = (float)doc_number(d, thing, "letter_spacing", 0, 1), fx.lines = (float)doc_number(d, thing, "line_spacing", 0, 1), fx.italic = (float)doc_number(d, thing, "italic", 0, 0);
        const auto which = [&](const char* name, const char* words) { const std::string w = doc_word(d, thing, name); if (w.empty()) return -1; int n = 0; for (const char* at = words; *at; ++n) { const size_t len = strcspn(at, " "); if (w == std::string(at, len)) return n; at += len + (at[len] ? 1 : 0); } return -1; };
        fx.cap = which("cap", "butt round square"), fx.join = which("join", "miter round bevel");   // (ThorVG's StrokeCap and StrokeJoin, in its order)
        fx.dash = (float)(doc_number(d, thing, "dash", 0, 0) / units), fx.dashFrom = (float)(doc_number(d, thing, "dash_offset", 0, 0) / units);
        bool fxOk = doc_fx(d, thing, 0.0, true, units, fx, why);   // (made as large as its effects reach at their most; each moment tells them as they are then)
        made.units = units;
        if (doc_among("film picture text shape", kind)) made.space = doc_word(d, thing, "space") == "2.5d" ? 25 : 2;
        if (kind == "camera") made.camera = true;   // (nothing of it is seen: it is how the rest is)
        else if (kind == "light") {   // (Filament's own light, made as the document says; placed and told how strong at each moment)
            const LightManager::Type type = file == "sun" ? LightManager::Type::SUN : file == "spot" ? LightManager::Type::FOCUSED_SPOT : LightManager::Type::POINT;
            if (!doc_among("sun point spot", file.c_str())) why = "a light is a sun, a point or a spot";
            else {
                made.light = utils::EntityManager::get().create();
                LightManager::ShadowOptions shade;   // (a finer map than as it comes, so the edge is not stepped; a lamp as large as its softness says)
                shade.mapSize = 2048;
                const float soft = (float)doc_number(d, thing, "shadow_softness", 0, 0);
                shade.screenSpaceContactShadows = doc_number(d, thing, "contact_shadows", 0, 0) > 0.5;
                if (soft > 0.f) shade.shadowBulbRadius = type == LightManager::Type::SUN ? 0.05f * soft : 0.5f * soft;
                LightManager::Builder(type).color(Color::toLinear<ACCURATE>(sRGBColor{fill.empty() ? float3{1.f} : colour})).falloff((float)doc_number(d, thing, "falloff", 0, 10))
                    .castShadows(doc_number(d, thing, "shadows", 0, 0) > 0.5).shadowOptions(shade).build(*s.engine, made.light);
                if (const int group = (int)doc_number(d, thing, "light_group", 0, 0); group > 0) {
                    auto& lights = s.engine->getLightManager();
                    lights.setLightChannel(lights.getInstance(made.light), 0, false), lights.setLightChannel(lights.getInstance(made.light), (unsigned)group, true);
                }
                made.in = false;   // (in the scenes when it is on)
            }
        }
        else if (shape.empty() && !std::filesystem::is_regular_file(file)) why = "there is no such file";
        else if (kind == "film") {
            const double from = doc_number(d, thing, "begin", 0, 0), length = doc_number(d, thing, "duration", 0, -1), speed = doc_number(d, thing, "speed", 0, 1);
            made.film = video(s, file.c_str(), mat4f{}, 1.f, (float)from, length < 0 ? -1.f : (float)(from + length * std::fabs(speed)), (float)speed, lay);
            if (!made.film) why = video_trouble;
            else if (const double keyed = doc_number(d, thing, "keyed", 0, -1); keyed != 0.0) key_own(made.film, keyed > 0.5);   // (not said: keyed if it has a ground of its own)
            if (made.film) s.playing.erase(made.film);   // (the document tells it its moment, and when its sound comes on)
            if (made.film && made.film->sound) s.sounding[made.film] = [&d, thing, v = made.film, file] {   // (counted from where the film being written begins: a sound already going then is heard from that far into it)
                const double on = doc_number(d, thing, "start", 0, 0) - d.written_from, lasts = doc_number(d, thing, "duration", 0, -1), late = std::max(0.0, -on);
                return Stage::Sound{file, v->from, v->to, v->speed, late, std::max(0.0, on), lasts < 0.0 ? lasts : std::max(1e-3, lasts - late)};
            };
        } else if (kind == "picture") {
            if (fxOk && (made.drawn = picture(s, file.c_str(), mat4f{}, 1.f, float3{-1.f}, true, lay, fx, doc_word(d, thing, "marker").c_str()))) s.playing.erase(made.drawn);
            if (made.drawn && made.drawn->lottie && !picture_deep(s, made.drawn).empty()) {   // (its 3D layers stood up where they are, as After Effects shows them)
                made.motion = std::make_shared<Motion>();
                if (motion_open(*made.motion, file.c_str()))
                    for (uint32_t id : made.drawn->deep) {
                        float4 box;
                        if (Drawn* own = layer_picture(s, file.c_str(), id, 1.f / made.motion->h, &box)) made.deep.push_back({id, own, box});
                    }
            }
        } else if (kind == "text") {   // (words in a font: said by `say N text ...`; until then, the font's own name)
            const std::string words = doc_word(d, thing, "text");
            if (!fill.empty() && rgb < 0) why = "a text's fill is six digits, as ff3366";
            else if ((stroke.empty() || edgeRgb >= 0) && fxOk) made.drawn = type(s, words.empty() ? std::filesystem::path(file).stem().string().c_str() : words.c_str(), file.c_str(), mat4f{}, 1.f, fill.empty() ? float3{0x11 / 255.f} : colour, lay, edge, (float)doc_number(d, thing, "align", 0, 0), fx);
        } else if (kind == "shape") {   // (a flat shape in one colour: ThorVG's own rectangle or circle)
            if (!fill.empty() && fill != "none" && rgb < 0) why = "a shape's fill is six digits, as ff3366, or none";
            else if ((!stroke.empty() && edgeRgb < 0) || !fxOk) {}
            else if (shape == "solid") {   // (a flat colour as large as the stage, where it stands at first: as wide to its height, and as high in the stage's units)
                const float high = (float)(2.0 * up.w / up.y);
                made.drawn = flat(s, "rect", (float)s.w / (float)s.h, mat4f{}, high, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, lay, float4{edge.xyz, edge.w / high}, fx.over(high));
            } else {   // (a line is long and thin as it comes: 25 times as long as it is thick, a tenth of a unit thick)
                const bool line = shape == "line", wave = shape == "wave";   // (a wave: a line waving, ten times as long as it is high)
                fx.hollow = fill == "none";   // (filled with nothing: its edge alone, a frame)
                const float high = line ? 0.04f : wave ? 0.3f : 1.f;
                // (its outline alone, in one colour: cut into triangles and drawn by Filament as the scene's other things
                //  are, flat_mesh; with an edge, a ramp, a dash, effects, a trim or a bend made on the CPU: ThorVG's picture)
                const bool plain = !wave && shape != "null" && fx.ramp.size() < 2 && fx.blur <= 0.f && fx.shadow.w <= 0.f && fx.glow.w <= 0.f && fx.tint <= 0.f && fx.tritone <= 0.f;
                if (plain)
                    made.drawn = flat_mesh(s, shape.c_str(), (float)doc_number(d, thing, "wide", 0, line ? 25 : 1), mat4f{}, high, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, lay,
                                           float4{edge.xyz, edge.w / high}, fx.over(high), (int)doc_number(d, thing, "points", 0, shape == "star" ? 5 : 6), (float)doc_number(d, thing, "inner", 0, 0.5), (float)doc_number(d, thing, "round", 0, 0));
                else made.drawn = flat(s, shape.c_str(), (float)doc_number(d, thing, "wide", 0, line ? 25 : wave ? 10 : 1), mat4f{}, high, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, lay,
                                  float4{edge.xyz, edge.w / high}, fx.over(high),
                                  (int)doc_number(d, thing, "points", 0, shape == "star" ? 5 : wave ? 4 : 6), (float)doc_number(d, thing, "inner", 0, 0.5), (float)doc_number(d, thing, "round", 0, 0));
            }
        } else if (kind == "solid") {   // (one of the kit's shapes, or a file; its fill is its surface: a colour, or the material glass)
            const bool glass = fill == "glass";
            const auto glassy = [&](MaterialInstance* look) {   // (the kit's glass as the document says it; not said, as it comes)
                look->setParameter("ior", (float)doc_number(d, thing, "ior", 0, 1.5)), look->setParameter("split", (float)doc_number(d, thing, "dispersion", 0, 0.1));
                look->setParameter("swim", (float)doc_number(d, thing, "swim", 0, 0.5)), look->setParameter("dark", (float)doc_number(d, thing, "dark", 0, 4));
                look->setParameter("vivid", (float)doc_number(d, thing, "vivid", 0, 1.3));
            };
            const Surface surface{(float)doc_number(d, thing, "metallic", 0, 0.08), (float)doc_number(d, thing, "roughness", 0, 0.6), (float)doc_number(d, thing, "reflectance", 0, 0.5),
                                  (float)doc_number(d, thing, "clear_coat", 0, 0), (float)doc_number(d, thing, "clear_coat_roughness", 0, 0), (float)doc_number(d, thing, "emissive", 0, 0),
                                  (float)doc_number(d, thing, "sheen", 0, 0), (float)doc_number(d, thing, "anisotropy", 0, 0), (float)doc_number(d, thing, "thickness", 0, 0.5)};
            const std::string shading = doc_word(d, thing, "shading");
            const char* model = doc_number(d, thing, "clip", 0, 0) > 0.5 && (shading.empty() || shading == "lit") ? "floor_lit_matte.filamat" : surface_model(shading);   // (clipped: one a matte can cut)
            if (!fill.empty() && !glass && (rgb < 0 || ramp.size() > 1)) why = "a solid's fill is six digits, as ff3366, or glass";
            else if (!shape.empty()) {
                made.shape = shape == "gem" ? gem(0.5f) : shape == "cube" ? cube(0.1f) : shape == "ball" ? sphere(0.5f) : torus(0.37f, 0.13f);
                MaterialInstance* look = glass ? thick_glass(s, made.shape, {mat4f{}}) : add(s, made.shape, model);
                made.look = look;
                if (look && glass) glassy(look);
                if (!look) made.shape = nullptr;
                else if (!glass) plain(look, fill.empty() ? float3{0xd9, 0xd2, 0xc3} / 255.f : colour, surface);
            } else if (glass) {   // (a file's shape as glass: its triangles alone, for the kit's glass, which draws a shape three times)
                MaterialInstance* look = (made.shape = solid_mesh(file.c_str())) ? thick_glass(s, made.shape, {mat4f{}}) : nullptr;
                made.look = look;
                if (!look) made.shape = nullptr; else glassy(look);
            }
            else if ((made.solid = solid(s, file.c_str(), mat4f{}, 1.f)) && rgb >= 0) solid_fill(s, made.solid, colour, surface, model);
            if (made.solid) s.playing.erase(made.solid);   // (the document tells it its moment)
        } else why = std::string("its kind is one of: ") + doc_kinds;
        if (!made.film && !made.drawn && !made.solid && !made.shape && !made.camera && made.light.isNull()) { d.trouble = why + ": the " + kind + " " + std::filesystem::path(file + shape).filename().string() + " could not be made"; all = false; continue; }
        d.shown[thing] = made;
        if (made.solid || made.shape) {   // (a solid thing casts a shadow from a light that throws them, and takes one)
            const int group = (int)doc_number(d, thing, "light_group", 0, 0);
            doc_parts(s, d.shown[thing], [group](RenderableManager& faces, RenderableManager::Instance one) {
                faces.setCastShadows(one, true), faces.setReceiveShadows(one, true), faces.setScreenSpaceContactShadows(one, true);   // (contact shadows: Filament draws them only where the light asks too)
                if (group > 0) faces.setLightChannel(one, (unsigned)group, true);
            });
        }
    }
    doc_moment(d, s.now);
    return all;
}

// How long the document is: to the end of what ends last (a thing with no duration: once through itself); 8 seconds if nothing in it ends.
static double doc_seconds(Doc& d) {
    double all = 0.0;
    for (const auto& [thing, t] : d.shown) {
        const double own = t.film ? (double)t.film->seconds : t.drawn ? (double)t.drawn->seconds : t.solid ? (double)t.solid->seconds : 0.0, lasts = doc_number(d, thing, "duration", 0, own);
        if (lasts > 0.0) all = std::max(all, doc_number(d, thing, "start", 0, 0) + lasts);
    }
    return all > 0.0 ? all : 8.0;
}

// The work area: the stretch of the document that is played round and written as a film (After Effects' own); not
// said, all of it. And the marks: moments of the document with a name (After Effects' composition markers). Both are
// facts of thing 0, the stage itself, and every change of them is a step.
static bool doc_change(Doc& d, char* change, bool later = false);   // (later: a key, whose act need not show it before its end)
static void doc_work(Doc& d, double* from, double* to) {
    const double all = doc_seconds(d);
    double in = 0.0, out = all;
    doc_rows(d, sqlite3_mprintf("select name, value from fact where thing = 0 and name in ('work_in', 'work_out')"),
             [&](sqlite3_stmt* row) { (strcmp((const char*)sqlite3_column_text(row, 0), "work_in") ? out : in) = sqlite3_column_double(row, 1); });
    *from = std::clamp(in, 0.0, std::max(0.0, all - 1e-3));
    *to = std::clamp(out, *from + 1e-3, all);
}
static bool doc_work_is(Doc& d, double from, double to) {
    if (!(from >= 0.0 && to > from + 1e-3 && to <= 1e6)) return d.trouble = "a work area is from one moment to a later one", false;
    return doc_change(d, sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'work_in', 0.0, %!.17g), (0, 'work_out', 0.0, %!.17g) on conflict do update set value = excluded.value", from, to));
}
static bool doc_mark(Doc& d, double at, const char* word) {   // ("": the mark there is taken away)
    if (!(at >= 0.0 && at <= 1e6)) return d.trouble = "a mark is at a moment of the stage: from 0 to 1000000 seconds", false;
    if (strchr(word, '\n')) return d.trouble = "a mark's name is one line", false;
    return doc_change(d, *word ? sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'mark', %!.17g, %Q) on conflict do update set value = excluded.value", at, word)
                               : sqlite3_mprintf("delete from fact where thing = 0 and name = 'mark' and at = %!.17g", at));
}
// The guides: lines across the picture at a place, in points of it (After Effects' composition guides): x, an upright
// line that far from the left; y, a level one that far from the top. Facts of the stage, a change of them a step.
static bool doc_guide(Doc& d, const char* axis, double at, bool on) {
    if (strcmp(axis, "x") && strcmp(axis, "y")) return d.trouble = "a guide is x (upright) or y (level)", false;
    if (!(std::fabs(at) <= 1e5)) return d.trouble = "a guide is within the picture's reach", false;
    return doc_change(d, on ? sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'guide_%s', %!.17g, 1) on conflict do nothing", axis, at)
                            : sqlite3_mprintf("delete from fact where thing = 0 and name = 'guide_%s' and at = %!.17g", axis, at));
}
static std::string doc_guides(Doc& d) {   // (a line each: x or y, a space, where)
    std::string all;
    doc_rows(d, sqlite3_mprintf("select substr(name, 7), at from fact where thing = 0 and name in ('guide_x', 'guide_y') order by name, at"), [&](sqlite3_stmt* row) {
        char line[60];
        std::snprintf(line, sizeof line, "%s %.6g\n", (const char*)sqlite3_column_text(row, 0), sqlite3_column_double(row, 1));
        all += line;
    });
    return all;
}
static std::string doc_marks(Doc& d) {   // (a line each, in time: the moment, a space, the name)
    std::string all;
    doc_rows(d, sqlite3_mprintf("select at, value from fact where thing = 0 and name = 'mark' order by at"), [&](sqlite3_stmt* row) {
        char at[40];
        std::snprintf(at, sizeof at, "%.6g ", sqlite3_column_double(row, 0));
        all += at + std::string((const char*)sqlite3_column_text(row, 1)) + "\n";
    });
    return all;
}

// A document's own stage: how many points wide and high it is (facts of thing 0, which is the stage itself; they are
// not steps: a stage is opened at a size, so a new size is the document's when it is next opened).
// doc_size: read before a stage is opened for the document (false: it does not say, or it is not there yet).
static bool doc_size(const char* path, int* wide, int* high) {
    sqlite3* file = nullptr;
    sqlite3_stmt* rows = nullptr;
    int w = 0, h = 0;
    if (sqlite3_open_v2(path, &file, SQLITE_OPEN_READONLY, nullptr) == SQLITE_OK && sqlite3_prepare_v2(file, "select name, value from fact where thing = 0 and name in ('wide', 'high')", -1, &rows, nullptr) == SQLITE_OK)
        while (sqlite3_step(rows) == SQLITE_ROW) (strcmp((const char*)sqlite3_column_text(rows, 0), "wide") ? h : w) = sqlite3_column_int(rows, 1);
    sqlite3_finalize(rows);
    sqlite3_close(file);
    return w > 0 && h > 0 ? (*wide = w, *high = h, true) : false;
}
// doc_size_words: the change that says it (nullptr: a stage cannot be that size). doc_size_is: a new document told
// the size it was first opened at (no step of undo: there is nothing before it).
static char* doc_size_words(int wide, int high) {
    if (wide < 16 || high < 16 || wide > 8192 || high > 8192 || wide % 2 || high % 2) return nullptr;
    return sqlite3_mprintf("insert into fact(thing, name, at, value) values(0, 'wide', 0.0, %d), (0, 'high', 0.0, %d) on conflict do update set value = excluded.value", wide, high);
}
static bool doc_size_is(sqlite3* file, int wide, int high) {
    char* say = doc_size_words(wide, high);
    const bool ok = say && sqlite3_exec(file, say, nullptr, nullptr, nullptr) == SQLITE_OK;
    sqlite3_free(say);
    return ok;
}

// The document in `path` (made if it is not there), on the stage. nullptr: the file is not a document.
static Doc* doc_open(Stage& s, const char* path) {
    Doc* d = new Doc;
    if (sqlite3_open(path, &d->file) != SQLITE_OK) return nullptr;
    // (a way defined in the document may ask how far a cubic bezier ease is at u, by its two handles, as the nine kinds'
    //  Bezier goes: bezier(u, x1, y1, x2, y2))
    sqlite3_create_function(d->file, "bezier", 5, SQLITE_UTF8 | SQLITE_DETERMINISTIC, nullptr, [](sqlite3_context* c, int, sqlite3_value** v) {
        sqlite3_result_double(c, interp_bezier(interp_clamp(sqlite3_value_double(v[1]), 0.0, 1.0), sqlite3_value_double(v[2]), interp_clamp(sqlite3_value_double(v[3]), 0.0, 1.0), sqlite3_value_double(v[4]), sqlite3_value_double(v[0])));
    }, nullptr, nullptr);
    sqlite3_busy_timeout(d->file, 2000);   // (another program at the same document is waited for)
    // (a key's `way` is how the stretch from it to the next key goes; `way` the table holds the ways defined in this document)
    if (sqlite3_exec(d->file, "create table if not exists fact(thing integer, name text, at numeric, value, way text, primary key(thing, name, at)) without rowid;"
                              "create table if not exists way(name text primary key, goes text) without rowid;"
                              "create table if not exists step(n integer primary key, change blob, undone integer default 0);"
                              "create table if not exists thing(id integer primary key autoincrement);"   // (a thing's number is never given again: SQLite's own)
                              "insert into thing(id) select max(thing) from fact where (select count(*) from thing) = 0 having max(thing) is not null", nullptr, nullptr, nullptr) != SQLITE_OK) return nullptr;
    if (sqlite3_exec(d->file, "select way from fact limit 0", nullptr, nullptr, nullptr) != SQLITE_OK)   // (a document from before stretches had ways)
        sqlite3_exec(d->file, "alter table fact add column way text", nullptr, nullptr, nullptr);
    // (a document from before its moments were numeric: SQLite's session keeps a row whose moment is a whole number
    //  made and then changed in one act as two, if the moment is real, and that step cannot be undone. Made again once)
    bool real = false;
    doc_rows(*d, sqlite3_mprintf("select 1 from pragma_table_info('fact') where name = 'at' and type = 'REAL'"), [&](sqlite3_stmt*) { real = true; });
    if (real)
        sqlite3_exec(d->file, "begin; create table fact_numeric(thing integer, name text, at numeric, value, way text, primary key(thing, name, at)) without rowid;"
                              "insert into fact_numeric select thing, name, at, value, way from fact; drop table fact; alter table fact_numeric rename to fact; commit", nullptr, nullptr, nullptr);
    d->stage = &s;
    s.playing[d] = [d](double now) { doc_moment(*d, now); };
    if (!doc_match(*d)) std::printf("%s\n", d->trouble.c_str());
    return d;
}

// One change to the document (in SQLite's own words), kept as one step of undo; the stage shows it at once. If the
// stage cannot show it, it is not made: the document and the stage stay as they were.
static void doc_act(Doc& d) {
    if (d.act++) return;
    sqlite3_exec(d.file, "begin", nullptr, nullptr, nullptr);
    sqlite3session_create(d.file, "main", &d.acting);
    sqlite3session_attach(d.acting, "fact");
    sqlite3session_attach(d.acting, "way");
    d.act_ok = true;
}
static bool doc_match(Doc& d);
static void doc_forget(Doc& d);
// What a step changed, as a shell is told it: each value (thing, name) its changeset touches, and whether a thing came
// or went (its kind written or taken away). The changeset is SQLite's own record of the step: nothing is missed.
static void doc_note(Doc& d, int size, void* changeset) {
    sqlite3_changeset_iter* it = nullptr;
    if (sqlite3changeset_start(&it, size, changeset) != SQLITE_OK) return;
    while (sqlite3changeset_next(it) == SQLITE_ROW) {
        const char* table = nullptr;
        int columns = 0, op = 0, indirect = 0;
        sqlite3changeset_op(it, &table, &columns, &op, &indirect);
        if (!table || strcmp(table, "fact")) { d.things_changed = true; continue; }   // (the ways defined: anything may go another way)
        sqlite3_value *thing = nullptr, *name = nullptr;
        (op == SQLITE_INSERT ? sqlite3changeset_new : sqlite3changeset_old)(it, 0, &thing);
        (op == SQLITE_INSERT ? sqlite3changeset_new : sqlite3changeset_old)(it, 1, &name);
        if (!thing || !name) continue;
        const std::string n = (const char*)sqlite3_value_text(name);
        d.changed.insert({sqlite3_value_int(thing), n});
        if (n == "kind" && op != SQLITE_UPDATE) d.things_changed = true;
    }
    sqlite3changeset_finalize(it);
}
static bool doc_acted(Doc& d) {
    if (d.act == 1 && d.act_unmatched) {   // (the act's changes shown once, all of them: if the stage cannot, none of them is made)
        d.act_unmatched = false;
        doc_forget(d);
        if (d.act_ok && !doc_match(d)) d.act_ok = false;
    }
    if (--d.act) return d.act_ok;
    int size = 0;
    void* changed = nullptr;
    sqlite3session_changeset(d.acting, &size, &changed);
    sqlite3session_delete(d.acting);
    d.acting = nullptr;
    if (d.act_ok && size) doc_note(d, size, changed);
    if (d.act_ok && size) {   // (what was undone and not done again is forgotten: a new step follows the last one that stands)
        sqlite3_exec(d.file, "delete from step where undone", nullptr, nullptr, nullptr);
        sqlite3_stmt* keep = nullptr;
        sqlite3_prepare_v2(d.file, "insert into step(change) values(?)", -1, &keep, nullptr);
        sqlite3_bind_blob(keep, 1, changed, size, SQLITE_TRANSIENT);
        sqlite3_step(keep);
        sqlite3_finalize(keep);
    }
    sqlite3_free(changed);
    sqlite3_exec(d.file, d.act_ok ? "commit" : "rollback", nullptr, nullptr, nullptr);
    if (!d.act_ok) { doc_forget(d); const std::string why = d.trouble; doc_match(d); d.trouble = why; }   // (the stage as the document was before the act)
    return d.act_ok;
}
// One change to the document (in SQLite's own words), an act by itself unless it is made within one; the stage shows
// it at once. If the stage cannot show it, it is not made, and no other change of its act is.
static bool doc_change(Doc& d, char* change, bool later) {
    doc_act(d);
    bool ok = d.act_ok;   // (a change before it in this act was refused: d.trouble still says why)
    if (ok && d.writing) ok = false, d.trouble = "the document is being written as a film: it is not changed until that is done";
    if (ok && !(ok = sqlite3_exec(d.file, change, nullptr, nullptr, nullptr) == SQLITE_OK)) d.trouble = sqlite3_errmsg(d.file);
    sqlite3_free(change);
    if (ok && later && d.act > 1) d.act_unmatched = true;   // (a key within a larger act, one of a thousand: matched once when the act ends)
    else if (ok) {
        doc_forget(d);
        ok = doc_match(d);
    }
    if (!ok) d.act_ok = false;
    return doc_acted(d) && ok;
}

// The last step undone (back = true), or the last undone step done again. false: there is none.
static bool doc_step(Doc& d, bool back) {
    int n = 0;
    std::vector<uint8_t> change;
    doc_rows(d, sqlite3_mprintf(back ? "select n, change from step where not undone order by n desc limit 1" : "select n, change from step where undone order by n limit 1"), [&](sqlite3_stmt* row) {
        n = sqlite3_column_int(row, 0);
        change.assign((const uint8_t*)sqlite3_column_blob(row, 1), (const uint8_t*)sqlite3_column_blob(row, 1) + sqlite3_column_bytes(row, 1));
    });
    if (!n) return d.trouble = back ? "nothing to undo" : "nothing to do again", false;
    int size = (int)change.size();
    void* apply = change.data();
    if (back) sqlite3changeset_invert((int)change.size(), change.data(), &size, &apply);
    const bool ok = sqlite3changeset_apply(d.file, size, apply, nullptr, [](void*, int, sqlite3_changeset_iter*) { return (int)SQLITE_CHANGESET_ABORT; }, nullptr) == SQLITE_OK;
    if (ok) doc_note(d, size, apply);
    if (back) sqlite3_free(apply);
    if (!ok) return d.trouble = "the step does not fit the document as it is", false;
    doc_rows(d, sqlite3_mprintf("update step set undone = %d where n = %d", back ? 1 : 0, n));
    doc_forget(d);
    return doc_match(d);
}
// The way of the stretch that begins at a key, said back in one form: a kind's name as the table writes it and all
// its numbers, however it was told ("": there is no key of that value at that moment).
static std::string doc_way_of(Doc& d, int thing, const char* name, double at) {
    std::string said;
    doc_rows(d, sqlite3_mprintf("select coalesce(way, 'Linear') from fact where thing = %d and name = %Q and at = %!.17g and " DOC_KEYED, thing, name, at, thing, name),
             [&](sqlite3_stmt* row) { said = (const char*)sqlite3_column_text(row, 0); });
    Interp kind;
    return !said.empty() && interp_said(said, &kind) == 0 ? interp_words(kind) : said;
}

// Whether a call names a thing that is there and tells it something a thing of its kind can be told (if not,
// d.trouble says what it can be told).
static std::string doc_words(Doc& d, int thing, const char* name);
static bool doc_knows(Doc& d, int thing, const char* name = nullptr, const double* number = nullptr, const char* word = nullptr, bool key = false) {
    const std::string kind = doc_word(d, thing, "kind");
    if (kind.empty()) return d.trouble = "there is no thing " + std::to_string(thing), false;
    if (!name) return true;
    const DocValue* v = nullptr;
    for (const DocValue& one : doc_values)
        if (!strcmp(one.name, name) && doc_among(one.kinds, kind)) { v = &one; break; }
    if (!v && !strncmp(name, "js_", 3) && doc_among("picture text shape stage", kind))   // (a knob of the thing's script)
        for (const DocScript* sc : doc_scripts_on(d, thing))
            for (const DocValue& one : sc->values)
                if (!strcmp(one.name, name)) v = &one;
    const auto said = [&](bool moves) {   // (the names it has, or those that move: said only when a call is refused)
        std::string all;
        for (const DocValue& one : doc_values)
            if (doc_among(one.kinds, kind) && (!moves || one.moves)) all += std::string(all.empty() ? "" : ", ") + one.name;
        if (doc_among("picture text shape stage", kind))
            for (const DocScript* sc : doc_scripts_on(d, thing))
                for (const DocValue& one : sc->values)
                    if (!moves || one.moves) all += std::string(all.empty() ? "" : ", ") + one.name;
        return all;
    };
    char range[120];
    if (!v) return d.trouble = "a " + kind + " has no " + name + ": it has " + said(false), false;
    if (number && v->words) return d.trouble = std::string(name) + " is a word, told by say" + (*v->words ? std::string(": ") + v->words : ""), false;
    if (word && !v->words) return d.trouble = std::string(name) + " is a number, told by set", false;
    if (word && *v->words && !doc_among(v->words, word)) return d.trouble = std::string(name) + " is one of: " + v->words, false;
    if (word && *word && (!strcmp(name, "marker") || !strcmp(name, "animation"))) {   // (a name its file has)
        const std::string has = doc_words(d, thing, name);
        if (has.empty()) return d.trouble = std::string("its file has no ") + (!strcmp(name, "marker") ? "markers" : "movements of its own"), false;
        if (!doc_among(has.c_str(), word)) return d.trouble = std::string(name) + " is one of: " + has, false;
    }
    if (word && *word && !strcmp(name, "copy_tall") && strcmp(word, "none")) {   // (numbers, one for each copy in their order)
        std::istringstream all(word);
        for (std::string one; all >> one;) { char* end = nullptr; const double is = std::strtod(one.c_str(), &end); if (*end || !std::isfinite(is) || is < 0.0 || is > 1e3) return d.trouble = "copy_tall is numbers from 0 to 1000, one for each copy, as 1 2.5 0.8", false; }
    }
    if (word && *word && !strcmp(name, "copy_fill") && strcmp(word, "none")) {   // (colours, six digits each; none: not coloured each)
        std::istringstream all(word);
        int n = 0;
        for (std::string one; all >> one; ++n)
            if (one.size() != 6 || one.find_first_not_of("0123456789abcdefABCDEF") != std::string::npos) return d.trouble = "copy_fill is colours of six digits, as ff3366 1b2fd8", false;
        if (n < 1) return d.trouble = "copy_fill is colours of six digits, as ff3366 1b2fd8, or none", false;
    }
    if (word && *word && (doc_among("vignette_color fog_color background", name) || (!strcmp(name, "fill") && kind == "light")))   // (one colour: six digits)
        if (strlen(word) != 6 || strspn(word, "0123456789abcdefABCDEF") != 6) return d.trouble = std::string(name) + " is six digits, as ff3366", false;
    if (number && (!std::isfinite(*number) || *number < v->low || *number > v->high || (doc_among("keyed clip visible solo lock shadows clip_invert", name) && *number != 0.0 && *number != 1.0)))
        return std::snprintf(range, sizeof range, doc_among("keyed clip visible solo lock shadows clip_invert", name) ? " is 0 or 1" : " is from %g to %g", v->low, v->high), d.trouble = name + std::string(range), false;
    if (key && !v->moves) { const std::string moving = said(true); return d.trouble = std::string(name) + " does not move: " + (moving.empty() ? std::string("nothing of it does") : "keys go on " + moving), false; }
    if ((number || word || key) && strcmp(name, "lock") && doc_number(d, thing, "lock", 0, 0) > 0.5) return d.trouble = "it is locked", false;
    if (number && (!strcmp(name, "connect") || !strcmp(name, "connect_to")) && *number != 0.0 && (*number != std::floor(*number) || (int)*number == thing || doc_word(d, (int)*number, "kind").empty()))
        return d.trouble = std::string(name) + " is another thing that is there", false;   // (as a parent is)
    if (number && !strcmp(name, "parent") && *number != 0.0) {   // (a parent is a thing that is there, and not the thing itself nor one that goes with it: the past repository's rule)
        const int to = (int)*number;
        if (to != *number || doc_word(d, to, "kind").empty()) return d.trouble = "a parent is a thing that is there", false;
        for (int up = to, n = 0; up && n < 1000; up = (int)doc_number(d, up, "parent", 0, 0), ++n)
            if (up == thing) return d.trouble = "a thing cannot go with itself, nor with one that goes with it", false;
    }
    return true;
}

// The calls. A new thing of a kind (film, picture, text, solid) from a file (a text: its font; a solid: a file, or ring, gem or cube): its number (0: not
// made, and no number is used up).
// What kind of thing a file makes, by its name's end ("": none): the one place that says so.
static const char* doc_kind_of(const char* file) {
    static const struct { const char* kind; const char* ends; } made[] = {
        {"film", "mp4 mov m4v webm mkv"}, {"picture", "png jpg jpeg webp svg json lot"}, {"solid", "glb gltf"}, {"text", "ttf otf ttc"}};
    std::string end = std::filesystem::path(file).extension().string();
    for (char& c : end) c = (char)std::tolower((unsigned char)c);
    if (doc_among(doc_shapes, file)) return "solid";
    if (doc_among("rect circle polygon star line null", file)) return "shape";
    for (const auto& one : made)
        if (end.size() > 1 && doc_among(one.ends, end.substr(1))) return one.kind;
    return "";
}
static int doc_add(Doc& d, const char* kind, const char* file) {
    if (!*kind) kind = doc_kind_of(file);   // (not said: what the file makes)
    if (!*kind) return d.trouble = std::string("no kind of thing is made from ") + file, 0;
    if (!doc_among(doc_kinds, kind)) return d.trouble = std::string("a thing is one of: ") + doc_kinds, 0;
    if (!strcmp(kind, "light") && !doc_among("sun point spot", file)) return d.trouble = "a light is a sun, a point or a spot", 0;
    // (it comes on at the stage's present moment)
    if (!doc_change(d, sqlite3_mprintf("insert into thing default values; insert into fact(thing, name, at, value) values(last_insert_rowid(), 'kind', 0.0, %Q); insert into fact(thing, name, at, value) values(last_insert_rowid(), %Q, 0.0, %Q); insert into fact(thing, name, at, value) select id, 'start', 0.0, %!.17g from thing where %d order by id desc limit 1%s",
                                       kind, (!strcmp(kind, "solid") && doc_among(doc_shapes, file)) || !strcmp(kind, "shape") ? "shape" : "file", file, d.stage->now, d.stage->now > 0.0,
                                       // (a new text comes with a word to read, at a size that fits a stage)
                                       strcmp(kind, "text") ? "" : "; insert into fact(thing, name, at, value) select id, 'text', 0.0, 'Text' from thing order by id desc limit 1; insert into fact(thing, name, at, value) select id, 'size', 0.0, 0.5 from thing order by id desc limit 1"))) return 0;
    int thing = 0;   // (the newest number: SQLite's own count of things, which a change not made does not move)
    doc_rows(d, sqlite3_mprintf("select max(id) from thing"), [&](sqlite3_stmt* row) { thing = sqlite3_column_int(row, 0); });
    return thing;
}
static bool doc_remove(Doc& d, int thing) {
    if (doc_knows(d, thing) && doc_number(d, thing, "lock", 0, 0) > 0.5) return d.trouble = "it is locked", false;
    return doc_knows(d, thing) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d", thing)); }
// A value that stays (its keys, if it had any, are gone); a word; a key at a moment; a key taken away.
static bool doc_parent(Doc& d, int thing, double to);
static bool doc_set(Doc& d, int thing, const char* name, double is) {
    if (!strcmp(name, "parent")) return doc_parent(d, thing, is);
    return doc_knows(d, thing, name, &is) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q and at <> 0; insert into fact(thing, name, at, value) values(%d, %Q, 0.0, %!.17g) on conflict do update set value = excluded.value, way = null", thing, name, thing, name, is));
}
static bool doc_say(Doc& d, int thing, const char* name, const char* word) {
    return doc_knows(d, thing, name, nullptr, word) && doc_change(d, sqlite3_mprintf("insert into fact(thing, name, at, value) values(%d, %Q, 0.0, %Q) on conflict do update set value = excluded.value", thing, name, word));
}
// A value either stays (one fact, at 0, with no way: what `set` writes) or is keyed: a key is a key even alone, and
// a value's first key takes the place of the value that stayed (a key at 0 is marked by its way, Linear until told).
static int doc_keys(Doc& d, int thing, const char* name, double at);
static bool doc_key(Doc& d, int thing, const char* name, double at, double is) {
    if (!(at >= 0.0 && at <= 1e6)) return d.trouble = "a key is at a moment of the stage: from 0 to 1000000 seconds", false;
    if (!doc_knows(d, thing, name, &is, nullptr, true)) return false;
    const bool stayed = !doc_keys(d, thing, name, -1.0) && at != 0.0;   // (the value that stayed goes: the key holds the place now)
    return doc_change(d, sqlite3_mprintf("%z insert into fact(thing, name, at, value, way) values(%d, %Q, %!.17g, %!.17g, %s) on conflict do update set value = excluded.value, way = coalesce(way, excluded.way)",
                                         stayed ? sqlite3_mprintf("delete from fact where thing = %d and name = %Q;", thing, name) : sqlite3_mprintf(""), thing, name, at, is, at == 0.0 ? "'Linear'" : "null"), true);
}
static int doc_keys(Doc& d, int thing, const char* name, double at = -1.0) {   // (how many keys a value has; or whether it has one at a moment)
    int n = 0;
    doc_rows(d, at < 0.0 ? sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q and " DOC_KEYED, thing, name, thing, name)
                         : sqlite3_mprintf("select count(*) from fact where thing = %d and name = %Q and at = %!.17g", thing, name, at), [&](sqlite3_stmt* row) { n = sqlite3_column_int(row, 0); });
    return n;
}
// What a hand does to a value at a moment, as the document's own rules (a shell does not work them out):
// put: the value changed there. Every change is being keyed (d.keying): a key there, and if it is the value's first, what
//      it was stays as a key where the keying began. Not being keyed: a value with keys gets a key there (as After
//      Effects keys a property that has keyframes, wherever it is changed), a value without is the value that stays.
// key_here: a key at that moment holding what the value is there (nothing, if it has one there).
// raise: brought nearer or further back among the things.
// stay: its keys are taken away, and it stays what it is at that moment.
// keyed: 0 the value has no keys, 1 it has keys but none at that moment, 2 it has a key at that moment.
// moving: the names of the values of a thing that can take keys.
static double doc_usual(const char* name);
static bool doc_set(Doc& d, int thing, const char* name, double is);
static bool doc_put(Doc& d, int thing, const char* name, double at, double is) {   // (After Effects' rule; the past repository moved a keyed value whole: motolii-edit, place_checked)
    if (!doc_knows(d, thing, name, &is)) return false;
    const bool keyed = doc_keys(d, thing, name) > 0;
    if (keyed) return doc_key(d, thing, name, at, is);   // (a value with keys: a key there, whether every change is keyed or not)
    if (!d.keying) return doc_set(d, thing, name, is);
    // (its first key while every change is keyed: what it was stays, as a key where the keying began, so it travels from there)
    const double was = doc_number(d, thing, name, d.keying_from, doc_usual(name));
    doc_act(d);
    if (d.keying_from != at) doc_key(d, thing, name, d.keying_from, was);
    doc_key(d, thing, name, at, is);
    return doc_acted(d);
}
// A thing given another to go with (0: none) without moving on the stage, as the sample's Parent seat promises and
// After Effects does: where it is, how far it is turned about the eye's line and how large are said again as the new
// parent sees them (a value with keys is moved whole, by the rule of doc_put; a keyed size is left as it is). A parent
// that leans, turned about x or y, is not undone in the turn: only where the thing is takes it in.
static bool doc_parent(Doc& d, int thing, double to) {
    if (!doc_knows(d, thing, "parent", &to)) return false;
    const double now = d.stage->now;
    const auto chain = [&](int up, double* turn, double* size) {   // (how far the parents turn it, and how large they make it)
        *turn = 0.0, *size = 1.0;
        for (int n = 0; up && n < 64 && d.shown.count(up); up = (int)doc_number(d, up, "parent", 0, 0), ++n) *turn += doc_number(d, up, "rotate", now, 0), *size *= doc_number(d, up, "size", now, 1);
    };
    double turnWas, sizeWas, turnTo, sizeTo;
    chain((int)doc_number(d, thing, "parent", 0, 0), &turnWas, &sizeWas);
    chain((int)to, &turnTo, &sizeTo);
    const math::double4 world = doc_above(d, thing, now) * math::double4{doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0), 1.0};
    const double turn = doc_number(d, thing, "rotate", now, 0), size = doc_number(d, thing, "size", now, 1);
    doc_act(d);
    const bool keying = d.keying;
    d.keying = false;   // (said again, not keyed)
    bool ok = doc_change(d, sqlite3_mprintf("insert into fact(thing, name, at, value) values(%d, 'parent', 0.0, %!.17g) on conflict do update set value = excluded.value", thing, to));
    const math::double3 here = (inverse(doc_above(d, thing, now)) * world).xyz;
    ok = ok && doc_put(d, thing, "x", now, here.x) && doc_put(d, thing, "y", now, here.y) && doc_put(d, thing, "z", now, here.z) && doc_put(d, thing, "rotate", now, turn + turnWas - turnTo);
    if (ok && !doc_keys(d, thing, "size")) ok = doc_set(d, thing, "size", std::clamp(size * sizeWas / sizeTo, 1e-3, 1e4));
    d.keying = keying;
    return doc_acted(d) && ok;
}
static bool doc_key_here(Doc& d, int thing, const char* name, double at) {
    if (!doc_knows(d, thing, name, nullptr, nullptr, true)) return false;
    return doc_keys(d, thing, name, at) && doc_keys(d, thing, name) ? true : doc_key(d, thing, name, at, doc_number(d, thing, name, at, doc_usual(name)));
}
// raise: a thing brought `places` places nearer among the things (below 0: further back): each thing is told where it
// lies now, as one change.
static bool doc_raise(Doc& d, int thing, int places) {
    if (!doc_knows(d, thing)) return false;
    std::vector<int> lie = doc_laid(d);
    const long from = std::find(lie.begin(), lie.end(), thing) - lie.begin(), to = std::clamp(from + (long)places, 0L, (long)lie.size() - 1);
    if (to == from) return d.trouble = places > 0 ? "it is the nearest already" : "it is the furthest back already", false;
    lie.erase(lie.begin() + from);
    lie.insert(lie.begin() + to, thing);
    char* all = sqlite3_mprintf("");
    for (size_t i = 0; i < lie.size(); ++i)
        all = sqlite3_mprintf("%zinsert into fact(thing, name, at, value) values(%d, 'order', 0.0, %d) on conflict do update set value = excluded.value;", all, lie[i], (int)i + 1);
    return doc_change(d, all);
}
// Pictures in one call (what Cavalry's tutorials build from a shape, a duplicator and a behaviour or two, each named):
// a file each in recipes/ (doc_folders: the library's, the maker's own; a file put there is on the shelf at once). Its head:
// "// Name: what it makes", "// make kind file", then a thing's values, one to a line: "set name number",
// "say name words", "key name seconds number", "then kind file" for another thing; its knobs as script.h says.
// Code below the head is the first thing's script.
struct DocRecipe { std::string name, what, path; std::vector<std::string> lines; bool code = false; };
static std::vector<DocRecipe> doc_recipes_read();
// The folders a kind of file is read from: the library's own (beside the out/ this library is in), then the maker's
// own, as a maker's scripts and presets are theirs (MOTOLII_OWN, or %APPDATA%/Motolii on Windows, or
// ~/Library/Application Support/Motolii on a Mac), each with a folder of that kind in it. The maker's of a name stands.
static std::vector<std::filesystem::path> doc_folders(const char* folder) {
    Dl_info lib{};
    dladdr((const void*)&doc_folders, &lib);
    std::error_code wrong;
    std::vector<std::filesystem::path> at{std::filesystem::weakly_canonical(lib.dli_fname ? lib.dli_fname : ".", wrong).parent_path().parent_path() / folder};
    if (const char* own = std::getenv("MOTOLII_OWN")) at.push_back(std::filesystem::path(own) / folder);
    else if (const char* app = std::getenv("APPDATA")) at.push_back(std::filesystem::path(app) / "Motolii" / folder);
    else if (const char* home = std::getenv("HOME")) at.push_back(std::filesystem::path(home) / "Library" / "Application Support" / "Motolii" / folder);
    return at;
}
// The files of that kind in those folders, by their name (the maker's after the library's: theirs of a name stands).
static std::vector<std::filesystem::path> doc_folder_files(const char* folder, const char* extension) {
    std::map<std::string, std::filesystem::path> by;
    std::error_code wrong;
    for (const auto& dir : doc_folders(folder))
        for (const auto& e : std::filesystem::directory_iterator(dir, wrong)) if (e.path().extension() == extension) by[e.path().filename().string()] = e.path();
    std::vector<std::filesystem::path> files;
    for (const auto& [_, f] : by) files.push_back(f);
    return files;
}
// Whether a kind of file is to be read again: a file put in or taken out of either folder (its time), else every two
// seconds (a file written over). A window asks at every build.
static bool doc_folder_moved(const char* folder, std::filesystem::file_time_type& was, std::chrono::steady_clock::time_point& looked) {
    std::error_code wrong;
    std::filesystem::file_time_type stamp{};
    for (const auto& dir : doc_folders(folder)) stamp = std::max(stamp, std::filesystem::last_write_time(dir, wrong));
    const auto now = std::chrono::steady_clock::now();
    if (stamp == was && now - looked < std::chrono::seconds(2)) return false;
    was = stamp, looked = now;
    return true;
}
static std::vector<DocRecipe> doc_recipes() {
    static std::vector<DocRecipe> all;
    static std::filesystem::file_time_type was{};
    static std::chrono::steady_clock::time_point looked{};
    if (doc_folder_moved("recipes", was, looked)) all = doc_recipes_read();
    return all;
}
static std::vector<DocRecipe> doc_recipes_read() {
    std::vector<DocRecipe> all;
    for (const auto& f : doc_folder_files("recipes", ".js")) {
        DocRecipe r;
        r.path = f.string();
        std::ifstream in(f);
        for (std::string line; std::getline(in, line);) {
            if (line.rfind("//", 0) != 0) { if (line.find_first_not_of(" \t\r") != std::string::npos) { r.code = true; break; } continue; }
            line.erase(0, line.find_first_not_of("/ "));
            if (line.rfind("--", 0) == 0) continue;   // (a knob)
            if (r.name.empty()) { const size_t colon = line.find(':'); r.name = line.substr(0, colon); r.what = colon == std::string::npos ? "" : line.substr(line.find_first_not_of(' ', colon + 1)); }
            else if (!line.empty()) r.lines.push_back(line);
        }
        if (!r.name.empty() && !r.lines.empty() && r.lines[0].rfind("make ", 0) == 0) all.push_back(r);
    }
    return all;
}
static std::string doc_recipe_names() {
    std::string all;
    for (const DocRecipe& r : doc_recipes()) all += (all.empty() ? "" : "\n") + r.name + "\t" + r.what;
    return all;
}
// Whether a file the document's pictures are made from (the stage's effects, the things' scripts) has been written
// over since it was read: a stage that is not drawing is woken by it (a file saved shows at once).
static bool doc_files_moved(Doc& d) {
    bool moved = screen_moved(d.screen);
    const int before = doc_scripts_made();
    for (int t : doc_laid(d)) doc_scripts_on(d, t);   // (each looks at its file's time and reads it again if it changed)
    if (doc_scripts_made() != before) moved = true, d.scripted.clear();
    return moved;
}
// The files of a kind in its folders (doc_folders) ("name\tpath", one to a line): the whole-screen effects (effects/*.wgsl),
// the motions a thing may be given (animators/*.js); a file's name as its first line says it, before the colon.
static std::string doc_folder_names(const char* folder, const char* extension) {
    struct Seen { std::string all; std::filesystem::file_time_type was{}; std::chrono::steady_clock::time_point looked{}; };
    static std::map<std::string, Seen> seen;
    Seen& one = seen[folder];
    std::string& all = one.all;
    if (!doc_folder_moved(folder, one.was, one.looked)) return all;
    all.clear();
    for (const auto& f : doc_folder_files(folder, extension)) {
        std::ifstream in(f);
        std::string first, name = f.stem().string();
        if (std::getline(in, first) && first.rfind("// ", 0) == 0 && first.find(':') != std::string::npos) name = first.substr(3, first.find(':') - 3);
        all += (all.empty() ? "" : "\n") + name + "\t" + f.string();
    }
    return all;
}
// The ways a maker keeps of their own, as a maker's scripts are theirs: a file each in ways/ of their own folder
// (doc_folders), and the few the library comes with (one put in either is a tile of the Ease desk at once). Its first line says its name
// before the colon; the rest is how far it is, as a way defined in a document says it (doc_define: a, b, u, p1..p4,
// bezier(u, x1, y1, x2, y2)). Put on a key, it is written into the document, so the document goes as it did wherever it
// is opened. The maker's own of a name goes before the library's.
static const std::map<std::string, std::string>& doc_own_ways() {
    static std::map<std::string, std::string> all;
    static std::chrono::steady_clock::time_point looked{};
    const auto now = std::chrono::steady_clock::now();
    if (looked != std::chrono::steady_clock::time_point{} && now - looked < std::chrono::milliseconds(500)) return all;   // (looked at again at most twice a second: a file put there shows at once)
    looked = now;
    all.clear();
    std::error_code wrong;
    for (const auto& dir : doc_folders("ways"))
        for (const auto& e : std::filesystem::directory_iterator(dir, wrong)) {
            if (e.path().extension() != ".way") continue;
            std::ifstream in(e.path());
            std::string line, name, goes;
            while (std::getline(in, line)) {
                if (line.rfind("//", 0) != 0) { goes += line + " "; continue; }
                if (name.empty() && line.find(':') != std::string::npos) name = line.substr(2, line.find(':') - 2);
            }
            name.erase(0, name.find_first_not_of(' '));
            if (name.empty()) name = e.path().stem().string();
            if (goes.find_first_not_of(' ') != std::string::npos) all[name] = goes;   // (the maker's folder is read last: theirs of a name stands)
        }
    return all;
}
// The recipe of that name made at the stage's present moment, one step (0: not made, d.trouble says why).
static int doc_make(Doc& d, const char* name) {
    std::vector<DocRecipe> all = doc_recipes();
    const DocRecipe* r = nullptr;
    for (const DocRecipe& one : all) if (one.name == name) r = &one;
    if (!r) return d.trouble = std::string("there is no recipe ") + name, 0;
    doc_act(d);
    int now = 0, first = 0, before = 0;   // (the thing the lines tell: "make" the first, "then" another, each named as the recipe)
    for (const std::string& line : r->lines) {
        std::istringstream w(line);
        std::string call, value;
        w >> call >> value;
        std::string rest;
        std::getline(w, rest);
        rest.erase(0, rest.find_first_not_of(' '));
        if (call == "make" || call == "then") {
            before = now;
            if (!(now = doc_add(d, value.c_str(), rest.c_str()))) break;
            if (!first) first = now;
            doc_say(d, now, "name", r->name.c_str()), doc_set(d, now, "start", 0);   // (on from the document's beginning, wherever the stage stands: a picture that goes round with it)
            if (now == first && r->code) doc_say(d, now, "script", r->path.c_str());
            continue;
        }
        const int thing = now;
        if (!thing) break;
        if (call == "set") doc_set(d, thing, value.c_str(), rest == "first" ? first : rest == "before" ? before : rest == "behind" ? first - 0.5 : std::atof(rest.c_str()));   // (a thing it goes with: the recipe's first, or the one made before this one; an order just behind the first)
        else if (call == "say") doc_say(d, thing, value.c_str(), value == "script" && rest == "this" ? r->path.c_str() : rest.c_str());   // (another of its things run by the recipe's own code: "this")
        else if (call == "key") { std::istringstream k(rest); double at = 0, is = 0; k >> at >> is; doc_key(d, thing, value.c_str(), at, is); }   // (at that moment of the document, wherever the stage stands: its film loops as made)
    }
    return doc_acted(d) && now ? first : 0;
}
static bool doc_stay(Doc& d, int thing, const char* name, double at) {   // (its keys go: it stays what it is at that moment)
    return doc_knows(d, thing, name) && (!doc_keys(d, thing, name) || doc_set(d, thing, name, doc_number(d, thing, name, at, doc_usual(name))));
}
static int doc_keyed(Doc& d, int thing, const char* name, double at) { return !doc_keys(d, thing, name) ? 0 : doc_keys(d, thing, name, at) ? 2 : 1; }
// The words a value of a thing may be (a blend's, a solid's shape), with spaces between ("": any word, or not a word).
static const DocValue* doc_script_value(Doc& d, int thing, const char* name) {   // (a knob of the thing's script)
    if (strncmp(name, "js_", 3)) return nullptr;
    for (const DocScript* sc : doc_scripts_on(d, thing))
        for (const DocValue& one : sc->values) if (!strcmp(one.name, name)) return &one;
    return nullptr;
}
static std::string doc_words(Doc& d, int thing, const char* name) {
    if (const DocValue* v = doc_script_value(d, thing, name)) return v->words ? v->words : "";
    const std::string kind = doc_word(d, thing, "kind");
    if (!strcmp(name, "animation") && kind == "solid") {   // (its file's own movements, as gltfio reads them)
        std::string all;
        if (const auto t = d.shown.find(thing); t != d.shown.end() && t->second.solid)
            for (const std::string& m : t->second.solid->moves) all += (all.empty() ? "" : " ") + m;
        return all;
    }
    if (!strcmp(name, "marker") && kind == "picture") {   // (the named stretches of its file, as ThorVG reads them; any: all of it)
        std::string all = "all";
        if (const auto t = d.shown.find(thing); t != d.shown.end() && t->second.drawn)
            for (const std::string& m : t->second.drawn->markers) all += " " + m;
        return all;
    }
    for (const DocValue& one : doc_values)
        if (!strcmp(one.name, name) && doc_among(one.kinds, kind)) return one.words ? one.words : "";
    return "";
}
// What a thing is called where things are listed: a text by its words, a shape by what shape it is, a file by its name.
static std::string doc_name(Doc& d, int thing) {
    if (const std::string said = doc_word(d, thing, "name"); !said.empty()) return said;
    if (doc_word(d, thing, "kind") == "camera") return "Camera";
    std::string is = doc_word(d, thing, "text");
    if (is.empty()) is = doc_word(d, thing, "shape");
    if (is.empty()) return std::filesystem::path(doc_word(d, thing, "file")).filename().string();
    if (doc_word(d, thing, "kind") != "text") is[0] = (char)std::toupper((unsigned char)is[0]);
    std::replace(is.begin(), is.end(), '\n', ' ');   // (a text of lines is listed on one)
    return is;
}
static std::string doc_moving(Doc& d, int thing) {
    const std::string kind = doc_word(d, thing, "kind");
    std::string names;
    for (const DocValue& one : doc_values)
        if (one.moves && doc_among(one.kinds, kind)) names += std::string(names.empty() ? "" : " ") + one.name;
    return names;
}
// The names of all the values a thing has (from the values table: those of its kind), and a number's range.
static std::string doc_has(Doc& d, int thing) {
    const std::string kind = doc_word(d, thing, "kind");
    std::string names;
    for (const DocValue& one : doc_values)
        if (doc_among(one.kinds, kind)) names += std::string(names.empty() ? "" : " ") + one.name;
    if (doc_among("picture text shape stage", kind))
        for (const DocScript* sc : doc_scripts_on(d, thing))
            for (const DocValue& one : sc->values) names += std::string(names.empty() ? "" : " ") + one.name;
    return names;
}
// The cards of a thing: "title: name name; title: name" (those of its values its kind has; a card with none, left out).
static bool doc_on(Doc& d, int thing, const char* name);
static std::string doc_cards_of(Doc& d, int thing) {
    const std::string has = " " + doc_has(d, thing) + " ";
    std::string all;
    for (const DocCard& card : doc_cards) {
        if (card.when && card.is[0] == '+') {   // (an effect's card while it is on: doc_on)
            if (!doc_on(d, thing, card.when)) continue;
        } else if (card.when && card.is[0] == '~') {   // (a ramp's card while the fill has more colours than one)
            if (doc_word(d, thing, card.when).find(card.is + 1) == std::string::npos) continue;
        } else if (card.when) {   // (a card for one fill and not another, or while it is clipped)
            const std::string w = doc_word(d, thing, card.when), want = card.is[0] == '!' ? card.is + 1 : card.is;
            char* end = nullptr;
            const double n = std::strtod(w.c_str(), &end);
            const bool is = w == want || (!w.empty() && !*end && n == std::atof(want.c_str()) && !want.empty() && isdigit((unsigned char)want[0]));
            if (is == (card.is[0] == '!')) continue;
        }
        std::string names;
        for (const char* at = card.names; *at;) {
            const size_t len = strcspn(at, " ");
            const std::string n(at, len);
            const bool still = (n == "animation" || n == "speed") && doc_word(d, thing, "kind") == "solid" && doc_words(d, thing, "animation").empty();   // (a solid with no movement of its own has none to play)
            if (has.find(" " + n + " ") != std::string::npos && !still) names += (names.empty() ? "" : " ") + n;
            at += len + (at[len] ? 1 : 0);
        }
        if (!names.empty()) all += (all.empty() ? "" : "; ") + std::string(card.title) + ": " + names;
    }
    if (doc_among("picture text shape stage", doc_word(d, thing, "kind")))   // (its script's own card: the knobs its head declares)
        if (const std::vector<DocScript*> scs = doc_scripts_on(d, thing); !scs.empty()) {
            std::string names = thing == 0 ? "script follow" : "script";
            for (const DocScript* sc : scs)
                for (const DocValue& one : sc->values)
                    if ((" " + names + " ").find(" " + std::string(one.name) + " ") == std::string::npos) names += std::string(" ") + one.name;   // (a knob two effects share is one)
            if (names != "script") all += (all.empty() ? "" : "; ") + std::string("Script: ") + names;   // (none of them with knobs: no card)
        }
    return all;
}
static bool doc_range(Doc& d, int thing, const char* name, double* low, double* high) {
    const std::string kind = doc_word(d, thing, "kind");
    for (const DocValue& one : doc_values)
        if (!strcmp(one.name, name) && doc_among(one.kinds, kind) && !one.words) return *low = one.low, *high = one.high, true;
    if (const DocValue* v = doc_script_value(d, thing, name); v && !v->words) return *low = v->low, *high = v->high, true;
    return false;
}
// Whether a value is on: keyed, or said and not nought (a colour said is on, black too).
static bool doc_on(Doc& d, int thing, const char* name) {
    if (doc_keys(d, thing, name)) return true;
    const std::string w = doc_word(d, thing, name);
    double low = 0, high = 0;
    return !w.empty() && (!doc_range(d, thing, name, &low, &high) || std::atof(w.c_str()) != 0);
}
// The effects a thing may be given (the Effects shelf; their cards are doc_cards' "+" rows): a name, the value that puts
// it on (ThorVG's own), and what it is told then: a number, a word, or keys from that moment ("keys 0 1 1 0": 1 now,
// 0 a second on: Trim Paths drawn on, as After Effects' is mostly used).
struct DocEffect { const char* name; const char* value; const char* on; bool word; };
static const DocEffect doc_effects[] = {
    {"Gaussian Blur", "blur", "12", false}, {"Drop Shadow", "shadow", "000000", true}, {"Glow", "glow", "ffd166", true},
    {"Trim Paths", "trim", "keys 0 1 1 0", false}, {"Duotone", "tint", "1", false}, {"Gradient Map", "tritone", "1", false},
};
// The shelf's effects, "name\tvalue" one to a line.
static std::string doc_effect_names() {
    std::string all;
    for (const DocEffect& e : doc_effects) all += (all.empty() ? "" : "\n") + std::string(e.name) + "\t" + e.value;
    return all;
}
// An effect put on a thing at a moment, or taken off it if it is on (unsaid, its keys too), one step.
static bool doc_effect(Doc& d, int thing, const char* name, double at) {
    for (const DocEffect& e : doc_effects) {
        if (strcmp(e.name, name)) continue;
        const bool on = doc_on(d, thing, e.value);
        doc_act(d);
        bool ok;
        if (on) ok = doc_knows(d, thing, e.value) && doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q", thing, e.value));   // (unsaid: as it was before it was put on)
        else if (!strncmp(e.on, "keys ", 5)) {
            double a0 = 0, v0 = 0, a1 = 0, v1 = 0;
            ok = std::sscanf(e.on + 5, "%lf %lf %lf %lf", &a0, &v0, &a1, &v1) == 4 && doc_key(d, thing, e.value, at + a0, v0) && doc_key(d, thing, e.value, at + a1, v1);
        } else ok = e.word ? doc_say(d, thing, e.value, e.on) : doc_set(d, thing, e.value, std::atof(e.on));
        if (!ok) d.act_ok = false;
        return doc_acted(d);
    }
    d.trouble = std::string("no effect is called ") + name;
    return false;
}
static bool doc_unkey(Doc& d, int thing, const char* name, double at) {
    if (!doc_knows(d, thing, name)) return false;
    if (!doc_keys(d, thing, name) || !doc_keys(d, thing, name, at)) return d.trouble = std::string(name) + " has no key at that moment", false;
    if (doc_keys(d, thing, name) == 1) return doc_set(d, thing, name, doc_number(d, thing, name, at, 0));   // (its last key: it stays what the key held)
    return doc_change(d, sqlite3_mprintf("delete from fact where thing = %d and name = %Q and at = %!.17g", thing, name, at));
}
// How the stretch from a key to the next goes: one of the nine kinds, alone or with all its numbers (Random 3 0.25
// 0.5 0.8 0.5), or a way defined in this document with up to four numbers.
static std::string doc_ways(Doc& d) {
    std::string all;
    for (const auto& kind : interp_kinds) all += std::string(all.empty() ? "" : ", ") + kind.name + (kind.numbers ? std::string(" (") + kind.names + ")" : "");
    all += "; by name:";
    for (const auto& one : named_ways) all += std::string(" ") + one.name;
    doc_rows(d, sqlite3_mprintf("select name from way order by name"), [&](sqlite3_stmt* row) { all += std::string("; defined here: ") + (const char*)sqlite3_column_text(row, 0); });
    return all;
}
static bool doc_way(Doc& d, int thing, const char* name, double at, const char* way) {
    if (!doc_knows(d, thing, name, nullptr, nullptr, true)) return false;
    if (!doc_keys(d, thing, name, at)) return d.trouble = std::string(name) + " has no key at that moment: a stretch begins at a key", false;
    Interp kind;
    std::istringstream said(way);
    std::string named, more;
    double p = 0;
    int numbers = 0;
    const int of_the_nine = interp_said(way, &kind, &d.trouble);
    if (of_the_nine == 1) return false;
    said >> named;
    while (said >> p) ++numbers;
    said.clear();
    if (of_the_nine < 0 && !doc_way_named(d, named)) return d.trouble = "a way is one of: " + doc_ways(d), false;
    if (of_the_nine < 0 && (said >> more || numbers > 4)) return d.trouble = "a way defined here is its name and up to four numbers", false;
    return doc_change(d, sqlite3_mprintf("update fact set way = %Q where thing = %d and name = %Q and at = %!.17g", way, thing, name, at));
}
static bool doc_define(Doc& d, const char* name, const char* expression) {
    Interp kind;
    if (interp_said(name, &kind) >= 0) return d.trouble = std::string(name) + " is one of the nine kinds already", false;
    sqlite3_stmt* goes = doc_goes(d, expression);
    if (!goes) return false;
    const double p[4] = {1, 1, 1, 1}, tried[2][3] = {{0, 1, 1}, {2, -5, 3}};   // (two stretches to try it on: values a to b, over that many seconds)
    for (const auto& s : tried) {
        const double from = doc_went(goes, 0, 0, s[2], s[0], s[1], p), to = doc_went(goes, 1, s[2], s[2], s[0], s[1], p);
        char why[160];
        if (std::isfinite(from) && std::isfinite(to) && std::fabs(from - s[0]) < 1e-6 && std::fabs(to - s[1]) < 1e-6) continue;
        std::snprintf(why, sizeof why, "a way begins at the one key's value and ends at the other's: from %g to %g this one begins at %g and ends at %g", s[0], s[1], from, to);
        return d.trouble = why, false;
    }
    return doc_change(d, sqlite3_mprintf("insert into way values(%Q, %Q) on conflict do update set goes = excluded.goes", name, expression));
}
// What a shell reads to draw its lists: the things there are, how long each is on (its duration; below 0: for good,
// going round), and the moments of a value's keys (a value with one fact has none).
static std::vector<int> doc_things(Doc& d) { return doc_laid(d); }   // (in the order they lie: the one furthest back first)
static double doc_lasts(Doc& d, int thing) {
    const double lasts = doc_number(d, thing, "duration", 0, -1);
    return lasts > 0.0 ? lasts : -1.0;
}
static std::vector<double> doc_key_moments(Doc& d, int thing, const char* name) {
    const auto known = d.keyed.find({thing, name});   // (asked again and again between changes: the document asked once)
    if (known != d.keyed.end()) return known->second;
    std::vector<double>& at = d.keyed[{thing, name}];
    doc_rows(d, sqlite3_mprintf("select at from fact where thing = %d and name = %Q and " DOC_KEYED " order by at", thing, name, thing, name),
             [&](sqlite3_stmt* row) { at.push_back(sqlite3_column_double(row, 0)); });
    return at;
}

// Half of how wide and how high a thing is at size 1, before it is placed (a sheet: the sheet's; a solid: its
// bounds'): what a shell draws a thing's frame from.
static bool doc_half(Doc& d, int thing, float2* half, float* deep) {   // (deep: half of how far it reaches towards the eye and away; a flat thing: 0)
    const auto shown = d.shown.find(thing);
    if (shown == d.shown.end()) return false;
    const Doc::Shown& t = shown->second;
    if (t.solid) {
        const Aabb box = t.solid->made->getBoundingBox();
        const float tall = std::max(box.max.y - box.min.y, 1e-6f);
        if (deep) *deep = 0.5f * (box.max.z - box.min.z) / tall;
        return *half = float2{0.5f * (box.max.x - box.min.x) / tall, 0.5f}, true;
    }
    const Mesh* m = t.film ? t.film->sheet : t.drawn ? t.drawn->sheet : t.shape;
    if (!m || m->pos.empty()) return false;
    float3 most{0.f};
    for (const float3& p : m->pos) most = max(most, abs(p));
    if (deep) *deep = most.z;
    return *half = most.xy, true;
}

// Where a point of the stage is seen on the picture (points from the top left), by the stage's own eye.
static math::double2 doc_seen(Doc& d, math::double3 p) {
    Stage& s = *d.stage;
    const math::double4 c = s.camera->getProjectionMatrix() * s.camera->getViewMatrix() * math::double4{p, 1.0};
    return math::double2{(0.5 + 0.5 * c.x / c.w) * s.w, (0.5 - 0.5 * c.y / c.w) * s.h};
}
// How many points of the picture one unit of the stage is, where a thing stands (at its own depth).
static double doc_point(Doc& d, int thing, double now) {
    const math::double3 at = (doc_above(d, thing, now) * math::double4{doc_number(d, thing, "x", now, 0), doc_number(d, thing, "y", now, 0), doc_number(d, thing, "z", now, 0), 1.0}).xyz;
    return doc_seen(d, at + math::double3{1, 0, 0}).x - doc_seen(d, at).x;
}
// Where a thing is seen at a moment: as the stage places it (a thing alone that waves or is late, as the member it is
// drawn as); one with copies, where they are laid about.
static math::mat4 doc_shown_at(Doc& d, int thing, double now) { return doc_place(d, thing, now, 0, 0, doc_members(d, thing) == 1 && doc_staggered(d, thing, now)); }
// A thing's frame on the picture at a moment, as the stage's own eye sees it: its middle, how wide and how high (points
// of the picture, from the top left), and how far it is turned (degrees, clockwise as seen). false: it is not shown.
static bool doc_frame(Doc& d, int thing, double now, double* frame) {
    float2 half{0.f};
    float deep = 0.f;
    if (!doc_half(d, thing, &half, &deep) || !d.shown[thing].in) return false;   // (a thing that is not on, or is put aside, has no frame: a hand cannot take hold of what is not seen)
    const double size = doc_number(d, thing, "size", now, 1), point = doc_point(d, thing, now);
    const math::double2 middle = doc_seen(d, (doc_shown_at(d, thing, now) * math::double4{0, 0, 0, 1}).xyz);   // (its own middle, wherever its anchor is)
    const double sx = doc_number(d, thing, "scale_x", now, 1), sy = doc_number(d, thing, "scale_y", now, 1), sz = doc_number(d, thing, "scale_z", now, 1);
    frame[0] = middle.x, frame[1] = middle.y, frame[2] = 2.0 * half.x * size * std::fabs(sx) * point, frame[3] = 2.0 * half.y * size * std::fabs(sy) * point;
    frame[4] = -doc_number(d, thing, "rotate", now, 0);
    const math::double3 anchor{doc_number(d, thing, "anchor_x", now, 0) * half.x, doc_number(d, thing, "anchor_y", now, 0) * half.y, 0.0};
    const math::double2 pivot = doc_seen(d, (doc_shown_at(d, thing, now) * math::double4{anchor, 1.0}).xyz);   // (the point it turns about)
    frame[5] = pivot.x, frame[6] = pivot.y, frame[7] = 2.0 * deep * size * std::fabs(sz) * point;   // (and how deep it is: a flat thing, 0)
    double box[4];   // (turned about another line than the eye's, its frame is the upright box of what it covers)
    if ((doc_number(d, thing, "rotate_x", now, 0) != 0.0 || doc_number(d, thing, "rotate_y", now, 0) != 0.0 || doc_number(d, thing, "parent", 0, 0) != 0.0) && doc_covers(d, thing, now, box))
        frame[2] = box[2], frame[3] = box[3], frame[4] = 0.0;   // (as wide and high as that box; its middle stays the thing's own)
    return true;
}
// The upright box of the picture a thing covers at a moment, however it is turned or leans: left, top, wide, high.
static bool doc_covers(Doc& d, int thing, double now, double* box) {
    float2 half{0.f};
    float deep = 0.f;
    if (!doc_half(d, thing, &half, &deep)) return false;
    const math::mat4 place = doc_shown_at(d, thing, now);
    math::double2 low{1e12}, high{-1e12};
    for (int corner = 0; corner < 8; ++corner) {
        const math::double2 at = doc_seen(d, (place * math::double4{corner & 1 ? half.x : -half.x, corner & 2 ? half.y : -half.y, corner & 4 ? deep : -deep, 1.0}).xyz);
        low = min(low, at), high = max(high, at);
    }
    box[0] = low.x, box[1] = low.y, box[2] = high.x - low.x, box[3] = high.y - low.y;
    return true;
}

// A thing's outline on the picture at a moment: the corners of its box as the stage's eye sees them, and the polygon
// round them all (a convex hull, Andrew's way): a cube's own outline, a flat thing's tilted four corners; nothing of
// it lies outside. Points of the picture from the top left, in order round it; how many (0: not shown).
static int doc_outline(Doc& d, int thing, double now, double* xy) {
    float2 half{0.f};
    float deep = 0.f;
    if (!doc_half(d, thing, &half, &deep) || !d.shown[thing].in) return 0;
    const math::mat4 place = doc_shown_at(d, thing, now);
    std::vector<math::double2> p;
    for (int corner = 0; corner < (deep > 0.f ? 8 : 4); ++corner)
        p.push_back(doc_seen(d, (place * math::double4{corner & 1 ? half.x : -half.x, corner & 2 ? half.y : -half.y, corner & 4 ? deep : -deep, 1.0}).xyz));
    std::sort(p.begin(), p.end(), [](const math::double2& a, const math::double2& b) { return a.x < b.x || (a.x == b.x && a.y < b.y); });
    const auto turn = [](const math::double2& o, const math::double2& a, const math::double2& b) { return (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x); };
    std::vector<math::double2> hull(2 * p.size());
    size_t k = 0;
    for (size_t i = 0; i < p.size(); ++i) { while (k >= 2 && turn(hull[k - 2], hull[k - 1], p[i]) <= 0) --k; hull[k++] = p[i]; }
    for (size_t i = p.size() - 1, t = k + 1; i > 0; --i) { while (k >= t && turn(hull[k - 2], hull[k - 1], p[i - 1]) <= 0) --k; hull[k++] = p[i - 1]; }
    const int n = (int)std::min<size_t>(k > 0 ? k - 1 : 0, 8);
    for (int i = 0; i < n; ++i) xy[2 * i] = hull[(size_t)i].x, xy[2 * i + 1] = hull[(size_t)i].y;
    return n;
}

static double doc_usual(const char* name) {
    if (!strncmp(name, "js_", 3))   // (a knob of a script: as its head says it comes)
        for (const auto& [file, sc] : doc_scripts())
            for (size_t i = 0; i < sc.names.size(); ++i)
                if (sc.names[i] == name) return sc.knobs[i].usual;
    return doc_among("scale_x scale_y scale_z size speed opacity wide tall visible distance zoom copies copy_size copy_wide blur_level letter_spacing line_spacing contrast saturation vibrance environment gradient_scale wave_speed slope power", name) ? 1.0 : !strcmp(name, "duration") ? -1.0
         : !strcmp(name, "metallic") ? 0.08 : !strcmp(name, "roughness") ? 0.6 : !strcmp(name, "reflectance") ? 0.5 : !strcmp(name, "thickness") ? 0.5 : !strcmp(name, "sun_size") ? 1.09 : !strcmp(name, "sun_halo") ? 10.0   // (a solid's surface as it comes)
         : !strcmp(name, "intensity") ? 1.0 : !strcmp(name, "falloff") ? 10.0 : !strcmp(name, "cone") ? 30.0   // (a light as it comes)
         : !strcmp(name, "bounce") ? 0.3 : !strcmp(name, "friction") ? 0.5 : !strcmp(name, "gravity") ? 9.81   // (what falls and strikes, as it comes)
         : !strcmp(name, "ior") ? 1.5 : !strcmp(name, "dispersion") ? 0.1 : !strcmp(name, "swim") ? 0.5 : !strcmp(name, "dark") ? 4.0 : !strcmp(name, "vivid") ? 1.3 : 0.0;   // (the kit's glass as it comes)
}
static bool doc_value(Doc& d, int thing, const char* name, double at, double* is) { return doc_knows(d, thing, name) && (*is = doc_number(d, thing, name, at, doc_usual(name)), true); }
// A thing held by a hand, carried, and let go: the one form of everything a hand drags (a thing on the stage, its bar
// on the timeline, a number). What is carried shows at once and is not the document's until it is let go, and then
// it is one step; let go without keeping, nothing happened. The reckoning is here, once: a shell says where the hand
// is, never what the document is to hold.
//   by "place"  the thing carried on the stage: a, b = how far the hand has gone since it took hold, in points of the
//               picture (to the right, down); the thing stays under the hand at its own depth
//   by "bar"    its bar slid on the timeline: a = how many seconds later (when it comes on)
//   by "head", "tail"  its bar cut at an end: a = how many seconds later that end is (a film cut at its head goes on
//               showing at each moment what it showed there)
//   by a value's name  that number carried: a = what it is now
static bool doc_hold(Doc& d, int thing) {
    if (!doc_knows(d, thing)) return false;
    if (d.writing) return d.trouble = "the document is being written as a film: nothing is held until that is done", false;
    if (d.held.thing != thing) d.held = Doc::Held{}, d.held.thing = thing;   // (held already: the same hold goes on)
    return true;
}
static bool doc_carry(Doc& d, const char* by, double a, double b) {
    Doc::Held& h = d.held;
    if (h.thing < 0) return d.trouble = "nothing is held", false;
    const double now = d.stage->now;
    const auto was = [&](const char* name) {   // (as it was when the hand first carried it)
        auto found = h.from.find(name);
        if (found == h.from.end()) found = h.from.emplace(name, doc_number(d, h.thing, name, now, doc_usual(name))).first;
        return found->second;
    };
    const auto tries = [&](const char* name, double is) { return doc_knows(d, h.thing, name, &is) && (h.tried[name] = is, true); };
    bool ok = true;
    if (!strcmp(by, "place")) {
        if (!h.from.count(" point")) h.from[" point"] = doc_point(d, h.thing, now);   // (remembered with the hold: one unit of the stage in points of the picture, where the thing stood)
        const double x = was("x"), y = was("y"), point = h.from[" point"];
        // (the hand's travel is the stage's; a thing that goes with another is moved in that one's own turned and sized way)
        const math::double3 by = inverse(doc_above(d, h.thing, now).upperLeft()) * math::double3{a / point, -b / point, 0.0};
        ok = tries("x", x + by.x) && tries("y", y + by.y);
    } else if (!strcmp(by, "grip")) {   // (where on the picture the hand took hold, points from the top left: what scale, turn and pivot reckon from)
        double frame[8];
        if (!doc_frame(d, h.thing, now, frame)) return d.trouble = "it has no frame on the picture", false;
        h.from[" grip x"] = a, h.from[" grip y"] = b, h.from[" pivot x"] = frame[5], h.from[" pivot y"] = frame[6], h.from[" turned"] = frame[4];
        h.from[" point"] = doc_point(d, h.thing, now);
        was("scale_x"), was("scale_y"), was("rotate"), was("anchor_x"), was("anchor_y"), was("x"), was("y");
    } else if (!strcmp(by, "scale") || !strcmp(by, "turn") || !strcmp(by, "pivot")) {   // (a, b: how far the hand has gone since it took hold)
        if (!h.from.count(" grip x")) return d.trouble = "the hand has not said where it took hold (grip)", false;
        const double px = h.from[" pivot x"], py = h.from[" pivot y"], gx = h.from[" grip x"] - px, gy = h.from[" grip y"] - py, hx = gx + a, hy = gy + b;
        const double turned = h.from[" turned"] * (double)TAU / 360.0, c = std::cos(-turned), s = std::sin(-turned);   // (into the thing's own upright: as the picture counts, clockwise)
        if (!strcmp(by, "scale")) {   // (each way as far from the point it turns about as the hand now is, against where it took hold)
            const double g0 = c * gx - s * gy, g1 = s * gx + c * gy, n0 = c * hx - s * hy, n1 = s * hx + c * hy;
            ok = (std::fabs(g0) < 2.0 || tries("scale_x", was("scale_x") * n0 / g0)) && (std::fabs(g1) < 2.0 || tries("scale_y", was("scale_y") * n1 / g1));
        } else if (!strcmp(by, "turn")) {   // (round the point it turns about as far as the hand has gone round it; the picture counts clockwise, the stage the other way)
            const double by = std::atan2(hy, hx) - std::atan2(gy, gx);
            ok = tries("rotate", was("rotate") - by * 360.0 / (double)TAU);
        } else {   // (the point it turns about moved under the hand, the thing staying where it is seen: After Effects' Pan Behind)
            float2 half{0.5f};
            doc_half(d, h.thing, &half);
            const double point = h.from[" point"], size = was("size") == 0.0 ? 1.0 : doc_number(d, h.thing, "size", now, 1);
            const double l0 = (c * a - s * b) / point, l1 = -(s * a + c * b) / point;   // (the hand's travel in the thing's own units, y up)
            const double sx = was("scale_x"), sy = was("scale_y");
            ok = tries("anchor_x", was("anchor_x") + l0 / (size * (sx == 0.0 ? 1.0 : sx) * std::max(half.x, 1e-6f))) &&
                 tries("anchor_y", was("anchor_y") + l1 / (size * (sy == 0.0 ? 1.0 : sy) * std::max(half.y, 1e-6f)));
            const math::double3 move = inverse(doc_above(d, h.thing, now).upperLeft()) * math::double3{a / point, -b / point, 0.0};
            ok = ok && tries("x", was("x") + move.x) && tries("y", was("y") + move.y);
        }
    } else if (!strcmp(by, "bar")) {
        ok = tries("start", std::max(0.0, was("start") + a));
    } else if (!strcmp(by, "head")) {
        const double start = was("start"), lasts = was("duration"), to = std::clamp(start + a, 0.0, lasts > 0.0 ? start + lasts - 1e-3 : 1e6);
        ok = tries("start", to) && (lasts <= 0.0 || tries("duration", start + lasts - to));
        if (ok && doc_word(d, h.thing, "kind") == "film") ok = tries("begin", std::max(0.0, was("begin") + (to - start) * std::fabs(was("speed"))));
    } else if (!strcmp(by, "tail")) {
        const double start = was("start"), lasts = was("duration");
        ok = tries("duration", std::max(1e-3, (lasts > 0.0 ? lasts : doc_seconds(d) - start) + a));
    } else {
        was(by);   // (as it was: whether it comes on or goes off, carried)
        ok = tries(by, a);
        if (ok && strstr(doc_making, ("'" + std::string(by) + "'").c_str())) ok = doc_match(d);   // (a value it is made by: made again with it now, not when let go)
    }
    doc_moment(d, now);
    return ok;
}
static bool doc_let_go(Doc& d, bool keep) {
    const Doc::Held h = d.held;
    d.held = Doc::Held{};
    if (h.thing < 0) return d.trouble = "nothing is held", false;
    if (!keep || h.tried.empty()) return doc_match(d), doc_moment(d, d.stage->now), true;   // (made again as the document has it)
    doc_act(d);   // (all that was carried is one step: a value that takes keys by the rule of a value changed at a moment, any other as the value it is)
    for (const auto& [name, is] : h.tried) {
        bool moves = false;
        for (const DocValue& one : doc_values) moves = moves || (one.moves && name == one.name);
        if (!(moves ? doc_put(d, h.thing, name.c_str(), d.stage->now, is) : doc_set(d, h.thing, name.c_str(), is))) break;
    }
    return doc_acted(d);
}
static bool doc_undo(Doc& d) { return doc_step(d, true); }
static bool doc_redo(Doc& d) { return doc_step(d, false); }
