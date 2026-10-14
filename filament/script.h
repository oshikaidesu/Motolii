// A recipe as code: a JavaScript file whose head declares its knobs in AviUtl2's manner, one a line,
//   // --track@count:Count,1,5000,800,1      a number: name:label,lowest,highest,as it comes,step
//   // --check@spin:Spin,1                    on or off (1 or 0)
//   // --select@way:Way,ring,grid,spiral      one of some words, the first as it comes
// and whose body says, for each copy of the thing it is given to, where it is and how:
//   function each(i, n, t, k) { return { x, y, z, size, rotate }; }   // i of n copies, t seconds, k the knobs
// (as Cavalry's JavaScript Utility gives a Duplicator its points). Behind the door: QuickJS-ng (js_bench chose it).
#pragma once
#include <map>
#include <memory>
#include <string>
#include <vector>

struct ScriptKnob {
    std::string name, label, kind;   // kind: track, check or select
    double low = 0, high = 1, usual = 0, step = 0;
    std::string words;               // a select's words, with spaces between
};
struct ScriptCopy { double x = 0, y = 0, z = 0, size = 1, rotate = 0; };

// The knobs a file declares (read from its head; empty if it has none or cannot be read).
std::vector<ScriptKnob> script_knobs(const std::string& file);

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
    struct Own;
    std::unique_ptr<Own> own;
};
