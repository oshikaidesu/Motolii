// A MIDI file's piano roll as SVG text (midi_roll.cpp: bladeSk's PianoRollSvg.js in QuickJS, ALMaMPlayer's look).
#pragma once
#include <string>
#include <utility>
#include <vector>

// bpm: the file's beats played at it (0: the file's own). "" and `why` if it cannot be made.
std::string midi_roll(const std::string& file, double bpm, std::string& why);
// The same, kept in ~/Library/Caches/Motolii/midi (named by the file, when it was last changed, and the bpm): its path.
std::string midi_roll_file(const std::string& file, double bpm, std::string& why);
// How large a roll is (its viewBox): points of it, 300 a second, its notes 18 high.
bool midi_roll_box(const std::string& svg, double& w, double& h);
// Its notes in the order they are drawn ("note<i>"): when each comes on and goes off, in seconds of the roll.
std::vector<std::pair<double, double>> midi_roll_notes(const std::string& svg);
