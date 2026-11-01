// doc_kinds.h: part of doc.h (the document and the names its values go by). Read only through doc.h.
#include "drawn.h"
#include <string_view>
#include <unordered_map>
#include <tuple>
#include "motion.h"
#include "interp.h"
#include "physics.h"
#include "script.h"
#include "lottie_slots.h"
#include "viewer_settings.h"
#include "absent_motion.h"
#include "isf.h"
#include "three_isf.h"
#include "midi_roll.h"
#include "ways.h"
#include "screen.h"
#include "thing_fx.h"
#include "mosh.h"
#include "points.h"
#include <filesystem>
#include <mutex>
#include <dlfcn.h>
#include <fstream>
#include "solid.h"
#include "video.h"
#include "sound.h"
#include "midi.h"
#include <filament/Skybox.h>
#include <set>
#include <sstream>
#include <sqlite3.h>

struct Doc {
    std::recursive_timed_mutex lock;   // held by whoever reads or changes it: a shell's reads need not wait for a picture being drawn (motolii.mm)
    sqlite3* file = nullptr;
    Stage* stage = nullptr;
    std::string trouble;   // why the last call that was not done was not
    std::vector<int> laid;   // the things in the order they lie, the one furthest back first (read when first asked for, forgotten at every change)
    // One act of a hand: the changes made from its beginning to its end are one step of undo, and if one of them is
    // refused none of them is made. Acts may lie one within another: the outermost one is the step.
    std::string graded;                 // the stage's grade as last made (doc_look): made again only when it is told otherwise
    std::set<std::pair<int, std::string>> changed;   // the values changed since a shell last asked (motolii_changes), each (thing, name), from each step's own changeset
    bool things_changed = false;        // and whether a thing came or went since
    std::map<uint32_t, MaterialInstance*> copyLooks, copyEdges;   // (copyEdges: the same for its edge, its second part)
    std::map<std::pair<int, std::string>, int> drivers;   // a value's behaviour (`<name>_by`), by thing and name: found once, forgotten at every change
    std::unordered_map<int, bool> connects;   // whether a thing has any value connected to a behaviour: asked once, forgotten at every change
    struct Relation { std::vector<int> members, under; std::vector<double> from; };   // (a relation's things in their order, 0 where one was taken out; for each, the relation it was in before (its result this one's start) and where it showed when it came in (x, y))
    std::unordered_map<int, Relation> relationsOf;   // found once, forgotten at every change
    bool settled = false;   // (reckoning where things settle: relations that come and go with time (follow, push) give nothing, so a relation made while one is pushing does not keep the push)
    int ctxCopy = 0, ctxN = 1;   // which of how many copies doc_place is placing now: what a behaviour gives is that copy's
    bool ctxHasAt = false; math::double3 ctxAt{0.0};   // (and where that copy lies on the stage, laid out, once doc_place knows: what a behaviour's falloff weighs)   // a copy's own look (its thing's look duplicated, by the copy's entity), while its copies are coloured each their own
    float roomLight = 0.f;              // the room's own light, as the stage was opened with it
    std::string roomIs;                 // which room lights it now (doc_look)
    Skybox* sky = nullptr;              // the room behind everything, once it is said
    Drawn* backdrop = nullptr;          // the colour behind everything: a flat colour as large as the stage, drawn before all else in the stage's own view
    std::string backdropIs;             // (the colour it was made in)
    bool keying = false;                // every value a hand changes gets a key at that moment (a shell's "animate all")
    int act = 0;                        // how many acts are open, one within another
    bool act_ok = true;                 // no change of the open act has been refused
    bool act_unmatched = false;         // a change within a larger act not yet shown: the stage is matched once, when the act ends
    bool act_together = false;          // the outermost act is motolii_together's: every change within it is matched once, when it ends (as a key is)
    std::unique_ptr<PhysWorld> phys;    // the things that fall and strike (Jolt): made when first asked, made again when what it is made of changes
    std::vector<int> phys_of;           // which thing each of its bodies is
    std::vector<int> phys_copy;         // and which of its copies (a split text's letter; -1: the thing whole)
    std::vector<double> phys_until;     // and till when its keys have it
    bool phys_stale = true, phys_any = false;   // (the document changed since the world was last looked at; some thing has physics)
    std::string phys_made;              // all the world was made of, as the document said it then (made again only when that changes)
    struct Scripted { double at = -1; int made = -1; std::vector<ScriptCopy> copies; };
    std::map<int, Scripted> scripted;
    struct AbsentKept { double at = -1; long changes = -1; int seen = -1; std::vector<AbsentCopy> copies; };
    std::map<int, AbsentKept> absentKept;
    std::map<int, std::pair<std::string, std::string>> drawingKept;   // (each freehand stroke's outline as AbsentMotion's drawing effects last made it, by what they were: doc_moment)   // (each thing's copies as AbsentMotion's repeat effects lay them, at the moment last asked: doc_absent_copies)
    std::shared_ptr<struct DocScripts> scripts;   // (its scripts, each run as its file says: doc_values.h)
    std::map<int, ThingFx> thingFx;     // the things with effects of their own (thing_fx.h): each drawn alone, its effects on, laid where it lies
    struct Read { Texture* seen = nullptr; RenderTarget* into = nullptr; };
    std::map<int, Read> reads;          // the things that read what is under them, each with its picture of it (doc_things_read)
    std::vector<int> beforeOf;          // whose each of the stage's `before` is
    ScreenStack screen;                 // the stage's whole-screen effects, its script .wgsl files one over another (screen.h)   // each thing's copies as its script said them, at one moment (worked out once a picture)
    bool phys_keyed = false;            // reading where a thing's keys put it (not where the world has it)
    bool writing = false;               // it is being written as a film: it is read, not changed, until that is done
    bool eyeAsked = false;              // the window looks by its working eye (After Effects' working 3D views, Alight Motion's view without the camera): never in the document
    double eyeIs[3] = {0.0, 0.0, 1.0};  // that eye round the world's middle: about, over, how far (as a camera's distance)
    bool working = false;               // the stage is seen by it at this moment: a camera is on, and no film is being written
    double written_from = 0.0;          // the moment of the document that a film being written begins at (its work area's start)
    int looking = 0;                    // the camera the stage is seen through now (0: the stage's own: doc_moment)
    double round = 8.0;                 // how long the document is, as last shown: waves, noise and drifts go round in it a whole number of times (a film loops)
    sqlite3_session* acting = nullptr;  // SQLite's own record of what the open act changes
    // A thing held by a hand (doc_hold): what of it is carried shows at once and is not yet the document's.
    struct Held { int thing = -1; std::map<std::string, double> from, tried; std::map<std::string, std::string> said; } held;   // (said: a word carried, a path's points moved)   // (-1: nothing held; 0 is the stage, which a hand may hold by a value of its own)
    struct Keys { std::vector<double> at, value; std::vector<Interp> interp; std::vector<std::string> own; std::string loop; };   // (own: a way defined in the document, where the stretch is not one of the nine kinds)
    struct NameHash { using is_transparent = void; size_t operator()(std::string_view n) const { return std::hash<std::string_view>{}(n); } };   // (a name looked up as it is said, no string made for it)
    std::unordered_map<int, std::unordered_map<std::string, Keys, NameHash, std::equal_to<>>> moving;   // a value's keys, and the way of the stretch after each, by thing and name; read from the facts when first asked for, forgotten at every change
    struct HeardWordHash { size_t operator()(const std::pair<int, const char*>& k) const { return std::hash<int>{}(k.first) * 31u ^ std::hash<const void*>{}(k.second); } };
    std::unordered_map<std::pair<int, const char*>, std::string, HeardWordHash> heardWords;   // (and its words so, and a thing's copy_tall as numbers)
    std::unordered_map<std::pair<int, const char*>, const Keys*, HeardWordHash> literals;   // where doc_place's own values have their keys, by thing and the name's place in the program (a literal: it never says another name): found once, forgotten at every change
    std::unordered_map<int, std::vector<double>> talls;
    std::map<int, std::pair<float2, float>> halves;   // a thing's half size and depth (doc_half), worked out once a moment: its copies all ask it
    std::map<std::pair<int, std::string>, std::string> words;   // a value's word, the same way
    std::map<std::pair<int, std::string>, std::vector<double>> keyed;   // the moments of a value's keys, the same way
    std::map<std::string, sqlite3_stmt*> named, goes;     // a way by its name, and by its expression: made ready once (SQLite works it out)
    struct Shown { std::string of; Video* film = nullptr; Drawn* drawn = nullptr; Solid* solid = nullptr; Mesh* shape = nullptr; bool in = true, clipped = false, camera = false, drivenLooks = false, behaviour = false, grains = false; std::shared_ptr<SplatEffect> splatFx; std::string splatFxFile; MaterialInstance* splatPlain = nullptr; bool splatDone = false; int run = -1, lies = -1;
                   utils::Entity light;   // (a light, Filament's own: in every scene of the stage while it is on)
                   double units = 1.0;    // (a flat thing's: points of the picture to a unit of the stage as it was made, which its effects are counted in)
                   int space = 3;
                   std::string words; bool wordsMove = false;   // (a text's words as made, and whether AbsentMotion's text effects change them as time goes: doc_text_shown)
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
    struct Along { double at = -1.0, key = 0.0; const void* of = nullptr; std::vector<float> each; };   // (a thing's copies laid on another's outline: where, worked out once a moment)
    std::map<int, Along> along;
    std::map<std::tuple<int, double, long long>, math::double3> tracks;   // (where a thing was at a tick of a trail's own grid: worked out once, forgotten at every change)
};
// What a thing may be told, by name: which kinds of thing have it, a number between which two (or one of which
// words), and whether it can move (take keys).
static const char* const doc_kinds = "film picture text solid shape camera light";   // (the kinds of thing)
static const char* const doc_drawn = "film picture text solid shape";          // (those that are seen)
static const char* const doc_relations = "around row follow push group";   // (the kinds of relation: a group is the plainest, things in an order and nothing else)
static const char* const doc_shapes = "ring gem cube ball";             // (the kit's own shapes a solid may be)
// (whether a value is keyed, in SQLite's words: see doc_key)
#define DOC_KEYED "(select count(*) > 1 or max(at) <> 0 or max(way) is not null from fact where thing = %d and name = %Q)"
struct DocValue { const char* name; const char* kinds; double low, high; bool moves; const char* words; bool open = false; };   // (open: low and high are how far its knob goes, more is taken typed, After Effects' slider and valid ranges, PF_FloatSliderDef)
static const DocValue doc_values[] = {
    {"x", "film picture text solid shape camera light behaviour", -1e4, 1e4, true, nullptr}, {"y", "film picture text solid shape camera light behaviour", -1e4, 1e4, true, nullptr}, {"z", doc_kinds, -1e4, 1e4, true, nullptr},   // (a group's own: how far its things are away when out)
    {"rotate", "film picture text solid shape camera light behaviour", -1e6, 1e6, true, nullptr}, {"rotate_x", "film picture text solid shape camera light behaviour", -1e6, 1e6, true, nullptr}, {"rotate_y", "film picture text solid shape camera light behaviour", -1e6, 1e6, true, nullptr},   // (a relation's own: added to each of its things in turn)
    {"orient", "film picture text solid shape", 0, 1, false, nullptr},   // (turned the way it is going along its path, its rotate added: After Effects' Auto-Orient)
    // (how many times as wide, as high and as deep as its size makes it, each on its own as After Effects' Scale: below 0
    //  turned over; a flat thing's scale_z is its thickness: none said, flat; said, its sheet is a box its picture is pushed
    //  through (deep.glsl), as After Effects' Extrusion Depth and Apple Motion's Depth make a flat text solid; 0: shut, a sheet)
    {"scale_x", doc_drawn, -1e4, 1e4, true, nullptr},
    {"scale_y", doc_drawn, -1e4, 1e4, true, nullptr},
    {"scale_z", "solid text shape picture", -1e4, 1e4, true, nullptr},
    // (falls and strikes others: moves, the world has it from its last key on; still, a wall where the document puts it;
    //  how much it bounces, how it rubs; and the stage's pull down, in units of the stage a second each second)
    {"physics", doc_drawn, 0, 0, false, "none moves still"},
    // (a recipe as code: a JavaScript file that says where each copy is (script.h); its head's knobs are values of the thing, named js_ and the knob's name)
    {"script", "picture text shape stage", 0, 0, false, ""},   // (the stage's: a whole-screen effect, a .wgsl file: screen.h)
    // (a thing's own effects, After Effects' effects on a layer: the whole-screen ones' .wgsl files, one over another, on the thing as the camera sees it)
    //  (a camera's: AbsentMotion's motions only, its place moved by them as AbsentMotion's camera is: doc_fx_say)
    {"fx", "film picture text shape solid camera", 0, 0, false, ""},
    {"fx_off", "film picture text shape solid camera", 0, 0, false, ""},   // (its effects turned off, the titles of their cards with ; between: Alight Motion's eye on an effect; their values kept)
    {"bounce", doc_drawn, 0, 1, false, nullptr},
    {"friction", doc_drawn, 0, 2, false, nullptr},
    {"gravity", "stage", -100, 100, false, nullptr},
    {"follow", "stage", 0, 1e9, false, nullptr},
    // (the thing the stage's effects follow in place of a mouse: their pointer, center, mouse or point knobs are where it is on the picture; 0: none)
    {"size", "film picture text solid shape behaviour", 1e-3, 1e4, true, nullptr}, {"anchor_x", doc_drawn, -100, 100, true, nullptr}, {"anchor_y", doc_drawn, -100, 100, true, nullptr}, {"anchor_z", doc_drawn, -100, 100, true, nullptr},
    // (a camera, the past repository's: it looks at where it is, from round about it as it is turned, so far away, so near)
    {"distance", "camera", 0.01, 100, true, nullptr}, {"zoom", "camera", 0.01, 100, true, nullptr},
    // (and its lens: how far off from it things are sharp, 0 being where it looks; the f-number, 0: all sharp)
    {"focus", "camera", 0, 1e4, true, nullptr}, {"aperture", "camera", 0, 64, true, nullptr}, {"blur_level", "camera", 0, 100, true, nullptr},   // (After Effects' blur level: how many times the lens's own blur)
    {"opacity", "film picture text shape behaviour", 0, 1, true, nullptr},
    {"clip", doc_drawn, 0, 1, false, nullptr},
    // (clipped by a flat thing before it, the kit's matte: by how clear that is or how light (After Effects' alpha and
    //  luma mattes), the other way about (inverted), its edge so soft, in points of the screen)
    {"group_mask", "shape", 0, 0, false, "none mask exclude"},   // (a group made to mask: its top thing the shape the others in it are seen in, or cut out by: Alight Motion's Group and Mask, Group and Exclude, reports/alight_ui.md)
    {"clip_matte", "film picture text shape", 0, 0, false, "alpha luma"}, {"clip_invert", "film picture text shape", 0, 1, false, nullptr}, {"clip_feather", "film picture text shape", 0, 500, false, nullptr},   // (a solid is cut by a box, not a matte)
    {"solo", doc_kinds, 0, 1, false, nullptr}, {"lock", doc_kinds, 0, 1, false, nullptr},   // (After Effects' own: while any thing is solo only those are seen; a locked thing is not changed)
    {"visible", "film picture text solid shape camera light behaviour", 0, 1, false, nullptr}, {"order", doc_kinds, -1e9, 1e9, false, nullptr}, {"wide", "shape", 1e-3, 1e3, false, nullptr}, {"shape", "shape", 0, 0, false, "rect circle solid polygon star line wave null ring arc arrow squircle cog path spiral chamfer"},
    {"points", "shape", 1, 360, false, nullptr}, {"inner", "shape", 0.01, 1, false, nullptr}, {"round", "shape picture", 0, 0.5, true, nullptr}, {"crop_left", "picture", 0, 0.95, false, nullptr}, {"crop_top", "picture", 0, 0.95, false, nullptr}, {"crop_right", "picture", 0, 0.95, false, nullptr}, {"crop_bottom", "picture", 0, 0.95, false, nullptr},   // (a photo cut in from each side, a share of its width or height: Alight Motion's crop)   // (a polygon's or star's points, a star's inner points, its corners rounded)
    {"start", doc_kinds, 0, 1e6, false, nullptr}, {"duration", doc_kinds, 1e-3, 1e6, false, nullptr},
    {"begin", "film picture text shape", 0, 1e6, false, nullptr}, {"speed", "film solid picture text shape", -100, 100, false, nullptr}, {"keyed", "film", 0, 1, false, nullptr},
    {"poster_at", "stage", -1, 1e6, false, nullptr},   // (the moment the document's own picture is drawn at, seconds; below 0: where the playhead is as it is left: Alight Motion's Set Project Thumbnail, motolii_poster)
    {"layer_length", "stage", 0, 3600, false, nullptr},   // (how long a thing put in lasts, seconds; 0: to the document's end: AbsentMotion's New Layer Length, doc_add)
    {"bpm", "stage", 0, 999, false, nullptr},   // (the document's tempo, beats a minute; 0: none. What the timeline's grid goes by: Live's Tempo field; Alight Motion has none)
    {"bpm_from", "stage", 0, 1e6, false, nullptr},   // (where a bar begins, seconds: Live's Set 1.1.1 Here)
    {"frame_rate", "film picture text solid shape stage", 0, 60, false, nullptr},   // (the stage's: the document's frame rate, all of it seen only at its frames, kit_things.h time_is)
    {"speed_hold", "film solid picture text shape", 0, 0, false, "start now end"},   // (where its Speed holds it in time, as After Effects' Time Stretch: its start, the playhead, its end)   // (its own frames a second, 0 as the stage's: doc_stepped)
    {"volume", "film", 0, 2, true, nullptr},   // (how loud its sound is heard and written: 1 as it comes, 0 none, 2 twice; Alight Motion's Volume, 0 to 200%)
    {"beats", "film", 0, 60, false, nullptr},   // (where its sound strikes: louder by so many dB at once; 0, none shown)
    {"blend", "film picture text shape", 0, 0, false, "normal add multiply screen darken lighten overlay soft_light hard_light color_dodge color_burn difference exclusion hue saturation color luminosity linear_burn linear_light vivid_light pin_light darker_color lighter_color subtract divide"}, {"file", "film picture solid text", 0, 0, false, ""}, {"shape", "solid", 0, 0, false, doc_shapes},
    {"text", "text", 0, 0, false, ""}, {"fill", "text solid shape light", 0, 0, false, ""}, {"fill_media", "shape", 0, 0, false, ""},
    {"marker", "picture", 0, 0, false, ""},   // (an After Effects file: which of its named stretches it plays, over and over; all: the whole of it)
    {"name", "film picture text solid shape camera light behaviour", 0, 0, false, ""},   // (what it is called where things are listed, a behaviour too; not said: by what it is)
    {"label", doc_kinds, 0, 0, false, "auto rose orange yellow lime green mint aqua sky blue periwinkle orchid magenta red tangerine azure teal"},   // (its colour on the timeline, chosen: Ableton Live's clip colours in the window's order; auto, one by its number)
    {"element", doc_kinds, 0, 0, false, ""},   // (the element it was put in from, linked: its file as a look says it, {own}/elements/...; doc_element_add, Alight Motion's Element)
    {"retime", doc_kinds, 0, 0, false, "freeze stretch loop loop_stretch blank"}, {"retime_in", doc_kinds, 0, 1e6, false, nullptr}, {"retime_out", doc_kinds, 0, 1e6, false, nullptr}, {"element_length", doc_kinds, 0, 1e6, false, nullptr},   // (an element's Re-timing: doc_element_clock)
    {"element_at", doc_kinds, -1, 1e12, false, nullptr}, {"element_part", doc_kinds, 0, 1e9, false, nullptr}, {"element_text", "text", 0, 0, false, ""},   // (what an element put in keeps of where it came from: doc_element_marks)
    {"note", doc_kinds, 0, 0, false, ""},   // (what a maker wrote of it for whoever opens the document next: After Effects' layer comment)
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
    {"parent", doc_kinds, 0, 1e9, false, nullptr},
    {"parent_rotate", doc_kinds, 0, 2, true, nullptr}, {"parent_size", doc_kinds, 0, 2, true, nullptr},   // (how much of its parent's turn and size it takes: 1 all, 0 none, 2 twice; Alight Motion's Parenting Helper weights)
    {"scale_assist", doc_drawn, 0, 0, false, "y x both"}, {"scale_assist_amount", doc_drawn, 0, 100, true, nullptr}, {"scale_assist_damp", doc_drawn, 1, 10, true, nullptr},   // (Alight Motion's Scale Assist, as scale_x and scale_y: grown one way alone, or wider and as much thinner as a squashed ball keeps its bulk)
    {"roll", doc_kinds, 0, 0, false, "x y"}, {"roll_radius", doc_kinds, -1e3, 1e3, true, nullptr},   // (turned as a wheel rolling as it goes along x, or y; its radius half its smaller side and this more)
    {"dissolve", "solid", 0, 100, true, nullptr}, {"grain", "solid", 0.05, 20, true, nullptr},   // (a point cloud's: its grains thrown out each its own way, so far; and how large they are drawn: points.h)
    {"mosh", "film", 0, 1, false, nullptr}, {"mosh_with", "film", 0, 1e9, false, nullptr}, {"mosh_at", "film", 0, 1e6, false, nullptr},   // (a real datamosh: from that moment of its film, the other's motion poured into its last picture: mosh.h)   // (the thing it goes with: it is moved, turned and sized as that one is, then as it is itself; 0: none)
    {"space", "film picture text shape", 0, 0, false, "2d 2.5d 3d"},   // (the past repository's three tags: 2d is not in the world and lies by the order alone; 2.5d is a sheet in the world, before or behind by depth, always facing the eye; 3d is filmed as it is, the camera turning round it, as a solid is)
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
    {"copies", "picture text shape solid", 1, 1000, false, nullptr}, {"copy_x", "picture text shape solid", -1e4, 1e4, true, nullptr}, {"copy_y", "picture text shape solid", -1e4, 1e4, true, nullptr},
    {"copy_rotate", "picture text shape solid", -1e6, 1e6, true, nullptr}, {"copy_size", "picture text shape solid", 0.01, 100, true, nullptr},
    {"copy_columns", "picture text shape solid", 0, 100, false, nullptr},   // (so many to a row, then the next row: copy_x along it, copy_y down; 0: one row)
    {"stagger", "picture text shape solid", 0, 1e3, false, nullptr},   // (how much later the last of its copies moves than the first, in seconds: each as late as its order says)
    {"stagger_from", "picture text shape solid", 0, 0, false, "start centre end edges"},   // (which of them moves first: the first, the middle one, the last, the outer ones)
    {"copy_fill", "picture text shape", 0, 0, false, ""},   // (its copies coloured from the first colour to the last, in their order: six digits each; one, all in it; none, not coloured each)
    {"copy_fill_way", "picture text shape", 0, 0, false, "ramp cycle"},   // (from the first to the last across them all, or one colour each in turn, round and round)
    {"copy_wide", "picture text shape solid", 0.01, 100, true, nullptr},   // (how many times as wide its last copy is as its first, those between in step: rows of waves, each with more of them)
    {"copy_z", "picture text shape solid", -1e4, 1e4, true, nullptr},   // (its copies so far apart towards the eye, in units of the stage: a row going into the picture)
    {"copy_rotate_x", "picture text shape solid", -1e6, 1e6, true, nullptr},   // (each copy turned so much more than the one before about the level, towards the eye: a fan of sheets in depth)
    {"copy_rotate_y", "picture text shape solid", -1e6, 1e6, true, nullptr},   // (and about the upright: sheets turning round like pages)
    {"wave_z", "picture text shape solid", -1e3, 1e3, true, nullptr},   // (and waved so far towards the eye and away, a quarter round after up and down: with wave_y, round and round, a helix)
    {"scatter_z", "picture text shape solid", 0, 1e4, true, nullptr},   // (scattered so far towards the eye and away)
    {"copy_turn", "picture text shape solid", -1e5, 1e5, true, nullptr},   // (how far its copies turn in all, spread evenly over them: 360 fills a round; copy_rotate's turn each on top)
    {"tall", "picture text shape", 0, 1e3, true, nullptr},   // (how many times as tall as it is, its anchor staying where it is: a bar growing up from its foot)
    {"copy_tall", "picture text shape solid", 0, 0, false, ""},
    {"copy_fill_at", "picture text shape", 0, 1, true, nullptr},   // (where along copy_fill each copy's colour is, given by a behaviour (copy_fill_at_by): 0 its first, 1 its last; not given, their order)   // (and its copies each so many times as tall again, in their order, going round: numbers, a bar chart's data; none: all as tall)
    {"copy_opacity_end", "picture text shape", 0, 1, true, nullptr},   // (the last copy as clear as this, the first as the thing: After Effects' Repeater, End Opacity)
    // (its copies laid on another shape's outline, each turned the way it goes, Cavalry's Duplicator on a shape; and slid along it, a share of its length)
    {"copy_on", "picture text shape solid", 0, 1e9, false, nullptr},
    {"copy_on_offset", "picture text shape solid", -1e4, 1e4, true, nullptr},
    {"connect", "shape", 0, 1e9, false, nullptr},   // (the thing whose copies its own copies join, one each from one to another: lines between dots; 0: none)
    {"connect_to", "shape", 0, 1e9, false, nullptr},   // (and another: the first's nth copy joined to this one's nth, rungs between two strands; 0: within the first)
    {"connect_reach", "shape", 0, 1e4, true, nullptr},
    // (put together with another shape, Cavalry's Boolean: its own less that one, where both are, either, either but not both)
    {"boolean", "shape", 0, 0, false, "union subtract intersect exclude"},
    {"boolean_with", "shape", 0, 1e9, false, nullptr},   // (that other shape, seen where it lies; 0: none)
    // (roughened, Illustrator's Roughen: so far out (a share of its height), a point so often (the same))
    {"resample", "shape", 0, 2048, true, nullptr},   // (walked again at so many points an outline, straight between, Cavalry's Resample Path: a round a polygon; 0: as made)
    {"voxel", "shape", 0, 1, true, nullptr},   // (made of squares so large (a share of its height), Cavalry's Voxelize; 0: as it is)
    {"rough", "shape", 0, 0.5, true, nullptr},
    {"rough_detail", "shape", 0.005, 1, true, nullptr},
    // (turned into another shape so far, Cavalry's Morph: 0 itself, 1 the other where it lies)
    {"fill_rule", "shape", 0, 0, false, "nonzero evenodd"},
    // (a behaviour, Cavalry's: a value it gives each copy of what it is connected to (`<name>_by` = it), by the copy's number
    //  and the moment: from minimum to maximum in their order (stagger), waving (oscillator: so many times a second, offset
    //  a share of a wave, stagger degrees of a wave between copies), a smooth noise, or at random by its seed)
    {"minimum", "behaviour", -1e6, 1e6, true, nullptr},
    {"maximum", "behaviour", -1e6, 1e6, true, nullptr},
    {"frequency", "behaviour", 0, 1000, true, nullptr},
    {"offset", "behaviour", -1e6, 1e6, true, nullptr},
    {"tilt", "behaviour", -1e6, 1e6, true, nullptr},
    {"stagger", "behaviour", -1e6, 1e6, true, nullptr},
    {"seed", "behaviour", 0, 1e9, false, nullptr},
    {"step", "behaviour", 0, 1e6, true, nullptr},   // (what it gives rounded to so many: 90, a quarter turn at a time; 0 not)
    // (where it works, Cavalry's falloff: inside a shape (any shape, seen or not, moving or not) all of it, outside none, between over
    //  falloff_soft units of the stage; inverted, the other way: elsewhere, the value is the thing's own)
    {"falloff", "behaviour", 0, 1e9, false, nullptr},
    {"falloff_soft", "behaviour", 0, 1e4, true, nullptr},
    {"falloff_invert", "behaviour", 0, 1, false, nullptr},   // (where its outlines lie over each other: filled, or a hole every other time, SVG's and Cavalry's fill rule)
    // (a relation, a behaviour that sets its things in relation to each other or to one of them: around a middle, in a row,
    //  following a leader (so many seconds later each, its own value as a step from the leader's; handed over to leader_b as
    //  hand goes 0 to 1), or pushed away by a leader within radius; strength how much of it works: 0 as if it were not there,
    //  so it is put on and taken off over time without a jump)
    {"strength", "behaviour", 0, 1, true, nullptr},
    {"radius", "behaviour", 0, 1e4, true, nullptr},
    {"gap", "behaviour", -1e4, 1e4, true, nullptr},
    {"centre_x", "behaviour", -1e6, 1e6, true, nullptr},
    {"centre_y", "behaviour", -1e6, 1e6, true, nullptr},
    {"leader", "behaviour", 0, 1e9, false, nullptr},
    {"leader_b", "behaviour", 0, 1e9, false, nullptr},
    {"hand", "behaviour", 0, 1, true, nullptr},
    {"lag", "behaviour", 0, 60, true, nullptr},
    {"spread", "behaviour", -10, 10, true, nullptr},   // (a group's: its things as far from their middle as they were put, times this: 0 all at the middle, 2 twice as far out)
    {"scatter", "behaviour", -10, 10, true, nullptr},   // (a group's: each thing so far away in its own fixed way, as a seed scatters)
    {"members", "behaviour", 0, 0, false, ""},   // (a relation's things in their order, by number, "_" where one was taken out: its place kept, so no other moves)
    {"under", "behaviour", 0, 0, false, ""},   // (for each of them, the relation it was in before, 0 none: this one starts from what that one gives)
    {"from", "behaviour", 0, 0, false, ""},   // (for each of them, x and y where it showed when it came in: an around or a row adds only the step from there to its place, a follow only how far its leader has gone since)
    {"morph", "shape", 0, 1, true, nullptr},
    // (its path another's track, Cavalry's Trails: whose, and over how many seconds back)
    {"trail", "shape", 0, 1e9, false, nullptr},
    {"trail_length", "shape", 0.001, 60, true, nullptr},
    {"morph_to", "shape", 0, 1e9, false, nullptr},
    {"morph_steps", "shape", 0, 64, false, nullptr},   // (and so many steps from itself to it all at once, a blend; 0 or 1: none)
    {"drawing", "shape", 0, 0, false, ""},   // (freehand strokes as they were drawn, ";" between: each its tool (0 pen, 1 brush, 2 fill, 3 eraser), how wide, then each point x y and its pressure, in the path's own units: doc_brush, made by AbsentMotion)
    {"path", "shape", 0, 0, false, ""},   // (a path shape's own path, drawn by a pen: SVG's path data, in its height from its middle, down as SVG goes)   // (only those nearer than this, in units of the stage, finer the further apart: a plexus; 0: each to the next in their order)
    // (a wave through it and its copies, Cavalry's oscillator: so far across and up (units), so far turned (degrees),
    //  so much larger (a share), so many times a second, each copy so far behind the one before (turns spread over all)
    {"wave_x", "picture text shape solid", -1e3, 1e3, true, nullptr}, {"wave_y", "picture text shape solid", -1e3, 1e3, true, nullptr},
    {"wave_rotate", "picture text shape solid", -1e4, 1e4, true, nullptr}, {"wave_size", "picture text shape solid", -1, 10, true, nullptr},
    {"wave_speed", "picture text shape solid", 0, 100, true, nullptr}, {"wave_phase", "picture text shape solid", -100, 100, true, nullptr},
    // (a field over its copies, Cavalry's falloff: a round place on the stage (its middle, units; how far it reaches); a
    //  copy within it is so much larger and so far turned, most at its middle, none at its edge)
    {"field_x", "picture text shape", -1e4, 1e4, true, nullptr}, {"field_y", "picture text shape", -1e4, 1e4, true, nullptr}, {"field_radius", "picture text shape", 0, 1e4, true, nullptr},
    {"field_size", "picture text shape", -1, 20, true, nullptr}, {"field_rotate", "picture text shape", -1e4, 1e4, true, nullptr},
    {"field_push", "picture text shape", -1e3, 1e3, true, nullptr},   // (and pushed so far away from its middle, in units of the stage; below 0: drawn in)
    {"field_band", "picture text shape", 0, 1e4, true, nullptr},   // (a ring so wide, strongest at its radius from its middle and nothing so far either side: a band of large dots going out; 0: a round, strongest in its middle)
    {"field_z", "picture text shape", -1e4, 1e4, true, nullptr},   // (where its middle is towards the eye: the field is a ball, near and far measured too)
    {"wave_noise", "picture text shape solid", 0, 1, true, nullptr},   // (how much of the wave is a noise of each copy's own instead of a sine: Cavalry's noise; 1 all noise)
    {"noise_scale", "picture text shape", 0, 1e3, true, nullptr},   // (that noise read where each copy lies, so near ones move alike: how large its blobs are, in units of the stage; 0: each copy its own)
    // (its copies scattered, each its own way by a seed: up to so far across and up (units), so far turned (degrees), so
    //  much larger or smaller (a share); another seed, another scattering)
    {"scatter_x", "picture text shape solid", 0, 1e4, true, nullptr}, {"scatter_y", "picture text shape solid", 0, 1e4, true, nullptr},
    {"scatter_rotate", "picture text shape solid", 0, 1e4, true, nullptr}, {"scatter_size", "picture text shape solid", 0, 1, true, nullptr},
    {"seed", "picture text shape", 0, 1e6, false, nullptr},
    {"drift_x", "picture text shape", -1e3, 1e3, true, nullptr}, {"drift_y", "picture text shape", -1e3, 1e3, true, nullptr},   // (scattered copies flowing so far a second, units of the stage: past the scattering's edge, back in at the other)
    {"tint", "film picture text shape", 0, 1, true, nullptr}, {"tint_black", "film picture text shape", 0, 0, false, ""}, {"tint_white", "film picture text shape", 0, 0, false, ""},
    {"tritone", "film picture text shape", 0, 1, true, nullptr}, {"tritone_shadow", "film picture text shape", 0, 0, false, ""}, {"tritone_midtone", "film picture text shape", 0, 0, false, ""}, {"tritone_highlight", "film picture text shape", 0, 0, false, ""},   // (how its lines lie one under another, ThorVG's own: 0 at the left, 0.5 about the middle, 1 at the right)
    {"stroke", "text shape picture", 0, 0, false, ""}, {"stroke_width", "text shape picture", 0, 1000, false, nullptr},   // (its edge: a colour, and how wide, in points of the picture at size 1)
    // (ThorVG's own: a text's letters and lines so many times as far apart as the font sets them, its slant; an edge's
    //  ends and corners, and dashes along it so long, with gaps as long, begun so far along)
    {"wrap", "text", 0, 0, false, "none character word smart ellipsis vertical"}, {"wrap_at", "text", 0, 1000, false, nullptr},   // (its lines broken in a box so many times as wide as its letters are high: ThorVG's layout and wrap; a text's size is the height of all its lines, so this goes with the letters, not the picture)
    {"split", "text", 0, 0, false, "none letters words"},   // (one line of it split into its letters or its words, each a copy of its own: they lie where they are, and move, wave and are coloured each in their order)
    {"letter_spacing", "text", 0, 10, false, nullptr}, {"line_spacing", "text", 0, 10, false, nullptr}, {"italic", "text", 0, 0.5, false, nullptr},
    // (a fill of two colours or more: across, ThorVG's linear gradient, or from the middle out, its radial; turned so far,
    //  in degrees clockwise, 0 from top to bottom; so long, as a share of how far it reaches; past its ends padded,
    //  reflected or repeated)
    {"gradient", "text shape", 0, 0, false, "linear radial"}, {"gradient_angle", "text shape", -1e6, 1e6, false, nullptr},
    {"gradient_scale", "text shape", 0.01, 100, false, nullptr}, {"gradient_spread", "text shape", 0, 0, false, "pad reflect repeat"},
    // (how the colours between its stops are mixed: CSS Color 4's color-interpolation-method, drawn_made.h ramp_of)
    {"gradient_blend", "text shape", 0, 0, false, "rgb oklab hue steps"},
    {"cap", "shape", 0, 0, false, "butt round square"}, {"join", "shape", 0, 0, false, "miter round bevel"},
    // (arrow heads on its edge's open ends, Cavalry's basicArrowHead: which ends, and so many times its edge long)
    {"heads", "shape", 0, 0, false, "none end start both"},
    {"head_size", "shape", 0.5, 40, false, nullptr},
    {"head_shape", "shape", 0, 0, false, "arrow circle box chevron bar t_line square square_filled diamond diamond_filled arrow_hollow arrow_filled circle_hollow circle_filled teardrop_hollow teardrop_filled"}, {"head_start", "shape", 0, 0, false, "arrow circle box chevron bar t_line square square_filled diamond diamond_filled arrow_hollow arrow_filled circle_hollow circle_filled teardrop_hollow teardrop_filled"},
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
    {"fog", "stage camera", 0, 1, false, nullptr}, {"fog_color", "stage camera", 0, 0, false, ""}, {"fog_near", "stage camera", 0, 1e4, false, nullptr}, {"fog_far", "stage camera", 0, 1e4, false, nullptr}, {"render_height", "stage", 0, 8640, false, nullptr}, {"render_rate", "stage", 0, 60, false, nullptr}, {"render_mbps", "stage", 0, 500, false, nullptr}, {"render_blur", "stage", 0, 64, false, nullptr}, {"render_alpha", "stage", 0, 1, false, nullptr},   // (how a film is written: its height, pictures a second, megabits a second, moments laid on one another for a motion blur; 0 as it was)   // (how far from the camera it begins, in the world's units: Filament's FogOptions distance)   // (fog: the stage's, or the camera's seen through, as Alight Motion's camera has its Fog)
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
struct DocCard { const char* title; const char* names; const char* when = nullptr; const char* is = nullptr; bool added = false; };   // (shown only while `when` is `is`; "!...": is not; "~...": has it in it; added: only once one of its values is said or keyed, or the maker asked for it by More: not one of the tiles Alight Motion shows for the kind, doc_cards_of)
// How a value is shown to a hand (a shell draws its knob by this, nothing of its own): its look (count: a whole number;
// onoff; angle; stage: a length of the stage, shown in points of the picture; down: one up the stage, shown down it as the picture counts; share: 0 to 1, shown in hundredths;
// colour, colours, numbers: words of those; words: any, typed; file: a file, given from a shelf; thing: another thing; seed; none: as its range says), the unit it is read
// in, a name in plain words, the span its knob covers (the value itself may go further, as its range says) and
// the name of the fold it lies in on its card, if it is folded away (named by what the fold holds; a card's first
// row holds eight at most, as on Ableton's Push).
struct DocShown { const char* name; const char* look; const char* unit; const char* label; double low, high; const char* fold; bool open = false; };
