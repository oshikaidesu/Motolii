// A recipe as code: a JavaScript file whose head declares its knobs in AviUtl2's manner, one a line,
//   // --track@count:Count,1,5000,800,1      a number: name:label,lowest,highest,as it comes,step
//   // --check@spin:Spin,1                    on or off (1 or 0)
//   // --select@way:Way,ring,grid,spiral      one of some words, the first as it comes
// and whose body says, for each copy of the thing it is given to, where it is and how:
//   function each(i, n, t, k) { return { x, y, z, size, rotate, rotate_x, rotate_y, opacity }; }   // i of n copies, t seconds since the thing began, k the knobs (and k.duration: how long the thing lasts)
// (as Cavalry's JavaScript Utility gives a Duplicator its points). Behind the door: QuickJS-ng (js_bench chose it).
#pragma once
#include <map>
#include <memory>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct ScriptKnob {
    std::string name, label, kind;   // kind: track, check, select or picture (an effect's image read from another thing)
    double low = 0, high = 1, usual = 0, step = 0;
    std::string words;               // a select's words, with spaces between
};
struct ScriptCopy { double x = 0, y = 0, z = 0, size = 1, rotate = 0, rotate_x = 0, rotate_y = 0, opacity = 1; };

// The files a word names in turn, with spaces between (an effect's list, the stage's): a path (a word with a "/" in it) is one to
// its .wgsl, .fs, .js or .mjs, spaces and all ("Application Support", an ISF effect's "Bad TV.fs"); any other word stands alone.
inline std::vector<std::string> script_files(const std::string& word) {
    std::vector<std::string> all;
    std::istringstream each(word);
    const auto ends = [](const std::string& w) { for (const char* e : {".wgsl", ".fs", ".js", ".mjs"}) if (w.size() >= strlen(e) && w.compare(w.size() - strlen(e), std::string::npos, e) == 0) return true; return false; };
    for (std::string w; each >> w;) {
        if (!all.empty() && all.back().find('/') != std::string::npos && !ends(all.back())) all.back() += " " + w;
        else all.push_back(w);
    }
    return all;
}

// The knobs a file declares (read from its head; empty if it has none or cannot be read).
std::vector<ScriptKnob> script_knobs(const std::string& file);

// How far an ease of anime.js (third_party/anime, MIT) has gone at u, 0 to 1: `said` as anime.js says it ("outElastic(1, .3)",
// "spring({ bounce: .5 })", "linear(0, '0.25 75%', 1)", "steps(4)", "irregular(10, 1)"); `bundle` is where its file is.
// Not an ease, or anime.js not there: u, and `trouble` says why.
double anime_ease(const std::string& bundle, const std::string& said, double u, std::string* trouble);

class Script {
public:
    explicit Script(const std::string& file);   // made once a file: its code run, its each() kept
    ~Script();
    bool ok() const;
    const std::string& trouble() const;
    // Each copy at a moment, the knobs as the document says them now (a select by the word it is).
    ScriptCopy each(int i, int n, double t, const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words);
    // Every copy at once: each() for each, or, for a file with no each(), the whole file run with its knobs as globals
    // (n0, n1...), time and frame, giving a Canvalry PointCloud whose points are in pixels (`unit` stage units a pixel).
    std::vector<ScriptCopy> copies(int n, double t, const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words, double unit);
private:
    void knobs(const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words);   // (told the script's knobs object)
    ScriptCopy ask(int i, int n, double t);   // (each() with the knobs as last told)

    struct Own;
    std::unique_ptr<Own> own;
};
